mod commands;
mod db;
pub mod native_host;
mod tracker;

use tauri::Manager;
use tracker::SessionEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = db::init_db(app.handle())?;
            app.manage(db);
            app.manage(SessionEngine::default());
            tracker::browser_bridge::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::profiles::list_profiles,
            commands::profiles::create_profile,
            commands::processes::list_running_processes,
            commands::rules::set_app_rules,
            commands::rules::get_app_rules,
            commands::domain_rules::set_domain_rules,
            commands::domain_rules::get_domain_rules,
            commands::session::start_session,
            commands::session::stop_session,
            commands::session::get_session_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
