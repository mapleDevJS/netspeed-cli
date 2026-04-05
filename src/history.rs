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
