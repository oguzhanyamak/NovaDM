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
//! Queued downloads are stored in a `VecDeque` for O(1) FIFO operations.
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
use crate::download::scheduler::DownloadScheduler;
use crate::core::DownloadHandle;

/// Download manager for handling HTTP downloads
pub struct DownloadManager {
    /// Active downloads indexed by ID
    active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
    /// Scheduler for queue management
    scheduler: Arc<DownloadScheduler>,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new() -> Self {
        Self {
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(DownloadScheduler::new()),
        }
    }

    /// Start a new download
    /// 
    /// If under the concurrent limit, starts immediately.
    /// Otherwise, enqueues for later.
    pub async fn start_download(
        &self,
        app: AppHandle,
        url: String,
        filename: String,
        save_location: String,
    ) -> Result<String> {
        let download_id = Uuid::new_v4().to_string();
        
        // Create task
        let task = DownloadTask {
            id: download_id.clone(),
            url,
            filename,
            save_location,
        };

        // Check if we can start immediately
        let active_count = self.active_downloads.read().await.len();
        
        if self.scheduler.can_start(active_count).await {
            // Start immediately
            self.start_download_immediately(app, task).await?;
        } else {
            // Enqueue for later
            let position = self.scheduler.enqueue(task).await?;
            
            // Emit queued event
            let _ = app.emit("download://queued", serde_json::json!({
                "id": download_id,
                "position": position
            }));
            
            tracing::info!("Download queued: {} at position {}", download_id, position);
        }

        Ok(download_id)
    }

    /// Start a download immediately
    async fn start_download_immediately(
        &self,
        app: AppHandle,
        task: DownloadTask,
    ) -> Result<()> {
        let output_path = PathBuf::from(&task.save_location).join(&task.filename);
        
        // Create download handle with cancellation token
        let mut handle = DownloadHandle::new(task.id.clone());
        handle.set_output_path(output_path.to_string_lossy().to_string());

        // Add to active downloads
        self.active_downloads.write().await.insert(task.id.clone(), handle.clone());

        // Emit started event
        let _ = app.emit("download://started", serde_json::json!({
            "id": task.id
        }));

        // Clone for worker
        let cancellation_token = handle.cancellation_token.clone();
        let active_downloads = self.active_downloads.clone();
        let scheduler = self.scheduler.clone();
        let task_id = task.id.clone();
        
        tauri::async_runtime::spawn(async move {
            let result = Self::download_file(app.clone(), task, cancellation_token).await;
            
            if let Err(e) = result {
                if !matches!(&e, DownloadError::Cancelled) {
                    tracing::error!("Download {} failed: {}", task_id, e);
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": task_id,
                        "message": e.to_string()
                    }));
                }
            }
            
            // Remove from active downloads
            active_downloads.write().await.remove(&task_id);
            
            // Try to start next queued download
            Self::try_start_next(app, active_downloads, scheduler);
        });

        Ok(())
    }

    /// Try to start the next queued download
    fn try_start_next(
        app: AppHandle,
        active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
        scheduler: Arc<DownloadScheduler>,
    ) {
        tauri::async_runtime::spawn(async move {
            // Get next task from queue
            if let Some((id, task)) = scheduler.pop_next().await {
                let output_path = PathBuf::from(&task.save_location).join(&task.filename);
                
                // Create handle
                let mut handle = DownloadHandle::new(id.clone());
                handle.set_output_path(output_path.to_string_lossy().to_string());
                
                // Add to active
                active_downloads.write().await.insert(id.clone(), handle.clone());
                
                // Emit started event
                let _ = app.emit("download://started", serde_json::json!({
                    "id": id
                }));
                
                // Clone token
                let cancellation_token = handle.cancellation_token.clone();
                
                // Spawn worker
                tauri::async_runtime::spawn(async move {
                    let result = Self::download_file(app.clone(), task, cancellation_token).await;
                    
                    if let Err(e) = result {
                        if !matches!(&e, DownloadError::Cancelled) {
                            let _ = app.emit("download://error", serde_json::json!({
                                "id": id,
                                "message": e.to_string()
                            }));
                        }
                    }
                    
                    active_downloads.write().await.remove(&id);
                    
                    // Try next again
                    Self::try_start_next(app, active_downloads, scheduler);
                });
            }
        });
    }

    /// Cancel a download (active or queued)
    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        // Check if it's active
        {
            let mut downloads = self.active_downloads.write().await;
            if let Some(handle) = downloads.remove(id) {
                handle.cancellation_token.cancel();
                tracing::info!("Active download cancelled: {}", id);
                return Ok(());
            }
        }
        
        // Check if it's queued
        if self.scheduler.dequeue(id).await.is_some() {
            tracing::info!("Queued download cancelled: {}", id);
            return Ok(());
        }
        
        Err(DownloadError::NotFound(id.to_string()))
    }

    /// Download a file with streaming and buffered writing
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
                drop(writer);
                let _ = tokio::fs::remove_file(&output_path).await;
                let _ = app.emit("download://cancelled", serde_json::json!({
                    "id": task.id
                }));
                tracing::info!("Download cancelled: {}", task.filename);
                return Err(DownloadError::Cancelled);
            }

            let chunk = chunk_result
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

            writer.write_all(&chunk).await
                .map_err(|e| DownloadError::IoError(e.to_string()))?;

            downloaded += chunk.len() as u64;
            Self::emit_progress(&app, &task.id, downloaded, content_length)?;
        }

        if cancellation_token.is_cancelled() {
            drop(writer);
            let _ = tokio::fs::remove_file(&output_path).await;
            let _ = app.emit("download://cancelled", serde_json::json!({
                "id": task.id
            }));
            return Err(DownloadError::Cancelled);
        }

        writer.flush().await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;
        
        tracing::info!("Download completed: {}", task.filename);

        let _ = app.emit("download://completed", serde_json::json!({
            "id": task.id
        }));

        Ok(())
    }

    /// Build the output file path and ensure directory exists
    async fn build_output_path(task: &DownloadTask) -> Result<PathBuf> {
        let initial_path = PathBuf::from(&task.save_location).join(&task.filename);
        
        if let Some(parent) = initial_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| crate::download::utils::categorize_io_error(&e))?;
        }

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

    /// Emit progress event
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

    /// Get queue position for a download
    pub async fn get_queue_position(&self, id: &str) -> Option<usize> {
        self.scheduler.get_position(id).await
    }

    /// Get queue length
    pub async fn queue_length(&self) -> usize {
        self.scheduler.len().await
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
        assert!(manager.get_active_downloads().await.is_empty());
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