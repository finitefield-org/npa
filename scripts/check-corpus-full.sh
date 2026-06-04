#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Full corpus gate for npa-mathlib promotion readiness, release handoff,
# package-tooling changes, and high-trust-adjacent checks. Normal proof-corpus
# authoring should use check-corpus-authoring.sh instead.
echo "[1/2] Proof corpus authoring gate"
bash scripts/check-corpus-authoring.sh

echo "[2/2] Proof corpus package gate"
bash scripts/check-corpus-package.sh
