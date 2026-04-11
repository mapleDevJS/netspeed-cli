# Phase 13 — Sentinel: Quality Assurance Audit

**Auditor**: Test Lead
**Date**: 2026-04-06
**Mode**: Audit (read-only) | **Domain**: CLI

---

## Test Inventory

| Category | Count | Files | Quality |
|----------|-------|-------|---------|
| Unit tests (inline `#[cfg(test)]`) | 147 | All modules | Good — pure functions well covered |
| Integration tests (wiremock) | 10 | `integration_upload_fetch_test.rs`, `mock_network_test.rs` | Good — HTTP mocking |
| CLI subprocess tests | 24 | `integration_test.rs` | Good — `cargo run` subprocess tests |
| E2E tests | 5 | `e2e_test.rs` | Moderate — full flow with mocks |
| Doc tests | 6 | `common.rs`, `servers.rs` | Good — public API with examples |
| Benchmarks | 1 suite | `core_benchmarks` | Good — Criterion for core functions |
| **Total** | **~192** | | |

### Test Results (Current Run)
```
running 147 tests — ok. 147 passed; 0 failed
running 5 e2e tests — ok. 5 passed; 0 failed
running 24 integration tests — ok. 24 passed; 0 failed
running 10 upload/ping tests — ok. 10 passed; 0 failed
running 2 mock network tests — ok. 2 passed; 0 failed
running 6 doc tests — ok. 6 passed; 0 failed
```

**100% pass rate. 0 failures. Build + test + clippy + fmt all clean.**

---

## Coverage Analysis by Module

| Module | Test Count | Coverage Quality | Gap |
|--------|-----------|-----------------|-----|
| `cli.rs` | 15 | ✅ Good — all validators tested | None |
| `common.rs` | 10 | ✅ Excellent — pure functions + doctests | None |
| `config.rs` | 6 | ✅ Good — merge logic, parsing | None |
| `error.rs` | 11 | ✅ Excellent — all error variants | None |
| `download.rs` | 10 | ✅ Good — URL generation, stream count | None |
| `upload.rs` | 10 | ✅ Good — data generation, URL, stream count | None |
| `bandwidth_loop.rs` | 0 | ⚠️ NO direct unit tests | Only tested indirectly via download/upload |
| `formatter/mod.rs` | 6 | ⚠️ Partial — format functions tested, but output not asserted | Tests verify no panic, not correctness |
| `formatter/ratings.rs` | 3 | ✅ Good — rating functions tested | None |
| `formatter/stability.rs` | 4 | ✅ Good — CV, percentiles | None |
| `formatter/estimates.rs` | 1 | ✅ Good — time estimate formatting | None |
| `history.rs` | 10 | ✅ Good — save/load, truncation, recovery | None |
| `http.rs` | 5 | ✅ Good — client creation, IP parsing | None |
| `orchestrator.rs` | 7 | ⚠️ Partial — only `is_verbose()` and creation tested | No `run()` flow test |
| `progress.rs` | 9 | ✅ Good — spinner, progress bar, NO_COLOR | None |
| `servers.rs` | 14 | ✅ Good — distance, ping, selection | None |
| `test_runner.rs` | 5 | ✅ Good — result structure | None |
| `types.rs` | 4 | ✅ Good — serialization | None |
| `validate.rs` | 4 | ✅ Good — IPv4 validation | None |

---

## Test Quality Deep Dive

### Strengths
1. **Pure function coverage**: `common.rs` has doctests + unit tests for every function
2. **Error type coverage**: Every `SpeedtestError` variant has display + source tests
3. **Serial test isolation**: History tests use `#[serial]` to prevent race conditions
4. **Recovery testing**: `test_save_result_invalid_json_recovery` verifies graceful handling of corrupt data
5. **Edge cases**: Zero elapsed time, empty vectors, single server, max boundaries
6. **Benchmark suite**: Criterion benchmarks for core functions
7. **CLI integration**: Subprocess tests verify actual binary behavior

### Gaps
| Gap ID | Description | Impact | Effort |
|--------|-------------|--------|--------|
| QA-001 | `bandwidth_loop.rs` has no direct unit tests | Medium | The shared state module is the most important refactor artifact but relies on indirect testing through download/upload tests |
| QA-002 | `format_verbose_sections()` output not asserted | Low | Test calls the function but doesn't verify stdout content — only checks no panic |
| QA-003 | No snapshot/golden file tests for JSON/CSV output | Low | Output format regression would not be caught |
| QA-004 | No `orchestrator.run()` integration test | Medium | Full lifecycle (fetch servers → ping → download → upload → format) is only tested via E2E with mocks |
| QA-005 | `print_history()` output not verified | Low | Function prints to stdout — not captured or asserted in tests |
| QA-006 | No property-based tests for Haversine formula | Low | `calculate_distance()` has 6 fixed cases but no invariants (symmetry, triangle inequality) |
| QA-007 | Upload HTTP error handling not tested | Medium | `integration_upload_fetch_test.rs` acknowledges HTTP 500 treated as success |

---

## Test Infrastructure

| Aspect | Status | Notes |
|--------|--------|-------|
| Test framework | ✅ | Built-in `cargo test` |
| Mocking | ✅ | `wiremock` for HTTP, manual mocks for data |
| Serial execution | ✅ | `serial_test` crate for env-dependent tests |
| Temporary files | ✅ | `tempfile` crate for history tests |
| Benchmarking | ✅ | Criterion with HTML reports |
| Test organization | ✅ | Inline `#[cfg(test)]` modules + separate test files |
| CI test execution | ✅ | Tests run in CI (per workflow files) |

### Unsafe Code in Tests
```rust
// progress.rs:150-162
#[allow(unsafe_code)]
unsafe { std::env::set_var("NO_COLOR", "1") }
```
**Assessment**: Marked with `#[allow(unsafe_code)]` and `#[serial]` guard. Acceptable for NO_COLOR testing. Alternative: refactor `no_color()` to accept explicit override parameter.

---

## Findings

| ID | Title | Severity | Description |
|----|-------|----------|-------------|
| QA-001 | `bandwidth_loop.rs` has no unit tests | MEDIUM | The extracted shared module (the key refactor from previous cycle) has zero direct tests. It's only tested indirectly. Fix: Add unit tests for `record_bytes()`, `finish()`, throttle behavior |
| QA-002 | Output format tests don't assert content | LOW | `test_format_simple_with_data` and `test_format_verbose_sections_integration` call functions but don't assert output values. Fix: Capture stdout and assert expected strings |
| QA-003 | No snapshot tests for output formats | LOW | JSON, CSV, detailed output have no golden file tests. Fix: Add `expectorate` or `insta` crate for snapshot testing |
| QA-004 | Upload HTTP 500 treated as success | MEDIUM | Previous audit finding (VAULT-SEC-006) — the fix checks `response.status().is_success()`, but no test asserts this behavior |

---

## Score: Test Quality — 82/100 (B+)

| Dimension | Score | Max |
|-----------|-------|-----|
| Test breadth | 22 | 25 |
| Edge case coverage | 18 | 20 |
| Non-regression suite | 15 | 20 |
| Mock quality | 10 | 15 |
| Benchmark coverage | 8 | 10 |
| Test infrastructure | 9 | 10 |
