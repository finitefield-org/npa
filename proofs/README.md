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
- `Proofs/Ai/Nat/`: Nat smoke theorem module importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs/Ai/Prop/`: import-free proposition-only implication search module.
- `Proofs/Ai/Reduction/`: reduction smoke theorem module importing `Std.Nat.Basic`.
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

| Theorem | Shape |
| --- | --- |
| `beta_id_nat` | `Nat -> Nat`, with proof `(fun x => x) n` |
| `beta_const_nat` | `Nat -> Nat -> Nat`, with proof `(fun x => fun _ => x) n m` |
| `let_id_nat` | `Nat -> Nat`, with proof `let x : Nat := n in x` |
| `let_const_nat` | `Nat -> Nat`, with proof `let z : Nat := Nat.zero in z` |
| `delta_id_nat` | `Nat -> Nat` through a local named identity definition |

### P8+: Pythagorean Theorem Roadmap

Long-term target: prove the Pythagorean theorem as a checked certificate. Prefer the coordinate /
inner-product route first:

```text
RightTriangle A B C -> distSq B C = distSq A B + distSq A C
```

This avoids making the first target depend on square roots. The later metric statement using
`dist` can be added after nonnegative square roots and ordered-field facts are stable.

Planned prerequisite and peer theorem targets:

| Layer | Module | Required for Pythagorean theorem | Same-level important theorem targets |
| --- | --- | --- | --- |
| P8 | `Proofs.Ai.EqReasoning` | `eq_symm`, `eq_trans`, `eq_congr_arg`, `eq_congr_fun`, `eq_subst` | `eq_congr2`, `eq_transport_const`, `eq_cast_trans` |
| P9 | `Proofs.Ai.Algebra.Ring` | `add_assoc`, `add_comm`, `add_zero`, `zero_add`, `neg_add_cancel`, `sub_eq_add_neg`, `mul_assoc`, `mul_comm`, `left_distrib`, `right_distrib` | `mul_zero`, `zero_mul`, `sub_self`, `add_left_cancel`, `mul_add`, `add_mul` |
| P10 | `Proofs.Ai.Algebra.Square` | `square_def`, `mul_self_eq_square`, `sq_add`, `sq_sub`, `sq_neg`, `two_mul` | `sum_two_squares_comm`, `sq_eq_sq_of_eq_or_neg_eq`, `square_nonneg` |
| P11 | `Proofs.Ai.OrderedField` | `add_nonneg`, `mul_nonneg`, `square_nonneg`, `sqrt_square_of_nonneg` for the later unsquared metric form | `sqrt_mul_self`, `dist_nonneg`, `eq_of_square_eq_square_nonneg` |
| P12 | `Proofs.Ai.Vector.Basic` | `vec_add_assoc`, `vec_add_comm`, `vec_zero_add`, `vec_neg_add_cancel`, `vec_sub_def`, `sub_sub_sub_cancel` | `vec_add_left_cancel`, `vec_sub_self`, `vec_sub_zero`, `vec_sub_eq_add_neg` |
| P13 | `Proofs.Ai.Vector.Dot` | `dot_add_left`, `dot_add_right`, `dot_neg_left`, `dot_neg_right`, `dot_sub_left`, `dot_sub_right`, `dot_comm`, `norm_sq_def`, `dist_sq_def` | `parallelogram_law`, `polarization_identity`, `norm_sq_nonneg`, `dot_self_eq_norm_sq` |
| P14 | `Proofs.Ai.Geometry.RightTriangle` | `perp_iff_dot_eq_zero`, `right_triangle_legs_perp`, `hypotenuse_vector_eq_sub_legs`, `norm_sq_add_of_dot_zero`, `pythagorean_distance_sq` | `law_of_cosines`, `thales_theorem`, `right_triangle_area`, `median_to_hypotenuse`, `altitude_on_hypotenuse` |
| P15 | `Proofs.Ai.Geometry.Metric` | `dist_def`, `dist_sq_eq_square_dist`, `pythagorean_distance` | `distance_symm`, `distance_zero_iff_eq`, `triangle_inequality`, `cauchy_schwarz` |

The intended dependency order is:

```text
EqReasoning
  -> Algebra.Ring -> Algebra.Square -> OrderedField
  -> Vector.Basic -> Vector.Dot
  -> Geometry.RightTriangle -> Geometry.Metric
```

Recommended theorem contents by module:

#### `Proofs.Ai.EqReasoning`

| Theorem | Shape / purpose |
| --- | --- |
| `eq_symm` | `x = y -> y = x` |
| `eq_trans` | `x = y -> y = z -> x = z` |
| `eq_congr_arg` | `x = y -> f x = f y` |
| `eq_congr_fun` | `f = g -> f x = g x` |
| `eq_congr2` | `a = a' -> b = b' -> f a b = f a' b'` |
| `eq_subst` | transport a proof across equality |
| `eq_rewrite_left` | rewrite the left side of an equality target |
| `eq_rewrite_right` | rewrite the right side of an equality target |
| `eq_calc3` | three-step equality chaining helper for AI-generated calc blocks |

#### `Proofs.Ai.Algebra.Ring`

| Theorem | Shape / purpose |
| --- | --- |
| `add_assoc` | `(a + b) + c = a + (b + c)` |
| `add_comm` | `a + b = b + a` |
| `add_zero` | `a + 0 = a` |
| `zero_add` | `0 + a = a` |
| `neg_add_cancel` | `-a + a = 0` |
| `add_neg_cancel` | `a + -a = 0` |
| `sub_eq_add_neg` | `a - b = a + -b` |
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
| `ring_normalize_add_mul3` | small normalization target for sums/products of three terms |

#### `Proofs.Ai.Algebra.Square`

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
| `square_nonneg` | `0 <= sq a`, used by the metric form |

#### `Proofs.Ai.OrderedField`

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl` | `a <= a` |
| `le_trans` | `a <= b -> b <= c -> a <= c` |
| `add_nonneg` | `0 <= a -> 0 <= b -> 0 <= a + b` |
| `mul_nonneg` | `0 <= a -> 0 <= b -> 0 <= a * b` |
| `square_nonneg` | `0 <= sq a` if it is not imported from `Algebra.Square` |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | nonnegative equality from equal squares |

#### `Proofs.Ai.Vector.Basic`

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_assoc` | `(u + v) + w = u + (v + w)` |
| `vec_add_comm` | `u + v = v + u` |
| `vec_zero_add` | `0 + v = v` |
| `vec_add_zero` | `v + 0 = v` |
| `vec_neg_add_cancel` | `-v + v = 0` |
| `vec_add_neg_cancel` | `v + -v = 0` |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_sub_self` | `v - v = 0` |
| `vec_sub_zero` | `v - 0 = v` |
| `vec_add_left_cancel` | `u + v = u + w -> v = w` |
| `sub_sub_sub_cancel` | vector subtraction rearrangement used for triangle vertices |

#### `Proofs.Ai.Vector.Dot`

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
| `polarization_identity` | expresses `dot u v` using squared norms |

#### `Proofs.Ai.Geometry.RightTriangle`

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | `Perp u v <-> dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg vectors from `RightTriangle A B C` |
| `right_triangle_not_collinear` | optional geometric sanity theorem |
| `hypotenuse_vector_eq_sub_legs` | express the hypotenuse vector through the two leg vectors |
| `dist_sq_leg_left` | rewrite the first leg length as a `distSq` term |
| `dist_sq_leg_right` | rewrite the second leg length as a `distSq` term |
| `dist_sq_hypotenuse` | rewrite the hypotenuse length as a `distSq` term |
| `pythagorean_distance_sq` | `RightTriangle A B C -> distSq B C = distSq A B + distSq A C` |
| `law_of_cosines` | peer theorem: squared distance with a dot-product correction term |
| `right_triangle_area` | area of a right triangle from its two legs |
| `median_to_hypotenuse` | classical right-triangle theorem |
| `altitude_on_hypotenuse` | classical right-triangle theorem |
| `thales_theorem` | circle/diameter characterization of right angles |

#### `Proofs.Ai.Geometry.Metric`

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSq A B)` |
| `dist_sq_eq_square_dist` | `distSq A B = sq (dist A B)` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | `dist A B = 0 <-> A = B` |
| `pythagorean_distance` | `RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)` or the square-root form when available |
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
