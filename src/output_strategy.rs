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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigSource, OutputSource, TestSource};
    use crate::task_runner::TestRunResult;

    /// Helper to create a minimal download TestRunResult for testing
    fn make_test_run(dl_bps: f64, dl_peak: f64, dl_bytes: u64, dl_dur: f64) -> TestRunResult {
        TestRunResult {
            avg_bps: dl_bps,
            peak_bps: dl_peak,
            total_bytes: dl_bytes,
            duration_secs: dl_dur,
            speed_samples: vec![dl_bps],
            latency_under_load: None,
        }
    }

    fn make_upload_run(bps: f64, peak: f64, bytes: u64, dur: f64) -> TestRunResult {
        TestRunResult {
            avg_bps: bps,
            peak_bps: peak,
            total_bytes: bytes,
            duration_secs: dur,
            speed_samples: vec![bps],
            latency_under_load: None,
        }
    }

    fn make_config(format: Option<Format>) -> Config {
        let source = ConfigSource {
            output: OutputSource {
                format,
                theme: String::from(if cfg!(windows) { "monochrome" } else { "dark" }),
                ..Default::default()
            },
            ..Default::default()
        };
        Config::from_source(&source)
    }

    #[test]
    fn test_resolve_output_format_json() {
        let config = make_config(Some(Format::Json));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        assert!(matches!(result, OutputFormat::Json));
    }

    #[test]
    fn test_resolve_output_format_jsonl() {
        let config = make_config(Some(Format::Jsonl));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        assert!(matches!(result, OutputFormat::Jsonl));
    }

    #[test]
    fn test_resolve_output_format_csv() {
        let source = ConfigSource {
            output: OutputSource {
                format: Some(Format::Csv),
                csv_delimiter: ';',
                csv_header: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::from_source(&source);
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        match result {
            OutputFormat::Csv { delimiter, header } => {
                assert_eq!(delimiter, ';');
                assert!(header);
            }
            _ => panic!("Expected Csv format"),
        }
    }

    #[test]
    fn test_resolve_output_format_minimal() {
        let config = make_config(Some(Format::Minimal));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        match result {
            OutputFormat::Minimal { theme } => {
                assert_eq!(
                    theme,
                    if cfg!(windows) {
                        crate::theme::Theme::Monochrome
                    } else {
                        crate::theme::Theme::Dark
                    }
                );
            }
            _ => panic!("Expected Minimal format"),
        }
    }

    #[test]
    fn test_resolve_output_format_simple() {
        let config = make_config(Some(Format::Simple));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        match result {
            OutputFormat::Simple { theme } => {
                assert_eq!(
                    theme,
                    if cfg!(windows) {
                        crate::theme::Theme::Monochrome
                    } else {
                        crate::theme::Theme::Dark
                    }
                );
            }
            _ => panic!("Expected Simple format"),
        }
    }

    #[test]
    fn test_resolve_output_format_compact() {
        let config = make_config(Some(Format::Compact));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        match result {
            OutputFormat::Compact {
                dl_bytes,
                ul_bytes,
                elapsed,
                profile,
                ..
            } => {
                assert_eq!(dl_bytes, 10_000_000);
                assert_eq!(ul_bytes, 5_000_000);
                assert_eq!(elapsed.as_secs(), 5);
                assert_eq!(profile, UserProfile::PowerUser); // default
            }
            _ => panic!("Expected Compact format"),
        }
    }

    #[test]
    fn test_resolve_output_format_dashboard() {
        let config = make_config(Some(Format::Dashboard));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(10));
        match result {
            OutputFormat::Dashboard {
                dl_mbps,
                dl_peak_mbps,
                ul_mbps,
                ul_peak_mbps,
                elapsed,
                ..
            } => {
                assert!((dl_mbps - 100.0).abs() < 0.01);
                assert!((dl_peak_mbps - 120.0).abs() < 0.01);
                assert!((ul_mbps - 50.0).abs() < 0.01);
                assert!((ul_peak_mbps - 60.0).abs() < 0.01);
                assert_eq!(elapsed.as_secs(), 10);
            }
            _ => panic!("Expected Dashboard format"),
        }
    }

    #[test]
    fn test_resolve_output_format_detailed() {
        let config = make_config(Some(Format::Detailed));
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(8));
        match result {
            OutputFormat::Detailed {
                dl_bytes,
                ul_bytes,
                elapsed,
                skipped,
                minimal,
                ..
            } => {
                assert_eq!(dl_bytes, 10_000_000);
                assert_eq!(ul_bytes, 5_000_000);
                assert_eq!(elapsed.as_secs(), 8);
                assert!(!skipped.download);
                assert!(!skipped.upload);
                assert!(!minimal);
            }
            _ => panic!("Expected Detailed format"),
        }
    }

    #[test]
    fn test_resolve_output_format_detailed_with_skipped() {
        let source = ConfigSource {
            test: TestSource {
                no_download: Some(true),
                no_upload: Some(true),
                ..Default::default()
            },
            output: OutputSource {
                format: Some(Format::Detailed),
                minimal: Some(true),
                theme: String::from("light"),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::from_source(&source);
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(3));
        match result {
            OutputFormat::Detailed {
                skipped,
                minimal,
                theme,
                ..
            } => {
                assert!(skipped.download);
                assert!(skipped.upload);
                assert!(minimal);
                assert_eq!(theme, crate::theme::Theme::Light);
            }
            _ => panic!("Expected Detailed format"),
        }
    }

    #[test]
    fn test_resolve_output_format_default_none() {
        // When format is None, default to Detailed
        let source = ConfigSource {
            output: OutputSource {
                format: None,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::from_source(&source);
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        assert!(matches!(result, OutputFormat::Detailed { .. }));
    }

    #[test]
    fn test_resolve_output_format_with_profile() {
        let source = ConfigSource {
            output: OutputSource {
                format: Some(Format::Compact),
                profile: Some(String::from("gamer")),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::from_source(&source);
        let dl = make_test_run(100_000_000.0, 120_000_000.0, 10_000_000, 2.0);
        let ul = make_upload_run(50_000_000.0, 60_000_000.0, 5_000_000, 1.0);

        let result = resolve_output_format(&config, &dl, &ul, std::time::Duration::from_secs(5));
        match result {
            OutputFormat::Compact { profile, .. } => {
                assert_eq!(profile, UserProfile::Gamer);
            }
            _ => panic!("Expected Compact format"),
        }
    }
}
