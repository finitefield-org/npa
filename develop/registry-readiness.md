# Registry Readiness Review

This is the CLR-10 evidence collection record. CLR-10 is a review and decision
gate; it does not implement a registry server.

Collection started: 2026-06-02.

Current decision: continue Git-release-based registry seed.

Allowed final decisions:

- create registry server
- continue Git-release-based registry seed
- defer registry work

## Decision

CLR-10 chooses to continue Git-release-based registry seed work and not start a
registry server yet.

Why this decision:

- Package manifests, source-free package verification, deterministic package
  artifacts, publish metadata, and downstream hash-pinned import fixtures are
  available.
- The current seed release can be consumed without a registry service.
- The next product need is a public theorem-library package shape, not a live
  metadata server.
- Public `npa-mathlib` namespace, standalone repository activation, release
  signing, namespace policy, and larger-corpus release layering still need
  explicit follow-up.

Why not create a registry server now:

- A server would add operations, namespace ownership, publication, search,
  moderation, storage, yanking, and incident-response work before public
  `npa-mathlib` artifacts are stable.
- Registry metadata is not proof evidence, so a server cannot replace local
  hash-pinned source-free verification.

Why not defer registry work entirely:

- The package contract and seed downstream import path are already strong
  enough to keep publishing checksum-pinned Git release artifacts while the
  public theorem-library package is prepared.

## Package Boundary Decision

Public release work will use three package/repository boundaries:

```text
npa
  kernel, certificate format, checker, frontend, tactic, package CLI

npa-std
  small standard-library package with stable core modules such as Std.Logic
  and Std.Nat

npa-mathlib
  public theorem library for community theorem development
```

`npa-mathlib-seed` remains an interim dogfood fixture and release seed. It is
not the final public theorem-library package name. The current `Proofs.Ai.*`
module namespace is accepted as CLR-09 seed evidence, but public `npa-mathlib`
work should decide and stabilize a library namespace before bulk theorem
release artifacts are treated as public-facing.

## Trusted Boundary

Registry metadata, publish metadata, theorem indexes, CI status, source files,
replay files, tactic traces, AI traces, package solver state, registry API
responses, and release upload automation are untrusted helper data.
They are not checker input and not proof evidence.

Proof acceptance remains local and source-free. Downstream packages must use
hash-pinned certificate artifacts and local checker verification. The kernel,
`npa-cert`, `npa-checker-ref`, and certificate verifier paths must not read
network data, registry state, hidden package caches, API responses, mutable
latest-version selectors, or package solver output.

The current CLR-09 seed release is reference-checker-only. Because the seed
fixture does not supply CLR-08 pinned external checker artifacts and release
audit evidence, `verified_high_trust` is unavailable and must not be inferred
from reference-checker-only results.

## Evidence Table

| blocker_id | blocker_name | status | evidence_artifacts | verification_commands | trusted_boundary_result | remaining_gap | follow_up_owner_or_file | decision_impact |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B1 | Package Manifest | pass | `crates/npa-package/src/manifest.rs`; `fixtures/npa-mathlib-seed/npa-package.toml`; `fixtures/npa-mathlib-seed-downstream/npa-package.toml`; package manifest negative fixtures under `crates/npa-package/tests/fixtures/package/invalid/` | `cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json` passed; `cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed-downstream --json` passed; `./scripts/check-fast.sh` passed manifest tests | Manifest imports are package/version/module/hash pinned; module-name-only external imports and forbidden registry lookup fields are rejected by tests. Manifest data is package metadata, not proof evidence. | None for CLR-10 evidence collection. | `develop/registry-readiness.md` | Supports readiness review without changing `npa.package.v0.1` semantics. |
| B2 | Package CLI | pass | `crates/npa-cli/src/package*.rs`; `README.md`; seed generated artifacts | Seed commands passed: `check`, `build-certs --check`, `verify-certs --checker reference`, `check-hashes`, `axiom-report --check`, `index --check`, `publish-plan --check` | CLI orchestrates deterministic local package checks. It does not add kernel, checker, or certificate network input. | None for reference-checker-only seed evidence. | `develop/registry-readiness.md` | Package command surface is available for CLR-10 evidence collection. |
| B3 | CI Contract | deferred | `ci-templates/github-actions/npa-package-pr.yml`; `ci-templates/github-actions/npa-package-release.yml`; `fixtures/npa-mathlib-seed/.github/workflows/npa-package-pr.yml`; `fixtures/npa-mathlib-seed/.github/workflows/npa-package-release.yml`; `docs/external-theorem-library-ci.md` | `python3 ci-templates/github-actions/validate-workflows.py` passed; seed package command sequence passed locally | Base PR/release templates are reference-checker-only and registry-free. Opt-in high-trust is separate and does not synthesize `verified_high_trust`. | No live standalone GitHub Actions run for a separate `npa-mathlib-seed` repository is recorded in this repo. | `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; future standalone seed repo issue | Blocks claiming live external CI evidence; does not block collecting registry-readiness evidence from the checked-in fixture. |
| B4 | External Package Import Resolution | pass | `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `fixtures/npa-mathlib-seed-downstream/npa-package.toml`; `fixtures/npa-mathlib/generated/publish-plan.json`; `fixtures/npa-mathlib-downstream/npa-package.toml`; `fixtures/npa-mathlib-downstream/generated/package-lock.json`; `fixtures/npa-mathlib-downstream/vendor/npa-mathlib/Mathlib/Logic/Basic/certificate.npcert` | Seed downstream `check`, `verify-certs --checker reference`, and `check-hashes` passed; public downstream `check`, `build-certs --check`, `verify-certs --checker reference`, and `check-hashes` passed; `cargo test -q -p npa-cli package_import_fixture` passed; `cargo test -q -p npa-package downstream_import_bundle` passed | Downstream import uses package, version, module, export hash, certificate hash, and certificate artifact hash. It does not use source files, theorem index contents, latest lookup, or registry network data. | None for the seed downstream fixture or public Layer 0 downstream fixture. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Supports Git-release-based registry seed consumption before any server exists and shows the same contract under public package name `npa-mathlib`. |
| B5 | Source-Free Package Verification | pass | `fixtures/npa-mathlib-seed/generated/package-lock.json`; `fixtures/npa-mathlib/generated/package-lock.json`; seed, public, and downstream `.npcert` artifacts; `crates/npa-api/src/package_verifier.rs`; `crates/npa-package/src/lock.rs` | Seed reference verification passed for 7 modules; public `npa-mathlib` reference verification passed for 7 modules; public downstream reference verification passed for 2 modules; `./scripts/check-fast.sh` passed package lock and verifier tests | Verification is certificate and import-artifact based in dependency order. Source, replay, meta, theorem index, AI traces, and registry metadata are not proof evidence. | None for reference checker evidence. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Source-free reference verification is ready as the CLR-10 baseline and public Layer 0 package baseline. |
| B6 | Deterministic Public Artifacts | pass | `fixtures/npa-mathlib-seed/generated/package-lock.json`; `fixtures/npa-mathlib-seed/generated/axiom-report.json`; `fixtures/npa-mathlib-seed/generated/theorem-index.json`; `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `fixtures/npa-mathlib/generated/package-lock.json`; `fixtures/npa-mathlib/generated/axiom-report.json`; `fixtures/npa-mathlib/generated/theorem-index.json`; `fixtures/npa-mathlib/generated/publish-plan.json` | `check-hashes`, `axiom-report --check`, `index --check`, and `publish-plan --check` passed for the seed fixture and public `npa-mathlib` fixture; `cargo test -q -p npa-package publish_plan` passed; `./scripts/check-fast.sh` passed | Generated artifacts are deterministic metadata. They do not become proof evidence and do not include mutable registry resolution as checker input. | Add explicit byte-identical rerun evidence if a final release audit requires it. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Artifact freshness evidence is sufficient to continue Git-release-based public package preparation. |
| B7 | Publish Metadata | pass | `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `proofs/generated/publish-plan.json`; `crates/npa-package/src/publish_plan.rs`; `crates/npa-package/src/registry.rs` | Seed `publish-plan --check` passed; `proofs` `publish-plan --check` passed; `cargo test -q -p npa-package publish_plan` passed | `npa.registry.module.v0.1` theorem package metadata is separate from independent checker binary registry metadata such as `npa.independent-checker.checker_binary_registry.v1`. Publish metadata is discoverability/import helper data, not proof evidence. | None for checksum-only MVP metadata. Signing remains later release workflow work. | `develop/registry-readiness.md` | CLR-06 publish metadata can feed the registry-readiness decision. |
| B8 | External Dogfood Repo | pass | `fixtures/npa-mathlib-seed/README.md`; `fixtures/npa-mathlib-seed/CONTRIBUTING.md`; `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; `fixtures/npa-mathlib/README.md`; `fixtures/npa-mathlib-downstream/README.md`; seed and public generated artifacts; seed and public downstream fixtures | Seed package commands passed locally; public `npa-mathlib` package commands passed locally; downstream fixtures passed; `DOGFOOD-AUDIT.md` records no blocking CLR-09 findings; `./scripts/check-fast.sh` passed | The seed and public Layer 0 releases are reference-checker-only. Registry seed entries are discoverability metadata and do not imply a live registry service, latest resolver, or trusted upload path. | Standalone repository activation, larger corpus import, and CLR-08 high-trust evidence are deferred. | `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; `develop/npa-mathlib-public-release-plan.md`; future standalone repository backlog | Supports using the fixture as CLR-10 input and as the local baseline for public `npa-mathlib` repository activation. |

## Collected Command Evidence

The following commands were run from the repository root and passed:

```sh
cargo run -q -p npa-cli -- package check --root proofs --json
cargo run -q -p npa-cli -- package publish-plan --root proofs --check --json
cargo test -q -p npa-package publish_plan
cargo test -q -p npa-cli package_publish

cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib-seed --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib-seed --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib-seed --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-mathlib-seed --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-mathlib-seed --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib-seed --check --json

cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed-downstream --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib-seed-downstream --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib-seed-downstream --json

cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib --check --json

cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-downstream --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib-downstream --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib-downstream --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib-downstream --json
cargo test -q -p npa-cli package_import_fixture
cargo test -q -p npa-package downstream_import_bundle

python3 ci-templates/github-actions/validate-workflows.py
git diff --check
./scripts/check-fast.sh
```

## Seed Publish Plan Facts

`fixtures/npa-mathlib-seed/generated/publish-plan.json` currently records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib-seed`
- version: `0.1.0`
- release artifact count: 11
- module registry seed entry count: 5
- downstream import bundle module count: 5
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The seed generated artifacts also record:

- package lock entries: 7
- theorem index entries: 54
- theorem index checker summaries: 7
- axiom report modules: 7
- axiom report local modules: 5
- axiom report external modules: 2
- direct axiom count: 0
- transitive axiom count: 0
- policy violation count: 0

## Public Layer 0 Publish Plan Facts

`fixtures/npa-mathlib/generated/publish-plan.json` currently records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.0`
- release artifact count: 11
- module registry seed entry count: 5
- downstream import bundle module count: 5
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The public Layer 0 generated artifacts also record:

- package lock entries: 7
- theorem index entries: 54
- theorem index checker summaries: 7
- axiom report modules: 7
- axiom report local modules: 5
- axiom report external modules: 2
- direct axiom count: 0
- transitive axiom count: 0
- policy violation count: 0

`fixtures/npa-mathlib-downstream/generated/package-lock.json` records one
local downstream module, `Downstream.MathlibBasic`, and one external imported
module, `Mathlib.Logic.Basic`, with the external certificate path
`vendor/npa-mathlib/Mathlib/Logic/Basic/certificate.npcert`.

## Deferred Follow-Up Candidates

- Record or create a live standalone `npa-mathlib-seed` repository CI run if
  CLR-10 needs evidence beyond the checked-in fixture and workflow validator.
- Activate standalone repositories for `npa`, `npa-std`, and `npa-mathlib`
  according to `develop/npa-standalone-repo-activation.md`. The local public Layer
  0 baseline is fixed by SRA-00 evidence, and the SRA-02-compatible `npa`
  toolchain reference is fixed by `v0.1.1` evidence. The next activation
  dependency is the standalone `npa-std` package repository image.
- Add larger theorem layers to `npa-mathlib` only after each layer has a closed
  dependency set, regenerated package artifacts, and downstream import
  evidence.
- Use `develop/npa-mathlib-public-release-plan.md` as the immediate follow-up plan
  for public theorem-library release preparation.
- Keep CLR-08 high-trust evidence unavailable until the seed repository
  supplies pinned external checker artifacts, runner policies, checker
  registry data, and release audit evidence.
- If server work is selected, split namespace ownership, authenticated publish,
  immutable version storage, artifact retention, yanking, search, moderation,
  rate limits, API versioning, and incident response into follow-up backlog
  items without changing checker proof acceptance.
