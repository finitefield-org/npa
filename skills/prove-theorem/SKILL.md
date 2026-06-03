---
name: prove-theorem
description: Prove a named theorem in this NPA repository's proof corpus using the fast AI-oriented workflow. Use when the user says "prove-theorem THEOREM_NAME", "prove-theorem THEOREM_NAME を証明", asks to add or prove a theorem in `proofs/**` or `tools/proof-corpus/**`, or wants a quick local proof loop without running the full proof corpus gate on every attempt.
---

# Prove Theorem

## Overview

Use the proof-corpus authoring fast path: resolve the theorem target, add or repair the smallest proof-corpus module, rebuild only that module plus import closure, then source-free verify the checked-in certificate.

Trust boundary: never treat AI, replay, metadata, theorem index, or tactic output as proof evidence. Accept a proof only after canonical certificate verification succeeds.

## Workflow

1. Parse the request as a theorem name plus optional statement/module hints.
2. Read `AGENTS.md` and, for details, `develop/proof-corpus-ai-workflow.md` or `references/npa-proof-corpus.md`.
3. Locate the target:
   - Search `proofs/generated/ai-theorem-index.json` for an existing theorem name.
   - Search `tools/proof-corpus/src/main.rs` for `name: "THEOREM_NAME"` and nearby module constants.
   - If the name is new, choose the smallest suitable `Proofs.Ai.*` module and keep imports minimal.
4. Prove using the cheap order:

```text
exact local hypothesis
exact known theorem
rw / simp-lite
apply theorem + subgoal generation
induction-nat
explicit proof term
new lemma
```

5. Edit the proof corpus:
   - For generated modules, update the relevant `TheoremArtifact` list in `tools/proof-corpus/src/main.rs`.
   - For checked-in source modules, update `proofs/**/source.npa` and keep the artifact metadata in `tools/proof-corpus/src/main.rs` aligned.
   - Do not add `sorry`, unapproved axioms, or trusted-base changes to make a theorem pass.
6. Rebuild only the target module:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
```

7. Verify locally:

```sh
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
```

8. If verification fails, use the structured diagnostic to repair the smallest proof term/import change. Optionally emit a focused replay for the failing declaration:

```sh
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.X::theorem_name /tmp/replay.json
```

9. Before finishing an authoring turn, run the local proof checks above and `./scripts/check-fast.sh` when appropriate. Do not run `./scripts/check-corpus.sh` as part of every proof attempt or repair loop.

## Gate Policy

During theorem authoring, the default verification loop is local:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-fast.sh
```

`./scripts/check-corpus.sh` is intentionally expensive. Run it once at a batch boundary, before commit/push or release handoff, when proof-corpus infrastructure, certificate/package/checker compatibility, or kernel semantics changed, or when the user explicitly asks for the full corpus gate. If it is skipped, say so in the final response with the local checks that did run.

## Guardrails

- Prefer adding a small lemma in the same narrow module over widening imports.
- Rebuild downstream modules only if a changed foundational module changes export hashes.
- Keep sidecars deterministic and untrusted; final acceptance is the `.npcert` verification result.
- If the theorem target or intended statement is ambiguous, ask one concise clarification before editing.

## Reference

Read `references/npa-proof-corpus.md` when you need the command cookbook, target resolution details, or repair loop guidance.
