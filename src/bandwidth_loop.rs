//! Shared bandwidth measurement loop for download/upload tests.
//!
//! Eliminates duplication between `download.rs` and `upload.rs` by providing
//! a unified state for:
//! - Throttled speed sampling (20 Hz max)
//! - Peak speed tracking
//! - Progress bar updates
//! - Atomic byte counting
//!
//! Each I/O operation (download chunk, upload round) calls `record_bytes()`
//! to update shared state. Call `finish()` at the end to compute final results.

use crate::common;
use crate::progress::SpeedProgress;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Throttle interval for speed sampling (20 Hz max).
const SAMPLE_INTERVAL_MS: u64 = 50;

/// Shared state for a bandwidth test (download or upload).
///
/// All fields are thread-safe for use across multiple concurrent streams.
pub struct BandwidthLoopState {
    pub total_bytes: Arc<AtomicU64>,
    pub peak_bps: Arc<AtomicU64>,
    pub speed_samples: Arc<Mutex<Vec<f64>>>,
    pub start: Instant,
    pub last_sample_ms: Arc<AtomicU64>,
    pub estimated_total: u64,
    pub progress: Arc<SpeedProgress>,
}

/// Final result from a bandwidth test.
pub struct BandwidthResult {
    pub avg_bps: f64,
    pub peak_bps: f64,
    pub total_bytes: u64,
    pub duration_secs: f64,
    pub speed_samples: Vec<f64>,
}

impl BandwidthLoopState {
    /// Create a new bandwidth measurement state.
    #[must_use]
    pub fn new(estimated_total: u64, progress: Arc<SpeedProgress>) -> Self {
        Self {
            total_bytes: Arc::new(AtomicU64::new(0)),
            peak_bps: Arc::new(AtomicU64::new(0)),
            speed_samples: Arc::new(Mutex::new(Vec::new())),
            start: Instant::now(),
            last_sample_ms: Arc::new(AtomicU64::new(0)),
            estimated_total,
            progress,
        }
    }

    /// Record transferred bytes and update progress (throttled to 20 Hz).
    ///
    /// This is the single point where all expensive operations (bandwidth calc,
    /// peak tracking, sample recording, progress update) are throttled.
    pub fn record_bytes(&self, len: u64) {
        self.total_bytes.fetch_add(len, Ordering::Relaxed);

        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        let last_ms = self.last_sample_ms.load(Ordering::Relaxed);
        let should_sample =
            last_ms == 0 || elapsed_ms.saturating_sub(last_ms) >= SAMPLE_INTERVAL_MS;

        if should_sample {
            self.last_sample_ms.store(elapsed_ms, Ordering::Relaxed);
            self.sample_now();
        }
    }

    /// Take a speed sample and update progress (no throttle check — caller must gate).
    fn sample_now(&self) {
        let total = self.total_bytes.load(Ordering::Acquire);
        let elapsed = self.start.elapsed().as_secs_f64();
        let speed = common::calculate_bandwidth(total, elapsed);

        let current_peak = self.peak_bps.load(Ordering::Relaxed);
        if speed > current_peak as f64 {
            self.peak_bps.store(speed as u64, Ordering::Relaxed);
        }

        if let Ok(mut samples) = self.speed_samples.lock() {
            samples.push(speed);
        }

        let pct = (total as f64 / self.estimated_total as f64).min(1.0);
        self.progress.update(speed / 1_000_000.0, pct, total);
    }

    /// Compute final results from accumulated state.
    #[must_use]
    pub fn finish(&self) -> BandwidthResult {
        let total = self.total_bytes.load(Ordering::Relaxed);
        let peak = self.peak_bps.load(Ordering::Relaxed) as f64;
        let duration = self.start.elapsed().as_secs_f64();
        let samples = self.speed_samples.lock().unwrap().to_vec();
        let avg = common::calculate_bandwidth(total, duration);

        BandwidthResult {
            avg_bps: avg,
            peak_bps: peak,
            total_bytes: total,
            duration_secs: duration,
            speed_samples: samples,
        }
    }
}
