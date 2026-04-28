//! Terminal progress bars and spinners for test feedback.
//!
//! This module provides user interface components for test progress:
//! - [`Tracker`] — Progress bar with real-time speed display and sparkline
//! - Spinners for individual test phases (server discovery, ping, etc.)
//! - Colorized finish messages with test results
//! - Grade reveal animation for intentional friction

use crate::common;
use crate::terminal;
use crate::theme::Colors;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use owo_colors::OwoColorize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SPARKLINE_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const SPARKLINE_LEN: usize = 12;
const MIN_BAR_WIDTH: usize = 20;

fn adaptive_bar_width() -> usize {
    let term_w = common::get_terminal_width().unwrap_or(100) as usize;
    let available = term_w.saturating_sub(52);
    available.clamp(MIN_BAR_WIDTH, 50)
}

fn speed_color(speed_mbps: f64, theme: crate::theme::Theme) -> String {
    if speed_mbps >= 100.0 {
        Colors::good(&format!("{speed_mbps:.1} Mb/s"), theme)
    } else if speed_mbps >= 25.0 {
        Colors::info(&format!("{speed_mbps:.1} Mb/s"), theme)
    } else if speed_mbps >= 5.0 {
        Colors::warn(&format!("{speed_mbps:.1} Mb/s"), theme)
    } else {
        Colors::bad(&format!("{speed_mbps:.1} Mb/s"), theme)
    }
}

fn speed_trend(samples: &[f64]) -> &'static str {
    if samples.len() < 6 {
        return "→";
    }
    let n = samples.len();
    let recent_count = 2;
    let older_count = 3;
    let recent_avg: f64 =
        samples[n - recent_count..].iter().copied().sum::<f64>() / recent_count as f64;
    let older_avg: f64 = samples[n - recent_count - older_count..n - recent_count]
        .iter()
        .copied()
        .sum::<f64>()
        / older_count as f64;
    let ratio = recent_avg / older_avg.max(0.01);
    if ratio > 1.05 {
        "↑"
    } else if ratio < 0.95 {
        "↓"
    } else {
        "→"
    }
}

fn render_sparkline(samples: &[f64]) -> String {
    if samples.len() < 2 {
        return String::new();
    }
    let min = samples.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let max = samples.iter().cloned().reduce(f64::max).unwrap_or(1.0);
    let range = max - min;
    if range < 0.001 {
        return SPARKLINE_CHARS[4].to_string().repeat(SPARKLINE_LEN);
    }
    let step = if samples.len() > SPARKLINE_LEN {
        samples.len() / SPARKLINE_LEN
    } else {
        1
    };
    let sampled: Vec<f64> = (0..SPARKLINE_LEN)
        .map(|i| {
            let idx = ((i * step) + (step / 2)).min(samples.len() - 1);
            samples[idx]
        })
        .collect();
    sampled
        .iter()
        .map(|s| {
            let norm = ((s - min) / range).clamp(0.0, 1.0);
            let idx = (norm * (SPARKLINE_CHARS.len() - 1) as f64).round() as usize;
            SPARKLINE_CHARS[idx.clamp(0, SPARKLINE_CHARS.len() - 1)]
        })
        .collect()
}

/// A progress tracker for download/upload tests.
/// Updates a single shared progress bar with live speed, sparkline, and trend.
pub struct Tracker {
    bar: ProgressBar,
    done: Arc<AtomicBool>,
    speed_samples: Mutex<Vec<f64>>,
}

// SAFETY: Tracker is only used from a single async task (download/upload loop)
// with shared reference through Arc. The internal Mutex protects speed_samples.
unsafe impl Send for Tracker {}
unsafe impl Sync for Tracker {}

impl Tracker {
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self::with_target(label, ProgressDrawTarget::stderr_with_hz(10))
    }

    #[must_use]
    pub fn new_animated(label: &str) -> Self {
        if terminal::no_animation() {
            return Self::new(label);
        }
        Self::with_target_animated(label, ProgressDrawTarget::stderr_with_hz(10))
    }

    #[must_use]
    pub fn with_target(label: &str, target: ProgressDrawTarget) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);
        let bw = adaptive_bar_width();

        let tmpl = format!(
            "  {{prefix}} {{bar:{bw}.cyan/blue}} {{percent:>3}}%  {{elapsed_precise}} | {{msg}}"
        );
        let style = ProgressStyle::with_template(&tmpl)
            .unwrap()
            .progress_chars("━░─");

        bar.set_style(style);
        let arrow = if label.starts_with('D') {
            "↓ "
        } else if label.starts_with('U') {
            "↑ "
        } else {
            "  "
        };
        bar.set_prefix(if terminal::no_color() {
            format!("{:<12}", format!("{arrow}{label}:"))
        } else {
            format!("{:<12}", format!("{arrow}{label}:").dimmed())
        });
        bar.set_message("starting...");
        bar.set_position(0);

        Self {
            bar,
            done,
            speed_samples: Mutex::new(Vec::new()),
        }
    }

    fn with_target_animated(label: &str, target: ProgressDrawTarget) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);
        let bw = adaptive_bar_width();

        let tmpl = format!(
            "  {{prefix}} {{spinner}} {{bar:{bw}.cyan/blue}} {{percent:>3}}%  {{elapsed_precise}} | {{msg}}"
        );
        let style = ProgressStyle::with_template(&tmpl)
            .unwrap()
            .progress_chars("━░─")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠏"]);

        bar.set_style(style);
        let arrow = if label.starts_with('D') {
            "↓ "
        } else if label.starts_with('U') {
            "↑ "
        } else {
            "  "
        };
        bar.set_prefix(if terminal::no_color() {
            format!("{:<12}", format!("{arrow}{label}:"))
        } else {
            format!("{:<12}", format!("{arrow}{label}:").dimmed())
        });
        bar.set_message("starting...");
        bar.set_position(0);
        bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            bar,
            done,
            speed_samples: Mutex::new(Vec::new()),
        }
    }

    pub fn update(&self, speed_mbps: f64, progress: f64, bytes: u64) {
        {
            let mut samples = self.speed_samples.lock().unwrap();
            samples.push(speed_mbps);
            if samples.len() > 60 {
                let drain = samples.len() - 60;
                samples.drain(..drain);
            }
        }

        let data_str = common::format_data_size(bytes);

        let msg = if terminal::no_color() {
            let speed_str = format!("{speed_mbps:.1} Mb/s");
            format!("{data_str} @ {speed_str}")
        } else {
            let speed_colored = speed_color(speed_mbps, crate::theme::Theme::Dark);
            let samples = self.speed_samples.lock().unwrap();
            let sparkline = render_sparkline(&samples);
            let trend = speed_trend(&samples);
            if sparkline.is_empty() {
                format!("{} @ {}", data_str.white(), speed_colored)
            } else {
                format!(
                    "{} {} {} @ {}",
                    data_str.white(),
                    sparkline.dimmed(),
                    trend,
                    speed_colored
                )
            }
        };

        self.bar.set_message(msg);
        let pct = (progress * 100.0).clamp(0.0, u64::MAX as f64) as u64;
        self.bar.set_position(pct.min(100));
    }

    pub fn finish(&self, final_speed_mbps: f64, total_bytes: u64, theme: crate::theme::Theme) {
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
                Colors::good("DONE", theme),
                data_str.dimmed(),
                Colors::good(&speed_str, theme)
            )
        };
        self.bar.finish_with_message(msg);
        self.done.store(true, Ordering::Relaxed);
    }
}

#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(10));
    pb.set_style(
        ProgressStyle::with_template("  {spinner} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}

pub fn finish_ok(pb: &ProgressBar, message: &str, theme: crate::theme::Theme) {
    if terminal::no_color() {
        pb.finish_with_message(format!("  {message}"));
    } else {
        pb.finish_with_message(format!("  {} {}", Colors::good("◉", theme), message));
    }
}

// ── Grade Reveal Animation (Intentional Friction) ────────────────────────────

pub fn reveal_grade(label: &str, grade_str: &str, grade_plain: &str, nc: bool) {
    if nc {
        std::thread::sleep(Duration::from_millis(300));
        eprintln!("  {label} → {grade_plain}");
    } else {
        let spinner = create_spinner(&format!("Computing {label}..."));
        std::thread::sleep(Duration::from_millis(400));
        spinner.finish_and_clear();
        eprintln!("  {label} → {grade_str}");
    }
}

pub fn reveal_scan_complete(
    sample_count: usize,
    grade_badge: &str,
    grade_plain: &str,
    nc: bool,
    theme: crate::theme::Theme,
) {
    if terminal::no_animation() {
        eprintln!("  ◉ Scanned {sample_count} samples  {grade_plain}");
    } else if nc {
        std::thread::sleep(Duration::from_millis(100));
        eprintln!("  ◉ Scanned {sample_count} samples  Grade: {grade_plain}");
    } else {
        std::thread::sleep(Duration::from_millis(100));
        eprintln!(
            "  {}  Scanned {} samples  {}",
            Colors::good("◉", theme),
            Colors::bold(&sample_count.to_string(), theme),
            grade_badge,
        );
    }
}

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

    fn set_no_color() {
        // SAFETY: tests in this module run serially via #[serial]; no concurrent set_var calls.
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    fn unset_no_color() {
        // SAFETY: tests in this module run serially via #[serial]; no concurrent remove_var calls.
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn test_no_color_default() {
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
        finish_ok(&pb, "Done", crate::theme::Theme::Dark);
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
    fn test_speed_progress_with_sparkline() {
        let sp = Tracker::new("Download");
        for i in 1..=20 {
            sp.update(50.0 + i as f64 * 2.0, i as f64 / 20.0, 1024 * 1024);
        }
        let samples = sp.speed_samples.lock().unwrap();
        assert_eq!(samples.len(), 20);
        let sparkline = render_sparkline(&samples);
        assert!(!sparkline.is_empty());
        assert_eq!(sparkline.chars().count(), SPARKLINE_LEN);
        sp.bar.finish_and_clear();
    }

    #[test]
    fn test_speed_progress_nc() {
        set_no_color();
        let sp = Tracker::new("Upload");
        sp.update(50.0, 0.25, 512 * 1024);
        assert_eq!(sp.bar.position(), 25);
        sp.finish(50.0, 1024 * 1024, crate::theme::Theme::Dark);
        assert!(sp.done.load(Ordering::Relaxed));
        unset_no_color();
    }

    #[test]
    fn test_speed_trend_up() {
        let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        assert_eq!(speed_trend(&samples), "↑");
    }

    #[test]
    fn test_speed_trend_down() {
        let samples = vec![60.0, 50.0, 40.0, 30.0, 20.0, 10.0];
        assert_eq!(speed_trend(&samples), "↓");
    }

    #[test]
    fn test_speed_trend_stable() {
        let samples = vec![50.0, 51.0, 49.0, 50.0, 51.0, 50.0];
        assert_eq!(speed_trend(&samples), "→");
    }

    #[test]
    fn test_speed_trend_few_samples() {
        assert_eq!(speed_trend(&[10.0, 20.0]), "→");
    }

    #[test]
    fn test_render_sparkline_basic() {
        let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0];
        let sparkline = render_sparkline(&samples);
        assert_eq!(sparkline.chars().count(), SPARKLINE_LEN);
    }

    #[test]
    fn test_render_sparkline_flat() {
        let samples = vec![50.0; 10];
        let sparkline = render_sparkline(&samples);
        assert_eq!(sparkline.chars().count(), SPARKLINE_LEN);
        let chars: Vec<char> = sparkline.chars().collect();
        assert!(chars.windows(2).all(|w| w[0] == w[1]));
    }

    #[test]
    fn test_render_sparkline_empty() {
        let sparkline = render_sparkline(&[]);
        assert!(sparkline.is_empty());
    }

    #[test]
    fn test_render_sparkline_single() {
        let sparkline = render_sparkline(&[42.0]);
        assert!(sparkline.is_empty());
    }

    #[test]
    fn test_speed_color_good() {
        let colored = speed_color(150.0, crate::theme::Theme::Dark);
        assert!(colored.contains("150.0"));
    }

    #[test]
    fn test_speed_color_warn() {
        let colored = speed_color(10.0, crate::theme::Theme::Dark);
        assert!(colored.contains("10.0"));
    }

    #[test]
    fn test_speed_color_bad() {
        let colored = speed_color(2.0, crate::theme::Theme::Dark);
        assert!(colored.contains("2.0"));
    }

    #[test]
    fn test_adaptive_bar_width() {
        let w = adaptive_bar_width();
        assert!(w >= MIN_BAR_WIDTH);
        assert!(w <= 50);
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
        finish_ok(&pb, "Done", crate::theme::Theme::Dark);
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
        reveal_scan_complete(42, "B+", "B+", true, crate::theme::Theme::Dark);
        unset_no_color();
    }

    #[test]
    fn test_reveal_pause() {
        reveal_pause();
    }
}
