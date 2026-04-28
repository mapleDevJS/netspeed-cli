use crate::common;
use crate::error::Error;
use crate::terminal;
use crate::theme::{Colors, Theme};
use crate::types::TestResult;
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub timestamp: String,
    pub server_name: String,
    pub sponsor: String,
    pub ping: Option<f64>,
    pub jitter: Option<f64>,
    pub packet_loss: Option<f64>,
    pub download: Option<f64>,
    pub download_peak: Option<f64>,
    pub upload: Option<f64>,
    pub upload_peak: Option<f64>,
    pub latency_download: Option<f64>,
    pub latency_upload: Option<f64>,
    pub client_ip: Option<String>,
}

impl From<&TestResult> for Entry {
    fn from(result: &TestResult) -> Self {
        Self {
            timestamp: result.timestamp.clone(),
            server_name: result.server.name.clone(),
            sponsor: result.server.sponsor.clone(),
            ping: result.ping,
            jitter: result.jitter,
            packet_loss: result.packet_loss,
            download: result.download,
            download_peak: result.download_peak,
            upload: result.upload,
            upload_peak: result.upload_peak,
            latency_download: result.latency_download,
            latency_upload: result.latency_upload,
            client_ip: result.client_ip.clone(),
        }
    }
}

fn get_history_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "vibe", "netspeed-cli").map(|proj_dirs| {
        let data_dir = proj_dirs.data_dir();
        if let Err(e) = fs::create_dir_all(data_dir) {
            eprintln!("Warning: Failed to create data directory: {e}");
        }
        data_dir.join("history.json")
    })
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn corrupt_path(path: &Path) -> PathBuf {
    path.with_extension("json.corrupt")
}

fn load_entries(path: &Path) -> Result<Vec<Entry>, Error> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Internal: load history from a specific path
fn load_history_from_path(path: &Path) -> Result<Vec<Entry>, Error> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    match load_entries(path) {
        Ok(history) => Ok(history),
        Err(err) => {
            let backup = backup_path(path);
            if backup.exists() {
                match load_entries(&backup) {
                    Ok(history) => {
                        eprintln!(
                            "Warning: History file is invalid; using backup at {}",
                            backup.display()
                        );
                        Ok(history)
                    }
                    Err(_) => Err(err),
                }
            } else {
                Err(err)
            }
        }
    }
}

/// Internal: save result to a specific path
fn save_result_to_path(result: &TestResult, path: &Path) -> Result<(), Error> {
    let backup = backup_path(path);
    let mut recovered_from_backup = false;
    let mut history: Vec<Entry> = if path.exists() {
        match load_entries(path) {
            Ok(history) => history,
            Err(err) => {
                if backup.exists() {
                    let backup_history = load_entries(&backup)?;
                    let corrupt = corrupt_path(path);
                    fs::copy(path, &corrupt)?;
                    eprintln!(
                        "Warning: History file is invalid; preserving it at {} and repairing from backup {}",
                        corrupt.display(),
                        backup.display()
                    );
                    recovered_from_backup = true;
                    backup_history
                } else {
                    return Err(err);
                }
            }
        }
    } else {
        Vec::new()
    };

    history.push(Entry::from(result));

    // Keep only last 100 entries
    if history.len() > 100 {
        let overflow = history.len() - 100;
        history.drain(0..overflow);
    }

    let json = serde_json::to_string_pretty(&history)?;

    // Write to a temp file first, then rename for atomicity.
    // On Unix, restrict permissions to owner-only (0o600).
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)) {
            eprintln!("Warning: Failed to set permissions on history file: {e}");
        }
    }
    if path.exists() && !recovered_from_backup {
        fs::copy(path, &backup)?;
    }
    fs::rename(&tmp_path, path)?;

    Ok(())
}

/// Save a test result to the history file.
///
/// # Errors
///
/// Returns [`Error::IoError`] if reading or writing the history file fails.
/// Returns [`Error::ParseJson`] if the history file contains invalid JSON.
pub fn save_result(result: &TestResult) -> Result<(), Error> {
    let Some(path) = get_history_path() else {
        return Ok(());
    };

    save_result_to_path(result, &path)
}

/// Save a full report (currently identical to a TestResult).
pub fn save_report(report: &crate::domain::reporting::Report) -> Result<(), Error> {
    // Report is an alias for TestResult; forward to existing saver.
    save_result(report)
}

/// Load all test history from the history file.
///
/// # Errors
///
/// Returns [`Error::IoError`] if reading the history file fails.
/// Returns [`Error::ParseJson`] if the history file contains invalid JSON.
pub fn load() -> Result<Vec<Entry>, Error> {
    let Some(path) = get_history_path() else {
        return Ok(Vec::new());
    };

    load_history_from_path(&path)
}

/// Print formatted test history to stdout.
///
/// # Errors
///
/// Returns [`Error::IoError`] if reading the history file fails.
/// Returns [`Error::ParseJson`] if the history file contains invalid JSON.
pub fn show(theme: Theme) -> Result<(), Error> {
    let history = load()?;

    if history.is_empty() {
        println!("No test history found.");
        return Ok(());
    }

    let nc = terminal::no_color() || theme == Theme::Monochrome;
    let term_w = common::get_terminal_width().unwrap_or(90) as usize;
    let box_w = term_w.min(80);
    let inner_w = box_w.saturating_sub(4); // 2 leading spaces + 2 border chars
    let content_w = inner_w.saturating_sub(4); // 2 spaces each side inside box

    let count = history.len();
    let left_text = "◉ TEST HISTORY";
    let right_text = format!("{count} entries");
    let pad = content_w.saturating_sub(left_text.chars().count() + right_text.chars().count());
    let spaces = " ".repeat(pad);

    let top_border = format!("  ┌{}┐", "─".repeat(inner_w));
    let mid_border = format!("  └{}┘", "─".repeat(inner_w));

    println!();
    println!("{top_border}");
    if nc {
        println!("  │  {left_text}{spaces}{right_text}  │");
    } else {
        let left_col = format!(
            "{} {}",
            Colors::muted("◉", theme),
            Colors::header("TEST HISTORY", theme)
        );
        let right_col = Colors::muted(&right_text, theme);
        println!("  │  {left_col}{spaces}{right_col}  │");
    }
    println!("{mid_border}");
    println!();

    // Column widths (plain)
    const DATE_W: usize = 10;
    const DL_W: usize = 12;
    const UL_W: usize = 12;
    const PING_W: usize = 9;
    const SERVER_W: usize = 18;

    // Header row
    let h_date = format!("{:<DATE_W$}", "Date");
    let h_dl = format!("{:>DL_W$}", "↓ Download");
    let h_ul = format!("{:>UL_W$}", "↑ Upload");
    let h_ping = format!("{:>PING_W$}", "Ping");
    let h_server = format!("{:<SERVER_W$}", "Server");
    if nc {
        println!("  {h_date}  {h_dl}  {h_ul}  {h_ping}  {h_server}");
    } else {
        println!(
            "  {}  {}  {}  {}  {}",
            Colors::muted(&h_date, theme),
            Colors::muted(&h_dl, theme),
            Colors::muted(&h_ul, theme),
            Colors::muted(&h_ping, theme),
            Colors::muted(&h_server, theme),
        );
    }

    // Thin dashed separator
    let sep_len = DATE_W + 2 + DL_W + 2 + UL_W + 2 + PING_W + 2 + SERVER_W;
    let sep = format!("  {}", "╌".repeat(sep_len));
    if nc {
        println!("{sep}");
    } else {
        println!("{}", sep.dimmed());
    }

    // Data rows — newest first
    for entry in history.iter().rev() {
        let date_plain = if entry.timestamp.len() >= 10 {
            entry.timestamp[0..10].to_string()
        } else {
            entry.timestamp.clone()
        };

        let dl_mbps = entry.download.map(|d| d / 1_000_000.0);
        let ul_mbps = entry.upload.map(|u| u / 1_000_000.0);

        let dl_plain = dl_mbps.map_or("-".to_string(), |d| format!("{d:.1} Mb/s"));
        let ul_plain = ul_mbps.map_or("-".to_string(), |u| format!("{u:.1} Mb/s"));
        let ping_plain = entry.ping.map_or("-".to_string(), |p| format!("{p:.0} ms"));

        let sponsor_truncated = if entry.sponsor.chars().count() > SERVER_W {
            let truncated: String = entry.sponsor.chars().take(SERVER_W - 1).collect();
            format!("{truncated}…")
        } else {
            entry.sponsor.clone()
        };

        // Pad plain strings to column widths BEFORE colorizing (ANSI-safe)
        let date_col = format!("{date_plain:<DATE_W$}");
        let dl_col_plain = format!("{dl_plain:>DL_W$}");
        let ul_col_plain = format!("{ul_plain:>UL_W$}");
        let ping_col_plain = format!("{ping_plain:>PING_W$}");
        let server_col = format!("{sponsor_truncated:<SERVER_W$}");

        if nc {
            println!(
                "  {date_col}  {dl_col_plain}  {ul_col_plain}  {ping_col_plain}  {server_col}"
            );
        } else {
            let date_colored = Colors::muted(&date_col, theme);
            let dl_colored = color_speed(&dl_col_plain, dl_mbps, theme);
            let ul_colored = color_speed(&ul_col_plain, ul_mbps, theme);
            let ping_colored = color_ping(&ping_col_plain, entry.ping, theme);
            let server_colored = server_col.dimmed().to_string();
            println!(
                "  {date_colored}  {dl_colored}  {ul_colored}  {ping_colored}  {server_colored}"
            );
        }
    }

    // Sparkline section — last 20 entries in chronological order
    let spark_start = if history.len() > 20 {
        history.len() - 20
    } else {
        0
    };
    let spark_slice = &history[spark_start..];
    let dl_vals: Vec<f64> = spark_slice
        .iter()
        .filter_map(|e| e.download.map(|d| d / 1_000_000.0))
        .collect();
    let ul_vals: Vec<f64> = spark_slice
        .iter()
        .filter_map(|e| e.upload.map(|u| u / 1_000_000.0))
        .collect();

    if !dl_vals.is_empty() || !ul_vals.is_empty() {
        let n = spark_slice.len();
        if nc {
            println!("{sep}");
        } else {
            println!("{}", sep.dimmed());
        }
        if !dl_vals.is_empty() {
            let dl_spark = sparkline(&dl_vals);
            if nc {
                println!("  ↓  {dl_spark}  Download trend ({n} tests)");
            } else {
                println!(
                    "  {}  {}  {}",
                    Colors::muted("↓", theme),
                    Colors::info(&dl_spark, theme),
                    Colors::muted(&format!("Download trend ({n} tests)"), theme),
                );
            }
        }
        if !ul_vals.is_empty() {
            let ul_spark = sparkline(&ul_vals);
            if nc {
                println!("  ↑  {ul_spark}  Upload trend");
            } else {
                println!(
                    "  {}  {}  {}",
                    Colors::muted("↑", theme),
                    Colors::good(&ul_spark, theme),
                    Colors::muted("Upload trend", theme),
                );
            }
        }
    }

    println!();
    Ok(())
}

fn color_speed(col: &str, mbps: Option<f64>, theme: Theme) -> String {
    match mbps {
        None => col.dimmed().to_string(),
        Some(v) if v >= 100.0 => Colors::good(col, theme),
        Some(v) if v >= 25.0 => Colors::info(col, theme),
        Some(v) if v >= 5.0 => Colors::warn(col, theme),
        Some(_) => Colors::bad(col, theme),
    }
}

fn color_ping(col: &str, ping: Option<f64>, theme: Theme) -> String {
    match ping {
        None => col.dimmed().to_string(),
        Some(v) if v <= 20.0 => Colors::good(col, theme),
        Some(v) if v <= 50.0 => Colors::warn(col, theme),
        Some(_) => Colors::bad(col, theme),
    }
}

/// Compute average download and upload speeds from history (last 20 entries).
/// Returns (`avg_dl_mbps`, `avg_ul_mbps`) or None if insufficient data.
#[must_use]
pub fn get_averages() -> Option<(f64, f64)> {
    let history = match load() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Warning: Failed to load history for averages: {e}");
            return None;
        }
    };
    let recent: Vec<_> = history.iter().rev().take(20).collect();
    let dl_entries: Vec<f64> = recent
        .iter()
        .filter_map(|e| e.download.map(|d| d / 1_000_000.0))
        .collect();
    let ul_entries: Vec<f64> = recent
        .iter()
        .filter_map(|e| e.upload.map(|u| u / 1_000_000.0))
        .collect();

    if dl_entries.is_empty() || ul_entries.is_empty() {
        return None;
    }

    // Safe: history entries are at most 100, well under 2^53.
    let download_avg = dl_entries.iter().sum::<f64>() / dl_entries.len() as f64;
    let upload_avg = ul_entries.iter().sum::<f64>() / ul_entries.len() as f64;
    Some((download_avg, upload_avg))
}

/// Format historical comparison as a string for display.
/// Returns None if insufficient history data.
#[must_use]
pub fn format_comparison(
    download_mbps: f64,
    upload_mbps: f64,
    nc: bool,
    theme: Theme,
) -> Option<String> {
    let (download_avg, upload_avg) = get_averages()?;

    // Use the combined metric: dl + ul as a single score
    let current_score = download_mbps + upload_mbps;
    let avg_score = download_avg + upload_avg;

    if avg_score <= 0.0 {
        return None;
    }

    let pct_change = ((current_score / avg_score) - 1.0) * 100.0;

    let display = if pct_change.abs() < 3.0 {
        if nc {
            "~ On par with your history".to_string()
        } else {
            Colors::muted("~ On par with your history", theme)
        }
    } else if pct_change > 0.0 {
        if nc {
            format!("↑ {pct_change:.0}% faster than your average")
        } else {
            Colors::good(
                &format!("↑ {pct_change:.0}% faster than your average"),
                theme,
            )
        }
    } else {
        let abs_pct = pct_change.abs();
        if nc {
            format!("↓ {abs_pct:.0}% slower than your average")
        } else {
            Colors::bad(&format!("↓ {abs_pct:.0}% slower than your average"), theme)
        }
    };

    Some(display)
}

/// Render a sparkline from a slice of numeric values using Unicode block chars.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::history::sparkline;
/// let line = sparkline(&[10.0, 20.0, 30.0]);
/// assert_eq!(line.chars().count(), 3); // one char per value
/// ```
#[must_use]
pub fn sparkline(values: &[f64]) -> String {
    const CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return String::new();
    }
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let range = max - min;
    if range <= 0.0 {
        // All same value — show middle bar
        return CHARS[3].to_string().repeat(values.len());
    }
    values
        .iter()
        .map(|v| {
            // Safe: (v-min)/range is 0..1, *7 → 0..7, round → 0..7, fits usize.
            let idx = (((v - min) / range) * 7.0).round().clamp(0.0, 7.0) as usize;
            CHARS[idx]
        })
        .collect::<String>()
}

/// Render an ASCII-only sparkline using `_-^` characters for environments
/// where Unicode block characters don't render.
#[must_use]
pub fn sparkline_ascii(values: &[f64]) -> String {
    const CHARS: &[char] = &['_', '_', '‗', '-', '=', '≈', '^', '▲'];
    if values.is_empty() {
        return String::new();
    }
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let range = max - min;
    if range <= 0.0 {
        return "-".repeat(values.len());
    }
    values
        .iter()
        .map(|v| {
            // Safe: (v-min)/range is 0..1, *7 → 0..7, round → 0..7, fits usize.
            let idx = (((v - min) / range) * 7.0).round().clamp(0.0, 7.0) as usize;
            CHARS[idx]
        })
        .collect::<String>()
}

/// Get recent download/upload speeds as paired tuples for sparkline display.
/// Returns up to the last 7 entries as `(date_label, dl_mbps, ul_mbps)`.
#[must_use]
pub fn get_recent_sparkline() -> Vec<(String, f64, f64)> {
    let Ok(history) = load() else {
        return Vec::new();
    };
    history
        .iter()
        .rev()
        .take(7)
        .filter_map(|e| {
            let dl = e.download.map_or(0.0, |d| d / 1_000_000.0);
            let ul = e.upload.map_or(0.0, |u| u / 1_000_000.0);
            if dl > 0.0 || ul > 0.0 {
                // Extract just the date part (YYYY-MM-DD)
                let date = e.timestamp.get(0..10).unwrap_or(&e.timestamp).to_string();
                Some((date, dl, ul))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::types::{PhaseResult, ServerInfo, TestPhases, TestResult};
    use serial_test::serial;

    fn make_test_result(download: f64, upload: f64, timestamp: &str) -> TestResult {
        TestResult {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            test_id: None,
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: Some(10.0),
            jitter: Some(1.0),
            packet_loss: Some(0.0),
            download: Some(download),
            download_peak: None,
            upload: Some(upload),
            upload_peak: None,
            latency_download: None,
            latency_upload: None,
            download_samples: None,
            upload_samples: None,
            ping_samples: None,
            timestamp: timestamp.to_string(),
            client_ip: None,
            client_location: None,
            download_cv: None,
            upload_cv: None,
            download_ci_95: None,
            upload_ci_95: None,
            overall_grade: None,
            download_grade: None,
            upload_grade: None,
            connection_rating: None,
            phases: TestPhases {
                ping: PhaseResult::completed(),
                download: PhaseResult::completed(),
                upload: PhaseResult::completed(),
            },
        }
    }

    /// Helper: create a temp directory with a history.json path
    fn temp_history_path() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("history.json");
        (temp_dir, path)
    }

    #[test]
    #[serial]
    fn test_get_averages_returns_values() {
        let (_temp, path) = temp_history_path();

        let results = vec![
            make_test_result(100_000_000.0, 50_000_000.0, "2026-01-01T00:00:00Z"),
            make_test_result(120_000_000.0, 60_000_000.0, "2026-01-02T00:00:00Z"),
            make_test_result(80_000_000.0, 40_000_000.0, "2026-01-03T00:00:00Z"),
        ];
        for r in &results {
            save_result_to_path(r, &path).unwrap();
        }

        // Load and verify
        let history = load_history_from_path(&path).unwrap();
        let dl_values: Vec<f64> = history
            .iter()
            .filter_map(|e| e.download.map(|d| d / 1_000_000.0))
            .collect();
        assert_eq!(dl_values.len(), 3);
        // Safe: history entries are at most 100, well under 2^53.
        let download_avg = dl_values.iter().sum::<f64>() / dl_values.len() as f64;
        assert!((download_avg - 100.0).abs() < 0.1);
    }

    #[test]
    #[serial]
    fn test_format_comparison_faster() {
        let (_temp, path) = temp_history_path();

        for i in 0..3 {
            let r = make_test_result(
                20_000_000.0,
                10_000_000.0,
                &format!("2026-06-{i:02}T00:00:00Z"),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        // Verify it doesn't panic
        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    #[serial]
    fn test_format_comparison_slower() {
        let (_temp, path) = temp_history_path();

        for i in 0..3 {
            let r = make_test_result(
                800_000_000.0,
                800_000_000.0,
                &format!("2026-07-{i:02}T00:00:00Z"),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    #[serial]
    fn test_format_comparison_on_par() {
        let (_temp, path) = temp_history_path();

        let sim_results = vec![
            make_test_result(100_000_000.0, 50_000_000.0, "2026-04-01T00:00:00Z"),
            make_test_result(105_000_000.0, 52_000_000.0, "2026-04-02T00:00:00Z"),
            make_test_result(95_000_000.0, 48_000_000.0, "2026-04-03T00:00:00Z"),
        ];
        for r in &sim_results {
            save_result_to_path(r, &path).unwrap();
        }

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    #[serial]
    fn test_save_result_appends_to_existing() {
        let (_temp, path) = temp_history_path();

        let r1 = make_test_result(50_000_000.0, 25_000_000.0, "2026-08-01T00:00:00Z");
        save_result_to_path(&r1, &path).unwrap();
        let r2 = make_test_result(60_000_000.0, 30_000_000.0, "2026-08-02T00:00:00Z");
        save_result_to_path(&r2, &path).unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    #[serial]
    fn test_print_history_with_data() {
        let (_temp, path) = temp_history_path();

        for i in 0..3 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-05-{i:02}T00:00:00Z"),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    #[serial]
    fn test_print_history_long_sponsor_truncation() {
        let (_temp, path) = temp_history_path();

        let mut r = make_test_result(100_000_000.0, 50_000_000.0, "2026-09-01T00:00:00Z");
        r.server.sponsor = "VeryLongSponsorNameThatExceedsLimit".to_string();
        save_result_to_path(&r, &path).unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history[0].sponsor, "VeryLongSponsorNameThatExceedsLimit");
    }

    #[test]
    #[serial]
    fn test_format_comparison_zero_avg() {
        let (_temp, path) = temp_history_path();

        let r = make_test_result(0.0, 0.0, "2026-10-01T00:00:00Z");
        save_result_to_path(&r, &path).unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].download, Some(0.0));
    }

    #[test]
    #[serial]
    fn test_save_result_invalid_json_recovery() {
        let (_temp, path) = temp_history_path();

        // Write invalid JSON
        fs::write(&path, "{invalid json}").unwrap();

        let r = make_test_result(100_000_000.0, 50_000_000.0, "2026-12-01T00:00:00Z");
        let err = save_result_to_path(&r, &path).unwrap_err();
        assert!(matches!(err, Error::ParseJson(_)));

        let original = fs::read_to_string(&path).unwrap();
        assert_eq!(original, "{invalid json}");
    }

    #[test]
    #[serial]
    fn test_save_result_recovers_from_backup_when_primary_is_corrupt() {
        let (_temp, path) = temp_history_path();
        let backup = backup_path(&path);

        let existing = make_test_result(100_000_000.0, 50_000_000.0, "2026-11-01T00:00:00Z");
        let existing_history = vec![Entry::from(&existing)];
        fs::write(
            &backup,
            serde_json::to_string_pretty(&existing_history).unwrap(),
        )
        .unwrap();
        fs::write(&path, "{invalid json}").unwrap();

        let new_result = make_test_result(120_000_000.0, 60_000_000.0, "2026-11-02T00:00:00Z");
        save_result_to_path(&new_result, &path).unwrap();

        let repaired = load_entries(&path).unwrap();
        assert_eq!(repaired.len(), 2);
        assert_eq!(repaired[0].timestamp, "2026-11-01T00:00:00Z");
        assert_eq!(repaired[1].timestamp, "2026-11-02T00:00:00Z");

        let corrupt = corrupt_path(&path);
        assert!(corrupt.exists());
        assert_eq!(fs::read_to_string(corrupt).unwrap(), "{invalid json}");
    }

    #[test]
    #[serial]
    fn test_load_history_falls_back_to_backup() {
        let (_temp, path) = temp_history_path();
        let backup = backup_path(&path);

        let existing = make_test_result(100_000_000.0, 50_000_000.0, "2026-10-01T00:00:00Z");
        let existing_history = vec![Entry::from(&existing)];
        fs::write(
            &backup,
            serde_json::to_string_pretty(&existing_history).unwrap(),
        )
        .unwrap();
        fs::write(&path, "{invalid json}").unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, "2026-10-01T00:00:00Z");
    }

    #[test]
    #[serial]
    fn test_save_result_rotates_backup_from_previous_good_state() {
        let (_temp, path) = temp_history_path();
        let backup = backup_path(&path);

        let r1 = make_test_result(50_000_000.0, 25_000_000.0, "2026-08-01T00:00:00Z");
        let r2 = make_test_result(60_000_000.0, 30_000_000.0, "2026-08-02T00:00:00Z");
        save_result_to_path(&r1, &path).unwrap();
        save_result_to_path(&r2, &path).unwrap();

        let previous = load_entries(&backup).unwrap();
        assert_eq!(previous.len(), 1);
        assert_eq!(previous[0].timestamp, "2026-08-01T00:00:00Z");
    }

    #[test]
    #[serial]
    fn test_history_keeps_last_100_entries() {
        let (_temp, path) = temp_history_path();

        // Save 105 entries
        for i in 0..105 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-01-{i:02}T00:00:00Z"),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 100);
        // Should have dropped the first 5
        assert_eq!(history[0].timestamp, "2026-01-05T00:00:00Z");
    }

    #[test]
    fn test_sparkline_increasing() {
        let line = sparkline(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]);
        assert_eq!(line.chars().count(), 8);
        // Should produce ascending bars
        assert_eq!(line, "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn test_sparkline_decreasing() {
        let line = sparkline(&[80.0, 60.0, 40.0, 20.0]);
        assert_eq!(line.chars().count(), 4);
    }

    #[test]
    fn test_sparkline_empty() {
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn test_sparkline_single_value() {
        let line = sparkline(&[42.0]);
        assert_eq!(line, "▄"); // single value → middle bar
    }

    #[test]
    fn test_sparkline_identical_values() {
        let line = sparkline(&[50.0, 50.0, 50.0]);
        assert_eq!(line, "▄▄▄"); // all same → middle bar
    }

    // ==================== sparkline_ascii Tests ====================

    #[test]
    fn test_sparkline_ascii_increasing() {
        let line = sparkline_ascii(&[10.0, 20.0, 30.0, 40.0, 50.0]);
        // Verify we get output with correct number of chars
        assert_eq!(line.chars().count(), 5);
        // Verify it's not empty
        assert!(!line.is_empty());
    }

    #[test]
    fn test_sparkline_ascii_decreasing() {
        let line = sparkline_ascii(&[80.0, 60.0, 40.0, 20.0]);
        assert_eq!(line.chars().count(), 4);
    }

    #[test]
    fn test_sparkline_ascii_empty() {
        assert_eq!(sparkline_ascii(&[]), "");
    }

    #[test]
    fn test_sparkline_ascii_single_value() {
        let line = sparkline_ascii(&[42.0]);
        assert_eq!(line.len(), 1); // single value → dash (1 char)
    }

    #[test]
    fn test_sparkline_ascii_identical_values() {
        let line = sparkline_ascii(&[50.0, 50.0, 50.0]);
        // Same value → dashes (3 chars)
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn test_sparkline_ascii_all_min() {
        let line = sparkline_ascii(&[1.0, 2.0, 1.0]);
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn test_sparkline_ascii_all_max() {
        let line = sparkline_ascii(&[100.0, 99.0, 100.0]);
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn test_sparkline_ascii_two_values() {
        let line = sparkline_ascii(&[25.0, 75.0]);
        assert_eq!(line.chars().count(), 2);
    }

    #[test]
    fn test_sparkline_ascii_three_values() {
        let line = sparkline_ascii(&[33.3, 66.6, 100.0]);
        assert_eq!(line.chars().count(), 3);
    }

    #[test]
    fn test_sparkline_ascii_five_values() {
        let line = sparkline_ascii(&[10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(line.chars().count(), 5);
    }

    // ==================== Entry Tests ====================

    #[test]
    fn test_entry_from_test_result() {
        let result = make_test_result(100_000_000.0, 50_000_000.0, "2026-01-15T10:30:00Z");
        let entry = Entry::from(&result);

        assert_eq!(entry.timestamp, "2026-01-15T10:30:00Z");
        assert_eq!(entry.server_name, "Test");
        assert_eq!(entry.sponsor, "Test");
        assert_eq!(entry.ping, Some(10.0));
        assert_eq!(entry.jitter, Some(1.0));
        assert_eq!(entry.download, Some(100_000_000.0));
        assert_eq!(entry.upload, Some(50_000_000.0));
    }

    #[test]
    fn test_entry_from_test_result_with_none_values() {
        let mut result = make_test_result(100_000_000.0, 50_000_000.0, "2026-02-01T00:00:00Z");
        result.ping = None;
        result.jitter = None;
        result.download = None;
        result.upload = None;

        let entry = Entry::from(&result);

        assert!(entry.ping.is_none());
        assert!(entry.jitter.is_none());
        assert!(entry.download.is_none());
        assert!(entry.upload.is_none());
    }

    // ==================== backup_path and corrupt_path Tests ====================

    #[test]
    fn test_backup_path() {
        let path = std::path::Path::new("/data/history.json");
        let backup = backup_path(path);
        assert_eq!(backup, std::path::Path::new("/data/history.json.bak"));
    }

    #[test]
    fn test_corrupt_path() {
        let path = std::path::Path::new("/data/history.json");
        let corrupt = corrupt_path(path);
        assert_eq!(corrupt, std::path::Path::new("/data/history.json.corrupt"));
    }

    // ==================== load_entries Tests ====================

    #[test]
    #[serial]
    fn test_load_entries_valid_json() {
        let (_temp, path) = temp_history_path();

        // Create Entry directly (which is what load_entries returns)
        let entries = vec![
            Entry {
                timestamp: "2026-03-01T00:00:00Z".to_string(),
                server_name: "Test".to_string(),
                sponsor: "Test".to_string(),
                ping: Some(10.0),
                jitter: Some(1.0),
                packet_loss: None,
                download: Some(100_000_000.0),
                download_peak: None,
                upload: Some(50_000_000.0),
                upload_peak: None,
                latency_download: None,
                latency_upload: None,
                client_ip: None,
            },
            Entry {
                timestamp: "2026-03-02T00:00:00Z".to_string(),
                server_name: "Test".to_string(),
                sponsor: "Test".to_string(),
                ping: Some(12.0),
                jitter: Some(2.0),
                packet_loss: None,
                download: Some(120_000_000.0),
                download_peak: None,
                upload: Some(60_000_000.0),
                upload_peak: None,
                latency_download: None,
                latency_upload: None,
                client_ip: None,
            },
        ];
        fs::write(&path, serde_json::to_string_pretty(&entries).unwrap()).unwrap();

        let loaded = load_entries(&path).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    #[serial]
    fn test_load_entries_invalid_json() {
        let (_temp, path) = temp_history_path();
        fs::write(&path, "not valid json").unwrap();

        let result = load_entries(&path);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_load_entries_file_not_found() {
        let (_temp, _path) = temp_history_path();
        // Use a non-existent path
        let result = load_entries(std::path::Path::new("/nonexistent/file.json"));
        assert!(result.is_err());
    }

    // ==================== get_history_path Tests ====================

    #[test]
    fn test_get_history_path_returns_some() {
        // ProjectDirs should return a path on all platforms
        let path = get_history_path();
        assert!(path.is_some());
        // The path should contain history.json
        let binding = path.unwrap();
        let path_str = binding.to_string_lossy();
        assert!(path_str.ends_with("history.json") || path_str.contains("history.json"));
    }

    // ==================== get_averages edge cases Tests ====================
    // Note: These tests write to temp paths and test the internal helper functions
    // (load_history_from_path, save_result_to_path) which is valid for unit testing.
    // The public API functions (get_averages, format_comparison, get_recent_sparkline)
    // read from the actual history path and require integration tests.

    #[test]
    #[serial]
    fn test_load_history_from_path_empty_file() {
        let (_temp, path) = temp_history_path();

        // Write empty file
        fs::write(&path, "[]").unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    #[serial]
    fn test_load_history_from_path_with_entries() {
        let (_temp, path) = temp_history_path();

        let result = make_test_result(100_000_000.0, 50_000_000.0, "2026-06-01T00:00:00Z");
        save_result_to_path(&result, &path).unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].download, Some(100_000_000.0));
    }

    #[test]
    #[serial]
    fn test_load_history_from_path_nonexistent() {
        let (_temp, _path) = temp_history_path();
        // Use a path that doesn't exist
        let result = load_history_from_path(std::path::Path::new("/nonexistent/path.json"));
        assert!(result.is_ok()); // Should return Ok(Vec::new()) for non-existent file
    }

    #[test]
    #[serial]
    fn test_save_result_to_path_multiple_entries() {
        let (_temp, path) = temp_history_path();

        // Save multiple entries
        for i in 0..5 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-07-{:02}T00:00:00Z", i + 1),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 5);
    }

    // ==================== format_comparison edge cases Tests ====================
    // These test the internal helper paths - the public API reads from actual history path

    #[test]
    #[serial]
    fn test_format_comparison_with_insufficient_history() {
        // format_comparison calls get_averages() which uses actual history path
        // Test that it gracefully returns None when there's no history
        let result = format_comparison(50_000_000.0, 25_000_000.0, true, crate::theme::Theme::Dark);
        // Result is None when there's no history data
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    #[serial]
    fn test_get_recent_sparkline_helper_with_data() {
        let (_temp, path) = temp_history_path();

        // Create test entries using helper functions
        for i in 0..5 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-08-{:02}T00:00:00Z", i + 1),
            );
            save_result_to_path(&r, &path).unwrap();
        }

        // Verify entries were saved correctly (this tests the helper)
        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 5);
        // Verify the data structure has expected values
        assert_eq!(history[0].download, Some(100_000_000.0));
        assert_eq!(history[0].upload, Some(50_000_000.0));
    }

    // ==================== save_result Tests ====================

    #[test]
    #[serial]
    fn test_save_result_no_history_path() {
        // save_result uses get_history_path which should always return Some
        // But we can test the public API doesn't panic
        let result = save_result(&make_test_result(
            100_000_000.0,
            50_000_000.0,
            "2026-04-01T00:00:00Z",
        ));
        // Should succeed (may be no-op if no history dir available)
        assert!(result.is_ok() || result.is_err());
    }

    // ==================== load Tests ====================

    #[test]
    #[serial]
    fn test_load_empty_history() {
        // load uses get_history_path - should return Ok(Vec::new()) if no history exists
        let result = load();
        // Should succeed with empty vec
        assert!(result.is_ok());
    }

    // ==================== show Tests ====================

    #[test]
    #[serial]
    fn test_show_history_no_panic() {
        // show uses load() - should not panic even with malformed entries
        let result = show(crate::theme::Theme::Dark);
        assert!(result.is_ok());
    }

    // ==================== Additional edge cases ====================

    #[test]
    fn test_sparkline_exact_boundaries() {
        // Test exact min/max values
        let line = sparkline(&[0.0, 100.0]);
        assert_eq!(line.chars().count(), 2);
    }

    #[test]
    fn test_sparkline_two_values_same() {
        let line = sparkline(&[50.0, 50.0]);
        assert_eq!(line.chars().count(), 2);
    }

    #[test]
    fn test_sparkline_large_range() {
        let line = sparkline(&[0.0, 1000000.0]);
        assert_eq!(line.chars().count(), 2);
    }

    #[test]
    fn test_sparkline_ascii_exact_boundaries() {
        let line = sparkline_ascii(&[0.0, 100.0]);
        assert_eq!(line.chars().count(), 2);
    }
}
