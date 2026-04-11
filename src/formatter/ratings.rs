//! Rating helper functions for speed test results.

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

pub fn colorize_rating(rating: &str, nc: bool) -> String {
    let no_emoji = crate::common::no_emoji();
    let display = if no_emoji {
        rating.to_string()
    } else {
        let icon = match rating {
            "Excellent" => "⚡  ",
            "Great" => "🔵  ",
            "Good" => "🟢  ",
            "Fair" => "🟡  ",
            "Moderate" => "🟠  ",
            "Poor" => "🔴  ",
            "Slow" => "🟤  ",
            "Very Slow" => "⚠️  ",
            "Bad" => "⛔  ",
            _ => "",
        };
        format!("{icon}{rating}")
    };
    if nc {
        return display;
    }
    match rating {
        "Excellent" => display.green().bold().to_string(),
        "Great" => display.blue().to_string(),
        "Good" => display.bright_green().to_string(),
        "Fair" => display.yellow().to_string(),
        "Moderate" => display.bright_yellow().to_string(),
        "Poor" => display.red().to_string(),
        "Slow" => display.bright_red().to_string(),
        "Very Slow" | "Bad" => display.red().bold().to_string(),
        _ => display.dimmed().to_string(),
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
    let value = bps / divider / 1_000_000.0;
    if value >= 1000.0 {
        let unit = if bytes { "GB/s" } else { "Gb/s" };
        SpeedComponents {
            value: value / 1000.0,
            unit,
        }
    } else {
        let unit = if bytes { "MB/s" } else { "Mb/s" };
        SpeedComponents { value, unit }
    }
}

pub fn format_speed_colored(bps: f64, bytes: bool) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    let mbps = bps / 1_000_000.0;
    let rating = speed_rating_mbps(mbps);
    match rating {
        "Excellent" | "Great" => format!("{value:.2} {unit} ({rating})")
            .green()
            .bold()
            .to_string(),
        "Good" => format!("{value:.2} {unit} ({rating})")
            .bright_green()
            .to_string(),
        "Fair" | "Moderate" => format!("{value:.2} {unit} ({rating})").yellow().to_string(),
        "Poor" | "Slow" | "Very Slow" => format!("{value:.2} {unit} ({rating})").red().to_string(),
        _ => format!("{value:.2} {unit}"),
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

pub fn bufferbloat_colorized(grade: BufferbloatGrade, added_ms: f64, nc: bool) -> String {
    if nc {
        return format!("{} ({added_ms:.0}ms)", grade.as_str());
    }
    let text = format!("{} ({added_ms:.0}ms added)", grade.as_str());
    match grade {
        BufferbloatGrade::A => text.green().bold().to_string(),
        BufferbloatGrade::B => text.bright_green().to_string(),
        BufferbloatGrade::C => text.yellow().to_string(),
        BufferbloatGrade::D => text.bright_yellow().to_string(),
        BufferbloatGrade::F => text.red().bold().to_string(),
    }
}

pub fn format_overall_rating(result: &TestResult, nc: bool) -> String {
    let rating = connection_rating(result);
    let no_emoji = crate::common::no_emoji();
    if no_emoji {
        let colored = if nc {
            rating.to_string()
        } else {
            match rating {
                "Excellent" => rating.green().bold().to_string(),
                "Great" => rating.blue().to_string(),
                "Good" => rating.bright_green().to_string(),
                "Fair" => rating.yellow().to_string(),
                "Moderate" => rating.bright_yellow().to_string(),
                "Poor" | "Bad" => rating.red().bold().to_string(),
                _ => rating.dimmed().to_string(),
            }
        };
        let label = if nc {
            "Overall:".to_string()
        } else {
            "Overall:".dimmed().to_string()
        };
        format!("  {label} {colored}")
    } else {
        let (icon, color) = match rating {
            "Excellent" => ("⚡ ", "green"),
            "Great" => ("🔵  ", "blue"),
            "Good" => ("🟢  ", "bright_green"),
            "Fair" => ("🟡  ", "yellow"),
            "Moderate" => ("🟠  ", "bright_yellow"),
            "Poor" => ("🔴  ", "red"),
            "Bad" => ("⛔  ", "red"),
            _ => ("", ""),
        };
        let text = format!("{icon}{rating}");
        let colored = match color {
            "green" => text.green().bold().to_string(),
            "blue" => text.blue().to_string(),
            "bright_green" => text.bright_green().to_string(),
            "yellow" => text.yellow().to_string(),
            "bright_yellow" => text.bright_yellow().to_string(),
            "red" => text.red().bold().to_string(),
            _ => text.dimmed().to_string(),
        };
        format!("  {} {colored}", "Overall:".dimmed())
    }
}

pub fn degradation_str(lat_load: f64, idle_ping: Option<f64>, nc: bool) -> String {
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
        text
    } else {
        match color {
            "green" => text.green().to_string(),
            "yellow" => text.yellow().to_string(),
            "red" => text.red().to_string(),
            _ => text.dimmed().to_string(),
        }
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
