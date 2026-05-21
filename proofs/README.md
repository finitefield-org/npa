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
- `Proofs/Ai/Eq/`: equality refl theorem module importing `Std.Logic.Eq`.
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
interfaces and builtin equality references without adding rewrite search as a dependency.

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

Regenerate the corpus:

```sh
cargo run -p npa-frontend --example write_ai_proof_artifacts
```

Verify the checked-in corpus:

```sh
cargo test -p npa-cert --test ai_proof_artifacts
```
