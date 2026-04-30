# Release Process

Releases are built and published from the **`main`** branch. Development flows
from **`develop`** to `main` by pull request, then the manual GitHub Actions
release workflow creates the release commit and tag.

## Workflow

```text
develop --(PR)--> main --(Release workflow)--> GitHub + crates.io + Homebrew PR
```

## Prerequisites

- `CARGO_REGISTRY_TOKEN` repository secret with crates.io publish access.
- `RELEASE_TOKEN` repository secret with contents access to create release
  commits, tags, and GitHub Releases in `mapleDevJS/netspeed-cli`.
- `HOMEBREW_TAP_TOKEN` repository secret with branch and PR access to
  `mapleDevJS/homebrew-netspeed-cli`.
- `main` contains the changes intended for release.
- The requested version is greater than the latest `vX.Y.Z` tag and is not
  already published on crates.io.

## Release Steps

### 1. Prepare `develop`

```bash
git checkout develop
git pull origin develop
just qa
```

Open and merge the release PR:

```bash
gh pr create --base main --head develop \
  --title "Release v<version>" \
  --body "Merge develop into main for v<version> release."
```

### 2. Start the Release Workflow

Run the canonical release workflow from `main`:

```bash
gh workflow run release.yml --ref main -f version=<version>
gh run list --workflow release.yml --limit 1
```

The legacy local command is intentionally non-mutating:

```bash
./scripts/release.sh <version>
```

It prints the `gh workflow run` command instead of editing files, committing,
tagging, or publishing.

### 3. What the Workflow Does

| Job | Responsibility |
|---|---|
| `release-context` | Validates version, updates versioned files, runs release checks, commits, and tags |
| `build-binaries` | Builds Linux, macOS, and Windows release binaries |
| `publish-github-release` | Creates the GitHub Release, checksums, SBOM, and uploads assets |
| `publish-crates-io` | Verifies and publishes the crate to crates.io |
| `homebrew-tap-pr` | Opens a PR in `mapleDevJS/homebrew-netspeed-cli` with the updated formula |

### 4. Merge the Homebrew Tap PR

After the release workflow succeeds, review and merge the generated tap PR.
This is the step that makes `brew upgrade netspeed-cli` pick up the new version.

### 5. Verify Channel Sync

```bash
scripts/check-release-sync.sh
```

Expected result: `Cargo.toml`, GitHub Releases, crates.io, the in-repo formula,
and the tap formula all report the same version.

## Release Readiness

The `Release Readiness` workflow runs on PRs to `main`, on demand, and weekly.
It checks:

- Rust formatting, clippy, unit/doc/socket tests, docs, package, and cargo-deny.
- Generated completions and man page are committed.
- `cargo publish --dry-run --locked` succeeds.
- The Homebrew formula can be rendered from the current package version.
- Scheduled runs detect drift between GitHub, crates.io, and Homebrew.

## Emergency Hotfix

```bash
git checkout main
git pull origin main
git checkout -b hotfix/critical-fix
# make the fix
git commit -m "fix: describe critical bug"
git push origin hotfix/critical-fix
gh pr create --base main --head hotfix/critical-fix
```

After the hotfix PR merges, run the release workflow with the next patch version.

## Version Numbering

This project follows Semantic Versioning:

- **Major**: breaking changes.
- **Minor**: backward-compatible features.
- **Patch**: backward-compatible fixes, docs/package/release corrections.
