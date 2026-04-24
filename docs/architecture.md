# Architecture & Design Decisions

This document describes the architectural decisions and design patterns used in netspeed-cli.

## Architecture Overview

netspeed-cli follows a **modular layered architecture** with clear separation between:
- **CLI Layer** (`cli.rs`, `config.rs`) — Argument parsing and configuration merging
- **Core Layer** (`download.rs`, `upload.rs`, `servers.rs`) — Network operations
- **Protocol Layer** (`endpoints.rs`) — Canonical speedtest endpoint derivation
- **Orchestration Layer** (`test_runner.rs`) — Test coordination and result aggregation
- **Presentation Layer** (`formatter/`) — Output formatting with Strategy pattern
- **Infrastructure Layer** (`http.rs`, `history.rs`, `progress.rs`) — Cross-cutting concerns

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│                   (Entry Point & Flow)                       │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   cli.rs     │   config.rs  │  formatter/  │   history.rs   │
│  (Parsing)   │  (Merging)   │ (Strategy)   │ (Persistence)  │
├──────────────┴──────────────┴──────────────┴────────────────┤
│                     test_runner.rs                           │
│               (Template Method Pattern)                      │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ download.rs  │  upload.rs   │  servers.rs  │   progress.rs  │
│ (Bandwidth)  │ (Bandwidth)  │ (Discovery)  │   (UI/UX)      │
├──────────────┴──────────────┬──────────────┴────────────────┤
│        endpoints.rs         │                                │
│   (Speedtest URL model)     │                                │
├──────────────┴──────────────┴──────────────┴────────────────┤
│                    common.rs                                 │
│              (Shared Pure Functions)                         │
├─────────────────────────────────────────────────────────────┤
│                    error.rs                                  │
│              (Unified Error Types)                           │
└─────────────────────────────────────────────────────────────┘
```

## Design Patterns

### 1. Strategy Pattern — Output Formatting

The `formatter` module uses the Strategy pattern to dispatch to different output formats:

```rust
pub enum OutputFormat {
    Detailed { dl_bytes, ul_bytes, dl_duration, ul_duration },
    Simple,
    Json,
    Csv { delimiter, header },
}

impl OutputFormat {
    pub fn format(&self, result: &TestResult, bytes: bool) -> Result<(), SpeedtestError> {
        match self {
            OutputFormat::Detailed { .. } => format_detailed(result, bytes),
            OutputFormat::Simple => format_simple(result, bytes),
            OutputFormat::Json => format_json(result),
            OutputFormat::Csv { .. } => format_csv(result, delimiter, header),
        }
    }
}
```

**Why:** Eliminates conditional branching in `main.rs`, makes adding new formats trivial (just add a variant and implementation).

**Sub-modules:**
- `sections.rs` — Header, results, connection info, summary sections
- `ratings.rs` — Connection quality rating calculation
- `stability.rs` — Latency stability analysis
- `estimates.rs` — Time-based download/upload estimates

### 2. Template Method Pattern — Test Runner

The `test_runner::run_bandwidth_test` function implements a template method:

```
1. Set up progress tracking
2. Spawn background latency monitoring
3. Execute the bandwidth test (via closure)
4. Stop latency monitoring
5. Aggregate results
```

**Why:** Download and upload tests share identical orchestration logic. The closure parameter (`test_fn`) allows injection of the specific network operation while keeping the flow consistent.

### 3. Protocol Normalization — `endpoints.rs`

The `endpoints` module converts the raw speedtest server URL into a canonical
set of runtime endpoints:

- `base()` — directory containing test assets
- `upload()` — upload endpoint
- `latency()` — latency probe endpoint
- `download_asset(name)` — download asset URL

**Why:** Protocol assumptions used to be duplicated across download, upload,
and latency measurement code. Centralizing them prevents drift and makes
regressions easier to test.

### 4. Pure Function Isolation — `common.rs`

All shared utilities are pure functions with no side effects:
- `calculate_bandwidth(bytes, elapsed)` → `bps`
- `determine_stream_count(single)` → `usize`
- `format_distance(km)` → `String`
- `format_data_size(bytes)` → `String`
- `is_valid_ipv4(s)` → `bool`

**Why:** Pure functions are trivially testable, reusable across modules, and have no hidden dependencies.

## Error Handling Strategy

### Unified Error Enum with `thiserror`

```rust
#[derive(Debug, Error)]
pub enum SpeedtestError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("{msg}")]
    Context { msg: String, source: Option<Box<dyn Error + Send + Sync>> },
    // ...
}
```

**Design decisions:**
- **`thiserror`** for automatic `Display` and `From` implementations
- **Error chain preservation** via `#[from]` and `source` field in `Context` variant
- **No `anyhow`** — we need a specific error type for the library API, not just a binary
- **Contextual errors** — `SpeedtestError::context()` and `with_source()` for adding domain-specific messages
- **Machine-readable failures** — `--format json` and `--format jsonl` emit a stable error envelope on runtime failure so automation does not need to scrape stderr prose
- **Machine-readable success metadata** — successful JSON payloads include top-level `status` plus explicit per-phase completion/skipped state

## Configuration Merging

Configuration follows a **three-tier merge strategy**:

```
CLI args (highest priority)
    ↓
Config file (~/.config/netspeed-cli/config.toml)
    ↓
Hardcoded defaults (lowest priority)
```

**Why:** Users can set persistent defaults in the config file while overriding per-invocation via CLI flags.

Mergeable boolean CLI flags are parsed as tri-state values so the application
can distinguish:
- flag omitted
- flag explicitly enabled
- config/default fallback

## History Persistence

- **Atomic writes** — history is written through a temp file and renamed into place
- **Backup rotation** — the previous valid file is copied to `history.json.bak`
- **Repair path** — if `history.json` is corrupt but the backup is valid, load/save operations recover from the backup and preserve the corrupt file as `history.json.corrupt`

## Concurrency Model

### Async Runtime
- **Tokio** with `#[tokio::main]` for the async runtime
- **HTTP/1 only** — Speedtest.net servers don't support HTTP/2 consistently
- **No gzip** — Raw bytes needed for accurate bandwidth measurement

### Multi-stream Testing
- **4 concurrent streams** by default (simulates real browser behavior)
- **1 stream** with `--single` flag (debugging/analysis mode)
- **Atomic counters** (`AtomicU64`) for thread-safe byte counting
- **Mutex-protected samples** for speed sample collection

### Latency Under Load
- **Background task** pings server every 100ms during bandwidth test
- **AtomicBool** for clean shutdown signaling
- **Sample aggregation** after test completion

## Performance Decisions

### Release Profile
```toml
[profile.release]
lto = "thin"        # Link-time optimization for better performance
opt-level = "3"     # Maximum speed optimization (network-bound workload)
codegen-units = 1   # Single codegen unit for better inlining
strip = true        # Remove debug symbols to reduce binary size
```

**Why `opt-level = "3"` over `"z"`:**
- Network I/O is the bottleneck, not binary size
- Maximum speed optimization benefits the bandwidth calculation loop
- Binary size difference is negligible for a CLI tool

### Streaming Downloads
- **`reqwest::bytes_stream()`** — Process chunks as they arrive, no buffering
- **Progress updates every 50ms** — Balance between responsiveness and overhead
- **Dynamic estimated total** — Adjusts as actual data is downloaded

## Testing Strategy

### Unit Tests
- **Pure functions** in `common.rs`, `formatter/`, `error.rs` — 100% coverage target
- **Serialization/deserialization** — XML, JSON, CSV parsing tests
- **Algorithm validation** — Haversine distance, bandwidth calculation

### Integration Tests
- **Mock network** — `wiremock` for HTTP endpoint simulation
- **CLI behavior** — End-to-end test runs with mock servers
- **Config file** — TOML loading and merge behavior

### Benchmarks
- **Criterion** for statistical benchmarking
- **Core functions** — Bandwidth calculation, distance, formatting, validation
- **CI integration** — Benchmarks run on PRs to detect regressions

## Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| `cli.rs` | Argument parsing with clap, input validation |
| `common.rs` | Pure utility functions (bandwidth, formatting, validation) |
| `config.rs` | Three-tier config merge (CLI > file > defaults) |
| `download.rs` | Multi-stream download bandwidth measurement |
| `endpoints.rs` | Canonical endpoint derivation from raw server URLs |
| `upload.rs` | Multi-stream upload bandwidth measurement |
| `error.rs` | Unified error types with thiserror |
| `formatter/` | Output formatting (Strategy pattern) |
| `history.rs` | Persistent test result storage and retrieval |
| `http.rs` | HTTP client creation, client IP discovery |
| `progress.rs` | Terminal progress bars and spinners |
| `servers.rs` | Server discovery, distance calculation, ping testing |
| `test_runner.rs` | Test orchestration with template method |
| `types.rs` | Shared data structures (Server, TestResult) |

## Dependencies Rationale

| Dependency | Purpose | Why This One |
|-----------|---------|-------------|
| `clap` | CLI argument parsing | Industry standard, derive API, shell completions |
| `reqwest` | HTTP client | Async, rustls support, streaming |
| `tokio` | Async runtime | Dominant async ecosystem, full feature set |
| `serde` | Serialization | Rust standard, derive macros |
| `quick-xml` | XML parsing | Fast, streaming and deserialization support |
| `indicatif` | Progress bars | Rich terminal UI, thread-safe |
| `thiserror` | Error handling | Zero-cost, derive macros, no runtime overhead |
| `chrono` | Timestamps | RFC3339 formatting, serialization |
| `directories` | Config paths | Cross-platform XDG compliance |
| `criterion` | Benchmarking | Statistical analysis, HTML reports |
| `wiremock` | HTTP mocking | Async, realistic mock server |
