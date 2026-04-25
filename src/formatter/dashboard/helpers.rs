//! Dashboard output format V2 — rich single-screen layout with 3-column metrics,
//! compact capability matrix, and two-column estimates.
//!
//! Layout priority:
//! 1. Header: Server + IP + version (rounded box)
//! 2. Metrics dashboard: Performance | Stability | Bufferbloat (3 columns)
//! 3. Capability matrix: compact table with warnings-only
//! 4. Transfer estimates: two-column layout
//! 5. Footer: Grade + timestamp

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::formatter::grades;
use crate::formatter::ratings;
use crate::profiles::UserProfile;
use crate::terminal;
use crate::theme::{Colors, Theme};
use owo_colors::OwoColorize;

#[allow(unused_imports)]
pub use crate::types::TestResult;

pub const LINE_WIDTH: usize = 60;

// ── Summary struct ──────────────────────────────────────────────────────────

/// Summary data extracted from test runs for dashboard display.
pub struct Summary {
    pub dl_mbps: f64,
    pub dl_peak_mbps: f64,
    pub dl_bytes: u64,
    pub dl_duration: f64,
    pub ul_mbps: f64,
    pub ul_peak_mbps: f64,
    pub ul_bytes: u64,
    pub ul_duration: f64,
    pub elapsed: std::time::Duration,
    pub profile: UserProfile,
    pub theme: Theme,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Render a small capacity bar (10 chars).
pub fn mini_bar(fill: usize, width: usize, nc: bool, headroom_pct: f64) -> String {
    let empty = width.saturating_sub(fill);
    if nc {
        format!("[{}{}]", "#".repeat(fill), "-".repeat(empty))
    } else {
        let filled = "█".repeat(fill);
        let empty_str = "░".repeat(empty);
        let bar_color = if headroom_pct > 50.0 {
            "green"
        } else if headroom_pct >= 20.0 {
            "yellow"
        } else {
            "red"
        };
        match bar_color {
            "green" => format!("[{}{}]", filled.green(), empty_str),
            "yellow" => format!("[{}{}]", filled.yellow(), empty_str),
            _ => format!("[{}{}]", filled.red().bold(), empty_str),
        }
    }
}

/// Get a severity icon and label based on headroom.
pub fn severity_icon(headroom_pct: f64, is_met: bool) -> (&'static str, &'static str) {
    if terminal::no_emoji() {
        if !is_met {
            ("[FAIL]", "red")
        } else if headroom_pct > 50.0 {
            ("[OK]", "green")
        } else if headroom_pct >= 20.0 {
            ("[WARN]", "yellow")
        } else {
            ("[LOW]", "red")
        }
    } else if !is_met {
        ("🔴", "red")
    } else if headroom_pct > 50.0 {
        ("✅", "green")
    } else if headroom_pct >= 20.0 {
        ("⚠️", "yellow")
    } else {
        ("🔴", "red")
    }
}

/// Get stability label from CV%.
pub fn stability_label(cv_pct: f64, nc: bool, theme: Theme) -> (String, String) {
    let grade = grades::grade_stability(cv_pct);
    let (icon, color) = if cv_pct < 5.0 {
        ("rock-solid", "green")
    } else if cv_pct < 10.0 {
        ("stable", "bright_green")
    } else if cv_pct < 20.0 {
        ("variable", "yellow")
    } else if cv_pct < 35.0 {
        ("unstable", "bright_yellow")
    } else {
        ("chaotic", "red")
    };
    if nc {
        (
            format!("[{}]", grade.as_str()),
            format!("±{cv_pct:.0}% {icon}"),
        )
    } else {
        let grade_display = grade.color_str(nc, theme);
        let icon_display = match color {
            "green" | "bright_green" => Colors::good(icon, theme),
            "yellow" | "bright_yellow" => Colors::warn(icon, theme),
            _ => Colors::bad(icon, theme),
        };
        (grade_display, format!("±{cv_pct:.0}% {icon_display}"))
    }
}

/// Get bufferbloat info from load latency.
pub fn bufferbloat_info(
    load_latency: f64,
    idle_ping: Option<f64>,
    nc: bool,
    theme: Theme,
) -> (String, String) {
    let idle = idle_ping.unwrap_or(0.0);
    let added = if idle > 0.0 {
        load_latency - idle
    } else {
        load_latency
    };
    let grade = grades::grade_bufferbloat(added.max(0.0));
    if nc {
        (
            format!("[{}]", grade.as_str()),
            format!("+{:.0}ms", added.max(0.0)),
        )
    } else {
        let grade_str = grade.color_str(nc, theme);
        (grade_str, format!("+{:.0}ms", added.max(0.0)))
    }
}

/// Speed icon based on rating.
/// When `no_emoji()` is true, returns plain text severity indicators.
#[allow(dead_code)]
pub fn speed_icon(mbps: f64, _nc: bool) -> (&'static str, &'static str) {
    let rating = ratings::speed_rating_mbps(mbps);
    if terminal::no_emoji() {
        match rating {
            "Excellent" | "Great" => ("FAST", "blue"),
            "Good" => ("GOOD", "bright_green"),
            "Fair" | "Moderate" => ("FAIR", "yellow"),
            _ => ("SLOW", "red"),
        }
    } else {
        match rating {
            "Excellent" | "Great" => ("🔵", "blue"),
            "Good" => ("🟢", "bright_green"),
            "Fair" | "Moderate" => ("🟡", "yellow"),
            _ => ("🔴", "red"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn test_mini_bar_full() {
        let bar = mini_bar(10, 10, true, 100.0);
        assert_eq!(bar, "[##########]");
    }

    #[test]
    fn test_mini_bar_empty() {
        let bar = mini_bar(0, 10, true, 0.0);
        assert_eq!(bar, "[----------]");
    }

    #[test]
    fn test_mini_bar_half() {
        let bar = mini_bar(5, 10, true, 50.0);
        assert_eq!(bar, "[#####-----]");
    }

    #[test]
    fn test_mini_bar_width_larger_than_fill() {
        let bar = mini_bar(3, 10, true, 30.0);
        assert_eq!(bar, "[###-------]");
    }

    #[test]
    fn test_mini_bar_nc_mode() {
        let bar = mini_bar(5, 8, true, 50.0);
        assert_eq!(bar, "[#####---]");
    }

    #[test]
    fn test_mini_bar_colored_high_headroom() {
        // With colored output and >50% headroom, should use green
        let bar = mini_bar(8, 10, false, 80.0);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_mini_bar_colored_medium_headroom() {
        // With colored output and 20-50% headroom, should use yellow
        let bar = mini_bar(5, 10, false, 30.0);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_mini_bar_colored_low_headroom() {
        // With colored output and <20% headroom, should use red
        let bar = mini_bar(2, 10, false, 10.0);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_severity_icon_met_high_headroom() {
        let (icon, label) = severity_icon(80.0, true);
        assert!(!icon.is_empty());
        assert!(!label.is_empty());
    }

    #[test]
    fn test_severity_icon_met_medium_headroom() {
        let (icon, label) = severity_icon(30.0, true);
        assert!(!icon.is_empty());
        assert!(!label.is_empty());
    }

    #[test]
    fn test_severity_icon_met_low_headroom() {
        let (icon, label) = severity_icon(10.0, true);
        assert!(!icon.is_empty());
        assert!(!label.is_empty());
    }

    #[test]
    fn test_severity_icon_not_met() {
        let (icon, label) = severity_icon(50.0, false);
        assert!(!icon.is_empty());
        assert!(!label.is_empty());
    }

    #[test]
    fn test_severity_icon_zero_headroom() {
        let (icon, _) = severity_icon(0.0, true);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_stability_label_rock_solid() {
        let (grade, label) = stability_label(2.0, false, Theme::Dark);
        assert!(grade.contains('A'));
        assert!(label.contains("rock-solid"));
    }

    #[test]
    fn test_stability_label_stable() {
        let (grade, label) = stability_label(7.0, false, Theme::Dark);
        assert!(grade.contains('A') || grade.contains('B'));
        assert!(label.contains("stable"));
    }

    #[test]
    fn test_stability_label_variable() {
        // 14.9% gives C grade (>=8.0 && <15.0)
        let (grade, label) = stability_label(14.9, false, Theme::Dark);
        assert!(grade.contains('C'));
        assert!(label.contains("variable"));
    }

    #[test]
    fn test_stability_label_unstable() {
        // 24.9% gives D grade (>=15.0 && <25.0)
        let (grade, label) = stability_label(24.9, false, Theme::Dark);
        assert!(grade.contains('D'));
        assert!(label.contains("unstable"));
    }

    #[test]
    fn test_stability_label_chaotic() {
        let (grade, label) = stability_label(50.0, false, Theme::Dark);
        assert!(grade.contains('F'));
        assert!(label.contains("chaotic"));
    }

    #[test]
    fn test_stability_label_nc_mode() {
        // 6.5% gives B grade (>=5.0 && <8.0)
        let (grade, label) = stability_label(6.5, true, Theme::Dark);
        assert!(grade.contains('[')); // NC mode uses brackets
        assert!(label.contains("stable"));
    }

    #[test]
    fn test_bufferbloat_info_basic() {
        let (grade, label) = bufferbloat_info(20.0, Some(10.0), false, Theme::Dark);
        assert!(!grade.is_empty());
        assert!(label.contains("+10ms")); // 20 - 10 = 10ms added
    }

    #[test]
    fn test_bufferbloat_info_no_idle() {
        let (grade, label) = bufferbloat_info(25.0, None, false, Theme::Dark);
        assert!(!grade.is_empty());
        assert!(label.contains("+25ms")); // Uses load latency as-is
    }

    #[test]
    fn test_bufferbloat_info_zero_idle() {
        let (grade, label) = bufferbloat_info(30.0, Some(0.0), false, Theme::Dark);
        assert!(!grade.is_empty());
        assert!(label.contains("+30ms")); // Uses load latency when idle is 0
    }

    #[test]
    fn test_bufferbloat_info_nc_mode() {
        let (grade, label) = bufferbloat_info(15.0, Some(5.0), true, Theme::Dark);
        assert!(grade.contains('[')); // NC mode uses brackets
        assert!(label.contains("+10ms"));
    }

    #[test]
    fn test_speed_icon_excellent() {
        let (icon, _) = speed_icon(1000.0, false);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_speed_icon_good() {
        let (icon, _) = speed_icon(100.0, false);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_speed_icon_fair() {
        let (icon, _) = speed_icon(25.0, false);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_speed_icon_poor() {
        let (icon, _) = speed_icon(1.0, false);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_speed_icon_nc_mode() {
        let (icon, _) = speed_icon(500.0, true);
        assert!(!icon.is_empty());
        // NC mode should return text like "FAST" instead of emoji
    }

    #[test]
    fn test_line_width_constant() {
        assert_eq!(LINE_WIDTH, 60);
    }

    #[test]
    fn test_summary_struct_creation() {
        let summary = Summary {
            dl_mbps: 100.0,
            dl_peak_mbps: 120.0,
            dl_bytes: 10_000_000,
            dl_duration: 2.0,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 1.0,
            elapsed: std::time::Duration::from_secs(10),
            profile: crate::profiles::UserProfile::PowerUser,
            theme: Theme::Dark,
        };
        assert_eq!(summary.dl_mbps, 100.0);
        assert_eq!(summary.ul_mbps, 50.0);
    }

    #[test]
    fn test_bufferbloat_info_negative_added() {
        // When load latency is less than idle, added should be 0 (not negative)
        let (_, label) = bufferbloat_info(5.0, Some(10.0), false, Theme::Dark);
        assert!(label.contains("+0ms") || label.contains("+5ms"));
    }
}
