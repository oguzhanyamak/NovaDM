//! Settings service - Business logic layer for settings operations
//!
//! This service provides a clean API for settings operations,
//! separating business logic from the repository layer.

use crate::storage::settings::{AppSettings, SettingsRepository, Theme};

/// Settings service for managing application configuration
pub struct SettingsService {
    repository: SettingsRepository,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new() -> Self {
        Self {
            repository: SettingsRepository::new(),
        }
    }

    /// Load settings from storage
    pub async fn load(&self) -> Result<AppSettings, String> {
        self.repository
            .load()
            .await
            .map_err(|e| e.to_string())
    }

    /// Save settings to storage
    pub async fn save(&self, settings: &AppSettings) -> Result<(), String> {
        self.repository
            .save(settings)
            .await
            .map_err(|e| e.to_string())
    }

    /// Update a single setting
    pub async fn update<F>(&self, updater: F) -> Result<AppSettings, String>
    where
        F: FnOnce(&mut AppSettings) -> &mut AppSettings,
    {
        let mut settings = self.load().await?;
        updater(&mut settings);
        self.save(&settings).await?;
        Ok(settings)
    }

    /// Export settings to JSON string
    pub async fn export(&self) -> Result<String, String> {
        self.repository
            .export()
            .await
            .map_err(|e| e.to_string())
    }

    /// Import settings from JSON string
    pub async fn import(&self, json: &str) -> Result<AppSettings, String> {
        self.repository
            .import(json)
            .await
            .map_err(|e| e.to_string())
    }

    /// Reset settings to defaults
    pub async fn reset(&self) -> Result<AppSettings, String> {
        self.repository
            .reset()
            .await
            .map_err(|e| e.to_string())
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}