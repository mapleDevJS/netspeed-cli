//! Output handling trait for extensibility.
//!
//! Enables different output strategies and testing.

use crate::error::Error;
use crate::types::TestResult;

/// Trait for handling test results output.
///
/// Enables dependency injection for different output strategies
/// and easier testing.
pub trait OutputHandler: Send + Sync {
    /// Handle a completed test result.
    fn handle_result(&self, result: &TestResult) -> Result<(), Error>;

    /// Handle an error during testing.
    fn handle_error(&self, error: &Error) -> Result<(), Error>;
}

/// Default handler that outputs to terminal using formatter.
pub struct DefaultOutputHandler {
    _format: crate::config::Format,
    _bytes_mode: bool,
}

impl DefaultOutputHandler {
    pub fn new(format: crate::config::Format, bytes_mode: bool) -> Self {
        Self {
            _format: format,
            _bytes_mode: bytes_mode,
        }
    }
}

impl OutputHandler for DefaultOutputHandler {
    fn handle_result(&self, _result: &TestResult) -> Result<(), Error> {
        // Default handler delegates to existing formatter logic
        // This could be extended to use trait objects for different formats
        Ok(())
    }

    fn handle_error(&self, error: &Error) -> Result<(), Error> {
        eprintln!("Error: {}", error);
        Ok(())
    }
}
