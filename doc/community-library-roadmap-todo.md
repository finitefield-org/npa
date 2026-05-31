# Community Library Roadmap Todo

Source: `doc/community-library-roadmap.md`

このタスク分解は、NPA を公開し、外部 theorem library / registry に進む前に必要な
package contract、CLI、source-free verification、CI、publish metadata を実装可能な単位へ分けたものです。

---

## Scope

対象:

```text
- `proofs/manifest.toml` と `tools/proof-corpus` を seed とする package contract 化
- `npa.package.v0.1` manifest / lock / artifact model
- 外部 package root を入力に取る package CLI
- source-free checker を package graph 全体に適用する verification flow
- deterministic axiom report / theorem index / publish metadata
- 外部 theorem library 用 CI template
- `npa-mathlib-seed` dogfood repo を作れる状態
```

非対象:

```text
- registry server
- package dependency solver
- binary cache service
- online theorem proving service
- production LLM / RAG integration
- online theorem graph store
- external SMT solver service
- browser IDE
```

信頼境界:

```text
信頼しない:
  source parser / elaborator / tactic / AI / theorem search / API orchestration / registry

信頼する:
  canonical certificate
  Rust kernel verdict
  source-free reference checker verdict
  deterministic export_hash / certificate_hash / axiom_report_hash
```

`npa-api`、package CLI、CI、registry metadata は trusted base ではありません。
kernel crate に filesystem、network、registry lookup、plugin loading、AI 呼び出しを入れてはいけません。

---

## Current Implementation Facts

```text
proofs/manifest.toml
  schema = "npa-ai-proof-corpus-v0.1"
  internal proof corpus manifest, not a generic package manifest

tools/proof-corpus
  hard-coded module list and repo-local layout
  generates source/certificate/meta/replay/manifest for the current corpus

crates/npa-cert
  canonical certificate encode/decode/verify and hashes

crates/npa-checker-ref
  source-free reference checker binary

crates/npa-api/src/independent_checker.rs
  MachineCheckRequest / RunnerPolicy / ImportLockManifest / release audit substrate

scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
  local gates; GitHub Actions workflow is currently removed
```

Resolved CLR-00 decision:

```text
The contributor-facing command contract is `npa package ...`.
The Cargo implementation target is package `npa-cli`, installed binary `npa`.
The detailed CLR-00 breakdown is `doc/community-library-roadmap-clr-00-todo.md`.
```

---

## Milestones

### CLR-00 Fix Package CLI And Schema Decisions

- Status: Completed
- Depends on: None
- Inputs:
  - `doc/community-library-roadmap.md`
  - `README.md`
  - `proofs/manifest.toml`
  - `tools/proof-corpus/src/main.rs`
  - `crates/npa-api/src/independent_checker.rs`
- Code or documentation areas:
  - `doc/community-library-roadmap.md`
  - new package design notes or this task document if the decision changes command names
  - workspace `Cargo.toml` if a new binary crate is introduced
- Deliverables:
  - Fixed command name and binary placement for package operations.
  - Version tags for `npa.package.v0.1`, package lock, generated axiom report, theorem index, and publish metadata.
  - Field-by-field diff between `npa-ai-proof-corpus-v0.1` and target `npa.package.v0.1`.
  - Decision on how package-level `imports` relate to module-level `imports`.
- Acceptance criteria:
  - No later milestone needs to guess the CLI binary name or manifest schema family.
  - The target schema identifies required, optional, forbidden, and generated fields.
  - The design states whether checked-in `proofs/manifest.toml` remains legacy, is migrated in place, or is generated from `npa-package.toml`.
  - Trusted-boundary rules explicitly exclude registry lookup from kernel / checker internals.
- Verification:
  - `rg -n "npa package|npa-package|npa\\.package|npa-ai-proof-corpus" doc/community-library-roadmap.md doc/community-library-roadmap-todo.md README.md`
  - `git diff --check`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-00-todo.md`.
  - Command references are normalized by CLR-00-07: repository verification uses
    `cargo run -p npa-cli -- package ...`, while contributor-facing examples use
    installed `npa package ...`.
  - Readiness is closed by CLR-00-08; CLR-01 can start without additional product
    decisions.
  - Keep this milestone documentation-focused unless a binary crate name is needed to unblock CLR-01.
  - Do not introduce registry network behavior.

### CLR-01 Implement `npa.package.v0.1` Data Model And Validator

- Status: Completed
- Depends on: CLR-00
- Inputs:
  - Target schema from CLR-00
  - `proofs/manifest.toml`
  - `crates/npa-api/src/independent_checker.rs`
  - `crates/npa-cert/src/hash.rs`
- Code or documentation areas:
  - `crates/npa-package`
  - tests for manifest parser / validator
  - documentation for `npa-package.toml`
- Deliverables:
  - Rust data model for package manifest, module entries, import entries, policy, and expected hashes.
  - Strict parser and validator with structured errors.
  - Duplicate module detection, path validation, hash grammar validation, schema version validation, axiom policy validation.
  - Module graph validation including unknown import and cycle detection.
- Acceptance criteria:
  - Invalid manifests fail before certificate build starts.
  - Unknown fields and duplicate keys are rejected or explicitly version-gated.
  - Module imports cannot be accepted by module name alone when hash fields are required.
  - Validator accepts a package representation equivalent to the current `proofs/manifest.toml` corpus.
- Verification:
  - `cargo test -p npa-package package_manifest`
  - `cargo test --workspace package_manifest`
  - `cargo test -p npa-proof-corpus`
  - `git diff --check`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-01-todo.md`.
  - Prefer structured parsing over ad hoc string scanning.
  - The validator lives in `crates/npa-package` and must not depend on `npa-api`.
  - Completed by `crates/npa-package`: it exposes the `npa.package.v0.1` manifest
    data model, structured TOML parser, closed-object validation, hash / path /
    name grammar checks, duplicate checks, import graph validation, axiom
    policy validation, deterministic structured errors, report API, fixtures,
    and crate docs for `npa-package.toml` implementers.
  - Parent acceptance is closed: invalid manifests fail before any certificate
    build, unknown fields and duplicate keys are rejected, module imports
    resolve only to local modules or hash-pinned top-level imports, and the
    proof-corpus-equivalent fixture validates without replacing
    `proofs/manifest.toml`.
  - Trusted base remains unchanged. `npa-package` is untrusted metadata tooling;
    `npa-kernel`, `npa-cert`, and `npa-checker-ref` do not depend on it.
  - CLR-02 can start from this contract. Remaining work belongs there: generate
    the full `proofs/npa-package.toml`, preserve `proofs/manifest.toml` as the
    legacy corpus manifest, and add proof-corpus tests for the complete checked
    artifact set.

### CLR-02 Represent The Existing Proof Corpus As A Package Fixture

- Status: Pending
- Depends on: CLR-01
- Inputs:
  - `proofs/manifest.toml`
  - `doc/community-library-roadmap-clr-02-todo.md`
  - `proofs/Proofs/Ai/**`
  - `tools/proof-corpus/src/main.rs`
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs`
- Code or documentation areas:
  - `tools/proof-corpus`
  - `proofs/npa-package.toml`
  - `proofs/vendor/npa-std/**`
  - package fixture tests in `tools/proof-corpus/tests`
  - package validator fixture coverage when shared coverage belongs in `crates/npa-package`
- Deliverables:
  - A package fixture that describes the current `proofs/` corpus without hard-coding the module graph in the validator.
  - Compatibility path from current `proofs/manifest.toml` to the new package model.
  - Hash-pinned top-level package import entries for `Std.Logic.Eq` and `Std.Nat.Basic`.
  - Deterministic external import certificate artifacts for the current Std imports.
  - Tests proving that package validation preserves the existing module list, source paths, certificate paths, hashes, theorem names, definitions, inductives, and axiom lists.
- Acceptance criteria:
  - `cargo test -p npa-proof-corpus` still verifies checked-in artifacts.
  - The package fixture can be loaded without reading Rust source constants from `tools/proof-corpus/src/main.rs`.
  - The fixture records enough import identity to build source-free checker requests later.
  - Existing proof-corpus generation remains deterministic.
  - `proofs/manifest.toml` remains a legacy `npa-ai-proof-corpus-v0.1` artifact while `proofs/npa-package.toml` uses `npa.package.v0.1`.
- Verification:
  - `cargo run -p npa-proof-corpus`
  - `cargo test -p npa-proof-corpus`
  - `rg -n "schema = \"npa-ai-proof-corpus-v0.1\"|npa.package.v0.1" proofs tools doc/community-library-roadmap-todo.md`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-02-todo.md`.
  - This milestone may keep `npa-ai-proof-corpus-v0.1` as a legacy output, but must define how it maps to package fields.

### CLR-03 Add Import Lock And Source-Free Package Graph Verification

- Status: Pending
- Depends on: CLR-02
- Inputs:
  - `doc/community-library-roadmap-clr-03-todo.md`
  - `crates/npa-api/src/independent_checker.rs`
  - `crates/npa-checker-ref/src/lib.rs`
  - `crates/npa-cert/src/lib.rs`
  - package fixture from CLR-02
- Code or documentation areas:
  - `crates/npa-package`
  - `crates/npa-api` Phase 8 package adapters
  - `crates/npa-checker-ref` only if policy parsing needs narrow extension
  - `proofs/generated/package-lock.json`
  - source-free package verification tests
- Deliverables:
  - Package-level `npa.package.lock.v0.1` artifact with module, export hash, certificate hash, certificate path, certificate file hash, direct imports, and axiom report hash.
  - Derived Phase 8 import locks and MachineCheckRequest materialization from the package lock.
  - Dependency-topological verification of package certificates without reading `.npa` source.
  - Integration with `npa-checker-ref` for reference checker mode.
  - Fast verifier mode for local development, clearly marked separate from reference checker verdict.
- Acceptance criteria:
  - A package graph with missing dependency, hash mismatch, duplicate module, or import cycle fails deterministically.
  - The source-free checker path does not read source, replay, theorem index, or AI trace.
  - Import identity uses `module + export_hash + certificate_hash`; module name alone is insufficient.
  - Verification order is derived from the package graph, not request order.
  - Reference checker mode builds imports from earlier same-checker successful results, not from unchecked directory scanning alone.
- Verification:
  - `cargo test -p npa-package package_lock`
  - `cargo test -p npa-api package_source_free`
  - `cargo test --workspace import_lock`
  - `cargo test -p npa-checker-ref`
  - `cargo test -p npa-api independent_checker`
  - `./scripts/phase8-release-audit.sh`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-03-todo.md`.
  - CLR-03 must derive source-free checker import locks from
    `proofs/npa-package.toml`, not from legacy `proofs/manifest.toml` or
    `tools/proof-corpus` Rust constants.
  - The CLR-02 fixture already records local module certificate paths,
    `expected_certificate_file_hash`, `expected_export_hash`,
    `expected_axiom_report_hash`, and `expected_certificate_hash`, plus
    top-level Std external import certificate paths with `export_hash` and
    `certificate_hash`.
  - CLR-03 derives external import `axiom_report_hash` values by reading the
    pinned external certificates listed in `proofs/npa-package.toml`.
  - `npa.package.lock.v0.1` is distinct from `npa.independent-checker.import_lock_manifest.v1`; the latter is derived per checker run.
  - Do not add full external checker as required in this milestone; keep `npa-checker-ext` target integration.
  - `proofs/generated/package-lock.json` is derived package metadata, not proof
    evidence. Accepted proof evidence remains canonical `.npcert` bytes plus
    the selected checker verdict. The lock records package graph identity,
    certificate paths, certificate file hashes, export hashes, certificate
    hashes, direct imports, and axiom report hashes; it must not introduce
    source, replay, meta, theorem index, AI trace, registry, network, or solver
    data as checker inputs.
  - CLR-03 library entry points for later CLI work are
    `npa_package::build_package_lock_from_package_root`,
    `npa_package::build_package_lock_from_artifacts`,
    `npa_package::parse_package_lock_json`,
    `npa_api::verify_package_fast_source_free`,
    `npa_api::verify_package_reference_source_free`,
    `npa_api::materialize_package_phase8_import_locks`, and
    `npa_api::materialize_package_phase8_requests`.
  - Library verification examples are `cargo test -p npa-api package_fast_verifier`,
    `cargo test -p npa-api package_reference_verifier`,
    `cargo test -p npa-api package_phase8_import_lock_adapter`,
    `cargo test -p npa-api package_phase8_request_materialization`,
    `cargo test -p npa-proof-corpus package_fast_source_free`, and
    `cargo test -p npa-proof-corpus package_reference_source_free`.
  - Raw `npa-checker-ref` CLI import scanning is not enough for high-trust
    package graph verification by itself. High-trust package verification also
    requires the CLR-03 manifest and package-lock validation, pinned import
    identity, dependency-topological order, checker policy mapping, and
    same-checker provenance for imports accepted earlier in the package run.

### CLR-04 Implement Package Build, Verify, And Hash Check Commands

- Status: Completed
- Depends on: CLR-03
- Inputs:
  - `doc/community-library-roadmap-clr-04-todo.md`
  - package model from CLR-01
  - proof corpus package fixture from CLR-02
  - source-free verification from CLR-03
  - `npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy`
- Code or documentation areas:
  - `crates/npa-cli`
  - CLI command parser and structured diagnostics
  - package root filesystem loader
  - package command tests
  - README / roadmap command examples if command naming changes
- Deliverables:
  - `package check` for manifest / graph / policy validation.
  - `package build-certs` for deterministic source-to-certificate build.
  - `package verify-certs` for fast and reference checker verification.
  - `package check-hashes` for expected source / certificate / export / axiom report hash checks.
  - Clear exit codes and structured diagnostics suitable for CI.
- Acceptance criteria:
  - Commands operate on an explicit package root and do not rely on the current working directory except as a default.
  - `build-certs` may read source and replay helper data, but `verify-certs --checker reference` reads certificate/import artifacts only.
  - `check-hashes` fails on stale checked-in artifacts.
  - Running the commands on the current proof corpus reproduces existing hashes.
- Verification:
  - `cargo run -p npa-cli -- package check --root proofs`
  - `cargo run -p npa-cli -- package build-certs --root proofs --check`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference`
  - `cargo run -p npa-cli -- package check-hashes --root proofs`
  - `cargo test -p npa-cli package_cli`
  - `cargo test --workspace package_cli`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-04-todo.md`.
  - `npa-cli` is the Cargo package name fixed by CLR-00; the installed binary is `npa`.
  - Avoid silently rewriting artifacts in check mode.
  - `--changed`, `--all`, and `--checker external` are outside CLR-04 unless a later milestone explicitly adds them.
  - `package verify-certs` wraps the CLR-03 lock builder, lock parser,
    fast/reference source-free verifiers, and Phase 8 materializers rather than
    reimplementing package graph traversal, checker policy mapping, or request
    adapter semantics.
  - `verify-certs --checker reference` uses `npa-checker-ref` as the per-module
    reference verdict engine through the CLR-03 package verifier path; raw
    `npa-checker-ref` CLI import scanning is not sufficient by itself for
    high-trust package graph verification.
  - CLR-04 package verification commands are local package-root operations.
    `build-certs` may read source and replay helper data, but source-free
    verification must consume package metadata and certificate artifacts only.
  - Completed by `crates/npa-cli`: it implements `package check`,
    `package build-certs` check/write mode, `package verify-certs` reference and
    fast modes, `package check-hashes`, common root loading, deterministic JSON
    diagnostics, exit-code mapping, and end-to-end proof-corpus CLI regression
    fixtures.
  - Contributor-facing examples use installed `npa package ...`; repository
    verification examples use `cargo run -p npa-cli -- package ...`.
  - CLI diagnostics, package locks, and generated package metadata are not proof
    evidence. Accepted proof evidence remains canonical `.npcert` bytes plus the
    selected source-free checker / kernel verifier verdict.
  - CLR-05 owns `npa package axiom-report` and `npa package index`; CLR-06 owns
    `npa package publish-plan`.

### CLR-05 Generate Deterministic Axiom Report And Theorem Index Artifacts

- Status: Pending
- Depends on: CLR-04
- Inputs:
  - `doc/community-library-roadmap-clr-05-todo.md`
  - `crates/npa-api/src/std_library.rs`
  - `crates/npa-api/src/search.rs`
  - `crates/npa-api/src/theorem_graph.rs`
  - `proofs/manifest.toml`
  - package verification results
- Code or documentation areas:
  - `crates/npa-package` package artifact schemas
  - `crates/npa-api` package artifact extraction adapters if needed
  - `crates/npa-cli` `package axiom-report` and `package index`
  - `proofs/generated/axiom-report.json`
  - `proofs/generated/theorem-index.json`
  - tests for deterministic ordering and hash stability
- Deliverables:
  - `package axiom-report` producing package-level and module-level axiom report JSON.
  - `package index` producing theorem index suitable for docs, search, and future registry metadata.
  - Canonical ordering for modules, declarations, axioms, theorem entries, tags, and checker summaries.
  - Check mode that compares generated artifacts with checked-in files.
- Acceptance criteria:
  - Same package input produces byte-identical generated artifacts.
  - Axiom report policy violations fail CI.
  - The theorem index includes module, theorem name, statement/interface hash, certificate hash, export hash, modes/tags if available, and axiom dependencies.
  - Generated artifacts do not become checker input.
- Verification:
  - `cargo run -p npa-cli -- package axiom-report --root proofs --check`
  - `cargo run -p npa-cli -- package index --root proofs --check`
  - `cargo test -p npa-cli package_axiom_report`
  - `cargo test -p npa-cli package_index`
  - `cargo test -p npa-package package_artifacts`
  - `cargo test --workspace axiom_report`
  - `cargo test --workspace theorem_index`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-05-todo.md`.
  - The theorem index is search/documentation metadata, not proof acceptance evidence.
  - `axiom-report` and `index` are full-package commands in CLR-05; changed-module selection remains later CI work.
  - Generated axiom report and theorem index artifacts must not require `.npa` source, replay, meta, theorem graph score, prompt metadata, or AI traces.

### CLR-06 Generate Publish Metadata And Registry Seed Artifacts

- Status: Pending
- Depends on: CLR-05
- Inputs:
  - `doc/community-library-roadmap-clr-06-todo.md`
  - registry entry sketch in `doc/community-library-roadmap.md`
  - package manifest / lock / axiom report / theorem index artifacts
  - checker results from CLR-03/CLR-04
- Code or documentation areas:
  - `crates/npa-package` publish-plan and registry seed schemas
  - `crates/npa-cli` `package publish-plan`
  - `proofs/generated/publish-plan.json`
  - release metadata tests
- Deliverables:
  - `package publish-plan` producing package release manifest and module registry entries.
  - Schema for registry seed entries containing package, version, module, core spec, kernel profile, certificate format, export hash, certificate hash, axiom report hash, imports, and checker results.
  - Artifact checksum / signature policy documented as target or MVP behavior.
  - Downstream import bundle metadata sufficient for another package to pin imports without a registry server.
- Acceptance criteria:
  - Publish metadata can be generated from Git release artifacts without contacting a registry service.
  - Registry metadata is treated as helper data; local checker can re-verify certificates from artifacts.
  - The publish plan fails when generated metadata disagrees with package hashes or checker results.
  - The schema explicitly distinguishes package registry metadata from independent checker binary registry metadata.
- Verification:
  - `cargo run -p npa-cli -- package publish-plan --root proofs --check`
  - `cargo test -p npa-cli package_publish_plan`
  - `cargo test -p npa-package publish_plan`
  - `cargo test --workspace publish_plan`
  - `rg -n "npa.registry.module.v0.1|independent-checker.checker_binary_registry" doc crates`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-06-todo.md`.
  - This milestone still does not implement a registry server.
  - MVP signature policy is checksum-only SHA-256; cryptographic signing remains target integration.
  - Publish metadata and registry seed entries are helper data, not proof acceptance evidence.

### CLR-07 Add External Theorem Library CI Template

- Status: Pending
- Depends on: CLR-05
- Inputs:
  - local gates in `scripts/phase8-release-audit.sh` and `scripts/phase9-regression.sh`
  - package commands from CLR-04
  - generated artifact checks from CLR-05
  - `doc/community-library-roadmap-clr-07-todo.md`
- Code or documentation areas:
  - `ci-templates/github-actions`
  - `doc/external-theorem-library-ci.md`
  - package CLI examples
  - `doc/community-library-roadmap.md` if workflow names are fixed
- Deliverables:
  - PR-mode CI template for external theorem libraries.
  - Release/high-trust CI template that runs full package verification and artifact checks.
  - Contributor-facing failure messages or troubleshooting guidance.
  - Documentation that current `npa` repo local gates remain separate from external library CI.
- Acceptance criteria:
  - PR mode requires package check, deterministic build/hash check, full-package reference checker fallback, axiom report check, and index check.
  - Release/high-trust mode requires full package check and leaves external checker as target integration unless CLR-08 is complete.
  - CI template does not rely on local machine state, registry network access, or hidden package cache.
  - Workflow examples match actual package CLI names and flags.
  - Base CLR-07 templates use only package commands and flags already owned by CLR-04 and CLR-05; they do not use `--changed`, `--all`, or `--checker external`.
- Verification:
  - `git diff --check`
  - `rg -n "npa-package-pr|npa-package-release|package check|verify-certs|axiom-report|package index" ci-templates doc README.md`
  - `rg -n -- "--changed|--all|--checker external|--registry|--network|--latest" ci-templates && false || true`
  - dry-run or syntax validation for workflow files if they are added
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-07-todo.md`.
  - Keep external theorem library templates outside `.github/workflows` unless this repository explicitly reintroduces active workflows.
  - Base templates use full-package reference verification as the conservative PR fallback until changed-module selection exists.
  - Publish-plan is optional after CLR-06; external checker required mode remains CLR-08.

### CLR-08 Define And Gate High-Trust External Checker Integration

- Status: Pending
- Depends on: CLR-03
- Inputs:
  - `doc/phase8-human.md`
  - `doc/phase8-ai.md`
  - `crates/npa-api/src/independent_checker.rs`
  - `crates/npa-checker-ref`
  - `doc/community-library-roadmap-clr-08-todo.md`
- Code or documentation areas:
  - `crates/npa-api/src/independent_checker.rs` RunnerPolicy / ReleasePolicy / CheckerBinaryRegistry integration
  - `crates/npa-cli` `package verify-certs --checker external` and package high-trust command
  - `crates/npa-package` `verified_high_trust` schema
  - external checker runner integration
  - high-trust release policy tests
  - generated `verified_high_trust` artifact schema
- Deliverables:
  - Explicit contract for when `npa-checker-ext` becomes required.
  - Source-free package verification path for `--checker external`, gated by runner policy and checker binary registry.
  - `verified_high_trust` artifact schema and generation path, or a documented target-integration placeholder if deferred.
  - Release/high-trust policy tests that do not affect AI candidate hot path.
  - Benchmark / audit collection plan for external checker.
- Acceptance criteria:
  - PR mode remains reference-checker-only unless explicitly configured otherwise.
  - Release/high-trust mode has a clear failure model for fast/reference/external disagreement.
  - External checker runner cannot read source, replay, AI sidecars, theorem index, or registry network data.
  - The integration does not expand the trusted base beyond certificate and checker verdict.
  - `verified_high_trust` is not emitted as verified from reference-checker-only evidence.
  - CLR-08 enables `--checker external` only with explicit runner policy and checker registry; it does not add `--changed`, `--all`, `--registry`, `--network`, or `--latest`.
- Verification:
  - `cargo test -p npa-api independent_checker`
  - `cargo test -p npa-checker-ref`
  - `cargo test -p npa-cli package_verify_external`
  - `cargo test -p npa-package verified_high_trust`
  - `./scripts/phase8-release-audit.sh`
  - targeted search: `rg -n "community-library-roadmap-clr-08-todo|verified_high_trust|npa-checker-ext|external checker|high-trust-reference|CheckerBinaryRegistry" doc crates`
- Notes:
  - Detailed breakdown: `doc/community-library-roadmap-clr-08-todo.md`.
  - This milestone may be deferred until after npa-mathlib-seed if reference-checker-only release seed is acceptable.
  - Policy / runner contract work can start from CLR-03, but package CLI integration depends on CLR-04 and CI integration depends on CLR-07.

### CLR-09 Dogfood `npa-mathlib-seed`

- Status: Pending
- Depends on: CLR-06, CLR-07
- Inputs:
  - package CLI and artifacts from CLR-04/CLR-05/CLR-06
  - current proof corpus modules such as `Proofs.Ai.Basic`, `Proofs.Ai.Eq`, `Proofs.Ai.Nat`
  - external CI template from CLR-07
  - detailed task breakdown in `doc/community-library-roadmap-clr-09-todo.md`
- Code or documentation areas:
  - separate `npa-mathlib-seed` repository, or a local fixture that models that external repository boundary
  - package fixture in `npa` for importing seed release artifacts through the CLR-06 downstream import bundle
  - CLR-07 CI templates and external theorem library documentation
  - contributor workflow and release handoff documentation
- Deliverables:
  - Minimal external theorem library with `npa-package.toml`, source, certificates, replay/meta where useful, axiom reports, theorem index, and CI.
  - Import fixture proving `npa` can consume hash-pinned artifacts from the external library.
  - Seed module scope decision that starts from a closed subset of the current proof corpus instead of moving the full corpus.
  - Release artifact set including package lock, axiom report, theorem index, and `generated/publish-plan.json`.
  - Documented contributor workflow for theorem-only PRs.
- Acceptance criteria:
  - Fresh checkout of the seed library can build certificates, check hashes, run source-free reference verification, check axiom report, and check theorem index.
  - Updating a theorem in the seed library does not require modifying `npa` kernel / checker / certificate code.
  - Downstream package import uses hash-pinned artifacts, not implicit latest registry lookup.
  - The seed library can publish release artifacts usable by another package without a registry server.
  - If CLR-08 is still deferred, seed release evidence is explicitly reference-checker-only and does not emit `verified_high_trust`.
- Verification:
  - Seed repo CI passes from fresh checkout.
  - In `npa`: package import fixture test passes.
  - In `npa`: downstream import bundle mismatch tests reject corrupted seed hashes.
  - `cargo test --workspace` in `npa` after adding any integration fixture.
  - targeted search: `rg -n "community-library-roadmap-clr-09-todo|npa-mathlib-seed|reference-checker-only|publish-plan|downstream_import_bundle" doc README.md ci-templates crates`
- Notes:
  - Start with a small subset. Do not move the entire proof corpus until package ergonomics are proven.
  - Detailed breakdown: `doc/community-library-roadmap-clr-09-todo.md`.
  - CLR-09 depends on CLR-06 and CLR-07. It can proceed without CLR-08 only when the seed release policy is explicitly reference-checker-only.

### CLR-10 Registry Readiness Review

- Status: Pending
- Depends on: CLR-06, CLR-09
- Inputs:
  - completed package CLI
  - seed library release artifacts
  - generated publish metadata
  - source-free checker results
  - detailed task breakdown in `doc/community-library-roadmap-clr-10-todo.md`
- Code or documentation areas:
  - `doc/community-library-roadmap.md`
  - `doc/registry-readiness.md` or equivalent release-readiness decision record
  - release checklist / registry readiness checklist
  - issue tracker, local follow-up backlog, or registry-server implementation plan
- Deliverables:
  - Registry readiness checklist with pass/fail evidence.
  - Decision on whether to create registry server, continue Git-release-based registry seed, or defer.
  - List of registry-server requirements that are not already solved by package artifacts.
  - Trusted-boundary audit confirming registry metadata is discoverability data and not checker input.
  - Follow-up backlog for registry server, signing, dependency solving, binary cache, and UX items that remain outside the package contract.
- Acceptance criteria:
  - Every blocker from `doc/community-library-roadmap.md` section 4.2 has concrete pass/fail evidence.
  - No registry requirement asks the kernel, checker, or certificate verifier to read network data.
  - The next step can be implemented without revisiting package manifest semantics.
  - Remaining non-goals are intentionally deferred.
  - If CLR-08 is deferred, the readiness decision explicitly distinguishes reference-checker-only release evidence from unavailable high-trust evidence.
- Verification:
  - `rg -n "Registry 前の blocker|npa.registry.module.v0.1|npa.package.v0.1" doc/community-library-roadmap.md doc/community-library-roadmap-todo.md`
  - `rg -n "community-library-roadmap-clr-10-todo|registry readiness|Git-release-based registry seed|not checker input|reference-checker-only" doc README.md`
  - `git diff --check`
- Notes:
  - This is a review / release-readiness milestone, not the registry server implementation.
  - Detailed breakdown: `doc/community-library-roadmap-clr-10-todo.md`.

---

## Review Findings

Initial review against `doc/community-library-roadmap.md`, README, Phase 8 docs, and current implementation produced these findings and resolutions:

```text
F1: The source design's M2 mixed build, verify, hash check, and source-free graph semantics.
    Resolution: split into CLR-03 and CLR-04, with import lock/source-free graph verification before CLI command completion.

F2: The design originally did not fix the Cargo package that owns `npa package`.
    Resolution: CLR-00 fixes `npa-cli` as the Cargo package and `npa` as the installed binary.

F3: Registry metadata could be confused with the independent checker binary registry in Phase 8 docs.
    Resolution: CLR-06 requires explicit distinction between module registry metadata and checker binary registry metadata.

F4: External checker / verified_high_trust target integration could block the first seed library unnecessarily.
    Resolution: CLR-08 is separate and may be deferred; CLR-09 depends on package CI and reference checker, not full external checker.

F5: Dependency review found that external CI requires deterministic axiom report / theorem index generation,
    and the seed library requires publish metadata before proving release artifact flow.
    Resolution: CLR-07 now depends on CLR-05, and CLR-09 depends on both CLR-06 and CLR-07.
```

No open findings remain in this task breakdown.

---

## Validation Plan

For documentation-only changes to this task file:

```sh
git diff --check
rg -n "TO""DO|TB""D|未""定|PLACE""HOLDER" doc/community-library-roadmap-todo.md
rg -n "npa-package|npa package|npa\\.package|registry|verified_high_trust|npa-checker-ext" \
  doc/community-library-roadmap.md doc/community-library-roadmap-todo.md README.md doc/phase8-human.md doc/phase8-ai.md
```

For implementation milestones, run the verification commands listed in each milestone and the relevant repo gates.
