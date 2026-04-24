use crate::formatter::grades;
use crate::theme::{Colors, Theme};
use crate::types::TestResult;
use owo_colors::OwoColorize;

use super::helpers::{
    bufferbloat_info, mini_bar, severity_icon, stability_label, Summary, LINE_WIDTH,
};

// ── Tabular Column Widths ────────────────────────────────────────────────────
const SPEED_TAB_WIDTH: usize = 12; // "  150.00 Mbps"
const LATENCY_TAB_WIDTH: usize = 12; // "    12.1 ms"

/// Build the rounded header with server info.
pub fn build_header(result: &TestResult, nc: bool, theme: Theme) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let title = format!(" NetSpeed CLI v{version} ");
    let half_pad = (LINE_WIDTH.saturating_sub(title.len())) / 2;
    let left_pad = "─".repeat(half_pad);
    let right_pad = "─".repeat(
        LINE_WIDTH
            .saturating_sub(half_pad)
            .saturating_sub(title.len()),
    );

    let subtitle = format!(
        "  {} ({} {}) • {} • {}",
        result.server.sponsor,
        result.server.name,
        result.server.country,
        crate::common::format_distance(result.server.distance),
        result.client_ip.as_deref().unwrap_or("unknown"),
    );

    let mut lines = Vec::new();
    if nc {
        lines.push(format!("╭{left_pad}{title}{right_pad}╮"));
    } else {
        lines.push(format!(
            "╭{}{}{}╮",
            Colors::dimmed(&left_pad, theme),
            Colors::header(title.trim(), theme),
            Colors::dimmed(&right_pad, theme),
        ));
    }

    if nc {
        lines.push(format!("│{subtitle:<LINE_WIDTH$}│"));
    } else {
        lines.push(format!("│{subtitle:<LINE_WIDTH$}│").dimmed().to_string());
    }

    let footer_line = "── Bandwidth test · speedtest.net ──";
    let footer_pad = LINE_WIDTH.saturating_sub(footer_line.len());
    if nc {
        lines.push(format!("╰{footer_line}{:─<footer_pad$}╯", ""));
    } else {
        lines.push(format!(
            "╰{}{:─<footer_pad$}╯",
            Colors::dimmed(footer_line, theme),
            ""
        ));
    }

    lines.join("\n")
}

/// Build the 3-column metrics dashboard.
pub fn build_metrics_dashboard(
    result: &TestResult,
    summary: &Summary,
    nc: bool,
    theme: Theme,
) -> String {
    let profile = summary.profile;
    let mut lines = Vec::new();

    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        profile,
    );
    let dl_grade = result
        .download
        .map(|d| grades::grade_download(d / 1_000_000.0, profile));
    let ul_grade = result
        .upload
        .map(|u| grades::grade_upload(u / 1_000_000.0, profile));
    let _ping_grade = result.ping.map(|p| grades::grade_ping(p, profile));

    let dl_cv = result.download_samples.as_ref().map(|s| compute_cv(s));
    let ul_cv = result.upload_samples.as_ref().map(|s| compute_cv(s));

    let bb_info =
        if let (Some(lat_dl), Some(lat_ul)) = (result.latency_download, result.latency_upload) {
            Some(bufferbloat_info(lat_dl.max(lat_ul), result.ping, nc, theme))
        } else {
            None
        };

    let col_w = 19;
    let header = if nc {
        format!(
            "┌{:<col_w$}┬{:<col_w$}┬{:<col_w$}┐",
            " PERFORMANCE ", " STABILITY ", " BUFFERBLOAT ",
        )
    } else {
        format!(
            "┌{:<col_w$}┬{:<col_w$}┬{:<col_w$}┐",
            Colors::header("PERFORMANCE", theme),
            Colors::header("STABILITY", theme),
            Colors::header("BUFFERBLOAT", theme),
        )
    };
    lines.push(header);

    let dl_color = result
        .download
        .map_or("dimmed", |d| speed_color(d / 1_000_000.0));

    let dl_speed = result.download.map_or_else(
        || "—".to_string(),
        |d| {
            let mbps = d / 1_000_000.0;
            crate::common::tabular_number(mbps, SPEED_TAB_WIDTH, 0)
        },
    );

    let dl_stab = if let (Some(cv), Some(_grade)) = (dl_cv, dl_grade) {
        let (g, lbl) = stability_label(cv, nc, theme);
        format!("DL: {g} {lbl}")
    } else {
        "DL: —".to_string()
    };

    let bb_col = if let Some((grade, added)) = &bb_info {
        format!("Grade: {grade}   {added}")
    } else {
        "N/A".to_string()
    };

    if nc {
        lines.push(format!(
            "│ {:<col_w$}│ {:<col_w$}│ {:<col_w$}│",
            format!("{} ↓", dl_speed),
            dl_stab,
            bb_col,
        ));
    } else {
        let dl_display = match dl_color {
            "blue" => format!("{} ↓", Colors::info(dl_speed.trim(), theme)),
            "bright_green" => format!("{} ↓", Colors::good(dl_speed.trim(), theme)),
            "yellow" => format!("{} ↓", Colors::warn(dl_speed.trim(), theme)),
            _ => format!("{} ↓", Colors::bad(dl_speed.trim(), theme)),
        };
        lines.push(format!("│ {dl_display} │ {dl_stab} │ {bb_col} │",));
    }

    let ul_color = result
        .upload
        .map_or("dimmed", |u| speed_color(u / 1_000_000.0));

    let ul_speed = result.upload.map_or_else(
        || "—".to_string(),
        |u| {
            let mbps = u / 1_000_000.0;
            crate::common::tabular_number(mbps, SPEED_TAB_WIDTH, 0)
        },
    );

    let ul_stab = if let (Some(cv), Some(_grade)) = (ul_cv, ul_grade) {
        let (g, lbl) = stability_label(cv, nc, theme);
        format!("UL: {g} {lbl}")
    } else {
        "UL: —".to_string()
    };

    if nc {
        lines.push(format!(
            "│ {:<col_w$}│ {:<col_w$}│ {:<col_w$}│",
            format!("{} ↑", ul_speed),
            ul_stab,
            "",
        ));
    } else {
        let ul_display = match ul_color {
            "blue" => format!("{} ↑", Colors::info(ul_speed.trim(), theme)),
            "bright_green" => format!("{} ↑", Colors::good(ul_speed.trim(), theme)),
            "yellow" => format!("{} ↑", Colors::warn(ul_speed.trim(), theme)),
            _ => format!("{} ↑", Colors::bad(ul_speed.trim(), theme)),
        };
        lines.push(format!("│ {} │ {} │ {:<col_w$}│", ul_display, ul_stab, "",));
    }

    let latency_str = result.ping.map_or_else(
        || "—".to_string(),
        |p| crate::common::format_latency_tabular(p, LATENCY_TAB_WIDTH),
    );

    let overall_display = if nc {
        format!("Overall: [{}]", overall_grade.as_str())
    } else {
        format!(
            "{} {}",
            "Overall:".dimmed(),
            overall_grade.color_str(nc, theme)
        )
    };

    if nc {
        lines.push(format!(
            "│ {:<col_w$}│ {:<col_w$}│ {:<col_w$}│",
            latency_str, "", overall_display,
        ));
    } else {
        let lat_display = if let Some(p) = result.ping {
            let lat_val = crate::common::format_latency_tabular(p, LATENCY_TAB_WIDTH);
            format!("{} {}", "🟢", Colors::info(lat_val.trim(), theme))
        } else {
            "—".dimmed().to_string()
        };
        lines.push(format!(
            "│ {} │ {:<col_w$}│ {} │",
            lat_display, "", overall_display,
        ));
    }

    lines.push(format!("└{:─<col_w$}┴{:─<col_w$}┴{:─<col_w$}┘", "", "", ""));

    lines.join("\n")
}

/// Get color name for a speed value.
fn speed_color(mbps: f64) -> &'static str {
    if mbps >= 200.0 {
        "bright_green"
    } else if mbps >= 100.0 {
        "blue"
    } else if mbps >= 50.0 {
        "yellow"
    } else {
        "red"
    }
}

/// Build the compact capability matrix.
pub fn build_capability_matrix(dl_mbps: f64, nc: bool, theme: Theme) -> String {
    let mut lines = Vec::new();

    lines.push(String::new());
    if nc {
        lines.push(format!("CAPABILITY MATRIX ({dl_mbps:.0} Mbps)"));
    } else {
        lines.push(Colors::header(
            &format!("CAPABILITY MATRIX ({dl_mbps:.0} Mbps)"),
            theme,
        ));
    }
    lines.push("─".repeat(LINE_WIDTH));

    // Tiered scenarios: name, required range, icon
    let tiers: &[(&str, f64, f64, &str)] = &[
        ("Communication", 8.0, 25.0, "💬"),
        ("Streaming", 30.0, 50.0, "🎬"),
        ("Cloud Work", 50.0, 80.0, "☁️"),
        ("Security Cams", 20.0, 20.0, "📷"),
        ("AI/ML Downloads", 200.0, 200.0, "🤖"),
    ];

    for (name, min_mbps, max_mbps, _icon) in tiers {
        let concurrent_min = if *min_mbps > 0.0 {
            (dl_mbps / min_mbps).floor() as u32
        } else {
            0
        };
        let concurrent_max = if *max_mbps > 0.0 {
            (dl_mbps / max_mbps).floor() as u32
        } else {
            0
        };
        let is_met = dl_mbps >= *min_mbps;
        let headroom = if dl_mbps >= *min_mbps {
            ((dl_mbps - min_mbps) / min_mbps * 100.0).max(0.0)
        } else {
            0.0
        };

        let fill = if is_met {
            ((headroom / 100.0) * 10.0).ceil().min(10.0) as usize
        } else {
            0
        };
        let bar = mini_bar(fill, 10, nc, headroom);
        let range_str = if (*min_mbps - *max_mbps).abs() < f64::EPSILON {
            format!("{min_mbps:.0} Mbps")
        } else {
            format!("{min_mbps:.0}-{max_mbps:.0} Mbps")
        };

        let concurrent_display = if concurrent_min == concurrent_max {
            format!("{concurrent_min:>3}x")
        } else {
            format!("{concurrent_min:>3}x-{concurrent_max:>2}x")
        };

        let (icon, _color) = severity_icon(headroom, is_met);

        // Only show warnings for constrained tiers, otherwise just show the line
        if nc {
            lines.push(format!(
                "  {name:<18} {range_str:>12}  {bar}  {concurrent_display}  {icon}"
            ));
        } else {
            lines.push(format!(
                "  {:<18} {}  {}  {}  {}",
                name.dimmed(),
                range_str,
                bar,
                concurrent_display,
                icon,
            ));
        }
    }

    lines.push("─".repeat(LINE_WIDTH));

    // Warning recommendation for worst constrained tier
    let mut worst: Option<(&str, f64)> = None;
    for (name, min_mbps, max_mbps, _) in tiers {
        let headroom = if dl_mbps >= *min_mbps {
            (dl_mbps - min_mbps) / min_mbps * 100.0
        } else {
            -100.0
        };
        if dl_mbps >= *min_mbps && headroom < 50.0 {
            match worst {
                None => worst = Some((name, *max_mbps)),
                Some((_, w_max)) if max_mbps > &w_max => worst = Some((name, *max_mbps)),
                _ => {}
            }
        }
    }

    if let Some((name, target)) = worst {
        let recommended = (target * 3.0).ceil() as u32;
        if nc {
            lines.push(format!(
                "  ⚠️  {}: Upgrade to {}+ Mbps for better {} performance.",
                "Recommendation", recommended, name
            ));
        } else {
            lines.push(format!(
                "  {} {} Upgrade to {}+ Mbps for better {} performance.",
                Colors::warn("⚠️", theme),
                Colors::warn("Recommendation:", theme),
                Colors::info(&recommended.to_string(), theme),
                name,
            ));
        }
    }

    lines.join("\n")
}

/// Build two-column transfer estimates.
pub fn build_transfer_estimates(dl_mbps: f64, nc: bool, theme: Theme) -> String {
    if dl_mbps <= 0.0 {
        return String::new();
    }

    let dl_bytes_per_sec = dl_mbps * 1_000_000.0 / 8.0;

    let files: &[(&str, u64)] = &[
        ("5 MB (MP3/Photo)", 5 * 1024 * 1024),
        ("100 MB (App)", 100 * 1024 * 1024),
        ("4 GB (HD Movie)", 4 * 1024 * 1024 * 1024),
        ("50 GB (Game)", 50 * 1024 * 1024 * 1024),
    ];

    let mut lines = Vec::new();
    lines.push(String::new());

    if nc {
        lines.push(format!("TRANSFER ESTIMATES @ {dl_mbps:.0} Mbps"));
    } else {
        lines.push(Colors::header(
            &format!("TRANSFER ESTIMATES @ {dl_mbps:.0} Mbps"),
            theme,
        ));
    }
    lines.push("─".repeat(LINE_WIDTH));

    // Two-column layout
    let mid = files.len().div_ceil(2);
    for i in 0..mid {
        let left = &files[i];
        let left_secs = left.1 as f64 / dl_bytes_per_sec;
        let left_time = format_time_short(left_secs);

        if i + mid < files.len() {
            let right = &files[i + mid];
            let right_secs = right.1 as f64 / dl_bytes_per_sec;
            let right_time = format_time_short(right_secs);

            if nc {
                lines.push(format!(
                    "  {:<22} → {:<8} │  {:<20} → {}",
                    left.0, left_time, right.0, right_time
                ));
            } else {
                lines.push(format!(
                    "  {} {} {} {} {} {}",
                    Colors::dimmed(left.0, theme),
                    "→".dimmed(),
                    Colors::good(&left_time, theme),
                    "│".dimmed(),
                    Colors::dimmed(right.0, theme),
                    Colors::good(&format!("→ {right_time}"), theme),
                ));
            }
        } else if nc {
            lines.push(format!("  {:<22} → {}", left.0, left_time));
        } else {
            lines.push(format!(
                "  {} {} {}",
                Colors::dimmed(left.0, theme),
                "→".dimmed(),
                Colors::good(&left_time, theme),
            ));
        }
    }

    lines.join("\n")
}

pub fn format_time_short(secs: f64) -> String {
    if secs < 1.0 {
        format!("{secs:.1}s")
    } else if secs < 60.0 {
        format!("{secs:.0}s")
    } else if secs < 3600.0 {
        format!("{}m {:02}s", secs as u64 / 60, (secs % 60.0) as u64)
    } else {
        format!(
            "{}h {:02}m",
            secs as u64 / 3600,
            ((secs % 3600.0) / 60.0) as u64
        )
    }
}

/// Compute coefficient of variation.
pub fn compute_cv(samples: &[f64]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    if mean == 0.0 {
        return 0.0;
    }
    let variance =
        samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
    (variance.sqrt() / mean) * 100.0
}
