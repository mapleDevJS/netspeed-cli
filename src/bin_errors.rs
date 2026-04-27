//! Binary-level error handling for netspeed-cli.
//!
//! This module contains the error presentation logic that is specific to
//! the binary entry point. Library users should use the [`Error`] type
//! directly without this presentation layer.
//!
//! ## Architecture
//!
//! - Exit codes following sysexits.h conventions
//! - Machine-readable error output for JSON/JSONL formats
//! - User-friendly error messages with suggestions

use crate::cli::OutputFormatType;
use crate::error::Error;
use crate::terminal::no_color;
use owo_colors::OwoColorize;
use serde::Serialize;

/// Exit codes following sysexits.h conventions.
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    #[allow(dead_code)]
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 64;
    pub const CONFIG_ERROR: i32 = 65;
    pub const NETWORK_ERROR: i32 = 69;
    pub const INTERNAL_ERROR: i32 = 70;
}

#[derive(Serialize)]
pub struct MachineErrorOutput<'a> {
    status: &'static str,
    exit_code: i32,
    timestamp: String,
    error: MachineErrorBody<'a>,
}

#[derive(Serialize)]
pub struct MachineErrorBody<'a> {
    code: &'static str,
    category: &'static str,
    message: String,
    suggestion: &'a str,
}

/// Determine machine-readable error format from CLI args.
pub fn machine_error_format(args: &crate::cli::Args) -> Option<OutputFormatType> {
    match args.format {
        Some(OutputFormatType::Json | OutputFormatType::Jsonl) => args.format,
        #[allow(deprecated)]
        _ if args.json.unwrap_or(false) => Some(OutputFormatType::Json),
        _ => None,
    }
}

/// Check if the error is the "--list was shown" sentinel.
pub fn is_list_sentinel(e: &Error) -> bool {
    matches!(e, Error::Context { msg, .. } if msg == "__list_displayed__")
}

/// Check if the error is network-related.
pub fn is_network_error(e: &Error) -> bool {
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
pub fn is_config_error(e: &Error) -> bool {
    matches!(e, Error::ServerNotFound(_) | Error::Context { .. })
}

/// Print a user-friendly error message.
pub fn print_error(e: &Error, exit_code: i32, machine_format: Option<OutputFormatType>) {
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

/// Output machine-readable error.
pub fn print_machine_error(e: &Error, exit_code: i32, format: OutputFormatType) {
    let output = render_machine_error(e, exit_code, format);
    println!("{output}");
}

/// Render error to machine-readable string.
pub fn render_machine_error(e: &Error, exit_code: i32, format: OutputFormatType) -> String {
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

/// Map error to machine-readable code and category.
pub fn machine_error_identity(e: &Error) -> (&'static str, &'static str) {
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

/// Get suggestion text for error type.
pub fn suggestion_for_error(e: &Error) -> &'static str {
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

/// Select exit code based on error type.
pub fn select_exit_code(e: &Error) -> i32 {
    if is_list_sentinel(e) {
        exit_codes::SUCCESS
    } else if is_network_error(e) {
        exit_codes::NETWORK_ERROR
    } else if is_config_error(e) {
        exit_codes::CONFIG_ERROR
    } else {
        exit_codes::INTERNAL_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormatType;
    use clap::Parser;

    #[test]
    fn test_exit_codes_values() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::USAGE_ERROR, 64);
        assert_eq!(exit_codes::CONFIG_ERROR, 65);
        assert_eq!(exit_codes::NETWORK_ERROR, 69);
        assert_eq!(exit_codes::INTERNAL_ERROR, 70);
    }

    #[test]
    fn test_machine_error_format_json() {
        let args = crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "json"]).unwrap();
        assert!(matches!(
            machine_error_format(&args),
            Some(OutputFormatType::Json)
        ));
    }

    #[test]
    fn test_machine_error_format_jsonl() {
        let args = crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "jsonl"]).unwrap();
        assert!(matches!(
            machine_error_format(&args),
            Some(OutputFormatType::Jsonl)
        ));
    }

    #[test]
    fn test_machine_error_format_legacy_json_flag() {
        let args = crate::cli::Args::try_parse_from(["netspeed-cli", "--json"]).unwrap();
        assert!(matches!(
            machine_error_format(&args),
            Some(OutputFormatType::Json)
        ));
    }

    #[test]
    fn test_machine_error_format_ignores_human_formats() {
        let args =
            crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "compact"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_machine_error_format_ignores_dashboard() {
        let args =
            crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "dashboard"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_machine_error_format_ignores_detailed() {
        let args =
            crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "detailed"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_machine_error_format_ignores_simple() {
        let args =
            crate::cli::Args::try_parse_from(["netspeed-cli", "--format", "simple"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_machine_error_format_none_by_default() {
        let args = crate::cli::Args::try_parse_from(["netspeed-cli"]).unwrap();
        assert!(machine_error_format(&args).is_none());
    }

    #[test]
    fn test_is_list_sentinel_true() {
        let sentinel = Error::Context {
            msg: "__list_displayed__".into(),
            source: None,
        };
        assert!(is_list_sentinel(&sentinel));
    }

    #[test]
    fn test_is_list_sentinel_false_different_message() {
        let other = Error::Context {
            msg: "other error".into(),
            source: None,
        };
        assert!(!is_list_sentinel(&other));
    }

    #[test]
    fn test_is_list_sentinel_false_different_error_type() {
        assert!(!is_list_sentinel(&Error::DownloadFailure("test".into())));
        assert!(!is_list_sentinel(&Error::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found"
        ))));
    }

    #[test]
    fn test_is_network_error_download_failure() {
        assert!(is_network_error(&Error::DownloadFailure("test".into())));
    }

    #[test]
    fn test_is_network_error_upload_failure() {
        assert!(is_network_error(&Error::UploadFailure("test".into())));
    }

    #[test]
    fn test_is_network_error_false_context() {
        assert!(!is_network_error(&Error::Context {
            msg: "config error".into(),
            source: None,
        }));
    }

    #[test]
    fn test_is_network_error_false_server_not_found() {
        assert!(!is_network_error(&Error::ServerNotFound("missing".into())));
    }

    #[test]
    fn test_is_config_error_server_not_found() {
        assert!(is_config_error(&Error::ServerNotFound("missing".into())));
    }

    #[test]
    fn test_is_config_error_context() {
        let err = Error::Context {
            msg: "config error".into(),
            source: None,
        };
        assert!(is_config_error(&err));
    }

    #[test]
    fn test_is_config_error_false_network() {
        assert!(!is_config_error(&Error::DownloadFailure("test".into())));
        assert!(!is_config_error(&Error::UploadFailure("test".into())));
    }

    #[test]
    fn test_machine_error_identity_download_failure() {
        let err = Error::DownloadFailure("zero bytes".into());
        let (code, category) = machine_error_identity(&err);
        assert_eq!(code, "download_failed");
        assert_eq!(category, "network");
    }

    #[test]
    fn test_machine_error_identity_upload_failure() {
        let err = Error::UploadFailure("timeout".into());
        let (code, category) = machine_error_identity(&err);
        assert_eq!(code, "upload_failed");
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
    fn test_machine_error_identity_parse_json() {
        let (code, category) = machine_error_identity(&Error::ParseJson(
            serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
        ));
        assert_eq!(code, "json_parse_failed");
        assert_eq!(category, "parse");
    }

    #[test]
    fn test_machine_error_identity_parse_xml() {
        let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid xml");
        let xml_err = quick_xml::Error::Io(io_err.into());
        let err = Error::ParseXml(xml_err);
        let (code, category) = machine_error_identity(&err);
        assert_eq!(code, "xml_parse_failed");
        assert_eq!(category, "parse");
    }

    #[test]
    fn test_machine_error_identity_deserialize_xml() {
        let invalid_xml = "<unclosed>";
        let result: Result<serde_json::Value, _> = quick_xml::de::from_str(invalid_xml);
        assert!(result.is_err());
        let err = Error::DeserializeXml(result.unwrap_err());
        let (code, category) = machine_error_identity(&err);
        assert_eq!(code, "xml_parse_failed");
        assert_eq!(category, "parse");
    }

    #[test]
    fn test_machine_error_identity_csv() {
        let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "csv error");
        let csv_err = csv::Error::from(io_err);
        let err = Error::Csv(csv_err);
        let (code, category) = machine_error_identity(&err);
        assert_eq!(code, "csv_output_failed");
        assert_eq!(category, "output");
    }

    #[test]
    fn test_machine_error_identity_io_error() {
        let (code, category) = machine_error_identity(&Error::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        )));
        assert_eq!(code, "io_error");
        assert_eq!(category, "io");
    }

    #[test]
    fn test_machine_error_identity_context() {
        let (code, category) = machine_error_identity(&Error::Context {
            msg: "test".into(),
            source: None,
        });
        assert_eq!(code, "context_error");
        assert_eq!(category, "internal");
    }

    #[test]
    fn test_machine_error_identity_context_with_source() {
        let source_err = Error::IoError(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "timed out",
        ));
        let (code, category) = machine_error_identity(&Error::Context {
            msg: "nested error".into(),
            source: Some(Box::new(source_err)),
        });
        assert_eq!(code, "context_error");
        assert_eq!(category, "internal");
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
        assert!(payload["error"]["message"].is_string());
        assert!(payload["error"]["suggestion"].is_string());
        assert!(!output.contains('\n'));
    }

    #[test]
    fn test_render_machine_error_json() {
        let output = render_machine_error(
            &Error::DownloadFailure("test error".to_string()),
            exit_codes::NETWORK_ERROR,
            OutputFormatType::Json,
        );
        let payload: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(payload["status"], "error");
        assert_eq!(payload["error"]["code"], "download_failed");
        assert!(payload["error"]["message"].is_string());
        assert!(payload["error"]["suggestion"].is_string());
    }

    #[test]
    fn test_render_machine_error_timestamp_format() {
        let output = render_machine_error(
            &Error::ServerNotFound("missing".to_string()),
            exit_codes::CONFIG_ERROR,
            OutputFormatType::Json,
        );
        let payload: serde_json::Value = serde_json::from_str(&output).unwrap();
        let timestamp = payload["timestamp"].as_str().unwrap();
        assert!(timestamp.contains("T") || timestamp.contains(" "));
        assert!(timestamp.ends_with('Z') || timestamp.ends_with("+00:00"));
    }

    #[test]
    fn test_render_machine_error_all_fields_present() {
        let output = render_machine_error(
            &Error::UploadFailure("connection reset".to_string()),
            exit_codes::NETWORK_ERROR,
            OutputFormatType::Json,
        );
        let payload: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(payload.get("status").is_some());
        assert!(payload.get("exit_code").is_some());
        assert!(payload.get("timestamp").is_some());
        assert!(payload.get("error").is_some());

        let error = &payload["error"];
        assert!(error.get("code").is_some());
        assert!(error.get("category").is_some());
        assert!(error.get("message").is_some());
        assert!(error.get("suggestion").is_some());
    }

    #[test]
    fn test_suggestion_for_error_download_failure() {
        let err = Error::DownloadFailure("connection timed out".into());
        let suggestion = suggestion_for_error(&err);
        assert!(suggestion.contains("firewall") || suggestion.contains("--single"));
    }

    #[test]
    fn test_suggestion_for_error_upload_failure() {
        let err = Error::UploadFailure("timeout".into());
        let suggestion = suggestion_for_error(&err);
        assert!(suggestion.contains("firewall") || suggestion.contains("--no-upload"));
    }

    #[test]
    fn test_suggestion_for_error_server_not_found() {
        let suggestion = suggestion_for_error(&Error::ServerNotFound("missing".into()));
        assert!(suggestion.contains("--list"));
    }

    #[test]
    fn test_suggestion_for_error_io_error() {
        let suggestion = suggestion_for_error(&Error::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        )));
        assert!(suggestion.contains("permissions") || suggestion.contains("disk"));
    }

    #[test]
    fn test_suggestion_for_error_parse_json() {
        let suggestion = suggestion_for_error(&Error::ParseJson(
            serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
        ));
        assert!(suggestion.contains("malformed") || suggestion.contains("Try again"));
    }

    #[test]
    fn test_suggestion_for_error_parse_xml() {
        let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid xml");
        let xml_err = quick_xml::Error::Io(io_err.into());
        let err = Error::ParseXml(xml_err);
        let suggestion = suggestion_for_error(&err);
        assert!(suggestion.contains("malformed") || suggestion.contains("Try again"));
    }

    #[test]
    fn test_suggestion_for_error_deserialize_xml() {
        let invalid_xml = "<unclosed>";
        let result: Result<serde_json::Value, _> = quick_xml::de::from_str(invalid_xml);
        assert!(result.is_err());
        let err = Error::DeserializeXml(result.unwrap_err());
        let suggestion = suggestion_for_error(&err);
        assert!(suggestion.contains("malformed") || suggestion.contains("Try again"));
    }

    #[test]
    fn test_suggestion_for_error_default() {
        let err = Error::Context {
            msg: "unknown".into(),
            source: None,
        };
        let suggestion = suggestion_for_error(&err);
        assert!(suggestion.contains("--help"));
    }

    #[test]
    fn test_select_exit_code_network_error() {
        let err = Error::DownloadFailure("test".into());
        assert_eq!(select_exit_code(&err), exit_codes::NETWORK_ERROR);
    }

    #[test]
    fn test_select_exit_code_config_error() {
        let err = Error::ServerNotFound("missing".into());
        assert_eq!(select_exit_code(&err), exit_codes::CONFIG_ERROR);
    }

    #[test]
    fn test_select_exit_code_list_sentinel() {
        let err = Error::Context {
            msg: "__list_displayed__".into(),
            source: None,
        };
        assert_eq!(select_exit_code(&err), exit_codes::SUCCESS);
    }

    #[test]
    fn test_select_exit_code_io_error() {
        // IoError is not network, config, or list sentinel → INTERNAL_ERROR
        let err = Error::IoError(std::io::Error::other("internal error"));
        assert_eq!(select_exit_code(&err), exit_codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_machine_error_output_serialization() {
        let output = MachineErrorOutput {
            status: "error",
            exit_code: 69,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            error: MachineErrorBody {
                code: "test_code",
                category: "test_category",
                message: "Test error message".to_string(),
                suggestion: "Test suggestion",
            },
        };

        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["status"], "error");
        assert_eq!(parsed["exit_code"], 69);
        assert_eq!(parsed["error"]["code"], "test_code");
        assert_eq!(parsed["error"]["category"], "test_category");
        assert_eq!(parsed["error"]["message"], "Test error message");
        assert_eq!(parsed["error"]["suggestion"], "Test suggestion");
    }

    #[test]
    fn test_machine_error_body_serialization() {
        let body = MachineErrorBody {
            code: "network_error",
            category: "network",
            message: "Connection failed".to_string(),
            suggestion: "Check connection",
        };

        let json = serde_json::to_string(&body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["code"], "network_error");
        assert_eq!(parsed["category"], "network");
        assert_eq!(parsed["message"], "Connection failed");
        assert_eq!(parsed["suggestion"], "Check connection");
    }
}
