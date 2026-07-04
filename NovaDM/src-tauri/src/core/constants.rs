// Application constants
// Centralized constants for the application

pub const APP_NAME: &str = "NovaDM";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_DOWNLOAD_PATH: &str = "~/Downloads/NovaDM";
pub const MAX_CONCURRENT_DOWNLOADS: usize = 3;

// Placeholder constants for future use
pub const MAX_RETRIES: u32 = 3;
pub const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB
pub const PROGRESS_UPDATE_INTERVAL_MS: u64 = 500;