//! Comprehensive quality grade system (A-F) for all connection metrics.
//!
//! Provides letter grades with color support for:
//! - Ping/Latency
//! - Jitter
//! - Download speed
//! - Upload speed
//! - Bufferbloat
//! - Stability (CV%)
//! - Overall connection quality

use crate::profiles::UserProfile;
use crate::terminal;
use crate::theme::{Theme, ThemeColors};

/// Letter grade for connection quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LetterGrade {
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    C,
    CMinus,
    D,
    F,
}

impl LetterGrade {
    /// Display string for the grade.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::APlus => "A+",
            Self::A => "A",
            Self::AMinus => "A-",
            Self::BPlus => "B+",
            Self::B => "B",
            Self::BMinus => "B-",
            Self::CPlus => "C+",
            Self::C => "C",
            Self::CMinus => "C-",
            Self::D => "D",
            Self::F => "F",
        }
    }

    /// Color for the grade (green = good, red = bad).
    pub fn color_str(&self, nc: bool, theme: Theme) -> String {
        let s = self.as_str();
        if nc {
            return format!("[{s}]");
        }
        match self {
            Self::APlus | Self::A | Self::AMinus => ThemeColors::good(s, theme),
            Self::BPlus | Self::B => ThemeColors::good(s, theme),
            Self::BMinus | Self::CPlus | Self::C => ThemeColors::warn(s, theme),
            Self::CMinus | Self::D => ThemeColors::warn(s, theme),
            Self::F => ThemeColors::bad(s, theme),
        }
    }

    /// Emoji indicator for the grade.
    pub fn emoji(&self) -> &'static str {
        if terminal::no_emoji() {
            return "";
        }
        match self {
            Self::APlus | Self::A | Self::AMinus => "⚡",
            Self::BPlus | Self::B => "✅",
            Self::BMinus | Self::CPlus | Self::C => "⚠️",
            Self::CMinus | Self::D => "❌",
            Self::F => "🚫",
        }
    }

    /// Numeric score 0-100 for the grade.
    pub fn score(&self) -> f64 {
        match self {
            Self::APlus => 100.0,
            Self::A => 95.0,
            Self::AMinus => 90.0,
            Self::BPlus => 85.0,
            Self::B => 80.0,
            Self::BMinus => 75.0,
            Self::CPlus => 70.0,
            Self::C => 65.0,
            Self::CMinus => 60.0,
            Self::D => 50.0,
            Self::F => 25.0,
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::APlus => "Exceptional",
            Self::A => "Excellent",
            Self::AMinus => "Very Good",
            Self::BPlus => "Good+",
            Self::B => "Good",
            Self::BMinus => "Above Average",
            Self::CPlus => "Average+",
            Self::C => "Average",
            Self::CMinus => "Below Average",
            Self::D => "Poor",
            Self::F => "Unacceptable",
        }
    }
}

/// Grade a ping value (lower is better).
/// Profile-aware thresholds.
pub fn grade_ping(ping_ms: f64, profile: UserProfile) -> LetterGrade {
    let excellent = profile.excellent_ping_threshold();
    let good = excellent * 3.0;
    let average = excellent * 6.0;

    if ping_ms <= excellent * 0.5 {
        LetterGrade::APlus
    } else if ping_ms <= excellent {
        LetterGrade::A
    } else if ping_ms <= excellent * 1.5 {
        LetterGrade::AMinus
    } else if ping_ms <= good {
        LetterGrade::B
    } else if ping_ms <= good * 1.5 {
        LetterGrade::C
    } else if ping_ms <= average {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Grade jitter value (lower is better).
pub fn grade_jitter(jitter_ms: f64, profile: UserProfile) -> LetterGrade {
    let excellent = profile.excellent_jitter_threshold();
    let good = excellent * 3.0;
    let average = excellent * 8.0;

    if jitter_ms <= excellent * 0.5 {
        LetterGrade::APlus
    } else if jitter_ms <= excellent {
        LetterGrade::A
    } else if jitter_ms <= excellent * 2.0 {
        LetterGrade::B
    } else if jitter_ms <= good {
        LetterGrade::C
    } else if jitter_ms <= average {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Grade download speed (higher is better).
pub fn grade_download(speed_mbps: f64, profile: UserProfile) -> LetterGrade {
    let excellent = profile.excellent_speed_threshold();
    let good = excellent * 0.4;
    let average = excellent * 0.15;

    if speed_mbps >= excellent * 2.0 {
        LetterGrade::APlus
    } else if speed_mbps >= excellent {
        LetterGrade::A
    } else if speed_mbps >= excellent * 0.75 {
        LetterGrade::B
    } else if speed_mbps >= good {
        LetterGrade::C
    } else if speed_mbps >= average {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Grade upload speed (higher is better).
pub fn grade_upload(speed_mbps: f64, profile: UserProfile) -> LetterGrade {
    // Upload thresholds are typically 50% of download
    let excellent = profile.excellent_speed_threshold() * 0.5;
    let good = excellent * 0.4;
    let average = excellent * 0.15;

    if speed_mbps >= excellent * 2.0 {
        LetterGrade::APlus
    } else if speed_mbps >= excellent {
        LetterGrade::A
    } else if speed_mbps >= excellent * 0.75 {
        LetterGrade::B
    } else if speed_mbps >= good {
        LetterGrade::C
    } else if speed_mbps >= average {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Grade bufferbloat based on added latency under load.
pub fn grade_bufferbloat(added_latency_ms: f64) -> LetterGrade {
    if added_latency_ms < 3.0 {
        LetterGrade::APlus
    } else if added_latency_ms < 5.0 {
        LetterGrade::A
    } else if added_latency_ms < 10.0 {
        LetterGrade::AMinus
    } else if added_latency_ms < 20.0 {
        LetterGrade::BPlus
    } else if added_latency_ms < 30.0 {
        LetterGrade::B
    } else if added_latency_ms < 50.0 {
        LetterGrade::C
    } else if added_latency_ms < 100.0 {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Grade stability based on CV% (lower = more stable).
pub fn grade_stability(cv_pct: f64) -> LetterGrade {
    if cv_pct < 3.0 {
        LetterGrade::APlus
    } else if cv_pct < 5.0 {
        LetterGrade::A
    } else if cv_pct < 8.0 {
        LetterGrade::B
    } else if cv_pct < 15.0 {
        LetterGrade::C
    } else if cv_pct < 25.0 {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Compute overall connection grade from individual grades and profile weights.
pub fn grade_overall(
    ping: Option<f64>,
    jitter: Option<f64>,
    download_bps: Option<f64>,
    upload_bps: Option<f64>,
    profile: UserProfile,
) -> LetterGrade {
    let (ping_w, jitter_w, dl_w, ul_w) = profile.scoring_weights();
    let mut total_score = 0.0;
    let mut total_weight = 0.0;

    if let Some(p) = ping {
        total_score += grade_ping(p, profile).score() * ping_w;
        total_weight += ping_w;
    }
    if let Some(j) = jitter {
        total_score += grade_jitter(j, profile).score() * jitter_w;
        total_weight += jitter_w;
    }
    if let Some(dl) = download_bps {
        total_score += grade_download(dl / 1_000_000.0, profile).score() * dl_w;
        total_weight += dl_w;
    }
    if let Some(ul) = upload_bps {
        total_score += grade_upload(ul / 1_000_000.0, profile).score() * ul_w;
        total_weight += ul_w;
    }

    if total_weight == 0.0 {
        return LetterGrade::F;
    }

    let avg_score = total_score / total_weight;
    score_to_grade(avg_score)
}

/// Convert a numeric score (0-100) to a letter grade.
pub fn score_to_grade(score: f64) -> LetterGrade {
    if score >= 97.0 {
        LetterGrade::APlus
    } else if score >= 93.0 {
        LetterGrade::A
    } else if score >= 90.0 {
        LetterGrade::AMinus
    } else if score >= 87.0 {
        LetterGrade::BPlus
    } else if score >= 80.0 {
        LetterGrade::B
    } else if score >= 77.0 {
        LetterGrade::BMinus
    } else if score >= 70.0 {
        LetterGrade::CPlus
    } else if score >= 65.0 {
        LetterGrade::C
    } else if score >= 60.0 {
        LetterGrade::CMinus
    } else if score >= 50.0 {
        LetterGrade::D
    } else {
        LetterGrade::F
    }
}

/// Format a grade line with label, grade, and optional value.
pub fn format_grade_line(
    label: &str,
    grade: LetterGrade,
    value: Option<&str>,
    nc: bool,
    theme: Theme,
) -> String {
    let emoji = grade.emoji();
    let grade_display = grade.color_str(nc, theme);
    let value_str = value.map(|v| format!(" ({v})")).unwrap_or_default();

    if nc || terminal::no_emoji() {
        format!("  {:>14}:   {grade_display}{value_str}", label)
    } else {
        format!("  {:>14}:   {emoji} {grade_display}{value_str}", label)
    }
}

pub fn grade_badge(grade: LetterGrade, nc: bool, theme: Theme) -> String {
    let emoji = grade.emoji();
    let grade_display = grade.color_str(nc, theme);
    if nc {
        format!("[{grade_display}]")
    } else if terminal::no_emoji() {
        grade_display.to_string()
    } else {
        format!("{emoji} {grade_display}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_grade_ordering() {
        // LetterGrade derives PartialOrd based on enum order
        // APlus=0, A=1, ... F=10, so lower discriminant = "greater"
        assert!(LetterGrade::APlus.score() > LetterGrade::A.score());
        assert!(LetterGrade::A.score() > LetterGrade::AMinus.score());
        assert!(LetterGrade::B.score() > LetterGrade::C.score());
        assert!(LetterGrade::F.score() < LetterGrade::D.score());
    }

    #[test]
    fn test_grade_ping() {
        let p = UserProfile::PowerUser;
        assert_eq!(grade_ping(3.0, p), LetterGrade::APlus);
        assert_eq!(grade_ping(10.0, p), LetterGrade::A);
        assert!(grade_ping(100.0, p).score() <= LetterGrade::D.score());
        assert_eq!(grade_ping(500.0, p), LetterGrade::F);
    }

    #[test]
    fn test_grade_ping_gamer() {
        let g = UserProfile::Gamer;
        // Gamer has stricter thresholds
        assert_eq!(grade_ping(3.0, g), LetterGrade::A); // 3ms is good for gamer (5ms excellent)
        assert!(grade_ping(50.0, g).score() <= LetterGrade::F.score());
    }

    #[test]
    fn test_grade_jitter() {
        let p = UserProfile::PowerUser;
        assert_eq!(grade_jitter(0.5, p), LetterGrade::APlus);
        assert_eq!(grade_jitter(2.0, p), LetterGrade::A);
        assert!(grade_jitter(50.0, p) == LetterGrade::F);
    }

    #[test]
    fn test_grade_download() {
        let p = UserProfile::PowerUser;
        assert_eq!(grade_download(1000.0, p), LetterGrade::APlus); // 1 Gbps
        assert_eq!(grade_download(500.0, p), LetterGrade::A);
        assert!(grade_download(50.0, p).score() <= LetterGrade::D.score());
        assert_eq!(grade_download(1.0, p), LetterGrade::F);
    }

    #[test]
    fn test_grade_download_streamer() {
        let s = UserProfile::Streamer;
        // Streamer has lower excellent threshold (200 vs 500)
        assert_eq!(grade_download(200.0, s), LetterGrade::A);
    }

    #[test]
    fn test_grade_upload() {
        let p = UserProfile::PowerUser;
        assert_eq!(grade_upload(500.0, p), LetterGrade::APlus);
        assert_eq!(grade_upload(250.0, p), LetterGrade::A);
        assert_eq!(grade_upload(1.0, p), LetterGrade::F);
    }

    #[test]
    fn test_grade_bufferbloat() {
        assert_eq!(grade_bufferbloat(1.0), LetterGrade::APlus);
        assert_eq!(grade_bufferbloat(4.0), LetterGrade::A);
        assert_eq!(grade_bufferbloat(25.0), LetterGrade::B);
        assert_eq!(grade_bufferbloat(75.0), LetterGrade::D);
        assert_eq!(grade_bufferbloat(200.0), LetterGrade::F);
    }

    #[test]
    fn test_grade_stability() {
        assert_eq!(grade_stability(1.0), LetterGrade::APlus);
        assert_eq!(grade_stability(4.0), LetterGrade::A);
        assert_eq!(grade_stability(10.0), LetterGrade::C);
        assert_eq!(grade_stability(50.0), LetterGrade::F);
    }

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(98.0), LetterGrade::APlus);
        assert_eq!(score_to_grade(95.0), LetterGrade::A);
        assert_eq!(score_to_grade(87.0), LetterGrade::BPlus);
        assert_eq!(score_to_grade(70.0), LetterGrade::CPlus);
        assert_eq!(score_to_grade(50.0), LetterGrade::D);
        assert_eq!(score_to_grade(25.0), LetterGrade::F);
    }

    #[test]
    fn test_grade_overall_excellent() {
        let p = UserProfile::PowerUser;
        let grade = grade_overall(
            Some(5.0),           // Excellent ping
            Some(1.0),           // Excellent jitter
            Some(600_000_000.0), // Excellent download
            Some(300_000_000.0), // Excellent upload
            p,
        );
        assert!(grade.score() >= LetterGrade::A.score());
    }

    #[test]
    fn test_grade_overall_poor() {
        let p = UserProfile::PowerUser;
        let grade = grade_overall(
            Some(200.0),       // Bad ping
            Some(50.0),        // Bad jitter
            Some(5_000_000.0), // Bad download
            Some(1_000_000.0), // Bad upload
            p,
        );
        assert!(grade.score() <= LetterGrade::F.score());
    }

    #[test]
    fn test_format_grade_line() {
        let grade = LetterGrade::A;
        let line = format_grade_line("Ping", grade, Some("5.2 ms"), false, Theme::Dark);
        assert!(line.contains("Ping"));
        assert!(line.contains("5.2 ms"));
    }

    #[test]
    fn test_grade_badge() {
        let badge = grade_badge(LetterGrade::A, false, Theme::Dark);
        assert!(badge.contains('A'));
    }
}
