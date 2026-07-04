//! Download utility functions
//! 
//! Helper functions for file operations and error handling.

use std::path::Path;
use crate::download::errors::DownloadError;

/// Resolve filename conflicts by appending a number
/// 
/// If the file already exists, rename it:
/// - movie.mp4 → movie (1).mp4
/// - movie (1).mp4 → movie (2).mp4
/// 
/// # Arguments
/// * `path` - The original file path
/// # Returns
/// * `Ok(PathBuf)` - A unique file path
pub fn resolve_filename_conflict(path: &Path) -> std::io::Result<std::path::PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }

    let parent = path.parent().unwrap_or(Path::new(""));
    let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().map(|e| format!(".{}", e.to_string_lossy()));
    
    let mut counter = 1;
    loop {
        let new_name = match &extension {
            Some(ext) => format!("{} ({}){}", file_name, counter, ext),
            None => format!("{} ({})", file_name, counter),
        };
        let new_path = parent.join(&new_name);
        
        if !new_path.exists() {
            return Ok(new_path);
        }
        
        counter += 1;
    }
}

/// Categorize IO errors into user-friendly messages
pub fn categorize_io_error(error: &std::io::Error) -> DownloadError {
    use std::io::ErrorKind;
    
    match error.kind() {
        ErrorKind::PermissionDenied => DownloadError::PermissionDenied,
        ErrorKind::NotFound => DownloadError::DiskFull, // Directory doesn't exist
        ErrorKind::StorageFull | ErrorKind::OutOfMemory => DownloadError::DiskFull,
        _ => DownloadError::IoError(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_resolve_no_conflict() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("unique_test_file_12345.txt");
        
        // Ensure file doesn't exist
        let _ = fs::remove_file(&path);
        
        let result = resolve_filename_conflict(&path).unwrap();
        assert_eq!(result, path);
        
        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_resolve_with_conflict() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("conflict_test_file_12345.txt");
        
        // Create the file
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"test").unwrap();
        
        let result = resolve_filename_conflict(&path).unwrap();
        assert!(result.to_string_lossy().contains("conflict_test_file_12345 (1)"));
        
        // Cleanup
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&result);
    }

    #[test]
    fn test_resolve_multiple_conflicts() {
        let temp_dir = std::env::temp_dir();
        let base_path = temp_dir.join("multi_conflict_test_12345.txt");
        
        // Create multiple files
        let mut file1 = fs::File::create(&base_path).unwrap();
        file1.write_all(b"test").unwrap();
        
        let path1 = temp_dir.join("multi_conflict_test_12345 (1).txt");
        let mut file2 = fs::File::create(&path1).unwrap();
        file2.write_all(b"test").unwrap();
        
        let result = resolve_filename_conflict(&base_path).unwrap();
        assert!(result.to_string_lossy().contains("multi_conflict_test_12345 (2)"));
        
        // Cleanup
        let _ = fs::remove_file(&base_path);
        let _ = fs::remove_file(&path1);
        let _ = fs::remove_file(&result);
    }
}