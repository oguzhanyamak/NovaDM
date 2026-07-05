// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod api;
mod core;
mod download;
mod services;
mod storage;
mod utils;

use api::{get_app_state, get_recovery_candidates, pause_download, ping, resume_download, start_download, cancel_download, open_file, show_in_folder, retry_download, set_bandwidth_limit, get_history, delete_history_entry, delete_history_entries, clear_history};
use core::AppState;
use download::manager::DownloadManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();
    let download_manager = DownloadManager::new();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .manage(download_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            ping,
            get_app_state,
            start_download,
            cancel_download,
            open_file,
            show_in_folder,
            retry_download,
            pause_download,
            resume_download,
            get_recovery_candidates,
            set_bandwidth_limit,
            get_history,
            delete_history_entry,
            delete_history_entries,
            clear_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}