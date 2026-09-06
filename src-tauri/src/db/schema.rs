use rusqlite::{Connection, Result};

/// Applies the full FocusTime schema. Idempotent (safe to run on every startup).
///
/// Timestamps are stored as INTEGER epoch seconds. `category` columns store
/// the `Category` enum serialized as TEXT (work/entertainment/distraction).
pub fn apply_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS profiles (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_rules (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id   INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            process_name TEXT NOT NULL,
            category     TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS domain_rules (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            domain     TEXT NOT NULL,
            category   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            started_at INTEGER NOT NULL,
            ended_at   INTEGER
        );

        CREATE TABLE IF NOT EXISTS usage_events (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id   INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            process_name TEXT NOT NULL,
            domain       TEXT,
            category     TEXT NOT NULL,
            started_at   INTEGER NOT NULL,
            ended_at     INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_app_rules_profile ON app_rules(profile_id);
        CREATE INDEX IF NOT EXISTS idx_domain_rules_profile ON domain_rules(profile_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_profile ON sessions(profile_id);
        CREATE INDEX IF NOT EXISTS idx_usage_events_session ON usage_events(session_id);
        "#,
    )
}
