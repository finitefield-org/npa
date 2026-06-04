# Proof Corpus Tooling Baseline PCT-00

Source:

- `develop/proof-corpus-tooling-improvement-plan.md`
- `develop/proof-corpus-tooling-improvement-plan-todo.md` milestone PCT-00

This document records the local baseline before implementing batch build, split
corpus gates, verified certificate cache, or promotion commands. It is
measurement evidence only and is not proof evidence.

## Measurement Context

- Date: 2026-06-03 / 2026-06-04 JST
- Repository commit before this documentation change: `8ca8511`
- Working directory: `npa`
- Machine: Apple M1 Max, 10 hardware threads, 64 GiB memory
- OS: macOS 26.5 build 25F71, Darwin 25.5.0 arm64
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Timer: `/usr/bin/time -p`
- Cargo state: warm local target cache; timings include process and test
  harness overhead, but not a clean rebuild of Rust dependencies.

## Baseline Results

| Area | Command | Status | real | user | sys | Notes |
| --- | --- | --- | ---: | ---: | ---: | --- |
| CLI inventory check | `cargo run -p npa-proof-corpus -- --help` | pass | 1.04s | 0.02s | 0.02s | Printed the current usage text listed below. |
| Single-module authoring build | `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic` | pass | 0.87s | 0.27s | 0.03s | Rebuilt one module and wrote the AI theorem index deterministically. |
| Changed-only source-free verification | `cargo run -p npa-proof-corpus -- --changed-only` | pass | 0.74s | 0.02s | 0.03s | Clean tree selected no proof corpus modules. |
| Full corpus gate | `./scripts/check-corpus.sh` | pass | 1059.81s | 1469.99s | 9.27s | Ran once only for this baseline. |

The full corpus gate currently runs four stages:

1. `cargo test -p npa-proof-corpus`
2. `cargo test --workspace --exclude npa-proof-corpus proof_corpus`
3. `cargo test --workspace --exclude npa-proof-corpus proof_package`
4. `cargo test -p npa-api --lib package_verifier`

Notable long-running tests from the full-gate run:

- `ai_certificates_match_manifest_and_verify`: 112.57s
- `manifest_package_audit` test binary: 64.85s
- `package_artifacts::tests::package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy`: 61.35s
- `package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts`: 61.73s
- `package_cli_examples_pass_on_proof_corpus`: 543.79s
- `package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean`: 64.67s
- `package_verifier::tests::package_fast_verifier_verifies_proof_package_source_free` /
  `package_reference_verifier_verifies_proof_package_source_free_in_topological_order`
  proof-package fixture group: 66.84s
- `npa-api --lib package_verifier` corpus verifier group: 69.88s

After the measurement commands, `git diff --name-only -- proofs tools/proof-corpus`
was empty. No source, certificate, manifest, package lock, theorem index, or
proof-corpus tool source changed as a result of measurement.

## Current `npa-proof-corpus` Command Inventory

Implemented in `tools/proof-corpus/src/main.rs`:

- `npa-proof-corpus`
  - Runs the full corpus artifact generation path and writes checked-in proof
    artifacts and package metadata.
- `npa-proof-corpus --package-lock-only`
  - Rewrites `proofs/generated/package-lock.json`.
- `npa-proof-corpus --build-module MODULE`
  - Builds one named corpus module plus its import closure, then rewrites
    `proofs/manifest.toml`, `proofs/npa-package.toml`,
    `proofs/generated/package-lock.json`, and
    `proofs/generated/ai-theorem-index.json`.
- `npa-proof-corpus --verify [--module MODULE ...] [--changed-only] [--shard INDEX/TOTAL] [--failures-out PATH]`
  - Verifies checked-in certificates source-free. With no module or
    `--changed-only`, it targets all proof corpus modules.
- `npa-proof-corpus --module MODULE [--shard INDEX/TOTAL] [--failures-out PATH]`
  - Verifies a selected module source-free. Multiple `--module` options are
    accepted by the shared verify parser.
- `npa-proof-corpus --changed-only [--shard INDEX/TOTAL] [--failures-out PATH]`
  - Uses `git status --porcelain -- proofs` to select changed proof modules.
    A change to package-wide metadata selects all modules.
- `--shard INDEX/TOTAL`
  - Applies zero-based shard selection after target sorting.
- `--failures-out PATH`
  - Writes failure diagnostics as a replay sidecar for selected verification
    failures.
- `npa-proof-corpus --write-ai-index [PATH]`
  - Writes the untrusted AI theorem index, either to the default generated
    path or to an explicit output path.
- `npa-proof-corpus --write-replay MODULE::DECL PATH`
  - Writes a focused replay sidecar for one declaration.

Planned but not implemented as of PCT-00:

- `--build-modules`
- `--build-modules-file`
- `--metadata-once`
- `--verified-cache`
- `--promote-plan`
- `--mathlib-root`
- `--to-module`
- promotion-plan `--out`
- `--promote-materialize`
- promotion dry-run / apply modes

## Measurement Convention For Later Milestones

- Record the exact command, status, `real/user/sys`, date, repo commit, machine
  context, and whether the Cargo target cache was warm.
- Do not introduce fixed wall-clock pass/fail thresholds. Compare before/after
  timings on the same machine and describe remaining bottlenecks explicitly.
- For commands that may write artifacts, check `git diff --name-only -- proofs
  tools/proof-corpus` before committing. The measurement itself must not leave
  source, certificate, manifest, package lock, theorem index, or generated
  package artifact changes.
- Run `./scripts/check-corpus.sh` at most once for a baseline or batch-boundary
  measurement. During authoring, prefer local module verification and
  `--changed-only`.
- Treat AI theorem indexes, replay files, metadata, cache files, timing logs,
  and promotion plans as untrusted sidecars. Only canonical certificates and
  source-free verifier verdicts can support proof acceptance.
