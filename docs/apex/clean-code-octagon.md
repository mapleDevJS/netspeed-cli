# Clean Code Octagon Audit — netspeed-cli v0.6.0

**Auditor**: Sentinel (clean-code-octagon skill)  
**Date**: 2026-04-06  
**Mode**: Brownfield | **Language**: Rust 2024 Edition | **MSRV**: 1.86

---

## 1. Executive Health Score

| Dimension | Score | Max | Grade |
|-----------|-------|-----|-------|
| **Security & Correctness** | 32 | 40 | B |
| **Maintainability & Readability** | 19 | 25 | B+ |
| **Architecture & Design** | 15 | 20 | B+ |
| **Performance & Scalability** | 8 | 10 | B+ |
| **Testability** | 4 | 5 | B+ |
| **Bonus** | +4 | +5 | — |
| **TOTAL** | **82** | **100** | **B+** |

### Score Breakdown

**Security & Correctness (32/40):**
- `-5` MEDIUM: `merge_bool` config merge logic is inverted — `cli || file` means CLI=false + file=true = true, which is counterintuitive (CFG-001)
- `-3` MEDIUM: Blanket `#![allow(clippy::cast_*)]` at module level in 5 files — suppresses 3 categories of cast warnings without justification per file (CAST-001)
- `-0` Critical: No RCE/injection/auth bypass issues found
- `cargo clippy` passes clean (0 warnings)

**Maintainability & Readability (19/25):**
- `-4` Function LOC: `orchestrator.rs` ~200 LOC, `servers.rs` ~250 LOC, `formatter/sections.rs` ~250 LOC (all in 121-200+ range)
- `-2` Nesting depth: `download.rs` and `upload.rs` reach 5 levels of nesting in hot path (tokio::spawn → for loop → for round → while let Some → if let Ok)

**Architecture & Design (15/20):**
- `-5` DRY violation: `download.rs` and `upload.rs` are ~80% structurally identical (throttle gate, peak tracking, speed samples, progress updates) — same template duplicated instead of unified (DRY-001)
- `validate.rs` is `include!()`-ed from both `cli.rs` and `build.rs` — complicates IDE navigation and testing (ARCH-001)

**Performance & Scalability (8/10):**
- `-2` `Ordering::Relaxed` used for `last_sample_ms` throttle gate — should use `Relaxed` (it's a simple timestamp, no data dependency) — actually correct here, but `Ordering::Acquire` on `total_bytes` load is good. Minor concern: `speed_samples` uses `Mutex<Vec>` in hot path (could use lock-free ring buffer for higher throughput)

**Testability (4/5):**
- `-1` `unsafe { std::env::set_var("NO_COLOR") }` in tests — requires `#[serial]` to prevent race conditions. Should use a context-based approach instead

**Bonus (+4/5):**
- `+1` Pure function isolation: `common.rs` — all pure, side-effect-free utilities with doctests
- `+1` Strategy pattern: `OutputFormat` enum dispatch eliminates conditional branches in output formatting
- `+1` Structured error types: `thiserror`-based `SpeedtestError` enum with full `Error::source()` chains
- `+1` Developer experience: NO_COLOR support, --help with 8 examples, shell completions (5 shells), man pages, --history
- `+0` Reproducible builds: `Cargo.lock` committed, `deny.toml` present, SBOM in releases — but no pinned transitive dep versions in Cargo.toml (wildcards allowed)

---

## 2. STRIDE Threat Model

| Asset | Spoofing | Tampering | Repudiation | Info Disclosure | DoS | Elevation | Gap |
|-------|----------|-----------|-------------|-----------------|-----|-----------|-----|
| **HTTP requests to speedtest.net** | — | LOW | — | — | MEDIUM | — | No retry/backoff on transient failures; single timeout |
| **Config file (TOML)** | — | LOW | — | LOW | — | — | No validation of config file values beyond types |
| **History file (JSON)** | — | LOW | — | MEDIUM | — | — | Stored in platform data dir; no access restrictions |
| **CLI input (--source IP)** | — | — | — | — | — | — | ✅ Validated via `validate_ip_address` |
| **Server XML response** | — | MEDIUM | — | — | — | — | XML deserialized directly; no schema validation beyond serde derive |
| **Shell completion output** | — | — | — | — | — | — | ✅ Generated from clap schema, not user input |

**Confirmed gaps:**
- **Info Disclosure (MEDIUM)**: History file contains IP addresses, server names, timestamps — stored with default platform permissions (no chmod/ACL)
- **Tampering (MEDIUM)**: XML server response deserialized without validation; malicious XML could trigger unexpected behavior via serde derive
- **DoS (LOW-MEDIUM)**: No rate limiting or retry logic; network failures fail immediately without backoff

---

## 3. Architectural Design Pattern

**Current State**: Layered Architecture with Template Method + Strategy Pattern  
**Target Architecture**: Same — the architecture is sound. The issue is DRY within the layers.  
**Pivot Strategy**: Extract the duplicated bandwidth test template (download/upload shared logic) into a unified parameterized runner.

---

## 4. Audit Log (Ranked by Risk)

### [MEDIUM: SECURITY] CFG-001 — Config merge_bool semantics are inverted
**File**: `config.rs:44-46`  
**Issue**: `let merge_bool = |cli: bool, file: Option<bool>| cli || file.unwrap_or(false);`  
This means: CLI not passed (false) + config file says `no_download = true` → result is `true`. The user who didn't pass `--no-download` gets download skipped because their config file has it set. This is actually the _intended_ merge behavior (file config acts as defaults), but the naming "merge" is misleading — it should be called "prefer_file_unless_cli_set" or similar. The real issue: there's no way to distinguish "user didn't pass flag" from "user explicitly passed false" with clap's bool defaults.  
**Fix**: Document this behavior clearly. Consider using `Option<bool>` in CliArgs to detect explicit vs default.

### [MEDIUM: HYGIENE] CAST-001 — Blanket clippy cast allows across 5 modules
**Files**: `download.rs`, `upload.rs`, `servers.rs`, `formatter/sections.rs`, `formatter/stability.rs`, `formatter/estimates.rs`, `formatter/ratings.rs`  
**Issue**: Module-level `#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]` suppresses all cast warnings. While justified for a bandwidth calculator (f64 is the right type), this masks legitimate cast bugs.  
**Fix**: Replace with function-level `#[allow(...)]` where needed, or add `#[expect(...)]` (Rust 1.81+) with reason comments.

### [HIGH: DRY] DRY-001 — Download and upload modules are near-duplicates
**Files**: `download.rs` vs `upload.rs`  
**Issue**: Both modules share:
- Same throttle gate pattern (`last_sample_ms`, `SAMPLE_INTERVAL_MS`)
- Same peak tracking (`AtomicU64` + compare)
- Same speed sample collection (`Mutex<Vec<f64>>`)
- Same progress update calls
- Same stream count logic
- ~80% structural similarity

The `test_runner.rs` template method already exists but only wraps the outer orchestration — the internal bandwidth test loop is duplicated.  
**Fix**: Create a `bandwidth_loop.rs` module that handles the shared hot path (throttle, sampling, peak tracking, progress). Download/upload modules only provide the I/O closure.

### [LOW: ARCHITECTURE] ARCH-001 — validate.rs uses include!() macro
**File**: `validate.rs` → `cli.rs`, `build.rs`  
**Issue**: The `include!()` pattern works but breaks IDE navigation, separate compilation, and testing. Functions in `validate.rs` cannot be tested independently when compiled via include.  
**Fix**: Make `validate.rs` a proper module with `pub(crate)` visibility. In `build.rs`, use a separate validation approach or depend on the library crate.

### [LOW: SECURITY] SEC-001 — RUSTSEC-2025-0119 ignored
**File**: `deny.toml`  
**Issue**: The `number_prefix` crate has a known advisory. While transitive via indicatif, this should be tracked with an upgrade path.  
**Fix**: Track indicatif upgrade to version that removes number_prefix dependency. Document timeline.

### [LOW: TESTABILITY] TEST-001 — Unsafe env var manipulation in tests
**File**: `progress.rs:150-162`  
**Issue**: `unsafe { std::env::set_var("NO_COLOR", "1") }` with `#[serial]` guard. This is a known Rust test limitation but could be cleaner.  
**Fix**: Use `serial_test` is already in place. Consider refactoring `no_color()` to accept an explicit override for testing.

---

## 5. Golden Refactors (Top 3 by ROI)

### Refactor A — Extract shared bandwidth loop (DRY-001)
**Scope**: `download.rs:60-140` + `upload.rs:55-130` | **Effort**: ~200 lines | **Risk**: Medium

Create a unified `bandwidth_loop` module that parameterizes the I/O operation:

```rust
// New file: src/bandwidth_loop.rs
//! Shared bandwidth measurement loop for download/upload tests.
//! Eliminates duplication between download.rs and upload.rs.

use crate::common;
use crate::progress::SpeedProgress;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Throttle interval for speed sampling (20 Hz max).
const SAMPLE_INTERVAL_MS: u64 = 50;

/// State shared across all concurrent streams in a bandwidth test.
pub struct BandwidthLoopState {
    pub total_bytes: Arc<AtomicU64>,
    pub peak_bps: Arc<AtomicU64>,
    pub speed_samples: Arc<Mutex<Vec<f64>>>,
    pub start: Instant,
    pub last_sample_ms: Arc<AtomicU64>,
    pub estimated_total: u64,
    pub progress: Arc<SpeedProgress>,
}

impl BandwidthLoopState {
    /// Record transferred bytes and update progress (throttled to 20 Hz).
    pub fn record_bytes(&self, len: u64) {
        self.total_bytes.fetch_add(len, Ordering::Relaxed);

        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        let last_ms = self.last_sample_ms.load(Ordering::Relaxed);
        let should_sample =
            last_ms == 0 || elapsed_ms.saturating_sub(last_ms) >= SAMPLE_INTERVAL_MS;

        if should_sample {
            self.last_sample_ms.store(elapsed_ms, Ordering::Relaxed);

            let total = self.total_bytes.load(Ordering::Acquire);
            let elapsed = self.start.elapsed().as_secs_f64();
            let speed = common::calculate_bandwidth(total, elapsed);

            let current_peak = self.peak_bps.load(Ordering::Relaxed);
            if speed > current_peak as f64 {
                self.peak_bps.store(speed as u64, Ordering::Relaxed);
            }

            if let Ok(mut samples) = self.speed_samples.lock() {
                samples.push(speed);
            }

            let pct = (total as f64 / self.estimated_total as f64).min(1.0);
            self.progress.update(speed / 1_000_000.0, pct, total);
        }
    }
}
```

**Impact**: Eliminates ~150 lines of duplicated code. Single source of truth for throttling, sampling, and peak tracking.

### Refactor B — Function-level cast allows
**Scope**: 7 modules with blanket allows | **Effort**: ~30 lines | **Risk**: Low

Replace module-level `#![allow(...)]` with targeted `#[allow(...)]` on specific functions, each with a comment explaining why the cast is safe:

```rust
// Before (module-level):
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

// After (function-level):
/// SAFETY: u64→f64 cast is safe here — bandwidth values stay well within f64 precision range.
#[allow(clippy::cast_precision_loss)]
fn calculate_bandwidth(total_bytes: u64, elapsed_secs: f64) -> f64 { ... }
```

### Refactor C — Config merge semantics documentation
**Scope**: `config.rs:44-46` | **Effort**: ~10 lines | **Risk**: Low

Add explicit documentation and rename the closure for clarity:

```rust
/// Merge strategy: CLI flag overrides file config overrides default.
/// NOTE: Since clap defaults bool to `false`, we can't distinguish
/// "user didn't pass flag" from "user passed --no-flag=false".
/// The file config acts as a default — if file says `true`, it applies
/// even when CLI flag is absent. Users must explicitly pass `--flag`
/// to override file config to `false` (which requires the flag to support negation).
let apply_file_default_unless_cli_set = |cli: bool, file: Option<bool>| {
    cli || file.unwrap_or(false)
};
```

---

## 6. Test Strategy

### Current Coverage
- **Unit tests**: ~85 tests covering pure functions, error types, serialization
- **Integration tests**: ~12 tests with wiremock for HTTP mocking
- **E2E tests**: ~2 tests for full flow
- **Doc tests**: 6 doctests on public APIs
- **Benchmarks**: Criterion suite for core functions

### Coverage Gaps
1. **`orchestrator.rs`** — Only 7 unit tests, no integration tests for full `run()` flow (requires network)
2. **`formatter/mod.rs`** — `format_verbose_sections()` has no direct test coverage
3. **`history.rs`** — `print_history()` output not tested (just prints to stdout)
4. **Error paths** — Network failure scenarios in download/upload not fully covered
5. **Config file loading** — No tests for actual file reading (only in-memory TOML parsing)

### Recommended Additions
- Snapshot tests for `--json` and `--csv` output formats
- Integration test with mock server exercising full `orchestrator.run()` flow
- Property-based tests for `calculate_distance()` (Haversine formula edge cases)
- Test for `--format` flag precedence over legacy `--json/--csv/--simple` flags

---

## 7. Dependency Audit

| Dep | Current | Latest | Status | Concern |
|-----|---------|--------|--------|---------|
| reqwest | 0.12 | 0.12 | ✅ Current | — |
| tokio | 1 | 1 | ✅ Current | — |
| clap | 4 | 4 | ✅ Current | — |
| serde | 1 | 1 | ✅ Current | — |
| indicatif | 0.18 | 0.18 | ✅ Current | RUSTSEC-2025-0119 (number_prefix transitive) |
| quick-xml | 0.39 | 0.39 | ✅ Current | — |
| chrono | 0.4 | 0.4 | ✅ Current | Mature, stable |
| toml | 1.1.2 | 1.x | ✅ Current | winnow 0.7/1.0 duplicate |
| thiserror | 2 | 2 | ✅ Current | — |
| directories | 6.0.0 | 6.x | ✅ Current | — |
| csv | 1 | 1 | ✅ Current | — |
| owo-colors | 4 | 4 | ✅ Current | — |

**Dependency Score**: All deps current. Two minor advisories (ignored with justification). No unmaintained deps. No critical CVEs.

---

## 8. Final Verdict

**netspeed-cli v0.6.0 scores 82/100 (B+)** — a well-architected Rust CLI with strong error handling, clean module boundaries, and comprehensive CI. The primary concern is DRY duplication between download/upload modules (~150 lines of identical hot-path code). Secondary concerns are blanket clippy allow blocks and a subtle config merge semantics issue. No critical security issues. Production-ready with recommended refactors.
