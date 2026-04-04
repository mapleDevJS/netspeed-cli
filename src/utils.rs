/// Calculate bits per second from bytes transferred and elapsed time
pub fn calculate_bps(total_bytes: u64, elapsed_secs: f64) -> f64 {
    if elapsed_secs > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed_secs
    } else {
        0.0
    }
}

/// Format speed in human-readable form
pub fn format_speed(bps: f64, bytes: bool) -> String {
    let (value, unit) = if bytes {
        (bps / 8.0 / 1_000_000.0, "MByte/s")
    } else {
        (bps / 1_000_000.0, "Mbit/s")
    };

    format!("{:.2} {}", value, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bps_basic() {
        let result = calculate_bps(1_000_000, 1.0);
        assert!((result - 8_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_bps_zero_time() {
        let result = calculate_bps(1_000_000, 0.0);
        assert!((result - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_bps_zero_bytes() {
        let result = calculate_bps(0, 1.0);
        assert!((result - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_bps_realistic_values() {
        // 50 MB in 10 seconds = 40 Mbit/s
        let result = calculate_bps(50_000_000, 10.0);
        assert!((result - 40_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_bps_very_small_time() {
        let result = calculate_bps(1_000_000, 0.001);
        assert!((result - 8_000_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_speed_mbps() {
        let result = format_speed(100_000_000.0, false);
        assert_eq!(result, "100.00 Mbit/s");
    }

    #[test]
    fn test_format_speed_mbytes() {
        let result = format_speed(100_000_000.0, true);
        assert_eq!(result, "12.50 MByte/s");
    }

    #[test]
    fn test_format_speed_zero_bps() {
        let result = format_speed(0.0, false);
        assert_eq!(result, "0.00 Mbit/s");
    }

    #[test]
    fn test_format_speed_zero_bps_bytes() {
        let result = format_speed(0.0, true);
        assert_eq!(result, "0.00 MByte/s");
    }

    #[test]
    fn test_format_speed_very_large_value() {
        let result = format_speed(10_000_000_000.0, false);
        assert_eq!(result, "10000.00 Mbit/s");
    }

    #[test]
    fn test_format_speed_small_value() {
        let result = format_speed(1_500_000.0, false);
        assert_eq!(result, "1.50 Mbit/s");
    }

    #[test]
    fn test_format_speed_fractional() {
        let result = format_speed(1_234_567.0, false);
        assert_eq!(result, "1.23 Mbit/s");
    }
}
