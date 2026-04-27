//! Dashboard output formatting.
//!
//! This is a placeholder module for dashboard output.

use crate::error::Error;
use crate::profiles::UserProfile;
use crate::theme::Theme;
use crate::types::TestResult;

pub struct Summary {
    pub dl_mbps: f64,
    pub dl_peak_mbps: f64,
    pub dl_bytes: u64,
    pub dl_duration: f64,
    pub ul_mbps: f64,
    pub ul_peak_mbps: f64,
    pub ul_bytes: u64,
    pub ul_duration: f64,
    pub elapsed: std::time::Duration,
    pub profile: UserProfile,
    pub theme: Theme,
}

pub fn show(_result: &TestResult, _summary: &Summary) -> Result<(), Error> {
    // Dashboard output disabled - use detailed or compact format instead
    eprintln!("Note: Dashboard format is temporarily unavailable. Use --format detailed or --format compact.");
    Ok(())
}
