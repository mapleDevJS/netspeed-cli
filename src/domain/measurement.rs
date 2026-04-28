//! Bandwidth measurement for download and upload tests.
//!
//! This module provides the core measurement logic for running bandwidth tests,
//! eliminating duplication between download and upload test orchestration.

use crate::error::Error;
use crate::progress::Tracker;
use crate::servers::measure_latency_under_load;
use crate::task_runner::TestRunResult;
use crate::types::Server;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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
/// * `client` - HTTP client to reuse
/// * `server` - Server to test against
/// * `test_label` - Label for progress display
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

    let latency_samples = Arc::new(Mutex::new(Vec::new()));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let ping_url = server.url.clone();
    let samples_clone = Arc::clone(&latency_samples);
    let stop_clone = Arc::clone(&stop_signal);
    let ping_handle = tokio::spawn(async move {
        measure_latency_under_load(client.clone(), ping_url, samples_clone, stop_clone).await;
    });

    let test_start = std::time::Instant::now();
    let (avg, peak, total_bytes, speed_samples) = test_fn(progress).await?;
    let duration = test_start.elapsed().as_secs_f64();

    stop_signal.store(true, Ordering::Release);
    let _ = ping_handle.await;

    let latency_under_load = {
        let lock = latency_samples
            .lock()
            .map_err(|e| Error::context(format!("latency samples lock poisoned: {e}")))?;
        if lock.is_empty() {
            None
        } else {
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
}
