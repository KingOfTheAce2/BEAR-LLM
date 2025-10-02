use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Configuration for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per time window
    pub max_requests: usize,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Whether to enable automatic cleanup of old entries
    pub auto_cleanup: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            auto_cleanup: true,
        }
    }
}

/// Record of requests for a specific user/identifier
#[derive(Debug, Clone)]
struct RequestRecord {
    /// Timestamps of requests within the current window
    timestamps: Vec<Instant>,
    /// Last cleanup time
    last_cleanup: Instant,
}

impl RequestRecord {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Remove expired timestamps outside the window
    fn cleanup(&mut self, window: Duration) {
        let now = Instant::now();
        self.timestamps.retain(|&ts| now.duration_since(ts) < window);
        self.last_cleanup = now;
    }

    /// Check if this record should be cleaned up (no requests in window)
    fn is_expired(&self, window: Duration) -> bool {
        self.timestamps.is_empty() &&
        Instant::now().duration_since(self.last_cleanup) > window
    }
}

/// Rate limiter for tracking and limiting requests
pub struct RateLimiter {
    /// Configuration
    config: RwLock<RateLimitConfig>,
    /// Map of identifier to request records
    records: Arc<RwLock<HashMap<String, RequestRecord>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config: RwLock::new(config),
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request should be allowed
    ///
    /// # Arguments
    /// * `identifier` - Unique identifier (e.g., user ID, IP address, session ID)
    ///
    /// # Returns
    /// * `Ok(())` if request is allowed
    /// * `Err(String)` with details if rate limit exceeded
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), String> {
        let config = self.config.read().await;
        let window = Duration::from_secs(config.window_seconds);
        let max_requests = config.max_requests;
        let auto_cleanup = config.auto_cleanup;
        drop(config);

        let mut records = self.records.write().await;

        // Get or create record for this identifier
        let record = records.entry(identifier.to_string())
            .or_insert_with(RequestRecord::new);

        // Clean up old timestamps
        record.cleanup(window);

        // Check if limit exceeded
        if record.timestamps.len() >= max_requests {
            let oldest = record.timestamps.first()
                .ok_or_else(|| "No timestamps found".to_string())?;
            let time_until_reset = window.saturating_sub(Instant::now().duration_since(*oldest));

            tracing::warn!(
                identifier = identifier,
                current_count = record.timestamps.len(),
                max_requests = max_requests,
                window_seconds = window.as_secs(),
                reset_seconds = time_until_reset.as_secs(),
                "Rate limit exceeded"
            );

            return Err(format!(
                "Rate limit exceeded. Maximum {} requests per {} seconds. Try again in {} seconds.",
                max_requests,
                window.as_secs(),
                time_until_reset.as_secs()
            ));
        }

        // Record this request
        record.timestamps.push(Instant::now());

        // Perform global cleanup if enabled
        if auto_cleanup && records.len() > 1000 {
            records.retain(|_, rec| !rec.is_expired(window * 2));
        }

        Ok(())
    }

    /// Record a successful request (alias for check_rate_limit for clarity)
    pub async fn record_request(&self, identifier: &str) -> Result<(), String> {
        self.check_rate_limit(identifier).await
    }

    /// Get current usage for an identifier
    pub async fn get_usage(&self, identifier: &str) -> Option<RateLimitUsage> {
        let config = self.config.read().await;
        let window = Duration::from_secs(config.window_seconds);
        let max_requests = config.max_requests;
        drop(config);

        let records = self.records.read().await;
        let record = records.get(identifier)?;

        let now = Instant::now();
        let current_count = record.timestamps.iter()
            .filter(|&&ts| now.duration_since(ts) < window)
            .count();

        Some(RateLimitUsage {
            identifier: identifier.to_string(),
            current_requests: current_count,
            max_requests,
            window_seconds: window.as_secs(),
            remaining: max_requests.saturating_sub(current_count),
        })
    }

    /// Update rate limit configuration
    pub async fn update_config(&self, new_config: RateLimitConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
        tracing::info!(
            max_requests = config.max_requests,
            window_seconds = config.window_seconds,
            "Rate limit configuration updated"
        );
    }

    /// Get current configuration
    pub async fn get_config(&self) -> RateLimitConfig {
        self.config.read().await.clone()
    }

    /// Clear all rate limit records
    pub async fn clear(&self) {
        let mut records = self.records.write().await;
        records.clear();
        tracing::info!("Rate limit records cleared");
    }

    /// Get total number of tracked identifiers
    pub async fn tracked_count(&self) -> usize {
        self.records.read().await.len()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Current usage information for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsage {
    pub identifier: String,
    pub current_requests: usize,
    pub max_requests: usize,
    pub window_seconds: u64,
    pub remaining: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            max_requests: 3,
            window_seconds: 1,
            auto_cleanup: true,
        };
        let limiter = RateLimiter::with_config(config);

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());

        // 4th request should fail
        assert!(limiter.check_rate_limit("user1").await.is_err());

        // Wait for window to expire
        sleep(Duration::from_secs(2)).await;

        // Should succeed after window reset
        assert!(limiter.check_rate_limit("user1").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_multiple_users() {
        let limiter = RateLimiter::new();

        // Different users should have independent limits
        for i in 0..100 {
            assert!(limiter.check_rate_limit(&format!("user{}", i)).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_get_usage() {
        let config = RateLimitConfig {
            max_requests: 10,
            window_seconds: 60,
            auto_cleanup: true,
        };
        let limiter = RateLimiter::with_config(config);

        limiter.check_rate_limit("user1").await.ok();
        limiter.check_rate_limit("user1").await.ok();
        limiter.check_rate_limit("user1").await.ok();

        let usage = limiter.get_usage("user1").await.unwrap();
        assert_eq!(usage.current_requests, 3);
        assert_eq!(usage.remaining, 7);
    }
}
