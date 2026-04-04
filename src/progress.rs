use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

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
            eprint!(".");
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

    /// Print final newline after progress
    pub fn finish(&self) {
        if self.show_dots {
            eprintln!();
        }
    }
}

/// Create an Arc-wrapped progress tracker for sharing across threads
pub fn create_progress_tracker(
    total_chunks: usize,
    show_dots: bool,
) -> Arc<ProgressTracker> {
    Arc::new(ProgressTracker::new(total_chunks, show_dots))
}
