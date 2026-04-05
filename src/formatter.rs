#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::error::SpeedtestError;
use crate::progress::no_color;
use crate::types::{CsvOutput, Server, TestResult};
use owo_colors::OwoColorize;

// ── Rating helpers ────────────────────────────────────────────────
#[must_use]
pub fn ping_rating(ping_ms: f64) -> &'static str {
    if ping_ms < 10.0 {
        "Excellent"
    } else if ping_ms < 30.0 {
        "Good"
    } else if ping_ms < 60.0 {
        "Fair"
    } else if ping_ms < 100.0 {
        "Poor"
    } else {
        "Bad"
    }
}

#[must_use]
pub fn speed_rating_mbps(mbps: f64) -> &'static str {
    if mbps >= 500.0 {
        "Excellent"
    } else if mbps >= 200.0 {
        "Great"
    } else if mbps >= 100.0 {
        "Good"
    } else if mbps >= 50.0 {
        "Fair"
    } else if mbps >= 25.0 {
        "Moderate"
    } else if mbps >= 10.0 {
        "Slow"
    } else {
        "Very Slow"
    }
}

fn colorize_rating(rating: &str, nc: bool) -> String {
    if nc {
        rating.to_string()
    } else {
        match rating {
            "Excellent" => format!("{} {}", "⚡", rating.green().bold()),
            "Great" => format!("{} {}", "🟢", rating.green()),
            "Good" => format!("{} {}", "🟢", rating.bright_green()),
            "Fair" => format!("{} {}", "🟡", rating.yellow()),
            "Moderate" => format!("{} {}", "🟠", rating.bright_yellow()),
            "Poor" => format!("{} {}", "🔴", rating.red()),
            "Slow" => format!("{} {}", "🔴", rating.bright_red()),
            "Very Slow" => format!("{} {}", "⚠️ ", rating.red().bold()),
            _ => rating.to_string(),
        }
    }
}

fn format_speed_colored(bps: f64, bytes: bool) -> String {
    let divider = if bytes { 8.0 } else { 1.0 };
    let unit = if bytes { "MB/s" } else { "Mb/s" };
    let value = bps / divider / 1_000_000.0;
    let mbps = bps / 1_000_000.0;
    let rating = speed_rating_mbps(mbps);
    match rating {
        "Excellent" | "Great" => format!("{value:.2} {unit}").green().bold().to_string(),
        "Good" => format!("{value:.2} {unit}").bright_green().to_string(),
        "Fair" | "Moderate" => format!("{value:.2} {unit}").yellow().to_string(),
        "Poor" | "Slow" | "Very Slow" => format!("{value:.2} {unit}").red().to_string(),
        _ => format!("{value:.2} {unit}"),
    }
}

fn format_speed_plain(bps: f64, bytes: bool) -> String {
    let divider = if bytes { 8.0 } else { 1.0 };
    let unit = if bytes { "MB/s" } else { "Mb/s" };
    let value = bps / divider / 1_000_000.0;
    format!("{value:.2} {unit}")
}

fn format_data(bytes: u64) -> String {
    if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        format!("{secs:.1}s")
    } else {
        let mins = secs as u64 / 60;
        let secs = secs % 60.0;
        format!("{mins}m {secs:.0}s")
    }
}

#[must_use]
pub fn connection_rating(result: &TestResult) -> &'static str {
    let mut score = 0.0;
    let mut factors = 0.0;

    if let Some(ping) = result.ping {
        score += if ping < 10.0 {
            100.0
        } else if ping < 30.0 {
            80.0
        } else if ping < 60.0 {
            60.0
        } else if ping < 100.0 {
            40.0
        } else {
            20.0
        };
        factors += 1.0;
    }

    if let Some(jitter) = result.jitter {
        score += if jitter < 2.0 {
            100.0
        } else if jitter < 5.0 {
            80.0
        } else if jitter < 10.0 {
            60.0
        } else if jitter < 20.0 {
            40.0
        } else {
            20.0
        };
        factors += 1.0;
    }

    if let Some(dl) = result.download {
        let mbps = dl / 1_000_000.0;
        score += if mbps >= 500.0 {
            100.0
        } else if mbps >= 200.0 {
            85.0
        } else if mbps >= 100.0 {
            70.0
        } else if mbps >= 50.0 {
            55.0
        } else if mbps >= 25.0 {
            40.0
        } else if mbps >= 10.0 {
            25.0
        } else {
            10.0
        };
        factors += 1.0;
    }

    if let Some(ul) = result.upload {
        let mbps = ul / 1_000_000.0;
        score += if mbps >= 500.0 {
            100.0
        } else if mbps >= 200.0 {
            85.0
        } else if mbps >= 100.0 {
            70.0
        } else if mbps >= 50.0 {
            55.0
        } else if mbps >= 25.0 {
            40.0
        } else if mbps >= 10.0 {
            25.0
        } else {
            10.0
        };
        factors += 1.0;
    }

    if factors == 0.0 {
        return "Unknown";
    }

    let avg = score / factors;
    if avg >= 90.0 {
        "Excellent"
    } else if avg >= 75.0 {
        "Great"
    } else if avg >= 55.0 {
        "Good"
    } else if avg >= 40.0 {
        "Fair"
    } else if avg >= 25.0 {
        "Moderate"
    } else {
        "Poor"
    }
}

// ── Section formatters ────────────────────────────────────────────

fn separator(nc: bool) -> String {
    if nc {
        "  ──────────────────────────────".to_string()
    } else {
        format!("  {}", "──────────────────────────────".bright_black())
    }
}

fn format_overall_rating(result: &TestResult, nc: bool) -> String {
    let rating = connection_rating(result);
    if nc {
        format!("  Overall: {rating}")
    } else {
        let (icon, color) = match rating {
            "Excellent" => ("⚡", "green"),
            "Great" => ("🟢", "green"),
            "Good" => ("🟢", "bright_green"),
            "Fair" => ("🟡", "yellow"),
            "Moderate" => ("🟠", "bright_yellow"),
            "Poor" => ("🔴", "red"),
            _ => ("", ""),
        };
        let text = format!("{icon} {rating}");
        let colored = match color {
            "green" => text.green().bold().to_string(),
            "bright_green" => text.bright_green().to_string(),
            "yellow" => text.yellow().to_string(),
            "bright_yellow" => text.bright_yellow().to_string(),
            "red" => text.red().to_string(),
            _ => text.dimmed().to_string(),
        };
        format!("  {} {colored}", "Overall:".dimmed())
    }
}

fn format_latency_section(result: &TestResult, nc: bool) {
    let Some(ping) = result.ping else { return };

    let rating_str = colorize_rating(ping_rating(ping), nc);
    if nc {
        eprintln!("  {:>14}:   {:>8.1} ms  ({rating_str})", "Latency", ping);
    } else {
        eprintln!(
            "  {:>14}:   {}  {rating_str}",
            "Latency".dimmed(),
            format!("{ping:.1} ms").cyan().bold(),
        );
    }

    if let Some(jitter) = result.jitter {
        if nc {
            eprintln!("  {:>14}:   {:>8.1} ms", "Jitter", jitter);
        } else {
            eprintln!(
                "  {:>14}:   {}",
                "Jitter".dimmed(),
                format!("{jitter:.1} ms").cyan()
            );
        }
    }
}

fn degradation_str(lat_load: f64, idle_ping: Option<f64>, nc: bool) -> String {
    let Some(idle) = idle_ping else {
        return String::new();
    };
    if idle <= 0.0 {
        return String::new();
    }
    let pct = ((lat_load / idle) - 1.0) * 100.0;
    let (label, color) = if pct < 25.0 {
        ("minimal", "green")
    } else if pct < 50.0 {
        ("moderate", "yellow")
    } else {
        ("significant", "red")
    };
    let text = format!("+{pct:.0}% ({label})");
    if nc {
        format!("  [{text:>8}]")
    } else {
        let colored = match color {
            "green" => text.green().to_string(),
            "yellow" => text.yellow().to_string(),
            "red" => text.red().to_string(),
            _ => text.dimmed().to_string(),
        };
        format!("  {colored}")
    }
}

fn format_speed_section(label: &str, speed_bps: f64, bytes: bool, nc: bool) {
    let speed = if nc {
        format_speed_plain(speed_bps, bytes)
    } else {
        format_speed_colored(speed_bps, bytes)
    };
    let rating = colorize_rating(speed_rating_mbps(speed_bps / 1_000_000.0), nc);
    if nc {
        eprintln!("  {label:>14}:   {speed}");
    } else {
        eprintln!("  {:>14}:   {speed}  {rating}", label.dimmed());
    }
}

fn format_peak_line(peak_bps: f64, bytes: bool, nc: bool) {
    let peak = if nc {
        format_speed_plain(peak_bps, bytes)
    } else {
        format_speed_colored(peak_bps, bytes)
    };
    if nc {
        eprintln!("  {:>14}:   {peak}", "Peak");
    } else {
        eprintln!("  {:>14}:   {peak}", "Peak".dimmed());
    }
}

fn format_latency_load_line(lat_load: f64, idle_ping: Option<f64>, nc: bool) {
    let degradation = degradation_str(lat_load, idle_ping, nc);
    if nc {
        eprintln!(
            "  {:>14}:   {:>8.1} ms{degradation}",
            "Latency (load)", lat_load
        );
    } else {
        eprintln!(
            "  {:>14}:   {}{degradation}",
            "Latency (load)".dimmed(),
            format!("{lat_load:.1} ms").yellow(),
        );
    }
}

fn format_download_section(result: &TestResult, bytes: bool, nc: bool) {
    let Some(dl) = result.download else { return };

    format_speed_section("Download", dl, bytes, nc);

    if let Some(peak) = result.download_peak {
        format_peak_line(peak, bytes, nc);
    }

    if let Some(lat_dl) = result.latency_download {
        format_latency_load_line(lat_dl, result.ping, nc);
    }
}

fn format_upload_section(result: &TestResult, bytes: bool, nc: bool) {
    let Some(ul) = result.upload else { return };

    format_speed_section("Upload", ul, bytes, nc);

    if let Some(peak) = result.upload_peak {
        format_peak_line(peak, bytes, nc);
    }

    if let Some(lat_ul) = result.latency_upload {
        format_latency_load_line(lat_ul, result.ping, nc);
    }
}

fn format_connection_info(result: &TestResult, nc: bool) {
    if nc {
        eprintln!("\n  CONNECTION INFO");
        eprintln!(
            "  {:>14}:   {} ({})",
            "Server", result.server.sponsor, result.server.name
        );
        eprintln!(
            "  {:>14}:   {}  ({:.0} km)",
            "Location", result.server.country, result.server.distance
        );
        if let Some(ip) = &result.client_ip {
            eprintln!("  {:>14}:   {ip}", "Client IP");
        }
    } else {
        eprintln!("\n  {}", "CONNECTION INFO".bold().underline());
        eprintln!(
            "  {:>14}:   {} ({})",
            "Server".dimmed(),
            result.server.sponsor.white().bold(),
            result.server.name
        );
        eprintln!(
            "  {:>14}:   {}  ({:.0} km)",
            "Location".dimmed(),
            result.server.country,
            result.server.distance
        );
        if let Some(ip) = &result.client_ip {
            eprintln!("  {:>14}:   {ip}", "Client IP".dimmed());
        }
    }
}

fn format_test_summary(dl_bytes: u64, ul_bytes: u64, dl_duration: f64, ul_duration: f64, nc: bool) {
    if nc {
        eprintln!("\n  TEST SUMMARY");
    } else {
        eprintln!("\n  {}", "TEST SUMMARY".bold().underline());
    }

    if dl_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Download",
            format_data(dl_bytes),
            format_duration(dl_duration)
        );
    }
    if ul_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Upload",
            format_data(ul_bytes),
            format_duration(ul_duration)
        );
    }
    let total = dl_bytes + ul_bytes;
    let total_dur = dl_duration + ul_duration;
    if total > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Total",
            format_data(total),
            format_duration(total_dur)
        );
    }
}

fn format_footer(timestamp: &str, nc: bool) {
    if nc {
        eprintln!("\n  Completed at: {timestamp}");
    } else {
        eprintln!(
            "\n  {} {}",
            "Completed at:".dimmed(),
            timestamp.bright_black()
        );
    }
}

// ── Public formatters ─────────────────────────────────────────────

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
            format_speed_plain(dl, bytes)
        } else {
            format_speed_colored(dl, bytes)
        };
        parts.push(format!("Download: {speed}"));
    }

    if let Some(ul) = result.upload {
        let speed = if nc {
            format_speed_plain(ul, bytes)
        } else {
            format_speed_colored(ul, bytes)
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
    let sep = separator(nc);

    if nc {
        eprintln!("\n  TEST RESULTS");
    } else {
        eprintln!("\n  {}", "TEST RESULTS".bold().underline());
    }
    eprintln!("{}", format_overall_rating(result, nc));
    eprintln!();

    format_latency_section(result, nc);
    eprintln!("{sep}");
    format_download_section(result, bytes, nc);
    format_upload_section(result, bytes, nc);
    eprintln!("{sep}");
    format_connection_info(result, nc);
    eprintln!("{sep}");
    format_test_summary(dl_bytes, ul_bytes, dl_duration, ul_duration, nc);
    format_footer(&result.timestamp, nc);

    Ok(())
}

/// Output test results as JSON to stdout.
///
/// # Errors
///
/// Returns [`SpeedtestError::ParseError`] if serialization fails.
pub fn format_json(result: &TestResult, _simple: bool) -> Result<(), SpeedtestError> {
    let is_tty = atty::is(atty::Stream::Stdout);
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
/// Returns [`SpeedtestError::Custom`] if CSV serialization fails.
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

/// Output a list of available servers to stderr.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_list(servers: &[Server]) -> Result<(), SpeedtestError> {
    let nc = no_color();

    if servers.is_empty() {
        if nc {
            eprintln!("No servers found.");
        } else {
            eprintln!("{}", "No servers found.".yellow().bold());
        }
        return Ok(());
    }

    if !nc {
        eprintln!(
            "{}",
            format!("Found {} servers:", servers.len()).dimmed().bold()
        );
        eprintln!();
    }

    let max_id = servers.iter().map(|s| s.id.len()).max().unwrap_or(4).max(4);
    let max_sponsor = servers
        .iter()
        .map(|s| s.sponsor.len())
        .max()
        .unwrap_or(8)
        .max(8);

    if nc {
        eprintln!(
            "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
            "ID",
            "Sponsor",
            "Server (Country)",
            "Distance",
            idw = max_id,
            sw = max_sponsor
        );
        eprintln!(
            "  {:─<idw$}  {:─<sw$}  {:─<24}  {:>10}",
            "",
            "",
            "",
            "─────────",
            idw = max_id,
            sw = max_sponsor
        );
    } else {
        eprintln!(
            "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
            "ID".bold(),
            "Sponsor".bold(),
            "Server (Country)".bold(),
            "Distance".bold(),
            idw = max_id,
            sw = max_sponsor
        );
        eprintln!(
            "  {}  {}  {:─<24}  {:>10}",
            "─".repeat(max_id).bright_black(),
            "─".repeat(max_sponsor).bright_black(),
            "",
            "─────────".bright_black()
        );
    }

    for server in servers {
        if nc {
            eprintln!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>6.1} km",
                server.id,
                server.sponsor,
                format!("{} ({})", server.name, server.country),
                server.distance,
                idw = max_id,
                sw = max_sponsor
            );
        } else {
            eprintln!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>6.1} km",
                server.id.cyan().bold(),
                server.sponsor,
                format!("{} ({})", server.name, server.country),
                format!("{:.1}", server.distance).dimmed(),
                idw = max_id,
                sw = max_sponsor
            );
        }
    }

    Ok(())
}

// ── TTY helper ────────────────────────────────────────────────────

mod atty {
    use std::io::IsTerminal;
    pub enum Stream {
        Stdout,
    }
    pub fn is(_stream: Stream) -> bool {
        std::io::stdout().is_terminal()
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServerInfo;

    fn create_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.5,
            },
            ping: Some(15.234),
            jitter: None,
            download: Some(150_000_000.0),
            download_peak: None,
            upload: Some(50_000_000.0),
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.1".to_string()),
        }
    }

    #[test]
    fn test_ping_ratings() {
        assert_eq!(ping_rating(5.0), "Excellent");
        assert_eq!(ping_rating(20.0), "Good");
        assert_eq!(ping_rating(45.0), "Fair");
        assert_eq!(ping_rating(80.0), "Poor");
        assert_eq!(ping_rating(150.0), "Bad");
    }

    #[test]
    fn test_speed_ratings() {
        assert_eq!(speed_rating_mbps(600.0), "Excellent");
        assert_eq!(speed_rating_mbps(300.0), "Great");
        assert_eq!(speed_rating_mbps(150.0), "Good");
        assert_eq!(speed_rating_mbps(75.0), "Fair");
        assert_eq!(speed_rating_mbps(30.0), "Moderate");
        assert_eq!(speed_rating_mbps(15.0), "Slow");
        assert_eq!(speed_rating_mbps(5.0), "Very Slow");
    }

    #[test]
    fn test_format_simple_bits() {
        assert!(format_simple(&create_test_result(), false).is_ok());
    }

    #[test]
    fn test_format_simple_bytes() {
        assert!(format_simple(&create_test_result(), true).is_ok());
    }

    #[test]
    fn test_format_simple_no_ping() {
        let mut r = create_test_result();
        r.ping = None;
        assert!(format_simple(&r, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_download() {
        let mut r = create_test_result();
        r.download = None;
        assert!(format_simple(&r, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_upload() {
        let mut r = create_test_result();
        r.upload = None;
        assert!(format_simple(&r, false).is_ok());
    }

    #[test]
    fn test_format_detailed() {
        assert!(format_detailed(
            &create_test_result(),
            false,
            15_000_000,
            5_000_000,
            3.5,
            2.1
        )
        .is_ok());
    }

    #[test]
    fn test_format_detailed_bytes() {
        assert!(
            format_detailed(&create_test_result(), true, 15_000_000, 5_000_000, 3.5, 2.1).is_ok()
        );
    }

    #[test]
    fn test_format_json_pretty() {
        assert!(format_json(&create_test_result(), false).is_ok());
    }

    #[test]
    fn test_format_json_compact() {
        assert!(format_json(&create_test_result(), true).is_ok());
    }

    #[test]
    fn test_format_csv_header() {
        assert!(format_csv(&create_test_result(), ',', true).is_ok());
    }

    #[test]
    fn test_format_csv_no_header() {
        assert!(format_csv(&create_test_result(), ',', false).is_ok());
    }

    #[test]
    fn test_format_csv_delimiter() {
        assert!(format_csv(&create_test_result(), ';', false).is_ok());
    }

    #[test]
    fn test_format_list() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://s1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP 1".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 15.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://s2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP 2".to_string(),
                country: "UK".to_string(),
                lat: 51.0,
                lon: -0.1,
                distance: 200.0,
                latency: 25.0,
            },
        ];
        assert!(format_list(&servers).is_ok());
    }

    #[test]
    fn test_format_empty_list() {
        let servers: Vec<Server> = vec![];
        assert!(format_list(&servers).is_ok());
    }

    #[test]
    fn test_colorize_rating_nc() {
        assert_eq!(colorize_rating("Excellent", true), "Excellent");
    }

    #[test]
    fn test_format_speed_colored() {
        let r = format_speed_colored(150_000_000.0, false);
        assert!(r.contains("Mb/s"));
    }

    #[test]
    fn test_format_speed_colored_bytes() {
        let r = format_speed_colored(150_000_000.0, true);
        assert!(r.contains("MB/s"));
    }

    #[test]
    fn test_format_speed_plain() {
        let r = format_speed_plain(150_000_000.0, false);
        assert!(r.contains("Mb/s"));
    }

    #[test]
    fn test_format_data_kb() {
        assert_eq!(format_data(5120), "5.0 KB");
    }

    #[test]
    fn test_format_data_mb() {
        assert_eq!(format_data(5_242_880), "5.0 MB");
    }

    #[test]
    fn test_format_data_gb() {
        assert_eq!(format_data(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(3.5), "3.5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(75.0), "1m 15s");
    }

    #[test]
    fn test_connection_rating_excellent() {
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: Some(5.0),
            jitter: Some(1.0),
            download: Some(600_000_000.0),
            download_peak: None,
            upload: Some(600_000_000.0),
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: None,
        };
        assert_eq!(connection_rating(&result), "Excellent");
    }

    #[test]
    fn test_connection_rating_poor() {
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: Some(200.0),
            jitter: Some(50.0),
            download: Some(5_000_000.0),
            download_peak: None,
            upload: Some(5_000_000.0),
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: None,
        };
        assert_eq!(connection_rating(&result), "Poor");
    }

    #[test]
    fn test_connection_rating_unknown() {
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
            download: None,
            download_peak: None,
            upload: None,
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: None,
        };
        assert_eq!(connection_rating(&result), "Unknown");
    }

    #[test]
    fn test_degradation_minimal() {
        let d = degradation_str(16.0, Some(15.0), true);
        assert!(d.contains("minimal"));
    }

    #[test]
    fn test_degradation_significant() {
        let d = degradation_str(45.0, Some(15.0), true);
        assert!(d.contains("significant"));
    }

    #[test]
    fn test_degradation_no_ping() {
        let d = degradation_str(30.0, None, true);
        assert!(d.is_empty());
    }

    #[test]
    fn test_separator_nc() {
        assert!(separator(true).contains("──"));
    }

    #[test]
    fn test_overall_rating_nc() {
        let r = create_test_result();
        let s = format_overall_rating(&r, true);
        assert!(s.contains("Overall"));
    }
}
