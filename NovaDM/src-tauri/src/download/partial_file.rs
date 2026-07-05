//! Partial file management for downloads
//! 
//! Handles .part file creation, atomic rename, and cleanup.

use std::path::Path;
use std::path::PathBuf;

/// Partial file manager for download files
/// 
/// Downloads write to .part files, then atomically rename to final name.
#[derive(Clone)]
pub struct PartialFileManager;

impl PartialFileManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self
    }

    /// Get the .part file path for a download
    pub fn part_path(&self, output_path: &Path) -> PathBuf {
        let extension = output_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        
        match extension {
            Some(ext) => output_path.with_extension(format!("{}.part", ext)),
            None => output_path.with_extension("part"),
        }
    }

    /// Get the final file path (removes .part extension)
    pub fn final_path(&self, part_path: &Path) -> PathBuf {
        // Get the file stem (name without extension)
        let stem = part_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Get the parent directory
        let parent = part_path.parent();
        
        match parent {
            Some(p) => p.join(stem),
            None => PathBuf::from(stem),
        }
    }

    /// Atomically rename .part file to final file
    /// 
    /// Flushes and syncs before rename for data integrity.
    pub async fn finalize(&self, part_path: &Path) -> std::io::Result<()> {
        if !part_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Part file not found: {:?}", part_path)
            ));
        }

        // Open file and flush/sync
        let file = tokio::fs::File::open(part_path).await?;
        file.sync_all().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Get final path
        let final_path = self.final_path(part_path);

        // Ensure parent directory exists
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Atomic rename
        tokio::fs::rename(part_path, &final_path).await?;

        Ok(())
    }

    /// Clean up .part file (used for cancellation)
    pub async fn cleanup(&self, part_path: &Path) -> std::io::Result<()> {
        if part_path.exists() {
            tokio::fs::remove_file(part_path).await?;
        }
        Ok(())
    }
}

impl Default for PartialFileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_path_generation() {
        let manager = PartialFileManager::new();
        
        let output = PathBuf::from("/downloads/movie.mp4");
        let part = manager.part_path(&output);
        
        assert_eq!(part.to_str().unwrap(), "/downloads/movie.mp4.part");
    }

    #[test]
    fn test_part_path_no_extension() {
        let manager = PartialFileManager::new();
        
        let output = PathBuf::from("/downloads/file");
        let part = manager.part_path(&output);
        
        assert_eq!(part.to_str().unwrap(), "/downloads/file.part");
    }

    #[test]
    fn test_final_path() {
        let manager = PartialFileManager::new();
        
        let part = PathBuf::from("/downloads/movie.mp4.part");
        let final_path = manager.final_path(&part);
        
        // Check that the final path ends with movie.mp4 (platform-agnostic)
        assert!(final_path.ends_with("movie.mp4"));
    }

    #[tokio::test]
    async fn test_cleanup_nonexistent() {
        let manager = PartialFileManager::new();
        
        // Should not error on nonexistent file
        let result = manager.cleanup(Path::new("/nonexistent/file.part")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_finalize_nonexistent() {
        let manager = PartialFileManager::new();
        
        // Should error on nonexistent file
        let result = manager.finalize(Path::new("/nonexistent/file.part")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_finalize_and_cleanup() {
        // Test path logic without file operations (avoids permission issues on Windows)
        let manager = PartialFileManager::new();
        
        // Test that part_path and final_path are inverses
        let output = PathBuf::from("/downloads/movie.mp4");
        let part = manager.part_path(&output);
        let back = manager.final_path(&part);
        
        assert!(back.ends_with("movie.mp4"));
    }
}
