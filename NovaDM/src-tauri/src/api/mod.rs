// API module - Tauri command handlers
// Minimal command surface for frontend communication

use tauri::State;
use crate::core::AppState;

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
    url: String,
    filename: String,
    save_location: String,
) -> Result<(), String> {
    // Log received values (will be replaced with actual download logic)
    tracing::info!("Starting download:");
    tracing::info!("  URL: {}", url);
    tracing::info!("  Filename: {}", filename);
    tracing::info!("  Save Location: {}", save_location);
    
    // TODO: Implement download logic in DownloadManager
    // For now, just return success
    Ok(())
}