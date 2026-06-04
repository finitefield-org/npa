# Analysis Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided analysis theorem inventory
one theorem at a time in the NPA proof corpus. It is a planning sidecar, not
proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- real numbers, limits, continuity, sequences, and series;
- one-variable differential and integral calculus;
- multivariable calculus;
- metric and topological analysis;
- measure theory and Lebesgue integration;
- functional analysis;
- complex analysis;
- Fourier analysis;
- ordinary and partial differential equations;
- calculus of variations and optimization.

The plan is intentionally staged. The first priority is not to encode every
famous theorem name immediately, but to build reusable foundations whose
statements will not need to be replaced after later theorems depend on them.

## Existing Baseline

The current proof corpus already has reusable analysis and linear-analysis
routes that should be reused instead of recreated:

| Corpus module | Existing role |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | metric balls, neighborhoods, local predicates, local equality, local uniqueness |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | normed-space law packages, product operations, product norm estimates |
| `Proofs.Ai.Analysis.AbstractLinearMap` | bounded linear maps, operator bounds, linear isomorphisms, block triangular inverse bridge |
| `Proofs.Ai.Analysis.AbstractDerivative` | Frechet derivative, differentiability, uniqueness, derivative rule packages, partial derivatives |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | completeness evidence, contractions, fixed-point evidence, Banach fixed-point theorem package |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | local inverse evidence and quantitative inverse-function theorem package |
| `Proofs.Ai.Analysis.AbstractImplicitPhi` | auxiliary map `Phi(x,y) = (x,F(x,y))` for the implicit-function route |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | implicit-function extraction, uniqueness, differentiability, derivative formula package |
| `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` | finite-dimensional normal-matrix spectral theorem package |
| `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem` | Hilbert-space spectral theorem package with construction evidence explicit |

These modules give a strong starting point for Banach fixed point, inverse
function, implicit function, derivative API, and spectral theorem work. They do
not yet replace the need for concrete real-number, sequence, series, Riemann
integral, measure, complex, Fourier, ODE, PDE, and variational foundations.

The current `develop/npa-mathlib-next-closure-roadmap.md` records public
materialization through `npa-mathlib v0.1.27`, including the metric-topology,
normed-space, linear-map, derivative, fixed-point, inverse-function, and
implicit-function analysis closures. This roadmap therefore focuses on the next
proof-corpus expansion work and on classical specializations, not on repeating
those completed closure audits.

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant or shape theorem only | no |
| `L1 Evidence package` | theorem conclusion follows from explicit construction or law evidence | only if explicitly marked as an interface milestone |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

For classical existence theorems, an `L1 Evidence package` is useful as a
stable interface, but it must not be confused with a fully derived theorem. A
task is considered mathematically complete only at `L2` or `L3`, unless the
scope explicitly says that the immediate target is an interface wrapper.

## One-Theorem Work Unit

For each theorem, use this work unit:

1. Freeze the statement in the smallest suitable `Proofs.Ai.*` module.
2. Classify the target as `L0`, `L1`, `L2`, or `L3`.
3. Audit the target for circular assumptions. The theorem conclusion itself
   must not appear as an input under another name.
4. Keep imports minimal and prefer existing corpus modules.
5. Add or update the checked source, replay, metadata, and certificate.
6. Verify the target module source-free.
7. Verify changed proof-corpus artifacts.
8. At the end of a coherent batch, run the authoring gate.

Default proof-corpus commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
only for package-wide compatibility, promotion, release readiness, or changes
to certificate encoding, checker behavior, package verification, or kernel
semantics.

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `ANA-00` | inventory and statement policy | theorem cards and dependency tags |
| `ANA-01` | real numbers and sequences | complete ordered field and sequence convergence API |
| `ANA-02` | series and power series | convergence criteria for numeric series |
| `ANA-03` | continuous functions on intervals | intermediate value, extrema, uniform continuity |
| `ANA-04` | one-variable differential calculus | Fermat, Rolle, mean value, Taylor route |
| `ANA-05` | Riemann integration | integrability criteria and fundamental theorem of calculus |
| `ANA-06` | finite-dimensional and multivariable calculus | Heine-Borel route, partials, inverse and implicit theorem aliases |
| `ANA-07` | metric and topological analysis | compactness, connectedness, Baire, Arzela-Ascoli route |
| `ANA-08` | measure and Lebesgue integration | measure construction and convergence theorems |
| `ANA-09` | functional analysis | Banach-space core theorems and Hilbert-space representation |
| `ANA-10` | complex analysis | holomorphic API and Cauchy theorem family |
| `ANA-11` | Fourier analysis | Fourier series and transform foundations |
| `ANA-12` | ordinary differential equations | Picard-Lindelof, Peano, Gronwall |
| `ANA-13` | PDE and Sobolev methods | weak formulation, Lax-Milgram, Sobolev embedding |
| `ANA-14` | variational methods and optimization | Euler-Lagrange, convex optimality, direct method |
| `ANA-15` | packaging and promotion | stable `npa-mathlib` closure audits |

## ANA-00 Inventory And Statement Policy

- Status: planned.
- Depends on: none.
- Deliverables:
  - Convert the theorem inventory into theorem cards.
  - Give every theorem a stable English identifier, Japanese display name,
    target level, dependencies, target module, and acceptance gate.
  - Mark theorem duplicates across areas, such as Fubini, Tonelli, inverse
    function, implicit function, Banach fixed point, Riesz representation, and
    spectral theorem.
- Acceptance criteria:
  - Every theorem has one primary home module.
  - Duplicates point to the primary theorem instead of being reproved.
  - Each card states whether the first target is a statement, evidence package,
    derived certificate, or public closure.
- Verification:
  - Documentation diff review.
  - `git diff --check`.

## ANA-01 Real Numbers And Sequences

- Status: planned.
- Depends on: existing algebra, ordered-field, metric-topology, and vector-space
  foundations.
- Target modules:
  - `Proofs.Ai.Analysis.Real.Basic`
  - `Proofs.Ai.Analysis.Sequence.Basic`
  - `Proofs.Ai.Analysis.Sequence.Compactness`
- Main design choice:
  - Start with an explicit `CompleteOrderedFieldArgs` package rather than
    adding a trusted real-number primitive to the kernel.
- Theorem order:
  1. real completeness;
  2. Archimedean property;
  3. interval nesting theorem;
  4. monotone convergence theorem for sequences;
  5. Cauchy sequence convergence theorem;
  6. squeeze theorem;
  7. Bolzano-Weierstrass theorem.
- Deliverables:
  - A reusable sequence convergence predicate.
  - Cauchy sequence and limit uniqueness lemmas compatible with
    `AbstractFixedPoint`.
  - Closed interval and bounded sequence vocabulary.
- Acceptance criteria:
  - Completeness is ordinary evidence over an explicit ordered-field structure.
  - Sequence convergence and Cauchy convergence are reusable by series,
    compactness, Riemann integration, Fourier analysis, and ODE milestones.
  - No theorem assumes the target convergence result as a law package unless
    it is marked `L1`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Analysis.Sequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## ANA-02 Series And Power Series

- Status: planned.
- Depends on: `ANA-01`.
- Target modules:
  - `Proofs.Ai.Analysis.Series.Basic`
  - `Proofs.Ai.Analysis.Series.Criteria`
  - `Proofs.Ai.Analysis.Series.Power`
- Theorem order:
  1. Cauchy convergence criterion for sequences and series;
  2. absolute convergence implies convergence;
  3. comparison test;
  4. d'Alembert ratio test;
  5. Cauchy root test;
  6. Leibniz alternating series test;
  7. Dirichlet test;
  8. Abel theorem for power series boundary limits;
  9. Riemann rearrangement theorem.
- Deliverables:
  - Partial sums and series convergence API.
  - Absolute convergence and conditional convergence API.
  - Power-series radius and boundary vocabulary.
- Acceptance criteria:
  - Series convergence reduces to sequence convergence of partial sums.
  - Rearrangement theorem is deferred until permutations, conditional
    convergence, and order-complete real estimates are stable.
  - Power-series statements do not depend on analytic-function machinery from
    complex analysis.
- Verification:
  - Module-local build and source-free verification for each new series module.
  - `./scripts/check-corpus-authoring.sh` after the criteria batch.

## ANA-03 Continuous Functions On Intervals

- Status: planned.
- Depends on: `ANA-01`.
- Target modules:
  - `Proofs.Ai.Analysis.Continuity.Basic`
  - `Proofs.Ai.Analysis.Continuity.Interval`
- Theorem order:
  1. limit laws for functions;
  2. continuity composition and local equality lemmas;
  3. Bolzano zero theorem;
  4. intermediate value theorem;
  5. extreme value theorem;
  6. uniform continuity theorem on compact intervals.
- Deliverables:
  - Pointwise and uniform continuity predicates.
  - Compact interval API specialized enough for one-variable calculus.
  - Image-of-interval and sign-change lemmas.
- Acceptance criteria:
  - Intermediate value and extreme value theorem use certified compactness or
    completeness lemmas from `ANA-01`, not unchecked topological assumptions.
  - Uniform continuity theorem is stated over closed bounded intervals first;
    general compact-space form belongs to `ANA-07`.
- Verification:
  - Build and verify `Proofs.Ai.Analysis.Continuity.Interval`.
  - Changed-only proof-corpus verification.

## ANA-04 One-Variable Differential Calculus

- Status: planned.
- Depends on: `ANA-03` and existing `AbstractDerivative`.
- Target modules:
  - `Proofs.Ai.Analysis.Calculus.OneVariable`
  - `Proofs.Ai.Analysis.Calculus.Taylor`
  - `Proofs.Ai.Analysis.Convex.Basic`
- Theorem order:
  1. Fermat theorem for differentiable local extrema;
  2. Rolle theorem;
  3. mean value theorem;
  4. Cauchy mean value theorem;
  5. Darboux theorem for derivatives;
  6. l'Hopital theorem;
  7. Taylor theorem with remainder;
  8. Maclaurin expansion as Taylor-at-zero specialization;
  9. one-variable inverse function theorem alias or specialization;
  10. convex tangent inequality.
- Deliverables:
  - Bridge from Frechet derivative API to one-dimensional derivative notation.
  - Local extrema and monotonicity vocabulary.
  - Taylor polynomial and remainder predicates.
- Acceptance criteria:
  - Rolle and mean value theorem depend on the interval continuity and
    compactness route, not on a primitive calculus axiom.
  - l'Hopital is added only after Cauchy mean value theorem and denominator
    nonzero derivative hypotheses are expressible.
  - Convexity is introduced as a reusable module for optimization and
    variational milestones.
- Verification:
  - Build and verify each calculus module.
  - Run `./scripts/check-corpus-authoring.sh` when the MVT batch lands.

## ANA-05 Riemann Integration

- Status: planned.
- Depends on: `ANA-01`, `ANA-03`, and parts of `ANA-04`.
- Target modules:
  - `Proofs.Ai.Analysis.Integral.Riemann.Basic`
  - `Proofs.Ai.Analysis.Integral.Riemann.Calculus`
- Theorem order:
  1. upper and lower sums;
  2. Riemann integrability criterion;
  3. continuous functions are Riemann integrable;
  4. bounded monotone functions are Riemann integrable;
  5. integral mean value theorem;
  6. fundamental theorem of calculus, part 1;
  7. fundamental theorem of calculus, part 2;
  8. integration by parts;
  9. substitution formula.
- Deliverables:
  - Partition, mesh, upper sum, lower sum, and refinement API.
  - Riemann integral value and uniqueness lemmas.
  - Bridge from derivative results to integral identities.
- Acceptance criteria:
  - Integrability proofs use interval compactness and continuity results from
    previous milestones.
  - Fundamental theorem statements are separated into continuity assumptions,
    primitive function assumptions, and interval orientation assumptions.
- Verification:
  - Module-local Riemann integration checks.
  - Authoring gate after the FTC batch.

## ANA-06 Finite-Dimensional And Multivariable Calculus

- Status: planned.
- Depends on: existing vector, normed-space, linear-map, derivative, inverse
  function, and implicit function modules; also `ANA-01` and `ANA-03`.
- Target modules:
  - `Proofs.Ai.Analysis.Euclidean.Basic`
  - `Proofs.Ai.Analysis.Calculus.Multivariable`
  - `Proofs.Ai.Analysis.Calculus.ChangeOfVariables`
  - `Proofs.Ai.Analysis.VectorCalculus`
- Theorem order:
  1. finite-dimensional Euclidean product norm and coordinate projections;
  2. compactness equivalence for Euclidean closed bounded sets;
  3. multivariable mean value theorem;
  4. multivariable Taylor theorem;
  5. equality of mixed partial derivatives under smoothness assumptions;
  6. inverse function theorem specialization from existing abstract route;
  7. implicit function theorem specialization from existing abstract route;
  8. Lagrange multipliers;
  9. Jacobian change-of-variables formula;
  10. Green theorem;
  11. Gauss divergence theorem;
  12. Stokes theorem.
- Deliverables:
  - Coordinate finite-product API.
  - Specializations of abstract inverse and implicit theorem modules to
    Euclidean statements.
  - Differential-form or vector-calculus boundary vocabulary for Green, Gauss,
    and Stokes.
- Acceptance criteria:
  - Inverse and implicit theorem milestones reuse
    `Proofs.Ai.Analysis.AbstractInverseFunction` and
    `Proofs.Ai.Analysis.AbstractImplicitFunction`.
  - Jacobian and vector-calculus theorems wait until the integration and
    orientation APIs are stable.
- Verification:
  - Source-free verification for abstract specialization modules.
  - Authoring gate after inverse and implicit theorem aliases are stable.

## ANA-07 Metric And Topological Analysis

- Status: planned.
- Depends on: existing metric topology and `ANA-01`, with some items depending
  on `ANA-08`.
- Target modules:
  - `Proofs.Ai.Topology.Basic`
  - `Proofs.Ai.Topology.Metric.Compact`
  - `Proofs.Ai.Topology.FunctionSpace`
- Theorem order:
  1. open and closed set basic theorems;
  2. compactness equivalence in metric and Euclidean settings;
  3. Heine-Borel theorem;
  4. continuous maps preserve compact sets;
  5. continuous maps preserve connected sets;
  6. Banach fixed-point theorem public alias from existing abstract route;
  7. Baire category theorem;
  8. Arzela-Ascoli theorem;
  9. Stone-Weierstrass theorem;
  10. Urysohn lemma.
- Deliverables:
  - General topological vocabulary with metric specializations.
  - Compact, connected, complete, and function-space APIs.
- Acceptance criteria:
  - Heine-Borel over Euclidean spaces depends on finite-dimensional and
    sequence compactness work, not only on a compactness law assumption.
  - Baire requires complete metric space foundation from the fixed-point and
    sequence milestones.
  - Stone-Weierstrass and Urysohn are late in this milestone because they need
    algebra of functions and normal-space topology.
- Verification:
  - Module-local builds.
  - Authoring gate after compactness and connectedness batches.

## ANA-08 Measure Theory And Lebesgue Integration

- Status: planned.
- Depends on: `ANA-01`, `ANA-07`, and parts of `ANA-05`.
- Detailed roadmap: `proofs/measure-theory-theorem-proof-roadmap.md`.
- Target modules:
  - `Proofs.Ai.Measure.Basic`
  - `Proofs.Ai.Measure.Construction`
  - `Proofs.Ai.Measure.Integral`
  - `Proofs.Ai.Measure.Product`
  - `Proofs.Ai.Measure.Decomposition`
- Theorem order:
  1. sigma algebra and measurable function API;
  2. outer measure and Caratheodory extension theorem;
  3. simple functions and Lebesgue integral construction;
  4. monotone convergence theorem;
  5. Fatou lemma;
  6. dominated convergence theorem;
  7. bounded convergence theorem;
  8. Tonelli theorem;
  9. Fubini theorem;
  10. Radon-Nikodym theorem;
  11. Lebesgue decomposition theorem;
  12. Egorov theorem;
  13. Lusin theorem;
  14. Riesz representation theorem for measures;
  15. Vitali convergence theorem;
  16. Lebesgue differentiation theorem.
- Deliverables:
  - Measure spaces, measurable sets, measurable functions, and almost
    everywhere predicates.
  - Integral construction and convergence theorem chain.
  - Product measures and signed or complex measures when needed.
- Acceptance criteria:
  - Monotone convergence is proved for Lebesgue integration separately from
    the sequence monotone convergence theorem in `ANA-01`.
  - Tonelli precedes Fubini.
  - Radon-Nikodym and decomposition results wait until signed measures and
    absolute continuity are stable.
- Verification:
  - Build and verify one measure module at a time.
  - Authoring gate after the convergence theorem chain.
  - Package gate before using measure foundations in Fourier, PDE, or
    variational milestones.

## ANA-09 Functional Analysis

- Status: planned.
- Depends on: existing normed-space, linear-map, fixed-point, inner-product,
  and spectral modules; several results depend on `ANA-07` and `ANA-08`.
- Target modules:
  - `Proofs.Ai.FunctionalAnalysis.Banach`
  - `Proofs.Ai.FunctionalAnalysis.Hilbert`
  - `Proofs.Ai.FunctionalAnalysis.WeakTopology`
  - `Proofs.Ai.FunctionalAnalysis.Spectral`
- Theorem order:
  1. Hahn-Banach theorem;
  2. uniform boundedness principle;
  3. open mapping theorem;
  4. closed graph theorem;
  5. Hilbert projection theorem;
  6. Riesz-Frechet representation theorem;
  7. Hilbert-space orthogonal decomposition theorem;
  8. Banach-Alaoglu theorem;
  9. Riesz representation theorem for locally compact spaces or measures;
  10. compact operator spectral theorem;
  11. Hilbert-space spectral theorem public alias from existing abstract route;
  12. Fredholm alternative;
  13. Krein-Milman theorem;
  14. Milman-Pettis theorem.
- Deliverables:
  - Banach and Hilbert space vocabulary beyond the current normed-space layer.
  - Continuous linear functional and dual-space APIs.
  - Weak and weak-star topology API.
- Acceptance criteria:
  - Hahn-Banach must state the scalar field and order or norm assumptions
    explicitly.
  - Open mapping and closed graph depend on Baire category.
  - Banach-Alaoglu depends on weak-star topology and compactness foundations.
  - Existing spectral theorem modules are treated as `L1` until their
    construction evidence is replaced or justified by derived foundations.
- Verification:
  - Build and verify each functional-analysis module.
  - Authoring gate after each theorem family.

## ANA-10 Complex Analysis

- Status: planned.
- Depends on: complex scalar field foundation, `ANA-01`, `ANA-03`, `ANA-05`,
  and parts of `ANA-07`.
- Target modules:
  - `Proofs.Ai.Complex.Basic`
  - `Proofs.Ai.Complex.Holomorphic`
  - `Proofs.Ai.Complex.Cauchy`
  - `Proofs.Ai.Complex.Meromorphic`
- Theorem order:
  1. complex field and norm foundation;
  2. holomorphic function and contour integral API;
  3. Cauchy integral theorem;
  4. Cauchy integral formula;
  5. Morera theorem;
  6. Liouville theorem;
  7. fundamental theorem of algebra;
  8. maximum modulus principle;
  9. minimum modulus principle;
  10. open mapping theorem for holomorphic functions;
  11. identity theorem;
  12. isolated singularity classification theorem;
  13. Laurent expansion theorem;
  14. residue theorem;
  15. argument principle;
  16. Rouche theorem;
  17. Schwarz lemma;
  18. Riemann mapping theorem;
  19. Mittag-Leffler theorem;
  20. Weierstrass factorization theorem.
- Deliverables:
  - Complex differentiability and path integration foundations.
  - Power series and analytic continuation vocabulary.
  - Meromorphic functions, residues, zeros, and poles.
- Acceptance criteria:
  - Cauchy theorem and formula must precede most downstream theorems.
  - Riemann mapping, Mittag-Leffler, and Weierstrass factorization are late
    because they need stronger topology, compactness, and approximation
    infrastructure.
- Verification:
  - Build and verify one complex-analysis module at a time.
  - Authoring gate after Cauchy theorem family and after residue theorem.

## ANA-11 Fourier Analysis

- Status: planned.
- Depends on: `ANA-02`, `ANA-08`, `ANA-10` for complex exponentials, and
  `ANA-09` for Hilbert-space viewpoints.
- Target modules:
  - `Proofs.Ai.Analysis.Fourier.Series`
  - `Proofs.Ai.Analysis.Fourier.Transform`
  - `Proofs.Ai.Analysis.Fourier.Sampling`
- Theorem order:
  1. trigonometric system and Fourier coefficient API;
  2. Fourier series expansion theorem under selected regularity assumptions;
  3. Dirichlet convergence theorem;
  4. Fejer theorem;
  5. Parseval identity;
  6. Plancherel theorem;
  7. Riemann-Lebesgue lemma;
  8. convolution theorem;
  9. Poisson summation formula;
  10. sampling theorem;
  11. Carleson theorem.
- Deliverables:
  - Periodic function spaces and trigonometric polynomial API.
  - Fourier transform and convolution API over integrable and square-integrable
    functions.
- Acceptance criteria:
  - Parseval and Plancherel use Hilbert-space and measure foundations.
  - Carleson theorem is a long-term target, not an early corpus theorem.
- Verification:
  - Module-local verification.
  - Package gate before Fourier modules are used by PDE milestones.

## ANA-12 Ordinary Differential Equations

- Status: planned.
- Depends on: existing fixed-point and derivative modules, plus `ANA-01`,
  `ANA-03`, `ANA-04`, and parts of `ANA-05`.
- Target modules:
  - `Proofs.Ai.Analysis.ODE.Basic`
  - `Proofs.Ai.Analysis.ODE.Existence`
  - `Proofs.Ai.Analysis.ODE.Linear`
  - `Proofs.Ai.Analysis.DynamicalSystems.Planar`
- Theorem order:
  1. integral equation formulation for ODEs;
  2. Gronwall inequality;
  3. Picard-Lindelof theorem;
  4. continuous dependence theorem;
  5. Peano existence theorem;
  6. linear ODE fundamental theorem;
  7. Floquet theorem;
  8. Sturm comparison theorem;
  9. Sturm-Liouville theory;
  10. Poincare-Bendixson theorem;
  11. Hartman-Grobman theorem.
- Deliverables:
  - Local solution, maximal solution, and flow vocabulary.
  - Lipschitz and continuous-dependence APIs.
  - Linear equation and fundamental matrix API.
- Acceptance criteria:
  - Picard-Lindelof should reuse Banach fixed-point machinery rather than add
    an ODE-specific existence primitive.
  - Peano requires compactness or selection machinery and is later than
    Picard-Lindelof.
  - Planar dynamical-system theorems wait until topology foundations are
    strong enough.
- Verification:
  - Module-local verification for the existence batch.
  - Authoring gate after Picard-Lindelof and Gronwall.

## ANA-13 PDE And Sobolev Methods

- Status: planned.
- Depends on: `ANA-08`, `ANA-09`, `ANA-11`, and parts of `ANA-12`.
- Target modules:
  - `Proofs.Ai.Analysis.Sobolev.Basic`
  - `Proofs.Ai.Analysis.PDE.Weak`
  - `Proofs.Ai.Analysis.PDE.Elliptic`
  - `Proofs.Ai.Analysis.PDE.Parabolic`
- Theorem order:
  1. weak derivative and Sobolev space API;
  2. Poincare inequality;
  3. Sobolev embedding theorem;
  4. Rellich compactness theorem;
  5. Lax-Milgram theorem;
  6. Hilbert-space weak solution existence theorem;
  7. energy estimates;
  8. maximum principle;
  9. elliptic regularity theorem;
  10. Cauchy-Kowalevski theorem;
  11. Holmgren uniqueness theorem.
- Deliverables:
  - Weak formulation and bilinear form API.
  - Sobolev norm, embedding, trace, and compactness vocabulary.
  - Elliptic and parabolic equation statement families.
- Acceptance criteria:
  - Lax-Milgram depends on Hilbert-space and bounded coercive bilinear form
    foundations.
  - Sobolev embedding and Rellich compactness require measure and compactness
    foundations, so they precede serious PDE theorem work.
  - Cauchy-Kowalevski and Holmgren require analytic-function or power-series
    infrastructure and are late.
- Verification:
  - Module-local verification.
  - Package or full corpus gate before PDE modules are treated as stable.

## ANA-14 Variational Methods And Optimization

- Status: planned.
- Depends on: `ANA-04`, `ANA-07`, `ANA-08`, `ANA-09`, and `ANA-13`.
- Target modules:
  - `Proofs.Ai.Analysis.Convex.Optimization`
  - `Proofs.Ai.Analysis.Variational.Basic`
  - `Proofs.Ai.Analysis.Variational.CriticalPoint`
- Theorem order:
  1. convex optimality conditions;
  2. Euler-Lagrange equation;
  3. Weierstrass existence theorem;
  4. direct method in the calculus of variations;
  5. Karush-Kuhn-Tucker conditions;
  6. Fenchel duality theorem;
  7. mountain pass theorem.
- Deliverables:
  - Convex sets, convex functions, lower semicontinuity, coercivity, and
    weak compactness APIs.
  - First variation and admissible variation vocabulary.
  - Dual pairing and subdifferential API for convex optimization.
- Acceptance criteria:
  - Direct method depends on compactness and lower semicontinuity, not an
    assumed minimizer.
  - KKT and Fenchel duality reuse convex analysis foundations from the tangent
    inequality route.
  - Mountain pass theorem waits until function-space topology and critical
    point vocabulary are mature.
- Verification:
  - Module-local verification.
  - Authoring gate after convex optimality and direct-method batches.

## ANA-15 Packaging And Promotion

- Status: planned.
- Depends on: any stable theorem batch.
- Deliverables:
  - Select closed theorem sets for `npa-mathlib` promotion.
  - Write closure audits before materialization.
  - Keep source-free package verification, package hash checks, theorem index
    checks, and axiom report checks as the public package acceptance criteria.
- Acceptance criteria:
  - The promoted module has stable names and statement shape.
  - The import closure does not drag staging modules into public `npa-mathlib`.
  - Axiom policy is not widened unless separately justified and reviewed.
- Verification:
  - Corpus module verification before promotion.
  - `npa-cli package check`, `build-certs --check`, `verify-certs`,
    `check-hashes`, `axiom-report --check`, `index --check`, and
    `publish-plan --check` against the target package.

## Recommended First Execution Queue

The first batch should focus on foundations that unlock many later theorem
families:

| Queue ID | Theorem or task | Target level | Primary milestone |
| --- | --- | --- | --- |
| `ANQ-001` | complete ordered field statement and law package audit | `L1` | `ANA-01` |
| `ANQ-002` | sequence convergence and limit uniqueness | `L2` | `ANA-01` |
| `ANQ-003` | Cauchy sequence API and convergence from completeness | `L2` | `ANA-01` |
| `ANQ-004` | monotone convergence theorem for sequences | `L2` | `ANA-01` |
| `ANQ-005` | squeeze theorem | `L2` | `ANA-01` |
| `ANQ-006` | interval nesting theorem | `L2` | `ANA-01` |
| `ANQ-007` | Bolzano-Weierstrass theorem | `L2` | `ANA-01` |
| `ANQ-008` | series partial sums and Cauchy criterion | `L2` | `ANA-02` |
| `ANQ-009` | absolute convergence implies convergence | `L2` | `ANA-02` |
| `ANQ-010` | comparison test | `L2` | `ANA-02` |
| `ANQ-011` | ratio and root tests | `L2` | `ANA-02` |
| `ANQ-012` | continuity API over intervals | `L2` | `ANA-03` |
| `ANQ-013` | intermediate value theorem | `L2` | `ANA-03` |
| `ANQ-014` | extreme value theorem | `L2` | `ANA-03` |
| `ANQ-015` | uniform continuity on compact intervals | `L2` | `ANA-03` |
| `ANQ-016` | one-dimensional derivative bridge | `L2` | `ANA-04` |
| `ANQ-017` | Fermat and Rolle theorems | `L2` | `ANA-04` |
| `ANQ-018` | mean value theorem | `L2` | `ANA-04` |
| `ANQ-019` | Riemann partition and integrability criterion | `L2` | `ANA-05` |
| `ANQ-020` | continuous functions are Riemann integrable | `L2` | `ANA-05` |

After `ANQ-020`, choose between:

- continuing the calculus route through the fundamental theorem of calculus;
- specializing the already available inverse and implicit function theorem
  routes to Euclidean statements;
- preparing the measure-theory route, including any missing topology
  prerequisites, if Fourier, PDE, and functional-analysis goals are more
  important than elementary calculus coverage.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Statements become too classical too early | later rewrites and broken imports | freeze abstract law-package statements first, then add concrete specializations |
| Evidence packages accidentally assume theorem conclusions | false sense of proof completion | require circular-assumption audit for every `L1` theorem |
| Large theorem batches make verification slow | slow repair loop | build and verify one small module or one theorem family at a time |
| Measure and PDE foundations pull in too much infrastructure | unstable imports and broad blast radius | keep measure, Sobolev, and PDE modules late and narrowly layered |
| Public promotion happens before names stabilize | compatibility burden | promote only `L2` theorem families with small import closures |

## Decision Points

- Whether the first real-analysis foundation should remain fully abstract over
  `CompleteOrderedFieldArgs`, or also introduce a named `Real` package early.
- Whether Riemann integration should be completed before measure theory, or
  whether Lebesgue integration should become the primary integration API.
- Whether complex analysis should use a separate complex-number construction
  or a law-package interface first.
- Whether high-level theorems such as Hahn-Banach, Radon-Nikodym, Riemann
  mapping, Carleson, and major PDE regularity theorems should first land as
  `L1` interfaces before derived proof attempts begin.
