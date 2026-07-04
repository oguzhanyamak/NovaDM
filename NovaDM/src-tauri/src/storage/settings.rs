// Settings management
// Placeholder for application settings

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_path: String,
    pub max_concurrent_downloads: usize,
    pub auto_start: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_path: "~/Downloads/NovaDM".to_string(),
            max_concurrent_downloads: 3,
            auto_start: false,
        }
    }
}

pub struct SettingsManager {
    settings: AppSettings,
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            settings: AppSettings::default(),
        }
    }

    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn update_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}