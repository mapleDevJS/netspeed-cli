//! Terminal presentation helpers for the CLI.
//!
//! All display formatting for user-facing output lives here,
//! keeping the orchestrator focused on control flow.

use crate::error::SpeedtestError;
use crate::formatter::formatting::format_distance;
use crate::history;
use crate::progress::no_color;
use crate::types::Server;
use owo_colors::OwoColorize;

/// Dry-run configuration data, extracted from the orchestrator.
pub struct DryRunConfig {
    pub timeout: u64,
    pub format_description: &'static str,
    pub quiet: bool,
    pub source_ip: Option<String>,
    pub no_download: bool,
    pub no_upload: bool,
    pub single_stream: bool,
}

/// Return a human-readable description of the output format.
pub fn dry_run_description(format: Option<&crate::cli::OutputFormatType>) -> &'static str {
    match format {
        Some(crate::cli::OutputFormatType::Json) => "JSON",
        Some(crate::cli::OutputFormatType::Csv) => "CSV",
        Some(crate::cli::OutputFormatType::Simple) => "Simple",
        Some(crate::cli::OutputFormatType::Detailed) => "Detailed",
        Some(crate::cli::OutputFormatType::Dashboard) => "Dashboard",
        None => "Detailed (default)",
    }
}

/// Print dry-run configuration confirmation to stdout.
pub fn print_dry_run(config: &DryRunConfig) -> Result<(), SpeedtestError> {
    let nc = no_color();

    if nc {
        eprintln!("Configuration valid:");
        eprintln!("  Timeout: {}s", config.timeout);
        eprintln!("  Format: {}", config.format_description);
        if config.quiet {
            eprintln!("  Quiet: enabled");
        }
        if let Some(ref source) = config.source_ip {
            eprintln!("  Source IP: {source}");
        }
        if config.no_download {
            eprintln!("  Download test: disabled");
        }
        if config.no_upload {
            eprintln!("  Upload test: disabled");
        }
        if config.single_stream {
            eprintln!("  Streams: single");
        }
        eprintln!("\nDry run complete. Run without --dry-run to perform speed test.");
    } else {
        eprintln!("{}", "Configuration valid:".green().bold());
        eprintln!(
            "  {}: {}s",
            "Timeout".dimmed(),
            config.timeout.to_string().cyan()
        );
        eprintln!(
            "  {}: {}",
            "Format".dimmed(),
            config.format_description.white()
        );
        if config.quiet {
            eprintln!("  {}: {}", "Quiet".dimmed(), "enabled".green());
        }
        if let Some(ref source) = config.source_ip {
            eprintln!("  {}: {source}", "Source IP".dimmed());
        }
        if config.no_download {
            eprintln!("  {}: {}", "Download test".dimmed(), "disabled".yellow());
        }
        if config.no_upload {
            eprintln!("  {}: {}", "Upload test".dimmed(), "disabled".yellow());
        }
        if config.single_stream {
            eprintln!("  {}: {}", "Streams".dimmed(), "single".yellow());
        }
        eprintln!(
            "\n{}",
            "Dry run complete. Run without --dry-run to perform speed test.".bright_black()
        );
    }

    Ok(())
}

/// Format a ping result message for spinner display.
pub fn format_ping_result(avg_latency_ms: f64) -> String {
    let nc = no_color();
    if nc {
        format!("Latency: {avg_latency_ms:.2} ms")
    } else {
        format!(
            "Latency: {}",
            format!("{avg_latency_ms:.2} ms").cyan().bold()
        )
    }
}

/// Print the CLI header with version info.
pub fn print_header() {
    eprintln!(
        "{}",
        format!("  ═══  NetSpeed CLI v{}  ═══", env!("CARGO_PKG_VERSION"))
            .dimmed()
            .bold()
    );
    eprintln!("{}", "  Bandwidth test · speedtest.net".dimmed());
    eprintln!();
}

/// Print server info with distance.
pub fn print_server_info(server: &Server) {
    let dist = format_distance(server.distance);
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
        eprintln!("  {} {} ({dist})", "Location:".dimmed(), server.country);
    }
    eprintln!();
}

/// Print formatted test history to stdout.
pub fn print_history() -> Result<(), SpeedtestError> {
    let entries = history::load_history()?;

    if entries.is_empty() {
        eprintln!("No test history found.");
        return Ok(());
    }

    let nc = no_color();
    if nc {
        eprintln!("\n  TEST HISTORY");
    } else {
        eprintln!("\n  {}", "TEST HISTORY".bold().underline());
    }
    eprintln!(
        "  {:<20}  {:<15}  {:>12}  {:>14}  {:>14}",
        "Date", "Sponsor", "Ping", "Download", "Upload"
    );

    for entry in entries.iter().rev() {
        let date = entry.timestamp.get(0..10).unwrap_or(&entry.timestamp);
        // Pad short dates to 10 chars for column alignment
        let date_padded = format!("{date:<10}");
        let ping = entry.ping.map_or("-".to_string(), |p| format!("{p:.1} ms"));
        let dl = entry
            .download
            .map_or("-".to_string(), |d| format!("{:.2} Mb/s", d / 1_000_000.0));
        let ul = entry
            .upload
            .map_or("-".to_string(), |u| format!("{:.2} Mb/s", u / 1_000_000.0));

        let sponsor_display = if entry.sponsor.len() > 15 {
            let truncated: String = entry.sponsor.chars().take(12).collect();
            format!("{truncated}…")
        } else {
            entry.sponsor.clone()
        };

        if nc {
            eprintln!(
                "  {:<20}  {:<15}  {:>12}  {:>14}  {:>14}",
                date_padded, sponsor_display, ping, dl, ul
            );
        } else {
            eprintln!(
                "  {:<20}  {:<15}  {:>12}  {:>14}  {:>14}",
                date_padded,
                sponsor_display,
                ping.cyan(),
                dl.green(),
                ul.yellow()
            );
        }
    }

    Ok(())
}
