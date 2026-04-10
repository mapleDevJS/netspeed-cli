//! Leaf-level utilities — no internal dependencies.
//!
//! Contains validation and unit-conversion functions used across the crate.
//!
//! ## Known Duplication: `is_valid_ipv4`
//!
//! The `is_valid_ipv4` function exists identically in both `common.rs` and
//! `validate.rs`. This is structurally required because:
//! - `validate.rs` is `include!()`-ed by `build.rs` (via `cli.rs`) for
//!   completion/man page generation. Build scripts cannot depend on crate
//!   modules, so `validate.rs` must be fully self-contained.
//! - `common.rs::is_valid_ipv4` is used by `http.rs` at runtime.
//!
//! Both copies must remain in sync. If validation logic changes (e.g., IPv6),
//! update both files.

/// Detect if `NO_COLOR` environment variable is set.
#[must_use]
pub fn no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
