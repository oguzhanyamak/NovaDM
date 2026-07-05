//! Settings repository for application configuration
//!
//! This module provides persistent storage for application settings.
//! Settings are isolated from other modules to ensure:
//! - Single source of truth for configuration
//! - Easy import/export of settings
//! - Validation at the persistence layer

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Theme preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Dark,
    Light,
}

/// Application settings
///
/// All settings are validated on load and save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // Download Settings
    /// Default download folder
    pub download_path: String,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Global bandwidth limit in KB/s (0 = unlimited)
    pub bandwidth_limit_kb: u64,
    /// Automatically resume interrupted downloads
    pub auto_resume: bool,
    /// Retry failed downloads automatically
    pub auto_retry: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,

    // Appearance
    /// Theme preference
    pub theme: Theme,

    // General
    /// Open NovaDM on startup
    pub open_on_startup: bool,
    /// Check for updates automatically
    pub auto_check_updates: bool,
    /// Enable notifications
    pub enable_notifications: bool,
    /// Enable browser integration
    pub enable_browser_integration: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_path: "~/Downloads/NovaDM".to_string(),
            max_concurrent_downloads: 3,
            bandwidth_limit_kb: 0,
            auto_resume: true,
            auto_retry: true,
            max_retry_attempts: 3,
            theme: Theme::System,
            open_on_startup: false,
            auto_check_updates: true,
            enable_notifications: true,
            enable_browser_integration: false,
        }
    }
}

impl AppSettings {
    /// Validate settings values
    pub fn validate(&self) -> Result<(), String> {
        // Validate download folder exists (or can be created)
        let path = self.download_path.replace("~", 
            dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default().as_str()
        );
        if !Path::new(&path).exists() {
            // We allow non-existent paths as they may be created later
        }

        // Validate concurrent downloads > 0
        if self.max_concurrent_downloads == 0 {
            return Err("Maximum concurrent downloads must be greater than 0".to_string());
        }

        // Validate retry count >= 0
        if self.max_retry_attempts == 0 && self.auto_retry {
            return Err("Maximum retry attempts must be greater than 0 when auto-retry is enabled".to_string());
        }

        // Validate bandwidth >= 0 (already enforced by type)

        Ok(())
    }
}

/// Settings repository for persistence
pub struct SettingsRepository {
    /// Path to settings file
    settings_path: PathBuf,
}

impl SettingsRepository {
    /// Create a new repository
    pub fn new() -> Self {
        let settings_path = dirs::config_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("novadm")
            .join("settings.json");
        Self { settings_path }
    }

    /// Get the settings file path
    pub fn get_path(&self) -> &Path {
        &self.settings_path
    }

    /// Load settings from disk
    pub async fn load(&self) -> std::io::Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let json = tokio::fs::read_to_string(&self.settings_path).await?;

        let settings: AppSettings = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Validate loaded settings
        if let Err(e) = settings.validate() {
            tracing::warn!("Invalid settings loaded, using defaults: {}", e);
            return Ok(AppSettings::default());
        }

        Ok(settings)
    }

    /// Save settings to disk
    pub async fn save(&self, settings: &AppSettings) -> std::io::Result<()> {
        // Validate before saving
        if let Err(e) = settings.validate() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }

        // Ensure directory exists
        if let Some(parent) = self.settings_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write JSON
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        tokio::fs::write(&self.settings_path, json).await
    }

    /// Export settings to a JSON string
    pub async fn export(&self) -> std::io::Result<String> {
        let settings = self.load().await?;
        serde_json::to_string_pretty(&settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Import settings from a JSON string
    pub async fn import(&self, json: &str) -> std::io::Result<AppSettings> {
        let settings: AppSettings = serde_json::from_str(json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Validate imported settings
        settings.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Save the imported settings
        self.save(&settings).await?;

        Ok(settings)
    }

    /// Reset to defaults
    pub async fn reset(&self) -> std::io::Result<AppSettings> {
        let settings = AppSettings::default();
        self.save(&settings).await?;
        Ok(settings)
    }
}

impl Default for SettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_repo() -> SettingsRepository {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("novadm-test-settings-{}", counter));
        let _ = std::fs::remove_dir_all(&temp_dir);
        SettingsRepository::with_path(temp_dir)
    }

    #[cfg(test)]
    impl SettingsRepository {
        pub fn with_path(base_dir: PathBuf) -> Self {
            Self {
                settings_path: base_dir.join("settings.json"),
            }
        }
    }

    #[tokio::test]
    async fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let repo = create_test_repo();
        let settings = repo.load().await.unwrap();
        assert_eq!(settings.max_concurrent_downloads, 3);
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let repo = create_test_repo();
        let settings = AppSettings {
            download_path: "/test/path".to_string(),
            max_concurrent_downloads: 5,
            bandwidth_limit_kb: 1024,
            auto_resume: false,
            auto_retry: false,
            max_retry_attempts: 5,
            theme: Theme::Dark,
            open_on_startup: true,
            auto_check_updates: false,
            enable_notifications: false,
            enable_browser_integration: true,
        };

        repo.save(&settings).await.unwrap();
        let loaded = repo.load().await.unwrap();
        assert_eq!(loaded.download_path, "/test/path");
        assert_eq!(loaded.max_concurrent_downloads, 5);
        assert_eq!(loaded.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_export_import() {
        let repo = create_test_repo();
        let settings = AppSettings {
            download_path: "/export/test".to_string(),
            max_concurrent_downloads: 10,
            bandwidth_limit_kb: 512,
            auto_resume: true,
            auto_retry: true,
            max_retry_attempts: 2,
            theme: Theme::Light,
            open_on_startup: false,
            auto_check_updates: true,
            enable_notifications: true,
            enable_browser_integration: false,
        };

        repo.save(&settings).await.unwrap();
        let exported = repo.export().await.unwrap();
        
        // Create new repo and import
        let repo2 = create_test_repo();
        let imported = repo2.import(&exported).await.unwrap();
        
        assert_eq!(imported.download_path, "/export/test");
        assert_eq!(imported.max_concurrent_downloads, 10);
    }

    #[tokio::test]
    async fn test_reset() {
        let repo = create_test_repo();
        let settings = AppSettings {
            download_path: "/custom/path".to_string(),
            max_concurrent_downloads: 10,
            bandwidth_limit_kb: 100,
            auto_resume: false,
            auto_retry: false,
            max_retry_attempts: 10,
            theme: Theme::Light,
            open_on_startup: true,
            auto_check_updates: false,
            enable_notifications: false,
            enable_browser_integration: true,
        };

        repo.save(&settings).await.unwrap();
        let reset = repo.reset().await.unwrap();
        
        assert_eq!(reset.download_path, "~/Downloads/NovaDM");
        assert_eq!(reset.max_concurrent_downloads, 3);
    }

    #[tokio::test]
    async fn test_validation() {
        let mut settings = AppSettings::default();
        
        // Test invalid concurrent downloads
        settings.max_concurrent_downloads = 0;
        assert!(settings.validate().is_err());
        
        // Test valid settings
        settings.max_concurrent_downloads = 1;
        settings.auto_retry = false;
        assert!(settings.validate().is_ok());
    }
}