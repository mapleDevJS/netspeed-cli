//! Dashboard output formatting — rich single-screen summary with gauges and sparklines.

use crate::common;
use crate::error::Error;
use crate::formatter::SkipState;
use crate::grades::{self, LetterGrade};
use crate::profiles::UserProfile;
use crate::terminal;
use crate::theme::{Colors, Theme};
use crate::types::TestResult;
use owo_colors::OwoColorize;

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
    pub bytes: bool,
    pub skipped: SkipState,
}

/// Round up to the nearest visually clean scale breakpoint (Mb/s).
fn gauge_scale(peak_mbps: f64) -> f64 {
    const BREAKPOINTS: &[f64] = &[50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0];
    BREAKPOINTS
        .iter()
        .copied()
        .find(|&b| b >= peak_mbps * 1.1)
        .unwrap_or(peak_mbps * 1.1)
}

fn render_sparkline_from_samples(samples: &[f64], width: usize) -> String {
    let samples: Vec<f64> = samples
        .iter()
        .copied()
        .filter(|sample| sample.is_finite() && *sample >= 0.0)
        .collect();
    if samples.len() < 2 || width < 2 {
        return String::new();
    }
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let len = width.min(samples.len());
    let min = samples.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let max = samples.iter().cloned().reduce(f64::max).unwrap_or(1.0);
    let range = max - min;
    if range < 0.001 {
        return chars[4].to_string().repeat(len);
    }
    let step = if samples.len() > len {
        samples.len() / len
    } else {
        1
    };
    (0..len)
        .map(|i| {
            let idx = ((i * step) + (step / 2)).min(samples.len() - 1);
            let norm = ((samples[idx] - min) / range).clamp(0.0, 1.0);
            let ci = (norm * (chars.len() - 1) as f64).round() as usize;
            chars[ci.clamp(0, chars.len() - 1)]
        })
        .collect()
}

fn speed_trend(samples: &[f64]) -> &'static str {
    let samples: Vec<f64> = samples
        .iter()
        .copied()
        .filter(|sample| sample.is_finite() && *sample >= 0.0)
        .collect();
    if samples.len() < 6 {
        return "→";
    }
    let n = samples.len();
    let recent: f64 = samples[n - 2..].iter().copied().sum::<f64>() / 2.0;
    let older: f64 = samples[n - 5..n - 2].iter().copied().sum::<f64>() / 3.0;
    let ratio = recent / older.max(0.01);
    if ratio > 1.05 {
        "↑"
    } else if ratio < 0.95 {
        "↓"
    } else {
        "→"
    }
}

/// Renders the boxed header panel with grade, score, and quality description.
pub fn boxed_header(grade: &LetterGrade, nc: bool, theme: Theme, term_w: usize) -> String {
    let box_w = term_w.min(80);
    if box_w < 24 {
        return console::truncate_str(
            &format!("NETSPEED {} · {}", grade.as_str(), grade.score() as u32),
            box_w,
            "",
        )
        .into_owned();
    }
    // inner_w: space between │ chars (box_w - 2 leading spaces - 2 border chars)
    let inner_w = box_w.saturating_sub(4);
    // content_w: usable space inside padding (inner_w - 2 leading spaces - 2 trailing spaces)
    let content_w = inner_w.saturating_sub(4);

    let score = grade.score() as u32;
    let grade_str = grade.as_str();
    let right_text = format!("{grade_str} · {score}");
    let right_vis = console::measure_text_width(&right_text);
    // Reserve the actual grade width and a gap before truncating the label.
    // A+ and a three-digit score need more room than A and a two-digit score.
    let label = format!("◉ NETSPEED  ·  {}", grade.description());
    let left_text = console::truncate_str(&label, content_w.saturating_sub(right_vis + 2), "…");
    let left_vis = console::measure_text_width(&left_text);
    let padding = content_w.saturating_sub(left_vis + right_vis);

    let top = format!("  ┌{}┐", "─".repeat(inner_w));
    let bot = format!("  └{}┘", "─".repeat(inner_w));

    if nc {
        let mid = format!("  │  {left_text}{}{}  │", " ".repeat(padding), right_text);
        format!("{top}\n{mid}\n{bot}")
    } else {
        let left_col = Colors::bold(&left_text, theme);
        let right_col = format!(
            "{}{}",
            grade.color_str(nc, theme),
            Colors::dimmed(&format!(" · {score}"), theme),
        );
        let top_col = format!("  ┌{}┐", "─".repeat(inner_w).dimmed());
        let bot_col = format!("  └{}┘", "─".repeat(inner_w).dimmed());
        let mid = format!("  │  {left_col}{}{}  │", " ".repeat(padding), right_col);
        format!("{top_col}\n{mid}\n{bot_col}")
    }
}

/// Two-line speed block: gauge + hero value on line 1, sparkline + trend on line 2.
#[allow(
    clippy::too_many_arguments,
    reason = "keeps the pure rendering inputs explicit"
)]
fn speed_block(
    dir: &str,
    label: &str,
    speed_mbps: f64,
    peak_mbps: Option<f64>,
    max_mbps: f64,
    theme: Theme,
    gauge_w: usize,
    sparkline: &str,
    trend: &str,
    nc: bool,
    bytes: bool,
) -> String {
    let pct = (speed_mbps / max_mbps).clamp(0.0, 1.0);
    let filled = (pct * gauge_w as f64).round() as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(gauge_w.saturating_sub(filled))
    );

    let speed_str = format_rate(speed_mbps, bytes);
    let peak_str = peak_mbps
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(|value| format!("peak {}", format_rate(value, bytes)));

    if gauge_w == 0 {
        let first = format!("  {dir}  {label}: {speed_str}");
        let mut lines = vec![first];
        if !sparkline.is_empty() {
            lines.push(format!("     {sparkline}  {trend}"));
        }
        if let Some(peak_str) = peak_str {
            lines.push(format!("     {peak_str}"));
        }
        return lines.join("\n");
    }

    // Prefix before the bar: "  {dir(1)}  {label(10)}  " = 17 visible chars.
    // Compute from plain text so the indent is the same in both nc and color paths.
    let indent = " ".repeat(2 + dir.chars().count() + 2 + 10 + 2);

    if nc {
        let peak = peak_str
            .as_deref()
            .map(|text| format!("   {text}"))
            .unwrap_or_default();
        let line1 = format!("  {dir}  {label:<10}  {bar}  {speed_str}{peak}");
        if sparkline.is_empty() {
            line1
        } else {
            format!("{line1}\n{indent}{sparkline}   {trend}")
        }
    } else {
        let bar_col = Colors::muted(&bar, theme);
        let speed_col = if dir == "↓" {
            Colors::info(&speed_str, theme)
        } else {
            Colors::header(&speed_str, theme)
        };
        let dir_col = Colors::muted(dir, theme);
        // Pad the plain label to 10 visible chars BEFORE colorizing so ANSI bytes
        // don't confuse Rust's format-string width specifier.
        let label_col = Colors::dimmed(&format!("{label:<10}"), theme);
        let peak_col = peak_str
            .as_deref()
            .map(|text| format!("   {}", Colors::dimmed(text, theme)))
            .unwrap_or_default();
        let trend_col = Colors::dimmed(trend, theme);

        let line1 = format!("  {dir_col}  {label_col}  {bar_col}  {speed_col}{peak_col}");

        if sparkline.is_empty() {
            line1
        } else {
            let spark_col = if dir == "↓" {
                Colors::info(sparkline, theme)
            } else {
                Colors::header(sparkline, theme)
            };
            format!("{line1}\n{indent}{spark_col}   {trend_col}")
        }
    }
}

fn format_rate(mbps: f64, bytes: bool) -> String {
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

/// Combined single-line latency + jitter + packet loss row.
fn latency_row(
    ping_ms: f64,
    jitter: Option<f64>,
    packet_loss: Option<f64>,
    nc: bool,
    theme: Theme,
    term_w: usize,
) -> String {
    let mut metrics = vec![format!("Latency {ping_ms:.1} ms")];
    if let Some(jitter) = jitter.filter(|v| v.is_finite() && *v >= 0.0) {
        metrics.push(format!("Jitter {jitter:.1} ms"));
    }
    if let Some(loss) = packet_loss.filter(|v| v.is_finite() && *v >= 0.0) {
        metrics.push(format!("Packet loss {loss:.1}%"));
    }
    if term_w < 72 {
        return metrics
            .into_iter()
            .map(|metric| format!("  {metric}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    let content = metrics.join("   ·   ");
    if nc {
        format!("  ◈  {content}")
    } else {
        format!(
            "  {}  {}",
            Colors::muted("◈", theme),
            Colors::info(&content, theme)
        )
    }
}

/// Dashed thin separator line.
fn thin_separator(w: usize, nc: bool, theme: Theme) -> String {
    let line = "╌".repeat(w.saturating_sub(4).min(78));
    if nc {
        format!("  {line}")
    } else {
        format!("  {}", Colors::dimmed(&line, theme))
    }
}

/// Inline connection info lines (server + client).
fn sanitize_metadata(text: &str, max_width: usize) -> String {
    terminal::sanitize_display_text(text, max_width)
}

fn connection_info(result: &TestResult, nc: bool, theme: Theme, term_w: usize) -> String {
    let distance = sanitize_metadata(&common::format_distance(result.server.distance), 12);
    let field_width = (term_w.saturating_sub(50) / 2).clamp(8, 48);
    let sponsor = sanitize_metadata(&result.server.sponsor, field_width);
    let country = sanitize_metadata(
        &format!("{}, {}", result.server.name, result.server.country),
        field_width.max(20),
    );
    let server_line = if term_w < 100 && nc {
        format!("  Server: {sponsor}\n  Location: {country} · {distance}")
    } else if term_w < 100 {
        format!(
            "  {} {}\n  {} {} · {}",
            Colors::dimmed("Server:", theme),
            Colors::bold(&sponsor, theme),
            Colors::dimmed("Location:", theme),
            Colors::dimmed(&country, theme),
            Colors::dimmed(&distance, theme),
        )
    } else if nc {
        format!(
            "  ◈  Server    {}  ·  {}  ·  {}",
            sponsor, country, distance
        )
    } else {
        let lbl = Colors::muted("◈", theme);
        format!(
            "  {lbl}  {}    {}  {}  {}  {}  {}",
            Colors::dimmed("Server", theme),
            sponsor,
            Colors::muted("·", theme),
            Colors::dimmed(&country, theme),
            Colors::muted("·", theme),
            Colors::dimmed(&distance, theme),
        )
    };

    let client_line = result.client_ip.as_deref().map(|ip| {
        let ip = sanitize_metadata(ip, field_width);
        if nc {
            format!("\n  ◈  Client    {ip}")
        } else {
            let lbl = Colors::muted("◈", theme);
            format!(
                "\n  {lbl}  {}    {}",
                Colors::dimmed("Client", theme),
                Colors::dimmed(&ip, theme),
            )
        }
    });

    match client_line {
        Some(cl) => format!("{server_line}{cl}"),
        None => server_line,
    }
}

/// Single-line data transfer summary.
fn data_summary(summary: &Summary, nc: bool, theme: Theme, term_w: usize) -> String {
    let elapsed = summary.elapsed.as_secs_f64();
    let total = summary.dl_bytes.saturating_add(summary.ul_bytes);

    let mut parts: Vec<String> = Vec::new();

    if summary.dl_bytes > 0 {
        let s = format!(
            "Downloaded {}  in  {:.1}s",
            common::format_data_size(summary.dl_bytes),
            summary.dl_duration
        );
        parts.push(s);
    }
    if summary.ul_bytes > 0 {
        let s = format!(
            "Uploaded {}  in  {:.1}s",
            common::format_data_size(summary.ul_bytes),
            summary.ul_duration
        );
        parts.push(s);
    }
    if total > 0 {
        parts.push(format!("Total {}", common::format_data_size(total)));
    }
    parts.push(format!("{elapsed:.1}s"));

    let sep = if nc {
        "  ·  ".to_string()
    } else {
        format!("  {}  ", Colors::muted("·", theme))
    };

    let joined = if term_w < 80 {
        parts.join("\n  ")
    } else {
        parts.join(&sep)
    };

    if nc {
        format!("  {joined}")
    } else {
        format!("  {}", Colors::dimmed(&joined, theme))
    }
}

pub fn show(result: &TestResult, summary: &Summary) -> Result<(), Error> {
    show_with_ascii(result, summary, false)
}

pub(crate) fn show_ascii(result: &TestResult, summary: &Summary) -> Result<(), Error> {
    show_with_ascii(result, summary, true)
}

fn show_with_ascii(result: &TestResult, summary: &Summary, ascii: bool) -> Result<(), Error> {
    use std::io::Write;
    let (width, height) = terminal_size::terminal_size_of(std::io::stderr())
        .map(|(width, height)| (width.0 as usize, height.0 as usize))
        .unwrap_or((90, 24));
    let report = render(result, summary, width, height);
    let report = if ascii { ascii_report(&report) } else { report };
    std::io::stderr().lock().write_all(report.as_bytes())?;
    Ok(())
}

fn ascii_report(report: &str) -> String {
    console::strip_ansi_codes(report)
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || !trimmed.chars().all(|ch| "┌┐└┘─╌".contains(ch))
        })
        .map(|line| {
            line.chars()
                .map(|ch| match ch {
                    '↓' => 'v',
                    '↑' => '^',
                    '→' => '>',
                    '—' | '·' => '-',
                    '◉' | '◈' => '*',
                    '│' => ' ',
                    '░' => ' ',
                    '▁' | '▂' => '.',
                    '▃' | '▄' => ':',
                    '▅' | '▆' => '*',
                    '▇' | '█' => '#',
                    ch if ch.is_ascii() => ch,
                    _ => '?',
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn render(result: &TestResult, summary: &Summary, term_w: usize, term_h: usize) -> String {
    let skipped = SkipState {
        download: summary.skipped.download
            || result.phases.download.state == crate::types::PhaseState::Skipped,
        upload: summary.skipped.upload
            || result.phases.upload.state == crate::types::PhaseState::Skipped,
    };
    let mut presented = result.clone();
    for value in [
        &mut presented.ping,
        &mut presented.jitter,
        &mut presented.download,
        &mut presented.upload,
    ] {
        *value = value.filter(|value| value.is_finite() && *value >= 0.0);
    }
    if skipped.download {
        presented.download = None;
        presented.download_samples = None;
        presented.download_cv = None;
        presented.latency_download = None;
    }
    if skipped.upload {
        presented.upload = None;
        presented.upload_samples = None;
        presented.upload_cv = None;
        presented.latency_upload = None;
    }
    let result = &presented;
    let mut output = String::new();
    macro_rules! eprintln {
        () => { output.push('\n') };
        ($($arg:tt)*) => {{
            use std::fmt::Write;
            let _ = writeln!(output, $($arg)*);
        }};
    }
    let nc = terminal::no_color() || summary.theme == Theme::Monochrome;
    let theme = summary.theme;
    let gauge_w = if term_w < 68 {
        0
    } else {
        (term_w.saturating_sub(58)).clamp(8, 30)
    };
    let gauge_max = gauge_scale(
        [
            summary.dl_peak_mbps,
            summary.ul_peak_mbps,
            summary.dl_mbps,
            summary.ul_mbps,
        ]
        .into_iter()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .fold(1.0_f64, f64::max),
    );
    let spark_w = if term_w < 68 {
        0
    } else {
        (term_w.saturating_sub(60)).clamp(8, 24)
    };

    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        summary.profile,
    );

    // Keep reports within short terminals by dropping secondary blank space.
    let compact_height = term_h < 12;
    // ── Header Panel ──────────────────────────────────────────────────────────
    eprintln!();
    if [result.ping, result.jitter, result.download, result.upload]
        .iter()
        .any(Option::is_some)
    {
        eprintln!(
            "{}",
            boxed_header(&overall_grade, nc, theme, term_w.saturating_sub(1))
        );
    } else {
        eprintln!("  NETSPEED · assessment unavailable");
    }
    if !compact_height {
        eprintln!();
    }

    // ── Download Block ────────────────────────────────────────────────────────
    let dl_spark = result
        .download_samples
        .as_deref()
        .map(|s| render_sparkline_from_samples(s, spark_w))
        .unwrap_or_default();
    let dl_trend = result
        .download_samples
        .as_deref()
        .map(speed_trend)
        .unwrap_or("→");

    if skipped.download {
        eprintln!("  ↓  Download: — (skipped)");
    } else if result
        .download
        .filter(|v| v.is_finite() && *v >= 0.0)
        .is_none()
        || !summary.dl_mbps.is_finite()
        || summary.dl_mbps < 0.0
    {
        eprintln!("  ↓  Download: unavailable");
    } else {
        eprintln!(
            "{}",
            speed_block(
                "↓",
                "Download",
                summary.dl_mbps,
                Some(summary.dl_peak_mbps),
                gauge_max,
                theme,
                gauge_w,
                &dl_spark,
                dl_trend,
                nc,
                summary.bytes,
            )
        );
    }
    if !compact_height {
        eprintln!();
    }

    // ── Upload Block ──────────────────────────────────────────────────────────
    let ul_spark = result
        .upload_samples
        .as_deref()
        .map(|s| render_sparkline_from_samples(s, spark_w))
        .unwrap_or_default();
    let ul_trend = result
        .upload_samples
        .as_deref()
        .map(speed_trend)
        .unwrap_or("→");

    if skipped.upload {
        eprintln!("  ↑  Upload: — (skipped)");
    } else if result
        .upload
        .filter(|v| v.is_finite() && *v >= 0.0)
        .is_none()
        || !summary.ul_mbps.is_finite()
        || summary.ul_mbps < 0.0
    {
        eprintln!("  ↑  Upload: unavailable");
    } else {
        eprintln!(
            "{}",
            speed_block(
                "↑",
                "Upload",
                summary.ul_mbps,
                Some(summary.ul_peak_mbps),
                gauge_max,
                theme,
                gauge_w,
                &ul_spark,
                ul_trend,
                nc,
                summary.bytes,
            )
        );
    }
    if !compact_height {
        eprintln!();
    }

    // ── Latency Row ───────────────────────────────────────────────────────────
    if let Some(ping) = result.ping.filter(|v| v.is_finite() && *v >= 0.0) {
        eprintln!(
            "{}",
            latency_row(ping, result.jitter, result.packet_loss, nc, theme, term_w)
        );
        if !compact_height {
            eprintln!();
        }
    }

    if result.ping.is_none() {
        if let Some(jitter) = result.jitter.filter(|v| v.is_finite() && *v >= 0.0) {
            eprintln!("  Jitter {jitter:.1} ms");
        }
        if let Some(loss) = result
            .packet_loss
            .filter(|v| v.is_finite() && (0.0..=100.0).contains(v))
        {
            eprintln!("  Packet loss {loss:.1}%");
        }
    }

    for (direction, load, cv) in [
        ("Download", result.latency_download, result.download_cv),
        ("Upload", result.latency_upload, result.upload_cv),
    ] {
        if let Some(load) = load.filter(|v| v.is_finite() && *v >= 0.0) {
            eprintln!("  {direction} load latency {load:.1} ms");
            if let Some(idle) = result.ping.filter(|v| v.is_finite() && *v >= 0.0) {
                let (grade, added) = super::ratings::bufferbloat_grade(load, idle);
                eprintln!(
                    "  Bufferbloat {}",
                    super::ratings::bufferbloat_colorized(grade, added, nc, theme)
                );
            }
        }
        if let Some(cv) = cv.filter(|v| v.is_finite() && *v >= 0.0) {
            eprintln!("  {direction} variation {:.1}%", cv * 100.0);
        }
    }
    eprintln!(
        "  Completed {}",
        terminal::sanitize_display_text(&result.timestamp, term_w.saturating_sub(12))
    );
    // ── Connection Info ───────────────────────────────────────────────────────
    eprintln!("{}", thin_separator(term_w, nc, theme));
    if !compact_height {
        eprintln!();
    }
    eprintln!("{}", connection_info(result, nc, theme, term_w));
    if !compact_height {
        eprintln!();
    }

    // ── Data Summary ──────────────────────────────────────────────────────────
    eprintln!("{}", thin_separator(term_w, nc, theme));
    if !compact_height {
        eprintln!();
    }
    eprintln!("{}", data_summary(summary, nc, theme, term_w));
    if let Some(text) = super::profile_interpretation(result, summary.profile) {
        let title = if nc {
            "Interpretation".to_string()
        } else {
            Colors::header("Interpretation", theme)
        };
        eprintln!("\n  {title}: {text}");
    }
    if !compact_height {
        eprintln!();
    }

    output
        .lines()
        .flat_map(|line| {
            textwrap::wrap(line, term_w.saturating_sub(1).max(1))
                .into_iter()
                .map(|line| console::truncate_str(&line, term_w.saturating_sub(1), "").into_owned())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary_fixture() -> Summary {
        Summary {
            dl_mbps: 100.0,
            dl_peak_mbps: 110.0,
            dl_bytes: 10_000_000,
            dl_duration: 12.0,
            ul_mbps: 30.0,
            ul_peak_mbps: 35.0,
            ul_bytes: 4_000_000,
            ul_duration: 10.0,
            elapsed: std::time::Duration::from_secs(25),
            profile: UserProfile::PowerUser,
            theme: Theme::Monochrome,
            bytes: false,
            skipped: SkipState::default(),
        }
    }

    #[test]
    fn historical_skips_and_independent_health_metrics_remain_visible() {
        let mut result = TestResult {
            download: Some(0.0),
            jitter: Some(2.0),
            packet_loss: Some(1.0),
            ..TestResult::default()
        };
        result.phases.download = crate::types::PhaseResult::skipped("legacy skipped test");
        let report = render(&result, &summary_fixture(), 80, 24);
        assert!(report.contains("Download: — (skipped)"));
        assert!(report.contains("Jitter 2.0 ms"));
        assert!(report.contains("Packet loss 1.0%"));
        assert!(!report.contains("peak"));
    }

    #[test]
    fn ascii_report_preserves_metrics_and_uses_plain_terminal_characters() {
        let mut result = TestResult::default();
        result.server.sponsor = "東京 Fixture".into();
        result.download = Some(100_000_000.0);
        result.upload = Some(30_000_000.0);
        let report = ascii_report(&render(&result, &summary_fixture(), 80, 24));
        assert!(report.is_ascii());
        assert!(!report.contains('\u{1b}'));
        assert!(report.contains("Download"));
        assert!(report.contains("100.0 Mb/s"));
        assert!(report.contains("Upload"));
        assert!(report.contains("Fixture"));
    }

    #[test]
    fn byte_rate_uses_byte_units() {
        assert_eq!(format_rate(100.0, true), "12.5 MB/s");
        assert_eq!(format_rate(0.5, true), "62.5 KB/s");
        assert_eq!(format_rate(100.0, false), "100.0 Mb/s");
    }

    #[test]
    fn narrow_speed_block_omits_gauge_and_keeps_value() {
        let block = speed_block(
            "↓",
            "Download",
            125.0,
            Some(140.0),
            500.0,
            Theme::Dark,
            0,
            "",
            "→",
            true,
            false,
        );
        assert!(block.contains("125.0 Mb/s"));
        assert!(!block.contains('█'));
    }

    #[test]
    fn metadata_sanitizer_removes_control_sequences_and_truncates_by_width() {
        let text = sanitize_metadata("East\u{1b}[31m\n東京 Server", 8);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\n'));
        assert!(console::measure_text_width(&text) <= 8);
    }

    #[test]
    fn sparkline_does_not_force_chart_into_narrow_layout() {
        assert!(render_sparkline_from_samples(&[1.0, 2.0, 3.0], 0).is_empty());
    }

    #[test]
    fn header_layout_fits_supported_terminal_widths() {
        for grade in [
            LetterGrade::APlus,
            LetterGrade::A,
            LetterGrade::AMinus,
            LetterGrade::BPlus,
            LetterGrade::B,
            LetterGrade::BMinus,
            LetterGrade::CPlus,
            LetterGrade::C,
            LetterGrade::CMinus,
            LetterGrade::D,
            LetterGrade::F,
        ] {
            for width in [0, 1, 12, 23, 24, 39, 40, 59, 60, 79, 80, 119, 120] {
                for theme in [
                    Theme::Dark,
                    Theme::Light,
                    Theme::HighContrast,
                    Theme::Monochrome,
                ] {
                    for nc in [true, false] {
                        let header = boxed_header(&grade, nc, theme, width);
                        let widths: Vec<_> =
                            header.lines().map(console::measure_text_width).collect();
                        assert!(
                            widths.iter().all(|&line| line <= width),
                            "{width}: {header}"
                        );
                        if width >= 24 {
                            assert_eq!(widths, vec![width.min(80); 3], "{header}");
                            assert!(console::strip_ansi_codes(&header).contains(&format!(
                                "{} · {}",
                                grade.as_str(),
                                grade.score() as u32
                            )));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn narrow_latency_layout_stacks_metrics_without_overflow() {
        let row = latency_row(18.2, Some(2.4), Some(0.1), true, Theme::Dark, 40);
        assert_eq!(row.lines().count(), 3);
        for line in row.lines() {
            assert!(console::measure_text_width(line) <= 40);
        }
    }

    #[test]
    fn narrow_data_summary_wraps_each_metric() {
        let mut summary = summary_fixture();
        let text = data_summary(&summary, true, Theme::Monochrome, 60);
        assert!(text.lines().count() > 1);
        for line in text.lines() {
            assert!(console::measure_text_width(line) <= 60);
        }
        let mut result = TestResult::default();
        result.server.sponsor = "東京 Server with a very long sponsor name".into();
        result.download = Some(100_000_000.0);
        result.upload = Some(30_000_000.0);
        for theme in [
            Theme::Dark,
            Theme::Light,
            Theme::HighContrast,
            Theme::Monochrome,
        ] {
            summary.theme = theme;
            for width in [40, 60, 80, 120] {
                for height in [4, 8, 24] {
                    let report = render(&result, &summary, width, height);
                    assert!(report.contains("Download"));
                    assert!(report.contains("Upload"));
                    for line in report.lines() {
                        assert!(
                            console::measure_text_width(line) < width,
                            "{width} columns: {line:?}"
                        );
                    }
                }
            }
        }
    }
}
