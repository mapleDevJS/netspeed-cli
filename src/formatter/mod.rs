//! Output formatting for speed test results.
//!
//! This module is organized into submodules:
//! - [`detailed`] — Simple, detailed, JSON, CSV, and verbose output implementations
//! - [`dashboard`] — Rich boxed layout with bar charts and sparkline history
//! - [`formatting`] — Formatting primitives (distance, data size, bar charts)
//! - [`ratings`] — Rating helper functions (ping, speed, connection, bufferbloat)
//! - [`sections`] — Output section formatters (latency, download, upload, etc.)
//! - [`stability`] — Speed stability analysis and latency percentiles
//! - [`estimates`] — Usage check targets and download time estimates

use crate::error::SpeedtestError;
use crate::test_runner::TestRunResult;
use crate::types::TestResult;

/// Output format selection — Strategy pattern.
/// Add new variants here to extend output formats (OCP).
pub enum OutputFormat {
    Json,
    Csv {
        delimiter: char,
        header: bool,
    },
    Simple,
    Detailed {
        dl: TestRunResult,
        ul: TestRunResult,
    },
    Dashboard {
        dl: TestRunResult,
        ul: TestRunResult,
        history_data: dashboard::HistoryData,
    },
}

impl OutputFormat {
    /// Execute the formatting strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if output serialization or writing fails.
    pub fn format(&self, result: &TestResult, bytes: bool) -> Result<(), SpeedtestError> {
        match self {
            OutputFormat::Json => detailed::format_json(result),
            OutputFormat::Csv { delimiter, header } => {
                detailed::format_csv(result, *delimiter, *header)
            }
            OutputFormat::Simple => detailed::format_simple(result, bytes),
            OutputFormat::Detailed { dl, ul } => {
                detailed::format_detailed(
                    result,
                    bytes,
                    dl.total_bytes,
                    ul.total_bytes,
                    dl.duration_secs,
                    ul.duration_secs,
                    dl.is_skipped(),
                    ul.is_skipped(),
                )?;
                detailed::format_verbose_sections(result);
                Ok(())
            }
            OutputFormat::Dashboard {
                dl,
                ul,
                history_data,
            } => {
                dashboard::format_dashboard(result, dl, ul, history_data, bytes)?;
                Ok(())
            }
        }
    }
}

pub mod dashboard;
pub mod detailed;
pub mod estimates;
pub mod formatting;
pub mod ratings;
pub mod sections;
pub mod stability;

// Re-export commonly used functions for backward compatibility
pub use estimates::{format_estimates, format_targets};
pub use ratings::{
    BufferbloatGrade, bufferbloat_colorized, bufferbloat_grade, colorize_rating, connection_rating,
    degradation_str, format_duration, format_overall_rating, format_speed_colored,
    format_speed_plain, ping_rating, speed_rating_mbps,
};
pub use sections::{
    format_connection_info, format_download_section, format_footer, format_latency_section,
    format_list, format_test_summary, format_upload_section,
};
pub use stability::{compute_cv, compute_percentiles, format_stability_line};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_with_data() {
        use crate::types::ServerInfo;
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: Some(10.0),
            jitter: None,
            packet_loss: None,
            download: Some(100_000_000.0),
            download_peak: None,
            upload: Some(50_000_000.0),
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };

        // Just verify it doesn't panic
        let _ = detailed::format_simple(&result, false);
    }

    #[test]
    fn test_format_data_kb() {
        assert_eq!(
            crate::formatter::formatting::format_data_size(5120),
            "5.0 KB"
        );
    }

    #[test]
    fn test_format_data_mb() {
        assert_eq!(
            crate::formatter::formatting::format_data_size(5_242_880),
            "5.0 MB"
        );
    }

    #[test]
    fn test_format_data_gb() {
        assert_eq!(
            crate::formatter::formatting::format_data_size(1_073_741_824),
            "1.00 GB"
        );
    }

    #[test]
    fn test_format_verbose_sections_integration() {
        use crate::types::ServerInfo;
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 10.0,
            },
            ping: Some(10.0),
            jitter: Some(1.5),
            packet_loss: Some(0.0),
            download: Some(100_000_000.0),
            download_peak: Some(120_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            latency_download: Some(15.0),
            latency_upload: Some(12.0),
            download_samples: Some(vec![95_000_000.0, 100_000_000.0, 105_000_000.0]),
            upload_samples: Some(vec![48_000_000.0, 50_000_000.0, 52_000_000.0]),
            ping_samples: Some(vec![9.5, 10.0, 10.5]),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            client_ip: Some("192.168.1.1".to_string()),
        };

        // Exercise the full integration path: targets, estimates, stability,
        // latency percentiles, and history comparison
        detailed::format_verbose_sections(&result);
    }

    #[test]
    fn test_format_verbose_sections_empty() {
        use crate::types::ServerInfo;
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: None,
            jitter: None,
            packet_loss: None,
            download: None,
            download_peak: None,
            upload: None,
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };

        // Should not panic with all None values
        detailed::format_verbose_sections(&result);
    }
}
