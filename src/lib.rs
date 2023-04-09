//! Taurusqlite is a plugin for Tauri that provides a light integration of Rusqlite
//!
use std::{path::Path, collections::HashMap, sync::Mutex};
use rusqlite::{Connection, Statement, types::{ValueRef, Value}, Row, Params};
use serde::{ser::Serializer, Serialize};
use tauri::{
  command,
  plugin::{Builder, TauriPlugin},
  Manager, 
  Runtime, State,
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Rusqlite(rusqlite::Error),
    #[error("Not connected")]
    NotConnected(),
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
/// let conn = get_conn("./path/to/db.sqlite");
/// ```
/// ## Failure
/// Panics on error
/// 
fn get_conn(path_to_db: &str) -> Connection {
    let database_path = Path::new(path_to_db);
    match Connection::open(database_path) {
        Ok(conn) => conn,
        Err(error) => panic!("Unable to open {:?}: {:?}", database_path, error)
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
fn prepare<'a>(conn: &'a Connection, query: &str) -> Statement<'a> {
    let stmt = match conn.prepare(query) {
        Ok(stmt) => stmt,
        Err(error) => panic!("Error create statement : {:?}", error)
    };
    stmt
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
fn parse_column_data(value_to_parse: ValueRef) -> Value {
    match value_to_parse {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Integer(i),
        ValueRef::Real(f) => Value::Real(f),
        ValueRef::Text(t) => Value::Text(std::str::from_utf8(t).unwrap().to_string()),
        ValueRef::Blob(b) => Value::Blob(b.to_vec())
    }
}

/// Parse row from result
/// 
fn parse_row(names: &[String], row: &Row) -> HashMap<String, Value> {
    let mut parsed_row: HashMap<String, Value> = HashMap::new();
    for name in names.iter() {
        match row.get_ref(name.as_str()) {
            Ok(column_ref) => parsed_row.insert(name.to_owned(), parse_column_data(column_ref)),
            Err(e) => panic!("{:?}", e)
        };
    }
    parsed_row
}

/// Query the database
/// 
/// # Example
/// ```
/// let conn = get_conn("./path/to/db.sqlite");
/// let persons = select(&conn, "SELECT * FROM student WHERE firstname = ?1", ["Bob"]);
/// ```
/// 
#[command]
fn select<P: Params>(conn: &Connection, query: &str, params: P) -> Vec<HashMap<String, Value>> {
    let mut result: Vec<HashMap<String, Value>> = Vec::new();
    let mut stmt = prepare(conn, query);
    let names = get_columns_names(&stmt);
    let rows_result = stmt.query(params);
    if let Ok(mut rows) = rows_result {
        loop {
            let row = match rows.next() {
                Ok(row_opt) => match row_opt {
                    Some(row) => row,
                    None => break
                },
                Err(_) => break
            };
            result.push(parse_row(&names, row));
        }
    }
    result
}

#[derive(Default)]
struct DbInstances (
    Mutex<HashMap<String, Connection>>
);

#[command]
fn open(state: State<'_, DbInstances>, database_path: &str) -> Result<bool> {
    let path = Path::new(database_path);
    match Connection::open(path) {
        Ok(conn) => {
            state.0.lock().unwrap().insert("conn".into(), conn);
            Ok(true)
        }    
        Err(error) => Err(Error::Rusqlite(error))
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("taurusqlite")
    .invoke_handler(tauri::generate_handler![open])
    .setup(|app| {
        app.manage(DbInstances::default());
        Ok(())
    })
    .build()
}
