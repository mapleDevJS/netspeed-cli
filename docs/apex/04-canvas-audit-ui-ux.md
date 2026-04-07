# Phase 04 — UI/UX Visual Design Audit (Canvas)

**Auditor**: UX Designer / CLI Interface Specialist
**Date**: 2026-04-06
**Mode**: Audit (read-only) | **Domain**: CLI
**Version**: v0.4.0 (Cargo.toml) / v0.5.0 (git HEAD)

---

## Executive Summary

Netspeed-cli presents a polished, modern CLI experience with excellent visual hierarchy, rich color-coded feedback, and strong accessibility considerations. The detailed output format is the crown jewel — a well-structured, information-dense display that balances aesthetics with utility. However, several gaps remain in edge-case UX, consistency, and discoverability.

**Overall Score: 80/100 (B+)**

---

## 1. Visual Hierarchy & Layout

### 1.1 Detailed Output (Default) — Excellent

| Aspect | Status | Notes |
|--------|--------|-------|
| Section headers | ✅ | Bold, underlined `TEST RESULTS`, `STABILITY`, `LATENCY PERCENTILES` |
| Label alignment | ✅ | Right-aligned 14-char labels create a clean vertical axis |
| Value alignment | ✅ | Values consistently positioned after `:` + spaces |
| Color-coded ratings | ✅ | Emoji + text (e.g. `⚡ Excellent` in green) |
| Horizontal separators | ⚠️ | README shows `──────────────` but code uses blank lines instead |
| Spacing | ✅ | Consistent double-newline between sections |

**Verdict**: Well-structured, information-dense, scannable. The right-aligned labels (`"{:>14}"`) create a professional, tabular feel.

### 1.2 Simple Output — Good

| Aspect | Status | Notes |
|--------|--------|-------|
| Format | ✅ | Single-line: `ping | Download: X | Upload: Y` |
| Piping separator | ✅ | Clean ` | ` delimiter |
| Color preservation | ✅ | Colors retained in simple mode |
| Missing labels | ⚠️ | Ping shows as `10.0 ms` without "Latency:" prefix — inconsistent |

**Verdict**: Functional but ping label inconsistency is noticeable.

### 1.3 Server List (`--list`) — Good

| Aspect | Status | Notes |
|--------|--------|-------|
| Column alignment | ✅ | Dynamic width calculation (`max_id_len`, `max_sponsor_len`) |
| Header formatting | ✅ | Dimmed header row |
| Separator line | ✅ | Dashed line under header |
| Distance display | ✅ | Formatted via `format_distance()` |

**Verdict**: Clean, well-aligned tabular output.

---

## 2. Color System & Rating Visuals

### 2.1 Color Palette Consistency

| Rating | Icon | Color | Bold | Source |
|--------|------|-------|------|--------|
| Excellent | ⚡ | Green | ✅ | `colorize_rating` |
| Great | 🟢 | Green | ❌ | `colorize_rating` |
| Good | 🟢 | Bright green | ❌ | `colorize_rating` |
| Fair | 🟡 | Yellow | ❌ | `colorize_rating` |
| Moderate | 🟠 | Bright yellow | ❌ | `colorize_rating` |
| Poor | 🔴 | Red | ❌ | `colorize_rating` |
| Slow | 🔴 | Bright red | ❌ | `colorize_rating` |
| Very Slow | ⚠️ | Red | ✅ | `colorize_rating` |

**Issues Found**:

| ID | Issue | Severity |
|----|-------|----------|
| VIS-001 | "Great" and "Good" both use 🟢 — indistinguishable in monochrome or terminal with limited colors | Medium |
| VIS-002 | "Slow" and "Poor" both use 🔴 — same icon, similar color intensity | Medium |
| VIS-003 | `degradation_str` uses inline color strings (`"green"`, `"yellow"`, `"red"`) instead of a shared enum — drift risk | Low |
| VIS-004 | `bufferbloat_colorized` has 5 distinct grade colors (A=green bold, B=bright green, C=yellow, D=bright yellow, F=red bold) — consistent but B and D differ only by shade, hard to distinguish on some terminals | Low |

### 2.2 Emoji Usage

| Context | Emojis Used | Assessment |
|---------|-------------|------------|
| Ratings | ⚡ 🟢 🟡 🟠 🔴 ⚠️ | Appropriate, widely supported |
| Packet Loss | ✓ | Unicode checkmark — good |
| Progress | · o O o | Minimal spinner — good for accessibility |
| Spinner check | ✓ | Green checkmark on completion |

**Issue**: No `NO_COLOR` handling for emoji in ratings — emojis persist even when `NO_COLOR` is set. While emojis aren't technically "color" in the ANSI sense, many users set `NO_COLOR` expecting a plain-text experience. Screen readers will vocalize every emoji.

---

## 3. Progress Feedback & Animations

### 3.1 Progress Bar (`SpeedProgress`)

| Aspect | Status | Notes |
|--------|--------|-------|
| Bar style | ✅ | 40-char `━╾─` progress characters — visually appealing |
| Color | ✅ | Cyan/blue bar — good contrast on dark terminals |
| Real-time speed | ✅ | Auto-scales Mb/s → Gb/s |
| Data transferred | ✅ | Auto-scales KB → MB → GB |
| Percentage | ✅ | Right-aligned 3-digit percentage |
| Elapsed time | ✅ | `{elapsed_precise}` from indicatif |
| Completion message | ✅ | `DONE` in green bold with total size and speed |

### 3.2 Spinners

| Aspect | Status | Notes |
|--------|--------|-------|
| Pattern | ✅ | `· o O o` — subtle, non-distracting |
| Hz | ✅ | 10 Hz for progress, 120ms tick for spinners |
| Completion | ✅ | `✓` green checkmark + message |
| Conditional display | ✅ | Only shown when `is_verbose` is true |

### 3.3 Issues

| ID | Issue | Severity |
|----|-------|----------|
| PROG-001 | Progress bar always outputs to stderr — no `--quiet` mode to fully suppress | Medium |
| PROG-002 | Upload task failures log to stderr with bare `eprintln!` — no color, no formatting, breaks visual consistency | Low |
| PROG-003 | No way to tell from output if `--no-download`/`--no-upload` was intentional vs failure — skipped tests silently return `TestRunResult::default()` | Medium |

---

## 4. Accessibility (a11y)

### 4.1 Screen Reader Compatibility

| Aspect | Status | Notes |
|--------|--------|-------|
| Box-drawing characters | ⚠️ | `═══`, `╾`, `─` read as gibberish by screen readers |
| Emojis in ratings | ⚠️ | Read aloud as "lightning bolt", "red circle", etc. |
| Color-only semantics | ⚠️ | Some info conveyed purely through color (e.g., speed values) |
| `NO_COLOR` support | ✅ | Detected and applied throughout |

### 4.2 NO_COLOR Compliance

| Module | Coverage | Gaps |
|--------|----------|------|
| `progress.rs` | ✅ Full | None |
| `formatter/mod.rs` | ✅ Full | None |
| `formatter/sections.rs` | ✅ Full | None |
| `formatter/ratings.rs` | ✅ Full | Emojis still emitted |
| `main.rs` (error) | ✅ Full | None |
| `upload.rs` | ❌ **Partial** | Bare `eprintln!("Warning: upload task {i} failed: {e}")` |

---

## 5. Help & Discoverability

### 5.1 `--help` Quality

| Aspect | Status | Notes |
|--------|--------|-------|
| Description | ✅ | Clear, concise |
| Flag descriptions | ✅ | All flags documented |
| After-help examples | ✅ | 8 practical examples |
| `--format` in examples | ❌ | Missing — only legacy `--json`, `--csv`, `--simple` shown |
| `--timeout` range | ⚠️ | Says "default: 10" but doesn't mention 1-300 range |
| `--csv-delimiter` options | ⚠️ | Says "single character" but doesn't list valid chars (`; | \t`) |
| Config file location | ❌ | Not mentioned anywhere in `--help` |
| History file location | ❌ | Not documented |

### 5.2 README Quality

| Aspect | Status | Notes |
|--------|--------|-------|
| Installation docs | ✅ | Homebrew + from-source |
| Usage examples | ✅ | 6 common use cases |
| Options table | ✅ | Complete flag reference |
| Output format samples | ✅ | All 4 formats shown |
| Rating system docs | ✅ | Table with scores and descriptions |
| `--format` flag | ❌ | **Not documented** — README only shows `--json`, `--csv`, `--simple` |
| `--timeout` range | ❌ | Not mentioned |
| Privacy section | ✅ | Clear data storage explanation |
| System requirements | ❌ | No minimum OS or Rust version stated |
| Troubleshooting | ❌ | No FAQ or common error guidance |

---

## 6. Error & Edge Case UX

### 6.1 Error Messages

| Scenario | Current Behavior | Quality |
|----------|-----------------|---------|
| Server not found | Red "Error: Server not found: ..." | ✅ Good — actionable, suggests `--list` |
| Network failure | Red "Error: Network error: ..." | ✅ Good — shows underlying cause |
| Invalid CSV delimiter | Clear validation message | ✅ Good |
| Invalid IP | Clear validation message | ✅ Good |
| Empty history | "No test history found." | ✅ Good — friendly, not alarming |
| Upload task failure | Bare `eprintln!("Warning: ...")` | ❌ Poor — inconsistent with error styling |

### 6.2 Edge Case Handling

| Edge Case | Handling | Issue |
|-----------|----------|-------|
| All servers filtered | Error with guidance to use `--list` | ✅ Good |
| No ping data (skip mode) | `None` values silently skipped in output | ⚠️ No "skipped" indicator |
| Zero upload speed | `opt_positive()` → `None` → omitted | ⚠️ User can't tell if 0 or skipped |
| History file corruption | Auto-recovery (graceful degrade to empty vec) | ✅ Good |
| Non-TTY JSON output | Compact (not pretty-printed) | ✅ Good |

---

## 7. Consistency Audit

### 7.1 Internal Consistency

| Aspect | Status | Notes |
|--------|--------|-------|
| Label width | ✅ | Consistent 14-char right-aligned labels in sections |
| Section spacing | ✅ | Double-newline between sections |
| Color function pattern | ✅ | All sections accept `nc: bool` parameter |
| Build + format pattern | ✅ | `build_*()` returns String, `format_*()` prints |
| Error format | ⚠️ | `main.rs` errors are styled; `upload.rs` warnings are not |
| Output destination | ✅ | All user output to stderr (except JSON/CSV to stdout) |

### 7.2 External Consistency (vs. industry standards)

| Convention | Status | Notes |
|------------|--------|-------|
| `--version` | ✅ | Standard |
| `--help` / `-h` | ✅ | Standard |
| `NO_COLOR` env | ✅ | Respected |
| Exit code 0/1 | ✅ | Standard |
| JSON to stdout | ✅ | Standard for pipeable output |
| Progress to stderr | ✅ | Standard |
| Shell completions | ✅ | 5 shells supported |
| Man page | ✅ | `netspeed-cli.1` included |
| Config file | ⚠️ | No standard `XDG_CONFIG_HOME` usage — uses `directories` crate |

---

## 8. New Findings

### P0 (User-Blocking)

| ID | Title | Severity | Description |
|----|-------|----------|-------------|
| UI-NEW-001 | No visual distinction between skipped and zero-value tests | MEDIUM | `--no-upload` and actual 0 Mbps upload both result in the upload section being absent. User cannot determine intent vs. failure. |

### P1 (Significant UX Gaps)

| ID | Title | Severity | Description |
|----|-------|----------|-------------|
| UI-NEW-002 | `--format` flag missing from README and help examples | MEDIUM | The newer unified `--format` flag supersedes `--json`/`--csv`/`--simple` but isn't shown in any examples or docs. |
| UI-NEW-003 | `--quiet` / `--silent` flag missing | MEDIUM | No way to suppress all stderr output for headless/cron usage. `--json`/`--csv` suppress progress but not spinners. |
| UI-NEW-004 | Emoji persist under `NO_COLOR` | LOW | Users setting `NO_COLOR` expect plain text, but emojis (⚡🟢🔴 etc.) are still emitted. These are read aloud by screen readers. |
| UI-NEW-005 | Upload warning not `NO_COLOR`-aware | LOW | `eprintln!("Warning: upload task {i} failed: {e}")` in `upload.rs:91` bypasses color check. |

### P2 (Polish)

| ID | Title | Severity | Description |
|----|-------|----------|-------------|
| UI-NEW-006 | "Great" and "Good" share the same 🟢 icon | LOW | Indistinguishable in limited-color terminals or for colorblind users. |
| UI-NEW-007 | No config file path in `--help` | LOW | Users cannot discover where config/history is stored without reading docs. |
| UI-NEW-008 | Simple mode ping lacks "Latency:" label | LOW | Shows `10.0 ms` vs. `Latency: 10.0 ms` — inconsistent with download/upload labels. |
| UI-NEW-009 | No system requirements in README | INFO | Minimum OS, Rust version, terminal compatibility not documented. |
| UI-NEW-010 | Box-drawing chars break screen readers | INFO | `═══` and `╾─` render as garbage to AT tools. |

---

## 9. Score Breakdown

| Dimension | Score | Max | Notes |
|-----------|-------|-----|-------|
| Visual hierarchy | 18 | 20 | Clean, professional layout |
| Color system | 14 | 20 | Emoji/icon ambiguity, `NO_COLOR` gaps |
| Progress feedback | 14 | 15 | Excellent bars/spinners, no quiet mode |
| Accessibility | 10 | 15 | Emojis, box-drawing chars hurt a11y |
| Help & discoverability | 12 | 15 | `--format` missing from docs |
| Error UX | 8 | 10 | Upload warning inconsistency |
| Consistency | 10 | 10 | Strong internal patterns |
| Documentation | 6 | 5 | *(bonus)* README is thorough |
| **Total** | **80** | **100** | **B+** |

---

## 10. Recommended Actions (Priority Order)

### Immediate (P0-P1)
1. **Add skipped indicator** — Show `Upload: — (skipped)` when `--no-upload` is used
2. **Document `--format`** — Add to README options table and `after_help` examples
3. **Add `--quiet` flag** — Suppress all stderr progress output for headless use
4. **Fix upload warning** — Apply `NO_COLOR`-aware formatting

### Near-term (P2)
5. **Differentiate rating icons** — Use distinct icons for "Great" vs "Good"
6. **Strip emojis under `NO_COLOR`** — Check `nc` before adding emoji prefixes
7. **Add config path to `--help`** — Via `after_help` or flag description
8. **Label ping in simple mode** — Consistent `Latency: X ms` format

### Nice-to-have
9. Add system requirements section to README
10. Consider `--no-emoji` flag for users who want colors but not emojis
11. Add troubleshooting/FAQ section to README

---

## Historical Comparison

| Metric | Previous (04-canvas) | Current | Change |
|--------|---------------------|---------|--------|
| Score | 82 | 80 | **-2** |
| Grade | B+ | B+ | — |
| Findings | 4 | 10 | +6 (more thorough audit) |
| 9-state coverage | 78/100 | 78/100 | — |

**Note**: The score decrease reflects a deeper, more rigorous audit — not actual regression. The codebase is the same; the scrutiny is higher.
