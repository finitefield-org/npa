# Proof Corpus AI Workflow

This document defines the operating policy for letting AI add theorems while
expanding the proof corpus without spending too much time. It does not change
NPA's trust boundary. AI, replay, metadata, and theorem indexes are all
untrusted sidecars; acceptance rests only on canonical certificates and the
results of checker / kernel verification.

Plans and specifications for tooling improvements are recorded in
`develop/proof-corpus-tooling-improvement-plan.md`.

## Basic Policy

AI does not produce proofs as "trusted artifacts"; it produces many cheap
candidates. Each candidate is immediately run through the Machine Surface /
tactic API / certificate verifier. If it fails, a structured diagnostic is
returned to the AI for repair.

As a rule, search proceeds from the cheapest option upward.

```text
exact local hypothesis
exact known theorem
rw / simp-lite
apply theorem + subgoal generation
induction-nat
explicit proof term
new lemma
```

Human Surface conveniences may be used for corpus authoring, but the AI search
hot path prioritizes the Machine Surface, tactic candidates, and source-free
certificate verification.

## Normal Addition Loop

1. Put the theorem to add in a small module.
2. Minimize imports.
3. Update the AI theorem index as needed.
4. Regenerate only the added module and its import closure from source.
5. Check only the added module source-free.
6. Extract only failed declarations into focused replay and send them back for
   AI repair.
7. Once the work is coherent, run `./scripts/check-corpus-authoring.sh`.

Common commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic
cargo run -p npa-proof-corpus -- --write-ai-index
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --failures-out proofs/generated/failed-corpus-replay.json
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Basic::id proofs/generated/replay-basic-id.json
```

`--build-module MODULE` is a fast authoring helper. It compiles only the
specified module and its import closure from Human Surface source and updates
`source.npa`, `certificate.npcert`, `meta.json`, `replay.json`, and the
untrusted AI theorem index. During normal authoring, it does not update the
public `manifest.toml`, `npa-package.toml`, or `generated/package-lock.json`.
Because downstream modules are not regenerated, if the export hash of a
foundation module changes, either run `--build-module` on the required
downstream modules in order or detect the issue before a promotion / package
gate.

When grouping multiple modules, process their import closures together with a
batch build.

```sh
cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.X Proofs.Ai.Y
cargo run -p npa-proof-corpus -- --build-modules-file proofs/generated/build-batch.txt
```

Add `--package-metadata` explicitly only when also updating public package
metadata for a promotion / release handoff. The old `--metadata-once` flag is
a compatibility alias.

`--module` and `--changed-only` check checked-in certificates source-free.
Dependency modules are loaded recursively, and verified modules / decoded
certificates are cached within the same process.

Only while authoring and repeatedly checking the same certificates, add
`--verified-cache authoring` to reuse the versioned verified cache across
processes. When cache mode is enabled, output includes
`cache_status = "hit"` / `"miss"` / `"stale"` and
`cache_mode = "authoring"`. The default is `off`. The normal authoring gate
`./scripts/check-corpus-authoring.sh` and the compatibility wrapper
`./scripts/check-corpus.sh` use the authoring cache, but package/full gates and
release / high-trust-equivalent checks do not. To inspect cache behavior, use
`--verified-cache read-through` and compare live verifier results with cache
entries.

## Shard

Use zero-based shards when splitting a larger verification run.

```sh
cargo run -p npa-proof-corpus -- --verify --shard 0/4
cargo run -p npa-proof-corpus -- --verify --shard 1/4
cargo run -p npa-proof-corpus -- --verify --shard 2/4
cargo run -p npa-proof-corpus -- --verify --shard 3/4
```

This can also be used for a changed set, for example
`--changed-only --shard 0/2`.

## AI Theorem Index

`--write-ai-index` generates `proofs/generated/ai-theorem-index.json`.
This is a lightweight index for AI retrieval. It includes theorem names,
statements, imports, certificate paths, replay paths, and focused replay
specs, but it is not a trusted artifact.

The existing package theorem index is a broader certificate-derived release
artifact. During AI work, use the lightweight index first and run the heavier
package / corpus gates only when needed.

## Focused Replay

Use `--write-replay MODULE::DECL PATH` to send only a failed declaration back
to the AI.

```sh
cargo run -p npa-proof-corpus -- \
  --write-replay Proofs.Ai.Basic::id proofs/generated/replay-basic-id.json
```

Focused replay is an untrusted sidecar. A resubmitted candidate advances to
certificate handoff only if it passes.

## Promotion Criteria for npa-mathlib

Treat the proof corpus as a staging / exploration area and `npa-mathlib` as a
stable theorem package. Theorems and modules added in the corpus are brought
into `npa-mathlib` once they satisfy the following conditions. After
promotion, new downstream corpus modules should use package imports from
`npa-mathlib` as much as possible instead of reproving the same content.
Existing corpus replacements are performed gradually when the relevant files
are touched or dependencies are reorganized, not all at once.

Promotion criteria:

- The name and statement are not expected to change soon.
- It is likely to be used by two or more downstream modules, or it is a clear
  foundation for a planned layer.
- The import closure is small and does not pull immature staging modules into
  the public package.
- The axiom policy is clear and does not unintentionally broaden the public
  `npa-mathlib` policy.
- The source-free verifier, package hash check, theorem index check, and axiom
  report check pass.
- Whether a compatibility alias needs to remain has already been decided.

Promotion does not change the grounds for proof acceptance. `source.npa`,
`replay.json`, `meta.json`, and the AI theorem index remain untrusted
sidecars. The public package's trust basis is the canonical certificate,
deterministic hashes, and source-free checker / verifier verdicts.

If the decision is unclear, use the `judge-promote-to-mathlib` skill to list
the evidence and explicitly choose `Promote`, `Defer`, or `Reject for now`.

Promotion plans and materialize dry-runs can be standardized with repo-local
commands.

```sh
cargo run -p npa-proof-corpus -- \
  --promote-plan Proofs.Ai.Algebra.AbstractField \
  --mathlib-root ../npa-mathlib \
  --to-module Mathlib.Algebra.Field.Basic \
  --out develop/npa-mathlib-field-closure-audit.md

cargo run -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-field-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --dry-run \
  --compat-alias none
```

`--promote-plan` is a read-only audit helper. `--promote-materialize` is a
dry-run by default and writes package files on the `npa-mathlib` side only when
`--apply` is specified. Neither replaces the source-free verifier / package
hash / index / axiom-report checks; they are untrusted helpers for aligning the
promotion decision and workflow.

## Gate

Use `./scripts/check-fast.sh` during normal development.

For normal completion checks while authoring proof corpus theorems, do not run
the package/full corpus gate every time during the work. Check locally with
`--module`, `--changed-only`, and `--shard`, then run
`./scripts/check-corpus-authoring.sh`. This gate runs only source-free checks
for changed modules with `--verified-cache authoring`; it does not include
package-wide CLI examples, axiom-report, theorem-index, or publish-plan. The
existing `./scripts/check-corpus.sh` runs the same lightweight authoring gate
as a compatibility wrapper.

Run `./scripts/check-corpus-package.sh` for changes involving compatibility of
certificate / kernel / checker / package verification, public-boundary checks
of package metadata / package CLI examples / axiom-report / index /
publish-plan, or immediately before promotion to `npa-mathlib`. Before a
release / high-trust gate, or when checking both authoring and package gates
together, explicitly run `./scripts/check-corpus-full.sh`.

In the PCT-08 measurements, the local small-module authoring loop was about
394 times shorter than the baseline full corpus gate. The current
`check-corpus-authoring.sh` leans further into this local authoring policy and
runs only changed-only source-free checks. Historical measurement details are
recorded in `develop/proof-corpus-tooling-pct-08-measurement.md`.
