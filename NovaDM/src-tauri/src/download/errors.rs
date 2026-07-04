// Download error types
// Specialized error handling for download operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Download not found: {0}")]
    NotFound(String),

    #[error("Download already exists: {0}")]
    AlreadyExists(String),

    #[error("Download failed: {0}")]
    Failed(String),
}

pub type Result<T> = std::result::Result<T, DownloadError>;