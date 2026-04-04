pub mod cli;
pub mod completions;
pub mod config;
pub mod discovery;
pub mod download;
pub mod error;
pub mod formatter;
pub mod http;
pub mod mini;
pub mod presenter;
pub mod progress;
pub mod runner;
pub mod servers;
pub mod share;
pub mod types;
pub mod upload;
pub mod utils;

use clap::Parser;
use cli::CliArgs;
use config::Config;
use discovery::ServerDiscovery;
use error::SpeedtestError;
use http::create_client;
use presenter::ResultPresenter;
use runner::TestRunner;

async fn run_speedtest() -> Result<(), SpeedtestError> {
    let args = CliArgs::parse();

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
        eprintln!("Discovering servers...");
    }
    let server = ServerDiscovery::discover(&client, &config).await?;

    if !config.simple {
        eprintln!(
            "Testing against server: {} ({})",
            server.sponsor, server.name
        );
    }

    // Run tests
    let result = TestRunner::run(&client, &server, &config).await?;

    // Present results
    ResultPresenter::present(&result, &config)?;
    ResultPresenter::handle_share(&client, &result, config.share).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_speedtest().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
