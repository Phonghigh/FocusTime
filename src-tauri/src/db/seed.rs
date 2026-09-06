use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Inserts a ready-to-use "Sample Profile" with a few app/domain rules, but
/// only if the `profiles` table is empty (fresh install). Lets a user try
/// the app immediately without manually configuring rules first.
pub fn seed_sample_profile_if_empty(conn: &Connection) -> Result<()> {
    let profile_count: i64 = conn.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;
    if profile_count > 0 {
        return Ok(());
    }

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO profiles (name, created_at) VALUES (?1, ?2)",
        ("Sample Profile", created_at),
    )?;
    let profile_id = conn.last_insert_rowid();

    let app_rules: &[(&str, &str)] = &[
        ("Code.exe", "work"),
        ("devenv.exe", "work"),
        ("WINWORD.EXE", "work"),
        ("Discord.exe", "distraction"),
        ("Spotify.exe", "entertainment"),
    ];
    for (process_name, category) in app_rules {
        conn.execute(
            "INSERT INTO app_rules (profile_id, process_name, category) VALUES (?1, ?2, ?3)",
            (profile_id, process_name, category),
        )?;
    }

    let domain_rules: &[(&str, &str)] = &[
        ("github.com", "work"),
        ("stackoverflow.com", "work"),
        ("docs.google.com", "work"),
        ("youtube.com", "entertainment"),
        ("netflix.com", "entertainment"),
        ("facebook.com", "distraction"),
        ("twitter.com", "distraction"),
        ("x.com", "distraction"),
    ];
    for (domain, category) in domain_rules {
        conn.execute(
            "INSERT INTO domain_rules (profile_id, domain, category) VALUES (?1, ?2, ?3)",
            (profile_id, domain, category),
        )?;
    }

    Ok(())
}
