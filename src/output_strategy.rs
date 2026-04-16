//! Output format resolution — extracts format selection from CLI args into
//! an `OutputFormat` strategy (Strategy pattern, OCP-compliant).
//!
//! Add new variants in `OutputFormat` (formatter/mod.rs) and extend the
//! `resolve` function below — no caller changes needed.

use crate::cli::CliArgs;
use crate::config::Config;
use crate::formatter::OutputFormat;
use crate::profiles::UserProfile;
use crate::task_runner::TestRunResult;

/// Resolve the active output format from CLI args and test results.
#[must_use]
pub fn resolve_output_format(
    args: &CliArgs,
    config: &Config,
    dl_result: &TestRunResult,
    ul_result: &TestRunResult,
    elapsed: std::time::Duration,
) -> OutputFormat {
    use crate::cli::OutputFormatType;
    let profile = config
        .profile
        .as_ref()
        .and_then(|p| UserProfile::from_name(p))
        .unwrap_or_default();
    let theme = config.theme;

    // --format flag takes precedence over legacy --json/--csv/--simple booleans
    match args.format {
        Some(OutputFormatType::Json) => OutputFormat::Json,
        Some(OutputFormatType::Jsonl) => OutputFormat::Jsonl,
        Some(OutputFormatType::Csv) => OutputFormat::Csv {
            delimiter: config.csv_delimiter,
            header: config.csv_header,
        },
        Some(OutputFormatType::Minimal) => OutputFormat::Minimal { theme },
        Some(OutputFormatType::Simple) => OutputFormat::Simple { theme },
        Some(OutputFormatType::Compact) => OutputFormat::Compact {
            dl_bytes: dl_result.total_bytes,
            ul_bytes: ul_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_duration: ul_result.duration_secs,
            elapsed,
            profile,
            theme,
        },
        Some(OutputFormatType::Dashboard) => OutputFormat::Dashboard {
            dl_mbps: dl_result.avg_bps / 1_000_000.0,
            dl_peak_mbps: dl_result.peak_bps / 1_000_000.0,
            dl_bytes: dl_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_mbps: ul_result.avg_bps / 1_000_000.0,
            ul_peak_mbps: ul_result.peak_bps / 1_000_000.0,
            ul_bytes: ul_result.total_bytes,
            ul_duration: ul_result.duration_secs,
            elapsed,
            profile,
            theme,
        },
        Some(OutputFormatType::Detailed) => OutputFormat::Detailed {
            dl_bytes: dl_result.total_bytes,
            ul_bytes: ul_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_duration: ul_result.duration_secs,
            dl_skipped: config.no_download,
            ul_skipped: config.no_upload,
            elapsed,
            profile,
            minimal: config.minimal,
            theme,
        },
        None => {
            // Default to detailed output
            OutputFormat::Detailed {
                dl_bytes: dl_result.total_bytes,
                ul_bytes: ul_result.total_bytes,
                dl_duration: dl_result.duration_secs,
                ul_duration: ul_result.duration_secs,
                dl_skipped: config.no_download,
                ul_skipped: config.no_upload,
                elapsed,
                profile,
                minimal: config.minimal,
                theme,
            }
        }
    }
}
