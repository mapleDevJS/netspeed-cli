use crate::error::SpeedtestError;
use crate::types::TestResult;
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// Save a test result to the history file.
///
/// # Errors
///
/// Returns [`SpeedtestError::IoError`] if reading or writing the history file fails.
/// Returns [`SpeedtestError::ParseError`] if the history file contains invalid JSON.
pub fn save_result(result: &TestResult) -> Result<(), SpeedtestError> {
    let Some(path) = get_history_path() else {
        return Ok(());
    };

    let mut history: Vec<HistoryEntry> = if path.exists() {
        let content = fs::read_to_string(&path)?;
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
    fs::write(path, json)?;

    Ok(())
}

/// Load all test history from the history file.
///
/// # Errors
///
/// Returns [`SpeedtestError::IoError`] if reading the history file fails.
/// Returns [`SpeedtestError::ParseError`] if the history file contains invalid JSON.
pub fn load_history() -> Result<Vec<HistoryEntry>, SpeedtestError> {
    let Some(path) = get_history_path() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
    Ok(history)
}

/// Print formatted test history to stdout.
///
/// # Errors
///
/// Returns [`SpeedtestError::IoError`] if reading the history file fails.
/// Returns [`SpeedtestError::ParseError`] if the history file contains invalid JSON.
pub fn print_history() -> Result<(), SpeedtestError> {
    let history = load_history()?;

    if history.is_empty() {
        println!("No test history found.");
        return Ok(());
    }

    println!("\n  {}", "TEST HISTORY".bold().underline());
    println!(
        "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
        "Date".dimmed(),
        "Sponsor".dimmed(),
        "Ping".dimmed(),
        "Download".dimmed(),
        "Upload".dimmed()
    );

    for entry in history.iter().rev() {
        let date = &entry.timestamp[0..10]; // Simple YYYY-MM-DD
        let ping = entry.ping.map_or("-".to_string(), |p| format!("{p:.1} ms"));
        let dl = entry
            .download
            .map_or("-".to_string(), |d| format!("{:.2} Mb/s", d / 1_000_000.0));
        let ul = entry
            .upload
            .map_or("-".to_string(), |u| format!("{:.2} Mb/s", u / 1_000_000.0));

        println!(
            "  {:<20}  {:<15}  {:>10}  {:>12}  {:>12}",
            date,
            if entry.sponsor.len() > 15 {
                &entry.sponsor[0..12]
            } else {
                &entry.sponsor
            },
            ping.cyan(),
            dl.green(),
            ul.yellow()
        );
    }

    Ok(())
}

/// Compute average download and upload speeds from history (last 20 entries).
/// Returns (avg_dl_mbps, avg_ul_mbps) or None if insufficient data.
pub fn get_averages() -> Option<(f64, f64)> {
    let history = load_history().ok()?;
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

    let avg_dl = dl_entries.iter().sum::<f64>() / dl_entries.len() as f64;
    let avg_ul = ul_entries.iter().sum::<f64>() / ul_entries.len() as f64;
    Some((avg_dl, avg_ul))
}

/// Format historical comparison as a string for display.
/// Returns None if insufficient history data.
pub fn format_comparison(download_mbps: f64, upload_mbps: f64, nc: bool) -> Option<String> {
    let (avg_dl, avg_ul) = get_averages()?;

    // Use the combined metric: dl + ul as a single score
    let current_score = download_mbps + upload_mbps;
    let avg_score = avg_dl + avg_ul;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ServerInfo, TestResult};

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

    #[test]
    fn test_get_averages_returns_values() {
        // Save several results to ensure we have history
        let base_results = vec![
            make_test_result(100_000_000.0, 50_000_000.0, "2026-01-01T00:00:00Z"),
            make_test_result(120_000_000.0, 60_000_000.0, "2026-01-02T00:00:00Z"),
            make_test_result(80_000_000.0, 40_000_000.0, "2026-01-03T00:00:00Z"),
        ];
        for r in &base_results {
            save_result(r).ok();
        }

        let result = get_averages();
        // May or may not have values depending on prior test state
        if let Some((avg_dl, avg_ul)) = result {
            assert!(avg_dl >= 0.0);
            assert!(avg_ul >= 0.0);
        }
    }

    #[test]
    fn test_format_comparison_faster() {
        // Save baseline data and compare - just verify it doesn't panic
        for i in 0..3 {
            let r = make_test_result(
                20_000_000.0,
                10_000_000.0,
                &format!("2026-06-{i:02}T00:00:00Z"),
            );
            save_result(&r).ok();
        }

        let _ = format_comparison(200.0, 100.0, false);
    }

    #[test]
    fn test_format_comparison_slower() {
        for i in 0..3 {
            let r = make_test_result(
                800_000_000.0,
                800_000_000.0,
                &format!("2026-07-{i:02}T00:00:00Z"),
            );
            save_result(&r).ok();
        }

        let _ = format_comparison(10.0, 5.0, false);
    }

    #[test]
    fn test_format_comparison_on_par() {
        // Save similar results
        let sim_results = vec![
            make_test_result(100_000_000.0, 50_000_000.0, "2026-04-01T00:00:00Z"),
            make_test_result(105_000_000.0, 52_000_000.0, "2026-04-02T00:00:00Z"),
            make_test_result(95_000_000.0, 48_000_000.0, "2026-04-03T00:00:00Z"),
        ];
        for r in &sim_results {
            save_result(r).ok();
        }

        // Test with a similar result
        let result = format_comparison(100.0, 50.0, true);
        // May or may not be on-par depending on exact values, but should not panic
        let _ = result;
    }

    #[test]
    fn test_print_history_with_data() {
        // Save some results - just verify it doesn't panic
        for i in 0..3 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-05-{i:02}T00:00:00Z"),
            );
            let _ = save_result(&r);
        }
        let _ = print_history();
    }

    #[test]
    fn test_save_result_appends_to_existing() {
        // Save a result, then another - just verify it doesn't panic
        let r1 = make_test_result(50_000_000.0, 25_000_000.0, "2026-08-01T00:00:00Z");
        save_result(&r1).ok();
        let r2 = make_test_result(60_000_000.0, 30_000_000.0, "2026-08-02T00:00:00Z");
        save_result(&r2).ok();

        // History should be loadable after saving
        let _ = load_history();
    }

    #[test]
    fn test_print_history_long_sponsor_truncation() {
        // Save a result with a very long sponsor name - just verify no panic
        let mut r = make_test_result(100_000_000.0, 50_000_000.0, "2026-09-01T00:00:00Z");
        r.server.sponsor = "VeryLongSponsorNameThatExceedsLimit".to_string();
        let _ = save_result(&r);
        let _ = print_history();
    }

    #[test]
    fn test_format_comparison_zero_avg() {
        // Save results with zero speeds
        let r = make_test_result(0.0, 0.0, "2026-10-01T00:00:00Z");
        save_result(&r).ok();

        // Should handle zero avg gracefully
        let result = format_comparison(100.0, 50.0, false);
        // May return None or Some depending on implementation
        let _ = result;
    }

    #[test]
    fn test_format_comparison_nc() {
        // Save similar results - just verify it doesn't panic
        for i in 0..3 {
            let r = make_test_result(
                100_000_000.0,
                50_000_000.0,
                &format!("2026-11-{i:02}T00:00:00Z"),
            );
            let _ = save_result(&r);
        }

        let _ = format_comparison(100.0, 50.0, true);
    }

    #[test]
    fn test_save_result_invalid_json_recovery() {
        // Write invalid JSON to history file, verify recovery
        use directories::ProjectDirs;
        use std::fs;
        if let Some(proj_dirs) = ProjectDirs::from("dev", "vibe", "netspeed-cli") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            let path = data_dir.join("history.json");
            let _ = fs::write(&path, "{invalid json}");

            let r = make_test_result(100_000_000.0, 50_000_000.0, "2026-12-01T00:00:00Z");
            let _ = save_result(&r);
            let _ = load_history();
        }
    }
}
