#!/usr/bin/env bash
#
# release.sh - Automate the netspeed-cli release pipeline
#
# Usage:
#   ./scripts/release.sh <version>
#
# Example:
#   ./scripts/release.sh 0.1.3
#
# This script will:
#   1. Update Cargo.toml version
#   2. Commit and push changes
#   3. Create and push git tag
#   4. Create GitHub release
#   5. Calculate SHA256 of release tarball
#   6. Update Homebrew formula in the tap repository
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="mapleDevJS"
REPO_NAME="netspeed-cli"
TAP_REPO_NAME="homebrew-netspeed-cli"
TAP_REPO_DIR="/tmp/${TAP_REPO_NAME}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

log_info()  { echo -e "${BLUE}[INFO]${NC}  $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validate arguments
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.3"
    exit 1
fi

VERSION="$1"

# Validate version format (e.g., 0.1.3)
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "Invalid version format: $VERSION (expected semver, e.g., 0.1.3)"
    exit 1
fi

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing=0
    for cmd in git gh cargo; do
        if ! command -v "$cmd" &>/dev/null; then
            log_error "$cmd is not installed"
            missing=1
        fi
    done
    
    if [[ $missing -eq 1 ]]; then
        exit 1
    fi
    
    # Verify we're in the right directory
    if [[ ! -f "${PROJECT_DIR}/Cargo.toml" ]]; then
        log_error "Cargo.toml not found in ${PROJECT_DIR}"
        exit 1
    fi
    
    # Verify GitHub CLI is authenticated
    if ! gh auth status &>/dev/null; then
        log_error "GitHub CLI is not authenticated. Run 'gh auth login' first."
        exit 1
    fi
    
    log_ok "All prerequisites satisfied"
}

# Step 1: Update Cargo.toml version
update_cargo_version() {
    log_info "Updating Cargo.toml version to $VERSION..."
    
    cd "$PROJECT_DIR"
    
    # Update version in Cargo.toml
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    fi
    
    # Verify the change
    local actual_version
    actual_version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    if [[ "$actual_version" != "$VERSION" ]]; then
        log_error "Failed to update Cargo.toml version (got: $actual_version)"
        exit 1
    fi
    
    log_ok "Cargo.toml version updated to $VERSION"
}

# Step 2: Commit and push
commit_and_push() {
    log_info "Committing changes..."
    
    cd "$PROJECT_DIR"
    
    git add -A
    git commit -m "release: v${VERSION}"
    
    log_ok "Committed changes"
    
    log_info "Pushing to GitHub..."
    git push origin master
    
    log_ok "Pushed to GitHub"
}

# Step 3: Create and push tag
create_tag() {
    log_info "Creating tag v${VERSION}..."
    
    cd "$PROJECT_DIR"
    
    # Delete local and remote tag if it exists
    git tag -d "v${VERSION}" 2>/dev/null || true
    git push origin ":refs/tags/v${VERSION}" 2>/dev/null || true
    
    git tag "v${VERSION}"
    git push origin "v${VERSION}"
    
    log_ok "Tag v${VERSION} created and pushed"
}

# Step 4: Create GitHub release
create_release() {
    log_info "Creating GitHub release v${VERSION}..."
    
    # Delete existing release if it exists
    gh release delete "v${VERSION}" --yes --cleanup-tag 2>/dev/null || true
    
    # Wait a moment for GitHub to process the tag deletion
    sleep 2
    
    gh release create "v${VERSION}" \
        --title "v${VERSION}" \
        --generate-notes
    
    log_ok "GitHub release v${VERSION} created"
}

# Step 5: Calculate SHA256
calculate_sha256() {
    log_info "Calculating SHA256 for release tarball..."
    
    # Wait for GitHub to generate the tarball
    sleep 3
    
    local sha256
    sha256=$(curl -sL "https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/tags/v${VERSION}.tar.gz" | shasum -a 256 | awk '{print $1}')
    
    if [[ -z "$sha256" ]]; then
        log_error "Failed to calculate SHA256"
        exit 1
    fi
    
    # Verify the tarball is accessible
    local http_code
    http_code=$(curl -sIL -o /dev/null -w "%{http_code}" "https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/tags/v${VERSION}.tar.gz")
    
    if [[ "$http_code" != "200" ]]; then
        log_error "Release tarball not accessible (HTTP $http_code)"
        exit 1
    fi
    
    export SHA256="$sha256"
    log_ok "SHA256: $sha256"
}

# Step 6: Update Homebrew formula
update_homebrew_formula() {
    log_info "Updating Homebrew formula..."
    
    cd "$PROJECT_DIR"
    
    # Update the local formula file
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s|url \"https://github.com/.*/archive/.*|url \"https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/tags/v${VERSION}.tar.gz\"|" netspeed-cli.rb
        sed -i '' "s/sha256 \".*\"/sha256 \"${SHA256}\"/" netspeed-cli.rb
        sed -i '' "s/version \".*\"/version \"${VERSION}\"/" netspeed-cli.rb
    else
        sed -i "s|url \"https://github.com/.*/archive/.*|url \"https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/refs/tags/v${VERSION}.tar.gz\"|" netspeed-cli.rb
        sed -i "s/sha256 \".*\"/sha256 \"${SHA256}\"/" netspeed-cli.rb
        sed -i "s/version \".*\"/version \"${VERSION}\"/" netspeed-cli.rb
    fi
    
    log_ok "Local formula updated"
    
    # Clone or update tap repository
    if [[ -d "$TAP_REPO_DIR" ]]; then
        log_info "Updating tap repository..."
        cd "$TAP_REPO_DIR"
        git pull origin main
    else
        log_info "Cloning tap repository..."
        git clone "git@github.com:${REPO_OWNER}/${TAP_REPO_NAME}.git" "$TAP_REPO_DIR"
        cd "$TAP_REPO_DIR"
    fi
    
    # Copy updated formula
    cp "${PROJECT_DIR}/netspeed-cli.rb" .
    
    # Commit and push
    git add netspeed-cli.rb
    git commit -m "Update to v${VERSION}"
    git push origin main
    
    log_ok "Homebrew formula updated in tap repository"
}

# Summary
print_summary() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Release v${VERSION} completed!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "  GitHub Release: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/tag/v${VERSION}"
    echo "  Tap Repository: https://github.com/${REPO_OWNER}/${TAP_REPO_NAME}"
    echo ""
    echo "  Users can now install/upgrade with:"
    echo "    brew upgrade ${REPO_OWNER}/${REPO_NAME}/${REPO_NAME}"
    echo ""
    echo "  Or fresh install:"
    echo "    brew tap ${REPO_OWNER}/${REPO_NAME}"
    echo "    brew install ${REPO_NAME}"
    echo ""
}

# Main execution
main() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  netspeed-cli Release Pipeline${NC}"
    echo -e "${BLUE}  Version: v${VERSION}${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    check_prerequisites
    update_cargo_version
    commit_and_push
    create_tag
    create_release
    calculate_sha256
    update_homebrew_formula
    print_summary
}

main "$@"
