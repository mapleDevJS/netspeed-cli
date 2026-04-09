use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub server: ServerInfo,
    pub ping: Option<f64>,
    pub jitter: Option<f64>,
    pub packet_loss: Option<f64>,
    pub download: Option<f64>,
    pub download_peak: Option<f64>,
    pub upload: Option<f64>,
    pub upload_peak: Option<f64>,
    pub latency_download: Option<f64>,
    pub latency_upload: Option<f64>,
    pub download_samples: Option<Vec<f64>>,
    pub upload_samples: Option<Vec<f64>>,
    pub ping_samples: Option<Vec<f64>>,
    pub timestamp: String,
    pub client_ip: Option<String>,
}

impl TestResult {
    /// Build a `TestResult` from ping test output and download/upload metrics.
    ///
    /// Takes `BandwidthMetrics` tuples instead of internal `TestRunResult` to
    /// decouple the public API from unstable implementation details.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_test_runs(
        server: ServerInfo,
        ping: Option<f64>,
        jitter: Option<f64>,
        packet_loss: Option<f64>,
        ping_samples: Vec<f64>,
        dl: &BandwidthMetrics,
        ul: &BandwidthMetrics,
        client_ip: Option<String>,
    ) -> Self {
        fn opt_samples(v: &[f64]) -> Option<Vec<f64>> {
            if v.is_empty() { None } else { Some(v.to_vec()) }
        }
        fn opt_positive(v: f64) -> Option<f64> {
            if v > 0.0 { Some(v) } else { None }
        }

        Self {
            server,
            ping,
            jitter,
            packet_loss,
            download: opt_positive(dl.avg_bps),
            download_peak: opt_positive(dl.peak_bps),
            upload: opt_positive(ul.avg_bps),
            upload_peak: opt_positive(ul.peak_bps),
            latency_download: dl.latency_under_load,
            latency_upload: ul.latency_under_load,
            download_samples: opt_samples(&dl.speed_samples),
            upload_samples: opt_samples(&ul.speed_samples),
            ping_samples: opt_samples(&ping_samples),
            timestamp: chrono::Utc::now().to_rfc3339(),
            client_ip,
        }
    }
}

/// Bandwidth test metrics — decouples `TestResult` from internal `test_runner`.
#[derive(Debug, Clone)]
pub struct BandwidthMetrics {
    pub avg_bps: f64,
    pub peak_bps: f64,
    pub total_bytes: u64,
    pub duration_secs: f64,
    pub speed_samples: Vec<f64>,
    pub latency_under_load: Option<f64>,
}

impl From<&crate::test_runner::TestRunResult> for BandwidthMetrics {
    fn from(r: &crate::test_runner::TestRunResult) -> Self {
        Self {
            avg_bps: r.avg_bps,
            peak_bps: r.peak_bps,
            total_bytes: r.total_bytes,
            duration_secs: r.duration_secs,
            speed_samples: r.speed_samples.clone(),
            latency_under_load: r.latency_under_load,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub sponsor: String,
    pub country: String,
    pub distance: f64,
}

impl From<&Server> for ServerInfo {
    fn from(server: &Server) -> Self {
        Self {
            id: server.id.clone(),
            name: server.name.clone(),
            sponsor: server.sponsor.clone(),
            country: server.country.clone(),
            distance: server.distance,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvOutput {
    pub server_id: String,
    pub sponsor: String,
    pub server_name: String,
    pub timestamp: String,
    pub distance: f64,
    pub ping: f64,
    pub jitter: f64,
    pub packet_loss: f64,
    pub download: f64,
    pub download_peak: f64,
    pub upload: f64,
    pub upload_peak: f64,
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
            jitter: Some(1.2),
            packet_loss: Some(0.0),
            download: Some(150_000_000.0),
            download_peak: Some(180_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            latency_download: Some(25.0),
            latency_upload: Some(30.0),
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.1".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ping\":15.234"));
        assert!(json.contains("\"jitter\":1.2"));
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
            jitter: 1.2,
            packet_loss: 0.0,
            download: 150_000_000.0,
            download_peak: 180_000_000.0,
            upload: 50_000_000.0,
            upload_peak: 60_000_000.0,
            ip_address: "192.168.1.1".to_string(),
        };

        let json = serde_json::to_string(&csv).unwrap();
        assert!(json.contains("\"server_id\":\"1234\""));
        assert!(json.contains("\"ping\":15.234"));
        assert!(json.contains("\"jitter\":1.2"));
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
        };

        let cloned = server.clone();
        assert_eq!(cloned.id, server.id);
        assert_eq!(cloned.name, server.name);
    }
}
