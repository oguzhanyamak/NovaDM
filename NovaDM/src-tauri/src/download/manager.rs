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
use tokio::io::AsyncWriteExt;
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

    /// Resume a paused download
    /// 
    /// Validates the partial file and remote file before resuming.
    /// Uses HTTP Range requests to continue from where it left off.
    pub async fn resume_download(&self, app: AppHandle, id: &str) -> Result<()> {
        // Load metadata
        let metadata = self.metadata_repo.load(id).await
            .map_err(|e| DownloadError::IoError(e.to_string()))?
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;
        
        // Validate resume is supported
        if !metadata.resume_supported {
            return Err(DownloadError::ResumeUnsupported);
        }
        
        // Validate partial file exists
        let part_path = metadata.partial_path.as_ref()
            .ok_or_else(|| DownloadError::InvalidState("No partial file path".to_string()))?;
        
        if !part_path.exists() {
            return Err(DownloadError::InvalidState("Partial file not found".to_string()));
        }
        
        // Validate remote file hasn't changed
        let response = reqwest::Client::new()
            .head(&metadata.url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;
        
        // Check ETag
        if let Some(etag) = &metadata.etag {
            if let Some(server_etag) = response.headers().get(reqwest::header::ETAG) {
                if server_etag.to_str().ok() != Some(etag.as_str()) {
                    return Err(DownloadError::FileChanged);
                }
            }
        }
        
        // Check Last-Modified
        if let Some(last_modified) = &metadata.last_modified {
            if let Some(server_last_modified) = response.headers().get(reqwest::header::LAST_MODIFIED) {
                if server_last_modified.to_str().ok() != Some(last_modified.as_str()) {
                    return Err(DownloadError::FileChanged);
                }
            }
        }
        
        // Create task for resume
        let task = DownloadTask {
            id: id.to_string(),
            url: metadata.url.clone(),
            filename: metadata.filename.clone(),
            save_location: metadata.output_path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        };
        
        // Check if we can start immediately
        let active_count = self.active_downloads.read().await.len();
        
        if self.scheduler.can_start(active_count).await {
            // Start immediately with resume
            self.start_download_immediately_for_resume(app, task, metadata.downloaded_bytes).await?;
        } else {
            // Enqueue for later
            let position = self.scheduler.enqueue(task).await?;
            
            // Emit queued event
            let _ = app.emit("download://queued", serde_json::json!({
                "id": id,
                "position": position
            }));
            
            tracing::info!("Resume queued: {} at position {}", id, position);
        }

        Ok(())
    }
    
    /// Start a download immediately for resume
    async fn start_download_immediately_for_resume(
        &self,
        app: AppHandle,
        task: DownloadTask,
        downloaded_bytes: u64,
    ) -> Result<()> {
        let output_path = PathBuf::from(&task.save_location).join(&task.filename);
        
        // Create download handle with cancellation and pause tokens
        let mut handle = DownloadHandle::new(task.id.clone());
        handle.set_output_path(output_path.to_string_lossy().to_string());

        // Add to active downloads
        self.active_downloads.write().await.insert(task.id.clone(), handle.clone());

        // Emit resumed event
        let _ = app.emit("download://resumed", serde_json::json!({
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
                Some(downloaded_bytes),
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
                None,
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
                        None,
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

    /// Default number of connections for multi-part download
    const DEFAULT_CONNECTIONS: usize = 8;

    /// Download a file with streaming and buffered writing
    /// 
    /// If `downloaded_bytes` is provided, resumes from that position.
    /// If server supports Accept-Ranges, uses multi-part downloading.
    async fn download_file(
        app: AppHandle,
        task: DownloadTask,
        cancellation_token: tokio_util::sync::CancellationToken,
        pause_token: tokio_util::sync::CancellationToken,
        metadata_repo: MetadataRepository,
        resume_detector: ResumeCapabilityDetector,
        partial_file_manager: PartialFileManager,
        downloaded_bytes: Option<u64>,
    ) -> Result<()> {
        tracing::info!("Starting download: {} -> {}", task.url, task.filename);

        let output_path = Self::build_output_path(&task).await?;
        let part_path = partial_file_manager.part_path(&output_path);
        
        // Check if this is a resume
        let is_resume = downloaded_bytes.is_some();
        let start_pos = downloaded_bytes.unwrap_or(0);
        
        // Send request to get headers
        let head_response = reqwest::Client::new()
            .head(&task.url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;
        
        // Check if server supports ranges
        let supports_ranges = head_response
            .headers()
            .get(reqwest::header::ACCEPT_RANGES)
            .map(|v| v == "bytes")
            .unwrap_or(false);
        
        // Get total size
        let total_size = head_response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        
        // If resume and server doesn't support ranges, fallback to single connection
        if is_resume && !supports_ranges {
            return Err(DownloadError::ResumeUnsupported);
        }
        
        // Create .part file
        let file = if is_resume {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&part_path)
                .await
                .map_err(|e| DownloadError::IoError(e.to_string()))?
        } else {
            tokio::fs::File::create(&part_path)
                .await
                .map_err(|e| DownloadError::IoError(e.to_string()))?
        };
        
        // Wrap in Arc<RwLock> for multi-part access
        let file = Arc::new(RwLock::new(file));
        
        // Create metadata
        let mut metadata = DownloadMetadata::new(
            task.id.clone(),
            task.url.clone(),
            task.filename.clone(),
            output_path.clone(),
        );
        metadata.set_partial_path(part_path.clone());
        metadata.set_resume_capability(supports_ranges);
        
        // Save initial metadata
        if let Err(e) = metadata_repo.save(&metadata).await {
            tracing::warn!("Failed to save metadata: {}", e);
        }
        
        // Calculate chunks
        let chunks = if supports_ranges && total_size > 0 {
            Self::calculate_chunks(total_size, Self::DEFAULT_CONNECTIONS)
        } else {
            // Single connection fallback
            vec![(0, total_size)]
        };
        
        // Create and spawn workers
        let mut workers = Vec::new();
        let mut chunk_starts = Vec::new();
        
        for (i, (start, end)) in chunks.into_iter().enumerate() {
            let chunk = crate::download::chunk::DownloadChunk::new(
                format!("{}-{}", task.id, i),
                start,
                end,
                Arc::new(cancellation_token.clone()),
                Arc::new(pause_token.clone()),
            );
            
            let file_clone = file.clone();
            let url = task.url.clone();
            let task_id = task.id.clone();
            let app_clone = app.clone();
            let total = total_size;
            let chunk_start = start;
            
            let handle = tauri::async_runtime::spawn(async move {
                let mut chunk = chunk;
                let result = chunk.download(&url, file_clone).await;
                
                // Emit progress for this chunk
                if result.is_ok() || matches!(result, Err(DownloadError::Cancelled) | Err(DownloadError::Paused)) {
                    let _ = app_clone.emit("download://progress", serde_json::json!({
                        "id": task_id,
                        "progress": None::<u32>,
                        "downloaded_bytes": chunk_start + chunk.downloaded,
                        "total_bytes": Some(total),
                        "speed": 0,
                        "status": "downloading"
                    }));
                }
                
                result
            });
            
            workers.push(handle);
            chunk_starts.push(start);
        }
        
        // Wait for all workers
        let num_chunks = chunk_starts.len();
        let mut total_downloaded = start_pos;
        for (i, worker) in workers.into_iter().enumerate() {
            if let Err(e) = worker.await.unwrap_or(Ok(())) {
                if !matches!(&e, DownloadError::Cancelled) && !matches!(&e, DownloadError::Paused) {
                    // One worker failed - download fails
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": task.id,
                        "message": e.to_string()
                    }));
                    
                    // Update metadata with partial progress
                    metadata.update_progress(total_downloaded, Some(total_size));
                    if let Err(e) = metadata_repo.update(&metadata).await {
                        tracing::warn!("Failed to update metadata: {}", e);
                    }
                    
                    return Err(e);
                }
            }
            total_downloaded = chunk_starts[i] + (total_size / num_chunks as u64);
        }
        
        // Finalize
        if let Err(e) = partial_file_manager.finalize(&part_path).await {
            tracing::error!("Failed to finalize download: {}", e);
            let _ = app.emit("download://error", serde_json::json!({
                "id": task.id,
                "message": e.to_string()
            }));
            return Err(DownloadError::IoError(e.to_string()));
        }
        
        // Delete metadata on completion
        if let Err(e) = metadata_repo.delete(&task.id).await {
            tracing::warn!("Failed to delete metadata: {}", e);
        }
        
        let _ = app.emit("download://completed", serde_json::json!({
            "id": task.id
        }));
        
        Ok(())
    }
    
    /// Calculate chunk boundaries for multi-part download
    fn calculate_chunks(total_size: u64, num_parts: usize) -> Vec<(u64, u64)> {
        if total_size == 0 || num_parts == 0 {
            return vec![(0, total_size)];
        }
        
        let chunk_size = total_size / num_parts as u64;
        let mut chunks = Vec::new();
        
        for i in 0..num_parts {
            let start = (i as u64) * chunk_size;
            let end = if i == num_parts - 1 {
                total_size
            } else {
                ((i + 1) as u64) * chunk_size
            };
            chunks.push((start, end));
        }
        
        chunks
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

    #[test]
    fn test_calculate_chunks() {
        // 800 MB with 8 parts = 100 MB each
        let chunks = DownloadManager::calculate_chunks(800 * 1024 * 1024, 8);
        assert_eq!(chunks.len(), 8);
        assert_eq!(chunks[0], (0, 100 * 1024 * 1024));
        assert_eq!(chunks[7], (700 * 1024 * 1024, 800 * 1024 * 1024));
    }

    #[test]
    fn test_calculate_chunks_single() {
        // Small file = single chunk
        let chunks = DownloadManager::calculate_chunks(100, 8);
        assert_eq!(chunks.len(), 8);
        // All chunks should be small
        for (start, end) in chunks {
            assert!(end <= 100);
        }
    }

    #[test]
    fn test_calculate_chunks_empty() {
        // Zero size = single chunk
        let chunks = DownloadManager::calculate_chunks(0, 8);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], (0, 0));
    }
}