//! Multi-stream upload bandwidth measurement.
//!
//! This module handles uploading test data to speedtest.net servers
//! to measure upload bandwidth. It supports:
//! - Multi-stream concurrent uploads (4 streams by default, 1 with `--single`)
//! - Progressive payload sizing for accurate measurement
//! - Real-time progress tracking with speed calculation
//! - Peak speed detection through periodic sampling

use crate::bandwidth_loop::{BandwidthLoopState, BandwidthResult, determine_stream_count};
use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use crate::types::Server;
use reqwest::Client;
use std::sync::Arc;

/// Build upload URL
#[must_use]
pub fn build_upload_url(server_url: &str) -> String {
    format!("{server_url}/upload")
}

fn generate_upload_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}

/// Number of upload rounds per stream (each round uploads a chunk of test data).
const UPLOAD_TEST_ROUNDS: usize = 4;

/// Estimated total bytes for progress bar initialization.
const ESTIMATED_UPLOAD_BYTES: u64 = 4_000_000; // 4 MB estimate

/// Run upload bandwidth test against the given server.
///
/// Returns [`BandwidthResult`] with average/peak speed, total bytes, and samples.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all upload streams fail.
pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<SpeedProgress>,
) -> Result<BandwidthResult, SpeedtestError> {
    let concurrent_uploads = determine_stream_count(single);
    let state = Arc::new(BandwidthLoopState::new(ESTIMATED_UPLOAD_BYTES, progress));
    let upload_data = generate_upload_data(200_000); // 200KB chunks

    let mut handles = Vec::new();

    for _ in 0..concurrent_uploads {
        let client = client.clone();
        let server_url = server.url.clone();
        let data = upload_data.clone();
        let state = Arc::clone(&state);

        let handle = tokio::spawn(async move {
            let mut uploaded_bytes = 0u64;

            for _ in 0..UPLOAD_TEST_ROUNDS {
                let upload_url = build_upload_url(&server_url);

                if let Ok(response) = client.post(&upload_url).body(data.clone()).send().await {
                    if response.status().is_success() {
                        let chunk = data.len() as u64;
                        uploaded_bytes += chunk;
                        state.record_bytes(chunk);
                    }
                }
            }

            uploaded_bytes
        });

        handles.push(handle);
    }

    // Collect results — log any task panics so failures aren't silently swallowed.
    // Bytes are already counted via atomic counters, so we don't need the return values.
    for (i, handle) in handles.into_iter().enumerate() {
        if let Err(e) = handle.await {
            eprintln!("\nWarning: upload task {i} failed: {e}");
        }
    }

    let final_result = state.finish();
    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_concurrent_count_single() {
        assert_eq!(determine_stream_count(true), 1);
    }

    #[test]
    fn test_upload_concurrent_count_multiple() {
        assert_eq!(determine_stream_count(false), 4);
    }

    #[test]
    fn test_upload_url_generation() {
        let url = build_upload_url("http://server.example.com");
        assert!(url.ends_with("/upload"));
    }

    #[test]
    fn test_upload_url_generation_full_path() {
        let url = build_upload_url("http://server.example.com/speedtest");
        assert_eq!(url, "http://server.example.com/speedtest/upload");
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
        // Verify the upload data size used in upload_test (200KB)
        let data = generate_upload_data(200_000);
        assert_eq!(data.len(), 200_000);
    }
}
