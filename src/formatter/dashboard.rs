//! Dashboard output formatting — rich single-screen summary with gauges and sparklines.

use crate::common;
use crate::error::Error;
use crate::grades;
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

fn speed_gauge(
    label: &str,
    speed_mbps: f64,
    peak_mbps: f64,
    theme: Theme,
    gauge_width: usize,
) -> String {
    let max_gauge = 1000.0;
    let pct = (speed_mbps / max_gauge).clamp(0.0, 1.0);
    let filled = (pct * gauge_width as f64).round() as usize;

    let bar_char = if pct >= 0.7 {
        "█"
    } else if pct >= 0.4 {
        "▓"
    } else {
        "░"
    };
    let empty_char = "·";
    let bar = format!(
        "{}{}",
        bar_char.repeat(filled),
        empty_char.repeat(gauge_width.saturating_sub(filled))
    );

    let speed_str = if speed_mbps < 1000.0 {
        format!("{speed_mbps:.1} Mb/s")
    } else {
        format!("{:.2} Gb/s", speed_mbps / 1000.0)
    };
    let peak_str = if peak_mbps < 1000.0 {
        format!("peak {peak_mbps:.0}")
    } else {
        format!("peak {:.0}", peak_mbps / 1000.0)
    };

    let bar_colored = if pct >= 0.7 {
        Colors::good(&bar, theme)
    } else if pct >= 0.4 {
        Colors::warn(&bar, theme)
    } else {
        Colors::bad(&bar, theme)
    };

    format!(
        "  {:<10} {} {} {}",
        label,
        bar_colored,
        Colors::bold(&speed_str, theme),
        peak_str.dimmed()
    )
}

fn metric_line(label: &str, value: &str, theme: Theme) -> String {
    format!("  {:<10} {}", label.dimmed(), Colors::info(value, theme))
}

fn latency_gauge(ping_ms: f64, theme: Theme) -> String {
    let max_ping = 100.0;
    let pct = (ping_ms / max_ping).clamp(0.0, 1.0);
    let gauge_width: usize = 10;
    let filled = (pct * gauge_width as f64).round() as usize;

    let bar_char = if pct <= 0.2 {
        "█"
    } else if pct <= 0.5 {
        "▓"
    } else {
        "░"
    };
    let bar = format!(
        "{}{}",
        bar_char.repeat(filled),
        "·".repeat(gauge_width.saturating_sub(filled))
    );

    let bar_colored = if ping_ms <= 20.0 {
        Colors::good(&bar, theme)
    } else if ping_ms <= 50.0 {
        Colors::warn(&bar, theme)
    } else {
        Colors::bad(&bar, theme)
    };

    format!("  {:<10} {} {:.1} ms", "Latency", bar_colored, ping_ms)
}

fn render_sparkline_from_samples(samples: &[f64]) -> String {
    if samples.len() < 2 {
        return "— no samples —".to_string();
    }
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let len = 20usize;
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

pub fn show(result: &TestResult, summary: &Summary) -> Result<(), Error> {
    let nc = terminal::no_color() || summary.theme == Theme::Monochrome;
    let theme = summary.theme;
    let term_w = common::get_terminal_width().unwrap_or(90) as usize;
    let gauge_w = (term_w.saturating_sub(52)).clamp(15, 35);

    let overall_grade = grades::grade_overall(
        result.ping,
        result.jitter,
        result.download,
        result.upload,
        summary.profile,
    );
    let grade_display = overall_grade.color_str(nc, theme);

    let separator = if nc {
        "─".repeat(term_w.min(80))
    } else {
        "─".repeat(term_w.min(80)).dimmed().to_string()
    };

    eprintln!();
    if nc {
        eprintln!("  {separator}");
        eprintln!("  SPEED TEST RESULTS");
        eprintln!("  {separator}");
    } else {
        eprintln!("  {}", separator);
        eprintln!("  {}", Colors::header("SPEED TEST RESULTS", theme));
        eprintln!("  {}", separator);
    }

    eprintln!();

    if !nc {
        eprintln!("  {} {}", "Grade:".dimmed(), grade_display);
    } else {
        eprintln!("  Grade:    {}", overall_grade.as_str());
    }
    eprintln!();

    eprintln!(
        "{}",
        speed_gauge(
            "Download",
            summary.dl_mbps,
            summary.dl_peak_mbps,
            theme,
            gauge_w
        )
    );
    eprintln!(
        "{}",
        speed_gauge(
            "Upload",
            summary.ul_mbps,
            summary.ul_peak_mbps,
            theme,
            gauge_w
        )
    );

    if let Some(ping) = result.ping {
        eprintln!("{}", latency_gauge(ping, theme));
    }

    if let Some(jitter) = result.jitter {
        eprintln!(
            "{}",
            metric_line("Jitter", &format!("{jitter:.1} ms"), theme)
        );
    }

    eprintln!();

    if let (Some(dl_samples), Some(ul_samples)) = (&result.download_samples, &result.upload_samples)
    {
        let dl_spark = render_sparkline_from_samples(dl_samples);
        let ul_spark = render_sparkline_from_samples(ul_samples);
        if nc {
            eprintln!("  {:<10} {}", "DL curve:", dl_spark);
            eprintln!("  {:<10} {}", "UL curve:", ul_spark);
        } else {
            eprintln!("  {:<10} {}", "DL curve:".dimmed(), dl_spark.cyan());
            eprintln!("  {:<10} {}", "UL curve:".dimmed(), ul_spark.magenta());
        }
        eprintln!();
    }

    if nc {
        eprintln!("  {separator}");
        eprintln!("  CONNECTION");
        eprintln!("  {separator}");
    } else {
        eprintln!("  {}", separator);
        eprintln!("  {}", Colors::header("CONNECTION", theme));
        eprintln!("  {}", separator);
    }

    eprintln!("  {} {}", "Server:".dimmed(), result.server.sponsor);
    eprintln!(
        "  {} {}, {} away",
        "Location:".dimmed(),
        result.server.country,
        common::format_distance(result.server.distance)
    );
    if let Some(ref ip) = result.client_ip {
        eprintln!("  {} {}", "Client IP:".dimmed(), ip);
    }
    eprintln!();

    if nc {
        eprintln!("  {separator}");
        eprintln!("  DATA TRANSFERRED");
        eprintln!("  {separator}");
    } else {
        eprintln!("  {}", separator);
        eprintln!("  {}", Colors::header("DATA TRANSFERRED", theme));
        eprintln!("  {}", separator);
    }

    if summary.dl_bytes > 0 {
        eprintln!(
            "  {:<14} {} in {:.1}s",
            "Download:".dimmed(),
            common::format_data_size(summary.dl_bytes),
            summary.dl_duration
        );
    }
    if summary.ul_bytes > 0 {
        eprintln!(
            "  {:<14} {} in {:.1}s",
            "Upload:".dimmed(),
            common::format_data_size(summary.ul_bytes),
            summary.ul_duration
        );
    }
    let total_bytes = summary.dl_bytes + summary.ul_bytes;
    if total_bytes > 0 {
        eprintln!(
            "  {:<14} {}",
            "Total:".dimmed(),
            common::format_data_size(total_bytes)
        );
    }
    eprintln!(
        "  {:<14} {:.1}s",
        "Test time:".dimmed(),
        summary.elapsed.as_secs_f64()
    );

    eprintln!();
    eprintln!("  {}", separator);
    eprintln!();

    Ok(())
}
