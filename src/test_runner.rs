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

use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Background monitor for latency under load.
/// Spawns a tokio task that pings the server's latency.txt endpoint
/// every 100ms until `stop()` is called.
pub struct LatencyUnderLoadMonitor {
    stop_signal: Arc<AtomicBool>,
    samples: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl LatencyUnderLoadMonitor {
    /// Start monitoring latency under load for the given server URL.
    pub fn start(client: &Client, server_url: &str) -> Self {
        let samples: Arc<std::sync::Mutex<Vec<f64>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let client = client.clone();
        let url = format!("{server_url}/latency.txt");
        let samples_clone = Arc::clone(&samples);
        let stop_clone = Arc::clone(&stop_signal);

        tokio::spawn(async move {
            while !stop_clone.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let response = client.get(&url).send().await;

                if let Ok(resp) = response {
                    if resp.status().is_success() {
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        if let Ok(mut lock) = samples_clone.lock() {
                            lock.push(elapsed);
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        Self {
            stop_signal,
            samples,
        }
    }
}

impl BackgroundMonitor for LatencyUnderLoadMonitor {
    fn stop(&self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }

    fn average(&self) -> Option<f64> {
        let lock = self.samples.lock().ok()?;
        if lock.is_empty() {
            None
        } else {
            Some(lock.iter().sum::<f64>() / lock.len() as f64)
        }
    }
}

/// Factory function to create a LatencyUnderLoadMonitor.
/// Used as the monitor_factory parameter in `run_bandwidth_test`.
pub fn create_latency_monitor(client: &Client, server_url: &str) -> Box<dyn BackgroundMonitor> {
    Box::new(LatencyUnderLoadMonitor::start(client, server_url))
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
        assert_eq!(result.avg_bps, 100_000_000.0);
        assert_eq!(result.peak_bps, 120_000_000.0);
    }

    #[test]
    fn test_test_run_result_default_values() {
        let result = TestRunResult::default();
        assert_eq!(result.avg_bps, 0.0);
        assert_eq!(result.peak_bps, 0.0);
        assert_eq!(result.total_bytes, 0);
        assert_eq!(result.duration_secs, 0.0);
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
