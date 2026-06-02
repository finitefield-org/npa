# npa-mathlib Layer 3C Subgroup Order Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.5` Layer 3B subgroup and normal-subgroup foundation release. It selects
only the subgroup containment/order surface and keeps kernel, image, quotient,
normal quotient, isomorphism, and correspondence routes out of this layer.

## Baseline

Current public package state:

- `npa-mathlib v0.1.5` is published from standalone repository commit
  `3050b36f83985eabb0c64cd8dbd55554371a9ffd`.
- The `v0.1.5` tag object is
  `cc495750acf520549d237c22a71182255d32a333`.
- The release bundle hash is
  `7893ab55d0f56e19cd0337f461d772c141442a33c80bd1113248938a6f3b930d`.
- Layer 3B public module:
  - `Mathlib.Algebra.Group.Subgroup`
- Layer 3A public modules:
  - `Mathlib.Logic.EqReasoning`
  - `Mathlib.Algebra.Group.Basic`
- Layer 2B public modules:
  - `Mathlib.Geometry.RightTriangle`
  - `Mathlib.Geometry.Metric`
- Layer 2A public modules:
  - `Mathlib.Vector.Basic`
  - `Mathlib.Vector.Dot`
- Layer 1 public modules:
  - `Mathlib.Algebra.Ring`
  - `Mathlib.Algebra.Square`
  - `Mathlib.Algebra.OrderedField`
- Layer 0 public modules:
  - `Mathlib.Logic.Basic`
  - `Mathlib.Logic.Prop`
  - `Mathlib.Logic.Eq`
  - `Mathlib.Data.Nat.Basic`
  - `Mathlib.Core.Reduction`
- The public axiom policy keeps custom package-local axioms disabled and
  permits only the builtin equality eliminator surface inherited from Layer 3A:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3C must add only subgroup containment/order facts on top of
`Mathlib.Algebra.Group.Subgroup`. It must not change package boundaries,
registry assumptions, import identity rules, or proof trust boundaries.

## Selected Candidate Set

The Layer 3C candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | `Mathlib.Algebra.Group.Subgroup.Order` | `Mathlib/Algebra/Group/Subgroup/Order/` | 3 definitions, 12 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroupSubgroup` | none |

The requested algebra namespace mapping is therefore:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | `Mathlib.Algebra.Group.Subgroup.Order` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Subgroup.Order
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`, and
`Mathlib.Algebra.Group.Subgroup` remain local public `npa-mathlib` modules
carried forward unchanged from the released `v0.1.5` baseline. Downstream smoke
evidence for `v0.1.6` must import the release-bundle certificate bytes for the
full closure, including these carried-forward modules.

The selected module introduces the following public surface:

- Definitions:
  - `SubgroupLe`
  - `SubgroupEquiv`
  - `NormalContains`
- Theorems:
  - `subgroup_le_refl`
  - `subgroup_le_trans`
  - `subgroup_equiv_intro`
  - `subgroup_equiv_left`
  - `subgroup_equiv_right`
  - `subgroup_equiv_refl`
  - `subgroup_equiv_symm`
  - `subgroup_equiv_trans`
  - `normal_contains_to_subgroup_le`
  - `subgroup_le_to_normal_contains`
  - `normal_contains_refl`
  - `normal_contains_trans`

The selected set does not depend on:

- `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage`
- `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`
- `Proofs.Ai.Algebra.AbstractGroup*Iso*`
- `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`
- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`

## Axiom Policy

Layer 3C does not widen the public axiom policy beyond the `v0.1.5` baseline.
The selected module has no direct axioms and no transitive axioms in the
checked-in corpus axiom report.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entry must declare:

```toml
axioms = []
```

The package-level policy still allows `Eq.rec` because carried-forward public
modules from Layer 3A and Layer 3B use that builtin equality eliminator surface.
The regenerated package axiom report must continue to show zero policy
violations. Source, replay, theorem index, publish plan, CI, Git tag, and
release page metadata remain outside proof evidence.

## Deferred Candidates

The following nearby modules are verified in the proof corpus, but they are not
part of this Layer 3C release candidate.

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Kernel/image/quotient/isomorphism | `Proofs.Ai.Algebra.AbstractGroupKernel`, `Proofs.Ai.Algebra.AbstractGroupImage`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroup*Iso*` | Introduces homomorphism/image/quotient/isomorphism API decisions. Keep separate from predicate-level subgroup containment. |
| Normal quotient | `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*` | Depends on subgroup foundations, but introduces quotient object and multiplication/group structure surfaces. Audit separately. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on subgroup order and normal quotient routes. Release only after their public APIs are fixed. |

The first follow-on audit after Layer 3C should choose one of these routes:

1. Kernel/image/quotient/isomorphism:
   `Proofs.Ai.Algebra.AbstractGroupKernel`,
   `Proofs.Ai.Algebra.AbstractGroupImage`,
   `Proofs.Ai.Algebra.AbstractGroupQuotient`, and the first/second/third
   isomorphism modules.
2. Normal quotient:
   `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`.
3. Correspondence:
   `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`, only after the needed
   quotient and subgroup-order surfaces are public.

Do not merge these routes into the `v0.1.6` subgroup order release unless a
separate closure audit replaces this decision.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | `sha256:7f10850d8346f35748b12431cf7d7c28330dbb5f06ca6fc62d0e7da4c6ea759c` | `sha256:fa5521f2655b4877272a55985a1bfa37219f052c58fe9a0deb13efe38aae8249` | `sha256:474920dc37a91a499b50591e71ee8c73c3c0f5e68c04857ba672661e2dd40806` | `sha256:3d3fdbf6a3ca4756ceaac9853e839a84878b24c0f6290e2246a78c6184b31e0e` | `sha256:242fa3ed3a61b0cf982d2b440f79f635a0f8729ae215c5ef9d3aa65aeb63a1c6` |

The checked-in package lock records these direct import hashes for the selected
corpus module:

| Import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Proofs.Ai.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:1a146be8c2aee52e4e19e44c84357bbb40bf6f649efcc78f8f8174213abfab8e` |
| `Proofs.Ai.Algebra.AbstractGroup` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:07d04f5fa969484c0fd1c2fe9fe08cfad7d07c3de93950a54a6beaed4850b0c6` |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `sha256:c2ae287a3e5e0f4de41b6a201d345af9f83aa200f20dbdcf487d15273ca5f3b4` | `sha256:0d356dd1f768b8665f5cf9b7d4a75ea1a34422181907125fbf167263e4b8092d` |

## Verification

The checked-in corpus certificate for the selected module passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSubgroupOrder
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`: verified 1 selected module,
  5 modules including dependency cache.

The source-to-certificate authoring path also regenerated the same closure:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSubgroupOrder
```

Result:

- built `Proofs.Ai.EqReasoning`
- built `Proofs.Ai.Algebra.AbstractGroup`
- built `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- built `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`
- wrote `proofs/generated/ai-theorem-index.json`
- built 4 local modules including import closure.

The difference between source-free verification count and build count is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

The generated theorem index records 12 theorem entries for the selected
module, and all 12 have zero axiom dependencies:

- `normal_contains_refl`
- `normal_contains_to_subgroup_le`
- `normal_contains_trans`
- `subgroup_equiv_intro`
- `subgroup_equiv_left`
- `subgroup_equiv_refl`
- `subgroup_equiv_right`
- `subgroup_equiv_symm`
- `subgroup_equiv_trans`
- `subgroup_le_refl`
- `subgroup_le_to_normal_contains`
- `subgroup_le_trans`

The generated axiom report entry for
`Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` records:

- direct axioms: none
- transitive axioms: none
- policy status: ok
- policy violations: none

## Readiness Decision

Layer 3C is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.6` with exactly this new public module:

```text
Mathlib.Algebra.Group.Subgroup.Order
```

Do not include kernel, image, quotient, normal quotient, isomorphism, or
correspondence modules in the same release. They are verified nearby routes,
but they widen the public API beyond predicate-level subgroup containment and
equivalence.

Materialization must not copy old proof identity as public evidence. The
source module currently uses historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.5`; provisionally this is
`v0.1.6`.

## Materialization Steps

1. Copy `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` into the standalone
   `npa-mathlib` repository under `Mathlib/Algebra/Group/Subgroup/Order/`.
2. Rename the module declaration to `Mathlib.Algebra.Group.Subgroup.Order`.
3. Replace internal imports with `Std.Logic.Eq`,
   `Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`, and
   `Mathlib.Algebra.Group.Subgroup`.
4. Keep package name `npa-mathlib` and use the existing `npa-std v0.1.0`
   hash-pinned import for `Std.Logic.Eq`.
5. Keep `Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`, and
   `Mathlib.Algebra.Group.Subgroup` as carried-forward local modules from the
   released `v0.1.5` baseline.
6. Add the new manifest entry for `Mathlib.Algebra.Group.Subgroup.Order` with
   `axioms = []`.
7. Keep `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`.
8. Bump package and release artifacts to `v0.1.6`.
9. Regenerate certificates for the renamed module.
10. Regenerate generated package artifacts: `package-lock.json`,
    `axiom-report.json`, `theorem-index.json`, and `publish-plan.json`.
11. Update downstream source-free smoke to import
    `Mathlib.Algebra.Group.Subgroup.Order` from release-bundle certificate
    bytes.
12. Run package gates for `npa-mathlib` and the downstream smoke.
13. Create the `v0.1.6` release bundle only after generated artifacts,
    downstream evidence, and negative import hash checks are fixed.
