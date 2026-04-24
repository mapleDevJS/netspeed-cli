//! Grouped, category-based internet usage scenario display.
//!
//! Replaces the flat usage checklist with a visually grouped TUI showing
//! 14 modern internet scenarios across 5 categories, with capacity bars,
//! simultaneous stream counts, and personalized recommendations.
//!
//! # Layout
//! ```text
//! ┌──────────────────── USAGE CAPABILITY ────────────────────┐
//! │  ┌─ COMMUNICATION ─────────────────────────────────┐   │
//! │  │ 📹 HD Video Calls         8 Mbps  [████████░░] 62× ✅ │
//! │  └──────────────────────────────────────────────────┘   │
//! │  ... (4 more categories)                                │
//! │  ──── SUMMARY ─────────────────────────────────────     │
//! │  Your 500 Mbps connection supports: ...                 │
//! │  ⚠️  Recommendation: ...                                │
//! └──────────────────────────────────────────────────────────┘
//! ```

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use owo_colors::OwoColorize;

use crate::terminal;

// ── Constants ────────────────────────────────────────────────────────────────

const BAR_WIDTH: usize = 10;
const NAME_WIDTH: usize = 26;
const LINE_WIDTH: usize = 86;

// ── Data Structures ──────────────────────────────────────────────────────────

/// A single usage scenario with bandwidth requirements.
pub struct UsageScenario {
    pub name: &'static str,
    pub required_mbps: f64,
    pub icon: &'static str,
    pub concurrent_label: &'static str,
}

/// A category grouping scenarios.
pub struct ScenarioCategory {
    pub name: &'static str,
    pub icon: &'static str,
    pub scenarios: &'static [UsageScenario],
}

/// Computed status for a single scenario.
pub struct ScenarioStatus {
    pub scenario: &'static UsageScenario,
    pub concurrent: u32,
    pub headroom_pct: f64,
    pub is_met: bool,
}

/// Overall headroom level for exit code determination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadroomLevel {
    Green,  // >50% headroom
    Yellow, // 20-50% headroom
    Red,    // <20% headroom
}

// ── Scenario Definitions ─────────────────────────────────────────────────────

static CAT_COMMUNICATION: ScenarioCategory = ScenarioCategory {
    name: "COMMUNICATION & COLLABORATION",
    icon: "💬",
    scenarios: &[
        UsageScenario {
            name: "HD Video Calls (Zoom/Teams+Share)",
            required_mbps: 8.0,
            icon: "📹",
            concurrent_label: "calls",
        },
        UsageScenario {
            name: "4K Video Calls (FaceTime/Meet)",
            required_mbps: 25.0,
            icon: "📹",
            concurrent_label: "calls",
        },
        UsageScenario {
            name: "VoIP + Encrypted VPN",
            required_mbps: 2.0,
            icon: "🔒",
            concurrent_label: "sessions",
        },
    ],
};

static CAT_STREAMING: ScenarioCategory = ScenarioCategory {
    name: "STREAMING & ENTERTAINMENT",
    icon: "🎬",
    scenarios: &[
        UsageScenario {
            name: "4K HDR Streaming (Netflix/Disney+)",
            required_mbps: 35.0,
            icon: "📺",
            concurrent_label: "streams",
        },
        UsageScenario {
            name: "Cloud Gaming (GeForce Now/Xbox)",
            required_mbps: 50.0,
            icon: "🎮",
            concurrent_label: "sessions",
        },
        UsageScenario {
            name: "Live Broadcast Upload (Twitch/YT)",
            required_mbps: 30.0,
            icon: "📡",
            concurrent_label: "streams",
        },
    ],
};

static CAT_PRODUCTIVITY: ScenarioCategory = ScenarioCategory {
    name: "WORK & PRODUCTIVITY",
    icon: "💼",
    scenarios: &[
        UsageScenario {
            name: "Cloud Sync Bulk Upload (Drive/Dropbox)",
            required_mbps: 50.0,
            icon: "☁️",
            concurrent_label: "syncs",
        },
        UsageScenario {
            name: "4K Video Upload (YouTube Creator)",
            required_mbps: 80.0,
            icon: "🎥",
            concurrent_label: "uploads",
        },
        UsageScenario {
            name: "Remote Desktop HD (Parsec/TeamViewer)",
            required_mbps: 30.0,
            icon: "🖥️",
            concurrent_label: "sessions",
        },
    ],
};

static CAT_SMART_HOME: ScenarioCategory = ScenarioCategory {
    name: "SMART HOME & IOT",
    icon: "🏠",
    scenarios: &[
        UsageScenario {
            name: "4x 1080p Security Cameras",
            required_mbps: 20.0,
            icon: "📷",
            concurrent_label: "arrays",
        },
        UsageScenario {
            name: "50+ IoT Devices Hub",
            required_mbps: 5.0,
            icon: "🔌",
            concurrent_label: "hubs",
        },
    ],
};

static CAT_NEXTGEN: ScenarioCategory = ScenarioCategory {
    name: "NEXT-GEN / HEAVY USAGE",
    icon: "🚀",
    scenarios: &[
        UsageScenario {
            name: "8K Streaming (YouTube 8K/AV1)",
            required_mbps: 100.0,
            icon: "🎬",
            concurrent_label: "streams",
        },
        UsageScenario {
            name: "VR/AR Streaming (Quest 3/Vision Pro)",
            required_mbps: 80.0,
            icon: "🥽",
            concurrent_label: "sessions",
        },
        UsageScenario {
            name: "AI Model Download (7-70GB LLM)",
            required_mbps: 200.0,
            icon: "🤖",
            concurrent_label: "downloads",
        },
        UsageScenario {
            name: "4x Simultaneous 4K Streams",
            required_mbps: 140.0,
            icon: "👨‍👩‍👧‍👦",
            concurrent_label: "households",
        },
    ],
};

/// All scenario categories in default display order.
const ALL_CATEGORIES: &[&ScenarioCategory] = &[
    &CAT_COMMUNICATION,
    &CAT_STREAMING,
    &CAT_PRODUCTIVITY,
    &CAT_SMART_HOME,
    &CAT_NEXTGEN,
];

/// Get all scenario categories.
#[must_use]
pub fn all_categories() -> &'static [&'static ScenarioCategory] {
    ALL_CATEGORIES
}

// ── Status Computation ───────────────────────────────────────────────────────

/// Compute status for all scenarios given download speed in Mbps.
#[must_use]
pub fn compute_all_statuses(dl_mbps: f64) -> Vec<Vec<ScenarioStatus>> {
    all_categories()
        .iter()
        .map(|cat| {
            cat.scenarios
                .iter()
                .map(|s| compute_scenario_status(dl_mbps, s))
                .collect()
        })
        .collect()
}

fn compute_scenario_status(dl_mbps: f64, scenario: &'static UsageScenario) -> ScenarioStatus {
    // Safe: dl_mbps/required_mbps is a small ratio; floor→u32 is bounded by
    // realistic bandwidth values (never approaching u32::MAX).
    let concurrent = if scenario.required_mbps > 0.0 {
        (dl_mbps / scenario.required_mbps)
            .floor()
            .clamp(0.0, f64::from(u32::MAX)) as u32
    } else {
        0
    };
    let headroom_pct = if scenario.required_mbps > 0.0 {
        ((dl_mbps - scenario.required_mbps) / scenario.required_mbps * 100.0).max(0.0)
    } else {
        100.0
    };
    let is_met = dl_mbps >= scenario.required_mbps;

    ScenarioStatus {
        scenario,
        concurrent,
        headroom_pct,
        is_met,
    }
}

/// Determine the worst headroom level across all statuses.
#[must_use]
pub fn worst_headroom_level(statuses: &[Vec<ScenarioStatus>]) -> HeadroomLevel {
    let mut worst = HeadroomLevel::Green;
    for cat in statuses {
        for s in cat {
            let level = headroom_level(s.headroom_pct);
            if level > worst {
                worst = level;
            }
        }
    }
    worst
}

fn headroom_level(pct: f64) -> HeadroomLevel {
    if pct > 50.0 {
        HeadroomLevel::Green
    } else if pct >= 20.0 {
        HeadroomLevel::Yellow
    } else {
        HeadroomLevel::Red
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

/// Render a capacity bar: [████████░░]
fn render_capacity_bar(headroom_pct: f64, is_met: bool, _nc: bool, minimal: bool) -> String {
    let fill = if is_met {
        ((headroom_pct / 100.0) * BAR_WIDTH as f64)
            .ceil()
            .min(BAR_WIDTH as f64) as usize
    } else {
        0
    };
    let empty = BAR_WIDTH.saturating_sub(fill);

    if minimal {
        format!("[{}{}]", "#".repeat(fill), "-".repeat(empty))
    } else if terminal::no_color() {
        format!("[{}{}]", "█".repeat(fill), "░".repeat(empty))
    } else {
        let bar_color = if headroom_pct > 50.0 {
            "green"
        } else if headroom_pct >= 20.0 {
            "yellow"
        } else {
            "red"
        };
        let filled = "█".repeat(fill);
        let empty_str = "░".repeat(empty);
        match bar_color {
            "green" => format!("[{}{}]", filled.green(), empty_str),
            "yellow" => format!("[{}{}]", filled.yellow(), empty_str),
            _ => format!("[{}{}]", filled.red().bold(), empty_str),
        }
    }
}

/// Render a status emoji/symbol.
fn render_status_symbol(headroom_pct: f64, is_met: bool) -> String {
    let hide_emoji = terminal::no_emoji();
    if !is_met {
        if hide_emoji { "FAIL" } else { "❌" }.to_string()
    } else if headroom_pct > 50.0 {
        if hide_emoji { "OK" } else { "✅" }.to_string()
    } else if headroom_pct >= 20.0 {
        if hide_emoji { "WARN" } else { "⚠️" }.to_string()
    } else {
        if hide_emoji { "LOW" } else { "🔴" }.to_string()
    }
}

/// Render a single scenario row.
fn render_scenario_row(status: &ScenarioStatus, nc: bool, minimal: bool) -> String {
    let s = status.scenario;
    let bar = render_capacity_bar(status.headroom_pct, status.is_met, nc, minimal);
    let symbol = render_status_symbol(status.headroom_pct, status.is_met);
    let concurrent = status.concurrent;

    let name_display = if minimal || terminal::no_emoji() {
        format!("{:<NAME_WIDTH$}", s.name)
    } else {
        format!("{} {:<NAME_WIDTH$}", s.icon, s.name)
    };

    let req_display = format!("{:>6.0} Mbps", s.required_mbps);

    // Build inner content (everything between the │ borders)
    let inner = if minimal || nc {
        format!("{name_display:<NAME_WIDTH$} {req_display}  {bar} {concurrent:>3}x {symbol:<5}",)
    } else {
        // Colorize the requirement based on whether it's met
        let req_colored = if status.is_met {
            req_display.dimmed().to_string()
        } else {
            req_display.red().bold().to_string()
        };
        format!("{name_display:<NAME_WIDTH$} {req_colored}  {bar} {concurrent:>3}x {symbol:<5}",)
    };

    // Right-pad to exactly LINE_WIDTH - 2 (1 space padding each side inside borders)
    let content_width = LINE_WIDTH - 2;
    let padded = format!("{inner:<content_width$}");
    format!("  │ {padded} │")
}

/// Render a category box header.
fn render_category_header(cat: &ScenarioCategory, nc: bool, minimal: bool) -> String {
    let content_width = LINE_WIDTH - 2; // 1 space padding each side
    let title = format!(" {} {} ", cat.icon, cat.name);
    let dashes = "─".repeat(content_width.saturating_sub(title.len()));
    let inner = format!("{title}{dashes}");
    // Right-pad to exactly content_width so the closing │ always aligns
    let padded = format!("{inner:<content_width$}");
    if minimal || nc {
        format!("  │ {padded} │")
    } else {
        // For colored output, compute the inner content length, then pad
        let inner_len = title.len() + dashes.len();
        let pad = " ".repeat(content_width.saturating_sub(inner_len));
        format!("  │ {}{}{pad} │", title.cyan().bold(), dashes.dimmed())
    }
}

/// Render a category box (header + rows + footer).
fn render_category_box(
    cat: &ScenarioCategory,
    statuses: &[ScenarioStatus],
    nc: bool,
    minimal: bool,
) -> String {
    let mut lines = Vec::new();
    let content_width = LINE_WIDTH - 2; // 1 space padding each side

    // Top border — use │ for interior lines (not ┌┐)
    if minimal || nc {
        lines.push(format!("  │ {:-<content_width$} │", ""));
    } else {
        lines.push(format!("  │ {} │", "─".repeat(content_width).dimmed()));
    }

    // Category header
    lines.push(render_category_header(cat, nc, minimal));

    // Scenario rows
    for status in statuses {
        lines.push(render_scenario_row(status, nc, minimal));
    }

    // Bottom border — use │ for interior lines (not └┘)
    if minimal || nc {
        lines.push(format!("  │ {:-<content_width$} │", ""));
    } else {
        lines.push(format!("  │ {} │", "─".repeat(content_width).dimmed()));
    }

    lines.join("\n")
}

/// Render the overall section header.
fn render_section_header(dl_mbps: f64, nc: bool, minimal: bool) -> String {
    let title = format!(" USAGE CAPABILITY — {dl_mbps:.0} Mbps ");
    let left = (LINE_WIDTH.saturating_sub(title.len())) / 2;
    let right = LINE_WIDTH.saturating_sub(left).saturating_sub(title.len());
    if minimal || nc {
        format!("  +{:─<left$}{}{:─<right$}+", "", title, "")
    } else {
        format!(
            "  {}{}{}{}{}",
            "┌".dimmed(),
            "─".repeat(left).dimmed(),
            title.cyan().bold(),
            "─".repeat(right).dimmed(),
            "┐".dimmed(),
        )
    }
}

/// Render the section footer.
fn render_section_footer(nc: bool, minimal: bool) -> String {
    if minimal || nc {
        format!("  +{:─<LINE_WIDTH$}+", "")
    } else {
        format!("  {}{:─<LINE_WIDTH$}{}", "└".dimmed(), "", "┘".dimmed(),)
    }
}

/// Render the summary section.
fn render_summary(
    statuses: &[Vec<ScenarioStatus>],
    dl_mbps: f64,
    nc: bool,
    minimal: bool,
) -> String {
    let mut lines = Vec::new();

    if minimal || nc {
        lines.push(String::new());
        lines.push(
            "  ---- SUMMARY --------------------------------------------------------".to_string(),
        );
    } else {
        lines.push(String::new());
        lines.push(format!(
            "  {}",
            "──── SUMMARY ────────────────────────────────────────────────".dimmed()
        ));
    }

    lines.push(format!("  Your {dl_mbps:.0} Mbps connection supports:"));

    // Find most notable concurrent counts
    let mut highlights: Vec<(&'static UsageScenario, u32)> = Vec::new();
    for cat in statuses {
        for s in cat {
            if s.concurrent > 0 {
                highlights.push((s.scenario, s.concurrent));
            }
        }
    }
    highlights.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

    // Show top items
    for (scenario, count) in highlights.iter().take(5) {
        if minimal || nc {
            lines.push(format!(
                "    - {:>3}x {} {}",
                count, scenario.name, scenario.concurrent_label,
            ));
        } else {
            lines.push(format!(
                "    {} {:>3}x {} {}",
                "•".cyan(),
                count.to_string().green().bold(),
                scenario.name,
                scenario.concurrent_label.dimmed(),
            ));
        }
    }

    lines.join("\n")
}

/// Render the recommendation footer.
fn render_recommendation(
    statuses: &[Vec<ScenarioStatus>],
    _dl_mbps: f64,
    nc: bool,
    minimal: bool,
) -> String {
    // Find the scenario with the worst headroom that is at least partially met
    let mut worst: Option<&ScenarioStatus> = None;
    for cat in statuses {
        for s in cat {
            if s.is_met {
                match worst {
                    None => worst = Some(s),
                    Some(w) if s.headroom_pct < w.headroom_pct => worst = Some(s),
                    _ => {}
                }
            }
        }
    }

    let Some(worst_s) = worst else {
        // Nothing is met — recommend upgrade
        let mut lines = Vec::new();
        lines.push(String::new());
        if minimal || nc {
            lines.push("  [!] Your connection speed is insufficient for modern usage.".to_string());
            lines.push("      Consider upgrading to at least 100 Mbps.".to_string());
        } else {
            lines.push(format!(
                "  {} {}",
                "⚠️".red().bold(),
                "Your connection speed is insufficient for modern usage."
                    .red()
                    .bold(),
            ));
            lines.push(format!(
                "      {} to at least 100 Mbps.",
                "Consider upgrading".bright_black(),
            ));
        }
        return lines.join("\n");
    };

    let s = worst_s.scenario;
    // Safe: required_mbps is small (≤200), *3 → ≤600, fits u32.
    let recommended = (s.required_mbps * 3.0)
        .ceil()
        .clamp(0.0, f64::from(u32::MAX)) as u32; // 3x headroom target

    let mut lines = Vec::new();
    lines.push(String::new());

    if minimal || nc {
        lines.push(format!(
            "  [!] {} has limited headroom at {:.0}%.",
            s.name, worst_s.headroom_pct,
        ));
        lines.push(format!(
            "      Consider upgrading to {recommended}+ Mbps for better performance.",
        ));
    } else {
        let warning_icon = if worst_s.headroom_pct < 20.0 {
            "🔴"
        } else {
            "⚠️"
        };
        lines.push(format!(
            "  {} {} {} has limited headroom at {:.0}%.",
            warning_icon,
            "Recommendation:".yellow().bold(),
            s.name.yellow(),
            worst_s.headroom_pct,
        ));
        lines.push(format!(
            "      {} to {}+ Mbps for better performance.",
            "Consider upgrading".bright_black(),
            recommended.to_string().cyan().bold(),
        ));
    }

    lines.join("\n")
}

/// Format the full scenario grid output.
#[must_use]
pub fn format_scenario_grid(dl_mbps: f64, nc: bool, minimal: bool) -> String {
    let statuses = compute_all_statuses(dl_mbps);
    let mut lines = Vec::new();

    // Opening
    lines.push(String::new());
    lines.push(render_section_header(dl_mbps, nc, minimal));
    lines.push(String::new());

    // Category boxes
    for (i, cat) in all_categories().iter().enumerate() {
        if i > 0 {
            lines.push(String::new());
        }
        lines.push(render_category_box(cat, &statuses[i], nc, minimal));
    }

    lines.push(String::new());

    // Closing
    lines.push(render_section_footer(nc, minimal));

    // Summary
    lines.push(render_summary(&statuses, dl_mbps, nc, minimal));

    // Recommendation
    lines.push(render_recommendation(&statuses, 0.0, nc, minimal));

    lines.push(String::new());

    lines.join("\n")
}

/// Format the scenario output, printing to stderr.
pub fn print_scenario_grid(dl_mbps: f64, minimal: bool) {
    let nc = terminal::no_color() || minimal;
    let output = format_scenario_grid(dl_mbps, nc, minimal);
    eprintln!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_scenario_status_met() {
        let s = &CAT_COMMUNICATION.scenarios[0]; // 8 Mbps
        let status = compute_scenario_status(100.0, s);
        assert!(status.is_met);
        assert_eq!(status.concurrent, 12); // 100/8 = 12.5 -> 12
        assert!((status.headroom_pct - 1150.0).abs() < 1.0); // (100-8)/8*100 = 1150%
    }

    #[test]
    fn test_compute_scenario_status_not_met() {
        let s = &CAT_NEXTGEN.scenarios[2]; // 200 Mbps
        let status = compute_scenario_status(50.0, s);
        assert!(!status.is_met);
        assert_eq!(status.concurrent, 0);
        assert!((status.headroom_pct - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_headroom_level() {
        assert_eq!(headroom_level(80.0), HeadroomLevel::Green);
        assert_eq!(headroom_level(50.1), HeadroomLevel::Green);
        assert_eq!(headroom_level(50.0), HeadroomLevel::Yellow);
        assert_eq!(headroom_level(20.0), HeadroomLevel::Yellow);
        assert_eq!(headroom_level(19.9), HeadroomLevel::Red);
        assert_eq!(headroom_level(0.0), HeadroomLevel::Red);
    }

    #[test]
    fn test_worst_headroom_level_all_green() {
        let statuses = compute_all_statuses(1000.0); // Very fast connection
        let worst = worst_headroom_level(&statuses);
        assert_eq!(worst, HeadroomLevel::Green);
    }

    #[test]
    fn test_worst_headroom_level_some_red() {
        let statuses = compute_all_statuses(50.0); // Moderate connection
        let worst = worst_headroom_level(&statuses);
        // AI Model Download at 200 Mbps won't be met -> red
        assert_eq!(worst, HeadroomLevel::Red);
    }

    #[test]
    fn test_render_capacity_bar_full() {
        let bar = render_capacity_bar(100.0, true, false, false);
        assert!(bar.contains("##########") || bar.contains("██████████"));
    }

    #[test]
    fn test_render_capacity_bar_empty() {
        let bar = render_capacity_bar(0.0, false, false, false);
        assert!(bar.contains("----------") || bar.contains("░░░░░░░░░░"));
    }

    #[test]
    fn test_render_capacity_bar_half() {
        let bar = render_capacity_bar(50.0, true, false, false);
        // 50% of 10 = 5 filled
        assert!(bar.contains("#####") || bar.contains("█████"));
    }

    #[test]
    fn test_format_scenario_grid_contains_header() {
        let output = format_scenario_grid(500.0, true, false);
        assert!(output.contains("USAGE CAPABILITY"));
        assert!(output.contains("COMMUNICATION"));
        assert!(output.contains("STREAMING"));
    }

    #[test]
    fn test_format_scenario_grid_contains_summary() {
        let output = format_scenario_grid(500.0, true, false);
        assert!(output.contains("SUMMARY"));
        assert!(output.contains("500 Mbps"));
    }

    #[test]
    fn test_all_categories_count() {
        let cats = all_categories();
        assert_eq!(cats.len(), 5);
        let total_scenarios: usize = cats.iter().map(|c| c.scenarios.len()).sum();
        assert_eq!(total_scenarios, 15);
    }

    #[test]
    fn test_recommendation_for_fast_connection() {
        // At 500 Mbps all scenarios are met with good headroom
        let output = format_scenario_grid(500.0, true, false);
        assert!(output.contains("SUMMARY"));
        assert!(output.contains("500 Mbps"));
    }

    #[test]
    fn test_recommendation_for_moderate_connection() {
        // At 100 Mbps, AI Model Download (200 Mbps) won't be met → triggers recommendation
        let output = format_scenario_grid(100.0, true, false);
        assert!(
            output.contains("Recommendation")
                || output.contains("recommend")
                || output.contains("100 Mbps")
        );
    }

    #[test]
    fn test_recommendation_for_slow_connection() {
        let output = format_scenario_grid(5.0, true, false);
        assert!(output.contains("insufficient") || output.contains("limited"));
    }
}
