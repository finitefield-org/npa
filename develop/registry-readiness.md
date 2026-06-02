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
- Public `npa-mathlib` namespace policy, standalone repository activation,
  Layer 0 release/downstream evidence, Layer 1 algebra/order release evidence,
  Layer 2A vector release evidence, Layer 2B concrete geometry release
  evidence, Layer 3A abstract group foundation release evidence, Layer 3B
  subgroup release evidence, Layer 3C subgroup containment/order release
  evidence, and Layer 3D-A kernel/image release evidence are now fixed. Larger
  theorem layers, release signing, and high-trust evidence remain explicit
  follow-up work.

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

The current public releases are reference-checker-only. Because the public
packages do not supply CLR-08 pinned external checker artifacts and release
audit evidence, `verified_high_trust` is unavailable and must not be inferred
from reference-checker-only results.

## Evidence Table

| blocker_id | blocker_name | status | evidence_artifacts | verification_commands | trusted_boundary_result | remaining_gap | follow_up_owner_or_file | decision_impact |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B1 | Package Manifest | pass | `crates/npa-package/src/manifest.rs`; `fixtures/npa-mathlib-seed/npa-package.toml`; `fixtures/npa-mathlib-seed-downstream/npa-package.toml`; package manifest negative fixtures under `crates/npa-package/tests/fixtures/package/invalid/` | `cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json` passed; `cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-seed-downstream --json` passed; `./scripts/check-fast.sh` passed manifest tests | Manifest imports are package/version/module/hash pinned; module-name-only external imports and forbidden registry lookup fields are rejected by tests. Manifest data is package metadata, not proof evidence. | None for CLR-10 evidence collection. | `develop/registry-readiness.md` | Supports readiness review without changing `npa.package.v0.1` semantics. |
| B2 | Package CLI | pass | `crates/npa-cli/src/package*.rs`; `README.md`; seed generated artifacts | Seed commands passed: `check`, `build-certs --check`, `verify-certs --checker reference`, `check-hashes`, `axiom-report --check`, `index --check`, `publish-plan --check` | CLI orchestrates deterministic local package checks. It does not add kernel, checker, or certificate network input. | None for reference-checker-only seed evidence. | `develop/registry-readiness.md` | Package command surface is available for CLR-10 evidence collection. |
| B3 | CI Contract | deferred | `ci-templates/github-actions/npa-package-pr.yml`; `ci-templates/github-actions/npa-package-release.yml`; `fixtures/npa-mathlib-seed/.github/workflows/npa-package-pr.yml`; `fixtures/npa-mathlib-seed/.github/workflows/npa-package-release.yml`; `docs/external-theorem-library-ci.md` | `python3 ci-templates/github-actions/validate-workflows.py` passed; seed package command sequence passed locally | Base PR/release templates are reference-checker-only and registry-free. Opt-in high-trust is separate and does not synthesize `verified_high_trust`. | No live standalone GitHub Actions run for a separate `npa-mathlib-seed` repository is recorded in this repo. | `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; future standalone seed repo issue | Blocks claiming live external CI evidence; does not block collecting registry-readiness evidence from the checked-in fixture. |
| B4 | External Package Import Resolution | pass | `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `fixtures/npa-mathlib-seed-downstream/npa-package.toml`; `fixtures/npa-mathlib/generated/publish-plan.json`; `fixtures/npa-mathlib-downstream/npa-package.toml`; `fixtures/npa-mathlib-downstream/generated/package-lock.json`; public `npa-mathlib v0.1.0` through `v0.1.7` release-bundle downstream smoke evidence | Seed downstream `check`, `verify-certs --checker reference`, and `check-hashes` passed; public downstream `check`, `build-certs --check`, `verify-certs --checker reference`, and `check-hashes` passed for `v0.1.0` through `v0.1.7`; `cargo test -q -p npa-cli package_import_fixture` passed; `cargo test -q -p npa-package downstream_import_bundle` passed | Downstream import uses package, version, module, export hash, certificate hash, and certificate artifact hash. It does not use source files, theorem index contents, latest lookup, or registry network data. | None for the seed downstream fixture or public Layer 0 through Layer 3D-A downstream smoke. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Supports Git-release-based registry seed consumption before any server exists and shows the same contract under public package name `npa-mathlib`. |
| B5 | Source-Free Package Verification | pass | `fixtures/npa-mathlib-seed/generated/package-lock.json`; `fixtures/npa-mathlib/generated/package-lock.json`; seed, public, and downstream `.npcert` artifacts; public `npa-mathlib v0.1.0` through `v0.1.7` release bundles; `crates/npa-api/src/package_verifier.rs`; `crates/npa-package/src/lock.rs` | Seed reference verification passed for 7 modules; public `npa-mathlib v0.1.0` reference verification passed for 7 modules, `v0.1.1` for 10 modules, `v0.1.2` for 12 modules, `v0.1.3` for 14 modules, `v0.1.4` for 16 modules, `v0.1.5` for 17 modules, `v0.1.6` for 18 modules, and `v0.1.7` for 20 modules; public downstream reference verification passed for 2 modules in `v0.1.0`, 5 modules in `v0.1.1`, 7 modules in `v0.1.2`, 9 modules in `v0.1.3`, 4 modules in `v0.1.4`, 5 modules in `v0.1.5`, 6 modules in `v0.1.6`, and 6 modules in `v0.1.7`; `./scripts/check-fast.sh` passed package lock and verifier tests | Verification is certificate and import-artifact based in dependency order. Source, replay, meta, theorem index, AI traces, and registry metadata are not proof evidence. | None for reference checker evidence. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Source-free reference verification is ready as the CLR-10 baseline and current public package baseline. |
| B6 | Deterministic Public Artifacts | pass | `fixtures/npa-mathlib-seed/generated/package-lock.json`; `fixtures/npa-mathlib-seed/generated/axiom-report.json`; `fixtures/npa-mathlib-seed/generated/theorem-index.json`; `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `fixtures/npa-mathlib/generated/package-lock.json`; `fixtures/npa-mathlib/generated/axiom-report.json`; `fixtures/npa-mathlib/generated/theorem-index.json`; `fixtures/npa-mathlib/generated/publish-plan.json`; standalone `npa-mathlib v0.1.0` through `v0.1.7` generated package artifacts | `check-hashes`, `axiom-report --check`, `index --check`, and `publish-plan --check` passed for the seed fixture, public `npa-mathlib` fixture, and standalone `npa-mathlib v0.1.0` through `v0.1.7` release states; `cargo test -q -p npa-package publish_plan` passed; `./scripts/check-fast.sh` passed | Generated artifacts are deterministic metadata. They do not become proof evidence and do not include mutable registry resolution as checker input. | Add explicit byte-identical rerun evidence if a final release audit requires it. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md` | Artifact freshness evidence is sufficient to continue Git-release-based public package preparation. |
| B7 | Publish Metadata | pass | `fixtures/npa-mathlib-seed/generated/publish-plan.json`; `proofs/generated/publish-plan.json`; `crates/npa-package/src/publish_plan.rs`; `crates/npa-package/src/registry.rs` | Seed `publish-plan --check` passed; `proofs` `publish-plan --check` passed; `cargo test -q -p npa-package publish_plan` passed | `npa.registry.module.v0.1` theorem package metadata is separate from independent checker binary registry metadata such as `npa.independent-checker.checker_binary_registry.v1`. Publish metadata is discoverability/import helper data, not proof evidence. | None for checksum-only MVP metadata. Signing remains later release workflow work. | `develop/registry-readiness.md` | CLR-06 publish metadata can feed the registry-readiness decision. |
| B8 | External Dogfood Repo | pass | `fixtures/npa-mathlib-seed/README.md`; `fixtures/npa-mathlib-seed/CONTRIBUTING.md`; `fixtures/npa-mathlib-seed/DOGFOOD-AUDIT.md`; `fixtures/npa-mathlib/README.md`; `fixtures/npa-mathlib-downstream/README.md`; seed and public generated artifacts; seed and public downstream fixtures; public `npa-std` and `npa-mathlib` release pages | Seed package commands passed locally; public `npa-mathlib` package commands passed locally; downstream fixtures passed; `npa-std v0.1.0` and `npa-mathlib v0.1.0` release workflows passed; `npa-mathlib v0.1.1` through `v0.1.7` release gates passed locally; published-release downstream smoke passed for `v0.1.0` through `v0.1.7`; `cargo test -q -p npa-cli package_import_fixture` passed | The seed and public releases are reference-checker-only. Registry seed entries are discoverability metadata and do not imply a live registry service, latest resolver, or trusted upload path. | Larger corpus import and CLR-08 high-trust evidence are deferred. Public Layer 3D-A kernel/image release evidence is fixed. | `develop/registry-readiness.md`; `develop/npa-mathlib-public-release-plan.md`; standalone `npa-mathlib/docs/namespace-policy.md` | Supports using Git release artifacts as the public package baseline before any registry server exists. |

## Post-Activation Evidence

SRA-09 fixes the public Layer 0 activation state on 2026-06-02. The same
evidence record now includes the `npa-mathlib v0.1.1` Layer 1 algebra/order
release continuation, the `npa-mathlib v0.1.2` Layer 2A vector release
continuation, the `npa-mathlib v0.1.3` Layer 2B concrete geometry release
continuation, the `npa-mathlib v0.1.4` Layer 3A abstract group foundation
release continuation, the `npa-mathlib v0.1.5` Layer 3B subgroup and
normal-subgroup foundation release continuation, the `npa-mathlib v0.1.6`
Layer 3C subgroup containment/order release continuation, and the
`npa-mathlib v0.1.7` Layer 3D-A kernel/image release continuation.

Repository and package split:

```text
npa
  package CLI and checker toolchain

npa-std
  package npa-std 0.1.0

npa-mathlib
  package npa-mathlib 0.1.7
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
- `finitefield-org/npa-mathlib` release tag `v0.1.1`
  - tag object: `04dba2cd9de58f2e02e990fa583939dbfa82e9ae`
  - target commit: `449855a37cbf1d3ebe777d5a6b044d47be324532`
- `finitefield-org/npa-mathlib` release tag `v0.1.2`
  - tag object: `d59032b305272d5fec557f3c07700720b2b51e27`
  - target commit: `4c28e82d3dc2e0a8a25bb2e01bb433c7a10a28fe`
- `finitefield-org/npa-mathlib` release tag `v0.1.3`
  - tag object: `689748138908401e0b9f9a1b58cce907e945f18b`
  - target commit: `dd5283666592ac9a15def166d0f7f11b197449f8`
- `finitefield-org/npa-mathlib` release tag `v0.1.4`
  - tag object: `88bc4cc1addae12f1babd45eeee4caa2a302a932`
  - target commit: `e775afff5b9a2abe7709d7d66afe333c37cab955`
- `finitefield-org/npa-mathlib` release tag `v0.1.5`
  - tag object: `cc495750acf520549d237c22a71182255d32a333`
  - target commit: `3050b36f83985eabb0c64cd8dbd55554371a9ffd`
- `finitefield-org/npa-mathlib` release tag `v0.1.6`
  - tag object: `3346dbd7dea47236d24280ece75e38322a442c23`
  - target commit: `7d2471d76263e966a61dbdc7c86199589cefa605`
- `finitefield-org/npa-mathlib` release tag `v0.1.7`
  - tag object: `3bb8ac860641d055fce59f3be3a3d9d089c9742f`
  - target commit: `3239a0a0d86e7599451dfb1ff72b485716fa6047`

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
- `npa-mathlib v0.1.1` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.1`
- `npa-mathlib-v0.1.1-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.1/npa-mathlib-v0.1.1-release-artifacts.tar.gz`
- `npa-mathlib v0.1.1` bundle SHA-256:
  `ada3f288537dc777697c1083765790aa9dbd8782f43356c1f8572a1fa6ccbcb9`
- `npa-mathlib v0.1.2` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.2`
- `npa-mathlib-v0.1.2-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.2/npa-mathlib-v0.1.2-release-artifacts.tar.gz`
- `npa-mathlib v0.1.2` bundle SHA-256:
  `7b1d8fe69b0bca46e77149453e79ece8198473ce9e760d90e9f8e2c66b117d68`
- `npa-mathlib v0.1.3` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.3`
- `npa-mathlib-v0.1.3-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.3/npa-mathlib-v0.1.3-release-artifacts.tar.gz`
- `npa-mathlib v0.1.3` bundle SHA-256:
  `07e5cdf2ebb6e139fbe0473b6bc4372f830182a7c5bc39ed3dbf1a151f930602`
- `npa-mathlib v0.1.4` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.4`
- `npa-mathlib-v0.1.4-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.4/npa-mathlib-v0.1.4-release-artifacts.tar.gz`
- `npa-mathlib v0.1.4` bundle SHA-256:
  `d216da5522a5d4cd5e37ae059387b93632a0d04aa6ea6f9b8e757c256789ee4c`
- `npa-mathlib v0.1.5` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.5`
- `npa-mathlib-v0.1.5-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.5/npa-mathlib-v0.1.5-release-artifacts.tar.gz`
- `npa-mathlib v0.1.5` bundle SHA-256:
  `7893ab55d0f56e19cd0337f461d772c141442a33c80bd1113248938a6f3b930d`
- `npa-mathlib v0.1.6` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.6`
- `npa-mathlib-v0.1.6-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.6/npa-mathlib-v0.1.6-release-artifacts.tar.gz`
- `npa-mathlib v0.1.6` bundle SHA-256:
  `e16b09b55956ee8709b4cb639bf06ad2b3f60463a41f9170ed34cc8feb7d0bda`
- `npa-mathlib v0.1.7` release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.7`
- `npa-mathlib-v0.1.7-release-artifacts.tar.gz`:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.7/npa-mathlib-v0.1.7-release-artifacts.tar.gz`
- `npa-mathlib v0.1.7` bundle SHA-256:
  `a5647e21f091f71e4e390f88a7bfd2f5250fa9ba7742fc4fb77729ea9dc07444`

Command results fixed by SRA-04, SRA-07, SRA-08, and the `v0.1.2` /
`v0.1.3` / `v0.1.4` / `v0.1.5` / `v0.1.6` / `v0.1.7` release evidence:

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
- `npa-mathlib v0.1.1` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.1` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`, and
  `Mathlib.Algebra.OrderedField`.
- `npa-mathlib v0.1.2` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.2` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`,
  `Mathlib.Algebra.OrderedField`, `Mathlib.Vector.Basic`, and
  `Mathlib.Vector.Dot`.
- `npa-mathlib v0.1.3` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.3` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`,
  `Mathlib.Algebra.OrderedField`, `Mathlib.Vector.Basic`,
  `Mathlib.Vector.Dot`, `Mathlib.Geometry.RightTriangle`, and
  `Mathlib.Geometry.Metric`.
- `npa-mathlib v0.1.4` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.4` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`, and
  `Mathlib.Algebra.Group.Basic`.
- `npa-mathlib v0.1.5` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.5` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
  `Mathlib.Algebra.Group.Basic`, and `Mathlib.Algebra.Group.Subgroup`.
- `npa-mathlib v0.1.6` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.6` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
  `Mathlib.Algebra.Group.Basic`, `Mathlib.Algebra.Group.Subgroup`, and
  `Mathlib.Algebra.Group.Subgroup.Order`.
- `npa-mathlib v0.1.7` release gates passed locally in the standalone
  repository: `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `npa-mathlib v0.1.7` published-release downstream smoke passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes` after vendoring only release-bundle certificate bytes for
  `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
  `Mathlib.Algebra.Group.Basic`, `Mathlib.Algebra.Group.Kernel`, and
  `Mathlib.Algebra.Group.Image`.
- GitHub Actions status for `npa-mathlib v0.1.1`, `v0.1.2`, `v0.1.3`,
  `v0.1.4`, `v0.1.5`, `v0.1.6`, and `v0.1.7` was intentionally ignored.
  This record uses local package gates and published release-bundle downstream
  smoke as operational evidence.
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
- Broader theorem layers beyond the released Layer 3D-A kernel/image set
  remain future `npa-mathlib` release work.

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

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa-mathlib` for the `npa-mathlib v0.1.1`
Layer 1 continuation and passed:

```sh
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root . --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package axiom-report --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package index --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package publish-plan --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root fixtures/downstream-smoke --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root fixtures/downstream-smoke --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root fixtures/downstream-smoke --json
git diff --check
```

The published `npa-mathlib v0.1.1` release bundle was then downloaded, checked
against its SHA sidecar value, extracted into a temporary directory, and used to
re-materialize the downstream smoke vendored dependency tree. The same
downstream `check`, `build-certs --check`, `verify-certs --checker reference`,
and `check-hashes` commands passed against the temporary downstream package.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa-mathlib` for the `npa-mathlib v0.1.2`
Layer 2A continuation and passed:

```sh
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root . --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package axiom-report --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package index --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package publish-plan --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root fixtures/downstream-smoke --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root fixtures/downstream-smoke --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root fixtures/downstream-smoke --json
git diff --check
```

The published `npa-mathlib v0.1.2` release bundle was downloaded, checked
against its SHA sidecar value, extracted into a temporary directory, and
source-free checked with `package check`, `verify-certs --checker reference`,
`axiom-report --check`, `index --check`, and `publish-plan --check`. The root
release bundle intentionally excludes source files, so `package check-hashes`
is not the release-bundle gate. The temporary downstream package then vendored
only certificate bytes from the downloaded release bundle and passed `check`,
`build-certs --check`, `verify-certs --checker reference`, and `check-hashes`.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa-mathlib` for the `npa-mathlib v0.1.3`
Layer 2B continuation and passed:

```sh
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root . --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package axiom-report --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package index --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package publish-plan --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root fixtures/downstream-smoke --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root fixtures/downstream-smoke --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root fixtures/downstream-smoke --json
git diff --check
```

The published `npa-mathlib v0.1.3` release bundle was downloaded, checked
against its SHA sidecar value, extracted into a temporary directory, and
source-free checked with `package check`, `verify-certs --checker reference`,
`axiom-report --check`, `index --check`, and `publish-plan --check`. The root
release bundle intentionally excludes source files, so `package check-hashes`
is not the source-free release-bundle gate. The temporary downstream package
then vendored only certificate bytes from the downloaded release bundle and
passed `check`, `build-certs --check`, `verify-certs --checker reference`, and
`check-hashes`.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa-mathlib` for the `npa-mathlib v0.1.4`
Layer 3A continuation and passed:

```sh
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root . --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root . --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package axiom-report --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package index --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package publish-plan --root . --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check --root fixtures/downstream-smoke --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package build-certs --root fixtures/downstream-smoke --check --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
/Users/kazuyoshitoshiya/ff/npa/target/debug/npa package check-hashes --root fixtures/downstream-smoke --json
git diff --check
```

The published GitHub release asset digest for
`npa-mathlib-v0.1.4-release-artifacts.tar.gz` matched the local bundle
SHA-256:
`d216da5522a5d4cd5e37ae059387b93632a0d04aa6ea6f9b8e757c256789ee4c`.
The downstream package vendored only certificate bytes from the release bundle
for `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`, and
`Mathlib.Algebra.Group.Basic`, then passed `check`, `build-certs --check`,
`verify-certs --checker reference`, and `check-hashes`.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa` for the `npa-mathlib v0.1.5` Layer 3B
continuation and passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
git -C /Users/kazuyoshitoshiya/ff/npa-mathlib diff --check
```

The published GitHub release asset digest for
`npa-mathlib-v0.1.5-release-artifacts.tar.gz` matched the local bundle
SHA-256:
`7893ab55d0f56e19cd0337f461d772c141442a33c80bd1113248938a6f3b930d`.
The downstream package vendored only certificate bytes from the release bundle
for `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
`Mathlib.Algebra.Group.Basic`, and `Mathlib.Algebra.Group.Subgroup`, then
passed `check`, `build-certs --check`, `verify-certs --checker reference`, and
`check-hashes`.

The `v0.1.5` negative checks rejected a bad
`Mathlib.Algebra.Group.Subgroup` export hash, bad certificate hash, corrupted
certificate bytes, and bad package version before proof acceptance.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa` for the `npa-mathlib v0.1.6` Layer 3C
continuation and passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
git -C /Users/kazuyoshitoshiya/ff/npa-mathlib diff --check
```

The published GitHub release asset digest for
`npa-mathlib-v0.1.6-release-artifacts.tar.gz` matched the local bundle
SHA-256:
`e16b09b55956ee8709b4cb639bf06ad2b3f60463a41f9170ed34cc8feb7d0bda`.
The downstream package vendored only certificate bytes from the release bundle
for `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
`Mathlib.Algebra.Group.Basic`, `Mathlib.Algebra.Group.Subgroup`, and
`Mathlib.Algebra.Group.Subgroup.Order`, then passed `check`,
`build-certs --check`, `verify-certs --checker reference`, and `check-hashes`.

The `v0.1.6` negative checks rejected a bad
`Mathlib.Algebra.Group.Subgroup.Order` export hash, bad certificate hash,
corrupted certificate bytes, and bad package version before proof acceptance.

The following additional commands were run from
`/Users/kazuyoshitoshiya/ff/npa` for the `npa-mathlib v0.1.7` Layer 3D-A
continuation and passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
git -C /Users/kazuyoshitoshiya/ff/npa-mathlib diff --check
```

The published GitHub release asset digest for
`npa-mathlib-v0.1.7-release-artifacts.tar.gz` matched the local bundle
SHA-256:
`a5647e21f091f71e4e390f88a7bfd2f5250fa9ba7742fc4fb77729ea9dc07444`.
The downstream package vendored only certificate bytes from the release bundle
for `Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
`Mathlib.Algebra.Group.Basic`, `Mathlib.Algebra.Group.Kernel`, and
`Mathlib.Algebra.Group.Image`, then passed `check`,
`build-certs --check`, `verify-certs --checker reference`, and `check-hashes`.

The `v0.1.7` negative checks rejected a bad
`Mathlib.Algebra.Group.Kernel` export hash, a bad
`Mathlib.Algebra.Group.Image` certificate hash, corrupted certificate bytes,
and bad package version before proof acceptance.

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

## Public Layer 1 Release Facts

The standalone `npa-mathlib v0.1.1` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.1`
- release artifact count: 14
- module registry seed entry count: 8
- downstream import bundle module count: 8
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The standalone `npa-mathlib v0.1.1` generated artifacts also record:

- package lock entries: 10
- theorem index entries: 94
- theorem index checker summaries: 10
- axiom report modules: 10
- axiom report local modules: 8
- axiom report external modules: 2
- direct axiom count: 0
- transitive axiom count: 0
- policy violation count: 0

The standalone `v0.1.1` downstream smoke package lock recorded one local
downstream module, `Downstream.AlgebraOrderedField`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Algebra.Ring`
- `Mathlib.Algebra.Square`
- `Mathlib.Algebra.OrderedField`

## Public Layer 2A Release Facts

The standalone `npa-mathlib v0.1.2` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.2`
- release artifact count: 16
- module registry seed entry count: 10
- downstream import bundle module count: 10
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The standalone `npa-mathlib v0.1.2` generated artifacts also record:

- package lock entries: 12
- theorem index entries: 123
- theorem index checker summaries: 12
- axiom report modules: 12
- axiom report local modules: 10
- axiom report external modules: 2
- direct axiom count: 0
- transitive axiom count: 0
- policy violation count: 0

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.VectorDot`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Algebra.Ring`
- `Mathlib.Algebra.Square`
- `Mathlib.Algebra.OrderedField`
- `Mathlib.Vector.Basic`
- `Mathlib.Vector.Dot`

## Public Layer 2B Release Facts

The standalone `npa-mathlib v0.1.3` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.3`
- release artifact count: 18
- module registry seed entry count: 12
- downstream import bundle module count: 12
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The standalone `npa-mathlib v0.1.3` generated artifacts also record:

- package lock entries: 14
- theorem index entries: 144
- theorem index checker summaries: 14
- axiom report modules: 14
- axiom report local modules: 12
- axiom report external modules: 2
- direct axiom count: 0
- transitive axiom count: 0
- policy violation count: 0

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.GeometryMetric`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Algebra.Ring`
- `Mathlib.Algebra.Square`
- `Mathlib.Algebra.OrderedField`
- `Mathlib.Vector.Basic`
- `Mathlib.Vector.Dot`
- `Mathlib.Geometry.RightTriangle`
- `Mathlib.Geometry.Metric`

## Public Layer 3A Release Facts

The standalone `npa-mathlib v0.1.4` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.4`
- release artifact count: 20
- module registry seed entry count: 14
- downstream import bundle module count: 14
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty

The standalone `npa-mathlib v0.1.4` generated artifacts also record:

- package lock entries: 16
- theorem index entries: 178
- theorem index checker summaries: 16
- axiom report modules: 16
- axiom report local modules: 14
- axiom report external modules: 2
- direct axiom count: 1
- transitive axiom count: 2
- policy violation count: 0
- allowed axioms: `Eq.rec`

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.GroupBasic`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`

## Public Layer 3B Release Facts

The standalone `npa-mathlib v0.1.5` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.5`
- release artifact count: 21
- module registry seed entry count: 15
- downstream import bundle module count: 15
- checker summary count: 34
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty
- publish plan hash:
  `sha256:f1c9e185a5f64d07efc6fd13ec005d832ffe62f6cc8f8564f506067b10fc0191`

The standalone `npa-mathlib v0.1.5` generated artifacts also record:

- package lock entries: 17
- theorem index entries: 206
- theorem index checker summaries: 17
- axiom report modules: 17
- axiom report local modules: 15
- axiom report external modules: 2
- direct axiom count: 1
- transitive axiom count: 3
- policy violation count: 0
- allowed axioms: `Eq.rec`

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.GroupSubgroup`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`

## Public Layer 3C Release Facts

The standalone `npa-mathlib v0.1.6` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.6`
- release artifact count: 22
- module registry seed entry count: 16
- downstream import bundle module count: 16
- checker summary count: 36
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty
- publish plan hash:
  `sha256:4d41f14bf900a4339b8325bcee6165ced870eee7eefb4eaf438962b057815155`

The standalone `npa-mathlib v0.1.6` generated artifacts also record:

- package lock entries: 18
- theorem index entries: 218
- theorem index checker summaries: 18
- axiom report modules: 18
- axiom report local modules: 16
- axiom report external modules: 2
- direct axiom count: 1
- transitive axiom count: 3
- policy violation count: 0
- allowed axioms: `Eq.rec`

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.GroupSubgroupOrder`, and these
external imported modules:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`
- `Mathlib.Algebra.Group.Subgroup.Order`

## Public Layer 3D-A Release Facts

The standalone `npa-mathlib v0.1.7` generated publish plan records:

- schema: `npa.package.publish_plan.v0.1`
- package: `npa-mathlib`
- version: `0.1.7`
- release artifact count: 24
- module registry seed entry count: 18
- downstream import bundle module count: 18
- checker summary count: 40
- signature policy: `checksum-only`
- hash algorithm: `sha256`
- signatures: empty
- publish plan hash:
  `sha256:79886bc28382a09da6d3a2508d29cd78277ecb099a10a52df3ce5d8adb7711c1`

The standalone `npa-mathlib v0.1.7` generated artifacts also record:

- package lock entries: 20
- theorem index entries: 226
- theorem index checker summaries: 20
- axiom report modules: 20
- axiom report local modules: 18
- axiom report external modules: 2
- direct axiom count: 1
- transitive axiom count: 5
- policy violation count: 0
- allowed axioms: `Eq.rec`

The standalone `fixtures/downstream-smoke/generated/package-lock.json` records
one local downstream module, `Downstream.GroupKernelImage`, and these external
imported modules:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Kernel`
- `Mathlib.Algebra.Group.Image`

## Follow-Up Candidates

- Treat `npa-mathlib v0.1.7` as the current public theorem-library baseline
  for Layer 3D-B kernel quotient materialization.
- Materialize the audited Layer 3D-B kernel quotient closure as
  `npa-mathlib v0.1.8` with exactly the four
  `Mathlib.Algebra.Group.Kernel.Quotient*` modules.
- Keep the homomorphism surface in `Mathlib.Algebra.Group.Basic`; do not
  create a separate `Mathlib.Algebra.Group.Hom` module without a new audit.
- Keep `Mathlib.Geometry.Pythagorean` deferred until its abstract/law-package
  closure has a separate audit and axiom-policy review.
- Keep first isomorphism, normal quotient, second/third isomorphism, and
  correspondence routes in separate closure audits before adding those modules
  to standalone `npa-mathlib`.
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
