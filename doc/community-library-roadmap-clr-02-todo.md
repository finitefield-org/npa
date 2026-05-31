# Community Library Roadmap CLR-02 Todo

Source: `doc/community-library-roadmap-todo.md` CLR-02

CLR-02 represents the checked-in proof corpus as a real `npa.package.v0.1`
package fixture. The result is a deterministic `proofs/npa-package.toml`
that package commands can load without reading the hard-coded Rust module
table in `tools/proof-corpus/src/main.rs`.

---

## Scope

対象:

```text
- checked-in package fixture at `proofs/npa-package.toml`
- deterministic proof-corpus generator output for both legacy and package manifests
- compatibility mapping from `npa-ai-proof-corpus-v0.1` fields to `npa.package.v0.1`
- top-level external import entries for the current Std imports
- generated or checked-in certificate artifacts for `Std.Logic.Eq` and `Std.Nat.Basic`
- package fixture validation with `npa-package`
- legacy manifest versus package fixture parity tests
- preservation of existing proof-corpus certificate verification tests
```

非対象:

```text
- package CLI commands
- package lock generation
- source-free checker execution over the package graph
- theorem index, axiom report, or publish-plan generation
- registry lookup, network fetch, dependency solving, or latest-version resolution
- changing `proofs/manifest.toml` from its legacy schema
- trusting TOML metadata as proof evidence
```

Trusted-boundary rule:

```text
`proofs/npa-package.toml` is package metadata, not a proof certificate.
The fixture may drive orchestration and validation, but it is accepted only
after canonical certificates and their pinned hashes are checked by later
verification stages. The kernel, `npa-cert`, and `npa-checker-ref` must not
gain filesystem, network, registry, plugin, AI, or package-manager behavior.
```

---

## Implementation Specification

### Fixture Location And Ownership

CLR-02 introduces this checked-in package fixture:

```text
proofs/npa-package.toml
```

`proofs/manifest.toml` remains checked in with:

```toml
schema = "npa-ai-proof-corpus-v0.1"
```

`cargo run -p npa-proof-corpus` must generate both files deterministically from
the same generated module data. The legacy manifest remains a compatibility
artifact for existing tests and documentation until a later milestone retires it.

Current corpus facts from `proofs/manifest.toml`:

```text
local proof modules: 66
external imports: Std.Logic.Eq, Std.Nat.Basic
declared axioms: Eq.rec
```

### Package Identity

The fixture uses the CLR-00 identity and profile strings:

```toml
schema = "npa.package.v0.1"
package = "npa-proof-corpus"
version = "0.1.0"
license = "MIT"

core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"
```

Policy is explicit, not derived from the current module list at write time:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

If the corpus gains a new axiom, CLR-02 tests must fail until the policy is
intentionally updated. The generator must not silently expand `allowed_axioms`
from whatever axioms happen to appear in generated modules.

### Legacy Field Mapping

For every legacy `[[proof_modules]]` entry, write one package `[[modules]]`
entry with this field mapping:

```text
module                         -> module
source                         -> source
certificate                    -> certificate
meta                           -> meta
replay                         -> replay
producer_profile               -> producer_profile
source_sha256                  -> expected_source_hash
certificate_file_sha256        -> expected_certificate_file_hash
export_hash                    -> expected_export_hash
axiom_report_hash              -> expected_axiom_report_hash
certificate_hash               -> expected_certificate_hash
imports                        -> imports
inductives                     -> inductives
definitions                    -> definitions
theorems                       -> theorems
axioms                         -> axioms
```

The package fixture must not copy legacy trust fields:

```text
trusted_status
verified_by_certificate
checker_result
```

`npa.package.v0.1` field names must match CLR-00 and CLR-01 exactly.

### External Std Import Artifacts

The current proof corpus imports `Std.Logic.Eq` and `Std.Nat.Basic` as
non-local modules. Legacy `proofs/manifest.toml` records only their names in
module import arrays, which is not enough for package identity.

CLR-02 must add deterministic import artifacts for those external modules:

```text
proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert
proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert
```

The package fixture must include one top-level `[[imports]]` entry for each
external Std module:

```toml
[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"
# export_hash and certificate_hash are required exact SHA-256 values
# generated from the canonical imported certificate artifact.

[[imports]]
module = "Std.Nat.Basic"
package = "npa-std"
version = "0.1.0"
certificate = "vendor/npa-std/Std/Nat/Basic/certificate.npcert"
# export_hash and certificate_hash are required exact SHA-256 values
# generated from the canonical imported certificate artifact.
```

The hashes come from the canonical certificates produced by the existing
`std_logic_eq_module` and `std_nat_basic_module` builder path. These import
artifacts are package fixtures and remain untrusted sidecars until checked by
certificate verification.

### Deterministic Serialization

`proofs/npa-package.toml` should be written with stable ordering:

```text
1. top-level scalar fields in the order shown in Package Identity
2. `[policy]`
3. `[[imports]]` sorted by module name
4. `[[modules]]` in the same module order as `proofs/manifest.toml`
5. module fields in the legacy mapping order above, using package field names
```

Array values preserve the order already used by the generator and checked-in
legacy manifest. The writer must not include timestamps, host paths, absolute
paths, Cargo target paths, registry URLs, or environment-dependent data.

### Generator Integration

`tools/proof-corpus` should keep a single source of truth for generated module
metadata. The recommended shape is:

```text
GeneratedModule
  source hash
  certificate file hash
  export hash
  axiom report hash
  certificate hash
  declaration summaries
  axioms
  verified module
  source interface

GeneratedExternalImport
  module
  package
  version
  certificate path
  export hash
  certificate hash
  verified module
  source interface
```

The generator may call `npa-package` serialization helpers if CLR-01 exposes
them. If CLR-01 only exposes parsing and validation, keep the writer in
`tools/proof-corpus` but validate the produced TOML with `npa-package` tests.

### Validation Test Strategy

Tests must prove four separate properties:

```text
package schema validation
  `proofs/npa-package.toml` parses and validates with `npa-package`

legacy parity
  every current legacy module is represented once with equivalent paths,
  hashes, imports, declaration summaries, and axiom summaries

artifact hash integrity
  package hash fields match the checked-in source and certificate bytes,
  including external Std import certificates

deterministic generation
  running `cargo run -p npa-proof-corpus` leaves `proofs/manifest.toml`,
  `proofs/npa-package.toml`, proof artifacts, and vendor import artifacts
  unchanged when inputs did not change
```

Parity tests must load `proofs/npa-package.toml` and `proofs/manifest.toml`.
They must not use `ExpectedModule` constants from
`tools/proof-corpus/tests/ai_proof_artifacts.rs` as the source of truth for the
package fixture.

---

## Tasks

### CLR-02-01 Audit Current Corpus Imports And Axioms

- Status: Completed
- Depends on: CLR-01
- Inputs:
  - `proofs/manifest.toml`
  - `proofs/Proofs/Ai/**`
  - `tools/proof-corpus/src/main.rs`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `doc/community-library-roadmap-clr-02-todo.md` if the audit changes known facts
- Deliverables:
  - A local versus external import classification for every module import in the current corpus.
  - Confirmation that the only external modules are `Std.Logic.Eq` and `Std.Nat.Basic`.
  - Confirmation that the package policy allowlist is exactly `["Eq.rec"]` for the current corpus.
- Acceptance criteria:
  - Every import in `proofs/manifest.toml` resolves either to a local `[[proof_modules]]` entry or to one of the top-level Std imports planned for `npa-package.toml`.
  - No package task depends on Rust constants to know which imports are external.
  - If a new external import or axiom is discovered, this document and the package fixture design are updated before implementation proceeds.
- Verification:
  - `rg -n "imports = \\[|axioms = \\[" proofs/manifest.toml`
  - `cargo test -p npa-proof-corpus`
- Notes:
  - Keep the audit focused on package fixture identity. Do not redesign the standard library here.
  - Completed audit source: `tools/proof-corpus/tests/manifest_package_audit.rs` reads
    `proofs/manifest.toml` as TOML and does not use `ExpectedModule` constants
    or `tools/proof-corpus/src/main.rs` module constants to classify imports.
  - Current manifest-derived classification:
    - local proof modules: 66
    - local import references: 261
    - external import references: 66
    - external modules: `Std.Logic.Eq` used by 63 modules and `Std.Nat.Basic`
      used by 3 modules
    - unknown imports outside local modules and planned top-level Std imports: none
  - Current manifest-derived axiom policy:
    - unique declared axioms: `Eq.rec`
    - modules declaring `Eq.rec`: 39
    - package policy allowlist for the current corpus remains exactly
      `["Eq.rec"]`

### CLR-02-02 Add Deterministic External Std Import Artifact Generation

- Status: Pending
- Depends on: CLR-02-01
- Inputs:
  - `std_logic_eq_module`
  - `std_nat_basic_module`
  - `verified_core_import_with_source_interface`
  - `crates/npa-cert/src/lib.rs`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/vendor/npa-std/**`
- Deliverables:
  - Generated certificate artifacts for `Std.Logic.Eq` and `Std.Nat.Basic` under `proofs/vendor/npa-std/`.
  - A small internal representation for generated external imports with module name, package, version, certificate path, export hash, certificate hash, verified module, and source interface.
  - Generator errors if a planned external Std import is missing from package output.
- Acceptance criteria:
  - External import certificate bytes are canonical `npa-cert` encodings.
  - Certificate paths are package-relative and match CLR-00 path rules.
  - Re-running the proof-corpus generator produces byte-identical Std import artifacts.
  - The generator still verifies every proof module with the same imported verified modules as before.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "vendor/npa-std|Std.Logic.Eq|Std.Nat.Basic" proofs tools/proof-corpus/src/main.rs`
- Notes:
  - Do not treat these artifacts as a registry cache. They are a checked-in fixture for package graph identity.

### CLR-02-03 Generate `proofs/npa-package.toml`

- Status: Pending
- Depends on: CLR-02-02
- Inputs:
  - CLR-00 target manifest shape
  - CLR-01 public schema constants and validator
  - generated proof modules
  - generated external Std imports
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `tools/proof-corpus/Cargo.toml`
  - `proofs/npa-package.toml`
- Deliverables:
  - Deterministic `npa.package.v0.1` writer for the current proof corpus.
  - Checked-in `proofs/npa-package.toml`.
  - Dependency on `npa-package` only if needed for schema constants, validated value construction, or serialization.
  - No copy of legacy `trusted_status` or generated trust claims into the package fixture.
- Acceptance criteria:
  - The fixture validates with `npa-package`.
  - The fixture includes all current proof modules exactly once.
  - The fixture includes the two Std external imports exactly once.
  - The fixture uses `expected_*` hash field names and keeps all values from the generated artifacts.
  - Existing `proofs/manifest.toml` is still generated with legacy schema and field names.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "schema = \"npa.package.v0.1\"|expected_certificate_hash" proofs/npa-package.toml`
  - `! rg -n "trusted_status|verified_by_certificate|checker_result|registry_url|latest|generated_at" proofs/npa-package.toml`
- Notes:
  - If a serialization helper is added to `npa-package`, keep it deterministic and free of filesystem access.

### CLR-02-04 Validate The Package Fixture With `npa-package`

- Status: Pending
- Depends on: CLR-02-03
- Inputs:
  - `proofs/npa-package.toml`
  - CLR-01 parser and validator API
- Code or documentation areas:
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
  - `tools/proof-corpus/Cargo.toml`
  - `crates/npa-package/tests/package_manifest.rs` only if fixture coverage belongs in the library crate
- Deliverables:
  - A test that reads `proofs/npa-package.toml` and validates it with `npa-package`.
  - Assertions that top-level schema, package identity, profile fields, and policy fields match CLR-00.
  - Assertions that forbidden package fields are absent from the package fixture.
- Acceptance criteria:
  - The package fixture test does not compile or inspect `tools/proof-corpus/src/main.rs`.
  - Validation errors are reported through `npa-package` structured errors.
  - The test fails if `trusted_status`, `verified_by_certificate`, `checker_result`, `registry_url`, `latest`, or `generated_at` appears in the package fixture.
- Verification:
  - `cargo test -p npa-proof-corpus package_fixture`
  - `cargo test -p npa-package package_manifest`
  - `rg -n "parse_and_validate_manifest_str|proofs/npa-package.toml|trusted_status" tools/proof-corpus/tests crates/npa-package/tests`
- Notes:
  - Prefer keeping full-corpus fixture tests in `npa-proof-corpus`; keep `npa-package` unit fixtures small unless shared coverage is useful.

### CLR-02-05 Add Legacy Manifest Versus Package Fixture Parity Tests

- Status: Pending
- Depends on: CLR-02-04
- Inputs:
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - legacy manifest parsing already used by proof-corpus tests
- Code or documentation areas:
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
- Deliverables:
  - A parity test that compares every legacy `[[proof_modules]]` entry with the matching package `[[modules]]` entry.
  - Field-by-field assertions for module names, source paths, certificate paths, meta paths, replay paths, producer profile, imports, hash fields, inductives, definitions, theorems, and axioms.
  - Assertions that the package fixture has no extra local modules and no missing local modules.
- Acceptance criteria:
  - The parity test derives expected module coverage from `proofs/manifest.toml`, not Rust constants.
  - Legacy hash fields map to package `expected_*` hash fields exactly.
  - Declaration and axiom arrays preserve legacy order.
  - The package fixture records enough import identity for CLR-03 to build source-free checker requests.
- Verification:
  - `cargo test -p npa-proof-corpus package_manifest_parity`
  - `cargo test -p npa-proof-corpus`
- Notes:
  - Keep the existing hard-coded artifact verification test until a later milestone replaces it with package-driven verification.

### CLR-02-06 Check Package Fixture Artifact Hashes

- Status: Pending
- Depends on: CLR-02-05
- Inputs:
  - `proofs/npa-package.toml`
  - checked-in source files
  - checked-in certificate files
  - checked-in Std vendor certificate files
  - `npa-cert` hash helpers
- Code or documentation areas:
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
- Deliverables:
  - Hash integrity tests for every package module source and certificate file.
  - Hash integrity tests for every top-level external import certificate file.
  - Assertions that export hash, axiom report hash, and certificate hash values are parseable package hashes.
- Acceptance criteria:
  - A stale source, stale certificate, stale vendor import artifact, or mistyped hash fails deterministically.
  - The test uses package paths relative to `proofs/`.
  - The test does not read `.npa` source while checking external Std imports, because their package identity comes from certificate artifacts.
- Verification:
  - `cargo test -p npa-proof-corpus package_fixture_hashes`
  - `cargo test -p npa-proof-corpus`
- Notes:
  - Full certificate replay and source-free checker execution remain CLR-03 and CLR-04 work.

### CLR-02-07 Preserve Generator Determinism

- Status: Pending
- Depends on: CLR-02-06
- Inputs:
  - `tools/proof-corpus/src/main.rs`
  - generated files under `proofs/`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/vendor/npa-std/**`
- Deliverables:
  - Existing generation behavior preserved for all current proof artifacts.
  - Determinism check documented for both legacy and package outputs.
  - No environment-dependent fields in either generated manifest.
- Acceptance criteria:
  - `cargo run -p npa-proof-corpus` followed by `git diff --exit-code -- proofs tools/proof-corpus` is clean when no inputs changed.
  - Existing `ai_certificates_match_manifest_and_verify` coverage still passes.
  - Package fixture tests pass after regeneration.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `git diff --exit-code -- proofs tools/proof-corpus`
  - `cargo test -p npa-proof-corpus`
- Notes:
  - If local generation rewrites artifacts because of an intentional format change, commit the generated output in the same implementation milestone.

### CLR-02-08 Document CLR-03 Handoff Data

- Status: Pending
- Depends on: CLR-02-07
- Inputs:
  - `proofs/npa-package.toml`
  - `doc/community-library-roadmap-todo.md` CLR-03
  - `doc/community-library-roadmap-clr-02-todo.md`
- Code or documentation areas:
  - `doc/community-library-roadmap-todo.md`
  - `proofs/README.md` if it describes proof corpus artifacts
- Deliverables:
  - A concise note that CLR-03 should derive source-free checker import locks from `proofs/npa-package.toml`, not from `proofs/manifest.toml` or Rust constants.
  - Documentation of the external Std import artifact paths and hash fields.
  - Any updated verification command examples needed for the package fixture.
- Acceptance criteria:
  - A later CLR-03 implementation can identify local modules, external imports, certificate paths, export hashes, certificate hashes, and axiom report hashes from the package fixture alone.
  - Documentation continues to state that `proofs/manifest.toml` is legacy compatibility output.
  - No registry or online dependency behavior is implied.
- Verification:
  - `rg -n "npa-package.toml|vendor/npa-std|manifest.toml|source-free" doc proofs/README.md`
  - `git diff --check`
- Notes:
  - Keep this documentation small and factual; the package lock format itself is CLR-03.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: legacy manifest imports `Std.Logic.Eq` and `Std.Nat.Basic` by name only,
which is insufficient for the CLR-00 external import identity contract.
Fix: CLR-02-02 requires deterministic vendor import certificate artifacts and
top-level hash-pinned `[[imports]]` entries.

Finding: deriving `allowed_axioms` from the generated module list would silently
permit newly introduced axioms.
Fix: Package policy is fixed to `["Eq.rec"]`; tests must fail until policy is
intentionally changed.

Finding: package fixture tests could accidentally keep using Rust constants as
the module graph source of truth.
Fix: CLR-02-04 and CLR-02-05 require loading `proofs/npa-package.toml` and
`proofs/manifest.toml` directly, without consulting `tools/proof-corpus` module
constants.

Finding: copying legacy `trusted_status` into `npa.package.v0.1` would conflict
with CLR-00 forbidden fields and blur the certificate trust boundary.
Fix: CLR-02-03 and CLR-02-04 require absence checks for legacy trust fields.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now has explicit dependencies,
fixture locations, external import identity, policy behavior, validation
coverage, determinism checks, and CLR-03 handoff data.
```
