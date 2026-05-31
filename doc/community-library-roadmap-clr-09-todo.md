# Community Library Roadmap CLR-09 Todo

Source milestone: `CLR-09 Dogfood npa-mathlib-seed` in
`doc/community-library-roadmap-todo.md`.

## Purpose

CLR-09 turns the package, CI, and release metadata work from CLR-04 through
CLR-07 into a concrete external theorem library dogfood. The target is a small
`npa-mathlib-seed` package that can be checked from a fresh checkout, published
as ordinary release artifacts, and imported by another package without a
registry server.

The important outcome is not the size of the library. The important outcome is
that theorem-only changes can happen outside the `npa` repository while the
trusted boundary remains:

```text
trusted:
  canonical certificate
  small Rust kernel
  source-free checker verdict

not trusted:
  seed repository CI
  package metadata
  publish-plan metadata
  theorem index
  replay files
  contributor workflow docs
```

## Scope

CLR-09 includes:

- A separate `npa-mathlib-seed` repository, or a local fixture that deliberately
  models that external repository boundary.
- A minimal, closed seed module set copied from the current proof corpus.
- A seed `npa-package.toml`.
- Checked-in source, certificate, replay, and meta artifacts where those files
  are still part of the package contract from CLR-04 and CLR-05.
- Generated package lock, axiom report, theorem index, and publish-plan
  artifacts.
- CLR-07 GitHub Actions templates applied to the seed repository.
- An `npa` integration fixture that imports the seed release artifacts using
  hash-pinned data from the CLR-06 downstream import bundle.
- Contributor documentation for theorem-only pull requests.
- A registry handoff audit showing that the seed release artifacts are usable
  before a registry service exists.

CLR-09 excludes:

- Moving the whole current proof corpus out of this repository.
- A registry server, package solver, network fetcher, binary cache, or implicit
  latest-version resolution.
- Production theorem search, LLM/RAG workflows, web IDE features, and browser
  contribution flows.
- Any change that expands the trusted base of the kernel, certificate format, or
  checker.
- External-checker release evidence when CLR-08 is deferred.
- `verified_high_trust` artifacts unless CLR-08 is already complete and the
  seed release policy explicitly enables them.

## Current Repository Facts

The current repository already contains a proof corpus under `proofs/` and a
hard-coded corpus tool under `tools/proof-corpus/`. The current manifest uses
the schema `npa-ai-proof-corpus-v0.1`, so CLR-09 must treat those modules as a
migration source rather than as the final external package layout.

Observed small modules that can seed the first package are:

- `Proofs.Ai.Basic`, with no imports and no axioms.
- `Proofs.Ai.Eq`, importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs.Ai.Nat`, importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs.Ai.Prop`, with no imports and no axioms.
- `Proofs.Ai.Reduction`, importing `Std.Nat.Basic`.

`Proofs.Ai.EqReasoning` is a possible extension only if the package axiom
policy intentionally allows its `Eq.rec` dependency. Algebra, geometry,
analysis, and larger generated modules stay out of the first seed because they
would test corpus migration volume rather than package ergonomics.

At the time this task list was written, `crates/npa-package`,
`crates/npa-cli`, `ci-templates/`, and `doc/external-theorem-library-ci.md`
were not present in the repository. CLR-09 therefore depends on the
implementation outputs from CLR-04, CLR-05, CLR-06, and CLR-07 instead of
assuming those files already exist.

## Implementation Contract

### Repository Boundary

The preferred implementation target is a separate repository named
`npa-mathlib-seed`. If the first implementation must stay inside this checkout,
use a fixture path such as `fixtures/npa-mathlib-seed/` and make it behave like
an external repository:

- The fixture has its own `npa-package.toml`.
- The fixture has its own package root, generated artifact directory, README,
  and contributor guide.
- The fixture does not rely on hidden relative paths into this repository.
- The fixture does not install active workflows under this repository's
  `.github/workflows/`.
- The fixture can be copied to a standalone repository and still run the same
  package commands from a fresh checkout.

The fixture is allowed only as an implementation convenience. The documented
product boundary remains a separate theorem library repository.

### Initial Module Set

Use a closed subset that minimizes churn:

```text
Proofs.Ai.Basic
Proofs.Ai.Prop
Proofs.Ai.Eq
Proofs.Ai.Nat
Proofs.Ai.Reduction
```

Preserve the current module names for the first dogfood unless a manifest-level
module rename is already implemented. Renaming to a public namespace can be a
later migration after certificates, imports, theorem indexes, and downstream
hashes are stable.

The module scope decision must be recorded in the seed repository README and in
the `npa` integration fixture. The record must state:

- why the subset is closed;
- which standard-library artifacts are imported;
- which axioms are permitted;
- why larger proof corpus modules are intentionally deferred.

### Seed Package Layout

The seed repository should have this logical layout:

```text
npa-mathlib-seed/
  npa-package.toml
  README.md
  CONTRIBUTING.md
  Proofs/Ai/Basic/source.npa
  Proofs/Ai/Basic/certificate.npcert
  Proofs/Ai/Basic/replay.json
  Proofs/Ai/Basic/meta.json
  Proofs/Ai/Prop/source.npa
  Proofs/Ai/Prop/certificate.npcert
  Proofs/Ai/Prop/replay.json
  Proofs/Ai/Prop/meta.json
  Proofs/Ai/Eq/source.npa
  Proofs/Ai/Eq/certificate.npcert
  Proofs/Ai/Eq/replay.json
  Proofs/Ai/Eq/meta.json
  Proofs/Ai/Nat/source.npa
  Proofs/Ai/Nat/certificate.npcert
  Proofs/Ai/Nat/replay.json
  Proofs/Ai/Nat/meta.json
  Proofs/Ai/Reduction/source.npa
  Proofs/Ai/Reduction/certificate.npcert
  Proofs/Ai/Reduction/replay.json
  Proofs/Ai/Reduction/meta.json
  vendor/npa-std/
  generated/package-lock.json
  generated/axiom-report.json
  generated/theorem-index.json
  generated/publish-plan.json
  .github/workflows/npa-package-pr.yml
  .github/workflows/npa-package-release.yml
```

The `.github/workflows/` files belong in the seed repository. If the seed is
represented as a fixture inside `npa`, store workflow templates as inert fixture
files and do not make them active for this repository.

### Package Manifest Contract

The seed `npa-package.toml` must declare:

- package name `npa-mathlib-seed`;
- package version for the release under test;
- package schema version owned by CLR-04;
- module roots and generated artifact paths owned by CLR-04 and CLR-05;
- dependencies on the standard-library package artifacts needed by the chosen
  module set;
- allowed axiom policy for the chosen modules;
- release artifact paths that CLR-06 publish-plan consumes.

The manifest must not declare registry network endpoints, latest-version
selectors, or source-repository-specific absolute paths.

### Package Command Contract

The seed CI and local contributor flow must use the package commands from
CLR-04 through CLR-07:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Release CI may also run the fast checker path if CLR-05 exposes it:

```sh
npa package verify-certs --root . --checker fast --json
```

Do not add changed-file selection, full-library selection flags, registry
network flags, or external-checker flags to CLR-09. If CLR-08 is complete, the
external-checker release profile is an optional extension and must remain
separate from the base CLR-09 acceptance criteria.

### Generated Artifact Contract

CLR-09 must produce and check these generated artifacts:

- `generated/package-lock.json`
- `generated/axiom-report.json`
- `generated/theorem-index.json`
- `generated/publish-plan.json`

`generated/publish-plan.json` must include the CLR-06 downstream import bundle.
The bundle is metadata for downstream consumers, not proof evidence. The
minimum fields consumed by the downstream fixture are:

- package name;
- package version;
- module name;
- exported declaration identifiers;
- export hash;
- certificate hash;
- certificate artifact path;
- artifact file hash;
- source-free checker result summary.

The downstream fixture must verify the hashes before accepting the import. A
mismatch in export hash, certificate hash, artifact file hash, package name, or
package version must reject the import.

### `npa` Integration Fixture Contract

The `npa` repository needs one fixture that consumes a released seed artifact
without depending on a registry server. The fixture should model a downstream
package with:

- its own `npa-package.toml`;
- a dependency entry pointing at a local release artifact directory or archive;
- hash-pinned module imports from the seed downstream import bundle;
- at least one local theorem or certificate that imports a seed module;
- positive verification for the valid seed release;
- negative tests for corrupted seed hash metadata.

The fixture must not read seed source, replay files, meta files, theorem index,
or registry state as proof evidence. It may read publish-plan metadata to locate
artifacts and expected hashes, then must verify certificate artifacts directly.

The fixture must also show that a theorem-only update to the seed package does
not require changes to:

- kernel type checking;
- certificate canonicalization;
- source-free checker logic;
- package import trust rules.

### Contributor Workflow Contract

The seed README and contributor guide must describe a theorem-only pull request
as this sequence:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Review guidance must focus on:

- theorem statement clarity;
- module naming and dependency direction;
- whether the axiom report changed;
- whether generated hashes changed only because proof artifacts changed;
- whether downstream import compatibility is intentionally changed;
- whether release artifacts remain source-free checkable.

The guide must state that tactics, automation, replay files, and AI-generated
suggestions may help produce certificates but are not trusted proof evidence.

### Release And Registry Handoff

CLR-09 release artifacts must be sufficient for a downstream package to import
the seed library before any registry server exists. The handoff to CLR-10 is:

- release artifact archive or directory;
- `generated/publish-plan.json`;
- downstream import bundle from the publish plan;
- axiom report;
- theorem index;
- source-free checker result summary;
- contributor workflow documentation;
- documented gaps discovered while dogfooding the seed.

The handoff must explicitly say that package metadata and registry seed entries
are discoverability metadata. They are not trusted proof evidence.

### CLR-08 Deferral Rule

CLR-09 can proceed while CLR-08 is deferred if the seed release policy is
reference-checker-only. In that mode:

- no `verified_high_trust` artifact is generated;
- no external checker is required in seed PR CI;
- no external checker is required in seed release CI;
- release notes state that high-trust external verification is deferred;
- downstream import acceptance relies on certificate hashes and reference
  checker results.

If CLR-08 is complete before CLR-09 starts, add an optional release profile for
external verification, but keep the reference-checker-only path as the base
acceptance test.

## Task Breakdown

### CLR-09-01 Decide Seed Repository Boundary And Module Scope

Dependencies:

- CLR-04 package manifest schema.
- CLR-05 generated artifact layout.
- Current `proofs/manifest.toml`.
- Current proof corpus artifact paths.

Implementation:

1. Choose the concrete seed location: a separate `npa-mathlib-seed`
   repository, or an inert local fixture that can be copied to a separate
   repository.
2. Record the boundary decision in the seed README or fixture README.
3. Select the initial module set from `Proofs.Ai.Basic`, `Proofs.Ai.Prop`,
   `Proofs.Ai.Eq`, `Proofs.Ai.Nat`, and `Proofs.Ai.Reduction`.
4. Verify the selected module set is closed over current proof corpus imports
   plus standard-library package dependencies.
5. Record the axiom policy for the selected modules.
6. Exclude larger proof corpus modules from the first seed and explain why.

Acceptance criteria:

- The seed boundary is explicit and does not depend on hidden paths in the
  `npa` checkout.
- The selected module set is small and closed.
- The seed scope document lists imports, allowed axioms, and deferred modules.
- No kernel, checker, or certificate-format change is required for the scope
  decision.

Verification:

```sh
rg -n "npa-mathlib-seed|Proofs.Ai.Basic|Proofs.Ai.Prop|Proofs.Ai.Eq|Proofs.Ai.Nat|Proofs.Ai.Reduction" README.md doc fixtures
rg -n "Eq.rec|allowed axiom|deferred" README.md doc fixtures
```

### CLR-09-02 Materialize Seed Package Layout

Dependencies:

- CLR-09-01 module scope.
- CLR-04 package root loader.
- CLR-05 artifact path conventions.

Implementation:

1. Create the seed package root.
2. Add `npa-package.toml` with package identity, schema version, module roots,
   generated artifact paths, dependency entries, and axiom policy.
3. Copy or generate the selected source, certificate, replay, and meta artifacts
   into the seed layout.
4. Add standard-library dependency artifacts under a vendor or fixture release
   path that the package loader can resolve from a fresh checkout.
5. Add README and CONTRIBUTING files that describe the package boundary and
   contribution flow.
6. Ensure the seed package can be copied away from the `npa` repository without
   breaking relative paths.

Acceptance criteria:

- `npa-package.toml` validates with the CLR-04 package loader.
- Source, certificates, replay, and meta files exist for every selected module.
- Standard-library dependencies are declared rather than implicitly loaded from
  the `npa` working tree.
- The seed layout contains no absolute local filesystem paths.

Verification:

```sh
npa package check --root . --json
rg -n "/Users/|/tmp/|target/debug|target/release" npa-package.toml README.md CONTRIBUTING.md Proofs generated vendor
```

### CLR-09-03 Wire Fresh-Checkout Package Commands

Dependencies:

- CLR-04 package commands.
- CLR-05 source-free verification and generated artifact commands.
- CLR-06 publish-plan command.

Implementation:

1. Run the base package command sequence from the seed package root.
2. Fix manifest paths, dependency entries, and generated artifact paths until
   check mode passes from a clean checkout.
3. Ensure `build-certs --check` reports drift instead of silently rewriting
   certificate artifacts.
4. Ensure `check-hashes` validates source, certificate, generated artifact, and
   import hashes.
5. Ensure `verify-certs --checker reference` runs without reading source,
   replay, meta, theorem index, or registry metadata as proof evidence.
6. Ensure `axiom-report --check`, `index --check`, and `publish-plan --check`
   fail when their generated files are stale.

Acceptance criteria:

- A fresh checkout of the seed package passes the full base command sequence.
- Stale generated artifacts are detected in check mode.
- Source-free reference verification succeeds for all selected modules.
- The command sequence does not use unsupported package flags or registry
  network access.

Verification:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

### CLR-09-04 Add Seed CI From CLR-07 Templates

Dependencies:

- CLR-07 CI templates.
- CLR-09-03 passing local command sequence.

Implementation:

1. Copy the CLR-07 PR template into the seed repository workflow path.
2. Copy the CLR-07 release template into the seed repository workflow path.
3. Configure workflow inputs so the seed repository can install or locate the
   `npa` CLI deterministically.
4. Configure PR CI to run the base reference-checker command sequence.
5. Configure release CI to run the base sequence plus the optional fast checker
   command if that checker is available.
6. Keep high-trust external verification disabled unless CLR-08 is complete and
   the seed release policy explicitly enables it.
7. Add CI documentation explaining which artifacts are uploaded or checked.

Acceptance criteria:

- The seed PR workflow passes from a fresh checkout.
- The seed release workflow produces the release artifacts needed by
  downstream packages.
- The workflow files are active only in the seed repository.
- The workflow does not introduce registry network lookup or implicit latest
  dependency selection.

Verification:

```sh
npa package check --root . --json
npa package verify-certs --root . --checker reference --json
rg -n "publish-plan|axiom-report|theorem-index|reference-checker-only" README.md CONTRIBUTING.md .github
```

### CLR-09-05 Generate Release Artifacts And Publish Plan

Dependencies:

- CLR-06 publish-plan schemas.
- CLR-09-03 generated artifact checks.
- CLR-09-04 release CI.

Implementation:

1. Generate or refresh `generated/package-lock.json`.
2. Generate or refresh `generated/axiom-report.json`.
3. Generate or refresh `generated/theorem-index.json`.
4. Generate or refresh `generated/publish-plan.json`.
5. Ensure the publish plan includes downstream import bundle entries for every
   exported seed module.
6. Ensure release metadata references certificate artifacts by file hash and
   certificate hash.
7. Ensure source-free checker result summaries are present in release metadata.
8. Check that publish metadata does not claim external-checker or high-trust
   evidence when CLR-08 is deferred.

Acceptance criteria:

- Generated artifacts are deterministic.
- Publish-plan check mode passes after artifacts are checked in.
- Downstream import bundle entries are complete for all exported seed modules.
- The release artifact set can be archived and consumed without a registry
  service.

Verification:

```sh
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
rg -n "downstream_import_bundle|reference-checker-only|verified_high_trust" generated README.md
```

### CLR-09-06 Add NPA Downstream Import Fixture

Dependencies:

- CLR-06 downstream import bundle.
- CLR-09-05 release artifact set.
- `npa` package import code from CLR-04 through CLR-06.

Implementation:

1. Add a downstream package fixture inside the `npa` repository.
2. Point the fixture dependency at the seed release artifact directory or
   archive.
3. Import at least one seed module using hash-pinned data from
   `generated/publish-plan.json`.
4. Add a positive test that checks the downstream package with valid seed
   artifacts.
5. Add negative tests that corrupt export hash, certificate hash, artifact file
   hash, package name, and package version.
6. Ensure the fixture never reads seed source, replay, meta, theorem index, or
   registry state as trusted proof evidence.
7. Ensure the tests do not modify kernel, checker, or certificate code when the
   seed theorem set changes.

Acceptance criteria:

- The downstream fixture accepts valid seed release artifacts.
- The downstream fixture rejects corrupted hash metadata.
- The fixture runs without a registry server.
- The fixture documents which seed release artifact is being consumed.

Verification:

```sh
cargo test -p npa-package downstream_import_bundle
cargo test -p npa-cli package_import_fixture
rg -n "downstream_import_bundle|npa-mathlib-seed|source-free|registry" crates fixtures doc
```

### CLR-09-07 Document Contributor Workflow And Review Policy

Dependencies:

- CLR-09-03 command sequence.
- CLR-09-04 CI behavior.
- CLR-09-05 release artifacts.

Implementation:

1. Document how to add or update a theorem in the seed repository.
2. Document when certificates and generated artifacts change.
3. Document review expectations for theorem statements, names, imports, axiom
   changes, and downstream compatibility.
4. Document that tactics, replay files, automation, and AI output are not
   trusted proof evidence.
5. Document the reference-checker-only release policy when CLR-08 is deferred.
6. Add a short handoff section explaining what CLR-10 needs from the seed
   release.

Acceptance criteria:

- A new contributor can run the local command sequence without knowing the
  internals of the `npa` repository.
- The review policy makes axiom report changes visible.
- The docs do not imply that package metadata, theorem indexes, replay files,
  or CI are trusted proof evidence.
- The docs do not imply a registry service already exists.

Verification:

```sh
rg -n "theorem-only|axiom report|not trusted proof evidence|reference-checker-only|registry" README.md CONTRIBUTING.md doc
```

### CLR-09-08 Run Dogfood Review And Registry-Handoff Audit

Dependencies:

- CLR-09-01 through CLR-09-07.
- CLR-10 registry readiness inputs.

Implementation:

1. Review the seed package from the perspective of a new contributor.
2. Review the seed release from the perspective of a downstream package.
3. Confirm that theorem-only seed changes do not require trusted-base changes in
   `npa`.
4. Confirm that all downstream imports are hash-pinned.
5. Confirm that release artifacts are sufficient without a registry server.
6. Capture gaps that should move to CLR-10 or later milestones.
7. Run repository-level validation after the integration fixture is added.

Acceptance criteria:

- Dogfood review findings are fixed or explicitly moved to later milestones.
- CLR-10 has concrete inputs: seed release artifact set, publish plan, import
  bundle, axiom report, theorem index, and contributor workflow.
- The working tree contains no accidental active CI for the seed fixture inside
  the `npa` repository.
- `npa` workspace tests pass after adding the integration fixture.

Verification:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
rg -n "npa-mathlib-seed|publish-plan|downstream_import_bundle|reference-checker-only|verified_high_trust" README.md doc crates fixtures
```

## Review Loop

### Pass 1 Findings

Finding: The parent milestone allowed either a new external repository or a
local fixture, which could let the implementation accidentally depend on hidden
paths in the `npa` checkout.

Fix: The repository boundary section now requires the fixture to behave like a
standalone repository, forbids hidden relative paths, and keeps active workflow
files out of this repository.

Finding: CLR-08 can be deferred, but the seed release still needs a clear
verification posture.

Fix: The CLR-08 deferral rule requires a reference-checker-only seed release,
forbids `verified_high_trust` output in that mode, and treats external
verification as an optional extension only after CLR-08 is complete.

Finding: The current proof corpus is much larger than the first seed should
carry.

Fix: The initial module set is limited to five small modules and requires a
documented deferral for larger corpus areas.

Finding: Publish-plan metadata could be mistaken for trusted proof evidence.

Fix: The trusted boundary and generated artifact sections state that publish
metadata, indexes, replay files, and CI are not proof evidence. The downstream
fixture verifies certificate artifacts and hashes directly.

Finding: A downstream fixture could accidentally read seed source or replay data
as part of proof acceptance.

Fix: The integration fixture contract explicitly restricts trusted inputs to
hash-pinned release metadata and certificate artifacts, and requires negative
tests for corrupted hashes.

### Pass 2 Findings

No remaining findings. The milestone now has a seed repository boundary,
minimal module scope, package layout, CI command contract, release artifact
contract, downstream import fixture contract, contributor workflow, and
registry handoff audit path.
