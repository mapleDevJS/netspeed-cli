# Security Audit — netspeed-cli v0.7.0

## Executive Summary

| Dimension | Score | Status |
|-----------|-------|--------|
| Secrets Management | 100/100 | ✅ PASS |
| TLS Configuration | 100/100 | ✅ PASS |
| Unsafe Code | 95/100 | ⚠️ MINOR |
| Error Handling | 90/100 | ✅ PASS |
| Input Validation | 95/100 | ✅ PASS |
| Branch Protection | 0/100 | ❌ FAIL |

## Findings

### SEC-01: No Hardcoded Secrets ✅
- **Scan result**: 0 hardcoded secrets, API keys, or credentials
- **Environment**: No secrets logged or printed to output
- **Verdict**: PASS

### SEC-02: TLS Configuration ✅
- **TLS backend**: `rustls-tls` (modern, pure-Rust TLS)
- **No insecure flags**: No `danger_accept_invalid_certs` or `insecure` flags
- **Verdict**: PASS

### SEC-03: Unsafe Code Usage ⚠️
- **Location**: `src/formatter/dashboard.rs:621,627` and `src/progress.rs:147,153`
- **Context**: Test code only — `set_var`/`remove_var` for `NO_COLOR` env var testing
- **Risk**: LOW — only in `#[cfg(test)]` blocks
- **Recommendation**: Add `#[allow(unsafe_code)]` comments documenting why unsafe is necessary
- **Verdict**: MINOR — acceptable for test isolation

### SEC-04: Branch Protection ❌
- **Status**: `master` branch is NOT protected
- **Risk**: HIGH — anyone with write access can force-push or merge without CI
- **Recommendation**: Enable branch protection with:
  - Require pull request reviews (≥ 1)
  - Require status checks to pass (CI, release)
  - Require linear history (no merge commits to main)
  - Restrict force pushes
  - Include administrators
- **Verdict**: BLOCK — must be fixed before production-grade release

### SEC-05: Input Validation ✅
- **CLI validation**: IP addresses, timeout, CSV delimiter all validated
- **File validation**: `check-toml`, `check-yaml`, `detect-private-key` in pre-commit hooks
- **Verdict**: PASS

### SEC-06: Error Handling ✅
- **Result-based**: All fallible operations use `Result<T, Error>` pattern
- **No silent failures**: Errors propagated or logged with context
- **No panic! in production**: All `panic!` calls are in test code
- **unwrap() usage**: 20 instances — all in test code or in contexts where failure indicates bugs (Mutex locks, serde serialization of known-good data)
- **Verdict**: PASS

## Dependency Security

| Dependency | Version | Known CVEs | Status |
|------------|---------|------------|--------|
| reqwest | 0.12 | 0 | ✅ |
| tokio | 1 | 0 | ✅ |
| serde | 1 | 0 | ✅ |
| quick-xml | 0.39 | 0 | ✅ |
| thiserror | 2 | 0 | ✅ |

## Recommendation

1. **P0 (BLOCK)**: Enable branch protection on `master` and `staging`
2. **P1**: Add `#[allow(unsafe_code)]` with SAFETY comments in test code
3. **P2**: Audit `unwrap()` calls in non-test code for potential DoS vectors
