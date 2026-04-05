use netspeed_cli::cli::{CliArgs, ShellType};
use netspeed_cli::config::Config;
use netspeed_cli::error::SpeedtestError;
use netspeed_cli::formatter::{
    format_csv, format_detailed, format_json, format_list, format_simple,
};
use netspeed_cli::history;
use netspeed_cli::http;
use netspeed_cli::progress::{SpeedProgress, create_spinner, finish_ok, no_color};
use netspeed_cli::servers::{
    fetch_servers, measure_latency_under_load, ping_test, select_best_server,
};
use netspeed_cli::types::{self, TestResult};
use netspeed_cli::{download, upload};

use clap::Parser;
use indicatif::ProgressDrawTarget;
use owo_colors::OwoColorize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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

    // Fetch server list
    let fetch_spinner = if is_verbose {
        Some(create_spinner("Finding servers..."))
    } else {
        None
    };
    let mut servers = fetch_servers(&client).await?;
    if let Some(ref pb) = fetch_spinner {
        finish_ok(pb, &format!("Found {} servers", servers.len()));
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
        eprintln!();
        if no_color() {
            eprintln!("  Server:   {} ({})", server.sponsor, server.name);
            eprintln!("  Location: {} ({:.1} km)", server.country, server.distance);
        } else {
            eprintln!(
                "  {}   {} ({})",
                "Server:".dimmed(),
                server.sponsor.white().bold(),
                server.name
            );
            eprintln!(
                "  {} {} ({:.1} km)",
                "Location:".dimmed(),
                server.country,
                server.distance
            );
        }
        eprintln!();
    }

    // Discover client IP
    let client_ip = http::discover_client_ip(&client).await.ok();

    // Run ping test
    let (ping, jitter) = if !config.no_download || !config.no_upload {
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
        (Some(ping_result.0), Some(ping_result.1))
    } else {
        (None, None)
    };

    // Run download test
    let (download, download_peak, latency_download, dl_bytes, dl_duration) = if config.no_download {
        (None, None, None, 0, 0.0)
    } else {
        let progress = Arc::new(if is_verbose {
            SpeedProgress::new("Download")
        } else {
            SpeedProgress::with_target("Download", ProgressDrawTarget::hidden())
        });

        let latency_samples = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let ping_client = http::create_client(&config)?;
        let ping_url = server.url.clone();
        let samples_clone = Arc::clone(&latency_samples);
        let stop_clone = Arc::clone(&stop_signal);
        let ping_handle = tokio::spawn(async move {
            measure_latency_under_load(ping_client, ping_url, samples_clone, stop_clone).await;
        });

        let test_start = std::time::Instant::now();
        let (dl_avg, dl_peak, total_bytes) =
            download::download_test(&client, &server, config.single, Arc::clone(&progress)).await?;
        let dl_duration = test_start.elapsed().as_secs_f64();
        progress.finish(dl_avg / 1_000_000.0, total_bytes);

        stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = ping_handle.await;

        let lat_under_load = {
            let lock = latency_samples.lock().unwrap();
            if lock.is_empty() {
                None
            } else {
                Some(lock.iter().sum::<f64>() / lock.len() as f64)
            }
        };

        (
            Some(dl_avg),
            Some(dl_peak),
            lat_under_load,
            total_bytes,
            dl_duration,
        )
    };

    // Run upload test
    let (upload, upload_peak, latency_upload, ul_bytes, ul_duration) = if config.no_upload {
        (None, None, None, 0, 0.0)
    } else {
        let progress = Arc::new(if is_verbose {
            SpeedProgress::new("Upload")
        } else {
            SpeedProgress::with_target("Upload", ProgressDrawTarget::hidden())
        });

        let latency_samples = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let ping_client = http::create_client(&config)?;
        let ping_url = server.url.clone();
        let samples_clone = Arc::clone(&latency_samples);
        let stop_clone = Arc::clone(&stop_signal);
        let ping_handle = tokio::spawn(async move {
            measure_latency_under_load(ping_client, ping_url, samples_clone, stop_clone).await;
        });

        let test_start = std::time::Instant::now();
        let (ul_avg, ul_peak, total_bytes) =
            upload::upload_test(&client, &server, config.single, Arc::clone(&progress)).await?;
        let ul_duration = test_start.elapsed().as_secs_f64();
        progress.finish(ul_avg / 1_000_000.0, total_bytes);

        stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = ping_handle.await;

        let lat_under_load = {
            let lock = latency_samples.lock().unwrap();
            if lock.is_empty() {
                None
            } else {
                Some(lock.iter().sum::<f64>() / lock.len() as f64)
            }
        };

        (
            Some(ul_avg),
            Some(ul_peak),
            lat_under_load,
            total_bytes,
            ul_duration,
        )
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
        jitter,
        download,
        download_peak,
        upload,
        upload_peak,
        latency_download,
        latency_upload,
        timestamp: chrono::Utc::now().to_rfc3339(),
        client_ip,
    };

    // Save to history (unless --json or --csv)
    if !config.json && !config.csv {
        history::save_result(&result).ok();
    }

    // Output
    if config.json {
        format_json(&result, config.simple)?;
    } else if config.csv {
        format_csv(&result, config.csv_delimiter, config.csv_header)?;
    } else if config.simple {
        format_simple(&result, config.bytes)?;
    } else {
        format_detailed(
            &result,
            config.bytes,
            dl_bytes,
            ul_bytes,
            dl_duration,
            ul_duration,
        )?;
    }

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
