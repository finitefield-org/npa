# npa-mathlib Fixture

Visibility: public example fixture.

This fixture models the first public `npa-mathlib` theorem-library package.
It is derived from the earlier seed fixture's Layer 0 modules, but uses the
public `Mathlib.*` namespace and package name `npa-mathlib`.

The fixture is reference-checker-only. Source, replay, metadata, generated
theorem indexes, publish metadata, CI status, and future registry metadata are
not proof evidence. Proof acceptance comes from canonical certificate artifacts
and local source-free checker verification.

Layer 0 modules:

- `Mathlib.Logic.Basic`
- `Mathlib.Logic.Prop`
- `Mathlib.Logic.Eq`
- `Mathlib.Data.Nat.Basic`
- `Mathlib.Core.Reduction`

The only external imports are hash-pinned `npa-std` certificate artifacts:

- `Std.Logic.Eq`
- `Std.Nat.Basic`

Baseline checks from the repository root:

```sh
cargo run -q -p npa-cli -- package check --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root fixtures/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root fixtures/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root fixtures/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root fixtures/npa-mathlib --check --json
```
