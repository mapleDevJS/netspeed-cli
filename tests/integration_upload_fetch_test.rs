//! Integration tests for upload and server parsing using wiremock + direct deserialization.

use netspeed_cli::config::File;
use netspeed_cli::progress;
use netspeed_cli::types::Server;
use netspeed_cli::upload::{UploadConfig, build_upload_url, run_with_config};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Upload Tests ──────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn test_upload_mocked_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/upload.php"))
        .respond_with(ResponseTemplate::new(200))
        .expect(4..)
        .mount(&mock_server)
        .await;

    let client = Client::new();
    let server = Server {
        id: "1".to_string(),
        url: format!("{}/upload.php", mock_server.uri()),
        name: "Mock Server".to_string(),
        sponsor: "Mock ISP".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };

    let progress = Arc::new(progress::Tracker::with_target(
        "Upload",
        indicatif::ProgressDrawTarget::hidden(),
    ));

    let result = run_with_config(
        &client,
        &server,
        true,
        progress,
        &UploadConfig {
            warmup: std::time::Duration::ZERO,
            measurement_bytes: 2_000_000,
            max_payload: 200_000,
            ..UploadConfig::default()
        },
    )
    .await;
    assert!(result.is_ok());
    let (avg, peak, total_bytes, samples) = result.unwrap();
    assert!(avg > 0.0);
    assert!(peak >= 0.0);
    assert!(total_bytes > 0);
    assert!(!samples.is_empty());
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn test_upload_mocked_all_failures() {
    let mock_server = MockServer::start().await;

    // Mix of 500 (failure) and 200 (success) — verify bytes counted only on success
    Mock::given(method("POST"))
        .and(path("/upload.php"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = Client::new();
    let server = Server {
        id: "1".to_string(),
        url: format!("{}/upload.php", mock_server.uri()),
        name: "Mock Server".to_string(),
        sponsor: "Mock ISP".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };

    let progress = Arc::new(progress::Tracker::with_target(
        "Upload",
        indicatif::ProgressDrawTarget::hidden(),
    ));

    let result = run_with_config(
        &client,
        &server,
        true,
        progress,
        &UploadConfig {
            warmup: std::time::Duration::ZERO,
            measurement_bytes: 2_000_000,
            max_payload: 200_000,
            ..UploadConfig::default()
        },
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_upload_build_url() {
    let url = build_upload_url("http://example.com/speedtest/upload.php");
    assert_eq!(url, "http://example.com/speedtest/upload.php");
}

// ── Server XML Deserialization ────────────────────────────────────────
// Tests the XML attribute format (@id, @url, etc.) used by speedtest.net

use quick_xml::de::from_str;

#[derive(Debug, Deserialize)]
struct TestServer {
    #[serde(rename = "@id")]
    #[allow(dead_code)]
    id: String,
    #[serde(rename = "@url")]
    #[allow(dead_code)]
    url: String,
    #[serde(rename = "@name")]
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "@sponsor")]
    #[allow(dead_code)]
    sponsor: String,
    #[serde(rename = "@country")]
    #[allow(dead_code)]
    country: String,
    #[serde(rename = "@lat")]
    #[allow(dead_code)]
    lat: f64,
    #[serde(rename = "@lon")]
    #[allow(dead_code)]
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct TestServersWrapper {
    #[serde(rename = "server", default)]
    servers: Vec<TestServer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "settings")]
struct TestServerConfig {
    #[serde(rename = "servers")]
    servers_wrapper: TestServersWrapper,
}

#[test]
fn test_server_xml_valid() {
    let xml = r#"<?xml version="1.0"?>
<settings>
    <servers>
        <server url="http://s1/upload.php" name="S1" sponsor="ISP1" country="US" id="1" lat="40.0" lon="-74.0"/>
        <server url="http://s2/upload.php" name="S2" sponsor="ISP2" country="CA" id="2" lat="43.0" lon="-79.0"/>
    </servers>
</settings>"#;
    let config: TestServerConfig = from_str(xml).unwrap();
    assert_eq!(config.servers_wrapper.servers.len(), 2);
    assert_eq!(config.servers_wrapper.servers[0].id, "1");
    assert_eq!(config.servers_wrapper.servers[1].country, "CA");
}

#[test]
fn test_server_xml_empty() {
    let xml = r#"<?xml version="1.0"?><settings><servers></servers></settings>"#;
    let config: TestServerConfig = from_str(xml).unwrap();
    assert!(config.servers_wrapper.servers.is_empty());
}

#[test]
fn test_server_xml_malformed() {
    let xml = "<servers><server unclosed>";
    let result: Result<TestServerConfig, _> = from_str(xml);
    assert!(result.is_err());
}

#[test]
fn test_server_xml_no_servers_tag() {
    let xml = r#"<?xml version="1.0"?><settings></settings>"#;
    // Without <servers> tag, deserialization fails because the field is required
    let result: Result<TestServerConfig, _> = from_str(xml);
    assert!(result.is_err()); // "missing field `servers`"
}

// ── Config File Tests ────────────────────────────────────────────────

#[test]
fn test_config_file_all_fields() {
    let toml = r"
        no_download = true
        no_upload = false
        single = true
        bytes = true
        simple = false
        csv = true
        csv_delimiter = ';'
        csv_header = true
        json = false
        timeout = 30
    ";
    let config: File = toml::from_str(toml).unwrap();
    assert_eq!(config.no_download, Some(true));
    assert_eq!(config.timeout, Some(30));
    assert_eq!(config.csv_delimiter, Some(';'));
}

#[test]
fn test_config_file_empty() {
    let toml = "";
    let config: File = toml::from_str(toml).unwrap();
    assert!(config.no_download.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_config_file_unknown_fields() {
    let toml = r#"
        no_download = true
        unknown_field = "ignored"
    "#;
    let config: File = toml::from_str(toml).unwrap();
    assert_eq!(config.no_download, Some(true));
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn upload_limits_warmup_adaptation_and_deadline() {
    use std::time::{Duration, Instant};
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;
    let server = Server {
        id: "1".into(),
        url: format!("{}/upload.php", mock.uri()),
        name: "Local".into(),
        sponsor: "Test".into(),
        country: "CA".into(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };
    let config = UploadConfig {
        warmup: Duration::from_secs(1),
        duration: Duration::from_secs(1),
        warmup_bytes: 3_000,
        measurement_bytes: 12_000,
        initial_payload: 1_000,
        max_payload: 4_000,
    };
    let tracker = || {
        Arc::new(progress::Tracker::with_target(
            "Upload",
            indicatif::ProgressDrawTarget::hidden(),
        ))
    };
    let (_, _, measured, _) = run_with_config(&Client::new(), &server, true, tracker(), &config)
        .await
        .unwrap();
    let requests = mock.received_requests().await.unwrap();
    let sizes: Vec<_> = requests.iter().map(|r| r.body.len()).collect();
    assert_eq!(measured, 12_000);
    assert_eq!(sizes.iter().sum::<usize>(), 15_000);
    assert_eq!(&sizes[..3], &[1_000, 2_000, 4_000]);
    mock.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&mock)
        .await;
    let start = Instant::now();
    let result = run_with_config(
        &Client::new(),
        &server,
        true,
        tracker(),
        &UploadConfig {
            warmup: Duration::ZERO,
            duration: Duration::from_millis(100),
            ..config
        },
    )
    .await;
    assert!(result.is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn latency_monitor_stops_on_failure_and_cancellation() {
    use std::time::Duration;
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;
    let server = Server {
        id: "1".into(),
        url: format!("{}/upload.php", mock.uri()),
        name: "Local".into(),
        sponsor: "Test".into(),
        country: "CA".into(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };
    for cancel in [false, true] {
        mock.reset().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;
        let future = netspeed_cli::task_runner::run_bandwidth_test(
            Client::new(),
            &server,
            "test",
            false,
            |_| async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                Err(netspeed_cli::error::Error::UploadFailure("expected".into()))
            },
        );
        if cancel {
            assert!(
                tokio::time::timeout(Duration::from_millis(120), future)
                    .await
                    .is_err()
            );
        } else {
            assert!(future.await.is_err());
        }
        tokio::time::sleep(Duration::from_millis(30)).await;
        let count = mock.received_requests().await.unwrap().len();
        assert!(count > 0);
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(mock.received_requests().await.unwrap().len(), count);
    }
}

#[tokio::test]
#[ignore = "requires local socket binding"]
async fn upload_acknowledgement_size_boundary_is_enforced() {
    let mock = MockServer::start().await;
    let server = Server {
        id: "1".into(),
        url: format!("{}/upload.php", mock.uri()),
        name: "local".into(),
        sponsor: "test".into(),
        country: "CA".into(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };
    let config = UploadConfig {
        warmup: std::time::Duration::ZERO,
        measurement_bytes: 200_000,
        ..UploadConfig::default()
    };
    for size in [64 * 1024, 64 * 1024 + 1] {
        mock.reset().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0; size]))
            .mount(&mock)
            .await;
        let tracker = Arc::new(progress::Tracker::with_target(
            "upload",
            indicatif::ProgressDrawTarget::hidden(),
        ));
        let result = run_with_config(&Client::new(), &server, true, tracker, &config).await;
        if size == 64 * 1024 {
            assert_eq!(result.unwrap().2, 200_000);
        } else {
            let error = result.unwrap_err();
            assert!(matches!(
                error,
                netspeed_cli::error::Error::UploadFailure(_)
            ));
            assert!(error.to_string().contains("acknowledgement exceeds"));
        }
    }
}
