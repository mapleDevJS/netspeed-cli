# Architecture and Design Decisions

## Architecture Overview

The binary parses CLI arguments, builds configuration, and hands control to an
`Orchestrator`. Rust applications can use the supported `api::run_speed_test`
entry point to receive a report without CLI output or history persistence.

```text
main.rs / cli.rs / api.rs / config/
  inputs, supported entry points, and configuration
             |
orchestrator.rs / phase_runner.rs / phase_registry.rs
  lifecycle coordination and ordered phase execution
             |
phases.rs / services.rs / task_runner.rs
  phase state, injected network services, transfer lifecycle
       /                                      \
download.rs / upload.rs                servers.rs / endpoints.rs
bandwidth measurement                   discovery and protocol URLs
       \                                      /
formatter/ / result_processor.rs / output.rs / storage.rs
  report enrichment, presentation, and optional local history
             |
http.rs / progress.rs / terminal.rs / error.rs
  network, terminal, and cross-cutting support
```

## Workflow and Boundaries

`PhaseRunner` executes registered lifecycle functions in order. `PhaseContext`
carries discovered server metadata and measurement results between phases.
`task_runner::run_bandwidth_test` shares transfer setup, latency-under-load
monitoring, cancellation, and aggregation for download and upload.

`services.rs` isolates server discovery, IP discovery, and latency monitoring
behind traits. `endpoints.rs` derives canonical latency, upload, and download
URLs from server feed entries. The `api` facade is the recommended embedding
interface; the lower-level modules remain public for advanced use and may
change between minor releases.

## Server Selection and History

Production discovery uses `ServerSelector::select_reachable`. The default
service probes at most 20 distinct server URLs with five concurrent requests,
a two-second budget per candidate, and a shared four-second deadline. Two
successful, bounded latency responses are required per candidate. Valid client
coordinates narrow the shortlist geographically; otherwise candidates are
sampled across the feed. Failed probes are skipped, and an entirely failed
shortlist produces a network error.

Unknown distances use an internal infinity sentinel and serialize as JSON
`null` or an empty CSV field. Human-readable output displays `unknown`.
Metadata requests check HTTP status and reject invalid client coordinates.

History schema version 1 preserves complete reports alongside legacy summary
fields. Existing summary-only entries remain readable. Unsupported schemas
fail without overwriting the file. `FileStorage::with_path` applies to every
save, load, report write, and clear operation. Writers and clearing share a
stable sibling lock file; saves use a unique temporary file and atomic
replacement. Clearing removes primary history, backup, and corrupt recovery
copies while retaining the lock file.

## Configuration and Output

The CLI merges configuration in this order:

```text
CLI flags > config file > built-in defaults
```

Mergeable boolean CLI flags use tri-state parsing to distinguish omitted
options from explicitly enabled or disabled values. The formatter modules
render structured results into human-readable and machine-readable formats;
`output.rs` dispatches output, while `result_processor.rs` enriches results.
JSON and JSONL failures use a machine-readable error envelope.

## Concurrency and Measurement

- Tokio runs asynchronous network work; the HTTP client uses HTTP/1 and
  disables gzip so transfer measurements count uncompressed network bytes.
- Downloads stream response chunks instead of buffering entire bodies.
- Four concurrent streams are used by default; `--single` selects one stream.
- Atomic counters track bytes and mutex-protected samples track interval rates.
- A background task samples latency during bandwidth phases and stops when
  transfer work completes or is cancelled.

Warm-up and byte caps, upload payload sizing, sampling limits, and the
interpretation of short or capped runs are documented in the README's
Measurement limits section.

## Testing and Quality Gates

- Unit tests cover pure functions, serialization, and measurement algorithms.
- Integration tests use `wiremock`; socket-binding suites run in a dedicated
  CI lane.
- Criterion benchmarks cover bandwidth calculations, distance, formatting,
  and validation. Regression comparisons run for main-targeting pull requests
  and manual dispatches.
- CI checks formatting, Clippy, tests, docs, package contents, generated docs,
  dependency policy, and the Rust 1.86 minimum supported version. Coverage
  enforcement runs on main-targeting pull requests and manual dispatches.

## Module Responsibilities

| Module | Responsibility |
|---|---|
| `api.rs` | Supported quiet library entry point returning a test report |
| `cli.rs`, `config/` | CLI parsing and three-tier configuration merge |
| `orchestrator.rs` | Workflow setup, dependencies, output and storage policy |
| `phase_runner.rs`, `phase_registry.rs` | Ordered phase registration and execution |
| `phases.rs` | Phase context and lifecycle implementations |
| `services.rs` | Network service interfaces and default implementations |
| `task_runner.rs` | Shared download/upload measurement lifecycle |
| `download.rs`, `upload.rs` | Transfer-specific bandwidth operations |
| `servers.rs`, `endpoints.rs` | Server discovery and canonical endpoint derivation |
| `formatter/`, `result_processor.rs`, `output.rs` | Result enrichment and output |
| `storage.rs`, `history.rs` | Storage interfaces and file-backed history |
| `http.rs`, `http_client.rs` | HTTP client settings and abstraction |
| `types.rs`, `error.rs` | Shared data types and error model |
| `progress.rs`, `terminal.rs`, `theme.rs` | Terminal feedback and presentation support |

## Key Dependencies

| Dependency | Purpose |
|---|---|
| `clap` | CLI parsing, shell completions, and help generation |
| `reqwest`, `rustls` | Async HTTP and TLS |
| `tokio` | Async runtime |
| `serde`, `serde_json`, `csv`, `quick-xml`, `toml` | Data formats and configuration |
| `jiff` | Date and time support |
| `indicatif`, `owo-colors`, `crossterm` | Terminal progress and styling |
| `thiserror` | Typed error definitions |
| `wiremock`, `criterion`, `proptest` | Network, performance, and property testing |
