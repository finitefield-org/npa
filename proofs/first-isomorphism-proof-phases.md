# First Isomorphism Proof Phases

This plan tracks the AI-facing route to the group first isomorphism theorem. Source, replay, and
metadata stay non-trusted sidecars; each completed layer is accepted only through its canonical
certificate and the certificate verifier.

Target theorem shape:

```text
first_isomorphism:
  for a group homomorphism f : G -> H,
  G / ker f is isomorphic to im f
```

For the AI proof corpus, the route is intentionally split into small certificates. The early layers
use explicit law packages and predicate-shaped kernels/images before introducing quotient-backed
carriers.

## FI0: Abstract Group And Kernel Relation Base

Module: `Proofs.Ai.Algebra.AbstractGroup`

Status: certificate generated.

Purpose:

- establish explicit `GroupLawArgs` and `GroupHomLawArgs` packages
- expose homomorphism projection theorems for multiplication, identity, and inverse preservation
- define `KernelPred f a := f a = oneH`
- define `KerRel f a b := f a = f b`
- prove `KerRel` is reflexive, symmetric, and transitive at the Prop level

Completed exports:

| Export | Role |
| --- | --- |
| `GroupLawArgs` | explicit group law package for AI theorem statements |
| `GroupHomLawArgs` | explicit homomorphism law package |
| `group_mul_assoc`, `group_one_mul`, `group_mul_one`, `group_inv_mul`, `group_mul_inv` | group law projections |
| `KernelPred` | predicate form of kernel membership |
| `KerRel` | relation used for the quotient route |
| `hom_mul`, `hom_one`, `hom_inv` | homomorphism law projections |
| `kernel_one` | identity lies in the kernel predicate |
| `ker_rel_refl`, `ker_rel_symm`, `ker_rel_trans` | equivalence ingredients for `KerRel` |

## FI1: Kernel Closure Layer

Module: `Proofs.Ai.Algebra.AbstractGroupKernel`

Status: certificate generated.

Goal:

- prove kernel closure under multiplication
- prove kernel closure under inverse
- prove kernel normality in predicate form
- keep proofs over explicit `GroupLawArgs` / `GroupHomLawArgs`

This layer should avoid quotient primitives. It should depend only on `Std.Logic.Eq`,
`Proofs.Ai.EqReasoning`, and `Proofs.Ai.Algebra.AbstractGroup`.

Completed exports:

| Export | Role |
| --- | --- |
| `kernel_mul_closed` | kernel predicate is closed under domain multiplication |
| `kernel_inv_closed` | kernel predicate is closed under domain inverse |
| `kernel_conj_closed` | kernel predicate is closed under conjugation, giving predicate normality |

## FI2: Image Predicate Layer

Module: `Proofs.Ai.Algebra.AbstractGroupImage`

Status: certificate generated.

Goal:

- define Church-encoded image membership
- prove `f a` lies in the image
- prove image closure under multiplication and inverse
- expose the inclusion and witness eliminators needed for the final isomorphism statement

Completed exports:

| Export | Role |
| --- | --- |
| `ImagePred` | Church-encoded predicate saying `y` is equal to some `f a` |
| `image_intro` | any value `f a` lies in the image |
| `image_elim` | eliminator for the Church-encoded image witness |
| `image_one` | homomorphic identity preservation puts `oneH` in the image |
| `image_mul_closed` | image predicate is closed under codomain multiplication |
| `image_inv_closed` | image predicate is closed under codomain inverse |

## FI3: Quotient Compatibility Layer

Module: `Proofs.Ai.Algebra.AbstractGroupQuotient`

Status: certificate generated.

Goal:

- build the `KerRel` quotient carrier with `quotient_v1`
- define the canonical map from quotient representatives to `H` using `Quotient.lift`
- prove the computation lemma on quotient representatives
- prove multiplication compatibility for representatives

Completed exports:

| Export | Role |
| --- | --- |
| `KerSetoid` | `KerRel` packaged as the quotient setoid |
| `KerQuot` | quotient carrier `G / KerRel f` backed by `quotient_v1` |
| `KerQuotMk` | representative injection into the quotient carrier |
| `KerQuotToH` | canonical map from the quotient carrier to `H` by non-dependent quotient lift |
| `ker_quot_sound` | equivalent representatives are equal in the quotient |
| `ker_quot_to_h_mk` | canonical map computes to `f a` on representatives |
| `ker_quot_to_h_mul_mk` | canonical map is multiplication-compatible on representatives |

Current blocker for the full statement: `quotient_v1` provides non-dependent `Quotient.lift`, but
not a quotient induction principle over arbitrary quotient elements. The full `G / ker f` theorem
will need either a dependent quotient eliminator or a final theorem statement restricted to
representatives.

## FI4: First Isomorphism MVP

Module: `Proofs.Ai.Algebra.AbstractGroupFirstIso`

Status: certificate generated.

Goal:

- prove the representative-level form:

```text
phi (Quotient.mk a) = f a
phi is a homomorphism on representative multiplication
phi is injective on representatives up to KerRel
every f a is in the image
```

This is the strongest version expected to be practical with only `quotient_v1`.

Completed exports:

| Export | Role |
| --- | --- |
| `FirstIsoRepMvp` | Church-encoded bundle of representative-level first-isomorphism data |
| `first_iso_phi_mk` | canonical quotient map computes to `f a` on representatives |
| `first_iso_phi_mul_mk` | canonical quotient map preserves multiplication on representatives |
| `first_iso_rep_injective` | equality of representative images gives `KerRel f a b` |
| `first_iso_rep_hits_image` | every representative image lies in the Church-encoded image predicate |
| `first_isomorphism_rep_mvp` | bundled representative-level first-isomorphism MVP theorem |

## FI5: Full First Isomorphism

Goal:

- add or adopt quotient induction support with the same trusted-boundary treatment as
  `quotient_v1`
- define a quotient group carrier and image group carrier
- prove a bundled `GroupIso (G / ker f) (im f)` theorem

### FI5a: Quotient Multiplication Compatibility

Module: `Proofs.Ai.Algebra.AbstractGroupQuotientMul`

Status: certificate generated.

Purpose:

- define the representative multiplication target `KerQuotMulRep a b := [a * b]`
- prove that replacing both representatives by `KerRel`-equivalent representatives gives the same
  quotient element
- isolate the exact compatibility proof needed by a future binary quotient-lift or quotient group
  operation primitive

Completed exports:

| Export | Role |
| --- | --- |
| `KerQuotMulRep` | representative-level multiplication into the kernel quotient |
| `ker_quot_mul_rep_compat` | well-definedness of representative multiplication under `KerRel` on both arguments |

### FI5b: Quotient-Level Group Operations And Laws

Module: `Proofs.Ai.Algebra.AbstractGroupQuotientGroup`

Status: certificate generated.

Purpose:

- introduce `quotient_v2` with `Quotient.lift2` as a conservative opt-in extension of
  `quotient_v1`
- define multiplication on arbitrary kernel quotient elements using the FI5a compatibility proof
- define the quotient identity and inverse operations
- use `Quotient.indProp` to prove associativity, identity, and inverse laws for arbitrary quotient
  elements
- prove representative computation rules for quotient multiplication and inverse

Completed exports:

| Export | Role |
| --- | --- |
| `KerQuotMul` | binary multiplication on the kernel quotient carrier |
| `KerQuotOne` | identity element on the kernel quotient carrier |
| `KerQuotInv` | inverse operation on the kernel quotient carrier |
| `ker_quot_mul_mk` | `KerQuotMul [a] [b] = [a * b]` by the `Quotient.lift2` computation rule |
| `ker_quot_inv_mk` | `KerQuotInv [a] = [a⁻¹]` by the `Quotient.lift` computation rule |
| `ker_quot_mul_assoc` | quotient multiplication is associative for arbitrary quotient elements |
| `ker_quot_one_mul`, `ker_quot_mul_one` | quotient identity laws for arbitrary quotient elements |
| `ker_quot_inv_mul`, `ker_quot_mul_inv` | quotient inverse laws for arbitrary quotient elements |

### FI5c: Quotient Homomorphism On Arbitrary Quotient Elements

Module: `Proofs.Ai.Algebra.AbstractGroupQuotientHom`

Status: certificate generated.

Purpose:

- use `quotient_v3` / `Quotient.indProp` for proposition-valued quotient induction
- use quotient induction twice to lift the representative multiplication compatibility theorem to
  arbitrary quotient elements
- prove the canonical quotient map preserves quotient-level multiplication

Completed exports:

| Export | Role |
| --- | --- |
| `ker_quot_to_h_mul` | for arbitrary quotient elements `q1 q2`, `phi (q1 * q2) = phi q1 * phi q2` |

### FI5d: Quotient-To-Image Isomorphism Facts

Module: `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull`

Status: certificate generated.

Purpose:

- lift representative-level injectivity to arbitrary quotient elements by quotient induction
- prove the canonical map sends every quotient element into the Church-encoded image predicate
- prove every element of the image predicate is hit by some quotient element
- package the map facts as a continuation theorem that downstream final statements can consume

Completed exports:

| Export | Role |
| --- | --- |
| `first_iso_phi_mul` | first-isomorphism named alias for arbitrary quotient-level homomorphism |
| `first_iso_phi_injective` | if `phi q1 = phi q2`, then `q1 = q2` in the quotient |
| `first_iso_phi_hits_image` | every quotient element maps into `ImagePred f` |
| `first_iso_phi_surj_image` | every `ImagePred f y` is represented by some quotient element |
| `first_isomorphism_image_facts` | continuation bundle for homomorphism, injectivity, image membership, and image-surjectivity |

### FI5e: Quotient-To-Image Bundle

Module: `Proofs.Ai.Algebra.AbstractGroupFirstIsoImage`

Status: certificate generated.

Purpose:

- prove the Church-encoded image predicate contains `oneH` and is closed under `mulH` and `invH`
- package each quotient group law as a small inductive evidence token, then collect those tokens
  into quotient-group evidence without rebuilding a compact `GroupLawArgs` term
- package image closure evidence together with the quotient-to-image homomorphism, injectivity,
  image membership, and image-surjectivity facts
- expose a single certificate-backed first-isomorphism evidence theorem over the Church-encoded
  image predicate

Completed exports:

| Export | Role |
| --- | --- |
| `FirstIsoQuotientAssocEvidence`, `FirstIsoQuotientOneMulEvidence`, `FirstIsoQuotientMulOneEvidence`, `FirstIsoQuotientInvMulEvidence`, `FirstIsoQuotientMulInvEvidence` | inductive evidence tokens for the five quotient group laws |
| `FirstIsoQuotientGroupEvidence` | grouped evidence that the kernel quotient has the certified group laws |
| `FirstIsoImageGroupEvidence` | grouped evidence that the Church-encoded image predicate has the identity and closure facts |
| `FirstIsoTheoremEvidence` | final evidence package for quotient group evidence, image group evidence, homomorphism, injectivity, image membership, and image-surjectivity |
| `FirstIsoImageGroupFacts` | continuation predicate for image identity, multiplication closure, and inverse closure |
| `FirstIsoImage` | continuation predicate for image closure plus the canonical quotient-to-image map facts |
| `first_iso_quotient_group_evidence` | certified quotient group evidence bundle |
| `first_iso_image_group_evidence` | certified image group evidence bundle |
| `first_iso_image_group_facts` | certified image closure bundle |
| `first_isomorphism_to_image` | certified quotient-to-image first-isomorphism bundle over the Church-encoded image predicate |
| `first_isomorphism_theorem_evidence` | certified AI-route first-isomorphism theorem evidence |

No remaining blocker for the AI-route first-isomorphism evidence theorem. A future ergonomic layer
could expose a native `GroupIso` or subtype-style image-carrier statement if the core grows those
abstractions, but the current route proves the theorem by certified inductive evidence and the
Church-encoded image predicate without expanding the trusted base.

Completion evidence:

- generated `.npcert` artifacts for every module in the route
- `tools/proof-corpus` manifest entries for those modules
- `cargo test -p npa-proof-corpus`
- a quotient-capable checker test for FI3+ modules
- full workspace `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace`
