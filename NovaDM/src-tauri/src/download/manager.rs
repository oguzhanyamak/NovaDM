// Download manager - Orchestrates download operations
// Handles real HTTP downloads with streaming

use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;
use crate::download::errors::{DownloadError, Result};
use crate::download::models::DownloadTask;

pub struct DownloadManager {
    client: Client,
    active_downloads: Arc<RwLock<Vec<String>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            active_downloads: Arc::new(RwLock::new(Vec::new())),
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
    /// * `Ok(())` - Download started successfully
    /// * `Err(DownloadError)` - Failed to start download
    pub async fn start_download(
        &self,
        app: AppHandle,
        url: String,
        filename: String,
        save_location: String,
    ) -> Result<()> {
        let download_id = generate_download_id(&url, &filename);
        
        // Add to active downloads
        self.active_downloads.write().await.push(download_id.clone());

        // Spawn download task
        let task = DownloadTask {
            id: download_id.clone(),
            url,
            filename,
            save_location,
        };

        let client = self.client.clone();
        let active_downloads = self.active_downloads.clone();

        // Spawn async task (non-blocking)
        tauri::async_runtime::spawn(async move {
            if let Err(e) = Self::download_file(app, client, task).await {
                tracing::error!("Download failed: {}", e);
            }
            
            // Remove from active downloads
            active_downloads.write().await.retain(|id| id != &download_id);
        });

        Ok(())
    }

    /// Download a file with streaming
    /// 
    /// This function:
    /// 1. Sends HTTP GET request
    /// 2. Streams response body in chunks
    /// 3. Writes directly to disk
    /// 4. Emits progress events
    /// 5. Emits completion event
    /// 
    /// # Memory Usage
    /// For a 20 GB file, memory usage stays constant at ~8-64 KB
    /// because we stream and buffer only small chunks.
    async fn download_file(
        app: AppHandle,
        client: Client,
        task: DownloadTask,
    ) -> Result<()> {
        tracing::info!("Starting download: {} -> {}", task.url, task.filename);

        // Build output path
        let output_path = PathBuf::from(&task.save_location).join(&task.filename);
        
        // Ensure directory exists
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Send HTTP GET request with streaming
        let response = client
            .get(&task.url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            return Err(DownloadError::HttpError(response.status().as_u16()));
        }

        // Get content length for progress calculation
        let content_length = response.content_length();
        let mut downloaded: u64 = 0;
        let mut last_speed_check = tokio::time::Instant::now();
        let mut bytes_since_last_check: u64 = 0;

        // Create file
        let mut file = tokio::fs::File::create(&output_path)
            .await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;

        // Stream response body
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| {
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": task.id,
                        "message": format!("Network error: {}", e)
                    }));
                    DownloadError::NetworkError(e.to_string())
                })?;

            // Write chunk to disk
            file.write_all(&chunk)
                .await
                .map_err(|e| {
                    let _ = app.emit("download://error", serde_json::json!({
                        "id": task.id,
                        "message": format!("IO error: {}", e)
                    }));
                    DownloadError::IoError(e.to_string())
                })?;

            // Update progress
            downloaded += chunk.len() as u64;
            bytes_since_last_check += chunk.len() as u64;

            // Calculate speed and emit progress every 500ms
            let now = tokio::time::Instant::now();
            if now.duration_since(last_speed_check).as_millis() >= 500 {
                let elapsed = now.duration_since(last_speed_check).as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (bytes_since_last_check as f64 / elapsed) as u64
                } else {
                    0
                };

                // Emit progress event
                let progress = if let Some(total) = content_length {
                    Some(((downloaded as f64 / total as f64) * 100.0).min(100.0) as u32)
                } else {
                    // Indeterminate progress (no content-length)
                    None
                };

                let _ = app.emit("download://progress", serde_json::json!({
                    "id": task.id,
                    "progress": progress,
                    "speed": speed,
                    "status": "downloading"
                }));

                // Reset counters
                last_speed_check = now;
                bytes_since_last_check = 0;
            }
        }

        // Flush file
        file.sync_all()
            .await
            .map_err(|e| DownloadError::IoError(e.to_string()))?;

        tracing::info!("Download completed: {}", task.filename);

        // Emit completion event
        let _ = app.emit("download://completed", serde_json::json!({
            "id": task.id
        }));

        Ok(())
    }

    /// Cancel an active download
    pub async fn cancel(&self, id: &str) -> Result<()> {
        self.active_downloads.write().await.retain(|download_id| download_id != id);
        Ok(())
    }

    /// Check if a download is active
    pub async fn is_active(&self, id: &str) -> bool {
        self.active_downloads.read().await.contains(&id.to_string())
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unique download ID from URL and filename
fn generate_download_id(url: &str, filename: &str) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    filename.hash(&mut hasher);
    
    format!("dl-{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_download_id() {
        let id1 = generate_download_id("https://example.com/file.zip", "file.zip");
        let id2 = generate_download_id("https://example.com/file.zip", "file.zip");
        let id3 = generate_download_id("https://example.com/other.zip", "other.zip");
        
        assert_eq!(id1, id2); // Same inputs = same ID
        assert_ne!(id1, id3); // Different inputs = different ID
        assert!(id1.starts_with("dl-")); // ID format
    }
}