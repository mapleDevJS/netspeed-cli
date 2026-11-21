#!/usr/bin/env bash
# Fail before expensive checks when the local CI prerequisites are unavailable.
set -euo pipefail
for tool in just cargo rustup actionlint shellcheck python3 docker act; do
  if ! command -v "$tool" >/dev/null; then
    echo "Missing $tool; see CONTRIBUTING.md for local CI setup." >&2
    exit 1
  fi
done
python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 11) else "Python 3.11+ is required for the release-script tests (tomllib).")'
cargo deny --version >/dev/null
cargo llvm-cov --version >/dev/null
rustup run 1.86 cargo --version >/dev/null
llvm_root="$(rustup run stable rustc --print sysroot)"
host="$(rustup run stable rustc -vV | sed -n 's/^host: //p')"
if [[ ! -x "$llvm_root/lib/rustlib/$host/bin/llvm-profdata" ]]; then
  echo 'Install coverage tools: rustup component add llvm-tools-preview --toolchain stable' >&2
  exit 1
fi
if ! docker info >/dev/null 2>&1; then
  echo 'Start Docker (for example, colima start) before running just ci-local.' >&2
  exit 1
fi
