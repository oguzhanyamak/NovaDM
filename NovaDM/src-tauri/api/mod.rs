// API module for Tauri commands
// This module will contain all Tauri command handlers
// that expose Rust functionality to the frontend

pub mod download;
pub mod storage;

use tauri::State;
use crate::download::DownloadManager;
use crate::storage::StorageManager;

#[tauri::command]
pub async fn get_downloads(
    download_manager: State<'_, DownloadManager>,
) -> Result<Vec<crate::download::DownloadInfo>, String> {
    download_manager.get_all()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(
    download_manager: State<'_, DownloadManager>,
) -> Result<Vec<crate::download::DownloadInfo>, String> {
    download_manager.get_history()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_download(
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    download_manager.pause(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_download(
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    download_manager.resume(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_download(
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    download_manager.cancel(&id)
        .map_err(|e| e.to_string())
}