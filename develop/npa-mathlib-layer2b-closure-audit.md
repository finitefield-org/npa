# npa-mathlib Layer 2B Closure Audit

Date: 2026-06-02

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.2` Layer 2A vector release. It served as the materialization input for
the standalone `finitefield-org/npa-mathlib` repository; the release outcome is
recorded below.

## Baseline

Current public package state:

- `npa-mathlib v0.1.2` is published from standalone repository commit
  `4c28e82d3dc2e0a8a25bb2e01bb433c7a10a28fe`.
- The release bundle hash is
  `7b1d8fe69b0bca46e77149453e79ece8198473ce9e760d90e9f8e2c66b117d68`.
- Layer 2A public modules are:
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

Layer 2B must add a small concrete geometry layer without changing the package
split, registry assumptions, axiom policy, or proof trust boundary.

## Selected Candidate Set

The Layer 2B candidate set was closed and small enough for the `v0.1.3`
materialization:

| Corpus module | Public module | Public path | Declarations | Direct imports |
| --- | --- | --- | --- | --- |
| `Proofs.Ai.Geometry.RightTriangle` | `Mathlib.Geometry.RightTriangle` | `Mathlib/Geometry/RightTriangle/` | 2 definitions, 13 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`, `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic`, `Proofs.Ai.Vector.Dot` |
| `Proofs.Ai.Geometry.Metric` | `Mathlib.Geometry.Metric` | `Mathlib/Geometry/Metric/` | 1 definition, 8 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`, `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic`, `Proofs.Ai.Vector.Dot`, `Proofs.Ai.Geometry.RightTriangle` |

After public namespace materialization, the internal imports became:

```text
Mathlib.Geometry.RightTriangle
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring
  imports Mathlib.Algebra.Square
  imports Mathlib.Algebra.OrderedField
  imports Mathlib.Vector.Basic
  imports Mathlib.Vector.Dot

Mathlib.Geometry.Metric
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring
  imports Mathlib.Algebra.Square
  imports Mathlib.Algebra.OrderedField
  imports Mathlib.Vector.Basic
  imports Mathlib.Vector.Dot
  imports Mathlib.Geometry.RightTriangle
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
Layer 0, Layer 1, and Layer 2A modules are already local public `npa-mathlib`
modules as of `v0.1.2`.

The selected set does not depend on:

- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Affine*`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`
- `Proofs.Ai.Algebra.Abstract*`
- `Proofs.Ai.Vector.Abstract*`

## Deferred Candidate

`Proofs.Ai.Geometry.Pythagorean` is verified in the proof corpus, but it is not
part of this Layer 2B release candidate.

Reasons:

- It directly imports abstract/law-package modules:
  `Proofs.Ai.Algebra.AbstractRing`,
  `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractSquareNormalize`,
  `Proofs.Ai.Algebra.AbstractScalarDerive`,
  `Proofs.Ai.Vector.AbstractSpace`,
  `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`,
  `Proofs.Ai.Geometry.Affine`,
  `Proofs.Ai.Geometry.AffineDerive`,
  `Proofs.Ai.Geometry.AbstractRightTriangle`,
  `Proofs.Ai.Geometry.AbstractRightTriangleDerive`, and
  `Proofs.Ai.Geometry.AbstractMetric`.
- It also imports `Proofs.Ai.EqReasoning`.
- Its source-free verification touches 15 modules including dependency cache.
- Its source-to-certificate build touches 14 local modules including import
  closure.
- Its axiom surface includes `Eq.rec`, while the selected Layer 2B candidate
  set has no direct or transitive axiom usage in the package axiom report.

Defer `Proofs.Ai.Geometry.Pythagorean` to a later abstract geometry /
law-package release track with a separate closure audit and explicit axiom
policy review.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Geometry.RightTriangle` | `sha256:66f65900e8043a84d5ff9a0868064fbf65cb4eec5de0e92977b0572de95cd7d6` | `sha256:bd6aef6b1d2808476910bf48b9afac4bdfa4031b4e228b228d77b6c11ccb5d66` | `sha256:c2a17e11ac8f8085c75bf7ab04e8d7a56de33c783d08b57d3d4248edc3c6015c` | `sha256:3d3fdbf6a3ca4756ceaac9853e839a84878b24c0f6290e2246a78c6184b31e0e` | `sha256:42d33464c03e9214d92cc5f1cc3e271dcd4cc6a2fc1ec6d5eea6b27648953527` |
| `Proofs.Ai.Geometry.Metric` | `sha256:1d7b3ae332c7ec21f0ebd2b7999cca7de1fa7761683a19036c9c877196b618b0` | `sha256:3c1dd7afeee64d2cab83e7e6c8d917daa92a3e403428b7f4f48944345240bb64` | `sha256:296df8f58ddcead3a3d5726665d2b044873e33af070c2a3018392685733dd0d9` | `sha256:c94182a358767e0d7e1bb5c488e82696570a5f6d442295c34fe105cf99c73a36` | `sha256:4d00e23b5c71e5cd276c0b4620a4a2e354c7c1ecbbf365de745ef2dd23bf9709` |

For comparison, the deferred corpus module has these current proof-corpus
hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Geometry.Pythagorean` | `sha256:62f6f82a4144b9306998746cd6f24abf963e766c0bb21ce65c66cda6829517ba` | `sha256:3a8c95687768687719fafcd699924479f0c9bf1a94318e00f1a36b66bcd025f9` | `sha256:eb05a29d4cf6110896d4b1cc5473ca511e3c1e620c00a79f16bc39eca7afd49a` | `sha256:067fa89c58bfb9d8a1a11be14a0d6e47a783909b69a3bda57dd93232c06af24b` | `sha256:0771e015c0b8285d4a2a7740036eac0e13e417510fd8fdb5ddee8fc0cb0e7e6c` |

## Verification

The checked-in corpus certificates passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Geometry.RightTriangle
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Metric
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Pythagorean
```

Results:

- `Proofs.Ai.Geometry.RightTriangle`: verified 1 selected module, 7 modules
  including dependency cache.
- `Proofs.Ai.Geometry.Metric`: verified 1 selected module, 8 modules including
  dependency cache.
- `Proofs.Ai.Geometry.Pythagorean`: verified 1 selected module, 15 modules
  including dependency cache.

The source-to-certificate authoring path also regenerated the closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.RightTriangle
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Metric
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Pythagorean
```

Results:

- `Proofs.Ai.Geometry.RightTriangle`: built 6 local modules including import
  closure:
  `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`,
  `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic`,
  `Proofs.Ai.Vector.Dot`, and `Proofs.Ai.Geometry.RightTriangle`.
- `Proofs.Ai.Geometry.Metric`: built 7 local modules including import
  closure:
  `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`,
  `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic`,
  `Proofs.Ai.Vector.Dot`, `Proofs.Ai.Geometry.RightTriangle`, and
  `Proofs.Ai.Geometry.Metric`.
- `Proofs.Ai.Geometry.Pythagorean`: built 14 local modules including import
  closure:
  `Proofs.Ai.Algebra.AbstractRing`,
  `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.EqReasoning`,
  `Proofs.Ai.Algebra.AbstractScalarDerive`,
  `Proofs.Ai.Vector.AbstractSpace`,
  `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Geometry.Affine`,
  `Proofs.Ai.Geometry.AbstractRightTriangle`,
  `Proofs.Ai.Geometry.AffineDerive`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`,
  `Proofs.Ai.Geometry.AbstractMetric`,
  `Proofs.Ai.Geometry.AbstractRightTriangleDerive`, and
  `Proofs.Ai.Geometry.Pythagorean`.

The difference between the source-free verification counts and build counts is
the external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

## Readiness Decision

Layer 2B materialization selected exactly these new public modules for the
standalone `npa-mathlib` repository:

```text
Mathlib.Geometry.RightTriangle
Mathlib.Geometry.Metric
```

Do not include `Mathlib.Geometry.Pythagorean` in the same release. Its current
proof-corpus closure belongs to a later abstract geometry / law-package layer.

Materialization must not copy the old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

The materialized package/release version after `v0.1.2` is `v0.1.3`.

## Completed Materialization Steps

These steps were completed in `../npa-mathlib`:

1. Completed: added `Mathlib/Geometry/RightTriangle/` and
   `Mathlib/Geometry/Metric/` from the selected corpus sources.
2. Completed: renamed module-local imports from `Proofs.Ai.*` to `Mathlib.*`.
3. Completed: kept the existing `npa-std v0.1.0` hash-pinned imports for
   `Std.Logic.Eq` and `Std.Nat.Basic`.
4. Completed: kept the released Layer 0, Layer 1, and Layer 2A modules local in
   `npa-mathlib`.
5. Completed: added manifest entries for the two new modules and bumped the
   package version to `0.1.3`.
6. Completed: regenerated certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
7. Completed: updated the downstream smoke fixture so it imports
   `Mathlib.Geometry.Metric` through a package import bundle.
8. Completed: ran package gates for `npa-mathlib` and downstream smoke.
9. Completed: published the release after release bundle and downstream smoke
   evidence were fixed.

Do not start the deferred abstract Pythagorean materialization or CLR-08
high-trust evidence work as part of this Layer 2B materialization. Both remain
separate release tracks.

## Release Outcome

Layer 2B was materialized and published as `npa-mathlib v0.1.3` on
2026-06-02.

Release refs:

- release: `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.3`
- tag object: `689748138908401e0b9f9a1b58cce907e945f18b`
- target commit: `dd5283666592ac9a15def166d0f7f11b197449f8`
- release bundle:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.3/npa-mathlib-v0.1.3-release-artifacts.tar.gz`
- release bundle SHA-256:
  `07e5cdf2ebb6e139fbe0473b6bc4372f830182a7c5bc39ed3dbf1a151f930602`

Local release gates passed for the standalone repository:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Local downstream smoke gates passed:

```sh
npa package check --root fixtures/downstream-smoke --json
npa package build-certs --root fixtures/downstream-smoke --check --json
npa package verify-certs --root fixtures/downstream-smoke --checker reference --json
npa package check-hashes --root fixtures/downstream-smoke --json
```

The published release bundle was downloaded, checked against its SHA sidecar,
extracted, and source-free verified with `package check`,
`verify-certs --checker reference`, `axiom-report --check`, `index --check`,
and `publish-plan --check`. The root release bundle intentionally excludes
source files, so `package check-hashes` is not the source-free bundle gate.

A temporary downstream smoke then vendored only certificate bytes from the
downloaded release bundle and passed `check`, `build-certs --check`,
`verify-certs --checker reference`, and `check-hashes`.

GitHub Actions status is intentionally not release evidence for `v0.1.3`.
