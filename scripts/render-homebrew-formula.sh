#!/usr/bin/env bash
#
# Render the Homebrew formula for a released netspeed-cli version.
#
# Usage:
#   scripts/render-homebrew-formula.sh <version> [output-path]
#
# The version may be passed with or without a leading "v". The script uses the
# GitHub release source tarball as the formula URL and computes the matching
# SHA256 from that immutable tag archive.
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-mapleDevJS/netspeed-cli}"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMULA_SOURCE="${PROJECT_DIR}/netspeed-cli.rb"

usage() {
    echo "Usage: $0 <version> [output-path]" >&2
    exit 2
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
    usage
fi

VERSION="${1#v}"
OUTPUT_PATH="${2:-${FORMULA_SOURCE}}"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: invalid version '${1}' (expected X.Y.Z or vX.Y.Z)" >&2
    exit 2
fi

if [[ ! -f "${FORMULA_SOURCE}" ]]; then
    echo "error: formula source not found: ${FORMULA_SOURCE}" >&2
    exit 1
fi

TAG="v${VERSION}"
URL="https://github.com/${REPO}/archive/refs/tags/${TAG}.tar.gz"

if command -v sha256sum >/dev/null 2>&1; then
    SHA256="$(curl -fsSL "${URL}" | sha256sum | awk '{print $1}')"
else
    SHA256="$(curl -fsSL "${URL}" | shasum -a 256 | awk '{print $1}')"
fi

if [[ -z "${SHA256}" ]]; then
    echo "error: failed to compute SHA256 for ${URL}" >&2
    exit 1
fi

TMP_FILE="$(mktemp)"
awk -v version="${VERSION}" '
    /^[[:space:]]*version "/ {
        if (!printed_version) {
            print "  version \"" version "\""
            printed_version = 1
        }
        next
    }
    {
        print
    }
    /^[[:space:]]*url "/ && !printed_version {
        print "  version \"" version "\""
        printed_version = 1
    }
' "${FORMULA_SOURCE}" > "${TMP_FILE}"

sed "s|^[[:space:]]*url \".*\"|  url \"${URL}\"|" "${TMP_FILE}" > "${TMP_FILE}.url"
sed "s|^[[:space:]]*sha256 \".*\"|  sha256 \"${SHA256}\"|" "${TMP_FILE}.url" > "${TMP_FILE}.final"

if ! grep -q "version \"${VERSION}\"" "${TMP_FILE}.final"; then
    echo "error: rendered formula is missing version ${VERSION}" >&2
    exit 1
fi

if ! grep -q "url \"${URL}\"" "${TMP_FILE}.final"; then
    echo "error: rendered formula is missing URL ${URL}" >&2
    exit 1
fi

if ! grep -q "sha256 \"${SHA256}\"" "${TMP_FILE}.final"; then
    echo "error: rendered formula is missing SHA256 ${SHA256}" >&2
    exit 1
fi

if [[ "${OUTPUT_PATH}" == "-" ]]; then
    cat "${TMP_FILE}.final"
    rm -f "${TMP_FILE}.final"
else
    mkdir -p "$(dirname "${OUTPUT_PATH}")"
    mv "${TMP_FILE}.final" "${OUTPUT_PATH}"
fi

rm -f "${TMP_FILE}" "${TMP_FILE}.url"
