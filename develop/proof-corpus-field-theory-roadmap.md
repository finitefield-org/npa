# Proof Corpus Field Theory Roadmap

Date: 2026-06-03

This document is the roadmap for adding field theory theorems to the proof
corpus. It is a planning document, not grounds for proof acceptance. It does
not override the next release order for the full public `npa-mathlib` closure;
it records the local addition order to use when expanding the field-theory
route.

This does not change NPA's trust boundary.

```text
Not trusted:
  this document
  source.npa
  replay.json
  meta.json
  theorem index
  AI-generated proof candidates

Trusted:
  canonical .npcert
  deterministic hash
  kernel / certificate verifier verdict
  source-free independent checker verdict
```

## 1. Current Status

In the existing corpus, the foundations for group theory and ring theory are
already substantial. In particular, the following layers can be used as the
foundation for field theory.

```text
Proofs.Ai.Algebra.AbstractGroup
Proofs.Ai.Algebra.AbstractGroupImage
Proofs.Ai.Algebra.AbstractGroupQuotient
Proofs.Ai.Algebra.AbstractGroupQuotientMul
Proofs.Ai.Algebra.AbstractGroupQuotientGroup
Proofs.Ai.Algebra.AbstractGroupQuotientHom
Proofs.Ai.Algebra.AbstractRing
Proofs.Ai.Algebra.AbstractRingFirstIsoBase
Proofs.Ai.Algebra.AbstractRingFirstIso
Proofs.Ai.Algebra.AbstractRingChineseRemainder
Proofs.Ai.Algebra.AbstractOrderedField
```

Before FT-01, `AbstractOrderedField` was centered on order and square-root law
bundles, and the inverse structure of fields themselves had not been split out
as a common module. At this point, FT-01 through FT-07 have added the
`AbstractField` foundation, field homs, field-to-integral-domain,
field ideal / quotient, and ordered-field bridge as verified staging on the
corpus side. The first half of this document covers the completed foundation
route, and the second half covers the advanced field-theory route built on top
of it.

## 2. Basic Policy

The foundation route did not jump directly to algebraically closed fields, the
Nullstellensatz, or field extensions. Instead, it built a small reusable layer
between `AbstractRing` and `AbstractOrderedField`. The advanced field-theory
route also avoids adding existence theorems as trusted axioms, and instead
adds explicit evidence packages and projection theorems in stages.

Design policy:

- Keep `FieldLawArgs` as an explicit law package.
- Do not add `inv` / `div` / `Nonzero` to the core calculus; treat them as
  ordinary operations and predicates on a carrier.
- Do not make `zero_ne_one` or `a != 0` decidable booleans; represent them as
  Prop-level negation evidence, matching the existing corpus.
- Do not make `div` primitive; first add a projection theorem to
  `mul a (inv b)`.
- Proof corpus authoring may use convenient source / replay files, but
  acceptance is limited to checked certificates.
- Do not immediately rewrite the existing `AbstractOrderedField`
  destructively; add bridges to `AbstractField` while watching the impact on
  downstream modules.

## 3. Foundation Route Priority

### 3.1 AbstractField foundation

Implemented module:

```text
Proofs.Ai.Algebra.AbstractField
```

Implemented public surface:

```text
Nonzero
div
FieldLawArgs
field_ring_laws
field_zero_ne_one
field_inv_mul_cancel
field_mul_inv_cancel
field_div_eq_mul_inv
```

Purpose:

- Add only field-specific laws on top of the `AbstractRing` law package.
- Fix basic projection theorems for inverses and division under names that are
  easy for theorem search to find.
- Allow downstream linear algebra, geometry, analysis, and higher ring-theory
  theorems to explicitly require a field.

### 3.2 Basic field calculation lemmas

Small lemmas useful for theorem search and rewriting have been added.

Implemented theorems:

```text
field_inv_one
field_div_one
field_div_self_nonzero
field_zero_div
field_mul_left_cancel_nonzero
field_mul_right_cancel_nonzero
field_nonzero_mul_closed
field_mul_eq_zero_cases
```

Purpose:

- Let downstream modules use equality transformations involving division and
  inverses without unfolding them from law arguments each time.
- Make closure and cancellation for `Nonzero` searchable as theorems.
- Provide `field_div_self_nonzero` and cancellation in a form usable for
  scalar normalization in linear algebra.

### 3.3 Field homomorphism bridge

A module connecting to the existing `RingHomLawArgs` has been added.

Implemented module:

```text
Proofs.Ai.Algebra.AbstractFieldHom
```

Implemented theorems:

```text
FieldHomLawArgs
field_hom_as_ring_hom
field_hom_inv_of_nonzero
field_hom_div
field_hom_preserves_nonzero
```

Purpose:

- Connect the ring-homomorphism first-isomorphism route to field
  homomorphisms.
- Make inverse preservation and division preservation reusable instead of
  deriving them from multiplicative preservation of ring homomorphisms each
  time.
- Provide scaffolding for downstream field isomorphism, embedding, and
  subfield APIs.

### 3.4 Field as integral domain

A layer that extracts integral-domain behavior from fields has been added.

Implemented module:

```text
Proofs.Ai.Algebra.AbstractFieldIntegralDomain
```

Implemented theorems:

```text
field_no_zero_divisors
field_integral_domain_laws
field_nonzero_product_left
field_nonzero_product_right
field_mul_eq_zero_elim
```

Purpose:

- Allow fields to be passed as integral domains to higher ring-theory layers
  such as `AbstractUfdPrimeFactorization`.
- Share proofs that use the fact that one factor is zero when `a * b = 0`.

### 3.5 Field ideals and quotient bridge

This is a higher layer that connects to ring-theory quotient and ideal
theorems.

Implemented module:

```text
Proofs.Ai.Algebra.AbstractFieldIdeal
```

Implemented theorems:

```text
field_ideal_zero_or_top
field_simple_ring_evidence
quotient_by_maximal_ideal_is_field
```

Purpose:

- Connect prerequisites for `AbstractKrullTheorem` and
  `AbstractHilbertNullstellensatz` to a more natural field-theory API.
- Make the standard route showing that a quotient by a maximal ideal is a
  field reusable in the proof corpus.

## 4. Connection to OrderedField

At present, `AbstractOrderedField` has order, square-root, and square
monotonicity as bundles. The following module has been added as a
compatibility bridge after adding `AbstractField`.

Implemented module:

```text
Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge
```

Implemented theorems:

```text
ordered_field_field_laws
ordered_field_nonzero_of_positive
ordered_field_inv_positive
ordered_field_div_positive
ordered_field_mul_pos
ordered_field_sq_pos_of_nonzero
```

The existing `OrderedFieldLawArgs` has not been removed or replaced wholesale.
The split bridge module adds projection theorems between `FieldLawArgs` and
order/sqrt laws, localizing the impact on certificate / export hashes for
existing `AbstractOrderedField` consumers.

## 5. Foundation Route Implementation Units

The implementation units have been completed in this order.

1. `Proofs.Ai.Algebra.AbstractField`
2. `Proofs.Ai.Algebra.AbstractFieldHom`
3. `Proofs.Ai.Algebra.AbstractFieldIntegralDomain`
4. `Proofs.Ai.Algebra.AbstractFieldIdeal`
5. `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`

Each module is kept small, with imports minimized. The initial `AbstractField`
starts only from `AbstractRing` and `Std.Logic.Eq`; dependencies on group
quotients, ring quotients, and CRT are confined to downstream bridge modules.

## 6. npa-mathlib materialization policy

This field-theory route is complete as verified staging on the corpus side.
Materialization into the public `npa-mathlib` is not performed by this roadmap;
it is decided after a separate closure audit checks the import closure, axiom
policy, statement stability, and whether compatibility aliases are needed.

## 7. Advanced Field Theory Addition Plan

FT-01 through FT-07 have completed verified staging for the `AbstractField`
foundation, field homs, field-to-integral-domain, field ideal / quotient, and
ordered-field bridge. The next additions should not be materialized into
`npa-mathlib` all at once; instead, build downstream usage on the corpus side
and make layers with small import closures public-package candidates first.

The advanced-route priority is as follows.

### 7.1 Field hom kernel / image / embedding

First candidate to add:

```text
Proofs.Ai.Algebra.AbstractFieldHomKernelImage
```

Candidate theorems / API:

```text
field_hom_kernel_zero_of_nonzero
field_hom_injective_of_nonzero
field_hom_image_field_laws
field_embedding_as_field_hom
field_embedding_comp
field_iso_symm
field_iso_trans
```

Purpose:

- Increase direct downstream use of `AbstractFieldHom` and strengthen the
  promotion decision.
- Make kernel / image / injectivity for field homs reusable from the existing
  `RingHomLawArgs` and field-level `Nonzero`.
- Provide the foundation for downstream field extension / embedding /
  isomorphism APIs.

Notes:

- Treat exclusion of the zero map and injectivity as explicit arguments, such
  as evidence for preservation of `1`, `zero_ne_one`, and kernel triviality.
- Do not build a concrete kernel quotient at first; keep this to explicit
  evidence packages and projection theorems.

### 7.2 Polynomial quotient over a field

Next candidate to add:

```text
Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient
```

Candidate theorems / API:

```text
PolynomialFieldQuotientArgs
irreducible_polynomial_generates_maximal_ideal
quotient_by_irreducible_polynomial_is_field
polynomial_eval_kernel_contains_minimal_polynomial
simple_algebraic_extension_as_polynomial_quotient
```

Purpose:

- Add the standard route showing that `F[x] / (p)` is a field to the corpus.
- Build foundations for field extensions, finite fields, minimal polynomials,
  and splitting fields.
- Connect to the existing abstract Hilbert / Nullstellensatz /
  polynomial-extension style.

Notes:

- Do not yet add concrete polynomial syntax or a polynomial evaluator to the
  trusted base.
- Make evidence packages such as `IrreduciblePolynomial`,
  `PrincipalIdealGeneratedBy`, and `PolynomialQuotientFieldArgs` explicit, and
  limit proof acceptance to certificates.
- Split out the minimal bridge needed for quotient fields so the large
  `AbstractFieldIdeal` closure is not made public as-is.

### 7.3 Field extension law package

Candidate module:

```text
Proofs.Ai.Algebra.AbstractFieldExtension
```

Candidate theorems / API:

```text
FieldExtensionLawArgs
field_extension_base_embedding
field_extension_as_field
field_extension_restrict_scalars
field_extension_tower
field_embedding_compose
```

Purpose:

- Treat base field `K`, extension field `L`, and embedding `K -> L` as an
  explicit law package.
- Provide an entry point toward algebraic extensions, finite extensions,
  splitting fields, and Galois theory.
- Provide scalar restriction in a form that can connect to the existing vector
  / linear algebra corpus.

Notes:

- Module names and statements are likely to change, so keep this in corpus
  staging at first.
- Design downstream use around `FieldHomLawArgs` and
  `field_hom_injective_of_nonzero`.

### 7.4 Algebraic elements and minimal polynomial

Candidate module:

```text
Proofs.Ai.Algebra.AbstractAlgebraicExtension
```

Candidate theorems / API:

```text
AlgebraicElement
MinimalPolynomial
minimal_polynomial_divides_annihilating_polynomial
minimal_polynomial_irreducible
degree_one_algebraic_element_in_base
field_adjoin_algebraic_element_is_finite_extension
```

Purpose:

- Build a bridge between algebraic extensions and finite extensions.
- Connect the polynomial quotient route and the field extension route.
- Make downstream splitting-field / algebraic-closure statements smaller.

Notes:

- The uniqueness / monic / irreducible conditions for minimal polynomials are
  likely to fluctuate as statements. Confine them to the `MinimalPolynomial`
  evidence package first.

### 7.5 Finite extension

Candidate module:

```text
Proofs.Ai.Algebra.AbstractFiniteFieldExtension
```

Candidate theorems / API:

```text
FiniteExtensionLawArgs
finite_extension_is_algebraic
extension_degree_tower
finite_dimensional_vector_space_bridge
finite_extension_embedding_preserves_degree
```

Purpose:

- Treat degree evidence of the `[L : K]` form as an explicit package.
- Add the tower law and finite-dimensional vector-space bridge to the corpus.
- Organize dependencies for finite fields / Galois theory.

Notes:

- If natural-number arithmetic or the dimension API is heavy, initially treat
  degree laws as Prop-level evidence.

### 7.6 Finite fields and Frobenius

Candidate module:

```text
Proofs.Ai.Algebra.AbstractFiniteField
```

Candidate theorems / API:

```text
FiniteFieldLawArgs
field_characteristic_prime_or_zero
finite_field_characteristic_prime
frobenius_is_field_hom
finite_field_pow_card_eq_self
finite_field_roots_x_pow_q_minus_x
```

Purpose:

- Build the finite-field route and make Frobenius, cardinality, and root
  characterization searchable as theorems.
- Create candidates for later finite-field-specific corpus work and
  `npa-mathlib` promotion.

Notes:

- APIs for cardinality, powers, and polynomial roots can easily become
  dependency-heavy. Start with `FiniteFieldLawArgs` and the Frobenius
  homomorphism.

### 7.7 Splitting field / algebraic closure

Candidate modules:

```text
Proofs.Ai.Algebra.AbstractSplittingField
Proofs.Ai.Algebra.AbstractAlgebraicClosure
```

Candidate theorems / API:

```text
SplittingFieldLawArgs
splitting_field_contains_all_roots
splitting_field_generated_by_roots
splitting_field_unique_up_to_field_iso
AlgebraicClosureLawArgs
algebraic_closure_is_algebraic
algebraic_closure_polynomial_has_root
```

Purpose:

- Before moving to Galois theory, create evidence packages for root existence
  and uniqueness up to isomorphism.
- Treat existence theorems as explicit construction evidence, not trusted
  axioms.

Notes:

- Existence is heavy, so begin with "given splitting-field evidence."

### 7.8 Galois theory starter

Candidate module:

```text
Proofs.Ai.Algebra.AbstractGaloisStarter
```

Candidate theorems / API:

```text
FieldAutomorphismGroupArgs
fixed_field_laws
galois_extension_args
automorphism_group_laws
fixed_field_is_field
galois_correspondence_order_bridge
```

Purpose:

- Build the pre-stage for field automorphism groups, fixed fields, and Galois
  correspondence.
- Connect existing group correspondence theorems with the field extension
  route.

Notes:

- This is the heaviest dependency layer, so start it only after field
  extensions, finite extensions, and splitting fields have stabilized on the
  corpus side.

## 8. Verification

When adding field-theory modules to the proof corpus, prioritize local checks
instead of running the package/full corpus gate every time.

Example:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
```

For changes that affect export hashes across multiple modules, certificate
encode / decode / hash behavior, kernel semantics, the independent checker, or
the package verifier, explicitly run the package/full corpus gate at the end.

```sh
./scripts/check-corpus-full.sh
```

For ordinary code / documentation changes only, use the fast gate first.

```sh
./scripts/check-fast.sh
```

## 9. Completion Criteria

Completion criteria for the FT-01 through FT-07 foundation route:

- `FieldLawArgs` clearly separates `RingLawArgs` from field-specific laws.
- Theorem names for `inv` / `div` / `Nonzero` are easy for downstream modules
  to search.
- `field_ring_laws` allows reuse of existing `AbstractRing` theorems.
- `field_inv_mul_cancel` / `field_mul_inv_cancel` / `field_div_eq_mul_inv` are
  checked as certificate-backed theorems.
- Generated `.npcert` files pass the source-free verifier.
- The axiom report does not grow unexpectedly.

Completion criteria for the FT-03 through FT-07 foundation bridge layers:

- Field homs are bridged to the existing `RingHomLawArgs`.
- There are theorems that extract integral-domain behavior from fields.
- Theorems that use fields in ideal / quotient-ring routes can be reused with
  `FieldLawArgs` as a prerequisite.
- A bridge to `AbstractOrderedField` is added without breaking the existing
  order / square-root corpus.

Completion criteria for the advanced field-theory route from FT-08 onward:

- Each module uses explicit evidence packages and does not add existence
  theorems as hidden axioms.
- New theorem / definition names and statements are stable enough to be used
  at least by direct downstream modules.
- Updates to source, certificate, replay, meta, manifest, package metadata,
  and AI theorem index are deterministic.
- Local authoring passes `--build-module` / `--module` / `--changed-only` for
  the target module.
- Before promotion to the public `npa-mathlib`, separately audit the import
  closure, axiom policy, statement stability, and whether compatibility aliases
  are needed.
- Promotion candidates pass the source-free verifier and package hash / index
  / axiom report checks.
