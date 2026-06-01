# Contributing To npa-mathlib-seed

This fixture models a standalone theorem-library repository. Treat
`fixtures/npa-mathlib-seed/` as the package root; package paths in
`npa-package.toml` are relative to that root and must keep working after the
directory is copied out of the `npa` checkout.

## Theorem-Only Changes

For CLR-09-03, contributors should run the base package command sequence:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
npa package publish-plan --root . --check --json
```

Before the `npa` binary is installed, use the parent workspace command shape:

```sh
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json
```

Check mode must fail when generated artifacts are stale. If a theorem-only
change intentionally updates certificates or hashes, regenerate the affected
artifacts first and then rerun the sequence above.

## Package Boundary

The seed package must not use absolute local filesystem paths or hidden paths
back into the parent `npa` repository. Standard-library dependencies are
declared as hash-pinned `npa-std` imports and resolved through vendored
certificate artifacts under `vendor/npa-std/`.

The base seed contains only:

- `Proofs.Ai.Basic`
- `Proofs.Ai.Prop`
- `Proofs.Ai.Eq`
- `Proofs.Ai.Nat`
- `Proofs.Ai.Reduction`

Adding larger proof-corpus modules, changing the axiom policy, adding
`Eq.rec`-dependent modules, or renaming the public namespace is outside
the current seed command wiring scope.

## Trust Boundary

Source files, replay files, metadata, package manifests, generated indexes,
publish plans, and CI are useful contributor artifacts, but they are not
trusted proof evidence. Acceptance remains based on canonical certificates plus
source-free checker verdicts.
