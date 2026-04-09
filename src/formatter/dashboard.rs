//! Dashboard output format — rich boxed layout with bar charts, history sparkline, and key hints.
//!
//! This module provides a visually rich single-screen output suitable for
//! terminal display. It uses only existing dependencies (`owo_colors`, `common`,
//! `history`, `ratings`) — no new crates.

use crate::formatter::formatting::{bar_chart, format_data_size, format_distance};
use crate::formatter::ratings;
use crate::history;
use crate::progress::no_color;
use crate::test_runner::TestRunResult;
use crate::types::TestResult;
use owo_colors::OwoColorize;

/// Pre-loaded history data for dashboard sparkline rendering.
/// Each entry: (date_string, download_mbps, upload_mbps).
pub type HistoryData = Vec<(String, f64, f64)>;

const BOX_WIDTH: usize = 60;
const BAR_WIDTH: usize = 28;

/// Render a horizontal bar scaled to the metric's typical range.
fn metric_bar(value: f64, max: f64, width: usize, nc: bool) -> String {
    let bar = bar_chart(value, max, width);
    if nc {
        bar
    } else {
        let fill_pct = (value / max).clamp(0.0, 1.0) * 100.0;
        if fill_pct >= 70.0 {
            bar.green().to_string()
        } else if fill_pct >= 40.0 {
            bar.yellow().to_string()
        } else {
            bar.red().to_string()
        }
    }
}

/// Build a section separator line.
fn section_divider(title: &str, nc: bool) -> String {
    let title_with_spaces = format!(" {title} ");
    let dash_count = BOX_WIDTH.saturating_sub(title_with_spaces.len() + 4);
    let dashes = "─".repeat(dash_count);
    if nc {
        format!("  {title_with_spaces}{dashes}")
    } else {
        format!("  {}", title_with_spaces.dimmed()) + &dashes.dimmed().to_string()
    }
}

/// Build the boxed header with version, server, and client IP.
fn build_header(result: &TestResult, nc: bool) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let title = format!(" netspeed-cli v{version} ");
    let half_pad = (BOX_WIDTH.saturating_sub(title.len())) / 2;
    let left_pad = "═".repeat(half_pad);
    let right_pad = "═".repeat(BOX_WIDTH.saturating_sub(half_pad + title.len()));
    let title_line = format!("{left_pad}{title}{right_pad}");

    let server_line = format!(
        "  Server: {} ({}) · {} · {}",
        result.server.sponsor,
        result.server.name,
        result.server.country,
        format_distance(result.server.distance)
    );

    let ip_line = result
        .client_ip
        .as_ref()
        .map(|ip| format!("  Client IP: {ip}"));

    let mut lines = Vec::new();

    // Top border
    if nc {
        lines.push(format!("╔{title_line}╗"));
    } else {
        lines.push(format!("╔{title_line}╗").dimmed().to_string());
    }

    // Server info
    let padded_server = format!("{server_line:<BOX_WIDTH$}");
    if nc {
        lines.push(format!("║{padded_server}║"));
    } else {
        lines.push(format!("║{padded_server}║").dimmed().to_string());
    }

    // Client IP (if available)
    if let Some(ip) = ip_line {
        let padded_ip = format!("{ip:<BOX_WIDTH$}");
        if nc {
            lines.push(format!("║{padded_ip}║"));
        } else {
            lines.push(format!("║{padded_ip}║").dimmed().to_string());
        }
    }

    // Bottom border
    if nc {
        lines.push(format!("╚{:═<BOX_WIDTH$}╝", ""));
    } else {
        lines.push(format!("╚{:═<BOX_WIDTH$}╝", "").dimmed().to_string());
    }

    lines.join("\n")
}

/// Build the overall connection rating line.
fn build_overall_rating(result: &TestResult, nc: bool) -> String {
    let rating = ratings::connection_rating(result);
    if nc {
        format!("  Overall: {rating}")
    } else {
        let rating_colored = ratings::colorize_rating(rating, nc);
        format!("  {} {rating_colored}", "Overall:".dimmed())
    }
}

/// Build metric lines with bar charts for latency, download, and upload.
fn build_metric_bars(result: &TestResult, nc: bool) -> String {
    let mut lines = Vec::new();

    // Latency bar (0–100 ms scale, direct: shorter bar = lower ping = better)
    if let Some(ping) = result.ping {
        let rating = ratings::ping_rating(ping);
        // Direct scale: 0ms = empty bar, 100ms = full bar
        let bar = metric_bar(ping, 100.0, BAR_WIDTH, nc);
        if nc {
            lines.push(format!(
                "  {:<14} {}  {:>8.1} ms  ({rating})",
                "Latency", bar, ping
            ));
        } else {
            let ping_str = format!("{ping:.1} ms");
            let rating_str = ratings::colorize_rating(rating, nc);
            lines.push(format!(
                "  {:<14} {}  {}  {}",
                "Latency".dimmed(),
                bar,
                ping_str.cyan().bold(),
                rating_str,
            ));
        }
    }

    // Download bar (0–1000 Mbps scale)
    if let Some(dl) = result.download {
        let dl_mbps = dl / 1_000_000.0;
        let rating = ratings::speed_rating_mbps(dl_mbps);
        let bar = metric_bar(dl_mbps, 1000.0, BAR_WIDTH, nc);
        if nc {
            lines.push(format!(
                "  {:<14} {}  {:>8.2} Mb/s  ({rating})",
                "Download", bar, dl_mbps
            ));
        } else {
            let speed_str = format!("{dl_mbps:.2} Mb/s");
            let colored_speed = if dl_mbps >= 200.0 {
                speed_str.green().bold().to_string()
            } else if dl_mbps >= 50.0 {
                speed_str.bright_green().to_string()
            } else if dl_mbps >= 25.0 {
                speed_str.yellow().to_string()
            } else {
                speed_str.red().to_string()
            };
            lines.push(format!(
                "  {:<14} {}  {}  {}",
                "Download".dimmed(),
                bar,
                colored_speed,
                ratings::colorize_rating(rating, nc),
            ));
        }
    }

    // Upload bar (0–1000 Mbps scale)
    if let Some(ul) = result.upload {
        let ul_mbps = ul / 1_000_000.0;
        let rating = ratings::speed_rating_mbps(ul_mbps);
        let bar = metric_bar(ul_mbps, 1000.0, BAR_WIDTH, nc);
        if nc {
            lines.push(format!(
                "  {:<14} {}  {:>8.2} Mb/s  ({rating})",
                "Upload", bar, ul_mbps
            ));
        } else {
            let speed_str = format!("{ul_mbps:.2} Mb/s");
            let colored_speed = if ul_mbps >= 200.0 {
                speed_str.green().bold().to_string()
            } else if ul_mbps >= 50.0 {
                speed_str.bright_green().to_string()
            } else if ul_mbps >= 25.0 {
                speed_str.yellow().to_string()
            } else {
                speed_str.red().to_string()
            };
            lines.push(format!(
                "  {:<14} {}  {}  {}",
                "Upload".dimmed(),
                bar,
                colored_speed,
                ratings::colorize_rating(rating, nc),
            ));
        }
    }

    lines.join("\n")
}

/// Build the download summary section.
fn build_download_summary(dl: &TestRunResult, nc: bool) -> String {
    if dl.duration_secs <= 0.0 {
        return String::new();
    }

    let dl_mbps = dl.avg_bps / 1_000_000.0;
    let dl_peak_mbps = dl.peak_bps / 1_000_000.0;

    let mut lines = Vec::new();
    lines.push(section_divider("Download Summary", nc));

    let bar = metric_bar(dl_mbps, 1000.0, BAR_WIDTH, nc);
    if nc {
        lines.push(format!("  {:<14} {:>8.2} Mb/s  {bar}", "Speed:", dl_mbps));
    } else {
        lines.push(format!(
            "  {:<14} {}  {}",
            "Speed:".dimmed(),
            format!("{dl_mbps:.2} Mb/s").cyan().bold(),
            bar,
        ));
    }

    if dl_peak_mbps > 0.0 {
        if nc {
            lines.push(format!("  {:<14} {dl_peak_mbps:.2} Mb/s", "Peak:"));
        } else {
            lines.push(format!(
                "  {:<14} {}",
                "Peak:".dimmed(),
                format!("{dl_peak_mbps:.2} Mb/s").bright_cyan(),
            ));
        }
    }

    if nc {
        lines.push(format!("  {:<14} {:.1}s", "Duration:", dl.duration_secs));
    } else {
        lines.push(format!(
            "  {:<14} {}",
            "Duration:".dimmed(),
            format!("{:.1}s", dl.duration_secs).white(),
        ));
    }

    if nc {
        lines.push(format!(
            "  {:<14} {}",
            "Transferred:",
            format_data_size(dl.total_bytes)
        ));
    } else {
        lines.push(format!(
            "  {:<14} {}",
            "Transferred:".dimmed(),
            format_data_size(dl.total_bytes).white(),
        ));
    }

    lines.join("\n")
}

/// Build the upload summary section.
fn build_upload_summary(ul: &TestRunResult, nc: bool) -> String {
    if ul.duration_secs <= 0.0 {
        return String::new();
    }

    let ul_mbps = ul.avg_bps / 1_000_000.0;
    let ul_peak_mbps = ul.peak_bps / 1_000_000.0;

    let mut lines = Vec::new();
    lines.push(section_divider("Upload Summary", nc));

    let bar = metric_bar(ul_mbps, 1000.0, BAR_WIDTH, nc);
    if nc {
        lines.push(format!("  {:<14} {:>8.2} Mb/s  {bar}", "Speed:", ul_mbps));
    } else {
        lines.push(format!(
            "  {:<14} {}  {}",
            "Speed:".dimmed(),
            format!("{ul_mbps:.2} Mb/s").yellow().bold(),
            bar,
        ));
    }

    if ul_peak_mbps > 0.0 {
        if nc {
            lines.push(format!("  {:<14} {ul_peak_mbps:.2} Mb/s", "Peak:"));
        } else {
            lines.push(format!(
                "  {:<14} {}",
                "Peak:".dimmed(),
                format!("{ul_peak_mbps:.2} Mb/s").bright_yellow(),
            ));
        }
    }

    if nc {
        lines.push(format!("  {:<14} {:.1}s", "Duration:", ul.duration_secs));
    } else {
        lines.push(format!(
            "  {:<14} {}",
            "Duration:".dimmed(),
            format!("{:.1}s", ul.duration_secs).white(),
        ));
    }

    if nc {
        lines.push(format!(
            "  {:<14} {}",
            "Transferred:",
            format_data_size(ul.total_bytes)
        ));
    } else {
        lines.push(format!(
            "  {:<14} {}",
            "Transferred:".dimmed(),
            format_data_size(ul.total_bytes).white(),
        ));
    }

    lines.join("\n")
}

/// Build history section with sparkline.
fn build_history(recent: &HistoryData, nc: bool) -> String {
    if recent.is_empty() {
        if nc {
            return String::from("  History:  No history available");
        }
        return format!(
            "  {} {}",
            "History:".dimmed(),
            "No history available".bright_black()
        );
    }

    let mut lines = Vec::new();
    lines.push(section_divider("History", nc));

    // Build sparkline from download and upload speeds
    let dl_values: Vec<f64> = recent.iter().map(|(_, dl, _)| *dl).collect();
    let ul_values: Vec<f64> = recent.iter().map(|(_, _, ul)| *ul).collect();

    let dl_spark = history::sparkline(&dl_values);
    let ul_spark = history::sparkline(&ul_values);

    if nc {
        lines.push(format!("  DL sparkline:  {dl_spark}"));
        lines.push(format!("  UL sparkline:  {ul_spark}"));
    } else {
        lines.push(format!("  {} {}", "DL:".dimmed(), dl_spark.green()));
        lines.push(format!("  {} {}", "UL:".dimmed(), ul_spark.yellow()));
    }

    // Last 3 entries as text
    for (date, dl, ul) in recent.iter().rev().take(3) {
        let indicator = if *dl >= 200.0 {
            "⚡"
        } else if *dl >= 50.0 {
            "●"
        } else if *dl >= 25.0 {
            "◐"
        } else {
            "○"
        };
        if nc {
            lines.push(format!("  {date}  {dl:>7.1}↓ / {ul:>6.1}↑ Mb/s"));
        } else {
            let indicator_colored = if *dl >= 200.0 {
                indicator.green().to_string()
            } else if *dl >= 50.0 {
                indicator.bright_green().to_string()
            } else if *dl >= 25.0 {
                indicator.yellow().to_string()
            } else {
                indicator.red().to_string()
            };
            lines.push(format!(
                "  {date}  {indicator_colored} {}↓ / {}↑ Mb/s",
                format!("{dl:.1}").green(),
                format!("{ul:.1}").yellow(),
            ));
        }
    }

    lines.join("\n")
}

/// Build footer with keyboard hints (informational — dashboard is static output).
fn build_footer() -> String {
    format!(
        "  {}",
        "Tip: Use --list to see servers, --history for full history".bright_black()
    )
}

/// Format the full dashboard output.
///
/// `history_data` is pre-loaded by the caller so this function has no I/O side effects.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_dashboard(
    result: &TestResult,
    dl: &TestRunResult,
    ul: &TestRunResult,
    history_data: HistoryData,
) -> Result<(), crate::error::SpeedtestError> {
    let nc = no_color();

    eprintln!();
    eprintln!("{}", build_header(result, nc));
    eprintln!();
    eprintln!("{}", build_overall_rating(result, nc));
    eprintln!();
    eprintln!("{}", build_metric_bars(result, nc));
    eprintln!();
    let dl_summary = build_download_summary(dl, nc);
    if !dl_summary.is_empty() {
        eprintln!("{dl_summary}");
        eprintln!();
    }
    let ul_summary = build_upload_summary(ul, nc);
    if !ul_summary.is_empty() {
        eprintln!("{ul_summary}");
        eprintln!();
    }
    eprintln!("{}", build_history(&history_data, nc));
    eprintln!();
    eprintln!("{}", build_footer());
    eprintln!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServerInfo;

    fn make_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "TestServer".to_string(),
                sponsor: "TestISP".to_string(),
                country: "US".to_string(),
                distance: 15.0,
            },
            ping: Some(12.0),
            jitter: Some(1.5),
            packet_loss: Some(0.0),
            download: Some(150_000_000.0),
            download_peak: Some(180_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            latency_download: Some(18.0),
            latency_upload: Some(15.0),
            download_samples: Some(vec![140_000_000.0, 150_000_000.0, 160_000_000.0]),
            upload_samples: Some(vec![48_000_000.0, 50_000_000.0, 52_000_000.0]),
            ping_samples: Some(vec![11.0, 12.0, 13.0]),
            timestamp: "2026-04-06T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.100".to_string()),
        }
    }

    #[test]
    fn test_metric_bar_half() {
        let bar = metric_bar(500.0, 1000.0, 20, true);
        assert_eq!(bar.chars().count(), 20);
        // Half filled: 10█ + 10░
        assert_eq!(bar, "██████████░░░░░░░░░░");
    }

    #[test]
    fn test_metric_bar_full() {
        let bar = metric_bar(1000.0, 1000.0, 10, true);
        assert_eq!(bar, "██████████");
    }

    #[test]
    fn test_build_header() {
        let result = make_result();
        let header = build_header(&result, true);
        assert!(header.contains("netspeed-cli"));
        assert!(header.contains("TestISP"));
        assert!(header.contains("192.168.1.100"));
        // Verify box structure
        assert!(header.starts_with("╔"));
        assert!(header.contains("╚"));
    }

    #[test]
    fn test_build_metric_bars() {
        let result = make_result();
        let bars = build_metric_bars(&result, true);
        assert!(bars.contains("Latency"));
        assert!(bars.contains("Download"));
        assert!(bars.contains("Upload"));
        assert!(bars.contains("█"));
    }

    #[test]
    fn test_build_overall_rating() {
        let result = make_result();
        let rating = build_overall_rating(&result, true);
        assert!(rating.contains("Overall"));
    }

    #[test]
    fn test_build_download_summary() {
        let dl = TestRunResult {
            avg_bps: 150_000_000.0,
            peak_bps: 180_000_000.0,
            total_bytes: 15_000_000,
            duration_secs: 3.2,
            speed_samples: vec![150_000_000.0],
            latency_under_load: None,
        };
        let result = build_download_summary(&dl, true);
        assert!(result.contains("Download Summary"));
        assert!(result.contains("Speed"));
        assert!(result.contains("Peak"));
        assert!(result.contains("150.00"));
    }

    #[test]
    fn test_build_upload_summary() {
        let ul = TestRunResult {
            avg_bps: 50_000_000.0,
            peak_bps: 60_000_000.0,
            total_bytes: 5_000_000,
            duration_secs: 2.1,
            speed_samples: vec![50_000_000.0],
            latency_under_load: None,
        };
        let result = build_upload_summary(&ul, true);
        assert!(result.contains("Upload Summary"));
        assert!(result.contains("Speed"));
        assert!(result.contains("50.00"));
    }

    #[test]
    fn test_build_history_no_data() {
        // History section renders even with empty data
        let section = build_history(&Vec::new(), true);
        // Should always contain the no-data message
        assert!(section.contains("History"));
        assert!(section.contains("No history available"));
    }

    #[test]
    fn test_build_footer() {
        let footer = build_footer();
        assert!(footer.contains("--list"));
        assert!(footer.contains("--history"));
    }

    fn make_dl_result() -> TestRunResult {
        TestRunResult {
            avg_bps: 150_000_000.0,
            peak_bps: 180_000_000.0,
            total_bytes: 15_000_000,
            duration_secs: 3.2,
            speed_samples: vec![150_000_000.0],
            latency_under_load: None,
        }
    }

    fn make_ul_result() -> TestRunResult {
        TestRunResult {
            avg_bps: 50_000_000.0,
            peak_bps: 60_000_000.0,
            total_bytes: 5_000_000,
            duration_secs: 2.1,
            speed_samples: vec![50_000_000.0],
            latency_under_load: None,
        }
    }

    #[test]
    fn test_format_dashboard_integration() {
        let result = make_result();
        let dl = make_dl_result();
        let ul = make_ul_result();
        // Should not panic
        format_dashboard(&result, &dl, &ul, Vec::new()).unwrap();
    }

    #[test]
    fn test_format_dashboard_no_color() {
        let result = make_result();
        let dl = make_dl_result();
        let ul = make_ul_result();
        // SAFETY: test context, no concurrent env access
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        format_dashboard(&result, &dl, &ul, Vec::new()).unwrap();
        // SAFETY: test context, no concurrent env access
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn test_section_divider() {
        let div = section_divider("Speed", true);
        assert!(div.contains("Speed"));
        assert!(div.contains("─"));
    }
}
