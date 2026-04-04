# netspeed-cli

[![CI](https://github.com/alexeyivanov/netspeed-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/alexeyivanov/netspeed-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.70+](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Audit](https://github.com/alexeyivanov/netspeed-cli/actions/workflows/ci.yml/badge.svg?event=push&job=audit)](https://github.com/alexeyivanov/netspeed-cli/actions/workflows/ci.yml)

Command line interface for testing internet bandwidth using speedtest.net

## Overview

netspeed-cli is a Rust-based command line tool for testing your internet bandwidth using speedtest.net servers. It provides fast, accurate speed testing with a simple command-line interface.

**Features:**
- Automatic server selection based on geographic distance and latency
- Concurrent download/upload testing for accurate results
- Multiple output formats: simple, JSON, CSV
- Shell completion generation
- Speedtest Mini server support
- Shareable results URLs

## Installation

### From source

```bash
git clone https://github.com/alexeyivanov/netspeed-cli.git
cd netspeed-cli
cargo build --release
./target/release/netspeed-cli --help
```

### Requirements

- Rust 1.70 or later
- Internet connection

## Usage

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

Share results with a URL:

```bash
netspeed-cli --share
```

List available servers:

```bash
netspeed-cli --list
```

## Options

| Option | Description |
|--------|-------------|
| `--no-download` | Skip download test |
| `--no-upload` | Skip upload test |
| `--single` | Use single connection |
| `--bytes` | Display values in bytes instead of bits |
| `--share` | Generate shareable results URL |
| `--simple` | Show minimal output |
| `-v, --verbose` | Enable debug logging (or set `RUST_LOG`) |
| `--csv` | Output in CSV format |
| `--csv-delimiter CHAR` | CSV delimiter character (default: `,`) |
| `--csv-header` | Include CSV header row |
| `--json` | Output in JSON format |
| `--list` | List available servers |
| `--server ID` | Test against specific server (repeatable) |
| `--exclude ID` | Exclude server from selection (repeatable) |
| `--source IP` | Bind to source IP address |
| `--timeout SEC` | HTTP timeout in seconds (default: 10) |
| `--secure` | Use HTTPS |
| `--generate-completion SHELL` | Generate shell completion script |

## Output Formats

### Simple (default)

```
Ping: 15.234 ms
Download: 150.45 Mbit/s
Upload: 50.12 Mbit/s
```

### JSON

```json
{
  "server": {
    "id": "1234",
    "name": "Server Name",
    "sponsor": "ISP",
    "country": "US",
    "distance": 10.5
  },
  "ping": 15.234,
  "download": 150450000.0,
  "upload": 50120000.0,
  "timestamp": "2026-04-04T12:00:00+00:00",
  "client_ip": "1.2.3.4"
}
```

### CSV

```csv
Server ID,Sponsor,Server Name,Timestamp,Distance,Ping,Download,Upload,Share,IP Address
1234,ISP,Server Name,2026-04-04T12:00:00+00:00,10.5,15.234,150450000.0,50120000.0,,1.2.3.4
```

## Logging & Debugging

netspeed-cli uses structured logging via the `tracing` crate. You can control log output in two ways:

### Via CLI flag

```bash
netspeed-cli -v        # Enable debug-level logging
netspeed-cli --verbose # Same as -v
```

### Via environment variable

The `RUST_LOG` variable provides fine-grained control:

```bash
# Debug all modules
RUST_LOG=debug netspeed-cli

# Info level for all modules
RUST_LOG=info netspeed-cli

# Debug only server discovery
RUST_LOG=netspeed_cli::discovery=debug netspeed-cli

# Trace HTTP requests
RUST_LOG=reqwest=debug netspeed-cli
```

### Example debug output

```
$ RUST_LOG=debug netspeed-cli
2026-04-04T12:00:00.000000Z  INFO Discovering servers...
2026-04-04T12:00:00.100000Z  INFO Testing against server sponsor="ISP 1" name="New York"
2026-04-04T12:00:00.100000Z DEBUG Ping test complete ping_ms=15.234
2026-04-04T12:00:05.200000Z DEBUG Transfer complete chunks=16 bytes=5600000
2026-04-04T12:00:05.200000Z  INFO Download test complete download_mbps=150.45
2026-04-04T12:00:10.300000Z  INFO Upload test complete upload_mbps=50.12
```

## Building from Source

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run clippy (linter)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

## Shell Completions

Shell completions are automatically generated during build. Find them in the `completions/` directory:

- **Bash**: `completions/netspeed-cli.bash`
- **Zsh**: `completions/_netspeed-cli`
- **Fish**: `completions/netspeed-cli.fish`
- **PowerShell**: `completions/_netspeed-cli.ps1`
- **Elvish**: `completions/netspeed-cli.elv`

To install Bash completions:

```bash
source completions/netspeed-cli.bash
```

## Architecture

The project is organized into modules:

```
src/
├── lib.rs          # Library entry point
├── main.rs         # Binary entry point
├── cli.rs          # CLI argument definitions
├── config.rs       # Application configuration
├── discovery.rs    # Server discovery logic
├── download.rs     # Download speed testing
├── upload.rs       # Upload speed testing
├── servers.rs      # Server list fetching/parsing
├── http.rs         # HTTP client utilities
├── share.rs        # Share URL generation
├── formatter.rs    # Output formatting
├── presenter.rs    # Result presentation
├── progress.rs     # Progress tracking
├── runner.rs       # Test orchestration
├── mini.rs         # Speedtest Mini support
├── completions.rs  # Shell completion generation
├── types.rs        # Core data types
├── utils.rs        # Utility functions
└── error.rs        # Error types
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo test && cargo clippy`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.
