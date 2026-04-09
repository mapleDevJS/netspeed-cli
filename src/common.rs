//! Input validation utilities.
//!
//! Contains shared validation functions used across the crate.
//! Note: `is_valid_ipv4` is duplicated in `validate.rs` (included by `cli.rs`
//! and `build.rs`) because the build script cannot depend on crate modules.

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
