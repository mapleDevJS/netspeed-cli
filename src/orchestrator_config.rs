//! Orchestrator configuration and dependency injection.
//!
//! This module provides the `OrchestratorConfig` struct that wraps all
//! dependencies needed by the orchestrator, following dependency inversion
//! principles to reduce direct module coupling.
//!
//! ## Usage
//!
//! Instead of passing multiple arguments to orchestrator methods, pass a single
//! `OrchestratorConfig` that encapsulates:
//! - HTTP client
//! - Configuration
//! - CLI arguments
//! - Resolved output format

use crate::cli::CliArgs;
use crate::config::Config;
use crate::formatter::OutputFormat;
use crate::http::HttpSettings;
use reqwest::Client;

/// Configuration container for orchestrator dependencies.
///
/// This struct reduces the orchestrator's direct imports from ~14 to ~4,
/// grouping related dependencies into cohesive units.
pub struct OrchestratorConfig {
    /// HTTP client for network requests
    pub client: Client,
    /// Parsed configuration (CLI + config file + defaults)
    pub config: Config,
    /// Original CLI arguments
    pub args: CliArgs,
    /// Resolved output format (Strategy pattern)
    pub output_format: OutputFormat,
    /// Resolved user profile for grading
    pub profile: crate::profiles::UserProfile,
}

impl OrchestratorConfig {
    /// Build orchestrator configuration from CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client creation fails or config is invalid.
    pub fn from_args(args: CliArgs) -> Result<Self, crate::error::SpeedtestError> {
        let config = Config::from_args(&args);

        let http_settings = HttpSettings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            user_agent: HttpSettings::default().user_agent,
        };
        let client = crate::http::create_client(&http_settings)?;

        let profile = crate::profiles::UserProfile::from_name(
            args.profile.as_deref().unwrap_or("power-user"),
        )
        .unwrap_or_default();

        // Output format will be resolved later with test results
        let output_format = crate::output_strategy::resolve_output_format(
            &args,
            &config,
            &crate::task_runner::TestRunResult::default(),
            &crate::task_runner::TestRunResult::default(),
            std::time::Duration::ZERO,
        );

        Ok(Self {
            client,
            config,
            args,
            output_format,
            profile,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliArgs;
    use clap::Parser;

    #[test]
    fn test_orchestrator_config_creation() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = OrchestratorConfig::from_args(args);
        assert!(config.is_ok());
    }

    #[test]
    fn test_orchestrator_config_default_profile() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = OrchestratorConfig::from_args(args).unwrap();
        // Default profile should be PowerUser
        assert_eq!(config.profile, crate::profiles::UserProfile::PowerUser);
    }
}
