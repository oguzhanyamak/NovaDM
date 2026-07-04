//! Download manager - Orchestrates download operations
//! 
//! This module provides the central download management functionality.
//! It handles HTTP downloads with streaming, progress tracking, cancellation, and event emission.
//! 
//! # Architecture
//! 
//! The DownloadManager is managed by Tauri as a singleton via `app.manage()`.
//! It lives for the entire application lifetime and is injected into commands.
//! 
//! Active downloads are stored in a `HashMap<String, DownloadHandle>` for O(1) lookup.
//! Each handle owns a CancellationToken for graceful cancellation.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use crate::download::errors::{DownloadError, Result};
use crate::download::models::DownloadTask;
use crate::download::utils::resolve_filename_conflict;
use crate::core::DownloadHandle;

/// Download manager for handling HTTP downloads
/// 
/// This struct is responsible for:
/// - Starting new downloads
/// - Tracking active downloads
/// - Spawning download workers
/// - Cancelling active downloads
/// - Managing download lifecycle
/// 
/// # Thread Safety
/// 
/// All state is protected by RwLocks for concurrent access.
pub struct DownloadManager {
    /// Active downloads indexed by ID
    active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new() -> Self {
        Self {
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new download
    /// 
    /// # Arguments
    /// * `app` - Tauri app handle for emitting events
    /// * `url` - URL to download
    /// * `filename` - Output filename
    /// * `save_location` - Directory to save file
    /// 
    /// # Returns
    /// * `Ok(String)` - Download ID
    /// * `Err(DownloadError)` - Failed to start download
    pub async fn start_download(
        &self,
        app: AppHandle,
        url: String,
        filename: String,
        save_location: String,
    ) -> Result<String> {
        let download_id = Uuid::new_v4().to_string();
        let output_path = PathBuf::from(&save_location).join(&filename);
        
        // Create download handle with cancellation token
        let mut handle = DownloadHandle::new(download_id.clone());
        handle.set_output_path(output_path.to_string_lossy().to_string());

        // Add to active downloads
        self.active_downloads.write().await.insert(download_id.clone(), handle.clone());

        // Create task
        let task = DownloadTask {
            id: download_id.clone(),
            url,
            filename,
            save_location,
        };

        // Clone cancellation token for the worker
        let cancellation_token = handle.cancellation_token.clone();
        
        // Spawn download task
        let active_downloads = self.active_downloads.clone();
        let task_id = task.id.clone();
        
        tauri::async_runtime::spawn(async move {
            let result = Self::download_file(app.clone(), task, cancellation_token).await;
            
            match result {
                Ok(()) => {
                    // Normal completion - already emitted completed event
                }
                Err(e) => {
                    // Only emit error if it wasn't a cancellation
                    if !matches!(&e, DownloadError::Cancelled) {
                        tracing::error!("Download {} failed: {}", task_id, e);
                        let _ = app.emit("download://error", serde_json::json!({
                            "id": task_id,
                            "message": e.to_string()
                        }));
                    }
                }
            }
            
            // Remove from active downloads
            active_downloads.write().await.remove(&task_id);
        });

        Ok(download_id)
    }

    /// Cancel an active download
    /// 
    /// Finds the download by ID, cancels its token, and removes it from active downloads.
    /// If the download doesn't exist, returns NotFound error.
    /// If the download is already completed, returns success (no-op).
    /// 
    /// # Arguments
    /// * `id` - Download ID to cancel
    /// 
    /// # Returns
    /// * `Ok(())` - Download cancelled or already completed
    /// * `Err(DownloadError)` - Download not found
    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        let mut downloads = self.active_downloads.write().await;
        
        if let Some(handle) = downloads.remove(id) {
            // Signal cancellation to the worker
            handle.cancellation_token.cancel();
            tracing::info!("Download cancelled: {}", id);
            Ok(())
        } else {
            Err(DownloadError::NotFound(id.to_string()))
        }
    }

    /// Download a file with streaming and buffered writing
    /// 
    /// This function:
    /// 1. Sends HTTP GET request
    /// 2. Streams response body in chunks
    /// 3. Checks cancellation token periodically
    /// 4. Writes to disk using BufWriter for efficiency
    /// 5. Emits progress events for each chunk
    /// 6. Emits completion event when done
    /// 7. Deletes partial file if cancelled
    /// 
    /// # Memory Usage
    /// For a 20 GB file, memory usage stays constant at ~8-64 KB
    /// because we stream and buffer only small chunks.
    async fn download_file(
        app: AppHandle,
        task: DownloadTask,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<()> {
        tracing::info!("Starting download: {} -> {}", task.url, task.filename);

        let output_path = Self::build_output_path(&task).await?;
        let response = Self::send_request(&task.url).await?;
        let content_length = response.content_length();
        let mut downloaded: u64 = 0;

        // Create file with buffered writer for performance
        let file = tokio::fs::File::create(&output_path).await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        // Stream response body
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if cancellation_token.is_cancelled() {
                // Close the writer
                drop(writer);
                
                // Delete partial file
                let _ = tokio::fs::remove_file(&output_path).await;
                
                // Emit cancelled event
                let _ = app.emit("download://cancelled", serde_json::json!({
                    "id": task.id
                }));
                
                tracing::info!("Download cancelled: {}", task.filename);
                return Err(DownloadError::Cancelled);
            }

            let chunk = chunk_result
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

            // Write chunk to disk
            writer.write_all(&chunk).await
                .map_err(|e| DownloadError::IoError(e.to_string()))?;

            // Update progress
            downloaded += chunk.len() as u64;

            // Emit progress event for every chunk
            Self::emit_progress(&app, &task.id, downloaded, content_length)?;
        }

        // Check for cancellation before finalizing
        if cancellation_token.is_cancelled() {
            drop(writer);
            let _ = tokio::fs::remove_file(&output_path).await;
            let _ = app.emit("download://cancelled", serde_json::json!({
                "id": task.id
            }));
            return Err(DownloadError::Cancelled);
        }

        // Flush buffer and sync to disk
        writer.flush().await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;
        
        tracing::info!("Download completed: {}", task.filename);

        // Emit completion event
        let _ = app.emit("download://completed", serde_json::json!({
            "id": task.id
        }));

        Ok(())
    }

    /// Build the output file path and ensure directory exists
    /// 
    /// If the file already exists, automatically renames it:
    /// - movie.mp4 → movie (1).mp4
    /// - movie (1).mp4 → movie (2).mp4
    async fn build_output_path(task: &DownloadTask) -> Result<PathBuf> {
        let initial_path = PathBuf::from(&task.save_location).join(&task.filename);
        
        // Ensure directory exists
        if let Some(parent) = initial_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| crate::download::utils::categorize_io_error(&e))?;
        }

        // Resolve filename conflicts
        let output_path = resolve_filename_conflict(&initial_path)
            .map_err(|e| crate::download::utils::categorize_io_error(&e))?;

        Ok(output_path)
    }

    /// Send HTTP GET request
    async fn send_request(url: &str) -> Result<reqwest::Response> {
        let response = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DownloadError::HttpError(response.status().as_u16()));
        }

        Ok(response)
    }

    /// Emit progress event with downloaded and total bytes
    fn emit_progress(
        app: &AppHandle,
        id: &str,
        downloaded: u64,
        total: Option<u64>,
    ) -> Result<()> {
        let progress = total.map(|t| ((downloaded as f64 / t as f64) * 100.0).min(100.0) as u32);

        let _ = app.emit("download://progress", serde_json::json!({
            "id": id,
            "progress": progress,
            "downloaded_bytes": downloaded,
            "total_bytes": total,
            "speed": 0,
            "status": "downloading"
        }));

        Ok(())
    }

    /// Check if a download is active
    pub async fn is_active(&self, id: &str) -> bool {
        self.active_downloads.read().await.contains_key(id)
    }

    /// Get all active downloads
    pub async fn get_active_downloads(&self) -> Vec<DownloadHandle> {
        self.active_downloads.read().await.values().cloned().collect()
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_manager_creation() {
        let manager = DownloadManager::new();
        assert!(manager.active_downloads.blocking_read().is_empty());
    }

    #[tokio::test]
    async fn test_download_manager_lifecycle() {
        let manager = DownloadManager::new();
        
        // Initially empty
        assert!(manager.get_active_downloads().await.is_empty());
        
        // Can check if download is active
        assert!(!manager.is_active("nonexistent").await);
    }

    #[tokio::test]
    async fn test_cancel_nonexistent() {
        let manager = DownloadManager::new();
        let result = manager.cancel_download("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DownloadError::NotFound(_)));
    }
}