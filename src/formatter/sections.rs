//! Output section formatters for detailed test results.

use crate::common;
use crate::terminal;
use crate::theme::{Colors, Theme};
use crate::types::{Server, TestResult};
use owo_colors::OwoColorize;

use super::ratings::{
    bufferbloat_colorized, bufferbloat_grade, colorize_rating, degradation_str, ping_rating,
    speed_rating_mbps,
};

// ── Tabular Column Widths ────────────────────────────────────────────────────
const LATENCY_WIDTH: usize = 10; // "    12.1 ms"
const JITTER_WIDTH: usize = 10; // "     1.5 ms"
const LOSS_WIDTH: usize = 8; // "     0.0%"
const SPEED_WIDTH: usize = 14; // "    150.00 Mb/s"
const DATA_SIZE_WIDTH: usize = 10; // "    15.0 MB"
const DURATION_WIDTH: usize = 8; // "    30.5s"

// ── Layout Mode ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Compact,
    Standard,
    Expanded,
}

impl LayoutMode {
    #[must_use]
    pub fn detect() -> Self {
        let width = crate::common::get_terminal_width().unwrap_or(100);
        if width < 80 {
            Self::Compact
        } else if width < 100 {
            Self::Standard
        } else {
            Self::Expanded
        }
    }
}

/// Build a section header with consistent formatting.
fn section_header(title: &str, nc: bool) -> String {
    if nc {
        format!("\n  {title}")
    } else {
        format!("\n  {}", title.bold().underline())
    }
}

fn build_skipped_line(label: &str, nc: bool) -> String {
    if nc {
        format!("  {label:>14}:   — (skipped)")
    } else {
        format!(
            "  {:>14}:   {}",
            label.dimmed(),
            "— (skipped)".bright_black()
        )
    }
}

fn build_speed_section(
    label: &str,
    speed_bps: f64,
    _bytes: bool,
    nc: bool,
    theme: Theme,
) -> String {
    let speed_tabular = common::format_speed_tabular(speed_bps, SPEED_WIDTH);
    let rating = colorize_rating(speed_rating_mbps(speed_bps / 1_000_000.0), nc, theme);
    let bar = crate::common::bar_chart(speed_bps / 1_000_000.0, 1000.0, 28);
    let bar_display = if nc {
        bar
    } else {
        let fill_pct = (speed_bps / 1_000_000.0 / 1000.0).clamp(0.0, 1.0) * 100.0;
        if fill_pct >= 70.0 {
            Colors::good(&bar, theme)
        } else if fill_pct >= 40.0 {
            Colors::warn(&bar, theme)
        } else {
            Colors::bad(&bar, theme)
        }
    };
    if nc {
        format!("  {label:>14}:   {speed_tabular}  {bar_display}")
    } else {
        let speed_colored = {
            let fill_pct = (speed_bps / 1_000_000.0 / 1000.0).clamp(0.0, 1.0) * 100.0;
            if fill_pct >= 70.0 {
                Colors::good(speed_tabular.trim(), theme)
            } else if fill_pct >= 40.0 {
                Colors::warn(speed_tabular.trim(), theme)
            } else {
                Colors::bad(speed_tabular.trim(), theme)
            }
        };
        format!(
            "  {:>14}:   {:>SPEED_WIDTH$}  {bar_display}  {rating}",
            Colors::dimmed(label, theme),
            speed_colored,
        )
    }
}

fn build_peak_line(peak_bps: f64, _bytes: bool, nc: bool, theme: Theme) -> String {
    let peak_tabular = common::format_speed_tabular(peak_bps, SPEED_WIDTH);
    let peak = if nc {
        peak_tabular
    } else {
        Colors::dimmed(peak_tabular.trim(), theme)
    };
    if nc {
        format!("  {:>14}:   {peak}", "Peak (1s avg)")
    } else {
        format!("  {:>14}:   {peak}", "Peak (1s avg)".dimmed())
    }
}

fn build_latency_load_line(
    lat_load: f64,
    idle_ping: Option<f64>,
    nc: bool,
    theme: Theme,
) -> String {
    let degradation = degradation_str(lat_load, idle_ping, nc, theme);
    let lat_val = common::format_latency_tabular(lat_load, LATENCY_WIDTH);
    if nc {
        format!("  {:>14}:   {lat_val}{degradation}", "Latency (load)")
    } else {
        format!(
            "  {:>14}:   {}{degradation}",
            "Latency (load)".dimmed(),
            Colors::warn(lat_val.trim(), theme),
        )
    }
}

#[must_use]
pub fn build_latency_section(result: &TestResult, nc: bool, theme: Theme) -> String {
    let Some(ping) = result.ping else {
        return String::new();
    };

    let mut lines = Vec::new();

    let rating_str = colorize_rating(ping_rating(ping), nc, theme);
    let latency_val = common::format_latency_tabular(ping, LATENCY_WIDTH);
    if nc {
        lines.push(format!(
            "  {:>14}:   {latency_val}  ({rating_str})",
            "Latency"
        ));
    } else {
        lines.push(format!(
            "  {:>14}:   {}  {rating_str}",
            "Latency".dimmed(),
            Colors::info(latency_val.trim(), theme),
        ));
    }

    if let Some(jitter) = result.jitter {
        let jitter_val = common::format_jitter_tabular(jitter, JITTER_WIDTH);
        lines.push(format!("  {:>14}:   {jitter_val}", "Jitter".dimmed()));
    }

    if let Some(loss) = result.packet_loss {
        let loss_str = if nc || terminal::no_emoji() {
            common::format_loss_tabular(loss, LOSS_WIDTH)
        } else {
            let loss_val = common::format_loss_tabular(loss, LOSS_WIDTH);
            if loss == 0.0 {
                Colors::good(loss_val.trim(), theme)
            } else if loss < 1.0 {
                Colors::warn(loss_val.trim(), theme)
            } else {
                Colors::bad(loss_val.trim(), theme)
            }
        };
        lines.push(format!("  {:>14}:   {loss_str}", "Packet Loss".dimmed()));
    }

    if let (Some(lat_dl), Some(lat_ul)) = (result.latency_download, result.latency_upload) {
        let max_load = lat_dl.max(lat_ul);
        let (grade, added) = bufferbloat_grade(max_load, result.ping.unwrap_or(0.0));
        let display = bufferbloat_colorized(grade, added, nc, theme);
        lines.push(format!("  {:>14}:   {display}", "Bufferbloat".dimmed()));
    }

    lines.join("\n")
}

pub fn format_latency_section(result: &TestResult, nc: bool, theme: Theme) {
    let output = build_latency_section(result, nc, theme);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

#[must_use]
pub fn build_download_section(
    result: &TestResult,
    bytes: bool,
    nc: bool,
    skipped: bool,
    theme: Theme,
) -> String {
    let Some(dl) = result.download else {
        if skipped {
            return build_skipped_line("Download", nc);
        }
        return String::new();
    };

    let mut lines = Vec::new();
    lines.push(build_speed_section("Download", dl, bytes, nc, theme));

    if let Some(peak) = result.download_peak {
        lines.push(build_peak_line(peak, bytes, nc, theme));
    }

    if let Some(lat_dl) = result.latency_download {
        lines.push(build_latency_load_line(lat_dl, result.ping, nc, theme));
    }

    if let Some(cv) = result.download_cv {
        let cv_pct = cv * 100.0;
        let stability = if cv_pct < 5.0 {
            "stable"
        } else if cv_pct < 15.0 {
            "variable"
        } else {
            "unstable"
        };
        if nc {
            lines.push(format!(
                "  {:>14}:   ±{cv_pct:.1}% ({stability})",
                "Variance"
            ));
        } else {
            let cv_display = format!("{cv_pct:.1}");
            let cv_color = if cv_pct < 5.0 {
                Colors::good(&cv_display, theme)
            } else if cv_pct < 15.0 {
                Colors::warn(&cv_display, theme)
            } else {
                Colors::bad(&cv_display, theme)
            };
            lines.push(format!(
                "  {:>14}:   ±{}% ({stability})",
                "Variance".dimmed(),
                cv_color
            ));
        }
    }

    lines.join("\n")
}

pub fn format_download_section(
    result: &TestResult,
    bytes: bool,
    nc: bool,
    skipped: bool,
    theme: Theme,
) {
    let output = build_download_section(result, bytes, nc, skipped, theme);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

#[must_use]
pub fn build_upload_section(
    result: &TestResult,
    bytes: bool,
    nc: bool,
    skipped: bool,
    theme: Theme,
) -> String {
    let Some(ul) = result.upload else {
        if skipped {
            return build_skipped_line("Upload", nc);
        }
        return String::new();
    };

    let mut lines = Vec::new();
    lines.push(build_speed_section("Upload", ul, bytes, nc, theme));

    if let Some(peak) = result.upload_peak {
        lines.push(build_peak_line(peak, bytes, nc, theme));
    }

    if let Some(lat_ul) = result.latency_upload {
        lines.push(build_latency_load_line(lat_ul, result.ping, nc, theme));
    }

    if let Some(cv) = result.upload_cv {
        let cv_pct = cv * 100.0;
        let stability = if cv_pct < 5.0 {
            "stable"
        } else if cv_pct < 15.0 {
            "variable"
        } else {
            "unstable"
        };
        if nc {
            lines.push(format!(
                "  {:>14}:   ±{cv_pct:.1}% ({stability})",
                "Variance"
            ));
        } else {
            let cv_display = format!("{cv_pct:.1}");
            let cv_color = if cv_pct < 5.0 {
                Colors::good(&cv_display, theme)
            } else if cv_pct < 15.0 {
                Colors::warn(&cv_display, theme)
            } else {
                Colors::bad(&cv_display, theme)
            };
            lines.push(format!(
                "  {:>14}:   ±{}% ({stability})",
                "Variance".dimmed(),
                cv_color
            ));
        }
    }

    if let (Some(dl), Some(ul)) = (result.download, result.upload) {
        let ratio = if ul > 0.0 { dl / ul } else { f64::INFINITY };
        let ratio_str = if nc {
            format!("{ratio:.2}x")
        } else {
            let label = if ratio > 1.5 {
                "download-heavy"
            } else if ratio < 0.67 {
                "upload-favored"
            } else {
                "balanced"
            };
            let text = format!("{ratio:.2}x {label}");
            if ratio > 1.5 {
                Colors::warn(&text, theme)
            } else if ratio < 0.67 {
                Colors::info(&text, theme)
            } else {
                Colors::good(&text, theme)
            }
        };
        lines.push(format!("  {:>14}:   {ratio_str}", "UL/DL Ratio".dimmed()));
    }

    lines.join("\n")
}

pub fn format_upload_section(
    result: &TestResult,
    bytes: bool,
    nc: bool,
    skipped: bool,
    theme: Theme,
) {
    let output = build_upload_section(result, bytes, nc, skipped, theme);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

#[must_use]
pub fn build_connection_info(result: &TestResult, nc: bool, theme: Theme) -> String {
    let dist = common::format_distance(result.server.distance);
    let mut lines = Vec::new();

    lines.push(section_header("CONNECTION INFO", nc));

    if nc {
        lines.push(format!(
            "  {:>16}:   {} ({})",
            "Server", result.server.sponsor, result.server.name
        ));
    } else {
        lines.push(format!(
            "  {:>16}:   {} ({})",
            "Server".dimmed(),
            Colors::bold(&result.server.sponsor, theme),
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
        lines.push(format!("  {:>16}:   {ip}", "Client IP".dimmed()));
    }

    lines.join("\n")
}

pub fn format_connection_info(result: &TestResult, nc: bool, theme: Theme) {
    eprintln!("{}", build_connection_info(result, nc, theme));
}

#[must_use]
pub fn build_test_summary(
    dl_bytes: u64,
    ul_bytes: u64,
    dl_duration: f64,
    ul_duration: f64,
    nc: bool,
) -> String {
    let mut lines = Vec::new();

    lines.push(section_header("TEST SUMMARY", nc));

    if dl_bytes > 0 {
        let size_val = common::format_data_size_tabular(dl_bytes, DATA_SIZE_WIDTH);
        let dur_val = common::format_duration_tabular(dl_duration, DURATION_WIDTH);
        let dur_display = if nc {
            dur_val
        } else {
            dur_val.dimmed().to_string()
        };
        lines.push(format!(
            "  {:>14}:   {size_val} in {dur_display}",
            "Download"
        ));
    }
    if ul_bytes > 0 {
        let size_val = common::format_data_size_tabular(ul_bytes, DATA_SIZE_WIDTH);
        let dur_val = common::format_duration_tabular(ul_duration, DURATION_WIDTH);
        let dur_display = if nc {
            dur_val
        } else {
            dur_val.dimmed().to_string()
        };
        lines.push(format!("  {:>14}:   {size_val} in {dur_display}", "Upload"));
    }
    let total = dl_bytes + ul_bytes;
    let total_dur = dl_duration + ul_duration;
    if total > 0 {
        let size_val = common::format_data_size_tabular(total, DATA_SIZE_WIDTH);
        let dur_val = common::format_duration_tabular(total_dur, DURATION_WIDTH);
        let size_display = if nc {
            size_val
        } else {
            size_val.bold().to_string()
        };
        let dur_display = if nc {
            dur_val
        } else {
            dur_val.dimmed().to_string()
        };
        lines.push(format!(
            "  {:>14}:   {size_display} in {dur_display}",
            "Total"
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

#[must_use]
pub fn build_footer(timestamp: &str, nc: bool, theme: Theme) -> String {
    if nc {
        format!("\n  Completed at: {timestamp}")
    } else {
        format!(
            "\n  {} {}",
            "Completed at:".dimmed(),
            Colors::muted(timestamp, theme),
        )
    }
}

pub fn format_footer(timestamp: &str, nc: bool, theme: Theme) {
    eprintln!("{}", build_footer(timestamp, nc, theme));
}

#[must_use]
pub fn build_elapsed_time(elapsed: std::time::Duration, nc: bool, theme: Theme) -> String {
    let secs = elapsed.as_secs_f64();
    let time_val = common::format_duration_tabular(secs, DURATION_WIDTH);
    if nc {
        format!("\n  Total time: {time_val}")
    } else {
        format!(
            "\n  {} {}",
            "Total time:".dimmed(),
            Colors::info(time_val.trim(), theme)
        )
    }
}

pub fn format_elapsed_time(elapsed: std::time::Duration, nc: bool, theme: Theme) {
    eprintln!("{}", build_elapsed_time(elapsed, nc, theme));
}

/// Format a list of available servers.
#[must_use]
pub fn build_list(servers: &[Server]) -> String {
    let nc = terminal::no_color();

    let (max_id_len, max_sponsor_len, max_name_len) =
        servers
            .iter()
            .fold((3, 7, 24), |(max_id, max_sponsor, max_name), s| {
                let name_len = s.name.len() + s.country.len() + 3;
                (
                    max_id.max(s.id.len()),
                    max_sponsor.max(s.sponsor.len()),
                    max_name.max(name_len),
                )
            });

    let idw = max_id_len.max(3);
    let sw = max_sponsor_len.max(7);
    let nw = max_name_len.max(24);

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
        let dist = common::format_distance(server.distance);
        if nc {
            lines.push(format!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
                server.id,
                server.sponsor,
                format!("{} ({})", server.name, server.country),
                dist,
            ));
        } else {
            lines.push(format!(
                "  {:<idw$}  {:<sw$}  {:<24}  {:>10}",
                server.id,
                server.sponsor.white().bold(),
                format!("{} ({})", server.name, server.country),
                dist.bright_black(),
            ));
        }
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
