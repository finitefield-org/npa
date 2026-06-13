# Optimization Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T10`, `BMQ-005`)
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/statistics-theorem-proof-roadmap-todo.md`
- `proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for optimization and
operations research. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers convex sets and functions, separation, Farkas lemma,
linear programming duality, polyhedra, conic duality, KKT conditions,
Fenchel duality, subgradients, projected and gradient methods, fixed-point and
proximal routes, dynamic programming, minimax and game theory, combinatorial
optimization bridges, stochastic optimization interfaces, and closure-boundary
planning.

Out of scope for this task document:

- adding convexity, optimization problems, algorithms, duality, or complexity
  models as trusted kernel primitives;
- treating an algorithm trace as proof of a theorem unless the trace checker
  and theorem statement make the invariant explicit;
- duplicating analysis convexity, linear-algebra matrix, statistics
  computation, or graph-optimization theorem ownership;
- publicly materializing unstable optimization modules before closure audit and package
  checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Optimization.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing related modules include `Proofs.Ai.Analysis.Convex.Optimization`,
  `Proofs.Ai.Combinatorics.Optimization`,
  `Proofs.Ai.Combinatorics.Optimization.Matroid`,
  `Proofs.Ai.Combinatorics.Optimization.Matroid.Polyhedral`, and
  `Proofs.Ai.Combinatorics.Optimization.Polytope`.
- Analysis currently owns the checked
  `Proofs.Ai.Analysis.Convex.Optimization` route, including convex
  set/function/subdifferential data, convex optimality, KKT, Fenchel duality,
  variational, and Euler-Lagrange facts.
- Linear algebra owns finite-dimensional matrix, rank, projection, positive
  definite, least-squares, and numerical linear algebra prerequisites.
- Statistics owns likelihood, risk, estimation, statistical computation, and
  stochastic approximation consequences that consume optimization results.
- Combinatorics owns graph and matroid applications. This todo owns the
  cross-roadmap ownership map, finite LP/operations-research routes not
  already owned by analysis, and algorithm correctness schemas; current
  KKT/Fenchel work should alias or audit `ANA-T37` unless a later migration
  plan explicitly changes the primary owner.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `OPT-00` inventory and namespace contract | `OPT-T00` |
| `OPT-01` convex sets, cones, and functions | `OPT-T01` |
| `OPT-02` separation and Farkas route | `OPT-T02` |
| `OPT-03` linear programming primal-dual theory | `OPT-T03` |
| `OPT-04` polyhedra and conic optimization | `OPT-T04` |
| `OPT-05` KKT and constrained smooth optimization | `OPT-T05` |
| `OPT-06` Fenchel duality and subgradients | `OPT-T06` |
| `OPT-07` descent, projected, and proximal methods | `OPT-T07` |
| `OPT-08` fixed-point and variational inequality routes | `OPT-T08` |
| `OPT-09` dynamic programming and Bellman equations | `OPT-T09` |
| `OPT-10` minimax and game theory | `OPT-T10` |
| `OPT-11` combinatorial optimization bridges | `OPT-T11` |
| `OPT-12` stochastic and statistical optimization bridges | `OPT-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `OPT-T00` | `L0` planning and theorem-card inventory |
| `OPT-T01`, `OPT-T05`, `OPT-T06` | ownership audit and `L2` aliases only after confirming they do not duplicate the current `ANA-T37` owner |
| `OPT-T02` through `OPT-T04` | `L2` for finite-dimensional separation, Farkas, LP, polyhedral, and conic certificates where linear-algebra prerequisites exist |
| `OPT-T07`, `OPT-T08` | `L2` where derivative, norm, and fixed-point prerequisites exist; split blockers before source edits |
| `OPT-T09` through `OPT-T12` | route packages first unless model and algorithm invariants are explicit |

## Milestones

### OPT-T00 Build Optimization Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/optimization-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory convexity, separation, LP, conic, KKT, Fenchel, algorithmic,
    dynamic-programming, minimax, combinatorial, and statistical optimization
    theorem families.
  - Assign primary owners and alias/import rules across analysis, linear
    algebra, combinatorics, and statistics.
  - Record target levels, model assumptions, and algorithm-invariant needs.
- Deliverables:
  - Optimization theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Duality and KKT have one primary owner and downstream aliases are explicit.
- Verification:
  - `rg -n "OPT-T00|Farkas|KKT|Fenchel|duality" proofs/optimization-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OPT-T01 Audit Convex Set And Function Core

- Status: Pending
- Depends on: `ANA-T37`, `LIN-T22`
- Areas: `Proofs.Ai.Analysis.Convex.Optimization`, future
  `Proofs.Ai.Optimization.Convex` only after an ownership migration plan
- Tasks:
  - Audit the existing analysis-owned convex set/function/subdifferential
    route and identify any LP/OR-specific aliases that should live under
    optimization.
  - Define only the missing cones, affine hulls, epigraphs, and Jensen-style
    route packages that are not already owned by `ANA-T37`.
  - Import normed-space and affine/vector foundations explicitly.
  - Prove elementary convex combination and epigraph lemmas only after their
    primary owner is clear.
- Deliverables:
  - Convex optimization audit notes or alias module.
- Acceptance criteria:
  - Convexity is stated over explicit vector and ordered-scalar law packages.
  - Existing `ANA-T37` theorem names are not duplicated under optimization.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Convex.Optimization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Convex.Optimization --verified-cache authoring`

### OPT-T02 Add Separation And Farkas Route

- Status: Pending
- Depends on: `OPT-T01`, `LIN-T40`
- Areas: `Proofs.Ai.Optimization.Separation`
- Tasks:
  - Define hyperplane separation, supporting hyperplanes, polar cones, and
    Farkas lemma route packages.
  - Split finite-dimensional algebraic Farkas from topological separation.
  - Prove finite-dimensional implication directions where prerequisites exist.
- Deliverables:
  - Separation and Farkas route module.
- Acceptance criteria:
  - Topological separation theorems state compactness, closure, and
    finite-dimensional assumptions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.Separation`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### OPT-T03 Add Linear Programming Duality Core

- Status: Pending
- Depends on: `OPT-T02`, `LIN-T09`
- Areas: `Proofs.Ai.Optimization.LinearProgramming`
- Tasks:
  - Define LP primal and dual problems, feasibility, boundedness, optimal
    value, weak duality, strong duality, and complementary slackness routes.
  - Prove weak duality from explicit matrix and order assumptions.
  - Split strong duality behind Farkas or separation evidence.
- Deliverables:
  - Linear programming duality module.
- Acceptance criteria:
  - Strong duality is not assumed to prove complementary slackness.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.LinearProgramming`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Optimization.LinearProgramming --verified-cache authoring`

### OPT-T04 Add Polyhedra And Conic Optimization Route

- Status: Pending
- Depends on: `OPT-T03`, `LIN-T13`
- Areas: `Proofs.Ai.Optimization.Conic`
- Tasks:
  - Define polyhedra, faces, extreme points, cones, dual cones, conic programs,
    and semidefinite-program route packages.
  - Coordinate polytope and matroid applications with combinatorics.
  - Split SDP duality behind positive-semidefinite and spectral prerequisites.
- Deliverables:
  - Polyhedral and conic route module.
- Acceptance criteria:
  - Combinatorial optimization modules import generic polyhedral facts rather
    than duplicating them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.Conic`
  - `rg -n "polyhedra|conic|semidefinite|dual cone" proofs/optimization-theorem-proof-roadmap-todo.md`

### OPT-T05 Audit KKT And Smooth Constrained Optimization Route

- Status: Pending
- Depends on: `OPT-T02`, `ANA-T13`, `ANA-T37`
- Areas: `Proofs.Ai.Analysis.Convex.Optimization`, future
  `Proofs.Ai.Optimization.KKT` only after an ownership migration plan
- Tasks:
  - Audit the existing analysis-owned KKT route.
  - Add optimization-side aliases only for LP/OR consumers that need stable
    names.
  - Split any new necessity theorem behind differentiability and separation
    prerequisites.
- Deliverables:
  - KKT ownership audit or alias module.
- Acceptance criteria:
  - Constraint qualifications are visible and not hidden in the KKT statement.
  - KKT theorem names from `ANA-T37` are not re-proved under a second owner.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Convex.Optimization`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### OPT-T06 Audit Fenchel Duality And Subgradient Route

- Status: Pending
- Depends on: `OPT-T01`, `OPT-T05`
- Areas: `Proofs.Ai.Analysis.Convex.Optimization`, future
  `Proofs.Ai.Optimization.Fenchel` only after an ownership migration plan
- Tasks:
  - Audit the existing analysis-owned Fenchel and subgradient route.
  - Add optimization-side aliases only for consumers that need stable
    operations-research names.
  - Prove any missing Fenchel-Young specialization from explicit definitions
    only after primary ownership is clear.
  - Split strong duality assumptions and closedness requirements.
- Deliverables:
  - Fenchel and subgradient audit or alias module.
- Acceptance criteria:
  - Subgradient and conjugate facts state domain and closure conditions.
  - Existing `ANA-T37` Fenchel theorem names are not duplicated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Convex.Optimization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Convex.Optimization --verified-cache authoring`

### OPT-T07 Add Descent And Proximal Algorithm Correctness Route

- Status: Pending
- Depends on: `OPT-T06`, `ANA-T22`
- Areas: `Proofs.Ai.Optimization.Algorithm`
- Tasks:
  - Define algorithm traces, descent invariants, step-size assumptions,
    projected gradient, proximal gradient, Newton, and coordinate descent
    routes.
  - Separate theorem statements from executable trace checking.
  - Prove deterministic recurrence and error-bound lemmas where assumptions
    are explicit.
- Deliverables:
  - Optimization algorithm correctness route module.
- Acceptance criteria:
  - Every algorithm theorem names the invariant and termination or convergence
    assumptions it uses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.Algorithm`
  - `rg -n "descent|proximal|step-size|invariant" proofs/optimization-theorem-proof-roadmap-todo.md`

### OPT-T08 Add Fixed-Point And Variational Inequality Route

- Status: Pending
- Depends on: `OPT-T01`, `ANA-T34`
- Areas: `Proofs.Ai.Optimization.Variational`
- Tasks:
  - Define variational inequalities, monotone operators, projection maps, and
    fixed-point reformulations.
  - Import Banach and contraction mapping foundations from analysis.
  - Split existence theorems by compactness and continuity assumptions.
- Deliverables:
  - Variational inequality route module.
- Acceptance criteria:
  - Fixed-point existence is imported from checked analysis routes.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.Variational`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### OPT-T09 Add Dynamic Programming Route

- Status: Pending
- Depends on: `OPT-T08`, `STAT-T55`
- Areas: `Proofs.Ai.Optimization.DynamicProgramming`
- Tasks:
  - Define finite-horizon decision processes, Bellman operators, value
    functions, policy improvement, and contraction route packages.
  - Split stochastic process assumptions from deterministic DP identities.
  - Coordinate Markov decision processes with stochastic calculus and
    statistics.
- Deliverables:
  - Dynamic programming route module.
- Acceptance criteria:
  - Bellman optimality statements identify state, action, measurability, and
    discount assumptions.
- Verification:
  - `rg -n "Bellman|dynamic programming|policy|Markov decision" proofs/optimization-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OPT-T10 Add Minimax And Game Theory Route

- Status: Pending
- Depends on: `OPT-T02`, `OPT-T03`
- Areas: `Proofs.Ai.Optimization.Game`
- Tasks:
  - Define zero-sum games, saddle points, minimax, Nash equilibrium route
    packages, and mixed strategies.
  - Prove finite matrix-game weak duality and route von Neumann minimax
    through LP or separation.
  - Split fixed-point Nash existence behind topology prerequisites.
- Deliverables:
  - Game theory and minimax route module.
- Acceptance criteria:
  - Existence theorems state compactness, convexity, and continuity
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Optimization.Game`
  - `rg -n "minimax|saddle|Nash|zero-sum" proofs/optimization-theorem-proof-roadmap-todo.md`

### OPT-T11 Add Combinatorial Optimization Bridge

- Status: Pending
- Depends on: `OPT-T03`, `CG-T29`
- Areas: `Proofs.Ai.Optimization.CombinatorialBridge`
- Tasks:
  - Map shortest path, matching, flow, matroid, and polytope theorem families
    to their combinatorics owners.
  - Provide generic LP and duality aliases used by graph and matroid modules.
  - Avoid relocating graph-specific theorems into optimization.
- Deliverables:
  - Bridge module or documentation alias map.
- Acceptance criteria:
  - Graph and matroid results import generic duality facts instead of
    reproving them.
- Verification:
  - `rg -n "matching|flow|matroid|polytope|duality" proofs/optimization-theorem-proof-roadmap-todo.md proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### OPT-T12 Add Stochastic And Statistical Optimization Bridge

- Status: Pending
- Depends on: `OPT-T07`, `STAT-T75`
- Areas: `Proofs.Ai.Optimization.StochasticBridge`
- Tasks:
  - Map SGD, stochastic approximation, EM, MM, MCMC, variational inference,
    and likelihood optimization to statistics owners.
  - Provide deterministic convex/algorithmic prerequisites for statistics.
  - Split stochastic convergence behind probability and stochastic-calculus
    prerequisites.
- Deliverables:
  - Statistical optimization bridge map.
- Acceptance criteria:
  - Statistical algorithms import optimization facts but keep statistical
    assumptions in statistics modules.
- Verification:
  - `rg -n "SGD|EM|MM|variational|likelihood" proofs/optimization-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `OPTQ-001` | theorem-card inventory | `L0` | `OPT-T00` |
| `OPTQ-002` | convex set/function ownership audit | `L2` aliases only after no-duplication review | `OPT-T01` |
| `OPTQ-003` | Farkas and separation split | `L2` for finite-dimensional routes | `OPT-T02` |
| `OPTQ-004` | LP weak duality and strong-duality route | `L2` for weak duality | `OPT-T03` |
| `OPTQ-005` | KKT ownership audit and consumer aliases | `L2` aliases only after no-duplication review | `OPT-T05` |
| `OPTQ-006` | Fenchel/subgradient ownership audit | `L2` aliases only after no-duplication review | `OPT-T06` |
| `OPTQ-007` | algorithm invariant schema | route package first | `OPT-T07` |
| `OPTQ-008` | statistics/combinatorics bridge map | documentation or aliases | `OPT-T11`, `OPT-T12` |

## Review Checklist

- Generic optimization theorem owners are distinct from analysis,
  combinatorics, linear algebra, and statistics owners.
- Algorithm theorems state invariants, models, and convergence assumptions.
- Strong duality, KKT necessity, and minimax existence are not assumed as
  shortcuts.
- Verification commands stay local until package metadata, checker compatibility, release, or high-trust changes.
