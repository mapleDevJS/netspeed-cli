# Contributing to netspeed-cli

Thank you for your interest in contributing to netspeed-cli! All contributions are welcome.

## Getting Started

### Prerequisites

- Rust 1.86 or later
- `cargo` package manager
- Node.js (for commitlint, optional but recommended)

### Setup

```bash
git clone https://github.com/mapleDevJS/netspeed-cli.git
cd netspeed-cli
cargo build

# Install commitlint for commit message validation (optional but recommended)
npm install -D @commitlint/config-conventional

# Install pre-commit hooks
pip install pre-commit
pre-commit install
```

Note: the repo includes [.cargo/config.toml](/Users/alexey.ivanov/vibe.dev/netspeed-cli/.cargo/config.toml) to set `SDKROOT` on macOS. This avoids sandbox-related `xcrun` SDK cache warnings during local builds and CI.

### Running Tests

```bash
just qa
```

Socket-binding integration tests are run separately:

```bash
just test-socket
```

### Code Style

This project follows standard Rust formatting with clippy pedantic mode enabled:

```bash
cargo fmt          # Format code
cargo clippy       # Run linter
cargo clippy -- -D warnings  # Fail on any warning

# Generate changelog from conventional commits (requires git-cliff)
cargo install git-cliff
just changelog     # Updates CHANGELOG.md
```

All CI checks must pass before a PR can be merged:
- `cargo test` on Ubuntu, macOS, and Windows
- socket-binding integration tests on Linux
- `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --doc`
- `cargo deny check` (security + license audit)
- MSRV verification with Rust 1.86

## Development Workflow

1. **Fork** the repository
2. **Create a branch** from `develop` (or `main` for hotfixes)
3. **Make your changes** with tests
4. **Run verification**: `just qa`
5. **Submit a PR** against `develop` (or `main` for hotfixes)

### Branch Strategy

| Branch | Purpose |
|---|---|
| `main` | Stable releases only |
| `develop` | Integration branch for features and fixes |
| Feature branches | `feature/your-feature-name` (branch from `develop`) |
| Hotfix branches | `hotfix/issue-description` (branch from `main`) |

**PR rules:**
- Feature PRs → target `develop`
- Release PRs → `develop` → `main` (see [Release Process](#release-process))
- Hotfix PRs → target `main` directly

## Release Process

Releases are published from **`main`** via CI automation. See [RELEASE.md](RELEASE.md)
for the complete release workflow.

**Quick summary:**
1. Develop on `develop`
2. Open PR `develop` → `main`
3. Merge PR to `main`
4. Run `./scripts/release.sh <version>` from `main`
5. CI builds binaries, publishes GitHub Release, updates Homebrew, and publishes to crates.io

## What to Contribute

### Bug Fixes

If you find a bug, please:
1. Check existing issues to avoid duplicates
2. Open an issue if none exists
3. Submit a PR with a test that reproduces the bug and the fix

### New Features

For new features:
1. Open an issue first to discuss the approach
2. Wait for maintainer approval
3. Implement with tests and documentation
4. Submit a PR

### Documentation

Documentation improvements are always welcome:
- README clarifications
- Code comments
- Module-level doc comments (use `///` and `//!`)
- Examples and usage guides

#### Version Badges

The README includes live version badges that auto-update on every release:

| Badge | Source | Updates when |
|---|---|---|
| Crates.io | crates.io index | `cargo publish` completes |
| GitHub Release | GitHub releases API | tag is created |
| Homebrew | Homebrew formula repo | formula is updated |

**Do not hardcode version numbers in the README.** The badges pull the current version dynamically. If you see a hardcoded version, it should be replaced with a badge or removed.

## Testing Guidelines

- Write unit tests for pure functions
- Write integration tests for CLI behavior
- Use `wiremock` for HTTP mocking in network tests
- Tests should be fast and deterministic
- Avoid tests that require real network access
- Mark socket-binding tests as ignored unless they are meant for the dedicated socket lane

### Test Locations

- `src/*/mod.rs` — inline unit tests in `#[cfg(test)]` modules
- `tests/integration_test.rs` — CLI integration tests
- `tests/mock_network_test.rs` — socket-binding HTTP mocking tests
- `tests/integration_upload_fetch_test.rs` — socket-binding upload/protocol tests
- `tests/e2e_test.rs` — socket-binding mocked end-to-end tests

## Code Conventions

- Follow [SOLID](https://en.wikipedia.org/wiki/SOLID) principles
- Use `#[must_use]` on pure functions
- Add `# Errors` section to `///` docs for `Result`-returning functions
- Keep modules focused on a single concern
- No dead code — remove unused functions and dependencies
- Error types go in `error.rs`, data types in `types.rs`

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat: add peak speed detection
fix: resolve XML parsing error on some servers
docs: update README installation instructions
test: add config file loading tests
ci: add cargo-audit job
```

**Commit types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

A pre-commit hook validates commit messages. Use `npm install -D @commitlint/config-conventional` to enable local validation.

## Questions?

If you have questions, feel free to:
- Open a [Discussion](https://github.com/mapleDevJS/netspeed-cli/discussions)
- Ask in an existing issue

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
