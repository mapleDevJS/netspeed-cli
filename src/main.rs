use clap::Parser;
use netspeed_cli::cli::CliArgs;
use netspeed_cli::error::SpeedtestError;
use netspeed_cli::orchestrator::SpeedTestOrchestrator;
use netspeed_cli::terminal::no_color;
use owo_colors::OwoColorize;

/// Exit codes following sysexits.h conventions.
mod exit_codes {
    pub const SUCCESS: i32 = 0;
    #[allow(dead_code)]
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 64;
    pub const CONFIG_ERROR: i32 = 65;
    pub const NETWORK_ERROR: i32 = 69;
    pub const INTERNAL_ERROR: i32 = 70;
}

#[tokio::main]
async fn main() {
    let exit_code = run_speedtest().await;
    std::process::exit(exit_code);
}

async fn run_speedtest() -> i32 {
    let args = match CliArgs::try_parse() {
        Ok(a) => a,
        Err(e) => {
            // Clap handles --help and --version internally — let it exit properly.
            // For actual parse errors, return USAGE_ERROR.
            let _ = e.print();
            if e.use_stderr() {
                return exit_codes::USAGE_ERROR;
            }
            return exit_codes::SUCCESS;
        }
    };

    let orchestrator = match SpeedTestOrchestrator::new(args) {
        Ok(o) => o,
        Err(e) => {
            print_error(&e);
            return exit_codes::CONFIG_ERROR;
        }
    };

    match orchestrator.run().await {
        Ok(()) => exit_codes::SUCCESS,
        Err(ref e) if is_list_sentinel(e) => exit_codes::SUCCESS,
        Err(ref e) if is_network_error(e) => {
            print_error(e);
            exit_codes::NETWORK_ERROR
        }
        Err(ref e) if is_config_error(e) => {
            print_error(e);
            exit_codes::CONFIG_ERROR
        }
        Err(e) => {
            print_error(&e);
            exit_codes::INTERNAL_ERROR
        }
    }
}

/// Check if the error is the "--list was shown" sentinel.
fn is_list_sentinel(e: &SpeedtestError) -> bool {
    matches!(e, SpeedtestError::Context { msg, .. } if msg == "__list_displayed__")
}

/// Check if the error is network-related.
fn is_network_error(e: &SpeedtestError) -> bool {
    matches!(
        e,
        SpeedtestError::NetworkError(_)
            | SpeedtestError::ServerListFetch(_)
            | SpeedtestError::DownloadTest(_)
            | SpeedtestError::UploadTest(_)
            | SpeedtestError::IpDiscovery(_)
    )
}

/// Check if the error is a configuration/validation error.
fn is_config_error(e: &SpeedtestError) -> bool {
    matches!(
        e,
        SpeedtestError::ServerNotFound(_) | SpeedtestError::Context { .. }
    )
}

/// Print a user-friendly error message.
fn print_error(e: &SpeedtestError) {
    let nc = no_color();
    if nc {
        eprintln!("\nError: {e}");
        print_suggestion(e);
    } else {
        eprintln!("\n{}", format!("Error: {e}").red().bold());
        print_suggestion(e);
    }
}

/// Print contextual suggestions based on error type.
fn print_suggestion(e: &SpeedtestError) {
    let nc = no_color();
    let suggestion = match e {
        SpeedtestError::NetworkError(_)
        | SpeedtestError::ServerListFetch(_)
        | SpeedtestError::IpDiscovery(_) => {
            "Tip: Check your network connection and try again.\n      You can also use --list to verify server access."
        }
        SpeedtestError::DownloadTest(_) => {
            "Tip: Download may be blocked by a firewall or proxy.\n      Try with --single for a simpler test."
        }
        SpeedtestError::UploadTest(_) => {
            "Tip: Upload may be blocked by a firewall or proxy.\n      Try with --no-upload to skip upload testing."
        }
        SpeedtestError::ServerNotFound(_) => "Tip: Use --list to see available servers.",
        SpeedtestError::IoError(_) => "Tip: Check file permissions and disk space.",
        SpeedtestError::ParseJson(_) | SpeedtestError::ParseXml(_) => {
            "Tip: The server response was malformed. Try again later."
        }
        _ => "For more information, run: netspeed-cli --help",
    };
    if nc {
        eprintln!("{suggestion}");
    } else {
        eprintln!("{}", suggestion.bright_black());
    }
}
