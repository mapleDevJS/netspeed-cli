#!/usr/bin/env bash
#
# Check whether GitHub releases, crates.io, and Homebrew formulas agree.
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="${GITHUB_REPOSITORY:-mapleDevJS/netspeed-cli}"
TAP_REPO="${HOMEBREW_TAP_REPO:-mapleDevJS/homebrew-netspeed-cli}"
FORMULA_PATH="${PROJECT_DIR}/netspeed-cli.rb"

version_from_formula() {
    local file="$1"
    local version

    version="$(sed -n 's/^[[:space:]]*version "\([^"]*\)".*/\1/p' "${file}" | head -1)"
    if [[ -n "${version}" ]]; then
        echo "${version}"
        return 0
    fi

    sed -n 's|^[[:space:]]*url ".*/refs/tags/v\([^"]*\)\.tar\.gz".*|\1|p' "${file}" | head -1
}

local_version="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "${PROJECT_DIR}/Cargo.toml" | head -1)"
github_version="$(gh release view --repo "${REPO}" --json tagName --jq '.tagName' | sed 's/^v//')"
crates_version="$(cargo search netspeed-cli --limit 1 | sed -n 's/^netspeed-cli = "\([^"]*\)".*/\1/p' | head -1)"
local_formula_version="$(version_from_formula "${FORMULA_PATH}")"

tap_formula_url="https://raw.githubusercontent.com/${TAP_REPO}/main/netspeed-cli.rb"
tap_formula_file="$(mktemp)"
curl -fsSL "${tap_formula_url}" -o "${tap_formula_file}"
tap_formula_version="$(version_from_formula "${tap_formula_file}")"
rm -f "${tap_formula_file}"

status=0
expected="${github_version}"

check_match() {
    local label="$1"
    local actual="$2"

    if [[ -z "${actual}" ]]; then
        echo "error: ${label} version could not be determined" >&2
        status=1
    elif [[ "${actual}" != "${expected}" ]]; then
        echo "error: ${label} is ${actual}, expected ${expected}" >&2
        status=1
    else
        echo "ok: ${label} is ${actual}"
    fi
}

if [[ -z "${expected}" ]]; then
    echo "error: GitHub release version could not be determined" >&2
    exit 1
fi

echo "expected release version: ${expected}"
check_match "Cargo.toml" "${local_version}"
check_match "crates.io" "${crates_version}"
check_match "local Homebrew formula" "${local_formula_version}"
check_match "tap Homebrew formula" "${tap_formula_version}"

exit "${status}"
