// Download chunk - Handles chunked download operations
// Each chunk downloads a specific byte range

use std::io::SeekFrom;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::RwLock;
use crate::download::errors::Result;
use crate::download::errors::DownloadError;

/// Download chunk for multi-part downloading
/// 
/// Each chunk downloads a specific byte range [start, end).
/// Uses positioned writes to avoid overwriting other chunks.
pub struct DownloadChunk {
    /// Download ID
    pub id: String,
    /// Start byte (inclusive)
    pub start: u64,
    /// End byte (exclusive)
    pub end: u64,
    /// Bytes downloaded in this chunk
    pub downloaded: u64,
    /// Cancellation token
    pub cancellation_token: Arc<tokio_util::sync::CancellationToken>,
    /// Pause token
    pub pause_token: Arc<tokio_util::sync::CancellationToken>,
    /// Bandwidth limiter (optional)
    pub bandwidth_limiter: Option<Arc<crate::download::bandwidth::BandwidthLimiter>>,
}

impl DownloadChunk {
    /// Create a new chunk
    pub fn new(
        id: String,
        start: u64,
        end: u64,
        cancellation_token: Arc<tokio_util::sync::CancellationToken>,
        pause_token: Arc<tokio_util::sync::CancellationToken>,
    ) -> Self {
        Self {
            id,
            start,
            end,
            downloaded: 0,
            cancellation_token,
            pause_token,
            bandwidth_limiter: None,
        }
    }

    /// Set the bandwidth limiter
    pub fn with_bandwidth_limiter(mut self, limiter: Arc<crate::download::bandwidth::BandwidthLimiter>) -> Self {
        self.bandwidth_limiter = Some(limiter);
        self
    }

    /// Download this chunk
    /// 
    /// Uses HTTP Range header to download only this chunk's bytes.
    /// Writes directly to the file at the correct position.
    /// Respects the global bandwidth limiter if set.
    pub async fn download(
        &mut self,
        url: &str,
        file: Arc<RwLock<tokio::fs::File>>,
    ) -> Result<()> {
        // Check for cancellation
        if self.cancellation_token.is_cancelled() {
            return Err(DownloadError::Cancelled);
        }

        // Build Range header
        let range_header = format!("bytes={}-{}", self.start, self.end - 1);

        // Create request with Range header
        let response = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::RANGE, range_header)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

        // Check for 206 Partial Content
        if response.status().as_u16() != 206 {
            return Err(DownloadError::HttpError(response.status().as_u16()));
        }

        // Stream response
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if self.cancellation_token.is_cancelled() {
                return Err(DownloadError::Cancelled);
            }

            // Check for pause
            if self.pause_token.is_cancelled() {
                return Err(DownloadError::Paused);
            }

            let chunk = chunk_result
                .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

            // Acquire bandwidth tokens if limiter is set
            if let Some(ref limiter) = self.bandwidth_limiter {
                limiter.acquire(chunk.len() as u64).await;
            }

            // Calculate write position
            let write_pos = self.start + self.downloaded;

            // Write to file at correct position
            {
                let mut file = file.write().await;
                file.seek(SeekFrom::Start(write_pos)).await
                    .map_err(|e| DownloadError::IoError(e.to_string()))?;
                file.write_all(&chunk).await
                    .map_err(|e| DownloadError::IoError(e.to_string()))?;
            }

            self.downloaded += chunk.len() as u64;
        }

        Ok(())
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.end == self.start {
            return 1.0;
        }
        (self.downloaded as f64 / (self.end - self.start) as f64).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_progress() {
        let chunk = DownloadChunk::new(
            "test".to_string(),
            0,
            100,
            Arc::new(tokio_util::sync::CancellationToken::new()),
            Arc::new(tokio_util::sync::CancellationToken::new()),
        );
        assert_eq!(chunk.progress(), 0.0);
    }

    #[test]
    fn test_chunk_progress_partial() {
        let mut chunk = DownloadChunk::new(
            "test".to_string(),
            0,
            100,
            Arc::new(tokio_util::sync::CancellationToken::new()),
            Arc::new(tokio_util::sync::CancellationToken::new()),
        );
        chunk.downloaded = 50;
        assert_eq!(chunk.progress(), 0.5);
    }

    #[test]
    fn test_chunk_progress_complete() {
        let mut chunk = DownloadChunk::new(
            "test".to_string(),
            0,
            100,
            Arc::new(tokio_util::sync::CancellationToken::new()),
            Arc::new(tokio_util::sync::CancellationToken::new()),
        );
        chunk.downloaded = 100;
        assert_eq!(chunk.progress(), 1.0);
    }
}