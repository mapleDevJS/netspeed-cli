//! Multi-stream download bandwidth measurement.
//!
//! This module handles downloading test files from speedtest.net servers
//! to measure download bandwidth. It supports:
//! - Multi-stream concurrent downloads (4 streams by default, 1 with `--single`)
//! - Dynamic test URL construction from server base URL
//! - Real-time progress tracking with speed calculation
//! - Peak speed detection through periodic sampling

use crate::bandwidth_loop::run_concurrent_streams;
use crate::endpoints::ServerEndpoints;
use crate::error::Error;
use crate::progress::Tracker;
use crate::test_config::TestConfig;
use crate::types::Server;
use reqwest::Client;
use std::sync::Arc;

/// Estimated total bytes for progress bar initialization.
/// This is a rough estimate; the bar will adjust as actual data is downloaded.
///
/// Deprecated: Use `TestConfig::default().estimated_download_bytes` instead.
#[deprecated(
    since = "0.9.0",
    note = "Use TestConfig::default().estimated_download_bytes"
)]
#[allow(dead_code)]
const ESTIMATED_DOWNLOAD_BYTES: u64 = 15_000_000; // 15 MB estimate

/// Number of download rounds per stream (each round fetches a different test file).
///
/// Deprecated: Use `TestConfig::default().download_rounds` instead.
#[deprecated(since = "0.9.0", note = "Use TestConfig::default().download_rounds")]
#[allow(dead_code)]
const DOWNLOAD_TEST_ROUNDS: usize = 4;

/// Extract base URL from server URL (strip /upload.php suffix)
#[must_use]
pub fn extract_base_url(url: &str) -> String {
    ServerEndpoints::from_server_url(url).base().to_string()
}

/// Build test file URL using Speedtest.net standard naming
#[must_use]
pub fn build_test_url(server_url: &str, file_index: usize) -> String {
    let sizes = ["2000x2000", "3000x3000", "3500x3500", "4000x4000"];
    let size = sizes[file_index % sizes.len()];
    ServerEndpoints::from_server_url(server_url).download_asset(&format!("random{size}.jpg"))
}

use futures_util::StreamExt;

/// Run download bandwidth test against the given server.
///
/// Returns `(avg_speed_bps, peak_speed_bps, total_bytes_downloaded, speed_samples)`.
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if all download streams fail.
/// Returns [`Error::Context`] if the server URL is invalid.
pub async fn run(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<Tracker>,
) -> Result<(f64, f64, u64, Vec<f64>), Error> {
    let config = TestConfig::default();
    let stream_count = TestConfig::stream_count_for(single);

    let result = run_concurrent_streams(
        config.estimated_download_bytes,
        stream_count,
        progress,
        "download",
        |_, state, sample_interval| {
            let client = client.clone();
            let server_url = Arc::new(server.url.clone());
            tokio::spawn(async move {
                for j in 0..config.download_rounds {
                    let test_url = build_test_url(&server_url, j);

                    let response = client
                        .get(&test_url)
                        .send()
                        .await
                        .map_err(Error::DownloadTest)?;

                    if !response.status().is_success() {
                        return Err(Error::DownloadFailure(format!(
                            "server returned {} for {test_url}",
                            response.status()
                        )));
                    }

                    let mut stream = response.bytes_stream();
                    while let Some(item) = stream.next().await {
                        let chunk = item.map_err(Error::DownloadTest)?;
                        let len = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
                        if len > 0 {
                            state.record_bytes(len, sample_interval);
                        }
                    }
                }
                Ok(())
            })
        },
    )
    .await?;

    Ok((
        result.avg_bps,
        result.peak_bps,
        result.total_bytes,
        result.speed_samples,
    ))
}

#[cfg(test)]
mod tests {
    use crate::common;
    use crate::test_config::TestConfig;

    use super::*;

    #[test]
    fn test_download_bandwidth_calculation() {
        let result = common::calculate_bandwidth(10_000_000, 2.0);
        assert!((result - 40_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_download_bandwidth_zero_elapsed() {
        let result = common::calculate_bandwidth(10_000_000, 0.0);
        assert!(result.abs() < f64::EPSILON);
    }

    #[test]
    fn test_download_concurrent_streams_single() {
        assert_eq!(TestConfig::stream_count_for(true), 1);
    }

    #[test]
    fn test_download_concurrent_streams_multiple() {
        assert_eq!(TestConfig::stream_count_for(false), 4);
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
        #[allow(deprecated)]
        {
            const _: () = assert!(ESTIMATED_DOWNLOAD_BYTES > 10_000_000);
            const _: () = assert!(ESTIMATED_DOWNLOAD_BYTES < 20_000_000);
        }
    }

    #[test]
    fn test_sample_interval_constant() {
        // Verify sample interval is 50ms (20 Hz) — now defined in LoopState
        const _: () = assert!(crate::bandwidth_loop::SAMPLE_INTERVAL_MS == 50);
    }
}
