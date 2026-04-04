// Unit tests for formatter module

#[cfg(test)]
mod tests {
    use netspeed_cli::formatter::format_simple;
    use netspeed_cli::types::{ServerInfo, TestResult};

    fn create_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 10.5,
            },
            ping: Some(15.234),
            download: Some(150_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        }
    }

    #[test]
    fn test_format_simple_bits() {
        let result = create_test_result();
        // Should not panic
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_bytes() {
        let result = create_test_result();
        // Should not panic
        assert!(format_simple(&result, true).is_ok());
    }

    #[test]
    fn test_format_simple_no_download() {
        let mut result = create_test_result();
        result.download = None;
        // Should not panic
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_upload() {
        let mut result = create_test_result();
        result.upload = None;
        // Should not panic
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_ping() {
        let mut result = create_test_result();
        result.ping = None;
        // Should not panic
        assert!(format_simple(&result, false).is_ok());
    }
}
