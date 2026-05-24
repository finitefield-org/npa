#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Fixed post-Phase-9 and post-Phase-3-Human gate: AI automation remains on
# Machine Surface fixtures, while Human Surface regressions run in workspace tests.
echo "[1/4] Phase 9 M9 regression fixtures"
cargo test -p npa-api --lib advanced_ai_m9

echo "[2/4] Formatting check"
cargo fmt --all -- --check

echo "[3/4] Clippy workspace gate"
cargo clippy --workspace --all-targets -- -D warnings

echo "[4/4] Workspace tests"
cargo test --workspace
