# AI Proof Corpus

This directory stores proof artifacts intended for AI-facing proof production and regression.
Artifact paths follow the module namespace. For example, module `Proofs.Ai.Basic` lives at
`Proofs/Ai/Basic/`.

The trust boundary follows the repository-wide certificate-first policy:

- `*.npa`, `*.replay.json`, and `*.meta.json` are non-trusted producer sidecars.
- `*.npcert` is the canonical artifact consumed by the certificate verifier and kernel.
- A proof is accepted only when the certificate decodes canonically and `verify_module_cert` succeeds.

Current bundles:

- `Proofs/Ai/Basic/`: small no-import, no-axiom combinator and implication theorem module.
- `Proofs/Ai/Eq/`: equality refl theorem module importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs/Ai/EqReasoning/`: equality reasoning module importing `Std.Logic.Eq` and using the
  expected builtin `Eq.rec` axiom interface.
- `Proofs/Ai/Algebra/Ring/`: singleton-carrier algebra API and ring-law theorem targets importing
  `Std.Logic.Eq`.
- `Proofs/Ai/Algebra/Square/`: square API and square-expansion theorem targets importing
  `Std.Logic.Eq` and `Proofs.Ai.Algebra.Ring`.
- `Proofs/Ai/Nat/`: Nat smoke theorem module importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs/Ai/OrderedField/`: order and square-root API theorem targets importing `Std.Logic.Eq`,
  `Proofs.Ai.Algebra.Ring`, and `Proofs.Ai.Algebra.Square`.
- `Proofs/Ai/Prop/`: import-free proposition-only implication search module.
- `Proofs/Ai/Reduction/`: reduction smoke theorem module importing `Std.Nat.Basic`.
- `Proofs/Ai/Vector/Basic/`: vector carrier and basic vector addition theorem targets importing
  `Std.Logic.Eq`.
- `Proofs/Ai/Vector/Dot/`: dot product, squared norm, and squared distance theorem targets
  importing vector, scalar, square, and order corpus layers.
- `Proofs/Ai/Vector/AbstractSpace/`: abstract vector-space theorem targets over the P17-P19
  scalar API layers and explicit vector operation/law assumptions.
- `Proofs/Ai/Vector/AbstractInnerProduct/`: abstract inner-product, squared norm, and vector
  squared-distance theorem targets over explicit scalar, vector, and inner-product law assumptions.
- `Proofs/Ai/Vector/AbstractInnerProductDerive/`: checked norm-expansion derivations from
  `InnerProductLawArgs`, P27 scalar rewrites, and `PerpVec`.
- `Proofs/Ai/Geometry/Affine/`: abstract point, displacement, and point squared-distance theorem
  targets over explicit affine compatibility law assumptions.
- `Proofs/Ai/Geometry/AffineDerive/`: checked affine displacement orientation and point-distance
  bridge derivations from primitive affine and vector law packages.
- `Proofs/Ai/Geometry/AbstractRightTriangle/`: abstract perpendicularity, right-triangle, and
  squared-distance Pythagorean theorem targets over explicit geometry law assumptions.
- `Proofs/Ai/Geometry/AbstractRightTriangleDerive/`: checked right-triangle-to-perpendicular
  bridge derivations for the abstract Pythagorean route.
- `Proofs/Ai/Geometry/AbstractMetric/`: abstract distance, metric law-package, ball API, and
  metric-distance theorem targets over explicit metric law assumptions.
- `Proofs/Ai/Geometry/Pythagorean/`: final abstract Pythagorean theorem names, including the
  checked squared-distance derivation from scalar, vector, inner-product, affine, and
  right-triangle law packages.
- `Proofs/Ai/Geometry/RightTriangle/`: right-triangle and squared-distance Pythagoras theorem
  targets importing vector dot and scalar corpus layers.
- `Proofs/Ai/Geometry/Metric/`: distance API and metric theorem targets importing the right-triangle
  and vector dot layers.
- `Proofs/Ai/Logic/Iff/`: first-class logical equivalence, conjunction, disjunction, falsehood, and
  negation theorem targets importing `Std.Logic.Eq`.
- `Proofs/Ai/Algebra/AbstractRing/`: abstract scalar ring theorem targets over explicit carrier,
  operation, and law assumptions importing `Std.Logic.Eq`.
- `Proofs/Ai/Algebra/AbstractOrderedField/`: abstract scalar order and square-root theorem targets
  over explicit carrier, operation, relation, function, and law assumptions.
- `Proofs/Ai/Algebra/AbstractSquareNormalize/`: abstract square-normalization theorem targets over
  the P17/P18 scalar APIs and explicit law assumptions.
- `Proofs/Ai/Algebra/AbstractScalarDerive/`: scalar rewrite derivations from `RingLawArgs` and
  equality transport, including the zero cross-term cancellation needed by the abstract
  Pythagorean route.
- `manifest.toml`: stable index for the corpus and expected hashes.

## Expansion Plan

Grow the corpus in small, checkable layers. Each layer should keep source, replay, metadata, and
certificate artifacts together, and every checked-in `.npcert` must be covered by an integration
test.

### P0: Basic Combinators

Module: `Proofs.Ai.Basic`

These are the initial no-import, no-axiom examples. They exercise binders, local lookup, direct
application, higher-order arguments, and simple proposition-shaped goals without relying on any
library theorem.

Implemented:

| Theorem | Shape |
| --- | --- |
| `id` | `A -> A` |
| `const_left` | `A -> B -> A` |
| `const_right` | `A -> B -> B` |
| `apply_fn` | `(A -> B) -> A -> B` |
| `compose` | `(B -> C) -> (A -> B) -> A -> C` |
| `flip` | `(A -> B -> C) -> B -> A -> C` |
| `duplicate` | `(A -> A -> B) -> A -> B` |
| `prop_id` | `P -> P` |
| `modus_ponens` | `(P -> Q) -> P -> Q` |
| `imp_trans` | `(P -> Q) -> (Q -> R) -> P -> R` |

### P1: More Basic Search Targets

Module: `Proofs.Ai.Basic`

These extend `Proofs.Ai.Basic` before introducing imports. They give AI search more variation while
staying in the same trusted boundary and proof style.

Implemented:

| Theorem | Shape |
| --- | --- |
| `compose_assoc` | `(C -> D) -> (B -> C) -> (A -> B) -> A -> D` |
| `apply_twice` | `(A -> A) -> A -> A`, with proof `f (f x)` |
| `ignore_middle` | `A -> B -> C -> A` |
| `select_middle` | `A -> B -> C -> B` |
| `select_last` | `A -> B -> C -> C` |
| `imp_swap` | `(P -> Q -> R) -> Q -> P -> R` |
| `imp_compose` | `(Q -> R) -> (P -> Q) -> P -> R` |
| `imp_ignore` | `P -> Q -> P` |
| `imp_duplicate` | `(P -> P -> Q) -> P -> Q` |
| `higher_apply` | `((A -> B) -> C) -> (A -> B) -> C` |

### P2: Equality Refl Corpus

Module: `Proofs.Ai.Eq`

This module imports `Std.Logic.Eq` and keeps the first equality examples refl-only. It checks import
interfaces and builtin equality references without adding rewrite search as a dependency. Later
Eq layers also import `Std.Nat.Basic` for Nat-specialized equality targets.

Implemented:

| Theorem | Shape |
| --- | --- |
| `eq_refl_self` | `x = x` |
| `eq_refl_fn_app` | `f x = f x` |
| `eq_refl_compose` | `f (g x) = f (g x)` |
| `eq_self_imp` | `x = x -> x = x` |
| `eq_refl_prop` | refl over a proposition-shaped term |

### P3: Nat Smoke Corpus

Module: `Proofs.Ai.Nat`

This module imports `Std.Nat.Basic` after P1 and P2 are stable. It also imports `Std.Logic.Eq`
for the refl-only equality smoke tests. Proofs stay closed by locals or refl/reduction so failures
are easy to attribute to import or kernel behavior.

Implemented:

| Theorem | Shape |
| --- | --- |
| `nat_zero_self_eq` | `Nat.zero = Nat.zero` |
| `nat_succ_zero_self_eq` | `Nat.succ Nat.zero = Nat.succ Nat.zero` |
| `nat_id` | `Nat -> Nat` |
| `nat_const_zero` | `Nat -> Nat`, with proof `Nat.zero` |
| `nat_apply_fn` | `(Nat -> Nat) -> Nat -> Nat` |

### P4: More Nat Search Targets

Module: `Proofs.Ai.Nat`

Extend the Nat corpus with closed local/application patterns before introducing recursion or
arithmetic lemmas. These should remain no-axiom proofs over `Std.Nat.Basic` and `Std.Logic.Eq`.

Implemented:

| Theorem | Shape |
| --- | --- |
| `nat_const_succ_zero` | `Nat -> Nat`, with proof `Nat.succ Nat.zero` |
| `nat_apply_twice` | `(Nat -> Nat) -> Nat -> Nat`, with proof `f (f n)` |
| `nat_compose` | `(Nat -> Nat) -> (Nat -> Nat) -> Nat -> Nat` |
| `nat_ignore_middle` | `Nat -> Nat -> Nat -> Nat`, selecting the first argument |
| `nat_select_middle` | `Nat -> Nat -> Nat -> Nat`, selecting the second argument |
| `nat_select_last` | `Nat -> Nat -> Nat -> Nat`, selecting the third argument |
| `nat_succ_self_eq` | `forall (n : Nat), Nat.succ n = Nat.succ n` |

### P5: Equality Shape Expansion

Module: `Proofs.Ai.Eq`

Add more refl-only equality targets with deeper application spines. The goal is to teach producers
to preserve the exact head, universe, and argument structure without relying on rewrite search.

Implemented:

| Theorem | Shape |
| --- | --- |
| `eq_refl_apply_twice` | `f (f x) = f (f x)` |
| `eq_refl_higher_apply` | `h f = h f` |
| `eq_refl_nested_compose` | `f (g (h x)) = f (g (h x))` |
| `eq_refl_prop_apply` | `h p = h p` for proposition-valued functions |
| `eq_local_passthrough` | `(h : x = x) -> x = x` |
| `eq_refl_nat_function` | `f n = f n` specialized to Nat |

### P6: Proposition Search Expansion

Module: `Proofs.Ai.Prop`

Split proposition-only implication patterns out of `Proofs.Ai.Basic` once the Basic module becomes
large. These remain import-free and should exercise binder ordering, argument permutation, and
higher-order implication search.

Implemented:

| Theorem | Shape |
| --- | --- |
| `imp_chain4` | `(P -> Q) -> (Q -> R) -> (R -> S) -> P -> S` |
| `imp_permute3` | `(P -> Q -> R -> S) -> R -> P -> Q -> S` |
| `imp_apply_twice` | `(P -> P) -> P -> P`, with proof `h (h p)` |
| `imp_const3` | `P -> Q -> R -> P` |
| `imp_flip_chain` | `(Q -> R) -> (P -> Q) -> P -> R` |
| `imp_higher_apply` | `((P -> Q) -> R) -> (P -> Q) -> R` |

### P7: Reduction Smoke Corpus

Module: `Proofs.Ai.Reduction`

Introduce small beta/zeta/delta-shaped examples only after the non-reduction corpora are stable.
Items involving named helper definitions may require extending the artifact generator beyond
theorem-only modules.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `reduction_id_nat` | local Nat identity definition used by `delta_id_nat` |

Theorem targets:

| Theorem | Shape |
| --- | --- |
| `beta_id_nat` | `Nat -> Nat`, with proof `(fun x => x) n` |
| `beta_const_nat` | `Nat -> Nat -> Nat`, with proof `(fun x => fun _ => x) n m` |
| `let_id_nat` | `Nat -> Nat`, with proof `let x : Nat := n in x` |
| `let_const_nat` | `Nat -> Nat`, with proof `let z : Nat := Nat.zero in z` |
| `delta_id_nat` | `Nat -> Nat` through a local named identity definition |

### P8: Equality Reasoning Corpus

Module: `Proofs.Ai.EqReasoning`

Introduce equality elimination as an explicit, audited dependency. This layer imports
`Std.Logic.Eq` and intentionally uses the kernel builtin `Eq.rec` axiom interface. `Eq.rec` is
recorded in the certificate axiom report and is checked against the expected axiom list; no
additional module-local axioms are introduced.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| none | Uses imported `Eq`, `Eq.refl`, and builtin `Eq.rec` only |

Theorem targets:

| Theorem | Shape |
| --- | --- |
| `eq_symm` | symmetry of equality |
| `eq_trans` | transitivity of equality |
| `eq_congr_arg` | congruence under a function argument |
| `eq_congr_fun` | congruence of equal functions at an argument |
| `eq_congr2` | congruence for a binary function |
| `eq_subst` | substitution into a proposition family |
| `eq_transport_const` | transport through a constant proposition family |
| `eq_rewrite_left` | left-to-right chained rewrite |
| `eq_rewrite_right` | right-side rewrite through symmetry-shaped input |
| `eq_cast_trans` | composed transport through two equalities |
| `eq_calc3` | three-step equality calculation using transitivity |

### P9: Algebra Ring Corpus

Module: `Proofs.Ai.Algebra.Ring`

Introduce a minimal algebra layer for later square/vector/geometry milestones. This module does
not add abstract ring axioms to the trusted base. Instead it defines a checked singleton carrier
`RingElem` and operation API over that carrier, then proves the selected ring-shaped law targets as
ordinary certificate-checked theorem declarations. The carrier and operations are API declarations,
not proof targets.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `RingElem` | singleton scalar carrier for this corpus layer |
| `zero` | additive identity API |
| `one` | multiplicative identity API |
| `add` | addition API |
| `neg` | additive inverse API |
| `sub` | subtraction API, defined as `add a (neg b)` |
| `mul` | multiplication API |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `a - b = a + -b` |
| `add_assoc` | `(a + b) + c = a + (b + c)` |
| `add_comm` | `a + b = b + a` |
| `add_zero` | `a + 0 = a` |
| `zero_add` | `0 + a = a` |
| `neg_add_cancel` | `-a + a = 0` |
| `add_neg_cancel` | `a + -a = 0` |
| `sub_self` | `a - a = 0` |
| `mul_assoc` | `(a * b) * c = a * (b * c)` |
| `mul_comm` | `a * b = b * a` |
| `mul_one` | `a * 1 = a` |
| `one_mul` | `1 * a = a` |
| `mul_zero` | `a * 0 = 0` |
| `zero_mul` | `0 * a = 0` |
| `left_distrib` | `a * (b + c) = a * b + a * c` |
| `right_distrib` | `(a + b) * c = a * c + b * c` |
| `add_left_cancel` | `a + b = a + c -> b = c` |
| `mul_add` | multiplication distributes over addition on the right argument |
| `add_mul` | multiplication distributes over addition on the left argument |
| `ring_normalize_add_mul3` | small normalization target for sums/products of three terms |

### P10: Algebra Square Corpus

Module: `Proofs.Ai.Algebra.Square`

Build on `Proofs.Ai.Algebra.Ring` with a small square API and the first square-expansion targets
needed by the coordinate / inner-product route to Pythagoras. As with P9, this is a concrete
singleton-carrier corpus layer rather than an abstract algebraic axiom package. `two` and `sq` are
API declarations; the square identities are proof targets checked through the certificate verifier.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `two` | scalar `2` API, defined as `add one one` |
| `sq` | square operation API, defined as `mul a a` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_zero` | `sq 0 = 0` |
| `sq_one` | `sq 1 = 1` |
| `sq_neg` | `sq (-a) = sq a` |
| `two_mul` | `2 * a = a + a` |
| `sq_add` | `sq (a + b) = sq a + 2 * a * b + sq b` |
| `sq_sub` | `sq (a - b) = sq a - 2 * a * b + sq b` |
| `sum_two_squares_comm` | `sq a + sq b = sq b + sq a` |
| `sq_eq_sq_of_eq_or_neg_eq` | square equality from an equality-or-negated-equality witness shape |
| `square_nonneg` | predicate-generic bridge `Nonneg 0 -> Nonneg (sq a)`; P11 adds the concrete `le_square_nonneg` version |

### P11: Ordered Field Corpus

Module: `Proofs.Ai.OrderedField`

Build on `Proofs.Ai.Algebra.Ring` and `Proofs.Ai.Algebra.Square` with the first order and
square-root API targets needed by the later metric form of Pythagoras. This layer remains a
concrete singleton-carrier corpus: `le`, `lt`, and `sqrt` are API declarations, while the order
and square-root facts are certificate-checked theorem targets. `le`/`lt` are currently trivial
relations over the singleton scalar carrier; later abstract ordered-field work can replace this
with structure fields without changing the trusted boundary.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le` | non-strict order relation API |
| `lt` | strict order relation API |
| `sqrt` | square-root API for the later metric form |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl` | `a <= a` |
| `le_trans` | `a <= b -> b <= c -> a <= c` |
| `add_nonneg` | `0 <= a -> 0 <= b -> 0 <= a + b` |
| `mul_nonneg` | `0 <= a -> 0 <= b -> 0 <= a * b` |
| `le_square_nonneg` | concrete `le` version of `0 <= sq a`; named separately from P10's imported `square_nonneg` |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | nonnegative equality from equal squares |

### P12: Vector Basic Corpus

Module: `Proofs.Ai.Vector.Basic`

Add the first vector carrier and additive group-shaped API targets for the coordinate route to
Pythagoras. This module intentionally stays independent of scalar/order APIs and imports only
`Std.Logic.Eq`; `Vector.Dot` is the next layer that combines vectors with the scalar field facts.
As with the earlier algebra layers, this is a concrete singleton-carrier corpus whose declarations
are checked through canonical certificates.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vec` | singleton vector or point-difference carrier |
| `vec_zero` | vector zero |
| `vec_add` | vector addition |
| `vec_neg` | vector negation |
| `vec_sub` | vector subtraction, defined as `u + -v` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_assoc` | `(u + v) + w = u + (v + w)` |
| `vec_add_comm` | `u + v = v + u` |
| `vec_zero_add` | `0 + v = v` |
| `vec_add_zero` | `v + 0 = v` |
| `vec_neg_add_cancel` | `-v + v = 0` |
| `vec_add_neg_cancel` | `v + -v = 0` |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_sub_eq_add_neg` | alias-style subtraction rewrite target for AI search |
| `vec_sub_self` | `v - v = 0` |
| `vec_sub_zero` | `v - 0 = v` |
| `vec_add_left_cancel` | `u + v = u + w -> v = w` |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v` |

### P13: Vector Dot Corpus

Module: `Proofs.Ai.Vector.Dot`

Connect `Proofs.Ai.Vector.Basic` with the scalar corpus by adding dot product, squared norm, and
squared distance APIs. This is still a singleton-carrier corpus, so the theorem statements are
designed as durable targets for later nontrivial instances while the current certificates remain
small and axiom-free.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | inner product API |
| `normSq` | squared norm, defined as `dot v v` |
| `distSq` | squared distance, defined as `normSq (B - A)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | `dot u v = dot v u` |
| `dot_add_left` | `dot (u + v) w = dot u w + dot v w` |
| `dot_add_right` | `dot u (v + w) = dot u v + dot u w` |
| `dot_neg_left` | `dot (-u) v = -dot u v` |
| `dot_neg_right` | `dot u (-v) = -dot u v` |
| `dot_sub_left` | `dot (u - v) w = dot u w - dot v w` |
| `dot_sub_right` | `dot u (v - w) = dot u v - dot u w` |
| `norm_sq_def` | `normSq v = dot v v` |
| `dist_sq_def` | `distSq A B = normSq (B - A)` |
| `dot_self_eq_norm_sq` | `dot v v = normSq v` |
| `norm_sq_add` | `normSq (u + v) = normSq u + 2 * dot u v + normSq v` |
| `norm_sq_sub` | `normSq (u - v) = normSq u - 2 * dot u v + normSq v` |
| `norm_sq_add_of_dot_zero` | `dot u v = 0 -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_dot_zero` | `dot u v = 0 -> normSq (u - v) = normSq u + normSq v` |
| `parallelogram_law` | `normSq (u + v) + normSq (u - v) = 2 * normSq u + 2 * normSq v` |
| `polarization_identity` | `2 * dot u v = normSq (u + v) - (normSq u + normSq v)` |
| `norm_sq_nonneg` | `0 <= normSq v` |

### P14: Geometry Right Triangle Corpus

Module: `Proofs.Ai.Geometry.RightTriangle`

Add the first geometry layer over `Vec`, `dot`, `normSq`, and `distSq`. The main milestone target is
the squared-distance Pythagorean theorem
`RightTriangle A B C -> distSq B C = distSq A B + distSq A C`, with helper rewrites for the leg and
hypotenuse vectors. `perp_iff_dot_eq_zero` uses a Church-encoded equivalence eliminator because the
corpus does not yet define a first-class `Iff`; the later geometric API placeholders such as
midpoint or altitude foot are passed as predicate parameters rather than new definitions.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | perpendicularity predicate, defined as `dot u v = 0` |
| `RightTriangle` | right-triangle predicate over three points, with the right angle at `A` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | Church-encoded `Perp u v <-> dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg vectors from `RightTriangle A B C` |
| `hypotenuse_vector_eq_sub_legs` | `C - B = (C - A) - (B - A)` |
| `dist_sq_leg_left` | `distSq A B = normSq (B - A)` |
| `dist_sq_leg_right` | `distSq A C = normSq (C - A)` |
| `dist_sq_hypotenuse` | `distSq B C = normSq (C - B)` |
| `pythagorean_distance_sq` | `RightTriangle A B C -> distSq B C = distSq A B + distSq A C` |
| `law_of_cosines` | squared distance with a dot-product correction term |
| `right_triangle_area` | double-area squared target parameterized by a future `Area2` API |
| `median_to_hypotenuse` | midpoint-on-hypotenuse target parameterized by a future midpoint predicate |
| `altitude_on_hypotenuse` | altitude-foot target parameterized by future length and foot predicates |
| `thales_theorem` | circle/diameter-to-right-triangle target parameterized by a future circle predicate |

### P15: Geometry Metric Corpus

Module: `Proofs.Ai.Geometry.Metric`

Add the first distance layer over `distSq`, `sqrt`, and the right-triangle corpus. The main milestone
target is the squared metric Pythagorean statement
`RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)`. `distance_zero_iff_eq`
uses the same Church-encoded equivalence shape as P14 because the corpus still does not define a
first-class `Iff`.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | distance API, defined as `sqrt (distSq A B)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSq A B)` |
| `dist_sq_eq_square_dist` | `distSq A B = sq (dist A B)` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | Church-encoded `dist A B = 0 <-> A = B` |
| `pythagorean_distance` | `RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)` |
| `cauchy_schwarz` | `sq (dot u v) <= normSq u * normSq v` |
| `triangle_inequality` | `dist A C <= dist A B + dist B C` |

### P16: Logic Iff Corpus

Module: `Proofs.Ai.Logic.Iff`

Add first-class logical connectives for later abstract theorem APIs. `Iff`, `And`, `Or`, `False`,
and `Not` are defined as Prop-valued APIs so later modules can stop embedding ad hoc
Church-encoded equivalence shapes in theorem statements. `iff_of_eq` and `iff_congr_arg` use the
same audited `Eq.rec` dependency as P8; the expected axiom report is fixed to `["Eq.rec"]`.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Iff` | first-class logical equivalence, replacing ad hoc theorem-local equivalence encodings |
| `And` | conjunction API for bundling law hypotheses |
| `Or` | disjunction API for square-root and order case splits |
| `False` | empty proposition API for contradiction and negation eliminators |
| `Not` | negation abbreviation, defined as `P -> False` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `iff_refl` | `Iff P P` |
| `iff_symm` | `Iff P Q -> Iff Q P` |
| `iff_trans` | `Iff P Q -> Iff Q R -> Iff P R` |
| `iff_mp` | `Iff P Q -> P -> Q` |
| `iff_mpr` | `Iff P Q -> Q -> P` |
| `and_intro` | `P -> Q -> And P Q` |
| `and_left` | `And P Q -> P` |
| `and_right` | `And P Q -> Q` |
| `iff_of_eq` | `P = Q -> Iff P Q` |
| `false_elim` | `False -> P` |
| `not_intro` | `(P -> False) -> Not P` |
| `not_elim` | `Not P -> P -> False` |
| `or_inl` | `P -> Or P Q` |
| `or_inr` | `Q -> Or P Q` |
| `or_elim` | `Or P Q -> (P -> R) -> (Q -> R) -> R` |
| `iff_congr_arg` | `P = Q -> Iff (F P) (F Q)` for Prop-valued contexts |

### General Euclidean Pythagorean Roadmap

Long-term target: prove the Pythagorean theorem as a checked certificate over an abstract Euclidean
space, not only over the current concrete singleton corpus layer. Prefer the coordinate /
inner-product route first:

```text
RightTriangle A B C -> distSqPoints B C = distSqPoints A B + distSqPoints A C
```

Post-P25 implementation phases are tracked in `proofs/pythagorean-proof-phases.md`.

This avoids making the first abstract target depend on square roots. P15 adds the checked bridge to
the squared `dist` form over the current concrete scalar and vector corpus. P17 starts replacing the
singleton scalar layer with explicit carrier, operation, and law assumptions. P18 extends that
abstract scalar layer with order and square-root APIs. P19 supplies the abstract square
normalization layer. P20 supplies the abstract vector-space layer. P21 supplies the abstract
inner-product and squared-norm layer. P22 supplies the affine point/displacement layer. P23 supplies
the abstract right-triangle theorem layer. P24 supplies the abstract metric-distance theorem layer.
P25 supplies the final theorem API names that downstream users can depend on, P31 connects the
squared-distance theorem name to the checked law-package derivation, and P32 connects the squared
metric-distance theorem name to the checked metric bridge. P34 reviews the optional converse and
unsquared-distance strengthenings and keeps them out of completed theorem claims until their
nondegeneracy, angle, and square-root cancellation prerequisites are available.

Planned contents:

The `Definition / API declarations` column lists declarations introduced by `def`, structure
fields, or primitives. They are type-checked declarations, not proof targets. The theorem columns
list declarations that should have checked proof certificates. Definitional rewrite lemmas such as
`sub_eq_add_neg` and `square_def` are theorem targets, although many should close by `Eq.refl`
after unfolding.

Completed prerequisite:

- P8 `Proofs.Ai.EqReasoning` supplies equality symmetry, transitivity, congruence, substitution,
  transport, and calculation lemmas.
- P9 `Proofs.Ai.Algebra.Ring` supplies the first algebra API declarations and certificate-checked
  ring-shaped law targets over a concrete singleton carrier.
- P10 `Proofs.Ai.Algebra.Square` supplies `two`, `sq`, and square-expansion theorem targets over
  the same concrete scalar carrier.
- P11 `Proofs.Ai.OrderedField` supplies `le`, `lt`, `sqrt`, and the nonnegative square-root theorem
  targets needed by later metric statements.
- P12 `Proofs.Ai.Vector.Basic` supplies the first vector carrier and additive vector theorem targets
  used by the dot-product and geometry layers.
- P13 `Proofs.Ai.Vector.Dot` supplies `dot`, `normSq`, `distSq`, and the dot-product expansion
  targets used by the squared-distance Pythagoras route.
- P14 `Proofs.Ai.Geometry.RightTriangle` supplies `Perp`, `RightTriangle`, leg/hypotenuse rewrites,
  and the checked squared-distance Pythagorean theorem target.
- P15 `Proofs.Ai.Geometry.Metric` supplies `dist`, the `distSq = sq dist` bridge, and the checked
  squared metric Pythagorean theorem target.
- P16 `Proofs.Ai.Logic.Iff` supplies first-class `Iff`, `And`, `Or`, `False`, and `Not` APIs for
  later abstract algebra and geometry theorem statements.
- P17 `Proofs.Ai.Algebra.AbstractRing` supplies checked abstract ring theorem targets over explicit
  carrier, operation, and law assumptions without adding unchecked algebra axioms.
- P18 `Proofs.Ai.Algebra.AbstractOrderedField` supplies checked abstract order and square-root
  theorem targets over explicit carrier, operation, relation, function, and law assumptions without
  adding unchecked order or square-root axioms.
- P19 `Proofs.Ai.Algebra.AbstractSquareNormalize` supplies checked abstract square-normalization
  theorem targets over the P17/P18 scalar APIs and explicit law assumptions without adding unchecked
  algebra or order axioms.
- P20 `Proofs.Ai.Vector.AbstractSpace` supplies checked abstract vector-space theorem targets over
  explicit vector carrier, operation, scalar, and law assumptions without adding unchecked vector
  space axioms.
- P21 `Proofs.Ai.Vector.AbstractInnerProduct` supplies checked abstract inner-product, squared norm,
  vector squared-distance, perpendicularity, and norm-expansion theorem targets over explicit law
  assumptions without adding unchecked inner-product or positivity axioms.
- P22 `Proofs.Ai.Geometry.Affine` supplies checked abstract point, displacement, point
  squared-distance, and point extensionality theorem targets over explicit affine law assumptions
  without adding unchecked affine or Euclidean axioms.
- P23 `Proofs.Ai.Geometry.AbstractRightTriangle` supplies checked abstract perpendicularity,
  right-triangle, squared-distance Pythagorean, law-of-cosines, area, and median theorem targets
  over explicit geometry law assumptions without adding unchecked Euclidean axioms.
- P24 `Proofs.Ai.Geometry.AbstractMetric` supplies checked abstract distance, metric law-package,
  ball API, distance/squared-distance bridge, metric Pythagorean, and triangle-inequality theorem
  targets over explicit metric law assumptions without adding unchecked metric axioms.
- P25 `Proofs.Ai.Geometry.Pythagorean` supplies checked final abstract Pythagorean theorem names,
  alias targets, converse target shape, and dependency-package theorem target over the P17-P24
  abstract geometry stack without adding unchecked Euclidean axioms.
- P27 `Proofs.Ai.Algebra.AbstractScalarDerive` supplies checked scalar zero-cross-term derivations
  from `RingLawArgs` and equality transport, without accepting direct theorem-shaped scalar
  normalization law arguments.
- P28 `Proofs.Ai.Vector.AbstractInnerProductDerive` supplies checked norm expansion and
  perpendicular special-case derivations from `InnerProductLawArgs`, P27 scalar rewrites, and
  equality transport, without accepting direct dot-zero or perpendicular norm law arguments.
- P29 `Proofs.Ai.Geometry.AffineDerive` supplies checked affine hypotenuse orientation and
  point-distance/norm bridge derivations from primitive `AffineLawArgs`, `VectorSpaceLawArgs`, and
  equality transport, without accepting direct hypotenuse-vector or point-distance-definition law
  arguments.
- P30 `Proofs.Ai.Geometry.AbstractRightTriangleDerive` supplies checked bridges from
  `RightTriangle A B C` to the exact `PerpVec` / dot-zero premises needed after P29's additive
  hypotenuse orientation, without accepting a direct Pythagorean theorem-shaped argument.
- P31 `Proofs.Ai.Geometry.Pythagorean` supplies the checked squared-distance Pythagorean theorem
  from `RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, `AffineLawArgs`, and
  `RightTriangle A B C`, without accepting a direct Pythagorean theorem-shaped equality law.
- P32 `Proofs.Ai.Geometry.Pythagorean` supplies the checked squared metric-distance theorem by
  composing P31 with the P32 metric bridge, without accepting a direct metric Pythagorean law.
- P33 refreshes the final public Pythagorean API names and documentation around the completed
  squared and squared-metric theorem claims.
- P34 records the optional-strengthening boundary: no converse or unsquared-distance theorem is
  exported until checked nondegeneracy, angle, first-class `Iff` import, and square-root
  cancellation prerequisites are available.

Post-P25 policy:

- Keep all algebraic, order, vector-space, and inner-product laws as explicit theorem assumptions or
  checked law-package arguments until NPA has a dedicated structure/class layer.
- Do not introduce module-level unchecked axioms for field, vector, order, real, or Euclidean facts.
- Keep the final theorem independent of the concrete singleton `RingElem` and `Vec` carriers.
- Prefer squared-distance statements first; add square-root distance forms only after the required
  nonnegative square-root and square-cancellation lemmas are available.

The current P17-P34 abstract Pythagorean roadmap now has checked squared-distance and squared
metric-distance theorem names from law packages. Later work can replace explicit law arguments with
checked structure/class packages, add direct first-class `Iff` imports once duplicate `Eq` handoff
is resolved, and strengthen the converse and unsquared-distance statements as the required
nondegeneracy, angle, and square-root cancellation APIs become available.

The intended dependency order is:

```text
EqReasoning
  -> Algebra.Ring -> Algebra.Square -> OrderedField
  -> Vector.Basic -> Vector.Dot
  -> Geometry.RightTriangle -> Geometry.Metric
  -> Logic.Iff
  -> Algebra.AbstractRing -> Algebra.AbstractOrderedField -> Algebra.AbstractSquareNormalize
  -> Algebra.AbstractScalarDerive
  -> Vector.AbstractSpace -> Vector.AbstractInnerProduct -> Vector.AbstractInnerProductDerive
  -> Geometry.Affine -> Geometry.AffineDerive
  -> Geometry.AbstractRightTriangle -> Geometry.AbstractRightTriangleDerive
  -> Geometry.AbstractMetric
  -> Geometry.Pythagorean
```

Recommended module contents:

#### `Proofs.Ai.EqReasoning`

No new definitions live here; the module builds theorem targets over imported `Eq` and the
expected builtin `Eq.rec` axiom interface.

| Theorem | Shape / purpose |
| --- | --- |
| `eq_symm` | `x = y -> y = x` |
| `eq_trans` | `x = y -> y = z -> x = z` |
| `eq_congr_arg` | `x = y -> f x = f y` |
| `eq_congr_fun` | `f = g -> f x = g x` |
| `eq_congr2` | `a = a' -> b = b' -> f a b = f a' b'` |
| `eq_subst` | transport a proof across equality |
| `eq_transport_const` | transport through a constant family |
| `eq_rewrite_left` | rewrite the left side of an equality target |
| `eq_rewrite_right` | rewrite the right side of an equality target |
| `eq_cast_trans` | compose transports through two equalities |
| `eq_calc3` | three-step equality chaining helper for AI-generated calc blocks |

#### `Proofs.Ai.Algebra.Ring`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `RingElem` | singleton scalar carrier for the current concrete corpus layer |
| `zero` | additive identity API |
| `one` | multiplicative identity API |
| `add` | addition API |
| `neg` | additive inverse API |
| `sub` | subtraction API, normally defined as `a + -b` |
| `mul` | multiplication API |

Definitional rewrite theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `a - b = a + -b` |

Algebra law theorem targets. If this module later represents an abstract `Ring` structure, these
may be law fields or projections there; concrete scalar instances still need checked certificates
for the laws.

| Theorem | Shape / purpose |
| --- | --- |
| `add_assoc` | `(a + b) + c = a + (b + c)` |
| `add_comm` | `a + b = b + a` |
| `add_zero` | `a + 0 = a` |
| `zero_add` | `0 + a = a` |
| `neg_add_cancel` | `-a + a = 0` |
| `add_neg_cancel` | `a + -a = 0` |
| `sub_self` | `a - a = 0` |
| `mul_assoc` | `(a * b) * c = a * (b * c)` |
| `mul_comm` | `a * b = b * a` |
| `mul_one` | `a * 1 = a` |
| `one_mul` | `1 * a = a` |
| `mul_zero` | `a * 0 = 0` |
| `zero_mul` | `0 * a = 0` |
| `left_distrib` | `a * (b + c) = a * b + a * c` |
| `right_distrib` | `(a + b) * c = a * c + b * c` |
| `add_left_cancel` | `a + b = a + c -> b = c` |
| `mul_add` | `a * (b + c) = a * b + a * c` |
| `add_mul` | `(a + b) * c = a * c + b * c` |
| `ring_normalize_add_mul3` | small normalization target for sums/products of three terms |

#### `Proofs.Ai.Algebra.Square`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `two` | scalar `2`, normally `1 + 1` |
| `sq` | square operation, normally `sq a := a * a` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_zero` | `sq 0 = 0` |
| `sq_one` | `sq 1 = 1` |
| `sq_neg` | `sq (-a) = sq a` |
| `two_mul` | `2 * a = a + a` |
| `sq_add` | `sq (a + b) = sq a + 2 * a * b + sq b` |
| `sq_sub` | `sq (a - b) = sq a - 2 * a * b + sq b` |
| `sum_two_squares_comm` | `sq a + sq b = sq b + sq a` |
| `sq_eq_sq_of_eq_or_neg_eq` | square equality from an equality-or-negated-equality witness shape |
| `square_nonneg` | predicate-generic bridge to P11's ordered relation work |

#### `Proofs.Ai.OrderedField`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le` | non-strict order relation |
| `lt` | strict order relation |
| `sqrt` | square-root API for the later metric form |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl` | `a <= a` |
| `le_trans` | `a <= b -> b <= c -> a <= c` |
| `add_nonneg` | `0 <= a -> 0 <= b -> 0 <= a + b` |
| `mul_nonneg` | `0 <= a -> 0 <= b -> 0 <= a * b` |
| `le_square_nonneg` | `0 <= sq a` over the ordered singleton scalar carrier; avoids colliding with P10's imported `square_nonneg` bridge |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | nonnegative equality from equal squares |

#### `Proofs.Ai.Vector.Basic`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vec` | vector or point-difference carrier |
| `vec_zero` | vector zero |
| `vec_add` | vector addition |
| `vec_neg` | vector negation |
| `vec_sub` | vector subtraction, normally `u + -v` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_assoc` | `(u + v) + w = u + (v + w)` |
| `vec_add_comm` | `u + v = v + u` |
| `vec_zero_add` | `0 + v = v` |
| `vec_add_zero` | `v + 0 = v` |
| `vec_neg_add_cancel` | `-v + v = 0` |
| `vec_add_neg_cancel` | `v + -v = 0` |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_sub_eq_add_neg` | alias-style subtraction rewrite target for AI search |
| `vec_sub_self` | `v - v = 0` |
| `vec_sub_zero` | `v - 0 = v` |
| `vec_add_left_cancel` | `u + v = u + w -> v = w` |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v`, used for triangle vertices |

#### `Proofs.Ai.Vector.Dot`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | inner product API |
| `normSq` | squared norm, normally `dot v v` |
| `distSq` | squared distance, normally `normSq (B - A)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | `dot u v = dot v u` |
| `dot_add_left` | `dot (u + v) w = dot u w + dot v w` |
| `dot_add_right` | `dot u (v + w) = dot u v + dot u w` |
| `dot_neg_left` | `dot (-u) v = -dot u v` |
| `dot_neg_right` | `dot u (-v) = -dot u v` |
| `dot_sub_left` | `dot (u - v) w = dot u w - dot v w` |
| `dot_sub_right` | `dot u (v - w) = dot u v - dot u w` |
| `norm_sq_def` | `normSq v = dot v v` |
| `dist_sq_def` | `distSq A B = normSq (B - A)` |
| `dot_self_eq_norm_sq` | `dot v v = normSq v` |
| `norm_sq_add` | `normSq (u + v) = normSq u + 2 * dot u v + normSq v` |
| `norm_sq_sub` | `normSq (u - v) = normSq u - 2 * dot u v + normSq v` |
| `norm_sq_add_of_dot_zero` | `dot u v = 0 -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_dot_zero` | `dot u v = 0 -> normSq (u - v) = normSq u + normSq v` |
| `parallelogram_law` | `normSq (u + v) + normSq (u - v) = 2 * normSq u + 2 * normSq v` |
| `polarization_identity` | `2 * dot u v = normSq (u + v) - (normSq u + normSq v)` |
| `norm_sq_nonneg` | `0 <= normSq v` |

#### `Proofs.Ai.Geometry.RightTriangle`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | perpendicularity predicate |
| `RightTriangle` | right-triangle predicate over three points |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | Church-encoded `Perp u v <-> dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg vectors from `RightTriangle A B C` |
| `hypotenuse_vector_eq_sub_legs` | express the hypotenuse vector through the two leg vectors |
| `dist_sq_leg_left` | rewrite the first leg length as a `distSq` term |
| `dist_sq_leg_right` | rewrite the second leg length as a `distSq` term |
| `dist_sq_hypotenuse` | rewrite the hypotenuse length as a `distSq` term |
| `pythagorean_distance_sq` | `RightTriangle A B C -> distSq B C = distSq A B + distSq A C` |
| `law_of_cosines` | peer theorem: squared distance with a dot-product correction term |
| `right_triangle_area` | double-area squared target parameterized by a future `Area2` API |
| `median_to_hypotenuse` | midpoint-on-hypotenuse target parameterized by a future midpoint predicate |
| `altitude_on_hypotenuse` | altitude-foot target parameterized by future length and foot predicates |
| `thales_theorem` | circle/diameter-to-right-triangle target parameterized by a future circle predicate |

#### `Proofs.Ai.Geometry.Metric`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | distance API, normally `sqrt (distSq A B)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSq A B)` |
| `dist_sq_eq_square_dist` | `distSq A B = sq (dist A B)` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | Church-encoded `dist A B = 0 <-> A = B` |
| `pythagorean_distance` | `RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)` |
| `cauchy_schwarz` | `sq (dot u v) <= normSq u * normSq v` |
| `triangle_inequality` | `dist A C <= dist A B + dist B C` |

#### `Proofs.Ai.Logic.Iff`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Iff` | first-class logical equivalence, replacing ad hoc theorem-local equivalence encodings |
| `And` | conjunction API for bundling law hypotheses when a law-package style is useful |
| `Or` | disjunction API for square-root and order case splits |
| `False` | empty proposition API for contradiction and negation eliminators |
| `Not` | negation abbreviation, normally `P -> False` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `iff_refl` | `Iff P P` |
| `iff_symm` | `Iff P Q -> Iff Q P` |
| `iff_trans` | `Iff P Q -> Iff Q R -> Iff P R` |
| `iff_mp` | `Iff P Q -> P -> Q` |
| `iff_mpr` | `Iff P Q -> Q -> P` |
| `and_intro` | `P -> Q -> And P Q` |
| `and_left` | `And P Q -> P` |
| `and_right` | `And P Q -> Q` |
| `iff_of_eq` | `P = Q -> Iff P Q` |
| `false_elim` | `False -> P` |
| `not_intro` | `(P -> False) -> Not P` |
| `not_elim` | `Not P -> P -> False` |
| `or_inl`, `or_inr`, `or_elim` | disjunction introduction and elimination helpers |
| `iff_congr_arg` | `P = Q -> Iff (F P) (F Q)` for Prop-valued contexts |

#### `Proofs.Ai.Algebra.AbstractRing`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Scalar` | local carrier parameter used by every abstract ring definition and theorem target |
| `zero`, `one`, `add`, `neg`, `sub`, `mul` | local operation parameters, keeping the module independent of concrete `RingElem` |
| `two` | parametric scalar `2`, defined from an explicit `one` and `add` |
| `sq` | parametric square helper, defined from an explicit `mul` |
| `RingLawArgs` | Church-encoded law package API over the explicit carrier and operations |

The checked theorem targets take the corresponding law as an explicit argument and return it at the
requested variables. This keeps the corpus certificate-first and avoids adding module-level
unchecked ring axioms. `mul_left_cancel_nonzero` remains deferred until a nonzero predicate is
introduced by a later ordered-field layer.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `sub a b = add a (neg b)` |
| `add_assoc`, `add_comm`, `add_zero`, `zero_add` | additive monoid/group laws |
| `neg_add_cancel`, `add_neg_cancel`, `sub_self` | additive inverse and subtraction laws |
| `mul_assoc`, `mul_comm`, `mul_one`, `one_mul` | commutative multiplication laws |
| `left_distrib`, `right_distrib` | distributivity over addition |
| `mul_zero`, `zero_mul`, `add_left_cancel` | cancellation and zero-product helper targets |
| `ring_normalize_add_mul3` | `((a*b)+(b*c))+(a*c) = ((a*b)+(a*c))+(b*c)` normalization target |
| `add_right_cancel` | `b + a = c + a -> b = c` |
| `neg_neg` | `-(-a) = a` |
| `sub_zero`, `zero_sub` | subtraction by zero and from zero |
| `sub_add_cancel`, `add_sub_cancel` | basic subtraction/addition cancellation lemmas |
| `sub_add_sub_cancel` | `(a - c) - (b - c) = a - b`, scalar analogue of vector displacement cancellation |

#### `Proofs.Ai.Algebra.AbstractOrderedField`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le`, `lt` | parametric adapters for explicit abstract order relation parameters |
| `sqrt` | parametric adapter for an explicit square-root function parameter |
| `Nonneg` | abbreviation for `le zero a`, useful for square-root APIs |
| `Positive` | abbreviation for `lt zero a`, useful for strict metric statements |
| `OrderedFieldLawArgs` | Church-encoded law package API over the explicit carrier, operations, order, and square root |

The checked theorem targets take the corresponding order, square-root, or compatibility law as an
explicit argument and return it at the requested variables. This keeps P18 independent of concrete
`RingElem`, avoids module-level unchecked ordered-field axioms, and uses P17's parametric `two` and
`sq` APIs. Bundled proposition shapes are Church-encoded locally so P18 does not need an additional
logic-module import.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl`, `le_trans` | order reflexivity and transitivity |
| `add_nonneg`, `mul_nonneg` | nonnegative closure under addition and multiplication |
| `square_nonneg` | `0 <= sq a` |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | equality from equal squares under nonnegativity |
| `add_le_add`, `mul_le_mul_nonneg`, `zero_le_two` | order helpers for metric proofs |
| `le_antisymm` | `a <= b -> b <= a -> a = b` |
| `lt_of_le_of_ne` | `0 <= a -> (a = 0 -> False) -> 0 < a`, with `False` Church-encoded |
| `le_of_eq` | equality implies both order directions as a Church-encoded conjunction |
| `sqrt_sq` | `0 <= a -> sq (sqrt a) = a` |
| `sq_eq_zero_iff` | Church-encoded `sq a = 0 <-> a = 0` under the abstract ordered-field assumptions |
| `sum_nonneg_eq_zero` | `0 <= a -> 0 <= b -> a + b = 0 ->` Church-encoded `(a = 0) /\ (b = 0)` |

#### `Proofs.Ai.Algebra.AbstractSquareNormalize`

No new carrier or operation definition lives here. This implemented module provides checked
normalization theorem targets over the P17/P18 abstract scalar APIs.

The checked theorem targets either close by definitional equality (`square_def`,
`mul_self_eq_square`) or take the corresponding normalization/order law as an explicit argument and
return it at the requested variables. This avoids adding unchecked algebra or order axioms while
giving later vector and norm layers stable target names.

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` in the abstract scalar layer |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_add` | expansion of `sq (a + b)` |
| `sq_sub` | expansion of `sq (a - b)` |
| `sum_two_squares_comm` | commutation of a sum of two squares |
| `cancel_double_zero_term` | remove a `2 * x` cross term under `x = 0` |
| `sq_zero`, `sq_one`, `sq_neg`, `two_mul` | square and scalar-2 helper lemmas |
| `sq_eq_sq_of_eq_or_neg_eq` | bridge for later square-root equality arguments |
| `sq_add_eq_add_sq_add_two_mul` | normalization-oriented alias for `sq_add` |
| `sq_sub_eq_add_sq_sub_two_mul` | normalization-oriented alias for `sq_sub` |
| `add_sq_eq_zero_iff` | sum of nonnegative squares is zero only when both terms are zero |
| `mul_two_zero_term` | `x = 0 -> 2 * x = 0`, used by later norm expansion |
| `normalize_add_with_zero_cross_term` | scalar-only normal form used by `norm_sq_add_of_dot_zero` |

#### `Proofs.Ai.Algebra.AbstractScalarDerive`

No new carrier or operation definition lives here. This implemented module derives scalar
normalization helpers from the P17 `RingLawArgs` package and `Std.Logic.Eq` equality transport,
while importing the P19 square-normalization layer for the Pythagorean scalar stack.

The checked theorem targets use equality transport, so the module records the expected `Eq.rec`
dependency. They do not accept
`normalize_add_with_zero_cross_term_law` as a direct theorem-shaped argument.

| Theorem | Shape / purpose |
| --- | --- |
| `mul_two_zero_term_from_ring_args` | `x = 0 -> 2 * x = 0`, derived from `RingLawArgs` |
| `cancel_double_zero_term_from_ring_args` | `x = 0 -> a + 2 * x = a` |
| `normalize_add_with_zero_cross_term_from_ring_args` | `x = 0 -> (a + 2 * x) + b = a + b` |

#### `Proofs.Ai.Vector.AbstractSpace`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vector` | local abstract vector carrier parameter over the scalar API |
| `vzero`, `vadd`, `vneg`, `smul` | local vector operation parameters |
| `vsub` | parametric vector subtraction helper, defined as `vadd x (vneg y)` |
| `linear_comb2` | helper API for two-term linear combinations, useful for generated proof terms |
| `linear_comb3` | helper API for three-term linear combinations in affine point proofs |
| `VectorSpaceLawArgs` | Church-encoded law package API over the explicit scalar/vector operations |

The checked theorem targets either close by definitional equality (`vec_sub_def`,
`vec_sub_eq_add_neg`, `linear_comb2_ext`, `linear_comb3_ext`) or take the corresponding vector-space
law as an explicit argument and return it at the requested variables. The vector subtraction alias
uses the `vec_` prefix to avoid colliding with P17's scalar `sub_eq_add_neg` declaration in the
kernel import environment.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_add_assoc`, `vec_add_comm`, `vec_add_zero`, `vec_zero_add` | additive vector laws |
| `vec_neg_add_cancel`, `vec_add_neg_cancel` | vector inverse laws |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v`, used for triangle vertices |
| `vec_sub_self`, `vec_sub_zero`, `vec_add_left_cancel` | vector subtraction and cancellation helpers |
| `smul_add`, `add_smul`, `one_smul`, `mul_smul` | scalar multiplication laws |
| `zero_smul`, `smul_zero` | zero scalar/vector multiplication |
| `neg_smul`, `smul_neg` | scalar multiplication and negation interaction |
| `vec_sub_eq_add_neg` | vector subtraction rewrite alias for search consistency |
| `sub_add_sub_cancel_left` | `(u - w) + (w - v) = u - v` displacement-style cancellation |
| `linear_comb2_ext` | expansion theorem for `linear_comb2` |
| `linear_comb3_ext` | expansion theorem for `linear_comb3` |

#### `Proofs.Ai.Vector.AbstractInnerProduct`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | parametric wrapper for a local abstract inner-product operation |
| `normSq` | squared norm, defined as `dot v v` |
| `distSq` | vector-level squared distance, defined as `normSq (vsub B A)`; P22 adds point-level `distSqPoints` |
| `PerpVec` | vector-level perpendicular predicate, defined as `dot u v = 0` |
| `InnerProductLawArgs` | Church-encoded law package API for symmetry, bilinearity, norm expansion, and positivity hypotheses |

The checked theorem targets either close by definitional equality (`norm_sq_def`,
`dist_sq_def`, `dot_self_eq_norm_sq`) or take the corresponding inner-product, norm-expansion, or
positivity law as an explicit argument and return it at the requested variables.
`perp_vec_iff_dot_eq_zero` and `norm_sq_zero_iff` use the same Church-encoded iff shape as P16's
`Iff` API without importing `Proofs.Ai.Logic.Iff`, because the current source handoff cannot combine
that module with the abstract algebra imports without duplicating the imported `Eq` declaration.
`cauchy_schwarz` uses the squared form `sq (dot u v) <= normSq u * normSq v`, avoiding a square-root
dependency at this layer.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | inner-product symmetry |
| `dot_add_left`, `dot_add_right` | additivity in each argument |
| `dot_neg_left`, `dot_neg_right` | negation in each argument |
| `dot_sub_left`, `dot_sub_right` | subtraction expansion in each argument |
| `norm_sq_def`, `dot_self_eq_norm_sq` | squared norm definition and reverse rewrite |
| `dist_sq_def` | squared distance definition after affine displacement is available |
| `norm_sq_add`, `norm_sq_sub` | squared norm expansion |
| `norm_sq_add_of_dot_zero`, `norm_sq_sub_of_dot_zero` | Pythagorean norm steps under perpendicularity |
| `norm_sq_nonneg`, `cauchy_schwarz` | positivity and Cauchy-Schwarz targets |
| `parallelogram_law`, `polarization_identity` | peer inner-product theorem targets |
| `perp_vec_iff_dot_eq_zero` | iff-shaped equivalence between `PerpVec u v` and `dot u v = 0` |
| `perp_vec_symm` | vector-level perpendicularity symmetry |
| `norm_sq_zero_iff` | iff-shaped `normSq v = 0 <-> v = 0` under positive-definiteness |
| `dist_sq_nonneg` | `0 <= distSq A B` after affine distance is connected |
| `norm_sq_add_of_perp` | `PerpVec u v -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_perp` | `PerpVec u v -> normSq (u - v) = normSq u + normSq v` |

#### `Proofs.Ai.Vector.AbstractInnerProductDerive`

No new vector or scalar operation definition lives here. This implemented module derives the
norm-expansion path needed by the abstract Pythagorean route from `InnerProductLawArgs`, the P27
`Proofs.Ai.Algebra.AbstractScalarDerive` zero-cross-term rewrite, and `Std.Logic.Eq` equality
transport.

The checked theorem targets record the expected `Eq.rec` dependency. They do not accept
`norm_sq_add_of_dot_zero_law` or `norm_sq_add_of_perp_law` as direct theorem-shaped arguments; the
perpendicular theorem accepts `RingLawArgs`, `InnerProductLawArgs`, and `PerpVec`.

| Theorem | Shape / purpose |
| --- | --- |
| `norm_sq_add_from_inner_args` | projects the primitive `normSq (x + y)` expansion from `InnerProductLawArgs` |
| `norm_sq_add_of_dot_zero_from_args` | `dot x y = 0 -> normSq (x + y) = normSq x + normSq y` using the P27 scalar rewrite |
| `norm_sq_add_of_perp_from_args` | `PerpVec x y -> normSq (x + y) = normSq x + normSq y` |

#### `Proofs.Ai.Geometry.Affine`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Point` | parametric wrapper for an abstract point carrier |
| `disp` | parametric wrapper for a displacement vector from one point to another |
| `distSqPoints` | point-level squared distance, defined as `normSq (disp A B)` |
| `translate` | point translation API wrapper for later affine law statements |
| `midpoint` | midpoint API wrapper for later right-triangle geometry |
| `collinear` | collinearity predicate wrapper for later geometric sanity lemmas |
| `AffineLawArgs` | Church-encoded law package API for point/vector compatibility hypotheses |

`distSqPoints` is intentionally separate from P21's vector-level `distSq`, so the affine layer can
state point-distance lemmas without colliding with the imported vector-distance declaration. The
checked theorem targets either close by definitional equality (`dist_sq_points_def`) or take the
corresponding affine compatibility law as an explicit argument and return it at the requested
points. `AffineLawArgs` itself keeps only the primitive affine compatibility fields needed by later
derivation layers; the theorem-shaped hypotenuse-vector and point-distance-definition fields were
removed from the law package in P29.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `disp_self` | `disp A A = 0` |
| `disp_reverse` | `disp B A = -disp A B` |
| `disp_comp` | `disp A C = disp A B + disp B C` |
| `hypotenuse_vector_eq_sub_legs` | express the hypotenuse displacement through two leg displacements |
| `dist_sq_points_def` | `distSqPoints A B = normSq (disp A B)` |
| `point_ext_of_zero_disp` | zero displacement implies point equality |
| `dist_sq_symm` | point squared-distance symmetry |
| `dist_sq_zero_iff_eq` | iff-shaped point squared-distance nondegeneracy |

#### `Proofs.Ai.Geometry.AffineDerive`

No new point, vector, or scalar operation definition lives here. This implemented module derives
the affine orientation path needed by the abstract Pythagorean route from primitive
`AffineLawArgs`, `VectorSpaceLawArgs`, and `Std.Logic.Eq` equality transport.

The checked theorem targets record the expected `Eq.rec` dependency. They do not accept
`hypotenuse_vector_eq_sub_legs_law` or `dist_sq_points_def_law` as direct theorem-shaped
arguments; the hypotenuse orientation is built from `disp_comp`, `disp_reverse`, vector addition
commutativity, and the definitional `vsub` / `distSqPoints` expansions.

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_comm_from_vector_args` | projects vector addition commutativity from `VectorSpaceLawArgs` |
| `disp_reverse_from_affine_args` | projects the primitive reverse displacement law from `AffineLawArgs` |
| `disp_comp_from_affine_args` | projects the primitive displacement composition law from `AffineLawArgs` |
| `dist_sq_points_def_from_args` | `distSqPoints X Y = normSq (disp X Y)` by definition |
| `hypotenuse_vector_eq_neg_left_add_right_from_args` | `disp B C = vadd (vneg (disp A B)) (disp A C)` |
| `hypotenuse_vector_eq_sub_legs_from_args` | `disp B C = vsub (disp A C) (disp A B)` |
| `dist_sq_hypotenuse_norm_neg_left_add_right_from_args` | rewrites `distSqPoints B C` to the norm of the additive hypotenuse orientation |
| `dist_sq_hypotenuse_norm_sub_legs_from_args` | rewrites `distSqPoints B C` to the norm of the subtraction hypotenuse orientation |

#### `Proofs.Ai.Geometry.AbstractRightTriangle`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | vector-level perpendicularity predicate, defined through P21 `PerpVec` |
| `RightTriangle` | point-level right-triangle predicate, with the right angle at `A` |
| `AngleRight` | angle-level predicate wrapper for later APIs that separate angle from triangle |
| `Area2` | doubled-area API wrapper for right-triangle area theorem targets |
| `FootOnHypotenuse` | altitude-foot predicate wrapper for later classical right-triangle targets |

The checked theorem targets either close by definition (`perp_iff_dot_eq_zero`,
`right_triangle_legs_perp`) or take the corresponding perpendicularity, Pythagorean,
law-of-cosines, area, or median law as an explicit argument and return it at the requested points.
`perp_iff_dot_eq_zero` uses the same iff-shaped Church encoding as P21's perpendicularity theorem
target without importing `Proofs.Ai.Logic.Iff`.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | iff-shaped equivalence between `Perp u v` and `dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg displacement vectors |
| `pythagorean_distance_sq_general` | `RightTriangle A B C -> distSqPoints B C = distSqPoints A B + distSqPoints A C` |
| `law_of_cosines_general` | squared-distance law of cosines over the abstract inner product |
| `right_triangle_area_general`, `median_to_hypotenuse_general` | same-level classical right-triangle targets |

#### `Proofs.Ai.Geometry.AbstractRightTriangleDerive`

No new geometric definition lives here. This implemented module derives the bridge needed by the
abstract Pythagorean route while keeping `RightTriangle A B C` as the public geometric hypothesis.
The key final premise matches P29's additive hypotenuse orientation:
`PerpVec (vneg (disp A B)) (disp A C)`.

The checked theorem targets record the expected `Eq.rec` dependency for equality transport. They do
not accept a theorem argument whose conclusion is already the Pythagorean equality.

| Theorem | Shape / purpose |
| --- | --- |
| `neg_zero_from_ring_args` | derives `-0 = 0` from `RingLawArgs` |
| `dot_neg_left_from_inner_args` | projects `dot (-x) y = -(dot x y)` from `InnerProductLawArgs` |
| `right_triangle_legs_perp_vec_from_rt` | `RightTriangle A B C -> PerpVec (disp A B) (disp A C)` by unfolding |
| `right_triangle_legs_dot_zero_from_rt` | `RightTriangle A B C -> dot (disp A B) (disp A C) = 0` by unfolding |
| `right_triangle_neg_left_dot_zero_from_rt` | derives `dot (-(disp A B)) (disp A C) = 0` |
| `right_triangle_neg_left_perp_vec_from_rt` | final P28 premise `PerpVec (-(disp A B)) (disp A C)` |
| `right_triangle_affine_additive_perp_bridge_from_rt` | packages P29's additive hypotenuse orientation with the matching P28 perpendicular premise |

#### `Proofs.Ai.Geometry.AbstractMetric`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | abstract distance API, defined as `sqrt (distSqPoints A B)` |
| `MetricSpaceLawArgs` | Church-encoded law package for distance definition, nonnegativity, symmetry, zero-distance equivalence, and triangle inequality exports |
| `Ball` | closed-ball API, defined through `dist center x <= radius` |

`MetricSpaceLawArgs` no longer carries a direct `distSqPoints = sq dist` bridge or a direct
metric Pythagorean field. `dist_def` closes by definitional equality. The P32 metric bridge derives
`sq (dist A B) = distSqPoints A B` from the primitive ordered-field `sqrt_sq` field and
`distSqPoints A B >= 0`, with the nonnegativity proof projected from `InnerProductLawArgs`; the
reverse bridge uses the audited `Eq.rec` equality transport. The remaining metric facts such as
symmetry and triangle inequality are still explicit metric-law wrappers.
`distance_zero_iff_eq` uses the same iff-shaped Church encoding as earlier geometry targets rather
than importing `Proofs.Ai.Logic.Iff` directly into this metric layer.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSqPoints A B)` |
| `point_dist_sq_nonneg_from_inner_args` | derives `0 <= distSqPoints A B` from `InnerProductLawArgs` |
| `square_dist_eq_dist_sq_from_law_packages` | derives `sq (dist A B) = distSqPoints A B` from `sqrt_sq` and nonnegativity |
| `dist_sq_eq_square_dist_from_law_packages` | reverses the bridge to `distSqPoints A B = sq (dist A B)` |
| `dist_sq_eq_square_dist` | public bridge alias backed by P32 law-package derivation |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | iff-shaped equivalence between `dist A B = 0` and `A = B` |
| `pythagorean_distance_general` | legacy explicit metric Pythagorean wrapper, not used by the final P32 path |
| `triangle_inequality` | `dist A C <= dist A B + dist B C` |

#### `Proofs.Ai.Geometry.Pythagorean`

No new API declarations live here. This final module collects the abstract prerequisites and
exports theorem names that users can depend on.

Implemented imports:

| Import | Purpose |
| --- | --- |
| `Std.Logic.Eq` | equality target statements and explicit certificate dependency |
| `Proofs.Ai.EqReasoning` | equality transitivity, symmetry, congruence, and transport helpers |
| `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Algebra.AbstractScalarDerive` | scalar law packages and checked zero-cross-term normalization |
| `Proofs.Ai.Vector.AbstractSpace`, `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Vector.AbstractInnerProductDerive` | vector-space, inner-product, and checked perpendicular norm-addition derivations |
| `Proofs.Ai.Geometry.Affine`, `Proofs.Ai.Geometry.AffineDerive` | point displacement API, hypotenuse orientation, and point-distance/norm bridges |
| `Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | right-triangle hypotheses and checked perpendicular bridge |
| `Proofs.Ai.Geometry.AbstractMetric` | distance API and metric theorem bridge |

The P31 squared-distance theorem is `pythagorean_distance_sq_from_law_packages`. It composes P29's
hypotenuse distance/norm bridge, P30's `RightTriangle` to perpendicular bridge, P28's perpendicular
norm-addition derivation, and small affine symmetry/reversal bridges in this module. It takes
`RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, `AffineLawArgs`, and
`RightTriangle A B C`; it does not accept a direct Pythagorean equality law.

`pythagorean_theorem_sq`, `law_of_cosines_right_angle_specialization`, and
`pythagorean_theorem_api_alias` delegate to the checked P31 derivation. `pythagorean_theorem_dist_sq`
now composes P31's squared-distance theorem with P32's metric bridge, so it no longer accepts a
direct metric Pythagorean law. The converse remains an explicit target until the nondegeneracy and
angle APIs are strong enough. P34 also leaves the unsquared distance form unexported: the current
ordered-field and metric layers can justify squared metric-distance equality, but they do not yet
provide a checked square-root cancellation path for the full Pythagorean right-hand side.
The module axiom report is `["Eq.rec"]`; this is the documented equality-recursion exception
inherited from imported equality reasoning and transport lemmas, not a geometry or metric axiom.
`Proofs.Ai.Logic.Iff` is not directly imported here because the current source handoff cannot
combine that module with the abstract geometry imports without duplicating the imported `Eq`
declaration.

| Theorem | Shape / purpose |
| --- | --- |
| `pythagorean_dist_sq_symm_from_affine_args` | extracts point squared-distance symmetry from `AffineLawArgs` |
| `pythagorean_dist_sq_reverse_norm_neg_from_law_packages` | rewrites `distSqPoints B A` to `normSq (vneg (disp A B))` |
| `pythagorean_left_leg_norm_neg_from_law_packages` | identifies `normSq (vneg (disp A B))` with `distSqPoints A B` |
| `pythagorean_distance_sq_from_law_packages` | checked squared-distance Pythagorean theorem from law packages and `RightTriangle A B C` |
| `pythagorean_theorem_sq` | public squared-distance theorem delegating to the checked P31 derivation |
| `pythagorean_theorem_dist_sq` | squared metric-distance theorem derived from P31 plus the P32 metric bridge |
| `pythagorean_converse_sq` | explicit converse target, not a completed theorem derivation, until the required nondegeneracy and angle API are available |
| `law_of_cosines_right_angle_specialization` | right-angle specialization alias backed by the checked squared-distance theorem |
| `pythagorean_theorem_api_alias` | stable alias backed by the checked squared-distance theorem |
| `pythagorean_theorem_dependencies` | documentation theorem or metadata target listing required law packages |

Regenerate the corpus:

```sh
cargo run -p npa-proof-corpus
```

Verify the checked-in corpus:

```sh
cargo test -p npa-proof-corpus
```
