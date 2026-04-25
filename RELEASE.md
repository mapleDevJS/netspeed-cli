# Release Process

Releases are built and published from the **`main`** branch. All development
happens on **`develop`** and flows to `main` via pull request.

## Workflow

```
develop ──(PR)──► main ──(tag)──► CI publishes
```

### Step-by-step

#### 1. Ensure `develop` is ready for release

```bash
git checkout develop
git pull origin develop
just qa
```

#### 2. Open PR from `develop` → `main`

```bash
gh pr create --base main --head develop \
  --title "Release v<version>" \
  --body "Merge develop into main for v<version> release"
```

Review the PR, ensure all CI checks pass, then **merge to `main`**.

#### 3. Create the release

Check out `main` and run the release script:

```bash
git checkout main
git pull origin main
./scripts/release.sh <version>
```

The script will:
- Validate you're on `main` with a clean tree
- Update `Cargo.toml` version
- Commit with `chore(release): bump to v0.5.0`
- Push to `origin/main`
- Create and push annotated tag `v0.5.0`

#### 4. Monitor CI

The `release.yml` workflow triggers automatically on the tag push and handles:

| Job | What it does |
|---|---|
| `build-binaries` | Cross-compiles for macOS, Linux, Windows (7 targets) |
| `socket-integration` | Runs ignored socket-binding integration tests |
| `publish-github-release` | Creates GitHub Release, uploads binaries + SBOM + checksums |
| `publish-crates-io` | Publishes the crate to crates.io |

Monitor progress:
```bash
gh run list --limit 1
```

#### 5. Verify the release

After CI completes:

```bash
# Check GitHub Release
gh release view v<version> --repo mapleDevJS/netspeed-cli

# Test Homebrew install
brew upgrade mapleDevJS/homebrew-netspeed-cli/netspeed-cli

# Test crates.io install
cargo install netspeed-cli
```

## Version Numbering

Follows [Semantic Versioning](https://semver.org/):

- **Major** (`1.0.0` → `2.0.0`): Breaking changes, incompatible API
- **Minor** (`0.4.0` → `0.5.0`): New features, backward compatible
- **Patch** (`0.4.0` → `0.4.1`): Bug fixes only

## Emergency Hotfix

If a critical bug needs immediate fixing:

```bash
git checkout main
git checkout -b hotfix/critical-fix
# ... make fix ...
git commit -m "fix: critical bug description"
git push origin hotfix/critical-fix
gh pr create --base main --head hotfix/critical-fix
# Merge PR, then run release script with patch version
./scripts/release.sh <patch-version>
```

## What CI Publishes

| Asset | Destination |
|---|---|
| Binaries (7 platforms) | GitHub Release |
| SBOM (SPDX JSON) | GitHub Release |
| SHA256 checksums | GitHub Release |
| Crate | crates.io |
| Homebrew formula | mapleDevJS/homebrew-netspeed-cli tap |
