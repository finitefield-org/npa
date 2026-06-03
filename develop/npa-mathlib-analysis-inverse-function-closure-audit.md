# npa-mathlib Analysis Inverse Function Closure Audit

Date: 2026-06-04

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.25`
analysis fixed-point closure. It is a sidecar planning and release record, not
proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, publish plans, release artifacts, and this
audit are untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.25`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-fixed-point-closure-audit.md`.
- `Mathlib.Topology.Metric.Basic`,
  `Mathlib.LinearAlgebra.VectorSpace`,
  `Mathlib.Analysis.NormedSpace.Basic`,
  `Mathlib.Analysis.LinearMap`,
  `Mathlib.Analysis.Calculus.Derivative`, and
  `Mathlib.Analysis.FixedPoint.Banach` are already public and are available as
  source-free dependencies for this closure.

This closure should materialize as `npa-mathlib v0.1.26`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | `Mathlib.Analysis.Calculus.InverseFunction` | `Mathlib/Analysis/Calculus/InverseFunction/` | 5 definitions, 16 theorems | `Std.Logic.Eq`, metric topology, vector space, normed space, linear map, derivative, fixed point | none |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | `Mathlib.Analysis.Calculus.InverseFunction` |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | `Mathlib.Analysis.FixedPoint.Banach` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.Calculus.InverseFunction` because
the module introduces inverse residuals, Newton maps, local inverse evidence,
local inverse results, and a quantitative inverse-function theorem route. It
belongs under calculus after derivative and fixed point, while remaining a
single inverse-function layer for later implicit-function routes to consume.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractInverseFunction` as a
single-module closure:

- it imports only foundations already available in public `npa-mathlib`;
- it combines the fixed-point, derivative, and linear-isomorphism dependencies
  into one coherent local inverse-function vocabulary;
- it is the last prerequisite before the implicit-function route can be
  audited as its own closure.

The declaration-name collision scan against existing public `Mathlib/` source
found no matching `def` or `theorem` names for this closure.

## Explicitly Deferred Modules

| Deferred route | Reason |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | Auxiliary implicit-function Phi route; it should be audited after inverse function is public. |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | Depends on the implicit Phi route and derivative/inverse-function vocabulary; it should ship with or after the Phi route. |

## Public Declaration Inventory

Definitions:

- `InverseResidual`
- `InverseNewtonMap`
- `LocalInverseEvidence`
- `LocalInverseResult`
- `QuantitativeInverseFunctionArgs`

Theorems:

- `inverse_residual_def`
- `inverse_newton_map_def`
- `local_inverse_evidence_intro`
- `local_inverse_evidence_elim`
- `local_inverse_base_mem_from_evidence`
- `local_inverse_image_mem_from_evidence`
- `local_inverse_maps_from_evidence`
- `local_inverse_left_from_evidence`
- `local_inverse_right_from_evidence`
- `local_inverse_unique_from_evidence`
- `local_inverse_fixed_point_from_evidence`
- `local_inverse_derivative_from_evidence`
- `local_inverse_linear_iso_from_evidence`
- `local_inverse_result_intro`
- `local_inverse_result_elim`
- `quantitative_inverse_function_from_args`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` | rewrite to public metric-topology foundation |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | rewrite to public normed-space foundation |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` | rewrite to public bounded-linear-map foundation |
| `Proofs.Ai.Analysis.AbstractDerivative` | `Mathlib.Analysis.Calculus.Derivative` | rewrite to public derivative foundation |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | `Mathlib.Analysis.FixedPoint.Banach` | rewrite to public fixed-point foundation |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractInverseFunction` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | none | none | ok |

The manifest records an empty axiom set. The package still permits `Eq.rec`
because earlier imported public modules use it, but this inverse-function
module does not add direct or transitive axioms according to the corpus axiom
report.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | `sha256:53a1070c25bf2593e9e5e2f9c125dd6cb08d436a372014204748a5da2552598e` | `sha256:c7bcc569c243a359ef276d2fd0efdf0227b00197ccea399f3331911c8ebd6f88` | `sha256:bc9e208d297e347dfb8bd56737e274cdb6e3810f1c704419d946c43d90ea6b37` | `sha256:4ce54129453ca5f2cc9726d9e809e739b6640402d7515115c64b8169ae6c93a6` | `sha256:bd86cb8b7c52bb5517fe4872d34b6e1e2e18983cc5863525a11940cd44ae1322` |

## Corpus Verification

The selected corpus closure passed source-free module verification in a clean
temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractInverseFunction
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractInverseFunction
verified 1 selected module(s), 12 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild in the same
clean temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractInverseFunction
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
built Proofs.Ai.Analysis.AbstractFixedPoint
built Proofs.Ai.Analysis.AbstractInverseFunction
wrote /private/tmp/npa-inverse-function-check.9Yi1OL/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractInverseFunction (11 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public inverse-function closure
through vendored certificate bytes and exercise these theorem names:

- `local_inverse_result_intro`
- `quantitative_inverse_function_from_args`

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

- bad public export hash for `Mathlib.Analysis.Calculus.InverseFunction` is
  rejected as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.Calculus.InverseFunction`
  is rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for
  `Mathlib.Analysis.Calculus.InverseFunction` are rejected by source-free
  reference verification;
- stale downstream `Mathlib.Analysis.Calculus.InverseFunction` import identity,
  at minimum the imported `export_hash` or `certificate_hash`, is rejected by
  the downstream build or lock gate.

## Materialization Result

Materialized package version: `npa-mathlib v0.1.26`.

Public module:

```text
Mathlib.Analysis.Calculus.InverseFunction
Mathlib/Analysis/Calculus/InverseFunction/
```

Public hashes after namespace/import rewrite:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:cda16cfa847a4ff6fa447bb6c1749c100fa03db1f9c19caef893ccc77daee013` | `sha256:cc39011125fa514e18739a83f30d5a36376a972b873810f3b8ea81bd50d65976` | `sha256:abb15f4f06e9487c57969f4492e342be57401d9976a19c76ba95ebe89d4ee46f` | `sha256:4ce54129453ca5f2cc9726d9e809e739b6640402d7515115c64b8169ae6c93a6` | `sha256:b37f975a3a1c64f716754ab66d20383899321d81fd4deea8cc0720df4ff38a68` |

Downstream smoke module:

```text
Downstream.InverseFunction
fixtures/downstream-smoke/Downstream/InverseFunction/
```

Downstream smoke hashes:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:78f4186cb51f0b8cc25790261f9be0ec2b377fad3f80501bc0a2f1703d8f8203` | `sha256:c37d79060b79102633b349f53c772d45ee5bd14fa2ec7f2a699b7c0483ce83b1` | `sha256:461ccb4950002ee537291b5f974eb2436cfe23dcfe08f113423bab2ac1384302` | `sha256:1b1a79456ee1ba2bc13de73e311a14ccd977b26904fabc748558b5350010ab1b` | `sha256:b2ea6036c4fc34771aa7732b92b1369e242352e97e66ab3f785edac74bc964ad` |

Downstream smoke theorem aliases:

- `local_inverse_result_intro_passthrough`
- `quantitative_inverse_function_from_args_passthrough`

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
| Bad public inverse-function `expected_export_hash` | main `package check-hashes` | `export_hash_mismatch` |
| Bad public inverse-function `expected_certificate_hash` | main `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public inverse-function certificate bytes | main `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Bad downstream inverse-function import `export_hash` | downstream `package check-hashes` | `export_hash_mismatch` |
| Stale downstream generated package lock for inverse-function import | downstream `package check-hashes` | `package_lock_stale` |

Release sidecars:

- Release artifact:
  `target/release-artifacts/npa-mathlib-v0.1.26-release-artifacts.tar.gz`
- Release artifact SHA-256:
  `66e8cdcdd2fe59c4bc44c580dc2ee259076bf1e79a8cdc9979cad7a17e0a37b9`
- `generated/publish-plan.json` file SHA-256:
  `d566d19ce402e9cd60de5817f045cb26e062874b292e004e3491f7193ebdf853`
- `publish_plan_hash`:
  `sha256:5a70a09e5e95b8702ec09b3e0086f8f60651d7271e9e986d107b3b6804209215`

Final verdict: `Mathlib.Analysis.Calculus.InverseFunction` is closed over the
already public metric-topology, vector-space, normed-space, linear-map,
derivative, and fixed-point foundations, adds no custom axioms, passes
source-free reference verification, and is materialized as
`npa-mathlib v0.1.26`.
