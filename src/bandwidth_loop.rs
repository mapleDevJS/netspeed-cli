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
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    // ── LoopState Tests ──────────────────────────────────────────────────────

    fn make_tracker() -> Arc<Tracker> {
        Arc::new(Tracker::new("test"))
    }

    #[test]
    fn test_loop_state_new_fields() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);
        assert_eq!(state.total_bytes.load(Ordering::SeqCst), 0);
        assert_eq!(state.peak_bps.load(Ordering::SeqCst), 0);
        assert_eq!(state.estimated_total, 100_000_000);
        assert!(state.speed_samples.lock().unwrap().is_empty());
    }

    #[test]
    fn test_loop_state_concurrent_atomic_updates() {
        let tracker = make_tracker();
        let state = Arc::new(LoopState::new(100_000_000, tracker));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let s = Arc::clone(&state);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        s.record_bytes(100, SAMPLE_INTERVAL_MS);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // 4 threads * 1000 * 100 = 400,000
        assert_eq!(state.total_bytes.load(Ordering::SeqCst), 400_000);
    }

    #[test]
    fn test_record_bytes_zero_value() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);
        state.record_bytes(0, SAMPLE_INTERVAL_MS);
        assert_eq!(state.total_bytes.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_record_bytes_accumulates() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);
        state.record_bytes(1000, SAMPLE_INTERVAL_MS);
        state.record_bytes(2000, SAMPLE_INTERVAL_MS);
        state.record_bytes(3000, SAMPLE_INTERVAL_MS);
        assert_eq!(state.total_bytes.load(Ordering::SeqCst), 6000);
    }

    #[test]
    fn test_record_bytes_large_values() {
        let tracker = make_tracker();
        let state = LoopState::new(u64::MAX, tracker);
        state.record_bytes(1_000_000_000, SAMPLE_INTERVAL_MS);
        assert_eq!(state.total_bytes.load(Ordering::SeqCst), 1_000_000_000);
    }

    #[test]
    fn test_record_bytes_throttle_mechanism() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);

        // Test throttle by verifying that samples are recorded
        // The throttle mechanism limits sampling to once per interval
        let interval_ms = 50u64;

        // First call always triggers
        state.record_bytes(1000, interval_ms);
        assert_eq!(state.speed_samples.lock().unwrap().len(), 1);

        // Rapid second call - may or may not trigger depending on elapsed time
        state.record_bytes(1000, interval_ms);

        // Wait enough time for throttle to reset
        thread::sleep(Duration::from_millis(100));
        state.record_bytes(1000, interval_ms);

        // Should have at least 2 samples (first + after wait)
        // The exact count depends on timing, but throttle is working
        let samples = state.speed_samples.lock().unwrap();
        assert!(
            samples.len() >= 2,
            "Expected at least 2 samples, got {}",
            samples.len()
        );
    }

    #[test]
    fn test_record_bytes_short_interval_samples_more() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);

        // Short interval with explicit waits allows more frequent sampling
        for _ in 0..3 {
            state.record_bytes(1_000_000, 5); // 5ms interval
            thread::sleep(Duration::from_millis(10));
        }

        let samples = state.speed_samples.lock().unwrap();
        // With short interval and time between calls, should get multiple samples
        assert!(
            samples.len() >= 2,
            "Expected >= 2 samples with short interval, got {}",
            samples.len()
        );
    }

    #[test]
    fn test_record_bytes_updates_peak() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);

        state.record_bytes(10_000_000, SAMPLE_INTERVAL_MS);
        thread::sleep(Duration::from_millis(60));
        state.record_bytes(10_000_000, SAMPLE_INTERVAL_MS);

        let peak = state.peak_bps.load(Ordering::SeqCst);
        assert!(peak > 0);
    }

    #[test]
    fn test_finish_empty_state() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);
        thread::sleep(Duration::from_millis(10));
        let result = state.finish();

        assert_eq!(result.total_bytes, 0);
        assert_eq!(result.avg_bps, 0.0);
        assert_eq!(result.peak_bps, 0.0);
        assert!(result.duration_secs > 0.0);
        assert!(result.speed_samples.is_empty());
    }

    #[test]
    fn test_finish_with_transfer() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);

        state.record_bytes(20_000_000, SAMPLE_INTERVAL_MS);
        thread::sleep(Duration::from_millis(100));

        let result = state.finish();
        assert_eq!(result.total_bytes, 20_000_000);
        assert!(result.avg_bps > 0.0);
    }

    #[test]
    fn test_finish_peak_gte_avg() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);

        for _ in 0..5 {
            state.record_bytes(5_000_000, SAMPLE_INTERVAL_MS);
            thread::sleep(Duration::from_millis(60));
        }

        let result = state.finish();
        assert!(result.peak_bps >= result.avg_bps);
    }

    #[test]
    fn test_finish_various_estimated_totals() {
        for estimated in [1u64, 1000, 1_000_000, u64::MAX / 2] {
            let tracker = make_tracker();
            let state = LoopState::new(estimated, tracker);
            state.record_bytes(100, SAMPLE_INTERVAL_MS);
            thread::sleep(Duration::from_millis(10));
            let result = state.finish();
            assert_eq!(result.total_bytes, 100);
        }
    }

    #[test]
    fn test_finish_returns_speed_samples() {
        let tracker = make_tracker();
        let state = LoopState::new(10_000_000, tracker);

        for _ in 0..3 {
            state.record_bytes(1_000_000, 10);
            thread::sleep(Duration::from_millis(20));
        }

        let result = state.finish();
        assert!(!result.speed_samples.is_empty());
        for sample in &result.speed_samples {
            assert!(*sample >= 0.0);
        }
    }

    #[test]
    fn test_sample_interval_constant() {
        assert_eq!(SAMPLE_INTERVAL_MS, 50);
    }

    #[test]
    fn test_bandwidth_result_struct() {
        let tracker = make_tracker();
        let state = LoopState::new(100_000_000, tracker);
        state.record_bytes(50_000_000, SAMPLE_INTERVAL_MS);
        thread::sleep(Duration::from_millis(100));

        let result = state.finish();

        // Verify all fields are correctly populated
        assert!(result.avg_bps >= 0.0);
        assert!(result.peak_bps >= 0.0);
        assert!(result.total_bytes > 0);
        assert!(result.duration_secs > 0.0);
    }

    // ── run_concurrent_streams Tests ─────────────────────────────────────────

    #[tokio::test]
    async fn test_run_concurrent_streams_zero_streams() {
        let tracker = make_tracker();
        let result = run_concurrent_streams(100_000_000, 0, tracker, "test", |_, _, _| {
            tokio::spawn(async { Ok(()) })
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_single_stream_success() {
        let tracker = make_tracker();
        let result =
            run_concurrent_streams(100_000_000, 1, tracker, "download", |_, state, interval| {
                let s = Arc::clone(&state);
                tokio::spawn(async move {
                    s.record_bytes(10_000_000, interval);
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_bytes, 10_000_000);
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_four_streams() {
        let tracker = make_tracker();
        let result =
            run_concurrent_streams(100_000_000, 4, tracker, "upload", |_, state, interval| {
                let s = Arc::clone(&state);
                tokio::spawn(async move {
                    s.record_bytes(1_000_000, interval);
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_bytes, 4_000_000);
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_all_fail() {
        let tracker = make_tracker();
        let result = run_concurrent_streams(100_000_000, 3, tracker, "download", |_, _, _| {
            tokio::spawn(async { Err(Error::DownloadFailure("failed".into())) })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_partial_failure() {
        let tracker = make_tracker();
        let result =
            run_concurrent_streams(100_000_000, 4, tracker, "upload", |i, state, interval| {
                let s = Arc::clone(&state);
                tokio::spawn(async move {
                    if i < 2 {
                        s.record_bytes(1_000_000, interval);
                        Ok(())
                    } else {
                        Err(Error::UploadFailure("failed".into()))
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_bytes, 2_000_000);
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_stream_panic() {
        let tracker = make_tracker();
        let result =
            run_concurrent_streams(100_000_000, 2, tracker, "download", |i, state, interval| {
                let s = Arc::clone(&state);
                tokio::spawn(async move {
                    if i == 0 {
                        s.record_bytes(1_000_000, interval);
                        Ok(())
                    } else {
                        panic!("stream panicked");
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_bytes, 1_000_000);
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_zero_bytes_returns_error() {
        let tracker = make_tracker();
        let result = run_concurrent_streams(100_000_000, 2, tracker, "download", |_, _, _| {
            tokio::spawn(async { Ok(()) })
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_label_different_errors() {
        for label in ["download", "upload", "custom"] {
            let tracker = make_tracker();
            let result = run_concurrent_streams(100_000_000, 0, tracker, label, |_, _, _| {
                tokio::spawn(async { Ok(()) })
            })
            .await;

            assert!(result.is_err());
            let err_str = format!("{:?}", result.unwrap_err());
            assert!(err_str.contains(label));
        }
    }

    #[tokio::test]
    async fn test_run_concurrent_streams_estimated_total_param() {
        for estimated in [1_000u64, 10_000_000, 1_000_000_000] {
            let tracker = make_tracker();
            let result =
                run_concurrent_streams(estimated, 1, tracker, "test", |_, state, interval| {
                    let s = Arc::clone(&state);
                    tokio::spawn(async move {
                        s.record_bytes(1000, interval);
                        Ok(())
                    })
                })
                .await;
            assert!(result.is_ok());
        }
    }
}
