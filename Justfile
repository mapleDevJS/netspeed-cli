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
	cargo test --lib socket_regression -- --ignored
	cargo test --test mock_network_test -- --ignored --nocapture
	cargo test --test integration_upload_fetch_test -- --ignored --nocapture
	cargo test --test e2e_test -- --ignored --nocapture

# Run linter checks (formatting + clippy) — mirrors CI exactly
lint:
	cargo fmt --all -- --check
	cargo clippy --locked --all-targets --all-features -- -D warnings

# Auto-fix formatting and clippy suggestions
fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

# Build release binary
build:
	cargo build --release

# Full release verification
release:
	cargo fmt --all -- --check
	cargo clippy --locked --all-targets --all-features -- -D warnings
	cargo test --verbose
	cargo test --doc
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
	cargo package --locked
	cargo build --release

# Native local gate; platform matrix and GitHub services still require CI.
qa:
	NETSPEED_ALLOW_DIRTY=1 bash scripts/ci-checks.sh qa

# Includes publication dry run and CI coverage threshold (no publishing).
qa-full:
	NETSPEED_ALLOW_DIRTY=1 bash scripts/ci-checks.sh full

# Complete native and Linux checks; used by the installed pre-push hook.
ci-local:
	bash scripts/check-local-ci-tools.sh
	just qa-full
	just act-test
	just act-quality

# Compare main and the candidate on this machine without GitHub API calls.
bench-compare reference="main":
	bash scripts/bench-compare.sh {{quote(reference)}}

# Validate Actions expressions and embedded shell (install ShellCheck too).
workflow-lint:
	bash scripts/ci-checks.sh workflows

# Execute only the Ubuntu test job. No release workflows or credentials.
act-test:
	bash scripts/act-local.sh workflow_dispatch -W .github/workflows/ci.yml -j test --matrix os:ubuntu-latest -P ubuntu-latest=catthehacker/ubuntu:act-latest

# Execute the Ubuntu quality job (requires Docker and network).
act-quality:
	bash scripts/act-local.sh workflow_dispatch -W .github/workflows/ci.yml -j quality --env NETSPEED_ALLOW_DIRTY=1 -P ubuntu-latest=catthehacker/ubuntu:act-latest

# Run security audit
audit:
	cargo audit

# Run cargo-deny security and license check
audit-deny:
	cargo deny check

# Run full security audit (audit + deny)
security:
	cargo audit
	cargo deny check

# Check for outdated dependencies
outdated:
	cargo install cargo-outdated --locked 2>/dev/null || true
	cargo outdated --root

# Run security-focused clippy checks
security-check:
	cargo clippy --locked --all-targets --all-features -- -D warnings

# Check for unsafe code usage
unsafe-check:
	@echo "=== Checking for unsafe code ==="
	@if grep -r 'unsafe' src/ --include='*.rs' | grep -v 'allow(unsafe_code)' | grep -v '#\[allow(unsafe_code)\]' > /dev/null; then \
		echo "Found unsafe code - review recommended"; \
		grep -rn 'unsafe' src/ --include='*.rs' | grep -v 'allow(unsafe_code)' | grep -v '#\[allow(unsafe_code)\]'; \
	else \
		echo "No unsafe code found (or all properly documented)"; \
	fi

# Scan for potential hardcoded secrets
secret-scan:
	@echo "=== Scanning for potential hardcoded secrets ==="
	@grep -rnE '(password|api_key|secret|token)\s*=\s*["\']' src/ --include='*.rs' | grep -v '//' | grep -v 'example' | grep -v 'placeholder' || echo "No obvious secrets found"

# Check file permissions handling
permissions-check:
	@echo "=== Checking file permissions ==="
	@if grep -q 'set_permissions.*0o600' src/; then \
		echo "Found 0o600 permission setting - good!"; \
	else \
		echo "Warning: No 0o600 permission setting found"; \
	fi

# Generate security report (runs audit + deny + clippy)
audit-report:
	just audit
	just audit-deny
	@echo "\n=== Security Audit Summary ==="
	@echo "All security checks completed"

# Generate and open documentation
doc:
	cargo doc --no-deps --open

# Run tests and generate coverage report (requires cargo-llvm-cov)
coverage:
	cargo llvm-cov --open

# Run tests and generate LCOV coverage file
coverage-lcov:
	bash scripts/ci-checks.sh coverage

# Generate shell completions and man page
generate-docs:
	cargo build
	@echo \"Completions and man page generated via build.rs\"

# Verify generated shell completions and man page are committed
check-generated-docs:
	scripts/check-generated-docs.sh

# Render the Homebrew formula for a released version
render-homebrew-formula version:
	scripts/render-homebrew-formula.sh {{version}}

# Check GitHub/crates.io/Homebrew release channel sync
release-sync:
	scripts/check-release-sync.sh

# Check MSRV (Minimum Supported Rust Version)
msrv:
	bash scripts/ci-checks.sh msrv

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

# Save a named Criterion baseline locally
bench-baseline:
	cargo bench --locked --bench core_benchmarks -- --save-baseline main --noplot

# Fail on significant regressions against a local named baseline
bench-regression:
	cargo bench --locked --bench core_benchmarks -- --noplot
	python3 scripts/check-benchmarks.py target/criterion target/criterion --baseline-name main --ignore-prefix url_ --minimum-absolute-delta-ns 25

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

# Install the full local replacement for routine hosted CI.
install-hooks:
	mkdir -p .githooks
	cp scripts/pre-push.sh .githooks/pre-push
	chmod +x .githooks/pre-push
	git config core.hooksPath .githooks
	@echo "Pre-push hook installed: checks a clean candidate with just ci-local."
