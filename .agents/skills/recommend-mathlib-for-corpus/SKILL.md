---
name: recommend-mathlib-for-corpus
description: Recommend which promoted npa-mathlib modules should be consumed next from the NPA proofs/ proof corpus. Use when the user asks which Mathlib.* modules to import, dogfood, pin, or migrate into proof corpus authoring, or wants a ranked plan before using use-mathlib-in-corpus to add hash-pinned npa-mathlib imports.
---

# Recommend Mathlib For Corpus

## Scope

Use this skill to recommend migration priority only. Do not edit source files,
copy certificates, or modify `proofs/npa-package.toml` from this skill. Use
`use-mathlib-in-corpus` for pinning and corpus rewrites after a recommendation
is accepted.

Treat `npa-mathlib` as stable public theorem package and `proofs/` as staging
authoring space. Recommendations must preserve the certificate-first trust
boundary: source, replay, metadata, roadmap references, and helper scores are
not proof evidence.

## Workflow

1. Locate repositories. Prefer current checkout as `npa` and sibling
   `../npa-mathlib` for the public package.
2. Read the current local policy before recommending:
   - `AGENTS.md`
   - `develop/proof-corpus-ai-workflow.md`
   - `proofs/README.md`
   - `../npa-mathlib/docs/namespace-policy.md`
   - `.agents/skills/use-mathlib-in-corpus/SKILL.md`
3. Run the read-only recommender:

```sh
.agents/skills/recommend-mathlib-for-corpus/scripts/recommend_mathlib_for_corpus.sh \
  --mathlib-root ../npa-mathlib \
  --limit 12
```

Use `--prefix Proofs.Ai.NumberTheory.` or another corpus namespace when the
user asks about a topic. Use `--include-pinned` to include modules already
present as `[[imports]]` in `proofs/npa-package.toml`.

4. Interpret the score as a prioritization hint:
   - high replacement-site count means many corpus modules still import a
     promoted `Proofs.Ai.*` counterpart;
   - `status: not-pinned` means adding a `[[imports]]` stanza and vendored
     certificate is a likely first step;
   - `status: pinned` means the module is already available in the corpus
     package and source rewrite may be the next step;
   - modules with no detected replacement sites are not rejected, but need a
     roadmap or new-dogfood rationale.
5. For the top module, manually inspect a small migration unit before acting:
   - source import sites listed by the helper;
   - corresponding public `Mathlib.*` module path and hashes;
   - whether the source statement/API is stable enough to use from corpus;
   - whether the migration can be verified with local authoring gates.
6. Hand off accepted recommendations to `use-mathlib-in-corpus`:

```sh
.agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh pin \
  Mathlib.X \
  --mathlib-root ../npa-mathlib \
  --apply
```

Then rewrite the selected `source.npa` imports narrowly and run the verification
commands from `use-mathlib-in-corpus`.

## Output Format

Report concise recommendations:

```text
Recommended next imports:
1. Mathlib.X
   Why: ...
   Corpus evidence: ...
   Pin status: ...
   Suggested first migration unit: ...
   Next command: ...

Risks:
- ...
```

If there are no strong candidates, say whether the blocker is no local
`npa-mathlib`, no detected promoted counterparts, all candidates already pinned,
or no topic-matching corpus source imports.

## Tooling

The bundled helper is Rust. The shell wrapper compiles it with `rustc` into a
temporary binary and runs it. It is read-only.

```sh
.agents/skills/recommend-mathlib-for-corpus/scripts/recommend_mathlib_for_corpus.sh --help
```
