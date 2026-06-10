#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

source scripts/package-gate-plan-report.sh

npa_package_gate_plan_report "./scripts/check-corpus-package.sh"
npa_package_gate_plan_apply_selection "./scripts/check-corpus-package.sh"

shared_snapshot="${NPA_PACKAGE_GATE_SHARED_SNAPSHOT:-1}"
case "${shared_snapshot}" in
  0|1) ;;
  *)
    echo "error: NPA_PACKAGE_GATE_SHARED_SNAPSHOT must be 0 or 1" >&2
    exit 2
    ;;
esac

if [[ "${shared_snapshot}" == "1" ]]; then
  total_steps=6
else
  total_steps=9
fi

# Package-wide proof corpus gate for npa-mathlib promotion readiness,
# package-tooling changes, release handoff, and high-trust-adjacent checks.
# This intentionally stays out of normal theorem-authoring repair loops.
# Keep build-certs check cache disabled in package gates. PAS-09 read-through
# cache entries are local counters only, not proof evidence or build evidence.
# CLI tier policy:
# - This package gate runs package_cli_smoke on small fixtures for help, argument,
#   JSON, and check-mode coverage.
# - Full proof-corpus CLI build/verify examples are required by check-corpus-full.sh
#   and remain runnable with: cargo test -p npa-cli package_cli_full_corpus
# - Projection and publish-plan proof-corpus checks stay here because package-tooling
#   changes must keep generated artifact check-mode behavior covered.
echo "[1/${total_steps}] Proof corpus package audit tests"
cargo test -p npa-proof-corpus --test manifest_package_audit

echo "[2/${total_steps}] Cross-crate proof_package fixture tests"
cargo test --workspace --exclude npa-proof-corpus proof_package

echo "[3/${total_steps}] Proof corpus package artifact projection fixture"
cargo test -p npa-api package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy

echo "[4/${total_steps}] Source-free package verifier corpus tests"
# Some package_verifier tests intentionally exercise process and disk cache
# behavior under the shared target/npa-package-audit-cache root.
cargo test -p npa-api --lib package_verifier -- --test-threads=1

echo "[5/${total_steps}] Package CLI smoke examples on small fixtures"
cargo test -p npa-cli package_cli_smoke

if [[ "${shared_snapshot}" == "1" ]]; then
  echo "[6/${total_steps}] Unified generated package checks on proof corpus"
  echo "Using package check-generated for local package gate checks (NPA_PACKAGE_GATE_SHARED_SNAPSHOT=1)."
  echo "Set NPA_PACKAGE_GATE_SHARED_SNAPSHOT=0 to run the standalone proof-corpus artifact checks."
  cargo run -p npa-cli -- package check-generated --root proofs --timings summary --json
else
  echo "Using standalone generated artifact checks (NPA_PACKAGE_GATE_SHARED_SNAPSHOT=0)."

  echo "[6/${total_steps}] Package axiom-report check on proof corpus"
  cargo test -p npa-cli package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts

  echo "[7/${total_steps}] Package theorem index check on proof corpus"
  cargo test -p npa-cli package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean

  echo "[8/${total_steps}] Package export-summary check on proof corpus"
  cargo test -p npa-cli package_export_summary_proof_corpus_check_mode_succeeds_with_checked_in_artifact

  echo "[9/${total_steps}] Package publish-plan check on proof corpus"
  cargo test -p npa-cli package_cli_full_corpus_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact
fi
