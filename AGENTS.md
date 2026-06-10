# AGENTS.md

Working guidelines for agents operating in this repository.

## Project Purpose

NPA is a certificate-first dependently typed proof assistant. It is designed
around the canonical proof certificate that is ultimately checked, not around
convenient higher-level features.

The most important trust boundary is:

```text
Not trusted:
  parser / elaborator / tactic / automation / AI / plugin / theorem search

Trusted:
  small Rust kernel
  canonical certificate
  independent checker
```

## Implementation Policy

- Implement the kernel in Rust.
- Keep the kernel small; do not put I/O, networking, plugin loading, or AI calls
  in it.
- Tactics and elaborators only generate proof terms / certificates; do not treat
  them as the basis of correctness.
- Limit the representation read by the certificate checker to the canonical core
  AST.
- Do not put surface syntax, notation, implicit arguments, typeclass search, or
  holes into the core calculus.
- Make hashes, serialization, and error reporting deterministic.
- As a rule, do not use `unsafe` Rust. If it is necessary, document the reason
  and boundary.

## Documents To Read Before Work

Before making large implementation changes, review the documents for the
relevant phase.

- Implementation baseline: `develop/core-spec-v0.1.md`
- Overall design: `develop/overall-design.md`
- kernel / core calculus: `develop/phase0.md`, `develop/phase1.md`
- certificate: `develop/phase2.md`
- surface language / elaborator: `develop/phase3-human.md`
- tactic: `develop/phase4-human.md`, `develop/phase4-ai.md`
- IDE / API: `develop/phase5-human.md`, `develop/phase5-ai.md`
- standard library: `develop/phase6-human.md`, `develop/phase6-ai.md`
- AI search: `develop/phase7-ai.md`
- independent checker: `develop/phase8-human.md`, `develop/phase8-ai.md`
- advanced features: `develop/phase9-human.md`, `develop/phase9-ai.md`
- AI-assisted proof corpus expansion: `develop/proof-corpus-ai-workflow.md`
- repo-local skill for theorem proving: `.agents/skills/prove-theorem/SKILL.md`

## Rust Kernel Design Rules

- Clearly separate type checking, definitional equality, reduction, universe
  constraints, and inductive checks.
- Treat ASTs as structured data, not string processing.
- Keep binding representations such as de Bruijn indexes / levels aligned with
  the specification and implementation.
- Make the responsibilities and termination of beta / delta / iota / zeta
  reduction explicit.
- Return errors not only as human-facing strings but also as testable structured
  enums.
- Make the kernel API directly callable from tests, and do not make it depend on
  a CLI or server.

## Test Policy

In normal development, do not put the proof corpus on the hot path. For changes
outside the corpus, first use the fast gate that finishes quickly.

```sh
./scripts/check-fast.sh
```

Internally, this runs:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --exclude npa-proof-corpus --all-targets -- -D warnings
cargo test --workspace --exclude npa-proof-corpus -- \
  --skip proof_corpus \
  --skip proof_package \
  --skip package_fast_verifier_ \
  --skip package_reference_verifier_ \
  --skip package_phase8_ \
  --skip package_source_free_
```

The proof corpus is a working staging space, not a public package. During normal
authoring that adds or modifies theorems under `proofs/**`, do not put
package-wide checks on the hot path; use local build / source-free verify and
the lightweight authoring gate. Normal `--build-module` / `--build-modules`
commands do not generate public package metadata; they update only source /
certificate / meta / replay and the untrusted AI theorem index.
`manifest.toml`, `npa-package.toml`, `generated/package-lock.json`,
axiom-report, theorem-index, and publish-plan are explicitly checked and
generated only at public boundaries such as promote / release handoff to
`npa-mathlib`.

Only proved theorems confirmed as `L2 Derived certificate` by roadmap / theorem
card / audit may be promoted to `npa-mathlib`. Do not promote `L0` statements /
conjectures, `L1` evidence packages / interfaces, boundary theorems that assume
the conclusion itself, or candidates with unknown level. If necessary, first
split and prove them as `L2` theorems, then re-evaluate.

Normal checks for proof corpus authoring:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

`check-corpus-authoring.sh` is a lightweight gate that runs only source-free
checks for changed proof corpus modules with `--verified-cache authoring`. The
existing `./scripts/check-corpus.sh` also runs this lightweight authoring gate as
a compatibility wrapper.

Run the heavier proof corpus package gate only when one of the following applies.

- Immediately before promotion to `npa-mathlib`, or to confirm completion of
  promote materialization / closure audit.
- Changes related to promotion, package lock, or artifact generation under
  `tools/proof-corpus/**`.
- When intentionally updating package generated artifacts such as
  `proofs/npa-package.toml`, `proofs/generated/package-lock.json`, axiom-report,
  theorem-index, or publish-plan for promote / release handoff.
- Changes related to canonical encode / decode / hash / import / axiom report
  for certificates.
- Changes related to kernel core semantics, typecheck, reduction, universe, or
  inductives.
- Changes related to the independent checker, package verifier, package lock, or
  artifact validation.
- Changes related to `.npcert` generation / checking compatibility.
- When running a release / high-trust gate.

When applicable, explicitly run the split corpus gate appropriate to the nature
of the change.

```sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

`check-corpus-package.sh` is used for package-wide regressions in the package
verifier, package CLI examples, axiom-report, index, and publish-plan.
`check-corpus-full.sh` combines the lightweight authoring gate and package gate
as the full gate before promote / release / high-trust work.

When authoring additions to the proof corpus, do not run the package/full gate
every time. Prefer the local verification commands in
`develop/proof-corpus-ai-workflow.md`.

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.X::theorem_name proofs/generated/replay-X-theorem.json
```

`--build-module` is an authoring helper that regenerates only the specified
module and import closure from source. During promotion preparation that also
needs to update public package metadata, pass `--package-metadata` explicitly.
`--module` / `--changed-only` are helpers that check checked-in certificates
source-free; to reflect source changes in certificates, run `--build-module`
first.

When the AI theorem index is needed, update it with the following command. This
index is an untrusted sidecar and is not a basis for proof acceptance.

```sh
cargo run -p npa-proof-corpus -- --write-ai-index
```

Around the kernel, add at least the following cases.

- well-typed terms are accepted
- ill-typed terms are rejected
- positive and negative cases for definitional equality
- positive and negative cases for universe constraints
- certificate hashes / import hashes are deterministic
- axiom reports do not grow unintentionally

## Notes When Changing Files

- Do not revert unrelated design documents or user changes.
- For changes that cross phase responsibilities, also update the README or the
  relevant `develop/phase*.md`.
- For changes that expand the kernel trusted base, always document the reason,
  alternatives, and checking boundary.
- In the standard library, do not rely on `sorry`-equivalent behavior or
  unauthorized axioms.
