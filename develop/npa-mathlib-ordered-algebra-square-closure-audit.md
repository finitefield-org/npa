# npa-mathlib Ordered Algebra And Square Normalization Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the ring
first-isomorphism and CRT materialization. It selects the proof-corpus ordered
field, square-normalization, and scalar-identity route as one public closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The ring first-isomorphism and CRT closure has been materialized, committed,
  tagged, and pushed in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
  `npa-mathlib v0.1.16`.
- The `npa-mathlib` release commit is
  `c950a66c864f507268bc515bcd69703078d0a12a`.
- The local ignored `v0.1.16` release-bundle tar hash is
  `sha256:a4e103d6b3dbce064946c601eae897783db065efd53c3fbd098fb54bd046681a`.

This closure should materialize as `npa-mathlib v0.1.17`. It must not change
package boundaries, registry assumptions, import identity rules, or proof trust
boundaries.

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
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` | `Mathlib/Algebra/OrderedField/Basic/` | 6 definitions, 24 theorems | `Proofs.Ai.Algebra.AbstractRing`, `Std.Logic.Eq` | none |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` | `Mathlib/Algebra/OrderedField/Square/` | 0 definitions, 16 theorems | abstract ordered field, abstract ring, `Std.Logic.Eq` | none |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Mathlib.Algebra.OrderedField.ScalarIdentities` | `Mathlib/Algebra/OrderedField/ScalarIdentities/` | 0 definitions, 13 theorems | abstract square normalize, abstract ring, equality reasoning, `Std.Logic.Eq` | `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Mathlib.Algebra.OrderedField.ScalarIdentities` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.OrderedField.Basic
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring.Basic

Mathlib.Algebra.OrderedField.Square
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic

Mathlib.Algebra.OrderedField.ScalarIdentities
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Square
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
other prerequisites are already public `npa-mathlib v0.1.16` modules or
modules selected in this audit.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.Algebra.OrderedField.Basic` | `le`, `lt`, `sqrt`, `Nonneg`, `Positive`, `OrderedFieldLawArgs` | `le_refl`, `le_trans`, `add_nonneg`, `mul_nonneg`, `square_nonneg`, `sqrt_nonneg`, `sqrt_square_of_nonneg`, `sqrt_mul_self`, `eq_of_square_eq_square_nonneg`, `add_le_add`, `mul_le_mul_nonneg`, `zero_le_two`, `le_antisymm`, `lt_of_le_of_ne`, `le_of_eq`, `sqrt_sq`, `sq_eq_zero_iff`, `sum_nonneg_eq_zero`, `square_completion_bound_from_ordered_args`, `le_of_sq_le_sq_nonneg_from_ordered_args`, `add_dist_nonneg_from_ordered_args`, `sqrt_sum_square_bound_from_ordered_args`, `le_mul_sqrt_of_sq_le_mul_nonneg_from_ordered_args`, `add_two_mul_le_sq_add_sqrt_from_ordered_args` |
| `Mathlib.Algebra.OrderedField.Square` | none | `square_def`, `mul_self_eq_square`, `sq_add`, `sq_sub`, `sum_two_squares_comm`, `cancel_double_zero_term`, `sq_zero`, `sq_one`, `sq_neg`, `two_mul`, `sq_eq_sq_of_eq_or_neg_eq`, `sq_add_eq_add_sq_add_two_mul`, `sq_sub_eq_add_sq_sub_two_mul`, `add_sq_eq_zero_iff`, `mul_two_zero_term`, `normalize_add_with_zero_cross_term` |
| `Mathlib.Algebra.OrderedField.ScalarIdentities` | none | `mul_two_zero_term_from_ring_args`, `cancel_double_zero_term_from_ring_args`, `normalize_add_with_zero_cross_term_from_ring_args`, `mul_two_neg_from_ring_args`, `add_neg_cross_term_to_sub_sum_from_ring_args`, `law_of_cosines_scalar_rhs_from_ring_args`, `two_mul_from_ring_args`, `add_sub_cross_cancel_from_ring_args`, `add_pairwise_commute_from_ring_args`, `add_cross_and_sub_cross_cancel_from_ring_args`, `parallelogram_scalar_rhs_from_ring_args`, `add_middle_to_front_from_ring_args`, `polarization_scalar_rhs_from_ring_args` |

## Closure Unit Rationale

This audit keeps ordered field, square normalization, and scalar identities
together in `v0.1.17` because the final scalar-identity route imports square
normalization, and square normalization imports ordered field. The resulting
public closure is a coherent prerequisite for vector-space, inner-product, and
abstract geometry theorem routes.

`Proofs.Ai.Algebra.AbstractScalarDerive` is included rather than deferred
because it has no additional mathematical prerequisite beyond the two selected
modules and already-public equality reasoning. It also gives the downstream
fixture a final nontrivial scalar identity to exercise.

The following nearby modules are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Abstract vector spaces | `Proofs.Ai.Vector.AbstractSpace` | Depends on this ordered algebra closure but should be a separate linear-algebra foundation release. |
| Inner product identities | `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Vector.AbstractInnerProductDerive` | Depends on vector-space and ordered algebra surfaces; the route is larger and should not be bundled into this algebra closure. |
| Abstract geometry / Pythagorean | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Geometry.Pythagorean` | Depends on vector-space and inner-product routes, so it remains a later geometry closure. |
| Concrete one-element ordered field and square routes | `Proofs.Ai.OrderedField`, `Proofs.Ai.Algebra.Square` | Already public as concrete modules; this audit materializes the abstract law-package route instead. |

## Namespace Decision

`Mathlib.Algebra.OrderedField.Basic` is used for the abstract ordered-field
law-package surface. The already released `Mathlib.Algebra.OrderedField`
module remains the concrete one-element ordered-field route.

`Mathlib.Algebra.OrderedField.Square` is used for square-normalization facts
over the abstract ordered-field/ring law packages. This keeps the abstract
route separate from the already released concrete `Mathlib.Algebra.Square`
module.

`Mathlib.Algebra.OrderedField.ScalarIdentities` is used for derived scalar RHS
identities that feed inner-product, law-of-cosines, parallelogram, and
polarization routes. A shorter `ScalarDerive` name is avoided because it
describes the proof-generation role rather than the public mathematical
surface.

The selected modules intentionally reuse declaration names such as `le`,
`sqrt`, `square_def`, `sq_add`, and `two_mul` in distinct modules. Downstream
packages should import either the concrete route or the abstract route
according to the proof surface they need, not both without a deliberate
disambiguation plan.

No declaration-name collisions were found inside the selected public modules.
Module-scoped overlaps with already released concrete modules are documented as
namespace-policy caveats.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Mathlib.Algebra.OrderedField.ScalarIdentities` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | already public |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public sources, manifest, package lock, publish plan, and
downstream fixture must contain no stale
`Proofs.Ai.Algebra.AbstractOrderedField`,
`Proofs.Ai.Algebra.AbstractSquareNormalize`, or
`Proofs.Ai.Algebra.AbstractScalarDerive` references.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.16`
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
| `Proofs.Ai.Algebra.AbstractOrderedField` | none | none | ok |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | none | none | ok |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `Eq.rec` | `Eq.rec` | ok |

The public module manifest entries should declare `axioms = []` for the first
two modules and `axioms = ["Eq.rec"]` for
`Mathlib.Algebra.OrderedField.ScalarIdentities`.

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `sha256:c3bea3a8e220653c14b2b4c7960050f426db753cf6b62f73791c02626113ab74` | `sha256:f58b0092b12cbca0c3e28ca8f0d08302298d56769a4dc61091e27c2830412708` | `sha256:9776f26ee39f4a6468dd15474356b44c2f9f2feb858d785b1399a96fb47dd89b` | `sha256:b77f3c86856e05f2733c7652d6c3e85e91668d19ed18a1c42004749a8eddf0e8` | `sha256:52b1cacffa499c5efbcee1d7cd74226dc2f53ceb63d47b98978619885cd9da0a` |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `sha256:f179fac43b37d92500dce44dbf86f76dc698aaf9813294fc44908a88efdda6c6` | `sha256:57144740d5d2ec1244e7fa57bc3dab02301bb50db041a2ec0beab2f789fd69a9` | `sha256:1ee23829bf6e36a395ccf56ea029a1120abbb9625d57d0e3b0016b6aab4e82ab` | `sha256:b1c966ff112a4d21e88c23443f7594255da9b1129a7d1cbea048b57e695c9f2e` | `sha256:79d400a9fb8963c72f831e3825fa5615d4d84086ce3ebe08a15eeb4ae5677198` |
| `Proofs.Ai.Algebra.AbstractScalarDerive` | `sha256:1fabe8d3bb84c796b238c17205e132ff36131c9cfc7501aa6e9c61c81429db70` | `sha256:8e4508a701870abd463a6c54ec575bc958239b62cc2c2897cfca12627bc96f2c` | `sha256:5cfce44c0b39d64991e1ce13f38da007e2a4146ce51eceee64568a9da72954e7` | `sha256:e07182db3dc623da3ef815d3fda91d057b11f0dff8f81befe6c60472cbf02dcf` | `sha256:7439787eed84b1556f1365091c3eabe56e6f266235805c35bb24b023ff089abe` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected final corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractScalarDerive
```

Observed result:

```text
verified Proofs.Ai.Algebra.AbstractScalarDerive
verified 1 selected module(s), 6 module(s) including dependency cache
```

The selected final corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractScalarDerive
```

Observed result:

```text
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.EqReasoning
built Proofs.Ai.Algebra.AbstractScalarDerive
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Algebra.AbstractScalarDerive (5 module(s) including import closure)
```

The `--build-module` index side effect produced no `npa` worktree diff.

## Materialization Plan

Materialization should create `npa-mathlib v0.1.17` with:

- public module: `Mathlib.Algebra.OrderedField.Basic`
- public module: `Mathlib.Algebra.OrderedField.Square`
- public module: `Mathlib.Algebra.OrderedField.ScalarIdentities`
- copied sidecars: `source.npa`, `replay.json`, `meta.json`
- generated certificates under the corresponding public paths
- downstream smoke fixture consuming vendored certificate bytes, not source
  files

Planned downstream smoke theorem names:

- `sqrt_sq_passthrough`
- `sq_add_eq_add_sq_add_two_mul_passthrough`
- `polarization_scalar_rhs_from_ring_args_passthrough`

## Positive Gates

Run these gates in `/Users/kazuyoshitoshiya/ff/npa-mathlib` after
materialization:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
```

Run these downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

## Negative Package-Copy Checks

Run these checks in temporary copies outside both repositories:

| Check | Expected reason code |
| --- | --- |
| bad public export hash | `export_hash_mismatch` |
| bad public certificate hash | `certificate_hash_mismatch` |
| corrupted public certificate bytes | verifier failure such as `certificate_decode_failed` |
| stale downstream version or lock | `package_lock_stale` |

## Materialization Result

Materialized successfully as `npa-mathlib v0.1.17`.

Public modules:

| Public module | Public path | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- | --- |
| `Mathlib.Algebra.OrderedField.Basic` | `Mathlib/Algebra/OrderedField/Basic/` | `sha256:4d326541c8684f869b3470dd00a67400dafe6cb4f1022baafb24b945dbfb44b6` | `sha256:0dd58a419cffe01c9a1e928c20b3127818502879eb5b9c8007256e9f92773923` | `sha256:d413f9f1558949e004cde3f49fd5622cb00b0a0fa312d08a04b22f23f13262ac` | `sha256:b77f3c86856e05f2733c7652d6c3e85e91668d19ed18a1c42004749a8eddf0e8` | `sha256:fc2f924e2f2d6df6286dee945cf07465733a2617871eae80e45e1c160ba3a53e` |
| `Mathlib.Algebra.OrderedField.Square` | `Mathlib/Algebra/OrderedField/Square/` | `sha256:7cb4aabb88d65edf953eb5f2572c7c6d0b0392d64cea29f5e64a4e15eda23db7` | `sha256:fae673ab05ad289b783fae31a2f5992dcc81863491167c4189e0700386c76e1a` | `sha256:1ee23829bf6e36a395ccf56ea029a1120abbb9625d57d0e3b0016b6aab4e82ab` | `sha256:b1c966ff112a4d21e88c23443f7594255da9b1129a7d1cbea048b57e695c9f2e` | `sha256:633a349876b76b967256ba42f75f83c2c2f48428af7f7712d1c0c4d6f3a8d388` |
| `Mathlib.Algebra.OrderedField.ScalarIdentities` | `Mathlib/Algebra/OrderedField/ScalarIdentities/` | `sha256:fd2b54bd0738780f87b01b777bcfab8892e0f8c5c04a5c77dc793ad850979bcb` | `sha256:5f74dde73d0d7ed701b58da85f529f43f4654e1782c8f69e6f6bf6763d6433a7` | `sha256:a1d3023263d92db731c77b25c5d9cef404011eb133bea508b943d01cc829a20b` | `sha256:e07182db3dc623da3ef815d3fda91d057b11f0dff8f81befe6c60472cbf02dcf` | `sha256:11fc4faa5d98f79f6082b55a38d36321f827c87dc12d9aa02111cbf171854919` |

Downstream smoke materialized `Downstream.OrderedAlgebra` with:

| Theorem | Purpose |
| --- | --- |
| `sqrt_sq_passthrough` | consumes the public abstract ordered-field square-root theorem |
| `sq_add_eq_add_sq_add_two_mul_passthrough` | consumes the public square-normalization theorem |
| `polarization_scalar_rhs_from_ring_args_passthrough` | consumes the public derived scalar RHS identity |

Downstream hashes:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:f63fdb6b9a5b57d3d98050ed4a5b5f8930480d9efcd2d3eefe91ae37e869d44b` | `sha256:327ad20bae38573ede13d76342ab49855b9ef8f90eebbf49ce6b7c5e19a22602` | `sha256:1086a652868e79ee6ddc29644bf0834b5adab6ce54a0f54f5b183d8816702c47` | `sha256:e9300baeff990276ee43e2e40557b16d7753ca3edf37a90af7dbb378a877117f` | `sha256:aa23a74f7d5aa8a92be938af5bf90dddae42b121cbb060a2ec2db5ccf0f89b8c` |

Positive gates:

| Gate | Status |
| --- | --- |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json` | passed, `modules=46` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed, `modules=7` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |

Stale-name scan found only historical namespace-policy mapping entries and the
previous `v0.1.16` release documentation. No stale route-specific
`Proofs.Ai.*`, `Downstream.RingIsoCrt`, or `RingIsoCrt` references remained
in public source, manifests, package artifacts, or fixture files.

Negative package-copy checks:

| Check | Observed reason code |
| --- | --- |
| bad public export hash for `Mathlib.Algebra.OrderedField.ScalarIdentities` | `export_hash_mismatch` |
| bad public certificate hash for `Mathlib.Algebra.OrderedField.ScalarIdentities` | `certificate_hash_mismatch` |
| corrupted public certificate bytes for `Mathlib.Algebra.OrderedField.ScalarIdentities` | `certificate_decode_failed` |
| stale downstream `Mathlib.Algebra.OrderedField.ScalarIdentities` version pin | `package_lock_stale` |

Release artifacts:

| Artifact | Hash |
| --- | --- |
| `/Users/kazuyoshitoshiya/ff/npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.17-release-artifacts.tar.gz` | `sha256:12c53ca2f0cd10b7fc18ba4834ec3a5512083584f931ff54245fe433701b1ea4` |
| `generated/publish-plan.json` file | `sha256:f29c78090b499df40cf089f2f3a7bedb02e6fa2f66fae860f9bb52854e58560f` |
| `generated/package-lock.json` file | `sha256:6512bb59a1b8f0984b7159ed697e221a2bce2df3cf1b32215a242af3923e234c` |
| `generated/axiom-report.json` file | `sha256:61443978b0ff19a779484b07c73eab6ab1573328edd2be3f8b2894d78a364b74` |
| `generated/theorem-index.json` file | `sha256:0cb9e2ae15635f449e60f15106632707187df666a6718f43d0d66f4d76f6c5ba` |
| `fixtures/downstream-smoke/generated/package-lock.json` file | `sha256:d82c2b92e546e4a35d19634010e7b291dcc66158c2336b1ace8154c3cfe138ea` |

The publish-plan internal hash is:

```text
sha256:85a81a92068336f9bf06b03b1d37d43a4874919c1ef6f69a7445e00ba9018b1f
```
