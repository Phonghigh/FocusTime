use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct RunningProcess {
    pub pid: u32,
    pub process_name: String,
}

/// Lists currently running processes (deduplicated by executable name) for
/// the Setup screen's "detect running apps" checklist.
#[tauri::command]
pub fn list_running_processes() -> Vec<RunningProcess> {
    let mut system = System::new_all();
    system.refresh_all();

    let mut seen = std::collections::HashSet::new();
    let mut processes = Vec::new();

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_string();
        if name.is_empty() || !seen.insert(name.clone()) {
            continue;
        }
        processes.push(RunningProcess {
            pid: pid.as_u32(),
            process_name: name,
        });
    }

    processes.sort_by(|a, b| a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase()));
    processes
}
