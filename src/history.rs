use crate::error::Error;
use crate::terminal;
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
pub fn show() -> Result<(), Error> {
    let history = load()?;

    if history.is_empty() {
        println!("No test history found.");
        return Ok(());
    }

    let nc = terminal::no_color();
    println!();
    if nc {
        println!("  TEST HISTORY");
    } else {
        println!("  {}", "TEST HISTORY".bold().underline());
    }
    println!(
        "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
        "Date", "Sponsor", "Ping", "Download", "Upload"
    );

    for entry in history.iter().rev() {
        let date = &entry.timestamp[0..10];
        let ping = entry.ping.map_or("-".to_string(), |p| format!("{p:.1} ms"));
        let dl = entry
            .download
            .map_or("-".to_string(), |d| format!("{:.2} Mb/s", d / 1_000_000.0));
        let ul = entry
            .upload
            .map_or("-".to_string(), |u| format!("{:.2} Mb/s", u / 1_000_000.0));

        let ping_display = if nc { ping } else { format!("{}", ping.cyan()) };
        let dl_display = if nc { dl } else { format!("{}", dl.green()) };
        let ul_display = if nc { ul } else { format!("{}", ul.yellow()) };

        println!(
            "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
            date,
            if entry.sponsor.len() > 15 {
                &entry.sponsor[0..12]
            } else {
                &entry.sponsor
            },
            ping_display,
            dl_display,
            ul_display
        );
    }

    Ok(())
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
pub fn format_comparison(download_mbps: f64, upload_mbps: f64, nc: bool) -> Option<String> {
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
            "~ On par with your history".bright_black().to_string()
        }
    } else if pct_change > 0.0 {
        if nc {
            format!("↑ {pct_change:.0}% faster than your average")
        } else {
            format!("↑ {pct_change:.0}% faster than your average")
                .green()
                .to_string()
        }
    } else {
        let abs_pct = pct_change.abs();
        if nc {
            format!("↓ {abs_pct:.0}% slower than your average")
        } else {
            format!("↓ {abs_pct:.0}% slower than your average")
                .red()
                .to_string()
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
}
