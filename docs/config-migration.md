# Configuration Migration Guide

This document helps users migrate their `config.toml` when netspeed-cli introduces breaking changes.

## v0.8.0 → v0.9.0

### Removed Options

The following options were deprecated in v0.8.x and have been **removed** in v0.9.0:

| Old Option | Replacement |
|------------|-------------|
| `simple = true` | Use `--format simple` or `--format compact` |
| `csv = true` | Use `--format csv` |
| `json = true` | Use `--format json` |

### New Options

The following new options are available in v0.9.0:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `strict` | boolean | `false` | Enable strict config mode - show warnings for invalid values |
| `profile` | string | `power-user` | User profile: `gamer`, `streamer`, `remote-worker`, `power-user`, `casual` |
| `theme` | string | `dark` | Output theme: `dark`, `light`, `high-contrast`, `monochrome` |
| `custom_user_agent` | string | (browser UA) | Custom HTTP user agent string |

### Example Migration

**Before (v0.7.x):**
```toml
no_download = false
no_upload = false
simple = false
csv = false
json = false
timeout = 10
```

**After (v0.9.0):**
```toml
no_download = false
no_upload = false
timeout = 10

# New in v0.9.0
profile = \"power-user\"
theme = \"dark\"
```

### Format Changes

The `--format` flag now supersedes individual `--json`, `--csv`, and `--simple` flags:

| Old Flag | New Flag |
|----------|----------|
| `--json` | `--format json` |
| `--csv` | `--format csv` |
| `--simple` | `--format simple` |

The old flags are still accepted but deprecated and will be removed in a future version.

## v0.7.0 → v0.8.0

No breaking config changes in this release.

## General Configuration

### Config File Location

- **Linux/macOS**: `~/.config/netspeed-cli/config.toml`
- **Windows**: `%APPDATA%\netspeed-cli\/config.toml`

### Configuration Precedence

CLI arguments always override config file values, which override built-in defaults:

```
CLI args (highest priority)
    ↓
Config file
    ↓
Built-in defaults (lowest priority)
```

### Validation

Run `--dry-run` to validate your configuration without executing a test:

```bash
netspeed-cli --dry-run
```

This will show which server would be selected and confirm connectivity.
