---
name: promote-closures-loop
description: Repeatedly discover, judge, and materialize L2 NPA proof-corpus theorem closures into the sibling npa-mathlib package until no clearly promotable L2 closure remains. Use when the user asks to loop $find-promotable-closure, $judge-promote-to-mathlib, and $closure-audit, promote all ready L2 proved-theorem closures, exhaust promotable corpus candidates, or run a conservative npa-mathlib promotion campaign; if promotion readiness or L2 status is uncertain, do not promote that candidate.
---

# Promote Closures Loop

## Scope

Use this skill to orchestrate existing repo-local skills:

1. `find-promotable-closure`
2. `judge-promote-to-mathlib`
3. `closure-audit`

Do not treat discovery as promotion permission. A candidate may be materialized
only after a clear `Promote` recommendation, or an equivalent
`Promote after package verification` recommendation with no unresolved naming,
axiom, dependency, alias, or L2-level decision. If unsure whether to promote,
do not promote.

Only `L2 Derived certificate` proved theorems may be promoted. Skip `L0`
statements/conjectures, `L1` evidence packages or interfaces, boundary
declarations that assume their own conclusion, mixed closures with non-L2
public declarations, and candidates whose L2 status is unclear.

Keep the trust boundary explicit: source, replay, metadata, theorem indexes,
AI traces, audit docs, and this loop are sidecars. Public acceptance still
depends on canonical certificates, deterministic hashes, source-free verifier
verdicts, package gates, and explicit namespace/axiom decisions.

## Preflight

Before the loop:

- Read this repository's `AGENTS.md`.
- Read `.agents/skills/find-promotable-closure/SKILL.md`,
  `.agents/skills/judge-promote-to-mathlib/SKILL.md`, and
  `.agents/skills/closure-audit/SKILL.md`.
- Locate `npa` and sibling `../npa-mathlib`; stop and ask for a path if either
  is missing.
- Read `../npa-mathlib/AGENTS.md` and
  `../npa-mathlib/docs/namespace-policy.md` before accepting any public module
  or theorem names.
- Run `git status --short` in both repositories. Do not overwrite unrelated
  user changes. Stage only files belonging to the current closure when
  `closure-audit` reaches finalization.

## Loop

Maintain an in-memory ledger for the current run:

- promoted closures;
- skipped candidates with `Defer`, `Reject for now`, or uncertainty reasons;
- blockers or failed materializations.

Repeat until no unskipped clearly promotable candidate remains:

1. Run `find-promotable-closure` with the normal broad scan. If the scan finds
   no ready candidates, optionally rerun with `--include-defer` only to report
   near misses; do not promote near misses.
2. Select the highest-ranked candidate that is not already in the skipped
   ledger and not already promoted under a different public namespace. If all
   candidates are skipped, stop.
3. Manually inspect the selected candidate as required by
   `find-promotable-closure`, including sidecars, manifest entry, downstream
   reuse, existing audits, public namespace fit, and explicit
   `L2 Derived certificate` evidence.
4. Run `judge-promote-to-mathlib` for the selected module or closure.
5. If the recommendation is `Defer`, `Reject for now`, ambiguous, missing
   critical verification evidence, or dependent on an unresolved naming,
   axiom, dependency, alias, or policy decision, add it to the skipped ledger
   and return to step 1. Do not materialize it.
6. If the recommendation is clearly promotable, run `closure-audit` for exactly
   that closure and public namespace. Let `closure-audit` perform its audit,
   materialization, package gates, downstream smoke, negative checks, commit,
   push, and tag workflow.
7. If `closure-audit` fails, stop the loop. Record the failed command, reason
   code or diagnostic, affected files/modules, and next concrete action.
8. If `closure-audit` succeeds, add the closure to the promoted ledger and
   return to step 1 with a fresh scan against the updated repositories.

## Conservative Rules

- Do not promote a candidate merely because the finder scored it highly.
- Do not promote anything except an explicit `L2 Derived certificate` proved
  theorem surface.
- Do not promote `L0` statements/conjectures, `L1` evidence packages or
  interfaces, or mixed closures containing non-L2 public declarations.
- Do not promote when the public namespace or theorem names are unclear.
- Do not promote if `npa-mathlib/docs/namespace-policy.md` has not been checked
  for the route.
- Do not promote if source-free verification, axiom policy, package hash,
  theorem index, or package gate evidence is missing at the promotion boundary.
- Do not promote source-fallback candidates unless they have been brought into
  package metadata and verified.
- Do not continue the loop after a failed materialization, failed package gate,
  failed downstream smoke, failed negative check, dirty-state conflict, or tag
  conflict.
- Do not run multiple `closure-audit` materializations in parallel.

## Output

Report a concise campaign summary:

```text
Promoted:
- ...

Skipped:
- Proofs.Ai.X: Defer/Reject/uncertain because ...

Stop reason:
- No unskipped ready candidates remain | blocker/failure ...

Verification:
- Commands/gates completed by closure-audit, or missing evidence if stopped.
```

Do not claim promotion is complete for any closure unless `closure-audit`
successfully materialized it and completed its required verification/finalization
workflow.
