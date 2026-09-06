use crate::db::{AppRule, AppRuleInput, Db};
use tauri::State;

/// Replaces all app_rules for `profile_id` with the given list (delete +
/// re-insert, simplest correct approach for a small per-profile rule set).
#[tauri::command]
pub fn set_app_rules(
    db: State<'_, Db>,
    profile_id: i64,
    rules: Vec<AppRuleInput>,
) -> Result<(), String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM app_rules WHERE profile_id = ?1", [profile_id])
        .map_err(|e| e.to_string())?;

    for rule in rules {
        tx.execute(
            "INSERT INTO app_rules (profile_id, process_name, category) VALUES (?1, ?2, ?3)",
            (profile_id, &rule.process_name, rule.category),
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns all app_rules configured for a profile.
#[tauri::command]
pub fn get_app_rules(db: State<'_, Db>, profile_id: i64) -> Result<Vec<AppRule>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, profile_id, process_name, category FROM app_rules WHERE profile_id = ?1")
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([profile_id], |row| {
            Ok(AppRule {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                process_name: row.get(2)?,
                category: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}
