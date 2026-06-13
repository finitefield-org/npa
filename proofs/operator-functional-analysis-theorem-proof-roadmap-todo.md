# Operator And Functional Analysis Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T07`)
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/measure-theory-theorem-proof-roadmap-todo.md`
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for operator algebras and
advanced functional analysis. It is a planning sidecar only: it does not add
trusted proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers Banach algebra law packages, C-star algebra interfaces,
bounded and compact operators, spectral calculus, von Neumann algebra routes,
locally convex spaces, distributions, unbounded operators, semigroup routes,
operator ideals, harmonic/PDE/quantum bridges, and closure-boundary planning.

Out of scope for this task document:

- adding Banach spaces, Hilbert spaces, operator algebras, spectra,
  distributions, or von Neumann algebras as trusted kernel primitives;
- treating spectral theorem, Hahn-Banach, compactness, Riesz representation,
  or functional calculus statements as proof evidence without certificates;
- duplicating the analysis roadmap's Banach, Hilbert, weak-topology, or
  spectral theorem ownership;
- hiding completeness, choice, topology, measure, boundedness, domain, or
  closure assumptions inside operator law packages;
- publicly materializing operator modules before closure audit and package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.Operator.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.FunctionalAnalysis.Operator.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing functional-analysis modules include
  `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem`,
  `Proofs.Ai.FunctionalAnalysis.Banach`,
  `Proofs.Ai.FunctionalAnalysis.Hilbert`,
  `Proofs.Ai.FunctionalAnalysis.Spectral`, and
  `Proofs.Ai.FunctionalAnalysis.WeakTopology`.
- The analysis roadmap already owns `ANA-T27` Banach-space functional
  analysis and `ANA-T28` Hilbert, weak-topology, and spectral routes.
- Measure theory owns measurable spaces, integration, Lp-style prerequisites,
  and convergence machinery used by functional analysis.
- Linear algebra owns finite-dimensional operator and spectral prerequisites.
- This todo owns the advanced operator-algebra and distribution roadmap layer
  that imports those foundations.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `OFA-00` inventory and namespace contract | `OFA-T00` |
| `OFA-01` Banach algebra core | `OFA-T01` |
| `OFA-02` C-star algebra route | `OFA-T02` |
| `OFA-03` bounded and compact operators | `OFA-T03` |
| `OFA-04` spectral calculus | `OFA-T04` |
| `OFA-05` von Neumann algebra route | `OFA-T05` |
| `OFA-06` locally convex spaces | `OFA-T06` |
| `OFA-07` distributions and test functions | `OFA-T07` |
| `OFA-08` unbounded operators | `OFA-T08` |
| `OFA-09` semigroups and evolution equations | `OFA-T09` |
| `OFA-10` operator ideals and trace routes | `OFA-T10` |
| `OFA-11` bridge aliases | `OFA-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `OFA-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `OFA-T01` through `OFA-T03` | `L2` for algebraic and bounded-operator law projections where prerequisites exist |
| `OFA-T04` through `OFA-T05` | route packages first unless spectral and measure prerequisites are explicit |
| `OFA-T06` through `OFA-T11` | dependency maps or `L2` only for explicit structural lemmas |

## Milestones

### OFA-T00 Build Operator Functional Analysis Inventory

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: None
- Areas: `proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory Banach algebra, C-star, bounded operator, compact operator,
    spectral calculus, von Neumann, locally convex, distribution, unbounded
    operator, semigroup, and bridge theorem families.
  - Assign primary homes across analysis, measure theory, linear algebra,
    mathematical physics, and operator functional analysis.
  - Mark completeness, domain, boundedness, topology, measure, and choice
    assumptions.
- Verification:
  - `rg -n "OFA-T00|Banach algebra|C-star|compact operator|spectral calculus" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T01 Add Banach Algebra Core

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T27`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.BanachAlgebra`
- Tasks:
  - Define Banach algebra law packages, multiplicative norms, units,
    invertibility, ideals, and spectrum route packages.
  - Prove elementary norm and inverse algebra lemmas where completeness and
    boundedness prerequisites are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.Operator.BanachAlgebra`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.FunctionalAnalysis.Operator.BanachAlgebra --verified-cache authoring`

### OFA-T02 Add C-Star Algebra Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T01`, `ANA-T28`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.CStarAlgebra`
- Tasks:
  - Define involutive Banach algebra and C-star identity interfaces with
    explicit norm and adjoint assumptions.
  - Split Gelfand-Naimark and state-space representation theorems behind
    separate prerequisites.
- Verification:
  - `rg -n "OFA-T02|C-star|Gelfand|involution" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T03 Add Bounded And Compact Operator Core

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T28`, `LIN-T26`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.Compact`
- Tasks:
  - Define bounded operators, adjoints, compact operators, finite-rank
    operators, and invariant subspace route packages.
  - Prove composition and norm-bound projection lemmas where prerequisites
    exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.FunctionalAnalysis.Operator.Compact`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### OFA-T04 Add Spectral Calculus Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T03`, `ANA-T28`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.SpectralCalculus`
- Tasks:
  - Audit existing spectral modules and define functional-calculus route
    packages with explicit normality, self-adjointness, and measure
    prerequisites.
  - Keep spectral theorem uses as imports from the analysis roadmap.
- Verification:
  - `rg -n "OFA-T04|spectral calculus|self-adjoint|normal operator|ANA-T28" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md proofs/analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T05 Add Von Neumann Algebra Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T02`, `OFA-T04`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.VonNeumann`
- Tasks:
  - Define weak operator topology, commutants, projections, factors, and
    trace route packages with explicit topology and measure assumptions.
  - Split bicommutant and direct-integral results until the prerequisites are
    proven.
- Verification:
  - `rg -n "OFA-T05|von Neumann|commutant|bicommutant|trace" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T06 Add Locally Convex Space Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T27`, `TOP-T11`
- Areas: `Proofs.Ai.FunctionalAnalysis.LocallyConvex`
- Tasks:
  - Define seminorm families, locally convex topologies, duals, weak
    topologies, and barrelled/nuclear route packages.
  - Keep Hahn-Banach, separation, and compactness assumptions explicit.
- Verification:
  - `rg -n "OFA-T06|locally convex|seminorm|Hahn-Banach" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T07 Add Distributions And Test Functions Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T06`, `ANA-T31`
- Areas: `Proofs.Ai.Analysis.Distribution`
- Tasks:
  - Define test functions, distributions, weak derivatives, and distribution
    convergence route packages.
  - Coordinate PDE use sites with the analysis roadmap instead of duplicating
    PDE theorem ownership.
- Verification:
  - `rg -n "OFA-T07|distribution|test function|weak derivative" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T08 Add Unbounded Operator Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T03`, `ANA-T28`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.Unbounded`
- Tasks:
  - Define densely defined, closed, closable, symmetric, self-adjoint, and
    essential self-adjoint route packages with explicit domains.
  - Split spectral and Stone theorem uses behind prerequisite tasks.
- Verification:
  - `rg -n "OFA-T08|unbounded operator|domain|self-adjoint|closable" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T09 Add Semigroup And Evolution Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T08`, `ANA-T33`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.Semigroup`
- Tasks:
  - Define strongly continuous semigroups, generators, resolvents, and
    evolution-equation route packages.
  - Split Hille-Yosida and Stone routes until topology and generator
    prerequisites are explicit.
- Verification:
  - `rg -n "OFA-T09|semigroup|generator|Hille-Yosida|Stone" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T10 Add Operator Ideals And Trace Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `OFA-T03`, `OFA-T05`
- Areas: `Proofs.Ai.FunctionalAnalysis.Operator.Ideal`
- Tasks:
  - Define Schatten-style, trace-class, Hilbert-Schmidt, and compact-ideal
    route packages with explicit Hilbert-space prerequisites.
  - Keep determinant and zeta-regularized routes outside core evidence.
- Verification:
  - `rg -n "OFA-T10|operator ideal|trace-class|Hilbert-Schmidt|Schatten" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OFA-T11 Coordinate Bridge Aliases

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: `OFA-T04`, `OFA-T07`, `MP-T05`
- Areas: `proofs/mathematical-physics-theorem-proof-roadmap-todo.md`,
  `proofs/analysis-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map harmonic analysis, PDE, quantum formalism, and mathematical physics
    uses to primary operator or analysis milestones.
  - Keep physical postulates out of mathematical proof evidence.
- Verification:
  - `rg -n "OFA-T11|MP-T05|quantum|PDE|operator" proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `OFAQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `OFA-T00` |
| `OFAQ-002` | Banach algebra core | `L2` where prerequisites exist | `OFA-T01` |
| `OFAQ-003` | bounded and compact operator core | `L2` | `OFA-T03` |
| `OFAQ-004` | spectral calculus audit | route package first | `OFA-T04` |
| `OFAQ-005` | distribution route dependency split | route package first | `OFA-T07` |

## Review Checklist

- The analysis roadmap remains primary for Banach, Hilbert, weak-topology, and
  spectral theorem foundations already assigned there.
- Completeness, topology, measure, domain, boundedness, and choice assumptions
  are visible.
- Spectral calculus, von Neumann, distribution, and semigroup statements do
  not assume the conclusion through interfaces.
- Mathematical physics aliases stay assumption-explicit.
- Public package work is outside this TODO until closure audit confirms stable `L2` derived
  certificates.
