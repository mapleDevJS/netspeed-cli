// HTTP integration tests using wiremock to mock speedtest.net endpoints

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Sample XML response from /speedtest-servers-static.php
    fn sample_servers_xml() -> String {
        r#"<?xml version="1.0" encoding="iso-8859-1"?>
        <settings>
        <servers>
            <server>
                <id>1001</id>
                <url>http://server1.test.com/</url>
                <lat>40.7128</lat>
                <lon>-74.0060</lon>
                <name>New York</name>
                <country>US</country>
                <sponsor>ISP 1</sponsor>
            </server>
            <server>
                <id>1002</id>
                <url>http://server2.test.com/</url>
                <lat>34.0522</lat>
                <lon>-118.2437</lon>
                <name>Los Angeles</name>
                <country>US</country>
                <sponsor>ISP 2</sponsor>
            </server>
            <server>
                <id>1003</id>
                <url>http://server3.test.com/</url>
                <lat>41.8781</lat>
                <lon>-87.6298</lon>
                <name>Chicago</name>
                <country>US</country>
                <sponsor>ISP 3</sponsor>
            </server>
        </servers>
        </settings>"#
            .to_string()
    }

    /// Sample XML response from /speedtest-config.php
    fn sample_client_config_xml() -> String {
        r#"<?xml version="1.0" encoding="iso-8859-1"?>
        <settings>
            <client>
                <ip>1.2.3.4</ip>
                <lat>40.7128</lat>
                <lon>-74.0060</lon>
            </client>
        </settings>"#
            .to_string()
    }

    #[tokio::test]
    async fn test_fetch_servers_parses_xml() {
        // Mock the server list endpoint
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/speedtest-servers-static.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string(sample_servers_xml()))
            .mount(&mock_server)
            .await;

        // Test XML parsing of raw server data
        use quick_xml::de::from_str;

        #[derive(Debug, serde::Deserialize)]
        struct TestServers {
            #[serde(rename = "servers", default)]
            servers_list: TestServersList,
        }

        #[derive(Debug, serde::Deserialize, Default)]
        struct TestServersList {
            #[serde(rename = "server", default)]
            servers: Vec<TestRawServer>,
        }

        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct TestRawServer {
            #[serde(rename = "id")]
            id: String,
            #[serde(rename = "url")]
            url: String,
            #[serde(rename = "lat")]
            lat: String,
            #[serde(rename = "lon")]
            lon: String,
            #[serde(rename = "name")]
            name: String,
            #[serde(rename = "country")]
            country: String,
            #[serde(rename = "sponsor")]
            sponsor: String,
        }

        let parsed: TestServers = from_str(&sample_servers_xml()).unwrap();
        assert_eq!(parsed.servers_list.servers.len(), 3);

        let server1 = &parsed.servers_list.servers[0];
        assert_eq!(server1.id, "1001");
        assert_eq!(server1.name, "New York");
        assert_eq!(server1.lat, "40.7128");
        assert_eq!(server1.lon, "-74.0060");
        assert_eq!(server1.sponsor, "ISP 1");

        let server2 = &parsed.servers_list.servers[1];
        assert_eq!(server2.id, "1002");
        assert_eq!(server2.name, "Los Angeles");
    }

    #[tokio::test]
    async fn test_fetch_client_config_parses_xml() {
        use quick_xml::de::from_str;

        let parsed: netspeed_cli::servers::SpeedtestConfig =
            from_str(&sample_client_config_xml()).unwrap();

        let client_info = parsed.client_info.unwrap();
        assert_eq!(client_info.ip, "1.2.3.4");
        assert_eq!(client_info.lat, "40.7128");
        assert_eq!(client_info.lon, "-74.0060");
    }

    #[tokio::test]
    async fn test_discover_client_ip_from_mock() {
        let mock_server = MockServer::start().await;

        // Mock the IP endpoint
        Mock::given(method("GET"))
            .and(path("/api/js/ip"))
            .respond_with(ResponseTemplate::new(200).set_body_string("203.0.113.42"))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        // Test that we can get a response from the mock endpoint directly
        let response = client
            .get(format!("{}/api/js/ip", mock_server.uri()))
            .send()
            .await
            .unwrap();
        let text = response.text().await.unwrap();
        assert_eq!(text.trim(), "203.0.113.42");
    }

    #[tokio::test]
    async fn test_server_selection_with_empty_list() {
        use netspeed_cli::servers::select_best_server;
        use netspeed_cli::types::Server;

        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No servers available"));
    }

    #[tokio::test]
    async fn test_server_selection_closest() {
        use netspeed_cli::servers::{calculate_distances, select_best_server};
        use netspeed_cli::types::Server;

        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://la.test.com/".to_string(),
                name: "LA".to_string(),
                sponsor: "ISP 1".to_string(),
                country: "US".to_string(),
                lat: 34.0522,
                lon: -118.2437,
                distance: 0.0,
                latency: 0.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://nyc.test.com/".to_string(),
                name: "NYC".to_string(),
                sponsor: "ISP 2".to_string(),
                country: "US".to_string(),
                lat: 40.7128,
                lon: -74.0060,
                distance: 0.0,
                latency: 0.0,
            },
        ];

        // Client is in NYC, so NYC server should be closest
        calculate_distances(&mut servers, 40.8, -74.1);
        let best = select_best_server(&servers).unwrap();
        assert_eq!(best.id, "2"); // NYC server
    }

    #[tokio::test]
    async fn test_http_error_handling() {
        let mock_server = MockServer::start().await;

        // Mock a 500 error response
        Mock::given(method("GET"))
            .and(path("/speedtest-servers-static.php"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/speedtest-servers-static.php",
                mock_server.uri()
            ))
            .send()
            .await
            .unwrap();

        assert!(!response.status().is_success());
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn test_http_timeout_simulation() {
        let mock_server = MockServer::start().await;

        // Mock a very slow response (simulates timeout scenario)
        Mock::given(method("GET"))
            .and(path("/slow-endpoint"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("OK")
                    .set_delay(std::time::Duration::from_millis(10)),
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(5))
            .build()
            .unwrap();

        // This should timeout
        let result = client
            .get(format!("{}/slow-endpoint", mock_server.uri()))
            .send()
            .await;

        assert!(result.is_err(), "Request should have timed out");
    }

    #[test]
    fn test_share_result_hash_generation() {
        use netspeed_cli::share::generate_result_hash;
        use netspeed_cli::types::{ServerInfo, TestResult};

        let result = TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 10.5,
            },
            ping: Some(15.0),
            download: Some(150_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        };

        let hash = generate_result_hash(&result);
        assert!(!hash.is_empty(), "Hash should not be empty");
        assert!(hash.len() == 16, "Hash should be 16 chars (8 bytes hex)");

        // Same input should produce same hash (deterministic)
        let hash2 = generate_result_hash(&result);
        assert_eq!(hash, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_share_result_hash_different_results() {
        use netspeed_cli::share::generate_result_hash;
        use netspeed_cli::types::{ServerInfo, TestResult};

        let result1 = TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 10.5,
            },
            ping: Some(15.0),
            download: Some(150_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        };

        let mut result2 = result1.clone();
        result2.download = Some(200_000_000.0); // Different download speed

        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        assert_ne!(
            hash1, hash2,
            "Different results should produce different hashes"
        );
    }

    #[test]
    fn test_server_selection_with_latency_weighting() {
        use netspeed_cli::servers::select_best_server;
        use netspeed_cli::types::Server;

        // Create servers where closest by distance has high latency
        let servers = vec![
            Server {
                id: "1".to_string(), // Close but slow
                url: "http://close.test.com/".to_string(),
                name: "Close".to_string(),
                sponsor: "ISP 1".to_string(),
                country: "US".to_string(),
                lat: 40.8, // Very close to client
                lon: -74.1,
                distance: 10.0, // Closest
                latency: 100.0, // High latency
            },
            Server {
                id: "2".to_string(), // Medium distance, medium latency
                url: "http://medium.test.com/".to_string(),
                name: "Medium".to_string(),
                sponsor: "ISP 2".to_string(),
                country: "US".to_string(),
                lat: 39.0,
                lon: -77.0,
                distance: 300.0,
                latency: 30.0, // Better latency
            },
            Server {
                id: "3".to_string(), // Far but worst latency
                url: "http://far.test.com/".to_string(),
                name: "Far".to_string(),
                sponsor: "ISP 3".to_string(),
                country: "US".to_string(),
                lat: 34.0,
                lon: -118.0,
                distance: 4000.0, // Farthest
                latency: 80.0,    // Medium latency
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // Server 2 should win due to better latency despite being farther
        assert_eq!(best.id, "2");
    }

    #[test]
    fn test_progress_tracker_basic() {
        use netspeed_cli::progress::ProgressTracker;

        let tracker = ProgressTracker::new(4, false);
        assert_eq!(tracker.progress(), 0.0);
        assert_eq!(tracker.total_bytes(), 0);

        tracker.add_chunk(1_000_000);
        assert_eq!(tracker.total_bytes(), 1_000_000);
        assert!((tracker.progress() - 0.25).abs() < 0.01);

        tracker.add_chunk(2_000_000);
        assert_eq!(tracker.total_bytes(), 3_000_000);
        assert!((tracker.progress() - 0.5).abs() < 0.01);

        tracker.finish();
    }

    #[test]
    fn test_progress_tracker_zero_chunks() {
        use netspeed_cli::progress::ProgressTracker;

        let tracker = ProgressTracker::new(0, false);
        assert_eq!(tracker.progress(), 0.0);
        tracker.finish();
    }
}
