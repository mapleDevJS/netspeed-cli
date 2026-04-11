//! Detailed and verbose output formatters.
//!
//! Implements the text-based output modes (simple, detailed, verbose)
//! that compose sections, ratings, estimates, stability, and history.

use crate::error::SpeedtestError;
use crate::formatter::{estimates, ratings, sections, stability};
use crate::progress::no_color;
use crate::types::{CsvOutput, TestResult};
use owo_colors::OwoColorize;

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
            format!("Latency: {ping:.1} ms")
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
    } else {
        parts.push(if nc {
            "Download: — (skipped)".to_string()
        } else {
            format!("Download: {}", "(skipped)".bright_black())
        });
    }

    if let Some(ul) = result.upload {
        let speed = if nc {
            ratings::format_speed_plain(ul, bytes)
        } else {
            ratings::format_speed_colored(ul, bytes)
        };
        parts.push(format!("Upload: {speed}"));
    } else {
        parts.push(if nc {
            "Upload: — (skipped)".to_string()
        } else {
            format!("Upload: {}", "(skipped)".bright_black())
        });
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
#[allow(clippy::too_many_arguments)]
pub fn format_detailed(
    result: &TestResult,
    bytes: bool,
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    dl_skipped: bool,
    ul_skipped: bool,
) -> Result<(), SpeedtestError> {
    let nc = no_color();

    if nc {
        eprintln!("\n  TEST RESULTS");
    } else {
        eprintln!("\n  {}", "TEST RESULTS".bold().underline());
    }
    eprintln!("{}", ratings::format_overall_rating(result, nc));
    eprintln!();

    eprintln!("{}", sections::section_divider("Latency", nc));
    sections::format_latency_section(result, nc);
    eprintln!();
    eprintln!("{}", sections::section_divider("Download", nc));
    sections::format_download_section(result, bytes, nc, dl_skipped);
    eprintln!();
    eprintln!("{}", sections::section_divider("Upload", nc));
    sections::format_upload_section(result, bytes, nc, ul_skipped);
    eprintln!();
    eprintln!("{}", sections::section_divider("Connection", nc));
    sections::format_connection_info(result, nc);
    eprintln!();
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
        let dl_cv = stability::compute_cv(dl_s);
        let ul_cv = stability::compute_cv(ul_s);
        let dl_stability = stability::format_stability_line(dl_cv, nc);
        let ul_stability = stability::format_stability_line(ul_cv, nc);
        eprintln!();
        eprintln!("  {}", "STABILITY".bold().underline());
        eprintln!("  {:>14}:   {dl_stability}", "Download".dimmed());
        eprintln!("  {:>14}:   {ul_stability}", "Upload".dimmed());
    }

    // Latency percentiles
    if let Some(ref samples) = result.ping_samples {
        if let Some((p50, p95, p99)) = stability::compute_percentiles(samples) {
            eprintln!();
            eprintln!("  {}", "LATENCY PERCENTILES".bold().underline());
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
    if let Some(comparison) = format_history_comparison(dl_mbps, ul_mbps, nc) {
        eprintln!();
        eprintln!("  {comparison}");
    }
}

/// Format historical comparison as a string for display.
fn format_history_comparison(download_mbps: f64, upload_mbps: f64, nc: bool) -> Option<String> {
    let history = crate::history::load_history().ok()?;
    let recent: Vec<_> = history.iter().rev().take(20).collect();
    let dl_entries: Vec<f64> = recent
        .iter()
        .filter_map(|e| e.download.map(|d| d / 1_000_000.0))
        .collect();
    let ul_entries: Vec<f64> = recent
        .iter()
        .filter_map(|e| e.upload.map(|u| u / 1_000_000.0))
        .collect();

    if dl_entries.is_empty() || ul_entries.is_empty() {
        return None;
    }

    let avg_dl = dl_entries.iter().sum::<f64>() / dl_entries.len() as f64;
    let avg_ul = ul_entries.iter().sum::<f64>() / ul_entries.len() as f64;

    let current_score = download_mbps + upload_mbps;
    let avg_score = avg_dl + avg_ul;

    if avg_score <= 0.0 {
        return None;
    }

    let pct_change = ((current_score / avg_score) - 1.0) * 100.0;

    let display = if pct_change.abs() < 3.0 {
        if nc {
            "~ On par with your average".to_string()
        } else {
            "~ On par with your average".bright_black().to_string()
        }
    } else if pct_change > 0.0 {
        if nc {
            format!("↑ {pct_change:.0}% faster than your average")
        } else {
            format!("↑ {pct_change:.0}% faster than your average")
                .green()
                .to_string()
        }
    } else {
        let abs_pct = pct_change.abs();
        if nc {
            format!("↓ {abs_pct:.0}% slower than your average")
        } else {
            format!("↓ {abs_pct:.0}% slower than your average")
                .red()
                .to_string()
        }
    };

    Some(display)
}
