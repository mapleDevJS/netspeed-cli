use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhaseState {
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhaseResult {
    pub state: PhaseState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl PhaseResult {
    #[must_use]
    pub fn completed() -> Self {
        Self {
            state: PhaseState::Completed,
            reason: None,
        }
    }

    #[must_use]
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            state: PhaseState::Skipped,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestPhases {
    pub ping: PhaseResult,
    pub download: PhaseResult,
    pub upload: PhaseResult,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub status: String,
    pub server: ServerInfo,
    pub ping: Option<f64>,
    pub jitter: Option<f64>,
    pub packet_loss: Option<f64>,
    pub download: Option<f64>,
    pub download_peak: Option<f64>,
    pub download_cv: Option<f64>, // coefficient of variation (0-1) for variance
    pub upload: Option<f64>,
    pub upload_peak: Option<f64>,
    pub upload_cv: Option<f64>, // coefficient of variation (0-1) for variance
    pub download_ci_95: Option<(f64, f64)>, // (lower, upper) 95% CI in Mbps
    pub upload_ci_95: Option<(f64, f64)>, // (lower, upper) 95% CI in Mbps
    pub latency_download: Option<f64>,
    pub latency_upload: Option<f64>,
    pub download_samples: Option<Vec<f64>>,
    pub upload_samples: Option<Vec<f64>>,
    pub ping_samples: Option<Vec<f64>>,
    pub timestamp: String,
    pub client_ip: Option<String>,
    // Computed grades for machine-readable output (screen readers, scripts)
    pub overall_grade: Option<String>,
    pub download_grade: Option<String>,
    pub upload_grade: Option<String>,
    pub connection_rating: Option<String>,
    pub phases: TestPhases,
}

impl TestResult {
    /// Build a `TestResult` from ping test output and download/upload test runs.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_test_runs(
        server: ServerInfo,
        ping: Option<f64>,
        jitter: Option<f64>,
        packet_loss: Option<f64>,
        ping_samples: &[f64],
        dl: &crate::task_runner::TestRunResult,
        ul: &crate::task_runner::TestRunResult,
        client_ip: Option<String>,
    ) -> Self {
        fn opt_samples(v: &[f64]) -> Option<Vec<f64>> {
            if v.is_empty() {
                None
            } else {
                Some(v.to_vec())
            }
        }
        fn opt_positive(v: f64) -> Option<f64> {
            if v > 0.0 {
                Some(v)
            } else {
                None
            }
        }

        Self {
            status: "ok".to_string(),
            server,
            ping,
            jitter,
            packet_loss,
            download: opt_positive(dl.avg_bps),
            download_peak: opt_positive(dl.peak_bps),
            download_cv: compute_cv(&dl.speed_samples),
            upload: opt_positive(ul.avg_bps),
            upload_peak: opt_positive(ul.peak_bps),
            upload_cv: compute_cv(&ul.speed_samples),
            download_ci_95: compute_ci_95(&dl.speed_samples, 1_000_000.0),
            upload_ci_95: compute_ci_95(&ul.speed_samples, 1_000_000.0),
            latency_download: dl.latency_under_load,
            latency_upload: ul.latency_under_load,
            download_samples: opt_samples(&dl.speed_samples),
            upload_samples: opt_samples(&ul.speed_samples),
            ping_samples: opt_samples(ping_samples),
            timestamp: chrono::Utc::now().to_rfc3339(),
            client_ip,
            overall_grade: None,
            download_grade: None,
            upload_grade: None,
            connection_rating: None,
            phases: TestPhases {
                ping: PhaseResult::completed(),
                download: PhaseResult::completed(),
                upload: PhaseResult::completed(),
            },
        }
    }
}

/// Compute coefficient of variation (CV) for a sample set.
/// Returns None for empty or single-element sets, or when mean is zero.
fn compute_cv(samples: &[f64]) -> Option<f64> {
    if samples.len() < 2 {
        return None;
    }
    // Safe: sample counts are small (≤1000), well under 2^53.
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    if mean == 0.0 {
        return None;
    }
    // Safe: sample counts are small (≤1000), well under 2^53.
    let variance: f64 =
        samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
    let std_dev = variance.sqrt();
    Some(std_dev / mean)
}

/// Compute 95% confidence interval for the mean bandwidth.
/// Returns `(lower, upper)` in Mbps. Uses t-distribution approximation for small samples.
fn compute_ci_95(samples: &[f64], scale: f64) -> Option<(f64, f64)> {
    let n = samples.len();
    if n < 2 {
        return None;
    }
    // Safe: sample counts are small (≤1000), well under 2^53.
    let mean: f64 = samples.iter().sum::<f64>() / n as f64;
    if n < 30 {
        // Small sample: use t ≈ 2.045 (df=29, 95% CI) as conservative estimate
        // Safe: n is small (≤1000), well under 2^53.
        let variance: f64 =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        let std_err = variance.sqrt() / (n as f64).sqrt();
        let margin = 2.045 * std_err;
        Some(((mean - margin) / scale, (mean + margin) / scale))
    } else {
        // Large sample: use z = 1.96
        // Safe: n is small (≤1000), well under 2^53.
        let variance: f64 =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        let std_err = variance.sqrt() / (n as f64).sqrt();
        let margin = 1.96 * std_err;
        Some(((mean - margin) / scale, (mean + margin) / scale))
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
            status: "ok".to_string(),
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
            download_cv: None,
            upload_cv: None,
            download_ci_95: None,
            upload_ci_95: None,
            overall_grade: None,
            download_grade: None,
            upload_grade: None,
            connection_rating: None,
            phases: TestPhases {
                ping: PhaseResult::completed(),
                download: PhaseResult::completed(),
                upload: PhaseResult::completed(),
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"ping\":15.234"));
        assert!(json.contains("\"jitter\":1.2"));
        assert!(json.contains("\"download\":150000000.0"));
        assert!(json.contains("\"phases\""));
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

    #[test]
    fn test_phase_result_skipped_serialization() {
        let phase = PhaseResult::skipped("disabled by user");
        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("\"state\":\"skipped\""));
        assert!(json.contains("\"reason\":\"disabled by user\""));
    }
}
