# Proof Corpus Package Audit Speed Plan

Date: 2026-06-08

This document extends `develop/proof-corpus-tooling-improvement-plan.md` from
the theorem authoring loop to package verification, promotion readiness, and
closure audit workflows. The goal is to apply Go-style package compilation
ideas to NPA audit time without changing the certificate-first trust boundary.

The useful analogy is:

```text
Go package export data      -> NPA verified export summary
Go build cache              -> NPA content-addressed checker result store
Go package dependency graph -> NPA package lock / certificate import DAG
Go stale package detection  -> NPA export_hash and certificate_hash invalidation
```

## 1. Scope

In scope:

- `npa package verify-certs` and package verifier fixture speedups.
- `npa-mathlib` promotion readiness and closure audit local loops.
- Source-free package graph traversal based on package lock entries and
  certificate-declared imports.
- Deterministic local cache and summary artifacts that shorten repeated local
  audits.
- Measurement of package gate and closure audit bottlenecks.

Out of scope:

- Changing proof acceptance criteria.
- Treating cache hits, timing logs, theorem indexes, promotion plans, or export
  summary files as proof evidence.
- Reading `.npa` source, replay files, metadata, AI traces, registry network
  data, or hidden package caches as checker inputs.
- Skipping the final source-free checker gate for release, high-trust, or public
  `npa-mathlib` handoff.
- Adding filesystem, network, registry, plugin, or package-manager behavior to
  the kernel, `npa-cert`, or `npa-checker-ref`.

## 2. Trust Boundary

This plan does not move the trusted boundary.

```text
Not trusted:
  cache file
  verified export summary
  package lock
  package audit plan
  promotion plan
  theorem index
  timing log
  AI / tactic / replay / metadata

Trusted:
  canonical certificate bytes
  deterministic hashes
  small Rust kernel
  selected source-free checker / verifier verdict
```

Every cacheable artifact must be reproducible from checked-in source-free
certificate artifacts and deterministic policy inputs. Deleting all package
audit cache files must not change any accepted or rejected proof result.

Release, high-trust, and public `npa-mathlib` handoff gates keep cache disabled
by default. A release-like command may use `read-through` mode only when it still
executes the live checker and treats the live checker result as authoritative.

## 3. Current Bottleneck

PCT-08 moved ordinary theorem authoring away from the full corpus gate. The
remaining slow path is the package gate:

```text
./scripts/check-corpus-package.sh
```

That gate intentionally covers package verifier behavior, package CLI examples,
axiom-report, theorem index, and publish-plan regressions. It must remain out of
the theorem repair hot path, but package/promotion/closure audit work still pays
for repeated source-free verification of the same certificate graph.

The next speedup layer should focus on repeated package audit inputs, not on
changing the final verification obligation.

## 4. Proposed Techniques

### 4.1 Package Checker Result Store

Add a content-addressed package checker result store for local audit loops. The
store records that a specific checker profile accepted or rejected a specific
certificate artifact under a specific package/import/policy context.

Proposed location:

```text
target/npa-package-audit-cache/results-v0.1/<cache-key>.json
```

Minimum cache key material:

```text
schema
core_spec
certificate_format
package_lock_hash
package_policy_hash
checker_mode
checker_id
checker_version
checker_build_hash
checker_profile
runner_policy_hash, when checker = external
module
certificate_file_hash
certificate_hash
export_hash
axiom_report_hash
direct import module names
direct import export hashes
direct import certificate hashes
import_lock_hash or dependency_summary_hash
enabled core features
```

Modes:

```text
off
  Default for release, high-trust, public package handoff, and current package
  gates. The live checker runs for every selected certificate.

read-through
  Looks up the result store, but still runs the live checker. If the stored
  result differs from the live result, the live result wins and the stale entry
  is discarded or replaced.

local-hit
  Local-only acceleration for repeated package audit loops. An exact cache hit
  may skip live checker execution for that module, but the output must mark
  cache_status = "hit" and proof_evidence = false. This mode is forbidden in
  release/high-trust scripts.
```

### 4.2 Verified Export Summary

Create a compact source-free summary derived from `.npcert` bytes and package
lock identity. This is the NPA equivalent of Go export data.

Proposed schema:

```text
npa.package.verified_export_summary.v0.1
```

Minimum fields:

```text
schema
module
core_spec
certificate_format
export_hash
certificate_hash
axiom_report_hash
exports: definitions / theorems / inductives with stable global refs
direct imports: module, export_hash, certificate_hash
module axioms
core features
summary_hash
source_certificate_file_hash
```

The summary is an audit acceleration artifact, not proof evidence. It may be
used to decide which downstream modules are stale, prepare checker request
stores, or avoid re-decoding large certificates for metadata-only questions.
It must not be accepted as a substitute for certificate bytes.

### 4.3 Reverse Dependency Invalidation

Package audit selection should be driven by the package lock DAG and module
identity hashes:

- If a module's `certificate_hash` changes, verify that module.
- If a module's `export_hash` changes, verify its reverse dependency closure.
- If a module's certificate changes but `export_hash` stays the same, downstream
  modules do not need semantic re-audit for import compatibility. They may still
  need metadata artifact checks if axiom report or package generated files
  changed.
- If package policy, checker profile, core spec, certificate format, or checker
  build identity changes, invalidate the relevant cache namespace.

This mirrors package-level stale detection: downstream work follows public
interface identity (`export_hash`), not arbitrary source or metadata churn.

### 4.4 Cheap Preflight Before Checker Execution

Before launching a checker, run deterministic cheap checks:

1. Read package manifest and package lock.
2. Validate package graph, paths, and policy.
3. Check certificate file hashes and canonical stored hashes.
4. Check certificate-declared import identities against package lock entries.
5. Compute the selected audit set and cache keys.

If these checks fail, do not run the expensive checker process. The diagnostic
should identify the first stale hash, missing artifact, graph cycle, path escape,
or import mismatch.

### 4.5 Topological Layer Parallelism

Package verification already has a dependency-topological order requirement.
Within that order, independent modules in the same layer can be verified in
parallel.

Rules:

- The selected module set is first sorted into deterministic topological layers.
- Workers may execute modules in one layer concurrently after all prior layers
  have accepted.
- Result output is emitted in deterministic package order, not completion order.
- Parallel execution must not change checker input bytes, cache keys, diagnostics
  for a fixed failing module, or generated artifacts.
- A `--jobs N` option may default to 1 for initial implementation and later to a
  conservative local CPU count in non-release scripts.

### 4.6 Closure Audit Integration

Closure audit workflows should use the same package audit substrate:

- Generate or read the promotion plan.
- Build the candidate import closure and target package lock identity.
- Run source-free verification for only the candidate closure and impacted
  downstream smoke modules.
- Use `read-through` or `local-hit` only for local iteration.
- Finish with cache-off `package check`, `check-hashes`, `build-certs --check`,
  `verify-certs --checker reference --audit-cache off`,
  `axiom-report --check`, `index --check`, `publish-plan --check` when the
  package checks in a publish plan, and downstream smoke before declaring a
  promotion ready.

## 5. Implementation Plan

### PAS-00 Baseline Package Audit Profile

Status: Planned

Purpose:

PAS-00 is a measurement-only milestone. It establishes the package audit
bottleneck before any cache, selection, or parallelism code is added.

Files to add or edit:

- Add `develop/proof-corpus-package-audit-baseline-pas-00.md`.
- Do not edit Rust code, proof source, certificates, package manifests, package
  generated artifacts, or gate scripts.

Measurement commands:

```sh
/usr/bin/time -p ./scripts/check-corpus-package.sh
/usr/bin/time -p cargo test -p npa-proof-corpus --test manifest_package_audit
/usr/bin/time -p cargo test --workspace --exclude npa-proof-corpus proof_package
/usr/bin/time -p cargo test -p npa-api package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy
/usr/bin/time -p cargo test -p npa-api --lib package_verifier
/usr/bin/time -p cargo test -p npa-cli package_cli_examples_pass_on_proof_corpus
/usr/bin/time -p cargo test -p npa-cli package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts
/usr/bin/time -p cargo test -p npa-cli package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean
/usr/bin/time -p cargo test -p npa-cli package_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact
```

Graph inventory method:

- PAS-00 must not add Rust helpers just to collect counts.
- Record counts by inspecting the checked package lock with a temporary local
  command or script outside the repository.
- If PAS-01 has already landed when PAS-00 is rerun, the reusable
  `package_audit_graph_inventory` helper may be used instead.

Baseline document required fields:

- Date, timezone, repository path, commit, branch, and dirty/clean status.
- Machine model, CPU thread count, memory, OS version, Rust version, Cargo
  version, and whether Cargo target cache was warm.
- Full package gate timing.
- Per-step timing for each command in `check-corpus-package.sh`.
- Package module count, external import count, package-lock entry count, direct
  import edge count, local reverse edge count, and topological layer count, plus
  the exact temporary command or helper used to collect those counts.
- A statement that the timing record is not proof evidence.

Acceptance criteria:

- Baseline timings are reproducible enough to compare PAS-08 against them.
- The document identifies the top three bottleneck commands.
- The milestone leaves no changes under `proofs/`, `tools/proof-corpus/`,
  `scripts/`, or `crates/`.

Verification:

```sh
git diff --name-only -- proofs tools/proof-corpus scripts crates
git diff --check
```

### PAS-01 Package Audit Identity Model

Status: Planned

Purpose:

PAS-01 adds the stable identity and serialization layer used by later cache,
summary, selection, and measurement milestones. It must not call a checker and
must not read the filesystem.

Files to add or edit:

- Add `crates/npa-package/src/audit_cache.rs`.
- Export the module from `crates/npa-package/src/lib.rs`.
- Add tests in the same module, following existing `npa-package` artifact model
  tests.
- No CLI changes in this milestone.

New constants:

```rust
pub const PACKAGE_AUDIT_CACHE_SCHEMA: &str = "npa.package.audit_cache.v0.1";
pub const PACKAGE_AUDIT_RESULT_SCHEMA: &str = "npa.package.audit_result.v0.1";
pub const PACKAGE_VERIFIED_EXPORT_SUMMARY_SCHEMA: &str =
    "npa.package.verified_export_summary.v0.1";
pub const PACKAGE_AUDIT_CACHE_LAYOUT_DIR: &str =
    "target/npa-package-audit-cache/results-v0.1";
```

New data types:

```rust
pub struct PackageAuditCheckerIdentity {
    pub mode: String,
    pub checker_id: String,
    pub checker_version: String,
    pub checker_build_hash: PackageHash,
    pub checker_profile: String,
    pub runner_policy_hash: Option<PackageHash>,
}

pub struct PackageAuditImportIdentity {
    pub module: Name,
    pub export_hash: PackageHash,
    pub certificate_hash: PackageHash,
}

pub struct PackageAuditCacheKeyInput {
    pub schema: String,
    pub core_spec: String,
    pub certificate_format: String,
    pub package_lock_hash: PackageHash,
    pub package_policy_hash: PackageHash,
    pub checker: PackageAuditCheckerIdentity,
    pub module: Name,
    pub certificate_file_hash: PackageHash,
    pub certificate_hash: PackageHash,
    pub export_hash: PackageHash,
    pub axiom_report_hash: PackageHash,
    pub direct_imports: Vec<PackageAuditImportIdentity>,
    pub dependency_summary_hash: Option<PackageHash>,
    pub enabled_core_features: Vec<String>,
}

pub enum PackageAuditCachedStatus {
    Accepted,
    Rejected,
}

pub struct PackageAuditResultEntry {
    pub schema: String,
    pub cache_key: String,
    pub trusted: bool,
    pub key_input: PackageAuditCacheKeyInput,
    pub status: PackageAuditCachedStatus,
    pub diagnostic_reason: Option<String>,
    pub trust_boundary: String,
}

pub struct PackageAuditGraphInventory {
    pub local_module_count: u64,
    pub external_import_count: u64,
    pub lock_entry_count: u64,
    pub direct_import_edge_count: u64,
    pub local_reverse_edge_count: u64,
    pub topological_layer_count: u64,
}
```

New public functions:

```rust
pub fn package_audit_cache_key_material(input: &PackageAuditCacheKeyInput) -> String;
pub fn package_audit_cache_key(input: &PackageAuditCacheKeyInput) -> String;
pub fn package_audit_result_entry_json(entry: &PackageAuditResultEntry) -> String;
pub fn parse_package_audit_result_entry_json(
    source: &str,
) -> PackageArtifactResult<PackageAuditResultEntry>;
pub fn validate_package_audit_result_entry(
    entry: &PackageAuditResultEntry,
) -> PackageArtifactResult<()>;
pub fn package_audit_direct_imports_for_entry(
    entry: &PackageLockEntry,
) -> Vec<PackageAuditImportIdentity>;
pub fn package_audit_graph_inventory(
    lock: &PackageLockManifest,
) -> PackageArtifactResult<PackageAuditGraphInventory>;
```

Canonicalization rules:

- Sort and deduplicate `direct_imports` by `(module, export_hash,
  certificate_hash)`.
- Sort and deduplicate `enabled_core_features`.
- Serialize JSON fields in fixed order; do not use `HashMap` iteration order.
- Hash key material with the existing package SHA-256 formatter.
- Treat unknown schema, invalid hash, missing `trusted = false`, or malformed
  checker identity as invalid.

Tests:

- `package_audit_cache_key_is_deterministic`
- `package_audit_cache_key_changes_for_package_lock_hash`
- `package_audit_cache_key_changes_for_checker_build_hash`
- `package_audit_cache_key_changes_for_certificate_hash`
- `package_audit_cache_key_sorts_direct_imports`
- `package_audit_result_entry_requires_trusted_false`
- `package_audit_result_entry_round_trips_canonical_json`
- `package_audit_graph_inventory_counts_entries_edges_and_layers`

Acceptance criteria:

- The same input produces byte-identical key material and cache key.
- Changing each required identity input changes the cache key.
- Cache entries cannot be validated with `trusted = true`.
- The graph inventory helper uses `PackageLockGraph` and does not inspect
  source, replay, meta, theorem index, AI trace, network, or hidden caches.

Verification:

```sh
cargo test -p npa-package package_audit
git diff --check
```

### PAS-02 Read-Through Result Store

Status: Planned

Purpose:

PAS-02 wires the identity model into `npa package verify-certs` in a safe
read-through mode. Read-through mode may read and write cache files, but it
still runs the live checker for every selected certificate.

Files to add or edit:

- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/package_verify.rs`
- `crates/npa-cli/src/diagnostic.rs`, only if a new diagnostic kind or JSON
  field is needed.
- `crates/npa-package/src/audit_cache.rs`, only for helper additions discovered
  during CLI wiring.

CLI changes:

```text
npa package verify-certs \
  [--root PATH] \
  [--json] \
  [--checker reference|fast|external] \
  [--audit-cache off|read-through] \
  [--runner-policy PATH --runner-policy-hash HASH --checker-registry PATH]
```

Parser changes:

- Add `PackageAuditCacheMode { Off, ReadThrough }` in `args.rs`.
- Add `audit_cache: PackageAuditCacheMode` to `PackageVerifyCertsOptions`.
- Default to `Off`.
- Reject duplicate `--audit-cache`.
- Reject unsupported value with a deterministic usage diagnostic.
- Document the flag in `HelpTopic::PackageVerifyCerts`.

Result-store path:

```text
<command cwd>/target/npa-package-audit-cache/results-v0.1/<cache-key>.json
```

`<command cwd>` is `std::env::current_dir()` captured at command start. Do not
derive the cache location from `--root` unless `--root` is also the current
directory. The cache is local build output, not package metadata.

Implementation details:

1. Keep the existing `run_package_verify_certs_on_stack` preflight:
   load package root, read checked package lock, read certificates, regenerate
   package lock, and reject stale lock before checker execution.
2. Add `package_audit_cache_key_input_for_entry(...)` in `package_verify.rs`.
   It builds PAS-01 key input from the checked lock entry, graph imports,
   package lock hash, manifest policy hash, checker identity, and certificate
   artifact bytes.
3. For `Off`, call the current `verify_package(...)` path unchanged.
4. For `ReadThrough`, call a wrapper:

```rust
struct PackageAuditVerificationRun {
    report: PackageVerificationReport,
    cache: PackageAuditCacheSummary,
}

fn verify_package_with_read_through_cache(
    checker: PackageChecker,
    loaded: &LoadedPackageRoot,
    lock: &PackageLockManifest,
    artifacts: &[CertificateArtifactBuffer],
) -> Result<PackageAuditVerificationRun, PackageVerificationError>
```

5. The wrapper must:
   - compute cache lookup status before live checking;
   - run the live checker through the existing verifier path;
   - write one result entry per module after live result is known;
   - discard or overwrite stale entries when stored entry differs from live
     status or key material;
   - never change the live report status.

External checker scope:

- PAS-02 covers `--checker reference` and `--checker fast`.
- `--checker external --audit-cache read-through` must be rejected with a
  deterministic usage diagnostic unless the same milestone explicitly wires the
  result store through `run_package_verify_external`.
- External checker result caching is allowed only after runner policy hash,
  checker binary identity, process result hash, and resource/error classes are
  part of the cache key and stale-entry comparison.

New CLI-local data type:

```rust
struct PackageAuditCacheSummary {
    mode: PackageAuditCacheMode,
    hits: usize,
    misses: usize,
    stale: usize,
    schema_misses: usize,
    written: usize,
    live_checked: usize,
    cached: usize,
    trusted: bool,
}
```

Output changes:

- Text output may append cache summary diagnostics after live verification.
- JSON output must include a deterministic artifact or diagnostic field with:

```json
{
  "audit_cache": {
    "mode": "read-through",
    "hits": 0,
    "misses": 0,
    "stale": 0,
    "schema_misses": 0,
    "written": 0,
    "trusted": false
  }
}
```

If the current `CommandResult` model cannot carry nested objects without churn,
use deterministic flat diagnostic fields first:

```text
audit_cache_mode
audit_cache_hits
audit_cache_misses
audit_cache_stale
audit_cache_schema_misses
audit_cache_written
```

Tests:

- `package_verify_certs_parses_audit_cache_off`
- `package_verify_certs_parses_audit_cache_read_through`
- `package_verify_certs_rejects_duplicate_audit_cache`
- `package_verify_certs_rejects_unknown_audit_cache_mode`
- `package_verify_certs_read_through_runs_live_checker_on_hit`
- `package_verify_certs_read_through_repairs_stale_entry`
- `package_verify_certs_read_through_does_not_mask_live_failure`
- `package_verify_certs_audit_cache_output_is_deterministic`

Acceptance criteria:

- `--audit-cache off` is byte-compatible with the current pass/fail behavior.
- `read-through` cannot turn a failed live checker result into success.
- Removing `target/npa-package-audit-cache` changes only cache counters, not the
  verification verdict.
- Release and high-trust scripts do not pass `--audit-cache read-through`.
- External checker mode is either explicitly unsupported for read-through or has
  equivalent live-result-dominates-cache tests.

Verification:

```sh
cargo test -p npa-cli package_verify_certs_audit_cache
cargo test -p npa-api --lib package_verifier
cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache read-through --json
cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache off --json
git diff --check
```

### PAS-03 Local-Hit Mode For Explicit Local Audits

Status: Planned

Purpose:

PAS-03 adds an explicit local-only cache-hit mode for repeated audit loops. This
mode is intentionally not proof evidence and must be hard to invoke
accidentally.

Files to add or edit:

- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/package_verify.rs`
- `crates/npa-api/src/package_verifier.rs`
- `scripts/check-corpus-package.sh`, `scripts/check-corpus-full.sh`, and release
  scripts only to add tests or comments proving they do not use local-hit.

CLI changes:

```text
--audit-cache off|read-through|local-hit
```

Safety rules:

- `local-hit` is accepted only for `npa package verify-certs`.
- First implementation accepts `local-hit` only for `--checker reference` and
  `--checker fast`; external checker local-hit is rejected until external
  read-through identity is implemented and tested.
- `local-hit` is rejected if a future release/high-trust wrapper forwards it.
- Existing scripts must continue to omit `--audit-cache` so they use `off`.
- `local-hit` output must include:

```text
proof_evidence = false
follow_up = "rerun with --audit-cache off before promotion/release/high-trust"
```

Implementation details:

1. Extend `PackageAuditCacheMode` with `LocalHit`.
2. Reuse the PAS-02 key and lookup helpers.
3. For each module in topological order:
   - if cache entry is an exact accepted hit and all dependencies are accepted
     in the current run, emit a cached passed module result with
     `proof_evidence = false`;
   - otherwise run the live checker for that module and write/repair the cache;
   - if a live dependency fails, downstream modules remain skipped just like
     current verifier behavior.
4. Do not cache timeout/resource errors from external checker mode; if external
   mode is still unsupported, reject it before cache lookup.
5. If any module used a cache hit, the top-level report must mark the run as
   locally accelerated.

Report model changes:

- Add a small evidence marker to `PackageModuleVerificationResult` in
  `crates/npa-api/src/package_verifier.rs`, for example:

```rust
pub enum PackageModuleVerificationEvidence {
    LiveChecker,
    LocalAuditCache,
}
```

- Existing verifier paths set `LiveChecker`.
- Local-hit synthesized module results set `LocalAuditCache`.
- `passed_report_diagnostics` must expose this marker as a deterministic field
  so JSON/text output cannot confuse a cached result with live proof evidence.

Important implementation constraint:

The current reference verifier builds each import store from modules already
accepted by the same reference checker. A cached dependency must therefore carry
enough accepted import-store material to support downstream live checking, or
the implementation must conservatively live-check dependencies needed by later
live modules. The first implementation should choose the conservative route:

```text
local-hit may skip leaf modules, but any cached module required as an import by
a later live-checked module must be live-checked in that run.
```

This keeps PAS-03 implementable before serializing reference-checker internal
import stores.

Tests:

- `package_verify_certs_parses_audit_cache_local_hit`
- `package_verify_certs_local_hit_marks_proof_evidence_false`
- `package_verify_certs_local_hit_prints_cache_off_follow_up`
- `package_verify_certs_local_hit_does_not_run_from_package_gate_script`
- `package_verify_certs_local_hit_live_checks_cached_dependency_needed_by_live_dependent`
- `package_verify_certs_local_hit_does_not_mask_live_miss_failure`

Acceptance criteria:

- `local-hit` cannot be confused with live checker evidence in text or JSON.
- `check-corpus-package.sh` and `check-corpus-full.sh` still run cache-off.
- A local-hit run tells the operator exactly which cache-off command to run
  before promotion, release, or high-trust handoff.

Verification:

```sh
cargo test -p npa-cli package_verify_certs_local_hit
./scripts/check-corpus-package.sh
git diff --check
```

### PAS-04 Verified Export Summary Artifact

Status: Planned

Purpose:

PAS-04 adds the NPA equivalent of Go export data. It is a compact, deterministic
summary derived from package lock identity and `.npcert` bytes. It accelerates
audit planning and metadata questions, not proof acceptance.

Files to add or edit:

- `crates/npa-package/src/audit_cache.rs`, or a new
  `crates/npa-package/src/export_summary.rs` if the file becomes too large.
- `crates/npa-package/src/lib.rs`
- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/lib.rs`, if a new module is added.
- `crates/npa-cli/src/package.rs`
- `crates/npa-cli/src/package_verify.rs`, only if generation is implemented as a
  `verify-certs` submode.
- Prefer adding a new CLI command in `crates/npa-cli/src/package_artifacts.rs` or
  a new `crates/npa-cli/src/package_export_summary.rs` if the command writes
  files.

Recommended CLI:

```text
npa package export-summary [--root PATH] [--json] [--check] [--out PATH]
```

Command wiring:

- Add `PackageCommand::ExportSummary(PackageExportSummaryOptions)` in `args.rs`.
- Add `HelpTopic::PackageExportSummary`.
- Add `run_package_export_summary` and dispatch it from `package.rs`.
- Register the module from `npa-cli/src/lib.rs` if a new source file is used.
- Update top-level `npa package --help` command listing.

Default paths:

```text
write mode: generated/verified-export-summary.json
check mode: compare generated/verified-export-summary.json without mutating it
```

New data types:

```rust
pub struct PackageVerifiedExportSummary {
    pub schema: String,
    pub package: PackageId,
    pub version: String,
    pub package_lock_hash: PackageHash,
    pub modules: Vec<PackageVerifiedExportSummaryModule>,
    pub summary_hash: PackageHash,
    pub trusted: bool,
}

pub struct PackageVerifiedExportSummaryModule {
    pub module: Name,
    pub origin: PackageLockEntryOrigin,
    pub certificate: PackagePath,
    pub certificate_file_hash: PackageHash,
    pub export_hash: PackageHash,
    pub certificate_hash: PackageHash,
    pub axiom_report_hash: PackageHash,
    pub direct_imports: Vec<PackageAuditImportIdentity>,
    pub exported_globals: Vec<PackageGlobalRef>,
    pub module_axioms: Vec<PackageGlobalRef>,
    pub core_features: Vec<String>,
}
```

Extraction rules:

- Source input is only package manifest, package lock, and certificate bytes.
- The helper may decode certificates and may reuse
  `verify_package_fast_source_free_with_modules` when it needs verified module
  records.
- It must not read source, replay, meta, theorem index, AI trace, network, or
  hidden cache.
- Module order follows package-lock topological order or canonical module order;
  pick one and document it in the schema. Prefer canonical module order for
  stable diff review, with a separate topological order field if needed.

Validation rules:

- `trusted` must be false.
- `summary_hash` is computed over the summary with its own hash field omitted or
  zeroed, following existing package artifact hash patterns.
- Every module hash must match the package lock entry.
- Every direct import identity must match certificate-declared imports and lock
  graph imports.
- Summary schema mismatch is a validation error, not a best-effort parse.

Tests:

- `verified_export_summary_is_deterministic`
- `verified_export_summary_requires_trusted_false`
- `verified_export_summary_rejects_tampered_export_hash`
- `verified_export_summary_rejects_tampered_direct_import`
- `verified_export_summary_does_not_require_source_artifacts`
- `package_export_summary_check_mode_does_not_mutate_artifacts`

Acceptance criteria:

- Summary generation is deterministic and source-free.
- Summary validation rejects stale or tampered fields.
- The artifact text says it is not proof evidence.

Verification:

```sh
cargo test -p npa-package verified_export_summary
cargo test -p npa-cli package_export_summary
cargo run -p npa-cli -- package export-summary --root proofs --check --json
git diff --check
```

### PAS-05 Reverse Dependency Audit Selection

Status: Planned

Purpose:

PAS-05 selects the minimal package audit set from hash changes. It does not
change checker behavior yet; it computes and reports the module set that PAS-06
will pass into the verifier execution options.

Files to add or edit:

- `crates/npa-package/src/audit_selection.rs`
- `crates/npa-package/src/lib.rs`
- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/package_verify.rs`

New data types:

```rust
pub enum PackageAuditChangeKind {
    CertificateHashChanged,
    ExportHashChanged,
    AxiomReportHashChanged,
    CertificateFileHashChanged,
    PolicyChanged,
    CheckerIdentityChanged,
    CoreSpecChanged,
    CertificateFormatChanged,
}

pub struct PackageAuditChangedModule {
    pub module: Name,
    pub changes: Vec<PackageAuditChangeKind>,
}

pub enum PackageAuditSelectionReason {
    ExplicitlyChanged,
    ReverseDependencyOfExportChange { dependency: Name },
    RequiredByPolicyChange,
    RequiredByCheckerIdentityChange,
    RequiredByCoreSpecChange,
    RequiredByCertificateFormatChange,
}

pub struct PackageAuditSelectedModule {
    pub module: Name,
    pub reasons: Vec<PackageAuditSelectionReason>,
}

pub struct PackageAuditSelection {
    pub modules: Vec<PackageAuditSelectedModule>,
    pub skipped_stable_export_dependents: Vec<Name>,
}
```

New functions:

```rust
pub fn package_lock_reverse_dependencies(
    lock: &PackageLockManifest,
) -> PackageArtifactResult<BTreeMap<Name, Vec<Name>>>;

pub fn select_package_audit_modules(
    lock: &PackageLockManifest,
    changed: &[PackageAuditChangedModule],
) -> PackageArtifactResult<PackageAuditSelection>;
```

Recommended CLI for selection reporting:

```text
npa package verify-certs \
  --changed-module MODULE[:certificate|export|axiom|file]... \
  --audit-selection explicit|reverse-deps \
  --print-audit-selection \
  --audit-cache off|read-through|local-hit
```

PAS-05 may keep this as an internal API and add the CLI only after tests prove
selection. If the CLI is added in PAS-05, `--print-audit-selection` reports the
selected modules and exits before checker execution. Actual partial verification
of the selected set belongs to PAS-06.

Selection algorithm:

1. Build and validate the package lock graph.
2. Build reverse edges from every direct import.
3. Include every explicitly changed module.
4. For `ExportHashChanged`, include the full reverse dependency closure.
5. For `CertificateHashChanged` without `ExportHashChanged`, include only the
   changed module.
6. For `AxiomReportHashChanged`, include the changed module and mark package
   axiom-report/index checks as required.
7. For policy, checker identity, core spec, or certificate format changes,
   select all modules.
8. Sort output in deterministic package-lock topological order.

Tests:

- `package_audit_selection_leaf_certificate_change_is_local`
- `package_audit_selection_leaf_export_change_selects_reverse_dependents`
- `package_audit_selection_root_export_change_selects_all_dependents`
- `package_audit_selection_shared_dependency_deduplicates_reasons`
- `package_audit_selection_policy_change_selects_all`
- `package_audit_selection_output_uses_topological_order`
- `package_audit_selection_rejects_unknown_changed_module`

Acceptance criteria:

- Selection is deterministic and graph-based.
- Diagnostics explain why each selected module was included.
- Stable-export certificate changes do not trigger downstream semantic audit.
- Any PAS-05 CLI output is selection-only and does not imply that unselected
  modules were verified.

Verification:

```sh
cargo test -p npa-package package_audit_selection
cargo test -p npa-cli package_audit_selection # only if the PAS-05 CLI is added
git diff --check
```

### PAS-06 Deterministic Topological Parallel Verification

Status: Planned

Purpose:

PAS-06 adds deterministic execution planning for parallel package verification.
It starts with a serial-compatible layer plan, then enables concurrency where
the checker implementation can support it without changing semantics.

Files to add or edit:

- `crates/npa-package/src/audit_selection.rs` or `crates/npa-package/src/lock.rs`
  for topological layer construction.
- `crates/npa-api/src/package_verifier.rs`
- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/package_verify.rs`

New data types:

```rust
pub struct PackageTopologicalLayers {
    pub layers: Vec<Vec<Name>>,
}

pub struct PackageVerificationExecutionOptions {
    pub jobs: usize,
    pub selected_modules: Option<BTreeSet<Name>>,
}
```

New functions:

```rust
pub fn package_lock_topological_layers(
    lock: &PackageLockManifest,
) -> PackageArtifactResult<PackageTopologicalLayers>;

pub fn verify_package_fast_source_free_with_options<'a>(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
    options: PackageVerificationExecutionOptions,
) -> PackageVerificationResult<PackageVerificationReport>;

pub fn verify_package_reference_source_free_with_options<'a>(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
    options: PackageVerificationExecutionOptions,
) -> PackageVerificationResult<PackageVerificationReport>;
```

CLI changes:

```text
--jobs N
```

Parser rules:

- Default `--jobs` to `1`.
- Reject `0`.
- Reject duplicate `--jobs`.
- In the first implementation, allow `--jobs > 1` only for fast verifier if the
  reference checker import-store dependency makes parallel reference mode unsafe.
  Alternatively parse `--jobs > 1` but execute reference mode serially with a
  diagnostic `parallel_reference_deferred`. The behavior must be explicit.

Layering algorithm:

1. Use package lock graph dependencies.
2. Layer 0 contains modules with no imports inside the selected set.
3. Layer N contains modules whose selected dependencies are all in layers `< N`.
4. Within each layer, sort by canonical package-lock module order.
5. If any module in a layer fails, do not execute later layers that depend on it.

Parallel execution rules:

- Results are stored by module and emitted in deterministic topological order.
- Worker completion order is never reflected in JSON or text output.
- Cache writes use content-addressed temp files followed by atomic rename.
- Diagnostics for a fixed failing module must match `--jobs 1` after ignoring
  timing fields.

Tests:

- `package_lock_topological_layers_are_deterministic`
- `package_lock_topological_layers_group_independent_modules`
- `package_verify_certs_rejects_jobs_zero`
- `package_verify_certs_jobs_one_matches_existing_order`
- `package_fast_verifier_jobs_four_matches_jobs_one_normalized`
- `package_parallel_verifier_skips_dependents_after_failed_dependency`
- `package_reference_verifier_parallel_mode_is_serial_or_explicitly_rejected`

Acceptance criteria:

- `--jobs 1` preserves existing behavior.
- Any enabled `--jobs N` mode produces normalized output identical to `--jobs 1`
  for successful checks.
- Parallelism is not enabled for checker paths that cannot preserve import-store
  semantics.

Verification:

```sh
cargo test -p npa-package package_lock_topological_layers
cargo test -p npa-api --lib package_verifier_parallel
cargo test -p npa-cli package_verify_certs_jobs
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 1 --json
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 4 --json
git diff --check
```

### PAS-07 Closure Audit Workflow Integration

Status: Planned

Purpose:

PAS-07 connects the package audit machinery to the existing closure audit and
promotion workflow without making local acceleration part of promotion evidence.

Files to add or edit:

- `develop/proof-corpus-package-audit-speed-plan.md`
- `.agents/skills/closure-audit/SKILL.md`
- `.agents/skills/judge-promote-to-mathlib/SKILL.md`, only if it needs to point
  at new selection commands.
- `tools/proof-corpus/src/main.rs`, only if `--promote-plan` or
  `--promote-materialize` should print new audit commands.
- Existing closure audit templates under `develop/npa-mathlib-*-closure-audit.md`
  are examples, not all to be bulk-edited.

Workflow spec:

1. Promotion candidate discovery remains read-only.
2. `--promote-plan` records candidate module, target module, import mapping,
   axiom policy, closure modules, and downstream smoke requirements.
3. Local iteration may use:

```sh
cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache read-through --json
cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache local-hit --json
```

4. Closure audit notes must label those runs as local acceleration only.
5. Final promotion readiness must still record cache-off commands:

```sh
cargo run -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache off --json
cargo run -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
```

Closure audit document required fields:

- `local_audit_cache_mode`
- `selected_modules`
- `selection_reasons`
- `cache_hits`
- `live_checker_count`
- `skipped_by_stable_export`
- `final_cache_off_verification`
- `trust_boundary_note`

Implementation tasks:

- Update the closure-audit skill to distinguish local acceleration commands from
  final evidence commands.
- Update promote-plan output to include package audit selection commands if the
  PAS-05 CLI exists; otherwise include a pending-integration note naming the
  internal API.
- Add one fixture test for generated plan text if `tools/proof-corpus` output is
  changed.

Tests:

- `promote_plan_mentions_cache_off_final_gate`
- `promote_plan_marks_local_hit_not_proof_evidence`
- `closure_audit_skill_mentions_package_audit_selection`

Acceptance criteria:

- Closure audit guidance cannot end with only `local-hit` or read-through
  evidence.
- The final checklist always includes cache-off reference verification.
- Publish-plan and downstream smoke requirements remain visible where the
  closure audit target crosses a public `npa-mathlib` handoff boundary.
- Existing promotion materialize behavior remains unchanged unless explicitly
  updated in this milestone.

Verification:

```sh
git diff --check
cargo run -p npa-proof-corpus -- --promote-plan Proofs.Ai.Basic --mathlib-root ../npa-mathlib --to-module Mathlib.Logic.Basic --out /tmp/npa-pas07-plan.md
rg -n "audit-cache|cache-off|proof evidence|source-free" /tmp/npa-pas07-plan.md .agents/skills/closure-audit/SKILL.md
```

### PAS-08 Final Measurement And Gate Policy Update

Status: Planned

Purpose:

PAS-08 proves the package audit speed work improved local iteration without
weakening the final package gate policy.

Files to add or edit:

- Add `develop/proof-corpus-package-audit-pas-08-measurement.md`.
- Update `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, and
  `develop/internal-readme-notes-ja.md` only if default operator guidance
  changes.
- Do not edit proof source, certificates, package manifests, or generated
  package artifacts as part of measurement.

Measurement commands:

```sh
/usr/bin/time -p ./scripts/check-corpus-package.sh
/usr/bin/time -p cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache off --json
/usr/bin/time -p cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache read-through --json
/usr/bin/time -p cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache local-hit --json
/usr/bin/time -p cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 1 --json
/usr/bin/time -p cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 4 --json
```

Representative closure audit loop:

- Pick one small existing closure audit target that has no unrelated dirty
  package changes.
- Record local acceleration commands separately from final evidence commands.
- If `../npa-mathlib` is unavailable or dirty in an unrelated way, document the
  blocker and skip the closure-loop timing rather than fabricating data.

Measurement document required fields:

- PAS-00 baseline reference.
- Exact commands, pass/fail status, real/user/sys time, and cache mode.
- Cache hit/miss/stale counts.
- Live checker count and skipped checker count.
- `--jobs 1` vs `--jobs N` normalized output comparison result.
- Final cache-off verification result.
- Remaining top three bottlenecks.
- Explicit statement that cache and timing logs are not proof evidence.

Gate policy update rules:

- If cache-off package gate remains required, do not relax README /
  CONTRIBUTING / AGENTS.
- If local-hit is useful, document it only as an optional local iteration tool.
- Never add `local-hit` to `check-corpus-package.sh`, `check-corpus-full.sh`,
  release scripts, or high-trust scripts.

Acceptance criteria:

- Final measurement includes at least one cache-off passing package verification.
- Any local-hit speedup is reported separately from proof evidence.
- Documentation preserves the rule that promotion, release, checker,
  certificate, and high-trust boundaries use explicit package/full gates.

Verification:

```sh
./scripts/check-fast.sh
./scripts/check-corpus-authoring.sh
./scripts/check-corpus-package.sh
git diff --name-only -- proofs tools/proof-corpus scripts crates
git diff --check
```

## 6. Rollout Policy

Implement PAS milestones in order. Do not introduce `local-hit` before
read-through mode exists and has tests proving live checker results dominate
stored entries. Do not enable parallel package verification by default until
`--jobs 1` and `--jobs N` normalized outputs are proven identical.

The package gate remains the authoritative local gate for package verifier,
package metadata, certificate/checker compatibility, promotion readiness,
release handoff, and high-trust-adjacent changes. This plan only makes repeated
local audit loops cheaper; it does not make cache files part of the proof.
