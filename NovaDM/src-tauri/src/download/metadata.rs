//! Download metadata for persistence
//! 
//! Stores download state to enable pause/resume in the future.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Download metadata stored on disk
/// 
/// This struct persists download state for potential resume support.
/// Additional fields can be added without breaking existing data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetadata {
    /// Unique download identifier
    pub download_id: String,
    /// Source URL
    pub url: String,
    /// Output filename
    pub filename: String,
    /// Full output path (final file)
    pub output_path: PathBuf,
    /// Partial file path (for incomplete downloads)
    #[serde(default)]
    pub partial_path: Option<PathBuf>,
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// HTTP ETag for resume validation
    pub etag: Option<String>,
    /// Last-Modified header for resume validation
    pub last_modified: Option<String>,
    /// When the download was created
    pub created_at: u64,
    /// When the metadata was last updated
    pub updated_at: u64,
    /// Whether the server supports resumable downloads
    #[serde(default)]
    pub resume_supported: bool,
}

impl DownloadMetadata {
    /// Create new metadata for a download
    pub fn new(
        download_id: String,
        url: String,
        filename: String,
        output_path: PathBuf,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            download_id,
            url,
            filename,
            output_path,
            partial_path: None,
            total_bytes: None,
            downloaded_bytes: 0,
            etag: None,
            last_modified: None,
            created_at: now,
            updated_at: now,
            resume_supported: false,
        }
    }

    /// Set partial path
    pub fn set_partial_path(&mut self, path: PathBuf) {
        self.partial_path = Some(path);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Set resume capability
    pub fn set_resume_capability(&mut self, supported: bool) {
        self.resume_supported = supported;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Update downloaded bytes
    pub fn update_progress(&mut self, downloaded: u64, total: Option<u64>) {
        self.downloaded_bytes = downloaded;
        if let Some(t) = total {
            self.total_bytes = Some(t);
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Update HTTP headers for resume
    pub fn update_headers(&mut self, etag: Option<&str>, last_modified: Option<&str>) {
        if let Some(e) = etag {
            self.etag = Some(e.to_string());
        }
        if let Some(l) = last_modified {
            self.last_modified = Some(l.to_string());
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Metadata repository for persistence
/// 
/// Handles reading/writing metadata to disk.
/// Errors are logged but do not crash downloads.
#[derive(Clone)]
pub struct MetadataRepository {
    /// Base directory for metadata files
    base_path: PathBuf,
}

impl MetadataRepository {
    /// Create a new repository
    pub fn new() -> Self {
        // Use app's local data directory
        let base_path = std::env::temp_dir().join("novadm").join("metadata");
        Self { base_path }
    }

    /// Get the base path
    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get the path for a metadata file
    fn metadata_path(&self, download_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", download_id))
    }

    /// Save metadata to disk
    pub async fn save(&self, metadata: &DownloadMetadata) -> std::io::Result<()> {
        let path = self.metadata_path(&metadata.download_id);
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Write JSON
        let json = serde_json::to_string_pretty(metadata)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        tokio::fs::write(&path, json).await
    }

    /// Load metadata from disk
    pub async fn load(&self, download_id: &str) -> std::io::Result<Option<DownloadMetadata>> {
        let path = self.metadata_path(download_id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let json = tokio::fs::read_to_string(&path).await?;
        
        let metadata: DownloadMetadata = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(Some(metadata))
    }

    /// Update metadata on disk
    pub async fn update(&self, metadata: &DownloadMetadata) -> std::io::Result<()> {
        self.save(metadata).await
    }

    /// Delete metadata from disk
    pub async fn delete(&self, download_id: &str) -> std::io::Result<()> {
        let path = self.metadata_path(download_id);
        
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        Ok(())
    }

    /// Load metadata from a specific path
    pub async fn load_from_path(&self, path: &Path) -> std::io::Result<DownloadMetadata> {
        let json = tokio::fs::read_to_string(path).await?;
        
        let metadata: DownloadMetadata = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(metadata)
    }

    /// Delete metadata from a specific path
    pub async fn delete_from_path(&self, path: &Path) -> std::io::Result<()> {
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }
}

impl Default for MetadataRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_save_and_load_metadata() {
        let repo = MetadataRepository::new();
        
        let metadata = DownloadMetadata::new(
            "test-id".to_string(),
            "https://example.com/file.zip".to_string(),
            "file.zip".to_string(),
            PathBuf::from("/downloads/file.zip"),
        );
        
        // Save
        repo.save(&metadata).await.unwrap();
        
        // Load
        let loaded = repo.load("test-id").await.unwrap();
        assert!(loaded.is_some());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.download_id, "test-id");
        assert_eq!(loaded.url, "https://example.com/file.zip");
        
        // Cleanup
        repo.delete("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn test_update_metadata() {
        let repo = MetadataRepository::new();
        
        let mut metadata = DownloadMetadata::new(
            "update-test".to_string(),
            "https://example.com/file.zip".to_string(),
            "file.zip".to_string(),
            PathBuf::from("/downloads/file.zip"),
        );
        
        // Save initial
        repo.save(&metadata).await.unwrap();
        
        // Update
        metadata.update_progress(1024, Some(2048));
        repo.update(&metadata).await.unwrap();
        
        // Load and verify
        let loaded = repo.load("update-test").await.unwrap().unwrap();
        assert_eq!(loaded.downloaded_bytes, 1024);
        assert_eq!(loaded.total_bytes, Some(2048));
        
        // Cleanup
        repo.delete("update-test").await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_metadata() {
        let repo = MetadataRepository::new();
        
        let metadata = DownloadMetadata::new(
            "delete-test".to_string(),
            "https://example.com/file.zip".to_string(),
            "file.zip".to_string(),
            PathBuf::from("/downloads/file.zip"),
        );
        
        // Save
        repo.save(&metadata).await.unwrap();
        
        // Delete
        repo.delete("delete-test").await.unwrap();
        
        // Verify deleted
        let loaded = repo.load("delete-test").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let repo = MetadataRepository::new();
        
        let loaded = repo.load("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_backward_compatibility() {
        // Test that old metadata without resume_supported field can be loaded
        let repo = MetadataRepository::new();
        
        // Create old-style JSON (without resume_supported)
        let old_json = r#"{
            "download_id": "compat-test",
            "url": "https://example.com/file.zip",
            "filename": "file.zip",
            "output_path": "/downloads/file.zip",
            "total_bytes": 1024,
            "downloaded_bytes": 512,
            "etag": null,
            "last_modified": null,
            "created_at": 1693520000,
            "updated_at": 1693520060
        }"#;
        
        // Write old-style JSON
        let path = repo.metadata_path("compat-test");
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(&path, old_json).await.unwrap();
        
        // Load should succeed with default value
        let loaded = repo.load("compat-test").await.unwrap();
        assert!(loaded.is_some());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.download_id, "compat-test");
        assert_eq!(loaded.downloaded_bytes, 512);
        // resume_supported should default to false
        assert!(!loaded.resume_supported);
        
        // Cleanup
        repo.delete("compat-test").await.unwrap();
    }
}
