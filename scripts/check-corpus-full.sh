#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Full corpus gate for push readiness, release handoff, and high-trust-adjacent
# changes. It composes the authoring gate and the package-wide gate.
echo "[1/2] Proof corpus authoring gate"
bash scripts/check-corpus-authoring.sh

echo "[2/2] Proof corpus package gate"
bash scripts/check-corpus-package.sh
