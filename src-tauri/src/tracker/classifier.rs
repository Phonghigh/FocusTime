use crate::db::Category;
use rusqlite::Connection;

/// Looks up `app_rules` for `profile_id` + `process_name` (case-insensitive)
/// and returns its category, falling back to `Unclassified` if no rule
/// matches (or the query itself fails).
pub fn classify(conn: &Connection, profile_id: i64, process_name: &str) -> Category {
    conn.query_row(
        "SELECT category FROM app_rules \
         WHERE profile_id = ?1 AND LOWER(process_name) = LOWER(?2) \
         LIMIT 1",
        (profile_id, process_name),
        |row| row.get::<_, Category>(0),
    )
    .unwrap_or(Category::Unclassified)
}
