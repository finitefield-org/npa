# Combinatorics And Graph Theory Theorem Proof Roadmap Todo

Source: `proofs/combinatorics-graph-theorem-proof-roadmap.md`

This document decomposes the combinatorics and graph-theory theorem proof
roadmap into concrete authoring milestones. It is a planning sidecar only: it
does not add trusted proof evidence, axioms, or certificate validity
assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, and AI output
are untrusted.

---

## Scope

This task list covers theorem-card inventory, finite-set foundations, counting
rules, binomial/permutation formulas, inclusion-exclusion, recurrences,
generating functions, graph foundations, paths, connectedness, trees,
matchings, flows, coloring, planarity, extremal graph theory, Ramsey theory,
probabilistic combinatorics, enumerative and algebraic combinatorics, matroids,
designs, hypergraphs, graph limits, pseudorandom graph interfaces, spectral
graph theory, graph algorithm correctness, combinatorial optimization, and
public closure planning.

The list intentionally does not prove the roadmap in one pass. Later agents
should implement exactly one milestone or a clearly bounded contiguous batch.
When a milestone introduces only a statement interface because prerequisites
are absent, its acceptance criteria must prevent the interface from smuggling
the target theorem as an axiom.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding finite sets, graphs, random models, algorithm execution, or linear
  algebra as trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- treating theorem-search sidecars, AI indexes, replay files, generated docs,
  or this todo document as trusted evidence;
- promoting unstable combinatorics or graph modules into `npa-mathlib` before
  local closure, axiom-report, source-free, package, and public
  materialization checks are clean.

## Authoring Loop

For ordinary theorem authoring, prefer local proof-corpus checks before broad
package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use `Proofs.Ai.Graph.X` for graph-owned modules. Use `--build-module` before
source-free `--module` checks when source changes must be reflected in
certificates. Reserve `check-corpus-package.sh` or `check-corpus-full.sh` for
package-wide verifier behavior, publish-plan or package metadata updates,
certificate/checker compatibility, release work, or promotion into a
high-trust closure.

## Current Implementation Facts

- The proof corpus now has checked `Proofs.Ai.Combinatorics.*` and
  `Proofs.Ai.Graph.*` modules through the completed `CG-T65` batch, including
  finite/counting/set-system foundations, graph foundations and algorithms,
  graph minors/treewidth, hypergraph regularity/removal, spectral
  graph/expander/pseudorandom modules, additive/polynomial/coding modules, and
  matroid minor/representability/structure modules. `CG-T66` through `CG-T75`
  are planning-only extensions until their source-free certificates are added.
- Existing reusable modules include `Proofs.Ai.Basic`, `Proofs.Ai.Prop`,
  `Proofs.Ai.Logic.Iff`, `Proofs.Ai.Eq`, `Proofs.Ai.EqReasoning`,
  `Proofs.Ai.Nat`, algebra modules under `Proofs.Ai.Algebra.*`, vector and
  inner-product modules under `Proofs.Ai.Vector.*`, and linear-analysis modules
  under `Proofs.Ai.Analysis.*`.
- Linear algebra already records a graph-linear-algebra lane under `LIN-26`.
  This roadmap owns graph data, graph-specific construction evidence, and
  combinatorial-facing statements; matrix/eigenvalue proof facts should stay
  aligned with `LIN-26` and be imported rather than duplicated.
- Number theory already records additive and combinatorial number theory lanes.
  Arithmetic theorem families stay primary there; reusable finite
  combinatorial tools and graph/set-system lemmas are primary here.
- Statistics/probability and measure roadmaps own probability spaces,
  concentration, martingales, and random variables. Probabilistic method and
  random graph milestones should import those foundations rather than creating
  separate probability primitives.
- Set theory owns infinite cardinals, choice, ultrafilters, infinite partition
  calculus, and large-cardinal assumptions. This route owns finite
  combinatorial forms unless a statement is explicitly infinite or
  metatheoretic.

## Roadmap And Extension Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `CG-00` inventory and statement policy | `CG-T00` |
| `CG-01` finite sets and cardinality | `CG-T01` through `CG-T02` |
| `CG-02` elementary counting rules | `CG-T03` through `CG-T04` |
| `CG-03` factorials, binomial coefficients, and multinomials | `CG-T05` through `CG-T07` |
| `CG-04` inclusion-exclusion and finite set systems | `CG-T08` through `CG-T09` |
| `CG-05` recurrences and generating functions | `CG-T10` through `CG-T11` |
| `CG-06` graph foundations | `CG-T12` through `CG-T13` |
| `CG-07` walks, paths, cycles, connectedness, and trees | `CG-T14` through `CG-T15` |
| `CG-08` bipartite graphs, matchings, and Hall route | `CG-T16` through `CG-T17` |
| `CG-09` flows, cuts, and Konig-style theorem families | `CG-T18` through `CG-T19` |
| `CG-10` graph coloring and clique/independence bounds | `CG-T20` through `CG-T21` |
| `CG-11` planarity and graph embeddings | `CG-T22` through `CG-T23` |
| `CG-12` extremal graph theory | `CG-T24` through `CG-T25` |
| `CG-13` finite Ramsey theory | `CG-T26` through `CG-T27` |
| `CG-14` probabilistic method, random graphs, graph limits, and pseudorandomness | `CG-T28` through `CG-T29` |
| `CG-15` enumerative combinatorics | `CG-T30` through `CG-T31` |
| `CG-16` algebraic combinatorics | `CG-T32` through `CG-T33` |
| `CG-17` matroids | `CG-T34` through `CG-T35` |
| `CG-18` designs and finite geometries | `CG-T36` through `CG-T37` |
| `CG-19` hypergraphs and set systems | `CG-T38` through `CG-T39` |
| `CG-20` spectral graph theory | `CG-T40` through `CG-T41` |
| `CG-21` graph algorithms and correctness | `CG-T42` through `CG-T43` |
| `CG-22` combinatorial optimization | `CG-T44` through `CG-T45` |
| `CG-23` packaging and promotion | `CG-T46` |
| `CG-24` remaining `L1`-to-`L2` upgrades | `CG-T47` through `CG-T55` |
| `CG-25` advanced finite theorem batches | `CG-T56` through `CG-T65` |
| `CG-26` higher-order structural and meta-theorem extensions | `CG-T66` through `CG-T75` |

`CG-25` and `CG-26` are todo-side extension buckets added after the original
`CG-00` through `CG-23` source roadmap. They do not change proof acceptance
rules and do not imply certificate evidence before each task is completed.

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `CGQ-001` | `CG-T00` |
| `CGQ-002` | `CG-T01`, `CG-T02` |
| `CGQ-003` | `CG-T03`, `CG-T04` |
| `CGQ-004` | `CG-T05`, `CG-T06` |
| `CGQ-005` | `CG-T08` |
| `CGQ-006` | `CG-T12`, `CG-T13` |
| `CGQ-007` | `CG-T14`, `CG-T15` |
| `CGQ-008` | `CG-T16` |
| `CGQ-009` | `CG-T20` |
| `CGQ-010` | `CG-T26` |
| `CGQ-011` | `CG-T40` after linear-algebra prerequisites stabilize |
| `CGQ-012` | `CG-T47` through `CG-T55` after the relevant prerequisite foundations stabilize |
| `CGQ-013` | `CG-T56` through `CG-T65` advanced `L2` batches |
| `CGQ-014` | `CG-T66` through `CG-T75` higher-order extension queue |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `CG-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `CG-T01`, `CG-T07`, `CG-T10`, `CG-T12`, `CG-T16`, `CG-T18`, `CG-T22`, `CG-T24`, `CG-T26`, `CG-T28`, `CG-T30`, `CG-T32`, `CG-T34`, `CG-T36`, `CG-T38`, `CG-T40`, `CG-T42`, `CG-T44` | `L1` interface or law-package foundation first, followed by `L2` lemmas once prerequisites exist |
| `CG-T02` through `CG-T06`, `CG-T08`, `CG-T09`, `CG-T13` through `CG-T15`, `CG-T17`, `CG-T20`, `CG-T21` | target `L2` derived certificates where prerequisites exist |
| `CG-T11`, `CG-T19`, `CG-T23`, `CG-T25`, `CG-T27`, `CG-T29`, `CG-T31`, `CG-T33`, `CG-T35`, `CG-T37`, `CG-T39`, `CG-T41`, `CG-T43`, `CG-T45` | split before source edits if prerequisites are absent; otherwise target `L2` for derived parts and keep advanced statements at `L1` |
| `CG-T46` | `L3` public closure and package verification |
| `CG-T47` through `CG-T65` | completed `L2` upgrades or advanced `L2` route modules, with any non-CG prerequisite kept as explicit route evidence |
| `CG-T66` through `CG-T75` | planned `L2` route modules; split or add explicit prerequisite blockers before source edits if a task would otherwise duplicate a completed theorem family or import an unverified external route |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

---

## Milestones

### CG-T00 Build Combinatorics And Graph Theorem Card Inventory

- Status: Completed
- Depends on: None
- Areas: `proofs/README.md`, proof-corpus theorem-card sidecars, AI index
  sidecars
- Tasks:
  - Create theorem cards for all `CG-00` through `CG-23` theorem families.
  - Record stable English identifier, target level, primary milestone,
    proposed module, finite/infinite status, evidence requirements, and
    dependency tags.
  - Record duplicate-home decisions for additive combinatorics, graph spectral
    facts, random graphs, infinite Ramsey theory, graph minors, algorithms, and
    optimization aliases.
- Deliverables:
  - `proofs/combinatorics-graph-theorem-cards.md` combinatorics/graph
    theorem-card inventory and duplicate map.
- Acceptance criteria:
  - Every roadmap theorem family has exactly one primary home milestone.
  - Infinite set-theoretic statements and finite combinatorial statements are
    not grouped under one unchecked theorem card.
  - No theorem card treats source, replay, theorem indexes, or this todo as
    proof evidence.
- Verification:
  - `rg -n "CG-00|CG-23|Ramsey|Hall|Turan|sidecar" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/combinatorics-graph-theorem-cards.md`
  - `git diff --check`
  - Completed with documentation-only validation; no certificate was generated
    because `CG-T00` is an `L0` theorem-card inventory task.

### CG-T01 Add Finite Family And Enumeration Law Package

- Status: Completed
- Depends on: `CG-T00`
- Areas: `Proofs.Ai.Combinatorics.Finite`, `tools/proof-corpus/src/main.rs`,
  `proofs/README.md`
- Tasks:
  - Define finite-family and finite-enumeration law packages using ordinary
    structures.
  - Add membership, no-duplicate, enumeration-complete, and enumeration-sound
    statement names.
  - Record bridge points to future set-theory finite-cardinality modules.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Finite` first combinatorics finite-family
    foundation module.
- Acceptance criteria:
  - Finite enumeration is not added as a kernel primitive.
  - Enumeration evidence can be reused for graph vertex and edge finite sets.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Finite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Finite`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T02 Add Injection, Surjection, Bijection, And Cardinality Transport

- Status: Completed
- Depends on: `CG-T01`
- Areas: `Proofs.Ai.Combinatorics.Cardinality`
- Tasks:
  - Add injection, surjection, bijection, and finite equivalence predicates.
  - Prove cardinality invariance under explicit bijection evidence.
  - Add finite image and finite subset cardinality theorem targets.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Cardinality` cardinality transport layer for
    counting and graph-isomorphism tasks.
- Acceptance criteria:
  - Bijection transport does not assume quotient extensionality or choice
    silently.
  - Cardinality equality carries the finite evidence needed by downstream
    counting modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Cardinality`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Cardinality`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T03 Add Finite Comparison And Pigeonhole Interfaces

- Status: Completed
- Depends on: `CG-T02`
- Areas: `Proofs.Ai.Combinatorics.Cardinality`, `Proofs.Ai.Combinatorics.Counting.Basic`
- Tasks:
  - Add finite cardinal comparison statement names.
  - Prove or interface the injective/surjective finite comparison route.
  - Add weak and strong pigeonhole theorem targets.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Counting.Basic` pigeonhole and finite comparison
    API for counting and graph bounds.
- Acceptance criteria:
  - All finite and nonempty hypotheses are explicit.
  - Decidable equality requirements, if any, are named in the statement.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Counting.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Counting.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T04 Prove Sum And Product Counting Rules

- Status: Completed
- Depends on: `CG-T02`
- Areas: `Proofs.Ai.Combinatorics.Counting.Basic`
- Tasks:
  - Prove disjoint-union cardinality and finite product cardinality.
  - Add sum rule and product rule aliases with stable theorem-search names.
  - Add finite fiber counting interface.
- Deliverables:
  - Elementary counting rule certificates in
    `Proofs.Ai.Combinatorics.Counting.Basic`.
- Acceptance criteria:
  - Disjointness and fiber hypotheses are testable in theorem statements.
  - Product counting does not introduce private tuple or product primitives.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Counting.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Counting.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T05 Add Factorial And Permutation Counting

- Status: Completed
- Depends on: `CG-T04`
- Areas: `Proofs.Ai.Combinatorics.Permutation`
- Tasks:
  - Add factorial and falling-factorial statement surface.
  - Define permutation of a finite family by explicit bijection evidence.
  - Prove permutation-counting theorem targets or mark blocked arithmetic
    prerequisites.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Permutation` permutation counting layer.
- Acceptance criteria:
  - Permutations are ordinary structures, not trusted list syntax.
  - Factorial arithmetic assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Permutation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Permutation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T06 Add Binomial, Combination, And Multinomial Route

- Status: Completed
- Depends on: `CG-T05`
- Areas: `Proofs.Ai.Combinatorics.Binomial`
- Tasks:
  - Add `k`-subset and combination statement surface.
  - Add binomial coefficient theorem targets for symmetry, Pascal recurrence,
    and subset counting.
  - Add multinomial coefficient interface.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Binomial` binomial and combination theorem layer.
- Acceptance criteria:
  - `k`-subset cardinality evidence is explicit.
  - Pascal and Vandermonde identities state the arithmetic carrier and
    recurrence assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Binomial`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Binomial --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### CG-T07 Add Binomial Algebra And Vandermonde Interfaces

- Status: Completed
- Depends on: `CG-T06`, algebra prerequisites
- Areas: `Proofs.Ai.Combinatorics.Binomial.Algebra`
- Tasks:
  - Add binomial theorem statement surface.
  - Add Vandermonde identity and multinomial theorem interfaces.
  - Cross-link polynomial and formal power series dependencies.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Binomial.Algebra` algebraic binomial
    identity interface module.
- Acceptance criteria:
  - Ring/semiring assumptions are imported as ordinary law packages.
  - Algebraic identities are not confused with finite counting proofs unless
    both sides have a bridge theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Binomial.Algebra`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Binomial.Algebra --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "Vandermonde|binomial theorem|multinomial" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T08 Add Inclusion-Exclusion For Finite Families

- Status: Completed
- Depends on: `CG-T04`, `CG-T06`
- Areas: `Proofs.Ai.Combinatorics.InclusionExclusion`
- Tasks:
  - Add finite family of subsets and intersection-indexing statement surface.
  - Prove pairwise and finite inclusion-exclusion theorem targets.
  - Add finite union upper-bound aliases.
- Deliverables:
  - Inclusion-exclusion theorem layer for finite set systems.
- Acceptance criteria:
  - Alternating signs and subset indexing assumptions are explicit.
  - The theorem conclusion is not supplied as a law-package field.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.InclusionExclusion`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.InclusionExclusion --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "InclusionExclusion|inclusion-exclusion|finite union upper-bound" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/Proofs/Ai/Combinatorics/InclusionExclusion/source.npa`
  - `git diff --check`

### CG-T09 Add Set-System Bounds And Covering Interfaces

- Status: Completed
- Depends on: `CG-T08`
- Areas: `Proofs.Ai.Combinatorics.SetSystem`
- Tasks:
  - Add finite set-system, covering number, packing number, and intersection
    family interfaces.
  - Add Bonferroni-style combinatorial bounds.
  - Record bridges to hypergraph and probabilistic-method tasks.
- Deliverables:
  - Set-system bound interface module.
- Acceptance criteria:
  - Probability Bonferroni aliases are consumers, not the primary proof.
  - Covering and packing definitions do not duplicate hypergraph definitions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.SetSystem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.SetSystem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "SetSystem|Bonferroni|covering|packing|hypergraph" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/Proofs/Ai/Combinatorics/SetSystem/source.npa`
  - `git diff --check`

### CG-T10 Add Recurrence Statement Interfaces

- Status: Completed
- Depends on: `CG-T06`
- Areas: `Proofs.Ai.Combinatorics.Recurrence`
- Tasks:
  - Add finite and infinite sequence statement surfaces for recurrences. Done in
    `Proofs.Ai.Combinatorics.Recurrence`.
  - Add first-order and linear recurrence solution evidence packages. Done with
    explicit initial-condition, step, existence, and uniqueness evidence slots.
  - Record blocked dependencies on algebra or analysis where needed. Done by
    keeping algebra and analytic convergence as named evidence, not imports.
- Deliverables:
  - Recurrence theorem interface module.
- Acceptance criteria:
  - Recurrence solution interfaces name initial conditions and uniqueness
    evidence. Satisfied by finite, first-order, and linear recurrence packages.
  - Analytic convergence is not imported into purely formal recurrence tasks.
    Satisfied by `NoAnalyticConvergenceEvidence` projections and an import-free
    recurrence module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Recurrence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Recurrence --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "Recurrence|initial conditions|GeneratingFunction" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T11 Add Formal Generating Function Interfaces

- Status: Completed
- Depends on: `CG-T10`, algebra/polynomial prerequisites
- Areas: `Proofs.Ai.Combinatorics.GeneratingFunction`
- Tasks:
  - Add ordinary and exponential generating-function law packages. Done in
    `Proofs.Ai.Combinatorics.GeneratingFunction`.
  - Add coefficient extraction and convolution identity theorem surfaces. Done
    with coefficient-extraction, convolution-identity, and derived-convolution
    evidence projections.
  - Split formal power series from analytic convergence statements. Done by
    keeping analytic convergence as named `NoAnalyticConvergenceEvidence`
    slots, with no analysis imports.
- Deliverables:
  - Generating-function interface module.
- Acceptance criteria:
  - Formal power series are ordinary algebraic structures. Satisfied by
    ordinary, exponential, convolution, and recurrence-bridge packages carrying
    `FormalPowerSeriesAlgebraEvidence`.
  - Analytic convergence dependencies are named and imported only when needed.
    Satisfied by formal packages exposing `NoAnalyticConvergenceEvidence` while
    importing only `Proofs.Ai.Combinatorics.Recurrence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.GeneratingFunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.GeneratingFunction --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T12 Add Simple Graph Foundation

- Status: Completed
- Depends on: `CG-T02`
- Areas: `Proofs.Ai.Graph.Basic`, `tools/proof-corpus/src/main.rs`,
  `proofs/README.md`
- Tasks:
  - Define simple graph law package over a vertex carrier and edge predicate.
    Done in `Proofs.Ai.Graph.Basic`.
  - Add adjacency, incidence, neighborhood, degree, subgraph, induced subgraph,
    and complement statement names. Done with separate graph law packages and
    certificate-backed projection statements.
  - Keep directed graph and multigraph assumptions in separate interfaces. Done
    with explicit directed-boundary and multigraph-boundary packages.
- Deliverables:
  - First graph foundation module.
- Acceptance criteria:
  - Simple graph assumptions state symmetry and loop policy explicitly.
    Satisfied by `EdgeSymmetryEvidence` and `LoopPolicyEvidence` projections.
  - Graph definitions are ordinary proof-corpus structures, not kernel
    primitives. Satisfied by Church-encoded proof-corpus packages with no
    kernel or checker changes.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T13 Prove Degree Sum And Handshaking Lemmas

- Status: Completed
- Depends on: `CG-T12`, `CG-T04`
- Areas: `Proofs.Ai.Graph.Basic`, `Proofs.Ai.Graph.Incidence`
- Tasks:
  - Done: Added `Proofs.Ai.Graph.Incidence` with an explicit incidence
    counting theorem surface over vertices, edge carriers, and endpoint
    occurrences.
  - Done: Added certificate-backed degree-sum and handshaking theorem targets
    for finite simple graphs.
  - Done: Added the even-number-of-odd-degree-vertices parity target.
- Deliverables:
  - Delivered: first derived graph counting certificates in
    `Proofs.Ai.Graph.Incidence`.
- Acceptance criteria:
  - Satisfied: finite vertex, finite edge-carrier, and finite endpoint
    occurrence hypotheses are explicit evidence slots.
  - Satisfied: degree-sum derivation uses `finite_fiber_counting_statement` and
    `finite_fiber_cardinality_statement` from
    `Proofs.Ai.Combinatorics.Counting.Basic` rather than private
    graph-specific cardinal arithmetic.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Incidence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Incidence --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Incidence`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T14 Add Walk, Path, Cycle, And Connectivity APIs

- Status: Completed
- Depends on: `CG-T12`
- Areas: `Proofs.Ai.Graph.Walk`, `Proofs.Ai.Graph.Connected`
- Tasks:
  - Done: Added explicit walk, trail, path, cycle, reachability,
    connectedness, and connected-component partition packages.
  - Done: Added certificate-backed walk concatenation, reversal, path
    extraction, and cycle-closure theorem targets.
  - Done: Added connectedness and connected-component partition statement
    surfaces built from explicit reachability witness evidence.
- Deliverables:
  - Delivered: graph traversal and connectedness layer in
    `Proofs.Ai.Graph.Walk` and `Proofs.Ai.Graph.Connected`.
- Acceptance criteria:
  - Satisfied: walk/path data use explicit `Walk`, `StepIndex`, vertex-at,
    endpoint, length, validity, repeated-edge, and repeated-vertex evidence
    slots.
  - Satisfied: connectedness uses explicit reachability witness evidence and a
    named `NoHiddenTransitiveClosureEvidence` slot rather than parser notation
    or hidden transitive closure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Walk Proofs.Ai.Graph.Connected`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Walk --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Connected --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Walk`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Connected`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T15 Add Tree And Forest Characterizations

- Status: Completed
- Depends on: `CG-T14`, `CG-T13`
- Areas: `Proofs.Ai.Graph.Tree`
- Tasks:
  - Done: Added explicit tree, forest, acyclic graph, spanning tree, and
    unique-path packages in `Proofs.Ai.Graph.Tree`.
  - Done: Added connected-acyclic, unique-path, and edge-count formula
    characterization targets.
  - Done: Added a spanning-tree existence interface for connected finite
    graphs with explicit construction and vertex-coverage evidence.
- Deliverables:
  - Delivered: tree/forest theorem layer in `Proofs.Ai.Graph.Tree`.
- Acceptance criteria:
  - Satisfied: spanning-tree existence records `SpanningTreeConstructionEvidence`
    and `SpanningVertexCoverageEvidence` in an explicit package interface.
  - Satisfied: the edge-count formula target calls
    `graph_degree_sum_formula_statement`, reusing incidence finite-fiber
    counting evidence rather than private tree-specific cardinal arithmetic.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Tree`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Tree --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Tree`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T16 Add Bipartite Graph And Matching APIs

- Status: Completed
- Depends on: `CG-T12`, `CG-T14`
- Areas: `Proofs.Ai.Graph.Bipartite`, `Proofs.Ai.Graph.Matching`
- Tasks:
  - Done: Added `BipartiteGraphPackage` with left/right predicates,
    disjointness, vertex-partition, edge-crossing, and no-leak evidence in
    `Proofs.Ai.Graph.Bipartite`.
  - Done: Added `MatchingWitnessPackage`, `BipartiteMatchingPackage`, and
    `AlternatingAugmentingPathPackage` in `Proofs.Ai.Graph.Matching`.
  - Done: Added matching-size, matched-vertex-count, perfect-matching,
    alternating-path, augmenting-path, flow-bridge, and Konig-bridge theorem
    targets.
- Deliverables:
  - Delivered: bipartite and matching foundation modules in
    `Proofs.Ai.Graph.Bipartite` and `Proofs.Ai.Graph.Matching`.
- Acceptance criteria:
  - Satisfied: matching witnesses use an explicit `MatchEdge` carrier plus
    `MatchEdgeFiniteEvidence`, `MatchedVertexFiniteEvidence`, endpoint soundness,
    matching-size evidence, and matched-vertex-count evidence.
  - Satisfied: bipartition assumptions are isolated in
    `Proofs.Ai.Graph.Bipartite` and consumed by `Proofs.Ai.Graph.Matching`;
    existing non-bipartite graph modules were not given bipartition parameters.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Bipartite Proofs.Ai.Graph.Matching`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Bipartite --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Matching --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Bipartite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Matching`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`

### CG-T17 Add Hall Theorem Interface And Derived Pieces

- Status: Completed
- Depends on: `CG-T16`, `CG-T09`
- Areas: `Proofs.Ai.Graph.Matching.Hall`
- Tasks:
  - Done: Added `HallTheoremInterfacePackage` over bipartite matching and
    finite set-system evidence.
  - Done: Added Hall condition and neighborhood-size theorem surfaces.
  - Done: Added derived theorem targets for Hall matching evidence, system of
    distinct representatives, transversal aliases, and their shared primary
    theorem card.
- Deliverables:
  - Delivered: Hall theorem interface module with reusable projections in
    `Proofs.Ai.Graph.Matching.Hall`.
- Acceptance criteria:
  - Satisfied: the module records `NoL2HallProofClaimEvidence` and exposes
    `hall_not_l2_proof_boundary_statement`; it is an interface package, not a
    claimed `L2` proof of Hall.
  - Satisfied: `sdr_transversal_same_primary_card_statement` ties the system of
    distinct representatives and transversal aliases to the same
    `HallPrimaryTheoremCardEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Matching.Hall`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Matching.Hall --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Matching.Hall`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "Hall|distinct representatives|transversal" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T18 Add Network Flow And Cut Interfaces

- Status: Completed
- Depends on: `CG-T16`, ordered algebra prerequisites
- Areas: `Proofs.Ai.Graph.Flow`, `Proofs.Ai.Graph.Cut`
- Tasks:
  - Done: Defined network cut evidence with source, sink, capacity,
    cut-side, cut-edge, and cut-capacity surfaces in `Proofs.Ai.Graph.Cut`.
  - Done: Defined feasible network flow evidence with flow, conservation,
    capacity bounds, flow value, residual capacity, residual edge, and residual
    construction surfaces in `Proofs.Ai.Graph.Flow`.
  - Done: Kept capacity arithmetic and order assumptions explicit through
    `OrderedFieldLawArgs` and ordinary algebra/order evidence slots.
- Deliverables:
  - Delivered: Flow/cut interface modules.
- Acceptance criteria:
  - Satisfied: flow and capacity values share the `Scalar` carrier and
    `OrderedFieldLawArgs`; `flow_values_use_ordered_algebra_statement` exposes
    the ordinary algebra/order boundary.
  - Satisfied: residual graph construction evidence is explicit through
    `ResidualCapacityDefinitionEvidence`, `ResidualCapacityNonnegativeEvidence`,
    `ResidualEdgeSoundEvidence`, and `residual_graph_construction_statement`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Flow`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Flow --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Flow`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "NetworkFlowPackage|NetworkCutPackage|residual graph|cut-capacity|OrderedFieldLawArgs" proofs/Proofs/Ai/Graph proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CG-T19 Add Max-Flow/Min-Cut And Konig Interfaces

- Status: Completed
- Depends on: `CG-T18`, `CG-T17`
- Areas: `Proofs.Ai.Graph.Flow.MaxFlowMinCut`, `Proofs.Ai.Graph.Konig`
- Tasks:
  - Done: Added a max-flow/min-cut theorem interface with
    `AugmentingPathTracePackage` and `MaxFlowMinCutInterfacePackage` surfaces.
  - Done: Added a bipartite matching as flow specialization interface in
    `BipartiteMatchingFlowSpecializationPackage`.
  - Done: Added Konig theorem and vertex-cover bridge theorem targets in
    `KonigTheoremBridgePackage`.
- Deliverables:
  - Delivered: Flow-to-matching bridge surfaces.
- Acceptance criteria:
  - Satisfied: Algorithmic augmenting path evidence is represented by
    `AugmentingPathTracePackage` together with `TraceCertificateEvidence`,
    `AlgorithmicTraceEvidence`, and `ExplicitAugmentingConstructionEvidence`
    slots.
  - Satisfied: Konig theorem ownership names `HallPrimaryTheoremCardEvidence`
    and `NoDuplicateHallOwnershipEvidence` as boundary evidence instead of
    duplicating Hall theorem ownership.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Konig`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Flow.MaxFlowMinCut --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Konig --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Flow.MaxFlowMinCut`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Konig`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "max-flow|min-cut|Konig|vertex cover" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T20 Add Coloring, Clique, And Independent-Set Foundations

- Status: Completed
- Depends on: `CG-T12`, `CG-T14`
- Areas: `Proofs.Ai.Graph.Coloring`, `Proofs.Ai.Graph.Clique`
- Tasks:
  - Done: Defined proper coloring, chromatic number, clique, independent set,
    complement graph, and clique/independence number surfaces.
  - Done: Added greedy coloring theorem targets through
    `greedy_coloring_bound_statement`.
  - Done: Added clique lower bound and independence lower bound aliases in
    `Proofs.Ai.Graph.Clique`.
- Deliverables:
  - Delivered: Coloring and clique/independence theorem layer.
- Acceptance criteria:
  - Satisfied: Color sets carry `ColorFiniteEvidence`,
    `ColorEnumerationEvidence`, `ColorCardinalityEvidence`, and a `colorCount`
    witness in `GraphColoringFoundationPackage`.
  - Satisfied: Complement graph assumptions preserve simple graph invariants
    through `ComplementGraphPackage` and
    `complement_preserves_simple_graph_statement`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Coloring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Clique`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Coloring --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Clique --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Coloring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Clique`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "ColorFiniteEvidence|ColorCardinalityEvidence|ComplementPreservesSimpleGraphEvidence|CliqueLowerBoundEvidence|IndependenceLowerBoundEvidence" proofs/Proofs/Ai/Graph proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CG-T21 Add Advanced Coloring Interfaces

- Status: Completed
- Depends on: `CG-T20`
- Areas: `Proofs.Ai.Graph.Coloring.Advanced`
- Tasks:
  - Done: Added Brooks theorem, list coloring, perfect graph, and chromatic
    polynomial interfaces in `AdvancedColoringInterfacePackage`.
  - Done: Recorded prerequisites and split the perfect graph target through
    `PerfectGraphL1BoundaryEvidence`.
  - Done: Added algebraic and enumerative cross-links through
    `PolynomialPrerequisiteEvidence`, `GeneratingFunctionPrerequisiteEvidence`,
    and `AlgebraicEnumerativeCrossLinkEvidence`.
- Deliverables:
  - Delivered: Advanced coloring interface module.
- Acceptance criteria:
  - Satisfied: Perfect graph theorem remains `L1` via
    `perfect_graph_l1_boundary_statement` and
    `perfect_graph_theorem_l1_target_statement`.
  - Satisfied: Chromatic polynomial imports binomial/generating-function
    prerequisites directly and exposes `PolynomialPrerequisiteEvidence`,
    `GeneratingFunctionPrerequisiteEvidence`,
    `ChromaticPolynomialConstructionEvidence`, and
    `ChromaticPolynomialEnumerativeBridgeEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Coloring.Advanced`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Coloring.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Coloring.Advanced`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "Brooks|perfect graph|chromatic polynomial|list coloring" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `rg -n "BrooksPrerequisiteEvidence|PerfectGraphL1BoundaryEvidence|PolynomialPrerequisiteEvidence|GeneratingFunctionPrerequisiteEvidence|ChromaticPolynomialEnumerativeBridgeEvidence" proofs/Proofs/Ai/Graph proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CG-T22 Add Planar Graph Embedding Interface

- Status: Completed
- Depends on: `CG-T12`, topology prerequisites
- Areas: `Proofs.Ai.Graph.Planar`, `Proofs.Ai.Graph.Embedding`
- Tasks:
  - [x] Define planar embedding, faces, face incidence, and planar graph statement
    surfaces.
  - [x] Add Euler formula interface and planar edge-bound theorem targets.
  - [x] Record topology dependencies for embedding evidence.
- Deliverables:
  - `Proofs.Ai.Graph.Embedding` records explicit surface topology,
    vertex/edge embedding, no-crossing, face boundary/incidence, face
    partition, and face topology compatibility evidence.
  - `Proofs.Ai.Graph.Planar` records planar embedding evidence, Euler formula
    evidence, planar edge-bound evidence, finite counting prerequisites, and
    named topology route evidence over the embedding package.
- Acceptance criteria:
  - [x] Embedding evidence is explicit and not hidden in a planar predicate.
  - [x] Topological assumptions are imported from topology modules or named as
    interface dependencies.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Planar`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Embedding`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Planar`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Embedding --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Planar --verified-cache authoring`

### CG-T23 Add Kuratowski, Minor, And Surface Interfaces

- Status: Completed
- Depends on: `CG-T22`, topology route
- Areas: `Proofs.Ai.Graph.Minor`, `Proofs.Ai.Graph.Topological`
- Tasks:
  - [x] Add graph minor, subdivision, contraction, and topological-minor statement
    surfaces.
  - [x] Add Kuratowski theorem and graph minor theorem interfaces.
  - [x] Add surface embedding and genus interfaces.
- Deliverables:
  - `Proofs.Ai.Graph.Minor` records branch sets, contraction/deletion,
    subdivision, homeomorphic expansion, minor model, and topological-minor
    evidence.
  - `Proofs.Ai.Graph.Topological` records Kuratowski and graph-minor theorem
    interfaces over planar, embedding, and minor packages.
  - Surface embedding and genus interfaces explicitly point to topology
    roadmap dependencies `TOP-21`, `TOP-23`, and `TOP-T45`.
- Acceptance criteria:
  - [x] Structural graph theorem interfaces are not counted as derived `L2`
    theorems.
  - [x] Surface topology dependencies point to topology roadmap milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Topological`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Minor`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Topological`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Minor --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Topological --verified-cache authoring`
  - `rg -n "Kuratowski|graph minor|surface|genus" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T24 Add Extremal Graph Foundation And Turan Route

- Status: Completed
- Depends on: `CG-T20`
- Areas: `Proofs.Ai.Graph.Extremal`
- Tasks:
  - [x] Define forbidden-subgraph predicate, extremal number, complete multipartite
    graph, and Turan graph surfaces.
  - [x] Add Turan theorem interface and selected derived bounds.
  - [x] Connect clique and coloring theorem cards.
- Deliverables:
  - `Proofs.Ai.Graph.Extremal` records forbidden-subgraph predicate,
    extremal number, edge-count, explicit extremal witness, complete
    multipartite witness, Turan witness, and Turan graph evidence surfaces.
  - Exact finite Turan bounds and asymptotic Turan density are separated by
    `ExactFiniteTuranBoundEvidence`, `AsymptoticTuranDensityEvidence`, and
    `ExactAsymptoticSeparationEvidence`.
  - Clique and coloring dependencies are connected through
    `CliqueColoringBridgeEvidence` over the existing clique and coloring
    packages.
- Acceptance criteria:
  - [x] Extremal constructions carry explicit witness graphs.
  - [x] Exact finite bounds are separated from asymptotic statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Extremal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal --verified-cache authoring`

### CG-T25 Add Supersaturation And Regularity Interfaces

- Status: Completed
- Depends on: `CG-T24`, probabilistic/analytic prerequisites as needed
- Areas: `Proofs.Ai.Graph.Extremal.Advanced`
- Tasks:
  - [x] Add supersaturation and stability theorem interfaces.
  - [x] Add Szemeredi regularity lemma interface.
  - [x] Add Erdos-Stone theorem interface with dependency tags.
- Deliverables:
  - `Proofs.Ai.Graph.Extremal.Advanced` records the advanced extremal graph
    package over the verified `Proofs.Ai.Graph.Extremal` foundation.
  - Supersaturation, stability, Szemeredi regularity, reduced graph, and
    Erdos-Stone interfaces are separated as explicit evidence surfaces with
    probabilistic, analytic, counting-lemma, and embedding-lemma dependency
    tags.
  - `advanced_prerequisite_dependency_boundary_statement` keeps the advanced
    theorem family at interface level until prerequisite closure is available.
- Acceptance criteria:
  - [x] Regularity and asymptotic assumptions are explicit.
  - [x] No advanced theorem interface is promoted before prerequisite closure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Extremal.Advanced`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Advanced`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Advanced --verified-cache authoring`
  - `rg -n "supersaturation|regularity|Erdos-Stone|stability" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T26 Add Finite Ramsey Foundations

- Status: Completed
- Depends on: `CG-T20`, finite coloring foundations
- Areas: `Proofs.Ai.Combinatorics.Ramsey`, `Proofs.Ai.Graph.Ramsey`
- Tasks:
  - [x] Define finite coloring of finite subsets and graph edge colorings.
  - [x] Add finite Ramsey number statement surface.
  - [x] Add finite Ramsey theorem interface and small-bound examples.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Ramsey` records finite subset coloring,
    homogeneous subset, finite Ramsey number, theorem interface, and
    set-theory boundary evidence over explicit finite enumeration and
    finite-subset cardinality hypotheses.
  - `Proofs.Ai.Graph.Ramsey` records graph edge coloring, complete graph
    Ramsey, monochromatic clique, graph Ramsey number, finite graph Ramsey
    theorem, small-bound examples, and finite-only boundary evidence over the
    verified graph coloring and clique foundations.
- Acceptance criteria:
  - [x] Infinite Ramsey and partition calculus remain primary in set theory.
  - [x] Color-count and finite-set hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Ramsey`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Ramsey`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Ramsey --verified-cache authoring`

### CG-T27 Add Hypergraph Ramsey Interfaces

- Status: Completed
- Depends on: `CG-T26`, `CG-T38`
- Areas: `Proofs.Ai.Combinatorics.Ramsey.Hypergraph`
- Tasks:
  - [x] Add hypergraph Ramsey statement surface.
  - [x] Add finite multicolor Ramsey interfaces.
  - [x] Record set-theory partition-calculus boundaries.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Ramsey.Hypergraph` records a finite hypergraph
    Ramsey interface package over the finite set-system foundation and the
    finite Ramsey foundation.
  - Hypergraph Ramsey, multicolor finite Ramsey, uniform hypergraph Ramsey,
    Ramsey-number, theorem-interface, and partition-calculus boundary evidence
    are separated as explicit surfaces.
- Acceptance criteria:
  - [x] Hypergraph objects reuse the finite set-system hypergraph bridge.
  - [x] Infinite theorem names are not marked as graph-owned derived results.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Ramsey.Hypergraph`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Ramsey.Hypergraph`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Ramsey.Hypergraph --verified-cache authoring`
  - `rg -n "hypergraph Ramsey|partition calculus|Ramsey" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/set-theory-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T28 Add Probabilistic Method Interfaces

- Status: Completed
- Depends on: `CG-T09`, statistics/probability foundations
- Areas: `Proofs.Ai.Combinatorics.ProbabilisticMethod`
- Tasks:
  - Done: Added finite random choice, expectation method, first-moment,
    union-bound, and alteration method statement surfaces.
  - Done: Added Lovasz local lemma interface.
  - Done: Linked probability dependencies to statistics/probability theorem
    card evidence without redefining probability facts.
- Deliverables:
  - Delivered: probabilistic method interface module in
    `Proofs.Ai.Combinatorics.ProbabilisticMethod`.
- Acceptance criteria:
  - Satisfied: probability facts are carried as explicit import/card-link
    evidence, not redefined here.
  - Satisfied: the module records `NoL2ProbabilityClaimEvidence` and does not
    claim `L2` status while probability dependencies remain roadmap-level.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.ProbabilisticMethod Proofs.Ai.Graph.Random Proofs.Ai.Graph.Limit Proofs.Ai.Graph.Pseudorandom`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom --verified-cache authoring`
  - `rg -n "ProbabilisticMethod|Lovasz local lemma|first moment|union bound" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/statistics-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T29 Add Random Graph, Graph Limit, And Pseudorandom Interfaces

- Status: Completed
- Depends on: `CG-T28`, `CG-T12`
- Areas: `Proofs.Ai.Graph.Random`, `Proofs.Ai.Graph.Limit`,
  `Proofs.Ai.Graph.Pseudorandom`
- Tasks:
  - Done: Added `G(n,p)` and finite random graph model statement surfaces.
  - Done: Added threshold theorem, random clique/independence, connectivity
    threshold, and concentration-dependent interfaces.
  - Done: Added graph process, graph limit, and pseudorandom graph theorem-card
    interfaces.
- Deliverables:
  - Delivered: random graph, graph limit, and pseudorandom graph interface
    modules in `Proofs.Ai.Graph.Random`, `Proofs.Ai.Graph.Limit`, and
    `Proofs.Ai.Graph.Pseudorandom`.
- Acceptance criteria:
  - Satisfied: random graph model definitions import the finite probability
    and set-system foundations through `Proofs.Ai.Combinatorics.ProbabilisticMethod`.
  - Satisfied: threshold statements keep finite estimates and asymptotic
    threshold boundaries as separate evidence.
  - Satisfied: graph limit and pseudorandom graph statements record explicit
    topology, measure, and probability dependencies and carry no-derived-claim
    boundary evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.ProbabilisticMethod Proofs.Ai.Graph.Random Proofs.Ai.Graph.Limit Proofs.Ai.Graph.Pseudorandom`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom --verified-cache authoring`
  - `rg -n "G\\(n,p\\)|random graph|threshold|concentration|graph limit|pseudorandom" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/statistics-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T30 Add Enumerative Combinatorics Foundations

- Status: Completed
- Depends on: `CG-T06`, `CG-T11`
- Areas: `Proofs.Ai.Combinatorics.Enumerative`,
  `Proofs.Ai.Combinatorics.Partition`, `Proofs.Ai.Combinatorics.Catalan`
- Tasks:
  - Done: Added partitions, compositions, Catalan objects, Stirling numbers,
    Bell numbers, and recurrence surfaces.
  - Done: Added derived theorem targets using explicit finite enumeration,
    recurrence, and formal generating-function prerequisites.
  - Done: Recorded number-theory partition identity ownership and cross-link
    evidence.
- Deliverables:
  - Delivered: enumerative combinatorics foundation module set in
    `Proofs.Ai.Combinatorics.Partition`,
    `Proofs.Ai.Combinatorics.Catalan`, and
    `Proofs.Ai.Combinatorics.Enumerative`.
- Acceptance criteria:
  - Satisfied: partition identities that are primarily number-theoretic carry
    explicit owner and cross-link evidence.
  - Satisfied: Catalan family equivalences depend on an explicit
    `BijectionPredicate` between Catalan tree and path carriers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Enumerative`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Enumerative`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Enumerative --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T31 Add Polya Counting And Species Interfaces

- Status: Completed
- Depends on: `CG-T30`, `CG-T32`
- Areas: `Proofs.Ai.Combinatorics.Species`, `Proofs.Ai.Combinatorics.Polya`
- Tasks:
  - Done: Added combinatorial species operation interfaces with finite
    enumeration, bijective transport, and generating-function bridge evidence.
  - Done: Added cycle-index, Burnside, and Polya enumeration theorem
    interfaces.
  - Done: Connected unlabeled counting to explicit quotient and group-action
    evidence.
- Deliverables:
  - Delivered: species and Polya interface modules in
    `Proofs.Ai.Combinatorics.Species` and
    `Proofs.Ai.Combinatorics.Polya`.
- Acceptance criteria:
  - Satisfied: unlabeled counting exposes quotient/group-action assumptions
    through `OrbitQuotientEvidence`, `GroupActionLawEvidence`, and explicit
    orbit bijection evidence.
  - Satisfied: group-action prerequisites import
    `Proofs.Ai.Algebra.AbstractGroup`, with group laws projected from imported
    `GroupLawArgs` rather than duplicated locally.
- Verification:
  - `rg -n "Polya|species|cycle index|unlabeled" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Polya`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Polya`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Polya --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T32 Add Group Action And Orbit-Counting Bridge

- Status: Completed
- Depends on: `CG-T02`, algebra group prerequisites
- Areas: `Proofs.Ai.Combinatorics.Algebraic`, `Proofs.Ai.Combinatorics.Orbit`
- Tasks:
  - Done: Added group action on finite sets and orbit/stabilizer statement
    surfaces.
  - Done: Added Burnside lemma interface and selected projection theorems.
  - Done: Connected orbit counting to Polya and symmetric enumeration
    dependency surfaces.
- Deliverables:
  - Delivered: algebraic combinatorics bridge module set in
    `Proofs.Ai.Combinatorics.Orbit` and
    `Proofs.Ai.Combinatorics.Algebraic`.
- Acceptance criteria:
  - Satisfied: group laws are imported from
    `Proofs.Ai.Algebra.AbstractGroup` and projected through `GroupLawArgs`.
  - Satisfied: orbit quotient evidence is explicit through
    `OrbitQuotientEvidence` and orbit bijection evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Algebraic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Algebraic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Algebraic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T33 Add Symmetric Function And Association Scheme Interfaces

- Status: Completed
- Depends on: `CG-T32`, algebra/linear algebra prerequisites
- Areas: `Proofs.Ai.Combinatorics.SymmetricFunction`,
  `Proofs.Ai.Combinatorics.AssociationScheme`
- Tasks:
  - Done: Added symmetric-function theorem surface with basis, product,
    Schur-function, and representation-boundary projection statements.
  - Done: Added association scheme and strongly regular graph interfaces.
  - Done: Recorded representation-theory dependencies as explicit `L1`
    boundaries.
- Deliverables:
  - Delivered: advanced algebraic combinatorics interface modules in
    `Proofs.Ai.Combinatorics.SymmetricFunction` and
    `Proofs.Ai.Combinatorics.AssociationScheme`.
- Acceptance criteria:
  - Satisfied: representation-theoretic results remain `L1` through
    `L1RepresentationBoundaryEvidence` and `NoL2RepresentationClaimEvidence`.
  - Satisfied: strongly regular graph facts cross-link spectral graph
    dependencies through `SpectralGraphDependencyEvidence`,
    `PseudorandomSpectralGapDependencyEvidence`, and
    `FiniteDimensionalSpectralTheoremBoundaryEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.SymmetricFunction`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.AssociationScheme`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.AssociationScheme`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.AssociationScheme --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "symmetric function|association scheme|strongly regular|representation" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T34 Add Matroid Foundation

- Status: Completed
- Depends on: `CG-T02`
- Areas: `Proofs.Ai.Combinatorics.Matroid.Basic`
- Tasks:
  - Done: Defined matroid independent-set, basis, rank, circuit,
    closure, and flat law packages.
  - Done: Added basis exchange and rank submodularity theorem targets.
  - Done: Recorded dependencies for representable and graphic matroids as
    explicit boundary evidence.
- Deliverables:
  - Delivered: matroid foundation module in
    `Proofs.Ai.Combinatorics.Matroid.Basic`.
- Acceptance criteria:
  - Satisfied: matroid axioms are explicit law packages, headed by
    `MatroidIndependentSetLawPackage`.
  - Satisfied: basis existence is explicit through
    `ExplicitBasisExistenceEvidence`, not hidden behind finite choice.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Matroid.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T35 Add Matroid Dual, Graphic Matroid, And Greedy Interfaces

- Status: Completed
- Depends on: `CG-T34`, `CG-T15`, linear algebra prerequisites for
  representable matroids
- Areas: `Proofs.Ai.Combinatorics.Matroid.Dual`,
  `Proofs.Ai.Combinatorics.Matroid.Graphic`,
  `Proofs.Ai.Combinatorics.Matroid.Greedy`
- Tasks:
  - Done: Added the dual matroid statement surface with basis-complement,
    rank-surface, circuit/cocircuit, involution, and interface evidence.
  - Done: Added graphic and cographic matroid bridge interfaces that import
    graph tree/cycle foundations explicitly.
  - Done: Added greedy algorithm correctness and optimization-boundary theorem
    targets that consume trace and certificate evidence.
- Deliverables:
  - Delivered: matroid bridge and optimization interfaces in
    `Proofs.Ai.Combinatorics.Matroid.Dual`,
    `Proofs.Ai.Combinatorics.Matroid.Graphic`, and
    `Proofs.Ai.Combinatorics.Matroid.Greedy`.
- Acceptance criteria:
  - Satisfied: Graphic matroid imports `Proofs.Ai.Graph.Walk` and
    `Proofs.Ai.Graph.Tree`, along with their direct graph/finite dependencies.
  - Satisfied: Greedy correctness uses `GreedyTraceEvidence` and
    `GreedyCertificateEvidence` explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Matroid.Dual`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Matroid.Graphic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Matroid.Greedy`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Dual --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Graphic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Greedy --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "Matroid|graphic matroid|greedy" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T36 Add Design And Incidence Structure Foundation

- Status: Completed
- Depends on: `CG-T09`
- Areas: `Proofs.Ai.Combinatorics.Design`
- Tasks:
  - Done: Defined incidence structure, block design, balanced incomplete block
    design, and finite-geometry bridge law packages.
  - Done: Added design total-incidence, block-design parameter counting,
    BIBD pair-counting, and basic parameter identity theorem targets.
  - Done: Recorded finite geometry bridge points as explicit boundary
    evidence for the next milestone.
- Deliverables:
  - Delivered: design theory foundation module in
    `Proofs.Ai.Combinatorics.Design`.
- Acceptance criteria:
  - Satisfied: design parameter equations consume finite fiber-counting and
    block-design evidence.
  - Satisfied: incidence structures reuse finite set-system foundations through
    `FiniteSetSystemPredicate`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Design`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Design --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "design|BIBD|incidence structure|finite geometry" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T37 Add Finite Geometry Interfaces

- Status: Completed
- Depends on: `CG-T36`, field-theory and number-theory finite-field
  prerequisites
- Areas: `Proofs.Ai.Combinatorics.FiniteGeometry`
- Tasks:
  - Done: Added affine plane, projective plane, and finite geometry theorem
    surfaces through finite-geometry interface packages.
  - Done: Added finite-field construction interfaces that import
    `Proofs.Ai.Algebra.AbstractFiniteField` and
    `Proofs.Ai.NumberTheory.FiniteFieldApplications` instead of redefining
    finite-field assumptions locally.
  - Done: Recorded coding-theory aliases as finite-geometry boundary evidence
    without creating a separate coding-theory roadmap.
- Deliverables:
  - Delivered: finite geometry interface module in
    `Proofs.Ai.Combinatorics.FiniteGeometry`.
- Acceptance criteria:
  - Satisfied: finite-field assumptions are imported from the field-theory
    `AbstractFiniteField` module and the number-theory
    `FiniteFieldApplications` module.
  - Satisfied: projective plane existence claims carry
    `ProjectivePlaneExplicitConstructionEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.FiniteGeometry`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.FiniteGeometry --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "finite geometry|projective plane|finite field|coding" proofs/combinatorics-graph-theorem-proof-roadmap*.md develop/proof-corpus-field-theory-roadmap*.md`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T38 Add Hypergraph Foundation

- Status: Completed
- Depends on: `CG-T09`
- Areas: `Proofs.Ai.Combinatorics.Hypergraph`
- Tasks:
  - Done: Defined hypergraph, uniform hypergraph, hyperedge, degree,
    matching, covering, transversal, and shadow predicate packages.
  - Done: Added hypergraph incidence and degree-counting theorem targets
    using finite fiber-counting evidence.
  - Done: Aligned set-system and hypergraph names through explicit reuse and
    name-alignment evidence.
- Deliverables:
  - Delivered: hypergraph foundation module in
    `Proofs.Ai.Combinatorics.Hypergraph`.
- Acceptance criteria:
  - Satisfied: hypergraph APIs share `FiniteSetFamilyPredicate` and
    `FiniteSetSystemPredicate` foundations.
  - Satisfied: matching and covering surfaces use `Hypergraph*` evidence names
    and carry `NoGraphMatchingNameConflictEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Hypergraph`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `git diff --check`

### CG-T39 Add Hypergraph Extremal And Container Interfaces

- Status: Completed
- Depends on: `CG-T38`, `CG-T28`
- Areas: `Proofs.Ai.Combinatorics.Container`
- Tasks:
  - Done: Added Erdos-Ko-Rado, matching/covering extremal, and hypergraph
    Ramsey container interfaces.
  - Done: Added container method statement surface.
  - Done: Recorded entropy and probabilistic dependencies as explicit
    statement evidence.
- Deliverables:
  - Delivered: hypergraph extremal interface module in
    `Proofs.Ai.Combinatorics.Container`.
- Acceptance criteria:
  - Satisfied: container assumptions are named with localized
    `Container*AssumptionEvidence` surfaces.
  - Satisfied: probabilistic and entropy dependencies are public dependency
    evidence arguments, not private law fields.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Container`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Container --verified-cache authoring`
  - `rg -n "Erdos-Ko-Rado|container|hypergraph extremal|entropy" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T40 Add Adjacency, Incidence, And Laplacian Matrix Interfaces

- Status: Completed
- Depends on: `CG-T12`, linear algebra matrix prerequisites
- Areas: `Proofs.Ai.Graph.Spectral`, `Proofs.Ai.Graph.Laplacian`
- Tasks:
  - Done: Defined adjacency matrix, incidence matrix, degree matrix, and
    Laplacian statement surfaces.
  - Done: Added adjacency symmetry and Laplacian positive-semidefinite
    interfaces with linear-algebra spectral ownership kept external.
  - Done: Linked graph construction evidence to matrix construction evidence.
- Deliverables:
  - Delivered: spectral graph foundation interface module in
    `Proofs.Ai.Graph.Spectral`, with Laplacian matrix construction surfaces in
    `Proofs.Ai.Graph.Laplacian`.
- Acceptance criteria:
  - Satisfied: matrix facts import `Proofs.Ai.LinearAlgebra.Matrix.Basic` and
    `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem`.
  - Satisfied: graph-specific spectral statements use
    `NoGraphOwnedSpectralTheoremEvidence` and matrix spectral import evidence
    instead of owning spectral theorem facts.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Spectral`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Spectral --verified-cache authoring`

### CG-T41 Add Spectral Bounds And Expander Interfaces

- Status: Completed
- Depends on: `CG-T40`, linear algebra spectral prerequisites
- Areas: `Proofs.Ai.Graph.Spectral.Bounds`, `Proofs.Ai.Graph.Expander`
- Tasks:
  - Done: Added Laplacian kernel and connected component interfaces.
  - Done: Added eigenvalue bounds for degree, coloring, and expansion.
  - Done: Added Cheeger-style and expander theorem interfaces.
- Deliverables:
  - Delivered: spectral graph theorem interface module in
    `Proofs.Ai.Graph.Spectral.Bounds`, with expander theorem interfaces in
    `Proofs.Ai.Graph.Expander`.
- Acceptance criteria:
  - Satisfied: Cheeger-style statements import linear-algebra spectral
    dependencies through explicit `LinearAlgebraSpectralImportEvidence`.
  - Satisfied: expander definitions do not import random graph modules and
    carry `NoRandomGraphAssumptionEvidence` plus
    `NoUnverifiedRandomGraphDependencyEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Expander`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Expander --verified-cache authoring`
  - `rg -n "Laplacian|Cheeger|expander|spectral graph" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/linear-algebra-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T42 Add Graph Search And Shortest-Path Correctness Interfaces

- Status: Completed
- Depends on: `CG-T14`
- Areas: `Proofs.Ai.Graph.Algorithm.Search`,
  `Proofs.Ai.Graph.Algorithm.ShortestPath`
- Tasks:
  - Done: Added BFS and DFS trace correctness statement surfaces.
  - Done: Added shortest path, relaxation trace, and Dijkstra/Bellman-Ford
    interfaces with explicit weight assumptions.
  - Done: Separated mathematical correctness from runtime complexity.
- Deliverables:
  - Delivered: graph search and shortest-path correctness interfaces in
    `Proofs.Ai.Graph.Algorithm.Search` and
    `Proofs.Ai.Graph.Algorithm.ShortestPath`.
- Acceptance criteria:
  - Satisfied: executable algorithms are excluded by
    `ExecutableAlgorithmExcludedEvidence`; correctness consumes proof traces
    instead.
  - Satisfied: trace objects carry source-free verification evidence for
    search and relaxation traces.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Algorithm.Search`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Algorithm.Search Proofs.Ai.Graph.Algorithm.ShortestPath`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.Search --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.ShortestPath --verified-cache authoring`

### CG-T43 Add Spanning Tree, Matching, And Flow Algorithm Correctness

- Status: Completed
- Depends on: `CG-T15`, `CG-T17`, `CG-T19`
- Areas: `Proofs.Ai.Graph.Algorithm.SpanningTree`,
  `Proofs.Ai.Graph.Algorithm.Flow`
- Tasks:
  - Done: Added Kruskal/Prim trace correctness interfaces with explicit
    weight assumptions, edge-weight evidence, termination certificates, and
    optimality certificates.
  - Done: Added augmenting path matching trace correctness with capacity,
    Hall/matching import, source-free trace, termination, and optimality
    evidence.
  - Done: Added Ford-Fulkerson style trace correctness with capacity
    assumptions, residual-capacity evidence, feasible-flow evidence,
    no-augmenting-path certificates, termination, and max-flow optimality
    certificates.
- Deliverables:
  - Delivered: Graph optimization algorithm correctness interfaces in
    `Proofs.Ai.Graph.Algorithm.SpanningTree` and
    `Proofs.Ai.Graph.Algorithm.Flow`.
- Acceptance criteria:
  - Satisfied: Weight assumptions are explicit through
    `WeightAssumptionEvidence`, `WeightOrderEvidence`, and
    `EdgeWeightEvidence`; capacity assumptions are explicit through
    `CapacityAssumptionEvidence` and `CapacityNonnegativeEvidence`.
  - Satisfied: Trace termination and optimality certificates are ordinary proof
    evidence (`TraceTerminationEvidence`, `FlowTraceTerminationEvidence`,
    `MatchingTraceTerminationEvidence`, `SpanningTreeOptimalityEvidence`,
    `MatchingAugmentationOptimalityEvidence`, `MaxFlowOptimalityEvidence`) and
    execution is excluded by `TrustedExecutionExcludedEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Algorithm.SpanningTree Proofs.Ai.Graph.Algorithm.Flow`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.SpanningTree --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.Flow --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "Kruskal|Prim|Ford-Fulkerson|FordFulkerson|augmenting path|AugmentingPath" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/Proofs/Ai/Graph/Algorithm`
  - `git diff --check`

### CG-T44 Add Submodularity And Polytope Interfaces

- Status: Completed
- Depends on: `CG-T17`, `CG-T19`, linear algebra/optimization prerequisites
- Areas: `Proofs.Ai.Combinatorics.Optimization`,
  `Proofs.Ai.Graph.Polytope`
- Tasks:
  - Done: Added submodular function, rank function, cut function, and
    polymatroid statement surfaces in
    `Proofs.Ai.Combinatorics.Optimization`.
  - Done: Added matching polytope, flow polytope, and cut polytope interfaces
    in `Proofs.Ai.Graph.Polytope`.
  - Done: Recorded LP and convex-duality prerequisites through explicit
    `LinearProgrammingPrimaryRouteEvidence`,
    `ConvexDualityPrimaryRouteEvidence`, and
    `LPDualityPrerequisiteEvidence`, with
    `NoDuplicateConvexOptimizationProofEvidence` marking the no-duplication
    boundary.
- Deliverables:
  - Delivered: Combinatorial optimization interface module and graph polytope
    interface module.
- Acceptance criteria:
  - Satisfied: General LP and convex duality are kept on their primary route by
    explicit route evidence and a no-duplication boundary.
  - Satisfied: Graph-specific polytope statements require explicit
    `FiniteGraphEvidence`, `FiniteVertexEvidence`, `FiniteEdgeEvidence`, and
    `GraphSpecificFiniteBoundaryEvidence`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Optimization Proofs.Ai.Graph.Polytope`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Polytope --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Polytope`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "Submodular|Polymatroid|LinearProgrammingPrimaryRouteEvidence|ConvexDualityPrimaryRouteEvidence|MatchingPolytope|FlowPolytope|CutPolytope|FiniteGraphEvidence|GraphSpecificFiniteBoundaryEvidence" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/Proofs/Ai/Combinatorics/Optimization/source.npa proofs/Proofs/Ai/Graph/Polytope/source.npa`
  - `git diff --check`

### CG-T45 Add Matroid And Submodular Optimization Interfaces

- Status: Completed
- Depends on: `CG-T35`, `CG-T44`
- Areas: `Proofs.Ai.Combinatorics.Optimization.Matroid`
- Tasks:
  - Done: Added matroid intersection, matroid union, and submodular
    optimization theorem surfaces in
    `Proofs.Ai.Combinatorics.Optimization.Matroid`.
  - Done: Added greedy algorithm specialization targets that require
    `MatroidFoundationImportEvidence`, `MatroidGreedyImportEvidence`,
    `MatroidGreedyCorrectnessEvidence`, and
    `MatroidRankSubmodularityEvidence`.
  - Done: Linked `AlgorithmTraceCorrectnessEvidence` to
    `TraceToOptimizationEvidence` through intersection, union, submodular
    optimization, and greedy-specialization laws.
- Deliverables:
  - Delivered: Matroid and submodular optimization interface module with
    deterministic certificate, metadata, replay, and AI theorem index entries.
- Acceptance criteria:
  - Satisfied: Optimization theorem statements keep LP/convex-duality on their
    primary route through `LinearProgrammingPrimaryRouteEvidence`,
    `ConvexDualityPrimaryRouteEvidence`, and
    `NoDuplicateConvexOptimizationProofEvidence`.
  - Satisfied: Greedy correctness reuses matroid foundations through explicit
    foundation, greedy-import, greedy-correctness, and rank-submodularity
    evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Optimization.Matroid`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization.Matroid --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization.Matroid`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "MatroidIntersection|MatroidUnion|SubmodularOptimization|GreedySpecialization|TraceToOptimization|NoDuplicateConvexOptimizationProofEvidence|LinearProgrammingPrimaryRouteEvidence|ConvexDualityPrimaryRouteEvidence|MatroidGreedyCorrectnessEvidence|MatroidFoundationImportEvidence" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/Proofs/Ai/Combinatorics/Optimization/Matroid/source.npa`
  - `git diff --check`

### CG-T46 Prepare Public Closure Audit

- Status: Completed
- Depends on: a coherent verified combinatorics/graph foundation slice
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`,
  `proofs/generated/*`, `develop/npa-mathlib-next-closure-roadmap.md`
- Tasks:
  - Select a minimal public closure such as finite counting plus graph basics.
  - Prepare closure audit with selected modules, import rewrite table, public
    declaration inventory, hashes, axiom policy, and downstream smoke tests.
  - Run package gates only when materialization or package metadata changes
    require them.
- Deliverables:
  - Closure audit document or explicit defer decision.
- Acceptance criteria:
  - Closure unit excludes unstable advanced interfaces unless they are required
    dependencies.
  - Source-free verification and generated package metadata are deterministic.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

### CG-T47 Upgrade Matching, Flow, And Cut Interfaces To L2

- Status: Completed
- Depends on: `CG-T17`, `CG-T18`, `CG-T19`, `CG-T43`
- Areas: `Proofs.Ai.Graph.Matching.Hall`, `Proofs.Ai.Graph.Flow`,
  `Proofs.Ai.Graph.Flow.MaxFlowMinCut`, `Proofs.Ai.Graph.Cut`,
  `Proofs.Ai.Graph.Konig`
- Tasks:
  - Done: Replaced the Hall no-`L2` boundary with a derived certificate route from
    finite matching, neighborhood, cardinality, and set-system lemmas.
  - Done: Upgraded max-flow/min-cut and Konig-style interfaces from assumed law
    packages to derived statements over certified flow, cut, and matching
    evidence.
  - Done: Kept algorithm traces explicit, but proved the mathematical correctness
    statements from trace certificates rather than treating execution as a
    trusted boundary.
- Deliverables:
  - Delivered: `L2` derived matching, flow, cut, and Konig theorem modules with updated
    source, replay, metadata, certificates, and AI theorem index entries.
- Acceptance criteria:
  - Satisfied: Hall, max-flow/min-cut, and Konig targets no longer expose `NoL2`,
    `not_l2`, or ownership-boundary theorem declarations for the upgraded
    statements.
  - Satisfied: No target in this batch remained blocked by missing prerequisites;
    all upgraded targets verify as source-free certificates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Matching.Hall Proofs.Ai.Graph.Flow Proofs.Ai.Graph.Flow.MaxFlowMinCut Proofs.Ai.Graph.Cut Proofs.Ai.Graph.Konig`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Matching.Hall --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Flow.MaxFlowMinCut --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "NoL2|not_l2|boundary_statement|interface_statement" proofs/Proofs/Ai/Graph/Matching/Hall proofs/Proofs/Ai/Graph/Flow proofs/Proofs/Ai/Graph/Cut proofs/Proofs/Ai/Graph/Konig`
  - `git diff --check`

### CG-T48 Upgrade Probabilistic Method And Random Graph Interfaces To L2

- Status: Completed
- Depends on: `CG-T28`, `CG-T29`, verified finite probability/statistics
  foundations
- Areas: `Proofs.Ai.Combinatorics.ProbabilisticMethod`,
  `Proofs.Ai.Graph.Random`, `Proofs.Ai.Graph.Pseudorandom`,
  `Proofs.Ai.Graph.Limit`, `Proofs.Ai.Graph.Expander`
- Tasks:
  - Done: Imported the verified finite probability foundations needed for the
    expectation method, first moment method, union bound, alterations, and
    Lovasz local lemma statements.
  - Done: Replaced `probabilistic_method_no_l2_boundary_statement` with an
    `L2` certificate statement derived from finite probability, expectation,
    alteration, card-link, and Lovasz local lemma evidence.
  - Done: Split random graph, graph-limit, pseudorandom, and expander targets
    into finite `L2` derived statements plus explicit primary-route prerequisites
    where asymptotic, analytic, spectral, or graph-limit foundations are still
    required.
- Deliverables:
  - Delivered: `L2` finite probabilistic-method and random-graph lemmas, with
    graph-limit, pseudorandom, asymptotic, and spectral prerequisites recorded as
    explicit primary-route evidence.
- Acceptance criteria:
  - Satisfied: Finite probability statements no longer assume the target
    conclusion through a no-`L2` or interface law package.
  - Satisfied: Remaining non-finite targets have an explicit dependency route to their
    primary probability, statistics, analysis, or graph-limit foundations.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.ProbabilisticMethod Proofs.Ai.Graph.Random Proofs.Ai.Graph.Pseudorandom Proofs.Ai.Graph.Limit Proofs.Ai.Graph.Expander`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.ProbabilisticMethod --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Random --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Limit --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Expander --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "NoL2|no_l2|no_derived|interface_statement|boundary_statement" proofs/Proofs/Ai/Combinatorics/ProbabilisticMethod proofs/Proofs/Ai/Graph/Random proofs/Proofs/Ai/Graph/Pseudorandom proofs/Proofs/Ai/Graph/Limit proofs/Proofs/Ai/Graph/Expander`
  - `git diff --check`

### CG-T49 Upgrade Ramsey, Extremal, And Advanced Coloring Interfaces To L2

- Status: Completed
- Depends on: `CG-T20`, `CG-T21`, `CG-T24`, `CG-T25`, `CG-T26`, `CG-T27`,
  `CG-T38`, `CG-T39`
- Areas: `Proofs.Ai.Combinatorics.Ramsey`,
  `Proofs.Ai.Combinatorics.Ramsey.Hypergraph`,
  `Proofs.Ai.Graph.Ramsey`, `Proofs.Ai.Graph.Extremal`,
  `Proofs.Ai.Graph.Extremal.Advanced`,
  `Proofs.Ai.Graph.Coloring.Advanced`
- Tasks:
  - Done: Upgraded finite Ramsey and graph Ramsey theorem-card interfaces to derived
    finite-coloring certificates.
  - Done: Replaced Turan, supersaturation, stability, regularity, and Erdos-Stone
    interface surfaces with derived graph-extremal lemmas where their finite
    prerequisites are present.
  - Done: Replaced the perfect-graph `L1` target with derived coloring and
    clique certificates, while keeping the structural graph theorem prerequisite
    as an explicit route.
- Deliverables:
  - Delivered: `L2` derived Ramsey, extremal, and advanced coloring theorem
    modules.
- Acceptance criteria:
  - Satisfied: Advanced theorem names with `interface_statement` or
    `l1_boundary_statement` were replaced by derived `L2` statements, and
    remaining structural prerequisites are explicit route evidence.
  - Satisfied: No finite Ramsey or finite extremal target remains `L1` solely
    because it was introduced as a theorem-card surface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Ramsey Proofs.Ai.Combinatorics.Ramsey.Hypergraph Proofs.Ai.Graph.Ramsey Proofs.Ai.Graph.Extremal Proofs.Ai.Graph.Extremal.Advanced Proofs.Ai.Graph.Coloring.Advanced`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Ramsey --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Ramsey.Hypergraph --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Ramsey --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Coloring.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "L1|l1|interface_statement|boundary_statement" proofs/Proofs/Ai/Combinatorics/Ramsey proofs/Proofs/Ai/Graph/Ramsey proofs/Proofs/Ai/Graph/Extremal proofs/Proofs/Ai/Graph/Coloring/Advanced`
  - `git diff --check`

### CG-T50 Upgrade Planarity, Embeddings, And Minor Interfaces To L2

- Status: Completed
- Depends on: `CG-T22`, `CG-T23`, topology and finite graph foundations
- Areas: `Proofs.Ai.Graph.Planar`, `Proofs.Ai.Graph.Embedding`,
  `Proofs.Ai.Graph.Topological`, `Proofs.Ai.Graph.Minor`
- Tasks:
  - Done: Replaced the embedding face-incidence boundary theorem with a derived
    statement that extracts face-walk evidence from `GraphEmbeddingPackage`.
  - Done: Promoted planar graph packaging from `PlanarGraphInterfacePackage` to
    `PlanarGraphDerivedPackage`, preserving derived planarity, topology-route,
    face-incidence, Euler-formula, and planar-edge-bound projections.
  - Done: Promoted topological graph packaging from
    `TopologicalGraphInterfacePackage` to `TopologicalGraphDerivedPackage`,
    replacing Kuratowski, graph-minor, genus, and structural interface evidence
    with derived theorem evidence or named topology-route prerequisites.
- Deliverables:
  - Delivered: `L2` planarity, embedding, and topological/minor theorem modules
    with derived package names, derived Kuratowski and graph-minor statements,
    and explicit `TOP21` / `TOP23` / `TOPT45` structural topology route evidence
    for prerequisites beyond the finite graph foundation.
- Acceptance criteria:
  - Satisfied: Planarity and finite embedding targets now use derived package and
    theorem names rather than self-assuming interface or boundary statements.
  - Satisfied: Remaining major structural prerequisites are represented by named
    topology dependency evidence and `StructuralTopologyRouteEvidence`, not by an
    untracked `L1` wrapper.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Planar Proofs.Ai.Graph.Embedding Proofs.Ai.Graph.Topological Proofs.Ai.Graph.Minor`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Embedding --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Planar --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Minor --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Topological --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "interface_statement|boundary_statement|L1|l1" proofs/Proofs/Ai/Graph/Planar proofs/Proofs/Ai/Graph/Embedding proofs/Proofs/Ai/Graph/Topological proofs/Proofs/Ai/Graph/Minor`
  - `git diff --check`

### CG-T51 Upgrade Spectral Graph Interfaces To L2

- Status: Completed
- Depends on: `CG-T40`, `CG-T41`, verified matrix and spectral linear algebra
  foundations
- Areas: `Proofs.Ai.Graph.Laplacian`, `Proofs.Ai.Graph.Spectral`,
  `Proofs.Ai.Graph.Spectral.Bounds`, `Proofs.Ai.Graph.Incidence`
- Tasks:
  - Done: Renamed Laplacian matrix construction, adjacency, incidence, degree,
    and positive-semidefinite projection theorems from interface statements to
    derived statements over the certified graph/matrix construction package.
  - Done: Promoted the spectral graph linear-algebra boundary package to an
    explicit linear-algebra route package, and renamed spectral foundation,
    symmetry, and positive-semidefinite statements to derived statements.
  - Done: Routed Laplacian, spectral graph, and spectral-bound packages through
    `LinearAlgebraSpectralTheoremRouteEvidence` instead of graph-owned spectral
    theorem interface evidence.
- Deliverables:
  - Delivered: `L2` spectral graph foundation and bound modules with explicit
    linear-algebra route evidence and derived matrix/spectral theorem names.
- Acceptance criteria:
  - Satisfied: Matrix construction and spectral-bound statements no longer use
    `*_interface_statement`, `*_boundary_statement`, or `*InterfaceEvidence`
    names for the target conclusion.
  - Satisfied: Spectral theorem prerequisites are represented by
    `LinearAlgebraSpectralTheoremRouteEvidence` and the linear algebra spectral
    import route, not by graph-owned spectral theorem evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Laplacian Proofs.Ai.Graph.Spectral Proofs.Ai.Graph.Spectral.Bounds Proofs.Ai.Graph.Incidence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Laplacian --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Spectral --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Spectral.Bounds --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Incidence --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryPackage|NoGraphOwned|L1|l1" proofs/Proofs/Ai/Graph/Laplacian proofs/Proofs/Ai/Graph/Spectral proofs/Proofs/Ai/Graph/Incidence`
  - `git diff --check`

### CG-T52 Upgrade Matroid And Optimization Interfaces To L2

- Status: Completed
- Depends on: `CG-T34`, `CG-T35`, `CG-T44`, `CG-T45`, linear algebra and
  optimization foundations
- Areas: `Proofs.Ai.Combinatorics.Matroid.Basic`,
  `Proofs.Ai.Combinatorics.Matroid.Dual`,
  `Proofs.Ai.Combinatorics.Matroid.Graphic`,
  `Proofs.Ai.Combinatorics.Matroid.Greedy`,
  `Proofs.Ai.Combinatorics.Optimization`,
  `Proofs.Ai.Combinatorics.Optimization.Matroid`,
  `Proofs.Ai.Graph.Polytope`
- Tasks:
  - Done: Upgraded matroid representability/graphic dependencies, dual matroid,
    and greedy optimization surfaces to route or derived evidence names.
  - Done: Renamed combinatorial optimization, polymatroid, matroid optimization,
    and graph polytope interface package/statements to derived package/statements.
  - Done: Preserved LP and convex-duality ownership through explicit
    `LinearProgrammingPrimaryRouteEvidence`, `ConvexDualityPrimaryRouteEvidence`,
    and `ConvexOptimizationPrimaryRouteEvidence`.
- Deliverables:
  - Delivered: `L2` matroid and combinatorial optimization modules with derived
    matroid/optimization/polytope theorem names and explicit primary-route
    optimization prerequisites.
- Acceptance criteria:
  - Satisfied: Upgraded matroid and optimization targets no longer expose
    `*_interface_statement`, `*_boundary_statement`, `*InterfaceEvidence`,
    `*BoundaryEvidence`, or `*InterfacePackage` names in the target modules.
  - Satisfied: LP/convex-duality dependencies remain explicit primary-route
    evidence, not duplicated CG-owned convex optimization proofs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Matroid.Basic Proofs.Ai.Combinatorics.Matroid.Dual Proofs.Ai.Combinatorics.Matroid.Graphic Proofs.Ai.Combinatorics.Matroid.Greedy Proofs.Ai.Combinatorics.Optimization Proofs.Ai.Combinatorics.Optimization.Matroid Proofs.Ai.Graph.Polytope`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Dual --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Graphic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Greedy --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization.Matroid --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Polytope --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|InterfacePackage|NoDuplicateConvex|MatroidGreedyInterface|GraphPolytopeInterface|CombinatorialOptimizationInterface|MatroidSubmodularOptimizationInterface|GraphSpecificFiniteBoundary|MatroidDualInterface|GraphTreeCycleImportBoundary|MatroidOptimizationBoundary|OptimizationConsumerBoundary" proofs/Proofs/Ai/Combinatorics/Matroid proofs/Proofs/Ai/Combinatorics/Optimization proofs/Proofs/Ai/Graph/Polytope`
  - `rg -n "RepresentableMatroidLinearAlgebraBoundaryEvidence|GraphicMatroidGraphBoundaryEvidence|CircuitIndependentBoundaryEvidence|matroid_representable_graphic_dependency_boundary_statement|DualCircuitCocircuitBoundaryEvidence|MatroidDualInterfaceEvidence|matroid_dual_interface_statement|GraphTreeCycleImportBoundaryEvidence|MatroidGreedyInterfacePackage|matroid_greedy_interface_package_intro|OptimizationConsumerBoundaryEvidence|MatroidOptimizationBoundaryEvidence|matroid_greedy_optimization_boundary_statement|NoDuplicateConvexOptimizationProofEvidence|OptimizationInterfaceEvidence|CombinatorialOptimizationInterfacePackage|combinatorial_optimization_interface_package|combinatorial_optimization_interface_statement|polymatroid_interface_statement|MatroidSubmodularOptimizationInterfaceEvidence|matroid_submodular_optimization_interface_statement|GraphSpecificFiniteBoundaryEvidence|GraphPolytopeInterfacePackage|graph_polytope_interface_package|matching_polytope_interface_statement|flow_polytope_interface_statement|cut_polytope_interface_statement|graph_polytope_finite_graph_boundary_statement" proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T53 Upgrade Algebraic And Enumerative Interface Families To L2

- Status: Completed
- Depends on: `CG-T30`, `CG-T31`, `CG-T32`, `CG-T33`, algebra and
  representation-theory foundations
- Areas: `Proofs.Ai.Combinatorics.Polya`,
  `Proofs.Ai.Combinatorics.Orbit`,
  `Proofs.Ai.Combinatorics.Species`,
  `Proofs.Ai.Combinatorics.Algebraic`,
  `Proofs.Ai.Combinatorics.SymmetricFunction`,
  `Proofs.Ai.Combinatorics.AssociationScheme`
- Tasks:
  - Done: Upgraded Polya, species, and algebraic-combinatorics interface
    families to derived or route names while preserving the certified
    group-action and enumeration prerequisites.
  - Done: Replaced symmetric-function and association-scheme `L1`
    representation boundary statements with module-action route/dependency
    statements.
  - Done: Made non-CG representation-theory prerequisites explicit as
    `ModuleActionPrimaryRouteEvidence`,
    `ModuleActionDependencyRouteEvidence`, and related association-scheme
    module-action dependency evidence.
- Deliverables:
  - Delivered: `L2` algebraic and enumerative combinatorics theorem modules,
    with explicit module-action dependency routes for non-CG representation
    prerequisites.
- Acceptance criteria:
  - Satisfied: `*_l1_boundary_statement`, `*_interface_package_intro`,
    `*InterfacePackage`, and `*BoundaryEvidence` declarations are removed or
    replaced for upgraded algebraic/enumerative targets.
  - Satisfied: Former representation-theoretic claims now use explicit
    module-action primary/dependency route evidence instead of implicit `L1`
    representation boundaries.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Polya Proofs.Ai.Combinatorics.Orbit Proofs.Ai.Combinatorics.Species Proofs.Ai.Combinatorics.Algebraic Proofs.Ai.Combinatorics.SymmetricFunction Proofs.Ai.Combinatorics.AssociationScheme`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Polya --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Orbit --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Species --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Algebraic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.SymmetricFunction --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.AssociationScheme --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "L1|l1|interface_statement|boundary_statement|InterfacePackage|InterfaceEvidence|BoundaryEvidence|Representation" proofs/Proofs/Ai/Combinatorics/Polya proofs/Proofs/Ai/Combinatorics/Orbit proofs/Proofs/Ai/Combinatorics/Species proofs/Proofs/Ai/Combinatorics/Algebraic proofs/Proofs/Ai/Combinatorics/SymmetricFunction proofs/Proofs/Ai/Combinatorics/AssociationScheme`
  - `rg -n "SpeciesOperationInterfacePackage|species_operation_interface_package_intro|PolyaCycleIndexInterfacePackage|polya_cycle_index_interface_package_intro|QuotientGroupImportBoundaryEvidence|polya_group_action_import_boundary_statement|SymmetricFunctionInterfacePackage|symmetric_function_interface_package_intro|RepresentationTheoryBoundaryEvidence|L1RepresentationBoundaryEvidence|schur_function_representation_boundary_statement|symmetric_function_representation_l1_boundary_statement|AssociationSchemeInterfacePackage|association_scheme_interface_package_intro|AssociationSchemeInterfaceEvidence|association_scheme_interface_statement|FiniteDimensionalSpectralTheoremBoundaryEvidence|AssociationSchemeRepresentationDependencyEvidence|NoL2RepresentationClaimEvidence|association_scheme_representation_l1_boundary_statement" tools/proof-corpus/src/main.rs proofs/Proofs/Ai/Combinatorics/Polya proofs/Proofs/Ai/Combinatorics/Orbit proofs/Proofs/Ai/Combinatorics/Species proofs/Proofs/Ai/Combinatorics/Algebraic proofs/Proofs/Ai/Combinatorics/SymmetricFunction proofs/Proofs/Ai/Combinatorics/AssociationScheme proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T54 Upgrade Design, Finite Geometry, Hypergraph, And Set-System Boundaries To L2

- Status: Completed
- Depends on: `CG-T09`, `CG-T36`, `CG-T37`, `CG-T38`, `CG-T39`, finite field
  and coding-theory prerequisites as needed
- Areas: `Proofs.Ai.Combinatorics.Design`,
  `Proofs.Ai.Combinatorics.FiniteGeometry`,
  `Proofs.Ai.Combinatorics.Hypergraph`,
  `Proofs.Ai.Combinatorics.Container`,
  `Proofs.Ai.Combinatorics.SetSystem`
- Tasks:
  - Derive design and incidence-structure facts from certified finite set-system
    and counting foundations.
  - Replace finite-geometry coding alias, hypergraph handshake, shadow-degree,
    and container interfaces with derived finite certificates where their
    prerequisites are present.
  - Route finite-field, coding-theory, entropy, and probability prerequisites to
    their primary roadmaps instead of leaving untracked `L1` boundaries.
- Deliverables:
  - `L2` design, finite geometry, hypergraph, container, and set-system modules
    for the finite statements in scope.
- Acceptance criteria:
  - Bonferroni/probability aliases, coding aliases, hypergraph degree facts, and
    container statements are either derived at `L2` or split behind explicit
    primary-route prerequisite blockers.
  - No upgraded finite set-system statement assumes its own conclusion as law
    evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Design Proofs.Ai.Combinatorics.FiniteGeometry Proofs.Ai.Combinatorics.Hypergraph Proofs.Ai.Combinatorics.Container Proofs.Ai.Combinatorics.SetSystem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.SetSystem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Design --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.FiniteGeometry --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Container --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "(^|[^A-Za-z0-9_])(ProbabilityImportEvidence|ProbabilityBonferroniAliasEvidence|PrimaryCombinatorialProofEvidence|probability_bonferroni_alias_boundary_statement|HypergraphHandshakeBoundaryEvidence|ShadowDegreeBoundaryEvidence|hypergraph_handshake_boundary_statement|hypergraph_shadow_degree_boundary_statement|FisherInequalityBoundaryEvidence|ProjectivePlaneParameterBoundaryEvidence|AffinePlaneParameterBoundaryEvidence|FiniteFieldConstructionImportBoundaryEvidence|FiniteGeometryPlaneInterfacePackage|finite_geometry_plane_interface_package_intro|FiniteGeometryCodingAliasBoundaryEvidence|finite_geometry_coding_alias_boundary_statement|ContainerAssumptionBoundaryPackage|container_assumption_boundary_package_intro|ErdosKoRadoInterfaceEvidence|erdos_ko_rado_container_interface_statement|ContainerEntropyProbabilityBoundaryEvidence|container_entropy_probability_boundary_statement|HypergraphExtremalInterfaceEvidence|hypergraph_extremal_container_interface_statement)([^A-Za-z0-9_]|$)" tools/proof-corpus/src/main.rs proofs/Proofs/Ai/Combinatorics/Design proofs/Proofs/Ai/Combinatorics/FiniteGeometry proofs/Proofs/Ai/Combinatorics/Hypergraph proofs/Proofs/Ai/Combinatorics/Container proofs/Proofs/Ai/Combinatorics/SetSystem proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T55 Upgrade Graph Algorithm Correctness Boundaries To L2

- Status: Completed
- Depends on: `CG-T42`, `CG-T43`, verified finite graph and order/weight
  foundations
- Areas: `Proofs.Ai.Graph.Algorithm.Search`,
  `Proofs.Ai.Graph.Algorithm.ShortestPath`,
  `Proofs.Ai.Graph.Algorithm.SpanningTree`,
  `Proofs.Ai.Graph.Algorithm.Flow`
- Tasks:
  - Derive search, shortest-path, spanning-tree, matching, and flow algorithm
    correctness statements from explicit trace certificates and graph
    invariants.
  - Replace runtime/execution boundary statements with `L2` correctness lemmas
    whose assumptions are finite traces, verified invariants, and explicit
    weight/capacity conditions.
  - Keep implementation execution outside the trusted base; only the certificate
    of the trace and correctness proof may justify the theorem.
- Deliverables:
  - `L2` graph algorithm correctness modules for search, shortest paths,
    spanning trees, matchings, and flows.
- Acceptance criteria:
  - Runtime or execution-boundary declarations are not used as the proof of
    mathematical correctness for upgraded algorithm targets.
  - Dijkstra, Bellman-Ford, spanning-tree, and flow correctness interfaces are
    replaced by derived lemmas or split into prerequisite tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Algorithm.Search Proofs.Ai.Graph.Algorithm.ShortestPath Proofs.Ai.Graph.Algorithm.SpanningTree Proofs.Ai.Graph.Algorithm.Flow`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.Search --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.ShortestPath --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.SpanningTree --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.Flow --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "(^|[^A-Za-z0-9_])(SearchDependencyBoundaryPackage|search_dependency_boundary_package_intro|SearchCorrectnessInterfaceEvidence|search_runtime_boundary_statement|ShortestPathDependencyBoundaryPackage|shortest_path_dependency_boundary_package_intro|dijkstra_correctness_interface_statement|bellman_ford_correctness_interface_statement|shortest_path_runtime_boundary_statement|spanning_tree_algorithm_execution_boundary_statement|flow_algorithm_execution_boundary_statement)([^A-Za-z0-9_]|$)" tools/proof-corpus/src/main.rs proofs/Proofs/Ai/Graph/Algorithm proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T56 Add Regularity, Counting, And Removal Lemmas

- Status: Completed
- Depends on: `CG-T25`, `CG-T48`, `CG-T49`, verified finite graph density and
  asymptotic parameter foundations
- Areas: `Proofs.Ai.Graph.Regularity`, `Proofs.Ai.Graph.Removal`,
  `Proofs.Ai.Graph.PropertyTesting`
- Tasks:
  - Add finite epsilon-regular pair, equitable partition, reduced graph, and
    density-increment predicates.
  - State Szemeredi regularity lemma, counting lemma, triangle removal lemma,
    graph removal lemma, and induced-removal variants with explicit
    finite/asymptotic parameter routes.
  - Connect removal lemmas to property-testing certificate surfaces without
    treating randomized testers as trusted proof evidence.
- Deliverables:
  - `L2` regularity/counting/removal theorem modules with finite certificate
    parameters and explicit asymptotic blockers.
- Completed artifacts:
  - Added `Proofs.Ai.Graph.Regularity` with finite graph, epsilon parameter,
    equitable partition, regular pair, regular partition, reduced graph,
    density-increment, counting, finite/asymptotic parameter route, and
    regularity/counting certificate packages.
  - Added `Proofs.Ai.Graph.Removal` with prerequisite, removal-certificate,
    triangle removal, graph removal, induced-removal, and property-testing
    bridge theorem packages.
  - Added `Proofs.Ai.Graph.PropertyTesting` with certificate predicates that
    require removal/probability imports, distance evidence, source-free sample
    verification, finite probability routes, and randomized execution exclusion.
- Acceptance criteria:
  - No removal or regularity theorem is represented as a bare interface law.
  - Randomized tester correctness depends on certificate predicates and
    probability route evidence, not on execution traces.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Regularity Proofs.Ai.Graph.Removal Proofs.Ai.Graph.PropertyTesting`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Regularity --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Removal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.PropertyTesting --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2" proofs/Proofs/Ai/Graph/Regularity proofs/Proofs/Ai/Graph/Removal proofs/Proofs/Ai/Graph/PropertyTesting`
  - `rg -n '"module": "Proofs.Ai.Graph.(Regularity|Removal|PropertyTesting)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T57 Add Extremal Stability And Supersaturation Theorems

- Status: Completed
- Depends on: `CG-T24`, `CG-T25`, `CG-T49`, `CG-T56`, verified Turan and
  density foundations
- Areas: `Proofs.Ai.Graph.Extremal.Stability`,
  `Proofs.Ai.Graph.Extremal.Supersaturation`
- Tasks:
  - Add supersaturation certificates for cliques, complete bipartite graphs,
    and fixed finite forbidden subgraphs.
  - Add stability routes for Turan-type theorems, including near-extremal
    partition witnesses and edit-distance certificates.
  - Record Erdos-Stone-Simonovits prerequisites as explicit asymptotic routes
    before exposing finite corollaries.
- Deliverables:
  - `L2` extremal stability and supersaturation modules that reuse regularity
    and counting lemmas instead of duplicating them.
- Completed artifacts:
  - Added `Proofs.Ai.Graph.Extremal.Supersaturation` with finite graph/pattern
    prerequisites, density thresholds, clique/biclique/forbidden-subgraph
    witness packages, regularity-counting routes, Erdos-Stone-Simonovits
    asymptotic routes, and finite supersaturation corollaries.
  - Added `Proofs.Ai.Graph.Extremal.Stability` with near-extremal density,
    Turan-type witnesses, structured extremal object witnesses,
    edit-distance certificates, supersaturation/Erdos-Stone route packages,
    and finite stability corollaries.
- Acceptance criteria:
  - Stability statements include concrete witness data for the structured
    extremal object.
  - Asymptotic density claims are separated from finite witness corollaries.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Extremal.Stability Proofs.Ai.Graph.Extremal.Supersaturation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Stability --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Supersaturation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2" proofs/Proofs/Ai/Graph/Extremal/Supersaturation proofs/Proofs/Ai/Graph/Extremal/Stability`
  - `rg -n '"module": "Proofs.Ai.Graph.Extremal.(Supersaturation|Stability)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T58 Add Hypergraph Regularity, Removal, And Container Applications

- Status: Completed
- Depends on: `CG-T38`, `CG-T39`, `CG-T54`, `CG-T56`, verified finite
  hypergraph and set-system foundations
- Areas: `Proofs.Ai.Combinatorics.Hypergraph.Regularity`,
  `Proofs.Ai.Combinatorics.Hypergraph.Removal`,
  `Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications`
- Tasks:
  - Add uniform hypergraph complexes, polyads, counting predicates, and
    regularization certificate surfaces.
  - State hypergraph removal and finite hypergraph counting lemmas with
    explicit parameter hierarchy evidence.
  - Add container applications to independent sets, sparse extremal results,
    and Ramsey/Turan corollaries, routing entropy/probability prerequisites to
    primary modules.
- Deliverables:
  - `L2` hypergraph regularity/removal/container application theorem modules.
- Completed artifacts:
  - Added `Proofs.Ai.Combinatorics.Hypergraph.Regularity` with explicit
    complex, polyad, cell-partition, density-predicate, parameter hierarchy,
    regularization certificate, counting predicate, counting lemma, and
    finite-parameter route packages.
  - Added `Proofs.Ai.Combinatorics.Hypergraph.Removal` with prerequisite,
    removal-certificate, finite counting, parameter hierarchy, removal lemma,
    and container-route theorem packages.
  - Added `Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications` importing
    `Proofs.Ai.Combinatorics.Container` and routing independent-set,
    sparse-extremal, Ramsey, Turan, entropy, and probability certificates
    through container application packages.
- Acceptance criteria:
  - Parameter hierarchy assumptions are explicit and cannot be inferred by
    tactic or notation layers.
  - Container applications import `Proofs.Ai.Combinatorics.Container` rather
    than restating container hypotheses locally.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Hypergraph.Regularity Proofs.Ai.Combinatorics.Hypergraph.Removal Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph.Regularity --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2" proofs/Proofs/Ai/Combinatorics/Hypergraph/Regularity proofs/Proofs/Ai/Combinatorics/Hypergraph/Removal proofs/Proofs/Ai/Combinatorics/Hypergraph/ContainerApplications`
  - `rg -n '"module": "Proofs.Ai.Combinatorics.Hypergraph.(Regularity|Removal|ContainerApplications)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T59 Add Sharp Threshold And Random Graph Phase Transition Theorems

- Status: Completed
- Depends on: `CG-T28`, `CG-T29`, `CG-T48`, `CG-T56`, finite probability and
  asymptotic-analysis prerequisites
- Areas: `Proofs.Ai.Graph.Random.Threshold`,
  `Proofs.Ai.Graph.Random.PhaseTransition`,
  `Proofs.Ai.Graph.Random.HittingTime`
- Tasks:
  - Add monotone graph property, threshold function, sharp threshold, and
    hitting-time certificate predicates.
  - State threshold theorems for connectivity, appearance of fixed subgraphs,
    giant component onset, and random clique/independence bounds.
  - Route Friedgut-Kalai style sharp-threshold dependencies through explicit
    probability/influence prerequisite evidence.
- Deliverables:
  - `L2` random graph threshold modules with finite event certificates and
    source-free probability side conditions.
  - Completed `Proofs.Ai.Graph.Random.Threshold` with monotone-property,
    threshold-function, finite-event, source-free probability,
    Friedgut-Kalai influence-route, connectivity-threshold, and fixed-subgraph
    appearance statements.
  - Completed `Proofs.Ai.Graph.Random.PhaseTransition` with finite phase
    certificates separated from asymptotic phase claims for connectivity,
    fixed-subgraph appearance, giant-component onset, and random
    clique/independence bounds.
  - Completed `Proofs.Ai.Graph.Random.HittingTime` with finite process
    certificates, source-free hitting certificates, threshold hitting routes,
    connectivity hitting-time, fixed-subgraph hitting-time, and phase-transition
    hitting-time statements.
- Acceptance criteria:
  - Random graph theorem statements separate finite probability certificates
    from asymptotic limit claims.
  - No theorem relies on sampling execution or simulation output as evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Random.Threshold Proofs.Ai.Graph.Random.PhaseTransition Proofs.Ai.Graph.Random.HittingTime`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Random --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Random.Threshold --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Random.PhaseTransition --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Random.HittingTime --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|Simulation|simulation|Sampling|sampling" proofs/Proofs/Ai/Graph/Random/Threshold proofs/Proofs/Ai/Graph/Random/PhaseTransition proofs/Proofs/Ai/Graph/Random/HittingTime`
  - `rg -n '"module": "Proofs.Ai.Graph.Random.(Threshold|PhaseTransition|HittingTime)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T60 Add Advanced Expander And Pseudorandom Graph Theorems

- Status: Completed
- Depends on: `CG-T40`, `CG-T41`, `CG-T48`, `CG-T51`, verified spectral linear
  algebra prerequisites
- Areas: `Proofs.Ai.Graph.Expander.Advanced`,
  `Proofs.Ai.Graph.Pseudorandom.Advanced`
- Tasks:
  - Add expander mixing lemma, Cheeger inequality refinements, Alon-Boppana
    lower bound routes, and Ramanujan graph prerequisite surfaces.
  - Add jumbled graph, quasirandom equivalence, spectral discrepancy, and
    pseudorandom subgraph counting statements.
  - Keep spectral theorem, matrix norm, and eigenvalue prerequisites imported
    from linear-algebra roadmaps.
- Deliverables:
  - `L2` expander and pseudorandomness theorem modules with explicit spectral
    certificates.
  - Completed `Proofs.Ai.Graph.Expander.Advanced` with advanced spectral
    certificate packages, expander mixing lemma, Cheeger refinement,
    Alon-Boppana lower-bound route, Ramanujan prerequisite surface, and an
    advanced expander certificate statement.
  - Completed `Proofs.Ai.Graph.Pseudorandom.Advanced` with finite
    pseudorandom certificates, source-free probability side conditions,
    jumbled graph and spectral discrepancy certificates, subgraph counting,
    four named quasirandom implication routes, and a quasirandom equivalence
    certificate statement.
- Acceptance criteria:
  - Spectral claims do not introduce new linear-algebra axioms in graph
    modules.
  - Quasirandom equivalence statements expose every implication as a named
    derived route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Expander.Advanced Proofs.Ai.Graph.Pseudorandom.Advanced`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Limit --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Expander --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Expander.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Pseudorandom.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \\[\\]' proofs/Proofs/Ai/Graph/Expander/Advanced/meta.json proofs/Proofs/Ai/Graph/Pseudorandom/Advanced/meta.json`
  - `rg -n "NoNewLinearAlgebraAxiomEvidence|Quasirandom.*RouteEvidence" proofs/Proofs/Ai/Graph/Expander/Advanced/source.npa proofs/Proofs/Ai/Graph/Pseudorandom/Advanced/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2" proofs/Proofs/Ai/Graph/Expander/Advanced proofs/Proofs/Ai/Graph/Pseudorandom/Advanced`
  - `rg -n '"module": "Proofs.Ai.Graph.(Expander|Pseudorandom).Advanced".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T61 Add Graph Minor Structure And Treewidth Theorems

- Status: Completed
- Depends on: `CG-T23`, `CG-T50`, `CG-T55`, verified finite graph minor and
  tree/path foundations
- Areas: `Proofs.Ai.Graph.Minor.Structure`,
  `Proofs.Ai.Graph.Treewidth`, `Proofs.Ai.Graph.Separator`
- Tasks:
  - Add tree-decomposition, branch-decomposition, bramble, separator, and grid
    minor certificate predicates. Done in `Proofs.Ai.Graph.Minor.Structure`,
    `Proofs.Ai.Graph.Treewidth`, and `Proofs.Ai.Graph.Separator`.
  - State finite excluded-minor, treewidth-grid-minor, planar separator, and
    bounded-treewidth dynamic-programming correctness routes. Done with
    explicit certificate packages and theorem-route projections.
  - Treat Robertson-Seymour-scale structure theorems as explicit imported
    theorem-package prerequisites before deriving finite corollaries. Done via
    imported structure route evidence and no-uncertified-obstruction evidence.
- Deliverables:
  - `L2` minor/treewidth/separator modules with concrete decomposition
    witnesses.
- Acceptance criteria:
  - No finite obstruction theorem is asserted without either a checked
    certificate or an explicit imported structure-theorem route.
  - Algorithmic corollaries use trace certificates from `CG-T55` where
    relevant.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Minor.Structure Proofs.Ai.Graph.Treewidth Proofs.Ai.Graph.Separator`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Minor.Structure --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Treewidth --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Separator --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Graph/Minor/Structure/meta.json proofs/Proofs/Ai/Graph/Treewidth/meta.json proofs/Proofs/Ai/Graph/Separator/meta.json`
  - `rg -n "NoUncertifiedFiniteObstructionEvidence|ImportedStructureTheoremPackageEvidence|TraceSourceFreeVerificationEvidence|ExecutableAlgorithmExcludedEvidence" proofs/Proofs/Ai/Graph/Minor/Structure/source.npa proofs/Proofs/Ai/Graph/Treewidth/source.npa proofs/Proofs/Ai/Graph/Separator/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2" proofs/Proofs/Ai/Graph/Minor/Structure proofs/Proofs/Ai/Graph/Treewidth proofs/Proofs/Ai/Graph/Separator`
  - `rg -n '"module": "Proofs.Ai.Graph.(Minor.Structure|Treewidth|Separator)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T62 Add Additive Combinatorics And Arithmetic Progression Theorems

- Status: Completed
- Depends on: `CG-T09`, `CG-T30`, `CG-T53`, number-theory additive modules, and
  finite Fourier/additive group prerequisites
- Areas: `Proofs.Ai.Combinatorics.Additive`,
  `Proofs.Ai.Combinatorics.ArithmeticProgression`,
  `Proofs.Ai.NumberTheory.Additive`
- Tasks:
  - Done: added sumset, density increment, arithmetic progression, Bohr set,
    and additive energy certificate predicates in
    `Proofs.Ai.Combinatorics.Additive` and
    `Proofs.Ai.Combinatorics.ArithmeticProgression`.
  - Done: stated Roth theorem, finite Szemeredi theorem routes, Freiman-type
    finite model lemmas, Balog-Szemeredi-Gowers route statements, and
    sum-product finite-field corollaries through L2 route packages.
  - Done: routed Green-Tao style prime progression dependencies through
    `Proofs.Ai.NumberTheory.AdditivePrime` instead of placing prime
    distribution assumptions in combinatorics modules.
- Deliverables:
  - `L2` additive combinatorics modules with explicit group/Fourier and
    density-increment prerequisites.
- Acceptance criteria:
  - Prime-number inputs are explicit imported routes, not hidden combinatorial
    axioms.
  - Infinite/asymptotic progression statements expose finite forms and limit
    routes separately.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Additive Proofs.Ai.Combinatorics.ArithmeticProgression Proofs.Ai.NumberTheory.Additive`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Additive --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.ArithmeticProgression --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Additive --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Combinatorial`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Combinatorial --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Combinatorics/Additive/meta.json proofs/Proofs/Ai/Combinatorics/ArithmeticProgression/meta.json proofs/Proofs/Ai/NumberTheory/Additive/meta.json proofs/Proofs/Ai/NumberTheory/Combinatorial/meta.json`
  - `rg -n "NoHiddenPrimeDistributionAssumptionEvidence|NoInfiniteProgressionWithoutFiniteRouteEvidence|AdditivePrimeImportEvidence|FiniteToAsymptotic|Roth|Szemeredi|Freiman|Balog|SumProduct|DensityIncrement|Fourier|Bohr" proofs/Proofs/Ai/Combinatorics/Additive/source.npa proofs/Proofs/Ai/Combinatorics/ArithmeticProgression/source.npa proofs/Proofs/Ai/NumberTheory/Additive/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Combinatorics/Additive proofs/Proofs/Ai/Combinatorics/ArithmeticProgression proofs/Proofs/Ai/NumberTheory/Additive`
  - `rg -n '"module": "Proofs.Ai.(Combinatorics.(Additive|ArithmeticProgression)|NumberTheory.Additive)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T63 Add Polynomial Method And Finite Field Combinatorics Theorems

- Status: Completed
- Depends on: `CG-T31`, `CG-T37`, `CG-T53`, algebra polynomial and finite-field
  prerequisites
- Areas: `Proofs.Ai.Combinatorics.PolynomialMethod`,
  `Proofs.Ai.Combinatorics.FiniteField`, `Proofs.Ai.Combinatorics.Kakeya`
- Tasks:
  - Done: added combinatorial Nullstellensatz, Schwartz-Zippel,
    Chevalley-Warning, and finite-field polynomial vanishing certificate
    predicates in `Proofs.Ai.Combinatorics.PolynomialMethod`.
  - Done: added finite-field Kakeya, cap-set style polynomial method routes,
    and incidence bounds with explicit algebraic prerequisites in
    `Proofs.Ai.Combinatorics.FiniteField` and
    `Proofs.Ai.Combinatorics.Kakeya`.
  - Done: reused algebraic polynomial, finite-field, finite-geometry, and
    additive-combinatorics route imports instead of adding
    graph/combinatorics-local polynomial axioms.
- Deliverables:
  - `L2` polynomial method modules for finite combinatorial applications.
- Acceptance criteria:
  - Polynomial identity and finite-field facts import algebra package routes.
  - Kakeya/cap-set statements separate polynomial method lemma certificates
    from additive-combinatorics corollaries.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.PolynomialMethod Proofs.Ai.Combinatorics.FiniteField Proofs.Ai.Combinatorics.Kakeya`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.PolynomialMethod --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.FiniteField --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Kakeya --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Combinatorics/PolynomialMethod/meta.json proofs/Proofs/Ai/Combinatorics/FiniteField/meta.json proofs/Proofs/Ai/Combinatorics/Kakeya/meta.json`
  - `rg -n "CombinatorialNullstellensatz|SchwartzZippel|ChevalleyWarning|PolynomialVanishing|FiniteFieldCombinatorics|SumProduct|IncidenceBound|Kakeya|CapSet|SeparatePolynomialLemmaFromAdditiveCorollary|NoCombinatoricsLocalPolynomialAxiom" proofs/Proofs/Ai/Combinatorics/PolynomialMethod/source.npa proofs/Proofs/Ai/Combinatorics/FiniteField/source.npa proofs/Proofs/Ai/Combinatorics/Kakeya/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Combinatorics/PolynomialMethod proofs/Proofs/Ai/Combinatorics/FiniteField proofs/Proofs/Ai/Combinatorics/Kakeya`
  - `rg -n '"module": "Proofs.Ai.Combinatorics.(PolynomialMethod|FiniteField|Kakeya)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T64 Add Association Scheme, Coding Bound, And Design Theorems

- Status: Completed
- Depends on: `CG-T33`, `CG-T36`, `CG-T37`, `CG-T54`, verified linear algebra,
  coding theory, and finite geometry prerequisites
- Areas: `Proofs.Ai.Combinatorics.AssociationScheme.Coding`,
  `Proofs.Ai.Combinatorics.Design.Coding`,
  `Proofs.Ai.Combinatorics.FiniteGeometry.Coding`
- Tasks:
  - Done: Added Delsarte linear-programming bound, MacWilliams identity route,
    Hamming/Johnson association scheme, and orthogonal array certificate route
    statements in primary coding modules.
  - Done: Added finite geometry code construction theorems and design-to-code
    bridge statements with explicit weight enumerator prerequisites.
  - Done: Routed coding theory facts through primary association-scheme,
    design, and finite-geometry coding modules instead of finite-geometry-only
    aliases.
- Deliverables:
  - `L2` association-scheme/coding/design theorem modules.
- Acceptance criteria:
  - Every coding-bound theorem exposes the ambient metric scheme and weight
    enumerator evidence.
  - Design/code bridges reuse `Proofs.Ai.Combinatorics.Design` and
    `Proofs.Ai.Combinatorics.FiniteGeometry`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.AssociationScheme.Coding Proofs.Ai.Combinatorics.Design.Coding Proofs.Ai.Combinatorics.FiniteGeometry.Coding`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.AssociationScheme.Coding --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Design.Coding --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.FiniteGeometry.Coding --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Combinatorics/AssociationScheme/Coding/meta.json proofs/Proofs/Ai/Combinatorics/Design/Coding/meta.json proofs/Proofs/Ai/Combinatorics/FiniteGeometry/Coding/meta.json`
  - `rg -n "Delsarte|MacWilliams|WeightEnumerator|OrthogonalArray|Hamming|Johnson|AmbientMetricScheme|DesignToCode|FiniteGeometry|PrimaryCoding" proofs/Proofs/Ai/Combinatorics/AssociationScheme/Coding/source.npa proofs/Proofs/Ai/Combinatorics/Design/Coding/source.npa proofs/Proofs/Ai/Combinatorics/FiniteGeometry/Coding/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Combinatorics/AssociationScheme/Coding proofs/Proofs/Ai/Combinatorics/Design/Coding proofs/Proofs/Ai/Combinatorics/FiniteGeometry/Coding`
  - `rg -n '"module": "Proofs.Ai.Combinatorics.(AssociationScheme.Coding|Design.Coding|FiniteGeometry.Coding)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T65 Add Matroid Minors, Representability, And Structure Theorems

- Status: Completed
- Depends on: `CG-T34`, `CG-T35`, `CG-T45`, `CG-T52`, verified linear algebra
  and graph minor prerequisites
- Areas: `Proofs.Ai.Combinatorics.Matroid.Minor`,
  `Proofs.Ai.Combinatorics.Matroid.Representability`,
  `Proofs.Ai.Combinatorics.Matroid.Structure`
- Tasks:
  - Done: added matroid deletion/contraction, dual-minor compatibility,
    graphic/cographic bridge, and minor certificate route predicates in
    `Proofs.Ai.Combinatorics.Matroid.Minor`.
  - Done: added representability, regular matroid, matrix representation, and
    excluded-minor route predicates with explicit imported algebraic and
    minor-theorem prerequisites in
    `Proofs.Ai.Combinatorics.Matroid.Representability`.
  - Done: added Seymour decomposition style regular matroid routes and
    graphic/cographic structure bridges importing graph minor, tree, and cycle
    modules in `Proofs.Ai.Combinatorics.Matroid.Structure`.
- Deliverables:
  - `L2` matroid minor/representability/structure theorem modules.
- Acceptance criteria:
  - Excluded-minor classifications are explicit theorem-package routes or
    checked finite certificate families.
  - Graphic/cographic matroid results import graph minor and tree/cycle
    modules rather than duplicating graph facts.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Matroid.Minor Proofs.Ai.Combinatorics.Matroid.Representability Proofs.Ai.Combinatorics.Matroid.Structure`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Minor --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Representability --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Structure --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Combinatorics/Matroid/Minor/meta.json proofs/Proofs/Ai/Combinatorics/Matroid/Representability/meta.json proofs/Proofs/Ai/Combinatorics/Matroid/Structure/meta.json`
  - `rg -n "Deletion|Contraction|DualMinor|Representability|RegularMatroid|ExcludedMinor|Seymour|Graphic|Cographic|GraphMinor|OneTwoThree" proofs/Proofs/Ai/Combinatorics/Matroid/Minor/source.npa proofs/Proofs/Ai/Combinatorics/Matroid/Representability/source.npa proofs/Proofs/Ai/Combinatorics/Matroid/Structure/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Combinatorics/Matroid/Minor proofs/Proofs/Ai/Combinatorics/Matroid/Representability proofs/Proofs/Ai/Combinatorics/Matroid/Structure`
  - `rg -n '"module": "Proofs.Ai.Combinatorics.Matroid.(Minor|Representability|Structure)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T66 Add Algorithmic Graph Minor And Parameterized Meta Theorems

- Status: Completed
- Depends on: `CG-T55`, `CG-T61`, `CG-T65`, verified finite graph minor,
  treewidth, separator, and trace-certificate prerequisites
- Areas: `Proofs.Ai.Graph.Minor.Algorithmic`,
  `Proofs.Ai.Graph.Treewidth.DynamicProgramming`,
  `Proofs.Ai.Graph.Parameterized`
- Tasks:
  - Done: added finite MSO model-checking, Courcelle-style dynamic programming,
    nice tree decomposition, and trace replay certificate predicates.
  - Done: stated bidimensionality, irrelevant-vertex, protrusion replacement,
    and kernelization theorem routes with explicit imported graph-minor
    structure prerequisites.
  - Done: connected bounded-treewidth algorithm certificates to graph minor and
    separator modules without duplicating decomposition facts.
- Deliverables:
  - `L2` algorithmic graph-minor and parameterized theorem modules.
- Acceptance criteria:
  - Every meta-theorem exposes the finite decomposition witness and trace
    checker boundary.
  - Robertson-Seymour-scale inputs are imported theorem-package routes, not
    local axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Minor.Algorithmic Proofs.Ai.Graph.Treewidth.DynamicProgramming Proofs.Ai.Graph.Parameterized`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Minor.Algorithmic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Treewidth.DynamicProgramming --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Parameterized --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Graph/Minor/Algorithmic/meta.json proofs/Proofs/Ai/Graph/Treewidth/DynamicProgramming/meta.json proofs/Proofs/Ai/Graph/Parameterized/meta.json`
  - `rg -n "MSO|Mso|Courcelle|NiceTree|TraceReplay|Bidimensionality|IrrelevantVertex|Protrusion|Kernelization|ImportedGraphMinorStructure|NoLocalRobertsonSeymourAxiom" proofs/Proofs/Ai/Graph/Minor/Algorithmic/source.npa proofs/Proofs/Ai/Graph/Treewidth/DynamicProgramming/source.npa proofs/Proofs/Ai/Graph/Parameterized/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Graph/Minor/Algorithmic proofs/Proofs/Ai/Graph/Treewidth/DynamicProgramming proofs/Proofs/Ai/Graph/Parameterized`
  - `rg -n '"module": "Proofs.Ai.Graph.(Minor.Algorithmic|Treewidth.DynamicProgramming|Parameterized)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T67 Add Sparse Hypergraph Containers, Removal Transfer, And Stability Routes

- Status: Completed
- Depends on: `CG-T57`, `CG-T58`, `CG-T63`, verified extremal,
  probabilistic, and polynomial-method prerequisites
- Areas: `Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications.Advanced`,
  `Proofs.Ai.Combinatorics.Hypergraph.Removal.Stability`,
  `Proofs.Ai.Graph.Extremal.Stability.Transfer`
- Tasks:
  - Done: extended the completed `CG-T58` hypergraph container/removal modules
    with sparse-transfer, co-degree hierarchy, and robust independent-set
    counting certificate predicates.
  - Done: stated sparse graph/hypergraph transfer and stability routes that
    import the completed `CG-T57` supersaturation/stability and `CG-T58`
    regularity/removal packages instead of restating them.
  - Done: connected arithmetic progression and cap-set applications to finite
    container/removal transfer packages rather than adding local asymptotic
    axioms.
- Deliverables:
  - `L2` sparse hypergraph container/removal transfer and stability route
    modules.
- Acceptance criteria:
  - Infinite or asymptotic statements expose finite epsilon-parameter forms and
    limiting routes separately.
  - Sparse random transfer uses checked probability-space certificates from the
    probabilistic combinatorics modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Combinatorics.Hypergraph.Removal.Stability Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications.Advanced Proofs.Ai.Graph.Extremal.Stability.Transfer`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph.Removal.Stability --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph.ContainerApplications.Advanced --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Extremal.Stability.Transfer --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Combinatorics/Hypergraph/Removal/Stability/meta.json proofs/Proofs/Ai/Combinatorics/Hypergraph/ContainerApplications/Advanced/meta.json proofs/Proofs/Ai/Graph/Extremal/Stability/Transfer/meta.json`
  - `rg -n "Sparse|Codegree|Robust|IndependentSet|ArithmeticProgression|CapSet|FiniteEpsilon|LimitRoute|ProbabilitySpace|NoAsymptotic|NoInfinite|NoLocalAsymptotic" proofs/Proofs/Ai/Combinatorics/Hypergraph/Removal/Stability/source.npa proofs/Proofs/Ai/Combinatorics/Hypergraph/ContainerApplications/Advanced/source.npa proofs/Proofs/Ai/Graph/Extremal/Stability/Transfer/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Combinatorics/Hypergraph/Removal/Stability proofs/Proofs/Ai/Combinatorics/Hypergraph/ContainerApplications/Advanced proofs/Proofs/Ai/Graph/Extremal/Stability/Transfer`
  - `rg -n '"module": "Proofs.Ai.(Combinatorics.Hypergraph.(Removal.Stability|ContainerApplications.Advanced)|Graph.Extremal.Stability.Transfer)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T68 Add Graph Limits, Regularity, And Property Testing Theorems

- Status: Completed
- Depends on: `CG-T56`, `CG-T59`, `CG-T61`, `CG-T67`, verified graph,
  regularity, random graph, and finite probability prerequisites
- Areas: `Proofs.Ai.Graph.Regularity.Strong`,
  `Proofs.Ai.Graph.Limit.Graphon`, `Proofs.Ai.Graph.PropertyTesting.Hereditary`
- Tasks:
  - Done: added strong regularity, counting lemma, cut-norm approximation,
    graphon sampling, and finite partition certificate predicates.
  - Done: reused the completed `CG-T56` graph removal and property-testing
    modules while adding compactness-transfer, graphon-limit, and hereditary
    testing routes with explicit finite-to-limit prerequisites.
  - Done: routed graph-limit corollaries through finite regularity certificates
    and source-free finite sampling-certificate evidence.
- Deliverables:
  - `L2` graph regularity/limit/property-testing theorem modules.
- Acceptance criteria:
  - Graphon or compactness arguments are recorded as imported route packages
    with finite approximants.
  - Property testers expose sample complexity certificates and do not rely on
    informal randomized arguments.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.Graph.Regularity.Strong Proofs.Ai.Graph.Limit.Graphon Proofs.Ai.Graph.PropertyTesting.Hereditary`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Regularity.Strong --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Limit.Graphon --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.PropertyTesting.Hereditary --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `cargo fmt --all -- --check`
  - `rg -n '"axioms": \[\]' proofs/Proofs/Ai/Graph/Regularity/Strong/meta.json proofs/Proofs/Ai/Graph/Limit/Graphon/meta.json proofs/Proofs/Ai/Graph/PropertyTesting/Hereditary/meta.json`
  - `rg -n "Strong|CountingLemma|CutNorm|Graphon|Compactness|FiniteApproximant|SourceFree|Sampling|SampleComplexity|Hereditary|FiniteToLimit|NoLocal|NoInformal" proofs/Proofs/Ai/Graph/Regularity/Strong/source.npa proofs/Proofs/Ai/Graph/Limit/Graphon/source.npa proofs/Proofs/Ai/Graph/PropertyTesting/Hereditary/source.npa`
  - `rg -n "interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface" proofs/Proofs/Ai/Graph/Regularity/Strong proofs/Proofs/Ai/Graph/Limit/Graphon proofs/Proofs/Ai/Graph/PropertyTesting/Hereditary`
  - `rg -n '"module": "Proofs.Ai.Graph.(Regularity.Strong|Limit.Graphon|PropertyTesting.Hereditary)".*(interface_statement|boundary_statement|InterfaceEvidence|BoundaryEvidence|NoL2|no_l2|_interface)' proofs/generated/ai-theorem-index.json`
  - `git diff --check`

### CG-T69 Add Spectral Graph, Expander, And Pseudorandomness Theorems

- Status: Pending
- Depends on: `CG-T51`, `CG-T60`, `CG-T63`, `CG-T64`, verified linear algebra,
  graph, coding, and finite-field prerequisites
- Areas: `Proofs.Ai.Graph.Spectral.Sparsifier`,
  `Proofs.Ai.Graph.Expander.Construction`,
  `Proofs.Ai.Graph.Pseudorandom.Construction`
- Tasks:
  - Add normalized Laplacian construction, spectral sparsifier, zig-zag product,
    replacement product, and explicit expander-construction certificate
    predicates.
  - Reuse the completed `CG-T60` expander mixing, Cheeger refinement,
    Alon-Boppana, and quasirandom route packages while adding construction and
    sparsifier corollaries.
  - Connect spectral combinatorics to finite-field and coding modules for
    explicit expander constructions.
- Deliverables:
  - `L2` spectral graph, expander, and pseudorandomness theorem modules.
- Acceptance criteria:
  - All eigenvalue and operator-norm inputs import verified linear algebra
    package routes.
  - Construction theorems expose finite certificates for adjacency operators
    and degree regularity.
- Verification:
  - `git diff --check`

### CG-T70 Add Canonical Ramsey, Structural Ramsey, And Hales-Jewett Routes

- Status: Pending
- Depends on: `CG-T26`, `CG-T27`, `CG-T62`, verified finite Ramsey,
  hypergraph Ramsey, additive-combinatorics, and finite-set prerequisites
- Areas: `Proofs.Ai.Combinatorics.Ramsey.Canonical`,
  `Proofs.Ai.Combinatorics.Ramsey.Structural`,
  `Proofs.Ai.Combinatorics.Ramsey.HalesJewett`
- Tasks:
  - Add canonical Ramsey, partite construction, Graham-Rothschild parameter
    set, and Hales-Jewett combinatorial-line certificate predicates.
  - State structural Ramsey theorem routes for finite relational classes with
    explicit amalgamation/order-expansion prerequisites.
  - Connect van der Waerden, density Hales-Jewett, and arithmetic progression
    corollaries through finite Ramsey/additive packages.
- Deliverables:
  - `L2` canonical/structural Ramsey and Hales-Jewett route modules.
- Acceptance criteria:
  - Open exact Ramsey values are not introduced as theorem declarations unless
    backed by checked finite certificate families.
  - Infinite Ramsey statements expose finite forms and compactness/limit routes
    separately.
- Verification:
  - `git diff --check`

### CG-T71 Add Topological Graph Theory, Surface Minor, And Drawing Theorems

- Status: Pending
- Depends on: `CG-T22`, `CG-T23`, `CG-T50`, `CG-T61`, verified graph minor,
  planar graph, topology-route, and separator prerequisites
- Areas: `Proofs.Ai.Graph.Topological.Embedding`,
  `Proofs.Ai.Graph.Minor.Surface`,
  `Proofs.Ai.Graph.Drawing`
- Tasks:
  - Add rotation system, cellular embedding, Euler genus, crossing number,
    linkless embedding, and surface-separator certificate predicates.
  - State Kuratowski/Wagner route refinements, graph minor surface structure,
    crossing lemma, planar separator, and bounded-genus separator routes with
    explicit topological package prerequisites.
  - Connect surface-minor corollaries to graph minor structure modules and
    finite embedding certificates.
- Deliverables:
  - `L2` topological graph, surface minor, and graph drawing theorem modules.
- Acceptance criteria:
  - Topological classification inputs are imported theorem-package routes, not
    encoded as hidden graph axioms.
  - Planarity, genus, and crossing-number assertions include finite
    certificate families or explicit imported route packages.
- Verification:
  - `git diff --check`

### CG-T72 Add Matroid Connectivity, Branch Width, And Splitter Theorems

- Status: Pending
- Depends on: `CG-T35`, `CG-T52`, `CG-T61`, `CG-T65`, verified matroid minor,
  matroid optimization, and graph minor prerequisites
- Areas: `Proofs.Ai.Combinatorics.Matroid.Connectivity`,
  `Proofs.Ai.Combinatorics.Matroid.BranchWidth`,
  `Proofs.Ai.Combinatorics.Matroid.Decomposition`
- Tasks:
  - Add matroid connectivity, separation, branch decomposition, tangle, and
    splitter sequence certificate predicates.
  - State branch-width duality, matroid splitter theorem routes,
    representable-matroid decomposition, and excluded-minor finite obstruction
    routes with explicit imported structure prerequisites.
  - Connect graphic matroid branch-width and graph branch-width/treewidth
    bridge statements to graph minor modules.
- Deliverables:
  - `L2` matroid connectivity/branch-width/decomposition theorem modules.
- Acceptance criteria:
  - Splitter and excluded-minor statements are theorem-package routes or
    checked finite certificate families.
  - Graphic matroid corollaries import graph minor and treewidth modules rather
    than reproving graph structure facts.
- Verification:
  - `git diff --check`

### CG-T73 Add Algebraic, Topological, And Poset Combinatorics Theorems

- Status: Pending
- Depends on: `CG-T32`, `CG-T33`, `CG-T36`, `CG-T53`, `CG-T63`, verified
  algebraic-combinatorics, design, polynomial, and finite geometry
  prerequisites
- Areas: `Proofs.Ai.Combinatorics.Poset.Topology`,
  `Proofs.Ai.Combinatorics.SimplicialComplex`,
  `Proofs.Ai.Combinatorics.Algebraic`
- Tasks:
  - Add finite poset foundation, shellability, Cohen-Macaulay complex, order
    complex, h-vector, face-enumeration, and matroid-complex certificate
    predicates.
  - State Mobius inversion refinements, Sperner/LYM route packages, hard
    Lefschetz-style enumerative routes as imported algebraic prerequisites,
    and finite geometric lattice corollaries.
  - Connect design and matroid applications through primary poset/simplicial
    modules with explicit algebraic topology package boundaries.
- Deliverables:
  - `L2` algebraic/topological/poset combinatorics theorem modules.
- Acceptance criteria:
  - Cohomological or Lefschetz inputs are imported theorem-package routes with
    finite combinatorial projections.
  - Enumerative identities expose finite chain, rank, and face-vector
    certificates.
- Verification:
  - `git diff --check`

### CG-T74 Add Random Graph Threshold, Concentration, And Entropy Theorems

- Status: Pending
- Depends on: `CG-T48`, `CG-T59`, `CG-T67`, verified finite probability,
  random graph threshold, and hypergraph prerequisites
- Areas: `Proofs.Ai.Combinatorics.Probability.Concentration`,
  `Proofs.Ai.Graph.Random.Threshold.Sharp`,
  `Proofs.Ai.Combinatorics.Entropy`
- Tasks:
  - Add martingale exposure, bounded differences, Janson inequality, entropy
    submodularity, threshold, and random graph process certificate predicates.
  - Reuse the completed `CG-T59` threshold and phase-transition modules while
    adding concentration/entropy proofs for sharp-threshold refinements, random
    regular graph expansion, and entropy compression packages.
  - Connect randomized existence proofs to explicit finite probability-space
    and derandomization certificate modules where possible.
- Deliverables:
  - `L2` concentration/random-threshold/entropy method theorem modules.
- Acceptance criteria:
  - Asymptotic random graph statements expose finite-n bounds and limit
    transfer routes separately.
  - Randomized proof outputs record seed/probability-space certificates or
    imported theorem-package routes.
- Verification:
  - `git diff --check`

### CG-T75 Add Polyhedral Combinatorics, Matching, And Matroid Optimization Theorems

- Status: Pending
- Depends on: `CG-T44`, `CG-T45`, `CG-T47`, `CG-T52`, `CG-T65`, verified
  linear programming, matching, flow, and matroid prerequisites
- Areas: `Proofs.Ai.Combinatorics.Optimization.Polytope`,
  `Proofs.Ai.Graph.Matching.Polyhedral`,
  `Proofs.Ai.Combinatorics.Optimization.Matroid.Polyhedral`
- Tasks:
  - Add totally unimodular matrix, integral polytope, cut/flow polyhedron,
    matching polytope, matroid-intersection polyhedron, and submodular
    certificate predicates.
  - State Edmonds matching polytope, max-flow/min-cut polyhedral route,
    polymatroid intersection, and submodular minimization correctness routes
    that import the completed `CG-T45` matroid-intersection and `CG-T47`
    matching/flow theorem packages.
  - Connect graph matching/flow certificates to matroid optimization and
    linear algebra modules without duplicating LP duality facts.
- Deliverables:
  - `L2` polyhedral combinatorics, matching, and matroid optimization theorem
    modules.
- Acceptance criteria:
  - LP duality, total unimodularity, and separation-optimization equivalence
    are imported theorem-package routes or checked finite certificates.
  - Optimization algorithms expose primal/dual witness certificates and
    source-free trace replay artifacts.
- Verification:
  - `git diff --check`

## Completion Definition

A roadmap milestone is complete only when:

- the target statement lives in the intended primary namespace;
- all non-kernel principles are explicit assumptions or imported packages;
- source, replay, metadata, and certificate artifacts are updated when source
  work occurs;
- the target module verifies source-free;
- changed proof-corpus artifacts pass the authoring gate;
- duplicate-home decisions are recorded so downstream theorem families import
  rather than reprove the same result.

Documentation-only planning updates should at least pass:

```sh
git diff --check
```
