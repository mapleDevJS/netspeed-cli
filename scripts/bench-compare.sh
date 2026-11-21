#!/usr/bin/env bash
# Measure an immutable local reference and candidate serially on the same host.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
reference_sha="$(git rev-parse --verify "${1:-main}^{commit}")"
reference_dir="$(mktemp -d)"
trap 'rm -rf "$reference_dir"' EXIT
git archive "$reference_sha" | tar -x -C "$reference_dir"
target_root="$PWD/target/local-ci-bench"
# Keep compiled dependencies, but never compare stale estimates from an older run.
rm -rf "$target_root/reference/criterion" "$target_root/candidate/criterion"
echo "Benchmark reference: $reference_sha"
CARGO_TARGET_DIR="$target_root/reference" cargo bench --locked --manifest-path "$reference_dir/Cargo.toml" --bench core_benchmarks -- --noplot
CARGO_TARGET_DIR="$target_root/candidate" cargo bench --locked --bench core_benchmarks -- --noplot
python3 scripts/check-benchmarks.py "$target_root/reference/criterion" "$target_root/candidate/criterion" --ignore-prefix url_ --minimum-absolute-delta-ns 25
