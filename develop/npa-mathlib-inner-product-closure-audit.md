# npa-mathlib Inner Product Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.18`
vector-space foundation closure. It selects the abstract inner-product
law-package route as one public linear-algebra closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The vector-space foundation closure has been materialized, committed,
  tagged, and pushed in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
  `npa-mathlib v0.1.18`.
- The `npa-mathlib` release commit is
  `fc804d8d437a1bbf7a47eeee3bff3d9f8f881d8e`.
- The local ignored `v0.1.18` release-bundle tar hash is
  `sha256:bdd30780a9e8759730796ee0878f634ca546c991e023ccd25048ad63397cf791`.

This closure should materialize as `npa-mathlib v0.1.19`. It must not change
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
| `Proofs.Ai.Vector.AbstractInnerProduct` | `Mathlib.LinearAlgebra.InnerProduct` | `Mathlib/LinearAlgebra/InnerProduct/` | 5 definitions, 25 theorems | abstract ordered field, abstract ring, abstract square normalize, abstract vector space, `Std.Logic.Eq` | none |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `Mathlib.LinearAlgebra.InnerProduct.Derived` | `Mathlib/LinearAlgebra/InnerProduct/Derived/` | 0 definitions, 19 theorems | abstract ordered field, abstract ring, abstract scalar derive, equality reasoning, abstract vector space, abstract inner product, `Std.Logic.Eq` transitively | `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public linear-algebra module |
| --- | --- |
| `Proofs.Ai.Vector.AbstractInnerProduct` | `Mathlib.LinearAlgebra.InnerProduct` |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `Mathlib.LinearAlgebra.InnerProduct.Derived` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.LinearAlgebra.InnerProduct
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
  imports Mathlib.LinearAlgebra.VectorSpace

Mathlib.LinearAlgebra.InnerProduct.Derived
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.ScalarIdentities
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.LinearAlgebra.VectorSpace
  imports Mathlib.LinearAlgebra.InnerProduct
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import via the
already public prerequisite modules. The other prerequisites are already public
`npa-mathlib v0.1.18` modules.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.LinearAlgebra.InnerProduct` | `dot`, `normSq`, `distSq`, `PerpVec`, `InnerProductLawArgs` | `dot_comm`, `dot_add_left`, `dot_add_right`, `dot_neg_left`, `dot_neg_right`, `dot_sub_left`, `dot_sub_right`, `norm_sq_def`, `dist_sq_def`, `dot_self_eq_norm_sq`, `norm_sq_add`, `norm_sq_sub`, `norm_sq_add_of_dot_zero`, `norm_sq_sub_of_dot_zero`, `norm_sq_nonneg`, `parallelogram_law`, `polarization_identity`, `cauchy_schwarz`, `perp_vec_iff_dot_eq_zero`, `perp_vec_symm`, `norm_sq_zero_iff`, `dist_sq_nonneg`, `norm_sq_add_of_perp`, `norm_sq_sub_of_perp`, `quadratic_norm_nonneg` |
| `Mathlib.LinearAlgebra.InnerProduct.Derived` | none | `norm_sq_add_from_inner_args`, `norm_sq_sub_from_inner_args`, `parallelogram_law_from_inner_args`, `polarization_identity_from_inner_args`, `dot_neg_left_from_inner_args`, `norm_sq_neg_from_inner_args`, `norm_sq_add_of_dot_zero_from_args`, `norm_sq_add_of_perp_from_args`, `norm_sq_add_neg_left_from_inner_args`, `dot_zero_left_from_law_packages`, `dot_zero_right_from_law_packages`, `dot_eq_zero_of_norm_sq_zero_left_from_inner_args`, `dot_eq_zero_of_norm_sq_zero_right_from_inner_args`, `cauchy_schwarz_zero_left_from_law_packages`, `cauchy_schwarz_zero_right_from_law_packages`, `cauchy_schwarz_from_law_packages`, `norm_sq_nonneg_from_inner_args`, `dot_le_mul_sqrt_norm_sq_from_cauchy`, `norm_sq_add_le_square_sum_norms_from_cauchy` |

## Closure Unit Rationale

This audit keeps the inner-product closure to the base law package plus its
derived theorem module. The two modules form a closed slice over already public
ring, ordered-field, scalar-identity, equality-reasoning, and vector-space
modules, and they expose the final theorem names that later geometry routes
need.

The following nearby modules are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Abstract geometry / Pythagorean | `Proofs.Ai.Geometry.Affine`, `Proofs.Ai.Geometry.AffineDerive`, `Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Geometry.AbstractRightTriangleDerive`, `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.Pythagorean` | Depends on this inner-product route and has a separate affine, right-triangle, metric, and Pythagorean API. |
| Analysis normed spaces | `Proofs.Ai.Analysis.AbstractNormedSpace` | Depends on vector-space and equality-reasoning foundations, but it has a distinct norm/product-space API. |
| Analysis linear maps and derivatives | `Proofs.Ai.Analysis.AbstractLinearMap`, `Proofs.Ai.Analysis.AbstractDerivative`, and downstream analysis modules | These are separate analysis closures over vector-space and normed-space foundations. |
| Concrete one-element vector route | `Proofs.Ai.Vector.Basic`, `Proofs.Ai.Vector.Dot` | Already public as `Mathlib.Vector.Basic` and `Mathlib.Vector.Dot`; this audit materializes the abstract law-package route instead. |

## Namespace Decision

`Mathlib.LinearAlgebra.InnerProduct` is used for the abstract inner-product
law-package surface because it extends `Mathlib.LinearAlgebra.VectorSpace` and
exposes scalar-valued bilinear, norm-square, perpendicularity, and
Cauchy-Schwarz statements. The alternative `Mathlib.Vector.InnerProduct` would
blur the abstract law-package route with the already released concrete
one-element vector route.

`Mathlib.LinearAlgebra.InnerProduct.Derived` is used for theorem derivations
from full law-package arguments. It keeps the base module's direct law
projection theorems separate from derived scalar/vector algebra proofs while
leaving both modules importable as one coherent closure.

No declaration-name collisions were found inside the selected public modules.
Module-scoped overlaps with already released concrete vector modules are
documented as namespace-policy caveats.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Vector.AbstractInnerProduct` | `Mathlib.LinearAlgebra.InnerProduct` | materialize as a local public module |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `Mathlib.LinearAlgebra.InnerProduct.Derived` | materialize as a local public module |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | already public |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` | already public |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Mathlib.Algebra.OrderedField.ScalarIdentities` | already public |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale
`Proofs.Ai.Vector.AbstractInnerProduct` or
`Proofs.Ai.Vector.AbstractInnerProductDerive` references.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.18`
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
| `Proofs.Ai.Vector.AbstractInnerProduct` | none | none | ok |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `Eq.rec` | `Eq.rec` through equality reasoning | ok, already allowed |

The public module manifest entry should declare `axioms = []` for
`Mathlib.LinearAlgebra.InnerProduct` and `axioms = ["Eq.rec"]` for
`Mathlib.LinearAlgebra.InnerProduct.Derived`.

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Vector.AbstractInnerProduct` | `sha256:51f0fb7b87851097a219459a1451028f2e430d1bbd8834a708a963234865a13b` | `sha256:3f7aff366ad7e6799a5f069df73e0b269f9a01714531e9df735b643473275da5` | `sha256:d092f6724ae3e9a4a35f0e381760051e384a395c7958d4bbc19461415dfa57fa` | `sha256:b77f3c86856e05f2733c7652d6c3e85e91668d19ed18a1c42004749a8eddf0e8` | `sha256:13e8573343de549ab41835392a150e351970c3e104ebe41e40b15f49de6dac8b` |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | `sha256:e9e1905b6353a804a8301fe5cc11dedff06da131af9492e40b916ea0203489a9` | `sha256:7f6ea7a6227b62d5466eaae70257090772e945f78a2139222c4cdd37e6c4b6f5` | `sha256:49692d86254f8d7db9c292dd6cec7fe67938e15c85a570dd29a0f2a3550e8f0f` | `sha256:a932e43d59afb7af1fd32a8680d65c057b745979ae169927e86d27984c9e3144` | `sha256:3575665a329343fffdbee0485927d58eabf84b6d65e880b6cb4fdf1f1760868b` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Vector.AbstractInnerProductDerive
```

Observed result:

```text
verified Proofs.Ai.Vector.AbstractInnerProductDerive
verified 1 selected module(s), 9 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Vector.AbstractInnerProductDerive
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
built Proofs.Ai.Vector.AbstractInnerProductDerive
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Vector.AbstractInnerProductDerive (8 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public inner-product closure
through vendored certificate bytes and exercise these theorem names:

- `parallelogram_law`
- `polarization_identity`
- `cauchy_schwarz_from_law_packages`

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

- bad public export hash for `Mathlib.LinearAlgebra.InnerProduct.Derived` is
  rejected as `export_hash_mismatch`;
- bad public certificate hash for
  `Mathlib.LinearAlgebra.InnerProduct.Derived` is rejected as
  `certificate_hash_mismatch`;
- corrupted public certificate bytes for
  `Mathlib.LinearAlgebra.InnerProduct.Derived` are rejected by source-free
  reference verification;
- stale downstream `Mathlib.LinearAlgebra.InnerProduct.Derived` version pin is
  rejected as `package_lock_stale`.

## Materialization Result

Materialization succeeded as `npa-mathlib v0.1.19`.

Public module hashes:

| Public module | Public path | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- | --- |
| `Mathlib.LinearAlgebra.InnerProduct` | `Mathlib/LinearAlgebra/InnerProduct/` | `sha256:7161182e477f37fb3414ed13b4a59bb823bf153a9feddecf499090e327dfed93` | `sha256:639f7e84e6bcc70b3ce3ea2f2a0818b4186b66910e1f80ebfc26908f242b8a0c` | `sha256:af2c57ceb6b5ca583d5d5cc0db14a7c46b7752a0929e5a9d23c29baf2c1e7b75` | `sha256:b77f3c86856e05f2733c7652d6c3e85e91668d19ed18a1c42004749a8eddf0e8` | `sha256:b9ab7df58fed220a35a7e8b78ab111b5e167bf304568ab643b29c48945e6d06e` |
| `Mathlib.LinearAlgebra.InnerProduct.Derived` | `Mathlib/LinearAlgebra/InnerProduct/Derived/` | `sha256:828cf229d4443aa1e2f154bcfb23da543f282930e8e933ce918c07dd7e295d95` | `sha256:1b32abe72a05c450b5e0ed33342dea21178d08b4959a3eca40d553861b232b93` | `sha256:18878a4407d7b0fd95b8cbdbe09569a395d557d08e37d8d6d4528da463da29e8` | `sha256:a932e43d59afb7af1fd32a8680d65c057b745979ae169927e86d27984c9e3144` | `sha256:03cc30cefb649d78f185c97d31d41070dffd350ebddd91433e6378e2d1b72dc6` |

Downstream smoke:

| Downstream module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.InnerProduct` | `sha256:619ef8aa5d44b6bddf2fb4fb3ce42b70375926c5f32fce12824cdebf305cfb4e` | `sha256:3ed3968557f8b39c32c393e45ecf7f44c9b25377840588539755b7d7e44954fe` | `sha256:8d225672eaf4cad1f3fd50e15ee28a97a60dbbe56e698e1a728609384c518630` | `sha256:0544a7cbc8ae41c11f909366f72b2c80b48b4429ded309accea7370850df08bb` | `sha256:1ddd31e352504916c05fc087be9d16e521113be4912a33b3748ca09bf85a1eb7` |

The downstream smoke theorem names are:

- `parallelogram_law_passthrough`
- `polarization_identity_passthrough`
- `cauchy_schwarz_from_law_packages_passthrough`

Positive gates:

| Command | Status |
| --- | --- |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json` | passed, `modules=49` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed, `modules=10` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |

Stale-name scan found only this audit document, roadmap corpus-to-public
mapping/history, release history for `v0.1.18`, and namespace-policy
corpus-to-public mapping entries. No stale route-specific
`Proofs.Ai.Vector.AbstractInnerProduct` references remained in public source,
manifest entries, package artifacts, or fixture files, and no
`Downstream.VectorSpace` fixture references remained.

Negative package-copy checks:

| Check | Observed reason code |
| --- | --- |
| bad public export hash for `Mathlib.LinearAlgebra.InnerProduct.Derived` | `export_hash_mismatch` |
| bad public certificate hash for `Mathlib.LinearAlgebra.InnerProduct.Derived` | `certificate_hash_mismatch` |
| corrupted public certificate bytes for `Mathlib.LinearAlgebra.InnerProduct.Derived` | `certificate_decode_failed` |
| stale downstream `Mathlib.LinearAlgebra.InnerProduct.Derived` version pin | `package_lock_stale` |

Release artifacts:

| Artifact | Hash |
| --- | --- |
| `/Users/kazuyoshitoshiya/ff/npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.19-release-artifacts.tar.gz` | `sha256:5f8be1a0c810187afa8dd904b75a27f92fe92b175a8154c9e92a5a679b598e42` |
| `generated/publish-plan.json` file | `sha256:cf78941a8cc76435359d98ed7065b2dc1a7457685ef0281ea139d892a2d7aada` |
| `generated/package-lock.json` file | `sha256:24f78073553211c689c56b8ae4a22353e32d5f6d1f6f1993de5c86b63a3dba2d` |
| `generated/axiom-report.json` file | `sha256:e138aae80eaddb8a89c07dbc9a22645c0245055f56501b32bebb2c70f2396d25` |
| `generated/theorem-index.json` file | `sha256:a8735190b6956d2aee92f3755bcda612529767132128d84bc308fe0943db0ed3` |
| `fixtures/downstream-smoke/generated/package-lock.json` file | `sha256:5901fd5b5972074b0e2029a399ca38f4a88c5f66068780500c68aa6e8e6134bf` |

The publish-plan internal hash is:

```text
sha256:257ba46673f628eec03f42f99a24be4f05fe905324678bc119f3bb8c0fad0d4b
```
