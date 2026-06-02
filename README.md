# Nano Proof Auditor (NPA)

NPA is an experimental, certificate-first proof assistant and verification
toolchain for dependent proofs.

The project is designed around a small trusted base. Surface syntax,
elaboration, tactics, automation, theorem search, plugins, and AI systems may
help produce proof candidates, but they are not trusted proof evidence. The
object that matters is the canonical proof certificate checked by the Rust
kernel and source-free checkers.

```text
untrusted:
  parser / elaborator / tactic / automation / AI / plugin / theorem search
  source files / replay files / theorem indexes / publish plans / CI status
  GitHub release pages / registry metadata

trusted:
  canonical .npcert bytes
  Rust kernel / verifier verdict
  source-free reference checker verdict
  deterministic export_hash, certificate_hash, and axiom_report_hash
```

NPA is not a production replacement for Lean or Rocq. It is a research and
implementation repository for a proof-certificate-centered toolchain.

## Current Status

The current SRA-02-compatible toolchain reference for external theorem package
repositories is:

```text
NPA_GIT_TAG = v0.1.1
RUST_TOOLCHAIN_VERSION = 1.95.0
```

The earlier `v0.1.0` tag is historical and should not be used as the current
external package toolchain pin.

The first split theorem package repository is:

- `npa-std`: <https://github.com/finitefield-org/npa-std>

## Build From Source

Install the pinned Rust toolchain and build the CLI:

```sh
rustup toolchain install 1.95.0 --profile minimal
cargo +1.95.0 build -p npa-cli
```

The installed binary name is `npa`. From the repository build output:

```sh
target/debug/npa --version
```

Expected output for the current toolchain ref:

```text
npa 0.1.1
```

## Package Verification Quick Start

External theorem libraries use the `npa package ...` command family with an
explicit package root:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package verify-certs --root . --checker reference --json
npa package check-hashes --root . --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

For release-ready packages that check in `generated/publish-plan.json`, also
run:

```sh
npa package publish-plan --root . --check --json
```

For local development in this repository, run the same commands through
`cargo` or the built `target/debug/npa` binary:

```sh
cargo run -p npa-cli -- package check --root fixtures/npa-std --json
cargo run -p npa-cli -- package build-certs --root fixtures/npa-std --check --json
cargo run -p npa-cli -- package verify-certs --root fixtures/npa-std --checker reference --json
cargo run -p npa-cli -- package check-hashes --root fixtures/npa-std --json
```

Package metadata, theorem indexes, publish plans, and CI output are deterministic
review and release metadata. They are not proof evidence. Downstream users must
still verify hash-pinned certificate bytes with a source-free checker.

## Repository Layout

```text
.
├── crates/
│   ├── npa-kernel/       trusted kernel core
│   ├── npa-cert/         canonical certificate encoding and checking handoff
│   ├── npa-checker-ref/  source-free reference checker
│   ├── npa-package/      package manifest, lock, artifact, and report tooling
│   ├── npa-cli/          installed `npa` command
│   ├── npa-frontend/     untrusted surface-language frontend
│   ├── npa-tactic/       untrusted tactic/proof-state layer
│   └── npa-api/          untrusted API and orchestration layer
├── checkers/
│   └── npa-checker-ext/  clean-room external checker prototype
├── ci-templates/
│   └── github-actions/  copyable external package workflows
├── docs/                user-facing documentation and package-author guides
├── develop/             implementation specs, phase plans, and release evidence
├── fixtures/            package fixtures and standalone-repo materialization
├── proofs/              repository proof corpus
├── scripts/             local verification gates
└── tools/
    └── proof-corpus/    proof-corpus tooling
```

## Documentation

Start with the user documentation:

- [NPA User Documentation](docs/README.md)

Public package-author and toolchain references:

- [Toolchain Reference v0.1.1](docs/npa-toolchain-reference-v0.1.1.md)
- [External Theorem Library CI](docs/external-theorem-library-ci.md)
- [GitHub Actions CI Templates](ci-templates/github-actions/README.md)

Developer-facing specs, release evidence, internal planning, and Japanese
development notes are routed from [Development Documentation](develop/README.md).

The in-repo Phase 6 standard-library design documents the MVP release modules
`Std.Logic`, `Std.Nat`, `Std.List`, and `Std.Algebra.Basic`. The current SRA-02
external package fixture path is the split `npa-std` package.
Phase 6 release/build artifact profiles include `std.nat.mvp`, `std.list.mvp`,
and `std.all.mvp`; source layout fixtures remain authoring and debug context,
not trusted proof evidence.

## Local Development Gates

For ordinary development, start with the fast gate:

```sh
./scripts/check-fast.sh
```

Run the corpus gate only for changes that affect proof corpus files, proof
corpus tooling, canonical certificate compatibility, kernel semantics,
independent checkers, package verification, package locks, artifact validation,
or release/high-trust evidence:

```sh
./scripts/check-corpus.sh
```

For contribution policy and the full local-gate checklist, see
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

NPA is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 Finite Field K.K. See [NOTICE](NOTICE).
