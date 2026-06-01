# Community Library Roadmap CLR-07 Todo

Source: `doc/community-library-roadmap-todo.md` CLR-07

CLR-07 defines external theorem library CI templates. The templates make a
fresh checkout run the package contract from CLR-04 and the deterministic
artifact checks from CLR-05 without depending on the `npa` repository's local
phase gates, hidden caches, registry lookup, or source-trusting shortcuts.

The CI templates are untrusted orchestration. Passing CI is useful release and
review evidence, but proof acceptance still comes from canonical certificates
and source-free checker verdicts.

---

## Scope

対象:

```text
- external theorem library CI contract
- GitHub Actions template files or copyable workflow snippets for theorem package repositories
- PR-mode package verification template
- release/high-trust package verification template without requiring external checker until CLR-08
- deterministic CI output and failure diagnostics
- artifact upload policy for package diagnostics and generated metadata
- documentation separating external theorem library CI from this repository's local gates
- documentation for pinning the `npa` CLI/toolchain without implicit latest resolution
- tests or static checks for template syntax and command drift
```

非対象:

```text
- enabling `.github/workflows` in this `npa` repository as a required local gate
- registry server
- registry client lookup
- package dependency solver
- network fetch for theorem packages or imports
- binary cache service
- implicit latest `npa` version resolution
- external checker required mode
- `verified_high_trust` artifact
- cryptographic signing
- package publish metadata generation from CLR-06
- seed repository creation from CLR-09
- production LLM, RAG, online theorem graph, browser IDE, or online proving service
```

Trusted-boundary rule:

```text
CI is not part of the trusted proof kernel. The external theorem library CI
templates may invoke `npa package ...` commands and upload diagnostics, but
they do not prove theorems by themselves and must not become checker input.

The templates must not require source, replay, meta, AI traces, theorem graph
scores, registry metadata, hidden package cache, or network package resolution
for source-free verification. The kernel, `npa-cert`, and `npa-checker-ref`
must not depend on CI templates or GitHub Actions behavior.
```

---

## Implementation Specification

### Template Placement

Keep external theorem library CI templates outside `.github/workflows` so this
repository does not accidentally reintroduce its own GitHub Actions gates.

Recommended paths:

```text
ci-templates/github-actions/npa-package-pr.yml
ci-templates/github-actions/npa-package-release.yml
ci-templates/github-actions/README.md
doc/external-theorem-library-ci.md
```

If an implementation chooses another template directory, the directory must be
documented and must not be confused with this repository's active workflows.

### Relationship To Existing Local Gates

This repository currently uses local scripts:

```text
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts are `npa` repository development gates. They are not external
theorem library CI templates and should not be copied wholesale into theorem
libraries.

External theorem libraries should run package commands:

```text
npa package check
npa package build-certs --check
npa package check-hashes
npa package verify-certs --checker reference
npa package axiom-report --check
npa package index --check
```

Optional release-template variants may add `npa package publish-plan --check`
after CLR-06 is implemented, but base CLR-07 depends on CLR-05 and must not
require publish metadata.

### Command Availability

The base CLR-07 templates must use only commands and flags owned by CLR-04 and
CLR-05. PR mode uses the reference checker as its acceptance gate. Release mode
may also run the fast verifier as an additional, clearly labeled gate:

```text
npa package check --root .
npa package build-certs --root . --check
npa package check-hashes --root .
npa package verify-certs --root . --checker fast
npa package verify-certs --root . --checker reference
npa package axiom-report --root . --check
npa package index --root . --check
```

The base templates must not use:

```text
--changed
--all
--checker external
--registry
--network
--latest
```

PR mode may compute and display changed modules for contributor ergonomics, but
until a later milestone implements package-command changed-module selection,
the required verifier command is the full-package reference check. This is a
conservative fallback: it checks changed modules and their reverse dependencies
by checking the entire package.

### Toolchain And CLI Pinning

CI must avoid implicit latest tool resolution. The template should support one
of these pinned installation modes:

```text
- use a checked-in or release-provided `npa` binary path
- build `npa-cli` from a specific Git tag or commit supplied by the workflow
- use a pinned package manager artifact once such distribution exists
```

The template must fail if the `npa` version source is unspecified. It must
print:

```text
npa --version
cargo --version when Rust is used
rustc --version when Rust is used
```

The template must not fetch theorem package dependencies from an NPA registry
or hidden cache. Fetching the Rust toolchain or pinned `npa` implementation is
tool setup, not theorem package dependency resolution, and should be documented
as such.

### PR Mode Template

PR mode is the required contributor gate for theorem-only pull requests.

Required steps:

```text
1. Checkout repository.
2. Install or locate pinned `npa`.
3. Print tool versions.
4. Run `npa package check --root . --json`.
5. Run `npa package build-certs --root . --check --json`.
6. Run `npa package check-hashes --root . --json`.
7. Run `npa package verify-certs --root . --checker reference --json`.
8. Run `npa package axiom-report --root . --check --json`.
9. Run `npa package index --root . --check --json`.
10. Upload deterministic JSON diagnostics and generated artifact diffs on failure.
```

Acceptance behavior:

```text
- Any package validation, build, hash, reference checker, axiom policy, or index failure fails the job.
- Usage errors fail the job.
- Missing generated artifacts fail the job.
- Full-package verification is acceptable until changed-module selection is implemented.
- External checker is not required in PR mode.
```

PR mode must not:

```text
- upload secrets
- write back to the branch
- contact an NPA package registry
- resolve package imports by latest version
- trust source, replay, meta, AI trace, or theorem index as proof evidence
```

### Release / High-Trust Template

Release mode runs full package verification and deterministic artifact checks
from a clean checkout.

Required steps for CLR-07 base release template:

```text
1. Checkout repository at the release ref.
2. Install or locate pinned `npa`.
3. Run `npa package check --root . --json`.
4. Run `npa package build-certs --root . --check --json`.
5. Run `npa package check-hashes --root . --json`.
6. Run `npa package verify-certs --root . --checker fast --json`.
7. Run `npa package verify-certs --root . --checker reference --json`.
8. Run `npa package axiom-report --root . --check --json`.
9. Run `npa package index --root . --check --json`.
10. Upload certificate, package lock, axiom report, theorem index, and JSON diagnostic artifacts.
```

Optional later extensions:

```text
- Add `npa package publish-plan --root . --check --json` after CLR-06.
- Add external checker job after CLR-08.
- Add release signing after a signing policy milestone.
```

Release mode must label fast verifier results as fast-kernel and reference
checker results as reference checker. It must not report fast-kernel success as
independent reference checker success.

### Artifact Upload Policy

CI may upload these artifacts:

```text
generated/package-lock.json
generated/axiom-report.json
generated/theorem-index.json
generated/publish-plan.json when CLR-06 extension is enabled
checked local certificates
command JSON diagnostics
plain text summary logs
```

CI must not upload by default:

```text
AI traces
tactic traces
prompt metadata
secrets
host-specific caches
unredacted environment dumps
```

Uploaded theorem index, axiom report, and publish metadata are helper artifacts,
not proof acceptance evidence.

### Failure Diagnostics

Every package command in the template should run with `--json` and save a
package-relative diagnostic file such as:

```text
ci-output/package-check.json
ci-output/build-certs.json
ci-output/check-hashes.json
ci-output/verify-certs-reference.json
ci-output/axiom-report.json
ci-output/index.json
```

The workflow summary should show:

```text
- failed command
- exit code
- diagnostic kind
- reason code
- module when available
- path when available
- expected hash and actual hash when available
```

Human-readable summaries are review aids. The structured JSON remains the
stable CI output.

### Template Syntax Validation

If YAML workflows are added, validate them with at least one cheap static check:

```text
- `actionlint` when available
- Ruby or Python YAML parsing when available
- repository-local schema/snapshot tests if workflow generation is templated
```

Do not require network access to validate template syntax.

### Handoff To Later Milestones

CLR-06 optional extension:

```text
npa package publish-plan --root . --check --json
release artifact list upload
registry seed metadata upload
```

CLR-08 optional extension:

```text
external checker required job
verified_high_trust artifact check
fast/reference/external disagreement gate
```

CLR-09 consumes:

```text
external theorem library CI templates
contributor workflow documentation
fresh-checkout CI expectations
```

---

## Tasks

### CLR-07-01 Define External CI Contract And Template Locations

- Status: Completed
- Depends on: CLR-05
- Inputs:
  - `doc/community-library-roadmap-todo.md` CLR-07
  - `doc/community-library-roadmap.md` section 5.3 and M3
  - local gate descriptions in README
  - existing `scripts/phase8-release-audit.sh`
  - existing `scripts/phase9-regression.sh`
- Code or documentation areas:
  - `doc/external-theorem-library-ci.md`
  - `ci-templates/github-actions/README.md`
  - `doc/community-library-roadmap-todo.md`
- Deliverables:
  - Written CI contract for PR mode and release mode.
  - Template directory decision.
  - Documentation that external theorem library CI is separate from this repository's local gates.
  - Documentation that CI is orchestration metadata, not proof evidence.
- Acceptance criteria:
  - The contract names only actual package commands and supported flags from CLR-04 and CLR-05.
  - The contract explicitly excludes registry lookup, hidden package caches, implicit latest version resolution, and external checker as a required CLR-07 gate.
  - The contract documents full-package verification as the conservative PR fallback until changed-module selection exists.
  - `.github/workflows` is not reintroduced for this repository unless explicitly documented as an active local workflow.
- Verification:
  - `rg -n "external theorem library CI|phase8-release-audit|phase9-regression|not proof evidence|--changed|--all" doc ci-templates README.md`
  - `git diff --check`
- Notes:
  - Keep this as the contract source before adding concrete YAML.

### CLR-07-02 Add Pinned Toolchain And `npa` CLI Setup Guidance

- Status: Completed
- Depends on: CLR-07-01
- Inputs:
  - `npa-cli` binary name fixed by CLR-00
  - package command contract from CLR-04 and CLR-05
  - external theorem library fresh-checkout requirement
- Code or documentation areas:
  - `ci-templates/github-actions/README.md`
  - `doc/external-theorem-library-ci.md`
  - YAML template environment variables if templates are added
- Deliverables:
  - Pinned `npa` CLI setup strategy for external library CI.
  - Required workflow inputs for `NPA_VERSION`, pinned Git tag, commit, or binary path.
  - Version-printing step for `npa`, `cargo`, and `rustc` when Rust is used.
  - Failure behavior when the `npa` source is not pinned.
- Acceptance criteria:
  - Templates do not use floating latest versions.
  - Templates do not fetch theorem packages or imports from a registry.
  - Tool installation is clearly separated from theorem package dependency resolution.
  - Setup docs work for a fresh checkout of an external theorem package.
- Verification:
  - `rg -n "NPA_VERSION|npa --version|latest|registry|hidden package cache" ci-templates doc/external-theorem-library-ci.md`
  - `git diff --check`
- Notes:
  - If the implementation cannot provide an installer yet, make the template accept an existing `npa` binary path and fail clearly otherwise.

### CLR-07-03 Create PR-Mode GitHub Actions Template

- Status: Completed
- Depends on: CLR-07-02
- Inputs:
  - PR mode command list from this document
  - structured diagnostics from CLR-04
  - axiom/index commands from CLR-05
- Code or documentation areas:
  - `ci-templates/github-actions/npa-package-pr.yml`
  - `ci-templates/github-actions/README.md`
  - optional helper scripts under `ci-templates/`
- Deliverables:
  - Copyable GitHub Actions PR workflow template.
  - Steps for package check, build-certs check, hash check, source-free reference verify, axiom report check, and theorem index check.
  - Command JSON output saved to deterministic `ci-output/*.json` paths.
  - Failure summary that points to command diagnostics.
- Acceptance criteria:
  - The template uses `npa package verify-certs --checker reference`, not fast verifier as the PR acceptance gate.
  - The template does not use `--changed`, `--all`, `--checker external`, `--registry`, `--network`, or `--latest`.
  - The template checks the full package as a conservative fallback for changed modules.
  - The job permissions are minimal and do not require secrets for pull requests.
  - The template uploads diagnostics only when useful and avoids host-specific caches or environment dumps.
- Verification:
  - `rg -n "package check|build-certs|check-hashes|verify-certs|axiom-report|package index|--changed|--checker external" ci-templates/github-actions/npa-package-pr.yml`
  - `git diff --check`
  - `actionlint ci-templates/github-actions/npa-package-pr.yml` when available
- Notes:
  - If `actionlint` is not available, document the skipped syntax validator and run a local YAML parser if one is available.

### CLR-07-04 Create Release/High-Trust GitHub Actions Template

- Status: Pending
- Depends on: CLR-07-03
- Inputs:
  - release mode command list from this document
  - source-free fast and reference verifier commands from CLR-04
  - generated artifact checks from CLR-05
  - optional publish-plan extension from CLR-06
- Code or documentation areas:
  - `ci-templates/github-actions/npa-package-release.yml`
  - `ci-templates/github-actions/README.md`
  - `doc/external-theorem-library-ci.md`
- Deliverables:
  - Copyable release/high-trust workflow template.
  - Full-package fast verifier and reference checker jobs.
  - Axiom report and theorem index artifact checks.
  - Artifact upload policy for package lock, certificates, axiom report, theorem index, and diagnostics.
  - Documented optional publish-plan step that is disabled unless CLR-06 artifacts are present.
- Acceptance criteria:
  - Release template does not require external checker until CLR-08.
  - Fast verifier output is labeled fast-kernel and not reference checker success.
  - Publish-plan is optional and clearly gated on CLR-06.
  - The workflow does not contact an NPA registry or resolve latest package versions.
  - Workflow examples match actual package CLI names and supported flags.
- Verification:
  - `rg -n "verify-certs --root \\. --checker fast|verify-certs --root \\. --checker reference|publish-plan|checker external|latest" ci-templates/github-actions/npa-package-release.yml`
  - `git diff --check`
  - `actionlint ci-templates/github-actions/npa-package-release.yml` when available
- Notes:
  - Treat high-trust as a release template profile name for CLR-07. The external checker and `verified_high_trust` artifact remain CLR-08.

### CLR-07-05 Add Contributor Failure Guidance And Diagnostic Mapping

- Status: Pending
- Depends on: CLR-07-03, CLR-07-04
- Inputs:
  - `npa.package.command_result.v0.1` diagnostics from CLR-04
  - axiom report and theorem index diagnostics from CLR-05
  - package command exit codes
- Code or documentation areas:
  - `doc/external-theorem-library-ci.md`
  - `ci-templates/github-actions/README.md`
  - optional helper script for summarizing JSON diagnostics
- Deliverables:
  - Failure guide mapping common command failures to contributor actions.
  - Examples for stale source hash, stale certificate, failed reference checker, axiom policy violation, stale axiom report, and stale theorem index.
  - Workflow summary format for failed command, module, path, reason code, and hash mismatch.
- Acceptance criteria:
  - Guidance tells contributors to update sources/certificates/artifacts explicitly rather than trusting CI output.
  - Guidance does not ask contributors to edit expected hashes blindly.
  - Guidance states that theorem index and axiom report metadata are not proof evidence.
  - Failure messages are deterministic and avoid absolute host paths.
- Verification:
  - `rg -n "source_hash_mismatch|certificate_hash_mismatch|axiom_report|theorem_index|reason_code|not proof evidence" doc/external-theorem-library-ci.md ci-templates`
  - `git diff --check`
- Notes:
  - Keep this contributor-facing; avoid exposing internal implementation details that are not actionable.

### CLR-07-06 Validate Template Syntax And Command Drift

- Status: Pending
- Depends on: CLR-07-03, CLR-07-04, CLR-07-05
- Inputs:
  - CI templates
  - package command examples from CLR-04 and CLR-05
  - repository validation tools available locally
- Code or documentation areas:
  - `ci-templates/github-actions/*.yml`
  - tests or scripts only if lightweight and useful
  - `doc/external-theorem-library-ci.md`
- Deliverables:
  - Lightweight static validation for workflow YAML.
  - Drift checks that fail if templates use unsupported package flags.
  - Documentation for any validator that is optional because it may not be installed.
- Acceptance criteria:
  - Validation catches unsupported `--changed`, `--all`, `--checker external`, `--registry`, `--network`, and `--latest` in base templates.
  - Validation checks that required PR-mode commands are present.
  - Validation checks that release-mode fast and reference checker commands are present.
  - The validation itself does not require network access.
- Verification:
  - `git diff --check`
  - `rg -n -- "--changed|--all|--checker external|--registry|--network|--latest" ci-templates && false || true`
  - `rg -n "package check|build-certs|check-hashes|verify-certs|axiom-report|package index" ci-templates`
  - `actionlint ci-templates/github-actions/*.yml` when available
- Notes:
  - If an implementation adds a dedicated validation script, keep it independent of this repository's local phase gates.

### CLR-07-07 Update Roadmap And Seed-Repo Handoff Documentation

- Status: Pending
- Depends on: CLR-07-06
- Inputs:
  - `doc/community-library-roadmap-todo.md`
  - `doc/community-library-roadmap.md`
  - `README.md`
  - `doc/external-theorem-library-ci.md`
  - CLR-09 seed repository expectations
- Code or documentation areas:
  - `README.md`
  - `doc/community-library-roadmap-todo.md`
  - `doc/community-library-roadmap.md`
  - `ci-templates/github-actions/README.md`
- Deliverables:
  - Documentation that external theorem library CI templates are available.
  - Handoff note for CLR-09 describing how `npa-mathlib-seed` should copy or reference the templates.
  - Documentation that this repository's local scripts remain separate from external theorem library workflows.
  - Note that external checker required mode waits for CLR-08.
- Acceptance criteria:
  - Parent roadmap points to this detailed CLR-07 task document.
  - Docs do not imply this repository has active GitHub Actions gates if `.github/workflows` remains absent.
  - Docs do not imply registry access, package solver, binary cache, or latest-version resolution.
  - Docs identify the exact package commands required for PR and release templates.
- Verification:
  - `rg -n "community-library-roadmap-clr-07-todo|external theorem library CI|npa-package-pr|npa-package-release|phase8-release-audit|phase9-regression" README.md doc ci-templates`
  - `git diff --check`
- Notes:
  - Keep dogfood execution itself in CLR-09.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: Earlier source roadmap drafts mentioned `--changed`, `--all`, and
external checker in CI examples, but CLR-04 and CLR-05 deliberately do not
implement those flags.
Fix: CLR-07 base templates use only supported package flags. PR mode performs
full-package verification as a conservative fallback until changed-module
selection is implemented.

Finding: The parent CLR-07 milestone depends on CLR-05, while publish-plan is
owned by CLR-06.
Fix: The base CLR-07 release template does not require publish-plan. A
publish-plan step is documented as an optional CLR-06 extension.

Finding: Reintroducing `.github/workflows` in this repository could be confused
with external theorem library CI.
Fix: Templates live under `ci-templates/github-actions` and docs explain that
`scripts/phase8-release-audit.sh` and `scripts/phase9-regression.sh` remain
local `npa` repository gates.

Finding: CI templates could accidentally rely on floating latest tool versions
or hidden package caches.
Fix: The setup specification requires pinned `npa` CLI/toolchain inputs and
forbids theorem package registry lookup, hidden package caches, and latest
package resolution.

Finding: CI pass/fail could be mistaken for proof evidence.
Fix: The trusted-boundary and diagnostic sections state that CI is untrusted
orchestration and proof acceptance remains canonical certificates plus
source-free checker verdicts.

Finding: The release template needs the fast verifier as an additional labeled
gate, but the command-availability list originally named only the reference
checker command.
Fix: The command-availability section now includes both `--checker fast` and
`--checker reference`, while keeping reference checker as the PR acceptance
gate and forbidding `--checker external` in base CLR-07 templates.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now fixes external CI contract,
toolchain pinning, PR and release templates, diagnostics, static validation,
and CLR-09 handoff boundaries without depending on unsupported package flags.
```
