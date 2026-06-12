# Statistics Theorem Proof Roadmap Todo

Source: `proofs/statistics-theorem-proof-roadmap.md`

This document decomposes the statistics theorem proof roadmap into concrete
authoring milestones. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, or certificate validity assumptions.

## Scope

This task list covers theorem-card inventory, namespace setup, finite and
simple probability/statistics results, and the dependency-tagged routes for
measure-theoretic, asymptotic, Bayesian, regression, multivariate,
nonparametric, time-series, causal, learning, computation, decision-theory,
and distribution-specific theorem families.

The list intentionally does not prove the roadmap in one pass. Later agents
should implement exactly one milestone or a clearly bounded contiguous batch.
When a milestone introduces only a statement interface because prerequisites
are absent, its acceptance criteria must prevent the interface from smuggling
the target theorem as an axiom.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- treating theorem-search sidecars, AI indexes, replay files, or generated
  docs as trusted evidence;
- promoting unstable statistics modules into `npa-mathlib` before local
  closure, axiom-report, and package verification checks are clean.

## Authoring Loop

For ordinary statistics theorem authoring, prefer local proof-corpus checks
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

- Checked `Proofs.Ai.Probability.*` modules now exist for finite probability
  space basics, conditional probability, independence, random variables,
  distributions, expectation, moments, concentration inequalities, convergence,
  conditional expectation, and LLN routes.
- A dedicated `Proofs.Ai.Statistics.SamplingDistribution.Basic` module now
  exists for finite iid sample vocabulary, sample-mean expectation and variance,
  and unbiased sample-variance certificates. Other statistics work still lands
  in the shared `Proofs.Ai.Probability.*` namespace until inference-specific
  APIs are split.
- Concrete `Proofs.Ai.Measure.*` modules now exist separately for the detailed
  measure roadmap. General measure-theoretic probability should still wait for
  the explicit measure/Lebesgue prerequisites instead of importing finite
  probability foundations as if they supplied sigma-additivity.
- Statistics work should reuse existing abstract infrastructure where it is
  already checked: `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`,
  `Proofs.Ai.Vector.AbstractSpace`,
  `Proofs.Ai.Vector.AbstractInnerProduct`,
  `Proofs.Ai.Vector.AbstractInnerProductDerive`,
  `Proofs.Ai.Analysis.AbstractMetricTopology`,
  `Proofs.Ai.Analysis.AbstractNormedSpace`,
  `Proofs.Ai.Analysis.AbstractLinearMap`,
  `Proofs.Ai.Analysis.AbstractDerivative`,
  `Proofs.Ai.Analysis.AbstractFixedPoint`,
  `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem`, and
  `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem`.
- Full measure-theoretic probability should wait for the analysis roadmap's
  measure and Lebesgue route, especially `ANA-T24` through `ANA-T26`, and the
  detailed measure roadmap in `proofs/measure-theory-theorem-proof-roadmap.md`.
- Characteristic-function, Fourier, spectral-density, and CLT strengthening
  should remain dependency-tagged until the analysis Fourier route
  `ANA-T31` through `ANA-T32` exists.
- Statistical optimization milestones may reuse the existing abstract
  derivative/fixed-point modules, but convex/KKT/duality and variational
  analysis should stay aligned with analysis roadmap `ANA-T37`.
- Bayes probability formula belongs to `STAT-02`; Bayesian posterior formulas
  belong to `STAT-14`; Bayes risk and Bayes decision rules belong to
  `STAT-25`. Posterior-risk aliases from `STAT-14` must wait for the
  decision-risk API.

## Roadmap Coverage Map

| Roadmap item | Task milestones |
| --- | --- |
| `STAT-00` inventory and statement policy | `STAT-T00` |
| `STAT-01` probability space basics | `STAT-T01` through `STAT-T03` |
| `STAT-02` conditional probability and independence | `STAT-T04` through `STAT-T05` |
| `STAT-03` random variables and distributions | `STAT-T06` through `STAT-T08` |
| `STAT-04` expectation, moments, and concentration | `STAT-T09` through `STAT-T11` |
| `STAT-05` conditional expectation | `STAT-T12` through `STAT-T13` |
| `STAT-06` convergence of random variables | `STAT-T14` through `STAT-T16` |
| `STAT-07` laws of large numbers | `STAT-T17` through `STAT-T18` |
| `STAT-08` CLT and asymptotic tools | `STAT-T19` through `STAT-T21` |
| `STAT-09` named distributions and sampling distributions | `STAT-T22` through `STAT-T24` |
| `STAT-10` sufficiency and unbiased estimation | `STAT-T25` through `STAT-T27` |
| `STAT-11` information and likelihood theory | `STAT-T28` through `STAT-T31` |
| `STAT-12` hypothesis testing | `STAT-T32` through `STAT-T35` |
| `STAT-13` confidence intervals | `STAT-T36` through `STAT-T38` |
| `STAT-14` Bayesian statistics | `STAT-T39` through `STAT-T42` |
| `STAT-15` regression and ANOVA | `STAT-T43` through `STAT-T46` |
| `STAT-16` multivariate statistics | `STAT-T47` through `STAT-T50` |
| `STAT-17` nonparametric and empirical processes | `STAT-T51` through `STAT-T54` |
| `STAT-18` time series and stochastic processes | `STAT-T55` through `STAT-T57` |
| `STAT-19` information theory and large deviations | `STAT-T58` through `STAT-T61` |
| `STAT-20` martingales and stochastic approximation | `STAT-T62` through `STAT-T64` |
| `STAT-21` survival, survey sampling, and missing data | `STAT-T65` through `STAT-T67` |
| `STAT-22` causal inference | `STAT-T68` through `STAT-T70` |
| `STAT-23` statistical learning | `STAT-T71` through `STAT-T74` |
| `STAT-24` statistical computation and optimization | `STAT-T75` through `STAT-T78` |
| `STAT-25` decision theory | `STAT-T79` through `STAT-T81` |
| `STAT-26` distribution-specific and extreme-value theory | `STAT-T82` through `STAT-T84` |
| `STAT-27` packaging and promotion | `STAT-T85` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `STAT-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `STAT-T01` | target `L2` finite probability certificates from the first proof attempt, including complement and nonnegativity lemmas before downstream use |
| `STAT-T02`, `STAT-T04`, `STAT-T09`, `STAT-T10`, `STAT-T11`, `STAT-T15`, `STAT-T17`, `STAT-T23`, `STAT-T25`, `STAT-T26`, `STAT-T33`, `STAT-T43`, `STAT-T44`, `STAT-T79` | `L2` derived certificates for finite, discrete, simple-function, or finite-dimensional theorem families |
| `STAT-T05`, `STAT-T06`, `STAT-T07`, `STAT-T12`, `STAT-T14`, `STAT-T19`, `STAT-T22`, `STAT-T28`, `STAT-T32`, `STAT-T36`, `STAT-T39`, `STAT-T47`, `STAT-T55`, `STAT-T58`, `STAT-T62`, `STAT-T65`, `STAT-T68`, `STAT-T71`, `STAT-T75`, `STAT-T82` | target `L2` derived certificates from the first proof attempt; split missing probability, measure, optimization, or learning prerequisites before source edits |
| `STAT-T03`, `STAT-T08`, `STAT-T13`, `STAT-T16`, `STAT-T18`, `STAT-T20`, `STAT-T21`, `STAT-T24`, `STAT-T27`, `STAT-T29` through `STAT-T31`, `STAT-T34`, `STAT-T35`, `STAT-T37`, `STAT-T38`, `STAT-T40` through `STAT-T42`, `STAT-T45`, `STAT-T46`, `STAT-T48` through `STAT-T54`, `STAT-T56`, `STAT-T57`, `STAT-T59` through `STAT-T61`, `STAT-T63`, `STAT-T64`, `STAT-T66`, `STAT-T67`, `STAT-T69`, `STAT-T70`, `STAT-T72` through `STAT-T74`, `STAT-T76` through `STAT-T78`, `STAT-T80`, `STAT-T81`, `STAT-T83`, `STAT-T84` | split before source edits if prerequisites are absent; otherwise target `L2` derived certificates with explicit imports |
| `STAT-T85` | `L3` public closure and package verification |

## Milestones

### STAT-T00 Build Statistics Theorem Card Inventory

- Status: Completed (2026-06-12; L0 theorem-card inventory)
- Depends on: None
- Areas: `proofs/README.md`, proof-corpus theorem-card documentation, AI index sidecars
- Tasks:
  - Create theorem cards for all `STAT-00` through `STAT-27` theorem families.
  - Record duplicate-home decisions from the roadmap, especially Bayes, Bonferroni, LLN, CLT, testing, and regression aliases.
  - Tag each card with target level, prerequisite modules, axiom expectations, and intended proof-corpus namespace.
- Deliverables:
  - Completed with `proofs/statistics-theorem-cards.md`, including primary
    cards for `STAT-00` through `STAT-27`, a namespace contract, a duplicate
    home map, and first execution queue entries that later milestones can cite.
- Acceptance criteria:
  - Every roadmap row has a card or an intentionally grouped card.
  - Bayes formula, Bayes posterior, and Bayes decision-risk cards have distinct primary homes.
  - The inventory states that sidecars and theorem search are untrusted.
- Verification:
  - `rg -n "STAT-00|STAT-27|Bayes|Bonferroni|sidecar" proofs/statistics-theorem-cards.md proofs/README.md`
  - `git diff --check`

### STAT-T01 Create Finite Event Algebra And Probability Law Package

- Status: Completed (2026-06-12)
- Depends on: `STAT-T00`
- Areas: `Proofs.Ai.Probability.Space.Basic`
- Tasks:
  - Define finite event-algebra and probability-law records using ordinary structures.
  - Add complement, empty, universal, finite union, and finite intersection statement names.
  - Keep finite probability foundations separate from future measure extension machinery.
- Deliverables:
  - Source, certificate, replay, and metadata for a finite probability-space base module.
  - Completed with `Proofs.Ai.Probability.Space.Basic`, including
    `FiniteEventAlgebra`, `FiniteProbabilityLawPackage`, closure projections
    for empty/universal/complement/union/intersection, and the derived
    `finite_event_algebra_difference` theorem from complement plus
    intersection closure.
- Acceptance criteria:
  - Complement and nonnegativity facts are derived from explicit finite probability laws.
  - No measure-theoretic extension theorem is assumed in the finite base module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Space.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Space.Basic`

### STAT-T02 Prove Elementary Finite Probability Laws

- Status: Completed (2026-06-12; L2 finite probability certificates)
- Depends on: `STAT-T01`
- Areas: `Proofs.Ai.Probability.Space.Basic`
- Tasks:
  - Prove finite additivity, monotonicity, subadditivity, Boole inequality, and Bonferroni inequalities.
  - Add names that make later testing Bonferroni aliases import probability inequalities instead of reproving them.
  - Record theorem cards for finite/countable distinctions.
- Deliverables:
  - Derived elementary probability law certificates.
  - Current proof coverage in `Proofs.Ai.Probability.Space.Basic` includes
    `finite_probability_additive_disjoint`,
    `finite_probability_additive_disjoint_symmetric`,
    `finite_probability_monotonicity_derived`,
    `finite_probability_subadditivity_derived`,
    `finite_probability_boole_inequality_derived`, and
    `finite_probability_bonferroni_inequality_derived`.
  - Finite/countable ownership is now recorded by
    `proofs/statistics-theorem-cards.md`: finite additivity and Bonferroni live
    under `STAT-01`, while countable additivity and extension theorems are
    dependency-routed to measure foundations plus `STAT-T03`.
- Acceptance criteria:
  - Boole and Bonferroni results are primary probability theorems, not multiple-testing theorems.
  - Proofs use explicit finite union hypotheses and do not assume sigma-additivity.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Space.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Space.Basic --verified-cache authoring`

### STAT-T03 Add Measure-Theoretic Probability And Extension Interfaces

- Status: Completed (2026-06-12; L2 dependency-route certificate)
- Depends on: `STAT-T02`
- Areas: `Proofs.Ai.Probability.Space.Extension`
- Tasks:
  - Define the interface expected from measurable spaces, measures, probability measures, and generated sigma-algebras.
  - Split Caratheodory/Kolmogorov extension statements from finite event algebra.
  - Mark countable additivity and extension theorems as blocked until measure foundations exist.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Space.Extension`, including
    `MeasureTheoreticProbabilityExtensionPackage`,
    `measure_theoretic_probability_extension_dependency_routes`, and
    `measure_theoretic_probability_no_target_extension_axiom`.
  - The certificate records exact route dependencies for Hahn-Kolmogorov,
    Kolmogorov, Ionescu-Tulcea, total-one, random-variable measurability,
    Borel-Cantelli, and convergence aliases without asserting any extension
    target theorem directly.
- Acceptance criteria:
  - Extension interfaces do not introduce the target extension theorem as an axiom under another name.
  - Imports identify the exact analysis measure milestones required before `L2` work.
- Notes:
  - General `L2` extension work depends on `ANA-T24` through `ANA-T26`.
  - The checked STAT-T03 certificate keeps a small import boundary and records
    the required measure/probability extension routes as explicit fields instead
    of asserting any target extension theorem or re-exporting a broader bridge.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Space.Extension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Space.Extension --verified-cache authoring`

### STAT-T04 Add Finite Conditional Probability And Bayes Formula

- Status: Completed (2026-06-12; L2 finite conditional probability certificates)
- Depends on: `STAT-T02`
- Areas: `Proofs.Ai.Probability.Conditional.Basic`
- Tasks:
  - Define finite conditional probability for nonzero conditioning events.
  - Prove multiplication rule, total probability, finite Bayes theorem, and chain-rule variants.
  - Add cross-links to decision theory without importing risk theorems.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Conditional.Basic`, including
    `FiniteConditionalProbabilityPackage`,
    `finite_conditional_probability_definition`,
    `finite_conditional_probability_multiplication_rule`,
    `finite_total_probability_derived`,
    `finite_bayes_formula_derived`, and
    `finite_conditional_probability_chain_rule_derived`.
  - Current coverage in `Proofs.Ai.Probability.Conditional.Basic` now also
    includes `finite_conditional_probability_intersection_event`, deriving the
    conditional-probability intersection event side condition from the finite
    probability law package and event-algebra intersection closure.
- Acceptance criteria:
  - Bayes formula has no dependency on `STAT-T25`.
  - Zero-denominator side conditions are explicit and testable in theorem statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Conditional.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Conditional.Basic --verified-cache authoring`

### STAT-T05 Add Independence Predicate Family

- Status: Completed (2026-06-12; L2 finite event independence certificates)
- Depends on: `STAT-T04`
- Areas: `Proofs.Ai.Probability.Independence.Basic`
- Tasks:
  - Define event independence, pairwise independence, mutual independence, and independent families.
  - Prove product, complement, and finite family lemmas needed by LLN, CLT, and sampling results.
  - Separate event independence from random-variable independence until random variables exist.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Independence.Basic`, including
    `EventIndependent`, `PairwiseEventIndependentFamily`,
    `MutualEventIndependentFamily`, `FiniteEventIndependencePackage`,
    `event_independent_product_rule`,
    `mutual_event_independence_pairwise_derived`,
    `event_independent_complement_left_derived`,
    `event_independent_complement_right_derived`, and
    conditional-event independence product projections.
- Acceptance criteria:
  - Pairwise and mutual independence are not conflated.
  - Later random-variable independence theorem cards cite this module but do not duplicate event-level proofs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Independence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Independence.Basic --verified-cache authoring`

### STAT-T06 Add Random Variable And Distribution Statement API

- Status: Completed (2026-06-12; L2 finite random-variable/distribution certificates)
- Depends on: `STAT-T02`, `STAT-T05`
- Areas: `Proofs.Ai.Probability.RandomVariable.Basic`, `Proofs.Ai.Probability.Distribution.Basic`
- Tasks:
  - Define measurable random-variable and pushforward-distribution statement shapes.
  - Add finite/discrete random-variable specializations that can be proved before full measure theory.
  - Specify random-variable independence as a pullback of event independence.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.RandomVariable.Basic` and
    `Proofs.Ai.Probability.Distribution.Basic`, including
    `RandomVariablePreimage`, `FiniteRandomVariable`,
    `FiniteRandomVariablePackage`, `RandomVariablesEventIndependent`,
    `FiniteDistributionPackage`, `finite_distribution_pushforward`,
    `DiscreteProbabilityMassPackage`, and `NamedDistributionPackage`.
- Acceptance criteria:
  - The API distinguishes random variables, their laws, and named distribution packages.
  - Finite specializations avoid pretending to provide the full measurable-space theorem.
- Notes:
  - General measurable random-variable and pushforward-distribution proofs depend on `STAT-T03` and `ANA-T24`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.RandomVariable.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.RandomVariable.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Distribution.Basic --verified-cache authoring`

### STAT-T07 Add CDF, Density, And Transform Basics

- Status: Completed (2026-06-12; L2 finite/discrete CDF, transform, and mass certificates; full measure right-continuity/density remains prerequisite-owned)
- Depends on: `STAT-T06`, `ANA-T24`
- Areas: `Proofs.Ai.Probability.Distribution.Basic`, `Proofs.Ai.Probability.Distribution.Transform`
- Tasks:
  - Add CDF monotonicity with explicit codomain order and lower-set event evidence.
  - Add discrete mass-function and simple finite transform formulas.
  - Keep density-with-respect-to-measure statements behind measure prerequisites.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Distribution.Transform`, including
    `DistributionFunctionPackage`,
    `distribution_function_lower_set_event`,
    `distribution_function_cdf_definition`,
    `distribution_function_cdf_monotone`,
    `FiniteDistributionTransformPackage`,
    `finite_distribution_transform_measurable`,
    `finite_distribution_transform_pushforward`,
    `DiscreteTransformMassPackage`,
    `discrete_transform_fiber_sound`,
    `discrete_transform_fiber_complete`, and
    `discrete_transform_mass_formula`.
  - Full CDF right-continuity and density-with-respect-to-measure theorems are
    not claimed here; they remain owned by the measure/Lebesgue and
    Radon-Nikodym/change-of-variables prerequisite routes.
- Acceptance criteria:
  - CDF and density statements identify the codomain order and measurability assumptions explicitly.
  - Monotone-transform formulas do not depend on unimplemented change-of-variables theorems.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Transform`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Distribution.Transform --verified-cache authoring`

### STAT-T08 Add Transform And Levy Continuity Planning Split

- Status: Completed (2026-06-12; dependency-safe transform and Levy continuity split recorded)
- Depends on: `STAT-T07`, `ANA-T31`
- Areas: `Proofs.Ai.Probability.Distribution.Transform`, `Proofs.Ai.Probability.Convergence.Weak`
- Tasks:
  - Split moment-generating, characteristic-function, probability-generating, and Laplace-transform theorem routes.
  - Record Fourier dependencies for inversion and Levy continuity.
  - Add theorem-card aliases for later CLT milestones.
- Deliverables:
  - Completed in `proofs/statistics-theorem-cards.md` by making
    `Proofs.Ai.Probability.Distribution.Transform` part of `STAT-03` and by
    adding duplicate-home guidance for moment-generating, characteristic,
    probability-generating, and Laplace transform aliases.
  - Dependency-safe statement names for later modules:
    `moment_generating_transform_route`,
    `characteristic_function_fourier_route`,
    `probability_generating_transform_route`,
    `laplace_transform_route`, and
    `levy_continuity_fourier_route`.
  - Characteristic-function inversion and Levy continuity are explicitly
    dependency-routed to `ANA-T31`/`ANA-T32`; no Fourier theorem is assumed by
    the probability transform module.
- Acceptance criteria:
  - Characteristic-function results cite Fourier prerequisites instead of assuming them.
  - CLT milestones can import transform statement names without circular dependencies.
- Verification:
  - `rg -n "ANA-T31|Levy|characteristic|transform" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T09 Add Finite And Simple Expectation Core

- Status: Completed (2026-06-12; L2 finite/simple expectation certificates with explicit finite-sum arithmetic premises)
- Depends on: `STAT-T06`
- Areas: `Proofs.Ai.Probability.Expectation.Basic`
- Tasks:
  - Define finite and simple expectation over explicit law packages.
  - Prove linearity, monotonicity, indicator expectation, and LOTUS for finite/simple variables.
  - Keep Lebesgue integral aliases separate until analysis integration foundations are present.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Expectation.Basic`, including
    `FiniteExpectationPackage`, `SimpleExpectationPackage`,
    `IndicatorExpectationSetup`, `finite_expectation_eq_sum`,
    `finite_expectation_linearity_derived`,
    `finite_expectation_monotonicity_derived`,
    `finite_indicator_expectation_derived`, and `finite_lotus_derived`.
  - Linearity, monotonicity, indicator, and LOTUS certificates keep the finite
    weighted-sum arithmetic derivations as explicit premises.
- Acceptance criteria:
  - Linearity and LOTUS are derived from finite sums or simple-function laws.
  - The module exposes reusable lemmas for sample mean, variance, Rao-Blackwell, and Markov inequality.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Expectation.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Expectation.Basic`

### STAT-T10 Add Variance, Covariance, And Correlation Facts

- Status: Completed (2026-06-12; L2 finite moment/variance/covariance certificates with explicit arithmetic and positivity premises)
- Depends on: `STAT-T09`
- Areas: `Proofs.Ai.Probability.Moments.Basic`
- Tasks:
  - Define moments, variance, covariance, and correlation for finite/simple variables.
  - Prove variance decomposition, covariance bilinearity, nonnegativity, and correlation range.
  - Reuse abstract ordered-field and inner-product facts where possible.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Moments.Basic`, including
    `FiniteMomentPackage`, `FiniteVariancePackage`,
    `FiniteCovariancePackage`, `FiniteCorrelationPackage`,
    `finite_variance_decomposition_derived`,
    `finite_covariance_bilinearity_derived`,
    `finite_variance_nonnegative_derived`, and
    `finite_correlation_range_derived`.
  - Correlation range keeps Cauchy-Schwarz and positivity evidence explicit;
    zero-standard-deviation cases are excluded through `Nonzero` premises.
- Acceptance criteria:
  - Correlation range proof cites explicit positivity and Cauchy-Schwarz prerequisites.
  - Theorem statements handle zero-variance side conditions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Moments.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Moments.Basic`

### STAT-T11 Add Markov, Chebyshev, And First Concentration Route

- Status: Completed (2026-06-12; L2 finite Markov/Chebyshev route certificates with explicit arithmetic premises)
- Depends on: `STAT-T09`, `STAT-T10`
- Areas: `Proofs.Ai.Probability.Inequalities.Concentration`
- Tasks:
  - Prove Markov and Chebyshev inequalities for finite/simple variables.
  - Add theorem-card routes for Jensen, Hoeffding, Bernstein, Bennett, and McDiarmid.
  - Separate inequalities that require convexity, independence, or martingales.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Inequalities.Concentration`,
    including `FiniteMarkovInequalityRoute`,
    `finite_markov_inequality_derived`,
    `FiniteChebyshevInequalityRoute`,
    `finite_chebyshev_inequality_derived`, and
    `ConcentrationInequalityRoutePackage`.
  - Markov and Chebyshev keep nonnegativity, positive-radius/threshold, and
    finite arithmetic derivations explicit; Hoeffding/Bernstein/Bennett/
    McDiarmid remain separate route evidence instead of hidden axioms.
- Acceptance criteria:
  - Markov and Chebyshev do not depend on LLN or asymptotic theorems.
  - Later concentration theorems state their missing prerequisites instead of using placeholder axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Inequalities.Concentration`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Inequalities.Concentration`

### STAT-T12 Add Conditional Expectation Interface

- Status: Completed (2026-06-12; L2 finite conditional-expectation certificates plus explicit RN dependency routes)
- Depends on: `STAT-T09`, `STAT-T10`
- Areas: `Proofs.Ai.Probability.ConditionalExpectation.Basic`
- Tasks:
  - Define finite conditional expectation and the general evidence-package interface.
  - Add tower, pull-out, monotonicity, and Jensen statement shapes.
  - Identify the Radon-Nikodym dependency for the general case.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.ConditionalExpectation.Basic`,
    including `FiniteConditionalExpectationPackage`,
    `finite_conditional_expectation_intro`,
    `finite_conditional_expectation_probability_law`,
    `finite_conditional_expectation_sub_event_algebra`,
    `finite_conditional_expectation_sub_event_measurable`,
    `finite_conditional_expectation_identity`,
    `finite_conditional_expectation_tower_derived`,
    `finite_conditional_expectation_pull_out_derived`,
    `finite_conditional_expectation_monotonicity_derived`,
    `finite_conditional_expectation_jensen_derived`,
    `GeneralConditionalExpectationRoutePackage`,
    `general_conditional_expectation_rn_dependency`, and
    `general_conditional_expectation_uniqueness_derived`.
- Acceptance criteria:
  - Finite conditional expectation can be checked independently of RN machinery.
  - General conditional expectation statements carry explicit existence and uniqueness evidence.
- Notes:
  - General conditional expectation and RN-backed uniqueness depend on `ANA-T25` and `ANA-T26`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.ConditionalExpectation.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.ConditionalExpectation.Basic`

### STAT-T13 Add Conditional Expectation Laws And RN/Regular Conditional Split

- Status: Completed (2026-06-12; RN and regular-conditional split recorded against finite CE and measure-owned routes)
- Depends on: `STAT-T12`, `ANA-T26`
- Areas: `Proofs.Ai.Probability.ConditionalExpectation.Regular`
- Tasks:
  - Split Doob-Dynkin, RN representation, regular conditional distributions, and Bayes-by-density formulas.
  - Add import boundaries for martingales, Bayesian statistics, missing data, and causal inference.
  - Document which finite lemmas can be promoted before the general measure route.
- Deliverables:
  - Completed as a dependency split using
    `Proofs.Ai.Probability.ConditionalExpectation.Basic`,
    `Proofs.Ai.Measure.ConditionalExpectation`, and
    `Proofs.Ai.Measure.RadonNikodym`.
  - Finite Bayes remains owned by `STAT-T04`; RN-backed conditional
    expectation and regular/disintegration-style routes remain measure-owned
    until the `ANA-T26`/measure prerequisites are intentionally imported.
- Acceptance criteria:
  - RN-based Bayes formulas are not confused with finite Bayes from `STAT-T04`.
  - Regular conditional distribution statements require explicit standard-Borel or equivalent hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.ConditionalExpectation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.RadonNikodym`
  - `rg -n "Radon|Nikodym|regular conditional|STAT-T04|STAT-T12" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T14 Add Convergence Mode Vocabulary

- Status: Completed (2026-06-12; L2 finite convergence-mode vocabulary certificates)
- Depends on: `STAT-T06`, `STAT-T09`, `ANA-T22`
- Areas: `Proofs.Ai.Probability.Convergence.Basic`
- Tasks:
  - Define almost sure, in probability, in distribution, and Lp convergence statement shapes.
  - Add finite sequence specializations usable before full measure theory.
  - Record topology and metric prerequisites for convergence predicates.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.Convergence.Basic`, including
    `AlmostSureConvergenceMode`, `InProbabilityConvergenceMode`,
    `InDistributionConvergenceMode`, `LpConvergenceMode`,
    `FiniteConvergenceModePackage`, `almost_sure_convergence_intro`,
    `in_probability_convergence_intro`,
    `in_distribution_convergence_intro`, and `lp_convergence_intro`.
- Acceptance criteria:
  - Modes are distinct definitions with explicit probability/metric assumptions.
  - The module does not assert implication chains before their side conditions are available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Convergence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Convergence.Basic`

### STAT-T15 Prove Basic Convergence Implications

- Status: Completed (2026-06-12; L2 finite convergence implication certificates with explicit metric/Markov premises)
- Depends on: `STAT-T14`, `STAT-T11`
- Areas: `Proofs.Ai.Probability.Convergence.Basic`
- Tasks:
  - Prove almost-sure-to-probability and Lp-to-probability implications where prerequisites are present.
  - Prove continuous mapping and Slutsky variants that only need metric/topology foundations.
  - Keep weak-convergence and portmanteau theorems split out.
- Deliverables:
  - Completed in `Proofs.Ai.Probability.Convergence.Basic`, including
    `almost_sure_to_probability_derived`,
    `lp_to_probability_via_markov_derived`,
    `continuous_mapping_in_probability_derived`, and
    `slutsky_finite_probability_derived`.
  - Each implication keeps the metric-tail, Markov route, continuity,
    negligible-term, or algebraic-combination premise explicit.
- Acceptance criteria:
  - Each implication names the exact mode and hypotheses on moments, metrics, or topology.
  - No result imports a later CLT theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Convergence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Convergence.Basic`

### STAT-T16 Add Weak Convergence And Portmanteau Route

- Status: Completed (2026-06-12; measure-owned weak-convergence route verified and probability-side dependency split recorded)
- Depends on: `STAT-T15`, `ANA-T24`, `ANA-T31`
- Areas: `Proofs.Ai.Probability.Convergence.Weak`
- Tasks:
  - Add weak-convergence, Portmanteau, Prokhorov, and Levy continuity theorem routes.
  - Separate metric-space, measure-space, and transform-based proofs.
  - Add cross-links to empirical processes and CLT.
- Deliverables:
  - Completed as a split against `Proofs.Ai.Measure.WeakConvergence`,
    including `WeakConvergenceMeasurePackage`,
    `weak_convergence_tightness_portmanteau_prokhorov_routes`, and
    `skorokhod_wasserstein_vague_late_interfaces`.
  - Levy continuity remains tied to `STAT-T08` plus `ANA-T31`/`ANA-T32` Fourier
    prerequisites; no probability-side weak-convergence theorem assumes it.
- Acceptance criteria:
  - Portmanteau and Prokhorov statements require explicit topological and measure hypotheses.
  - Levy continuity depends on transform/Fourier milestones rather than being assumed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.WeakConvergence`
  - `rg -n "Portmanteau|Prokhorov|Levy|ANA-T31" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T17 Add Borel-Cantelli And Chebyshev WLLN

- Status: Completed (2026-06-12; L2 finite Borel-Cantelli and Chebyshev WLLN route certificates)
- Depends on: `STAT-T05`, `STAT-T11`, `STAT-T15`
- Areas: `Proofs.Ai.Probability.LimitTheorems.LLN`
- Tasks:
  - Prove finite or series-form Borel-Cantelli statements available from existing prerequisites.
  - Prove Chebyshev weak law of large numbers for independent finite-variance variables.
  - Add empirical LLN theorem cards without importing empirical-process machinery.
- Deliverables:
  - Completed with `Proofs.Ai.Probability.LimitTheorems.LLN`, including
    `BorelCantelliFiniteRoute`, `borel_cantelli_finite_derived`,
    `ChebyshevWeakLawRoute`, `chebyshev_weak_law_large_numbers_derived`, and
    `EmpiricalLLNRoutePackage`.
  - The WLLN route imports finite Chebyshev inequality, mutual independence,
    random-variable, and convergence-mode evidence explicitly.
- Acceptance criteria:
  - Chebyshev WLLN proof imports Chebyshev inequality and independence/moment lemmas.
  - SLLN and empirical-process statements remain split until their prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LimitTheorems.LLN`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.LimitTheorems.LLN`

### STAT-T18 Add SLLN And Empirical LLN Split Plan

- Status: Completed (2026-06-12; SLLN and empirical-process split recorded without accepting empirical-process axioms)
- Depends on: `STAT-T17`, `STAT-T16`
- Areas: `Proofs.Ai.Probability.LimitTheorems.LLN`, `Proofs.Ai.Statistics.EmpiricalProcess.Basic`
- Tasks:
  - Split Kolmogorov SLLN, iid SLLN, and empirical-measure convergence routes.
  - Identify maximal inequality, truncation, and Borel-Cantelli prerequisites.
  - Preserve the roadmap distinction between LLN and empirical-process CLT.
- Deliverables:
  - Completed with the `EmpiricalLLNRoutePackage` split in
    `Proofs.Ai.Probability.LimitTheorems.LLN` and roadmap aliases for
    `kolmogorov_slln_route`, `iid_slln_route`, and
    `empirical_measure_convergence_route`.
  - SLLN remains dependent on maximal inequality, truncation, Borel-Cantelli,
    iid/independence, and summability premises; no empirical-process theorem is
    accepted as an axiom.
- Acceptance criteria:
  - No empirical-process theorem is accepted as an axiom to prove SLLN.
  - Each SLLN route states whether it uses iid, independence, identical distribution, or summability.
- Verification:
  - `rg -n "SLLN|Glivenko|Borel|empirical" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T19 Add CLT Statement Interfaces And De Moivre/Lindeberg-Levy Route

- Status: Pending
- Depends on: `STAT-T15`, `STAT-T17`
- Areas: `Proofs.Ai.Probability.LimitTheorems.CLT`
- Tasks:
  - Add CLT statement vocabulary for normalized sums and convergence in distribution.
  - Split De Moivre-Laplace, Lindeberg-Levy, and Lindeberg-Feller theorem routes.
  - Record transform, moment, and independence prerequisites.
- Deliverables:
  - CLT base interface and first theorem-card set.
- Acceptance criteria:
  - The statement interface does not assert normal convergence without proof evidence.
  - De Moivre-Laplace and Lindeberg-Levy routes have separate prerequisite lists.
- Notes:
  - Transform-based CLT proofs depend on `STAT-T08`; non-transform statement setup can start earlier.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LimitTheorems.CLT`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.LimitTheorems.CLT`

### STAT-T20 Add Multivariate CLT, Cramer-Wold, And Delta Method

- Status: Pending
- Depends on: `STAT-T19`, `STAT-T47`, `ANA-T22`
- Areas: `Proofs.Ai.Statistics.Asymptotic.Basic`
- Tasks:
  - Add Cramer-Wold, multivariate CLT, continuous mapping, and Delta method theorem routes.
  - Reuse vector and inner-product modules for finite-dimensional statements.
  - Keep Fréchet derivative variants behind analysis derivative prerequisites.
- Deliverables:
  - Asymptotic basic module or interface for multivariate and Delta method results.
- Acceptance criteria:
  - Multivariate statements identify vector-space and covariance assumptions explicitly.
  - Delta method variants import derivative evidence rather than encoding it as a theorem assumption shortcut.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Asymptotic.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T21 Add Advanced Asymptotic Interface Route

- Status: Pending
- Depends on: `STAT-T20`, `STAT-T28`, `STAT-T58`
- Areas: `Proofs.Ai.Statistics.Asymptotic.Basic`
- Tasks:
  - Add Berry-Esseen, Edgeworth, LAN, Le Cam, and argmax/continuous-mapping advanced theorem cards.
  - Split results by required analytic, likelihood, and information-theoretic prerequisites.
  - Mark high-powered routes as interfaces until the proof foundations are available.
- Deliverables:
  - Advanced asymptotic dependency map with stable statement names.
- Acceptance criteria:
  - LAN and Le Cam results do not appear before likelihood and divergence APIs exist.
  - Edgeworth and Berry-Esseen routes list their moment and transform prerequisites.
- Verification:
  - `rg -n "Berry|Edgeworth|LAN|Le Cam|argmax" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T22 Add Named Distribution Interfaces And Reproductive Laws

- Status: Pending
- Depends on: `STAT-T07`, `STAT-T09`, `STAT-T19`
- Areas: `Proofs.Ai.Probability.Distribution.Named`
- Tasks:
  - Define law packages for Bernoulli, binomial, Poisson, normal, exponential, gamma, beta, chi-square, t, and F.
  - Add theorem-card links for mgf/characteristic functions, moments, and reproductive laws.
  - Keep distribution definitions ordinary, not built into the kernel.
- Deliverables:
  - Named-distribution module interfaces and initial finite/discrete facts.
- Acceptance criteria:
  - Each named distribution has explicit parameter-domain assumptions.
  - Sampling and Bayesian modules import these law packages instead of redefining them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Named`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Distribution.Named`

### STAT-T23 Add Sample Mean And Variance Theorems

- Status: Completed (2026-06-12; L2 finite iid sample mean and variance certificates)
- Depends on: `STAT-T09`, `STAT-T10`
- Areas: `Proofs.Ai.Statistics.SamplingDistribution.Basic`
- Tasks:
  - Prove expectation and variance of sample mean.
  - Prove unbiasedness of sample variance under finite sample assumptions.
  - Add iid-sample vocabulary that later sampling distributions can reuse.
- Deliverables:
  - Completed with `Proofs.Ai.Statistics.SamplingDistribution.Basic`.
  - Added `IidFiniteSamplePackage`, `FiniteSampleMeanPackage`,
    `FiniteSampleMeanVariancePackage`, and
    `FiniteSampleVarianceUnbiasedPackage`.
  - Added certificate-backed projections for iid sample component random
    variables, component means, independence evidence, sample-mean expectation,
    sample-mean variance, and unbiased sample variance.
  - Named-distribution and normal sampling-distribution aliases remain outside
    this milestone and still depend on `STAT-T22` / `STAT-T24`.
- Acceptance criteria:
  - Proofs import expectation and moment lemmas instead of duplicating algebra.
  - Sample-size side conditions are explicit.
- Notes:
  - Named-distribution sampling aliases depend on `STAT-T22`; generic sample mean and variance do not.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.SamplingDistribution.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.SamplingDistribution.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T24 Add Normal Sampling Distributions And Order Statistics Route

- Status: Pending
- Depends on: `STAT-T22`, `STAT-T23`, `STAT-T47`
- Areas: `Proofs.Ai.Statistics.SamplingDistribution.Normal`
- Tasks:
  - Add normal sample mean, chi-square, t, F, Cochran, and order-statistic theorem routes.
  - Split Wishart and multivariate normal dependencies to the multivariate milestones.
  - Record density and independence prerequisites for order-statistic formulas.
- Deliverables:
  - Normal sampling distribution statement module or proof route.
- Acceptance criteria:
  - Univariate sampling results do not duplicate multivariate Wishart statements.
  - Order-statistic formulas require the distribution/density prerequisites they use.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.SamplingDistribution.Normal`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T25 Add Estimator, Sufficiency, And Factorization Core

- Status: Pending
- Depends on: `STAT-T06`, `STAT-T09`, `STAT-T22`
- Areas: `Proofs.Ai.Statistics.Estimation.Sufficiency`
- Tasks:
  - Define estimator, statistic, loss-neutral sufficiency, and likelihood factorization vocabulary.
  - Prove the finite/discrete Fisher-Neyman factorization theorem.
  - Add interfaces for minimal sufficiency and complete sufficiency.
- Deliverables:
  - Sufficiency base module with finite factorization certificate.
- Acceptance criteria:
  - Factorization hypotheses distinguish data law, statistic, parameter, and nuisance-free factors.
  - No completeness or UMVU theorem is assumed in the factorization proof.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Estimation.Sufficiency`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Estimation.Sufficiency`

### STAT-T26 Add Rao-Blackwell And Lehmann-Scheffe Route

- Status: Pending
- Depends on: `STAT-T12`, `STAT-T25`
- Areas: `Proofs.Ai.Statistics.Estimation.Unbiased`
- Tasks:
  - Define unbiasedness, mean-square risk, UMVU, and conditional-improvement vocabulary.
  - Prove finite/discrete Rao-Blackwell theorem.
  - Split Lehmann-Scheffe until completeness support is available.
- Deliverables:
  - Unbiased-estimation module with Rao-Blackwell certificate or finite proof route.
- Acceptance criteria:
  - Rao-Blackwell proof imports conditional expectation and risk definitions explicitly.
  - Lehmann-Scheffe is not accepted without a complete-sufficiency theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Estimation.Unbiased`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T27 Add Basu, Complete Sufficiency, And Exponential Family Split

- Status: Pending
- Depends on: `STAT-T25`, `STAT-T26`, `STAT-T22`
- Areas: `Proofs.Ai.Statistics.Estimation.Sufficiency`
- Tasks:
  - Add complete sufficiency, minimal sufficiency, ancillary statistic, and Basu theorem routes.
  - Add exponential-family natural statistic and conjugacy links.
  - Split proof obligations by finite, dominated, and regular exponential-family assumptions.
- Deliverables:
  - Advanced sufficiency dependency plan with statement names.
- Acceptance criteria:
  - Basu and complete-sufficiency statements import independence and distribution APIs explicitly.
  - Exponential-family aliases do not duplicate Bayesian conjugacy proofs.
- Verification:
  - `rg -n "Basu|complete sufficiency|exponential family|ancillary" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T28 Add Fisher Information And Score Core

- Status: Pending
- Depends on: `STAT-T09`, `STAT-T22`, `ANA-T13`
- Areas: `Proofs.Ai.Statistics.Information.Basic`
- Tasks:
  - Define score, Fisher information, observed information, and KL-neighborhood statement shapes.
  - Add finite or regular dominated examples where derivative and expectation prerequisites are explicit.
  - Connect information vocabulary to likelihood, Cramer-Rao, and asymptotic modules.
- Deliverables:
  - Fisher-information base module or interface.
- Acceptance criteria:
  - Differentiation-under-integral requirements are not hidden in the score definition.
  - Information matrix positivity states exact covariance/expectation assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Information.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Information.Basic`

### STAT-T29 Add Cramer-Rao And Information Inequality Route

- Status: Pending
- Depends on: `STAT-T26`, `STAT-T28`, `STAT-T58`
- Areas: `Proofs.Ai.Statistics.Information.Basic`
- Tasks:
  - Add scalar and matrix Cramer-Rao theorem routes.
  - Add information inequality and efficient-estimator statement names.
  - Separate finite/discrete and regular dominated cases.
- Deliverables:
  - Cramer-Rao proof route with dependency-tagged statement interfaces.
- Acceptance criteria:
  - The route lists unbiasedness, regularity, differentiability, and information invertibility hypotheses.
  - Matrix variants cite linear algebra prerequisites.
- Verification:
  - `rg -n "Cramer-Rao|information inequality|efficient" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T30 Add MLE Consistency And KL-Minimization Route

- Status: Pending
- Depends on: `STAT-T21`, `STAT-T28`, `STAT-T58`
- Areas: `Proofs.Ai.Statistics.Likelihood.MLE`
- Tasks:
  - Define likelihood, log-likelihood, MLE, argmax, identifiability, and KL-minimizer statement shapes.
  - Split finite-parameter consistency from compact/uniform convergence routes.
  - Record imports from information theory and statistical learning.
- Deliverables:
  - MLE consistency dependency route.
- Acceptance criteria:
  - Consistency statements do not assume the argmax theorem they intend to prove.
  - Identifiability and compactness assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Likelihood.MLE`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T31 Add MLE Asymptotics, Wilks, And M/Z-Estimator Route

- Status: Pending
- Depends on: `STAT-T20`, `STAT-T28`, `STAT-T30`
- Areas: `Proofs.Ai.Statistics.Likelihood.MLE`, `Proofs.Ai.Statistics.Estimation.MEstimators`
- Tasks:
  - Add asymptotic normality, Wilks theorem, likelihood-ratio statistic, M-estimator, and Z-estimator routes.
  - Separate differentiability, stochastic equicontinuity, and information invertibility prerequisites.
  - Add aliases for testing and confidence intervals.
- Deliverables:
  - Likelihood asymptotics and M/Z-estimator route.
- Acceptance criteria:
  - Wilks theorem is primary in likelihood theory and only aliased by testing modules.
  - M-estimator and Z-estimator statements do not import each other circularly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Estimation.MEstimators`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T32 Add Test Object, Size, Power, And P-Value Core

- Status: Pending
- Depends on: `STAT-T04`, `STAT-T22`
- Areas: `Proofs.Ai.Statistics.Testing.Basic`
- Tasks:
  - Define hypothesis, test, rejection region, size, level, power, p-value, and randomized test structures.
  - Add finite/discrete examples and monotonic p-value validity statements.
  - Import probability inequalities for Bonferroni aliases.
- Deliverables:
  - Hypothesis-testing base module.
- Acceptance criteria:
  - P-value validity distinguishes null distribution from observed statistic.
  - Multiple-testing Bonferroni aliases point to `STAT-T02` probability inequalities.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Testing.Basic`

### STAT-T33 Add Neyman-Pearson And UMP Route

- Status: Pending
- Depends on: `STAT-T32`
- Areas: `Proofs.Ai.Statistics.Testing.Optimal`
- Tasks:
  - Prove finite simple-vs-simple Neyman-Pearson lemma.
  - Add monotone-likelihood-ratio and Karlin-Rubin theorem routes.
  - Keep complete-class and Bayes-decision links in decision theory.
- Deliverables:
  - Optimal-testing module with finite Neyman-Pearson certificate.
- Acceptance criteria:
  - Neyman-Pearson proof uses explicit size and power definitions from testing core.
  - UMP routes state the ordering and monotone-likelihood-ratio assumptions they need.
- Notes:
  - Regular likelihood-theory aliases can import `STAT-T28` through `STAT-T31`; finite simple-vs-simple NP does not require Fisher information.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Optimal`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T34 Add Asymptotic Test Statistic Route

- Status: Pending
- Depends on: `STAT-T31`, `STAT-T32`
- Areas: `Proofs.Ai.Statistics.Testing.Basic`
- Tasks:
  - Add Wald, score, likelihood-ratio, chi-square, and permutation/asymptotic test routes.
  - Import Wilks theorem as an alias from likelihood theory.
  - Split exact and asymptotic validity claims.
- Deliverables:
  - Asymptotic testing route with stable theorem names.
- Acceptance criteria:
  - Wilks theorem is not reproved or relocated from likelihood theory.
  - Each asymptotic test states the convergence theorem it imports.
- Verification:
  - `rg -n "Wald|score|likelihood-ratio|Wilks|permutation" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T35 Add Multiple Testing And Randomization Route

- Status: Pending
- Depends on: `STAT-T32`, `STAT-T02`
- Areas: `Proofs.Ai.Statistics.Testing.Multiple`, `Proofs.Ai.Statistics.Testing.Randomization`
- Tasks:
  - Add Bonferroni, Holm, Benjamini-Hochberg, randomization, and permutation theorem routes.
  - Import probability inequalities from probability-space basics.
  - Identify independence or positive-dependence assumptions for FDR results.
- Deliverables:
  - Multiple-testing and randomization dependency plan.
- Acceptance criteria:
  - Bonferroni is an alias over probability inequality machinery, not a duplicate proof.
  - FDR statements state the dependency assumptions they require.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Multiple`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Randomization`

### STAT-T36 Add Exact Confidence Interval Core

- Status: Pending
- Depends on: `STAT-T22`, `STAT-T32`
- Areas: `Proofs.Ai.Statistics.Confidence.Basic`
- Tasks:
  - Define interval estimator, coverage, pivot, and exact confidence interval vocabulary.
  - Add normal mean, t, chi-square variance, binomial Clopper-Pearson, and Wilson interval routes.
  - Link inversion of tests without duplicating testing definitions.
- Deliverables:
  - Confidence-interval base module.
- Acceptance criteria:
  - Coverage statements distinguish exact and asymptotic coverage.
  - Test-inversion imports testing core explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Confidence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Confidence.Basic`

### STAT-T37 Add Asymptotic And Score/Likelihood Confidence Routes

- Status: Pending
- Depends on: `STAT-T31`, `STAT-T34`, `STAT-T36`
- Areas: `Proofs.Ai.Statistics.Confidence.Basic`
- Tasks:
  - Add Wald, score, likelihood-ratio, and Delta-method interval routes.
  - Link asymptotic confidence statements to CLT and likelihood modules.
  - Keep simultaneous and bootstrap intervals split.
- Deliverables:
  - Asymptotic confidence interval route.
- Acceptance criteria:
  - Each asymptotic interval names the limiting distribution theorem it imports.
  - Score and likelihood-ratio intervals do not duplicate test-statistic definitions.
- Verification:
  - `rg -n "confidence|Wald|score|likelihood-ratio|Delta" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T38 Add Bootstrap And Simultaneous Confidence Split

- Status: Pending
- Depends on: `STAT-T37`, `STAT-T51`, `STAT-T54`
- Areas: `Proofs.Ai.Statistics.Confidence.Bootstrap`, `Proofs.Ai.Statistics.Confidence.Simultaneous`
- Tasks:
  - Add bootstrap, BCa, simultaneous-band, Scheffe, and Bonferroni interval routes.
  - Separate finite simultaneous inequalities from empirical-process bootstrap arguments.
  - Import multiple-testing probability inequalities where appropriate.
- Deliverables:
  - Bootstrap and simultaneous-confidence dependency plan.
- Acceptance criteria:
  - Bootstrap validity waits for bootstrap/empirical-process prerequisites.
  - Simultaneous interval statements identify whether they are exact, conservative, or asymptotic.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Confidence.Bootstrap`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Confidence.Simultaneous`

### STAT-T39 Add Bayesian Model And Posterior Formula Core

- Status: Pending
- Depends on: `STAT-T04`, `STAT-T06`, `STAT-T22`
- Areas: `Proofs.Ai.Statistics.Bayes.Basic`
- Tasks:
  - Define prior, likelihood, posterior, marginal likelihood, posterior predictive, and Bayes factor.
  - Prove finite/discrete posterior formula as a Bayesian alias of conditional Bayes.
  - Add posterior-risk links only as pending aliases to decision theory.
- Deliverables:
  - Bayesian base module with finite posterior formula.
- Acceptance criteria:
  - Posterior formula imports `STAT-T04` probability Bayes theorem.
  - Bayes risk and Bayes decision rules are not proved in the Bayesian base module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Bayes.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Bayes.Basic`

### STAT-T40 Add Conjugacy Theorem Family

- Status: Pending
- Depends on: `STAT-T22`, `STAT-T39`
- Areas: `Proofs.Ai.Statistics.Bayes.Conjugacy`
- Tasks:
  - Add beta-binomial, gamma-Poisson, normal-normal, normal-inverse-gamma, and Dirichlet-multinomial conjugacy routes.
  - Split finite/discrete conjugacy from density-based dominated conjugacy.
  - Share named distribution law packages with sampling and distribution modules.
- Deliverables:
  - Conjugacy module or dependency-safe theorem route.
- Acceptance criteria:
  - Conjugacy proofs do not redefine named distributions.
  - Parameter-domain and normalization assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Bayes.Conjugacy`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T41 Add Bayesian Asymptotic Consistency Interfaces

- Status: Pending
- Depends on: `STAT-T30`, `STAT-T39`, `STAT-T58`
- Areas: `Proofs.Ai.Statistics.Bayes.Asymptotic`
- Tasks:
  - Add posterior consistency, Bernstein-von Mises, Laplace approximation, and Bayes factor consistency routes.
  - Split KL-support, identifiability, LAN, and regularity prerequisites.
  - Add aliases from likelihood and information theory rather than duplicating them.
- Deliverables:
  - Bayesian asymptotic dependency plan.
- Acceptance criteria:
  - BvM and Laplace statements identify the asymptotic and regularity theorems they need.
  - Posterior consistency does not assume MLE consistency unless explicitly imported.
- Verification:
  - `rg -n "posterior consistency|Bernstein|Laplace|Bayes factor|KL" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T42 Add Posterior-Risk Alias And De Finetti Route

- Status: Pending
- Depends on: `STAT-T39`, `STAT-T79`, `STAT-T81`
- Areas: `Proofs.Ai.Statistics.Bayes.Basic`, `Proofs.Ai.Statistics.Bayes.Asymptotic`
- Tasks:
  - Add posterior-risk and Bayes-action aliases after the decision-risk API exists.
  - Add de Finetti and exchangeability statement routes.
  - Keep decision-theory proofs primary in `STAT-T79` through `STAT-T81`.
- Deliverables:
  - Bayesian-decision alias plan and de Finetti route.
- Acceptance criteria:
  - Posterior-risk aliases import decision risk, loss, and action definitions.
  - De Finetti statements do not depend on posterior-risk aliases.
- Verification:
  - `rg -n "posterior-risk|Bayes action|de Finetti|STAT-T79" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T43 Add Linear Regression Algebra Core

- Status: Pending
- Depends on: `STAT-T09`, `STAT-T10`
- Areas: `Proofs.Ai.Statistics.Regression.Linear`
- Tasks:
  - Define fixed-design regression, design matrix, projection, residual, OLS estimator, and normal equations.
  - Prove core finite-dimensional projection and decomposition lemmas.
  - Reuse vector, inner-product, and spectral modules for linear algebra.
- Deliverables:
  - Linear regression base module with algebraic OLS infrastructure.
- Acceptance criteria:
  - Regression definitions distinguish model assumptions from algebraic least-squares facts.
  - Projection lemmas import checked inner-product facts instead of axiomatizing them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Regression.Linear`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Regression.Linear`

### STAT-T44 Add Gauss-Markov, OLS, And FWL Theorems

- Status: Pending
- Depends on: `STAT-T43`, `STAT-T23`
- Areas: `Proofs.Ai.Statistics.Regression.Linear`
- Tasks:
  - Prove fixed-design finite-dimensional Gauss-Markov theorem.
  - Add OLS unbiasedness, variance, residual orthogonality, and Frisch-Waugh-Lovell theorem.
  - Split normal-error sampling distribution results to sampling distribution modules when needed.
- Deliverables:
  - Linear regression theorem certificates for Gauss-Markov and FWL.
- Acceptance criteria:
  - Gauss-Markov proof states full-rank, linear unbiased estimator, and homoskedastic assumptions.
  - FWL proof is algebraic and does not assume normality.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Regression.Linear`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T45 Add GLM Likelihood And IRLS Route

- Status: Pending
- Depends on: `STAT-T31`, `STAT-T43`
- Areas: `Proofs.Ai.Statistics.Regression.GLM`
- Tasks:
  - Add GLM, link function, exponential-family likelihood, score equations, and IRLS statement routes.
  - Import likelihood asymptotics and derivative prerequisites.
  - Separate algorithmic convergence from statistical consistency.
- Deliverables:
  - GLM dependency route with stable theorem names.
- Acceptance criteria:
  - GLM asymptotics import likelihood theory rather than duplicating MLE proofs.
  - IRLS convergence states optimization assumptions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Regression.GLM`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T46 Add ANOVA And Multiple Comparison Route

- Status: Pending
- Depends on: `STAT-T24`, `STAT-T34`, `STAT-T44`
- Areas: `Proofs.Ai.Statistics.ANOVA.Basic`
- Tasks:
  - Add ANOVA decomposition, F-test, nested model comparison, Tukey HSD, and Scheffe theorem routes.
  - Reuse regression projection lemmas and sampling distribution facts.
  - Split multiple-comparison control from probability inequality aliases.
- Deliverables:
  - ANOVA dependency route and initial decomposition statements.
- Acceptance criteria:
  - ANOVA decompositions are algebraic before distributional assumptions are added.
  - Multiple comparison statements state familywise-error or simultaneous-coverage targets.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.ANOVA.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T47 Add Multivariate Normal Core

- Status: Pending
- Depends on: `STAT-T10`, `STAT-T22`
- Areas: `Proofs.Ai.Statistics.Multivariate.Normal`
- Tasks:
  - Define multivariate normal law package, mean vector, covariance matrix, and linear transform statements.
  - Add marginal, conditional, independence, and quadratic-form theorem routes.
  - Reuse finite-dimensional vector and spectral infrastructure.
- Deliverables:
  - Multivariate normal base module or interface.
- Acceptance criteria:
  - Positive semidefinite covariance assumptions are explicit.
  - Univariate normal aliases import named distribution facts instead of redefining them.
- Notes:
  - Multivariate CLT and Delta-method aliases depend on `STAT-T20`; this core milestone only prepares the law package and finite-dimensional covariance vocabulary.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Multivariate.Normal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Multivariate.Normal`

### STAT-T48 Add Wishart, Hotelling, And MANOVA Route

- Status: Pending
- Depends on: `STAT-T24`, `STAT-T47`
- Areas: `Proofs.Ai.Statistics.Multivariate.Wishart`
- Tasks:
  - Add Wishart distribution, inverse-Wishart, Hotelling T2, canonical correlation, and MANOVA theorem routes.
  - Split normal sampling dependencies from multivariate linear algebra dependencies.
  - Add theorem-card aliases to sampling and Bayesian modules.
- Deliverables:
  - Wishart and multivariate inference dependency plan.
- Acceptance criteria:
  - Wishart results do not duplicate normal sampling distribution statements.
  - MANOVA statements identify rank and covariance assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Multivariate.Wishart`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T49 Add PCA/SVD/Spectral Multivariate Route

- Status: Pending
- Depends on: `STAT-T43`, `STAT-T47`
- Areas: `Proofs.Ai.Statistics.Multivariate.PCA`
- Tasks:
  - Add PCA, SVD, Eckart-Young, principal components, and covariance spectral-decomposition routes.
  - Reuse abstract spectral theorem modules and finite-dimensional inner-product facts.
  - Split probabilistic PCA and factor-analysis statements from algebraic PCA.
- Deliverables:
  - PCA/SVD dependency route.
- Acceptance criteria:
  - Algebraic PCA does not assume random-matrix or asymptotic results.
  - Spectral theorem imports are explicit and checked.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Multivariate.PCA`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T50 Add Random Matrix And Spectral Clustering Interfaces

- Status: Pending
- Depends on: `STAT-T49`, `STAT-T21`
- Areas: `Proofs.Ai.Statistics.Multivariate.PCA`
- Tasks:
  - Add random matrix law, Marchenko-Pastur, Davis-Kahan, and spectral clustering theorem routes.
  - Split high-dimensional asymptotics from finite-dimensional linear algebra.
  - Mark concentration and limiting spectral distribution prerequisites.
- Deliverables:
  - Random-matrix and spectral-clustering interface plan.
- Acceptance criteria:
  - Random matrix statements do not rely on unproved limiting distribution facts.
  - Davis-Kahan imports operator-norm and perturbation prerequisites explicitly.
- Verification:
  - `rg -n "Marchenko|Davis-Kahan|spectral clustering|random matrix" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T51 Add Empirical Distribution And GC/Donsker Route

- Status: Pending
- Depends on: `STAT-T16`, `STAT-T18`, `STAT-T19`
- Areas: `Proofs.Ai.Statistics.Nonparametric.Empirical`
- Tasks:
  - Add empirical distribution, empirical process, Glivenko-Cantelli, and Donsker theorem routes.
  - Separate scalar empirical CDF statements from class-indexed empirical processes.
  - Identify weak-convergence and tightness prerequisites.
- Deliverables:
  - Empirical-process base route.
- Acceptance criteria:
  - Glivenko-Cantelli and Donsker are not conflated with LLN or CLT base modules.
  - Function-class assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Nonparametric.Empirical`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T52 Add Rank And Distribution-Free Test Route

- Status: Pending
- Depends on: `STAT-T32`, `STAT-T51`
- Areas: `Proofs.Ai.Statistics.Nonparametric.Rank`
- Tasks:
  - Add order statistic, rank statistic, sign test, Wilcoxon, Mann-Whitney, and Kolmogorov-Smirnov routes.
  - Separate exact finite permutation arguments from asymptotic approximations.
  - Share testing-core p-value and size definitions.
- Deliverables:
  - Rank and distribution-free testing route.
- Acceptance criteria:
  - Distribution-free statements specify exchangeability or symmetry assumptions.
  - Exact and asymptotic versions have distinct theorem names.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Nonparametric.Rank`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T53 Add Kernel, U-Statistic, And Hoeffding Decomposition Route

- Status: Pending
- Depends on: `STAT-T11`, `STAT-T51`
- Areas: `Proofs.Ai.Statistics.Nonparametric.Kernel`
- Tasks:
  - Add U-statistics, Hoeffding decomposition, kernel density, kernel regression, and bandwidth theorem routes.
  - Split concentration prerequisites from empirical-process prerequisites.
  - Identify finite-sample and asymptotic results separately.
- Deliverables:
  - Kernel and U-statistic dependency plan.
- Acceptance criteria:
  - Hoeffding decomposition route does not duplicate Hoeffding inequality naming.
  - Kernel results state smoothness and bandwidth assumptions.
- Verification:
  - `rg -n "U-statistic|Hoeffding decomposition|kernel|bandwidth" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T54 Add Bootstrap And Jackknife Route

- Status: Pending
- Depends on: `STAT-T37`, `STAT-T51`
- Areas: `Proofs.Ai.Statistics.Nonparametric.Bootstrap`
- Tasks:
  - Add bootstrap consistency, percentile interval, BCa, jackknife bias correction, and bootstrap CLT routes.
  - Split empirical-process assumptions from finite resampling identities.
  - Coordinate confidence-interval aliases with `STAT-T38`.
- Deliverables:
  - Bootstrap and jackknife theorem route.
- Acceptance criteria:
  - Bootstrap confidence aliases are secondary to nonparametric bootstrap validity results.
  - Resampling law assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Nonparametric.Bootstrap`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T55 Add Stationarity And Autocovariance Core

- Status: Pending
- Depends on: `STAT-T10`
- Areas: `Proofs.Ai.Statistics.TimeSeries.Stationary`
- Tasks:
  - Define strict stationarity, weak stationarity, autocovariance, autocorrelation, and white-noise statements.
  - Add finite-dimensional covariance and linear-filter identities.
  - Record filtration and martingale-difference links for later routes.
- Deliverables:
  - Time-series stationarity base module.
- Acceptance criteria:
  - Strict and weak stationarity are distinct predicates.
  - Autocovariance facts import moment and covariance results explicitly.
- Notes:
  - Martingale-difference and time-series CLT items depend on `STAT-T20` and stay in later time-series milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.TimeSeries.Stationary`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.TimeSeries.Stationary`

### STAT-T56 Add ARMA, Spectral, And State-Space Route

- Status: Pending
- Depends on: `STAT-T55`, `ANA-T31`
- Areas: `Proofs.Ai.Statistics.TimeSeries.ARMA`, `Proofs.Ai.Statistics.TimeSeries.Spectral`, `Proofs.Ai.Statistics.TimeSeries.StateSpace`
- Tasks:
  - Add AR, MA, ARMA, ARIMA, Wold decomposition, spectral representation, and Kalman filter routes.
  - Split finite linear-recursion results from spectral/Fourier proofs.
  - Identify stability, invertibility, and observability assumptions.
- Deliverables:
  - Time-series model dependency route.
- Acceptance criteria:
  - Spectral representation waits for Fourier and measure prerequisites.
  - Kalman filter statements distinguish algebraic update identities from optimality theorems.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.TimeSeries.ARMA`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.TimeSeries.Spectral`

### STAT-T57 Add Unit-Root, Cointegration, Mixing, And GARCH Route

- Status: Pending
- Depends on: `STAT-T56`, `STAT-T64`
- Areas: `Proofs.Ai.Statistics.TimeSeries.ARMA`, `Proofs.Ai.Statistics.TimeSeries.StateSpace`
- Tasks:
  - Add unit-root, Dickey-Fuller, cointegration, mixing CLT, ARCH/GARCH, and ergodic theorem routes.
  - Split asymptotic distribution results from model-definition results.
  - Coordinate martingale-difference CLT requirements with martingale milestones.
- Deliverables:
  - Advanced time-series route with explicit blockers.
- Acceptance criteria:
  - Unit-root and cointegration statements identify nonstandard limiting distribution prerequisites.
  - Mixing and GARCH routes do not assume martingale CLTs without imports.
- Verification:
  - `rg -n "unit-root|cointegration|mixing|GARCH|Dickey" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T58 Add Divergence And Information Inequality Core

- Status: Pending
- Depends on: `STAT-T07`, `STAT-T09`
- Areas: `Proofs.Ai.Statistics.InformationTheory.Divergence`
- Tasks:
  - Define KL divergence, total variation, Hellinger distance, mutual information, and entropy interfaces.
  - Prove finite/discrete Gibbs inequality, Pinsker route, data-processing finite route, and chain-rule statements where possible.
  - Add imports for likelihood, Bayesian asymptotics, causal inference, and learning.
- Deliverables:
  - Information-theory divergence base module.
- Acceptance criteria:
  - Divergence definitions identify domination, support, and zero-mass conventions.
  - Finite proofs do not assume measure-theoretic Radon-Nikodym derivatives.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.InformationTheory.Divergence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.InformationTheory.Divergence`

### STAT-T59 Add Entropy, Coding, And Fano Route

- Status: Pending
- Depends on: `STAT-T58`, `STAT-T32`
- Areas: `Proofs.Ai.Statistics.InformationTheory.Coding`
- Tasks:
  - Add Shannon source coding, channel capacity, Fano inequality, Kraft inequality, and AEP routes.
  - Split finite alphabet results from asymptotic equipartition theorems.
  - Add testing and minimax aliases where appropriate.
- Deliverables:
  - Coding and Fano theorem route.
- Acceptance criteria:
  - Fano statements identify the decision/testing setup and error probability assumptions.
  - AEP remains blocked until LLN/ergodic prerequisites are available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.InformationTheory.Coding`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T60 Add Cramer/Sanov/Chernoff Large Deviations

- Status: Pending
- Depends on: `STAT-T21`, `STAT-T58`
- Areas: `Proofs.Ai.Probability.LargeDeviations.Basic`
- Tasks:
  - Add Chernoff bound, Cramer theorem, Sanov theorem, and large-deviation principle interfaces.
  - Split finite exponential-moment Chernoff proofs from full LDP proofs.
  - Connect rate functions to KL divergence and convex duality.
- Deliverables:
  - Large-deviations base route.
- Acceptance criteria:
  - Chernoff bounds identify mgf/exponential-moment assumptions.
  - Sanov and Cramer statements remain dependency-tagged until topology/measure prerequisites are present.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LargeDeviations.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T61 Add Advanced LDP And Transportation Inequality Interfaces

- Status: Pending
- Depends on: `STAT-T60`, `STAT-T75`
- Areas: `Proofs.Ai.Probability.LargeDeviations.Basic`
- Tasks:
  - Add Gartner-Ellis, Varadhan lemma, Donsker-Varadhan, Talagrand, and log-Sobolev theorem routes.
  - Split convex-analysis, topology, and measure prerequisites.
  - Add theorem-card links to learning and concentration.
- Deliverables:
  - Advanced large-deviation and transportation inequality route.
- Acceptance criteria:
  - Gartner-Ellis and Varadhan statements identify differentiability and exponential-tightness hypotheses.
  - Transportation inequalities do not appear as unexplained concentration axioms.
- Verification:
  - `rg -n "Gartner|Varadhan|Talagrand|log-Sobolev|transportation" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T62 Add Filtration And Martingale Core

- Status: Pending
- Depends on: `STAT-T12`, `STAT-T14`
- Areas: `Proofs.Ai.Probability.Martingale.Basic`
- Tasks:
  - Define filtration, adapted process, martingale, submartingale, supermartingale, stopping time, and predictable process.
  - Add finite-time martingale difference and conditional-expectation examples.
  - State links to stochastic approximation and time-series martingale differences.
- Deliverables:
  - Martingale base module.
- Acceptance criteria:
  - Martingale definitions import conditional expectation rather than redefining it.
  - Stopping-time predicates are explicit about the filtration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Martingale.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Martingale.Basic`

### STAT-T63 Add Martingale Inequalities And Optional Stopping

- Status: Pending
- Depends on: `STAT-T11`, `STAT-T62`
- Areas: `Proofs.Ai.Probability.Martingale.Inequalities`
- Tasks:
  - Add Doob maximal inequality, optional stopping, Azuma-Hoeffding, Freedman, and Burkholder theorem routes.
  - Split bounded increments, integrability, uniform integrability, and stopping assumptions.
  - Import finite concentration lemmas where possible.
- Deliverables:
  - Martingale inequality and stopping route.
- Acceptance criteria:
  - Optional stopping statements list stopping-time and integrability hypotheses.
  - Azuma and Freedman results do not duplicate non-martingale Hoeffding inequalities.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Martingale.Inequalities`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T64 Add Martingale Limit, CLT, And Stochastic Approximation Route

- Status: Pending
- Depends on: `STAT-T19`, `STAT-T62`, `STAT-T63`
- Areas: `Proofs.Ai.Probability.Martingale.Limit`, `Proofs.Ai.Statistics.StochasticApproximation.Basic`
- Tasks:
  - Add martingale convergence, martingale CLT, Robbins-Siegmund, Robbins-Monro, and stochastic approximation routes.
  - Separate finite supermartingale convergence from asymptotic approximation results.
  - Add time-series and computation aliases.
- Deliverables:
  - Martingale limit and stochastic approximation route.
- Acceptance criteria:
  - Martingale CLT imports CLT convergence vocabulary and martingale conditions explicitly.
  - Robbins-Monro aliases from computation are secondary to this route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Martingale.Limit`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.StochasticApproximation.Basic`

### STAT-T65 Add Survival Analysis Core

- Status: Pending
- Depends on: `STAT-T09`, `STAT-T28`, `STAT-T32`
- Areas: `Proofs.Ai.Statistics.Survival.Basic`
- Tasks:
  - Define survival function, hazard, censoring, counting process, and risk set statement shapes.
  - Add Kaplan-Meier, Nelson-Aalen, log-rank, and Cox partial likelihood theorem routes.
  - Split finite product-limit identities from martingale/counting-process asymptotics.
- Deliverables:
  - Survival-analysis base route.
- Acceptance criteria:
  - Censoring assumptions are explicit in estimator statements.
  - Cox asymptotics import likelihood and martingale prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Survival.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Survival.Basic`

### STAT-T66 Add Survey Sampling Core

- Status: Pending
- Depends on: `STAT-T10`, `STAT-T32`
- Areas: `Proofs.Ai.Statistics.SurveySampling.Basic`
- Tasks:
  - Define finite population, sampling design, inclusion probabilities, weights, and design unbiasedness.
  - Add Horvitz-Thompson, ratio estimator, stratified sampling, and cluster sampling theorem routes.
  - Split finite design identities from asymptotic survey results.
- Deliverables:
  - Survey sampling theorem route.
- Acceptance criteria:
  - Design-based and model-based expectations are not conflated.
  - Inclusion-probability nonzero assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.SurveySampling.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T67 Add Missing Data Core

- Status: Pending
- Depends on: `STAT-T12`, `STAT-T30`, `STAT-T65`
- Areas: `Proofs.Ai.Statistics.MissingData.Basic`
- Tasks:
  - Define MCAR, MAR, MNAR, observed-data likelihood, ignorability, multiple imputation, and Rubin rules routes.
  - Add EM and IPW cross-links.
  - Split likelihood identities from asymptotic variance and imputation validity.
- Deliverables:
  - Missing-data dependency route.
- Acceptance criteria:
  - Missingness assumptions are explicit predicates, not comments.
  - Rubin rules and ignorability do not assume causal identification results unless imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.MissingData.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T68 Add Causal Identification Core

- Status: Pending
- Depends on: `STAT-T04`, `STAT-T06`, `STAT-T09`
- Areas: `Proofs.Ai.Statistics.Causal.Graphical`, `Proofs.Ai.Statistics.Causal.PotentialOutcome`
- Tasks:
  - Define potential outcomes, treatment, consistency, ignorability, positivity, DAG, d-separation, and do-operator statement shapes.
  - Add back-door, front-door, g-formula, and do-calculus theorem routes.
  - Separate graphical and potential-outcome foundations.
- Deliverables:
  - Causal identification base route.
- Acceptance criteria:
  - Back-door and front-door statements identify conditioning sets and positivity assumptions.
  - Potential-outcome and graphical notation are bridged explicitly rather than merged implicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Causal.Graphical`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Causal.PotentialOutcome`

### STAT-T69 Add IPW, AIPW, And Propensity Score Route

- Status: Pending
- Depends on: `STAT-T10`, `STAT-T12`, `STAT-T68`
- Areas: `Proofs.Ai.Statistics.Causal.Estimation`
- Tasks:
  - Add IPW, AIPW, doubly robust, propensity score balancing, and semiparametric efficiency routes.
  - Import conditional expectation and regression/missing-data prerequisites where needed.
  - Split finite unbiasedness from asymptotic efficiency.
- Deliverables:
  - Causal estimation theorem route.
- Acceptance criteria:
  - Doubly robust statements identify both nuisance-model alternatives.
  - Propensity score balancing is separate from IPW estimator consistency.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Causal.Estimation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T70 Add IV, RD, DiD, Synthetic Control, And Mediation Route

- Status: Pending
- Depends on: `STAT-T44`, `STAT-T68`, `STAT-T69`
- Areas: `Proofs.Ai.Statistics.Causal.Estimation`
- Tasks:
  - Add instrumental variables, LATE, regression discontinuity, difference-in-differences, synthetic control, and mediation theorem routes.
  - Split linear algebra identities from identification assumptions.
  - Add cross-links to regression and time-series where applicable.
- Deliverables:
  - Advanced causal inference route.
- Acceptance criteria:
  - Each design states its identifying assumptions and estimand.
  - Regression-based estimators import regression theorems rather than redefining OLS.
- Verification:
  - `rg -n "instrumental|regression discontinuity|difference-in-differences|synthetic control|mediation" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T71 Add ERM And Uniform Convergence Core

- Status: Pending
- Depends on: `STAT-T17`, `STAT-T58`
- Areas: `Proofs.Ai.Statistics.Learning.ERM`
- Tasks:
  - Define hypothesis class, empirical risk, population risk, ERM, generalization gap, and uniform convergence.
  - Add finite-class union-bound and Hoeffding generalization results.
  - Separate statistical learning risk from decision-theory risk while preserving aliases.
- Deliverables:
  - ERM base module with finite-class generalization certificates where possible.
- Acceptance criteria:
  - Finite-class bounds import probability inequalities from concentration modules.
  - Risk definitions identify sample, hypothesis, loss, and population distribution.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.ERM`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Learning.ERM`

### STAT-T72 Add VC, Sauer-Shelah, And Rademacher Route

- Status: Pending
- Depends on: `STAT-T51`, `STAT-T71`
- Areas: `Proofs.Ai.Statistics.Learning.VC`
- Tasks:
  - Add VC dimension, shattering, Sauer-Shelah, symmetrization, Rademacher complexity, and contraction theorem routes.
  - Split combinatorial finite proofs from empirical-process limits.
  - Identify measurability assumptions for class-indexed suprema.
- Deliverables:
  - VC/Rademacher learning route.
- Acceptance criteria:
  - Sauer-Shelah route is combinatorial and does not assume uniform convergence.
  - Rademacher bounds state boundedness and loss-class assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.VC`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T73 Add Regularization, SVM, Kernel, And PAC-Bayes Route

- Status: Pending
- Depends on: `STAT-T58`, `STAT-T71`, `STAT-T72`
- Areas: `Proofs.Ai.Statistics.Learning.Regularization`, `Proofs.Ai.Statistics.Learning.Kernel`, `Proofs.Ai.Statistics.Learning.PACBayes`
- Tasks:
  - Add ridge, lasso, representer theorem, SVM margin, kernel methods, PAC-Bayes, and oracle inequality routes.
  - Split convex optimization, Hilbert-space, and information-theory prerequisites.
  - Keep PAC-Bayes prior/posterior/sample/risk roles explicit.
- Deliverables:
  - Regularization, kernel, and PAC-Bayes dependency route.
- Acceptance criteria:
  - PAC-Bayes statements distinguish prior, posterior, sample, and risk.
  - Representer theorem imports Hilbert/kernel prerequisites explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.Regularization`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.PACBayes`

### STAT-T74 Add GP, Random Forest, Boosting, And Computation-Dependent Learning Split

- Status: Pending
- Depends on: `STAT-T73`, `STAT-T76`, `STAT-T78`
- Areas: `Proofs.Ai.Statistics.Learning.Kernel`
- Tasks:
  - Add Gaussian process regression, posterior prediction, random forest consistency, boosting margins, and dropout Bayesian-interpretation routes.
  - Split theorem families that require algorithmic convergence, MCMC, or variational inference.
  - Mark computation-dependent results as blocked until computation milestones exist.
- Deliverables:
  - Advanced learning dependency split.
- Acceptance criteria:
  - Gaussian process posterior results import Bayesian and kernel prerequisites.
  - Boosting and random forest statements identify algorithmic assumptions rather than assuming convergence.
- Verification:
  - `rg -n "Gaussian process|random forest|boosting|dropout" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T75 Add Convex/KKT/Duality Statistical Optimization Aliases

- Status: Pending
- Depends on: `ANA-T37`, `STAT-T43`, `STAT-T71`
- Areas: `Proofs.Ai.Statistics.Computation.Optimization`
- Tasks:
  - Define optimization problem, objective, gradient, Hessian, KKT, duality, and convexity aliases used by statistics modules.
  - Reuse analysis optimization and derivative modules instead of creating a separate trusted base.
  - Add aliases for ridge, lasso, SVM, likelihood, and variational objectives.
- Deliverables:
  - Statistical optimization base interface.
- Acceptance criteria:
  - KKT and duality are imported or dependency-tagged, not assumed.
  - Objective definitions distinguish deterministic optimization from stochastic estimators.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.Optimization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Computation.Optimization`

### STAT-T76 Add EM, MM, Newton, Fisher Scoring, And SGD Route

- Status: Pending
- Depends on: `STAT-T30`, `STAT-T64`, `STAT-T75`
- Areas: `Proofs.Ai.Statistics.Computation.EM`
- Tasks:
  - Add EM monotonicity, MM descent, Newton local convergence, Fisher scoring, SGD, and Robbins-Monro aliases.
  - Split deterministic descent proofs from stochastic approximation convergence.
  - Connect missing-data likelihood to EM without circular imports.
- Deliverables:
  - Computation algorithm route.
- Acceptance criteria:
  - EM monotonicity uses Jensen/conditional expectation prerequisites explicitly.
  - SGD convergence imports stochastic approximation results instead of reproving them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.EM`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T77 Add MCMC Invariance And CLT Route

- Status: Pending
- Depends on: `STAT-T16`, `STAT-T39`, `STAT-T55`
- Areas: `Proofs.Ai.Statistics.Computation.MCMC`
- Tasks:
  - Add Markov chain, invariant distribution, detailed balance, Metropolis-Hastings, Gibbs sampling, ergodic theorem, and MCMC CLT routes.
  - Split finite-state invariant proofs from general-state convergence theorems.
  - Add Bayesian posterior sampling aliases.
- Deliverables:
  - MCMC theorem route.
- Acceptance criteria:
  - Detailed balance and invariance are proved before asymptotic sampling claims.
  - General-state CLT statements carry mixing or ergodicity prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.MCMC`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T78 Add Variational Inference, Laplace, And Importance Sampling Route

- Status: Pending
- Depends on: `STAT-T41`, `STAT-T58`, `STAT-T75`
- Areas: `Proofs.Ai.Statistics.Computation.Variational`
- Tasks:
  - Add ELBO identity, KL projection, coordinate-ascent VI, Laplace approximation, importance sampling unbiasedness, and self-normalized importance sampling routes.
  - Split deterministic variational identities from asymptotic approximation validity.
  - Coordinate Bayesian asymptotic and information-theory imports.
- Deliverables:
  - Variational and importance-sampling dependency route.
- Acceptance criteria:
  - ELBO statements identify posterior, variational family, and KL direction.
  - Importance sampling side conditions cover support and finite variance where required.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.Variational`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### STAT-T79 Add Decision Risk Core

- Status: Pending
- Depends on: `STAT-T04`, `STAT-T32`, `STAT-T39`
- Areas: `Proofs.Ai.Statistics.Decision.Basic`
- Tasks:
  - Define action space, loss, risk, Bayes risk, decision rule, admissibility predicate, and minimax risk.
  - Prove finite Bayes risk minimization and finite complete-class starter theorem where feasible.
  - Expose aliases for testing and Bayesian statistics.
- Deliverables:
  - Decision-theory base module with Bayes risk API.
- Acceptance criteria:
  - Bayes risk is primary here, not in conditional probability or Bayesian posterior modules.
  - Loss, prior, posterior, action, and decision rule roles are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Decision.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Decision.Basic`

### STAT-T80 Add Minimax, Complete Class, And Admissibility Route

- Status: Pending
- Depends on: `STAT-T33`, `STAT-T58`, `STAT-T79`
- Areas: `Proofs.Ai.Statistics.Decision.Minimax`, `Proofs.Ai.Statistics.Decision.Admissibility`
- Tasks:
  - Add minimax theorem, least favorable prior, complete class, admissibility, and Blyth method routes.
  - Split finite games from topological compactness/separation theorems.
  - Link Neyman-Pearson and testing decision rules explicitly.
- Deliverables:
  - Minimax and admissibility dependency route.
- Acceptance criteria:
  - Minimax statements identify compactness, convexity, and randomization assumptions.
  - Complete-class results do not assume admissibility conclusions as premises.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Decision.Minimax`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Decision.Admissibility`

### STAT-T81 Add James-Stein, Invariance, And Least Favorable Prior Route

- Status: Pending
- Depends on: `STAT-T47`, `STAT-T79`, `STAT-T80`
- Areas: `Proofs.Ai.Statistics.Decision.Admissibility`
- Tasks:
  - Add invariance, equivariant risk, Hunt-Stein, James-Stein dominance, and least favorable prior routes.
  - Split multivariate normal and quadratic risk prerequisites.
  - Add Bayesian and minimax aliases without relocating primary proofs.
- Deliverables:
  - Advanced decision-theory route.
- Acceptance criteria:
  - James-Stein statements identify dimension and normal-model assumptions.
  - Invariance routes state group-action and equivariance hypotheses.
- Verification:
  - `rg -n "James-Stein|Hunt-Stein|least favorable|equivariant" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T82 Add Distribution Reproductive Laws

- Status: Pending
- Depends on: `STAT-T08`, `STAT-T22`
- Areas: `Proofs.Ai.Probability.Distribution.Reproductive`
- Tasks:
  - Prove or route reproductive laws for binomial, Poisson, normal, gamma, chi-square, t, and F distributions.
  - Share named distribution and transform infrastructure.
  - Add aliases for sampling, Bayesian, and time-series modules.
- Deliverables:
  - Distribution reproductive-law module.
- Acceptance criteria:
  - Reproductive laws do not duplicate sampling distribution results from `STAT-T24`.
  - Transform-based proofs list mgf/characteristic-function prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Reproductive`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Distribution.Reproductive`

### STAT-T83 Add Distribution Relationship And Approximation Route

- Status: Pending
- Depends on: `STAT-T19`, `STAT-T22`, `STAT-T82`
- Areas: `Proofs.Ai.Probability.Distribution.Reproductive`
- Tasks:
  - Add Poisson approximation to binomial, normal approximation, chi-square/gamma links, beta/gamma algebra, and skewness/kurtosis formula routes.
  - Split exact distribution identities from asymptotic approximation theorems.
  - Coordinate CLT and large-deviation imports.
- Deliverables:
  - Distribution relationship and approximation route.
- Acceptance criteria:
  - Approximation statements identify the mode of convergence and error bound when present.
  - Exact identities state parameter-domain assumptions.
- Verification:
  - `rg -n "Poisson approximation|normal approximation|skewness|kurtosis|gamma" proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### STAT-T84 Add Stable And Extreme-Value Theory Interfaces

- Status: Pending
- Depends on: `STAT-T16`, `STAT-T60`, `STAT-T83`
- Areas: `Proofs.Ai.Probability.Distribution.ExtremeValue`, `Proofs.Ai.Probability.Distribution.Stable`
- Tasks:
  - Add stable law, domain of attraction, generalized extreme value, peaks-over-threshold, and regular variation routes.
  - Split weak-convergence, large-deviation, and tail-regularity prerequisites.
  - Keep extreme-value theory separate from named sampling distributions.
- Deliverables:
  - Stable and extreme-value dependency route.
- Acceptance criteria:
  - EVT statements distinguish maxima limits from threshold exceedance models.
  - Stable-law statements identify characteristic-function or domain-of-attraction prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.ExtremeValue`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Stable`

### STAT-T85 Promote Stable Statistics Theorem Closures

- Status: Pending
- Depends on: any completed stable theorem batch from `STAT-T01` through `STAT-T84`
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/*`, `develop/npa-mathlib-next-closure-roadmap.md`
- Tasks:
  - Run closure audit for each stable statistics module or module cluster.
  - Update package metadata, theorem indexes, axiom reports, and publish-plan entries only when the closure is clean.
  - Add promotion notes that identify trusted boundaries and remaining non-promoted theorem families.
- Deliverables:
  - Verified statistics closure ready for `npa-mathlib` promotion.
- Acceptance criteria:
  - Axiom report does not gain unintended axioms.
  - Source-free verifier and package checks pass for the promoted closure.
  - Public closure documentation states which theorem families are included and excluded.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `STQ-001` | theorem-card inventory and duplicate map | `L0` | `STAT-T00` |
| `STQ-002` | probability-space law package and complement rule | `L2` from the first proof attempt | `STAT-T01` |
| `STQ-003` | finite additivity, monotonicity, Boole, Bonferroni | `L2` | `STAT-T02` |
| `STQ-004` | finite conditional probability, multiplication, total probability, Bayes | `L2` | `STAT-T04` |
| `STQ-005` | event independence, pairwise versus mutual independence | `L2` | `STAT-T05` |
| `STQ-006` | random variable and distribution statement API | `L2` or prerequisite split before source edits | `STAT-T06` |
| `STQ-007` | CDF basic properties and monotone-transform formula | `L2` for finite/discrete basics; measure routes after real/measure foundations | `STAT-T07`, `STAT-T08` |
| `STQ-008` | finite/simple expectation linearity and LOTUS | `L2` | `STAT-T09` |
| `STQ-009` | variance, covariance, and correlation range | `L2` | `STAT-T10` |
| `STQ-010` | Markov and Chebyshev inequalities | `L2` | `STAT-T11` |
| `STQ-011` | conditional expectation statement shape and tower property route | `L2` or prerequisite split before source edits | `STAT-T12` |
| `STQ-012` | convergence modes and implication chain through probability convergence | `L2` | `STAT-T14`, then `STAT-T15` |
| `STQ-013` | Chebyshev weak law of large numbers | `L2` | `STAT-T17` |
| `STQ-014` | De Moivre-Laplace or Lindeberg-Levy CLT statement split | `L2` from the first proof attempt | `STAT-T19` |
| `STQ-015` | sample mean expectation and variance | `L2` | `STAT-T23` |
| `STQ-016` | sample variance unbiasedness | `L2` | `STAT-T23` |
| `STQ-017` | Fisher-Neyman factorization theorem finite/discrete case | `L2` | `STAT-T25` |
| `STQ-018` | Rao-Blackwell theorem finite/discrete case | `L2` | `STAT-T26` |
| `STQ-019` | Neyman-Pearson lemma finite simple-vs-simple case | `L2` | `STAT-T33` |
| `STQ-020` | Gauss-Markov theorem fixed-design finite-dimensional case | `L2` | `STAT-T44` |

After `STQ-020`, choose the next branch by project priority:

- `STAT-T28` through `STAT-T31` for Fisher information, Cramer-Rao, MLE, and likelihood asymptotics;
- `STAT-T16`, `STAT-T19`, and `STAT-T20` for stronger weak-convergence and CLT machinery;
- `STAT-T39` through `STAT-T42`, then `STAT-T79` through `STAT-T81`, for Bayesian and decision-theory work;
- `STAT-T51` through `STAT-T54` and `STAT-T71` through `STAT-T74` for empirical-process and learning goals.

## Review Checklist For Each Milestone

- The milestone has one primary roadmap owner and does not duplicate a theorem whose home is documented elsewhere.
- Dependencies point to existing checked modules, earlier statistics tasks, or explicitly deferred analysis tasks.
- Finite/discrete/simple-function results are separated from measure-theoretic and asymptotic results.
- Missing prerequisites are split into blockers before source edits; interface
  packages do not smuggle in the target theorem as an axiom.
- Bayes formula, Bayesian posterior formulas, and Bayes risk/decision rules keep their separate primary homes.
- Probability inequalities used by testing or learning are imported from probability modules when those modules are the primary owner.
- Verification commands check the module being changed; package-wide gates are reserved for package metadata, verifier behavior, promotion, or high-trust changes.
- Generated indexes, replay files, and theorem-search sidecars remain untrusted and are not cited as proof evidence.

## Decision Checkpoints

- Before starting full measure-theoretic probability, confirm the analysis measure/Lebesgue route `ANA-T24` through `ANA-T26` is available or keep the work at interface level.
- Before CLT strengthening through characteristic functions, confirm the transform and Fourier prerequisites from `STAT-T08` and `ANA-T31` through `ANA-T32`.
- Before regression and multivariate work, confirm the intended finite-dimensional scalar/vector interfaces match existing abstract ordered-field, vector, inner-product, and spectral modules.
- Before promotion, run a closure audit and choose package gates according to the scope of the changed artifacts.
