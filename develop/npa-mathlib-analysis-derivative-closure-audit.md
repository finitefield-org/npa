# npa-mathlib Analysis Derivative Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.23`
analysis linear-map closure. It is a sidecar planning and release record, not
proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, publish plans, release artifacts, and this
audit are untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.23`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-linear-map-closure-audit.md`.
- `Mathlib.Topology.Metric.Basic`,
  `Mathlib.LinearAlgebra.VectorSpace`,
  `Mathlib.Analysis.NormedSpace.Basic`, and
  `Mathlib.Analysis.LinearMap` are already public and are available as
  source-free dependencies for this closure.

This closure should materialize as `npa-mathlib v0.1.24`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` | `Mathlib/Analysis/Calculus/Derivative/` | 19 definitions, 24 theorems | `Std.Logic.Eq`, equality reasoning, metric topology, vector space, normed space, linear map | none |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.Calculus.Derivative` because the
module introduces Frechet derivative, differentiability, derivative uniqueness,
product/pair rules, composition, and partial derivative rule APIs. It is
placed under `Analysis.Calculus` rather than directly under
`Mathlib.Analysis` because it starts the calculus subdomain that later
inverse-function and implicit-function routes will extend.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractDerivative` as a single-module
closure:

- it imports only public foundations already available in `npa-mathlib`
  (`Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
  `Mathlib.Topology.Metric.Basic`, `Mathlib.LinearAlgebra.VectorSpace`,
  `Mathlib.Analysis.NormedSpace.Basic`, and
  `Mathlib.Analysis.LinearMap`);
- it introduces one coherent derivative and derivative-rule vocabulary;
- it is a required foundation for later inverse-function and implicit-function
  closures.

The declaration-name collision scan against existing public `Mathlib/` source
found no matching `def` or `theorem` names for this closure.

## Explicitly Deferred Modules

| Deferred route | Reason |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | Independent Banach fixed-point route over metric topology and normed space; it does not need to ship with derivative. |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | Depends on derivative and fixed point, so it should wait until both are public. |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | Depends on derivative, linear map, normed space, and vector space, but is an auxiliary implicit-function route. |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | Depends on `AbstractImplicitPhi` and derivative; it should ship with or after the implicit Phi route. |

## Public Declaration Inventory

Definitions:

- `FrechetRemainder`
- `FrechetDerivativeAt`
- `FrechetDifferentiableAt`
- `FrechetDifferentiableOn`
- `DerivativeUniqueArgs`
- `ConstMap`
- `ZeroMap`
- `PairMap`
- `PartialXMap`
- `PartialYMap`
- `PartialXDerivativeMap`
- `PartialYDerivativeMap`
- `DerivativeConstRuleArgs`
- `DerivativeIdRuleArgs`
- `DerivativeFstRuleArgs`
- `DerivativeSndRuleArgs`
- `DerivativePairRuleArgs`
- `DerivativeCompRuleArgs`
- `PartialDerivativeRuleArgs`

Theorems:

- `frechet_remainder_def`
- `frechet_derivative_at_intro`
- `frechet_derivative_linear_from_at`
- `frechet_derivative_bound_from_at`
- `frechet_derivative_remainder_from_at`
- `frechet_differentiable_at_intro`
- `frechet_differentiable_at_elim`
- `frechet_differentiable_on_apply`
- `derivative_unique_from_args`
- `const_map_def`
- `zero_map_def`
- `pair_map_def`
- `derivative_const_from_args`
- `derivative_id_from_args`
- `derivative_fst_from_args`
- `derivative_snd_from_args`
- `derivative_pair_from_args`
- `derivative_comp_from_args`
- `partial_x_map_def`
- `partial_y_map_def`
- `partial_x_derivative_map_def`
- `partial_y_derivative_map_def`
- `partial_x_derivative_from_args`
- `partial_y_derivative_from_args`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | rewrite to public mathlib equality reasoning |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` | rewrite to public metric-topology foundation |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | rewrite to public normed-space foundation |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` | rewrite to public linear-map foundation |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractDerivative` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractDerivative` | none | none | ok |

The manifest records an empty axiom set. The package still permits `Eq.rec`
because earlier imported public modules use it, but this derivative module does
not add direct or transitive axioms according to the corpus axiom report.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractDerivative` | `sha256:5aec52426cda6d9af5183fe03eea8a9342eda521d2d5556f60ad96c8a0bee4ce` | `sha256:252f8006ae9d3968d7934d3d22d186bc38b8ac3556db88bb05d2d1d178884472` | `sha256:e2da8694bde95902399b6d05e3c2246e3b0f5b96cf97617f1bb0e19c4d5d1bc8` | `sha256:aacbe01532a99d075e6a5943c920c51381fff510fd7e56959ab34351a9b96090` | `sha256:fe97196f2d75d631d693285516377088aba597f2c2d686bdc9b45b2e878c90c6` |

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractDerivative
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractDerivative
verified 1 selected module(s), 10 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild in a clean
temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractDerivative
```

Observed result:

```text
built Proofs.Ai.EqReasoning
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.Vector.AbstractSpace
built Proofs.Ai.Analysis.AbstractNormedSpace
built Proofs.Ai.Analysis.AbstractLinearMap
built Proofs.Ai.Analysis.AbstractMetricTopology
built Proofs.Ai.Analysis.AbstractDerivative
wrote /private/tmp/npa-derivative-check.M52f17/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractDerivative (9 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public derivative closure
through vendored certificate bytes and exercise these theorem names:

- `frechet_derivative_at_intro`
- `derivative_comp_from_args`
- `partial_x_derivative_from_args`

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

- bad public export hash for `Mathlib.Analysis.Calculus.Derivative` is
  rejected as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.Calculus.Derivative` is
  rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for
  `Mathlib.Analysis.Calculus.Derivative` are rejected by source-free reference
  verification;
- stale downstream `Mathlib.Analysis.Calculus.Derivative` import identity, at
  minimum the imported `export_hash` or `certificate_hash`, is rejected by the
  downstream build or lock gate.

## Materialization Result

Materialized package version: `npa-mathlib v0.1.24`.

Public module:

```text
Mathlib.Analysis.Calculus.Derivative
Mathlib/Analysis/Calculus/Derivative/
```

Public hashes after namespace/import rewrite:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:ecdc172c135acc7da5f53016aa9cfabe4976c2b00061f61b654240f394cfa90d` | `sha256:1d2599bf57a57dc250cc9313064bb3ac2159bddb4913bb8c6e11f6183957f3e6` | `sha256:9c665ab4b07f95261f66ef9d34f3f1df62955b5925ed52a1e7ab8ff1dff00ed9` | `sha256:aacbe01532a99d075e6a5943c920c51381fff510fd7e56959ab34351a9b96090` | `sha256:36b6c9e1d5bc128cd500cd5cad18f146f32e61fc86f8868a9ef6d2c0e9688fe3` |

Downstream smoke module:

```text
Downstream.Derivative
fixtures/downstream-smoke/Downstream/Derivative/
```

Downstream smoke hashes:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:926f9db15e04ddbdb5523d02e70ca675f6f089c057bcf24723bd7c826ca17cdb` | `sha256:59f1cad11a2d737ffc66aac94b834910c657bd9c571606a75085340e0d5feb2c` | `sha256:64d83191e06311df3edc8722d8317d049f04047de2e2a68517bcf4d4b49f17e6` | `sha256:0544a7cbc8ae41c11f909366f72b2c80b48b4429ded309accea7370850df08bb` | `sha256:0f51db7ba7d7a2e7b08851fd40cdca4c0da4069be1e2106bbfad9c3f5abc8056` |

Downstream smoke theorem aliases:

- `frechet_derivative_at_intro_passthrough`
- `derivative_comp_from_args_passthrough`
- `partial_x_derivative_from_args_passthrough`

Positive gates passed:

- `package check` for main `npa-mathlib`
- `package build-certs --check` for main `npa-mathlib`
- `package verify-certs --checker reference` for main `npa-mathlib`
- `package check-hashes` for main `npa-mathlib`
- `package axiom-report --check` for main `npa-mathlib`
- `package index --check` for main `npa-mathlib`
- `package publish-plan --check` for main `npa-mathlib`
- `package check` for downstream smoke
- `package build-certs --check` for downstream smoke
- `package verify-certs --checker reference` for downstream smoke
- `package check-hashes` for downstream smoke

Negative package-copy checks passed:

| Mutated evidence | Command surface | Observed rejection |
| --- | --- | --- |
| Bad public derivative `expected_export_hash` | main `package check-hashes` | `export_hash_mismatch` |
| Bad public derivative `expected_certificate_hash` | main `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public derivative certificate bytes | main `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Bad downstream derivative import `export_hash` | downstream `package check-hashes` | `export_hash_mismatch` |
| Stale downstream generated package lock for derivative import | downstream `package check-hashes` | `package_lock_stale` |

Release sidecars:

- Release artifact:
  `target/release-artifacts/npa-mathlib-v0.1.24-release-artifacts.tar.gz`
- Release artifact SHA-256:
  `aaa37d8d42cec27b72d462e1d9163b8828b7304c338af80f62936c8da0113f09`
- `generated/publish-plan.json` file SHA-256:
  `982a428b164719594b0bff33d0b5b14b3df92a31ce9fb313e9e93b26c3d7313c`
- `publish_plan_hash`:
  `sha256:39dc436f919e61ee9affef3a93ec70cd084d02fe0dd0b65d06c2b978393ac6aa`

Final verdict: `Mathlib.Analysis.Calculus.Derivative` is closed over the
already public metric-topology, vector-space, normed-space, and linear-map
foundations, adds no custom axioms, passes source-free reference verification,
and is materialized as `npa-mathlib v0.1.24`.
