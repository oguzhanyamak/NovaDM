// API module - Tauri command handlers
// Minimal command surface for frontend communication

use tauri::{AppHandle, State};
use crate::core::AppState;
use crate::download::manager::DownloadManager;
use crate::download::recovery::{RecoveryCandidate, RecoveryService};
use crate::services::HistoryService;
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

#[tauri::command]
pub async fn retry_download(
    app_handle: AppHandle,
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<String, String> {
    tracing::info!("Retrying download: {}", id);

    download_manager
        .retry_download(app_handle, &id)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Download retry initiated: {}", id);
    Ok(id)
}

#[tauri::command]
pub async fn pause_download(
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    tracing::info!("Pausing download: {}", id);

    download_manager
        .pause_download(&id)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Download paused: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn resume_download(
    app_handle: AppHandle,
    download_manager: State<'_, DownloadManager>,
    id: String,
) -> Result<(), String> {
    tracing::info!("Resuming download: {}", id);

    download_manager
        .resume_download(app_handle, &id)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Download resume initiated: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn get_recovery_candidates() -> Result<Vec<RecoveryCandidate>, String> {
    let service = RecoveryService::new();
    let candidates = service.scan().await;
    Ok(candidates)
}

#[tauri::command]
pub async fn set_bandwidth_limit(
    download_manager: State<'_, DownloadManager>,
    limit_kb_per_sec: u64,
) -> Result<(), String> {
    // Convert KB/s to bytes/s
    let limit_bytes_per_sec = limit_kb_per_sec * 1024;
    download_manager
        .set_bandwidth_limit(limit_bytes_per_sec)
        .await;
    Ok(())
}

// History commands

use crate::storage::HistoryEntry;

/// Get all history entries
#[tauri::command]
pub async fn get_history() -> Result<Vec<HistoryEntry>, String> {
    let service = HistoryService::new();
    service.load_history().await.map_err(|e| e.to_string())
}

/// Delete a history entry
#[tauri::command]
pub async fn delete_history_entry(id: String) -> Result<(), String> {
    let service = HistoryService::new();
    service.delete_entry(&id).await.map_err(|e| e.to_string())
}

/// Delete multiple history entries
#[tauri::command]
pub async fn delete_history_entries(ids: Vec<String>) -> Result<(), String> {
    let service = HistoryService::new();
    service.delete_entries(&ids).await.map_err(|e| e.to_string())
}

/// Clear all history
#[tauri::command]
pub async fn clear_history() -> Result<(), String> {
    let service = HistoryService::new();
    service.clear_history().await.map_err(|e| e.to_string())
}

// Settings commands

use crate::storage::AppSettings;
use crate::services::SettingsService;

/// Get all settings
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    let service = SettingsService::new();
    service.load().await
}

/// Save settings
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let service = SettingsService::new();
    service.save(&settings).await
}

/// Update a single setting
#[tauri::command]
pub async fn update_setting(key: String, value: serde_json::Value) -> Result<AppSettings, String> {
    let service = SettingsService::new();
    let mut settings = service.load().await?;
    
    match key.as_str() {
        "download_path" => {
            settings.download_path = value.as_str().unwrap_or_default().to_string();
        }
        "max_concurrent_downloads" => {
            settings.max_concurrent_downloads = value.as_u64().unwrap_or(3) as usize;
        }
        "bandwidth_limit_kb" => {
            settings.bandwidth_limit_kb = value.as_u64().unwrap_or(0);
        }
        "auto_resume" => {
            settings.auto_resume = value.as_bool().unwrap_or(true);
        }
        "auto_retry" => {
            settings.auto_retry = value.as_bool().unwrap_or(true);
        }
        "max_retry_attempts" => {
            settings.max_retry_attempts = value.as_u64().unwrap_or(3) as u32;
        }
        "theme" => {
            settings.theme = match value.as_str().unwrap_or("system") {
                "dark" => crate::storage::Theme::Dark,
                "light" => crate::storage::Theme::Light,
                _ => crate::storage::Theme::System,
            };
        }
        "open_on_startup" => {
            settings.open_on_startup = value.as_bool().unwrap_or(false);
        }
        "auto_check_updates" => {
            settings.auto_check_updates = value.as_bool().unwrap_or(true);
        }
        "enable_notifications" => {
            settings.enable_notifications = value.as_bool().unwrap_or(true);
        }
        "enable_browser_integration" => {
            settings.enable_browser_integration = value.as_bool().unwrap_or(false);
        }
        _ => return Err(format!("Unknown setting key: {}", key)),
    }
    
    service.save(&settings).await?;
    Ok(settings)
}

/// Export settings to JSON
#[tauri::command]
pub async fn export_settings() -> Result<String, String> {
    let service = SettingsService::new();
    service.export().await
}

/// Import settings from JSON
#[tauri::command]
pub async fn import_settings(json: String) -> Result<AppSettings, String> {
    let service = SettingsService::new();
    service.import(&json).await
}

/// Reset settings to defaults
#[tauri::command]
pub async fn reset_settings() -> Result<AppSettings, String> {
    let service = SettingsService::new();
    service.reset().await
}

/// Select a folder via native file dialog
#[tauri::command]
pub async fn select_folder() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await;
    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}