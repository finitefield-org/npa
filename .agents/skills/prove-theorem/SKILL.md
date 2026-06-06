---
name: prove-theorem
description: Prove a named theorem in this NPA repository's proof corpus using the fast AI-oriented workflow. Use when the user says "prove-theorem THEOREM_NAME", "prove-theorem THEOREM_NAME を証明", asks to add or prove a theorem in `proofs/**` or `tools/proof-corpus/**`, or wants a quick local proof loop without running the full proof corpus gate on every attempt.
---

# Prove Theorem

## Overview

Use the proof-corpus authoring fast path: resolve the theorem target, add or repair the smallest proof-corpus module, rebuild only that module plus import closure, then source-free verify the checked-in certificate.

Trust boundary: never treat AI, replay, metadata, theorem index, or tactic output as proof evidence. Accept a proof only after canonical certificate verification succeeds.

## Basic Procedure

1. Interpret the theorem target:
   - Parse the theorem name, statement, and any module hint.
   - Read `AGENTS.md` and, for details, `develop/proof-corpus-ai-workflow.md` or `references/npa-proof-corpus.md`.
   - If the theorem already exists, do not reprove it. Locate it by searching `proofs/generated/ai-theorem-index.json`, `tools/proof-corpus/src/main.rs`, and `proofs/**/source.npa`.
2. For a new theorem, choose the smallest suitable `Proofs.Ai.*` module:
   - Do not widen imports by default.
   - If an existing module's imports already provide the constants in the statement, add the theorem there.
3. Explore proof candidates in the cheap order:

```text
exact local hypothesis
exact known theorem
rw / simp-lite
apply theorem + subgoal generation
induction-nat
explicit proof term
new small lemma
```

   This order minimizes import growth, axiom growth, downstream hash changes, and proof-term complexity.
4. Edit the proof corpus according to the repository's actual artifact layout:
   - Most generated proof modules are driven by `TheoremArtifact` entries in `tools/proof-corpus/src/main.rs`.
   - For checked-in source modules, also edit `proofs/**/source.npa`.
   - Keep source, metadata, replay artifacts, and `proofs/generated/ai-theorem-index.json` aligned when they are affected.
   - Do not add `sorry`, unapproved axioms, or kernel/checker/trusted-base changes to make a theorem pass.
5. Rebuild only the target module and its import closure:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
```

6. Source-free verify the checked-in certificate:

```sh
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
```

   `--verified-cache authoring` speeds up authoring iterations, but a cache hit is not proof evidence.
7. If verification fails, read the structured diagnostic and repair the smallest proof term, import, or lemma change. When helpful, emit a focused replay for the failing declaration:

```sh
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.X::theorem_name /tmp/npa-replay.json
```

   Focused replay is also an untrusted sidecar. Re-verify the resulting certificate before accepting the proof.
8. For a coherent theorem batch, run the lightweight authoring gate:

```sh
./scripts/check-corpus-authoring.sh
```

   Do not run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh` during ordinary proof authoring loops. Use them only for certificate encoding/hash, kernel semantics, checker/package verifier, promotion, release, or explicitly requested package/full coverage.
9. After the theorem proof is verified and the intended theorem batch is complete, commit and push the proof changes. Stage only files that belong to the theorem work; leave unrelated user changes unstaged.

## Gate Policy

During theorem authoring, the default verification loop is local:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
```

Use `--verified-cache authoring` only for repeated local checks where a cached
verdict can shorten authoring feedback. Cache hits are not proof evidence.

Run `./scripts/check-corpus-authoring.sh` before finishing a theorem batch. It
is the normal proof-corpus authoring completion gate and checks changed modules
source-free with the authoring cache. `./scripts/check-corpus.sh` is a
compatibility alias for the same lightweight gate.

Run `./scripts/check-fast.sh` when the change also touches non-corpus code or
when a fast workspace regression check is otherwise appropriate.

`./scripts/check-corpus-package.sh` and `./scripts/check-corpus-full.sh` are
intentionally expensive. Run them for proof-corpus package infrastructure,
certificate/package/checker compatibility, kernel semantics changes,
`npa-mathlib` promotion readiness, release handoff, or when the user explicitly
asks for package/full corpus coverage. If the package/full gate is skipped, say
so in the final response with the local checks that did run.

## Finalization

After the proof is complete and the local gates for that theorem or theorem
batch pass:

1. Inspect `git status --short` and identify only the files changed for this
   theorem work.
2. Stage the theorem source, generated certificates, replay/meta sidecars,
   manifest/package/index/report updates, README or roadmap updates, and any
   directly related tooling changes.
3. Do not stage unrelated dirty files or unrelated user changes, even if they
   are present in the worktree.
4. Commit with a concise theorem-focused message.
5. Push the current branch.
6. In the final response, report the commit hash, pushed branch, local proof
   checks run, and any expensive package/full corpus gate that was intentionally
   skipped.

## Guardrails

- Do not create interface-like theorems that merely accept and return a law without performing any actual proof. Create only theorems with substantial, meaningful proofs. (law を受け取って返すだけで、実質何も証明していないinterface 定理は作成しないでください。実質的に意味のある証明のみ作成してください。)
- Prefer adding a small lemma in the same narrow module over widening imports.
- Rebuild downstream modules only if a changed foundational module changes export hashes.
- Keep sidecars deterministic and untrusted; final acceptance is the `.npcert` verification result.
- If the theorem target or intended statement is ambiguous, ask one concise clarification before editing.
- Never commit or push a theorem proof before certificate verification succeeds.
- Once the proof is complete and verified, ensure all related changes are staged, committed, and pushed. (証明が完了し検証が成功したら、関連する変更を必ずstage/commit/pushしてください。)

## Reference

Read `references/npa-proof-corpus.md` when you need the command cookbook, target resolution details, or repair loop guidance.
