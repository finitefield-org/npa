# npa-mathlib Layer 3B Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the public
`v0.1.4` Layer 3A abstract group foundation release. It is an input to
materialization in the standalone `finitefield-org/npa-mathlib` repository; it
does not publish new artifacts by itself.

## Baseline

Current public package state:

- `npa-mathlib v0.1.4` is published from standalone repository commit
  `e775afff5b9a2abe7709d7d66afe333c37cab955`.
- The release bundle hash is
  `d216da5522a5d4cd5e37ae059387b93632a0d04aa6ea6f9b8e757c256789ee4c`.
- Layer 3A public modules are:
  - `Mathlib.Logic.EqReasoning`
  - `Mathlib.Algebra.Group.Basic`
- Layer 2B public modules remain:
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
- The current public axiom policy keeps custom package-local axioms disabled
  and permits only the builtin equality eliminator surface:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3B must add only the subgroup and normal-subgroup foundation needed
after `Mathlib.Algebra.Group.Basic`. It must not change the package split,
registry assumptions, import identity rules, or proof trust boundary.

## Selected Candidate Set

The Layer 3B candidate set is closed and small enough to materialize next:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axioms |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `Mathlib.Algebra.Group.Subgroup` | `Mathlib/Algebra/Group/Subgroup/` | 5 definitions, 28 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.EqReasoning` | `Eq.rec` |

The requested algebra namespace mapping is therefore:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `Mathlib.Algebra.Group.Subgroup` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.Subgroup
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` remain local
public `npa-mathlib` modules carried forward unchanged from the released
`v0.1.4` baseline. Downstream smoke evidence for `v0.1.5` must import the
release-bundle certificate bytes for the full closure, including these
carried-forward modules.

The selected module introduces the following public surface:

- Definitions:
  - `SubgroupLawArgs`
  - `NormalSubgroupLawArgs`
  - `SubgroupInterPred`
  - `SubgroupProductPred`
  - `NormalRel`
- Theorems:
  - `subgroup_one`
  - `subgroup_mul_closed`
  - `subgroup_inv_closed`
  - `normal_subgroup_laws`
  - `normal_conj_closed`
  - `normal_inv_conj_closed`
  - `subgroup_inter_intro`
  - `subgroup_inter_left`
  - `subgroup_inter_right`
  - `subgroup_inter_one`
  - `subgroup_inter_mul_closed`
  - `subgroup_inter_inv_closed`
  - `subgroup_inter_normal_in_left`
  - `subgroup_product_intro`
  - `subgroup_product_elim`
  - `subgroup_product_one`
  - `subgroup_product_mul_closed`
  - `subgroup_product_inv_closed`
  - `subgroup_product_laws`
  - `normal_rel_refl`
  - `normal_rel_symm`
  - `normal_rel_trans`
  - `normal_rel_of_eq`
  - `normal_rel_mul_compat`
  - `normal_rel_inv_compat`
  - `normal_rel_one_of_mem`
  - `normal_rel_one_to_mem`
  - `normal_rel_product_right`

The selected set does not depend on:

- `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`
- `Proofs.Ai.Algebra.AbstractGroupKernel`
- `Proofs.Ai.Algebra.AbstractGroupImage`
- `Proofs.Ai.Algebra.AbstractGroupQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`
- `Proofs.Ai.Algebra.AbstractGroup*Iso*`
- `Proofs.Ai.Algebra.AbstractGroupCorrespondence*`
- `Proofs.Ai.Geometry.Pythagorean`
- `Proofs.Ai.Geometry.Abstract*`
- `Proofs.Ai.Analysis.*`

## Axiom Policy

Layer 3B does not widen the public axiom policy beyond the `v0.1.4` baseline.
The selected module uses the builtin equality eliminator `Eq.rec`, which is
already allowed by the public package policy.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The new local module entry must declare:

```toml
axioms = ["Eq.rec"]
```

The regenerated package axiom report must show `Eq.rec` as allowed and must
show zero policy violations. `Eq.rec` is builtin equality eliminator surface;
it is not `sorry` and it is not evidence from source, replay, theorem index,
publish plan, CI, Git tag, or registry metadata.

## Deferred Candidates

The following nearby modules are verified in the proof corpus, but they are not
part of this Layer 3B release candidate.

| Corpus module | Declarations | Direct imports | Axioms | Verification result | Build result | Reason deferred |
| --- | --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | 3 definitions, 12 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroupSubgroup` | none | verified 1 selected module, 5 modules including dependency cache | built 4 local modules including import closure | Adds subgroup containment/order surface. Release separately after base subgroup API is stable. |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Kernel route should be grouped with image, quotient, and isomorphism decisions, not the subgroup foundation layer. |
| `Proofs.Ai.Algebra.AbstractGroupImage` | 1 definition, 5 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Image route should be grouped with kernel, quotient, and isomorphism decisions. |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | 4 definitions, 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup` | `Eq.rec` | verified 1 selected module, 4 modules including dependency cache | built 3 local modules including import closure | Introduces quotient/setoid-facing public surface; keep it in a separate quotient audit. |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | 3 definitions, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `Eq.rec` | verified 1 selected module, 5 modules including dependency cache | built 4 local modules including import closure | Starts normal quotient public API; it depends on this subgroup foundation but should not ship with it. |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | 1 definition, 1 theorem | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `Eq.rec` | verified 1 selected module, 6 modules including dependency cache | built 5 local modules including import closure | Belongs to the normal quotient route. |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | 3 definitions, 7 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `Eq.rec` | verified 1 selected module, 7 modules including dependency cache | built 6 local modules including import closure | Belongs to the normal quotient route. |

The first follow-on audit after Layer 3B should choose one of these routes:

1. Subgroup order/containment:
   `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`.
2. Kernel/image/quotient/isomorphism:
   `Proofs.Ai.Algebra.AbstractGroupKernel`,
   `Proofs.Ai.Algebra.AbstractGroupImage`,
   `Proofs.Ai.Algebra.AbstractGroupQuotient`, and the first/second/third
   isomorphism modules.
3. Normal quotient:
   `Proofs.Ai.Algebra.AbstractGroupNormalQuotient*`.

Do not merge these routes into the `v0.1.5` subgroup release unless a separate
closure audit replaces this decision.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroup` | `sha256:45c33f367e505194a12eb206376c0881a9cf98a1858d2cc550f4ec5577993aef` | `sha256:890d6d64962651eb0bd5ee735a91d2ffcc3826cc97a338b31c4b916c2e9bdcde` | `sha256:c2ae287a3e5e0f4de41b6a201d345af9f83aa200f20dbdcf487d15273ca5f3b4` | `sha256:1298b31aa5ef802d3af57bb01d8cc3f3a11b162f4f51bbe1fd8e650e7f036612` | `sha256:0d356dd1f768b8665f5cf9b7d4a75ea1a34422181907125fbf167263e4b8092d` |

For comparison, the nearby deferred modules currently have these proof-corpus
hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder` | `sha256:7f10850d8346f35748b12431cf7d7c28330dbb5f06ca6fc62d0e7da4c6ea759c` | `sha256:fa5521f2655b4877272a55985a1bfa37219f052c58fe9a0deb13efe38aae8249` | `sha256:474920dc37a91a499b50591e71ee8c73c3c0f5e68c04857ba672661e2dd40806` | `sha256:3d3fdbf6a3ca4756ceaac9853e839a84878b24c0f6290e2246a78c6184b31e0e` | `sha256:242fa3ed3a61b0cf982d2b440f79f635a0f8729ae215c5ef9d3aa65aeb63a1c6` |
| `Proofs.Ai.Algebra.AbstractGroupKernel` | `sha256:8d11dd74da047650e74282aeea0d7a0ac2aacfc5cb3bae1bf85c12bf5c38fadb` | `sha256:6d4390bbcf7d5b22c92d24b144575cbc3f870971e03574e43633bd9a64068144` | `sha256:5c1a39b0e9f82dc1014e3e63549c71eab3db6383024a87408e4e95e06bc33fbb` | `sha256:39f92e3a74edf201d900d93abeebb9c404ea8c141bfc46761f7e889c9f8cf9f7` | `sha256:1d5b2a02b6cc7d9dfafbb781f3d27b9494e7655644aa046871bbb7abb9170bd8` |
| `Proofs.Ai.Algebra.AbstractGroupImage` | `sha256:8c6f85eb72c6d04a2b5b97448d7aa7f7b331250591be2afe3ce329ed6a171080` | `sha256:f5e396c6b08319835864296ee0fbb273dbfeda738652f4f44767e05cf57023cc` | `sha256:0935f963b77bb9ed124d4b38435ad2a9a19f860e1f06de11c1ab8ca3f05cdd0e` | `sha256:0869976f857fbf07454df1db004a649ae3ee9ede0e1429dbf40af6dfaac5bfeb` | `sha256:af05a4354e4be42ad76b6fb1da6a149d34dfe5207637fdff9b7ff246076935a1` |
| `Proofs.Ai.Algebra.AbstractGroupQuotient` | `sha256:a72a0498b88ed9cf0487d309a710ff021647e01ed9b04fe7663caa2cb7dd88e7` | `sha256:c64b8020f5bc26953e4ddcd4e003bb8a5421a326b6662f1b44ce6a1060d55748` | `sha256:456e96fbacbb84ec968918cae2b0914f9b851c96f3bdf1fda65be372c247ea48` | `sha256:30155dfd0399b8bb9222cee6aff0c26065a6766282a10f97ef2d27d45d89aa6a` | `sha256:263097d5b8ce78ffd3ce21a3a74d8d4598cf2bc18da274a99431e64b6e3739d2` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotient` | `sha256:60d6124f409e4a90b8961c3f88fcdc711a2d0fc8c3c1dd97b2230e27e6470d26` | `sha256:39ad344763f3148f110d29e29ce6be7b7d9da057dcd7f0e168118301cf1b35e4` | `sha256:cbd62585f53d70c8a734c0940bcec5edbd592837eba293a7ab5528e45db75164` | `sha256:9fdeea3b7c832309cc77d8a52df14c8bdf8c3cc2ade0673e86e2d27b1d6f089e` | `sha256:dd888b01bad33171ae12757ca2c80fbf3809f7a20fabefc1bd89f65cdae2a11f` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul` | `sha256:7b410018d5d354f047dc9319ec649ce5d59a321e47dd250adcaacb773f468426` | `sha256:e7a6f9ca72c85e219b0f8235bd224cc349e7f539d07e08212808c81c652bf951` | `sha256:ae3c54a54764c6da92a86eaa668a9d02f0f23276e5a6e1369f37e3f82512acb6` | `sha256:5bd7339ac208646efb0ed11ff2499737a45b42195bb28180b5685aed4560bbe3` | `sha256:644f16f9818987b670656b3a6b7f5f3c915a2916927bb0c19b75d3580186e408` |
| `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `sha256:5867f39d3822e100b46971afecc9aec82956075cb809cc2ef8af789ed8b1386b` | `sha256:698e10e857d2cd6a8a4fad6db55fb6727603f49b4f12a36547ca76a058e86687` | `sha256:b204d6179c784b33aaf14dab92d3be171ec78a67aa69c7fc7052467762ff339a` | `sha256:35475c48af8ff4a446c4f9b72745aea69e6442d8ee83fe8587a0a88eeec2e307` | `sha256:deaade4b8916c7d1c87d80a850b25bd470f96c81ea6687324f20635ff5588df5` |

## Verification

The checked-in corpus certificate for the selected module passed source-free
verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSubgroup
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`: verified 1 selected module,
  4 modules including dependency cache.

The source-to-certificate authoring path also regenerated the same closure:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSubgroup
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupSubgroup`: built 3 local modules including
  import closure: `Proofs.Ai.EqReasoning`,
  `Proofs.Ai.Algebra.AbstractGroup`, and
  `Proofs.Ai.Algebra.AbstractGroupSubgroup`.

The deferred comparison modules also passed source-free verification and
source-to-certificate builds:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSubgroupOrder
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupImage
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupQuotient
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotient
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSubgroupOrder
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupKernel
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupImage
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupQuotient
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotient
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup
```

Results:

- `Proofs.Ai.Algebra.AbstractGroupSubgroupOrder`: verified 1 selected module,
  5 modules including dependency cache; built 4 local modules including import
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
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`: verified 1 selected module,
  5 modules including dependency cache; built 4 local modules including import
  closure.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`: verified 1 selected
  module, 6 modules including dependency cache; built 5 local modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`: verified 1 selected
  module, 7 modules including dependency cache; built 6 local modules including
  import closure.

The difference between source-free verification counts and build counts is the
external cached dependency `Std.Logic.Eq`, which is verified as an import
artifact and is not rebuilt as a local corpus module.

## Readiness Decision

Layer 3B is ready for materialization in the standalone `npa-mathlib`
repository as `npa-mathlib v0.1.5` with exactly this new public module:

```text
Mathlib.Algebra.Group.Subgroup
```

Do not include subgroup order, kernel, image, quotient, normal quotient,
isomorphism, or correspondence modules in the same release. They are verified,
but they should be released only after separate closure audits because they
widen the public API surface beyond the subgroup and normal-subgroup
foundation.

Materialization must not copy old proof identity as public evidence. The
source module currently uses historical corpus names under `Proofs.Ai.*`, and
module names are proof-relevant. The public package must rename source imports
to `Mathlib.*`, regenerate certificates, regenerate generated package
artifacts, and update downstream smoke fixtures before release.

Use the next package/release version after `v0.1.4`; provisionally this is
`v0.1.5`.

## Materialization Steps

1. Copy `Proofs.Ai.Algebra.AbstractGroupSubgroup` into the standalone
   `npa-mathlib` repository under `Mathlib/Algebra/Group/Subgroup/`.
2. Rename the module declaration to `Mathlib.Algebra.Group.Subgroup`.
3. Replace internal imports with `Std.Logic.Eq`,
   `Mathlib.Logic.EqReasoning`, and `Mathlib.Algebra.Group.Basic`.
4. Keep package name `npa-mathlib` and use the existing `npa-std v0.1.0`
   hash-pinned import for `Std.Logic.Eq`.
5. Keep `Mathlib.Logic.EqReasoning` and `Mathlib.Algebra.Group.Basic` as
   carried-forward local modules from the released `v0.1.4` baseline.
6. Add the new manifest entry for `Mathlib.Algebra.Group.Subgroup` with
   `axioms = ["Eq.rec"]`.
7. Keep `allow_custom_axioms = false` and
   `allowed_axioms = ["Eq.rec"]`.
8. Bump package and release artifacts to `v0.1.5`.
9. Regenerate certificates for the renamed module.
10. Regenerate generated package artifacts: `package-lock.json`,
    `axiom-report.json`, `theorem-index.json`, and `publish-plan.json`.
11. Update downstream source-free smoke to import
    `Mathlib.Algebra.Group.Subgroup` from release-bundle certificate bytes.
12. Run package gates for `npa-mathlib` and the downstream smoke.
13. Create the `v0.1.5` release bundle only after generated artifacts,
    downstream evidence, and negative import hash checks are fixed.

## Materialization Result

The audited closure was materialized and published as `npa-mathlib v0.1.5` in
the standalone `finitefield-org/npa-mathlib` repository.

Release identity:

- GitHub Release:
  `https://github.com/finitefield-org/npa-mathlib/releases/tag/v0.1.5`
- standalone commit:
  `3050b36f83985eabb0c64cd8dbd55554371a9ffd`
- tag object:
  `cc495750acf520549d237c22a71182255d32a333`
- tag target:
  `3050b36f83985eabb0c64cd8dbd55554371a9ffd`
- release bundle SHA-256:
  `7893ab55d0f56e19cd0337f461d772c141442a33c80bd1113248938a6f3b930d`

The public `Mathlib.Algebra.Group.Subgroup` module was regenerated after the
`Mathlib.*` namespace rename. Its public hashes are:

| Public module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Group.Subgroup` | `sha256:94093566a05cbfc8f495124578d2614a54e6ebe546e181ba7432e274af25a1bc` | `sha256:f2a6bb8cd9329c2b9d5cc7c433f2b6ba6d43ecfd1e8fa5c503a1897c2e3326ca` | `sha256:2b527c5b3039075812d564522facb4a41e421e1902f863df111f0e83c5b9d85c` | `sha256:1298b31aa5ef802d3af57bb01d8cc3f3a11b162f4f51bbe1fd8e650e7f036612` | `sha256:b3b47c9d314d69594d1c80e24fd4d1954c242ef289e2031207be9acd798ce2a9` |

The regenerated package artifacts record:

- package version: `0.1.5`
- local modules: 15
- external imports: 2
- release artifacts: 21
- module registry seed entries: 15
- checker summaries in publish plan: 34
- axiom report module count: 17
- direct axiom count: 1
- transitive axiom count: 3
- policy violation count: 0
- publish plan hash:
  `sha256:f1c9e185a5f64d07efc6fd13ec005d832ffe62f6cc8f8564f506067b10fc0191`

The downstream smoke fixture was updated to
`Downstream.GroupSubgroup::subgroup_one_passthrough`. It vendors only
release-bundle certificate bytes for:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`

The following package gates passed locally for the standalone package and the
downstream smoke:

- `package check`
- `package build-certs --check`
- `package verify-certs --checker reference`
- `package check-hashes`
- `package axiom-report --check`
- `package index --check`
- `package publish-plan --check`

Negative checks rejected a bad `Mathlib.Algebra.Group.Subgroup` export hash,
bad certificate hash, corrupted certificate bytes, and bad package version
before proof acceptance. No source, replay data, theorem index content, publish
plan metadata, Git tag, GitHub Release page, or CI status is proof evidence.
