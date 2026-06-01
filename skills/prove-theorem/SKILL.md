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
2. Read `AGENTS.md` and, for details, `doc/proof-corpus-ai-workflow.md` or `references/npa-proof-corpus.md`.
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

9. Before finishing, run `./scripts/check-fast.sh`. Run `./scripts/check-corpus.sh` only when the corpus gate conditions in `AGENTS.md` apply or the user asks for the full gate.

## Guardrails

- Prefer adding a small lemma in the same narrow module over widening imports.
- Rebuild downstream modules only if a changed foundational module changes export hashes.
- Keep sidecars deterministic and untrusted; final acceptance is the `.npcert` verification result.
- If the theorem target or intended statement is ambiguous, ask one concise clarification before editing.

## Reference

Read `references/npa-proof-corpus.md` when you need the command cookbook, target resolution details, or repair loop guidance.
