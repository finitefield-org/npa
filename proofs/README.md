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

Planned:

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

Planned:

| Theorem | Shape |
| --- | --- |
| `beta_id_nat` | `Nat -> Nat`, with proof `(fun x => x) n` |
| `beta_const_nat` | `Nat -> Nat -> Nat`, with proof `(fun x => fun _ => x) n m` |
| `let_id_nat` | `Nat -> Nat`, with proof `let x : Nat := n in x` |
| `let_const_nat` | `Nat -> Nat`, with proof `let z : Nat := Nat.zero in z` |
| `delta_id_nat` | `Nat -> Nat` through a local named identity definition |

Regenerate the corpus:

```sh
cargo run -p npa-frontend --example write_ai_proof_artifacts
```

Verify the checked-in corpus:

```sh
cargo test -p npa-cert --test ai_proof_artifacts
```
