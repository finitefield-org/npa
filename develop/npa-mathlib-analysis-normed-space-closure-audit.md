# npa-mathlib Analysis Normed Space Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.21`
analysis metric topology closure. It is a sidecar planning and release record,
not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes, deterministic
hashes, and source-free checker verdicts. Source files, replay files, metadata,
theorem indexes, publish plans, release artifacts, and this audit are
untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.21`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-metric-topology-closure-audit.md`.
- `Mathlib.LinearAlgebra.VectorSpace` is already public and is available as a
  source-free dependency for this closure.

This closure should materialize as `npa-mathlib v0.1.22`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | `Mathlib/Analysis/NormedSpace/Basic/` | 10 definitions, 28 theorems | `Std.Logic.Eq`, equality reasoning, vector space | transitive `Eq.rec` |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.NormedSpace.Basic` because the module
introduces the normed-space and product-norm law-package API used by later
analysis routes. It is not placed under `Mathlib.Topology.Metric.Basic`; the
metric-topology closure is predicate-local topology vocabulary, while this
closure provides norm and product-distance structure over vector-space
interfaces.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractNormedSpace` as a single-module
closure:

- it imports only public foundations already available in `npa-mathlib`
  (`Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`, and
  `Mathlib.LinearAlgebra.VectorSpace`);
- it introduces one coherent norm/product-norm vocabulary layer;
- it is the required foundation for later linear-map, derivative, fixed-point,
  inverse-function, and implicit-function closures.

## Explicitly Deferred Modules

| Deferred route | Reason |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractLinearMap` | Depends on this normed-space layer and adds bounded-linear-map/isomorphism APIs. |
| `Proofs.Ai.Analysis.AbstractDerivative` | Depends on metric topology, vector space, normed space, and linear maps. |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | Depends on metric topology and normed-space foundations, but is a theorem route rather than foundation vocabulary. |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | Depends on derivative, fixed point, linear map, normed space, vector space, and metric topology. |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | Depends on normed space, linear maps, and derivative; it is an auxiliary implicit-function route. |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | Depends on `AbstractImplicitPhi`, derivative, linear map, normed space, and vector space. |

## Public Declaration Inventory

Definitions:

- `NormDist`
- `NormedSpaceLawArgs`
- `ProductZero`
- `ProductAdd`
- `ProductNeg`
- `ProductSmul`
- `ProductSub`
- `ProductNorm`
- `ProductDist`
- `ProductNormEstimateArgs`

Theorems:

- `norm_dist_def`
- `norm_nonneg_from_args`
- `norm_zero_from_args`
- `norm_triangle_from_args`
- `norm_neg_from_args`
- `norm_dist_self_from_args`
- `norm_dist_symm_from_args`
- `norm_dist_triangle_from_args`
- `product_zero_def`
- `product_add_def`
- `product_neg_def`
- `product_smul_def`
- `product_sub_def`
- `product_norm_def`
- `product_dist_def`
- `product_fst_pair_from_args`
- `product_snd_pair_from_args`
- `product_pair_eta_from_args`
- `product_add_fst_from_pair_law`
- `product_add_snd_from_pair_law`
- `product_smul_fst_from_pair_law`
- `product_smul_snd_from_pair_law`
- `product_norm_pair_eq_from_pair_laws`
- `product_norm_fst_le_from_args`
- `product_norm_snd_le_from_args`
- `product_norm_pair_le_add_from_args`
- `product_norm_add_le_from_args`
- `product_dist_pair_le_add_from_args`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | rewrite to public mathlib equality reasoning |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractNormedSpace` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | none | `Eq.rec` | ok, already allowed |

The direct axiom set is empty. The manifest records `Eq.rec` because the
certificate closure imports equality reasoning and uses equality transport
transitively.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `sha256:e19b892341ea2d8c318a19e16fa4347c328b3b9aa111ab3a75f4ef72c018f359` | `sha256:b22cab5fed2e7d7cbead5b276cbad83e2dd7cfcf243cda99eaa0c6b6aa4545a0` | `sha256:9ba4bceb63822a367a670c83b85d18478c16eff046a8d3ef1801451e259294d8` | `sha256:4ffc0ac83684ca65d83296a7fb4b8104c1099ad79204df7e4efd1f430359d3c0` | `sha256:bffd653157060243f880baadf316dcc6fa80ff588d90fb13945621339f2fcc99` |

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractNormedSpace
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractNormedSpace
verified 1 selected module(s), 7 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild in a clean
temporary worktree to avoid modifying unrelated dirty files in the main `npa`
checkout:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractNormedSpace
```

Observed result:

```text
built Proofs.Ai.EqReasoning
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.Vector.AbstractSpace
built Proofs.Ai.Analysis.AbstractNormedSpace
wrote /private/tmp/npa-normed-space-check.sLAZxP/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractNormedSpace (6 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public normed-space closure
through vendored certificate bytes and exercise these theorem names:

- `norm_dist_triangle_from_args`
- `product_norm_pair_le_add_from_args`

The smoke module should remain source-free for upstream `npa-mathlib`
dependencies and should build only its own downstream certificate from source.

## Positive Gate Plan

Main package gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
```

Downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

## Negative Check Plan

Use temporary package copies outside both repositories. Do not corrupt the live
working tree.

Required negative checks:

- bad public export hash for `Mathlib.Analysis.NormedSpace.Basic` is rejected
  as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.NormedSpace.Basic` is
  rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.Analysis.NormedSpace.Basic`
  are rejected by source-free reference verification;
- stale downstream `Mathlib.Analysis.NormedSpace.Basic` import identity, at
  minimum the imported `export_hash` or `certificate_hash`, is rejected by the
  downstream build or lock gate.

## Materialization Result

Result: materialized in `npa-mathlib` v0.1.22.

Public module:

- `Mathlib.Analysis.NormedSpace.Basic`

Namespace mapping:

- `Proofs.Ai.Analysis.AbstractNormedSpace` ->
  `Mathlib.Analysis.NormedSpace.Basic`

Public module hashes:

| Field | Hash |
| --- | --- |
| source file | `sha256:e9ebe9423b89282c7b67dca89b928df621b6224b7ac18264f1c0c82d03d8604e` |
| certificate file | `sha256:c593e76d0bef4a190d20d7a0940befc66dde9e8348517b3293edc80dcfe07f76` |
| export | `sha256:d1ce796f7af0aa26d4b1236c440276f4a7a8b30421af244bf8839aca49f9e56b` |
| axiom report | `sha256:4ffc0ac83684ca65d83296a7fb4b8104c1099ad79204df7e4efd1f430359d3c0` |
| canonical certificate | `sha256:236207f99ee6b9e58509b92f8f4d64493aabc93dbfd4979096d674e37810c24d` |

Downstream smoke module:

- `Downstream.NormedSpace`
- imported public theorems:
  `norm_dist_triangle_from_args`, `product_norm_pair_le_add_from_args`

Downstream smoke hashes:

| Field | Hash |
| --- | --- |
| downstream source file | `sha256:0cb1a1cdca643d3bcd763bf4353cdd99f47765839663cca605eec31f5f92ba99` |
| downstream manifest file | `sha256:068be2b7e6554e649c5def33f4c04399d50418b7a04c60a7bde9ec8c56518b4a` |
| downstream package lock file | `sha256:56b9351a0cccb462fde8bd9b51b39ac0a0be6341d4f5119790fdfdba23a89f41` |
| downstream certificate file | `sha256:75b0f17bbd1ab62c25047012fc35fe7c2e03c544ac5b23cc5ad18aa0c62fc5d3` |
| downstream export | `sha256:d99fd0e9a0d762bd69a9ad299f1f5505e9912df126b79d94a7337948ee81dbad` |
| downstream axiom report | `sha256:1b1a79456ee1ba2bc13de73e311a14ccd977b26904fabc748558b5350010ab1b` |
| downstream canonical certificate | `sha256:e299a0c747a08b24cfec9ba16b82f041e16c5fde1721b564e36ce037d73870d2` |

Generated package artifacts:

| Artifact | Hash |
| --- | --- |
| `generated/package-lock.json` | `sha256:2aee674ac655e3847b6ddaa8667f2df69b814cdb84b81fcd2f2bc026dab68350` |
| `generated/axiom-report.json` | `sha256:ffc3b897beef40cc30b1bc1d6b1cebca8c206243f6baca6ee4ebd4ef97931493` |
| `generated/theorem-index.json` | `sha256:3f3d20ab219e0222644a41b2b7719aba3a09454cc6057c98f569691fc202213d` |
| `generated/publish-plan.json` | `sha256:c184ed1ef11e4198396a398afcdca0fe449ad9032784733a9d017e16401af754` |
| publish plan self hash | `sha256:e30b43ee33424740b8f338b256f8e2a76d5d7e476c518707300ea0c18fc55c9f` |

Release artifact:

- `target/release-artifacts/npa-mathlib-v0.1.22-release-artifacts.tar.gz`
- sha256:
  `76a4dd20d9b655d87480bb89a39dab2306d39f9ad96a7d5bfe5d4ac6056e9cc1`

Positive gates passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

The main package reference verification passed with `modules=57`. The
downstream smoke reference verification passed with `modules=8`.

Negative gates on temporary package copies:

- Public `expected_export_hash` tamper rejected by
  `package check-hashes` as `export_hash_mismatch`.
- Public `expected_certificate_hash` tamper rejected by
  `package check-hashes` as `certificate_hash_mismatch`.
- Public certificate byte corruption rejected by
  `package verify-certs --checker reference` as `certificate_decode_failed`
  after aligning the temporary certificate file hash.
- Downstream `Mathlib.Analysis.NormedSpace.Basic` import `export_hash` tamper
  rejected by `package build-certs --check` as `export_hash_mismatch`.
