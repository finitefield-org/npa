# npa-mathlib Next Closure Roadmap

Date: 2026-06-03

This roadmap records the remaining proof-corpus routes that are good
candidates for future public `npa-mathlib` materialization after Layer 3D-G
Correspondence. It is a planning document, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes, deterministic
hashes, and source-free checker verdicts. Source files, replay files, meta
files, theorem indexes, publish plans, release notes, and this roadmap are
untrusted sidecars.

## Current Baseline

The standalone `npa-mathlib` package has materialized through Layer 3D-G as
package version `0.1.13`.

The latest completed closure audit is:

```text
develop/npa-mathlib-layer3d-g-correspondence-closure-audit.md
```

The currently public group-theory closure includes:

```text
Mathlib.Algebra.Group.Basic
Mathlib.Algebra.Group.Subgroup
Mathlib.Algebra.Group.Subgroup.Order
Mathlib.Algebra.Group.Kernel
Mathlib.Algebra.Group.Image
Mathlib.Algebra.Group.Kernel.Quotient
Mathlib.Algebra.Group.Kernel.Quotient.Mul
Mathlib.Algebra.Group.Kernel.Quotient.Group
Mathlib.Algebra.Group.Kernel.Quotient.Hom
Mathlib.Algebra.Group.FirstIsomorphism
Mathlib.Algebra.Group.FirstIsomorphism.Image
Mathlib.Algebra.Group.Quotient
Mathlib.Algebra.Group.Quotient.Mul
Mathlib.Algebra.Group.Quotient.Group
Mathlib.Algebra.Group.SecondIsomorphism.Map
Mathlib.Algebra.Group.SecondIsomorphism.Kernel
Mathlib.Algebra.Group.SecondIsomorphism.Image
Mathlib.Algebra.Group.SecondIsomorphism
Mathlib.Algebra.Group.ThirdIsomorphism
Mathlib.Algebra.Group.Correspondence.Basic
Mathlib.Algebra.Group.Correspondence.Order
Mathlib.Algebra.Group.Correspondence
Mathlib.Algebra.Group.Correspondence.Ordered
```

## Recommended Audit Queue

### 1. Logic Iff Closure

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

### 2. Abstract Ring Foundation Closure

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

Alternative public module names to evaluate:

```text
Mathlib.Algebra.Ring.Abstract
Mathlib.Algebra.Ring.Laws
```

Public surface to audit:

- `RingLawArgs`
- abstract `two`
- abstract `sq`
- additive group laws
- multiplication laws
- distributivity laws
- subtraction and cancellation facts

Why this should be next after Logic Iff:

- It is the foundation for ring homomorphisms, ring first isomorphism, CRT,
  UFD, Hilbert basis, Nullstellensatz, Krull, ordered algebra, vector spaces,
  and geometry law-package routes.
- It forces a namespace decision between the existing concrete
  `Mathlib.Algebra.Ring` module and the proof-corpus abstract ring API.

Audit focus:

- Do not overload the existing `Mathlib.Algebra.Ring` module casually.
- Decide whether the existing concrete ring module should remain as-is while
  abstract law-package facts move under `Mathlib.Algebra.Ring.Basic`.
- Verify that public source names do not collide with already released
  `Mathlib.Algebra.Ring` declarations.
- Add downstream smoke that consumes `RingLawArgs`, `sub_add_cancel`, and
  `ring_normalize_add_mul3` source-free.

### 3. Ring First Isomorphism And CRT Closure

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

Candidate public modules:

```text
Mathlib.Algebra.Ring.Hom
Mathlib.Algebra.Ring.FirstIsomorphism.Basic
Mathlib.Algebra.Ring.FirstIsomorphism
Mathlib.Algebra.Ring.ChineseRemainder
```

Why this closure matters:

- It is the natural ring analogue of the already public group first
  isomorphism route.
- It adds public ring homomorphism laws, image predicates, kernel quotient
  construction, and a CRT theorem evidence surface.

Audit focus:

- Determine how much of the additive-group quotient import closure should be
  exposed as a public ring quotient API.
- Decide whether `RingKerQuot*` names are acceptable public identifiers or
  should move under a clearer ring quotient namespace.
- Confirm that the closure imports public group quotient / image /
  first-isomorphism modules rather than corpus names.
- Add downstream smoke for both `ring_first_isomorphism_to_image` and
  `ring_chinese_remainder_theorem`.

### 4. Ordered Algebra And Square Normalization Closure

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

Candidate public modules:

```text
Mathlib.Algebra.OrderedField.Basic
Mathlib.Algebra.OrderedField.Square
Mathlib.Algebra.OrderedField.ScalarIdentities
```

Why this closure matters:

- It prepares the vector, inner-product, and geometry law-package routes.
- It exposes reusable ordered-field square facts and scalar identities such as
  square completion, law-of-cosines scalar RHS, and polarization scalar RHS.

Audit focus:

- Decide how this abstract ordered-field surface relates to already released
  `Mathlib.Algebra.OrderedField`.
- Confirm whether the square-normalization API should live under ordered field
  or under a separate algebra square namespace.
- Add downstream smoke for `sqrt_sq`, `sq_add_eq_add_sq_add_two_mul`, and one
  scalar derive theorem.

### 5. Vector Space And Inner Product Closure

Recommended audit file:

```text
develop/npa-mathlib-vector-inner-product-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Vector.AbstractSpace
Proofs.Ai.Vector.AbstractInnerProduct
Proofs.Ai.Vector.AbstractInnerProductDerive
```

Candidate public modules:

```text
Mathlib.LinearAlgebra.VectorSpace
Mathlib.LinearAlgebra.InnerProduct
Mathlib.LinearAlgebra.InnerProduct.Derived
```

Alternative public module prefix to evaluate:

```text
Mathlib.Vector.AbstractSpace
Mathlib.Vector.InnerProduct
```

Why this closure matters:

- It adds high-value reusable mathematics: vector-space laws, inner-product
  identities, parallelogram law, polarization identity, and Cauchy-Schwarz.
- It is a prerequisite for the abstract geometry and normed-space routes.

Audit focus:

- Decide whether public namespace should move from existing `Mathlib.Vector.*`
  to `Mathlib.LinearAlgebra.*` for abstract vector spaces.
- Confirm the dependency path through abstract ordered field and square
  normalization.
- Add downstream smoke for `parallelogram_law`, `polarization_identity`, and
  `cauchy_schwarz`.

### 6. Geometry Pythagorean Closure

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

Audit focus:

- Do not materialize this before ordered algebra and inner-product routes are
  public.
- Decide whether existing `Mathlib.Geometry.RightTriangle` and
  `Mathlib.Geometry.Metric` remain concrete modules while abstract routes get
  `.Abstract` suffixes.
- Add downstream smoke that consumes `pythagorean_distance_general` or the
  final `Pythagorean` theorem surface source-free.

### 7. Analysis Foundation Closure

Recommended audit file:

```text
develop/npa-mathlib-analysis-foundation-closure-audit.md
```

Candidate corpus modules:

```text
Proofs.Ai.Analysis.AbstractMetricTopology
Proofs.Ai.Analysis.AbstractNormedSpace
Proofs.Ai.Analysis.AbstractLinearMap
Proofs.Ai.Analysis.AbstractDerivative
Proofs.Ai.Analysis.AbstractFixedPoint
Proofs.Ai.Analysis.AbstractInverseFunction
Proofs.Ai.Analysis.AbstractImplicitPhi
Proofs.Ai.Analysis.AbstractImplicitFunction
```

Recommended split:

```text
MetricTopology
NormedSpace
LinearMap
Derivative
FixedPoint
InverseFunction
ImplicitPhi
ImplicitFunction
```

Candidate public modules:

```text
Mathlib.Topology.Metric.Basic
Mathlib.Analysis.NormedSpace.Basic
Mathlib.Analysis.LinearMap
Mathlib.Analysis.Calculus.Derivative
Mathlib.Analysis.FixedPoint.Banach
Mathlib.Analysis.Calculus.InverseFunction
Mathlib.Analysis.Calculus.ImplicitFunction.Phi
Mathlib.Analysis.Calculus.ImplicitFunction
```

Why this closure matters:

- It opens the path to calculus theorem evidence and fixed-point based
  inverse/implicit function routes.

Audit focus:

- Do not ship the entire analysis chain in one release.
- First audit `AbstractMetricTopology` separately if a small safe release is
  desired.
- Keep topology, normed-space, linear-map, derivative, fixed-point, inverse,
  and implicit-function namespaces separate.
- Add downstream smoke for the final theorem of each split, not just helper
  definitions.

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
