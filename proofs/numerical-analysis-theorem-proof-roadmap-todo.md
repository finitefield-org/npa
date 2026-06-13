# Numerical Analysis Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T09`)
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/optimization-theorem-proof-roadmap-todo.md`
- `proofs/statistics-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for numerical analysis and
approximation theory. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers numerical models, roundoff assumptions, interpolation,
approximation, quadrature, root finding, ODE methods, PDE and finite-element
routes, iterative linear solvers, conditioning, stability, optimization
algorithms, randomized numerical methods, algorithm trace correctness, and
closure-boundary planning.

Out of scope for this task document:

- adding floating point, real numbers, norms, matrices, differential
  equations, or algorithms as trusted kernel primitives;
- treating convergence or stability statements as proof evidence without
  explicit recurrence, norm, and error-bound certificates;
- duplicating linear algebra numerical milestones, analysis ODE/PDE
  milestones, or optimization algorithm milestones;
- hiding discretization, mesh regularity, smoothness, conditioning, machine
  arithmetic, or probability assumptions in route packages;
- publicly materializing numerical modules before closure audit and package checks are
  clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumericalAnalysis.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumericalAnalysis.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Linear algebra already records numerical iteration and Krylov recurrence
  plans in `LIN-T52` and stability or randomized numerical interfaces in
  `LIN-T53`.
- Analysis already owns calculus, sequences, Fourier, ODE, PDE, Sobolev,
  variational, convex, and transform foundations.
- Optimization owns convex optimization, LP duality, KKT, algorithm
  correctness, dynamic programming, and minimax route planning.
- Statistics owns randomized concentration, stochastic approximation, and
  computation-dependent learning aliases.
- This todo owns the numerical-analysis execution order that imports those
  foundations and keeps machine or discretization assumptions visible.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `NUM-00` inventory and namespace contract | `NUM-T00` |
| `NUM-01` numerical model and roundoff assumptions | `NUM-T01` |
| `NUM-02` interpolation and polynomial approximation | `NUM-T02` |
| `NUM-03` quadrature and integration error bounds | `NUM-T03` |
| `NUM-04` root finding and fixed-point iterations | `NUM-T04` |
| `NUM-05` ODE numerical methods | `NUM-T05` |
| `NUM-06` PDE and finite-element route | `NUM-T06` |
| `NUM-07` iterative linear solvers and Krylov aliases | `NUM-T07` |
| `NUM-08` conditioning, stability, and backward error | `NUM-T08` |
| `NUM-09` optimization algorithm bridges | `NUM-T09` |
| `NUM-10` randomized numerical methods | `NUM-T10` |
| `NUM-11` algorithm trace correctness schemas | `NUM-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `NUM-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `NUM-T01` through `NUM-T04` | `L2` for finite recurrence and deterministic error-bound theorems |
| `NUM-T05` through `NUM-T08` | `L2` where analysis and linear-algebra prerequisites exist; otherwise split blockers |
| `NUM-T09` through `NUM-T11` | trace-correctness certificates or assumption-explicit route packages |

## Milestones

### NUM-T00 Build Numerical Analysis Inventory

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: None
- Areas: `proofs/numerical-analysis-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory roundoff, interpolation, quadrature, root finding, ODE, PDE,
    finite element, Krylov, conditioning, optimization, randomized numerical,
    and trace-correctness theorem families.
  - Assign primary homes across numerical analysis, linear algebra, analysis,
    optimization, statistics, and theoretical computer science.
  - Mark smoothness, mesh, stability, conditioning, arithmetic, and
    probability assumptions.
- Verification:
  - `rg -n "NUM-T00|roundoff|interpolation|quadrature|Krylov" proofs/numerical-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T01 Add Numerical Model And Roundoff Boundary

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `NUM-T00`
- Areas: `Proofs.Ai.NumericalAnalysis.Model`
- Tasks:
  - Define exact arithmetic, floating or rounded arithmetic assumptions,
    machine epsilon route packages, and finite trace models.
  - Separate machine-model assumptions from mathematical convergence
    theorems.
- Verification:
  - `rg -n "NUM-T01|roundoff|machine epsilon|floating" proofs/numerical-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T02 Add Interpolation And Polynomial Approximation Core

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T13`, `LIN-T05`
- Areas: `Proofs.Ai.NumericalAnalysis.Approximation`
- Tasks:
  - Define interpolation nodes, polynomial interpolants, error formulas, and
    best-approximation route packages.
  - Split Weierstrass, Stone-Weierstrass, and Chebyshev routes behind
    topology and analysis prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumericalAnalysis.Approximation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumericalAnalysis.Approximation --verified-cache authoring`

### NUM-T03 Add Quadrature And Integration Error Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T15`, `NUM-T02`
- Areas: `Proofs.Ai.NumericalAnalysis.Quadrature`
- Tasks:
  - Define quadrature rules, exactness degree, composite rules, and finite
    error-bound recurrence packages.
  - Keep measure-theoretic integration and adaptive quadrature assumptions
    explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumericalAnalysis.Quadrature`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### NUM-T04 Add Root Finding And Fixed-Point Iteration Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T24`, `NUM-T01`
- Areas: `Proofs.Ai.NumericalAnalysis.RootFinding`
- Tasks:
  - Define bisection, Newton, secant, contraction, and residual recurrence
    route packages.
  - Prove finite deterministic error-update lemmas where assumptions are
    explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumericalAnalysis.RootFinding`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumericalAnalysis.RootFinding --verified-cache authoring`

### NUM-T05 Add ODE Numerical Method Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T33`, `NUM-T04`
- Areas: `Proofs.Ai.NumericalAnalysis.ODE`
- Tasks:
  - Define Euler, Runge-Kutta, consistency, stability, and convergence route
    packages with explicit Lipschitz and local-error assumptions.
  - Split existence and uniqueness imports to analysis.
- Verification:
  - `rg -n "NUM-T05|Euler|Runge-Kutta|Lipschitz|local error" proofs/numerical-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T06 Add PDE And Finite-Element Route

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `ANA-T35`, `NUM-T02`
- Areas: `Proofs.Ai.NumericalAnalysis.FEM`
- Tasks:
  - Define mesh, finite-element space, weak form, consistency, stability, and
    Galerkin route packages.
  - Split Sobolev, compactness, and elliptic regularity prerequisites before
    source edits.
- Verification:
  - `rg -n "NUM-T06|finite element|Galerkin|Sobolev|mesh" proofs/numerical-analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T07 Coordinate Iterative Linear Solver Aliases

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: `LIN-T52`
- Areas: `Proofs.Ai.LinearAlgebra.Numerical.*`
- Tasks:
  - Map Jacobi, Gauss-Seidel, conjugate-gradient, GMRES, and Krylov theorem
    families to their primary linear algebra milestones.
  - Add numerical-analysis aliases only for error-model and convergence
    orchestration.
- Verification:
  - `rg -n "NUM-T07|LIN-T52|Krylov|conjugate-gradient" proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/linear-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T08 Add Conditioning, Stability, And Backward Error Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `NUM-T01`, `LIN-T53`
- Areas: `Proofs.Ai.NumericalAnalysis.Stability`
- Tasks:
  - Define condition numbers, forward error, backward error, stable
    algorithms, and perturbation route packages.
  - Keep probabilistic or randomized stability assumptions explicit.
- Verification:
  - `rg -n "NUM-T08|conditioning|backward error|LIN-T53|stability" proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/linear-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T09 Coordinate Optimization Algorithm Bridges

- Status: Completed (2026-06-13; roadmap inventory or alias split recorded; no promotion)
- Depends on: `OPT-T07`, `NUM-T04`
- Areas: `Proofs.Ai.Optimization.*`
- Tasks:
  - Map gradient, Newton, proximal, interior-point, and dynamic-programming
    numerical algorithms to optimization ownership.
  - Keep trace correctness and convergence assumptions explicit.
- Verification:
  - `rg -n "NUM-T09|OPT-T07|Newton|proximal|interior-point" proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/optimization-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T10 Add Randomized Numerical Method Routes

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `STAT-T11`, `LIN-T53`
- Areas: `Proofs.Ai.NumericalAnalysis.Randomized`
- Tasks:
  - Define randomized sketches, Monte Carlo error, randomized linear algebra,
    and probabilistic error-bound route packages.
  - Split concentration and martingale assumptions to statistics.
- Verification:
  - `rg -n "NUM-T10|randomized|Monte Carlo|sketch|STAT-T11" proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### NUM-T11 Add Algorithm Trace Correctness Schemas

- Status: Completed (2026-06-13; L2 route certificate; no promotion)
- Depends on: `NUM-T01`, `TCS-T07`
- Areas: `Proofs.Ai.NumericalAnalysis.Trace`
- Tasks:
  - Define finite trace schemas for numerical algorithms, loop invariants,
    and postcondition certificates.
  - Coordinate general semantics and complexity models with theoretical
    computer science.
- Verification:
  - `rg -n "NUM-T11|TCS-T07|trace|loop invariant" proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `NUMQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `NUM-T00` |
| `NUMQ-002` | roundoff and numerical model boundary | assumption-explicit route | `NUM-T01` |
| `NUMQ-003` | interpolation and finite error formulas | `L2` where prerequisites exist | `NUM-T02` |
| `NUMQ-004` | root-finding recurrence core | `L2` | `NUM-T04` |
| `NUMQ-005` | Krylov and stability alias map | alias and dependency split | `NUM-T07` |

## Review Checklist

- Deterministic recurrence and error-bound theorems are separated from
  machine-model assumptions.
- Linear algebra, analysis, optimization, statistics, and TCS ownership is
  respected through imports or aliases.
- Smoothness, mesh, conditioning, arithmetic, and probability assumptions are
  visible.
- No convergence or stability theorem assumes its conclusion through a route
  package.
- Public package work is outside this TODO until closure audit confirms stable `L2` derived
  certificates.
