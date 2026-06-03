# npa-mathlib Abstract Ring Foundation Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the Layer 3E
`Logic.Iff` materialization. It selects the proof-corpus abstract ring law
package route as the next public closure. Ring first isomorphism, CRT, ordered
algebra, vector spaces, geometry, analysis, and lower-priority commutative
algebra seeds remain out of this layer.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- Layer 3E has been materialized, committed, tagged, and pushed in
  `/Users/kazuyoshitoshiya/ff/npa-mathlib` as `npa-mathlib v0.1.14`.
- The `npa-mathlib` release commit is
  `1a8dbdeb43da59d3b40f22b07d458b415e9783c0`.
- The local ignored `v0.1.14` release-bundle tar hash is
  `sha256:eacfc65d24ee5b9b6c678c0ab9b134c64c11a357301fb18f1fe33f7fcf3a740e`.

This closure should materialize as `npa-mathlib v0.1.15`. It must not change
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
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | `Mathlib/Algebra/Ring/Basic/` | 3 definitions, 25 theorems | `Std.Logic.Eq` | no direct axioms |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Ring.Basic
  imports Std.Logic.Eq
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. No
already released `Mathlib.*` module is required for this closure.

The selected module introduces the following public surface:

- definitions:
  - `two`
  - `sq`
  - `RingLawArgs`
- theorems:
  - `sub_eq_add_neg`
  - `add_assoc`
  - `add_comm`
  - `add_zero`
  - `zero_add`
  - `neg_add_cancel`
  - `add_neg_cancel`
  - `sub_self`
  - `mul_assoc`
  - `mul_comm`
  - `mul_one`
  - `one_mul`
  - `left_distrib`
  - `right_distrib`
  - `mul_zero`
  - `zero_mul`
  - `add_left_cancel`
  - `ring_normalize_add_mul3`
  - `add_right_cancel`
  - `neg_neg`
  - `sub_zero`
  - `zero_sub`
  - `sub_add_cancel`
  - `add_sub_cancel`
  - `sub_add_sub_cancel`

## Closure Unit Rationale

`Proofs.Ai.Algebra.AbstractRing` is an appropriate single-module foundation
closure:

- it imports only `Std.Logic.Eq`;
- it provides the law-package surface required by ring first isomorphism, CRT,
  ordered algebra, vector-space, geometry, and commutative-algebra seed routes;
- it does not require pulling in any theorem-family module to become useful;
- downstream smoke can exercise the final surface with `RingLawArgs`,
  `sub_add_cancel`, and `ring_normalize_add_mul3`.

The following nearby modules are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Ring first isomorphism | `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`, `Proofs.Ai.Algebra.AbstractRingFirstIso` | These build on the abstract ring law package and group quotient/image/first-isomorphism imports. They should be audited after the public ring foundation exists. |
| Ring CRT | `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | This imports the ring first-isomorphism route and should either follow it or be included in a separately justified ring-isomorphism theorem closure. |
| Ordered algebra and square normalization | `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Algebra.AbstractScalarDerive` | These depend on abstract ring and need a separate namespace decision around the already public concrete ordered-field/square modules. |
| Higher algebra seeds | `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`, `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`, `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`, `Proofs.Ai.Algebra.AbstractKrullTheorem` | These require abstract ring plus domain, ideal, factorization, and algebraic-geometry namespace audits. |

## Namespace Decision

The selected public module is `Mathlib.Algebra.Ring.Basic`, not
`Mathlib.Algebra.Ring`, because the existing released `Mathlib.Algebra.Ring`
module is a concrete one-element ring API with declarations such as `RingElem`,
`zero`, `one`, `add`, `mul`, `sub_eq_add_neg`, and `ring_normalize_add_mul3`.
The abstract route is a law-package API over an arbitrary `Scalar`.

`Mathlib.Algebra.Ring.Abstract` and `Mathlib.Algebra.Ring.Laws` were considered.
`Basic` is preferred for this release because it becomes the foundation for the
future public abstract ring namespace; later ring homomorphism, quotient, first
isomorphism, CRT, ideal, and factorization modules can import
`Mathlib.Algebra.Ring.Basic`.

There are declaration-name overlaps with already released modules:

| Existing module | Overlapping names |
| --- | --- |
| `Mathlib.Algebra.Ring` | `sub_eq_add_neg`, `add_assoc`, `add_comm`, `add_zero`, `zero_add`, `neg_add_cancel`, `add_neg_cancel`, `sub_self`, `mul_assoc`, `mul_comm`, `mul_one`, `one_mul`, `mul_zero`, `zero_mul`, `left_distrib`, `right_distrib`, `add_left_cancel`, `ring_normalize_add_mul3` |
| `Mathlib.Algebra.Square` | `two`, `sq` |

These overlaps are accepted for this release because package artifacts identify
public declarations by module and declaration name, and future abstract ring
routes should import `Mathlib.Algebra.Ring.Basic` rather than the old concrete
`Mathlib.Algebra.Ring` / `Mathlib.Algebra.Square` route. Downstream smoke must
not import the old concrete ring/square modules together with
`Mathlib.Algebra.Ring.Basic`.

The audit should fail materialization if the package checker rejects these
module-scoped duplicate declaration names.

## Import Rewrite Table

The only materialization rewrite is the package module name itself:

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractRing` | `Mathlib.Algebra.Ring.Basic` | materialize as a local public module |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale `Proofs.Ai.Algebra.AbstractRing`
reference.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.14`
baseline.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entry should declare:

```toml
axioms = []
```

The expected axiom status for `Mathlib.Algebra.Ring.Basic` is:

- direct axioms: none
- transitive axioms: none
- policy status: ok
- policy violations: none

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes for `Proofs.Ai.Algebra.AbstractRing`:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:1a545c8cce7c0efe5f0c4754d63d5f846d055a329705397b3d5c95569c13dc71` |
| certificate file hash | `sha256:e86d9cafaacdb1545c2d9131c332cfe9a185b609aa5d7aae20132a46cbfe9948` |
| export hash | `sha256:d9ee6937c14ad1e94c85d5b4eb664022da239a794802a948601c207a0152f2ff` |
| axiom report hash | `sha256:aa19bce6d8162a8b9cbf3d4c5c9b7076a45a326d4ab073bcbb2177328a00ae12` |
| certificate hash | `sha256:1545d9e5ef5d90e4bc2c2bcb9a9726600c587ac01a1eaedf5748ac562d5cf3ca` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. The public certificate/export hashes can differ from the
corpus hashes because the module name changes from
`Proofs.Ai.Algebra.AbstractRing` to `Mathlib.Algebra.Ring.Basic`.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractRing
```

Observed result:

```text
verified Proofs.Ai.Algebra.AbstractRing
verified 1 selected module(s), 2 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractRing
```

Observed result:

```text
built Proofs.Ai.Algebra.AbstractRing
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Algebra.AbstractRing (1 module(s) including import closure)
```

The `--build-module` index side effect produced no `npa` worktree diff.

## Materialization Plan

Materialization should create `npa-mathlib v0.1.15` with:

- public module: `Mathlib.Algebra.Ring.Basic`
- public path: `Mathlib/Algebra/Ring/Basic/`
- copied sidecars: `source.npa`, `replay.json`, `meta.json`
- generated certificate: `Mathlib/Algebra/Ring/Basic/certificate.npcert`
- downstream smoke fixture consuming vendored certificate bytes, not source
  files

Planned downstream smoke theorem names:

- `ring_law_args_passthrough`
- `sub_add_cancel_passthrough`
- `ring_normalize_add_mul3_passthrough`

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

Materialization succeeded as `npa-mathlib v0.1.15`.

Public module:

| Module | Path |
| --- | --- |
| `Mathlib.Algebra.Ring.Basic` | `Mathlib/Algebra/Ring/Basic/` |

Public materialization hashes:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:37af595454630f02c434e510923320218158ab32d9d5da8fe84ecf576acc4a5c` |
| certificate file hash | `sha256:db64bef589981cb8c68d395a924761ffde4f68e3c8f06cee763caf36ca0e2009` |
| export hash | `sha256:d9ee6937c14ad1e94c85d5b4eb664022da239a794802a948601c207a0152f2ff` |
| axiom report hash | `sha256:aa19bce6d8162a8b9cbf3d4c5c9b7076a45a326d4ab073bcbb2177328a00ae12` |
| certificate hash | `sha256:9c1d44e6906a80b92a7439a2da9a80938940744d0802c544db51f5aa3aa4390f` |

The public source sidecar is semantically identical to the corpus route but is
stored without an extra trailing blank line. The public certificate file hash
and certificate hash differ from the corpus hashes because the public module
identity changed from `Proofs.Ai.Algebra.AbstractRing` to
`Mathlib.Algebra.Ring.Basic`. The export and axiom-report hashes are unchanged.

Downstream smoke module:

| Module | Imports | Theorems |
| --- | --- | --- |
| `Downstream.RingBasic` | `Std.Logic.Eq`, `Mathlib.Algebra.Ring.Basic` | `ring_law_args_passthrough`, `sub_add_cancel_passthrough`, `ring_normalize_add_mul3_passthrough` |

Downstream smoke hashes:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:c0655bacb671724cbcfd114a1f9bbf91c317cf553de0ac45b6f2a29c7c2140d4` |
| certificate file hash | `sha256:d5ccfad73c75e819ea0d2629f4d0a07133d22a4747a811d0239f44a112ad6172` |
| export hash | `sha256:018f82d948ea4d07ecb0a3e01f4f3f1c3a963119408f1a7d00a034eb402c0567` |
| axiom report hash | `sha256:0544a7cbc8ae41c11f909366f72b2c80b48b4429ded309accea7370850df08bb` |
| certificate hash | `sha256:583abc6c1ad619ef6055a473fb289a41fc447b1fc26f04ca5836b7eba9ea1cec` |

Positive gates passed:

| Command | Status |
| --- | --- |
| `cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json` | passed, `package_verified`, `modules=40` |
| `cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json` | passed |
| `cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json` | passed |
| `cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |
| `cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json` | passed |
| `cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json` | passed, `package_verified`, `modules=3` |
| `cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json` | passed |

The package checker accepted the module-scoped declaration-name overlaps with
`Mathlib.Algebra.Ring` and `Mathlib.Algebra.Square`. The downstream smoke
fixture imports only `Mathlib.Algebra.Ring.Basic` from the overlapping abstract
route and does not import the old concrete ring/square route.

Negative package-copy checks used temporary copies under `/tmp` and observed
the expected rejection reason codes:

| Check | Observed reason code |
| --- | --- |
| bad public export hash | `export_hash_mismatch` |
| bad public certificate hash | `certificate_hash_mismatch` |
| corrupted public certificate bytes | `certificate_decode_failed` |
| stale downstream version pin | `package_lock_stale` |

Generated sidecar hashes:

| Artifact | SHA-256 |
| --- | --- |
| `generated/publish-plan.json` | `sha256:7767e0cc45883c378a0809b2cbd5c29b023305684111396b2f1ba496dffd66d5` |
| internal publish plan hash | `sha256:6626f3a256f56b6d732296691d07cf75dac92b2fbe5081f7b530e6ad7f782451` |
| `generated/package-lock.json` | `sha256:75346f965c202c5a438b363314ccaafb444753667ccf645a1f822f3830eb98ba` |
| `generated/axiom-report.json` | `sha256:15bb36c37433f739ff65b1a5e2289ad50148739c4e4297f982b8077d0375c0d8` |
| `generated/theorem-index.json` | `sha256:f4d63a27a3eec9acc357f05abb87b9d9a86e527d262ec09d86d6cdd38486f449` |
| `fixtures/downstream-smoke/generated/package-lock.json` | `sha256:dae8d51015728fc39f9100bd7b375ba4c23da4bca1c5211e4149cd21a0f68fa4` |

Local release artifact:

| Artifact | SHA-256 |
| --- | --- |
| `/Users/kazuyoshitoshiya/ff/npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.15-release-artifacts.tar.gz` | `sha256:027fa2b6571bda37e2f2702c7fccac046bf39693f80e222847d62b17252dbd82` |

The release artifact includes `npa-package.toml`, public and vendored
certificate bytes, `generated/package-lock.json`, `generated/axiom-report.json`,
`generated/theorem-index.json`, and `generated/publish-plan.json`.
