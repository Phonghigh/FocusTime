use crate::db::{Db, Profile};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// Creates a new profile and returns it.
#[tauri::command]
pub fn create_profile(db: State<'_, Db>, name: String) -> Result<Profile, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO profiles (name, created_at) VALUES (?1, ?2)",
        (&name, created_at),
    )
    .map_err(|e| e.to_string())?;

    Ok(Profile {
        id: conn.last_insert_rowid(),
        name,
        created_at,
    })
}

/// Returns all saved profiles, ordered by creation time. Proof-of-connectivity
/// command for Phase 1 — returns an empty list until profiles are created.
#[tauri::command]
pub fn list_profiles(db: State<'_, Db>) -> Result<Vec<Profile>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM profiles ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;

    let profiles = stmt
        .query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;

    Ok(profiles)
}
