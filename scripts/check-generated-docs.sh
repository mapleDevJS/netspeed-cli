#!/usr/bin/env bash
#
# Verify generated shell completions and the man page are committed.
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${PROJECT_DIR}"

cargo build --quiet

if ! git diff --quiet -- completions netspeed-cli.1; then
    echo "error: generated completions or man page are stale" >&2
    echo "Run 'cargo build' and commit the generated files." >&2
    git diff --stat -- completions netspeed-cli.1 >&2
    exit 1
fi

echo "generated docs are current"
