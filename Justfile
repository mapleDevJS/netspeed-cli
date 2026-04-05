# Justfile for netspeed-cli
# Usage: just <recipe>
# See: https://github.com/casey/just

# Default recipe
default:
    @just --list

# Run all tests
test:
    cargo test --verbose

# Run all tests (release mode, slower but more realistic)
test-release:
    cargo test --release --verbose

# Run linter checks (formatting + clippy)
lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

# Auto-fix formatting and clippy suggestions
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged

# Build release binary
build:
    cargo build --release

# Full release verification
release:
    cargo test --verbose
    cargo build --release

# Run security audit
audit:
    cargo audit

# Generate and open documentation
doc:
    cargo doc --no-deps --open

# Run tests and generate coverage report (requires cargo-llvm-cov)
coverage:
    cargo llvm-cov --open

# Run tests and generate LCOV coverage file
coverage-lcov:
    cargo llvm-cov --lcov --output-path lcov.info

# Generate shell completions and man page
generate-docs:
    cargo build
    @echo "Completions and man page generated via build.rs"

# Check MSRV (Minimum Supported Rust Version)
msrv:
    cargo check

# Clean build artifacts
clean:
    cargo clean

# Install from source
install:
    cargo install --path .

# Uninstall
uninstall:
    cargo uninstall netspeed-cli

# Run the CLI
run *args:
    cargo run -- {{args}}
