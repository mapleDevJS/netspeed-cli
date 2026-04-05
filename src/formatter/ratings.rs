//! Rating helper functions for speed test results.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

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
    if nc {
        rating.to_string()
    } else {
        match rating {
            "Excellent" => format!("{}{}", "⚡ ".green().bold(), rating.green().bold()),
            "Great" => format!("{}{}", "🟢  ".green(), rating.green()),
            "Good" => format!("{}{}", "🟢  ".bright_green(), rating.bright_green()),
            "Fair" => format!("{}{}", "🟡  ".yellow(), rating.yellow()),
            "Moderate" => format!("{}{}", "🟠  ".bright_yellow(), rating.bright_yellow()),
            "Poor" => format!("{}{}", "🔴  ".red(), rating.red()),
            "Slow" => format!("{}{}", "🔴  ".bright_red(), rating.bright_red()),
            "Very Slow" => format!("{}{}", "⚠️  ".red().bold(), rating.red().bold()),
            _ => rating.to_string(),
        }
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

pub fn format_speed_colored(bps: f64, bytes: bool) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    let mbps = bps / 1_000_000.0;
    let rating = speed_rating_mbps(mbps);
    match rating {
        "Excellent" | "Great" => format!("{value:.2} {unit}").green().bold().to_string(),
        "Good" => format!("{value:.2} {unit}").bright_green().to_string(),
        "Fair" | "Moderate" => format!("{value:.2} {unit}").yellow().to_string(),
        "Poor" | "Slow" | "Very Slow" => format!("{value:.2} {unit}").red().to_string(),
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
    let mut score = 0.0;
    let mut factors = 0.0;

    if let Some(ping) = result.ping {
        score += if ping < 10.0 {
            100.0
        } else if ping < 30.0 {
            80.0
        } else if ping < 60.0 {
            60.0
        } else if ping < 100.0 {
            40.0
        } else {
            20.0
        };
        factors += 1.0;
    }

    if let Some(jitter) = result.jitter {
        score += if jitter < 2.0 {
            100.0
        } else if jitter < 5.0 {
            80.0
        } else if jitter < 10.0 {
            60.0
        } else if jitter < 20.0 {
            40.0
        } else {
            20.0
        };
        factors += 1.0;
    }

    if let Some(dl) = result.download {
        let mbps = dl / 1_000_000.0;
        score += if mbps >= 500.0 {
            100.0
        } else if mbps >= 200.0 {
            85.0
        } else if mbps >= 100.0 {
            70.0
        } else if mbps >= 50.0 {
            55.0
        } else if mbps >= 25.0 {
            40.0
        } else if mbps >= 10.0 {
            25.0
        } else {
            10.0
        };
        factors += 1.0;
    }

    if let Some(ul) = result.upload {
        let mbps = ul / 1_000_000.0;
        score += if mbps >= 500.0 {
            100.0
        } else if mbps >= 200.0 {
            85.0
        } else if mbps >= 100.0 {
            70.0
        } else if mbps >= 50.0 {
            55.0
        } else if mbps >= 25.0 {
            40.0
        } else if mbps >= 10.0 {
            25.0
        } else {
            10.0
        };
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

/// Compute bufferbloat grade from latency degradation under load.
#[must_use]
pub fn bufferbloat_grade(load_latency: f64, idle_latency: f64) -> (&'static str, f64) {
    let added = if idle_latency > 0.0 {
        load_latency - idle_latency
    } else {
        load_latency
    };
    let grade = if added < 5.0 {
        "A"
    } else if added < 20.0 {
        "B"
    } else if added < 50.0 {
        "C"
    } else if added < 100.0 {
        "D"
    } else {
        "F"
    };
    (grade, added.max(0.0))
}

pub fn bufferbloat_colorized(grade: &str, added_ms: f64, nc: bool) -> String {
    if nc {
        format!("{grade} ({added_ms:.0}ms)")
    } else {
        let (color, bold) = match grade {
            "A" => ("green", true),
            "B" => ("bright_green", false),
            "C" => ("yellow", false),
            "D" => ("bright_yellow", false),
            "F" => ("red", true),
            _ => ("dimmed", false),
        };
        let text = format!("{grade} ({added_ms:.0}ms added)");
        match (color, bold) {
            ("green", true) => text.green().bold().to_string(),
            ("bright_green", _) => text.bright_green().to_string(),
            ("yellow", _) => text.yellow().to_string(),
            ("bright_yellow", _) => text.bright_yellow().to_string(),
            ("red", true) => text.red().bold().to_string(),
            _ => text.dimmed().to_string(),
        }
    }
}

pub fn format_overall_rating(result: &TestResult, nc: bool) -> String {
    let rating = connection_rating(result);
    if nc {
        format!("  Overall: {rating}")
    } else {
        let (icon, color) = match rating {
            "Excellent" => ("⚡ ", "green"),
            "Great" => ("🟢  ", "green"),
            "Good" => ("🟢  ", "bright_green"),
            "Fair" => ("🟡  ", "yellow"),
            "Moderate" => ("🟠  ", "bright_yellow"),
            "Poor" => ("🔴  ", "red"),
            _ => ("", ""),
        };
        let text = format!("{icon}{rating}");
        let colored = match color {
            "green" => text.green().bold().to_string(),
            "bright_green" => text.bright_green().to_string(),
            "yellow" => text.yellow().to_string(),
            "bright_yellow" => text.bright_yellow().to_string(),
            "red" => text.red().to_string(),
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
        format!("  [{text:>8}]")
    } else {
        let colored = match color {
            "green" => text.green().to_string(),
            "yellow" => text.yellow().to_string(),
            "red" => text.red().to_string(),
            _ => text.dimmed().to_string(),
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
