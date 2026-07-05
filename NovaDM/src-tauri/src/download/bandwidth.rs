//! Bandwidth limiter using token bucket algorithm
//! 
//! Provides global bandwidth limiting for all downloads.
//! Thread-safe and configurable.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// Bandwidth limiter using token bucket algorithm
/// 
/// The token bucket algorithm allows bursts up to the bucket size
/// while maintaining an average rate. This is ideal for download
/// limiting because:
/// - It allows natural bursts when bandwidth is available
/// - It doesn't require busy waiting
/// - It's simple and efficient
#[derive(Clone)]
pub struct BandwidthLimiter {
    /// Maximum bytes per second (0 = unlimited)
    max_bytes_per_second: Arc<RwLock<u64>>,
    /// Current token count
    tokens: Arc<RwLock<u64>>,
    /// Last refill time
    last_refill: Arc<RwLock<std::time::Instant>>,
}

impl BandwidthLimiter {
    /// Create a new bandwidth limiter
    /// 
    /// Default is unlimited (0).
    pub fn new() -> Self {
        Self {
            max_bytes_per_second: Arc::new(RwLock::new(0)),
            tokens: Arc::new(RwLock::new(0)),
            last_refill: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }

    /// Set the maximum bandwidth in bytes per second
    /// 
    /// 0 = unlimited
    pub async fn set_limit(&self, bytes_per_second: u64) {
        *self.max_bytes_per_second.write().await = bytes_per_second;
    }

    /// Get the current limit
    pub async fn get_limit(&self) -> u64 {
        *self.max_bytes_per_second.read().await
    }

    /// Acquire tokens for writing
    /// 
    /// Blocks until enough tokens are available.
    /// Returns the number of bytes that can be written.
    pub async fn acquire(&self, bytes: u64) -> u64 {
        // If unlimited, return immediately
        {
            let limit = *self.max_bytes_per_second.read().await;
            if limit == 0 {
                return bytes;
            }
        }

        // Refill tokens based on elapsed time
        self.refill_tokens().await;

        // Try to acquire tokens
        {
            let mut tokens = self.tokens.write().await;
            let limit = *self.max_bytes_per_second.read().await;

            if *tokens >= bytes {
                // Enough tokens available
                *tokens -= bytes;
                return bytes;
            }
        }

        // Not enough tokens - wait and retry
        let limit = *self.max_bytes_per_second.read().await;
        if limit > 0 {
            // Calculate wait time
            let ms_per_byte = 1000.0 / limit as f64;
            let ms_needed = (bytes as f64 * ms_per_byte).min(1000.0) as u64;
            
            if ms_needed > 0 {
                sleep(Duration::from_millis(ms_needed)).await;
                self.refill_tokens().await;
            }
        }

        // Try again (non-recursive)
        let mut tokens = self.tokens.write().await;
        if *tokens >= bytes {
            *tokens -= bytes;
            return bytes;
        }
        
        // Return what we can
        let available = *tokens;
        *tokens = 0;
        available
    }

    /// Refill tokens based on elapsed time
    async fn refill_tokens(&self) {
        let now = std::time::Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);
        *last_refill = now;
        drop(last_refill);

        let limit = *self.max_bytes_per_second.read().await;
        if limit == 0 {
            return;
        }

        // Add tokens based on elapsed time
        let new_tokens = (elapsed.as_millis() as u64 * limit / 1000).min(limit);
        
        let mut tokens = self.tokens.write().await;
        *tokens = (*tokens + new_tokens).min(limit);
    }
}

impl Default for BandwidthLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unlimited() {
        let limiter = BandwidthLimiter::new();
        limiter.set_limit(0).await;
        
        // Should return immediately
        let result = limiter.acquire(1000).await;
        assert_eq!(result, 1000);
    }

    #[tokio::test]
    async fn test_limited() {
        let limiter = BandwidthLimiter::new();
        limiter.set_limit(100).await; // 100 bytes/sec
        
        // Should be able to acquire some tokens
        let result = limiter.acquire(50).await;
        assert!(result > 0);
    }

    #[tokio::test]
    async fn test_dynamic_limit_change() {
        let limiter = BandwidthLimiter::new();
        limiter.set_limit(1000).await;
        
        // Change limit
        limiter.set_limit(500).await;
        
        // Should still work
        let result = limiter.acquire(100).await;
        assert!(result > 0);
    }
}