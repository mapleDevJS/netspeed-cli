use thiserror::Error;

/// All possible errors in the speedtest CLI application.
///
/// This enum uses `thiserror` for automatic derivation of:
/// - `Display` via `#[error(...)]` attributes
/// - `Error` trait via `#[derive(Error)]`
/// - `From` conversions via `#[from]` attributes
#[derive(Debug, Error)]
pub enum SpeedtestError {
    /// Network-related error (e.g., connection refused, timeout)
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Failed to parse response data (XML, JSON, etc.)
    #[error("Parse error: {0}")]
    ParseError(String),

    /// No suitable server found for testing
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// I/O error (e.g., file write failure)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// CSV serialization/deserialization error
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    /// Generic custom error for edge cases
    #[error("{0}")]
    Custom(String),
}

impl From<reqwest::Error> for SpeedtestError {
    fn from(err: reqwest::Error) -> Self {
        SpeedtestError::NetworkError(err.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_network_error_display() {
        let err = SpeedtestError::NetworkError("connection refused".to_string());
        assert_eq!(format!("{}", err), "Network error: connection refused");
    }

    #[test]
    fn test_parse_error_display() {
        let err = SpeedtestError::ParseError("invalid XML".to_string());
        assert_eq!(format!("{}", err), "Parse error: invalid XML");
    }

    #[test]
    fn test_server_not_found_display() {
        let err = SpeedtestError::ServerNotFound("no servers available".to_string());
        assert_eq!(format!("{}", err), "Server not found: no servers available");
    }

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = SpeedtestError::IoError(io_err);
        assert_eq!(format!("{}", err), "I/O error: access denied");
    }

    #[test]
    fn test_csv_error_display() {
        let csv_err = csv::Error::from(io::Error::new(io::ErrorKind::Other, "csv write error"));
        let err = SpeedtestError::CsvError(csv_err);
        assert!(format!("{}", err).contains("CSV error:"));
    }

    #[test]
    fn test_custom_error_display() {
        let err = SpeedtestError::Custom("custom error message".to_string());
        assert_eq!(format!("{}", err), "custom error message");
    }

    #[test]
    fn test_from_reqwest_error() {
        // Create a reqwest error by attempting to build a client with invalid URL
        let rt = tokio::runtime::Runtime::new().unwrap();
        let reqwest_err = rt.block_on(async {
            reqwest::get("invalid-url-not-a-url").await.unwrap_err()
        });
        let err: SpeedtestError = reqwest_err.into();
        match err {
            SpeedtestError::NetworkError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected NetworkError variant"),
        }
    }

    #[test]
    fn test_from_quick_xml_error() {
        // Create a ParseError from quick_xml error context
        let _xml_result = quick_xml::de::from_str::<serde_json::Value>("<invalid>");
        let xml_err_str = "XML parsing failed";
        let err = SpeedtestError::ParseError(xml_err_str.to_string());
        match err {
            SpeedtestError::ParseError(msg) => {
                assert_eq!(msg, "XML parsing failed");
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: SpeedtestError = json_err.into();
        match err {
            SpeedtestError::ParseError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_from_quick_xml_de_error() {
        let de_err = quick_xml::de::DeError::Custom("deserialization failed".to_string());
        let err: SpeedtestError = de_err.into();
        match err {
            SpeedtestError::ParseError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_error_is_network_error() {
        let err = SpeedtestError::NetworkError("timeout".to_string());
        assert!(matches!(err, SpeedtestError::NetworkError(_)));
    }

    #[test]
    fn test_error_is_parse_error() {
        let err = SpeedtestError::ParseError("bad format".to_string());
        assert!(matches!(err, SpeedtestError::ParseError(_)));
    }

    #[test]
    fn test_error_is_server_not_found() {
        let err = SpeedtestError::ServerNotFound("empty list".to_string());
        assert!(matches!(err, SpeedtestError::ServerNotFound(_)));
    }

    #[test]
    fn test_error_is_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = SpeedtestError::IoError(io_err);
        assert!(matches!(err, SpeedtestError::IoError(_)));
    }

    #[test]
    fn test_error_is_csv_error() {
        let csv_err = csv::Error::from(io::Error::new(io::ErrorKind::Other, "error"));
        let err = SpeedtestError::CsvError(csv_err);
        assert!(matches!(err, SpeedtestError::CsvError(_)));
    }

    #[test]
    fn test_error_is_custom() {
        let err = SpeedtestError::Custom("error".to_string());
        assert!(matches!(err, SpeedtestError::Custom(_)));
    }
}
