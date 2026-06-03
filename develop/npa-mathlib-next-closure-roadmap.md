# npa-mathlib Next Closure Roadmap

Date: 2026-06-03

This roadmap records the remaining proof-corpus routes that are good
candidates for future public `npa-mathlib` materialization after the
`v0.1.18` vector-space foundation closure. It is a planning
document, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes, deterministic
hashes, and source-free checker verdicts. Source files, replay files, meta
files, theorem indexes, publish plans, release notes, and this roadmap are
untrusted sidecars.

## Current Baseline

The standalone `npa-mathlib` package has materialized through the vector-space
foundation closure as package version `0.1.18`.

The latest completed closure audit is:

```text
develop/npa-mathlib-vector-space-closure-audit.md
```

The currently public package includes:

```text
Mathlib.Algebra.Group.Correspondence.Basic
Mathlib.Algebra.Group.Correspondence.Order
Mathlib.Algebra.Group.Correspondence
Mathlib.Algebra.Group.Correspondence.Ordered
Mathlib.Algebra.Group.ThirdIsomorphism
Mathlib.Algebra.Group.SecondIsomorphism
Mathlib.Algebra.Group.SecondIsomorphism.Image
Mathlib.Algebra.Group.SecondIsomorphism.Kernel
Mathlib.Algebra.Group.SecondIsomorphism.Map
Mathlib.Algebra.Group.Quotient.Group
Mathlib.Algebra.Group.Quotient.Mul
Mathlib.Algebra.Group.Quotient
Mathlib.Algebra.Group.FirstIsomorphism.Image
Mathlib.Algebra.Group.FirstIsomorphism
Mathlib.Algebra.Group.Kernel.Quotient.Hom
Mathlib.Algebra.Group.Kernel.Quotient.Group
Mathlib.Algebra.Group.Kernel.Quotient.Mul
Mathlib.Algebra.Group.Kernel.Quotient
Mathlib.Algebra.Group.Image
Mathlib.Algebra.Group.Kernel
Mathlib.Algebra.Group.Subgroup.Order
Mathlib.Algebra.Group.Subgroup
Mathlib.Logic.Iff
Mathlib.Logic.EqReasoning
Mathlib.Algebra.Group.Basic
Mathlib.Algebra.Ring.Basic
Mathlib.Algebra.Ring.FirstIsomorphism.Basic
Mathlib.Algebra.Ring.FirstIsomorphism
Mathlib.Algebra.Ring.ChineseRemainder
Mathlib.Algebra.OrderedField.Basic
Mathlib.Algebra.OrderedField.Square
Mathlib.Algebra.OrderedField.ScalarIdentities
Mathlib.LinearAlgebra.VectorSpace
Mathlib.Geometry.RightTriangle
Mathlib.Geometry.Metric
Mathlib.Vector.Basic
Mathlib.Vector.Dot
Mathlib.Algebra.Ring
Mathlib.Algebra.Square
Mathlib.Algebra.OrderedField
Mathlib.Logic.Basic
Mathlib.Logic.Prop
Mathlib.Logic.Eq
Mathlib.Data.Nat.Basic
Mathlib.Core.Reduction
```

## Completed Queue Items

The `Logic Iff Closure` item from this queue was completed as
`npa-mathlib v0.1.14`. Its audit is recorded in:

```text
develop/npa-mathlib-layer3e-logic-iff-closure-audit.md
```

The `Abstract Ring Foundation Closure` item from this queue was completed as
`npa-mathlib v0.1.15`. Its audit is recorded in:

```text
develop/npa-mathlib-ring-basic-closure-audit.md
```

The `Ring First Isomorphism And CRT Closure` item from this queue was completed
as `npa-mathlib v0.1.16`. Its audit is recorded in:

```text
develop/npa-mathlib-ring-isomorphism-crt-closure-audit.md
```

The `Ordered Algebra And Square Normalization Closure` item from this queue was
completed as `npa-mathlib v0.1.17`. Its audit is recorded in:

```text
develop/npa-mathlib-ordered-algebra-square-closure-audit.md
```

The `Vector Space Foundation Closure` item from this queue was completed as
`npa-mathlib v0.1.18`. Its audit is recorded in:

```text
develop/npa-mathlib-vector-space-closure-audit.md
```

The next open item is the inner product closure.

### Logic Iff Closure

Status: completed in `npa-mathlib v0.1.14`.

Recommended audit file:

```text
develop/npa-mathlib-layer3e-logic-iff-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Logic.Iff
```

Candidate public modules:

```text
Mathlib.Logic.Iff
```

Public surface to audit:

- `Iff`
- `And`
- `Or`
- `False`
- `Not`
- `iff_refl`
- `iff_symm`
- `iff_trans`
- `iff_mp`
- `iff_mpr`
- `and_intro`
- `and_left`
- `and_right`
- `iff_of_eq`
- `false_elim`
- `not_intro`
- `not_elim`
- `or_inl`
- `or_inr`
- `or_elim`
- `iff_congr_arg`

Why this should be early:

- It is small and almost standalone.
- It avoids later theorem families inventing separate proposition encodings.
- It gives downstream packages stable public names for conjunction,
  disjunction, negation, false elimination, and iff reasoning.

Audit focus:

- Decide whether `And`, `Or`, `False`, and `Not` belong in `Mathlib.Logic.Iff`
  or a broader `Mathlib.Logic.Connectives` module.
- Check whether `Eq.rec` is direct or only transitive after public renaming.
- Add downstream smoke that consumes at least `iff_mp`, `or_elim`, and
  `false_elim` source-free.

### Abstract Ring Foundation Closure

Status: completed in `npa-mathlib v0.1.15`.

Recommended audit file:

```text
develop/npa-mathlib-ring-basic-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Algebra.AbstractRing
```

Candidate public modules:

```text
Mathlib.Algebra.Ring.Basic
```

Public surface audited:

- `RingLawArgs`
- abstract `two`
- abstract `sq`
- additive group laws
- multiplication laws
- distributivity laws
- subtraction and cancellation facts

Closure unit verdict:

- `Proofs.Ai.Algebra.AbstractRing` is an appropriate single-module foundation
  closure because it imports only `Std.Logic.Eq`.
- The public namespace decision is `Mathlib.Algebra.Ring.Basic`. The existing
  concrete `Mathlib.Algebra.Ring` and `Mathlib.Algebra.Square` modules remain
  separate despite module-scoped declaration-name overlaps.
- Downstream smoke consumes `RingLawArgs`, `sub_add_cancel`, and
  `ring_normalize_add_mul3` source-free from the public certificate.

### Ring First Isomorphism And CRT Closure

Status: completed in `npa-mathlib v0.1.16`.

Recommended audit file:

```text
develop/npa-mathlib-ring-isomorphism-crt-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Algebra.AbstractRingFirstIsoBase
Proofs.Ai.Algebra.AbstractRingFirstIso
Proofs.Ai.Algebra.AbstractRingChineseRemainder
```

Public modules:

```text
Mathlib.Algebra.Ring.FirstIsomorphism.Basic
Mathlib.Algebra.Ring.FirstIsomorphism
Mathlib.Algebra.Ring.ChineseRemainder
```

Public surface audited:

- `RingHomLawArgs`, `RingImagePred`, and `RingKerQuot*` scaffolding.
- `RingFirstIso` and `ring_first_isomorphism_to_image`.
- `RingCrtPairMap`, `RingCrtCombine`, `RingCrtIntersectionPred`, and
  `RingChineseRemainder`.
- `ring_chinese_remainder_theorem`.

Closure unit verdict:

- Ring first-isomorphism and CRT were kept in one release because the CRT route
  imports the first-isomorphism modules and depends on
  `ring_first_isomorphism_to_image`.
- No separate `Mathlib.Algebra.Ring.Hom` module was introduced. The checked
  corpus base module bundles homomorphism laws, image predicates, and
  kernel-quotient construction.
- The `RingKerQuot*` names remain public in
  `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` for this release.
- Downstream smoke consumes both `ring_first_isomorphism_to_image` and
  `ring_chinese_remainder_theorem` source-free from vendored certificates.

### Ordered Algebra And Square Normalization Closure

Status: completed in `npa-mathlib v0.1.17`.

Recommended audit file:

```text
develop/npa-mathlib-ordered-algebra-square-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Algebra.AbstractOrderedField
Proofs.Ai.Algebra.AbstractSquareNormalize
Proofs.Ai.Algebra.AbstractScalarDerive
```

Public modules:

```text
Mathlib.Algebra.OrderedField.Basic
Mathlib.Algebra.OrderedField.Square
Mathlib.Algebra.OrderedField.ScalarIdentities
```

Public surface audited:

- `le`, `lt`, `sqrt`, `Nonneg`, `Positive`, and `OrderedFieldLawArgs`.
- Ordered field facts including `sqrt_sq`,
  `square_completion_bound_from_ordered_args`, and
  `add_two_mul_le_sq_add_sqrt_from_ordered_args`.
- Square-normalization facts including `sq_add_eq_add_sq_add_two_mul` and
  `normalize_add_with_zero_cross_term`.
- Scalar RHS identities including `law_of_cosines_scalar_rhs_from_ring_args`,
  `parallelogram_scalar_rhs_from_ring_args`, and
  `polarization_scalar_rhs_from_ring_args`.

Closure unit verdict:

- The three corpus modules were kept in one release because
  `AbstractScalarDerive` imports `AbstractSquareNormalize`, and
  `AbstractSquareNormalize` imports `AbstractOrderedField`.
- The abstract route is separate from the existing concrete
  `Mathlib.Algebra.OrderedField` and `Mathlib.Algebra.Square` modules.
- Downstream smoke consumes `sqrt_sq`, `sq_add_eq_add_sq_add_two_mul`, and
  `polarization_scalar_rhs_from_ring_args` source-free from vendored
  certificates.

## Closure Unit Rules

This review treats a future closure unit as appropriate only when it satisfies
all of the following conditions:

- The selected corpus modules form a closed import slice over already public
  `Mathlib.*` / `Std.*` modules plus modules selected in the same audit.
- The slice has one coherent public purpose and at least one downstream smoke
  theorem that exercises the final surface, not only helper definitions.
- The slice does not bundle an avoidably independent prerequisite chain with a
  later theorem family.
- The audit explicitly checks declaration-name collisions against already
  released modules before materialization.
- If a large route can be validly shipped either as one final-theorem closure
  or as smaller foundation closures, the audit records the chosen unit and the
  reason.

The current open queue below reflects these rules. The main changes from the
older queue are:

- `VectorSpace` is split from `InnerProduct` because analysis normed-space
  modules depend only on `Proofs.Ai.Vector.AbstractSpace`.
- The analysis chain is split into metric topology, normed space, linear map,
  derivative, fixed point, inverse function, and implicit function closures.
  The previous "analysis foundation" group was useful as a roadmap cluster but
  too large to be a default release closure.
- The ring first-isomorphism, CRT, ordered algebra, square-normalization, and
  vector-space foundation closures are complete. A separate public
  ring-homomorphism namespace should wait for either a homomorphism-only corpus
  module or an audited alias layer.

## Open Audit Queue

### 1. Inner Product Closure

Recommended audit file:

```text
develop/npa-mathlib-inner-product-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Vector.AbstractInnerProduct
Proofs.Ai.Vector.AbstractInnerProductDerive
```

Candidate public modules:

```text
Mathlib.LinearAlgebra.InnerProduct
Mathlib.LinearAlgebra.InnerProduct.Derived
```

Alternative public module prefix to evaluate:

```text
Mathlib.Vector.InnerProduct
```

Why this closure matters:

- It adds inner-product identities, parallelogram law, polarization identity,
  Cauchy-Schwarz, perpendicularity facts, and norm-square facts.
- It is a prerequisite for the abstract geometry/Pythagorean route.

Audit focus:

- Confirm the prerequisite public imports from abstract ring, ordered algebra,
  scalar derive, and vector-space foundation.
- Add downstream smoke for `parallelogram_law`, `polarization_identity`, and
  `cauchy_schwarz`.

### 2. Geometry Pythagorean Closure

Recommended audit file:

```text
develop/npa-mathlib-geometry-pythagorean-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Geometry.Affine
Proofs.Ai.Geometry.AffineDerive
Proofs.Ai.Geometry.AbstractRightTriangle
Proofs.Ai.Geometry.AbstractRightTriangleDerive
Proofs.Ai.Geometry.AbstractMetric
Proofs.Ai.Geometry.Pythagorean
```

Candidate public modules:

```text
Mathlib.Geometry.Affine
Mathlib.Geometry.Affine.Derived
Mathlib.Geometry.RightTriangle.Abstract
Mathlib.Geometry.RightTriangle.Derived
Mathlib.Geometry.Metric.Abstract
Mathlib.Geometry.Pythagorean
```

Why this closure matters:

- It upgrades the currently public concrete geometry layer into an abstract
  law-package geometry route.
- It should finally make `Mathlib.Geometry.Pythagorean` publishable without
  hiding abstract algebra/vector dependencies.

Closure unit verdict:

- This is a valid final-theorem closure after inner product is public because
  `Proofs.Ai.Geometry.Pythagorean` imports the other five geometry modules.
- If the materialization diff is too large, the audit should split it into:

```text
Geometry Affine/RightTriangle Foundation:
  Proofs.Ai.Geometry.Affine
  Proofs.Ai.Geometry.AffineDerive
  Proofs.Ai.Geometry.AbstractRightTriangle
  Proofs.Ai.Geometry.AbstractRightTriangleDerive

Geometry Metric/Pythagorean Final:
  Proofs.Ai.Geometry.AbstractMetric
  Proofs.Ai.Geometry.Pythagorean
```

Audit focus:

- Do not materialize this before ordered algebra and inner-product routes are
  public.
- Decide whether existing `Mathlib.Geometry.RightTriangle` and
  `Mathlib.Geometry.Metric` remain concrete modules while abstract routes get
  `.Abstract` suffixes.
- Add downstream smoke that consumes `pythagorean_distance_general` or the
  final `Pythagorean` theorem surface source-free.

### 3. Analysis Metric Topology Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-metric-topology-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractMetricTopology
```

Candidate public modules:

```text
Mathlib.Topology.Metric.Basic
```

Why this closure matters:

- It is a small standalone analysis foundation route; it imports only
  `Std.Logic.Eq` and equality reasoning.
- It should be materialized before derivative, fixed-point, and inverse
  function closures.

Audit focus:

- Decide whether the module belongs under `Mathlib.Topology.Metric.Basic` or
  a more analysis-oriented namespace.
- Add downstream smoke for `metric_ball_mono`, `local_eq_trans`, and
  `local_unique_apply`.

### 4. Analysis Normed Space Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-normed-space-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractNormedSpace
```

Candidate public modules:

```text
Mathlib.Analysis.NormedSpace.Basic
```

Why this closure matters:

- It adds norm and product norm law-package facts needed by linear maps,
  derivative, fixed-point, and inverse-function routes.

Audit focus:

- Confirm that `Mathlib.LinearAlgebra.VectorSpace` is already public.
- Add downstream smoke for `norm_dist_triangle_from_args` and
  `product_norm_pair_le_add_from_args`.

### 5. Analysis Linear Map Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-linear-map-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractLinearMap
```

Candidate public modules:

```text
Mathlib.Analysis.LinearMap
```

Why this closure matters:

- It adds bounded-linear-map, linear-isomorphism, composition, inverse, and
  block-triangular map facts used by derivative and implicit-function routes.

Audit focus:

- Confirm public imports from vector-space and normed-space closures.
- Add downstream smoke for `linear_comp_law_args`,
  `linear_inv_left_inverse_from_iso`, and
  `block_triangular_b_iso_from_args`.

### 6. Analysis Derivative Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-derivative-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractDerivative
```

Candidate public modules:

```text
Mathlib.Analysis.Calculus.Derivative
```

Why this closure matters:

- It adds Frechet derivative, differentiability, uniqueness, product/pair, and
  composition rule surfaces.

Audit focus:

- Confirm public imports from metric topology, vector-space, normed-space, and
  linear-map closures.
- Add downstream smoke for `frechet_derivative_at_intro`,
  `derivative_comp_from_args`, and `partial_x_derivative_from_args`.

### 7. Analysis Fixed Point Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-fixed-point-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractFixedPoint
```

Candidate public modules:

```text
Mathlib.Analysis.FixedPoint.Banach
```

Why this closure matters:

- It adds completeness, contractive map, fixed-point evidence, and Banach
  fixed-point theorem surfaces used by the inverse-function route.

Audit focus:

- Confirm public imports from metric topology, vector-space, and normed-space.
- Add downstream smoke for `fixed_point_unique_from_evidence` and
  `banach_fixed_point_from_args`.

### 8. Analysis Inverse Function Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-inverse-function-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractInverseFunction
```

Candidate public modules:

```text
Mathlib.Analysis.Calculus.InverseFunction
```

Why this closure matters:

- It adds local inverse evidence and the quantitative inverse function theorem
  route after derivative and fixed point are public.

Audit focus:

- Confirm public imports from metric topology, vector-space, normed-space,
  linear-map, derivative, and fixed-point closures.
- Add downstream smoke for `local_inverse_result_intro` and
  `quantitative_inverse_function_from_args`.

### 9. Analysis Implicit Function Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-implicit-function-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractImplicitPhi
Proofs.Ai.Analysis.AbstractImplicitFunction
```

Candidate public modules:

```text
Mathlib.Analysis.Calculus.ImplicitFunction.Phi
Mathlib.Analysis.Calculus.ImplicitFunction
```

Why this closure matters:

- It adds the auxiliary implicit `Phi` map and the final implicit-function
  theorem evidence surface after derivative and linear-map APIs are public.

Audit focus:

- Keep `ImplicitPhi` and `ImplicitFunction` in one closure because the final
  module imports `AbstractImplicitPhi`.
- Confirm public imports from vector-space, normed-space, linear-map, and
  derivative closures.
- Add downstream smoke for `implicit_phi_derivative_from_args`,
  `implicit_function_theorem`, and `implicit_function_derivative_theorem`.

## Lower Priority Seed Closures

The following routes are useful, but should not jump ahead of their foundation
closures:

| Candidate corpus module | Suggested public route | Reason to defer |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization` | `Mathlib.Algebra.Ring.UFD` | Requires abstract ring and careful domain/factorization namespace decisions. |
| `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem` | `Mathlib.Algebra.Ring.Noetherian` or `Mathlib.Algebra.Commutative.HilbertBasis` | Requires abstract ring and ideal namespace decisions. |
| `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz` | `Mathlib.AlgebraicGeometry.Nullstellensatz` | Depends on Hilbert basis and algebraic-geometry naming decisions. |
| `Proofs.Ai.Algebra.AbstractKrullTheorem` | `Mathlib.Algebra.Ring.Ideal.Krull` | Depends on ideal/maximal ideal namespace decisions. |
| `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` | `Mathlib.LinearAlgebra.SpectralTheorem` | Current corpus route is standalone and needs field/matrix namespace audit. |
| `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem` | `Mathlib.Analysis.Functional.SpectralTheorem` | Current corpus route is standalone and should wait for analysis/functional-analysis namespace review. |

## Required Shape Of Each Future Closure Audit

Each new closure audit should include:

- selected corpus modules;
- closure-unit rationale, including why the selected modules should ship
  together and which nearby modules are intentionally split out;
- explicitly deferred nearby modules;
- public module names and filesystem paths;
- import rewrite table from `Proofs.Ai.*` to `Mathlib.*` / `Std.*`;
- public declaration inventory;
- source hash, certificate file hash, export hash, axiom report hash, and
  certificate hash for corpus inputs;
- expected public downstream smoke theorem names;
- axiom policy and `Eq.rec` direct/transitive status;
- positive package gates to run;
- negative package-copy checks to run;
- release artifact and publish-plan evidence to record after materialization.

The minimum positive gates after materialization remain:

```sh
cargo run -q -p npa-cli -- package check --root /path/to/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /path/to/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /path/to/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /path/to/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /path/to/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /path/to/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /path/to/npa-mathlib --check --json
```

Each materialized release should also keep a downstream smoke fixture that
consumes only vendored certificate bytes for the relevant public import closure.

Minimum negative checks:

- bad public export hash is rejected as `export_hash_mismatch`;
- bad public certificate hash is rejected as `certificate_hash_mismatch`;
- corrupted certificate bytes are rejected by source-free reference
  verification;
- stale downstream lock or package-version pin is rejected as
  `package_lock_stale`.
