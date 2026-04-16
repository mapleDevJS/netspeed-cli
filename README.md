# netspeed-cli

Command line interface for testing internet bandwidth using speedtest.net

[![Crates.io](https://img.shields.io/crates/v/netspeed-cli.svg)](https://crates.io/crates/netspeed-cli)
[![GitHub Release](https://img.shields.io/github/v/release/mapleDevJS/netspeed-cli?label=github&sort=semver)](https://github.com/mapleDevJS/netspeed-cli/releases)
[![Homebrew](https://img.shields.io/homebrew/v/netspeed-cli)](https://formulae.brew.sh/formula/netspeed-cli)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Overview

netspeed-cli is a Rust-based command line tool for testing your internet bandwidth using speedtest.net servers. It provides fast, accurate speed testing with detailed metrics including latency under load, peak speeds, jitter, and an overall connection quality rating.

## Installation

### Homebrew (macOS/Linux) - Recommended

```bash
# Add the tap (one-time)
brew tap mapleDevJS/homebrew-netspeed-cli

# Install netspeed-cli
brew install netspeed-cli
```

> **Note:** After adding the tap, you can use `brew install netspeed-cli` for all future installations and updates.

### Direct Download

Pre-built binaries are available for download at:
[https://github.com/mapleDevJS/netspeed-cli/releases/latest](https://github.com/mapleDevJS/netspeed-cli/releases/latest)

### From source

```bash
git clone https://github.com/mapleDevJS/netspeed-cli.git
cd netspeed-cli
cargo build --release
./target/release/netspeed-cli
```

> **Note:** The CLI is officially supported on macOS 12+, Linux (kernel 5.4+), and Windows 10+. While it may work on other Unix-like systems, it's not guaranteed.

## System Requirements

| Requirement | Details |
|-------------|---------|
| **OS** | macOS 12+, Linux (kernel 5.4+) |
| **Rust** | 1.86+ (for building from source) |
| **Terminal** | Any Unicode-capable terminal (UTF-8) |
| **Network** | Internet access to speedtest.net servers |
| **Architecture** | x86_64, aarch64 (Apple Silicon, ARM Linux) |

> **Note:** The CLI uses Unicode box-drawing characters (`═`, `─`, `╾`) and emoji indicators (⚡, 🟢, etc.). Set `NO_COLOR=1` in your environment for a plain-text fallback compatible with screen readers and limited terminals.

## Usage

```bash
$ netspeed-cli --help
```

### Basic Usage

Test your connection automatically:

```bash
netspeed-cli
```

Test against a specific server:

```bash
netspeed-cli --server 1234
```

Output in JSON format:

```bash
netspeed-cli --json
```

Output in CSV format:

```bash
netspeed-cli --csv
```

Test download speed only:

```bash
netspeed-cli --no-upload
```

View test history:

```bash
netspeed-cli --history
```

## Options

| Option | Description |
|--------|-------------|
| `--no-download` | Skip download test |
| `--no-upload` | Skip upload test |
| `--single` | Use single connection |
| `--bytes` | Display values in bytes instead of bits |
| `--simple` | Show minimal output |
| `--format TYPE` | Output format: `json`, `jsonl`, `csv`, `minimal`, `simple`, `compact`, `detailed`, `dashboard` (supersedes `--json`, `--csv`, `--simple`) |
| `--csv` | Output in CSV format |
| `--csv-delimiter CHAR` | CSV delimiter character: `,`, `;`, `\|`, or tab (default: `,`) |
| `--csv-header` | Include CSV header row |
| `--json` | Output in JSON format |
| `--quiet` | Suppress all progress output (for cron jobs / CI) |
| `--list` | List available servers |
| `--server ID` | Test against specific server (can be used multiple times) |
| `--exclude ID` | Exclude server from selection (can be used multiple times) |
| `--source IP` | Bind to source IP address |
| `--timeout SEC` | HTTP timeout in seconds (default: 10, range: 1–300) |
| `--theme THEME` | Color theme: `dark`, `light`, `high-contrast`, `monochrome` (default: `dark`) |
| `--history` | Show test history |
| `--generate-completion SHELL` | Generate shell completion script |
| `--version` | Show version |

## Output Formats

### Dashboard

Rich terminal dashboard with 3-column metrics and capability matrix:

```
  ╭────────────────────────────────────────────────────────╮
  │          NetSpeed CLI v0.8.0                          │
  │  Rogers (Toronto) • CA • 12km • 192.168.1.1            │
  ╰────────────────────────────────────────────────────────╯

  ┌ PERFORMANCE ┬ STABILITY ┬ BUFFERBLOAT ┐
  │  450.2 Mb/s ↓│ DL: A+    │ Grade: C    │
  │  120.5 Mb/s ↑│ UL: A+    │             │
  │    12.1 ms  │           │ Overall: B+ │
  └─────────────┴───────────┴─────────────┘
```

### Compact

Key metrics with quality ratings between simple and detailed:

```
  TEST RESULTS
  Overall: 🟢 Good

  Latency        12.1 ms    (Good)
  Download     450.23 Mb/s  (Excellent)
  Peak         520.10 Mb/s
  Upload      120.45 Mb/s   (Good)

  Download: 14.6 MB in 3.2s
  Upload: 4.1 MB in 2.1s
  Total time: 5.3s
```

### Detailed (Default)

Full analysis with per-metric grades, stability metrics, and variance data.

### Minimal

Ultra-compact single line for status bars and scripts:
```
B+  450.2↓  120.5↑  12ms
```
  Upload:        50.45 Mb/s  ██████████████░░░░░░░░░░░░     (2.1s, 5.0 MB)
  Peak:          60.00 Mb/s

  ── History (recent tests) ───────────────────────────────
  DL: ▃ ▅ ▄ ▇ █ ▆ ▅
  UL: ▂ ▄ ▃ ▅ ▇ ▅ ▄
  Apr 5  ⚡ 445.0↓ / 118.0↑ Mb/s
  Apr 4  🟢 412.0↓ / 115.0↑ Mb/s
  Apr 3  ⚡ 498.0↓ / 122.0↑ Mb/s

  Tip: Use --list to see servers, --history for full history
```

Run with `netspeed-cli --format dashboard`.

### Detailed (default)

```
  TEST RESULTS
  Overall: ⚡ Excellent

  Latency:        5.2 ms  (⚡ Excellent)
  Jitter:         1.3 ms
  ──────────────────────────────
  Download:     450.23 Mb/s  ████████████████████░░░░░░░░  (⚡ Excellent)
  Peak:         520.10 Mb/s
  Latency (load): 12.4 ms  +138% (significant)
  Upload:       120.45 Mb/s  ██████████████░░░░░░░░░░░░    (🟢 Good)
  Peak:         145.80 Mb/s
  Latency (load):  8.1 ms  +56% (significant)
  ──────────────────────────────
  Connection Info
  Server:       Rogers (Toronto)
  Location:     CA  (12 km)
  Client IP:    192.168.1.1
  ──────────────────────────────
  Test Summary
  Download:     12.4 MB in 3.2s
  Upload:       4.1 MB in 2.1s
  Total:        16.5 MB in 5.3s

  Completed at: 2026-04-04T12:00:00Z
```

> **Tip:** Use `--no-download` or `--no-upload` to skip a phase. Skipped tests show `— (skipped)` in the output:
> ```
>   Download:     450.23 Mb/s  (⚡ Excellent)
>   Upload:       — (skipped)
> ```

### Simple

```
Latency: 5.2 ms | Download: 450.23 Mb/s | Upload: 120.45 Mb/s
```

### Minimal

Ultra-compact single line for status bars and scripts:
```
B+  450.2↓  120.5↑  12ms
```

### JSON

One-line JSON object:
```json
{"server":{"id":"1234",...},"ping":5.2,"download":450230000,...}
```

### JSONL

JSON Lines format - one JSON object per line, ideal for logging:
```json
{"server":{"id":"1234",...},"ping":5.2,"download":450230000,...}
{"server":{"id":"1234",...},"ping":4.8,"download":445000000,...}
```

### CSV

```
Server ID,Sponsor,Server Name,Timestamp,Distance,Ping,Jitter,Download,Download Peak,Upload,Upload Peak,IP Address
1234,Rogers,Toronto,2026-04-04T12:00:00Z,12.0,5.2,1.3,450230000.0,520100000.0,120450000.0,145800000.0,192.168.1.1
```

## Features

### Connection Quality Rating

An overall rating combining all metrics:

| Rating | Score | Description |
|--------|-------|-------------|
| Excellent | 90+ | ⚡ Fiber-grade connection |
| Great | 75-89 | 🔵 Very good performance |
| Good | 55-74 | 🟢 Solid everyday connection |
| Fair | 40-54 | 🟡 Acceptable, some limitations |
| Moderate | 25-39 | 🟠 Noticeable performance issues |
| Poor | <25 | 🔴 Significant problems |

### Latency Under Load

Measures ping latency during download and upload tests to show how your connection degrades under bandwidth saturation. The degradation percentage shows how much worse latency gets compared to idle:

- **< 25%** (green): Minimal impact — great for gaming/calls while downloading
- **25-50%** (yellow): Moderate impact — noticeable but manageable
- **> 50%** (red): Significant impact — connection struggles under load

### Peak Speeds

Shows the maximum burst speed observed during each test phase, helping you understand your connection's capacity beyond just the average.

### Test History

Results are automatically saved and can be viewed with `--history`.

## Building from Source

### Requirements

- Rust 1.86+
- cargo

```bash
cargo build --release
cargo test
```

## Privacy

netspeed-cli stores test results locally for historical comparison. The following data is saved:

- **Server information**: name, sponsor, country, distance
- **Test metrics**: ping, jitter, download/upload speeds, timestamps
- **Client IP address**: discovered from speedtest.net during each test

**Storage location**: Platform-specific data directory (via the `directories` crate). On Unix systems, the history file is created with `0o600` permissions (owner-only access).

**No data is transmitted** to any server other than speedtest.net infrastructure. No analytics, telemetry, or crash reporting is included.

**To disable history**: Results are only saved after a successful test. Use `--json` or `--csv` output to suppress history saving (these modes output to stdout only).

## Verification

After installation, verify your installation worked correctly by running:
```bash
netspeed-cli --version
```
or
```bash
netspeed-cli --help
```

## License

MIT License - see [LICENSE](LICENSE) for details.
```