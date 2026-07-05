//! History repository for completed, failed, and cancelled downloads
//!
//! This module provides persistent storage for download history.
//! History is separate from active download metadata to ensure:
//! - Completed downloads are immutable (no resume needed)
//! - History survives app restarts
//! - History can be cleared without affecting active downloads
//!
//! # Architecture
//!
//! History is stored in a dedicated directory with one JSON file per entry.
//! This allows for:
//! - Efficient individual entry deletion
//! - Easy backup/export
//! - Future virtualization support

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// History status for completed downloads
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryStatus {
    Completed,
    Failed,
    Cancelled,
}

/// History entry for a completed, failed, or cancelled download
///
/// This struct is immutable once created - it represents a historical record.
/// Unlike DownloadMetadata, this is not updated during download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier (matches the original download ID)
    pub id: String,
    /// Output filename
    pub filename: String,
    /// Source URL
    pub url: String,
    /// Full output path
    pub output_path: String,
    /// Final status
    pub status: HistoryStatus,
    /// Total file size in bytes
    pub file_size: u64,
    /// Average download speed in bytes/second
    pub average_speed: u64,
    /// When the download started (Unix timestamp)
    pub started_at: u64,
    /// When the download completed/failed/cancelled (Unix timestamp)
    pub completed_at: u64,
    /// Duration in seconds
    pub duration: u64,
    /// Optional checksum (for completed downloads)
    pub checksum: Option<String>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        id: String,
        filename: String,
        url: String,
        output_path: String,
        status: HistoryStatus,
        file_size: u64,
        average_speed: u64,
        started_at: u64,
        completed_at: u64,
        duration: u64,
    ) -> Self {
        Self {
            id,
            filename,
            url,
            output_path,
            status,
            file_size,
            average_speed,
            started_at,
            completed_at,
            duration,
            checksum: None,
        }
    }

    /// Create a history entry with checksum
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }
}

/// History repository for persistence
///
/// Stores history entries in a dedicated directory.
/// Each entry is stored as a separate JSON file for efficient operations.
pub struct HistoryRepository {
    /// Base directory for history files
    base_path: PathBuf,
}

impl HistoryRepository {
    /// Create a new repository
    pub fn new() -> Self {
        // Use app's local data directory
        let base_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("novadm")
            .join("history");
        Self { base_path }
    }

    /// Create a repository with a custom path (for testing)
    #[cfg(test)]
    pub fn with_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the base path
    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get the path for a history file
    fn history_path(&self, id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", id))
    }

    /// Save a history entry to disk
    pub async fn save(&self, entry: &HistoryEntry) -> std::io::Result<()> {
        let path = self.history_path(&entry.id);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write JSON
        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        tokio::fs::write(&path, json).await
    }

    /// Load a history entry from disk
    pub async fn load(&self, id: &str) -> std::io::Result<Option<HistoryEntry>> {
        let path = self.history_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let json = tokio::fs::read_to_string(&path).await?;

        let entry: HistoryEntry = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(Some(entry))
    }

    /// Load all history entries
    pub async fn load_all(&self) -> std::io::Result<Vec<HistoryEntry>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        let mut dir = tokio::fs::read_dir(&self.base_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = tokio::fs::read_to_string(&path).await {
                    if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&json) {
                        entries.push(entry);
                    }
                }
            }
        }

        // Sort by completed_at, newest first
        entries.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));

        Ok(entries)
    }

    /// Delete a history entry from disk
    pub async fn delete(&self, id: &str) -> std::io::Result<()> {
        let path = self.history_path(id);

        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        Ok(())
    }

    /// Clear all history entries
    pub async fn clear(&self) -> std::io::Result<()> {
        if !self.base_path.exists() {
            return Ok(());
        }

        let mut dir = tokio::fs::read_dir(&self.base_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    tracing::warn!("Failed to delete history file {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Delete multiple history entries
    pub async fn delete_multiple(&self, ids: &[String]) -> std::io::Result<()> {
        for id in ids {
            self.delete(id).await?;
        }
        Ok(())
    }
}

impl Default for HistoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn create_test_repo() -> HistoryRepository {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("novadm-test-history-{}", counter));
        // Clean up any existing test directory
        let _ = std::fs::remove_dir_all(&temp_dir);
        HistoryRepository::with_path(temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_load_history_entry() {
        let repo = create_test_repo();
        let id = format!("test-{}-save", std::process::id());

        let entry = HistoryEntry::new(
            id.clone(),
            "file.zip".to_string(),
            "https://example.com/file.zip".to_string(),
            "/downloads/file.zip".to_string(),
            HistoryStatus::Completed,
            1024,
            512,
            get_timestamp(),
            get_timestamp(),
            10,
        );

        // Save
        repo.save(&entry).await.unwrap();

        // Load
        let loaded = repo.load(&id).await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.filename, "file.zip");
        assert_eq!(loaded.status, HistoryStatus::Completed);
    }

    #[tokio::test]
    async fn test_load_all_history() {
        let repo = create_test_repo();
        let base_id = format!("load-all-{}", std::process::id());

        // Create multiple entries
        for i in 0..3 {
            let entry = HistoryEntry::new(
                format!("{}-{}", base_id, i),
                format!("file{}.zip", i),
                format!("https://example.com/file{}.zip", i),
                format!("/downloads/file{}.zip", i),
                HistoryStatus::Completed,
                1024 * (i as u64 + 1),
                512,
                get_timestamp(),
                get_timestamp(),
                10,
            );
            repo.save(&entry).await.unwrap();
        }

        // Load all
        let entries = repo.load_all().await.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_history_entry() {
        let repo = create_test_repo();
        let id = format!("delete-test-{}", std::process::id());

        let entry = HistoryEntry::new(
            id.clone(),
            "file.zip".to_string(),
            "https://example.com/file.zip".to_string(),
            "/downloads/file.zip".to_string(),
            HistoryStatus::Completed,
            1024,
            512,
            get_timestamp(),
            get_timestamp(),
            10,
        );

        // Save
        repo.save(&entry).await.unwrap();

        // Delete
        repo.delete(&id).await.unwrap();

        // Verify deleted
        let loaded = repo.load(&id).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_clear_history() {
        let repo = create_test_repo();
        let base_id = format!("clear-test-{}", std::process::id());

        // Create multiple entries
        for i in 0..3 {
            let entry = HistoryEntry::new(
                format!("{}-{}", base_id, i),
                format!("file{}.zip", i),
                format!("https://example.com/file{}.zip", i),
                format!("/downloads/file{}.zip", i),
                HistoryStatus::Completed,
                1024,
                512,
                get_timestamp(),
                get_timestamp(),
                10,
            );
            repo.save(&entry).await.unwrap();
        }

        // Clear
        repo.clear().await.unwrap();

        // Verify empty
        let entries = repo.load_all().await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_delete_multiple() {
        let repo = create_test_repo();
        let base_id = format!("multi-del-test-{}", std::process::id());

        // Create entries
        for i in 0..5 {
            let entry = HistoryEntry::new(
                format!("{}-{}", base_id, i),
                format!("file{}.zip", i),
                format!("https://example.com/file{}.zip", i),
                format!("/downloads/file{}.zip", i),
                HistoryStatus::Completed,
                1024,
                512,
                get_timestamp(),
                get_timestamp(),
                10,
            );
            repo.save(&entry).await.unwrap();
        }

        // Delete multiple
        let ids: Vec<String> = (0..3).map(|i| format!("{}-{}", base_id, i)).collect();
        repo.delete_multiple(&ids).await.unwrap();

        // Verify
        let entries = repo.load_all().await.unwrap();
        assert_eq!(entries.len(), 2);
    }
}
