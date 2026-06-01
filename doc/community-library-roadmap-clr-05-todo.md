# Community Library Roadmap CLR-05 Todo

Source: `doc/community-library-roadmap-todo.md` CLR-05

CLR-05 adds deterministic package metadata artifacts for axiom review and
theorem discovery. It introduces package-level generated JSON files for
axiom reports and theorem indexes, plus `npa package axiom-report` and
`npa package index` commands to generate or check those files.

These artifacts are useful for CI, documentation, search, review, and later
publish metadata. They are not proof evidence. Proof acceptance remains the
canonical certificate plus the selected source-free checker verdict.

---

## Scope

対象:

```text
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `npa.package.axiom_report.v0.1`
- `npa.package.theorem_index.v0.1`
- package-level canonical JSON writers and parsers for these artifacts
- source-free extraction from checked package certificate artifacts
- package axiom policy evaluation against generated report contents
- theorem entries for public theorem and axiom exports
- deterministic ordering for modules, axioms, declarations, theorem entries, tags, and checker summaries
- `npa package axiom-report`
- `npa package index`
- check mode for both commands
- write mode for both commands
- CLI diagnostics and tests for deterministic generated artifacts
```

非対象:

```text
- `npa package publish-plan`
- registry module entries
- registry server, registry lookup, dependency solving, network fetch, binary cache, or latest-version resolution
- changed-module or reverse-dependency selection
- external checker binary as a required pass condition
- source-to-certificate build
- package lock generation internals already owned by CLR-03
- package check, build-certs, verify-certs, and check-hashes commands already owned by CLR-04
- Human pretty statement rendering from `.npa` source
- source, replay, meta, theorem graph score, AI trace, prompt metadata, or tactic trace indexing
- rewrite profile, simp profile, prompt metadata, or full Phase 6 Std release artifact generation
- publish metadata checks owned by CLR-06
```

Trusted-boundary rule:

```text
The package axiom report and theorem index are untrusted generated metadata.
They may summarize certificate-derived information, checker summaries, package
policy results, and declaration identities, but they never become checker input
and never replace canonical certificate verification.

CLR-05 artifact generation must not require `.npa` source, replay files,
meta files, AI traces, registry data, package solver state, or network access.
It starts from the validated package manifest, generated package lock,
certificate artifacts, and source-free package verifier output or in-process
source-free certificate verification used only for metadata extraction.

The kernel, `npa-cert`, and `npa-checker-ref` must not depend on `npa-cli`.
Generated package metadata must not move filesystem, network, registry,
plugin, AI, or theorem-search behavior into the trusted kernel.
```

---

## Implementation Specification

### Artifact Locations

CLR-05 owns these checked package artifacts:

```text
generated/axiom-report.json
generated/theorem-index.json
```

For the in-repo proof corpus fixture, those paths resolve to:

```text
proofs/generated/axiom-report.json
proofs/generated/theorem-index.json
```

Only the CLR-05 write commands write these files. `package build-certs` from
CLR-04 must continue to avoid writing them.

### Schema Names

Use the schema constants fixed by CLR-00 and exposed by CLR-01:

```text
npa.package.axiom_report.v0.1
npa.package.theorem_index.v0.1
```

These schemas are distinct from:

```text
npa.package.lock.v0.1
npa.package.publish_plan.v0.1
npa.independent-checker.axiom_report.v1
npa.std-library.std-axiom-report.v1
npa.std-library.std-theorem-index.v1
```

The package artifacts may reuse projection logic from existing Std and
independent-checker code, but they must not reuse schema names that imply a
Std-only or independent-checker artifact.

### Crate Placement And Dependency Boundaries

Recommended code ownership:

```text
crates/npa-package
  owns package artifact data models, schema constants, canonical JSON writers,
  parsers, and deterministic validation for `npa.package.axiom_report.v0.1`
  and `npa.package.theorem_index.v0.1`

crates/npa-api
  may own reusable package artifact extraction adapters when they need existing
  certificate projection, theorem graph, or search-index helper logic

crates/npa-cli
  owns filesystem loading, command parsing, check/write behavior, and command
  diagnostics for `package axiom-report` and `package index`
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

If extraction helpers live in `npa-api`, keep `npa-package` as the pure
artifact model and canonicalization crate. Do not make `npa-package` depend on
API search, session, tactic, or theorem graph modules.

### Command Contract

Cargo development commands:

```sh
cargo run -p npa-cli -- package axiom-report --root proofs --check
cargo run -p npa-cli -- package index --root proofs --check
```

Contributor-facing commands after installation:

```sh
npa package axiom-report --root proofs --check
npa package index --root proofs --check
```

Write mode:

```sh
cargo run -p npa-cli -- package axiom-report --root proofs
cargo run -p npa-cli -- package index --root proofs
```

Common flags inherited from CLR-04:

```text
--root PATH
  Package root. Defaults to the current directory only. The CLI must not search
  parent directories or registry locations.

--json
  Emit deterministic JSON diagnostics using `npa.package.command_result.v0.1`.

--check
  Generate in memory and fail if the checked-in generated artifact would
  change. No files are written.
```

Unsupported in CLR-05:

```text
--changed
--all
--checker external
--registry
--network
--update-manifest-hashes
--include-source
--include-replay
--include-ai-traces
```

Full-package operation is the only CLR-05 scope. Changed-module selection is a
later CI feature and must not be smuggled into the artifact schema.

### Exit Codes And Diagnostics

Use the CLR-04 process exit codes:

```text
0  command succeeded
1  package validation, stale artifact, axiom policy, extraction, or checker failure
2  CLI usage error or unexpected internal command failure
```

Extend the `npa.package.command_result.v0.1` diagnostic vocabulary with these
categories if CLR-04 did not already add them:

```text
AxiomReport
TheoremIndex
GeneratedArtifact
PackagePolicy
```

Required reason codes:

```text
axiom_report_missing
axiom_report_stale
axiom_report_hash_mismatch
axiom_report_policy_violation
axiom_report_non_canonical_order
theorem_index_missing
theorem_index_stale
theorem_index_hash_mismatch
theorem_index_non_canonical_order
theorem_index_entry_mismatch
generated_artifact_changed
generated_artifact_write_failed
metadata_extraction_failed
source_artifact_read_attempted
unsupported_metadata_field
```

Diagnostics must identify the package-relative path, module, declaration, field,
expected hash, and actual hash whenever applicable.

### Input Loading

Both commands use the package root loader from CLR-04 and then load:

```text
npa-package.toml
generated/package-lock.json
local and external certificate artifacts referenced by the package lock
existing generated/axiom-report.json only in axiom-report check mode
existing generated/theorem-index.json only in index check mode
```

Both commands must not read:

```text
.npa source files
replay.json
meta.json
Human debug JSON
theorem graph scores
AI traces
tactic traces
prompt metadata
registry metadata
network resources
package solver state
```

The implementation should share one source-free artifact extraction pipeline so
both commands derive module identities, verified modules, axiom data, public
exports, and checker summaries from the same package graph and lock.

### Artifact Extraction Pipeline

The shared extraction pipeline should run in this order:

```text
1. Load and validate `npa-package.toml`.
2. Load and validate `generated/package-lock.json`.
3. Rebuild the in-memory package lock from certificate bytes and compare it
   with the checked-in lock.
4. Verify certificates in dependency-topological order using the source-free
   package verifier API from CLR-03.
5. Collect verified module outputs, module hashes, direct imports, declared
   exports, axiom reports, and checker summaries.
6. Project package axiom report and theorem index values.
7. Canonicalize generated JSON and compute self hashes.
```

The verifier result used for metadata extraction may be fast-kernel mode if
that is the only mode returning `VerifiedModule` data to the Rust process.
If a reference checker summary is included, it must be labeled as reference and
must come from a successful source-free reference verification path. Do not
label fast-kernel extraction as a reference checker verdict.

If certificate verification fails, metadata generation fails. The failure still
does not mean the generated metadata is proof evidence; it only means safe
extraction could not be performed.

### Package Axiom Report Schema

The package axiom report JSON has stable top-level fields:

```text
schema
package
version
manifest
package_lock
policy
modules
checker_summaries
summary
package_axiom_report_hash
```

`manifest` fields:

```text
path
file_hash
```

`package_lock` fields:

```text
path
file_hash
```

`policy` fields:

```text
allow_custom_axioms
allowed_axioms
```

`modules` is sorted by module dotted name. Each module entry contains:

```text
module
origin
export_hash
certificate_hash
axiom_report_hash
certificate_file_hash
direct_axioms
transitive_axioms
policy_status
```

`origin` is:

```text
local
external
```

Each axiom reference contains:

```text
module
name
export_hash
decl_interface_hash
```

`direct_axioms` are the axioms declared by or directly used inside the module
certificate report. `transitive_axioms` include the direct axiom set plus
dependency axiom reports from imported modules. Both lists are sorted by
canonical axiom reference bytes and deduplicated.

`policy_status` fields:

```text
status
violations
```

`status` is:

```text
ok
violation
```

Each violation contains:

```text
axiom
reason_code
```

Required policy violation reason codes:

```text
custom_axiom_disallowed
axiom_not_allowlisted
sorry_disallowed
```

The package summary contains:

```text
module_count
local_module_count
external_module_count
direct_axiom_count
transitive_axiom_count
policy_violation_count
```

`checker_summaries` are sorted by module, checker mode, checker id, and profile.
Each summary contains:

```text
module
checker
profile
mode
status
export_hash
certificate_hash
axiom_report_hash
```

`checker_summaries` are informational metadata. They must not be accepted as a
replacement for rerunning `package verify-certs`.

`package_axiom_report_hash` is the self hash of the package axiom report
canonical bytes excluding the self-hash field. It is not the same value as any
single certificate's `axiom_report_hash`.

### Package Theorem Index Schema

The package theorem index JSON has stable top-level fields:

```text
schema
package
version
manifest
package_lock
index_profile
entries
checker_summaries
summary
theorem_index_hash
```

`index_profile` should start as:

```text
npa.package.theorem_index.v0.1.certificate_derived
```

The index is certificate-derived and source-free. It does not include pretty
statements, source spans, comments, tactic scripts, replay traces, proof-search
scores, theorem graph scores, or natural-language descriptions.

`entries` contains public exports whose certificate export kind is:

```text
theorem
axiom
```

Definitions, inductives, constructors, and recursors are not theorem index
entries in CLR-05 unless a later milestone explicitly expands the artifact into
a full declaration index.

Each theorem index entry contains:

```text
global_ref
kind
statement
modes
tags
axiom_dependencies
module_axiom_report_hash
artifact
```

`global_ref` fields:

```text
module
name
export_hash
certificate_hash
decl_interface_hash
```

`kind` is:

```text
theorem
axiom
```

`statement` fields:

```text
core_hash
head
constants
```

`head` is either null or a global reference view derived from the statement
term. `constants` is sorted by canonical global reference view bytes and
deduplicated. The entry must not include source text or a pretty-printed
statement in CLR-05.

`modes` contains deterministic search-mode hints derived from certificate data:

```text
exact
apply
rw
simp
```

`exact` is present for every theorem or axiom entry. `apply` may be derived from
a leading Pi statement. `rw` and `simp` are present only when deterministic
package metadata or certificate-derived profiles are available. If no such
metadata is available in CLR-05, omit them rather than guessing from theorem
names.

`tags` contains deterministic package tags only when they are present in
`npa-package.toml` or another CLR-05-owned deterministic package metadata input.
Do not infer tags from file paths, comments, theorem names, or AI sidecars.

`axiom_dependencies` is sorted by canonical axiom reference bytes and
deduplicated.

`module_axiom_report_hash` is the module certificate's axiom report hash.

`artifact` fields:

```text
origin
certificate
```

`certificate` is the package-relative certificate path. It is an artifact
locator, not a source locator.

The summary contains:

```text
entry_count
theorem_count
axiom_count
module_count
entries_with_axioms_count
```

`theorem_index_hash` is the self hash of theorem index canonical bytes excluding
the self-hash field.

### Canonical JSON Rules

Generated JSON must be canonical:

```text
- stable object field order as specified by the schema
- arrays sorted by canonical bytes unless the schema explicitly says package graph order
- no timestamps
- no absolute paths
- no host names
- no environment variables
- no Cargo target paths
- no registry URLs
- no nondeterministic map iteration
- no source text
- no replay, meta, tactic, prompt, theorem graph score, or AI trace payloads
```

File hashes are exact SHA-256 hashes of the checked file bytes. Self hashes are
computed over schema-defined canonical bytes, not over pretty-printed JSON with
the self-hash field included.

### Check Mode And Write Mode

Check mode for both commands:

```text
1. Generate the artifact in memory.
2. Read the checked-in generated artifact.
3. Parse and validate the checked-in artifact.
4. Compare canonical JSON bytes, self hash, and file hash.
5. Exit with code 1 if the artifact is missing, stale, non-canonical, or policy-invalid.
6. Write no files.
```

Write mode for both commands:

```text
1. Generate the artifact in memory.
2. Validate the generated artifact by parsing it back through the schema parser.
3. Write only the command-owned generated JSON file.
4. Use an atomic or all-or-nothing write strategy where practical.
5. Leave the file unchanged if bytes are identical.
```

Write mode must not write:

```text
npa-package.toml
generated/package-lock.json
source files
certificate files
replay files
meta files
publish-plan artifacts
registry artifacts
AI sidecars
```

### Policy Behavior

`package axiom-report` is the CI policy gate for axiom usage. It fails when:

```text
- any direct or transitive axiom violates package policy
- `sorry` or a `sorry`-equivalent axiom appears
- a module axiom report hash differs from the package lock
- a module's projected axiom report differs from its certificate report
- canonical axiom order or deduplication is invalid
```

Package policy comes from `npa-package.toml`:

```text
allow_custom_axioms
allowed_axioms
```

The current proof-corpus fixture is expected to allow only the existing
`Eq.rec` exception. The generator must not silently expand the allowlist when
new axioms appear.

### Interaction With CLR-04

CLR-05 commands should reuse these CLR-04 pieces:

```text
- package root loader
- structured command result renderer
- package lock freshness checks
- certificate hash checks
- source-free package verifier API
- CLI integration test fixture
```

CLR-05 commands should not duplicate CLI argument parsing or introduce a second
diagnostic schema.

Before `axiom-report --check` or `index --check` reports success, stale package
lock or stale certificate hashes must already have failed. The generated
artifacts should never mask a failed `check-hashes` or `verify-certs` result.

### Handoff To Later Milestones

CLR-06 consumes:

```text
generated/axiom-report.json
generated/theorem-index.json
generated/package-lock.json
source-free checker summaries
```

CLR-06 owns:

```text
generated/publish-plan.json
npa.package.publish_plan.v0.1
npa.registry.module.v0.1 seed entries
release artifact list
```

CLR-07 owns:

```text
external theorem library CI templates
changed-module CI selection
release workflow examples
```

CLR-08 owns:

```text
external checker required mode
verified_high_trust artifacts
external checker disagreement gates
```

---

## Tasks

### CLR-05-01 Define Package Artifact Schemas And Canonical Models

- Status: Completed
- Depends on: CLR-04
- Inputs:
  - schema constants from CLR-00 and CLR-01
  - `doc/community-library-roadmap-todo.md` CLR-05
  - existing `MachineStdAxiomReport` and `MachineStdTheoremIndex` field shapes
  - existing independent-checker axiom report hash behavior
- Code or documentation areas:
  - `crates/npa-package/src/schema.rs`
  - `crates/npa-package/src/artifacts.rs`
  - `crates/npa-package/src/axiom_report.rs`
  - `crates/npa-package/src/theorem_index.rs`
  - `crates/npa-package/tests/package_artifacts.rs`
- Deliverables:
  - Public data model for package axiom report artifacts.
  - Public data model for package theorem index artifacts.
  - Canonical JSON writer for both artifact schemas.
  - Parser and validator for checked-in generated artifacts.
  - Self-hash computation for package axiom report and theorem index.
  - Structured artifact validation errors.
- Acceptance criteria:
  - Schema names are exactly `npa.package.axiom_report.v0.1` and `npa.package.theorem_index.v0.1`.
  - Object field order and array order are schema-defined.
  - Self hashes exclude the self-hash field.
  - Parsers reject unknown fields, duplicate entries, non-canonical order, stale self hashes, absolute paths, timestamps, host paths, registry URLs, and source payload fields.
  - The package artifact model does not use Std-only schema strings or independent-checker schema strings.
- Verification:
  - `cargo test -p npa-package package_axiom_report_schema`
  - `cargo test -p npa-package package_theorem_index_schema`
  - `cargo test -p npa-package package_artifact_canonical_json`
  - `rg -n "npa.package.axiom_report.v0.1|npa.package.theorem_index.v0.1" crates/npa-package doc/community-library-roadmap-clr-05-todo.md`
- Notes:
  - Keep `npa-package` free of filesystem loading as the primary API. CLI reads files and passes bytes or strings into the library.

### CLR-05-02 Implement Source-Free Package Artifact Extraction

- Status: Completed
- Depends on: CLR-05-01
- Inputs:
  - CLR-03 package lock and source-free verifier APIs
  - CLR-04 package root loader and hash checks
  - `npa_cert::VerifiedModule`
  - `crates/npa-api/src/theorem_graph.rs`
  - `crates/npa-api/src/std_library.rs`
- Code or documentation areas:
  - `crates/npa-api/src/package_artifacts.rs` if extraction lives in `npa-api`
  - `crates/npa-cli/src/package_artifacts.rs` if extraction stays CLI-local
  - `crates/npa-package/src/artifacts.rs`
  - tests proving no source/replay/meta/AI reads occur
- Deliverables:
  - Shared extraction input type built from a validated package manifest and package lock.
  - Source-free certificate loading and verification in package topological order.
  - Verified module collection keyed by module, export hash, and certificate hash.
  - Checker summary projection with explicit mode labels.
  - File access guard or tests that fail if source, replay, meta, theorem index, or AI trace files are read.
- Acceptance criteria:
  - Extraction reads only package metadata, package lock, certificate artifacts, and generated artifact files when check mode asks for them.
  - Extraction rejects stale package lock, stale certificate hashes, missing certificates, missing imports, and verifier failures before generating final artifacts.
  - Fast-kernel extraction summaries are not labeled as reference checker verdicts.
  - Reference checker summaries, if included, come only from the CLR-03 source-free reference verifier path.
  - Extraction does not add dependencies from trusted crates to `npa-cli`.
- Verification:
  - `cargo test -p npa-api package_artifact_extraction`
  - `cargo test -p npa-cli package_artifact_source_free_boundary`
  - `cargo test -p npa-package package_lock`
  - `cargo tree -p npa-package`
- Notes:
  - If `npa-api` is used, keep the adapter narrow and independent of API session state, tactic execution, and AI search controllers.

### CLR-05-03 Generate Package Axiom Report Values

- Status: Pending
- Depends on: CLR-05-02
- Inputs:
  - source-free extraction output
  - package policy from `npa-package.toml`
  - module certificate axiom reports from `npa-cert`
  - package lock module hashes
- Code or documentation areas:
  - `crates/npa-package/src/axiom_report.rs`
  - `crates/npa-api/src/package_artifacts.rs` or CLI-local artifact projection module
  - `crates/npa-package/tests/package_axiom_report.rs`
- Deliverables:
  - Projection from verified package modules to `npa.package.axiom_report.v0.1`.
  - Direct axiom and transitive axiom computation for local and external modules.
  - Package policy evaluation with per-module policy status.
  - Package summary counts.
  - Deterministic checker summary ordering.
- Acceptance criteria:
  - Module entries are sorted by module dotted name.
  - Axiom references are sorted and deduplicated by canonical axiom reference bytes.
  - Direct and transitive axiom sets are derived from verified certificates and package import order, not source files.
  - A new non-allowlisted axiom causes a policy failure instead of silently updating `allowed_axioms`.
  - The current proof-corpus fixture passes with the expected `Eq.rec` policy after CLR-02 through CLR-04 are complete.
- Verification:
  - `cargo test -p npa-package package_axiom_report`
  - `cargo test -p npa-api package_axiom_report_projection`
  - `cargo test --workspace axiom_report`
- Notes:
  - Do not use legacy `proofs/manifest.toml` as the source of truth. Use `proofs/npa-package.toml` and `generated/package-lock.json`.

### CLR-05-04 Implement `package axiom-report`

- Status: Pending
- Depends on: CLR-05-03
- Inputs:
  - CLR-04 `npa-cli` package command framework
  - package axiom report generator
  - existing generated report path `generated/axiom-report.json`
- Code or documentation areas:
  - `crates/npa-cli/src/package_axiom_report.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/tests/package_cli.rs`
  - `proofs/generated/axiom-report.json`
- Deliverables:
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check`.
  - `cargo run -p npa-cli -- package axiom-report --root proofs`.
  - Check mode that compares generated canonical JSON with checked-in output.
  - Write mode that updates only `generated/axiom-report.json`.
  - JSON and human diagnostics for stale report and policy failure.
- Acceptance criteria:
  - `--check` writes no files.
  - Write mode writes only `generated/axiom-report.json`.
  - Missing, stale, non-canonical, or policy-invalid reports fail with exit code `1`.
  - Usage errors and unsupported flags fail with exit code `2`.
  - The command does not read source, replay, meta, theorem index, publish plan, registry, or AI files.
  - The command result uses `npa.package.command_result.v0.1`.
- Verification:
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check`
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check --json`
  - `cargo test -p npa-cli package_axiom_report`
  - `git diff --exit-code -- proofs/generated/axiom-report.json`
- Notes:
  - This command may reuse CLR-04 hash and verification helpers, but it must not rewrite the package lock or manifest.

### CLR-05-05 Generate Package Theorem Index Values

- Status: Pending
- Depends on: CLR-05-02
- Inputs:
  - source-free extraction output
  - `npa_cert::ExportEntry`
  - existing search theorem mode concepts from `crates/npa-api/src/search.rs`
  - theorem graph and Std theorem index projection helpers where reusable
- Code or documentation areas:
  - `crates/npa-package/src/theorem_index.rs`
  - `crates/npa-api/src/package_artifacts.rs` or CLI-local artifact projection module
  - `crates/npa-package/tests/package_theorem_index.rs`
- Deliverables:
  - Projection from verified package modules to `npa.package.theorem_index.v0.1`.
  - Theorem and axiom public export entries with global reference identity.
  - Statement core hash, statement head, constants, modes, tags, and axiom dependencies.
  - Package index summary counts.
  - Deterministic theorem index self hash.
- Acceptance criteria:
  - Index entries include public theorem and axiom exports only.
  - Every entry includes module, theorem or axiom name, `decl_interface_hash`, `export_hash`, `certificate_hash`, module `axiom_report_hash`, statement core hash, modes, tags, and axiom dependencies.
  - Entries are sorted by canonical global reference bytes.
  - Modes and tags are omitted when deterministic source-free inputs are unavailable; they are never inferred from theorem names, comments, file paths, or AI sidecars.
  - No source text, pretty statement, replay trace, tactic trace, prompt metadata, theorem graph score, or AI score appears in the artifact.
- Verification:
  - `cargo test -p npa-package package_theorem_index`
  - `cargo test -p npa-api package_theorem_index_projection`
  - `cargo test --workspace theorem_index`
- Notes:
  - Existing `crates/npa-api/src/search.rs` has an internal theorem index for sessions. CLR-05 needs a package artifact, not a session-local search response.

### CLR-05-06 Implement `package index`

- Status: Pending
- Depends on: CLR-05-05
- Inputs:
  - CLR-04 `npa-cli` package command framework
  - package theorem index generator
  - existing generated index path `generated/theorem-index.json`
- Code or documentation areas:
  - `crates/npa-cli/src/package_index.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/tests/package_cli.rs`
  - `proofs/generated/theorem-index.json`
- Deliverables:
  - `cargo run -p npa-cli -- package index --root proofs --check`.
  - `cargo run -p npa-cli -- package index --root proofs`.
  - Check mode that compares generated canonical JSON with checked-in output.
  - Write mode that updates only `generated/theorem-index.json`.
  - JSON and human diagnostics for stale, missing, or non-canonical theorem index.
- Acceptance criteria:
  - `--check` writes no files.
  - Write mode writes only `generated/theorem-index.json`.
  - The command fails if package lock, certificates, or generated index are stale.
  - The command does not read source, replay, meta, axiom report input beyond the generated axiom report only if the implementation intentionally cross-checks it, publish plan, registry, or AI files.
  - The command result uses `npa.package.command_result.v0.1`.
- Verification:
  - `cargo run -p npa-cli -- package index --root proofs --check`
  - `cargo run -p npa-cli -- package index --root proofs --check --json`
  - `cargo test -p npa-cli package_index`
  - `git diff --exit-code -- proofs/generated/theorem-index.json`
- Notes:
  - If the index command cross-checks `generated/axiom-report.json`, stale axiom report diagnostics must be clear. It must not require source files to rebuild that report.

### CLR-05-07 Add Determinism, Boundary, And Negative Fixture Tests

- Status: Pending
- Depends on: CLR-05-04, CLR-05-06
- Inputs:
  - proof-corpus package fixture
  - package CLI test fixture from CLR-04
  - generated artifact parsers
  - temp package fixtures
- Code or documentation areas:
  - `crates/npa-cli/tests/package_cli.rs`
  - `crates/npa-package/tests/package_artifacts.rs`
  - `crates/npa-cli/tests/fixtures/package`
  - `proofs/generated/axiom-report.json`
  - `proofs/generated/theorem-index.json`
- Deliverables:
  - End-to-end tests for both CLR-05 commands in check mode.
  - Write-mode tests in temp package copies.
  - Negative tests for stale artifact, non-canonical order, stale self hash, disallowed axiom, missing certificate, stale package lock, unsupported flags, and attempted source read.
  - Snapshot-like assertions for JSON diagnostic shape without host-specific paths.
- Acceptance criteria:
  - Running both commands twice without input changes is idempotent.
  - A fresh checkout can run both check commands successfully after generated artifacts are checked in.
  - Removing `.npa`, replay, and meta files from a temp package does not break artifact generation when certificates and package metadata remain valid.
  - Invalid generated artifacts fail with exit code `1`; unsupported flags fail with exit code `2`.
  - Tests do not mutate checked-in `proofs/` except when a specific generation command is intentionally run and then checked for a clean diff.
- Verification:
  - `cargo test -p npa-package package_artifacts`
  - `cargo test -p npa-cli package_axiom_report`
  - `cargo test -p npa-cli package_index`
  - `cargo test --workspace axiom_report`
  - `cargo test --workspace theorem_index`
- Notes:
  - Boundary tests should be explicit because artifact generation is tempting to implement by reading legacy `proofs/manifest.toml` or `meta.json`.

### CLR-05-08 Update Documentation And CLR-06 Handoff

- Status: Pending
- Depends on: CLR-05-07
- Inputs:
  - `doc/community-library-roadmap-todo.md`
  - `doc/community-library-roadmap.md`
  - `README.md`
  - `proofs/README.md`
  - generated artifact schemas from CLR-05
- Code or documentation areas:
  - README package command examples
  - `proofs/README.md`
  - `doc/community-library-roadmap-todo.md`
  - package artifact schema docs if added under `crates/npa-package`
- Deliverables:
  - Contributor-facing examples for `npa package axiom-report --check` and `npa package index --check`.
  - Documentation that generated axiom report and theorem index are metadata, not proof evidence.
  - Documentation of source-free artifact generation boundaries.
  - CLR-06 handoff note listing fields publish-plan can consume.
- Acceptance criteria:
  - Docs do not imply generated metadata is checker input.
  - Docs do not imply source, replay, meta, theorem graph score, prompt metadata, or AI traces are required for CLR-05 artifacts.
  - Docs distinguish package axiom report schema from independent-checker and Std-only axiom report schemas.
  - Parent roadmap points to this detailed CLR-05 task document.
- Verification:
  - `rg -n "axiom-report|theorem-index|npa.package.axiom_report.v0.1|npa.package.theorem_index.v0.1|metadata, not proof evidence|source-free" README.md doc proofs/README.md`
  - `git diff --check`
- Notes:
  - Keep `publish-plan` examples marked as CLR-06 until that command is implemented.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: The parent CLR-05 milestone names package verification results and
checker summaries, but generated metadata could be mistaken for proof evidence.
Fix: The specification labels generated artifacts and checker summaries as
metadata only, requires source-free verifier inputs, and keeps `verify-certs`
as the proof acceptance gate.

Finding: Existing code has Std-specific and independent-checker axiom report
schemas, but CLR-05 needs a package schema for external theorem libraries.
Fix: CLR-05 defines distinct package schema names and requires parsers to
reject Std-only or independent-checker schema names for package artifacts.

Finding: The theorem index could accidentally depend on source, pretty
statements, meta files, search scores, or AI sidecars.
Fix: The theorem index schema is certificate-derived and source-free. Pretty
statements, source spans, theorem graph scores, prompt metadata, replay, and AI
traces are explicitly out of scope.

Finding: The parent milestone requires modes and tags if available, but the
current package contract may not have deterministic per-theorem tags.
Fix: The task document allows modes and tags only from deterministic
source-free package inputs and forbids inference from names, comments, paths,
or AI sidecars.

Finding: Axiom policy checks could silently normalize current axioms into the
allowlist.
Fix: CLR-05 requires policy failures for new non-allowlisted axioms and states
that the generator must not expand `allowed_axioms`.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now fixes package artifact schemas,
source-free extraction, axiom report policy checks, theorem index projection,
CLI check/write behavior, deterministic tests, and CLR-06 handoff boundaries.
```
