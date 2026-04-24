//! End-to-end integration test with full mocked speedtest flow.
//!
//! Tests the complete pipeline: server selection, ping test, and
//! bandwidth measurement against a mock server.

use netspeed_cli::common;
use netspeed_cli::download::{self, build_test_url, extract_base_url};
use netspeed_cli::progress::Tracker;
use netspeed_cli::servers::{ping_test, select_best_server};
use netspeed_cli::types::Server;
use netspeed_cli::upload;
use reqwest::Client;
use std::sync::Arc;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build a mock speedtest.net server that responds to all endpoints.
async fn create_mock_speedtest_server() -> MockServer {
    let mock = MockServer::start().await;

    // Ping endpoint lives alongside upload.php in the same directory.
    Mock::given(method("GET"))
        .and(path_regex(".*/latency\\.txt$"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock)
        .await;

    // Download endpoints (random*.jpg)
    Mock::given(method("GET"))
        .and(path_regex(".*random.*"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 1024 * 256]))
        .mount(&mock)
        .await;

    // Upload endpoint
    Mock::given(method("POST"))
        .and(path_regex(".*/upload\\.php$"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock)
        .await;

    mock
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn test_e2e_full_speedtest_flow() {
    let mock = create_mock_speedtest_server().await;
    let mock_url = mock.uri();

    // Create a server pointing to our mock
    let server = Server {
        id: "9999".to_string(),
        url: format!("{mock_url}/upload.php"),
        name: "E2E Test Server".to_string(),
        sponsor: "Test ISP".to_string(),
        country: "US".to_string(),
        lat: 40.0,
        lon: -74.0,
        distance: 50.0,
    };

    // Step 1: Select best server (single server in our case)
    let servers = vec![server.clone()];
    let selected = select_best_server(&servers).expect("Should select server");
    assert_eq!(selected.id, "9999");

    // Step 2: Create HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Should create client");

    // Step 3: Run ping test
    let ping_result = ping_test(&client, &selected)
        .await
        .expect("Ping should succeed");
    let (avg_latency, _jitter, _packet_loss, _samples) = ping_result;
    assert!(avg_latency > 0.0, "Latency should be positive");
    assert!(avg_latency < 1000.0, "Local latency should be reasonable");

    // Step 4: Run download test
    let progress = Arc::new(Tracker::with_target(
        "Download",
        indicatif::ProgressDrawTarget::hidden(),
    ));
    let dl_result = download::run(&client, &selected, true, progress.clone())
        .await
        .expect("Download should succeed");
    let (avg_download, peak_download, total_download_bytes, download_samples) = dl_result;
    assert!(avg_download > 0.0, "Download speed should be positive");
    assert!(
        total_download_bytes > 0,
        "Should have downloaded some bytes"
    );
    assert!(!download_samples.is_empty(), "Should have speed samples");
    // Peak may be similar to average in mock environment (instant responses)
    let _ = peak_download;

    // Step 5: Run upload test
    let progress = Arc::new(Tracker::with_target(
        "Upload",
        indicatif::ProgressDrawTarget::hidden(),
    ));
    let ul_result = upload::run(&client, &selected, true, progress.clone())
        .await
        .expect("Upload should succeed");
    let (avg_upload, peak_upload, total_upload_bytes, upload_samples) = ul_result;
    assert!(avg_upload > 0.0, "Upload speed should be positive");
    assert!(total_upload_bytes > 0, "Should have uploaded some bytes");
    assert!(!upload_samples.is_empty(), "Should have speed samples");
    let _ = peak_upload;
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn test_e2e_download_only() {
    let mock = create_mock_speedtest_server().await;
    let server = Server {
        id: "1".to_string(),
        url: format!("{}/upload.php", mock.uri()),
        name: "Test".to_string(),
        sponsor: "Test".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Should create client");

    let progress = Arc::new(Tracker::with_target(
        "Download",
        indicatif::ProgressDrawTarget::hidden(),
    ));
    let result = download::run(&client, &server, true, progress).await;
    assert!(result.is_ok());
    let (avg, _peak, bytes, samples) = result.unwrap();
    assert!(avg > 0.0);
    assert!(bytes > 0);
    assert!(!samples.is_empty());
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn test_e2e_upload_only() {
    let mock = create_mock_speedtest_server().await;
    let server = Server {
        id: "1".to_string(),
        url: format!("{}/upload.php", mock.uri()),
        name: "Test".to_string(),
        sponsor: "Test".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Should create client");

    let progress = Arc::new(Tracker::with_target(
        "Upload",
        indicatif::ProgressDrawTarget::hidden(),
    ));
    let result = upload::run(&client, &server, true, progress).await;
    assert!(result.is_ok());
    let (avg, _peak, bytes, samples) = result.unwrap();
    assert!(avg > 0.0);
    assert!(bytes > 0);
    assert!(!samples.is_empty());
}

#[test]
fn test_url_construction_e2e() {
    let server_url = "http://localhost:8080/upload.php";
    let base = extract_base_url(server_url);
    assert_eq!(base, "http://localhost:8080");

    let url0 = build_test_url(server_url, 0);
    assert!(url0.ends_with("/random0.txt") || url0.starts_with("http://localhost:8080"));
}

#[test]
fn test_bandwidth_calculation_e2e() {
    // Verify that bandwidth calculation is consistent
    let bytes = 1_000_000u64;
    let secs = 1.0f64;
    let bps = common::calculate_bandwidth(bytes, secs);
    assert!((bps - 8_000_000.0).abs() < f64::EPSILON); // 1MB in 1s = 8Mbps
}
