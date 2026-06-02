# npa-mathlib Layer 3D-A Hom Kernel Image Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.6` Layer 3C subgroup containment/order release. It selects only the
group kernel and image surface. It keeps quotient, normal quotient,
isomorphism, correspondence, ring isomorphism, and geometry routes out of this
layer.

## Baseline

Current public package state:

- `npa-mathlib v0.1.6` is published from standalone repository commit
  `7d2471d76263e966a61dbdc7c86199589cefa605`.
- The `v0.1.6` tag object is
  `3346dbd7dea47236d24280ece75e38322a442c23`.
- The release bundle hash is
  `e16b09b55956ee8709b4cb639bf06ad2b3f60463a41f9170ed34cc8feb7d0bda`.
- Layer 3C public module:
  - `Mathlib.Algebra.Group.Subgroup.Order`
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

Layer 3D-A must add only the kernel and image facts already supported by the
public homomorphism surface in `Mathlib.Algebra.Group.Basic`. It must not
change package boundaries, registry assumptions, import identity rules, or
proof trust boundaries.

## Selected Candidate Set

The Layer 3D-A candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | `Mathlib.Algebra.Group.Kernel` | `Mathlib/Algebra/Group/Kernel/` | 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `Mathlib.Algebra.Group.Image` | `Mathlib/Algebra/Group/Image/` | 1 definition, 5 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` allowed/transitive |

The requested algebra namespace mapping is therefore:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | `Mathlib.Algebra.Group.Kernel` |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `Mathlib.Algebra.Group.Image` |

No separate `Mathlib.Algebra.Group.Hom` module is selected for `v0.1.7`.
`GroupHomLawArgs`, `hom_mul`, `hom_one`, and `hom_inv` are already part of the
released `Mathlib.Algebra.Group.Basic` surface. Splitting them into a new
module now would create a proof-identity and import-surface refactor instead of
a small theorem expansion.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Kernel
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic

Mathlib.Algebra.Group.Image
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` remain local
public `npa-mathlib` modules carried forward unchanged from the released
`v0.1.6` baseline. `Mathlib.Algebra.Group.Subgroup` and
`Mathlib.Algebra.Group.Subgroup.Order` remain in the package baseline, but the
selected Layer 3D-A modules do not import them.

Downstream smoke evidence for `v0.1.7` should import the release-bundle
certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Kernel`
- `Mathlib.Algebra.Group.Image`

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.Kernel` theorems:
  - `kernel_mul_closed`
  - `kernel_inv_closed`
  - `kernel_conj_closed`
- `Mathlib.Algebra.Group.Image` definitions:
  - `ImagePred`
- `Mathlib.Algebra.Group.Image` theorems:
  - `image_intro`
  - `image_elim`
  - `image_one`
  - `image_mul_closed`
  - `image_inv_closed`

The selected set does not depend on:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`
- `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`
- `Proofs.Ai.Algebra.AbstractGroupQuotientHom`
- `Proofs.Ai.Algebra.AbstractGroup*Iso*`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`
- `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`
- `Proofs.Ai.Algebra.AbstractRingFirstIso*`
- `Proofs.Ai.Algebra.AbstractRingChineseRemainder`
- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`

## Axiom Policy

Layer 3D-A does not widen the public axiom policy beyond the `v0.1.6`
baseline. The selected modules use only the builtin equality eliminator
surface already allowed by the public package policy.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entries should declare:

```toml
axioms = ["Eq.rec"]
```

The checked-in package axiom report records zero direct axioms for both
selected modules, transitive `Eq.rec` for both selected modules, and zero
policy violations. `Eq.rec` is builtin equality eliminator surface; it is not
`sorry` and it is not evidence from source, replay, theorem index, publish
plan, CI, Git tag, or registry metadata.

The generated AI theorem index records 8 theorem entries for the selected
modules. The introduction/elimination/unit-image entries have no theorem-level
axiom dependencies. The kernel closure entries and image multiplication/inverse
closure entries list the expected `Eq.rec` equality-reasoning dependency.

## Deferred Candidates

The following nearby routes are verified or partially supported in the proof
corpus, but they are not part of this Layer 3D-A release candidate.

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Hom split | none selected; existing surface is in `Proofs.Ai.Algebra.AbstractGroup` | `GroupHomLawArgs` and `hom_*` are already public through `Mathlib.Algebra.Group.Basic`. Splitting them is a refactor, not a small theorem layer. |
| Quotient foundation | `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | Introduces quotient/setoid-facing public API and quotient module naming decisions. Audit separately. |
| First isomorphism | `Proofs.Ai.Algebra.AbstractGroupFirstIso*` | Depends on image plus quotient route. Release only after quotient foundation is public. |
| Normal quotient | `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*` | Depends on subgroup foundations and quotient object decisions. Keep separate from kernel/image. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on subgroup order and normal quotient routes. Release only after their public APIs are fixed. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group image/quotient/isomorphism plus ring API decisions. Keep in later algebra layers. |

The first follow-on audit after Layer 3D-A should choose one of these routes:

1. Quotient foundation:
   `Proofs.Ai.Algebra.AbstractGroupQuotient`,
   `Proofs.Ai.Algebra.AbstractGroupQuotientMul`,
   `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`, and
   `Proofs.Ai.Algebra.AbstractGroupQuotientHom`.
2. First isomorphism:
   `Proofs.Ai.Algebra.AbstractGroupFirstIso*`, only after the quotient
   foundation surface is public.
3. Normal quotient:
   `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`.
4. Correspondence:
   `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`, only after the needed
   quotient and subgroup-order surfaces are public.

Do not merge these routes into the `v0.1.7` kernel/image release unless a
separate closure audit replaces this decision.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | `sha256:8d11dd74da047650e74282aeea0d7a0ac2aacfc5cb3bae1bf85c12bf5c38fadb` | `sha256:6d4390bbcf7d5b22c92d24b144575cbc3f870971e03574e43633bd9a64068144` | `sha256:5c1a39b0e9f82dc1014e3e63549c71eab3db6383024a87408e4e95e06bc33fbb` | `sha256:39f92e3a74edf201d900d93abeebb9c404ea8c141bfc46761f7e889c9f8cf9f7` | `sha256:1d5b2a02b6cc7d9dfafbb781f3d27b9494e7655644aa046871bbb7abb9170bd8` |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `sha256:8c6f85eb72c6d04a2b5b97448d7aa7f7b331250591be2afe3ce329ed6a171080` | `sha256:f5e396c6b08319835864296ee0fbb273dbfeda738652f4f44767e05cf57023cc` | `sha256:0935f963b77bb9ed124d4b38435ad2a9a19f860e1f06de11c1ab8ca3f05cdd0e` | `sha256:0869976f857fbf07454df1db004a649ae3ee9ede0e1429dbf40af6dfaac5bfeb` | `sha256:af05a4354e4be42ad76b6fb1da6a149d34dfe5207637fdff9b7ff246076935a1` |

The checked-in corpus package lock records these direct import hashes for the
selected modules:

| Import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Proofs.Ai.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:1a146be8c2aee52e4e19e44c84357bbb40bf6f649efcc78f8f8174213abfab8e` |
| `Proofs.Ai.Algebra.AbstractGroup` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:07d04f5fa969484c0fd1c2fe9fe08cfad7d07c3de93950a54a6beaed4850b0c6` |

The public `npa-mathlib v0.1.6` package lock records these corresponding
public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |

## Verification

The checked-in corpus certificates for the selected modules passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupImage
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupKernel`: verified 1 selected module,
  4 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupImage`: verified 1 selected module,
  4 modules including dependency cache.

The source-to-certificate authoring path also regenerated the same closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupImage
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupKernel` build wrote
  `proofs/generated/ai-theorem-index.json` and built 3 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage` build wrote
  `proofs/generated/ai-theorem-index.json` and built 3 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupImage`

The difference between source-free verification count and build count is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

The generated theorem index records 8 theorem entries for the selected
modules:

- `kernel_mul_closed`
- `kernel_inv_closed`
- `kernel_conj_closed`
- `image_intro`
- `image_elim`
- `image_one`
- `image_mul_closed`
- `image_inv_closed`

The entries for `image_intro`, `image_elim`, and `image_one` list no
theorem-level axiom dependencies. The entries for `kernel_mul_closed`,
`kernel_inv_closed`, `kernel_conj_closed`, `image_mul_closed`, and
`image_inv_closed` list the expected `Eq.rec` equality-reasoning dependency.

The generated axiom report records for both selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

## Readiness Decision

Layer 3D-A is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.7` with exactly these new public modules:

```text
Mathlib.Algebra.Group.Kernel
Mathlib.Algebra.Group.Image
```

Do not include a new `Mathlib.Algebra.Group.Hom` module in the same release.
The required homomorphism surface is already public through
`Mathlib.Algebra.Group.Basic`.

Do not include quotient, normal quotient, isomorphism, correspondence, ring
isomorphism, CRT, geometry, or analysis modules in the same release. They are
verified nearby routes, but they widen the public API beyond kernel/image
closure.

Materialization must not copy old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.6`; provisionally this is
`v0.1.7`.

## Materialization Steps

1. Copy `Proofs.Ai.Algebra.AbstractGroupKernel` into the standalone
   `npa-mathlib` repository under `Mathlib/Algebra/Group/Kernel/`.
2. Copy `Proofs.Ai.Algebra.AbstractGroupImage` into the standalone
   `npa-mathlib` repository under `Mathlib/Algebra/Group/Image/`.
3. Rename module declarations to `Mathlib.Algebra.Group.Kernel` and
   `Mathlib.Algebra.Group.Image`.
4. Replace internal imports with `Std.Logic.Eq`,
   `Mathlib.Logic.EqReasoning`, and `Mathlib.Algebra.Group.Basic`.
5. Keep package name `npa-mathlib` and use the existing `npa-std v0.1.0`
   hash-pinned import for `Std.Logic.Eq`.
6. Keep `Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` as
   carried-forward local modules from the released `v0.1.6` baseline.
7. Do not create `Mathlib.Algebra.Group.Hom` in this release.
8. Add new manifest entries for `Mathlib.Algebra.Group.Kernel` and
   `Mathlib.Algebra.Group.Image` with `axioms = ["Eq.rec"]`.
9. Keep `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`.
10. Bump package and release artifacts to `v0.1.7`.
11. Regenerate certificates for the renamed modules.
12. Regenerate generated package artifacts: `package-lock.json`,
    `axiom-report.json`, `theorem-index.json`, and `publish-plan.json`.
13. Update downstream source-free smoke to import
    `Mathlib.Algebra.Group.Kernel` and `Mathlib.Algebra.Group.Image` from
    release-bundle certificate bytes.
14. Run package gates for `npa-mathlib` and the downstream smoke.
15. Run negative checks for bad export hash, bad certificate hash, corrupted
    certificate bytes, and bad package version before proof acceptance.
16. Create the `v0.1.7` release bundle only after generated artifacts,
    downstream evidence, and negative import hash checks are fixed.

## Materialization Result

Layer 3D-A was materialized in the standalone `npa-mathlib` repository as
`npa-mathlib v0.1.7`.

Release identity:

- repository: `finitefield-org/npa-mathlib`
- commit: `3239a0a0d86e7599451dfb1ff72b485716fa6047`
- tag: `v0.1.7`
- tag object: `3bb8ac860641d055fce59f3be3a3d9d089c9742f`
- release URL:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.7`
- release bundle:
  `npa-mathlib-v0.1.7-release-artifacts.tar.gz`
- release bundle SHA-256:
  `a5647e21f091f71e4e390f88a7bfd2f5250fa9ba7742fc4fb77729ea9dc07444`

Added public modules:

| Public module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Group.Kernel` | `sha256:962c2db5be02218d92e681a9d92035d12cd06f23515aafad793dd5612ae14736` | `sha256:66724ebc31b2a845aa5e91dc9468f5470ee1b6ecb22b7b5632dfc912b3f96d0c` | `sha256:5c1a39b0e9f82dc1014e3e63549c71eab3db6383024a87408e4e95e06bc33fbb` | `sha256:39f92e3a74edf201d900d93abeebb9c404ea8c141bfc46761f7e889c9f8cf9f7` | `sha256:8d1c72935c2c22edd3911a907df17b3e84b95ebae56fb27f4f514dcd6dbef14d` |
| `Mathlib.Algebra.Group.Image` | `sha256:904df4b2602b8e0a7799aaeea9e6e310e61138f497c5fcc6a44d24443c9c3b9c` | `sha256:c4efe5d45e4dc167cd9137675e993d3ffe5678f4579484814736b72cfc77fa30` | `sha256:0935f963b77bb9ed124d4b38435ad2a9a19f860e1f06de11c1ab8ca3f05cdd0e` | `sha256:0869976f857fbf07454df1db004a649ae3ee9ede0e1429dbf40af6dfaac5bfeb` | `sha256:6b69557ec90109fac32db07e78f35a2150dbda092294245455d1b1b00bd2f588` |

Generated artifact summary:

- package version: `0.1.7`
- package lock entries: 20
- local modules: 18
- external imports: 2
- release artifact count: 24
- module registry seed entries: 18
- theorem index entries: 226
- theorem index entries for the two new modules: 8
- axiom report modules: 20
- direct axiom count: 1
- transitive axiom count: 5
- policy violation count: 0
- publish plan hash:
  `sha256:79886bc28382a09da6d3a2508d29cd78277ecb099a10a52df3ce5d8adb7711c1`

The downstream smoke fixture imports only release-bundle certificate bytes for:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Kernel`
- `Mathlib.Algebra.Group.Image`

The downstream local theorems are:

- `Downstream.GroupKernelImage::kernel_mul_closed_passthrough`
- `Downstream.GroupKernelImage::image_intro_passthrough`

The main package passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
git -C /Users/kazuyoshitoshiya/ff/npa-mathlib diff --check
```

The downstream smoke fixture passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

Negative checks rejected:

- bad `Mathlib.Algebra.Group.Kernel` export hash
- bad `Mathlib.Algebra.Group.Image` certificate hash
- corrupted certificate bytes
- bad package version

The next theorem expansion should not extend `v0.1.7` in place. The follow-on
Layer 3D-B kernel quotient foundation audit is recorded in
`develop/npa-mathlib-layer3d-b-quotient-foundation-closure-audit.md`.
