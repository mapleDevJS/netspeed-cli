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
//! The command-line application is the primary interface. For Rust applications,
//! [`api`] is the only supported embedding interface. Other modules remain public
//! for compatibility and advanced use, but their APIs are unstable and may
//! change between releases.
//!
//! # Example
//!
//! ```no_run
//! use netspeed_cli::api::{run_speed_test, TestOptions};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let report = run_speed_test(TestOptions::default()).await?;
//! println!("Download: {:?} Mbps", report.download);
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Features
//!
//! - **Multi-format output**: dashboard, detailed, compact, simple, minimal, JSON, JSONL, CSV
//! - **Theming**: dark, light, high-contrast, monochrome with `NO_COLOR` support
//! - **User profiles**: gamer, streamer, remote-worker, power-user, casual
//! - **Quality grading**: A–F ratings for each metric and an overall score
//! - **Latency under load**: measures ping degradation during bandwidth saturation
//! - **Test history**: persistent local storage with backup and corruption recovery
//! - **TLS options**: custom CA certs, domain-restricted TLS, configurable TLS version
//!
//! ## Public modules
//!
//! [`api`] is the only supported embedding interface. All other modules remain
//! public for compatibility and advanced use, but are unstable implementation
//! details.
//!
//! - [`api`] — Supported quiet library entry point returning a test report
//! - [`bandwidth_loop`] — Inner bandwidth measurement loop with live progress
//! - [`bin_errors`] — Binary-level error handling and exit codes
//! - [`cli`] — Command-line argument parsing with clap
//! - [`common`] — Shared utilities (bandwidth calculation, formatting, validation)
//! - [`config`] — Configuration management (CLI args + config file)
//! - [`domain`] — Core business logic (measurement, reporting, server, speedtest)
//! - [`download`] — Multi-stream download bandwidth measurement
//! - [`endpoints`] — Canonical speedtest endpoint derivation
//! - [`error`] — Unified error types with categorization
//! - [`formatter`] — Output formatting (dashboard, detailed, compact, simple, JSON, CSV)
//! - [`grades`] — Quality grade system (A–F ratings)
//! - [`history`] — Persistent test result history with sparkline trends
//! - [`http`] — HTTP client creation and IP discovery
//! - [`http_client`] — Typed HTTP client abstraction
//! - [`logging`] — Structured JSON logging
//! - [`orchestrator`] — Top-level test orchestration and service wiring
//! - [`output`] — Output dispatch and rendering
//! - [`output_strategy`] — Format resolution from config and flags
//! - [`phase_registry`] — Phase registration and lookup
//! - [`phase_runner`] — Phase execution with template method pattern
//! - [`phases`] — Phase context and executor definitions
//! - [`profiles`] — User profiles/roles (gamer, streamer, etc.)
//! - [`progress`] — Terminal progress bars, spinners, and sparklines
//! - [`result_processor`] — Result aggregation and processing
//! - [`servers`] — Server discovery, distance calculation, and selection
//! - [`services`] — Service container for dependency injection
//! - [`storage`] — Abstract storage trait for test results
//! - [`task_runner`] — Test orchestration with retry and timeout
//! - [`terminal`] — Terminal capability detection (color, animation, width)
//! - [`test_config`] — Per-test configuration (retries, stream count)
//! - [`theme`] — Color theming (Dark, Light, HighContrast, Monochrome)
//! - [`types`] — Shared data structures (Server, TestResult, etc.)
//! - [`upload`] — Multi-stream upload bandwidth measurement

// Pedantic lints allowed at crate level — too noisy for a CLI bandwidth tester.
// Individual modules may re-enable specific lints where stricter checking is desired.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

pub mod api;
pub mod bandwidth_loop;
pub mod bin_errors;
pub mod cli;
pub mod common;
pub mod config;
pub mod domain;
pub mod download;
pub mod endpoints;
pub mod error;
pub mod formatter;
pub mod grades;
pub mod history;
pub mod http;
pub mod http_client;
#[doc(hidden)]
pub mod interrupt;
pub mod logging;
pub mod orchestrator;
pub mod output;
pub mod output_strategy;
pub mod phase_registry;
pub mod phase_runner;
pub mod phases;
pub mod profiles;
pub mod progress;
pub mod result_processor;
pub mod servers;
pub mod services;
pub mod storage;
pub mod task_runner;
pub mod terminal;
pub mod test_config;
pub mod theme;
pub mod types;
pub mod upload;

mod live;
