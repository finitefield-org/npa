# GitHub Actions Templates

This directory contains copyable GitHub Actions templates for external NPA
theorem package repositories. It is intentionally outside `.github/workflows`
so the files are not active workflows for the `npa` repository itself.

The templates help package authors run deterministic package checks with a
pinned `npa` toolchain. They do not make CI status, GitHub release pages,
package registry metadata, or uploaded artifacts part of proof acceptance.

## Files

```text
npa-package-pr.yml             pull request package checks
npa-package-release.yml        release package checks
npa-package-high-trust.yml     optional high-trust release extension
setup-pinned-npa.sh            pinned npa setup helper
summarize-npa-diagnostics.py   deterministic diagnostic summary helper
validate-workflows.py          local workflow drift validator
```

External theorem libraries should copy the workflow files and helper scripts
they use into the same paths referenced by the templates. Do not copy local
`npa` repository development gates such as:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts test this repository's checker and regression fixtures. External
theorem libraries should run package commands against their own package root.

## Quick Start

For the current package-author path, set these repository variables:

```text
NPA_GIT_TAG = v0.1.1
RUST_TOOLCHAIN_VERSION = 1.95.0
```

Then copy:

```text
ci-templates/github-actions/npa-package-pr.yml
ci-templates/github-actions/npa-package-release.yml
ci-templates/github-actions/setup-pinned-npa.sh
ci-templates/github-actions/summarize-npa-diagnostics.py
ci-templates/github-actions/validate-workflows.py
```

Install the copied `npa-package-*.yml` files under `.github/workflows/` in the
theorem package repository. Keep helper scripts under
`ci-templates/github-actions/`, or update the helper script paths in the
workflow YAML in the same change.

The high-trust workflow is optional. Copy
`npa-package-high-trust.yml` only after the package repository also provides
the documented checker binary, runner policy, checker registry, and release
audit evidence.

## Trust Boundary

CI is untrusted orchestration metadata. A passing workflow is useful review
evidence, but proof acceptance remains:

```text
canonical .npcert bytes
source-free reference checker or kernel verifier verdicts
deterministic certificate, import, export, axiom-report, and index hashes
```

Fetching the pinned `npa` implementation or the pinned Rust toolchain is tool
setup. Package workflows must not use registry lookup, latest-version
resolution, hidden package caches, package dependency solvers, or network
package fetching as proof acceptance input.

Source-free verification may read package metadata, package locks, canonical
certificate files, import certificates, and axiom policy. It must not trust
`.npa` source files, replay files, tactic traces, AI traces, prompt metadata,
theorem search indexes, registry network data, hidden caches, or plugins as
proof evidence.

## Pinned Setup Inputs

The setup helper requires exactly one pinned `npa` source:

```text
NPA_BINARY_PATH
  Path to an existing executable npa binary.

NPA_VERSION
  Exact release version or release tag for a later release-download strategy.
  This mode is currently rejected until release-download artifacts are added.
  The value latest is invalid.

NPA_GIT_TAG
  Exact immutable Git tag for building npa-cli.

NPA_GIT_COMMIT
  Full lowercase 40-hex Git commit SHA for building npa-cli.
```

If none or multiple are set, setup fails before running package commands.

When the helper builds `npa-cli` from `NPA_GIT_TAG` or `NPA_GIT_COMMIT`, it
uses the exact `RUST_TOOLCHAIN_VERSION`. If the variable is unset, the helper
defaults to:

```text
RUST_TOOLCHAIN_VERSION = 1.95.0
```

The setup helper prints:

```sh
npa --version
cargo --version
rustc --version
```

`cargo --version` and `rustc --version` are printed only when Rust is used to
build `npa-cli`.

## Package Commands

All template package commands use explicit `--root .` so the package root is
the checked-out theorem package repository. The base package command set is:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

The release workflow also runs the fast verifier as a labeled non-reference
gate:

```sh
npa package verify-certs --root . --checker fast --json
```

Fast verifier output is labeled fast-kernel and must not be reported as
reference checker success.

Publish-plan checking is optional. Set `NPA_ENABLE_PUBLISH_PLAN` to `true` and
check in `generated/publish-plan.json` to run:

```sh
npa package publish-plan --root . --check --json
```

Publish metadata is release review metadata, not proof evidence, and it does
not imply a package registry server exists.

## Pull Request Workflow

`npa-package-pr.yml` is the pull request template. It:

- checks out the theorem package repository;
- locates or builds `npa` from exactly one pinned source;
- runs the full package command set with explicit `--root .`;
- saves deterministic JSON diagnostics under `ci-output/`;
- fails on package validation, deterministic certificate build, hash,
  source-free reference checker, axiom policy, or theorem index failures.

The pull request workflow is the contributor hot path. It intentionally does
not use changed-module selectors, external checker mode, registry lookup,
network package resolution, hidden caches, or implicit latest package
resolution.

## Release Workflow

`npa-package-release.yml` runs from a clean checkout at the release ref. It:

- runs package artifact checks;
- runs fast-kernel source-free verification;
- runs reference checker source-free verification;
- uploads checked package metadata, certificate artifacts, and JSON
  diagnostics.

The base release workflow does not require the external checker and does not
emit high-trust release evidence.

## High-Trust Extension

`npa-package-high-trust.yml` is an optional release extension. It requires all
of these inputs before verifier commands run:

```text
NPA_CHECKER_EXT_BINARY_PATH
NPA_RELEASE_POLICY_HASH
NPA_RUNNER_POLICY_HASH
NPA_CHALLENGE_RUNNER_POLICY_HASH
ci/release.high-trust.json
ci/runner.high-trust.json
ci/runner.challenge.json
ci/checker-binaries.json
generated/release-audit/manifest.json
```

The external checker command is:

```sh
npa package verify-certs --root . --checker external \
  --runner-policy ci/runner.high-trust.json \
  --runner-policy-hash "$NPA_RUNNER_POLICY_HASH" \
  --checker-registry ci/checker-binaries.json \
  --json
```

The high-trust workflow also runs:

```sh
npa package high-trust --root . \
  --release-policy ci/release.high-trust.json \
  --release-policy-hash "$NPA_RELEASE_POLICY_HASH" \
  --runner-policy ci/runner.high-trust.json \
  --runner-policy-hash "$NPA_RUNNER_POLICY_HASH" \
  --challenge-runner-policy ci/runner.challenge.json \
  --challenge-runner-policy-hash "$NPA_CHALLENGE_RUNNER_POLICY_HASH" \
  --checker-registry ci/checker-binaries.json \
  --out generated/verified-high-trust.json \
  --check \
  --json
```

`verified_high_trust` is generated only after external checker and
high-trust-reference release audit evidence exists. It must not be emitted from
reference-checker-only release evidence.

## Diagnostics

Failure summaries use package command JSON diagnostics when
`summarize-npa-diagnostics.py` is copied with the workflows. The summary table
is deterministic and intentionally limited to:

```text
file | command | status | exit_code | kind | reason_code | module | path | expected_hash | actual_hash
```

The table uses package-relative paths and omits raw runner stderr, absolute
host paths, environment variables, and caches.

Common diagnostic actions:

| Diagnostic | Action |
| --- | --- |
| `source_hash_mismatch` | Review the source change, then update package metadata through the normal package update flow. Do not edit expected hashes blindly. |
| `certificate_hash_mismatch` or `certificate_file_hash_mismatch` | Rebuild/check certificates explicitly, review the certificate and package-lock diffs, then rerun hash and verifier checks. |
| `reference_checker_rejected` | Treat the `.npcert` as rejected proof evidence; fix the theorem or certificate generation path and rerun reference verification. |
| `axiom_policy_rejected` or `axiom_report_policy_violation` | Remove the unapproved axiom dependency or update package axiom policy through review. |
| `axiom_report_stale` or `axiom_report_hash_mismatch` | Regenerate `generated/axiom-report.json`, review the diff, then rerun `npa package axiom-report --root . --check --json`. |
| `theorem_index_stale` or `theorem_index_hash_mismatch` | Regenerate `generated/theorem-index.json`, review the diff, then rerun `npa package index --root . --check --json`. |

Theorem index and axiom report metadata are derived review/search artifacts.
They are not proof evidence.

## Local Validation

Validate workflow syntax with `actionlint` when it is installed:

```sh
actionlint ci-templates/github-actions/*.yml
```

If `actionlint` is unavailable, use Ruby's bundled YAML parser as a cheap
syntax fallback:

```sh
for workflow in ci-templates/github-actions/*.yml; do
  ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "$workflow"
done
```

Run the local no-network validator to combine YAML parsing with package-command
drift checks:

```sh
python3 ci-templates/github-actions/validate-workflows.py
```

The validator checks that workflows still contain the required package
commands. It also fails if a workflow adds unsupported changed/all selectors,
registry lookup, network package resolution, implicit latest package
resolution, or unsupported external checker mode in the base workflows.

## Explicit Exclusions

Base package workflows must not:

- use `--changed`;
- use `--all`;
- use `--registry`;
- use `--network`;
- use `--latest`;
- require external checker mode;
- contact an NPA package registry;
- use hidden package caches;
- use a package dependency solver;
- resolve imports by implicit latest version;
- treat CI output, release pages, registry metadata, source files, replay
  files, theorem indexes, or AI traces as proof evidence.
