// API module - Tauri command handlers
// Minimal command surface for frontend communication

use tauri::State;
use crate::core::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub progress: Option<u32>,
    pub speed: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCompletedPayload {
    pub id: String,
}

#[tauri::command]
pub async fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub async fn get_app_state(_state: State<'_, AppState>) -> Result<String, String> {
    // Placeholder for getting application state
    Ok("ready".to_string())
}

#[tauri::command]
pub async fn start_download(
    app: tauri::AppHandle,
    url: String,
    filename: String,
    save_location: String,
) -> Result<(), String> {
    tracing::info!("Starting download:");
    tracing::info!("  URL: {}", url);
    tracing::info!("  Filename: {}", filename);
    tracing::info!("  Save Location: {}", save_location);

    let manager = DownloadManager::new();
    
    manager
        .start_download(app, url, filename, save_location)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
