//! Usage check targets and real-world download time estimates.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::common;
use crate::terminal;
use crate::theme::{Theme, ThemeColors};

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
        lines.push("\n  USAGE CHECK".to_string());
    } else {
        lines.push(format!("\n  {}", ThemeColors::header("USAGE CHECK", theme)));
    }

    for target in targets {
        let met = dl_mbps >= target.required_mbps;
        let ratio = dl_mbps / target.required_mbps;
        let suffix = if ratio >= 10.0 {
            format!("{:.0}x", ratio)
        } else {
            format!("{:.1}x", ratio)
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
                lines.push(format!("  {}", ThemeColors::good(&line, theme)));
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
                lines.push(format!("  {}", ThemeColors::bad(&line, theme)));
            }
        }
    }

    lines.join("\n")
}

/// Format target usage check against download speed.
pub fn format_targets(download_bps: Option<f64>, nc: bool, theme: Theme) {
    let output = build_targets(download_bps, nc, theme);
    if !output.is_empty() {
        eprintln!("{}", output);
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
        format!("{:.1}s", secs)
    } else if secs < 60.0 {
        format!("{secs:.0}s")
    } else if secs < 3600.0 {
        format!("{}m {:02}s", secs as u64 / 60, (secs % 60.0) as u64)
    } else {
        format!(
            "{}h {:02}m",
            secs as u64 / 3600,
            ((secs % 3600.0) / 60.0) as u64
        )
    }
}

/// Build real-world download time estimates as a string.
pub fn build_estimates(download_bps: Option<f64>, nc: bool, theme: Theme) -> String {
    let Some(dl) = download_bps else {
        return String::new();
    };
    let dl_bytes_per_sec = dl / 8.0;

    let mut lines = Vec::new();

    if nc {
        lines.push("\n  ESTIMATES".to_string());
    } else {
        lines.push(format!("\n  {}", ThemeColors::header("ESTIMATES", theme)));
    }

    for file in ESTIMATES {
        let secs = file.size_bytes as f64 / dl_bytes_per_sec;
        let time_str = format_time_estimate(secs, nc);
        let size_str = common::format_data_size(file.size_bytes);
        let label = format!("{:<24} {:>8}   ~{time_str}", file.name, size_str);
        if nc {
            lines.push(format!("  {label}"));
        } else {
            lines.push(format!("  {}", ThemeColors::good(&label, theme)));
        }
    }

    lines.join("\n")
}

pub fn format_estimates(download_bps: Option<f64>, nc: bool, theme: Theme) {
    let output = build_estimates(download_bps, nc, theme);
    if !output.is_empty() {
        eprintln!("{}", output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_estimate() {
        assert!(format_time_estimate(0.5, false).contains("0.5s"));
        assert!(format_time_estimate(30.0, false).contains("30s"));
        assert!(format_time_estimate(120.0, false).contains("2m"));
    }
}
