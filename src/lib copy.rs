//! Taurusqlite is a plugin for Tauri that provides a light integration of Rusqlite
//!
use std::{path::Path, collections::HashMap, sync::Mutex};
use rusqlite::{Connection, Statement, types::{ValueRef, Value}, Row, Params, params_from_iter};
use serde::{ser::Serializer, Serialize, Deserialize};
use serde_json::{json, Value as JsonValue};
use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, 
  Runtime, State,
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Rusqlite(rusqlite::Error),
    #[error("{0}")]
    RusqliteT(String),
    #[error("Connection failed to {0}")]
    ConnectionFailed(String),
    #[error("Not connected to {0}")]
    NotConnected(String),
}

// Pass error through API
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self.to_string().as_ref())
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Open connection on SQLite database
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite")?;
/// ```
/// ## Failure
/// Panics on error
/// 
fn get_conn(path_to_db: &str) -> Result<Connection> {
    let database_path = Path::new(path_to_db);
    match Connection::open(database_path) {
        Ok(conn) => Ok(conn),
        Err(_) => Err(Error::ConnectionFailed(path_to_db.to_string()))
    }
}

/// Prepare statement from SQL query
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite");
/// let mut stmt = prepare(&conn, "SELECT * FROM person");
/// ```
/// 
/// ## Failure
/// Panics on error
/// 
fn prepare<'a>(conn: &'a Connection, query: &str) -> Result<Statement<'a>> {
    match conn.prepare(query) {
        Ok(stmt) => Ok(stmt),
        Err(e) => Err(Error::Rusqlite(e))
    }
}

/// Get columns name from statement
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite");
/// let mut stmt = prepare(&conn, "SELECT * FROM person");
/// let names = get_columns_names(&stmt);
/// ```
/// 
fn get_columns_names(stmt: &Statement) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for name in stmt.column_names() {
        result.push(name.to_string());
    }
    result
}

/// Parse column data from result
/// 
fn parse_column_data(value_to_parse: ValueRef) -> JsonValue {
    match value_to_parse {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => json!(std::str::from_utf8(t).unwrap()),
        ValueRef::Blob(b) => json!(b)
    }
}

/// Parse row from result
/// 
fn parse_row(names: &[String], row: &Row) -> HashMap<String, JsonValue> {
    let mut parsed_row: HashMap<String, JsonValue> = HashMap::new();
    for name in names.iter() {
        match row.get_ref(name.as_str()) {
            Ok(column_ref) => parsed_row.insert(name.to_owned(), parse_column_data(column_ref)),
            Err(e) => panic!("{:?}", e)
        };
    }
    parsed_row
}

/// Parse params from JSON format to Rusqlite
fn parse_params(params: Vec<JsonValue>) -> Vec<Value> {
    let mut parsed: Vec<Value> = Vec::new();
    for p in params {
        if p.is_null() {
            parsed.push(Value::Null);
        } else if p.is_string() {
            parsed.push(Value::Text(p.as_str().unwrap().to_string()));
        } else if p.is_i64() {
            parsed.push(Value::Integer(p.as_i64().unwrap()));
        } else if p.is_boolean() {
            parsed.push(Value::Integer(p.as_bool().unwrap() as i64));
        }
        // @TODO Blob to implements
    }
    parsed
}

/// Query the database
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite");
/// let persons = select_query(&conn, "SELECT * FROM student WHERE firstname = ?1", ["Bob"]);
/// ```
/// 
fn select_query<P: Params>(conn: &Connection, query: &str, params: P) -> Result<Vec<HashMap<String, JsonValue>>> {
    let mut result: Vec<HashMap<String, JsonValue>> = Vec::new();
    match prepare(conn, query) {
        Ok(mut stmt) => {
            let names = get_columns_names(&stmt);
            let rows_result = stmt.query(params);
            if let Ok(mut rows) = rows_result {
                loop {
                    let row = match rows.next() {
                        Ok(row_opt) => match row_opt {
                            Some(row) => row,
                            None => break
                        },
                        Err(e) => {
                            return Err(Error::Rusqlite(e));
                        }
                    };
                    result.push(parse_row(&names, row));
                }
            }
            Ok(result)
        },
        Err(e) => Err(e)
    }
}

/// Execute a query
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite");
/// let success = execute_query(&conn, "DELETE FROM student WHRE id = ?1", [1]);
/// ```
/// 
fn execute_query<P: Params>(conn: &Connection, query: &str, params: P) -> Result<bool> {
    println!("{:?}", query);
    match prepare(conn, query) {
        Ok(mut stmt) => {
            match stmt.execute(params) {
                Ok(_) => Ok(true),
                Err(e) => Err(Error::Rusqlite(e))
            }        
        },
        Err(e) => Err(e)
    }
}

#[derive(Default)]
struct DbInstances (
    Mutex<HashMap<String, Connection>>
);

#[derive(Deserialize)]
struct OpenOptions {
    disable_foreign_keys: Option<bool>
}

#[tauri::command]
fn open(state: State<'_, DbInstances>, db_path: String, options: OpenOptions) -> Result<bool> {
    match get_conn(&db_path) {
        Ok(conn) => {
            if let Some(disable_foreign_keys) = options.disable_foreign_keys {
                if disable_foreign_keys == true {
                    _ = execute_query(&conn, "PRAGMA foreign_keys = 0", []);
                }
            }
            state.0.lock().unwrap().insert(db_path, conn);
            Ok(true)
        }    
        Err(error) => Err(error)
    }
}

#[tauri::command]
fn set_pragma(state: State<'_, DbInstances>, db_path: String, key: String, value: JsonValue) -> Result<bool> {
    let mut mutex_map = state.0.lock().unwrap();
    let conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    execute_query(&conn, format!("PRAGMA {} = {}", key, value).as_str(), [])
}

#[tauri::command]
fn select(state: State<'_, DbInstances>, db_path: String, query: String, params: Vec<JsonValue>) -> Result<Vec<HashMap<String, JsonValue>>> {
    let mut mutex_map = state.0.lock().unwrap();
    let conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    select_query(&conn, query.as_str(), params_from_iter(parse_params(params)))
}

#[tauri::command]
fn execute(state: State<'_, DbInstances>, db_path: String, query: String, params: Vec<JsonValue>) -> Result<bool> {
    let mut mutex_map = state.0.lock().unwrap();
    let conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    if params.len() > 0 {
        if params.get(0).unwrap().is_array() {
            match conn.transaction() {
                Ok(transaction) => {
                    for p in params {
                        let result = transaction.execute(query.as_str(), params_from_iter(parse_params(p.as_array().unwrap().to_owned())));
                        if result.is_err() {
                            return Err(Error::RusqliteT(format!("Error on query {:?}", query)));
                        }
                    }
                    match transaction.commit() {
                        Ok(()) => Ok(true),
                        Err(e) => Err(Error::Rusqlite(e))
                    }
                },
                Err(e) => {
                    return Err(Error::Rusqlite(e))
                }
            }
        } else {
            execute_query(&conn, query.as_str(), params_from_iter(parse_params(params)))
        }
    } else {
        execute_query(&conn, query.as_str(), [])
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("taurusqlite")
    .invoke_handler(tauri::generate_handler![open, select, execute, set_pragma])
    .setup(|app| {
        app.manage(DbInstances::default());
        Ok(())
    })
    .build()
}
