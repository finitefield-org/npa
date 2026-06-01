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
ci-templates/github-actions/npa-package-high-trust.yml
ci-templates/github-actions/summarize-npa-diagnostics.py
ci-templates/github-actions/validate-workflows.py
ci-templates/github-actions/README.md
```

CLR-07-01 establishes this directory and the CI contract only. It does not add
active repository workflows under `.github/workflows`.

## Pinned `npa` Setup

External theorem library CI must use an explicitly pinned `npa` CLI. It must
not use floating names such as `latest`, `stable`, `main`, `master`, or `HEAD`
to choose the verifier implementation.

Workflow templates should support these setup inputs:

```text
NPA_BINARY_PATH
  Package-relative or workspace-relative path to an existing executable `npa`
  binary. This is the CLR-07-02 baseline because it works before an installer
  or package manager distribution exists.

NPA_VERSION
  Exact release version or release tag for `npa`, for example `v0.1.0`.
  This is allowed only when the template has a pinned release download or
  checked release artifact strategy. The value `latest` is invalid.

NPA_GIT_TAG
  Exact immutable Git tag to build `npa-cli` from. Branch names are invalid.

NPA_GIT_COMMIT
  Full Git commit SHA to build `npa-cli` from. Short SHA prefixes are not
  enough for the base contract.
```

A workflow must provide exactly one `npa` source: `NPA_BINARY_PATH`,
`NPA_VERSION`, `NPA_GIT_TAG`, or `NPA_GIT_COMMIT`. If none are set, or if more
than one is set, the setup step must fail before running any package command.

Until concrete installer templates are added, the portable fresh-checkout setup
is:

```sh
test -n "${NPA_BINARY_PATH:-}" || {
  echo "NPA_BINARY_PATH must point to a pinned npa binary" >&2
  exit 2
}
test -x "$NPA_BINARY_PATH" || {
  echo "NPA_BINARY_PATH is not executable: $NPA_BINARY_PATH" >&2
  exit 2
}
"$NPA_BINARY_PATH" --version
```

When a template builds from `NPA_GIT_TAG` or `NPA_GIT_COMMIT`, it may fetch the
pinned `npa` implementation as tool setup. That fetch is not theorem package
dependency resolution and must not fetch theorem packages, package imports,
registry metadata, or hidden package cache entries.

If Rust is used to build `npa-cli`, the Rust toolchain must also be pinned by a
checked-in `rust-toolchain.toml` or an exact workflow input such as
`RUST_TOOLCHAIN_VERSION`. Floating toolchain names are not allowed in the base
contract. The setup step must print:

```sh
npa --version
cargo --version
rustc --version
```

`cargo --version` and `rustc --version` are required only when Rust is used by
the workflow. A checked-in or release-provided `npa` binary path does not need
Cargo or Rust.

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
- Locate or install the `npa` CLI from exactly one pinned source.
- Print `npa --version`, and print `cargo --version` plus `rustc --version`
  when Rust is used to build `npa`.
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

- Locate or install the `npa` CLI from exactly one pinned source.
- Print `npa --version`, and print `cargo --version` plus `rustc --version`
  when Rust is used to build `npa`.
- Run the base package checks.
- Run both fast and reference source-free verification.
- Run axiom-report and theorem-index check mode.
- Upload deterministic diagnostics, package lock, checked certificates, axiom
  report, theorem index, and optional publish metadata useful for review.

The base CLR-07 release template must not require the external checker. It
continues to produce reference-checker-only release evidence plus the labeled
fast-kernel verifier result.

`npa-package-high-trust.yml` is the opt-in high-trust release extension. It may
add the implemented external checker package command only when the documented
CI environment provides a pinned built `npa-checker-ext` executable and
runner-owned policy / registry files:

```sh
npa package verify-certs --root . --checker external \
  --runner-policy ci/runner.high-trust.json \
  --runner-policy-hash "$NPA_RUNNER_POLICY_HASH" \
  --checker-registry ci/checker-binaries.json \
  --json
```

The high-trust template must fail before verifier execution unless these
inputs are present:

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

That command is source-free: it may read package metadata, package lock,
canonical `.npcert` files, import certificates, runner policy, checker
registry, checker executable bytes, and axiom policy. It must not read `.npa`
source, replay files, meta files, theorem index files, AI traces, registry
network data, hidden package caches, plugins, or source-derived unchecked
environments.

The high-trust-reference job validates the release audit bundle through
`npa package high-trust`; it does not reuse a normal `reference` result as
`high-trust-reference`. `verified_high_trust` artifacts require
`npa package high-trust` and complete external / high-trust-reference release
audit evidence. They must not be emitted from reference-checker-only evidence.

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

Fetching a pinned `npa` implementation or pinned Rust toolchain is tool setup.
It is not theorem package dependency resolution.

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

Copyable templates may use
`ci-templates/github-actions/summarize-npa-diagnostics.py` to render a summary
table from `npa.package.command_result.v0.1` JSON:

```text
file | command | status | exit_code | kind | reason_code | module | path | expected_hash | actual_hash
```

The table must use package-relative paths from diagnostics such as
`generated/package-lock.json` or `Proofs/A/certificate.npcert`. It must not
include absolute host paths, environment dumps, or raw stderr with local runner
state. If a command fails before writing JSON, the workflow should still point
to the missing `ci-output/*.json` file and the failed GitHub Actions step.

## Contributor Failure Guide

When CI fails, contributors should fix the source package state and regenerate
owned artifacts with `npa package ...` commands. They should not edit expected
hashes blindly to match CI output. Expected hashes are package pins; changing
them is correct only after reviewing the corresponding source, certificate, or
generated artifact change.

Common diagnostic mappings:

| Diagnostic | Likely cause | Contributor action |
| --- | --- | --- |
| `source_hash_mismatch` from `package check-hashes` | A checked source file changed but `expected_source_hash` still pins the old bytes. | Review the source change, then update package metadata through the normal package update flow. Rerun `npa package check-hashes --root . --json`. |
| `certificate_hash_mismatch`, `certificate_file_hash_mismatch`, `export_hash_mismatch`, or `axiom_report_hash_mismatch` | A certificate artifact is stale, missing from the package lock, or no longer matches the manifest pins. | Rebuild/check certificates explicitly with `npa package build-certs --root .` followed by `npa package build-certs --root . --check --json`, `npa package check-hashes --root . --json`, and source-free verification. For external imports, intentionally update the pinned imported certificate and its manifest hashes. |
| `reference_checker_rejected` from `package verify-certs --checker reference` | The canonical `.npcert` bytes are not accepted by the independent reference checker. | Treat this as a proof/certificate failure. Fix the theorem or certificate generation path, rebuild certificates, and rerun `npa package verify-certs --root . --checker reference --json`. Do not treat fast-kernel success as reference checker success. |
| `axiom_policy_rejected` or `axiom_report_policy_violation` | A certificate or package axiom report uses an axiom outside the package policy. | Remove the unapproved axiom dependency or update the package axiom policy through review. Rerun reference verification and `npa package axiom-report --root . --check --json`. |
| `axiom_report_stale` or `axiom_report_hash_mismatch` in `generated/axiom-report.json` | The checked axiom report no longer matches the verified certificates. | Regenerate `generated/axiom-report.json` with `npa package axiom-report --root .`, review the diff, then rerun check mode. |
| `theorem_index_stale` or `theorem_index_hash_mismatch` in `generated/theorem-index.json` | The theorem index metadata no longer matches the verified certificates. | Regenerate `generated/theorem-index.json` with `npa package index --root .`, review the diff, then rerun check mode. |
| missing `NPA_CHECKER_EXT_BINARY_PATH`, missing `ci/checker-binaries.json`, or `checker_binary_file_unreadable` in high-trust mode | The high-trust template cannot resolve a pinned built external checker from the fresh checkout. | Add the reviewed `npa-checker-ext` executable and matching checker registry entry. Do not depend on a runner cache or registry network lookup. |
| `checker_binary_hash_mismatch`, `checker_identity_mismatch`, or `checker_build_hash_mismatch` | The external checker bytes or identity metadata differ from runner policy pins. | Treat the binary as changed release evidence. Review build provenance, then update runner policy, checker registry, and checker identity metadata together. |
| `not_verified`, `checker_disagreement`, `status_disagreement`, or normalized comparison failure | Required checker profiles did not all produce the same checked release result. | Inspect the saved external and release audit JSON. Fix the certificate/checker disagreement; do not relabel fast-kernel or reference output as external success. |
| failed `import_certificate_hash` auxiliary result | High-trust import certificate hash evidence is missing, stale, or failed. | Regenerate the auxiliary evidence from the intended import lock and rerun `npa package high-trust --check`. |

Theorem index and axiom report metadata are not proof evidence. They are
review, search, documentation, and release helper artifacts derived from
canonical certificates and source-free checker results.

Allowed uploads include generated package metadata, checked local
certificates, command JSON diagnostics, and plain text summary logs. Default
uploads must exclude AI traces, tactic traces, prompt metadata, secrets,
host-specific caches, and unredacted environment dumps.

## Template Validation

`ci-templates/github-actions/validate-workflows.py` is a local, no-network
validator for the copyable templates. It performs cheap YAML syntax validation
through PyYAML or Ruby's bundled YAML parser, checks that the PR workflow
contains the full-package reference command set, checks that the release
workflow contains both fast and reference verifier commands, checks that the
high-trust workflow contains the external checker and `package high-trust`
command wiring, and rejects unsupported flags:

```text
--changed
--all
--checker external
--registry
--network
--latest
```

The `--checker external` rejection is intentional for the PR and base release
templates only. The opt-in high-trust template validates its own pinned
`npa-checker-ext`, runner policy, checker registry, release audit evidence, and
source-free boundary in the same workflow variant that adds the external job.

Run it from the repository root:

```sh
python3 ci-templates/github-actions/validate-workflows.py
```

`actionlint` remains the preferred GitHub Actions-specific validator when it is
installed, but it is optional because it may not be available on every
developer machine. If `actionlint` is unavailable, run the local validator and
the PyYAML fallback documented in `ci-templates/github-actions/README.md`.

## Later Handoff

CLR-07-03 adds the concrete `npa-package-pr` workflow template under
`ci-templates/github-actions/`. CLR-07-04 adds the concrete
`npa-package-release` workflow template. CLR-07-05 adds
`summarize-npa-diagnostics.py`, and CLR-07-06 adds `validate-workflows.py`.

CLR-09 should copy or reference these files for `npa-mathlib-seed`:

```text
ci-templates/github-actions/npa-package-pr.yml
ci-templates/github-actions/npa-package-release.yml
ci-templates/github-actions/summarize-npa-diagnostics.py
ci-templates/github-actions/validate-workflows.py
```

`ci-templates/github-actions/npa-package-high-trust.yml` is available for a
later high-trust seed release after the seed repository can provide pinned
external checker bytes and release audit evidence. It is not part of the
reference-checker-only default seed gate.

If the seed repository installs workflow YAML under `.github/workflows/`, it
should keep the helper scripts at the path the templates reference, or adjust
the helper path in the copied workflows in the same review.

The seed repository should not copy this repository's local development gates:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts remain local `npa` repository checks, and the templates remain
outside this repository's `.github/workflows`. Until a seed repository opts into
a pinned `npa-checker-ext` binary with runner policy / checker registry and
release audit evidence consumable by `npa package high-trust`, `npa-mathlib-seed`
release evidence is reference-checker-only plus the labeled fast-kernel
verifier; it must not require `--checker external` or emit
`verified_high_trust`.
