//! Test runner orchestration for download and upload bandwidth tests.
//!
//! This module provides a reusable template for running bandwidth tests,
//! eliminating the duplication between download and upload test orchestration
//! in `main.rs`. Both tests follow the same pattern:
//! 1. Set up progress tracking
//! 2. Spawn latency-under-load monitoring in background
//! 3. Run the actual bandwidth test
//! 4. Stop latency monitoring
//! 5. Aggregate results

use crate::error::Error;
use crate::progress::Tracker;
use crate::types::Server;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

/// Trait for bandwidth test results - enables dependency injection and mocking.
pub trait TestMetrics: Send + Sync {
    fn avg_bps(&self) -> f64;
    fn peak_bps(&self) -> f64;
    fn total_bytes(&self) -> u64;
    fn duration_secs(&self) -> f64;
    fn speed_samples(&self) -> &[f64];
    fn latency_under_load(&self) -> Option<f64>;
}

/// Result from a bandwidth test (download or upload).
#[derive(PartialEq, Debug, Clone)]
pub struct TestRunResult {
    /// Average speed in bits per second
    pub avg_bps: f64,
    /// Peak speed in bits per second
    pub peak_bps: f64,
    /// Total bytes transferred
    pub total_bytes: u64,
    /// Duration of the test in seconds
    pub duration_secs: f64,
    /// Speed samples over time
    pub speed_samples: Vec<f64>,
    /// Average latency under load (ms), if measured
    pub latency_under_load: Option<f64>,
}

impl Default for TestRunResult {
    fn default() -> Self {
        Self {
            avg_bps: 0.0,
            peak_bps: 0.0,
            total_bytes: 0,
            duration_secs: 0.0,
            speed_samples: Vec::new(),
            latency_under_load: None,
        }
    }
}

impl TestMetrics for TestRunResult {
    fn avg_bps(&self) -> f64 {
        self.avg_bps
    }

    fn peak_bps(&self) -> f64 {
        self.peak_bps
    }

    fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    fn speed_samples(&self) -> &[f64] {
        &self.speed_samples
    }

    fn latency_under_load(&self) -> Option<f64> {
        self.latency_under_load
    }
}

/// Run a bandwidth test with latency-under-load monitoring.
///
/// This is a template method that handles:
/// - Progress bar setup
/// - Background latency monitoring
/// - Test execution via the provided closure
/// - Result aggregation
///
/// # Arguments
///
/// * `client` - HTTP client to reuse (avoids creating a second connection pool)
/// * `server` - Server to test against
/// * `test_label` - Label for progress display (e.g., "Download", "Upload")
/// * `is_verbose` - Whether to show visible progress
/// * `test_fn` - Async closure that runs the actual bandwidth test
///
/// # Errors
///
/// Returns [`Error`] if the test fails.
pub async fn run_bandwidth_test<F, Fut>(
    client: reqwest::Client,
    server: &Server,
    test_label: &str,
    is_verbose: bool,
    test_fn: F,
) -> Result<TestRunResult, Error>
where
    F: FnOnce(Arc<Tracker>) -> Fut,
    Fut: std::future::Future<Output = Result<(f64, f64, u64, Vec<f64>), Error>>,
{
    let progress = Arc::new(if is_verbose {
        Tracker::new(test_label)
    } else {
        Tracker::with_target(test_label, indicatif::ProgressDrawTarget::hidden())
    });

    // Set up latency-under-load monitoring
    let latency_samples = Arc::new(Mutex::new(Vec::new()));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let ping_url = server.url.clone();
    let samples_clone = Arc::clone(&latency_samples);
    let stop_clone = Arc::clone(&stop_signal);
    let ping_handle = tokio::spawn(async move {
        crate::servers::measure_latency_under_load(
            client.clone(),
            ping_url,
            samples_clone,
            stop_clone,
        )
        .await;
    });

    // Run the actual test
    let test_start = std::time::Instant::now();
    let (avg, peak, total_bytes, speed_samples) = test_fn(progress).await?;
    let duration = test_start.elapsed().as_secs_f64();

    // Stop latency monitoring
    stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = ping_handle.await;

    // Calculate average latency under load
    let latency_under_load = {
        let lock = latency_samples
            .lock()
            .map_err(|e| Error::context(format!("latency samples lock poisoned: {e}")))?;
        if lock.is_empty() {
            None
        } else {
            // Safe: latency sample count is small (≤100), well under 2^53.
            Some(lock.iter().sum::<f64>() / lock.len() as f64)
        }
    };

    Ok(TestRunResult {
        avg_bps: avg,
        peak_bps: peak,
        total_bytes,
        duration_secs: duration,
        speed_samples,
        latency_under_load,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_run_result_structure() {
        let result = TestRunResult {
            avg_bps: 100_000_000.0,
            peak_bps: 120_000_000.0,
            total_bytes: 10_000_000,
            duration_secs: 1.0,
            speed_samples: vec![100_000_000.0],
            latency_under_load: Some(15.0),
        };
        assert!((result.avg_bps - 100_000_000.0).abs() < f64::EPSILON);
        assert!((result.peak_bps - 120_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_test_run_result_default_values() {
        let result = TestRunResult::default();
        assert!(result.avg_bps.abs() < f64::EPSILON);
        assert!(result.peak_bps.abs() < f64::EPSILON);
        assert_eq!(result.total_bytes, 0);
        assert!(result.duration_secs.abs() < f64::EPSILON);
        assert!(result.speed_samples.is_empty());
        assert!(result.latency_under_load.is_none());
    }

    #[test]
    fn test_test_run_result_default_explicit() {
        let result = TestRunResult {
            avg_bps: 0.0,
            peak_bps: 0.0,
            total_bytes: 0,
            duration_secs: 0.0,
            speed_samples: Vec::new(),
            latency_under_load: None,
        };
        assert_eq!(result, TestRunResult::default());
    }

    #[test]
    fn test_test_run_result_with_samples() {
        let samples = vec![50_000_000.0, 75_000_000.0, 100_000_000.0];
        let result = TestRunResult {
            avg_bps: 75_000_000.0,
            peak_bps: 100_000_000.0,
            total_bytes: 5_000_000,
            duration_secs: 0.5,
            speed_samples: samples.clone(),
            latency_under_load: Some(12.0),
        };
        assert_eq!(result.speed_samples, samples);
        assert_eq!(result.speed_samples.len(), 3);
    }

    #[test]
    fn test_test_run_result_peak_greater_than_average() {
        let result = TestRunResult {
            avg_bps: 100_000_000.0,
            peak_bps: 150_000_000.0,
            total_bytes: 8_000_000,
            duration_secs: 0.8,
            speed_samples: vec![100_000_000.0],
            latency_under_load: None,
        };
        assert!(result.peak_bps > result.avg_bps);
    }

    #[test]
    fn test_test_metrics_impl_returns_correct_values() {
        let result = TestRunResult {
            avg_bps: 75_000_000.0,
            peak_bps: 100_000_000.0,
            total_bytes: 5_000_000,
            duration_secs: 0.5,
            speed_samples: vec![50_000_000.0, 75_000_000.0, 100_000_000.0],
            latency_under_load: Some(12.0),
        };
        assert!((result.avg_bps() - 75_000_000.0).abs() < f64::EPSILON);
        assert!((result.peak_bps() - 100_000_000.0).abs() < f64::EPSILON);
        assert_eq!(result.total_bytes(), 5_000_000);
        assert!((result.duration_secs() - 0.5).abs() < f64::EPSILON);
        assert_eq!(result.speed_samples().len(), 3);
        assert_eq!(result.latency_under_load(), Some(12.0));
    }

    #[test]
    fn test_test_metrics_impl_default() {
        let result = TestRunResult::default();
        assert!(result.avg_bps().abs() < f64::EPSILON);
        assert!(result.peak_bps().abs() < f64::EPSILON);
        assert_eq!(result.total_bytes(), 0);
        assert!(result.duration_secs().abs() < f64::EPSILON);
        assert!(result.speed_samples().is_empty());
        assert!(result.latency_under_load().is_none());
    }
}
