// API module - Tauri command handlers
// Minimal command surface for frontend communication

use tauri::{AppHandle, State};
use crate::core::AppState;
use crate::download::manager::DownloadManager;
use serde::{Deserialize, Serialize};

/// Progress payload for download progress events
/// 
/// Contains both calculated percentage and raw byte counts.
/// The frontend can derive percentage from bytes if needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub progress: Option<u32>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed: u64,
    pub status: String,
}

/// Completion payload for download completion events
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
    app_handle: AppHandle,
    download_manager: State<'_, DownloadManager>,
    url: String,
    filename: String,
    save_location: String,
) -> Result<String, String> {
    tracing::info!("Starting download:");
    tracing::info!("  URL: {}", url);
    tracing::info!("  Filename: {}", filename);
    tracing::info!("  Save Location: {}", save_location);

    // Use DownloadManager singleton from Tauri managed state
    let download_id = download_manager
        .start_download(app_handle, url, filename, save_location)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Download started with ID: {}", download_id);
    Ok(download_id)
}

#[tauri::command]
pub async fn cancel_download(
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    tracing::info!("Cancelling download: {}", id);

    download_manager
        .cancel_download(&id)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Download cancelled: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    tracing::info!("Opening file: {}", path);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .args([&path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
    tracing::info!("Showing in folder: {}", path);
    
    let parent = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args([&parent])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args([&parent])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .args([&parent])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
