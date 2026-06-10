#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

source scripts/package-gate-plan-report.sh

npa_package_gate_plan_report "./scripts/check-corpus-full.sh"
npa_package_gate_plan_apply_selection "./scripts/check-corpus-full.sh"

# Full corpus gate for npa-mathlib promotion readiness, release handoff,
# package-tooling changes, and high-trust-adjacent checks. Normal proof-corpus
# authoring should use check-corpus-authoring.sh instead.
# This is the gate that requires full proof-corpus package CLI examples. The
# package gate runs the smoke tier and projection/publish check-mode tests; this
# final tier keeps build-certs --check and verify-certs examples covered.
echo "[1/3] Proof corpus authoring gate"
NPA_PACKAGE_GATE_PLAN_SELECT=0 bash scripts/check-corpus-authoring.sh

echo "[2/3] Proof corpus package gate"
NPA_PACKAGE_GATE_PLAN_SELECT=0 NPA_PACKAGE_GATE_SHARED_SNAPSHOT=0 bash scripts/check-corpus-package.sh

echo "[3/3] Package CLI full proof-corpus examples"
cargo test -p npa-cli package_cli_full_corpus_examples_pass_on_proof_corpus
