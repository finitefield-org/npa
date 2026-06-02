# NPA Standalone Repository Activation Procedure

Source plans:

- `develop/registry-readiness.md`
- `develop/npa-mathlib-public-release-plan.md`
- `develop/community-library-roadmap.md`

This procedure activates the public repository split:

```text
npa
  trusted kernel, certificate format, checker, frontend, tactic, package CLI

npa-std
  small stable standard library package

npa-mathlib
  public theorem library package
```

The activation path continues the CLR-10 decision: use Git release artifacts
and hash-pinned package imports. It does not create a registry server.

## Trust Boundary

Activation must not move proof acceptance into GitHub, CI, release metadata, a
registry, or a package resolver.

Trusted proof evidence remains:

- canonical `.npcert` bytes
- Rust kernel / verifier verdict
- source-free reference checker verdict
- deterministic `export_hash`, `certificate_hash`, and `axiom_report_hash`

Untrusted helper data remains:

- `.npa` source
- replay and meta files
- theorem indexes
- publish plans
- CI status
- Git tags and release pages
- registry seed entries
- future registry or API responses

`verified_high_trust` stays unavailable until a consuming repository supplies
CLR-08 pinned external checker binaries, runner policies, checker registry
data, and release audit evidence.

## Activation Order

The repositories must be activated in this order:

```text
SRA-00 Freeze local baseline
SRA-01 Publish npa toolchain reference
SRA-02 Materialize npa-std local package fixture
SRA-03 Activate npa-std standalone repository
SRA-04 Publish npa-std v0.1.0 release artifacts
SRA-05 Re-pin npa-mathlib against the npa-std release
SRA-06 Activate npa-mathlib standalone repository
SRA-07 Publish npa-mathlib v0.1.0 release artifacts
SRA-08 Run downstream source-free import smoke
SRA-09 Record post-activation evidence
```

Do not start `npa-mathlib` standalone activation before `npa-std` has a
published `v0.1.0` release artifact set. `npa-mathlib` currently vendors
`npa-std` certificate bytes from this repository; public activation needs those
bytes to come from the `npa-std` release bundle.

## SRA-00 Freeze Local Baseline

- Status: Completed
- Depends on: current CLR-10 changes
- Inputs:
  - `fixtures/npa-mathlib/`
  - `fixtures/npa-mathlib-downstream/`
  - `develop/registry-readiness.md`
  - `develop/npa-mathlib-public-release-plan.md`
- Deliverables:
  - Local activation baseline commit in this repository.
  - Command transcript showing package gates pass for public Layer 0.
  - Confirmation that public fixtures do not contain stale `Proofs.Ai.*`
    module names, except historical seed docs.
- Acceptance criteria:
  - `fixtures/npa-mathlib/` passes all package artifact checks.
  - `fixtures/npa-mathlib-downstream/` verifies source-free against vendored
    `npa-mathlib` certificate bytes.
  - The current repository fast gate passes.
- Verification:

```sh
cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib --check --json

cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib-downstream --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib-downstream --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib-downstream --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib-downstream --json

cargo test -q -p npa-cli package_import_fixture
./scripts/check-fast.sh
git diff --check
```

Evidence fixed on 2026-06-02:

- `fixtures/npa-mathlib/` passed `check`, `build-certs --check`,
  `verify-certs --checker reference`, `check-hashes`,
  `axiom-report --check`, `index --check`, and `publish-plan --check`.
- `fixtures/npa-mathlib-downstream/` passed `check`,
  `build-certs --check`, `verify-certs --checker reference`, and
  `check-hashes`.
- `cargo test -q -p npa-cli package_import_fixture` passed with five tests.
- `./scripts/check-fast.sh` passed formatting, clippy, and workspace tests
  without `npa-proof-corpus`.
- `git diff --check` passed.
- Stale public fixture name scan found no `Proofs.Ai.*`, `SeedBasic`, or
  `seed_id` hits in package artifacts; the remaining `npa-mathlib-seed` hit is
  historical context in `fixtures/npa-mathlib/README.md`.

## SRA-01 Publish `npa` Toolchain Reference

- Status: Completed
- Depends on: SRA-00
- Inputs:
  - this repository
  - `ci-templates/github-actions/`
  - `crates/npa-cli`
  - `crates/npa-checker-ref`
- Deliverables:
  - A stable Git ref for external theorem libraries, for example `v0.1.0`.
  - External-library setup instructions that install or build the `npa`
    binary from that ref.
  - Copyable PR and release workflow templates for package repositories.
- Acceptance criteria:
  - A fresh checkout of `npa` at the chosen ref can build `npa-cli`.
  - `npa package ...` commands work without registry/network access.
  - `npa-kernel`, `npa-cert`, and `npa-checker-ref` remain independent of
    package registry/network behavior.
- Verification:

```sh
cargo build -p npa-cli
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib --json
python3 ci-templates/github-actions/validate-workflows.py
tmpdir="$(mktemp -d)"
GITHUB_PATH="$tmpdir/github-path" RUNNER_TEMP="$tmpdir" GITHUB_WORKSPACE="$PWD" \
  NPA_BINARY_PATH=target/debug/npa \
  bash ci-templates/github-actions/setup-pinned-npa.sh
./scripts/check-fast.sh
```

Evidence fixed on 2026-06-02:

- Original stable toolchain ref: Git tag `v0.1.0`.
- Current SRA-02-compatible toolchain ref: Git tag `v0.1.1`.
- External repository defaults:
  - `NPA_GIT_TAG = v0.1.1`
  - `RUST_TOOLCHAIN_VERSION = 1.95.0`
- `ci-templates/github-actions/setup-pinned-npa.sh` supports exactly one of
  `NPA_BINARY_PATH`, `NPA_GIT_TAG`, or `NPA_GIT_COMMIT`; `NPA_VERSION` is
  reserved for release-download artifacts and is rejected by the current
  helper.
- Copyable PR, release, and opt-in high-trust templates call the same pinned
  setup helper and remain no-registry/no-hidden-package-cache package gates.
- `npa --version` and `npa version` print the deterministic CLI package
  version for setup evidence.
- `cargo build -p npa-cli` passed.
- `cargo run -p npa-cli -- package check --root fixtures/npa-mathlib --json`
  passed.
- `python3 ci-templates/github-actions/validate-workflows.py` passed.
- Binary-path setup smoke with `NPA_BINARY_PATH=target/debug/npa` passed and
  printed `npa 0.1.1`.
- `v0.1.1` includes the `std-library-legacy-core-builder` package build path
  required by `fixtures/npa-std`.
- `cargo test -q -p npa-cli package_cli_args` passed.
- `./scripts/check-fast.sh` passed.
- `git diff --check` passed.

Recommended release note fields:

- `npa` Git ref
- Rust toolchain version used by CI
- package CLI command list
- reference-checker-only release boundary
- statement that `verified_high_trust` is not emitted from reference-only
  evidence

## SRA-02 Materialize `npa-std` Local Package Fixture

- Status: Completed
- Depends on: SRA-01
- Inputs:
  - `proofs/vendor/npa-std/Std/Logic/Eq/certificate.npcert`
  - `proofs/vendor/npa-std/Std/Nat/Basic/certificate.npcert`
  - `crates/npa-api/src/std_library.rs`
  - `develop/phase6-human.md`
  - `develop/phase6-ai.md`
- Deliverables:
  - `fixtures/npa-std/npa-package.toml`
  - `fixtures/npa-std/Std/Logic/Eq/`
  - `fixtures/npa-std/Std/Nat/Basic/`
  - `fixtures/npa-std/generated/package-lock.json`
  - `fixtures/npa-std/generated/axiom-report.json`
  - `fixtures/npa-std/generated/theorem-index.json`
  - `fixtures/npa-std/generated/publish-plan.json`
- Acceptance criteria:
  - Package name is exactly `npa-std`.
  - Version is `0.1.0`.
  - Local modules are exactly `Std.Logic.Eq` and `Std.Nat.Basic` for the first
    release.
  - `allow_custom_axioms = false` unless the exact kernel-standard
    `Std.Logic.Eq.rec` exception is explicitly required by the current
    certificate representation and documented in the axiom report.
  - The fixture can regenerate or check certificates from a documented source
    provenance. If source files cannot yet be produced, stop here and create a
    focused task to expose the standard-library source/certificate generation
    path before external activation.
- Verification:

```sh
cargo run -q -p npa-cli -- package check --root fixtures/npa-std --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-std --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-std --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-std --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-std --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-std --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-std --check --json
```

Stop condition:

- Do not create a public `npa-std` repository with certificate bytes only and
  no documented source/certificate regeneration story. That would be usable as
  a binary dependency fixture but not as a maintainable standard-library
  repository.

Evidence fixed on 2026-06-02:

- `fixtures/npa-std/` materializes package `npa-std` at version `0.1.0`.
- Local modules are exactly `Std.Logic.Eq` and `Std.Nat.Basic`.
- `allow_custom_axioms = false`; `generated/axiom-report.json` records zero
  direct axioms, zero transitive axioms, and zero policy violations.
- Source provenance is documented in `fixtures/npa-std/README.md`: the
  package manifest fixes module membership and certificate paths; the
  checked-in `source.npa` files are source-package skeletons that fix import
  intent; certificate contents are regenerated by the deterministic Rust
  core-module builder selected with
  `producer_profile = "std-library-legacy-core-builder"`.
- `generated/package-lock.json`, `generated/axiom-report.json`,
  `generated/theorem-index.json`, and `generated/publish-plan.json` were
  materialized. The publish plan hash is
  `sha256:6b513ba2ba97f8ddb46b566ad61510b99a710577029a6ac829552b8a30e4863f`.
- `cargo run -q -p npa-cli -- package check --root fixtures/npa-std --json`
  passed.
- `cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-std --check --json`
  passed.
- `cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-std --checker reference --json`
  passed with `npa-checker-ref` verdicts for both modules.
- `cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-std --json`
  passed.
- `cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-std --check --json`
  passed.
- `cargo run -q -p npa-cli -- package index --root fixtures/npa-std --check --json`
  passed.
- `cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-std --check --json`
  passed.
- `cargo test -q -p npa-cli package_build_certs_check` passed with seven
  tests, including the legacy `npa-std` producer profile fixture.
- `./scripts/check-fast.sh` passed formatting, clippy, and workspace tests
  without `npa-proof-corpus`.
- `git diff --check` passed.

## SRA-03 Activate `npa-std` Standalone Repository

- Status: Completed
- Depends on: SRA-02
- Inputs:
  - `fixtures/npa-std/`
  - `ci-templates/github-actions/npa-package-pr.yml`
  - `ci-templates/github-actions/npa-package-release.yml`
  - `ci-templates/github-actions/summarize-npa-diagnostics.py`
  - setup action from `fixtures/npa-mathlib-seed/.github/actions/setup-npa/`
- Deliverables:
  - New repository root for `npa-std`.
  - Repository README and CONTRIBUTING files.
  - `.github/workflows/npa-package-pr.yml`
  - `.github/workflows/npa-package-release.yml`
  - `.github/actions/setup-npa/action.yml`
- Suggested file layout:

```text
npa-std/
  README.md
  CONTRIBUTING.md
  npa-package.toml
  Std/
    Logic/Eq/
    Nat/Basic/
  generated/
    package-lock.json
    axiom-report.json
    theorem-index.json
    publish-plan.json
  .github/
    actions/setup-npa/action.yml
    workflows/npa-package-pr.yml
    workflows/npa-package-release.yml
```

- Acceptance criteria:
  - The standalone repo passes the package PR gate from a fresh checkout.
  - Workflows use explicit package root `.`.
  - Workflows install/build `npa` from the SRA-01 toolchain ref.
  - No workflow uses registry lookup, latest-version resolution, or network
    package fetching for proof acceptance.
- Verification in the standalone repository:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Evidence fixed on 2026-06-02:

- Standalone repository: `finitefield-org/npa-std`
  (`https://github.com/finitefield-org/npa-std`).
- Repository default branch: `main`.
- Activation commit pushed to `main`:
  `849e8eed057e4fcf42799962245db142d50eb79a`.
- Repository visibility is public as of the 2026-06-02 PUB-11 evidence.
- Repository variables fixed:
  - `NPA_GIT_TAG = v0.1.1`
  - `RUST_TOOLCHAIN_VERSION = 1.95.0`
  - `NPA_ENABLE_PUBLISH_PLAN = true`
- Standalone layout materialized:
  - `README.md`
  - `CONTRIBUTING.md`
  - `npa-package.toml`
  - `Std/Logic/Eq/source.npa`
  - `Std/Logic/Eq/certificate.npcert`
  - `Std/Nat/Basic/source.npa`
  - `Std/Nat/Basic/certificate.npcert`
  - `generated/package-lock.json`
  - `generated/axiom-report.json`
  - `generated/theorem-index.json`
  - `generated/publish-plan.json`
  - `.github/actions/setup-npa/action.yml`
  - `.github/actions/setup-npa/setup-pinned-npa.sh`
  - `.github/scripts/summarize-npa-diagnostics.py`
  - `.github/workflows/npa-package-pr.yml`
  - `.github/workflows/npa-package-release.yml`
- Workflows use explicit package root `.` and do not perform registry lookup,
  latest-version resolution, or network package fetching for proof acceptance.
- Workflow/action YAML parsed with the local Ruby YAML parser.
- Local standalone checkout passed:
  - `npa package check --root . --json`
  - `npa package build-certs --root . --check --json`
  - `npa package verify-certs --root . --checker reference --json`
  - `npa package check-hashes --root . --json`
  - `npa package axiom-report --root . --check --json`
  - `npa package index --root . --check --json`
  - `npa package publish-plan --root . --check --json`
- Fresh checkout from GitHub at
  `849e8eed057e4fcf42799962245db142d50eb79a` passed the same package gate.
- Copied setup helper fetched `finitefield-org/npa` tag `v0.1.1`, built
  `npa-cli` with Rust `1.95.0`, and printed `npa 0.1.1`.
- `git diff --check` passed in the standalone repository.

## SRA-04 Publish `npa-std` v0.1.0 Release Artifacts

- Status: Completed
- Depends on: SRA-03
- Inputs:
  - activated `npa-std` repository
- Deliverables:
  - Git tag `v0.1.0`.
  - Release artifact bundle containing:
    - `npa-package.toml`
    - `Std/**/certificate.npcert`
    - `generated/package-lock.json`
    - `generated/axiom-report.json`
    - `generated/theorem-index.json`
    - `generated/publish-plan.json`
  - Command transcript for the release gate.
- Acceptance criteria:
  - `generated/publish-plan.json` has package `npa-std`, version `0.1.0`,
    checksum-only signature policy, and downstream import bundle entries for
    `Std.Logic.Eq` and `Std.Nat.Basic`.
  - Release artifacts are immutable once the tag is published. Any change uses
    a new version.
  - The release page states that this is reference-checker-only evidence.
- Verification:

```sh
npa package publish-plan --root . --check --json
npa package verify-certs --root . --checker reference --json
```

Evidence fixed on 2026-06-02:

- `finitefield-org/npa` visibility is public:
  `{"isPrivate":false,"nameWithOwner":"finitefield-org/npa","url":"https://github.com/finitefield-org/npa","visibility":"PUBLIC"}`.
- Public `npa` toolchain fetch smoke passed:
  `git ls-remote https://github.com/finitefield-org/npa.git refs/tags/v0.1.1`
  returned `8c405babb29df985b43c69fe6c857646f11cb8b7 refs/tags/v0.1.1`.
- `npa-std` release workflow rerun passed:
  `https://github.com/finitefield-org/npa-std/actions/runs/26806975884`.
- Run metadata: `attempt=2`, `status=completed`, `conclusion=success`,
  `event=push`, `headSha=849e8eed057e4fcf42799962245db142d50eb79a`.
- Successful workflow jobs:
  - `Package artifact checks`
  - `Fast-kernel source-free verification`
  - `Reference checker source-free verification`
- Release URL:
  `https://github.com/finitefield-org/npa-std/releases/tag/v0.1.0`.
- Release assets:
  - `npa-std-v0.1.0-release-artifacts.tar.gz`
    SHA-256 `3ed967d1870f97f7042e87a75efebd3cf553e8c86d8959c720080115a78fe85c`
  - `npa-std-v0.1.0-release-artifacts.tar.gz.sha256`
    SHA-256 `332ca2f07521b5b92f92aa0d8153f2156dcbbf5a98b8c79685c78e139f544ea4`
  - `npa-std-v0.1.0-release-gate.txt`
    SHA-256 `a732871792193515b5b69cc06034f5b5633099c380f6792af78472e38ba08457`
- Release tarball contents:
  - `npa-package.toml`
  - `Std/Logic/Eq/certificate.npcert`
  - `Std/Nat/Basic/certificate.npcert`
  - `generated/package-lock.json`
  - `generated/axiom-report.json`
  - `generated/theorem-index.json`
  - `generated/publish-plan.json`
- Release gate file records the package gate, reference checker verification,
  and fast-kernel verification commands as passed on 2026-06-02.

## SRA-05 Re-Pin `npa-mathlib` Against The `npa-std` Release

- Status: Pending
- Depends on: SRA-04
- Inputs:
  - `fixtures/npa-mathlib/`
  - `npa-std` `v0.1.0` release bundle
- Deliverables:
  - `fixtures/npa-mathlib/vendor/npa-std/**/certificate.npcert` copied from
    the `npa-std` release bundle, not from ad hoc local fixture bytes.
  - `fixtures/npa-mathlib/npa-package.toml` import pins checked against the
    `npa-std` publish plan downstream import bundle.
  - Regenerated `fixtures/npa-mathlib/generated/package-lock.json`.
  - Updated downstream test evidence if any hash changes.
- Acceptance criteria:
  - `npa-mathlib` imports `Std.Logic.Eq` and `Std.Nat.Basic` by package,
    version, module, export hash, certificate hash, and certificate artifact
    path.
  - `npa-mathlib` verification succeeds source-free with only vendored
    `npa-std` certificate artifacts.
  - No `npa-mathlib` command needs the `npa-std` source tree.
- Verification:

```sh
cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib --check --json
```

## SRA-06 Activate `npa-mathlib` Standalone Repository

- Status: Pending
- Depends on: SRA-05
- Inputs:
  - `fixtures/npa-mathlib/`
  - `fixtures/npa-mathlib-downstream/`
  - `ci-templates/github-actions/`
  - `npa-std` `v0.1.0` release bundle
- Deliverables:
  - New repository root for `npa-mathlib`.
  - README and CONTRIBUTING files.
  - `.github/workflows/npa-package-pr.yml`
  - `.github/workflows/npa-package-release.yml`
  - Optional `fixtures/downstream-smoke/` copied from
    `fixtures/npa-mathlib-downstream/`.
- Suggested file layout:

```text
npa-mathlib/
  README.md
  CONTRIBUTING.md
  npa-package.toml
  Mathlib/
    Logic/Basic/
    Logic/Prop/
    Logic/Eq/
    Data/Nat/Basic/
    Core/Reduction/
  vendor/
    npa-std/
      Std/Logic/Eq/certificate.npcert
      Std/Nat/Basic/certificate.npcert
  generated/
    package-lock.json
    axiom-report.json
    theorem-index.json
    publish-plan.json
  fixtures/
    downstream-smoke/
  .github/
    actions/setup-npa/action.yml
    workflows/npa-package-pr.yml
    workflows/npa-package-release.yml
```

- Acceptance criteria:
  - The standalone repo has package `npa-mathlib`, version `0.1.0`.
  - Local modules use only the public `Mathlib.*` namespace.
  - Historical `Proofs.Ai.*` seed names do not appear in package source,
    manifest, lock, publish plan, or downstream smoke fixture.
  - The downstream smoke fixture imports `Mathlib.Logic.Basic` from
    `npa-mathlib` source-free.
- Verification in the standalone repository:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
rg -n "Proofs\\.Ai|npa-mathlib-seed" npa-package.toml Mathlib generated fixtures/downstream-smoke
```

The `rg` command should return no hits.

## SRA-07 Publish `npa-mathlib` v0.1.0 Release Artifacts

- Status: Pending
- Depends on: SRA-06
- Inputs:
  - activated `npa-mathlib` repository
- Deliverables:
  - Git tag `v0.1.0`.
  - Release artifact bundle containing:
    - `npa-package.toml`
    - `Mathlib/**/certificate.npcert`
    - vendored `npa-std` certificate artifacts
    - `generated/package-lock.json`
    - `generated/axiom-report.json`
    - `generated/theorem-index.json`
    - `generated/publish-plan.json`
  - Release notes listing all local modules and the `npa-std` release ref.
- Acceptance criteria:
  - `generated/publish-plan.json` has package `npa-mathlib`, version `0.1.0`,
    checksum-only signature policy, five module registry seed entries, and five
    downstream import bundle modules.
  - Axiom report records zero direct axioms, zero transitive axioms, and zero
    policy violations, unless a reviewed standard exception is explicitly
    recorded.
  - Release notes state that source, replay, theorem index, publish metadata,
    and CI status are not proof evidence.
- Verification:

```sh
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package publish-plan --root . --check --json
```

## SRA-08 Run Downstream Source-Free Import Smoke

- Status: Pending
- Depends on: SRA-07
- Inputs:
  - `npa-mathlib` `v0.1.0` release bundle
  - `fixtures/npa-mathlib-downstream/`
- Deliverables:
  - A downstream package fixture or tiny standalone repository that vendors
    only `npa-mathlib` certificate bytes needed for import.
  - Command transcript proving source-free verification.
  - Negative checks for corrupted package name, version, export hash,
    certificate hash, and certificate artifact hash.
- Acceptance criteria:
  - The downstream package does not vendor `npa-mathlib` source, replay, meta,
    theorem index, registry state, or package source tree.
  - The downstream package verifies `Mathlib.Logic.Basic` and its own local
    theorem with `npa-checker-ref`.
  - Corrupted import identity or hash pins are rejected before proof acceptance.
- Verification:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
```

## SRA-09 Record Post-Activation Evidence

- Status: Pending
- Depends on: SRA-08
- Inputs:
  - `npa` toolchain ref
  - `npa-std` `v0.1.0` release URL or local artifact path
  - `npa-mathlib` `v0.1.0` release URL or local artifact path
  - downstream smoke transcript
- Deliverables:
  - Update `develop/registry-readiness.md` or a successor release evidence record.
  - Update `develop/npa-mathlib-public-release-plan.md` with activation status.
  - Add follow-up tasks for Layer 1 theorem expansion.
- Acceptance criteria:
  - Evidence identifies exact Git refs, package versions, artifact paths, and
    command results.
  - Remaining gaps are explicit: registry server, dependency solver, signing,
    binary cache, and CLR-08 high-trust evidence.
  - The next theorem-layer task can start without revisiting repository split
    or package manifest semantics.
- Verification:

```sh
rg -n "npa-std|npa-mathlib|v0.1.0|Git-release-based registry seed|verified_high_trust" doc
git diff --check
```

## Release Artifact Rule

For `npa-std` and `npa-mathlib`, a release artifact bundle is acceptable only
when it contains all proof-relevant certificate bytes and deterministic package
metadata required by downstream import. A release bundle must not require a
registry server, latest-version lookup, hidden local cache, or source checkout
for proof acceptance.

Required release files:

```text
npa-package.toml
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
generated/publish-plan.json
Std/**/certificate.npcert for npa-std releases
Mathlib/**/certificate.npcert for npa-mathlib releases
vendor/npa-std/**/certificate.npcert for npa-mathlib direct imports
```

Optional release files:

```text
source.npa
replay.json
meta.json
README.md
CONTRIBUTING.md
command transcripts
```

Optional files are useful for authors and reviewers, but they are not proof
evidence.

## Stop Conditions

Stop activation and fix the local plan if any of these occur:

- `npa-std` cannot document how `Std.*` source or certificates are generated.
- A package import is accepted by module name alone.
- Any package command needs registry network access for proof acceptance.
- A downstream fixture needs source, replay, meta, theorem index, or registry
  metadata to verify.
- Public `npa-mathlib` artifacts still contain `Proofs.Ai.*` module names.
- `verified_high_trust` is generated from reference-checker-only evidence.
- `npa-mathlib` adds Layer 1 modules before Layer 0 standalone activation
  evidence is recorded.
