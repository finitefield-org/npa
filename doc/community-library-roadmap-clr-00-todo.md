# Community Library Roadmap CLR-00 Todo

Source: `doc/community-library-roadmap-todo.md` CLR-00

CLR-00 fixes the package command, crate placement, schema names, manifest field mapping,
and import semantics that CLR-01 and later milestones depend on. This document turns that
decision milestone into implementation-ready specification tasks.

---

## Scope

対象:

```text
- contributor-facing package command contract
- Cargo package / binary placement for the package CLI
- package manifest, lock, report, index, and publish metadata schema names
- field mapping from `npa-ai-proof-corpus-v0.1` to `npa.package.v0.1`
- required / optional / forbidden manifest fields
- package-level imports vs module-level imports semantics
- trusted-boundary notes that later implementation must preserve
```

非対象:

```text
- manifest parser / validator implementation
- package graph verification implementation
- package build / verify command implementation
- generated package artifacts
- registry server
- package dependency solver
- network fetch, binary cache, or implicit latest-version resolution
```

Trusted-boundary rule:

```text
`npa-package` and `npa-cli` are untrusted orchestration layers.
The kernel, certificate verifier, and source-free checker must not gain filesystem,
network, registry lookup, plugin loading, AI calls, or package-manager behavior.
```

---

## Fixed Implementation Decisions

### Command Contract

Contributor-facing command:

```sh
npa package check
npa package build-certs
npa package verify-certs
npa package check-hashes
npa package axiom-report
npa package index
npa package publish-plan
```

Cargo development command:

```sh
cargo run -p npa-cli -- package check --root proofs
```

Implementation placement:

```text
crates/npa-cli
  Cargo package name: npa-cli
  binary name: npa
  responsibility: CLI parsing, filesystem access, package command orchestration

crates/npa-package
  Cargo package name: npa-package
  library crate name: npa_package
  responsibility: package manifest data model, schema constants, validation helpers,
                  field mapping, deterministic package metadata helpers
```

The installed command documented for contributors is `npa`. The Cargo package used by
repository verification commands is `npa-cli`.

### Schema Names

Target `npa-package` Rust constants:

| Constant | Schema string | Primary artifact | Boundary |
| --- | --- | --- | --- |
| `PACKAGE_MANIFEST_SCHEMA` | `npa.package.v0.1` | `npa-package.toml` | Package input manifest, not checker evidence. |
| `PACKAGE_LOCK_SCHEMA` | `npa.package.lock.v0.1` | `generated/package-lock.json` | Generated package metadata; source for checker-run import lock derivation, not checker evidence. |
| `PACKAGE_AXIOM_REPORT_SCHEMA` | `npa.package.axiom_report.v0.1` | `generated/axiom-report.json` | Generated policy/report metadata, not checker evidence. |
| `PACKAGE_THEOREM_INDEX_SCHEMA` | `npa.package.theorem_index.v0.1` | `generated/theorem-index.json` | Generated search/docs metadata, not checker evidence. |
| `PACKAGE_PUBLISH_PLAN_SCHEMA` | `npa.package.publish_plan.v0.1` | `generated/publish-plan.json` | Generated release/registry upload plan, not checker evidence. |
| `REGISTRY_MODULE_SCHEMA` | `npa.registry.module.v0.1` | module registry entry | Distribution/search metadata, not trusted base. |

Existing Phase 8 schemas remain separate:

| Existing constant | Schema string | Boundary |
| --- | --- | --- |
| `INDEPENDENT_CHECKER_IMPORT_LOCK_MANIFEST_SCHEMA` | `npa.independent-checker.import_lock_manifest.v1` | Source-free checker input derived per checker run. |
| `INDEPENDENT_CHECKER_CHECKER_BINARY_REGISTRY_SCHEMA` | `npa.independent-checker.checker_binary_registry.v1` | Runner-local checker executable configuration. |

`npa.package.lock.v0.1` is the package-level import/package lock artifact. The
independent-checker import lock is a source-free checker input derived from package
metadata, not the same public package lock schema.
No `npa.package.*` schema string is reused for `npa.independent-checker.*` artifacts,
and no `npa.independent-checker.*` schema string is reused for public package artifacts.

### Current Proof Corpus Compatibility

`proofs/manifest.toml` remains a legacy proof-corpus artifact with:

```toml
schema = "npa-ai-proof-corpus-v0.1"
```

CLR-00 does not rename or migrate it. CLR-02 will introduce the package fixture for
the current proof corpus at:

```text
proofs/npa-package.toml
```

After CLR-02, `proofs/npa-package.toml` is the package fixture used by package commands,
while `proofs/manifest.toml` remains a compatibility artifact until the proof-corpus
tool is retired or rewritten.

### Target Manifest Shape

External theorem libraries use `npa-package.toml` at the package root. CLR-02 will
materialize the in-repo proof corpus fixture at `proofs/npa-package.toml`; the example
below is the target shape for the current `Proofs.Ai.Basic` module using the checked-in
legacy manifest's actual hash values.
The complete `proofs/npa-package.toml` created in CLR-02 repeats this `[[modules]]`
shape for every current `[[proof_modules]]` entry and adds hash-pinned top-level
`[[imports]]` entries for external modules such as `Std.Logic.Eq` and `Std.Nat.Basic`.

```toml
schema = "npa.package.v0.1"
package = "npa-proof-corpus"
version = "0.1.0"
license = "MIT"

core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"

[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]

[[modules]]
module = "Proofs.Ai.Basic"
source = "Proofs/Ai/Basic/source.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
meta = "Proofs/Ai/Basic/meta.json"
replay = "Proofs/Ai/Basic/replay.json"
producer_profile = "human-surface-explicit-term"

imports = []
expected_source_hash = "sha256:28330ae585898b77be110adcdd53fe50e7f141a54113f12e6af9143fa4fcf54e"
expected_certificate_file_hash = "sha256:464a0d224b8667e4870888522454782231cd2cdd9049e6fa930cbefa62c18ffc"
expected_export_hash = "sha256:3341d28e9d1d9dd875138399ab1bd7aa6e2727449cb87fe03c73b220c4b231c0"
expected_axiom_report_hash = "sha256:fed11e73accfbfb0dfc28b4f510e151fa33d8af82d58fdb23b92567e04e59e40"
expected_certificate_hash = "sha256:69cb8c64c6ce722209e27820cd790af6d325c98478b3599ae796ee03df528b13"

inductives = []
definitions = []
theorems = [
  "id",
  "const_left",
  "const_right",
  "apply_fn",
  "compose",
  "flip",
  "duplicate",
  "prop_id",
  "modus_ponens",
  "imp_trans",
  "compose_assoc",
  "apply_twice",
  "ignore_middle",
  "select_middle",
  "select_last",
  "imp_swap",
  "imp_compose",
  "imp_ignore",
  "imp_duplicate",
  "higher_apply",
]
axioms = []
```

### Required, Optional, Forbidden, And Generated Fields

Top-level required fields in `npa.package.v0.1`:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | string | Must equal `npa.package.v0.1`. |
| `package` | string | Package identity, using package-name grammar fixed in CLR-01. |
| `version` | string | Exact package version string; no implicit latest resolution. |
| `core_spec` | string | Core spec profile, e.g. `npa.core.v0.1`. |
| `kernel_profile` | string | Kernel compatibility profile, e.g. `npa.kernel.v0.1`. |
| `certificate_format` | string | Certificate format, e.g. `npa.certificate.canonical.v0.1`. |
| `checker_profile` | string | Required checker profile, e.g. `npa.checker.reference.v0.1`. |
| `policy` | table | Package axiom policy object. |
| `modules` | non-empty array of tables | Local module entries. |

Top-level optional fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `license` | string | Package license expression. |
| `repository` | string | Informational source repository URL. |
| `description` | string | Informational package description. |
| `imports` | array of tables | Hash-pinned external package/module imports. |

Top-level forbidden fields:

```text
trusted_status
verified_by_certificate
checker_result
checker_results
registry_url
latest
generated_at
```

Policy object required fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `allow_custom_axioms` | bool | Whether axioms outside `allowed_axioms` may appear. |
| `allowed_axioms` | array of strings | Exact axiom names permitted by package policy. |

Policy object optional fields in `npa.package.v0.1`:

```text
none
```

Package import `[[imports]]` required fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `module` | string | External module name. |
| `package` | string | External package name. |
| `version` | string | Exact external package version. |
| `certificate` | string | Package-relative path to vendored external certificate. |
| `export_hash` | hash string | Exact canonical export hash for the external module. |
| `certificate_hash` | hash string | Exact canonical certificate hash for high-trust identity. |

Package import optional fields in `npa.package.v0.1`:

```text
none
```

Module required fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `module` | string | Local module name. |
| `source` | string | Package-relative source path. |
| `certificate` | string | Package-relative certificate path. |
| `imports` | array of strings | Direct module imports, resolved by CLR-00-05 rules. |
| `expected_source_hash` | hash string | SHA-256 of source file bytes. |
| `expected_certificate_file_hash` | hash string | SHA-256 of certificate file bytes. |
| `expected_export_hash` | hash string | Canonical export hash from the certificate. |
| `expected_axiom_report_hash` | hash string | Canonical axiom report hash from the certificate. |
| `expected_certificate_hash` | hash string | Canonical certificate hash from the certificate. |

Module optional fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `meta` | string | Optional untrusted metadata path. |
| `replay` | string | Optional untrusted replay path. |
| `producer_profile` | string | Optional producer profile metadata. |
| `inductives` | array of strings | Optional declaration summary. |
| `definitions` | array of strings | Optional declaration summary. |
| `theorems` | array of strings | Optional declaration summary. |
| `axioms` | array of strings | Optional declaration summary checked against policy. |
| `tags` | array of strings | Optional search/docs metadata. |

Module forbidden fields:

```text
trusted_status
verified_by_certificate
checker_result
checker_results
registry_url
latest
generated_at
source_sha256
certificate_file_sha256
export_hash
axiom_report_hash
certificate_hash
```

The legacy fields `source_sha256`, `certificate_file_sha256`, `export_hash`,
`axiom_report_hash`, and `certificate_hash` are forbidden inside `[[modules]]`
because the target manifest uses the `expected_*` names there. The plain
`export_hash` and `certificate_hash` names remain valid only inside top-level
`[[imports]]`, where they describe external module identity.

Forbidden in every `npa.package.v0.1` object:

```text
trusted_status
verified_by_certificate
checker_result
checker_results
registry_url
latest
generated_at
```

Unknown fields in the top-level object, `[policy]`, each `[[imports]]` object, and
each `[[modules]]` object are rejected in `npa.package.v0.1`. There is no `x-*`,
`extra`, or `metadata` escape hatch in v0.1. Forward-compatible extension fields
require a new schema version. Duplicate keys are rejected by parsing or validation
before package graph checks begin.

Generated artifacts, not manifest input:

```text
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
generated/publish-plan.json
checker result JSON
release audit bundle
verified_high_trust artifact
```

Generated artifacts may support review, search, publishing, or checker orchestration.
They must not become proof acceptance evidence.
The proof acceptance boundary remains canonical certificate plus Rust kernel or
source-free checker verdict. Generated package metadata can make a CI job fail when it
is stale or inconsistent, but it cannot make an unchecked certificate accepted.

### Legacy Field Mapping

Compatibility decision:

```text
`proofs/manifest.toml` remains a checked-in legacy proof-corpus artifact with
schema `npa-ai-proof-corpus-v0.1`.

CLR-02 adds `proofs/npa-package.toml` beside it. The new file is the package fixture
used by package commands and uses schema `npa.package.v0.1`.

Do not migrate `proofs/manifest.toml` in place during CLR-02. Keep it until
`tools/proof-corpus` is retired or rewritten to emit only the package manifest.
```

Mapping from `proofs/manifest.toml` to `npa.package.v0.1`:

| Legacy field | Target field or artifact | Classification | Notes |
| --- | --- | --- | --- |
| `schema` | `schema` | target | Change from `npa-ai-proof-corpus-v0.1` to `npa.package.v0.1`. |
| missing | `package` | generated fixture default | Use `npa-proof-corpus` for the in-repo fixture until release metadata exists. |
| missing | `version` | generated fixture default | Use `0.1.0` until release tagging is introduced. |
| missing | `license` | generated fixture default | Use workspace license `MIT` for the in-repo fixture. |
| missing | `core_spec` | generated fixture default | Use `npa.core.v0.1`. |
| missing | `kernel_profile` | generated fixture default | Use `npa.kernel.v0.1`. |
| missing | `certificate_format` | generated fixture default | Use `npa.certificate.canonical.v0.1`. |
| missing | `checker_profile` | generated fixture default | Use `npa.checker.reference.v0.1`. |
| missing | `[policy]` | generated fixture default | Use `allow_custom_axioms = false` and `allowed_axioms = ["Eq.rec"]` for the current corpus. |
| missing | `[[imports]]` | generated from module imports plus external artifacts | Add one hash-pinned top-level entry for each external module name used by `modules[].imports`, such as `Std.Logic.Eq` and `Std.Nat.Basic`. |
| `[[proof_modules]]` | `[[modules]]` | target | Rename table family without reordering modules unless CLR-02 explicitly chooses deterministic sorting. |
| `module` | `modules[].module` | target | Preserve dotted module name exactly. |
| `source` | `modules[].source` | target | Preserve package-relative source path exactly. |
| `certificate` | `modules[].certificate` | target | Preserve package-relative certificate path exactly. |
| `meta` | `modules[].meta` | target optional | Preserve optional untrusted metadata path when present. |
| `replay` | `modules[].replay` | target optional | Preserve optional untrusted replay path when present. |
| `producer_profile` | `modules[].producer_profile` | target optional | Preserve optional build-helper metadata. |
| `trusted_status` | generated checker result artifact only | forbidden in package manifest | `verified_by_certificate` is a legacy status string, not proof evidence. Package manifest must reject it. |
| `source_sha256` | `modules[].expected_source_hash` | target renamed field | Required source file byte hash check input. |
| `certificate_file_sha256` | `modules[].expected_certificate_file_hash` | target renamed field | Required certificate file byte hash check input. |
| `export_hash` | `modules[].expected_export_hash` | target renamed field | Required canonical export identity check. |
| `axiom_report_hash` | `modules[].expected_axiom_report_hash` | target renamed field | Required axiom report identity check. |
| `certificate_hash` | `modules[].expected_certificate_hash` | target renamed field | Required high-trust certificate identity check. |
| `imports` | `modules[].imports` plus top-level `[[imports]]` for external modules | target plus generated external import metadata | Preserve the module string list. Local names resolve to `[[modules]]`; external names require hash-pinned top-level import entries. |
| absent `inductives` | omitted `modules[].inductives` or `[]` | target optional default | Treat absence as an empty declaration summary. |
| `inductives` | `modules[].inductives` | target optional | Preserve declaration summary exactly when present. |
| `definitions` | `modules[].definitions` | target optional | Preserve declaration summary exactly. |
| `theorems` | `modules[].theorems` | target optional | Preserve declaration summary exactly. |
| `axioms` | `modules[].axioms` | target optional | Preserve declaration summary exactly and check against `[policy]` and generated axiom report. |

Generated or derived artifacts from the mapping:

```text
proofs/npa-package.toml
  package fixture generated from the legacy manifest plus fixed CLR-00 defaults

proofs/vendor/npa-std/**
  external Std import certificates introduced by CLR-02, not present in the legacy manifest

proofs/generated/package-lock.json
  generated later from `npa-package.toml`, not an input to the manifest mapping

checker result JSON / release audit bundle / verified_high_trust artifact
  generated status and audit outputs; never copied into `npa-package.toml`
```

The mapping must be data-driven from checked-in artifacts. CLR-02 may read
`proofs/manifest.toml`, current proof files, certificate files, and vendored external
import artifacts, but the resulting `proofs/npa-package.toml` must not require reading
Rust constants from `tools/proof-corpus/src/main.rs` at package validation time.

### Import Semantics

Package-level `[[imports]]` defines external modules made available to this package.
Each entry must include:

```text
module
package
version
export_hash
certificate_hash
certificate
```

Module-level `imports = [...]` lists the modules a module depends on.
In the current proof corpus, `Proofs.Ai.*` names are same-package imports and
`Std.Logic.Eq` / `Std.Nat.Basic` are external imports that must become top-level
hash-pinned `[[imports]]` entries in `proofs/npa-package.toml`.

Resolution rules:

```text
1. Build a local module map from `[[modules]].module`.
   Duplicate local module names fail before resolving any module import.
   Report the smallest later duplicate module entry deterministically.

2. Build an external import map from top-level `[[imports]].module`.
   Duplicate external module names fail before resolving any module import.
   Report the smallest later duplicate external import entry deterministically.

3. If a module name appears in both maps, validation fails before resolving any
   `modules[].imports` list. Report the smallest colliding top-level `[[imports]]`
   entry. A package cannot shadow a local module with an external import.

4. For each `modules[].imports` string, first resolve it against the local module map.
   If it matches a local `[[modules]]` entry, it is a same-package import.
   Its identity is the target module's `expected_export_hash`,
   `expected_axiom_report_hash`, `expected_certificate_hash`,
   `expected_certificate_file_hash`, and `certificate` path.

5. If the import string does not match a local module, resolve it against the external
   import map. If it matches a top-level `[[imports]]` entry, it is an external import.
   Its identity is the top-level entry's `package`, `version`, `export_hash`,
   `certificate_hash`, and vendored `certificate` path. External imports are never
   accepted by module name alone.

6. If the import string matches neither map, validation fails as an unknown import.
   The validator must not search the filesystem, registry, network, package cache, or
   installed packages to resolve it.

7. Registry lookup, network fetch, package-cache fallback, directory scan fallback, or
   implicit latest-version resolution is forbidden during manifest validation,
   package-lock generation, and checker-request materialization.
```

The package lock generated later must materialize each resolved direct import with at least:

```text
module
kind = same-package | external
package identity for external imports
export_hash
certificate_hash
certificate path
certificate file hash
axiom_report_hash when available
```

Same-package imports may stay concise in `modules[].imports` because the referenced
local `[[modules]]` entry carries the expected hashes. This does not weaken hash
pinning: `npa.package.lock.v0.1`, `check-hashes`, and CLR-03 source-free verification
must use the target module's expected hashes, not only the module string.

When deriving `npa.independent-checker.import_lock_manifest.v1` for a source-free
checker run, package tooling uses the resolved package-lock identity. It emits the
Phase 8 shape `module`, `export_hash`, `certificate.path`,
`certificate.file_hash`, and `certificate.certificate_hash`; it does not read source,
replay, theorem index, registry metadata, or AI sidecars to justify an import.

---

## Tasks

### CLR-00-01 Record CLI And Crate Topology Decision

- Status: Completed
- Depends on: None
- Inputs:
  - `doc/community-library-roadmap-todo.md` CLR-00
  - root `Cargo.toml`
  - existing `tools/proof-corpus/Cargo.toml`
  - existing `crates/npa-checker-ref/Cargo.toml`
- Deliverables:
  - Documentation update confirming `crates/npa-cli` and `crates/npa-package`.
  - Verification command examples using `cargo run -p npa-cli -- package ...`.
  - Contributor examples using installed `npa package ...`.
- Acceptance criteria:
  - Later milestones no longer use the old package CLI placeholder.
  - The `npa` binary name and `npa-cli` Cargo package name are both documented.
  - No kernel, certificate, or checker trusted crate is assigned package CLI responsibilities.
- Verification:
  - `rg -n "PACKAGE_""CLI_CRATE|npa-cli|npa package" doc/community-library-roadmap-todo.md doc/community-library-roadmap-clr-00-todo.md README.md`
  - `git diff --check`
- Notes:
  - This task is documentation-only unless a later implementation milestone starts the crates.
  - Implemented by documenting the `npa` binary, `npa-cli` Cargo package, future
    `crates/npa-package` library crate, and trusted-boundary placement in `README.md`.
  - No workspace members are added by CLR-00-01.

### CLR-00-02 Define Schema Constant Names And Artifact Boundaries

- Status: Completed
- Depends on: CLR-00-01
- Inputs:
  - Phase 8 schema constants in `crates/npa-api/src/independent_checker.rs`
  - registry sketch in `doc/community-library-roadmap.md`
- Deliverables:
  - Stable schema names for package manifest, package lock, axiom report, theorem index, publish plan, and module registry entry.
  - Explicit note separating package lock from independent-checker import lock.
  - Explicit note that generated metadata is not checker evidence.
- Acceptance criteria:
  - Package schemas do not reuse independent-checker schema names.
  - The schema names are short enough to become Rust constants in `npa-package`.
  - The document states which generated artifacts are non-input metadata.
- Verification:
  - `rg -n "npa\\.package\\.|independent-checker\\.import_lock|npa\\.registry\\.module\\.v0\\.1" doc/community-library-roadmap-clr-00-todo.md doc/community-library-roadmap.md doc/phase8-ai.md`
  - `git diff --check`
- Notes:
  - `verified_high_trust` remains target integration outside CLR-00.
  - Implemented by fixing `npa-package` target constants, separating
    `npa.package.lock.v0.1` from `npa.independent-checker.import_lock_manifest.v1`,
    and documenting generated package metadata as non-evidence.

### CLR-00-03 Specify `npa.package.v0.1` Manifest Field Contract

- Status: Completed
- Depends on: CLR-00-02
- Inputs:
  - `doc/community-library-roadmap.md` section 5.1
  - `proofs/manifest.toml`
  - `tools/proof-corpus/src/main.rs`
- Deliverables:
  - Required / optional / forbidden top-level field list.
  - Required / optional / forbidden module field list.
  - Unknown-field rejection rule for every manifest object.
  - Example manifest for the current proof corpus fixture.
  - Decision that `proofs/npa-package.toml` becomes the package fixture path in CLR-02.
- Acceptance criteria:
  - CLR-01 can implement parser structs without choosing field names.
  - CLR-01 can reject unknown fields without deciding extension policy.
  - The source hash and certificate file hash are included, not only canonical certificate hashes.
  - `trusted_status` is explicitly rejected from the target manifest.
  - The manifest does not contain checker verdicts or registry fetch instructions.
- Verification:
  - `rg -n "expected_source_hash|expected_certificate_file_hash|trusted_status|proofs/npa-package.toml" doc/community-library-roadmap-clr-00-todo.md doc/community-library-roadmap-todo.md`
  - `git diff --check`
- Notes:
  - The target manifest is package input. Generated status belongs in checker or publish artifacts.
  - Implemented by expanding the field contract into top-level, policy, import, and
    module object fields; adding the current `Proofs.Ai.Basic` fixture shape with
    `expected_source_hash` and `expected_certificate_file_hash`; and forbidding
    legacy checker/status fields from manifest input.

### CLR-00-04 Specify Legacy Manifest Mapping

- Status: Completed
- Depends on: CLR-00-03
- Inputs:
  - `proofs/manifest.toml`
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
  - `doc/community-library-roadmap-todo.md` CLR-02
- Deliverables:
  - Field-by-field mapping table from `npa-ai-proof-corpus-v0.1` to `npa.package.v0.1`.
  - Compatibility decision for keeping `proofs/manifest.toml`.
  - Notes for fields that become generated or forbidden.
- Acceptance criteria:
  - Every existing proof-corpus manifest field has a target, generated, or forbidden classification.
  - The mapping preserves current module names, paths, hashes, declaration summaries, and axiom lists.
  - No target field requires reading Rust constants from `tools/proof-corpus/src/main.rs`.
- Verification:
  - `rg -n "npa-ai-proof-corpus-v0.1|expected_export_hash|expected_axiom_report_hash|trusted_status" doc/community-library-roadmap-clr-00-todo.md proofs/manifest.toml`
  - `git diff --check`
- Notes:
  - CLR-02 will use this mapping to build the proof-corpus package fixture.
  - Implemented by classifying each legacy proof-corpus manifest field as target,
    generated/defaulted, or forbidden, and by recording that `proofs/manifest.toml`
    remains beside the future `proofs/npa-package.toml` fixture.

### CLR-00-05 Specify Import Resolution Semantics

- Status: Completed
- Depends on: CLR-00-03
- Inputs:
  - existing `imports` entries in `proofs/manifest.toml`
  - Phase 4 and Phase 8 import identity docs
  - `IndependentCheckerImportLockManifest` in `crates/npa-api/src/independent_checker.rs`
- Deliverables:
  - Local module import rule.
  - External top-level `[[imports]]` rule.
  - Duplicate local/external module rejection rule.
  - Rule forbidding module-name-only external imports.
  - Rule forbidding registry/network resolution during validation.
- Acceptance criteria:
  - CLR-01 can implement module graph validation without inventing import behavior.
  - CLR-03 can derive source-free checker import locks from the same semantics.
  - A module name collision between local and external imports has a deterministic failure.
  - Same-package imports can remain concise without weakening external hash pinning.
- Verification:
  - `rg -n "module name alone|same-package import|external import|registry lookup" doc/community-library-roadmap-clr-00-todo.md doc/community-library-roadmap.md doc/phase4-ai.md doc/phase8-ai.md`
  - `git diff --check`
- Notes:
  - Package-level import resolution is untrusted helper logic; checker identity remains certificate/hash based.
  - Implemented by defining local and external import maps, deterministic duplicate
    and collision failures, module-level resolution order, and the package-lock /
    Phase 8 import-lock derivation boundary.

### CLR-00-06 Define Validator Error And Test Surface For CLR-01

- Status: Pending
- Depends on: CLR-00-03, CLR-00-05
- Inputs:
  - manifest field contract from CLR-00-03
  - import semantics from CLR-00-05
  - repository test conventions in `AGENTS.md`
- Deliverables:
  - List of validator cases CLR-01 must cover.
  - Expected structured-error categories for schema, domain, path, duplicate, graph, hash, and policy failures.
  - Fixture names for valid and invalid package manifests.
- Acceptance criteria:
  - CLR-01 tests can be written before the parser implementation is complete.
  - Errors are testable enum-like categories, not human text only.
  - Validation failure order is deterministic enough for golden assertions.
- Verification:
  - `rg -n "schema failure|domain failure|duplicate|import cycle|hash grammar|axiom policy" doc/community-library-roadmap-clr-00-todo.md`
  - `git diff --check`
- Notes:
  - Suggested fixture roots: `tests/fixtures/package/valid` and `tests/fixtures/package/invalid`.

### CLR-00-07 Update Parent Roadmap Commands

- Status: Pending
- Depends on: CLR-00-01
- Inputs:
  - `doc/community-library-roadmap-todo.md`
  - `doc/community-library-roadmap.md`
  - `README.md`
- Deliverables:
  - Parent task doc references this CLR-00 detailed task document.
  - Old package CLI placeholder examples are replaced with `npa-cli` after the decision is accepted.
  - Contributor-facing examples continue to use `npa package ...`.
- Acceptance criteria:
  - There is no command-name placeholder left in the community roadmap task docs.
  - The parent roadmap and detailed CLR-00 task agree on the CLI package and binary name.
- Verification:
  - `rg -n "PACKAGE_""CLI_CRATE|npa-cli|npa package" doc/community-library-roadmap-todo.md doc/community-library-roadmap-clr-00-todo.md README.md`
  - `git diff --check`
- Notes:
  - The source roadmap may keep historical prose saying the old name was tentative only if a nearby note records the final decision.

### CLR-00-08 Close CLR-00 Readiness Review

- Status: Pending
- Depends on: CLR-00-01, CLR-00-02, CLR-00-03, CLR-00-04, CLR-00-05, CLR-00-06, CLR-00-07
- Inputs:
  - this task document
  - parent community roadmap todo
  - related Phase 8 import/checker docs
- Deliverables:
  - Short review note confirming CLR-01 can start without new product decisions.
  - List of deliberately deferred decisions.
  - Confirmation that trusted base is unchanged.
- Acceptance criteria:
  - No open finding remains for CLI name, crate placement, schema names, field names, or import semantics.
  - Deferred decisions are not blockers for CLR-01.
  - Review confirms no kernel/checker trusted boundary expansion.
- Verification:
  - `git diff --check`
  - `rg -n "npa package|npa-cli|npa\\.package\\.v0\\.1|npa.package.lock.v0.1|trusted_status|registry lookup" doc/community-library-roadmap-clr-00-todo.md doc/community-library-roadmap-todo.md`
- Notes:
  - Deferred decisions include external checker high-trust enforcement, dependency solving, registry server behavior, and binary cache behavior.

---

## CLR-01 Validator Cases Implied By CLR-00

CLR-01 should at least cover:

```text
- valid minimal package manifest
- valid current proof-corpus package fixture
- wrong schema
- missing required top-level field
- missing required module field
- forbidden `trusted_status`
- unknown top-level, policy, import, and module fields are rejected
- duplicate module name
- duplicate external import module
- local/external module name collision
- unknown module-level import
- import cycle among local modules
- invalid hash grammar
- path escaping package root
- absolute path rejection unless explicitly allowed by future policy
- expected hash mismatch surfaced as check-hashes failure, not manifest parse failure
- module axiom list violating package axiom policy
```

Suggested structured error families:

```text
Schema
Domain
Duplicate
Path
Graph
Hash
Policy
UnsupportedVersion
```

---

## Review Findings

Review against `doc/community-library-roadmap.md`, `doc/community-library-roadmap-todo.md`,
`proofs/manifest.toml`, `tools/proof-corpus`, and Phase 8 checker docs produced these findings:

```text
F1: The parent CLR-00 left command ownership unresolved, while later milestones need Cargo commands.
    Resolution: fix Cargo package `npa-cli`, installed binary `npa`, and examples under `npa package`.

F2: The roadmap example did not include the existing source file hash or certificate file hash.
    Resolution: require `expected_source_hash` and `expected_certificate_file_hash` in modules.

F3: Existing `trusted_status` could be misread as evidence in the package manifest.
    Resolution: mark it forbidden; checker verdicts belong in generated result artifacts.

F4: Module-level string imports could weaken external hash pinning if accepted directly.
    Resolution: allow strings only as references resolved to local modules or hash-pinned top-level imports.

F5: Package lock naming could collide conceptually with Phase 8 independent-checker import lock.
    Resolution: distinguish `npa.package.lock.v0.1` from `npa.independent-checker.import_lock_manifest.v1`.

F6: The first CLR-00 task draft left unknown-field policy for CLR-01 to decide.
    Resolution: `npa.package.v0.1` rejects unknown fields in every manifest object.
```

No open findings remain in this CLR-00 task breakdown.

---

## Validation Plan

For documentation-only changes to this task file:

```sh
git diff --check
rg -n "TO""DO|TB""D|未""定|PLACE""HOLDER" doc/community-library-roadmap-clr-00-todo.md
rg -n "PACKAGE_""CLI_CRATE|npa-cli|npa package|npa\\.package\\.v0\\.1|npa.package.lock.v0.1|trusted_status|registry lookup" \
  doc/community-library-roadmap-clr-00-todo.md doc/community-library-roadmap-todo.md doc/community-library-roadmap.md README.md
```

For implementation of CLR-00 tasks, run the verification commands listed in each task.
