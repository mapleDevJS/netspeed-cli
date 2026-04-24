//! Shared bandwidth measurement loop for download/upload tests.
//!
//! Eliminates duplication between `download.rs` and `upload.rs` by providing:
//! - [`LoopState`] — unified state for throttled speed sampling,
//!   peak tracking, progress bar updates, and atomic byte counting
//! - [`run_concurrent_streams`] — shared spawn/collect/report pattern
//!   that both download and upload tests delegate to
//!
//! Each I/O operation (download chunk, upload round) calls `record_bytes()`
//! to update shared state. Call `finish()` at the end to compute final results.

use crate::common;
use crate::error::Error;
use crate::progress::Tracker;
use crate::terminal;
use crate::test_config::TestConfig;
use owo_colors::OwoColorize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

/// Throttle interval for speed sampling (20 Hz max).
pub const SAMPLE_INTERVAL_MS: u64 = 50;

/// Shared state for a bandwidth test (download or upload).
///
/// All fields are thread-safe for use across multiple concurrent streams.
pub struct LoopState {
    pub total_bytes: Arc<AtomicU64>,
    pub peak_bps: Arc<AtomicU64>,
    pub speed_samples: Arc<Mutex<Vec<f64>>>,
    pub start: Instant,
    pub last_sample_ms: Arc<AtomicU64>,
    pub estimated_total: u64,
    pub progress: Arc<Tracker>,
}

/// Final result from a bandwidth test.
pub struct BandwidthResult {
    pub avg_bps: f64,
    pub peak_bps: f64,
    pub total_bytes: u64,
    pub duration_secs: f64,
    pub speed_samples: Vec<f64>,
}

impl LoopState {
    /// Create a new bandwidth measurement state.
    #[must_use]
    pub fn new(estimated_total: u64, progress: Arc<Tracker>) -> Self {
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
    ///
    /// Note: Uses cached sample_interval to avoid repeated TestConfig::default() calls.
    pub fn record_bytes(&self, len: u64, sample_interval_ms: u64) {
        // Release ensures writes are visible to the final Acquire load in finish()
        self.total_bytes.fetch_add(len, Ordering::Release);

        let elapsed_ms = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let last_ms = self.last_sample_ms.load(Ordering::Relaxed);
        let should_sample =
            last_ms == 0 || elapsed_ms.saturating_sub(last_ms) >= sample_interval_ms;

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

        // Safe: peak_bps stores bits-per-second; even 100 Gbps = 1e11, well under 2^53.
        let current_peak = self.peak_bps.load(Ordering::Relaxed) as f64;
        if speed > current_peak {
            let peak_u64 = speed.clamp(0.0, u64::MAX as f64) as u64;
            // Release pairs with the Acquire load in finish()
            self.peak_bps.store(peak_u64, Ordering::Release);
        }

        if let Ok(mut samples) = self.speed_samples.lock() {
            samples.push(speed);
        }

        // Safe: total and estimated_total are byte counts from a test lasting seconds;
        // they cannot approach 2^53 (~9 PB) where f64 loses precision.
        let pct = (total as f64 / self.estimated_total as f64).min(1.0);
        self.progress.update(speed / 1_000_000.0, pct, total);
    }

    /// Compute final results from accumulated state.
    #[must_use]
    pub fn finish(&self) -> BandwidthResult {
        // Acquire pairs with the Release fetch_add/stores to see all writes
        let total = self.total_bytes.load(Ordering::Acquire);
        // Safe: peak_bps is bits/sec; even 100 Gbps = 1e11, well under 2^53.
        let peak = self.peak_bps.load(Ordering::Acquire) as f64;
        let duration = self.start.elapsed().as_secs_f64();
        // Graceful fallback: if lock is poisoned (thread panicked), return empty samples
        let samples = self
            .speed_samples
            .lock()
            .map(|g| g.to_vec())
            .unwrap_or_default();
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

/// Run a bandwidth test using multiple concurrent streams.
///
/// This is the shared spawn/collect/report pattern used by both download
/// and upload tests. It:
/// 1. Creates a [`LoopState`] for the test
/// 2. Spawns `stream_count` tasks via `spawn_fn`
/// 3. Collects results, logging any task panics
/// 4. Returns a [`BandwidthResult`] (zeroed if all tasks failed)
///
/// The `spawn_fn` closure receives the stream index and a shared reference
/// to the loop state. Each call should create and return a `JoinHandle<()>`
/// that performs I/O and calls [`LoopState::record_bytes`] for each
/// transferred chunk.
///
/// # Arguments
/// * `estimated_total` — Estimated total bytes for progress bar initialization
/// * `stream_count` — Number of concurrent streams to spawn
/// * `progress` — Shared progress bar for the test phase
/// * `spawn_fn` — Closure that creates one stream's async task
///
/// # Panics
///
/// Individual task panics are caught and logged; they do not propagate.
#[must_use = "the BandwidthResult should be used to report test outcomes"]
pub async fn run_concurrent_streams(
    estimated_total: u64,
    stream_count: usize,
    progress: Arc<Tracker>,
    label: &str,
    mut spawn_fn: impl FnMut(usize, Arc<LoopState>, u64) -> tokio::task::JoinHandle<Result<(), Error>>,
) -> Result<BandwidthResult, Error> {
    let config = TestConfig::default();
    let sample_interval_ms = config.sample_interval_ms;
    let state = Arc::new(LoopState::new(estimated_total, progress));

    let mut handles = Vec::with_capacity(stream_count);
    for i in 0..stream_count {
        handles.push(spawn_fn(i, Arc::clone(&state), sample_interval_ms));
    }

    // Collect results — log any task panics so failures aren't silently swallowed.
    let mut any_succeeded = false;
    let mut first_error: Option<Error> = None;
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(Ok(())) => any_succeeded = true,
            Ok(Err(err)) => {
                let msg = format!("Warning: {label} stream {i} failed: {err}");
                if terminal::no_color() {
                    eprintln!("\n{msg}");
                } else {
                    eprintln!("\n{}", msg.yellow().bold());
                }
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
            Err(e) => {
                let msg = format!("Warning: {label} stream {i} failed: {e}");
                if terminal::no_color() {
                    eprintln!("\n{msg}");
                } else {
                    eprintln!("\n{}", msg.yellow().bold());
                }
                if first_error.is_none() {
                    first_error = Some(Error::context(format!("{label} stream {i} panicked: {e}")));
                }
            }
        }
    }

    if !any_succeeded {
        return Err(
            first_error.unwrap_or_else(|| Error::context(format!("all {label} streams failed")))
        );
    }

    let result = state.finish();
    if result.total_bytes == 0 {
        return Err(first_error.unwrap_or_else(|| match label {
            "download" => {
                Error::DownloadFailure("test completed without transferring data".to_string())
            }
            "upload" => {
                Error::UploadFailure("test completed without transferring data".to_string())
            }
            _ => Error::context(format!("{label} test completed without transferring data")),
        }));
    }

    Ok(result)
}
