# Third Isomorphism Proof Phases

This plan tracks the AI-facing route to the group third isomorphism theorem. As with the first and
second isomorphism routes, source, replay, and metadata are non-trusted sidecars; completed layers
are accepted only through canonical certificates and the certificate verifier.

Target theorem shape:

```text
third_isomorphism:
  for normal subgroups N <= H of G,
  (G / N) / (H / N) is isomorphic to G / H
```

The current corpus uses predicate-shaped subgroups and evidence packages rather than native
subtype quotient carriers or record-shaped isomorphisms. The implemented AI-facing route builds the
standard quotient map `G/N -> G/H`, proves its representative-level operation compatibility,
surjectivity, representative-kernel soundness for the `H/N` predicate, and decomposed closure facts
showing that `H/N` behaves as a normal predicate inside `G/N`.

## TI0: Normal Quotient Reuse

Modules reused:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`

Status: certificate generated in the existing second-isomorphism route.

Purpose:

- provide normal-relation predicates and quotient carriers
- provide quotient multiplication, identity, inverse, and group-law facts
- keep quotient primitives behind the existing `CoreFeature` gates

## TI1: H Over N Predicate

Module: `Proofs.Ai.Algebra.AbstractGroupThirdIso`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `ThirdIsoGN`, `ThirdIsoGNOne`, `ThirdIsoGNMul`, `ThirdIsoGNInv` | aliases for the quotient carrier `G/N` and its quotient-level operations |
| `ThirdIsoHNPred` | Church-encoded predicate for elements of `G/N` represented by an element of `H` |
| `third_iso_hn_intro`, `third_iso_hn_elim` | introduction and elimination rules for the `H/N` predicate |
| `ThirdIsoKernelPred` | representative-kernel predicate, currently the `H/N` predicate on `G/N` |

## TI1.5: H/N Closure In G/N

Module: `Proofs.Ai.Algebra.AbstractGroupThirdIso`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `third_iso_hn_one` | quotient identity of `G/N` is represented by an element of `H` |
| `third_iso_hn_mul_closed` | `H/N` predicate is closed under quotient multiplication |
| `third_iso_hn_inv_closed` | `H/N` predicate is closed under quotient inverse |
| `third_iso_hn_conj_closed` | `H/N` predicate is closed under conjugation by arbitrary elements of `G/N` |
| `ThirdIsoHNSubgroupLawArgs` | target type alias for a bundled subgroup-law object for `H/N` inside `G/N` |
| `ThirdIsoHNNormalSubgroupLawArgs` | target type alias for a bundled normal-subgroup-law object for `H/N` inside `G/N` |

## TI2: Canonical Map G/N To G/H

Module: `Proofs.Ai.Algebra.AbstractGroupThirdIso`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `third_iso_rel_lift` | relation lift from `N` to `H` using the assumption `N <= H` |
| `ThirdIsoPhi` | canonical quotient map `G/N -> G/H` |
| `ThirdIsoPhiKernelQuot` | kernel-relation quotient of `G/N` induced by `ThirdIsoPhi` |
| `third_iso_phi_mk` | representative computation for `ThirdIsoPhi` |
| `third_iso_phi_mul` | representative-level multiplication compatibility |
| `third_iso_phi_one` | identity compatibility |
| `third_iso_phi_inv` | representative-level inverse compatibility |
| `third_iso_phi_surjective` | quotient-induction proof that every element of `G/H` is hit |

## TI3: Kernel Evidence

Module: `Proofs.Ai.Algebra.AbstractGroupThirdIso`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `third_iso_hn_to_kernel_sound` | `H/N` predicate membership maps to the identity in `G/H` |
| `third_iso_kernel_intro` | representatives from `H` introduce kernel-predicate membership in `G/N` |
| `ThirdIsoKernelEvidence` | packaged kernel soundness and representative introduction |
| `third_iso_kernel_evidence` | certificate-backed kernel evidence package |

## TI4: Final AI-Facing Evidence

Module: `Proofs.Ai.Algebra.AbstractGroupThirdIso`

Status: certificate generated.

Completed exports:

| Export | Role |
| --- | --- |
| `ThirdIsoTheoremEvidence` | final AI-route evidence target, packaging relation lifting, surjectivity, and kernel evidence |
| `third_isomorphism_theorem_evidence` | certificate-backed third-isomorphism evidence theorem |

Scope note: this is the current AI-facing theorem shape for the third-isomorphism route. It exposes
normality of `H/N` as decomposed closure lemmas and now names both the bundled-law target types and
the kernel-relation quotient induced by `ThirdIsoPhi`. It does not yet prove a bundled
`NormalSubgroupLawArgs` object over `G/N`, introduce the native quotient-of-quotient carrier
`(G/N)/(H/N)`, or provide a record-shaped isomorphism object.

## Completion Evidence

The current TI0-TI4 route is complete when these checks pass:

- generated `.npcert` artifacts for `Proofs.Ai.Algebra.AbstractGroupThirdIso`
- `tools/proof-corpus` manifest entry for the module
- `cargo run -p npa-proof-corpus`
- `cargo test -p npa-proof-corpus`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
