# Phase 07 — ATLAS: Systems Architecture (Refactoring Assessment)

**Mode**: 3B — Full Refactoring  
**Date**: 2026-04-06  
**Cycle**: 2 (incremental from Mode 3C audit baseline)

---

## Architecture Assessment for Refactoring

### Current Architecture (as-is)

```
┌──────────────────────────────────────────────────────────────┐
│                        Binary (main.rs)                       │
│                    tokio runtime, error display                │
├──────────────────────────────────────────────────────────────┤
│                     Library (lib.rs)                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │  cli.rs  │ │ config.rs│ │ types.rs │ │   error.rs       │ │
│  │ + clap   │ │ + toml   │ │ + serde  │ │  + thiserror     │ │
│  │ validate │ │ merge    │ │ chrono   │ │                  │ │
│  │ (include)│ │ logic    │ │          │ │                  │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬─────────┘ │
│       │            │           │                  │           │
│  ┌────┴────────────┴───────────┴──────────────────┴───────┐  │
│  │              orchestrator.rs (central hub)              │  │
│  │  run() → fetch → select → ping → download → upload     │  │
│  └────┬──────────────────────────────┬────────────────────┘  │
│       │                              │                       │
│  ┌────┴──────────┐  ┌───────────────┴───────────────┐       │
│  │   http.rs     │  │        test_runner.rs          │       │
│  │  client/IP    │  │   (template method wrapper)    │       │
│  └───────────────┘  └───────────┬───────────────────┘       │
│                                 │                           │
│              ┌──────────────────┼──────────────────┐        │
│              │                  │                   │        │
│       ┌──────┴──────┐   ┌──────┴──────┐    ┌──────┴──────┐ │
│       │ download.rs │   │  upload.rs  │    │ servers.rs  │ │
│       │  (80% dup)  │   │  (80% dup)  │    │ ping/fetch  │ │
│       └─────────────┘   └─────────────┘    └─────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                  formatter/ (strategy)                 │  │
│  │  mod.rs → ratings, sections, stability, estimates     │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           progress.rs  │  history.rs                   │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Refactoring Targets (from audit findings)

| ID | Layer | Issue | Sprint | Risk |
|----|-------|-------|--------|------|
| **VAULT-SEC-006** | Upload I/O | HTTP 500 counted as success | 09A | LOW |
| **VAULT-SEC-001** | HTTP layer | No retry/backoff | 09A | LOW |
| **VAULT-SEC-002** | History I/O | No file permissions | 09A | LOW |
| **VAULT-SEC-003** | XML parsing | No schema validation | 09A | LOW |
| **CFG-001** | Config layer | Merge semantics unclear | 09A | LOW |
| **DRY-001** | Hot path | download/upload ~80% duplicate | 09B | MEDIUM |
| **CAST-001** | Code hygiene | Blanket clippy allows | 09B | LOW |
| **ARCH-001** | Build system | validate.rs include!() | 09B | LOW |

### Alternatives Matrix — DRY-001 (Sprint 2)

| Approach | Pros | Cons | Score |
|----------|------|------|-------|
| **A: Extract `bandwidth_loop.rs`** (Recommended) | Single source of truth, clean API, no behavior change | Requires careful parameterization of I/O closure | 9/10 |
| **B: Macro-based deduplication** | Less structural change | Macros harder to debug, doesn't fix underlying design | 5/10 |
| **C: Trait-based abstraction** | Most extensible | Overkill for 2 implementations, adds indirection | 6/10 |
| **D: Keep as-is with better comments** | Zero risk, no breaking changes | Debt compounds, every future change must be mirrored | 3/10 |

**Decision**: Approach A — `bandwidth_loop.rs` module with closure-based I/O. Score: 9/10.

### Alternatives Matrix — Retry/Backoff (Sprint 1)

| Approach | Pros | Cons | Score |
|----------|------|------|-------|
| **A: Simple retry loop (inline)** | No new deps, 2 retries + fixed backoff, <20 lines | Limited to this specific use case | 8/10 |
| **B: `reqwest-middleware` + `reqwest-retry`** | Production-grade, configurable | New dependency (+12 transitive), binary size increase | 5/10 |
| **C: `backoff` crate** | Well-tested, exponential + jitter | New dependency, overkill for single use | 6/10 |

**Decision**: Approach A — inline retry loop. No new dependencies needed for a CLI tool that makes a few HTTP calls per run. Score: 8/10.

### Sprint 1 Scope (09A — Infrastructure)

| File | Change | Lines |
|------|--------|-------|
| `src/upload.rs` | Check `response.status().is_success()` in upload loop | ~10 |
| `src/http.rs` | Add retry wrapper for server fetch + IP discovery | ~30 |
| `src/servers.rs` | Add retry to `fetch_client_location` + `fetch_servers` | ~20 |
| `src/history.rs` | Apply `0o600` permissions on file creation | ~10 |
| `src/config.rs` | Document merge semantics, improve naming | ~15 |
| `src/servers.rs` | Add XML field validation on deserialization | ~20 |
| `deny.toml` | Document RUSTSEC-2025-0119 upgrade timeline | ~5 |
| `SECURITY.md` | Update supported versions table | ~5 |

**Total Sprint 1**: ~115 lines, low risk, no behavior changes for happy path.

### Sprint 2 Scope (09B — Architecture)

| File | Change | Lines |
|------|--------|-------|
| `src/bandwidth_loop.rs` | **NEW** — Shared bandwidth measurement loop | ~80 |
| `src/download.rs` | Refactor to use `BandwidthLoopState` | ~60 (was ~150) |
| `src/upload.rs` | Refactor to use `BandwidthLoopState` | ~60 (was ~150) |
| `src/lib.rs` | Add `bandwidth_loop` module | ~1 |
| `benches/core_benchmarks.rs` | Update imports if needed | ~5 |
| 7 modules | Replace blanket `#![allow(...)]` with function-level | ~30 |
| `src/validate.rs` | Convert to proper module (or document include!() rationale) | ~5 |

**Total Sprint 2**: ~241 lines, medium risk, behavior-preserving refactor.

### Downstream Requirements

- **FORGE (09A)**: Maintain all existing tests. Add test for HTTP 500 handling in upload. Add retry behavior test.
- **FORGE (09B)**: Every change to download.rs must be mirrored to upload.rs (or better — eliminated by shared module). All benchmarks must still compile and pass.
- **SENTINEL (12)**: Re-run full test suite. Add HTTP 500 test. Add retry test.
- **CONDUCTOR (10.5)**: Verify no DRY violations, no dead code, no type escapes, no secrets.
