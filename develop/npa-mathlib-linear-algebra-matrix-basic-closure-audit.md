# Promotion Plan: Proofs.Ai.LinearAlgebra.Matrix.Basic -> Mathlib.LinearAlgebra.Matrix.Basic

This plan is untrusted planning metadata. Proof acceptance still comes from canonical certificates, deterministic hashes, and source-free verification.

## Module Mapping

- Corpus module: `Proofs.Ai.LinearAlgebra.Matrix.Basic`
- Target module: `Mathlib.LinearAlgebra.Matrix.Basic`
- Target source: `Mathlib/LinearAlgebra/Matrix/Basic/source.npa`
- Target certificate: `Mathlib/LinearAlgebra/Matrix/Basic/certificate.npcert`
- Corpus source: `Proofs/Ai/LinearAlgebra/Matrix/Basic/source.npa`
- Corpus certificate: `Proofs/Ai/LinearAlgebra/Matrix/Basic/certificate.npcert`
- Corpus meta: `Proofs/Ai/LinearAlgebra/Matrix/Basic/meta.json`
- Corpus replay: `Proofs/Ai/LinearAlgebra/Matrix/Basic/replay.json`

## Readiness Checklist

| Criterion | Status | Detail |
| --- | --- | --- |
| Name and statement stable | Verified evidence | `Mathlib.LinearAlgebra.Matrix.Basic` follows the namespace policy's `Basic` rule for a first stable domain module and leaves `Matrix.Determinant`, `Matrix.Rank`, and representation modules under the same prefix. Public declarations describe matrix carriers, pointwise equality, entrywise operations, product evidence, and square-matrix law projections. |
| L2 proved-theorem status | Verified evidence | `proofs/linear-algebra-theorem-proof-roadmap.md` marks `LIN-04-CARD` as `L2 Derived certificate` for `Proofs.Ai.LinearAlgebra.Matrix.Basic`; `LAQ-007` records matrix representation and composition/product as `L2`; `LIN-04` status records `LAQ-007` complete for this module. |
| Likely downstream reuse | Verified evidence | Direct downstream importers include `Proofs.Ai.LinearAlgebra.Matrix.Representation`, `Proofs.Ai.LinearAlgebra.Systems.Basic`, `Proofs.Ai.LinearAlgebra.Matrix.Elimination`, `Proofs.Ai.LinearAlgebra.Matrix.Determinant`, `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`, and `Proofs.Ai.LinearAlgebra.Matrix.Rank`. |
| Import closure small | Verified evidence | Corpus closure has 1 internal module(s). |
| Axiom policy explicit | Verified evidence | Corpus module axioms: `none`; package allow-list: `Eq.rec`. |
| Source-free package evidence | Verified evidence | Corpus package artifact files and hashes are listed below; rerun gates before promotion. |
| Compatibility alias decision | Verified evidence | No public compatibility alias is requested for this promotion. Corpus modules keep their staging names; downstream public smoke should import `Mathlib.LinearAlgebra.Matrix.Basic` directly. |

## Explicitly Deferred Nearby Modules

- `Proofs.Ai.LinearAlgebra.Matrix.Representation`: depends on vector-space, subspace, basis, linear-map, and matrix basics; promote separately after this matrix foundation is public.
- `Proofs.Ai.LinearAlgebra.Matrix.Determinant`: `LIN-06` L2 target, but depends on the matrix foundation and should be audited as its own theorem layer.
- `Proofs.Ai.LinearAlgebra.Matrix.Elimination`: row-reduction route remains a separate `LIN-05` layer because its roadmap level is `L1 Evidence package`, then `L2`.
- `Proofs.Ai.LinearAlgebra.Systems.Basic`: depends on vector-space/subspace/basis/linear-map/matrix foundations and is not part of the minimal matrix basic closure.
- `Proofs.Ai.LinearAlgebra.Matrix.Adjugate` and `Proofs.Ai.LinearAlgebra.Matrix.Rank`: later matrix theorem layers depending on determinant/elimination/representation routes.

## Public Naming Decision

- Public module: `Mathlib.LinearAlgebra.Matrix.Basic`
- Public path: `Mathlib/LinearAlgebra/Matrix/Basic/`
- Rationale: the namespace policy uses `Basic` for first stable domain modules when a more specific name is not better, and lists `Mathlib.LinearAlgebra.Matrix.Determinant` as a later specialized theorem group. `Matrix.Basic` keeps the foundation separate from determinant, rank, representation, and elimination layers.

## Direct Import Mapping

| Corpus import | Proposed target import | Status |
| --- | --- | --- |
| `Std.Logic.Eq` | `npa-std 0.1.0 (Std.Logic.Eq)` | Verified evidence |

## Import Closure

| Corpus module | Proposed target module | Certificate | Source imports | Package imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.LinearAlgebra.Matrix.Basic` | `Mathlib.LinearAlgebra.Matrix.Basic` | `Proofs/Ai/LinearAlgebra/Matrix/Basic/certificate.npcert` | `Std.Logic.Eq` | `Std.Logic.Eq` | `none` |

## Public Exports

### Inductives

- None.

### Definitions

- `Matrix`
- `MatrixEq`
- `MatrixAdd`
- `MatrixTranspose`
- `MatrixProductEvidence`
- `SquareMatrixMulLawArgs`

### Theorems

- `matrix_intro`
- `matrix_eq_intro`
- `matrix_eq_apply`
- `matrix_add_entry`
- `matrix_transpose_entry`
- `matrix_product_evidence_intro`
- `matrix_product_eq`
- `square_matrix_mul_law_args_intro`
- `square_matrix_mul_assoc`
- `square_matrix_left_unit`
- `square_matrix_right_unit`


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
| Corpus module metadata | `Proofs/Ai/LinearAlgebra/Matrix/Basic/meta.json` | Verified evidence | source `sha256:40ae0beab7cda8f974f6e1722ac9dbd5b512bcc3d66552d662c84510964fa040`, certificate file `sha256:41ec1d746fcc9a1205c409bd1d972881ff6382341723727877329f699c77bbc6`, export `sha256:c51eb2d5d58db376c70c8219013bec5c5ad16d33c5f99ca6f3603bc54c072888`, axiom report `sha256:55932adb6d068a32ac76b43afee2b808d61b89bb36b85b1805fe77d82a1028b3`, certificate `sha256:9f4bebc967e5fde57e57f89b4a10101c10321849e6a907d6d8913363b354f296` |
| Corpus manifest | `manifest.toml` | Verified evidence | file present; sha256:7e1010bf86f4ac584df08f52ca41f7e2b3000ed17acbf727f3035f66abab75d9 |
| Corpus package manifest | `npa-package.toml` | Verified evidence | file present; sha256:7e68aa6299bc2f97ed35b9a2c25e8511bf196968d4ed7652afb1d1f52116bafb |
| Corpus package lock | `generated/package-lock.json` | Verified evidence | file present; sha256:eb0f0806f884c1ac72c5a19603abcbff19f6ea45e52696fa9f6ae43045e39d0d |
| Corpus axiom report | `generated/axiom-report.json` | Verified evidence | file present; sha256:0951db3722935e23e99da6a804dc5d995e5cac2d674b6ed4739bd2bd9cfad042 |
| Corpus theorem index | `generated/theorem-index.json` | Verified evidence | file present; sha256:a1f2ef2bad234638cbd01b39e0b936527b88d32bfdd73d7e9393c946dbed6942 |
| npa-mathlib package manifest | `npa-package.toml` | Verified evidence | file present after materialization; sha256:5b0a253065477e9ae42c0959ee2c9b31c12db46994549add30e95bfc3e3fa8d3 |
| npa-mathlib package lock | `generated/package-lock.json` | Verified evidence | file present after materialization; sha256:6a1061d66fa07d3816548b0ac2bfa7b7b199d67f8f43ddbec9b3302b0a7d46c7 |
| npa-mathlib axiom report | `generated/axiom-report.json` | Verified evidence | file present after materialization; sha256:068c849d5d7cc569e8cd4418f8a26238e819d65aff72daf7ca1b1235173563ab |
| npa-mathlib theorem index | `generated/theorem-index.json` | Verified evidence | file present after materialization; sha256:29d8b74b2da4a503fec7048d21116c112c87ab9aafabc5775e5c2569b33ad6d7 |
| npa-mathlib publish plan | `generated/publish-plan.json` | Verified evidence | file present after materialization; sha256:6d90eeba37ec571e501aebbe8b64e88c1d859ea723583568ee25077a3c8cc097 |
| Downstream smoke manifest | `fixtures/downstream-smoke/npa-package.toml` | Verified evidence | file present after materialization; sha256:0b495bd44a8a24d9c9bd0653994ac854bfbb40726d8d5f57d5d7d421b910d990 |
| Stable statement review | `manual` | Verified evidence | Statements are copied unchanged modulo module/import rewriting into `Mathlib.LinearAlgebra.Matrix.Basic`; public names are accepted for the matrix foundation layer. |
| Compatibility alias review | `manual` | Verified evidence | Use `--compat-alias none`; no old corpus-facing public alias is added. |

## Manual Verification Performed Before Materialization

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Basic --verified-cache authoring
```

Result: `verified Proofs.Ai.LinearAlgebra.Matrix.Basic`, with one selected module and the `Std.Logic.Eq` dependency cache verified.

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

- Public module: `Mathlib.LinearAlgebra.Matrix.Basic`
- Public path: `Mathlib/LinearAlgebra/Matrix/Basic/`
- Package version: `0.1.29`
- Compatibility aliases: none.
- Materialization command: `cargo run -q -p npa-proof-corpus -- --promote-materialize develop/npa-mathlib-linear-algebra-matrix-basic-closure-audit.md --mathlib-root ../npa-mathlib --apply --compat-alias none`
- Materialized files: `source.npa`, `certificate.npcert`, `meta.json`, `replay.json`, and `npa-package.toml`.

### Public Hashes

| Item | SHA-256 |
| --- | --- |
| Public source file | `40ae0beab7cda8f974f6e1722ac9dbd5b512bcc3d66552d662c84510964fa040` |
| Public meta sidecar | `360a5499ab84572a37877aae6f257990ea348d671e56965dc359f314658bd526` |
| Public replay sidecar | `a800be2b9d159cda88aa1f1f59dcdf1a1e64c98f989249b599d3da3b27a47434` |
| Public certificate file | `05e0035794fac5bcbaf7de1b16c384524f4d0c5d2422cbf990bbbd5efa8a6a3e` |
| Public export hash | `7aebf76d1c55b942aa7f7dcbea99239745a1c9dcf30bc4c27944e9e31d00dc9e` |
| Public axiom-report hash | `55932adb6d068a32ac76b43afee2b808d61b89bb36b85b1805fe77d82a1028b3` |
| Public certificate hash | `f6e25b4a787b0c468da309b0b02e2a7764e706ad5b6c93be3b5b3f2cb5cf304b` |
| Publish-plan file | `6d90eeba37ec571e501aebbe8b64e88c1d859ea723583568ee25077a3c8cc097` |

### Downstream Smoke

- Smoke module: `Downstream.ImplicitFunction`
- Added public import: `Mathlib.LinearAlgebra.Matrix.Basic`
- Added theorem: `matrix_intro_passthrough`

| Item | SHA-256 |
| --- | --- |
| Downstream source | `4c27b7e9582366d8f68fabe4ec4669d74e227d2e8bc7f4fefbcbe4f08b27f813` |
| Downstream certificate file | `3fea2560cc53254468604f34c7bf60770ebcffb58723aa7ab83b2b96f18d3b2e` |
| Downstream export hash | `edad5e4b804cc39983908664b41df8d1e630576ab2c34920857e14cd9f8af64c` |
| Downstream axiom-report hash | `3899386f4a669615995b3a194a635a81861411983a4397d9bf88a91ae998e71c` |
| Downstream certificate hash | `5878cbe35746ec0b942f1705816eb9bbf82ce7a4a8241e2ffe0e148c079493c4` |
| Downstream package lock | `653a577906a62f8dfc681446268c631bfba3a1e73fd32b3369ea5bdd66a7f0cc` |

### Positive Gates

All commands below passed:

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
git diff --check
git -C ../npa-mathlib diff --check
```

The reference verifier reported `package_verified` for `../npa-mathlib`
with 65 modules and for the downstream smoke fixture with 15 modules.

### Negative Checks

All checks used temporary package copies outside the repositories:

| Check | Command family | Observed reason code |
| --- | --- | --- |
| Bad public export hash | `package check-hashes` | `export_hash_mismatch` |
| Bad public certificate hash | `package check-hashes` | `certificate_hash_mismatch` |
| Corrupted public certificate bytes | `package verify-certs --checker reference` | `certificate_file_hash_mismatch` |
| Stale downstream package-version pin/lock | `package build-certs --check` | `package_lock_stale` |

### Release Artifact

- Artifact: `../npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.29-release-artifacts.tar.gz`
- SHA-256: `709a925ae6f3c3e28fed223254c3042ec68074cece63403282e9abf6eb324882`
