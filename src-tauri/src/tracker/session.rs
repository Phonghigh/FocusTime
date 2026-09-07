use super::classifier;
use crate::db::Category;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-category running totals (seconds), used for the live view and report.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CategoryTotals {
    pub work: i64,
    pub entertainment: i64,
    pub distraction: i64,
    pub unclassified: i64,
}

impl CategoryTotals {
    fn add(&mut self, category: Category, seconds: i64) {
        match category {
            Category::Work => self.work += seconds,
            Category::Entertainment => self.entertainment += seconds,
            Category::Distraction => self.distraction += seconds,
            Category::Unclassified => self.unclassified += seconds,
        }
    }
}

/// Snapshot broadcast to the frontend roughly once a second.
#[derive(Debug, Clone, Serialize)]
pub struct LiveState {
    pub process_name: String,
    pub domain: Option<String>,
    pub category: Category,
    pub elapsed_seconds: i64,
    pub totals: CategoryTotals,
}

struct Inner {
    session_id: i64,
    profile_id: i64,
    session_start: i64,
    current_process: String,
    current_category: Category,
    current_domain: Option<String>,
    event_start: i64,
    totals: CategoryTotals,
}

/// Holds the state of the currently running tracking session (if any) behind
/// a mutex, plus a flag the poller thread checks to know when to stop.
#[derive(Default)]
pub struct SessionEngine {
    inner: Mutex<Option<Inner>>,
    running: Arc<AtomicBool>,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

impl SessionEngine {
    /// Starts a new session for `profile_id`: inserts the `sessions` row,
    /// classifies whatever is currently in the foreground, and opens the
    /// first in-memory usage event. Returns the new session id.
    pub fn start(&self, conn: &Connection, profile_id: i64) -> rusqlite::Result<i64> {
        let started_at = now();
        conn.execute(
            "INSERT INTO sessions (profile_id, started_at) VALUES (?1, ?2)",
            (profile_id, started_at),
        )?;
        let session_id = conn.last_insert_rowid();

        let process_name =
            super::window_poller::foreground_process_name().unwrap_or_else(|| "unknown".into());
        let category = classifier::classify(conn, profile_id, &process_name);

        *self.inner.lock().unwrap() = Some(Inner {
            session_id,
            profile_id,
            session_start: started_at,
            current_process: process_name,
            current_category: category,
            current_domain: None,
            event_start: started_at,
            totals: CategoryTotals::default(),
        });
        self.running.store(true, Ordering::SeqCst);

        Ok(session_id)
    }

    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    /// Called every poll tick with the current foreground process name. If it
    /// differs from the tracked one, closes the previous usage_event (writing
    /// it to the DB) and opens a new one.
    pub fn on_tick(&self, conn: &Connection, process_name: &str) {
        let mut guard = self.inner.lock().unwrap();
        let Some(inner) = guard.as_mut() else { return };
        if inner.current_process == process_name {
            return;
        }

        let ts = now();
        Self::close_event(conn, inner, ts);

        inner.current_process = process_name.to_string();
        inner.current_category = classifier::classify(conn, inner.profile_id, process_name);
        inner.current_domain = None;
        inner.event_start = ts;
    }

    /// Called whenever the browser extension (via the native-messaging
    /// bridge) reports the active tab's domain. Only takes effect when the
    /// currently tracked foreground process is a known browser executable;
    /// otherwise the update is stale/irrelevant and ignored. Reclassifies
    /// using `domain_rules` instead of `app_rules`.
    pub fn on_domain_update(&self, conn: &Connection, domain: &str) {
        let mut guard = self.inner.lock().unwrap();
        let Some(inner) = guard.as_mut() else {
            crate::tracker::browser_bridge::log("session: no active session, dropping domain update");
            return;
        };
        if !classifier::is_browser(&inner.current_process) {
            crate::tracker::browser_bridge::log(&format!(
                "session: current_process {:?} is not a browser, dropping domain {domain:?}",
                inner.current_process
            ));
            return;
        }
        if inner.current_domain.as_deref() == Some(domain) {
            return;
        }

        let ts = now();
        Self::close_event(conn, inner, ts);

        inner.current_category = classifier::classify_domain(conn, inner.profile_id, domain);
        inner.current_domain = Some(domain.to_string());
        inner.event_start = ts;
    }

    fn close_event(conn: &Connection, inner: &mut Inner, ended_at: i64) {
        let duration = (ended_at - inner.event_start).max(0);
        let _ = conn.execute(
            "INSERT INTO usage_events (session_id, process_name, domain, category, started_at, ended_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                inner.session_id,
                &inner.current_process,
                &inner.current_domain,
                inner.current_category,
                inner.event_start,
                ended_at,
            ),
        );
        inner.totals.add(inner.current_category, duration);
    }

    /// Closes the final usage_event and marks the session as ended. Returns
    /// the session id that was stopped, if a session was running.
    pub fn stop(&self, conn: &Connection) -> Option<i64> {
        self.running.store(false, Ordering::SeqCst);
        let mut guard = self.inner.lock().unwrap();
        let inner = guard.take()?;
        let mut inner = inner;
        let ts = now();
        Self::close_event(conn, &mut inner, ts);
        let _ = conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            (ts, inner.session_id),
        );
        Some(inner.session_id)
    }

    /// Snapshot for the `live-state-update` event. Includes the elapsed time
    /// of the currently open (not-yet-persisted) event folded into totals.
    pub fn live_state(&self) -> Option<LiveState> {
        let guard = self.inner.lock().unwrap();
        let inner = guard.as_ref()?;
        let ts = now();
        let mut totals = inner.totals;
        totals.add(inner.current_category, (ts - inner.event_start).max(0));

        Some(LiveState {
            process_name: inner.current_process.clone(),
            domain: inner.current_domain.clone(),
            category: inner.current_category,
            elapsed_seconds: (ts - inner.session_start).max(0),
            totals,
        })
    }
}
