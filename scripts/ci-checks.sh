#!/usr/bin/env bash
# Shared checks for local execution and GitHub Actions. Never publishes.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
package_args=(--locked)
# Local gates include the working tree; CI packaging retains its clean-tree check.
if [[ "${NETSPEED_ALLOW_DIRTY:-0}" == 1 ]]; then
  package_args+=(--allow-dirty)
fi
case "${1:-qa}" in
  test) cargo test --locked --verbose ;;
  socket)
    cargo test --locked --lib socket_regression -- --ignored
    cargo test --locked --test mock_network_test --test integration_upload_fetch_test --test e2e_test -- --ignored --nocapture
    ;;
  quality)
    cargo fmt --all -- --check
    cargo clippy --locked --all-targets --all-features -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --workspace
    cargo package "${package_args[@]}"
    bash scripts/check-generated-docs.sh
    python3 -m unittest discover -s scripts -p 'test_*.py'
    cargo deny check
    ;;
  workflows)
    actionlint
    shellcheck scripts/ci-checks.sh scripts/act-local.sh scripts/pre-push.sh scripts/bench-compare.sh scripts/check-local-ci-tools.sh
    ;;
  msrv)
    RUSTC="$(rustup which --toolchain 1.86 rustc)" rustup run 1.86 cargo check --locked
    ;;
  build) cargo build --locked --release ;;
  coverage)
    # Cargo can find Homebrew rustc on PATH even when invoked through rustup.
    # Bind the compiler and LLVM tools to the same toolchain/profile format.
    RUSTC="$(rustup which --toolchain stable rustc)"
    RUSTDOC="$(rustup which --toolchain stable rustdoc)"
    export RUSTC RUSTDOC
    rustup run stable cargo llvm-cov clean --workspace
    rustup run stable cargo llvm-cov --locked --all-features --no-report
    rustup run stable cargo llvm-cov --locked --all-features --no-report --lib -- socket_regression --ignored
    rustup run stable cargo llvm-cov --locked --all-features --no-report --test mock_network_test --test integration_upload_fetch_test --test e2e_test -- --ignored
    rustup run stable cargo llvm-cov report --lcov --output-path lcov.info --fail-under-lines=83
    ;;
  qa)
    for check in workflows quality test socket build msrv; do
      bash scripts/ci-checks.sh "$check"
    done
    cargo test --locked --doc
    ;;
  publish) cargo publish --dry-run "${package_args[@]}" ;;
  full)
    bash scripts/ci-checks.sh qa
    bash scripts/ci-checks.sh publish
    bash scripts/ci-checks.sh coverage
    bash scripts/bench-compare.sh
    ;;
  *) echo "Usage: $0 {qa|full|test|socket|quality|workflows|msrv|build|coverage|publish}" >&2; exit 2 ;;
esac
