# npa-mathlib Layer 3D-G Correspondence Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the Layer 3D-F
third-isomorphism materialization. It selects the abstract-group correspondence
route, including the order-aware final evidence, as the next coherent closure.
It keeps ring isomorphism, CRT, higher algebra, geometry, and analysis routes
out of this layer.

## Baseline

Current package state:

- Layer 3D-F has been materialized, committed, tagged, and pushed in
  `/Users/kazuyoshitoshiya/ff/npa-mathlib` as `npa-mathlib v0.1.12`.
- The `npa-mathlib` release commit is
  `1474648969337fa077a9fe1cd2732f54232bb489`.
- The annotated `v0.1.12` tag object observed from `origin` is
  `c751a1dd1f36b556abdd03e5ba6dd5cb9ebb5d07`.
- The local ignored `v0.1.12` release-bundle tar hash is
  `sha256:dcd57bdb6c0b711462df4d715a20df2f08c70720edd3e5dfd8f5dbff523acec7`.
- The `v0.1.12` publish-plan hash is
  `sha256:6bcadbaed70595cb497e881da010d7c093772e76c503a7532707fc7a73d37cd0`.
- The current package check passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
```

- The current source-free reference verifier passed with 34 modules:

```sh
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
```

Layer 3D-G should materialize as `npa-mathlib v0.1.13`. It must not change
package boundaries, registry assumptions, import identity rules, or proof trust
boundaries.

The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

## Selected Candidate Set

The selected Layer 3D-G candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | `Mathlib.Algebra.Group.Correspondence.Basic` | `Mathlib/Algebra/Group/Correspondence/Basic/` | 8 definitions, 18 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder` | `Mathlib.Algebra.Group.Correspondence.Order` | `Mathlib/Algebra/Group/Correspondence/Order/` | 4 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` | `Mathlib.Algebra.Group.Correspondence` | `Mathlib/Algebra/Group/Correspondence/` | 6 inductives, 8 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal` | `Mathlib.Algebra.Group.Correspondence.Ordered` | `Mathlib/Algebra/Group/Correspondence/Ordered/` | 1 definition, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupCorrespondence`, `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder`, `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` | `Eq.rec` allowed/transitive |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | `Mathlib.Algebra.Group.Correspondence.Basic` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder` | `Mathlib.Algebra.Group.Correspondence.Order` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` | `Mathlib.Algebra.Group.Correspondence` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal` | `Mathlib.Algebra.Group.Correspondence.Ordered` |

This deliberately reserves the root
`Mathlib.Algebra.Group.Correspondence` module for the final
`correspondence_theorem_evidence` route. The foundational predicates and
subgroup/preimage/image lemmas live under `Correspondence.Basic`, while the
monotonicity/respects-equivalence facts live under `Correspondence.Order`.
`Correspondence.Ordered` is the order-aware final evidence module.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Correspondence.Basic
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group

Mathlib.Algebra.Group.Correspondence.Order
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Subgroup.Order
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.Correspondence.Basic

Mathlib.Algebra.Group.Correspondence
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.Correspondence.Basic

Mathlib.Algebra.Group.Correspondence.Ordered
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Subgroup.Order
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.Correspondence.Basic
  imports Mathlib.Algebra.Group.Correspondence.Order
  imports Mathlib.Algebra.Group.Correspondence
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`,
`Mathlib.Algebra.Group.Subgroup`, `Mathlib.Algebra.Group.Subgroup.Order`, and
`Mathlib.Algebra.Group.Quotient.*` remain local public `npa-mathlib` modules
carried forward unchanged from the released baseline. The Layer 3D-F
`Mathlib.Algebra.Group.ThirdIsomorphism` module remains part of the package
baseline, but this correspondence closure does not import it directly.

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.Correspondence.Basic` definitions:
  - `CorrespondenceImagePred`
  - `CorrespondencePreimagePred`
  - `CorrespondenceSaturationPred`
  - `CorrespondenceImageSubgroupMk`
  - `CorrespondencePreimageSubgroupMk`
  - `CorrespondenceImageSubgroupLawArgs`
  - `CorrespondencePreimageSubgroupLawArgs`
  - `CorrespondenceTheoremMk`
- `Mathlib.Algebra.Group.Correspondence.Basic` theorems:
  - `correspondence_group_mul_inv_left_reassoc`
  - `correspondence_subgroup_saturates`
  - `correspondence_image_intro`
  - `correspondence_saturation_intro`
  - `correspondence_saturation_elim`
  - `correspondence_image_elim`
  - `correspondence_image_one`
  - `correspondence_image_mul_closed`
  - `correspondence_image_inv_closed`
  - `correspondence_preimage_one`
  - `correspondence_preimage_mul_closed`
  - `correspondence_preimage_inv_closed`
  - `correspondence_preimage_contains_normal`
  - `correspondence_subgroup_to_preimage_image`
  - `correspondence_subgroup_to_saturation`
  - `correspondence_saturation_to_subgroup`
  - `correspondence_quotient_to_image_preimage`
  - `correspondence_image_preimage_to_quotient`
- `Mathlib.Algebra.Group.Correspondence.Order` theorems:
  - `correspondence_image_mono`
  - `correspondence_preimage_mono`
  - `correspondence_image_respects_equiv`
  - `correspondence_preimage_respects_equiv`
- `Mathlib.Algebra.Group.Correspondence` inductives:
  - `CorrespondenceImageSubgroupEvidence`
  - `CorrespondencePreimageSubgroupEvidence`
  - `CorrespondenceContainmentEvidence`
  - `CorrespondenceSubgroupSaturationEvidence`
  - `CorrespondenceQuotientRoundTripEvidence`
  - `CorrespondenceTheoremEvidence`
- `Mathlib.Algebra.Group.Correspondence` theorems:
  - `correspondence_image_subgroup_law_args`
  - `correspondence_preimage_subgroup_law_args`
  - `correspondence_image_subgroup_evidence`
  - `correspondence_preimage_subgroup_evidence`
  - `correspondence_containment_evidence`
  - `correspondence_subgroup_saturation_evidence`
  - `correspondence_quotient_round_trip_evidence`
  - `correspondence_theorem_evidence`
- `Mathlib.Algebra.Group.Correspondence.Ordered` definition:
  - `CorrespondenceOrderEvidence`
- `Mathlib.Algebra.Group.Correspondence.Ordered` theorem:
  - `correspondence_order_evidence`

Downstream smoke evidence for the materialized release should import
release-bundle certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`
- `Mathlib.Algebra.Group.Subgroup.Order`
- `Mathlib.Algebra.Group.Quotient`
- `Mathlib.Algebra.Group.Quotient.Mul`
- `Mathlib.Algebra.Group.Quotient.Group`
- `Mathlib.Algebra.Group.Correspondence.Basic`
- `Mathlib.Algebra.Group.Correspondence.Order`
- `Mathlib.Algebra.Group.Correspondence`
- `Mathlib.Algebra.Group.Correspondence.Ordered`

The downstream smoke fixture should consume both
`correspondence_theorem_evidence` and `correspondence_order_evidence`, not only
a helper lemma, so the smoke fixture proves the root correspondence route and
the order-aware final evidence route are source-free importable.

## Deferred Candidates

The following nearby routes are intentionally not selected for Layer 3D-G:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Provisional first-isomorphism MVP | `Proofs.Ai.Algebra.AbstractGroupFirstIso` | This corpus module exposes the provisional `FirstIsoRepMvp` surface and remains superseded by the already public Layer 3D-C `FirstIsomorphism` modules. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`, `Proofs.Ai.Algebra.AbstractRingFirstIso`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | These depend on ring API decisions and should receive a separate algebra-layer namespace audit. |
| Higher algebra seeds | `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`, `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`, `Proofs.Ai.Algebra.AbstractKrullTheorem`, `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization` | Their public namespaces and algebraic prerequisites are broader than the group correspondence closure. |
| Geometry and analysis routes | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Analysis.*`, `Proofs.Ai.LinearAlgebra.*`, `Proofs.Ai.FunctionalAnalysis.*` | Not part of the group isomorphism and correspondence closure sequence. |

## Axiom Policy

Layer 3D-G does not widen the public axiom policy beyond the `v0.1.12`
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

The generated corpus axiom report records for the selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The generated theorem index records 31 theorem entries for the selected
modules:

| Corpus module | Theorem entries | Entries with axiom dependencies |
| --- | ---: | ---: |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | 18 | 16 |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder` | 4 | 4 |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` | 8 | 8 |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal` | 1 | 1 |

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondence` | `sha256:758957e638d539ddfa75b5a7f782320e9a7d2513884e6368c3bc4ab00306fbea` | `sha256:38b0571477e65f5d4b71190daaea5f794f21839ae5cb94f522ddd0d1e6c70d9d` | `sha256:c5f000c544037c766530531411fa4230faaada5a0d2840ef67411d2d73e76a7f` | `sha256:a1c7dff2e43196d9ed0f707b49ddcb949d8c3f3c7b3a85369478372380820393` | `sha256:e27522869a6949215187b97c6a5acc116e71ab2ea85fd7641012d19678f381aa` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder` | `sha256:a05567d63e7bd6ae522705820c802c8f91044bb1425cae17a93c6181cb629bc7` | `sha256:9a4023430ebad6aa957180468d003d2d16d5a3d068011d30b46860dc15db956b` | `sha256:e8d7668d809035879ec4db3de3235fdbb5ebb719c6b5a888e30867c9c55cdef9` | `sha256:930e872459fa0a237548e58d011ab133dca95a0d4dce5979f68cb3fe009ce914` | `sha256:df7650170f8b8663b7808db663774eb727427993c16fe0ef89006b4cab5e8da5` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` | `sha256:3008db140aa8c1ba7b65432af7580f3b1087a25a20bd9adb81080494d844aee9` | `sha256:e6d5c5701e054867cffe2056a1c09711e6cd0931c3f5d96ddb466356344d1595` | `sha256:4f3d5aa76e66dde59ebe00f2c472fcab6e48cebef59f2c5641ae17622ec2108c` | `sha256:f5d781dcd1af03f4c02e2c247ae1bcca0de8eb1b5bfae9b9b3803bc323f2b936` | `sha256:98ba022c24a75d18b2ec35fd2ef876bf09dd5449167f298a6751c2b10d2f9405` |
| `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal` | `sha256:2a23e1f4d24fe4c166e8fe24e268b74367e23af5927a7f4116aa60909e6756bd` | `sha256:ba113ebf540d0ecd38733d6222a2d63b24d322bdaf3a44e426271225c95c16e7` | `sha256:edd3c2b48c760ef1388779f6cd04f0b001a0d176ebf0b1cf52910fe558482054` | `sha256:48fad99b017e564eae3a94ad24d12f5a3572af147ee049749faf426445d75ddf` | `sha256:40a0d254961e9110f7ba3462d544e168a330c810a79f8b971ef649cb33ee2268` |

The local `npa-mathlib v0.1.12` materialization records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |
| `Mathlib.Algebra.Group.Subgroup` | `sha256:2b527c5b3039075812d564522facb4a41e421e1902f863df111f0e83c5b9d85c` | `sha256:b3b47c9d314d69594d1c80e24fd4d1954c242ef289e2031207be9acd798ce2a9` |
| `Mathlib.Algebra.Group.Subgroup.Order` | `sha256:e437fa6d0d71c25cfd931ece5572e3f232a08d7c9c8ece7c3ebf1cd4cf0beee6` | `sha256:c501393a7f67b539b33378767cd9a3f89205a604c902036c001aa1d6f4dd84f5` |
| `Mathlib.Algebra.Group.Quotient` | `sha256:14d52188e958a89a9f87ad32a12c2738a73c0e185d68bcd390875bf4dea7f4de` | `sha256:6fc7c518e7eaa04e792bb1d4dd3af7b4f5c4987136a118ce98a75d601c4feb19` |
| `Mathlib.Algebra.Group.Quotient.Mul` | `sha256:9570dfe4c8b6ca61fe53acd2bf41cf193be20a4d9118cbc295f1c1a10d602cc7` | `sha256:5495b3aba0eb579efd418b5a86fad16765c2f21889c9531747fa0d152f774013` |
| `Mathlib.Algebra.Group.Quotient.Group` | `sha256:fb6bb1472ee946b6db2174ccf4cf845b0a7dda6c8c42ccc9396950a5a8a92654` | `sha256:41a716e2220952290f35148b55d9505f1c8f7b90491233c10ee704bb5b678ef1` |

## Verification

The checked-in corpus certificates passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupCorrespondence
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupCorrespondence`: verified 1 selected
  module, 8 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder`: verified 1 selected
  module, 10 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`: verified 1 selected
  module, 9 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal`: verified 1
  selected module, 12 modules including dependency cache.

The source-to-certificate authoring path also regenerated the closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupCorrespondence
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupCorrespondence`: built 7 modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder`: built 9 modules
  including import closure.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`: built 8 modules
  including import closure.
- `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal`: built 11 modules
  including import closure.

The build checks rewrote `proofs/generated/ai-theorem-index.json`, but
`git diff -- proofs/generated/ai-theorem-index.json` reported no tracked diff.

## Decision

Materialize Layer 3D-G as `npa-mathlib v0.1.13` with exactly these new public
modules:

```text
Mathlib.Algebra.Group.Correspondence.Basic
Mathlib.Algebra.Group.Correspondence.Order
Mathlib.Algebra.Group.Correspondence
Mathlib.Algebra.Group.Correspondence.Ordered
```

Do not publish only the base correspondence predicates or monotonicity lemmas:
downstream users need the final `correspondence_theorem_evidence` route and the
order-aware `correspondence_order_evidence` route to consume the closure
source-free.

Do not include ring first isomorphism, ring CRT, or higher algebra theorem
seeds in `v0.1.13`. They should receive separate namespace and dependency
audits after the group correspondence layer is public.

The future materialization step must:

- copy and rename the selected `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`
  modules to the public `Mathlib.Algebra.Group.Correspondence*` namespaces;
- rewrite imports to the public modules listed in this audit;
- regenerate certificates, package lock, package index, axiom report, publish
  plan, and release-bundle artifacts;
- update namespace policy and README allowlists for the four new public
  modules;
- update downstream smoke to consume `correspondence_theorem_evidence` and
  `correspondence_order_evidence`;
- run positive package gates, source-free reference verification, hash checks,
  axiom-report checks, index checks, and publish-plan checks; and
- run negative package-copy checks for export hash mismatch, certificate hash
  mismatch, certificate decode failure, and stale package lock/version.

## Materialization Result

Layer 3D-G was materialized in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
package version `0.1.13` with the four selected public modules. The package
manifest, package lock, axiom report, theorem index, publish plan, namespace
policy, README, and downstream smoke fixture were updated.

The materialized public module hashes are:

| Public module | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Group.Correspondence.Basic` | `sha256:dace40eccfee78cce71f880507e76ddb8a5a11d4872b070e7abbea999bb0544c` | `sha256:6a457720e4aef4102591fab03c2d50b9748b06bff9e7fe635dc8e30ae63b1c9a` | `sha256:a1c7dff2e43196d9ed0f707b49ddcb949d8c3f3c7b3a85369478372380820393` | `sha256:17d611fd3212bf953d900f1acbac0ae2403eecdd436e6116c6257238b43e1ec5` |
| `Mathlib.Algebra.Group.Correspondence.Order` | `sha256:b24d4155ae756178e124c6d600821947a3f05a53b2e69106157ee35516f44bb1` | `sha256:343f663b129a1ecdfddc751614f5471b9256fb48567bd600e8ea7300b737fda8` | `sha256:930e872459fa0a237548e58d011ab133dca95a0d4dce5979f68cb3fe009ce914` | `sha256:aaeddce85ffe318f8ce637974e0011ad562148c5342fd28fee7140e80b19f20e` |
| `Mathlib.Algebra.Group.Correspondence` | `sha256:0e4cd4a9b306a3faa97f56ba2cb034fedb0c12bd03151b78159122c0e887a2d2` | `sha256:53fd29e96f6e6f11234b9771a415d02b5958826424f22b99261710fe4d3051ea` | `sha256:f5d781dcd1af03f4c02e2c247ae1bcca0de8eb1b5bfae9b9b3803bc323f2b936` | `sha256:d670ec4a6223d1002a5ebad83f804b4835a67bda7f4f5055ea5cc99172a09776` |
| `Mathlib.Algebra.Group.Correspondence.Ordered` | `sha256:e7095005a7709c8107ba2d3bf813392201597eea592aa0fa8da2a039a0c21af7` | `sha256:c265cf2dd797fee3682500227e12667debe58d472406723f89ceb8c47f01801b` | `sha256:48fad99b017e564eae3a94ad24d12f5a3572af147ee049749faf426445d75ddf` | `sha256:38cf43f485f7480e9674983644dd3d5572f64c911be1c569c116215dea9223ad` |

The downstream smoke fixture now checks both public evidence routes:

```text
Downstream.GroupCorrespondence::correspondence_theorem_evidence_passthrough
Downstream.GroupCorrespondence::correspondence_order_evidence_passthrough
```

The downstream local module hashes are:

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.GroupCorrespondence` | `sha256:06339f5e9149a25fa701b13a800f31957218c757fa3a38a7d568235097847923` | `sha256:3bce2b85ae2de5fcd64ec8c64cc6ab14ed8192bca6a81efaed86f7764598996c` | `sha256:915791bb3488a57fd02b2fa3e55ed83955930bfc1fbd0f57dc85406729c2e842` | `sha256:62a2039f05232dfc414b4bcafacadbe41a666c398485458f0767efc9f600242b` | `sha256:2fdfc7501868c7405bb6476dfe94ffad5ae57075d74249eb49fc7437ad120002` |

The release artifact was generated locally:

```text
target/release-artifacts/npa-mathlib-v0.1.13-release-artifacts.tar.gz
sha256:88d1ef15907dc65f19c175cb2eabd69168355c8f236218f0e6b498e11737e0b9
```

The generated publish-plan hash is:

```text
sha256:3dd2278931e045c1573ba8c7b3f06783a39fad65f741320a39bc592b7cb10e35
```

Positive verification passed:

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
```

Negative package-copy checks rejected the expected failure modes:

- `export_hash_mismatch`
- `certificate_hash_mismatch`
- `certificate_decode_failed`
- `package_lock_stale`
