# Community Library Roadmap CLR-10 Todo

Source milestone: `CLR-10 Registry Readiness Review` in
`doc/community-library-roadmap-todo.md`.

## Purpose

CLR-10 is the decision gate after package metadata and the seed theorem library
have been dogfooded. It does not build a registry server. It reviews evidence
from CLR-06 and CLR-09, checks every blocker in
`doc/community-library-roadmap.md` section 4.2, and decides one of three next
steps:

```text
create registry server
continue Git-release-based registry seed
defer registry work
```

The result must be concrete enough that the next implementation step can start
without reopening package manifest semantics, source-free verification rules,
or trusted-boundary decisions.

## Scope

CLR-10 includes:

- A registry readiness checklist with pass/fail evidence for every section 4.2
  blocker.
- A decision record for registry server creation, Git-release-based registry
  seed continuation, or deferral.
- A list of registry-server requirements not solved by the package artifact
  contract.
- A trusted-boundary audit for registry metadata, package metadata, source-free
  checker results, and optional high-trust evidence.
- A follow-up backlog that can live in an issue tracker or in a local
  documentation file if no tracker is available.
- Updates to the community roadmap so contributors can see the next ecosystem
  step.

CLR-10 excludes:

- Implementing a registry server.
- Implementing package dependency solving.
- Implementing a binary cache service.
- Adding network fetch to kernel, certificate, checker, or proof acceptance
  paths.
- Adding implicit latest-version import resolution.
- Implementing production LLM, RAG, online theorem graph, browser IDE, or
  online theorem proving services.
- Requiring external-checker high-trust evidence when CLR-08 is deferred.

## Trusted Boundary

The readiness review must preserve the existing certificate-first boundary:

```text
trusted:
  canonical certificate
  Rust kernel verdict
  source-free reference checker verdict
  deterministic export hash, certificate hash, and axiom report hash

not trusted:
  registry server
  registry metadata
  package publish metadata
  theorem index
  CI workflow pass status
  release upload automation
  API endpoint response
  source, replay, tactic trace, AI trace, and search metadata
```

A future registry may help users discover packages and download artifacts. It
must not become a proof acceptance boundary. Downstream acceptance still
requires hash-pinned certificates and local source-free verification.

The kernel, `npa-cert`, and `npa-checker-ref` must not read network data,
registry state, hidden package caches, API responses, or package solver state.

## Current Repository Facts

At the time this task list was written, the repository still did not contain
implemented `crates/npa-package`, `crates/npa-cli`, `ci-templates/`,
`proofs/generated/`, or `doc/external-theorem-library-ci.md` paths. CLR-10
therefore depends on the future implementation outputs of CLR-06, CLR-07, and
CLR-09 rather than assuming those artifacts already exist.

The source roadmap section 4.2 names eight registry blockers:

```text
1. package manifest
2. package CLI
3. CI contract
4. external package import resolution
5. source-free package verification
6. deterministic public artifacts
7. publish metadata
8. external dogfood repo
```

CLR-06 provides the publish-plan, registry seed entries, release artifact list,
and downstream import bundle needed for the readiness review. CLR-09 provides
the seed theorem library release, downstream import fixture, contributor
workflow, and dogfood gaps.

CLR-08 is optional for the first readiness decision. If CLR-08 remains
deferred, CLR-10 must record that the evidence is reference-checker-only and
that `verified_high_trust` is unavailable.

## Readiness Evidence Model

Create or update a readiness decision record, preferably:

```text
doc/registry-readiness.md
```

The record must contain one row for each section 4.2 blocker with these fields:

```text
blocker_id
blocker_name
status
evidence_artifacts
verification_commands
trusted_boundary_result
remaining_gap
follow_up_owner_or_file
decision_impact
```

Allowed status values:

```text
pass
fail
deferred
not evaluated
```

`not evaluated` is allowed only while collecting evidence. The final CLR-10
decision must not leave any section 4.2 blocker in that state.

Evidence artifacts should reference package-relative paths and real release
artifacts from CLR-06 and CLR-09. The readiness record must not use fake hash
values or placeholder release IDs.

## Section 4.2 Blocker Evidence Requirements

### Blocker 1: Package Manifest

Required evidence:

- `npa.package.v0.1` manifest schema documentation.
- Seed library `npa-package.toml`.
- Validation output from `npa package check --root . --json`.
- Negative validation evidence for unknown fields, duplicate modules, invalid
  paths, missing hashes, module-name-only imports, and forbidden registry
  lookup fields.

Pass rule:

- External package graph, source paths, certificate paths, imports, expected
  hashes, and axiom policy can be read from the manifest without `npa`
  repository-local assumptions.

Fail rule:

- Any package import can still be accepted by module name alone, hidden local
  layout, or implicit registry state.

### Blocker 2: Package CLI

Required evidence:

- `npa-cli` installed binary contract for `npa package ...`.
- Passing command output for:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Pass rule:

- A fresh checkout can run the commands with an explicit package root and
  deterministic diagnostics.

Fail rule:

- Commands require hidden working-directory state, local cache assumptions,
  registry lookup, or source-trusting verification.

### Blocker 3: CI Contract

Required evidence:

- CLR-07 PR and release CI template paths.
- Seed library CI run result from a fresh checkout.
- Toolchain pinning evidence for `npa`, Rust, and any template dependencies.
- Static check showing base templates do not use unsupported package flags or
  implicit package resolution.

Pass rule:

- External theorem libraries can copy the templates and run package validation,
  deterministic artifact checks, and source-free reference verification without
  registry network access.

Fail rule:

- CI requires hidden package caches, floating tool versions, branch writeback,
  source-trusting shortcuts, or external checker as a default PR gate.

### Blocker 4: External Package Import Resolution

Required evidence:

- CLR-06 downstream import bundle in `generated/publish-plan.json`.
- Seed downstream import fixture from CLR-09.
- Positive fixture result for valid seed artifacts.
- Negative fixture results for corrupted export hash, certificate hash,
  artifact file hash, package name, and package version.

Pass rule:

- A downstream package can import a seed module using package, version, module,
  export hash, certificate hash, and certificate artifact hash without registry
  lookup.

Fail rule:

- Import resolution requires module name alone, mutable latest-version lookup,
  source files, theorem index contents, replay files, or registry network data.

### Blocker 5: Source-Free Package Verification

Required evidence:

- Package lock artifact.
- Reference checker command output.
- Tests proving source, replay, meta, theorem index, AI traces, and registry
  metadata are not read as proof evidence.
- Dependency-topological verification evidence for local and imported
  certificates.

Pass rule:

- The package graph can be checked from certificates and import artifacts in a
  deterministic dependency order.

Fail rule:

- Verification depends on source re-elaboration, tactic replay, AI sidecars,
  theorem index contents, registry metadata, or unchecked directory scanning.

### Blocker 6: Deterministic Public Artifacts

Required evidence:

- `generated/package-lock.json`.
- `generated/axiom-report.json`.
- `generated/theorem-index.json`.
- `generated/publish-plan.json`.
- Check-mode evidence that stale or non-canonical artifacts fail.
- Determinism evidence showing no timestamps, absolute paths, host names,
  environment variables, registry URLs, or network-derived fields.

Pass rule:

- Re-running the artifact commands produces byte-identical canonical outputs
  for unchanged inputs.

Fail rule:

- Artifacts contain host-specific data, non-deterministic ordering, source text
  used as proof evidence, or mutable registry resolution fields.

### Blocker 7: Publish Metadata

Required evidence:

- `npa.package.publish_plan.v0.1`.
- `npa.registry.module.v0.1` module entries.
- Release artifact list with exact file hashes.
- Checksum-only MVP signature policy from CLR-06.
- Source-free checker summaries used by publish-plan validation.
- Schema separation evidence for theorem package registry metadata and
  independent checker binary registry metadata.

Pass rule:

- Publish metadata can be generated from release artifacts without contacting a
  registry service and is sufficient for downstream hash-pinned imports.

Fail rule:

- Publish metadata is treated as proof evidence, includes registry URLs or
  mutable selectors, or confuses theorem package registry entries with checker
  binary registry entries.

### Blocker 8: External Dogfood Repo

Required evidence:

- `npa-mathlib-seed` repository or fixture that models a standalone external
  repository.
- Fresh-checkout package command results.
- Seed CI result.
- Seed release artifact set.
- Downstream import fixture result in the `npa` repository.
- Contributor workflow documentation.
- Dogfood gap list from CLR-09.

Pass rule:

- A theorem-only seed library change can be reviewed, checked, released, and
  consumed downstream without changing `npa` kernel, certificate, checker, or
  trusted import rules.

Fail rule:

- Seed updates still require `npa` trusted-base changes, hidden local paths,
  active workflows in the wrong repository, or registry server behavior.

## Decision Rules

### Create Registry Server

Choose this only when:

- all eight section 4.2 blockers pass;
- downstream packages can already consume Git release artifacts without a
  registry server;
- the desired registry work is discoverability, publication workflow,
  namespace management, metadata mirroring, or UX;
- registry failure cannot change proof acceptance;
- the server requirements can be implemented without changing
  `npa.package.v0.1` semantics or checker input rules.

The first registry server milestone must be scoped as an untrusted metadata and
artifact distribution service.

### Continue Git-Release-Based Registry Seed

Choose this when:

- the package contract is sound;
- seed release artifacts are consumable downstream;
- remaining gaps are mostly publishing UX, namespace policy, signing policy,
  metadata indexing, or contributor ergonomics;
- a server would add operational complexity before it removes a real blocker.

In this decision, the next work should improve release artifacts, docs,
automation, and seed library scale before server implementation.

### Defer Registry Work

Choose this when any of these are true:

- a section 4.2 blocker fails and affects the package trust boundary;
- package manifest semantics are still changing;
- source-free package verification is incomplete;
- downstream import still depends on registry lookup, latest-version
  resolution, source files, replay files, or theorem index contents;
- seed dogfood cannot be completed from a fresh checkout;
- registry requirements would force kernel, certificate, or checker network
  access.

In this decision, create follow-up work against the failing blockers rather
than starting a server.

## Registry Server Requirements Not Solved By Package Artifacts

If the readiness decision allows server work, the registry implementation plan
must separately specify:

- package namespace and ownership policy;
- authenticated publish workflow;
- immutable package-version and module-version storage;
- artifact retention, yanking, deprecation, and mirror policy;
- metadata ingestion from `generated/publish-plan.json`;
- checksum and future signing policy;
- source-free verification replay or audit job policy;
- search and browse indexes over theorem metadata;
- moderation and abuse response;
- rate limits and storage limits;
- API versioning for registry metadata;
- offline export or mirror format;
- compatibility policy for `npa.package.v0.1` and
  `npa.registry.module.v0.1`;
- incident response for bad metadata, missing artifacts, and revoked releases.

These requirements are product and operations work. They must not change the
checker's proof acceptance rule.

## Task Breakdown

### CLR-10-01 Define Registry Readiness Checklist Format

Dependencies:

- CLR-06 publish-plan schema and release artifact contract.
- CLR-09 seed library handoff.
- `doc/community-library-roadmap.md` section 4.2.

Implementation:

1. Create or update `doc/registry-readiness.md`.
2. Define the evidence table with the fields from this task document.
3. Add one row for each of the eight section 4.2 blockers.
4. Add decision rule sections for server creation, Git-release continuation,
   and deferral.
5. Add a trusted-boundary section explaining that registry metadata is not
   checker input.

Acceptance criteria:

- Every section 4.2 blocker has a named readiness row.
- The checklist has concrete evidence fields, not prose-only status.
- The decision record can be completed without inventing package semantics.
- Registry metadata is explicitly classified as untrusted helper data.

Verification:

```sh
rg -n "Registry 前の blocker|blocker_id|registry metadata|not checker input|Git-release-based registry seed" doc/registry-readiness.md doc/community-library-roadmap.md
git diff --check
```

### CLR-10-02 Collect CLR-06 Publish Metadata Evidence

Dependencies:

- CLR-06 completion.
- `generated/publish-plan.json` from the package under review.
- Package lock, axiom report, theorem index, and source-free checker summaries.

Implementation:

1. Record the publish-plan schema and file path.
2. Record module registry seed entry evidence.
3. Record downstream import bundle evidence.
4. Record release artifact list and checksum-only signature policy evidence.
5. Record publish-plan check-mode command output.
6. Record evidence that publish metadata contains no registry URL, network
   fetch field, or mutable version selector.
7. Record schema separation evidence between `npa.registry.module.v0.1` and
   independent checker binary registry schemas.

Acceptance criteria:

- Blocker 6 and blocker 7 have pass/fail evidence.
- Evidence uses real generated artifacts from the package under review.
- Publish metadata is not represented as proof evidence.
- Any missing publish artifact becomes a failing or deferred checklist row.

Verification:

```sh
npa package publish-plan --root . --check --json
rg -n "npa.package.publish_plan.v0.1|npa.registry.module.v0.1|downstream_import_bundle|checksum-only|checker_binary_registry" doc/registry-readiness.md generated
```

### CLR-10-03 Collect CLR-09 Seed Dogfood Evidence

Dependencies:

- CLR-09 completion.
- Seed release artifacts.
- Downstream import fixture.
- Seed CI results.
- Contributor workflow docs.

Implementation:

1. Record the seed package root or external repository URL.
2. Record fresh-checkout package command results.
3. Record seed CI result and toolchain pinning evidence.
4. Record release artifact set and publish-plan path.
5. Record downstream import fixture positive result.
6. Record downstream import fixture negative hash-mismatch results.
7. Record contributor workflow and review policy docs.
8. Record dogfood gaps that should become follow-up work.

Acceptance criteria:

- Blocker 3, blocker 4, blocker 5, and blocker 8 have pass/fail evidence.
- Seed evidence does not rely on hidden paths in the `npa` repository.
- Downstream import evidence is hash-pinned and registry-free.
- The dogfood gap list is specific enough to create follow-up tasks.

Verification:

```sh
npa package check --root . --json
npa package verify-certs --root . --checker reference --json
cargo test -p npa-package downstream_import_bundle
cargo test -p npa-cli package_import_fixture
rg -n "npa-mathlib-seed|fresh checkout|downstream_import_bundle|reference-checker-only|theorem-only" doc/registry-readiness.md README.md CONTRIBUTING.md
```

### CLR-10-04 Audit Trusted Boundary And Non-Goals

Dependencies:

- CLR-10-02.
- CLR-10-03.
- AGENTS.md trusted-boundary rules.
- README architecture and development notes.

Implementation:

1. Check that no readiness row requires kernel, certificate, or checker network
   access.
2. Check that registry metadata, theorem index, CI pass status, publish-plan
   metadata, source, replay, tactic traces, and AI traces are never listed as
   proof evidence.
3. Check that optional CLR-08 evidence is recorded separately from
   reference-checker-only evidence.
4. Check that package dependency solver, binary cache, browser IDE, online
   theorem proving, production LLM, RAG, and online theorem graph work remain
   non-goals unless a separate milestone is created.
5. Record any trusted-boundary concern as a blocking readiness finding.

Acceptance criteria:

- The checklist has a trusted-boundary result for every blocker.
- No registry requirement expands the trusted base.
- If CLR-08 is deferred, `verified_high_trust` is marked unavailable rather
  than approximated from reference-checker-only evidence.
- Non-goals are explicitly deferred or moved to later follow-up work.

Verification:

```sh
rg -n "not checker input|not proof evidence|reference-checker-only|verified_high_trust|kernel.*network|registry lookup" doc/registry-readiness.md doc/community-library-roadmap-clr-10-todo.md
git diff --check
```

### CLR-10-05 Make Registry Path Decision

Dependencies:

- CLR-10-01 through CLR-10-04.

Implementation:

1. Apply the decision rules from this task document.
2. Choose exactly one decision: create registry server, continue
   Git-release-based registry seed, or defer registry work.
3. Record the decision date, evidence summary, and blocking conditions.
4. Record why the other two options were not chosen.
5. Record whether CLR-08 high-trust evidence was available.
6. Record the next milestone or implementation plan entry.

Acceptance criteria:

- The decision is unambiguous.
- The chosen path follows from the evidence table.
- No failing section 4.2 trust-boundary blocker is ignored.
- The next step can be started without reopening package manifest semantics.

Verification:

```sh
rg -n "Decision|create registry server|continue Git-release-based registry seed|defer registry work|why not" doc/registry-readiness.md
git diff --check
```

### CLR-10-06 Identify Registry Server Requirements And Follow-Up Backlog

Dependencies:

- CLR-10-05.
- Dogfood gap list from CLR-09.
- Publish metadata gaps from CLR-06.

Implementation:

1. List registry server requirements not solved by package artifacts.
2. Split requirements into server MVP, later ecosystem work, and non-goals.
3. Record whether each follow-up belongs in code, documentation, operations,
   release engineering, or product policy.
4. If an issue tracker is available, create issues and link them from the
   readiness record.
5. If an issue tracker is not available, create a local follow-up section in
   `doc/registry-readiness.md`.
6. Ensure every failing or deferred readiness row has a follow-up entry.

Acceptance criteria:

- The backlog covers all failing and deferred readiness rows.
- Server requirements are separated from package artifact requirements.
- Follow-up work does not ask the checker to trust registry metadata.
- Non-goals remain outside the registry MVP.

Verification:

```sh
rg -n "Follow-up|server MVP|later ecosystem work|non-goal|namespace|authenticated publish|immutable" doc/registry-readiness.md
git diff --check
```

### CLR-10-07 Update Roadmap And Contributor-Facing Docs

Dependencies:

- CLR-10-05.
- CLR-10-06.

Implementation:

1. Update `doc/community-library-roadmap.md` with the registry readiness
   decision summary.
2. Update `doc/community-library-roadmap-todo.md` to point to the detailed
   CLR-10 task document and the readiness record.
3. Update README or package docs only if the contributor-facing path changes.
4. Ensure docs distinguish registry server readiness from registry server
   implementation.
5. Ensure docs state whether the next step is server MVP, Git-release seed
   continuation, or deferral.

Acceptance criteria:

- Readers can find the readiness decision from the roadmap.
- Docs do not imply a registry server exists before it does.
- Docs do not imply package imports can rely on implicit latest resolution.
- Docs preserve the certificate-first trusted boundary.

Verification:

```sh
rg -n "community-library-roadmap-clr-10-todo|registry readiness|Git-release-based registry seed|registry server implementation|certificate-first" README.md doc
git diff --check
```

### CLR-10-08 Run Final Readiness Review Gate

Dependencies:

- CLR-10-01 through CLR-10-07.

Implementation:

1. Re-read the readiness record against `doc/community-library-roadmap.md`
   section 4.2.
2. Confirm every blocker row has pass, fail, or deferred status.
3. Confirm no row remains not evaluated.
4. Confirm failing or deferred rows have follow-up entries.
5. Run targeted stale-term and trusted-boundary searches.
6. Run repository-level validation appropriate to the implementation changes.
7. Capture final review findings and fixes in the readiness record.

Acceptance criteria:

- The final review has no unresolved findings.
- The decision record contains enough evidence to justify the selected path.
- Validation passes or skipped validation is explicitly recorded.
- The final diff contains only readiness-review documentation and directly
  required follow-up references unless implementation work was intentionally
  included.

Verification:

```sh
git diff --check
rg -n "not evaluated|placeholder text|fake hash|latest-version import|module name alone" doc/registry-readiness.md
rg -n "Registry 前の blocker|npa.registry.module.v0.1|npa.package.v0.1|downstream_import_bundle|reference-checker-only|verified_high_trust" doc/registry-readiness.md doc/community-library-roadmap.md doc/community-library-roadmap-todo.md
cargo test --workspace
```

## Review Loop

### Pass 1 Findings

Finding: The parent milestone could be misread as permission to start building
a registry server.

Fix: The purpose, scope, decision rules, and task breakdown now state that
CLR-10 is a readiness review and decision gate, not a server implementation.

Finding: The parent acceptance criterion references section 4.2 blockers but
does not say what evidence is required for each blocker.

Fix: The blocker evidence section maps all eight section 4.2 blockers to
required artifacts, pass rules, and fail rules.

Finding: Registry metadata could be treated as checker input once a registry
server exists.

Fix: The trusted-boundary section and individual tasks require registry
metadata, publish metadata, theorem indexes, and CI status to remain untrusted
helper data.

Finding: CLR-08 high-trust evidence might incorrectly block the first registry
readiness decision or be faked from reference-checker-only results.

Fix: CLR-10 records CLR-08 evidence separately, allows a reference-checker-only
decision when CLR-08 is deferred, and forbids approximating
`verified_high_trust`.

Finding: A decision of "continue Git release artifacts" could be treated as a
non-decision.

Fix: The decision rules define Git-release-based registry seed continuation as
a valid outcome with specific conditions and follow-up expectations.

Finding: A registry server backlog could mix package trust requirements with
product and operations requirements.

Fix: The registry server requirements section separates namespace, publishing,
storage, search, moderation, and operations work from proof acceptance.

### Pass 2 Findings

No remaining findings. The milestone now defines the registry readiness
evidence model, blocker-by-blocker pass/fail requirements, trusted-boundary
audit, decision rules, follow-up backlog, roadmap update tasks, and final review
gate without implementing a registry server or expanding the trusted base.
