# npa-mathlib Analysis Linear Map Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.22`
analysis normed-space closure. It is a sidecar planning and release record,
not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes, deterministic
hashes, and source-free checker verdicts. Source files, replay files, metadata,
theorem indexes, publish plans, release artifacts, and this audit are
untrusted sidecars.

## Baseline

- Current public package before this closure:
  `npa-mathlib v0.1.22`.
- Latest completed audit before this closure:
  `develop/npa-mathlib-analysis-normed-space-closure-audit.md`.
- `Mathlib.LinearAlgebra.VectorSpace` and
  `Mathlib.Analysis.NormedSpace.Basic` are already public and are available as
  source-free dependencies for this closure.

This closure should materialize as `npa-mathlib v0.1.23`. It must not change
the trusted base or the allowed axiom policy.

## Selected Closure

| Corpus module | Public module | Public path | Surface | Imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` | `Mathlib/Analysis/LinearMap/` | 10 definitions, 30 theorems | `Std.Logic.Eq`, equality reasoning, vector space, normed space | transitive `Eq.rec` |

## Public Namespace Decision

| Corpus namespace | Public namespace |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `Mathlib.Analysis.LinearMap` |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` |
| `Std.Logic.Eq` | `Std.Logic.Eq` |

The public namespace is `Mathlib.Analysis.LinearMap` because the module
introduces bounded-linear-map, linear-isomorphism, composition, inverse, and
block-triangular map APIs over the normed-space foundation. It is not nested
under `NormedSpace.Basic`; it is the next analysis API layer built on top of
that foundation.

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractLinearMap` as a single-module
closure:

- it imports only public foundations already available in `npa-mathlib`
  (`Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`,
  `Mathlib.LinearAlgebra.VectorSpace`, and
  `Mathlib.Analysis.NormedSpace.Basic`);
- it introduces one coherent linear-map vocabulary and theorem layer;
- it is a required foundation for later derivative, inverse-function, and
  implicit-function closures.

## Explicitly Deferred Modules

| Deferred route | Reason |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractDerivative` | Depends on this linear-map layer and adds Frechet derivative APIs. |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | Independent theorem route over metric topology and normed space; does not need to ship with linear maps. |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | Depends on derivative, fixed point, linear map, normed space, vector space, and metric topology. |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | Depends on normed space, linear maps, and derivative; it is an auxiliary implicit-function route. |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | Depends on `AbstractImplicitPhi`, derivative, linear map, normed space, and vector space. |

## Public Declaration Inventory

Definitions:

- `OperatorNormBound`
- `LinearMapLawArgs`
- `BoundedLinearMapArgs`
- `LinearIsoArgs`
- `LinearId`
- `LinearComp`
- `LinearInv`
- `BlockTriangularMap`
- `BlockTriangularInverse`
- `BlockTriangularIsoArgs`

Theorems:

- `operator_norm_bound_apply`
- `linear_map_zero_from_args`
- `linear_map_add_from_args`
- `linear_map_neg_from_args`
- `linear_map_smul_from_args`
- `bounded_linear_map_linear_from_args`
- `bounded_linear_map_bound_from_args`
- `bounded_linear_map_bound_apply`
- `linear_iso_forward_linear_from_args`
- `linear_iso_inverse_linear_from_args`
- `linear_iso_forward_bound_from_args`
- `linear_iso_inverse_bound_from_args`
- `linear_iso_left_inverse_from_args`
- `linear_iso_right_inverse_from_args`
- `linear_id_def`
- `linear_id_zero`
- `linear_id_add`
- `linear_id_neg`
- `linear_id_smul`
- `linear_id_law_args`
- `linear_comp_def`
- `linear_comp_law_args`
- `linear_inv_def`
- `linear_inv_left_inverse_from_iso`
- `linear_inv_right_inverse_from_iso`
- `block_triangular_map_def`
- `block_triangular_inverse_def`
- `block_triangular_b_iso_from_args`
- `block_triangular_left_inverse_from_args`
- `block_triangular_right_inverse_from_args`

## Import Rewrite Table

| Corpus import | Public import | Action |
| --- | --- | --- |
| `Std.Logic.Eq` | `Std.Logic.Eq` | keep external std import |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | rewrite to public mathlib equality reasoning |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | rewrite to public vector-space foundation |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | `Mathlib.Analysis.NormedSpace.Basic` | rewrite to public normed-space foundation |

Public source, replay, metadata, package lock, publish plan, and downstream
fixture artifacts must not contain stale
`Proofs.Ai.Analysis.AbstractLinearMap` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the current
`Eq.rec` allowance.

Corpus axiom report:

| Module | Direct axioms | Transitive axioms | Policy |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractLinearMap` | none | `Eq.rec` | ok, already allowed |

The direct axiom set is empty. The manifest records `Eq.rec` because the
certificate closure imports equality reasoning and uses equality transport
transitively.

## Corpus Hashes

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractLinearMap` | `sha256:3006c5792e434e45923b4f4ab89f92ea1d6f0998b09845cecb19f70f668901ee` | `sha256:68bbfe49c9966015802bf23f111458bf4ee074d0d99448f2ba2bef4d84826ea2` | `sha256:5e825419cad422753439af18d927efdd27c8f9c23bb6bec2345bf45500d46b93` | `sha256:baf69fb307b5263ee1a8d7b94c8f3e8072680b801274a99eb85a955a3e1b50ff` | `sha256:ea1e46520fdb71a7a2d53eff2465a2b9b0caeea2d3656995a7a66d1e3c47a825` |

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractLinearMap
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractLinearMap
verified 1 selected module(s), 8 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild in a clean
temporary worktree to avoid modifying unrelated dirty files in the main `npa`
checkout:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractLinearMap
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
wrote /private/tmp/npa-linear-map-check.qCyum3/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractLinearMap (7 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public linear-map closure
through vendored certificate bytes and exercise these theorem names:

- `linear_comp_law_args`
- `linear_inv_left_inverse_from_iso`
- `block_triangular_b_iso_from_args`

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

- bad public export hash for `Mathlib.Analysis.LinearMap` is rejected as
  `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Analysis.LinearMap` is rejected as
  `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.Analysis.LinearMap` are
  rejected by source-free reference verification;
- stale downstream `Mathlib.Analysis.LinearMap` import identity, at minimum
  the imported `export_hash` or `certificate_hash`, is rejected by the
  downstream build or lock gate.

## Materialization Result

Status: succeeded as `npa-mathlib v0.1.23`.

Public module and path:

| Public module | Public path |
| --- | --- |
| `Mathlib.Analysis.LinearMap` | `Mathlib/Analysis/LinearMap/` |

Public hashes:

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Analysis.LinearMap` | `sha256:eb2ce4a6e76ea8693cd8e4fa850c7d2ae1928772c44477572b7c101d94a6ee8e` | `sha256:d36ee0db405af0a21af489183cd0ff78e26c3e8ea8c87a9bf4c2e15b4ea3333a` | `sha256:f4c6a05d5b50cb41ef031bfb44cf43fb0ef8ac873a629f96255a0549191cafd7` | `sha256:baf69fb307b5263ee1a8d7b94c8f3e8072680b801274a99eb85a955a3e1b50ff` | `sha256:6d3947e1c38337eb05b37e27b32152c648920e4b61920413b9619ffe23c71034` |

Downstream smoke:

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.LinearMap` | `sha256:2f7f91615a63970d568f4c81a7f642eda50d1df120bb60aca9d7dddbb3ddb07a` | `sha256:1f1150638dd039d1f913ded79726a2c8f48075a1a5b66767c396c07fc3f82e2d` | `sha256:d65f81edd855cc94536532636b613f15a39065d4ddf82ff95b6e9e5c24f326ee` | `sha256:91a142c1b2f4dc1420db2f193034d32485bac15e21ce3c5f2e6928a2b1a265d3` | `sha256:2d64ce1674110545abf61e90db2fbdcf87fd21d2302054e4902ccbc6f8e34e9c` |

The downstream smoke theorem aliases are:

- `linear_comp_law_args_passthrough`
- `linear_inv_left_inverse_passthrough`
- `block_triangular_b_iso_passthrough`

Positive gates all passed:

- `cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json`
- `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json`
- `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json`
- `cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json`
- `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json`
- `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json`

Negative package-copy checks:

| Check | Command | Observed reason code |
| --- | --- | --- |
| Bad public export hash for `Mathlib.Analysis.LinearMap` | `package check-hashes` | `export_hash_mismatch` |
| Bad public certificate hash for `Mathlib.Analysis.LinearMap` | `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public certificate bytes for `Mathlib.Analysis.LinearMap` | `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Stale downstream package lock | `package check-hashes` | `package_lock_stale` |

Release artifact:

```text
../npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.23-release-artifacts.tar.gz
```

Release artifact SHA-256:

```text
3d1173c858ccfe3cdbe7f4a4ef9fd2069c0903fa9b2bd6d1e4a4263a3c41df3e
```

Publish-plan evidence:

- `generated/publish-plan.json` file hash:
  `sha256:7e0a09c7dfcddaab94d5cf1289765686774083ce11cb56dc577bd0a4d1f550d5`
- `publish_plan_hash`:
  `sha256:310c45fcf129bd7f52d2ad7b1da0f7e18398d8eac5e3fc4303f2d66b5f146515`
