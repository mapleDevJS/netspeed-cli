//! Terminal progress bars and spinners for test feedback.
//!
//! This module provides user interface components for test progress:
//! - [`Tracker`] — Progress bar with real-time speed display
//! - Spinners for individual test phases (server discovery, ping, etc.)
//! - Colorized finish messages with test results
//! - Grade reveal animation for intentional friction
//!
//! ## Note
//!
//! Terminal environment detection ([`crate::terminal::no_color`], [`crate::terminal::no_emoji`], [`crate::terminal::no_animation`])
//! has been moved to the [`crate::terminal`] module.

use crate::common;
use crate::terminal;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use owo_colors::OwoColorize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// A progress tracker for download/upload tests.
/// Updates a single shared progress bar with live speed.
pub struct Tracker {
    bar: ProgressBar,
    done: Arc<AtomicBool>,
}

impl Tracker {
    /// Create a new progress tracker for a test phase.
    /// `label` is something like "Download" or "Upload".
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self::with_target(label, ProgressDrawTarget::stderr_with_hz(10))
    }

    /// Create an animated progress tracker with "marching ants" effect.
    /// Uses moving bar pattern for visual engagement.
    /// Falls back to static bar if terminal::no_animation() is set.
    #[must_use]
    pub fn new_animated(label: &str) -> Self {
        if terminal::no_animation() {
            return Self::new(label);
        }
        Self::with_target_animated(label, ProgressDrawTarget::stderr_with_hz(10))
    }

    /// Create with a custom draw target (use `ProgressDrawTarget::hidden()` for silent mode).
    ///
    /// # Panics
    ///
    /// Panics if the progress bar template string is invalid (should never happen).
    #[must_use]
    pub fn with_target(label: &str, target: ProgressDrawTarget) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);

        let style = ProgressStyle::with_template(
            "  {prefix} {bar:40.cyan/blue} {percent:>3}%  {elapsed_precise} | {msg}",
        )
        .unwrap()
        .progress_chars("█░─");

        bar.set_style(style);
        bar.set_prefix(if terminal::no_color() {
            format!("{:<10}", format!("{}:", label))
        } else {
            format!("{:<10}", format!("{label}:").dimmed())
        });
        bar.set_message("starting...");
        bar.set_position(0);

        Self { bar, done }
    }

    /// Create animated tracker with custom draw target.
    /// Uses "marching ants" moving bar pattern and spinner for continuous motion.
    fn with_target_animated(label: &str, target: ProgressDrawTarget) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);

        let style = ProgressStyle::with_template(
            "  {prefix} {spinner} {bar:40.cyan/blue} {percent:>3}%  {elapsed_precise} | {msg}",
        )
        .unwrap()
        .progress_chars("▓▒░▒▓▒░▒▓")
        .tick_strings(&["▸", "▹", "►", "▻", "▼", "▽", "▾", "▿"]);

        bar.set_style(style);
        bar.set_prefix(if terminal::no_color() {
            format!("{:<10}", format!("{}:", label))
        } else {
            format!("{::<10}", format!("{label}:").dimmed())
        });
        bar.set_message("starting...");
        bar.set_position(0);
        bar.enable_steady_tick(Duration::from_millis(100));

        Self { bar, done }
    }

    /// Update the live speed and data display.
    /// `speed_mbps` is the current speed in Mb/s (or MB/s if bytes mode).
    /// `progress` is 0.0 to 1.0.
    /// `bytes` is total bytes transferred so far.
    pub fn update(&self, speed_mbps: f64, progress: f64, bytes: u64) {
        let speed_str = if speed_mbps < 1000.0 {
            format!("{speed_mbps:.1} Mb/s")
        } else {
            format!("{:.2} Gb/s", speed_mbps / 1000.0)
        };

        let data_str = common::format_data_size(bytes);

        let msg = if terminal::no_color() {
            format!("{data_str} @ {speed_str}")
        } else {
            format!("{} @ {}", data_str.white(), speed_str.cyan())
        };

        self.bar.set_message(msg);
        // Safe: progress is 0.0..1.0, *100 → 0..100, fits u64.
        let pct = (progress * 100.0).clamp(0.0, u64::MAX as f64) as u64;
        self.bar.set_position(pct.min(100));
    }

    /// Mark the test as complete and display final speed.
    pub fn finish(&self, final_speed_mbps: f64, total_bytes: u64) {
        let speed_str = if final_speed_mbps < 1000.0 {
            format!("{final_speed_mbps:.2} Mb/s")
        } else {
            format!("{:.2} Gb/s", final_speed_mbps / 1000.0)
        };

        let data_str = common::format_data_size(total_bytes);

        self.bar.set_position(100);
        let msg = if terminal::no_color() {
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
        self.done.store(true, Ordering::Relaxed);
    }
}

/// Simple spinner for non-speed phases (server fetch, ping).
///
/// # Panics
///
/// Panics if the spinner template string is invalid (should never happen).
#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(10));
    pb.set_style(
        ProgressStyle::with_template("  {spinner} {msg}")
            .unwrap()
            .tick_strings(&["·", "o", "O", "o"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}

/// Finish a simple spinner with a checkmark.
pub fn finish_ok(pb: &ProgressBar, message: &str) {
    if terminal::no_color() {
        pb.finish_with_message(format!("  {message}"));
    } else {
        pb.finish_with_message(format!("  {} {}", "✓".green(), message));
    }
}

// ── Grade Reveal Animation (Intentional Friction) ────────────────────────────

/// Animate a grade reveal with a brief "computing" pause followed by the final grade.
/// Creates intentional friction — the user anticipates the result before it appears.
///
/// # Arguments
/// * `label` — The metric being graded (e.g., "Overall", "Latency")
/// * `grade_str` — The final grade string (already colorized if needed)
/// * `grade_plain` — The plain grade string for no-color mode
/// * `nc` — No-color mode flag
pub fn reveal_grade(label: &str, grade_str: &str, grade_plain: &str, nc: bool) {
    if nc {
        // Brief pause for friction, then show result
        std::thread::sleep(Duration::from_millis(300));
        eprintln!("  {} → {grade_plain}", label.dimmed());
    } else {
        // Show a brief "computing" spinner
        let spinner = create_spinner(&format!("Computing {label}..."));
        std::thread::sleep(Duration::from_millis(400));
        spinner.finish_and_clear();
        eprintln!("  {label} → {grade_str}");
    }
}

/// Animate a scan completion summary before revealing results.
/// Shows total samples collected and overall grade.
///
/// # Arguments
/// * `sample_count` — Number of samples collected
/// * `grade_badge` — Colorized grade badge
/// * `grade_plain` — Plain grade text for no-color mode
/// * `nc` — No-color mode flag
pub fn reveal_scan_complete(sample_count: usize, grade_badge: &str, grade_plain: &str, nc: bool) {
    if terminal::no_animation() {
        // Skip all animation for users who prefer reduced motion
        eprintln!("  SCAN COMPLETE ✓ Scanned {sample_count} samples → {grade_plain}");
    } else if nc {
        std::thread::sleep(Duration::from_millis(100));
        eprintln!(
            "  {} ✓ Scanned {sample_count} samples → Grade: {grade_plain}",
            "SCAN COMPLETE".bold()
        );
    } else {
        // Brief pause for dramatic effect
        std::thread::sleep(Duration::from_millis(100));
        eprintln!(
            "  {} {} Scanned {} samples → {}",
            "SCAN COMPLETE".cyan().bold(),
            "✓".green(),
            sample_count.to_string().white().bold(),
            grade_badge,
        );
    }
}

/// Brief pause between section reveals for visual breathing room.
pub fn reveal_pause() {
    if terminal::no_animation() {
        return;
    }
    std::thread::sleep(Duration::from_millis(40));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Set `NO_COLOR` env var for testing.
    ///
    /// SAFETY: Tests run single-threaded under `#[serial]`, so there is no
    /// concurrent read/write race on the environment variable. The var is
    /// removed in `unset_no_color()` after each test to avoid leaking state.
    fn set_no_color() {
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    /// Remove `NO_COLOR` env var after testing.
    ///
    /// SAFETY: Same rationale as `set_no_color()` — serial test execution
    /// guarantees no concurrent env access.
    fn unset_no_color() {
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn test_no_color_default() {
        // Note: This may return true if NO_COLOR is set by another test.
        // We just verify the function doesn't panic.
        let _ = terminal::no_color();
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
        let sp = Tracker::new("Download");
        assert!(!sp.done.load(Ordering::Relaxed));
        sp.bar.finish_and_clear();
    }

    #[test]
    fn test_speed_progress_update() {
        let sp = Tracker::new("Download");
        sp.update(150.5, 0.5, 1024 * 1024);
        assert_eq!(sp.bar.position(), 50);
        sp.bar.finish_and_clear();
    }

    #[test]
    fn test_speed_progress_nc() {
        set_no_color();
        let sp = Tracker::new("Upload");
        sp.update(50.0, 0.25, 512 * 1024);
        assert_eq!(sp.bar.position(), 25);
        sp.finish(50.0, 1024 * 1024);
        assert!(sp.done.load(Ordering::Relaxed));
        unset_no_color();
    }

    #[test]
    #[serial]
    fn test_no_color_env_set() {
        set_no_color();
        assert!(terminal::no_color());
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
    fn test_reveal_grade_nc() {
        set_no_color();
        reveal_grade("Overall", "A", "A", true);
        unset_no_color();
    }

    #[test]
    #[serial]
    fn test_reveal_scan_complete_nc() {
        set_no_color();
        reveal_scan_complete(42, "B+", "B+", true);
        unset_no_color();
    }

    #[test]
    fn test_reveal_pause() {
        // Just verify it doesn't panic
        reveal_pause();
    }
}
