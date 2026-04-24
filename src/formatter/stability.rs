//! Speed stability analysis and latency percentiles.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::theme::{Colors, Theme};

/// Compute coefficient of variation (CV) as a percentage.
#[must_use]
pub fn compute_cv(speeds: &[f64]) -> f64 {
    if speeds.is_empty() {
        return 0.0;
    }
    // Safe: sample counts are small (≤1000), well under 2^53.
    let n = speeds.len() as f64;
    let mean = speeds.iter().sum::<f64>() / n;
    if mean <= 0.0 {
        return 0.0;
    }
    let variance = speeds.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    (stddev / mean) * 100.0
}

#[must_use]
pub fn format_stability_line(cv: f64, nc: bool, theme: Theme) -> String {
    let label = if cv < 5.0 {
        "rock-solid"
    } else if cv < 10.0 {
        "very stable"
    } else if cv < 20.0 {
        "moderate"
    } else if cv < 35.0 {
        "variable"
    } else {
        "unstable"
    };
    let text = format!("±{cv:.0}% {label}");
    if nc {
        text
    } else if cv < 5.0 {
        Colors::good(&text, theme)
    } else if cv < 20.0 {
        Colors::warn(&text, theme)
    } else {
        Colors::bad(&text, theme)
    }
}

#[must_use]
pub fn compute_percentiles(samples: &[f64]) -> Option<(f64, f64, f64)> {
    let n = samples.len();
    if n < 3 {
        return None;
    }
    let mut data = samples.to_vec();
    let p50_idx = n * 50 / 100;
    let p95_idx = (n * 95 / 100).min(n - 1);
    let p99_idx = (n * 99 / 100).min(n - 1);

    // Partition at p99: elements before are <= p99, elements after are >= p99
    data.select_nth_unstable_by(p99_idx, |a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    let p99 = data[p99_idx];

    // Partition the left slice at p95
    data.select_nth_unstable_by(p95_idx, |a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    let p95 = data[p95_idx];

    // Partition the left slice at p50
    data.select_nth_unstable_by(p50_idx, |a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    let p50 = data[p50_idx];

    Some((p50, p95, p99))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cv_constant() {
        let speeds = vec![100.0, 100.0, 100.0];
        assert!(compute_cv(&speeds).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_cv_empty() {
        assert!(compute_cv(&[]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_percentiles_basic() {
        let samples: Vec<f64> = (1..=100).map(f64::from).collect();
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
