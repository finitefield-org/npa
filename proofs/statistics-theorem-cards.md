# Statistics Theorem Cards

Source roadmaps:

- `proofs/statistics-theorem-proof-roadmap.md`
- `proofs/statistics-theorem-proof-roadmap-todo.md`

This file is the `STAT-T00` theorem-card inventory for the statistics and
probability proof roadmap. It is a planning sidecar only. It does not add
trusted proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, theorem-search sidecars, this document,
roadmaps, tactics, plugins, and AI output are untrusted.

## Card Legend

| Field | Meaning |
| --- | --- |
| Card | Primary roadmap theorem family. |
| Stable id | English identifier used for later source/module naming. |
| Level | Initial target level from the roadmap: `L0 Statement`, dependency-map / blocker, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary milestones | `STAT-T*` task milestones that own the first formalization. |
| Proposed modules | Planned `Proofs.Ai.Probability.*` or `Proofs.Ai.Statistics.*` entry points. |
| Kind | `foundation`, `derived theorem`, `interface`, `package alias`, `specialization`, or `promotion`. |
| Evidence | Main law package, route, model, or hypothesis evidence that must be explicit in statements. |
| Dependencies | Roadmap or module families that this card imports or waits for. |
| Gate | First acceptance gate for the card. |

Theorem cards decide ownership and target level. They never turn statement
text, generated indexes, replay files, or AI output into proof evidence.

## Namespace Contract

Current checked probability entry points:

| Module | Contract |
| --- | --- |
| `Proofs.Ai.Probability.Space.Basic` | finite event algebra and finite probability law package; no sigma-additivity or extension theorem assumption |
| `Proofs.Ai.Probability.Space.Extension` | dependency-route package for measure-theoretic probability extension names; no target extension theorem axiom |
| `Proofs.Ai.Probability.Conditional.Basic` | finite conditional probability, multiplication, total probability, and finite Bayes route |
| `Proofs.Ai.Probability.Independence.Basic` | event independence and finite-family independence route |
| `Proofs.Ai.Probability.RandomVariable.Basic` | finite random-variable and measurability vocabulary |
| `Proofs.Ai.Probability.Distribution.Basic` | distribution and pushforward statement API over checked probability foundations |
| `Proofs.Ai.Probability.Expectation.Basic` | finite/simple expectation package and derived route targets |
| `Proofs.Ai.Probability.Moments.Basic` | variance, covariance, correlation, and finite moment route targets |
| `Proofs.Ai.Probability.Inequalities.Concentration` | Markov, Chebyshev, and first concentration route targets |
| `Proofs.Ai.Probability.ConditionalExpectation.Basic` | finite conditional-expectation interface and general route hooks |
| `Proofs.Ai.Probability.Convergence.Basic` | convergence mode vocabulary and finite sequence specialization routes |
| `Proofs.Ai.Probability.LimitTheorems.LLN` | Borel-Cantelli and weak-law route targets |

Namespace ownership rules:

- `Proofs.Ai.Probability.*` owns finite probability spaces, finite
  conditional probability, independence, random variables, distributions,
  finite/simple expectation, moments, concentration, convergence modes,
  conditional-expectation interfaces, and basic limit-theorem routes.
- Future `Proofs.Ai.Statistics.*` modules own inference-specific APIs:
  estimators, sufficient statistics, Fisher information, likelihood,
  hypothesis tests, confidence intervals, Bayesian models, regression,
  multivariate statistics, nonparametric statistics, time series, causal
  inference, learning, computation, and decision theory.
- `Proofs.Ai.Measure.*` owns sigma algebras, measurable spaces, measure
  extension, integration, weak convergence, Radon-Nikodym, conditional
  expectation via measure, martingales, and probability-as-measure bridges.
- Probability and statistics modules may specialize or alias measure-owned
  theorem cards only after the required checked measure modules exist.
- Full measure-theoretic probability, regular conditional probability,
  dominated likelihood theory, and asymptotic distribution theory must not be
  marked `L2` from finite probability laws alone.

## Primary Roadmap Cards

| Card | Stable id | Level | Primary milestones | Proposed modules | Kind |
| --- | --- | --- | --- | --- | --- |
| `STAT-00` | `statistics_inventory_statement_policy` | `L0 Statement` | `STAT-T00` | this file, future `Proofs.Ai.Statistics.Inventory` | foundation |
| `STAT-01` | `finite_probability_space_basics` | `L2 Derived certificate` where finite | `STAT-T01` through `STAT-T03` | `Proofs.Ai.Probability.Space.Basic`, `Proofs.Ai.Probability.Space.Extension` | foundation |
| `STAT-02` | `conditional_probability_and_independence` | `L2 Derived certificate` where finite | `STAT-T04` through `STAT-T05` | `Proofs.Ai.Probability.Conditional.Basic`, `Proofs.Ai.Probability.Independence.Basic` | derived theorem |
| `STAT-03` | `random_variables_distributions_transforms` | `L2` for finite/discrete APIs, dependency-routed for full measure routes | `STAT-T06` through `STAT-T08` | `Proofs.Ai.Probability.RandomVariable.Basic`, `Proofs.Ai.Probability.Distribution.Basic`, `Proofs.Ai.Probability.Distribution.Transform` | interface |
| `STAT-04` | `expectation_moments_concentration` | `L2 Derived certificate` for finite/simple routes | `STAT-T09` through `STAT-T11` | `Proofs.Ai.Probability.Expectation.Basic`, `Proofs.Ai.Probability.Moments.Basic`, `Proofs.Ai.Probability.Inequalities.Concentration` | derived theorem |
| `STAT-05` | `conditional_expectation` | `L2` for finite interface, RN route stays as dependency-map work | `STAT-T12` through `STAT-T13` | `Proofs.Ai.Probability.ConditionalExpectation.Basic`, `Proofs.Ai.Measure.ConditionalExpectation` | interface |
| `STAT-06` | `convergence_of_random_variables` | `L2` for finite mode routes, weak convergence stays as dependency-map work | `STAT-T14` through `STAT-T16` | `Proofs.Ai.Probability.Convergence.Basic`, `Proofs.Ai.Measure.WeakConvergence` | derived theorem |
| `STAT-07` | `laws_of_large_numbers` | `L2 Derived certificate` where finite variance route exists | `STAT-T17` through `STAT-T18` | `Proofs.Ai.Probability.LimitTheorems.LLN` | derived theorem |
| `STAT-08` | `central_limit_and_asymptotic_tools` | `L2`; split blockers first after Fourier/weak convergence routes | `STAT-T19` through `STAT-T21` | future `Proofs.Ai.Probability.LimitTheorems.CLT`, `Proofs.Ai.Statistics.Asymptotic` | interface |
| `STAT-09` | `named_distributions_sampling_distributions` | `L2` for finite/discrete families | `STAT-T22` through `STAT-T24` | `Proofs.Ai.Statistics.SamplingDistribution.Basic`, future `Proofs.Ai.Probability.Distribution.Named` and normal/order-statistic modules | specialization |
| `STAT-10` | `sufficiency_unbiased_estimation` | `L2` for finite/discrete factorization and Rao-Blackwell routes | `STAT-T25` through `STAT-T27` | future `Proofs.Ai.Statistics.Sufficiency`, `Proofs.Ai.Statistics.UnbiasedEstimation` | derived theorem |
| `STAT-11` | `information_likelihood_theory` | `L2` for finite/dominated examples, otherwise dependency-map work | `STAT-T28` through `STAT-T31` | future `Proofs.Ai.Statistics.Information`, `Proofs.Ai.Statistics.Likelihood` | interface |
| `STAT-12` | `hypothesis_testing` | `L2` for finite simple-vs-simple routes | `STAT-T32` through `STAT-T35` | future `Proofs.Ai.Statistics.Testing` | derived theorem |
| `STAT-13` | `confidence_intervals` | `L2` for exact finite routes, dependency maps for asymptotic routes | `STAT-T36` through `STAT-T38` | future `Proofs.Ai.Statistics.ConfidenceInterval` | interface |
| `STAT-14` | `bayesian_statistics` | `L2` for finite posterior formula, dependency maps for asymptotics | `STAT-T39` through `STAT-T42` | future `Proofs.Ai.Statistics.Bayesian` | derived theorem |
| `STAT-15` | `regression_anova_glm` | `L2` for finite-dimensional algebraic routes | `STAT-T43` through `STAT-T46` | future `Proofs.Ai.Statistics.Regression`, `Proofs.Ai.Statistics.ANOVA`, `Proofs.Ai.Statistics.GLM` | derived theorem |
| `STAT-16` | `multivariate_statistics` | `L2` for finite-dimensional covariance vocabulary | `STAT-T47` through `STAT-T50` | future `Proofs.Ai.Statistics.Multivariate` | specialization |
| `STAT-17` | `nonparametric_empirical_processes` | `L2`; split blockers first after empirical-process prerequisites | `STAT-T51` through `STAT-T54` | future `Proofs.Ai.Statistics.Nonparametric`, `Proofs.Ai.Statistics.EmpiricalProcess` | interface |
| `STAT-18` | `time_series_stochastic_processes` | `L2` for finite linear-recursion routes, otherwise dependency-map work | `STAT-T55` through `STAT-T57` | future `Proofs.Ai.Statistics.TimeSeries` | interface |
| `STAT-19` | `information_theory_large_deviations` | `L2` for finite alphabet routes | `STAT-T58` through `STAT-T61` | future `Proofs.Ai.Statistics.InformationTheory`, `Proofs.Ai.Probability.LargeDeviation` | derived theorem |
| `STAT-20` | `martingales_stochastic_approximation` | `L2` after conditional expectation/martingale routes | `STAT-T62` through `STAT-T64` | `Proofs.Ai.Measure.Martingale`, future `Proofs.Ai.Probability.Martingale` | interface |
| `STAT-21` | `survival_survey_missing_data` | `L2` for finite design identities, otherwise dependency-map work | `STAT-T65` through `STAT-T67` | future `Proofs.Ai.Statistics.Survival`, `Proofs.Ai.Statistics.Survey`, `Proofs.Ai.Statistics.MissingData` | interface |
| `STAT-22` | `causal_inference` | `L2` for finite identification laws where assumptions are explicit | `STAT-T68` through `STAT-T70` | future `Proofs.Ai.Statistics.Causal` | interface |
| `STAT-23` | `statistical_learning` | `L2` for finite-class generalization routes | `STAT-T71` through `STAT-T74` | future `Proofs.Ai.Statistics.Learning` | derived theorem |
| `STAT-24` | `statistical_computation_optimization` | `L2` where abstract derivative/fixed-point routes suffice | `STAT-T75` through `STAT-T78` | future `Proofs.Ai.Statistics.Optimization`, `Proofs.Ai.Statistics.MCMC`, `Proofs.Ai.Statistics.Variational` | interface |
| `STAT-25` | `decision_theory` | `L2` for finite risk/minimax routes | `STAT-T79` through `STAT-T81` | future `Proofs.Ai.Statistics.DecisionTheory` | derived theorem |
| `STAT-26` | `distribution_specific_extreme_value_theory` | `L2` for discrete/reproductive laws, otherwise dependency-map work | `STAT-T82` through `STAT-T84` | future `Proofs.Ai.Probability.Distribution.Named`, `Proofs.Ai.Statistics.ExtremeValue` | specialization |
| `STAT-27` | `statistics_public_closure_promotion` | `L3 Public closure deferred` | `STAT-T85` | future `Mathlib.Statistics.*` closure batch after separate closure audit | promotion |

## Evidence And Dependency Map

| Card | Evidence | Dependencies | Gate |
| --- | --- | --- | --- |
| `STAT-00` | roadmap review, theorem cards, duplicate-home map, target levels; no source, replay, theorem index, or todo evidence | roadmap only | `git diff --check` |
| `STAT-01` | finite event algebra, probability law package, finite additivity, monotonicity, subadditivity, Boole, Bonferroni, and extension-route dependency tags | set theory basics, measure roadmap for extension routes | source-free module verify for `Probability.Space.Basic` and `Probability.Space.Extension` |
| `STAT-02` | nonzero conditioning events, finite conditional probability, multiplication, total probability, finite Bayes, pairwise/mutual independence evidence | `STAT-01` | source-free module verify |
| `STAT-03` | random-variable measurability evidence, distribution/pushforward hooks, finite/discrete CDF and transform certificates, transform aliases for characteristic/mgf/pgf/Laplace routes | `STAT-01`, measure roadmap for general routes, `ANA-T31`/`ANA-T32` for Fourier and Levy continuity | source-free module verify or interface audit |
| `STAT-04` | finite/simple expectation, finite-sum linearity, LOTUS, moments, Markov/Chebyshev, concentration evidence | `STAT-01`, `STAT-03` | source-free module verify |
| `STAT-05` | finite conditional-expectation law package and RN/regular-conditional split | `STAT-02`, `STAT-04`, measure RN route | source-free module verify or interface audit |
| `STAT-06` | convergence mode vocabulary, in-probability/almost-sure/distribution routes, Portmanteau ownership split | `STAT-03`, `STAT-04`, measure weak convergence | source-free module verify or interface audit |
| `STAT-07` | Borel-Cantelli, Chebyshev WLLN, empirical LLN aliases with finite variance and independence evidence explicit | `STAT-04`, `STAT-06` | source-free module verify |
| `STAT-08` | CLT statement interfaces, characteristic-function/Fourier blockers, continuous-mapping and delta-method route evidence | `STAT-06`, analysis Fourier, measure weak convergence | interface audit before derived source |
| `STAT-09` | finite iid sample package, sample mean expectation and variance, unbiased sample variance, named distribution law packages, order-statistics route split | `STAT-02`, `STAT-03`, `STAT-04` | source-free module verify or interface audit |
| `STAT-10` | factorization theorem evidence, sufficient statistic API, unbiasedness, Rao-Blackwell and Lehmann-Scheffe routes | `STAT-02`, `STAT-04`, `STAT-09` | source-free module verify |
| `STAT-11` | score, Fisher information, Cramer-Rao, KL minimization, MLE consistency/asymptotics, Wilks route evidence | calculus, integration, `STAT-10` | source-free module verify or interface audit |
| `STAT-12` | test object, size, power, p-value, Neyman-Pearson, UMP, asymptotic statistic, multiple-testing routes | `STAT-02`, `STAT-04`, `STAT-11` for likelihood routes | source-free module verify |
| `STAT-13` | exact interval constructions, score/likelihood/asymptotic interval split, bootstrap/simultaneous interval blockers | `STAT-09`, `STAT-11`, `STAT-12` | source-free module verify or interface audit |
| `STAT-14` | finite posterior formula, conjugacy split, posterior-risk alias, de Finetti route blockers | `STAT-02`, `STAT-25` for risk aliases | source-free module verify or interface audit |
| `STAT-15` | finite-dimensional linear algebra, OLS, Gauss-Markov, FWL, ANOVA, GLM likelihood route | vector and inner-product modules, `STAT-11` | source-free module verify |
| `STAT-16` | multivariate normal, covariance, Wishart/Hotelling/MANOVA, PCA/SVD, spectral route evidence | linear algebra, spectral modules, `STAT-08` for asymptotics | source-free module verify or interface audit |
| `STAT-17` | empirical distribution, GC/Donsker blockers, rank tests, kernels, U-statistics, bootstrap/jackknife split | `STAT-06`, `STAT-08`, combinatorics finite routes | source-free module verify or interface audit |
| `STAT-18` | stationarity, autocovariance, ARMA/state-space, unit-root/cointegration/mixing/GARCH split | `STAT-04`, `STAT-06`, spectral/Fourier routes | source-free module verify or interface audit |
| `STAT-19` | divergence, entropy, data processing, Fano, Chernoff, Sanov, Cramer, transportation inequalities | finite alphabet probability, analysis convexity where needed | source-free module verify or interface audit |
| `STAT-20` | filtration, martingale law packages, optional stopping, martingale convergence/CLT route evidence | conditional expectation and measure martingale modules | source-free module verify or interface audit |
| `STAT-21` | survival product-limit identities, finite survey design identities, missing-data assumptions | `STAT-02`, `STAT-04`, `STAT-20` for counting-process routes | source-free module verify or interface audit |
| `STAT-22` | potential outcomes, ignorability, IPW/AIPW, IV/RD/DiD/synthetic control/mediation assumptions | `STAT-02`, `STAT-14`, `STAT-25` | source-free module verify or interface audit |
| `STAT-23` | ERM, finite-class union bound, VC, Sauer-Shelah, Rademacher, regularization, PAC-Bayes | finite probability, combinatorics, convex optimization | source-free module verify or interface audit |
| `STAT-24` | convex/KKT/duality aliases, EM/MM/Newton/Fisher scoring/SGD, MCMC, VI/Laplace/importance sampling | analysis derivative/fixed-point, optimization roadmap | source-free module verify or interface audit |
| `STAT-25` | decision risk, finite Bayes risk, minimax, complete class, admissibility, James-Stein split | `STAT-02`, `STAT-14`, convex/separation routes | source-free module verify |
| `STAT-26` | distribution reproductive laws, distribution relationships, stable and extreme-value interfaces | `STAT-03`, `STAT-09`, `STAT-19` | source-free module verify or interface audit |
| `STAT-27` | selected certificate-backed closures, deterministic hashes, axiom report, downstream smoke, closure audit evidence | completed stable statistics/probability batch | package and closure audit gates |

## Duplicate-Home Map

| Theorem family or alias | Primary home | Statistics status | Reason |
| --- | --- | --- | --- |
| finite additivity, monotonicity, subadditivity, Boole inequality, Bonferroni inequality | `STAT-01` | primary here | these are finite probability theorems; multiple-testing and set-system modules may alias them |
| countable additivity, Hahn-Kolmogorov, Kolmogorov extension, Ionescu-Tulcea | measure roadmap plus `STAT-01` extension-route card | dependency-routed | finite probability must not claim sigma-additivity or extension existence |
| finite Bayes formula and total probability | `STAT-02` | primary here | Bayesian posterior cards alias this only for finite/discrete posterior formulas |
| posterior formula and conjugacy | `STAT-14` | primary Bayesian route | these are model/posterior statements, not the same home as finite conditional probability |
| Bayes risk and Bayes decision rules | `STAT-25` | primary decision-theory route | risk minimization requires decision/loss API, not only posterior formula |
| independence of events versus random variables | `STAT-02` for event independence, `STAT-03` for random-variable distribution API | split owner | random-variable independence should import event/distribution evidence rather than duplicate it |
| moment-generating, characteristic, probability-generating, and Laplace transforms | `STAT-03` for transform vocabulary, `STAT-08` for CLT/asymptotic consumption | dependency-routed | characteristic-function inversion and Levy continuity require `ANA-T31`/`ANA-T32`; CLT modules consume these aliases instead of assuming Fourier facts |
| weak convergence, Portmanteau, Prokhorov | measure weak-convergence route with `STAT-06` aliases | external measure owner for general theorems | topology and measure prerequisites are not finite probability facts |
| WLLN, SLLN, empirical LLN | `STAT-07` | primary probability limit route | empirical-process modules alias the completed LLN card |
| CLT, Cramer-Wold, delta method, continuous mapping | `STAT-08` | primary asymptotic route | regression, multivariate, and likelihood modules import these names instead of reproving them |
| Fisher information, Cramer-Rao, MLE, Wilks | `STAT-11` | primary likelihood route | testing and confidence-interval modules consume this route when asymptotic likelihood theory is needed |
| Neyman-Pearson and UMP | `STAT-12` | primary testing route | likelihood modules provide inputs, but optimal-testing theorem names live here |
| exact confidence intervals | `STAT-13` | primary interval route | testing p-values and intervals may be related but have separate API homes |
| Gauss-Markov, OLS, FWL, ANOVA, GLM | `STAT-15` | primary regression route | linear algebra provides prerequisites, not statistical theorem ownership |
| multivariate normal, Wishart, Hotelling, MANOVA, PCA/SVD | `STAT-16` | primary multivariate route | spectral and linear algebra modules provide shared infrastructure |
| entropy, KL, Fano, Chernoff, Sanov | `STAT-19` | primary information/large-deviation route | likelihood and learning cards may alias divergence facts |
| martingales and optional stopping | `STAT-20` with measure martingale dependency | split owner | measure modules own general martingale foundations; statistics owns process/asymptotic uses |

## First Execution Queue

| Queue ID | Theorem or task | Target level | Primary milestone | Primary card |
| --- | --- | --- | --- | --- |
| `STQ-001` | theorem-card inventory and duplicate map | `L0` | `STAT-T00` | `STAT-00` |
| `STQ-002` | probability-space law package and complement rule | `L2` | `STAT-T01` | `STAT-01` |
| `STQ-003` | finite additivity, monotonicity, Boole, Bonferroni | `L2` | `STAT-T02` | `STAT-01` |
| `STQ-004` | finite conditional probability, multiplication, total probability, Bayes | `L2` | `STAT-T04` | `STAT-02` |
| `STQ-005` | event independence, pairwise versus mutual independence | `L2` | `STAT-T05` | `STAT-02` |
| `STQ-006` | random variable and distribution statement API | `L2` or prerequisite split | `STAT-T06` | `STAT-03` |
| `STQ-007` | CDF basic properties and monotone-transform formula | `L2` for finite/discrete basics; measure routes after real/measure foundations | `STAT-T07`, `STAT-T08` | `STAT-03` |
| `STQ-008` | finite/simple expectation linearity and LOTUS | `L2` | `STAT-T09` | `STAT-04` |
| `STQ-009` | variance, covariance, and correlation range | `L2` | `STAT-T10` | `STAT-04` |
| `STQ-010` | Markov and Chebyshev inequalities | `L2` | `STAT-T11` | `STAT-04` |
| `STQ-011` | conditional expectation statement shape and tower property route | `L2` or prerequisite split | `STAT-T12` | `STAT-05` |
| `STQ-012` | convergence modes and implication chain through probability convergence | `L2` | `STAT-T14`, `STAT-T15` | `STAT-06` |
| `STQ-013` | Chebyshev weak law of large numbers | `L2` | `STAT-T17` | `STAT-07` |
| `STQ-014` | De Moivre-Laplace or Lindeberg-Levy CLT statement split | `L2` from first proof attempt where prerequisites exist | `STAT-T19` | `STAT-08` |
| `STQ-015` | sample mean expectation and variance | `L2` | `STAT-T23` | `STAT-09` |
| `STQ-016` | sample variance unbiasedness | `L2` | `STAT-T23` | `STAT-09` |
| `STQ-017` | Fisher-Neyman factorization theorem finite/discrete case | `L2` | `STAT-T25` | `STAT-10` |
| `STQ-018` | Rao-Blackwell theorem finite/discrete case | `L2` | `STAT-T26` | `STAT-10` |
| `STQ-019` | Neyman-Pearson lemma finite simple-vs-simple case | `L2` | `STAT-T33` | `STAT-12` |
| `STQ-020` | Gauss-Markov theorem fixed-design finite-dimensional case | `L2` | `STAT-T44` | `STAT-15` |

## Milestone-To-Card Checklist

| Roadmap item | Card present | Primary home unique | Prerequisites explicit | Sidecar trust boundary clear |
| --- | --- | --- | --- | --- |
| `STAT-00` | yes | yes | yes | yes |
| `STAT-01` | yes | yes | yes | yes |
| `STAT-02` | yes | yes | yes | yes |
| `STAT-03` | yes | yes | yes | yes |
| `STAT-04` | yes | yes | yes | yes |
| `STAT-05` | yes | yes | yes | yes |
| `STAT-06` | yes | yes | yes | yes |
| `STAT-07` | yes | yes | yes | yes |
| `STAT-08` | yes | yes | yes | yes |
| `STAT-09` | yes | yes | yes | yes |
| `STAT-10` | yes | yes | yes | yes |
| `STAT-11` | yes | yes | yes | yes |
| `STAT-12` | yes | yes | yes | yes |
| `STAT-13` | yes | yes | yes | yes |
| `STAT-14` | yes | yes | yes | yes |
| `STAT-15` | yes | yes | yes | yes |
| `STAT-16` | yes | yes | yes | yes |
| `STAT-17` | yes | yes | yes | yes |
| `STAT-18` | yes | yes | yes | yes |
| `STAT-19` | yes | yes | yes | yes |
| `STAT-20` | yes | yes | yes | yes |
| `STAT-21` | yes | yes | yes | yes |
| `STAT-22` | yes | yes | yes | yes |
| `STAT-23` | yes | yes | yes | yes |
| `STAT-24` | yes | yes | yes | yes |
| `STAT-25` | yes | yes | yes | yes |
| `STAT-26` | yes | yes | yes | yes |
| `STAT-27` | yes | yes | yes | yes |

## Acceptance Status

`STAT-T00` is complete when this file is present, cited from
`proofs/README.md`, and the roadmap/todo verification searches find the
required duplicate-home and sidecar-trust terms. This file does not prove any
mathematical theorem and does not create a certificate. Later milestones must
replace `L0` and `L1` planning surfaces with certificate-backed source modules
before claiming `L2` or `L3` status.
