# Phase 15 — Guardian: Compliance & Legal Audit

**Auditor**: Compliance Officer
**Date**: 2026-04-06
**Mode**: Audit (read-only) | **Domain**: CLI

---

## Compliance Assessment

### License Compliance

| Check | Status | Evidence |
|-------|--------|----------|
| Project license | ✅ | MIT — permissive, well-known |
| `LICENSE` file | ✅ | Present in project root |
| Dep license audit | ✅ | `deny.toml` allows 12 compatible licenses |
| Copyleft deps | ✅ None | All deps use MIT, Apache-2.0, BSD, ISC, etc. |
| SPDX identifiers | ✅ | `Cargo.toml` has `license = "MIT"` |

### Distribution

| Check | Status | Evidence |
|-------|--------|----------|
| Homebrew tap | ✅ | `mapleDevJS/homebrew-netspeed-cli` |
| Cargo registry | ✅ | Published to crates.io |
| Multi-platform releases | ✅ | CI workflow builds for macOS/Linux |
| Binary artifacts | ⚠️ | No pre-built binaries — users build from source or use Homebrew |
| SBOM in releases | ⚠️ | `cargo deny` can generate SBOM but not automated in releases |
| Checksums | ⚠️ | No SHA256 checksums published with releases |

### Privacy & Data

| Check | Status | Evidence |
|-------|--------|----------|
| Data collection disclosure | ✅ | README has "Privacy" section |
| Data types collected | ✅ | Server info, test metrics, client IP |
| Storage location | ✅ | Platform-specific data directory documented |
| Data retention | ✅ | Max 100 entries, automatic truncation |
| No telemetry | ✅ | README states "No analytics, telemetry, or crash reporting" |
| No third-party data sharing | ✅ | Only communicates with speedtest.net |
| GDPR compliance | ⚠️ Partial | No explicit GDPR statement, but minimal data collection is GDPR-friendly |
| `--no-history` flag | ❌ Missing | No way to disable history collection entirely. Fix: Add `--no-history` flag |
| Data deletion | ❌ Missing | No `--clear-history` command. Fix: Add command to delete history file |

### Repository Governance

| Check | Status | Evidence |
|-------|--------|----------|
| CODEOWNERS | ✅ NEW | `.github/CODEOWNERS` present — covers security-critical files |
| SECURITY.md | ✅ | Present with supported versions, reporting process |
| CONTRIBUTING.md | ✅ | Present with contribution guidelines |
| Branch protection | ⚠️ | Not verifiable from code — `RELEASE.md` mentions develop→main but no settings visible |
| CI pipeline | ✅ | GitHub Actions for build, test, lint |
| Release automation | ✅ | `release.yml` workflow with Homebrew formula update |
| Changelog | ✅ | `CHANGELOG.md` present |

### CI/CD Governance

| Check | Status | Evidence |
|-------|--------|----------|
| Build on PR | ✅ | CI runs `cargo build` |
| Test on PR | ✅ | CI runs `cargo test` |
| Lint on PR | ✅ | CI runs `cargo clippy` |
| Format check | ✅ | CI runs `cargo fmt --check` |
| Dependency audit | ✅ | `cargo deny` in CI |
| Secret scanning | ⚠️ | GitHub secret scanning likely enabled (default for public repos) but not verified |
| Dependabot/Renovate | ⚠️ | No `.github/dependabot.yml` found |

---

## Previous Compliance Findings — Status

| Previous ID | Finding | Current Status |
|-------------|---------|---------------|
| GUARD-COMP-001 | No privacy policy | ⚠️ PARTIALLY — README has privacy section but no dedicated policy |
| GUARD-COMP-002 | SECURITY.md outdated | ✅ FIXED — Now says "Only latest release" |
| GUARD-COMP-003 | No CODEOWNERS | ✅ FIXED — `.github/CODEOWNERS` created |
| GUARD-COMP-004 | Homebrew formula unverified | ⚠️ REMAINING — No `brew audit` step in release workflow |
| GUARD-COMP-005 | Branch protection undocumented | ⚠️ REMAINING — No documentation of branch protection rules |

---

## New Compliance Findings

| ID | Title | Severity | Description |
|----|-------|----------|-------------|
| COMP-NEW-001 | No `--no-history` flag | MEDIUM | Users cannot disable history collection. GDPR right to erasure not fully supported. Fix: Add `--no-history` CLI flag |
| COMP-NEW-002 | No `--clear-history` command | LOW | Users cannot delete stored history. Fix: Add command to remove history file |
| COMP-NEW-003 | No Dependabot/Renovate config | LOW | Dependency updates are manual. Fix: Add `.github/dependabot.yml` |
| COMP-NEW-004 | No pre-built binary releases | LOW | Users must build from source or use Homebrew. Fix: Add `cargo-dist` or `cross` for pre-built binaries |
| COMP-NEW-005 | No SBOM generation in CI | INFO | SBOM not automated. Fix: Add `cargo deny` SBOM output to release workflow |

---

## Score: Compliance — 82/100 (B+)

| Dimension | Score | Max |
|-----------|-------|-----|
| License compliance | 20 | 20 |
| Distribution quality | 14 | 20 |
| Privacy compliance | 14 | 20 |
| Repository governance | 17 | 20 |
| CI/CD governance | 13 | 15 |
| Data rights (erasure, deletion) | 4 | 5 |

### Previous Findings Remediation Rate
- **Fixed**: 2 of 5 (40%)
- **Partially fixed**: 1 of 5 (20%)
- **Remaining**: 2 of 5 (40%)
