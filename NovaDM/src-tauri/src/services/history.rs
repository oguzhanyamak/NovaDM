//! History service for business logic
//!
//! This service provides the business logic for history operations.
//! It uses HistoryRepository for persistence and handles:
//! - Saving completed/failed/cancelled downloads to history
//! - Loading history entries
//! - Deleting history entries
//! - Clearing all history

use crate::storage::{HistoryEntry, HistoryRepository, HistoryStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

/// History service for managing download history
///
/// This is a singleton service that wraps the repository.
/// It provides business logic for history operations.
pub struct HistoryService {
    repository: Arc<RwLock<HistoryRepository>>,
}

impl HistoryService {
    /// Create a new history service
    pub fn new() -> Self {
        Self {
            repository: Arc::new(RwLock::new(HistoryRepository::new())),
        }
    }

    /// Save a completed download to history
    pub async fn save_completed(
        &self,
        id: String,
        filename: String,
        url: String,
        output_path: String,
        file_size: u64,
        average_speed: u64,
        started_at: u64,
        completed_at: u64,
        duration: u64,
        checksum: Option<String>,
    ) -> std::io::Result<()> {
        let entry = HistoryEntry::new(
            id,
            filename,
            url,
            output_path,
            HistoryStatus::Completed,
            file_size,
            average_speed,
            started_at,
            completed_at,
            duration,
        );

        let entry = if let Some(checksum) = checksum {
            entry.with_checksum(checksum)
        } else {
            entry
        };

        self.repository.write().await.save(&entry).await
    }

    /// Save a failed download to history
    pub async fn save_failed(
        &self,
        id: String,
        filename: String,
        url: String,
        output_path: String,
        file_size: u64,
        average_speed: u64,
        started_at: u64,
        completed_at: u64,
        duration: u64,
    ) -> std::io::Result<()> {
        let entry = HistoryEntry::new(
            id,
            filename,
            url,
            output_path,
            HistoryStatus::Failed,
            file_size,
            average_speed,
            started_at,
            completed_at,
            duration,
        );

        self.repository.write().await.save(&entry).await
    }

    /// Save a cancelled download to history
    pub async fn save_cancelled(
        &self,
        id: String,
        filename: String,
        url: String,
        output_path: String,
        file_size: u64,
        average_speed: u64,
        started_at: u64,
        completed_at: u64,
        duration: u64,
    ) -> std::io::Result<()> {
        let entry = HistoryEntry::new(
            id,
            filename,
            url,
            output_path,
            HistoryStatus::Cancelled,
            file_size,
            average_speed,
            started_at,
            completed_at,
            duration,
        );

        self.repository.write().await.save(&entry).await
    }

    /// Load all history entries
    pub async fn load_history(&self) -> std::io::Result<Vec<HistoryEntry>> {
        self.repository.read().await.load_all().await
    }

    /// Delete a history entry
    pub async fn delete_entry(&self, id: &str) -> std::io::Result<()> {
        self.repository.write().await.delete(id).await
    }

    /// Delete multiple history entries
    pub async fn delete_entries(&self, ids: &[String]) -> std::io::Result<()> {
        self.repository.write().await.delete_multiple(ids).await
    }

    /// Clear all history
    pub async fn clear_history(&self) -> std::io::Result<()> {
        self.repository.write().await.clear().await
    }
}

impl Default for HistoryService {
    fn default() -> Self {
        Self::new()
    }
}