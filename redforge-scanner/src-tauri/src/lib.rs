mod models;
mod scanners;
mod commands;

use commands::scan::ScanState;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scan_state = ScanState {
        current_tasks: Arc::new(Mutex::new(Vec::new())),
        scan_results: Arc::new(Mutex::new(HashMap::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:redforge.db", vec![
                tauri_plugin_sql::Migration {
                    version: 1,
                    description: "Initial schema",
                    sql: include_str!("../migrations/001_initial.sql"),
                    kind: tauri_plugin_sql::MigrationKind::Up,
                }
            ])
            .build())
        .manage(scan_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_scan,
            commands::get_scan_status,
            commands::list_scans,
            commands::get_scan_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
