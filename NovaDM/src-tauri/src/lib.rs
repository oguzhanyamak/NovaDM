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

use api::{ping, get_app_state, start_download, start_fake_download};
use core::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            ping,
            get_app_state,
            start_download,
            start_fake_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
