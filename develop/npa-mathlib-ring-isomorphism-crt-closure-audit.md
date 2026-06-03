# npa-mathlib Ring First Isomorphism And CRT Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the abstract ring
foundation materialization. It selects the proof-corpus ring first
isomorphism and Chinese remainder route as one public closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The abstract ring foundation has been materialized, committed, tagged, and
  pushed in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
  `npa-mathlib v0.1.15`.
- The `npa-mathlib` release commit is
  `44bc2d8490ae6d551145fbc67511859f72eb990e`.
- The local ignored `v0.1.15` release-bundle tar hash is
  `sha256:027fa2b6571bda37e2f2702c7fccac046bf39693f80e222847d62b17252dbd82`.

This closure should materialize as `npa-mathlib v0.1.16`. It must not change
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
| `Proofs.Ai.Algebra.AbstractRingFirstIsoBase` | `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` | `Mathlib/Algebra/Ring/FirstIsomorphism/Basic/` | 9 definitions, 14 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, abstract ring, public group quotient/image modules | transitive `Eq.rec` |
| `Proofs.Ai.Algebra.AbstractRingFirstIso` | `Mathlib.Algebra.Ring.FirstIsomorphism` | `Mathlib/Algebra/Ring/FirstIsomorphism/` | 3 definitions, 9 theorems | first-isomorphism basic plus public group first-isomorphism modules | transitive `Eq.rec` |
| `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | `Mathlib.Algebra.Ring.ChineseRemainder` | `Mathlib/Algebra/Ring/ChineseRemainder/` | 4 definitions, 8 theorems | ring first-isomorphism modules plus public group quotient/image modules | transitive `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractRingFirstIsoBase` | `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` |
| `Proofs.Ai.Algebra.AbstractRingFirstIso` | `Mathlib.Algebra.Ring.FirstIsomorphism` |
| `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | `Mathlib.Algebra.Ring.ChineseRemainder` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Ring.FirstIsomorphism.Basic
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Image
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group

Mathlib.Algebra.Ring.FirstIsomorphism
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Image
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group
  imports Mathlib.Algebra.Group.FirstIsomorphism
  imports Mathlib.Algebra.Ring.FirstIsomorphism.Basic

Mathlib.Algebra.Ring.ChineseRemainder
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.FirstIsomorphism
  imports Mathlib.Algebra.Group.Image
  imports Mathlib.Algebra.Group.Kernel.Quotient
  imports Mathlib.Algebra.Group.Kernel.Quotient.Mul
  imports Mathlib.Algebra.Group.Kernel.Quotient.Group
  imports Mathlib.Algebra.Ring.Basic
  imports Mathlib.Algebra.Ring.FirstIsomorphism.Basic
  imports Mathlib.Algebra.Ring.FirstIsomorphism
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
other prerequisites are already public `npa-mathlib v0.1.15` modules.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` | `RingHomLawArgs`, `RingImagePred`, `RingKerQuot`, `RingKerQuotMk`, `RingKerQuotToS`, `RingKerQuotAdd`, `RingKerQuotZero`, `RingKerQuotNeg`, `RingKerQuotMulRep` | `ring_hom_zero`, `ring_hom_one`, `ring_hom_add`, `ring_hom_neg`, `ring_hom_mul`, `ring_as_additive_group_laws`, `ring_hom_as_additive_group_hom`, `ring_ker_quot_mul_rep_compat`, `ring_image_intro`, `ring_image_zero`, `ring_image_one`, `ring_image_add_closed`, `ring_image_neg_closed`, `ring_image_mul_closed` |
| `Mathlib.Algebra.Ring.FirstIsomorphism` | `RingKerQuotMul`, `RingKerQuotOne`, `RingFirstIso` | `ring_ker_quot_mul_mk`, `ring_first_iso_phi_zero`, `ring_first_iso_phi_one`, `ring_first_iso_phi_add`, `ring_first_iso_phi_mul`, `ring_first_iso_phi_injective`, `ring_first_iso_phi_hits_image`, `ring_first_iso_phi_surj_image`, `ring_first_isomorphism_to_image` |
| `Mathlib.Algebra.Ring.ChineseRemainder` | `RingCrtPairMap`, `RingCrtCombine`, `RingCrtIntersectionPred`, `RingChineseRemainder` | `ring_crt_intersection_intro`, `ring_crt_intersection_left`, `ring_crt_intersection_right`, `ring_crt_pair_hom_laws`, `ring_crt_kernel_to_intersection`, `ring_crt_intersection_to_kernel`, `ring_crt_pair_surjective`, `ring_chinese_remainder_theorem` |

## Closure Unit Rationale

This audit keeps ring first isomorphism and CRT together in `v0.1.16` because
`Proofs.Ai.Algebra.AbstractRingChineseRemainder` imports both selected ring
first-isomorphism modules and its final theorem depends on
`ring_first_isomorphism_to_image`. The resulting public closure is a coherent
ring-isomorphism theorem release.

`Proofs.Ai.Algebra.AbstractRingFirstIsoBase` and
`Proofs.Ai.Algebra.AbstractRingFirstIso` should not be separated from each
other because the latter imports the former and exports the final
`ring_first_isomorphism_to_image` evidence surface.

The following nearby modules are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Standalone ring homomorphism namespace | no separate corpus module yet | The corpus base module already bundles `RingHomLawArgs`, image predicates, and `RingKerQuot*` scaffolding. Creating `Mathlib.Algebra.Ring.Hom` would require a source/certificate split or an explicit alias layer. |
| Ordered algebra and square normalization | `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Algebra.AbstractScalarDerive` | These depend only on abstract ring and should remain the next algebraic foundation closure after the ring isomorphism/CRT route. |
| Higher commutative algebra seeds | `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`, `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`, `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`, `Proofs.Ai.Algebra.AbstractKrullTheorem` | These require additional ideal/domain/factorization namespace decisions. |

## Namespace Decision

`Mathlib.Algebra.Ring.FirstIsomorphism.Basic` is used for the corpus
`AbstractRingFirstIsoBase` module because the module is not only homomorphism
laws. It also exposes image and kernel-quotient scaffolding used by the first
isomorphism theorem.

`Mathlib.Algebra.Ring.Hom` is deferred. A future release can add it if the
corpus provides a separate homomorphism-only module or if an explicit public
alias layer is worth auditing.

The existing `RingKerQuot*` public identifiers are accepted for this closure.
They match the checked corpus surface and keep a clear distinction from the
already public additive-group quotient modules. Renaming them would require
new public source and certificates, so it is intentionally deferred.

No declaration-name collisions were found against the currently released
`npa-mathlib v0.1.15` public surface for the selected definitions and
theorems.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractRingFirstIsoBase` | `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractRingFirstIso` | `Mathlib.Algebra.Ring.FirstIsomorphism` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | `Mathlib.Algebra.Ring.ChineseRemainder` | materialize as a local public module |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | already public |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | already public |
| `Proofs.Ai.Algebra.AbstractGroup` | `Mathlib.Algebra.Group.Basic` | already public |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `Mathlib.Algebra.Group.Image` | already public |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `Mathlib.Algebra.Group.Kernel.Quotient` | already public specialized kernel quotient route |
| `Proofs.Ai.Algebra.AbstractGroupQuotientMul` | `Mathlib.Algebra.Group.Kernel.Quotient.Mul` | already public specialized kernel quotient route |
| `Proofs.Ai.Algebra.AbstractGroupQuotientGroup` | `Mathlib.Algebra.Group.Kernel.Quotient.Group` | already public specialized kernel quotient route |
| `Proofs.Ai.Algebra.AbstractGroupFirstIsoFull` | `Mathlib.Algebra.Group.FirstIsomorphism` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public sources, manifest, package lock, publish plan, and
downstream fixture must contain no stale `Proofs.Ai.Algebra.AbstractRingFirstIso*`
or `Proofs.Ai.Algebra.AbstractRingChineseRemainder` references.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.15`
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
| `Proofs.Ai.Algebra.AbstractRingFirstIsoBase` | none | `Eq.rec` | ok |
| `Proofs.Ai.Algebra.AbstractRingFirstIso` | none | `Eq.rec` | ok |
| `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | none | `Eq.rec` | ok |

The public module manifest entries should declare:

```toml
axioms = ["Eq.rec"]
```

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractRingFirstIsoBase` | `sha256:46a5b0d381e4e383d1c20a47f931352e5500d1e1edbcd475f39bada130a6c693` | `sha256:c59e19dbd1cfa06f2aa908d09d51d9669ac055d7b7c548e7d2a152b79077a470` | `sha256:9f8f5ffb6f11a3326fb57c947a73c204c167a861b46cf69b7794971bce7019d2` | `sha256:cb6d6f45a78dfb3b8ae3aaf5bf13880ac499739b7e45d8c6593356efc29367e3` | `sha256:b2cc8cbea389b4ccd12a52f6dca6740eb64e73b11032a5a0a03e8bdffb0787da` |
| `Proofs.Ai.Algebra.AbstractRingFirstIso` | `sha256:5f421b8c2d4d6ddd976e145e866ea5b9d284cb19da112c6f507ad3a31b10b3e8` | `sha256:09925484dec6d1f5d460fe915e10e2d2496fd4c874eb85d1a523301c5e61b01e` | `sha256:9ef53bef360e0e274a73ca9ba975f66eed4296483446e0ff633520a02fab9447` | `sha256:f39c6826ea5fd04328e88ce53f46ee3da7422779015b92583cdcef2697007f68` | `sha256:2a894bb9720c4af15b2a52f48255b29a3fbd352bf2b487e810134f210bfff9e2` |
| `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | `sha256:9122ca60598d34ed661b0ec9d8b87cfd1d39d3e1e2c22d0334e99362d4a0844e` | `sha256:a19f43f6d68312fc07384aee18f725e9388ee2a54cd324395adb64b2577951c0` | `sha256:b60589726ed65d621b4cd00bb74d286ba2ac024a76b34607e4ede2f61e719a5a` | `sha256:ef7d0eaa598be1de31fe3519ebceaa3a49511caf9ca312a771ba4e7272eaffdb` | `sha256:386d87ebbdd3fffc3e1c8789744856886a49a8831cf620c6d360d0c8e984f130` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected final corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractRingChineseRemainder
```

Observed result:

```text
verified Proofs.Ai.Algebra.AbstractRingChineseRemainder
verified 1 selected module(s), 13 module(s) including dependency cache
```

The selected final corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractRingChineseRemainder
```

Observed result:

```text
built Proofs.Ai.EqReasoning
built Proofs.Ai.Algebra.AbstractGroup
built Proofs.Ai.Algebra.AbstractGroupImage
built Proofs.Ai.Algebra.AbstractGroupQuotient
built Proofs.Ai.Algebra.AbstractGroupQuotientMul
built Proofs.Ai.Algebra.AbstractGroupQuotientGroup
built Proofs.Ai.Algebra.AbstractGroupQuotientHom
built Proofs.Ai.Algebra.AbstractGroupFirstIsoFull
built Proofs.Ai.Algebra.AbstractRing
built Proofs.Ai.Algebra.AbstractRingFirstIsoBase
built Proofs.Ai.Algebra.AbstractRingFirstIso
built Proofs.Ai.Algebra.AbstractRingChineseRemainder
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Algebra.AbstractRingChineseRemainder (12 module(s) including import closure)
```

The `--build-module` index side effect produced no `npa` worktree diff.

## Materialization Plan

Materialization should create `npa-mathlib v0.1.16` with:

- public module: `Mathlib.Algebra.Ring.FirstIsomorphism.Basic`
- public module: `Mathlib.Algebra.Ring.FirstIsomorphism`
- public module: `Mathlib.Algebra.Ring.ChineseRemainder`
- copied sidecars: `source.npa`, `replay.json`, `meta.json`
- generated certificates under the corresponding public paths
- downstream smoke fixture consuming vendored certificate bytes, not source
  files

Planned downstream smoke theorem names:

- `ring_first_isomorphism_to_image_passthrough`
- `ring_chinese_remainder_theorem_passthrough`

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

Materialized successfully as `npa-mathlib v0.1.16`.

Public modules:

| Public module | Public path | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Ring.FirstIsomorphism.Basic` | `Mathlib/Algebra/Ring/FirstIsomorphism/Basic/` | `sha256:7e96fa4562f6413b223f3cf9c5c2c11226ea81536f006134cb67f7f5e9e0b378` | `sha256:e2dd5e4096a80d518c34d9d5c399fa93335b8dc7f55f5ff77e06c0ed784c03fb` | `sha256:9f8f5ffb6f11a3326fb57c947a73c204c167a861b46cf69b7794971bce7019d2` | `sha256:cb6d6f45a78dfb3b8ae3aaf5bf13880ac499739b7e45d8c6593356efc29367e3` | `sha256:75c47310b5159c7d43f02978c9babf5bceaa86d8a17322cc2b2d31956991b5e3` |
| `Mathlib.Algebra.Ring.FirstIsomorphism` | `Mathlib/Algebra/Ring/FirstIsomorphism/` | `sha256:ae5ff402aafb79a21c3cce567f420d0a185e581ab9bb6a42d2302a9009c74250` | `sha256:cd5ed74355fac4f19568206e0b3709edad835c2f8396b7e5e54a8932c0efe094` | `sha256:9ef53bef360e0e274a73ca9ba975f66eed4296483446e0ff633520a02fab9447` | `sha256:f39c6826ea5fd04328e88ce53f46ee3da7422779015b92583cdcef2697007f68` | `sha256:094440c29ba32666c059099a3f94495e240872b703d1f4eb534b9b09a2e8aad4` |
| `Mathlib.Algebra.Ring.ChineseRemainder` | `Mathlib/Algebra/Ring/ChineseRemainder/` | `sha256:6e79e7e0514c81f547da7de06f1c9a63656995d23a2e29a1131e6b60cbc63279` | `sha256:c4c5cfffa946a0dbfaa9e615d881417da14751b4138c80582d4b694338b6c84c` | `sha256:b60589726ed65d621b4cd00bb74d286ba2ac024a76b34607e4ede2f61e719a5a` | `sha256:ef7d0eaa598be1de31fe3519ebceaa3a49511caf9ca312a771ba4e7272eaffdb` | `sha256:8e8c0736e1158db3a0a298ac41bb513ee87200d943455dfbf51cff9b17090a31` |

Downstream smoke materialized `Downstream.RingIsoCrt` with:

| Theorem | Purpose |
| --- | --- |
| `ring_first_isomorphism_to_image_passthrough` | consumes the public ring first-isomorphism theorem |
| `ring_chinese_remainder_theorem_passthrough` | consumes the public CRT theorem |

Downstream hashes:

| Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `sha256:32b9e2ab5bb415a9e8942974ca5732f4119b9a00baa78db8b7b3948329599df6` | `sha256:0a49d3b9b4207f15e16eee4ba3e36dc7e46f1865d62c48b25c49f051b9edd76f` | `sha256:6c1505dfd2719e5790be800e12794f9a6d18c58b72875520eed63db9a8d7282a` | `sha256:b3d0660b2cf1248a54992af7f5b4732ab56b318d9bd20e6574f91f8718b8c9f0` | `sha256:6cc7f1256741e31af8bf3590d6ad03d7ba1bc23e4ae662930f97f610d7fc3657` |

Positive gates:

| Gate | Status |
| --- | --- |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json` | passed, `modules=43` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed, `modules=14` |
| `package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |

Stale-name scan found only historical namespace-policy mapping entries for the
materialized corpus names. No stale route-specific `Proofs.Ai.*`,
`Downstream.RingBasic`, `RingBasic`, or `0.1.15` references remained in public
source, manifests, package artifacts, or fixture files.

Negative package-copy checks:

| Check | Observed reason code |
| --- | --- |
| bad public export hash for `Mathlib.Algebra.Ring.ChineseRemainder` | `export_hash_mismatch` |
| bad public certificate hash for `Mathlib.Algebra.Ring.ChineseRemainder` | `certificate_hash_mismatch` |
| corrupted public certificate bytes for `Mathlib.Algebra.Ring.ChineseRemainder` | `certificate_decode_failed` |
| stale downstream `Mathlib.Algebra.Ring.ChineseRemainder` version pin | `package_lock_stale` |

Release artifacts:

| Artifact | Hash |
| --- | --- |
| `/Users/kazuyoshitoshiya/ff/npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.16-release-artifacts.tar.gz` | `sha256:a4e103d6b3dbce064946c601eae897783db065efd53c3fbd098fb54bd046681a` |
| `generated/publish-plan.json` file | `sha256:31bcacf7510fc623675f5a1629503bf32e78efb2b615f156e2d7744e7f5c026c` |
| `generated/package-lock.json` file | `sha256:7eca758de81c54c725799fef96c832718116e8e7e3cfef0d6a4173e6e1f77b5d` |
| `generated/axiom-report.json` file | `sha256:9d689e3589721a57a44828635f20eb5ee2bb4f51019ef72b902d21351f169638` |
| `generated/theorem-index.json` file | `sha256:5fd788f5ed9535dd39c06d0415f011eca4ed8fc6851a881999bd8acf5ebcbfa9` |
| `fixtures/downstream-smoke/generated/package-lock.json` file | `sha256:466ca9644dabf9fe9d1e5fd752566a652b7f1b207cb9d5a519e6335c43a92ee1` |

The publish-plan internal hash is:

```text
sha256:0c20d19ec3f8b238caf4f8e64a9227ac8459e65e0f636d57e7846b6154ed214d
```
