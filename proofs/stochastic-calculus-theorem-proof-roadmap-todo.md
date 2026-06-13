# Stochastic Calculus Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T08`, `BMQ-008`)
- `proofs/statistics-theorem-proof-roadmap-todo.md`
- `proofs/measure-theory-theorem-proof-roadmap-todo.md`
- `proofs/analysis-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for stochastic processes and
stochastic calculus. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers filtered probability spaces, adapted processes,
stopping times, martingales, optional stopping route packages, Brownian motion,
finite and discrete stochastic process certificates, stochastic integrals,
Ito formula, stochastic differential equations, Markov processes and
semigroups, Girsanov route packages, Feynman-Kac route packages, stochastic
approximation bridges, and closure-boundary planning.

Out of scope for this task document:

- adding probability spaces, filtrations, Brownian motion, stochastic
  integrals, or SDEs as trusted kernel primitives;
- treating Brownian motion, martingale convergence, Ito integral existence, or
  Girsanov theorem as statement-only shortcuts;
- duplicating finite probability/statistics, measure-theory martingale, or
  time-series theorem ownership;
- publicly materializing unstable stochastic modules before closure audit and package
  checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Stochastic.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Finite probability and statistics modules already exist for probability
  spaces, conditional probability, independence, random variables,
  distributions, expectation, moments, concentration, convergence,
  conditional expectation, LLN routes, and finite sampling distributions.
- Measure modules already include `Proofs.Ai.Measure.ProbabilityBridge`,
  `Proofs.Ai.Measure.ConditionalExpectation`,
  `Proofs.Ai.Measure.Martingale`, `Proofs.Ai.Measure.Ergodic`,
  `Proofs.Ai.Measure.WeakConvergence`, and integral/convergence foundations.
- Statistics owns time-series, stochastic approximation, MCMC, and statistical
  learning consequences. This todo owns the process and stochastic-calculus
  foundations those routes import.
- Continuous-time stochastic calculus should wait for measure-theoretic
  probability and integration prerequisites. First certificates should prefer
  finite or discrete-time process statements when possible.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `SC-00` inventory and namespace contract | `SC-T00` |
| `SC-01` filtered probability spaces and processes | `SC-T01` |
| `SC-02` stopping times and optional sampling setup | `SC-T02` |
| `SC-03` discrete-time martingales and inequalities | `SC-T03` |
| `SC-04` martingale convergence and optional stopping | `SC-T04` |
| `SC-05` Brownian motion and Gaussian process routes | `SC-T05` |
| `SC-06` stochastic integrals and Ito isometry | `SC-T06` |
| `SC-07` Ito formula and semimartingale route | `SC-T07` |
| `SC-08` stochastic differential equations | `SC-T08` |
| `SC-09` Markov processes and semigroups | `SC-T09` |
| `SC-10` Girsanov and change of measure | `SC-T10` |
| `SC-11` Feynman-Kac and PDE bridges | `SC-T11` |
| `SC-12` statistics and optimization bridges | `SC-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `SC-T00` | `L0` planning and theorem-card inventory |
| `SC-T01` through `SC-T03` | `L2` for finite or discrete-time certificates where probability and measure prerequisites exist |
| `SC-T04` through `SC-T07` | route packages first unless martingale, integration, and convergence prerequisites are explicit |
| `SC-T08` through `SC-T12` | dependency maps or `L2` only for finite/discrete structural lemmas |

## Milestones

### SC-T00 Build Stochastic Calculus Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory filtration, process, stopping time, martingale, Brownian motion,
    stochastic integral, Ito, SDE, Markov semigroup, Girsanov, Feynman-Kac,
    and statistics bridge theorem families.
  - Assign primary homes across probability/statistics, measure theory,
    analysis, optimization, and stochastic calculus.
  - Mark finite/discrete first targets separately from continuous-time routes.
- Deliverables:
  - Stochastic calculus theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Finite probability results and continuous-time stochastic results have
    distinct owners.
- Verification:
  - `rg -n "SC-T00|martingale|Brownian|Ito|Girsanov" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T01 Add Filtered Probability And Process Core

- Status: Pending
- Depends on: `STAT-T03`, `MEA-T48`
- Areas: `Proofs.Ai.Probability.Stochastic.Process`
- Tasks:
  - Define filtered probability spaces, measurable processes, adapted
    processes, predictable route packages, and finite time-index processes.
  - Import probability-measure and conditional-expectation prerequisites.
  - Prove elementary adaptedness and process restriction lemmas for finite
    index sets.
- Deliverables:
  - Filtered probability and process core module.
- Acceptance criteria:
  - Filtration assumptions are explicit and not confused with finite event
    algebras.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.Process`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Stochastic.Process --verified-cache authoring`

### SC-T02 Add Stopping Time Core

- Status: Pending
- Depends on: `SC-T01`
- Areas: `Proofs.Ai.Probability.Stochastic.StoppingTime`
- Tasks:
  - Define stopping times, stopped processes, optional processes, and hitting
    time route packages.
  - Prove finite-index stopped-process measurability facts where possible.
  - Split debut theorem and continuous-time hitting results behind topology
    and measure prerequisites.
- Deliverables:
  - Stopping time core module.
- Acceptance criteria:
  - Stopping-time measurability is proved or listed as an explicit
    prerequisite.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.StoppingTime`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### SC-T03 Add Discrete-Time Martingale Certificates

- Status: Pending
- Depends on: `SC-T02`, `MEA-T49`
- Areas: `Proofs.Ai.Probability.Stochastic.Martingale.Discrete`
- Tasks:
  - Define discrete-time martingales, submartingales, supermartingales, Doob
    decomposition route packages, and finite optional-sampling lemmas.
  - Import conditional expectation and finite-index process prerequisites.
  - Prove finite optional-sampling identities where assumptions are explicit.
- Deliverables:
  - Discrete-time martingale module.
- Acceptance criteria:
  - Optional sampling statements include boundedness or finite-index
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.Martingale.Discrete`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Stochastic.Martingale.Discrete --verified-cache authoring`

### SC-T04 Add Martingale Convergence And Optional Stopping Route

- Status: Pending
- Depends on: `SC-T03`, `MEA-T51`
- Areas: `Proofs.Ai.Probability.Stochastic.Martingale.Convergence`
- Tasks:
  - Define uniform integrability, upcrossing, martingale convergence,
    optional stopping, and Doob inequality route packages.
  - Import measure-theory martingale and convergence prerequisites.
  - Split continuous-time optional stopping behind right-continuity and
    integrability assumptions.
- Deliverables:
  - Martingale convergence route module.
- Acceptance criteria:
  - Integrability, uniform integrability, and stopping-time hypotheses are
    visible.
- Verification:
  - `rg -n "optional stopping|upcrossing|Doob|uniform integrability" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T05 Add Brownian Motion And Gaussian Process Route

- Status: Pending
- Depends on: `STAT-T22`, `MEA-T47`, `SC-T01`
- Areas: `Proofs.Ai.Probability.Stochastic.Brownian`
- Tasks:
  - Define Gaussian processes, Brownian motion route packages, independent
    increments, continuity, quadratic variation route packages, and finite
    dimensional distributions.
  - Split construction of Brownian motion behind Kolmogorov extension and
    continuity prerequisites.
  - Coordinate Gaussian law packages with named distribution milestones.
- Deliverables:
  - Brownian motion dependency map.
- Acceptance criteria:
  - Brownian existence is not assumed without extension and path-regularity
    prerequisites.
- Verification:
  - `rg -n "Brownian|Gaussian process|quadratic variation|Kolmogorov" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T06 Add Stochastic Integral And Ito Isometry Route

- Status: Pending
- Depends on: `SC-T04`, `SC-T05`, `MEA-T40`
- Areas: `Proofs.Ai.Probability.Stochastic.Integral`
- Tasks:
  - Define simple predictable integrands, stochastic integral route packages,
    Ito isometry, localization, and extension to square-integrable integrands.
  - Prove finite simple-integrand identities where prerequisites exist.
  - Split existence/completion behind `L^2` and martingale convergence
    prerequisites.
- Deliverables:
  - Stochastic integral route module.
- Acceptance criteria:
  - Ito isometry states integrand class and square-integrability assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.Integral`
  - `rg -n "Ito isometry|predictable|square-integrable|localization" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`

### SC-T07 Add Ito Formula And Semimartingale Route

- Status: Pending
- Depends on: `SC-T06`, `ANA-T13`
- Areas: `Proofs.Ai.Probability.Stochastic.Ito`
- Tasks:
  - Define semimartingales, quadratic variation, stochastic differential
    notation, and Ito formula route packages.
  - Split one-dimensional and multidimensional Ito formula.
  - Import differentiability prerequisites from analysis.
- Deliverables:
  - Ito formula route module.
- Acceptance criteria:
  - Smoothness, integrability, and semimartingale assumptions are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Stochastic.Ito`
  - `rg -n "Ito formula|semimartingale|quadratic variation" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`

### SC-T08 Add SDE Existence And Uniqueness Route

- Status: Pending
- Depends on: `SC-T07`, `ANA-T34`
- Areas: `Proofs.Ai.Probability.Stochastic.SDE`
- Tasks:
  - Define SDEs, strong solution, weak solution, pathwise uniqueness, weak
    uniqueness, Lipschitz and growth assumptions, and Euler-Maruyama route
    packages.
  - Split existence proofs through fixed-point or approximation routes.
  - Coordinate numerical convergence with numerical analysis later.
- Deliverables:
  - SDE route module.
- Acceptance criteria:
  - Existence and uniqueness statements state coefficient regularity,
    filtration, and Brownian assumptions.
- Verification:
  - `rg -n "SDE|strong solution|pathwise uniqueness|Euler-Maruyama" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T09 Add Markov Process And Semigroup Route

- Status: Pending
- Depends on: `SC-T01`, `STAT-T55`
- Areas: `Proofs.Ai.Probability.Stochastic.Markov`
- Tasks:
  - Define Markov kernels, transition semigroups, generators, invariant
    measures, strong Markov property, and martingale problem route packages.
  - Split finite-state Markov chain certificates from general Markov
    processes.
  - Coordinate time-series and MCMC ownership with statistics.
- Deliverables:
  - Markov process route module.
- Acceptance criteria:
  - Strong Markov and generator theorems state measurability and domain
    assumptions.
- Verification:
  - `rg -n "Markov|semigroup|generator|invariant measure" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T10 Add Girsanov And Change-Of-Measure Route

- Status: Pending
- Depends on: `SC-T06`, `MEA-T30`
- Areas: `Proofs.Ai.Probability.Stochastic.Girsanov`
- Tasks:
  - Define equivalent measures, Radon-Nikodym densities, exponential
    martingales, Novikov condition route packages, and Girsanov route
    packages.
  - Import measure-theory change-of-measure prerequisites.
  - Split finite discrete change-of-measure certificates from continuous-time
    Girsanov.
- Deliverables:
  - Girsanov dependency module.
- Acceptance criteria:
  - Absolute continuity, density, and integrability assumptions are visible.
- Verification:
  - `rg -n "Girsanov|Novikov|Radon-Nikodym|change of measure" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T11 Add Feynman-Kac And PDE Bridge

- Status: Pending
- Depends on: `SC-T08`, `ANA-T35`
- Areas: `Proofs.Ai.Probability.Stochastic.FeynmanKac`
- Tasks:
  - Define generator/PDE correspondence, stopping-domain route packages,
    Feynman-Kac route packages, and boundary condition assumptions.
  - Coordinate PDE prerequisites with analysis.
  - Split existence and uniqueness of PDE solutions from stochastic
    representation.
- Deliverables:
  - Feynman-Kac dependency map.
- Acceptance criteria:
  - PDE regularity and stochastic process assumptions are both visible.
- Verification:
  - `rg -n "Feynman-Kac|PDE|generator|boundary" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### SC-T12 Add Statistics And Optimization Bridge

- Status: Pending
- Depends on: `SC-T03`, `SC-T09`, `OPT-T12`
- Areas: `Proofs.Ai.Probability.Stochastic.StatisticsBridge`
- Tasks:
  - Map stochastic approximation, time series, MCMC, sequential testing,
    survival models, and statistical learning routes to their statistics
    owners.
  - Provide stochastic-process prerequisites as aliases or route packages.
  - Avoid moving statistical inference theorems into stochastic calculus.
- Deliverables:
  - Statistics and optimization bridge map.
- Acceptance criteria:
  - Statistics modules import stochastic prerequisites without duplicating
    process foundations.
- Verification:
  - `rg -n "stochastic approximation|MCMC|time series|sequential" proofs/stochastic-calculus-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `SCQ-001` | theorem-card inventory | `L0` | `SC-T00` |
| `SCQ-002` | filtered probability and finite process core | `L2` where measure prerequisites exist | `SC-T01` |
| `SCQ-003` | stopping time core | `L2` for finite-index facts | `SC-T02` |
| `SCQ-004` | discrete-time martingale certificates | `L2` | `SC-T03` |
| `SCQ-005` | optional stopping route split | route package first | `SC-T04` |
| `SCQ-006` | Brownian dependency split | route package first | `SC-T05` |
| `SCQ-007` | simple stochastic integral route | `L2` for simple finite facts | `SC-T06` |
| `SCQ-008` | statistics bridge map | documentation or aliases | `SC-T12` |

## Review Checklist

- Finite probability, measure-theory martingale, statistics, and stochastic
  calculus owners are distinct.
- Continuous-time existence theorems are not accepted before measure,
  integration, and path-regularity prerequisites are visible.
- Brownian motion, Ito, Girsanov, and SDE routes are dependency maps until
  their foundations exist.
- Verification commands stay local until package metadata, checker compatibility, release, or high-trust changes.
