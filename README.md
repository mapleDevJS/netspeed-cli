# netspeed-cli

Command line interface for testing internet bandwidth using speedtest.net

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Overview

netspeed-cli is a Rust-based command line tool for testing your internet bandwidth using speedtest.net servers. It provides fast, accurate speed testing with a simple command-line interface.

## Installation

### From source

```bash
git clone https://github.com/yourusername/netspeed-cli.git
cd netspeed-cli
cargo build --release
./target/release/netspeed-cli
```

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

Share results with a URL:

```bash
netspeed-cli --share
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
| `--csv` | Output in CSV format |
| `--json` | Output in JSON format |
| `--list` | List available servers |
| `--server ID` | Test against specific server |
| `--exclude ID` | Exclude server from selection |
| `--source IP` | Bind to source IP address |
| `--timeout SEC` | HTTP timeout in seconds |
| `--version` | Show version |

## Output Formats

### Simple

```
Ping: 15.234 ms
Download: 150.45 Mbit/s
Upload: 50.12 Mbit/s
```

### JSON

```json
{
  "ping": 15.234,
  "download": 150450000,
  "upload": 50120000,
  "server": {
    "id": "1234",
    "name": "Server Name",
    "sponsor": "ISP"
  }
}
```

### CSV

```
Server ID,Sponsor,Server Name,Timestamp,Ping,Download,Upload
1234,ISP,Server Name,2026-04-04T12:00:00Z,15.234,150450000,50120000
```

## Building from Source

### Requirements

- Rust 1.70+
- cargo

```bash
cargo build --release
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.
