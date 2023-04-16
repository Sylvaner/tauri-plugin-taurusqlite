use std::{collections::HashMap, sync::Mutex};
use rusqlite::Connection;
use serde::{ser::Serializer, Serialize, Deserialize};
use serde_json::{Value as JsonValue};
use sqlite::{connect, execute as sqlite_execute, select as sqlite_select, batch as sqlite_batch};
use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, 
  AppHandle, Runtime, State,
};
mod sqlite;

const STORE_FILENAME: &str = "store.sqlite";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Rusqlite(rusqlite::Error),
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

#[derive(Default)]
struct DbInstances (
    Mutex<HashMap<String, Connection>>
);

#[derive(Deserialize)]
struct OpenOptions {
    disable_foreign_keys: Option<bool>
}

fn open_db(state: State<'_, DbInstances>, db_path: String, options: OpenOptions) -> Result<bool> {
    match connect(&db_path) {
        Ok(mut conn) => {
            if let Some(disable_foreign_keys) = options.disable_foreign_keys {
                if disable_foreign_keys == true {
                    _ = sqlite_execute(&mut conn, "PRAGMA foreign_keys = 0", vec!());
                }
            }
            state.0.lock().unwrap().insert(db_path, conn);
            Ok(true)
        }    
        Err(e) => Err(Error::Rusqlite(e))
    }
}

#[tauri::command]
async fn load<R: Runtime>(app: AppHandle<R>, state: State<'_, DbInstances>, options: OpenOptions) -> Result<String> {
    let app_dir = app.path_resolver().app_data_dir().expect("Failed to resolve app_dir");
    let db_path = app_dir.join(STORE_FILENAME).as_path().display().to_string();
    match open_db(state, db_path.clone(), options) {
        Ok(_) => Ok(db_path),
        Err(e) => Err(e)
    }
}

#[tauri::command]
async fn open(state: State<'_, DbInstances>, db_path: String, options: OpenOptions) -> Result<bool> {
    open_db(state, db_path, options)
}

#[tauri::command]
async fn set_pragma(state: State<'_, DbInstances>, db_path: String, key: String, value: JsonValue) -> Result<bool> {
    let mut mutex_map = state.0.lock().unwrap();
    let mut conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    match sqlite_execute(&mut conn, format!("PRAGMA {} = {}", key, value).as_str(), vec!()) {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::Rusqlite(e))
    }
}

#[tauri::command]
async fn select(state: State<'_, DbInstances>, db_path: String, query: String, params: Vec<JsonValue>) -> Result<Vec<HashMap<String, JsonValue>>> {
    let mut mutex_map = state.0.lock().unwrap();
    let conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    match sqlite_select(&conn, query.as_str(), params) {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::Rusqlite(e))
    }
}

#[tauri::command]
async fn execute(state: State<'_, DbInstances>, db_path: String, query: String, params: Vec<JsonValue>) -> Result<bool> {
    let mut mutex_map = state.0.lock().unwrap();
    let mut conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    match sqlite_execute(&mut conn, query.as_str(), params) {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::Rusqlite(e))
    }
}

#[tauri::command]
async fn batch(state: State<'_, DbInstances>, db_path: String, queries: Vec<(&str, Vec<JsonValue>)>) -> Result<bool> {
    let mut mutex_map = state.0.lock().unwrap();
    let mut conn = mutex_map.get_mut(&db_path).ok_or(Error::NotConnected(db_path))?;
    match sqlite_batch(&mut conn, queries) {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::Rusqlite(e))
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("taurusqlite")
    .invoke_handler(tauri::generate_handler![open, select, execute, set_pragma, batch, load])
    .setup(|app| {
        app.manage(DbInstances::default());
        Ok(())
    })
    .build()
}
