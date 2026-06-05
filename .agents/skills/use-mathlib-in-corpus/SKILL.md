---
name: use-mathlib-in-corpus
description: Migrate NPA proof corpus authoring to consume promoted npa-mathlib modules as hash-pinned source-free package imports. Use when the user asks to dogfood npa-mathlib from proofs/, replace Proofs.Ai.* imports with Mathlib.* imports, add npa-mathlib import pins/vendor certificates to proofs/npa-package.toml, or update corpus modules after a closure has been promoted.
---

# Use Mathlib In Corpus

## Scope

Use this skill to make `proofs/` consume public `npa-mathlib` certificates.
Do not use it to promote new material into `npa-mathlib`; use
`find-promotable-closure`, `judge-promote-to-mathlib`, and `closure-audit` for
that path.

The trust boundary stays certificate-first:

- import checked-in `.npcert` bytes through hash-pinned package imports;
- do not depend on `npa-mathlib` source, replay, meta, Git branch state, or a
  hidden cache as proof evidence;
- keep `Proofs.Ai.*` as staging namespace and `Mathlib.*` as stable public
  package namespace.

## Workflow

1. Locate repositories. Prefer current checkout as `npa` and sibling
   `../npa-mathlib` as the public package.
2. Read current policy before editing:
   - `AGENTS.md`
   - `develop/proof-corpus-ai-workflow.md`
   - `proofs/README.md`
   - `../npa-mathlib/docs/namespace-policy.md`
   - `fixtures/npa-mathlib-downstream/npa-package.toml`
3. Inspect current state:

```sh
git status --short
.agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh scan \
  --mathlib-root ../npa-mathlib
```

Use `--prefix Proofs.Ai.NumberTheory.` or another namespace to focus.

4. Choose a small migration unit:
   - new dogfood module importing one or a few `Mathlib.*` modules; or
   - one existing corpus module whose old `Proofs.Ai.*` imports have promoted
     `Mathlib.*` counterparts.
5. Pin public imports before rewriting source:

```sh
.agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh pin \
  Mathlib.Logic.Basic \
  --mathlib-root ../npa-mathlib \
  --apply
```

This copies only certificate bytes into `proofs/vendor/npa-mathlib/**` and adds
missing `[[imports]]` stanzas to `proofs/npa-package.toml`. Without `--apply`,
the command prints the planned stanza and copy operation.

6. Rewrite selected `source.npa` imports manually and narrowly, for example:

```text
import Proofs.Ai.Basic
```

to:

```text
import Mathlib.Logic.Basic
```

Do not bulk-rewrite all corpus modules in one change. Do not introduce local
`Mathlib.*` modules inside `proofs/`.

7. Rebuild and verify the smallest affected closure:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

When `proofs/npa-package.toml`, vendored certificate bytes, package lock,
package hash behavior, or package import handling changed, also run the package
boundary gate:

```sh
./scripts/check-corpus-package.sh
```

## Tooling

The bundled helper is read-only unless `pin --apply` is passed:

```sh
.agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh scan
.agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh pin Mathlib.X --apply
```

The helper implementation is Rust and the shell wrapper compiles it with
`rustc` into a temporary binary before running it.

`scan` compares `proofs/npa-package.toml` with `../npa-mathlib/npa-package.toml`
by export hash and declaration signature, then reports `Proofs.Ai.*` imports
that can likely become `Mathlib.*` imports. Treat the output as a migration
hint, not proof evidence.

`pin` reads the public manifest, copies the public certificate into
`proofs/vendor/npa-mathlib/<module path>/certificate.npcert`, and inserts a
hash-pinned package import stanza when missing.

## Output Format

Report:

```text
Changed:
- ...

Verification:
- ...

Remaining risks:
- ...

Next action:
- ...
```

If a module cannot be migrated, give the exact blocker: missing local
`npa-mathlib`, no public counterpart, missing certificate, changed public hash,
large import closure, failing source-free verification, or package gate failure.
