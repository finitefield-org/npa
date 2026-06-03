# npa-mathlib Layer 1 Closure Audit

Date: 2026-06-02

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.0` Layer 0 release. It is an input to materialization in the standalone
`finitefield-org/npa-mathlib` repository; it does not publish new artifacts by
itself.

## Baseline

Current public package state:

- `npa-mathlib v0.1.0` is published from standalone repository commit
  `8d8db311916cb3bae7fd9ce783139d17e3196747`.
- The release bundle hash is
  `d89dd2cb08ae21c20b9ca889285d9fcb50b1c133d40556e0601588a44e9632d9`.
- Layer 0 public modules are:
  - `Mathlib.Logic.Basic`
  - `Mathlib.Logic.Prop`
  - `Mathlib.Logic.Eq`
  - `Mathlib.Data.Nat.Basic`
  - `Mathlib.Core.Reduction`
- The standalone namespace policy is
  `../npa-mathlib/docs/namespace-policy.md`.

Layer 1 must add modules without changing the package split, registry
assumptions, or proof trust boundary.

## Selected Candidate Set

The Layer 1 candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports |
| --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.Ring` | `Mathlib.Algebra.Ring` | `Mathlib/Algebra/Ring/` | 1 inductive, 6 definitions, 20 theorems | `Std.Logic.Eq` |
| `Proofs.Ai.Algebra.Square` | `Mathlib.Algebra.Square` | `Mathlib/Algebra/Square/` | 2 definitions, 11 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.Ring` |
| `Proofs.Ai.OrderedField` | `Mathlib.Algebra.OrderedField` | `Mathlib/Algebra/OrderedField/` | 3 definitions, 9 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Square
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring

Mathlib.Algebra.OrderedField
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring
  imports Mathlib.Algebra.Square
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
selected Layer 1 modules do not introduce a new external package dependency.
The existing `Std.Nat.Basic` package import remains required by Layer 0 modules
already present in `npa-mathlib`.

## Verification

The checked-in corpus certificates for the selected modules passed
source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.Ring
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.Square
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.OrderedField
```

Results:

- `Proofs.Ai.Algebra.Ring`: verified 1 selected module, 2 modules including
  dependency cache.
- `Proofs.Ai.Algebra.Square`: verified 1 selected module, 3 modules including
  dependency cache.
- `Proofs.Ai.OrderedField`: verified 1 selected module, 4 modules including
  dependency cache.

The source-to-certificate authoring path also regenerated the same closure:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Ring
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Square
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.OrderedField
```

Results:

- `Proofs.Ai.Algebra.Ring`: built 1 module including import closure.
- `Proofs.Ai.Algebra.Square`: built 2 modules including import closure.
- `Proofs.Ai.OrderedField`: built 3 modules including import closure.

All three candidate `meta.json` files report `axioms = []`.

## Readiness Decision

Layer 1 is ready for materialization in the standalone `npa-mathlib`
repository.

Materialization must not copy the old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.0`; provisionally this is
`v0.1.1`.

## Next Materialization Steps

Run these steps in `../npa-mathlib`:

1. Add `Mathlib/Algebra/Ring/`, `Mathlib/Algebra/Square/`, and
   `Mathlib/Algebra/OrderedField/` from the selected corpus sources.
2. Rename module-local imports from `Proofs.Ai.*` to `Mathlib.*`.
3. Keep the existing `npa-std v0.1.0` hash-pinned imports for `Std.Logic.Eq`
   and `Std.Nat.Basic`.
4. Add manifest entries for the three new modules and bump the package version
   for the next release.
5. Regenerate certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
6. Update the downstream smoke fixture so it imports at least one Layer 1
   certificate artifact through a package import bundle.
7. Run package gates for `npa-mathlib` and downstream smoke.
8. Publish the next release only after release bundle and downstream smoke
   evidence are fixed.

Do not start CLR-08 high-trust evidence work as part of this Layer 1
materialization. It remains a separate release assurance track.
