# Topology Theorem Proof Roadmap Todo

Source: `proofs/topology-theorem-proof-roadmap.md`

This document decomposes the topology theorem proof roadmap into concrete
authoring milestones. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, or certificate validity assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, and AI output
are untrusted.

---

## Scope

This task list covers theorem-card inventory, general topology foundations,
continuous maps, homeomorphisms, separation, compactness, metric topology,
connectedness, countability, Baire spaces, products, quotients, local
properties, nets, filters, homotopy, fundamental groups, covering spaces,
homology, cohomology, CW and simplicial complexes, manifolds, differential
topology, low-dimensional topology, fixed-point theorem families, dimension
theory, topological dynamics, characteristic classes, K-theory, spectral
sequences, stable homotopy interfaces, and public closure planning.

The list intentionally does not prove the roadmap in one pass. Later agents
should implement exactly one milestone or a clearly bounded contiguous batch.
When a milestone introduces only a statement interface because prerequisites
are absent, its acceptance criteria must prevent the interface from smuggling
the target theorem as an axiom.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding topological spaces, compactness, homotopy, homology, manifolds,
  bundles, spectra, or set-theoretic choice principles as trusted kernel
  primitives;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- treating theorem-search sidecars, AI indexes, replay files, generated docs,
  or this todo document as trusted evidence;
- promoting unstable topology modules into `npa-mathlib` before local closure,
  axiom-report, source-free, package, and public materialization checks are
  clean.

## Authoring Loop

For ordinary topology theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Reserve `check-corpus-package.sh` or
`check-corpus-full.sh` for package-wide verifier behavior, publish-plan or
package metadata updates, certificate/checker compatibility, release work, or
promotion into a high-trust closure.

## Current Implementation Facts

- The proof corpus now has checked concrete `Proofs.Ai.Topology.*` modules for
  basic topological vocabulary, closure, generated/subspace/initial/final
  topologies, continuity/map classes, homeomorphism/invariants, separation,
  compactness, metric compactness, connectedness core routes, countability /
  separability / Lindelof route vocabulary, product topology core routes, and
  quotient topology core routes.
- Analysis roadmap items `ANA-07`, `ANA-T22`, and `ANA-T23` already reserve
  early topology work for `Proofs.Ai.Topology.Basic`,
  `Proofs.Ai.Topology.Metric.Compact`, and
  `Proofs.Ai.Topology.FunctionSpace`. In this todo, those analysis items are
  compatibility aliases; the `TOP-*` and `TOP-T*` IDs define primary topology
  ownership once this roadmap is active.
- Existing reusable modules include `Proofs.Ai.Analysis.AbstractMetricTopology`,
  `Proofs.Ai.Analysis.AbstractNormedSpace`,
  `Proofs.Ai.Analysis.AbstractFixedPoint`,
  `Proofs.Ai.Analysis.AbstractLinearMap`,
  `Proofs.Ai.Analysis.AbstractDerivative`,
  `Proofs.Ai.Analysis.AbstractInverseFunction`,
  `Proofs.Ai.Analysis.AbstractImplicitFunction`,
  `Proofs.Ai.Geometry.AbstractMetric`, `Proofs.Ai.Geometry.Pythagorean`,
  `Proofs.Ai.Category.Classical`,
  `Proofs.Ai.Category.Infinity.SimplicialSet`, and checked algebra group
  modules under `Proofs.Ai.Algebra.AbstractGroup*`.
- Public `npa-mathlib` has already materialized metric-topology,
  normed-space, linear-map, derivative, fixed-point, inverse-function, and
  implicit-function closures through the current closure roadmap. New topology
  work should build on those closures instead of widening the trusted kernel.
- Open mapping, closed graph, and uniform boundedness stay primary in analysis
  `ANA-T27`; topology `TOP-T18` and `TOP-T19` only provide the completion and
  Baire inputs those functional-analysis theorems import.
- The measure-theory roadmap in
  `proofs/measure-theory-theorem-proof-roadmap.md` refines analysis `ANA-08`
  into `MEA-*` tasks. Measure recurrence should use the ergodic route
  `MEA-T51`; Stokes, de Rham, and characteristic-class routes that need
  Lebesgue-style integration should use `MEA-T19` through `MEA-T25` instead of
  treating `ANA-T24` through `ANA-T26` as the only dependency plan.
- Probability/process aliases still coordinate with statistics `STAT-T55`
  through `STAT-T57` where the statement is probabilistic rather than purely
  measure-theoretic.

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `TOP-00` inventory and statement policy | `TOP-T00` |
| `TOP-01` topological-space foundations | `TOP-T01` through `TOP-T02` |
| `TOP-02` generated, relative, initial, and final topologies | `TOP-T03` through `TOP-T04` |
| `TOP-03` continuous maps and map classes | `TOP-T05` through `TOP-T06` |
| `TOP-04` homeomorphisms and topological invariants | `TOP-T07` |
| `TOP-05` separation axioms, normality, and Urysohn routes | `TOP-T08` through `TOP-T09` |
| `TOP-06` general compactness | `TOP-T10` through `TOP-T11` |
| `TOP-07` metric compactness and function spaces | `TOP-T12` through `TOP-T13` |
| `TOP-08` connectedness and path connectedness | `TOP-T14` through `TOP-T15` |
| `TOP-09` countability, separability, Lindelof, and metrizability | `TOP-T16` through `TOP-T17` |
| `TOP-10` complete metric and Baire spaces | `TOP-T18` through `TOP-T19` |
| `TOP-11` product spaces | `TOP-T20` through `TOP-T21` |
| `TOP-12` quotient spaces and gluing | `TOP-T22` through `TOP-T23` |
| `TOP-13` local properties and paracompactness | `TOP-T24` through `TOP-T25` |
| `TOP-14` nets, filters, and ultrafilters | `TOP-T26` through `TOP-T27` |
| `TOP-15` homotopy foundations | `TOP-T28` through `TOP-T29` |
| `TOP-16` fundamental groups | `TOP-T30` through `TOP-T31` |
| `TOP-17` covering spaces | `TOP-T32` through `TOP-T33` |
| `TOP-18` homology | `TOP-T34` through `TOP-T36` |
| `TOP-19` cohomology | `TOP-T37` through `TOP-T38` |
| `TOP-20` simplicial and CW complexes | `TOP-T39` through `TOP-T40` |
| `TOP-21` topological manifolds | `TOP-T41` through `TOP-T42` |
| `TOP-22` differential topology | `TOP-T43` through `TOP-T44` |
| `TOP-23` surfaces and low-dimensional topology | `TOP-T45` through `TOP-T46` |
| `TOP-24` fixed-point and degree-style theorems | `TOP-T47` through `TOP-T48` |
| `TOP-25` dimension theory | `TOP-T49` through `TOP-T50` |
| `TOP-26` topological dynamics | `TOP-T51` through `TOP-T52` |
| `TOP-27` geometric topology and characteristic classes | `TOP-T53` through `TOP-T54` |
| `TOP-28` K-theory, spectral sequences, and stable homotopy | `TOP-T55` through `TOP-T56` |
| `TOP-29` packaging and promotion | `TOP-T57` |

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `TOQ-001` | `TOP-T00` |
| `TOQ-002` | `TOP-T01` |
| `TOQ-003` | `TOP-T02` |
| `TOQ-004` | `TOP-T03` |
| `TOQ-005` | `TOP-T04` |
| `TOQ-006` | `TOP-T05` |
| `TOQ-007` | `TOP-T06`, `TOP-T07` |
| `TOQ-008` | `TOP-T08` |
| `TOQ-009` | `TOP-T10` |
| `TOQ-010` | `TOP-T11` |
| `TOQ-011` | `TOP-T12` |
| `TOQ-012` | `TOP-T14` |
| `TOQ-013` | `TOP-T16` |
| `TOQ-014` | `TOP-T20` |
| `TOQ-015` | `TOP-T22` |
| `TOQ-016` | `TOP-T18`, `TOP-T19` |
| `TOQ-017` | `TOP-T26` |
| `TOQ-018` | `TOP-T28` |
| `TOQ-019` | `TOP-T30` |
| `TOQ-020` | `TOP-T34` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `TOP-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `TOP-T01`, `TOP-T03`, `TOP-T05`, `TOP-T10`, `TOP-T18`, `TOP-T26`, `TOP-T30`, `TOP-T34`, `TOP-T37`, `TOP-T39`, `TOP-T41`, `TOP-T43`, `TOP-T49`, `TOP-T51`, `TOP-T53`, `TOP-T55` | target `L2` derived certificates from the first proof attempt; split missing foundation evidence before source edits instead of landing interface milestones |
| `TOP-T02`, `TOP-T04`, `TOP-T06` through `TOP-T08`, `TOP-T12`, `TOP-T14`, `TOP-T16`, `TOP-T20`, `TOP-T22`, `TOP-T24`, `TOP-T28`, `TOP-T32`, `TOP-T47` | target `L2` derived certificates where prerequisites exist |
| `TOP-T09`, `TOP-T11`, `TOP-T13`, `TOP-T15`, `TOP-T17`, `TOP-T19`, `TOP-T21`, `TOP-T23`, `TOP-T25`, `TOP-T27`, `TOP-T29`, `TOP-T31`, `TOP-T33`, `TOP-T35`, `TOP-T36`, `TOP-T38`, `TOP-T40`, `TOP-T42`, `TOP-T44` through `TOP-T46`, `TOP-T48`, `TOP-T50`, `TOP-T52`, `TOP-T54`, `TOP-T56` | split before source edits if prerequisites are absent; otherwise target `L2` derived certificates for all theorem statements |
| `TOP-T57` | `L3` public closure and package verification |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

---

## Milestones

### TOP-T00 Build Topology Theorem Card Inventory

- Status: Completed
- Depends on: None
- Areas: `proofs/README.md`, `proofs/topology-theorem-cards.md`,
  proof-corpus theorem-card sidecars, AI index sidecars
- Tasks:
  - Create theorem cards for all `TOP-00` through `TOP-29` theorem families.
  - Record stable English identifier, Japanese display name, target level,
    primary milestone, proposed module, set-theoretic evidence, and dependency
    tags.
  - Record duplicate-home decisions for compactness, Baire, Urysohn, Tietze,
    Banach fixed point, fundamental group, homology, de Rham, Stokes, and
    Poincare-family theorem names.
  - Mark each target as foundation, derived theorem, specialization, package
    alias, or long-term interface.
- Deliverables:
  - `proofs/topology-theorem-cards.md` topology theorem-card inventory and
    duplicate map.
- Acceptance criteria:
  - Every roadmap theorem family has exactly one primary home milestone.
  - Analysis aliases `ANA-T22`, `ANA-T23`, and `ANA-T27` point to topology
    theorem cards only where topology owns the result.
  - No theorem card treats source, replay, theorem indexes, or this todo as
    proof evidence.
- Verification:
  - `rg -n "TOP-00|TOP-29|ANA-T22|ANA-T23|ANA-T27|Poincare" proofs/topology-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md proofs/topology-theorem-cards.md`
  - `git diff --check`
  - Completed with documentation-only validation; no certificate was generated
    because `TOP-T00` is an `L0` theorem-card inventory task.

### TOP-T01 Add Topological-Space Law Package

- Status: Completed
- Depends on: `TOP-T00`
- Areas: `Proofs.Ai.Topology.Basic`, `proofs/Proofs/Ai/Topology/Basic/*`,
  `tools/proof-corpus/src/main.rs`, `proofs/README.md`
- Tasks:
  - Define the first general topological-space law package using ordinary
    structures.
  - Add open-set and closed-set predicates, empty/universal opens, finite
    intersection and arbitrary union laws, and open/closed duality names.
  - Bridge existing `Proofs.Ai.Analysis.AbstractMetricTopology` neighborhood
    vocabulary without replacing it.
- Deliverables:
  - First certificate-backed `Proofs.Ai.Topology.Basic` module.
- Acceptance criteria:
  - Topology is not added as a kernel primitive.
  - The metric-topology bridge keeps the analysis module reusable by later
    metric compactness tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completed with `UniversalSet`, `EmptySet`, `SetIntersection`,
    `IndexedUnion`, `SetComplement`, `ClosedSet`,
    `TopologicalNeighborhood`, `TopologicalSpaceLawPackage`, and
    `MetricBallOpenBridge` as ordinary proof-corpus declarations. Topology was
    not added as a kernel primitive, and the metric bridge imports
    `Proofs.Ai.Analysis.AbstractMetricTopology`.

### TOP-T02 Add Closure, Interior, Boundary, And Dense-Set Laws

- Status: Completed
- Depends on: `TOP-T01`
- Areas: `Proofs.Ai.Topology.Closure`,
  `proofs/Proofs/Ai/Topology/Closure/*`, `tools/proof-corpus/src/main.rs`,
  `proofs/README.md`
- Tasks:
  - Added neighborhood, interior, closure, exterior, boundary, limit-point,
    isolated-point, and dense-set statement names in
    `Proofs.Ai.Topology.Closure`.
  - Proved closure/interior local-neighborhood equivalence, exterior/complement
    interior duality, boundary projections, and dense-set neighborhood hitting
    laws from the `TOP-T01` topology vocabulary.
  - Added `ClosureOperatorCharacterization` for the builtin closure-point
    operator and an explicitly marked `KuratowskiClosureInterface` split with
    projection theorems.
- Deliverables:
  - Certificate-backed closure and local-set theorem layer for later
    compactness, connectedness, and maps.
- Acceptance criteria:
  - Closure and interior laws are derived from `TOP-T01` assumptions and checked
    by source-free certificate verification.
  - Dense and limit-point definitions use open-neighborhood/intersection
    vocabulary and do not depend on metric-specific sequence vocabulary.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Closure`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Closure`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Closure` has 11 definitions, 37
    theorems, and no module-local axioms.

### TOP-T03 Add Bases, Subbases, And Generated Topologies

- Status: Completed
- Depends on: `TOP-T02`
- Areas: `Proofs.Ai.Topology.Generated`,
  `proofs/Proofs/Ai/Topology/Generated/*`, `tools/proof-corpus/src/main.rs`,
  `proofs/README.md`
- Tasks:
  - Defined basis covers, basis refinements, basis-generated open sets,
    generated topologies, topological-basis packages, topology comparison,
    subbasis finite-intersection refinements, subbasis-generated open sets, and
    explicit subbasis choice routes.
  - Proved basis open-set characterization, generated topology universal/empty
    open laws, binary-intersection and indexed-union closure, topology
    comparison reflexivity/transitivity/application, and cover/refinement
    transport lemmas.
  - Recorded subbasis route choice requirements as `SubbasisChoiceRoute` and
    proved the comparison from subbasis-generated opens to a chosen
    basis-generated topology without compactness assumptions.
- Deliverables:
  - Certificate-backed generated-topology module reusable by products,
    quotients, compact-open topology, and examples.
- Acceptance criteria:
  - Generated topology proofs keep cover/refinement evidence explicit through
    `BasisCoverAt`, `BasisRefinesAt`, `SubbasisRefinesAt`, and
    `SubbasisChoiceRoute`.
  - Subbasis theorem names do not assume compactness results from `TOP-T10`
    or `TOP-T11`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Generated`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Generated`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Generated` has 10 definitions, 32
    theorems, and no module-local axioms.

### TOP-T04 Add Subspace, Initial, And Final Topology Routes

- Status: Completed
- Depends on: `TOP-T03`
- Areas: `Proofs.Ai.Topology.Subspace`,
  `Proofs.Ai.Topology.InitialFinal`,
  `proofs/Proofs/Ai/Topology/Subspace/*`,
  `proofs/Proofs/Ai/Topology/InitialFinal/*`,
  `tools/proof-corpus/src/main.rs`, `proofs/README.md`
- Tasks:
  - Defined `SubspaceOpen`, `SubspaceTopology`, and `SubspaceClosed` with
    certificate-backed relative open and closed characterization theorems.
  - Added initial topology and final topology universal-property route packages
    through `OpenPreimageRoute`, `InitialTopologyRoute`, and
    `FinalTopologyRoute`.
  - Prepared dependency hooks for embeddings, products, and quotients without
    redefining the subspace topology in downstream module families.
- Deliverables:
  - Subspace and initial/final topology modules with source, certificate, meta,
    and replay sidecars.
- Acceptance criteria:
  - Subspace topology is not redefined in metric, manifold, or CW modules.
  - Initial/final universal properties do not import `TOP-T05` continuity
    facts; they use preimage-open route statements as dependency hooks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Subspace`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.InitialFinal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Subspace --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.InitialFinal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Subspace` has 5 definitions and 12
    theorems; `Proofs.Ai.Topology.InitialFinal` has 6 definitions and 10
    theorems. Both modules declare no axioms.

### TOP-T05 Add Continuous-Map Core

- Status: Completed
- Depends on: `TOP-T04`
- Areas: `Proofs.Ai.Topology.Continuous`,
  `proofs/Proofs/Ai/Topology/Continuous/*`,
  `tools/proof-corpus/src/main.rs`, `proofs/README.md`
- Tasks:
  - Added a continuous-map package by open-preimage witnesses with explicit
    membership laws, avoiding set-extensionality assumptions in the kernel.
  - Proved closed-preimage, neighborhood/local-continuity, and closure-image
    characterization routes.
  - Added identity, composition, subspace-inclusion, and subspace-restriction
    lemmas.
  - Kept product and quotient continuity criteria as dependency-tagged aliases
    to the `TOP-T04` initial/final hooks until `TOP-T20` and `TOP-T22`.
- Deliverables:
  - Continuous-map theorem layer with source, certificate, meta, and replay
    sidecars.
- Acceptance criteria:
  - Continuous-map proofs use `TOP-T01` through `TOP-T04` vocabulary, including
    topological neighborhoods, closure points, subspace topology, and
    initial/final hooks.
  - Maps into products and maps out of quotients are aliases to
    `ProductInitialHook` and `QuotientFinalHook`, not duplicate product or
    quotient definitions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Continuous`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Continuous --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Continuous` has 9 definitions and 14
    theorems. The module declares no axioms.

### TOP-T06 Add Pasting, Open/Closed Maps, And Embeddings

- Status: Completed
- Depends on: `TOP-T05`
- Areas: `Proofs.Ai.Topology.MapClass`
- Tasks:
  - Add pasting lemma variants with closed-cover and open-cover side
    conditions.
  - Define open map, closed map, embedding, quotient-map, and homeomorphism
    predicate hooks.
  - Add compact-open topology statement interface for later function-space
    work.
- Deliverables:
  - Map-class module with reusable theorem names.
  - Added `proofs/Proofs/Ai/Topology/MapClass/*` certificate artifacts and
    registered `Proofs.Ai.Topology.MapClass` in the proof-corpus module
    catalog.
- Acceptance criteria:
  - Embedding distinguishes injective continuous maps from homeomorphisms onto
    images.
  - Pasting lemmas state cover hypotheses and compatibility assumptions
    explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.MapClass`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.MapClass --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.MapClass` has 12 definitions and 33
    theorems. The module declares no axioms.

### TOP-T07 Add Homeomorphism And Invariant Alias Framework

- Status: Completed
- Depends on: `TOP-T06`
- Areas: `Proofs.Ai.Topology.Homeomorphism`, `Proofs.Ai.Topology.Invariant`
- Tasks:
  - Prove homeomorphism equivalence, inverse continuity criteria, and open and
    closed set correspondence under homeomorphism.
  - Add a preservation framework for compactness, connectedness, separation,
    countability, homotopy, homology, Euler characteristic, and manifold
    dimension.
  - Create non-homeomorphism theorem cards that point to the invariant they
    will use.
- Deliverables:
  - Homeomorphism module and invariant alias framework.
  - Added `proofs/Proofs/Ai/Topology/Homeomorphism/*` and
    `proofs/Proofs/Ai/Topology/Invariant/*` certificate artifacts and
    registered both modules in the proof-corpus module catalog.
- Acceptance criteria:
  - Preservation theorems import primary property milestones instead of
    defining compactness, homology, or dimension locally.
  - Invariance of domain, Euler characteristic, and dimension invariance remain
    interfaces until their primary routes exist.
  - Closed-map correspondence is recorded as an explicit hook because the
    current map-image layer avoids adding set extensionality or complement
    exactness axioms; open-map correspondence is derived from the inverse
    continuity witness.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homeomorphism`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Invariant`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Homeomorphism --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Invariant --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Homeomorphism` has 5 definitions and
    15 theorems; `Proofs.Ai.Topology.Invariant` has 4 definitions and 8
    theorems. Both modules declare no axioms.

### TOP-T08 Add T0, T1, Hausdorff, Regular, And Normal Basics

- Status: Completed
- Depends on: `TOP-T05`
- Areas: `Proofs.Ai.Topology.Separation.Basic`, `Proofs.Ai.Topology.Separation.Normal`
- Tasks:
  - Define T0, Kolmogorov, T1, Hausdorff, regular, completely regular, normal,
    and Tychonoff predicates.
  - Prove singleton and finite-set closedness in T1 spaces, diagonal
    characterization of Hausdorffness, compact subsets of Hausdorff spaces are
    closed, and metric spaces are normal.
  - Split net-limit uniqueness until `TOP-T26` if the proof needs nets.
- Deliverables:
  - Added and registered `Proofs.Ai.Topology.Separation.Basic` with 9
    definitions and 29 theorems for distinct points, point-open exclusion,
    disjoint open neighborhoods, T0/Kolmogorov, T1, Hausdorff, Hausdorff
    diagonal criteria, and compact-Hausdorff closed-subset routes.
  - Added and registered `Proofs.Ai.Topology.Separation.Normal` with 9
    definitions and 25 theorems for closed-set disjointness, open-set
    separation, point/closed-set separation, regularity, complete regularity,
    normality, Tychonoff spaces, metric normality, and compact Hausdorff
    normality routes.
- Acceptance criteria:
  - Every separation theorem states the exact axiom level it requires.
  - T1 singleton/finite closedness, Hausdorff diagonal characterization,
    compact-Hausdorff closed subsets, metric normality, and compact Hausdorff
    normality are exposed as explicit evidence/route packages until the
    compactness infrastructure in `TOP-T10`/`TOP-T11` is available.
  - Net-limit uniqueness is left to `TOP-T26`; no net vocabulary was added in
    this layer.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Separation.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Separation.Normal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Separation.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Separation.Normal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: both separation modules declare no axioms.

### TOP-T09 Add Urysohn, Tietze, And Compactification Interfaces

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T10`
- Areas: `Proofs.Ai.Topology.Separation.Urysohn`
- Tasks:
  - Add Urysohn lemma and Tietze extension theorem routes with normality and
    codomain assumptions.
  - Add Stone-Cech compactification interface.
  - Record `TOP-T27` as the ultrafilter blocker for Stone-Cech work beyond
    theorem-card or interface level.
- Deliverables:
  - Separation Urysohn/Tietze module or dependency-tagged interfaces.
- Acceptance criteria:
  - Urysohn and Tietze do not land before normal-space APIs are stable.
  - Stone-Cech starts as an `L2` proof route only after ultrafilters and
    function-algebra prerequisites exist; otherwise split that blocker.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Separation.Urysohn`
  - `rg -n "Urysohn|Tietze|Stone-Cech|TOP-T09" proofs/topology-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T10 Add Open-Cover Compactness Core

- Status: Completed
- Depends on: `TOP-T05`
- Areas: `Proofs.Ai.Topology.Compact.Basic`
- Tasks:
  - Define open-cover compactness and finite-intersection-property
    compactness.
  - Prove closed subsets of compact spaces are compact, continuous images of
    compact spaces are compact, compactness is homeomorphism invariant, and
    compact-to-Hausdorff continuous bijections are homeomorphisms.
  - Add the tube lemma if finite products are available; otherwise leave it
    dependency-tagged for `TOP-T20`.
- Deliverables:
  - Added and registered `Proofs.Ai.Topology.Compact.Basic` with 18
    definitions and 34 theorems for selected subfamily membership, open covers
    of subsets, finite subcovers, open-cover compactness,
    finite-intersection-property compactness, compact spaces, closed-subset
    compactness routes, continuous-image compactness routes, compactness
    invariant transfer, compact-to-Hausdorff continuous-bijection routes, and a
    tube-lemma dependency tag.
- Acceptance criteria:
  - Compactness is not specialized to metric spaces in this module.
  - Hausdorff-dependent compactness theorems import separation results.
  - Closed-subset, continuous-image, homeomorphism-invariance, and
    compact-to-Hausdorff continuous-bijection results state their exact route
    evidence instead of adding hidden compactness, choice, quotient, or
    equality-transport axioms.
  - The tube lemma is recorded as dependency-tagged for `TOP-T20` because the
    finite-product topology layer is not yet part of this milestone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Compact.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Compact.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: `Proofs.Ai.Topology.Compact.Basic` declares no axioms.

### TOP-T11 Add Alexander, Tychonoff, And Compactification Routes

- Status: Completed
- Depends on: `TOP-T03`, `TOP-T10`, `TOP-T20`
- Areas: `Proofs.Ai.Topology.Compact.Product`, `Proofs.Ai.Topology.Compactification`
- Tasks:
  - Completed: Add Alexander subbase theorem with explicit subbasis and choice evidence.
  - Completed: Add Tychonoff theorem route and product compactness alias.
  - Completed: Add one-point compactification and record `TOP-T26` and `TOP-T27` as
    blockers for compactness via nets, filters, and ultrafilters.
- Deliverables:
  - `Proofs.Ai.Topology.Compact.Product` adds `ProductCompactnessAlias`,
    `FiniteProductCompactnessRoute`, `TychonoffProductTheoremRoute`, and
    `AlexanderSubbaseTheoremRoute`, with 16 certificate-backed theorem projections/applications.
  - `Proofs.Ai.Topology.Compactification` adds compactification data, generic route, one-point
    route, Stone-Cech route, and universal-property packages, with 15 certificate-backed theorem
    projections/applications.
- Acceptance criteria:
  - Completed: Choice, product-topology, subbase-cover, ultrafilter, and function-algebra
    assumptions are explicit evidence slots.
  - Completed: Metric compactness stays primary in `TOP-T12`; these modules only add general
    compactness and compactification routes.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Compact.Product`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Compactification`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Compact.Product --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Compactification --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: both modules declare no axioms; Stone-Cech remains dependency-tagged until
    the `TOP-T26`/`TOP-T27` net/filter/ultrafilter layers exist.

### TOP-T12 Add Metric Compactness Bridge

- Status: Completed
- Depends on: `TOP-T10`, `ANA-T05`, `ANA-T12`, `ANA-T22`
- Areas: `Proofs.Ai.Topology.Metric.Compact`
- Tasks:
  - Completed: Bridge `Proofs.Ai.Analysis.AbstractMetricTopology` to the general
    topology vocabulary.
  - Completed: Prove compact metric spaces are complete and totally bounded through an explicit
    `CompactMetricTheoremRoute`.
  - Completed: Prove compact metric iff complete and totally bounded, and add sequential
    compactness equivalence where sequence compactness exists.
- Deliverables:
  - `Proofs.Ai.Topology.Metric.Compact` adds finite epsilon-net, total boundedness,
    Cauchy-completeness, sequential compactness, metric-compact bridge, compact/complete/total
    bounded equivalence route, Heine-Borel prerequisite route, and Bolzano-Weierstrass prerequisite
    route packages, with 22 certificate-backed theorem projections/compositions.
- Acceptance criteria:
  - Completed: The metric bridge reuses `MetricBall`, `MetricBallOpenBridge`, and
    `TopologicalNeighborhood`; it does not fork a second neighborhood API.
  - Completed: Heine-Borel and Bolzano-Weierstrass aliases list Euclidean and sequence
    prerequisites before source edits.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metric.Compact`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Metric.Compact --verified-cache authoring`
  - `rg -n "AbstractMetricTopology|ANA-T22|Metric.Compact" proofs/topology-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: the module declares no axioms; Euclidean-specific Heine-Borel construction
    remains dependency-tagged for the broader `ANA-T22` track.

### TOP-T13 Add Function-Space And Arzela-Ascoli Interfaces

- Status: Pending
- Depends on: `TOP-T06`, `TOP-T12`, `TOP-T18`, `ANA-T23`
- Areas: `Proofs.Ai.Topology.FunctionSpace`
- Tasks:
  - Define compact-open topology basics and equicontinuity vocabulary.
  - Add Lebesgue number lemma and Arzela-Ascoli theorem route.
  - Keep Stone-Weierstrass out of this milestone unless algebra-of-functions
    prerequisites exist.
- Deliverables:
  - Function-space topology module or statement interface.
- Acceptance criteria:
  - Arzela-Ascoli states compactness and equicontinuity hypotheses explicitly.
  - Function-space topology work remains compatible with analysis `ANA-T23`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FunctionSpace`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T14 Add Connectedness And Component Core

- Status: Completed
- Depends on: `TOP-T05`, `TOP-T20`
- Areas: `Proofs.Ai.Topology.Connected.Basic`
- Tasks:
  - Completed: Define clopen sets, clopen separations, connected spaces, and the
    clopen-separation characterization.
  - Completed: Add continuous image, closure, union, and product connectedness
    routes under stated hypotheses.
  - Completed: Add connected components, closedness of components, local
    connectedness hooks, and totally disconnected interfaces.
- Deliverables:
  - `Proofs.Ai.Topology.Connected.Basic` adds 12 definitions and 31
    certificate-backed theorem projections/applications for connectedness,
    connected subsets, image/closure/union/product routes, components, local
    connectedness, and total disconnectedness.
- Acceptance criteria:
  - Completed: Connected components are separate `ConnectedComponent` packages
    and no path-component vocabulary is introduced in this layer.
  - Completed: Product connectedness imports and requires `ProductInitialHook`
    from `Proofs.Ai.Topology.InitialFinal`; it does not duplicate product-space
    definitions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Connected.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Connected.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - Completion note: the module declares no axioms; product connectedness remains
    dependency-routed until the later full `TOP-T20` product-space layer exists.

### TOP-T15 Add Path Connectedness And Continuum Interfaces

- Status: Pending
- Depends on: `TOP-T14`, `TOP-T28`, analysis real interval foundations
- Areas: `Proofs.Ai.Topology.Connected.Path`, `Proofs.Ai.Topology.Continuum`
- Tasks:
  - Define path-connectedness, path components, and local path-connectedness.
  - Prove path-connected implies connected and local path-connected component
    openness routes.
  - Add real interval connectedness, IVT aliases, Peano continuum,
    Hahn-Mazurkiewicz, Jordan curve, and Jordan-Brouwer interfaces.
- Deliverables:
  - Path-connectedness theorem layer and continuum theorem interfaces.
- Acceptance criteria:
  - Interval and IVT statements import analysis real/continuity foundations.
  - Jordan and continuum theorems start as `L2` proof routes only after plane,
    manifold, and homology prerequisites exist; otherwise split blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Connected.Path`
  - `rg -n "Jordan|Hahn-Mazurkiewicz|IVT|TOP-T15" proofs/topology-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T16 Add Countability, Separability, And Lindelof Basics

- Status: Completed (2026-06-07)
- Depends on: `TOP-T05`, `TOP-T12`
- Areas: `Proofs.Ai.Topology.Countability`
- Tasks:
  - Completed: Define first countable, second countable, separable,
    Lindelof, sigma-compact, and Frechet-Urysohn predicates.
  - Completed: Prove second countable implies first countable, separable, and
    Lindelof through an explicit consequence route.
  - Completed: Prove separable metric iff second countable, Lindelof
    closed-subspace and continuous-image routes, and first-countable closure by
    sequences with the relevant route hypotheses explicit.
- Deliverables:
  - Completed: `Proofs.Ai.Topology.Countability` source, certificate, metadata,
    replay, and AI theorem index entries.
- Acceptance criteria:
  - Satisfied: General separability inheritance claims are not stated without
    hypotheses.
  - Satisfied: Sequence sufficiency theorems depend on first-countability
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Countability`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Countability --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Countability --verified-cache off`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`

### TOP-T17 Add Metrization And Example-Space Routes

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T16`
- Areas: `Proofs.Ai.Topology.Metrization`, `Proofs.Ai.Topology.Examples.Sorgenfrey`
- Tasks:
  - Add Urysohn, Nagata-Smirnov, Bing, Moore, and Smirnov metrization routes.
  - Add Polish second-countability and separability interfaces.
  - Add Sorgenfrey line and plane theorem-card examples.
- Deliverables:
  - Metrization interfaces and example-space cards.
- Acceptance criteria:
  - Metrization statements identify regularity, normality, sigma-local
    finiteness, developability, or Moore-space assumptions explicitly.
  - Example spaces remain statement cards until their topology constructions
    are available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metrization`
  - `rg -n "Sorgenfrey|Nagata|Bing|Moore|Smirnov|TOP-T17" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T18 Add Complete Metric Space And Completion Core

- Status: Completed (2026-06-08)
- Depends on: `TOP-T12`, `Proofs.Ai.Analysis.AbstractFixedPoint`
- Areas: `Proofs.Ai.Topology.Metric.Completion`
- Tasks:
  - Completed: Define Cauchy sequence and complete metric-space interfaces compatible
    with existing analysis fixed-point evidence.
  - Completed: Add completion existence and uniqueness route.
  - Completed: Prove closed subspaces of complete metric spaces are complete and add
    Cantor intersection theorem.
  - Completed: Add Banach fixed point alias from `Proofs.Ai.Analysis.AbstractFixedPoint`.
- Deliverables:
  - Completed: `Proofs.Ai.Topology.Metric.Completion` source, certificate,
    metadata, replay, and AI theorem index entries.
  - Completed: The module adds 7 definitions and 15 certificate-backed
    theorem projections/applications for complete metric cores, completion
    route uniqueness from universal property, closed-subspace completeness,
    Cantor intersection, and the Banach fixed-point topology alias.
- Acceptance criteria:
  - Satisfied: Banach fixed point remains primary in the analysis fixed-point
    module; `banach_fixed_point_from_topology_alias` delegates to
    `banach_fixed_point_from_args`.
  - Satisfied: Completion uniqueness is obtained by applying a
    `CompletionUniversalProperty -> CompletionUniquenessEvidence` law packaged
    in `MetricCompletionRoute`, not by assuming uniqueness as the universal
    property itself.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metric.Completion`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Metric.Completion --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`

### TOP-T19 Add Baire Category And Genericity Route

- Status: Completed (2026-06-08)
- Depends on: `TOP-T18`, `ANA-T23`
- Areas: `Proofs.Ai.Topology.Baire`
- Tasks:
  - Completed: Prove Baire category theorem for complete metric spaces.
  - Completed: Add Baire routes for locally compact Hausdorff spaces and Polish spaces
    when prerequisites exist.
  - Completed: Define nowhere dense, meagre, comeagre, generic property, and dense open
    countable-intersection lemmas.
  - Completed: Add Choquet and Banach-Mazur game interfaces.
- Deliverables:
  - Completed: `Proofs.Ai.Topology.Baire` source, certificate, metadata,
    replay, and AI theorem index entries.
  - Completed: The module adds 14 definitions and 22 certificate-backed
    theorem projections/applications for nowhere dense/meagre/comeagre/generic
    vocabulary, dense-open countable intersections, complete metric Baire,
    locally compact Hausdorff Baire, Polish Baire, functional-analysis input,
    and game interfaces.
- Acceptance criteria:
  - Satisfied: Functional-analysis theorems are not reproved here; Baire
    exposes `FunctionalAnalysisBaireInput` for open mapping, closed graph, and
    uniform boundedness routes to import.
  - Satisfied: Game-theoretic Baire statements are kept as non-promoted
    interfaces through `ChoquetGameInterface` and `BanachMazurGameInterface`;
    L2 proof routes require the game definitions first.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Baire`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Baire --verified-cache authoring`
  - `rg -n "Baire|open mapping|closed graph|uniform boundedness|ANA-T27" proofs/topology-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`

### TOP-T20 Add Product Topology Core

- Status: Completed (2026-06-07)
- Depends on: `TOP-T04`, `TOP-T05`
- Areas: `Proofs.Ai.Topology.Product.Basic`
- Tasks:
  - Completed: Define product topology core evidence and package the universal
    property through `ProductInitialHook`.
  - Completed: Prove projections are continuous and product basic-open facts,
    and record maps-into-products continuity criteria, product basis routes,
    and finite product projection continuity facts with explicit route
    evidence.
  - Completed: Add product hooks for compactness, connectedness, countability,
    and local properties.
- Deliverables:
  - Completed: `Proofs.Ai.Topology.Product.Basic` source, certificate,
    metadata, replay, and AI theorem index entries.
- Acceptance criteria:
  - Satisfied: Product definitions use generated/initial topology
    infrastructure through `ProductInitialHook` and `InitialTopologyRoute`.
  - Satisfied: Continuity into products imports `TOP-T05` and uses
    `ContinuousMap` rather than restating continuity.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Product.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Product.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Product.Basic --verified-cache off`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`

### TOP-T21 Add Product Preservation Theorems

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T10`, `TOP-T14`, `TOP-T16`, `TOP-T20`
- Areas: `Proofs.Ai.Topology.Product.Properties`
- Tasks:
  - Prove products preserve Hausdorff, regular, completely regular,
    connected, and path-connected properties under stated hypotheses.
  - Add Tychonoff/product compactness alias, finite and infinite local
    compactness criteria, and countable product metrizability/Polishness
    routes.
  - Prove product closure formula with indexing and projection assumptions.
- Deliverables:
  - Product property theorem module.
- Acceptance criteria:
  - Product compactness imports `TOP-T11` and records choice evidence.
  - Countable product metric results import countability and metrizability
    hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Product.Properties`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T22 Add Quotient Topology Core

- Status: Completed (2026-06-07)
- Depends on: `TOP-T04`, `TOP-T05`
- Areas: `Proofs.Ai.Topology.Quotient.Basic`
- Tasks:
  - Completed: Define quotient topology core evidence and quotient topology
    map evidence over `QuotientFinalHook`.
  - Completed: Prove quotient projection continuity, open/closed set
    characterization routes, descent continuity from exact composition
    preimage-open evidence, and the reverse composition-continuity direction.
  - Completed: Add open and closed quotient map route theorems with explicit
    image-open and image-closed hypotheses.
- Deliverables:
  - Completed: `Proofs.Ai.Topology.Quotient.Basic` source, certificate,
    metadata, replay, and AI theorem index entries.
- Acceptance criteria:
  - Satisfied: Quotient topology uses final topology infrastructure from
    `TOP-T04` through `QuotientFinalHook` and `FinalTopologyRoute`.
  - Satisfied: Quotient continuity criteria import `TOP-T05`, produce and
    consume `ContinuousMap`, and do not duplicate the continuity definition.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Quotient.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Quotient.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Quotient.Basic --verified-cache off`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
  - `git diff --check`

### TOP-T23 Add Gluing And Standard Quotient Model Interfaces

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T10`, `TOP-T14`, `TOP-T22`, `TOP-T39`
- Areas: `Proofs.Ai.Topology.Quotient.Models`
- Tasks:
  - Prove compact, connected, and path-connected quotient preservation
    theorems.
  - Add Hausdorff quotient conditions and closed equivalence-relation graph
    route.
  - Add gluing-space theorem and interfaces for circle, sphere, torus,
    projective space, and CW quotient models.
- Deliverables:
  - Quotient model and gluing theorem module.
- Acceptance criteria:
  - Failure of Hausdorff preservation is recorded as an example card, not a
    false universal theorem.
  - Standard quotient models start as `L2` proof routes only after their
    concrete spaces exist; otherwise split those blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Quotient.Models`
  - `rg -n "Hausdorff quotient|circle|torus|projective|TOP-T23" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T24 Add Local Property Core

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T10`, `TOP-T14`, `TOP-T16`, `TOP-T20`
- Areas: `Proofs.Ai.Topology.Local`
- Tasks:
  - Define local compactness, local connectedness, and local path-connectedness.
  - Prove local compact Hausdorff properties, one-point compactification alias,
    and local component openness theorems.
  - Add manifold-local-property hooks for `TOP-T41`.
- Deliverables:
  - Local property theorem module.
- Acceptance criteria:
  - Local compactness and local connectedness are distinct predicates.
  - One-point compactification imports compactification results rather than
    redefining them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Local`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T25 Add Paracompactness And Partition-Of-Unity Route

- Status: Pending
- Depends on: `TOP-T08`, `TOP-T10`, `TOP-T12`, `TOP-T24`
- Areas: `Proofs.Ai.Topology.Paracompact`, `Proofs.Ai.Topology.PartitionOfUnity`
- Tasks:
  - Define paracompactness and locally finite open refinements.
  - Prove compact spaces and metric spaces are paracompact, and paracompact
    Hausdorff spaces are normal.
  - Add partition of unity subordinate to an open cover and manifold
    partition-of-unity alias.
  - Add Michael selection, Morita, Stone paracompactification, and
    Nagata-Smirnov paracompactness interfaces.
- Deliverables:
  - Paracompactness module and partition-of-unity interface.
- Acceptance criteria:
  - Partition of unity states paracompactness, Hausdorffness, local finiteness,
    and codomain/ring assumptions.
  - Selection and paracompactification theorems start as `L2` proof routes only
    after selection machinery exists; otherwise split blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Paracompact`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.PartitionOfUnity`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T26 Add Nets And Filters Core

- Status: Pending
- Depends on: `TOP-T05`, `TOP-T08`, `TOP-T10`, `TOP-T16`
- Areas: `Proofs.Ai.Topology.Net`, `Proofs.Ai.Topology.Filter`
- Tasks:
  - Define net convergence and filter convergence.
  - Prove closure and closed-set characterization by nets, continuity by net
    convergence, uniqueness of net limits in Hausdorff spaces, and first
    countable sequence sufficiency.
  - Add compactness via convergent subnets when subnet infrastructure exists.
- Deliverables:
  - Net and filter convergence modules.
- Acceptance criteria:
  - Sequence sufficiency imports first-countability results from `TOP-T16`.
  - Hausdorff uniqueness imports separation results instead of duplicating
    them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Net`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Filter`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T27 Add Ultrafilter, Tychonoff, And Stone-Cech Routes

- Status: Pending
- Depends on: `TOP-T09`, `TOP-T11`, `TOP-T20`, `TOP-T26`
- Areas: `Proofs.Ai.Topology.Ultrafilter`
- Tasks:
  - Define ultrafilter convergence and ultrafilter compactness.
  - Add compactness via filters and ultrafilters, Tychonoff ultrafilter proof
    interface, Moore-Smith convergence, subnet existence, universal net, and
    ultrafilter lemma.
  - Add Stone-Cech compactification via ultrafilters after normality/function
    prerequisites exist.
- Deliverables:
  - Ultrafilter and advanced convergence theorem route.
- Acceptance criteria:
  - Ultrafilter lemma and Tychonoff record set-theoretic evidence explicitly.
  - Stone-Cech imports both normality/function and ultrafilter prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Ultrafilter`
  - `rg -n "ultrafilter|Tychonoff|Stone-Cech|TOP-T27" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T28 Add Homotopy And Retract Core

- Status: Pending
- Depends on: `TOP-T05`, `TOP-T14`, `TOP-T20`, `TOP-T22`
- Areas: `Proofs.Ai.Topology.Homotopy.Basic`, `Proofs.Ai.Topology.Homotopy.Retract`
- Tasks:
  - Define homotopy, homotopy equivalence, contractibility, deformation
    retract, and strong deformation retract.
  - Prove homotopy equivalence is an equivalence relation and basic
    contractible/retract facts.
  - Add homotopy extension and lifting property interfaces.
- Deliverables:
  - Homotopy vocabulary and retract theorem modules.
- Acceptance criteria:
  - Homotopy invariance of fundamental group and homology remains primary in
    `TOP-T30` and `TOP-T35`.
  - Retract statements import product and quotient infrastructure as needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homotopy.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homotopy.Retract`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T29 Add Advanced Homotopy Interfaces

- Status: Pending
- Depends on: `TOP-T28`, `TOP-T35`, `TOP-T39`, `TOP-T55`
- Areas: `Proofs.Ai.Topology.Homotopy.Basic`
- Tasks:
  - Add CW homotopy extension alias.
  - Add Whitehead, Hurewicz, Freudenthal, Blakers-Massey, and Brown
    representation interfaces.
  - Split advanced homotopy statements by CW, homology, cohomology, and
    spectral prerequisites.
- Deliverables:
  - Advanced homotopy theorem-card interfaces.
- Acceptance criteria:
  - Advanced homotopy theorems start as `L2` proof routes only after CW,
    homology, and spectral prerequisites exist; otherwise split blockers.
  - No advanced theorem is used as an axiom by earlier milestones.
- Verification:
  - `rg -n "Whitehead|Hurewicz|Freudenthal|Blakers|Brown representation|TOP-T29" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T30 Add Fundamental Group Core

- Status: Completed
- Depends on: `TOP-T15`, `TOP-T22`, `TOP-T28`, algebra group modules
- Areas: `Proofs.Ai.Topology.FundamentalGroup.Basic`
- Tasks:
  - Define based loops, path homotopy, and fundamental group.
  - Prove group structure, basepoint-change isomorphism, functoriality, and
    homotopy equivalence induces group isomorphism.
  - Record group-law imports from checked algebra group modules.
- Deliverables:
  - Fundamental group base module.
- Acceptance criteria:
  - Basepoints and path-connectedness hypotheses are explicit.
  - Group laws import algebra modules instead of local group axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FundamentalGroup.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T31 Add Van Kampen And Fundamental Group Computations

- Status: Completed
- Depends on: `TOP-T23`, `TOP-T30`, `TOP-T32`
- Areas: `Proofs.Ai.Topology.FundamentalGroup.VanKampen`, `Proofs.Ai.Topology.FundamentalGroup.Computation`
- Tasks:
  - Add Seifert-van Kampen theorem with open-cover and intersection
    hypotheses.
  - Add circle fundamental group route and interfaces for sphere, product,
    wedge, graph, torus, projective space, Klein bottle, and surface
    computations.
  - Add two-cell attachment quotient theorem and Brouwer/Borsuk-Ulam
    application interfaces.
- Deliverables:
  - Van Kampen module and fundamental group computation route.
- Acceptance criteria:
  - Computation examples wait for their model spaces.
  - Van Kampen does not assume the computation conclusion as an input group
    presentation.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FundamentalGroup.VanKampen`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FundamentalGroup.Computation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T32 Add Covering Space And Lifting Core

- Status: Completed
- Depends on: `TOP-T15`, `TOP-T22`, `TOP-T28`, `TOP-T30`
- Areas: `Proofs.Ai.Topology.Covering.Basic`, `Proofs.Ai.Topology.Covering.Lifting`
- Tasks:
  - Define covering maps and local homeomorphism theorem route.
  - Prove path lifting, homotopy lifting, unique lifting, and lifting
    criterion.
  - Add semilocally simply connected and local path-connected hypotheses where
    needed.
- Deliverables:
  - Covering-space base and lifting modules.
- Acceptance criteria:
  - Lifting theorems state basepoint and path/homotopy side conditions.
  - Universal-cover prerequisites are not hidden in the covering definition.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Covering.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Covering.Lifting`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T33 Add Covering Classification And Examples

- Status: Completed
- Depends on: `TOP-T30`, `TOP-T32`
- Areas: `Proofs.Ai.Topology.Covering.Classification`
- Tasks:
  - Add covering spaces and fundamental group subgroup correspondence.
  - Add universal cover existence and uniqueness, simply connected covering
    classification, deck transformation group, and regular/Galois covering
    characterization.
  - Add circle, torus, projective space, Klein bottle, and monodromy action
    example interfaces.
- Deliverables:
  - Covering classification module and example routes.
- Acceptance criteria:
  - Classification theorems import fundamental group results from `TOP-T30`.
  - Example covers remain interfaces until the model spaces exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Covering.Classification`
  - `rg -n "universal cover|deck|monodromy|TOP-T33" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T34 Add Singular Chain And Boundary Core

- Status: Completed
- Depends on: `TOP-T22`, `TOP-T28`, algebra group modules, chain-complex infrastructure
- Areas: `Proofs.Ai.Topology.Homology.Singular`
- Tasks:
  - Define simplex, singular simplex, chain group, boundary, and homology
    group interfaces.
  - Prove boundary squared is zero before homology groups are used.
  - Add functoriality theorem names.
- Deliverables:
  - Singular chain and homology base module.
- Acceptance criteria:
  - Boundary-square-zero is derived, not assumed as the homology definition.
  - Chain groups and coefficients state algebraic prerequisites explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homology.Singular`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T35 Add Homology Invariance And Exactness Route

- Status: Completed
- Depends on: `TOP-T34`
- Areas: `Proofs.Ai.Topology.Homology.Exact`
- Tasks:
  - Prove homotopy invariance of homology.
  - Add long exact sequence, pair long exact sequence, excision, and
    Mayer-Vietoris sequence routes.
  - Keep exactness evidence explicit in theorem statements and proof terms.
- Deliverables:
  - Homology exactness module.
- Acceptance criteria:
  - Exactness is represented by ordinary theorem evidence.
  - Excision and Mayer-Vietoris do not precede the needed subspace and
    quotient foundations.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homology.Exact`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T36 Add Homology Computations And Duality Interfaces

- Status: Completed
- Depends on: `TOP-T31`, `TOP-T33`, `TOP-T35`, `TOP-T39`, `TOP-T41`
- Areas: `Proofs.Ai.Topology.Homology.Computation`
- Tasks:
  - Add simplicial/singular homology comparison and Eilenberg-Steenrod axiom
    interface.
  - Add sphere, torus, projective space, surface, and CW homology computation
    routes.
  - Add Hurewicz, Kunneth, universal coefficient, Poincare duality, Lefschetz
    duality, Alexander duality, and Thom isomorphism interfaces.
- Deliverables:
  - Homology computation module and late duality interfaces.
- Acceptance criteria:
  - Computation theorems import model-space definitions.
  - Duality and Kunneth results start as `L2` proof routes only after
    coefficient, chain-level, and manifold prerequisites exist; otherwise split
    blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homology.Computation`
  - `rg -n "Kunneth|Poincare duality|Alexander duality|TOP-T36" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T37 Add Singular Cohomology And Cup Product Core

- Status: Completed
- Depends on: `TOP-T34`, algebra modules
- Areas: `Proofs.Ai.Topology.Cohomology.Singular`, `Proofs.Ai.Topology.Cohomology.CupProduct`
- Tasks:
  - Define singular cohomology, cochains, coboundary, and cohomology groups.
  - Prove functoriality, homotopy invariance, and cohomology long exact
    sequence route.
  - Define cup product and prove cohomology ring theorem.
- Deliverables:
  - Singular cohomology and cup-product modules.
- Acceptance criteria:
  - Cup product signs, degrees, and coefficient assumptions are explicit.
  - Cohomology exactness imports homology or chain-complex infrastructure
    instead of duplicating it.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Cohomology.Singular`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Cohomology.CupProduct`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T38 Add Cohomology Duality, Operations, And Spectral Interfaces

- Status: Completed
- Depends on: `TOP-T36`, `TOP-T37`, `TOP-T41`
- Areas: `Proofs.Ai.Topology.Cohomology.Duality`
- Tasks:
  - Add Mayer-Vietoris cohomology sequence, universal coefficient theorem,
    Kunneth theorem, and Cech cohomology interface.
  - Add de Rham theorem, Poincare, Lefschetz, Alexander, Thom duality, Gysin
    sequence, and characteristic-class interfaces.
  - Record `TOP-T53`, `TOP-T54`, and `TOP-T55` as blockers for de Rham,
    characteristic-class, and spectral sequence work beyond interface level.
- Deliverables:
  - Cohomology duality and operations interface module.
- Acceptance criteria:
  - de Rham and characteristic-class theorem cards point to `TOP-T53` and
    `TOP-T54`.
  - Spectral sequence entries start as `L2` proof routes only after `TOP-T55`;
    otherwise split that blocker.
- Deferred blockers:
  - de Rham interface is represented by
    `de_rham_interface_blocked_by_top_t53` until `TOP-T53`.
  - characteristic-class and Steenrod interfaces are represented through
    `characteristic_class_interface_blocked_by_top_t54` until `TOP-T54`.
  - spectral sequence interface is represented by
    `spectral_sequence_interface_blocked_by_top_t55` until `TOP-T55`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Cohomology.Duality`
  - `rg -n "de Rham|Steenrod|Gysin|spectral sequence|TOP-T38" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T39 Add Simplicial And CW Complex Core

- Status: Completed
- Depends on: `TOP-T22`, `TOP-T28`, category/simplicial-set interfaces
- Areas: `Proofs.Ai.Topology.SimplicialComplex`, `Proofs.Ai.Topology.CWComplex.Basic`
- Tasks:
  - Define simplicial complex, geometric realization, CW complex, weak
    topology, closure finiteness, and skeleton filtration.
  - Add barycentric subdivision and simplicial approximation interface.
  - Add product and quotient hooks for CW complexes.
- Deliverables:
  - Simplicial complex and CW complex base modules.
- Acceptance criteria:
  - Weak topology and attaching maps import quotient infrastructure from
    `TOP-T22`.
  - Simplicial-set interfaces are not treated as topology proof evidence unless
    imported as checked certificates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.SimplicialComplex`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CWComplex.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T40 Add Cellular Homology And CW Advanced Routes

- Status: Completed
- Depends on: `TOP-T35`, `TOP-T39`
- Areas: `Proofs.Ai.Topology.CWComplex.Cellular`
- Tasks:
  - Add CW homotopy extension property, cellular approximation theorem, and
    cellular homology theorem.
  - Add attaching-map homology computation, Euler characteristic from cell
    decomposition, finite CW compactness, local contractibility, and ANR
    interfaces.
  - Add Whitehead and CW approximation theorem interfaces.
- Deliverables:
  - Cellular homology module and CW advanced interfaces.
- Acceptance criteria:
  - Cellular homology imports homology foundations from `TOP-T34` and
    `TOP-T35`.
  - Whitehead theorem remains an interface until homotopy and homology
    prerequisites are stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CWComplex.Cellular`
  - `rg -n "cellular|Whitehead|Euler characteristic|TOP-T40" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T41 Add Topological Manifold Core

- Status: Completed
- Depends on: `TOP-T08`, `TOP-T15`, `TOP-T16`, `TOP-T22`, `TOP-T24`
- Areas: `Proofs.Ai.Topology.Manifold.Topological`
- Tasks:
  - Define topological manifolds, local Euclidean property, charts, dimension
    tag, Hausdorffness, and second-countability assumptions.
  - Prove local compactness, local connectedness, and local path-connectedness
    for manifolds when Euclidean topology prerequisites exist.
  - Add paracompact manifold partition-of-unity alias.
- Deliverables:
  - Topological manifold base module.
- Acceptance criteria:
  - Manifold dimension invariance and invariance of domain are not assumed by
    the manifold definition.
  - Euclidean-space topology assumptions are imported explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Topological`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T42 Add Manifold Invariance And Separation Interfaces

- Status: Pending
- Depends on: `TOP-T36`, `TOP-T41`, `TOP-T49`
- Areas: `Proofs.Ai.Topology.Manifold.Invariance`
- Tasks:
  - Add manifold dimension invariance and invariance of domain interfaces.
  - Add Jordan-Brouwer separation alias.
  - Record dependencies on homology, dimension theory, and Euclidean topology
    before source edits for the derived proof route.
- Deliverables:
  - Manifold invariance interface module.
- Acceptance criteria:
  - Invariance of domain is not a general homeomorphism axiom.
  - Jordan-Brouwer imports manifold and homology prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Invariance`
  - `rg -n "invariance of domain|Jordan-Brouwer|dimension invariance|TOP-T42" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T43 Add Smooth Manifold And Differential Topology Core

- Status: Pending
- Depends on: `TOP-T41`, analysis derivative/inverse/implicit routes, linear algebra foundations
- Areas: `Proofs.Ai.Topology.Manifold.Smooth`, `Proofs.Ai.Topology.Differential.Sard`
- Tasks:
  - Define smooth manifold, chart compatibility, tangent, smooth map,
    submersion, immersion, and embedding interfaces.
  - Add Sard theorem and regular value theorem routes.
  - Add inverse and implicit function theorem aliases from existing analysis
    modules.
- Deliverables:
  - Smooth manifold and Sard/regular-value modules or statement interfaces.
- Acceptance criteria:
  - Smooth structure is not inferred from topological manifold structure.
  - Sard and regular value statements state smoothness, dimension, regularity,
    compactness, and boundary hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Smooth`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Differential.Sard`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T44 Add Transversality, Morse, And Cobordism Interfaces

- Status: Pending
- Depends on: `TOP-T43`, `TOP-T53`
- Areas: `Proofs.Ai.Topology.Differential.Transversality`, `Proofs.Ai.Topology.Morse`
- Tasks:
  - Add submanifold theorem, transversality theorem, Thom transversality, and
    Ehresmann fibration theorem interfaces.
  - Add Morse lemma, Morse inequalities, and Morse theory route.
  - Add Whitney embedding/approximation, h-cobordism, s-cobordism, smooth
    Poincare, surgery, and Kirby-Siebenmann obstruction interfaces.
- Deliverables:
  - Differential topology advanced interface modules.
- Acceptance criteria:
  - Transversality and Morse statements list smoothness and compactness
    hypotheses explicitly.
  - Surgery and cobordism statements start as late `L2` proof routes only after
    their prerequisites are named and available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Differential.Transversality`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Morse`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T45 Add Surface Classification Interface Route

- Status: Pending
- Depends on: `TOP-T30`, `TOP-T36`, `TOP-T39`, `TOP-T41`
- Areas: `Proofs.Ai.Topology.Surface.Classification`
- Tasks:
  - Add compact surface classification theorem interface.
  - Add orientable and non-orientable closed surface classification, surface
    Euler characteristic formula, surface fundamental group presentations, and
    surface homology computation routes.
  - Record dependencies on CW, fundamental group, homology, and manifold
    prerequisites.
- Deliverables:
  - Surface classification module or interface.
- Acceptance criteria:
  - Surface classification imports fundamental group, homology, and CW
    prerequisites.
  - Surface invariants are not assumed as classification axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Surface.Classification`
  - `rg -n "surface classification|Euler characteristic|TOP-T45" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T46 Add Three-Manifold And Knot Interfaces

- Status: Pending
- Depends on: `TOP-T41`, `TOP-T43`, `TOP-T45`
- Areas: `Proofs.Ai.Topology.ThreeManifold.Interfaces`, `Proofs.Ai.Topology.Knot.Basic`
- Tasks:
  - Add Jordan curve, Schoenflies, Dehn lemma, loop theorem, sphere theorem,
    Kneser-Milnor, JSJ, geometrization, hyperbolization, Mostow rigidity, and
    Dehn surgery interfaces.
  - Add Reidemeister moves, Alexander theorem, Markov theorem, Jones polynomial
    invariance, and knot group interfaces.
  - Split PL/smooth, algebraic, and invariant prerequisites before any
    derived proof attempt.
- Deliverables:
  - Low-dimensional topology interface modules.
- Acceptance criteria:
  - Three-manifold and knot theorems start as `L2` proof routes only after
    manifold, PL/smooth, and algebraic invariants exist; otherwise split
    blockers.
  - Poincare conjecture is not treated as a foundational axiom.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.ThreeManifold.Interfaces`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Knot.Basic`
  - `rg -n "Poincare|JSJ|Jones|TOP-T46" proofs/topology-theorem-proof-roadmap*.md`

### TOP-T47 Add Brouwer And Degree-Style Fixed-Point Core

- Status: Pending
- Depends on: `TOP-T18`, `TOP-T30`, `TOP-T35`, `TOP-T37`, `TOP-T41`
- Areas: `Proofs.Ai.Topology.FixedPoint.Brouwer`, `Proofs.Ai.Topology.FixedPoint.BorsukUlam`
- Tasks:
  - Add Banach fixed point alias from analysis and Brouwer fixed point theorem
    route.
  - Add Borsuk-Ulam theorem and hairy ball theorem routes.
  - Identify whether each theorem uses degree, homology, cohomology, or
    manifold orientation evidence.
- Deliverables:
  - Brouwer and Borsuk-Ulam fixed-point route modules.
- Acceptance criteria:
  - Banach fixed point remains primary in `Proofs.Ai.Analysis.AbstractFixedPoint`.
  - Brouwer and Borsuk-Ulam do not assume degree or homology conclusions as
    input.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FixedPoint.Brouwer`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FixedPoint.BorsukUlam`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T48 Add Lefschetz, Schauder, And Order Fixed-Point Interfaces

- Status: Pending
- Depends on: `TOP-T37`, `TOP-T43`, `TOP-T47`
- Areas: `Proofs.Ai.Topology.FixedPoint.Lefschetz`
- Tasks:
  - Add Schauder fixed point interface and Lefschetz fixed point theorem
    route.
  - Add Poincare-Hopf theorem route.
  - Add Kakutani, Tarski, Knaster-Tarski, Caristi, Markov-Kakutani, Nielsen,
    Eilenberg-Montgomery, and Fan-Browder interfaces.
- Deliverables:
  - Lefschetz and fixed-point interface module.
- Acceptance criteria:
  - Lefschetz imports cohomology or homology prerequisites explicitly.
  - Order-theoretic fixed-point theorems import order/lattice foundations when
    available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FixedPoint.Lefschetz`
  - `rg -n "Schauder|Lefschetz|Knaster|Poincare-Hopf|TOP-T48" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T49 Add Covering And Inductive Dimension Core

- Status: Completed
- Depends on: `TOP-T10`, `TOP-T16`, `TOP-T36`, `TOP-T37`, `TOP-T41`
- Areas: `Proofs.Ai.Topology.Dimension.Covering`, `Proofs.Ai.Topology.Dimension.Inductive`
- Tasks:
  - Define covering dimension, small inductive dimension, and large inductive
    dimension.
  - Add Lebesgue covering dimension theorem, Urysohn dimension theorem, and
    separation characterization route.
  - Add product dimension inequality and compact metric dimension theory.
- Deliverables:
  - Covering and inductive dimension modules.
- Completion notes:
  - Added `CoveringDimensionCore` and `CoveringDimensionTheoremRoute` with
    Lebesgue covering dimension, product inequality, compact metric dimension,
    and cohomology-prerequisite routes.
  - Added `InductiveDimensionRoute` with small and large inductive dimension,
    Urysohn dimension, separation characterization, compact metric inductive
    dimension, and covering/manifold-dimension separation routes.
- Acceptance criteria:
  - Covering dimension is not conflated with vector-space or manifold
    dimension.
  - Cohomological dimension statements import cohomology prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dimension.Covering`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dimension.Inductive`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T50 Add Dimension Invariance And Infinite-Dimensional Interfaces

- Status: Completed
- Depends on: `TOP-T42`, `TOP-T49`
- Areas: `Proofs.Ai.Topology.Dimension.Invariance`
- Tasks:
  - Add Euclidean topological dimension theorem interface and dimension
    invariance/invariance-of-domain aliases.
  - Add Menger-Nobeling, Hurewicz dimension-lowering, Peano continuum
    dimension, Hilbert cube, infinite-dimensional topology, ANR dimension,
    Alexandroff theorem, and Pontryagin surface interfaces.
  - Split cohomological dimension routes by coefficient and cohomology
    prerequisites.
- Deliverables:
  - Dimension invariance module and late dimension interfaces.
- Completed artifacts:
  - Added `Proofs.Ai.Topology.Dimension.Invariance` with
    `DimensionInvarianceRoute`.
  - Added L2 route projections for Euclidean topological dimension,
    invariance-of-domain, dimension invariance, Menger-Nobeling, Hurewicz
    dimension lowering, Peano continuum dimension, Hilbert cube
    infinite-dimensional topology, ANR dimension, coefficient cohomological
    dimension, Alexandroff dimension, and Pontryagin surface evidence.
  - Kept Hilbert cube infinite-dimensional topology behind explicit
    `HilbertCubeModelEvidence` and coefficient cohomological dimension behind
    explicit coefficient-group and cohomology-prerequisite evidence.
- Acceptance criteria:
  - Invariance of domain remains primary in manifold/dimension route, not a
    general homeomorphism axiom.
  - Infinite-dimensional examples start as `L2` proof routes only after model
    spaces exist; otherwise split blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dimension.Invariance`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Dimension.Invariance --verified-cache authoring`
  - `rg -n "Menger|Hilbert cube|Pontryagin|TOP-T50" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T51 Add Topological Dynamics Core

- Status: Pending
- Depends on: `TOP-T05`, `TOP-T10`, `TOP-T14`, `TOP-T18`, `TOP-T28`
- Areas: `Proofs.Ai.Topology.Dynamics.Basic`
- Tasks:
  - Define topological dynamical systems and topological conjugacy.
  - Prove orbit closure properties, minimal set existence route,
    transitivity, mixing characterizations, and Brouwer translation theorem
    interface.
  - Add Birkhoff recurrence and Lefschetz fixed point aliases with explicit
    prerequisites.
- Deliverables:
  - Topological dynamics base module.
- Acceptance criteria:
  - Measure recurrence does not land before measure/probability foundations.
  - Fixed-point aliases import `TOP-T48` rather than duplicate Lefschetz.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dynamics.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T52 Add Symbolic, Measure, And Stability Dynamics Interfaces

- Status: Pending
- Depends on: `TOP-T20`, `TOP-T43`, `TOP-T51`, `MEA-T51`, `STAT-T55` through `STAT-T57`
- Areas: `Proofs.Ai.Topology.Dynamics.Symbolic`, `Proofs.Ai.Topology.Dynamics.Stability`
- Tasks:
  - Add symbolic dynamics and shift-space properties using product topology.
  - Add Poincare recurrence alias from measure theory only after measure or
    probability-process foundations exist.
  - Add Smale horseshoe, Sharkovsky, Conley index, structural stability,
    shadowing, stable manifold, Hartman-Grobman, and Morse-Smale interfaces.
- Deliverables:
  - Symbolic and stability dynamics interface modules.
- Acceptance criteria:
  - Poincare recurrence aliases wait for `MEA-T51`, which refines `ANA-T24`
    through `ANA-T26`, or probability/process routes `STAT-T55` through
    `STAT-T57`.
  - Stable manifold and Hartman-Grobman routes import differential and ODE
    prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dynamics.Symbolic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dynamics.Stability`
  - `rg -n "Poincare recurrence|MEA-T51|Hartman|Morse-Smale|Dynamics" proofs/topology-theorem-proof-roadmap*.md proofs/measure-theory-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md proofs/statistics-theorem-proof-roadmap*.md`

### TOP-T53 Add Stokes And De Rham Interfaces

- Status: Pending
- Depends on: `TOP-T37`, `TOP-T41`, `TOP-T43`, `ANA-T16`, `ANA-T18`, `MEA-T19` through `MEA-T25`
- Areas: `Proofs.Ai.Topology.DifferentialForms.Stokes`, `Proofs.Ai.Topology.DeRham`
- Tasks:
  - Add Stokes theorem and de Rham theorem interfaces.
  - Add Gauss-Bonnet and Chern-Gauss-Bonnet interfaces.
  - Record Riemann/Lebesgue integration, differential-form, orientation, and
    smooth-manifold prerequisites before derived source work.
- Deliverables:
  - Stokes and de Rham interface modules.
- Acceptance criteria:
  - Differential-form integration is not assumed before analysis integration
    and manifold orientation foundations.
  - Stokes and de Rham routes wait for analysis milestones `ANA-T16` through
    `ANA-T18` and the measure roadmap's integration route `MEA-T19` through
    `MEA-T25`, which refines `ANA-T24` through `ANA-T26`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.DifferentialForms.Stokes`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.DeRham`
  - `rg -n "Stokes|de Rham|MEA-T19|MEA-T25" proofs/topology-theorem-proof-roadmap*.md proofs/measure-theory-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T54 Add Characteristic Classes And Index-Theory Interfaces

- Status: Pending
- Depends on: `TOP-T38`, `TOP-T48`, `TOP-T53`, `TOP-T55`
- Areas: `Proofs.Ai.Topology.CharacteristicClass`, `Proofs.Ai.Topology.IndexTheory.Interfaces`
- Tasks:
  - Add Chern-Weil theory and Chern, Euler, Pontryagin, and
    Stiefel-Whitney class theorem interfaces.
  - Add Poincare-Hopf alias, Hodge decomposition, Hodge theorem,
    Riemann-Roch, Hirzebruch-Riemann-Roch, Thom isomorphism,
    Pontryagin-Thom construction, cobordism ring, and Atiyah-Singer index
    theorem interfaces.
  - Add Bott periodicity alias from `TOP-T55` when K-theory exists.
- Deliverables:
  - Characteristic-class and index-theory interface modules.
- Acceptance criteria:
  - Characteristic classes state bundle, coefficient, naturality, and
    obstruction-theory assumptions.
  - Index and Riemann-Roch theorems start as `L2` proof routes only after
    analytic and K-theory prerequisites exist; otherwise split blockers.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CharacteristicClass`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.IndexTheory.Interfaces`
  - `rg -n "Atiyah-Singer|Chern|Pontryagin|Riemann-Roch|TOP-T54" proofs/topology-theorem-proof-roadmap*.md`

### TOP-T55 Add K-Theory And Spectral Sequence Core Interfaces

- Status: Pending
- Depends on: `TOP-T36`, `TOP-T38`, `TOP-T39`, `TOP-T53`, category theory
- Areas: `Proofs.Ai.Topology.KTheory.Basic`, `Proofs.Ai.Topology.SpectralSequence.Basic`
- Tasks:
  - Add vector bundle classification, clutching construction, K-theory
    definition, Bott periodicity, and Thom isomorphism in K-theory interfaces.
  - Add Atiyah-Hirzebruch, Serre, Adams, and Eilenberg-Moore spectral sequence
    interfaces.
  - Define filtration, exact couple, convergence, and grading statement
    shapes before any spectral sequence proof attempt.
- Deliverables:
  - K-theory and spectral sequence interface modules.
- Acceptance criteria:
  - K-theory does not assume vector bundle classification without bundle
    infrastructure.
  - Spectral sequences state filtration, exact couple, convergence, and
    grading assumptions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.KTheory.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.SpectralSequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### TOP-T56 Add Stable Homotopy And Representation Interfaces

- Status: Pending
- Depends on: `TOP-T29`, `TOP-T55`
- Areas: `Proofs.Ai.Topology.StableHomotopy.Interfaces`
- Tasks:
  - Add Brown representation theorem, stable homotopy category, Freudenthal
    suspension, stable Hurewicz theorem, Spanier-Whitehead duality, Postnikov
    and Whitehead tower, obstruction theory, spectral sequence convergence,
    Eilenberg-Mac Lane spaces, and homotopy groups of spheres interfaces.
  - Split stable homotopy, spectra, and category dependencies before source
    edits.
  - Prevent stable interfaces from entering earlier theorem proofs as axioms.
- Deliverables:
  - Stable homotopy and representation interface module.
- Acceptance criteria:
  - Stable homotopy interfaces do not enter earlier theorem proofs as axioms.
  - Spectral sequence convergence statements import `TOP-T55` assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.StableHomotopy.Interfaces`
  - `rg -n "Brown representation|stable Hurewicz|Postnikov|Eilenberg-Mac Lane|TOP-T56" proofs/topology-theorem-proof-roadmap*.md`
  - `git diff --check`

### TOP-T57 Package And Promote Stable Topology Closures

- Status: Pending
- Depends on: any completed stable theorem batch from `TOP-T01` through `TOP-T56`
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/*`, `develop/npa-mathlib-next-closure-roadmap.md`
- Tasks:
  - Run closure audit for each stable topology module cluster.
  - Update theorem indexes, axiom reports, package metadata, and publish-plan
    entries only when closure is clean.
  - Materialize accepted topology clusters into `npa-mathlib` with public
    documentation of included and excluded theorem families.
- Deliverables:
  - Closure audit notes, package metadata updates, and public promotion plan
    for stable topology modules.
- Acceptance criteria:
  - Axiom report does not gain unintended axioms.
  - Source-free verifier and package checks pass for the promoted closure.
  - Public closure documentation states which theorem families are included
    and excluded.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## Review Checklist

- Every `TOP-00` through `TOP-29` roadmap milestone is covered by at least one
  `TOP-T*` task.
- Every `TOQ-001` through `TOQ-020` recommended queue item maps to a concrete
  task milestone.
- Every task has dependencies, deliverables, acceptance criteria, and
  verification commands or targeted review searches.
- Tasks that require choice, ultrafilters, maximal constructions, manifolds,
  smoothness, measure, integration, algebraic invariants, or spectral
  machinery keep those prerequisites explicit.
- Analysis aliases remain aligned with `ANA-T22`, `ANA-T23`, and `ANA-T27`,
  while topology owns general-topology theorem cards.
- Measure-dependent topology aliases remain aligned with `MEA-T19` through
  `MEA-T25` and `MEA-T51`, while `ANA-T24` through `ANA-T26` stay coarse
  analysis milestones.
- Late low-dimensional, geometric, K-theory, spectral sequence, and stable
  homotopy statements remain interfaces until their prerequisite clusters are
  certified.

## Decision Points

- Decide the first general `TopologySpace` law-package shape before
  `TOP-T01`, including whether open sets or closure operators are primary.
- Decide how `Proofs.Ai.Analysis.AbstractMetricTopology` embeds into the
  general topology vocabulary before `TOP-T12`.
- Decide the set-theoretic evidence strategy for ultrafilters, Tychonoff,
  Stone-Cech, and paracompactness before `TOP-T11`, `TOP-T25`, or `TOP-T27`
  start their derived proof routes.
- Decide coefficient groups, chain-complex representation, grading, and
  exactness evidence before `TOP-T34` through `TOP-T38`.
- Decide the model of simplicial complexes and CW complexes before
  `TOP-T39` and `TOP-T40`.
- Decide topological versus smooth manifold namespace boundaries before
  `TOP-T43` and `TOP-T44`.
- Before any `L3` promotion, run closure audit and choose package gates
  according to changed artifacts.
