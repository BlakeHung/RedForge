mod commands;
mod models;
mod scanners;

use commands::scan::{ScanState, start_scan, get_scan_status, list_scans, get_scan_report};
use commands::collaboration::{export_scan_data, deduplicate_import_data, import_scan_data};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ScanState {
            current_tasks: Arc::new(Mutex::new(Vec::new())),
            scan_results: Arc::new(Mutex::new(HashMap::new())),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            start_scan,
            get_scan_status,
            list_scans,
            get_scan_report,
            export_scan_data,
            deduplicate_import_data,
            import_scan_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
