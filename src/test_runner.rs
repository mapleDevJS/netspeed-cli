//! Generic test runner orchestration.
//!
//! Provides a reusable template for running bandwidth tests with optional
//! background monitoring. Eliminates duplication between download and upload
//! test orchestration.
//!
//! The template handles:
//! 1. Progress bar setup (visible or hidden)
//! 2. Optional background monitoring (e.g., latency under load)
//! 3. Test execution via the provided closure
//! 4. Monitor teardown
//! 5. Result aggregation
//!
//! To use for a custom test, provide:
//! - A `test_fn` closure that performs the bandwidth measurement
//! - An optional `monitor_fn` closure for background monitoring

use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use reqwest::Client;
use std::sync::Arc;

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

/// Background monitor handle — returned when a monitor is started.
/// Call `stop()` to signal the monitor to shut down.
pub trait BackgroundMonitor: Send + Sync {
    fn stop(&self);
    fn average(&self) -> Option<f64>;
}

/// Run a bandwidth test with optional background monitoring.
///
/// This is a template method that handles:
/// - Progress bar setup (visible or hidden based on `is_verbose`)
/// - Optional background monitoring (latency under load, etc.)
/// - Test execution via the provided closure
/// - Monitor teardown and result aggregation
///
/// # Arguments
///
/// * `client` — HTTP client for background monitoring (may differ from test client)
/// * `server_url` — Base URL of the server under test (for monitoring endpoint)
/// * `test_label` — Label for progress display (e.g., "Download", "Upload")
/// * `is_verbose` — Whether to show visible progress
/// * `test_fn` — Async closure that runs the actual bandwidth test
/// * `monitor_factory` — Optional closure that creates a background monitor
///
/// # Errors
///
/// Returns [`SpeedtestError`] if the test fails.
pub async fn run_bandwidth_test<F, Fut, M>(
    client: &Client,
    server_url: &str,
    test_label: &str,
    is_verbose: bool,
    test_fn: F,
    monitor_factory: Option<M>,
) -> Result<TestRunResult, SpeedtestError>
where
    F: FnOnce(Arc<SpeedProgress>) -> Fut,
    Fut: std::future::Future<Output = Result<(f64, f64, u64, Vec<f64>), SpeedtestError>>,
    M: FnOnce(&Client, &str) -> Box<dyn BackgroundMonitor>,
{
    let progress = Arc::new(if is_verbose {
        SpeedProgress::new(test_label)
    } else {
        SpeedProgress::with_target(test_label, indicatif::ProgressDrawTarget::hidden())
    });

    // Set up optional background monitoring
    let monitor = monitor_factory.map(|f| f(client, server_url));

    // Run the actual test
    let test_start = std::time::Instant::now();
    let (avg, peak, total_bytes, speed_samples) = test_fn(progress).await?;
    let duration = test_start.elapsed().as_secs_f64();

    // Stop background monitoring
    let latency_under_load = monitor.as_ref().and_then(|m| m.average());
    if let Some(m) = &monitor {
        m.stop();
    }

    Ok(TestRunResult {
        avg_bps: avg,
        peak_bps: peak,
        total_bytes,
        duration_secs: duration,
        speed_samples,
        latency_under_load,
    })
}
