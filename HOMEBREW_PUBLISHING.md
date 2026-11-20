# Homebrew Publishing

`netspeed-cli` is distributed through the tap
`mapleDevJS/homebrew-netspeed-cli`. The main repository owns the formula source
and release automation opens reviewed PRs against the tap.

## Automated Tap Updates

After the GitHub Release is created, automation opens a PR for the in-repository
formula. The tap PR is opened only after crates.io publishing succeeds and the
released formula passes its Homebrew audit. Both PRs require review and merge.

For the tap PR, automation:

1. Computes the SHA256 for the GitHub source tarball
   `https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/vX.Y.Z.tar.gz`.
2. Renders `netspeed-cli.rb` with the new `url` and `sha256`. Homebrew infers
   the version from the tagged source archive URL; an explicit `version`
   declaration is redundant and fails strict formula audit.
3. Pushes branch `release/netspeed-cli-vX.Y.Z` to the tap repository.
4. Opens a PR titled `netspeed-cli vX.Y.Z`.

Required secret:

```text
HOMEBREW_TAP_TOKEN
```

The token needs permission to create branches and pull requests in
`mapleDevJS/homebrew-netspeed-cli`.

## Manual Formula Rendering

To render the formula locally:

```bash
scripts/render-homebrew-formula.sh <version>
```

To render to another path without changing the tracked formula:

```bash
scripts/render-homebrew-formula.sh <version> /tmp/netspeed-cli.rb
```

The version may be passed as `0.10.2` or `v0.10.2`.

## Local Formula Checks

```bash
brew style ./netspeed-cli.rb
brew audit --strict --online ./netspeed-cli.rb
brew install --build-from-source ./netspeed-cli.rb
netspeed-cli --version
```

## Drift Detection

Run:

```bash
scripts/check-release-sync.sh
```

This compares:

- Latest GitHub Release.
- Latest crates.io version.
- Local `Cargo.toml`.
- Local `netspeed-cli.rb`.
- Tap `netspeed-cli.rb`.

The weekly `Release Readiness` schedule checks channel sync and opens an issue
if those channels drift. The workflow's full QA gate runs only on manual
dispatch; pull requests use the regular CI workflow.

## Recovery

If the tap PR was not created:

```bash
VERSION=<version>
scripts/render-homebrew-formula.sh "$VERSION" /tmp/netspeed-cli.rb
git clone git@github.com:mapleDevJS/homebrew-netspeed-cli.git /tmp/homebrew-netspeed-cli
cp /tmp/netspeed-cli.rb /tmp/homebrew-netspeed-cli/Formula/netspeed-cli.rb
cd /tmp/homebrew-netspeed-cli
git checkout -b "release/netspeed-cli-v${VERSION}"
git add Formula/netspeed-cli.rb
git commit -m "netspeed-cli v${VERSION}"
git push origin "release/netspeed-cli-v${VERSION}"
gh pr create --base main --head "release/netspeed-cli-v${VERSION}" \
  --title "netspeed-cli v${VERSION}" \
  --body "Update netspeed-cli formula to v${VERSION}."
```

If Homebrew reports a SHA mismatch, rerender the formula for the exact tag and
open a corrected tap PR. Do not edit the SHA by hand.
