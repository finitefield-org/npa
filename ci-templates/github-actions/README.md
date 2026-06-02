# GitHub Actions Template Location

This directory is reserved for copyable external theorem library CI templates.
It is intentionally outside `.github/workflows` so these files are not active
local workflows for the `npa` repository.

Template files:

```text
npa-package-pr.yml          available
npa-package-release.yml     available
npa-package-high-trust.yml  available
setup-pinned-npa.sh         available
summarize-npa-diagnostics.py available
validate-workflows.py       available
```

CLR-07-03 adds the PR template. CLR-07-04 adds the base release template.
CLR-08-06 adds the opt-in high-trust release template. The contract source is:

```text
doc/external-theorem-library-ci.md
```

External theorem libraries should copy or reference templates from this
directory. They should not copy local `npa` repository gates such as:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts are repository development gates, not external theorem library CI.
CLR-09 should use `npa-package-pr.yml`, `npa-package-release.yml`,
`setup-pinned-npa.sh`, `summarize-npa-diagnostics.py`, and
`validate-workflows.py` for the reference-checker-only seed release. Copy
`npa-package-high-trust.yml` only after the seed repository also provides the
pinned high-trust checker binary, runner policies, checker registry, and
release audit evidence. If a seed repository installs workflow YAML under
`.github/workflows/`, keep helper scripts at the path referenced by the
templates or update the copied workflow paths in the same change.

## Pinned Setup Inputs

The base templates must fail unless the `npa` CLI source is explicit and
pinned. They must not infer a floating `latest` version.

Supported inputs:

```text
NPA_BINARY_PATH
  Path to an existing executable `npa` binary.

NPA_VERSION
  Exact release version or release tag for a later release-download strategy.
  `latest` is invalid. The current SRA-01 setup script rejects this mode until
  release-download artifacts are added.

NPA_GIT_TAG
  Exact immutable Git tag for building `npa-cli`.

NPA_GIT_COMMIT
  Full Git commit SHA for building `npa-cli`.
```

Exactly one of `NPA_BINARY_PATH`, `NPA_VERSION`, `NPA_GIT_TAG`, or
`NPA_GIT_COMMIT` must be set. If none or multiple are set, setup fails before
running package commands.

Baseline binary-path setup:

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

`npa-package-pr.yml` reads `NPA_BINARY_PATH`, `NPA_VERSION`, `NPA_GIT_TAG`, and
`NPA_GIT_COMMIT` from GitHub repository variables. SRA-01 adds Git-ref setup:
set exactly one of `NPA_BINARY_PATH`, `NPA_GIT_TAG`, or `NPA_GIT_COMMIT`.

`npa-package-release.yml` and `npa-package-high-trust.yml` use the same pinned
source variables. For the first public toolchain ref, external repositories may
set:

```text
NPA_GIT_TAG = v0.1.0
RUST_TOOLCHAIN_VERSION = 1.95.0
```

Alternatively, use `NPA_GIT_COMMIT` with a full lowercase 40-hex commit SHA.

When Rust is used to build `npa-cli`, `setup-pinned-npa.sh` uses exact
`RUST_TOOLCHAIN_VERSION`, defaulting to `1.95.0`, then prints:

```sh
npa --version
cargo --version
rustc --version
```

Fetching the pinned `npa` implementation or pinned Rust toolchain is tool
setup. It is separate from theorem package dependency resolution and must not
fetch theorem packages, package imports, registry metadata, or hidden package
cache entries.

The base contract uses full-package package commands:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

`npa-package-pr.yml` is the PR-mode template. It runs the full package as the
conservative changed-module fallback and saves command JSON to:

```text
ci-output/package-check.json
ci-output/build-certs.json
ci-output/check-hashes.json
ci-output/verify-certs-reference.json
ci-output/axiom-report.json
ci-output/index.json
```

Validate workflow syntax with `actionlint` when it is installed:

```sh
actionlint ci-templates/github-actions/*.yml
```

If `actionlint` is unavailable, use a local YAML parser as a cheap syntax
fallback. The validator uses PyYAML when available and Ruby's bundled YAML
parser otherwise:

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

The validator checks that PR and release workflows still contain the required
full-package package commands, and that the opt-in high-trust workflow keeps
the external checker, release audit validation, and `verified_high_trust`
commands wired to explicit policy and checker-registry inputs. It also fails if
any workflow adds unsupported changed/all selectors, registry or network
package resolution, or implicit latest package resolution. External checker
mode remains forbidden in the PR and base release workflows.
Reference / external checker benchmark collection is likewise not a PR hot
path requirement. The opt-in high-trust path may attach an external checker
benchmark summary to the release audit bundle and fail release/high-trust
policy from that regression evidence, but the benchmark summary does not change
proof validity or any checker verdict.

Release templates may add the fast verifier as a labeled non-reference gate:

```sh
npa package verify-certs --root . --checker fast --json
```

`npa-package-release.yml` runs package artifact checks, a fast-kernel
source-free verification job, and a reference checker source-free verification
job. Fast-kernel output is labeled fast-kernel and is not reported as reference
checker success. The template uploads checked package artifacts, certificate
artifacts, and JSON diagnostics; it does not upload AI traces, prompt metadata,
host-specific caches, or environment dumps.

Failure summaries use package command JSON diagnostics when
`ci-templates/github-actions/summarize-npa-diagnostics.py` is copied with the
workflow. The summary table is deterministic and intentionally limited to:

```text
file | command | status | exit_code | kind | reason_code | module | path | expected_hash | actual_hash
```

The table uses package-relative paths and omits raw runner stderr, absolute host
paths, environment variables, and caches.

Contributor failure mapping:

| Diagnostic | Action |
| --- | --- |
| `source_hash_mismatch` | Review the source change, then update package metadata through the normal package update flow. Do not edit expected hashes blindly. |
| `certificate_hash_mismatch` or `certificate_file_hash_mismatch` | Rebuild/check certificates explicitly, review the certificate and package-lock diffs, then rerun hash and verifier checks. |
| `reference_checker_rejected` | Treat the `.npcert` as rejected proof evidence; fix the theorem or certificate generation path and rerun reference verification. |
| `axiom_policy_rejected` or `axiom_report_policy_violation` | Remove the unapproved axiom dependency or update package axiom policy through review. |
| `axiom_report_stale` or `axiom_report_hash_mismatch` | Regenerate `generated/axiom-report.json`, review the diff, then rerun `npa package axiom-report --root . --check --json`. |
| `theorem_index_stale` or `theorem_index_hash_mismatch` | Regenerate `generated/theorem-index.json`, review the diff, then rerun `npa package index --root . --check --json`. |

Theorem index and axiom report metadata are not proof evidence; they are derived
review/search artifacts.

The CLR-06 publish-plan check is optional. Set `NPA_ENABLE_PUBLISH_PLAN` to
`true` and check in `generated/publish-plan.json` to run:

```sh
npa package publish-plan --root . --check --json
```

Base CLR-07 templates must not add unsupported changed/all selectors, external
checker mode, registry or network package resolution, or implicit latest
package resolution. They must not contact an NPA package registry, use hidden
package caches, or resolve imports by implicit latest version.

CI output is not proof evidence. The proof boundary remains canonical
certificate artifacts plus source-free checker or kernel verifier verdicts.
Base CLR-07 templates remain reference-checker-only for PR acceptance and do
not use registry access, a package solver, a binary cache, or implicit latest
package resolution. Release base mode adds only the labeled fast-kernel gate.

`npa-package-high-trust.yml` is the implemented opt-in high-trust extension,
not a base-template requirement. It adds separate jobs for external checker
source-free verification, high-trust-reference release audit validation, and
`verified_high_trust` generation/check. It requires all of these inputs to be
present before it runs verifier commands:

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

The template also runs:

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

That extension requires a pinned built `npa-checker-ext` executable in the fresh
checkout, plus runner-owned policy and checker registry entries that validate
binary identity and hash. The source-free boundary stays the same: the checker
path reads package metadata, package lock, canonical certificates, import
certificates, runner policy, checker registry, checker executable bytes, and
axiom policy, not `.npa` source, replay/meta files, theorem index, AI traces,
registry network data, hidden caches, or plugins.

`verified_high_trust` is generated by `npa package high-trust` only after
external and high-trust-reference release audit evidence exists. It must not be
emitted from reference-checker-only release evidence.
External checker benchmark rows are release audit metadata: they record checker
identity, certificate hash, module, timing, timeout/memory limits, result hash,
and run artifact hash so release audit can link the benchmark to the saved
checker result.

High-trust failure mapping:

| Diagnostic | Action |
| --- | --- |
| missing `NPA_CHECKER_EXT_BINARY_PATH`, missing `ci/checker-binaries.json`, or `checker_binary_file_unreadable` | Pin a built `npa-checker-ext` executable in the fresh checkout and add the matching checker registry entry. Do not fall back to a runner cache. |
| `checker_binary_hash_mismatch`, `checker_identity_mismatch`, or `checker_build_hash_mismatch` | Treat the checker binary as changed release evidence. Review the binary build provenance, then update runner policy and checker identity metadata in the same review. |
| `not_verified`, `checker_disagreement`, `status_disagreement`, or normalized comparison failure | Inspect the saved external and release audit JSON. Fix the certificate/checker disagreement; do not relabel fast-kernel or reference output as external success. |
| failed external checker benchmark policy | Treat it as release/high-trust regression evidence. Review thresholds and machine provenance, then update policy or evidence without changing the saved checker verdicts. |
| failed `import_certificate_hash` auxiliary result | Regenerate the high-trust import certificate hash auxiliary evidence from the intended import lock and rerun `npa package high-trust --check`. |
