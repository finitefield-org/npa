# Measure Theory Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided measure-theory theorem
inventory one theorem family at a time in the NPA proof corpus. It refines the
analysis roadmap's `ANA-08` measure and Lebesgue integration milestone. It is a
planning sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- sigma algebras, measurable spaces, generated sigma algebras, pi-lambda and
  monotone class principles;
- measures, continuity of measure, finite and sigma-finite measure spaces, and
  completion;
- outer measures, Caratheodory measurability, extension theorems, Lebesgue
  measure, and Lebesgue-Stieltjes measure;
- measurable functions, simple-function approximation, Lebesgue integration,
  and convergence theorems;
- product measures, sections, Tonelli theorem, Fubini theorem, and iterated
  integrals;
- pushforward measures, distribution measures, change-of-variables formulas,
  and density transformation statements;
- signed and complex measures, Hahn and Jordan decomposition, total variation,
  Lebesgue decomposition, and Radon-Nikodym theorem;
- Lebesgue regularity, density, differentiation, and covering-theorem
  interfaces;
- `L^p` spaces, inequalities, completeness, duality, interpolation, and
  convolution interfaces;
- Borel, Radon, Polish-space, weak-convergence, probability, martingale,
  ergodic, geometric-measure, and measure-algebra theorem families.

The first priority is not to encode every named theorem immediately. The first
priority is to build small reusable measure foundations whose statements will
not need to be replaced after probability, Fourier analysis, PDE, functional
analysis, or statistics milestones depend on them.

## Existing Baseline

The current proof corpus has reusable algebra, order, metric-topology,
derivative, fixed-point, linear-map, Hilbert-space, and spectral theorem
routes. It does not yet expose a dedicated `Proofs.Ai.Measure.*` namespace.

Measure-theory work should therefore start with explicit law packages and
statement-level interfaces, then gradually replace them with derived
certificate-backed modules. Probability and statistics roadmaps should import
or specialize this route rather than carrying separate measure primitives.

Relevant roadmap dependencies:

| Needed foundation | Expected source |
| --- | --- |
| ordered fields, inequalities, and sequences | `proofs/analysis-theorem-proof-roadmap.md` `ANA-01` |
| metric and topological analysis | `ANA-07` and `proofs/topology-theorem-proof-roadmap.md` |
| Riemann integration comparison points | `ANA-05` |
| functional analysis and Hilbert-space interfaces | `ANA-09` |
| probability and statistics specialization | `proofs/statistics-theorem-proof-roadmap.md` |

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant or theorem shape only | no |
| `L1 Evidence package` | theorem conclusion follows from explicit construction or law evidence | no for pending theorem-proof tasks; use only as a blocker/dependency note |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

Classical existence theorems may first be recorded as dependency-map entries
when the fully derived construction is too large for the current corpus layer.
Those entries must not be confused with derived theorems, and the theorem
conclusion itself must not appear as an input under another name.

## One-Theorem Work Unit

For each theorem, use this work unit:

1. Freeze the statement in the smallest suitable `Proofs.Ai.Measure.*` module.
2. Classify the target as `L0`, `L1`, `L2`, or `L3`.
3. Audit the target for circular assumptions.
4. Keep imports minimal and prefer existing corpus modules.
5. Add or update checked source, replay, metadata, and certificate.
6. Verify the target module source-free.
7. Verify changed proof-corpus artifacts.
8. At the end of a coherent batch, run the authoring gate.

Default proof-corpus commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
only for package-wide compatibility, promotion, release readiness, or changes
to certificate encoding, checker behavior, package verification, or kernel
semantics.

## Statement Policy

- Measure spaces are ordinary structures over a carrier, a sigma algebra, and
  a countably additive size function; they are not kernel primitives.
- Sigma algebras, measurable functions, almost-everywhere predicates, null
  sets, and completions are ordinary definitions or explicit law packages.
- Tactics, elaborators, theorem search, notation, implicit arguments, and
  automation may produce proof terms or certificates, but are not trusted.
- Extension theorems should stay as blocker/dependency-map work until the full
  Caratheodory derivation is available; any package fields must state the exact
  construction boundary.
- Lebesgue measure, product measure, Radon measures, probability measures, and
  spectral measures are specializations or evidence packages over the same
  measure-space API.
- Riemann integration and Lebesgue integration must not be silently identified.
  Any bridge theorem must name both APIs and its assumptions.

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `MEA-00` | inventory and statement policy | theorem cards and dependency tags |
| `MEA-01` | sigma algebras and measurable spaces | generated sigma algebra API |
| `MEA-02` | basic measures | monotonicity, subadditivity, and continuity of measure |
| `MEA-03` | outer measure and extension | Caratheodory measurable sets and extension interface |
| `MEA-04` | Lebesgue and Lebesgue-Stieltjes measures | real-line measure construction statements |
| `MEA-05` | measurable functions | closure and pointwise-limit measurability |
| `MEA-06` | simple functions and integral construction | nonnegative Lebesgue integral API |
| `MEA-07` | convergence theorem chain | monotone convergence, Fatou, dominated convergence |
| `MEA-08` | product measures | Tonelli, Fubini, sections, and iterated integrals |
| `MEA-09` | pushforwards and change of variables | image-measure and distribution-measure formulas |
| `MEA-10` | signed, complex, and decomposed measures | Hahn, Jordan, total variation, Radon-Nikodym |
| `MEA-11` | Lebesgue regularity and differentiation | regularity, density, differentiation, covering interfaces |
| `MEA-12` | `L^p` spaces and inequalities | Holder, Minkowski, Riesz-Fischer route |
| `MEA-13` | topological and weak-convergence measures | Borel, Radon, Portmanteau, Prokhorov interfaces |
| `MEA-14` | probability, martingale, and ergodic bridges | conditional expectation and recurrence interfaces |
| `MEA-15` | geometric and abstract measure theory | Hausdorff measure and measure-algebra interfaces |
| `MEA-16` | packaging and promotion | stable measure-theory closure batches |

## MEA-00 Inventory And Statement Policy

- Status: planned.
- Depends on: analysis roadmap `ANA-00`.
- Target modules:
  - `Proofs.Ai.Measure.Inventory`
- Theorem order:
  1. classify each theorem from the inventory into exactly one primary
     milestone;
  2. mark duplicates shared with analysis, topology, functional analysis,
     statistics, probability, ergodic theory, or geometric measure theory;
  3. assign each theorem a stable English identifier, theorem level, target
     module, dependencies, and acceptance gate.
- Deliverables:
  - Measure-theory theorem cards.
  - Dependency tags for later `MEA-*` batches.
- Acceptance criteria:
  - Every theorem family has one primary home.
  - Cross-roadmap aliases point to the primary theorem instead of duplicating
    the proof.

## MEA-01 Sigma Algebras And Measurable Spaces

- Status: planned.
- Depends on: basic set and proposition APIs.
- Target modules:
  - `Proofs.Ai.Measure.SigmaAlgebra`
  - `Proofs.Ai.Measure.MeasurableSpace`
  - `Proofs.Ai.Measure.MonotoneClass`
- Theorem order:
  1. sigma-algebra definition and basic closure laws;
  2. empty set, universal set, complement, countable union, countable
     intersection, set difference, and symmetric difference closure;
  3. arbitrary intersection of sigma algebras;
  4. generated sigma algebra and minimality theorem;
  5. Borel sigma algebra as a generated sigma algebra over a topology;
  6. real-line Borel generators by open, closed, half-open, and rational-end
     intervals;
  7. product sigma algebra generated by measurable rectangles;
  8. pi systems, lambda systems, Dynkin pi-lambda theorem;
  9. monotone class theorem.
- Proof strategy:
  - Define generated sigma algebra as the intersection of all sigma algebras
    containing the seed family.
  - Use closure under complement and countable union as the primitive
    sigma-algebra laws, deriving the other set constructors.
  - Use pi-lambda and monotone class principles as reusable proof tools for
    later uniqueness and extension arguments.
- Acceptance criteria:
  - Generated sigma algebra exposes both introduction and minimality lemmas.
  - Borel generator results do not assume real-line measure.
  - Product sigma algebra does not assume product measure.

## MEA-02 Basic Measures

- Status: planned.
- Depends on: `MEA-01`.
- Target modules:
  - `Proofs.Ai.Measure.Basic`
  - `Proofs.Ai.Measure.Completion`
  - `Proofs.Ai.Measure.Restriction`
- Theorem order:
  1. measure definition over a measurable space;
  2. null empty set, finite additivity, and countable additivity for disjoint
     families;
  3. monotonicity;
  4. finite and countable subadditivity;
  5. difference formula under finite-measure hypotheses;
  6. inclusion-exclusion for finite families;
  7. continuity from below;
  8. continuity from above under finite first set;
  9. finite, probability, and sigma-finite measure-space interfaces;
  10. complete measure spaces, completion, null-subset measurability;
  11. restricted measures, subspace measures, measure sums, scalar multiples,
      and monotone limits of measure sequences.
- Proof strategy:
  - Reduce finite additivity and continuity facts to countable additivity by
    decomposing sets into disjoint differences.
  - Make all finiteness assumptions explicit in theorem names and arguments.
  - Treat completion as a construction package until the full null-subset
    derivation is certified.
- Acceptance criteria:
  - Structured theorem statements distinguish finite, sigma-finite, and
    probability specializations.
  - Upper continuity cannot be applied without its finite-measure premise.

## MEA-03 Outer Measure And Extension

- Status: planned.
- Depends on: `MEA-01` and `MEA-02`.
- Target modules:
  - `Proofs.Ai.Measure.Outer`
  - `Proofs.Ai.Measure.Caratheodory`
  - `Proofs.Ai.Measure.Extension`
- Theorem order:
  1. outer measure definition, monotonicity, and countable subadditivity;
  2. Caratheodory measurability criterion;
  3. Caratheodory measurable sets form a sigma algebra;
  4. restriction of an outer measure to Caratheodory measurable sets is a
     measure;
  5. outer measure induced by a premeasure;
  6. Caratheodory extension theorem;
  7. Hahn-Kolmogorov extension theorem;
  8. extension uniqueness under sigma-finiteness;
  9. semiring, ring, and algebra premeasure extension interfaces.
- Proof strategy:
  - Use the Caratheodory equality
    `outer(E) = outer(E cap A) + outer(E \ A)` as the measurable-set predicate.
  - Prove complement and countable-union closure before measure restriction.
  - Isolate uniqueness proofs behind pi-lambda or monotone-class arguments.
- Acceptance criteria:
  - The checker-facing representation is a core AST statement over explicit
    predicates, not a parser-level set expression.
  - Sigma-finite uniqueness hypotheses are explicit.

## MEA-04 Lebesgue And Lebesgue-Stieltjes Measures

- Status: planned.
- Depends on: `MEA-03`, analysis roadmap real-line foundations, and topology
  roadmap Borel foundations.
- Target modules:
  - `Proofs.Ai.Measure.Lebesgue`
  - `Proofs.Ai.Measure.LebesgueStieltjes`
- Theorem order:
  1. Lebesgue outer measure by interval covers;
  2. Lebesgue measurable sets;
  3. Borel sets are Lebesgue measurable;
  4. interval measure equals interval length;
  5. uniqueness of Lebesgue measure under interval and invariance laws;
  6. translation invariance;
  7. scaling law;
  8. countable sets, rationals, Cantor set, and selected null-set examples;
  9. Lebesgue measure as completion of Borel measure;
  10. Vitali nonmeasurable set interface;
  11. distribution functions and Lebesgue-Stieltjes measure construction;
  12. right-continuous monotone functions and measure correspondence;
  13. atoms and jumps of distribution functions;
  14. Cantor distribution and singular-continuous measure interface.
- Proof strategy:
  - First prove interval-cover outer-measure facts, then invoke `MEA-03`.
  - Delay Vitali and Bernstein examples until set-theoretic choice interfaces
    are available.
  - Keep Stieltjes construction separate from Lebesgue measure so probability
    distribution measures can reuse it.
- Acceptance criteria:
  - Lebesgue measurability and Borel measurability remain distinct predicates.
  - Translation and scaling laws are deterministic theorem statements, not
    side-effecting transformations.

## MEA-05 Measurable Functions

- Status: planned.
- Depends on: `MEA-01`, `MEA-02`, and ordered-real foundations.
- Target modules:
  - `Proofs.Ai.Measure.MeasurableFunction`
  - `Proofs.Ai.Measure.SimpleFunction`
- Theorem order:
  1. measurable function definition by preimages;
  2. real-valued measurability criteria by sublevel or superlevel sets;
  3. continuous functions are Borel measurable;
  4. indicator functions and measurable sets;
  5. simple functions and finite measurable partitions;
  6. closure under sum, product, quotient where defined, absolute value,
     positive part, negative part, max, and min;
  7. countable sup, inf, limsup, liminf, and pointwise limits;
  8. a.e. limit measurability;
  9. composition with Borel and measurable maps;
  10. coordinate maps and product-space measurability;
  11. vector-valued componentwise measurability interface;
  12. nonnegative simple-function approximation from below;
  13. bounded and cut-off simple-function approximations.
- Proof strategy:
  - Express pointwise limit operations through countable unions and
    intersections of measurable preimage sets.
  - Build dyadic simple approximants for nonnegative measurable functions.
  - Keep vector-valued statements as interfaces until product and topology
    dependencies are stable.
- Acceptance criteria:
  - Simple-function approximation is monotone when used by the integral route.
  - All quotient theorems include the nonzero-domain premise.

## MEA-06 Lebesgue Integral Construction

- Status: planned.
- Depends on: `MEA-02` and `MEA-05`.
- Target modules:
  - `Proofs.Ai.Measure.Integral.Simple`
  - `Proofs.Ai.Measure.Integral.Nonnegative`
  - `Proofs.Ai.Measure.Integral.Basic`
- Theorem order:
  1. integral of nonnegative simple functions;
  2. nonnegative measurable integral as a supremum of simple integrals;
  3. general integral by positive and negative parts;
  4. integrability and absolute integrability;
  5. positivity and monotonicity;
  6. linearity for integrable functions;
  7. triangle inequality;
  8. a.e.-equal functions have equal integrals;
  9. changing a function on a null set does not change the integral;
  10. nonnegative integral zero implies zero a.e.;
  11. integral over a set by indicator functions;
  12. restriction-measure and set-integral formulas;
  13. integrable truncation and simple-function approximation.
- Proof strategy:
  - Prove simple integral independence from representation before extending to
    measurable functions.
  - Derive a.e. invariance from null-set and monotonicity facts.
  - Isolate extended-real arithmetic assumptions as explicit interfaces until
    numeric foundations are stable.
- Acceptance criteria:
  - General integrability requires finite integrals of both positive and
    negative parts.
  - The integral API distinguishes nonnegative, extended-valued, and finite
    integrable functions.

## MEA-07 Convergence Theorem Chain

- Status: planned.
- Depends on: `MEA-06`.
- Target modules:
  - `Proofs.Ai.Measure.Integral.Convergence`
  - `Proofs.Ai.Measure.Convergence`
  - `Proofs.Ai.Measure.UniformIntegrability`
- Theorem order:
  1. monotone convergence theorem;
  2. Beppo Levi theorem;
  3. Fatou lemma;
  4. dominated convergence theorem;
  5. bounded convergence theorem;
  6. exchange of limit and integral under domination hypotheses;
  7. lower semicontinuity of the integral;
  8. reverse Fatou and Fatou-Lebesgue variants;
  9. Scheffe lemma;
  10. a.e. convergence, convergence in measure, `L^1` and `L^p`
      convergence definitions;
  11. `L^p` convergence implies convergence in measure under finite-measure
      hypotheses;
  12. a.e. convergence implies convergence in measure on finite measure
      spaces;
  13. subsequence principle and Riesz subsequence theorem;
  14. Cauchy in measure;
  15. uniform integrability and Vitali convergence theorem;
  16. de la Vallee-Poussin criterion;
  17. Egorov theorem;
  18. Lusin theorem.
- Proof strategy:
  - Prove monotone convergence directly from the nonnegative integral
    definition and simple approximations.
  - Prove Fatou by applying monotone convergence to increasing infima.
  - Prove dominated convergence by applying Fatou to `g + f_n` and `g - f_n`
    or equivalent nonnegative variants.
  - Delay Vitali and de la Vallee-Poussin until uniform-integrability
    vocabulary is stable.
- Acceptance criteria:
  - Monotone convergence here is the Lebesgue-integral theorem, distinct from
    monotone convergence of sequences.
  - Dominated convergence names the dominating function and its integrability
    premise.

## MEA-08 Product Measures And Fubini-Tonelli

- Status: planned.
- Depends on: `MEA-03`, `MEA-06`, and `MEA-07`.
- Target modules:
  - `Proofs.Ai.Measure.Product`
  - `Proofs.Ai.Measure.Section`
  - `Proofs.Ai.Measure.Fubini`
- Theorem order:
  1. product sigma algebra by measurable rectangles;
  2. existence and uniqueness of product measure;
  3. finite, sigma-finite, probability, finite-product, and countable-product
     measure interfaces;
  4. Kolmogorov extension theorem interface;
  5. measurable sections;
  6. section-measure measurability;
  7. section-integral measurability;
  8. Cavalieri principle;
  9. Tonelli theorem for nonnegative functions;
  10. Fubini theorem for integrable functions;
  11. order-exchange theorem for iterated integrals;
  12. product-space null-set and a.e. section theorems;
  13. convolution integral formula and Young convolution inequality
      interface;
  14. kernel composition formula.
- Proof strategy:
  - Prove rectangle formulas first, then extend by pi-lambda or monotone class.
  - Prove Tonelli for indicator functions, simple functions, and then
    nonnegative measurable functions by monotone convergence.
  - Derive Fubini by applying Tonelli to positive and negative parts.
- Acceptance criteria:
  - Sigma-finiteness hypotheses are explicit for uniqueness and Fubini.
  - Tonelli precedes Fubini in dependency order.

## MEA-09 Pushforwards And Change Of Variables

- Status: planned.
- Depends on: `MEA-05`, `MEA-06`, and selected topology/calculus milestones.
- Target modules:
  - `Proofs.Ai.Measure.Pushforward`
  - `Proofs.Ai.Measure.Distribution`
  - `Proofs.Ai.Measure.ChangeOfVariables`
- Theorem order:
  1. image measure and pushforward definition;
  2. existence of pushforward under measurable maps;
  3. integral formula for pushforwards;
  4. distribution measure and probability-variable specialization;
  5. expectation formula and LOTUS as probability aliases;
  6. measurable isomorphism and measure-preserving transformations;
  7. translation and scaling as change-of-variables examples;
  8. linear transformation formula for Lebesgue measure;
  9. differentiable change-of-variables theorem interface;
  10. polar and spherical coordinate formulas;
  11. coarea, area, and Sard-related interfaces;
  12. density transformation as Radon-Nikodym derivative statement;
  13. disintegration theorem interface.
- Proof strategy:
  - Prove pushforward integration first because many probability formulas are
    aliases of that theorem.
  - Keep differentiable change of variables dependent on derivative,
    determinant, and inverse-function foundations.
  - Treat coarea, area, and disintegration as late interfaces until geometric
    and topological prerequisites exist.
- Acceptance criteria:
  - Pushforward formulas require measurability of the map.
  - Change-of-variables formulas name the exact regularity and nonsingularity
    assumptions.

## MEA-10 Signed, Complex, And Decomposed Measures

- Status: planned.
- Depends on: `MEA-02`, `MEA-06`, and `MEA-07`.
- Target modules:
  - `Proofs.Ai.Measure.Signed`
  - `Proofs.Ai.Measure.Complex`
  - `Proofs.Ai.Measure.Decomposition`
  - `Proofs.Ai.Measure.RadonNikodym`
- Theorem order:
  1. signed measure definition;
  2. positive and negative sets;
  3. Hahn decomposition theorem;
  4. uniqueness of Hahn decomposition modulo null sets;
  5. Jordan decomposition theorem;
  6. positive variation, negative variation, and total variation;
  7. minimality of total variation;
  8. integration with respect to signed measures;
  9. absolute continuity and singularity;
  10. Radon-Nikodym theorem;
  11. uniqueness of Radon-Nikodym derivative;
  12. density representation of measures;
  13. Lebesgue decomposition theorem and uniqueness;
  14. chain rule and change-of-measure formulas for Radon-Nikodym
      derivatives;
  15. conditional expectation as a Radon-Nikodym interface;
  16. complex measures, total variation, polar decomposition, complex
      Radon-Nikodym theorem, and complex Lebesgue decomposition.
- Proof strategy:
  - Establish Hahn decomposition before Jordan and total variation.
  - Prove Radon-Nikodym only after signed-measure and absolute-continuity APIs
    are stable.
  - Keep complex-measure work separate from signed-measure core facts.
- Acceptance criteria:
  - No decomposition theorem assumes the decomposition as a law package; if
    the route is not derivable, split a blocker before source edits.
  - Total variation statements are reusable by functional-analysis duality.

## MEA-11 Lebesgue Regularity And Differentiation

- Status: planned.
- Depends on: `MEA-04`, `MEA-07`, `MEA-08`, and topology compactness
  foundations.
- Target modules:
  - `Proofs.Ai.Measure.Lebesgue.Regularity`
  - `Proofs.Ai.Measure.Lebesgue.Density`
  - `Proofs.Ai.Measure.Lebesgue.Differentiation`
  - `Proofs.Ai.Measure.Covering`
- Theorem order:
  1. outer regularity and inner regularity;
  2. Borel and Lebesgue regularity;
  3. approximation by open, closed, `G_delta`, and `F_sigma` sets;
  4. measurable sets as Borel sets modulo null sets;
  5. Vitali covering theorem interface;
  6. Hardy-Littlewood maximal inequality and maximal theorem;
  7. Lebesgue density theorem;
  8. Lebesgue differentiation theorem;
  9. absolute-continuous function differentiation theorem;
  10. bounded-variation and monotone-function a.e. differentiability
      interfaces;
  11. fundamental theorem of calculus for the Lebesgue integral;
  12. Lebesgue criterion for Riemann integrability.
- Proof strategy:
  - Prove regularity separately from differentiation so topological
    assumptions remain visible.
  - Use covering and maximal-function interfaces as the late bridge to
    density and differentiation.
  - Keep Riemann/Lebesgue comparison theorems explicitly cross-referenced with
    analysis roadmap `ANA-05`.
- Acceptance criteria:
  - Covering lemmas state their metric or Euclidean assumptions.
  - Differentiation theorems do not become prerequisites for basic integration.

## MEA-12 Lp Spaces And Inequalities

- Status: planned.
- Depends on: `MEA-06`, `MEA-07`, and normed-space foundations.
- Target modules:
  - `Proofs.Ai.Measure.Lp.Basic`
  - `Proofs.Ai.Measure.Lp.Inequality`
  - `Proofs.Ai.Measure.Lp.Duality`
  - `Proofs.Ai.Measure.Lp.Interpolation`
- Theorem order:
  1. `L^p` space and a.e. equivalence classes;
  2. `L^p` norm definition and norm laws;
  3. Markov and Chebyshev inequalities;
  4. Jensen inequality;
  5. Young inequality;
  6. Holder inequality;
  7. Cauchy-Schwarz inequality;
  8. Minkowski inequality;
  9. finite-measure `L^p` inclusions;
  10. Riesz-Fischer theorem and completeness;
  11. `L^2` Hilbert-space structure, projection theorem, Bessel and Parseval
      interfaces;
  12. separability and essential supremum;
  13. `L^p` duality for `1 < p < infinity`;
  14. `L^1` duality with `L^infinity` interface;
  15. reflexivity for `1 < p < infinity`;
  16. Dunford-Pettis and weak compactness interfaces;
  17. Riesz-Thorin, Marcinkiewicz, Hausdorff-Young, Plancherel, Sobolev, and
      Rellich-Kondrachov interfaces.
- Proof strategy:
  - Prove Young, Holder, then Minkowski in that order.
  - Prove completeness using Cauchy subsequences, a.e. convergence, and Fatou.
  - Delay duality and interpolation until functional-analysis prerequisites
    are available.
- Acceptance criteria:
  - `L^p` values are quotient/equivalence-class aware.
  - Theorems distinguish `p = infinity`, `p = 1`, and `1 < p < infinity`.

## MEA-13 Topological Measures And Weak Convergence

- Status: planned.
- Depends on: `MEA-04`, `MEA-11`, topology roadmap foundations, and functional
  analysis interfaces.
- Target modules:
  - `Proofs.Ai.Measure.Borel`
  - `Proofs.Ai.Measure.Radon`
  - `Proofs.Ai.Measure.WeakConvergence`
  - `Proofs.Ai.Measure.Selection`
- Theorem order:
  1. Borel measures and regular Borel measures;
  2. Radon measure definition;
  3. Radon measures on locally compact Hausdorff spaces;
  4. Riesz-Markov-Kakutani representation interface;
  5. Polish-space Borel probability regularity;
  6. tightness;
  7. Ulam theorem interface;
  8. Prokhorov theorem interface;
  9. Portmanteau theorem;
  10. weak-convergence characterizations;
  11. Skorokhod representation interface;
  12. Wasserstein weak-convergence interface;
  13. vague convergence and vague compactness;
  14. analytic sets, Suslin theorem, Lusin separation, measurable selection,
      standard Borel spaces, and Lusin-space interfaces.
- Proof strategy:
  - Separate topological regularity from probability weak convergence.
  - Keep representation and selection theorems as interfaces until topological
    and functional prerequisites are certified.
- Acceptance criteria:
  - Every theorem names whether it uses Borel, Radon, Polish, locally compact,
    Hausdorff, or standard Borel assumptions.

## MEA-14 Probability, Martingale, And Ergodic Bridges

- Status: planned.
- Depends on: `MEA-08`, `MEA-10`, `MEA-13`, and the statistics roadmap.
- Target modules:
  - `Proofs.Ai.Measure.ProbabilityBridge`
  - `Proofs.Ai.Measure.ConditionalExpectation`
  - `Proofs.Ai.Measure.Martingale`
  - `Proofs.Ai.Measure.Ergodic`
- Theorem order:
  1. probability spaces as finite measure spaces of total mass one;
  2. random variables as measurable functions;
  3. expectation as Lebesgue integral;
  4. independence by product measure and pi-lambda extension;
  5. conditional expectation existence and uniqueness via Radon-Nikodym;
  6. tower, pull-out, monotonicity, and conditional Jensen properties;
  7. regular conditional probability and disintegration interfaces;
  8. Kolmogorov and Ionescu-Tulcea extension interfaces;
  9. Borel-Cantelli lemmas;
  10. convergence in probability and distribution aliases;
  11. martingale definition, Doob inequalities, optional stopping, upcrossing,
      and martingale convergence interfaces;
  12. measure-preserving transformations, ergodicity, Poincare recurrence,
      Birkhoff and von Neumann ergodic theorem interfaces.
- Proof strategy:
  - Keep probability-specific names as aliases or specializations over measure
    foundations.
  - Use Radon-Nikodym as the primary route to conditional expectation.
  - Delay ergodic and martingale convergence proofs until `L^p` and weak
    convergence prerequisites exist.
- Acceptance criteria:
  - Probability does not introduce a second measure API.
  - Conditional expectation statements include the conditioning sigma algebra.

## MEA-15 Geometric And Abstract Measure Theory

- Status: planned.
- Depends on: `MEA-11`, topology, metric, and functional-analysis
  prerequisites.
- Target modules:
  - `Proofs.Ai.Measure.Geometric`
  - `Proofs.Ai.Measure.Algebra`
- Theorem order:
  1. Hausdorff measure and Hausdorff dimension definitions;
  2. basic Hausdorff-measure properties;
  3. packing measure interface;
  4. Frostman lemma interface;
  5. Marstrand and Besicovitch-Federer projection interfaces;
  6. area and coarea formulas;
  7. Rademacher theorem and Lipschitz a.e. differentiability interface;
  8. Kirszbraun extension interface;
  9. rectifiability, finite-perimeter, BV compactness, isoperimetric, and
      Preiss theorem interfaces;
  10. measure algebra by quotienting null sets;
  11. complete Boolean algebra and measure relation;
  12. atomic and nonatomic decompositions;
  13. Maharam, Lebesgue-space isomorphism, standard probability-space
      classification, Stone, and Loomis-Sikorski interfaces.
- Proof strategy:
  - Treat geometric-measure and measure-algebra results as late routes, not
    prerequisites for basic integration.
  - Start with statement cards and evidence packages until metric and
    topological dependencies are strong enough for derived certificates.
- Acceptance criteria:
  - These late theorems do not add trusted kernel primitives.
  - Interfaces state which construction evidence remains external.

## MEA-16 Packaging And Promotion

- Status: planned.
- Depends on: stable `MEA-*` batches.
- Target areas:
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/*`
  - `develop/npa-mathlib-next-closure-roadmap.md`
- Deliverables:
  - Stable measure-theory module batches.
  - Source-free module verification.
  - Axiom report review.
  - Public closure audit candidates when a batch is ready for
    `npa-mathlib`.
- Acceptance criteria:
  - Closure batches are small enough to audit.
  - The axiom report does not grow unexpectedly.
  - Package verification is run before downstream roadmaps rely on the
    measure modules.

## Core Dependency Route

The shortest high-value abstract integration route through the inventory is:

```text
sigma algebras
-> basic measures
-> outer measure
-> Caratheodory extension
-> measurable functions
-> simple-function approximation
-> Lebesgue integral
-> monotone convergence theorem
-> Fatou lemma
-> dominated convergence theorem
-> product measure
-> Tonelli theorem
-> Fubini theorem
-> signed measures
-> Radon-Nikodym theorem
-> Lebesgue decomposition theorem
```

The real-line construction route branches from Caratheodory extension:

```text
Caratheodory extension
-> Lebesgue measure
-> Lebesgue-Stieltjes measure
-> regularity, density, and change-of-variables theorems
```

This route should be completed before Fourier, PDE, probability,
measure-theoretic statistics, martingale, or ergodic milestones claim their
measure-theoretic results as fully derived `L2` theorems.

## Cross-Roadmap Ownership

| Theorem family | Primary home | Alias or consumer |
| --- | --- | --- |
| measure and Lebesgue integration foundations | this roadmap `MEA-01` through `MEA-08` | analysis `ANA-08` |
| Fubini, Tonelli, dominated convergence | this roadmap | statistics and probability roadmaps |
| Radon-Nikodym and Lebesgue decomposition | this roadmap `MEA-10` | probability conditional expectation, functional analysis duality |
| Borel, Radon, Polish weak-convergence theorems | this roadmap `MEA-13` with topology dependencies | statistics asymptotics and probability |
| `L^p`, Holder, Minkowski, Riesz-Fischer | this roadmap `MEA-12` with functional-analysis dependencies | analysis `ANA-09`, Fourier `ANA-11`, PDE `ANA-13` |
| Lebesgue differentiation and covering theorems | this roadmap `MEA-11` | real analysis, harmonic analysis, PDE |
| martingale and ergodic theorems | this roadmap as bridge interfaces, statistics/probability as consumers | statistics roadmap |
| geometric measure theory | this roadmap `MEA-15` | topology, PDE, variational analysis |

## Initial Execution Queue

Start with small batches that create reusable APIs without requiring the entire
classical measure stack:

1. `MEA-T00`: create theorem cards and target-module names for the inventory.
2. `MEA-T01`: create the namespace skeleton and statement policy.
3. `MEA-T02`: add `SigmaAlgebra` statement and closure-law interface.
4. `MEA-T03`: add generated sigma algebra and minimality theorem.
5. `MEA-T04`: add pi-lambda and monotone-class theorem interfaces.
6. `MEA-T05`: add measurable-space and product-sigma interfaces.
7. `MEA-T06`: add `MeasureSpace` and basic additivity theorem targets.
8. `MEA-T07`: derive monotonicity, subadditivity, and difference laws.
9. `MEA-T08`: derive continuity from below and above under explicit
   hypotheses.
10. `MEA-T09`: add `OuterMeasure` and Caratheodory measurable-set statements.
11. `MEA-T10`: prove Caratheodory measurable sets form a sigma algebra.

After `MEA-T10`, continue through extension uniqueness (`MEA-T11` and
`MEA-T12`), then decide whether to prioritize the abstract integration route
(`MEA-T16` through `MEA-T25`) or the real-line Lebesgue-measure branch
(`MEA-T13` through `MEA-T15`). The choice should be made from actual
dependency pressure in the corpus, not from the size of the theorem inventory.

## Verification Checklist

- Documentation-only changes:
  - `git diff --check`
- Module authoring batches:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.X`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.X --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`
- Package or public closure batches:
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## Risks And Guardrails

| Risk | Impact | Guardrail |
| --- | --- | --- |
| measure theory becomes a trusted kernel feature | expands trusted base | keep measure structures in proof corpus modules |
| high-level theorems land as circular law packages | false sense of proof completion | require proof-level labels and circularity audits |
| probability duplicates measure APIs | incompatible downstream statements | make probability a specialization over this route |
| product measure and Fubini are attempted before convergence theorems | unstable proofs and broad rewrites | finish simple integral and convergence chain first |
| Radon-Nikodym is attempted before signed measures | circular decomposition route | complete Hahn, Jordan, and total variation APIs first |
| topology-dependent regularity theorems land too early | hidden assumptions | keep Borel, Radon, Polish, compactness, and Euclidean hypotheses explicit |

## Open Decisions

- Whether the first derived integral route should prioritize abstract measure
  spaces or real-line Lebesgue measure examples.
- Whether extended real numbers should be a dedicated numeric layer or an
  explicit law package used only by measure modules at first.
- Whether Caratheodory extension should be fully derived before nonnegative
  integral work, or initially recorded as dependency-map work.
- How much of probability's conditional expectation API should live in
  `Proofs.Ai.Measure.*` versus `Proofs.Ai.Statistics.*`.
- Which measure-theory batch is the first realistic candidate for
  `npa-mathlib` public closure promotion.
