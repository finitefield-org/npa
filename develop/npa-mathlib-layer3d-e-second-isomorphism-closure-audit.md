# npa-mathlib Layer 3D-E Second Isomorphism Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the Layer 3D-D
normal quotient materialization. It selects the abstract-group second
isomorphism route. It keeps third isomorphism, correspondence, ring
isomorphism, CRT, geometry, and analysis routes out of this layer.

## Baseline

Current intended package state:

- Layer 3D-D has been materialized locally in the standalone repository
  `/Users/kazuyoshitoshiya/ff/npa-mathlib` as `npa-mathlib v0.1.10`.
- `v0.1.10` release publication, tag creation, and push are still pending.
  Layer 3D-E must not be published until the `v0.1.10` release-bundle evidence
  is fixed.
- Layer 3D-D public modules:
  - `Mathlib.Algebra.Group.Quotient`
  - `Mathlib.Algebra.Group.Quotient.Mul`
  - `Mathlib.Algebra.Group.Quotient.Group`
- Layer 3D-C public modules:
  - `Mathlib.Algebra.Group.FirstIsomorphism`
  - `Mathlib.Algebra.Group.FirstIsomorphism.Image`
- Layer 3D-B public kernel-quotient modules remain separate from the standard
  normal-subgroup quotient route:
  - `Mathlib.Algebra.Group.Kernel.Quotient`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Mul`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Group`
  - `Mathlib.Algebra.Group.Kernel.Quotient.Hom`
- Layer 3B public subgroup and normal-subgroup surface remains in
  `Mathlib.Algebra.Group.Subgroup`.
- Layer 3A public homomorphism and group-law surface remains in
  `Mathlib.Algebra.Group.Basic`.
- The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

Layer 3D-E must add only the second-isomorphism map, kernel/intersection
facts, image/product-quotient facts, and final theorem evidence. It must not
change package boundaries, registry assumptions, import identity rules, or
proof trust boundaries.

## Selected Candidate Set

The selected Layer 3D-E candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi` | `Mathlib.Algebra.Group.SecondIsomorphism.Map` | `Mathlib/Algebra/Group/SecondIsomorphism/Map/` | 1 definition, 4 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel` | `Mathlib.Algebra.Group.SecondIsomorphism.Kernel` | `Mathlib/Algebra/Group/SecondIsomorphism/Kernel/` | 1 definition, 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage` | `Mathlib.Algebra.Group.SecondIsomorphism.Image` | `Mathlib/Algebra/Group/SecondIsomorphism/Image/` | 2 definitions, 6 theorems | `Std.Logic.Eq`, `Proofs.Ai.EqReasoning`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi` | `Eq.rec` allowed/transitive |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal` | `Mathlib.Algebra.Group.SecondIsomorphism` | `Mathlib/Algebra/Group/SecondIsomorphism/` | 3 definitions, 3 theorems | `Std.Logic.Eq`, `Proofs.Ai.Algebra.AbstractGroup`, `Proofs.Ai.Algebra.AbstractGroupSubgroup`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotient`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul`, `Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup`, `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi`, `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel`, `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage` | `Eq.rec` allowed/transitive |

The public namespace mapping is:

| Corpus module | Public algebra module |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi` | `Mathlib.Algebra.Group.SecondIsomorphism.Map` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel` | `Mathlib.Algebra.Group.SecondIsomorphism.Kernel` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage` | `Mathlib.Algebra.Group.SecondIsomorphism.Image` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal` | `Mathlib.Algebra.Group.SecondIsomorphism` |

This uses `SecondIsomorphism.Map` instead of a public `Phi` module name to
avoid Greek-letter implementation terminology in released module identifiers.
The declaration names still use the existing corpus surface, such as
`SecondIsoPhi` and `second_iso_phi_mul`; declaration renaming is outside this
closure audit.

The root module `Mathlib.Algebra.Group.SecondIsomorphism` is reserved for the
final second-isomorphism theorem evidence. It imports the `Map`, `Kernel`, and
`Image` submodules.

After public namespace materialization, the internal imports must become:

```text
Mathlib.Algebra.Group.SecondIsomorphism.Map
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group

Mathlib.Algebra.Group.SecondIsomorphism.Kernel
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.SecondIsomorphism.Map

Mathlib.Algebra.Group.SecondIsomorphism.Image
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.SecondIsomorphism.Map

Mathlib.Algebra.Group.SecondIsomorphism
  imports Std.Logic.Eq
  imports Mathlib.Algebra.Group.Basic
  imports Mathlib.Algebra.Group.Subgroup
  imports Mathlib.Algebra.Group.Quotient
  imports Mathlib.Algebra.Group.Quotient.Mul
  imports Mathlib.Algebra.Group.Quotient.Group
  imports Mathlib.Algebra.Group.SecondIsomorphism.Map
  imports Mathlib.Algebra.Group.SecondIsomorphism.Kernel
  imports Mathlib.Algebra.Group.SecondIsomorphism.Image
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning`, `Mathlib.Algebra.Group.Basic`,
`Mathlib.Algebra.Group.Subgroup`, and `Mathlib.Algebra.Group.Quotient.*`
remain local public `npa-mathlib` modules carried forward unchanged from the
released baseline. Only `SecondIsomorphism.Image` imports
`Mathlib.Logic.EqReasoning` directly.

The selected modules introduce the following public surface:

- `Mathlib.Algebra.Group.SecondIsomorphism.Map` definition:
  - `SecondIsoPhi`
- `Mathlib.Algebra.Group.SecondIsomorphism.Map` theorems:
  - `second_iso_phi_mk`
  - `second_iso_phi_mul`
  - `second_iso_phi_one`
  - `second_iso_phi_inv`
- `Mathlib.Algebra.Group.SecondIsomorphism.Kernel` definition:
  - `SecondIsoKernelPred`
- `Mathlib.Algebra.Group.SecondIsomorphism.Kernel` theorems:
  - `second_iso_kernel_sound`
  - `second_iso_kernel_to_inter`
  - `second_iso_inter_to_kernel`
- `Mathlib.Algebra.Group.SecondIsomorphism.Image` definitions:
  - `SecondIsoImagePred`
  - `SecondIsoProductQuotPred`
- `Mathlib.Algebra.Group.SecondIsomorphism.Image` theorems:
  - `second_iso_image_intro`
  - `second_iso_image_elim`
  - `second_iso_product_quot_intro`
  - `second_iso_product_quot_elim`
  - `second_iso_image_to_product_quot`
  - `second_iso_product_quot_to_image`
- `Mathlib.Algebra.Group.SecondIsomorphism` definitions:
  - `SecondIsoKernelEvidence`
  - `SecondIsoImageEvidence`
  - `SecondIsoTheoremEvidence`
- `Mathlib.Algebra.Group.SecondIsomorphism` theorems:
  - `second_iso_kernel_evidence`
  - `second_iso_image_evidence`
  - `second_isomorphism_theorem_evidence`

Downstream smoke evidence for the materialized release should import
release-bundle certificate bytes for the actual closure:

- `Std.Logic.Eq`
- `Mathlib.Logic.EqReasoning`
- `Mathlib.Algebra.Group.Basic`
- `Mathlib.Algebra.Group.Subgroup`
- `Mathlib.Algebra.Group.Quotient`
- `Mathlib.Algebra.Group.Quotient.Mul`
- `Mathlib.Algebra.Group.Quotient.Group`
- `Mathlib.Algebra.Group.SecondIsomorphism.Map`
- `Mathlib.Algebra.Group.SecondIsomorphism.Kernel`
- `Mathlib.Algebra.Group.SecondIsomorphism.Image`
- `Mathlib.Algebra.Group.SecondIsomorphism`

The downstream theorem should consume `second_isomorphism_theorem_evidence`,
not only `SecondIsoPhi` or a kernel/image helper, so the smoke fixture proves
the final theorem-evidence route is source-free importable.

## Deferred Candidates

The following nearby routes are intentionally not selected for Layer 3D-E:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Third isomorphism | `Proofs.Ai.Algebra.AbstractGroupThirdIso` | Larger nested quotient route. It also introduces quotient-subgroup and kernel-evidence surfaces that should be audited separately after the second-isomorphism API is public. |
| Correspondence | `Proofs.Ai.Algebra.AbstractGroupCorrespondence*` | Depends on normal quotient plus subgroup order and saturation/preimage/image APIs. Its API is broader than the second-isomorphism theorem evidence selected here. |
| Ring isomorphism and CRT | `Proofs.Ai.Algebra.AbstractRingFirstIso*`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Depends on group quotient/isomorphism plus ring API decisions. Keep in later algebra layers. |
| Geometry and analysis routes | `Proofs.Ai.Geometry.*`, `Proofs.Ai.Analysis.*`, `Proofs.Ai.LinearAlgebra.*`, `Proofs.Ai.FunctionalAnalysis.*` | Not part of the group isomorphism closure sequence. |

## Axiom Policy

Layer 3D-E does not widen the public axiom policy beyond the `v0.1.10`
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

The generated corpus axiom report records for the selected modules:

- direct axioms: none
- transitive axioms: `Eq.rec`
- policy status: ok
- policy violations: none

The generated AI theorem index records 16 theorem entries for the selected
modules. The theorem index is an untrusted sidecar; proof acceptance remains
based on canonical certificate bytes and verifier results.

## Hash Inputs

These checked-in corpus hashes are the audit inputs. Public materialization
must regenerate hashes after renaming to `Mathlib.*`.

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi` | `sha256:2b4bdb19544a4a2249342a6ee00361d01af6c740c4b893f62db76d775ad67ac5` | `sha256:414199676f78007f2aaf93ae9a57d1dfb14721ccc32bf28f9b7d9fc5741fa3db` | `sha256:d94e37947265642c849691c92b1064aa95688992bf0535367df27161385c3984` | `sha256:be1a77ed10a39666141513d10276a53768c73a3c2f624e2abac75c22ae46554f` | `sha256:e491b3a4a6c32989573f3ef140feb02ae10d5a100af14780b61cd150ed518898` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel` | `sha256:11b4377a2ea0acb91af664f2a719c62dd5864b3688418c6592d4f3b772d04275` | `sha256:d96838e22596285945272f83c4a89c20055cb753ed22a29890b9f83dc8d09527` | `sha256:7a245cb5542167151a9835c83e0906b5a4591deb2e6ad479cc228665b38b4778` | `sha256:1f6f1e5c334718f52501f6e5180cb7bbb7ef35771fae3310d147efdb3cdac988` | `sha256:990807b0f54eb7d427eee40433491245af4fb3d2d28e53861309366ad64bdcd4` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage` | `sha256:bd586d575e033e1cc68f9803e32fe71832a64c1a506f9afde14c61ef599afc44` | `sha256:55a0d4c6f1c5da18e630cbb19f699f9a720358ed48f95c47f5d71fc620f48956` | `sha256:48412f9f4a2c28037d1fd1df932ebdd35bdc57da98f37f28a1b467d337ca45e0` | `sha256:e223c3fab423c0c22975a4f6493b66825b610b147c89f817df22b9119a74a170` | `sha256:a9e7a72b59b222116800210609e625cda4044bb3d517b9880b405e4400d020ee` |
| `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal` | `sha256:603244f923dba61de97e74c9992b0f5334c49117f2f0e048803fedbdf04fe6b0` | `sha256:b711ddffe6e3e3a9d5129876f3ebe82d925f323d73cef970958f4919bcd72879` | `sha256:9b3ada0115a732aa31cca384b796773b363cd0258f0002bfdbe3a2c29c6de488` | `sha256:bf5a5d3525044aca58e5ef80bc5fd13035d13b7a339decccf8fea603d7f379e3` | `sha256:9f306516ca5dd5313fe66b7245d1c906d06922a8072106273bd54c5fb255ce4f` |

The local `npa-mathlib v0.1.10` materialization records these corresponding
carried-forward public import hashes:

| Public import module | Export hash | Certificate hash |
| --- | --- | --- |
| `Std.Logic.Eq` | `sha256:b78b442d5f593458cc12f079b59e3c70259c0bec4967511e1a01d262d2f0e874` | `sha256:5a5b68a51b3e90223f1e0cca730f8b155c79f881b9dca70d67e6bf10058054aa` |
| `Mathlib.Logic.EqReasoning` | `sha256:67f90711ce596378579688b337552c3ae555aada85f97c5d40eab2381e2d1679` | `sha256:5f4d2c7abdf117a41633f904bc11345963ee8c36cd7ea1cfc0d8369657a22bad` |
| `Mathlib.Algebra.Group.Basic` | `sha256:36a59f49575ead1441d64314b9f301f159d391e5dc159c874fe2e7c89416db5f` | `sha256:ae0e5ac36b7f4c2729fb4f202627afd575763927db61e88f330b7a245c185756` |
| `Mathlib.Algebra.Group.Subgroup` | `sha256:2b527c5b3039075812d564522facb4a41e421e1902f863df111f0e83c5b9d85c` | `sha256:b3b47c9d314d69594d1c80e24fd4d1954c242ef289e2031207be9acd798ce2a9` |
| `Mathlib.Algebra.Group.Quotient` | `sha256:14d52188e958a89a9f87ad32a12c2738a73c0e185d68bcd390875bf4dea7f4de` | `sha256:6fc7c518e7eaa04e792bb1d4dd3af7b4f5c4987136a118ce98a75d601c4feb19` |
| `Mathlib.Algebra.Group.Quotient.Mul` | `sha256:9570dfe4c8b6ca61fe53acd2bf41cf193be20a4d9118cbc295f1c1a10d602cc7` | `sha256:5495b3aba0eb579efd418b5a86fad16765c2f21889c9531747fa0d152f774013` |
| `Mathlib.Algebra.Group.Quotient.Group` | `sha256:fb6bb1472ee946b6db2174ccf4cf845b0a7dda6c8c42ccc9396950a5a8a92654` | `sha256:41a716e2220952290f35148b55d9505f1c8f7b90491233c10ee704bb5b678ef1` |

## Verification

The checked-in corpus certificates passed source-free verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSecondIsoImage
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi`: verified 1 selected module,
  8 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel`: verified 1 selected
  module, 9 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage`: verified 1 selected
  module, 9 modules including dependency cache.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal`: verified 1 selected
  module, 11 modules including dependency cache.

The source-to-certificate authoring path also regenerated the closures:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSecondIsoImage
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal
```

Result:

- `Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi`: built 7 modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel`: built 8 modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage`: built 8 modules including
  import closure.
- `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal`: built 10 modules including
  import closure.

The build checks rewrote `proofs/generated/ai-theorem-index.json`, but the
checked-in bytes did not change.

## Materialization Evidence

Layer 3D-E was materialized locally in `/Users/kazuyoshitoshiya/ff/npa-mathlib`
as `npa-mathlib v0.1.11`.

Before the `v0.1.11` materialization, local `v0.1.10` release-bundle evidence
was generated under `target/release-artifacts/`:

- bundle: `npa-mathlib-v0.1.10-release-artifacts.tar.gz`
- bundle SHA-256:
  `360a5b0dec8627c99eaaeb27e86b9f7e445494185516bfcb344dbbac544be603`

The public materialization generated these hashes:

| Public module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Mathlib.Algebra.Group.SecondIsomorphism.Map` | `sha256:bf167686c871620f655081c61a536a7538eb2f9d327e30de91d06e33bfde3ae4` | `sha256:2114f1a9368313eaefcb31079e9c02c44e4a1def10aa0a22c73027aa434dfc98` | `sha256:7ee14328dc55734480495e5b6bdc07124481d60ac50ea2cd2cfe0ae1af1ee6d7` | `sha256:be1a77ed10a39666141513d10276a53768c73a3c2f624e2abac75c22ae46554f` | `sha256:d9fca8faf318fd62e6ae8e45c9f1ebfa9a982364ea7d105d2be495382166be34` |
| `Mathlib.Algebra.Group.SecondIsomorphism.Kernel` | `sha256:a42ee04d7c4e8ace9ef454d16ed178c4fb661201552a88b727fcf001d7cf6148` | `sha256:a380515b9fe711dc33de2c3f579a7a929eab8a7c2b7e1c9bf114475532720404` | `sha256:3466bcdd983128af0e213dd9b2ce47e17b95e5a3da110dd3d9701801a781423c` | `sha256:1f6f1e5c334718f52501f6e5180cb7bbb7ef35771fae3310d147efdb3cdac988` | `sha256:bba7002ca527e90934c0d392fd438a0b5e60a835fe5bbb4a8fbd2ccbdda73e8f` |
| `Mathlib.Algebra.Group.SecondIsomorphism.Image` | `sha256:16fba1a4324b21865d566b8bf9085ed7ad2f4e558ea1eaa2ee40616d5ee37299` | `sha256:53383f558f81ca4d1b1befad3b73417751d1e87a8f1d049c3689568812d17a92` | `sha256:46ca9c9bc448f734b59cb23ec561f9792a086a5cb7c24b51289927b2df28b674` | `sha256:e223c3fab423c0c22975a4f6493b66825b610b147c89f817df22b9119a74a170` | `sha256:3c5d1a2421041b1cdb3e8b7e2465487155389e98b426107c055da7db37efb623` |
| `Mathlib.Algebra.Group.SecondIsomorphism` | `sha256:0e1ab3a17ab0c45a727fc47981340aa05615aa29beb3fa85428fbe7878336f2c` | `sha256:aa0ca6ce399b9c5534dd5bd434d885541b014bb62234116673aaf4986b5a3fc3` | `sha256:15bf8ddefccbafa5f0616331435775e8ce2a5f9db6023d5793038da2a810babb` | `sha256:bf5a5d3525044aca58e5ef80bc5fd13035d13b7a339decccf8fea603d7f379e3` | `sha256:63991024f404cf1ff14d22f340af79682b96c2721549b2983d296a8349df4424` |

Local `v0.1.11` release-bundle evidence was generated under
`target/release-artifacts/`:

- bundle: `npa-mathlib-v0.1.11-release-artifacts.tar.gz`
- bundle SHA-256:
  `84826c82142ad1b601ee15b36a2f87fd6897403a253cf7f32d7e12dc7c4d4532`

The downstream smoke fixture was updated to consume
`second_isomorphism_theorem_evidence` through
`Downstream.GroupSecondIsomorphism::second_isomorphism_theorem_evidence_passthrough`.

Downstream smoke hashes:

| Downstream module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Downstream.GroupSecondIsomorphism` | `sha256:31b87bec703dfe1ecd8262b10736be751e08fe8965691b225b02d4033aaaa4db` | `sha256:67e5a42067ad574e5d6d215e86e6ec686c6f25338e2b8b20475c4726f6bb6586` | `sha256:0230a5a0a1b722da975373ddf60148d732f48a2454731b602ee016246b2c4b14` | `sha256:f5256bf35f9e2e0e8dce219f32faf81b992fdade65fed2defa8d81b4bd5bf5f9` | `sha256:a9a61beeae511d32aec739d1a959629850c4fdfd9f7c253fdb7b7418c42d4bb2` |

Package gates passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
```

Downstream gates passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

Negative checks on a temporary downstream copy rejected:

- bad `Mathlib.Algebra.Group.SecondIsomorphism.Map` export hash:
  `export_hash_mismatch`
- bad `Mathlib.Algebra.Group.SecondIsomorphism` certificate hash:
  `certificate_hash_mismatch`
- corrupted vendored `Mathlib.Algebra.Group.SecondIsomorphism` certificate
  bytes: `certificate_decode_failed`
- bad imported `Mathlib.Algebra.Group.SecondIsomorphism` package version:
  `package_lock_stale`

Tag creation, push, and GitHub release publication were not performed.

## Decision

Layer 3D-E is materialized as `npa-mathlib v0.1.11` with exactly these new public
modules:

```text
Mathlib.Algebra.Group.SecondIsomorphism.Map
Mathlib.Algebra.Group.SecondIsomorphism.Kernel
Mathlib.Algebra.Group.SecondIsomorphism.Image
Mathlib.Algebra.Group.SecondIsomorphism
```

Do not split this closure further: publishing only `Map`, `Kernel`, or `Image`
would leave downstream users without the final
`second_isomorphism_theorem_evidence` route. Do not include third isomorphism
or correspondence theorem modules in the same release.

Release publication must:

1. Review the local `v0.1.11` package and downstream evidence.
2. Review the generated `v0.1.11` release bundle.
3. Publish `npa-mathlib v0.1.11` only after package, downstream, and
   release-bundle evidence are fixed.
