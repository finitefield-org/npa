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

```text
npa.package.v0.1
npa.package.lock.v0.1
npa.package.axiom_report.v0.1
npa.package.theorem_index.v0.1
npa.package.publish_plan.v0.1
npa.registry.module.v0.1
```

Existing Phase 8 schemas remain separate:

```text
npa.independent-checker.import_lock_manifest.v1
npa.independent-checker.checker_binary_registry.v1
```

`npa.package.lock.v0.1` is the package-level import/package lock artifact. The
independent-checker import lock is a source-free checker input derived from package
metadata, not the same public package lock schema.

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

External theorem libraries use `npa-package.toml` at the package root:

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

[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
export_hash = "sha256:..."
certificate_hash = "sha256:..."
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"

[[modules]]
module = "Proofs.Ai.Basic"
source = "Proofs/Ai/Basic/source.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
meta = "Proofs/Ai/Basic/meta.json"
replay = "Proofs/Ai/Basic/replay.json"
producer_profile = "human-surface-explicit-term"

imports = []
expected_source_hash = "sha256:..."
expected_certificate_file_hash = "sha256:..."
expected_export_hash = "sha256:..."
expected_axiom_report_hash = "sha256:..."
expected_certificate_hash = "sha256:..."

inductives = []
definitions = []
theorems = ["id"]
axioms = []
```

### Required, Optional, Forbidden, And Generated Fields

Top-level required fields:

```text
schema
package
version
core_spec
kernel_profile
certificate_format
checker_profile
policy
modules
```

Top-level optional fields:

```text
license
repository
description
imports
```

Module required fields:

```text
module
source
certificate
imports
expected_source_hash
expected_certificate_file_hash
expected_export_hash
expected_axiom_report_hash
expected_certificate_hash
```

Module optional fields:

```text
meta
replay
producer_profile
inductives
definitions
theorems
axioms
tags
```

Forbidden in `npa.package.v0.1`:

```text
trusted_status
verified_by_certificate
checker_result
registry_url
latest
generated_at
```

Unknown fields in the top-level object, `[policy]`, `[[imports]]`, and `[[modules]]`
are rejected in `npa.package.v0.1`. Forward-compatible extension fields require a
new schema version.

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

### Legacy Field Mapping

Mapping from `proofs/manifest.toml` to `npa.package.v0.1`:

| Legacy field | Target field | Notes |
| --- | --- | --- |
| `schema` | `schema` | Change from `npa-ai-proof-corpus-v0.1` to `npa.package.v0.1`. |
| missing | `package` | Use `npa-proof-corpus` for the in-repo fixture. |
| missing | `version` | Use `0.1.0` until release tagging is introduced. |
| missing | `core_spec` | Use `npa.core.v0.1`. |
| missing | `kernel_profile` | Use `npa.kernel.v0.1`. |
| missing | `certificate_format` | Use `npa.certificate.canonical.v0.1`. |
| missing | `checker_profile` | Use `npa.checker.reference.v0.1`. |
| `[[proof_modules]]` | `[[modules]]` | Rename table family. |
| `module` | `module` | Preserve dotted module name. |
| `source` | `source` | Preserve package-relative path. |
| `certificate` | `certificate` | Preserve package-relative path. |
| `meta` | `meta` | Optional untrusted metadata path. |
| `replay` | `replay` | Optional untrusted replay path. |
| `producer_profile` | `producer_profile` | Optional build helper metadata. |
| `trusted_status` | forbidden | Checker verdict belongs in generated result artifacts. |
| `source_sha256` | `expected_source_hash` | Required hash check input. |
| `certificate_file_sha256` | `expected_certificate_file_hash` | Required byte-level artifact check. |
| `export_hash` | `expected_export_hash` | Required canonical export identity check. |
| `axiom_report_hash` | `expected_axiom_report_hash` | Required axiom report identity check. |
| `certificate_hash` | `expected_certificate_hash` | Required high-trust certificate identity check. |
| `imports` | `modules[].imports` plus top-level `[[imports]]` for external modules | Local imports may resolve to `[[modules]]`; external imports must resolve to hash-pinned top-level imports. |
| `inductives` | `inductives` | Optional declaration summary. |
| `definitions` | `definitions` | Optional declaration summary. |
| `theorems` | `theorems` | Optional declaration summary. |
| `axioms` | `axioms` | Optional declaration summary checked against policy and generated axiom report. |

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

Resolution rules:

```text
1. If the imported module name matches a local `[[modules]]` entry, it is a same-package import.
   Its export and certificate identity comes from that module's expected hash fields.

2. If the imported module name matches a top-level `[[imports]]` entry, it is an external import.
   Its identity comes from the hash-pinned top-level import entry.

3. If a module name matches both local and external entries, validation fails.

4. If a module-level import does not resolve locally or through top-level `[[imports]]`,
   validation fails.

5. External imports are never accepted by module name alone.

6. Registry lookup, network fetch, or implicit latest-version resolution is forbidden.
```

The package lock generated later must use at least:

```text
module
package identity for external imports
export_hash
certificate_hash
certificate path
certificate file hash
axiom_report_hash when available
```

---

## Tasks

### CLR-00-01 Record CLI And Crate Topology Decision

- Status: Pending
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

### CLR-00-02 Define Schema Constant Names And Artifact Boundaries

- Status: Pending
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

### CLR-00-03 Specify `npa.package.v0.1` Manifest Field Contract

- Status: Pending
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

### CLR-00-04 Specify Legacy Manifest Mapping

- Status: Pending
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

### CLR-00-05 Specify Import Resolution Semantics

- Status: Pending
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
