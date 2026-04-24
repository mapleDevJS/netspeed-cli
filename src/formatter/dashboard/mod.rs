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

mod helpers;
mod render;

pub use helpers::Summary;

use crate::formatter::grades;
use crate::terminal;
use crate::theme::Colors;
use crate::types::TestResult;
use owo_colors::OwoColorize;

use render::{
    build_capability_matrix, build_header, build_metrics_dashboard, build_transfer_estimates,
};

/// Format the full dashboard output V2.
///
/// # Errors
///
/// This function does not currently return errors, but the signature is
/// `Result` for future extensibility.
pub fn show(result: &TestResult, summary: &Summary) -> Result<(), crate::error::Error> {
    let nc = terminal::no_color();
    let theme = summary.theme;
    let dl_mbps = summary.dl_mbps;
    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        summary.profile,
    );

    eprintln!();
    eprintln!("{}", build_header(result, nc, theme));
    eprintln!();
    eprintln!("{}", build_metrics_dashboard(result, summary, nc, theme));
    eprintln!();
    eprintln!("{}", build_capability_matrix(dl_mbps, nc, theme));
    let estimates = build_transfer_estimates(dl_mbps, nc, theme);
    if !estimates.is_empty() {
        eprintln!("{estimates}");
    }
    eprintln!();

    if nc {
        eprintln!(
            "Grade: [{}]  ·  Completed at: {}",
            overall_grade.as_str(),
            result.timestamp
        );
    } else {
        eprintln!(
            "{}  ·  {} {}",
            overall_grade.color_str(nc, theme),
            "Completed at:".dimmed(),
            Colors::muted(&result.timestamp, theme),
        );
    }
    eprintln!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::UserProfile;
    use crate::theme::Theme;
    use crate::types::{PhaseResult, ServerInfo, TestPhases};

    fn make_result() -> TestResult {
        TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
            server: ServerInfo {
                id: "1".to_string(),
                name: "TestServer".to_string(),
                sponsor: "TestISP".to_string(),
                country: "US".to_string(),
                distance: 15.0,
            },
            ping: Some(12.0),
            jitter: Some(1.5),
            packet_loss: Some(0.0),
            download: Some(150_000_000.0),
            download_peak: Some(180_000_000.0),
            upload: Some(50_000_000.0),
            upload_peak: Some(60_000_000.0),
            latency_download: Some(18.0),
            latency_upload: Some(15.0),
            download_samples: Some(vec![140_000_000.0, 150_000_000.0, 160_000_000.0]),
            upload_samples: Some(vec![48_000_000.0, 50_000_000.0, 52_000_000.0]),
            ping_samples: Some(vec![11.0, 12.0, 13.0]),
            timestamp: "2026-04-06T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.100".to_string()),
            client_location: None,
            download_cv: Some(0.067),
            upload_cv: Some(0.04),
            download_ci_95: Some((140.0, 160.0)),
            upload_ci_95: Some((48.0, 52.0)),
            overall_grade: Some("B+".to_string()),
            download_grade: Some("B+".to_string()),
            upload_grade: Some("B".to_string()),
            connection_rating: Some("Good".to_string()),
            phases: TestPhases {
                ping: PhaseResult::completed(),
                download: PhaseResult::completed(),
                upload: PhaseResult::completed(),
            },
        }
    }

    #[test]
    fn test_mini_bar_full() {
        let bar = helpers::mini_bar(10, 10, true, 100.0);
        assert_eq!(bar, "[##########]");
    }

    #[test]
    fn test_mini_bar_empty() {
        let bar = helpers::mini_bar(0, 10, true, 0.0);
        assert_eq!(bar, "[----------]");
    }

    #[test]
    fn test_mini_bar_half() {
        let bar = helpers::mini_bar(5, 10, true, 50.0);
        assert_eq!(bar, "[#####-----]");
    }

    #[test]
    fn test_build_header() {
        let result = make_result();
        let header = build_header(&result, true, Theme::Dark);
        assert!(header.contains("NetSpeed"));
        assert!(header.contains("TestISP"));
        assert!(header.starts_with("╭"));
        assert!(header.contains("╮"));
    }

    #[test]
    fn test_build_metrics_dashboard() {
        let result = make_result();
        let summary = Summary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
            elapsed: std::time::Duration::from_secs(10),
            profile: UserProfile::default(),
            theme: Theme::Dark,
        };
        let output = build_metrics_dashboard(&result, &summary, true, Theme::Dark);
        assert!(output.contains("PERFORMANCE"));
        assert!(output.contains("STABILITY"));
        assert!(output.contains("BUFFERBLOAT"));
        assert!(output.contains("Overall:"));
    }

    #[test]
    fn test_build_capability_matrix() {
        let output = build_capability_matrix(500.0, true, Theme::Dark);
        assert!(output.contains("CAPABILITY MATRIX"));
        assert!(output.contains("Communication"));
        assert!(output.contains("Streaming"));
        assert!(output.contains("AI/ML"));
    }

    #[test]
    fn test_build_transfer_estimates() {
        let output = build_transfer_estimates(281.0, true, Theme::Dark);
        assert!(output.contains("TRANSFER ESTIMATES"));
        assert!(output.contains("5 MB"));
        assert!(output.contains("4 GB"));
    }

    #[test]
    fn test_format_dashboard_integration() {
        let result = make_result();
        let summary = Summary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
            elapsed: std::time::Duration::from_secs(10),
            profile: UserProfile::default(),
            theme: Theme::Dark,
        };
        show(&result, &summary).unwrap();
    }

    #[test]
    fn test_format_dashboard_no_color() {
        let result = make_result();
        let summary = Summary {
            dl_mbps: 150.0,
            dl_peak_mbps: 180.0,
            dl_bytes: 15_000_000,
            dl_duration: 3.2,
            ul_mbps: 50.0,
            ul_peak_mbps: 60.0,
            ul_bytes: 5_000_000,
            ul_duration: 2.1,
            elapsed: std::time::Duration::from_secs(10),
            profile: UserProfile::default(),
            theme: Theme::Dark,
        };
        // SAFETY: This test runs single-threaded; no concurrent tokio tasks
        // can race on the env var read/write.
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        show(&result, &summary).unwrap();
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn test_format_time_short() {
        assert!(render::format_time_short(0.5).contains("0.5s"));
        assert!(render::format_time_short(30.0).contains("30s"));
        assert!(render::format_time_short(120.0).contains("2m"));
        assert!(render::format_time_short(7260.0).contains("2h"));
    }
}
