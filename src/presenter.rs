use reqwest::Client;
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::formatter::{format_csv, format_json, format_simple};
use crate::share;
use crate::types::TestResult;

/// Handles result presentation (output formatting and sharing)
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
            eprintln!("Share results: {}", share_url);
        }

        Ok(())
    }
}
