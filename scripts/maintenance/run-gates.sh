#!/bin/sh
# run-gates.sh — required local validation wrapper.
# Does not tag, push, or call the maintainer network scripts.
set -eu
cd "$(dirname "$0")/../.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace

os=$(uname -s)
if [ "$os" = "Darwin" ]; then
  scripts/no-egress/run-macos.sh
fi

workflow_changed=0
if git diff --name-only | grep -q '^\.github/workflows/.*\.yml$'; then
  workflow_changed=1
fi
if git rev-parse --verify origin/main >/dev/null 2>&1; then
  if git diff --name-only origin/main...HEAD | grep -q '^\.github/workflows/.*\.yml$'; then
    workflow_changed=1
  fi
fi

if [ "$workflow_changed" -eq 1 ]; then
  if ! command -v actionlint >/dev/null 2>&1; then
    echo "FAIL: workflow files changed and actionlint is not on PATH" >&2
    exit 1
  fi
  actionlint
fi
