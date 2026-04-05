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
//! - [`cli`] — Command-line argument parsing with clap
//! - [`config`] — Configuration merging (CLI args + config file)
//! - [`download`] — Multi-stream download bandwidth measurement
//! - [`upload`] — Multi-stream upload bandwidth measurement
//! - [`error`] — Unified error types
//! - [`formatter`] — Output formatting (detailed, simple, JSON, CSV)
//! - [`history`] — Persistent test result history
//! - [`http`] — HTTP client creation and IP discovery
//! - [`progress`] — Terminal progress bars and spinners
//! - [`servers`] — Server discovery, distance calculation, and selection
//! - [`types`] — Shared data structures (Server, `TestResult`, etc.)
//! - [`upload`] — Upload bandwidth measurement

pub mod cli;
pub mod common;
pub mod config;
pub mod download;
pub mod error;
pub mod formatter;
pub mod history;
pub mod http;
pub mod progress;
pub mod servers;
pub mod test_runner;
pub mod types;
pub mod upload;
