# npa-mathlib Layer 3D-C First Isomorphism Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.8` Layer 3D-B kernel quotient release. It selects the first
isomorphism-to-image route for group homomorphisms. It keeps the MVP
representative-only module, normal quotient, second/third isomorphism,
correspondence, ring isomorphism, CRT, geometry, and analysis routes out of
this layer.

## Baseline

Current public package state:

- `npa-mathlib v0.1.8` is published from standalone repository commit
  `dd5cec062b08023eacb3252864b64653b3199814`.
- The `v0.1.8` tag object is
  `25aa8ef4bf8a68b15807769b0a57a4cf0729ff3d`.
- The release bundle hash is
  `3d179c92c455628a9ffeb3b0cd607fcabc783cfe990a0e100f7b36fdf2696ed7`.
- Layer 3D-B public modules:
  - `Mathlib.Algebra.Group.Kernel.Quotient`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Group`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Hom`
- Layer 3D-A public modules:
  - `Mathlib.Algebra.Group.Kernel`
  - `Mathlib.Algebra.Group.Image`
- Layer 3A public homomorphism surface remains in
  `Mathlib.Algebra.Group.Basic`.
- The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3D-C must add only the first isomorphism facts over the already public
image and kernel quotient foundations. It must not change package boundaries,
registry assumptions, import identity rules, or proof trust boundaries.

## Selected Candidate Set

The selected Layer 3D-C candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` | `Mathlib.Algebra.Group.FirstIsomorphism` | `Mathlib/Algebra/Group/FirstIsomorphism/` | 5 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupImage`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotientHom` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage` | `Mathlib.Algebra.Group.FirstIsomorphism.Image` | `Mathlib/Algebra/Group/FirstIsomorphism/Image/` | 8 inductives, 2 definitions, 10 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupImage`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` | `Eq.rec` allowed/transitive |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` | `Mathlib.Algebra.Group.FirstIsomorphism` |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage` | `Mathlib.Algebra.Group.FirstIsomorphism.Image` |

This uses `FirstIsomorphism` instead of `FirstIso` in public module names to
avoid abbreviation in released package identifiers. The source theorem names
may still use the shorter existing declaration names such as
`first_iso_phi_mul`; declaration names are already part of the checked corpus
surface and will be reviewed separately if a breaking API cleanup process is
introduced.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.FirstIsomorphism
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Image
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group
  imports Mathlib.Algebra.Group.Kernel.Quotient.Hom

Mathlib.Algebra.Group.FirstIsomorphism.Image
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Image
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group
  imports Mathlib.Algebra.Group.FirstIsomorphism
```

`Mathlib.Algebra.Group.Kernel`, `Mathlib.Algebra.Group.Subgroup`, and
`Mathlib.Algebra.Group.Subgroup.Order` remain in the package baseline, but the
selected Layer 3D-C modules do not import them directly.

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.FirstIsomorphism` theorems:
  - `first_iso_phi_mul`
  - `first_iso_phi_injective`
  - `first_iso_phi_hits_image`
  - `first_iso_phi_surj_image`
  - `first_isomorphism_image_facts`
- `Mathlib.Algebra.Group.FirstIsomorphism.Image` inductives:
  - `FirstIsoQuotientAssocEvidence`
  - `FirstIsoQuotientOneMulEvidence`
  - `FirstIsoQuotientMulOneEvidence`
  - `FirstIsoQuotientInvMulEvidence`
  - `FirstIsoQuotientMulInvEvidence`
  - `FirstIsoQuotientGroupEvidence`
  - `FirstIsoImageGroupEvidence`
  - `FirstIsoTheoremEvidence`
- `Mathlib.Algebra.Group.FirstIsomorphism.Image` definitions:
  - `FirstIsoImageGroupFacts`
  - `FirstIsoImage`
- `Mathlib.Algebra.Group.FirstIsomorphism.Image` theorems:
  - `first_iso_quotient_assoc_evidence`
  - `first_iso_quotient_one_mul_evidence`
  - `first_iso_quotient_mul_one_evidence`
  - `first_iso_quotient_inv_mul_evidence`
  - `first_iso_quotient_mul_inv_evidence`
  - `first_iso_quotient_group_evidence`
  - `first_iso_image_group_evidence`
  - `first_iso_image_group_facts`
  - `first_isomorphism_theorem_evidence`
  - `first_isomorphism_to_image`

Downstream smoke evidence for `v0.1.9` should import release-bundle
certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Image`
- `Mathlib.Algebra.Group.Kernel.Quotient`
- `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
- `Mathlib.Algebra.Group.Kernel.Quotient.Group`
- `Mathlib.Algebra.Group.Kernel.Quotient.Hom`
- `Mathlib.Algebra.Group.FirstIsomorphism`
- `Mathlib.Algebra.Group.FirstIsomorphism.Image`

The downstream theorem should consume `first_isomorphism_to_image`, not only
the lower-level facts, so the smoke fixture proves the final public theorem
route is source-free importable.

## Deferred Candidates

`Proofs.Ai.Algebra.AbstractGroupFirstIso` is intentionally not selected for
this layer.

Reasons:

- It exposes `FirstIsoRepMvp`, which is an explicitly provisional MVP name and
  should not become stable public API.
- Its theorem set is a representative-only subset of the route now covered by
  `AbstractGroupFirstIsoFull` and `AbstractGroupFirstIsoImage`.
- It is not imported by the selected first isomorphism modules.
- Publishing it would force either a public `Mvp` declaration or a separate
  declaration-renaming decision, both outside this closure audit.

Nearby routes remain deferred:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Normal quotient | `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*` | Different quotient route based on normal subgroups. It needs a separate namespace decision and downstream smoke. |
| Second/third isomorphism | `Proofs.Ai.Algebra.AbstractGroupSecondIso*`, `Proofs.Ai.Algebra.AbstractGroupThirdIso` | Depends on normal quotient and subgroup foundations. Audit after normal quotient is public. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on subgroup order and normal quotient routes. Release only after their public APIs are fixed. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group first isomorphism plus ring API decisions. Keep in later algebra layers. |

## Axiom Policy

Layer 3D-C does not widen the public axiom policy beyond the `v0.1.8`
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

The generated axiom report records for the selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The generated AI theorem index records 15 theorem entries for the selected
modules. The theorem index is an untrusted sidecar; proof acceptance remains
based on canonical certificate bytes and verifier results.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` | `sha256:9f758382a0104bc2dde0c3480f6b785086c989d800f8786c8fa68d0d01e89664` | `sha256:65d44727aba81daf55886bc16b7db0828e662f95308a78906f84c96c901e2d0b` | `sha256:aaf83557307aef06abfc3bf75495930d8d4caa84c45f2d164036b68f67041563` | `sha256:9a0ded81408eba6f81619705659e98a0abcbe93475d802786eea2cf85209e352` | `sha256:4a1c2d5231da668083571d921de8eb18f133e18d8e84338b553228f73dacb9a4` |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage` | `sha256:356d7a1e7e1ddf6b67035fc058da15c4ebfa4a9811c9ac77c74d7579c847f823` | `sha256:5efc0f8a1861b251604608ebcf58dbf132989e8e52c5aef901a46e9ff35d7c0f` | `sha256:70278f8ad4b92548010b37f06e2ffa2189fe0ed1750f62e74a7b66ccc3da6bef` | `sha256:d49ad4975ee17563b2f91520c600ae88e4834d89007d2d04e0c47b5f7494f3d4` | `sha256:1bf626d2b839260590e44fe1700ac91a9f479d2e9471a1fb3cb814c8e107b09f` |

The rejected MVP module has these checked-in corpus hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupFirstIso` | `sha256:f3ceb82473fdef6c17c055eb3602b95c324f0ae81c649636b74aadedb6086610` | `sha256:55ce32e05e9304c593750fe7de1a8f3659c4edc8dd10223a5caf5f1a8ca38e84` | `sha256:a68e41d10bf747d6131f7d792f06ea9fe9d231eebd5b61eff5297d31f609cc5e` | `sha256:6b53c11846665e400ccfc86f2382a4694ce0223745eb42ecd283c672a9f65fd5` | `sha256:15d658e3b0278ca31b157beaffee7a40539eac7dffed5135c6a0e8d3d1edf311` |

The public `npa-mathlib v0.1.8` package lock records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |
| `Mathlib.Algebra.Group.Image` | `sha256:0935f963b77bb9ed124d4b38435ad2a9a19f860e1f06de11c1ab8ca3f05cdd0e` | `sha256:6b69557ec90109fac32db07e78f35a2150dbda092294245455d1b1b00bd2f588` |
| `Mathlib.Algebra.Group.Kernel.Quotient` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:de182e1a592999e7b630930514be307ac7e38c012a6214196397bd96def89ed9` |
| `Mathlib.Algebra.Group.Kernel.Quotient.Mul` | `sha256:149df869acd227ea5d7476160595d2338e9c00b1fb2f72d5e4ee6fca5e663c55` | `sha256:603136356e732b815a7040c25aa21a055ac9dfa448cfd1f2ae0f8675292d76d8` |
| `Mathlib.Algebra.Group.Kernel.Quotient.Group` | `sha256:4e67aa4b715757c1e8d8b4cef1637464a43fe2a4348c909b893e4f4c1330fa69` | `sha256:28b4b550a12b0e1cb4ae1624c6cf7007c8262b52254729110cd26fadda72acac` |
| `Mathlib.Algebra.Group.Kernel.Quotient.Hom` | `sha256:2a0178a6cd6a9fff1e449bffb7a62e25d4e4a944ceb514083db0bcf5b0229700` | `sha256:ed2a9ba856a0437961f5b62b26d755e26f5a526274d3b7f0cb2f6f5d960453f7` |

## Verification

The checked-in corpus certificates passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupFirstIso
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupFirstIsoFull
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupFirstIsoImage
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupFirstIso`: verified 1 selected module,
  6 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull`: verified 1 selected module,
  9 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage`: verified 1 selected module,
  10 modules including dependency cache.

The source-to-certificate authoring path also regenerated the closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupFirstIso
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupFirstIsoFull
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupFirstIsoImage
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupFirstIso` built 5 local modules including
  import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupImage`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupFirstIso`
- `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` built 8 local modules
  including import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupImage`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientHom`
  - `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull`
- `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage` built 9 local modules
  including import closure:
  - `Proofs.Ai.EqReasoning`
  - `Proofs.Ai.Algebra.AbstractGroup`
  - `Proofs.Ai.Algebra.AbstractGroupImage`
  - `Proofs.Ai.Algebra.AbstractGroupQuotient`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientMul`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`
  - `Proofs.Ai.Algebra.AbstractGroupQuotientHom`
  - `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull`
  - `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage`

The difference between source-free verification count and build count is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

## Readiness Decision

Layer 3D-C is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.9` with exactly these new public modules:

```text
Mathlib.Algebra.Group.FirstIsomorphism
Mathlib.Algebra.Group.FirstIsomorphism.Image
```

Do not include `Mathlib.Algebra.Group.FirstIsomorphism.Mvp`,
`Mathlib.Algebra.Group.NormalQuotient`, second/third isomorphism,
correspondence, ring isomorphism, CRT, geometry, or analysis modules in the
same release.

Materialization must rename source imports to `Mathlib.*`, regenerate
certificates, regenerate generated package artifacts, update namespace policy,
update downstream smoke fixtures, and run package/downstream/negative gates
before release.

Use the next package/release version after `v0.1.8`; provisionally this is
`v0.1.9`.

## Materialization Steps

1. Copy `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` into the standalone
   `npa-mathlib` repository under
   `Mathlib/Algebra/Group/FirstIsomorphism/`.
2. Copy `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage` into
   `Mathlib/Algebra/Group/FirstIsomorphism/Image/`.
3. Rename module declarations and imports to the selected public `Mathlib.*`
   module names.
4. Keep package name `npa-mathlib` and use the existing `npa-std v0.1.0`
   hash-pinned import for `Std.Logic.Eq`.
5. Keep all required dependency modules as carried-forward local modules from
   the released `v0.1.8` baseline.
6. Do not materialize `Proofs.Ai.Algebra.AbstractGroupFirstIso` in this
   release.
7. Add new manifest entries for the two selected modules with
   `axioms = ["Eq.rec"]`.
8. Keep `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`.
9. Bump package and release artifacts to `v0.1.9`.
10. Regenerate certificates for the renamed modules.
11. Regenerate generated package artifacts: `package-lock.json`,
    `axiom-report.json`, `theorem-index.json`, and `publish-plan.json`.
12. Update downstream source-free smoke to import the Layer 3D-C certificate
    closure from release-bundle bytes and apply `first_isomorphism_to_image`.
13. Run package gates for `npa-mathlib` and the downstream smoke.
14. Run negative checks for bad export hash, bad certificate hash, corrupted
    certificate bytes, and bad package version before proof acceptance.
15. Create the `v0.1.9` release bundle only after generated artifacts,
    downstream evidence, and negative import hash checks are fixed.
