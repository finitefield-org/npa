# Representation And Lie Theory Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T06`)
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/differential-geometry-theorem-proof-roadmap-todo.md`
- `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
- `proofs/representation-lie-theory-theorem-cards.md`

This task breakdown is the first dedicated todo for representation theory and
Lie theory. It is a planning sidecar only: it does not add trusted proof
evidence, axioms, source-free certificate verdicts, or package verification
claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers finite-group representations, group algebras, characters,
orthogonality relations, Maschke-style semisimplicity, modules over algebras,
Lie algebras, universal enveloping algebra route packages, PBW routes, Lie
groups and Lie algebras, compact Lie representation routes, algebraic group
representations, harmonic-analysis aliases, modular and Langlands aliases, and
closure-boundary planning.

Out of scope for this task document:

- adding groups, representations, Lie groups, manifolds, Haar measure, or
  algebraic groups as trusted kernel primitives;
- treating character theory, semisimplicity, PBW, Peter-Weyl, or Langlands
  interfaces as proof evidence before lower-level certificates exist;
- duplicating linear algebra, group theory, differential geometry, functional
  analysis, or arithmetic-geometry theorem ownership;
- hiding choice, finite-dimensionality, algebraic-closedness, compactness,
  smoothness, or measure assumptions in law packages;
- publicly materializing representation modules before closure audit and package checks are
  clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.RepresentationTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.RepresentationTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- No broad `Proofs.Ai.RepresentationTheory.*` or `Proofs.Ai.LieTheory.*`
  roadmap file is currently present.
- Linear algebra already covers vector spaces, linear maps, matrices,
  eigenvalue-oriented routes, tensor/exterior algebra, and some
  representation-shaped algebraic prerequisites.
- Differential geometry owns smooth manifolds, vector fields, flows, Lie
  brackets on manifolds, and Lie-group analytic prerequisites.
- Functional analysis owns Hilbert-space and spectral-analysis prerequisites
  needed by unitary and compact-group routes.
- Arithmetic geometry and number theory consume representation-theoretic
  aliases but should not own the foundational representation proofs.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `RLT-00` inventory and namespace contract | `RLT-T00` |
| `RLT-01` finite-group representation core | `RLT-T01` |
| `RLT-02` group algebras and module routes | `RLT-T02` |
| `RLT-03` characters and orthogonality | `RLT-T03` |
| `RLT-04` Maschke and semisimplicity routes | `RLT-T04` |
| `RLT-05` Lie algebra core | `RLT-T05` |
| `RLT-06` universal enveloping algebra and PBW route | `RLT-T06` |
| `RLT-07` Lie groups and Lie algebra bridge | `RLT-T07` |
| `RLT-08` compact Lie and unitary representation routes | `RLT-T08` |
| `RLT-09` algebraic group representation routes | `RLT-T09` |
| `RLT-10` harmonic, modular, and Langlands aliases | `RLT-T10` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `RLT-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `RLT-T01` through `RLT-T04` | `L2` derived certificates for finite algebraic statements where prerequisites exist |
| `RLT-T05` through `RLT-T06` | `L2` for Lie algebra law projections; split PBW construction prerequisites before source edits |
| `RLT-T07` through `RLT-T10` | route packages or `L2` only for explicit finite-dimensional structural lemmas |

## Milestones

### RLT-T00 Build Representation And Lie Theory Inventory

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: None
- Areas: `proofs/representation-lie-theory-theorem-proof-roadmap-todo.md`,
  `proofs/representation-lie-theory-theorem-cards.md`
- Tasks:
  - Inventory finite-group, group-algebra, character, Lie algebra, Lie group,
    compact-group, algebraic-group, and arithmetic-facing theorem families.
  - Assign primary homes across representation theory, linear algebra,
    differential geometry, functional analysis, and arithmetic geometry.
  - Mark finite-dimensionality, algebraic-closedness, compactness, smoothness,
    Haar-measure, and choice assumptions.
- Deliverables:
  - `proofs/representation-lie-theory-theorem-cards.md` theorem-card inventory,
    checked route certificate register, duplicate-owner map, and assumption
    taxonomy.
- Acceptance criteria:
  - Every `RLT-T*` milestone has a theorem card or an intentionally grouped
    alias card.
  - L2 route certificates are indexed by module, theorem declaration, and
    certificate path without treating this sidecar as proof evidence.
  - Linear algebra, differential geometry, functional analysis, measure,
    algebraic geometry, arithmetic geometry, and Langlands ownership boundaries
    remain explicit.
- Verification:
  - `rg -n "RLT-T00|representation|character|Lie|Maschke" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md proofs/representation-lie-theory-theorem-cards.md`
  - `git diff --check`

### RLT-T01 Add Finite-Group Representation Core

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T00`, `LIN-T05`
- Areas: `Proofs.Ai.RepresentationTheory.FiniteGroup`
- Tasks:
  - Define finite-group actions by linear automorphisms on explicit vector
    spaces.
  - Prove identity, multiplication, invariant subspace, and direct-sum
    projection lemmas from law packages.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.RepresentationTheory.FiniteGroup`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.RepresentationTheory.FiniteGroup --verified-cache authoring`

### RLT-T02 Add Group Algebra And Module Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T01`, `LIN-T38`
- Areas: `Proofs.Ai.RepresentationTheory.GroupAlgebra`
- Tasks:
  - Define group algebra action route packages and the equivalence between
    representations and suitable modules.
  - Keep ring and module assumptions imported rather than restated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.RepresentationTheory.GroupAlgebra`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### RLT-T03 Add Character And Orthogonality Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T01`, `LIN-T26`
- Areas: `Proofs.Ai.RepresentationTheory.Character`
- Tasks:
  - Define traces, characters, class functions, inner products, and
    irreducibility route packages.
  - Split orthogonality behind finite-dimensional trace and finite group
    averaging prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.RepresentationTheory.Character`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.RepresentationTheory.Character --verified-cache authoring`

### RLT-T04 Add Maschke And Semisimplicity Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T01`, `RLT-T03`
- Areas: `Proofs.Ai.RepresentationTheory.Semisimple`
- Tasks:
  - State finite-group semisimplicity with explicit field characteristic and
    group-order invertibility assumptions.
  - Prove projection and complement lemmas where averaging evidence exists.
- Verification:
  - `rg -n "RLT-T04|Maschke|semisimple|characteristic" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### RLT-T05 Add Lie Algebra Core

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `LIN-T38`
- Areas: `Proofs.Ai.LieTheory.LieAlgebra`
- Tasks:
  - Define Lie algebra law packages, homomorphisms, ideals, subalgebras, and
    representation actions by derivations.
  - Prove antisymmetry and Jacobi-derived structural lemmas when explicit
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LieTheory.LieAlgebra`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LieTheory.LieAlgebra --verified-cache authoring`

### RLT-T06 Add Universal Enveloping Algebra And PBW Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T05`, `CMA-T06`
- Areas: `Proofs.Ai.LieTheory.EnvelopingAlgebra`
- Tasks:
  - Define universal enveloping algebra interfaces with explicit tensor and
    quotient assumptions.
  - Keep PBW as a dependency-split route until the filtration and basis
    machinery is proven.
- Verification:
  - `rg -n "RLT-T06|PBW|enveloping|filtration" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### RLT-T07 Add Lie Group And Lie Algebra Bridge

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T05`, `DG-T04`
- Areas: `Proofs.Ai.LieTheory.LieGroup`
- Tasks:
  - Coordinate smooth-group law packages with differential geometry ownership.
  - Prove only bridge lemmas whose smooth, tangent, and flow prerequisites are
    explicit.
- Verification:
  - `rg -n "RLT-T07|Lie group|DG-T04|tangent" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md proofs/differential-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### RLT-T08 Add Compact Lie And Unitary Representation Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T07`, `ANA-T28`
- Areas: `Proofs.Ai.RepresentationTheory.CompactLie`
- Tasks:
  - Record compact-group unitary representation, Haar averaging, and
    Peter-Weyl prerequisites explicitly.
  - Avoid using Haar measure or Peter-Weyl as an unproved theorem-shaped
    shortcut.
- Verification:
  - `rg -n "RLT-T08|compact Lie|Peter-Weyl|Haar" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### RLT-T09 Add Algebraic Group Representation Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `RLT-T05`, `AG-T03`
- Areas: `Proofs.Ai.RepresentationTheory.AlgebraicGroup`
- Tasks:
  - Define algebraic group representation route packages over explicit
    algebraic-geometry prerequisites.
  - Split reductive, highest-weight, and category O routes behind separate
    theorem cards.
- Verification:
  - `rg -n "RLT-T09|algebraic group|highest-weight|AG-T03" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### RLT-T10 Coordinate Harmonic, Modular, And Langlands Aliases

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: `RLT-T03`, `RLT-T08`, `AGL-T10`
- Areas: `Proofs.Ai.ArithmeticGeometry.*`, `Proofs.Ai.Analysis.*`
- Tasks:
  - Map harmonic-analysis, automorphic, modular-form, and Langlands uses to
    primary representation milestones.
  - Keep arithmetic consequences in the arithmetic-geometry roadmap.
- Verification:
  - `rg -n "RLT-T10|Langlands|automorphic|modular" proofs/representation-lie-theory-theorem-proof-roadmap-todo.md proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `RLTQ-001` | `proofs/representation-lie-theory-theorem-cards.md` theorem-card inventory and duplicate-owner map | `L0` | `RLT-T00` |
| `RLTQ-002` | finite-group representation core | `L2` | `RLT-T01` |
| `RLTQ-003` | character and trace route | `L2` where prerequisites exist | `RLT-T03` |
| `RLTQ-004` | Lie algebra law package | `L2` | `RLT-T05` |
| `RLTQ-005` | Lie group bridge dependency split | route package first | `RLT-T07` |

## Review Checklist

- Finite algebraic representation results are separated from analytic Lie
  group routes.
- Linear algebra, differential geometry, functional analysis, and arithmetic
  geometry dependencies remain imported or aliased rather than duplicated.
- Field characteristic, finite-dimensionality, compactness, smoothness, Haar,
  and choice assumptions are visible.
- PBW, Peter-Weyl, and Langlands-facing statements are not used as evidence
  before lower-level proof certificates exist.
- Public package work is outside this TODO until closure audit confirms stable `L2` derived
  certificates.
