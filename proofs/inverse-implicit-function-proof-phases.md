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

- Status: Planned.
- Deliverables:
  - Define product-space operations and product norm.
  - Prove projection and pairing laws for products.
  - Prove enough norm inequalities for product-space estimates.
- Needed later for:
  - the map `Phi(x,y) = (x, F(x,y))` and the inverse theorem applied to `X x Y`.

### IIF3: Linear Isomorphism And Operator Norm API

- Status: Planned.
- Deliverables:
  - Define bounded linear maps and linear isomorphism evidence.
  - Prove composition, identity, inverse, and operator-norm bound lemmas.
  - Prove block-triangular inverse construction:

```text
(h, k) |-> (h, A h + B k)
```

is invertible when `B` is invertible.

- Needed later for:
  - proving that `D Phi(a,b)` is invertible from the invertibility of `D_y F(a,b)`.

### IIF4: Frechet Derivative Calculus

- Status: Planned.
- Deliverables:
  - Define Frechet differentiability at a point and on a neighborhood.
  - Prove derivative uniqueness.
  - Prove derivative rules for constants, identity, projections, pairing, composition, and product
    maps.
  - Prove partial derivative packaging for maps out of a product.
- Needed later for:
  - computing `D Phi(a,b)` and deriving the formula for `Dg`.

### IIF5: Contraction Mapping Theorem

- Status: Planned.
- Deliverables:
  - Define complete metric/normed-space package.
  - Prove Banach fixed point theorem with uniqueness and local stability.
  - Provide a certificate-backed theorem usable by the inverse-function theorem proof.
- Needed later for:
  - constructing local inverse candidates by Newton/contraction estimates.

### IIF6: Quantitative Inverse Function Theorem

- Status: Planned.
- Deliverables:
  - Prove a quantitative inverse function theorem from IIF3-IIF5.
  - Export both:
    - local inverse existence and uniqueness,
    - differentiability of the local inverse and derivative formula.
- Acceptance criteria:
  - The theorem takes explicit radius and Lipschitz/smallness hypotheses.
  - The proof does not assume an inverse function as input.
  - The generated certificate lists only the expected analysis axioms/law packages.

### IIF7: Build The Auxiliary Map `Phi`

- Status: Planned.
- Deliverables:
  - Define `Phi(x,y) = (x, F(x,y))`.
  - Prove `Phi(a,b) = (a,0)` from `F(a,b)=0`.
  - Compute `D Phi(a,b)` using product derivative rules.
  - Prove `D Phi(a,b)` is a linear isomorphism when `D_y F(a,b)` is.
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
