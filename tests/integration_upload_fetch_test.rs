//! Integration tests for upload and server parsing using wiremock + direct deserialization.

use netspeed_cli::config::File;
use netspeed_cli::progress;
use netspeed_cli::types::Server;
use netspeed_cli::upload::{build_upload_url, run};
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

    let result = run(&client, &server, true, progress).await;
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

    let result = run(&client, &server, true, progress).await;
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
