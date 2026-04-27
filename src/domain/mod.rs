//! Domain layer for netspeed-cli.
//!
//! This module contains the core business logic extracted from the broader
//! codebase into focused, cohesive domains.
//!
//! ## Architecture
//!
//! - [`speedtest`] — Full test lifecycle orchestration
//! - [`measurement`] — Bandwidth measurement (download/upload)
//! - [`server`] — Server discovery and selection
//! - [`reporting`] — Result assembly and grading
//!
//! ## Design Principles
//!
//! - Single responsibility per module
//! - Clear boundaries between domains
//! - Testable with minimal dependencies

pub mod measurement;
pub mod reporting;
pub mod server;
pub mod speedtest;

pub use crate::task_runner::TestRunResult;
pub use measurement::run_bandwidth_test;
pub use reporting::TestResultBuilder;
pub use server::{ServerDiscovery, select_best_server};
pub use speedtest::run_all_phases;
