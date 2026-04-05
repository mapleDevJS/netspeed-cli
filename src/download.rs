//! Multi-stream download bandwidth measurement.
//!
//! This module handles downloading test files from speedtest.net servers
//! to measure download bandwidth. It supports:
//! - Multi-stream concurrent downloads (4 streams by default, 1 with `--single`)
//! - Dynamic test URL construction from server base URL
//! - Real-time progress tracking with speed calculation
//! - Peak speed detection through periodic sampling

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::common;
use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use crate::types::Server;
use reqwest::Client;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Estimated total bytes for progress bar initialization.
/// This is a rough estimate; the bar will adjust as actual data is downloaded.
const ESTIMATED_DOWNLOAD_BYTES: u64 = 15_000_000; // 15 MB estimate

/// Interval between speed samples in seconds.
/// Throttling prevents excessive sampling overhead.
const SAMPLE_INTERVAL_SECS: f64 = 0.05; // 50ms between samples (20 Hz max)

/// Extract base URL from server URL (strip /upload.php suffix)
#[must_use]
pub fn extract_base_url(url: &str) -> &str {
    url.strip_suffix("/upload.php").unwrap_or(url)
}

/// Build test file URL using Speedtest.net standard naming
#[must_use]
pub fn build_test_url(server_url: &str, file_index: usize) -> String {
    let base = extract_base_url(server_url);
    let sizes = ["2000x2000", "3000x3000", "3500x3500", "4000x4000"];
    let size = sizes[file_index % sizes.len()];
    format!("{base}/random{size}.jpg")
}

/// Result from a single download stream
struct StreamResult {
    bytes: u64,
    elapsed_secs: f64,
}

use futures_util::StreamExt;

/// Run download bandwidth test against the given server.
///
/// Returns `(avg_speed_bps, peak_speed_bps, total_bytes_downloaded, speed_samples)`.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all download streams fail.
/// Returns [`SpeedtestError::Context`] if the server URL is invalid.
pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<SpeedProgress>,
) -> Result<(f64, f64, u64, Vec<f64>), SpeedtestError> {
    let concurrent_streams = common::determine_stream_count(single);
    let total_bytes = Arc::new(AtomicU64::new(0));
    let peak_bps = Arc::new(AtomicU64::new(0));
    let speed_samples = Arc::new(Mutex::new(Vec::new()));
    let start = Instant::now();

    // Estimated total: progress bar will update dynamically as data is downloaded
    let estimated_total: u64 = ESTIMATED_DOWNLOAD_BYTES;

    // Spawn streams that report progress
    let mut handles = Vec::new();
    for _ in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let total_ref = Arc::clone(&total_bytes);
        let peak_ref = Arc::clone(&peak_bps);
        let samples_ref = Arc::clone(&speed_samples);
        let start_ref = start;
        let prog = Arc::clone(&progress);

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            for j in 0..4 {
                let test_url = build_test_url(&server_url, j);

                if let Ok(response) = client.get(&test_url).send().await {
                    let mut stream = response.bytes_stream();
                    while let Some(item) = stream.next().await {
                        if let Ok(chunk) = item {
                            let len = chunk.len() as u64;
                            stream_bytes += len;
                            total_ref.fetch_add(len, Ordering::Relaxed);

                            let total_so_far = total_ref.load(Ordering::Relaxed);
                            let elapsed = start_ref.elapsed().as_secs_f64();
                            let speed = common::calculate_bandwidth(total_so_far, elapsed);

                            let current_peak = peak_ref.load(Ordering::Relaxed);
                            if speed > current_peak as f64 {
                                peak_ref.store(speed as u64, Ordering::Relaxed);
                            }

                            // Record speed sample (throttled to avoid excessive overhead)
                            if let Ok(mut samples) = samples_ref.lock() {
                                if samples.is_empty()
                                    || elapsed - samples.last().copied().unwrap_or(0.0) > SAMPLE_INTERVAL_SECS
                                {
                                    samples.push(speed);
                                }
                            }

                            let pct = (total_so_far as f64 / estimated_total as f64).min(1.0);
                            prog.update(speed / 1_000_000.0, pct, total_so_far);
                        }
                    }
                }
            }

            StreamResult {
                bytes: stream_bytes,
                elapsed_secs: start_ref.elapsed().as_secs_f64(),
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    if results.is_empty() {
        return Ok((0.0, 0.0, 0, Vec::new()));
    }

    let total_bandwidth: f64 = results
        .iter()
        .map(|r| common::calculate_bandwidth(r.bytes, r.elapsed_secs))
        .sum();

    let final_total_bytes = total_bytes.load(Ordering::Relaxed);
    let final_peak_speed = peak_bps.load(Ordering::Relaxed) as f64;
    let avg_bandwidth = total_bandwidth / results.len() as f64;
    let samples = speed_samples.lock().unwrap().to_vec();
    Ok((avg_bandwidth, final_peak_speed, final_total_bytes, samples))
}

#[cfg(test)]
mod tests {
    use crate::common;

    use super::*;

    #[test]
    fn test_download_bandwidth_calculation() {
        let result = common::calculate_bandwidth(10_000_000, 2.0);
        assert_eq!(result, 40_000_000.0);
    }

    #[test]
    fn test_download_bandwidth_zero_elapsed() {
        let result = common::calculate_bandwidth(10_000_000, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_download_concurrent_streams_single() {
        assert_eq!(common::determine_stream_count(true), 1);
    }

    #[test]
    fn test_download_concurrent_streams_multiple() {
        assert_eq!(common::determine_stream_count(false), 4);
    }

    #[test]
    fn test_download_url_generation() {
        let server_url = "http://server.example.com/speedtest/upload.php";
        let test_url = build_test_url(server_url, 0);
        assert_eq!(
            test_url,
            "http://server.example.com/speedtest/random2000x2000.jpg"
        );
    }

    #[test]
    fn test_download_url_generation_cycles() {
        let server_url = "http://server.example.com/speedtest/upload.php";
        let url_0 = build_test_url(server_url, 0);
        let url_4 = build_test_url(server_url, 4);
        assert_eq!(url_0, url_4);
    }

    #[test]
    fn test_extract_base_url() {
        let url = "http://server.example.com:8080/speedtest/upload.php";
        assert_eq!(
            extract_base_url(url),
            "http://server.example.com:8080/speedtest"
        );
    }
}
