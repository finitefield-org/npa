# Correspondence Theorem Proof Phases

This plan tracks the AI-facing route to the group correspondence theorem. The trusted boundary is
unchanged: source, replay, and metadata are producer sidecars, while acceptance depends on the
canonical certificate and verifier.

Target theorem shape:

```text
correspondence_theorem:
  for a normal subgroup N of G,
  subgroups H of G with N <= H correspond to subgroups K of G / N
```

The current corpus represents subgroups by predicates and law-package evidence. Instead of adding a
native bijection, subtype carrier, or quotient-exactness primitive to the kernel, this route exports
the two standard operations, the quotient-side round trip, and the subgroup-side `NormalRel`
saturation equivalence needed for the reverse direction.

## CT0: Normal Quotient Reuse

Modules reused:

- `Proofs.Ai.Algebra.AbstractGroup`
- `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`

Status: certificate generated in earlier isomorphism routes.

Purpose:

- provide subgroup and normal-subgroup law packages
- provide the normal relation `NormalRel`
- provide the quotient carrier `NormalQuot` and quotient operations
- provide quotient soundness from normal-relation evidence

## CT1: Image And Preimage Predicates

Modules: `Proofs.Ai.Algebra.AbstractGroupCorrespondence` and
`Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `CorrespondenceImagePred` | quotient predicate for elements represented by some `h` satisfying `Hpred h` |
| `CorrespondencePreimagePred` | predicate on `G` obtained by pulling a quotient predicate back along `NormalQuotMk` |
| `CorrespondenceSaturationPred` | relation-level predicate for elements `NormalRel`-equivalent to some element of `H` |
| `correspondence_image_intro` | representative-level introduction for quotient image membership |
| `correspondence_image_elim` | representative-level elimination for quotient image membership |
| `correspondence_saturation_intro` | introduction for subgroup-side saturation |
| `correspondence_saturation_elim` | elimination for subgroup-side saturation |

`CorrespondenceImagePred` follows the existing third-isomorphism route: image membership means
equality in `G/N` to the class of an element of `H`. The subgroup-side reverse direction is exposed
as saturation by `NormalRel`, which avoids adding quotient exactness to the trusted kernel boundary.

## CT2: Closure Facts

Module: `Proofs.Ai.Algebra.AbstractGroupCorrespondence`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `correspondence_image_one` | the image predicate contains the quotient identity |
| `correspondence_image_mul_closed` | the image predicate is closed under quotient multiplication |
| `correspondence_image_inv_closed` | the image predicate is closed under quotient inverse |
| `correspondence_preimage_one` | a quotient subgroup preimage contains the group identity |
| `correspondence_preimage_mul_closed` | a quotient subgroup preimage is closed under multiplication |
| `correspondence_preimage_inv_closed` | a quotient subgroup preimage is closed under inverse |
| `correspondence_preimage_contains_normal` | every preimage of a quotient subgroup contains `N` |

## CT3: Round-Trip Conversions

Module: `Proofs.Ai.Algebra.AbstractGroupCorrespondence`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `correspondence_group_mul_inv_left_reassoc` | group simplification lemma for saturation |
| `correspondence_subgroup_saturates` | if `N <= H`, then `H` is closed under `NormalRel`-equivalent representatives |
| `correspondence_subgroup_to_preimage_image` | any `x : H` lies in the preimage of the image of `H` |
| `correspondence_subgroup_to_saturation` | any `x : H` lies in the `NormalRel` saturation of `H` |
| `correspondence_saturation_to_subgroup` | any element in the `NormalRel` saturation of `H` lies in `H` when `N <= H` |
| `correspondence_quotient_to_image_preimage` | any `q : K` lies in the image of the preimage of `K` |
| `correspondence_image_preimage_to_quotient` | any image-of-preimage element lies in `K` |

The subgroup-side reverse direction uses saturation: a witness `h : H` with `h ~ x` gives
`h * (h^-1 * x) = x`, and `h^-1 * x` lies in `N`, hence in `H`.

## CT4: Final AI-Facing Evidence

Modules: `Proofs.Ai.Algebra.AbstractGroupCorrespondence` and
`Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `CorrespondenceImageSubgroupLawArgs` | target package type in `AbstractGroupCorrespondence` for image subgroup closure in `G/N` |
| `CorrespondencePreimageSubgroupLawArgs` | target package type in `AbstractGroupCorrespondence` for preimage subgroup closure in `G` |
| `CorrespondenceTheoremMk` | Church-style target type in `AbstractGroupCorrespondence` for collecting the checked correspondence components |
| `CorrespondenceTheoremEvidence` | final evidence inductive in `AbstractGroupCorrespondenceFinal` for closure, containment, quotient-side round trip, and subgroup-side saturation equivalence |
| `correspondence_theorem_evidence` | certificate-backed final theorem in `AbstractGroupCorrespondenceFinal` collecting the checked correspondence components |

Scope note: this route proves the correspondence as checked predicate and law-package evidence. It
does not add native subtype carriers, a bundled bijection record, lattice operations, order
preservation, quotient equality reflection, or a kernel-level subgroup object.

## Next Extension Plan

The current CT0-CT4 route is complete, but downstream theorem proving would benefit from a small
API layer over the checked components. These extensions should stay outside the kernel trusted
base: they should be ordinary certificate-backed definitions and theorems in the proof corpus.

### CT5: Predicate Inclusion API

Module: `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `SubgroupLe` | abbreviation for `forall x, H x -> K x` |
| `SubgroupEquiv` | bidirectional predicate inclusion for subgroup predicates |
| `NormalContains` | abbreviation for `forall x, N x -> H x` |
| `subgroup_le_refl` | reflexivity of predicate inclusion |
| `subgroup_le_trans` | transitivity of predicate inclusion |
| `subgroup_equiv_intro` | constructor for bidirectional predicate equivalence |
| `subgroup_equiv_left` | extracts the forward inclusion from predicate equivalence |
| `subgroup_equiv_right` | extracts the reverse inclusion from predicate equivalence |
| `subgroup_equiv_refl` | reflexivity of predicate equivalence |
| `subgroup_equiv_symm` | symmetry of predicate equivalence |
| `subgroup_equiv_trans` | transitivity of predicate equivalence |
| `normal_contains_to_subgroup_le` | views `NormalContains N H` as `SubgroupLe N H` |
| `subgroup_le_to_normal_contains` | views `SubgroupLe N H` as `NormalContains N H` |
| `normal_contains_refl` | reflexivity of `NormalContains` |
| `normal_contains_trans` | transitivity of `NormalContains` |

Reason: the current correspondence and isomorphism routes repeat raw `forall x` inclusion
arguments. Naming these relations will make later order and equivalence theorems shorter and easier
for AI proof search to target.

### CT6: Correspondence Monotonicity

Module: `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrder`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `correspondence_image_mono` | if `H <= H'`, then `image(H) <= image(H')` in `G/N` |
| `correspondence_preimage_mono` | if `K <= K'`, then `preimage(K) <= preimage(K')` in `G` |
| `correspondence_image_respects_equiv` | equivalent subgroup predicates have equivalent images |
| `correspondence_preimage_respects_equiv` | equivalent quotient predicates have equivalent preimages |

Reason: these facts are the next step from membership round trips to an order-theoretic
correspondence. They should be lightweight because they mostly eliminate and reintroduce the
existing Church-encoded image predicate.

### CT7: Direct Law-Package Theorems

Suggested module: `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal` or a follow-up final API
module.

Add direct theorem exports for the law packages already represented by target definitions:

| Export | Role |
| --- | --- |
| `correspondence_image_subgroup_law_args` | constructs `CorrespondenceImageSubgroupLawArgs` directly |
| `correspondence_preimage_subgroup_law_args` | constructs `CorrespondencePreimageSubgroupLawArgs` directly |

Reason: `correspondence_theorem_evidence` is useful as a final certificate-backed package, but
downstream proofs often need the concrete subgroup law package rather than an evidence wrapper.

### CT8: Order Evidence Wrapper

Suggested module: `Proofs.Ai.Algebra.AbstractGroupCorrespondenceOrderFinal`.

Add a final AI-facing evidence wrapper for the order-level form of the theorem:

| Export | Role |
| --- | --- |
| `CorrespondenceOrderEvidence` | packages image/preimage monotonicity, round trips, and saturation |
| `correspondence_order_evidence` | certificate-backed theorem constructing that package |

Reason: this would state the correspondence theorem closer to its standard mathematical form while
still using predicate-level and certificate-first artifacts.

### Boundary Note

Do not add quotient equality reflection merely to state `preimage(image(H)) = H` directly. The
current route intentionally proves the subgroup-side reverse direction through `NormalRel`
saturation. If a later route needs exact quotient-class reflection, it should first document the
trusted-boundary impact and prefer a certificate-backed theorem over a kernel primitive.

## Completion Evidence

The CT0-CT4 route is complete when these checks pass:

- generated `.npcert` artifacts for `Proofs.Ai.Algebra.AbstractGroupCorrespondence`
- generated `.npcert` artifacts for `Proofs.Ai.Algebra.AbstractGroupCorrespondenceFinal`
- `correspondence_theorem_evidence` is exported by that certificate
- `tools/proof-corpus` manifest entry for the module
- `cargo run -p npa-proof-corpus`
- `cargo test -p npa-proof-corpus`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
