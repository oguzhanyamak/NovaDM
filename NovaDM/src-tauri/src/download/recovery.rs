//! Recovery service for crash recovery and session restoration
//! 
//! Scans metadata directory on startup to find unfinished downloads.

use serde::{Deserialize, Serialize};
use crate::download::metadata::MetadataRepository;

/// Recovery candidate for unfinished downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCandidate {
    /// Download ID
    pub download_id: String,
    /// Filename
    pub filename: String,
    /// Source URL
    pub url: String,
    /// Partial file path
    pub partial_path: String,
    /// Bytes downloaded
    pub downloaded_bytes: u64,
    /// Total bytes (if known)
    pub total_bytes: Option<u64>,
    /// Whether resume is supported
    pub resume_supported: bool,
    /// When the download was created
    pub created_at: u64,
}

/// Recovery service for scanning and validating unfinished downloads
pub struct RecoveryService {
    /// Metadata repository
    metadata_repo: MetadataRepository,
}

impl RecoveryService {
    /// Create a new recovery service
    pub fn new() -> Self {
        Self {
            metadata_repo: MetadataRepository::new(),
        }
    }

    /// Scan metadata directory for recovery candidates
    /// 
    /// Returns all valid unfinished downloads.
    /// Never fails - errors are logged and skipped.
    pub async fn scan(&self) -> Vec<RecoveryCandidate> {
        let mut candidates = Vec::new();
        
        // Get metadata directory
        let base_path = self.metadata_repo.get_base_path().to_path_buf();
        
        // Read directory
        let mut entries = match tokio::fs::read_dir(&base_path).await {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Failed to read metadata directory: {}", e);
                return candidates;
            }
        };
        
        // Process each metadata file
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let path = entry.path();
            
            // Only process .json files
            if path.extension().map(|e| e.to_str()) != Some(Some("json")) {
                continue;
            }
            
            // Try to load metadata
            let metadata = match self.metadata_repo.load_from_path(&path).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to load metadata {:?}: {}", path, e);
                    continue;
                }
            };
            
            // Validate partial file exists
            let partial_path = match &metadata.partial_path {
                Some(p) => p.clone(),
                None => {
                    tracing::warn!("No partial path in metadata {:?}", path);
                    // Delete invalid metadata
                    let _ = self.metadata_repo.delete_from_path(&path).await;
                    continue;
                }
            };
            
            if !partial_path.exists() {
                tracing::warn!("Partial file not found: {:?}", partial_path);
                // Delete metadata for missing partial file
                let _ = self.metadata_repo.delete_from_path(&path).await;
                continue;
            }
            
            // Create recovery candidate
            candidates.push(RecoveryCandidate {
                download_id: metadata.download_id,
                filename: metadata.filename,
                url: metadata.url,
                partial_path: metadata.partial_path.unwrap().to_string_lossy().to_string(),
                downloaded_bytes: metadata.downloaded_bytes,
                total_bytes: metadata.total_bytes,
                resume_supported: metadata.resume_supported,
                created_at: metadata.created_at,
            });
        }
        
        candidates
    }
}

impl Default for RecoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_directory() {
        let service = RecoveryService::new();
        let candidates = service.scan().await;
        assert!(candidates.is_empty());
    }
}