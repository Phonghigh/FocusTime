use crate::db::Db;
use crate::tracker::SessionEngine;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;
use tauri::{AppHandle, Manager};

/// Fixed localhost port the native-messaging host (spawned by the browser as
/// `focustime.exe --native-host`) connects to in order to forward the active
/// tab's domain. Kept simple (no discovery/handshake) since both ends are
/// pieces of this same project running on the same machine.
pub const BRIDGE_PORT: u16 = 47821;

#[derive(Debug, Deserialize)]
struct DomainMessage {
    domain: String,
}

/// Starts a background thread listening on `127.0.0.1:BRIDGE_PORT` for
/// newline-delimited JSON `{ "domain": "..." }` messages forwarded by the
/// native messaging host, and feeds each one into the running
/// `SessionEngine`. Safe to call once during app setup; failures to bind
/// (e.g. port already in use) are logged and non-fatal.
pub fn start(app: &AppHandle) {
    let app_handle = app.clone();
    thread::spawn(move || {
        let listener = match TcpListener::bind(("127.0.0.1", BRIDGE_PORT)) {
            Ok(l) => l,
            Err(err) => {
                eprintln!("browser_bridge: failed to bind port {BRIDGE_PORT}: {err}");
                return;
            }
        };

        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let app_handle = app_handle.clone();
            thread::spawn(move || handle_connection(app_handle, stream));
        }
    });
}

fn handle_connection(app_handle: AppHandle, stream: std::net::TcpStream) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<DomainMessage>(&line) else {
            eprintln!("browser_bridge: dropping malformed message: {line}");
            continue;
        };

        let db_state = app_handle.state::<Db>();
        let engine_state = app_handle.state::<SessionEngine>();
        let lock_result = db_state.0.lock();
        if let Ok(conn) = lock_result {
            engine_state.on_domain_update(&conn, &msg.domain);
        }
    }
}
