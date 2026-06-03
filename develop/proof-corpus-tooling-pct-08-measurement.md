# Proof Corpus Tooling PCT-08 Measurement

Source:

- `develop/proof-corpus-tooling-improvement-plan.md`
- `develop/proof-corpus-tooling-improvement-plan-todo.md` milestone PCT-08
- `develop/proof-corpus-tooling-baseline-pct-00.md`

This document records the end-to-end measurement after PCT-01 through PCT-07.
It is timing and workflow evidence only. It is not proof evidence.

## Measurement Context

- Date: 2026-06-04 JST
- Repository commit before this documentation change: `2619d19`
- Working directory: `npa`
- Machine: Apple M1 Max, 10 hardware threads, 64 GiB memory
- OS: macOS 26.5 build 25F71, Darwin 25.5.0 arm64
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Timer: `/usr/bin/time -p`
- Cargo state: warm local target cache; timings include process and test
  harness overhead, but not a clean rebuild of Rust dependencies.

## Before And After Summary

PCT-00 baseline:

| Area | Command | Status | real | Notes |
| --- | --- | --- | ---: | --- |
| Single-module authoring build | `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic` | pass | 0.87s | Rebuilt one module and wrote deterministic metadata. |
| Changed-only source-free verification | `cargo run -p npa-proof-corpus -- --changed-only` | pass | 0.74s | Clean tree selected no proof corpus modules. |
| Full corpus gate | `./scripts/check-corpus.sh` | pass | 1059.81s | Included authoring, package verifier, package CLI examples, index, axiom report, and publish-plan coverage. |

PCT-08 after PCT-01 through PCT-07:

| Area | Command | Status | real | Notes |
| --- | --- | --- | ---: | --- |
| Single-module authoring build | `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic` | pass | 1.39s | Rebuilt one module and its import closure. |
| Source-free selected module, cache off | `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache off` | pass | 0.71s | Live source-free verification. |
| Source-free selected module, authoring cache | `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache authoring` | pass | 0.69s | Reported `cache_status = "hit"` for the selected module. |
| Changed-only, cache off | `cargo run -p npa-proof-corpus -- --changed-only --verified-cache off` | pass | 0.59s | Clean tree selected no proof corpus modules. |
| Changed-only, authoring cache | `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring` | pass | 0.60s | Clean tree selected no proof corpus modules. |
| Fast non-corpus gate | `./scripts/check-fast.sh` | pass | 37.75s | Excludes proof-corpus crate and corpus/package fixture groups. |
| Authoring corpus gate | `./scripts/check-corpus-authoring.sh` | pass | 115.21s | Excludes package-wide CLI examples. |
| Package corpus gate | `./scripts/check-corpus-package.sh` | pass | 1122.39s | Includes package verifier, package CLI examples, axiom report, index, and publish-plan checks. |
| Promotion materialize dry-run | `cargo run -p npa-proof-corpus -- --promote-materialize /tmp/npa-promote-basic-pct08.md --mathlib-root ../npa-mathlib --compat-alias none` | pass | 0.70s | Dry-run only; reported intended source, meta, replay, certificate, and manifest actions. |

For the clean small-module authoring loop, the local loop
`--build-module Proofs.Ai.Basic` plus selected module verification plus
`--changed-only` took 2.69s with cache off and 2.68s with an authoring cache
hit. That is about 394 times faster than the PCT-00 full corpus gate timing.
The split authoring gate itself is about 9.2 times faster than the PCT-00 full
corpus gate.

The package gate remains the bottleneck and should stay out of the theorem
repair hot path. In this run the long package-gate items were:

- `package_cli_examples_pass_on_proof_corpus`: 544.18s
- `package_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact`: 184.82s
- source-free package verifier fixture groups: 65.24s and 69.36s
- `package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy`: 63.60s
- `package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts`: 63.29s
- `package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean`: 64.81s

## Final Verification Matrix

| Check | Command | Status | real | Trust note |
| --- | --- | --- | ---: | --- |
| Fast gate | `./scripts/check-fast.sh` | pass | 37.75s | Non-corpus development gate only. |
| Authoring gate | `./scripts/check-corpus-authoring.sh` | pass | 115.21s | Local proof-corpus completion gate; no package-wide CLI examples. |
| Package gate | `./scripts/check-corpus-package.sh` | pass | 1122.39s | Package verifier and package artifact regression gate. |
| Cache off selected module | `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache off` | pass | 0.71s | Live source-free verification. |
| Cache on selected module | `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache authoring` | pass | 0.69s | Authoring cache hit is a speed hint, not proof evidence. |
| Cache off changed-only | `cargo run -p npa-proof-corpus -- --changed-only --verified-cache off` | pass | 0.59s | Clean tree selected no proof corpus modules. |
| Cache on changed-only | `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring` | pass | 0.60s | Clean tree selected no proof corpus modules. |
| Promotion plan | `cargo run -p npa-proof-corpus -- --promote-plan Proofs.Ai.Basic --mathlib-root ../npa-mathlib --to-module Mathlib.Logic.Basic --out /tmp/npa-promote-basic-pct08.md` | pass | not timed | Read-only plan generation. The plan is not proof evidence. |
| Promotion dry-run | `cargo run -p npa-proof-corpus -- --promote-materialize /tmp/npa-promote-basic-pct08.md --mathlib-root ../npa-mathlib --compat-alias none` | pass | 0.70s | Dry-run did not modify `npa-mathlib`. The dry-run is not proof evidence. |

After the measurement commands, `git diff --name-only -- proofs tools/proof-corpus scripts`
was empty. No proof corpus source, certificate, manifest, package lock, theorem
index, generated package artifact, tool source, or gate script changed as a
result of measurement.

## Stale Command Scan

PCT-08 reviewed the documentation and repo-local skills matched by:

```sh
rg -n "check-corpus-authoring|check-corpus-package|check-corpus-full|verified-cache|promote-plan|promote-materialize" AGENTS.md CONTRIBUTING.md README.md develop skills
```

Cleanup performed:

- The README now points proof-corpus theorem repair at targeted local commands
  and `check-corpus-authoring.sh` before mentioning full gates.
- `develop/proof-corpus-ai-workflow.md` now names
  `check-corpus-authoring.sh` as the theorem batch boundary and documents the
  `--promote-plan` / `--promote-materialize` commands as untrusted promotion
  helpers.
- `skills/prove-theorem/SKILL.md` now uses local module verification and
  `check-corpus-authoring.sh` as the default completion path, reserving
  package/full gates for compatibility, push readiness, release handoff, or an
  explicit user request.
- `skills/closure-audit/SKILL.md` now prefers the PCT-07 materialize command
  and uses sibling repository paths instead of placeholder absolute paths.

## Workflow Conclusion

The default theorem authoring loop is now:

1. Rebuild only the target module or batch with `--build-module`,
   `--build-modules`, or `--build-modules-file`.
2. Verify only the target module and changed modules source-free, optionally
   using `--verified-cache authoring` for local repeated checks.
3. Finish a coherent theorem batch with `./scripts/check-corpus-authoring.sh`.
4. Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
   only at package/push/release/compatibility boundaries.

Cache files, promotion plans, theorem indexes, replay files, metadata, CI
status, and timing logs remain untrusted sidecars. Proof acceptance continues
to depend on canonical certificates, deterministic hashes, and source-free
checker or verifier verdicts.
