# npa-mathlib Analysis Fixed Point Closure Audit

Date: 2026-06-04

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.24`
analysis derivative closure. It is a sidecar planning and release record, not
proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, publish plans, release artifacts, and this
audit are untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.24`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-derivative-closure-audit.md`.
- `Mathlib.Topology.Metric.Basic`,
  `Mathlib.LinearAlgebra.VectorSpace`, and
  `Mathlib.Analysis.NormedSpace.Basic` are already public and are available as
  source-free dependencies for this closure.

This closure should materialize as `npa-mathlib v0.1.25`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | `Mathlib.Analysis.FixedPoint.Banach` | `Mathlib/Analysis/FixedPoint/Banach/` | 10 definitions, 18 theorems | `Std.Logic.Eq`, metric topology, vector space, normed space | none |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | `Mathlib.Analysis.FixedPoint.Banach` |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.FixedPoint.Banach` because the
module introduces completeness, contractive self-maps, fixed-point evidence,
fixed-point results, and the Banach fixed-point theorem route. It is placed
under `Analysis.FixedPoint` rather than `Analysis.Calculus` because it is an
analysis theorem family used by inverse-function routes but not itself a
calculus derivative surface.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractFixedPoint` as a single-module
closure:

- it imports only public foundations already available in `npa-mathlib`
  (`Std.Logic.Eq`, `Mathlib.Topology.Metric.Basic`,
  `Mathlib.LinearAlgebra.VectorSpace`, and
  `Mathlib.Analysis.NormedSpace.Basic`);
- it introduces one coherent fixed-point and Banach fixed-point vocabulary;
- it is a required foundation for the later inverse-function closure.

The declaration-name collision scan against existing public `Mathlib/` source
found no matching `def` or `theorem` names for this closure.

## Explicitly Deferred Modules

| Deferred route | Reason |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | Depends on fixed point, derivative, linear map, normed space, vector space, and metric topology; it should wait until fixed point is public. |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | Depends on derivative, linear map, normed space, and vector space, but is an auxiliary implicit-function route independent of the Banach fixed-point result. |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | Depends on `AbstractImplicitPhi` and derivative; it should ship with or after the implicit Phi route. |

## Public Declaration Inventory

Definitions:

- `CauchySeq`
- `ConvergesTo`
- `CompleteMetricArgs`
- `SelfMapOn`
- `ContractiveOn`
- `FixedPoint`
- `FixedPointStability`
- `FixedPointEvidence`
- `FixedPointResult`
- `BanachFixedPointArgs`

Theorems:

- `cauchy_seq_intro`
- `cauchy_seq_apply`
- `converges_to_intro`
- `converges_to_apply`
- `complete_metric_limit_from_args`
- `self_map_on_apply`
- `contractive_on_apply`
- `fixed_point_def`
- `fixed_point_stability_apply`
- `fixed_point_evidence_intro`
- `fixed_point_evidence_elim`
- `fixed_point_mem_from_evidence`
- `fixed_point_eq_from_evidence`
- `fixed_point_unique_from_evidence`
- `fixed_point_stability_from_evidence`
- `fixed_point_result_intro`
- `fixed_point_result_elim`
- `banach_fixed_point_from_args`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` | rewrite to public metric-topology foundation |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | rewrite to public normed-space foundation |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractFixedPoint` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | none | none | ok |

The manifest records an empty axiom set. The package still permits `Eq.rec`
because earlier imported public modules use it, but this fixed-point module
does not add direct or transitive axioms according to the corpus axiom report.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | `sha256:58c71947705da5f7afd0cdfdd74b5f30b4feba9d0895b039ede4bf6ccb88e1e9` | `sha256:a3408d90281935c8d282957ba9eab1ee181297b751bafbff4683ee6f8191fd39` | `sha256:109e22398f17c8d23e5fff44199f6460944e9ae9eabadb4b23c4961ee169b185` | `sha256:aa19bce6d8162a8b9cbf3d4c5c9b7076a45a326d4ab073bcbb2177328a00ae12` | `sha256:bebfe83d397757e2ce3e6764f72286449f329c304ff1176c382e4248b8ceacfd` |

## Corpus Verification

The selected corpus closure passed source-free module verification in a clean
temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractFixedPoint
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractFixedPoint
verified 1 selected module(s), 9 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild in the same
clean temporary worktree:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractFixedPoint
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
built Proofs.Ai.Analysis.AbstractFixedPoint
wrote /private/tmp/npa-fixed-point-check.i09VIX/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractFixedPoint (8 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public fixed-point closure
through vendored certificate bytes and exercise these theorem names:

- `fixed_point_unique_from_evidence`
- `banach_fixed_point_from_args`

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

- bad public export hash for `Mathlib.Analysis.FixedPoint.Banach` is rejected
  as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.FixedPoint.Banach` is
  rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.Analysis.FixedPoint.Banach`
  are rejected by source-free reference verification;
- stale downstream `Mathlib.Analysis.FixedPoint.Banach` import identity, at
  minimum the imported `export_hash` or `certificate_hash`, is rejected by the
  downstream build or lock gate.

## Materialization Result

Materialized package version: `npa-mathlib v0.1.25`.

Public module:

```text
Mathlib.Analysis.FixedPoint.Banach
Mathlib/Analysis/FixedPoint/Banach/
```

Public hashes after namespace/import rewrite:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:9574f5719a73de411aa998e79846e62d60dbc08ece113d86e3a8279e2ce53da9` | `sha256:f03b2b830652ee7941e6ce09120b47959e2114fc3ed35dd9dc793ae8327d06ff` | `sha256:19fc9c8bc36358b02ff1bfad9f3695b4ab335e51474d67f7a55076d7ece074e2` | `sha256:aa19bce6d8162a8b9cbf3d4c5c9b7076a45a326d4ab073bcbb2177328a00ae12` | `sha256:0f03af71effebe775cb89cf02841eb7053ec72b7faf722639d08188a5ac0ba61` |

Downstream smoke module:

```text
Downstream.FixedPoint
fixtures/downstream-smoke/Downstream/FixedPoint/
```

Downstream smoke hashes:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:43df820b1ba6f06d3c9d31434d9940bd1cebef5bfc20ca6185ffd7a2bb800eac` | `sha256:028c15eac75bb9924945b2c5dcd86343fae590bdce9573cbe90b036cc18a4737` | `sha256:d9aa121ae91490c01ffa2278af6041dd36cb7b2977c874898d5400d57ade9ed9` | `sha256:1b1a79456ee1ba2bc13de73e311a14ccd977b26904fabc748558b5350010ab1b` | `sha256:f24e08b7541511f40a820c1f0f0495adf303d02b8858301f97804dbe81502219` |

Downstream smoke theorem aliases:

- `fixed_point_unique_from_evidence_passthrough`
- `banach_fixed_point_from_args_passthrough`

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
| Bad public fixed-point `expected_export_hash` | main `package check-hashes` | `export_hash_mismatch` |
| Bad public fixed-point `expected_certificate_hash` | main `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public fixed-point certificate bytes | main `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Bad downstream fixed-point import `export_hash` | downstream `package check-hashes` | `export_hash_mismatch` |
| Stale downstream generated package lock for fixed-point import | downstream `package check-hashes` | `package_lock_stale` |

Release sidecars:

- Release artifact:
  `target/release-artifacts/npa-mathlib-v0.1.25-release-artifacts.tar.gz`
- Release artifact SHA-256:
  `4126e1470f88a4a7c8b3dc66ea4777cf464c65ab8561ed8ff540ad265747c519`
- `generated/publish-plan.json` file SHA-256:
  `705607a3f0649b236ffd144893bcf01386ad03efaa92ab094b3290d7d6e6c62d`
- `publish_plan_hash`:
  `sha256:61f38b67b57475f603b74ccad9363deac55d29337cdef1de4d93736539539a0e`

Final verdict: `Mathlib.Analysis.FixedPoint.Banach` is closed over the
already public metric-topology, vector-space, and normed-space foundations,
adds no custom axioms, passes source-free reference verification, and is
materialized as `npa-mathlib v0.1.25`.
