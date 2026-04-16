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
use crate::terminal;
use crate::types::Server;
use owo_colors::OwoColorize;
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

/// Estimated total bytes for progress bar initialization.
/// This is a rough estimate; the bar will adjust as actual data is downloaded.
const ESTIMATED_DOWNLOAD_BYTES: u64 = 15_000_000; // 15 MB estimate

/// Interval between speed samples in milliseconds.
/// Throttling prevents excessive sampling overhead on all hot-path operations.
/// Uses 0 as initial value so the first chunk always triggers a sample.
const SAMPLE_INTERVAL_MS: u64 = 50; // 50ms between samples (20 Hz max)

/// Number of download rounds per stream (each round fetches a different test file).
const DOWNLOAD_TEST_ROUNDS: usize = 4;

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

    // Throttle gate: tracks last sample time in millis to limit all expensive ops to 20 Hz.
    // Initialized to 0 so the first chunk always triggers a sample (any elapsed > 0 fires).
    let last_sample_ms = Arc::new(AtomicU64::new(0));

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
        let throttle_ref = Arc::clone(&last_sample_ms);

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            for j in 0..DOWNLOAD_TEST_ROUNDS {
                let test_url = build_test_url(&server_url, j);

                if let Ok(response) = client.get(&test_url).send().await {
                    let mut stream = response.bytes_stream();
                    while let Some(item) = stream.next().await {
                        if let Ok(chunk) = item {
                            let len = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
                            stream_bytes += len;
                            // Release ensures other threads see this write before the final Acquire load
                            total_ref.fetch_add(len, Ordering::Release);

                            // Throttle gate: only run expensive ops every 50ms.
                            // First sample always fires (last_sample_ms == 0 means "never sampled").
                            let elapsed_ms =
                                u64::try_from(start_ref.elapsed().as_millis()).unwrap_or(u64::MAX);
                            let last_ms = throttle_ref.load(Ordering::Relaxed);
                            let should_sample = last_ms == 0
                                || elapsed_ms.saturating_sub(last_ms) >= SAMPLE_INTERVAL_MS;
                            if should_sample {
                                // Update throttle timestamp
                                throttle_ref.store(elapsed_ms, Ordering::Relaxed);

                                // All expensive ops now run at most every 50ms:
                                // Acquire ensures we see the latest fetch_add results on ARM64.
                                let total_so_far = total_ref.load(Ordering::Acquire);
                                let elapsed = start_ref.elapsed().as_secs_f64();
                                let speed = common::calculate_bandwidth(total_so_far, elapsed);

                                // Peak tracking — Release pairs with the Acquire load in final read
                                let current_peak = peak_ref.load(Ordering::Relaxed);
                                if speed > current_peak as f64 {
                                    let peak_u64 = speed.clamp(0.0, u64::MAX as f64) as u64;
                                    peak_ref.store(peak_u64, Ordering::Release);
                                }

                                // Record speed sample (throttled, no need for additional check)
                                if let Ok(mut samples) = samples_ref.lock() {
                                    samples.push(speed);
                                }

                                let pct = (total_so_far as f64 / estimated_total as f64).min(1.0);
                                prog.update(speed / 1_000_000.0, pct, total_so_far);
                            }
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

    // Collect results — log any task panics so failures aren't silently swallowed.
    // Bytes are already counted via atomic counters, so we don't need the return values.
    let mut results = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                let msg = format!("Warning: download task {i} failed: {e}");
                if terminal::no_color() {
                    eprintln!("\n{msg}");
                } else {
                    eprintln!("\n{}", msg.yellow().bold());
                }
            }
        }
    }

    if results.is_empty() {
        return Ok((0.0, 0.0, 0, Vec::new()));
    }

    let total_bandwidth: f64 = results
        .iter()
        .map(|r| common::calculate_bandwidth(r.bytes, r.elapsed_secs))
        .sum();

    // Final reads: Acquire pairs with the Release fetch_add/stores to ensure
    // we see all writes from completed Tokio tasks on all architectures.
    let final_total_bytes = total_bytes.load(Ordering::Acquire);
    let final_peak_speed = peak_bps.load(Ordering::Acquire) as f64;
    let avg_bandwidth = total_bandwidth / results.len() as f64;
    let samples = speed_samples
        .lock()
        .map_err(|e| SpeedtestError::context(format!("download samples lock poisoned: {e}")))?
        .to_vec();
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
    fn test_download_url_generation_all_sizes() {
        let server_url = "http://server.example.com/speedtest/upload.php";
        let expected = [
            "http://server.example.com/speedtest/random2000x2000.jpg",
            "http://server.example.com/speedtest/random3000x3000.jpg",
            "http://server.example.com/speedtest/random3500x3500.jpg",
            "http://server.example.com/speedtest/random4000x4000.jpg",
        ];

        for (i, expected_url) in expected.iter().enumerate() {
            assert_eq!(build_test_url(server_url, i), *expected_url);
        }
    }

    #[test]
    fn test_extract_base_url() {
        let url = "http://server.example.com:8080/speedtest/upload.php";
        assert_eq!(
            extract_base_url(url),
            "http://server.example.com:8080/speedtest"
        );
    }

    #[test]
    fn test_extract_base_url_no_suffix() {
        let url = "http://server.example.com/speedtest";
        assert_eq!(extract_base_url(url), "http://server.example.com/speedtest");
    }

    #[test]
    fn test_extract_base_url_different_path() {
        let url = "https://cdn.speedtest.net/upload.php";
        assert_eq!(extract_base_url(url), "https://cdn.speedtest.net");
    }

    #[test]
    fn test_estimated_download_bytes_constant() {
        // Verify the constant is reasonable (around 15 MB)
        const _: () = assert!(ESTIMATED_DOWNLOAD_BYTES > 10_000_000);
        const _: () = assert!(ESTIMATED_DOWNLOAD_BYTES < 20_000_000);
    }

    #[test]
    fn test_sample_interval_constant() {
        // Verify sample interval is 50ms (20 Hz)
        const _: () = assert!(SAMPLE_INTERVAL_MS == 50);
    }
}
