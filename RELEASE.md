# Release Process

Releases are built and published from the **`master`** branch. The project uses a
three-branch model:

| Branch | Purpose |
|---|---|
| `master` | Stable releases only — every merge is a tagged release |
| `develop` | Integration branch for features, fixes, and refactors |
| `staging` | Release preparation — promoted to `master` for releases |

## Workflow

```
feature → develop → staging ──(PR)──► master ──(tag)──► CI publishes
```

### Step-by-step

#### 1. Prepare `staging` for release

```bash
git checkout staging
git pull origin staging
cargo test && cargo clippy -- -D warnings
```

#### 2. Open PR from `staging` → `master`

```bash
gh pr create --base master --head staging \
  --title "Release v0.8.0" \
  --body "Promote staging to master for v0.8.0 release"
```

Review the PR, ensure all CI checks pass, then **merge to `master`**.

#### 3. Create the release tag

Check out `master` and tag the release:

```bash
git checkout master
git pull origin master
git tag -a v0.8.0 -m "Release v0.8.0"
git push origin v0.8.0
```

Pushing the tag triggers the release CI workflow.

#### 4. Reset `staging` to match `develop`

After the release, reset staging so it's ready for the next cycle:

```bash
git checkout staging
git reset --hard origin/develop
git push origin staging --force-with-lease
```

#### 5. Monitor CI

The `release.yml` workflow triggers automatically on the tag push and handles:

| Job | What it does |
|---|---|
| `build-binaries` | Cross-compiles for macOS, Linux, Windows (7 targets) |
| `publish-github-release` | Creates GitHub Release, uploads binaries + SBOM + checksums |
| `publish-crates-io` | Publishes the crate to crates.io |

Monitor progress:
```bash
gh run list --limit 1
```

#### 6. Verify the release

After CI completes:

```bash
# Check GitHub Release
gh release view v0.8.0 --repo mapleDevJS/netspeed-cli

# Test Homebrew install
brew upgrade mapledevjs/netspeed-cli/netspeed-cli

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
git checkout master
git checkout -b hotfix/critical-fix
# ... make fix ...
git add . && git commit -m "fix: critical bug description"
git push origin hotfix/critical-fix
gh pr create --base master --head hotfix/critical-fix
# Merge PR, then tag the patch
git tag -a v0.4.1 -m "Release v0.4.1"
git push origin v0.4.1
```

## What CI Publishes

| Asset | Destination |
|---|---|
| Binaries (7 platforms) | GitHub Release |
| SBOM (SPDX JSON) | GitHub Release |
| SHA256 checksums | GitHub Release |
| Crate | crates.io |
| Homebrew formula | mapledevjs/homebrew-netspeed-cli tap |
