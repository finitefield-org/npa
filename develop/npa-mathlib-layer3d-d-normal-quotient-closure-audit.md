# npa-mathlib Layer 3D-D Normal Quotient Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.9` Layer 3D-C first isomorphism release. It selects the normal-subgroup
quotient route for abstract groups. It keeps second isomorphism, third
isomorphism, correspondence, ring isomorphism, CRT, geometry, and analysis
routes out of this layer.

## Baseline

Current public package state:

- `npa-mathlib v0.1.9` is published from standalone repository commit
  `d64e8a70e4b6c1b9f8427285a9c6dc46342db618`.
- The `v0.1.9` tag object is
  `0b5cff064283a0659154ac605f551625f9f6b553`.
- The release bundle hash is
  `101d1dfc017c2ade6d0e37e0eeafa11245770b16594046a3e7a033e486b7f752`.
- Layer 3D-C public modules:
  - `Mathlib.Algebra.Group.FirstIsomorphism`
  - `Mathlib.Algebra.Group.FirstIsomorphism.Image`
- Layer 3D-B public modules:
  - `Mathlib.Algebra.Group.Kernel.Quotient`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Group`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Hom`
- Layer 3B public subgroup and normal-subgroup surface remains in
  `Mathlib.Algebra.Group.Subgroup`.
- Layer 3A public homomorphism surface remains in
  `Mathlib.Algebra.Group.Basic`.
- The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3D-D must add only the normal-subgroup quotient carrier, representative
multiplication compatibility, quotient operations, and quotient group laws. It
must not change package boundaries, registry assumptions, import identity
rules, or proof trust boundaries.

## Selected Candidate Set

The selected Layer 3D-D candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `Mathlib.Algebra.Group.Quotient` | `Mathlib/Algebra/Group/Quotient/` | 3 definitions, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `Mathlib.Algebra.Group.Quotient.Mul` | `Mathlib/Algebra/Group/Quotient/Mul/` | 1 definition, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `Mathlib.Algebra.Group.Quotient.Group` | `Mathlib/Algebra/Group/Quotient/Group/` | 3 definitions, 7 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `Eq.rec` allowed/transitive |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `Mathlib.Algebra.Group.Quotient` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `Mathlib.Algebra.Group.Quotient.Mul` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `Mathlib.Algebra.Group.Quotient.Group` |

This uses `Mathlib.Algebra.Group.Quotient.*` for the normal-subgroup quotient
route. The already released `Mathlib.Algebra.Group.Kernel.Quotient.*` remains
the specialized quotient by a homomorphism kernel relation. `Group.Quotient`
is therefore reserved for the standard quotient group by a normal subgroup, not
for arbitrary equivalence quotients.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Quotient
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup

Mathlib.Algebra.Group.Quotient.Mul
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient

Mathlib.Algebra.Group.Quotient.Group
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Algebra.Group.Basic` and `Mathlib.Algebra.Group.Subgroup` remain
local public `npa-mathlib` modules carried forward unchanged from the released
baseline. `Mathlib.Logic.EqReasoning` remains in the package baseline, but the
selected Layer 3D-D modules do not import it directly.

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.Quotient` definitions:
  - `NormalSetoid`
  - `NormalQuot`
  - `NormalQuotMk`
- `Mathlib.Algebra.Group.Quotient` theorem:
  - `normal_quot_sound`
- `Mathlib.Algebra.Group.Quotient.Mul` definition:
  - `NormalQuotMulRep`
- `Mathlib.Algebra.Group.Quotient.Mul` theorem:
  - `normal_quot_mul_rep_compat`
- `Mathlib.Algebra.Group.Quotient.Group` definitions:
  - `NormalQuotMul`
  - `NormalQuotOne`
  - `NormalQuotInv`
- `Mathlib.Algebra.Group.Quotient.Group` theorems:
  - `normal_quot_mul_mk`
  - `normal_quot_inv_mk`
  - `normal_quot_mul_assoc`
  - `normal_quot_one_mul`
  - `normal_quot_mul_one`
  - `normal_quot_inv_mul`
  - `normal_quot_mul_inv`

Downstream smoke evidence for the materialized release should import
release-bundle certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`
- `Mathlib.Algebra.Group.Quotient`
- `Mathlib.Algebra.Group.Quotient.Mul`
- `Mathlib.Algebra.Group.Quotient.Group`

The downstream theorem should consume a quotient group law such as
`normal_quot_mul_assoc`, not only the carrier constructor, so the smoke fixture
proves the final group-operation route is source-free importable.

## Deferred Candidates

The following nearby routes are intentionally not selected for Layer 3D-D:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Second isomorphism | `Proofs.Ai.Algebra.AbstractGroupSecondIso*` | Depends on the normal quotient surface selected here. Audit after `Mathlib.Algebra.Group.Quotient.*` is public. |
| Third isomorphism | `Proofs.Ai.Algebra.AbstractGroupThirdIso` | Depends on nested normal quotient and quotient-subgroup surfaces. Audit after normal quotient is public. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on subgroup order and normal quotient routes. Release only after their public APIs are fixed. |
| First-isomorphism MVP | `Proofs.Ai.Algebra.AbstractGroupFirstIso` | Still exposes the provisional `FirstIsoRepMvp` surface and is not needed for the normal quotient route. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group quotient/isomorphism plus ring API decisions. Keep in later algebra layers. |

## Axiom Policy

Layer 3D-D does not widen the public axiom policy beyond the `v0.1.9`
baseline. The selected modules use only the builtin equality eliminator surface
already allowed by the public package policy.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entries should declare:

```toml
axioms = ["Eq.rec"]
```

The checked-in corpus axiom report records for the selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The generated AI theorem index records 9 theorem entries for the selected
modules. The theorem index is an untrusted sidecar; proof acceptance remains
based on canonical certificate bytes and verifier results.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `sha256:60d6124f409e4a90b8961c3f88fcdc711a2d0fc8c3c1dd97b2230e27e6470d26` | `sha256:39ad344763f3148f110d29e29ce6be7b7d9da057dcd7f0e168118301cf1b35e4` | `sha256:cbd62585f53d70c8a734c0940bcec5edbd592837eba293a7ab5528e45db75164` | `sha256:9fdeea3b7c832309cc77d8a52df14c8bdf8c3cc2ade0673e86e2d27b1d6f089e` | `sha256:dd888b01bad33171ae12757ca2c80fbf3809f7a20fabefc1bd89f65cdae2a11f` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `sha256:7b410018d5d354f047dc9319ec649ce5d59a321e47dd250adcaacb773f468426` | `sha256:e7a6f9ca72c85e219b0f8235bd224cc349e7f539d07e08212808c81c652bf951` | `sha256:ae3c54a54764c6da92a86eaa668a9d02f0f23276e5a6e1369f37e3f82512acb6` | `sha256:5bd7339ac208646efb0ed11ff2499737a45b42195bb28180b5685aed4560bbe3` | `sha256:644f16f9818987b670656b3a6b7f5f3c915a2916927bb0c19b75d3580186e408` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `sha256:5867f39d3822e100b46971afecc9aec82956075cb809cc2ef8af789ed8b1386b` | `sha256:698e10e857d2cd6a8a4fad6db55fb6727603f49b4f12a36547ca76a058e86687` | `sha256:b204d6179c784b33aaf14dab92d3be171ec78a67aa69c7fc7052467762ff339a` | `sha256:35475c48af8ff4a446c4f9b72745aea69e6442d8ee83fe8587a0a88eeec2e307` | `sha256:deaade4b8916c7d1c87d80a850b25bd470f96c81ea6687324f20635ff5588df5` |

The public `npa-mathlib v0.1.9` package lock records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |
| `Mathlib.Algebra.Group.Subgroup` | `sha256:2b527c5b3039075812d564522facb4a41e421e1902f863df111f0e83c5b9d85c` | `sha256:b3b47c9d314d69594d1c80e24fd4d1954c242ef289e2031207be9acd798ce2a9` |

## Verification

The checked-in corpus certificates passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotient
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`: verified 1 selected module,
  5 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`: verified 1 selected
  module, 6 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`: verified 1 selected
  module, 7 modules including dependency cache.

The source-to-certificate authoring path also regenerated the closures without
changing checked-in artifacts:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotient
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`: built 4 modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`: built 5 modules
  including import closure.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`: built 6 modules
  including import closure.

`git status --short` was clean after the build checks, so the checked-in
certificates and sidecars are already stable for this audit input.

## Decision

Materialize Layer 3D-D as `npa-mathlib v0.1.10` with exactly these new public
modules:

```text
Mathlib.Algebra.Group.Quotient
Mathlib.Algebra.Group.Quotient.Mul
Mathlib.Algebra.Group.Quotient.Group
```

Do not split this closure further: publishing only the carrier or only
representative multiplication would leave downstream users without the quotient
group operations and group laws. Do not include second isomorphism, third
isomorphism, or correspondence theorem modules in the same release.

Materialization must:

1. Copy the selected modules into the standalone `npa-mathlib` repository.
2. Rename module declarations, imports, paths, meta, replay, and manifest
   entries to the public `Mathlib.Algebra.Group.Quotient.*` namespace.
3. Regenerate certificates and package artifacts.
4. Update `npa-mathlib/docs/namespace-policy.md` with the Layer 3D-D mapping.
5. Update downstream smoke to import the normal quotient closure source-free.
6. Run package gates and negative hash/certificate/package-version checks.
7. Publish `npa-mathlib v0.1.10` only after package, downstream, and
   release-bundle evidence are fixed.

## Materialization Evidence

Layer 3D-D has been materialized locally in the standalone repository
`../npa-mathlib` as package version `0.1.10`. Release
publication, tag creation, and push were not performed in this step.

New public modules:

```text
Mathlib.Algebra.Group.Quotient
Mathlib.Algebra.Group.Quotient.Mul
Mathlib.Algebra.Group.Quotient.Group
```

The regenerated public hashes are:

| Public module | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Group.Quotient` | `sha256:95564c1e3a7172a768384daa7392e06895df59581cba4e532ddd9f1c722f4f31` | `sha256:14d52188e958a89a9f87ad32a12c2738a73c0e185d68bcd390875bf4dea7f4de` | `sha256:9fdeea3b7c832309cc77d8a52df14c8bdf8c3cc2ade0673e86e2d27b1d6f089e` | `sha256:6fc7c518e7eaa04e792bb1d4dd3af7b4f5c4987136a118ce98a75d601c4feb19` |
| `Mathlib.Algebra.Group.Quotient.Mul` | `sha256:541772d27e36bc290e149eb6b1498e659dea43e7d53884b2d828f26e5c9c28b8` | `sha256:9570dfe4c8b6ca61fe53acd2bf41cf193be20a4d9118cbc295f1c1a10d602cc7` | `sha256:5bd7339ac208646efb0ed11ff2499737a45b42195bb28180b5685aed4560bbe3` | `sha256:5495b3aba0eb579efd418b5a86fad16765c2f21889c9531747fa0d152f774013` |
| `Mathlib.Algebra.Group.Quotient.Group` | `sha256:cf0072a2d9bd870cea4f1ccad9eab7ece8a7d6f6b582a109fc506c31f8bc06a0` | `sha256:fb6bb1472ee946b6db2174ccf4cf845b0a7dda6c8c42ccc9396950a5a8a92654` | `sha256:35475c48af8ff4a446c4f9b72745aea69e6442d8ee83fe8587a0a88eeec2e307` | `sha256:41a716e2220952290f35148b55d9505f1c8f7b90491233c10ee704bb5b678ef1` |

Generated package artifacts were refreshed:

- `generated/package-lock.json`
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `generated/publish-plan.json`

`generated/publish-plan.json` now records:

- local modules: 27
- external imports: 2
- registry entries: 27
- checker summaries: 58
- publish plan hash:
  `sha256:7c256cb10a27d73707e8fec851d52d1cf3832e170c2a3deb9ab20142b21eea49`

The downstream smoke fixture now proves
`Downstream.GroupNormalQuotient::normal_quot_mul_assoc_passthrough` by applying
`normal_quot_mul_assoc`. Its vendored source-free certificate closure includes
the Layer 3D-D modules plus the certificate dependencies needed to verify
`Mathlib.Algebra.Group.Basic` and `Mathlib.Algebra.Group.Subgroup`.

Package gates passed:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
git -C ../npa-mathlib diff --check
```

Downstream smoke gates passed:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
```

The downstream source-free reference verifier reported package verification for
8 modules.

Negative checks rejected:

- bad `Mathlib.Algebra.Group.Quotient` export hash:
  `export_hash_mismatch`
- bad `Mathlib.Algebra.Group.Quotient.Group` certificate hash:
  `certificate_hash_mismatch`
- corrupted vendored `Mathlib.Algebra.Group.Quotient.Group` certificate bytes:
  `certificate_decode_failed`
- bad imported `npa-mathlib` package version:
  `package_lock_stale`
