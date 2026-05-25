# Law Of Cosines Proof Phase Breakdown

Source: `proofs/README.md`, `proofs/pythagorean-proof-phases.md`

This task breakdown starts after P34, where the squared-distance Pythagorean theorem and squared
metric-distance theorem are checked by certificates. The next goal is to prove the squared
law-of-cosines theorem without accepting a direct theorem-shaped law whose conclusion is already the
law of cosines.

## Scope

The main theorem target is the squared point-distance law of cosines over the existing abstract
Euclidean law packages:

```text
distSqPoints B C
  = (distSqPoints A B + distSqPoints A C)
      - 2 * dot (disp A B) (disp A C)
```

The intended proof route is the same certificate-first route used for P31:

```text
distSqPoints B C
  = normSq (disp B C)
  = normSq (vadd (vneg (disp A B)) (disp A C))
  = normSq (vneg (disp A B))
      + 2 * dot (vneg (disp A B)) (disp A C)
      + normSq (disp A C)
  = distSqPoints A B + distSqPoints A C
      - 2 * dot (disp A B) (disp A C)
```

Important constraints:

```text
- Do not add unchecked Euclidean, field, vector-space, metric, or square-root axioms.
- Keep `crates/` pure NPA unless a proof-corpus tooling gap blocks certificate generation.
- Keep proof-corpus generation under `tools/proof-corpus` and checked artifacts under `proofs/`.
- Treat source, generator, tactics, and AI output as untrusted; acceptance comes from certificates.
- The final proof path must not accept `law_of_cosines_general` or any equivalent direct law.
- Unsquared distance, angle-cosine, and trigonometric cosine statements are out of scope.
```

## Milestones

### LC1 Direct-Law Audit And Target Shape

- Status: Completed
- Depends on: P34
- Inputs: `proofs/README.md`, `proofs/pythagorean-proof-phases.md`,
  `proofs/Proofs/Ai/Geometry/AbstractRightTriangle/source.npa`,
  `proofs/Proofs/Ai/Geometry/Pythagorean/source.npa`, `tools/proof-corpus/src/main.rs`
- Deliverables:
  - A short audit section in this document listing every remaining direct law-of-cosines wrapper.
  - A frozen target signature for `law_of_cosines_sq_from_law_packages`.
- Acceptance criteria:
  - The target takes `RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, and `AffineLawArgs`.
  - The target does not take `law`, `law_of_cosines_general`, or any theorem-shaped argument whose
    conclusion is the law-of-cosines equality.
  - The theorem statement uses squared point distance and the existing dot-product correction term;
    it does not introduce square roots or trigonometric cosine.
- Verification:
  - `rg -n "law_of_cosines|law : forall .*distSqPoints|cosines.*law" proofs tools/proof-corpus`
  - `git diff --check`

#### LC1 Audit Result

The current corpus has one direct abstract law-of-cosines wrapper and several related non-blocking
hits:

| Area | Current item | Direct law-of-cosines dependency? | LC follow-up |
| --- | --- | --- | --- |
| Concrete singleton geometry | `Proofs.Ai.Geometry.RightTriangle.law_of_cosines` | No. It is a checked concrete singleton theorem closing by `Eq.refl` over the old `Vec` / `RingElem` layer and takes no `law` argument. | Keep as an older checked corpus theorem; do not use it as the abstract LC proof. |
| Abstract right-triangle geometry | `Proofs.Ai.Geometry.AbstractRightTriangle.law_of_cosines_general` | Yes. It takes `law : forall A B C, ...` whose conclusion is exactly the abstract law-of-cosines equality. | Replace the final proof path with `law_of_cosines_sq_from_law_packages`; LC5 must not delegate to this wrapper. |
| Final Pythagorean API | `Proofs.Ai.Geometry.Pythagorean.law_of_cosines_right_angle_specialization` | No. It currently delegates to the checked P31 `pythagorean_theorem_sq` and states the right-angle Pythagorean specialization, not the general law of cosines. | LC6 may reconnect it to the checked law-of-cosines route, but LC1 does not treat it as a direct law-of-cosines dependency. |
| Manifest, replay, generator, and generator tests | `law_of_cosines`, `law_of_cosines_general`, `law_of_cosines_right_angle_specialization` declaration entries | No. These entries mirror the declarations above and are not additional proof paths. | Keep expected names until the relevant implementation milestone changes them. |
| Non-cosine distance wrappers | Generic `law : forall ... distSqPoints ...` hits such as affine symmetry and zero-distance equivalence wrappers | No. These are direct wrappers for their own affine facts, not the law-of-cosines equality. | Leave to their own future audits; they are outside LC1 unless a later LC proof path depends on the target equality directly. |

The LC1 replacement target below is therefore frozen as the first abstract law-of-cosines theorem
that must avoid any direct law-of-cosines argument.

#### LC1 Target Shape

The intended checked theorem is:

```text
theorem law_of_cosines_sq_from_law_packages.{p,u,v} :
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
  @Eq.{u} Scalar
    (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C)
    (sub
      (add
        (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)
        (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))
      (mul
        (@two.{u} Scalar one add)
        (@dot.{u,v} Scalar Vector inner
          (@disp.{p,v} PointCarrier Vector disp_op A B)
          (@disp.{p,v} PointCarrier Vector disp_op A C))))
```

### LC2 Scalar Correction-Term Normalization

- Status: Completed
- Depends on: LC1
- Inputs: `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractScalarDerive`,
  `Proofs.Ai.EqReasoning`
- Deliverables:
  - Checked scalar lemmas deriving the correction-term rewrite from `RingLawArgs`.
  - Suggested theorem targets:
    - `mul_two_neg_from_ring_args`
    - `add_neg_cross_term_to_sub_sum_from_ring_args`
    - `law_of_cosines_scalar_rhs_from_ring_args`
- Acceptance criteria:
  - The proof does not accept direct scalar normalization laws beyond `RingLawArgs`.
  - The final scalar lemma rewrites the norm-expansion RHS into the LC1 target RHS:
    `add (add a (mul two (neg x))) b = sub (add a b) (mul two x)`.
  - Any `Eq.rec` use is inherited from equality transport and appears in the module axiom report as
    the documented equality-recursion exception only.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "law_of_cosines_scalar_rhs.*law|add_neg_cross_term.*law|mul_two_neg.*law" proofs tools/proof-corpus`

#### LC2 Result

`Proofs.Ai.Algebra.AbstractScalarDerive` now contains checked scalar correction-term lemmas:

| Theorem | Role |
| --- | --- |
| `mul_two_neg_from_ring_args` | Derives `mul two (neg x) = neg (mul two x)` from `RingLawArgs` by distributivity, additive cancellation, and `mul_zero`. |
| `add_neg_cross_term_to_sub_sum_from_ring_args` | Reassociates and commutes `add (add a (neg t)) b`, then folds it to `sub (add a b) t` via `sub_eq_add_neg`. |
| `law_of_cosines_scalar_rhs_from_ring_args` | Composes the two lemmas to rewrite `add (add a (mul two (neg x))) b` to `sub (add a b) (mul two x)`. |

The proof path takes only `RingLawArgs` as its scalar law package. The module axiom report remains
the documented equality-transport exception `Eq.rec`.

### LC3 Inner-Product Additive Cosine Expansion

- Status: Pending
- Depends on: LC2
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractScalarDerive`
- Deliverables:
  - Checked inner-product theorem targets for the additive hypotenuse orientation.
  - Suggested theorem targets:
    - `norm_sq_neg_from_inner_args`
    - `norm_sq_add_neg_left_from_inner_args`
- Acceptance criteria:
  - `norm_sq_add_neg_left_from_inner_args` derives
    `normSq (vadd (vneg x) y) = sub (add (normSq x) (normSq y)) (mul two (dot x y))`.
  - The proof uses `norm_sq_add_from_inner_args`, `dot_neg_left_from_inner_args`, and LC2 scalar
    normalization; it does not accept a direct norm law for the target statement.
  - The theorem remains vector-level and has no point or metric dependency.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "norm_sq_add_neg_left.*law|norm_sq_neg.*law|law_of_cosines.*law" proofs/Proofs/Ai/Vector tools/proof-corpus/src/main.rs`

### LC4 Affine Distance Bridge For Cosines

- Status: Pending
- Depends on: LC3
- Inputs: `Proofs.Ai.Geometry.Pythagorean`, `Proofs.Ai.Geometry.Affine`,
  `Proofs.Ai.Geometry.AffineDerive`, `Proofs.Ai.Vector.AbstractInnerProductDerive`
- Deliverables:
  - Checked point-level bridge from `distSqPoints B C` to the LC3 vector expansion.
  - Suggested theorem target:
    - `dist_sq_law_of_cosines_rhs_from_law_packages`
- Acceptance criteria:
  - The proof uses the existing P29 additive hypotenuse orientation
    `hypotenuse_vector_eq_neg_left_add_right_from_args`.
  - Any exposed `normSq (vneg (disp A B))` intermediate is eliminated through checked LC3, affine,
    or symmetry bridges, not through a direct norm-negation axiom.
  - The proof does not depend on `RightTriangle`; the law of cosines is for arbitrary points.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "law_of_cosines.*law|hypotenuse_vector_eq_sub_legs_law|dist_sq_points_def_law" proofs/Proofs/Ai/Geometry tools/proof-corpus/src/main.rs`

### LC5 Final Squared Law-Of-Cosines Theorem

- Status: Pending
- Depends on: LC4
- Inputs: `Proofs.Ai.Geometry.Pythagorean`, `Proofs.Ai.Geometry.AffineDerive`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractScalarDerive`
- Deliverables:
  - Checked public theorem `law_of_cosines_sq_from_law_packages`.
  - A public alias, if useful, that preserves existing naming expectations without using the direct
    `law_of_cosines_general` wrapper.
- Acceptance criteria:
  - The final proof path takes only law packages and point variables.
  - The final proof path does not take a direct theorem-shaped law-of-cosines argument.
  - The module axiom report remains empty except for the documented `Eq.rec` equality-transport
    exception already used by the Pythagorean module.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "law_of_cosines_sq_from_law_packages|law_of_cosines_general|law : forall .*distSqPoints" proofs/Proofs/Ai/Geometry/Pythagorean tools/proof-corpus/src/main.rs`
  - Targeted review of `proofs/manifest.toml` for imports, theorem names, and axiom report.

### LC6 Right-Angle Specialization Cross-Check

- Status: Pending
- Depends on: LC5, P30, P31
- Inputs: `Proofs.Ai.Geometry.Pythagorean`,
  `Proofs.Ai.Geometry.AbstractRightTriangleDerive`, `Proofs.Ai.Algebra.AbstractScalarDerive`
- Deliverables:
  - A checked theorem showing that the law of cosines specializes to the existing squared
    Pythagorean theorem when `RightTriangle A B C` supplies the zero dot-product premise.
  - The existing `law_of_cosines_right_angle_specialization` should either delegate to this checked
    specialization or remain clearly documented as a Pythagorean alias if the compatibility cost is
    too high.
- Acceptance criteria:
  - The proof uses `right_triangle_legs_dot_zero_from_rt` or an equivalent checked derivation.
  - The proof uses scalar zero-cross-term normalization from checked law packages.
  - The specialization does not become the primary proof of P31 unless the document explicitly
    records that public dependency change.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "law_of_cosines_right_angle_specialization|right_triangle_legs_dot_zero|law_of_cosines.*law" proofs/Proofs/Ai/Geometry/Pythagorean tools/proof-corpus/src/main.rs`

### LC7 Squared Metric-Distance Law Of Cosines

- Status: Pending
- Depends on: LC5, P32
- Inputs: `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.Pythagorean`
- Deliverables:
  - Checked squared metric-distance theorem target:
    `sq (dist B C) = sub (add (sq (dist A B)) (sq (dist A C))) (mul two (dot (disp A B) (disp A C)))`.
- Acceptance criteria:
  - The proof composes LC5 with the P32 metric square bridge.
  - The proof does not introduce unsquared `dist B C = ...` or square-root cancellation claims.
  - Any remaining metric assumptions are primitive metric or ordered-field law package fields, not a
    direct metric law-of-cosines theorem.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - Targeted source review for direct metric law-of-cosines or square-root cancellation assumptions.

### LC8 Public Documentation And API Refresh

- Status: Pending
- Depends on: LC5
- Inputs: `proofs/README.md`, `proofs/manifest.toml`,
  `proofs/Proofs/Ai/Geometry/Pythagorean/meta.json`, this document
- Deliverables:
  - README documentation that distinguishes completed squared law-of-cosines results from optional
    metric, right-angle, unsquared, and trigonometric variants. If LC6 or LC7 is not complete yet,
    those variants must be explicitly marked as pending.
  - Manifest and metadata review notes confirming that no direct law-of-cosines law remains on the
    final proof path.
  - Status updates in this phase document for completed law-of-cosines milestones.
- Acceptance criteria:
  - README theorem tables identify the final checked theorem names and their law-package
    dependencies.
  - Any legacy direct wrapper remains clearly marked as a target or compatibility wrapper, not the
    completed checked theorem.
  - The document does not claim completion of unsquared distance or angle-cosine statements.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "direct law-of-cosines|law_of_cosines.*law|unsquared|angle-cosine" proofs/README.md proofs/law-of-cosines-proof-phases.md proofs/Proofs/Ai/Geometry/Pythagorean tools/proof-corpus/src/main.rs`

## Completion Definition

The squared law-of-cosines theorem is complete when all of the following hold:

```text
law_of_cosines_sq_from_law_packages
  no longer takes a direct law-of-cosines equality law,
  depends only on checked law packages and prior checked lemmas,
  verifies as a canonical certificate,
  has a reviewed axiom report that adds no non-equality unchecked axiom,
  and is documented as completed in `proofs/README.md`.
```

The metric squared-distance variant is a follow-on milestone. Unsquared distance and trigonometric
angle-cosine statements remain out of scope until square-root cancellation, angle measure, and
cosine APIs are available as checked certificate-backed components.
