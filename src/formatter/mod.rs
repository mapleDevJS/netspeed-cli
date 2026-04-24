//! Output formatting for speed test results.
//!
//! This module is organized into submodules:
//! - [`ratings`] — Rating helper functions (ping, speed, connection, bufferbloat)
//! - [`sections`] — Output section formatters (latency, download, upload, etc.)
//! - [`stability`] — Speed stability analysis and latency percentiles
//! - [`estimates`] — Usage check targets and download time estimates

use crate::common;
use crate::error::Error;
use crate::grades;
use crate::profiles::UserProfile;
use crate::terminal;
use crate::theme::{Colors, Theme};
use crate::types::{CsvOutput, TestResult};
use owo_colors::OwoColorize;

/// Which test phases were skipped by the user (e.g. `--no-download`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SkipState {
    /// Download test was skipped.
    pub download: bool,
    /// Upload test was skipped.
    pub upload: bool,
}

/// Build a section header with consistent formatting.
fn section_header(title: &str, nc: bool, theme: Theme) -> String {
    if nc {
        format!("  {title}")
    } else {
        format!("  {}", Colors::header(title, theme))
    }
}

/// Output format selection — Strategy pattern.
/// Add new variants here to extend output formats (OCP).
pub enum OutputFormat {
    Json,
    Csv {
        delimiter: char,
        header: bool,
    },
    Simple {
        theme: Theme,
    },
    Minimal {
        theme: Theme,
    },
    Jsonl,
    Compact {
        dl_bytes: u64,
        ul_bytes: u64,
        dl_duration: f64,
        ul_duration: f64,
        elapsed: std::time::Duration,
        profile: UserProfile,
        theme: Theme,
    },
    Detailed {
        dl_bytes: u64,
        ul_bytes: u64,
        dl_duration: f64,
        ul_duration: f64,
        skipped: SkipState,
        elapsed: std::time::Duration,
        profile: UserProfile,
        minimal: bool,
        theme: Theme,
    },
    Dashboard {
        dl_mbps: f64,
        dl_peak_mbps: f64,
        dl_bytes: u64,
        dl_duration: f64,
        ul_mbps: f64,
        ul_peak_mbps: f64,
        ul_bytes: u64,
        ul_duration: f64,
        elapsed: std::time::Duration,
        profile: UserProfile,
        theme: Theme,
    },
}

impl OutputFormat {
    /// Execute the formatting strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if output serialization or writing fails.
    pub fn format(&self, result: &TestResult, bytes: bool) -> Result<(), Error> {
        match self {
            OutputFormat::Json => format_json(result),
            OutputFormat::Jsonl => format_jsonl(result),
            OutputFormat::Csv { delimiter, header } => format_csv(result, *delimiter, *header),
            OutputFormat::Simple { theme } => format_simple(result, bytes, *theme),
            OutputFormat::Minimal { theme } => format_minimal(result, bytes, *theme),
            OutputFormat::Compact {
                dl_bytes,
                ul_bytes,
                dl_duration,
                ul_duration,
                elapsed,
                profile,
                theme,
            } => {
                format_compact(
                    result,
                    bytes,
                    *dl_bytes,
                    *ul_bytes,
                    *dl_duration,
                    *ul_duration,
                    *elapsed,
                    *profile,
                    *theme,
                );
                Ok(())
            }
            OutputFormat::Detailed {
                dl_bytes,
                ul_bytes,
                dl_duration,
                ul_duration,
                skipped,
                elapsed,
                profile,
                minimal,
                theme,
            } => {
                format_detailed(
                    result,
                    bytes,
                    *dl_bytes,
                    *ul_bytes,
                    *dl_duration,
                    *ul_duration,
                    *skipped,
                    *elapsed,
                    *profile,
                    *minimal,
                    *theme,
                )?;
                format_verbose_sections(result, *profile, *minimal, *theme);
                Ok(())
            }
            OutputFormat::Dashboard {
                dl_mbps,
                dl_peak_mbps,
                dl_bytes,
                dl_duration,
                ul_mbps,
                ul_peak_mbps,
                ul_bytes,
                ul_duration,
                elapsed,
                profile,
                theme,
            } => {
                dashboard::show(
                    result,
                    &dashboard::Summary {
                        dl_mbps: *dl_mbps,
                        dl_peak_mbps: *dl_peak_mbps,
                        dl_bytes: *dl_bytes,
                        dl_duration: *dl_duration,
                        ul_mbps: *ul_mbps,
                        ul_peak_mbps: *ul_peak_mbps,
                        ul_bytes: *ul_bytes,
                        ul_duration: *ul_duration,
                        elapsed: *elapsed,
                        profile: *profile,
                        theme: *theme,
                    },
                )?;
                Ok(())
            }
        }
    }
}

/// Trait for output formatting strategies.
///
/// Implement this trait to provide custom output formatters.
/// This enables the Open-Closed Principle: new formatters can be added
/// without modifying existing code that uses formatters.
///
/// # Example
///
/// ```
/// use netspeed_cli::formatter::{Formatter, OutputFormat};
/// use netspeed_cli::types::{Server, TestResult};
/// use netspeed_cli::error::Error;
///
/// struct MyFormatter;
///
/// impl Formatter for MyFormatter {
///     fn format(&self, result: &TestResult, use_bytes: bool) -> Result<(), Error> {
///         println!("Custom: {:?}", result.ping);
///         Ok(())
///     }
///     
///     fn format_list(&self, servers: &[Server]) -> Result<(), Error> {
///         println!("Servers: {}", servers.len());
///         Ok(())
///     }
/// }
/// ```
pub trait Formatter {
    /// Format a test result for output.
    ///
    /// # Errors
    ///
    /// Returns an error if output fails.
    fn format(
        &self,
        result: &crate::types::TestResult,
        use_bytes: bool,
    ) -> Result<(), crate::error::Error>;

    /// Format a list of servers for output.
    ///
    /// # Errors
    ///
    /// Returns an error if output fails.
    fn format_list(&self, servers: &[crate::types::Server]) -> Result<(), crate::error::Error>;
}

/// Allows using `OutputFormat` polymorphically through the trait.
impl Formatter for OutputFormat {
    fn format(
        &self,
        result: &crate::types::TestResult,
        use_bytes: bool,
    ) -> Result<(), crate::error::Error> {
        self.format(result, use_bytes)
    }

    fn format_list(&self, servers: &[crate::types::Server]) -> Result<(), crate::error::Error> {
        sections::format_list(servers).map_err(crate::error::Error::IoError)
    }
}

pub mod bandwidth_dashboard;
pub mod dashboard;
pub mod estimates;
pub mod ratings;
pub mod scenarios;
pub mod sections;
pub mod stability;

// Re-export commonly used functions for backward compatibility
pub use estimates::{format_targets, show};
pub use ratings::{
    bufferbloat_colorized, bufferbloat_grade, colorize_rating, connection_rating, degradation_str,
    format_duration, format_overall_rating, format_speed_colored, format_speed_plain, ping_rating,
    speed_rating_mbps, BufferbloatGrade,
};
pub use sections::{
    build_elapsed_time, format_connection_info, format_download_section, format_elapsed_time,
    format_footer, format_latency_section, format_list, format_test_summary, format_upload_section,
};
pub use stability::{compute_cv, compute_percentiles, format_stability_line};

/// Simple mode — single line.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_simple(result: &TestResult, bytes: bool, theme: Theme) -> Result<(), Error> {
    let nc = terminal::no_color();
    let mut parts = Vec::new();

    if let Some(ping) = result.ping {
        parts.push(if nc {
            format!("Latency: {ping:.1} ms")
        } else {
            format!("Latency: {} ms", Colors::info(&format!("{ping:.1}"), theme))
        });
    }

    if let Some(dl) = result.download {
        let speed = if nc {
            ratings::format_speed_plain(dl, bytes)
        } else {
            ratings::format_speed_colored(dl, bytes, theme)
        };
        parts.push(format!("Download: {speed}"));
    }

    if let Some(ul) = result.upload {
        let speed = if nc {
            ratings::format_speed_plain(ul, bytes)
        } else {
            ratings::format_speed_colored(ul, bytes, theme)
        };
        parts.push(format!("Upload: {speed}"));
    }

    eprintln!("{}", parts.join(" | "));
    Ok(())
}

/// Minimal mode — ultra-compact: just "B+ 150.5↓ 25.3↑ 12ms"
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_minimal(result: &TestResult, _bytes: bool, theme: Theme) -> Result<(), Error> {
    let nc = terminal::no_color();
    let profile = UserProfile::default();

    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        profile,
    );
    let grade_str = if nc {
        format!("[{}]", overall_grade.as_str())
    } else {
        overall_grade.color_str(nc, theme)
    };

    let dl_str = result.download.map_or_else(
        || "—↓".to_string(),
        |d| {
            let mbps = d / 1_000_000.0;
            format!("{mbps:.1}↓")
        },
    );

    let ul_str = result.upload.map_or_else(
        || "—↑".to_string(),
        |u| {
            let mbps = u / 1_000_000.0;
            format!("{mbps:.1}↑")
        },
    );

    let lat_str = result
        .ping
        .map_or_else(|| "—ms".to_string(), |p| format!("{p:.0}ms"));

    eprintln!("{grade_str}  {dl_str}  {ul_str}  {lat_str}");
    Ok(())
}

/// JSONL mode — one JSON object per line, ideal for logging/parsing.
///
/// # Errors
///
/// Returns [`Error::ParseJson`] if serialization fails.
pub fn format_jsonl(result: &TestResult) -> Result<(), Error> {
    println!("{}", serde_json::to_string(result)?);
    Ok(())
}

/// Compact mode — key metrics with ratings and brief summary.
/// Middle ground between simple (too minimal) and detailed (too verbose).
pub fn format_compact(
    result: &TestResult,
    bytes: bool,
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    elapsed: std::time::Duration,
    profile: UserProfile,
    theme: Theme,
) {
    let nc = terminal::no_color();
    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        profile,
    );

    eprintln!();
    eprintln!("{}", section_header("TEST RESULTS", nc, theme));
    eprintln!("{}", ratings::format_overall_rating(result, nc, theme));
    if !nc {
        eprintln!(
            "  {} {}",
            "Grade:".dimmed(),
            overall_grade.color_str(nc, theme)
        );
    }
    eprintln!();

    sections::format_latency_section(result, nc, theme);
    eprintln!();

    sections::format_download_section(result, bytes, nc, false, theme);
    eprintln!();

    sections::format_upload_section(result, bytes, nc, false, theme);
    eprintln!();

    if let Some(ip) = &result.client_ip {
        if nc {
            eprintln!("  Server: {} · {}", result.server.sponsor, ip);
        } else {
            eprintln!(
                "  {} {} · {}",
                "Server:".dimmed(),
                Colors::bold(&result.server.sponsor, theme),
                Colors::muted(ip, theme),
            );
        }
        eprintln!();
    }

    if nc {
        eprintln!("  SUMMARY");
    } else {
        eprintln!("  {}", Colors::header("SUMMARY", theme));
    }
    if dl_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {:.1}s",
            "Download".dimmed(),
            common::format_data_size(dl_bytes),
            dl_duration
        );
    }
    if ul_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {:.1}s",
            "Upload".dimmed(),
            common::format_data_size(ul_bytes),
            ul_duration
        );
    }

    eprintln!();
    if nc {
        eprintln!("  Total time: {:.1}s", elapsed.as_secs_f64());
    } else {
        eprintln!(
            "  {}: {}",
            "Total time".dimmed(),
            Colors::info(&format!("{:.1}s", elapsed.as_secs_f64()), theme),
        );
    }
    sections::format_footer(&result.timestamp, nc, theme);
}

/// Detailed mode — clean key/value pairs.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
#[allow(clippy::too_many_arguments)]
pub fn format_detailed(
    result: &TestResult,
    bytes: bool,
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    skipped: SkipState,
    elapsed: std::time::Duration,
    profile: UserProfile,
    minimal: bool,
    theme: Theme,
) -> Result<(), Error> {
    let nc = terminal::no_color() || minimal;
    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        profile,
    );

    if nc {
        eprintln!("\n  TEST RESULTS");
    } else {
        eprintln!("\n  {}", Colors::header("TEST RESULTS", theme));
    }
    eprintln!("{}", ratings::format_overall_rating(result, nc, theme));
    if !nc {
        eprintln!(
            "  {} {}",
            "Grade:".dimmed(),
            overall_grade.color_str(nc, theme)
        );
    }
    eprintln!();

    sections::format_latency_section(result, nc, theme);
    sections::format_download_section(result, bytes, nc, skipped.download, theme);
    sections::format_upload_section(result, bytes, nc, skipped.upload, theme);
    sections::format_connection_info(result, nc, theme);
    sections::format_test_summary(dl_bytes, ul_bytes, dl_duration, ul_duration, nc);

    eprintln!();
    if nc {
        eprintln!("  Total time: {:.1}s", elapsed.as_secs_f64());
    } else {
        eprintln!(
            "  {}: {}",
            "Total time".dimmed(),
            Colors::info(&format!("{:.1}s", elapsed.as_secs_f64()), theme),
        );
    }

    sections::format_footer(&result.timestamp, nc, theme);

    Ok(())
}

/// Output test results as JSON to stdout.
///
/// # Errors
///
/// Returns [`Error::ParseJson`] if serialization fails.
pub fn format_json(result: &TestResult) -> Result<(), Error> {
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
/// Returns [`Error::Csv`] if CSV serialization fails.
pub fn format_csv(result: &TestResult, delimiter: char, print_header: bool) -> Result<(), Error> {
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
pub fn format_verbose_sections(
    result: &TestResult,
    profile: UserProfile,
    minimal: bool,
    theme: Theme,
) {
    let nc = terminal::no_color() || minimal;

    if profile.show_estimates() {
        let estimates = estimates::build(result.download, nc, theme);
        if !estimates.is_empty() {
            eprintln!("{estimates}");
        }
    }

    if profile.show_stability() {
        if let (Some(dl_s), Some(ul_s)) = (&result.download_samples, &result.upload_samples) {
            let dl_cv = compute_cv(dl_s);
            let ul_cv = compute_cv(ul_s);
            let dl_grade = grades::grade_stability(dl_cv);
            let ul_grade = grades::grade_stability(ul_cv);
            let dl_stability = format_stability_line(dl_cv, nc, theme);
            let ul_stability = format_stability_line(ul_cv, nc, theme);
            eprintln!();
            eprintln!("{}", section_header("STABILITY", nc, theme));
            if nc {
                eprintln!("  {:>14}:   [{dl_stability}]", "Download");
                eprintln!("  {:>14}:   [{ul_stability}]", "Upload");
            } else {
                eprintln!(
                    "  {:>14}:   {} {dl_stability}",
                    "Download".dimmed(),
                    dl_grade.color_str(nc, theme)
                );
                eprintln!(
                    "  {:>14}:   {} {ul_stability}",
                    "Upload".dimmed(),
                    ul_grade.color_str(nc, theme)
                );
            }
        }
    }

    if profile.show_percentiles() {
        if let Some(ref samples) = result.ping_samples {
            if let Some((p50, p95, p99)) = compute_percentiles(samples) {
                eprintln!();
                eprintln!("{}", section_header("LATENCY PERCENTILES", nc, theme));
                let p50_str = format!("{p50:.1} ms");
                let p95_str = format!("{p95:.1} ms");
                let p99_str = format!("{p99:.1} ms");
                if nc {
                    eprintln!("  P50: {p50_str}  P95: {p95_str}  P99: {p99_str}");
                } else {
                    eprintln!(
                        "  {}: {}  {}: {}  {}: {}",
                        "P50".dimmed(),
                        Colors::info(&p50_str, theme),
                        "P95".dimmed(),
                        Colors::warn(&p95_str, theme),
                        "P99".dimmed(),
                        Colors::bad(&p99_str, theme),
                    );
                }
            }
        }
    }

    if profile.show_history() {
        let dl_mbps = result.download.map_or(0.0, |d| d / 1_000_000.0);
        let ul_mbps = result.upload.map_or(0.0, |u| u / 1_000_000.0);
        if let Some(comparison) = crate::history::format_comparison(dl_mbps, ul_mbps, nc) {
            eprintln!();
            eprintln!("  {comparison}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_with_data() {
        use crate::types::{PhaseResult, ServerInfo, TestPhases, TestResult};
        let result = TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
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

        // Just verify it doesn't panic
        let _ = format_simple(&result, false, Theme::Dark);
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

    #[test]
    fn test_format_verbose_sections_integration() {
        use crate::types::{PhaseResult, ServerInfo, TestPhases, TestResult};
        let result = TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
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
            client_location: None,
            download_cv: Some(0.05),
            upload_cv: Some(0.04),
            download_ci_95: Some((140.0, 160.0)),
            upload_ci_95: Some((45.0, 55.0)),
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

        // Exercise the full integration path: targets, estimates, stability,
        // latency percentiles, and history comparison
        format_verbose_sections(&result, UserProfile::default(), false, Theme::Dark);
    }

    #[test]
    fn test_format_verbose_sections_empty() {
        use crate::types::{PhaseResult, ServerInfo, TestPhases, TestResult};
        let result = TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
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

        // Should not panic with all None values
        format_verbose_sections(&result, UserProfile::default(), false, Theme::Dark);
    }
}
