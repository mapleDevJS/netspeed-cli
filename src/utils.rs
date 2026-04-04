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
