# npa-mathlib Analysis Implicit Function Closure Audit

Date: 2026-06-04

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.26`
analysis inverse-function closure. It is a sidecar planning and release record,
not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, publish plans, release artifacts, and this
audit are untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.26`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-inverse-function-closure-audit.md`.
- `Mathlib.Logic.EqReasoning`,
  `Mathlib.LinearAlgebra.VectorSpace`,
  `Mathlib.Analysis.NormedSpace.Basic`,
  `Mathlib.Analysis.LinearMap`, and
  `Mathlib.Analysis.Calculus.Derivative` are already public and are available
  as source-free dependencies for this closure.

This closure should materialize as `npa-mathlib v0.1.27`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | `Mathlib.Analysis.Calculus.ImplicitFunction.Phi` | `Mathlib/Analysis/Calculus/ImplicitFunction/Phi/` | 5 definitions, 13 theorems | `Std.Logic.Eq`, equality reasoning, vector space, normed space, linear map, derivative | transitive `Eq.rec` |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | `Mathlib.Analysis.Calculus.ImplicitFunction` | `Mathlib/Analysis/Calculus/ImplicitFunction/` | 11 definitions, 33 theorems | `Std.Logic.Eq`, equality reasoning, vector space, normed space, linear map, derivative, implicit Phi | transitive `Eq.rec` |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | `Mathlib.Analysis.Calculus.ImplicitFunction.Phi` |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | `Mathlib.Analysis.Calculus.ImplicitFunction` |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.Calculus.ImplicitFunction` because
the selected route introduces the auxiliary implicit `Phi` map, implicit
function extraction, differentiability evidence, derivative evidence, and the
final implicit-function theorem route. The auxiliary Phi surface is kept in
`Mathlib.Analysis.Calculus.ImplicitFunction.Phi` because the final theorem
module imports it directly.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractImplicitPhi` and
`Proofs.Ai.Analysis.AbstractImplicitFunction` in one closure:

- `AbstractImplicitFunction` imports `AbstractImplicitPhi` directly, so
  shipping the final theorem without Phi would leave an unpublished public
  dependency;
- the pair has one coherent calculus purpose: building the implicit Phi map and
  extracting the implicit-function theorem and derivative theorem surfaces;
- all other dependencies are already public foundations in `npa-mathlib`.

The declaration-name collision scan against existing public `Mathlib/` source
found no matching `def` or `theorem` names for this closure.

## Explicitly Deferred Modules

No nearby `Proofs.Ai.Analysis.*` modules remain deferred after this closure.
Lower-priority algebra, functional-analysis, and algebraic-geometry seeds
remain outside this analysis closure and should be audited separately.

## Public Declaration Inventory

`Mathlib.Analysis.Calculus.ImplicitFunction.Phi` definitions:

- `ImplicitPhiCoord`
- `ImplicitPhi`
- `ImplicitPhiDerivativeMap`
- `ImplicitPhiDerivativeArgs`
- `ImplicitPhiIsoArgs`

`Mathlib.Analysis.Calculus.ImplicitFunction.Phi` theorems:

- `implicit_phi_coord_def`
- `implicit_phi_def`
- `implicit_phi_coord_base_value_from_zero`
- `implicit_phi_derivative_map_def`
- `implicit_phi_full_derivative_from_args`
- `implicit_phi_partial_x_from_args`
- `implicit_phi_partial_y_from_args`
- `implicit_phi_derivative_from_args`
- `implicit_phi_dy_iso_from_args`
- `implicit_phi_block_triangular_args_from_args`
- `implicit_phi_linear_iso_from_args`
- `implicit_phi_block_left_inverse_from_args`
- `implicit_phi_block_right_inverse_from_args`

`Mathlib.Analysis.Calculus.ImplicitFunction` definitions:

- `ImplicitTargetPoint`
- `ImplicitFunction`
- `ImplicitGraphPoint`
- `ImplicitTargetDerivativeMap`
- `ImplicitFunctionDerivativeChainMap`
- `ImplicitFunctionDerivativeFormulaMap`
- `ImplicitPhiLocalInverseLaws`
- `ImplicitFunctionExtractionArgs`
- `ImplicitFunctionDerivativeArgs`
- `ImplicitFunctionTheoremEvidence`
- `ImplicitFunctionDerivativeEvidence`

`Mathlib.Analysis.Calculus.ImplicitFunction` theorems:

- `implicit_target_point_def`
- `implicit_function_def`
- `implicit_graph_point_def`
- `implicit_target_derivative_map_def`
- `implicit_function_derivative_chain_map_def`
- `implicit_function_derivative_formula_map_def`
- `implicit_extraction_local_inverse_from_args`
- `implicit_extraction_target_mem_from_args`
- `implicit_function_value_mem_from_args`
- `implicit_function_zero_from_args`
- `implicit_function_unique_from_args`
- `implicit_function_derivative_extraction_args_from_args`
- `implicit_function_target_derivative_from_args`
- `implicit_function_partial_x_from_derivative_args`
- `implicit_function_partial_y_from_derivative_args`
- `implicit_function_dy_iso_from_derivative_args`
- `implicit_function_phi_inverse_derivative_from_args`
- `implicit_function_snd_projection_derivative_from_args`
- `implicit_function_derivative_from_args`
- `implicit_function_differentiable_from_args`
- `implicit_function_derivative_formula_from_args`
- `implicit_function_theorem_args_from_evidence`
- `implicit_function_theorem_target_mem_from_evidence`
- `implicit_function_theorem_value_mem_from_evidence`
- `implicit_function_theorem_zero_from_evidence`
- `implicit_function_theorem_unique_from_evidence`
- `implicit_function_theorem`
- `implicit_function_derivative_evidence_args`
- `implicit_function_derivative_evidence_basic`
- `implicit_function_derivative_evidence_differentiable`
- `implicit_function_derivative_evidence_derivative`
- `implicit_function_derivative_evidence_formula`
- `implicit_function_derivative_theorem`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | rewrite to public equality-reasoning foundation |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | rewrite to public normed-space foundation |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` | rewrite to public bounded-linear-map foundation |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` | rewrite to public derivative foundation |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | `Mathlib.Analysis.Calculus.ImplicitFunction.Phi` | rewrite final module dependency to public Phi module |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractImplicitPhi` or
`Proofs.Ai.Analysis.AbstractImplicitFunction` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | none | `Eq.rec` | ok |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | none | `Eq.rec` | ok |

The manifest records `Eq.rec` as the allowed axiom surface for both modules.
Neither module adds a direct custom axiom; both inherit the policy-approved
`Eq.rec` equality-reasoning dependency.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | `sha256:0951636d8795d0830bcff0f6e95b03a3bca72f1036f77f3cd2bcc057785353fa` | `sha256:2fffc97bf1c1d7fd7094c3551b4b141ea9dc3463c54b556bb01ede065c17f753` | `sha256:dd9c653c329eb71aacd3311c389bac98094fa5e7697eb621eeec8e94c9b6db99` | `sha256:9f292648aa00dddbe49e31f78bc11448b5ee88db69aba9ec37edaa67a112d59b` | `sha256:d3bcc891183a1bd16e7bcd4032ab782a9906b005a2ae4f472f41de3be98e721c` |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | `sha256:1d4f94f5dc845cdef89b0f8b66707eb803bdf7531e49a362e758809378bc6fee` | `sha256:9e1bb4268c2a63c8cf805e17878f0dbde30437ed93ad1f24000849dec9f93b9d` | `sha256:d6d7f5d6fec41a2d92ab07f456fc54ce3b83901808de8df555931c77a857fdd2` | `sha256:0d1690c6eba73dbd259c7cfc6f42702c48cb349a31bca7078cd2a76bb5ac6fcf` | `sha256:74f05808dcf79330c403ffa4364b9efe74959f29d8e73a559c9cd563e67ae18b` |

## Corpus Verification

The selected Phi corpus module passed source-free module verification in a
clean temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractImplicitPhi
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractImplicitPhi
verified 1 selected module(s), 11 module(s) including dependency cache
```

The selected final corpus module passed source-free module verification in the
same clean temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractImplicitFunction
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractImplicitFunction
verified 1 selected module(s), 12 module(s) including dependency cache
```

The selected closure also passed final-module source rebuild in the same clean
temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractImplicitFunction
```

Observed result:

```text
built Proofs.Ai.EqReasoning
built Proofs.Ai.Analysis.AbstractMetricTopology
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.Vector.AbstractSpace
built Proofs.Ai.Analysis.AbstractNormedSpace
built Proofs.Ai.Analysis.AbstractLinearMap
built Proofs.Ai.Analysis.AbstractDerivative
built Proofs.Ai.Analysis.AbstractImplicitPhi
built Proofs.Ai.Analysis.AbstractImplicitFunction
wrote /private/tmp/npa-implicit-function-check.O6SDgA/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractImplicitFunction (11 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public implicit-function closure
through vendored certificate bytes and exercise these theorem names:

- `implicit_phi_derivative_from_args`
- `implicit_function_theorem`
- `implicit_function_derivative_theorem`

The smoke module should remain source-free for upstream `npa-mathlib`
dependencies and should build only its own downstream certificate from source.

## Positive Gate Plan

Main package gates:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
```

Downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
```

## Negative Check Plan

Use temporary package copies outside both repositories. Do not corrupt the live
working tree.

Required negative checks:

- bad public export hash for `Mathlib.Analysis.Calculus.ImplicitFunction` is
  rejected as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.Calculus.ImplicitFunction`
  is rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for
  `Mathlib.Analysis.Calculus.ImplicitFunction` are rejected by the source-free
  hash gate as `certificate_file_hash_mismatch`;
- stale downstream `Mathlib.Analysis.Calculus.ImplicitFunction` import
  identity, at minimum the imported `export_hash` or `certificate_hash`, is
  rejected by the downstream build or lock gate.

## Materialization Result

Result: materialized in `npa-mathlib v0.1.27`.

Public modules added:

- `Mathlib.Analysis.Calculus.ImplicitFunction.Phi`
- `Mathlib.Analysis.Calculus.ImplicitFunction`

Public package hashes after namespace rewrite:

| Public module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Analysis.Calculus.ImplicitFunction.Phi` | `sha256:746667e3cdabc718d17756ef8a961fb84c393cb3a2677374121687ff28e76e5f` | `sha256:e7951516c245ba3359d036917e34966627b1540bc9db3583501cb37c84f2a5c6` | `sha256:60eb204552d70bd36579339db92c140cfcd00fb680854f21416955cc49f16e9b` | `sha256:9f292648aa00dddbe49e31f78bc11448b5ee88db69aba9ec37edaa67a112d59b` | `sha256:97c25c688a9eca54d4e111a21569cee25b1ede25c14893f973bf52d4de030672` |
| `Mathlib.Analysis.Calculus.ImplicitFunction` | `sha256:4ad9b6a6a100e2b7964d11e3aec36c1518a3312cf8ce30cebbcf89140cf26ab0` | `sha256:e9c8e022da767c64a68a7d325d7973c716017978c833886cc4e568fe3afd578a` | `sha256:efc45f11663333dc28aa373c19bc943dd353aad33025b241eafe73dde91475fc` | `sha256:0d1690c6eba73dbd259c7cfc6f42702c48cb349a31bca7078cd2a76bb5ac6fcf` | `sha256:0b285ded8c84adf179a8d5e27831eadd240f81b1b0711e964f77438bc2b17784` |

Downstream smoke result:

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.ImplicitFunction` | `sha256:2b020728db71975f1d7e1cdc9b363828650e3afb9dd2effe2ab23250a091bb4c` | `sha256:5e6988e435940498171256f1e849bc337a320718f89ea7b5d8052fd30705f9aa` | `sha256:bbb4e2a24220aa8911e867c16996ffd622d5a1d688f7548185c1d32d88c4af71` | `sha256:264e20271c467decb51c6d88450323ab332a5299d533567d458be2be3b5f452c` | `sha256:7e3c2d13b12d4c92322dbba58f85a3afe3fa407af44bbf793456609e3971c59e` |

Generated package artifact hashes:

- `generated/package-lock.json`: `sha256:7b237107744142d0a5e80efea10648bcf9436c68cd3023b42332cdf94bb429db`
- `generated/axiom-report.json`: `sha256:535d04b9a022d26664fa6ced455e50e01121d313a48bbd2afa8ad99c73b382ee`
- `generated/theorem-index.json`: `sha256:b5e42502d7a58c008f3acc5e3b37e78905523a39bc5d1f30ed9dd9cb6858dc57`
- `generated/publish-plan.json`: `sha256:1f960656ccac46824ca49a1b742ca6b2d83f750ecd6da80e1b08ff8c93a029fa`
- publish plan self hash: `sha256:7e87685c4d59a3a81b4056bb2193ccbb7a59278f2336152c790811b622567c38`
- downstream `generated/package-lock.json`: `sha256:3844e345c25c68d1df6f74b79db71d99873cc1c54f502d2508bae4c7c4723beb`

Positive gates passed:

- `cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json`
- `cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json` (`modules=63`)
- `cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json`
- `cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json`
- `cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json` (`modules=13`)
- `cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json`

Negative checks passed on temporary package copies:

- bad public export hash for `Mathlib.Analysis.Calculus.ImplicitFunction`:
  `export_hash_mismatch`
- bad public certificate hash for `Mathlib.Analysis.Calculus.ImplicitFunction`:
  `certificate_hash_mismatch`
- corrupted public certificate bytes for
  `Mathlib.Analysis.Calculus.ImplicitFunction`:
  `certificate_file_hash_mismatch`
- bad downstream import export hash for
  `Mathlib.Analysis.Calculus.ImplicitFunction`: `export_hash_mismatch`
- stale downstream package lock: `package_lock_stale`

Release artifact:

- `target/release-artifacts/npa-mathlib-v0.1.27-release-artifacts.tar.gz`
- SHA-256:
  `b5b98f3c432f6001feac343cb39e00ed5191f4d810c13927826e345ca447583d`
