# Phase 09B — FORGE: Sprint 2 (Architecture)

**Mode**: 3B — Full Refactoring  
**Date**: 2026-04-06  
**Sprint**: 2 of 2 — Architecture (DRY elimination, code structure)

---

## Changes Applied

### 1. DRY-001: Extract shared bandwidth measurement loop — FIXED
**New file**: `src/bandwidth_loop.rs` (108 lines)  
**Refactored**: `src/download.rs` (218 → 165 lines, -24%)  
**Refactored**: `src/upload.rs` (239 → 159 lines, -34%)  
**Total removed**: ~150 lines of duplicated hot-path code

**What was extracted**:
- Throttle gate (`last_sample_ms`, `SAMPLE_INTERVAL_MS`)
- Atomic byte counting (`total_bytes`)
- Peak speed tracking (`peak_bps`)
- Speed sample collection (`speed_samples`)
- Progress bar updates
- Result computation (`finish()`)

**What remains module-specific**:
- Download: URL construction, HTTP streaming, `StreamResult`
- Upload: URL construction, data generation, HTTP POST
- Both: Module-specific constants (estimated bytes, round count)

**API**:
```rust
let state = Arc::new(BandwidthLoopState::new(estimated_total, progress));
// In each stream:
state.record_bytes(chunk_len);
// At end:
let result = state.finish(); // → BandwidthResult { avg_bps, peak_bps, total_bytes, ... }
```

### 2. Removed blanket `#![allow(...)]` from download.rs and upload.rs
**Before**: Module-level `#![allow(clippy::cast_precision_loss, cast_possible_truncation, cast_sign_loss)]`  
**After**: Removed — the `bandwidth_loop.rs` module has no blanket allows

### 3. Module registration
**File**: `src/lib.rs` — added `pub mod bandwidth_loop;`

---

## Sprint 2 Gate Verification

### CONDUCTOR 10.5 Checklist

- [x] **Build passes**: `cargo build` — 0 errors, 0 warnings
- [x] **Tests pass**: `cargo test` — 192 tests, 0 failures
- [x] **Clippy clean**: `cargo clippy --all-targets --all-features -- -D warnings` — 0 warnings
- [x] **Format clean**: `cargo fmt --check` — passes
- [x] **No DRY violations**: Bandwidth loop is single source of truth
- [x] **No dead code**: All new code called, old code removed
- [x] **No type escapes**: No new `as` casts beyond originals
- [x] **No secrets**: No credentials, keys, or tokens
- [x] **Error handling**: Same error paths preserved
- [x] **Exit codes**: No change
- [x] **Benchmarks compile**: `cargo bench --no-run` — all 3 executables built
- [x] **Claims verified**:
  - "download.rs reduced by ~24%" — 218 → 165 lines
  - "upload.rs reduced by ~34%" — 239 → 159 lines
  - "No behavior change" — All 192 tests pass

### Behavior Changes
| Change | Impact | Risk |
|--------|--------|------|
| Shared bandwidth loop | Identical behavior | NONE (refactor only) |
| Blanket allow removed | Cleaner linting | NONE (improvement) |
| `bandwidth_loop` pub module | New public API | LOW (documented) |
