# Internal README Notes

This document preserves internal notes that were removed from the old README
when PUB-01 reorganized the root `README.md` into the public English entry
point.

The public entry points for users are the root `README.md` and
`docs/README.md`. This document is an internal note for maintainers and
development agents; it is not proof evidence.

## Implementation Status Notes from the Old README

The Rust kernel and the Phase 2 certificate verifier are currently
implemented. `crates/npa-cert` is responsible for canonical `.npcert`
encoding and decoding, hash recomputation, import checks, axiom report
checks, and handoff to the Rust kernel for rechecking.

Phase 3 is implemented in `crates/npa-frontend` with separate Human Surface
and Machine Surface layers. Phase 3 Human is the human-oriented convenience
layer used through `parse_human_*` and `compile_human_source_to_*`. It can
handle `open`, `namespace`, notation, implicit arguments, holes, simple
inductives, and similar surface features, but the parser, resolver,
elaborator, and metadata are not part of the trusted base. Phase 3 AI is the
explicit fast path used through `parse_machine_*` and the Machine Surface term
API. AI candidate generation and tactic / search / replay / verify check
Machine Surface requests directly, without going through the Human Surface and
without notation tables, open scopes, overload transactions, or holes.

Phase 4 Human connects the Human API wrapper in `crates/npa-api` to the
proof-state primitives in `crates/npa-tactic`, translating `intro`, `exact`,
`apply`, `rw`, `simp-lite`, and `induction` in `by` proof blocks into proof
terms that the kernel can check. Certificate-compatible Human examples that
include `rw` and `induction` are fixed as regressions that do not change
Machine Surface fixture hashes. This Human parser / bridge is not on the
default path for the AI-oriented Machine API.

The AI-oriented Phase 4 M1/M2/M3/M4/M5/M6/M7 tactic proof-state core and
`exact` / `intro` / `apply` / `rw` / `simp-lite` / `induction-nat` are
implemented in `crates/npa-tactic`. The same crate also implements the handoff
API from a closed proof state to a canonical certificate, together with
deterministic budget hashes, tactic cache keys, and batch execution gates for
AI search.

The Phase 5 AI substrate is being developed in `crates/npa-api`. It includes a
lossless JSON request decoder, import/current projection, the Phase 4 adapter
boundary, the Machine Surface callable interface table builder,
owner-aware MachineExprRenderer v1 / renderer QA substrate, and
MachineApiDiagnostic canonicalization. It also includes the M2 Machine API
types / ID and HashString wire grammar / endpoint envelope validation, plus
library APIs for M5 `/machine/snapshots/get`, M6 `/machine/tactics/run`, M7
`/machine/tactics/batch`, M8 `/machine/search/for_goal`, M9
`/machine/replay`, M10 `/machine/verify`, and M11 `/machine/prompt_payload`.

The Phase 5 Human IDE/API profile is also implemented in `crates/npa-api`. It
provides Human sessions, structured proof states, transactional `/tactic/run`,
theorem search, goal display, verify / certificate handoff, an incremental
document cache, LSP-facing payloads, and optional assistant payloads. The
Phase 5 Human integration fixtures cover session creation, state lookup,
tactic run, search, display, and verify, while also fixing as a regression
that the Human path does not change the Phase 7 Machine API candidate hash or
state fingerprint.

The Phase 6 Human / AI standard-library handoff is also implemented in
`crates/npa-api`. On the Human side, it fixes the source package layout and
certificate build boundary for `Std.Logic` / `Std.Nat` / `Std.List` /
`Std.Algebra.Basic`. On the AI side, it regenerates the release manifest,
import bundles, theorem index, rewrite / simp profiles, and axiom report from
the same raw `.npcert` files. `std.nat.mvp`, `std.list.mvp`, and `std.all.mvp`
are expanded into requests equivalent to Phase 5 `/machine/sessions` and
reverified. It is fixed as a regression that Phase 7 retrieval candidates are
always routed back through Phase 5 batch / replay / verify before adoption.
The generated `.npcert` files and `Std.machine-*.json` files are release/build
artifacts; in this repository, the source layout fixtures, Rust builders, and
tests are the source of truth and regenerate them in a temporary package.

The same `crates/npa-api` crate also implements the Phase 7 search controller,
Phase 8 checker audit automation, and Phase 9 advanced automation endpoint
substrate. The Phase 9 Human target scope is implemented through advanced
inductives, strengthened universe polymorphism, typeclasses, `quotient_v1`,
the SMT certificate surface / reconstruction boundary, theorem graph, and the
natural-language formalization confirmation flow. The Phase 9 Human / AI
boundary is fixed by
`p9h00_advanced_ai_sidecars_scores_and_smt_outputs_stay_untrusted` and
`p9h00_ai_fast_path_request_shapes_exclude_phase9_human_heavy_checks`.
Sidecars, scores, solver outputs, confidence values, and heavy audits for
advanced features are not used as grounds for the AI candidate hot path or the
checker verdict.

Phase 9 AI has implemented the deterministic validation / replay substrate and
the M9 fixture matrix, but production LLM / RAG, an online theorem graph
store, an external SMT solver service, and a non-empty solver-native SMT
success profile are target integrations and are not treated as implemented in
this repository.

In Phase 8, the `npa-checker-ref` binary in `crates/npa-checker-ref` checks
`.npcert` files without source, and `crates/npa-api` fixes the untrusted
orchestration for checker request / result normalization, release audit
bundles, challenge replay, and AI sidecar validation. The OCaml clean-room
`npa-checker-ext` source is in `checkers/npa-checker-ext/`. Evidence is treated
as release/high-trust evidence only when a built binary is resolved from a
runner-owned checker registry and package `--checker external` integration
plus binary hash / identity validation have passed. `package high-trust` is
implemented as the `verified_high_trust` artifact generator, and the copyable
opt-in high-trust CI template is available at
`ci-templates/github-actions/npa-package-high-trust.yml`. However, it does not
generate artifacts from reference-only evidence.

External checker benchmark summaries are release audit metadata linked to
checker result hashes. They may fail release/high-trust policy as regression
evidence, but they are not checker verdicts and do not affect proof validity.

These `npa-api` automation / library APIs are an untrusted layer responsible
for candidate generation, construction of verification requests, normalization
of audit artifacts, and execution of regression fixtures. They do not expand
the trusted base. Proof acceptance continues to rest only on canonical
certificates and the deterministic results returned by the Rust kernel /
independent checker.

## Development Notes from the Old README

During normal development, do not put the proof corpus on the hot path. Run
the short fast gate first.

```sh
./scripts/check-fast.sh
```

`./scripts/check-fast.sh` runs format / clippy / workspace tests while
excluding `npa-proof-corpus` and proof-corpus-backed package verifier / CLI
fixture tests. The proof corpus is a working staging space; during normal
authoring, package-wide checks are not put on the hot path.

Normal checks for proof corpus theorem authoring:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

`./scripts/check-corpus.sh` runs the lightweight authoring gate as a
compatibility wrapper.

Run the package/full corpus gates explicitly only when one of the following
conditions applies.

- Immediately before promotion to `npa-mathlib`, or for a release /
  high-trust gate.
- Changes involving package metadata / promotion / package lock / artifact
  generation under `tools/proof-corpus/**`.
- Changes involving package generated artifacts such as
  `proofs/npa-package.toml`, `proofs/generated/package-lock.json`,
  axiom-report, theorem-index, or publish-plan.
- Changes involving certificate canonical encode / decode / hash / import /
  axiom report behavior.
- Changes involving kernel core semantics, typecheck, reduction, universes, or
  inductives.
- Changes involving the independent checker, package verifier, package lock,
  or artifact validation.
- Changes involving `.npcert` generation or checker compatibility.

When those conditions apply, run:

```sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

While adding theorems to the proof corpus, regenerate and check only the
target module instead of running the package/full corpus gate every time.

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
```

`--build-module` is an authoring helper that regenerates only the specified
module and import closure from source. `--module` / `--changed-only` are
source-free checks of checked-in certificates. For detailed AI-oriented
procedures, see `develop/proof-corpus-ai-workflow.md`.

The required release completion gate after Phase 9 Human is:

```sh
./scripts/phase9-regression.sh
```

This gate first runs the Phase 9 AI M9 deterministic fixture matrix, then runs
`fmt --check`, `clippy -D warnings`, and the full workspace tests. Release /
high-trust pass/fail is determined by checker results and deterministic
artifacts; AI sidecars, theorem graph scores, formalization confidence, and
SMT solver outputs do not enter the trusted boundary. The GitHub Actions
workflow has been removed. Run this gate locally as needed.

The Phase 8 release audit fixture gate is:

```sh
./scripts/phase8-release-audit.sh
```

This gate runs `cargo test -p npa-checker-ref`,
`cargo test -p npa-api independent_checker`, the standard-library release
audit fixtures, and `cargo test -p npa-api ai_search`. The GitHub Actions
workflow has been removed. Run this gate locally as needed. The Phase 8 gate
is a narrow gate that checks the source-free checker / release audit / AI fast
path boundary, while Phase 9 Regression is a broader regression gate that also
covers later functionality across the workspace.
