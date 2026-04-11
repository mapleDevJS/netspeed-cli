//! Output section formatters for detailed test results.

use crate::formatter::formatting::{bar_chart, format_data_size, format_distance};
use crate::progress::no_color;
use crate::types::{Server, TestResult};
use owo_colors::OwoColorize;

use super::ratings::{
    bufferbloat_colorized, bufferbloat_grade, colorize_rating, degradation_str, format_duration,
    format_speed_colored, format_speed_plain, ping_rating, speed_rating_mbps,
};

/// Build a section separator line for detailed output.
pub(crate) fn section_divider(title: &str, nc: bool) -> String {
    let title_with_spaces = format!(" {title} ");
    let dash_count = 60usize.saturating_sub(title_with_spaces.len());
    let dashes = "─".repeat(dash_count);
    if nc {
        format!("  {title_with_spaces}{dashes}")
    } else {
        format!("  {title_with_spaces}{}", dashes.dimmed())
    }
}

fn build_skipped_line(label: &str, nc: bool) -> String {
    if nc {
        format!("  {:>14}:   — (skipped)", label)
    } else {
        format!(
            "  {:>14}:   {}",
            label.dimmed(),
            "— (skipped)".bright_black()
        )
    }
}

fn build_speed_section(label: &str, speed_bps: f64, bytes: bool, nc: bool) -> String {
    let speed = if nc {
        format_speed_plain(speed_bps, bytes)
    } else {
        format_speed_colored(speed_bps, bytes)
    };
    let rating = colorize_rating(speed_rating_mbps(speed_bps / 1_000_000.0), nc);
    let bar = bar_chart(speed_bps / 1_000_000.0, 1000.0, 28);
    let bar_display = if nc {
        bar
    } else {
        let fill_pct = (speed_bps / 1_000_000.0 / 1000.0).clamp(0.0, 1.0) * 100.0;
        if fill_pct >= 70.0 {
            bar.green().to_string()
        } else if fill_pct >= 40.0 {
            bar.yellow().to_string()
        } else {
            bar.red().to_string()
        }
    };
    if nc {
        format!("  {label:>14}:   {speed}  {bar_display}")
    } else {
        format!(
            "  {:>14}:   {speed}  {bar_display}  {rating}",
            label.dimmed()
        )
    }
}

fn build_peak_line(peak_bps: f64, bytes: bool, nc: bool) -> String {
    let peak = if nc {
        format_speed_plain(peak_bps, bytes)
    } else {
        format_speed_colored(peak_bps, bytes)
    };
    if nc {
        format!("  {:>14}:   {peak}", "Peak (1s avg)")
    } else {
        format!("  {:>14}:   {peak}", "Peak (1s avg)".dimmed())
    }
}

fn build_latency_load_line(lat_load: f64, idle_ping: Option<f64>, nc: bool) -> String {
    let degradation = degradation_str(lat_load, idle_ping, nc);
    if nc {
        format!(
            "  {:>14}:   {:>8.1} ms {degradation}",
            "Latency (load)", lat_load
        )
    } else {
        format!(
            "  {:>14}:   {} {degradation}",
            "Latency (load)".dimmed(),
            format!("{lat_load:.1} ms").yellow(),
        )
    }
}

pub fn build_latency_section(result: &TestResult, nc: bool) -> String {
    let Some(ping) = result.ping else {
        return String::new();
    };

    let mut lines = Vec::new();

    let rating_str = colorize_rating(ping_rating(ping), nc);
    if nc {
        lines.push(format!(
            "  {:>14}:   {:>8.1} ms  ({rating_str})",
            "Latency", ping
        ));
    } else {
        lines.push(format!(
            "  {:>14}:   {}  {rating_str}",
            "Latency".dimmed(),
            format!("{ping:.1} ms").cyan().bold(),
        ));
    }

    if let Some(jitter) = result.jitter {
        if nc {
            lines.push(format!("  {:>14}:   {:>8.1} ms", "Jitter", jitter));
        } else {
            lines.push(format!(
                "  {:>14}:   {}",
                "Jitter".dimmed(),
                format!("{jitter:.1} ms").cyan()
            ));
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
        let loss_str = format!("{loss:.1}%");
        if nc {
            lines.push(format!("  {:>14}:   {:>8}", "Packet Loss", loss_str));
        } else {
            let show_checkmark = loss == 0.0 && !crate::common::no_emoji();
            let display = if show_checkmark {
                format!("{} {}", loss_str.green(), "✓".green())
            } else {
                match loss_color {
                    "green" => loss_str.green().to_string(),
                    "yellow" => loss_str.yellow().to_string(),
                    "red" => loss_str.red().bold().to_string(),
                    _ => loss_str.dimmed().to_string(),
                }
            };
            lines.push(format!("  {:>14}:   {display}", "Packet Loss".dimmed()));
        }
    }

    // Bufferbloat: show if we have latency-under-load data
    if let (Some(lat_dl), Some(lat_ul)) = (result.latency_download, result.latency_upload) {
        let max_load = lat_dl.max(lat_ul);
        let (grade, added) = bufferbloat_grade(max_load, result.ping.unwrap_or(0.0));
        let display = bufferbloat_colorized(grade, added, nc);
        if nc {
            lines.push(format!("  {:>14}:   {:>12}", "Bufferbloat", display));
        } else {
            lines.push(format!("  {:>14}:   {display}", "Bufferbloat".dimmed()));
        }
    }

    lines.join("\n")
}

pub fn format_latency_section(result: &TestResult, nc: bool) {
    let output = build_latency_section(result, nc);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

pub fn build_download_section(result: &TestResult, bytes: bool, nc: bool, skipped: bool) -> String {
    let Some(dl) = result.download else {
        if skipped {
            return build_skipped_line("Download", nc);
        }
        return String::new();
    };

    let mut lines = Vec::new();
    lines.push(build_speed_section("Download", dl, bytes, nc));

    if let Some(peak) = result.download_peak {
        lines.push(build_peak_line(peak, bytes, nc));
    }

    if let Some(lat_dl) = result.latency_download {
        lines.push(build_latency_load_line(lat_dl, result.ping, nc));
    }

    lines.join("\n")
}

pub fn format_download_section(result: &TestResult, bytes: bool, nc: bool, skipped: bool) {
    let output = build_download_section(result, bytes, nc, skipped);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

pub fn build_upload_section(result: &TestResult, bytes: bool, nc: bool, skipped: bool) -> String {
    let Some(ul) = result.upload else {
        if skipped {
            return build_skipped_line("Upload", nc);
        }
        return String::new();
    };

    let mut lines = Vec::new();
    lines.push(build_speed_section("Upload", ul, bytes, nc));

    if let Some(peak) = result.upload_peak {
        lines.push(build_peak_line(peak, bytes, nc));
    }

    if let Some(lat_ul) = result.latency_upload {
        lines.push(build_latency_load_line(lat_ul, result.ping, nc));
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
            lines.push(format!("  {:>14}:   {ratio_str}", "UL/DL Ratio"));
        } else {
            lines.push(format!("  {:>14}:   {ratio_str}", "UL/DL Ratio".dimmed()));
        }
    }

    lines.join("\n")
}

pub fn format_upload_section(result: &TestResult, bytes: bool, nc: bool, skipped: bool) {
    let output = build_upload_section(result, bytes, nc, skipped);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

pub fn build_connection_info(result: &TestResult, nc: bool) -> String {
    let dist = format_distance(result.server.distance);
    let mut lines = Vec::new();

    if nc {
        lines.push(String::from("\n  CONNECTION INFO"));
    } else {
        lines.push(format!("\n  {}", "CONNECTION INFO".bold().underline()));
    }

    if nc {
        lines.push(format!(
            "  {:>16}:   {} ({})",
            "Server", result.server.sponsor, result.server.name
        ));
    } else {
        lines.push(format!(
            "  {:>16}:   {} ({})",
            "Server".dimmed(),
            result.server.sponsor.white().bold(),
            result.server.name
        ));
    }

    if nc {
        lines.push(format!(
            "  {:>16}:   {}  ({dist})",
            "Location", result.server.country
        ));
    } else {
        lines.push(format!(
            "  {:>16}:   {}  ({dist})",
            "Location".dimmed(),
            result.server.country,
        ));
    }

    if let Some(ip) = &result.client_ip {
        if nc {
            lines.push(format!("  {:>16}:   {ip}", "Client IP"));
        } else {
            lines.push(format!("  {:>16}:   {ip}", "Client IP".dimmed()));
        }
    }

    lines.join("\n")
}

pub fn format_connection_info(result: &TestResult, nc: bool) {
    eprintln!("{}", build_connection_info(result, nc));
}

pub fn build_test_summary(
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    nc: bool,
) -> String {
    let mut lines = Vec::new();

    if nc {
        lines.push(String::from("\n  TEST SUMMARY"));
    } else {
        lines.push(format!("\n  {}", "TEST SUMMARY".bold().underline()));
    }

    if dl_bytes > 0 {
        lines.push(format!(
            "  {:>14}:   {} in {}",
            "Download",
            format_data_size(dl_bytes),
            format_duration(dl_duration)
        ));
    }
    if ul_bytes > 0 {
        lines.push(format!(
            "  {:>14}:   {} in {}",
            "Upload",
            format_data_size(ul_bytes),
            format_duration(ul_duration)
        ));
    }
    let total = dl_bytes + ul_bytes;
    let total_dur = dl_duration + ul_duration;
    if total > 0 {
        lines.push(format!(
            "  {:>14}:   {} in {}",
            "Total",
            format_data_size(total),
            format_duration(total_dur)
        ));
    }

    lines.join("\n")
}

pub fn format_test_summary(
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    nc: bool,
) {
    eprintln!(
        "{}",
        build_test_summary(dl_bytes, ul_bytes, dl_duration, ul_duration, nc)
    );
}

pub fn build_footer(timestamp: &str, nc: bool) -> String {
    if nc {
        format!("\n  Completed at: {timestamp}")
    } else {
        format!(
            "\n  {} {}",
            "Completed at:".dimmed(),
            timestamp.bright_black()
        )
    }
}

pub fn format_footer(timestamp: &str, nc: bool) {
    eprintln!("{}", build_footer(timestamp, nc));
}

/// Format a list of available servers.
pub fn build_list(servers: &[Server]) -> String {
    let nc = no_color();
    const MAX_SPONSOR: usize = 35;
    const MAX_NAME: usize = 40;

    let (max_id_len, max_sponsor_len, max_name_len) =
        servers
            .iter()
            .fold((3, 7, 24), |(max_id, max_sponsor, max_name), s| {
                let name_len =
                    (s.name.chars().count() + s.country.chars().count() + 3).min(MAX_NAME);
                (
                    max_id.max(s.id.chars().count()),
                    max_sponsor.max(s.sponsor.chars().count().min(MAX_SPONSOR)),
                    max_name.max(name_len),
                )
            });

    let idw = max_id_len.max(3);
    let sw = max_sponsor_len.clamp(7, MAX_SPONSOR);
    let nw = max_name_len.clamp(24, MAX_NAME);

    /// Truncate a string with ellipsis if it exceeds max_chars.
    fn ellipsis(s: &str, max_chars: usize) -> String {
        let char_count = s.chars().count();
        if char_count <= max_chars {
            s.to_string()
        } else {
            let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
            format!("{truncated}…")
        }
    }

    let mut lines = Vec::new();

    if nc {
        lines.push(String::from("\n  AVAILABLE SERVERS"));
    } else {
        lines.push(format!("\n  {}", "AVAILABLE SERVERS".bold().underline()));
    }

    if nc {
        lines.push(format!(
            "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
            "ID", "Sponsor", "Name (Country)", "Distance"
        ));
    } else {
        lines.push(format!(
            "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
            "ID".dimmed(),
            "Sponsor".dimmed(),
            "Name (Country)".dimmed(),
            "Distance".dimmed()
        ));
    }

    if nc {
        lines.push(format!(
            "  {:->idw$}  {:->sw$}  {:->nw$}  {:->10}",
            "", "", "", ""
        ));
    } else {
        lines.push(format!(
            "  {:->idw$}  {:->sw$}  {:->nw$}  {:->10}",
            "",
            "",
            "",
            "".dimmed()
        ));
    }

    for server in servers {
        let dist = format_distance(server.distance);
        let sponsor_display = ellipsis(&server.sponsor, MAX_SPONSOR);
        let name_display = ellipsis(&format!("{} ({})", server.name, server.country), MAX_NAME);
        if nc {
            lines.push(format!(
                "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
                server.id, sponsor_display, name_display, dist,
            ));
        } else {
            lines.push(format!(
                "  {:<idw$}  {:<sw$}  {:<nw$}  {:>10}",
                server.id,
                sponsor_display.white().bold(),
                name_display,
                dist.bright_black(),
            ));
        }
    }

    lines.push(String::new());
    if nc {
        lines.push(format!("  {} server(s) found", servers.len()));
    } else {
        lines.push(format!(
            "  {} server(s) found",
            servers.len().to_string().green().bold()
        ));
    }

    lines.join("\n")
}

/// Format a list of available servers.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn format_list(servers: &[Server]) -> Result<(), std::io::Error> {
    eprintln!("{}", build_list(servers));
    Ok(())
}
