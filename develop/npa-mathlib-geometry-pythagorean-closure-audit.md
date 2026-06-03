# npa-mathlib Geometry Pythagorean Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.19`
inner-product closure. It selects the abstract affine, right-triangle, metric,
and Pythagorean law-package route as one public geometry closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The inner-product closure has been materialized, committed, tagged, and
  pushed in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
  `npa-mathlib v0.1.19`.
- The `npa-mathlib` release commit is
  `259f4038afb2cf593c5b70c897cda974b5c706f9`.
- The local ignored `v0.1.19` release-bundle tar hash is
  `sha256:5f8be1a0c810187afa8dd904b75a27f92fe92b175a8154c9e92a5a679b598e42`.

This closure should materialize as `npa-mathlib v0.1.20`. It must not change
package boundaries, registry assumptions, import identity rules, or proof
trust boundaries.

The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

## Selected Candidate Set

The selected candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Geometry.Affine` | `Mathlib.Geometry.Affine` | `Mathlib/Geometry/Affine/` | 7 definitions, 8 theorems | abstract ordered field, abstract ring, abstract square normalize, abstract vector space, abstract inner product, `Std.Logic.Eq` | none |
| `Proofs.Ai.Geometry.AffineDerive` | `Mathlib.Geometry.Affine.Derived` | `Mathlib/Geometry/Affine/Derived/` | 0 definitions, 8 theorems | affine, abstract ordered field, abstract ring, abstract square normalize, abstract vector space, abstract inner product, `Std.Logic.Eq` | `Eq.rec` |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | `Mathlib.Geometry.RightTriangle.Abstract` | `Mathlib/Geometry/RightTriangle/Abstract/` | 5 definitions, 7 theorems | affine, abstract ordered field, abstract ring, abstract square normalize, abstract vector space, abstract inner product, `Std.Logic.Eq` | none |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | `Mathlib.Geometry.RightTriangle.Derived` | `Mathlib/Geometry/RightTriangle/Derived/` | 0 definitions, 6 theorems | affine, affine derived, abstract right triangle, abstract ordered field, abstract ring, abstract square normalize, abstract vector space, abstract inner product, inner-product derived, `Std.Logic.Eq` | `Eq.rec` |
| `Proofs.Ai.Geometry.AbstractMetric` | `Mathlib.Geometry.Metric.Abstract` | `Mathlib/Geometry/Metric/Abstract/` | 3 definitions, 13 theorems | affine, affine derived, abstract right triangle, abstract ordered field, abstract ring, abstract square normalize, abstract vector space, abstract inner product, inner-product derived, `Std.Logic.Eq` | `Eq.rec` |
| `Proofs.Ai.Geometry.Pythagorean` | `Mathlib.Geometry.Pythagorean` | `Mathlib/Geometry/Pythagorean/` | 0 definitions, 14 theorems | affine, affine derived, abstract right triangle, right-triangle derived, abstract metric, abstract ordered field, abstract ring, abstract square normalize, scalar identities, equality reasoning, abstract vector space, abstract inner product, inner-product derived, `Std.Logic.Eq` | `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public geometry module |
| --- | --- |
| `Proofs.Ai.Geometry.Affine` | `Mathlib.Geometry.Affine` |
| `Proofs.Ai.Geometry.AffineDerive` | `Mathlib.Geometry.Affine.Derived` |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | `Mathlib.Geometry.RightTriangle.Abstract` |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | `Mathlib.Geometry.RightTriangle.Derived` |
| `Proofs.Ai.Geometry.AbstractMetric` | `Mathlib.Geometry.Metric.Abstract` |
| `Proofs.Ai.Geometry.Pythagorean` | `Mathlib.Geometry.Pythagorean` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Geometry.Affine
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct

Mathlib.Geometry.Affine.Derived
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.Geometry.Affine

Mathlib.Geometry.RightTriangle.Abstract
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.Geometry.Affine

Mathlib.Geometry.RightTriangle.Derived
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.LinearAlgebra.InnerProduct.Derived
  imports Mathlib.Geometry.Affine
  imports Mathlib.Geometry.Affine.Derived
  imports Mathlib.Geometry.RightTriangle.Abstract

Mathlib.Geometry.Metric.Abstract
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.LinearAlgebra.InnerProduct.Derived
  imports Mathlib.Geometry.Affine
  imports Mathlib.Geometry.Affine.Derived
  imports Mathlib.Geometry.RightTriangle.Abstract

Mathlib.Geometry.Pythagorean
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.Algebra.OrderedField.ScalarIdentities
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.LinearAlgebra.InnerProduct.Derived
  imports Mathlib.Geometry.Affine
  imports Mathlib.Geometry.Affine.Derived
  imports Mathlib.Geometry.RightTriangle.Abstract
  imports Mathlib.Geometry.RightTriangle.Derived
  imports Mathlib.Geometry.Metric.Abstract
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import through
the already public prerequisite modules and direct manifest imports. The other
prerequisites are already public `npa-mathlib v0.1.19` modules.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.Geometry.Affine` | `Point`, `disp`, `distSqPoints`, `translate`, `midpoint`, `collinear`, `AffineLawArgs` | `disp_self`, `disp_reverse`, `disp_comp`, `hypotenuse_vector_eq_sub_legs`, `dist_sq_points_def`, `point_ext_of_zero_disp`, `dist_sq_symm`, `dist_sq_zero_iff_eq` |
| `Mathlib.Geometry.Affine.Derived` | none | `vec_add_comm_from_vector_args`, `disp_reverse_from_affine_args`, `disp_comp_from_affine_args`, `dist_sq_points_def_from_args`, `hypotenuse_vector_eq_neg_left_add_right_from_args`, `hypotenuse_vector_eq_sub_legs_from_args`, `dist_sq_hypotenuse_norm_neg_left_add_right_from_args`, `dist_sq_hypotenuse_norm_sub_legs_from_args` |
| `Mathlib.Geometry.RightTriangle.Abstract` | `Perp`, `RightTriangle`, `AngleRight`, `Area2`, `FootOnHypotenuse` | `perp_iff_dot_eq_zero`, `perp_symm`, `right_triangle_legs_perp`, `pythagorean_distance_sq_general`, `law_of_cosines_general`, `right_triangle_area_general`, `median_to_hypotenuse_general` |
| `Mathlib.Geometry.RightTriangle.Derived` | none | `neg_zero_from_ring_args`, `right_triangle_legs_perp_vec_from_rt`, `right_triangle_legs_dot_zero_from_rt`, `right_triangle_neg_left_dot_zero_from_rt`, `right_triangle_neg_left_perp_vec_from_rt`, `right_triangle_affine_additive_perp_bridge_from_rt` |
| `Mathlib.Geometry.Metric.Abstract` | `dist`, `MetricSpaceLawArgs`, `Ball` | `dist_def`, `point_dist_sq_nonneg_from_inner_args`, `square_dist_eq_dist_sq_from_law_packages`, `dist_sq_eq_square_dist_from_law_packages`, `dist_sq_eq_square_dist`, `dist_sq_points_le_square_sum_dist_from_law_packages`, `dist_nonneg_from_ordered_args`, `triangle_inequality_from_law_packages`, `dist_nonneg`, `distance_symm`, `distance_zero_iff_eq`, `pythagorean_distance_general`, `triangle_inequality` |
| `Mathlib.Geometry.Pythagorean` | none | `pythagorean_dist_sq_symm_from_affine_args`, `pythagorean_dist_sq_reverse_norm_neg_from_law_packages`, `pythagorean_left_leg_norm_neg_from_law_packages`, `dist_sq_law_of_cosines_rhs_from_law_packages`, `law_of_cosines_sq_from_law_packages`, `law_of_cosines_dist_sq_from_law_packages`, `pythagorean_distance_sq_from_law_packages`, `pythagorean_theorem_sq`, `pythagorean_theorem_dist_sq`, `pythagorean_converse_sq`, `law_of_cosines_right_angle_specialization_from_law_packages`, `law_of_cosines_right_angle_specialization`, `pythagorean_theorem_api_alias`, `pythagorean_theorem_dependencies` |

## Closure Unit Rationale

This audit keeps the six geometry modules in one release because
`Proofs.Ai.Geometry.Pythagorean` imports the other five candidate modules and
the route's final public value is the Pythagorean/law-of-cosines theorem
surface. Splitting after `Affine` or after `AbstractRightTriangle` would ship a
geometry foundation without the final theorem surface that motivated this
closure.

The following nearby routes are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Analysis metric topology | `Proofs.Ai.Analysis.AbstractMetricTopology` | This is an analysis/topology route over metric facts, not part of the geometry Pythagorean API. |
| Analysis normed spaces | `Proofs.Ai.Analysis.AbstractNormedSpace` | Uses vector-space/equality foundations but has a distinct norm/product-space API. |
| Analysis linear maps and derivatives | `Proofs.Ai.Analysis.AbstractLinearMap`, `Proofs.Ai.Analysis.AbstractDerivative`, and downstream analysis modules | Separate analysis closures over vector-space and normed-space foundations. |
| Concrete geometry route | already public `Mathlib.Geometry.RightTriangle`, `Mathlib.Geometry.Metric` | These are concrete one-element/vector-route modules and remain separate from the abstract law-package geometry route. |

If materialization becomes too large for one release, the fallback split is:

| Split | Corpus modules |
| --- | --- |
| Geometry affine/right-triangle foundation | `Proofs.Ai.Geometry.Affine`, `Proofs.Ai.Geometry.AffineDerive`, `Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Geometry.AbstractRightTriangleDerive` |
| Geometry metric/Pythagorean final | `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.Pythagorean` |

## Namespace Decision

`Mathlib.Geometry.Affine` is used for the abstract point/displacement surface.
`Mathlib.Geometry.Affine.Derived` holds derived affine facts from the full
affine/vector law packages.

The existing `Mathlib.Geometry.RightTriangle` and `Mathlib.Geometry.Metric`
modules remain the concrete route released earlier. The abstract law-package
route therefore uses `Mathlib.Geometry.RightTriangle.Abstract`,
`Mathlib.Geometry.RightTriangle.Derived`, and
`Mathlib.Geometry.Metric.Abstract` to avoid repurposing released concrete
module names.

`Mathlib.Geometry.Pythagorean` is used for the final theorem surface because it
exports the Pythagorean theorem and law-of-cosines specializations rather than
another abstract structure package.

No declaration-name collisions were found inside the selected public modules.
Module-scoped overlaps with already released concrete geometry modules are
documented as namespace-policy caveats.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Geometry.Affine` | `Mathlib.Geometry.Affine` | materialize as a local public module |
| `Proofs.Ai.Geometry.AffineDerive` | `Mathlib.Geometry.Affine.Derived` | materialize as a local public module |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | `Mathlib.Geometry.RightTriangle.Abstract` | materialize as a local public module |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | `Mathlib.Geometry.RightTriangle.Derived` | materialize as a local public module |
| `Proofs.Ai.Geometry.AbstractMetric` | `Mathlib.Geometry.Metric.Abstract` | materialize as a local public module |
| `Proofs.Ai.Geometry.Pythagorean` | `Mathlib.Geometry.Pythagorean` | materialize as a local public module |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | already public |
| `Proofs.Ai.Vector.AbstractInnerProduct` | `Mathlib.LinearAlgebra.InnerProduct` | already public |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `Mathlib.LinearAlgebra.InnerProduct.Derived` | already public |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` | already public |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Mathlib.Algebra.OrderedField.ScalarIdentities` | already public |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale route-specific
`Proofs.Ai.Geometry.*` names for the selected closure.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.19`
baseline.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The selected modules have:

| Module | Direct axioms | Transitive axioms | Policy status |
| --- | --- | --- | --- |
| `Proofs.Ai.Geometry.Affine` | none | none | ok |
| `Proofs.Ai.Geometry.AffineDerive` | `Eq.rec` | `Eq.rec` | ok, already allowed |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | none | none | ok |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | `Eq.rec` | `Eq.rec` | ok, already allowed |
| `Proofs.Ai.Geometry.AbstractMetric` | `Eq.rec` | `Eq.rec` | ok, already allowed |
| `Proofs.Ai.Geometry.Pythagorean` | `Eq.rec` | `Eq.rec` | ok, already allowed |

The public module manifest entries should declare `axioms = []` for
`Mathlib.Geometry.Affine` and `Mathlib.Geometry.RightTriangle.Abstract`, and
`axioms = ["Eq.rec"]` for the four derived/final modules.

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Geometry.Affine` | `sha256:820b71f59ab6e1cd0e65959c238ea1a20bec73b90462d48367d178e8d50fb17c` | `sha256:2990a22fa5e1eeb3d638c8730c37e9715b6a4c1374c475db3cf07cef013de514` | `sha256:4979816eb5b94ff0fd65935f11a76484c03590473bf103eb2c1c0b4304880051` | `sha256:3d3fdbf6a3ca4756ceaac9853e839a84878b24c0f6290e2246a78c6184b31e0e` | `sha256:02200ef99e9a29763bba253e46875d7fcf50c310961f1367ef159cabbd3253fd` |
| `Proofs.Ai.Geometry.AffineDerive` | `sha256:9e9564ad8e4c1ef8c7af94430b6ade599db272de414ba8c8b0b5f6f1dc693047` | `sha256:3ed50f3cdd25fab4283772f11d6b0810c6fb1ce7f7e8cf6f5f9c6dc89d472811` | `sha256:952ab394c2bd8f278bb4ce88212dc89972c67a9af64e466bbeb6f0b7b7777285` | `sha256:31ec1ff27164f1a9d7251898656c6888bcda59666472aac23aac45eca8a6f44c` | `sha256:475fff3a8e7d950a12e210f6f04415b44434e2682ff6813a7e783dd5f7c6986e` |
| `Proofs.Ai.Geometry.AbstractRightTriangle` | `sha256:99b459b1616447d93f1b4740b8205c61f823c72a417b1cbf68220431e98004fa` | `sha256:fb88ad1a737d10c9ed453eb7247c2e2d8da60af92b6f398abde0da7edd948135` | `sha256:4e849f7938936db16ea84b5c0b48b7c81a8171b2947ddc2451c677b3b114eace` | `sha256:7715c06c634de0020345067c58ecf3bd23c10e71d84f4b97cd4c2e00e82c1a07` | `sha256:b818f6bb67873f8f899d75d992c8d2afb3a50187a0d6529eee975afd411b7e7d` |
| `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | `sha256:5ca4d6bb116b91c964e3f1aec00b3625cd38d41b01c77408b03d8f55a94d93ff` | `sha256:ca154e53921d10f654067b0fc4ab5e2221a32ace3abf2fb8cdb8d5e0d8b5b5b5` | `sha256:55a00a2ec96fb087e03ff235614bd8ae3aba60629825cdd0bf30a7d80d1c7728` | `sha256:c52744f66c569059e6a3785b92ab69c1954e8f5031de9c4f6e1002d5a029e06c` | `sha256:3b7123a358d0487934b527f433b8bd027e57f9b540f581bd6f90e5bbd0eed78d` |
| `Proofs.Ai.Geometry.AbstractMetric` | `sha256:619c6dff1b2e254ef03ba855da1b2af73ea8d1866d039415fdd9c80d6907f6f2` | `sha256:90387a971d741f8fb119b52a9838a6df856eaba22d79c8f7223e15e5aadf14ca` | `sha256:7ceb20daa4fa1060de30b37a91b8184e529381df0b3d1d7036fb7bf6c42e5fdd` | `sha256:b683ec26dd9da17affedaeb69c226b9b8b3a3dd7bc68442117e84b903240ec1b` | `sha256:ba19db44c9bb86bebcd692f8ec454c5b8dae511850406b9efd040ffcd8396c3c` |
| `Proofs.Ai.Geometry.Pythagorean` | `sha256:62f6f82a4144b9306998746cd6f24abf963e766c0bb21ce65c66cda6829517ba` | `sha256:3a8c95687768687719fafcd699924479f0c9bf1a94318e00f1a36b66bcd025f9` | `sha256:eb05a29d4cf6110896d4b1cc5473ca511e3c1e620c00a79f16bc39eca7afd49a` | `sha256:067fa89c58bfb9d8a1a11be14a0d6e47a783909b69a3bda57dd93232c06af24b` | `sha256:0771e015c0b8285d4a2a7740036eac0e13e417510fd8fdb5ddee8fc0cb0e7e6c` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Pythagorean
```

Observed result:

```text
verified Proofs.Ai.Geometry.Pythagorean
verified 1 selected module(s), 15 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Pythagorean
```

Observed result:

```text
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.EqReasoning
built Proofs.Ai.Algebra.AbstractScalarDerive
built Proofs.Ai.Vector.AbstractSpace
built Proofs.Ai.Vector.AbstractInnerProduct
built Proofs.Ai.Geometry.Affine
built Proofs.Ai.Geometry.AbstractRightTriangle
built Proofs.Ai.Geometry.AffineDerive
built Proofs.Ai.Vector.AbstractInnerProductDerive
built Proofs.Ai.Geometry.AbstractMetric
built Proofs.Ai.Geometry.AbstractRightTriangleDerive
built Proofs.Ai.Geometry.Pythagorean
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Geometry.Pythagorean (14 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public geometry closure through
vendored certificate bytes and exercise these theorem names:

- `pythagorean_distance_general`
- `pythagorean_theorem_dist_sq`
- `pythagorean_theorem_api_alias`

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

- bad public export hash for `Mathlib.Geometry.Pythagorean` is rejected as
  `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Geometry.Pythagorean` is rejected
  as `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.Geometry.Pythagorean` are
  rejected by source-free reference verification;
- stale downstream `Mathlib.Geometry.Pythagorean` version pin is rejected as
  `package_lock_stale`.

## Materialization Result

Status: materialized as `npa-mathlib v0.1.20`.

Materialized public modules:

```text
Mathlib.Geometry.Affine
Mathlib.Geometry.Affine.Derived
Mathlib.Geometry.RightTriangle.Abstract
Mathlib.Geometry.RightTriangle.Derived
Mathlib.Geometry.Metric.Abstract
Mathlib.Geometry.Pythagorean
```

Public source hashes:

| Public module | Source hash |
| --- | --- |
| `Mathlib.Geometry.Affine` | `sha256:f1697fd4c3d4a81503821b1540ed3807ee97e9aaaed2bd5225257981cbace973` |
| `Mathlib.Geometry.Affine.Derived` | `sha256:4774d46cd5cfeffddc8484a8931d9680a440a4673eb65669c38baa4b576b3bc1` |
| `Mathlib.Geometry.RightTriangle.Abstract` | `sha256:dda93a618ea737a9bfa9fe359deedcae08497bad08d5df2962ccb66dd076cc44` |
| `Mathlib.Geometry.RightTriangle.Derived` | `sha256:2eee45f1afde28aaf959db810acaddc87c21b94ccc50e9cee1e2c9108c9e6701` |
| `Mathlib.Geometry.Metric.Abstract` | `sha256:5e01c606e2e731e0ee8bcd85c4fbcbff6dd310e848f3550bf12c39e6b0ee4c33` |
| `Mathlib.Geometry.Pythagorean` | `sha256:3e8f3d40e94a3a3e68568ebcd3fa42e3f9d1e4777e6777dbdc870e3bc4c3d794` |

Public certificate hashes:

| Public module | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `Mathlib.Geometry.Affine` | `sha256:12ce563b23b473796fc4bdf0e17fcbb2ccd3c4565534e9f17bc911eee9660b6c` | `sha256:e4e3f862585856027ffe88cb2ed9cf0705d3fc14aa7a05e05e56d17edf0b60c5` | `sha256:3d3fdbf6a3ca4756ceaac9853e839a84878b24c0f6290e2246a78c6184b31e0e` | `sha256:e1f00b05158d700bfcdd2a947a500ab1f58bec799ee255e66eb316077a0f2085` |
| `Mathlib.Geometry.Affine.Derived` | `sha256:3f2862c65cba4b2ba6f4052a493ddd6bce9c6e1639894216652e9cb398d1085b` | `sha256:952ab394c2bd8f278bb4ce88212dc89972c67a9af64e466bbeb6f0b7b7777285` | `sha256:31ec1ff27164f1a9d7251898656c6888bcda59666472aac23aac45eca8a6f44c` | `sha256:51c24ff1368a6b76548c439fecc97b55d6e5265c08d3e9aa6922b8f6775d8e3f` |
| `Mathlib.Geometry.RightTriangle.Abstract` | `sha256:4ad51ace89ad52cc869e0793a7c9a29bee48c99fa05f259fa3df0218fc0cb8b7` | `sha256:1ed84dc5fb2771b39e9780daafa2d67d237875716091e1ceb9762f105adbe501` | `sha256:7715c06c634de0020345067c58ecf3bd23c10e71d84f4b97cd4c2e00e82c1a07` | `sha256:12c9b50e2bce346b793d58b483e71e74f68184ba77d54a7575f33360025b6841` |
| `Mathlib.Geometry.RightTriangle.Derived` | `sha256:2418d3e1f3fafe72fc93113222a9f748105257316cd920cc8631fc84406d5bf6` | `sha256:5057ab1e9f7efb30fe7b278a3e47d6714cd3940a840c85a072f5ee06bec59eed` | `sha256:c52744f66c569059e6a3785b92ab69c1954e8f5031de9c4f6e1002d5a029e06c` | `sha256:3593d2fc159412ac4429f1b817e13c70c7e76586aa5fda298a3bb43a26c050d6` |
| `Mathlib.Geometry.Metric.Abstract` | `sha256:f787c65efc58e02729d3db0dbcc49ac56e7d81e31046fc6cd640aaee6b66d792` | `sha256:915b8dc17e88b5108d8bd450a40fa88a7c213b58d20399ce26c3ac6ad3cd2b48` | `sha256:b683ec26dd9da17affedaeb69c226b9b8b3a3dd7bc68442117e84b903240ec1b` | `sha256:e81ba9ee769c5a61ee7cf8b22672de6112bd8918c0666e00fda71894562abed8` |
| `Mathlib.Geometry.Pythagorean` | `sha256:cc20a83f3a2331a3d22a952ed713d577465cf121e437df9ae74ed89d727950f2` | `sha256:f68ec468e44dfdb9e309d35f41fa341b3bbdb880c6a186620ce67fd4cc6e5af4` | `sha256:067fa89c58bfb9d8a1a11be14a0d6e47a783909b69a3bda57dd93232c06af24b` | `sha256:75c412b9048c7b459c6879047b55e5a10619fdc2ef282ef4a39aa37ba7921f39` |

Downstream smoke result:

| Module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.GeometryPythagorean` | `sha256:ad214e2a5286b00f6479f4539924cde3d5f9e117b420c3e724f3c21da2988d0f` | `sha256:c361ae6e8cb8448cd0b3e35a8adb2954d484ad815ffd52a9b606428de20fa714` | `sha256:5e370d10f5d987491262eeedc8388932cb0ae37b3d4eecd53e502107450665bd` | `sha256:61f67892f3c0d3b605321c1a35de9591cc8ca80d585caf668d6bda23b6aaca07` | `sha256:766a759470c47fba4f9e68e5d87222e9221e2b8beccc5a0e02a86c2a202373cb` |

Positive gates passed:

```text
package check                         passed
package build-certs --check           passed
package verify-certs --checker reference passed; modules=55
package check-hashes                  passed
package axiom-report --check          passed
package index --check                 passed
package publish-plan --check          passed
downstream package check              passed
downstream package build-certs --check passed
downstream package verify-certs --checker reference passed; modules=16
downstream package check-hashes       passed
```

Negative checks passed on temporary package copies:

| Check | Observed rejection |
| --- | --- |
| Bad `Mathlib.Geometry.Pythagorean` export hash | `export_hash_mismatch` |
| Bad `Mathlib.Geometry.Pythagorean` certificate hash | `certificate_hash_mismatch` |
| Corrupted `Mathlib.Geometry.Pythagorean` certificate bytes before hash refresh | `certificate_file_hash_mismatch` |
| Corrupted `Mathlib.Geometry.Pythagorean` certificate bytes after refreshing file-hash pins in the temporary copy | `certificate_decode_failed` |
| Stale downstream `Mathlib.Geometry.Pythagorean` version pin | `package_lock_stale` |

Generated sidecar hashes:

```text
generated/package-lock.json   sha256:6f7d5bef8202347909c7b97ecdc6023a133fd78fda2c9f89b8c26aad9c3e8fd9
generated/axiom-report.json   sha256:ce747a25dd9e535edd6423b7116f470fbef845451fb3aa261c6ff9dde0e81376
generated/theorem-index.json  sha256:b2d48b938b0984e00f5622c23e45bcc88585378671bbc28c5b930a597f3617ad
generated/publish-plan.json   sha256:f46f06643eba25e294195006c6d4d98f6b8ff43063905bf9a87dc99b73e210ad
publish_plan_hash            sha256:54a03339db4cc12af5125163b89c58bf4c68c23ef3d5e8479069837d714b9318
```

Release artifact:

```text
target/release-artifacts/npa-mathlib-v0.1.20-release-artifacts.tar.gz
sha256:253ef90e75cc7c8f0b54c6e488b61291a327097d9bb016e6b41efe16253419dc
```
