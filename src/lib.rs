//! # netspeed-cli
//!
//! A command-line internet bandwidth tester using speedtest.net servers.
//!
//! ## Overview
//!
//! This crate provides both a library and a binary (`netspeed-cli`) for
//! measuring download speed, upload speed, latency, jitter, and latency
//! under load. It connects to speedtest.net's server infrastructure to
//! perform real-world bandwidth tests.
//!
//! ## Public API
//!
//! The stable public API consists of:
//!
//! - [`SpeedTestOrchestrator`] — Main entry point for running speed tests
//! - [`Server`] — Speedtest server information
//! - [`TestResult`] — Complete test result with all metrics
//! - [`ServerInfo`] — Server metadata included in results
//! - [`SpeedtestError`] — Unified error type
//! - [`CliArgs`] — CLI argument definitions (for programmatic CLI construction)
//!
//! ## Example
//!
//! ```no_run
//! use netspeed_cli::{CliArgs, SpeedTestOrchestrator};
//! use clap::Parser;
//!
//! # async fn example() -> Result<(), netspeed_cli::SpeedtestError> {
//! let args = CliArgs::parse_from(["netspeed-cli", "--simple"]);
//! let orchestrator = SpeedTestOrchestrator::new(args)?;
//! orchestrator.run().await?;
//! # Ok(())
//! # }
//! ```

// ─── Module Declarations ─────────────────────────────────────────────
mod cli;
mod error;
mod orchestrator;
mod presentation;

// ─── Stable Public API ───────────────────────────────────────────────
pub use cli::CliArgs;
pub use cli::OutputFormatType;
pub use error::SpeedtestError;
pub use orchestrator::SpeedTestOrchestrator;
pub use types::{BandwidthMetrics, Server, ServerInfo, TestResult};

// ─── Internal Modules (pub for integration tests, not part of stable API) ─
// These modules are exposed for integration testing but are not considered
// part of the stable public API. Breaking changes may occur between versions.
//
// Module cohesion notes:
// - `bandwidth_loop`: bandwidth math + concurrent measurement state
// - `common`: input validation only (is_valid_ipv4)
// - `formatter/formatting`: formatting primitives (distance, data size, bar charts)
// - `geo`: Haversine distance formula (pure math)
// - `server_fetch`: HTTP/XML server discovery
// - `ping`: latency/jitter/packet-loss measurement
// - `servers`: backward-compat re-exports from geo, server_fetch, ping
pub mod bandwidth_loop;
pub mod common;
pub mod config;
pub mod download;
pub mod formatter;
pub mod geo;
pub mod history;
pub mod http;
pub mod ping;
pub mod progress;
pub mod server_fetch;
pub mod servers;
pub mod test_runner;
pub mod types;
pub mod upload;
