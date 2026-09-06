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

/// Looks up `domain_rules` for `profile_id` + `domain` (case-insensitive) and
/// returns its category, falling back to `Unclassified` if no rule matches.
pub fn classify_domain(conn: &Connection, profile_id: i64, domain: &str) -> Category {
    conn.query_row(
        "SELECT category FROM domain_rules \
         WHERE profile_id = ?1 AND LOWER(domain) = LOWER(?2) \
         LIMIT 1",
        (profile_id, domain),
        |row| row.get::<_, Category>(0),
    )
    .unwrap_or(Category::Unclassified)
}

/// Executable names (lowercase) treated as browsers: when one of these is the
/// foreground process, domain-based classification takes over from the
/// process-name lookup once a domain update arrives.
pub const BROWSER_EXECUTABLES: [&str; 3] = ["chrome.exe", "msedge.exe", "firefox.exe"];

pub fn is_browser(process_name: &str) -> bool {
    BROWSER_EXECUTABLES
        .iter()
        .any(|b| b.eq_ignore_ascii_case(process_name))
}
