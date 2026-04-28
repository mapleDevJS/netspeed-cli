//! Usage check targets and real-world download time estimates.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::common;
use crate::terminal;
use crate::theme::{Colors, Theme};

// ── Target benchmarks ────────────────────────────────────────────────

struct Target {
    name: &'static str,
    required_mbps: f64,
}

const TARGETS: &[Target] = &[
    Target {
        name: "Video calls (1080p)",
        required_mbps: 3.0,
    },
    Target {
        name: "HD streaming",
        required_mbps: 5.0,
    },
    Target {
        name: "4K streaming",
        required_mbps: 25.0,
    },
    Target {
        name: "Cloud gaming",
        required_mbps: 35.0,
    },
    Target {
        name: "Large file transfers",
        required_mbps: 100.0,
    },
];

/// Build target usage check output as a string.
#[must_use]
pub fn build_targets(download_bps: Option<f64>, nc: bool, theme: Theme) -> String {
    let targets: Vec<crate::profiles::UsageTarget> = TARGETS
        .iter()
        .map(|t| crate::profiles::UsageTarget {
            name: t.name,
            required_mbps: t.required_mbps,
            icon: "",
        })
        .collect();
    let dl_mbps = download_bps.map(|d| d / 1_000_000.0);
    build_profile_targets(download_bps, nc, theme, &targets, dl_mbps)
}

/// Build profile-specific target usage check output.
#[must_use]
pub fn build_profile_targets(
    download_bps: Option<f64>,
    nc: bool,
    theme: Theme,
    targets: &[crate::profiles::UsageTarget],
    dl_mbps: Option<f64>,
) -> String {
    let Some(dl) = download_bps else {
        return String::new();
    };
    let dl_mbps = dl_mbps.unwrap_or_else(|| dl / 1_000_000.0);

    let mut lines = Vec::new();

    if nc {
        lines.push("\n  ◈ USAGE CHECK".to_string());
    } else {
        lines.push(format!(
            "\n  {} {}",
            Colors::muted("◈", theme),
            Colors::header("USAGE CHECK", theme)
        ));
    }

    for target in targets {
        let met = dl_mbps >= target.required_mbps;
        let ratio = dl_mbps / target.required_mbps;
        let suffix = if ratio >= 10.0 {
            format!("{ratio:.0}x")
        } else {
            format!("{ratio:.1}x")
        };
        let hide_emoji = terminal::no_emoji();
        let icon = if target.icon.is_empty() {
            "🎯"
        } else {
            target.icon
        };
        if met {
            let status = if hide_emoji { "✓" } else { "✅" };
            let line = format!("{icon} {:<24} {status} {} above", target.name, suffix);
            if nc || hide_emoji {
                lines.push(format!("  {line}"));
            } else {
                lines.push(format!("  {}", Colors::good(&line, theme)));
            }
        } else {
            let shortfall = target.required_mbps - dl_mbps;
            let status = if hide_emoji { "✗" } else { "❌" };
            let line = format!(
                "{icon} {:<24} {status} {:.1} Mb/s short",
                target.name, shortfall
            );
            if nc || hide_emoji {
                lines.push(format!("  {line}"));
            } else {
                lines.push(format!("  {}", Colors::bad(&line, theme)));
            }
        }
    }

    lines.join("\n")
}

/// Format target usage check against download speed.
pub fn format_targets(download_bps: Option<f64>, nc: bool, theme: Theme) {
    let output = build_targets(download_bps, nc, theme);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

// ── Real-world download estimates ──────────────────────────────────────

struct FileEstimate {
    name: &'static str,
    size_bytes: u64,
}

const ESTIMATES: &[FileEstimate] = &[
    FileEstimate {
        name: "Song / Podcast episode",
        size_bytes: 8 * 1024 * 1024,
    },
    FileEstimate {
        name: "Photo (RAW)",
        size_bytes: 30 * 1024 * 1024,
    },
    FileEstimate {
        name: "App install",
        size_bytes: 300 * 1024 * 1024,
    },
    FileEstimate {
        name: "HD movie (1080p)",
        size_bytes: 8 * 1024 * 1024 * 1024,
    },
    FileEstimate {
        name: "4K movie (HDR)",
        size_bytes: 25 * 1024 * 1024 * 1024,
    },
    FileEstimate {
        name: "Game install (AAA)",
        size_bytes: 120 * 1024 * 1024 * 1024,
    },
];

fn format_time_estimate(secs: f64, _nc: bool) -> String {
    if secs < 1.0 {
        format!("{secs:.1}s")
    } else if secs < 60.0 {
        format!("{secs:.0}s")
    } else if secs < 3600.0 {
        // Safe: secs is 60..3600, results fit in u64.
        format!(
            "{}m {:02}s",
            (secs / 60.0).clamp(0.0, u64::MAX as f64) as u64,
            (secs % 60.0).clamp(0.0, u64::MAX as f64) as u64
        )
    } else {
        // Safe: secs is ≥3600 but bounded by test duration (minutes), fits u64.
        format!(
            "{}h {:02}m",
            (secs / 3600.0).clamp(0.0, u64::MAX as f64) as u64,
            ((secs % 3600.0) / 60.0).clamp(0.0, u64::MAX as f64) as u64
        )
    }
}

/// Build real-world download time estimates as a string.
#[must_use]
pub fn build(download_bps: Option<f64>, nc: bool, theme: Theme) -> String {
    let Some(dl) = download_bps else {
        return String::new();
    };
    let dl_bytes_per_sec = dl / 8.0;

    let mut lines = Vec::new();

    if nc {
        lines.push("\n  ◈ ESTIMATES".to_string());
    } else {
        lines.push(format!(
            "\n  {} {}",
            Colors::muted("◈", theme),
            Colors::header("ESTIMATES", theme)
        ));
    }

    for file in ESTIMATES {
        // Safe: file sizes are at most ~120 GB, well under 2^53 (~9 PB).
        let secs = file.size_bytes as f64 / dl_bytes_per_sec;
        let time_str = format_time_estimate(secs, nc);
        let size_str = common::format_data_size(file.size_bytes);
        let label = format!("{:<24} {:>8}   ~{time_str}", file.name, size_str);
        if nc {
            lines.push(format!("  {label}"));
        } else {
            lines.push(format!("  {}", Colors::good(&label, theme)));
        }
    }

    lines.join("\n")
}

pub fn show(download_bps: Option<f64>, nc: bool, theme: Theme) {
    let output = build(download_bps, nc, theme);
    if !output.is_empty() {
        eprintln!("{output}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::UsageTarget;
    use crate::theme::Theme;

    #[test]
    fn test_format_time_estimate() {
        assert!(format_time_estimate(0.5, false).contains("0.5s"));
        assert!(format_time_estimate(30.0, false).contains("30s"));
        assert!(format_time_estimate(120.0, false).contains("2m"));
    }

    #[test]
    fn test_format_time_estimate_sub_second() {
        // Sub-second times should show 1 decimal
        assert_eq!(format_time_estimate(0.1, false), "0.1s");
        assert_eq!(format_time_estimate(0.9, false), "0.9s");
    }

    #[test]
    fn test_format_time_estimate_seconds() {
        // 1 to 59 seconds should show whole number
        assert_eq!(format_time_estimate(1.0, false), "1s");
        assert_eq!(format_time_estimate(45.5, false), "46s");
        assert_eq!(format_time_estimate(59.9, false), "60s");
    }

    #[test]
    fn test_format_time_estimate_minutes() {
        // 60+ seconds should show minutes and seconds
        let result = format_time_estimate(90.0, false);
        assert!(result.contains('m'));

        let result = format_time_estimate(125.5, false);
        assert!(result.contains('m'));
        assert!(result.contains('s'));
    }

    #[test]
    fn test_format_time_estimate_hours() {
        // 3600+ seconds should show hours and minutes
        let result = format_time_estimate(3661.0, false);
        assert!(result.contains('h'));
        assert!(result.contains('m'));
    }

    #[test]
    fn test_build_targets_none_download() {
        // Should return empty string when download is None
        let result = build_targets(None, false, Theme::Dark);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_targets_with_download() {
        // 100 Mbps download
        let result = build_targets(Some(100_000_000.0), false, Theme::Dark);
        assert!(!result.is_empty());
        assert!(result.contains("USAGE CHECK"));
    }

    #[test]
    fn test_build_targets_all_targets_present() {
        let result = build_targets(Some(100_000_000.0), false, Theme::Dark);
        // All target names should be present
        assert!(result.contains("Video calls"));
        assert!(result.contains("HD streaming"));
        assert!(result.contains("4K streaming"));
        assert!(result.contains("Cloud gaming"));
        assert!(result.contains("Large file transfers"));
    }

    #[test]
    fn test_build_targets_excellent_speed() {
        // 500 Mbps should meet all targets
        let result = build_targets(Some(500_000_000.0), false, Theme::Dark);
        // Should show passing indicators for all targets (or error count)
        assert!(!result.is_empty());
    }

    #[test]
    fn test_build_targets_poor_speed() {
        // 1 Mbps should fail most targets
        let result = build_targets(Some(1_000_000.0), false, Theme::Dark);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_build_profile_targets_custom_targets() {
        let targets = vec![
            UsageTarget {
                name: "Custom Target",
                required_mbps: 25.0,
                icon: "🎯",
            },
            UsageTarget {
                name: "Another Target",
                required_mbps: 50.0,
                icon: "⭐",
            },
        ];

        let result = build_profile_targets(
            Some(100_000_000.0),
            false,
            Theme::Dark,
            &targets,
            Some(100.0),
        );

        assert!(result.contains("Custom Target"));
        assert!(result.contains("Another Target"));
    }

    #[test]
    fn test_build_profile_targets_calculates_ratio() {
        // Test that ratio is calculated and displayed
        let targets = vec![UsageTarget {
            name: "Test",
            required_mbps: 50.0,
            icon: "",
        }];

        let result = build_profile_targets(
            Some(200_000_000.0),
            false,
            Theme::Dark,
            &targets,
            Some(200.0),
        );

        // Should show 4.0x ratio for 200/50 = 4 (format is {:.1}x for <10x)
        assert!(result.contains("4.0x"));
    }

    #[test]
    fn test_build_profile_targets_shortfall() {
        // When speed is below requirement, show shortfall
        let targets = vec![UsageTarget {
            name: "Test",
            required_mbps: 100.0,
            icon: "",
        }];

        let result =
            build_profile_targets(Some(30_000_000.0), false, Theme::Dark, &targets, Some(30.0));

        // Should show 70 Mb/s short (100 - 30)
        assert!(result.contains("short"));
    }

    #[test]
    fn test_build_profile_targets_no_download() {
        let targets = vec![UsageTarget {
            name: "Test",
            required_mbps: 50.0,
            icon: "",
        }];

        let result = build_profile_targets(None, false, Theme::Dark, &targets, None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_targets_nc_mode() {
        // No color mode should still produce output
        let result = build_targets(Some(100_000_000.0), true, Theme::Dark);
        assert!(!result.is_empty());
        // Should have the header
        assert!(result.contains("USAGE CHECK"));
    }

    #[test]
    fn test_build_profile_targets_nc_mode() {
        let targets = vec![UsageTarget {
            name: "Test Target",
            required_mbps: 50.0,
            icon: "",
        }];

        let result = build_profile_targets(
            Some(100_000_000.0),
            true,
            Theme::Dark,
            &targets,
            Some(100.0),
        );

        assert!(result.contains("Test Target"));
    }

    #[test]
    fn test_build_none_download() {
        let result = build(None, false, Theme::Dark);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_with_download() {
        let result = build(Some(100_000_000.0), false, Theme::Dark);
        assert!(!result.is_empty());
        assert!(result.contains("ESTIMATES"));
    }

    #[test]
    fn test_build_all_file_types() {
        let result = build(Some(100_000_000.0), false, Theme::Dark);
        // All file estimate names should be present
        assert!(result.contains("Song / Podcast"));
        assert!(result.contains("Photo"));
        assert!(result.contains("App install"));
        assert!(result.contains("HD movie"));
        assert!(result.contains("4K movie"));
        assert!(result.contains("Game install"));
    }

    #[test]
    fn test_build_gigabit_speed() {
        // 1 Gbps should show fast download times
        let result = build(Some(1_000_000_000.0), false, Theme::Dark);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_build_slow_speed() {
        // 1 Mbps should show slow download times
        let result = build(Some(1_000_000.0), false, Theme::Dark);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_build_nc_mode() {
        // No color mode should still produce output
        let result = build(Some(100_000_000.0), true, Theme::Dark);
        assert!(!result.is_empty());
        assert!(result.contains("ESTIMATES"));
    }

    #[test]
    fn test_format_targets_function() {
        // Should not panic and produces output
        format_targets(Some(100_000_000.0), false, Theme::Dark);
    }

    #[test]
    fn test_format_targets_none() {
        // Should not panic with None
        format_targets(None, false, Theme::Dark);
    }

    #[test]
    fn test_show_function() {
        // Should not panic
        show(Some(100_000_000.0), false, Theme::Dark);
    }

    #[test]
    fn test_show_none() {
        // Should not panic with None
        show(None, false, Theme::Dark);
    }

    #[test]
    fn test_build_profile_targets_no_dl_mbps() {
        // When download is set but dl_mbps is None, should calculate from download
        let targets = vec![UsageTarget {
            name: "Test",
            required_mbps: 50.0,
            icon: "",
        }];

        // Pass None for dl_mbps but Some for download_bps - should still work
        let result = build_profile_targets(
            Some(100_000_000.0),
            false,
            Theme::Dark,
            &targets,
            None, // dl_mbps is None
        );

        assert!(!result.is_empty());
    }

    #[test]
    fn test_high_ratio_rounds_correctly() {
        // Test that 10x+ shows integer only
        let targets = vec![UsageTarget {
            name: "Test",
            required_mbps: 10.0,
            icon: "",
        }];

        let result = build_profile_targets(
            Some(500_000_000.0),
            false,
            Theme::Dark,
            &targets,
            Some(500.0),
        );

        // 50x should show "50x" not "50.0x"
        assert!(result.contains("50x"));
    }
}
