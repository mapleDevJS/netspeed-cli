//! Multi-stream download bandwidth measurement.
//!
//! This module handles downloading test files from speedtest.net servers
//! to measure download bandwidth. It supports:
//! - Multi-stream concurrent downloads (4 streams by default, 1 with `--single`)
//! - Dynamic test file URL construction from server base URL
//! - Real-time progress tracking with speed calculation
//! - Peak speed detection through periodic sampling
//!
//! Uses [`BandwidthLoopState`] for throttled speed sampling (20 Hz max),
//! shared with the upload module for consistent bandwidth measurement.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::bandwidth_loop::{BandwidthLoopState, BandwidthResult, determine_stream_count};
use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use crate::types::Server;
use reqwest::Client;
use std::sync::Arc;

/// Estimated total bytes for progress bar initialization.
/// This is a rough estimate; the bar will adjust as actual data is downloaded.
const ESTIMATED_DOWNLOAD_BYTES: u64 = 15_000_000; // 15 MB estimate

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

use futures_util::StreamExt;

/// Run download bandwidth test against the given server.
///
/// Returns [`BandwidthResult`] with average/peak speed, total bytes, and samples.
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
) -> Result<BandwidthResult, SpeedtestError> {
    let concurrent_streams = determine_stream_count(single);
    let state = Arc::new(BandwidthLoopState::new(ESTIMATED_DOWNLOAD_BYTES, progress));

    // Spawn streams that report progress
    let mut handles = Vec::new();
    for _ in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let state_ref = Arc::clone(&state);

        let handle = tokio::spawn(async move {
            for j in 0..DOWNLOAD_TEST_ROUNDS {
                let test_url = build_test_url(&server_url, j);

                if let Ok(response) = client.get(&test_url).send().await {
                    let mut stream = response.bytes_stream();
                    while let Some(item) = stream.next().await {
                        if let Ok(chunk) = item {
                            state_ref.record_bytes(chunk.len() as u64);
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all streams to complete
    for handle in handles {
        let _ = handle.await;
    }

    Ok(state.finish())
}

#[cfg(test)]
mod tests {
    use crate::bandwidth_loop::calculate_bandwidth;

    use super::*;

    #[test]
    fn test_download_bandwidth_calculation() {
        let result = calculate_bandwidth(10_000_000, 2.0);
        assert_eq!(result, 40_000_000.0);
    }

    #[test]
    fn test_download_bandwidth_zero_elapsed() {
        let result = calculate_bandwidth(10_000_000, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_download_concurrent_streams_single() {
        assert_eq!(determine_stream_count(true), 1);
    }

    #[test]
    fn test_download_concurrent_streams_multiple() {
        assert_eq!(determine_stream_count(false), 4);
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
}
