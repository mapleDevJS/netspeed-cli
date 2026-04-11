# Phase 00 — DISCOVER: Existing Codebase Assessment

**Mode**: 3C — Full Audit  
**Date**: 2026-04-06  
**Agent**: DISCOVER

---

## 1. Architecture Map

### Module Responsibility Table

| Module | Lines | Responsibility | Dependencies |
|--------|-------|---------------|--------------|
| `main.rs` | ~40 | Binary entry point, error display, tokio runtime | clap, orchestrator, owo-colors |
| `lib.rs` | ~30 | Library root, module re-exports | All modules |
| `cli.rs` | ~130 | CLI argument parsing with clap derive, validation | clap, validate.rs (include!) |
| `validate.rs` | ~30 | IP address + timeout validation (shared with build.rs) | None (self-contained) |
| `config.rs` | ~110 | Config file loading + CLI/file merge | directories, serde, toml, clap |
| `types.rs` | ~100 | Data structures: Server, TestResult, ServerInfo, CsvOutput | serde, chrono |
| `error.rs` | ~80 | Unified error type (thiserror) | thiserror, reqwest, quick-xml, serde_json, csv |
| `orchestrator.rs` | ~200 | Speed test lifecycle orchestration | All runtime modules |
| `http.rs` | ~80 | HTTP client creation, IP discovery | reqwest, quick-xml, common |
| `servers.rs` | ~250 | Server fetch, distance calc, ping test, selection | reqwest, quick-xml, types |
| `download.rs` | ~150 | Multi-stream download bandwidth measurement | reqwest, tokio, common, progress |
| `upload.rs` | ~150 | Multi-stream upload bandwidth measurement | reqwest, tokio, common, progress |
| `test_runner.rs` | ~80 | Template method for bandwidth tests | tokio, config, http, servers, progress |
| `progress.rs` | ~130 | Progress bars, spinners, NO_COLOR support | indicatif, owo-colors, common |
| `history.rs` | ~200 | Persistent test result history (JSON file) | directories, serde_json, owo-colors |
| `common.rs` | ~90 | Shared utilities: bandwidth, formatting, validation | None |
| `formatter/mod.rs` | ~160 | Output format strategy (JSON/CSV/Simple/Detailed) | All formatter submodules, owo-colors |
| `formatter/ratings.rs` | ~180 | Rating functions (ping, speed, bufferbloat) | owo-colors, types |
| `formatter/sections.rs` | ~250 | Section-formatted output builders | owo-colors, common, ratings |
| `formatter/stability.rs` | ~60 | CV computation, percentile calculation | owo-colors |
| `formatter/estimates.rs` | ~120 | Usage check targets, download time estimates | owo-colors, common |
| `build.rs` | — | Build-time shell completion + man page generation | clap, clap_complete, clap_mangen |

**Total source files**: 21 Rust files (~2,400 LOC)  
**Test files**: 5 (3 integration, 1 e2e, 1 mock network)

### Architecture Pattern
- **Layered architecture** with template method pattern for test execution
- **Strategy pattern** for output formatting (`OutputFormat` enum)
- **Binary + library** dual crate (published to crates.io)
- **Async runtime**: tokio (full features)
- **HTTP**: reqwest with rustls-tls (no native-tls)

---

## 2. Baseline Quality

### Build Status
- `cargo build` — ✅ PASS (dev profile, 7.86s)
- `cargo test` — ✅ PASS (all tests passing)
- `cargo clippy` — Enforced in CI with `-D warnings`
- `cargo fmt --check` — Enforced in CI

### Known Issues from Configuration
- `deny.toml` ignores `RUSTSEC-2025-0119` (transitive via indicatif → number_prefix)
- Multiple versions warning: `winnow` 0.7 and 1.0 via toml_parser
- Several `#![allow(...)]` blocks for clippy casts (precision_loss, possible_truncation, cast_sign_loss) in download.rs, upload.rs, servers.rs, formatter modules

---

## 3. Test Inventory

| Test Type | Count | Location | Coverage Notes |
|-----------|-------|----------|----------------|
| Unit tests | ~85 | Inline in each module | Good coverage of pure functions |
| Integration tests | ~12 | `tests/integration_test.rs`, `tests/integration_upload_fetch_test.rs` | Mock server tests with wiremock |
| E2E tests | ~2 | `tests/e2e_test.rs` | Full flow tests |
| Mock network tests | ~2 | `tests/mock_network_test.rs` | Network mocking |
| Doc tests | 6 | `common.rs`, `servers.rs` | Doctests on public APIs |
| Benchmark tests | 1 suite | `benches/core_benchmarks.rs` | Criterion benchmarks |

**Total tests**: ~107 (all passing)  
**Coverage tooling**: cargo-llvm-cov in CI, uploaded to Codecov

---

## 4. Dependency Inventory

### Production Dependencies (18 total)

| Dependency | Version | License | Status |
|-----------|---------|---------|--------|
| clap | 4 | MIT/Apache-2.0 | ✅ Current |
| clap_complete | 4 | MIT/Apache-2.0 | ✅ Current |
| clap_mangen | 0.3 | MIT/Apache-2.0 | ✅ Current |
| reqwest | 0.12 | MIT/Apache-2.0 | ✅ Current |
| tokio | 1 | MIT | ✅ Current |
| serde | 1 | MIT/Apache-2.0 | ✅ Current |
| serde_json | 1 | MIT/Apache-2.0 | ✅ Current |
| quick-xml | 0.39 | MIT | ✅ Current |
| chrono | 0.4 | MIT/Apache-2.0 | ✅ Current |
| csv | 1 | MIT/Apache-2.0 | ✅ Current |
| indicatif | 0.18 | MIT | ✅ Current |
| owo-colors | 4 | MIT | ✅ Current |
| futures-util | 0.3.31 | MIT/Apache-2.0 | ✅ Current |
| directories | 6.0.0 | MIT | ✅ Current |
| toml | 1.1.2 | MIT/Apache-2.0 | ✅ Current |
| thiserror | 2 | MIT/Apache-2.0 | ✅ Current |

### Dev Dependencies
- wiremock 0.6.5 (mock HTTP server)
- serial_test 3.2 (serial test execution)
- tempfile 3.19 (temp directories)
- criterion 0.8 (benchmarking)

### Build Dependencies
- clap 4, clap_complete 4, clap_mangen 0.3 (same as runtime)

### Dependency Concerns
- **RUSTSEC-2025-0119**: number_prefix crate (transitive via indicatif) — ignored in deny.toml
- **winnow duplicate**: 0.7 and 1.0 versions via toml_parser — skipped in deny.toml
- **chrono**: Known timezone database concerns; version 0.4 is mature but has had yanked releases historically

---

## 5. Hot Spot Analysis

### Top 5 Files with Most Complexity
1. **`orchestrator.rs`** (~200 LOC) — Central orchestration, highest coupling
2. **`formatter/sections.rs`** (~250 LOC) — Output formatting, most code duplication across build_* functions
3. **`servers.rs`** (~250 LOC) — Network logic, distance calc, ping, server selection
4. **`history.rs`** (~200 LOC) — File I/O, JSON serialization, history display
5. **`download.rs` / `upload.rs`** (~150 LOC each) — Near-identical structure (DRY concern)

---

## 6. Change Risk Assessment

| Module | Risk | Reason |
|--------|------|--------|
| `orchestrator.rs` | HIGH | Central coordinator; changes ripple to all modules |
| `error.rs` | MEDIUM | Error type changes affect all modules |
| `types.rs` | MEDIUM | Shared data structures; breaking changes cascade |
| `download.rs` / `upload.rs` | LOW-MEDIUM | Mirrored structure; fix one, fix both |
| `formatter/*` | LOW | Well-separated, strategy pattern isolates changes |
| `common.rs` | LOW | Pure functions, well-tested |

---

## 7. CLI Interface Audit

### Commands
| Command | Type | Description |
|---------|------|-------------|
| (default) | Action | Run full speed test |
| `--list` | Flag | List available servers |
| `--history` | Flag | Display test history |
| `--generate-completion <SHELL>` | Flag | Generate shell completion script |

### Flags (all optional)

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--no-download` | bool | false | Skip download test |
| `--no-upload` | bool | false | Skip upload test |
| `--single` | bool | false | Single connection mode |
| `--bytes` | bool | false | Show MB/s instead of Mbit/s |
| `--simple` | bool | false | Minimal output |
| `--json` | bool | false | JSON output |
| `--csv` | bool | false | CSV output |
| `--csv-delimiter` | char | `,` | CSV delimiter (`,;|\t`) |
| `--csv-header` | bool | false | Print CSV headers |
| `--format` | enum | — | Output format (json/csv/simple/detailed) |
| `--server` | Vec\<String\> | [] | Server IDs to test against |
| `--exclude` | Vec\<String\> | [] | Server IDs to exclude |
| `--source` | String | — | Source IP address |
| `--timeout` | u64 | 10 | HTTP timeout in seconds |
| `--history` | bool | false | Show test history |
| `--generate-completion` | Shell | — | Generate shell completion |
| `--version` | — | — | Show version |
| `--help` | — | — | Show help |

### Shell Completions
- Bash, Zsh, Fish, PowerShell, Elvish (via clap_complete)

### Man Pages
- `netspeed-cli.1` generated via clap_mangen

### Help Text Quality
- ✅ Comprehensive `after_help` with 8 examples
- ✅ All flags have doc comments
- ✅ Version from CARGO_PKG_VERSION

### Config File Support
- Location: platform-specific via `directories` crate
- Format: TOML
- Fields: no_download, no_upload, single, bytes, simple, csv, csv_delimiter, csv_header, json, timeout
- Merge strategy: CLI args override file config override defaults

---

## 8. CI/CD Pipeline

### GitHub Actions Workflows

**`ci.yml`** (7 jobs):
| Job | Platform | Description |
|-----|----------|-------------|
| test | Ubuntu, macOS, Windows | cargo test + cargo build --release |
| clippy | Ubuntu | cargo clippy --all-targets --all-features -D warnings |
| fmt | Ubuntu | cargo fmt --check |
| doc | Ubuntu | cargo doc --no-deps -D warnings |
| audit | Ubuntu | cargo deny check |
| msrv | Ubuntu | cargo check on Rust 1.86 |
| coverage | Ubuntu | cargo llvm-cov → Codecov |
| brew-audit | macOS | Homebrew audit (tag-gated) |

**`release.yml`** (4 stages):
| Stage | Description |
|-------|-------------|
| verify-tag-on-main | Ensures tag is on master branch |
| build-binaries | 7 target platforms (x86_64/aarch64 Linux, macOS, Windows) |
| publish-github-release | GH release + SBOM + Homebrew formula update |
| publish-crates-io | cargo publish |

### Concurrency
- ✅ `cancel-in-progress: true` on CI
- ✅ Tag-gated release workflow

---

## 9. State of the Union (1-Page Summary)

**netspeed-cli v0.6.0** is a mature Rust CLI tool with solid architecture. It follows a layered design with clear separation between argument parsing (cli), orchestration (orchestrator), network operations (http/servers/download/upload), and output (formatter). The project has comprehensive CI coverage (test, clippy, fmt, doc, audit, msrv, coverage) and a multi-platform release pipeline (GitHub Releases + crates.io + Homebrew).

**Strengths:**
- Clean module boundaries with strategy pattern for output
- Strong CI/CD pipeline with security auditing (cargo-deny)
- Good test coverage across unit, integration, e2e, and doc tests
- Well-documented CLI with examples, shell completions, and man pages
- NO_COLOR support, colorized output with graceful degradation
- Persistent history feature with platform-specific storage

**Concerns:**
- download.rs and upload.rs are near-duplicate (~80% structural similarity) — DRY violation
- Config merge logic has a subtle bug: `merge_bool` uses `cli || file` which means CLI=false + file=true yields true (counterintuitive)
- RUSTSEC advisory ignored (number_prefix transitive dependency)
- Several clippy allow blocks for cast warnings — should audit for correctness
- `validate.rs` is `include!()`-ed which complicates IDE navigation
- History module uses `unsafe` env var manipulation in tests
- No env variable support for configuration (only CLI flags + config file)
- `--json` and `--csv` output goes to stdout, verbose output to stderr — inconsistent with some CLI conventions

**Verdict**: Codebase is in good shape (estimated B+ to A-). Ready for audit phases.
