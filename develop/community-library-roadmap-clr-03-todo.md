# Community Library Roadmap CLR-03 Todo

Source: `develop/community-library-roadmap-todo.md` CLR-03

CLR-03 adds the package-level lock and source-free package graph verification
layer. The implementation starts from the `proofs/npa-package.toml` fixture
created in CLR-02, derives pinned certificate identity from checked-in
`.npcert` files, and verifies the package in dependency-topological order
without reading `.npa` source, replay files, metadata, theorem indexes, AI
traces, or registry data.

---

## Scope

In scope:

```text
- `npa.package.lock.v0.1` canonical JSON data model
- package lock generation from a validated `npa.package.v0.1` manifest
- local and external certificate artifact hash checks
- certificate-declared import identity checks against package graph identity
- dependency-topological package verification plan
- source-free fast verifier mode for local development
- source-free reference checker mode using `npa-checker-ref`
- adapters from package lock entries to Phase 8 independent-checker import locks and requests
- deterministic structured errors for graph, hash, certificate, import, policy, and checker failures
- tests proving package verification does not read source/replay/meta/theorem-index/AI artifacts
```

Out of scope:

```text
- public `npa package verify-certs` CLI command
- source-to-certificate build
- source hash checking against `.npa` files
- theorem index generation
- standalone package axiom report artifact generation
- publish metadata generation
- registry lookup, dependency solving, network fetch, binary cache, or implicit latest-version resolution
- requiring the external checker binary as a pass condition
- replacing `npa.independent-checker.import_lock_manifest.v1`
```

Trusted-boundary rule:

```text
Package lock, package graph verification, and MachineCheckRequest materialization
are orchestration artifacts. They are not proof evidence by themselves.
Accepted proof evidence remains canonical certificate bytes plus the selected
checker verdict. The kernel, `npa-cert`, and `npa-checker-ref` must not depend
on package metadata or gain filesystem, network, registry, plugin, AI, or
package-manager behavior.
```

---

## Implementation Specification

### Placement And Dependency Boundaries

Use the crate boundaries fixed by CLR-00 and CLR-01:

```text
crates/npa-package
  owns package lock data model, canonical JSON serialization, source-free
  verification plan construction, and package-level structured errors

crates/npa-api
  owns Phase 8 independent-checker adapters and request/import-lock
  materialization helpers that need existing `IndependentChecker*` types

crates/npa-checker-ref
  remains a package-unaware checker that accepts certificate bytes,
  explicit import stores, and checker policy only

crates/npa-cli
  remains the future CLI wrapper for CLR-04; CLR-03 may add no public command
```

Allowed dependency movement in CLR-03:

```text
npa-api -> npa-package is allowed because CLR-03 explicitly bridges package
metadata to existing Phase 8 request and import-lock artifacts.

npa-api -> npa-checker-ref is allowed only if reference package verification
uses the in-process reference checker API. The dependency must remain one-way;
`npa-checker-ref` must not depend on `npa-api` or `npa-package`.
```

Disallowed dependency movement:

```text
npa-package -> npa-api
npa-package -> npa-checker-ref
npa-package -> npa-cli
npa-checker-ref -> npa-package
npa-cert -> npa-package
npa-kernel -> npa-package
```

If an implementation wants to avoid a runtime `npa-api -> npa-checker-ref`
dependency, it may place the reference-mode runner behind a feature or in
`npa-cli`, but the CLR-03 tests still need a source-free reference-mode path.

### Package Lock Artifact

CLR-03 introduces this generated package artifact:

```text
proofs/generated/package-lock.json
```

The schema string is:

```text
npa.package.lock.v0.1
```

The lock is canonical JSON, not TOML. It is generated from exact package
manifest bytes plus certificate artifact bytes. It must not contain timestamps,
host paths, absolute paths, registry URLs, environment data, checker verdicts,
source text, replay data, or AI traces.

Required top-level fields:

```text
schema
package
version
manifest
entries
```

`manifest` fields:

```text
path
file_hash
```

`manifest.file_hash` is the exact SHA-256 of the checked `npa-package.toml`
bytes. CLR-03 does not introduce a separate semantic manifest hash.

`entries` is sorted by module dotted-name using the same deterministic ordering
as package graph validation. Each entry has:

```text
module
origin
certificate
certificate_file_hash
export_hash
axiom_report_hash
certificate_hash
imports
```

`origin` is one of:

```text
local
external
```

External entries also carry:

```text
package
version
```

Each `imports` item records the direct certificate import identity:

```text
module
export_hash
certificate_hash
```

The package lock includes `axiom_report_hash` even though the existing
`npa.independent-checker.import_lock_manifest.v1` does not. The Phase 8 import
lock remains a checker-run input. The package lock is the package-level closure
artifact.

### Lock Builder Semantics

The lock builder starts from a `ValidatedPackageManifest` produced by CLR-01 and
the package fixture produced by CLR-02.

For each local package module:

```text
1. Read only the module certificate path.
2. Compute exact certificate file SHA-256.
3. Decode the canonical certificate enough to obtain module name, declared
   imports, export hash, axiom report hash, and certificate hash.
4. Check certificate file hash against `expected_certificate_file_hash`.
5. Check export, axiom report, and certificate hashes against the package
   module `expected_*` fields.
6. Check the decoded certificate module name equals the manifest module name.
7. Check the certificate-declared direct imports match the manifest-resolved
   direct imports by module, export hash, and certificate hash.
```

For each top-level external import:

```text
1. Read only the external import certificate path.
2. Compute exact certificate file SHA-256.
3. Decode the canonical certificate enough to obtain module name, declared
   imports, export hash, axiom report hash, and certificate hash.
4. Check module, export hash, and certificate hash against the top-level
   `[[imports]]` entry.
5. Require every decoded certificate import to resolve to another lock entry.
6. Reject any external import certificate that depends on a local package
   module.
```

The lock builder rejects unresolved imports, local/external collisions, import
cycles, path escape, missing certificates, stale file hashes, stale canonical
hashes, and certificate-declared imports that are not present in the package
graph. It does not read `.npa` source, `meta.json`, `replay.json`, theorem
indexes, AI traces, or registry metadata.

### Verification Order

Package verification uses this order:

```text
1. Validate package manifest with `npa-package`.
2. Build package lock from certificate artifacts.
3. Build a dependency graph from lock entries and certificate-declared imports.
4. Verify external import entries in dependency-topological order.
5. Verify local package modules in dependency-topological order.
6. Compare checker-produced module, export hash, axiom report hash, and
   certificate hash against the package lock entry after every check.
```

Verification order must be derived from dependency edges, not request order,
manifest table order, filesystem traversal order, or import lock entry order.
Independent nodes use the deterministic order fixed by `npa-package` graph
validation.

### Fast Verifier Mode

Fast mode is for local development only. It verifies canonical certificate bytes
through the existing Rust certificate verifier:

```text
npa_cert::verify_module_cert
```

Fast mode may use `npa_cert::VerifierSession` and an `AxiomPolicy` derived from
the package policy plus the selected package verifier profile. It must still be
source-free and topo ordered.

Fast mode result names must make the trust status explicit:

```text
checker_mode = "fast-kernel"
reference_checker_verdict = false
```

Fast mode must never be labeled as a source-free reference checker verdict.

### Reference Checker Mode

Reference mode verifies the same package lock through `npa-checker-ref`.

The package verifier must use the reference checker library API in
dependency-topological order:

```text
check_certificate
ReferenceImportStore::from_checked_modules
```

Do not treat `cargo run -p npa-checker-ref -- --imports package-lock.json` as a
complete package high-trust check. The current checker CLI can load source-free
import certificates, but those imports are not marked as checked by the same
reference checker. High-trust package verification requires checked imports
built from earlier successful reference checker results.

Reference mode policy is derived deterministically:

```text
trust_mode = high_trust for package graph verification
deny_sorry = true
deny_custom_axioms = not package.policy.allow_custom_axioms
allowed_axioms = package.policy.allowed_axioms
supported_core_features = features supported by checker_profile
```

For the current `npa.checker.reference.v0.1` profile, the implementation must
explicitly decide the supported core feature set. The existing proof corpus has
certificates that may require `quotient_v1`, `quotient_v2`, and `quotient_v3`;
defaulting to an empty feature set would reject the current corpus for the
wrong reason. Unsupported certificate features must fail deterministically.

### Phase 8 Adapter Semantics

CLR-03 bridges package lock entries to existing Phase 8 artifacts without
changing their schema names.

For each module check, derive an independent-checker import lock:

```text
schema = "npa.independent-checker.import_lock_manifest.v1"
imports = direct imports needed by that module
```

Each derived import entry uses the existing Phase 8 fields:

```text
module
export_hash
certificate.kind = "path"
certificate.path
certificate.file_hash
certificate.certificate_hash
```

The package lock remains the only CLR-03 artifact that records
`axiom_report_hash` for every dependency. Do not mutate
`npa.independent-checker.import_lock_manifest.v1` in this milestone.

For each module, the adapter may materialize or return an in-memory
`IndependentCheckerMachineCheckRequest` with:

```text
certificate.path
certificate.file_hash
certificate.expected_certificate_hash
imports.mode = "locked_store"
imports.manifest
imports.manifest_hash
checker_profile
trust_mode
axiom_policy
budget
```

The request must not include source path, source text, tactic script, replay
data, theorem index, AI trace, registry URL, or package solver state.

### Structured Error Surface

Expose deterministic package verification errors with stable categories:

```text
Manifest
LockSchema
ArtifactIo
Path
Hash
CertificateDecode
CertificateIdentity
Graph
Policy
FastChecker
ReferenceChecker
Phase8Adapter
SourceFreeBoundary
```

Required reason codes:

```text
manifest_invalid
lock_schema_invalid
certificate_missing
certificate_path_invalid
certificate_file_hash_mismatch
export_hash_mismatch
axiom_report_hash_mismatch
certificate_hash_mismatch
certificate_module_mismatch
certificate_import_missing_from_manifest
certificate_import_hash_mismatch
unresolved_import
import_cycle
duplicate_lock_entry
unsupported_core_feature
disallowed_axiom
fast_checker_rejected
reference_checker_rejected
phase8_import_lock_invalid
phase8_request_invalid
source_artifact_read_attempted
```

Errors should carry:

```text
kind
reason_code
path or module when applicable
field when applicable
expected_hash when applicable
actual_hash when applicable
expected_value when applicable
actual_value when applicable
checker_error when applicable
```

Tests should assert structured fields, not human display text.

---

## Tasks

### CLR-03-01 Define Package Lock Data Model And Canonical JSON

- Status: Pending
- Depends on: CLR-02
- Inputs:
  - `develop/community-library-roadmap-clr-00-todo.md`
  - `develop/community-library-roadmap-clr-01-todo.md`
  - `develop/community-library-roadmap-clr-02-todo.md`
  - `crates/npa-api/src/independent_checker.rs`
- Code or documentation areas:
  - `crates/npa-package/src/schema.rs`
  - `crates/npa-package/src/lock.rs`
  - `crates/npa-package/src/error.rs`
  - `crates/npa-package/tests/package_lock.rs`
- Deliverables:
  - `PACKAGE_LOCK_SCHEMA = "npa.package.lock.v0.1"` if not already present from CLR-01.
  - Public `PackageLockManifest`, `PackageLockManifestReference`, `PackageLockEntry`, and `PackageLockImport` structs.
  - Deterministic canonical JSON serializer and parser for package locks.
  - Closed-object validation with unknown field rejection.
- Acceptance criteria:
  - Package lock entries include module, origin, certificate path, certificate file hash, export hash, axiom report hash, certificate hash, and direct import identities.
  - External lock entries include package and version.
  - Package lock serialization has stable field and entry ordering.
  - Package lock parsing rejects duplicate modules, duplicate certificate paths, malformed hashes, malformed names, and unknown fields.
  - `npa-package` still does not depend on `npa-api`, `npa-checker-ref`, or `npa-cli`.
- Verification:
  - `cargo test -p npa-package package_lock_schema`
  - `cargo test -p npa-package package_lock_canonical_json`
  - `cargo tree -p npa-package`
- Notes:
  - Keep the package lock separate from `npa.independent-checker.import_lock_manifest.v1`.

### CLR-03-02 Build Lock Entries From Certificate Artifacts

- Status: Pending
- Depends on: CLR-03-01
- Inputs:
  - validated `PackageManifest` from CLR-01
  - `proofs/npa-package.toml` from CLR-02
  - `proofs/Proofs/Ai/**/certificate.npcert`
  - `proofs/vendor/npa-std/**/certificate.npcert`
  - `crates/npa-cert/src/lib.rs`
- Code or documentation areas:
  - `crates/npa-package/src/lock.rs`
  - `crates/npa-package/src/artifact.rs` if a byte-provider abstraction is useful
  - `crates/npa-package/tests/package_lock.rs`
  - `proofs/generated/package-lock.json`
- Deliverables:
  - Package lock builder that consumes a validated manifest and certificate bytes.
  - Exact certificate file hash computation.
  - Certificate decode/hash extraction for module, import table, export hash, axiom report hash, and certificate hash.
  - Local module hash checks against `expected_certificate_file_hash`, `expected_export_hash`, `expected_axiom_report_hash`, and `expected_certificate_hash`.
  - External import checks against top-level package import `export_hash` and `certificate_hash`.
- Acceptance criteria:
  - Missing certificate files fail before checker execution.
  - Stale local certificate bytes fail by file hash or canonical certificate hash.
  - Stale external import certificates fail by module, export hash, or certificate hash.
  - The lock builder does not read source, replay, meta, theorem index, or AI trace paths.
  - The generated `proofs/generated/package-lock.json` is deterministic.
- Verification:
  - `cargo test -p npa-package package_lock_builder`
  - `cargo test -p npa-proof-corpus package_lock_fixture`
  - `rg -n "npa.package.lock.v0.1|axiom_report_hash|certificate_file_hash" proofs/generated crates/npa-package`
- Notes:
  - If `npa-package` remains filesystem-free, put filesystem reads in a small caller-owned byte provider and test both direct bytes and proof-corpus file loading.

### CLR-03-03 Validate Certificate Imports Against Package Graph

- Status: Pending
- Depends on: CLR-03-02
- Inputs:
  - package manifest graph from CLR-01
  - decoded certificate import tables
  - package lock entries from CLR-03-02
- Code or documentation areas:
  - `crates/npa-package/src/graph.rs`
  - `crates/npa-package/src/lock.rs`
  - `crates/npa-package/tests/package_lock.rs`
- Deliverables:
  - Comparison between manifest-resolved direct imports and certificate-declared direct imports.
  - Resolution of external import certificate dependencies through package lock entries.
  - Deterministic lock graph cycle detection.
  - Topological verification order over local and external lock entries.
- Acceptance criteria:
  - A certificate importing a module not present in the package graph fails.
  - A certificate import with matching module but wrong export hash or certificate hash fails.
  - External import certificates cannot silently pull new dependencies from outside the package lock.
  - Verification order is derived from the lock graph, not manifest order or filesystem order.
- Verification:
  - `cargo test -p npa-package package_lock_import_identity`
  - `cargo test -p npa-package package_lock_topological_order`
  - `cargo test --workspace import_lock`
- Notes:
  - This task makes package graph metadata accountable to the certificate import table.

### CLR-03-04 Add Fast Source-Free Package Verification Mode

- Status: Pending
- Depends on: CLR-03-03
- Inputs:
  - package lock entries
  - `npa_cert::verify_module_cert`
  - `npa_cert::VerifierSession`
  - package axiom policy from CLR-01
- Code or documentation areas:
  - `crates/npa-api/src/package_verifier.rs` or an equivalent untrusted orchestration module
  - `crates/npa-api/src/lib.rs`
  - `crates/npa-api/Cargo.toml`
  - tests in `crates/npa-api`
- Deliverables:
  - `PackageVerificationMode::FastKernel` or equivalent public mode.
  - Source-free fast verifier that checks certificates in lock topological order.
  - Structured per-module result with module, checker mode, status, export hash, axiom report hash, and certificate hash.
  - Clear result field indicating this is not a reference checker verdict.
- Acceptance criteria:
  - Fast mode reads certificate bytes and lock/manifest metadata only.
  - Fast mode rejects missing imports, stale hashes, and disallowed axioms deterministically.
  - Fast mode uses checked imports from earlier topo-order successes.
  - Fast mode results cannot be confused with `npa-checker-ref` results.
- Verification:
  - `cargo test -p npa-api package_fast_verifier`
  - `cargo test -p npa-api independent_checker`
  - `cargo test -p npa-proof-corpus package_fast_source_free`
- Notes:
  - Fast mode exists to keep local package development ergonomic. Release/high-trust acceptance still needs reference checker mode.

### CLR-03-05 Add Reference Source-Free Package Verification Mode

- Status: Pending
- Depends on: CLR-03-03
- Inputs:
  - package lock entries
  - `crates/npa-checker-ref/src/lib.rs`
  - `ReferenceImportStore::from_checked_modules`
  - `check_certificate`
  - package axiom policy from CLR-01
- Code or documentation areas:
  - `crates/npa-api/src/package_verifier.rs` or equivalent untrusted orchestration module
  - `crates/npa-api/Cargo.toml`
  - `crates/npa-checker-ref/src/main.rs` only if checker policy file parsing must be extended for package usage
  - tests in `crates/npa-api`
- Deliverables:
  - `PackageVerificationMode::Reference` or equivalent public mode.
  - Reference checker policy derivation from package policy and checker profile.
  - Topological reference checker execution using checked modules from earlier successful checks.
  - Hash comparison between `ReferenceCheckedModule` output and package lock entry.
  - Deterministic handling of supported core features for `npa.checker.reference.v0.1`.
- Acceptance criteria:
  - Reference mode verifies imports as already checked by the same reference checker.
  - High-trust package verification does not rely on raw `npa-checker-ref --imports` directory scanning.
  - Unsupported core features fail with a structured package verification error.
  - The current proof corpus can be verified after CLR-02, including modules that require quotient core features.
  - The verifier does not call `npa_cert::verify_module_cert` in reference mode.
- Verification:
  - `cargo test -p npa-api package_reference_verifier`
  - `cargo test -p npa-checker-ref`
  - `cargo test -p npa-proof-corpus package_reference_source_free`
  - `./scripts/phase8-release-audit.sh`
- Notes:
  - If `npa-checker-ref` CLI policy parsing is extended for `supported_core_features`, keep it deterministic and covered by checker-ref tests.

### CLR-03-06 Materialize Phase 8 Import Locks And Requests From Package Lock

- Status: Pending
- Depends on: CLR-03-03, CLR-03-05
- Inputs:
  - `PackageLockManifest`
  - `IndependentCheckerImportLockManifest`
  - `IndependentCheckerMachineCheckRequest`
  - `independent_checker_request_materialize`
- Code or documentation areas:
  - `crates/npa-api/src/package_verifier.rs` or equivalent adapter module
  - `crates/npa-api/src/independent_checker.rs` only if existing helpers need narrow public exposure
  - tests in `crates/npa-api`
  - generated fixture paths under `proofs/generated/checker-requests/` if checked in
- Deliverables:
  - Per-module Phase 8 import lock materialization from package lock direct imports.
  - Per-module MachineCheckRequest materialization using existing Phase 8 request schema.
  - Request IDs and paths that are deterministic from package, version, module, and checker profile.
  - Tests showing generated Phase 8 import locks and requests parse with existing `npa-api` validators.
- Acceptance criteria:
  - Derived Phase 8 import lock entries include only direct imports required by the checked module.
  - Derived Phase 8 artifacts contain no source, replay, meta, theorem index, AI trace, registry URL, or package solver data.
  - Existing `npa.independent-checker.import_lock_manifest.v1` remains unchanged.
  - MachineCheckRequest hash recomputation passes.
- Verification:
  - `cargo test -p npa-api package_phase8_import_lock_adapter`
  - `cargo test -p npa-api package_phase8_request_materialization`
  - `cargo test -p npa-api independent_checker`
- Notes:
  - CLR-04 will wrap these adapters in `npa package verify-certs`.

### CLR-03-07 Add Source-Free Boundary Regression Tests

- Status: Pending
- Depends on: CLR-03-04, CLR-03-05, CLR-03-06
- Inputs:
  - proof-corpus package fixture from CLR-02
  - package lock builder
  - fast and reference package verifier modes
- Code or documentation areas:
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
  - `crates/npa-api` package verifier tests
  - package fixtures under `crates/npa-package/tests/fixtures`
- Deliverables:
  - A test package or temp copy where `.npa` source, replay, and meta files are absent but certificate verification still succeeds.
  - Negative tests for missing dependency, certificate file hash mismatch, export hash mismatch, certificate hash mismatch, axiom report hash mismatch, unlisted certificate import, and import cycle.
  - Tests that invalid package graphs fail before checker execution.
  - Tests that checker failures preserve checker-specific structured error payloads.
- Acceptance criteria:
  - Removing source files does not affect source-free verify success when certificate artifacts and package metadata are valid.
  - Removing or corrupting certificate files fails before or during certificate verification with deterministic errors.
  - A package graph with request-order shuffling still verifies in the same topo order.
  - Tests assert structured error kind and reason code.
- Verification:
  - `cargo test --workspace package_source_free`
  - `cargo test --workspace package_lock`
  - `cargo test --workspace import_lock`
- Notes:
  - Do not weaken CLR-01 manifest validation tests; source-free verification starts after a valid manifest exists.

### CLR-03-08 Update Documentation And CLR-04 Handoff

- Status: Pending
- Depends on: CLR-03-01, CLR-03-02, CLR-03-03, CLR-03-04, CLR-03-05, CLR-03-06, CLR-03-07
- Inputs:
  - `develop/community-library-roadmap-todo.md` CLR-03 and CLR-04
  - `proofs/README.md` if it lists generated proof artifacts
  - package verifier API names selected during implementation
- Code or documentation areas:
  - `develop/community-library-roadmap-todo.md`
  - `proofs/README.md`
  - README command examples only if CLR-03 introduces hidden or library-level commands
- Deliverables:
  - Documentation that `proofs/generated/package-lock.json` is derived package metadata, not proof evidence.
  - Documentation that CLR-04 package commands should call the CLR-03 lock builder and verifier rather than reimplement graph traversal.
  - Verification examples for fast and reference package verification library tests.
  - Note that `npa-checker-ref` raw CLI import scanning is not sufficient for high-trust package graph verification by itself.
- Acceptance criteria:
  - Later CLR-04 work can implement `npa package verify-certs` without redefining lock schema, graph order, checker policy mapping, or Phase 8 adapters.
  - Documentation continues to forbid registry/network resolution in package verification.
  - Documentation continues to state that source, replay, meta, theorem index, and AI traces are not source-free checker inputs.
- Verification:
  - `rg -n "package-lock.json|npa.package.lock.v0.1|source-free|npa-checker-ref|verify-certs" doc proofs/README.md`
  - `git diff --check`
- Notes:
  - Keep CLI wording forward-looking unless CLR-04 has already implemented commands.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: Existing Phase 8 import lock schema lacks `axiom_report_hash`, while
CLR-03 needs package-level lock identity that includes it.
Fix: Define `npa.package.lock.v0.1` as the package-level lock carrying
`axiom_report_hash`; keep `npa.independent-checker.import_lock_manifest.v1`
unchanged as a derived checker-run input.

Finding: The `npa-checker-ref` CLI can load import certificates from an import
lock, but high-trust mode requires imports checked by the same reference
checker. Raw CLI import scanning alone is therefore not a complete package
graph verifier.
Fix: Reference mode must execute the checker library in dependency-topological
order and build `ReferenceImportStore::from_checked_modules` from prior
successful results.

Finding: Package manifests contain source, replay, and metadata paths, so a
package verifier could accidentally read untrusted sidecars while verifying.
Fix: Lock builder and verifier tasks explicitly read only manifest/lock and
certificate bytes; boundary tests remove source/replay/meta files and still
expect source-free verification to pass.

Finding: The current proof corpus may require quotient core features, but the
reference checker default policy supports no optional core features.
Fix: Reference package verification must derive an explicit supported core
feature set from `checker_profile` and fail unsupported features
deterministically.

Finding: A manifest graph can claim imports that differ from the actual
certificate import table.
Fix: CLR-03-03 requires certificate-declared direct import identity to match
manifest-resolved direct import identity by module, export hash, and certificate
hash.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now separates package lock from Phase
8 import locks, preserves trusted boundaries, handles high-trust reference
imports correctly, forbids source-side reads, covers optional core features,
and gives CLR-04 a concrete API handoff.
```
