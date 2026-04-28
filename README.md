# netspeed-cli

Command line interface for testing internet bandwidth using speedtest.net

[![Crates.io](https://img.shields.io/crates/v/netspeed-cli.svg)](https://crates.io/crates/netspeed-cli)
[![GitHub Release](https://img.shields.io/github/v/release/mapleDevJS/netspeed-cli?label=github&sort=semver)](https://github.com/mapleDevJS/netspeed-cli/releases)
[![Homebrew](https://img.shields.io/homebrew/v/netspeed-cli)](https://formulae.brew.sh/formula/netspeed-cli)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Overview

netspeed-cli is a Rust-based command line tool for testing your internet bandwidth using speedtest.net servers. It provides fast, accurate speed testing with detailed metrics including latency under load, peak speeds, jitter, per-metric letter grades, and an overall connection quality rating.

Runtime behavior highlights:
- CLI configuration precedence is `CLI flags > config file > built-in defaults`.
- Speedtest server URLs from the XML feed are normalized internally, so latency, download, and upload endpoints are derived from the same canonical server definition.
- Corrupted local history files fail safely instead of being silently overwritten.
- Human-readable formats output to **stderr** (so they don't interfere with pipes); machine-readable formats (JSON, JSONL, CSV) output to **stdout**.

## Installation

### Homebrew (macOS/Linux) - Recommended

```bash
# Add the tap (one-time)
brew tap mapleDevJS/homebrew-netspeed-cli

# Install netspeed-cli
brew install netspeed-cli
```

> **Note:** After adding the tap, you can use `brew install netspeed-cli` for all future installations and updates.

### Cargo

```bash
cargo install netspeed-cli
```

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

> **Note:** The CLI uses Unicode box-drawing characters and emoji indicators (set `NO_COLOR=1` or `--no-emoji` for plain-text fallback compatible with screen readers and limited terminals).

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
netspeed-cli --format json
```

Output in CSV format:

```bash
netspeed-cli --format csv
```

Test download speed only:

```bash
netspeed-cli --no-upload
```

View test history:

```bash
netspeed-cli --history
```

Use a user profile:

```bash
netspeed-cli --profile gamer
```

Show config file location:

```bash
netspeed-cli --show-config-path
```

## Options

| Option | Description |
|--------|-------------|
| `--no-download` | Skip download test |
| `--no-upload` | Skip upload test |
| `--single` | Use single connection |
| `--bytes` | Display values in bytes instead of bits |
| `--format TYPE` | Output format: `json`, `jsonl`, `csv`, `minimal`, `simple`, `compact`, `detailed`, `dashboard` (supersedes `--json`, `--csv`, `--simple`) |
| `--json` | Legacy alias for `--format json` |
| `--csv` | Legacy alias for `--format csv` |
| `--simple` | Legacy alias for `--format simple` |
| `--minimal` | Legacy alias for `--format minimal` |
| `--csv-delimiter CHAR` | CSV delimiter character: `,`, `;`, `\|`, or tab (default: `,`) |
| `--csv-header` | Include CSV header row |
| `--quiet` | Suppress all progress output (for cron jobs / CI) |
| `--list` | List available servers |
| `--server ID` | Test against specific server (can be used multiple times) |
| `--exclude ID` | Exclude server from selection (can be used multiple times) |
| `--source IP` | Bind to source IP address |
| `--timeout SEC` | HTTP timeout in seconds (default: 10, range: 1-300) |
| `--theme THEME` | Color theme: `dark`, `light`, `high-contrast`, `monochrome` (default: `dark`) |
| `--profile PROFILE` | User profile: `power-user`, `gamer`, `streamer`, `remote-worker`, `casual` (default: `power-user`) |
| `--no-emoji` | Disable emoji indicators in output |
| `--ca-cert PATH` | Path to custom CA certificate file (PEM/DER) |
| `--tls-version VERSION` | Minimum TLS version: `1.2` or `1.3` |
| `--pin-certs` | Enable certificate pinning for speedtest.net servers |
| `--history` | Show test history with sparkline trends |
| `--dry-run` | Show test configuration without running the test |
| `--show-config-path` | Print the config file path and exit |
| `--strict-config` | Treat config file warnings as errors |
| `--generate-completion SHELL` | Generate shell completion script |
| `--version` | Show version |

## Configuration File

netspeed-cli can be configured via a TOML file. Use `--show-config-path` to find the platform-specific location.

```bash
netspeed-cli --show-config-path
```

Precedence: `CLI flags > config file > built-in defaults`.

### Example `config.toml`

```toml
# Test selection
no_download = false
no_upload = false
single = false
timeout = 10

# Output
format = "detailed"          # json, jsonl, csv, minimal, simple, compact, detailed, dashboard
bytes = false
theme = "dark"               # dark, light, high-contrast, monochrome
profile = "power-user"       # power-user, gamer, streamer, remote-worker, casual
no_emoji = false
quiet = false

# CSV
csv_delimiter = ","
csv_header = false

# TLS
ca_cert = "/path/to/ca.pem"
tls_version = "1.2"
pin_certs = false

# Strict mode (warnings become errors)
strict = false
```

## User Profiles

Profiles adjust scoring weights, grading thresholds, and which sections appear in the output. Use `--profile` or set `profile` in the config file.

| Profile | Description | Notes |
|---------|-------------|-------|
| `power-user` | Full report with all metrics (default) | Shows everything: estimates, percentiles, stability, bufferbloat, UL/DL ratio |
| `gamer` | Optimized for gaming quality | Higher weight on ping and jitter; strict latency thresholds |
| `streamer` | Optimized for live streaming | Higher weight on upload speed and stability |
| `remote-worker` | Optimized for remote work/video calls | Balanced weights with emphasis on jitter and upload |
| `casual` | Simplified output | Relaxed thresholds, fewer sections, minimal output |

Profiles are case-insensitive. Aliases: `poweruser`, `remote`, `remoteworker`.

## Output Formats

### Dashboard

Rich terminal dashboard with 3-column metrics and capability matrix:

```
  ┌────────────────────────────────────────────────────────┐
  │          NetSpeed CLI v0.10.0                         │
  │  Rogers (Toronto) - CA - 12km - 192.168.1.1           │
  └────────────────────────────────────────────────────────┘

  ┌ PERFORMANCE ┬ STABILITY ┬ BUFFERBLOAT ┐
  │  450.2 Mb/s v│ DL: A+    │ Grade: C    │
  │  120.5 Mb/s ^│ UL: A+    │             │
  │    12.1 ms  │           │ Overall: B+ │
  └─────────────┴───────────┴─────────────┘
```

### Compact

Key metrics with quality ratings between simple and detailed:

```
  TEST RESULTS
  Overall: Good

  Latency        12.1 ms    (Good)
  Download     450.23 Mb/s  (Excellent)
  Peak         520.10 Mb/s
  Upload      120.45 Mb/s   (Good)

  Download: 14.6 MB in 3.2s
  Upload: 4.1 MB in 2.1s
  Total time: 5.3s
```

### Detailed (default)

Default full report with overall grading, latency metrics, transfer speeds, connection info, summary totals, total time, and completion timestamp.
When available, it also includes packet loss, bufferbloat, latency under load, variance, and UL/DL ratio.
Profile-driven extras such as transfer estimates, stability analysis, latency percentiles, and history comparison may be appended after the main report when the necessary data is available.

```
  TEST RESULTS
  Overall: Excellent

  Latency:        5.2 ms  (Excellent)
  Jitter:         1.3 ms
  Packet Loss:    0.0%
  Bufferbloat:    A (+7.2 ms)
  Download:     450.23 Mb/s  (Excellent)
  Peak:         520.10 Mb/s
  Latency (load): 12.4 ms  +138% (significant)
  Variance:     +/-4.8% (stable)
  Upload:       120.45 Mb/s  (Good)
  Peak:         145.80 Mb/s
  Latency (load):  8.1 ms  +56% (significant)
  Variance:     +/-8.6% (variable)
  UL/DL Ratio:  3.74x download-heavy

  CONNECTION INFO
  Server:       Rogers (Toronto)
  Location:     CA  (12 km)
  Client IP:    192.168.1.1

  TEST SUMMARY
  Download:     12.4 MB in 3.2s
  Upload:       4.1 MB in 2.1s
  Total:        16.5 MB in 5.3s

  Total time: 5.3s
  Completed at: 2026-04-04T12:00:00Z
```

> **Tip:** Use `--no-download` or `--no-upload` to skip a phase. Skipped tests show `-- (skipped)` in the output.

### Simple

```
Latency: 5.2 ms | Download: 450.23 Mb/s | Upload: 120.45 Mb/s
```

### Minimal

Ultra-compact single line for status bars and scripts:
```
B+  450.2v  120.5^  12ms
```

### JSON

One-line JSON object:
```json
{"status":"ok","server":{"id":"1234",...},"ping":5.2,"download":450230000,"phases":{"ping":{"state":"completed"},"download":{"state":"completed"},"upload":{"state":"completed"}}}
```

Runtime failures in JSON mode also emit a JSON object with a stable error envelope and a non-zero exit code:
```json
{"status":"error","exit_code":69,"timestamp":"2026-04-18T12:00:00Z","error":{"code":"download_failed","category":"network","message":"Download test failed: all streams failed","suggestion":"Try with --single for a simpler test."}}
```

### JSONL

JSON Lines format - one JSON object per line, ideal for logging:
```json
{"server":{"id":"1234",...},"ping":5.2,"download":450230000,...}
{"server":{"id":"1234",...},"ping":4.8,"download":445000000,...}
```

On failure, JSONL emits a single error object line with the same schema as JSON mode.
Successful JSON and JSONL payloads also include per-phase state so scripts can distinguish completed phases from user-skipped phases without inferring from missing metrics.

### CSV

```
Server ID,Sponsor,Server Name,Timestamp,Distance,Ping,Jitter,Packet Loss,Download,Download Peak,Upload,Upload Peak,IP Address
1234,Rogers,Toronto,2026-04-04T12:00:00Z,12.0,5.2,1.3,0.0,450230000.0,520100000.0,120450000.0,145800000.0,192.168.1.1
```

## Features

### Connection Quality Rating

An overall rating combining all metrics:

| Rating | Score | Description |
|--------|-------|-------------|
| Excellent | 90+ | Fiber-grade connection |
| Great | 75-89 | Very good performance |
| Good | 55-74 | Solid everyday connection |
| Fair | 40-54 | Acceptable, some limitations |
| Moderate | 25-39 | Noticeable performance issues |
| Poor | <25 | Significant problems |

### Letter Grades (A+ to F)

Each metric receives a letter grade based on profile-aware thresholds. For example, the `gamer` profile requires lower latency for an A+ than the `casual` profile.

| Grade | Score Range |
|-------|-------------|
| A+ | 97+ |
| A | 93-96 |
| A- | 90-92 |
| B+ | 87-89 |
| B | 83-86 |
| B- | 80-82 |
| C+ | 77-79 |
| C | 73-76 |
| C- | 70-72 |
| D | 60-69 |
| F | <60 |

Grades are assigned to: ping, jitter, download speed, upload speed, bufferbloat, and stability (CV%).

### Latency Under Load

Measures ping latency during download and upload tests to show how your connection degrades under bandwidth saturation. The degradation percentage shows how much worse latency gets compared to idle:

- **< 25%**: Minimal impact -- great for gaming/calls while downloading
- **25-50%**: Moderate impact -- noticeable but manageable
- **> 50%**: Significant impact -- connection struggles under load

### Stability Analysis

Coefficient of Variation (CV%) measures how consistent your speeds are during a test:

- **< 3%**: Very stable
- **3-8%**: Stable
- **> 8%**: Variable

### Latency Percentiles

When latency samples are available, the detailed output includes P50, P95, and P99 values to show the full distribution of your connection's latency.

### Peak Speeds

Shows the maximum burst speed observed during each test phase, helping you understand your connection's capacity beyond just the average.

### Test History

Results are automatically saved after a successful test (up to 100 entries). View with `--history`.

- **Sparkline trends**: visual trend of download/upload speeds across the last 20 tests
- **Comparison**: shows percentage faster/slower than your historical average
- **Atomic writes**: temp file written first, then renamed; previous valid file rotated to `history.json.bak`
- **Corruption recovery**: corrupt primary file is preserved as `history.json.corrupt` before recovery from backup

**To disable history**: machine-readable formats (`--format json`, `--format jsonl`, `--format csv`) skip history saving automatically.

### Usage Estimates

When using profiles with usage targets, the detailed output shows estimated download times for common file sizes (e.g., a 1 GB file, a 50 GB game).

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

- **Server information**: name, sponsor
- **Test metrics**: ping, jitter, packet loss, download/upload speeds and peaks, latency under load (download/upload), timestamps
- **Client IP address**: discovered from speedtest.net during each test

**Storage location**: Platform-specific data directory (via the `directories` crate). Use `--show-config-path` to display the exact path. On Unix systems, the history file is created with `0o600` permissions (owner-only access).

**No data is transmitted** to any server other than speedtest.net infrastructure. No analytics, telemetry, or crash reporting is included.

**To disable history**: machine-readable formats (`--format json`, `--format jsonl`, `--format csv`) skip history saving automatically.

## Verification

After installation, verify your installation worked correctly by running:
```bash
netspeed-cli --version
```
or
```bash
netspeed-cli --help
```

## Security

For security-related documentation and audit procedures, see [docs/security-audit.md](docs/security-audit.md).

To report a security vulnerability, please follow our [Security Policy](SECURITY.md#reporting-a-vulnerability).

## License

MIT License - see [LICENSE](LICENSE) for details.
