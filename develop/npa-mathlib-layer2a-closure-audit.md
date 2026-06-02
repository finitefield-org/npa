# npa-mathlib Layer 2A Closure Audit

Date: 2026-06-02

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.1` Layer 1 algebra/order release. It is an input to materialization in
the standalone `finitefield-org/npa-mathlib` repository; it does not publish new
artifacts by itself.

## Baseline

Current public package state:

- `npa-mathlib v0.1.1` is published from standalone repository commit
  `449855a37cbf1d3ebe777d5a6b044d47be324532`.
- The release bundle hash is
  `ada3f288537dc777697c1083765790aa9dbd8782f43356c1f8572a1fa6ccbcb9`.
- Layer 1 public modules are:
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
  `/Users/kazuyoshitoshiya/ff/npa-mathlib/docs/namespace-policy.md`.

Layer 2A must add vector modules without changing the package split, registry
assumptions, or proof trust boundary. Geometry remains Layer 2B and must not be
materialized as part of this audit.

## Selected Candidate Set

The Layer 2A candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports |
| --- | --- | --- | --- | --- |
| `Proofs.Ai.Vector.Basic` | `Mathlib.Vector.Basic` | `Mathlib/Vector/Basic/` | 1 inductive, 4 definitions, 12 theorems | `Std.Logic.Eq` |
| `Proofs.Ai.Vector.Dot` | `Mathlib.Vector.Dot` | `Mathlib/Vector/Dot/` | 3 definitions, 17 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`, `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Vector.Basic
  imports Std.Logic.Eq

Mathlib.Vector.Dot
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Ring
  imports Mathlib.Algebra.Square
  imports Mathlib.Algebra.OrderedField
  imports Mathlib.Vector.Basic
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import. The
selected Layer 2A modules do not introduce a new external package dependency.
The Layer 1 algebra/order modules are already local public `npa-mathlib`
modules as of `v0.1.1`.

The selected set does not depend on:

- `Proofs.Ai.Geometry.*`
- `Proofs.Ai.Analysis.*`
- `Proofs.Ai.Algebra.Abstract*`
- `Proofs.Ai.Vector.Abstract*`

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization must
regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Vector.Basic` | `sha256:19db7e04cd08724e4f8e393786e86caac000a62a408b5ac25b9024e741f262a0` | `sha256:c8246be50af72b984424bd1c670cadcb0cca3e6c9cd84bdd3702ba473a9bffc3` | `sha256:3ba8f7b514c7f041a1ac86bf1800a21186255bac9ce31e2fea0b7d9c91d4c938` | `sha256:55932adb6d068a32ac76b43afee2b808d61b89bb36b85b1805fe77d82a1028b3` | `sha256:1bf2067f0bca0d1e49465dc022a3f93e3b2bb2e945d81e583f5bcc8d08a7aebf` |
| `Proofs.Ai.Vector.Dot` | `sha256:017d38abee99db667ef21a8ac942b3e278edbd355126fef1c2970c54505489a7` | `sha256:8d9086efc1c77c9bfe7da657ffddde46cfb5c4393254228c65dbf1a77a0d6f57` | `sha256:f1b845f94e0c81e8f6e83eba598f5ea74373e2cb947f695ee477a8e2c295d777` | `sha256:fed11e73accfbfb0dfc28b4f510e151fa33d8af82d58fdb23b92567e04e59e40` | `sha256:d340fa14eb072f59ddaf4901c5357e0152a1c0893dd09988953562178ec8b624` |

Both selected `meta.json` files report `axioms = []`.

## Verification

The checked-in corpus certificates for the selected modules passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Vector.Basic
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Vector.Dot
```

Results:

- `Proofs.Ai.Vector.Basic`: verified 1 selected module, 2 modules including
  dependency cache.
- `Proofs.Ai.Vector.Dot`: verified 1 selected module, 6 modules including
  dependency cache.

The source-to-certificate authoring path also regenerated the same closure:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Vector.Basic
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Vector.Dot
```

Results:

- `Proofs.Ai.Vector.Basic`: built 1 module including import closure.
- `Proofs.Ai.Vector.Dot`: built 5 modules including import closure:
  `Proofs.Ai.Algebra.Ring`, `Proofs.Ai.Algebra.Square`,
  `Proofs.Ai.OrderedField`, `Proofs.Ai.Vector.Basic`, and
  `Proofs.Ai.Vector.Dot`.

The difference between the source-free verification count and build count is
the external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

## Readiness Decision

Layer 2A is ready for materialization in the standalone `npa-mathlib`
repository.

Materialization must not copy the old proof identity as public evidence. The
source modules currently use historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.1`; provisionally this is
`v0.1.2`.

## Next Materialization Steps

Run these steps in `/Users/kazuyoshitoshiya/ff/npa-mathlib`:

1. Add `Mathlib/Vector/Basic/` and `Mathlib/Vector/Dot/` from the selected
   corpus sources.
2. Rename module-local imports from `Proofs.Ai.*` to `Mathlib.*`.
3. Keep the existing `npa-std v0.1.0` hash-pinned imports for `Std.Logic.Eq`
   and `Std.Nat.Basic`.
4. Keep the released Layer 0 and Layer 1 modules local in `npa-mathlib`.
5. Add manifest entries for the two new modules and bump the package version
   for the next release.
6. Regenerate certificates and generated package artifacts:
   `package-lock.json`, `axiom-report.json`, `theorem-index.json`, and
   `publish-plan.json`.
7. Update the downstream smoke fixture so it imports at least one Layer 2A
   certificate artifact through a package import bundle.
8. Run package gates for `npa-mathlib` and downstream smoke.
9. Publish the next release only after release bundle and downstream smoke
   evidence are fixed.

Do not start Layer 2B geometry or CLR-08 high-trust evidence work as part of
this Layer 2A materialization. Both remain separate release tracks.

## Release Outcome

Layer 2A was materialized and published as `npa-mathlib v0.1.2` on
2026-06-02.

Published release evidence:

- release URL:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.2`
- tag object: `d59032b305272d5fec557f3c07700720b2b51e27`
- target commit: `4c28e82d3dc2e0a8a25bb2e01bb433c7a10a28fe`
- release bundle:
  `https://github.com/finitefield-org/npa-mathlib/releases/download/v0.1.2/npa-mathlib-v0.1.2-release-artifacts.tar.gz`
- release bundle SHA-256:
  `7b1d8fe69b0bca46e77149453e79ece8198473ce9e760d90e9f8e2c66b117d68`

The standalone package gates passed locally for `check`,
`build-certs --check`, `verify-certs --checker reference`, `check-hashes`,
`axiom-report --check`, `index --check`, and `publish-plan --check`.

The published release bundle was then downloaded, checked against its SHA
sidecar, and used as the only `npa-mathlib` certificate source for a downstream
smoke fixture. The downstream smoke passed `check`, `build-certs --check`,
`verify-certs --checker reference`, and `check-hashes`.

GitHub Actions status is intentionally not used as release evidence for this
release. The fixed evidence is the local package gate output, the published
bundle checksum, and the published-release downstream smoke.
