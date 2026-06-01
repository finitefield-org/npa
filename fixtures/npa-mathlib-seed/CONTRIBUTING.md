# Contributing To npa-mathlib-seed

This fixture models a standalone theorem-library repository. Treat
`fixtures/npa-mathlib-seed/` as the package root; package paths in
`npa-package.toml` are relative to that root and must keep working after the
directory is copied out of the `npa` checkout.

## Theorem-Only Changes

For CLR-09-02, contributors should validate the package manifest and layout:

```sh
npa package check --root . --json
```

Before the `npa` binary is installed, the equivalent workspace command is:

```sh
cargo run -p npa-cli -- package check --root fixtures/npa-mathlib-seed --json
```

Later CLR-09 milestones add certificate rebuild checks, source-free
verification, generated axiom reports, theorem indexes, publish plans, and CI.

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
CLR-09-02.

## Trust Boundary

Source files, replay files, metadata, package manifests, and future generated
indexes are useful contributor artifacts, but they are not trusted proof
evidence. Acceptance remains based on canonical certificates plus source-free
checker verdicts.
