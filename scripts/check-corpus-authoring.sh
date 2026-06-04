#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Authoring gate for normal proof-corpus theorem work. This keeps package-wide
# CLI examples out of the hot path; run check-corpus-package.sh for those.
echo "[1/5] Proof corpus CLI/unit tests"
cargo test -p npa-proof-corpus --bin npa-proof-corpus

echo "[2/5] Proof corpus certificate artifact verification"
cargo test -p npa-proof-corpus --test ai_proof_artifacts

echo "[3/5] Lightweight proof corpus package metadata audit"
cargo test -p npa-proof-corpus --test manifest_package_audit -- \
  --skip package_fast_source_free_verifies_checked_in_package_lock \
  --skip package_reference_source_free_verifies_checked_in_package_lock \
  --skip package_source_free_temp_copy_without_source_replay_or_meta_verifies_certificates

echo "[4/5] Cross-crate proof corpus checker smoke"
cargo test -p npa-checker-ref proof_corpus_eq_reasoning_uses_checked_std_logic_eq_builtin_bridge

echo "[5/5] Source-free changed proof corpus modules"
cargo run -p npa-proof-corpus -- --changed-only
