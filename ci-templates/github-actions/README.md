# GitHub Actions Template Location

This directory is reserved for copyable external theorem library CI templates.
It is intentionally outside `.github/workflows` so these files are not active
local workflows for the `npa` repository.

Template files:

```text
npa-package-pr.yml          available
npa-package-release.yml     available
```

CLR-07-03 adds the PR template. CLR-07-04 adds the release template. The
contract source is:

```text
doc/external-theorem-library-ci.md
```

External theorem libraries should copy templates from this directory once later
CLR-07 milestones add them. They should not copy local `npa` repository gates
such as:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts are repository development gates, not external theorem library CI.

## Pinned Setup Inputs

The base templates must fail unless the `npa` CLI source is explicit and
pinned. They must not infer a floating `latest` version.

Supported inputs:

```text
NPA_BINARY_PATH
  Path to an existing executable `npa` binary. This is the baseline setup mode
  until concrete installer templates are added.

NPA_VERSION
  Exact release version or release tag for a later release-download strategy.
  `latest` is invalid.

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
`NPA_GIT_COMMIT` from GitHub repository variables. For CLR-07-03 it accepts
`NPA_BINARY_PATH` only and fails clearly if a later installer-mode variable is
selected.

`npa-package-release.yml` uses the same pinned source variables. For CLR-07-04
it also accepts `NPA_BINARY_PATH` only and fails clearly if a later
installer-mode variable is selected.

When Rust is used to build `npa-cli`, templates must use a checked-in
`rust-toolchain.toml` or exact `RUST_TOOLCHAIN_VERSION`, then print:

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
actionlint ci-templates/github-actions/npa-package-pr.yml
actionlint ci-templates/github-actions/npa-package-release.yml
```

If `actionlint` is unavailable, use a local YAML parser as a cheap syntax
fallback:

```sh
python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1], encoding="utf-8"))' ci-templates/github-actions/npa-package-pr.yml
python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1], encoding="utf-8"))' ci-templates/github-actions/npa-package-release.yml
```

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

The CLR-06 publish-plan check is optional. Set `NPA_ENABLE_PUBLISH_PLAN` to
`true` and check in `generated/publish-plan.json` to run:

```sh
npa package publish-plan --root . --check --json
```

Base CLR-07 templates must not use `--changed`, `--all`, `--checker external`,
`--registry`, `--network`, or `--latest`. They must not contact an NPA package
registry, use hidden package caches, or resolve imports by implicit latest
version.

CI output is not proof evidence. The proof boundary remains canonical
certificate artifacts plus source-free checker or kernel verifier verdicts.
