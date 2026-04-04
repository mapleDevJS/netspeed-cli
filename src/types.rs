use serde::{Deserialize, Serialize};

/// Root element for the Speedtest.net servers XML response
/// XML structure: <settings><servers><server .../></servers></settings>
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "settings")]
pub struct ServerConfig {
    #[serde(rename = "servers")]
    pub servers_wrapper: ServersWrapper,
}

/// Wrapper for the list of servers (maps to <servers> element)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServersWrapper {
    #[serde(rename = "server")]
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@sponsor")]
    pub sponsor: String,
    #[serde(rename = "@country")]
    pub country: String,
    #[serde(rename = "@lat")]
    pub lat: f64,
    #[serde(rename = "@lon")]
    pub lon: f64,
    #[serde(skip)]
    pub distance: f64,
    #[serde(skip)]
    pub latency: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub server: ServerInfo,
    pub ping: Option<f64>,
    pub download: Option<f64>,
    pub upload: Option<f64>,
    pub share_url: Option<String>,
    pub timestamp: String,
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub sponsor: String,
    pub country: String,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ServerListOutput {
    pub servers: Vec<ServerListItem>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ServerListItem {
    pub id: String,
    pub name: String,
    pub sponsor: String,
    pub country: String,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvOutput {
    pub server_id: String,
    pub sponsor: String,
    pub server_name: String,
    pub timestamp: String,
    pub distance: f64,
    pub ping: f64,
    pub download: f64,
    pub upload: f64,
    pub share: String,
    pub ip_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_serialization() {
        let server = Server {
            id: "1234".to_string(),
            url: "http://example.com".to_string(),
            name: "Test Server".to_string(),
            sponsor: "Test ISP".to_string(),
            country: "US".to_string(),
            lat: 40.7128,
            lon: -74.0060,
            distance: 100.5,
            latency: 15.2,
        };

        let json = serde_json::to_string(&server).unwrap();
        // With @ prefix for XML attributes, serde serializes them as normal fields in JSON
        assert!(json.contains("\"1234\""));
        assert!(json.contains("\"Test Server\""));
    }

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.5,
            },
            ping: Some(15.234),
            download: Some(150_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.1".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ping\":15.234"));
        assert!(json.contains("\"download\":150000000.0"));
    }

    #[test]
    fn test_csv_output_serialization() {
        let csv = CsvOutput {
            server_id: "1234".to_string(),
            sponsor: "Test ISP".to_string(),
            server_name: "Test Server".to_string(),
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            distance: 100.5,
            ping: 15.234,
            download: 150_000_000.0,
            upload: 50_000_000.0,
            share: "http://example.com/share".to_string(),
            ip_address: "192.168.1.1".to_string(),
        };

        let json = serde_json::to_string(&csv).unwrap();
        assert!(json.contains("\"server_id\":\"1234\""));
        assert!(json.contains("\"ping\":15.234"));
    }

    #[test]
    fn test_server_clone() {
        let server = Server {
            id: "1234".to_string(),
            url: "http://example.com".to_string(),
            name: "Test".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            lat: 40.0,
            lon: -74.0,
            distance: 0.0,
            latency: 0.0,
        };

        let cloned = server.clone();
        assert_eq!(cloned.id, server.id);
        assert_eq!(cloned.name, server.name);
    }
}
