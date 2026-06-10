# Community Library Roadmap CLR-01 Todo

Source: `develop/community-library-roadmap-todo.md` CLR-01

CLR-01 implements the `npa.package.v0.1` manifest data model and validator fixed by
CLR-00. The output of this milestone is a reusable `npa-package` library crate that
package CLI, proof-corpus migration, package graph verification, and publish metadata
generation can depend on without adding package-manager behavior to the kernel or checker.

---

## Scope

In scope:

```text
- `crates/npa-package` workspace crate
- `npa.package.v0.1` manifest schema constants
- Rust data model for package manifest, policy, modules, external imports, hashes, and paths
- TOML parsing through a structured parser
- closed-object schema validation with unknown field rejection
- duplicate key / duplicate module / duplicate import detection
- package-relative path validation
- hash string grammar validation
- package, version, module, declaration, and axiom name grammar validation
- module import resolution and local graph cycle detection
- package-level axiom policy validation
- deterministic structured validation errors
- valid / invalid fixture suite for CLR-01
```

Out of scope:

```text
- `npa-cli` package command implementation
- source to certificate build
- reading certificate bytes or source files
- checking expected hash values against files
- generating `proofs/npa-package.toml`
- generating package lock / axiom report / theorem index / publish plan
- source-free checker execution
- registry lookup, network fetch, dependency solving, or binary cache behavior
```

Trusted-boundary rule:

```text
`npa-package` is an untrusted metadata parser and validator.
It may reject malformed package metadata before build or check commands run,
but it never proves a theorem and never becomes certificate acceptance evidence.
The kernel, `npa-cert`, and `npa-checker-ref` must not depend on `npa-package`.
```

---

## Implementation Specification

### Crate Placement

Add a workspace crate:

```text
crates/npa-package
  Cargo package name: npa-package
  library crate name: npa_package
```

Workspace dependency direction:

```text
npa-cli -> npa-package
tools/proof-corpus -> npa-package only after CLR-02 needs it
npa-api -> npa-package only if a later milestone explicitly needs package metadata
npa-package -> npa-cert
npa-package must not depend on npa-api, npa-frontend, npa-tactic, npa-checker-ref, or npa-cli
npa-kernel, npa-cert, and npa-checker-ref must not depend on npa-package
```

`npa-package` should depend on a structured TOML parser. It should not hand-roll TOML
parsing with line scanning. Use local typed extraction on top of the parsed TOML value
when stable error paths are needed.

### Suggested Module Layout

```text
crates/npa-package/src/lib.rs
crates/npa-package/src/schema.rs
crates/npa-package/src/manifest.rs
crates/npa-package/src/error.rs
crates/npa-package/src/hash.rs
crates/npa-package/src/path.rs
crates/npa-package/src/name.rs
crates/npa-package/src/validate.rs
crates/npa-package/src/graph.rs
crates/npa-package/tests/package_manifest.rs
crates/npa-package/tests/fixtures/package/valid/minimal/npa-package.toml
crates/npa-package/tests/fixtures/package/valid/with-external-import/npa-package.toml
crates/npa-package/tests/fixtures/package/valid/proof-corpus-equivalent/npa-package.toml
crates/npa-package/tests/fixtures/package/invalid
```

The fixture names are part of the test contract. Later milestones may add fixtures, but
CLR-01 should establish the valid / invalid directory convention.

### Public Data Model

Public structs should be plain, cloneable, comparable Rust values:

```text
PackageManifest
PackagePolicy
PackageExternalImport
PackageModule
PackageHash
PackagePath
PackageId
PackageVersion
PackageGraph
ValidatedPackageManifest
ResolvedModuleImport
ResolvedModuleImportKind
```

Required manifest fields:

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

Optional top-level fields:

```text
license
repository
description
imports
```

Required module fields:

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

Optional module fields:

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

Top-level external import fields:

```text
module
package
version
export_hash
certificate_hash
certificate
```

Policy fields:

```text
allow_custom_axioms
allowed_axioms
```

Forbidden manifest fields remain rejected even if a parser would otherwise ignore them:

```text
trusted_status
verified_by_certificate
checker_result
registry_url
latest
generated_at
```

Unknown fields in every manifest object are rejected in `npa.package.v0.1`.

### Public API Contract

The crate should expose API at this level:

```text
parse_manifest_str
parse_and_validate_manifest_str
validate_manifest
validate_manifest_with_options
```

Expected behavior:

```text
parse_manifest_str
  parses TOML and returns raw typed package data without graph resolution

validate_manifest
  validates an already parsed manifest and returns either a validated package value
  or deterministic structured errors

parse_and_validate_manifest_str
  convenience wrapper for tests and CLI

validate_manifest_with_options
  accepts future flags such as path root policy without changing the core error model
```

CLR-01 should not expose filesystem-loading APIs as the primary interface. `npa-cli`
can read files and pass strings to `npa-package` in CLR-04. If a helper for loading from
a path is added, it must remain a convenience wrapper and must not canonicalize symlinks
or perform registry/network resolution.

### Schema Constants

Expose constants:

```text
PACKAGE_MANIFEST_SCHEMA = "npa.package.v0.1"
PACKAGE_LOCK_SCHEMA = "npa.package.lock.v0.1"
PACKAGE_AXIOM_REPORT_SCHEMA = "npa.package.axiom_report.v0.1"
PACKAGE_THEOREM_INDEX_SCHEMA = "npa.package.theorem_index.v0.1"
PACKAGE_PUBLISH_PLAN_SCHEMA = "npa.package.publish_plan.v0.1"
REGISTRY_MODULE_SCHEMA = "npa.registry.module.v0.1"
CORE_SPEC_V0_1 = "npa.core.v0.1"
KERNEL_PROFILE_V0_1 = "npa.kernel.v0.1"
CERTIFICATE_FORMAT_CANONICAL_V0_1 = "npa.certificate.canonical.v0.1"
CHECKER_PROFILE_REFERENCE_V0_1 = "npa.checker.reference.v0.1"
```

CLR-01 only parses the package manifest schema. The other constants are exposed so
later milestones do not duplicate string literals.

### Grammar Rules

Package id:

```text
lowercase ASCII
first byte: a-z
following bytes: a-z, 0-9, or hyphen
max length: 64 bytes
```

Package version:

```text
MAJOR.MINOR.PATCH
each segment is a decimal integer
segment "0" is allowed
leading zeroes are rejected
pre-release and build metadata are rejected in v0.1
```

Module and axiom names:

```text
use the same canonical dotted-name rule as `npa_cert::Name::is_canonical`
empty components are rejected
```

Declaration summary names in `inductives`, `definitions`, and `theorems`:

```text
use canonical dotted-name parsing
single-component names are accepted
duplicates across all three lists in the same module are rejected
```

Hash strings:

```text
must be `sha256:` followed by exactly 64 lowercase hex characters
uppercase hex is rejected
the parsed digest is stored as `npa_cert::Hash`
```

Path strings:

```text
must be non-empty UTF-8 strings
must use `/` as separator
must be package-relative
absolute paths are rejected
empty components are rejected
`.` and `..` components are rejected
backslash is rejected
control characters are rejected
registry URLs and URI schemes are rejected
```

CLR-01 path validation is lexical. It must not require files to exist and must not resolve symlinks.

### Validation Order

Validation must be deterministic. Use this pass order:

```text
1. TOML parse and duplicate key rejection
2. closed-object schema validation and type checking
3. fixed schema/profile value validation
4. scalar domain validation for package id, version, names, hashes, and paths
5. duplicate detection for package modules, external imports, declaration summaries, axioms, and artifact paths
6. module import resolution
7. local module graph cycle detection and topological order construction
8. package axiom policy validation
```

The validator may collect multiple errors per pass, but it must not report later-pass
errors before earlier-pass errors. Within a pass, report in source order first, then
lexicographic order for derived maps.

### Structured Error Contract

Expose structured errors with stable categories:

```text
TomlSyntax
Schema
UnsupportedVersion
Domain
Duplicate
Path
Hash
Graph
Policy
```

Each error should carry:

```text
kind
path
field when applicable
reason_code
expected_value when useful
actual_value when useful
```

Path format:

```text
$
policy.allow_custom_axioms
imports[0].module
modules[1].expected_export_hash
modules[2].imports[0]
```

Required reason codes:

```text
invalid_toml
duplicate_field
unknown_field
missing_field
wrong_type
wrong_schema
unsupported_schema
invalid_package_id
invalid_version
invalid_profile
invalid_module_name
invalid_declaration_name
invalid_axiom_name
invalid_hash_format
invalid_path
duplicate_module
duplicate_external_import
duplicate_declaration
duplicate_axiom
duplicate_artifact_path
local_external_module_collision
unknown_import
import_cycle
disallowed_axiom
```

Human display strings may be added, but tests should assert the structured fields.

### Import Resolution Contract

Validation resolves every module-level import string:

```text
local import
  import name matches exactly one `[[modules]]` entry
  identity uses that module's expected export and certificate hashes

external import
  import name matches exactly one top-level `[[imports]]` entry
  identity uses the top-level hash-pinned import entry

collision
  a module name present in both `[[modules]]` and top-level `[[imports]]` is invalid

unknown
  a module-level import that resolves neither locally nor externally is invalid
```

External imports are never accepted by module name alone. A top-level external import
entry without both `export_hash` and `certificate_hash` is invalid before graph validation.

Graph validation:

```text
only local module edges participate in cycle detection
external imports are leaves
topological order is deterministic
modules with no dependency relationship are ordered by source order, then module name
```

### Axiom Policy Contract

Policy semantics:

```text
allow_custom_axioms = false
  every module axiom must be present in allowed_axioms

allow_custom_axioms = true
  module axioms outside allowed_axioms are allowed but still recorded

allowed_axioms
  duplicate entries are rejected
  names must use canonical dotted-name grammar

sorry
  rejected regardless of allow_custom_axioms
```

CLR-01 validates the manifest's declared axiom summary. Later milestones still need to
compare generated certificate axiom reports with manifest expectations.

### Current Proof Corpus Compatibility

CLR-01 should prove the data model can represent the current proof corpus shape without
making `proofs/manifest.toml` the new package manifest.

Required approach:

```text
add a `proof-corpus-equivalent` test fixture under `crates/npa-package/tests/fixtures`
model at least the current top-level package fields, existing hash fields under expected names,
local imports, external Std imports, declaration summaries, and Eq.rec policy
document that full `proofs/npa-package.toml` generation is CLR-02
```

The CLR-01 fixture may be smaller than the full checked-in proof corpus only if it covers
both local imports and external imports. The full corpus package fixture is still a CLR-02 deliverable.

---

## Tasks

### CLR-01-01 Create `npa-package` Crate Skeleton

- Status: Completed
- Depends on: CLR-00
- Inputs:
  - root `Cargo.toml`
  - `develop/community-library-roadmap-clr-00-todo.md`
  - existing crate layout under `crates/`
- Code or documentation areas:
  - `Cargo.toml`
  - `crates/npa-package/Cargo.toml`
  - `crates/npa-package/src/lib.rs`
- Deliverables:
  - Workspace member `crates/npa-package`.
  - Library crate named `npa_package`.
  - Dependency on `npa-cert` for `Hash` and canonical name types.
  - Structured TOML parser dependency.
  - Initial module files for schema, manifest, error, hash, path, name, validate, and graph.
- Acceptance criteria:
  - `cargo test -p npa-package` builds an empty crate test target.
  - `npa-package` does not depend on `npa-api`, `npa-frontend`, `npa-tactic`, `npa-checker-ref`, or `npa-cli`.
  - Trusted crates do not depend on `npa-package`.
- Verification:
  - `cargo test -p npa-package`
  - `cargo tree -p npa-package`
  - `rg -n "npa-package" Cargo.toml crates/*/Cargo.toml`
- Notes:
  - Keep this crate free of filesystem, network, plugin, AI, and checker execution behavior.
  - Implemented by adding the workspace crate, initial module layout, `npa-cert`
    and structured TOML parser dependencies, and a parser-smoke unit test.

### CLR-01-02 Define Schema Constants And Public Raw Types

- Status: Completed
- Depends on: CLR-01-01
- Inputs:
  - CLR-00 schema names
  - CLR-00 field contract
- Code or documentation areas:
  - `crates/npa-package/src/schema.rs`
  - `crates/npa-package/src/manifest.rs`
  - `crates/npa-package/src/lib.rs`
- Deliverables:
  - Public schema constants from the Implementation Specification.
  - Raw manifest structs for package, policy, external imports, and modules.
  - Public newtypes for package id, package version, package hash, and package path.
  - Documentation comments describing trusted-boundary status.
- Acceptance criteria:
  - Field names exactly match CLR-00.
  - Optional fields are represented explicitly.
  - Forbidden and generated fields are not represented as accepted manifest inputs.
  - Rustdoc or crate docs say the manifest is metadata, not proof evidence.
- Verification:
  - `cargo test -p npa-package schema`
  - `rg -n "PACKAGE_MANIFEST_SCHEMA|PACKAGE_LOCK_SCHEMA|trusted_status|PackageManifest|PackageModule" crates/npa-package/src`
- Notes:
  - Do not expose stringly typed hash values after validation; store parsed digests.
  - Implemented by exposing CLR-00 schema/profile constants, accepted manifest input
    structs, package id/version/hash/path newtypes, and schema/type smoke tests.

### CLR-01-03 Implement Structured TOML Parse And Closed-Object Extraction

- Status: Completed
- Depends on: CLR-01-02
- Inputs:
  - CLR-00 unknown-field rejection rule
  - TOML parser dependency selected in CLR-01-01
- Code or documentation areas:
  - `crates/npa-package/src/manifest.rs`
  - `crates/npa-package/src/error.rs`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - `parse_manifest_str`.
  - Type validation for top-level object, `[policy]`, `[[imports]]`, and `[[modules]]`.
  - Unknown field rejection for every manifest object.
  - Duplicate key parse failure surfaced as structured schema error when the TOML parser reports it.
- Acceptance criteria:
  - Invalid TOML fails before schema validation.
  - Missing required fields are reported with stable paths.
  - Unknown fields are rejected with `unknown_field`.
  - Type mismatches are rejected with `wrong_type`.
  - Duplicate keys are rejected and do not silently use the last value.
- Verification:
  - `cargo test -p npa-package package_manifest_parse`
  - `cargo test -p npa-package package_manifest_closed_objects`
- Notes:
  - Use parsed TOML values and typed extraction. Avoid ad hoc line scanning.
  - Implemented by exposing `parse_manifest_str`, structured manifest errors,
    closed-object field allowlists, required-field/type extraction, duplicate-key
    TOML error mapping, and integration tests for parse and closed-object failures.

### CLR-01-04 Implement Scalar Domain Validators

- Status: Completed
- Depends on: CLR-01-03
- Inputs:
  - package id, version, name, hash, and path grammar in this document
  - `npa_cert::Name`
  - `npa_cert::Hash`
- Code or documentation areas:
  - `crates/npa-package/src/hash.rs`
  - `crates/npa-package/src/path.rs`
  - `crates/npa-package/src/name.rs`
  - `crates/npa-package/src/validate.rs`
- Deliverables:
  - Package id validator.
  - Package version validator.
  - Exact profile/schema value validators.
  - Canonical dotted-name validators.
  - `sha256:` lower-hex hash parser.
  - Lexical package-relative path validator.
- Acceptance criteria:
  - Uppercase hash hex is rejected.
  - Absolute paths, parent traversal, dot components, backslashes, control characters, and URI schemes are rejected.
  - Module and axiom names align with `npa_cert::Name::is_canonical`.
  - Exact v0.1 schema/profile string mismatches fail deterministically.
- Verification:
  - `cargo test -p npa-package package_manifest_scalar_domains`
  - `cargo test -p npa-package package_manifest_paths`
  - `cargo test -p npa-package package_manifest_hashes`
- Notes:
  - Do not import `npa-api` only to reuse hash parsing. Match the same wire grammar locally.
  - Implemented by adding scalar validation entry points, package id/version
    validators, exact schema/profile checks, canonical name validators, lexical
    package-relative path validation, local `sha256:` hash parsing, and targeted
    scalar/path/hash tests.

### CLR-01-05 Implement Duplicate And Artifact Path Checks

- Status: Completed
- Depends on: CLR-01-04
- Inputs:
  - validated raw manifest values
  - current proof-corpus artifact layout
- Code or documentation areas:
  - `crates/npa-package/src/validate.rs`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - Duplicate module name detection.
  - Duplicate top-level external import module detection.
  - Local/external module name collision detection.
  - Duplicate declaration summary detection within each module.
  - Duplicate allowed axiom detection.
  - Duplicate module artifact path detection for source, certificate, meta, and replay paths when present.
- Acceptance criteria:
  - Duplicate failures return structured `Duplicate` errors with stable paths.
  - Collision between a local module and external import is reported before module import resolution.
  - Optional path duplicates are checked only when the optional path is present.
- Verification:
  - `cargo test -p npa-package package_manifest_duplicates`
- Notes:
  - Duplicate artifact paths are rejected to avoid ambiguous writes in later build commands.
  - Implemented by adding duplicate checks for local modules, top-level external
    imports, local/external collisions, per-module declaration summaries,
    package and module axioms, and module artifact paths, with targeted duplicate
    validation tests.

### CLR-01-06 Implement Import Resolution And Graph Validation

- Status: Completed
- Depends on: CLR-01-05
- Inputs:
  - CLR-00 import semantics
  - local module list
  - top-level external import list
- Code or documentation areas:
  - `crates/npa-package/src/graph.rs`
  - `crates/npa-package/src/validate.rs`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - Resolved import representation for each module.
  - Local vs external import classification.
  - Unknown import rejection.
  - Local graph cycle detection.
  - Deterministic topological order for local modules.
- Acceptance criteria:
  - External imports are never accepted without hash-pinned top-level import entries.
  - Local imports use the imported module's expected export and certificate hashes.
  - External imports use top-level external import hashes.
  - Cycles are reported with stable module paths.
  - Independent local modules have deterministic order.
- Verification:
  - `cargo test -p npa-package package_manifest_import_resolution`
  - `cargo test -p npa-package package_manifest_import_cycles`
- Notes:
  - This task builds graph metadata only. It does not construct Phase 8 checker requests.
  - Implemented by adding package graph metadata, local/external import
    classification, hash identity selection, unknown import rejection, local
    cycle detection, and deterministic topological order tests.

### CLR-01-07 Implement Package Axiom Policy Validation

- Status: Completed
- Depends on: CLR-01-04, CLR-01-05, CLR-01-06
- Inputs:
  - CLR-00 policy fields
  - current proof-corpus modules that use `Eq.rec`
  - `npa_cert::AxiomPolicy` semantics as background only
- Code or documentation areas:
  - `crates/npa-package/src/validate.rs`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - `allow_custom_axioms` validation.
  - `allowed_axioms` grammar and duplicate validation.
  - Module axiom summary validation against package policy.
  - Hard rejection of `sorry`.
- Acceptance criteria:
  - A package with `allow_custom_axioms = false` rejects unlisted module axioms.
  - A package with `allow_custom_axioms = true` accepts recorded custom axioms.
  - Duplicate allowed axioms fail.
  - `sorry` fails regardless of policy setting.
- Verification:
  - `cargo test -p npa-package package_manifest_axiom_policy`
- Notes:
  - CLR-01 checks declared manifest summaries. Generated axiom report comparison is CLR-05.
  - Implemented by adding the package axiom policy pass after graph validation:
    listed axioms are accepted when custom axioms are disabled, recorded custom
    axioms are accepted only when enabled, and any `sorry`-shaped axiom summary
    is rejected regardless of policy setting.

### CLR-01-08 Implement Validation Report And Error Tests

- Status: Completed
- Depends on: CLR-01-03, CLR-01-04, CLR-01-05, CLR-01-06, CLR-01-07
- Inputs:
  - structured error contract in this document
  - repository preference for enum-like structured errors
- Code or documentation areas:
  - `crates/npa-package/src/error.rs`
  - `crates/npa-package/src/validate.rs`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - Public validation error type.
  - Public validation report or result type.
  - Stable validation pass ordering.
  - Tests that assert kind, path, reason code, expected value, and actual value for representative errors.
- Acceptance criteria:
  - Tests do not depend only on human display text.
  - Earlier validation passes suppress later graph or policy assumptions when needed.
  - Multiple errors in the same pass are deterministic.
- Verification:
  - `cargo test -p npa-package package_manifest_errors`
- Notes:
  - It is acceptable to collect multiple errors or return one report object, as long as behavior is deterministic and tested.
  - Implemented by exposing a deterministic validation report API that carries
    the first structured error, re-exporting the public result type, and adding
    targeted tests for structured fields plus validation-pass suppression.

### CLR-01-09 Add Valid And Invalid Manifest Fixtures

- Status: Completed
- Depends on: CLR-01-08
- Inputs:
  - CLR-00 manifest example
  - current `proofs/manifest.toml`
  - validator cases listed in `develop/community-library-roadmap-clr-00-todo.md`
- Code or documentation areas:
  - `crates/npa-package/tests/fixtures/package/valid`
  - `crates/npa-package/tests/fixtures/package/invalid`
  - `crates/npa-package/tests/package_manifest.rs`
- Deliverables:
  - Minimal valid package fixture.
  - Valid fixture with a hash-pinned external import.
  - Valid proof-corpus-equivalent fixture that covers local imports, external Std imports, expected hash fields, declaration summaries, and Eq.rec policy.
  - Invalid fixtures covering every required CLR-00 validator case.
- Acceptance criteria:
  - Every fixture has an assertion for expected pass or structured failure.
  - The proof-corpus-equivalent fixture does not claim to replace `proofs/manifest.toml`.
  - Expected hash mismatches are not treated as manifest validation failures; only malformed hash strings fail.
- Verification:
  - `cargo test -p npa-package package_manifest_fixtures`
  - `rg -n "trusted_status|verified_by_certificate|registry_url|latest|generated_at" crates/npa-package/tests/fixtures/package`
- Notes:
  - The full checked-in `proofs/npa-package.toml` fixture remains CLR-02.
  - Implemented by adding fixture-backed package manifest tests for valid
    minimal, external-import, local-import, proof-corpus-shaped, and hash-value
    mismatch scenarios plus invalid fixtures for the CLR-00 schema, domain,
    hash, path, duplicate, graph, and policy validator cases.
  - Fixture expectations follow the implemented CLR-01 pass ordering: forbidden
    status, checker verdict, registry, and latest-version fields are closed
    schema `unknown_field` failures, local/external collisions remain
    `Duplicate` failures from CLR-01-05, and absolute / escaping paths share
    the lexical `InvalidPath` reason.

### CLR-01-10 Document `npa-package.toml` For Implementers

- Status: Completed
- Depends on: CLR-01-09
- Inputs:
  - public API and fixture behavior from earlier CLR-01 tasks
  - `develop/community-library-roadmap-clr-00-todo.md`
- Code or documentation areas:
  - `crates/npa-package/src/lib.rs`
  - package docs under `docs/` if the implementation creates one
  - `develop/community-library-roadmap-clr-01-todo.md` if task status is updated during implementation
- Deliverables:
  - Crate-level docs with a small valid manifest example.
  - Trusted-boundary note for package metadata.
  - Short validator behavior summary for CLI implementers.
- Acceptance criteria:
  - Docs state that `npa-package` does not read source or certificate files for proof acceptance.
  - Docs state that registry/network resolution is forbidden in manifest validation.
  - Docs point CLI implementers to structured errors instead of parsing display strings.
- Verification:
  - `cargo test -p npa-package --doc`
  - `rg -n "trusted base|registry|structured error|npa.package.v0.1" crates/npa-package develop/community-library-roadmap-clr-01-todo.md`
- Notes:
  - Keep user-facing package CLI docs for CLR-04 and CI docs for CLR-07.
  - Implemented by expanding the `npa-package` crate-level docs with a minimal
    `npa.package.v0.1` manifest doctest, trusted-boundary notes, metadata-only
    validator behavior, registry/network prohibition, and guidance for CLI
    implementers to consume structured errors instead of display text.

### CLR-01-11 Close CLR-01 Integration Readiness

- Status: Completed
- Depends on: CLR-01-01, CLR-01-02, CLR-01-03, CLR-01-04, CLR-01-05, CLR-01-06, CLR-01-07, CLR-01-08, CLR-01-09, CLR-01-10
- Inputs:
  - completed `npa-package` crate
  - parent roadmap CLR-01 acceptance criteria
  - CLR-02 proof-corpus package fixture needs
- Code or documentation areas:
  - `develop/community-library-roadmap-todo.md`
  - `develop/community-library-roadmap-clr-01-todo.md`
  - `README.md` only if repository overview needs updating
- Deliverables:
  - Readiness note confirming CLR-02 can start.
  - Confirmation that the validator accepts a proof-corpus-equivalent package representation.
  - Confirmation that trusted base is unchanged.
  - Follow-up list for CLR-02 only if fixture generation gaps remain.
- Acceptance criteria:
  - Parent CLR-01 acceptance criteria are all satisfied.
  - `cargo test -p npa-package` passes.
  - `cargo test -p npa-proof-corpus` still passes if proof-corpus code is touched.
  - No kernel/checker crate depends on `npa-package`.
- Verification:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p npa-package`
  - `cargo test -p npa-proof-corpus`
  - `cargo test --workspace package_manifest`
- Notes:
  - Full workspace tests are appropriate if implementation touches workspace dependencies or shared types.
  - CLR-02 can start. CLR-01 now provides the reusable `npa-package` crate,
    public `npa.package.v0.1` data model, structured parser / validator,
    deterministic graph validation, axiom-policy checks, report API, fixture
    suite, and implementer-facing crate docs.
  - The validator accepts the proof-corpus-equivalent fixture at
    `crates/npa-package/tests/fixtures/package/valid/proof-corpus-equivalent/npa-package.toml`.
    That fixture covers local imports, hash-pinned `Std.Logic.Eq` and
    `Std.Nat.Basic` imports, expected source / certificate / export / axiom
    report / certificate hashes, declaration summaries, and the `Eq.rec`
    package policy without claiming to replace `proofs/manifest.toml`.
  - Trusted base is unchanged: `npa-package` remains an untrusted metadata
    parser / validator; it performs no source or certificate file reads, no
    checker execution, no registry or network resolution, and no proof
    acceptance. `npa-kernel`, `npa-cert`, and `npa-checker-ref` do not depend on
    `npa-package`.
  - CLR-02 follow-up: generate the full checked-in `proofs/npa-package.toml`
    fixture from the existing proof corpus, keep `proofs/manifest.toml` as the
    legacy `npa-ai-proof-corpus-v0.1` artifact, materialize deterministic
    vendored Std certificate paths / hashes, and add proof-corpus tests that
    compare the full module list and artifact summaries against the existing
    corpus.

---

## Review Findings

Review against `develop/community-library-roadmap-todo.md`, `develop/community-library-roadmap-clr-00-todo.md`,
`proofs/manifest.toml`, `tools/proof-corpus`, `npa-cert`, and existing Phase 8 validation patterns
produced these findings:

```text
F1: The parent CLR-01 did not fix the validator dependency direction.
    Resolution: place it in `crates/npa-package`; depend on `npa-cert`; avoid `npa-api` dependency.

F2: CLR-00 fixed unknown-field rejection, but CLR-01 needed an implementation strategy for stable paths.
    Resolution: require structured TOML parsing plus typed extraction with closed-object checks.

F3: Current proof-corpus manifest includes source and certificate file hashes that are not canonical certificate hashes.
    Resolution: keep `expected_source_hash` and `expected_certificate_file_hash` as required module fields.

F4: Path validation could accidentally become filesystem validation and introduce environment dependence.
    Resolution: CLR-01 path validation is lexical only; existence and symlink checks are deferred to CLI/build tasks.

F5: Import graph validation could accept external imports by module name alone.
    Resolution: module strings resolve only to local modules or hash-pinned top-level external imports.

F6: Axiom policy could be confused with generated certificate axiom reports.
    Resolution: CLR-01 validates manifest summaries only; generated axiom report comparison remains CLR-05.
```

No open findings remain in this CLR-01 task breakdown.

---

## Validation Plan

For documentation-only changes to this task file:

```sh
git diff --check
rg -n "TO""DO|TB""D|UNDECIDED|PLACE""HOLDER" develop/community-library-roadmap-clr-01-todo.md
rg -n "npa-package|npa.package.v0.1|trusted_status|expected_source_hash|expected_certificate_file_hash|unknown_field|import_cycle|registry lookup" \
  develop/community-library-roadmap-clr-01-todo.md develop/community-library-roadmap-clr-00-todo.md develop/community-library-roadmap-todo.md
```

For implementation of CLR-01 tasks, run the verification commands listed in each task.
