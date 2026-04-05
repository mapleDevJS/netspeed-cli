use std::fmt;

/// Unified error type for netspeed-cli operations.
///
/// This enum preserves the original error cause chains by storing
/// the underlying errors directly, enabling better debugging and
/// error reporting via the `std::error::Error::source()` method.
#[derive(Debug)]
pub enum SpeedtestError {
    /// Network-related errors from HTTP requests
    NetworkError(reqwest::Error),
    /// XML parsing errors
    ParseXml(quick_xml::Error),
    /// JSON parsing/serialization errors
    ParseJson(serde_json::Error),
    /// XML deserialization errors
    DeserializeXml(quick_xml::de::DeError),
    /// CSV parsing/serialization errors
    Csv(csv::Error),
    /// Server selection errors
    ServerNotFound(String),
    /// I/O errors from file operations
    IoError(std::io::Error),
    /// Application-specific errors with context
    Context {
        msg: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl fmt::Display for SpeedtestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeedtestError::NetworkError(err) => {
                write!(f, "Network error: {err}")
            }
            SpeedtestError::ParseXml(err) => {
                write!(f, "XML parse error: {err}")
            }
            SpeedtestError::ParseJson(err) => {
                write!(f, "JSON parse error: {err}")
            }
            SpeedtestError::DeserializeXml(err) => {
                write!(f, "XML deserialization error: {err}")
            }
            SpeedtestError::Csv(err) => {
                write!(f, "CSV error: {err}")
            }
            SpeedtestError::ServerNotFound(msg) => {
                write!(f, "Server not found: {msg}")
            }
            SpeedtestError::IoError(err) => {
                write!(f, "I/O error: {err}")
            }
            SpeedtestError::Context { msg, .. } => {
                write!(f, "{msg}")
            }
        }
    }
}

impl std::error::Error for SpeedtestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SpeedtestError::NetworkError(err) => Some(err),
            SpeedtestError::ParseXml(err) => Some(err),
            SpeedtestError::ParseJson(err) => Some(err),
            SpeedtestError::DeserializeXml(err) => Some(err),
            SpeedtestError::Csv(err) => Some(err),
            SpeedtestError::ServerNotFound(_) => None,
            SpeedtestError::IoError(err) => Some(err),
            SpeedtestError::Context { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
        }
    }
}

impl SpeedtestError {
    /// Create a contextual error with an optional source error.
    #[must_use]
    pub fn context(msg: impl Into<String>) -> Self {
        Self::Context {
            msg: msg.into(),
            source: None,
        }
    }

    /// Create a contextual error with a source error chain.
    #[must_use]
    pub fn with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Context {
            msg: msg.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<reqwest::Error> for SpeedtestError {
    fn from(err: reqwest::Error) -> Self {
        SpeedtestError::NetworkError(err)
    }
}

impl From<std::io::Error> for SpeedtestError {
    fn from(err: std::io::Error) -> Self {
        SpeedtestError::IoError(err)
    }
}

impl From<quick_xml::Error> for SpeedtestError {
    fn from(err: quick_xml::Error) -> Self {
        SpeedtestError::ParseXml(err)
    }
}

impl From<serde_json::Error> for SpeedtestError {
    fn from(err: serde_json::Error) -> Self {
        SpeedtestError::ParseJson(err)
    }
}

impl From<quick_xml::de::DeError> for SpeedtestError {
    fn from(err: quick_xml::de::DeError) -> Self {
        SpeedtestError::DeserializeXml(err)
    }
}

impl From<csv::Error> for SpeedtestError {
    fn from(err: csv::Error) -> Self {
        SpeedtestError::Csv(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_network_error_display() {
        // Test display via context method since we can't easily create reqwest::Error
        let err = SpeedtestError::context("connection failed");
        assert_eq!(format!("{err}"), "connection failed");
    }

    #[test]
    fn test_json_error_display() {
        let invalid_json = "{invalid}";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err = SpeedtestError::from(result.unwrap_err());
        assert!(format!("{err}").contains("JSON parse error"));
    }

    #[test]
    fn test_server_not_found_display() {
        let err = SpeedtestError::ServerNotFound("no servers".to_string());
        assert_eq!(format!("{err}"), "Server not found: no servers");
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let speedtest_err = SpeedtestError::from(io_err);
        assert!(format!("{speedtest_err}").contains("I/O error"));
    }

    #[test]
    fn test_context_error_display() {
        let err = SpeedtestError::context("custom error");
        assert_eq!(format!("{err}"), "custom error");
    }

    #[test]
    fn test_context_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = SpeedtestError::with_source("Failed to read config", io_err);
        assert_eq!(format!("{err}"), "Failed to read config");
        assert!(err.source().is_some());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let speedtest_err: SpeedtestError = io_err.into();
        assert!(matches!(speedtest_err, SpeedtestError::IoError(_)));
        assert!(format!("{speedtest_err}").contains("I/O error"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let err = SpeedtestError::context("test error");
        // Test that Error trait is implemented
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_debug_trait() {
        let err = SpeedtestError::context("debug test");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Context"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid_json = "{invalid}";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err: SpeedtestError = result.unwrap_err().into();
        assert!(matches!(err, SpeedtestError::ParseJson(_)));
    }

    #[test]
    fn test_from_quick_xml_de_error() {
        let invalid_xml = "<unclosed>";
        let result: Result<serde_json::Value, _> = quick_xml::de::from_str(invalid_xml);
        assert!(result.is_err());
        let err: SpeedtestError = result.unwrap_err().into();
        assert!(matches!(err, SpeedtestError::DeserializeXml(_)));
    }

    #[test]
    fn test_from_csv_error_direct() {
        let data = b"a,b\n1,2,3";
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(false)
            .from_reader(&data[..]);
        for result in reader.records() {
            if let Err(e) = result {
                let err: SpeedtestError = e.into();
                assert!(matches!(err, SpeedtestError::Csv(_)));
                return;
            }
        }
        panic!("Expected CSV parse error");
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = SpeedtestError::with_source("Failed to load history", io_err);

        // Verify source chain is preserved
        assert!(matches!(err, SpeedtestError::Context { .. }));
        let source = err.source();
        assert!(source.is_some());

        // Verify it's an io::Error
        let source = source.unwrap();
        assert!(source.is::<std::io::Error>());
    }

    #[test]
    fn test_context_without_source() {
        let err = SpeedtestError::context("standalone error");
        assert!(matches!(err, SpeedtestError::Context { source: None, .. }));
        assert!(err.source().is_none());
    }
}
