---
name: find-promotable-closure
description: Find NPA proof-corpus closures under proofs/ that are not yet promoted to the sibling npa-mathlib package but look ready for promotion. Use when the user asks for the next promotable corpus closure, wants to discover unpromoted proof-corpus modules suitable for npa-mathlib, or needs a ranked read-only promotion-candidate scan before using judge-promote-to-mathlib or closure-audit.
---

# Find Promotable Closure

## Scope

Use this skill to discover candidate closures only. Do not materialize
`npa-mathlib`, generate release artifacts, tag, push, or claim promotion is
complete from this skill alone.

Keep the trust boundary explicit: this skill ranks untrusted staging evidence.
Promotion readiness still depends on canonical certificates, deterministic
hashes, source-free verification, package gates, and an explicit axiom/import
decision.

## Workflow

1. Locate repositories. Prefer the current checkout as `npa`; default the
   mathlib checkout to sibling `../npa-mathlib`. If missing, report the blocker
   and ask for the path.
2. Read current policy before judging candidates:
   - `AGENTS.md`
   - `develop/proof-corpus-ai-workflow.md`
   - `proofs/README.md`
   - recent `develop/npa-mathlib-*-closure-audit.md` docs, if they exist
   - `../npa-mathlib/AGENTS.md`, if it exists
3. Run a read-only candidate scan:

```sh
.agents/skills/find-promotable-closure/scripts/find_promotable_closures.sh \
  --mathlib-root ../npa-mathlib \
  --limit 15 \
  --max-closure 8
```

The helper is a thin shell wrapper around a Node.js script and does not modify
the repository.

Use `--prefix Proofs.Ai.NumberTheory.` or another corpus namespace when the
user asks for a topic. Use `--include-defer` only when no ready candidates are
found or when investigating near misses. Add `--with-axiom-report` only for a
shortlist; `proofs/generated/axiom-report.json` can be large, and broad
discovery should stay lightweight. Add `--include-source-fallback` when the
candidate is in authoring state and not yet present in `proofs/npa-package.toml`.

4. Interpret the scan as a heuristic, not a proof. The helper reads
   `proofs/npa-package.toml` by default. With `--include-source-fallback`, it
   also scans `proofs/Proofs/Ai/**/source.npa` for authoring modules that have
   not had package metadata generated yet. Treat `metadata: source-fallback`
   rows as weaker evidence and verify them before recommendation. A
   `candidate` means:
   - the root is not detected in `npa-mathlib` by export hash or declaration
     signature;
   - unpromoted `Proofs.Ai.*` dependencies are contained in the proposed
     closure, and promoted/public dependencies are outside the closure;
   - source, certificate, meta, and replay sidecars exist for the closure;
   - manifest evidence, or axiom-report evidence when `--with-axiom-report` is
     used, does not violate the current `Eq.rec` policy.
5. Inspect the top candidate manually:
   - read each closure module's `source.npa`, `meta.json`, and manifest entry;
   - check whether public names are stable and map cleanly to `Mathlib.*`;
   - confirm downstream reuse by searching direct imports, transitive imports,
     and roadmap references;
   - compare with existing closure audits to avoid duplicating promoted layers.
6. If the user asks whether it should be promoted, invoke
   `judge-promote-to-mathlib` on the selected root or closure. If the user asks
   to perform promotion, hand off to `closure-audit`.

## Useful Commands

Validate the selected corpus module source-free before recommending it:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
```

Seed a read-only promotion plan for a single root after selecting the intended
public module name:

```sh
cargo run -q -p npa-proof-corpus -- \
  --promote-plan Proofs.Ai.X \
  --mathlib-root ../npa-mathlib \
  --to-module Mathlib.X \
  --out develop/npa-mathlib-x-closure-audit.md
```

Do not run package/full corpus gates during broad discovery. Reserve package
gates for promotion boundary work, certificate/checker/package changes, or when
`closure-audit` requires them.

## Output Format

Report concise, evidence-backed results:

```text
Top candidate: Proofs.Ai.X
Suggested closure: Proofs.Ai.A, Proofs.Ai.X
Why it looks promotable: ...
Evidence checked: ...
Risks or unknowns: ...
Next action: judge-promote-to-mathlib | closure-audit | defer
```

If no candidate meets the ready heuristic, report the best near misses and the
exact blocker, such as missing `npa-mathlib`, large unpromoted dependency
closure, missing sidecars, unclear axiom policy, or weak downstream reuse.
