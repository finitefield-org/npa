#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Default development gate for non-corpus changes. Keep the proof corpus out of
# the hot path; run the split corpus gates only when the corpus gate conditions
# in AGENTS.md apply.
echo "[1/3] Formatting check"
cargo fmt --all -- --check

echo "[2/3] Clippy workspace gate without proof corpus"
cargo clippy --workspace --exclude npa-proof-corpus --all-targets -- -D warnings

echo "[3/3] Workspace tests without proof corpus"
cargo test --workspace --exclude npa-proof-corpus -- \
  --skip proof_corpus \
  --skip proof_package \
  --skip package_fast_verifier_ \
  --skip package_reference_verifier_ \
  --skip package_phase8_ \
  --skip package_source_free_
