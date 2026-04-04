use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Progress tracker for download/upload operations
pub struct ProgressTracker {
    total_chunks: AtomicUsize,
    completed_chunks: AtomicUsize,
    total_bytes: AtomicU64,
    show_dots: bool,
}

impl ProgressTracker {
    pub fn new(total_chunks: usize, show_dots: bool) -> Self {
        Self {
            total_chunks: AtomicUsize::new(total_chunks),
            completed_chunks: AtomicUsize::new(0),
            total_bytes: AtomicU64::new(0),
            show_dots,
        }
    }

    /// Record completion of a chunk
    pub fn add_chunk(&self, bytes: u64) {
        self.completed_chunks.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);

        if self.show_dots {
            tracing::info!(target: "progress", ".");
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
    }

    /// Get total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes.load(Ordering::Relaxed)
    }

    /// Get completion percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let completed = self.completed_chunks.load(Ordering::Relaxed);
        let total = self.total_chunks.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            completed as f64 / total as f64
        }
    }

    /// Print final newline after progress and log completion.
    pub fn finish(&self) {
        if self.show_dots {
            eprintln!();
        }
        tracing::debug!(
            chunks = self.completed_chunks.load(Ordering::Relaxed),
            bytes = self.total_bytes.load(Ordering::Relaxed),
            "Transfer complete"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_initializes_correctly() {
        let tracker = ProgressTracker::new(10, false);
        assert_eq!(tracker.total_chunks.load(Ordering::Relaxed), 10);
        assert_eq!(tracker.completed_chunks.load(Ordering::Relaxed), 0);
        assert_eq!(tracker.total_bytes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_add_chunk_increments_counters() {
        let tracker = ProgressTracker::new(10, false);
        tracker.add_chunk(1000);
        tracker.add_chunk(2000);

        assert_eq!(tracker.completed_chunks.load(Ordering::Relaxed), 2);
        assert_eq!(tracker.total_bytes.load(Ordering::Relaxed), 3000);
    }

    #[test]
    fn test_total_bytes_returns_accumulated() {
        let tracker = ProgressTracker::new(5, false);
        tracker.add_chunk(500);
        tracker.add_chunk(1500);

        assert_eq!(tracker.total_bytes(), 2000);
    }

    #[test]
    fn test_progress_returns_correct_fraction() {
        let tracker = ProgressTracker::new(10, false);
        tracker.add_chunk(100);
        tracker.add_chunk(200);
        tracker.add_chunk(300);

        assert!((tracker.progress() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_with_zero_total() {
        let tracker = ProgressTracker::new(0, false);
        assert!((tracker.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_with_no_chunks() {
        let tracker = ProgressTracker::new(5, false);
        assert!((tracker.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_with_all_completed() {
        let tracker = ProgressTracker::new(3, false);
        tracker.add_chunk(100);
        tracker.add_chunk(200);
        tracker.add_chunk(300);

        assert!((tracker.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_finish_outputs_newline_with_dots() {
        // When show_dots=true, finish should print newline
        // We can't easily capture stderr in unit tests, but we can verify no panic
        let tracker = ProgressTracker::new(2, true);
        tracker.add_chunk(100);
        tracker.add_chunk(200);
        tracker.finish(); // Should not panic
    }

    #[test]
    fn test_finish_no_newline_without_dots() {
        let tracker = ProgressTracker::new(2, false);
        tracker.add_chunk(100);
        tracker.finish(); // Should not panic
    }

    #[test]
    fn test_thread_safety_concurrent_adds() {
        use std::sync::Arc;
        use std::thread;

        let tracker = Arc::new(ProgressTracker::new(100, false));
        let mut handles = vec![];

        for _ in 0..10 {
            let t = Arc::clone(&tracker);
            handles.push(thread::spawn(move || {
                for _ in 0..10 {
                    t.add_chunk(100);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(tracker.completed_chunks.load(Ordering::Relaxed), 100);
        assert_eq!(tracker.total_bytes.load(Ordering::Relaxed), 10000);
    }
}
