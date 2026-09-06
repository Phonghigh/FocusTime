// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Chrome/Edge/Firefox launch the native messaging host as this same
    // executable with `--native-host` (per the registered manifest's "path").
    // Detect that case before starting the full Tauri/WebView2 app.
    if std::env::args().any(|arg| arg == "--native-host") {
        focustime_lib::native_host::run();
        return;
    }

    focustime_lib::run()
}
