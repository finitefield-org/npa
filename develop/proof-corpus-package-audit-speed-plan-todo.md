# Proof Corpus Package Audit Speed Plan Todo

Source: `develop/proof-corpus-package-audit-speed-plan.md`

このタスク分解は、Go の package build cache / export data / dependency graph
invalidation に相当する考え方を、NPA の package certificate audit に安全に適用するための
実装順を固定します。対象は PAS-00 から PAS-14 までです。

## Scope

対象:

- `npa package verify-certs` の repeated local audit loop 短縮。
- package checker result store、verified export summary、reverse dependency selection、
  topological layer execution、closure audit guidance の実装。
- `npa-mathlib` promotion readiness / closure audit の局所反復を速くする補助。
- package gate と closure audit loop の baseline / final measurement。
- 関連 CLI help、diagnostic、repo-local skill、operator docs の最小更新。

非対象:

- proof acceptance criteria の変更。
- cache hit、verified export summary、timing log、package audit selection、promotion plan を
  proof evidence として扱うこと。
- release / high-trust / public `npa-mathlib` handoff の最終 source-free checker gate を
  省略すること。
- kernel、`npa-cert`、`npa-checker-ref` に filesystem / network / registry / plugin /
  package-manager behavior を入れること。
- proof source、replay、metadata、AI trace、theorem index、registry network data、hidden cache を
  checker input として読むこと。

現在の実装前提:

- package command / verifier の主な実装点は `crates/npa-cli/src/package_verify.rs`、
  `crates/npa-cli/src/args.rs`、`crates/npa-api/src/package_verifier.rs`、
  `crates/npa-package/src/lock.rs` にある。
- proof corpus package gate は `./scripts/check-corpus-package.sh`、
  full gate は `./scripts/check-corpus-full.sh`。
- `check-corpus-package.sh` / `check-corpus-full.sh` / release or high-trust scripts は
  cache-off が既定であり、`local-hit` を入れてはいけない。
- `npa package check`、`check-hashes`、`build-certs --check`、
  `verify-certs --checker reference`、`axiom-report --check`、`index --check`、
  checked-in publish plan がある場合の `publish-plan --check` は promotion /
  release 境界の evidence command として扱う。

## Trusted Boundary

```text
信頼しない:
  cache file
  verified export summary
  package lock
  package audit selection
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

全 milestone の共通ルール:

- Deleting `target/npa-package-audit-cache/**` must not change proof acceptance
  or rejection.
- `read-through` may read/write cache, but live checker result dominates.
- `local-hit` may skip selected local checks only when explicitly requested, and
  every output must mark that it is not proof evidence.
- Final promotion / release / high-trust handoff must record cache-off
  source-free verification.

## Milestone Dependencies

| ID | Title | Depends on | Main output |
| --- | --- | --- | --- |
| PAS-00 | Baseline Package Audit Profile | None | Baseline timing document |
| PAS-01 | Package Audit Identity Model | PAS-00 | Stable cache key and audit identity API |
| PAS-02 | Read-Through Result Store | PAS-01 | Safe cache read/write mode that still runs checker |
| PAS-03 | Local-Hit Mode For Explicit Local Audits | PAS-02 | Local-only cache hit mode with evidence markers |
| PAS-04 | Verified Export Summary Artifact | PAS-03 | Source-free export summary command and artifact |
| PAS-05 | Reverse Dependency Audit Selection | PAS-04 | Deterministic selected-module planner |
| PAS-06 | Deterministic Topological Parallel Verification | PAS-05 | `--jobs` execution planning and safe parallel mode |
| PAS-07 | Closure Audit Workflow Integration | PAS-06 | Closure audit guidance and promote-plan integration |
| PAS-08 | Final Measurement And Gate Policy Update | PAS-07 | Final measurement and policy documentation |
| PAS-09 | Build-Certs Check Reuse | PAS-08 | Local cache for repeated `build-certs --check` work |
| PAS-10 | Shared Package Snapshot Projection | PAS-09 | One source-free package snapshot reused by projection checks |
| PAS-11 | Package CLI Example Tiering | PAS-10 | Smoke vs full corpus package CLI example tiers |
| PAS-12 | Dependency-Level Verification Memo | PAS-11 | Process-local verifier memo for repeated module checks |
| PAS-13 | Impact-Aware Gate Planner | PAS-12 | Deterministic gate recommendation from diff impact |
| PAS-14 | Audit Timing Telemetry | PAS-09 | Optional phase timing JSON for package commands |

Implement PAS dependencies in order. PAS-00 through PAS-08 have already applied
the original ordering rule: `local-hit` followed PAS-02 read-through tests, and
`--jobs N` did not become the default because `--jobs 1` and `--jobs N`
normalized behavior was not fully proven for every checker path. After PAS-08,
start with PAS-09. PAS-14 is behavior-neutral telemetry and may be pulled
forward immediately after PAS-09 if phase-level timings are needed before
PAS-10. Do not relax package gate tiers in PAS-11 until PAS-09 and PAS-10 have
moved the expensive repeated work behind deterministic cache-off or
shared-snapshot coverage.

## Milestones

### PAS-00 Baseline Package Audit Profile

- Status: Completed
- Depends on: None
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 3 and 5 PAS-00
  - `./scripts/check-corpus-package.sh`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-api/src/package_verifier.rs`
  - `proofs/generated/package-lock.json`
- Files to add or edit:
  - Add `develop/proof-corpus-package-audit-baseline-pas-00.md`.
  - Do not edit Rust code, proof artifacts, package manifests, generated package
    artifacts, or gate scripts.
- Implementation tasks:
  - Record repo path, commit, branch, dirty/clean state, date, timezone, OS,
    machine model, CPU thread count, memory, Rust version, Cargo version, and
    whether Cargo target cache is warm.
  - Time the full package gate once with `/usr/bin/time -p ./scripts/check-corpus-package.sh`.
  - Time each package-gate subcommand listed in the source plan with exact
    command text and pass/fail status.
  - Inspect the checked package lock with a temporary local command or script
    outside the repository to count package modules, external imports, lock
    entries, direct import edges, local reverse edges, and topological layers.
  - Identify the top three bottleneck commands by real time.
  - State explicitly that timing data is not proof evidence.
- Deliverables:
  - Baseline document with raw command timings and graph inventory.
  - A comparison anchor that PAS-08 can cite by filename, commit, and date.
- Acceptance criteria:
  - Baseline is reproducible enough for later before/after comparison.
  - Measurement does not leave changes under `proofs/`, `tools/proof-corpus/`,
    `scripts/`, or `crates/`.
  - No repository helper is added solely for baseline graph counts.
- Verification:
  - `git diff --name-only -- proofs tools/proof-corpus scripts crates`
  - `git diff --check`
- Notes:
  - If PAS-01 has already landed during a rerun, `package_audit_graph_inventory`
    may be used for graph counts. The initial PAS-00 implementation should not
    add that helper.
  - Completed in `develop/proof-corpus-package-audit-baseline-pas-00.md`.

### PAS-01 Package Audit Identity Model

- Status: Completed
- Depends on: PAS-00
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.1, 4.2, 4.3, and 5 PAS-01
  - `crates/npa-package/src/lib.rs`
  - `crates/npa-package/src/lock.rs`
  - Existing package artifact JSON / hash helper patterns in `crates/npa-package/src`
- Files to add or edit:
  - Add `crates/npa-package/src/audit_cache.rs`.
  - Export the module from `crates/npa-package/src/lib.rs`.
  - Add unit tests in `audit_cache.rs`.
  - No CLI files in this milestone.
- Implementation tasks:
  - Add constants:
    - `PACKAGE_AUDIT_CACHE_SCHEMA`
    - `PACKAGE_AUDIT_RESULT_SCHEMA`
    - `PACKAGE_VERIFIED_EXPORT_SUMMARY_SCHEMA`
    - `PACKAGE_AUDIT_CACHE_LAYOUT_DIR`
  - Add identity types:
    - `PackageAuditCheckerIdentity`
    - `PackageAuditImportIdentity`
    - `PackageAuditCacheKeyInput`
    - `PackageAuditCachedStatus`
    - `PackageAuditResultEntry`
    - `PackageAuditGraphInventory`
  - Add public helpers:
    - `package_audit_cache_key_material`
    - `package_audit_cache_key`
    - `package_audit_result_entry_json`
    - `parse_package_audit_result_entry_json`
    - `validate_package_audit_result_entry`
    - `package_audit_direct_imports_for_entry`
    - `package_audit_graph_inventory`
  - Canonicalize by sorting and deduplicating direct imports by module,
    `export_hash`, and `certificate_hash`.
  - Sort and deduplicate `enabled_core_features`.
  - Serialize JSON in fixed field order without depending on `HashMap`
    iteration order.
  - Hash key material with the existing package SHA-256 formatter.
  - Reject unknown schema, invalid hash, missing `trusted = false`, and malformed
    checker identity.
  - Compute graph inventory from `PackageLockGraph`; do not inspect source,
    replay, meta, theorem index, AI trace, network, or hidden caches.
- Deliverables:
  - Stable, deterministic package audit identity API.
  - Canonical cache result entry JSON parser / writer.
  - Graph inventory helper usable by later measurement and selection work.
- Acceptance criteria:
  - Same input produces byte-identical key material and cache key.
  - Changing package lock hash, checker build hash, certificate hash, direct
    import identity, or enabled core feature changes the cache key.
  - Cache result entries validate only with `trusted = false`.
  - Public API has no filesystem or checker execution dependency.
- Verification:
  - `cargo test -p npa-package package_audit`
  - `git diff --check`
- Notes:
  - This is the foundational identity layer. Avoid reaching into CLI or verifier
    behavior until PAS-02.
  - Completed in `crates/npa-package/src/audit_cache.rs`.

### PAS-02 Read-Through Result Store

- Status: Completed
- Depends on: PAS-01
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.1, 4.4, and 5 PAS-02
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-cli/src/diagnostic.rs`
  - `crates/npa-package/src/audit_cache.rs`
- Files to add or edit:
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-cli/src/diagnostic.rs`, only if needed for deterministic diagnostics.
  - `crates/npa-package/src/audit_cache.rs`, only for helper additions discovered
    during CLI wiring.
- Implementation tasks:
  - Add `PackageAuditCacheMode { Off, ReadThrough }` in `args.rs`.
  - Add `audit_cache: PackageAuditCacheMode` to `PackageVerifyCertsOptions`.
  - Parse `--audit-cache off|read-through`; default to `off`; reject duplicate
    or unknown values deterministically.
  - Update `HelpTopic::PackageVerifyCerts`.
  - Keep the existing preflight sequence: load root, read checked lock, read
    certificates, regenerate package lock, reject stale lock, then run checker.
  - Add a CLI-local key input builder that derives PAS-01
    `PackageAuditCacheKeyInput` from checked lock entry, graph imports, package
    lock hash, manifest policy hash, checker identity, and certificate bytes.
  - Store results under
    `target/npa-package-audit-cache/results-v0.1/{cache-key}.json` relative to
    `std::env::current_dir()` captured at command start.
  - For `off`, call the existing verifier path unchanged.
  - For `read-through`, look up cache before live checking, run the live checker
    for every selected module, then write or repair one result entry per module.
  - Discard or overwrite stale entries when stored entry differs from live
    status or key material.
  - Add `PackageAuditVerificationRun` and `PackageAuditCacheSummary` with mode,
    hits, misses, stale, schema misses, written, live checked, cached, and
    `trusted = false`.
  - Include deterministic text / JSON cache summary fields.
  - Reject `--checker external --audit-cache read-through` unless the milestone
    also fully wires external checker identity and live-result comparison.
- Deliverables:
  - `npa package verify-certs --audit-cache read-through`.
  - Cache files that are safe to delete and never authoritative.
  - Deterministic cache summary in text / JSON output.
- Acceptance criteria:
  - `--audit-cache off` is byte-compatible with current pass/fail behavior.
  - `read-through` cannot turn a failed live checker result into success.
  - Removing `target/npa-package-audit-cache` changes only cache counters, not
    the verification verdict.
  - Release and high-trust scripts do not pass `read-through`.
  - External checker mode is either unsupported with a deterministic diagnostic
    or has equivalent live-result-dominates-cache coverage.
- Verification:
  - `cargo test -p npa-cli package_verify_certs_audit_cache`
  - `cargo test -p npa-api --lib package_verifier`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache read-through --json`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache off --json`
  - `git diff --check`
- Notes:
  - Do not derive cache location from `--root`; it is local build output, not
    package metadata.
  - Completed with `--audit-cache read-through` support in `crates/npa-cli/src/package_verify.rs`.

### PAS-03 Local-Hit Mode For Explicit Local Audits

- Status: Completed
- Depends on: PAS-02
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.1, 4.5, and 5 PAS-03
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-api/src/package_verifier.rs`
  - `scripts/check-corpus-package.sh`
  - `scripts/check-corpus-full.sh`
- Files to add or edit:
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-api/src/package_verifier.rs`
  - Gate scripts only for tests or comments proving they do not use `local-hit`.
- Implementation tasks:
  - Extend `PackageAuditCacheMode` with `LocalHit`.
  - Parse `--audit-cache local-hit` only for `npa package verify-certs`.
  - Reject `local-hit` for external checker unless external read-through identity
    and process-result identity are fully implemented and tested.
  - Reuse PAS-02 key and lookup helpers.
  - Add `PackageModuleVerificationEvidence { LiveChecker, LocalAuditCache }` to
    `PackageModuleVerificationResult` or an equivalent explicit evidence marker.
  - Emit `proof_evidence = false` and a cache-off follow-up command whenever any
    module result is synthesized from cache.
  - Process modules in topological order.
  - Use the conservative first implementation: a cached module required as an
    import by a later live-checked module must be live-checked in the same run.
  - Allow cache hits only for modules that do not need to provide fresh
    reference-checker import-store material to later live modules.
  - Keep downstream skip semantics aligned with the existing verifier when a
    dependency fails.
  - Ensure package/full/release/high-trust scripts continue to omit
    `--audit-cache`.
- Deliverables:
  - Explicit local-only `local-hit` mode.
  - Evidence marker visible in deterministic text / JSON output.
  - Tests proving local hits are not confused with live checker proof evidence.
- Acceptance criteria:
  - `local-hit` cannot be confused with live checker evidence.
  - `check-corpus-package.sh` and `check-corpus-full.sh` remain cache-off.
  - Local-hit output tells the operator exactly which cache-off command to run
    before promotion, release, or high-trust handoff.
  - Local-hit does not mask live failures for cache misses or required
    live-checked dependencies.
- Verification:
  - `cargo test -p npa-cli package_verify_certs_local_hit`
  - `./scripts/check-corpus-package.sh`
  - `git diff --check`
- Notes:
  - This milestone intentionally does not serialize reference-checker internal
    import stores. That larger optimization can be a later plan.
  - Implemented explicit `local-hit` mode with per-module evidence markers,
    `proof_evidence=false` diagnostics for cache-synthesized results, and a
    cache-off follow-up command for promotion/release/high-trust handoff.

### PAS-04 Verified Export Summary Artifact

- Status: Completed
- Depends on: PAS-03
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.2, 4.3, and 5 PAS-04
  - `crates/npa-package/src/audit_cache.rs`
  - `crates/npa-package/src/lib.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package.rs`
  - Existing package artifact command patterns in `crates/npa-cli/src`
- Files to add or edit:
  - `crates/npa-package/src/audit_cache.rs`, or add
    `crates/npa-package/src/export_summary.rs` if the type set becomes too large.
  - `crates/npa-package/src/lib.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/src/lib.rs`, if a new CLI source file is added.
  - Prefer `crates/npa-cli/src/package_export_summary.rs` or existing package
    artifact command modules for write/check behavior.
- Implementation tasks:
  - Add `PackageVerifiedExportSummary` and
    `PackageVerifiedExportSummaryModule`.
  - Include package identity, version, package lock hash, module list,
    summary hash, and `trusted = false`.
  - For each module, include module name, origin, certificate path,
    certificate file hash, export hash, certificate hash, axiom report hash,
    direct import identities, exported globals, module axioms, and core features.
  - Derive summary only from package manifest, package lock, and `.npcert`
    bytes. Do not read source, replay, meta, theorem index, AI trace, network,
    or hidden cache.
  - Certificate decoding is allowed. Reuse existing source-free verifier helpers
    such as `verify_package_fast_source_free_with_modules` if verified module
    records are needed for exported globals or module axioms.
  - Compute `summary_hash` over the summary with the hash field omitted or
    zeroed, following existing package artifact hash patterns.
  - Validate schema, `trusted = false`, module hashes, direct import identities,
    and summary hash.
  - Pick and document deterministic module order. Prefer canonical module order
    for stable diff review unless package-lock topological order is already the
    local package artifact convention.
  - Add CLI command
    `npa package export-summary [--root PATH] [--json] [--check] [--out PATH]`.
  - Wire `PackageCommand::ExportSummary(PackageExportSummaryOptions)`,
    `HelpTopic::PackageExportSummary`, command dispatch from `package.rs`, and
    package help listing.
  - Default write path to `generated/verified-export-summary.json`; `--check`
    compares without mutating.
- Deliverables:
  - Deterministic source-free export summary artifact.
  - `npa package export-summary` command with write/check/json modes.
  - Validation tests for tampering and source-free operation.
- Acceptance criteria:
  - Summary generation is deterministic and source-free.
  - Summary validation rejects stale or tampered fields.
  - Artifact and command output state that the summary is not proof evidence.
  - `--check` mode does not mutate generated artifacts.
- Verification:
  - `cargo test -p npa-package verified_export_summary`
  - `cargo test -p npa-cli package_export_summary`
  - `cargo run -p npa-cli -- package export-summary --root proofs --check --json`
  - `git diff --check`
- Notes:
  - The summary accelerates planning and metadata questions; it never replaces
    certificate bytes for checking.
  - Implemented as `generated/verified-export-summary.json` with
    package-lock-topological module order, `trusted=false`, and an explicit
    `proof_evidence=false` command diagnostic.

### PAS-05 Reverse Dependency Audit Selection

- Status: Completed
- Depends on: PAS-04
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.3, 4.4, and 5 PAS-05
  - `crates/npa-package/src/lock.rs`
  - `crates/npa-package/src/lib.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
- Files to add or edit:
  - Add `crates/npa-package/src/audit_selection.rs`.
  - Export it from `crates/npa-package/src/lib.rs`.
  - `crates/npa-cli/src/args.rs`, only if selection reporting CLI is added.
  - `crates/npa-cli/src/package_verify.rs`, only if selection reporting CLI is added.
- Implementation tasks:
  - Add change model:
    - `PackageAuditChangeKind`
    - `PackageAuditChangedModule`
    - `PackageAuditSelectionReason`
    - `PackageAuditSelectedModule`
    - `PackageAuditSelection`
  - Add `package_lock_reverse_dependencies(lock)`.
  - Add `select_package_audit_modules(lock, changed)`.
  - Build and validate the package lock graph before selection.
  - Include explicitly changed modules.
  - For `ExportHashChanged`, include the full reverse dependency closure.
  - For `CertificateHashChanged` without `ExportHashChanged`, include only the
    changed module.
  - For `AxiomReportHashChanged`, include the changed module and mark package
    axiom-report/index checks as required in the result model or diagnostic.
  - For policy, checker identity, core spec, or certificate format changes,
    select all modules.
  - Deduplicate selected modules and reasons deterministically.
  - Sort selected output in package-lock topological order.
  - Optional CLI:
    `--changed-module MODULE[:certificate|export|axiom|file]`,
    `--audit-selection explicit|reverse-deps`, and
    `--print-audit-selection`.
  - If the CLI is added in PAS-05, `--print-audit-selection` must report and
    exit before checker execution.
- Deliverables:
  - Deterministic reverse dependency selection API.
  - Optional selection-only CLI diagnostics.
  - Tests explaining why each selected module is included.
- Acceptance criteria:
  - Stable-export certificate changes do not select downstream modules for
    semantic audit.
  - Export hash changes select all reverse dependents.
  - Policy, checker identity, core spec, or certificate format changes select
    all modules.
  - Selection output does not imply unselected modules were verified.
- Verification:
  - `cargo test -p npa-package package_audit_selection`
  - `cargo test -p npa-cli package_audit_selection`, only if the PAS-05 CLI is added.
  - `git diff --check`
- Notes:
  - Actual partial verification of selected modules belongs to PAS-06.
  - Implemented as an `npa-package` API only; PAS-05 did not add the optional
    selection-reporting CLI.

### PAS-06 Deterministic Topological Parallel Verification

- Status: Completed
- Depends on: PAS-05
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.5 and 5 PAS-06
  - `crates/npa-package/src/audit_selection.rs`
  - `crates/npa-package/src/lock.rs`
  - `crates/npa-api/src/package_verifier.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
- Files to add or edit:
  - `crates/npa-package/src/audit_selection.rs` or `crates/npa-package/src/lock.rs`
    for topological layer construction.
  - `crates/npa-api/src/package_verifier.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package_verify.rs`
- Implementation tasks:
  - Add `PackageTopologicalLayers`.
  - Add `PackageVerificationExecutionOptions { jobs, selected_modules }`.
  - Add `package_lock_topological_layers(lock)`.
  - Add verifier entry points with execution options for fast and reference
    source-free verification:
    - `verify_package_fast_source_free_with_options`
    - `verify_package_reference_source_free_with_options`
  - Parse `--jobs N`; default to `1`; reject `0`, duplicate values, and
    non-integers deterministically.
  - Preserve existing behavior for `--jobs 1`.
  - Apply PAS-05 selected module set to verifier execution options.
  - Layer selected modules so layer 0 has no imports inside the selected set,
    layer N depends only on earlier layers, and modules inside each layer are
    sorted by canonical package-lock order.
  - Enable `--jobs > 1` only for checker implementations that can preserve
    semantics. First implementation may allow it only for fast verifier.
  - For reference checker, either execute serially with deterministic
    `parallel_reference_deferred` diagnostic or reject `--jobs > 1`
    deterministically.
  - Store worker results by module and emit text / JSON in deterministic
    topological order, never completion order.
  - Use content-addressed temp files and atomic rename for concurrent cache
    writes.
  - Do not execute later layers that depend on a failed module.
- Deliverables:
  - Deterministic topological layer planner.
  - `--jobs` CLI for package verification.
  - Safe partial selected-module execution path.
  - Parallel fast verifier path, or explicit deferral for unsupported checkers.
- Acceptance criteria:
  - `--jobs 1` preserves existing behavior.
  - Successful `--jobs N` output is normalized-identical to `--jobs 1` after
    ignoring timing fields.
  - Diagnostics for a fixed failing module match `--jobs 1` after ignoring
    timing fields.
  - Parallelism is not enabled for checker paths that cannot preserve import
    store semantics.
- Verification:
  - `cargo test -p npa-package package_lock_topological_layers`
  - `cargo test -p npa-api --lib package_verifier_parallel`
  - `cargo test -p npa-cli package_verify_certs_jobs`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 1 --json`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 4 --json`
  - `git diff --check`
- Notes:
  - Do not default to local CPU count in release-like commands until a later
    policy update explicitly allows it.
  - Implemented `--jobs > 1` for cache-off fast verification. Reference,
    external, and local audit-cache modes reject parallel jobs deterministically.
  - `selected_modules` execution verifies transitive imports needed to build the
    source-free import context; the selected set is a requested audit set, not a
    claim that dependencies can be skipped without cache/import evidence.

### PAS-07 Closure Audit Workflow Integration

- Status: Completed
- Depends on: PAS-06
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.6 and 5 PAS-07
  - `.agents/skills/closure-audit/SKILL.md`
  - `.agents/skills/judge-promote-to-mathlib/SKILL.md`
  - `tools/proof-corpus/src/main.rs`
  - Existing `develop/npa-mathlib-*-closure-audit.md` documents as examples only.
- Files to add or edit:
  - `.agents/skills/closure-audit/SKILL.md`
  - `.agents/skills/judge-promote-to-mathlib/SKILL.md`, only if it should point
    at the new selection commands.
  - `tools/proof-corpus/src/main.rs`, only if `--promote-plan` or
    `--promote-materialize` should print new audit commands.
  - `develop/proof-corpus-package-audit-speed-plan.md`, only for direct guidance
    corrections discovered during integration.
- Implementation tasks:
  - Update closure-audit guidance to separate local acceleration commands from
    final evidence commands.
  - Require the final evidence checklist to include cache-off `package check`,
    `check-hashes`, `build-certs --check`, `verify-certs --checker reference
    --audit-cache off`, `axiom-report --check`, `index --check`,
    `publish-plan --check`, and downstream smoke when applicable.
  - Require closure audit notes to include:
    - `local_audit_cache_mode`
    - `selected_modules`
    - `selection_reasons`
    - `cache_hits`
    - `live_checker_count`
    - `skipped_by_stable_export`
    - `final_cache_off_verification`
    - `trust_boundary_note`
  - If PAS-05 CLI exists, update promote-plan output to include package audit
    selection commands.
  - If PAS-05 CLI does not exist, keep promote-plan text explicit that selection
    is internal or pending and do not invent a runnable command.
  - Preserve existing `--promote-materialize` behavior unless this milestone
    explicitly changes and tests it.
  - Add fixture tests if `tools/proof-corpus` generated plan text changes.
  - Do not bulk-edit historical closure audit documents.
- Deliverables:
  - Closure audit skill that allows `read-through` / `local-hit` for local
    iteration but cannot end with those as final evidence.
  - Promote-plan or guidance text that names the cache-off final package gate.
  - Final checklist text that cannot be satisfied by cache or selection output
    alone.
  - Tests or targeted text checks covering the guidance.
- Acceptance criteria:
  - Closure audit guidance always ends with cache-off reference verification for
    final readiness.
  - Publish-plan and downstream smoke requirements remain visible where the
    closure audit target crosses a public `npa-mathlib` handoff boundary.
  - Local-hit and read-through are labeled local acceleration only.
  - Existing promotion materialize behavior remains unchanged unless explicitly
    updated and tested.
- Verification:
  - `git diff --check`
  - `cargo run -p npa-proof-corpus -- --promote-plan Proofs.Ai.Basic --mathlib-root ../npa-mathlib --to-module Mathlib.Logic.Basic --out /tmp/npa-pas07-plan.md`
  - `rg -n "audit-cache|cache-off|proof evidence|source-free" /tmp/npa-pas07-plan.md .agents/skills/closure-audit/SKILL.md`
- Notes:
  - `../npa-mathlib` availability may vary locally. If the command cannot run
    because the sibling checkout is missing or dirty, record the blocker instead
    of weakening acceptance criteria.
  - PAS-05 does not expose a public package audit selection CLI yet; promote-plan
    now names the internal `npa_package::select_package_audit_modules` API and
    records selection output as non-evidence.
  - `--promote-plan` now separates local `read-through` / `local-hit`
    acceleration from final cache-off promotion evidence. `--promote-materialize`
    behavior was intentionally left unchanged in this milestone.
  - Added targeted fixture tests for cache-off final gate text, local-hit
    non-evidence text, and closure-audit skill package audit guidance.

### PAS-08 Final Measurement And Gate Policy Update

- Status: Completed
- Depends on: PAS-07
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 5 PAS-08 and 6
  - `develop/proof-corpus-package-audit-baseline-pas-00.md`
  - `README.md`
  - `CONTRIBUTING.md`
  - `AGENTS.md`
  - `develop/internal-readme-notes-ja.md`
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
- Files to add or edit:
  - Add `develop/proof-corpus-package-audit-pas-08-measurement.md`.
  - Update README / CONTRIBUTING / AGENTS / internal notes only if default
    operator guidance changes.
  - Do not edit proof source, certificates, package manifests, or generated
    package artifacts as part of measurement.
- Implementation tasks:
  - Cite PAS-00 baseline by filename, commit, date, and machine context.
  - Measure current full package gate with `/usr/bin/time -p ./scripts/check-corpus-package.sh`.
  - Measure cache-off, read-through, and local-hit reference verifier runs with
    exact commands, pass/fail status, real/user/sys time, and cache counters.
  - Measure fast verifier `--jobs 1` and `--jobs N` runs and compare normalized
    output.
  - Record cache hit/miss/stale counts, live checker count, skipped checker
    count, and final cache-off verification result.
  - Pick one small existing closure audit target with no unrelated dirty package
    changes and record local acceleration separately from final evidence.
  - If `../npa-mathlib` is missing or dirty in an unrelated way, document the
    blocker and skip closure-loop timing rather than fabricating data.
  - Identify remaining top three bottlenecks.
  - State explicitly that cache and timing logs are not proof evidence.
  - Update operator docs only to clarify actual policy. Do not relax package
    gate policy if cache-off package gate remains required.
- Deliverables:
  - Final measurement document comparing PAS-00 to PAS-08.
  - Optional doc updates preserving release / promotion / high-trust cache-off
    final evidence.
  - Clear statement of remaining bottlenecks and next candidates.
- Acceptance criteria:
  - Final measurement includes at least one passing cache-off package
    verification.
  - Any local-hit speedup is reported separately from proof evidence.
  - Documentation preserves explicit package/full gates for promotion, release,
    checker, certificate, and high-trust boundaries.
  - `local-hit` is not added to `check-corpus-package.sh`,
    `check-corpus-full.sh`, release scripts, or high-trust scripts.
- Verification:
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `git diff --name-only -- proofs tools/proof-corpus scripts crates`
  - `git diff --check`
- Notes:
  - If the package gate remains intentionally expensive, PAS-08 should say so
    directly and separate local iteration improvements from final gate cost.
  - Added `develop/proof-corpus-package-audit-pas-08-measurement.md`.
  - Final cache-off reference verification passed for the proof corpus package:
    `real 253.88s`, 230 live-checked modules.
  - Local-hit reference verification passed in `real 1.38s` with 230 cache hits,
    zero live checker runs, and `proof_evidence=false`.
  - Representative clean `../npa-mathlib` closure-loop verification passed
    cache-off in `real 51.54s`; local-hit passed in `real 2.28s` with 66 cache
    hits and `proof_evidence=false`.
  - The PAS-08 measurement document does not claim a dedicated PAS-08
    full-gate timing. A later pre-main-merge `check-corpus-package.sh` run
    completed successfully as merge verification, but it is not used as the
    PAS-08 measurement baseline.
  - Fast `--jobs 4` proof-corpus measurement failed with stack overflow
    (`real 37.32s`); normalized comparison with `--jobs 1` is unavailable and
    recorded as a remaining issue.
  - README / CONTRIBUTING / AGENTS / internal notes were not changed because the
    existing default operator policy already keeps package/full/release/high
    trust gates cache-off and does not add `local-hit` to scripts.
  - `check-fast.sh` exposed clippy-only issues unrelated to PAS-08 behavior;
    they were fixed mechanically in `crates/npa-cert/src/tests.rs`,
    `crates/npa-package/src/audit_selection.rs`,
    `crates/npa-api/src/package_verifier.rs`, and
    `crates/npa-cli/src/package_verify.rs`.

### PAS-09 Build-Certs Check Reuse

- Status: Completed
- Depends on: PAS-08
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.7 and 5 PAS-09
  - `tools/proof-corpus/src/main.rs`
  - existing `package build-certs --check` tests in `crates/npa-cli/tests/`
  - `crates/npa-package/src/audit_cache.rs`
- Files to add or edit:
  - Add `crates/npa-package/src/build_check_cache.rs`, or extend
    `crates/npa-package/src/audit_cache.rs` if the API remains small.
  - Export the module from `crates/npa-package/src/lib.rs`.
  - `tools/proof-corpus/src/main.rs`
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/tests/package_build_certs_check.rs`
  - `scripts/check-corpus-package.sh`, only for an explicit cache-off guard or
    comment. Do not enable local-hit in the script.
- Implementation tasks:
  - Add `PackageBuildCheckCacheMode { Off, ReadThrough }` first.
  - Add key input type with schema, tool version/build hash, core spec,
    certificate format, module, source hash, direct import identities, compiler
    options, package metadata mode, and expected certificate/export/axiom hashes.
  - Add deterministic key material and SHA-256 key helpers.
  - Add cache entry writer/parser requiring `trusted=false` and
    `build_evidence=false`.
  - Wire `--build-check-cache off|read-through` into
    `package build-certs --check`.
  - In read-through mode, always run the live build comparison, then write or
    repair the cache entry.
  - Emit counters for hits, misses, stale entries, written entries, and live
    builds in JSON.
  - Reject duplicate/unknown cache mode values with deterministic usage
    diagnostics.
- Deliverables:
  - Local build-check result store for repeated `build-certs --check` loops.
  - CLI flag and help text.
  - Tests proving read-through cannot hide stale generated artifacts.
- Acceptance criteria:
  - `--build-check-cache off` preserves existing behavior and output after
    normalizing any explicitly requested timing/cache counters.
  - `read-through` still performs live source-to-certificate comparison.
  - Deleting `target/npa-package-audit-cache/**` changes only counters and
    timings.
  - Package/full/release/high-trust scripts do not use local-hit or skip live
    build comparison.
- Verification:
  - `cargo test -p npa-package build_check_cache`
  - `cargo test -p npa-cli package_build_certs_check`
  - `cargo run -p npa-cli -- package build-certs --root proofs --check --build-check-cache off --json`
  - `git diff --check`
- Notes:
  - Add `local-hit` only in a later follow-up if read-through metrics show the
    cache keys are stable and the output can clearly mark
    `build_evidence=false`.
  - Added `crates/npa-package/src/build_check_cache.rs` with canonical key
    material, parser/writer helpers, and validation requiring `trusted=false`
    and `build_evidence=false`.
  - Added `--build-check-cache off|read-through` for
    `npa package build-certs --check`; `off` remains the default and preserves
    existing empty-diagnostics JSON output.
  - `read-through` always runs the live build comparison and records only local
    counters plus untrusted result entries. It does not add local-hit skipping.
  - `scripts/check-corpus-package.sh` keeps build-certs check cache disabled and
    documents that cache entries are not proof evidence or build evidence.
  - Verification passed:
    `cargo test -p npa-package build_check_cache`,
    `cargo test -p npa-cli package_build_certs_check`,
    `cargo test -p npa-cli package_cli_args`,
    `cargo clippy -p npa-cli --all-targets -- -D warnings`,
    `./scripts/check-fast.sh`, and `git diff --check`.
  - The full proof-corpus command
    `cargo run -p npa-cli -- package build-certs --root proofs --check --build-check-cache off --json`
    was attempted, but was still CPU-bound after more than 25 minutes and was
    terminated without a pass/fail result. No cache mode was used for that
    attempted run.

### PAS-10 Shared Package Snapshot Projection

- Status: Completed
- Depends on: PAS-09
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.8 and 5 PAS-10
  - `crates/npa-api/src/package_artifacts.rs`
  - `crates/npa-package/src/export_summary.rs`
  - existing axiom-report, theorem-index, export-summary, and publish-plan CLI
    command implementations.
- Files to add or edit:
  - `crates/npa-api/src/package_artifacts.rs`
  - `crates/npa-package/src/export_summary.rs`
  - projection command modules under `crates/npa-cli/src/`
  - tests for axiom-report, index, export-summary, and publish-plan.
- Implementation tasks:
  - Add an internal `PackageAuditSnapshot` type with manifest hash, package lock
    hash, policy hash, certificate artifact buffers, decoded module records,
    verified export summary records, topological order, reverse dependency
    graph, and projection input hashes.
  - Add a builder that reads only package manifest, package lock, certificate
    bytes, checked generated artifacts when in check mode, and policy inputs.
  - Reject stale lock, stale certificate file hash, path escape, or policy
    mismatch before snapshot reuse.
  - Refactor projection helpers so standalone commands can either build their
    own snapshot or receive one from a combined test/gate path.
  - Add one combined in-process test that runs axiom-report, theorem-index,
    export-summary, and publish-plan from the same snapshot.
  - Keep public CLI output byte-compatible unless `--timings` or a future
    combined command is explicitly requested.
- Deliverables:
  - Reusable source-free snapshot API for package projections.
  - Combined projection test proving snapshot reuse.
- Acceptance criteria:
  - Standalone projection commands still pass unchanged.
  - Combined snapshot output matches standalone generated artifact bytes.
  - Snapshot data is not serialized as proof evidence and has no proof
    acceptance status.
  - Snapshot builder does not read source, replay, AI traces, theorem index as a
    checker input, network, or hidden caches.
- Verification:
  - `cargo test -p npa-cli package_projection_snapshot`
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check --json`
  - `cargo run -p npa-cli -- package index --root proofs --check --json`
  - `cargo run -p npa-cli -- package export-summary --root proofs --check --json`
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check --json`
  - `git diff --check`

### PAS-11 Package CLI Example Tiering

- Status: Completed
- Depends on: PAS-10
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.9 and 5 PAS-11
  - `crates/npa-cli/tests/package_cli.rs`
  - `scripts/check-corpus-package.sh`
  - `scripts/check-corpus-full.sh`
- Files to add or edit:
  - `crates/npa-cli/tests/package_cli.rs`
  - `scripts/check-corpus-package.sh`
  - `scripts/check-corpus-full.sh`
  - release/high-trust helper docs or scripts only if they name package CLI
    example tests.
- Implementation tasks:
  - Split `package_cli_examples_pass_on_proof_corpus` into clearly named smoke
    and full corpus tests.
  - Smoke tests must cover help text, argument parsing, JSON shape, and
    check-mode behavior using small fixtures or metadata-only proof-corpus
    commands.
  - Full corpus tests must continue to cover full proof-corpus
    `build-certs --check`, `verify-certs`, projection checks, and publish-plan
    examples by exact test name.
  - Update package/full gate scripts to document which tier they run.
  - Do not remove full proof-corpus coverage from all gates. If the package gate
    drops the monolithic full CLI example, `check-corpus-full.sh` or explicit
    release/high-trust guidance must still run it.
- Deliverables:
  - Visible smoke/full test names.
  - Gate script comments or command changes that make the cost tier explicit.
- Acceptance criteria:
  - Developers can run smoke CLI examples quickly by exact test name.
  - Full corpus CLI examples remain runnable by exact test name.
  - Gate policy remains explicit about when full corpus CLI examples are
    required.
  - No package gate relaxation happens unless PAS-09/PAS-10 coverage is present
    for the expensive commands that were formerly covered by the monolithic
    example.
- Verification:
  - `cargo test -p npa-cli package_cli_smoke`
  - `cargo test -p npa-cli package_cli_full_corpus`
  - `./scripts/check-corpus-package.sh`
  - `git diff --check`

### PAS-12 Dependency-Level Verification Memo

- Status: Planned
- Depends on: PAS-11
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.10 and 5 PAS-12
  - `crates/npa-api/src/package_verifier.rs`
  - `crates/npa-package/src/audit_cache.rs`
  - `crates/npa-cli/src/package_verify.rs`
- Files to add or edit:
  - `crates/npa-api/src/package_verifier.rs`
  - `crates/npa-package/src/audit_cache.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-cli/tests/package_verify_certs.rs`
- Implementation tasks:
  - Add process-local verifier memo data structures first; defer disk-backed
    memo until process-local behavior is proven.
  - Key memo entries by checker mode, checker identity, package lock hash,
    module, certificate file hash, certificate hash, export hash, direct import
    identities, policy hash, and enabled core features.
  - Keep fast and reference checker namespaces separate.
  - Add execution option to enable/disable memoization.
  - Ensure memo hits do not change normalized success/failure output, dependency
    skip behavior, or diagnostic ordering.
  - Do not memoize external checker timeout, signal, resource, or environment
    errors.
  - Add JSON counters only behind explicit diagnostics/timings output.
- Deliverables:
  - Process-local verifier memo for repeated checks in one package gate run.
  - Normalization tests for memo enabled vs disabled.
- Acceptance criteria:
  - Memo disabled preserves current behavior.
  - Memo enabled cannot turn failure into success.
  - Dependency failure still skips downstream modules deterministically.
  - Output order remains package/topological order, not memo lookup order.
- Verification:
  - `cargo test -p npa-api --lib package_verifier_memo`
  - `cargo test -p npa-cli package_verify_certs_memo`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --json`
  - `git diff --check`

### PAS-13 Impact-Aware Gate Planner

- Status: Planned
- Depends on: PAS-12
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.11 and 5 PAS-13
  - `AGENTS.md` package gate policy
  - `scripts/check-fast.sh`
  - `scripts/check-corpus-authoring.sh`
  - `scripts/check-corpus-package.sh`
  - `scripts/check-corpus-full.sh`
- Files to add or edit:
  - Add `crates/npa-package/src/gate_plan.rs`.
  - Export it from `crates/npa-package/src/lib.rs`.
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/tests/package_cli_args.rs`
  - Add `crates/npa-cli/tests/package_gate_plan.rs`.
  - Update `AGENTS.md` or `develop/internal-readme-notes-ja.md` only after the
    command is stable enough to recommend.
- Recommended CLI:
  - `npa package gate-plan --base origin/main --root proofs --json`
- Implementation tasks:
  - Read changed paths from `git diff --name-only <base>...HEAD` or an explicit
    file list input for tests.
  - Classify changes into docs-only, proof authoring, package
    metadata/projection, checker/certificate semantics, kernel/core semantics,
    and release/high-trust-adjacent.
  - Map each class to required commands and optional local acceleration
    commands.
  - Explain escalation reasons in deterministic order.
  - Include changed proof modules and package generated artifacts where
    derivable from paths.
  - Avoid invoking heavy gates from the planner; it only prints a plan.
- Deliverables:
  - Deterministic gate recommendation API and CLI.
  - Tests for representative changed-file sets.
- Acceptance criteria:
  - Docs-only changes do not recommend package/full gates.
  - Kernel, certificate format, checker, package verifier, package lock, or
    generated package artifact changes escalate to package/full gates according
    to existing policy.
  - The planner never says a proof is accepted; it only recommends commands.
  - Output includes a trust-boundary note.
- Verification:
  - `cargo test -p npa-package package_gate_plan`
  - `cargo test -p npa-cli package_gate_plan`
  - `cargo run -p npa-cli -- package gate-plan --base origin/main --root proofs --json`
  - `git diff --check`

### PAS-14 Audit Timing Telemetry

- Status: Planned
- Depends on: PAS-09
- Inputs:
  - `develop/proof-corpus-package-audit-speed-plan.md` sections 4.12 and 5 PAS-14
  - package CLI args and command modules
  - PAS-00 / PAS-08 timing documents for desired fields.
- Files to add or edit:
  - `crates/npa-cli/src/args.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/src/package_verify.rs`
  - package projection command modules
  - `crates/npa-cli/tests/package_cli_args.rs`
  - package command tests covering JSON output shape.
- CLI shape:
  - `--timings off|summary|detailed`
- Implementation tasks:
  - Add `PackageTimingMode { Off, Summary, Detailed }`.
  - Parse timing mode consistently across package subcommands that opt in.
  - Add a small timing collector with stable phase names and millisecond units.
  - Instrument root load, lock load, certificate decode, graph construction,
    selection, cache lookup, checker, projection, artifact compare, JSON write,
    and total time where applicable.
  - Keep default output unchanged with timings off.
  - Add normalization helpers in tests that remove timing values before
    comparing command semantics.
  - Document that timings are informational and not proof evidence.
- Deliverables:
  - Optional JSON timing telemetry for package verification and projection
    commands.
  - Tests for parser behavior and timing output shape.
- Acceptance criteria:
  - Existing JSON consumers do not see timing fields unless requested.
  - Timing field names and units are stable.
  - Commands that do not execute a phase omit it or emit zero consistently.
  - Timings never influence pass/fail status.
- Verification:
  - `cargo test -p npa-cli package_timings`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --timings summary --json`
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check --timings summary --json`
  - `git diff --check`
- Completion notes:
  - Implemented `--timings off|summary|detailed` for package verification and
    projection commands (`axiom-report`, `index`, `export-summary`, and
    `publish-plan`).
  - Timing JSON is emitted only when requested, uses `ms` phase fields, and
    records `proof_evidence=false` / `build_evidence=false`.
  - Verification passed with the commands listed above, plus `./scripts/check-fast.sh`.

## Review Checklist

Use this checklist when implementing or reviewing each PAS milestone:

- Does the milestone keep cache / summary / selection artifacts outside the
  trusted proof boundary?
- Does every new CLI mode have deterministic parsing errors and help text?
- Does every local acceleration mode report whether live checker evidence was
  used?
- Does deleting local cache output leave verification verdicts unchanged, except
  for expected cache counters?
- Are source-free commands still source-free?
- Are package/full/release/high-trust gates still cache-off unless explicitly
  documented otherwise?
- Are JSON/text outputs deterministic and independent of parallel worker
  completion order?
- Does the milestone avoid unrelated proof source, certificate, manifest, or
  generated artifact churn?
