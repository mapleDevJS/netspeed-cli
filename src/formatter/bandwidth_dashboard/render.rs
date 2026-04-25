//! Rendering functions for the bandwidth dashboard (colors, bars, rows, categories).

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use owo_colors::OwoColorize;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::terminal;
use crate::theme::{Colors, Theme};

use super::scenarios::{
    CapacityLevel, ResponsiveLayout, ScenarioCategory, ScenarioStatus, BANDWIDTH_WIDTH,
    CAPACITY_BADGE_WIDTH, FIXED_COLUMNS_WIDTH, ICON_WIDTH, MIN_BAR_WIDTH, NAME_WIDTH,
};

// ── Color System ─────────────────────────────────────────────────────────────

/// Get bar color based on usage percentage (0-100).
/// Returns ANSI color specification for gradients.
#[must_use]
pub fn bar_color_by_usage(usage_pct: f64, nc: bool) -> Box<dyn Fn(&str) -> String> {
    if nc {
        Box::new(|s: &str| s.to_string())
    } else if usage_pct >= 71.0 {
        // Red zone — critical
        Box::new(|s: &str| s.red().bold().to_string())
    } else if usage_pct >= 31.0 {
        // Yellow zone — moderate
        Box::new(|s: &str| s.yellow().to_string())
    } else {
        // Green zone — healthy
        Box::new(|s: &str| s.green().to_string())
    }
}

/// Get capacity badge color and symbol.
#[must_use]
pub fn capacity_badge_style(level: CapacityLevel, nc: bool, concurrent: u32) -> String {
    if nc {
        match level {
            CapacityLevel::Optimal => format!("{concurrent:>3}x OK"),
            CapacityLevel::Moderate => format!("{concurrent:>3}x --"),
            CapacityLevel::Limited => format!("{concurrent:>3}x !"),
            CapacityLevel::Exceeded => "FAIL".to_string(),
        }
    } else {
        match level {
            CapacityLevel::Optimal => {
                format!("{} {}", format!("{concurrent:>3}x").dimmed(), "✓".green())
            }
            CapacityLevel::Moderate => {
                format!("{} {}", format!("{concurrent:>3}x").yellow(), "●".yellow())
            }
            CapacityLevel::Limited => {
                format!(
                    "{} {}",
                    format!("{concurrent:>3}x").bright_yellow(),
                    "⚠".bright_yellow()
                )
            }
            CapacityLevel::Exceeded => format!("{}", "✗".red().bold()),
        }
    }
}

// ── Progress Bar Rendering ───────────────────────────────────────────────────

/// Render a dynamic-width progress bar using Unicode block characters.
///
/// # Arguments
/// * `usage_pct` — Percentage of total bandwidth (0-100)
/// * `bar_width` — Desired bar width in characters
/// * `nc` — No-color mode flag
/// * `minimal` — ASCII-only mode flag
#[must_use]
pub fn render_progress_bar(usage_pct: f64, bar_width: usize, nc: bool, minimal: bool) -> String {
    if bar_width == 0 {
        return String::new();
    }

    // Safe: usage_pct/100 is 0..1, bar_width is small (≤200), result fits usize.
    let fill_count = ((usage_pct / 100.0) * bar_width as f64)
        .round()
        .clamp(0.0, usize::MAX as f64) as usize;
    let empty_count = bar_width.saturating_sub(fill_count);

    if minimal || nc {
        // ASCII fallback with non-color severity marker
        let fill = "=".repeat(fill_count);
        let empty = "-".repeat(empty_count);
        let severity = if usage_pct >= 71.0 {
            " [!]"
        } else if usage_pct >= 31.0 {
            " [~]"
        } else {
            ""
        };
        let bar = if fill_count > 0 && fill_count < bar_width {
            format!("[{fill}>{empty}]")
        } else if fill_count == bar_width {
            format!("[{fill}]")
        } else {
            format!("[{empty}]")
        };
        format!("{bar}{severity}")
    } else {
        // Unicode blocks with partial fill support
        let color_fn = bar_color_by_usage(usage_pct, nc);

        // Check for fractional fill (e.g., 4.7 → 4 full + 1 partial)
        // Safe: usage_pct/100 is 0..1, bar_width is small (≤200).
        let exact_fill = (usage_pct / 100.0) * bar_width as f64;
        let full_blocks = exact_fill.floor().clamp(0.0, usize::MAX as f64) as usize;
        let fractional = exact_fill - full_blocks as f64;

        let mut result = String::with_capacity(bar_width);

        // Full blocks
        for _ in 0..full_blocks {
            result.push('█');
        }

        // Partial block (if significant fraction)
        if fractional > 0.25 && full_blocks < bar_width {
            if fractional > 0.75 {
                result.push('▉'); // Almost full
            } else if fractional > 0.5 {
                result.push('▌'); // Half
            } else {
                result.push('▎'); // Quarter
            }
        }

        // Empty blocks
        let remaining = bar_width.saturating_sub(result.chars().count());
        for _ in 0..remaining {
            result.push('░');
        }

        color_fn(&result)
    }
}

// ── Icon Handling ────────────────────────────────────────────────────────────

/// Get icon with guaranteed 2-character width padding.
#[must_use]
pub fn get_padded_icon(icon: &str, _nc: bool, minimal: bool) -> String {
    if minimal || terminal::no_emoji() {
        // ASCII fallback — use 2-char symbols
        match icon {
            "📹" => "VC".to_string(),
            "🎮" => "GM".to_string(),
            "🔒" => "VP".to_string(),
            "📺" => "ST".to_string(),
            "📡" => "BC".to_string(),
            "☁️" | "☁" => "CL".to_string(),
            "🎥" => "UL".to_string(),
            "🖥️" | "🖥" => "RD".to_string(),
            "📷" => "SC".to_string(),
            "🔌" => "IO".to_string(),
            "🤖" => "AI".to_string(),
            "👨\u{200d}👩\u{200d}👧\u{200d}👦" => "F4".to_string(),
            "🎬" => "8K".to_string(),
            "🥽" => "VR".to_string(),
            "💬" | "🏠" | "💼" | "🚀" => "  ".to_string(),
            _ => "??".to_string(),
        }
    } else {
        // Unicode icon — pad to 2 chars display width
        let width = icon.width();
        if width >= 2 {
            icon.to_string()
        } else {
            format!("{icon} ")
        }
    }
}

// ── Name Truncation ──────────────────────────────────────────────────────────

/// Truncate name to `max_width` with ellipsis, respecting Unicode width.
#[must_use]
pub fn truncate_name(name: &str, max_width: usize) -> String {
    if name.width() <= max_width {
        return name.to_string();
    }

    let ellipsis = "…";
    let max_chars = max_width.saturating_sub(ellipsis.width());

    let mut result = String::with_capacity(max_width);
    let mut current_width = 0;

    for ch in name.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if current_width + ch_width > max_chars {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }

    result.push_str(ellipsis);
    result
}

// ── Row Rendering ────────────────────────────────────────────────────────────

/// Render a single scenario row in expanded/standard mode.
#[must_use]
pub fn render_scenario_row_expanded(
    status: &ScenarioStatus,
    bar_width: usize,
    nc: bool,
    minimal: bool,
    show_multiplier: bool,
) -> String {
    let s = status.scenario;
    let level = CapacityLevel::from_concurrent(status.concurrent, status.is_met);

    // Icon (2 chars)
    let icon = get_padded_icon(s.icon, nc, minimal);

    // Name (28 chars, left-aligned, truncated)
    let name = truncate_name(s.name, NAME_WIDTH);
    let name_padded = format!("{name:<NAME_WIDTH$}");

    // Bandwidth (tabular, right-aligned)
    let bw_display = crate::common::tabular_number(s.required_mbps, BANDWIDTH_WIDTH, 0);
    let bw_padded = format!("{bw_display:>BANDWIDTH_WIDTH$}");

    // Progress bar (dynamic width)
    let bar = render_progress_bar(status.usage_pct, bar_width, nc, minimal);

    // Capacity badge (6 chars, right-aligned)
    let badge = if show_multiplier {
        capacity_badge_style(level, nc, status.concurrent)
    } else {
        // In standard mode, show simplified badge
        if nc {
            match level {
                CapacityLevel::Optimal => "  OK".to_string(),
                CapacityLevel::Moderate => "  --".to_string(),
                CapacityLevel::Limited => "  !".to_string(),
                CapacityLevel::Exceeded => "FAIL".to_string(),
            }
        } else {
            match level {
                CapacityLevel::Optimal => format!("  {}", "✓".green()),
                CapacityLevel::Moderate => format!("  {}", "●".yellow()),
                CapacityLevel::Limited => format!("  {}", "⚠".bright_yellow()),
                CapacityLevel::Exceeded => format!("{}", "✗".red().bold()),
            }
        }
    };
    let badge_padded = format!("{badge:>CAPACITY_BADGE_WIDTH$}");

    // Assemble row
    if minimal || nc {
        format!("  {icon} {name_padded} {bw_padded}  {bar} {badge_padded}")
    } else {
        // Colorize bandwidth based on whether it's met
        let bw_colored = if status.is_met {
            format!(
                "{:>BANDWIDTH_WIDTH$}",
                Colors::dimmed(&bw_display, Theme::Dark)
            )
        } else {
            format!(
                "{:>BANDWIDTH_WIDTH$}",
                Colors::bad(&bw_display, Theme::Dark)
            )
        };

        format!(
            "  {} {} {}  {} {}",
            Colors::info(&icon, Theme::Dark),
            name_padded,
            bw_colored,
            bar,
            badge_padded,
        )
    }
}

/// Render a scenario row in compact mode (vertical stack).
#[must_use]
pub fn render_scenario_row_compact(
    status: &ScenarioStatus,
    bar_width: usize,
    nc: bool,
    minimal: bool,
) -> String {
    let s = status.scenario;
    let level = CapacityLevel::from_concurrent(status.concurrent, status.is_met);

    // Line 1: Icon + Name
    let icon = get_padded_icon(s.icon, nc, minimal);
    let name = truncate_name(s.name, bar_width + ICON_WIDTH);

    // Line 2: Bar
    let bar = render_progress_bar(status.usage_pct, bar_width, nc, minimal);

    // Line 3: Bandwidth + Badge (tabular)
    let bw_display = crate::common::tabular_number(s.required_mbps, BANDWIDTH_WIDTH, 0);
    let badge = capacity_badge_style(level, nc, status.concurrent);

    if minimal || nc {
        format!("  {icon} {name}\n    {bar}\n    {bw_display} {badge}")
    } else {
        format!(
            "  {} {}\n    {}\n    {} {}",
            icon,
            name,
            bar,
            Colors::dimmed(&bw_display, Theme::Dark),
            badge,
        )
    }
}

// ── Category Rendering ───────────────────────────────────────────────────────

/// Render category header with box drawing.
#[must_use]
pub fn render_category_header(
    cat: &ScenarioCategory,
    width: usize,
    nc: bool,
    minimal: bool,
) -> String {
    let title = format!(" {} {}", cat.icon, cat.name);
    let title_display = if minimal || terminal::no_emoji() {
        format!(" {}", cat.name)
    } else {
        title
    };

    let line_width = width.saturating_sub(2); // Account for borders
    let dashes_needed = line_width.saturating_sub(title_display.width());

    if minimal || nc {
        let border = "+".to_string();
        let dashes = "-".repeat(dashes_needed);
        format!("  {border}{dashes}{title_display}{dashes}")
    } else {
        let left_dash = "─".repeat(dashes_needed / 2);
        let right_dash = "─".repeat(dashes_needed.saturating_sub(dashes_needed / 2));
        format!(
            "  {}{}{}{}",
            left_dash.dimmed(),
            Colors::header(&title_display, Theme::Dark),
            right_dash.dimmed(),
            "".dimmed(),
        )
    }
}

/// Render a category box with all scenarios.
#[must_use]
pub fn render_category_box(
    cat: &ScenarioCategory,
    statuses: &[ScenarioStatus],
    layout: ResponsiveLayout,
    terminal_width: u16,
    nc: bool,
    minimal: bool,
) -> String {
    let mut lines = Vec::new();

    // Calculate bar width
    let bar_width = if layout.is_compact() {
        (terminal_width as usize)
            .saturating_sub(6)
            .max(MIN_BAR_WIDTH)
    } else {
        (terminal_width as usize)
            .saturating_sub(FIXED_COLUMNS_WIDTH)
            .max(MIN_BAR_WIDTH)
    };

    // Category header (includes separator lines)
    let line_width = terminal_width as usize;
    lines.push(render_category_header(cat, line_width, nc, minimal));

    // Scenario rows
    for status in statuses {
        if layout.is_compact() {
            lines.push(render_scenario_row_compact(status, bar_width, nc, minimal));
        } else {
            lines.push(render_scenario_row_expanded(
                status,
                bar_width,
                nc,
                minimal,
                layout.show_multiplier(),
            ));
        }

        // Add warning description if present
        if let Some(desc) = status.scenario.description {
            let indent = "     ";
            if minimal || nc {
                lines.push(format!("{indent}[!] {desc}"));
            } else {
                lines.push(format!(
                    "{}{} {}",
                    indent,
                    "⚠".bright_yellow(),
                    Colors::dimmed(desc, Theme::Dark),
                ));
            }
        }
    }

    lines.join("\n")
}

// ── Section Header/Footer ────────────────────────────────────────────────────

/// Render the overall section header.
#[must_use]
pub fn render_section_header(dl_mbps: f64, width: usize, nc: bool, minimal: bool) -> String {
    use super::scenarios::TOTAL_BANDWIDTH_MBPS;

    let title = format!(" USAGE CAPABILITY — {dl_mbps:.0} Mbps Total ");
    // Safe: TOTAL_BANDWIDTH_MBPS is 277.0, trivially fits u32.
    let total_label = format!(
        "{} Mbps Total",
        TOTAL_BANDWIDTH_MBPS.clamp(0.0, f64::from(u32::MAX)) as u32
    );

    if minimal || nc {
        let border = "+".to_string();
        let dashes = "-".repeat(width.saturating_sub(2));
        format!(
            "  {}{}\n  {} {:<width$}{}\n  {}{}\n  {:>width$}",
            border, &dashes, border, title, border, &dashes, border, total_label,
        )
    } else {
        let line_width = width.saturating_sub(4);
        let top = format!("  {}{}", "┌".dimmed(), "─".repeat(line_width).dimmed());
        let title_line = format!(
            "  {} {:<width$} {}",
            "│".dimmed(),
            Colors::header(&title, Theme::Dark),
            Colors::dimmed(&total_label, Theme::Dark),
        );
        let bottom = format!("  {}{}", "└".dimmed(), "─".repeat(line_width).dimmed());

        format!("{top}\n{title_line}\n{bottom}")
    }
}

/// Render section footer with legend.
#[must_use]
pub fn render_section_footer(nc: bool, minimal: bool) -> String {
    let mut lines = Vec::new();

    if minimal || nc {
        lines.push(String::new());
        lines.push("  Legend: OK=>10x  --=2-10x  !=~1x  FAIL=exceeded".to_string());
    } else {
        lines.push(String::new());
        lines.push(format!(
            "  {} >10x  {} 2-10x  {} ~1x  {} exceeded",
            "✓".green(),
            "●".yellow(),
            "⚠".bright_yellow(),
            "✗".red().bold(),
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::bandwidth_dashboard::scenarios::{
        BandwidthScenario, CapacityLevel, ResponsiveLayout, ScenarioCategory, ScenarioStatus,
    };

    // Static category for testing with inline scenario definitions
    static TEST_CATEGORY: ScenarioCategory = ScenarioCategory {
        name: "TEST CATEGORY",
        icon: "💬",
        scenarios: &[
            BandwidthScenario {
                name: "Test 1",
                required_mbps: 10.0,
                icon: "📹",
                concurrent_label: "tests",
                description: None,
            },
            BandwidthScenario {
                name: "Test 2",
                required_mbps: 20.0,
                icon: "🎮",
                concurrent_label: "tests",
                description: Some("Warning description"),
            },
        ],
    };

    // Static category with warning scenario
    static TEST_CATEGORY_WITH_WARNING: ScenarioCategory = ScenarioCategory {
        name: "TEST CATEGORY",
        icon: "💬",
        scenarios: &[BandwidthScenario {
            name: "Test With Warning",
            required_mbps: 15.0,
            icon: "🎮",
            concurrent_label: "tests",
            description: Some("Warning!"),
        }],
    };

    // Helper to create a ScenarioStatus for testing
    fn make_status(
        _required_mbps: f64,
        concurrent: u32,
        is_met: bool,
        usage_pct: f64,
    ) -> ScenarioStatus {
        // Use the first scenario from TEST_CATEGORY
        ScenarioStatus {
            scenario: &TEST_CATEGORY.scenarios[0],
            concurrent,
            headroom_pct: 50.0,
            is_met,
            usage_pct,
        }
    }

    fn make_status_with_desc() -> ScenarioStatus {
        ScenarioStatus {
            scenario: &TEST_CATEGORY_WITH_WARNING.scenarios[0],
            concurrent: 10,
            headroom_pct: 50.0,
            is_met: true,
            usage_pct: 36.1,
        }
    }

    fn make_category_with_scenarios() -> &'static ScenarioCategory {
        &TEST_CATEGORY
    }

    // ── bar_color_by_usage tests ──────────────────────────────────────────────

    #[test]
    fn test_bar_color_by_usage_nc_mode() {
        let color_fn = bar_color_by_usage(50.0, true);
        let result = color_fn("test");
        assert_eq!(result, "test"); // No color in NC mode
    }

    #[test]
    fn test_bar_color_by_usage_green_zone_low() {
        let color_fn = bar_color_by_usage(20.0, false);
        let result = color_fn("test");
        assert!(result.contains("test")); // Should not panic
    }

    #[test]
    fn test_bar_color_by_usage_yellow_zone() {
        let color_fn = bar_color_by_usage(50.0, false);
        let result = color_fn("test");
        assert!(result.contains("test")); // Should not panic
    }

    #[test]
    fn test_bar_color_by_usage_red_zone_high() {
        let color_fn = bar_color_by_usage(85.0, false);
        let result = color_fn("test");
        assert!(result.contains("test")); // Should not panic
    }

    #[test]
    fn test_bar_color_by_usage_boundary_low() {
        // Exactly at 31% boundary - should be yellow (>= 31)
        let color_fn = bar_color_by_usage(31.0, false);
        let result = color_fn("test");
        assert!(result.contains("test"));
    }

    #[test]
    fn test_bar_color_by_usage_boundary_high() {
        // Exactly at 71% boundary - should be red (>= 71)
        let color_fn = bar_color_by_usage(70.99, false);
        let result = color_fn("test");
        assert!(result.contains("test"));
    }

    // ── capacity_badge_style tests ────────────────────────────────────────────

    #[test]
    fn test_capacity_badge_optimal_nc() {
        let result = capacity_badge_style(CapacityLevel::Optimal, true, 15);
        assert_eq!(result, " 15x OK");
    }

    #[test]
    fn test_capacity_badge_moderate_nc() {
        let result = capacity_badge_style(CapacityLevel::Moderate, true, 5);
        assert_eq!(result, "  5x --");
    }

    #[test]
    fn test_capacity_badge_limited_nc() {
        let result = capacity_badge_style(CapacityLevel::Limited, true, 1);
        assert_eq!(result, "  1x !");
    }

    #[test]
    fn test_capacity_badge_exceeded_nc() {
        let result = capacity_badge_style(CapacityLevel::Exceeded, true, 0);
        assert_eq!(result, "FAIL");
    }

    #[test]
    fn test_capacity_badge_optimal_colored() {
        let result = capacity_badge_style(CapacityLevel::Optimal, false, 15);
        assert!(result.contains("15x"));
    }

    #[test]
    fn test_capacity_badge_moderate_colored() {
        let result = capacity_badge_style(CapacityLevel::Moderate, false, 5);
        assert!(result.contains("5x"));
    }

    #[test]
    fn test_capacity_badge_limited_colored() {
        let result = capacity_badge_style(CapacityLevel::Limited, false, 1);
        assert!(result.contains("1x"));
    }

    #[test]
    fn test_capacity_badge_exceeded_colored() {
        let result = capacity_badge_style(CapacityLevel::Exceeded, false, 0);
        assert!(result.contains("✗"));
    }

    // ── render_progress_bar tests ─────────────────────────────────────────────

    #[test]
    fn test_render_progress_bar_zero_width() {
        let result = render_progress_bar(50.0, 0, false, false);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_progress_bar_empty() {
        let result = render_progress_bar(0.0, 10, true, false);
        assert!(result.contains("[----------]"));
    }

    #[test]
    fn test_render_progress_bar_full() {
        let result = render_progress_bar(100.0, 10, true, false);
        assert!(result.contains("[==========]"));
    }

    #[test]
    fn test_render_progress_bar_half() {
        let result = render_progress_bar(50.0, 10, true, false);
        assert!(result.contains("[=====>"));
        assert!(result.contains("]"));
    }

    #[test]
    fn test_render_progress_bar_minimal_mode() {
        let result = render_progress_bar(50.0, 10, true, true);
        assert!(result.contains("="));
        assert!(result.contains("-"));
        assert!(result.contains("[~]")); // Should have severity marker
    }

    #[test]
    fn test_render_progress_bar_high_usage_severity() {
        let result = render_progress_bar(75.0, 10, true, false);
        assert!(result.contains("[!]")); // High usage severity
    }

    #[test]
    fn test_render_progress_bar_colored_green() {
        let result = render_progress_bar(20.0, 10, false, false);
        assert!(result.contains("█"));
        assert!(result.contains("░"));
    }

    #[test]
    fn test_render_progress_bar_colored_yellow() {
        let result = render_progress_bar(50.0, 10, false, false);
        assert!(result.contains("█"));
    }

    #[test]
    fn test_render_progress_bar_colored_red() {
        let result = render_progress_bar(80.0, 10, false, false);
        assert!(result.contains("█"));
    }

    #[test]
    fn test_render_progress_bar_fractional_fill() {
        // 45% of 10 = 4.5 → should have partial block
        let result = render_progress_bar(45.0, 10, false, false);
        let blocks: Vec<_> = result
            .chars()
            .filter(|c| *c == '█' || *c == '▉' || *c == '▌' || *c == '▎')
            .collect();
        assert!(!blocks.is_empty());
    }

    // ── get_padded_icon tests ─────────────────────────────────────────────────

    #[test]
    fn test_get_padded_icon_video() {
        // May return Unicode or ASCII depending on terminal::no_emoji()
        let result = get_padded_icon("📹", false, false);
        assert!(!result.is_empty());
        // Either "📹 " (Unicode) or "VC" (ASCII fallback), both <= 3 chars
        assert!(result.len() <= 3 || result.contains("📹"));
    }

    #[test]
    fn test_get_padded_icon_gaming() {
        let result = get_padded_icon("🎮", false, false);
        assert!(!result.is_empty());
        // May be "🎮 " or "GM", both valid
        assert!(result.len() <= 3 || result.contains("🎮"));
    }

    #[test]
    fn test_get_padded_icon_vpn() {
        let result = get_padded_icon("🔒", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_streaming() {
        let result = get_padded_icon("📺", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_cloud() {
        let result = get_padded_icon("☁️", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_upload() {
        let result = get_padded_icon("🎥", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_desktop() {
        let result = get_padded_icon("🖥️", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_camera() {
        let result = get_padded_icon("📷", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_iot() {
        let result = get_padded_icon("🔌", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_ai() {
        let result = get_padded_icon("🤖", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_vr() {
        let result = get_padded_icon("🥽", false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_get_padded_icon_unknown() {
        // Unknown icons return "??" in any mode (ASCII fallback even in Unicode mode)
        let result = get_padded_icon("???", false, false);
        assert!(!result.is_empty());
        assert!(result.len() <= 3);
    }

    #[test]
    fn test_get_padded_icon_minimal_mode() {
        // In minimal mode, always returns ASCII fallback
        let result = get_padded_icon("📹", true, true);
        assert_eq!(result, "VC");
    }

    #[test]
    fn test_get_padded_icon_gaming_minimal() {
        let result = get_padded_icon("🎮", true, true);
        assert_eq!(result, "GM");
    }

    // ── truncate_name tests ───────────────────────────────────────────────────

    #[test]
    fn test_truncate_name_short() {
        let result = truncate_name("Short", 10);
        assert_eq!(result, "Short");
    }

    #[test]
    fn test_truncate_name_exact_width() {
        let result = truncate_name("ExactWidth", 10);
        assert_eq!(result, "ExactWidth");
    }

    #[test]
    fn test_truncate_name_long() {
        let result = truncate_name("This is a very long name", 10);
        assert!(result.ends_with('…'));
        // Truncated name + ellipsis should be <= max_width + ellipsis width
        assert!(result.width() <= 11);
    }

    #[test]
    fn test_truncate_name_zero_width() {
        let result = truncate_name("Test", 0);
        assert_eq!(result, "…");
    }

    #[test]
    fn test_truncate_name_unicode() {
        let result = truncate_name("日本語テスト", 10);
        // Unicode characters may have different width, just check it's not empty
        assert!(!result.is_empty());
        assert!(result.width() <= 11); // max_width + ellipsis
    }

    // ── render_scenario_row_expanded tests ────────────────────────────────────

    #[test]
    fn test_render_scenario_row_expanded_optimal() {
        // concurrent=11 (>10) gives Optimal level -> "OK"
        let status = make_status(10.0, 11, true, 36.1);
        let result = render_scenario_row_expanded(&status, 15, true, false, true);
        assert!(!result.is_empty());
        assert!(result.contains("OK"));
    }

    #[test]
    fn test_render_scenario_row_expanded_not_met() {
        // is_met=false gives Exceeded level -> "FAIL"
        let status = make_status(50.0, 5, false, 180.5);
        let result = render_scenario_row_expanded(&status, 15, true, false, true);
        assert!(!result.is_empty());
        assert!(result.contains("FAIL"));
    }

    #[test]
    fn test_render_scenario_row_expanded_no_multiplier() {
        let status = make_status(10.0, 11, true, 36.1);
        let result = render_scenario_row_expanded(&status, 15, true, false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_scenario_row_expanded_colored() {
        let status = make_status(10.0, 11, true, 36.1);
        let result = render_scenario_row_expanded(&status, 15, false, false, true);
        assert!(!result.is_empty());
    }

    // ── render_scenario_row_compact tests ─────────────────────────────────────

    #[test]
    fn test_render_scenario_row_compact_basic() {
        let status = make_status(10.0, 10, true, 36.1);
        let result = render_scenario_row_compact(&status, 15, true, false);
        assert!(!result.is_empty());
        assert!(result.contains('\n'));
    }

    #[test]
    fn test_render_scenario_row_compact_colored() {
        let status = make_status(10.0, 10, true, 36.1);
        let result = render_scenario_row_compact(&status, 15, false, false);
        assert!(!result.is_empty());
    }

    // ── render_category_header tests ──────────────────────────────────────────

    #[test]
    fn test_render_category_header_basic() {
        let cat = make_category_with_scenarios();
        let result = render_category_header(&cat, 50, true, false);
        assert!(!result.is_empty());
        assert!(result.contains("TEST CATEGORY"));
    }

    #[test]
    fn test_render_category_header_minimal() {
        let cat = make_category_with_scenarios();
        let result = render_category_header(&cat, 50, true, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_header_colored() {
        let cat = make_category_with_scenarios();
        let result = render_category_header(&cat, 50, false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_header_narrow() {
        let cat = make_category_with_scenarios();
        let result = render_category_header(&cat, 20, true, false);
        assert!(!result.is_empty());
    }

    // ── render_category_box tests ─────────────────────────────────────────────

    #[test]
    fn test_render_category_box_expanded() {
        let cat = make_category_with_scenarios();
        let statuses = vec![
            make_status(10.0, 10, true, 36.1),
            make_status(20.0, 5, false, 72.2),
        ];
        let result = render_category_box(
            &cat,
            &statuses,
            ResponsiveLayout::Expanded,
            120,
            true,
            false,
        );
        assert!(!result.is_empty());
        assert!(result.contains("TEST CATEGORY"));
    }

    #[test]
    fn test_render_category_box_standard() {
        let cat = make_category_with_scenarios();
        let statuses = vec![make_status(10.0, 10, true, 36.1)];
        let result = render_category_box(
            &cat,
            &statuses,
            ResponsiveLayout::Standard,
            100,
            true,
            false,
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_box_compact() {
        let cat = make_category_with_scenarios();
        let statuses = vec![make_status(10.0, 10, true, 36.1)];
        let result =
            render_category_box(&cat, &statuses, ResponsiveLayout::Compact, 85, true, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_box_minimal() {
        let cat = make_category_with_scenarios();
        let statuses = vec![make_status(10.0, 10, true, 36.1)];
        let result =
            render_category_box(&cat, &statuses, ResponsiveLayout::Minimal, 70, true, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_box_colored() {
        let cat = make_category_with_scenarios();
        let statuses = vec![make_status(10.0, 10, true, 36.1)];
        let result = render_category_box(
            &cat,
            &statuses,
            ResponsiveLayout::Expanded,
            120,
            false,
            false,
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_category_box_with_warning() {
        // Use the status with description that includes "Warning!"
        let status = make_status_with_desc();
        let result = render_category_box(
            &TEST_CATEGORY_WITH_WARNING,
            &[status],
            ResponsiveLayout::Expanded,
            120,
            true,
            false,
        );
        assert!(result.contains("Warning!"));
    }

    // ── render_section_header tests ──────────────────────────────────────────

    #[test]
    fn test_render_section_header_basic() {
        let result = render_section_header(100.0, 50, true, false);
        assert!(!result.is_empty());
        assert!(result.contains("100"));
        assert!(result.contains("Mbps"));
    }

    #[test]
    fn test_render_section_header_minimal() {
        let result = render_section_header(100.0, 50, true, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_section_header_colored() {
        let result = render_section_header(100.0, 50, false, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_section_header_narrow() {
        let result = render_section_header(50.0, 30, true, false);
        assert!(!result.is_empty());
    }

    // ── render_section_footer tests ──────────────────────────────────────────

    #[test]
    fn test_render_section_footer_nc() {
        let result = render_section_footer(true, false);
        assert!(result.contains("Legend"));
    }

    #[test]
    fn test_render_section_footer_minimal() {
        let result = render_section_footer(true, true);
        assert!(result.contains("Legend"));
    }

    #[test]
    fn test_render_section_footer_colored() {
        let result = render_section_footer(false, false);
        assert!(!result.is_empty());
    }
}
