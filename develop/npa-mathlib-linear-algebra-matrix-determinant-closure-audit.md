# Promotion Plan: Proofs.Ai.LinearAlgebra.Matrix.Determinant -> Mathlib.LinearAlgebra.Matrix.Determinant

This plan is untrusted planning metadata. Proof acceptance still comes from canonical certificates, deterministic hashes, and source-free verification.

## Module Mapping

- Corpus module: `Proofs.Ai.LinearAlgebra.Matrix.Determinant`
- Target module: `Mathlib.LinearAlgebra.Matrix.Determinant`
- Target source: `Mathlib/LinearAlgebra/Matrix/Determinant/source.npa`
- Target certificate: `Mathlib/LinearAlgebra/Matrix/Determinant/certificate.npcert`
- Corpus source: `Proofs/Ai/LinearAlgebra/Matrix/Determinant/source.npa`
- Corpus certificate: `Proofs/Ai/LinearAlgebra/Matrix/Determinant/certificate.npcert`
- Corpus meta: `Proofs/Ai/LinearAlgebra/Matrix/Determinant/meta.json`
- Corpus replay: `Proofs/Ai/LinearAlgebra/Matrix/Determinant/replay.json`

## Readiness Checklist

| Criterion | Status | Detail |
| --- | --- | --- |
| Name and statement stable | Verified evidence | `Mathlib.LinearAlgebra.Matrix.Determinant` follows the namespace policy's specialized theorem-group rule and was already listed as the determinant route under the matrix namespace. Public declarations describe determinant normalization, transpose invariance, basic multilinear/alternating evidence, and determinant product projections. |
| L2 proved-theorem status | Verified evidence | `proofs/linear-algebra-theorem-proof-roadmap.md` marks `LIN-06-CARD` as `L2 Derived certificate`; `LIN-06` status records `LAQ-010` complete for `Proofs.Ai.LinearAlgebra.Matrix.Determinant`; `LAQ-010` records determinant basic properties and determinant product theorem as `L2`. |
| Likely downstream reuse | Verified evidence | Direct downstream importer: `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`. Planned/referenced downstream routes include rank minor criteria, eigenvalue/characteristic polynomial, Cayley-Hamilton, matrix groups, exterior determinant bridge, and Schur/block determinant formulas. |
| Import closure small | Verified evidence | Effective promotion closure has 1 new internal module. `Proofs.Ai.LinearAlgebra.Matrix.Basic` is already public as `Mathlib.LinearAlgebra.Matrix.Basic` and is imported rather than rematerialized. |
| Axiom policy explicit | Verified evidence | Corpus module axioms: `none`; package allow-list: `Eq.rec`. |
| Source-free package evidence | Verified evidence | Corpus package artifact files and hashes are listed below; rerun gates before promotion. |
| Compatibility alias decision | Verified evidence | No public compatibility alias is requested for this promotion. Corpus modules keep their staging names; downstream public smoke should import `Mathlib.LinearAlgebra.Matrix.Determinant` directly. |

## Explicitly Deferred Nearby Modules

- `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`: `LIN-06`/`LAQ-011` L2, but it depends on determinant plus the row-reduction/system route and should be audited separately after those dependencies are public or split.
- `Proofs.Ai.LinearAlgebra.Matrix.Elimination` and `Proofs.Ai.LinearAlgebra.Systems.Basic`: `LIN-05` is marked `L1 Evidence package`, then `L2`, so this mixed closure is not promoted here.
- `Proofs.Ai.LinearAlgebra.Matrix.Rank`: `LIN-07` L2 target, but its closure imports vector-space/subspace/basis/linear-map/system/elimination/representation layers that are not all public L2 closures.
- `Proofs.Ai.LinearAlgebra.Matrix.Representation`: `LIN-04` L2 target, but its closure currently drags `LIN-T01` vector-space bridge material and should be split or audited separately.

## Public Naming Decision

- Public module: `Mathlib.LinearAlgebra.Matrix.Determinant`
- Public path: `Mathlib/LinearAlgebra/Matrix/Determinant/`
- Rationale: determinant is a specialized matrix theorem group; the namespace policy explicitly lists `Mathlib.LinearAlgebra.Matrix.Determinant` as the appropriate shape for this route.

## Direct Import Mapping

| Corpus import | Proposed target import | Status |
| --- | --- | --- |
| `Std.Logic.Eq` | `npa-std 0.1.0 (Std.Logic.Eq)` | Verified evidence |
| `Proofs.Ai.LinearAlgebra.Matrix.Basic` | `Mathlib.LinearAlgebra.Matrix.Basic` | Verified evidence |

## Import Closure

| Corpus module | Proposed target module | Certificate | Source imports | Package imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.LinearAlgebra.Matrix.Determinant` | `Mathlib.LinearAlgebra.Matrix.Determinant` | `Proofs/Ai/LinearAlgebra/Matrix/Determinant/certificate.npcert` | `Std.Logic.Eq, Proofs.Ai.LinearAlgebra.Matrix.Basic` | `Std.Logic.Eq, Mathlib.LinearAlgebra.Matrix.Basic` | `none` |

## Public Exports

### Inductives

- None.

### Definitions

- `DeterminantIdentityNormalized`
- `DeterminantTransposeInvariant`
- `DeterminantBasicProperties`
- `DeterminantProductDerivationEvidence`

### Theorems

- `determinant_identity_normalized_intro`
- `determinant_identity_normalized_eq`
- `determinant_transpose_invariant_intro`
- `determinant_transpose`
- `determinant_basic_properties_intro`
- `determinant_basic_identity_normalized`
- `determinant_basic_identity_eq`
- `determinant_basic_transpose`
- `determinant_basic_transpose_eq`
- `determinant_basic_multilinear`
- `determinant_basic_alternating`
- `determinant_product_derivation_evidence_intro`
- `determinant_product_basic_properties`
- `determinant_product_matrix_mul_laws`
- `determinant_product_from_derivation`
- `determinant_product_theorem`
- `determinant_product`


## Axiom Policy Diff

| Item | Corpus value | Target action |
| --- | --- | --- |
| `allow_custom_axioms` | `false` | Keep `false` in npa-mathlib unless a separate policy review approves a change. |
| `allowed_axioms` | `Eq.rec` | Target package allow-list must cover exactly the required axioms without accidental widening. |
| Module axioms | `none` | Verify the target module axiom report remains within package policy. |
| Expected core features | `none` | Source-free verifier must report the same required features. |
| Supported authoring features | `none` | Do not treat authoring support as release evidence. |

## Evidence

| Evidence | Path | Status | Detail |
| --- | --- | --- | --- |
| Corpus module metadata | `Proofs/Ai/LinearAlgebra/Matrix/Determinant/meta.json` | Verified evidence | source `sha256:8388484869a315e32f15f6f195728fd38b6c4446d1e259e2860e1544ca2daca0`, certificate file `sha256:995a8de398b605e1d330e9c494641afbade88b953f109a88185dfad9648501bf`, export `sha256:d0d416d83280d97bab0bb9a2c6f159ce49584ff1560db535cbf4c52efa0e3025`, axiom report `sha256:4ce54129453ca5f2cc9726d9e809e739b6640402d7515115c64b8169ae6c93a6`, certificate `sha256:3ec8f7dd297be64c1d8a1d1a7a8ef39e4ba11da6464b94d39468aabda89a7c75` |
| Corpus manifest | `manifest.toml` | Verified evidence | file present; sha256:7e1010bf86f4ac584df08f52ca41f7e2b3000ed17acbf727f3035f66abab75d9 |
| Corpus package manifest | `npa-package.toml` | Verified evidence | file present; sha256:7e68aa6299bc2f97ed35b9a2c25e8511bf196968d4ed7652afb1d1f52116bafb |
| Corpus package lock | `generated/package-lock.json` | Verified evidence | file present; sha256:eb0f0806f884c1ac72c5a19603abcbff19f6ea45e52696fa9f6ae43045e39d0d |
| Corpus axiom report | `generated/axiom-report.json` | Verified evidence | file present; sha256:0951db3722935e23e99da6a804dc5d995e5cac2d674b6ed4739bd2bd9cfad042 |
| Corpus theorem index | `generated/theorem-index.json` | Verified evidence | file present; sha256:a1f2ef2bad234638cbd01b39e0b936527b88d32bfdd73d7e9393c946dbed6942 |
| npa-mathlib package manifest | `npa-package.toml` | Verified evidence | file present; sha256:4d7d109d80457133eec1091420dc2992f6c964ade1d124c945e680fea21a3a37 |
| npa-mathlib package lock | `generated/package-lock.json` | Verified evidence | file present; sha256:4186fe1e25e9e1059f283bbdaffda766e53e6a4898938a0d2f05716d78c77c47 |
| npa-mathlib axiom report | `generated/axiom-report.json` | Verified evidence | file present; sha256:1264b900cabed0dfaa1165958f7cb06690c5f10c933767a411bed189a608c7fe |
| npa-mathlib theorem index | `generated/theorem-index.json` | Verified evidence | file present; sha256:8cf6b60fd2b7bb7e48256fbeeb1d0731e92d78b14eebe1f240312fc4e82e4b2c |
| npa-mathlib publish plan | `generated/publish-plan.json` | Verified evidence | file present; sha256:b14afad9544ff3b0ea0bca73896dc56a62f2560ab3b4b732e4e2fcf7d5c66c47 |
| Downstream smoke manifest | `fixtures/downstream-smoke/npa-package.toml` | Verified evidence | file present; sha256:616ad514908cc51d9fd4941b6164a3567ac0ffdda320d2d930559dc64c07aee0 |
| Downstream smoke package lock | `fixtures/downstream-smoke/generated/package-lock.json` | Verified evidence | file present; sha256:c5b38682bf473bb83e38492704f614cb0d7e18c0feae6879eb20e499156b418c |
| Stable statement review | `manual` | Verified evidence | Statements are copied unchanged modulo module/import rewriting into `Mathlib.LinearAlgebra.Matrix.Determinant`; public names are accepted for the determinant theorem layer. |
| Compatibility alias review | `manual` | Verified evidence | Use `--compat-alias none`; no old corpus-facing public alias is added. |

## Manual Verification Performed Before Materialization

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Determinant --verified-cache authoring
```

Result: `verified Proofs.Ai.LinearAlgebra.Matrix.Determinant`, with one selected module and the `Std.Logic.Eq` plus `Proofs.Ai.LinearAlgebra.Matrix.Basic` dependency cache verified.

## Gate Commands

Run these from the NPA repository after materialization or before accepting promotion evidence.

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
```

## Materialization Result

- Public package release: `npa-mathlib` version `0.1.30`.
- Public module: `Mathlib.LinearAlgebra.Matrix.Determinant`.
- Public path: `Mathlib/LinearAlgebra/Matrix/Determinant/`.
- Compatibility aliases: none.
- Materialization command:

```sh
cargo run -q -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-linear-algebra-matrix-determinant-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --apply \
  --compat-alias none
```

### Public Hashes

| Artifact | Hash |
| --- | --- |
| `Mathlib/LinearAlgebra/Matrix/Determinant/source.npa` | `sha256:447702e9ed1427c85562e6c0ff8bc95e63c2206c3238b3fcb14d491a06066ed2` |
| `Mathlib/LinearAlgebra/Matrix/Determinant/meta.json` | `sha256:131d4df7762b05154777d465884e94ef0468fabae5a59f1ac93d093b99937b18` |
| `Mathlib/LinearAlgebra/Matrix/Determinant/replay.json` | `sha256:e46bdbf1337384a184477800fd08bae47b43e8fef8ffdbec296d2e4cbed9e528` |
| `Mathlib/LinearAlgebra/Matrix/Determinant/certificate.npcert` file | `sha256:1dd9b1f9d53c405aeb643da672251a7c4a92b3113d850641246f9a9ed77dd8bc` |
| export hash | `sha256:b3758548406298afcae732d201b5c631a0b89efe46e957977169a4f26d14233c` |
| axiom report hash | `sha256:4ce54129453ca5f2cc9726d9e809e739b6640402d7515115c64b8169ae6c93a6` |
| canonical certificate hash | `sha256:bc7aae940a6f0fe171c1ed51403d3c47d0525e9bd006dc1f2c99ef9877c879d9` |
| `generated/publish-plan.json` | `sha256:b14afad9544ff3b0ea0bca73896dc56a62f2560ab3b4b732e4e2fcf7d5c66c47` |

### Downstream Smoke

- Downstream fixture import added: `Mathlib.LinearAlgebra.Matrix.Determinant`, pinned to package version `0.1.30`.
- Downstream theorem added: `determinant_product_passthrough`.
- Downstream source hash: `sha256:4b63678681e91c034d2d505bc66fb7626c10c92a252c57fc54c13ce4fb7c5500`.
- Downstream certificate file hash: `sha256:5e6b2d783ce748aa051d6045f0b46f7e99ccc9409060aca3b8ed852085a0f4d8`.
- Downstream export hash: `sha256:21d330347ca337022f661261e0a86c606f1186041276b72116cdb3b2eca1588c`.
- Downstream axiom report hash: `sha256:10f91bb612c8188b25fd8e8dae8f9685197c0cdbad4c1298bf1e60d633bc62c5`.
- Downstream canonical certificate hash: `sha256:028f3e626855c04fc85ec1b734106007de1b214a07c8185c9d0e3e5cffad5102`.

### Positive Gates

All required gates passed after materialization.

- `cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json`
- `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json`
  reported `package_verified` with `modules=66`.
- `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json`
- `cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json`
- `cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json`
- `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json`
- `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json`
  reported `package_verified` with `modules=16`.
- `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json`

### Negative Checks

Temporary-copy checks outside both repositories rejected the expected failures.

| Check | Command family | Observed reason code |
| --- | --- | --- |
| Bad public determinant export hash | `package check-hashes` | `export_hash_mismatch` |
| Bad public determinant certificate hash | `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public determinant certificate bytes | `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Stale downstream determinant package-version pin | `package build-certs --check` | `package_lock_stale` |

### Release Artifact

- Archive: `target/release-artifacts/npa-mathlib-v0.1.30-release-artifacts.tar.gz`
- SHA-256: `5826ad4950edf02ce38ef3f9da3eef3daa7e209ff8c332e1ddaefe0a3a9a6a36`
