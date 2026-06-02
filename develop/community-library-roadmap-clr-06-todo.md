# Community Library Roadmap CLR-06 Todo

Source: `develop/community-library-roadmap-todo.md` CLR-06

CLR-06 generates deterministic publish metadata for a package release without
building a registry service. It adds `npa package publish-plan`, a checked
`generated/publish-plan.json` artifact, package release metadata, module
registry seed entries, and downstream import bundle metadata.

The goal is to let another package pin and verify released certificate
artifacts from Git release files alone. Registry metadata remains helper data;
proof acceptance remains canonical certificates plus source-free checker
verdicts.

---

## Scope

対象:

```text
- `generated/publish-plan.json`
- `npa.package.publish_plan.v0.1`
- `npa.registry.module.v0.1`
- package release metadata derived from manifest, lock, axiom report, theorem index, certificates, and checker summaries
- deterministic artifact list with exact SHA-256 file hashes
- checksum-only MVP signing policy
- module registry seed entries for local package modules
- downstream import bundle metadata sufficient for another package to pin imports
- `npa package publish-plan`
- check mode and write mode for `publish-plan`
- tests for deterministic ordering, stale metadata rejection, and registry/checker schema separation
```

非対象:

```text
- registry server
- registry client lookup
- package dependency solver
- network fetch
- binary cache service
- latest-version resolution
- GitHub release upload
- release signing key management or cryptographic signature generation
- external checker required mode
- `verified_high_trust` artifact
- changed-module or reverse-dependency selection
- source-to-certificate build
- package lock, axiom report, or theorem index generation internals
- theorem search service or online theorem graph store
```

Trusted-boundary rule:

```text
Publish metadata and registry seed entries are untrusted helper data. They may
help another package locate artifacts, pin import hashes, and decide what to
verify, but they never replace local certificate verification.

`npa package publish-plan` must not contact a registry, fetch dependencies,
resolve latest versions, or read network resources. It must not make registry
metadata a checker input. The kernel, `npa-cert`, and `npa-checker-ref` must
not depend on package publish metadata or `npa-cli`.
```

---

## Implementation Specification

### Artifact Location

CLR-06 owns this checked package artifact:

```text
generated/publish-plan.json
```

For the in-repo proof corpus fixture, that path resolves to:

```text
proofs/generated/publish-plan.json
```

Only `npa package publish-plan` write mode writes this file. CLR-04 build
commands and CLR-05 artifact commands must continue to avoid writing it.

### Schema Names

Use the schema constants fixed by CLR-00 and exposed by CLR-01:

```text
npa.package.publish_plan.v0.1
npa.registry.module.v0.1
```

These schemas are distinct from Phase 8 checker binary registry schemas:

```text
npa.independent-checker.checker_binary_registry.v1
npa.independent-checker.runner_policy.v1
npa.independent-checker.machine_check_result.v1
```

`npa.registry.module.v0.1` describes theorem package module metadata. It must
not be parsed or validated as an independent checker binary registry.

### Crate Placement And Dependency Boundaries

Recommended code ownership:

```text
crates/npa-package
  owns publish-plan and registry module data models, schema constants,
  canonical JSON writers, parsers, self hashes, and deterministic validation

crates/npa-api
  may own narrow adapters for source-free checker summaries when existing
  Phase 8 result types are reused

crates/npa-cli
  owns package root loading, filesystem reads and writes, command parsing,
  check/write behavior, and diagnostics for `package publish-plan`
```

Allowed dependency direction:

```text
npa-cli -> npa-package
npa-cli -> npa-api
npa-cli -> npa-cert
npa-api -> npa-package
npa-package -> npa-cert
```

Disallowed dependency direction:

```text
npa-package -> npa-api
npa-package -> npa-cli
npa-api -> npa-cli
npa-checker-ref -> npa-package
npa-checker-ref -> npa-cli
npa-cert -> npa-package
npa-kernel -> npa-package
npa-kernel -> npa-api
npa-kernel -> npa-cli
```

`npa-package` should remain the pure model/canonicalization crate. If
`npa-api` adapters are needed, keep them independent of tactic execution,
Machine API sessions, AI search controllers, theorem graph scoring, and
network configuration.

### Command Contract

Cargo development command:

```sh
cargo run -p npa-cli -- package publish-plan --root proofs --check
```

Contributor-facing command after installation:

```sh
npa package publish-plan --root proofs --check
```

Write mode:

```sh
cargo run -p npa-cli -- package publish-plan --root proofs
npa package publish-plan --root proofs
```

Common flags inherited from CLR-04:

```text
--root PATH
  Package root. Defaults to the current directory only. The CLI must not search
  parent directories or registry locations.

--json
  Emit deterministic JSON diagnostics using `npa.package.command_result.v0.1`.

--check
  Generate in memory and fail if `generated/publish-plan.json` would change.
  No files are written.
```

Unsupported in CLR-06:

```text
--changed
--all
--checker external
--registry
--network
--latest
--upload
--sign
--update-manifest-hashes
```

Full-package operation is the only CLR-06 scope. Release workflow wiring and
changed-module CI selection belong to CLR-07. External checker required mode
and high-trust disagreement gates belong to CLR-08.

### Exit Codes And Diagnostics

Use the CLR-04 process exit codes:

```text
0  command succeeded
1  package validation, stale artifact, source-free verification, publish metadata, or checksum failure
2  CLI usage error or unexpected internal command failure
```

Extend the `npa.package.command_result.v0.1` diagnostic vocabulary with these
categories if not already present:

```text
PublishPlan
RegistrySeed
ReleaseArtifact
SignaturePolicy
DownstreamImportBundle
```

Required reason codes:

```text
publish_plan_missing
publish_plan_stale
publish_plan_hash_mismatch
publish_plan_non_canonical_order
registry_entry_mismatch
registry_schema_conflict
release_artifact_missing
release_artifact_hash_mismatch
release_artifact_self_reference
checker_summary_missing
checker_summary_stale
downstream_import_bundle_mismatch
signature_policy_unsupported
network_access_attempted
```

Diagnostics must identify the package-relative path, module, artifact role,
field, expected hash, and actual hash whenever applicable.

### Input Loading

`package publish-plan` uses the package root loader from CLR-04 and reads:

```text
npa-package.toml
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
local module certificate files referenced by the package lock
external import certificate files referenced by the package lock
existing generated/publish-plan.json only in check mode
```

It must not read:

```text
.npa source files
replay.json
meta.json
AI traces
tactic traces
prompt metadata
theorem graph scores
registry URLs
network resources
package solver state
Git host APIs
```

The command may reuse source-free package verification from CLR-03/CLR-04 to
obtain fresh reference checker summaries. If it includes only fast-kernel
summaries, they must be labeled as fast-kernel and must not be reported as a
reference checker verdict.

### Publish Plan Generation Order

Generate the publish plan in this order:

```text
1. Validate `npa-package.toml`.
2. Load and validate `generated/package-lock.json`.
3. Load and validate `generated/axiom-report.json`.
4. Load and validate `generated/theorem-index.json`.
5. Recompute exact file hashes for required release artifacts.
6. Rebuild or validate source-free package verification summaries.
7. Check that package lock, axiom report, theorem index, certificate hashes,
   and checker summaries agree.
8. Project module registry seed entries for local modules.
9. Project downstream import bundle metadata.
10. Build canonical `npa.package.publish_plan.v0.1`.
11. Compute `publish_plan_hash` over canonical bytes excluding the self-hash field.
```

Failure in any earlier source-free validation step prevents publish metadata
from being reported as fresh.

### Publish Plan Schema

The publish plan JSON has stable top-level fields:

```text
schema
package
version
release
artifacts
module_registry_entries
downstream_import_bundle
checker_summaries
signature_policy
summary
publish_plan_hash
```

`release` fields:

```text
core_spec
kernel_profile
certificate_format
checker_profile
manifest
package_lock
axiom_report
theorem_index
```

Each release metadata reference contains:

```text
path
file_hash
content_hash when the referenced artifact has a self hash
schema when applicable
```

`artifacts` is sorted by artifact role, module when present, and path. Each
artifact entry contains:

```text
role
path
file_hash
module when applicable
origin when applicable
schema when applicable
```

Required artifact roles:

```text
package_manifest
package_lock
axiom_report
theorem_index
local_certificate
external_import_certificate
```

The publish plan itself is not included in `artifacts` to avoid a self-hash
cycle. Release upload tooling may include `generated/publish-plan.json` as a
file, but its identity is represented by `publish_plan_hash`.

`module_registry_entries` contains one `npa.registry.module.v0.1` entry for
each local package module. External imports are dependency pins, not registry
entries for this package.

`downstream_import_bundle` contains enough information for another package to
create hash-pinned `[[imports]]` entries without contacting a registry server.

`checker_summaries` records source-free checker summaries used to validate the
release metadata. It is informational metadata and must not replace rerunning
`package verify-certs`.

`signature_policy` fields:

```text
mode
hash_algorithm
signature_required
signatures
```

CLR-06 MVP values are:

```text
mode = "checksum-only"
hash_algorithm = "sha256"
signature_required = false
signatures = []
```

Cryptographic signatures and signing key management are target integration for
a later release workflow. CLR-06 must not invent fake signature bytes.

`summary` fields:

```text
local_module_count
external_import_count
artifact_count
registry_entry_count
checker_summary_count
```

### Registry Module Entry Schema

Each module registry seed entry has schema:

```text
npa.registry.module.v0.1
```

Required fields:

```text
schema
package
package_version
module
core_spec
kernel_profile
certificate_format
export_hash
certificate_hash
axiom_report_hash
certificate
imports
checker_results
artifact_hashes
```

`certificate` fields:

```text
path
file_hash
```

`imports` records direct certificate imports, sorted by module name:

```text
module
origin
package when external
version when external
export_hash
certificate_hash
```

`origin` is:

```text
local
external
```

`checker_results` is sorted by checker mode, checker id, profile, and module.
Each checker result contains:

```text
checker
profile
mode
status
export_hash
certificate_hash
axiom_report_hash
```

Allowed `status` values in CLR-06:

```text
accepted
rejected
not_run
```

Publish-plan success requires the configured release checker summaries to be
accepted. For CLR-06, source-free reference checker success is required.
External checker `not_run` may be represented only as target integration
metadata and must not be treated as high-trust verification.

`artifact_hashes` contains hashes of release-wide artifacts relevant to this
module:

```text
package_lock_file_hash
axiom_report_file_hash
theorem_index_file_hash
```

### Downstream Import Bundle Schema

The downstream import bundle is embedded in `generated/publish-plan.json`.
It is not a separate schema in CLR-06.

Top-level fields:

```text
package
version
modules
```

`modules` is sorted by module dotted name. Each module entry contains:

```text
module
package
version
export_hash
certificate_hash
axiom_report_hash
certificate
certificate_file_hash
```

This is the data another package needs to write a top-level `[[imports]]` entry
for a released module:

```text
module
package
version
export_hash
certificate_hash
certificate
```

The downstream package still must obtain certificate bytes and run its own
source-free verification. The import bundle does not authorize importing a
module by name alone.

### Canonical JSON Rules

Generated JSON must be canonical:

```text
- stable object field order as specified by the schema
- arrays sorted by canonical bytes unless the schema explicitly says module order
- no timestamps
- no absolute paths
- no host names
- no environment variables
- no Cargo target paths
- no Git commit lookup requirement
- no registry URLs
- no network-derived data
- no latest-version marker
- no nondeterministic map iteration
- no source text
- no replay, meta, tactic, prompt, theorem graph score, or AI trace payloads
```

File hashes are exact SHA-256 hashes of checked file bytes. Self hashes are
computed over schema-defined canonical bytes, not over pretty-printed JSON with
the self-hash field included.

### Check Mode And Write Mode

Check mode:

```text
1. Generate the publish plan in memory.
2. Read `generated/publish-plan.json`.
3. Parse and validate the checked-in publish plan.
4. Compare canonical JSON bytes and `publish_plan_hash`.
5. Exit with code 1 if the artifact is missing, stale, non-canonical, or
   inconsistent with package artifacts.
6. Write no files.
```

Write mode:

```text
1. Generate the publish plan in memory.
2. Parse the generated JSON back through the schema parser.
3. Write only `generated/publish-plan.json`.
4. Use an atomic or all-or-nothing write strategy where practical.
5. Leave the file unchanged if bytes are identical.
```

Write mode must not write:

```text
npa-package.toml
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
source files
certificate files
replay files
meta files
registry server files
AI sidecars
```

### Consistency Checks

Publish-plan generation fails if:

```text
- package manifest validation fails
- package lock is stale
- axiom report is stale
- theorem index is stale
- a local certificate file hash differs from the package lock
- a registry entry hash disagrees with the package lock
- a checker summary hash disagrees with the package lock
- required source-free checker result is missing or rejected
- a module registry entry uses only a module name for import identity
- a registry module entry schema is confused with checker binary registry schema
- an artifact path escapes the package root
- the generated publish plan would include a registry URL or latest-version marker
```

### Handoff To Later Milestones

CLR-07 can use these outputs in release-template variants once CLR-06 is
complete:

```text
npa package publish-plan --check
generated/publish-plan.json
release artifact list
failure diagnostics
```

The parent roadmap keeps base CLR-07 CI templates dependent on CLR-05. CLR-09
is the milestone that requires both CLR-06 publish metadata and CLR-07 CI
templates for the seed repository release flow.

CLR-08 owns:

```text
external checker required mode
verified_high_trust artifacts
external checker disagreement gates
cryptographic signing if it becomes part of high-trust release policy
```

CLR-09 consumes:

```text
downstream_import_bundle
module registry seed entries
release artifact list
```

CLR-10 consumes:

```text
registry seed entries
publish-plan pass/fail evidence
remaining registry-server requirements
```

---

## Tasks

### CLR-06-01 Define Publish Plan And Registry Seed Schemas

- Status: Completed
- Depends on: CLR-05
- Inputs:
  - schema constants from CLR-00 and CLR-01
  - registry entry sketch in `develop/community-library-roadmap.md`
  - `develop/community-library-roadmap-todo.md` CLR-06
  - independent-checker binary registry schema names from Phase 8
- Code or documentation areas:
  - `crates/npa-package/src/schema.rs`
  - `crates/npa-package/src/publish_plan.rs`
  - `crates/npa-package/src/registry.rs`
  - `crates/npa-package/tests/publish_plan.rs`
- Deliverables:
  - Public Rust data model for `npa.package.publish_plan.v0.1`.
  - Public Rust data model for `npa.registry.module.v0.1`.
  - Canonical JSON writers and parsers for both schemas.
  - Self-hash computation for publish plans.
  - Structured validation errors for publish-plan and registry seed artifacts.
- Acceptance criteria:
  - Schema strings are exactly `npa.package.publish_plan.v0.1` and `npa.registry.module.v0.1`.
  - Parsers reject unknown fields, duplicate module registry entries, non-canonical order, stale self hashes, absolute paths, timestamps, registry URLs, and latest-version markers.
  - `npa.registry.module.v0.1` is validated separately from `npa.independent-checker.checker_binary_registry.v1`.
  - The publish plan does not include itself in its own artifact list.
  - `npa-package` does not depend on `npa-api`, `npa-cli`, or `npa-checker-ref`.
- Verification:
  - `cargo test -p npa-package publish_plan_schema`
  - `cargo test -p npa-package registry_module_schema`
  - `cargo test -p npa-package publish_plan_canonical_json`
  - `cargo tree -p npa-package`
  - `rg -n "npa.package.publish_plan.v0.1|npa.registry.module.v0.1|checker_binary_registry" crates/npa-package develop/community-library-roadmap-clr-06-todo.md`
- Notes:
  - Keep cryptographic signatures out of CLR-06 implementation. Represent checksum-only MVP policy explicitly.

### CLR-06-02 Implement Publish Input Collection And Freshness Checks

- Status: Completed
- Depends on: CLR-06-01
- Inputs:
  - CLR-04 package root loader and diagnostics
  - CLR-03 package lock and source-free verification APIs
  - CLR-05 axiom report and theorem index parsers
  - package fixture from `proofs/`
- Code or documentation areas:
  - `crates/npa-cli/src/package_publish.rs`
  - `crates/npa-api/src/package_publish.rs` only if source-free checker adapters belong there
  - `crates/npa-package/src/publish_plan.rs`
  - `crates/npa-cli/tests/package_cli.rs`
- Deliverables:
  - Shared publish input collector for manifest, package lock, axiom report, theorem index, certificates, and checker summaries.
  - Freshness checks for package lock, axiom report, theorem index, and certificate file hashes.
  - Source-free checker summary collection with explicit mode labels.
  - File access guard or tests proving source/replay/meta/AI/network files are not read.
- Acceptance criteria:
  - The collector reads only allowed CLR-06 inputs.
  - Stale lock, stale axiom report, stale theorem index, stale certificate, missing checker result, or rejected checker result fails before publish metadata is generated.
  - Fast-kernel summaries are never labeled as reference checker verdicts.
  - Reference checker summaries come from the source-free reference verification path.
  - No registry lookup or Git host API call is required.
- Verification:
  - `cargo test -p npa-cli package_publish_inputs`
  - `cargo test -p npa-cli package_publish_source_free_boundary`
  - `cargo test -p npa-api package_source_free`
  - `cargo test -p npa-package package_lock`
- Notes:
  - If an implementation needs Git commit data later, add it in a later schema. CLR-06 must remain deterministic without invoking Git.

### CLR-06-03 Generate Release Artifact List And Checksum Policy

- Status: Completed
- Depends on: CLR-06-02
- Inputs:
  - publish input collection output
  - exact file bytes for manifest, package lock, generated artifacts, and certificates
  - checksum-only MVP signing policy
- Code or documentation areas:
  - `crates/npa-package/src/publish_plan.rs`
  - `crates/npa-cli/src/package_publish.rs`
  - `crates/npa-package/tests/publish_plan.rs`
- Deliverables:
  - Deterministic release artifact list.
  - Exact SHA-256 file hash computation for every listed artifact.
  - Signature policy object with checksum-only MVP values.
  - Validation rejecting self-referential artifact list entries.
- Acceptance criteria:
  - Required artifact roles are present: package manifest, package lock, axiom report, theorem index, local certificates, and external import certificates.
  - Artifact paths are package-relative and cannot escape the package root.
  - Artifact list order is canonical and independent of filesystem traversal.
  - The publish plan file is excluded from its own artifact list.
  - Signature policy is explicit and does not contain fake signature bytes.
- Verification:
  - `cargo test -p npa-package publish_plan_artifacts`
  - `cargo test -p npa-cli package_publish_artifact_hashes`
  - `rg -n "checksum-only|signature_required|generated/publish-plan.json" crates/npa-package develop/community-library-roadmap-clr-06-todo.md`
- Notes:
  - CLR-06 documents signing as target integration. It does not implement release signing.

### CLR-06-04 Generate Module Registry Seed Entries

- Status: Completed
- Depends on: CLR-06-02
- Inputs:
  - package lock local module entries
  - certificate import identities
  - source-free checker summaries
  - package manifest profile fields
- Code or documentation areas:
  - `crates/npa-package/src/registry.rs`
  - `crates/npa-cli/src/package_publish.rs`
  - `crates/npa-package/tests/registry_module.rs`
- Deliverables:
  - One `npa.registry.module.v0.1` entry for each local package module.
  - Direct import identity list using module, origin, export hash, and certificate hash.
  - Checker results with explicit checker mode and status.
  - Registry entry artifact hash links back to package lock, axiom report, and theorem index file hashes.
- Acceptance criteria:
  - Registry entries are generated only for local modules owned by the package.
  - External imports are represented as dependency pins, not as registry entries owned by the current package.
  - Import identity never relies on module name alone.
  - A checker result mismatch or missing required checker result fails publish-plan generation.
  - Registry module schema is not confused with independent checker binary registry schema.
- Verification:
  - `cargo test -p npa-package registry_module`
  - `cargo test -p npa-cli package_publish_registry_entries`
  - `rg -n "npa.registry.module.v0.1|checker_binary_registry" crates develop/community-library-roadmap-clr-06-todo.md`
- Notes:
  - External checker results may appear as `not_run` target integration metadata, but they must not satisfy high-trust verification.

### CLR-06-05 Generate Downstream Import Bundle Metadata

- Status: Completed
- Depends on: CLR-06-04
- Inputs:
  - local module registry entries
  - package manifest identity
  - local certificate paths and hashes
- Code or documentation areas:
  - `crates/npa-package/src/publish_plan.rs`
  - `crates/npa-cli/src/package_publish.rs`
  - `crates/npa-package/tests/publish_plan.rs`
- Deliverables:
  - Embedded downstream import bundle in `generated/publish-plan.json`.
  - Module entries sorted by module dotted name.
  - Import-ready module, package, version, export hash, certificate hash, axiom report hash, certificate path, and certificate file hash.
  - Validation that downstream import entries contain enough information for `npa-package.toml` top-level `[[imports]]`.
- Acceptance criteria:
  - Another package can copy a module entry into a top-level `[[imports]]` entry without guessing hashes.
  - The bundle does not include latest-version, registry URL, or network fetch fields.
  - The bundle does not authorize importing by module name alone.
  - Every downstream import bundle module corresponds to a local registry seed entry.
- Verification:
  - `cargo test -p npa-package downstream_import_bundle`
  - `cargo test -p npa-cli package_publish_downstream_import_bundle`
  - `rg -n "downstream_import_bundle|latest|registry_url" crates/npa-package develop/community-library-roadmap-clr-06-todo.md`
- Notes:
  - This bundle is embedded in the publish plan for CLR-06. A separate bundle schema can be added later if release tooling needs it.

### CLR-06-06 Implement `package publish-plan`

- Status: Completed
- Depends on: CLR-06-03, CLR-06-04, CLR-06-05
- Inputs:
  - CLR-04 `npa-cli` package command framework
  - publish plan generator
  - existing generated path `generated/publish-plan.json`
- Code or documentation areas:
  - `crates/npa-cli/src/package_publish.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/tests/package_cli.rs`
  - `proofs/generated/publish-plan.json`
- Deliverables:
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check`.
  - `cargo run -p npa-cli -- package publish-plan --root proofs`.
  - Check mode that compares generated canonical JSON with checked-in output.
  - Write mode that updates only `generated/publish-plan.json`.
  - JSON and human diagnostics for stale publish plan and registry seed mismatches.
- Acceptance criteria:
  - `--check` writes no files.
  - Write mode writes only `generated/publish-plan.json`.
  - The command fails if package lock, axiom report, theorem index, certificates, or checker summaries are stale.
  - Unsupported registry/network/upload/sign/latest flags fail with exit code `2`.
  - The command result uses `npa.package.command_result.v0.1`.
  - The command does not read source, replay, meta, AI, registry, network, or Git host API data.
- Verification:
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check`
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check --json`
  - `cargo test -p npa-cli package_publish_plan`
  - `git diff --exit-code -- proofs/generated/publish-plan.json`
- Notes:
  - Keep upload/release automation out of this command. It generates the plan; it does not publish to a service.

### CLR-06-07 Add End-To-End Publish Metadata Tests

- Status: Completed
- Depends on: CLR-06-06
- Inputs:
  - proof-corpus package fixture
  - generated package lock, axiom report, theorem index, and publish plan
  - temp package fixtures
- Code or documentation areas:
  - `crates/npa-cli/tests/package_cli.rs`
  - `crates/npa-package/tests/publish_plan.rs`
  - `crates/npa-cli/tests/fixtures/package`
  - `proofs/generated/publish-plan.json`
- Deliverables:
  - End-to-end check-mode test for `package publish-plan`.
  - Write-mode test in a temp package copy.
  - Negative tests for stale package lock, stale axiom report, stale theorem index, stale certificate, missing checker result, wrong registry schema, latest-version marker, registry URL, and attempted network/source read.
  - Snapshot-like assertions for JSON diagnostic shape without host-specific paths.
- Acceptance criteria:
  - Running publish-plan write mode twice without input changes is idempotent.
  - A fresh checkout can run publish-plan check after generated artifacts are checked in.
  - Removing `.npa`, replay, and meta files from a temp package does not break publish-plan generation when certificate artifacts and generated metadata remain valid.
  - Invalid metadata failures exit with code `1`; unsupported flags exit with code `2`.
  - Tests do not mutate checked-in `proofs/` except when a specific generation command is intentionally run and then checked for a clean diff.
- Verification:
  - `cargo test -p npa-package publish_plan`
  - `cargo test -p npa-cli package_publish_plan`
  - `cargo test --workspace publish_plan`
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check`
- Notes:
  - Include a direct test that `npa.registry.module.v0.1` and `npa.independent-checker.checker_binary_registry.v1` cannot be substituted for each other.

### CLR-06-08 Update Documentation And Handoff To CLR-07/09

- Status: Completed
- Depends on: CLR-06-07
- Inputs:
  - `develop/community-library-roadmap-todo.md`
  - `develop/community-library-roadmap.md`
  - `README.md`
  - `proofs/README.md`
  - generated publish-plan schema
- Code or documentation areas:
  - README package command examples
  - `proofs/README.md`
  - `develop/community-library-roadmap-todo.md`
  - package artifact schema docs if added under `crates/npa-package`
- Deliverables:
  - Contributor-facing example for `npa package publish-plan --check`.
  - Documentation that publish metadata and registry seed entries are helper data, not proof evidence.
  - Documentation that CLR-06 uses checksum-only MVP signature policy.
  - Handoff note for optional CLR-07 release-template use and CLR-09 seed library release artifacts.
- Acceptance criteria:
  - Docs do not imply a registry server exists.
  - Docs do not imply registry metadata is checker input.
  - Docs distinguish theorem package registry metadata from independent checker binary registry metadata.
  - Docs state that source-free local verification remains required downstream.
  - Parent roadmap points to this detailed CLR-06 task document.
- Verification:
  - `rg -n "publish-plan|npa.package.publish_plan.v0.1|npa.registry.module.v0.1|checksum-only|checker_binary_registry|not proof evidence" README.md doc proofs/README.md`
  - `git diff --check`
- Notes:
  - Keep CI workflow implementation in CLR-07 and dogfood repository work in CLR-09.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: Publish metadata could be mistaken for a registry service or implicit
dependency resolver.
Fix: CLR-06 explicitly excludes registry server, network lookup, latest-version
resolution, and dependency solving. Downstream import metadata remains hash
pins only.

Finding: The parent milestone asks for artifact checksum/signature policy but
does not choose what MVP supports.
Fix: CLR-06 fixes MVP to checksum-only SHA-256 with no signature bytes and
defers signing keys/signature generation to a later high-trust release workflow.

Finding: Registry module metadata could be confused with the Phase 8 checker
binary registry.
Fix: The schema section and tests require `npa.registry.module.v0.1` to remain
distinct from `npa.independent-checker.checker_binary_registry.v1`.

Finding: A publish plan artifact list can accidentally include itself and make
hashing circular.
Fix: CLR-06 excludes `generated/publish-plan.json` from its own `artifacts`
list and represents it only through `publish_plan_hash`.

Finding: Checker summaries in registry entries could be treated as proof
acceptance evidence.
Fix: The trusted-boundary section states that summaries are helper metadata and
downstream packages must rerun source-free verification locally.

Finding: A handoff note that said CLR-07 consumes publish-plan output could
conflict with the parent roadmap where base CLR-07 depends on CLR-05.
Fix: The handoff now says CLR-07 may use publish-plan output in release-template
variants after CLR-06, while CLR-09 requires both CLR-06 and CLR-07.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now fixes publish-plan schemas,
release artifact hashing, registry seed entries, downstream import bundles,
CLI check/write behavior, deterministic tests, and CLR-07/09 handoff
boundaries.
```
