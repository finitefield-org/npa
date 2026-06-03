#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Package-wide proof corpus gate. This keeps package verifier coverage and the
# CLI examples that are too heavy for normal theorem-authoring repair loops.
echo "[1/8] Proof corpus package audit tests"
cargo test -p npa-proof-corpus --test manifest_package_audit

echo "[2/8] Cross-crate proof_package fixture tests"
cargo test --workspace --exclude npa-proof-corpus proof_package

echo "[3/8] Proof corpus package artifact projection fixture"
cargo test -p npa-api package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy

echo "[4/8] Source-free package verifier corpus tests"
cargo test -p npa-api --lib package_verifier

echo "[5/8] Package CLI examples on proof corpus"
cargo test -p npa-cli package_cli_examples_pass_on_proof_corpus

echo "[6/8] Package axiom-report check on proof corpus"
cargo test -p npa-cli package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts

echo "[7/8] Package theorem index check on proof corpus"
cargo test -p npa-cli package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean

echo "[8/8] Package publish-plan check on proof corpus"
cargo test -p npa-cli package_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact
