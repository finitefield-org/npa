# Contributing

NPA is certificate-first. Contributions are welcome, but proof acceptance must
remain based on canonical proof certificates and source-free checker verdicts,
not on convenience layers or repository metadata.

## Trust Boundary

NPA keeps a small trusted base:

```text
trusted:
  canonical .npcert bytes
  Rust kernel / verifier verdict
  source-free reference checker verdict
  deterministic export_hash, certificate_hash, and axiom_report_hash

not trusted:
  parser / elaborator / tactic / automation / AI / plugin / theorem search
  source files / replay files / theorem indexes / publish plans / CI status
  GitHub release pages / registry metadata
```

Do not make parser output, elaborator output, tactic traces, AI traces, theorem
search results, CI status, release metadata, or registry metadata part of proof
acceptance.

## Local Gates

For ordinary development, run the fast gate first:

```sh
./scripts/check-fast.sh
```

This is the default hot-path check for changes outside the proof corpus and
certificate/checker compatibility surface.

Run the corpus gate only when a change affects one of these areas:

- `proofs/**`
- `tools/proof-corpus/**`
- canonical certificate encode, decode, hash, import, or axiom report behavior
- kernel core semantics, typecheck, reduction, universe, or inductive behavior
- independent checker, package verifier, package lock, or artifact validation
- `.npcert` generation or verification compatibility
- release or high-trust evidence

For those changes, run:

```sh
./scripts/check-corpus.sh
```

When adding or editing proof corpus theorems, use the targeted authoring
commands from `develop/proof-corpus-ai-workflow.md` for the normal repair loop.
Do not run the full corpus gate after every proof attempt; reserve it for batch
boundaries, commit/push readiness, release handoff, or certificate/package/checker
compatibility changes.

## Certificate Compatibility

Changes around `.npcert` bytes, canonical encoding, declaration hashes, import
hashes, axiom reports, package locks, or source-free verification have a larger
blast radius. Include focused tests for deterministic hashes and for both
accepted and rejected cases.

Kernel-adjacent changes should include tests for:

- well-typed terms that pass
- ill-typed terms that are rejected
- definitional equality positive and negative cases
- universe constraint positive and negative cases
- deterministic certificate hash and import hash behavior
- axiom reports that do not grow unexpectedly

## Unsafe Rust

Do not use `unsafe` Rust by default. If `unsafe` is necessary, document why it
is necessary, what boundary contains it, and what safe alternatives were
rejected. Keep trusted-kernel changes small and directly testable.

## Working Tree Etiquette

Do not revert unrelated changes. If you see local modifications that are not
part of your task, treat them as user or teammate work and leave them alone.
If those changes affect your task, work with them rather than discarding them.

Keep changes scoped to the task. If a change crosses a phase boundary or
widens the trusted base, update the relevant design document and explain the
trust-boundary impact.

## Package Authoring

External theorem package checks use the installed `npa` binary with an explicit
package root:

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

These commands produce deterministic diagnostics and release metadata. They do
not make source files, theorem indexes, publish plans, CI status, or GitHub
release pages trusted proof evidence.
