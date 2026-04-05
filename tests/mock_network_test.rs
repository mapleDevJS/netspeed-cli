use netspeed_cli::servers::ping_test;
use netspeed_cli::types::Server;
use reqwest::Client;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_ping_test_mocked() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/latency.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string("test"))
        .mount(&mock_server)
        .await;

    let client = Client::new();
    let server = Server {
        id: "1".to_string(),
        url: mock_server.uri(),
        name: "Mock Server".to_string(),
        sponsor: "Mock ISP".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };

    let result = ping_test(&client, &server).await;
    assert!(result.is_ok());
    let (ping, _jitter, _packet_loss, _samples) = result.unwrap();
    assert!(ping > 0.0);
}

#[tokio::test]
async fn test_ping_test_failure_mocked() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/latency.txt"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = Client::new();
    let server = Server {
        id: "1".to_string(),
        url: mock_server.uri(),
        name: "Mock Server".to_string(),
        sponsor: "Mock ISP".to_string(),
        country: "US".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: 0.0,
    };

    let result = ping_test(&client, &server).await;
    assert!(result.is_err());
}
