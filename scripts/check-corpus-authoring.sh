#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

source scripts/package-gate-plan-report.sh

npa_package_gate_plan_report "./scripts/check-corpus-authoring.sh"
npa_package_gate_plan_apply_selection "./scripts/check-corpus-authoring.sh"

# Lightweight authoring gate for normal proof-corpus theorem work.
#
# The in-repo proof corpus is a staging workspace, not the public theorem
# package. Keep package-wide verifier, CLI example, axiom-report, theorem-index,
# and publish-plan checks out of the authoring hot path; run
# check-corpus-package.sh before npa-mathlib promotion or package/release work.
echo "[1/1] Source-free changed proof corpus modules"
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
