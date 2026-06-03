# npa-mathlib Vector Space Foundation Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.17`
ordered algebra and square-normalization closure. It selects the abstract
vector-space law-package route as one public linear-algebra foundation
closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The ordered algebra and square-normalization closure has been materialized,
  committed, tagged, and pushed in `../npa-mathlib`
  as `npa-mathlib v0.1.17`.
- The `npa-mathlib` release commit is
  `447fdc1e74c8ecd00fc47a8c26718b2c8c6f2240`.
- The local ignored `v0.1.17` release-bundle tar hash is
  `sha256:12c53ca2f0cd10b7fc18ba4834ec3a5512083584f931ff54245fe433701b1ea4`.

This closure should materialize as `npa-mathlib v0.1.18`. It must not change
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
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | `Mathlib/LinearAlgebra/VectorSpace/` | 4 definitions, 23 theorems | abstract ordered field, abstract ring, abstract square normalize, `Std.Logic.Eq` | none |

The public namespace mapping is:

| Corpus module | Public linear-algebra module |
| --- | --- |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.LinearAlgebra.VectorSpace
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.OrderedField.Basic
  imports Mathlib.Algebra.OrderedField.Square
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
other prerequisites are already public `npa-mathlib v0.1.17` modules.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.LinearAlgebra.VectorSpace` | `vsub`, `linear_comb2`, `linear_comb3`, `VectorSpaceLawArgs` | `vec_sub_def`, `vec_add_assoc`, `vec_add_comm`, `vec_add_zero`, `vec_zero_add`, `vec_neg_add_cancel`, `vec_add_neg_cancel`, `sub_sub_sub_cancel`, `vec_sub_self`, `vec_sub_zero`, `vec_add_left_cancel`, `smul_add`, `add_smul`, `one_smul`, `mul_smul`, `zero_smul`, `smul_zero`, `neg_smul`, `smul_neg`, `vec_sub_eq_add_neg`, `sub_add_sub_cancel_left`, `linear_comb2_ext`, `linear_comb3_ext` |

## Closure Unit Rationale

This audit keeps the vector-space foundation to the single
`Proofs.Ai.Vector.AbstractSpace` corpus module because it forms a closed slice
over already public ordered-field, square-normalization, ring, and equality
modules. It also gives later analysis and inner-product routes the shared
`VectorSpaceLawArgs` API without bundling inner products, normed spaces, or
geometry into this release.

The following nearby modules are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Inner product identities | `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Vector.AbstractInnerProductDerive` | Depends on this vector-space surface and adds inner-product, norm-square, distance-square, and Cauchy-Schwarz surfaces. |
| Analysis normed spaces | `Proofs.Ai.Analysis.AbstractNormedSpace` | Depends only on this vector-space foundation and equality reasoning, but it has a distinct norm/product-space API. |
| Analysis linear maps and derivatives | `Proofs.Ai.Analysis.AbstractLinearMap`, `Proofs.Ai.Analysis.AbstractDerivative`, and downstream analysis modules | These are separate analysis closures over vector-space and normed-space foundations. |
| Abstract geometry / Pythagorean | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Geometry.Pythagorean` | Depends on vector-space and inner-product routes, so it remains a later geometry closure. |
| Concrete one-element vector route | `Proofs.Ai.Vector.Basic`, `Proofs.Ai.Vector.Dot` | Already public as `Mathlib.Vector.Basic` and `Mathlib.Vector.Dot`; this audit materializes the abstract law-package route instead. |

## Namespace Decision

`Mathlib.LinearAlgebra.VectorSpace` is used for the abstract vector-space
law-package surface. It is more stable than `Mathlib.Vector.AbstractSpace`
because the module provides scalar multiplication and vector-space laws, not
only a generic vector carrier.

The already released `Mathlib.Vector.Basic` and `Mathlib.Vector.Dot` modules
remain the concrete one-element vector and dot-product route. The selected
module intentionally reuses declaration names such as `vsub`, `vec_add_assoc`,
and `vec_sub_def` in a distinct public module. Downstream packages should
import either the concrete route or the abstract linear-algebra route according
to the proof surface they need, not both without a deliberate disambiguation
plan.

No declaration-name collisions were found inside the selected public module.
Module-scoped overlaps with already released concrete vector modules are
documented as namespace-policy caveats.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Vector.AbstractSpace` | `Mathlib.LinearAlgebra.VectorSpace` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractOrderedField` | `Mathlib.Algebra.OrderedField.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractSquareNormalize` | `Mathlib.Algebra.OrderedField.Square` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale
`Proofs.Ai.Vector.AbstractSpace` references.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.17`
baseline.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The selected module has:

| Module | Direct axioms | Transitive axioms | Policy status |
| --- | --- | --- | --- |
| `Proofs.Ai.Vector.AbstractSpace` | none | none | ok |

The public module manifest entry should declare `axioms = []` for
`Mathlib.LinearAlgebra.VectorSpace`.

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Vector.AbstractSpace` | `sha256:e3db64e32830222c7320aaacdb9827feebc43bdfd264292b2e66406fb6cce9c0` | `sha256:80557d4aa807f82125c6044bef1ac619e3e9a503c2eac132c0689bd5ac95398c` | `sha256:2665099f8cfe0789db8e597c477440e6fa0c89262d32b284e5491eaf27e098aa` | `sha256:2eb8901dcadbaffb969aabfd5e10172a4e17abc50558a0954952232d438fa2ed` | `sha256:2630b7fa0d19f9b235ac3c3a3bac6a70611d9b47fd861586c003d37ac0325fca` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Vector.AbstractSpace
```

Observed result:

```text
verified Proofs.Ai.Vector.AbstractSpace
verified 1 selected module(s), 5 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Vector.AbstractSpace
```

Observed result:

```text
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractOrderedField
built Proofs.Ai.Algebra.AbstractSquareNormalize
built Proofs.Ai.Vector.AbstractSpace
wrote proofs/generated/ai-theorem-index.json
built Proofs.Ai.Vector.AbstractSpace (4 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public vector-space closure
through vendored certificate bytes and exercise these theorem names:

- `VectorSpaceLawArgs`
- `linear_comb2_ext`
- `linear_comb3_ext`

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

- bad public export hash for `Mathlib.LinearAlgebra.VectorSpace` is rejected
  as `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.LinearAlgebra.VectorSpace` is
  rejected as `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.LinearAlgebra.VectorSpace`
  are rejected by source-free reference verification;
- stale downstream `Mathlib.LinearAlgebra.VectorSpace` version pin is rejected
  as `package_lock_stale`.

## Materialization Result

Materialization succeeded as `npa-mathlib v0.1.18`.

Public module hashes:

| Public module | Public path | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- | --- |
| `Mathlib.LinearAlgebra.VectorSpace` | `Mathlib/LinearAlgebra/VectorSpace/` | `sha256:f137a3b0263a3f06f8ec8f05aa2d9a31509ff479f848640b0917cf1be2bbd5fc` | `sha256:f12c2fd82693fb2cff04974bc76528ec588bfac597e3280bbc4fc19fea28444b` | `sha256:2665099f8cfe0789db8e597c477440e6fa0c89262d32b284e5491eaf27e098aa` | `sha256:2eb8901dcadbaffb969aabfd5e10172a4e17abc50558a0954952232d438fa2ed` | `sha256:f53bf9435015310b0a3e423d2609b00dab0dfcf30bd676fa012cfb2842e6db2d` |

Downstream smoke:

| Downstream module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.VectorSpace` | `sha256:04f515f9aff150561004033bc368aa8c76824fa6f7131e36dcc24e1dca40d779` | `sha256:046441767918707ab980cfa8d586f52ddbe652f1d10fef8a099ccf5b060d2213` | `sha256:b260c6b71cbe35853856a457f122672d5b7c829b52df5fe60cc07cc0599f14c8` | `sha256:0544a7cbc8ae41c11f909366f72b2c80b48b4429ded309accea7370850df08bb` | `sha256:6a2ced0f9140fe2747b04fc2432cf405e08fbb7911dbbd0f3fcd57147feb6ca5` |

The downstream smoke theorem names are:

- `vector_space_law_args_passthrough`
- `linear_comb2_ext_passthrough`
- `linear_comb3_ext_passthrough`

Positive gates:

| Command | Status |
| --- | --- |
| `package check --root ../npa-mathlib --json` | passed |
| `package build-certs --root ../npa-mathlib --check --json` | passed |
| `package verify-certs --root ../npa-mathlib --checker reference --json` | passed, `modules=47` |
| `package check-hashes --root ../npa-mathlib --json` | passed |
| `package axiom-report --root ../npa-mathlib --check --json` | passed |
| `package index --root ../npa-mathlib --check --json` | passed |
| `package publish-plan --root ../npa-mathlib --check --json` | passed |
| `package check --root ../npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed, `modules=6` |
| `package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json` | passed |

Stale-name scan found only this audit document, roadmap history, release
history for `v0.1.17`, the namespace-policy corpus-to-public mapping entry,
and the unrelated category example `Mathlib.Order.OrderedAlgebra`. No stale
route-specific `Proofs.Ai.Vector.AbstractSpace` references remained in public
source, manifest entries, package artifacts, or fixture files, and no
`Downstream.OrderedAlgebra` fixture references remained.

Negative package-copy checks:

| Check | Observed reason code |
| --- | --- |
| bad public export hash for `Mathlib.LinearAlgebra.VectorSpace` | `export_hash_mismatch` |
| bad public certificate hash for `Mathlib.LinearAlgebra.VectorSpace` | `certificate_hash_mismatch` |
| corrupted public certificate bytes for `Mathlib.LinearAlgebra.VectorSpace` | `certificate_decode_failed` |
| stale downstream `Mathlib.LinearAlgebra.VectorSpace` version pin | `package_lock_stale` |

Release artifacts:

| Artifact | Hash |
| --- | --- |
| `../npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.18-release-artifacts.tar.gz` | `sha256:bdd30780a9e8759730796ee0878f634ca546c991e023ccd25048ad63397cf791` |
| `generated/publish-plan.json` file | `sha256:fd76decd43af83ae324a680fb00bda9ebae653f4ff1ced53453802f5c5913a38` |
| `generated/package-lock.json` file | `sha256:62b53bf0791c6a05d7a50aaf558db101d0f75011cbf8f294a000336156b42d5e` |
| `generated/axiom-report.json` file | `sha256:8331e0c2cf7a9864e35016f383a141123512eb5ec4b70b24c4f42ffe498bf3e9` |
| `generated/theorem-index.json` file | `sha256:6e44ee6e197f02b2d46eaf455fb6fa7ce4802ef5a84f90d8d14d604f217f6100` |
| `fixtures/downstream-smoke/generated/package-lock.json` file | `sha256:aa4dbfdf2812c5883f0a3c52fadce5fdc507a4f9efb8bbc04a053b79a9848b53` |

The publish-plan internal hash is:

```text
sha256:fff96145fc4c6197fbb0416139a11d0fc691e8922278cf68657d27eb20dbb4f9
```
