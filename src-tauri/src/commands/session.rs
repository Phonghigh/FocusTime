use crate::db::{Category, Db, UsageEvent};
use crate::tracker::{window_poller, SessionEngine};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

/// Starts a tracking session for `profile_id`: opens the session engine and
/// spawns a background thread that polls the foreground window once a
/// second, feeding changes into the engine and emitting `live-state-update`.
#[tauri::command]
pub fn start_session(
    app: AppHandle,
    db: State<'_, Db>,
    engine: State<'_, SessionEngine>,
    profile_id: i64,
) -> Result<i64, String> {
    let session_id = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        engine.start(&conn, profile_id).map_err(|e| e.to_string())?
    };

    let running = engine.running_flag();
    let app_handle = app.clone();

    thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            let db_state = app_handle.state::<Db>();
            let engine_state = app_handle.state::<SessionEngine>();

            if let Some(name) = window_poller::foreground_process_name() {
                if let Ok(conn) = db_state.0.lock() {
                    engine_state.on_tick(&conn, &name);
                }
            }

            if let Some(live_state) = engine_state.live_state() {
                let _ = app_handle.emit("live-state-update", live_state);
            }

            thread::sleep(Duration::from_secs(1));
        }
    });

    Ok(session_id)
}

/// Stops the running session (closes final usage_event, sets ended_at).
#[tauri::command]
pub fn stop_session(db: State<'_, Db>, engine: State<'_, SessionEngine>) -> Result<Option<i64>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(engine.stop(&conn))
}

#[derive(Debug, Serialize)]
pub struct SessionReport {
    pub session_id: i64,
    pub totals: crate::tracker::session::CategoryTotals,
    pub events: Vec<UsageEvent>,
    pub interruption_count: i64,
}

/// Aggregates a finished session: per-category totals, ordered timeline, and
/// interruption count (a work/entertainment/distraction event... specifically
/// an entertainment/distraction event that follows a work event in the same
/// session, computed via a simple sequential scan).
#[tauri::command]
pub fn get_session_report(db: State<'_, Db>, session_id: i64) -> Result<SessionReport, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, process_name, domain, category, started_at, ended_at \
             FROM usage_events WHERE session_id = ?1 ORDER BY started_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let events = stmt
        .query_map([session_id], |row| {
            Ok(UsageEvent {
                id: row.get(0)?,
                session_id: row.get(1)?,
                process_name: row.get(2)?,
                domain: row.get(3)?,
                category: row.get(4)?,
                started_at: row.get(5)?,
                ended_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;

    let mut totals = crate::tracker::session::CategoryTotals::default();
    for event in &events {
        let duration = event.ended_at.unwrap_or(event.started_at) - event.started_at;
        add_total(&mut totals, event.category, duration.max(0));
    }

    let mut interruption_count = 0i64;
    let mut saw_work = false;
    for event in &events {
        match event.category {
            Category::Work => saw_work = true,
            Category::Entertainment | Category::Distraction if saw_work => {
                interruption_count += 1;
            }
            _ => {}
        }
    }

    Ok(SessionReport {
        session_id,
        totals,
        events,
        interruption_count,
    })
}

fn add_total(totals: &mut crate::tracker::session::CategoryTotals, category: Category, seconds: i64) {
    match category {
        Category::Work => totals.work += seconds,
        Category::Entertainment => totals.entertainment += seconds,
        Category::Distraction => totals.distraction += seconds,
        Category::Unclassified => totals.unclassified += seconds,
    }
}
