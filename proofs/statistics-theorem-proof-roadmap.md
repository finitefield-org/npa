# Statistics Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided statistics and probability
theorem inventory one theorem at a time in the NPA proof corpus. It is a
planning sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- probability spaces, measure-theoretic foundations, and extension theorems;
- conditional probability, independence, conditional expectation, and Bayes
  formulas;
- random variables, distributions, transforms, expectation, moments, and
  concentration inequalities;
- modes of convergence, law of large numbers, central limit theorems, and
  asymptotic statistics;
- sampling distributions, sufficiency, unbiased estimation, Fisher
  information, maximum likelihood, M-estimation, and testing;
- confidence intervals, Bayesian statistics, regression, ANOVA, GLM, and
  multivariate statistics;
- nonparametric statistics, empirical processes, time series, martingales,
  survival analysis, causal inference, survey sampling, and missing data;
- information theory, large deviations, statistical learning, optimization,
  MCMC, variational inference, decision theory, and named distribution
  families.

The plan is intentionally staged. The first priority is not to encode every
famous theorem name immediately, but to build reusable probability and
statistical-decision foundations whose statements will not need to be replaced
after later asymptotic or model-specific theorems depend on them.

## Existing Baseline

The current proof corpus does not yet expose a dedicated probability or
statistics namespace. Statistics work should therefore reuse existing algebra,
order, vector, metric, analysis, and spectral routes where possible, and should
wait for the analysis roadmap's measure and integration foundations before
claiming measure-theoretic probability results as derived theorems.

Reusable existing corpus modules include:

| Corpus module | Existing role |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractOrderedField` | ordered scalar laws and square-root theorem targets |
| `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge` | bridge between field and ordered-field law packages |
| `Proofs.Ai.Vector.AbstractSpace` | vector-space theorem targets over explicit carriers |
| `Proofs.Ai.Vector.AbstractInnerProduct` | inner-product, norm-square, and vector norm theorem targets |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | checked norm expansion and Cauchy-Schwarz route |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | metric balls, neighborhoods, local predicates, and local uniqueness |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | normed-space law packages, product operations, and product norm estimates |
| `Proofs.Ai.Analysis.AbstractLinearMap` | bounded linear maps, operator bounds, and linear isomorphisms |
| `Proofs.Ai.Analysis.AbstractDerivative` | Frechet derivative, differentiability, uniqueness, and derivative rules |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | completeness evidence, contractions, and Banach fixed-point package |
| `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` | finite-dimensional spectral theorem interface |
| `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem` | Hilbert-space spectral theorem interface with explicit construction evidence |

Planned analysis-roadmap foundations are direct prerequisites for many
statistics milestones:

| Needed foundation | Expected source |
| --- | --- |
| real numbers and sequences | `proofs/analysis-theorem-proof-roadmap.md` `ANA-01` |
| series, continuity, and calculus | `ANA-02` through `ANA-04` |
| Riemann integration | `ANA-05` |
| topology and compactness | `ANA-07` |
| measure and Lebesgue integration | `ANA-08` |
| functional analysis and Hilbert spaces | `ANA-09` |
| Fourier analysis | `ANA-11` |
| ODE and PDE tools | `ANA-12` and `ANA-13` |
| convex and variational optimization | `ANA-14` |

Until those prerequisites exist, probability and statistics milestones may land
as `L0` statement cards or `L1` evidence-package interfaces, but not as fully
derived `L2` measure-theoretic theorems.

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant or shape theorem only | no |
| `L1 Evidence package` | theorem conclusion follows from explicit construction, model, or law evidence | only if explicitly marked as an interface milestone |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

For statistical models, `L1` interfaces are often useful: a model can carry
regularity, domination, differentiability, identifiability, or asymptotic
tightness evidence. Such interfaces must not be confused with derived
theorems. A task is mathematically complete only at `L2` or `L3`, unless the
scope explicitly says that the immediate target is a model-interface wrapper.

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

## Statement Policy

Probability and statistics theorem statements must keep these boundaries
explicit:

- Probability spaces are ordinary structures over measurable spaces and
  measures; they are not kernel primitives.
- Random variables are measurable functions, not a separate trusted term
  language.
- Expectations and conditional expectations are measure/integral constructions
  or evidence packages until the measure roadmap provides enough derived
  infrastructure.
- Independence is a theorem-level predicate over events, sigma-algebras, or
  random variables; pairwise and mutual independence are separate notions.
- Statistical models are explicit families of distributions plus regularity
  evidence. Identifiability, domination, differentiability, Fisher
  information, LAN, and tightness are not implicit typeclass facts.
- Asymptotic theorems must state the convergence mode, indexing scheme,
  normalization, and regularity hypotheses.
- Algorithms such as EM, SGD, IRLS, Kalman filtering, MCMC, and variational
  inference are proof targets about deterministic or stochastic recurrences;
  their implementation is not trusted evidence.

## Duplicate Theorem Policy

Several theorem names appear in multiple inventory sections. Each duplicate
must have one primary home, with other modules importing or aliasing it:

| Theorem family | Primary home |
| --- | --- |
| Bayes probability formula | `STAT-02` for finite and conditional probability; `STAT-14` for Bayesian posterior formulas |
| Bayes risk and Bayes decision rules | `STAT-25`, with finite aliases from `STAT-02` or posterior-risk aliases from `STAT-14` only after the decision-risk API exists |
| Boole, Bonferroni, and multiple-testing Bonferroni | `STAT-01` for probability inequalities; `STAT-12` for testing procedure aliases |
| Jensen, Markov, Chebyshev, Hoeffding, Bernstein, Bennett, McDiarmid | `STAT-04` for expectation and concentration |
| Fubini, Tonelli, Radon-Nikodym, dominated convergence | analysis roadmap `ANA-08`; statistics modules import or specialize them |
| Slutsky, continuous mapping, Delta method, Cramer-Wold | `STAT-08` |
| Borel-Cantelli, LLN, Glivenko-Cantelli | `STAT-07` |
| Donsker, Brownian bridge, empirical process CLT | `STAT-17` |
| Basu, Cochran, Wilks, Neyman-Pearson | `STAT-10` through `STAT-12` with model-specific aliases |
| Gauss-Markov, Frisch-Waugh-Lovell, ANOVA decompositions | `STAT-15` |
| Wishart, Hotelling, MANOVA, PCA, SVD | `STAT-16` |
| Cramer, Sanov, Gartner-Ellis, Chernoff | `STAT-19` |
| Azuma-Hoeffding, Freedman, optional stopping, martingale CLT | `STAT-20` |
| EM monotonicity, Robbins-Monro, stochastic approximation | `STAT-24` |
| Neyman-Pearson and Bayes decision rules | `STAT-12` and `STAT-25`, with explicit cross-links |

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `STAT-00` | inventory and statement policy | theorem cards, duplicate map, target levels |
| `STAT-01` | probability space basics | finite/countable additivity and elementary probability inequalities |
| `STAT-02` | conditional probability and independence | Bayes theorem, total probability, independence API |
| `STAT-03` | random variables and distributions | measurable random variables, CDFs, density and transform formulas |
| `STAT-04` | expectation, moments, and concentration | expectation linearity, variance, Markov and Chebyshev |
| `STAT-05` | conditional expectation | tower property, pull-out property, Doob-Dynkin, RN representation |
| `STAT-06` | convergence of random variables | convergence modes, Slutsky, continuous mapping, Portmanteau route |
| `STAT-07` | laws of large numbers | Borel-Cantelli, WLLN, SLLN, empirical distribution foundations |
| `STAT-08` | CLT and asymptotic tools | classical CLT, Cramer-Wold, Delta method |
| `STAT-09` | named distributions and sampling distributions | normal, chi-square, t, F, Wishart, order-statistic formulas |
| `STAT-10` | sufficiency and unbiased estimation | factorization, Rao-Blackwell, Lehmann-Scheffe, Basu |
| `STAT-11` | information and likelihood theory | Cramer-Rao, Fisher information, MLE, M-estimation |
| `STAT-12` | hypothesis testing | Neyman-Pearson, UMP, p-values, multiple testing |
| `STAT-13` | confidence intervals | normal, t, chi-square, Wilson, likelihood, bootstrap intervals |
| `STAT-14` | Bayesian statistics | conjugacy, posterior risk, BvM, de Finetti, posterior consistency |
| `STAT-15` | regression and ANOVA | Gauss-Markov, OLS, FWL, GLM, ANOVA decompositions |
| `STAT-16` | multivariate statistics | multivariate normal, Wishart, PCA, MANOVA, spectral tools |
| `STAT-17` | nonparametric and empirical processes | Glivenko-Cantelli, Donsker, rank tests, bootstrap |
| `STAT-18` | time series and stochastic processes | stationarity, Wold, spectral representation, ARMA, Kalman |
| `STAT-19` | information theory and large deviations | KL, Pinsker, Fano, Shannon, Sanov, Cramer, Gartner-Ellis |
| `STAT-20` | martingales and stochastic approximation | Doob, optional stopping, Azuma, martingale CLT, Robbins-Siegmund |
| `STAT-21` | survival, survey sampling, and missing data | Kaplan-Meier, Cox, Horvitz-Thompson, Rubin rules |
| `STAT-22` | causal inference | back-door, front-door, g-formula, IPW, doubly robust, IV |
| `STAT-23` | statistical learning | ERM, VC, Rademacher, PAC-Bayes, ridge, lasso, SVM |
| `STAT-24` | statistical computation and optimization | EM, MM, Newton, SGD, MCMC, VI, Laplace, importance sampling |
| `STAT-25` | decision theory | risk, Bayes, minimax, admissibility, James-Stein |
| `STAT-26` | distribution-specific and extreme-value theory | reproductive laws, Poisson limit, stable laws, EVT |
| `STAT-27` | packaging and promotion | stable `npa-mathlib` closure audits |

## STAT-00 Inventory And Statement Policy

- Status: planned.
- Depends on: none.
- Deliverables:
  - Convert the theorem inventory into theorem cards.
  - Give every theorem a stable English identifier, Japanese display name,
    target level, dependencies, target module, and acceptance gate.
  - Mark duplicate theorem names across probability, inference, testing,
    Bayesian statistics, regression, decision theory, information theory, and
    statistical learning.
- Acceptance criteria:
  - Every theorem has one primary home module.
  - Duplicates point to the primary theorem instead of being reproved.
  - Each card states whether the first target is a statement, evidence
    package, derived certificate, or public closure.
- Verification:
  - Documentation diff review.
  - `git diff --check`.

## STAT-01 Probability Space Basics

- Status: planned.
- Depends on: analysis roadmap `ANA-08` for full measure-theoretic probability;
  finite Boolean-event versions may start earlier over explicit event algebras.
- Target modules:
  - `Proofs.Ai.Probability.Space.Basic`
  - `Proofs.Ai.Probability.Space.Extension`
- Theorem order:
  1. probability-space law package;
  2. finite additivity and complement rule;
  3. monotonicity;
  4. Boole inequality;
  5. Bonferroni inequality;
  6. inclusion-exclusion for finite families;
  7. continuity from below and above;
  8. complete probability-space interface;
  9. Caratheodory extension theorem interface or import from measure roadmap;
  10. Kolmogorov and Ionescu-Tulcea extension interfaces.
- Deliverables:
  - Probability-space structures over measurable spaces.
  - Finite-event theorem subset usable before the full measure stack lands.
  - Extension-theorem statement cards with explicit dependency on measure
    construction.
- Acceptance criteria:
  - Countable additivity is ordinary structure or derived measure evidence,
    not a kernel primitive.
  - Finite probability lemmas do not assume countable extension theorems.
  - Extension theorems are marked `L1` until measure construction provides the
    required derived proof route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Space.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Space.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-02 Conditional Probability And Independence

- Status: planned.
- Depends on: `STAT-01`.
- Target modules:
  - `Proofs.Ai.Probability.Conditional.Basic`
  - `Proofs.Ai.Probability.Independence.Basic`
- Theorem order:
  1. conditional probability definition for positive-probability events;
  2. multiplication rule;
  3. total probability formula;
  4. finite Bayes theorem;
  5. prior-posterior relation formulas;
  6. event and random-variable independence;
  7. mutual independence versus pairwise independence;
  8. pi-lambda extension interface.
- Deliverables:
  - Conditional probability API for finite and measure-theoretic settings.
  - Independence predicates with pairwise and mutual forms separated.
  - Cross-links to the `STAT-25` decision-risk API for later Bayes decision
    aliases.
- Acceptance criteria:
  - Division by `P(B)` requires an explicit nonzero hypothesis.
  - Conditional independence does not reuse unconditional independence under a
    hidden assumption.
  - Bayes formula results do not assume a decision-theory risk theorem.
  - Conditional-expectation Bayes variants are split until `STAT-05` is
    available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Conditional.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Independence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-03 Random Variables And Distributions

- Status: planned.
- Depends on: `STAT-01`, `STAT-02`, and analysis roadmap real/measure
  foundations.
- Target modules:
  - `Proofs.Ai.Probability.RandomVariable.Basic`
  - `Proofs.Ai.Probability.Distribution.Basic`
  - `Proofs.Ai.Probability.Distribution.Transform`
- Theorem order:
  1. random variable as measurable function;
  2. law or distribution of a random variable;
  3. distribution function definition and right-continuity;
  4. monotone transform formula;
  5. multivariate transform and Jacobian formula interface;
  6. marginal and conditional distribution formulas;
  7. density, joint density, and marginal density relations;
  8. mixture distribution decomposition;
  9. convolution formula;
  10. probability-generating, moment-generating, and characteristic-function
      uniqueness interfaces;
  11. Levy continuity theorem route.
- Deliverables:
  - Stable random-variable and distribution vocabulary.
  - Transform theorem split plan separating discrete, continuous, and
    multivariate cases.
- Acceptance criteria:
  - CDF theorems explicitly state real-order and measure assumptions.
  - Density formulas do not assume all distributions have densities.
  - Characteristic-function uniqueness waits for Fourier or measure
    prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Distribution.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-04 Expectation, Moments, And Concentration

- Status: planned.
- Depends on: `STAT-03` and analysis roadmap measure/integration foundations;
  finite simple-random-variable results may start earlier.
- Target modules:
  - `Proofs.Ai.Probability.Expectation.Basic`
  - `Proofs.Ai.Probability.Moments.Basic`
  - `Proofs.Ai.Probability.Inequalities.Concentration`
- Theorem order:
  1. expectation of finite/simple random variables;
  2. linearity of expectation;
  3. LOTUS;
  4. expectation formula for nonnegative random variables;
  5. variance, covariance, and correlation formulas;
  6. Jensen, Cauchy-Schwarz, Holder, and Minkowski inequalities by importing
     analysis inequalities where available;
  7. Markov and Chebyshev inequalities;
  8. Chernoff bound;
  9. Hoeffding, Bernstein, Bennett, Azuma-Hoeffding, and McDiarmid
     inequalities in separate batches.
- Deliverables:
  - Expectation and moment API shared by LLN, CLT, regression, and inference.
  - Concentration theorem family with duplicate aliases for large deviations
    and martingales.
- Acceptance criteria:
  - Integrability hypotheses are explicit.
  - Correlation range proof uses variance nonnegativity and Cauchy-Schwarz,
    not an assumed bound.
  - Each concentration inequality states boundedness, independence,
    martingale, or Lipschitz assumptions precisely.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Expectation.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Inequalities.Concentration`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-05 Conditional Expectation

- Status: planned.
- Depends on: `STAT-04` and measure roadmap Radon-Nikodym support.
- Target modules:
  - `Proofs.Ai.Probability.ConditionalExpectation.Basic`
  - `Proofs.Ai.Probability.ConditionalExpectation.Regular`
- Theorem order:
  1. conditional expectation existence interface;
  2. uniqueness up to almost-sure equality;
  3. linearity;
  4. tower property;
  5. pull-out property;
  6. conditional Jensen, Markov, and Chebyshev;
  7. best square approximation theorem;
  8. Doob-Dynkin lemma;
  9. regular conditional probability existence interface;
  10. Radon-Nikodym expression for Bayes formula;
  11. convergence theorem for conditional expectations.
- Deliverables:
  - Conditional expectation API used by martingales, Bayes statistics,
    survival analysis, and causal inference.
- Acceptance criteria:
  - Almost-sure equality is the equality notion for uniqueness.
  - Existence is `L1` until Radon-Nikodym and sigma-algebra infrastructure is
    strong enough for a derived proof.
  - Pull-out property states measurability and integrability hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.ConditionalExpectation.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.ConditionalExpectation.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-06 Convergence Of Random Variables

- Status: planned.
- Depends on: `STAT-03`, `STAT-04`, and analysis roadmap topology/measure
  foundations.
- Target modules:
  - `Proofs.Ai.Probability.Convergence.Basic`
  - `Proofs.Ai.Probability.Convergence.Weak`
- Theorem order:
  1. almost-sure convergence;
  2. convergence in probability;
  3. convergence in distribution;
  4. convergence in mean and mean square;
  5. almost-sure implies in probability;
  6. mean square implies in probability;
  7. convergence in probability implies convergence in distribution;
  8. subsequence almost-sure convergence from convergence in probability;
  9. Slutsky theorem;
  10. continuous mapping theorem;
  11. Portmanteau theorem;
  12. Skorokhod representation, Prokhorov, Helly, and Scheffe theorem
      interfaces.
- Deliverables:
  - Convergence-mode vocabulary shared by LLN, CLT, inference, and learning.
- Acceptance criteria:
  - Each theorem states the exact convergence mode in conclusion and
    hypotheses.
  - Weak convergence theorems wait for topological measure prerequisites.
  - Scheffe theorem and lemma are disambiguated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Convergence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.Convergence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-07 Laws Of Large Numbers

- Status: planned.
- Depends on: `STAT-02`, `STAT-04`, and `STAT-06`.
- Target modules:
  - `Proofs.Ai.Probability.LimitTheorems.LLN`
  - `Proofs.Ai.Statistics.EmpiricalProcess.Basic`
- Theorem order:
  1. Borel-Cantelli lemmas;
  2. Bernoulli law of large numbers;
  3. Chebyshev weak law;
  4. Khintchine weak law interface;
  5. Kolmogorov strong law interface and derived route;
  6. Etemadi and Marcinkiewicz-Zygmund strong laws;
  7. Kolmogorov three-series theorem interface;
  8. Glivenko-Cantelli theorem;
  9. uniform LLN;
  10. VC-type uniform LLN;
  11. ergodic LLN.
- Deliverables:
  - LLN theorem chain from finite-variance WLLN to empirical process targets.
- Acceptance criteria:
  - Independence and identical-distribution assumptions are explicit.
  - WLLN by Chebyshev is the first `L2` target.
  - Strong-law variants are split by assumptions rather than collapsed into a
    single law package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LimitTheorems.LLN`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Probability.LimitTheorems.LLN`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

## STAT-08 CLT And Asymptotic Tools

- Status: planned.
- Depends on: `STAT-06`, `STAT-07`, and characteristic-function or Fourier
  foundations from `STAT-03` and analysis roadmap `ANA-11`.
- Target modules:
  - `Proofs.Ai.Probability.LimitTheorems.CLT`
  - `Proofs.Ai.Statistics.Asymptotic.Basic`
- Theorem order:
  1. De Moivre-Laplace theorem;
  2. Lindeberg-Levy CLT;
  3. Lyapunov CLT;
  4. Lindeberg-Feller CLT;
  5. triangular-array CLT;
  6. multivariate CLT;
  7. Cramer-Wold theorem;
  8. Delta method and multivariate Delta method;
  9. Berry-Esseen theorem;
  10. Edgeworth and Cornish-Fisher expansion interfaces;
  11. Anscombe theorem;
  12. LAN and Le Cam lemmas;
  13. Hajek-Le Cam convolution theorem.
- Deliverables:
  - Central asymptotic theorem module used by inference, regression, GLM,
    time-series, and M-estimation milestones.
- Acceptance criteria:
  - Normalization, centering, variance assumptions, and triangular-array
    indexing are explicit.
  - Delta method imports differentiability from analysis rather than assuming
    differentiable maps implicitly.
  - LAN and Le Cam results are late interfaces until likelihood-process
    vocabulary is stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LimitTheorems.CLT`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Asymptotic.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-09 Named Distributions And Sampling Distributions

- Status: planned.
- Depends on: `STAT-03`, `STAT-04`, `STAT-08`, and algebra/vector foundations.
- Target modules:
  - `Proofs.Ai.Probability.Distribution.Named`
  - `Proofs.Ai.Statistics.SamplingDistribution.Basic`
  - `Proofs.Ai.Statistics.SamplingDistribution.Normal`
- Theorem order:
  1. normal, Poisson, gamma, binomial, beta, Dirichlet, multinomial, and stable
     distribution interfaces;
  2. reproductive laws for named distributions;
  3. sample mean expectation and variance;
  4. sample variance unbiasedness;
  5. normal sample mean distribution;
  6. chi-square, t, and F distribution derivations;
  7. sample mean and sample variance independence under normality;
  8. Wishart distribution interface;
  9. order statistic distribution formulas;
  10. sample median asymptotic distribution.
- Deliverables:
  - Named-distribution theorem base for inference, regression, Bayesian
    conjugacy, ANOVA, and multivariate statistics.
- Acceptance criteria:
  - Distribution laws are ordinary definitions or law packages, not built-in
    kernel constants.
  - Exact finite-sample distribution proofs are separated from asymptotic
    approximations.
  - Wishart and multivariate normal results coordinate with `STAT-16`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Named`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.SamplingDistribution.Normal`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-10 Sufficiency And Unbiased Estimation

- Status: planned.
- Depends on: `STAT-03`, `STAT-04`, `STAT-05`, and `STAT-09`.
- Target modules:
  - `Proofs.Ai.Statistics.Estimation.Sufficiency`
  - `Proofs.Ai.Statistics.Estimation.Unbiased`
- Theorem order:
  1. estimator, bias, risk, and unbiasedness definitions;
  2. Fisher-Neyman factorization theorem;
  3. sufficient statistic existence and minimal sufficiency characterization;
  4. Rao-Blackwell theorem;
  5. complete sufficient statistic theorem;
  6. Lehmann-Scheffe theorem;
  7. Basu theorem;
  8. Pitman-Koopman-Darmois theorem interface;
  9. exponential-family sufficient statistic theorem.
- Deliverables:
  - Estimation vocabulary for later likelihood, Bayesian, and decision theory
    work.
- Acceptance criteria:
  - Sufficient-statistic statements specify domination or factorization
    assumptions.
  - Rao-Blackwell uses conditional expectation rather than assuming risk
    improvement.
  - Completeness is an explicit statistic-family property.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Estimation.Sufficiency`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Estimation.Sufficiency`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-11 Information And Likelihood Theory

- Status: planned.
- Depends on: `STAT-08`, `STAT-10`, and analysis derivative/integration
  foundations.
- Target modules:
  - `Proofs.Ai.Statistics.Information.Basic`
  - `Proofs.Ai.Statistics.Likelihood.MLE`
  - `Proofs.Ai.Statistics.Estimation.MEstimators`
- Theorem order:
  1. Fisher information and score definitions;
  2. information matrix equality;
  3. Fisher information additivity;
  4. Cramer-Rao and multivariate Cramer-Rao inequalities;
  5. Bhattacharyya, Chapman-Robbins, Barankin, Hammersley-Chapman-Robbins, Van
     Trees, and Godambe interfaces;
  6. KL minimization view of MLE;
  7. consistency of MLE;
  8. asymptotic normality and efficiency of MLE;
  9. Wald expansion;
  10. Wilks theorem and likelihood-ratio asymptotics;
  11. score and Wald statistic asymptotics;
  12. AIC derivation and BIC consistency;
  13. consistency and asymptotic normality of M-estimators and Z-estimators.
- Deliverables:
  - Likelihood and information theorem base for testing, confidence sets, GLM,
    survival, and Bayesian asymptotics.
- Acceptance criteria:
  - Regularity conditions are named and explicit.
  - Differentiation under the integral is not assumed without an evidence
    package or derived lemma.
  - Wilks theorem is shared with the testing milestone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Information.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Likelihood.MLE`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-12 Hypothesis Testing

- Status: planned.
- Depends on: `STAT-02`, `STAT-09`, `STAT-10`, and `STAT-11`.
- Target modules:
  - `Proofs.Ai.Statistics.Testing.Basic`
  - `Proofs.Ai.Statistics.Testing.Optimal`
  - `Proofs.Ai.Statistics.Testing.Multiple`
  - `Proofs.Ai.Statistics.Testing.Randomization`
- Theorem order:
  1. tests, size, level, power, and p-value definitions;
  2. Neyman-Pearson lemma;
  3. UMP existence conditions;
  4. monotone likelihood ratio and Karlin-Rubin theorem;
  5. likelihood-ratio, Wald, and score test properties;
  6. Wilks theorem alias from `STAT-11`;
  7. p-value uniformity and validity;
  8. Bonferroni, Holm, Hochberg, Benjamini-Hochberg, Benjamini-Yekutieli, and
     closed-testing procedures;
  9. permutation and randomization test exactness or validity.
- Deliverables:
  - Testing theorem family separated into optimal, asymptotic, multiple, and
    randomization tracks.
- Acceptance criteria:
  - Null and alternative model classes are explicit.
  - p-value theorems state whether they are exact, conservative, or asymptotic.
  - FWER and FDR statements do not share one ambiguous error-rate predicate.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Testing.Optimal`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

## STAT-13 Confidence Intervals

- Status: planned.
- Depends on: `STAT-09`, `STAT-11`, and `STAT-12`.
- Target modules:
  - `Proofs.Ai.Statistics.Confidence.Basic`
  - `Proofs.Ai.Statistics.Confidence.Bootstrap`
  - `Proofs.Ai.Statistics.Confidence.Simultaneous`
- Theorem order:
  1. coverage probability and confidence set definitions;
  2. normal mean confidence interval;
  3. t interval for unknown variance;
  4. chi-square variance interval;
  5. two-sample mean difference interval;
  6. Welch approximation interface;
  7. Wald, Wilson, and Clopper-Pearson intervals for proportions;
  8. likelihood-ratio and score confidence interval asymptotics;
  9. bootstrap percentile, bootstrap-t, and BCa intervals;
  10. Bonferroni, Scheffe, Tukey HSD, and simultaneous confidence intervals.
- Deliverables:
  - Coverage-theorem layer connected to sampling distributions and testing.
- Acceptance criteria:
  - Exact and asymptotic coverage are separate predicates.
  - Approximation theorems state limiting regime and error terms.
  - Bootstrap interval sub-batches depend on bootstrap consistency from
    `STAT-17` and should be split if that prerequisite is not available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Confidence.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Statistics.Confidence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-14 Bayesian Statistics

- Status: planned.
- Depends on: `STAT-02`, `STAT-03`, `STAT-09`, `STAT-10`, and `STAT-11`.
- Target modules:
  - `Proofs.Ai.Statistics.Bayes.Basic`
  - `Proofs.Ai.Statistics.Bayes.Conjugacy`
  - `Proofs.Ai.Statistics.Bayes.Asymptotic`
- Theorem order:
  1. posterior distribution formula;
  2. conjugate prior theorem;
  3. beta-binomial, gamma-Poisson, normal-normal, normal-inverse-gamma, and
     Dirichlet-multinomial conjugacy;
  4. posterior predictive distribution formula;
  5. posterior mean under squared loss and posterior median under absolute
     loss;
  6. MAP and regularization correspondence;
  7. Bernstein-von Mises theorem;
  8. Doob and Schwartz posterior consistency interfaces;
  9. Bayes factor consistency and Savage-Dickey density ratio;
  10. exchangeability and de Finetti theorem;
  11. Blackwell-Dubins merging of opinions;
  12. posterior-risk aliases after the `STAT-25` decision-risk API is
      available.
- Deliverables:
  - Bayesian model and posterior API with finite conjugacy as early derived
    targets.
  - Posterior-risk theorem aliases that can later import the decision-theory
    risk API from `STAT-25`.
- Acceptance criteria:
  - Priors, likelihoods, and domination assumptions are explicit.
  - MAP regularization correspondence states the penalty-prior relation.
  - BvM and consistency theorems wait for asymptotic likelihood foundations.
  - Posterior-risk optimality results are aliases or follow-ups after
    `STAT-25`, not primary Bayesian foundation assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Bayes.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Bayes.Conjugacy`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-15 Regression And ANOVA

- Status: planned.
- Depends on: `STAT-04`, `STAT-08`, `STAT-09`, `STAT-11`, `STAT-12`, and
  existing vector/inner-product/linear-map modules.
- Target modules:
  - `Proofs.Ai.Statistics.Regression.Linear`
  - `Proofs.Ai.Statistics.Regression.GLM`
  - `Proofs.Ai.Statistics.ANOVA.Basic`
- Theorem order:
  1. normal equations;
  2. least-squares estimator existence conditions;
  3. Gauss-Markov theorem and BLUE properties;
  4. OLS unbiasedness, variance, consistency, and asymptotic normality;
  5. residual sum-of-squares and R-squared decompositions;
  6. Frisch-Waugh-Lovell theorem;
  7. hat matrix, VIF, Cook distance, and robust covariance estimators;
  8. GLM exponential-family base and score equations;
  9. IRLS derivation;
  10. logistic and Poisson regression MLE;
  11. deviance asymptotic chi-square and GLM tests;
  12. ANOVA square-sum decompositions, F tests, interactions, and multiple
      comparison procedures.
- Deliverables:
  - Linear-model theorem base that reuses certified linear algebra instead of
    model-specific matrix axioms.
- Acceptance criteria:
  - Rank and design-matrix assumptions are explicit.
  - Gauss-Markov distinguishes fixed-design, homoskedastic, and unbiasedness
    hypotheses.
  - GLM asymptotics import likelihood regularity from `STAT-11`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Regression.Linear`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.ANOVA.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-16 Multivariate Statistics

- Status: planned.
- Depends on: `STAT-08`, `STAT-09`, `STAT-11`, and existing spectral/linear
  algebra modules.
- Target modules:
  - `Proofs.Ai.Statistics.Multivariate.Normal`
  - `Proofs.Ai.Statistics.Multivariate.Wishart`
  - `Proofs.Ai.Statistics.Multivariate.PCA`
- Theorem order:
  1. multivariate normal basic, marginal, conditional, and linear-transform
     theorems;
  2. Wishart distribution theorem;
  3. Hotelling T-squared distribution;
  4. Wilks Lambda and MANOVA test theorem;
  5. PCA variance maximization theorem;
  6. SVD existence and Eckart-Young-Mirsky theorem;
  7. sample covariance matrix properties;
  8. canonical correlation theorem;
  9. Fisher discriminant optimality;
  10. Mahalanobis distance properties;
  11. multivariate Delta method alias from `STAT-08`;
  12. Marchenko-Pastur, Tracy-Widom, Davis-Kahan, Perron-Frobenius, and
      spectral clustering consistency interfaces.
- Deliverables:
  - Multivariate statistics layer connected to linear algebra and spectral
    theorem routes.
- Acceptance criteria:
  - Matrix dimensions, rank, and positive-definiteness assumptions are
    explicit.
  - PCA and SVD routes reuse spectral theorem modules.
  - Random matrix limits are late interfaces until measure/asymptotic
    prerequisites are stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Multivariate.Normal`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Multivariate.PCA`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-17 Nonparametric And Empirical Processes

- Status: planned.
- Depends on: `STAT-06`, `STAT-07`, `STAT-08`, `STAT-09`, and measure/topology
  foundations.
- Target modules:
  - `Proofs.Ai.Statistics.Nonparametric.Empirical`
  - `Proofs.Ai.Statistics.Nonparametric.Rank`
  - `Proofs.Ai.Statistics.Nonparametric.Bootstrap`
  - `Proofs.Ai.Statistics.Nonparametric.Kernel`
- Theorem order:
  1. empirical distribution consistency;
  2. Glivenko-Cantelli theorem alias or specialization;
  3. Donsker theorem and Brownian bridge convergence interface;
  4. Kolmogorov-Smirnov, Cramer-von Mises, and Anderson-Darling tests;
  5. Wilcoxon, Mann-Whitney, Kruskal-Wallis, Friedman, sign test, Spearman,
     and Kendall theorem families;
  6. kernel density consistency and asymptotic normality;
  7. Nadaraya-Watson and local linear regression properties;
  8. U-statistic asymptotic normality and Hoeffding decomposition;
  9. jackknife and bootstrap consistency;
  10. Efron bootstrap theorem.
- Deliverables:
  - Empirical-process and resampling theorem layer for confidence intervals,
    testing, and statistical learning.
- Acceptance criteria:
  - Rank tests state exchangeability or null distribution assumptions.
  - Bootstrap results state the bootstrap scheme and convergence mode.
  - Donsker theorem is not used as a generic empirical-process axiom.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Nonparametric.Empirical`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Nonparametric.Bootstrap`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-18 Time Series And Stochastic Processes

- Status: planned.
- Depends on: `STAT-04`, `STAT-08`, analysis Fourier foundations, and linear
  algebra.
- Target modules:
  - `Proofs.Ai.Statistics.TimeSeries.Stationary`
  - `Proofs.Ai.Statistics.TimeSeries.ARMA`
  - `Proofs.Ai.Statistics.TimeSeries.Spectral`
  - `Proofs.Ai.Statistics.TimeSeries.StateSpace`
- Theorem order:
  1. stationarity and weak stationarity;
  2. autocovariance properties;
  3. Wold decomposition interface;
  4. Herglotz theorem and spectral representation;
  5. ARMA stationarity and invertibility conditions;
  6. Yule-Walker equations and Levinson-Durbin algorithm;
  7. unit-root process facts and Dickey-Fuller-style limits;
  8. Granger representation and Johansen cointegration interface;
  9. VAR stability;
  10. Kalman filter and Rauch-Tung-Striebel smoother recursions;
  11. martingale-difference and mixing CLTs;
  12. ergodic theorem and Ljung-Box asymptotics;
  13. ARCH/GARCH stationarity conditions.
- Deliverables:
  - Stochastic-process theorem base for time-series models.
- Acceptance criteria:
  - Indexing, filtration, stationarity, and mixing assumptions are explicit.
  - Spectral theorems reuse Fourier and measure foundations.
  - Filtering recursions are algebraic theorem targets, not trusted
    algorithms.
  - Martingale-difference CLT items depend on `STAT-20` and are split if the
    martingale foundation is not available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.TimeSeries.Stationary`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.TimeSeries.ARMA`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-19 Information Theory And Large Deviations

- Status: planned.
- Depends on: `STAT-03`, `STAT-04`, `STAT-08`, `STAT-12`, and analysis
  measure/integration foundations.
- Target modules:
  - `Proofs.Ai.Statistics.InformationTheory.Divergence`
  - `Proofs.Ai.Statistics.InformationTheory.Coding`
  - `Proofs.Ai.Probability.LargeDeviations.Basic`
- Theorem order:
  1. Gibbs inequality and KL nonnegativity;
  2. Pinsker inequality;
  3. Jensen-Shannon, Hellinger, total variation, chi-square, and f-divergence
     properties;
  4. data processing inequality for f-divergence;
  5. mutual information nonnegativity;
  6. entropy and mutual-information chain rules;
  7. Fano inequality;
  8. Kraft-McMillan inequality;
  9. Shannon source and channel coding theorem interfaces;
  10. asymptotic equipartition property;
  11. Cramer theorem, Chernoff theorem, Sanov theorem;
  12. Varadhan lemma, Gartner-Ellis theorem, contraction principle;
  13. Mogulskii and Donsker-Varadhan interfaces;
  14. Bahadur efficiency and transportation/Talagrand inequalities.
- Deliverables:
  - Divergence and large-deviation theorem layer for MLE, learning, testing,
    and information criteria.
- Acceptance criteria:
  - Absolute-continuity requirements for divergences are explicit.
  - Coding theorems are late interfaces until sequence and entropy
    infrastructure is stable.
  - Large-deviation principles name rate functions and topological spaces.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.InformationTheory.Divergence`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.LargeDeviations.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-20 Martingales And Stochastic Approximation

- Status: planned.
- Depends on: `STAT-05`, `STAT-06`, `STAT-08`, and filtrations from
  probability foundations.
- Target modules:
  - `Proofs.Ai.Probability.Martingale.Basic`
  - `Proofs.Ai.Probability.Martingale.Inequalities`
  - `Proofs.Ai.Probability.Martingale.Limit`
  - `Proofs.Ai.Statistics.StochasticApproximation.Basic`
- Theorem order:
  1. filtration, adaptedness, martingale, submartingale, and supermartingale
     definitions;
  2. Doob maximal inequality;
  3. martingale convergence theorem;
  4. optional stopping and optional sampling;
  5. Doob decomposition;
  6. Azuma-Hoeffding and Freedman inequalities;
  7. Burkholder-Davis-Gundy interface;
  8. predictable quadratic variation;
  9. martingale CLT and strong law;
  10. Robbins-Siegmund convergence theorem;
  11. stochastic approximation convergence theorem;
  12. Doob-Meyer and Girsanov interfaces.
- Deliverables:
  - Martingale theorem family used by concentration, time-series, survival,
    MCMC, and stochastic approximation.
- Acceptance criteria:
  - Filtration and stopping-time assumptions are explicit.
  - Optional stopping variants state boundedness or integrability conditions.
  - Continuous-time theorems stay `L1` until stochastic-process foundations
    justify them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Martingale.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Martingale.Inequalities`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

## STAT-21 Survival, Survey Sampling, And Missing Data

- Status: planned.
- Depends on: `STAT-04`, `STAT-08`, `STAT-11`, `STAT-12`, `STAT-17`, and
  `STAT-20`.
- Target modules:
  - `Proofs.Ai.Statistics.Survival.Basic`
  - `Proofs.Ai.Statistics.SurveySampling.Basic`
  - `Proofs.Ai.Statistics.MissingData.Basic`
- Theorem order:
  1. survival function, hazard, and cumulative hazard relations;
  2. Kaplan-Meier and Nelson-Aalen consistency interfaces;
  3. Greenwood formula;
  4. log-rank asymptotics;
  5. Cox partial likelihood and Cox estimator consistency/asymptotic
     normality;
  6. Aalen additive hazards and competing-risks/Fine-Gray interfaces;
  7. Horvitz-Thompson and Hansen-Hurwitz unbiasedness;
  8. ratio estimator approximate bias, stratified and cluster variance
     formulas, Neyman allocation, PPS, calibration, finite-population CLT;
  9. MCAR, MAR, MNAR, Rubin missing mechanism, complete-case consistency, IPW,
     multiple imputation validity, Rubin rules, EM convergence, data
     augmentation, pattern-mixture and selection-model identifiability.
- Deliverables:
  - Applied-statistics theorem families layered over core inference and
    martingale tools.
- Acceptance criteria:
  - Censoring and at-risk process assumptions are explicit in survival
    theorems.
  - Survey results distinguish design-based and model-assisted assumptions.
  - Missing-data results state the missingness mechanism and identifiability
    target.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Survival.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.SurveySampling.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-22 Causal Inference

- Status: planned.
- Depends on: `STAT-02`, `STAT-03`, `STAT-04`, `STAT-10`, `STAT-11`, and
  `STAT-15`.
- Target modules:
  - `Proofs.Ai.Statistics.Causal.Graphical`
  - `Proofs.Ai.Statistics.Causal.PotentialOutcome`
  - `Proofs.Ai.Statistics.Causal.Estimation`
- Theorem order:
  1. potential outcome and counterfactual model interfaces;
  2. exchangeability, consistency, and positivity;
  3. back-door criterion;
  4. front-door criterion;
  5. do-calculus rules;
  6. g-formula;
  7. IPW, doubly robust, and AIPW estimator properties;
  8. propensity score balance and Rosenbaum-Rubin theorem;
  9. instrumental-variable identification and Wald estimator theorem;
  10. LATE theorem;
  11. regression discontinuity, difference-in-differences, synthetic control,
      and mediation identification interfaces.
- Deliverables:
  - Causal identification and estimator theorem layer.
  - Cross-links to `STAT-21` for missing-data and censoring variants.
- Acceptance criteria:
  - Identification theorems are separated from estimator consistency theorems.
  - Graphical, potential-outcome, and structural-model assumptions are not
    silently interchanged.
  - Positivity assumptions are explicit wherever conditioning or weighting is
    used.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Causal.Graphical`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Causal.Estimation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-23 Statistical Learning

- Status: planned.
- Depends on: `STAT-07`, `STAT-08`, `STAT-15`, `STAT-17`, `STAT-19`,
  and analysis optimization foundations.
- Target modules:
  - `Proofs.Ai.Statistics.Learning.ERM`
  - `Proofs.Ai.Statistics.Learning.VC`
  - `Proofs.Ai.Statistics.Learning.Regularization`
  - `Proofs.Ai.Statistics.Learning.Kernel`
  - `Proofs.Ai.Statistics.Learning.PACBayes`
- Theorem order:
  1. empirical risk minimization consistency;
  2. uniform convergence theorem;
  3. VC-dimension generalization bound;
  4. Sauer-Shelah lemma;
  5. Vapnik-Chervonenkis theorem;
  6. Rademacher complexity bound;
  7. PAC learnability theorem and no-free-lunch theorem;
  8. bias-variance decomposition and cross-validation consistency;
  9. ridge closed-form solution;
  10. lasso KKT and oracle-property interfaces;
  11. elastic net properties;
  12. SVM dual theorem, representer theorem, and kernel trick;
  13. Gaussian-process posterior formula;
  14. random forest consistency and boosting margin theory;
  15. AdaBoost and exponential loss;
  16. dropout approximate Bayesian interpretation interface;
  17. SGD convergence and stochastic approximation theorem alias;
  18. PAC-Bayes bound.
- Deliverables:
  - Learning-theory theorem layer linked to empirical processes, optimization,
    and information divergences.
- Acceptance criteria:
  - Hypothesis class, loss, measurability, and capacity assumptions are
    explicit.
  - Optimization statements are not treated as statistical consistency
    theorems unless estimation error and optimization error are both stated.
  - Computation-dependent learning statements, including SGD-specific results,
    depend on `STAT-24` and are split if that milestone is not available.
  - PAC-Bayes statements distinguish prior, posterior, sample, and risk.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.ERM`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Learning.VC`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-24 Statistical Computation And Optimization

- Status: planned.
- Depends on: analysis optimization foundations, `STAT-11`, `STAT-14`,
  and `STAT-20`.
- Target modules:
  - `Proofs.Ai.Statistics.Computation.Optimization`
  - `Proofs.Ai.Statistics.Computation.EM`
  - `Proofs.Ai.Statistics.Computation.MCMC`
  - `Proofs.Ai.Statistics.Computation.Variational`
- Theorem order:
  1. convex first-order and second-order conditions;
  2. KKT, duality, and Fenchel duality aliases from analysis optimization;
  3. EM and MM monotonicity;
  4. Newton local quadratic convergence;
  5. Fisher scoring convergence interface;
  6. SGD convergence;
  7. Robbins-Monro theorem alias from `STAT-20`;
  8. Metropolis-Hastings detailed balance;
  9. Gibbs sampler invariant distribution;
  10. Markov-chain ergodic theorem and MCMC CLT;
  11. Hamiltonian Monte Carlo invariance interface;
  12. ELBO decomposition;
  13. coordinate-ascent VI monotonicity;
  14. Laplace approximation theorem;
  15. importance sampling unbiasedness.
- Deliverables:
  - Computation theorem layer that supports likelihood, Bayesian, missing-data,
    and learning workflows.
- Acceptance criteria:
  - Algorithms are modeled as recurrences or transition kernels with explicit
    assumptions.
  - MCMC invariance is separated from convergence to stationarity.
  - VI monotonicity does not imply posterior consistency unless separately
    proved.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.EM`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Computation.MCMC`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-25 Decision Theory

- Status: planned.
- Depends on: `STAT-02`, `STAT-10`, `STAT-12`, `STAT-14`, and `STAT-19`.
- Target modules:
  - `Proofs.Ai.Statistics.Decision.Basic`
  - `Proofs.Ai.Statistics.Decision.Minimax`
  - `Proofs.Ai.Statistics.Decision.Admissibility`
- Theorem order:
  1. risk function and randomized decision rules;
  2. Bayes risk minimization theorem;
  3. sufficient statistic risk improvement;
  4. minimax theorem;
  5. Wald complete class theorem;
  6. admissibility theorem;
  7. James-Stein domination theorem;
  8. Stein lemma;
  9. Hunt-Stein theorem and complete class theorem interface;
  10. invariance principle and equivariant estimator optimality;
  11. least favorable prior theorem.
- Deliverables:
  - Decision-theory API shared by testing, Bayesian statistics, and shrinkage
    estimation.
- Acceptance criteria:
  - Loss, action, experiment, and parameter spaces are explicit.
  - Minimax theorem states compactness, convexity, or topological assumptions.
  - James-Stein result is separated from general admissibility vocabulary.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Decision.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Statistics.Decision.Minimax`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-26 Distribution-Specific And Extreme-Value Theory

- Status: planned.
- Depends on: `STAT-03`, `STAT-04`, `STAT-08`, `STAT-09`, and `STAT-19`.
- Target modules:
  - `Proofs.Ai.Probability.Distribution.Reproductive`
  - `Proofs.Ai.Probability.Distribution.ExtremeValue`
  - `Proofs.Ai.Probability.Distribution.Stable`
- Theorem order:
  1. reproductive laws for normal, Poisson, gamma, and binomial
     distributions;
  2. Poisson limit theorem and normal approximation theorem;
  3. chi-square, t, F, beta-gamma, Dirichlet-gamma, and multinomial
     relationships;
  4. stable distribution properties;
  5. extreme value theorem;
  6. Fisher-Tippett-Gnedenko theorem;
  7. Pickands-Balkema-de Haan theorem;
  8. generalized extreme-value and generalized Pareto limit theorem
     interfaces;
  9. Maxwell theorem.
- Deliverables:
  - Distribution-specific theorem track that feeds sampling, Bayesian
    conjugacy, multivariate statistics, and extreme-value applications.
- Acceptance criteria:
  - Parameterization choices are recorded before proofs depend on them.
  - Extreme-value theorems state domain-of-attraction assumptions.
  - Named distribution aliases do not duplicate `STAT-09` sampling results.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.Reproductive`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Probability.Distribution.ExtremeValue`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## STAT-27 Packaging And Promotion

- Status: planned.
- Depends on: any completed stable theorem batch.
- Deliverables:
  - Select closed theorem sets for `npa-mathlib` promotion.
  - Write closure audits before materialization.
  - Keep source-free package verification, package hash checks, theorem index
    checks, and axiom report checks as public package acceptance criteria.
- Acceptance criteria:
  - The promoted module has stable names and statement shape.
  - The import closure does not drag staging modules into public
    `npa-mathlib`.
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
| `STQ-001` | theorem-card inventory and duplicate map | `L0` | `STAT-00` |
| `STQ-002` | probability-space law package and complement rule | `L1` then `L2` | `STAT-01` |
| `STQ-003` | finite additivity, monotonicity, Boole, Bonferroni | `L2` | `STAT-01` |
| `STQ-004` | finite conditional probability, multiplication, total probability, Bayes | `L2` | `STAT-02` |
| `STQ-005` | event independence, pairwise versus mutual independence | `L2` | `STAT-02` |
| `STQ-006` | random variable and distribution statement API | `L1` | `STAT-03` |
| `STQ-007` | CDF basic properties and monotone-transform formula | `L2` after real/measure foundations | `STAT-03` |
| `STQ-008` | finite/simple expectation linearity and LOTUS | `L2` | `STAT-04` |
| `STQ-009` | variance, covariance, and correlation range | `L2` | `STAT-04` |
| `STQ-010` | Markov and Chebyshev inequalities | `L2` | `STAT-04` |
| `STQ-011` | conditional expectation statement shape and tower property interface | `L1` | `STAT-05` |
| `STQ-012` | convergence modes and implication chain through probability convergence | `L2` | `STAT-06` |
| `STQ-013` | Chebyshev weak law of large numbers | `L2` | `STAT-07` |
| `STQ-014` | De Moivre-Laplace or Lindeberg-Levy CLT statement split | `L1` then `L2` | `STAT-08` |
| `STQ-015` | sample mean expectation and variance | `L2` | `STAT-09` |
| `STQ-016` | sample variance unbiasedness | `L2` | `STAT-09` |
| `STQ-017` | Fisher-Neyman factorization theorem finite/discrete case | `L2` | `STAT-10` |
| `STQ-018` | Rao-Blackwell theorem finite/discrete case | `L2` | `STAT-10` |
| `STQ-019` | Neyman-Pearson lemma finite simple-vs-simple case | `L2` | `STAT-12` |
| `STQ-020` | Gauss-Markov theorem fixed-design finite-dimensional case | `L2` | `STAT-15` |

After `STQ-020`, choose based on project priority:

- continue to `STAT-11` for Fisher information, Cramer-Rao, and MLE;
- continue to `STAT-08` for stronger CLT/asymptotic machinery;
- continue to `STAT-14` and `STAT-25` for Bayesian and decision-theory
  foundations;
- continue to `STAT-17` and `STAT-23` if empirical-process and learning goals
  are more important than classical inference coverage.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Measure-theoretic probability is attempted before measure foundations | broad rewrites and circular `L1` packages | start with finite/simple results and keep full measure theorems dependency-tagged |
| Statistical regularity assumptions are hidden | invalid asymptotic or likelihood theorems | name domination, differentiability, identifiability, tightness, and information assumptions explicitly |
| Duplicate theorem names are proved independently | incompatible APIs and later alias churn | maintain theorem cards and a duplicate map from `STAT-00` |
| Asymptotic statements omit convergence modes | unusable downstream inference results | require every asymptotic theorem to state mode, normalization, and indexing |
| Algorithms are treated as proof evidence | trusted boundary expands incorrectly | model algorithms as recurrences or kernels and prove properties of them |
| Large applied-statistics modules arrive too early | slow verification and unstable imports | keep survival, causal, survey, missing-data, and ML tracks late and layered |
| Public promotion happens before names stabilize | long-term compatibility burden | promote only `L2` theorem families with small import closures |

## Decision Points

- Whether the first probability namespace should be fully measure-theoretic
  from the start or include a finite-event sublayer for early `L2` progress.
- Whether expectations should first land for finite/simple random variables or
  wait for the full Lebesgue integral route.
- Which normal distribution parameterization is canonical for sampling,
  Bayesian conjugacy, and regression.
- Whether characteristic functions are routed through the Fourier analysis
  roadmap or a probability-specific transform layer.
- Whether high-level theorems such as Kolmogorov extension, Lindeberg-Feller
  CLT, Bernstein-von Mises, Donsker, de Finetti, Sanov, and Girsanov should
  first land as `L1` interfaces before derived proof attempts begin.
- Whether causal inference should use graphical, potential-outcome, or
  structural-equation modules as the primary namespace, with aliases for the
  other views.
- Which theorem families are appropriate for public `npa-mathlib` promotion
  before the full statistics stack is mature.
