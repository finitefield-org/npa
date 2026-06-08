# Proof Corpus Package Audit Speed Plan Todo

Source: `develop/proof-corpus-package-audit-speed-plan.md`

このタスク分解は、Go の package build cache / export data / dependency graph
invalidation に相当する考え方を、NPA の package certificate audit に安全に適用するための
実装順を固定します。対象は PAS-00 から PAS-08 までです。

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

## Milestone Order

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

Implement PAS milestones in order. Do not introduce `local-hit` before PAS-02
has live-result-dominates-cache tests. Do not make `--jobs N` the default before
`--jobs 1` and `--jobs N` normalized outputs are proven identical.

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

- Status: Pending
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

### PAS-06 Deterministic Topological Parallel Verification

- Status: Pending
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

### PAS-07 Closure Audit Workflow Integration

- Status: Pending
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

### PAS-08 Final Measurement And Gate Policy Update

- Status: Pending
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
