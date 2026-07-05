// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod api;
mod core;
mod download;
mod storage;
mod utils;

use api::{ping, get_app_state, start_download, cancel_download, open_file, show_in_folder, retry_download};
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
            retry_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
