# Phase 08 — LEDGER: Data Engineering Assessment

**Mode**: 3B — Full Refactoring  
**Date**: 2026-04-06  
**Cycle**: 2 (incremental)

---

## Data Layer Assessment

### Local Storage Inventory

| File | Format | Location | Size | Purpose | Schema Version |
|------|--------|----------|------|---------|----------------|
| Config | TOML | platform-specific config dir | <1KB | User preferences | None (flat key-value) |
| History | JSON | platform-specific data dir | ~50KB max | Test results (100 entries) | Implicit via `HistoryEntry` struct |
| Shell completions | Shell scripts | `completions/` dir | ~5KB each | Generated at build time | clap schema |
| Man page | roff | project root | ~3KB | Generated at build time | clap schema |

### Config Layer (`config.rs`)

**Current schema** (`ConfigFile` struct):
```rust
no_download: Option<bool>
no_upload: Option<bool>
single: Option<bool>
bytes: Option<bool>
simple: Option<bool>
csv: Option<bool>
csv_delimiter: Option<char>
csv_header: Option<bool>
json: Option<bool>
timeout: Option<u64>
```

**Merge logic**: `cli || file.unwrap_or(false)` for bools; `cli` for values unless at default.

**Findings**:
- **CFG-001 [MEDIUM]**: Merge semantics mean "file config acts as a default that activates even when CLI flag is absent." User with `no_download = true` in config cannot disable it without removing the config line — `--no-download` defaults to `false` in clap, so `false || true = true`.
- **No unknown field rejection**: `toml::from_str` silently ignores unknown fields (good for forward compatibility, but no warning).
- **No validation**: Config values are parsed but not validated (e.g., `timeout` could be 0 if file is hand-edited, though the CLI validates it).

**Fixes planned for Sprint 1 (09A)**:
1. Document merge semantics clearly
2. Add validation for config file values
3. Rename merge closure for clarity

### History Layer (`history.rs`)

**Current schema** (`HistoryEntry` struct):
```rust
timestamp: String
server_name: String
sponsor: String
ping: Option<f64>
jitter: Option<f64>
packet_loss: Option<f64>
download: Option<f64>
download_peak: Option<f64>
upload: Option<f64>
upload_peak: Option<f64>
latency_download: Option<f64>
latency_upload: Option<f64>
client_ip: Option<String>
```

**Storage**: Platform-specific data directory (`directories` crate), `history.json`, max 100 entries (ring buffer via `remove(0)`).

**Findings**:
- **VAULT-SEC-002 [MEDIUM]**: No file permissions set — history contains IP addresses stored with default umask.
- **No schema versioning**: If fields are added/removed, old history files may fail to parse (though `unwrap_or_default()` provides graceful degradation).
- **No migration path**: No `version` field in history entries.

**Fixes planned for Sprint 1 (09A)**:
1. Apply `0o600` permissions on file creation (Unix-specific)

### Data Flow Diagram

```
User ──→ CLI args (clap) ──┐
                             ├──→ Config::from_args() ──→ Config struct
Config file (TOML) ─────────┘                             │
                                                          │
Server XML ──→ quick_xml::de ──→ Vec<Server> ──→ select_best_server
                                                          │
HTTP requests ──→ reqwest ──→ bytes ──→ bandwidth calc ──→ TestResult
                                                          │
                                              ┌───────────┴───────────┐
                                              │                       │
                                         History (JSON)         Output (stdout)
                                         save_result()          format_*()
```

### No Data Engineering Changes Required

The data layer is simple and well-designed. Sprint 1 fixes are additive (validation + permissions), not structural. No data migrations needed — the history file has no schema version yet, but the `unwrap_or_default()` pattern provides forward compatibility.

### Downstream Requirements

- **FORGE (09A)**: Add config file validation. Add history file permissions. No schema changes.
- **FORGE (09B)**: No data layer changes planned.
