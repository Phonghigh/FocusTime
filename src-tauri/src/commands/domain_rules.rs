use crate::db::{Db, DomainRule, DomainRuleInput};
use tauri::State;

/// Replaces all domain_rules for `profile_id` with the given list (delete +
/// re-insert, mirrors `set_app_rules`).
#[tauri::command]
pub fn set_domain_rules(
    db: State<'_, Db>,
    profile_id: i64,
    rules: Vec<DomainRuleInput>,
) -> Result<(), String> {
    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM domain_rules WHERE profile_id = ?1", [profile_id])
        .map_err(|e| e.to_string())?;

    for rule in rules {
        tx.execute(
            "INSERT INTO domain_rules (profile_id, domain, category) VALUES (?1, ?2, ?3)",
            (profile_id, &rule.domain, rule.category),
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns all domain_rules configured for a profile.
#[tauri::command]
pub fn get_domain_rules(db: State<'_, Db>, profile_id: i64) -> Result<Vec<DomainRule>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, profile_id, domain, category FROM domain_rules WHERE profile_id = ?1")
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([profile_id], |row| {
            Ok(DomainRule {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                domain: row.get(2)?,
                category: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}
