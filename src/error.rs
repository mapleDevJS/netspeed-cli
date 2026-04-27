use thiserror::Error;

/// Error category for machine-readable error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Network-related errors (connectivity, timeouts, etc.)
    Network,
    /// Configuration errors (invalid settings, missing files, etc.)
    Config,
    /// Parse errors (invalid JSON, XML, CSV, etc.)
    Parse,
    /// Output errors (file writing, formatting, etc.)
    Output,
    /// Internal errors (bugs, unexpected states, etc.)
    Internal,
}

/// Unified error type for netspeed-cli operations.
///
/// This enum preserves the original error cause chains by storing
/// the underlying errors directly, enabling better debugging and
/// error reporting via the `std::error::Error::source()` method.
#[derive(Debug, Error)]
pub enum Error {
    /// Network-related errors from HTTP requests
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Failed to fetch the server list from speedtest.net
    #[error("Failed to fetch server list: {0}")]
    ServerListFetch(#[source] reqwest::Error),

    /// Failed during download bandwidth test
    #[error("Download test failed: {0}")]
    DownloadTest(#[source] reqwest::Error),

    /// Failed during upload bandwidth test
    #[error("Upload test failed: {0}")]
    UploadTest(#[source] reqwest::Error),

    /// Download test failed for a non-HTTP reason
    #[error("Download test failed: {0}")]
    DownloadFailure(String),

    /// Upload test failed for a non-HTTP reason
    #[error("Upload test failed: {0}")]
    UploadFailure(String),

    /// Failed to discover client IP address
    #[error("Failed to discover client IP: {0}")]
    IpDiscovery(#[source] reqwest::Error),

    /// XML parsing errors
    #[error("XML parse error: {0}")]
    ParseXml(#[from] quick_xml::Error),

    /// JSON parsing/serialization errors
    #[error("JSON parse error: {0}")]
    ParseJson(#[from] serde_json::Error),

    /// XML deserialization errors
    #[error("XML deserialization error: {0}")]
    DeserializeXml(#[from] quick_xml::de::DeError),

    /// CSV parsing/serialization errors
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Server selection errors
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// I/O errors from file operations
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Application-specific errors with context
    #[error("{msg}")]
    Context {
        msg: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
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

    /// Get the category for this error.
    #[must_use]
    pub fn category(&self) -> ErrorCategory {
        match self {
            Error::NetworkError(_) => ErrorCategory::Network,
            Error::ServerListFetch(_) => ErrorCategory::Network,
            Error::DownloadTest(_) => ErrorCategory::Network,
            Error::DownloadFailure(_) => ErrorCategory::Network,
            Error::UploadTest(_) => ErrorCategory::Network,
            Error::UploadFailure(_) => ErrorCategory::Network,
            Error::IpDiscovery(_) => ErrorCategory::Network,
            Error::ParseJson(_) => ErrorCategory::Parse,
            Error::ParseXml(_) => ErrorCategory::Parse,
            Error::DeserializeXml(_) => ErrorCategory::Parse,
            Error::Csv(_) => ErrorCategory::Output,
            Error::ServerNotFound(_) => ErrorCategory::Config,
            Error::IoError(_) => ErrorCategory::Output,
            Error::Context { .. } => ErrorCategory::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn test_network_error_display() {
        // Test display via context method since we can't easily create reqwest::Error
        let err = Error::context("connection failed");
        assert_eq!(format!("{err}"), "connection failed");
    }

    #[test]
    fn test_json_error_display() {
        let invalid_json = "{invalid}";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err = Error::from(result.unwrap_err());
        assert!(format!("{err}").contains("JSON parse error"));
    }

    #[test]
    fn test_server_not_found_display() {
        let err = Error::ServerNotFound("no servers".to_string());
        assert_eq!(format!("{err}"), "Server not found: no servers");
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let speedtest_err = Error::from(io_err);
        assert!(format!("{speedtest_err}").contains("I/O error"));
    }

    #[test]
    fn test_context_error_display() {
        let err = Error::context("custom error");
        assert_eq!(format!("{err}"), "custom error");
    }

    #[test]
    fn test_context_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::with_source("Failed to read config", io_err);
        assert_eq!(format!("{err}"), "Failed to read config");
        assert!(err.source().is_some());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let speedtest_err: Error = io_err.into();
        assert!(matches!(speedtest_err, Error::IoError(_)));
        assert!(format!("{speedtest_err}").contains("I/O error"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let err = Error::context("test error");
        // Test that Error trait is implemented
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_debug_trait() {
        let err = Error::context("debug test");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Context"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid_json = "{invalid}";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
        let err: Error = result.unwrap_err().into();
        assert!(matches!(err, Error::ParseJson(_)));
    }

    #[test]
    fn test_from_quick_xml_de_error() {
        let invalid_xml = "<unclosed>";
        let result: Result<serde_json::Value, _> = quick_xml::de::from_str(invalid_xml);
        assert!(result.is_err());
        let err: Error = result.unwrap_err().into();
        assert!(matches!(err, Error::DeserializeXml(_)));
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
                let err: Error = e.into();
                assert!(matches!(err, Error::Csv(_)));
                return;
            }
        }
        panic!("Expected CSV parse error");
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::with_source("Failed to load history", io_err);

        // Verify source chain is preserved
        assert!(matches!(err, Error::Context { .. }));
        let source = err.source();
        assert!(source.is_some());

        // Verify it's an io::Error
        let source = source.unwrap();
        assert!(source.is::<std::io::Error>());
    }

    #[test]
    fn test_context_without_source() {
        let err = Error::context("standalone error");
        assert!(matches!(err, Error::Context { source: None, .. }));
        assert!(err.source().is_none());
    }
}
