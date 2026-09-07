use crate::db::Db;
use crate::tracker::SessionEngine;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use tauri::{AppHandle, Manager};

/// The GUI app has no console in release builds (`windows_subsystem =
/// "windows"`), so `eprintln!` goes nowhere — log to a file next to the exe
/// instead so the bridge can actually be debugged.
pub fn log(msg: &str) {
    if let Ok(mut dir) = std::env::current_exe() {
        dir.pop();
        dir.push("browser-bridge.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(dir) {
            let _ = writeln!(f, "{msg}");
        }
    }
}

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
                log(&format!("browser_bridge: failed to bind port {BRIDGE_PORT}: {err}"));
                return;
            }
        };
        log(&format!("browser_bridge: listening on {BRIDGE_PORT}"));

        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            log("browser_bridge: native host connected");
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
            log(&format!("browser_bridge: dropping malformed message: {line}"));
            continue;
        };
        log(&format!("browser_bridge: received domain {:?}", msg.domain));

        let db_state = app_handle.state::<Db>();
        let engine_state = app_handle.state::<SessionEngine>();
        let lock_result = db_state.0.lock();
        if let Ok(conn) = lock_result {
            engine_state.on_domain_update(&conn, &msg.domain);
        } else {
            log("browser_bridge: could not lock db");
        }
    }
}
