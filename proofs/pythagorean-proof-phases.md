# Pythagorean Proof Phase Breakdown

Source: `proofs/README.md`

This task breakdown starts from the current P25 state. P25 provides checked final theorem names in
`Proofs.Ai.Geometry.Pythagorean`, but the public theorem currently accepts an explicit Pythagorean
law argument. The target of this plan is to replace that direct law argument with checked
derivations from scalar, vector, inner-product, and affine law packages while preserving the
certificate-first trust boundary.

## Scope

The main theorem target is the squared-distance Pythagorean theorem:

```text
RightTriangle A B C ->
  distSqPoints B C = distSqPoints A B + distSqPoints A C
```

The squared metric-distance theorem is a follow-on target once the squared-distance theorem no
longer depends on a direct Pythagorean law. The unsquared statement using `dist B C` itself is out
of scope until square-root cancellation and positivity APIs are strong enough.

Important constraints:

```text
- Do not add unchecked Euclidean, field, vector-space, metric, or square-root axioms.
- Keep `crates/` pure NPA unless a proof-corpus tooling gap blocks certificate generation.
- Keep proof-corpus generation under `tools/proof-corpus` and checked artifacts under `proofs/`.
- Preserve P25 public theorem names or add aliases before renaming any public target.
- Treat source, generator, tactics, and AI output as untrusted; acceptance comes from certificates.
```

## Milestones

### P26 Direct-Law Audit

- Status: Pending
- Depends on: P25
- Inputs: `proofs/README.md`, `proofs/Proofs/Ai/Geometry/Pythagorean/source.npa`,
  `proofs/Proofs/Ai/Geometry/AbstractRightTriangle/source.npa`,
  `proofs/Proofs/Ai/Geometry/AbstractMetric/source.npa`, `tools/proof-corpus/src/main.rs`
- Deliverables:
  - A short documentation update listing every remaining theorem that accepts a direct
    Pythagorean, law-of-cosines, metric-distance, or normalization law argument.
  - A target theorem signature for the first non-direct-law squared Pythagorean theorem.
- Acceptance criteria:
  - The chosen target still takes explicit law packages, but it does not take a direct theorem-shaped
    argument whose conclusion is the Pythagorean equality itself.
  - The dependency list identifies which imported law packages are required for the proof.
- Verification:
  - `rg -n "pythagorean.*law|law : forall .*RightTriangle|law_of_cosines|norm_sq_.*law" proofs tools/proof-corpus`
  - `git diff --check`
- Notes:
  - This phase is deliberately documentation-first so later proof phases can be implemented one at a
    time without changing the final target shape midstream.

### P27 Scalar Algebra Derivation Layer

- Status: Pending
- Depends on: P26
- Inputs: `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractSquareNormalize`,
  `Proofs.Ai.EqReasoning`
- Deliverables:
  - A new proof-corpus module, or an extension of the abstract algebra modules, that proves the
    scalar rewrite lemmas needed by the Pythagorean proof from `RingLawArgs` and existing square
    definitions.
  - The key target is a checked cross-term cancellation lemma of the shape:

```text
dot = 0 ->
  add (add a (mul two dot)) b = add a b
```

- Acceptance criteria:
  - The module's `axioms` list is empty except for the existing permitted `Std.Logic.Eq.rec`
    dependency when equality transport is actually used.
  - No theorem in this layer delegates to a direct `normalize_add_with_zero_cross_term_law` argument.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "normalize_add_with_zero_cross_term_law|cancel_double_zero_term.*law" proofs/Proofs/Ai tools/proof-corpus/src/main.rs`

### P28 Inner-Product Norm Expansion From Laws

- Status: Pending
- Depends on: P27
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Algebra.AbstractSquareNormalize`
- Deliverables:
  - Checked theorem targets deriving `normSq (x + y)` and the perpendicular special case from
    `InnerProductLawArgs`, scalar algebra rewrites, and `PerpVec`.
  - A theorem equivalent to:

```text
PerpVec x y ->
  normSq (vadd x y) = add (normSq x) (normSq y)
```

- Acceptance criteria:
  - The proof does not accept `norm_sq_add_of_perp_law` or
    `norm_sq_add_of_dot_zero_law` as a direct argument.
  - The theorem may still take primitive inner-product laws such as bilinearity and symmetry through
    `InnerProductLawArgs`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - Targeted review of the new theorem proof terms for absence of direct theorem-shaped law
    arguments.

### P29 Affine Hypotenuse Vector Derivation

- Status: Pending
- Depends on: P28
- Inputs: `Proofs.Ai.Geometry.Affine`, `Proofs.Ai.Vector.AbstractSpace`
- Deliverables:
  - Checked derivations connecting point displacement to vector addition/subtraction, especially:

```text
disp B C = vsub (disp A C) (disp A B)
distSqPoints X Y = normSq (disp X Y)
```

  - Orientation lemmas needed to express the hypotenuse vector in the same form expected by the
    inner-product norm expansion theorem.
- Acceptance criteria:
  - The proof does not accept `hypotenuse_vector_eq_sub_legs_law` or
    `dist_sq_points_def_law` as direct theorem-shaped arguments.
  - The theorem may still take `AffineLawArgs` as the explicit source of primitive affine laws.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "hypotenuse_vector_eq_sub_legs_law|dist_sq_points_def_law" proofs/Proofs/Ai/Geometry tools/proof-corpus/src/main.rs`

### P30 Right-Triangle Perpendicular Bridge

- Status: Pending
- Depends on: P28, P29
- Inputs: `Proofs.Ai.Geometry.AbstractRightTriangle`,
  `Proofs.Ai.Vector.AbstractInnerProduct`
- Deliverables:
  - Checked theorem targets converting `RightTriangle A B C` into the exact perpendicular or
    dot-zero premise required by P28 after applying the affine orientation lemmas from P29.
  - A small bridge theorem that keeps `RightTriangle` as the public geometric hypothesis.
- Acceptance criteria:
  - The bridge proof is definitional or uses existing checked `perp_iff_dot_eq_zero` style lemmas.
  - It does not take a theorem argument whose conclusion is already the Pythagorean equality.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - Targeted review of `RightTriangle`, `Perp`, and `PerpVec` unfolding in generated source.

### P31 Squared Pythagorean Proof From Law Packages

- Status: Pending
- Depends on: P27, P28, P29, P30
- Inputs: `Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Geometry.Pythagorean`
- Deliverables:
  - A checked theorem proving the squared-distance Pythagorean theorem from law packages:

```text
RingLawArgs ->
VectorSpaceLawArgs ->
InnerProductLawArgs ->
AffineLawArgs ->
RightTriangle A B C ->
  distSqPoints B C = add (distSqPoints A B) (distSqPoints A C)
```

  - An update to P25's `pythagorean_theorem_sq` or an alias theorem that delegates to this checked
    derivation instead of accepting a direct Pythagorean law.
- Acceptance criteria:
  - The theorem statement has no direct argument of the shape
    `forall A B C, RightTriangle A B C -> pythagorean equality`.
  - The generated module verifies with `axioms = []`, aside from imported modules' existing allowed
    equality-recursion report where applicable.
  - Existing public theorem names remain available.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `jq '.axioms, .declarations' proofs/Proofs/Ai/Geometry/Pythagorean/meta.json`
  - `rg -n "pythagorean_distance_law|pythagorean.*law : forall .*RightTriangle" proofs/Proofs/Ai/Geometry/Pythagorean tools/proof-corpus/src/main.rs`

### P32 Metric Squared-Distance Bridge Without Direct Metric Law

- Status: Pending
- Depends on: P31
- Inputs: `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Geometry.AbstractMetric`
- Deliverables:
  - Checked theorem targets deriving the needed `distSqPoints = sq dist` bridge from the definition
    of `dist`, square-root laws, and nonnegativity packages instead of a direct
    `dist_sq_eq_square_dist_law` argument.
  - A refreshed `MetricSpaceLawArgs` shape if needed, with direct Pythagorean fields removed from
    the final theorem path.
- Acceptance criteria:
  - The squared metric-distance theorem no longer requires a direct
    `pythagorean_distance_law` field.
  - Any remaining square-root assumptions are primitive square-root law package fields, not the
    target metric theorem itself.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - Targeted source review for `dist_sq_eq_square_dist_law` and `pythagorean_distance_law` use.

### P33 Final Public Pythagorean API Refresh

- Status: Pending
- Depends on: P31 for the squared theorem; P32 for the metric-distance theorem
- Inputs: `Proofs.Ai.Geometry.Pythagorean`, `proofs/README.md`,
  `proofs/manifest.toml`
- Deliverables:
  - Final public theorem names whose squared-distance proof terms are backed by P31 derivations.
  - Metric-distance proof terms backed by P32 derivations when that bridge is available.
  - README wording that distinguishes the completed squared theorem from optional future
    converse and unsquared-distance results.
  - Manifest and metadata showing the final module has no direct Pythagorean law dependency.
- Acceptance criteria:
  - `pythagorean_theorem_sq` is the completed squared-distance theorem over abstract Euclidean
    law packages.
  - `pythagorean_theorem_dist_sq` is completed if P32's bridge is available; otherwise it is clearly
    marked as a later metric layer target and not presented as fully derived.
  - `axioms = []` for the final Pythagorean module unless a documented imported equality recursor
    exception is present.
- Verification:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `rg -n "direct Pythagorean law|theorem-target layer|law : forall .*RightTriangle" proofs/README.md proofs/Proofs/Ai/Geometry/Pythagorean tools/proof-corpus/src/main.rs`

### P34 Optional Strengthening: Converse And Unsquared Distance

- Status: Pending
- Depends on: P33
- Inputs: `Proofs.Ai.Logic.Iff`, `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Geometry.AbstractMetric`
- Deliverables:
  - Converse theorem targets once nondegeneracy, angle, and square-cancellation APIs are strong
    enough.
  - Unsquared distance theorem targets when square-root cancellation can be justified from checked
    nonnegative assumptions.
- Acceptance criteria:
  - These results are not used to claim completion of the squared Pythagorean theorem.
  - Any first-class `Iff` imports avoid duplicate `Eq` source handoff issues.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - Targeted review of imported axiom reports and public theorem statements.

## Completion Definition

The squared-distance Pythagorean theorem should be considered complete when P31 and the
squared-distance portion of P33 are done:

```text
pythagorean_theorem_sq
  no longer takes a direct Pythagorean equality law,
  depends only on checked law packages and prior checked lemmas,
  verifies as a canonical certificate,
  and is documented as completed in `proofs/README.md`.
```

The squared metric-distance theorem should be considered complete when P32 and the metric portion
of P33 are done. The converse and unsquared-distance statements remain separate strengthening work.
