//! Core module - Application state and configuration
//! 
//! This module provides the central application state that is shared across all Tauri commands.
//! It owns configuration and other lifecycle-managed resources.

pub mod config;
pub mod constants;
pub mod errors;
pub mod events;

pub use config::AppConfig;

use tokio_util::sync::CancellationToken;

/// Download handle for tracking and controlling active downloads
/// 
/// Each handle contains:
/// - `id`: Unique identifier for the download
/// - `output_path`: Path to the output file
/// - `cancellation_token`: Token used to signal cancellation to the download worker
/// - `pause_token`: Token used to signal pause to the download worker
/// 
/// # Cancellation Flow
/// 
/// 1. User clicks Cancel button
/// 2. Frontend sends cancel_download command
/// 3. Backend finds handle by ID
/// 4. Calls `token.cancel()` on the CancellationToken
/// 5. Download loop checks `token.is_cancelled()` and exits gracefully
/// 6. Partial file is deleted
/// 
/// # Pause Flow
/// 
/// 1. User clicks Pause button
/// 2. Frontend sends pause_download command
/// 3. Backend finds handle by ID
/// 4. Calls `token.cancel()` on the pause token
/// 5. Download loop checks `token.is_cancelled()` and exits gracefully
/// 6. Partial file and metadata are preserved
#[derive(Debug, Clone)]
pub struct DownloadHandle {
    pub id: String,
    pub output_path: Option<String>,
    pub cancellation_token: CancellationToken,
    pub pause_token: CancellationToken,
}

impl DownloadHandle {
    /// Create a new download handle with cancellation and pause tokens
    pub fn new(id: String) -> Self {
        Self {
            id,
            output_path: None,
            cancellation_token: CancellationToken::new(),
            pause_token: CancellationToken::new(),
        }
    }

    /// Set the output path for this download
    pub fn set_output_path(&mut self, path: String) {
        self.output_path = Some(path);
    }
}

/// Application state shared across all commands
/// 
/// This struct is managed by Tauri and injected into commands via dependency injection.
/// It holds configuration and other lightweight state.
#[derive(Debug, Default)]
pub struct AppState {
    pub config: AppConfig,
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self::default()
    }
}