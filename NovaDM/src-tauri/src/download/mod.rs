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
pub mod metadata;
pub mod resume_detector;
pub mod partial_file;
pub mod recovery;

pub use manager::DownloadManager;
pub use models::DownloadInfo;
pub use errors::DownloadError;
pub use utils::resolve_filename_conflict;
pub use scheduler::{DownloadScheduler, DownloadState};
pub use metadata::{DownloadMetadata, MetadataRepository};
pub use resume_detector::{ResumeCapability, ResumeCapabilityDetector};
pub use partial_file::PartialFileManager;
pub use recovery::{RecoveryCandidate, RecoveryService};
