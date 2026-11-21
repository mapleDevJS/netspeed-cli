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

Note: the repo includes `.cargo/config.toml` to set `SDKROOT` on macOS. This avoids sandbox-related `xcrun` SDK cache warnings during local builds and CI.

### Running Tests

```bash
just qa
```

Socket-binding integration tests are run separately:

```bash
just test-socket
```

### Code Style

This project follows standard Rust formatting with clippy lints enabled:

```bash
cargo fmt          # Format code
cargo clippy       # Run linter
cargo clippy -- -D warnings  # Fail on any warning

# Generate changelog from conventional commits (requires git-cliff)
cargo install git-cliff
just changelog     # Updates CHANGELOG.md
```

Local checks and CI use `scripts/ci-checks.sh` to avoid command drift. Install
Python 3.11+, `just`, `actionlint`, `shellcheck`, and `cargo-deny`, plus Rust 1.86:

```bash
brew install python just actionlint shellcheck
cargo install cargo-deny cargo-llvm-cov --locked
rustup toolchain install 1.86 --profile minimal
rustup component add llvm-tools-preview --toolchain stable
# Install and start a Docker engine, plus act, for Ubuntu checks.
brew install act colima docker
colima start
just install-hooks
just ci-local
```

`just qa` runs workflow lint, formatting, clippy, documentation, package and
generated-doc verification, release-script tests, dependency/license checks,
regular and socket tests, a release build, and MSRV verification. It checks the
native host; it does not establish Windows or Linux compatibility on macOS.
Local packaging includes uncommitted changes for review; CI packaging requires a
clean checkout. Hooks are local and bypassable.

`just qa-full` adds a publication **dry run**, coverage with CI's 83% line
threshold, and a benchmark comparison against the local `main` commit. Install
`cargo-llvm-cov` and `llvm-tools-preview` first. `just bench-compare <ref>` measures
an immutable reference and the candidate serially on the same machine; no GitHub
API is needed. Keep local `main` current. Regressions fail the gate.

`just ci-local` runs `qa-full` followed by the Ubuntu test and quality jobs through
Docker. The installed pre-push hook runs this entire gate, requires a clean
candidate, rejects pushes of a different checked-out commit, and checks that the
candidate did not change during validation. This is the local replacement for
routine hosted CI. Install it in every clone with `just install-hooks`.

For workflow debugging with Docker running:

```bash
just workflow-lint
just act-test
just act-quality
```

The `act` commands select only Ubuntu jobs in `ci.yml`; they do not run publishing,
issue creation, or Codecov upload. Initial execution downloads runner images and
actions. The wrapper respects `DOCKER_HOST` or the selected Docker context.
Linked worktrees expose their shared Git metadata read-only inside the runner;
local runner jobs trust the copied checkout for Git's ownership checks.
The newer action cache handles pinned commits, and local quality
packaging includes uncommitted changes. The daemon socket inside the runner is `/var/run/docker.sock`, which
also works with Colima's VM socket layout. They use the container's native architecture; add
`--container-architecture linux/amd64` to the underlying `act` command when
checking x86 Linux on Apple Silicon. Containers approximate GitHub runners;
local success does not replace exact-commit GitHub release validation.

Routine pushes and PRs trigger **zero workflows from repository YAML files**. All
non-release workflows are manual-only; there are no Actions schedules. Dependabot
updates still arrive, but automatic merge enablement is now manual so it cannot
assume hosted checks ran. Run `just release-sync` locally when checking released
channel drift; there is no background drift monitor.

Release publishing retains its manual preparation and tag-triggered publication,
including successful exact-commit validation. `ci.yml` is available for explicit
manual platform checks and `act`; it never runs automatically. The local gate
covers the native host plus Linux; Windows requires a Windows machine or an
explicit manual matrix run. Local success does not establish Windows acceptance.

Hooks remain bypassable and require the developer machine to be available.
Default-setup CodeQL is still enabled in GitHub settings and can trigger
automatic security runs separately. Disabling that security control requires
explicit owner authorization. The prepared `Manual CodeQL` workflow requires
default setup to be disabled first. Local clippy, workflow lint, and dependency
checks do not provide equivalent CodeQL analysis.
Repository rules are also separate. Remove required CI contexts from rules for branches
that now use local validation; do not leave merges waiting for manual checks.
Full local checks need network access for tools, dependencies, and advisories.

## Development Workflow

1. **Fork** the repository
2. **Create a branch** from `develop` (or `main` for hotfixes)
3. **Make your changes** with tests
4. **Run verification**: `just ci-local`
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
4. Run `gh workflow run release.yml --ref main -f version=<version>`
5. Run `just ci-local`, then review and merge the version PR
6. Run the same workflow command again; it validates the exact `main` commit,
   tags it, and starts publication
7. Review and merge the generated formula PRs

`./scripts/release.sh <version>` is a non-mutating compatibility wrapper that
prints the canonical workflow command. Hosted validation and maintenance are
manual-only outside tag publication. Dependency updates are managed by Dependabot.

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

Changelog PR generation is available on demand from `develop`:

```bash
gh workflow run changelog.yml --ref develop
```

Dependency update PRs are managed by Dependabot. Run the Lock File Check
workflow manually when you need an additional dependency freshness report.

## Questions?

If you have questions, feel free to:
- Open a [Discussion](https://github.com/mapleDevJS/netspeed-cli/discussions)
- Ask in an existing issue

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

### Benchmark regression checks

Use `just bench-baseline` to save a local Criterion baseline, then `just bench-regression` after making changes. The comparison fails when the current mean's lower confidence bound exceeds the baseline's upper bound by more than 10%; missing benchmarks also fail.

CI selects the latest successful manual CI run on `main` and measures that commit on the current runner. If no reference exists, CI reports a bootstrap run. Benchmark comparisons use the same script locally and in CI.

The release workflow renders the formula for the released tag and runs Homebrew installation and audit after the GitHub release exists. The tap PR depends on that audit passing.

CI coverage includes the default suite and the local socket suites, including production orchestration, before enforcing the 83% line threshold. Coverage collection uses `--no-report` for each suite and a single `cargo llvm-cov report` to merge the results.

Release tags must match the package version (`vX.Y.Z`) and point to a commit on `main`. `scripts/validate-release.py` checks identity before release QA; binary builds and publishing depend on that gate. Test release validation locally with `python3 -m unittest discover -s scripts -p 'test_*.py'`. CLI integration tests execute Cargo’s built binary and use dry-run for configuration checks, without contacting public speedtest services.
