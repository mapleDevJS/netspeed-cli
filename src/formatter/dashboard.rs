//! Dashboard output format — rich boxed layout with bar charts, history sparkline, and key hints.
//!
//! This module provides a visually rich single-screen output suitable for
//! terminal display. It uses only existing dependencies (`owo_colors`, `common`,
//! `history`) — no new crates.

use crate::common;
use crate::formatter::ratings;
use crate::history;
use crate::progress::no_color;
use crate::types::TestResult;
use owo_colors::OwoColorize;

/// Render a horizontal bar scaled to the metric's typical range.
fn metric_bar(value_mbps: f64, max_mbps: f64, width: usize, nc: bool) -> String {
    let bar = common::bar_chart(value_mbps, max_mbps, width);
    if nc {
        bar
    } else {
        let fill_pct = (value_mbps / max_mbps).clamp(0.0, 1.0) * 100.0;
        if fill_pct >= 70.0 {
            bar.green().to_string()
        } else if fill_pct >= 40.0 {
            bar.yellow().to_string()
        } else {
            bar.red().to_string()
        }
    }
}

/// Summary data extracted from test runs for dashboard display.
pub struct DashboardSummary {
    pub dl_mbps: f64,
    pub dl_peak_mbps: f64,
    pub dl_bytes: u64,
    pub dl_duration: f64,
    pub ul_mbps: f64,
    pub ul_peak_mbps: f64,
    pub ul_bytes: u64,
    pub ul_duration: f64,
}

/// Build the boxed header with version, server, and client IP.
fn build_header(result: &TestResult, nc: bool) -> String {
    let width = 62;
    let version = env!("CARGO_PKG_VERSION");
    let title = format!("netspeed-cli v{version}");
    let padded_title = format!(" {title:^width$} ", width = width - 4);

    let server_line = format!(
        "  Server: {} ({}) · {} · {}",
        result.server.sponsor,
        result.server.name,
        result.server.country,
        common::format_distance(result.server.distance)
    );

    let mut lines = Vec::new();
    if nc {
        lines.push(format!("╔{padded_title}╗"));
    } else {
        lines.push(format!("╔{padded_title}╗").dimmed().to_string());
    }
    if nc {
        lines.push(format!("║{server_line:<width$}║"));
    } else {
        lines.push(format!("║{server_line:<width$}║").dimmed().to_string());
    }
    if let Some(ip) = &result.client_ip {
        let ip_line = format!("  Client IP: {ip}");
        if nc {
            lines.push(format!("║{ip_line:<width$}║"));
        } else {
            lines.push(format!("║{ip_line:<width$}║").dimmed().to_string());
        }
    }
    if nc {
        lines.push(format!("╚{:═<width$}╝", ""));
    } else {
        lines.push(format!("╚{:═<width$}╝", "").dimmed().to_string());
    }
    lines.join("\n")
}

/// Build metric lines with bar charts for latency, download, and upload.
fn build_metric_bars(result: &TestResult, nc: bool) -> String {
    let bar_width = 28;
    let mut lines = Vec::new();

    // Latency bar (0–100 ms scale, inverted: lower is better)
    if let Some(ping) = result.ping {
        let rating = ratings::ping_rating(ping);
        // Invert: 0ms = full bar, 100ms = empty bar
        let inverted = (100.0 - ping).max(0.0);
        let bar = metric_bar(inverted, 100.0, bar_width, nc);
        if nc {
            lines.push(format!(
                "  {:<10} {}  {:>7.1} ms  ({rating})",
                "Latency", bar, ping
            ));
        } else {
            let ping_str = format!("{ping:.1} ms");
            let rating_str = ratings::colorize_rating(rating, nc);
            lines.push(format!(
                "  {:<10} {}  {}  {}",
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
        let bar = metric_bar(dl_mbps, 1000.0, bar_width, nc);
        if nc {
            lines.push(format!(
                "  {:<10} {}  {:>8.2} Mb/s  ({rating})",
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
                "  {:<10} {}  {}  {}",
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
        let bar = metric_bar(ul_mbps, 1000.0, bar_width, nc);
        if nc {
            lines.push(format!(
                "  {:<10} {}  {:>8.2} Mb/s  ({rating})",
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
                "  {:<10} {}  {}  {}",
                "Upload".dimmed(),
                bar,
                colored_speed,
                ratings::colorize_rating(rating, nc),
            ));
        }
    }

    lines.join("\n")
}

/// Build the live progress summary from dashboard summary data.
fn build_progress_summary(summary: &DashboardSummary, nc: bool) -> String {
    let mut lines = Vec::new();

    if nc {
        lines.push(String::from(
            "  ── Summary ──────────────────────────────────────────────────",
        ));
    } else {
        lines.push(format!(
            "  {}",
            "── Summary ──────────────────────────────────────────────────".dimmed()
        ));
    }

    if summary.dl_duration > 0.0 {
        let bar = metric_bar(summary.dl_mbps, 1000.0, 28, nc);
        if nc {
            lines.push(format!(
                "  Download: {:>8.2} Mb/s  {}  ({:.1}s, {})",
                summary.dl_mbps,
                bar,
                summary.dl_duration,
                common::format_data_size(summary.dl_bytes)
            ));
        } else {
            lines.push(format!(
                "  {:>14}: {} {}  ({:.1}s, {})",
                "Download".dimmed(),
                format!("{:.2} Mb/s", summary.dl_mbps).cyan(),
                bar,
                summary.dl_duration,
                common::format_data_size(summary.dl_bytes).white(),
            ));
        }
        if summary.dl_peak_mbps > 0.0 {
            if nc {
                lines.push(format!("  Peak:       {:.2} Mb/s", summary.dl_peak_mbps));
            } else {
                lines.push(format!(
                    "  {:>14}: {}",
                    "Peak".dimmed(),
                    format!("{:.2} Mb/s", summary.dl_peak_mbps).bright_cyan()
                ));
            }
        }
    }

    if summary.ul_duration > 0.0 {
        let bar = metric_bar(summary.ul_mbps, 1000.0, 28, nc);
        if nc {
            lines.push(format!(
                "  Upload:   {:>8.2} Mb/s  {}  ({:.1}s, {})",
                summary.ul_mbps,
                bar,
                summary.ul_duration,
                common::format_data_size(summary.ul_bytes)
            ));
        } else {
            lines.push(format!(
                "  {:>14}: {} {}  ({:.1}s, {})",
                "Upload".dimmed(),
                format!("{:.2} Mb/s", summary.ul_mbps).yellow(),
                bar,
                summary.ul_duration,
                common::format_data_size(summary.ul_bytes).white(),
            ));
        }
        if summary.ul_peak_mbps > 0.0 {
            if nc {
                lines.push(format!("  Peak:       {:.2} Mb/s", summary.ul_peak_mbps));
            } else {
                lines.push(format!(
                    "  {:>14}: {}",
                    "Peak".dimmed(),
                    format!("{:.2} Mb/s", summary.ul_peak_mbps).bright_yellow()
                ));
            }
        }
    }

    lines.join("\n")
}

/// Build history section with sparkline.
fn build_history(nc: bool) -> String {
    let recent = history::get_recent_sparkline();
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
    if nc {
        lines.push(String::from(
            "  ── History (recent tests) ───────────────────────────────",
        ));
    } else {
        lines.push(format!(
            "  {}",
            "── History (recent tests) ───────────────────────────────".dimmed()
        ));
    }

    // Build sparkline from download speeds
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
        let rating = if *dl >= 200.0 {
            "⚡"
        } else if *dl >= 50.0 {
            "🟢"
        } else if *dl >= 25.0 {
            "🟡"
        } else {
            "🔴"
        };
        if nc {
            lines.push(format!("  {date}  {dl:>7.1}↓ / {ul:>6.1}↑ Mb/s"));
        } else {
            lines.push(format!(
                "  {date}  {rating} {}↓ / {}↑ Mb/s",
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
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_dashboard(
    result: &TestResult,
    summary: &DashboardSummary,
) -> Result<(), crate::error::SpeedtestError> {
    let nc = no_color();

    eprintln!();
    eprintln!("{}", build_header(result, nc));
    eprintln!();
    eprintln!("{}", build_metric_bars(result, nc));
    eprintln!();
    eprintln!("{}", build_progress_summary(summary, nc));
    eprintln!();
    eprintln!("{}", build_history(nc));
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
    fn test_build_progress_summary() {
        let summary = DashboardSummary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
        };
        let result = build_progress_summary(&summary, true);
        assert!(result.contains("Download"));
        assert!(result.contains("Upload"));
        assert!(result.contains("150.00"));
    }

    #[test]
    fn test_build_history_no_data() {
        // History section renders regardless of actual data
        let section = build_history(true);
        // Should always contain the sparkline header
        assert!(section.contains("History"));
    }

    #[test]
    fn test_build_footer() {
        let footer = build_footer();
        assert!(footer.contains("--list"));
        assert!(footer.contains("--history"));
    }

    #[test]
    fn test_format_dashboard_integration() {
        let result = make_result();
        let summary = DashboardSummary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
        };
        // Should not panic
        format_dashboard(&result, &summary).unwrap();
    }

    #[test]
    fn test_format_dashboard_no_color() {
        let result = make_result();
        let summary = DashboardSummary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
        };
        // SAFETY: test context, no concurrent env access
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        format_dashboard(&result, &summary).unwrap();
        // SAFETY: test context, no concurrent env access
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }
}
