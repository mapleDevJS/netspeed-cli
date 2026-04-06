#!/usr/bin/env bash
#
# release.sh — Prepare and trigger a release for netspeed-cli
#
# Usage:
#   ./scripts/release.sh <version>
#
# Example:
#   ./scripts/release.sh 0.5.0
#
# Workflow:
#   1. Validate you're on `main` with a clean tree
#   2. Update Cargo.toml version
#   3. Commit with conventional commit message
#   4. Create annotated git tag
#   5. Push commit + tag to origin
#   6. CI (release.yml) builds binaries, creates GitHub Release,
#      updates Homebrew formula, and publishes to crates.io
#
# Prerequisites:
#   - You must be on the `main` branch
#   - Working tree must be clean
#   - git, cargo, and gh CLI must be installed and authenticated
#
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC}  $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# ── Validate arguments ──────────────────────────────────────────────
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.5.0"
    exit 1
fi

VERSION="$1"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "Invalid version format: '$VERSION' (expected semver, e.g., 0.5.0)"
    exit 1
fi

# ── Pre-flight checks ───────────────────────────────────────────────
check_prerequisites() {
    log_info "Running pre-flight checks..."

    for cmd in git cargo gh; do
        if ! command -v "$cmd" &>/dev/null; then
            log_error "'$cmd' is not installed"
            exit 1
        fi
    done

    if [[ ! -f "${PROJECT_DIR}/Cargo.toml" ]]; then
        log_error "Cargo.toml not found in ${PROJECT_DIR}"
        exit 1
    fi

    if ! gh auth status &>/dev/null; then
        log_error "GitHub CLI is not authenticated. Run 'gh auth login' first."
        exit 1
    fi

    log_ok "All prerequisites satisfied"
}

# ── Branch validation ───────────────────────────────────────────────
check_branch() {
    cd "$PROJECT_DIR"

    local branch
    branch="$(git branch --show-current)"

    if [[ "$branch" != "main" ]]; then
        log_error "You must be on the 'main' branch to create a release (currently on '$branch')"
        log_error "Merge your changes from 'develop' to 'main' first:"
        log_error "  gh pr create --base main --head develop --title 'Release v${VERSION}'"
        exit 1
    fi

    # Check for uncommitted changes
    if [[ -n "$(git status --porcelain)" ]]; then
        log_error "Working tree is not clean. Commit or stash changes first."
        git status --short
        exit 1
    fi

    log_ok "On 'main' branch with clean working tree"
}

# ── Version bump ─────────────────────────────────────────────────────
bump_version() {
    cd "$PROJECT_DIR"

    local current_version
    current_version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)

    if [[ "$current_version" == "$VERSION" ]]; then
        log_error "Cargo.toml is already at version $VERSION"
        exit 1
    fi

    log_info "Bumping version: $current_version → $VERSION"

    # Update Cargo.toml
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    fi

    # Verify
    local actual_version
    actual_version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    if [[ "$actual_version" != "$VERSION" ]]; then
        log_error "Failed to update Cargo.toml (got: '$actual_version')"
        exit 1
    fi

    log_ok "Cargo.toml updated to $VERSION"
}

# ── Commit ───────────────────────────────────────────────────────────
commit_changes() {
    cd "$PROJECT_DIR"

    git add Cargo.toml

    # Regenerate build artifacts (completions, man page)
    cargo build --quiet

    git add -A

    git commit -m "chore(release): bump to v${VERSION}"

    log_ok "Committed: chore(release): bump to v${VERSION}"
}

# ── Push ─────────────────────────────────────────────────────────────
push_to_origin() {
    cd "$PROJECT_DIR"

    log_info "Pushing to origin/main..."
    git push origin main

    log_ok "Pushed to origin/main"
}

# ── Tag ──────────────────────────────────────────────────────────────
create_tag() {
    cd "$PROJECT_DIR"

    # Clean up existing tag if present
    git tag -d "v${VERSION}" 2>/dev/null || true
    git push origin ":refs/tags/v${VERSION}" 2>/dev/null || true

    log_info "Creating annotated tag v${VERSION}..."
    git tag -a "v${VERSION}" -m "Release v${VERSION}"
    git push origin "v${VERSION}"

    log_ok "Tag v${VERSION} created and pushed"
}

# ── Summary ──────────────────────────────────────────────────────────
print_summary() {
    echo ""
    echo -e "${GREEN}============================================${NC}"
    echo -e "${GREEN}  Release v${VERSION} triggered!${NC}"
    echo -e "${GREEN}============================================${NC}"
    echo ""
    echo "  CI is now building and publishing:"
    echo "    • Multi-platform binaries (macOS, Linux, Windows)"
    echo "    • GitHub Release with auto-generated notes"
    echo "    • Homebrew formula update"
    echo "    • crates.io publication"
    echo ""
    echo "  Monitor progress:"
    echo "    gh run list --limit 1"
    echo ""
    echo "  Release page (after CI):"
    echo "    https://github.com/${REPO_OWNER:-mapleDevJS}/netspeed-cli/releases/tag/v${VERSION}"
    echo ""
}

# ── Main ─────────────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}  netspeed-cli Release Pipeline${NC}"
    echo -e "${BLUE}  Version: v${VERSION}${NC}"
    echo -e "${BLUE}============================================${NC}"
    echo ""

    check_prerequisites
    check_branch
    bump_version
    commit_changes
    push_to_origin
    create_tag
    print_summary
}

main "$@"
