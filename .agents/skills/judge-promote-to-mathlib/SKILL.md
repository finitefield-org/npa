---
name: judge-promote-to-mathlib
description: Judge whether an NPA proof-corpus theorem, module, or closure is an L2 proved-theorem candidate ready to be promoted into the standalone npa-mathlib package. Use when the user asks whether to promote corpus work to npa-mathlib, evaluate promotion readiness, review a closure for mathlib inclusion, or decide if a theorem should remain in corpus staging.
---

# Judge Promote To Mathlib

## Overview

Use this skill to make a promotion recommendation, not to perform the promotion. Treat the proof corpus as staging and `npa-mathlib` as the stable theorem package.

Trust boundary: source, replay, metadata, theorem indexes, and AI traces are not proof evidence. Promotion readiness depends on canonical certificates, deterministic package artifacts, source-free verification, and explicit dependency/axiom decisions.

Hard gate: only `L2 Derived certificate` proved theorems may be recommended for
promotion. If the candidate is `L0`, `L1`, an interface/evidence package, a
boundary theorem that assumes the conclusion, a mixed closure containing
non-L2 public declarations, or has unclear level evidence, recommend `Defer`
or `Reject for now`, not `Promote`.

## Workflow

1. Identify the promotion candidate:
   - theorem, module, or closure name;
   - current corpus path under `proofs/**` or generator entry in `tools/proof-corpus/src/main.rs`;
   - intended public `Mathlib.*` namespace, if already proposed.
2. Read repo policy before judging:
   - `AGENTS.md`
   - `develop/proof-corpus-ai-workflow.md`
   - any relevant `develop/npa-mathlib-*-closure-audit.md` for the current layer.
3. Gather evidence with cheap local inspection first:
   - Search `proofs/generated/ai-theorem-index.json` and `proofs/generated/theorem-index.json` for the candidate.
   - Inspect direct imports in `proofs/manifest.toml` / `proofs/npa-package.toml`.
   - Inspect axiom usage in `proofs/generated/axiom-report.json`.
   - Inspect the relevant roadmap, theorem card, audit, or source comments for
     explicit `L2 Derived certificate` classification.
   - Inspect downstream references with `rg "candidate_name|module_name" proofs tools/proof-corpus`.
4. Apply the promotion criteria below.
5. Recommend one of:
   - `Promote`: all required criteria are satisfied or have concrete evidence.
   - `Defer`: useful candidate, but at least one criterion is uncertain or not yet satisfied.
   - `Reject for now`: statement/API is unstable, dependency cost is too high, or verification/policy evidence is missing.

## Promotion Criteria

Promote to `npa-mathlib` only when these checks are satisfied or explicitly accepted with rationale:

- Every public theorem in the proposed closure is an `L2 Derived certificate`:
  its conclusion is derived from prior certified definitions and lemmas without
  assuming the conclusion itself.
- No `L0` statement/conjecture, `L1` evidence package/interface, or unclear-level
  declaration is included in the public promoted surface. Split the L2 subset
  first if the closure is mixed.
- The theorem/module name and statement are expected to remain stable.
- At least two downstream modules are likely to reuse it, or it is a clear foundation for a planned layer.
- The import closure is small and does not drag unrelated corpus staging modules into the public package.
- The axiom policy is clear and does not widen public `npa-mathlib` policy unintentionally.
- Source-free verifier, package hash check, theorem index check, and axiom report check pass for the promoted package.
- Compatibility alias needs are decided: either no alias is needed, or old corpus-facing names have an intentional bridge/deprecation plan.

## Verification Commands

When judging a local `npa-mathlib` checkout, prefer the package gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
```

If there is no local `npa-mathlib` checkout or the user only asked for a planning judgment, do not block on commands. Mark command evidence as missing and recommend `Defer` unless the rest of the evidence is strong enough for a planning-only `Promote after verification`.

## Output Format

Report concisely:

```text
Recommendation: Promote | Defer | Reject for now

Evidence:
- L2 proved-theorem status:
- Stable API:
- Downstream reuse:
- Import closure:
- Axiom policy:
- Verification:
- Compatibility alias:

Risks:
- ...

Next action:
- ...
```

Do not claim that promotion is complete unless the user asked to perform promotion and the actual package changes and verification commands have succeeded.
