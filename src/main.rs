use netspeed_cli::cli::{CliArgs, ShellType};
use netspeed_cli::common;
use netspeed_cli::config::Config;
use netspeed_cli::error::SpeedtestError;
use netspeed_cli::formatter::{OutputFormat, format_list};
use netspeed_cli::history;
use netspeed_cli::http;
use netspeed_cli::progress::{create_spinner, finish_ok, no_color};
use netspeed_cli::servers::{fetch_servers, ping_test, select_best_server};
use netspeed_cli::test_runner::{self, TestRunResult};
use netspeed_cli::types::{self, TestResult};
use netspeed_cli::{download, upload};

use clap::Parser;
use owo_colors::OwoColorize;

fn generate_shell_completion(shell: ShellType) {
    use clap::CommandFactory;
    use clap_complete::{Shell as CompleteShell, generate};
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

    // Handle history display
    if args.history {
        history::print_history()?;
        return Ok(());
    }

    let config = Config::from_args(&args);
    let client = http::create_client(&config)?;

    let is_verbose = !config.simple && !config.json && !config.csv && !config.list;

    // Print header for verbose mode
    if is_verbose {
        eprintln!(
            "{}",
            format!("  ═══  NetSpeed CLI v{}  ═══", env!("CARGO_PKG_VERSION"))
                .dimmed()
                .bold()
        );
        eprintln!("{}", "  Bandwidth test · speedtest.net".dimmed());
        eprintln!();
    }

    // Fetch server list
    let fetch_spinner = if is_verbose {
        Some(create_spinner("Finding servers..."))
    } else {
        None
    };
    let mut servers = fetch_servers(&client).await?;
    if let Some(ref pb) = fetch_spinner {
        finish_ok(pb, &format!("Found {} servers", servers.len()));
        eprintln!();
    }

    // Handle --list option
    if config.list {
        format_list(&servers)?;
        return Ok(());
    }

    // Filter servers
    if !config.server_ids.is_empty() {
        servers.retain(|s| config.server_ids.contains(&s.id));
    }
    if !config.exclude_ids.is_empty() {
        servers.retain(|s| !config.exclude_ids.contains(&s.id));
    }

    if servers.is_empty() {
        return Err(SpeedtestError::ServerNotFound(
            "No servers match your criteria. Try running without --server/--exclude filters, or use --list to see available servers.".to_string(),
        ));
    }

    // Select best server
    let server = select_best_server(&servers)?;

    // Server info
    if is_verbose {
        let dist = common::format_distance(server.distance);
        eprintln!();
        if no_color() {
            eprintln!("  Server:   {} ({})", server.sponsor, server.name);
            eprintln!("  Location: {} ({dist})", server.country);
        } else {
            eprintln!(
                "  {}   {} ({})",
                "Server:".dimmed(),
                server.sponsor.white().bold(),
                server.name
            );
            eprintln!("  {} {} ({dist})", "Location:".dimmed(), server.country,);
        }
        eprintln!();
    }

    // Discover client IP
    let client_ip = http::discover_client_ip(&client).await.ok();

    // Run ping test
    let (ping, jitter, packet_loss, ping_samples) = if !config.no_download || !config.no_upload {
        let ping_spinner = if is_verbose {
            Some(create_spinner("Testing latency..."))
        } else {
            None
        };
        let ping_result = ping_test(&client, &server).await?;
        if let Some(ref pb) = ping_spinner {
            let msg = if no_color() {
                format!("Latency: {:.2} ms", ping_result.0)
            } else {
                format!(
                    "Latency: {}",
                    format!("{:.2} ms", ping_result.0).cyan().bold()
                )
            };
            finish_ok(pb, &msg);
        }
        (
            Some(ping_result.0),
            Some(ping_result.1),
            Some(ping_result.2),
            ping_result.3,
        )
    } else {
        (None, None, None, Vec::new())
    };

    // Run download test
    let dl_result = if config.no_download {
        TestRunResult::default()
    } else {
        test_runner::run_bandwidth_test(
            &config,
            &server,
            "Download",
            is_verbose,
            |progress| async {
                download::download_test(&client, &server, config.single, progress).await
            },
        )
        .await?
    };

    // Run upload test
    let ul_result = if config.no_upload {
        TestRunResult::default()
    } else {
        test_runner::run_bandwidth_test(&config, &server, "Upload", is_verbose, |progress| async {
            upload::upload_test(&client, &server, config.single, progress).await
        })
        .await?
    };

    // Build result
    let result = TestResult::from_test_runs(
        types::ServerInfo {
            id: server.id.clone(),
            name: server.name.clone(),
            sponsor: server.sponsor.clone(),
            country: server.country.clone(),
            distance: server.distance,
        },
        ping,
        jitter,
        packet_loss,
        ping_samples,
        &dl_result,
        &ul_result,
        client_ip,
    );

    // Save to history (unless --json or --csv)
    if !config.json && !config.csv {
        history::save_result(&result).ok();
    }

    // Output — Strategy pattern dispatch
    let output_format = if config.json {
        OutputFormat::Json
    } else if config.csv {
        OutputFormat::Csv {
            delimiter: config.csv_delimiter,
            header: config.csv_header,
        }
    } else if config.simple {
        OutputFormat::Simple
    } else {
        OutputFormat::Detailed {
            dl_bytes: dl_result.total_bytes,
            ul_bytes: ul_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_duration: ul_result.duration_secs,
        }
    };
    output_format.format(&result, config.bytes)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_speedtest().await {
        let nc = no_color();
        if nc {
            eprintln!("\nError: {e}");
            eprintln!("For more information, run: netspeed-cli --help");
        } else {
            eprintln!("\n{}", format!("Error: {e}").red().bold());
            eprintln!(
                "{}",
                "For more information, run: netspeed-cli --help".bright_black()
            );
        }
        std::process::exit(1);
    }
}
