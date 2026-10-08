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
use std::time::Duration;

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
    run_with_config(client, server, single, progress, &DownloadConfig::default()).await
}

/// Phase-wide download budgets across all streams. Warm-up is not measured.
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub warmup: Duration,
    pub duration: Duration,
    pub warmup_bytes: u64,
    pub measurement_bytes: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            warmup: Duration::from_secs(1),
            duration: Duration::from_secs(5),
            warmup_bytes: 16 * 1024 * 1024,
            measurement_bytes: 256 * 1024 * 1024,
        }
    }
}

/// Run download with explicit phase-wide time and traffic limits.
///
/// # Errors
/// Returns a download error for invalid limits, failed requests, or no data.
pub async fn run_with_config(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<Tracker>,
    config: &DownloadConfig,
) -> Result<(f64, f64, u64, Vec<f64>), Error> {
    let streams = TestConfig::stream_count_for(single);
    if config.duration.is_zero() || config.measurement_bytes < streams as u64 {
        return Err(Error::DownloadFailure(
            "invalid download measurement limits".into(),
        ));
    }
    let warmup_deadline = tokio::time::Instant::now() + config.warmup;
    futures::future::try_join_all((0..streams).map(|i| {
        download_stream(
            client,
            &server.url,
            i,
            warmup_deadline,
            config.warmup_bytes / streams as u64,
            None,
        )
    }))
    .await?;
    let deadline = tokio::time::Instant::now() + config.duration;
    let result = run_concurrent_streams(
        config.measurement_bytes,
        streams,
        progress,
        "download",
        |i, state, interval| {
            let client = client.clone();
            let url = server.url.clone();
            let budget = config.measurement_bytes / streams as u64;
            tokio::spawn(async move {
                download_stream(&client, &url, i, deadline, budget, Some((state, interval))).await
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

async fn download_stream(
    client: &Client,
    url: &str,
    mut index: usize,
    deadline: tokio::time::Instant,
    budget: u64,
    recording: Option<(Arc<crate::bandwidth_loop::LoopState>, u64)>,
) -> Result<(), Error> {
    let mut received = 0;
    while received < budget && tokio::time::Instant::now() < deadline {
        let request = async {
            let response = client
                .get(build_test_url(url, index))
                .send()
                .await
                .map_err(Error::DownloadTest)?
                .error_for_status()
                .map_err(Error::DownloadTest)?;
            let mut stream = response.bytes_stream();
            let before = received;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(Error::DownloadTest)?;
                let counted = (chunk.len() as u64).min(budget - received);
                received += counted;
                if let Some((state, interval)) = &recording {
                    state.record_bytes(counted, *interval);
                }
                if received == budget {
                    break;
                }
            }
            if received == before {
                return Err(Error::DownloadFailure("empty download response".into()));
            }
            Ok(())
        };
        match tokio::time::timeout_at(deadline, request).await {
            Ok(result) => result?,
            Err(_) => break,
        }
        index = (index + 1) % 4;
    }
    if recording.is_some() && received == 0 {
        return Err(Error::DownloadFailure(
            "no download data received before the deadline".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::common;
    use crate::test_config::TestConfig;

    use super::*;

    fn local_server(uri: &str) -> Server {
        Server {
            id: "1".into(),
            url: format!("{uri}/upload.php"),
            name: "local".into(),
            sponsor: "test".into(),
            country: "CA".into(),
            lat: 0.0,
            lon: 0.0,
            distance: 0.0,
        }
    }

    fn hidden_tracker() -> Arc<Tracker> {
        Arc::new(Tracker::with_target(
            "download",
            indicatif::ProgressDrawTarget::hidden(),
        ))
    }

    #[tokio::test]
    async fn invalid_download_limits_fail_without_io() {
        let config = DownloadConfig {
            duration: Duration::ZERO,
            ..DownloadConfig::default()
        };
        assert!(matches!(
            run_with_config(
                &Client::new(),
                &local_server("http://invalid"),
                true,
                hidden_tracker(),
                &config
            )
            .await,
            Err(Error::DownloadFailure(_))
        ));
    }

    #[tokio::test]
    #[ignore = "requires local socket binding"]
    async fn socket_regression_download_caps_exclude_warmup_and_span_streams() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0; 4096]))
            .mount(&mock)
            .await;
        let config = DownloadConfig {
            warmup: Duration::from_secs(1),
            duration: Duration::from_secs(1),
            warmup_bytes: 1024,
            measurement_bytes: 4096,
        };
        let (_, _, measured, _) = run_with_config(
            &Client::new(),
            &local_server(&mock.uri()),
            true,
            hidden_tracker(),
            &config,
        )
        .await
        .unwrap();
        assert_eq!(measured, 4096);
        assert_eq!(mock.received_requests().await.unwrap().len(), 2);
        let config = DownloadConfig {
            warmup: Duration::ZERO,
            measurement_bytes: 10_003,
            ..config
        };
        let (_, _, measured, _) = run_with_config(
            &Client::new(),
            &local_server(&mock.uri()),
            false,
            hidden_tracker(),
            &config,
        )
        .await
        .unwrap();
        assert_eq!(measured, 10_000);
    }

    #[tokio::test]
    #[ignore = "requires local socket binding"]
    async fn socket_regression_download_phase_deadline_and_empty_responses() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(vec![0; 1024])
                    .set_delay(Duration::from_millis(30)),
            )
            .mount(&mock)
            .await;
        let config = DownloadConfig {
            warmup: Duration::ZERO,
            duration: Duration::from_millis(100),
            warmup_bytes: 0,
            measurement_bytes: 1_000_000,
        };
        let started = std::time::Instant::now();
        let (_, _, measured, samples) = run_with_config(
            &Client::new(),
            &local_server(&mock.uri()),
            true,
            hidden_tracker(),
            &config,
        )
        .await
        .unwrap();
        assert!(measured > 0 && measured < config.measurement_bytes);
        assert!(!samples.is_empty());
        assert!(started.elapsed() < Duration::from_secs(2));
        mock.reset().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(3)))
            .mount(&mock)
            .await;
        assert!(matches!(
            run_with_config(
                &Client::new(),
                &local_server(&mock.uri()),
                true,
                hidden_tracker(),
                &config
            )
            .await,
            Err(Error::DownloadFailure(_))
        ));
        mock.reset().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;
        assert!(matches!(
            run_with_config(
                &Client::new(),
                &local_server(&mock.uri()),
                true,
                hidden_tracker(),
                &config
            )
            .await,
            Err(Error::DownloadFailure(_))
        ));
    }

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
    fn test_estimated_download_bytes_from_config() {
        // Verify the config value is reasonable (around 15 MB)
        let config = TestConfig::default();
        assert!(config.estimated_download_bytes > 10_000_000);
        assert!(config.estimated_download_bytes < 20_000_000);
    }

    #[test]
    fn test_sample_interval_constant() {
        // Verify sample interval is 50ms (20 Hz) — now defined in LoopState
        const _: () = assert!(crate::bandwidth_loop::SAMPLE_INTERVAL_MS == 50);
    }
}
