use clap::Parser;
use netspeed_cli::cli::Args;
use netspeed_cli::cli::OutputFormatType;
use netspeed_cli::error::Error;
use netspeed_cli::orchestrator::Orchestrator;
use netspeed_cli::terminal::no_color;
use owo_colors::OwoColorize;
use serde::Serialize;

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

fn main() {
    // Parse CLI args *before* entering the async runtime so we can safely
    // mutate process-global state (env vars) without racing against
    // concurrent tokio tasks that may read them.
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            // Clap handles --help and --version internally — let it exit properly.
            // For actual parse errors, return USAGE_ERROR.
            let _ = e.print();
            let code = if e.use_stderr() {
                exit_codes::USAGE_ERROR
            } else {
                exit_codes::SUCCESS
            };
            std::process::exit(code);
        }
    };
    let machine_error_format = machine_error_format(&args);

    // Apply --no-emoji before entering async context.
    // SAFETY: This is the only place we mutate the NO_EMOJI env var, and it
    // happens on the main thread before any async tasks are spawned. All
    // subsequent reads (from `terminal::no_emoji()`) happen inside the tokio
    // runtime, so there is no data race.
    if args.no_emoji {
        // SAFETY: No tokio tasks exist yet; the runtime is created below.
        unsafe {
            std::env::set_var("NO_EMOJI", "1");
        }
    }

    // Build orchestrator (may fail for invalid --source IP, etc.)
    // Load file config once for validation (avoids double-loading in from_args)
    let file_config = netspeed_cli::config::load_config_file();
    let orchestrator = match Orchestrator::new(args, file_config) {
        Ok(o) => o,
        Err(e) => {
            print_error(&e, exit_codes::CONFIG_ERROR, machine_error_format);
            std::process::exit(exit_codes::CONFIG_ERROR);
        }
    };

    // Enter the async runtime — no env mutations happen after this point.
    // Uses the same builder as #[tokio::main]: multi-threaded with I/O + time.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    let exit_code = rt.block_on(async { run_speedtest(orchestrator, machine_error_format).await });
    std::process::exit(exit_code);
}

async fn run_speedtest(
    orchestrator: Orchestrator,
    machine_error_format: Option<OutputFormatType>,
) -> i32 {
    match orchestrator.run().await {
        Ok(()) => exit_codes::SUCCESS,
        Err(ref e) if is_list_sentinel(e) => exit_codes::SUCCESS,
        Err(ref e) if is_network_error(e) => {
            print_error(e, exit_codes::NETWORK_ERROR, machine_error_format);
            exit_codes::NETWORK_ERROR
        }
        Err(ref e) if is_config_error(e) => {
            print_error(e, exit_codes::CONFIG_ERROR, machine_error_format);
            exit_codes::CONFIG_ERROR
        }
        Err(e) => {
            print_error(&e, exit_codes::INTERNAL_ERROR, machine_error_format);
            exit_codes::INTERNAL_ERROR
        }
    }
}

#[allow(deprecated)]
fn machine_error_format(args: &Args) -> Option<OutputFormatType> {
    match args.format {
        Some(OutputFormatType::Json | OutputFormatType::Jsonl) => args.format,
        _ if args.json.unwrap_or(false) => Some(OutputFormatType::Json),
        _ => None,
    }
}

/// Check if the error is the "--list was shown" sentinel.
fn is_list_sentinel(e: &Error) -> bool {
    matches!(e, Error::Context { msg, .. } if msg == "__list_displayed__")
}

/// Check if the error is network-related.
fn is_network_error(e: &Error) -> bool {
    matches!(
        e,
        Error::NetworkError(_)
            | Error::ServerListFetch(_)
            | Error::DownloadTest(_)
            | Error::DownloadFailure(_)
            | Error::UploadTest(_)
            | Error::UploadFailure(_)
            | Error::IpDiscovery(_)
    )
}

/// Check if the error is a configuration/validation error.
fn is_config_error(e: &Error) -> bool {
    matches!(e, Error::ServerNotFound(_) | Error::Context { .. })
}

/// Print a user-friendly error message.
fn print_error(e: &Error, exit_code: i32, machine_format: Option<OutputFormatType>) {
    if let Some(format) = machine_format {
        print_machine_error(e, exit_code, format);
        return;
    }

    let nc = no_color();
    if nc {
        eprintln!("\nError: {e}");
        print_suggestion(e);
    } else {
        eprintln!("\n{}", format!("Error: {e}").red().bold());
        print_suggestion(e);
    }
}

#[derive(Serialize)]
struct MachineErrorOutput<'a> {
    status: &'static str,
    exit_code: i32,
    timestamp: String,
    error: MachineErrorBody<'a>,
}

#[derive(Serialize)]
struct MachineErrorBody<'a> {
    code: &'static str,
    category: &'static str,
    message: String,
    suggestion: &'a str,
}

fn print_machine_error(e: &Error, exit_code: i32, format: OutputFormatType) {
    let output = render_machine_error(e, exit_code, format);
    println!("{output}");
}

fn render_machine_error(e: &Error, exit_code: i32, format: OutputFormatType) -> String {
    let (code, category) = machine_error_identity(e);
    let payload = MachineErrorOutput {
        status: "error",
        exit_code,
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: MachineErrorBody {
            code,
            category,
            message: e.to_string(),
            suggestion: suggestion_for_error(e),
        },
    };

    match format {
        OutputFormatType::Jsonl => {
            serde_json::to_string(&payload).expect("machine error JSONL serialization failed")
        }
        OutputFormatType::Json => {
            let is_tty = {
                use std::io::IsTerminal;
                std::io::stdout().is_terminal()
            };
            if is_tty {
                serde_json::to_string_pretty(&payload)
            } else {
                serde_json::to_string(&payload)
            }
            .expect("machine error JSON serialization failed")
        }
        _ => unreachable!("machine-readable error output is only supported for JSON/JSONL"),
    }
}

fn machine_error_identity(e: &Error) -> (&'static str, &'static str) {
    match e {
        Error::NetworkError(_) => ("network_error", "network"),
        Error::ServerListFetch(_) => ("server_list_fetch_failed", "network"),
        Error::DownloadTest(_) | Error::DownloadFailure(_) => ("download_failed", "network"),
        Error::UploadTest(_) | Error::UploadFailure(_) => ("upload_failed", "network"),
        Error::IpDiscovery(_) => ("ip_discovery_failed", "network"),
        Error::ParseJson(_) => ("json_parse_failed", "parse"),
        Error::ParseXml(_) | Error::DeserializeXml(_) => ("xml_parse_failed", "parse"),
        Error::Csv(_) => ("csv_output_failed", "output"),
        Error::ServerNotFound(_) => ("server_not_found", "config"),
        Error::IoError(_) => ("io_error", "io"),
        Error::Context { .. } => ("context_error", "internal"),
    }
}

/// Print contextual suggestions based on error type.
fn print_suggestion(e: &Error) {
    let nc = no_color();
    let suggestion = suggestion_for_error(e);
    if nc {
        eprintln!("{suggestion}");
    } else {
        eprintln!("{}", suggestion.bright_black());
    }
}

fn suggestion_for_error(e: &Error) -> &'static str {
    match e {
        Error::NetworkError(_) | Error::ServerListFetch(_) | Error::IpDiscovery(_) => {
            "Tip: Check your network connection and try again.\n      You can also use --list to verify server access."
        }
        Error::DownloadTest(_) | Error::DownloadFailure(_) => {
            "Tip: Download may be blocked by a firewall or proxy.\n      Try with --single for a simpler test."
        }
        Error::UploadTest(_) | Error::UploadFailure(_) => {
            "Tip: Upload may be blocked by a firewall or proxy.\n      Try with --no-upload to skip upload testing."
        }
        Error::ServerNotFound(_) => "Tip: Use --list to see available servers.",
        Error::IoError(_) => "Tip: Check file permissions and disk space.",
        Error::ParseJson(_) | Error::ParseXml(_) | Error::DeserializeXml(_) => {
            "Tip: The server response was malformed. Try again later."
        }
        _ => "For more information, run: netspeed-cli --help",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_error_format_prefers_explicit_json() {
        let args = Args::try_parse_from(["netspeed-cli", "--format", "json"]).unwrap();
        assert!(matches!(
            machine_error_format(&args),
            Some(OutputFormatType::Json)
        ));
    }

    #[test]
    fn test_machine_error_format_supports_legacy_json_flag() {
        let args = Args::try_parse_from(["netspeed-cli", "--json"]).unwrap();
        assert!(matches!(
            machine_error_format(&args),
            Some(OutputFormatType::Json)
        ));
    }

    #[test]
    fn test_machine_error_format_ignores_human_formats() {
        let args = Args::try_parse_from(["netspeed-cli", "--format", "compact"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_machine_error_identity_download_failure() {
        let (code, category) =
            machine_error_identity(&Error::DownloadFailure("zero bytes".to_string()));
        assert_eq!(code, "download_failed");
        assert_eq!(category, "network");
    }

    #[test]
    fn test_machine_error_identity_server_not_found() {
        let (code, category) =
            machine_error_identity(&Error::ServerNotFound("missing".to_string()));
        assert_eq!(code, "server_not_found");
        assert_eq!(category, "config");
    }

    #[test]
    fn test_render_machine_error_jsonl() {
        let output = render_machine_error(
            &Error::DownloadFailure("all streams failed".to_string()),
            exit_codes::NETWORK_ERROR,
            OutputFormatType::Jsonl,
        );
        let payload: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(payload["status"], "error");
        assert_eq!(payload["exit_code"], exit_codes::NETWORK_ERROR);
        assert_eq!(payload["error"]["code"], "download_failed");
        assert_eq!(payload["error"]["category"], "network");
    }
}
