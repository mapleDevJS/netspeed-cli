//! Leaf-level utilities — no internal dependencies.
//!
//! Contains validation and unit-conversion functions used across the crate.
//!
//! ## Validation
//!
//! IP validation functions are shared via `include!("validate.rs")`, which is
//! also `include!()`-ed by `cli.rs` for build-time CLI argument validation.
//! This ensures a single source of truth for `is_valid_ipv4`.

// Shared validation functions (single source of truth, also used by cli.rs/build.rs)
include!("validate.rs");

/// Detect if `NO_COLOR` environment variable is set.
#[must_use]
pub fn no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Detect if `NO_EMOJI` environment variable is set (set by `--no-emoji` flag).
#[must_use]
pub fn no_emoji() -> bool {
    std::env::var("NO_EMOJI").is_ok()
}

/// Format byte count into a human-readable string (KB, MB, GB).
///
/// # Examples
///
/// ```
/// # use netspeed_cli::common::format_data_size;
/// assert!(format_data_size(512).contains(" B"));
/// assert!(format_data_size(500 * 1024).contains("KB"));
/// assert!(format_data_size(1_048_576).contains("MB"));
/// assert!(format_data_size(1_073_741_824).contains("GB"));
/// ```
#[must_use]
pub fn format_data_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
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
    fn test_format_data_size_zero() {
        assert_eq!(format_data_size(0), "0 B");
    }

    #[test]
    fn test_format_data_size_bytes() {
        assert_eq!(format_data_size(512), "512 B");
    }

    #[test]
    fn test_format_data_size_just_under_kb() {
        assert_eq!(format_data_size(1023), "1023 B");
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
