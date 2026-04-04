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
use cli::{CliArgs, ShellType};
use config::Config;
use error::SpeedtestError;
use formatter::{format_csv, format_json, format_list, format_simple};
use http::create_client;
use servers::{fetch_servers, select_best_server};
use types::TestResult;

fn generate_shell_completion(shell: ShellType) {
    use clap_complete::{generate, Shell as CompleteShell};
    use clap::CommandFactory;
    use std::io;

    let shell_type = match shell {
        ShellType::Bash => CompleteShell::Bash,
        ShellType::Zsh => CompleteShell::Zsh,
        ShellType::Fish => CompleteShell::Fish,
        ShellType::PowerShell => CompleteShell::PowerShell,
        ShellType::Elvish => CompleteShell::Elvish,
    };

    let mut cmd = CliArgs::command();
    let bin_name = "netspeed-cli";
    generate(shell_type, &mut cmd, bin_name, &mut io::stdout());
}

async fn run_speedtest() -> Result<(), SpeedtestError> {
    let args = CliArgs::parse();

    // Handle shell completion generation
    if let Some(shell) = args.generate_completion {
        generate_shell_completion(shell);
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

/// Filter servers based on provided include/exclude lists
#[allow(dead_code)]
pub fn filter_servers(servers: &mut Vec<types::Server>, server_ids: &[String], exclude_ids: &[String]) {
    if !server_ids.is_empty() {
        servers.retain(|s| server_ids.contains(&s.id));
    }
    if !exclude_ids.is_empty() {
        servers.retain(|s| !exclude_ids.contains(&s.id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Server;

    #[test]
    fn test_filter_servers_by_include_list() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 15.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 200.0,
                latency: 25.0,
            },
        ];

        filter_servers(&mut servers, &["1".to_string()], &[]);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "1");
    }

    #[test]
    fn test_filter_servers_by_exclude_list() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 15.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 200.0,
                latency: 25.0,
            },
        ];

        filter_servers(&mut servers, &[], &["2".to_string()]);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "1");
    }

    #[test]
    fn test_filter_servers_empty_lists() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 15.0,
            },
        ];

        let original_len = servers.len();
        filter_servers(&mut servers, &[], &[]);
        assert_eq!(servers.len(), original_len);
    }

    #[test]
    fn test_filter_servers_no_matches() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 15.0,
            },
        ];

        filter_servers(&mut servers, &["999".to_string()], &[]);
        assert!(servers.is_empty());
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_speedtest().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
