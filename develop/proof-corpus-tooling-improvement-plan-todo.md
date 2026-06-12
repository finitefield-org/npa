# Proof Corpus Tooling Improvement Plan Todo

Source: `develop/proof-corpus-tooling-improvement-plan.md`

This task breakdown fixes the implementation order for shortening proof
corpus authoring iteration time while preserving NPA's certificate-first trust
boundary.

## Scope

In scope:

- Authoring CLI improvements for `tools/proof-corpus`.
- Splitting proof corpus gate scripts.
- An authoring-only verified certificate cache.
- Promotion plan / materialize helpers from corpus modules to `npa-mathlib`.
- Updates to related AGENTS / CONTRIBUTING / workflow docs / repo-local
  skills.
- Timing measurements for the authoring loop and package gate.

Out of scope:

- Expanding the trust boundary of the kernel / certificate verifier.
- Using cache hits as grounds for proof acceptance / release verdicts /
  high-trust audits.
- Shortening public `npa-mathlib` releases with a cache.
- Registry lookup, latest-version resolution, network fetch, or implicit
  dependency solving.
- Adding proof corpus theorem content or semantically rewriting existing
  proofs.

Current implementation assumptions:

- `npa-proof-corpus` has `--build-module`, `--build-modules`,
  `--build-modules-file`, `--module`, `--changed-only`, `--write-ai-index`,
  `--write-replay`, `--shard`, `--failures-out`, `--verified-cache`,
  `--promote-plan`, and `--promote-materialize`.
- `--promote-materialize` is a dry-run by default and writes target package
  files only when `--apply` is specified.
- `./scripts/check-corpus.sh` runs the lightweight authoring gate as a
  compatibility wrapper, and split gate scripts are implemented for the
  authoring / package / full use cases.
- This repository itself has no active `.github/workflows`; the files under
  `ci-templates/github-actions/**` are copyable templates for external theorem
  package repositories.

## Trusted Boundary

```text
Not trusted:
  AI / tactic / replay / metadata / theorem index / promotion plan / cache file

Trusted:
  canonical certificate
  deterministic hash
  small Rust kernel
  source-free checker / verifier verdict
```

No milestone treats caches, promotion plans, CI status, or package metadata as
proof evidence. Release / high-trust / public `npa-mathlib` handoffs rerun
source-free verification and deterministic package artifact checks.

## Milestones

### PCT-00 Baseline Metrics And Current Command Inventory

- Status: Completed
- Depends on: None
- Inputs:
  - `develop/proof-corpus-tooling-improvement-plan.md`
  - `tools/proof-corpus/src/main.rs`
  - `scripts/check-fast.sh`
  - `scripts/check-corpus.sh`
  - `AGENTS.md`
  - `CONTRIBUTING.md`
- Deliverables:
  - Baseline timing note for current single-module authoring loop, `--changed-only`, and full corpus gate.
  - Current command inventory documenting implemented and planned `npa-proof-corpus` options.
  - A small measurement convention for later milestones.
- Acceptance criteria:
  - Baseline records exact commands, date, machine context if available, and pass/fail status.
  - The PCT-00 inventory states that `--build-modules`, `--verified-cache`, and promotion commands were not implemented at baseline time.
  - No source, certificate, manifest, or generated package artifact is changed by measurement.
- Verification:
  - `git diff --check`
  - `cargo run -p npa-proof-corpus -- --help`
  - Targeted review of the baseline note against `tools/proof-corpus/src/main.rs`
- Notes:
  - Do not run the full corpus gate repeatedly while measuring. One full-gate timing is enough for the baseline.
  - Completed in `develop/proof-corpus-tooling-baseline-pct-00.md`.

### PCT-01 Batch Build CLI And Dependency Selection

- Status: Completed
- Depends on: PCT-00
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - `develop/proof-corpus-tooling-improvement-plan.md` sections 3.2 and 3.3
- Deliverables:
  - `--build-modules MODULE...` parser and help text.
  - `--build-modules-file PATH` parser with empty-line and `#` comment handling.
  - Shared batch selection code that computes the requested module set plus import closure in topological order.
  - Unit tests for argument parsing, file parsing, duplicate modules, unknown modules, and topological ordering.
- Acceptance criteria:
  - Existing `--build-module MODULE` remains accepted and behaves as a one-module batch.
  - Batch selection builds shared import closure only once per process.
  - Invalid module names fail before writing artifacts.
  - Help output lists the new batch commands without removing existing commands.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - `cargo run -p npa-proof-corpus -- --help`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Basic Proofs.Ai.Eq`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache authoring`
- Notes:
  - Keep this milestone focused on selection and compatibility. Metadata transaction behavior is PCT-02.
  - Completed in `tools/proof-corpus/src/main.rs`.

### PCT-02 Batch Metadata Transaction And Failure Semantics

- Status: Completed
- Depends on: PCT-01
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/package-lock.json`
  - `proofs/generated/ai-theorem-index.json`
- Deliverables:
  - Metadata-once write path for batch builds.
  - Failure behavior that avoids updating package-wide metadata after partial batch failure.
  - `--failures-out PATH` support for batch build diagnostics.
  - Tests proving `--build-module` and `--build-modules` produce deterministic metadata for equivalent inputs.
- Acceptance criteria:
  - Manifest, package manifest, package lock, and AI theorem index are updated once after all selected modules build.
  - Partial failure may leave successfully generated module-local artifacts, but does not write stale package-wide metadata.
  - Re-running the same successful batch is deterministic and leaves no diff.
  - `--build-module MODULE` remains a compatibility wrapper over the batch path.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Basic Proofs.Ai.Eq`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `git diff --check`
- Notes:
  - Do not introduce exact-count audit assertions that fail only because a module was added.
  - Completed in `tools/proof-corpus/src/main.rs`.

### PCT-03 Split Corpus Gates

- Status: Completed
- Depends on: PCT-02
- Inputs:
  - `scripts/check-corpus.sh`
  - `scripts/check-fast.sh`
  - `AGENTS.md`
  - `CONTRIBUTING.md`
  - `README.md`
  - `develop/proof-corpus-ai-workflow.md`
  - `ci-templates/github-actions/**`
- Deliverables:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`
  - Compatibility wrapper behavior for existing `./scripts/check-corpus.sh`.
  - Documentation updates explaining when to use each gate.
- Acceptance criteria:
  - `check-corpus-authoring.sh` runs changed-only source-free verification with the authoring cache and is suitable for normal theorem authoring completion.
  - `check-corpus-package.sh` retains package verifier, package CLI examples, publish-plan, index, and axiom-report coverage.
  - `check-corpus-full.sh` composes authoring and package gates.
  - Existing `check-corpus.sh` remains valid and invokes the lightweight authoring gate.
  - Docs reserve package/full gates for explicit promotion, release, high-trust, or compatibility boundaries.
- Verification:
  - `bash -n scripts/check-corpus-authoring.sh scripts/check-corpus-package.sh scripts/check-corpus-full.sh scripts/check-corpus.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`
  - `rg -n "check-corpus-authoring|check-corpus-package|check-corpus-full|check-corpus.sh" AGENTS.md CONTRIBUTING.md README.md develop/proof-corpus-ai-workflow.md`
- Notes:
  - The in-repo `ci-templates` are package-repository templates, not active workflows for this repository. Do not imply that this repo has active `.github/workflows` unless they are actually added.
  - Completed with split gate scripts and repo-local docs.

### PCT-04 Promotion Plan Command

- Status: Completed
- Depends on: PCT-03
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/axiom-report.json`
  - `proofs/generated/theorem-index.json`
  - `develop/npa-mathlib-*-closure-audit.md`
  - `judge-promote-to-mathlib` skill
- Deliverables:
  - `--promote-plan CORPUS_MODULE` command.
  - `--mathlib-root PATH`, `--to-module Mathlib.*`, and `--out PATH` options.
  - Markdown promotion plan output with module mapping, import closure, axiom policy diff, exports, alias decision, gate commands, and evidence placeholders.
  - Tests for plan generation without mutating `npa-mathlib`.
- Acceptance criteria:
  - Plan generation is read-only with respect to `--mathlib-root`.
  - Unknown corpus module or invalid target `Mathlib.*` module fails with structured, deterministic diagnostics.
  - The generated plan distinguishes verified evidence from missing evidence.
  - The plan includes source-free package gate and downstream smoke commands.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - `cargo run -p npa-proof-corpus -- --promote-plan Proofs.Ai.Algebra.AbstractField --mathlib-root ../npa-mathlib --to-module Mathlib.Algebra.Field.Basic --out /tmp/npa-promote-plan.md`
  - `git diff --check`
- Notes:
  - This milestone decides and records promotion readiness. It does not materialize files into `npa-mathlib`.
  - Completed with read-only `--promote-plan` generation, deterministic diagnostics, and source-free gate commands.

### PCT-05 Verified Certificate Cache Data Model

- Status: Completed
- Depends on: PCT-04
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - `npa_cert::VerifiedModule` API
  - package lock / certificate hash types
  - `develop/proof-corpus-tooling-improvement-plan.md` section 4
- Deliverables:
  - Versioned authoring cache schema.
  - Content-addressed cache key implementation.
  - Cache path layout under `target/npa-proof-cache/verified-v0.1/`.
  - Tests for key changes across certificate hash, import identity, axiom policy, core features, verifier profile, and schema version.
- Acceptance criteria:
  - Cache key includes every field required by the design document.
  - Schema version mismatch is a miss, not an error.
  - Cache files are not read by release / high-trust / full corpus gate code paths by default.
  - Deleting the cache cannot change verification success or failure.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - Targeted unit tests for cache key equality / inequality
  - `rg -n "npa-proof-cache|verified-cache|cache_status" tools/proof-corpus/src/main.rs develop`
- Notes:
  - Keep cache serialization deterministic, but do not treat cache bytes as trusted proof evidence.
  - Completed with versioned key/entry schema, path layout helpers, and schema-mismatch-as-miss tests.

### PCT-06 Verified Cache CLI Integration

- Status: Completed
- Depends on: PCT-05
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - `scripts/check-corpus*.sh`
  - `develop/proof-corpus-ai-workflow.md`
- Deliverables:
  - `--verified-cache off|authoring|read-through` parser and help text.
  - Cache hit / miss integration for `--module` and `--changed-only`.
  - Machine-readable output field such as `cache_status = "hit"` where JSON output exists or a deterministic text equivalent otherwise.
  - Read-through mode that verifies live and compares against cache for debugging.
- Acceptance criteria:
  - Default mode is `off` for package/full corpus gates and release-like paths.
  - `authoring` mode may shorten local verification but clearly reports cached status.
  - `read-through` mode discards inconsistent cache entries and reports the live verifier result.
  - `./scripts/check-corpus-authoring.sh` and `./scripts/check-corpus.sh` enable authoring cache; package/full gates do not.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache off`
  - `./scripts/check-corpus-authoring.sh`
- Notes:
  - Implemented parser/help for `--verified-cache off|authoring|read-through`.
  - `--module` / `--changed-only` report deterministic `cache_status = "..."` text when cache mode is enabled.
  - `authoring` mode writes on miss and may skip live verification on hit; `read-through` always verifies live and rewrites stale/inconsistent entries.
  - Gate scripts keep the default cache-off behavior and do not pass `--verified-cache authoring`.

### PCT-07 Promotion Materialize Command

- Status: Completed
- Depends on: PCT-04, PCT-06
- Inputs:
  - Promotion plan output from PCT-04
  - local `../npa-mathlib` checkout
  - `npa-mathlib` package gate commands
  - downstream smoke fixture conventions
- Deliverables:
  - `--promote-materialize PLAN` command.
  - `--mathlib-root PATH` option.
  - Dry-run mode that prints intended file, manifest, package metadata, and namespace changes.
  - Apply mode that writes the target package changes.
  - Post-materialize checklist for package gates and downstream smoke.
- Acceptance criteria:
  - Dry-run mode does not modify `npa-mathlib`.
  - Apply mode stages no git changes automatically; it only writes files and reports exact paths changed.
  - Materialization preserves canonical certificates and deterministic hashes.
  - Materialization refuses plans with unresolved import mapping, axiom policy widening, or alias decisions.
  - Output tells the operator which `npa-mathlib` package commands must pass before release.
- Verification:
  - `cargo test -p npa-proof-corpus`
  - Dry-run against a known small corpus module.
  - Package gates in `npa-mathlib` after apply:
    - `cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json`
    - `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json`
    - `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json`
    - `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json`
    - `cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json`
    - `cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json`
    - `cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json`
- Notes:
  - This command helps move stable artifacts; it must not decide that proof evidence is valid without source-free verification.
  - Completed in `tools/proof-corpus/src/main.rs` with `--promote-materialize PLAN --mathlib-root PATH [--dry-run|--apply] [--compat-alias none]`.
  - Dry-run reports source / certificate / meta / replay / manifest actions and namespace change without modifying `npa-mathlib`.
  - Apply writes files only; it does not stage git changes.
  - Materialization rejects unresolved import mapping, package policy widening, and unresolved compatibility alias decisions unless the operator explicitly passes `--compat-alias none`.

### PCT-08 End-To-End Measurement And Documentation Cleanup

- Status: Completed
- Depends on: PCT-07
- Inputs:
  - Baseline from PCT-00
  - Implemented commands and scripts from PCT-01 through PCT-07
  - `AGENTS.md`
  - `CONTRIBUTING.md`
  - `README.md`
  - `develop/proof-corpus-ai-workflow.md`
  - `.agents/skills/prove-theorem/SKILL.md`
  - `judge-promote-to-mathlib` skill
- Deliverables:
  - Before/after timing summary.
  - Updated authoring workflow docs and repo-local skill instructions.
  - Stale command scan and cleanup.
  - Final verification matrix for authoring gate, package gate, cache off/on, and promotion dry-run.
- Acceptance criteria:
  - Docs consistently present local authoring checks as the default proof-corpus repair loop.
  - Docs consistently reserve full corpus/package gates for batch boundaries, push readiness, release handoff, or compatibility changes.
  - No doc claims that cache, promotion plan, theorem index, replay, metadata, or CI status is proof evidence.
  - The measured authoring loop is faster than the baseline full corpus gate, or the remaining bottleneck is documented with follow-up tasks.
- Verification:
  - `git diff --check`
  - `rg -n "check-corpus-authoring|check-corpus-package|check-corpus-full|verified-cache|promote-plan|promote-materialize" AGENTS.md CONTRIBUTING.md README.md develop skills`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
- Notes:
  - `./scripts/check-corpus-full.sh` should be run before promotion/release, or if implementation touched proof corpus package tooling or package verification.
  - Completed in `develop/proof-corpus-tooling-pct-08-measurement.md`.
  - This milestone changed documentation and repo-local skills only. It did not touch proof corpus tooling or package verification code, so the split authoring and package gates were run instead of the full compatibility wrapper.
  - The measured local small-module authoring loop is about 394 times faster than the PCT-00 full corpus gate. The package gate remains the long boundary gate and is documented as a PR / push / release / compatibility check.

## Open Questions

- Should split corpus gates become active workflows in this repository later, or remain local scripts plus external package CI templates?
- Should the verified certificate cache support only proof corpus verification first, or also package verifier authoring paths in the same milestone?
- Should promotion materialization later grow richer alias-file generation, or should aliases remain a manual package authoring step?

## Review Notes

Findings addressed while creating this task breakdown:

- The source design lists future script and CLI names; this task document marks them as unimplemented until their milestone lands.
- The source design mentions daily / PR gates, while the repository has no active `.github/workflows`; this task document treats CI integration as local scripts and external package templates unless active workflows are explicitly added.
- The cache design could be misread as a public `npa-mathlib` release accelerator; this task document constrains it to authoring-only use and cache-off release verification.
