# npa-mathlib Layer 3D-B Kernel Quotient Foundation Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.7` Layer 3D-A kernel/image release. It selects the kernel quotient
foundation for group homomorphisms. It keeps first isomorphism, normal
quotient, correspondence, ring isomorphism, CRT, geometry, and analysis routes
out of this layer.

## Baseline

Current public package state:

- `npa-mathlib v0.1.7` is published from standalone repository commit
  `3239a0a0d86e7599451dfb1ff72b485716fa6047`.
- The `v0.1.7` tag object is
  `3bb8ac860641d055fce59f3be3a3d9d089c9742f`.
- The release bundle hash is
  `a5647e21f091f71e4e390f88a7bfd2f5250fa9ba7742fc4fb77729ea9dc07444`.
- Layer 3D-A public modules:
  - `Mathlib.Algebra.Group.Kernel`
  - `Mathlib.Algebra.Group.Image`
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

Layer 3D-B must add only the kernel quotient carrier, quotient multiplication
compatibility, quotient group operations/laws, and the quotient-to-codomain
multiplication theorem. It must not change package boundaries, registry
assumptions, import identity rules, or proof trust boundaries.

## Selected Candidate Set

The Layer 3D-B candidate set is closed and coherent enough to materialize next.
Publishing only the base quotient carrier would leave the public surface
without quotient multiplication, group laws, or the canonical quotient map's
multiplicativity on arbitrary quotient elements. Publishing beyond these four
modules would cross into first isomorphism or normal quotient API decisions.

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `Mathlib.Algebra.Group.Kernel.Quotient` | `Mathlib/Algebra/Group/Kernel/Quotient/` | 4 definitions, 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `Mathlib.Algebra.Group.Kernel.Quotient.Mul` | `Mathlib/Algebra/Group/Kernel/Quotient/Mul/` | 1 definition, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotient` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `Mathlib.Algebra.Group.Kernel.Quotient.Group` | `Mathlib/Algebra/Group/Kernel/Quotient/Group/` | 3 definitions, 7 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | `Mathlib.Algebra.Group.Kernel.Quotient.Hom` | `Mathlib/Algebra/Group/Kernel/Quotient/Hom/` | 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `Eq.rec` allowed/transitive |

The requested algebra namespace mapping is therefore:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `Mathlib.Algebra.Group.Kernel.Quotient` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `Mathlib.Algebra.Group.Kernel.Quotient.Mul` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `Mathlib.Algebra.Group.Kernel.Quotient.Group` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | `Mathlib.Algebra.Group.Kernel.Quotient.Hom` |

This uses `Kernel.Quotient` instead of the broader
`Mathlib.Algebra.Group.Quotient` name because the selected corpus route is not
the general normal-subgroup quotient. It is the quotient by the kernel relation
attached to a group homomorphism. The later normal quotient route should keep a
separate namespace decision.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Kernel.Quotient
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic

Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Kernel.Quotient

Mathlib.Algebra.Group.Kernel.Quotient.Group
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul

Mathlib.Algebra.Group.Kernel.Quotient.Hom
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` remain local
public `npa-mathlib` modules carried forward unchanged from the released
`v0.1.7` baseline. `Mathlib.Algebra.Group.Kernel` and
`Mathlib.Algebra.Group.Image` remain in the package baseline, but the selected
Layer 3D-B modules do not import them directly.

Downstream smoke evidence for `v0.1.8` should import the release-bundle
certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Kernel.Quotient`
- `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
- `Mathlib.Algebra.Group.Kernel.Quotient.Group`
- `Mathlib.Algebra.Group.Kernel.Quotient.Hom`

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.Kernel.Quotient` definitions:
  - `KerSetoid`
  - `KerQuot`
  - `KerQuotMk`
  - `KerQuotToH`
- `Mathlib.Algebra.Group.Kernel.Quotient` theorems:
  - `ker_quot_sound`
  - `ker_quot_to_h_mk`
  - `ker_quot_to_h_mul_mk`
- `Mathlib.Algebra.Group.Kernel.Quotient.Mul` definitions:
  - `KerQuotMulRep`
- `Mathlib.Algebra.Group.Kernel.Quotient.Mul` theorems:
  - `ker_quot_mul_rep_compat`
- `Mathlib.Algebra.Group.Kernel.Quotient.Group` definitions:
  - `KerQuotMul`
  - `KerQuotOne`
  - `KerQuotInv`
- `Mathlib.Algebra.Group.Kernel.Quotient.Group` theorems:
  - `ker_quot_mul_mk`
  - `ker_quot_inv_mk`
  - `ker_quot_mul_assoc`
  - `ker_quot_one_mul`
  - `ker_quot_mul_one`
  - `ker_quot_inv_mul`
  - `ker_quot_mul_inv`
- `Mathlib.Algebra.Group.Kernel.Quotient.Hom` theorems:
  - `ker_quot_to_h_mul`

The selected set uses the quotient primitive surface already supported by the
NPA kernel/checker profile:

- `Setoid`
- `Setoid.mk`
- `Setoid.r`
- `Quotient`
- `Quotient.mk`
- `Quotient.sound`
- `Quotient.lift`
- `Quotient.lift2`
- `Quotient.indProp`

These primitives are part of the checked certificate/kernel surface, not
source, replay, theorem-index, or registry evidence.

The selected set does not depend on:

- `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage`
- `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`
- `Proofs.Ai.Algebra.AbstractGroupFirstIso*`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`
- `Proofs.Ai.Algebra.AbstractGroupSecondIso*`
- `Proofs.Ai.Algebra.AbstractGroupThirdIso`
- `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`
- `Proofs.Ai.Algebra.AbstractRingFirstIso*`
- `Proofs.Ai.Algebra.AbstractRingChineseRemainder`
- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`

## Axiom Policy

Layer 3D-B does not widen the public axiom policy beyond the `v0.1.7`
baseline. The selected modules use only the builtin equality eliminator surface
already allowed by the public package policy.

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

The checked-in package axiom report records zero direct axioms for all selected
modules, transitive `Eq.rec` for all selected modules, and zero policy
violations. `Eq.rec` is builtin equality eliminator surface; it is not `sorry`
and it is not evidence from source, replay, theorem index, publish plan, CI,
Git tag, or registry metadata.

The generated AI theorem index records 12 theorem entries for the selected
modules:

- `ker_quot_sound`
- `ker_quot_to_h_mk`
- `ker_quot_to_h_mul_mk`
- `ker_quot_mul_rep_compat`
- `ker_quot_mul_mk`
- `ker_quot_inv_mk`
- `ker_quot_mul_assoc`
- `ker_quot_one_mul`
- `ker_quot_mul_one`
- `ker_quot_inv_mul`
- `ker_quot_mul_inv`
- `ker_quot_to_h_mul`

The AI theorem index is an untrusted sidecar. The axiom report is the source
for package-level axiom-surface evidence.

## Deferred Candidates

The following nearby routes are verified or partially supported in the proof
corpus, but they are not part of this Layer 3D-B release candidate.

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| First isomorphism | `Proofs.Ai.Algebra.AbstractGroupFirstIso*` | Depends on image plus the selected kernel quotient foundation. It should be audited after the quotient surface is public, not merged into the quotient foundation release. |
| Normal quotient | `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*` | Starts a different quotient route based on normal subgroups, not kernel relations. It needs a separate namespace decision and downstream smoke. |
| Second/third isomorphism | `Proofs.Ai.Algebra.AbstractGroupSecondIso*`, `Proofs.Ai.Algebra.AbstractGroupThirdIso` | Depends on normal quotient and subgroup foundations. Audit after normal quotient is public. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on subgroup order and normal quotient routes. Release only after their public APIs are fixed. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group image/quotient/isomorphism plus ring API decisions. Keep in later algebra layers. |

The first follow-on audit after Layer 3D-B should choose one of these routes:

1. First isomorphism:
   `Proofs.Ai.Algebra.AbstractGroupFirstIso*`, now that image and kernel
   quotient foundation will be public after materialization.
2. Normal quotient:
   `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`.
3. Second/third isomorphism:
   only after the normal quotient surface is public.
4. Correspondence:
   only after the needed normal quotient and subgroup-order surfaces are
   public.

Do not merge these routes into the `v0.1.8` kernel quotient release unless a
separate closure audit replaces this decision.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `sha256:a72a0498b88ed9cf0487d309a710ff021647e01ed9b04fe7663caa2cb7dd88e7` | `sha256:c64b8020f5bc26953e4ddcd4e003bb8a5421a326b6662f1b44ce6a1060d55748` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:30155dfd0399b8bb9222cee6aff0c26065a6766282a10f97ef2d27d45d89aa6a` | `sha256:263097d5b8ce78ffd3ce21a3a74d8d4598cf2bc18da274a99431e64b6e3739d2` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `sha256:38cc86e919557bba343c8a3270fa02656e957858321178f4b907d96e7d81f14a` | `sha256:78289f0fb3d0f5d1ac37c4be4f3d2d63543e297f37c420d2217ae9556324562e` | `sha256:149df869acd227ea5d7476160595d2338e9c00b1fb2f72d5e4ee6fca5e663c55` | `sha256:5bd7339ac208646efb0ed11ff2499737a45b42195bb28180b5685aed4560bbe3` | `sha256:a027a793c9f3fc020a9702f89376d72675625bf62776499f0b0f6ca2c69884f1` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `sha256:13d0829b279b3519e409daadd5866a9c1554a7cdbf3e3765b917085ff313431d` | `sha256:8d7e4ed32ae2d46e22ba032000d9a96f9391791983767a7c4046345398eb892c` | `sha256:4e67aa4b715757c1e8d8b4cef1637464a43fe2a4348c909b893e4f4c1330fa69` | `sha256:35475c48af8ff4a446c4f9b72745aea69e6442d8ee83fe8587a0a88eeec2e307` | `sha256:db3f019580b4c496dcabb5930444dbfdffb7421d5f676d28a12556b8df8c8eaa` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | `sha256:157d83c26c44655c5fe6453fd8764dc21ea5043ab68ffd3ed1b88d31ae96717e` | `sha256:540a44f4fd2a7fe6180b21e2f9029d442f9c546864f7caa390d4f1bcf7074bc1` | `sha256:2a0178a6cd6a9fff1e449bffb7a62e25d4e4a944ceb514083db0bcf5b0229700` | `sha256:b8556fe24d881da2124391b4b0d69d4d22f988483234a12d4f565c15909305e6` | `sha256:233e658f93574afcf1759b191e3dc5c6a25a179c7ea3285aadc918081f5147df` |

The checked-in corpus package lock records these relevant import hashes:

| Import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Proofs.Ai.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:1a146be8c2aee52e4e19e44c84357bbb40bf6f649efcc78f8f8174213abfab8e` |
| `Proofs.Ai.Algebra.AbstractGroup` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:07d04f5fa969484c0fd1c2fe9fe08cfad7d07c3de93950a54a6beaed4850b0c6` |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:263097d5b8ce78ffd3ce21a3a74d8d4598cf2bc18da274a99431e64b6e3739d2` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `sha256:149df869acd227ea5d7476160595d2338e9c00b1fb2f72d5e4ee6fca5e663c55` | `sha256:a027a793c9f3fc020a9702f89376d72675625bf62776499f0b0f6ca2c69884f1` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `sha256:4e67aa4b715757c1e8d8b4cef1637464a43fe2a4348c909b893e4f4c1330fa69` | `sha256:db3f019580b4c496dcabb5930444dbfdffb7421d5f676d28a12556b8df8c8eaa` |
| `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | `sha256:2a0178a6cd6a9fff1e449bffb7a62e25d4e4a944ceb514083db0bcf5b0229700` | `sha256:233e658f93574afcf1759b191e3dc5c6a25a179c7ea3285aadc918081f5147df` |

The public `npa-mathlib v0.1.7` package lock records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |

`Mathlib.Algebra.Group.Kernel` and `Mathlib.Algebra.Group.Image` are also
present in the `v0.1.7` package baseline, but the selected Layer 3D-B modules
do not import them directly.

## Verification

The checked-in corpus certificates for the selected modules passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotient
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotientMul
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotientGroup
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotientHom
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupQuotient`: verified 1 selected module,
  4 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupQuotientMul`: verified 1 selected module,
  5 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`: verified 1 selected module,
  6 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupQuotientHom`: verified 1 selected module,
  7 modules including dependency cache.

The source-to-certificate authoring path also regenerated the same closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotient
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotientMul
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotientGroup
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotientHom
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupQuotient` build wrote
  `proofs/generated/ai-theorem-index.json` and built 3 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroupQuotientMul` build wrote
  `proofs/generated/ai-theorem-index.json` and built 4 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` build wrote
  `proofs/generated/ai-theorem-index.json` and built 5 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`
- `Proofs.Ai.Algebra.AbstractGroupQuotientHom` build wrote
  `proofs/generated/ai-theorem-index.json` and built 6 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientHom`

The difference between source-free verification count and build count is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

The generated axiom report records for all four selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

## Readiness Decision

Layer 3D-B is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.8` with exactly these new public modules:

```text
Mathlib.Algebra.Group.Kernel.Quotient
Mathlib.Algebra.Group.Kernel.Quotient.Mul
Mathlib.Algebra.Group.Kernel.Quotient.Group
Mathlib.Algebra.Group.Kernel.Quotient.Hom
```

Do not include first isomorphism, normal quotient, second/third isomorphism,
correspondence, ring isomorphism, CRT, geometry, or analysis modules in the
same release. They are nearby routes, but they widen the public API beyond the
kernel quotient foundation.

Materialization must not copy old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.7`; provisionally this is
`v0.1.8`.

## Materialization Steps

1. Copy `Proofs.Ai.Algebra.AbstractGroupQuotient` into the standalone
   `npa-mathlib` repository under
   `Mathlib/Algebra/Group/Kernel/Quotient/`.
2. Copy `Proofs.Ai.Algebra.AbstractGroupQuotientMul` into
   `Mathlib/Algebra/Group/Kernel/Quotient/Mul/`.
3. Copy `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` into
   `Mathlib/Algebra/Group/Kernel/Quotient/Group/`.
4. Copy `Proofs.Ai.Algebra.AbstractGroupQuotientHom` into
   `Mathlib/Algebra/Group/Kernel/Quotient/Hom/`.
5. Rename module declarations to the selected public `Mathlib.*` module names.
6. Replace internal imports with `Std.Logic.Eq`,
   `Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`, and the
   selected `Mathlib.Algebra.Group.Kernel.Quotient*` modules as appropriate.
7. Keep package name `npa-mathlib` and use the existing `npa-std v0.1.0`
   hash-pinned import for `Std.Logic.Eq`.
8. Keep `Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` as
   carried-forward local modules from the released `v0.1.7` baseline.
9. Do not introduce a broader `Mathlib.Algebra.Group.Quotient` module in this
   release.
10. Add new manifest entries for the four selected modules with
    `axioms = ["Eq.rec"]`.
11. Keep `allow_custom_axioms = false` and
    `allowed_axioms = ["Eq.rec"]`.
12. Bump package and release artifacts to `v0.1.8`.
13. Regenerate certificates for the renamed modules.
14. Regenerate generated package artifacts: `package-lock.json`,
    `axiom-report.json`, `theorem-index.json`, and `publish-plan.json`.
15. Update downstream source-free smoke to import the Layer 3D-B certificate
    closure from release-bundle bytes.
16. Run package gates for `npa-mathlib` and the downstream smoke.
17. Run negative checks for bad export hash, bad certificate hash, corrupted
    certificate bytes, and bad package version before proof acceptance.
18. Create the `v0.1.8` release bundle only after generated artifacts,
    downstream evidence, and negative import hash checks are fixed.
