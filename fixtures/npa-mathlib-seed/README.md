# npa-mathlib-seed Scope

This fixture records the CLR-09-01 boundary and initial module-scope decision
for the external `npa-mathlib-seed` theorem library dogfood.

## Boundary Decision

The first implementation target is an inert local fixture at
`fixtures/npa-mathlib-seed/`. It models a repository that will later be copied
or moved to a separate `npa-mathlib-seed` checkout.

This fixture is intentionally not wired into this repository's active CI:

- it has no active workflow under this repository's `.github/workflows/`;
- it does not depend on hidden relative paths into this `npa` checkout;
- package files use package-relative paths from the fixture root;
- standard-library inputs are declared as `npa-std` package artifacts, not
  loaded implicitly from this repository's `proofs/` tree.

The fixture can be copied to a standalone checkout as the package root. The
only expected external input is an `npa` CLI built from a compatible `npa`
checkout or release.

## Package Layout

The materialized seed package contains:

```text
npa-package.toml
README.md
CONTRIBUTING.md
Proofs/Ai/{Basic,Prop,Eq,Nat,Reduction}/source.npa
Proofs/Ai/{Basic,Prop,Eq,Nat,Reduction}/certificate.npcert
Proofs/Ai/{Basic,Prop,Eq,Nat,Reduction}/replay.json
Proofs/Ai/{Basic,Prop,Eq,Nat,Reduction}/meta.json
vendor/npa-std/Std/Logic/Eq/certificate.npcert
vendor/npa-std/Std/Nat/Basic/certificate.npcert
```

Generated package artifacts use the CLR-04 through CLR-06 conventional paths
under `generated/`:

```text
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
generated/publish-plan.json
```

CLR-09-03 checks in these generated files so a fresh checkout can run the base
package command sequence in check mode. They are still untrusted generated
metadata, not proof evidence.

## Initial Module Set

The seed starts with this closed theorem module subset copied from the current
proof corpus:

| Module | Current corpus imports | Current corpus axioms |
| --- | --- | --- |
| `Proofs.Ai.Basic` | none | none |
| `Proofs.Ai.Prop` | none | none |
| `Proofs.Ai.Eq` | `Std.Logic.Eq`, `Std.Nat.Basic` | none |
| `Proofs.Ai.Nat` | `Std.Logic.Eq`, `Std.Nat.Basic` | none |
| `Proofs.Ai.Reduction` | `Std.Nat.Basic` | none |

The subset is closed because no selected module imports another proof-corpus
module outside the set. Its only imports are standard-library artifacts:

- `Std.Logic.Eq` from package `npa-std` version `0.1.0`;
- `Std.Nat.Basic` from package `npa-std` version `0.1.0`.

The first seed keeps the current `Proofs.Ai.*` module names. A public namespace
rename is deferred until package manifests, certificates, import hashes, theorem
indexes, publish-plan data, and downstream fixtures are stable.

## Axiom Policy

The selected modules have no package-local axioms in the current corpus
metadata. The seed scope therefore permits no custom package-local axioms:

```text
allow_custom_axioms = false
allowed axioms for selected seed modules = []
```

`Eq.rec` is not part of the base seed axiom policy. Modules that require
`Eq.rec`, such as `Proofs.Ai.EqReasoning`, are deferred until a later seed
extension intentionally updates the axiom policy and regenerated artifacts.

## Deferred Corpus Modules

All proof-corpus modules outside the five selected modules are deferred from the
first `npa-mathlib-seed` dogfood. This includes `Proofs.Ai.EqReasoning` and the
larger algebra, vector, geometry, analysis, linear-algebra,
functional-analysis, and additional logic modules.

They are deferred because the first dogfood is testing package ergonomics,
source-free checking, deterministic metadata, and downstream import handoff
rather than bulk corpus migration. Adding larger modules would expand the
dependency graph, artifact churn, and axiom-policy review surface before the
external package boundary has been proven.

## Trusted Boundary

This scope decision requires no kernel, checker, or certificate-format change.
The trusted boundary remains the canonical certificate, the small Rust kernel,
and the source-free checker verdict. Package metadata, fixture documentation,
replay files, theorem indexes, publish-plan data, CI, and future registry seed
entries remain untrusted metadata.

## Local Check

From this fixture root, the CLR-09-03 validation sequence is:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

When running from the parent `npa` workspace before installing `npa`, replace
`npa` with `cargo run -p npa-cli --`, for example:

```sh
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json
```

## CI

The seed fixture includes inactive-in-this-repository workflow files under
`.github/workflows/`. They become active only after this fixture is copied to a
standalone `npa-mathlib-seed` repository.

The workflows locate `npa` deterministically. Repository variables may set
`NPA_BINARY_PATH` to an executable package-local or absolute binary path, or
`NPA_GIT_COMMIT` to a full 40-hex `npa` commit SHA. If neither is set, the
checked-in workflow default builds `npa-cli` from the pinned commit recorded in
the workflow. Git-tag and release-version installer modes are intentionally
reserved until their download strategy is fixed. Rust builds use an exact
`RUST_TOOLCHAIN_VERSION`, defaulting to `1.95.0`.

PR CI runs the full package check sequence, including source-free reference
verification and `generated/publish-plan.json` check. Release CI uploads:

- `generated/package-lock.json`
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `generated/publish-plan.json`
- checked certificate artifacts
- package command JSON diagnostics under `ci-output/`

The base release profile is reference-checker-only for proof acceptance. It
also records a labeled fast-kernel source-free verification job when the
checked-in `npa` CLI supports it. The workflows do not run `--checker external`,
do not generate `verified_high_trust`, and do not use registry lookup, package
solver, network package resolution, or implicit latest-version package imports.
