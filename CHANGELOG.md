# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
