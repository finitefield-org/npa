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

Those generated files are intentionally not produced in CLR-09-02. Later
milestones wire the full fresh-checkout command sequence and release metadata.

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

From this fixture root, the CLR-09-02 validation command is:

```sh
npa package check --root . --json
```

When running from the parent `npa` workspace before installing `npa`, use:

```sh
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json
```
