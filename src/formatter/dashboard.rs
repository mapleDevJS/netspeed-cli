//! Dashboard output formatting — rich single-screen summary with gauges and sparklines.

use crate::common;
use crate::error::Error;
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
    if samples.len() < 2 {
        return String::new();
    }
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let len = width.max(2);
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
    // inner_w: space between │ chars (box_w - 2 leading spaces - 2 border chars)
    let inner_w = box_w.saturating_sub(4);
    // content_w: usable space inside padding (inner_w - 2 leading spaces - 2 trailing spaces)
    let content_w = inner_w.saturating_sub(4);

    let score = grade.score() as u32;
    let grade_str = grade.as_str();
    let desc_str = grade.description();
    let left_text = format!("◉ NETSPEED  ·  {desc_str}");
    let right_text = format!("{grade_str} · {score}");

    // Visible widths (all BMP narrow chars, char count == display width)
    let left_vis = left_text.chars().count();
    let right_vis = right_text.chars().count();
    let padding = content_w.saturating_sub(left_vis + right_vis);

    let top = format!("  ┌{}┐", "─".repeat(inner_w));
    let bot = format!("  └{}┘", "─".repeat(inner_w));

    if nc {
        let mid = format!("  │  {left_text}{}{}  │", " ".repeat(padding), right_text);
        format!("{top}\n{mid}\n{bot}")
    } else {
        let left_col = format!(
            "{}  ·  {}",
            Colors::bold("◉ NETSPEED", theme),
            Colors::dimmed(desc_str, theme),
        );
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
fn speed_block(
    dir: &str,
    label: &str,
    speed_mbps: f64,
    peak_mbps: f64,
    max_mbps: f64,
    theme: Theme,
    gauge_w: usize,
    sparkline: &str,
    trend: &str,
    nc: bool,
) -> String {
    let pct = (speed_mbps / max_mbps).clamp(0.0, 1.0);
    let filled = (pct * gauge_w as f64).round() as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(gauge_w.saturating_sub(filled))
    );

    let speed_str = if speed_mbps < 1000.0 {
        format!("{speed_mbps:.1} Mb/s")
    } else {
        format!("{:.2} Gb/s", speed_mbps / 1000.0)
    };
    let peak_str = if peak_mbps < 1000.0 {
        format!("peak {peak_mbps:.0}")
    } else {
        format!("peak {:.1}G", peak_mbps / 1000.0)
    };

    // Prefix before the bar: "  {dir(1)}  {label(10)}  " = 17 visible chars.
    // Compute from plain text so the indent is the same in both nc and color paths.
    let indent = " ".repeat(2 + dir.chars().count() + 2 + 10 + 2);

    if nc {
        let line1 = format!("  {dir}  {label:<10}  {bar}  {speed_str}   {peak_str}");
        if sparkline.is_empty() {
            line1
        } else {
            format!("{line1}\n{indent}{sparkline}   {trend}")
        }
    } else {
        let bar_col = if pct >= 0.7 {
            Colors::good(&bar, theme)
        } else if pct >= 0.4 {
            Colors::warn(&bar, theme)
        } else {
            Colors::bad(&bar, theme)
        };
        let speed_col = if pct >= 0.7 {
            Colors::good(&speed_str, theme)
        } else if pct >= 0.4 {
            Colors::warn(&speed_str, theme)
        } else {
            Colors::bad(&speed_str, theme)
        };
        let dir_col = Colors::muted(dir, theme);
        // Pad the plain label to 10 visible chars BEFORE colorizing so ANSI bytes
        // don't confuse Rust's format-string width specifier.
        let label_col = Colors::dimmed(&format!("{label:<10}"), theme);
        let peak_col = Colors::dimmed(&peak_str, theme);
        let trend_col = Colors::dimmed(trend, theme);

        let line1 = format!("  {dir_col}  {label_col}  {bar_col}  {speed_col}   {peak_col}");

        if sparkline.is_empty() {
            line1
        } else {
            let spark_col = if dir == "↓" {
                Colors::info(sparkline, theme)
            } else {
                Colors::good(sparkline, theme)
            };
            format!("{line1}\n{indent}{spark_col}   {trend_col}")
        }
    }
}

/// Combined single-line latency + jitter + packet loss row.
fn latency_row(
    ping_ms: f64,
    jitter: Option<f64>,
    packet_loss: Option<f64>,
    nc: bool,
    theme: Theme,
) -> String {
    let gauge_w: usize = 12;
    let max_ping = 100.0;
    let pct = (ping_ms / max_ping).clamp(0.0, 1.0);
    let filled = ((1.0 - pct) * gauge_w as f64).round() as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(gauge_w.saturating_sub(filled))
    );

    let bar_col = if ping_ms <= 20.0 {
        Colors::good(&bar, theme)
    } else if ping_ms <= 50.0 {
        Colors::warn(&bar, theme)
    } else {
        Colors::bad(&bar, theme)
    };

    let ping_str = format!("{ping_ms:.1} ms");
    let ping_col = if ping_ms <= 20.0 {
        Colors::good(&ping_str, theme)
    } else if ping_ms <= 50.0 {
        Colors::warn(&ping_str, theme)
    } else {
        Colors::bad(&ping_str, theme)
    };

    let mut parts = if nc {
        format!("  ◈  Latency    {bar}  {ping_str}")
    } else {
        let lbl = Colors::muted("◈", theme);
        format!(
            "  {lbl}  {}    {bar_col}  {ping_col}",
            Colors::dimmed("Latency", theme)
        )
    };

    if let Some(j) = jitter {
        let j_str = format!("{j:.1} ms");
        if nc {
            parts.push_str(&format!("   ◈  Jitter  {j_str}"));
        } else {
            let lbl = Colors::muted("◈", theme);
            parts.push_str(&format!(
                "   {lbl}  {}  {}",
                Colors::dimmed("Jitter", theme),
                Colors::info(&j_str, theme),
            ));
        }
    }

    if let Some(loss) = packet_loss {
        let l_str = format!("{loss:.1}%");
        if nc {
            parts.push_str(&format!("   ◈  Loss  {l_str}"));
        } else {
            let lbl = Colors::muted("◈", theme);
            let loss_col = if loss < 0.5 {
                Colors::good(&l_str, theme)
            } else if loss < 2.0 {
                Colors::warn(&l_str, theme)
            } else {
                Colors::bad(&l_str, theme)
            };
            parts.push_str(&format!(
                "   {lbl}  {}  {loss_col}",
                Colors::dimmed("Loss", theme),
            ));
        }
    }

    parts
}

/// Dashed thin separator line.
fn thin_separator(w: usize, nc: bool, theme: Theme) -> String {
    let line = "╌".repeat(w.min(78));
    if nc {
        format!("  {line}")
    } else {
        format!("  {}", Colors::dimmed(&line, theme))
    }
}

/// Inline connection info lines (server + client).
fn connection_info(result: &TestResult, nc: bool, theme: Theme) -> String {
    let distance = common::format_distance(result.server.distance);
    let server_line = if nc {
        format!(
            "  ◈  Server    {}  ·  {}  ·  {}",
            result.server.sponsor, result.server.country, distance
        )
    } else {
        let lbl = Colors::muted("◈", theme);
        format!(
            "  {lbl}  {}    {}  {}  {}  {}  {}",
            Colors::dimmed("Server", theme),
            result.server.sponsor,
            Colors::muted("·", theme),
            Colors::dimmed(&result.server.country, theme),
            Colors::muted("·", theme),
            Colors::dimmed(&distance, theme),
        )
    };

    let client_line = result.client_ip.as_deref().map(|ip| {
        if nc {
            format!("\n  ◈  Client    {ip}")
        } else {
            let lbl = Colors::muted("◈", theme);
            format!(
                "\n  {lbl}  {}    {}",
                Colors::dimmed("Client", theme),
                Colors::dimmed(ip, theme),
            )
        }
    });

    match client_line {
        Some(cl) => format!("{server_line}{cl}"),
        None => server_line,
    }
}

/// Single-line data transfer summary.
fn data_summary(summary: &Summary, nc: bool, theme: Theme) -> String {
    let elapsed = summary.elapsed.as_secs_f64();
    let total = summary.dl_bytes + summary.ul_bytes;

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

    let joined = parts.join(&sep);

    if nc {
        format!("  {joined}")
    } else {
        format!("  {}", Colors::dimmed(&joined, theme))
    }
}

pub fn show(result: &TestResult, summary: &Summary) -> Result<(), Error> {
    let nc = terminal::no_color() || summary.theme == Theme::Monochrome;
    let theme = summary.theme;
    let term_w = common::get_terminal_width().unwrap_or(90) as usize;
    let gauge_w = (term_w.saturating_sub(52)).clamp(15, 35);
    let gauge_max = gauge_scale(summary.dl_peak_mbps.max(summary.ul_peak_mbps).max(1.0));
    let spark_w = (term_w.saturating_sub(54)).clamp(8, 30);

    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        summary.profile,
    );

    // ── Header Panel ──────────────────────────────────────────────────────────
    eprintln!();
    eprintln!("{}", boxed_header(&overall_grade, nc, theme, term_w));
    eprintln!();

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

    eprintln!(
        "{}",
        speed_block(
            "↓",
            "Download",
            summary.dl_mbps,
            summary.dl_peak_mbps,
            gauge_max,
            theme,
            gauge_w,
            &dl_spark,
            dl_trend,
            nc,
        )
    );
    eprintln!();

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

    eprintln!(
        "{}",
        speed_block(
            "↑",
            "Upload",
            summary.ul_mbps,
            summary.ul_peak_mbps,
            gauge_max,
            theme,
            gauge_w,
            &ul_spark,
            ul_trend,
            nc,
        )
    );
    eprintln!();

    // ── Latency Row ───────────────────────────────────────────────────────────
    if let Some(ping) = result.ping {
        eprintln!(
            "{}",
            latency_row(ping, result.jitter, result.packet_loss, nc, theme)
        );
        eprintln!();
    }

    // ── Connection Info ───────────────────────────────────────────────────────
    eprintln!("{}", thin_separator(term_w, nc, theme));
    eprintln!();
    eprintln!("{}", connection_info(result, nc, theme));
    eprintln!();

    // ── Data Summary ──────────────────────────────────────────────────────────
    eprintln!("{}", thin_separator(term_w, nc, theme));
    eprintln!();
    eprintln!("{}", data_summary(summary, nc, theme));
    eprintln!();

    Ok(())
}
