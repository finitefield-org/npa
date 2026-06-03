# npa-mathlib Layer 3A Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.3` Layer 2B concrete geometry release. It is an input to
materialization in the standalone `finitefield-org/npa-mathlib` repository; it
does not publish new artifacts by itself.

## Baseline

Current public package state:

- `npa-mathlib v0.1.3` is published from standalone repository commit
  `dd5283666592ac9a15def166d0f7f11b197449f8`.
- The release bundle hash is
  `07e5cdf2ebb6e139fbe0473b6bc39ed3dbf1a151f930602`.
- Layer 2B public modules are:
  - `Mathlib.Geometry.RightTriangle`
  - `Mathlib.Geometry.Metric`
- Layer 2A public modules remain:
  - `Mathlib.Vector.Basic`
  - `Mathlib.Vector.Dot`
- Layer 1 public modules remain:
  - `Mathlib.Algebra.Ring`
  - `Mathlib.Algebra.Square`
  - `Mathlib.Algebra.OrderedField`
- Layer 0 public modules remain:
  - `Mathlib.Logic.Basic`
  - `Mathlib.Logic.Prop`
  - `Mathlib.Logic.Eq`
  - `Mathlib.Data.Nat.Basic`
  - `Mathlib.Core.Reduction`
- The standalone namespace policy is
  `../npa-mathlib/docs/namespace-policy.md`.

Layer 3A must add only the abstract group foundation needed before subgroup,
kernel, image, quotient, and isomorphism routes. It must not change the package
split, registry assumptions, or proof trust boundary.

## Selected Candidate Set

The Layer 3A candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | `Mathlib/Logic/EqReasoning/` | 11 theorems | `Std.Logic.Eq` | `Eq.rec` |
| `Proofs.Ai.Algebra.AbstractGroup` | `Mathlib.Algebra.Group.Basic` | `Mathlib/Algebra/Group/Basic/` | 4 definitions, 23 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning` | `Eq.rec` |

The requested algebra namespace mapping is therefore:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroup` | `Mathlib.Algebra.Group.Basic` |

`Proofs.Ai.EqReasoning` is not an algebra module. It is a reusable equality
reasoning support module, so it belongs under `Mathlib.Logic.*`.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Logic.EqReasoning
  imports Std.Logic.Eq

Mathlib.Algebra.Group.Basic
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. Existing
Layer 0, Layer 1, Layer 2A, and Layer 2B modules remain local public
`npa-mathlib` modules.

The selected set does not depend on:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`
- `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage`
- `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroup*Iso*`
- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`

## Axiom Policy

Layer 3A is the first selected public `npa-mathlib` layer with a non-empty
axiom surface. Both selected modules use the builtin equality eliminator
`Eq.rec`.

Materialization must not enable arbitrary custom axioms. The expected manifest
policy for `npa-mathlib v0.1.4` is:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The two new local module entries must declare:

```toml
axioms = ["Eq.rec"]
```

The regenerated package axiom report must show `Eq.rec` as allowed and must
show zero policy violations. `Eq.rec` is builtin equality eliminator surface;
it is not `sorry` and it is not evidence from source, replay, theorem index,
publish plan, CI, Git tag, or registry metadata.

If the public package is not ready to allow `Eq.rec`, stop before
materialization and run a separate public axiom-policy review. Do not hide the
axiom surface by omitting `EqReasoning` or by relying on transitive imports
without manifest policy.

## Deferred Candidates

The following nearby modules are verified in the proof corpus, but they are not
part of this Layer 3A release candidate.

| Corpus module | Declarations | Direct imports | Axioms | Verification result | Build result | Reason deferred |
| --- | --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | 5 definitions, 28 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.EqReasoning` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Starts subgroup, normal subgroup, intersection, and product predicate surface; release separately after base group API is stable. |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Kernel route should be grouped with image/quotient/isomorphism decisions, not the foundation layer. |
| `Proofs.Ai.Algebra.AbstractGroupImage` | 1 definition, 5 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Image route should be grouped with kernel/quotient/isomorphism decisions, not the foundation layer. |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | 4 definitions, 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Introduces quotient/setoid-facing public surface; keep it in a separate quotient audit. |

Layer 3B should audit subgroup and normal subgroup modules. A later
isomorphism route should audit kernel, image, quotient, and first/second/third
isomorphism modules together.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.EqReasoning` | `sha256:3eb1f94054f46fe74c2b7294f50ff6f0a5700e53a613f6b48f8a6d0f379a34b4` | `sha256:cd4c2338a26f0bd259103706e056bb05320777b1f473ce9263fda4ad94f86682` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5283e4bbd120c3ffa60356b600be06364c3739f9c1992538f75aa4c7df947968` | `sha256:1a146be8c2aee52e4e19e44c84357bbb40bf6f649efcc78f8f8174213abfab8e` |
| `Proofs.Ai.Algebra.AbstractGroup` | `sha256:ae2f646d49c1e45f7ffd343f15968a4eba5be179b7ca4c0da7386ceb12615890` | `sha256:c2822a14b5ed3683d270130dd04e94f12944a44bb08483ca641c5e0de1bfe863` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:63c3ca0596e94ceb5c525266931264176f2096a320083864a9662bbc9db78269` | `sha256:07d04f5fa969484c0fd1c2fe9fe08cfad7d07c3de93950a54a6beaed4850b0c6` |

For comparison, the nearby deferred modules currently have these proof-corpus
hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `sha256:45c33f367e505194a12eb206376c0881a9cf98a1858d2cc550f4ec5577993aef` | `sha256:890d6d64962651eb0bd5ee735a91d2ffcc3826cc97a338b31c4b916c2e9bdcde` | `sha256:c2ae287a3e5e0f4de41b6a201d345af9f83aa200f20dbdcf487d15273ca5f3b4` | `sha256:1298b31aa5ef802d3af57bb01d8cc3f3a11b162f4f51bbe1fd8e650e7f036612` | `sha256:0d356dd1f768b8665f5cf9b7d4a75ea1a34422181907125fbf167263e4b8092d` |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | `sha256:8d11dd74da047650e74282aeea0d7a0ac2aacfc5cb3bae1bf85c12bf5c38fadb` | `sha256:6d4390bbcf7d5b22c92d24b144575cbc3f870971e03574e43633bd9a64068144` | `sha256:5c1a39b0e9f82dc1014e3e63549c71eab3db6383024a87408e4e95e06bc33fbb` | `sha256:39f92e3a74edf201d900d93abeebb9c404ea8c141bfc46761f7e889c9f8cf9f7` | `sha256:1d5b2a02b6cc7d9dfafbb781f3d27b9494e7655644aa046871bbb7abb9170bd8` |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `sha256:8c6f85eb72c6d04a2b5b97448d7aa7f7b331250591be2afe3ce329ed6a171080` | `sha256:f5e396c6b08319835864296ee0fbb273dbfeda738652f4f44767e05cf57023cc` | `sha256:0935f963b77bb9ed124d4b38435ad2a9a19f860e1f06de11c1ab8ca3f05cdd0e` | `sha256:0869976f857fbf07454df1db004a649ae3ee9ede0e1429dbf40af6dfaac5bfeb` | `sha256:af05a4354e4be42ad76b6fb1da6a149d34dfe5207637fdff9b7ff246076935a1` |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `sha256:a72a0498b88ed9cf0487d309a710ff021647e01ed9b04fe7663caa2cb7dd88e7` | `sha256:c64b8020f5bc26953e4ddcd4e003bb8a5421a326b6662f1b44ce6a1060d55748` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:30155dfd0399b8bb9222cee6aff0c26065a6766282a10f97ef2d27d45d89aa6a` | `sha256:263097d5b8ce78ffd3ce21a3a74d8d4598cf2bc18da274a99431e64b6e3739d2` |

## Verification

The checked-in corpus certificates for the selected modules passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.EqReasoning
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroup
```

Results:

- `Proofs.Ai.EqReasoning`: verified 1 selected module, 2 modules including
  dependency cache.
- `Proofs.Ai.Algebra.AbstractGroup`: verified 1 selected module, 3 modules
  including dependency cache.

The source-to-certificate authoring path also regenerated the same closure:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.EqReasoning
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroup
```

Results:

- `Proofs.Ai.EqReasoning`: built 1 local module including import closure.
- `Proofs.Ai.Algebra.AbstractGroup`: built 2 local modules including import
  closure: `Proofs.Ai.EqReasoning` and
  `Proofs.Ai.Algebra.AbstractGroup`.

The deferred comparison modules also passed source-free verification and
source-to-certificate builds:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSubgroup
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSubgroup
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupImage
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupImage
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotient
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotient
```

Results:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`: verified 1 selected module,
  4 modules including dependency cache; built 3 local modules including import
  closure.
- `Proofs.Ai.Algebra.AbstractGroupKernel`: verified 1 selected module,
  4 modules including dependency cache; built 3 local modules including import
  closure.
- `Proofs.Ai.Algebra.AbstractGroupImage`: verified 1 selected module,
  4 modules including dependency cache; built 3 local modules including import
  closure.
- `Proofs.Ai.Algebra.AbstractGroupQuotient`: verified 1 selected module,
  4 modules including dependency cache; built 3 local modules including import
  closure.

The difference between source-free verification counts and build counts is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

## Readiness Decision

Layer 3A is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.4` with exactly these new public modules:

```text
Mathlib.Logic.EqReasoning
Mathlib.Algebra.Group.Basic
```

Do not include subgroup, normal subgroup, kernel, image, quotient, or
isomorphism modules in the same release. They are verified, but they should be
released only after separate closure audits because they widen the public API
surface beyond the group foundation.

Materialization must not copy old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.3`; provisionally this is
`v0.1.4`.

## Materialization Steps

These steps were completed in `../npa-mathlib`:

1. Completed: added `Mathlib/Logic/EqReasoning/` and
   `Mathlib/Algebra/Group/Basic/` from the selected corpus sources.
2. Completed: renamed module-local imports from `Proofs.Ai.*` to `Mathlib.*`.
3. Completed: kept the existing `npa-std v0.1.0` hash-pinned imports for
   `Std.Logic.Eq` and `Std.Nat.Basic`.
4. Completed: kept the released Layer 0, Layer 1, Layer 2A, and Layer 2B
   modules local in `npa-mathlib`.
5. Completed: added manifest entries for the two new modules and bumped the
   package version to `0.1.4`.
6. Completed: updated `docs/namespace-policy.md` in the standalone
   `npa-mathlib` repository with the Layer 3A released module list and
   mapping.
7. Completed: set package policy to `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`; set `axioms = ["Eq.rec"]` on both new module
   entries.
8. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
9. Completed: updated the downstream smoke fixture so it imports
   `Mathlib.Algebra.Group.Basic` through a package import bundle.
10. Completed: ran package gates for `npa-mathlib` and downstream smoke.
11. Completed: published `npa-mathlib v0.1.4` after release bundle, axiom
    report, and downstream smoke evidence were fixed.

Do not start Layer 3B subgroup/normal-subgroup materialization, isomorphism
route materialization, Pythagorean materialization, or CLR-08 high-trust
evidence work as part of this Layer 3A materialization. They remain separate
release tracks.

## Release Outcome

Layer 3A was materialized and published as `npa-mathlib v0.1.4` on
2026-06-03.

Release identifiers:

- release: `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.4`
- tag object: `88bc4cc1addae12f1babd45eeee4caa2a302a932`
- target commit: `e775afff5b9a2abe7709d7d66afe333c37cab955`
- release bundle:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.4/npa-mathlib-v0.1.4-release-artifacts.tar.gz`
- release bundle SHA-256:
  `d216da5522a5d4cd5e37ae059387b93632a0d04aa6ea6f9b8e757c256789ee4c`

Public module hashes after namespace materialization:

| Public module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Logic.EqReasoning` | `sha256:676d15f72088b4f107773ad566920c02a5e29d8773a4c5c24ccb0a1b19de930f` | `sha256:d864825b1b4618bb7d04c541c74ef3d4dc78076abd458df79bc22d648ae0b23a` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5283e4bbd120c3ffa60356b600be06364c3739f9c1992538f75aa4c7df947968` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:566c57ad04412e8e71ec892f2817c008ce63e0966b31414dec8e5d31e5cc1ee5` | `sha256:c6407b46a7421a1a56c16270a481fa9ab4d93249ae0bfe0bbec098e7af6380f0` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:63c3ca0596e94ceb5c525266931264176f2096a320083864a9662bbc9db78269` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |

The regenerated package axiom report records:

- `allow_custom_axioms = false`
- `allowed_axioms = ["Eq.rec"]`
- direct axiom count: 1
- transitive axiom count: 2
- policy violation count: 0
- package axiom report hash:
  `sha256:57994c334195fa396314537155ade05753b9838d97947aa056685536586ea4ea`

The standalone package gates passed locally:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

The downstream smoke fixture imported only release-bundle certificate bytes for
`Std.Logic.Eq`, `Mathlib.Logic.EqReasoning`, and
`Mathlib.Algebra.Group.Basic`, then passed:

```sh
npa package check --root fixtures/downstream-smoke --json
npa package build-certs --root fixtures/downstream-smoke --check --json
npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
npa package check-hashes --root fixtures/downstream-smoke --json
```

GitHub Actions status is intentionally not release evidence for `v0.1.4`.
This release remains reference-checker-only and does not emit
`verified_high_trust`.
