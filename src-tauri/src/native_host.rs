//! Native messaging host entry point. Chrome/Edge/Firefox spawn this same
//! executable as `focustime.exe --native-host` (per the manifest registered
//! in the Windows registry, see browser-extension/README.md) and communicate
//! over stdin/stdout using Chrome's native messaging protocol: each message
//! is a 4-byte little-endian length prefix followed by that many bytes of
//! UTF-8 JSON.
//!
//! This host does the minimum needed to bridge that protocol to the already
//! running FocusTime app: it re-serializes each `{ domain, timestamp }`
//! message as a single newline-delimited JSON line and forwards it over a
//! plain TCP loopback connection to the main app's `browser_bridge` listener.
//! No retries/reconnect loop beyond "try to (re)connect lazily on next
//! message" — kept intentionally simple.

use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::net::TcpStream;

const BRIDGE_PORT: u16 = 47821;

#[derive(Debug, Deserialize)]
struct IncomingMessage {
    domain: String,
}

/// Chrome spawns this process with no attached console, so `eprintln!` goes
/// nowhere — log to a file next to the exe instead so the connection can
/// actually be debugged.
fn log(msg: &str) {
    if let Ok(mut dir) = std::env::current_exe() {
        dir.pop();
        dir.push("native-host.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(dir) {
            let _ = writeln!(f, "{msg}");
        }
    }
}

/// Runs the stdin-read loop until the browser closes the pipe (EOF).
pub fn run() {
    log("native_host: started");
    let stdin = io::stdin();
    let mut connection: Option<TcpStream> = None;

    loop {
        match read_message(&stdin) {
            Ok(Some(msg)) => {
                log(&format!("native_host: received domain {:?}", msg.domain));
                let line = format!("{{\"domain\":{:?}}}\n", msg.domain);
                if connection.is_none() {
                    match TcpStream::connect(("127.0.0.1", BRIDGE_PORT)) {
                        Ok(stream) => {
                            log("native_host: connected to FocusTime app");
                            connection = Some(stream);
                        }
                        Err(err) => log(&format!(
                            "native_host: could not reach FocusTime app on port {BRIDGE_PORT} \
                             (is it running?): {err}"
                        )),
                    }
                }
                if let Some(stream) = connection.as_mut() {
                    if let Err(err) = stream.write_all(line.as_bytes()) {
                        log(&format!("native_host: lost connection to FocusTime app: {err}"));
                        connection = None;
                    }
                }
            }
            Ok(None) => {
                log("native_host: stdin EOF, browser closed the pipe");
                break;
            }
            Err(_) => continue, // Malformed message: skip and keep reading.
        }
    }
}

/// Reads one native-messaging-protocol frame from stdin. Returns `Ok(None)`
/// on clean EOF (browser disconnected).
fn read_message(mut stdin: impl Read) -> Result<Option<IncomingMessage>, ()> {
    let mut len_buf = [0u8; 4];
    if let Err(err) = stdin.read_exact(&mut len_buf) {
        return if err.kind() == io::ErrorKind::UnexpectedEof {
            Ok(None)
        } else {
            Err(())
        };
    }
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut payload = vec![0u8; len];
    stdin.read_exact(&mut payload).map_err(|_| ())?;

    serde_json::from_slice::<IncomingMessage>(&payload)
        .map(Some)
        .map_err(|_| ())
}
