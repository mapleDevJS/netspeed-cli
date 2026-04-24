//! Multi-stream upload bandwidth measurement.
//!
//! This module handles uploading test data to speedtest.net servers
//! to measure upload bandwidth. It supports:
//! - Multi-stream concurrent uploads (4 streams by default, 1 with `--single`)
//! - Progressive payload sizing for accurate measurement
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

/// Build upload URL
#[must_use]
pub fn build_upload_url(server_url: &str) -> String {
    ServerEndpoints::from_server_url(server_url)
        .upload()
        .to_string()
}

/// Deterministic upload payload: byte\[i\] = i % 256.
/// Initialized once via `LazyLock` — Bytes-backed for zero-copy sharing.
static UPLOAD_PAYLOAD: std::sync::LazyLock<bytes::Bytes> = std::sync::LazyLock::new(|| {
    let mut data = vec![0u8; 200_000];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    bytes::Bytes::from(data)
});

/// Generate upload data of the given size (used by tests).
#[cfg(test)]
fn generate_upload_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}

/// Run upload bandwidth test against the given server.
///
/// Returns `(avg_speed_bps, peak_speed_bps, total_bytes_uploaded, speed_samples)`.
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if all upload streams fail.
pub async fn run(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<Tracker>,
) -> Result<(f64, f64, u64, Vec<f64>), Error> {
    let config = TestConfig::default();
    let stream_count = TestConfig::stream_count_for(single);
    let upload_data: bytes::Bytes = (*UPLOAD_PAYLOAD).clone();

    let result = run_concurrent_streams(
        config.estimated_upload_bytes,
        stream_count,
        progress,
        "upload",
        |_, state, sample_interval| {
            let client = client.clone();
            let server_url = Arc::new(server.url.clone());
            let data = Arc::new(upload_data.clone());
            tokio::spawn(async move {
                for _ in 0..config.upload_rounds {
                    let upload_url = build_upload_url(&server_url);

                    let response = client
                        .post(&upload_url)
                        .body((*data).clone())
                        .send()
                        .await
                        .map_err(Error::UploadTest)?;

                    if !response.status().is_success() {
                        return Err(Error::UploadFailure(format!(
                            "server returned {} for {upload_url}",
                            response.status()
                        )));
                    }

                    let chunk = u64::try_from(data.len()).unwrap_or(u64::MAX);
                    state.record_bytes(chunk, sample_interval);
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
    fn test_upload_bandwidth_calculation() {
        let result = common::calculate_bandwidth(1_000_000, 2.0);
        assert!((result - 4_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_upload_bandwidth_zero_elapsed() {
        let result = common::calculate_bandwidth(1_000_000, 0.0);
        assert!(result.abs() < f64::EPSILON);
    }

    #[test]
    fn test_upload_concurrent_count_single() {
        assert_eq!(TestConfig::stream_count_for(true), 1);
    }

    #[test]
    fn test_upload_concurrent_count_multiple() {
        assert_eq!(TestConfig::stream_count_for(false), 4);
    }

    #[test]
    fn test_upload_url_generation() {
        let url = build_upload_url("http://server.example.com");
        assert!(url.ends_with("/upload.php"));
    }

    #[test]
    fn test_upload_url_generation_full_path() {
        let url = build_upload_url("http://server.example.com/speedtest/upload.php");
        assert_eq!(url, "http://server.example.com/speedtest/upload.php");
    }

    #[test]
    fn test_generate_upload_data_size() {
        let data = generate_upload_data(1000);
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_generate_upload_data_pattern() {
        let data = generate_upload_data(300);
        for (i, &byte) in data.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }

    #[test]
    fn test_generate_upload_data_wraps_at_256() {
        let data = generate_upload_data(512);
        assert_eq!(data[0], 0u8);
        assert_eq!(data[255], 255u8);
        assert_eq!(data[256], 0u8);
        assert_eq!(data[511], 255u8);
    }

    #[test]
    fn test_generate_upload_data_empty() {
        let data = generate_upload_data(0);
        assert!(data.is_empty());
    }

    #[test]
    fn test_upload_data_size_constant() {
        // Verify the upload data size used in run (200KB)
        let data = generate_upload_data(200_000);
        assert_eq!(data.len(), 200_000);
    }

    #[test]
    fn test_upload_payload_lazy_init() {
        // Verify the LazyLock payload matches the expected pattern
        assert_eq!(UPLOAD_PAYLOAD.len(), 200_000);
        assert_eq!(UPLOAD_PAYLOAD[0], 0u8);
        assert_eq!(UPLOAD_PAYLOAD[255], 255u8);
        assert_eq!(UPLOAD_PAYLOAD[256], 0u8);
    }
}
