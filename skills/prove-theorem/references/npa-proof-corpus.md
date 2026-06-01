# NPA Proof Corpus Fast Proving Reference

## Target Resolution

Use these in order:

```sh
rg -n '"theorem": "THEOREM_NAME"|"focused_replay_spec": ".*::THEOREM_NAME"' proofs/generated/ai-theorem-index.json
rg -n 'name: "THEOREM_NAME"|const .*_THEOREMS|module: "Proofs\.Ai\.' tools/proof-corpus/src/main.rs
rg -n 'theorem THEOREM_NAME|def THEOREM_NAME|inductive THEOREM_NAME' proofs tools
```

If the theorem already exists, do not re-prove it first. Verify its module:

```sh
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
```

If the theorem is new and no module is specified, choose the narrowest existing module whose imports already cover the statement's constants. Prefer `Proofs.Ai.Basic`, `Proofs.Ai.Eq`, `Proofs.Ai.Nat`, `Proofs.Ai.Prop`, or an existing topic module before creating a new module.

## Editing Rules

Most proof corpus modules are generated from `tools/proof-corpus/src/main.rs`; add a `TheoremArtifact` with `name`, optional `universe_params`, `statement`, and `proof`.

Some modules are checked-in source modules. For those, edit `proofs/**/source.npa` and keep the corresponding artifact list in `tools/proof-corpus/src/main.rs` aligned so metadata, AI index, and replay remain useful.

Never introduce:

- `sorry`-like placeholders
- unapproved custom axioms
- kernel/checker/trusted-base changes just to pass the theorem
- broad imports that pull in unrelated axioms or features

## Fast Loop

Use this loop while proving:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
```

Then check the changed set:

```sh
cargo run -p npa-proof-corpus -- --changed-only
```

For larger changed sets:

```sh
cargo run -p npa-proof-corpus -- --changed-only --shard 0/2
cargo run -p npa-proof-corpus -- --changed-only --failures-out /tmp/npa-proof-failures.json
```

For a single failing declaration:

```sh
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.X::theorem_name /tmp/npa-replay.json
```

Regenerate AI retrieval metadata after adding/renaming theorem artifacts:

```sh
cargo run -p npa-proof-corpus -- --write-ai-index
```

## Proof Search Heuristic

Try candidates in this order:

1. Exact local hypothesis.
2. Exact existing theorem from `proofs/generated/ai-theorem-index.json`.
3. Rewriting or simp-lite style theorem already in scope.
4. Apply a theorem and solve generated subgoals.
5. Nat induction only when the statement structurally requires it.
6. Explicit proof term from local constants and imports.
7. New small lemma in the same module.

Prefer the shortest proof term that keeps imports unchanged and does not increase the axiom report.

## Completion Checklist

Before final response:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-fast.sh
```

Run the full corpus gate only when required by `AGENTS.md` or requested:

```sh
./scripts/check-corpus.sh
```

Report any skipped full corpus gate explicitly.
