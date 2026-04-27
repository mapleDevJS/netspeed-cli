/// Shared validation functions for CLI argument parsing.
///
/// This file is `include!()`-ed by both `build.rs` (via `cli.rs`) and the
/// main crate (via `common::is_valid_ipv4`), so it must be self-contained
/// with no external dependencies.
fn validate_ip_address(s: &str) -> Result<String, String> {
    if is_valid_ipv4(s) || is_valid_ipv6(s) {
        return Ok(s.to_string());
    }
    Err(format!(
        "Invalid IP address format: '{s}'. Expected format: x.x.x.x or IPv6"
    ))
}

fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

/// Validate an IPv6 address (simple heuristic: contains `:`).
fn is_valid_ipv6(s: &str) -> bool {
    if !s.contains(':') {
        return false;
    }
    // Accept standard, compressed, and mapped formats
    s.parse::<std::net::Ipv6Addr>().is_ok()
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn test_validate_ip_v4() {
        assert!(validate_ip_address("192.168.1.1").is_ok());
        assert!(validate_ip_address("0.0.0.0").is_ok());
        assert!(validate_ip_address("255.255.255.255").is_ok());
    }

    #[test]
    fn test_validate_ip_v6() {
        assert!(validate_ip_address("::1").is_ok());
        assert!(validate_ip_address("fe80::1").is_ok());
        assert!(validate_ip_address("2001:db8::1").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid() {
        assert!(validate_ip_address("192.168.1").is_err());
        assert!(validate_ip_address("192.168.1.1.1").is_err());
        assert!(validate_ip_address("").is_err());
        assert!(validate_ip_address("not-an-ip").is_err());
    }

    #[test]
    fn test_validate_ip_invalid_octet() {
        assert!(validate_ip_address("192.168.1.999").is_err());
        assert!(validate_ip_address("256.0.0.1").is_err());
    }

    #[test]
    fn test_is_valid_ipv6() {
        assert!(is_valid_ipv6("::1"));
        assert!(is_valid_ipv6("fe80::1"));
        assert!(is_valid_ipv6("2001:db8::1"));
        assert!(!is_valid_ipv6("192.168.1.1"));
        assert!(!is_valid_ipv6("abc"));
    }
}
