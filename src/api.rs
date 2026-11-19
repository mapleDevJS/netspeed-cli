//! Supported embedding API for running a speed test and receiving its report.
//!
//! The CLI remains the primary user interface. This module provides a small,
//! quiet entry point for Rust applications that need the measured result.
//!
//! ```no_run
//! use netspeed_cli::api::{run_speed_test, TestOptions};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let report = run_speed_test(TestOptions::default()).await?;
//! println!("Download: {:?} Mbps", report.download);
//! # Ok(())
//! # }
//! ```

pub use crate::error::Error;
pub use crate::types::TestResult;

/// Options for a library speed test.
#[derive(Debug, Clone)]
pub struct TestOptions {
    /// Run the download phase. Enabled by default.
    pub download: bool,
    /// Run the upload phase. Enabled by default.
    pub upload: bool,
    /// Use one transfer stream instead of the normal multi-stream test.
    pub single_connection: bool,
    /// Optional speedtest.net server ID. Automatic selection is used by default.
    pub server_id: Option<String>,
    /// Optional local source IP address to bind network connections to.
    pub source_ip: Option<String>,
    /// Per-request HTTP timeout in seconds, from 1 through 300.
    pub timeout_secs: u64,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            download: true,
            upload: true,
            single_connection: false,
            server_id: None,
            source_ip: None,
            timeout_secs: 10,
        }
    }
}

/// Run a speed test quietly and return its measured report.
///
/// This function does not print progress or results and does not persist
/// history. It still performs real network requests to speedtest.net.
///
/// # Errors
/// Returns an error if options are invalid, setup fails, or a required
/// measurement phase fails.
pub async fn run_speed_test(options: TestOptions) -> Result<TestResult, Error> {
    if !(1..=300).contains(&options.timeout_secs) {
        return Err(Error::context("timeout_secs must be between 1 and 300"));
    }
    if let Some(source_ip) = options.source_ip.as_deref()
        && source_ip.parse::<std::net::IpAddr>().is_err()
    {
        return Err(Error::context(
            "source_ip must be a valid IPv4 or IPv6 address",
        ));
    }

    let args = crate::cli::Args {
        no_download: Some(!options.download),
        no_upload: Some(!options.upload),
        single: Some(options.single_connection),
        quiet: Some(true),
        server: options.server_id.into_iter().collect(),
        source: options.source_ip,
        timeout: options.timeout_secs,
        ..crate::cli::Args::default()
    };

    let orchestrator = crate::orchestrator::Orchestrator::new_for_library(args)?;
    orchestrator.run_for_library().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_enable_both_phases_and_use_normal_streaming() {
        let options = TestOptions::default();
        assert!(options.download);
        assert!(options.upload);
        assert!(!options.single_connection);
        assert_eq!(options.timeout_secs, 10);
        assert!(options.server_id.is_none());
        assert!(options.source_ip.is_none());
    }

    #[tokio::test]
    async fn rejects_timeout_outside_supported_range_before_network_activity() {
        let options = TestOptions {
            timeout_secs: 0,
            ..TestOptions::default()
        };
        let error = run_speed_test(options).await.unwrap_err();
        assert!(error.to_string().contains("between 1 and 300"));
    }

    #[tokio::test]
    async fn rejects_invalid_source_address_before_network_activity() {
        let options = TestOptions {
            source_ip: Some("not-an-ip".into()),
            ..TestOptions::default()
        };
        let error = run_speed_test(options).await.unwrap_err();
        assert!(error.to_string().contains("valid IPv4 or IPv6"));
    }
}
