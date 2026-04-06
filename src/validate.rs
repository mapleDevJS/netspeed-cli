/// Shared validation functions for CLI argument parsing.
///
/// This file is `include!()`-ed by both `build.rs` (via `cli.rs`) and the
/// main crate (via `common::is_valid_ipv4`), so it must be self-contained
/// with no external dependencies.
fn validate_ip_address(s: &str) -> Result<String, String> {
    if !is_valid_ipv4(s) {
        return Err(format!(
            "Invalid IP address format: '{s}'. Expected format: x.x.x.x"
        ));
    }
    Ok(s.to_string())
}

fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn test_validate_ip_valid() {
        assert!(validate_ip_address("192.168.1.1").is_ok());
        assert!(validate_ip_address("0.0.0.0").is_ok());
        assert!(validate_ip_address("255.255.255.255").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid_format() {
        assert!(validate_ip_address("192.168.1").is_err());
        assert!(validate_ip_address("192.168.1.1.1").is_err());
        assert!(validate_ip_address("").is_err());
    }

    #[test]
    fn test_validate_ip_invalid_octet() {
        assert!(validate_ip_address("192.168.1.999").is_err());
        assert!(validate_ip_address("256.0.0.1").is_err());
    }

    #[test]
    fn test_is_valid_ipv4_comprehensive() {
        assert!(is_valid_ipv4("10.0.0.1"));
        assert!(is_valid_ipv4("172.16.0.1"));
        assert!(!is_valid_ipv4("abc"));
        assert!(!is_valid_ipv4("1.2.3"));
        assert!(!is_valid_ipv4("1.2.3.4.5"));
        assert!(!is_valid_ipv4("1.2.3.256"));
    }
}
