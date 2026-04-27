//! Full speed test lifecycle orchestration.
//!
//! This module provides the public API for running a complete speed test.
//! Implementation delegates to the phases module.

use crate::error::Error;
use crate::orchestrator::Orchestrator;
use crate::task_runner::TestRunResult;

/// Run all test phases in sequence.
///
/// This is the main entry point for running a complete speed test.
/// It executes phases in order: server discovery → IP discovery →
/// ping test → download test → upload test → results.
///
/// # Errors
///
/// Returns various [`Error`] types depending on which phase fails.
pub async fn run_all_phases(orch: &Orchestrator) -> Result<(), Error> {
    // Delegate to phases module - eliminates duplicate logic
    crate::phases::run_all_phases(orch).await
}

/// Phase results from all test phases.
pub type PhaseResults = (
    Option<(f64, f64, f64, Vec<f64>)>, // ping: (latency, jitter, packet_loss, samples)
    Option<TestRunResult>,             // download result
    Option<TestRunResult>,             // upload result
);
