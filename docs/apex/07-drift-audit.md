# Phase 07 — Architecture Drift Audit (Light)

**Auditor**: Software Architect
**Date**: 2026-04-06
**Mode**: Audit (read-only) | **Domain**: CLI

---

## Architecture Drift Check

### Current Module Structure

```
src/
├── main.rs              # Entry point → run_speedtest()
├── lib.rs               # Module declarations + re-exports
├── cli.rs               # Clap CLI args + validation (include! validate.rs)
├── validate.rs          # Shared validation (include!-ed by cli.rs and build.rs)
├── config.rs            # Config file loading + merge logic
├── orchestrator.rs      # SpeedTestOrchestrator — lifecycle management
├── http.rs              # HTTP client creation, client IP discovery
├── servers.rs           # Server XML fetching, ping test, distance calculation
├── download.rs          # Download bandwidth test
├── upload.rs            # Upload bandwidth test
├── bandwidth_loop.rs    # Shared bandwidth measurement state (NEW — from previous refactor)
├── test_runner.rs       # Bandwidth test orchestration wrapper
├── progress.rs          # Progress bars, spinners, NO_COLOR
├── history.rs           # Test history storage and display
├── types.rs             # Data types (Server, TestResult, etc.)
├── error.rs             # SpeedtestError enum
├── common.rs            # Pure utility functions
└── formatter/
    ├── mod.rs           # OutputFormat strategy pattern
    ├── sections.rs      # Output section formatters
    ├── ratings.rs       # Rating functions (ping, speed, connection, bufferbloat)
    ├── stability.rs     # Speed stability analysis (CV, percentiles)
    └── estimates.rs     # Usage check targets, download time estimates
```

### Architecture Assessment

| Principle | Status | Evidence |
|-----------|--------|----------|
| **Single Responsibility** | ✅ | Each module has one clear purpose |
| **DRY** | ✅ Good | `bandwidth_loop.rs` extracted (previous refactor fixed DRY-001) |
| **Open/Closed** | ✅ | `OutputFormat` enum — add new formats without modifying existing code |
| **Dependency Direction** | ✅ | `orchestrator.rs` depends on leaf modules, not vice versa |
| **Error Handling** | ✅ | `thiserror`-based `SpeedtestError` covers all error paths |
| **Async Discipline** | ✅ | `tokio::spawn` for concurrent streams, proper `JoinHandle` collection |
| **Thread Safety** | ✅ | `Arc<AtomicU64>`, `Arc<Mutex<Vec>>` for shared state |

### Previous Audit Findings — Status

| Previous ID | Finding | Current Status |
|-------------|---------|---------------|
| DRY-001 | Download/upload duplication | ✅ FIXED — `bandwidth_loop.rs` extracted |
| CFG-001 | Config merge_bool semantics | ⚠️ PARTIALLY — Well-documented in code comments, but semantics remain |
| CAST-001 | Blanket clippy cast allows | ⚠️ PARTIALLY — `progress.rs` still has module-level `#![allow(...)]` |
| ARCH-001 | `validate.rs` uses `include!()` | ⚠️ UNCHANGED — Justified with documentation, dual-include still present |
| SEC-001 | RUSTSEC-2025-0119 ignored | ✅ TRACKED — `deny.toml` has comment with upgrade timeline |

### Current Architecture Concerns

| ID | Concern | Severity | Description |
|----|---------|----------|-------------|
| ARCH-NEW-001 | `test_runner.rs` partially duplicates `bandwidth_loop` orchestration | LOW | `test_runner::run_bandwidth_test` handles progress creation and result aggregation, while `bandwidth_loop.rs` handles per-sample tracking. Two layers of abstraction for one concept. Could be unified. |
| ARCH-NEW-002 | `include!()` chain: `build.rs` → `cli.rs` → `validate.rs` | LOW | Double include creates compilation coupling. Changes to `cli.rs` can break `build.rs` and vice versa. The current approach works but is fragile. |
| ARCH-NEW-003 | `orchestrator.rs` at ~230 LOC | INFO | Largest source file. Handles lifecycle, output formatting, server management, and shell completion. Could benefit from splitting into `lifecycle.rs` and `output.rs`. |
| ARCH-NEW-004 | No trait abstraction for HTTP operations | INFO | `download.rs` and `upload.rs` directly use `reqwest::Client`. No trait boundary makes mocking harder (though wiremock handles this at integration level). |

### Module Dependency Graph (Clean)

```
main.rs
  └── orchestrator.rs
        ├── cli.rs → validate.rs (include!)
        ├── config.rs
        ├── http.rs
        ├── servers.rs
        ├── download.rs → bandwidth_loop.rs
        ├── upload.rs → bandwidth_loop.rs
        ├── test_runner.rs
        ├── progress.rs
        ├── history.rs
        ├── formatter/mod.rs
        │   ├── sections.rs
        │   ├── ratings.rs
        │   ├── stability.rs
        │   └── estimates.rs
        ├── types.rs
        └── error.rs
        └── common.rs
```

**Assessment**: Clean layered architecture. No circular dependencies. `bandwidth_loop.rs` correctly sits between `download.rs`/`upload.rs` and `common.rs`.

---

## Score: Architecture — 85/100 (A-)

| Dimension | Score | Max |
|-----------|-------|-----|
| Module boundaries | 9 | 10 |
| DRY compliance | 8 | 10 |
| Dependency direction | 10 | 10 |
| Extensibility (OCP) | 9 | 10 |
| Error architecture | 9 | 10 |
| Abstraction boundaries | 8 | 10 |
| Code organization | 9 | 10 |
| Build system cleanliness | 8 | 10 |
| Documentation of trade-offs | 5 | 5 |
| Bonus: Strategy pattern | +5 | +5 |
