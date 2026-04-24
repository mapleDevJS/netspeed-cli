//! Centralized test configuration constants.
//!
//! This module consolidates all magic numbers used across download/upload tests,
//! making it easy to tune test behavior and avoid inconsistent values.
//!
//! # Usage
//!
//! ```rust
//! use netspeed_cli::test_config::TestConfig;
//!
//! let config = TestConfig::default();
//! println!("rounds: {}, streams: {}, interval: {}ms",
//!     config.download_rounds, config.stream_count, config.sample_interval_ms);
//! ```

/// Centralized test configuration for bandwidth measurement.
///
/// All timing, count, and sizing values used across the test pipeline
/// are concentrated here so they can be tuned in one place.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Number of download rounds per stream (each round fetches a different test file).
    pub download_rounds: usize,

    /// Number of upload rounds per stream (each round uploads a chunk of test data).
    pub upload_rounds: usize,

    /// Number of concurrent streams in multi-stream mode.
    pub stream_count: usize,

    /// Throttle interval for speed sampling in milliseconds (20 Hz max).
    pub sample_interval_ms: u64,

    /// Number of ping attempts to measure latency, jitter, and packet loss.
    pub ping_attempts: usize,

    /// Payload size for each upload chunk in bytes.
    pub upload_payload_bytes: usize,

    /// Estimated total download bytes for progress bar initialization.
    pub estimated_download_bytes: u64,

    /// Estimated total upload bytes for progress bar initialization.
    pub estimated_upload_bytes: u64,

    /// How often to poll latency under load (milliseconds).
    pub latency_poll_interval_ms: u64,

    /// Default number of HTTP retry attempts for transient failures.
    pub http_retry_attempts: usize,

    /// Initial HTTP retry backoff in milliseconds.
    pub http_retry_base_ms: u64,

    /// Maximum HTTP retry backoff in milliseconds.
    pub http_retry_max_ms: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            // Test rounds per stream
            download_rounds: 4,
            upload_rounds: 4,

            // Concurrency
            stream_count: 4,

            // Sampling (20 Hz = 50ms interval)
            sample_interval_ms: 50,

            // Ping measurement
            ping_attempts: 8,

            // Upload payload (200 KB — matches speedtest.net standard)
            upload_payload_bytes: 200_000,

            // Progress bar estimates
            estimated_download_bytes: 15_000_000, // 15 MB
            estimated_upload_bytes: 4_000_000,    // 4 MB

            // Latency under load polling
            latency_poll_interval_ms: 100,

            // HTTP retry configuration
            http_retry_attempts: 3,
            http_retry_base_ms: 100,
            http_retry_max_ms: 5000,
        }
    }
}

impl TestConfig {
    /// Get stream count based on single-connection mode.
    #[must_use]
    pub fn stream_count_for(single: bool) -> usize {
        if single {
            1
        } else {
            Self::default().stream_count
        }
    }

    /// Calculate retry delay with exponential backoff.
    /// Returns (delay_ms, should_retry).
    #[must_use]
    pub fn retry_delay(attempt: usize) -> (u64, bool) {
        let config = Self::default();
        if attempt >= config.http_retry_attempts {
            return (0, false);
        }
        // Exponential backoff: 100ms, 200ms, 400ms, ... capped at max
        let delay = config.http_retry_base_ms * 2u64.saturating_pow(attempt as u32);
        let delay = delay.min(config.http_retry_max_ms);
        (delay, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = TestConfig::default();
        assert_eq!(config.download_rounds, 4);
        assert_eq!(config.upload_rounds, 4);
        assert_eq!(config.stream_count, 4);
        assert_eq!(config.sample_interval_ms, 50);
        assert_eq!(config.ping_attempts, 8);
        assert_eq!(config.upload_payload_bytes, 200_000);
        assert_eq!(config.estimated_download_bytes, 15_000_000);
        assert_eq!(config.estimated_upload_bytes, 4_000_000);
        assert_eq!(config.http_retry_attempts, 3);
    }

    #[test]
    fn test_stream_count_for_single() {
        assert_eq!(TestConfig::stream_count_for(true), 1);
    }

    #[test]
    fn test_stream_count_for_multi() {
        assert_eq!(TestConfig::stream_count_for(false), 4);
    }

    #[test]
    fn test_retry_delay_first_attempt() {
        let (delay, should_retry) = TestConfig::retry_delay(0);
        assert!(should_retry);
        assert_eq!(delay, 100); // 100ms base
    }

    #[test]
    fn test_retry_delay_second_attempt() {
        let (delay, should_retry) = TestConfig::retry_delay(1);
        assert!(should_retry);
        assert_eq!(delay, 200); // 100ms * 2^1
    }

    #[test]
    fn test_retry_delay_third_attempt() {
        let (delay, should_retry) = TestConfig::retry_delay(2);
        assert!(should_retry);
        assert_eq!(delay, 400); // 100ms * 2^2
    }

    #[test]
    fn test_retry_delay_exhausted() {
        let (_, should_retry) = TestConfig::retry_delay(3);
        assert!(!should_retry);
    }

    #[test]
    fn test_retry_delay_beyond_max_attempts() {
        // Attempt 10 >= http_retry_attempts (3) returns should_retry=false with delay=0
        let (delay, should_retry) = TestConfig::retry_delay(10);
        assert!(!should_retry, "attempt 10 is beyond max retry attempts");
        assert_eq!(delay, 0, "delay is 0 when retries exhausted");
    }
}
