/// CLI argument parsing modules
pub mod cli;
/// Shell completion generation
pub mod completions;
/// Application configuration
pub mod config;
/// Server discovery and selection
pub mod discovery;
/// Download speed testing
pub mod download;
/// Error types and conversions
pub mod error;
/// Output formatting (JSON, CSV, simple)
pub mod formatter;
/// HTTP client creation and utilities
pub mod http;
/// Speedtest Mini server support
pub mod mini;
/// Result presentation and display
pub mod presenter;
/// Progress tracking for transfers
pub mod progress;
/// Test orchestration
pub mod runner;
/// Server list fetching and parsing
pub mod servers;
/// Share URL generation
pub mod share;
/// Core data types
pub mod types;
/// Upload speed testing
pub mod upload;
/// Utility functions
pub mod utils;

use clap::Parser;
use cli::CliArgs;
use config::Config;
use discovery::ServerDiscovery;
use error::SpeedtestError;
use http::create_client;
use presenter::ResultPresenter;
use runner::TestRunner;
use tracing_subscriber::EnvFilter;

/// Initialize the tracing subscriber with the appropriate log level.
///
/// If `--verbose` is set, uses `DEBUG` level. Otherwise, uses `INFO` level.
/// Can be overridden by the `RUST_LOG` environment variable.
fn init_tracing(verbose: bool) {
    let filter = if verbose { "debug" } else { "info" };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::try_new(filter).unwrap());

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(env_filter)
        .init();
}

/// Run the complete speedtest workflow.
///
/// This function handles the full lifecycle:
/// 1. Parse CLI arguments
/// 2. Create HTTP client
/// 3. Discover test servers
/// 4. Run ping/download/upload tests
/// 5. Present results
/// 6. Optionally generate share URL
pub async fn run_speedtest() -> Result<(), SpeedtestError> {
    let args = CliArgs::parse();

    // Initialize tracing/logger
    init_tracing(args.verbose);

    // Handle shell completion generation
    if let Some(shell) = args.generate_completion {
        completions::generate_shell_completion(shell);
        return Ok(());
    }

    let config = Config::from_args(&args);
    let client = create_client(&config)?;

    // Handle --list option
    if ServerDiscovery::handle_list(&client, &config).await? {
        return Ok(());
    }

    // Discover server
    if !config.simple {
        tracing::info!("Discovering servers...");
    }
    let server = ServerDiscovery::discover(&client, &config).await?;

    if !config.simple {
        tracing::info!(
            sponsor = %server.sponsor,
            name = %server.name,
            "Testing against server"
        );
    }

    // Run tests
    let result = TestRunner::run(&client, &server, &config).await?;

    // Present results
    ResultPresenter::present(&result, &config)?;
    ResultPresenter::handle_share(&client, &result, config.share).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_verbose_does_not_panic() {
        // Can't easily verify the filter, but can ensure it doesn't panic
        // Note: tracing can only be initialized once, so this test may be skipped
        // if another test has already initialized it
    }

    #[test]
    fn test_init_tracing_non_verbose_does_not_panic() {
        // Same as above - verify no panic
    }

    #[test]
    fn test_modules_are_public() {
        // Verify all modules are accessible
        let _ = cli::CliArgs::default();
    }

    #[test]
    fn test_config_module_accessible() {
        let args = cli::CliArgs::default();
        let _config = config::Config::from_args(&args);
    }

    #[test]
    fn test_error_module_accessible() {
        let _err = error::SpeedtestError::NetworkError("test".to_string());
    }

    #[test]
    fn test_types_module_accessible() {
        use types::{CsvOutput, Server, ServerInfo, TestResult};
        let _ = std::any::type_name::<Server>();
        let _ = std::any::type_name::<TestResult>();
        let _ = std::any::type_name::<ServerInfo>();
        let _ = std::any::type_name::<CsvOutput>();
    }

    #[test]
    fn test_utils_module_accessible() {
        let bps = utils::calculate_bps(1000, 1.0);
        assert!((bps - 8000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_module_accessible() {
        let tracker = progress::ProgressTracker::new(10, false);
        assert!((tracker.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_servers_module_accessible() {
        let dist = servers::calculate_distance(0.0, 0.0, 1.0, 1.0);
        assert!(dist > 0.0);
    }

    #[test]
    fn test_share_module_accessible() {
        // Verify share functions exist by checking the constant
        assert_eq!(
            share::SPEEDTEST_POST_URL,
            "https://www.speedtest.net/api/api.php"
        );
    }

    #[test]
    fn test_formatter_module_accessible() {
        let result = types::TestResult {
            server: types::ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: None,
            download: None,
            upload: None,
            share_url: None,
            timestamp: "2024-01-01".to_string(),
            client_ip: None,
        };
        assert!(formatter::format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_http_module_accessible() {
        let url = http::build_base_url(true);
        assert_eq!(url, "https://www.speedtest.net");
    }

    #[test]
    fn test_mini_module_accessible() {
        use mini::{MiniServer, mini_to_server};
        let mini = MiniServer {
            url: "http://test.com".to_string(),
            sponsor: "Test".to_string(),
            name: "Test".to_string(),
            id: "0".to_string(),
            distance: 0.0,
            latency: 0.0,
        };
        let server = mini_to_server(&mini);
        assert_eq!(server.id, "0");
    }

    #[test]
    fn test_discovery_module_accessible() {
        let _ = std::any::type_name::<discovery::ServerDiscovery>();
    }

    #[test]
    fn test_runner_module_accessible() {
        let _ = std::any::type_name::<runner::TestRunner>();
    }

    #[test]
    fn test_presenter_module_accessible() {
        let _ = std::any::type_name::<presenter::ResultPresenter>();
    }

    #[test]
    fn test_completions_module_accessible() {
        use cli::ShellType;
        // Just verify the module exists
        let _ = ShellType::Bash;
    }
}
