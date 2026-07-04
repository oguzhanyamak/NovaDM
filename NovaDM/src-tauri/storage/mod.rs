// Storage management module
// Handles persistent storage for downloads and settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub download_path: PathBuf,
    pub max_concurrent_downloads: usize,
    pub auto_start: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            download_path: dirs::download_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("NovaDM"),
            max_concurrent_downloads: 3,
            auto_start: false,
        }
    }
}

pub struct StorageManager {
    config: StorageConfig,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            config: StorageConfig::default(),
        }
    }

    pub fn get_config(&self) -> &StorageConfig {
        &self.config
    }

    pub fn set_download_path(&mut self, path: PathBuf) {
        self.config.download_path = path;
    }

    pub fn ensure_download_dir(&self) -> Result<PathBuf, String> {
        let path = &self.config.download_path;
        std::fs::create_dir_all(path)
            .map(|_| path.clone())
            .map_err(|e| e.to_string())
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}