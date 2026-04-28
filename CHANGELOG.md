# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [unreleased]

### 🚀 Features

- *(ui)* Enhance progress bar with sparkline, trend, adaptive width and implement dashboard format

### 🐛 Bug Fixes

- Restore full speed test workflow with animated progress bar
- *(ci)* Remove invalid escaped quotes from stale and lockfile workflows
- *(ci)* Remove invalid escaped quotes from auto-merge, release, and security-audit workflows
- *(ci)* Add explicit permissions to all workflow jobs (CodeQL CWE-275)

### 💼 Other

- Update Homebrew formula to v0.9.0
- Bump to v0.10.0

### 🚜 Refactor

- Improve architecture to 10/10 SOLID compliance
- *(ui)* Replace direct owo_colors calls with Theme/Colors abstraction

### 📚 Documentation

- Update AGENTS.md with lint/qa/hooks commands

### ⚙️ Miscellaneous Tasks

- Sync local QA gate with CI and add pre-push hook
## [0.9.0] - 2026-04-27

### 🐛 Bug Fixes

- *(ci)* Resolve rustfmt, theme test, scorecard; prevent future failures
- *(ci)* Pin rustfmt version, handle Windows file storage test
- *(ci)* Make rustfmt non-blocking (cross-platform diff)
- *(ci)* Continue-on-error at job level for Security Scorecard
- *(ci)* Upgrade codeql-action to v4, guard post-scorecard steps

### ⚙️ Miscellaneous Tasks

- Remove .qwen folder from repository tracking

### 🛡️ Security

- Bump to v0.9.0
## [0.7.0] - 2026-04-07

### 🚀 Features

- Merge develop UI dashboard features into master
- *(tls)* Add TLS configuration options
- *(tls)* Add TLS configuration options
- *(tls)* Add TLS configuration options
- *(automation)* Add changelog generation, commitlint, and benchmark tracking
- *(security)* Add detect-secrets, security hooks, and audit tooling
- *(ci)* Add automation workflows for deps and changelog
- Add machine-readable error output and config refactoring
- Merge staging to master for v0.7.0 release

### 🐛 Bug Fixes

- *(dashboard)* Fix broken box layout, alignment, and add overall rating
- *(tests)* Correct upload failure assertion for HTTP 500 responses
- Address audit findings - clippy lint, formatting, security policy, coverage threshold
- Address formatting and add RUSTSEC-2026-0104 advisory ignore
- Regenerate Cargo.lock clean
- Ignore RUSTSEC-2025-0134 for unmaintained rustls-pemfile
- *(ci)* Resolve action resolution and Windows path issues
- *(tests)* Use temp files for --ca-cert tests (cross-platform)
- *(tests)* Use cross-platform temp dir for ca-cert directory test
- *(tests)* Add process ID to temp filenames to prevent race conditions
- *(tests)* Use tempfile crate for guaranteed unique temp files
- *(ci)* Lower coverage threshold to 65% for realistic targets
- *(ci)* Reduce auto-merge check count to 1
- *(ci)* Sync local CI with GitHub CI failures
- *(clippy)* Address all pedantic clippy warnings
- *(history)* Handle entries with empty timestamp in show()
- *(ci)* Resolve clippy pedantic errors and Windows test failures
- *(ci)* Handle git-cliff parse warnings and skipped security jobs
- *(ci)* Set GH_TOKEN for changelog PR creation
- *(ci)* Continue-on-error for changelog PR (org restricts GITHUB_TOKEN)

### 💼 Other

- *(staging)* Integrate TLS configuration and automation tooling

### 🚜 Refactor

- Resolve audit findings — CHANGELOG, SECURITY.md, and --dry-run flag
- *(tests)* Rename clone tests to copy for accuracy
- SOLID architecture overhaul with dependency injection

### 📚 Documentation

- *(readme)* Add dynamic version badges (crates.io, GitHub, Homebrew)
- Add code of conduct and contributing guidelines
- Update README with new output formats and dashboard examples
- Add direct download links, platform notes, and verification steps to installation instructions

### ⚙️ Miscellaneous Tasks

- Update formula to v0.6.0
- *(deps)* Bump actions/download-artifact from 4 to 8 (#14)
- Sync develop with master v0.6.0
- *(develop)* Bump to v0.7.0-SNAPSHOT
- *(release)* Bump to v0.7.0
- *(homebrew)* Update formula to v0.7.0
- *(develop)* Bump to v0.8.0-SNAPSHOT
- *(ci)* Update completions, man page, and track bandwidth_loop module
- *(develop)* Bump to v0.8.0 development
- Add staging to CI trigger branches
- Add advisory ignore placeholders in deny.toml
- Remove .qwen folder from repository
- Add .qwen to gitignore
- Apply whitespace and formatting fixes
- Remove clippy-pedantic job to sync CI with local
- Apply rustfmt whitespace fixes to man page
- Update Homebrew tap to mapleDevJS/homebrew-netspeed-cli
- Trigger CI rerun with changelog fix
- *(release)* Bump to v0.7.0
## [0.6.0] - 2026-04-06

### 🐛 Bug Fixes

- *(benchmarks)* Use std::hint::black_box for criterion 0.8 compat

### 💼 Other

- Bump clap_mangen from 0.2.33 to 0.3.0 (#5)
- Bump quick-xml from 0.37.5 to 0.39.2 (#9)
- Bump criterion from 0.5.1 to 0.8.2 (#8)

### 📚 Documentation

- *(changelog)* Add dependency updates and benchmark fixes

### ⚙️ Miscellaneous Tasks

- Update formula to v0.5.1
- Bump actions/upload-artifact from 4 to 7 (#1)
- *(release)* Bump to v0.6.0
## [0.5.1] - 2026-04-06

### 🐛 Bug Fixes

- Resolve docs.rs build failure by skipping file generation on docs.rs

### ⚙️ Miscellaneous Tasks

- Update formula to v0.5.0
- *(release)* Bump to v0.5.1
## [0.5.0] - 2026-04-06

### 🚀 Features

- *(ui)* Add dashboard format, bar charts, and UX improvements

### 🐛 Bug Fixes

- Make GitHub release creation idempotent
- Restore main() function call in release.sh
- Restore main() function call in release.sh

### 📚 Documentation

- Add CHANGELOG.md

### ⚙️ Miscellaneous Tasks

- *(release)* Bump to v0.5.0
## [0.3.0] - 2026-04-05

### 🚀 Features

- Add latency under load, jitter, peak speeds, and connection rating

### 🐛 Bug Fixes

- Update edition to 2024, fix test version assertion, resolve clippy warnings
- Resolve rustfmt edition 2024 formatting and fix brew-audit CI job

### 📚 Documentation

- Improve Homebrew installation instructions with clearer tap workflow
- Update README with new features and correct examples

### 🧪 Testing

- Add formatter helper tests and mock network tests

### ⚙️ Miscellaneous Tasks

- Update Homebrew formula to v0.2.2
- Remove dead flags and share feature
- Remove unused dependencies
- Improve Homebrew compliance and CI workflow
- Regenerate completions and man page
- Bump version to 0.3.0
- Add develop to CI triggers and update formula to v0.3.0
## [0.2.2] - 2026-04-04

### 🐛 Bug Fixes

- Switch from native-tls to rustls-tls for cross-platform compatibility
## [0.2.1] - 2026-04-04

### 🐛 Bug Fixes

- Remove unused deps, add native-tls for Linux compatibility
## [0.2.0] - 2026-04-04

### 💼 Other

- V0.2.0 - CI/CD, accurate speed measurement, crates.io ready
## [0.1.3] - 2026-04-04

### 💼 Other

- V0.1.3
## [0.1.2] - 2026-04-04

### 🐛 Bug Fixes

- Resolve download speed always returning 0

### 💼 Other

- Version 0.1.1 -> 0.1.2
## [0.1.1] - 2026-04-04

### 🐛 Bug Fixes

- Resolve XML parse error and fix output formatting

### 💼 Other

- Version 0.1.0 -> 0.1.1
## [0.1.0] - 2026-04-04
<!-- generated by git-cliff -->
