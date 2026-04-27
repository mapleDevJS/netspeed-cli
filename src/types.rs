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
    pub version: String, // CLI version for API compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>, // Unique identifier for this test run
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_location: Option<ClientLocation>,
    // Computed grades for machine-readable output (screen readers, scripts)
    pub overall_grade: Option<String>,
    pub download_grade: Option<String>,
    pub upload_grade: Option<String>,
    pub connection_rating: Option<String>,
    pub phases: TestPhases,
}

/// Builder for constructing `TestResult` — separates construction from data types (SRP).
///
/// Uses injected `StatsService` for statistical computations,
/// enabling alternative algorithms without modifying `TestResult`.
pub struct TestResultBuilder {
    server: ServerInfo,
    ping: Option<f64>,
    jitter: Option<f64>,
    packet_loss: Option<f64>,
    ping_samples: Vec<f64>,
    download_avg_bps: f64,
    download_peak_bps: f64,
    download_samples: Vec<f64>,
    download_latency: Option<f64>,
    upload_avg_bps: f64,
    upload_peak_bps: f64,
    upload_samples: Vec<f64>,
    upload_latency: Option<f64>,
    client_ip: Option<String>,
    client_location: Option<ClientLocation>,
}

impl TestResultBuilder {
    pub fn new(server: ServerInfo) -> Self {
        Self {
            server,
            ping: None,
            jitter: None,
            packet_loss: None,
            ping_samples: Vec::new(),
            download_avg_bps: 0.0,
            download_peak_bps: 0.0,
            download_samples: Vec::new(),
            download_latency: None,
            upload_avg_bps: 0.0,
            upload_peak_bps: 0.0,
            upload_samples: Vec::new(),
            upload_latency: None,
            client_ip: None,
            client_location: None,
        }
    }

    pub fn ping(mut self, ping: f64) -> Self {
        self.ping = Some(ping);
        self
    }

    pub fn jitter(mut self, jitter: f64) -> Self {
        self.jitter = Some(jitter);
        self
    }

    pub fn packet_loss(mut self, loss: f64) -> Self {
        self.packet_loss = Some(loss);
        self
    }

    pub fn ping_samples(mut self, samples: &[f64]) -> Self {
        self.ping_samples = samples.to_vec();
        self
    }

    pub fn download_stats(
        mut self,
        avg_bps: f64,
        peak_bps: f64,
        samples: &[f64],
        latency_under_load: Option<f64>,
    ) -> Self {
        self.download_avg_bps = avg_bps;
        self.download_peak_bps = peak_bps;
        self.download_samples = samples.to_vec();
        self.download_latency = latency_under_load;
        self
    }

    pub fn upload_stats(
        mut self,
        avg_bps: f64,
        peak_bps: f64,
        samples: &[f64],
        latency_under_load: Option<f64>,
    ) -> Self {
        self.upload_avg_bps = avg_bps;
        self.upload_peak_bps = peak_bps;
        self.upload_samples = samples.to_vec();
        self.upload_latency = latency_under_load;
        self
    }

    pub fn client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    pub fn client_location(mut self, location: ClientLocation) -> Self {
        self.client_location = Some(location);
        self
    }

    /// Build the `TestResult` using the given stats service for CV/CI computation.
    pub fn build(self, stats: &dyn StatsService) -> TestResult {
        fn opt_samples(v: &[f64]) -> Option<Vec<f64>> {
            if v.is_empty() { None } else { Some(v.to_vec()) }
        }
        fn opt_positive(v: f64) -> Option<f64> {
            if v > 0.0 { Some(v) } else { None }
        }

        TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: Some(uuid_v4()),
            server: self.server,
            ping: self.ping,
            jitter: self.jitter,
            packet_loss: self.packet_loss,
            download: opt_positive(self.download_avg_bps),
            download_peak: opt_positive(self.download_peak_bps),
            download_cv: stats.coefficient_of_variation(&self.download_samples),
            upload: opt_positive(self.upload_avg_bps),
            upload_peak: opt_positive(self.upload_peak_bps),
            upload_cv: stats.coefficient_of_variation(&self.upload_samples),
            download_ci_95: stats.confidence_interval_95(&self.download_samples, 1_000_000.0),
            upload_ci_95: stats.confidence_interval_95(&self.upload_samples, 1_000_000.0),
            latency_download: self.download_latency,
            latency_upload: self.upload_latency,
            download_samples: opt_samples(&self.download_samples),
            upload_samples: opt_samples(&self.upload_samples),
            ping_samples: opt_samples(&self.ping_samples),
            timestamp: chrono::Utc::now().to_rfc3339(),
            client_ip: self.client_ip,
            client_location: self.client_location,
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

impl TestResult {
    /// Build a `TestResult` from ping test output and download/upload test runs.
    /// Delegates to `TestResultBuilder` for construction.
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
        client_location: Option<ClientLocation>,
    ) -> Self {
        let mut builder = TestResultBuilder::new(server)
            .ping_samples(ping_samples)
            .download_stats(
                dl.avg_bps,
                dl.peak_bps,
                &dl.speed_samples,
                dl.latency_under_load,
            )
            .upload_stats(
                ul.avg_bps,
                ul.peak_bps,
                &ul.speed_samples,
                ul.latency_under_load,
            );

        if let Some(p) = ping {
            builder = builder.ping(p);
        }
        if let Some(j) = jitter {
            builder = builder.jitter(j);
        }
        if let Some(pl) = packet_loss {
            builder = builder.packet_loss(pl);
        }
        if let Some(ip) = client_ip {
            builder = builder.client_ip(ip);
        }
        if let Some(loc) = client_location {
            builder = builder.client_location(loc);
        }

        builder.build(&DefaultStats)
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

/// Trait for statistical computations - enables alternative algorithms.
pub trait StatsService: Send + Sync {
    fn coefficient_of_variation(&self, samples: &[f64]) -> Option<f64>;
    fn confidence_interval_95(&self, samples: &[f64], scale: f64) -> Option<(f64, f64)>;
}

/// Default statistics implementation using standard formulas.
pub struct DefaultStats;

impl DefaultStats {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultStats {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsService for DefaultStats {
    fn coefficient_of_variation(&self, samples: &[f64]) -> Option<f64> {
        compute_cv(samples)
    }

    fn confidence_interval_95(&self, samples: &[f64], scale: f64) -> Option<(f64, f64)> {
        compute_ci_95(samples, scale)
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

/// Client geographic location derived from speedtest.net config API.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ClientLocation {
    pub lat: f64,
    pub lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
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
#[allow(clippy::items_after_test_module)]
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
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
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
            client_location: None,
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

    #[test]
    fn test_uuid_v4_format() {
        let id = uuid_v4();
        // UUID v4 format: 8-4-4-4-12 hex characters
        assert_eq!(id.len(), 36);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
        assert_eq!(&id[14..15], "4"); // Version 4 marker
    }

    #[test]
    fn test_uuid_v4_unique() {
        let id1 = uuid_v4();
        let id2 = uuid_v4();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_client_location_serialization() {
        let loc = ClientLocation {
            lat: 40.7128,
            lon: -74.0060,
            city: Some("New York".to_string()),
            country: Some("US".to_string()),
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("\"lat\":40.7128"));
        assert!(json.contains("\"lon\":-74.006"));
        assert!(json.contains("\"city\":\"New York\""));
    }

    #[test]
    fn test_client_location_minimal() {
        let loc = ClientLocation {
            lat: 0.0,
            lon: 0.0,
            city: None,
            country: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(!json.contains("city"));
        assert!(!json.contains("country"));
    }

    #[test]
    fn test_result_builder_full() {
        let server = ServerInfo {
            id: "1".to_string(),
            name: "Server".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            distance: 50.0,
        };
        let result = TestResultBuilder::new(server)
            .ping(10.0)
            .jitter(2.0)
            .packet_loss(0.5)
            .ping_samples(&[10.0, 12.0])
            .download_stats(
                100_000_000.0,
                120_000_000.0,
                &[90_000_000.0, 110_000_000.0],
                Some(25.0),
            )
            .upload_stats(
                50_000_000.0,
                60_000_000.0,
                &[45_000_000.0, 55_000_000.0],
                Some(30.0),
            )
            .client_ip("1.2.3.4")
            .client_location(ClientLocation {
                lat: 40.0,
                lon: -74.0,
                city: Some("NYC".to_string()),
                country: Some("US".to_string()),
            })
            .build(&DefaultStats);

        assert_eq!(result.status, "ok");
        assert_eq!(result.ping, Some(10.0));
        assert_eq!(result.jitter, Some(2.0));
        assert_eq!(result.packet_loss, Some(0.5));
        assert_eq!(result.download, Some(100_000_000.0));
        assert_eq!(result.download_peak, Some(120_000_000.0));
        assert_eq!(result.upload, Some(50_000_000.0));
        assert_eq!(result.client_ip, Some("1.2.3.4".to_string()));
        assert!(result.download_samples.is_some());
        assert!(result.ping_samples.is_some());
        assert!(result.download_cv.is_some());
        assert!(result.download_ci_95.is_some());
        assert!(result.test_id.is_some());
    }

    #[test]
    fn test_result_builder_minimal() {
        let server = ServerInfo {
            id: "0".to_string(),
            name: "".to_string(),
            sponsor: "".to_string(),
            country: "".to_string(),
            distance: 0.0,
        };
        let result = TestResultBuilder::new(server).build(&DefaultStats);

        assert_eq!(result.status, "ok");
        assert!(result.ping.is_none());
        assert!(result.jitter.is_none());
        assert!(result.download.is_none());
        assert!(result.upload.is_none());
        assert!(result.client_ip.is_none());
        assert!(result.download_samples.is_none());
        assert!(result.ping_samples.is_none());
        assert!(result.download_cv.is_none());
        assert!(result.upload_cv.is_none());
    }

    #[test]
    fn test_stats_service_cv_known_data() {
        let stats = DefaultStats::new();
        let cv = stats.coefficient_of_variation(&[10.0, 12.0]);
        assert!(cv.is_some());
        let val = cv.unwrap();
        assert!(val > 0.0);
    }

    #[test]
    fn test_stats_service_cv_empty() {
        let stats = DefaultStats::new();
        assert!(stats.coefficient_of_variation(&[]).is_none());
    }

    #[test]
    fn test_stats_service_cv_single() {
        let stats = DefaultStats::new();
        assert!(stats.coefficient_of_variation(&[42.0]).is_none());
    }

    #[test]
    fn test_stats_service_cv_zero_mean() {
        let stats = DefaultStats::new();
        assert!(stats.coefficient_of_variation(&[0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_stats_service_ci95_known_data() {
        let stats = DefaultStats::new();
        let ci = stats.confidence_interval_95(&[100.0, 102.0, 98.0, 101.0, 99.0], 1.0);
        assert!(ci.is_some());
        let (lo, hi) = ci.unwrap();
        assert!(lo < hi);
        let mid = (lo + hi) / 2.0;
        assert!((mid - 100.0).abs() < 2.0);
    }

    #[test]
    fn test_stats_service_ci95_too_few_samples() {
        let stats = DefaultStats::new();
        assert!(stats.confidence_interval_95(&[50.0], 1.0).is_none());
    }
}

/// Generate a simple UUID v4-like identifier using timestamp and random bytes.
/// This is not a standards-compliant UUID but provides uniqueness for test tracking.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random: u64 = rand_simple();
    // Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (36 chars)
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (timestamp as u64) & 0xFFFFFFFF,
        ((random >> 48) & 0xFFFF) as u16,
        ((random >> 32) & 0xFFF) as u16,
        ((random >> 16) & 0xFFFF) as u16,
        random & 0xFFFFFFFFFFFF
    )
}

/// Simple pseudo-random number generator based on xorshift.
/// Not cryptographically secure, but sufficient for test ID generation.
fn rand_simple() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0x123456789ABCDEF0);
    let mut state = STATE.load(Ordering::Relaxed);
    if state == 0 {
        state = 0x123456789ABCDEF0;
    }
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    STATE.store(state, Ordering::Relaxed);
    state
}
