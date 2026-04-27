//! Rating helper functions for speed test results.

use crate::terminal::no_emoji;
use crate::theme::{Colors, Theme};
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

#[must_use]
pub fn colorize_rating(rating: &str, nc: bool, theme: Theme) -> String {
    if nc || no_emoji() {
        format!("[{rating}]")
    } else {
        let (icon, colored) = match rating {
            "Excellent" => ("⚡ ", Colors::good(rating, theme)),
            "Great" => ("🔵  ", Colors::info(rating, theme)),
            "Good" => ("🟢  ", Colors::good(rating, theme)),
            "Fair" => ("🟡  ", Colors::warn(rating, theme)),
            "Moderate" => ("🟠  ", Colors::warn(rating, theme)),
            "Poor" => ("🔴  ", Colors::bad(rating, theme)),
            "Slow" => ("🟤  ", Colors::bad(rating, theme)),
            "Very Slow" => ("⚠️  ", Colors::bad(rating, theme)),
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

#[must_use]
pub fn format_speed_colored(bps: f64, bytes: bool, theme: Theme) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    let mbps = bps / 1_000_000.0;
    let rating = speed_rating_mbps(mbps);
    let text = format!("{value:.2} {unit}");
    match rating {
        "Excellent" | "Great" | "Good" => Colors::good(&text, theme),
        "Fair" | "Moderate" => Colors::warn(&text, theme),
        "Poor" | "Slow" | "Very Slow" => Colors::bad(&text, theme),
        _ => text,
    }
}

#[must_use]
pub fn format_speed_plain(bps: f64, bytes: bool) -> String {
    let SpeedComponents { value, unit } = speed_components(bps, bytes);
    format!("{value:.2} {unit}")
}

#[must_use]
pub fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        format!("{secs:.1}s")
    } else {
        // Safe: secs is test duration (seconds), always non-negative and small.
        let mins = (secs / 60.0).clamp(0.0, u64::MAX as f64) as u64;
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

#[must_use]
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
            BufferbloatGrade::A | BufferbloatGrade::B => Colors::good(&text, theme),
            BufferbloatGrade::C | BufferbloatGrade::D => Colors::warn(&text, theme),
            BufferbloatGrade::F => Colors::bad(&text, theme),
        }
    }
}

#[must_use]
pub fn format_overall_rating(result: &TestResult, nc: bool, theme: Theme) -> String {
    let rating = connection_rating(result);
    if nc || no_emoji() {
        format!("  Overall: {rating}")
    } else {
        let (icon, colored) = match rating {
            "Excellent" => ("⚡ ", Colors::good(rating, theme)),
            "Great" => ("🔵  ", Colors::info(rating, theme)),
            "Good" => ("🟢  ", Colors::good(rating, theme)),
            "Fair" => ("🟡  ", Colors::warn(rating, theme)),
            "Moderate" => ("🟠  ", Colors::warn(rating, theme)),
            "Poor" => ("🔴  ", Colors::bad(rating, theme)),
            _ => ("", rating.to_string()),
        };
        format!("  {} {icon}{colored}", "Overall:".dimmed())
    }
}

#[must_use]
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
            Colors::good(&text, theme)
        } else if pct < 50.0 {
            Colors::warn(&text, theme)
        } else {
            Colors::bad(&text, theme)
        };
        format!("  {colored}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PhaseResult, ServerInfo, TestPhases, TestResult};

    fn make_test_result() -> TestResult {
        TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 10.0,
            },
            ping: Some(15.0),
            jitter: Some(1.5),
            packet_loss: Some(0.0),
            download: Some(100_000_000.0),
            download_peak: Some(120_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            latency_download: Some(20.0),
            latency_upload: Some(18.0),
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            client_ip: None,
            client_location: None,
            download_cv: None,
            upload_cv: None,
            download_ci_95: None,
            upload_ci_95: None,
            overall_grade: None,
            download_grade: None,
            upload_grade: None,
            connection_rating: None,
            phases: TestPhases {
                ping: PhaseResult::completed(),
                download: PhaseResult::completed(),
                upload: PhaseResult::completed(),
            },
        }
    }

    // ── ping_rating tests ───────────────────────────────────────────────────

    #[test]
    fn test_ping_rating_excellent() {
        assert_eq!(ping_rating(5.0), "Excellent");
        assert_eq!(ping_rating(9.9), "Excellent");
        assert_eq!(ping_rating(0.0), "Excellent");
    }

    #[test]
    fn test_ping_rating_good() {
        assert_eq!(ping_rating(10.0), "Good");
        assert_eq!(ping_rating(29.9), "Good");
        assert_eq!(ping_rating(15.0), "Good");
    }

    #[test]
    fn test_ping_rating_fair() {
        assert_eq!(ping_rating(30.0), "Fair");
        assert_eq!(ping_rating(59.9), "Fair");
        assert_eq!(ping_rating(45.0), "Fair");
    }

    #[test]
    fn test_ping_rating_poor() {
        assert_eq!(ping_rating(60.0), "Poor");
        assert_eq!(ping_rating(99.9), "Poor");
        assert_eq!(ping_rating(80.0), "Poor");
    }

    #[test]
    fn test_ping_rating_bad() {
        assert_eq!(ping_rating(100.0), "Bad");
        assert_eq!(ping_rating(500.0), "Bad");
        assert_eq!(ping_rating(10000.0), "Bad");
    }

    // ── speed_rating_mbps tests ─────────────────────────────────────────────

    #[test]
    fn test_speed_rating_excellent() {
        assert_eq!(speed_rating_mbps(500.0), "Excellent");
        assert_eq!(speed_rating_mbps(1000.0), "Excellent");
    }

    #[test]
    fn test_speed_rating_great() {
        assert_eq!(speed_rating_mbps(200.0), "Great");
        assert_eq!(speed_rating_mbps(499.9), "Great");
        assert_eq!(speed_rating_mbps(300.0), "Great");
    }

    #[test]
    fn test_speed_rating_good() {
        assert_eq!(speed_rating_mbps(100.0), "Good");
        assert_eq!(speed_rating_mbps(199.9), "Good");
        assert_eq!(speed_rating_mbps(150.0), "Good");
    }

    #[test]
    fn test_speed_rating_fair() {
        assert_eq!(speed_rating_mbps(50.0), "Fair");
        assert_eq!(speed_rating_mbps(99.9), "Fair");
    }

    #[test]
    fn test_speed_rating_moderate() {
        assert_eq!(speed_rating_mbps(25.0), "Moderate");
        assert_eq!(speed_rating_mbps(49.9), "Moderate");
        assert_eq!(speed_rating_mbps(30.0), "Moderate");
    }

    #[test]
    fn test_speed_rating_slow() {
        assert_eq!(speed_rating_mbps(10.0), "Slow");
        assert_eq!(speed_rating_mbps(24.9), "Slow");
        assert_eq!(speed_rating_mbps(15.0), "Slow");
    }

    #[test]
    fn test_speed_rating_very_slow() {
        assert_eq!(speed_rating_mbps(0.0), "Very Slow");
        assert_eq!(speed_rating_mbps(9.9), "Very Slow");
        assert_eq!(speed_rating_mbps(1.0), "Very Slow");
    }

    // ── colorize_rating tests ───────────────────────────────────────────────

    #[test]
    fn test_colorize_rating_excellent() {
        let result = colorize_rating("Excellent", true, Theme::Dark);
        assert!(result.contains("[Excellent]"));
    }

    #[test]
    fn test_colorize_rating_great() {
        let result = colorize_rating("Great", true, Theme::Dark);
        assert!(result.contains("[Great]"));
    }

    #[test]
    fn test_colorize_rating_good() {
        let result = colorize_rating("Good", true, Theme::Dark);
        assert!(result.contains("[Good]"));
    }

    #[test]
    fn test_colorize_rating_fair() {
        let result = colorize_rating("Fair", true, Theme::Dark);
        assert!(result.contains("[Fair]"));
    }

    #[test]
    fn test_colorize_rating_moderate() {
        let result = colorize_rating("Moderate", true, Theme::Dark);
        assert!(result.contains("[Moderate]"));
    }

    #[test]
    fn test_colorize_rating_poor() {
        let result = colorize_rating("Poor", true, Theme::Dark);
        assert!(result.contains("[Poor]"));
    }

    #[test]
    fn test_colorize_rating_slow() {
        let result = colorize_rating("Slow", true, Theme::Dark);
        assert!(result.contains("[Slow]"));
    }

    #[test]
    fn test_colorize_rating_very_slow() {
        let result = colorize_rating("Very Slow", true, Theme::Dark);
        assert!(result.contains("[Very Slow]"));
    }

    #[test]
    fn test_colorize_rating_unknown() {
        // Unknown ratings should just return the text
        let result = colorize_rating("Unknown", false, Theme::Dark);
        assert_eq!(result, "Unknown");
    }

    // ── format_speed_colored tests ──────────────────────────────────────────

    #[test]
    fn test_format_speed_colored_mbps_excellent() {
        let result = format_speed_colored(500_000_000.0, false, Theme::Dark);
        assert!(result.contains("500.00"));
        assert!(result.contains("Mb/s"));
    }

    #[test]
    fn test_format_speed_colored_bytes_mode() {
        // 8_000_000 bps / 8 = 1_000_000 bytes/sec = 1.00 MB/s
        let result = format_speed_colored(8_000_000.0, true, Theme::Dark);
        assert!(result.contains("1.00"));
        assert!(result.contains("MB/s"));
    }

    #[test]
    fn test_format_speed_colored_light_theme() {
        let result = format_speed_colored(100_000_000.0, false, Theme::Light);
        assert!(result.contains("100.00"));
    }

    #[test]
    fn test_format_speed_colored_low_speed() {
        // Low speed should use bad color
        let result = format_speed_colored(5_000_000.0, false, Theme::Dark);
        assert!(result.contains("5.00"));
    }

    // ── format_speed_plain tests ────────────────────────────────────────────

    #[test]
    fn test_format_speed_plain_mbps() {
        let result = format_speed_plain(100_000_000.0, false);
        assert_eq!(result, "100.00 Mb/s");
    }

    #[test]
    fn test_format_speed_plain_bytes() {
        // 8_000_000 bps / 8 = 1_000_000 bytes/sec = 1.00 MB/s
        let result = format_speed_plain(8_000_000.0, true);
        assert_eq!(result, "1.00 MB/s");
    }

    #[test]
    fn test_format_speed_plain_zero() {
        let result = format_speed_plain(0.0, false);
        assert_eq!(result, "0.00 Mb/s");
    }

    #[test]
    fn test_format_speed_plain_fractional() {
        let result = format_speed_plain(55_555_555.0, false);
        assert!(result.contains("55.56"));
    }

    // ── format_duration tests ───────────────────────────────────────────────

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(30.0), "30.0s");
        assert_eq!(format_duration(59.9), "59.9s");
        assert_eq!(format_duration(0.5), "0.5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60.0), "1m 0s");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(125.0), "2m 5s");
    }

    #[test]
    fn test_format_duration_many_minutes() {
        assert_eq!(format_duration(600.0), "10m 0s");
        assert_eq!(format_duration(3661.0), "61m 1s");
    }

    // ── connection_rating tests ─────────────────────────────────────────────

    #[test]
    fn test_connection_rating_excellent() {
        let mut result = make_test_result();
        result.ping = Some(5.0);
        result.jitter = Some(1.0);
        result.download = Some(500_000_000.0);
        result.upload = Some(250_000_000.0);
        assert_eq!(connection_rating(&result), "Excellent");
    }

    #[test]
    fn test_connection_rating_great() {
        let mut result = make_test_result();
        result.ping = Some(20.0);
        result.jitter = Some(3.0);
        result.download = Some(200_000_000.0);
        result.upload = Some(100_000_000.0);
        assert_eq!(connection_rating(&result), "Great");
    }

    #[test]
    fn test_connection_rating_good() {
        let mut result = make_test_result();
        result.ping = Some(40.0);
        result.jitter = Some(8.0);
        result.download = Some(100_000_000.0);
        result.upload = Some(50_000_000.0);
        assert_eq!(connection_rating(&result), "Good");
    }

    #[test]
    fn test_connection_rating_fair() {
        let mut result = make_test_result();
        result.ping = Some(60.0);
        result.jitter = Some(15.0);
        result.download = Some(50_000_000.0);
        result.upload = Some(25_000_000.0);
        assert_eq!(connection_rating(&result), "Fair");
    }

    #[test]
    fn test_connection_rating_moderate() {
        let mut result = make_test_result();
        result.ping = Some(80.0);
        result.jitter = Some(18.0);
        result.download = Some(25_000_000.0);
        result.upload = Some(12_000_000.0);
        assert_eq!(connection_rating(&result), "Moderate");
    }

    #[test]
    fn test_connection_rating_poor() {
        let mut result = make_test_result();
        result.ping = Some(150.0);
        result.jitter = Some(30.0);
        result.download = Some(5_000_000.0);
        result.upload = Some(1_000_000.0);
        assert_eq!(connection_rating(&result), "Poor");
    }

    #[test]
    fn test_connection_rating_unknown_no_data() {
        let mut result = make_test_result();
        result.ping = None;
        result.jitter = None;
        result.download = None;
        result.upload = None;
        assert_eq!(connection_rating(&result), "Unknown");
    }

    #[test]
    fn test_connection_rating_partial_data() {
        let mut result = make_test_result();
        result.ping = Some(10.0);
        result.jitter = None;
        result.download = None;
        result.upload = None;
        // With only ping, should still return a rating
        let rating = connection_rating(&result);
        assert!(!rating.is_empty());
    }

    // ── BufferbloatGrade tests ──────────────────────────────────────────────

    #[test]
    fn test_bufferbloat_grade_a() {
        let (grade, added) = bufferbloat_grade(10.0, 8.0);
        assert_eq!(grade, BufferbloatGrade::A);
        assert!((added - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_b() {
        let (grade, added) = bufferbloat_grade(30.0, 15.0);
        assert_eq!(grade, BufferbloatGrade::B);
        assert!((added - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_c() {
        let (grade, added) = bufferbloat_grade(60.0, 20.0);
        assert_eq!(grade, BufferbloatGrade::C);
        assert!((added - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_d() {
        let (grade, added) = bufferbloat_grade(120.0, 30.0);
        assert_eq!(grade, BufferbloatGrade::D);
        assert!((added - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_f() {
        let (grade, added) = bufferbloat_grade(200.0, 50.0);
        assert_eq!(grade, BufferbloatGrade::F);
        assert!((added - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_zero_idle() {
        // When idle is 0, uses load_latency directly (10.0)
        // 10.0 < 5.0 (A)? No → 10.0 < 20.0 (B)? Yes → Grade B
        let (grade, added) = bufferbloat_grade(10.0, 0.0);
        assert_eq!(grade, BufferbloatGrade::B);
        assert!((added - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_bufferbloat_grade_boundaries() {
        // Test exact boundary values
        let (grade, _) = bufferbloat_grade(4.99, 0.0);
        assert_eq!(grade, BufferbloatGrade::A);

        let (grade, _) = bufferbloat_grade(5.0, 0.0);
        assert_eq!(grade, BufferbloatGrade::B);

        let (grade, _) = bufferbloat_grade(19.99, 0.0);
        assert_eq!(grade, BufferbloatGrade::B);

        let (grade, _) = bufferbloat_grade(20.0, 0.0);
        assert_eq!(grade, BufferbloatGrade::C);
    }

    // ── BufferbloatGrade as_str tests ───────────────────────────────────────

    #[test]
    fn test_bufferbloat_grade_as_str() {
        assert_eq!(BufferbloatGrade::A.as_str(), "A");
        assert_eq!(BufferbloatGrade::B.as_str(), "B");
        assert_eq!(BufferbloatGrade::C.as_str(), "C");
        assert_eq!(BufferbloatGrade::D.as_str(), "D");
        assert_eq!(BufferbloatGrade::F.as_str(), "F");
    }

    // ── bufferbloat_colorized tests ─────────────────────────────────────────

    #[test]
    fn test_bufferbloat_colorized_nc_a() {
        let result = bufferbloat_colorized(BufferbloatGrade::A, 2.0, true, Theme::Dark);
        assert!(result.contains("A"));
        assert!(result.contains("2ms"));
    }

    #[test]
    fn test_bufferbloat_colorized_nc_f() {
        let result = bufferbloat_colorized(BufferbloatGrade::F, 150.0, true, Theme::Dark);
        assert!(result.contains("F"));
        assert!(result.contains("150ms"));
    }

    #[test]
    fn test_bufferbloat_colorized_colored_a() {
        let result = bufferbloat_colorized(BufferbloatGrade::A, 2.0, false, Theme::Dark);
        assert!(result.contains("A"));
        assert!(result.contains("added"));
    }

    #[test]
    fn test_bufferbloat_colorized_colored_b() {
        let result = bufferbloat_colorized(BufferbloatGrade::B, 15.0, false, Theme::Dark);
        assert!(result.contains("B"));
    }

    #[test]
    fn test_bufferbloat_colorized_colored_c() {
        let result = bufferbloat_colorized(BufferbloatGrade::C, 40.0, false, Theme::Dark);
        assert!(result.contains("C"));
    }

    #[test]
    fn test_bufferbloat_colorized_colored_d() {
        let result = bufferbloat_colorized(BufferbloatGrade::D, 90.0, false, Theme::Dark);
        assert!(result.contains("D"));
    }

    #[test]
    fn test_bufferbloat_colorized_colored_f() {
        let result = bufferbloat_colorized(BufferbloatGrade::F, 150.0, false, Theme::Dark);
        assert!(result.contains("F"));
    }

    // ── format_overall_rating tests ─────────────────────────────────────────

    #[test]
    fn test_format_overall_rating_nc_excellent() {
        let result = make_test_result();
        let output = format_overall_rating(&result, true, Theme::Dark);
        assert!(output.contains("Overall:"));
    }

    #[test]
    fn test_format_overall_rating_colored() {
        let result = make_test_result();
        let output = format_overall_rating(&result, false, Theme::Dark);
        assert!(output.contains("Overall:"));
    }

    #[test]
    fn test_format_overall_rating_light_theme() {
        let result = make_test_result();
        let output = format_overall_rating(&result, false, Theme::Light);
        assert!(output.contains("Overall:"));
    }

    // ── degradation_str tests ───────────────────────────────────────────────

    #[test]
    fn test_degradation_str_minimal() {
        // 20% increase = minimal
        let result = degradation_str(12.0, Some(10.0), true, Theme::Dark);
        assert!(result.contains("minimal"));
    }

    #[test]
    fn test_degradation_str_moderate() {
        // 40% increase = moderate
        let result = degradation_str(14.0, Some(10.0), true, Theme::Dark);
        assert!(result.contains("moderate"));
    }

    #[test]
    fn test_degradation_str_significant() {
        // 60% increase = significant
        let result = degradation_str(16.0, Some(10.0), true, Theme::Dark);
        assert!(result.contains("significant"));
    }

    #[test]
    fn test_degradation_str_no_idle() {
        let result = degradation_str(15.0, None, false, Theme::Dark);
        assert_eq!(result, "");
    }

    #[test]
    fn test_degradation_str_zero_idle() {
        let result = degradation_str(15.0, Some(0.0), false, Theme::Dark);
        assert_eq!(result, "");
    }

    #[test]
    fn test_degradation_str_negative_idle() {
        let result = degradation_str(15.0, Some(-5.0), false, Theme::Dark);
        assert_eq!(result, "");
    }

    #[test]
    fn test_degradation_str_nc_mode() {
        let result = degradation_str(12.0, Some(10.0), true, Theme::Dark);
        assert!(result.contains("["));
    }

    #[test]
    fn test_degradation_str_colored_minimal() {
        let result = degradation_str(12.0, Some(10.0), false, Theme::Dark);
        assert!(!result.is_empty());
    }

    // ── SpeedComponents helper tests ────────────────────────────────────────

    #[test]
    fn test_speed_components_helper() {
        // Test through format_speed_plain which uses SpeedComponents
        let result = format_speed_plain(100_000_000.0, false);
        assert!(result.contains("100.00"));
        assert!(result.contains("Mb/s"));
    }
}
