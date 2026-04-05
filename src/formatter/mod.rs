//! Output formatting for speed test results.
//!
//! This module is organized into submodules:
//! - [`ratings`] — Rating helper functions (ping, speed, connection, bufferbloat)
//! - [`sections`] — Output section formatters (latency, download, upload, etc.)
//! - [`stability`] — Speed stability analysis and latency percentiles
//! - [`estimates`] — Usage check targets and download time estimates

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::error::SpeedtestError;
use crate::progress::no_color;
use crate::types::{CsvOutput, TestResult};
use owo_colors::OwoColorize;

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
        dl_bytes: u64,
        ul_bytes: u64,
        dl_duration: f64,
        ul_duration: f64,
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
            OutputFormat::Json => format_json(result),
            OutputFormat::Csv { delimiter, header } => format_csv(result, *delimiter, *header),
            OutputFormat::Simple => format_simple(result, bytes),
            OutputFormat::Detailed {
                dl_bytes,
                ul_bytes,
                dl_duration,
                ul_duration,
            } => {
                format_detailed(
                    result,
                    bytes,
                    *dl_bytes,
                    *ul_bytes,
                    *dl_duration,
                    *ul_duration,
                )?;
                format_verbose_sections(result);
                Ok(())
            }
        }
    }
}

pub mod estimates;
pub mod ratings;
pub mod sections;
pub mod stability;

// Re-export commonly used functions for backward compatibility
pub use estimates::{format_estimates, format_targets};
pub use ratings::{
    bufferbloat_colorized, bufferbloat_grade, colorize_rating, connection_rating, degradation_str,
    format_duration, format_overall_rating, format_speed_colored, format_speed_plain, ping_rating,
    speed_rating_mbps, BufferbloatGrade,
};
pub use sections::{
    format_connection_info, format_download_section, format_footer, format_latency_section,
    format_list, format_test_summary, format_upload_section,
};
pub use stability::{compute_cv, compute_percentiles, format_stability_line};

/// Simple mode — single line.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_simple(result: &TestResult, bytes: bool) -> Result<(), SpeedtestError> {
    let nc = no_color();
    let mut parts = Vec::new();

    if let Some(ping) = result.ping {
        parts.push(if nc {
            format!("{ping:.1} ms")
        } else {
            format!("Latency: {} ms", ping.cyan())
        });
    }

    if let Some(dl) = result.download {
        let speed = if nc {
            ratings::format_speed_plain(dl, bytes)
        } else {
            ratings::format_speed_colored(dl, bytes)
        };
        parts.push(format!("Download: {speed}"));
    }

    if let Some(ul) = result.upload {
        let speed = if nc {
            ratings::format_speed_plain(ul, bytes)
        } else {
            ratings::format_speed_colored(ul, bytes)
        };
        parts.push(format!("Upload: {speed}"));
    }

    eprintln!("{}", parts.join(" | "));
    Ok(())
}

/// Detailed mode — clean key/value pairs.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_detailed(
    result: &TestResult,
    bytes: bool,
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
) -> Result<(), SpeedtestError> {
    let nc = no_color();

    if nc {
        eprintln!("\n  TEST RESULTS");
    } else {
        eprintln!("\n  {}", "TEST RESULTS".bold().underline());
    }
    eprintln!("{}", ratings::format_overall_rating(result, nc));
    eprintln!();

    sections::format_latency_section(result, nc);
    sections::format_download_section(result, bytes, nc);
    sections::format_upload_section(result, bytes, nc);
    sections::format_connection_info(result, nc);
    sections::format_test_summary(dl_bytes, ul_bytes, dl_duration, ul_duration, nc);
    sections::format_footer(&result.timestamp, nc);

    Ok(())
}

/// Output test results as JSON to stdout.
///
/// # Errors
///
/// Returns [`SpeedtestError::ParseJson`] if serialization fails.
pub fn format_json(result: &TestResult) -> Result<(), SpeedtestError> {
    let is_tty = {
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
    };
    let output = if is_tty {
        serde_json::to_string_pretty(result)?
    } else {
        serde_json::to_string(result)?
    };
    println!("{output}");
    Ok(())
}

/// Output test results as CSV to stdout.
///
/// # Errors
///
/// Returns [`SpeedtestError::Csv`] if CSV serialization fails.
pub fn format_csv(
    result: &TestResult,
    delimiter: char,
    print_header: bool,
) -> Result<(), SpeedtestError> {
    let stdout = std::io::stdout();
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter as u8)
        .from_writer(stdout);
    if print_header {
        wtr.write_record([
            "Server ID",
            "Sponsor",
            "Server Name",
            "Timestamp",
            "Distance",
            "Ping",
            "Jitter",
            "Packet Loss",
            "Download",
            "Download Peak",
            "Upload",
            "Upload Peak",
            "IP Address",
        ])?;
    }
    let csv_output = CsvOutput {
        server_id: result.server.id.clone(),
        sponsor: result.server.sponsor.clone(),
        server_name: result.server.name.clone(),
        timestamp: result.timestamp.clone(),
        distance: result.server.distance,
        ping: result.ping.unwrap_or(0.0),
        jitter: result.jitter.unwrap_or(0.0),
        packet_loss: result.packet_loss.unwrap_or(0.0),
        download: result.download.unwrap_or(0.0),
        download_peak: result.download_peak.unwrap_or(0.0),
        upload: result.upload.unwrap_or(0.0),
        upload_peak: result.upload_peak.unwrap_or(0.0),
        ip_address: result.client_ip.clone().unwrap_or_default(),
    };
    wtr.serialize(csv_output)?;
    wtr.flush()?;
    Ok(())
}

/// Format additional verbose output sections: stability, latency percentiles, and historical comparison.
/// Only used in detailed (verbose) mode.
pub fn format_verbose_sections(result: &TestResult) {
    let nc = no_color();

    // Usage check targets
    let targets = estimates::build_targets(result.download, nc);
    if !targets.is_empty() {
        eprintln!("{targets}");
    }

    // Download time estimates
    let estimates = estimates::build_estimates(result.download, nc);
    if !estimates.is_empty() {
        eprintln!("{estimates}");
    }

    // Speed stability (DL + UL)
    if let (Some(dl_s), Some(ul_s)) = (&result.download_samples, &result.upload_samples) {
        let dl_cv = compute_cv(dl_s);
        let ul_cv = compute_cv(ul_s);
        let dl_stability = format_stability_line(dl_cv, nc);
        let ul_stability = format_stability_line(ul_cv, nc);
        eprintln!();
        if nc {
            eprintln!("  STABILITY");
        } else {
            eprintln!("\n  {}", "STABILITY".bold().underline());
        }
        eprintln!("  {:>14}:   {dl_stability}", "Download".dimmed());
        eprintln!("  {:>14}:   {ul_stability}", "Upload".dimmed());
    }

    // Latency percentiles
    if let Some(ref samples) = result.ping_samples {
        if let Some((p50, p95, p99)) = compute_percentiles(samples) {
            eprintln!();
            if nc {
                eprintln!("  LATENCY PERCENTILES");
            } else {
                eprintln!("\n  {}", "LATENCY PERCENTILES".bold().underline());
            }
            let p50_str = format!("{p50:.1} ms");
            let p95_str = format!("{p95:.1} ms");
            let p99_str = format!("{p99:.1} ms");
            if nc {
                eprintln!("  P50: {p50_str}  P95: {p95_str}  P99: {p99_str}");
            } else {
                eprintln!(
                    "  {}: {}  {}: {}  {}: {}",
                    "P50".dimmed(),
                    p50_str.cyan(),
                    "P95".dimmed(),
                    p95_str.yellow(),
                    "P99".dimmed(),
                    p99_str.red().bold(),
                );
            }
        }
    }

    // Historical comparison
    let dl_mbps = result.download.map(|d| d / 1_000_000.0).unwrap_or(0.0);
    let ul_mbps = result.upload.map(|u| u / 1_000_000.0).unwrap_or(0.0);
    if let Some(comparison) = crate::history::format_comparison(dl_mbps, ul_mbps, nc) {
        eprintln!();
        eprintln!("  {comparison}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_with_data() {
        use crate::types::{ServerInfo, TestResult};
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
        let _ = format_simple(&result, false);
    }

    #[test]
    fn test_format_data_kb() {
        assert_eq!(crate::common::format_data_size(5120), "5.0 KB");
    }

    #[test]
    fn test_format_data_mb() {
        assert_eq!(crate::common::format_data_size(5_242_880), "5.0 MB");
    }

    #[test]
    fn test_format_data_gb() {
        assert_eq!(crate::common::format_data_size(1_073_741_824), "1.00 GB");
    }
}
