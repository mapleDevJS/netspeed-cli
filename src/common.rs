//! Common shared utilities used across download, upload, formatting, and progress modules.
//!
//! This module consolidates duplicated functionality to follow DRY principles:
//! - Bandwidth calculation
//! - Stream count determination
//! - Distance formatting
//! - Data size formatting

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
    if single { 1 } else { 4 }
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
}
