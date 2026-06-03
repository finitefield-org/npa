# npa-mathlib Next Closure Roadmap

Date: 2026-06-03

This roadmap records the remaining proof-corpus routes that are good
candidates for future public `npa-mathlib` materialization after the
`v0.1.24` analysis derivative closure. It is a planning
document, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes, deterministic
hashes, and source-free checker verdicts. Source files, replay files, meta
files, theorem indexes, publish plans, release notes, and this roadmap are
untrusted sidecars.

## Current Baseline

The standalone `npa-mathlib` package has materialized through the analysis
derivative closure as package version `0.1.24`.

The latest completed closure audit is:

```text
develop/npa-mathlib-analysis-derivative-closure-audit.md
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
Mathlib.Topology.Metric.Basic
Mathlib.Algebra.Group.Basic
Mathlib.Algebra.Ring.Basic
Mathlib.Algebra.Ring.FirstIsomorphism.Basic
Mathlib.Algebra.Ring.FirstIsomorphism
Mathlib.Algebra.Ring.ChineseRemainder
Mathlib.Algebra.OrderedField.Basic
Mathlib.Algebra.OrderedField.Square
Mathlib.Algebra.OrderedField.ScalarIdentities
Mathlib.LinearAlgebra.VectorSpace
Mathlib.Analysis.NormedSpace.Basic
Mathlib.Analysis.LinearMap
Mathlib.Analysis.Calculus.Derivative
Mathlib.LinearAlgebra.InnerProduct
Mathlib.LinearAlgebra.InnerProduct.Derived
Mathlib.Geometry.Affine
Mathlib.Geometry.Affine.Derived
Mathlib.Geometry.RightTriangle.Abstract
Mathlib.Geometry.RightTriangle.Derived
Mathlib.Geometry.Metric.Abstract
Mathlib.Geometry.Pythagorean
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

The `Inner Product Closure` item from this queue was completed as
`npa-mathlib v0.1.19`. Its audit is recorded in:

```text
develop/npa-mathlib-inner-product-closure-audit.md
```

The `Geometry Pythagorean Closure` item from this queue was completed as
`npa-mathlib v0.1.20`. Its audit is recorded in:

```text
develop/npa-mathlib-geometry-pythagorean-closure-audit.md
```

The `Analysis Metric Topology Closure` item from this queue was completed as
`npa-mathlib v0.1.21`. Its audit is recorded in:

```text
develop/npa-mathlib-analysis-metric-topology-closure-audit.md
```

The `Analysis Normed Space Closure` item from this queue was completed as
`npa-mathlib v0.1.22`. Its audit is recorded in:

```text
develop/npa-mathlib-analysis-normed-space-closure-audit.md
```

The `Analysis Linear Map Closure` item from this queue was completed as
`npa-mathlib v0.1.23`. Its audit is recorded in:

```text
develop/npa-mathlib-analysis-linear-map-closure-audit.md
```

The `Analysis Derivative Closure` item from this queue was completed as
`npa-mathlib v0.1.24`. Its audit is recorded in:

```text
develop/npa-mathlib-analysis-derivative-closure-audit.md
```

The next open item is the analysis fixed-point closure.

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

### Inner Product Closure

Status: completed in `npa-mathlib v0.1.19`.

Recommended audit file:

```text
develop/npa-mathlib-inner-product-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Vector.AbstractInnerProduct
Proofs.Ai.Vector.AbstractInnerProductDerive
```

Public modules:

```text
Mathlib.LinearAlgebra.InnerProduct
Mathlib.LinearAlgebra.InnerProduct.Derived
```

Public surface audited:

- `dot`, `normSq`, `distSq`, `PerpVec`, and `InnerProductLawArgs`.
- Inner-product law projections including `parallelogram_law`,
  `polarization_identity`, `cauchy_schwarz`, perpendicularity facts, and
  norm-square facts.
- Derived law-package theorem routes including
  `parallelogram_law_from_inner_args`,
  `polarization_identity_from_inner_args`,
  `cauchy_schwarz_from_law_packages`, and norm-square derived bounds.

Closure unit verdict:

- The two corpus modules were kept in one release because
  `AbstractInnerProductDerive` imports `AbstractInnerProduct`, and the pair
  forms the coherent public inner-product closure over already released ring,
  ordered algebra, scalar-identity, equality-reasoning, and vector-space
  modules.
- The abstract route is separate from the existing concrete
  `Mathlib.Vector.Dot` module.
- Downstream smoke consumes `parallelogram_law`,
  `polarization_identity`, and `cauchy_schwarz_from_law_packages` source-free
  from vendored certificates.

### Geometry Pythagorean Closure

Status: completed in `npa-mathlib v0.1.20`.

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

Public modules:

```text
Mathlib.Geometry.Affine
Mathlib.Geometry.Affine.Derived
Mathlib.Geometry.RightTriangle.Abstract
Mathlib.Geometry.RightTriangle.Derived
Mathlib.Geometry.Metric.Abstract
Mathlib.Geometry.Pythagorean
```

Public surface audited:

- Abstract affine point/displacement facts including `Point`, `disp`,
  `distSqPoints`, `AffineLawArgs`, `dist_sq_symm`, and
  `dist_sq_zero_iff_eq`.
- Abstract right-triangle facts including `Perp`, `RightTriangle`,
  `right_triangle_legs_perp`, `pythagorean_distance_sq_general`,
  `law_of_cosines_general`, and `median_to_hypotenuse_general`.
- Metric and Pythagorean theorem facts including `dist`,
  `MetricSpaceLawArgs`, `pythagorean_distance_general`,
  `pythagorean_theorem_dist_sq`, `pythagorean_converse_sq`, and
  `pythagorean_theorem_api_alias`.

Closure unit verdict:

- The six corpus modules were kept in one release because
  `Proofs.Ai.Geometry.Pythagorean` imports the other five geometry modules and
  the final public value is the Pythagorean/law-of-cosines theorem surface.
- The abstract route is separate from the existing concrete
  `Mathlib.Geometry.RightTriangle` and `Mathlib.Geometry.Metric` modules.
- Downstream smoke consumes `pythagorean_distance_general`,
  `pythagorean_theorem_dist_sq`, and `pythagorean_theorem_api_alias`
  source-free from vendored certificates.

### Analysis Metric Topology Closure

Status: completed in `npa-mathlib v0.1.21`.

Recommended audit file:

```text
develop/npa-mathlib-analysis-metric-topology-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractMetricTopology
```

Public modules:

```text
Mathlib.Topology.Metric.Basic
```

Public surface audited:

- `MetricBall`, `Neighborhood`, `LocalMem`, `LocalPred`, `LocalEq`, and
  `LocalUnique`.
- Introduction/elimination and shrink/monotonicity facts including
  `metric_ball_intro`, `metric_ball_elim`, `neighborhood_shrink`,
  `local_pred_shrink`, and `metric_ball_mono`.
- Local equality and uniqueness facts including `local_eq_refl`,
  `local_eq_symm`, `local_eq_trans`, and `local_unique_apply`.

Closure unit verdict:

- The corpus module is a valid single-module topology closure because it
  imports only `Std.Logic.Eq` and equality reasoning.
- The module belongs under `Mathlib.Topology.Metric.Basic` because it provides
  local metric-topology vocabulary rather than a final analysis theorem
  surface.
- Downstream smoke consumes `metric_ball_mono`, `local_eq_trans`, and
  `local_unique_apply` source-free from vendored certificates.

### Analysis Normed Space Closure

Status: completed in `npa-mathlib v0.1.22`.

Recommended audit file:

```text
develop/npa-mathlib-analysis-normed-space-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractNormedSpace
```

Public modules:

```text
Mathlib.Analysis.NormedSpace.Basic
```

Public surface audited:

- `NormDist`, `NormedSpaceLawArgs`, product vector operations, `ProductNorm`,
  `ProductDist`, and `ProductNormEstimateArgs`.
- Norm facts including `norm_dist_def`, `norm_nonneg_from_args`,
  `norm_zero_from_args`, `norm_triangle_from_args`,
  `norm_dist_symm_from_args`, and `norm_dist_triangle_from_args`.
- Product norm facts including `product_norm_pair_eq_from_pair_laws`,
  `product_norm_fst_le_from_args`, `product_norm_snd_le_from_args`,
  `product_norm_pair_le_add_from_args`, `product_norm_add_le_from_args`, and
  `product_dist_pair_le_add_from_args`.

Closure unit verdict:

- The corpus module is a valid single-module normed-space closure because it
  imports only public `Std.Logic.Eq`, equality reasoning, and vector-space
  foundations.
- The module belongs under `Mathlib.Analysis.NormedSpace.Basic` because it
  provides norm/product-norm structure used by later analysis routes.
- Downstream smoke consumes `norm_dist_triangle_from_args` and
  `product_norm_pair_le_add_from_args` source-free from vendored certificates.

### Analysis Linear Map Closure

Status: completed in `npa-mathlib v0.1.23`.

Recommended audit file:

```text
develop/npa-mathlib-analysis-linear-map-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractLinearMap
```

Public modules:

```text
Mathlib.Analysis.LinearMap
```

Public surface audited:

- `OperatorNormBound`, `LinearMapLawArgs`, `BoundedLinearMapArgs`, and
  `LinearIsoArgs`.
- Identity, composition, and inverse map APIs including `LinearId`,
  `LinearComp`, `LinearInv`, `linear_comp_law_args`, and
  `linear_inv_left_inverse_from_iso`.
- Block-triangular map APIs including `BlockTriangularMap`,
  `BlockTriangularInverse`, `BlockTriangularIsoArgs`, and
  `block_triangular_b_iso_from_args`.

Closure unit verdict:

- The corpus module is a valid single-module linear-map closure because it
  imports only public `Std.Logic.Eq`, equality reasoning, vector-space, and
  normed-space foundations.
- The module belongs under `Mathlib.Analysis.LinearMap` because the checked
  surface is bounded and normed rather than purely algebraic linear algebra.
- Downstream smoke consumes `linear_comp_law_args`,
  `linear_inv_left_inverse_from_iso`, and
  `block_triangular_b_iso_from_args` source-free from vendored certificates.

### Analysis Derivative Closure

Status: completed in `npa-mathlib v0.1.24`.

Recommended audit file:

```text
develop/npa-mathlib-analysis-derivative-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractDerivative
```

Public modules:

```text
Mathlib.Analysis.Calculus.Derivative
```

Public surface audited:

- `FrechetRemainder`, `FrechetDerivativeAt`,
  `FrechetDifferentiableAt`, `FrechetDifferentiableOn`, and
  `DerivativeUniqueArgs`.
- Constant, zero, pair, and partial-map APIs including `ConstMap`, `ZeroMap`,
  `PairMap`, `PartialXMap`, `PartialYMap`, `PartialXDerivativeMap`, and
  `PartialYDerivativeMap`.
- Derivative rule argument packages including `DerivativeConstRuleArgs`,
  `DerivativeIdRuleArgs`, `DerivativeFstRuleArgs`,
  `DerivativeSndRuleArgs`, `DerivativePairRuleArgs`,
  `DerivativeCompRuleArgs`, and `PartialDerivativeRuleArgs`.
- Theorem surfaces including `frechet_derivative_at_intro`,
  `derivative_comp_from_args`, `partial_x_derivative_from_args`, and
  `partial_y_derivative_from_args`.

Closure unit verdict:

- The corpus module is a valid single-module derivative closure because it
  imports only public `Std.Logic.Eq`, equality reasoning, metric topology,
  vector-space, normed-space, and linear-map foundations.
- The module belongs under `Mathlib.Analysis.Calculus.Derivative` because it
  starts the calculus namespace used by later inverse-function and
  implicit-function routes.
- Downstream smoke consumes `frechet_derivative_at_intro`,
  `derivative_comp_from_args`, and `partial_x_derivative_from_args`
  source-free from vendored certificates.

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
- The ring first-isomorphism, CRT, ordered algebra, square-normalization,
  vector-space foundation, inner-product, geometry Pythagorean, analysis
  metric topology, analysis normed-space, analysis linear-map, and analysis
  derivative closures are complete. A separate public ring-homomorphism
  namespace should wait for either a homomorphism-only corpus module or an
  audited alias layer.

## Open Audit Queue

### 1. Analysis Fixed Point Closure

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

### 2. Analysis Inverse Function Closure

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

### 3. Analysis Implicit Function Closure

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
- stale downstream import identity, at minimum the imported `export_hash` or
  `certificate_hash`, is rejected by the downstream build or lock gate.

## Package Tooling Follow-up Audits

The analysis metric topology closure found one non-blocking package tooling
gap: changing only a downstream external import version string from `0.1.21`
to `0.1.20` did not fail `package check` or `package check-hashes`.

This is not a certificate soundness failure because downstream imports are
bound by `export_hash` and `certificate_hash`, and those identity mismatches
are rejected. A future package tooling audit should decide whether generated
locks and artifact checks must also reject version-only drift explicitly.
