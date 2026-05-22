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
- `Proofs/Ai/Geometry/RightTriangle/`: right-triangle and squared-distance Pythagoras theorem
  targets importing vector dot and scalar corpus layers.
- `Proofs/Ai/Geometry/Metric/`: distance API and metric theorem targets importing the right-triangle
  and vector dot layers.
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

### P16+: Pythagorean Theorem Roadmap

Long-term target: prove the Pythagorean theorem as a checked certificate. Prefer the coordinate /
inner-product route first:

```text
RightTriangle A B C -> distSq B C = distSq A B + distSq A C
```

This avoids making the first target depend on square roots. P15 adds the checked bridge to the
squared `dist` form over the current concrete scalar and vector corpus.

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

No additional Pythagorean-critical layer is currently enumerated in this roadmap. Add future P16+
geometry layers here once the next API boundary is selected.

The intended dependency order is:

```text
EqReasoning
  -> Algebra.Ring -> Algebra.Square -> OrderedField
  -> Vector.Basic -> Vector.Dot
  -> Geometry.RightTriangle -> Geometry.Metric
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

Regenerate the corpus:

```sh
cargo run -p npa-frontend --example write_ai_proof_artifacts
```

Verify the checked-in corpus:

```sh
cargo test -p npa-cert --test ai_proof_artifacts
```
