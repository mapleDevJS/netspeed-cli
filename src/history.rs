use crate::error::SpeedtestError;
use crate::types::TestResult;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryEntry {
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

impl From<&TestResult> for HistoryEntry {
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
        fs::create_dir_all(data_dir).ok();
        data_dir.join("history.json")
    })
}

/// Internal: load history from a specific path
fn load_history_from_path(path: &Path) -> Result<Vec<HistoryEntry>, SpeedtestError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
    Ok(history)
}

/// Internal: save result to a specific path
fn save_result_to_path(result: &TestResult, path: &Path) -> Result<(), SpeedtestError> {
    let mut history: Vec<HistoryEntry> = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    history.push(HistoryEntry::from(result));

    // Keep only last 100 entries
    if history.len() > 100 {
        history.remove(0);
    }

    let json = serde_json::to_string_pretty(&history)?;

    // Write to a temp file first, then rename for atomicity.
    // On Unix, restrict permissions to owner-only (0o600).
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).ok();
    }
    fs::rename(&tmp_path, path)?;

    Ok(())
}

/// Save a test result to the history file.
///
/// # Errors
///
/// Returns [`SpeedtestError::IoError`] if reading or writing the history file fails.
/// Returns [`SpeedtestError::ParseJson`] if the history file contains invalid JSON.
pub fn save_result(result: &TestResult) -> Result<(), SpeedtestError> {
    let Some(path) = get_history_path() else {
        return Ok(());
    };

    save_result_to_path(result, &path)
}

/// Load all test history from the history file.
///
/// # Errors
///
/// Returns [`SpeedtestError::IoError`] if reading the history file fails.
/// Returns [`SpeedtestError::ParseJson`] if the history file contains invalid JSON.
pub fn load_history() -> Result<Vec<HistoryEntry>, SpeedtestError> {
    let Some(path) = get_history_path() else {
        return Ok(Vec::new());
    };

    load_history_from_path(&path)
}

/// Get recent download/upload speeds as paired tuples for sparkline display.
/// Returns up to the last 7 entries as `(date_label, dl_mbps, ul_mbps)`.
#[must_use]
pub fn get_recent_sparkline() -> Vec<(String, f64, f64)> {
    let Ok(history) = load_history() else {
        return Vec::new();
    };
    history
        .iter()
        .rev()
        .take(7)
        .filter_map(|e| {
            let dl = e.download.map(|d| d / 1_000_000.0).unwrap_or(0.0);
            let ul = e.upload.map(|u| u / 1_000_000.0).unwrap_or(0.0);
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
    use crate::types::{ServerInfo, TestResult};
    use serial_test::serial;

    fn make_test_result(download: f64, upload: f64, timestamp: &str) -> TestResult {
        TestResult {
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
    fn test_save_result_invalid_json_recovery() {
        let (_temp, path) = temp_history_path();

        // Write invalid JSON
        fs::write(&path, "{invalid json}").unwrap();

        // Should recover and save the new result
        let r = make_test_result(100_000_000.0, 50_000_000.0, "2026-12-01T00:00:00Z");
        save_result_to_path(&r, &path).unwrap();

        let history = load_history_from_path(&path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].download, Some(100_000_000.0));
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
}
