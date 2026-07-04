// API module - Tauri command handlers
// Simplified to only expose essential commands

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
