# Analysis Theorem Proof Roadmap Todo

Source: `proofs/analysis-theorem-proof-roadmap.md`

This task breakdown converts the analysis theorem roadmap into implementation
units that later agents can complete one milestone at a time. It is a planning
sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files,
replay files, metadata, theorem indexes, this todo document, tactics, and AI
output are untrusted.

---

## Scope

In scope:

```text
- theorem-card inventory for the full analysis theorem list
- explicit law-package statements for real/complete ordered field foundations
- sequence, series, continuity, one-variable calculus, and Riemann integration
  foundations from the first execution queue
- classical specializations of existing abstract inverse, implicit, derivative,
  fixed-point, normed-space, and linear-map routes
- later track kickoff milestones for topology, measure theory, functional
  analysis, complex analysis, Fourier analysis, ODE, PDE, and variational
  methods
- proof-corpus sidecar updates required by each theorem batch
- promotion planning for stable theorem families
```

Out of scope:

```text
- adding real numbers, integration, measure theory, topology, or calculus as
  trusted kernel primitives
- widening the trusted base with parser, elaborator, tactic, theorem search,
  AI, plugin, network, or registry assumptions
- relying on source.npa, replay.json, meta.json, theorem indexes, or this todo
  as proof evidence
- running full corpus or package gates inside every local authoring attempt
- changing public npa-mathlib release order except through a separate closure
  audit
```

Default authoring loop for theorem milestones:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `./scripts/check-corpus-package.sh` or
`./scripts/check-corpus-full.sh` only when the milestone changes package-wide
metadata, package verification, checker compatibility, certificate
compatibility, release readiness, or public `npa-mathlib` materialization.

---

## Current Implementation Facts

Existing reusable analysis corpus modules:

```text
Proofs.Ai.Analysis.AbstractMetricTopology
Proofs.Ai.Analysis.AbstractNormedSpace
Proofs.Ai.Analysis.AbstractLinearMap
Proofs.Ai.Analysis.AbstractDerivative
Proofs.Ai.Analysis.AbstractFixedPoint
Proofs.Ai.Analysis.AbstractInverseFunction
Proofs.Ai.Analysis.AbstractImplicitPhi
Proofs.Ai.Analysis.AbstractImplicitFunction
Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem
Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem
```

Public `npa-mathlib` has already materialized through
`npa-mathlib v0.1.27`, including:

```text
Mathlib.Topology.Metric.Basic
Mathlib.Analysis.NormedSpace.Basic
Mathlib.Analysis.LinearMap
Mathlib.Analysis.Calculus.Derivative
Mathlib.Analysis.FixedPoint.Banach
Mathlib.Analysis.Calculus.InverseFunction
Mathlib.Analysis.Calculus.ImplicitFunction.Phi
Mathlib.Analysis.Calculus.ImplicitFunction
```

Implications:

- New work should reuse these abstract routes and public closures instead of
  reproving their APIs under new names.
- There are not yet concrete proof-corpus modules for
  `Proofs.Ai.Analysis.Sequence.*`, `Series.*`, `Continuity.*`,
  `Integral.Riemann.*`, `Measure.*`, `Complex.*`, `Fourier.*`, `ODE.*`,
  `PDE.*`, or `Variational.*`.
- First-pass real-analysis foundations should use explicit law packages such
  as `CompleteOrderedFieldArgs`; they must not add a trusted `Real` primitive
  to the kernel.
- Existing spectral theorem modules keep construction evidence explicit and
  should be treated as interface-level evidence until later foundations justify
  or replace that evidence.

---

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `ANA-00` inventory and statement policy | ANA-T00 |
| `ANA-01` real numbers and sequences | ANA-T01 through ANA-T05 |
| `ANA-02` series and power series | ANA-T06 through ANA-T09 |
| `ANA-03` continuous functions on intervals | ANA-T10 through ANA-T12 |
| `ANA-04` one-variable differential calculus | ANA-T13 through ANA-T15 |
| `ANA-05` Riemann integration | ANA-T16 through ANA-T18 |
| `ANA-06` finite-dimensional and multivariable calculus | ANA-T19 through ANA-T21 |
| `ANA-07` metric and topological analysis | ANA-T22 through ANA-T23 |
| `ANA-08` measure and Lebesgue integration | ANA-T24 through ANA-T26 |
| `ANA-09` functional analysis | ANA-T27 through ANA-T28 |
| `ANA-10` complex analysis | ANA-T29 through ANA-T30 |
| `ANA-11` Fourier analysis | ANA-T31 through ANA-T32 |
| `ANA-12` ordinary differential equations | ANA-T33 through ANA-T34 |
| `ANA-13` PDE and Sobolev methods | ANA-T35 through ANA-T36 |
| `ANA-14` variational methods and optimization | ANA-T37 |
| `ANA-15` packaging and promotion | ANA-T38 |

## Target Level Defaults

| Task milestone | Default target level |
| --- | --- |
| ANA-T00 | documentation planning for `L0` through `L3` classification |
| ANA-T01 | `L1` evidence-package foundation, with `L2` follow-up expected before promotion |
| ANA-T02 through ANA-T23 | `L2` derived certificates unless a milestone explicitly says a statement split is needed |
| ANA-T24 | `L1` construction interface is allowed for Caratheodory/simple-function construction; derived convergence theorems wait for `L2` |
| ANA-T25 through ANA-T27 | `L2` derived certificates, with construction-heavy existence statements audited for circular assumptions |
| ANA-T28 | `L1` is allowed for existing spectral theorem aliases; Hilbert and weak-topology foundations target `L2` |
| ANA-T29 through ANA-T37 | `L2` derived certificates where prerequisites exist; otherwise the milestone must split before source edits |
| ANA-T38 | `L3` public closure and package verification |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

---

## Milestones

### ANA-T00 Build Theorem Card Inventory

- Status: Pending
- Depends on: None
- Inputs:
  - `proofs/analysis-theorem-proof-roadmap.md`
  - user-provided theorem inventory attached to the roadmap request
  - `proofs/README.md`
  - `develop/npa-mathlib-next-closure-roadmap.md`
- Code or documentation areas:
  - `proofs/analysis-theorem-proof-roadmap-todo.md`
  - a future theorem-card sidecar under `proofs/` if the card table becomes too large
- Tasks:
  - Create one theorem card for every theorem in the roadmap inventory.
  - Record stable English identifier, Japanese display name, target level
    (`L0`, `L1`, `L2`, or `L3`), primary home milestone, proposed module, and
    dependency tags.
  - Mark duplicate theorem names and shared theorem families, including
    Fubini, Tonelli, inverse function, implicit function, Banach fixed point,
    Riesz representation, and spectral theorem.
  - Mark each target as foundation, derived theorem, specialization, package
    alias, or long-term theorem.
- Deliverables:
  - Theorem-card inventory sidecar or a compact table in this todo document.
  - Duplicate map that points every repeated theorem name to one primary home.
- Acceptance criteria:
  - Every theorem from the roadmap has exactly one primary home milestone.
  - No duplicated theorem family is scheduled for independent reproving in two
    modules.
  - Each card states the minimum verification gate for its first landing.
- Verification:
  - `rg -n "Fubini|Tonelli|inverse function|implicit function|Banach fixed|Riesz|spectral" proofs/analysis-theorem-proof-roadmap*.md`
  - `git diff --check`
- Notes:
  - This is documentation-only and should precede any theorem source changes.

### ANA-T01 Fix Real And Complete Ordered Field Statement Shape

- Status: `ANQ-001` complete. `Proofs.Ai.Analysis.Real.Basic` now keeps the
  first real-analysis foundation fully abstract over a `Scalar`, with
  `CompleteOrderedFieldArgs` packaging ordered-field laws, field bridge laws,
  interval laws, order completeness, and Archimedean evidence. No named `Real`
  carrier or completeness primitive was added to the kernel.
- Depends on: ANA-T00
- Inputs:
  - `proofs/analysis-theorem-proof-roadmap.md` section `ANA-01`
  - existing ordered-field and scalar modules in `proofs/Proofs/Ai/Algebra/`
  - `Proofs.Ai.Algebra.AbstractOrderedField`
  - `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Analysis/Real/Basic/`
  - `proofs/README.md`
  - proof-corpus package and manifest sidecars
- Tasks:
  - Decide the first foundation shape for `CompleteOrderedFieldArgs`.
  - Decide whether to introduce a named `Real` package now or keep the first
    route fully abstract.
  - Add statement-level or evidence-package definitions for completeness,
    order, Archimedean property, intervals, bounds, suprema, and infima.
  - Audit that completeness is not added as a kernel primitive.
- Deliverables:
  - First `Proofs.Ai.Analysis.Real.Basic` module or a documented insertion plan
    if the implementation milestone stays statement-only.
  - README entry describing the trusted-boundary role of real-analysis law
    packages.
- Acceptance criteria:
  - The route keeps real completeness as ordinary proof-corpus evidence.
  - No theorem imports Riemann integration, measure theory, or topology
    foundations before the real/sequence APIs exist.
  - The decision about named `Real` versus abstract complete ordered field is
    recorded before downstream sequence modules depend on it.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Real.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Real.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`
- Notes:
  - Downstream sequence, series, and continuity modules should import this
    abstract complete ordered-field evidence rather than assuming a trusted
    real-number primitive.

### ANA-T02 Add Sequence Convergence Core

- Status: `ANQ-002` complete. `Proofs.Ai.Analysis.Sequence.Basic` now defines
  the reusable sequence, subsequence, limit, eventuality, boundedness, and
  limit-uniqueness vocabulary over the abstract complete ordered-field
  foundation from `ANA-T01`. It also exposes checked aliases to the existing
  `AbstractFixedPoint` `ConvergesTo` and `CauchySeq` concepts without changing
  that module.
- Depends on: ANA-T01
- Inputs:
  - `Proofs.Ai.Analysis.Real.Basic`
  - `Proofs.Ai.Analysis.AbstractMetricTopology`
  - `Proofs.Ai.Analysis.AbstractFixedPoint`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Sequence/Basic/`
  - `tools/proof-corpus/src/main.rs`
  - `proofs/README.md`
  - proof-corpus package and generated sidecars
- Tasks:
  - Define sequence, subsequence, limit, eventually, bounded sequence, and
    uniqueness-of-limit vocabulary.
  - Bridge `ConvergesTo` and `CauchySeq` concepts to the existing fixed-point
    module where possible without changing the existing module.
  - Prove limit uniqueness and simple convergence projection lemmas.
  - Add theorem names needed by later series, compactness, and continuity work.
- Deliverables:
  - Certificate-backed `Proofs.Ai.Analysis.Sequence.Basic` module.
  - README module entry and theorem inventory update.
- Acceptance criteria:
  - Convergence is reusable by series partial sums and function limits.
  - Limit uniqueness is an `L2` derived certificate, not an assumed law package.
  - Imports stay below series, continuity, integral, measure, and ODE modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`
- Notes:
  - The checked `sequence_limit_unique` theorem derives equality from explicit
    local `SequenceLimitUniquenessEvidence`; uniqueness is not installed as a
    trusted kernel primitive or module-level law package.
  - Later series, compactness, and continuity modules should import
    `Proofs.Ai.Analysis.Sequence.Basic` rather than redefining convergence
    vocabulary.

### ANA-T03 Add Cauchy And Completeness Sequence Theorems

- Status: `ANQ-003` complete. `Proofs.Ai.Analysis.Sequence.Basic` now includes
  `SequenceCauchySeq`, `SequenceConvergenceChoice`, and
  `SequenceCauchyCompletenessEvidence`, plus checked theorem names for deriving
  sequence convergence from Cauchy evidence through explicit fixed-point metric
  completeness bridges.
- Depends on: ANA-T02
- Inputs:
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.Real.Basic`
  - `develop/proof-corpus-ai-workflow.md`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Sequence/Basic/` or
    `Proofs/Ai/Analysis/Sequence/Completeness/`
  - `tools/proof-corpus/src/main.rs`
  - generated proof-corpus sidecars
- Tasks:
  - Define Cauchy sequence predicates for numeric sequences if the existing
    abstract fixed-point `CauchySeq` is not directly reusable.
  - Prove Cauchy sequence convergence from explicit completeness evidence.
  - Add the Cauchy convergence criterion shape used later by series.
  - Keep focused replay support for each theorem target.
- Deliverables:
  - Certificate-backed Cauchy convergence theorem for sequences.
  - Stable theorem names for later `Series.Basic` imports.
- Acceptance criteria:
  - The proof uses `CompleteOrderedFieldArgs` or equivalent explicit evidence.
  - The theorem is not encoded by assuming convergence of every Cauchy sequence
    as the exact conclusion under another name.
  - `--module` source-free verification passes for the changed module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Analysis.Sequence.Basic::sequence_cauchy_converges_from_completeness /tmp/anq003-sequence-cauchy-converges-replay.json`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`
- Notes:
  - `sequence_cauchy_converges_from_completeness` calls the fixed-point
    `CompleteMetricArgs` eliminator obtained from explicit
    `SequenceCauchyCompletenessEvidence`; the final sequence convergence choice
    is not a renamed assumption.
  - Stable downstream import names are
    `sequence_cauchy_convergence_criterion` and
    `cauchy_convergence_criterion`.

### ANA-T04 Add Monotone, Squeeze, And Interval-Nesting Theorems

- Status: `ANQ-004` complete for monotone convergence; `ANQ-005` complete for
  squeeze; `ANQ-006` complete for interval nesting.
  `Proofs.Ai.Analysis.Sequence.Basic` now defines one-sided boundedness,
  monotone increasing/decreasing predicates, explicit monotone-completeness
  evidence, explicit squeeze-bounds/convergence evidence, and explicit nested
  closed-interval/length evidence, with checked theorem names deriving monotone
  convergence through the real-order supremum route, deriving middle-sequence
  convergence through squeeze evidence, and deriving interval nesting through
  a lower-endpoint supremum route.
- Depends on: ANA-T03
- Inputs:
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Sequence/Basic/`
  - `Proofs/Ai/Analysis/Sequence/Monotone/` if split
  - `Proofs/Ai/Analysis/Real/Basic/`
- Tasks:
  - Define monotone sequence and bounded-above/bounded-below sequence evidence.
  - Prove monotone convergence theorem for sequences.
  - Define squeeze hypotheses, nested closed intervals, and shrinking interval
    length.
  - Prove squeeze theorem.
  - Prove interval nesting theorem from completeness.
- Deliverables:
  - Certificate-backed monotone convergence theorem for sequences.
  - Certificates for squeeze theorem and interval nesting.
  - Imports ready for continuity and compactness milestones.
- Acceptance criteria:
  - Nested intervals use explicit closed interval and length hypotheses.
  - Monotone convergence does not assume supremum existence outside the
    explicit real-completeness package.
  - The three theorem statements can be found in the AI theorem index after
    rebuild.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Analysis.Sequence.Basic::sequence_monotone_converges_from_completeness /tmp/anq004-sequence-monotone-converges-replay.json`
  - `cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Analysis.Sequence.Basic::sequence_squeeze_converges /tmp/anq005-sequence-squeeze-replay.json`
  - `cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Analysis.Sequence.Basic::nested_interval_point_from_completeness /tmp/anq006-nested-interval-replay.json`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`
- Notes:
  - `sequence_monotone_converges_from_completeness` extracts
    `OrderCompletenessLawArgs` from the ambient `CompleteOrderedFieldArgs` and
    calls `supremum_exists_from_completeness` for the supplied value set. The
    final convergence choice is produced from the returned supremum evidence by
    `SequenceMonotoneCompletenessEvidence`; supremum existence is not assumed as
    a separate theorem input.
  - Stable downstream import names for monotone convergence are
    `sequence_monotone_converges_from_completeness` and
    `monotone_convergence_theorem`.
  - Stable downstream import names for squeeze are `sequence_squeeze_converges`
    and `squeeze_theorem`.
  - Stable downstream import names for interval nesting are
    `nested_interval_point_from_completeness` and `interval_nesting_theorem`.

### ANA-T05 Add Bolzano-Weierstrass And Sequence Compactness

- Status: Pending
- Depends on: ANA-T04
- Inputs:
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Sequence/Compactness/`
  - generated proof-corpus sidecars
- Tasks:
  - Define subsequence extraction evidence and bounded sequence compactness.
  - Prove Bolzano-Weierstrass for bounded real sequences.
  - Add reusable lemmas for later Heine-Borel and compact interval results.
- Deliverables:
  - Certificate-backed `Proofs.Ai.Analysis.Sequence.Compactness` module.
  - README entry explaining the compactness dependency route.
- Acceptance criteria:
  - The theorem is derived from sequence/completeness foundations, not from a
    primitive compactness axiom.
  - Downstream continuity milestones can import compactness without importing
    series or integration modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sequence.Compactness`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Sequence.Compactness`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T06 Add Series Basics And Cauchy Criterion

- Status: Pending
- Depends on: ANA-T03
- Inputs:
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Series/Basic/`
  - `tools/proof-corpus/src/main.rs`
  - proof-corpus metadata sidecars
- Tasks:
  - Define partial sums, series convergence, absolute convergence, tails, and
    Cauchy series criterion.
  - Prove that series convergence is sequence convergence of partial sums.
  - Add Cauchy convergence criterion for series.
- Deliverables:
  - Certificate-backed `Proofs.Ai.Analysis.Series.Basic` module.
  - Theorem names consumed by convergence criteria modules.
- Acceptance criteria:
  - Series convergence reduces to existing sequence convergence.
  - No theorem introduces an independent, incompatible notion of limit.
  - `Series.Basic` imports sequence modules but not continuity or integration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Series.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Series.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T07 Add Absolute Convergence And Comparison Tests

- Status: Pending
- Depends on: ANA-T06
- Inputs:
  - `Proofs.Ai.Analysis.Series.Basic`
  - ordered-field absolute value or norm evidence from real foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Series/Criteria/`
  - `Proofs/Ai/Analysis/Real/Basic/` if absolute-value support is missing
- Tasks:
  - Define nonnegative series, termwise order domination, and absolute-value
    series.
  - Prove absolute convergence implies convergence.
  - Prove comparison test for nonnegative and absolutely dominated series.
  - Add focused replay support for each theorem.
- Deliverables:
  - Certificate-backed comparison and absolute-convergence theorem targets.
- Acceptance criteria:
  - Comparison hypotheses are explicit and do not rely on typeclass search.
  - Absolute convergence theorem reuses the series Cauchy criterion.
  - All new order or absolute-value assumptions are ordinary law packages.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Series.Criteria`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Series.Criteria`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T08 Add Ratio And Root Tests

- Status: Pending
- Depends on: ANA-T07
- Inputs:
  - `Proofs.Ai.Analysis.Series.Criteria`
  - real order and exponent/root support from earlier foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Series/Criteria/`
  - `Proofs/Ai/Analysis/Series/Power/` only if the root-test support belongs there
- Tasks:
  - Define ratio-test limit hypotheses and root-test limit hypotheses.
  - Prove d'Alembert ratio test.
  - Prove Cauchy root test.
  - Document any exponentiation or nth-root assumptions introduced for the
    first time.
- Deliverables:
  - Certificate-backed ratio and root tests.
  - README theorem table entries for the criteria module.
- Acceptance criteria:
  - The tests reduce to comparison or geometric-series style lemmas.
  - Missing exponent/root foundations are added as explicit law evidence, not
    trusted primitives.
  - No power-series boundary theorem is bundled into this milestone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Series.Criteria`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Series.Criteria`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T09 Add Alternating, Dirichlet, Abel, And Rearrangement Planning Split

- Status: Pending
- Depends on: ANA-T08
- Inputs:
  - `Proofs.Ai.Analysis.Series.Criteria`
  - sequence and real foundations from ANA-T01 through ANA-T08
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Series/Criteria/`
  - `Proofs/Ai/Analysis/Series/Power/`
  - `proofs/analysis-theorem-proof-roadmap-todo.md`
- Tasks:
  - Prove Leibniz alternating series test.
  - Prove Dirichlet test once partial-sum boundedness and monotone-to-zero
    predicates are stable.
  - Add Abel theorem for power-series boundary limits only after the power
    series API is fixed.
  - Create a separate implementation note for Riemann rearrangement theorem if
    permutations and conditional convergence are not yet available.
- Deliverables:
  - Certificate-backed alternating and Dirichlet theorem targets.
  - Power-series module statement shape for Abel theorem.
  - Deferred note for Riemann rearrangement if needed.
- Acceptance criteria:
  - Riemann rearrangement is not attempted before permutation and conditional
    convergence APIs exist.
  - Abel theorem does not depend on complex analytic-function infrastructure.
  - Authoring gate passes after the criteria batch.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Series.Criteria`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Series.Power`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T10 Add Function Limits And Continuity Core

- Status: Pending
- Depends on: ANA-T05
- Inputs:
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.AbstractMetricTopology`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Continuity/Basic/`
  - generated proof-corpus sidecars
- Tasks:
  - Define pointwise limit, continuity at a point, continuity on a set,
    uniform continuity, and composition.
  - Add local equality and neighborhood bridge lemmas using the existing
    metric-topology module.
  - Prove basic continuity composition and restriction lemmas.
- Deliverables:
  - Certificate-backed continuity foundation module.
  - Theorem names required by interval continuity and one-variable calculus.
- Acceptance criteria:
  - The module does not import Riemann integration or derivative modules.
  - Local predicates reuse `AbstractMetricTopology` instead of introducing
    incompatible neighborhood vocabulary.
  - Uniform continuity statement shape is fixed before compact-interval proofs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Continuity.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Continuity.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T11 Add Bolzano And Intermediate Value Theorems

- Status: Pending
- Depends on: ANA-T10
- Inputs:
  - `Proofs.Ai.Analysis.Continuity.Basic`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Continuity/Interval/`
- Tasks:
  - Define interval membership, endpoint order, sign change, and interval image
    vocabulary.
  - Prove Bolzano zero theorem.
  - Prove intermediate value theorem.
  - Add theorem aliases only if they do not duplicate primary theorem names.
- Deliverables:
  - Certificate-backed interval continuity module.
- Acceptance criteria:
  - IVT uses certified completeness or compactness route from previous
    milestones.
  - Bolzano theorem is represented as a specialization or corollary of IVT
    without duplicating proof infrastructure unnecessarily.
  - The theorem-card duplicate map is updated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Continuity.Interval`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Continuity.Interval`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T12 Add Extreme Value And Uniform Continuity On Compact Intervals

- Status: Pending
- Depends on: ANA-T11
- Inputs:
  - `Proofs.Ai.Analysis.Continuity.Interval`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Continuity/Interval/`
  - `Proofs/Ai/Topology/Metric/Compact/` if compactness generalization is split
- Tasks:
  - Define maximum/minimum attainment and compact interval evidence.
  - Prove extreme value theorem for closed intervals.
  - Prove uniform continuity theorem on closed intervals.
  - Leave the general compact-space form to the topology track.
- Deliverables:
  - Certificate-backed extrema and uniform-continuity theorem targets.
- Acceptance criteria:
  - Theorems are interval-specific and do not claim the general compact-space
    theorem before topology foundations exist.
  - No global choice principle is added to select extrema without explicit
    compactness evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Continuity.Interval`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Continuity.Interval`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T13 Add One-Dimensional Derivative Bridge

- Status: Pending
- Depends on: ANA-T12
- Inputs:
  - `Proofs.Ai.Analysis.AbstractDerivative`
  - `Proofs.Ai.Analysis.Continuity.Interval`
  - `Proofs.Ai.Analysis.Real.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Calculus/OneVariable/`
  - `proofs/README.md`
- Tasks:
  - Define one-dimensional derivative notation as a specialization or bridge
    from Frechet derivative.
  - Define local extrema, critical points, differentiability on intervals, and
    derivative-zero vocabulary.
  - Prove bridge lemmas from Frechet derivative rules to one-variable rules.
- Deliverables:
  - Certificate-backed one-variable derivative bridge module.
- Acceptance criteria:
  - No separate derivative primitive is added.
  - One-variable derivative theorem statements can import existing Frechet
    derivative rules.
  - The bridge is sufficient for Fermat, Rolle, and mean value theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.OneVariable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Calculus.OneVariable`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T14 Add Fermat, Rolle, And Mean Value Theorem

- Status: Pending
- Depends on: ANA-T13
- Inputs:
  - `Proofs.Ai.Analysis.Calculus.OneVariable`
  - `Proofs.Ai.Analysis.Continuity.Interval`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Calculus/OneVariable/`
- Tasks:
  - Prove Fermat theorem for differentiable local extrema.
  - Prove Rolle theorem.
  - Prove mean value theorem.
  - Add theorem-card aliases for Japanese theorem names.
- Deliverables:
  - Certificate-backed Fermat, Rolle, and MVT theorem targets.
- Acceptance criteria:
  - Rolle and MVT depend on interval continuity and compactness facts, not on
    a primitive calculus axiom.
  - Fermat theorem requires an explicit differentiability hypothesis.
  - The MVT theorem statement exposes endpoint and open-interval assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.OneVariable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Calculus.OneVariable`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T15 Add Cauchy MVT, l'Hopital, Taylor, And Convex Tangent Route

- Status: Pending
- Depends on: ANA-T14
- Inputs:
  - `Proofs.Ai.Analysis.Calculus.OneVariable`
  - `Proofs.Ai.Analysis.Convex.Basic`
  - optional `Proofs.Ai.Analysis.Series.Power` support only if this milestone
    explicitly chooses a power-series proof route
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Calculus/OneVariable/`
  - `Proofs/Ai/Analysis/Calculus/Taylor/`
  - `Proofs/Ai/Analysis/Convex/Basic/`
- Tasks:
  - Prove Cauchy mean value theorem.
  - Prove l'Hopital theorem after denominator derivative nonzero hypotheses are
    expressible.
  - Define Taylor polynomial and remainder evidence.
  - Prove Taylor theorem with remainder and Maclaurin specialization.
  - Define convex functions and prove tangent inequality.
- Deliverables:
  - Certificate-backed Cauchy MVT, l'Hopital, Taylor, Maclaurin, and convex
    tangent theorem targets.
- Acceptance criteria:
  - l'Hopital uses Cauchy MVT and explicit nonzero hypotheses.
  - Taylor theorem does not depend on power-series convergence unless the
    statement explicitly chooses a power-series route.
  - Any power-series-dependent Taylor or Maclaurin variant is split into a
    follow-up that depends on ANA-T09.
  - Convex tangent inequality is reusable by optimization milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.Taylor`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Convex.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T16 Add Riemann Partition And Integrability Criterion

- Status: Pending
- Depends on: ANA-T12
- Inputs:
  - `Proofs.Ai.Analysis.Real.Basic`
  - `Proofs.Ai.Analysis.Continuity.Interval`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Integral/Riemann/Basic/`
- Tasks:
  - Define partitions, tagged partitions if needed, mesh, refinements, upper
    sums, lower sums, upper integral, lower integral, and Riemann integral
    value.
  - Prove upper/lower sum refinement lemmas.
  - Prove Riemann integrability criterion from equality of upper and lower
    integrals.
- Deliverables:
  - Certificate-backed Riemann integration foundation module.
- Acceptance criteria:
  - Integral value uniqueness is certified.
  - The criterion does not assume integrability as an input.
  - Partition APIs are deterministic and structurally represented.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T17 Prove Continuous And Monotone Functions Are Riemann Integrable

- Status: Pending
- Depends on: ANA-T16
- Inputs:
  - `Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `Proofs.Ai.Analysis.Continuity.Interval`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Integral/Riemann/Basic/`
- Tasks:
  - Prove continuous functions on closed intervals are Riemann integrable.
  - Define bounded monotone functions on intervals.
  - Prove bounded monotone functions are Riemann integrable.
  - Add theorem names for later fundamental theorem of calculus work.
- Deliverables:
  - Certificate-backed integrability theorem targets.
- Acceptance criteria:
  - Continuous integrability uses compact interval or uniform continuity
    results.
  - Monotone integrability requires explicit boundedness and interval
    hypotheses.
  - No Lebesgue integral concepts are imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T18 Add Fundamental Theorem Of Calculus And Riemann Integral Identities

- Status: Pending
- Depends on: ANA-T17
- Inputs:
  - `Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `Proofs.Ai.Analysis.Calculus.OneVariable`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Integral/Riemann/Calculus/`
- Tasks:
  - Prove integral mean value theorem.
  - Prove fundamental theorem of calculus part 1.
  - Prove fundamental theorem of calculus part 2.
  - Prove integration by parts.
  - Prove substitution formula.
- Deliverables:
  - Certificate-backed calculus/Riemann integration module.
- Acceptance criteria:
  - FTC statements separate continuity, primitive function, differentiability,
    and interval orientation assumptions.
  - Integration by parts follows from product derivative and FTC assumptions.
  - Substitution formula exposes differentiability and interval mapping
    hypotheses explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Integral.Riemann.Calculus`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Integral.Riemann.Calculus`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T19 Add Euclidean Specialization And Existing Inverse/Implicit Aliases

- Status: Pending
- Depends on: ANA-T14
- Inputs:
  - existing vector, normed-space, linear-map, derivative, inverse-function,
    and implicit-function modules
  - `Proofs.Ai.Analysis.Sequence.Compactness`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Euclidean/Basic/`
  - `Proofs/Ai/Analysis/Calculus/Multivariable/`
- Tasks:
  - Define finite-dimensional Euclidean coordinate products and product norm
    laws needed by multivariable calculus.
  - Prove or package closed-bounded compactness equivalence for Euclidean
    spaces after sequence compactness is available.
  - Add inverse-function theorem specialization from
    `Proofs.Ai.Analysis.AbstractInverseFunction`.
  - Add implicit-function theorem specialization from
    `Proofs.Ai.Analysis.AbstractImplicitFunction`.
- Deliverables:
  - Euclidean foundation module and inverse/implicit theorem alias module.
- Acceptance criteria:
  - Existing abstract inverse and implicit theorem APIs are reused.
  - The alias route does not duplicate the abstract proof under a new theorem
    family.
  - Heine-Borel dependencies are documented before compactness is claimed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Euclidean.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.Multivariable`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T20 Add Multivariable Differential Calculus Core

- Status: Pending
- Depends on: ANA-T19
- Inputs:
  - `Proofs.Ai.Analysis.Euclidean.Basic`
  - `Proofs.Ai.Analysis.AbstractDerivative`
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Calculus/Multivariable/`
- Tasks:
  - Prove multivariable mean value theorem.
  - Prove multivariable Taylor theorem.
  - Prove equality of mixed partial derivatives under explicit smoothness
    assumptions.
  - Prove Lagrange multipliers after constraint and rank hypotheses are fixed.
- Deliverables:
  - Certificate-backed multivariable calculus theorem targets.
- Acceptance criteria:
  - Smoothness assumptions are explicit law packages or derived predicates.
  - Mixed partial theorem does not assume equality of partials as a law input.
  - Lagrange multipliers exposes constraint regularity conditions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.Multivariable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Calculus.Multivariable`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T21 Add Change Of Variables And Vector Calculus Theorem Route

- Status: Pending
- Depends on: ANA-T18, ANA-T20
- Inputs:
  - Riemann or measure integration route selected for multivariable integrals
  - Euclidean orientation and boundary vocabulary
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Calculus/ChangeOfVariables/`
  - `Proofs/Ai/Analysis/VectorCalculus/`
- Tasks:
  - Fix Jacobian determinant and orientation statement shape.
  - Prove Jacobian change-of-variables formula once integration foundation is
    sufficient.
  - Define the boundary and differential-form or vector-field vocabulary needed
    for Green, Gauss, and Stokes.
  - Prove Green, Gauss divergence, and Stokes theorems only after their
    boundary hypotheses are stable.
- Deliverables:
  - Change-of-variables theorem module.
  - Vector-calculus theorem module or statement-level split plan.
- Acceptance criteria:
  - No theorem uses informal orientation or boundary side conditions.
  - Stokes-family theorems do not precede the integration/orientation API.
  - Duplicate theorem cards point to the same vector-calculus family.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Calculus.ChangeOfVariables`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.VectorCalculus`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T22 Add Metric And Topological Compactness Route

- Status: Pending
- Depends on: ANA-T05, ANA-T10, ANA-T19
- Inputs:
  - `Proofs.Ai.Analysis.AbstractMetricTopology`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
  - `Proofs.Ai.Analysis.Euclidean.Basic`
- Code or documentation areas:
  - `Proofs/Ai/Topology/Basic/`
  - `Proofs/Ai/Topology/Metric/Compact/`
- Tasks:
  - Define open sets, closed sets, closure, limit points, compactness, and
    connectedness in a general topological vocabulary.
  - Prove open/closed set basics.
  - Prove compactness equivalence for metric and Euclidean settings.
  - Prove Heine-Borel theorem.
  - Prove continuous maps preserve compact and connected sets.
- Deliverables:
  - Certificate-backed topology and metric compactness modules.
- Acceptance criteria:
  - The route extends existing metric topology without replacing it.
  - Heine-Borel depends on Euclidean and sequence compactness foundations.
  - General compact-space uniform continuity is separated from interval
    uniform continuity.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metric.Compact`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T23 Add Baire And Function-Space Topology Route

- Status: Pending
- Depends on: ANA-T22
- Inputs:
  - `Proofs.Ai.Topology.Metric.Compact`
  - `Proofs.Ai.Analysis.AbstractFixedPoint`
  - normed-space and function-space law packages
- Code or documentation areas:
  - `Proofs/Ai/Topology/FunctionSpace/`
- Tasks:
  - Prove Banach fixed-point theorem public alias from existing abstract route.
  - Prove Baire category theorem for complete metric spaces.
  - Define equicontinuity and compactness for function families.
  - Prove Arzela-Ascoli theorem.
  - Plan Stone-Weierstrass and Urysohn lemma only after algebra-of-functions
    and normal-space APIs are fixed.
- Deliverables:
  - Function-space topology module and Baire theorem target.
- Acceptance criteria:
  - Baire uses complete metric space evidence from earlier foundations.
  - Arzela-Ascoli states compactness and equicontinuity hypotheses explicitly.
  - Stone-Weierstrass and Urysohn are not bundled if prerequisites are missing.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FunctionSpace`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.FunctionSpace`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T24 Add Measure And Lebesgue Construction Foundation

- Status: Pending
- Depends on: ANA-T17, ANA-T22
- Inputs:
  - topology foundation from ANA-T22
  - real and sequence foundations from ANA-T01 through ANA-T05
  - Riemann integration foundation from ANA-T16 through ANA-T17 for the
    roadmap's ANA-05 dependency and Riemann/Lebesgue disambiguation checks
- Code or documentation areas:
  - `Proofs/Ai/Measure/Basic/`
  - `Proofs/Ai/Measure/Construction/`
- Tasks:
  - Define sigma algebras, measurable sets, measurable functions, measures,
    null sets, and almost-everywhere predicates.
  - Define outer measure and measurable-set criterion.
  - Prove Caratheodory extension theorem or land an `L1` construction
    interface if full derivation is too large for the first batch.
  - Define simple functions and the initial Lebesgue integral construction
    interface.
- Deliverables:
  - Measure foundation modules with stable names and statement shapes.
- Acceptance criteria:
  - Measure construction evidence is explicit and not a checker primitive.
  - Almost-everywhere vocabulary is reusable by convergence theorems.
  - Riemann integration is not silently identified with Lebesgue integration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Construction`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T25 Add Lebesgue Convergence Theorem Chain

- Status: Pending
- Depends on: ANA-T24
- Inputs:
  - `Proofs.Ai.Measure.Basic`
  - `Proofs.Ai.Measure.Construction`
  - initial Lebesgue integral construction interface from ANA-T24
- Code or documentation areas:
  - `Proofs/Ai/Measure/Integral/`
- Tasks:
  - Prove monotone convergence theorem for Lebesgue integration.
  - Prove Fatou lemma.
  - Prove dominated convergence theorem.
  - Prove bounded convergence theorem as a specialization or separate theorem.
  - Add theorem-card disambiguation from sequence monotone convergence.
- Deliverables:
  - Certificate-backed measure convergence theorem chain.
- Acceptance criteria:
  - Tonelli/Fubini work waits until product measures are available.
  - Monotone convergence for integrals is not conflated with sequence
    monotone convergence.
  - Dominating functions and integrability hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Integral`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### ANA-T26 Add Product Measures, Decomposition, And Differentiation Route

- Status: Pending
- Depends on: ANA-T25
- Inputs:
  - `Proofs.Ai.Measure.Integral`
  - measure foundation modules from ANA-T24
  - topology and compactness modules
- Code or documentation areas:
  - `Proofs/Ai/Measure/Product/`
  - `Proofs/Ai/Measure/Decomposition/`
- Tasks:
  - Prove Tonelli theorem for nonnegative measurable functions.
  - Prove Fubini theorem for integrable functions.
  - Define absolute continuity, singularity, signed measures, and density.
  - Prove Radon-Nikodym theorem and Lebesgue decomposition theorem after the
    signed-measure API is stable.
  - Prove Egorov, Lusin, Vitali convergence, Riesz representation for
    measures, and Lebesgue differentiation theorem in separate batches if the
    module grows too large.
- Deliverables:
  - Product-measure and measure-decomposition theorem modules.
- Acceptance criteria:
  - Tonelli precedes Fubini.
  - Decomposition theorems expose absolute-continuity and singularity
    hypotheses explicitly.
  - Late regularity theorems are split if their dependencies exceed one
    coherent closure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Product`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Decomposition`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T27 Add Banach-Space Functional Analysis Core

- Status: Pending
- Depends on: ANA-T23
- Inputs:
  - existing normed-space, linear-map, fixed-point, and topology modules
  - Baire theorem from ANA-T23
- Code or documentation areas:
  - `Proofs/Ai/FunctionalAnalysis/Banach/`
- Tasks:
  - Define Banach spaces, continuous linear functionals, dual spaces, operator
    boundedness, and quotient or extension evidence as needed.
  - Prove Hahn-Banach theorem with explicit scalar/order/norm assumptions.
  - Prove uniform boundedness principle.
  - Prove open mapping theorem.
  - Prove closed graph theorem.
- Deliverables:
  - Certificate-backed Banach functional-analysis theorem module.
- Acceptance criteria:
  - Hahn-Banach keeps extension evidence explicit if the first landing is
    `L1`.
  - Open mapping and closed graph depend on Baire category.
  - No theorem widens the public axiom policy silently.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.Banach`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.FunctionalAnalysis.Banach`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T28 Add Hilbert, Weak Topology, And Spectral Functional Analysis Route

- Status: Pending
- Depends on: ANA-T26, ANA-T27
- Inputs:
  - inner-product modules
  - existing spectral theorem modules
  - measure foundations from ANA-T26
  - topology and compactness foundations from ANA-T22 and ANA-T23
- Code or documentation areas:
  - `Proofs/Ai/FunctionalAnalysis/Hilbert/`
  - `Proofs/Ai/FunctionalAnalysis/WeakTopology/`
  - `Proofs/Ai/FunctionalAnalysis/Spectral/`
- Tasks:
  - Prove Hilbert projection theorem.
  - Prove Riesz-Frechet representation theorem.
  - Prove Hilbert-space orthogonal decomposition theorem.
  - Define weak and weak-star topology APIs.
  - Prove Banach-Alaoglu theorem after weak-star compactness is available.
  - Re-audit finite-dimensional and Hilbert-space spectral theorem modules
    before adding public aliases.
  - Plan compact operator spectral theorem, Fredholm alternative,
    Krein-Milman, and Milman-Pettis as separate batches.
- Deliverables:
  - Hilbert functional-analysis module.
  - Weak topology module.
  - Spectral theorem alias or replacement plan.
- Acceptance criteria:
  - Existing spectral modules remain `L1` until construction evidence is
    justified or replaced by derived foundations.
  - Banach-Alaoglu depends on weak-star topology.
  - Public aliases wait for namespace and closure audit decisions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.Hilbert`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.WeakTopology`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T29 Add Complex Numbers, Holomorphic Functions, And Cauchy Theorem Family

- Status: Pending
- Depends on: ANA-T18, ANA-T22
- Inputs:
  - real and Riemann integration foundations
  - topology foundations from ANA-T22
  - series foundations are deferred to ANA-T30 unless this milestone
    explicitly splits off local power-series statements
- Code or documentation areas:
  - `Proofs/Ai/Complex/Basic/`
  - `Proofs/Ai/Complex/Holomorphic/`
  - `Proofs/Ai/Complex/Cauchy/`
- Tasks:
  - Fix complex-number construction or law-package interface.
  - Define complex norm, holomorphic functions, paths, contour integrals, and
    primitives.
  - Prove Cauchy integral theorem.
  - Prove Cauchy integral formula.
  - Prove Morera theorem, Liouville theorem, and fundamental theorem of algebra
    after Cauchy formula is stable.
- Deliverables:
  - Complex foundation and Cauchy theorem modules.
- Acceptance criteria:
  - Complex scalar assumptions are explicit and not hard-coded into kernel.
  - Cauchy theorem and formula precede downstream complex analysis theorems.
  - Fundamental theorem of algebra theorem-card dependency is updated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Complex.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Complex.Cauchy`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T30 Add Meromorphic, Residue, And Advanced Complex Analysis Route

- Status: Pending
- Depends on: ANA-T09, ANA-T22, ANA-T29
- Inputs:
  - `Proofs.Ai.Complex.Cauchy`
  - series and topology foundations
- Code or documentation areas:
  - `Proofs/Ai/Complex/Meromorphic/`
- Tasks:
  - Prove maximum and minimum modulus principles.
  - Prove holomorphic open mapping theorem and identity theorem.
  - Define isolated singularities, Laurent expansions, residues, zeros, and
    poles.
  - Prove singularity classification, Laurent theorem, residue theorem,
    argument principle, Rouche theorem, and Schwarz lemma.
  - Leave Riemann mapping, Mittag-Leffler, and Weierstrass factorization as
    separate late batches unless all topology and approximation prerequisites
    are available.
- Deliverables:
  - Meromorphic/residue theorem module and advanced theorem split plan.
- Acceptance criteria:
  - Residue theorem depends on Laurent or contour integral infrastructure.
  - Riemann mapping and factorization do not land as unreviewed `L1`
    conclusions.
  - All zeros/poles counting theorems share one vocabulary.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Complex.Meromorphic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Complex.Meromorphic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T31 Add Fourier Series And Transform Foundations

- Status: Pending
- Depends on: ANA-T09, ANA-T26, ANA-T28, ANA-T29
- Inputs:
  - measure integration
  - complex exponentials
  - Hilbert or inner-product foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Fourier/Series/`
  - `Proofs/Ai/Analysis/Fourier/Transform/`
- Tasks:
  - Define periodic function spaces, trigonometric system, Fourier
    coefficients, and trigonometric polynomials.
  - Prove Fourier series expansion theorem under selected regularity
    assumptions.
  - Prove Dirichlet convergence theorem.
  - Prove Fejer theorem.
  - Define Fourier transform and convolution APIs.
- Deliverables:
  - Fourier series and transform foundation modules.
- Acceptance criteria:
  - Fourier coefficients use complex and integration foundations rather than
    new primitive operations.
  - Regularity assumptions for Fourier series convergence are explicit.
  - The theorem-card inventory distinguishes series and transform results.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Fourier.Series`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Fourier.Transform`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T32 Add Parseval, Plancherel, Riemann-Lebesgue, Convolution, Poisson, And Sampling

- Status: Pending
- Depends on: ANA-T31
- Inputs:
  - Fourier foundations
  - Hilbert and measure foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Fourier/Transform/`
  - `Proofs/Ai/Analysis/Fourier/Sampling/`
- Tasks:
  - Prove Parseval identity.
  - Prove Plancherel theorem.
  - Prove Riemann-Lebesgue lemma.
  - Prove convolution theorem.
  - Prove Poisson summation formula after lattice/summability assumptions are
    fixed.
  - Prove sampling theorem after bandlimited-function vocabulary is stable.
  - Leave Carleson theorem as a long-term theorem-card unless prerequisites
    are explicitly completed.
- Deliverables:
  - Advanced Fourier theorem modules.
- Acceptance criteria:
  - Parseval and Plancherel use Hilbert-space or L2 foundations.
  - Poisson and sampling statements expose summability and bandlimit
    hypotheses.
  - Carleson is not scheduled as an early derived certificate.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Fourier.Transform`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Fourier.Sampling`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T33 Add ODE Foundations, Gronwall, And Picard-Lindelof

- Status: Pending
- Depends on: ANA-T14, ANA-T18, ANA-T23
- Inputs:
  - existing fixed-point and derivative modules
  - Riemann integration and continuity foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/ODE/Basic/`
  - `Proofs/Ai/Analysis/ODE/Existence/`
- Tasks:
  - Define local solution, maximal solution, initial value problem, flow, and
    integral equation formulation.
  - Prove Gronwall inequality.
  - Prove Picard-Lindelof theorem using Banach fixed-point machinery.
  - Prove continuous dependence theorem once solution uniqueness is available.
- Deliverables:
  - ODE basic and existence theorem modules.
- Acceptance criteria:
  - Picard-Lindelof does not add an ODE-specific existence primitive.
  - Lipschitz, continuity, and interval hypotheses are explicit.
  - Gronwall is reusable by continuous-dependence and PDE energy estimates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.ODE.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.ODE.Existence`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T34 Add Peano, Linear ODE, Sturm, And Planar Dynamical Systems Route

- Status: Pending
- Depends on: ANA-T33
- Inputs:
  - ODE existence module
  - topology and compactness foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/ODE/Linear/`
  - `Proofs/Ai/Analysis/DynamicalSystems/Planar/`
- Tasks:
  - Prove Peano existence theorem after compactness or selection machinery is
    available.
  - Prove linear ODE fundamental theorem.
  - Prove Floquet theorem.
  - Prove Sturm comparison theorem and plan Sturm-Liouville theory.
  - Add Poincare-Bendixson and Hartman-Grobman only after planar topology and
    hyperbolicity vocabulary are stable.
- Deliverables:
  - Linear ODE theorem module and planar dynamics split plan.
- Acceptance criteria:
  - Peano theorem does not land before the required compactness/selection
    mechanism.
  - Linear ODE theorem states solution-space structure explicitly.
  - Planar dynamics theorems are deferred if topology prerequisites are absent.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.ODE.Linear`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.DynamicalSystems.Planar`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T35 Add Sobolev And Weak PDE Foundations

- Status: Pending
- Depends on: ANA-T26, ANA-T28, ANA-T31, ANA-T33
- Inputs:
  - measure foundations
  - Hilbert functional analysis
  - Fourier foundations when needed
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Sobolev/Basic/`
  - `Proofs/Ai/Analysis/PDE/Weak/`
- Tasks:
  - Define weak derivatives, Sobolev spaces, Sobolev norms, weak formulation,
    bilinear forms, coercivity, and weak solution predicates.
  - Prove Poincare inequality.
  - Prove Sobolev embedding theorem.
  - Prove Rellich compactness theorem.
  - Prove Lax-Milgram theorem and Hilbert-space weak solution existence.
- Deliverables:
  - Sobolev foundation module and weak PDE theorem module.
- Acceptance criteria:
  - Sobolev embedding and Rellich depend on measure and compactness
    foundations.
  - Lax-Milgram uses Hilbert-space and bounded coercive bilinear form evidence.
  - Weak solution existence is not assumed as a primitive theorem package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sobolev.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.PDE.Weak`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T36 Add PDE Estimates, Maximum Principles, Regularity, And Analytic PDE Route

- Status: Pending
- Depends on: ANA-T35
- Inputs:
  - weak PDE module
  - Sobolev foundations
  - complex or power-series analytic foundations for analytic PDE theorems
- Code or documentation areas:
  - `Proofs/Ai/Analysis/PDE/Elliptic/`
  - `Proofs/Ai/Analysis/PDE/Parabolic/`
- Tasks:
  - Prove energy estimates.
  - Prove maximum principle for selected elliptic and parabolic equation
    classes.
  - Prove elliptic regularity theorem under explicit coefficient and domain
    assumptions.
  - Prove Cauchy-Kowalevski theorem only after analytic-function or
    power-series infrastructure is available.
  - Prove Holmgren uniqueness theorem after analytic coefficient vocabulary is
    stable.
- Deliverables:
  - Elliptic and parabolic PDE theorem modules.
- Acceptance criteria:
  - Equation class, domain regularity, boundary conditions, and coefficient
    assumptions are explicit.
  - Analytic PDE theorems are not mixed into the weak PDE foundation module.
  - Package or full corpus gate is run before treating PDE modules as stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.PDE.Elliptic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.PDE.Parabolic`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`

### ANA-T37 Add Variational And Optimization Route

- Status: Pending
- Depends on: ANA-T15, ANA-T23, ANA-T26, ANA-T28, ANA-T35
- Inputs:
  - convex tangent route
  - compactness, measure, Hilbert, and Sobolev foundations
- Code or documentation areas:
  - `Proofs/Ai/Analysis/Convex/Optimization/`
  - `Proofs/Ai/Analysis/Variational/Basic/`
  - `Proofs/Ai/Analysis/Variational/CriticalPoint/`
- Tasks:
  - Define convex sets, convex functions, subdifferentials, lower
    semicontinuity, coercivity, admissible variations, and first variation.
  - Prove convex optimality conditions.
  - Prove Euler-Lagrange equation.
  - Prove Weierstrass existence theorem.
  - Prove direct method in the calculus of variations.
  - Prove KKT conditions and Fenchel duality theorem.
  - Add mountain pass theorem only after function-space topology and critical
    point vocabulary are mature.
- Deliverables:
  - Convex optimization and variational theorem modules.
- Acceptance criteria:
  - Direct method depends on compactness and lower semicontinuity, not an
    assumed minimizer.
  - KKT and Fenchel duality reuse convex foundations.
  - Mountain pass theorem is split out if prerequisites are not available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Convex.Optimization`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Variational.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### ANA-T38 Promote Stable Analysis Theorem Closures

- Status: Pending
- Depends on: any completed stable theorem batch from ANA-T01 through ANA-T37
- Inputs:
  - completed corpus modules
  - `develop/npa-mathlib-next-closure-roadmap.md`
  - `develop/proof-corpus-ai-workflow.md`
- Code or documentation areas:
  - future `develop/npa-mathlib-analysis-*-closure-audit.md` files
  - `../npa-mathlib` when materialization is explicitly requested
  - downstream smoke fixtures
- Tasks:
  - Select a closed theorem set whose names and statements are stable.
  - Write a closure audit with import rewrite table, declaration inventory,
    axiom policy, hash inputs, positive gates, and negative checks.
  - Run source-free corpus verification for selected modules.
  - Materialize into `npa-mathlib` only when the closure audit and user intent
    require it.
  - Add downstream smoke tests that consume only vendored certificate bytes.
- Deliverables:
  - Closure audit and, when requested, materialized public package module.
- Acceptance criteria:
  - The closure does not drag immature staging modules into public mathlib.
  - Axiom policy is unchanged or separately justified.
  - Package hash, theorem index, publish plan, and axiom report checks pass.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.X`
  - `cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json`
  - `cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json`
  - `cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json`
  - `cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json`
  - `cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json`
  - `cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json`
  - `cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json`

---

## First Execution Queue

The roadmap's first execution queue maps directly to these milestones:

| Queue ID | Task | Primary milestone |
| --- | --- | --- |
| `ANQ-001` | complete ordered field statement and law package audit | ANA-T01 |
| `ANQ-002` | sequence convergence and limit uniqueness | ANA-T02 |
| `ANQ-003` | Cauchy sequence API and convergence from completeness | ANA-T03 |
| `ANQ-004` | monotone convergence theorem for sequences | ANA-T04 |
| `ANQ-005` | squeeze theorem | ANA-T04 |
| `ANQ-006` | interval nesting theorem | ANA-T04 |
| `ANQ-007` | Bolzano-Weierstrass theorem | ANA-T05 |
| `ANQ-008` | series partial sums and Cauchy criterion | ANA-T06 |
| `ANQ-009` | absolute convergence implies convergence | ANA-T07 |
| `ANQ-010` | comparison test | ANA-T07 |
| `ANQ-011` | ratio and root tests | ANA-T08 |
| `ANQ-012` | continuity API over intervals | ANA-T10 |
| `ANQ-013` | intermediate value theorem | ANA-T11 |
| `ANQ-014` | extreme value theorem | ANA-T12 |
| `ANQ-015` | uniform continuity on compact intervals | ANA-T12 |
| `ANQ-016` | one-dimensional derivative bridge | ANA-T13 |
| `ANQ-017` | Fermat and Rolle theorems | ANA-T14 |
| `ANQ-018` | mean value theorem | ANA-T14 |
| `ANQ-019` | Riemann partition and integrability criterion | ANA-T16 |
| `ANQ-020` | continuous functions are Riemann integrable | ANA-T17 |

After `ANQ-020`, pick one route based on project priority:

- continue to `ANA-T18` for the fundamental theorem of calculus;
- continue to `ANA-T19` for Euclidean inverse/implicit specializations;
- prepare the measure-theory route by completing the ANA-T22 topology
  prerequisite, then continue to `ANA-T24` if measure theory is the next
  strategic foundation.

---

## Review Checklist For Each Milestone

- The target theorem level is recorded and matches the roadmap's `L0` through
  `L3` policy.
- Every imported module is either an existing corpus module, a completed
  earlier milestone, or a consciously deferred public `npa-mathlib` dependency.
- No theorem conclusion is smuggled into the assumptions under a different
  name.
- Sidecars are deterministic and are not cited as proof evidence.
- Local module build, source-free module verification, and changed-only checks
  are used before authoring gates.
- Package or full corpus gates are reserved for package compatibility,
  promotion, release readiness, or high-trust changes.
