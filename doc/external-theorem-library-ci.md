# External Theorem Library CI Contract

This document is the CLR-07 contract source for external theorem library CI.
It describes what a theorem package repository should run in pull requests and
release checks before concrete workflow templates are added.

The templates are external-theorem-library CI examples. They are not active
workflows for this `npa` repository, and they must remain separate from local
`npa` development gates such as:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts test this repository's checker and regression fixtures. External
theorem libraries should run package commands against their own package root
instead.

CI is untrusted orchestration metadata, not proof evidence. A passing CI job is
useful review and release evidence, but proof acceptance remains canonical
`.npcert` bytes plus source-free checker or kernel verifier verdicts. CI output,
diagnostics, generated package metadata, logs, and uploaded artifacts do not
become checker input.

## Template Location

Copyable GitHub Actions templates live under:

```text
ci-templates/github-actions/
```

Reserved template names are:

```text
ci-templates/github-actions/npa-package-pr.yml
ci-templates/github-actions/npa-package-release.yml
ci-templates/github-actions/README.md
```

CLR-07-01 establishes this directory and the CI contract only. It does not add
active repository workflows under `.github/workflows`.

## Required Package Commands

The base CLR-07 contract uses only package commands and flags already owned by
CLR-04 and CLR-05:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

Release mode may also run the fast verifier as an additional labeled gate:

```sh
npa package verify-certs --root . --checker fast --json
```

Fast verifier success must be labeled fast-kernel. It must not be reported as
reference checker success.

Source-free verification steps read package metadata and certificate artifacts.
They must not require `.npa` source files, replay files, meta files, theorem
graph scores, prompt metadata, AI traces, registry metadata, hidden package
caches, or network package resolution.

## PR Mode

PR mode is the required contributor gate for theorem-only pull requests.

Required behavior:

- Check out the external theorem package repository.
- Locate or install the `npa` CLI according to the pinning rules that CLR-07-02
  will define.
- Print the `npa` version.
- Run the required package commands against `--root .`.
- Save deterministic JSON diagnostics under package-relative paths such as
  `ci-output/package-check.json`.
- Fail the job on package validation, deterministic build, hash, source-free
  reference checker, axiom policy, or theorem index failures.

The PR verifier is full-package reference verification. Changed-module
selection is useful ergonomics, but the required command remains the full
package fallback until package-command changed-module selection exists.

PR mode must not:

- use `--changed`;
- use `--all`;
- use `--checker external`;
- use `--registry`, `--network`, or `--latest`;
- write back to the contributor branch;
- upload secrets;
- contact an NPA package registry;
- resolve imports through hidden package caches or implicit latest versions;
- trust source, replay, meta, AI trace, prompt metadata, or theorem index data
  as proof evidence.

## Release Mode

Release mode runs from a clean checkout at the release ref.

Required behavior:

- Locate or install the `npa` CLI according to the pinning rules that CLR-07-02
  will define.
- Print the `npa` version.
- Run the base package checks.
- Run both fast and reference source-free verification.
- Run axiom-report and theorem-index check mode.
- Upload deterministic diagnostics and checked release artifacts useful for
  review.

The base CLR-07 release template must not require the external checker.
`--checker external`, external checker disagreement gates, and
`verified_high_trust` artifacts belong to CLR-08.

Publish metadata is not a base CLR-07 dependency. A later optional release
template variant may add:

```sh
npa package publish-plan --root . --check --json
```

That optional extension is release metadata, not proof evidence, and it must
not imply a registry server exists.

## Explicit Exclusions

Base CLR-07 CI must not rely on:

- registry lookup;
- hidden package caches;
- package dependency solvers;
- implicit latest version resolution;
- binary cache services;
- source-trusting shortcuts;
- external checker required mode;
- `verified_high_trust`;
- cryptographic signing;
- this repository's local phase gates.

Fetching an `npa` implementation or Rust toolchain is tool setup. It is not
theorem package dependency resolution. CLR-07-02 defines the exact required
pinning inputs and failure behavior for unspecified `npa` sources.

## Diagnostic Output

Structured command output should be saved as deterministic JSON diagnostics,
for example:

```text
ci-output/package-check.json
ci-output/build-certs.json
ci-output/check-hashes.json
ci-output/verify-certs-reference.json
ci-output/axiom-report.json
ci-output/index.json
```

Human-readable workflow summaries may show the failed command, exit code,
diagnostic kind, reason code, module, package-relative path, and expected or
actual hashes when available. Summaries are review aids only; structured JSON
is the stable CI output.

Allowed uploads include generated package metadata, checked local
certificates, command JSON diagnostics, and plain text summary logs. Default
uploads must exclude AI traces, tactic traces, prompt metadata, secrets,
host-specific caches, and unredacted environment dumps.

## Later Handoff

CLR-07-03 and CLR-07-04 add concrete `npa-package-pr` and
`npa-package-release` workflow templates under `ci-templates/github-actions/`.
CLR-09 copies or references those templates for the seed theorem library.
