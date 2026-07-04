// Application configuration
// Placeholder for future configuration management

#[derive(Debug, Clone)]
pub struct AppConfig {
    // Placeholder fields
    pub version: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
        }
    }
}