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
const UI_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgressLayout {
    Minimal,
    Compact,
    Standard,
}

impl ProgressLayout {
    fn from_terminal_width(width: usize) -> Self {
        match width {
            0..60 => Self::Minimal,
            60..100 => Self::Compact,
            _ => Self::Standard,
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Minimal => 0,
            Self::Compact => 1,
            Self::Standard => 2,
        }
    }

    const fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Minimal,
            1 => Self::Compact,
            _ => Self::Standard,
        }
    }
}

fn format_progress_rate(mbps: f64, bytes: bool) -> String {
    let value = if bytes { mbps / 8.0 } else { mbps };
    let (value, unit) = if bytes {
        if value < 1.0 {
            (value * 1000.0, "KB/s")
        } else if value < 1000.0 {
            (value, "MB/s")
        } else {
            (value / 1000.0, "GB/s")
        }
    } else if value < 1000.0 {
        (value, "Mb/s")
    } else {
        (value / 1000.0, "Gb/s")
    };
    if value < 10.0 {
        format!("{value:.2} {unit}")
    } else {
        format!("{value:.1} {unit}")
    }
}

fn bar_width_for_terminal(term_w: usize) -> usize {
    match ProgressLayout::from_terminal_width(term_w) {
        ProgressLayout::Minimal => 0,
        ProgressLayout::Compact => term_w.saturating_sub(39).clamp(8, 40),
        ProgressLayout::Standard => term_w.saturating_sub(72).clamp(8, 50),
    }
}

fn progress_style(layout: ProgressLayout, width: usize, animated: bool) -> ProgressStyle {
    let spinner = if animated { "{spinner} " } else { "" };
    let template = match layout {
        ProgressLayout::Minimal => format!("{{prefix}} {spinner}{{msg}}"),
        ProgressLayout::Compact => {
            format!("  {{prefix}} {spinner}{{bar:{width}}} {{percent:>3}}% {{msg}}")
        }
        ProgressLayout::Standard => format!(
            "  {{prefix}} {spinner}{{bar:{width}}} {{percent:>3}}%  {{elapsed_precise}} | {{msg}}"
        ),
    };
    let style = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("━░─");
    if animated {
        style.tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠏"])
    } else {
        style
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

fn progress_prefix(label: &str, upload: bool, layout: ProgressLayout) -> String {
    let arrow = if upload { "↑" } else { "↓" };
    let text = if layout == ProgressLayout::Minimal {
        format!("{arrow} {}", if upload { "UL" } else { "DL" })
    } else {
        format!("{arrow} {label}:")
    };
    let text = if layout == ProgressLayout::Minimal {
        text
    } else {
        format!("{text:<12}")
    };
    if terminal::no_color() {
        text
    } else {
        use owo_colors::OwoColorize;
        text.dimmed().to_string()
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
    theme: crate::theme::Theme,
    bytes: bool,
    updates_enabled: bool,
    upload: bool,
    animated: bool,
    bar_width: std::sync::atomic::AtomicUsize,
    layout: std::sync::atomic::AtomicU8,
    label: String,
    created_at: std::time::Instant,
    last_ui_update_ms: std::sync::atomic::AtomicU64,
    latest_display: Mutex<(f64, f64, u64)>,
    live: Option<crate::live::Handle>,
}

impl Tracker {
    pub(crate) fn warning(&self, message: &str) {
        if let Some(live) = &self.live {
            live.warning(message);
        } else {
            eprintln!("{}", crate::terminal::sanitize_display_text(message, 240));
        }
    }
    pub(crate) fn for_live(label: &str, handle: crate::live::Handle) -> Self {
        let mut tracker = Self::hidden(label);
        tracker.live = Some(handle);
        tracker
    }

    /// Publish the newest sample without terminal I/O or sample-vector work.
    pub fn publish(&self, speed_mbps: f64, progress: f64, bytes: u64) {
        if let Some(live) = &self.live {
            live.sample(speed_mbps, bytes);
            return;
        }

        *self
            .latest_display
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = (speed_mbps, progress, bytes);
    }

    /// Render the latest published sample. Called by the bounded UI worker.
    pub fn render_latest(&self) {
        let (speed, progress, bytes) = *self
            .latest_display
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        self.update(speed, progress, bytes);
    }

    pub fn mark_done(&self) {
        self.done.store(true, Ordering::Release);
    }

    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self::with_display(
            label,
            ProgressDrawTarget::stderr_with_hz(10),
            crate::theme::Theme::Dark,
            false,
        )
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
        Self::with_display(label, target, crate::theme::Theme::Dark, false)
    }

    pub fn with_display(
        label: &str,
        target: ProgressDrawTarget,
        theme: crate::theme::Theme,
        bytes: bool,
    ) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);
        let term_w = common::get_terminal_width().unwrap_or(100) as usize;
        let layout = ProgressLayout::from_terminal_width(term_w);
        let bw = bar_width_for_terminal(term_w);
        let upload = label.starts_with('U') || label.to_ascii_lowercase().contains("upload");

        let animated = !terminal::no_animation();
        bar.set_style(progress_style(layout, bw, animated));
        bar.set_prefix(progress_prefix(label, upload, layout));
        bar.set_message("starting...");
        bar.set_position(0);
        if animated {
            bar.enable_steady_tick(Duration::from_millis(100));
        }

        Self {
            bar,
            done,
            speed_samples: Mutex::new(Vec::new()),
            theme,
            bytes,
            updates_enabled: true,
            upload,
            animated,
            bar_width: std::sync::atomic::AtomicUsize::new(bw),
            layout: std::sync::atomic::AtomicU8::new(layout.code()),
            label: label.to_string(),
            created_at: std::time::Instant::now(),
            last_ui_update_ms: std::sync::atomic::AtomicU64::new(u64::MAX),
            latest_display: Mutex::new((0.0, 0.0, 0)),
            live: None,
        }
    }

    fn with_target_animated(label: &str, target: ProgressDrawTarget) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let bar = ProgressBar::with_draw_target(Some(100), target);
        let term_w = common::get_terminal_width().unwrap_or(100) as usize;
        let layout = ProgressLayout::from_terminal_width(term_w);
        let bw = bar_width_for_terminal(term_w);
        let upload = label.starts_with('U') || label.to_ascii_lowercase().contains("upload");

        bar.set_style(progress_style(layout, bw, true));
        bar.set_prefix(progress_prefix(label, upload, layout));
        bar.set_message("starting...");
        bar.set_position(0);
        bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            bar,
            done,
            speed_samples: Mutex::new(Vec::new()),
            theme: crate::theme::Theme::Dark,
            bytes: false,
            updates_enabled: true,
            upload,
            animated: true,
            bar_width: std::sync::atomic::AtomicUsize::new(bw),
            layout: std::sync::atomic::AtomicU8::new(layout.code()),
            label: label.to_string(),
            created_at: std::time::Instant::now(),
            last_ui_update_ms: std::sync::atomic::AtomicU64::new(u64::MAX),
            latest_display: Mutex::new((0.0, 0.0, 0)),
            live: None,
        }
    }

    pub fn update(&self, speed_mbps: f64, progress: f64, bytes: u64) {
        if !self.updates_enabled {
            return;
        }
        let now_ms = self.created_at.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let previous = self.last_ui_update_ms.load(Ordering::Relaxed);
        if previous != u64::MAX
            && now_ms.saturating_sub(previous) < UI_UPDATE_INTERVAL.as_millis() as u64
        {
            return;
        }
        if self
            .last_ui_update_ms
            .compare_exchange(previous, now_ms, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        let terminal_width = common::get_terminal_width().unwrap_or(100) as usize;
        let width = bar_width_for_terminal(terminal_width);
        let layout = ProgressLayout::from_terminal_width(terminal_width);
        let previous_width = self.bar_width.swap(width, Ordering::Relaxed);
        let previous_layout =
            ProgressLayout::from_code(self.layout.swap(layout.code(), Ordering::Relaxed));
        if width != previous_width || layout != previous_layout {
            self.bar
                .set_style(progress_style(layout, width, self.animated));
            self.bar
                .set_prefix(progress_prefix(&self.label, self.upload, layout));
        }
        let mut samples = self.speed_samples.lock().unwrap();
        samples.push(speed_mbps);
        if samples.len() > 60 {
            let drain = samples.len() - 60;
            samples.drain(..drain);
        }
        let sparkline = render_sparkline(&samples);
        let trend = speed_trend(&samples);
        let compact_chart: String = sparkline
            .chars()
            .rev()
            .take(5)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let pct = (progress * 100.0).clamp(0.0, u64::MAX as f64) as u64;
        let speed_text = format_progress_rate(speed_mbps, self.bytes);
        let speed_display = if terminal::no_color() {
            speed_text.clone()
        } else if self.upload {
            Colors::header(&speed_text, self.theme)
        } else {
            Colors::info(&speed_text, self.theme)
        };
        let msg = match layout {
            ProgressLayout::Minimal => {
                let chart = if compact_chart.is_empty() {
                    String::new()
                } else {
                    format!(" {compact_chart} {trend}")
                };
                format!("{speed_display}{chart} {pct:>3}%")
            }
            ProgressLayout::Compact => {
                if compact_chart.is_empty() {
                    speed_display
                } else {
                    format!("{speed_display} {compact_chart}")
                }
            }
            ProgressLayout::Standard if terminal::no_color() => {
                format!("{} @ {}", common::format_data_size(bytes), speed_text)
            }
            ProgressLayout::Standard => {
                let data_str = common::format_data_size(bytes);
                let speed_colored = if self.upload {
                    Colors::header(&speed_text, self.theme)
                } else {
                    Colors::info(&speed_text, self.theme)
                };
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
            }
        };

        self.bar.set_message(msg);
        self.bar.set_position(pct.min(100));
    }

    pub fn finish(&self, final_speed_mbps: f64, total_bytes: u64, theme: crate::theme::Theme) {
        let speed_str = format_progress_rate(final_speed_mbps, self.bytes);
        let data_str = common::format_data_size(total_bytes);
        let layout = ProgressLayout::from_code(self.layout.load(Ordering::Relaxed));
        if layout != ProgressLayout::Standard {
            let speed = if terminal::no_color() {
                speed_str
            } else if self.upload {
                Colors::header(&speed_str, theme)
            } else {
                Colors::info(&speed_str, theme)
            };
            let done = if terminal::no_color() {
                "DONE".to_string()
            } else {
                Colors::good("DONE", theme)
            };
            self.bar.finish_with_message(format!("{done} {speed}"));
            self.done.store(true, Ordering::Relaxed);
            return;
        }
        self.bar.set_position(100);
        let msg = if terminal::no_color() {
            format!("DONE ({data_str} total @ {speed_str})")
        } else {
            let speed = if self.upload {
                Colors::header(&speed_str, theme)
            } else {
                Colors::info(&speed_str, theme)
            };
            format!(
                "{} ({} total @ {})",
                Colors::good("DONE", theme),
                data_str.dimmed(),
                speed
            )
        };
        self.bar.finish_with_message(msg);
        self.done.store(true, Ordering::Relaxed);
    }

    pub fn hidden(label: &str) -> Self {
        let mut tracker = Self::with_target(label, ProgressDrawTarget::hidden());
        tracker.updates_enabled = false;
        tracker
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(10));
    let style = if terminal::no_animation() {
        ProgressStyle::with_template("  {msg}").unwrap()
    } else {
        ProgressStyle::with_template("  {spinner} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠏"])
    };
    pb.set_style(style);
    pb.set_message(message.to_string());
    if !terminal::no_animation() {
        pb.enable_steady_tick(std::time::Duration::from_millis(120));
    }
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
        eprintln!("  {label} → {grade_plain}");
    } else {
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
        eprintln!("  ◉ Scanned {sample_count} samples  Grade: {grade_plain}");
    } else {
        eprintln!(
            "  {}  Scanned {} samples  {}",
            Colors::good("◉", theme),
            Colors::bold(&sample_count.to_string(), theme),
            grade_badge,
        );
    }
}

pub fn reveal_pause() {
    // Kept for compatibility with callers; result presentation never blocks.
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
        assert!((1..=20).contains(&samples.len()));
        let sparkline = render_sparkline(&samples);
        if samples.len() >= 2 {
            assert_eq!(sparkline.chars().count(), SPARKLINE_LEN);
        } else {
            assert!(sparkline.is_empty());
        }
        sp.bar.finish_and_clear();
    }

    #[test]
    fn rapid_updates_are_coalesced_for_display_work() {
        let tracker = Tracker::with_target("Download", ProgressDrawTarget::hidden());
        for i in 1..=100 {
            tracker.update(i as f64, i as f64 / 100.0, i);
        }
        let samples = tracker.speed_samples.lock().unwrap();
        assert_eq!(samples.len(), 1);
        assert_eq!(tracker.bar.position(), 1);
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
    fn progress_rate_respects_byte_display() {
        assert_eq!(format_progress_rate(100.0, true), "12.5 MB/s");
        assert_eq!(format_progress_rate(100.0, false), "100.0 Mb/s");
    }

    #[test]
    fn test_adaptive_bar_width() {
        assert_eq!(bar_width_for_terminal(40), 0);
        assert_eq!(bar_width_for_terminal(60), 21);
        assert_eq!(bar_width_for_terminal(80), 40);
        assert_eq!(bar_width_for_terminal(120), 48);
    }

    #[test]
    fn bar_width_tracks_terminal_size() {
        assert_eq!(
            ProgressLayout::from_terminal_width(40),
            ProgressLayout::Minimal
        );
        assert_eq!(
            ProgressLayout::from_terminal_width(60),
            ProgressLayout::Compact
        );
        assert_eq!(
            ProgressLayout::from_terminal_width(99),
            ProgressLayout::Compact
        );
        assert_eq!(
            ProgressLayout::from_terminal_width(100),
            ProgressLayout::Standard
        );
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
