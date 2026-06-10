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

対象:

- `npa package verify-certs` and package verifier fixture speedups.
- `npa-mathlib` promotion readiness and closure audit local loops.
- Source-free package graph traversal based on package lock entries and
  certificate-declared imports.
- Deterministic local cache and summary artifacts that shorten repeated local
  audits.
- Measurement of package gate and closure audit bottlenecks.

非対象:

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
信頼しない:
  cache file
  verified export summary
  package lock
  package audit plan
  promotion plan
  theorem index
  timing log
  AI / tactic / replay / metadata

信頼する:
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

### 4.7 Build-Certs Check Reuse

The next largest remaining bottleneck is `npa package build-certs --check`,
especially when it is invoked through package CLI examples. This command
rebuilds checked certificate artifacts from source and compares them with the
checked-in source-free artifacts. It is not a checker proof-evidence command,
but it protects the package boundary from stale generated artifacts.

Add a local, content-addressed build-certs check result store keyed by the
inputs that can change the generated artifacts:

```text
schema
tool_version
tool_build_hash
core_spec
certificate_format
module
source_sha256
direct import module names
direct import export hashes
direct import certificate hashes
source compiler options
package metadata mode
expected certificate_file_sha256
expected export_hash
expected axiom_report_hash
expected certificate_hash
```

PAS-09 starts with read-through mode: it may read and write cache entries, but
it still performs the live build comparison. A future explicit local-hit mode
may skip rebuilding a module only after read-through tests prove the key is
stable and the output marks `build_evidence = false`. Release, high-trust, and
public handoff gates keep build-check caching disabled by default.

### 4.8 Shared Package Snapshot Projection

`axiom-report --check`, `index --check`, `export-summary --check`, and
`publish-plan --check` repeatedly load the same package root, lock, certificate
artifacts, verified export summaries, and policy inputs. A package gate run
should be able to construct a single deterministic source-free package snapshot
and feed that snapshot into all projection checks.

The snapshot is an in-memory or temporary-file acceleration artifact. It is not
proof evidence and must be reproducible from checked-in package artifacts:

```text
package root identity
manifest hash
package lock hash
policy hash
certificate artifact buffers
decoded source-free module records
verified export summary records
topological order
reverse dependency graph
projection input hashes
```

Standalone CLI commands remain backwards-compatible. Gate scripts and tests may
opt into shared snapshot mode to avoid repeated decode, graph construction, and
projection input scanning.

### 4.9 Package CLI Example Tiering

The previous `package_cli_examples_pass_on_proof_corpus` test exercised multiple
full proof-corpus commands through one long test. That coverage is useful as a
release smoke test but too expensive as an always-on package gate component.

Split CLI example coverage into tiers:

```text
smoke examples
  Small fixtures and representative proof-corpus metadata-only checks. These
  cover argument parsing, help text, check-mode behavior, and JSON shape.

full corpus examples
  Full proof-corpus `build-certs --check`, `verify-certs`, projection checks,
  and publish-plan examples. These stay in release, high-trust, and explicit
  full package gates.
```

This split does not remove final verification obligations. It makes the normal
package development gate pay for one representative CLI path and leaves the full
corpus CLI example suite available for explicit high-confidence runs. The
package gate runs `package_cli_smoke` plus exact projection/publish check-mode
tests, while `check-corpus-full.sh` keeps the full proof-corpus build/verify
example tier runnable by exact test name.

### 4.10 Dependency-Level Verification Memo

Several tests verify the same package lock entries with the same checker mode
inside one process or one local package-gate run. Add a verifier memo keyed by:

```text
checker_mode
checker_identity
package_lock_hash
module
certificate_file_hash
certificate_hash
export_hash
direct import identities
policy hash
enabled core features
```

The memo may be process-local first. A disk-backed form can reuse the PAS-01
audit cache layout only if its entries keep `trusted = false` and never replace
release proof evidence. Fast and reference checker modes must use disjoint
namespaces.

### 4.11 Impact-Aware Gate Planner

Package work should choose the cheapest sufficient gate from the changed files
and package identity impact. Add a deterministic planner that classifies a
worktree diff into gate requirements:

```text
docs-only
  `git diff --check` and relevant documentation review.

proof authoring only
  authoring gate and changed-module source-free verification.

package metadata/projection
  package artifact checks and selected projection regeneration checks.

checker/certificate semantics
  cache-off package gate and, when needed, full gate.

kernel/core semantics
  fast gate, package gate, full gate, and high-trust notes.
```

The planner recommends commands; it does not mark a proof accepted. It should
explain why a gate was selected and when the operator must escalate to
cache-off full verification.

### 4.12 Audit Timing Telemetry

Further speed work needs stable timing data for each expensive phase. Add
optional JSON timing telemetry to package subcommands and package gate helpers:

```text
load_root_ms
load_lock_ms
decode_certificates_ms
build_graph_ms
selection_ms
cache_lookup_ms
checker_ms
projection_ms
json_write_ms
artifact_compare_ms
total_ms
```

Timing logs are not proof evidence. They must be deterministic in field names
and units, but wall-clock values are informational. The default text output can
remain quiet; JSON or `--timings` output is enough.

### 4.13 Further Speedups After Timing And Gate Planning

After PAS-14, use measured phase data and PAS-13 impact classes to continue
reducing repeated local package audit work. These follow-up optimizations keep
the same trust boundary: caches, plans, snapshots, decoded structures, shard
outputs, and incremental projections are untrusted acceleration artifacts and
never replace canonical certificate bytes or source-free checker verdicts.

PAS-15 through PAS-20 have now implemented disk-backed verifier memoization,
report-only gate-plan integration, a process-local shared snapshot command
group, process-local decode/import context reuse, deterministic outer verifier
shards, and conservative incremental generated-artifact checks. The remaining
candidate work should target the measured `checker_ms` bottleneck rather than
JSON projection cost:

```text
package gate shared snapshot default
  Make package gate scripts use the in-process shared snapshot command group
  where safe, so axiom report, theorem index, export summary, publish plan, and
  fast verification do not each rebuild the same source-free package snapshot.

persistent per-module verified result cache
  Store verified module records by certificate/import/checker identity across
  processes. Cache hits are acceleration records only; release and high-trust
  gates must keep a cache-off verification path.

reference checker summary cache
  Cache source-free reference checker summaries separately from fast verifier
  results because publish-plan checks pay this cost repeatedly.

persistent import context export data cache
  Persist Go-style export data for dependency contexts so unchanged import
  closures do not need repeated context materialization.

cache-aware DAG verifier
  Extend impacted-module planning into the verifier so dirty modules and reverse
  dependents run live while unchanged modules can be read from trusted-false
  verifier records.

unified generated package check command
  Add one CLI command for generated package artifact checks so local gates can
  share one snapshot and one verifier run while keeping standalone commands for
  compatibility.
```

PAS-14 telemetry should continue to guide priority. After PAS-20, the preferred
next step is to make local package gates consume the shared snapshot path by
default, because current proof-corpus checks show `checker_ms` dominates
`projection_ms`. Persistent caches and cache-aware DAG verification should
remain opt-in until tests prove cache deletion leaves verification verdicts
unchanged and release handoff still records cache-off source-free verification.

### 4.14 Further Speedups After PAS-20 Checker-Dominated Timing

PAS-20 added generated-artifact invalidation planning and avoids unnecessary
JSON rewrite work for unchanged checked artifacts, but the post-PAS-20 timing
profile still shows package checks dominated by source-free checker work. The
next optimization wave should therefore move repeated verifier/checker work out
of the hot path while preserving the same certificate-first trust boundary.

Implementation rules for PAS-21 and later:

- Cache keys must include package-lock schema, package identity, checker
  profile, core spec, certificate format, certificate file hash, canonical
  certificate hash, export hash, axiom report hash, direct import identities,
  and dependency package identities where applicable.
- Cache values may contain decoded modules, import contexts, verified module
  records, checker summaries, and timing metadata, but every value is
  `proof_evidence=false`.
- A cache hit must be observationally equivalent to deleting the cache and
  rerunning the live source-free checker, except for timing/cache counters.
- Global schema, checker binary/profile, policy, core semantics, certificate
  canonical encoding, or dependency identity changes must force a live full
  verification path.
- Release, high-trust, and public handoff gates must keep an explicit cache-off
  source-free verification command.

Candidate milestones:

```text
PAS-21 Package Gate Shared Snapshot Default
  Route local package gate scripts through the shared snapshot check group when
  the diff impact class allows it. Keep standalone command output unchanged.

PAS-22 Persistent Per-Module Verified Result Cache
  Persist verified module records by exact verifier identity for later
  cache-aware reuse. Start with read-through mode and add explicit local-hit
  only after live-result dominance tests pass.

PAS-23 Reference Checker Summary Cache
  Add a separate cache for reference checker summaries used by publish-plan and
  release metadata checks. Fast and reference cache namespaces must not collide.

PAS-24 Persistent Import Context Export Data Cache
  Persist dependency import contexts/export data by import-closure hash. Cache
  invalidation follows package-lock dependency identity, not source paths.

PAS-25 Cache-Aware DAG Verifier
  Combine PAS-20 impacted-module planning with verifier records so changed
  modules and reverse dependents run live while unchanged modules can be reused
  with explicit non-evidence diagnostics.

PAS-26 Unified Generated Package Check Command
  Add a single source-free command for axiom report, theorem index, export
  summary, publish plan, and fast verifier checks. Gate scripts use it to avoid
  repeated CLI startup, root loading, certificate decode, and checker runs.
```

## 5. Implementation Plan

### PAS-00 Baseline Package Audit Profile

Status: Completed

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

Status: Completed

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

Status: Completed

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

Status: Completed

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

Status: Completed

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

Status: Completed

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

Status: Completed

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
    pub memoization: PackageVerificationMemoMode,
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

Status: Completed

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

Status: Completed

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

### PAS-09 Build-Certs Check Reuse

Status: Completed

Purpose:

PAS-09 reduces repeated `package build-certs --check` work in local package
audit loops. It adds a local check result cache for source-to-certificate
rebuild equivalence. The cache is not proof evidence and must not be used by
release or high-trust scripts by default.

Files to add or edit:

- `tools/proof-corpus/src/main.rs`
- `crates/npa-package/src/audit_cache.rs` or a new
  `crates/npa-package/src/build_check_cache.rs`
- `crates/npa-package/src/lib.rs`
- `crates/npa-cli/tests/package_build_certs_check.rs`
- `scripts/check-corpus-package.sh`, only if adding an explicit cache-off guard
  or a new local-only script path.

CLI shape:

```text
npa package build-certs --root proofs --check \
  --build-check-cache off|read-through \
  --json
```

Initial implementation may support only `off|read-through`. `local-hit` should
be added only after read-through proves that cached status never dominates a
fresh build comparison.

Acceptance criteria:

- `off` preserves existing behavior.
- `read-through` still performs the live build comparison and only records or
  repairs cache entries.
- Deleting the cache changes only counters and timings.
- If a later follow-up adds explicit local-hit build-check skipping, every
  skipped result must be marked `proof_evidence=false` and
  `build_evidence=false`.

Verification:

```sh
cargo test -p npa-package build_check_cache
cargo test -p npa-cli package_build_certs_check
cargo run -p npa-cli -- package build-certs --root proofs --check --build-check-cache off --json
git diff --check
```

### PAS-10 Shared Package Snapshot Projection

Status: Completed

Purpose:

PAS-10 avoids repeated package root loading, certificate decoding, graph
construction, and projection input scanning across axiom-report, theorem-index,
export-summary, and publish-plan checks.

Files to add or edit:

- `crates/npa-package/src/export_summary.rs`
- `crates/npa-api/src/package_artifacts.rs`
- `crates/npa-cli/src/package_axiom_report.rs`
- `crates/npa-cli/src/package_export_summary.rs`
- `crates/npa-cli/src/package_index.rs`
- `crates/npa-cli/src/package.rs`
- `crates/npa-cli/tests/package_axiom_report.rs`
- `crates/npa-cli/tests/package_export_summary.rs`
- `crates/npa-cli/tests/package_index.rs`
- `crates/npa-package/tests/publish_plan.rs`

Implementation rules:

- Add one internal `PackageAuditSnapshot` built from checked-in package
  manifest, package lock, certificate bytes, and policy.
- Reuse the snapshot for projection checks when multiple checks are run in one
  process.
- Keep standalone CLI command behavior and output unchanged unless
  `--timings` or a future combined command is explicitly requested.
- Reject stale package lock or stale certificate file hashes before snapshot
  reuse.

Acceptance criteria:

- Each projection check still passes when run standalone.
- A combined in-process test reuses one snapshot and produces byte-identical
  artifacts compared with standalone generation.
- Snapshot data is source-free and is never serialized as proof evidence.

Verification:

```sh
cargo test -p npa-cli package_projection_snapshot
cargo run -p npa-cli -- package axiom-report --root proofs --check --json
cargo run -p npa-cli -- package index --root proofs --check --json
cargo run -p npa-cli -- package export-summary --root proofs --check --json
cargo run -p npa-cli -- package publish-plan --root proofs --check --json
git diff --check
```

### PAS-11 Package CLI Example Tiering

Status: Completed

Purpose:

PAS-11 splits monolithic proof-corpus CLI example coverage into smoke and full
corpus tiers. The normal package development gate can run deterministic smoke
examples, while explicit release/high-trust/full gates keep full corpus
examples available.

Files to add or edit:

- `crates/npa-cli/tests/package_cli.rs`
- `scripts/check-corpus-package.sh`
- `scripts/check-corpus-full.sh`
- Any release/high-trust helper script that names package CLI examples.

Implementation rules:

- Rename or split the existing full proof-corpus example test so its purpose is
  explicit.
- Add smoke examples that use small fixtures or metadata-only proof-corpus
  checks to cover help text, JSON shape, argument parsing, and check-mode
  behavior.
- Keep full proof-corpus examples runnable by exact test name.
- Do not remove cache-off `verify-certs`, projection, or publish-plan checks
  from the package gate until their coverage is present elsewhere in the gate.

Acceptance criteria:

- Test names make smoke vs full corpus cost visible.
- `check-corpus-package.sh` documents which tier it runs and why.
- `check-corpus-full.sh` or release/high-trust instructions keep the full corpus
  example tier.

Verification:

```sh
cargo test -p npa-cli package_cli_smoke
cargo test -p npa-cli package_cli_full_corpus
./scripts/check-corpus-package.sh
git diff --check
```

### PAS-12 Dependency-Level Verification Memo

Status: Completed

Purpose:

PAS-12 memoizes repeated source-free module verification results inside one
package gate run and, later, across local runs. It targets repeated verifier
work across package tests that use the same package lock and checker mode.

Files to add or edit:

- `crates/npa-api/src/package_verifier.rs`
- `crates/npa-package/src/audit_cache.rs`
- `crates/npa-cli/src/package_verify.rs`
- `crates/npa-cli/tests/package_verify_certs.rs`

Implementation rules:

- Start with a process-local memo passed through verifier execution options.
- Key entries by checker mode, checker identity, package lock hash, module
  identity, direct import identity, policy hash, and enabled core features.
- Keep reference and fast checker memo namespaces separate.
- Do not memoize external checker timeout/resource errors.
- Emit deterministic memo counters only when JSON diagnostics or timings are
  requested.

Implemented notes:

- Process-local memoization is available through verifier execution options and
  is schema-separated from disk-backed audit cache keys.
- CLI `package verify-certs` enables the process-local memo for normal
  source-free verification, while memo counters are emitted only as explicit
  timing diagnostics.
- Memo entries are not proof evidence and are not persisted.

Acceptance criteria:

- Memo hits cannot change pass/fail status compared with memo disabled.
- Normalized output is identical with memo disabled vs enabled, ignoring timing
  and memo counters.
- A dependency failure still skips downstream modules deterministically.

Verification:

```sh
cargo test -p npa-api --lib package_verifier_memo
cargo test -p npa-cli package_verify_certs_memo
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

### PAS-13 Impact-Aware Gate Planner

Status: Completed

Purpose:

PAS-13 gives operators a deterministic command recommendation from the current
diff. It reduces unnecessary full package gate runs without weakening the cases
that require them.

Files to add or edit:

- Add `crates/npa-package/src/gate_plan.rs`.
- Export the module from `crates/npa-package/src/lib.rs`.
- Add a CLI command in `crates/npa-cli/src/package.rs` and `args.rs`, for
  example `npa package gate-plan`.
- Update `AGENTS.md` or `develop/internal-readme-notes-ja.md` only if the
  command becomes part of standard operator guidance.

Recommended CLI:

```text
npa package gate-plan \
  --base origin/main \
  --root proofs \
  --json
```

Planner output:

```text
changed_files
changed_modules
impact_class
required_commands
optional_local_acceleration_commands
escalation_reasons
trust_boundary_note
```

Acceptance criteria:

- Docs-only changes do not recommend package/full gates.
- Package metadata, generated artifacts, checker, certificate, kernel, or core
  semantics changes escalate to the documented gates.
- The planner never claims proof acceptance; it only recommends commands.

Implemented notes:

- `npa-package` exposes a path-list planner API for tests and other
  orchestration callers.
- `npa package gate-plan --base REF --root proofs --json` reads changed paths
  from `git diff --name-only REF...HEAD`, prints deterministic plan
  diagnostics, and does not run any gate command.
- The output includes a trust-boundary note that the planner is untrusted
  guidance and not proof evidence.

Verification:

```sh
cargo test -p npa-package package_gate_plan
cargo test -p npa-cli package_gate_plan
cargo run -p npa-cli -- package gate-plan --base origin/main --root proofs --json
git diff --check
```

### PAS-14 Audit Timing Telemetry

Status: Completed

Purpose:

PAS-14 adds optional timing telemetry to package subcommands and gate helpers so
future speed work can target measured phases instead of whole-command totals.

Files to add or edit:

- `crates/npa-cli/src/args.rs`
- `crates/npa-cli/src/package.rs`
- `crates/npa-cli/src/package_verify.rs`
- package projection command modules
- `crates/npa-cli/tests/package_cli_args.rs`
- relevant package command tests for JSON output shape

CLI shape:

```text
--timings off|summary|detailed
```

Output rules:

- Default is `off`.
- JSON timings use milliseconds and stable field names.
- Text output may omit timings unless explicitly requested.
- Timing fields are informational and never proof evidence.
- Tests compare normalized output with timing fields removed.

Acceptance criteria:

- Timing output exists for root load, lock load, certificate decode, graph
  construction, selection, cache lookup, checker, projection, artifact compare,
  JSON write, and total time where applicable.
- Commands without a phase omit or zero that phase consistently.
- Existing JSON consumers do not see timing fields unless requested.

Verification:

```sh
cargo test -p npa-cli package_timings
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --timings summary --json
cargo run -p npa-cli -- package axiom-report --root proofs --check --timings summary --json
git diff --check
```

### PAS-15 Disk-Backed Verifier Memo

Status: Completed

Purpose:

PAS-15 extends the PAS-12 process-local verifier memo to repeated package gate
processes. It keeps the memo schema-separated from proof artifacts and from
release evidence.

Files to add or edit:

- `crates/npa-api/src/package_verifier.rs`
- `crates/npa-package/src/audit_cache.rs` or a new local memo module
- `crates/npa-cli/src/package_verify.rs`
- `crates/npa-cli/tests/package_verify_certs.rs`
- package audit cache/memo tests

Implementation rules:

- Reuse the PAS-12 exact key material and add a disk schema version.
- Store only normalized verifier outcomes that are safe to reuse locally.
- Mark every disk memo hit as `trusted=false` and `proof_evidence=false`.
- Never store timeout, signal, resource, environment, or external checker
  runner failures.
- Keep fast/reference/checker identity namespaces disjoint.
- Provide a clear delete-and-rerun invariant: removing memo files changes only
  counters and elapsed time, never pass/fail status.

Acceptance criteria:

- Memo disabled and memo enabled produce identical normalized output except
  counters/timings.
- A stale certificate, lock, import, policy, checker identity, or core feature
  causes a miss.
- Cache-off package/full/release gates remain available and documented.

Implemented notes:

- Added schema-separated disk verifier memo key/result schemas under
  `target/npa-package-audit-cache/verifier-memo-v0.1`.
- Added `--verifier-memo off|disk`; disk mode is rejected for external checker
  runs and when combined with `--audit-cache`.
- Disk memo entries reuse PAS-12 verifier memo key material, but disk hits are
  reported as `evidence=disk-verifier-memo` with `proof_evidence=false`.
- Disk memo counters are emitted only through explicit timing diagnostics.

Verification:

```sh
cargo test -p npa-api package_verifier_disk_memo
cargo test -p npa-cli package_verify_certs_disk_memo
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

### PAS-16 Gate-Plan Driven Test Selection

Status: Completed

Purpose:

PAS-16 lets gate scripts consume PAS-13 classifications to recommend or select
the cheapest sufficient command set for local development while preserving
full-gate behavior at promotion, release, and high-trust boundaries.

Files to add or edit:

- `scripts/check-fast.sh`
- `scripts/check-corpus-authoring.sh`
- `scripts/check-corpus-package.sh`
- `scripts/check-corpus-full.sh`
- `scripts/package-gate-plan-report.sh`
- `crates/npa-cli/src/package_gate_plan.rs`
- `crates/npa-cli/tests/package_gate_plan.rs`
- docs describing operator policy

Implementation rules:

- Start with report-only mode in scripts; do not skip commands by default.
- If command selection is added, require explicit opt-in such as an environment
  variable or CLI flag.
- Release/high-trust-adjacent classes always include full gate guidance.
- Unknown non-doc paths must conservatively escalate to package tooling impact.
- Script output must list the gate-plan input base and changed path count.

Acceptance criteria:

- Docs-only changes do not recommend package/full gates.
- Kernel, certificate, checker, verifier, package lock, and generated package
  artifact changes still recommend package/full gates according to policy.
- Script selection never states that a proof is accepted.

Operator policy:

- Gate scripts run `npa package gate-plan` in report-only mode by default before
  their existing commands. This prints the current gate, input base, changed path
  count, impact class, and selected command list, but does not skip work.
- `NPA_PACKAGE_GATE_PLAN_BASE=<ref>` overrides the default `origin/main` base for
  script reports.
- `NPA_PACKAGE_GATE_PLAN=off` disables script reports.
- `NPA_PACKAGE_GATE_PLAN_SELECT=1` enables conservative per-script opt-in
  selection: a script exits early only when the PAS-13 selected command list does
  not contain that script. Selection remains local orchestration guidance and is
  not proof evidence.
- Promotion, release, and high-trust-adjacent work must continue to run the
  required package/full/release gates listed by the plan.

Implemented notes:

- Added `gate_plan_base`, `gate_plan_changed_path_count`, and
  `gate_plan_selected_commands` diagnostics to `package gate-plan`.
- Added shared script report/selection helper and integrated it into fast,
  authoring, package, and full gate scripts.
- Kept script behavior report-only by default; command skipping requires
  explicit `NPA_PACKAGE_GATE_PLAN_SELECT=1`.

Verification:

```sh
cargo test -p npa-cli package_gate_plan
./scripts/check-fast.sh
cargo run -p npa-cli -- package gate-plan --base origin/main --root proofs --json
git diff --check
```

### PAS-17 Shared Package Snapshot Command Group

Status: Completed

Purpose:

PAS-17 removes repeated package root, lock, graph, artifact byte, certificate
decode, and selection work across package projection and verifier commands that
run in the same local gate process.

Files to add or edit:

- `crates/npa-cli/src/package_artifacts.rs`
- `crates/npa-cli/src/package_axiom_report.rs`
- `crates/npa-cli/src/package_index.rs`
- `crates/npa-cli/src/package_export_summary.rs`
- `crates/npa-cli/src/package_publish.rs`
- `crates/npa-cli/src/package_verify.rs`
- tests covering snapshot reuse and normalized output

Implementation rules:

- Treat snapshots as in-memory acceleration only.
- Build the snapshot from checked-in source-free artifacts and package lock data.
- Do not put source text, replay traces, AI traces, or hidden cache content into
  checker input.
- Reuse snapshot data only when command options and package root identity match.
- Preserve existing command JSON shape unless explicit timing/diagnostic output
  is requested.

Implementation notes:

- Added an in-process shared snapshot check group in
  `crates/npa-cli/src/package_artifacts.rs`. The group loads package root,
  manifest, lock, artifact bytes, decoded certificates, graph, checked generated
  artifacts, and fast verification report once for five package command results.
- Added snapshot-backed check helpers for axiom report, theorem index, verified
  export summary, publish plan, and fast certificate verification. These helpers
  reuse only the already-loaded source-free snapshot and preserve standalone
  command JSON for the projection/check-mode commands.
- Added group diagnostics that report `commands=5`, `snapshot_builds=1`,
  `shared_load_root=1`, `shared_decode=1`, and the explicit boundary
  `source_text=false;replay=false;ai_trace=false;hidden_cache=false;network=false`.
- Added tests that compare snapshot and standalone projection/check JSON, delete
  the local package audit cache before rerunning the group, and assert timing
  telemetry exposes the reduced repeated load/decode phases.

Acceptance criteria:

- Projection/check-mode commands produce identical normalized output with and
  without shared snapshot reuse.
- Deleting local caches does not affect snapshot command results.
- Timing telemetry shows reduced repeated load/decode phases.

Verification:

```sh
cargo test -p npa-cli package_shared_snapshot
cargo test -p npa-cli package_timings
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --timings summary --json
git diff --check
```

### PAS-18 Certificate Decode And Import Context Cache

Status: Completed

Purpose:

PAS-18 caches decoded certificate structures and import context materialization
by content identity so repeated source-free verifier runs avoid decoding the
same certificate bytes and rebuilding identical import contexts.

Files to add or edit:

- `crates/npa-api/src/package_verifier.rs`
- certificate decode/import helper modules
- `crates/npa-api` verifier tests
- `crates/npa-cli/tests/package_verify_certs.rs`

Implementation rules:

- Key decoded entries by certificate file hash, certificate hash, certificate
  format, core spec, checker mode, and enabled core features.
- Key import contexts by ordered direct import identities and checker policy.
- Cache decoded structures only; never cache proof acceptance without the
  verifier result key from PAS-12/PAS-15.
- Preserve deterministic diagnostic ordering on cache hits and misses.

Implementation notes:

- Added a process-local decode/import cache in
  `crates/npa-api/src/package_verifier.rs`. Fast decoded certificates are keyed
  by certificate file hash, certificate hash, certificate format, core spec,
  checker mode, and enabled core features. Reference import contexts are keyed
  by ordered direct import identities plus checker policy hash.
- Added `npa_cert::verify_decoded_module_cert`, which verifies an already
  decoded certificate only after comparing its canonical encoding against the
  current certificate bytes. Cache hits therefore skip decode work but still run
  live verification and cannot become proof evidence.
- Added optional decode/import cache counters to package verification reports.
  `npa package verify-certs --timings ...` emits a `decode_cache_summary`
  diagnostic; timings off preserves the existing command JSON shape.
- Added API and CLI tests for second-run decode hits, corrupt certificate miss
  behavior, import identity invalidation, and policy failure despite cache hits.

Acceptance criteria:

- Corrupt certificate bytes miss or fail exactly as without the cache.
- Import identity changes miss.
- Cache hits cannot turn a verifier failure into success.

Verification:

```sh
cargo test -p npa-api package_verifier_decode_cache
cargo test -p npa-cli package_verify_certs_decode_cache
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

### PAS-19 Package Verifier Shard Runner

Status: Completed

Purpose:

PAS-19 adds deterministic outer sharding for package verification so independent
topological regions can run concurrently while final output remains in
package/topological order.

Files to add or edit:

- `crates/npa-api/src/package_verifier.rs`
- `crates/npa-package/src/audit_selection.rs` or a shard planning module
- `crates/npa-cli/src/package_verify.rs`
- tests for shard planning, failure propagation, and output order

Implementation rules:

- Build shards from topological layers and direct import dependencies.
- A shard may execute in parallel only when its import context is complete and
  deterministic.
- Merge results by package/topological order, not worker completion order.
- Downstream skip behavior must match serial verification.
- Reference checker paths may remain serial until deterministic sharding is
  proven safe.

Implementation notes:

- Added a deterministic fast verifier shard runner in
  `crates/npa-api/src/package_verifier.rs`. The public `--jobs > 1` fast path
  now plans contiguous shards within each topological layer after direct-import
  contexts are complete.
- Shard planning refuses incomplete or same-layer import contexts and falls back
  to independent serial checking rather than parallelizing an unsafe context.
  Shard worker threads use the package verifier's large stack budget so full
  proof-corpus verification can run under `--jobs N`.
- Kept the pre-PAS-19 per-layer parallel verifier as a private test-only
  strategy so shard output can be compared against both `--jobs 1` and the
  legacy parallel path.
- Shard workers merge results by planned shard ordinal and layer order; final
  reports are still rebuilt from package topological order.
- Reference checker mode remains serial and still rejects `jobs > 1`.
- Added API and CLI tests for shard planning, success normalization, failure
  normalization, downstream skip behavior, and preserved verifier diagnostics.

Acceptance criteria:

- `--jobs 1`, existing parallel verification, and shard runner output normalize
  to the same result for success and failure cases.
- Dependency failure still skips downstream modules deterministically.
- Shard execution never hides checker diagnostics.

Verification:

```sh
cargo test -p npa-api package_verifier_shards
cargo test -p npa-cli package_verify_certs_shards
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 4 --json
git diff --check
```

### PAS-20 Incremental Generated Artifact Checks

Status: Completed

Purpose:

PAS-20 makes generated artifact check mode incrementally planned for axiom
report, theorem index, verified export summary, and publish plan. The
completed implementation computes deterministic impacted-module sets from
package lock and module artifact hash changes, keeps checked artifact payloads
under source-free projection comparison, and avoids unnecessary JSON
regeneration for unchanged artifacts.

Files to add or edit:

- `crates/npa-package/src/axiom_report.rs`
- `crates/npa-package/src/theorem_index.rs`
- `crates/npa-package/src/export_summary.rs`
- `crates/npa-package/src/publish_plan.rs`
- package projection CLI modules and tests

Implementation rules:

- Use package lock hash and per-module artifact hashes as the invalidation
  boundary.
- Fall back to full projection comparison when the lock, schema, policy,
  checker profile, or dependency identity changes globally.
- Keep generated artifacts deterministic and byte-for-byte stable.
- Do not use incremental projection output as proof evidence.

Acceptance criteria:

- Checked artifacts and full source-free projections are byte-identical or
  structurally equal for unchanged inputs; canonical payload tampering is still
  rejected.
- Impacted module sets are deterministic and explainable.
- Global metadata changes conservatively force full projection checks.

Verification:

```sh
cargo test -p npa-package package_incremental_generated_artifacts
cargo test -p npa-cli package_projection_incremental
cargo run -p npa-cli -- package axiom-report --root proofs --check --timings summary --json
cargo run -p npa-cli -- package publish-plan --root proofs --check --timings summary --json
git diff --check
```

### PAS-21 Package Gate Shared Snapshot Default

Status: Completed

Purpose:

PAS-21 makes the local package gate consume the PAS-17 shared snapshot command
group by default when the gate-plan impact class allows local acceleration. This
avoids repeated package root loading, package lock parsing, certificate decode,
source-free checker execution, and projection setup across generated artifact
checks.

Implementation rules:

- Standalone CLI commands keep their current output and compatibility behavior.
- Gate scripts may call the shared snapshot command group only for local package
  audit loops; release/high-trust paths keep explicit cache-off commands.
- Shared snapshot output must continue to report `proof_evidence=false`.
- A failure in any subcommand must remain attributable to the same diagnostic
  kind and package-relative path as the standalone command.

Acceptance criteria:

- Package gate output is equivalent to standalone generated artifact checks plus
  fast certificate verification for success and failure cases.
- Removing local caches does not change pass/fail verdicts.
- Gate scripts explain when they selected shared snapshot mode.

Verification:

```sh
cargo test -p npa-cli package_shared_snapshot
./scripts/check-corpus-package.sh
NPA_PACKAGE_GATE_SHARED_SNAPSHOT=0 ./scripts/check-corpus-package.sh
git diff --check
```

Completed notes:

- `scripts/check-corpus-package.sh` now defaults local package gates to the
  PAS-17 shared snapshot command group and prints the selected mode plus the
  `NPA_PACKAGE_GATE_SHARED_SNAPSHOT=0` standalone override.
- `scripts/check-corpus-full.sh` forces the standalone package-gate sequence so
  release/high-trust-adjacent full gates keep explicit cache-off commands.
- `package_shared_snapshot` tests now compare shared and standalone projection
  results for success and failure fixtures, verify cache deletion keeps verdicts
  stable, and exercise the proof-corpus shared path with `proof_evidence=false`
  timing output.

### PAS-22 Persistent Per-Module Verified Result Cache

Status: Completed

Purpose:

PAS-22 persists verified module records across processes so later cache-aware
local audit modes can avoid repeated source-free fast verifier work for
unchanged modules. This milestone starts with read-through validation: the live
verifier still runs and dominates stored records.

Implementation rules:

- Cache keys include package id/version, package-lock schema, module name,
  origin, certificate path, certificate file hash, canonical certificate hash,
  export hash, axiom report hash, direct import identities, core spec,
  certificate format, kernel/checker profile, and verifier cache schema.
- Cache values may store verified module records and checker summaries, but must
  be marked `proof_evidence=false`.
- Initial mode is read-through: live verifier still runs and dominates stored
  records.
- Explicit local-hit mode may be added only after read-through tests prove
  stored records cannot hide stale certificates or dependency identity changes.

Acceptance criteria:

- Cache deletion leaves verifier verdicts unchanged.
- Stale certificate, stale import identity, checker profile change, and package
  lock schema change force live verification.
- Cache hit/miss counters are deterministic and do not affect diagnostics.

Verification:

```sh
cargo test -p npa-api package_verified_result_cache
cargo test -p npa-cli package_verify_certs_persistent_cache
rm -rf target/npa-package-audit-cache
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

Implementation notes:

- `PackageAuditCacheKeyInput` now includes package id/version,
  package-lock schema, lock entry origin, and certificate path in addition to
  the previous certificate/import/checker identity material.
- Persistent verifier memo entries serialize `proof_evidence=false`, and the
  parser rejects entries that claim proof evidence.
- `package verify-certs --verifier-memo read-through` reads and writes the
  disk-backed verifier memo store but always runs live source-free verification;
  exact hits only affect deterministic counters.
- Existing `--verifier-memo disk` remains the explicit local-hit acceleration
  mode, while release/high-trust/package gate scripts keep verifier memo
  disabled by default.

### PAS-23 Reference Checker Summary Cache

Status: Completed

Purpose:

PAS-23 adds a separate persistent cache for reference checker summaries used by
publish-plan and release metadata checks. This targets repeated
`verify-certs --checker reference`-equivalent work without mixing reference
results into the fast verifier cache namespace.

Implementation rules:

- Cache keys include reference checker profile, checker binary identity when
  available, package id/version, package-lock hash, certificate identities,
  import identities, core spec, and certificate format.
- Fast verifier and reference checker caches use distinct schemas and
  directories.
- Publish-plan check may reuse reference summaries only after validating the
  checked axiom report, theorem index, package lock, and certificate identities.
- Release/high-trust gates must have a cache-off reference checker path.

Acceptance criteria:

- Reference summary cache hits never change publish-plan pass/fail verdicts.
- Tampered axiom report, theorem index, package lock, or certificate bytes
  invalidate the reference summary cache.
- JSON timing reports separate fast cache and reference cache counters.

Implementation notes:

- Added `npa.package.reference_summary_cache.v0.1` and
  `npa.package.reference_summary_cache_entry.v0.1` as schemas separate from
  the fast verifier cache and disk verifier memo schemas.
- Reference summary cache entries live under
  `target/npa-package-audit-cache/reference-summary-v0.1`, require
  `trusted=false` and `proof_evidence=false`, and are validated as canonical
  JSON before use.
- `package publish-plan --check --timings summary|detailed` validates checked
  package lock, axiom report, theorem index, and certificate identities before
  reading cached reference summaries. Exact hits synthesize only a local
  non-proof-evidence reference report for publish-plan metadata comparison.
- The default publish-plan path and release/high-trust paths keep a cache-off
  live reference checker route.
- Timing JSON now reports `reference_summary_cache_summary` separately from
  fast verifier cache counters.

Verification:

```sh
cargo test -p npa-api package_reference_summary_cache
cargo test -p npa-cli package_publish_plan_reference_cache
cargo run -p npa-cli -- package publish-plan --root proofs --check --timings summary --json
git diff --check
```

### PAS-24 Persistent Import Context Export Data Cache

Status: Completed

Purpose:

PAS-24 persists dependency import contexts and export data by import-closure
identity. It extends PAS-18 process-local reuse across repeated local package
audit commands.

Implementation rules:

- Cache keys are derived from package-lock dependency identities, not source
  paths.
- Cached export data must include dependency module name, origin, package
  id/version for external imports, export hash, certificate hash, axiom report
  hash, certificate format, and cache schema.
- Cache values are decoded/materialized import contexts only; they are not proof
  evidence and cannot bypass certificate identity checks.
- Any dependency removal, external package identity change, or core/certificate
  format change invalidates the import context cache.

Acceptance criteria:

- Reusing cached import contexts gives byte-identical verifier diagnostics and
  package artifact projections to cache-off execution.
- Dependency identity changes are reported as stale rather than silently reused.
- Cache entries are deterministic and host-path-free.

Implementation notes:

- Added `npa.package.import_context_export_cache.v0.1` and
  `npa.package.import_context_export_cache_entry.v0.1` for deterministic
  import-context export-data entries under
  `target/npa-package-audit-cache/import-context-export-v0.1`.
- Cache entries are keyed by owner module plus ordered package-lock dependency
  identities. The file slot is stable per owner context, while the entry
  `cache_key` includes direct dependency export/certificate/axiom identity so
  dependency changes are reported as stale at the slot.
- Cached export data records dependency module, origin, external package
  id/version, export hash, certificate hash, axiom-report hash, certificate
  format, and cache schema. Entries require `trusted=false` and
  `proof_evidence=false`.
- `package verify-certs` uses the persistent import-context export cache only
  on timings-enabled verifier runs that already collect decode/import counters.
  Cache hits still require the current run's checked imports before the
  reference checker can use the materialized import store.
- The default timings-off verifier output and proof acceptance path remain
  cache-off for this optimization.

Verification:

```sh
cargo test -p npa-package package_import_context_export_cache
cargo test -p npa-api package_import_context_export_cache
cargo test -p npa-cli package_verify_certs_import_context_cache
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

### PAS-25 Cache-Aware DAG Verifier

Status: Completed

Purpose:

PAS-25 combines PAS-20 impacted-module planning with verifier cache records.
Changed modules and reverse dependents run live; unchanged modules may be read
from trusted-false verifier records in explicit local acceleration mode.

Implementation rules:

- Dirty set selection uses package-lock changes, per-module certificate/export
  hashes, direct import identity, checker profile, core spec, and certificate
  format.
- Reverse dependents of every dirty module run live.
- Cached clean modules must still be validated against current package-lock
  identities before reuse.
- Output order and diagnostics normalize to full live topological verification.
- Release/high-trust paths remain cache-off.

Acceptance criteria:

- Cache-aware and full live verification normalize to identical success/failure
  reports on unchanged packages.
- A dirty dependency forces all reverse dependents through live verification.
- Deleting the cache only changes timing/cache counters.

Implementation notes:

- Added metadata-only cache-aware live-set selection in
  `crates/npa-package/src/audit_selection.rs`. Dirty modules are validated
  against the package lock, reverse dependents are selected in topological order,
  and the selection is marked `proof_evidence=false`.
- Added cache-aware disk-memo verifier entry points in
  `crates/npa-api/src/package_verifier.rs`. They accept exact trusted-false
  disk memo hits plus dirty modules; dirty modules, reverse dependents, and any
  imports needed by live modules run through the live checker.
- `package verify-certs --verifier-memo disk` now computes dirty modules from
  non-exact disk memo lookups, filters reverse-dependent live modules out of the
  cache-hit set, and reports `invalidated` in `disk_memo_summary`.
- Release/high-trust and default verifier paths remain cache-off unless
  `--verifier-memo disk` is explicitly selected.

Verification:

```sh
cargo test -p npa-package package_cache_aware_live_selection
cargo test -p npa-api package_cache_aware_dag_verifier
cargo test -p npa-cli package_verify_certs_cache_aware
cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json
git diff --check
```

### PAS-26 Unified Generated Package Check Command

Status: Planned

Purpose:

PAS-26 adds a single source-free CLI command that checks axiom report, theorem
index, verified export summary, publish plan, and fast certificate verification
through one package snapshot. Gate scripts can use it to avoid repeated CLI
startup and repeated checker work while standalone commands remain available.

Implementation rules:

- The command must not read proof source, replay, metadata, AI traces, hidden
  caches, registry network data, or theorem-search sidecars.
- Output contains per-artifact sub-results plus one aggregate result, all
  deterministic and host-path-free.
- Any sub-result failure causes aggregate failure with the original
  package-relative diagnostic.
- The command is local acceleration only and reports `proof_evidence=false`.

Acceptance criteria:

- Aggregate output matches standalone command results for success and failure
  fixtures.
- Gate scripts can opt into the unified command without changing release or
  high-trust command requirements.
- Timing output shows one root load, one package lock load, one certificate
  decode phase, and one checker phase for the combined check.

Verification:

```sh
cargo test -p npa-cli package_generated_check_command
cargo run -p npa-cli -- package check-generated --root proofs --timings summary --json
./scripts/check-corpus-package.sh
git diff --check
```

## 6. Rollout Policy

For PAS-00 through PAS-08, the ordering constraint has already been applied:
`local-hit` was added only after read-through mode had tests proving live
checker results dominate stored entries, and parallel package verification did
not become a default because `--jobs 1` and `--jobs N` normalized behavior was
not fully proven for every checker path.

PAS-09 through PAS-20 are now complete. The completed ordering preserved the
original safety rule: PAS-14 telemetry remained behavior-neutral, PAS-10 through
PAS-12 reduced repeated work without changing gate semantics, PAS-13 turned the
measured impact rules into a deterministic command recommendation, and PAS-15
kept disk verifier memo hits outside proof evidence. PAS-16 integrated the
planner into local gates as report-only guidance by default. PAS-17 added an
in-memory command group that reuses one source-free package snapshot without
changing standalone command output. PAS-18 added process-local decoded
certificate and import-context reuse while keeping live source-free verification
as the acceptance boundary. PAS-19 added deterministic fast-verifier shard
execution while normalizing output back to package topological order. PAS-20
added conservative incremental generated-artifact planning and tests that
canonical checked artifact tampering is still rejected.

After PAS-20, PAS-21 through PAS-26 are follow-up candidates. They should stay
local-audit-only until cache deletion, cache-off package gates, and release
handoff tests prove no proof acceptance decision depends on acceleration data.

The package gate remains the authoritative local gate for package verifier,
package metadata, certificate/checker compatibility, promotion readiness,
release handoff, and high-trust-adjacent changes. This plan only makes repeated
local audit loops cheaper; it does not make cache files part of the proof.
