//! Output section formatters for detailed test results.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::common;
use crate::progress::no_color;
use crate::types::{Server, TestResult};
use owo_colors::OwoColorize;

use super::ratings::{
    bufferbloat_colorized, bufferbloat_grade, colorize_rating, degradation_str, format_duration,
    format_speed_colored, format_speed_plain, ping_rating, speed_rating_mbps,
};

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
        eprintln!("  {:>14}:   {peak}", "Peak (1s avg)");
    } else {
        eprintln!("  {:>14}:   {peak}", "Peak (1s avg)".dimmed());
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

pub fn format_latency_section(result: &TestResult, nc: bool) {
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

    if let Some(loss) = result.packet_loss {
        let loss_color = if loss == 0.0 {
            "green"
        } else if loss < 1.0 {
            "yellow"
        } else {
            "red"
        };
        if nc {
            let loss_str = format!("{loss:.1}%");
            eprintln!("  {:>14}:   {:>8}", "Packet Loss", loss_str);
        } else {
            let loss_str = format!("{loss:.1}%");
            let display = if loss == 0.0 {
                format!("{} {}", loss_str.green(), "✓".green())
            } else {
                match loss_color {
                    "green" => loss_str.green().to_string(),
                    "yellow" => loss_str.yellow().to_string(),
                    "red" => loss_str.red().bold().to_string(),
                    _ => loss_str.dimmed().to_string(),
                }
            };
            eprintln!("  {:>14}:   {display}", "Packet Loss".dimmed());
        }
    }

    // Bufferbloat: show if we have latency-under-load data
    if let (Some(lat_dl), Some(lat_ul)) = (result.latency_download, result.latency_upload) {
        let max_load = lat_dl.max(lat_ul);
        let (grade, added) = bufferbloat_grade(max_load, result.ping.unwrap_or(0.0));
        let display = bufferbloat_colorized(grade, added, nc);
        if nc {
            eprintln!("  {:>14}:   {:>12}", "Bufferbloat", display);
        } else {
            eprintln!("  {:>14}:   {display}", "Bufferbloat".dimmed());
        }
    }
}

pub fn format_download_section(result: &TestResult, bytes: bool, nc: bool) {
    let Some(dl) = result.download else { return };

    format_speed_section("Download", dl, bytes, nc);

    if let Some(peak) = result.download_peak {
        format_peak_line(peak, bytes, nc);
    }

    if let Some(lat_dl) = result.latency_download {
        format_latency_load_line(lat_dl, result.ping, nc);
    }
}

pub fn format_upload_section(result: &TestResult, bytes: bool, nc: bool) {
    let Some(ul) = result.upload else { return };

    format_speed_section("Upload", ul, bytes, nc);

    if let Some(peak) = result.upload_peak {
        format_peak_line(peak, bytes, nc);
    }

    if let Some(lat_ul) = result.latency_upload {
        format_latency_load_line(lat_ul, result.ping, nc);
    }

    // Show UL/DL ratio if both are available
    if let (Some(dl), Some(ul)) = (result.download, result.upload) {
        let ratio = if ul > 0.0 { dl / ul } else { f64::INFINITY };
        let ratio_str = if nc {
            format!("{ratio:.2}x")
        } else {
            let (color, label) = if ratio > 1.5 {
                ("yellow", "download-heavy")
            } else if ratio < 0.67 {
                ("cyan", "upload-favored")
            } else {
                ("green", "balanced")
            };
            match color {
                "green" => format!("{ratio:.2}x {label}").green().to_string(),
                "yellow" => format!("{ratio:.2}x {label}").yellow().to_string(),
                "cyan" => format!("{ratio:.2}x {label}").cyan().to_string(),
                _ => format!("{ratio:.2}x {label}"),
            }
        };
        if nc {
            eprintln!("  {:>14}:   {ratio_str}", "UL/DL Ratio");
        } else {
            eprintln!("  {:>14}:   {ratio_str}", "UL/DL Ratio".dimmed());
        }
    }
}

pub fn format_connection_info(result: &TestResult, nc: bool) {
    let dist = common::format_distance(result.server.distance);
    if nc {
        eprintln!("\n  CONNECTION INFO");
        eprintln!(
            "  {:>16}:   {} ({})",
            "Server", result.server.sponsor, result.server.name
        );
        eprintln!(
            "  {:>16}:   {}  ({dist})",
            "Location", result.server.country
        );
        if let Some(ip) = &result.client_ip {
            eprintln!("  {:>16}:   {ip}", "Client IP");
        }
    } else {
        eprintln!("\n  {}", "CONNECTION INFO".bold().underline());
        eprintln!(
            "  {:>16}:   {} ({})",
            "Server".dimmed(),
            result.server.sponsor.white().bold(),
            result.server.name
        );
        eprintln!(
            "  {:>16}:   {}  ({dist})",
            "Location".dimmed(),
            result.server.country,
        );
        if let Some(ip) = &result.client_ip {
            eprintln!("  {:>16}:   {ip}", "Client IP".dimmed());
        }
    }
}

pub fn format_test_summary(
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    nc: bool,
) {
    if nc {
        eprintln!("\n  TEST SUMMARY");
    } else {
        eprintln!("\n  {}", "TEST SUMMARY".bold().underline());
    }

    if dl_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Download",
            common::format_data_size(dl_bytes),
            format_duration(dl_duration)
        );
    }
    if ul_bytes > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Upload",
            common::format_data_size(ul_bytes),
            format_duration(ul_duration)
        );
    }
    let total = dl_bytes + ul_bytes;
    let total_dur = dl_duration + ul_duration;
    if total > 0 {
        eprintln!(
            "  {:>14}:   {} in {}",
            "Total",
            common::format_data_size(total),
            format_duration(total_dur)
        );
    }
}

pub fn format_footer(timestamp: &str, nc: bool) {
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

/// Format a list of available servers.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_list(servers: &[Server]) -> Result<(), std::io::Error> {
    let nc = no_color();

    let max_id_len = servers.iter().map(|s| s.id.len()).max().unwrap_or(3);
    let max_sponsor_len = servers.iter().map(|s| s.sponsor.len()).max().unwrap_or(7);
    let max_name_len = servers
        .iter()
        .map(|s| s.name.len() + s.country.len() + 3)
        .max()
        .unwrap_or(24);

    let idw = max_id_len.max(3);
    let sw = max_sponsor_len.max(7);
    let nw = max_name_len.max(24);

    if nc {
        eprintln!("\n  AVAILABLE SERVERS");
    } else {
        eprintln!("\n  {}", "AVAILABLE SERVERS".bold().underline());
    }
    if nc {
        eprintln!(
            "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
            "ID", "Sponsor", "Name (Country)", "Distance"
        );
    } else {
        eprintln!(
            "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
            "ID".dimmed(),
            "Sponsor".dimmed(),
            "Name (Country)".dimmed(),
            "Distance".dimmed()
        );
    }
    if nc {
        eprintln!("  {:->idw$}  {:->sw$}  {:->nw$}  {:->10}", "", "", "", "");
    } else {
        eprintln!(
            "  {:->idw$}  {:->sw$}  {:->nw$}  {:->10}",
            "",
            "",
            "",
            "".dimmed()
        );
    }

    for server in servers {
        let dist = common::format_distance(server.distance);
        if nc {
            eprintln!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
                server.id,
                server.sponsor,
                format!("{} ({})", server.name, server.country),
                dist,
            );
        } else {
            eprintln!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
                server.id,
                server.sponsor.white().bold(),
                format!("{} ({})", server.name, server.country),
                dist.bright_black(),
            );
        }
    }

    Ok(())
}
