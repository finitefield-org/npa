# Public Repository Readiness Todo

Source request: prepare `finitefield-org/npa` to become public by making
user-facing documentation English by default while keeping development/internal
documentation in Japanese where appropriate.

## Purpose

`finitefield-org/npa` must be publicly fetchable before external theorem
package repositories can build the pinned toolchain ref, for example:

```text
NPA_GIT_TAG = v0.1.1
RUST_TOOLCHAIN_VERSION = 1.95.0
```

This task list prepares the `npa` repository for public visibility. It is a
documentation and repository-presentation milestone, not a change to proof
acceptance semantics.

The immediate downstream blocker is `npa-std` SRA-04: the `npa-std v0.1.0`
release workflow cannot fetch `finitefield-org/npa` from GitHub Actions while
`npa` remains private.

## Scope

対象:

```text
- public user が最初に読む repository root の英語化
- external package author が読む CI / package verification docs の英語化
- internal Japanese development docs と public-facing docs の分類
- public 化前の stale/private 前提チェック
- public 化後の toolchain fetch smoke と npa-std SRA-04 rerun
```

非対象:

```text
- kernel / certificate / checker semantics の変更
- package resolver / registry server の実装
- theorem library の追加
- `npa-std` package artifact bytes の変更
- すべての内部 phase docs の完全英訳
```

## Trusted Boundary

This milestone must not change the trusted proof boundary:

```text
trusted:
  canonical .npcert bytes
  Rust kernel / verifier verdict
  source-free reference checker verdict
  deterministic export_hash, certificate_hash, and axiom_report_hash

not trusted:
  README / docs
  CI status
  GitHub release pages
  package publish metadata
  theorem indexes
  source files, replay files, tactic traces, AI traces
```

Documentation may explain evidence, but it must not make GitHub metadata,
registry metadata, source scripts, or CI status part of proof acceptance.

## Current Facts

- `README.md` is currently Japanese-heavy and mixes public introduction,
  internal implementation status, package CLI details, and local development
  notes.
- `finitefield-org/npa-std` is public.
- `finitefield-org/npa` is still private as of this todo.
- `npa` toolchain ref `v0.1.1` exists and is the current SRA-02-compatible
  external package pin.
- `npa-std v0.1.0` tag and release assets exist, but the GitHub release
  workflow failed before package checks because the runner could not fetch the
  private `finitefield-org/npa` repository.

## Milestones

### PUB-00 Classify Public And Internal Documentation

- Status: Completed
- Deliverables:
  - A short classification table in `doc/README.md` or `doc/index.md`.
  - Public-facing docs list.
  - Internal/development docs list.
- Public-facing docs should include:
  - `README.md`
  - `CONTRIBUTING.md`
  - `LICENSE`
  - `doc/README.md` or `doc/index.md`
  - `doc/npa-toolchain-reference-v0.1.1.md`
  - `doc/external-theorem-library-ci.md`
  - `ci-templates/github-actions/README.md`
  - `checkers/npa-checker-ext/README.md` if external checker users are in
    scope for the first public release.
- Internal/development docs may remain Japanese:
  - `doc/phase*.md`
  - `doc/*-todo.md`
  - `doc/npa-standalone-repo-activation.md`
  - `doc/registry-readiness.md`
  - proof-corpus expansion workflow docs unless explicitly published as user
    guidance.
- Acceptance criteria:
  - A new contributor can tell which docs are public user docs and which docs
    are internal planning/evidence docs.
- Evidence fixed on 2026-06-02:
  - Added `doc/README.md` as the public/internal documentation classification
    page.
  - Classified public-facing docs, internal/development docs, design specs,
    public examples/fixtures, external package CI docs, toolchain reference
    docs, and external checker docs.
  - At PUB-00 completion time, kept PUB-01 through PUB-13 pending; no root
    README rewrite, LICENSE file, or CONTRIBUTING file was added in PUB-00.

### PUB-01 Rewrite Root README In English

- Status: Completed
- Deliverables:
  - English-first `README.md`.
  - Old Japanese implementation detail moved into an internal doc if still
    needed.
- README should cover:
  - What NPA is.
  - Experimental status.
  - Certificate-first trust boundary.
  - Build from source.
  - `npa --version`.
  - Package verification quick start.
  - Repository layout.
  - Links to public docs and internal development docs.
- README should not include:
  - Long phase-by-phase implementation logs.
  - Internal milestone evidence.
  - Japanese-only explanations in the public entry path.
- Acceptance criteria:
  - Root README is useful to a public English-speaking package author or
    auditor in under five minutes.
  - README clearly states that parser, elaborator, tactic, automation, AI,
    CI, release pages, and registry metadata are not trusted proof evidence.
- Evidence fixed on 2026-06-02:
  - Rewrote `README.md` as an English-first public repository entry.
  - Covered project purpose, experimental status, certificate-first trust
    boundary, source build instructions, `npa --version`, package verification
    quick start, repository layout, documentation links, and local gates.
  - Moved the old Japanese implementation-status and development-gate notes to
    `doc/internal-readme-notes-ja.md`.
  - Kept PUB-02 and PUB-03 pending; no root `LICENSE` or `CONTRIBUTING.md` was
    added in PUB-01.

### PUB-02 Add Root LICENSE

- Status: Completed
- Deliverables:
  - `LICENSE` at repository root.
- Acceptance criteria:
  - License matches workspace crate metadata, currently MIT.
  - Public GitHub page shows a recognized license.
- Evidence fixed on 2026-06-02:
  - Added root `LICENSE` with standard MIT License text.
  - Confirmed workspace crate metadata remains `license = "MIT"`.
  - Updated `doc/README.md` to mark the public repository entry license item
    as added in PUB-02.

### PUB-03 Add Root CONTRIBUTING In English

- Status: Completed
- Deliverables:
  - `CONTRIBUTING.md`.
- Must cover:
  - Certificate-first contribution policy.
  - Trusted/untrusted boundary.
  - Fast gate:

```sh
./scripts/check-fast.sh
```

  - Proof corpus gate trigger conditions:
    - `proofs/**`
    - `tools/proof-corpus/**`
    - canonical certificate encode/decode/hash/import/axiom report changes
    - kernel semantics/typecheck/reduction/universe/inductive changes
    - independent checker/package verifier/package lock/artifact validation
      changes
    - `.npcert` compatibility changes
  - `unsafe` Rust policy.
  - Rule that unrelated user changes must not be reverted.
- Acceptance criteria:
  - External contributors can run the correct local gate without reading
    `AGENTS.md`.
- Evidence fixed on 2026-06-02:
  - Added root `CONTRIBUTING.md` in English.
  - Covered certificate-first contribution policy, trusted/untrusted boundary,
    fast gate, proof corpus gate trigger conditions, `unsafe` Rust policy,
    unrelated-change handling, certificate compatibility risk, and package
    authoring checks.
  - Updated `README.md` to link to `CONTRIBUTING.md`.
  - Updated `doc/README.md` to mark the public repository entry contribution
    item as added in PUB-03.

### PUB-04 Add English Documentation Index

- Status: Completed
- Deliverables:
  - `doc/README.md` or `doc/index.md`.
- Must categorize:
  - user/package author docs
  - toolchain reference docs
  - design/spec docs
  - internal Japanese development notes
  - release/evidence docs
- Acceptance criteria:
  - Public README can link to the docs index instead of linking directly to
    many internal Japanese docs.
- Evidence fixed on 2026-06-02:
  - Expanded `doc/README.md` from a classification page into the English
    documentation index.
  - Added explicit sections for user/package author docs, toolchain reference
    docs, design/spec docs, release/evidence docs, and internal Japanese
    development notes.
  - Updated root `README.md` to route internal planning, release evidence, and
    Japanese development notes through `doc/README.md` instead of linking them
    directly.

### PUB-05 Clean Public CI Template Documentation

- Status: Completed
- Inputs:
  - `ci-templates/github-actions/README.md`
  - `ci-templates/github-actions/npa-package-pr.yml`
  - `ci-templates/github-actions/npa-package-release.yml`
  - `ci-templates/github-actions/setup-pinned-npa.sh`
- Deliverables:
  - English user-facing CI template docs.
  - Current default example uses:

```text
NPA_GIT_TAG = v0.1.1
RUST_TOOLCHAIN_VERSION = 1.95.0
```

- Acceptance criteria:
  - No stale `v0.1.0` toolchain recommendation remains in public CI docs.
  - Docs state that package workflows use explicit `--root .`.
  - Docs state that package workflows do not use registry lookup,
    latest-version resolution, hidden package caches, or network package
    fetching for proof acceptance.
- Evidence fixed on 2026-06-02:
  - Rewrote `ci-templates/github-actions/README.md` as an English
    package-author guide for copyable GitHub Actions templates.
  - Documented the current default package-author variables:
    `NPA_GIT_TAG = v0.1.1` and `RUST_TOOLCHAIN_VERSION = 1.95.0`.
  - Documented that package workflows run package commands with explicit
    `--root .`.
  - Documented that package workflows do not use registry lookup,
    latest-version resolution, hidden package caches, package solvers, or
    network package fetching as proof acceptance input.
  - Updated `setup-pinned-npa.sh` tag validation guidance to use `v0.1.1` as
    the exact immutable tag example.
  - Removed the stale `v0.1.0` setup example from the linked external theorem
    library CI contract without otherwise broadening the PUB-07 cleanup.

### PUB-06 Review Toolchain Reference Docs

- Status: Completed
- Inputs:
  - `doc/npa-toolchain-reference-v0.1.0.md`
  - `doc/npa-toolchain-reference-v0.1.1.md`
- Deliverables:
  - Confirm `v0.1.1` is documented as the current SRA-02-compatible ref.
  - Confirm `v0.1.0` is documented as historical.
- Acceptance criteria:
  - External package authors are guided to `v0.1.1`.
  - The docs do not imply that `v0.1.0` can build/check the SRA-02
    `npa-std` package fixture.
- Evidence fixed on 2026-06-02:
  - Marked `doc/npa-toolchain-reference-v0.1.1.md` as the current
    SRA-02-compatible package-author ref.
  - Marked `doc/npa-toolchain-reference-v0.1.0.md` as historical and
    audit-only, with an explicit redirect to `v0.1.1`.
  - Documented that `v0.1.0` must not be used to build or check the SRA-02
    `fixtures/npa-std` package fixture.
  - Updated the documentation index to label the `v0.1.0` reference as
    historical rather than a current package-author pin.

### PUB-07 Clean External Theorem Library CI Docs

- Status: Completed
- Inputs:
  - `doc/external-theorem-library-ci.md`
- Deliverables:
  - English external package author documentation.
- Acceptance criteria:
  - The doc explains PR and release gate commands.
  - The doc distinguishes reference-checker-only evidence from future
    high-trust evidence.
  - The doc does not make CI status proof evidence.
- Evidence fixed on 2026-06-02:
  - Rewrote `doc/external-theorem-library-ci.md` as an English package-author
    CI guide.
  - Documented pull request and release gate commands with explicit `--root .`.
  - Separated base reference-checker-only evidence from optional high-trust
    evidence requiring pinned checker binaries, runner policy, checker
    registry, release policy, and release audit evidence.
  - Documented that CI status, GitHub release pages, registry metadata, source
    files, theorem indexes, publish plans, AI traces, and tactic traces are not
    proof evidence.

### PUB-08 Clean External Checker README

- Status: Completed
- Inputs:
  - `checkers/npa-checker-ext/README.md`
- Deliverables:
  - English external checker README, or an explicit note that the checker is
    not part of the first public user path.
- Acceptance criteria:
  - Public readers understand that external checker evidence is optional and
    high-trust only when pinned checker binaries, runner policy, checker
    registry, and release audit evidence are supplied.
- Evidence fixed on 2026-06-02:
  - Rewrote `checkers/npa-checker-ext/README.md` as an English external
    checker README for high-trust integrators.
  - Documented that `npa-checker-ext` is not part of the default public
    package-author path and that base package CI remains
    reference-checker-only.
  - Documented that external checker evidence is optional high-trust release
    evidence only when pinned checker binaries, runner policy, checker
    registry, release policy, and release audit evidence are supplied.
  - Updated `doc/README.md` to mark external checker docs as cleaned in PUB-08
    and optional high-trust only.

### PUB-09 Decide Fixture README Visibility

- Status: Pending
- Inputs:
  - `fixtures/*/README.md`
  - `fixtures/*/CONTRIBUTING.md`
  - `proofs/README.md`
- Deliverables:
  - Decide whether each fixture README is a public example or internal test
    fixture note.
  - English cleanup for any fixture README linked from public docs.
  - Internal/test fixture label for any fixture README not meant as user
    guidance.
- Acceptance criteria:
  - Public README does not send users to Japanese-only internal fixture notes
    without context.

### PUB-10 Stale/Private Assumption Scan

- Status: Pending
- Deliverables:
  - Scan result recorded in the final evidence commit or PR description.
- Verification:

```sh
rg -n "private|PRIVATE|v0\\.1\\.0|NPA_GIT_TAG|latest-version|registry lookup" README.md CONTRIBUTING.md doc ci-templates checkers fixtures proofs
rg -n "[ぁ-んァ-ン一-龯]" README.md CONTRIBUTING.md doc/README.md doc/index.md ci-templates/github-actions/README.md doc/external-theorem-library-ci.md checkers/npa-checker-ext/README.md
```

- Acceptance criteria:
  - Public-facing docs do not refer to `finitefield-org/npa` as private.
  - Public-facing docs do not recommend `v0.1.0` as the current toolchain ref.
  - Public-facing docs do not contain Japanese unless intentionally marked as
    internal/development context.

### PUB-11 Make `finitefield-org/npa` Public And Verify Fetch

- Status: Pending
- Depends on: PUB-00 through PUB-10
- Deliverables:
  - Repository visibility changed to public.
  - Public fetch smoke evidence.
- Verification:

```sh
gh repo view finitefield-org/npa --json nameWithOwner,url,isPrivate,visibility
git ls-remote https://github.com/finitefield-org/npa.git refs/tags/v0.1.1
```

- Acceptance criteria:
  - Unauthenticated GitHub Actions runners can fetch tag `v0.1.1`.

### PUB-12 Rerun `npa-std` SRA-04 Release Workflow

- Status: Pending
- Depends on: PUB-11
- Inputs:
  - `finitefield-org/npa-std` release workflow run that failed while `npa` was
    private.
- Deliverables:
  - Successful `NPA Package Release` workflow run for `npa-std v0.1.0`.
- Verification:

```sh
gh run list --repo finitefield-org/npa-std --workflow "NPA Package Release" --limit 5
gh run view <run-id> --repo finitefield-org/npa-std --json status,conclusion,url,headSha,event
```

- Acceptance criteria:
  - Package artifact checks pass.
  - Fast-kernel source-free verification records diagnostics.
  - Reference checker source-free verification passes.

### PUB-13 Record Final Evidence

- Status: Pending
- Depends on: PUB-12
- Deliverables:
  - Update `doc/npa-standalone-repo-activation.md` SRA-04 evidence.
  - Record public `npa` fetch smoke and successful `npa-std` workflow run.
  - Record release URL and artifact SHA-256.
- Acceptance criteria:
  - SRA-04 can be marked Completed without relying on private repository
    access or local-only evidence.

## Final Acceptance Criteria

- `finitefield-org/npa` is safe to make public from a documentation standpoint.
- Public-facing documentation defaults to English.
- Internal Japanese development docs remain available but are clearly labeled.
- External package authors can find the current toolchain pin and package gate
  commands.
- No public-facing doc suggests that GitHub metadata, CI status, source files,
  theorem indexes, or publish plans are trusted proof evidence.
- `npa-std v0.1.0` release workflow succeeds after `npa` becomes public.
