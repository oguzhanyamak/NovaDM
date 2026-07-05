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
//! Failed downloads are stored in a `HashMap` for retry.
//! Metadata is persisted to disk for potential resume support.

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
use crate::download::metadata::{DownloadMetadata, MetadataRepository};
use crate::download::resume_detector::ResumeCapabilityDetector;
use crate::download::partial_file::PartialFileManager;
use crate::core::DownloadHandle;

/// Download manager for handling HTTP downloads
pub struct DownloadManager {
    /// Active downloads indexed by ID
    active_downloads: Arc<RwLock<HashMap<String, DownloadHandle>>>,
    /// Scheduler for queue management
    scheduler: Arc<DownloadScheduler>,
    /// Failed downloads indexed by ID (for retry)
    failed_downloads: Arc<RwLock<HashMap<String, DownloadTask>>>,
    /// Metadata repository for persistence
    metadata_repo: MetadataRepository,
    /// Resume capability detector
    resume_detector: ResumeCapabilityDetector,
    /// Partial file manager
    partial_file_manager: PartialFileManager,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new() -> Self {
        Self {
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(DownloadScheduler::new()),
            failed_downloads: Arc::new(RwLock::new(HashMap::new())),
            metadata_repo: MetadataRepository::new(),
            resume_detector: ResumeCapabilityDetector::new(),
            partial_file_manager: PartialFileManager::new(),
        }
    }

    /// Start a new download
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

    /// Retry a failed download
    /// 
    /// Only failed downloads can be retried.
    /// The retry enters the queue exactly like a new download.
    pub async fn retry_download(&self, app: AppHandle, id: &str) -> Result<()> {
        // Check if it's active
        {
            let downloads = self.active_downloads.read().await;
            if downloads.contains_key(id) {
                return Err(DownloadError::InvalidState("Download is active".to_string()));
            }
        }
        
        // Check if it's in queue
        if self.scheduler.contains(id).await {
            return Err(DownloadError::InvalidState("Download is queued".to_string()));
        }
        
        // Get the failed task
        let task = {
            let mut failed = self.failed_downloads.write().await;
            failed.remove(id)
        };
        
        let task = task.ok_or_else(|| {
            DownloadError::InvalidState("Download not found or not failed".to_string())
        })?;
        
        // Generate new ID for retry
        let new_id = Uuid::new_v4().to_string();
        let retry_task = DownloadTask {
            id: new_id.clone(),
            url: task.url,
            filename: task.filename,
            save_location: task.save_location,
        };
        
        // Emit retry event
        let _ = app.emit("download://retry", serde_json::json!({
            "id": id,
            "new_id": new_id
        }));
        
        // Check if we can start immediately
        let active_count = self.active_downloads.read().await.len();
        
        if self.scheduler.can_start(active_count).await {
            // Start immediately
            self.start_download_immediately(app, retry_task).await?;
        } else {
            // Enqueue for later
            let position = self.scheduler.enqueue(retry_task).await?;
            
            // Emit queued event
            let _ = app.emit("download://queued", serde_json::json!({
                "id": new_id,
                "position": position
            }));
            
            tracing::info!("Retry queued: {} at position {}", new_id, position);
        }

        Ok(())
    }

    /// Pause an active download
    /// 
    /// Paused downloads keep their .part file and metadata.
    /// The scheduler will start the next queued download.
    pub async fn pause_download(&self, id: &str) -> Result<()> {
        let mut downloads = self.active_downloads.write().await;
        
        if let Some(handle) = downloads.get(id) {
            // Check if already paused (pause_token is already cancelled)
            if handle.pause_token.is_cancelled() {
                return Err(DownloadError::InvalidState("Download is already paused".to_string()));
            }
            
            // Signal pause
            handle.pause_token.cancel();
            tracing::info!("Download paused: {}", id);
            
            // Remove from active downloads (so scheduler can start next)
            downloads.remove(id);
            
            return Ok(());
        }
        
        Err(DownloadError::NotFound(id.to_string()))
    }

    /// Start a download immediately
    async fn start_download_immediately(
        &self,
        app: AppHandle,
        task: DownloadTask,
    ) -> Result<()> {
        let output_path = PathBuf::from(&task.save_location).join(&task.filename);
        let part_path = self.partial_file_manager.part_path(&output_path);
        
        // Create download handle with cancellation and pause tokens
        let mut handle = DownloadHandle::new(task.id.clone());
        handle.set_output_path(output_path.to_string_lossy().to_string());

        // Add to active downloads
        self.active_downloads.write().await.insert(task.id.clone(), handle.clone());

        // Create and save metadata with partial path
        let mut metadata = DownloadMetadata::new(
            task.id.clone(),
            task.url.clone(),
            task.filename.clone(),
            output_path.clone(),
        );
        metadata.set_partial_path(part_path.clone());
        
        // Save metadata (ignore errors - don't crash download)
        if let Err(e) = self.metadata_repo.save(&metadata).await {
            tracing::warn!("Failed to save metadata: {}", e);
        }

        // Emit started event
        let _ = app.emit("download://started", serde_json::json!({
            "id": task.id
        }));

        // Clone for worker
        let cancellation_token = handle.cancellation_token.clone();
        let pause_token = handle.pause_token.clone();
        let active_downloads = self.active_downloads.clone();
        let scheduler = self.scheduler.clone();
        let failed_downloads = self.failed_downloads.clone();
        let metadata_repo = self.metadata_repo.clone();
        let resume_detector = self.resume_detector.clone();
        let partial_file_manager = self.partial_file_manager.clone();
        let task_id = task.id.clone();
        let task_for_retry = task.clone();
        
        tauri::async_runtime::spawn(async move {
            let result = Self::download_file(
                app.clone(),
                task,
                cancellation_token,
                pause_token,
                metadata_repo,
                resume_detector,
                partial_file_manager,
            ).await;
            
            if let Err(e) = result {
                if !matches!(&e, DownloadError::Cancelled) && !matches!(&e, DownloadError::Paused) {
                    tracing::error!("Download {} failed: {}", task_id, e);
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": task_id,
                        "message": e.to_string()
                    }));
                    
                    // Store for retry
                    failed_downloads.write().await.insert(task_id.clone(), task_for_retry);
                }
            }
            
            // Remove from active downloads if still there
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
                
                // Clone tokens
                let cancellation_token = handle.cancellation_token.clone();
                let pause_token = handle.pause_token.clone();
                
                // Spawn worker
                tauri::async_runtime::spawn(async move {
                    let result = Self::download_file(
                        app.clone(),
                        task,
                        cancellation_token,
                        pause_token,
                        MetadataRepository::new(),
                        ResumeCapabilityDetector::new(),
                        PartialFileManager::new(),
                    ).await;
                    
                    if let Err(e) = result {
                        if !matches!(&e, DownloadError::Cancelled) && !matches!(&e, DownloadError::Paused) {
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
                
                // Clean up .part file on cancellation
                if let Some(metadata) = self.metadata_repo.load(id).await.unwrap_or(None) {
                    if let Some(part_path) = metadata.partial_path {
                        if let Err(e) = self.partial_file_manager.cleanup(&part_path).await {
                            tracing::warn!("Failed to cleanup part file: {}", e);
                        }
                    }
                }
                
                // Delete metadata on cancellation
                if let Err(e) = self.metadata_repo.delete(id).await {
                    tracing::warn!("Failed to delete metadata: {}", e);
                }
                
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
        pause_token: tokio_util::sync::CancellationToken,
        metadata_repo: MetadataRepository,
        resume_detector: ResumeCapabilityDetector,
        partial_file_manager: PartialFileManager,
    ) -> Result<()> {
        tracing::info!("Starting download: {} -> {}", task.url, task.filename);

        let output_path = Self::build_output_path(&task).await?;
        let part_path = partial_file_manager.part_path(&output_path);
        let response = Self::send_request(&task.url).await?;
        let content_length = response.content_length();
        let mut downloaded: u64 = 0;
        
        // Detect resume capability
        let capability = resume_detector.detect(&response);
        
        // Create metadata with resume capability and partial path
        let mut metadata = DownloadMetadata::new(
            task.id.clone(),
            task.url.clone(),
            task.filename.clone(),
            output_path.clone(),
        );
        metadata.set_partial_path(part_path.clone());
        metadata.set_resume_capability(capability.resume_supported);
        
        // Save initial metadata with capability
        if let Err(e) = metadata_repo.save(&metadata).await {
            tracing::warn!("Failed to save metadata: {}", e);
        }

        // Create .part file with buffered writer for performance
        let file = tokio::fs::File::create(&part_path).await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        // Stream response body
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if cancellation_token.is_cancelled() {
                drop(writer);
                let _ = tokio::fs::remove_file(&part_path).await;
                let _ = app.emit("download://cancelled", serde_json::json!({
                    "id": task.id
                }));
                tracing::info!("Download cancelled: {}", task.filename);
                
                // Delete metadata on cancellation
                if let Err(e) = metadata_repo.delete(&task.id).await {
                    tracing::warn!("Failed to delete metadata: {}", e);
                }
                
                return Err(DownloadError::Cancelled);
            }

            // Check for pause
            if pause_token.is_cancelled() {
                // Flush and sync before exiting
                let _ = writer.flush().await;
                
                // Update final progress in metadata
                metadata.update_progress(downloaded, content_length);
                if let Err(e) = metadata_repo.update(&metadata).await {
                    tracing::warn!("Failed to update metadata on pause: {}", e);
                }
                
                let _ = app.emit("download://paused", serde_json::json!({
                    "id": task.id
                }));
                tracing::info!("Download paused: {}", task.filename);
                
                return Err(DownloadError::Paused);
            }

            let chunk = chunk_result
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

            writer.write_all(&chunk).await
                .map_err(|e| DownloadError::IoError(e.to_string()))?;

            downloaded += chunk.len() as u64;
            
            // Update metadata periodically (every 1MB or so)
            if downloaded % (1024 * 1024) == 0 || downloaded == content_length.unwrap_or(0) {
                metadata.update_progress(downloaded, content_length);
                if let Err(e) = metadata_repo.update(&metadata).await {
                    tracing::warn!("Failed to update metadata: {}", e);
                }
            }
            
            Self::emit_progress(&app, &task.id, downloaded, content_length)?;
        }

        if cancellation_token.is_cancelled() {
            drop(writer);
            let _ = tokio::fs::remove_file(&part_path).await;
            let _ = app.emit("download://cancelled", serde_json::json!({
                "id": task.id
            }));
            
            if let Err(e) = metadata_repo.delete(&task.id).await {
                tracing::warn!("Failed to delete metadata: {}", e);
            }
            
            return Err(DownloadError::Cancelled);
        }

        if pause_token.is_cancelled() {
            // Flush and sync before exiting
            let _ = writer.flush().await;
            
            // Update final progress in metadata
            metadata.update_progress(downloaded, content_length);
            if let Err(e) = metadata_repo.update(&metadata).await {
                tracing::warn!("Failed to update metadata on pause: {}", e);
            }
            
            let _ = app.emit("download://paused", serde_json::json!({
                "id": task.id
            }));
            tracing::info!("Download paused: {}", task.filename);
            
            return Err(DownloadError::Paused);
        }

        writer.flush().await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;
        
        // Atomically rename .part to final file
        if let Err(e) = partial_file_manager.finalize(&part_path).await {
            tracing::error!("Failed to finalize download: {}", e);
            let _ = app.emit("download://error", serde_json::json!({
                "id": task.id,
                "message": e.to_string()
            }));
            
            // Keep .part file for potential future resume
            return Err(DownloadError::IoError(e.to_string()));
        }

        tracing::info!("Download completed: {}", task.filename);

        // Delete metadata on completion
        if let Err(e) = metadata_repo.delete(&task.id).await {
            tracing::warn!("Failed to delete metadata: {}", e);
        }

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

    #[tokio::test]
    async fn test_retry_nonexistent() {
        let manager = DownloadManager::new();
        // Cannot create AppHandle in test, so we test the failed_downloads map directly
        let failed = manager.failed_downloads.read().await;
        assert!(!failed.contains_key("nonexistent"));
    }

    #[tokio::test]
    async fn test_pause_nonexistent() {
        let manager = DownloadManager::new();
        let result = manager.pause_download("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DownloadError::NotFound(_)));
    }
}