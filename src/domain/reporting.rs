//! Result assembly and grading.
//!
//! This module handles building test results from individual test runs
//! and computing grades/ratings.

use crate::grades;
use crate::profiles::UserProfile;
use crate::task_runner::TestRunResult;
use crate::types::{ClientLocation, PhaseResult, ServerInfo, TestPhases, TestResult};

/// Alias for a full test result that can be persisted as a report.
pub type Report = TestResult;

/// Builder for constructing [`TestResult`] from test runs.
pub struct TestResultBuilder {
    server: ServerInfo,
    ping: Option<f64>,
    jitter: Option<f64>,
    packet_loss: Option<f64>,
    ping_samples: Vec<f64>,
    download_result: Option<TestRunResult>,
    upload_result: Option<TestRunResult>,
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
            download_result: None,
            upload_result: None,
            client_ip: None,
            client_location: None,
        }
    }

    pub fn with_ping(
        mut self,
        latency: f64,
        jitter: f64,
        packet_loss: f64,
        samples: Vec<f64>,
    ) -> Self {
        self.ping = Some(latency);
        self.jitter = Some(jitter);
        self.packet_loss = Some(packet_loss);
        self.ping_samples = samples;
        self
    }

    pub fn with_download(mut self, result: TestRunResult) -> Self {
        self.download_result = Some(result);
        self
    }

    pub fn with_upload(mut self, result: TestRunResult) -> Self {
        self.upload_result = Some(result);
        self
    }

    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_client_location(mut self, location: ClientLocation) -> Self {
        self.client_location = Some(location);
        self
    }

    pub fn build(self) -> TestResult {
        let default_dl = TestRunResult::default();
        let default_ul = TestRunResult::default();
        let dl = self.download_result.as_ref().unwrap_or(&default_dl);
        let ul = self.upload_result.as_ref().unwrap_or(&default_ul);

        TestResult::from_test_runs(
            self.server,
            self.ping,
            self.jitter,
            self.packet_loss,
            &self.ping_samples,
            dl,
            ul,
            self.client_ip,
            self.client_location,
        )
    }
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            status: String::new(),
            version: String::new(),
            test_id: None,
            server: ServerInfo {
                id: String::new(),
                name: String::new(),
                sponsor: String::new(),
                country: String::new(),
                distance: 0.0,
            },
            ping: None,
            jitter: None,
            packet_loss: None,
            download: None,
            download_peak: None,
            download_cv: None,
            upload: None,
            upload_peak: None,
            upload_cv: None,
            download_ci_95: None,
            upload_ci_95: None,
            latency_download: None,
            latency_upload: None,
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: String::new(),
            client_ip: None,
            client_location: None,
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

pub fn compute_overall_grade(
    ping: Option<f64>,
    jitter: Option<f64>,
    download: Option<f64>,
    upload: Option<f64>,
    profile: UserProfile,
) -> String {
    grades::grade_overall(ping, jitter, download, upload, profile)
        .as_str()
        .to_string()
}

pub fn compute_download_grade(download_mbps: f64, profile: UserProfile) -> String {
    grades::grade_download(download_mbps, profile)
        .as_str()
        .to_string()
}

pub fn compute_upload_grade(upload_mbps: f64, profile: UserProfile) -> String {
    grades::grade_upload(upload_mbps, profile)
        .as_str()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_builder_default() {
        let result = TestResult::default();
        assert!(result.server.id.is_empty());
    }

    #[test]
    fn test_result_builder_basic() {
        let server = ServerInfo {
            id: "123".to_string(),
            name: "Test Server".to_string(),
            sponsor: "Test ISP".to_string(),
            country: "US".to_string(),
            distance: 100.0,
        };

        let builder = TestResultBuilder::new(server);
        let result = builder.build();

        assert_eq!(result.server.id, "123");
        assert_eq!(result.status, "ok");
    }
}
