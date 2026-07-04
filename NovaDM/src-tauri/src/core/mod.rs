// Core module - Application state and configuration
// Central place for shared application state

pub mod config;
pub mod constants;
pub mod errors;
pub mod events;

use config::AppConfig;

#[derive(Debug, Default)]
pub struct AppState {
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
