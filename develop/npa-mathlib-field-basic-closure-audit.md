# Promotion Plan: Proofs.Ai.Algebra.AbstractField -> Mathlib.Algebra.Field.Basic

This plan is untrusted planning metadata. Proof acceptance still comes from canonical certificates, deterministic hashes, and source-free verification.

## Module Mapping

- Corpus module: `Proofs.Ai.Algebra.AbstractField`
- Target module: `Mathlib.Algebra.Field.Basic`
- Target source: `Mathlib/Algebra/Field/Basic/source.npa`
- Target certificate: `Mathlib/Algebra/Field/Basic/certificate.npcert`
- Corpus source: `Proofs/Ai/Algebra/AbstractField/source.npa`
- Corpus certificate: `Proofs/Ai/Algebra/AbstractField/certificate.npcert`
- Corpus meta: `Proofs/Ai/Algebra/AbstractField/meta.json`
- Corpus replay: `Proofs/Ai/Algebra/AbstractField/replay.json`

## Readiness Checklist

| Criterion | Status | Detail |
| --- | --- | --- |
| Name and statement stable | Verified evidence | `Mathlib.Algebra.Field.Basic` follows the namespace policy's `Basic` convention for the first stable module in a domain. The public declarations are the corpus field law-package surface without renaming. |
| Likely downstream reuse | Verified evidence | Direct corpus dependents include `Proofs.Ai.Algebra.AbstractFieldHom`, `Proofs.Ai.Algebra.AbstractFieldExtension`, and later finite-field / polynomial-field routes. |
| Import closure small | Verified evidence | Corpus closure has 2 internal module(s). |
| Axiom policy explicit | Verified evidence | Corpus module axioms: `none`; package allow-list: `Eq.rec`. |
| Source-free package evidence | Verified evidence | Corpus package artifact files and hashes are listed below; rerun gates before promotion. |
| Compatibility alias decision | Verified evidence | Use `--compat-alias none`; public package artifacts must not expose `Proofs.Ai.*` aliases. |

## Direct Import Mapping

| Corpus import | Proposed target import | Status |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | Verified evidence |

## Import Closure

| Corpus module | Proposed target module | Certificate | Source imports | Package imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | `Proofs/Ai/Algebra/AbstractRing/certificate.npcert` | `Std.Logic.Eq` | `Std.Logic.Eq` | `none` |
| `Proofs.Ai.Algebra.AbstractField` | `Mathlib.Algebra.Field.Basic` | `Proofs/Ai/Algebra/AbstractField/certificate.npcert` | `Proofs.Ai.Algebra.AbstractRing` | `Std.Logic.Eq, Proofs.Ai.Algebra.AbstractRing` | `none` |

## Deferred Nearby Modules

The selected closure intentionally materializes only the abstract field
foundation:

- `Proofs.Ai.Algebra.AbstractFieldHom`
- `Proofs.Ai.Algebra.AbstractFieldHomKernelImage`
- `Proofs.Ai.Algebra.AbstractFieldExtension`
- `Proofs.Ai.Algebra.AbstractFieldIdeal`
- `Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient`
- `Proofs.Ai.Algebra.AbstractAlgebraicExtension`
- `Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
- `Proofs.Ai.Algebra.AbstractFiniteField`
- `Proofs.Ai.Algebra.AbstractFieldIntegralDomain`

These modules depend on the field foundation but also introduce separate
homomorphism, extension, ideal, finite-field, or polynomial-field surfaces that
need their own public namespace review and downstream smoke coverage.

## Local Corpus Gates

These source-free and source-rebuild checks were run before materialization:

| Command | Status |
| --- | --- |
| `cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField` | passed; verified 1 selected module and 3 modules including dependency cache |
| `cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField` | passed; rebuilt `Proofs.Ai.Algebra.AbstractRing` and `Proofs.Ai.Algebra.AbstractField`; no repository diff |

## Public Exports

### Inductives

- None.

### Definitions

- `FieldFalse`
- `FieldNot`
- `Nonzero`
- `div`
- `FieldLawArgs`

### Theorems

- `field_ring_laws`
- `field_zero_ne_one`
- `field_inv_mul_cancel`
- `field_mul_inv_cancel`
- `field_div_eq_mul_inv`
- `field_inv_one`
- `field_div_one`
- `field_div_self_nonzero`
- `field_zero_div`
- `field_mul_left_cancel_nonzero`
- `field_mul_right_cancel_nonzero`
- `field_nonzero_mul_closed`
- `field_mul_eq_zero_cases`


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
| Corpus module metadata | `Proofs/Ai/Algebra/AbstractField/meta.json` | Verified evidence | source `sha256:570d26f6da6133797d99e88528c9d7dcb8f076199cde4f67970036bc566b3e9e`, certificate file `sha256:6a0ef30bfde4d9d94c6463a8e08c57f4be79bda6255dd08d46bd2f3622cdbf6b`, export `sha256:fba4c80edb5d353d74425fb5d090cde7837f108a0ba60997bf63d0d77ff07284`, axiom report `sha256:6fb10daa6bb059bfea6535d1d96e9ae73c2da1aee15921bf824480bbeee32550`, certificate `sha256:4bb6a8b4f1aa64dfbdf189c21738ea3fd12f17a9b77d335674c3354fc694a7d0` |
| Corpus manifest | `manifest.toml` | Verified evidence | file present; sha256:7e1010bf86f4ac584df08f52ca41f7e2b3000ed17acbf727f3035f66abab75d9 |
| Corpus package manifest | `npa-package.toml` | Verified evidence | file present; sha256:7e68aa6299bc2f97ed35b9a2c25e8511bf196968d4ed7652afb1d1f52116bafb |
| Corpus package lock | `generated/package-lock.json` | Verified evidence | file present; sha256:eb0f0806f884c1ac72c5a19603abcbff19f6ea45e52696fa9f6ae43045e39d0d |
| Corpus axiom report | `generated/axiom-report.json` | Verified evidence | file present; sha256:0951db3722935e23e99da6a804dc5d995e5cac2d674b6ed4739bd2bd9cfad042 |
| Corpus theorem index | `generated/theorem-index.json` | Verified evidence | file present; sha256:a1f2ef2bad234638cbd01b39e0b936527b88d32bfdd73d7e9393c946dbed6942 |
| npa-mathlib package manifest | `npa-package.toml` | Verified evidence | file present; sha256:8314c30633a48f6fe4eec5048df314152d3d6b7b238c10745ac9b837267260a3 |
| npa-mathlib package lock | `generated/package-lock.json` | Verified evidence | file present; sha256:7842659b30561dd243bc8a437a9c53dbe853af8291864ec2d259cf4691d0a699 |
| npa-mathlib axiom report | `generated/axiom-report.json` | Verified evidence | file present; sha256:78cdfae268e5ef29d859f15f1afe4c34c050c53934b4b118ab52cbc0dc2bd95e |
| npa-mathlib theorem index | `generated/theorem-index.json` | Verified evidence | file present; sha256:54c540f9c0d0e58f04a89e228bd814c29360fd9e079fb9226cc63bc8bbf9f3eb |
| npa-mathlib publish plan | `generated/publish-plan.json` | Verified evidence | file present; sha256:761448115c328ac7ca9427508d693884044de67d0cbf1ef23aae7a20f51d0413 |
| Downstream smoke manifest | `fixtures/downstream-smoke/npa-package.toml` | Verified evidence | file present; sha256:5d1535e58dd3d07823dd679bd6e3e3e1bf9f30bf0feb8e87de6f938045bd3319 |
| Stable statement review | `manual` | Missing evidence | Compare corpus statement against intended public Mathlib statement. |
| Compatibility alias review | `manual` | Verified evidence | No compatibility alias; use `--compat-alias none` and keep `Proofs.Ai.*` out of public package artifacts. |

## Negative Package-Copy Checks

Run these checks on temporary package copies after materialization:

| Check | Expected rejection reason |
| --- | --- |
| bad public export hash for `Mathlib.Algebra.Field.Basic` | `export_hash_mismatch` |
| bad public certificate hash for `Mathlib.Algebra.Field.Basic` | `certificate_hash_mismatch` |
| corrupted public certificate bytes for `Mathlib.Algebra.Field.Basic` | `certificate_decode_failed` or another verifier failure |
| stale downstream package-version pin or lock | `package_lock_stale` |

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

Status: materialized and verified in `npa-mathlib` v0.1.28 as
`Mathlib.Algebra.Field.Basic`.

Materialized public module:

| Item | Value |
| --- | --- |
| Module | `Mathlib.Algebra.Field.Basic` |
| Path | `Mathlib/Algebra/Field/Basic/` |
| Imports | `Std.Logic.Eq`, `Mathlib.Algebra.Ring.Basic` |
| Source hash | `sha256:2f03f5f56d4aecec1c0a27234a877438893a1b235bee099ec8044e2a1cbc8903` |
| Certificate file hash | `sha256:bc9987144ffb7997a759b752ebb92a6a3d9c4a0284081c954f791e18a2c5fc72` |
| Export hash | `sha256:777de3ddecd35e7f14255d942b5e4e31896a22fee9fd47e0b7669a52c8070c35` |
| Axiom report hash | `sha256:6fb10daa6bb059bfea6535d1d96e9ae73c2da1aee15921bf824480bbeee32550` |
| Certificate hash | `sha256:be49329fb3c992cf1d5e592c83f79c125c8be9697eb2acc001f7b345b0df92ed` |
| Module axioms | `[]` |

Package artifact hashes:

| Artifact | SHA-256 |
| --- | --- |
| `generated/package-lock.json` | `sha256:37822627e763fd625b0b5f349b29852f498d5a35403362a3e10fec3bfa4074cf` |
| `generated/axiom-report.json` | `sha256:66bf9ed8d98392381a86129ef4ca771bc0f15edb59fb1de73d617ce3a7e4a693` |
| `generated/theorem-index.json` | `sha256:71060e9a4b44d93661af8f90d03b77b3fc864dcbc17a4b050efe61ad0178abf7` |
| `generated/publish-plan.json` file | `sha256:2a40a73718085cc5150d513f9ee2a1740f951075142ec969458082cf989427a8` |
| `generated/publish-plan.json` content | `sha256:10714bd0bc8e0cf7598613bee9b1ae578fcbed6e90527ed77abcdc8e3f9647ba` |

Release artifact:

| Artifact | SHA-256 |
| --- | --- |
| `target/release-artifacts/npa-mathlib-v0.1.28-release-artifacts.tar.gz` | `sha256:05f743ea189f16b2cee987c336046715ea89f86979534b5622befa9686474330` |
| `target/release-artifacts/npa-mathlib-v0.1.28-release-artifacts.tar.gz.sha256` | `sha256:27191d982ccb6259c7a82f6c2f539cb60012785c342ba90711bf267309e7a8ed` |

Downstream smoke:

| Item | Value |
| --- | --- |
| Package | `fixtures/downstream-smoke` |
| Imported field module | `Mathlib.Algebra.Field.Basic` v0.1.28 |
| New theorem | `field_div_eq_mul_inv_passthrough` |
| Full theorem list | `field_div_eq_mul_inv_passthrough`, `implicit_augmented_map_derivative_passthrough`, `implicit_function_theorem_passthrough`, `implicit_function_derivative_theorem_passthrough` |
| Downstream source hash | `sha256:75770b47e994754a4b7f7cd1eec7646c06c3a90319ecdd80b54bfcab44ef34c8` |
| Downstream certificate file hash | `sha256:99838ae06c3c6844a79d3b68233ada0845351443f9a3145270262bcd2e10edd8` |
| Downstream export hash | `sha256:4f36060b285901ccc9a9d81c1f6f4979fd5766182e27cd083dffb645aeddc631` |
| Downstream axiom report hash | `sha256:cd08cfec03cc168b11b832b90fc5f22949e37b96febaa487ecbcef3d9e1faada` |
| Downstream certificate hash | `sha256:782e6c770641b245394c6f77c46c258a90840621a27bc369b55db70b3a9dfd97` |
| Downstream package lock file hash | `sha256:564f315683efce23d874540fcdbb7b929140dbeeaee383ea50629581e59e8270` |

Positive gates:

| Command | Status |
| --- | --- |
| `cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json` | passed |
| `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json` | passed; `package_verified`, 64 modules |
| `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json` | passed |
| `cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed; `package_verified`, 14 modules |
| `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `cargo fmt --all -- --check` | passed in `npa` |
| `cargo test -p npa-proof-corpus promote_materialize -- --nocapture` | passed; 7 tests |
| `git diff --check` | passed in `npa` and `npa-mathlib` |

Negative package-copy checks:

| Check | Observed rejection reason |
| --- | --- |
| Bad public export hash for `Mathlib.Algebra.Field.Basic` | `export_hash_mismatch` |
| Bad public certificate hash for `Mathlib.Algebra.Field.Basic` | `certificate_hash_mismatch` |
| Corrupted `Mathlib/Algebra/Field/Basic/certificate.npcert` bytes | `certificate_file_hash_mismatch` |
| Stale downstream `Mathlib.Algebra.Field.Basic` package-version pin | `package_lock_stale` |

Final namespace check:

- No stale `Proofs.Ai.Algebra.AbstractField` route names were found in public
  sources, manifests, generated package artifacts, publish plan, README, or
  downstream fixture files. The only remaining `Proofs.Ai.*` references are the
  intentional historical mapping rows in `docs/namespace-policy.md`.
- The materializer source-import table was corrected so the public field module
  carries the direct verified `Std.Logic.Eq` import required by the package
  compiler. Source, replay, metadata, theorem index, publish plan, and this
  audit remain untrusted sidecars; release evidence is the canonical certificate
  bytes and source-free checker verdicts above.
