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

- Status: Completed
- Depends on: P25
- Inputs: `proofs/README.md`, `proofs/Proofs/Ai/Geometry/Pythagorean/source.npa`,
  `proofs/Proofs/Ai/Geometry/AbstractRightTriangle/source.npa`,
  `proofs/Proofs/Ai/Geometry/AbstractRightTriangleDerive/source.npa`,
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

#### P26 Audit Result

The remaining direct-law arguments in the post-P25 route are grouped below. A direct-law argument is
an explicit theorem-shaped argument whose conclusion is already the Pythagorean equality,
law-of-cosines equality, metric-distance equality, or normalization equality that a later phase must
derive from smaller checked components. Concrete singleton modules from P10-P15 remain checked
predecessors, but the replacement target below is the P17-P25 abstract route.

| Area | Current direct-law item | Why it matters | Follow-up |
| --- | --- | --- | --- |
| Scalar normalization | `Proofs.Ai.Algebra.AbstractSquareNormalize`: `sq_add`, `sq_sub`, `sum_two_squares_comm`, `cancel_double_zero_term`, `sq_zero`, `sq_one`, `sq_neg`, `two_mul`, `sq_eq_sq_of_eq_or_neg_eq`, `sq_add_eq_add_sq_add_two_mul`, `sq_sub_eq_add_sq_sub_two_mul`, `add_sq_eq_zero_iff`, `mul_two_zero_term`, `normalize_add_with_zero_cross_term` | These currently accept a direct scalar rewrite law. The Pythagorean proof needs the cross-term path from `dot = 0` to removing `mul two dot`. | P27 derives the required scalar rewrite lemmas from `RingLawArgs`, square definitions, and equality reasoning. |
| Inner-product norm normalization | `Proofs.Ai.Vector.AbstractInnerProduct`: `norm_sq_add`, `norm_sq_sub`, `norm_sq_add_of_dot_zero`, `norm_sq_sub_of_dot_zero`, `parallelogram_law`, `polarization_identity`, `norm_sq_zero_iff`, `norm_sq_nonneg`, `dist_sq_nonneg`, `norm_sq_add_of_perp`, `norm_sq_sub_of_perp`; `Proofs.Ai.Vector.AbstractInnerProductDerive`: `norm_sq_add_from_inner_args`, `norm_sq_add_of_dot_zero_from_args`, `norm_sq_add_of_perp_from_args` | The base module still exposes direct norm, dot-zero, or norm-identity law wrappers. The squared theorem needs the perpendicular special case without taking it as a law. | P28 derives the norm expansion and perpendicular special case from `InnerProductLawArgs` and P27 scalar rewrites in `AbstractInnerProductDerive`. |
| Affine normalization | `Proofs.Ai.Geometry.Affine`: legacy theorem target `hypotenuse_vector_eq_sub_legs` still accepts an explicit theorem-shaped argument; `Proofs.Ai.Geometry.AffineDerive`: `hypotenuse_vector_eq_neg_left_add_right_from_args`, `hypotenuse_vector_eq_sub_legs_from_args`, `dist_sq_hypotenuse_norm_neg_left_add_right_from_args`, `dist_sq_hypotenuse_norm_sub_legs_from_args` | The final proof must connect `distSqPoints B C` to the norm of the hypotenuse vector and orient that vector against the two legs. `distSqPoints` is definitional, and `AffineLawArgs` no longer carries direct hypotenuse-vector or point-distance-definition fields. | P29 derives the needed affine orientation and distance-square bridge from primitive `AffineLawArgs` fields, `VectorSpaceLawArgs`, and equality transport in `AffineDerive`. |
| Right-triangle geometry | `Proofs.Ai.Geometry.AbstractRightTriangle`: `pythagorean_distance_sq_general`, `law_of_cosines_general`, `right_triangle_area_general`, `median_to_hypotenuse_general`; `Proofs.Ai.Geometry.AbstractRightTriangleDerive`: `right_triangle_neg_left_perp_vec_from_rt` | `pythagorean_distance_sq_general` is the direct squared Pythagorean law. `law_of_cosines_general` is the intended intermediate identity but is also currently direct. Area and median targets are same-level right-triangle facts and must not be mistaken for prerequisites of the first squared theorem. | P30 builds the right-triangle to perpendicular bridge in `AbstractRightTriangleDerive`; P31 replaces the Pythagorean direct law with the derived squared theorem. Area and median remain later peer work. |
| Metric-distance layer | `Proofs.Ai.Geometry.AbstractMetric`: `MetricSpaceLawArgs` fields `dist_def_law`, `dist_nonneg_law`, `distance_symm_law`, `distance_zero_iff_eq_law`, `triangle_inequality_law`; theorem targets `point_dist_sq_nonneg_from_inner_args`, `square_dist_eq_dist_sq_from_law_packages`, `dist_sq_eq_square_dist_from_law_packages`, and `dist_sq_eq_square_dist` derive the metric square bridge from `OrderedFieldLawArgs` and `InnerProductLawArgs`. Legacy metric wrappers such as `pythagorean_distance_general` remain available but are not on the final P32 path. | The squared metric-distance theorem now avoids the direct metric Pythagorean field and the direct `distSqPoints = sq dist` law field. | P32 is complete; P33 can refresh the public API wording around the completed metric-distance theorem. |
| Final public API | `Proofs.Ai.Geometry.Pythagorean`: `pythagorean_theorem_sq`, `pythagorean_theorem_dist_sq`, `pythagorean_converse_sq`, `law_of_cosines_right_angle_specialization`, `pythagorean_theorem_api_alias` | `pythagorean_theorem_sq` and `pythagorean_theorem_dist_sq` are backed by P31/P32 checked derivations. `pythagorean_theorem_dependencies` is only a law-package identity and is not a direct theorem law. | P33 refreshes the public theorem names and documentation around the now-completed squared and squared-metric targets. Converse strengthening remains P34. |

The first non-direct-law target is the squared-distance theorem below. It still takes explicit law
packages, because NPA does not yet have the structure/class layer, but it does not take an argument
whose conclusion is already the Pythagorean equality.

```text
theorem pythagorean_distance_sq_from_law_packages.{p,u,v} :
  forall (Scalar : Sort u),
  forall (zero : Scalar),
  forall (one : Scalar),
  forall (add : Scalar -> Scalar -> Scalar),
  forall (neg : Scalar -> Scalar),
  forall (sub : Scalar -> Scalar -> Scalar),
  forall (mul : Scalar -> Scalar -> Scalar),
  forall (le_rel : Scalar -> Scalar -> Prop),
  forall (Vector : Sort v),
  forall (vzero : Vector),
  forall (vadd : Vector -> Vector -> Vector),
  forall (vneg : Vector -> Vector),
  forall (smul : Scalar -> Vector -> Vector),
  forall (inner : Vector -> Vector -> Scalar),
  forall (PointCarrier : Sort p),
  forall (disp_op : PointCarrier -> PointCarrier -> Vector),
  @RingLawArgs.{u} Scalar zero one add neg sub mul ->
  @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul ->
  @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner ->
  @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner PointCarrier disp_op ->
  forall (A : PointCarrier),
  forall (B : PointCarrier),
  forall (C : PointCarrier),
  @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C ->
  @Eq.{u} Scalar
    (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C)
    (add
      (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)
      (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))
```

Required imported packages for this target are:

| Package | Required declarations |
| --- | --- |
| `Std.Logic.Eq` / `Proofs.Ai.EqReasoning` | equality, transitivity, congruence, substitution, and transport for rewriting |
| `Proofs.Ai.Algebra.AbstractRing` | `RingLawArgs`, `two`, `sq`, additive/multiplicative ring rewrites |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | existing square-normalization theorem targets from P19 |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | checked scalar zero-cross-term derivations from P27 |
| `Proofs.Ai.Vector.AbstractSpace` | `VectorSpaceLawArgs`, `vsub`, vector additive rewrites |
| `Proofs.Ai.Vector.AbstractInnerProduct` | `InnerProductLawArgs`, `dot`, `normSq`, `PerpVec` |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | checked P28 norm expansion and perpendicular norm theorem from `InnerProductLawArgs`, `PerpVec`, and P27 scalar rewrites |
| `Proofs.Ai.Geometry.Affine` | `AffineLawArgs`, `disp`, `distSqPoints` |
| `Proofs.Ai.Geometry.AffineDerive` | checked P29 hypotenuse orientation and point-distance/norm bridge from primitive affine/vector law packages |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | `RightTriangle`, `Perp` |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | checked P30 bridge from `RightTriangle A B C` to the P28 `PerpVec (vneg (disp A B)) (disp A C)` premise, plus a small bridge that packages it with P29's additive hypotenuse orientation |

### P27 Scalar Algebra Derivation Layer

- Status: Completed
- Depends on: P26
- Inputs: `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractSquareNormalize`,
  `Std.Logic.Eq`
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

#### P27 Result

Implemented `Proofs.Ai.Algebra.AbstractScalarDerive` as a checked derivation layer over
`RingLawArgs`, `Proofs.Ai.Algebra.AbstractSquareNormalize`, and `Std.Logic.Eq` equality transport.
It provides:

- `mul_two_zero_term_from_ring_args`
- `cancel_double_zero_term_from_ring_args`
- `normalize_add_with_zero_cross_term_from_ring_args`

The cross-term theorem has the required shape
`x = 0 -> add (add a (mul two x)) b = add a b` and does not accept a direct
`normalize_add_with_zero_cross_term_law` argument.

### P28 Inner-Product Norm Expansion From Laws

- Status: Completed
- Depends on: P27
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Algebra.AbstractSquareNormalize`,
  `Proofs.Ai.Algebra.AbstractScalarDerive`
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

#### P28 Result

Implemented `Proofs.Ai.Vector.AbstractInnerProductDerive` as a checked derivation layer over
`InnerProductLawArgs`, `Proofs.Ai.Algebra.AbstractScalarDerive`, and `Std.Logic.Eq` equality
transport. It provides:

- `norm_sq_add_from_inner_args`
- `norm_sq_add_of_dot_zero_from_args`
- `norm_sq_add_of_perp_from_args`

The perpendicular theorem has the required shape
`PerpVec x y -> normSq (vadd x y) = add (normSq x) (normSq y)` after the shared scalar/vector
parameters and law packages. The P28 theorem proof terms do not accept
`norm_sq_add_of_perp_law` or `norm_sq_add_of_dot_zero_law` as direct arguments; the final step uses
the P27 `normalize_add_with_zero_cross_term_from_ring_args` theorem to cancel the zero cross term.

### P29 Affine Hypotenuse Vector Derivation

- Status: Completed
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

#### P29 Result

Implemented `Proofs.Ai.Geometry.AffineDerive` as a checked derivation layer over primitive
`AffineLawArgs`, `VectorSpaceLawArgs`, and `Std.Logic.Eq` equality transport. It provides:

- `vec_add_comm_from_vector_args`
- `disp_reverse_from_affine_args`
- `disp_comp_from_affine_args`
- `dist_sq_points_def_from_args`
- `hypotenuse_vector_eq_neg_left_add_right_from_args`
- `hypotenuse_vector_eq_sub_legs_from_args`
- `dist_sq_hypotenuse_norm_neg_left_add_right_from_args`
- `dist_sq_hypotenuse_norm_sub_legs_from_args`

The P29 route proves both additive and subtraction orientations for the hypotenuse displacement:
`disp B C = vadd (vneg (disp A B)) (disp A C)` and
`disp B C = vsub (disp A C) (disp A B)`. It also rewrites `distSqPoints B C` to the corresponding
`normSq` forms. The proof terms do not accept `hypotenuse_vector_eq_sub_legs_law` or
`dist_sq_points_def_law`; the direct theorem-shaped fields were removed from `AffineLawArgs`, while
the legacy standalone theorem target remains available as a direct wrapper for compatibility.

### P30 Right-Triangle Perpendicular Bridge

- Status: Completed
- Depends on: P28, P29
- Inputs: `Proofs.Ai.Geometry.AbstractRightTriangle`,
  `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Geometry.AffineDerive`
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

#### P30 Result

Implemented `Proofs.Ai.Geometry.AbstractRightTriangleDerive` as a checked bridge layer over
`Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Vector.AbstractInnerProduct`, P29's affine
orientation module, and `Std.Logic.Eq` equality transport. It provides:

- `neg_zero_from_ring_args`
- `dot_neg_left_from_inner_args`
- `right_triangle_legs_perp_vec_from_rt`
- `right_triangle_legs_dot_zero_from_rt`
- `right_triangle_neg_left_dot_zero_from_rt`
- `right_triangle_neg_left_perp_vec_from_rt`
- `right_triangle_affine_additive_perp_bridge_from_rt`

The final bridge theorem has the P29-compatible orientation
`RightTriangle A B C -> PerpVec (vneg (disp A B)) (disp A C)` after the shared scalar, vector,
point, and law-package parameters. The direct `RightTriangle -> PerpVec (disp A B) (disp A C)` and
dot-zero bridges close by unfolding `RightTriangle`, `Perp`, and `PerpVec`; the negated-left
orientation additionally uses the primitive `dot_neg_left` field from `InnerProductLawArgs` and the
checked `-0 = 0` scalar helper. The final packaged bridge also invokes P29's
`hypotenuse_vector_eq_neg_left_add_right_from_args` so P31 can consume the hypotenuse orientation
and perpendicular premise together. No P30 theorem accepts a theorem argument whose conclusion is
the Pythagorean equality.

### P31 Squared Pythagorean Proof From Law Packages

- Status: Completed
- Depends on: P27, P28, P29, P30
- Inputs: `Proofs.Ai.Geometry.AbstractRightTriangle`,
  `Proofs.Ai.Geometry.AbstractRightTriangleDerive`, `Proofs.Ai.Geometry.Pythagorean`
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
  - The generated module verifies without non-equality unchecked axioms; `Eq.rec` may appear through
    the existing equality-transport reports where applicable.
  - Existing public theorem names remain available.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `jq '.axioms, .declarations' proofs/Proofs/Ai/Geometry/Pythagorean/meta.json`
  - `rg -n "forall \(law : forall .*forall \(h : @RightTriangle.*@Eq\.\{u\} Scalar \(@distSqPoints|pythagorean_distance_sq_general" proofs/Proofs/Ai/Geometry/Pythagorean/source.npa`

#### P31 Result

Implemented the checked squared-distance theorem in `Proofs.Ai.Geometry.Pythagorean` as
`pythagorean_distance_sq_from_law_packages`. The theorem takes the explicit law packages
`RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, and `AffineLawArgs`, plus
`RightTriangle A B C`, and proves:

```text
distSqPoints B C = add (distSqPoints A B) (distSqPoints A C)
```

The proof composes the prior derivation layers:

- P29 rewrites the hypotenuse point distance to the norm of
  `vadd (vneg (disp A B)) (disp A C)`.
- P30 supplies the matching perpendicular premise from `RightTriangle A B C`.
- P28 derives the perpendicular norm-addition equality from `InnerProductLawArgs` and P27 scalar
  zero-cross-term cancellation.
- Small P31 bridge lemmas use affine distance symmetry, displacement reversal, and `EqReasoning` to
  identify `normSq (vneg (disp A B))` with `distSqPoints A B`.

`pythagorean_theorem_sq`, `law_of_cosines_right_angle_specialization`, and
`pythagorean_theorem_api_alias` now delegate to this checked derivation instead of accepting a
direct theorem-shaped Pythagorean equality law. The module axiom report is `["Eq.rec"]`, inherited
through the imported equality-reasoning/transport lemmas used to compose the checked derivations.
The squared metric-distance theorem is completed by P32; the converse remains a later-layer target
for P34.

### P32 Metric Squared-Distance Bridge Without Direct Metric Law

- Status: Completed
- Depends on: P31
- Inputs: `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.Pythagorean`
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
- Result:
  - `MetricSpaceLawArgs` no longer contains the direct square-distance bridge or direct metric
    Pythagorean fields.
  - `dist_sq_eq_square_dist` is backed by `OrderedFieldLawArgs.sqrt_sq_law` plus
    `InnerProductLawArgs.norm_sq_nonneg_law` through `point_dist_sq_nonneg_from_inner_args`.
  - `pythagorean_theorem_dist_sq` composes the P31 squared-distance theorem with the P32 metric
    bridge instead of accepting a theorem-shaped metric Pythagorean law.

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
  - `pythagorean_theorem_dist_sq` is completed through P32's bridge and no longer presented as a
    later metric-layer target.
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
