# ADR-001: Config Bool Merge Semantics

**Status:** Accepted (Known Limitation)  
**Date:** 2026-04-09  
**Context:** Architecture audit revealed a design limitation in config file + CLI flag merging

## Problem

The configuration system merges CLI arguments with a TOML config file using OR semantics:

```rust
let merge_bool = |cli: bool, file: Option<bool>| cli || file.unwrap_or(false);
```

This means a config file setting like `no_download = true` cannot be overridden by *not* passing `--no-download` on the CLI, because clap's derive macro defaults boolean flags to `false`. There is no way to distinguish "user didn't pass the flag" from "user explicitly set it to false."

**Practical effect:** Once a boolean is set in the config file, it acts as a persistent default that can only be "enabled" further (OR), never disabled by absence.

## Why It Can't Be Fixed Without Breaking Changes

### The Core Issue

Clap's derive macro for `#[arg(long)]` on `bool` fields produces:
- Flag not passed → `false`
- Flag passed → `true`

There is no third "not specified" state. To get three-state behavior (`None`/`Some(true)`/`Some(false)`), the CLI interface would need to change to something like:
- `--no-download` / `--no-download=true` / `--no-download=false`

This would break backward compatibility with all existing scripts and documentation using the current `--no-download` flag syntax.

### Workarounds Considered

1. **`Option<bool>` with `num_args = 0..=1`** — Requires users to pass `--no-download=true` or `--no-download=false`, breaking the existing flag-based interface
2. **Separate enable/disable flags** — `--no-download` and `--download` would conflict with clap's derive and create confusing help text
3. **`ArgAction::SetTrue` with tracking** — Still produces `bool`, not `Option<bool>`; doesn't solve the three-state problem

## Decision

**Keep OR-merge semantics as-is.** The current behavior is documented and has an intuitive interpretation: the config file acts as a persistent default. Users can remove or comment out lines in the config file to disable persistent settings.

This is acceptable because:
1. The config file is an *opt-in* feature — users who don't want persistent defaults simply don't create one
2. The limitation is clearly documented in `config.rs`
3. Changing the CLI interface would break existing usage patterns
4. The workaround (edit config file) is straightforward

## Consequences

- **Positive:** No breaking CLI changes, simple implementation, predictable behavior
- **Negative:** Users with `no_download = true` in config must edit the file (not just pass a CLI flag) to re-enable downloads
- **Mitigation:** This ADR documents the limitation so future contributors don't waste time investigating it

## Related

- `src/config.rs` — `merge_bool` function and inline documentation
- Architecture audit 2026-04-09, Score 6/10 for Configuration Management
