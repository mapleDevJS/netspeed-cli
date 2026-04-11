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

### From source

```bash
git clone https://github.com/mapleDevJS/netspeed-cli.git
cd netspeed-cli
cargo build --release
./target/release/netspeed-cli
```

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
| `--format TYPE` | Output format: `json`, `csv`, `simple`, `detailed`, `dashboard` (supersedes `--json`, `--csv`, `--simple`) |
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
| `--history` | Show test history |
| `--generate-completion SHELL` | Generate shell completion script |
| `--version` | Show version |

## Output Formats

### Dashboard

```
  ╔══════════════════ netspeed-cli v0.5.0 ═══════════════════╗
  ║  Server: Rogers (Toronto) · CA · 12km                    ║
  ║  Client IP: 192.168.1.1                                  ║
  ╚══════════════════════════════════════════════════════════╝

  Latency    ████████████████████████████████    5.2 ms  ⚡ Excellent
  Download   ████████████████████░░░░░░░░       450.23 Mb/s  ⚡ Excellent
  Upload     ██████████████░░░░░░░░░░░░         120.45 Mb/s  🟢 Good

  ── Summary ──────────────────────────────────────────────────
  Download:     450.23 Mb/s  ████████████████████░░░░░░░░  (3.2s, 14.6 MB)
  Peak:         520.10 Mb/s
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

### JSON

```json
{
  "server": {
    "id": "1234",
    "name": "Toronto",
    "sponsor": "Rogers",
    "country": "CA",
    "distance": 12.0
  },
  "ping": 5.2,
  "jitter": 1.3,
  "download": 450230000.0,
  "download_peak": 520100000.0,
  "upload": 120450000.0,
  "upload_peak": 145800000.0,
  "latency_download": 12.4,
  "latency_upload": 8.1,
  "timestamp": "2026-04-04T12:00:00Z",
  "client_ip": "192.168.1.1"
}
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

## License

MIT License - see [LICENSE](LICENSE) for details.
