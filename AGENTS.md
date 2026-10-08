# netspeed-cli

Rust CLI for testing internet bandwidth via speedtest.net. Published via Homebrew and Cargo.

## Global Skills (Auto-Loaded)

When working on this project, apply these global skills:

| Skill | When to Use |
|---|---|
| `test-master` | Unit tests, integration tests, test strategies |
| `clean-code-octagon` | Code reviews, architecture audits |
| `release-agent` | Conventional Commits, SemVer, CI enforcement |

## Key Docs

- `README.md` — Installation and usage
- `CONTRIBUTING.md` — Contribution guidelines
- `SECURITY.md` — Security policy
- `docs/architecture.md` — System design
- `HOMEBREW_PUBLISHING.md` — Release process for Homebrew

## Commands

```bash
cargo build          # Build
cargo test           # Run tests
cargo clippy --all-targets --all-features -- -D warnings  # Lint (mirrors CI)
cargo fmt --check    # Format check
just lint            # Format + clippy (mirrors CI exactly)
just qa              # Full CI gate: fmt + clippy + tests + doc + deny
just install-hooks   # Install pre-push hook (runs `just qa` before every push)
netspeed-cli         # Run speed test
```

## Pre-Push Hook

A git pre-push hook is available that runs `just qa` before every push. Install once with `just install-hooks`. The hook prevents pushes that would fail CI.

## Architecture

- **Language**: Rust
- **Distribution**: Homebrew (macOS/Linux), Cargo
- **API**: speedtest.net servers
- **Metrics**: Latency, peak speeds, jitter, connection quality rating

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)
