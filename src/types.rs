use serde::{Deserialize, Serialize};

/// Represents a Speedtest server with location and performance data.
///
/// This struct is used both for server discovery (parsing the server list)
/// and for tracking measured values during tests.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    /// Unique server identifier
    pub id: String,
    /// Base URL for test file downloads and uploads
    pub url: String,
    /// Human-readable server name
    pub name: String,
    /// Server operator (ISP or hosting company)
    pub sponsor: String,
    /// Country code (e.g., "US", "DE")
    pub country: String,
    /// Server latitude
    pub lat: f64,
    /// Server longitude
    pub lon: f64,
    /// Distance from client in kilometers (calculated via Haversine formula)
    pub distance: f64,
    /// Measured latency in milliseconds
    pub latency: f64,
}

/// Complete results from a speed test session.
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    /// Information about the test server
    pub server: ServerInfo,
    /// Ping latency in milliseconds (None if ping test was skipped)
    pub ping: Option<f64>,
    /// Download speed in bits per second (None if download test was skipped)
    pub download: Option<f64>,
    /// Upload speed in bits per second (None if upload test was skipped)
    pub upload: Option<f64>,
    /// Shareable results URL (if `--share` was used)
    pub share_url: Option<String>,
    /// ISO 8601 timestamp when the test was run
    pub timestamp: String,
    /// Client public IP address (if discovery succeeded)
    pub client_ip: Option<String>,
}

/// Summary information about the test server for result output.
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    /// Server ID
    pub id: String,
    /// Server name
    pub name: String,
    /// Server operator
    pub sponsor: String,
    /// Country code
    pub country: String,
    /// Distance from client in km
    pub distance: f64,
}

/// CSV output format for speed test results.
#[derive(Debug, Clone, Serialize)]
pub struct CsvOutput {
    /// Server ID
    pub server_id: String,
    /// Server operator
    pub sponsor: String,
    /// Server name
    pub server_name: String,
    /// Test timestamp
    pub timestamp: String,
    /// Distance from client in km
    pub distance: f64,
    /// Ping in ms
    pub ping: f64,
    /// Download speed in bits/s
    pub download: f64,
    /// Upload speed in bits/s
    pub upload: f64,
    /// Share URL (empty if not requested)
    pub share: String,
    /// Client IP address
    pub ip_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_server() -> Server {
        Server {
            id: "12345".to_string(),
            url: "http://server1.example.com".to_string(),
            name: "Server One".to_string(),
            sponsor: "ISP Corp".to_string(),
            country: "US".to_string(),
            lat: 40.7128,
            lon: -74.0060,
            distance: 100.5,
            latency: 25.3,
        }
    }

    fn create_sample_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Server One".to_string(),
                sponsor: "ISP Corp".to_string(),
                country: "US".to_string(),
                distance: 100.5,
            },
            ping: Some(25.3),
            download: Some(100_000_000.0),
            upload: Some(50_000_000.0),
            share_url: Some("https://example.com/result".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        }
    }

    #[test]
    fn test_server_clone() {
        let server = create_sample_server();
        let cloned = server.clone();
        assert_eq!(cloned.id, server.id);
        assert_eq!(cloned.name, server.name);
    }

    #[test]
    fn test_server_debug_format() {
        let server = create_sample_server();
        let debug_str = format!("{:?}", server);
        assert!(debug_str.contains("12345"));
        assert!(debug_str.contains("Server One"));
    }

    #[test]
    fn test_server_deserialize_from_json() {
        let json = r#"{
            "id": "67890",
            "url": "http://server2.example.com",
            "name": "Server Two",
            "sponsor": "Telco Inc",
            "country": "DE",
            "lat": 52.5200,
            "lon": 13.4050,
            "distance": 200.0,
            "latency": 30.0
        }"#;

        let server: Server = serde_json::from_str(json).unwrap();
        assert_eq!(server.id, "67890");
        assert_eq!(server.name, "Server Two");
        assert_eq!(server.country, "DE");
        assert!((server.lat - 52.5200).abs() < f64::EPSILON);
    }

    #[test]
    fn test_server_serialize_to_json() {
        let server = create_sample_server();
        let json = serde_json::to_string(&server).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("Server One"));
        assert!(json.contains("ISP Corp"));
    }

    #[test]
    fn test_server_roundtrip_json() {
        let server = create_sample_server();
        let json = serde_json::to_string(&server).unwrap();
        let deserialized: Server = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, server.id);
        assert_eq!(deserialized.name, server.name);
        assert_eq!(deserialized.sponsor, server.sponsor);
        assert_eq!(deserialized.country, server.country);
        assert!((deserialized.lat - server.lat).abs() < f64::EPSILON);
        assert!((deserialized.lon - server.lon).abs() < f64::EPSILON);
    }

    #[test]
    fn test_test_result_clone() {
        let result = create_sample_test_result();
        let cloned = result.clone();
        assert_eq!(cloned.server.id, result.server.id);
        assert_eq!(cloned.ping, result.ping);
    }

    #[test]
    fn test_test_result_serialize_to_json() {
        let result = create_sample_test_result();
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("100000000"));
        assert!(json.contains("2024-01-01T00:00:00Z"));
    }

    #[test]
    fn test_test_result_with_none_values() {
        let result = TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Server One".to_string(),
                sponsor: "ISP Corp".to_string(),
                country: "US".to_string(),
                distance: 100.5,
            },
            ping: None,
            download: None,
            upload: None,
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_server_info_serialize() {
        let info = ServerInfo {
            id: "999".to_string(),
            name: "Test".to_string(),
            sponsor: "Sponsor".to_string(),
            country: "GB".to_string(),
            distance: 50.0,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("999"));
        assert!(json.contains("Test"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_csv_output_serialize() {
        let csv = CsvOutput {
            server_id: "123".to_string(),
            sponsor: "ISP".to_string(),
            server_name: "Server".to_string(),
            timestamp: "2024-01-01".to_string(),
            distance: 100.0,
            ping: 25.0,
            download: 100_000_000.0,
            upload: 50_000_000.0,
            share: "https://example.com".to_string(),
            ip_address: "1.2.3.4".to_string(),
        };

        let json = serde_json::to_string(&csv).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("100000000"));
    }

    #[test]
    fn test_server_debug_contains_fields() {
        let server = create_sample_server();
        let debug = format!("{:?}", server);
        assert!(debug.contains("latency"));
        assert!(debug.contains("distance"));
    }
}
