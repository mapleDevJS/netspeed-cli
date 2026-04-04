mod cli;
mod config;
mod download;
mod error;
mod formatter;
mod http;
mod servers;
mod share;
mod types;
mod upload;

use clap::Parser;
use cli::CliArgs;
use config::Config;
use error::SpeedtestError;
use formatter::{format_csv, format_json, format_list, format_simple};
use http::create_client;
use servers::{fetch_servers, select_best_server};
use types::TestResult;

async fn run_speedtest() -> Result<(), SpeedtestError> {
    let args = CliArgs::parse();

    // Handle shell completion generation
    if let Some(shell) = args.generate_completion {
        return Ok(());
    }

    let config = Config::from_args(&args);
    let client = create_client(&config)?;

    // Fetch server list
    let mut servers = fetch_servers(&client, &config).await?;

    // Handle --list option
    if config.list {
        format_list(&servers)?;
        return Ok(());
    }

    // Filter servers based on --server and --exclude options
    if !config.server_ids.is_empty() {
        servers.retain(|s| config.server_ids.contains(&s.id));
    }
    if !config.exclude_ids.is_empty() {
        servers.retain(|s| !config.exclude_ids.contains(&s.id));
    }

    if servers.is_empty() {
        return Err(SpeedtestError::ServerNotFound(
            "No servers available for testing".to_string(),
        ));
    }

    // Select best server (closest with lowest latency)
    let server = select_best_server(&servers)?;

    if !config.simple {
        eprintln!("Testing against server: {} ({})", server.sponsor, server.name);
    }

    // Discover client IP
    let client_ip = http::discover_client_ip(&client).await.ok();
    
    // Run ping test
    let ping = if !config.no_download || !config.no_upload {
        let ping_result = servers::ping_test(&client, &server).await?;
        if !config.simple {
            eprintln!("Ping: {:.3} ms", ping_result);
        }
        Some(ping_result)
    } else {
        None
    };

    // Run download test
    let download = if !config.no_download {
        let dl_result = download::download_test(&client, &server, config.single).await?;
        if !config.simple {
            eprintln!("Download: {:.2} Mbit/s", dl_result / 1_000_000.0);
        }
        Some(dl_result)
    } else {
        None
    };

    // Run upload test
    let upload = if !config.no_upload {
        let ul_result = upload::upload_test(&client, &server, config.single).await?;
        if !config.simple {
            eprintln!("Upload: {:.2} Mbit/s", ul_result / 1_000_000.0);
        }
        Some(ul_result)
    } else {
        None
    };

    // Build result
    let result = TestResult {
        server: types::ServerInfo {
            id: server.id.clone(),
            name: server.name.clone(),
            sponsor: server.sponsor.clone(),
            country: server.country.clone(),
            distance: server.distance,
        },
        ping,
        download,
        upload,
        share_url: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
        client_ip,
    };

    // Handle output formatting
    if config.json {
        format_json(&result, config.simple)?;
    } else if config.csv {
        format_csv(&result, config.csv_delimiter, config.csv_header)?;
    } else if config.simple {
        format_simple(&result, config.bytes)?;
    }

    // Handle share URL generation
    if config.share {
        let share_url = share::generate_share_url(&client, &result).await?;
        eprintln!("Share results: {}", share_url);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_speedtest().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
