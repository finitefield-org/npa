# npa-mathlib Layer 3D-F Third Isomorphism Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the Layer 3D-E
second-isomorphism materialization. It selects the abstract-group third
isomorphism route as a single closure. It keeps correspondence, ring
isomorphism, CRT, geometry, and analysis routes out of this layer.

## Baseline

Current intended package state:

- Layer 3D-E has been materialized locally in the standalone repository
  `/Users/kazuyoshitoshiya/ff/npa-mathlib` as `npa-mathlib v0.1.11`.
- `v0.1.11` release publication, tag creation, and push are still pending.
  Layer 3D-F must not be published until the `v0.1.11` release-bundle evidence
  is reviewed and accepted.
- The local ignored `v0.1.11` release-bundle tar hash observed during the
  previous materialization is
  `sha256:84826c82142ad1b601ee15b36a2f87fd6897403a253cf7f32d7e12dc7c4d4532`.
- Layer 3D-E public modules:
  - `Mathlib.Algebra.Group.SecondIsomorphism.Map`
  - `Mathlib.Algebra.Group.SecondIsomorphism.Kernel`
  - `Mathlib.Algebra.Group.SecondIsomorphism.Image`
  - `Mathlib.Algebra.Group.SecondIsomorphism`
- Layer 3D-D public modules:
  - `Mathlib.Algebra.Group.Quotient`
  - `Mathlib.Algebra.Group.Quotient.Mul`
  - `Mathlib.Algebra.Group.Quotient.Group`
- Layer 3D-C public modules:
  - `Mathlib.Algebra.Group.FirstIsomorphism`
  - `Mathlib.Algebra.Group.FirstIsomorphism.Image`
- Layer 3D-B public kernel-quotient modules remain separate from the standard
  normal-subgroup quotient route:
  - `Mathlib.Algebra.Group.Kernel.Quotient`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Group`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Hom`
- Layer 3B public subgroup and normal-subgroup surface remains in
  `Mathlib.Algebra.Group.Subgroup`.
- Layer 3A public homomorphism and group-law surface remains in
  `Mathlib.Algebra.Group.Basic`.
- The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3D-F must add only the third-isomorphism theorem evidence route for
nested normal quotients `G/N -> G/H` where `N <= H`. It must not change package
boundaries, registry assumptions, import identity rules, or proof trust
boundaries.

## Selected Candidate Set

The selected Layer 3D-F candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupThirdIso` | `Mathlib.Algebra.Group.ThirdIsomorphism` | `Mathlib/Algebra/Group/ThirdIsomorphism/` | 12 definitions, 16 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupQuotient`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `Eq.rec` allowed/transitive |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupThirdIso` | `Mathlib.Algebra.Group.ThirdIsomorphism` |

The root module `Mathlib.Algebra.Group.ThirdIsomorphism` is reserved for the
full third-isomorphism theorem evidence. This layer does not introduce
additional public submodules because the checked-in corpus route is already a
single module and its internal helper definitions are tightly coupled.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.ThirdIsomorphism
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`,
`Mathlib.Algebra.Group.Kernel.Quotient`,
`Mathlib.Algebra.Group.Subgroup`, and `Mathlib.Algebra.Group.Quotient.*`
remain local public `npa-mathlib` modules carried forward unchanged from the
released baseline. The third-isomorphism source does not directly import the
Layer 3D-E second-isomorphism modules.

The selected module introduces the following public surface:

- `Mathlib.Algebra.Group.ThirdIsomorphism` definitions:
  - `ThirdIsoGN`
  - `ThirdIsoGNOne`
  - `ThirdIsoGNMul`
  - `ThirdIsoGNInv`
  - `ThirdIsoHNPred`
  - `ThirdIsoHNSubgroupLawArgs`
  - `ThirdIsoHNNormalSubgroupLawArgs`
  - `ThirdIsoPhi`
  - `ThirdIsoPhiKernelQuot`
  - `ThirdIsoKernelPred`
  - `ThirdIsoKernelEvidence`
  - `ThirdIsoTheoremEvidence`
- `Mathlib.Algebra.Group.ThirdIsomorphism` theorems:
  - `third_iso_rel_lift`
  - `third_iso_hn_intro`
  - `third_iso_hn_elim`
  - `third_iso_hn_one`
  - `third_iso_hn_mul_closed`
  - `third_iso_hn_inv_closed`
  - `third_iso_hn_conj_closed`
  - `third_iso_phi_mk`
  - `third_iso_phi_mul`
  - `third_iso_phi_one`
  - `third_iso_phi_inv`
  - `third_iso_phi_surjective`
  - `third_iso_hn_to_kernel_sound`
  - `third_iso_kernel_intro`
  - `third_iso_kernel_evidence`
  - `third_isomorphism_theorem_evidence`

Downstream smoke evidence for the materialized release should import
release-bundle certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Kernel.Quotient`
- `Mathlib.Algebra.Group.Subgroup`
- `Mathlib.Algebra.Group.Quotient`
- `Mathlib.Algebra.Group.Quotient.Mul`
- `Mathlib.Algebra.Group.Quotient.Group`
- `Mathlib.Algebra.Group.ThirdIsomorphism`

The downstream theorem should consume `third_isomorphism_theorem_evidence`,
not only `ThirdIsoPhi` or a helper lemma, so the smoke fixture proves the final
theorem-evidence route is source-free importable.

## Deferred Candidates

The following nearby routes are intentionally not selected for Layer 3D-F:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence`, `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder`, `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`, `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal` | Depends on normal quotient plus subgroup order and saturation/preimage/image APIs. Its public API is broader than the third-isomorphism theorem evidence selected here. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group quotient/isomorphism plus ring API decisions. Keep in later algebra layers. |
| Geometry and analysis routes | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Analysis.*`, `Proofs.Ai.LinearAlgebra.*`, `Proofs.Ai.FunctionalAnalysis.*` | Not part of the group isomorphism closure sequence. |

## Axiom Policy

Layer 3D-F does not widen the public axiom policy beyond the `v0.1.11`
baseline. The selected module uses only the builtin equality eliminator surface
already allowed by the public package policy.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entry should declare:

```toml
axioms = ["Eq.rec"]
```

The generated corpus axiom report records for the selected module:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The generated AI theorem index records 16 theorem entries for the selected
module. The theorem index is an untrusted sidecar; proof acceptance remains
based on canonical certificate bytes and verifier results.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupThirdIso` | `sha256:b6a916e915abf98eb4256da9fadbb5ce5143911dc92f37bcab2cc0dca307ed34` | `sha256:57f653c648f5dbb2995dadf5bf7b27eae520d929a6b1259ece495c462a971ddd` | `sha256:f76985158305810227c882d221f2401891dafe496873aa2b4ba58d1b844e4a68` | `sha256:0108ad64717a722b59da86273c6f6182ab9467621b71e499023718c8ea574f6a` | `sha256:7f7f3e78e7dea037823c7054d84f3718dc6f6dbe4b3d5c36dc72f5d747917769` |

The local `npa-mathlib v0.1.11` materialization records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |
| `Mathlib.Algebra.Group.Kernel.Quotient` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:de182e1a592999e7b630930514be307ac7e38c012a6214196397bd96def89ed9` |
| `Mathlib.Algebra.Group.Subgroup` | `sha256:2b527c5b3039075812d564522facb4a41e421e1902f863df111f0e83c5b9d85c` | `sha256:b3b47c9d314d69594d1c80e24fd4d1954c242ef289e2031207be9acd798ce2a9` |
| `Mathlib.Algebra.Group.Quotient` | `sha256:14d52188e958a89a9f87ad32a12c2738a73c0e185d68bcd390875bf4dea7f4de` | `sha256:6fc7c518e7eaa04e792bb1d4dd3af7b4f5c4987136a118ce98a75d601c4feb19` |
| `Mathlib.Algebra.Group.Quotient.Mul` | `sha256:9570dfe4c8b6ca61fe53acd2bf41cf193be20a4d9118cbc295f1c1a10d602cc7` | `sha256:5495b3aba0eb579efd418b5a86fad16765c2f21889c9531747fa0d152f774013` |
| `Mathlib.Algebra.Group.Quotient.Group` | `sha256:fb6bb1472ee946b6db2174ccf4cf845b0a7dda6c8c42ccc9396950a5a8a92654` | `sha256:41a716e2220952290f35148b55d9505f1c8f7b90491233c10ee704bb5b678ef1` |

## Verification

The checked-in corpus certificate passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupThirdIso
```

Result:

```text
verified Proofs.Ai.Algebra.AbstractGroupThirdIso
verified 1 selected module(s), 9 module(s) including dependency cache
```

The selected module and its import closure also passed source rebuilding:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupThirdIso
```

Result:

```text
built Proofs.Ai.Algebra.AbstractGroupThirdIso
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Algebra.AbstractGroupThirdIso (8 module(s) including import closure)
```

After this rebuild, `git diff -- proofs/generated/ai-theorem-index.json`
reported no tracked diff.

## Decision

Materialize Layer 3D-F as `npa-mathlib v0.1.12` with exactly one new public
module:

- `Mathlib.Algebra.Group.ThirdIsomorphism`

Do not split this closure into submodules during materialization, because the
checked-in corpus route is a single theorem-evidence module and its helper
surface is not independently reusable enough to justify a public boundary.

Do not include correspondence modules in `v0.1.12`. They should receive their
own closure audit after the third-isomorphism route is public and downstream
source-free import evidence is available.

The future materialization step must:

- copy and rename `Proofs.Ai.Algebra.AbstractGroupThirdIso` to the public
  `Mathlib.Algebra.Group.ThirdIsomorphism` namespace;
- rewrite imports to the public modules listed in this audit;
- regenerate certificates, package lock, package index, axiom report, and
  release-bundle artifacts;
- update namespace policy allowlists for the single new public module;
- update downstream smoke to consume
  `third_isomorphism_theorem_evidence`;
- run positive package gates, source-free reference verification, hash checks,
  axiom-report checks, index checks, and publish-plan checks;
- run negative package-copy checks for export hash mismatch, certificate hash
  mismatch, certificate decode failure, and stale package lock/version; and
- publish only after `v0.1.11` release evidence is reviewed.
