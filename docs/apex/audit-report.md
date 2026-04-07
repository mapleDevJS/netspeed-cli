# Mode 3C Audit Report — SENTINEL, VAULT, GUARDIAN

**Mode**: 3C — Full Audit  
**Date**: 2026-04-06  
**Phases**: 12 (SENTINEL), 13 (VAULT), 14 (GUARDIAN) — executed in parallel

---

## Phase 12 — SENTINEL: Quality Assurance Audit

### AUDIT-SCORE: Test Coverage & QA — 78/100

#### Test Inventory Summary

| Category | Count | Quality |
|----------|-------|---------|
| Unit tests (inline) | ~85 | Good — pure functions well covered |
| Integration tests (wiremock) | ~15 | Good — mock HTTP server tests |
| E2E tests | 5 | Moderate — full flow with mocks |
| CLI subprocess tests | ~22 | Good — cargo run subprocess tests |
| Doc tests | 6 | Good — public API documented |
| Benchmarks | 1 suite | Good — Criterion for core functions |
| **Total** | **~133** | |

#### Sub-Scores

| Dimension | Score | Max | Notes |
|-----------|-------|-----|-------|
| Test breadth | 28 | 35 | Good unit + integration coverage |
| Edge case coverage | 18 | 25 | Missing some error paths |
| Non-regression suite | 22 | 25 | CLI parsing well tested |
| Mock quality | 10 | 15 | Wiremock used well but limited scenarios |

#### Findings

**SENT-QA-001 [MEDIUM]: Upload test treats HTTP 500 as success**
- File: `tests/integration_upload_fetch_test.rs:76` — comment acknowledges: "upload_test treats HTTP 500 as success (.is_ok()), so bytes are still counted"
- Root cause: `upload_test` in `upload.rs` uses `.is_ok()` on the `send()` result, not checking response status code
- Impact: HTTP 500 from speedtest server is treated as successful upload, inflating results
- Fix: Check `response.status().is_success()` in upload loop

**SENT-QA-002 [LOW]: No snapshot tests for output formats**
- JSON, CSV, Simple, and Detailed output formats have no snapshot/fixture tests
- A regression in output format would not be caught
- Fix: Add golden file tests for `--json`, `--csv`, `--simple` output

**SENT-QA-003 [LOW]: `format_verbose_sections()` untested**
- File: `formatter/mod.rs` — this function calls `build_targets()`, `build_estimates()`, `compute_cv()`, `compute_percentiles()`, and `history::format_comparison()`
- No direct test exercises this integration path
- Fix: Add integration test with full `TestResult` containing samples

**SENT-QA-004 [LOW]: History `print_history()` not tested**
- Prints to stdout — hard to test without capturing
- No test verifies truncation of long sponsor names in display
- Fix: Refactor to return String, then test the string

**SENT-QA-005 [LOW]: No property-based tests for Haversine formula**
- `calculate_distance()` has 5 fixed test cases but no property tests
- Missing properties: symmetry (dist(a,b) == dist(b,a)), triangle inequality, zero for same point
- Fix: Add proptest crate with these invariants

---

## Phase 13 — VAULT: Security Audit

### AUDIT-SCORE: Security — 80/100

#### Sub-Scores

| Dimension | Score | Max | Notes |
|-----------|-------|-----|-------|
| Input validation | 18 | 20 | CLI args validated, config file not |
| Network security | 16 | 20 | rustls TLS, no retry/backoff |
| Data protection | 14 | 20 | History file has no access restrictions |
| Dependency security | 16 | 20 | RUSTSEC advisory ignored |
| Error handling | 16 | 20 | Structured errors, but some silent failures |

#### STRIDE Worksheet (Completed)

| Asset | Spoofing | Tampering | Repudiation | Info Disclosure | DoS | Elevation |
|-------|----------|-----------|-------------|-----------------|-----|-----------|
| **HTTP (speedtest.net)** | — | LOW | — | — | MED | — |
| **Config file (TOML)** | — | LOW | — | LOW | — | — |
| **History file (JSON)** | — | LOW | — | MED | — | — |
| **Server XML response** | — | MED | — | — | — | — |
| **CLI input** | — | — | — | — | — | — |

#### Findings

**VAULT-SEC-001 [MEDIUM]: No HTTP retry/backoff on transient failures**
- All HTTP requests are single-attempt with timeout
- No exponential backoff, no jitter, no retry on 5xx
- If speedtest.net returns transient 503, the entire test fails
- Fix: Add `reqwest-middleware` with `reqwest-retry` or implement simple retry loop

**VAULT-SEC-002 [MEDIUM]: History file stored without access restrictions**
- File: `history.rs` — `get_history_path()` creates directory with `fs::create_dir_all()`
- No `chmod` or platform-specific access controls applied
- Contains: IP addresses, server names, timestamps, speed data
- On shared systems, other users could read this file
- Fix: Apply `0o600` permissions on history file creation

**VAULT-SEC-003 [MEDIUM]: XML deserialization without schema validation**
- File: `servers.rs` — `from_str(&response)` directly deserializes XML into structs
- Malicious XML from a compromised speedtest server could trigger:
  - Integer overflow in coordinate fields (f64, but still)
  - Unexpected enum variants if serde derives change
  - Deep nesting causing stack overflow (serde has no depth limit by default)
- Fix: Use `serde_with` limits, or validate XML structure before deserialization

**VAULT-SEC-004 [LOW]: RUSTSEC-2025-0119 ignored**
- `number_prefix` crate (transitive via indicatif) has a known advisory
- Ignored in `deny.toml` with comment but no upgrade timeline
- Fix: Track indicatif release that removes this dependency

**VAULT-SEC-005 [LOW]: User-Agent string impersonation**
- File: `http.rs` — User-Agent set to Chrome 120 on macOS
- This is a common pattern for speedtest clients but could be flagged by some servers
- Not a security vulnerability but a transparency concern
- Fix: Consider using `netspeed-cli/{version}` or documenting this choice

**VAULT-SEC-006 [LOW]: Silent error consumption in upload**
- File: `upload.rs:100` — `.is_ok()` on POST result without checking HTTP status
- HTTP 500 responses count as successful uploads
- Data transferred but result is meaningless
- Fix: Check `response.status().is_success()` before counting bytes

---

## Phase 14 — GUARDIAN: Compliance & Legal Audit

### AUDIT-SCORE: Compliance — 85/100

#### Sub-Scores

| Dimension | Score | Max | Notes |
|-----------|-------|-----|-------|
| License compliance | 20 | 20 | MIT license, all deps compatible |
| Documentation | 18 | 20 | Comprehensive but missing some edge cases |
| Privacy | 15 | 20 | No privacy policy, data collection disclosure |
| Distribution | 17 | 20 | Multi-platform releases, SBOM included |
| CI/CD governance | 15 | 20 | Good CI, no branch protection documented |

#### Findings

**GUARD-COMP-001 [LOW]: No privacy policy or data collection disclosure**
- The tool collects and stores: IP addresses, server locations, test results
- No privacy policy in README or docs
- No `--no-history` flag to disable data collection
- Users may not know their IP is stored locally
- Fix: Add privacy section to README, document history storage

**GUARD-COMP-002 [LOW]: SECURITY.md version table is outdated**
- File: `SECURITY.md` — lists 0.3.x as supported but current is 0.6.0
- This creates confusion about which versions receive security updates
- Fix: Update to "Only latest release" or maintain accurate table

**GUARD-COMP-003 [LOW]: No CODEOWNERS file**
- Repository has no CODEOWNERS for security-critical files
- `deny.toml`, `SECURITY.md`, CI workflows have no required reviewers
- Fix: Add `.github/CODEOWNERS` for critical paths

**GUARD-COMP-004 [INFO]: Homebrew formula update is automated but unverified**
- File: `release.yml` — `sed` commands update version/SHA in formula
- No verification that the updated formula builds correctly before push
- If sed pattern doesn't match, formula could be corrupted
- Fix: Add `brew audit` step before `git push` in release workflow

**GUARD-COMP-005 [INFO]: No branch protection rules documented**
- CI requires tests on PRs but no evidence of branch protection rules
- `RELEASE.md` mentions develop→main workflow but GitHub settings not visible
- Fix: Document branch protection requirements in CONTRIBUTING.md

---

## Consolidated Finding Summary

| ID | Title | Severity | Category | Effort |
|----|-------|----------|----------|--------|
| DRY-001 | Download/upload modules ~80% duplicate | HIGH | Architecture | Medium |
| VAULT-SEC-001 | No HTTP retry/backoff on transient failures | MEDIUM | Security | Low |
| VAULT-SEC-002 | History file stored without access restrictions | MEDIUM | Security | Low |
| VAULT-SEC-003 | XML deserialization without schema validation | MEDIUM | Security | Low |
| VAULT-SEC-004 | RUSTSEC-2025-0119 ignored, no upgrade timeline | LOW | Security | Low |
| VAULT-SEC-005 | User-Agent impersonation | LOW | Security | Low |
| VAULT-SEC-006 | Silent error: HTTP 500 counted as successful upload | MEDIUM | Security | Low |
| CFG-001 | Config merge_bool semantics are counterintuitive | MEDIUM | Correctness | Low |
| CAST-001 | Blanket clippy cast allows across 7 modules | LOW | Hygiene | Low |
| ARCH-001 | validate.rs uses include!() macro | LOW | Architecture | Low |
| SENT-QA-001 | Upload test treats HTTP 500 as success | MEDIUM | QA | Low |
| SENT-QA-002 | No snapshot tests for output formats | LOW | QA | Medium |
| SENT-QA-003 | format_verbose_sections() untested | LOW | QA | Low |
| TEST-001 | Unsafe env var manipulation in tests | LOW | Testability | Low |
| GUARD-COMP-001 | No privacy policy or data collection disclosure | LOW | Compliance | Low |
| GUARD-COMP-002 | SECURITY.md version table outdated | LOW | Compliance | Low |
| GUARD-COMP-003 | No CODEOWNERS file | LOW | Compliance | Low |
| GUARD-COMP-004 | Homebrew formula update unverified | INFO | Distribution | Low |
| GUARD-COMP-005 | Branch protection rules not documented | INFO | Governance | Low |

---

## Remediation Roadmap (Ranked by Risk/Effort)

### P0 — Fix Now (Security + Correctness)

| # | Finding | Fix | Effort |
|---|---------|-----|--------|
| 1 | VAULT-SEC-006: HTTP 500 counted as upload success | Check `response.status().is_success()` in upload loop | ~5 lines |
| 2 | VAULT-SEC-001: No HTTP retry/backoff | Add 2-retry loop with 500ms backoff on 5xx | ~30 lines |
| 3 | VAULT-SEC-002: History file permissions | Apply `0o600` on file creation (Unix) | ~10 lines |

### P1 — Next Sprint (Architecture + QA)

| # | Finding | Fix | Effort |
|---|---------|-----|--------|
| 4 | DRY-001: Extract shared bandwidth loop | Create `bandwidth_loop.rs` module | ~200 lines refactor |
| 5 | VAULT-SEC-003: XML deserialization validation | Add field-level validation on XML parse | ~20 lines |
| 6 | CFG-001: Document config merge semantics | Add doc comments, consider Option<bool> | ~15 lines |
| 7 | SENT-QA-001: Upload HTTP 500 test | Add assertion for status code checking | ~10 lines |

### P2 — Backlog (Hygiene + Compliance)

| # | Finding | Fix | Effort |
|---|---------|-----|--------|
| 8 | CAST-001: Function-level cast allows | Replace module-level with function-level | ~30 lines |
| 9 | GUARD-COMP-001: Privacy policy | Add to README | ~20 lines |
| 10 | GUARD-COMP-002: Update SECURITY.md | Fix version table | ~5 lines |
| 11 | VAULT-SEC-004: RUSTSEC tracking | Add upgrade timeline comment | ~5 lines |
| 12 | SENT-QA-002: Snapshot tests | Add golden file tests | ~50 lines |
| 13 | GUARD-COMP-003: CODEOWNERS | Add `.github/CODEOWNERS` | ~5 lines |
| 14 | ARCH-001: validate.rs module | Convert include!() to proper module | ~20 lines |
| 15 | GUARD-COMP-004: Brew audit in CI | Add verification step | ~10 lines |
| 16 | GUARD-COMP-005: Document branch protection | Add to CONTRIBUTING.md | ~10 lines |
| 17 | TEST-001: Unsafe env tests | Refactor to context-based approach | ~15 lines |
| 18 | SENT-QA-003: Test verbose sections | Add integration test | ~20 lines |

---

## Final Scorecard

```
┌─────────────────────────────┬──────────┬──────────┬────────┐
│ Dimension                   │ Max      │ Score    │ Grade  │
├─────────────────────────────┼──────────┼──────────┼────────┤
│ Security & Correctness      │    40    │    32    │   B    │
│ Maintainability             │    25    │    19    │  B+    │
│ Architecture                │    20    │    15    │  B+    │
│ Performance                 │    10    │     8    │  B+    │
│ Testability                 │     5    │     4    │  B+    │
│ Bonus                       │    +5    │    +4    │   —    │
├─────────────────────────────┼──────────┼──────────┼────────┤
│ TOTAL (clean-code-octagon)  │   100    │    82    │  B+    │
├─────────────────────────────┼──────────┼──────────┼────────┤
│ Test Coverage (SENTINEL)    │   100    │    78    │  B+    │
│ Security (VAULT)            │   100    │    80    │   B+   │
│ Compliance (GUARDIAN)       │   100    │    85    │   B+   │
├─────────────────────────────┼──────────┼──────────┼────────┤
│ WEIGHTED AVERAGE            │   100    │    81    │   B+   │
└─────────────────────────────┴──────────┴──────────┴────────┘
```

## Consumption Report

```
┌──────────────────┬───────────┬───────────┬────────┐
│ Phase            │ Est. Input│ Est. Output│ Cost   │
├──────────────────┼───────────┼───────────┼────────┤
│ DISCOVER         │    12,000 │     4,000 │  $0.08 │
│ clean-code-oct   │     8,000 │     6,000 │  $0.12 │
│ SENTINEL audit   │     6,000 │     3,000 │  $0.07 │
│ VAULT audit      │     7,000 │     3,000 │  $0.08 │
│ GUARDIAN audit   │     5,000 │     2,500 │  $0.06 │
│ Scorecard        │     3,000 │     4,000 │  $0.07 │
├──────────────────┼───────────┼───────────┼────────┤
│ TOTAL            │    41,000 │    22,500 │  $0.48 │
└──────────────────┴───────────┴───────────┴────────┘

Pricing: Claude 3.5 Sonnet — Input $3/M, Output $15/M
Context optimization: Used parallel execution for independent audit phases
```

## Recommended Next Steps

**Current state: B+ (81/100 weighted average).** Code is healthy. 3 P0 items can be fixed in under an hour. The DRY-001 refactor is the highest-effort item (~200 lines) but yields the biggest architectural improvement.

| Priority | Action | Mode |
|----------|--------|------|
| 1 | Fix P0 security items (HTTP 500, retry, file perms) | Mode 3B Sprint |
| 2 | Extract shared bandwidth loop (DRY-001) | Mode 3B Sprint |
| 3 | Add snapshot tests + config docs | Mode 1 (test focus) |
| 4 | Ship next release | Mode 1 (PIPELINE + CHRONICLE) |
