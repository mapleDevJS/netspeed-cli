//! Speed stability analysis and latency percentiles.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use owo_colors::OwoColorize;

/// Compute coefficient of variation (CV) as a percentage.
#[must_use]
pub fn compute_cv(speeds: &[f64]) -> f64 {
    if speeds.is_empty() {
        return 0.0;
    }
    let n = speeds.len() as f64;
    let mean = speeds.iter().sum::<f64>() / n;
    if mean <= 0.0 {
        return 0.0;
    }
    let variance = speeds.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    (stddev / mean) * 100.0
}

pub fn format_stability_line(cv: f64, nc: bool) -> String {
    let (color, label) = if cv < 5.0 {
        ("green", "rock-solid")
    } else if cv < 10.0 {
        ("bright_green", "very stable")
    } else if cv < 20.0 {
        ("yellow", "moderate")
    } else if cv < 35.0 {
        ("bright_yellow", "variable")
    } else {
        ("red", "unstable")
    };
    let text = format!("±{cv:.0}% {label}");
    if nc {
        text
    } else {
        match color {
            "green" => text.green().to_string(),
            "bright_green" => text.bright_green().to_string(),
            "yellow" => text.yellow().to_string(),
            "bright_yellow" => text.bright_yellow().to_string(),
            "red" => text.red().bold().to_string(),
            _ => text.to_string(),
        }
    }
}

pub fn compute_percentiles(samples: &[f64]) -> Option<(f64, f64, f64)> {
    if samples.len() < 3 {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let p50 = sorted[n * 50 / 100];
    let p95 = sorted[(n * 95 / 100).min(n - 1)];
    let p99 = sorted[(n * 99 / 100).min(n - 1)];
    Some((p50, p95, p99))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cv_constant() {
        let speeds = vec![100.0, 100.0, 100.0];
        assert_eq!(compute_cv(&speeds), 0.0);
    }

    #[test]
    fn test_compute_cv_empty() {
        assert_eq!(compute_cv(&[]), 0.0);
    }

    #[test]
    fn test_compute_percentiles_basic() {
        let samples: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let result = compute_percentiles(&samples);
        assert!(result.is_some());
        let (p50, p95, p99) = result.unwrap();
        // With 100 elements (indices 0..99), n*50/100 = 50 → index 50 → value 51.0
        assert!((p50 - 51.0).abs() < 1.0);
        assert!((p95 - 96.0).abs() < 1.0);
        assert!((p99 - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_compute_percentiles_too_few() {
        assert!(compute_percentiles(&[1.0, 2.0]).is_none());
    }
}
