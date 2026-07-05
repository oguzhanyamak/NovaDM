//! Download error types
//! Specialized error handling for download operations

use thiserror::Error;

/// Errors that can occur during download operations
#[derive(Error, Debug)]
pub enum DownloadError {
    /// Network-related error (connection, timeout, etc.)
    #[error("Network error: {0}")]
    NetworkError(String),

    /// HTTP error with status code
    #[error("HTTP error: {0}")]
    HttpError(u16),

    /// File I/O error
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid URL provided
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Download not found
    #[error("Download not found: {0}")]
    NotFound(String),

    /// Download already exists
    #[error("Download already exists: {0}")]
    AlreadyExists(String),

    /// Download was cancelled by user
    #[error("Download cancelled")]
    Cancelled,

    /// Download was paused by user
    #[error("Download paused")]
    Paused,

    /// File changed on server
    #[error("File changed on server")]
    FileChanged,

    /// Resume not supported
    #[error("Resume not supported")]
    ResumeUnsupported,

    /// Permission denied
    #[error("Permission denied")]
    PermissionDenied,

    /// Disk full
    #[error("Disk full")]
    DiskFull,

    /// Network timeout
    #[error("Network timeout")]
    Timeout,

    /// Network disconnected
    #[error("Network disconnected")]
    NetworkDisconnected,

    /// Invalid state for operation
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Result type alias for download operations
pub type Result<T> = std::result::Result<T, DownloadError>;