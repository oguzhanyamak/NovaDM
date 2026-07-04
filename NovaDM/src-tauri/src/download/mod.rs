// Download module - Core download functionality
// Refactored into focused submodules

pub mod models;
pub mod manager;
pub mod worker;
pub mod queue;
pub mod chunk;
pub mod errors;
pub mod utils;
pub mod scheduler;

pub use manager::DownloadManager;
pub use models::DownloadInfo;
pub use errors::DownloadError;
pub use utils::resolve_filename_conflict;
pub use scheduler::{DownloadScheduler, DownloadState};
