// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod api;
mod download;
mod storage;
mod utils;

use api::{get_downloads, get_history, pause_download, resume_download, cancel_download};
use download::DownloadManager;
use storage::StorageManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DownloadManager::new())
        .manage(StorageManager::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_downloads,
            get_history,
            pause_download,
            resume_download,
            cancel_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
