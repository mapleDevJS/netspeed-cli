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
use crate::theme::{Colors, Theme};

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
    #[must_use]
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
    #[must_use]
    pub fn color_str(&self, nc: bool, theme: Theme) -> String {
        let s = self.as_str();
        if nc {
            return format!("[{s}]");
        }
        match self {
            Self::APlus | Self::A | Self::AMinus | Self::BPlus | Self::B => Colors::good(s, theme),
            Self::BMinus | Self::CPlus | Self::C | Self::CMinus | Self::D => Colors::warn(s, theme),
            Self::F => Colors::bad(s, theme),
        }
    }

    /// Emoji indicator for the grade.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
#[must_use]
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
        format!("  {label:>14}:   {grade_display}{value_str}")
    } else {
        format!("  {label:>14}:   {emoji} {grade_display}{value_str}")
    }
}

#[must_use]
pub fn grade_badge(grade: LetterGrade, nc: bool, theme: Theme) -> String {
    let emoji = grade.emoji();
    let grade_display = grade.color_str(nc, theme);
    if nc {
        format!("[{grade_display}]")
    } else if terminal::no_emoji() {
        grade_display.clone()
    } else {
        format!("{emoji} {grade_display}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    // ── LetterGrade enum tests ──────────────────────────────────────────────

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
    fn test_letter_grade_as_str_all_variants() {
        assert_eq!(LetterGrade::APlus.as_str(), "A+");
        assert_eq!(LetterGrade::A.as_str(), "A");
        assert_eq!(LetterGrade::AMinus.as_str(), "A-");
        assert_eq!(LetterGrade::BPlus.as_str(), "B+");
        assert_eq!(LetterGrade::B.as_str(), "B");
        assert_eq!(LetterGrade::BMinus.as_str(), "B-");
        assert_eq!(LetterGrade::CPlus.as_str(), "C+");
        assert_eq!(LetterGrade::C.as_str(), "C");
        assert_eq!(LetterGrade::CMinus.as_str(), "C-");
        assert_eq!(LetterGrade::D.as_str(), "D");
        assert_eq!(LetterGrade::F.as_str(), "F");
    }

    #[test]
    fn test_letter_grade_score_all_variants() {
        assert_eq!(LetterGrade::APlus.score(), 100.0);
        assert_eq!(LetterGrade::A.score(), 95.0);
        assert_eq!(LetterGrade::AMinus.score(), 90.0);
        assert_eq!(LetterGrade::BPlus.score(), 85.0);
        assert_eq!(LetterGrade::B.score(), 80.0);
        assert_eq!(LetterGrade::BMinus.score(), 75.0);
        assert_eq!(LetterGrade::CPlus.score(), 70.0);
        assert_eq!(LetterGrade::C.score(), 65.0);
        assert_eq!(LetterGrade::CMinus.score(), 60.0);
        assert_eq!(LetterGrade::D.score(), 50.0);
        assert_eq!(LetterGrade::F.score(), 25.0);
    }

    #[test]
    fn test_letter_grade_description_all_variants() {
        assert_eq!(LetterGrade::APlus.description(), "Exceptional");
        assert_eq!(LetterGrade::A.description(), "Excellent");
        assert_eq!(LetterGrade::AMinus.description(), "Very Good");
        assert_eq!(LetterGrade::BPlus.description(), "Good+");
        assert_eq!(LetterGrade::B.description(), "Good");
        assert_eq!(LetterGrade::BMinus.description(), "Above Average");
        assert_eq!(LetterGrade::CPlus.description(), "Average+");
        assert_eq!(LetterGrade::C.description(), "Average");
        assert_eq!(LetterGrade::CMinus.description(), "Below Average");
        assert_eq!(LetterGrade::D.description(), "Poor");
        assert_eq!(LetterGrade::F.description(), "Unacceptable");
    }

    #[test]
    fn test_letter_grade_color_str_nc_mode() {
        // In NC mode, should return bracketed string
        assert_eq!(LetterGrade::A.color_str(true, Theme::Dark), "[A]");
        assert_eq!(LetterGrade::F.color_str(true, Theme::Dark), "[F]");
        assert_eq!(LetterGrade::BPlus.color_str(true, Theme::Dark), "[B+]");
    }

    #[test]
    fn test_letter_grade_color_str_all_grades() {
        // Test all grades have color output (not empty in non-NC mode)
        for grade in &[
            LetterGrade::APlus,
            LetterGrade::A,
            LetterGrade::AMinus,
            LetterGrade::BPlus,
            LetterGrade::B,
            LetterGrade::BMinus,
            LetterGrade::CPlus,
            LetterGrade::C,
            LetterGrade::CMinus,
            LetterGrade::D,
            LetterGrade::F,
        ] {
            let colored = grade.color_str(false, Theme::Dark);
            assert!(
                !colored.is_empty(),
                "Grade {:?} should have color output",
                grade
            );
            assert!(
                colored.contains(grade.as_str()),
                "Colored output should contain grade string"
            );
        }
    }

    #[test]
    fn test_letter_grade_color_str_different_themes() {
        let grade = LetterGrade::A;
        // Test with different themes
        let dark = grade.color_str(false, Theme::Dark);
        let light = grade.color_str(false, Theme::Light);
        assert!(!dark.is_empty());
        assert!(!light.is_empty());
        // Both should contain the grade letter
        assert!(dark.contains("A"));
        assert!(light.contains("A"));
    }

    // ── grade_ping tests ────────────────────────────────────────────────────

    #[test]
    fn test_grade_ping_excellent() {
        let p = UserProfile::PowerUser;
        // PowerUser has 10ms excellent threshold
        // APlus: <= 5ms
        assert_eq!(grade_ping(3.0, p), LetterGrade::APlus);
        assert_eq!(grade_ping(5.0, p), LetterGrade::APlus);
        // A: <= 10ms
        assert_eq!(grade_ping(6.0, p), LetterGrade::A);
        assert_eq!(grade_ping(10.0, p), LetterGrade::A);
        // A-: <= 15ms
        assert_eq!(grade_ping(11.0, p), LetterGrade::AMinus);
        assert_eq!(grade_ping(15.0, p), LetterGrade::AMinus);
    }

    #[test]
    fn test_grade_ping_good_to_fail() {
        let p = UserProfile::PowerUser;
        // B: <= 30ms (good = 3 * 10 = 30)
        assert_eq!(grade_ping(20.0, p), LetterGrade::B);
        assert_eq!(grade_ping(30.0, p), LetterGrade::B);
        // C: <= 45ms (good * 1.5)
        assert_eq!(grade_ping(40.0, p), LetterGrade::C);
        assert_eq!(grade_ping(45.0, p), LetterGrade::C);
        // D: <= 60ms (average = 6 * 10 = 60)
        assert_eq!(grade_ping(50.0, p), LetterGrade::D);
        assert_eq!(grade_ping(60.0, p), LetterGrade::D);
        // F: > 60ms
        assert_eq!(grade_ping(61.0, p), LetterGrade::F);
        assert_eq!(grade_ping(500.0, p), LetterGrade::F);
    }

    #[test]
    fn test_grade_ping_gamer_profile() {
        let g = UserProfile::Gamer;
        // Gamer has 5ms excellent threshold
        // APlus: <= 2.5ms
        assert_eq!(grade_ping(2.0, g), LetterGrade::APlus);
        // A: <= 5ms
        assert_eq!(grade_ping(3.0, g), LetterGrade::A);
        assert_eq!(grade_ping(5.0, g), LetterGrade::A);
        // A-: <= 7.5ms
        assert_eq!(grade_ping(7.0, g), LetterGrade::AMinus);
        // B: <= 15ms (good = 3 * 5 = 15)
        assert_eq!(grade_ping(10.0, g), LetterGrade::B);
        // F: > 30ms (average = 6 * 5 = 30)
        assert_eq!(grade_ping(100.0, g), LetterGrade::F);
    }

    #[test]
    fn test_grade_ping_streamer_profile() {
        let s = UserProfile::Streamer;
        // Streamer has 30ms excellent threshold (2x Gamer)
        // APlus: <= 15ms, A: <= 30ms, A-: <= 45ms
        assert_eq!(grade_ping(10.0, s), LetterGrade::APlus);
        assert_eq!(grade_ping(30.0, s), LetterGrade::A);
        assert_eq!(grade_ping(200.0, s), LetterGrade::F);
    }

    // ── grade_jitter tests ──────────────────────────────────────────────────

    #[test]
    fn test_grade_jitter_excellent() {
        let p = UserProfile::PowerUser;
        // PowerUser has 2ms excellent threshold
        // APlus: <= 1ms
        assert_eq!(grade_jitter(0.5, p), LetterGrade::APlus);
        assert_eq!(grade_jitter(1.0, p), LetterGrade::APlus);
        // A: <= 2ms
        assert_eq!(grade_jitter(1.5, p), LetterGrade::A);
        assert_eq!(grade_jitter(2.0, p), LetterGrade::A);
        // B: <= 4ms (excellent * 2.0)
        assert_eq!(grade_jitter(3.0, p), LetterGrade::B);
        assert_eq!(grade_jitter(4.0, p), LetterGrade::B);
    }

    #[test]
    fn test_grade_jitter_good_to_fail() {
        let p = UserProfile::PowerUser;
        // C: <= 6ms (good = 3 * 2 = 6)
        assert_eq!(grade_jitter(5.0, p), LetterGrade::C);
        // D: <= 16ms (average = 8 * 2 = 16)
        assert_eq!(grade_jitter(10.0, p), LetterGrade::D);
        assert_eq!(grade_jitter(16.0, p), LetterGrade::D);
        // F: > 16ms
        assert_eq!(grade_jitter(50.0, p), LetterGrade::F);
    }

    // ── grade_download tests ────────────────────────────────────────────────

    #[test]
    fn test_grade_download_excellent() {
        let p = UserProfile::PowerUser;
        // PowerUser has 500 Mbps excellent threshold
        // APlus: >= 1000 Mbps (2x)
        assert_eq!(grade_download(1000.0, p), LetterGrade::APlus);
        assert_eq!(grade_download(500.0, p), LetterGrade::A);
        // B: >= 375 Mbps (0.75x)
        assert_eq!(grade_download(400.0, p), LetterGrade::B);
        assert_eq!(grade_download(375.0, p), LetterGrade::B);
    }

    #[test]
    fn test_grade_download_good_to_fail() {
        let p = UserProfile::PowerUser;
        // C: >= 200 Mbps (0.4x = good)
        assert_eq!(grade_download(200.0, p), LetterGrade::C);
        // D: >= 75 Mbps (0.15x = average)
        assert_eq!(grade_download(75.0, p), LetterGrade::D);
        // F: < 75 Mbps
        assert_eq!(grade_download(50.0, p), LetterGrade::F);
        assert_eq!(grade_download(1.0, p), LetterGrade::F);
    }

    #[test]
    fn test_grade_download_streamer_profile() {
        let s = UserProfile::Streamer;
        // Streamer has 200 Mbps excellent threshold
        // APlus: >= 400 Mbps, A: >= 200 Mbps
        // B: >= 150 Mbps (0.75x), C: >= 80 Mbps (0.4x), D: >= 30 Mbps (0.15x)
        assert_eq!(grade_download(400.0, s), LetterGrade::APlus);
        assert_eq!(grade_download(200.0, s), LetterGrade::A);
        // 100 < 150, so C (100 >= 80)
        assert_eq!(grade_download(100.0, s), LetterGrade::C);
        // 80 >= 80 → C boundary
        assert_eq!(grade_download(80.0, s), LetterGrade::C);
        assert_eq!(grade_download(40.0, s), LetterGrade::D);
        assert_eq!(grade_download(10.0, s), LetterGrade::F);
    }

    #[test]
    fn test_grade_download_gamer_profile() {
        let g = UserProfile::Gamer;
        // Gamer has 100 Mbps excellent threshold
        assert_eq!(grade_download(200.0, g), LetterGrade::APlus);
        assert_eq!(grade_download(100.0, g), LetterGrade::A);
        assert_eq!(grade_download(50.0, g), LetterGrade::C);
    }

    // ── grade_upload tests ──────────────────────────────────────────────────

    #[test]
    fn test_grade_upload_excellent() {
        let p = UserProfile::PowerUser;
        // Upload excellent is 50% of download (250 Mbps for PowerUser)
        assert_eq!(grade_upload(500.0, p), LetterGrade::APlus);
        assert_eq!(grade_upload(250.0, p), LetterGrade::A);
        // B: >= 187.5 Mbps (0.75x)
        assert_eq!(grade_upload(200.0, p), LetterGrade::B);
    }

    #[test]
    fn test_grade_upload_good_to_fail() {
        let p = UserProfile::PowerUser;
        // C: >= 100 Mbps (0.4x)
        assert_eq!(grade_upload(100.0, p), LetterGrade::C);
        // D: >= 37.5 Mbps (0.15x)
        assert_eq!(grade_upload(40.0, p), LetterGrade::D);
        // F: < 37.5 Mbps
        assert_eq!(grade_upload(1.0, p), LetterGrade::F);
    }

    // ── grade_bufferbloat tests ─────────────────────────────────────────────

    #[test]
    fn test_grade_bufferbloat_all_levels() {
        // APlus: < 3ms
        assert_eq!(grade_bufferbloat(0.0), LetterGrade::APlus);
        assert_eq!(grade_bufferbloat(2.0), LetterGrade::APlus);
        // A: 3-5ms
        assert_eq!(grade_bufferbloat(3.0), LetterGrade::A);
        assert_eq!(grade_bufferbloat(4.0), LetterGrade::A);
        // A-: 5-10ms
        assert_eq!(grade_bufferbloat(5.0), LetterGrade::AMinus);
        assert_eq!(grade_bufferbloat(8.0), LetterGrade::AMinus);
        // B+: 10-20ms
        assert_eq!(grade_bufferbloat(15.0), LetterGrade::BPlus);
        // B: 20-30ms
        assert_eq!(grade_bufferbloat(25.0), LetterGrade::B);
        // C: 30-50ms
        assert_eq!(grade_bufferbloat(40.0), LetterGrade::C);
        // D: 50-100ms
        assert_eq!(grade_bufferbloat(75.0), LetterGrade::D);
        // F: >= 100ms
        assert_eq!(grade_bufferbloat(100.0), LetterGrade::F);
        assert_eq!(grade_bufferbloat(200.0), LetterGrade::F);
    }

    #[test]
    fn test_grade_bufferbloat_boundary_cases() {
        // Exact boundary values
        assert_eq!(grade_bufferbloat(2.999), LetterGrade::APlus);
        assert_eq!(grade_bufferbloat(3.001), LetterGrade::A);
        assert_eq!(grade_bufferbloat(4.999), LetterGrade::A);
        assert_eq!(grade_bufferbloat(5.001), LetterGrade::AMinus);
        assert_eq!(grade_bufferbloat(9.999), LetterGrade::AMinus);
        assert_eq!(grade_bufferbloat(10.001), LetterGrade::BPlus);
    }

    // ── grade_stability tests ───────────────────────────────────────────────

    #[test]
    fn test_grade_stability_all_levels() {
        // APlus: < 3%, A: 3-5%, B: 5-8%, C: 8-15%, D: 15-25%, F: >= 25%
        assert_eq!(grade_stability(0.0), LetterGrade::APlus);
        assert_eq!(grade_stability(2.0), LetterGrade::APlus);
        assert_eq!(grade_stability(3.0), LetterGrade::A);
        assert_eq!(grade_stability(4.0), LetterGrade::A);
        assert_eq!(grade_stability(5.0), LetterGrade::B);
        assert_eq!(grade_stability(7.0), LetterGrade::B);
        assert_eq!(grade_stability(10.0), LetterGrade::C);
        assert_eq!(grade_stability(20.0), LetterGrade::D);
        assert_eq!(grade_stability(50.0), LetterGrade::F);
    }

    // ── score_to_grade tests ────────────────────────────────────────────────

    #[test]
    fn test_score_to_grade_all_levels() {
        // APlus: >= 97
        assert_eq!(score_to_grade(97.0), LetterGrade::APlus);
        assert_eq!(score_to_grade(100.0), LetterGrade::APlus);
        // A: >= 93
        assert_eq!(score_to_grade(93.0), LetterGrade::A);
        assert_eq!(score_to_grade(96.99), LetterGrade::A);
        // A-: >= 90
        assert_eq!(score_to_grade(90.0), LetterGrade::AMinus);
        assert_eq!(score_to_grade(92.99), LetterGrade::AMinus);
        // B+: >= 87
        assert_eq!(score_to_grade(87.0), LetterGrade::BPlus);
        assert_eq!(score_to_grade(89.99), LetterGrade::BPlus);
        // B: >= 80
        assert_eq!(score_to_grade(80.0), LetterGrade::B);
        assert_eq!(score_to_grade(86.99), LetterGrade::B);
        // B-: >= 77
        assert_eq!(score_to_grade(77.0), LetterGrade::BMinus);
        assert_eq!(score_to_grade(79.99), LetterGrade::BMinus);
        // C+: >= 70
        assert_eq!(score_to_grade(70.0), LetterGrade::CPlus);
        assert_eq!(score_to_grade(76.99), LetterGrade::CPlus);
        // C: >= 65
        assert_eq!(score_to_grade(65.0), LetterGrade::C);
        assert_eq!(score_to_grade(69.99), LetterGrade::C);
        // C-: >= 60
        assert_eq!(score_to_grade(60.0), LetterGrade::CMinus);
        assert_eq!(score_to_grade(64.99), LetterGrade::CMinus);
        // D: >= 50
        assert_eq!(score_to_grade(50.0), LetterGrade::D);
        assert_eq!(score_to_grade(59.99), LetterGrade::D);
        // F: < 50
        assert_eq!(score_to_grade(49.99), LetterGrade::F);
        assert_eq!(score_to_grade(0.0), LetterGrade::F);
    }

    #[test]
    fn test_score_to_grade_boundary_cases() {
        // Thresholds: A+>=97, A>=93, A->=90, B+>=87, B>=80, B->=77, C+>=70, C>=65, C->=60, D>=50, F<50
        assert_eq!(score_to_grade(96.99), LetterGrade::A); // 96.99 < 97
        assert_eq!(score_to_grade(92.99), LetterGrade::AMinus); // 92.99 < 93
        assert_eq!(score_to_grade(86.99), LetterGrade::B); // 86.99 < 87
        assert_eq!(score_to_grade(79.99), LetterGrade::BMinus); // 79.99 < 80 but >= 77
        assert_eq!(score_to_grade(76.99), LetterGrade::CPlus); // 76.99 < 77
        assert_eq!(score_to_grade(69.99), LetterGrade::C); // 69.99 < 70
        assert_eq!(score_to_grade(64.99), LetterGrade::CMinus); // 64.99 < 65
        assert_eq!(score_to_grade(59.99), LetterGrade::D); // 59.99 >= 50
    }

    // ── grade_overall tests ─────────────────────────────────────────────────

    #[test]
    fn test_grade_overall_all_none() {
        let p = UserProfile::PowerUser;
        // All None should return F
        let grade = grade_overall(None, None, None, None, p);
        assert_eq!(grade, LetterGrade::F);
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
    fn test_grade_overall_partial_data() {
        let p = UserProfile::PowerUser;
        // Only ping and download
        let grade = grade_overall(Some(5.0), None, Some(600_000_000.0), None, p);
        assert!(grade.score() >= LetterGrade::A.score());

        // Only download
        let grade = grade_overall(None, None, Some(600_000_000.0), None, p);
        assert!(grade.score() >= LetterGrade::A.score());

        // Only ping
        let grade = grade_overall(Some(5.0), None, None, None, p);
        assert!(grade.score() >= LetterGrade::A.score());

        // Only jitter
        let grade = grade_overall(None, Some(1.0), None, None, p);
        assert!(grade.score() >= LetterGrade::A.score());
    }

    #[test]
    fn test_grade_overall_mixed_quality() {
        let p = UserProfile::PowerUser;
        // Good ping, bad download
        let grade = grade_overall(Some(5.0), None, Some(5_000_000.0), None, p);
        // Should be in D-F range (bad download drags down good ping)
        assert!(grade.score() <= LetterGrade::D.score());
        assert!(grade.score() >= LetterGrade::F.score());
    }

    // ── format_grade_line tests ─────────────────────────────────────────────

    #[test]
    fn test_format_grade_line_with_value() {
        let grade = LetterGrade::A;
        let line = format_grade_line("Ping", grade, Some("5.2 ms"), false, Theme::Dark);
        assert!(line.contains("Ping"));
        assert!(line.contains("5.2 ms"));
        assert!(line.contains("A"));
    }

    #[test]
    fn test_format_grade_line_without_value() {
        let grade = LetterGrade::B;
        let line = format_grade_line("Jitter", grade, None, false, Theme::Dark);
        assert!(line.contains("Jitter"));
        assert!(line.contains("B"));
        // Should not have extra parentheses without value
        assert!(!line.contains("()"));
    }

    #[test]
    fn test_format_grade_line_nc_mode() {
        let grade = LetterGrade::C;
        let line = format_grade_line("Download", grade, Some("100 Mbps"), true, Theme::Dark);
        // In NC mode, should use brackets
        assert!(line.contains("["));
        assert!(line.contains("]"));
    }

    #[test]
    fn test_format_grade_line_all_grade_levels() {
        // Test all grade levels produce valid output
        let metrics = ["Ping", "Jitter", "Download", "Upload", "Bufferbloat"];
        for grade in &[
            LetterGrade::APlus,
            LetterGrade::A,
            LetterGrade::B,
            LetterGrade::C,
            LetterGrade::D,
            LetterGrade::F,
        ] {
            for metric in &metrics {
                let line = format_grade_line(metric, *grade, Some("value"), false, Theme::Dark);
                assert!(!line.is_empty());
            }
        }
    }

    // ── grade_badge tests ───────────────────────────────────────────────────

    #[test]
    fn test_grade_badge_nc_mode() {
        let badge = grade_badge(LetterGrade::A, true, Theme::Dark);
        // NC mode uses brackets, format: [grade_str]
        assert!(badge.starts_with('['));
        assert!(badge.ends_with(']'));
        assert!(badge.contains('A'));
    }

    #[test]
    fn test_grade_badge_colored_mode() {
        let badge = grade_badge(LetterGrade::B, false, Theme::Dark);
        assert!(badge.contains('B'));
        assert!(!badge.is_empty());
    }

    #[test]
    fn test_grade_badge_all_grades() {
        for grade in &[
            LetterGrade::APlus,
            LetterGrade::A,
            LetterGrade::B,
            LetterGrade::C,
            LetterGrade::F,
        ] {
            let badge = grade_badge(*grade, false, Theme::Dark);
            assert!(!badge.is_empty());
            assert!(badge.contains(grade.as_str()));
        }
    }

    // ── Copy trait verification ─────────────────────────────────────────────

    #[test]
    fn test_letter_grade_is_copy() {
        // Verify LetterGrade implements Copy by being able to copy assign
        let grade: LetterGrade = LetterGrade::A;
        let _copied = grade; // This only works if Copy is implemented
        assert_eq!(grade, _copied);
    }

    // ── Clone trait verification ────────────────────────────────────────────

    #[test]
    fn test_letter_grade_is_clone() {
        let grade = LetterGrade::BPlus;
        let cloned = grade.clone();
        assert_eq!(grade, cloned);
    }

    // ── Debug trait verification ────────────────────────────────────────────

    #[test]
    fn test_letter_grade_debug() {
        let grade = LetterGrade::C;
        let debug_str = format!("{:?}", grade);
        assert!(debug_str.contains("C"));
    }

    // ── PartialEq and Eq verification ───────────────────────────────────────

    #[test]
    fn test_letter_grade_partial_eq() {
        assert_eq!(LetterGrade::A, LetterGrade::A);
        assert_ne!(LetterGrade::A, LetterGrade::B);
        assert_eq!(LetterGrade::APlus, LetterGrade::APlus);
    }

    // ── Ord verification ────────────────────────────────────────────────────

    #[test]
    fn test_letter_grade_ord() {
        assert!(LetterGrade::APlus < LetterGrade::A);
        assert!(LetterGrade::A < LetterGrade::B);
        assert!(LetterGrade::B < LetterGrade::C);
        assert!(LetterGrade::C < LetterGrade::D);
        assert!(LetterGrade::D < LetterGrade::F);
    }
}
