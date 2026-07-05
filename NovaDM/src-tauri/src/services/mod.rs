// Services module - Business logic layer
// Provides services for download and history operations

pub mod history;
pub mod settings;

pub use history::HistoryService;
pub use settings::SettingsService;
