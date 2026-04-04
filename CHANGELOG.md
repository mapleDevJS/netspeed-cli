# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-04-04

### Added
- `SECURITY.md` with vulnerability reporting process
- Code coverage CI job using `cargo-llvm-cov` with Codecov upload
- 6 cross-compilation release targets (x86_64/aarch64 Linux, macOS, Windows, musl)
- Unit tests for `runner.rs` module
- `std::io::IsTerminal` usage replacing deprecated `atty` module

### Changed
- Replace `eprintln!` with `tracing::error!` for consistent error handling
- Replace `eprint!(".")` with `tracing::info!` for progress dots
- Raise `cargo-deny` confidence threshold from 0.8 to 0.9
- Expand `.gitignore` with OS/IDE artifacts, `.env` files, and benchmark output
- Expand Code of Conduct to reference Contributor Covenant v2.1
- Update `cli.rs` doc comments to remove implementation details from man page
- Fix `clippy::field-reassign-with-default` warnings across all test files
- Change `RUST_BACKTRACE` from `1` to `full` in CI

### Removed
- Unused `digest` crate dependency
- Inline `atty` compatibility module from `formatter.rs`
- Duplicated tests between unit and integration test suites
- Unused imports in `completions.rs` and `error.rs`

### Fixed
- CI cache key hash pattern for `Cargo.lock`
- Unused variable warning in `error.rs` test

### Added
- SHA-256 based hashing for share URL generation (replaced MD5)
- `thiserror` for cleaner error type definitions
- Doc comments for all public APIs
- Named constants for magic numbers in download/upload tests
- Integration tests with comprehensive coverage
- GitHub Actions CI/CD pipeline (build, test, clippy, fmt)
- Dependabot configuration for automated dependency updates
- `CONTRIBUTING.md` with contribution guidelines
- `CHANGELOG.md` for tracking changes
- `wiremock`-based HTTP integration tests for XML parsing and error handling
- `tracing` for structured logging with `env-filter` support
- `--verbose`/`-v` CLI flag for debug-level logging
- `cargo-deny` security and license auditing in CI
- CLI flag conflict validation (`--json` vs `--csv`)
- CLI `requires` validation (`--csv-header`, `--csv-delimiter` require `--csv`)

### Changed
- Split `main.rs` into separate `lib.rs` and `main.rs` for proper library/binary separation
- Improved error handling in download/upload tests (no longer silently ignoring errors)
- Updated repository URL from placeholder to actual URL
- Replaced `md-5` with `sha2` for better security practices
- Removed unused `ctrlc` dependency
- Replaced all `eprintln!` calls with `tracing` macros (`info!`, `warn!`, `error!`)
- Fixed GitHub Actions workflow to use correct `dtolnay/rust-toolchain` action
- Server selection now uses weighted scoring: 60% distance + 40% latency
- `--simple` mode now suppresses `tracing::info!` output (user-facing messages only)
- Removed `Cargo.lock` from `.gitignore` (standard for binary crates)
- Fixed `deny.toml` — removed invalid `[[sources.allow]]` section

### Fixed
- Download test now properly reports and handles stream errors
- Upload test now validates HTTP response status codes
- Ping test logic corrected to run when either download or upload is enabled
- Lib target in `Cargo.toml` now correctly points to `src/lib.rs`
- Fixed `needless_borrows_for_generic_args` clippy warning in formatter

## [0.1.0] - 2026-04-04

### Added
- Initial release
- Command-line interface for speedtest.net bandwidth testing
- Server discovery and selection by distance
- Concurrent download and upload speed testing
- Multiple output formats: simple, JSON, CSV
- Shell completion generation (Bash, Zsh, Fish, PowerShell, Elvish)
- Man page generation
- Speedtest Mini server support
- Share URL generation
- Progress dots during tests
- Source IP binding
- Timeout configuration

[Unreleased]: https://github.com/alexeyivanov/netspeed-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/alexeyivanov/netspeed-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alexeyivanov/netspeed-cli/releases/tag/v0.1.0
