# Measure Theory Theorem Cards

Source roadmaps:

- `proofs/measure-theory-theorem-proof-roadmap.md`
- `proofs/measure-theory-theorem-proof-roadmap-todo.md`

This file is the `MEA-T00` theorem-card inventory for the measure-theory proof
roadmap. It is a planning sidecar only. It does not add trusted proof evidence,
axioms, source-free certificate verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, theorem-search sidecars, this document,
roadmaps, tactics, plugins, and AI output are untrusted.

## Card Legend

| Field | Meaning |
| --- | --- |
| Card | Primary roadmap theorem family. |
| Stable id | English identifier used for later source/module naming. |
| Level | Initial target level from the roadmap: `L0 Statement`, `L1 Evidence package`, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary milestones | `MEA-T*` task milestones that own the first formalization. |
| Proposed modules | Planned `Proofs.Ai.Measure.*` entry points or explicitly external owners. |
| Kind | `foundation`, `derived theorem`, `construction interface`, `specialization`, `package alias`, `long-term interface`, or `promotion`. |
| Evidence | Main structure, route, or hypothesis evidence that must be explicit in statements. |
| Dependencies | Roadmap or module families that this card imports or waits for. |
| Gate | First acceptance gate for the card. |

Broad existence, extension, regularity, probability, martingale, ergodic, and
geometric-measure statements stay `L1` interfaces until their prerequisite
definitions and intermediate lemmas are certificate-backed. They must not be
marked `L2` merely because a theorem-search sidecar or roadmap row exists.

## Namespace Contract

Concrete entry point: `Proofs.Ai.Measure.Inventory`.

This module is a certificate-backed policy entry point, not a mathematical
measure-theory foundation. Its checked declarations preserve:

| Checked theorem | Contract evidence it preserves |
| --- | --- |
| `measure_object_structure_policy` | measure objects are ordinary proof-corpus structures supplied by later modules |
| `measure_namespace_split_policy` | the detailed route uses `Measure.Basic`, `Measure.Outer`, `Measure.Caratheodory`, `Measure.Extension`, `Measure.Integral`, `Measure.Product`, and `Measure.Decomposition` as separate namespaces |
| `measure_duplicate_home_policy` | aliases point to a single primary theorem home instead of duplicating proofs |
| `measure_sidecar_untrusted_policy` | source, replay, metadata, AI theorem indexes, theorem-search sidecars, tactics, and roadmaps are not proof evidence |
| `measure_probability_specialization_policy` | probability spaces specialize measure spaces instead of introducing a second measure API |
| `measure_derived_target_certificate_policy` | derived theorem targets require source-free certificate-verdict evidence |

Namespace ownership rules:

- `Proofs.Ai.Measure.*` owns sigma algebras, measurable spaces, measures,
  outer measures, extension routes, measurable functions, measure integrals,
  convergence theorems, product measures, pushforwards, signed and complex
  measures, regular measures, and abstract measure interfaces.
- `Proofs.Ai.Analysis.*` remains the owner for real-analysis scalar,
  sequence, continuity, Riemann integral, series, inverse-function, and
  implicit-function theorem routes. Measure modules may import analysis
  evidence but must not duplicate those theorem homes.
- `Proofs.Ai.Topology.*` owns general topology, compactness, Baire category,
  countability, product and quotient topologies, and regularity assumptions
  used by Borel, Radon, Polish, and weak-convergence statements.
- `Proofs.Ai.FunctionalAnalysis.*` owns spectral theorem and operator-family
  specializations. Projection-valued measures are aliases or specializations
  until the general measure route provides reusable foundations.
- Probability, martingale, ergodic, statistics, and geometric-measure theorem
  names must specialize or alias measure-owned cards unless their roadmap
  explicitly owns the primary proof.
- Measure objects, sigma algebras, measurable spaces, integrals, probability
  spaces, and Radon-Nikodym derivatives are ordinary proof-corpus structures or
  explicit law packages. They are not kernel primitives.

## Primary Roadmap Cards

| Card | Stable id | Level | Primary milestones | Proposed modules | Kind |
| --- | --- | --- | --- | --- | --- |
| `MEA-00` | `measure_inventory_statement_policy` | `L0 Statement` | `MEA-T00`, `MEA-T01` | `Proofs.Ai.Measure.Inventory` | foundation |
| `MEA-01` | `sigma_algebras_measurable_spaces` | `L1` then `L2` | `MEA-T02` through `MEA-T05` | `Proofs.Ai.Measure.SigmaAlgebra`, `Proofs.Ai.Measure.MonotoneClass`, `Proofs.Ai.Measure.MeasurableSpace`, `Proofs.Ai.Measure.Product.SigmaAlgebra` | foundation |
| `MEA-02` | `basic_measures` | `L2` where prerequisites exist | `MEA-T06` through `MEA-T08` | `Proofs.Ai.Measure.Basic` | derived theorem |
| `MEA-03` | `outer_measure_extension` | `L1` then `L2` | `MEA-T09` through `MEA-T12` | `Proofs.Ai.Measure.Outer`, `Proofs.Ai.Measure.Caratheodory`, `Proofs.Ai.Measure.Extension` | construction interface |
| `MEA-04` | `lebesgue_stieltjes_measures` | `L2` construction packages | `MEA-T13` through `MEA-T15` | `Proofs.Ai.Measure.Lebesgue`, `Proofs.Ai.Measure.LebesgueStieltjes` | construction interface |
| `MEA-05` | `measurable_functions` | `L2` where prerequisites exist | `MEA-T16` through `MEA-T18` | `Proofs.Ai.Measure.Function` | derived theorem |
| `MEA-06` | `simple_functions_integral_construction` | `L2` after simple-function foundations | `MEA-T19` through `MEA-T21` | `Proofs.Ai.Measure.Integral.Simple`, `Proofs.Ai.Measure.Integral.Basic` | construction interface |
| `MEA-07` | `convergence_theorem_chain` | `L2` after integral foundations | `MEA-T22` through `MEA-T25` | `Proofs.Ai.Measure.Integral.Convergence` | derived theorem |
| `MEA-08` | `product_measures_fubini_tonelli` | `L2` after product and integral foundations | `MEA-T26` through `MEA-T29` | `Proofs.Ai.Measure.Product`, `Proofs.Ai.Measure.Section`, `Proofs.Ai.Measure.Fubini` | derived theorem |
| `MEA-09` | `pushforwards_change_of_variables` | `L2` where prerequisites exist | `MEA-T30` through `MEA-T32` | `Proofs.Ai.Measure.Pushforward`, `Proofs.Ai.Measure.Distribution`, `Proofs.Ai.Measure.ChangeOfVariables` | derived theorem |
| `MEA-10` | `signed_complex_decomposed_measures` | `L2` construction packages | `MEA-T33` through `MEA-T36` | `Proofs.Ai.Measure.Signed`, `Proofs.Ai.Measure.RadonNikodym`, `Proofs.Ai.Measure.Decomposition`, `Proofs.Ai.Measure.Complex` | derived theorem |
| `MEA-11` | `lebesgue_regularity_differentiation` | `L2` late-interface packages | `MEA-T37` through `MEA-T39` | `Proofs.Ai.Measure.Lebesgue.Regularity`, `Proofs.Ai.Measure.Covering`, `Proofs.Ai.Measure.Lebesgue.Density`, `Proofs.Ai.Measure.Lebesgue.Differentiation` | derived theorem |
| `MEA-12` | `lp_spaces_inequalities` | `L2` late-interface packages | `MEA-T40` through `MEA-T43` | `Proofs.Ai.Measure.Lp.Basic`, `Proofs.Ai.Measure.Lp.Inequality`, `Proofs.Ai.Measure.Lp.Duality`, `Proofs.Ai.Measure.Lp.Interpolation` | derived theorem |
| `MEA-13` | `topological_measures_weak_convergence` | `L2` late-interface packages | `MEA-T44` through `MEA-T47` | `Proofs.Ai.Measure.Borel`, `Proofs.Ai.Measure.Radon`, `Proofs.Ai.Measure.WeakConvergence`, `Proofs.Ai.Measure.Selection`, `Proofs.Ai.Measure.Topological` | specialization |
| `MEA-14` | `probability_martingale_ergodic_bridges` | `L2` bridge packages | `MEA-T48` through `MEA-T51` | `Proofs.Ai.Measure.ProbabilityBridge`, `Proofs.Ai.Measure.ConditionalExpectation`, `Proofs.Ai.Measure.Martingale`, `Proofs.Ai.Measure.Ergodic` | package alias |
| `MEA-15` | `geometric_abstract_measure_theory` | `L2` late-interface packages | `MEA-T52` through `MEA-T55` | `Proofs.Ai.Measure.Geometric`, `Proofs.Ai.Measure.Algebra` | long-term interface |
| `MEA-16` | `measure_public_closure_promotion` | `L3 Public closure deferred` | `MEA-T56` | future `Mathlib.Measure.*` closure batch after separate closure audit | promotion |

## Outer Measure And Extension Split

| Subroute | Owner | Milestones | Level | Boundary |
| --- | --- | --- | --- | --- |
| `outer_measure_laws` | `Proofs.Ai.Measure.Outer` | `MEA-T09` | `L2` law package and projections | empty-set, monotonicity, and countable-subadditivity laws only; no premeasure extension |
| `caratheodory_measurability_sigma` | `Proofs.Ai.Measure.Caratheodory` | `MEA-T09`, `MEA-T10` | `L2` split criterion, closure evidence, and derived structure certificates | builds `SigmaAlgebraCore`, `MeasurableSpace`, and restricted `MeasureSpace` from explicit split/subadditivity evidence, not from an assumed sigma algebra |
| `extension_interfaces` | `Proofs.Ai.Measure.Extension` | `MEA-T11`, `MEA-T12` | `L1` construction interfaces plus pi-lambda uniqueness certificate | premeasure-induced outer measures, extension existence, and uniqueness remain outside the Outer/Caratheodory base modules; construction evidence and sigma-finiteness are explicit fields |

## Extension Uniqueness Variant Cards

| Variant | Owner | Required generator evidence | Required finiteness evidence | Route |
| --- | --- | --- | --- | --- |
| `semiring_extension_uniqueness` | `Proofs.Ai.Measure.Extension` | `SetSemiringInterface` plus generated sigma algebra | left and right `SigmaFiniteOnSeed` covers | `DynkinPiLambdaRoute` to generated sigma algebra |
| `ring_extension_uniqueness` | `Proofs.Ai.Measure.Extension` | `SetRingInterface` plus generated sigma algebra | left and right `SigmaFiniteOnSeed` covers | same pi-lambda uniqueness certificate |
| `algebra_extension_uniqueness` | `Proofs.Ai.Measure.Extension` | `SetAlgebraInterface` plus generated sigma algebra | left and right `SigmaFiniteOnSeed` covers | same pi-lambda uniqueness certificate |

## Evidence And Dependency Map

| Card | Evidence | Dependencies | Gate |
| --- | --- | --- | --- |
| `MEA-00` | roadmap review, theorem cards, duplicate-home map, target levels; no source, replay, theorem index, or todo evidence | roadmap only, plus `Proofs.Ai.Measure.Inventory` for namespace policy | source-free module verify and `git diff --check` |
| `MEA-01` | set predicates, sigma-algebra closure laws, generated sigma algebra, Borel generator routes, pi-lambda and monotone-class evidence, measurable-space/map laws, product sigma algebra generated by rectangles | topology basics for Borel routes | source-free module verify |
| `MEA-02` | explicit measure structure, empty-set law, monotonicity, subadditivity, finite and sigma-finite hypotheses, continuity from below and above | `MEA-01` | source-free module verify |
| `MEA-03` | outer measure laws and Caratheodory criterion/sigma-algebra evidence are separated from premeasure and extension evidence; Extension adds premeasure-induced outer-measure packages and pi-lambda uniqueness under explicit sigma-finiteness | `MEA-01` monotone-class tools, `MEA-02` for restriction measure | source-free module verify for Outer/Caratheodory/Extension |
| `MEA-04` | interval covers, Borel measure, completion, translation/scaling invariance, monotone function and distribution-function evidence | `MEA-03`, real-analysis and topology routes | source-free module verify for Lebesgue and Lebesgue-Stieltjes construction packages |
| `MEA-05` | measurable-map preimage evidence, indicator functions, algebraic closure, composition, pointwise and a.e. routes | `MEA-01`, `MEA-02` | source-free module verify |
| `MEA-06` | simple-function representation, finite range evidence, nonnegative integral construction, monotone extension evidence | `MEA-05`, `MEA-02` | source-free module verify |
| `MEA-07` | monotone convergence, Fatou, dominated convergence, a.e. convergence, convergence in measure, Egorov and Lusin hypotheses | `MEA-06`, topology where Lusin is used | source-free module verify or interface audit |
| `MEA-08` | product sigma algebra, product measure existence and uniqueness, sigma-finiteness, sections, nonnegative and integrable function hypotheses, pi-lambda or monotone-class extension routes | `MEA-01` product-sigma and monotone-class tools, `MEA-02`, `MEA-07` | source-free module verify for Product, Section, and Fubini |
| `MEA-09` | measurable map, pushforward definition, distribution measures, measurable isomorphism, elementary translation/scaling routes, and late Jacobian / density / disintegration evidence | `MEA-05`, `MEA-08`, `MEA-10` for density transformation | source-free module verify for Pushforward, Distribution, and ChangeOfVariables |
| `MEA-10` | signed-measure, positive/negative set predicates, Hahn/Jordan, total variation, absolute continuity, Radon-Nikodym derivative, Lebesgue decomposition, and complex-measure evidence | `MEA-02`, `MEA-07` | source-free module verify for Signed, RadonNikodym, Decomposition, and Complex |
| `MEA-11` | Borel/Lebesgue regularity, Vitali covering, Hardy-Littlewood maximal interfaces, density theorem, differentiation basis, and Riemann/Lebesgue bridge assumptions | `MEA-04`, `MEA-10`, topology compactness/regularity | source-free module verify for Lebesgue.Regularity, Covering, Lebesgue.Density, and Lebesgue.Differentiation |
| `MEA-12` | seminorm/norm evidence, quotient by a.e. equality, essential supremum, Markov/Chebyshev/Jensen/Young, Holder before Minkowski, Riesz-Fischer, L2 Hilbert, Lp duality/reflexivity, and interpolation/Fourier/Sobolev interfaces | `MEA-07`, analysis normed-space routes | source-free module verify for Lp.Basic, Lp.Inequality, Lp.Duality, and Lp.Interpolation |
| `MEA-13` | Borel, regular Borel, Radon, tightness, weak convergence, Ulam, Portmanteau, Prokhorov, Skorokhod, Wasserstein/vague hooks, analytic sets, Suslin/Lusin, selection hypotheses, and topological split boundaries | topology regularity/countability/compactness | source-free module verify for Borel, Radon, WeakConvergence, Selection, and Topological |
| `MEA-14` | probability as finite measure with total mass one, independence via product measure, conditional expectation via Radon-Nikodym, recurrence and ergodic hypotheses | `MEA-08`, `MEA-10`, probability/statistics route cards | source-free module verify for ProbabilityBridge, ConditionalExpectation, Martingale, and Ergodic |
| `MEA-15` | Hausdorff measure, packing measure, coarea, area formula, measure algebra, Boolean algebra and classification evidence | `MEA-03`, `MEA-11`, topology and geometry routes | source-free module verify for Geometric and Algebra |
| `MEA-16` | selected certificate-backed measure closure, deterministic hashes, axiom report, downstream smoke, closure audit evidence | stable `Proofs.Ai.Measure.*` batch | package and closure audit gates |

## Duplicate-Home Map

| Theorem family or alias | Primary home | Measure status | Reason |
| --- | --- | --- | --- |
| Tonelli theorem, Fubini theorem, product-measure uniqueness, section-measure measurability | `MEA-08` | primary here | these require product measure, integral, and sigma-finiteness evidence |
| dominated convergence, monotone convergence, Fatou, bounded convergence, Egorov, Lusin | `MEA-07` | primary here | sequence/analysis roadmaps may alias, but measure-convergence and integral hypotheses are measure-owned |
| Radon-Nikodym theorem, Lebesgue decomposition, Hahn/Jordan decomposition, total variation | `MEA-10` | primary here | probability and functional-analysis consumers import the measure-owned decomposition route |
| `L^p` norm laws, Holder, Minkowski, completeness, finite-measure inclusions | `MEA-12`, coordinated with analysis/functional analysis | primary measure route for measure-space hypotheses | analytic norm infrastructure stays in analysis, but a.e. quotient and finite-measure assumptions are measure-owned |
| weak convergence, Portmanteau, Prokhorov, Radon/tightness interfaces | `MEA-13` | primary here with topology inputs | statistics asymptotics and probability convergence names alias this route |
| probability spaces, independence, conditional expectation, martingales, ergodic recurrence | `MEA-14` for measure bridge, probability/ergodic roadmaps for domain-specific theorems | bridge here | probability must specialize measure spaces rather than add a second measure API |
| projection-valued measures and spectral measure representation | functional-analysis spectral theorem modules, with later measure aliases | external owner until measure foundations can be imported | operator/spectral evidence is functional-analysis-owned |
| p-adic measures, Amice transform, Kubota-Leopoldt p-adic L-functions | `Proofs.Ai.NumberTheory.PadicMeasure` | external owner | arithmetic p-adic measure specializations stay number-theory-owned and import or alias general measure facts later |
| Hausdorff measure, packing measure, coarea and area formulas, measure algebra | `MEA-15` | long-term measure interface | geometric and abstract measure-theory names are late and must not block basic measure construction |

## Milestone-To-Card Checklist

| Milestone | Card present | Primary home unique | Prerequisites explicit | Sidecar trust boundary clear |
| --- | --- | --- | --- | --- |
| `MEA-00` | yes | yes | yes | yes |
| `MEA-01` | yes | yes | yes | yes |
| `MEA-02` | yes | yes | yes | yes |
| `MEA-03` | yes | yes | yes | yes |
| `MEA-04` | yes | yes | yes | yes |
| `MEA-05` | yes | yes | yes | yes |
| `MEA-06` | yes | yes | yes | yes |
| `MEA-07` | yes | yes | yes | yes |
| `MEA-08` | yes | yes | yes | yes |
| `MEA-09` | yes | yes | yes | yes |
| `MEA-10` | yes | yes | yes | yes |
| `MEA-11` | yes | yes | yes | yes |
| `MEA-12` | yes | yes | yes | yes |
| `MEA-13` | yes | yes | yes | yes |
| `MEA-14` | yes | yes | yes | yes |
| `MEA-15` | yes | yes | yes | yes |
| `MEA-16` | yes | yes | yes | yes |

## Acceptance Status

`MEA-T00` is complete when this file is present, cited from `proofs/README.md`,
and the roadmap/todo verification searches find the required terms. This file
does not prove any mathematical theorem and does not create a certificate.
`MEA-T01` is complete when `Proofs.Ai.Measure.Inventory` verifies as the
certificate-backed namespace and statement-policy entry point. Later milestones
must replace `L0` and `L1` planning surfaces with certificate-backed source
modules before claiming `L2` or `L3` status.
