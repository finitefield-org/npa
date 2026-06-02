# npa-mathlib Public Release Plan

This plan follows the CLR-10 registry readiness decision: continue
Git-release-based registry seed work while preparing a public theorem-library
package. It does not implement a registry server.

## Goal

Publish NPA with a substantial theorem library while preserving the
certificate-first trust boundary.

Target repository and package split:

```text
npa
  kernel, certificate format, checker, frontend, tactic, package CLI

npa-std
  small stable standard library

npa-mathlib
  public theorem library
```

The current `fixtures/npa-mathlib-seed` package is a dogfood seed. It proves
that a theorem package can be checked, released as deterministic artifacts, and
consumed downstream without a registry service. It is not the final public
`npa-mathlib` package.

## Non-Goals

- Do not build a registry server in this plan.
- Do not add network lookup, latest-version resolution, or package cache
  fallback to the kernel, certificate verifier, or checker.
- Do not treat source files, replay files, theorem indexes, publish metadata,
  CI status, or registry metadata as proof evidence.
- Do not require CLR-08 high-trust evidence for the first public
  reference-checker-only release.

## Public Package Boundary

`npa-mathlib` should be a separate repository once the local fixture is stable.
Inside this repository, use a fixture first:

```text
fixtures/npa-mathlib/
  npa-package.toml
  Mathlib/
  generated/
```

Keep `fixtures/npa-mathlib-seed/` as historical CLR-09 dogfood evidence. The
new `fixtures/npa-mathlib/` should model the public package layout and naming.

Standalone repository activation is specified in
`develop/npa-standalone-repo-activation.md`. That procedure must be followed before
public `npa-mathlib` Layer 0 artifacts are treated as an external repository
release.

## Namespace Decision

The current seed uses `Proofs.Ai.*` because it was copied from the proof corpus.
That namespace is acceptable for seed evidence but not ideal for a public
theorem library.

The standalone `npa-mathlib` repository now carries the public namespace policy
in `docs/namespace-policy.md`. That policy is the source of truth for public
module naming after repository activation.

Fixed Layer 0 public namespace shape:

```text
Mathlib.Logic.Basic
Mathlib.Logic.Prop
Mathlib.Logic.Eq
Mathlib.Data.Nat.Basic
Mathlib.Core.Reduction
```

Provisional examples for later layers must be checked against that policy before
being published:

```text
Mathlib.Algebra.Ring
Mathlib.Algebra.Square
Mathlib.Algebra.OrderedField
Mathlib.Algebra.Group.Basic
Mathlib.Algebra.Group.Subgroup
Mathlib.Vector.Basic
Mathlib.Geometry.Pythagorean
```

`Std.*` modules remain in `npa-std`. `Mathlib.*` modules may import `Std.*` via
hash-pinned package imports, never by hidden local path or module name alone.

The namespace must be fixed before public-facing artifact hashes are treated as
stable, because module names affect certificate identity, export hashes,
package locks, axiom reports, theorem indexes, publish plans, and downstream
fixtures.

## Initial Release Layers

Release layers should be closed over package imports and internal module
dependencies.

Layer 0 mapping is fixed for the first public `npa-mathlib` fixture:

| Source corpus module | Public module | Public path |
| --- | --- | --- |
| `Proofs.Ai.Basic` | `Mathlib.Logic.Basic` | `Mathlib/Logic/Basic/` |
| `Proofs.Ai.Prop` | `Mathlib.Logic.Prop` | `Mathlib/Logic/Prop/` |
| `Proofs.Ai.Eq` | `Mathlib.Logic.Eq` | `Mathlib/Logic/Eq/` |
| `Proofs.Ai.Nat` | `Mathlib.Data.Nat.Basic` | `Mathlib/Data/Nat/Basic/` |
| `Proofs.Ai.Reduction` | `Mathlib.Core.Reduction` | `Mathlib/Core/Reduction/` |

Layer 0 acceptance criteria:

- `fixtures/npa-mathlib/npa-package.toml` uses package `npa-mathlib`.
- The five public modules above are the only local modules in the first
  `fixtures/npa-mathlib/` package.
- Public module paths match the table exactly.
- `npa-std` imports remain hash-pinned package imports for `Std.Logic.Eq` and
  `Std.Nat.Basic`.
- Custom package-local axioms remain disabled and the Layer 0 axiom report has
  zero direct axioms, zero transitive axioms, and zero policy violations.
- The downstream fixture imports `npa-mathlib` modules by package, version,
  module, export hash, certificate hash, certificate file hash, and artifact
  path, never by module name alone.
- `fixtures/npa-mathlib-seed/` remains unchanged as CLR-09 evidence.

Layer 0 fixture status:

- `fixtures/npa-mathlib/` now models package `npa-mathlib` with the five public
  modules in the mapping table.
- `fixtures/npa-mathlib/generated/package-lock.json`,
  `generated/axiom-report.json`, `generated/theorem-index.json`, and
  `generated/publish-plan.json` are regenerated for the `Mathlib.*`
  namespace.
- `fixtures/npa-mathlib-downstream/` imports `Mathlib.Logic.Basic` from package
  `npa-mathlib` through a vendored source-free certificate at
  `vendor/npa-mathlib/Mathlib/Logic/Basic/certificate.npcert`.
- `crates/npa-cli/tests/package_import_fixture.rs` checks source-free public
  downstream import, publish-plan import-bundle consistency, artifact hash
  consistency, and corrupted package/hash pin rejection.
- Public release `v0.1.0` has been published at
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.0`.
- The `v0.1.0` release bundle hash is
  `d89dd2cb08ae21c20b9ca889285d9fcb50b1c133d40556e0601588a44e9632d9`.

Layer 1 was completed in `npa-mathlib v0.1.1` using the following selected
mapping. Keep later theorem layers aligned with the standalone `npa-mathlib`
namespace policy:

Layer 1, small algebra/order:

- `Proofs.Ai.Algebra.Ring`
- `Proofs.Ai.Algebra.Square`
- `Proofs.Ai.OrderedField`

Layer 1 closure audit is fixed in
`develop/npa-mathlib-layer1-closure-audit.md`. The selected public mapping is:

| Source corpus module | Public module |
| --- | --- |
| `Proofs.Ai.Algebra.Ring` | `Mathlib.Algebra.Ring` |
| `Proofs.Ai.Algebra.Square` | `Mathlib.Algebra.Square` |
| `Proofs.Ai.OrderedField` | `Mathlib.Algebra.OrderedField` |

The selected set is closed over new internal dependencies after namespace
renaming. It introduces no new external package dependency beyond the existing
hash-pinned `npa-std v0.1.0` imports already used by `npa-mathlib`.

Layer 2 is intentionally split into two smaller release candidates. Do not ship
vector and geometry in one release unless both closures have already been
audited separately.

Layer 2A, vector:

| Source corpus module | Public module | Notes |
| --- | --- | --- |
| `Proofs.Ai.Vector.Basic` | `Mathlib.Vector.Basic` | First vector API over the released Layer 1 algebra/order baseline. |
| `Proofs.Ai.Vector.Dot` | `Mathlib.Vector.Dot` | Dot-product layer; expected to depend on `Mathlib.Vector.Basic` and the released Layer 1 algebra/order modules. |

Layer 2A closure audit is fixed in
`develop/npa-mathlib-layer2a-closure-audit.md`. The selected vector closure is
limited to `npa-std v0.1.0`, the released Layer 1 algebra/order baseline, and
the selected vector modules. It does not require geometry, analysis, abstract
algebra, or abstract vector modules.

Layer 2B, geometry:

| Source corpus module | Public module | Notes |
| --- | --- | --- |
| `Proofs.Ai.Geometry.RightTriangle` | `Mathlib.Geometry.RightTriangle` | First right-triangle facts; should consume the released Layer 2A vector artifacts source-free. |
| `Proofs.Ai.Geometry.Metric` | `Mathlib.Geometry.Metric` | Metric facts over the vector/right-triangle closure. |
| `Proofs.Ai.Geometry.Pythagorean` | `Mathlib.Geometry.Pythagorean` | Deferred. Its current proof-corpus closure imports abstract/law-package modules and uses `Eq.rec`. |

Layer 2A is fixed as `npa-mathlib v0.1.2`, including package artifacts, a
release bundle, and downstream source-free smoke evidence. Layer 2B is fixed as
`npa-mathlib v0.1.3`, including package artifacts, a release bundle, and
downstream source-free smoke evidence.

Layer 2B closure audit is fixed in
`develop/npa-mathlib-layer2b-closure-audit.md`. The selected concrete geometry
closure is limited to `npa-std v0.1.0`, the released Layer 1 algebra/order
baseline, the released Layer 2A vector baseline, and
`Mathlib.Geometry.RightTriangle` / `Mathlib.Geometry.Metric`. It does not
include the abstract Pythagorean/law-package closure.

Layer 3, algebraic structures and isomorphism routes:

- group, subgroup, quotient, image, and isomorphism modules

Layer 3A, abstract group foundation:

| Source corpus module | Public module | Notes |
| --- | --- | --- |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | Reusable equality reasoning support module required by the abstract group foundation. |
| `Proofs.Ai.Algebra.AbstractGroup` | `Mathlib.Algebra.Group.Basic` | First abstract group API: group law package, hom law package, kernel predicate, kernel relation, and base group/hom/kernel facts. |

Layer 3A closure audit is fixed in
`develop/npa-mathlib-layer3a-closure-audit.md`. The selected foundation
closure is limited to `npa-std v0.1.0`, `Mathlib.Logic.EqReasoning`, and
`Mathlib.Algebra.Group.Basic`. It introduces the first public `npa-mathlib`
`Eq.rec` axiom surface, so materialization must keep
`allow_custom_axioms = false`, set `allowed_axioms = ["Eq.rec"]`, and verify
that the regenerated axiom report has zero policy violations.

Layer 3B, subgroup and normal-subgroup foundation:

| Source corpus module | Public module | Notes |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `Mathlib.Algebra.Group.Subgroup` | First subgroup and normal-subgroup API: subgroup laws, normal subgroup laws, intersection/product predicates, and normal relation facts. |

Layer 3B closure audit is fixed in
`develop/npa-mathlib-layer3b-closure-audit.md`. The selected subgroup closure
is limited to `npa-std v0.1.0`, the released `npa-mathlib v0.1.4` Layer 3A
baseline, and `Mathlib.Algebra.Group.Subgroup`. It does not introduce a new
axiom policy beyond the existing `Eq.rec` allowance.

Layer 3C, subgroup containment/order foundation:

| Source corpus module | Public module | Notes |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | `Mathlib.Algebra.Group.Subgroup.Order` | Predicate-level subgroup inclusion, subgroup equivalence, and normal-subgroup containment API. |

Layer 3C subgroup-order closure audit is fixed in
`develop/npa-mathlib-layer3c-subgroup-order-closure-audit.md`. The selected
closure is limited to `npa-std v0.1.0`, the released `npa-mathlib v0.1.5`
Layer 3B baseline, and `Mathlib.Algebra.Group.Subgroup.Order`. It introduces
no new direct or transitive axioms.

Layer 3D and later algebraic routes should remain separate audits:

- `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage`
- `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`
- `Proofs.Ai.Algebra.AbstractGroup*Iso*`
- `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`

Layer 4, analysis and functional analysis:

- metric topology, normed spaces, derivatives, inverse/implicit function
  routes, spectral theorem routes

## Artifact Stabilization Workflow

For each layer:

1. Copy the selected closed module set into `fixtures/npa-mathlib/`.
2. Rename module declarations and paths to the public `Mathlib.*` namespace.
3. Declare `npa-std` imports in `npa-package.toml` with package, version,
   module, export hash, certificate hash, and certificate artifact path.
4. Regenerate certificates for renamed modules.
5. Regenerate `generated/package-lock.json`.
6. Regenerate `generated/axiom-report.json`.
7. Regenerate `generated/theorem-index.json`.
8. Regenerate `generated/publish-plan.json`.
9. Update or create a downstream fixture that imports public `npa-mathlib`
   artifacts through the publish-plan downstream import bundle.
10. Run source-free verification and artifact freshness checks.

The expected baseline commands are:

```sh
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
```

## Axiom Policy

Prefer `allow_custom_axioms = false` for public release layers.

If a layer requires a known builtin axiom interface such as `Eq.rec`, keep that
layer separate until the public axiom policy and axiom report are reviewed. Any
allowed axiom must appear in `npa-package.toml`, `generated/axiom-report.json`,
and release documentation.

## Downstream Fixture Requirements

The downstream fixture must prove that a consumer can import `npa-mathlib`
without a registry server.

It must pin:

- package name
- package version
- module name
- export hash
- certificate hash
- certificate file hash
- certificate artifact path

It must reject corrupted package name, package version, export hash,
certificate hash, and certificate artifact hash.

## Release Evidence

The first public release may be reference-checker-only. It must include:

- `npa-package.toml`
- local certificate artifacts
- vendored or referenced `npa-std` certificate artifacts
- `generated/package-lock.json`
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `generated/publish-plan.json`
- package command JSON diagnostics
- downstream fixture evidence

`verified_high_trust` remains unavailable until the public repository supplies
CLR-08 pinned external checker artifacts, runner policies, checker registry
data, and release audit evidence.

Evidence fixed through 2026-06-03:

- `npa-mathlib v0.1.0` is published as a public GitHub Release.
- `npa-mathlib v0.1.1` is published as a public GitHub Release with the
  Layer 1 algebra/order modules.
- `npa-mathlib v0.1.2` is published as a public GitHub Release with the
  Layer 2A vector modules.
- `npa-mathlib v0.1.3` is published as a public GitHub Release with the
  Layer 2B concrete geometry modules.
- `npa-mathlib v0.1.4` is published as a public GitHub Release with the
  Layer 3A abstract group foundation modules.
- `npa-mathlib v0.1.5` is published as a public GitHub Release with the
  Layer 3B subgroup and normal-subgroup foundation module.
- The `v0.1.3` release bundle hash is
  `07e5cdf2ebb6e139fbe0473b6bc4372f830182a7c5bc39ed3dbf1a151f930602`.
- The `v0.1.4` release bundle hash is
  `d216da5522a5d4cd5e37ae059387b93632a0d04aa6ea6f9b8e757c256789ee4c`.
- The `v0.1.5` release bundle hash is
  `7893ab55d0f56e19cd0337f461d772c141442a33c80bd1113248938a6f3b930d`.
- The release artifact bundle contains only the required package manifest,
  generated package artifacts, local `Mathlib.*` certificate artifacts, and
  vendored `npa-std` certificate artifacts.
- The release notes explicitly keep source, replay, theorem index, publish
  metadata, package manifest, CI status, Git tags, and release pages outside
  proof evidence.
- A downstream source-free smoke materialized from the published release bundle
  passed `check`, `build-certs --check`, `verify-certs --checker reference`,
  and `check-hashes`.
- The `v0.1.1` downstream smoke materialized from the published release bundle
  imports only release-bundle certificate bytes for `Std.Logic.Eq`,
  `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`, and
  `Mathlib.Algebra.OrderedField`.
- The `v0.1.2` downstream smoke materialized from the published release bundle
  imports only release-bundle certificate bytes for `Std.Logic.Eq`,
  `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`,
  `Mathlib.Algebra.OrderedField`, `Mathlib.Vector.Basic`, and
  `Mathlib.Vector.Dot`.
- The `v0.1.3` downstream smoke materialized from the published release bundle
  imports only release-bundle certificate bytes for `Std.Logic.Eq`,
  `Mathlib.Algebra.Ring`, `Mathlib.Algebra.Square`,
  `Mathlib.Algebra.OrderedField`, `Mathlib.Vector.Basic`,
  `Mathlib.Vector.Dot`, `Mathlib.Geometry.RightTriangle`, and
  `Mathlib.Geometry.Metric`.
- The `v0.1.4` downstream smoke materialized from the published release bundle
  imports only release-bundle certificate bytes for `Std.Logic.Eq`,
  `Mathlib.Logic.EqReasoning`, and `Mathlib.Algebra.Group.Basic`.
- The `v0.1.5` downstream smoke materialized from the published release bundle
  imports only release-bundle certificate bytes for `Std.Logic.Eq`,
  `Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`, and
  `Mathlib.Algebra.Group.Subgroup`.
- GitHub Actions status for `npa-mathlib v0.1.1`, `v0.1.2`, `v0.1.3`,
  `v0.1.4`, and `v0.1.5` is intentionally not used as release evidence in
  this pass.
- Negative checks rejected corrupted import package name, package version,
  export hash, certificate hash, and certificate artifact data before proof
  acceptance.
- SRA-09 post-activation evidence is recorded in
  `develop/registry-readiness.md` and
  `develop/npa-standalone-repo-activation.md`.

## Layer 1 Expansion Tasks

Status: Completed for the first Layer 1 algebra/order release in
`npa-mathlib v0.1.1`.

The repository split and package manifest semantics are fixed. Do not revisit
them for later theorem layers. Add later theorem layers in the standalone
`finitefield-org/npa-mathlib` repository.

Layer 1 selected module set:

```text
Mathlib.Algebra.Ring
Mathlib.Algebra.Square
Mathlib.Algebra.OrderedField
```

Concrete task sequence:

1. Completed: selected a closed Layer 1 source set from the current proof
   corpus algebra and ordered-field candidates.
2. Completed: mapped each source module to the `Mathlib.*` namespace according
   to `npa-mathlib/docs/namespace-policy.md`.
3. Completed: kept package name `npa-mathlib` and used the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Completed: added the new module directories and manifest entries in the
   standalone `npa-mathlib` repository.
5. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
6. Completed: updated downstream source-free smoke to import the new Layer 1
   certificate closure from release-bundle bytes.
7. Completed: ran package gates for `npa-mathlib` and the downstream smoke.
8. Completed: published `npa-mathlib v0.1.1` after release bundle and
   downstream smoke evidence were fixed.

## Layer 2A Expansion Tasks

Status: Completed for the first Layer 2A vector release in
`npa-mathlib v0.1.2`.

Layer 2A selected module set:

```text
Mathlib.Vector.Basic
Mathlib.Vector.Dot
```

Concrete task sequence:

1. Completed: audited the selected vector closure in
   `develop/npa-mathlib-layer2a-closure-audit.md`.
2. Completed: mapped each source module to the `Mathlib.*` namespace according
   to `npa-mathlib/docs/namespace-policy.md`.
3. Completed: kept package name `npa-mathlib` and used the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Completed: added `Mathlib/Vector/Basic/` and `Mathlib/Vector/Dot/` in the
   standalone `npa-mathlib` repository.
5. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
6. Completed: updated downstream source-free smoke to import the Layer 2A
   certificate closure from release-bundle bytes.
7. Completed: ran package gates for `npa-mathlib` and the downstream smoke.
8. Completed: published `npa-mathlib v0.1.2` after release bundle and
   downstream smoke evidence were fixed.

## Layer 2B Expansion Tasks

Status: Completed for the first Layer 2B concrete geometry release in
`npa-mathlib v0.1.3`.

Layer 2B selected module set:

```text
Mathlib.Geometry.RightTriangle
Mathlib.Geometry.Metric
```

Concrete task sequence:

1. Completed: audited the selected concrete geometry closure in
   `develop/npa-mathlib-layer2b-closure-audit.md`.
2. Completed: mapped each source module to the `Mathlib.*` namespace according
   to `npa-mathlib/docs/namespace-policy.md`.
3. Completed: kept package name `npa-mathlib` and used the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Completed: added `Mathlib/Geometry/RightTriangle/` and
   `Mathlib/Geometry/Metric/` in the standalone `npa-mathlib` repository.
5. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
6. Completed: updated downstream source-free smoke to import the Layer 2B
   certificate closure from release-bundle bytes.
7. Completed: ran package gates for `npa-mathlib` and the downstream smoke.
8. Completed: published `npa-mathlib v0.1.3` after release bundle and
   downstream smoke evidence were fixed.

## Layer 3A Expansion Tasks

Status: Completed for the first Layer 3A abstract group foundation release in
`npa-mathlib v0.1.4`.

Layer 3A selected module set:

```text
Mathlib.Logic.EqReasoning
Mathlib.Algebra.Group.Basic
```

Concrete task sequence:

1. Completed: audited the selected abstract group foundation closure in
   `develop/npa-mathlib-layer3a-closure-audit.md`.
2. Completed: mapped each source module to the `Mathlib.*` namespace according
   to `npa-mathlib/docs/namespace-policy.md`.
3. Completed: kept package name `npa-mathlib` and used the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Completed: added `Mathlib/Logic/EqReasoning/` and
   `Mathlib/Algebra/Group/Basic/` in the standalone `npa-mathlib` repository.
5. Completed: set `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]` for the first public equality eliminator
   surface.
6. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
7. Completed: updated downstream source-free smoke to import the Layer 3A
   certificate closure from release-bundle bytes.
8. Completed: ran package gates for `npa-mathlib` and the downstream smoke.
9. Completed: published `npa-mathlib v0.1.4` after release bundle and
   downstream smoke evidence were fixed.

## Layer 3B Expansion Tasks

Status: Completed for the first Layer 3B subgroup and normal-subgroup
foundation release in `npa-mathlib v0.1.5`.

Layer 3B selected module set:

```text
Mathlib.Algebra.Group.Subgroup
```

Concrete task sequence:

1. Completed: audited the selected subgroup and normal-subgroup closure in
   `develop/npa-mathlib-layer3b-closure-audit.md`.
2. Completed: mapped the source module to the `Mathlib.*` namespace according to
   `npa-mathlib/docs/namespace-policy.md`.
3. Completed: kept package name `npa-mathlib` and used the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Completed: added `Mathlib/Algebra/Group/Subgroup/` in the standalone
   `npa-mathlib` repository.
5. Completed: kept `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`.
6. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
7. Completed: updated downstream source-free smoke to import the Layer 3B
   certificate closure from release-bundle bytes.
8. Completed: ran package gates for `npa-mathlib` and the downstream smoke.
9. Completed: published `npa-mathlib v0.1.5` after release bundle and
   downstream smoke evidence were fixed.

## Layer 3C Expansion Tasks

Status: Audit fixed for the first Layer 3C subgroup containment/order release.
Materialization is pending for `npa-mathlib v0.1.6`.

Layer 3C selected module set:

```text
Mathlib.Algebra.Group.Subgroup.Order
```

Concrete task sequence:

1. Completed: audited the selected subgroup containment/order closure in
   `develop/npa-mathlib-layer3c-subgroup-order-closure-audit.md`.
2. Pending: map the source module to the `Mathlib.*` namespace according to
   `npa-mathlib/docs/namespace-policy.md`.
3. Pending: keep package name `npa-mathlib` and use the existing
   `npa-std v0.1.0` hash-pinned imports.
4. Pending: add `Mathlib/Algebra/Group/Subgroup/Order/` in the standalone
   `npa-mathlib` repository.
5. Pending: keep `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`, while declaring no axioms for the new module.
6. Pending: regenerate certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
7. Pending: update downstream source-free smoke to import the Layer 3C
   certificate closure from release-bundle bytes.
8. Pending: run package gates for `npa-mathlib` and the downstream smoke.
9. Pending: publish `npa-mathlib v0.1.6` after release bundle and downstream
   smoke evidence are fixed.

## Immediate Tasks

1. Treat `npa-mathlib v0.1.5` as the current public theorem-library baseline
   for Layer 3B subgroup and normal-subgroup imports.
2. Materialize the audited Layer 3C subgroup containment/order closure as
   `npa-mathlib v0.1.6` with exactly
   `Mathlib.Algebra.Group.Subgroup.Order`.
3. Keep kernel, image, quotient, normal quotient, isomorphism, and
   correspondence routes in separate follow-on audits.
4. Keep `Mathlib.Geometry.Pythagorean` deferred until its abstract/law-package
   closure has a separate audit and axiom-policy review.
5. Keep CLR-08 high-trust release evidence separate from the reference-checker
   public package releases.
6. Choose each later theorem expansion layer before changing package boundaries,
   registry semantics, or import identity rules.
