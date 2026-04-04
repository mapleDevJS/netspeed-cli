use crate::config::Config;
use crate::error::SpeedtestError;
use crate::formatter::{format_csv, format_json, format_simple};
use crate::share;
use crate::types::TestResult;
use reqwest::Client;

/// Handles result presentation (output formatting and sharing).
///
/// Delegates to the formatter module for specific output formats
/// (simple, JSON, CSV, list).
pub struct ResultPresenter;

impl ResultPresenter {
    /// Format and display results according to config
    pub fn present(result: &TestResult, config: &Config) -> Result<(), SpeedtestError> {
        if config.json {
            format_json(result)?;
        } else if config.csv {
            format_csv(result, config.csv_delimiter, config.csv_header)?;
        } else if config.simple {
            format_simple(result, config.bytes)?;
        }

        Ok(())
    }

    /// Handle share URL generation if requested
    pub async fn handle_share(
        client: &Client,
        result: &TestResult,
        share_requested: bool,
    ) -> Result<(), SpeedtestError> {
        if share_requested {
            let share_url = share::generate_share_url(client, result).await?;
            tracing::info!(%share_url, "Share URL generated");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliArgs;
    use crate::types::ServerInfo;

    fn create_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.0,
            },
            ping: Some(25.0),
            download: Some(100_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: None,
        }
    }

    #[test]
    fn test_present_json_mode() {
        let args = CliArgs { json: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_present_csv_mode() {
        let args = CliArgs { csv: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_present_simple_mode() {
        let args = CliArgs { simple: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_present_no_format_flags() {
        let config = Config::from_args(&CliArgs::default());
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_present_csv_with_header() {
        let args = CliArgs { csv: true, csv_header: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_present_bytes_mode() {
        let args = CliArgs { simple: true, bytes: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_test_result();

        let res = ResultPresenter::present(&result, &config);
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_share_not_requested() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = Config::from_args(&CliArgs::default());
        let client = crate::http::create_client(&config).unwrap();
        let result = create_test_result();

        let res = rt.block_on(ResultPresenter::handle_share(
            &client,
            &result,
            false,
        ));
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_share_requested_error_on_network_failure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = Config::from_args(&CliArgs::default());
        let client = crate::http::create_client(&config).unwrap();
        let result = create_test_result();

        // Share requested but network call will fail (no mock)
        let res = rt.block_on(ResultPresenter::handle_share(
            &client,
            &result,
            true,
        ));
        // Expected to fail with network error
        assert!(res.is_err());
    }
}
