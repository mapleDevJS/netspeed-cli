---
name: netspeed-cli
description: Domain knowledge for netspeed-cli — Rust bandwidth testing CLI via speedtest.net. Architecture, domain rules, distribution, and key decisions.
---

# netspeed-cli — Project Knowledge

Auto-generated from APEX refactor cycle (Mode 3B). Load when working on this project.

## Project Overview

| Field | Value |
|-------|-------|
| **Type** | CLI tool — internet bandwidth testing |
| **Language** | Rust 2024 Edition |
| **MSRV** | 1.86 |
| **Version** | 0.6.0 (next: 0.7.0) |
| **License** | MIT |
| **Repository** | github.com/mapleDevJS/netspeed-cli |

## Architecture

```
Binary (main.rs) → Orchestrator → {Servers → Ping → Download → Upload} → Formatter
                                            ↑
                                    bandwidth_loop.rs (shared hot path)
```

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Entry point, tokio runtime, error display |
| `cli.rs` | Clap argument parsing, includes `validate.rs` |
| `config.rs` | CLI + TOML config merge (OR semantics) |
| `orchestrator.rs` | Test lifecycle: fetch → select → ping → dl → ul → output |
| `http.rs` | Client creation, IP discovery, `request_with_retry()` |
| `servers.rs` | Server fetch (XML), distance calc, ping test, selection |
| `download.rs` | Multi-stream download via `BandwidthLoopState` |
| `upload.rs` | Multi-stream upload via `BandwidthLoopState` |
| `bandwidth_loop.rs` | Shared: throttle gate, peak tracking, sampling, progress |
| `formatter/` | Strategy pattern: JSON, CSV, Simple, Detailed output |
| `history.rs` | Persistent test results (JSON, 0o600, atomic write, max 100) |
| `progress.rs` | Progress bars, spinners, NO_COLOR support |
| `validate.rs` | IP/timeout validation, `include!()`-ed by `build.rs` |
| `types.rs` | Server, TestResult, ServerInfo, CsvOutput |

## Domain Rules

### Rating Thresholds
| Rating | Score | Description |
|--------|-------|-------------|
| Excellent | ≥90 | Fiber-grade |
| Great | 75-89 | Very good |
| Good | 55-74 | Solid everyday |
| Fair | 40-54 | Acceptable |
| Moderate | 25-39 | Noticeable issues |
| Poor | <25 | Significant problems |

### Speed Rating (Mbps)
| Rating | Mbps |
|--------|------|
| Excellent | ≥500 |
| Great | ≥200 |
| Good | ≥100 |
| Fair | ≥50 |
| Moderate | ≥25 |
| Slow | ≥10 |
| Very Slow | <10 |

### Bufferbloat Grades (added latency under load)
| Grade | Added ms |
|-------|----------|
| A | <5 |
| B | <20 |
| C | <50 |
| D | <100 |
| F | ≥100 |

### Speedtest.net API Endpoints
- Server list: `https://www.speedtest.net/speedtest-servers-static.php` (XML)
- Client config: `https://www.speedtest.net/api/ios-config.php` (XML)
- IP discovery: `https://www.speedtest.net/api/ip.php` (plain text)
- Ping: `{server_url}/latency.txt`
- Download: `{base_url}/random{size}.jpg` (2000x2000, 3000x3000, 3500x3500, 4000x4000)
- Upload: `{server_url}/upload`

### Network Defaults
- HTTP timeout: 10 seconds (configurable, 1-300)
- Download streams: 4 concurrent (1 with `--single`)
- Upload streams: 4 concurrent (1 with `--single`)
- Retry: 2 attempts, 500ms fixed backoff on network errors
- Sample rate: 20 Hz max (50ms throttle gate)
- Download rounds: 4 per stream
- Upload rounds: 4 per stream, 200KB chunks

## Key Architecture Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| DEC-001 | `bandwidth_loop.rs` shared module | Eliminates ~150 lines of download/upload duplication (DRY-001) |
| DEC-002 | `request_with_retry()` inline retry | No new dependency needed for CLI tool |
| DEC-003 | History: atomic write + 0o600 | Crash safety + privacy on shared systems |
| DEC-004 | Coordinate validation on XML parse | Reject malformed server data |
| DEC-005 | Upload checks `response.status().is_success()` | HTTP 500 no longer counted as success |
| DEC-006 | `validate.rs` uses `include!()` | build.rs cannot depend on lib crate; DRY over IDE convenience |
| DEC-007 | Config merge: `cli \|\| file` (OR semantics) | File acts as persistent default; documented in code |
| DEC-008 | rustls (not native-tls) | No OpenSSL dependency, better cross-platform |
| DEC-009 | HTTP/1 only, no gzip compression | Matches speedtest.net server behavior |

## Commands

| Command | Description |
|---------|-------------|
| `netspeed-cli` | Full speed test |
| `netspeed-cli --list` | List servers |
| `netspeed-cli --history` | Show test history |
| `netspeed-cli --json` | JSON output |
| `netspeed-cli --csv` | CSV output |
| `netspeed-cli --simple` | Minimal output |
| `netspeed-cli --no-download` | Skip download |
| `netspeed-cli --no-upload` | Skip upload |
| `netspeed-cli --single` | Single connection |
| `netspeed-cli --server ID` | Specific server |
| `netspeed-cli --source IP` | Bind to IP |
| `netspeed-cli --timeout SEC` | HTTP timeout |
| `netspeed-cli --generate-completion SHELL` | Shell completion |

## Distribution

| Channel | Details |
|---------|---------|
| **crates.io** | `cargo install netspeed-cli` |
| **Homebrew** | `brew tap mapleDevJS/homebrew-netspeed-cli && brew install netspeed-cli` |
| **GitHub Releases** | 7 platforms: x86_64/aarch64 Linux (gnu+musl), macOS, Windows |
| **CI** | GitHub Actions: test (3 OS), clippy, fmt, doc, deny, msrv, coverage |
| **Release** | Tag on main → CI builds → GH Release + Homebrew update + crates.io publish |

## Commands

```bash
cargo build          # Build
cargo test           # Run tests (~194 tests)
cargo clippy         # Lint (-D warnings)
cargo fmt --check    # Format check
cargo deny check     # Dependency audit
cargo bench          # Benchmarks (Criterion)
```

## Quality Scores (as of 2026-04-06)

| Dimension | Score | Grade |
|-----------|-------|-------|
| Security | 38/40 | A- |
| Maintainability | 22/25 | A- |
| Architecture | 18/20 | A- |
| Performance | 8/10 | B+ |
| Testability | 5/5 | A |
| **Total** | **91/100** | **A** |
