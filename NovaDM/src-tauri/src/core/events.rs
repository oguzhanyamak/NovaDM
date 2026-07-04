// Application events
// Placeholder for future event system

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Placeholder events
    DownloadStarted(String),
    DownloadProgress(String, f64),
    DownloadCompleted(String),
    DownloadFailed(String, String),
}

impl AppEvent {
    pub fn name(&self) -> &'static str {
        match self {
            AppEvent::DownloadStarted(_) => "download:started",
            AppEvent::DownloadProgress(_, _) => "download:progress",
            AppEvent::DownloadCompleted(_) => "download:completed",
            AppEvent::DownloadFailed(_, _) => "download:failed",
        }
    }
}