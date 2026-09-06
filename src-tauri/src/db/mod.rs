pub mod category;
pub mod models;
pub mod schema;

use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub use category::Category;
pub use models::{AppRule, AppRuleInput, Profile, UsageEvent};

/// Shared, mutex-guarded SQLite connection stored as Tauri managed state.
pub struct Db(pub Mutex<Connection>);

/// Opens (creating if needed) `focustime.db` in the app's data directory and
/// applies schema migrations. Called once during app setup.
pub fn init_db(app_handle: &AppHandle) -> rusqlite::Result<Db> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("app data dir should be resolvable");
    ensure_dir(&data_dir);

    let db_path = data_dir.join("focustime.db");
    let conn = Connection::open(db_path)?;
    schema::apply_migrations(&conn)?;

    Ok(Db(Mutex::new(conn)))
}

fn ensure_dir(dir: &Path) {
    if !dir.exists() {
        fs::create_dir_all(dir).expect("should be able to create app data dir");
    }
}
