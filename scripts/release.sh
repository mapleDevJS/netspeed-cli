#!/usr/bin/env bash
#
# Compatibility wrapper for the GitHub Actions release workflow.
#
# The canonical release path is the manual "Release" workflow. Keeping this
# wrapper prevents local release scripts from mutating main, creating tags, or
# publishing channels with behavior that differs from CI.
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>" >&2
    echo "Example: $0 0.11.0" >&2
    exit 2
fi

VERSION="${1#v}"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: invalid version '${1}' (expected X.Y.Z or vX.Y.Z)" >&2
    exit 2
fi

cat <<EOF
Local releases are disabled.

Use the canonical GitHub Actions release workflow:

  gh workflow run release.yml --ref main -f version=${VERSION}
  gh run list --workflow release.yml --limit 1

The workflow validates the version, creates the release commit and tag,
publishes GitHub/crates.io assets, and opens the Homebrew tap PR.
EOF
