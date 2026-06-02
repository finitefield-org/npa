# NPA Toolchain Reference v0.1.0

This is the SRA-01 toolchain reference for external theorem package
repositories.

## Stable Ref

Use the Git tag:

```text
v0.1.0
```

External theorem repositories should set exactly one pinned `npa` source. The
recommended SRA-01 setting is:

```text
NPA_GIT_TAG = v0.1.0
RUST_TOOLCHAIN_VERSION = 1.95.0
```

`NPA_GIT_COMMIT` is also supported when a repository wants to pin the full
40-hex commit SHA instead of a tag. `NPA_BINARY_PATH` remains supported for
runner-local binary provisioning. `NPA_VERSION` is reserved for a later
release-download mode and is rejected by the current setup script.

## Setup Contract

Copy these files into external theorem repositories:

```text
ci-templates/github-actions/npa-package-pr.yml
ci-templates/github-actions/npa-package-release.yml
ci-templates/github-actions/setup-pinned-npa.sh
ci-templates/github-actions/summarize-npa-diagnostics.py
```

Copy `ci-templates/github-actions/npa-package-high-trust.yml` only when the
repository also supplies CLR-08 pinned external checker binaries, runner
policies, checker registry data, and release audit evidence.

The setup script fetches only the pinned `npa` implementation and exact Rust
toolchain needed to build `npa-cli`. It must not fetch theorem package
dependencies, package imports, registry metadata, or hidden package caches.

## Package Commands

The reference-checker PR gate uses:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

The base release gate additionally records:

```sh
npa package verify-certs --root . --checker fast --json
```

`publish-plan` remains release metadata and is enabled by setting
`NPA_ENABLE_PUBLISH_PLAN=true` when `generated/publish-plan.json` is checked in:

```sh
npa package publish-plan --root . --check --json
```

## Trust Boundary

This reference is reference-checker-only. It does not produce
`verified_high_trust`.

Trusted proof evidence remains:

- canonical `.npcert` bytes
- Rust kernel / verifier verdict
- source-free reference checker verdict
- deterministic `export_hash`, `certificate_hash`, and `axiom_report_hash`

Untrusted helper data remains:

- source files
- replay and meta files
- theorem indexes
- publish plans
- CI status
- Git tags and release pages
- registry seed entries
- future registry or API responses

## Local Verification

The SRA-01 local gate is:

```sh
cargo build -p npa-cli
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib --json
python3 ci-templates/github-actions/validate-workflows.py
cargo test -q -p npa-cli package_cli_args
tmpdir="$(mktemp -d)"
GITHUB_PATH="$tmpdir/github-path" RUNNER_TEMP="$tmpdir" GITHUB_WORKSPACE="$PWD" \
  NPA_BINARY_PATH=target/debug/npa \
  bash ci-templates/github-actions/setup-pinned-npa.sh
./scripts/check-fast.sh
```
