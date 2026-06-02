# npa-mathlib Public Release Plan

This plan follows the CLR-10 registry readiness decision: continue
Git-release-based registry seed work while preparing a public theorem-library
package. It does not implement a registry server.

## Goal

Publish NPA with a substantial theorem library while preserving the
certificate-first trust boundary.

Target repository and package split:

```text
npa
  kernel, certificate format, checker, frontend, tactic, package CLI

npa-std
  small stable standard library

npa-mathlib
  public theorem library
```

The current `fixtures/npa-mathlib-seed` package is a dogfood seed. It proves
that a theorem package can be checked, released as deterministic artifacts, and
consumed downstream without a registry service. It is not the final public
`npa-mathlib` package.

## Non-Goals

- Do not build a registry server in this plan.
- Do not add network lookup, latest-version resolution, or package cache
  fallback to the kernel, certificate verifier, or checker.
- Do not treat source files, replay files, theorem indexes, publish metadata,
  CI status, or registry metadata as proof evidence.
- Do not require CLR-08 high-trust evidence for the first public
  reference-checker-only release.

## Public Package Boundary

`npa-mathlib` should be a separate repository once the local fixture is stable.
Inside this repository, use a fixture first:

```text
fixtures/npa-mathlib/
  npa-package.toml
  Mathlib/
  generated/
```

Keep `fixtures/npa-mathlib-seed/` as historical CLR-09 dogfood evidence. The
new `fixtures/npa-mathlib/` should model the public package layout and naming.

Standalone repository activation is specified in
`develop/npa-standalone-repo-activation.md`. That procedure must be followed before
public `npa-mathlib` Layer 0 artifacts are treated as an external repository
release.

## Namespace Decision

The current seed uses `Proofs.Ai.*` because it was copied from the proof corpus.
That namespace is acceptable for seed evidence but not ideal for a public
theorem library.

The standalone `npa-mathlib` repository now carries the public namespace policy
in `docs/namespace-policy.md`. That policy is the source of truth for public
module naming after repository activation.

Fixed Layer 0 public namespace shape:

```text
Mathlib.Logic.Basic
Mathlib.Logic.Prop
Mathlib.Logic.Eq
Mathlib.Data.Nat.Basic
Mathlib.Core.Reduction
```

Provisional examples for later layers must be checked against that policy before
being published:

```text
Mathlib.Algebra.Ring
Mathlib.Algebra.Square
Mathlib.Algebra.OrderedField
Mathlib.Vector.Basic
Mathlib.Geometry.Pythagorean
```

`Std.*` modules remain in `npa-std`. `Mathlib.*` modules may import `Std.*` via
hash-pinned package imports, never by hidden local path or module name alone.

The namespace must be fixed before public-facing artifact hashes are treated as
stable, because module names affect certificate identity, export hashes,
package locks, axiom reports, theorem indexes, publish plans, and downstream
fixtures.

## Initial Release Layers

Release layers should be closed over package imports and internal module
dependencies.

Layer 0 mapping is fixed for the first public `npa-mathlib` fixture:

| Source corpus module | Public module | Public path |
| --- | --- | --- |
| `Proofs.Ai.Basic` | `Mathlib.Logic.Basic` | `Mathlib/Logic/Basic/` |
| `Proofs.Ai.Prop` | `Mathlib.Logic.Prop` | `Mathlib/Logic/Prop/` |
| `Proofs.Ai.Eq` | `Mathlib.Logic.Eq` | `Mathlib/Logic/Eq/` |
| `Proofs.Ai.Nat` | `Mathlib.Data.Nat.Basic` | `Mathlib/Data/Nat/Basic/` |
| `Proofs.Ai.Reduction` | `Mathlib.Core.Reduction` | `Mathlib/Core/Reduction/` |

Layer 0 acceptance criteria:

- `fixtures/npa-mathlib/npa-package.toml` uses package `npa-mathlib`.
- The five public modules above are the only local modules in the first
  `fixtures/npa-mathlib/` package.
- Public module paths match the table exactly.
- `npa-std` imports remain hash-pinned package imports for `Std.Logic.Eq` and
  `Std.Nat.Basic`.
- Custom package-local axioms remain disabled and the Layer 0 axiom report has
  zero direct axioms, zero transitive axioms, and zero policy violations.
- The downstream fixture imports `npa-mathlib` modules by package, version,
  module, export hash, certificate hash, certificate file hash, and artifact
  path, never by module name alone.
- `fixtures/npa-mathlib-seed/` remains unchanged as CLR-09 evidence.

Layer 0 fixture status:

- `fixtures/npa-mathlib/` now models package `npa-mathlib` with the five public
  modules in the mapping table.
- `fixtures/npa-mathlib/generated/package-lock.json`,
  `generated/axiom-report.json`, `generated/theorem-index.json`, and
  `generated/publish-plan.json` are regenerated for the `Mathlib.*`
  namespace.
- `fixtures/npa-mathlib-downstream/` imports `Mathlib.Logic.Basic` from package
  `npa-mathlib` through a vendored source-free certificate at
  `vendor/npa-mathlib/Mathlib/Logic/Basic/certificate.npcert`.
- `crates/npa-cli/tests/package_import_fixture.rs` checks source-free public
  downstream import, publish-plan import-bundle consistency, artifact hash
  consistency, and corrupted package/hash pin rejection.
- Public release `v0.1.0` has been published at
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.0`.
- The `v0.1.0` release bundle hash is
  `d89dd2cb08ae21c20b9ca889285d9fcb50b1c133d40556e0601588a44e9632d9`.

Layer 1 and later mappings are provisional until the Layer 0 release bundle,
downstream source-free smoke, and post-activation evidence are fixed:

Layer 1, small algebra/order:

- `Proofs.Ai.Algebra.Ring`
- `Proofs.Ai.Algebra.Square`
- `Proofs.Ai.OrderedField`

Layer 2, vector and geometry:

- vector basics and inner-product modules
- right-triangle, metric, and Pythagorean modules

Layer 3, algebraic structures and isomorphism routes:

- group, subgroup, quotient, image, and isomorphism modules

Layer 4, analysis and functional analysis:

- metric topology, normed spaces, derivatives, inverse/implicit function
  routes, spectral theorem routes

## Artifact Stabilization Workflow

For each layer:

1. Copy the selected closed module set into `fixtures/npa-mathlib/`.
2. Rename module declarations and paths to the public `Mathlib.*` namespace.
3. Declare `npa-std` imports in `npa-package.toml` with package, version,
   module, export hash, certificate hash, and certificate artifact path.
4. Regenerate certificates for renamed modules.
5. Regenerate `generated/package-lock.json`.
6. Regenerate `generated/axiom-report.json`.
7. Regenerate `generated/theorem-index.json`.
8. Regenerate `generated/publish-plan.json`.
9. Update or create a downstream fixture that imports public `npa-mathlib`
   artifacts through the publish-plan downstream import bundle.
10. Run source-free verification and artifact freshness checks.

The expected baseline commands are:

```sh
cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-downstream --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib-downstream --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib-downstream --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib-downstream --json
cargo test -q -p npa-cli package_import_fixture
```

## Axiom Policy

Prefer `allow_custom_axioms = false` for public release layers.

If a layer requires a known builtin axiom interface such as `Eq.rec`, keep that
layer separate until the public axiom policy and axiom report are reviewed. Any
allowed axiom must appear in `npa-package.toml`, `generated/axiom-report.json`,
and release documentation.

## Downstream Fixture Requirements

The downstream fixture must prove that a consumer can import `npa-mathlib`
without a registry server.

It must pin:

- package name
- package version
- module name
- export hash
- certificate hash
- certificate file hash
- certificate artifact path

It must reject corrupted package name, package version, export hash,
certificate hash, and certificate artifact hash.

## Release Evidence

The first public release may be reference-checker-only. It must include:

- `npa-package.toml`
- local certificate artifacts
- vendored or referenced `npa-std` certificate artifacts
- `generated/package-lock.json`
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `generated/publish-plan.json`
- package command JSON diagnostics
- downstream fixture evidence

`verified_high_trust` remains unavailable until the public repository supplies
CLR-08 pinned external checker artifacts, runner policies, checker registry
data, and release audit evidence.

Evidence fixed on 2026-06-02:

- `npa-mathlib v0.1.0` is published as a public GitHub Release.
- The release artifact bundle contains only the required package manifest,
  generated package artifacts, local `Mathlib.*` certificate artifacts, and
  vendored `npa-std` certificate artifacts.
- The release notes explicitly keep source, replay, theorem index, publish
  metadata, package manifest, CI status, Git tags, and release pages outside
  proof evidence.
- A downstream source-free smoke materialized from the published release bundle
  passed `check`, `build-certs --check`, `verify-certs --checker reference`,
  and `check-hashes`.
- Negative checks rejected corrupted import package name, package version,
  export hash, certificate hash, and certificate artifact data before proof
  acceptance.

## Immediate Tasks

1. Record SRA-09 post-activation evidence now that the downstream smoke has
   passed.
2. Add the next closed theorem layer only after SRA-09 fixes the complete
   activation evidence.
3. Keep CLR-08 high-trust release evidence separate from the reference-checker
   public Layer 0 fixture.
