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
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

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
    let mut data = vec![0u8; 2_000_000];
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
    run_with_config(client, server, single, progress, &UploadConfig::default()).await
}

/// Upload traffic and time limits. Caps include all streams; warm-up traffic is
/// capped separately and excluded from measured bytes and speed samples.
#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub warmup: Duration,
    pub duration: Duration,
    pub warmup_bytes: u64,
    pub measurement_bytes: u64,
    pub initial_payload: usize,
    pub max_payload: usize,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            warmup: Duration::from_secs(1),
            duration: Duration::from_secs(5),
            warmup_bytes: 16 * 1024 * 1024,
            measurement_bytes: 256 * 1024 * 1024,
            initial_payload: TestConfig::default().upload_payload_bytes,
            max_payload: UPLOAD_PAYLOAD.len(),
        }
    }
}

/// Run an upload with explicit limits (also useful for bounded local tests).
pub async fn run_with_config(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<Tracker>,
    config: &UploadConfig,
) -> Result<(f64, f64, u64, Vec<f64>), Error> {
    let streams = TestConfig::stream_count_for(single);
    if config.duration.is_zero()
        || config.measurement_bytes < streams as u64
        || config.initial_payload == 0
        || config.initial_payload > config.max_payload
        || config.max_payload > UPLOAD_PAYLOAD.len()
    {
        return Err(Error::UploadFailure(
            "invalid upload measurement limits".into(),
        ));
    }
    let url = build_upload_url(&server.url);
    // Warm every connection concurrently before starting the measurement clock.
    let warmups = (0..streams).map(|_| async {
        upload_stream(
            client,
            &url,
            config.warmup,
            config.warmup_bytes / streams as u64,
            config.initial_payload,
            config.max_payload,
            None,
        )
        .await
    });
    let payloads = futures::future::try_join_all(warmups).await?;
    let result = run_concurrent_streams(
        config.measurement_bytes,
        streams,
        progress,
        "upload",
        |i, state, interval| {
            let client = client.clone();
            let url = url.clone();
            let config = config.clone();
            let initial_payload = payloads[i];
            tokio::spawn(async move {
                upload_stream(
                    &client,
                    &url,
                    config.duration,
                    config.measurement_bytes / streams as u64,
                    initial_payload,
                    config.max_payload,
                    Some((state, interval)),
                )
                .await?;
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

/// Adapt toward roughly 200 ms per request, keeping RTT overhead small without
/// allowing a single request or an unresponsive server to extend the phase.
async fn upload_stream(
    client: &Client,
    url: &str,
    duration: Duration,
    budget: u64,
    mut payload: usize,
    max_payload: usize,
    recording: Option<(Arc<crate::bandwidth_loop::LoopState>, u64)>,
) -> Result<usize, Error> {
    let deadline = tokio::time::Instant::now() + duration;
    let mut sent = 0;
    while sent < budget && tokio::time::Instant::now() < deadline {
        let len = payload.min(usize::try_from(budget - sent).unwrap_or(usize::MAX));
        let start = tokio::time::Instant::now();
        let request = async {
            let body = if let Some((state, interval)) = &recording {
                let state = state.clone();
                let interval = *interval;
                // Count chunks as HTTP consumes them, rather than crediting an
                // entire request at acknowledgement time (which creates false
                // zero-speed intervals and spikes on a steady connection).
                let chunks =
                    futures::stream::iter((0..len).step_by(16 * 1024)).map(move |offset| {
                        let chunk = UPLOAD_PAYLOAD.slice(offset..(offset + 16 * 1024).min(len));
                        state.record_bytes(chunk.len() as u64, interval);
                        Ok::<_, std::io::Error>(chunk)
                    });
                reqwest::Body::wrap_stream(chunks)
            } else {
                reqwest::Body::from(UPLOAD_PAYLOAD.slice(..len))
            };
            let mut response = client
                .post(url)
                .header(reqwest::header::CONTENT_LENGTH, len)
                .body(body)
                .send()
                .await
                .map_err(Error::UploadTest)?;
            if !response.status().is_success() {
                return Err(Error::UploadFailure(format!(
                    "server returned {} for {url}",
                    response.status()
                )));
            }
            // Drain small acknowledgements so the HTTP/1 connection can be reused.
            const MAX_ACK_BYTES: usize = 64 * 1024;
            let mut received = 0;
            while let Some(chunk) = response.chunk().await.map_err(Error::UploadTest)? {
                received += chunk.len();
                if received > MAX_ACK_BYTES {
                    return Err(Error::UploadFailure(
                        "upload acknowledgement exceeds 64 KiB".into(),
                    ));
                }
            }
            Ok(())
        };
        match tokio::time::timeout_at(deadline, request).await {
            Ok(result) => result?,
            Err(_) => break,
        }
        sent += len as u64;
        if start.elapsed() < Duration::from_millis(200) {
            payload = payload.saturating_mul(2).min(max_payload);
        } else if start.elapsed() > Duration::from_millis(400) {
            payload = (payload / 2).max(1024).min(max_payload);
        }
    }
    if recording.is_some() && sent == 0 {
        return Err(Error::UploadFailure(
            "no upload request completed before the deadline".into(),
        ));
    }
    Ok(payload)
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
        assert_eq!(UPLOAD_PAYLOAD.len(), UploadConfig::default().max_payload);
        assert_eq!(UPLOAD_PAYLOAD[0], 0u8);
        assert_eq!(UPLOAD_PAYLOAD[255], 255u8);
        assert_eq!(UPLOAD_PAYLOAD[256], 0u8);
    }
}
