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
//! ## Modules
//!
//! - [`bin_errors`] — Binary-level error handling and exit codes
//! - [`cli`] — Command-line argument parsing with clap
//! - [`common`] — Shared utilities (bandwidth calculation, formatting, validation)
//! - [`config`] — Configuration management (CLI args + config file)
//! - [`domain`] — Core business logic (measurement, reporting, server, speedtest)
//! - [`download`] — Multi-stream download bandwidth measurement
//! - [`endpoints`] — Canonical speedtest endpoint derivation
//! - [`upload`] — Multi-stream upload bandwidth measurement
//! - [`error`] — Unified error types with categorization
//! - [`formatter`] — Output formatting (detailed, simple, JSON, CSV)
//! - [`grades`] — Quality grade system (A-F ratings)
//! - [`history`] — Persistent test result history
//! - [`http`] — HTTP client creation and IP discovery
//! - [`profiles`] — User profiles/roles (gamer, streamer, etc.)
//! - [`progress`] — Terminal progress bars and spinners
//! - [`servers`] — Server discovery, distance calculation, and selection
//! - [`task_runner`] — Test orchestration with template method pattern
//! - [`types`] — Shared data structures (Server, `TestResult`, etc.)

// Pedantic lints allowed at crate level — too noisy for a CLI bandwidth tester.
// Individual modules may re-enable specific lints where stricter checking is desired.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

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
