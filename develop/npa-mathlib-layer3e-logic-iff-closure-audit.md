# npa-mathlib Layer 3E Logic Iff Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the Layer 3D-G
correspondence materialization. It selects the small propositional-logic
`Iff` route as the next public closure and keeps the ring, CRT, ordered
algebra, geometry, and analysis routes out of this layer.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- Layer 3D-G has been materialized, committed, tagged, and pushed in
  `../npa-mathlib` as `npa-mathlib v0.1.13`.
- The `npa-mathlib` release commit is
  `b2ee22f9f526ae80fea7b8c9dea58888c3479ed5`.
- The annotated `v0.1.13` tag object observed locally is
  `a361a448f3ee7dec81d6124b589ae93bd7e85ad7`.
- The local ignored `v0.1.13` release-bundle tar hash is
  `sha256:88d1ef15907dc65f19c175cb2eabd69168355c8f236218f0e6b498e11737e0b9`.

Layer 3E should materialize as `npa-mathlib v0.1.14`. It must not change
package boundaries, registry assumptions, import identity rules, or proof trust
boundaries.

The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

## Selected Candidate Set

The selected Layer 3E candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Logic.Iff` | `Mathlib.Logic.Iff` | `Mathlib/Logic/Iff/` | 5 definitions, 16 theorems | `Std.Logic.Eq` | direct and transitive `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public logic module |
| --- | --- |
| `Proofs.Ai.Logic.Iff` | `Mathlib.Logic.Iff` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Logic.Iff
  imports Std.Logic.Eq
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. No
already released `Mathlib.*` module is required for this closure.

The selected module introduces the following public surface:

- definitions:
  - `Iff`
  - `And`
  - `Or`
  - `False`
  - `Not`
- theorems:
  - `iff_refl`
  - `iff_symm`
  - `iff_trans`
  - `iff_mp`
  - `iff_mpr`
  - `and_intro`
  - `and_left`
  - `and_right`
  - `iff_of_eq`
  - `false_elim`
  - `not_intro`
  - `not_elim`
  - `or_inl`
  - `or_inr`
  - `or_elim`
  - `iff_congr_arg`

## Namespace Decision

`And`, `Or`, `False`, and `Not` are materialized together with `Iff` in
`Mathlib.Logic.Iff` for this release. The reason is pragmatic: the corpus
module is closed, small, and standalone; splitting the connectives into a new
`Mathlib.Logic.Connectives` module would require editing the source namespace
without adding proof value, and would make the first public propositional
connective layer larger than needed.

The broader `Mathlib.Logic.Connectives` split remains deferred. It should be
reconsidered only when another corpus route needs those connectives without
the `Iff` API.

## Import Rewrite Table

The only materialization rewrite is the package module name itself:

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Logic.Iff` | `Mathlib.Logic.Iff` | materialize as a local public module |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale `Proofs.Ai.Logic.Iff` reference.

## Deferred Candidates

The following nearby routes are intentionally not selected for Layer 3E:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Abstract ring foundation | `Proofs.Ai.Algebra.AbstractRing` | It needs a separate namespace decision because `Mathlib.Algebra.Ring` already exists as a concrete ring module while the corpus route exposes an abstract law-package API. |
| Ring first isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`, `Proofs.Ai.Algebra.AbstractRingFirstIso`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | These depend on the abstract ring foundation namespace and should be audited after it. |
| Ordered algebra and square normalization | `Proofs.Ai.Algebra.*`, `Proofs.Ai.OrderedField`, related square routes | These require a broader algebra-layer audit and do not belong to the propositional logic closure. |
| Geometry and analysis routes | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Analysis.*`, `Proofs.Ai.LinearAlgebra.*`, `Proofs.Ai.FunctionalAnalysis.*` | Their public namespaces and imports are broader than the `Iff` closure. |

## Axiom Policy

Layer 3E does not widen the public axiom policy beyond the `v0.1.13` baseline.
The selected module uses only the builtin equality eliminator surface already
allowed by the public package policy.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entry should declare:

```toml
axioms = ["Eq.rec"]
```

`Eq.rec` is direct in the source:

- `iff_of_eq` calls `@Eq.rec.{1,0}`.
- `iff_congr_arg` calls `@Eq.rec.{1,0}`.

The expected direct and transitive axiom status for `Mathlib.Logic.Iff` is:

- direct axioms: `Eq.rec`
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes for `Proofs.Ai.Logic.Iff`:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:4cfbee5247057d885bf4b6af1da7576a52672853f1c6250e9dd1a70fd168f0a6` |
| certificate file hash | `sha256:cc9af8dedec12fadef41aac2c5637fc986b04ecc566cb15c89aa9205e41e2251` |
| export hash | `sha256:d2daace47d7e3dd6b1c9833d6c5d1bfaebdca3284e04ec70c3e9cf74f4bfd147` |
| axiom report hash | `sha256:e857a3aea953927d44e330377c011e7fe1f0892d779d46d8eb9d4cc62236ade7` |
| certificate hash | `sha256:64c824049b33fe998e99bfdfa984cd1fa74acbea37ee53f1a0330661ddd14f4d` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. The public certificate/export hashes can differ from the
corpus hashes because the module name changes from `Proofs.Ai.Logic.Iff` to
`Mathlib.Logic.Iff`.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Logic.Iff
```

Observed result:

```text
verified Proofs.Ai.Logic.Iff
verified 1 selected module(s), 2 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Logic.Iff
```

Observed result:

```text
built Proofs.Ai.Logic.Iff
wrote proofs/generated/ai-theorem-index.json
built Proofs.Ai.Logic.Iff (1 module(s) including import closure)
```

The `--build-module` index side effect produced no `npa` worktree diff.

## Materialization Plan

Materialization should create `npa-mathlib v0.1.14` with:

- public module: `Mathlib.Logic.Iff`
- public path: `Mathlib/Logic/Iff/`
- copied sidecars: `source.npa`, `replay.json`, `meta.json`
- generated certificate: `Mathlib/Logic/Iff/certificate.npcert`
- downstream smoke fixture consuming vendored certificate bytes, not source
  files

The downstream smoke fixture should consume at least:

- `iff_mp`
- `or_elim`
- `false_elim`

Planned downstream smoke theorem names:

- `iff_mp_passthrough`
- `or_elim_passthrough`
- `false_elim_passthrough`

## Positive Gates

Run these gates in `../npa-mathlib` after
materialization:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
```

Run these downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
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

Materialization succeeded.

Public module:

| Public module | Public path |
| --- | --- |
| `Mathlib.Logic.Iff` | `Mathlib/Logic/Iff/` |

Public hashes for `Mathlib.Logic.Iff`:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:4cfbee5247057d885bf4b6af1da7576a52672853f1c6250e9dd1a70fd168f0a6` |
| certificate file hash | `sha256:5db35953be54b9278c60ca721f5ee4a3a22e71c8087f1d3898e23b70f964bd8f` |
| export hash | `sha256:e17dc3a48900d70ad426461379d429257631555845b50b6413c20d80ca0626a9` |
| axiom report hash | `sha256:e857a3aea953927d44e330377c011e7fe1f0892d779d46d8eb9d4cc62236ade7` |
| certificate hash | `sha256:0d6359b2b0f58d8a99c86f2ea0fd579c3d99b62391712f9b47239039ae2cf919` |

Downstream smoke module:

| Downstream module | Public imports | Theorems |
| --- | --- | --- |
| `Downstream.LogicIff` | `Std.Logic.Eq`, `Mathlib.Logic.Iff` | `iff_mp_passthrough`, `or_elim_passthrough`, `false_elim_passthrough` |

Downstream smoke hashes for `Downstream.LogicIff`:

| Hash kind | Value |
| --- | --- |
| source hash | `sha256:75303868b2d495f9203b5130c8e2bec95022aa27393ac262b5dbbb1eaf448784` |
| certificate file hash | `sha256:95359d0a8826c83896713dbdab8f294628c9fe0abaeb8e4035752a9e831ae3dd` |
| export hash | `sha256:7e35ee61d3a10f9a5c1242ca4e70e8e06dfae7f9ac460e22ae9f940f3bbad166` |
| axiom report hash | `sha256:0544a7cbc8ae41c11f909366f72b2c80b48b4429ded309accea7370850df08bb` |
| certificate hash | `sha256:a72834f6336e24ee8ed133d5c7be28874435b957f3ea33199b2a2e1a44eabd7f` |

Generated artifact hashes:

| Artifact | SHA-256 |
| --- | --- |
| `generated/package-lock.json` | `sha256:75213a4fe528a8ea82cc9b0210e59406a5a9cbf32ca37aae8370c79415a98c5a` |
| `generated/publish-plan.json` | `sha256:04c04fafcd8aab9107ba1f4cf6672c6e811ac959c0e5db65535e734cf997eca3` |
| `fixtures/downstream-smoke/generated/package-lock.json` | `sha256:e81f6c8072ce119159c61192bee7aa0729b741375a405865e1edd4770d2386c3` |

Release artifact:

```text
../npa-mathlib/target/release-artifacts/npa-mathlib-v0.1.14-release-artifacts.tar.gz
sha256:eacfc65d24ee5b9b6c678c0ab9b134c64c11a357301fb18f1fe33f7fcf3a740e
```

Positive gates passed:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
```

The source-free reference verifier reported:

```text
mode=reference;verdict_source=npa-checker-ref;reference_checker_verdict=true;modules=39
```

Downstream smoke gates passed:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
```

The downstream source-free reference verifier reported:

```text
mode=reference;verdict_source=npa-checker-ref;reference_checker_verdict=true;modules=3
```

Negative package-copy checks were run in
`/tmp/npa-mathlib-logic-iff-neg.NKbPl3`, outside both repositories. The
temporary tree was removed after the checks.

| Check | Command surface | Observed reason code |
| --- | --- | --- |
| bad public export hash | `package build-certs --check` | `export_hash_mismatch` |
| bad public certificate hash | `package build-certs --check` | `certificate_hash_mismatch` |
| corrupted public certificate bytes | `package verify-certs --checker reference` after matching the corrupted file hash in the temp manifest | `certificate_decode_failed` |
| stale downstream package-version pin | `package check-hashes` on the downstream fixture | `package_lock_stale` |
