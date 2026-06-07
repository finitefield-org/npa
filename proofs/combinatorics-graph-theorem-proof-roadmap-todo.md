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
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `Proofs.Ai.Graph.X` for graph-owned modules. Use `--build-module` before
source-free `--module` checks when source changes must be reflected in
certificates. Reserve `check-corpus-package.sh` or `check-corpus-full.sh` for
package-wide verifier behavior, publish-plan or package metadata updates,
certificate/checker compatibility, release work, or promotion into a
high-trust closure.

## Current Implementation Facts

- The proof corpus now has a checked `Proofs.Ai.Combinatorics.*` foundation
  tree through `Proofs.Ai.Combinatorics.SetSystem` and the first checked
  graph-owned foundation module, `Proofs.Ai.Graph.Basic`.
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

## Roadmap Coverage Map

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

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `CG-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `CG-T01`, `CG-T07`, `CG-T10`, `CG-T12`, `CG-T16`, `CG-T18`, `CG-T22`, `CG-T24`, `CG-T26`, `CG-T28`, `CG-T30`, `CG-T32`, `CG-T34`, `CG-T36`, `CG-T38`, `CG-T40`, `CG-T42`, `CG-T44` | `L1` interface or law-package foundation first, followed by `L2` lemmas once prerequisites exist |
| `CG-T02` through `CG-T06`, `CG-T08`, `CG-T09`, `CG-T13` through `CG-T15`, `CG-T17`, `CG-T20`, `CG-T21` | target `L2` derived certificates where prerequisites exist |
| `CG-T11`, `CG-T19`, `CG-T23`, `CG-T25`, `CG-T27`, `CG-T29`, `CG-T31`, `CG-T33`, `CG-T35`, `CG-T37`, `CG-T39`, `CG-T41`, `CG-T43`, `CG-T45` | split before source edits if prerequisites are absent; otherwise target `L2` for derived parts and keep advanced statements at `L1` |
| `CG-T46` | `L3` public closure and package verification |

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

- Status: Pending
- Depends on: `CG-T30`, `CG-T32`
- Areas: `Proofs.Ai.Combinatorics.Species`, `Proofs.Ai.Combinatorics.Polya`
- Tasks:
  - Add combinatorial species operation interfaces.
  - Add cycle index and Polya enumeration theorem interfaces.
  - Connect unlabeled counting to quotient and group-action evidence.
- Deliverables:
  - Species and Polya interface modules.
- Acceptance criteria:
  - Unlabeled counting exposes quotient/group-action assumptions.
  - Group-action prerequisites import algebra modules rather than duplicating
    group laws.
- Verification:
  - `rg -n "Polya|species|cycle index|unlabeled" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T32 Add Group Action And Orbit-Counting Bridge

- Status: Pending
- Depends on: `CG-T02`, algebra group prerequisites
- Areas: `Proofs.Ai.Combinatorics.Algebraic`, `Proofs.Ai.Combinatorics.Orbit`
- Tasks:
  - Add group action on finite sets and orbit/stabilizer statement surfaces.
  - Add Burnside lemma interface and selected projection theorems.
  - Connect to Polya and symmetric enumeration tasks.
- Deliverables:
  - Algebraic combinatorics bridge module.
- Acceptance criteria:
  - Group laws are imported from existing algebra modules.
  - Orbit quotient evidence is explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Algebraic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Algebraic`

### CG-T33 Add Symmetric Function And Association Scheme Interfaces

- Status: Pending
- Depends on: `CG-T32`, algebra/linear algebra prerequisites
- Areas: `Proofs.Ai.Combinatorics.SymmetricFunction`,
  `Proofs.Ai.Combinatorics.AssociationScheme`
- Tasks:
  - Add symmetric-function theorem surface.
  - Add association scheme and strongly regular graph interfaces.
  - Record representation-theory dependencies as explicit boundaries.
- Deliverables:
  - Advanced algebraic combinatorics interface module.
- Acceptance criteria:
  - Representation-theoretic results remain `L1` until representation
    foundations exist.
  - Strongly regular graph facts cross-link spectral graph dependencies.
- Verification:
  - `rg -n "symmetric function|association scheme|strongly regular|representation" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T34 Add Matroid Foundation

- Status: Pending
- Depends on: `CG-T02`
- Areas: `Proofs.Ai.Combinatorics.Matroid.Basic`
- Tasks:
  - Define matroid independent-set, basis, rank, circuit, closure, and flat law
    packages.
  - Add basis exchange and rank submodularity theorem targets.
  - Record dependencies for representable and graphic matroids.
- Deliverables:
  - Matroid foundation module.
- Acceptance criteria:
  - Matroid axioms are explicit law packages.
  - Basis existence is not hidden behind finite choice.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Matroid.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Matroid.Basic`

### CG-T35 Add Matroid Dual, Graphic Matroid, And Greedy Interfaces

- Status: Pending
- Depends on: `CG-T34`, `CG-T15`, linear algebra prerequisites for
  representable matroids
- Areas: `Proofs.Ai.Combinatorics.Matroid.Dual`,
  `Proofs.Ai.Combinatorics.Matroid.Graphic`,
  `Proofs.Ai.Combinatorics.Matroid.Greedy`
- Tasks:
  - Add dual matroid statement surface.
  - Add graphic and cographic matroid bridge interfaces.
  - Add greedy algorithm correctness theorem targets.
- Deliverables:
  - Matroid bridge and optimization interfaces.
- Acceptance criteria:
  - Graphic matroid imports graph tree/cycle foundations.
  - Greedy correctness uses trace/certificate evidence.
- Verification:
  - `rg -n "Matroid|graphic matroid|greedy" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T36 Add Design And Incidence Structure Foundation

- Status: Pending
- Depends on: `CG-T09`
- Areas: `Proofs.Ai.Combinatorics.Design`
- Tasks:
  - Define incidence structure, block design, balanced incomplete block design,
    and parameter law packages.
  - Add basic design parameter counting identities.
  - Record finite geometry bridge points.
- Deliverables:
  - Design theory foundation module.
- Acceptance criteria:
  - Design parameter equations are derived counting statements where possible.
  - Incidence structures reuse finite set-system foundations.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Design`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Design`

### CG-T37 Add Finite Geometry Interfaces

- Status: Pending
- Depends on: `CG-T36`, field-theory and number-theory finite-field
  prerequisites
- Areas: `Proofs.Ai.Combinatorics.FiniteGeometry`
- Tasks:
  - Add affine plane, projective plane, and finite geometry theorem surfaces.
  - Add finite-field construction interfaces.
  - Record coding-theory aliases without creating a coding-theory roadmap here.
- Deliverables:
  - Finite geometry interface module.
- Acceptance criteria:
  - Finite-field assumptions are imported from field-theory/number-theory
    modules.
  - Projective plane existence claims carry explicit construction evidence.
- Verification:
  - `rg -n "finite geometry|projective plane|finite field|coding" proofs/combinatorics-graph-theorem-proof-roadmap*.md develop/proof-corpus-field-theory-roadmap*.md`
  - `git diff --check`

### CG-T38 Add Hypergraph Foundation

- Status: Pending
- Depends on: `CG-T09`
- Areas: `Proofs.Ai.Combinatorics.Hypergraph`
- Tasks:
  - Define hypergraph, uniform hypergraph, hyperedge, degree, matching,
    covering, transversal, and shadow predicates.
  - Add hypergraph incidence and degree-counting theorem targets.
  - Align set-system and hypergraph names.
- Deliverables:
  - Hypergraph foundation module.
- Acceptance criteria:
  - Hypergraph APIs share finite-family foundations.
  - Matching and covering names do not conflict with graph-specific modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Hypergraph`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Hypergraph`

### CG-T39 Add Hypergraph Extremal And Container Interfaces

- Status: Pending
- Depends on: `CG-T38`, `CG-T28`
- Areas: `Proofs.Ai.Combinatorics.Container`
- Tasks:
  - Add Erdos-Ko-Rado, matching/covering extremal, and hypergraph Ramsey
    interfaces.
  - Add container method statement surface.
  - Record entropy and probabilistic dependencies.
- Deliverables:
  - Hypergraph extremal interface module.
- Acceptance criteria:
  - Container assumptions are named and localized.
  - Probabilistic and entropy dependencies are not private law fields.
- Verification:
  - `rg -n "Erdos-Ko-Rado|container|hypergraph extremal|entropy" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T40 Add Adjacency, Incidence, And Laplacian Matrix Interfaces

- Status: Pending
- Depends on: `CG-T12`, linear algebra matrix prerequisites
- Areas: `Proofs.Ai.Graph.Spectral`, `Proofs.Ai.Graph.Laplacian`
- Tasks:
  - Define adjacency matrix, incidence matrix, degree matrix, and Laplacian
    statement surfaces.
  - Add symmetry and positive-semidefinite interfaces where prerequisites
    exist.
  - Link graph construction evidence to matrix construction evidence.
- Deliverables:
  - Spectral graph foundation interface module.
- Acceptance criteria:
  - Matrix facts import linear algebra modules.
  - Graph-specific statements do not duplicate spectral theorem ownership.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Spectral`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Spectral`

### CG-T41 Add Spectral Bounds And Expander Interfaces

- Status: Pending
- Depends on: `CG-T40`, linear algebra spectral prerequisites
- Areas: `Proofs.Ai.Graph.Spectral.Bounds`, `Proofs.Ai.Graph.Expander`
- Tasks:
  - Add Laplacian kernel and connected component interfaces.
  - Add eigenvalue bounds for degree, coloring, and expansion.
  - Add Cheeger-style and expander theorem interfaces.
- Deliverables:
  - Spectral graph theorem interface module.
- Acceptance criteria:
  - Cheeger-style statements import analytic or linear-algebra dependencies
    explicitly.
  - Expander definitions do not depend on unverified random graph assumptions.
- Verification:
  - `rg -n "Laplacian|Cheeger|expander|spectral graph" proofs/combinatorics-graph-theorem-proof-roadmap*.md proofs/linear-algebra-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T42 Add Graph Search And Shortest-Path Correctness Interfaces

- Status: Pending
- Depends on: `CG-T14`
- Areas: `Proofs.Ai.Graph.Algorithm.Search`,
  `Proofs.Ai.Graph.Algorithm.ShortestPath`
- Tasks:
  - Add BFS and DFS trace correctness statement surfaces.
  - Add shortest path, relaxation trace, and Dijkstra/Bellman-Ford interfaces
    with explicit weight assumptions.
  - Separate mathematical correctness from runtime complexity.
- Deliverables:
  - Graph search and shortest-path correctness interfaces.
- Acceptance criteria:
  - Executable algorithms are not checker inputs.
  - Trace objects contain the evidence needed for source-free verification.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Graph.Algorithm.Search`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Graph.Algorithm.Search`

### CG-T43 Add Spanning Tree, Matching, And Flow Algorithm Correctness

- Status: Pending
- Depends on: `CG-T15`, `CG-T17`, `CG-T19`
- Areas: `Proofs.Ai.Graph.Algorithm.SpanningTree`,
  `Proofs.Ai.Graph.Algorithm.Flow`
- Tasks:
  - Add Kruskal/Prim trace correctness interfaces.
  - Add augmenting path matching trace correctness interface.
  - Add Ford-Fulkerson style trace correctness interface.
- Deliverables:
  - Graph optimization algorithm correctness interfaces.
- Acceptance criteria:
  - Weight and capacity assumptions are explicit.
  - Trace termination and optimality certificates are ordinary proof evidence,
    not trusted execution.
- Verification:
  - `rg -n "Kruskal|Prim|Ford-Fulkerson|augmenting path" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T44 Add Submodularity And Polytope Interfaces

- Status: Pending
- Depends on: `CG-T17`, `CG-T19`, linear algebra/optimization prerequisites
- Areas: `Proofs.Ai.Combinatorics.Optimization`,
  `Proofs.Ai.Graph.Polytope`
- Tasks:
  - Add submodular function, rank function, cut function, and polymatroid
    statement surfaces.
  - Add matching polytope, flow polytope, and cut polytope interfaces.
  - Record LP/duality prerequisites.
- Deliverables:
  - Combinatorial optimization interface module.
- Acceptance criteria:
  - General LP and convex duality are imported from their primary route.
  - Graph-specific polytope statements keep finite graph evidence explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.Optimization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.Optimization`

### CG-T45 Add Matroid And Submodular Optimization Interfaces

- Status: Pending
- Depends on: `CG-T35`, `CG-T44`
- Areas: `Proofs.Ai.Combinatorics.Optimization.Matroid`
- Tasks:
  - Add matroid intersection, matroid union, and submodular optimization
    theorem surfaces.
  - Add greedy algorithm specialization theorem targets.
  - Link algorithm trace correctness to optimization statements.
- Deliverables:
  - Matroid and submodular optimization interfaces.
- Acceptance criteria:
  - Optimization theorem statements do not duplicate general convex-analysis
    ownership.
  - Greedy correctness reuses matroid foundations.
- Verification:
  - `rg -n "matroid intersection|submodular optimization|greedy" proofs/combinatorics-graph-theorem-proof-roadmap*.md`
  - `git diff --check`

### CG-T46 Prepare Public Closure Audit

- Status: Pending
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
