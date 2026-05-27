# Inverse And Implicit Function Proof Phase Breakdown

This plan records the route for the basic inverse-function and implicit-function theorem track.
The immediate target is a certificate-backed Implicit Function Theorem, built without adding an
unchecked analysis primitive to the kernel.

## Scope

The final theorem target is the standard local implicit-function statement for a map

```text
F : X x Y -> Z
F(a, b) = 0
D_y F(a, b) : Y -> Z is a linear isomorphism
```

Under suitable differentiability and local invertibility assumptions, there are neighborhoods
`U` of `a` and `V` of `b` and a unique function

```text
g : U -> V
```

such that

```text
F(x, g(x)) = 0
```

for all `x : U`.  The strengthened follow-on target proves differentiability of `g` and the
derivative formula

```text
Dg(x) = - (D_y F(x, g(x)))^{-1} o D_x F(x, g(x)).
```

The preferred proof route is through a quantitative inverse function theorem applied to

```text
Phi(x, y) = (x, F(x, y)).
```

This keeps the implicit theorem mostly as a packaging theorem once the inverse-function theorem,
product-space calculus, and block-triangular derivative inverse are available.

## Constraints

```text
- Do not add real-analysis, completeness, differentiability, inverse-map, or fixed-point facts as
  trusted kernel primitives.
- Keep topology, metric, norm, derivative, and contraction facts as ordinary proof-corpus
  definitions/theorems with canonical certificates.
- Keep source, elaboration, tactics, automation, and AI output outside the trusted boundary.
- Prefer explicit law packages over typeclass or implicit instance search.
- Avoid assuming global inverse, global uniqueness, or differentiability of the implicit function
  before the local theorem has constructed them.
```

## Target Modules

The exact module names can be adjusted to fit the corpus, but the intended layering is:

| Module | Role |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | metric balls, neighborhoods, local membership, local predicates/equality/uniqueness |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | normed additive/vector-space laws and product norms |
| `Proofs.Ai.Analysis.AbstractLinearMap` | bounded linear maps, operator norm, linear isomorphisms |
| `Proofs.Ai.Analysis.AbstractDerivative` | Frechet derivative and calculus rules |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | contraction mapping theorem |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | quantitative inverse function theorem |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | auxiliary `Phi(x,y)=(x,F(x,y))` map and block derivative bridge |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | implicit function theorem evidence and final theorem |

## Milestones

### IIF0: Target Shape And Direct-Law Audit

- Status: Initial audit recorded.
- Deliverables:
  - Freeze the target theorem statement for `implicit_function_theorem`.
  - Identify any existing or planned wrappers that already assume the implicit-function conclusion.
  - Decide whether the first theorem is over Banach spaces or over finite-dimensional Euclidean
    spaces with an explicit completeness package.
- Current audit result:
  - Before starting IIF1, the proof corpus had no `Proofs.Ai.Analysis.*` modules and no existing
    implicit-function wrapper.
  - The preferred first route remains Banach-style with explicit completeness and law packages;
    the finite-dimensional route remains the fallback if the completeness API is not ready.
- Acceptance criteria:
  - The target exposes all assumptions explicitly: base point, zero equation, differentiability
    neighborhood, invertibility of `D_y F(a,b)`, and quantitative radius/Lipschitz bounds.
  - No argument has the shape of the final local implicit-function conclusion.

### IIF1: Local Topology API

- Status: Certificate generated for the predicate-level local topology API in
  `Proofs.Ai.Analysis.AbstractMetricTopology`.
- Deliverables:
  - Define balls, neighborhoods, local membership, local uniqueness, and local equality.
  - Prove basic ball monotonicity and neighborhood shrink lemmas.
- Completed exports:
  - Definitions: `MetricBall`, `Neighborhood`, `LocalMem`, `LocalPred`, `LocalEq`,
    `LocalUnique`.
  - Theorems: `metric_ball_intro`, `metric_ball_elim`, `neighborhood_intro`,
    `neighborhood_center`, `neighborhood_shrink`, `local_mem_intro`, `local_mem_elim`,
    `local_pred_intro`, `local_pred_apply`, `local_pred_shrink`, `metric_ball_mono`,
    `local_eq_refl`, `local_eq_symm`, `local_eq_trans`, `local_unique_apply`.
- Needed later for:
  - expressing the domains on which the local inverse and implicit function are valid.

### IIF2: Normed Product Spaces

- Status: Certificate generated for the explicit-law normed-space and product-norm API in
  `Proofs.Ai.Analysis.AbstractNormedSpace`.
- Deliverables:
  - Define product-space operations and product norm.
  - Prove projection and pairing laws for products.
  - Prove enough norm inequalities for product-space estimates.
- Completed exports:
  - Definitions: `NormDist`, `NormedSpaceLawArgs`, `ProductZero`, `ProductAdd`, `ProductNeg`,
    `ProductSmul`, `ProductSub`, `ProductNorm`, `ProductDist`, `ProductNormEstimateArgs`.
  - Theorems: `norm_dist_def`, `norm_nonneg_from_args`, `norm_zero_from_args`,
    `norm_triangle_from_args`, `norm_neg_from_args`, `norm_dist_self_from_args`,
    `norm_dist_symm_from_args`, `norm_dist_triangle_from_args`, `product_zero_def`,
    `product_add_def`, `product_neg_def`, `product_smul_def`, `product_sub_def`,
    `product_norm_def`, `product_dist_def`, `product_fst_pair_from_args`,
    `product_snd_pair_from_args`, `product_pair_eta_from_args`,
    `product_add_fst_from_pair_law`, `product_add_snd_from_pair_law`,
    `product_smul_fst_from_pair_law`, `product_smul_snd_from_pair_law`,
    `product_norm_pair_eq_from_pair_laws`, `product_norm_fst_le_from_args`,
    `product_norm_snd_le_from_args`, `product_norm_pair_le_add_from_args`,
    `product_norm_add_le_from_args`, `product_dist_pair_le_add_from_args`.
- Boundary note:
  - Product carrier, pairing, projections, and norm estimates are explicit law assumptions, not
    kernel primitives or typeclass instances.
  - Completeness/Banach-space structure remains deferred to IIF5.
- Needed later for:
  - the map `Phi(x,y) = (x, F(x,y))` and the inverse theorem applied to `X x Y`.

### IIF3: Linear Isomorphism And Operator Norm API

- Status: Certificate generated for the explicit-law linear-map, operator-bound, linear-isomorphism,
  and block-triangular inverse API in `Proofs.Ai.Analysis.AbstractLinearMap`.
- Deliverables:
  - Define bounded linear maps and linear isomorphism evidence.
  - Prove composition, identity, inverse, and operator-norm bound lemmas.
  - Prove block-triangular inverse construction:

```text
(h, k) |-> (h, A h + B k)
```

is invertible when `B` is invertible.

- Completed exports:
  - Definitions: `OperatorNormBound`, `LinearMapLawArgs`, `BoundedLinearMapArgs`,
    `LinearIsoArgs`, `LinearId`, `LinearComp`, `LinearInv`, `BlockTriangularMap`,
    `BlockTriangularInverse`, `BlockTriangularIsoArgs`.
  - Theorems: `operator_norm_bound_apply`, `linear_map_zero_from_args`,
    `linear_map_add_from_args`, `linear_map_neg_from_args`, `linear_map_smul_from_args`,
    `bounded_linear_map_linear_from_args`, `bounded_linear_map_bound_from_args`,
    `bounded_linear_map_bound_apply`, `linear_iso_forward_linear_from_args`,
    `linear_iso_inverse_linear_from_args`, `linear_iso_forward_bound_from_args`,
    `linear_iso_inverse_bound_from_args`, `linear_iso_left_inverse_from_args`,
    `linear_iso_right_inverse_from_args`, `linear_id_def`, `linear_id_zero`, `linear_id_add`,
    `linear_id_neg`, `linear_id_smul`, `linear_id_law_args`, `linear_comp_def`,
    `linear_comp_law_args`, `linear_inv_def`, `linear_inv_left_inverse_from_iso`,
    `linear_inv_right_inverse_from_iso`, `block_triangular_map_def`,
    `block_triangular_inverse_def`, `block_triangular_b_iso_from_args`,
    `block_triangular_left_inverse_from_args`, `block_triangular_right_inverse_from_args`.
- Boundary note:
  - The identity and composition linear-law constructors are checked proof terms.
  - Operator-norm estimates, inverse laws, and block-triangular cancellation facts are packaged as
    explicit law evidence, not trusted kernel facts.
- Needed later for:
  - proving that `D Phi(a,b)` is invertible from the invertibility of `D_y F(a,b)`.

### IIF4: Frechet Derivative Calculus

- Status: Foundational certificate generated for the explicit-law derivative, differentiability,
  uniqueness, calculus-rule, and partial-derivative API in
  `Proofs.Ai.Analysis.AbstractDerivative`.
- Deliverables:
  - Define Frechet differentiability at a point and on a neighborhood.
  - Prove derivative uniqueness.
  - Prove derivative rules for constants, identity, projections, pairing, composition, and product
    maps.
  - Prove partial derivative packaging for maps out of a product.
- Completed exports:
  - Definitions: `FrechetRemainder`, `FrechetDerivativeAt`, `FrechetDifferentiableAt`,
    `FrechetDifferentiableOn`, `DerivativeUniqueArgs`, `ConstMap`, `ZeroMap`, `PairMap`,
    `PartialXMap`, `PartialYMap`, `PartialXDerivativeMap`, `PartialYDerivativeMap`,
    `DerivativeConstRuleArgs`, `DerivativeIdRuleArgs`, `DerivativeFstRuleArgs`,
    `DerivativeSndRuleArgs`, `DerivativePairRuleArgs`, `DerivativeCompRuleArgs`,
    `PartialDerivativeRuleArgs`.
  - Theorems: `frechet_remainder_def`, `frechet_derivative_at_intro`,
    `frechet_derivative_linear_from_at`, `frechet_derivative_bound_from_at`,
    `frechet_derivative_remainder_from_at`, `frechet_differentiable_at_intro`,
    `frechet_differentiable_at_elim`, `frechet_differentiable_on_apply`,
    `derivative_unique_from_args`, `const_map_def`, `zero_map_def`, `pair_map_def`,
    `derivative_const_from_args`, `derivative_id_from_args`, `derivative_fst_from_args`,
    `derivative_snd_from_args`, `derivative_pair_from_args`, `derivative_comp_from_args`,
    `partial_x_map_def`, `partial_y_map_def`, `partial_x_derivative_map_def`,
    `partial_y_derivative_map_def`, `partial_x_derivative_from_args`,
    `partial_y_derivative_from_args`.
- Boundary note:
  - `FrechetDerivativeAt` records bounded linearity and an explicit remainder-smallness predicate
    over the definitional remainder; the small-o content is ordinary law evidence, not a kernel
    primitive.
  - Uniqueness and calculus rules are exported as explicit rule packages and projection theorems.
    The current API covers constants, identity, projections, pairing, composition, and partial
    derivative extraction; richer product-map estimates can refine these packages later without
    changing the trusted boundary.
- Remaining refinement:
  - Add a dedicated product-map derivative rule package once IIF7 fixes the exact product-map
    shape needed for `Phi`.
- Needed later for:
  - computing `D Phi(a,b)` and deriving the formula for `Dg`.

### IIF5: Contraction Mapping Theorem

- Status: Foundational certificate generated for the explicit-law completeness, contraction, fixed
  point, uniqueness, local-stability, and Banach fixed-point result API in
  `Proofs.Ai.Analysis.AbstractFixedPoint`.
- Deliverables:
  - Define complete metric/normed-space package.
  - Prove Banach fixed point theorem with uniqueness and local stability.
  - Provide a certificate-backed theorem usable by the inverse-function theorem proof.
- Completed exports:
  - Definitions: `CauchySeq`, `ConvergesTo`, `CompleteMetricArgs`, `SelfMapOn`,
    `ContractiveOn`, `FixedPoint`, `FixedPointStability`, `FixedPointEvidence`,
    `FixedPointResult`, `BanachFixedPointArgs`.
  - Theorems: `cauchy_seq_intro`, `cauchy_seq_apply`, `converges_to_intro`,
    `converges_to_apply`, `complete_metric_limit_from_args`, `self_map_on_apply`,
    `contractive_on_apply`, `fixed_point_def`, `fixed_point_stability_apply`,
    `fixed_point_evidence_intro`, `fixed_point_evidence_elim`, `fixed_point_mem_from_evidence`,
    `fixed_point_eq_from_evidence`, `fixed_point_unique_from_evidence`,
    `fixed_point_stability_from_evidence`, `fixed_point_result_intro`,
    `fixed_point_result_elim`, `banach_fixed_point_from_args`.
- Boundary note:
  - Completeness, strict contraction, and the Banach fixed-point conclusion are ordinary explicit
    law evidence. The kernel only checks the package definitions, projections, and theorem terms.
  - `FixedPointResult` packages existence through a Church-encoded fixed point and
    `FixedPointEvidence`; local stability is represented through the existing `LocalPred` API.
- Needed later for:
  - constructing local inverse candidates by Newton/contraction estimates.

### IIF6: Quantitative Inverse Function Theorem

- Status: Foundational certificate generated for the explicit-law residual/Newton-map, local
  inverse evidence/result, uniqueness, fixed-point bridge, inverse differentiability, and
  quantitative inverse-function result API in `Proofs.Ai.Analysis.AbstractInverseFunction`.
- Deliverables:
  - Prove a quantitative inverse function theorem from IIF3-IIF5.
  - Export both:
    - local inverse existence and uniqueness,
    - differentiability of the local inverse and derivative formula.
- Completed exports:
  - Definitions: `InverseResidual`, `InverseNewtonMap`, `LocalInverseEvidence`,
    `LocalInverseResult`, `QuantitativeInverseFunctionArgs`.
  - Theorems: `inverse_residual_def`, `inverse_newton_map_def`,
    `local_inverse_evidence_intro`, `local_inverse_evidence_elim`,
    `local_inverse_base_mem_from_evidence`, `local_inverse_image_mem_from_evidence`,
    `local_inverse_maps_from_evidence`, `local_inverse_left_from_evidence`,
    `local_inverse_right_from_evidence`, `local_inverse_unique_from_evidence`,
    `local_inverse_fixed_point_from_evidence`, `local_inverse_derivative_from_evidence`,
    `local_inverse_linear_iso_from_evidence`, `local_inverse_result_intro`,
    `local_inverse_result_elim`, `quantitative_inverse_function_from_args`.
- Boundary note:
  - The local inverse is packaged through `LocalInverseResult`, so the quantitative theorem does not
    take a completed inverse function as an input.
  - Analytic estimates, completeness, contraction construction, and the quantitative bounds remain
    explicit law evidence. The kernel checks only the package definitions, projections, and theorem
    terms.
  - `InverseNewtonMap` records the Newton-style fixed-point map
    `x |-> x - df_inv (f x - target)`, and `LocalInverseEvidence` links each target to a
    `FixedPointResult` for that map.
- Acceptance criteria:
  - The theorem takes explicit radius and Lipschitz/smallness hypotheses.
  - The proof does not assume an inverse function as input.
  - The generated certificate lists only the expected analysis axioms/law packages.

### IIF7: Build The Auxiliary Map `Phi`

- Status: Foundational certificate generated for the auxiliary `Phi` construction, base-point
  equation, derivative law package, and block-triangular linear-isomorphism bridge in
  `Proofs.Ai.Analysis.AbstractImplicitPhi`.
- Deliverables:
  - Define `Phi(x,y) = (x, F(x,y))`.
  - Prove `Phi(a,b) = (a,0)` from `F(a,b)=0`.
  - Compute `D Phi(a,b)` using product derivative rules.
  - Prove `D Phi(a,b)` is a linear isomorphism when `D_y F(a,b)` is.
- Completed exports:
  - Definitions: `ImplicitPhiCoord`, `ImplicitPhi`, `ImplicitPhiDerivativeMap`,
    `ImplicitPhiDerivativeArgs`, `ImplicitPhiIsoArgs`.
  - Theorems: `implicit_phi_coord_def`, `implicit_phi_def`,
    `implicit_phi_coord_base_value_from_zero`, `implicit_phi_derivative_map_def`,
    `implicit_phi_full_derivative_from_args`, `implicit_phi_partial_x_from_args`,
    `implicit_phi_partial_y_from_args`, `implicit_phi_derivative_from_args`,
    `implicit_phi_dy_iso_from_args`, `implicit_phi_block_triangular_args_from_args`,
    `implicit_phi_linear_iso_from_args`, `implicit_phi_block_left_inverse_from_args`,
    `implicit_phi_block_right_inverse_from_args`.
- Boundary note:
  - The block derivative computation and estimates are explicit law evidence in
    `ImplicitPhiDerivativeArgs`; they are not kernel primitives.
  - The invertibility bridge records both `D_y F(a,b)` as a `LinearIsoArgs` package and the
    block-triangular inverse package needed to treat `D Phi(a,b)` as a linear isomorphism.  The
    full linearity and operator-bound evidence for `D Phi(a,b)` remains explicit in
    `ImplicitPhiIsoArgs`.
  - The only expected axiom reported by this module is `Eq.rec`, used through the equality
    congruence proof turning `F(a,b)=0` into `Phi(a,b)=(a,0)`.
- Needed later for:
  - applying IIF6 to obtain a local inverse for `Phi`.

### IIF8: Extract The Implicit Function

- Status: Planned.
- Deliverables:
  - From the local inverse of `Phi`, define `g(x)` as the second projection of
    `Phi^{-1}(x,0)`.
  - Prove local membership `g(x) : V`.
  - Prove the zero equation `F(x,g(x)) = 0`.
  - Prove uniqueness: if `F(x,y)=0` and `y` is in the chosen local neighborhood, then `y = g(x)`.
- Acceptance criteria:
  - The theorem states local, not global, uniqueness.
  - Neighborhood shrink steps are explicit certificate-backed lemmas, not informal side conditions.

### IIF9: Differentiability And Derivative Formula For `g`

- Status: Planned.
- Deliverables:
  - Prove differentiability of `g` from differentiability of the local inverse and projections.
  - Prove the derivative formula

```text
Dg(x) = - (D_y F(x, g(x)))^{-1} o D_x F(x, g(x)).
```

  - Package the formula as a separate theorem so the basic existence/uniqueness theorem can land
    first if the derivative formula requires more linear algebra.

### IIF10: Final Evidence Wrapper

- Status: Planned.
- Deliverables:
  - Add `ImplicitFunctionTheoremEvidence` or a Church-encoded package collecting:
    - existence of `g`,
    - zero equation,
    - local uniqueness,
    - optional differentiability and derivative formula.
  - Add `implicit_function_theorem` as the public certificate-backed theorem.
  - Update `proofs/manifest.toml`, metadata, replay, and generator tests.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`

## Boundary Notes

- If completeness of the ambient space is not yet available, prove a finite-dimensional version
  first with an explicit completeness law package, then generalize to Banach spaces.
- If the quantitative inverse theorem is too large as one certificate, split it into:
  - contraction construction,
  - injectivity/local uniqueness,
  - surjectivity/local existence,
  - differentiability of the inverse.
- Do not add quotient-like or choice-like primitives to select `g`.  The construction should be
  through the certified local inverse or through a certified fixed-point construction.
- Keep the basic existence/uniqueness theorem separate from the derivative formula so the first
  implicit-function certificate can be completed before the full calculus API is mature.
