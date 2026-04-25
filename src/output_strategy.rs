//! Output format resolution — extracts format selection from Config into
//! an `OutputFormat` strategy (Strategy pattern, OCP-compliant).
//!
//! Add new variants in `OutputFormat` (formatter/mod.rs) and extend the
//! `resolve` function below — no caller changes needed.

use crate::config::{Config, Format};
use crate::formatter::{OutputFormat, SkipState};
use crate::profiles::UserProfile;
use crate::task_runner::TestRunResult;

/// Resolve the active output format from config and test results.
#[must_use]
pub fn resolve_output_format(
    config: &Config,
    dl_result: &TestRunResult,
    ul_result: &TestRunResult,
    elapsed: std::time::Duration,
) -> OutputFormat {
    let profile = config
        .profile()
        .and_then(UserProfile::from_name)
        .unwrap_or_default();
    let theme = config.theme();

    // --format flag takes precedence over legacy --json/--csv/--simple booleans
    match config.format() {
        Some(Format::Json) => OutputFormat::Json,
        Some(Format::Jsonl) => OutputFormat::Jsonl,
        Some(Format::Csv) => OutputFormat::Csv {
            delimiter: config.csv_delimiter(),
            header: config.csv_header(),
        },
        Some(Format::Minimal) => OutputFormat::Minimal { theme },
        Some(Format::Simple) => OutputFormat::Simple { theme },
        Some(Format::Compact) => OutputFormat::Compact {
            dl_bytes: dl_result.total_bytes,
            ul_bytes: ul_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_duration: ul_result.duration_secs,
            elapsed,
            profile,
            theme,
        },
        Some(Format::Dashboard) => OutputFormat::Dashboard {
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
        Some(Format::Detailed) => OutputFormat::Detailed {
            dl_bytes: dl_result.total_bytes,
            ul_bytes: ul_result.total_bytes,
            dl_duration: dl_result.duration_secs,
            ul_duration: ul_result.duration_secs,
            skipped: SkipState {
                download: config.no_download(),
                upload: config.no_upload(),
            },
            elapsed,
            profile,
            minimal: config.minimal(),
            theme,
        },
        None => {
            // Default to detailed output
            OutputFormat::Detailed {
                dl_bytes: dl_result.total_bytes,
                ul_bytes: ul_result.total_bytes,
                dl_duration: dl_result.duration_secs,
                ul_duration: ul_result.duration_secs,
                skipped: SkipState {
                    download: config.no_download(),
                    upload: config.no_upload(),
                },
                elapsed,
                profile,
                minimal: config.minimal(),
                theme,
            }
        }
    }
}
