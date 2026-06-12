# Topology Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided topology theorem inventory
one theorem at a time in the NPA proof corpus. It is a planning sidecar, not
proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- topological spaces, open sets, closed sets, neighborhoods, closure, interior,
  boundary, dense sets, bases, subbases, initial topology, and final topology;
- continuous maps, open maps, closed maps, embeddings, quotient maps, pasting
  lemmas, function spaces, and compact-open topology;
- homeomorphisms, topological invariants, non-homeomorphism criteria, and
  high-level invariants such as fundamental groups, homology, Euler
  characteristic, manifold dimension, invariance of domain, and Jordan-type
  separation theorems;
- separation axioms, regularity, normality, Urysohn lemma, Tietze extension,
  metrizability theorems, and compactifications;
- compactness, metric compactness, connectedness, countability axioms,
  separability, Lindelof properties, metric spaces, completions, Baire spaces,
  product spaces, quotient spaces, local properties, paracompactness, nets,
  filters, and ultrafilters;
- homotopy, fundamental groups, covering spaces, homology, cohomology, CW
  complexes, simplicial complexes, manifolds, differential topology,
  low-dimensional topology, fixed-point theorems, dimension theory,
  topological dynamics, characteristic classes, K-theory, spectral sequences,
  and stable homotopy interfaces.

The plan is intentionally staged. The first priority is not to encode every
named theorem immediately, but to build reusable general-topology
foundations. Later algebraic, differential, and geometric topology statements
must import those foundations and must not replace them with incompatible
definitions of space, map, compactness, quotient, product, homotopy, or
manifold.

## Existing Baseline

The current proof corpus already has reusable metric, analysis, algebra,
geometry, and category routes that should be reused instead of recreated:

| Corpus module | Existing role |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | metric balls, neighborhoods, local predicates, local equality, local uniqueness |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | normed-space law packages and product norm estimates |
| `Proofs.Ai.Analysis.AbstractFixedPoint` | completeness evidence, contractions, fixed-point evidence, Banach fixed-point theorem package |
| `Proofs.Ai.Analysis.AbstractLinearMap` | bounded linear maps, operator bounds, and linear isomorphism packages |
| `Proofs.Ai.Analysis.AbstractDerivative` | Frechet derivative, differentiability, uniqueness, derivative rule packages |
| `Proofs.Ai.Analysis.AbstractInverseFunction` | local inverse evidence and quantitative inverse-function theorem package |
| `Proofs.Ai.Analysis.AbstractImplicitFunction` | implicit-function extraction, uniqueness, differentiability, derivative formula package |
| `Proofs.Ai.Geometry.AbstractMetric` | abstract metric and metric-distance theorem targets |
| `Proofs.Ai.Geometry.Pythagorean` | checked geometric Pythagorean theorem names |
| `Proofs.Ai.Category.Classical` | category-theory theorem targets useful for functorial statements |
| `Proofs.Ai.Category.Infinity.SimplicialSet` | higher-categorical and simplicial-set interface targets |
| `Proofs.Ai.Algebra.AbstractGroup` and related group modules | group, subgroup, quotient, isomorphism, and correspondence theorem targets |

There is not yet a checked concrete `Proofs.Ai.Topology.*` tree. The analysis
roadmap already reserves early topology work under `ANA-07` and task
milestones `ANA-T22` and `ANA-T23`:

| Analysis task | Relevance here |
| --- | --- |
| `ANA-T22` | first `Proofs.Ai.Topology.Basic` and `Proofs.Ai.Topology.Metric.Compact` route |
| `ANA-T23` | Baire theorem and function-space topology route |

This topology roadmap makes `Proofs.Ai.Topology.*` the primary home for
general topology and algebraic topology. Analysis modules should import these
theorems after they exist, while existing analysis milestones remain valid as
the initial compactness, Baire, and function-space authoring route.

For compatibility with the existing analysis roadmap, `ANA-07`, `ANA-T22`, and
`ANA-T23` are treated as initial authoring aliases for the same
`Proofs.Ai.Topology.*` modules. Once this roadmap is active, theorem cards
should point primary ownership to the relevant `TOP-*` milestone and leave
analysis milestones as importing or specialization routes.

The current `develop/npa-mathlib-next-closure-roadmap.md` records public
materialization through `npa-mathlib v0.1.27`, including metric-topology,
normed-space, linear-map, derivative, fixed-point, inverse-function, and
implicit-function closures. New topology work should build on those closures
instead of widening the trusted kernel.

The measure-theory roadmap in `proofs/measure-theory-theorem-proof-roadmap.md`
now refines analysis `ANA-08` into `MEA-*` tasks. Topology milestones that
mention measure recurrence, Lebesgue-style integration, Borel/Radon measures,
or weak convergence should import that detailed measure route rather than
treating `ANA-T24` through `ANA-T26` as the only dependency plan.

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant or shape theorem only | no |
| `L1 Evidence package` | theorem conclusion follows from explicit construction, topology, cover, homotopy, chain, or manifold evidence | no for pending theorem-proof tasks; use only as a blocker/dependency note |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

For broad existence theorems such as Tychonoff, Stone-Cech compactification,
universal covering spaces, Eilenberg-Steenrod theories, Poincare duality,
Whitney embedding, transversality, and spectral sequence convergence,
dependency-map entries are useful first records. They must keep all
construction evidence explicit and must not be confused with derived theorems.

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
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
only for package-wide compatibility, promotion, release readiness, or changes
to certificate encoding, checker behavior, package verification, or kernel
semantics.

## Statement Policy

Topology theorem statements must keep these boundaries explicit:

- Topological spaces, bases, subbases, open sets, closed sets, covers,
  products, quotients, compactness, connectedness, homotopy, chains,
  cochains, manifolds, bundles, and spectra are ordinary structures and
  theorem-level predicates, not kernel primitives.
- Set-theoretic principles are explicit. Choice, ultrafilter lemma,
  Tychonoff, Zorn-style arguments, paracompact refinements, and maximal
  constructions must record their evidence or remain blocker work.
- Metric topology should reuse `Proofs.Ai.Analysis.AbstractMetricTopology`
  and existing metric closures. It must not fork a second incompatible metric
  neighborhood vocabulary.
- Analysis interval, Euclidean, Baire, Arzela-Ascoli, Banach fixed point,
  open mapping, closed graph, inverse function, and implicit function
  theorems remain analysis/functional-analysis primary results unless this
  roadmap explicitly owns a general-topological abstraction.
- Algebraic topology statements must keep basepoints, path components,
  homotopies, chain complexes, coefficient groups, exactness evidence, and
  functoriality explicit.
- Differential topology statements must keep smooth structure, chart,
  tangent, transversality, orientation, boundary, and compactness assumptions
  explicit. Smooth theorems are not consequences of bare topological
  manifolds unless the statement says so.
- Low-dimensional topology, characteristic classes, K-theory, spectra, and
  index-theorem statements are late interfaces until the algebraic,
  differential, and geometric prerequisites exist.

## Duplicate Theorem Policy

Several theorem names appear in multiple inventory sections. Each duplicate
must have one primary home, with other modules importing or aliasing it:

| Theorem family | Primary home |
| --- | --- |
| open/closed/interior/closure/boundary/neighborhood/base/subbase/topology axioms | `TOP-01` through `TOP-02` |
| continuity, pasting, maps into products, quotient-induced maps | `TOP-03`, with product aliases from `TOP-11` and quotient aliases from `TOP-12` |
| homeomorphism invariance of compactness, connectedness, separation, countability | `TOP-04`, importing primary properties from later milestones |
| T0, T1, Hausdorff, regular, normal, Urysohn, Tietze | `TOP-05` |
| compactness, Alexander subbase theorem, Tychonoff, compact Hausdorff normality | `TOP-06`, with product-space alias from `TOP-11` |
| Heine-Borel, Bolzano-Weierstrass, Lebesgue number lemma, metric compactness | `TOP-07`, coordinated with analysis `ANA-T05`, `ANA-T12`, and `ANA-T22` |
| connectedness, path connectedness, components, Jordan curve theorem | `TOP-08`, with Jordan-Brouwer and manifold aliases later |
| countability, separability, Lindelof, metrizability, Polish basics | `TOP-09` and `TOP-10` |
| completion and Baire category | `TOP-10`; open mapping, closed graph, and uniform boundedness stay primary in analysis `ANA-T27` and import Baire |
| products and Tychonoff-style product compactness | `TOP-11`, importing compactness from `TOP-06` |
| quotient spaces, gluing spaces, CW quotient constructions | `TOP-12`, with CW aliases from `TOP-20` |
| local compactness, local connectedness, paracompactness, partition of unity | `TOP-13`, with manifold aliases from `TOP-21` |
| nets, filters, ultrafilters, Moore-Smith convergence, compactness via filters | `TOP-14` |
| homotopy, deformation retracts, homotopy equivalence | `TOP-15` |
| fundamental groups and van Kampen | `TOP-16`, importing group results from algebra modules |
| covering spaces, lifting, monodromy, universal covers | `TOP-17` |
| homology, exact sequences, excision, Mayer-Vietoris, Hurewicz | `TOP-18` |
| cohomology, cup products, universal coefficients, Poincare duality, characteristic classes | `TOP-19`, with characteristic-class details in `TOP-27` |
| simplicial complexes, CW complexes, cellular approximation, cellular homology | `TOP-20` |
| topological manifolds, invariance of domain, topological manifold basics | `TOP-21` |
| smooth manifolds, Sard, regular value, transversality, Morse theory | `TOP-22`, coordinated with analysis derivative/inverse/implicit routes |
| surface classification, 3-manifold and knot interfaces | `TOP-23` |
| Brouwer, Schauder, Lefschetz, Borsuk-Ulam, hairy ball, Poincare-Hopf | `TOP-24`, importing homology/cohomology or manifold routes as needed |
| covering dimension, inductive dimensions, Hilbert cube, ANR dimension | `TOP-25` |
| topological dynamics, recurrence, symbolic dynamics, structural stability | `TOP-26`, with measure/differential aliases from analysis when needed |
| de Rham, Stokes, Gauss-Bonnet, Chern-Weil, Hodge, index-theorem interfaces | `TOP-27`, coordinated with analysis, geometry, and differential topology |
| K-theory, spectra, stable homotopy, spectral sequences, Brown representation | `TOP-28` |

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `TOP-00` | inventory and statement policy | theorem cards, duplicate map, target levels |
| `TOP-01` | topological-space foundations | open/closed/neighborhood/interior/closure/boundary API |
| `TOP-02` | generated, relative, initial, and final topologies | bases, subbases, subspaces, generated topologies |
| `TOP-03` | continuous maps and map classes | continuity characterizations, pasting, embeddings, quotient-induced maps |
| `TOP-04` | homeomorphisms and invariants | preservation and non-homeomorphism theorem cards |
| `TOP-05` | separation axioms and normality | T0/T1/Hausdorff, regular, normal, Urysohn, Tietze route |
| `TOP-06` | general compactness | open-cover compactness, finite intersection property, Tychonoff interface |
| `TOP-07` | metric compactness and function spaces | Heine-Borel route, Lebesgue number, Arzela-Ascoli interface |
| `TOP-08` | connectedness and path connectedness | connected components, path components, IVT aliases, Jordan interfaces |
| `TOP-09` | countability, separability, Lindelof, and metrizability | first/second countable, separable metric, Urysohn metrization route |
| `TOP-10` | complete metric and Baire spaces | completion, Baire, Polish basics, functional-analysis aliases |
| `TOP-11` | product spaces | product topology universal property, product preservation theorems |
| `TOP-12` | quotient spaces and gluing | quotient topology universal property and standard quotient models |
| `TOP-13` | local properties and paracompactness | local compactness/connectedness, refinements, partition of unity |
| `TOP-14` | nets, filters, and ultrafilters | closure/compactness/convergence via nets and filters |
| `TOP-15` | homotopy foundations | homotopy, homotopy equivalence, retracts, contractibility |
| `TOP-16` | fundamental groups | loop group, functoriality, circle, van Kampen route |
| `TOP-17` | covering spaces | lifting, universal covers, subgroup correspondence |
| `TOP-18` | homology | chain complexes, homotopy invariance, exact sequences, computations |
| `TOP-19` | cohomology | cochain functor, cup product, duality interfaces |
| `TOP-20` | simplicial and CW complexes | geometric realization, CW weak topology, cellular homology |
| `TOP-21` | topological manifolds | manifold basics, dimension/invariance-of-domain interfaces |
| `TOP-22` | differential topology | Sard, regular value, transversality, Morse routes |
| `TOP-23` | surfaces and low-dimensional topology | surface classification and 3-manifold/knot interfaces |
| `TOP-24` | fixed-point and degree-style theorem families | Brouwer, Lefschetz, Borsuk-Ulam, Poincare-Hopf routes |
| `TOP-25` | dimension theory | covering and inductive dimensions, dimension invariance route |
| `TOP-26` | topological dynamics | conjugacy, recurrence, symbolic dynamics, stability interfaces |
| `TOP-27` | geometric topology and characteristic classes | Stokes, de Rham, Gauss-Bonnet, characteristic class interfaces |
| `TOP-28` | K-theory, spectral sequences, stable homotopy | Bott, spectral sequence, Brown representation interfaces |
| `TOP-29` | packaging and promotion | stable `npa-mathlib` closure audits |

## TOP-00 Inventory And Statement Policy

- Status: planned.
- Depends on: none.
- Deliverables:
  - Convert the theorem inventory into theorem cards.
  - Give every theorem a stable English identifier, Japanese display name,
    target level, dependencies, target module, and acceptance gate.
  - Mark duplicates across analysis, geometry, algebra, category theory,
    algebraic topology, differential topology, and low-dimensional topology.
- Acceptance criteria:
  - Every theorem has one primary home module.
  - Duplicates point to the primary theorem instead of being reproved.
  - Each card states whether the first target is a statement, evidence
    package, derived certificate, or public closure.
- Verification:
  - Documentation diff review.
  - `git diff --check`.

## TOP-01 Topological-Space Foundations

- Status: planned.
- Depends on: set/predicate foundations and existing equality/logical theorem
  targets.
- Target modules:
  - `Proofs.Ai.Topology.Basic`
  - `Proofs.Ai.Topology.Closure`
- Theorem order:
  1. topological-space law package;
  2. open-set and closed-set definitions;
  3. neighborhoods and neighborhood-system characterization;
  4. interior, closure, exterior, boundary;
  5. closure/interior duality and boundary formulas;
  6. limit points, isolated points, dense sets;
  7. closure-operator characterization and Kuratowski closure axioms.
- Deliverables:
  - General topology vocabulary used by all later milestones.
- Acceptance criteria:
  - Open and closed predicates are ordinary structures, not kernel primitives.
  - Closure and interior laws are derived from the selected topology package,
    or explicitly marked as the package interface.
  - Metric neighborhoods from `AbstractMetricTopology` are bridged without
    replacing the general topology vocabulary.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Closure`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-02 Generated, Relative, Initial, And Final Topologies

- Status: planned.
- Depends on: `TOP-01`.
- Target modules:
  - `Proofs.Ai.Topology.Generated`
  - `Proofs.Ai.Topology.Subspace`
  - `Proofs.Ai.Topology.InitialFinal`
- Theorem order:
  1. subspace topology and open/closed subset characterizations;
  2. basis and subbasis generated topology;
  3. topology comparison theorem;
  4. initial topology universal property;
  5. final topology universal property.
- Deliverables:
  - Generated-topology layer for products, quotients, embeddings, and
    compact-open topology.
- Acceptance criteria:
  - Subspace topology is not duplicated in manifold or metric modules.
  - Generated topology proofs state cover/refinement evidence explicitly.
  - Initial/final topology universal properties do not assume continuity
    criteria from `TOP-03` unless imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Generated`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Subspace`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-03 Continuous Maps And Map Classes

- Status: planned.
- Depends on: `TOP-01` and `TOP-02`.
- Target modules:
  - `Proofs.Ai.Topology.Continuous`
  - `Proofs.Ai.Topology.MapClass`
- Theorem order:
  1. continuity by open preimages;
  2. continuity by closed preimages;
  3. neighborhood and closure characterizations;
  4. identity, composition, restriction, and local continuity;
  5. pasting lemma;
  6. open map, closed map, homeomorphism, and embedding predicates;
  7. maps into products and maps out of quotients as dependency-tagged hooks;
  8. compact-open topology dependency-map entry.
- Deliverables:
  - Continuous-map theorem layer for the entire roadmap.
- Acceptance criteria:
  - Pasting lemma side conditions specify closed or open cover hypotheses.
  - Embedding distinguishes injective continuous map from homeomorphism onto
    image.
  - Product and quotient characterizations are aliases once `TOP-11` and
    `TOP-12` exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Continuous`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.MapClass`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-04 Homeomorphisms And Topological Invariants

- Status: planned.
- Depends on: `TOP-03`; later preservation theorems also depend on the
  property milestone they preserve.
- Target modules:
  - `Proofs.Ai.Topology.Homeomorphism`
  - `Proofs.Ai.Topology.Invariant`
- Theorem order:
  1. homeomorphism equivalence and inverse continuity criteria;
  2. open and closed set correspondence under homeomorphism;
  3. property-preservation framework for compactness, connectedness,
     path-connectedness, separation, countability, homotopy, homology, Euler
     characteristic, and manifold dimension;
  4. non-homeomorphism theorem-card interfaces.
- Deliverables:
  - Homeomorphism and invariant alias framework.
- Acceptance criteria:
  - Preservation theorems import the primary theorem for the preserved
    property; this module does not define duplicate compactness or homology.
  - Invariance of domain, dimension invariance, Jordan separation, and Euler
    characteristic remain interfaces until their primary milestones exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homeomorphism`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Invariant`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-05 Separation Axioms, Normality, And Urysohn Routes

- Status: planned.
- Depends on: `TOP-01`, `TOP-02`, and `TOP-03`.
- Target modules:
  - `Proofs.Ai.Topology.Separation.Basic`
  - `Proofs.Ai.Topology.Separation.Normal`
  - `Proofs.Ai.Topology.Separation.Urysohn`
- Theorem order:
  1. T0, Kolmogorov, T1, Hausdorff predicates and characterizations;
  2. T1 singleton and finite-set closedness;
  3. Hausdorff uniqueness of sequence/net limits;
  4. Hausdorff subspace/product routes;
  5. diagonal closed iff Hausdorff route;
  6. compact subsets of Hausdorff spaces are closed;
  7. regular, completely regular, normal, and Tychonoff predicates;
  8. metric spaces are normal;
  9. compact Hausdorff spaces are normal;
  10. Urysohn lemma;
  11. Tietze extension theorem;
  12. Stone-Cech compactification interface.
- Deliverables:
  - Separation and normality theorem layer.
- Acceptance criteria:
  - Hausdorff uniqueness results for nets wait for `TOP-14` if nets are used.
  - Urysohn and Tietze identify normality and codomain assumptions explicitly.
  - Stone-Cech stays as a dependency-map entry until ultrafilters and function-algebra
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Separation.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Separation.Urysohn`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-06 General Compactness

- Status: planned.
- Depends on: `TOP-01`, `TOP-03`, and selected set/choice evidence.
- Target modules:
  - `Proofs.Ai.Topology.Compact.Basic`
  - `Proofs.Ai.Topology.Compact.Product`
  - `Proofs.Ai.Topology.Compactification`
- Theorem order:
  1. open-cover compactness definition;
  2. finite-intersection-property characterization;
  3. closed subsets of compact spaces are compact;
  4. continuous image of compact space is compact;
  5. compactness as homeomorphism invariant;
  6. compact-to-Hausdorff continuous bijection is homeomorphism;
  7. tube lemma;
  8. Alexander subbase theorem;
  9. Tychonoff theorem;
  10. one-point/Alexandroff compactification;
  11. compactness via nets, filters, and ultrafilters after `TOP-14`.
- Deliverables:
  - General compactness theorem layer for products, quotients, manifolds, and
    analysis aliases.
- Acceptance criteria:
  - Tychonoff and Alexander subbase theorem record choice/subbase evidence.
  - Compact Hausdorff normality imports `TOP-05`.
  - Metric compactness theorems are primary in `TOP-07`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Compact.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Compact.Product`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-07 Metric Compactness And Function Spaces

- Status: planned.
- Depends on: `TOP-06`, existing `Proofs.Ai.Analysis.AbstractMetricTopology`,
  and analysis sequence compactness milestones `ANA-T05` and `ANA-T22`.
- Target modules:
  - `Proofs.Ai.Topology.Metric.Compact`
  - `Proofs.Ai.Topology.FunctionSpace`
- Theorem order:
  1. compact metric spaces are complete;
  2. compact metric spaces are totally bounded;
  3. compact metric iff complete and totally bounded;
  4. sequential compactness equivalence in metric spaces;
  5. Heine-Borel theorem route;
  6. Bolzano-Weierstrass alias from analysis sequence compactness;
  7. Lebesgue number lemma;
  8. compact metric spaces are separable;
  9. compact-open topology basics;
  10. Arzela-Ascoli theorem interface.
- Deliverables:
  - Metric compactness and function-space topology layer.
- Acceptance criteria:
  - Metric topology reuses `AbstractMetricTopology`.
  - Heine-Borel identifies Euclidean, normed-space, and sequence compactness
    prerequisites.
  - Arzela-Ascoli states compactness and equicontinuity hypotheses explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metric.Compact`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FunctionSpace`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-08 Connectedness And Path Connectedness

- Status: planned.
- Depends on: `TOP-01`, `TOP-03`, and real interval foundations from analysis
  for interval-specific theorems.
- Target modules:
  - `Proofs.Ai.Topology.Connected.Basic`
  - `Proofs.Ai.Topology.Connected.Path`
  - `Proofs.Ai.Topology.Continuum`
- Theorem order:
  1. connectedness definition and clopen characterization;
  2. continuous image of connected space;
  3. closure of connected subset;
  4. union theorems for connected families;
  5. product connectedness;
  6. real interval connectedness and IVT aliases;
  7. connected components are closed;
  8. local connectedness and open components;
  9. path-connected implies connected;
  10. path components and local path-connected equivalence;
  11. totally disconnected spaces and Cantor set interfaces;
  12. Peano continuum, Hahn-Mazurkiewicz, Jordan curve, and
      Jordan-Brouwer separation interfaces.
- Deliverables:
  - Connectedness theorem layer for algebraic topology and manifolds.
- Acceptance criteria:
  - Interval and IVT results import analysis real/continuity foundations.
  - Jordan and continuum theorems stay as blocker work until plane, manifold,
    and homology prerequisites exist.
  - Component and path-component predicates are not conflated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Connected.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Connected.Path`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-09 Countability, Separability, Lindelof, And Metrizability

- Status: planned.
- Depends on: `TOP-01`, `TOP-03`, `TOP-05`, and `TOP-07` for metric
  specializations.
- Target modules:
  - `Proofs.Ai.Topology.Countability`
  - `Proofs.Ai.Topology.Metrization`
  - `Proofs.Ai.Topology.Examples.Sorgenfrey`
- Theorem order:
  1. first countable and second countable characterizations;
  2. second countable implies first countable;
  3. second countable implies separable and Lindelof;
  4. separable metric iff second countable;
  5. Lindelof closed subspaces and continuous images;
  6. sigma-compact implies Lindelof;
  7. first countable closure by sequences;
  8. Frechet-Urysohn characterization;
  9. Polish-space second countability;
  10. Sorgenfrey line and plane theorem-card examples;
  11. Urysohn, Nagata-Smirnov, Bing, Moore, and Smirnov metrizability routes.
- Deliverables:
  - Countability and metrizability theorem layer.
- Acceptance criteria:
  - General separability inheritance claims are not stated without hypotheses.
  - Metrization theorems identify regularity, normality, sigma-local
    finiteness, developability, or Moore-space assumptions explicitly.
  - Example spaces are statement cards until their topology constructions are
    available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Countability`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metrization`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-10 Complete Metric And Baire Spaces

- Status: planned.
- Depends on: `TOP-07`, existing `Proofs.Ai.Analysis.AbstractFixedPoint`, and
  analysis `ANA-T23` for the initial Baire authoring route.
- Target modules:
  - `Proofs.Ai.Topology.Metric.Completion`
  - `Proofs.Ai.Topology.Baire`
- Theorem order:
  1. Cauchy sequence and complete metric-space interfaces;
  2. completion existence and uniqueness;
  3. closed subspaces of complete metric spaces are complete;
  4. Cantor intersection theorem;
  5. Banach fixed point alias from `AbstractFixedPoint`;
  6. Baire category theorem;
  7. complete metric spaces, locally compact Hausdorff spaces, and Polish
     spaces are Baire;
  8. dense open countable intersections are dense;
  9. nowhere dense, meagre, comeagre, and generic property lemmas;
  10. Choquet and Banach-Mazur game interfaces;
  11. open mapping, closed graph, and uniform boundedness aliases from
      functional analysis.
- Deliverables:
  - Complete metric and Baire theorem layer.
- Acceptance criteria:
  - Banach fixed point remains primary in analysis fixed-point modules.
  - Functional-analysis theorems import Baire rather than reprove it.
  - Game-theoretic Baire statements stay as dependency-map entries until game definitions exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Metric.Completion`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Baire`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-11 Product Spaces

- Status: planned.
- Depends on: `TOP-02`, `TOP-03`, `TOP-05`, `TOP-06`, `TOP-08`, and
  `TOP-09`.
- Target modules:
  - `Proofs.Ai.Topology.Product.Basic`
  - `Proofs.Ai.Topology.Product.Properties`
- Theorem order:
  1. product topology definition;
  2. product topology universal property;
  3. projections are continuous;
  4. maps into products continuity criterion;
  5. basis for product topology;
  6. finite product basic opens;
  7. products preserve Hausdorff, regular, completely regular, connected, and
     path-connected properties under stated hypotheses;
  8. product compactness and Tychonoff alias;
  9. finite and infinite product local compactness criteria;
  10. countable product metrizability, Polishness, second countability, and
      separability routes;
  11. product closure formula.
- Deliverables:
  - Product-space theorem layer.
- Acceptance criteria:
  - Product compactness imports `TOP-06` and records choice evidence.
  - Countable product metric results import countability/metrizability
    hypotheses.
  - Product closure formula states indexing and projection assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Product.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Product.Properties`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-12 Quotient Spaces And Gluing

- Status: planned.
- Depends on: `TOP-02`, `TOP-03`, `TOP-05`, `TOP-06`, and `TOP-08`.
- Target modules:
  - `Proofs.Ai.Topology.Quotient.Basic`
  - `Proofs.Ai.Topology.Quotient.Models`
- Theorem order:
  1. quotient topology definition and universal property;
  2. quotient-map continuity criteria;
  3. open and closed set characterizations;
  4. open and closed quotient map theorems;
  5. compact, connected, and path-connected quotients;
  6. Hausdorff quotient conditions and closed equivalence-relation graph
     route;
  7. gluing-space theorem;
  8. circle, sphere, torus, projective space, and CW quotient model
     interfaces.
- Deliverables:
  - Quotient theorem layer used by algebraic topology and manifolds.
- Acceptance criteria:
  - Failure of Hausdorff preservation is recorded as an example card, not as a
    universal negative theorem without model data.
  - Standard quotient models stay as dependency-map entries until their concrete spaces exist.
  - CW quotient construction imports `TOP-20` when promoted beyond an
    interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Quotient.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Quotient.Models`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-13 Local Properties And Paracompactness

- Status: planned.
- Depends on: `TOP-05`, `TOP-06`, `TOP-08`, `TOP-09`, and `TOP-11`.
- Target modules:
  - `Proofs.Ai.Topology.Local`
  - `Proofs.Ai.Topology.Paracompact`
  - `Proofs.Ai.Topology.PartitionOfUnity`
- Theorem order:
  1. local compactness, local connectedness, and local path-connectedness;
  2. local compact Hausdorff properties and one-point compactification alias;
  3. local component openness theorems;
  4. paracompactness definition;
  5. compact spaces and metric spaces are paracompact;
  6. paracompact Hausdorff spaces are normal;
  7. locally finite open refinement theorem;
  8. partition of unity subordinate to an open cover;
  9. manifold partition-of-unity alias;
  10. Michael selection, Morita, Stone paracompactification, and
      Nagata-Smirnov paracompactness interfaces.
- Deliverables:
  - Local and paracompact theorem layer for manifolds and analysis.
- Acceptance criteria:
  - Partition of unity states paracompactness, Hausdorffness, local finiteness,
    and codomain/ring assumptions.
  - Selection and paracompactification theorems stay as dependency-map entries until selection
    machinery exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Local`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Paracompact`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-14 Nets, Filters, And Ultrafilters

- Status: planned.
- Depends on: `TOP-01`, `TOP-03`, `TOP-05`, `TOP-06`, and selected
  choice/ultrafilter evidence.
- Target modules:
  - `Proofs.Ai.Topology.Net`
  - `Proofs.Ai.Topology.Filter`
  - `Proofs.Ai.Topology.Ultrafilter`
- Theorem order:
  1. net and filter convergence definitions;
  2. closure and closed set characterization by nets;
  3. continuity by net convergence;
  4. uniqueness of net limits in Hausdorff spaces;
  5. compactness via convergent subnets;
  6. compactness via filters and ultrafilters;
  7. Tychonoff ultrafilter proof interface;
  8. Moore-Smith convergence, subnet existence, universal net;
  9. ultrafilter lemma;
  10. Stone-Cech compactification via ultrafilters;
  11. product and quotient convergence notes;
  12. first countable spaces need only sequences.
- Deliverables:
  - General convergence theorem layer.
- Acceptance criteria:
  - Ultrafilter lemma and Tychonoff record set-theoretic evidence.
  - Sequence sufficiency imports first-countability facts from `TOP-09`.
  - Stone-Cech imports both normality/function and ultrafilter prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Net`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Filter`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-15 Homotopy Foundations

- Status: planned.
- Depends on: `TOP-03`, `TOP-08`, `TOP-11`, and `TOP-12`.
- Target modules:
  - `Proofs.Ai.Topology.Homotopy.Basic`
  - `Proofs.Ai.Topology.Homotopy.Retract`
- Theorem order:
  1. homotopy definition;
  2. homotopy equivalence and equivalence relation proof;
  3. contractible spaces and basic properties;
  4. deformation retract and strong deformation retract;
  5. homotopy extension and lifting property interfaces;
  6. CW homotopy extension alias;
  7. Whitehead, Hurewicz, Freudenthal, Blakers-Massey, and Brown
     representation interfaces.
- Deliverables:
  - Homotopy vocabulary and early theorem layer.
- Acceptance criteria:
  - Homotopy invariance of fundamental group and homology is primary in
    `TOP-16` and `TOP-18`.
  - Advanced homotopy theorems stay as dependency-map entries until CW, homology, and spectral
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homotopy.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homotopy.Retract`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-16 Fundamental Groups

- Status: planned.
- Depends on: `TOP-08`, `TOP-12`, `TOP-15`, and algebra group modules.
- Target modules:
  - `Proofs.Ai.Topology.FundamentalGroup.Basic`
  - `Proofs.Ai.Topology.FundamentalGroup.VanKampen`
  - `Proofs.Ai.Topology.FundamentalGroup.Computation`
- Theorem order:
  1. based loops and path homotopy;
  2. fundamental group definition and group structure;
  3. basepoint-change isomorphism;
  4. functoriality;
  5. homotopy equivalence induces group isomorphism;
  6. circle fundamental group route;
  7. sphere, product, wedge, graph, torus, projective space, Klein bottle, and
     surface computation interfaces;
  8. Seifert-van Kampen theorem;
  9. two-cell attachment quotient theorem;
  10. Brouwer fixed point and Borsuk-Ulam application interfaces.
- Deliverables:
  - Fundamental group theorem layer.
- Acceptance criteria:
  - Group laws import algebra group theorem targets.
  - Basepoints and path-connectedness hypotheses are explicit.
  - Van Kampen states open-cover and intersection conditions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FundamentalGroup.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FundamentalGroup.VanKampen`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-17 Covering Spaces

- Status: planned.
- Depends on: `TOP-03`, `TOP-08`, `TOP-12`, `TOP-15`, and `TOP-16`.
- Target modules:
  - `Proofs.Ai.Topology.Covering.Basic`
  - `Proofs.Ai.Topology.Covering.Lifting`
  - `Proofs.Ai.Topology.Covering.Classification`
- Theorem order:
  1. covering map definition and local homeomorphism theorem;
  2. path lifting;
  3. homotopy lifting;
  4. unique lifting;
  5. lifting criterion;
  6. covering spaces and fundamental group subgroup correspondence;
  7. universal cover existence and uniqueness;
  8. simply connected covering classification;
  9. deck transformation group and regular/Galois covering characterization;
  10. circle, torus, projective space, and Klein bottle covering examples;
  11. monodromy action.
- Deliverables:
  - Covering-space theorem layer for fundamental group computations.
- Acceptance criteria:
  - Semilocally simply connected and local path-connected hypotheses are
    explicit for universal-cover existence.
  - Classification theorems import fundamental group results from `TOP-16`.
  - Example covers remain interfaces until the model spaces exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Covering.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Covering.Lifting`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-18 Homology

- Status: planned.
- Depends on: `TOP-12`, `TOP-15`, `TOP-16`, `TOP-17`, algebra group/modules,
  and chain-complex infrastructure.
- Target modules:
  - `Proofs.Ai.Topology.Homology.Singular`
  - `Proofs.Ai.Topology.Homology.Exact`
  - `Proofs.Ai.Topology.Homology.Computation`
- Theorem order:
  1. simplex, singular simplex, chain group, and boundary definitions;
  2. boundary squared is zero;
  3. homology group definition;
  4. functoriality;
  5. homotopy invariance;
  6. long exact sequence and pair long exact sequence;
  7. excision;
  8. Mayer-Vietoris sequence;
  9. simplicial and singular homology comparison;
  10. Eilenberg-Steenrod axiom interface;
  11. sphere, torus, projective space, surface, and CW homology
      computations;
  12. Hurewicz, Kunneth, universal coefficient, Poincare duality,
      Lefschetz duality, Alexander duality, and Thom isomorphism interfaces.
- Deliverables:
  - Homology theorem layer.
- Acceptance criteria:
  - Boundary-square-zero is derived before homology groups are used.
  - Exactness evidence is explicit.
  - Duality and Kunneth results are late interfaces until coefficient,
    chain-level, and manifold prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homology.Singular`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Homology.Exact`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-19 Cohomology

- Status: planned.
- Depends on: `TOP-18`, algebra modules, and later manifold/orientation
  foundations for duality.
- Target modules:
  - `Proofs.Ai.Topology.Cohomology.Singular`
  - `Proofs.Ai.Topology.Cohomology.CupProduct`
  - `Proofs.Ai.Topology.Cohomology.Duality`
- Theorem order:
  1. singular cohomology definition;
  2. functoriality;
  3. homotopy invariance;
  4. cohomology long exact sequence;
  5. Mayer-Vietoris cohomology sequence;
  6. cup product definition;
  7. cohomology ring theorem;
  8. universal coefficient theorem for cohomology;
  9. Kunneth theorem;
  10. Cech cohomology interface;
  11. de Rham theorem interface;
  12. Poincare, Lefschetz, Alexander, and Thom duality interfaces;
  13. Gysin sequence and characteristic class interfaces;
  14. Steenrod operations and spectral sequence interfaces.
- Deliverables:
  - Cohomology theorem layer for characteristic classes and fixed-point
    theorems.
- Acceptance criteria:
  - Cup product signs, degrees, and coefficient assumptions are explicit.
  - de Rham and characteristic class theorems are coordinated with `TOP-27`.
  - Spectral sequence entries are interfaces until `TOP-28`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Cohomology.Singular`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Cohomology.CupProduct`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-20 Simplicial And CW Complexes

- Status: planned.
- Depends on: `TOP-12`, `TOP-15`, `TOP-18`, and category/simplicial-set
  interfaces.
- Target modules:
  - `Proofs.Ai.Topology.SimplicialComplex`
  - `Proofs.Ai.Topology.CWComplex.Basic`
  - `Proofs.Ai.Topology.CWComplex.Cellular`
- Theorem order:
  1. simplicial complex definition and geometric realization;
  2. barycentric subdivision;
  3. simplicial approximation theorem interface;
  4. CW complex definition, weak topology, and closure finiteness;
  5. CW homotopy extension property;
  6. cellular approximation theorem;
  7. cellular homology theorem;
  8. Whitehead and CW approximation theorem interfaces;
  9. product and quotient theorems for CW complexes;
  10. skeleton filtration;
  11. attaching-map homology computation;
  12. Euler characteristic from cell decomposition;
  13. finite CW compactness;
  14. local contractibility and ANR interfaces.
- Deliverables:
  - CW/simplicial theorem layer.
- Acceptance criteria:
  - Weak topology and quotient construction import `TOP-12`.
  - Cellular homology imports homology foundations from `TOP-18`.
  - Whitehead theorem remains interface until homotopy and homology
    prerequisites are stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CWComplex.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CWComplex.Cellular`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-21 Topological Manifolds

- Status: planned.
- Depends on: `TOP-05`, `TOP-08`, `TOP-09`, `TOP-12`, `TOP-13`, and
  Euclidean topology foundations from analysis/linear algebra.
- Target modules:
  - `Proofs.Ai.Topology.Manifold.Topological`
  - `Proofs.Ai.Topology.Manifold.Invariance`
- Theorem order:
  1. topological manifold definition;
  2. local Euclidean property;
  3. Hausdorff second-countable manifold properties;
  4. local compactness, local connectedness, and local path-connectedness;
  5. paracompact manifold partition-of-unity alias;
  6. manifold dimension invariance interface;
  7. invariance of domain interface;
  8. Jordan-Brouwer separation alias.
- Deliverables:
  - Topological manifold theorem layer.
- Acceptance criteria:
  - Manifold dimension invariance and invariance of domain are not assumed by
    the manifold definition.
  - Euclidean-space topology assumptions are imported explicitly.
  - Smooth structure is deferred to `TOP-22`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Topological`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Invariance`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-22 Differential Topology

- Status: planned.
- Depends on: `TOP-21`, analysis derivative/inverse/implicit milestones,
  linear algebra foundations, and smooth-manifold infrastructure.
- Target modules:
  - `Proofs.Ai.Topology.Manifold.Smooth`
  - `Proofs.Ai.Topology.Differential.Sard`
  - `Proofs.Ai.Topology.Differential.Transversality`
  - `Proofs.Ai.Topology.Morse`
- Theorem order:
  1. smooth manifold, chart, tangent, smooth map, submersion, immersion, and
     embedding interfaces;
  2. Whitney embedding and approximation theorem interfaces;
  3. Sard theorem;
  4. regular value theorem;
  5. inverse and implicit function theorem aliases from analysis;
  6. submanifold theorem;
  7. transversality theorem and Thom transversality interface;
  8. Ehresmann fibration theorem interface;
  9. Morse lemma;
  10. Morse inequalities and Morse theory route;
  11. h-cobordism, s-cobordism, smooth Poincare, surgery, and
      Kirby-Siebenmann obstruction interfaces.
- Deliverables:
  - Differential topology theorem interfaces and early derived routes.
- Acceptance criteria:
  - Sard, transversality, and Morse statements state smoothness, dimension,
    regularity, compactness, and boundary hypotheses.
  - Inverse/implicit theorem aliases import existing analysis modules rather
    than restating their proofs.
  - Surgery and cobordism statements remain late dependency-map entries.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Manifold.Smooth`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Differential.Sard`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-23 Surfaces And Low-Dimensional Topology

- Status: planned.
- Depends on: `TOP-16`, `TOP-18`, `TOP-20`, `TOP-21`, and `TOP-22` where
  smooth/PL methods are used.
- Target modules:
  - `Proofs.Ai.Topology.Surface.Classification`
  - `Proofs.Ai.Topology.ThreeManifold.Interfaces`
  - `Proofs.Ai.Topology.Knot.Basic`
- Theorem order:
  1. compact surface classification theorem interface;
  2. orientable and non-orientable closed surface classification;
  3. surface Euler characteristic formula;
  4. surface fundamental group presentations;
  5. surface homology computations;
  6. Jordan curve and Schoenflies interfaces;
  7. Dehn lemma, loop theorem, sphere theorem interfaces;
  8. Kneser-Milnor prime decomposition and JSJ interfaces;
  9. geometrization, hyperbolization, Mostow rigidity, and Dehn surgery
     interfaces;
  10. Reidemeister moves, Alexander theorem, Markov theorem, Jones polynomial
      invariance, and knot group interfaces.
- Deliverables:
  - Low-dimensional topology statement and interface layer.
- Acceptance criteria:
  - Surface classification imports fundamental group, homology, and CW
    prerequisites.
  - Three-manifold and knot theorems stay as dependency-map entries until manifold, PL/smooth,
    and algebraic invariants exist.
  - Poincare conjecture is not treated as a foundational axiom.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Surface.Classification`
  - `rg -n "Poincare|JSJ|Jones|Surface.Classification" proofs/topology-theorem-proof-roadmap.md`
  - `git diff --check`

## TOP-24 Fixed-Point And Degree-Style Theorems

- Status: planned.
- Depends on: `TOP-10`, `TOP-16`, `TOP-18`, `TOP-19`, `TOP-21`, and
  analysis fixed-point/convexity foundations where needed.
- Target modules:
  - `Proofs.Ai.Topology.FixedPoint.Brouwer`
  - `Proofs.Ai.Topology.FixedPoint.Lefschetz`
  - `Proofs.Ai.Topology.FixedPoint.BorsukUlam`
- Theorem order:
  1. Banach fixed point alias from analysis;
  2. Brouwer fixed point theorem;
  3. Schauder fixed point interface;
  4. Lefschetz fixed point theorem;
  5. Kakutani, Tarski, Knaster-Tarski, Caristi, Markov-Kakutani, Nielsen,
     Eilenberg-Montgomery, and Fan-Browder interfaces;
  6. Borsuk-Ulam theorem;
  7. hairy ball theorem;
  8. Poincare-Hopf theorem.
- Deliverables:
  - Fixed-point theorem route organized by proof prerequisites.
- Acceptance criteria:
  - Banach fixed point remains primary in `Proofs.Ai.Analysis.AbstractFixedPoint`.
  - Brouwer, Borsuk-Ulam, Lefschetz, and Poincare-Hopf identify whether they
    use degree, homology, cohomology, or manifold orientation evidence.
  - Order-theoretic fixed-point theorems import order/lattice foundations when
    available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FixedPoint.Brouwer`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.FixedPoint.Lefschetz`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-25 Dimension Theory

- Status: planned.
- Depends on: `TOP-06`, `TOP-09`, `TOP-18`, `TOP-19`, and `TOP-21`.
- Target modules:
  - `Proofs.Ai.Topology.Dimension.Covering`
  - `Proofs.Ai.Topology.Dimension.Inductive`
  - `Proofs.Ai.Topology.Dimension.Invariance`
- Theorem order:
  1. covering dimension definition;
  2. small and large inductive dimension definitions;
  3. Euclidean topological dimension theorem interface;
  4. dimension invariance and invariance-of-domain aliases;
  5. Lebesgue covering dimension theorem;
  6. Menger-Nobeling embedding theorem interface;
  7. Hurewicz dimension-lowering theorem interface;
  8. Urysohn dimension theorem;
  9. separation characterization of dimension;
  10. product dimension inequality;
  11. compact metric dimension theory;
  12. Peano continuum dimension route;
  13. Hilbert cube, infinite-dimensional topology, ANR dimension, covering
      dimension and cohomology, Alexandroff theorem, and Pontryagin surface
      interfaces.
- Deliverables:
  - Dimension theory statement and derived theorem layer.
- Acceptance criteria:
  - Covering dimension is not conflated with vector-space or manifold
    dimension.
  - Cohomological dimension statements import cohomology prerequisites.
  - Invariance of domain remains primary in manifold/dimension route, not a
    general homeomorphism axiom.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dimension.Covering`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dimension.Invariance`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-26 Topological Dynamics

- Status: planned.
- Depends on: `TOP-03`, `TOP-06`, `TOP-08`, `TOP-10`, `TOP-15`,
  measure/probability foundations for measure recurrence, and differential
  topology for smooth dynamics.
- Target modules:
  - `Proofs.Ai.Topology.Dynamics.Basic`
  - `Proofs.Ai.Topology.Dynamics.Symbolic`
  - `Proofs.Ai.Topology.Dynamics.Stability`
- Theorem order:
  1. topological dynamical system definition;
  2. topological conjugacy theorem;
  3. orbit closure properties;
  4. minimal set existence theorem;
  5. Birkhoff recurrence interface;
  6. Poincare recurrence alias from measure theory;
  7. transitivity and mixing characterizations;
  8. symbolic dynamics and shift-space properties;
  9. Smale horseshoe and Sharkovsky interfaces;
  10. Brouwer translation theorem;
  11. Lefschetz fixed point alias;
  12. Conley index, structural stability, shadowing, stable manifold,
      Hartman-Grobman, and Morse-Smale interfaces.
- Deliverables:
  - Topological dynamics theorem interfaces.
- Acceptance criteria:
  - Measure recurrence does not land before measure/probability foundations.
  - Poincare recurrence aliases wait for the measure roadmap's ergodic route
    `MEA-T51`, which refines analysis `ANA-T24` through `ANA-T26`, or
    probability/process routes `STAT-T55` through `STAT-T57`.
  - Stable manifold and Hartman-Grobman routes import differential/ODE
    prerequisites.
  - Symbolic dynamics defines shift spaces using product topology from
    `TOP-11`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Dynamics.Basic`
  - `rg -n "Poincare recurrence|Hartman|Morse-Smale|Dynamics|MEA-T51" proofs/topology-theorem-proof-roadmap.md proofs/measure-theory-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md proofs/statistics-theorem-proof-roadmap*.md`
  - `git diff --check`

## TOP-27 Geometric Topology And Characteristic Classes

- Status: planned.
- Depends on: `TOP-19`, `TOP-21`, `TOP-22`, analysis differential forms,
  measure/integration foundations, and geometry routes.
- Target modules:
  - `Proofs.Ai.Topology.DifferentialForms.Stokes`
  - `Proofs.Ai.Topology.DeRham`
  - `Proofs.Ai.Topology.CharacteristicClass`
  - `Proofs.Ai.Topology.IndexTheory.Interfaces`
- Theorem order:
  1. Stokes theorem interface;
  2. de Rham theorem;
  3. Gauss-Bonnet and Chern-Gauss-Bonnet interfaces;
  4. Poincare-Hopf alias from fixed-point/manifold route;
  5. Hodge decomposition and Hodge theorem interfaces;
  6. Riemann-Roch and Hirzebruch-Riemann-Roch interfaces;
  7. Chern-Weil theory;
  8. Chern, Euler, Pontryagin, and Stiefel-Whitney class theorems;
  9. Thom isomorphism and Pontryagin-Thom construction;
  10. cobordism ring interface;
  11. Atiyah-Singer index theorem interface;
  12. Bott periodicity alias from `TOP-28`.
- Deliverables:
  - Differential-form, de Rham, characteristic-class, and index interfaces.
- Acceptance criteria:
  - Differential-form integration is not assumed before analysis integration
    and manifold orientation foundations.
  - Stokes and de Rham routes wait for Riemann/Lebesgue/differential-form
    infrastructure from analysis milestones `ANA-T16` through `ANA-T18` and
    the measure roadmap's integration route `MEA-T19` through `MEA-T25`,
    which refines `ANA-T24` through `ANA-T26`.
  - Characteristic classes state bundle, coefficient, naturality, and
    obstruction-theory assumptions.
  - Index and Riemann-Roch theorems stay as dependency-map entries until analytic and K-theory
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.DeRham`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.CharacteristicClass`
  - `rg -n "Stokes|de Rham|MEA-T19|MEA-T25" proofs/topology-theorem-proof-roadmap.md proofs/measure-theory-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap*.md`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-28 K-Theory, Spectral Sequences, And Stable Homotopy

- Status: planned.
- Depends on: `TOP-18`, `TOP-19`, `TOP-20`, `TOP-27`, category theory, and
  stable homotopy infrastructure.
- Target modules:
  - `Proofs.Ai.Topology.KTheory.Basic`
  - `Proofs.Ai.Topology.SpectralSequence.Basic`
  - `Proofs.Ai.Topology.StableHomotopy.Interfaces`
- Theorem order:
  1. vector bundle classification and clutching construction interfaces;
  2. K-theory definition;
  3. Bott periodicity;
  4. Thom isomorphism in K-theory;
  5. Atiyah-Hirzebruch, Serre, Adams, and Eilenberg-Moore spectral sequence
     interfaces;
  6. Brown representation theorem;
  7. stable homotopy category interface;
  8. Freudenthal suspension and stable Hurewicz theorem aliases;
  9. Spanier-Whitehead duality;
  10. Postnikov and Whitehead tower interfaces;
  11. obstruction theory;
  12. spectral sequence convergence theorem;
  13. Eilenberg-Mac Lane spaces and homotopy groups of spheres interfaces.
- Deliverables:
  - Late-stage high topology interface layer.
- Acceptance criteria:
  - Spectral sequences state filtration, exact couple, convergence, and
    grading assumptions explicitly.
  - K-theory does not assume vector bundle classification without bundle
    infrastructure.
  - Stable homotopy interfaces do not enter earlier theorem proofs as
    axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.KTheory.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.SpectralSequence.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
## TOP-29 Packaging And Promotion

- Status: planned.
- Depends on: any completed stable theorem batch from `TOP-01` through
  `TOP-28`.
- Target areas:
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/*`
  - `develop/npa-mathlib-next-closure-roadmap.md`
- Deliverables:
  - Closure audits and `npa-mathlib` promotion notes for stable topology
    module clusters.
  - Updated theorem indexes, axiom reports, package metadata, and publish-plan
    entries only when closure is clean.
- Acceptance criteria:
  - Axiom report does not gain unintended axioms.
  - Source-free verifier and package checks pass for the promoted closure.
  - Public closure documentation states which theorem families are included
    and excluded.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## Recommended First Execution Queue

The first batch should focus on general topology foundations that unlock many
later theorem families:

| Queue ID | Theorem or task | Target level | Primary milestone |
| --- | --- | --- | --- |
| `TOQ-001` | theorem-card inventory and duplicate map | `L0` | `TOP-00` |
| `TOQ-002` | topological-space law package, open/closed predicates | `L2`; split blockers first | `TOP-01` |
| `TOQ-003` | interior, closure, boundary, dense and limit-point facts | `L2` | `TOP-01` |
| `TOQ-004` | basis, subbasis, generated topology | `L2` | `TOP-02` |
| `TOQ-005` | subspace topology and relative open/closed facts | `L2` | `TOP-02` |
| `TOQ-006` | continuity by open/closed preimages and composition | `L2` | `TOP-03` |
| `TOQ-007` | pasting lemma and embedding/homeomorphism basics | `L2` | `TOP-03`, `TOP-04` |
| `TOQ-008` | T0/T1/Hausdorff basics and diagonal theorem | `L2` | `TOP-05` |
| `TOQ-009` | compactness open-cover and finite-intersection characterizations | `L2` | `TOP-06` |
| `TOQ-010` | compact closed subsets, continuous images, compact-to-Hausdorff homeomorphism | `L2` | `TOP-06` |
| `TOQ-011` | metric compactness bridge and compact metric completeness | `L2` | `TOP-07` |
| `TOQ-012` | connectedness basics and continuous image of connected spaces | `L2` | `TOP-08` |
| `TOQ-013` | first/second countable, separable, Lindelof basics | `L2` | `TOP-09` |
| `TOQ-014` | product topology universal property and projections | `L2` | `TOP-11` |
| `TOQ-015` | quotient topology universal property | `L2` | `TOP-12` |
| `TOQ-016` | complete metric space and Baire interface | `L2`; split blockers first | `TOP-10` |
| `TOQ-017` | nets and filters statement API | `L2`; split blockers first | `TOP-14` |
| `TOQ-018` | homotopy and contractibility basics | `L2` | `TOP-15` |
| `TOQ-019` | fundamental group statement API and loop group structure | `L2`; split blockers first | `TOP-16` |
| `TOQ-020` | singular chain boundary-square-zero statement route | `L2`; split blockers first | `TOP-18` |

After `TOQ-020`, choose based on project priority:

- continue to `TOP-05`, `TOP-06`, `TOP-09`, and `TOP-14` for Urysohn, Tietze,
  Tychonoff, Stone-Cech, and metrization theorem routes;
- continue to `TOP-16` and `TOP-17` for circle fundamental group, van Kampen,
  and covering spaces;
- continue to `TOP-18` through `TOP-20` for homology, cohomology, and CW
  complexes;
- continue to `TOP-21` and `TOP-22` for manifold and differential topology
  routes;
- continue to `TOP-24` for Brouwer, Lefschetz, Borsuk-Ulam, hairy ball, and
  Poincare-Hopf theorem families.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| General topology and analysis topology APIs diverge | compactness, Baire, and metric results cannot be shared | make `TOP-01`/`TOP-07` bridge existing `AbstractMetricTopology` and analysis `ANA-T22`/`ANA-T23` |
| Choice principles are hidden | Tychonoff, ultrafilter, paracompactness, and maximal constructions become circular | record choice/ultrafilter/Zorn evidence or keep the target as blocker work |
| Product and quotient topologies are defined separately in later areas | algebraic topology and manifold modules duplicate basic topology | make `TOP-11` and `TOP-12` the primary home |
| Homotopy, fundamental group, and homology land before exact algebra foundations | algebraic topology proofs cannot be checked cleanly | import algebra group and chain-complex infrastructure explicitly |
| Differential topology assumes smooth structure from topological manifolds | smooth theorems become ill-scoped | keep topological manifolds in `TOP-21` and smooth/manifold calculus in `TOP-22` |
| Low-dimensional and high-topology named theorems land too early | roadmap becomes a list of unchecked axioms | keep them as interfaces until prerequisite clusters are certified |
| Fixed-point theorems are duplicated across analysis and topology | Banach, Brouwer, Schauder, and Lefschetz aliases conflict | keep Banach primary in analysis and degree/homology fixed-point families in `TOP-24` |
| Public promotion is attempted before closure audit | unintended axioms or package drift enter `npa-mathlib` | require `TOP-29` closure audit and package gates before promotion |

## Decision Points

- Decide the first general `TopologySpace` law-package shape before `TOP-01`
  lands, including whether open sets or closure operators are primary.
- Decide how `Proofs.Ai.Analysis.AbstractMetricTopology` embeds into the
  general topology vocabulary before metric compactness work starts.
- Decide the set-theoretic evidence strategy for ultrafilters, Tychonoff,
  Stone-Cech, and paracompactness before those theorems move into `L2`.
- Decide coefficient groups, chain-complex representation, grading, and
  exactness evidence before homology and cohomology modules land.
- Decide the model of simplicial complexes and CW complexes before cellular
  homology and Whitehead theorem interfaces depend on it.
- Decide topological versus smooth manifold namespace boundaries before
  Whitney, Sard, transversality, and Morse theorem routes land.
- Before any `L3` promotion, run closure audit and choose package gates
  according to changed artifacts.
