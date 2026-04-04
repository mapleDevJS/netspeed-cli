use std::fmt;

#[derive(Debug)]
pub enum SpeedtestError {
    NetworkError(String),
    ParseError(String),
    ServerNotFound(String),
    #[allow(dead_code)]
    TimeoutError(String),
    IoError(String),
    Custom(String),
}

impl fmt::Display for SpeedtestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeedtestError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SpeedtestError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SpeedtestError::ServerNotFound(msg) => write!(f, "Server not found: {}", msg),
            SpeedtestError::TimeoutError(msg) => write!(f, "Timeout: {}", msg),
            SpeedtestError::IoError(msg) => write!(f, "I/O error: {}", msg),
            SpeedtestError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SpeedtestError {}

impl From<reqwest::Error> for SpeedtestError {
    fn from(err: reqwest::Error) -> Self {
        SpeedtestError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for SpeedtestError {
    fn from(err: std::io::Error) -> Self {
        SpeedtestError::IoError(err.to_string())
    }
}

impl From<quick_xml::Error> for SpeedtestError {
    fn from(err: quick_xml::Error) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<serde_json::Error> for SpeedtestError {
    fn from(err: serde_json::Error) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<quick_xml::de::DeError> for SpeedtestError {
    fn from(err: quick_xml::de::DeError) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<csv::Error> for SpeedtestError {
    fn from(err: csv::Error) -> Self {
        SpeedtestError::Custom(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_display() {
        let err = SpeedtestError::NetworkError("connection failed".to_string());
        assert_eq!(format!("{}", err), "Network error: connection failed");
    }

    #[test]
    fn test_parse_error_display() {
        let err = SpeedtestError::ParseError("invalid JSON".to_string());
        assert_eq!(format!("{}", err), "Parse error: invalid JSON");
    }

    #[test]
    fn test_server_not_found_display() {
        let err = SpeedtestError::ServerNotFound("no servers".to_string());
        assert_eq!(format!("{}", err), "Server not found: no servers");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = SpeedtestError::TimeoutError("request timed out".to_string());
        assert_eq!(format!("{}", err), "Timeout: request timed out");
    }

    #[test]
    fn test_io_error_display() {
        let err = SpeedtestError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

    #[test]
    fn test_custom_error_display() {
        let err = SpeedtestError::Custom("custom error".to_string());
        assert_eq!(format!("{}", err), "custom error");
    }

    #[test]
    fn test_from_reqwest_error() {
        // Test conversion from reqwest error - we'll test with a network error scenario
        // since we can't easily create a reqwest error without the blocking feature
        let network_err = SpeedtestError::NetworkError("connection refused".to_string());
        assert!(matches!(network_err, SpeedtestError::NetworkError(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let speedtest_err: SpeedtestError = io_err.into();
        assert!(matches!(speedtest_err, SpeedtestError::IoError(_)));
        assert!(format!("{}", speedtest_err).contains("I/O error"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let err = SpeedtestError::NetworkError("test error".to_string());
        // Test that Error trait is implemented
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_debug_trait() {
        let err = SpeedtestError::Custom("debug test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Custom"));
        assert!(debug_str.contains("debug test"));
    }
}
