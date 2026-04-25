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

# Run the socket-binding integration suite explicitly
test-socket:
	cargo test --test mock_network_test -- --ignored --nocapture
	cargo test --test integration_upload_fetch_test -- --ignored --nocapture
	cargo test --test e2e_test -- --ignored --nocapture

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
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --verbose
	cargo test --doc
	cargo build --release

# CI-quality gate used locally and in automation
qa:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --verbose
	cargo test --doc

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
	@echo \"Completions and man page generated via build.rs\"

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

# Run benchmarks
bench:
	cargo bench

# Compile benchmark targets without executing them (CI-safe smoke check)
bench-check:
	cargo bench --no-run

# Run benchmarks with Criterion reports
bench-report:
	cargo bench

# Run benchmarks with regression detection (compare against baseline)
bench-regression:
	@echo \"Running benchmarks and checking for regressions...\"
	@if [ ! -f benches/baseline.txt ]; then echo \"No baseline found. Run 'just bench-report' first to create one.\"; exit 1; fi
	cargo bench 2>&1 | tee /tmp/bench_output.txt
	(cargo benchcmp benches/baseline.txt /tmp/bench_output.txt 2>&1 || { echo \"Benchmark regression detected!\"; exit 1; })
	@echo \"All benchmarks passed comparison.\"

# Generate changelog from conventional commits (requires git-cliff)
changelog:
	@if ! command -v git-cliff &>/dev/null; then echo \"git-cliff not found. Install: cargo install git-cliff\"; exit 1; fi
	git-cliff --config .cliff.toml --changelog CHANGELOG.md

# Preview changelog without writing to file
changelog-preview:
	@if ! command -v git-cliff &>/dev/null; then echo \"git-cliff not found. Install: cargo install git-cliff\"; exit 1; fi
	git-cliff --config .cliff.toml

# Generate changelog for a specific tag
changelog-tag tag:
	@if ! command -v git-cliff &>/dev/null; then echo \"git-cliff not found. Install: cargo install git-cliff\"; exit 1; fi
	git-cliff --config .cliff.toml --tag {{tag}}
