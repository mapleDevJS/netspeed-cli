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
use crate::theme::{Theme, ThemeColors};
use owo_colors::OwoColorize;

#[allow(unused_imports)]
pub use crate::types::TestResult;

pub const LINE_WIDTH: usize = 60;

// ── Summary struct ──────────────────────────────────────────────────────────

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
    } else {
        if !is_met {
            ("🔴", "red")
        } else if headroom_pct > 50.0 {
            ("✅", "green")
        } else if headroom_pct >= 20.0 {
            ("⚠️", "yellow")
        } else {
            ("🔴", "red")
        }
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
            format!("±{:.0}% {}", cv_pct, icon),
        )
    } else {
        let grade_display = grade.color_str(nc, theme);
        let icon_display = match color {
            "green" => ThemeColors::good(icon, theme),
            "bright_green" => ThemeColors::good(icon, theme),
            "yellow" => ThemeColors::warn(icon, theme),
            "bright_yellow" => ThemeColors::warn(icon, theme),
            _ => ThemeColors::bad(icon, theme),
        };
        (grade_display, format!("±{:.0}% {}", cv_pct, icon_display))
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
