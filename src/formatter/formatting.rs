//! Formatting primitives for terminal output.
//!
//! Pure text functions — callers apply color via `owo_colors`.

/// Format distance consistently: 1 decimal for < 100 km, 0 decimals for >= 100 km.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::formatter::formatting::format_distance;
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
/// # use netspeed_cli::formatter::formatting::format_data_size;
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

/// Render a horizontal bar chart using Unicode block characters.
///
/// `value` and `max` define the proportion. `width` is the bar length in chars.
/// Returns filled (`█`) and empty (`░`) segments.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::formatter::formatting::bar_chart;
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

/// Render a sparkline from a slice of numeric values using Unicode block chars.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::formatter::formatting::sparkline;
/// let line = sparkline(&[10.0, 20.0, 30.0]);
/// assert_eq!(line.chars().count(), 3); // one char per value
/// ```
#[must_use]
pub fn sparkline(values: &[f64]) -> String {
    const CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return String::new();
    }
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let range = max - min;
    if range <= 0.0 {
        return CHARS[3].to_string().repeat(values.len());
    }
    values
        .iter()
        .map(|v| {
            let idx = (((v - min) / range) * 7.0).round() as usize;
            CHARS[idx.min(7)]
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_sparkline_increasing() {
        let line = sparkline(&[10.0, 20.0, 30.0]);
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn test_sparkline_empty() {
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn test_sparkline_identical_values() {
        let line = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(line.chars().count(), 3);
        assert!(line.chars().all(|c| c == '▄'));
    }
}
