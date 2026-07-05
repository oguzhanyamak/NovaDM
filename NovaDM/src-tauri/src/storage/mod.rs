// Storage module - Application settings and configuration
// Handles both application settings and download history

pub mod history;
pub mod settings;

pub use history::{HistoryEntry, HistoryRepository, HistoryStatus};
pub use settings::{AppSettings, SettingsRepository, Theme};
