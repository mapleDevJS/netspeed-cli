# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Architecture**: `download.rs` refactored to use shared `BandwidthLoopState` instead of inline throttle logic (~60 lines of duplicated measurement state eliminated)
- **API stability**: `lib.rs` now documents a stable public API facade (`CliArgs`, `OutputFormatType`, `SpeedtestError`, `SpeedTestOrchestrator`, `Server`, `ServerInfo`, `TestResult`). Internal modules remain `pub` for integration tests but are marked as non-stable
- **API consistency**: Both `download_test()` and `upload_test()` now return `BandwidthResult` instead of tuple types `(f64, f64, u64, Vec<f64>)`
- **Dependencies**: Updated 6 dependencies to latest versions
  - `indicatif`: 0.17.11 → 0.18.4
  - `clap_mangen`: 0.2.33 → 0.3.0
  - `quick-xml`: 0.37.5 → 0.39.2
  - `toml`: 0.9.12 → 1.1.2
  - `criterion`: 0.5.1 → 0.8.2
  - `actions/upload-artifact`: 4 → 7

### Fixed

- docs.rs build failure: `build.rs` now skips file generation on docs.rs (read-only filesystem)
- Added `[package.metadata.docs.rs]` configuration to `Cargo.toml`
- Benchmark compatibility with criterion 0.8 (`std::hint::black_box`)
- `main.rs` imports updated to use stable public API re-exports instead of internal module paths

### Pending

- `actions/download-artifact`: 7 → 8 (requires manual merge — workflow file)

## [0.7.0] - 2026-04-07

### Added

- Dashboard UI: rich boxed layout with bar charts, sparkline history, and section dividers
- Overall connection rating display below dashboard header
- `--quiet` flag to suppress all progress output (JSON/CSV still go to stdout)
- `bandwidth_loop` module for shared download/upload measurement state
- Shell completion updates: added `dashboard` format and `--quiet` flag to all shells

### Changed

- Dashboard bar alignment: consistent `{:>8.2}` formatting for all speed values
- Dashboard latency bar: direct scale (proportional to ping value) instead of inverted
- Dashboard summary: restructured into separate "Download Summary" and "Upload Summary" sections
- Dashboard history: Unicode block chars for sparklines instead of emojis for better TTY support
- Homebrew formula SHA256 updated for v0.7.0 release

### Fixed

- Dashboard ASCII box header: proper dynamic width and `═` padding (was producing empty borders)
- Upload test assertion: HTTP 500 responses correctly return 0 total bytes
- `bandwidth_loop` module: added to `lib.rs` (was missing, caused build failure on CI)
- Man page and completions regenerated with updated CLI surface

## [0.6.0] - 2026-04-06

### Changed

- **Dependencies**: Updated 5 dependencies
  - `indicatif`: 0.17.11 → 0.18.4
  - `clap_mangen`: 0.2.33 → 0.3.0
  - `quick-xml`: 0.37.5 → 0.39.2
  - `toml`: 0.9.12 → 1.1.2
  - `criterion`: 0.5.1 → 0.8.2
- CI: `actions/upload-artifact` 4 → 7

### Fixed

- Benchmark compatibility with criterion 0.8 (`std::hint::black_box`)

## [0.5.1] - 2026-04-06

### Fixed

- docs.rs build failure: `build.rs` now skips file generation on docs.rs (read-only filesystem)
- Added `[package.metadata.docs.rs]` configuration to `Cargo.toml`

## [0.5.0] - 2026-04-06

### Fixed

- Release script: restored `main()` function call after refactoring
- Release workflow: stabilized PR merge flow from develop to master

## [0.4.0] - 2026-04-05

### Added

- Criterion benchmark suite for core functions
- Architecture documentation (`docs/architecture.md`)
- `CHANGELOG.md` with structured release notes
- CI: codecov-action v6, actions/checkout v6, download-artifact v7
- Test isolation: serial tests for environment-dependent tests (`history`, `progress`)

### Changed

- Adopted `thiserror` for unified error handling
- Refactored ratings, servers, and stability modules into focused helpers
- Split formatter module: separated pure formatting from I/O side-effects
- Throttled hot-path operations (download/upload bandwidth sampling) to 50ms intervals
- MSRV bumped to 1.86
- Coverage threshold adjusted to 85%
- Resolved all audit findings (score 8.2 → 9.2)

### Fixed

- Upload double-counting of uploaded bytes
- Unneeded unit expression in HTTP test
- rustfmt issues in CI
- Release workflow: split create and upload steps, added `--clobber` for idempotent releases
- Homebrew formula: pushed to master branch instead of develop
- Cargo audit: replaced rustsec/audit-check with direct `cargo audit` command

## [0.3.0] - 2026-04-05

### Added

- Latency under load measurement with background pinger during download/upload tests
- Jitter display alongside latency results
- Peak speed detection (maximum burst speed for download and upload)
- Connection quality rating (Excellent / Great / Good / Fair / Moderate / Poor)
- Data transferred and test duration in summary footer
- Real server distance using haversine formula with client geolocation from speedtest.net API
- `NO_COLOR` environment variable support for disabling colored output
- Crate-level documentation (`src/lib.rs`)
- Persistent test history module (`src/history.rs`)
- Progress bar module (`src/progress.rs`) with speed indicator and latency spinner
- Formatter helper unit tests (101 total tests)
- Mock network tests
- Homebrew CI audit job
- `develop` branch CI triggers

### Changed

- Bumped version from 0.2.2 to 0.3.0
- Updated Rust edition from 2021 to 2024
- Updated minimum Rust version from 1.70 to 1.85
- Corrected author email to `alexey.ivanov.js@gmail.com`
- Refactored `format_detailed` output from ~350 lines to ~80 lines with 8 sub-functions
- Added `#[must_use]` annotations to ~15 pure functions
- Added `# Errors` documentation to ~15 public `Result`-returning functions
- Resolved all clippy warnings (85 pedantic warnings → 0)
- Regenerated shell completions and man page for v0.3.0

### Removed

- `--mini` flag (dead code)
- `--secure` flag (dead code)
- `--no-pre-allocate` flag (dead code)
- `--share` flag and entire share feature (`src/share.rs` deleted)
- `share_url` field from `TestResult` and `CsvOutput`
- Unused dependencies: `md-5`, `digest`, `hex`
- Dead code: `TimeoutError`, `ServerListOutput`, `ServerListItem`, `calculate_server_distances`, `validate_ip`, `build_timeout_duration`, `build_base_url`

### Fixed

- Homebrew formula now passes `brew audit --strict --online`
- CI brew-audit job uses proper tap structure instead of standalone formula file
- Release workflow handles duplicate GitHub releases gracefully
- Test version assertion updated from `0.2.x` to `0.3.x`

### Distribution

- Published to [crates.io](https://crates.io/crates/netspeed-cli) as `netspeed-cli 0.3.0`
- Homebrew tap updated: `brew install mapledevjs/netspeed-cli/netspeed-cli`
- GitHub Release: [v0.3.0](https://github.com/mapleDevJS/netspeed-cli/releases/tag/v0.3.0)

## [0.2.2] - 2026-04-04

### Fixed

- Switched from `native-tls` to `rustls-tls` for cross-platform TLS compatibility

## [0.2.1] - 2026-04-04

### Fixed

- Linux compatibility fix

## [0.2.0] - 2026-04-04

### Added

- CI/CD pipeline with GitHub Actions
- More accurate speed measurement

## [0.1.3] - 2026-04-04

### Fixed

- Minor fixes

## [0.1.2] - 2026-04-04

### Fixed

- Download speed always returning 0

## [0.1.1] - 2026-04-04

### Fixed

- XML parsing error and output formatting

## [0.1.0] - 2026-04-04

### Added

- Initial release
- Download, upload, and latency testing against speedtest.net servers
- Multiple output formats: simple, JSON, CSV
- Shell completions (bash, zsh, fish, PowerShell, Elvish)
- Man page
- Server selection and listing
