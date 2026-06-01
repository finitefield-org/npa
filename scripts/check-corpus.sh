#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Opt-in proof corpus gate. Use this for proof corpus changes and for changes
# that can affect source-free certificate/package verification.
echo "[1/4] Proof corpus crate tests"
cargo test -p npa-proof-corpus

echo "[2/4] Cross-crate proof_corpus fixture tests"
cargo test --workspace --exclude npa-proof-corpus proof_corpus

echo "[3/4] Cross-crate proof_package fixture tests"
cargo test --workspace --exclude npa-proof-corpus proof_package

echo "[4/4] Source-free package verifier corpus tests"
cargo test -p npa-api --lib package_verifier
