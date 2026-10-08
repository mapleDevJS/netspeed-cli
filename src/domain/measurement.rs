//! Shared bandwidth measurement orchestration.
//!
//! The domain entry point and CLI use the same cancellation-safe implementation.
pub use crate::task_runner::run_bandwidth_test;
