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
- Public `npa-mathlib` namespace policy, standalone repository activation, and
  Layer 0 release/downstream evidence are now fixed. Larger theorem layers,
  release signing, and high-trust evidence remain explicit follow-up work.

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

The current public Layer 0 releases are reference-checker-only. Because the
public packages do not supply CLR-08 pinned external checker artifacts and
release audit evidence, `verified_high_trust` is unavailable and must not be
inferred from reference-checker-only results.

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
| B8 | External Dogfood Repo | pass | `fixtures/npa-mathlib-seed/README.md`; `fixtures/npa-mathlib-seed/CONTRIBUTING.md`; `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; `fixtures/npa-mathlib/README.md`; `fixtures/npa-mathlib-downstream/README.md`; seed and public generated artifacts; seed and public downstream fixtures; public `npa-std` and `npa-mathlib` release pages | Seed package commands passed locally; public `npa-mathlib` package commands passed locally; downstream fixtures passed; `npa-std v0.1.0` and `npa-mathlib v0.1.0` release workflows passed; published-release downstream smoke passed; `cargo test -q -p npa-cli package_import_fixture` passed | The seed and public Layer 0 releases are reference-checker-only. Registry seed entries are discoverability metadata and do not imply a live registry service, latest resolver, or trusted upload path. | Larger corpus import and CLR-08 high-trust evidence are deferred. Public Layer 0 standalone activation is complete. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md`; standalone `npa-mathlib/docs/namespace-policy.md` | Supports using Git release artifacts as the public Layer 0 package baseline before any registry server exists. |

## Post-Activation Evidence

SRA-09 fixes the public Layer 0 activation state on 2026-06-02.

Repository and package split:

```text
npa
  package CLI and checker toolchain

npa-std
  package npa-std 0.1.0

npa-mathlib
  package npa-mathlib 0.1.0
```

Exact refs:

- `finitefield-org/npa` toolchain tag `v0.1.1`
  - tag object: `8c405babb29df985b43c69fe6c857646f11cb8b7`
  - target commit: `5b1bbb3052dd2740297e9731754d5d91626352d7`
- `finitefield-org/npa-std` release tag `v0.1.0`
  - tag object: `fcfc1a51a342242719f84cd92e67b3551f3367ab`
  - target commit: `849e8eed057e4fcf42799962245db142d50eb79a`
- `finitefield-org/npa-mathlib` release tag `v0.1.0`
  - tag object: `66ee38a360c63cbe1723a7902cd4b188feb70bf0`
  - target commit: `8d8db311916cb3bae7fd9ce783139d17e3196747`

Release artifact paths:

- `npa-std v0.1.0` release:
  `https://github.com/finitefield-org/npa-std/releases/tag/v0.1.0`
- `npa-std-v0.1.0-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-std/releases/download/v0.1.0/npa-std-v0.1.0-release-artifacts.tar.gz`
- `npa-std` bundle SHA-256:
  `3ed967d1870f97f7042e87a75efebd3cf553e8c86d8959c720080115a78fe85c`
- `npa-mathlib v0.1.0` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.0`
- `npa-mathlib-v0.1.0-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.0/npa-mathlib-v0.1.0-release-artifacts.tar.gz`
- `npa-mathlib` bundle SHA-256:
  `d89dd2cb08ae21c20b9ca889285d9fcb50b1c133d40556e0601588a44e9632d9`

Command results fixed by SRA-04, SRA-07, and SRA-08:

- `npa-std` release workflow run
  `https://github.com/finitefield-org/npa-std/actions/runs/26806975884`
  completed successfully for package artifact checks, reference checker
  source-free verification, and fast-kernel source-free verification. CI status
  remains operational evidence only.
- `npa-mathlib` release gates passed: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`, `axiom-report --check`,
  `index --check`, and `publish-plan --check`.
- `npa-mathlib` release workflow run
  `https://github.com/finitefield-org/npa-mathlib/actions/runs/26822203340`
  completed successfully for package artifact checks, reference checker
  source-free verification, and fast-kernel source-free verification. CI status
  remains operational evidence only.
- Published-release downstream smoke passed `check`, `build-certs --check`,
  `verify-certs --checker reference`, and `check-hashes` after vendoring only
  `Mathlib/Logic/Basic/certificate.npcert` from the `npa-mathlib v0.1.0`
  release bundle.
- Downstream negative checks rejected corrupted package name, package version,
  export hash, certificate hash, and certificate artifact data before proof
  acceptance.
- `cargo test -q -p npa-cli package_import_fixture` passed with five tests.

Proof boundary result:

- No registry server, dependency solver, binary cache, latest-version lookup,
  hidden package cache, source checkout, theorem index, publish plan, CI status,
  Git tag, or GitHub Release page is required for proof acceptance.
- Trusted proof evidence remains canonical `.npcert` bytes plus local
  `npa-checker-ref` / kernel verification verdicts and deterministic hashes.
- `verified_high_trust` remains unavailable because CLR-08 pinned external
  checker binaries, runner policies, checker registry data, and release audit
  evidence are not part of this activation.

Remaining gaps:

- Registry server: not implemented; Git-release-based registry seed remains
  the selected path.
- Dependency solver: not implemented; downstream imports remain explicit
  package/version/module/hash pins.
- Signing: release metadata uses checksum-only policy; cryptographic signing is
  future work.
- Binary cache: not implemented; release bundles carry certificate artifacts
  directly.
- CLR-08 high-trust evidence: not provided; reference-checker-only releases
  must not be upgraded to `verified_high_trust`.

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

## Follow-Up Candidates

- Start the next closed theorem layer in the standalone `npa-mathlib`
  repository, using `docs/namespace-policy.md` as the namespace policy and
  `develop/npa-mathlib-public-release-plan.md` as the layer plan.
- Add larger theorem layers to `npa-mathlib` only after each layer has a closed
  dependency set, regenerated package artifacts, release-bundle evidence, and
  downstream import evidence.
- Keep CLR-08 high-trust evidence unavailable until a package supplies pinned
  external checker artifacts, runner policies, checker registry data, and
  release audit evidence.
- If server work is selected, split namespace ownership, authenticated publish,
  immutable version storage, artifact retention, yanking, search, moderation,
  rate limits, API versioning, and incident response into follow-up backlog
  items without changing checker proof acceptance.
