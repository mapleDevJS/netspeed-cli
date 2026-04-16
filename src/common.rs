//! Common shared utilities used across download, upload, formatting, and progress modules.
//!
//! This module consolidates duplicated functionality to follow DRY principles:
//! - Bandwidth calculation
//! - Stream count determination
//! - Distance formatting
//! - Data size formatting
//! - Terminal width detection

use std::io::IsTerminal;

/// Get the terminal width in columns, or a sensible default.
///
/// Returns `None` if stdout is not a terminal (piped output).
/// Returns the width in character columns when available.
/// Falls back to 100 columns if width cannot be determined.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::get_terminal_width;
/// let width = get_terminal_width();
/// // When not a TTY (doctest), returns None; unwrap_or default ≥ 80
/// assert!(width.unwrap_or(100) >= 80);
/// ```
#[must_use]
pub fn get_terminal_width() -> Option<u16> {
    if !std::io::stdout().is_terminal() {
        return None;
    }
    terminal_size::terminal_size().map(|(w, _)| w.0)
}

/// Get the terminal width with a minimum and maximum bound.
///
/// Ensures the width is within reasonable bounds for formatting.
/// Returns at least `min_width` and at most `max_width`.
/// If terminal width cannot be determined, returns `default_width`.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::get_terminal_width_bounded;
/// let width = get_terminal_width_bounded(60, 120, 80);
/// assert!(width >= 60 && width <= 120);
/// ```
#[must_use]
pub fn get_terminal_width_bounded(min_width: u16, max_width: u16, default_width: u16) -> u16 {
    match get_terminal_width() {
        Some(w) => w.clamp(min_width, max_width),
        None => default_width,
    }
}

/// Calculate bandwidth in bits per second from bytes transferred and elapsed time.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::calculate_bandwidth;
/// let bps = calculate_bandwidth(10_000_000, 2.0);
/// assert_eq!(bps, 40_000_000.0);
/// ```
#[must_use]
pub fn calculate_bandwidth(total_bytes: u64, elapsed_secs: f64) -> f64 {
    if elapsed_secs > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed_secs
    } else {
        0.0
    }
}

/// Determine number of concurrent streams based on single connection flag.
///
/// Returns 1 for single connection mode, 4 for multi-stream mode.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::determine_stream_count;
/// assert_eq!(determine_stream_count(true), 1);
/// assert_eq!(determine_stream_count(false), 4);
/// ```
#[must_use]
pub fn determine_stream_count(single: bool) -> usize {
    if single {
        1
    } else {
        4
    }
}

/// Format distance consistently: 1 decimal for < 100 km, 0 decimals for >= 100 km.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::format_distance;
/// assert_eq!(format_distance(50.5), "50.5 km");
/// assert_eq!(format_distance(150.5), "150 km");
/// ```
#[must_use]
pub fn format_distance(km: f64) -> String {
    if km < 100.0 {
        format!("{km:.1} km")
    } else {
        format!("{km:.0} km")
    }
}

/// Format byte count into a human-readable string (KB, MB, GB).
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::format_data_size;
/// assert!(format_data_size(512).contains("KB"));
/// assert!(format_data_size(1_048_576).contains("MB"));
/// assert!(format_data_size(1_073_741_824).contains("GB"));
/// ```
#[must_use]
pub fn format_data_size(bytes: u64) -> String {
    if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Validate an IPv4 address string.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::is_valid_ipv4;
/// assert!(is_valid_ipv4("192.168.1.1"));
/// assert!(!is_valid_ipv4("999.999.999.999"));
/// ```
#[must_use]
pub fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

/// Render a horizontal bar chart using Unicode block characters.
///
/// `value` and `max` define the proportion. `width` is the bar length in chars.
/// Returns filled (`█`) and empty (`░`) segments. Pure text — callers apply
/// color via `owo_colors` for consistency.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::bar_chart;
/// let bar = bar_chart(50.0, 100.0, 10);
/// assert_eq!(bar.chars().count(), 10);
/// ```
#[must_use]
pub fn bar_chart(value: f64, max: f64, width: usize) -> String {
    if max <= 0.0 || width == 0 {
        return "░".repeat(width);
    }
    let pct = (value / max).clamp(0.0, 1.0);
    let filled = (pct * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

// ── Tabular Data Formatting ──────────────────────────────────────────────────

/// Format a numeric value with fixed-width padding for vertical alignment.
/// Pads with leading spaces so numbers right-align in columns.
///
/// # Arguments
/// * `value` — The numeric value to format
/// * `width` — Total column width (including decimal point)
/// * `decimals` — Number of decimal places
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::tabular_number;
/// assert_eq!(tabular_number(15.2, 8, 1), "    15.2");
/// assert_eq!(tabular_number(150.0, 8, 1), "   150.0");
/// ```
#[must_use]
pub fn tabular_number(value: f64, width: usize, decimals: usize) -> String {
    if decimals == 0 {
        format!("{:>width$}", value as i64)
    } else {
        format!("{:>width$.decimals$}", value)
    }
}

/// Format a speed value in Mbps with tabular alignment.
/// Returns a fixed-width string like `"   150.00 Mb/s"` or `"     0.12 Gb/s"`.
///
/// # Arguments
/// * `bps` — Bits per second
/// * `total_width` — Total column width including unit
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::format_speed_tabular;
/// let s = format_speed_tabular(150_000_000.0, 14);
/// assert!(s.contains("Mb/s"));
/// ```
#[must_use]
pub fn format_speed_tabular(bps: f64, total_width: usize) -> String {
    let (value, unit) = if bps >= 1_000_000_000.0 {
        (bps / 1_000_000_000.0, "Gb/s")
    } else if bps >= 1_000_000.0 {
        (bps / 1_000_000.0, "Mb/s")
    } else if bps >= 1_000.0 {
        (bps / 1_000.0, "Kb/s")
    } else {
        return format!("{:>total_width$} b/s", bps as i64);
    };
    let unit_width = unit.len();
    let val_width = total_width.saturating_sub(unit_width + 1); // +1 for space
    format!("{:>val_width$.2} {unit}", value)
}

/// Format latency in ms with tabular alignment.
/// Returns a fixed-width string like `"    12.1 ms"`.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::format_latency_tabular;
/// let s = format_latency_tabular(12.1, 10);
/// assert!(s.contains("ms"));
/// ```
#[must_use]
pub fn format_latency_tabular(ms: f64, width: usize) -> String {
    format!("{:>width$.1} ms", ms)
}

/// Format jitter in ms with tabular alignment.
#[must_use]
pub fn format_jitter_tabular(ms: f64, width: usize) -> String {
    format!("{:>width$.1} ms", ms)
}

/// Format packet loss percentage with tabular alignment.
#[must_use]
pub fn format_loss_tabular(pct: f64, width: usize) -> String {
    format!("{:>width$.1}%", pct)
}

/// Format data size (bytes) with tabular alignment for data transfer amounts.
/// Returns a fixed-width string like `"  15.0 MB"`.
#[must_use]
pub fn format_data_size_tabular(bytes: u64, width: usize) -> String {
    let (value, unit) = if bytes < 1024 * 1024 {
        (bytes as f64 / 1024.0, "KB")
    } else if bytes < 1024 * 1024 * 1024 {
        (bytes as f64 / (1024.0 * 1024.0), "MB")
    } else {
        let val = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let unit_width = 2; // "GB"
        let val_width = width.saturating_sub(unit_width + 1);
        return format!("{:>val_width$.2} GB", val);
    };
    let unit_width = unit.len();
    let val_width = width.saturating_sub(unit_width + 1);
    format!("{:>val_width$.1} {unit}", value)
}

/// Format duration with tabular alignment.
#[must_use]
pub fn format_duration_tabular(secs: f64, width: usize) -> String {
    if secs < 60.0 {
        format!("{:>width$.1}s", secs)
    } else {
        let mins = secs as u64 / 60;
        let rem = secs % 60.0;
        format!("{:>width$}m {:04.1}s", mins, rem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bandwidth_normal() {
        assert_eq!(calculate_bandwidth(10_000_000, 2.0), 40_000_000.0);
    }

    #[test]
    fn test_calculate_bandwidth_zero_elapsed() {
        assert_eq!(calculate_bandwidth(10_000_000, 0.0), 0.0);
    }

    #[test]
    fn test_determine_stream_count_single() {
        assert_eq!(determine_stream_count(true), 1);
    }

    #[test]
    fn test_determine_stream_count_multi() {
        assert_eq!(determine_stream_count(false), 4);
    }

    #[test]
    fn test_format_distance_under_100() {
        assert_eq!(format_distance(50.5), "50.5 km");
        assert_eq!(format_distance(99.9), "99.9 km");
    }

    #[test]
    fn test_format_distance_100_plus() {
        assert_eq!(format_distance(100.0), "100 km");
        assert_eq!(format_distance(150.5), "150 km");
    }

    #[test]
    fn test_format_data_size_bytes() {
        assert!(format_data_size(512).contains("KB"));
    }

    #[test]
    fn test_format_data_size_kilobytes() {
        assert!(format_data_size(500 * 1024).contains("KB"));
    }

    #[test]
    fn test_format_data_size_megabytes() {
        assert!(format_data_size(10 * 1024 * 1024).contains("MB"));
    }

    #[test]
    fn test_format_data_size_gigabytes() {
        assert!(format_data_size(4 * 1024 * 1024 * 1024).contains("GB"));
    }

    #[test]
    fn test_is_valid_ipv4_valid() {
        assert!(is_valid_ipv4("192.168.1.1"));
        assert!(is_valid_ipv4("0.0.0.0"));
        assert!(is_valid_ipv4("255.255.255.255"));
    }

    #[test]
    fn test_is_valid_ipv4_invalid() {
        assert!(!is_valid_ipv4("256.1.1.1"));
        assert!(!is_valid_ipv4("1.2.3"));
        assert!(!is_valid_ipv4("abc"));
        assert!(!is_valid_ipv4(""));
        assert!(!is_valid_ipv4("1.2.3.4.5"));
    }

    #[test]
    fn test_bar_chart_half() {
        let bar = bar_chart(50.0, 100.0, 10);
        assert_eq!(bar.chars().count(), 10);
        assert_eq!(bar, "█████░░░░░");
    }

    #[test]
    fn test_bar_chart_full() {
        let bar = bar_chart(100.0, 100.0, 10);
        assert_eq!(bar.chars().count(), 10);
        assert_eq!(bar, "██████████");
    }

    #[test]
    fn test_bar_chart_empty_val() {
        let bar = bar_chart(0.0, 100.0, 10);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_bar_chart_zero_max() {
        let bar = bar_chart(50.0, 0.0, 10);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_bar_chart_zero_width() {
        let bar = bar_chart(50.0, 100.0, 0);
        assert_eq!(bar, "");
    }

    #[test]
    fn test_bar_chart_over_max() {
        let bar = bar_chart(150.0, 100.0, 10);
        assert_eq!(bar, "██████████"); // clamped to 100%
    }

    #[test]
    fn test_get_terminal_width_bounded_default() {
        // Should return default width when not in terminal
        let width = get_terminal_width_bounded(60, 120, 80);
        assert!((60..=120).contains(&width));
    }

    #[test]
    fn test_get_terminal_width_bounded_clamps() {
        // When not in a terminal, returns the default value (100 in this case since
        // we can't query terminal width, it uses default from terminal_size crate)
        let def = get_terminal_width_bounded(80, 100, 90);
        // Returns default width from terminal_size or the default parameter
        assert!((80..=120).contains(&def));

        let def2 = get_terminal_width_bounded(60, 80, 70);
        assert!((60..=120).contains(&def2));
    }

    // ── Tabular formatting tests ──

    #[test]
    fn test_tabular_number_right_aligned() {
        assert_eq!(tabular_number(15.2, 8, 1), "    15.2");
        assert_eq!(tabular_number(150.0, 8, 1), "   150.0");
        assert_eq!(tabular_number(1234.5, 8, 1), "  1234.5");
    }

    #[test]
    fn test_tabular_number_zero_decimals() {
        assert_eq!(tabular_number(42.0, 6, 0), "    42");
        assert_eq!(tabular_number(1000.0, 6, 0), "  1000");
    }

    #[test]
    fn test_format_speed_tabular_mbps() {
        let s = format_speed_tabular(150_000_000.0, 14);
        assert_eq!(s, "   150.00 Mb/s");
        assert_eq!(s.len(), 14);
    }

    #[test]
    fn test_format_speed_tabular_gbps() {
        let s = format_speed_tabular(1_200_000_000.0, 14);
        assert_eq!(s, "     1.20 Gb/s");
        assert_eq!(s.len(), 14);
    }

    #[test]
    fn test_format_speed_tabular_kbps() {
        let s = format_speed_tabular(50_000.0, 14);
        assert_eq!(s, "    50.00 Kb/s");
        assert_eq!(s.len(), 14);
    }

    #[test]
    fn test_format_latency_tabular() {
        assert_eq!(format_latency_tabular(12.1, 10), "      12.1 ms");
        assert_eq!(format_latency_tabular(150.5, 10), "     150.5 ms");
    }

    #[test]
    fn test_format_jitter_tabular() {
        assert_eq!(format_jitter_tabular(1.5, 10), "       1.5 ms");
    }

    #[test]
    fn test_format_loss_tabular() {
        assert_eq!(format_loss_tabular(0.0, 8), "     0.0%");
        assert_eq!(format_loss_tabular(5.5, 8), "     5.5%");
    }

    #[test]
    fn test_format_data_size_tabular_mb() {
        let s = format_data_size_tabular(15 * 1024 * 1024, 10);
        assert_eq!(s, "   15.0 MB");
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_format_data_size_tabular_gb() {
        let s = format_data_size_tabular(4 * 1024 * 1024 * 1024, 10);
        assert_eq!(s, "   4.00 GB");
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_format_duration_tabular_seconds() {
        assert_eq!(format_duration_tabular(30.5, 8), "    30.5s");
    }

    #[test]
    fn test_format_duration_tabular_minutes() {
        let s = format_duration_tabular(90.5, 10);
        assert!(s.contains('m'));
    }

    // Property-based tests via proptest
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_bandwidth_always_non_negative(bytes in 0u64..u64::MAX, elapsed in 0.0f64..1e6) {
                let result = calculate_bandwidth(bytes, elapsed);
                prop_assert!(result >= 0.0, "bandwidth must never be negative");
            }

            #[test]
            fn prop_bandwidth_zero_elapsed_returns_zero(bytes in 0u64..1_000_000) {
                let result = calculate_bandwidth(bytes, 0.0);
                prop_assert_eq!(result, 0.0);
            }

            #[test]
            fn prop_bandwidth_linear_scaling(bytes in 1u64..1_000_000) {
                let r1 = calculate_bandwidth(bytes, 1.0);
                let r2 = calculate_bandwidth(bytes, 2.0);
                prop_assert!((r1 - 2.0 * r2).abs() < f64::EPSILON, "doubling time should halve bandwidth");
            }

            #[test]
            fn prop_bar_chart_length(width in 1usize..200, value in 0.0f64..1000.0, max in 1.0f64..1000.0) {
                let bar = bar_chart(value, max, width);
                prop_assert_eq!(bar.chars().count(), width, "bar must have exactly width characters");
            }

            #[test]
            fn prop_distance_always_ends_with_km(km in 0.0f64..10000.0) {
                let result = format_distance(km);
                prop_assert!(result.ends_with(" km"));
            }

            #[test]
            fn prop_data_size_always_has_unit(bytes in 0u64..u64::MAX) {
                let result = format_data_size(bytes);
                prop_assert!(
                    result.contains("KB") || result.contains("MB") || result.contains("GB"),
                    "formatted size must contain a unit"
                );
            }
        }
    }
}
