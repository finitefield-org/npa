# Inner Product To Metric Proof Phase Breakdown

Source: `proofs/README.md`, `proofs/pythagorean-proof-phases.md`,
`proofs/law-of-cosines-proof-phases.md`

This task breakdown starts after LC8, where the squared law of cosines is checked by certificates.
The next goal is to grow the abstract inner-product and metric theorem stack in the following order:

```text
parallelogram law
  -> polarization identity
  -> Cauchy-Schwarz inequality
  -> triangle inequality
```

The intended route keeps the same certificate-first boundary as the Pythagorean and law-of-cosines
work. Existing theorem-shaped wrappers may remain as older targets, but completed theorem names in
this plan must be backed by checked proof paths that do not simply accept the target theorem as a
law argument.

## Scope

Main theorem targets:

```text
parallelogram_law_from_inner_args
  normSq (x + y) + normSq (x - y)
    = 2 * normSq x + 2 * normSq y

polarization_identity_from_inner_args
  2 * dot x y
    = normSq (x + y) - (normSq x + normSq y)

cauchy_schwarz_from_law_packages
  sq (dot x y) <= normSq x * normSq y

triangle_inequality_from_law_packages
  dist A C <= dist A B + dist B C
```

Important constraints:

```text
- Do not add unchecked field, order, inner-product, metric, or square-root axioms.
- Keep `crates/` pure NPA unless a proof-corpus tooling gap blocks certificate generation.
- Keep proof-corpus generation under `tools/proof-corpus` and checked artifacts under `proofs/`.
- Treat source, generator, tactics, and AI output as untrusted; acceptance comes from certificates.
- Do not use `parallelogram_law_law`, `polarization_identity_law`,
  `cauchy_schwarz_law`, or `triangle_inequality_law` on the final completed proof paths.
- Prefer squared statements before square-root distance statements. The metric triangle inequality
  must wait for checked square-comparison and nonnegativity support.
```

## Dependency Sketch

```text
IPM1 audit and target freeze
  -> IPM2 scalar parallelogram normalization
  -> IPM3 checked parallelogram law
  -> IPM4 scalar polarization normalization
  -> IPM5 checked polarization identity
  -> IPM6 inner-product API refresh
  -> IPM7 Cauchy-Schwarz prerequisite audit
  -> IPM8 ordered-field quadratic support
  -> IPM9 Cauchy-Schwarz degenerate cases
  -> IPM10 checked Cauchy-Schwarz
  -> IPM11 metric square-comparison support
  -> IPM12 squared Minkowski bound
  -> IPM13 checked triangle inequality
  -> IPM14 public documentation and API refresh
```

## Milestones

### IPM1 Direct-Law Audit And Target Shape

- Status: Completed
- Depends on: LC8
- Inputs: `proofs/README.md`, `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Geometry.AbstractMetric`,
  `tools/proof-corpus/src/main.rs`
- Deliverables:
  - A short audit section in this document listing remaining direct theorem-shaped wrappers for
    parallelogram, polarization, Cauchy-Schwarz, and triangle inequality.
  - Frozen target signatures for the four `_from_*` checked theorem names in this plan.
- Acceptance criteria:
  - The target signatures use existing law packages where possible:
    `RingLawArgs`, `OrderedFieldLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`,
    and `AffineLawArgs`.
  - The targets do not take theorem-shaped law arguments whose conclusions are exactly the target
    theorem statements.
  - The Cauchy-Schwarz and triangle-inequality targets explicitly record any missing scalar/order
    prerequisites instead of hiding them inside direct theorem laws.
- Verification:
  - `rg -n "parallelogram|polarization|cauchy_schwarz|triangle_inequality" proofs tools/proof-corpus`
  - `git diff --check`

#### IPM1 Audit Result

The current corpus has direct theorem-shaped wrappers for all four public goals, plus older
singleton-carrier concrete theorem targets. They are acceptable as older targets or law-package
fields, but none of them may be used as the final completed proof path for this sequence.

| Area | Current item | Direct theorem dependency? | IPM follow-up |
| --- | --- | --- | --- |
| Concrete vector dot corpus | `Proofs.Ai.Vector.Dot.parallelogram_law`, `Proofs.Ai.Vector.Dot.polarization_identity` | No direct law argument. They are checked singleton-carrier theorems closing by the concrete corpus, not abstract law-package derivations. | Keep as older concrete corpus targets; do not use them as abstract IPM proof dependencies. |
| Concrete metric corpus | `Proofs.Ai.Geometry.Metric.cauchy_schwarz`, `Proofs.Ai.Geometry.Metric.triangle_inequality` | No direct law argument. They are checked singleton-carrier metric/order targets and do not establish the abstract law-package derivations. | Keep as older concrete corpus targets; do not use them as abstract IPM proof dependencies. |
| Abstract inner-product law package | `InnerProductLawArgs` fields `parallelogram_law_law`, `polarization_identity_law`, and `cauchy_schwarz_law` | Yes. Each field states exactly the corresponding target theorem. | Later `_from_inner_args` / `_from_law_packages` proofs must avoid projecting these fields. |
| Abstract inner-product wrappers | `Proofs.Ai.Vector.AbstractInnerProduct.parallelogram_law`, `polarization_identity`, `cauchy_schwarz` | Yes. Each theorem takes a `law` argument whose conclusion is exactly the theorem. | Keep as legacy target/compatibility wrappers; replace final checked route with derived theorem names. |
| Abstract inner-product derivations | `Proofs.Ai.Vector.AbstractInnerProductDerive` helper destructuring of `InnerProductLawArgs` | No by itself. Existing completed helpers destructure the package and ignore the parallelogram, polarization, and Cauchy-Schwarz fields. | New proofs must continue to avoid using `parallelogram_arg`, `polarization_identity_arg`, and `cauchy_schwarz_arg`. |
| Abstract metric law package | `MetricSpaceLawArgs` field `triangle_inequality_law` | Yes. The field states exactly the metric triangle inequality. | `triangle_inequality_from_law_packages` must not take `MetricSpaceLawArgs` unless that package is later split so the direct triangle field is outside the trusted path. |
| Abstract metric wrapper | `Proofs.Ai.Geometry.AbstractMetric.triangle_inequality` | Yes. It takes a direct `law` argument and returns it at `A B C`. | Keep as a legacy target/compatibility wrapper; final proof must use Cauchy-Schwarz, affine composition, and ordered-field square comparison. |

Additional prerequisite notes:

- Cauchy-Schwarz is not just missing scalar/order algebra. The present `InnerProductLawArgs` has
  additivity, negation, subtraction, norm expansion, positivity, and direct Cauchy-Schwarz fields,
  but it does not expose scalar-linearity of the inner product such as `dot (smul a x) y = a *
  dot x y`. IPM7 must decide whether the chosen checked route can avoid that fact; otherwise it
  must add a generic inner-product scalar-linearity field rather than a theorem-shaped
  Cauchy-Schwarz field.
- The metric triangle inequality needs generic ordered-field square-comparison support, including a
  path from nonnegative `a`, nonnegative `b`, and `sq a <= sq b` to `a <= b`. IPM11 owns that
  support. This must remain scalar/order-only and must not mention vectors, metric distance, or
  triangle inequality.

#### IPM1 Frozen Target Shapes

The common vector parameters used below are:

```text
forall (Scalar : Sort u),
forall (zero : Scalar),
forall (one : Scalar),
forall (add : forall (a : Scalar), forall (b : Scalar), Scalar),
forall (neg : forall (a : Scalar), Scalar),
forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar),
forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar),
forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop),
forall (Vector : Sort v),
forall (vzero : Vector),
forall (vadd : forall (x : Vector), forall (y : Vector), Vector),
forall (vneg : forall (x : Vector), Vector),
forall (smul : forall (a : Scalar), forall (x : Vector), Vector),
forall (inner : forall (x : Vector), forall (y : Vector), Scalar),
```

The checked parallelogram target is:

```text
theorem parallelogram_law_from_inner_args.{u,v} :
  <common vector parameters>
  @RingLawArgs.{u} Scalar zero one add neg sub mul ->
  @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner ->
  forall (x : Vector),
  forall (y : Vector),
  @Eq.{u} Scalar
    (add
      (@normSq.{u,v} Scalar Vector inner (vadd x y))
      (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)))
    (add
      (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x))
      (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))
```

The checked polarization target is:

```text
theorem polarization_identity_from_inner_args.{u,v} :
  <common vector parameters>
  @RingLawArgs.{u} Scalar zero one add neg sub mul ->
  @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner ->
  forall (x : Vector),
  forall (y : Vector),
  @Eq.{u} Scalar
    (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))
    (sub
      (@normSq.{u,v} Scalar Vector inner (vadd x y))
      (add
        (@normSq.{u,v} Scalar Vector inner x)
        (@normSq.{u,v} Scalar Vector inner y)))
```

The checked Cauchy-Schwarz target is:

```text
theorem cauchy_schwarz_from_law_packages.{u,v} :
  forall (Scalar : Sort u),
  forall (zero : Scalar),
  forall (one : Scalar),
  forall (add : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (neg : forall (a : Scalar), Scalar),
  forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop),
  forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop),
  forall (sqrt_fn : forall (a : Scalar), Scalar),
  forall (Vector : Sort v),
  forall (vzero : Vector),
  forall (vadd : forall (x : Vector), forall (y : Vector), Vector),
  forall (vneg : forall (x : Vector), Vector),
  forall (smul : forall (a : Scalar), forall (x : Vector), Vector),
  forall (inner : forall (x : Vector), forall (y : Vector), Scalar),
  @RingLawArgs.{u} Scalar zero one add neg sub mul ->
  @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ->
  @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul ->
  @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner ->
  forall (x : Vector),
  forall (y : Vector),
  le_rel
    (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y))
    (mul
      (@normSq.{u,v} Scalar Vector inner x)
      (@normSq.{u,v} Scalar Vector inner y))
```

The checked metric triangle target is:

```text
theorem triangle_inequality_from_law_packages.{p,u,v} :
  forall (Scalar : Sort u),
  forall (zero : Scalar),
  forall (one : Scalar),
  forall (add : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (neg : forall (a : Scalar), Scalar),
  forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar),
  forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop),
  forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop),
  forall (sqrt_fn : forall (a : Scalar), Scalar),
  forall (Vector : Sort v),
  forall (vzero : Vector),
  forall (vadd : forall (x : Vector), forall (y : Vector), Vector),
  forall (vneg : forall (x : Vector), Vector),
  forall (smul : forall (a : Scalar), forall (x : Vector), Vector),
  forall (inner : forall (x : Vector), forall (y : Vector), Scalar),
  forall (PointCarrier : Sort p),
  forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector),
  @RingLawArgs.{u} Scalar zero one add neg sub mul ->
  @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ->
  @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul ->
  @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner ->
  @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel
    Vector vzero vadd vneg smul inner PointCarrier disp_op ->
  forall (A : PointCarrier),
  forall (B : PointCarrier),
  forall (C : PointCarrier),
  le_rel
    (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)
    (add
      (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)
      (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C))
```

These signatures intentionally avoid `MetricSpaceLawArgs`, `parallelogram_law_law`,
`polarization_identity_law`, `cauchy_schwarz_law`, and `triangle_inequality_law`. Any extra support
needed by IPM7-IPM13 must be added as generic scalar/order, vector-space, affine, or inner-product
API and then used as checked imported theorem support, not as direct target theorem arguments.

### IPM2 Scalar Normalization For Parallelogram

- Status: Completed
- Depends on: IPM1
- Inputs: `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractScalarDerive`,
  `Proofs.Ai.EqReasoning`
- Deliverables:
  - Checked scalar lemmas deriving the cancellation of opposite cross terms from `RingLawArgs`.
  - Suggested theorem targets:
    - `add_cross_and_sub_cross_cancel_from_ring_args`
    - `parallelogram_scalar_rhs_from_ring_args`
- Acceptance criteria:
  - The proof uses only `RingLawArgs` and previously checked scalar/equality lemmas.
  - The final scalar lemma rewrites the sum of the `normSq (x + y)` and `normSq (x - y)`
    expansions into `2 * normSq x + 2 * normSq y`.
  - No vector, inner-product, parallelogram, or polarization theorem law is accepted.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "parallelogram_scalar|cross.*cancel|parallelogram_arg" proofs tools/proof-corpus`

#### IPM2 Result

`Proofs.Ai.Algebra.AbstractScalarDerive` now includes the checked scalar-only path needed by IPM3:

| Theorem | Purpose |
| --- | --- |
| `two_mul_from_ring_args` | Derives `2 * a = a + a` from `RingLawArgs`. |
| `add_sub_cross_cancel_from_ring_args` | Cancels one opposite cross term: `x + (a - x) = a`. |
| `add_pairwise_commute_from_ring_args` | Reassociates `(a + b) + (c + d)` into `(a + c) + (b + d)`. |
| `add_cross_and_sub_cross_cancel_from_ring_args` | Turns the two norm-expansion scalar summands into `(a + a) + (b + b)`. |
| `parallelogram_scalar_rhs_from_ring_args` | Rewrites `(a + x + b) + (a - x + b)` into `2 * a + 2 * b`. |

The generated certificates verify these lemmas using `RingLawArgs` plus checked equality
composition from `Proofs.Ai.EqReasoning`. The scalar module does not import vector,
inner-product, parallelogram, or polarization theorem laws. The `parallelogram_arg` matches in the
verification search remain the pre-existing vector-level law-package binders and are not referenced
by the new scalar proof path.

### IPM3 Checked Parallelogram Law

- Status: Completed
- Depends on: IPM2
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractScalarDerive`
- Deliverables:
  - Checked inner-product theorem targets for the parallelogram route.
  - Suggested theorem targets:
    - `norm_sq_sub_from_inner_args`
    - `parallelogram_law_from_inner_args`
- Acceptance criteria:
  - The proof combines `norm_sq_add_from_inner_args`, `norm_sq_sub_from_inner_args`, and the IPM2
    scalar normalization.
  - The proof does not project or accept `parallelogram_arg` / `parallelogram_law_law`.
  - The theorem remains vector-level and has no point, metric, or square-root dependency.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "parallelogram_law_from_inner_args|parallelogram_arg|parallelogram_law_law" proofs/Proofs/Ai/Vector tools/proof-corpus/src/main.rs`

#### IPM3 Result

`Proofs.Ai.Vector.AbstractInnerProductDerive` now includes the checked vector-level
parallelogram route:

| Theorem | Purpose |
| --- | --- |
| `norm_sq_sub_from_inner_args` | Projects the primitive `normSq (x - y)` expansion from `InnerProductLawArgs`. |
| `parallelogram_law_from_inner_args` | Combines `norm_sq_add_from_inner_args`, `norm_sq_sub_from_inner_args`, and `parallelogram_scalar_rhs_from_ring_args` to derive the parallelogram identity. |

The final theorem uses `Proofs.Ai.EqReasoning` to compose the add/sub norm expansions with the IPM2
scalar normalization. It does not project `parallelogram_arg` or `parallelogram_law_law`; the
remaining search hits are the law-package field declarations or unused binders in existing package
destructuring. The theorem stays in `Proofs.Ai.Vector.AbstractInnerProductDerive` and has no point,
metric, or square-root dependency.

### IPM4 Scalar Normalization For Polarization

- Status: Pending
- Depends on: IPM3
- Inputs: `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractScalarDerive`,
  `Proofs.Ai.EqReasoning`
- Deliverables:
  - Checked scalar lemmas turning the norm-addition expansion into the polarization RHS.
  - Suggested theorem target:
    - `polarization_scalar_rhs_from_ring_args`
- Acceptance criteria:
  - The final scalar lemma derives
    `2 * d = (nx + 2 * d + ny) - (nx + ny)` from `RingLawArgs`.
  - The proof does not accept a direct polarization or inner-product theorem law.
  - The lemma is scalar-only and reusable outside the vector module.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "polarization_scalar|polarization_arg|polarization_identity_law" proofs tools/proof-corpus`

### IPM5 Checked Polarization Identity

- Status: Pending
- Depends on: IPM4
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractScalarDerive`
- Deliverables:
  - Checked public theorem `polarization_identity_from_inner_args`.
- Acceptance criteria:
  - The proof uses `norm_sq_add_from_inner_args` and IPM4 scalar normalization.
  - The proof does not project or accept `polarization_identity_arg` /
    `polarization_identity_law`.
  - The theorem remains vector-level and avoids metric and square-root dependencies.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "polarization_identity_from_inner_args|polarization_identity_arg|polarization_identity_law" proofs/Proofs/Ai/Vector tools/proof-corpus/src/main.rs`

### IPM6 Inner-Product Documentation And API Refresh

- Status: Pending
- Depends on: IPM5
- Inputs: `proofs/README.md`, `proofs/manifest.toml`,
  `proofs/Proofs/Ai/Vector/AbstractInnerProductDerive/meta.json`
- Deliverables:
  - README updates distinguishing checked derived theorem names from older direct theorem targets.
  - Manifest and metadata review notes for the completed parallelogram and polarization exports.
- Acceptance criteria:
  - The README theorem tables identify `parallelogram_law_from_inner_args` and
    `polarization_identity_from_inner_args` as the completed checked theorem names.
  - Legacy direct wrappers are clearly marked as target/compatibility wrappers, not completed
    checked proof paths.
  - The document does not claim Cauchy-Schwarz or triangle inequality completion yet.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "parallelogram_law_from_inner_args|polarization_identity_from_inner_args|cauchy_schwarz|triangle_inequality" proofs/README.md proofs/manifest.toml proofs/Proofs/Ai/Vector/AbstractInnerProductDerive`

### IPM7 Cauchy-Schwarz Prerequisite Audit

- Status: Pending
- Depends on: IPM6
- Inputs: `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, this document
- Deliverables:
  - A short audit section in this document identifying which scalar/order facts are missing for a
    checked Cauchy-Schwarz proof.
  - A frozen route for Cauchy-Schwarz that avoids the direct `cauchy_schwarz_law` field.
- Acceptance criteria:
  - The audit explicitly checks whether inverse/division, square comparison, or quadratic
    minimization support is required.
  - Any new scalar/order support is stated generically over scalars and does not mention vectors,
    dot products, norms, or Cauchy-Schwarz.
  - The planned final theorem remains the squared form
    `sq (dot x y) <= normSq x * normSq y`.
- Verification:
  - `rg -n "cauchy_schwarz|inv|div|square.*le|sqrt|quadratic|norm_sq_zero" proofs tools/proof-corpus`
  - `git diff --check`

### IPM8 Ordered-Field Quadratic Support

- Status: Pending
- Depends on: IPM7
- Inputs: `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Algebra.AbstractRing`
- Deliverables:
  - Checked scalar/order lemmas needed by the IPM7 route.
  - Suggested theorem targets, to be finalized by IPM7:
    - `le_of_square_le_square_nonneg_from_ordered_args`
    - `square_completion_bound_from_ordered_args`
    - optional minimal inverse/division helpers if the audit proves they are necessary.
- Acceptance criteria:
  - The lemmas are scalar/order facts only; they do not mention vectors, inner products, normSq, or
    metric distance.
  - The proof does not introduce a direct Cauchy-Schwarz theorem under another name.
  - Any added order or square-root law package field is documented as a general scalar API with a
    clear trust boundary.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "cauchy_schwarz|dot|normSq|triangle_inequality" proofs/Proofs/Ai/Algebra tools/proof-corpus/src/main.rs`

### IPM9 Cauchy-Schwarz Degenerate Cases

- Status: Pending
- Depends on: IPM8
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractOrderedField`
- Deliverables:
  - Checked helper theorems for zero-norm cases used by Cauchy-Schwarz.
  - Suggested theorem targets:
    - `dot_eq_zero_of_norm_sq_zero_left_from_inner_args`
    - `dot_eq_zero_of_norm_sq_zero_right_from_inner_args`
    - `cauchy_schwarz_zero_left_from_law_packages`
    - `cauchy_schwarz_zero_right_from_law_packages`
- Acceptance criteria:
  - The proofs use positive-definiteness and order facts through `InnerProductLawArgs` and
    `OrderedFieldLawArgs`.
  - The proofs do not project or accept `cauchy_schwarz_arg` / `cauchy_schwarz_law`.
  - The results are reusable by the nonzero case and do not mention metric distance.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "cauchy_schwarz_arg|cauchy_schwarz_law|norm_sq_zero|zero_left|zero_right" proofs/Proofs/Ai/Vector tools/proof-corpus/src/main.rs`

### IPM10 Checked Cauchy-Schwarz Inequality

- Status: Pending
- Depends on: IPM9
- Inputs: `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Algebra.AbstractOrderedField`
- Deliverables:
  - Checked public theorem `cauchy_schwarz_from_law_packages`.
- Acceptance criteria:
  - The proof derives the squared inequality from checked scalar/order support and
    `InnerProductLawArgs`.
  - The proof covers zero and nonzero cases through IPM9 or a documented equivalent route.
  - The proof does not project or accept `cauchy_schwarz_arg` / `cauchy_schwarz_law`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "cauchy_schwarz_from_law_packages|cauchy_schwarz_arg|cauchy_schwarz_law" proofs/Proofs/Ai/Vector tools/proof-corpus/src/main.rs`

### IPM11 Metric Square-Comparison Support

- Status: Pending
- Depends on: IPM10
- Inputs: `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Geometry.AbstractMetric`
- Deliverables:
  - Checked scalar/order square-comparison lemmas required to turn a squared metric bound into an
    unsquared distance inequality.
  - Suggested theorem targets:
    - `le_of_sq_le_sq_nonneg_from_ordered_args`
    - `add_dist_nonneg_from_ordered_args`
    - `sqrt_sum_square_bound_from_ordered_args`
- Acceptance criteria:
  - The lemmas are generic ordered-field and square-root facts.
  - The proof does not mention vectors, affine points, dot products, Cauchy-Schwarz, or triangle
    inequality.
  - The support is enough to prove `a <= b` from `0 <= a`, `0 <= b`, and `sq a <= sq b`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "triangle_inequality|dist A C|dot|normSq" proofs/Proofs/Ai/Algebra tools/proof-corpus/src/main.rs`

### IPM12 Squared Minkowski Bound

- Status: Pending
- Depends on: IPM11
- Inputs: `Proofs.Ai.Vector.AbstractInnerProductDerive`, `Proofs.Ai.Geometry.AffineDerive`,
  `Proofs.Ai.Geometry.AbstractMetric`
- Deliverables:
  - Checked squared bound for the affine displacement path behind triangle inequality.
  - Suggested theorem targets:
    - `norm_sq_add_le_square_sum_norms_from_cauchy`
    - `dist_sq_points_le_square_sum_dist_from_law_packages`
- Acceptance criteria:
  - The proof uses IPM10 Cauchy-Schwarz, IPM11 square-root/order support, and affine displacement
    composition.
  - The proof stays squared; it does not yet export `dist A C <= dist A B + dist B C`.
  - The proof does not project or accept `triangle_inequality_law`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "dist_sq_points_le_square_sum_dist|triangle_inequality_law|cauchy_schwarz_from_law_packages" proofs/Proofs/Ai/Geometry tools/proof-corpus/src/main.rs`

### IPM13 Checked Triangle Inequality

- Status: Pending
- Depends on: IPM12
- Inputs: `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.AffineDerive`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`
- Deliverables:
  - Checked public theorem `triangle_inequality_from_law_packages`.
  - Optional public alias refresh for `triangle_inequality`, if the existing compatibility surface
    can delegate to the checked route without breaking earlier declarations.
- Acceptance criteria:
  - The proof derives the metric inequality from IPM12 squared bound and IPM11 square comparison.
  - The proof uses nonnegativity of `dist A C` and `dist A B + dist B C`.
  - The proof does not project or accept `triangle_inequality_law`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "triangle_inequality_from_law_packages|triangle_inequality_law|dist_sq_points_le_square_sum_dist" proofs/Proofs/Ai/Geometry tools/proof-corpus/src/main.rs`

### IPM14 Public Documentation And API Refresh

- Status: Pending
- Depends on: IPM13
- Inputs: `proofs/README.md`, `proofs/manifest.toml`,
  `proofs/Proofs/Ai/Vector/AbstractInnerProductDerive/meta.json`,
  `proofs/Proofs/Ai/Geometry/AbstractMetric/meta.json`, this document
- Deliverables:
  - README updates distinguishing completed checked results from legacy direct wrappers.
  - Manifest and metadata review notes confirming that no direct theorem-shaped law remains on the
    final completed paths for parallelogram, polarization, Cauchy-Schwarz, or triangle inequality.
  - Status updates in this phase document.
- Acceptance criteria:
  - README theorem tables identify final checked theorem names and law-package dependencies.
  - Legacy direct wrappers are clearly marked target/compatibility wrappers, not completed checked
    theorem paths.
  - The document does not claim angle, trigonometric, or stronger analytic statements beyond the
    checked metric triangle inequality.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "parallelogram_law_from_inner_args|polarization_identity_from_inner_args|cauchy_schwarz_from_law_packages|triangle_inequality_from_law_packages|direct" proofs/README.md proofs/inner-product-to-metric-proof-phases.md proofs/Proofs/Ai`

## Completion Definition

The full sequence is complete when all of the following hold:

```text
parallelogram_law_from_inner_args
polarization_identity_from_inner_args
cauchy_schwarz_from_law_packages
triangle_inequality_from_law_packages
```

are certificate-verified theorem exports, their proof paths do not project the corresponding direct
theorem-shaped law fields, and the README/manifest/metadata explain the final checked theorem names
and remaining compatibility wrappers.
