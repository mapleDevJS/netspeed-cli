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
        "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
        "Date", "Sponsor", "Ping", "Download", "Upload"
    );

    for entry in entries.iter().rev() {
        let date = &entry.timestamp[0..10];
        let ping = entry.ping.map_or("-".to_string(), |p| format!("{p:.1} ms"));
        let dl = entry
            .download
            .map_or("-".to_string(), |d| format!("{:.2} Mb/s", d / 1_000_000.0));
        let ul = entry
            .upload
            .map_or("-".to_string(), |u| format!("{:.2} Mb/s", u / 1_000_000.0));

        let sponsor_display = if entry.sponsor.len() > 15 {
            &entry.sponsor[0..12]
        } else {
            &entry.sponsor
        };

        if nc {
            eprintln!(
                "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
                date, sponsor_display, ping, dl, ul
            );
        } else {
            eprintln!(
                "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
                date,
                sponsor_display,
                ping.cyan(),
                dl.green(),
                ul.yellow()
            );
        }
    }

    Ok(())
}
