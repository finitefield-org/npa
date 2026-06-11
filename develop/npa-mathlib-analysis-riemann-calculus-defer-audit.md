# npa-mathlib Analysis Riemann Calculus Deferred Closure Audit

Date: 2026-06-11

This audit evaluates the newly completed
`Proofs.Ai.Analysis.Integral.Riemann.Calculus` corpus module as a possible
public `npa-mathlib` closure. It is a sidecar audit record, not proof
evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, publish plans, release artifacts, and this
audit are untrusted sidecars.

## Candidate Closure

| Corpus module | Proposed public module | Corpus surface | Direct axioms |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.Integral.Riemann.Calculus` | `Mathlib.Analysis.Integral.Riemann.Calculus` | 8 definitions, 5 theorems | none |

The candidate includes the following theorem surfaces:

- `integral_mean_value_theorem`
- `fundamental_theorem_of_calculus_part_one`
- `fundamental_theorem_of_calculus_part_two`
- `integration_by_parts`
- `substitution_formula`

## Corpus Evidence

| Artifact | Hash |
| --- | --- |
| source | `sha256:70039831953531fe5ebfe5995bd4eddb483ea250318e3a91d98618ca99ae348a` |
| certificate file | `sha256:11efcbd2e90969c63d5d5abcf974e153bcd30631c24c04fe7ff2c9363623cdb6` |
| export | `sha256:a3f0885ac049814d70d1fedc8e755571c9b26e416ffcf2171d626d86be39a163` |
| axiom report | `sha256:e73446550ebd2b8f59300fca0b3c3199e0a7a6f7722008c664f50b8a67fd00b9` |
| certificate | `sha256:9f86aabbb405bf818550cb8b28f4bfaa3457189f66f88d542da04284373ba5b2` |

## Promotion Decision

Decision: defer public materialization.

Reasons:

- The current `develop/npa-mathlib-next-closure-roadmap.md` baseline records
  no high-priority analysis closure queued after `npa-mathlib v0.1.27`.
- The candidate imports several proof-corpus staging modules that are not yet
  public `Mathlib.*` dependencies for this route, including real, sequence,
  continuity, interval, one-variable calculus, and Riemann integration modules.
- The candidate theorem surfaces are valid corpus route theorems, but their
  final mathematical conclusions still depend on theorem-specific bridge
  assumptions such as `mean_value_bridge`, `derivative_bridge`, and
  substitution or integration-by-parts bridge terms. These should not be
  published as final unconditional public theorem surfaces until the bridge
  assumptions are replaced by public prerequisite closures or the statements
  are intentionally renamed as conditional route theorems.
- A read-only `--promote-plan` attempt stopped before writing a plan because
  normal authoring did not generate public package metadata for this staging
  module:

```text
promote-plan error: missing_corpus_metadata Proofs.Ai.Analysis.Integral.Riemann.Calculus
```

Generating package metadata now would pull the newly authored staging modules
into public package artifacts. That would violate ANA-T38's acceptance
criterion that a promotion must not drag immature staging modules into public
mathlib.

## Required Work Before Reconsideration

- Promote or explicitly defer the imported real, sequence, continuity,
  one-variable calculus, and Riemann integration foundations as coherent
  public closures.
- Replace theorem-specific bridge assumptions with public prerequisite
  theorem surfaces, or rename the exported statements as conditional route
  theorems with an explicit public policy decision.
- Generate package metadata only for an audited public closure unit.
- Re-run the full `npa-mathlib` positive package gates and downstream
  source-free smoke checks after any actual materialization.

## ANA-T38 Result

This satisfies ANA-T38 for the current proof-corpus authoring pass as an
audit-only decision. No `../npa-mathlib` files, public package metadata,
package lock, axiom report, theorem index, publish plan, or downstream smoke
fixture were changed.
