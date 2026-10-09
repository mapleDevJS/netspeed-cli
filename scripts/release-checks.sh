#!/usr/bin/env bash
# Checks run before opening a release PR or tagging its approved main commit.
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --doc
cargo test --locked --lib socket_regression -- --ignored
cargo test --locked --test mock_network_test --test integration_upload_fetch_test --test e2e_test -- --ignored
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --workspace
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo deny check
cargo package --locked --allow-dirty
cargo publish --dry-run --locked --allow-dirty
