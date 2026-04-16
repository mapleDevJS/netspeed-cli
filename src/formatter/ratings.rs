//! Rating helper functions for speed test results.

use crate::terminal::no_emoji;
use crate::theme::{Theme, ThemeColors};
use crate::types::TestResult;
use owo_colors::OwoColorize;

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

pub fn colorize_rating(rating: &str, nc: bool, theme: Theme) -> String {
    if nc || no_emoji() {
        format!("[{rating}]")
    } else {
        let (icon, colored) = match rating {
            "Excellent" => ("⚡ ", ThemeColors::good(rating, theme)),
            "Great" => ("🔵  ", ThemeColors::info(rating, theme)),
            "Good" => ("🟢  ", ThemeColors::good(rating, theme)),
            "Fair" => ("🟡  ", ThemeColors::warn(rating, theme)),
            "Moderate" => ("🟠  ", ThemeColors::warn(rating, theme)),
            "Poor" => ("🔴  ", ThemeColors::bad(rating, theme)),
            "Slow" => ("🟤  ", ThemeColors::bad(rating, theme)),
            "Very Slow" => ("⚠️  ", ThemeColors::bad(rating, theme)),
            _ => ("", rating.to_string()),
        };
        format!("{icon}{colored}")
    }
}

/// Helper struct to hold speed formatting components.
struct SpeedComponents {
    value: f64,
    unit: &'static str,
}

/// Extract speed components for formatting.
fn speed_components(bps: f64, bytes: bool) -> SpeedComponents {
    let divider = if bytes { 8.0 } else { 1.0 };
    let unit = if bytes { "MB/s" } else { "Mb/s" };
    let value = bps / divider / 1_000_000.0;
    SpeedComponents { value, unit }
}

pub fn format_speed_colored(bps: f64, bytes: bool, theme: Theme) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    let mbps = bps / 1_000_000.0;
    let rating = speed_rating_mbps(mbps);
    let text = format!("{value:.2} {unit}");
    match rating {
        "Excellent" | "Great" | "Good" => ThemeColors::good(&text, theme),
        "Fair" | "Moderate" => ThemeColors::warn(&text, theme),
        "Poor" | "Slow" | "Very Slow" => ThemeColors::bad(&text, theme),
        _ => text,
    }
}

pub fn format_speed_plain(bps: f64, bytes: bool) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    format!("{value:.2} {unit}")
}

pub fn format_duration(secs: f64) -> String {
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
    /// Score a "lower is better" metric (ping, jitter) on a 0–100 scale.
    fn score_lower(value: f64, thresholds: [f64; 5]) -> f64 {
        if value < thresholds[0] {
            100.0
        } else if value < thresholds[1] {
            80.0
        } else if value < thresholds[2] {
            60.0
        } else if value < thresholds[3] {
            40.0
        } else {
            20.0
        }
    }

    /// Score a "higher is better" metric (download, upload) on a 0–100 scale.
    fn score_higher(mbps: f64, thresholds: [f64; 6]) -> f64 {
        if mbps >= thresholds[0] {
            100.0
        } else if mbps >= thresholds[1] {
            85.0
        } else if mbps >= thresholds[2] {
            70.0
        } else if mbps >= thresholds[3] {
            55.0
        } else if mbps >= thresholds[4] {
            40.0
        } else if mbps >= thresholds[5] {
            25.0
        } else {
            10.0
        }
    }

    let mut score = 0.0;
    let mut factors = 0.0;

    // Ping (lower is better)
    if let Some(ping) = result.ping {
        score += score_lower(ping, [10.0, 30.0, 60.0, 100.0, f64::MAX]);
        factors += 1.0;
    }

    // Jitter (lower is better)
    if let Some(jitter) = result.jitter {
        score += score_lower(jitter, [2.0, 5.0, 10.0, 20.0, f64::MAX]);
        factors += 1.0;
    }

    // Download speed (higher is better)
    if let Some(dl) = result.download {
        score += score_higher(dl / 1_000_000.0, [500.0, 200.0, 100.0, 50.0, 25.0, 10.0]);
        factors += 1.0;
    }

    // Upload speed (higher is better)
    if let Some(ul) = result.upload {
        score += score_higher(ul / 1_000_000.0, [500.0, 200.0, 100.0, 50.0, 25.0, 10.0]);
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

/// Bufferbloat grade based on added latency under load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferbloatGrade {
    A,
    B,
    C,
    D,
    F,
}

impl BufferbloatGrade {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }
}

/// Compute bufferbloat grade from latency degradation under load.
#[must_use]
pub fn bufferbloat_grade(load_latency: f64, idle_latency: f64) -> (BufferbloatGrade, f64) {
    let added = if idle_latency > 0.0 {
        load_latency - idle_latency
    } else {
        load_latency
    };
    let grade = if added < 5.0 {
        BufferbloatGrade::A
    } else if added < 20.0 {
        BufferbloatGrade::B
    } else if added < 50.0 {
        BufferbloatGrade::C
    } else if added < 100.0 {
        BufferbloatGrade::D
    } else {
        BufferbloatGrade::F
    };
    (grade, added.max(0.0))
}

pub fn bufferbloat_colorized(
    grade: BufferbloatGrade,
    added_ms: f64,
    nc: bool,
    theme: Theme,
) -> String {
    if nc {
        format!("{} ({added_ms:.0}ms)", grade.as_str())
    } else {
        let text = format!("{} ({added_ms:.0}ms added)", grade.as_str());
        match grade {
            BufferbloatGrade::A => ThemeColors::good(&text, theme),
            BufferbloatGrade::B => ThemeColors::good(&text, theme),
            BufferbloatGrade::C => ThemeColors::warn(&text, theme),
            BufferbloatGrade::D => ThemeColors::warn(&text, theme),
            BufferbloatGrade::F => ThemeColors::bad(&text, theme),
        }
    }
}

pub fn format_overall_rating(result: &TestResult, nc: bool, theme: Theme) -> String {
    let rating = connection_rating(result);
    if nc || no_emoji() {
        format!("  Overall: {rating}")
    } else {
        let (icon, colored) = match rating {
            "Excellent" => ("⚡ ", ThemeColors::good(rating, theme)),
            "Great" => ("🔵  ", ThemeColors::info(rating, theme)),
            "Good" => ("🟢  ", ThemeColors::good(rating, theme)),
            "Fair" => ("🟡  ", ThemeColors::warn(rating, theme)),
            "Moderate" => ("🟠  ", ThemeColors::warn(rating, theme)),
            "Poor" => ("🔴  ", ThemeColors::bad(rating, theme)),
            _ => ("", rating.to_string()),
        };
        format!("  {} {icon}{colored}", "Overall:".dimmed())
    }
}

pub fn degradation_str(lat_load: f64, idle_ping: Option<f64>, nc: bool, theme: Theme) -> String {
    let Some(idle) = idle_ping else {
        return String::new();
    };
    if idle <= 0.0 {
        return String::new();
    }
    let pct = ((lat_load / idle) - 1.0) * 100.0;
    let text = format!(
        "+{pct:.0}% ({})",
        if pct < 25.0 {
            "minimal"
        } else if pct < 50.0 {
            "moderate"
        } else {
            "significant"
        }
    );
    if nc {
        format!("  [{text:>8}]")
    } else {
        let colored = if pct < 25.0 {
            ThemeColors::good(&text, theme)
        } else if pct < 50.0 {
            ThemeColors::warn(&text, theme)
        } else {
            ThemeColors::bad(&text, theme)
        };
        format!("  {colored}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_rating() {
        assert_eq!(ping_rating(5.0), "Excellent");
        assert_eq!(ping_rating(20.0), "Good");
        assert_eq!(ping_rating(50.0), "Fair");
        assert_eq!(ping_rating(80.0), "Poor");
        assert_eq!(ping_rating(150.0), "Bad");
    }

    #[test]
    fn test_speed_rating() {
        assert_eq!(speed_rating_mbps(600.0), "Excellent");
        assert_eq!(speed_rating_mbps(150.0), "Good");
        assert_eq!(speed_rating_mbps(5.0), "Very Slow");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30.0), "30.0s");
        assert_eq!(format_duration(90.0), "1m 30s");
    }
}
