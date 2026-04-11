//! Terminal progress bars and spinners for test feedback.
//!
//! This module provides user interface components for test progress:
//! - [`SpeedProgress`] — Indeterminate progress bar with real-time speed display
//! - Spinners for individual test phases (server discovery, ping, etc.)
//! - `NO_COLOR` environment variable support for disabling colored output
//! - `NO_EMOJI` environment variable support for disabling emoji characters
//! - Colorized finish messages with test results

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::common::format_data_size;
pub use crate::common::no_color;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use owo_colors::OwoColorize;

/// A progress tracker for download/upload tests.
/// Updates a single shared progress bar with live speed.
pub struct SpeedProgress {
    bar: ProgressBar,
    bytes: bool,
}

impl SpeedProgress {
    /// Create a new progress tracker for a test phase.
    /// `label` is something like "Download" or "Upload".
    /// `bytes` controls display units (true=MB/s, false=Mb/s).
    #[must_use]
    pub fn new(label: &str, bytes: bool) -> Self {
        Self::with_target(label, bytes, ProgressDrawTarget::stderr_with_hz(10))
    }

    /// Create with a custom draw target (use `ProgressDrawTarget::hidden()` for silent mode).
    /// Uses an indeterminate-style progress bar — no misleading percentage, just elapsed time,
    /// live speed, and total bytes.
    #[must_use]
    pub fn with_target(label: &str, bytes: bool, target: ProgressDrawTarget) -> Self {
        let bar = ProgressBar::with_draw_target(None, target); // None = indeterminate

        let nc = no_color();
        let no_emoji = crate::common::no_emoji();
        let style = if no_emoji {
            ProgressStyle::with_template("  {prefix} {spinner}  {elapsed_precise} | {msg}")
                .unwrap()
                .tick_strings(&["—", "\\", "|", "/"])
        } else {
            ProgressStyle::with_template("  {prefix} {spinner}  {elapsed_precise} | {msg}")
                .unwrap()
                .tick_strings(&["━", "╾", "━", "╾"])
        };

        bar.set_style(style);
        bar.set_prefix(if nc {
            format!("{:<10}", format!("{}:", label))
        } else {
            format!("{:<10}", format!("{label}:").dimmed())
        });
        bar.set_message("starting...");

        Self { bar, bytes }
    }

    /// Update the live speed and data display.
    /// `speed_mbps` is the current speed in Mb/s (or MB/s if bytes mode).
    /// `bytes` is total bytes transferred so far.
    pub fn update(&self, speed_mbps: f64, bytes: u64) {
        let unit = if self.bytes { "MB/s" } else { "Mb/s" };
        let speed_str = if speed_mbps < 1000.0 {
            format!("{speed_mbps:.1} {unit}")
        } else {
            let gunit = if self.bytes { "GB/s" } else { "Gb/s" };
            format!("{:.2} {gunit}", speed_mbps / 1000.0)
        };

        let data_str = format_data_size(bytes);

        let msg = if no_color() {
            format!("{data_str} @ {speed_str}")
        } else {
            format!("{} @ {}", data_str.white(), speed_str.cyan())
        };

        self.bar.set_message(msg);
    }

    /// Mark the test as complete and display final speed.
    pub fn finish(&self, final_speed_mbps: f64, total_bytes: u64) {
        let unit = if self.bytes { "MB/s" } else { "Mb/s" };
        let speed_str = if final_speed_mbps < 1000.0 {
            format!("{final_speed_mbps:.2} {unit}")
        } else {
            let gunit = if self.bytes { "GB/s" } else { "Gb/s" };
            format!("{:.2} {gunit}", final_speed_mbps / 1000.0)
        };

        let data_str = format_data_size(total_bytes);

        let msg = if no_color() {
            format!("DONE ({data_str} total @ {speed_str})")
        } else {
            format!(
                "{} ({} total @ {})",
                "DONE".green().bold(),
                data_str.dimmed(),
                speed_str.green()
            )
        };
        self.bar.finish_with_message(msg);
    }
}

/// Simple spinner for non-speed phases (server fetch, ping).
#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(10));
    let tick_chars = if crate::common::no_emoji() {
        &["|", "/", "-", "\\"][..]
    } else {
        &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][..]
    };
    pb.set_style(
        ProgressStyle::with_template("  {spinner} {msg}")
            .unwrap()
            .tick_strings(tick_chars),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}

/// Finish a simple spinner with a checkmark.
pub fn finish_ok(pb: &ProgressBar, message: &str) {
    if crate::common::no_emoji() {
        pb.finish_with_message(format!("  [OK] {message}"));
    } else {
        pb.finish_with_message(format!("  {} {}", "✓".green(), message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Safely set NO_COLOR env var
    fn set_no_color() {
        // SAFETY: Tests using this function are marked with #[serial] to prevent concurrent env access
        unsafe { std::env::set_var("NO_COLOR", "1") }
    }

    /// Safely remove NO_COLOR env var
    fn unset_no_color() {
        // SAFETY: Tests using this function are marked with #[serial] to prevent concurrent env access
        unsafe { std::env::remove_var("NO_COLOR") }
    }

    #[test]
    fn test_no_color_default() {
        // Note: This may return true if NO_COLOR is set by another test.
        // We just verify the function doesn't panic.
        let _ = no_color();
    }

    #[test]
    fn test_create_spinner() {
        let pb = create_spinner("Testing...");
        assert!(!pb.is_finished());
        pb.finish_and_clear();
    }

    #[test]
    fn test_finish_ok() {
        let pb = create_spinner("Testing...");
        finish_ok(&pb, "Done");
        assert!(pb.is_finished());
    }

    #[test]
    fn test_speed_progress_new() {
        let sp = SpeedProgress::new("Download", false);
        sp.bar.finish_and_clear();
    }

    #[test]
    fn test_speed_progress_update() {
        let sp = SpeedProgress::new("Download", false);
        sp.update(125.4, 5_000_000);
        sp.finish(125.40, 10_000_000);
    }

    #[test]
    #[serial]
    fn test_no_color_env_set() {
        set_no_color();
        assert!(no_color());
        unset_no_color();
    }

    #[test]
    #[serial]
    fn test_create_spinner_nc() {
        set_no_color();
        let pb = create_spinner("Testing...");
        assert!(!pb.is_finished());
        pb.finish_and_clear();
        unset_no_color();
    }

    #[test]
    #[serial]
    fn test_finish_ok_nc() {
        set_no_color();
        let pb = create_spinner("Testing...");
        finish_ok(&pb, "Done");
        assert!(pb.is_finished());
        unset_no_color();
    }

    #[test]
    #[serial]
    fn test_speed_progress_nc() {
        set_no_color();
        let sp = SpeedProgress::new("Download", false);
        sp.update(125.4, 5_000_000);
        sp.finish(125.40, 10_000_000);
        unset_no_color();
    }
}
