# Topology Theorem Cards

Source roadmaps:

- `proofs/topology-theorem-proof-roadmap.md`
- `proofs/topology-theorem-proof-roadmap-todo.md`

This file is the `TOP-T00` theorem-card inventory for the topology proof
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
| Display | Human-facing English family name for roadmap review. |
| Level | Initial target level from the roadmap: `L0 Statement`, `L1 Evidence package`, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary milestones | `TOP-T*` task milestones that own the first formalization. |
| Proposed modules | Planned `Proofs.Ai.Topology.*` entry points or explicitly external owners. |
| Kind | `foundation`, `derived theorem`, `specialization`, `package alias`, or `long-term interface`. |
| Evidence | Main set-theoretic or construction evidence that must be explicit in statements. |
| Dependencies | Roadmap or module families that this card imports or waits for. |
| Gate | First acceptance gate for the card. |

Large structural theorem families are never marked `L2` until their prerequisite
definitions and intermediate lemmas are certificate-backed. Choice, ultrafilter,
Zorn-style, transversality, manifold, integration, homology, cohomology,
spectral-sequence, and stable-homotopy statements stay `L1` interfaces unless
their evidence is explicit and source-free verified.

## Namespace Contract

Primary concrete entry point planned:

```text
Proofs.Ai.Topology.Inventory
```

This inventory module is a future certificate-backed policy entry point, not a
mathematical foundation by itself. Its role is to preserve:

| Contract item | Evidence it should preserve |
| --- | --- |
| `topological_object_structure_policy` | spaces, open sets, bases, products, quotients, covers, paths, chains, manifolds, bundles, and spectra are ordinary proof-corpus structures |
| `analysis_alias_policy` | analysis compactness, Baire, and function-space aliases import topology-owned cards instead of duplicating them |
| `set_theoretic_evidence_policy` | choice, ultrafilter, Zorn-style, maximal, and paracompact refinement assumptions are named explicitly |
| `algebraic_topology_dependency_policy` | basepoints, path components, coefficients, chain complexes, exactness, and functoriality are ordinary theorem evidence |
| `differential_topology_dependency_policy` | smoothness, chart, tangent, orientation, boundary, compactness, and integration assumptions are stated before Stokes, de Rham, and index interfaces |
| `derived_target_certificate_policy` | derived theorem targets require source-free certificate-verdict evidence |

Namespace ownership rules:

- `Proofs.Ai.Topology.*` owns general topology, algebraic topology,
  topological manifolds, differential-topology interfaces, geometric topology,
  characteristic-class interfaces, K-theory interfaces, spectral-sequence
  interfaces, and stable-homotopy interfaces.
- `Proofs.Ai.Analysis.*` remains the primary owner for interval and Euclidean
  analysis specializations, Banach fixed point, inverse and implicit function
  theorem packages, open mapping, closed graph, uniform boundedness, and
  Sobolev/PDE analytic theorem families.
- `Proofs.Ai.Measure.*`, probability, and statistics roadmaps own measure
  recurrence, ergodic, integration, weak-convergence, and probabilistic
  process facts. Topology dynamics may alias those facts only after checked
  measure or probability foundations exist.
- Algebra modules own group, module, coefficient, quotient-group, exactness
  carrier, and representation facts. Topology modules import those facts for
  fundamental group, homology, cohomology, K-theory, and characteristic-class
  statements.
- Category and higher-category modules own categorical infrastructure and
  simplicial-set interfaces. Topology modules may consume them only through
  checked certificates or explicitly marked `L1` interfaces.

## Primary Roadmap Cards

| Card | Stable id | Display | Level | Primary milestones | Proposed modules | Kind |
| --- | --- | --- | --- | --- | --- | --- |
| `TOP-00` | `topology_inventory_statement_policy` | Topology roadmap inventory | `L0 Statement` | `TOP-T00` | `Proofs.Ai.Topology.Inventory` | foundation |
| `TOP-01` | `topological_space_foundation` | Topological space foundations | `L1 Evidence package` then `L2 Derived certificate` | `TOP-T01` through `TOP-T02` | `Proofs.Ai.Topology.Basic`, `Proofs.Ai.Topology.Closure` | foundation |
| `TOP-02` | `generated_relative_initial_final_topologies` | Generated, relative, initial, and final topologies | `L1` then `L2` | `TOP-T03` through `TOP-T04` | `Proofs.Ai.Topology.Generated`, `Proofs.Ai.Topology.Subspace`, `Proofs.Ai.Topology.InitialFinal` | foundation |
| `TOP-03` | `continuous_maps_and_map_classes` | Continuous maps and map classes | `L2 Derived certificate` where prerequisites exist | `TOP-T05` through `TOP-T06` | `Proofs.Ai.Topology.Continuous`, `Proofs.Ai.Topology.MapClass` | derived theorem |
| `TOP-04` | `homeomorphism_and_topological_invariants` | Homeomorphisms and topological invariants | `L2` with interface aliases | `TOP-T07` | `Proofs.Ai.Topology.Homeomorphism`, `Proofs.Ai.Topology.Invariant` | derived theorem, long-term interface |
| `TOP-05` | `separation_axioms_normality_urysohn` | Separation axioms, normality, and Urysohn-style results | `L1` then `L2` | `TOP-T08` through `TOP-T09` | `Proofs.Ai.Topology.Separation.Basic`, `Proofs.Ai.Topology.Separation.Normal`, `Proofs.Ai.Topology.Separation.Urysohn` | foundation, long-term interface |
| `TOP-06` | `general_compactness` | General compactness | `L1` then `L2` | `TOP-T10` through `TOP-T11` | `Proofs.Ai.Topology.Compact.Basic`, `Proofs.Ai.Topology.Compact.Product`, `Proofs.Ai.Topology.Compactification` | foundation, long-term interface |
| `TOP-07` | `metric_compactness_and_function_spaces` | Metric compactness and function spaces | `L2` plus `L1` interfaces | `TOP-T12` through `TOP-T13` | `Proofs.Ai.Topology.Metric.Compact`, `Proofs.Ai.Topology.FunctionSpace` | specialization, long-term interface |
| `TOP-08` | `connectedness_and_path_connectedness` | Connectedness and path-connectedness | `L2` plus `L1` interfaces | `TOP-T14` through `TOP-T15` | `Proofs.Ai.Topology.Connected.Basic`, `Proofs.Ai.Topology.Connected.Path`, `Proofs.Ai.Topology.Continuum` | foundation, long-term interface |
| `TOP-09` | `countability_separability_lindelof_metrization` | Countability, separability, Lindelofness, and metrizability | `L2` plus `L1` interfaces | `TOP-T16` through `TOP-T17` | `Proofs.Ai.Topology.Countability`, `Proofs.Ai.Topology.Metrization`, `Proofs.Ai.Topology.Examples.Sorgenfrey` | foundation, long-term interface |
| `TOP-10` | `complete_metric_and_baire_spaces` | Complete metric spaces and Baire spaces | `L1` then `L2` | `TOP-T18` through `TOP-T19` | `Proofs.Ai.Topology.Metric.Completion`, `Proofs.Ai.Topology.Baire` | foundation, derived theorem |
| `TOP-11` | `product_spaces` | Product spaces | `L2` plus `L1` aliases | `TOP-T20` through `TOP-T21` | `Proofs.Ai.Topology.Product.Basic`, `Proofs.Ai.Topology.Product.Properties` | derived theorem, long-term interface |
| `TOP-12` | `quotient_spaces_and_gluing` | Quotient spaces and gluing | `L2` plus `L1` models | `TOP-T22` through `TOP-T23` | `Proofs.Ai.Topology.Quotient.Basic`, `Proofs.Ai.Topology.Quotient.Models` | derived theorem, long-term interface |
| `TOP-13` | `local_properties_and_paracompactness` | Local properties and paracompactness | `L1` then `L2` | `TOP-T24` through `TOP-T25` | `Proofs.Ai.Topology.Local`, `Proofs.Ai.Topology.Paracompact`, `Proofs.Ai.Topology.PartitionOfUnity` | foundation, long-term interface |
| `TOP-14` | `nets_filters_and_ultrafilters` | Nets, filters, and ultrafilters | `L1` then `L2` | `TOP-T26` through `TOP-T27` | `Proofs.Ai.Topology.Net`, `Proofs.Ai.Topology.Filter`, `Proofs.Ai.Topology.Ultrafilter` | foundation, long-term interface |
| `TOP-15` | `homotopy_foundations` | Homotopy foundations | `L1` then `L2` | `TOP-T28` through `TOP-T29` | `Proofs.Ai.Topology.Homotopy.Basic`, `Proofs.Ai.Topology.Homotopy.Retract` | foundation, long-term interface |
| `TOP-16` | `fundamental_groups` | Fundamental groups | `L1` then `L2` | `TOP-T30` through `TOP-T31` | `Proofs.Ai.Topology.FundamentalGroup.Basic`, `Proofs.Ai.Topology.FundamentalGroup.VanKampen`, `Proofs.Ai.Topology.FundamentalGroup.Computation` | foundation, derived theorem |
| `TOP-17` | `covering_spaces` | Covering spaces | `L1` then `L2` | `TOP-T32` through `TOP-T33` | `Proofs.Ai.Topology.Covering.Basic`, `Proofs.Ai.Topology.Covering.Lifting`, `Proofs.Ai.Topology.Covering.Classification` | foundation, derived theorem |
| `TOP-18` | `homology` | Homology | `L1` then `L2` | `TOP-T34` through `TOP-T36` | `Proofs.Ai.Topology.Homology.Singular`, `Proofs.Ai.Topology.Homology.Exact`, `Proofs.Ai.Topology.Homology.Computation` | foundation, derived theorem, long-term interface |
| `TOP-19` | `cohomology` | Cohomology | `L1` then `L2` | `TOP-T37` through `TOP-T38` | `Proofs.Ai.Topology.Cohomology.Singular`, `Proofs.Ai.Topology.Cohomology.CupProduct`, `Proofs.Ai.Topology.Cohomology.Duality` | foundation, long-term interface |
| `TOP-20` | `simplicial_and_cw_complexes` | Simplicial complexes and CW complexes | `L1` then `L2` | `TOP-T39` through `TOP-T40` | `Proofs.Ai.Topology.SimplicialComplex`, `Proofs.Ai.Topology.CWComplex.Basic`, `Proofs.Ai.Topology.CWComplex.Cellular` | foundation, long-term interface |
| `TOP-21` | `topological_manifolds` | Topological manifolds | `L1` then `L2` | `TOP-T41` through `TOP-T42` | `Proofs.Ai.Topology.Manifold.Topological`, `Proofs.Ai.Topology.Manifold.Invariance` | foundation, long-term interface |
| `TOP-22` | `differential_topology` | Differential topology | `L1 Evidence package` first | `TOP-T43` through `TOP-T44` | `Proofs.Ai.Topology.Manifold.Smooth`, `Proofs.Ai.Topology.Differential.Sard`, `Proofs.Ai.Topology.Differential.Transversality`, `Proofs.Ai.Topology.Morse` | long-term interface |
| `TOP-23` | `surfaces_and_low_dimensional_topology` | Surfaces and low-dimensional topology | `L1 Evidence package` | `TOP-T45` through `TOP-T46` | `Proofs.Ai.Topology.Surface.Classification`, `Proofs.Ai.Topology.ThreeManifold.Interfaces`, `Proofs.Ai.Topology.Knot.Basic` | long-term interface |
| `TOP-24` | `fixed_point_and_degree_theorems` | Fixed point and degree theorems | `L2` where prerequisites exist | `TOP-T47` through `TOP-T48` | `Proofs.Ai.Topology.FixedPoint.Brouwer`, `Proofs.Ai.Topology.FixedPoint.BorsukUlam`, `Proofs.Ai.Topology.FixedPoint.Lefschetz` | derived theorem, long-term interface |
| `TOP-25` | `dimension_theory` | Dimension theory | `L1` then `L2` | `TOP-T49` through `TOP-T50` | `Proofs.Ai.Topology.Dimension.Covering`, `Proofs.Ai.Topology.Dimension.Inductive`, `Proofs.Ai.Topology.Dimension.Invariance` | foundation, long-term interface |
| `TOP-26` | `topological_dynamics` | Topological dynamics | `L1` then `L2` | `TOP-T51` through `TOP-T52` | `Proofs.Ai.Topology.Dynamics.Basic`, `Proofs.Ai.Topology.Dynamics.Symbolic`, `Proofs.Ai.Topology.Dynamics.Stability` | specialization, long-term interface |
| `TOP-27` | `geometric_topology_and_characteristic_classes` | Geometric topology and characteristic classes | `L1 Evidence package` | `TOP-T53` through `TOP-T54` | `Proofs.Ai.Topology.DifferentialForms.Stokes`, `Proofs.Ai.Topology.DeRham`, `Proofs.Ai.Topology.CharacteristicClass`, `Proofs.Ai.Topology.IndexTheory.Interfaces` | long-term interface |
| `TOP-28` | `k_theory_spectral_sequences_and_stable_homotopy` | K-theory, spectral sequences, and stable homotopy | `L1 Evidence package` | `TOP-T55` through `TOP-T56` | `Proofs.Ai.Topology.KTheory.Basic`, `Proofs.Ai.Topology.SpectralSequence.Basic`, `Proofs.Ai.Topology.StableHomotopy.Interfaces` | long-term interface |
| `TOP-29` | `topology_public_closure_and_promotion` | Packaging and promotion | `L3 Public closure` | `TOP-T57` | future `Mathlib.Topology.*` closure batch | package alias, promotion |

## Evidence And Dependency Map

| Card | Evidence | Dependencies | Gate |
| --- | --- | --- | --- |
| `TOP-00` | roadmap review, theorem cards, duplicate-home map, target levels; no source, replay, theorem index, or todo evidence | roadmap only | `git diff --check` |
| `TOP-01` | open-set and closed-set law packages, neighborhoods, closure/interior/boundary, dense-set evidence | existing metric bridge for `ANA-T22` | source-free module verify |
| `TOP-02` | basis, subbasis, generated topology, subspace, initial/final topology universal-property evidence | `TOP-01`; later products and quotients | source-free module verify |
| `TOP-03` | open/closed preimage, neighborhood, closure, identity, composition, restriction, pasting, map-class evidence | `TOP-01`, `TOP-02`; product and quotient hooks | source-free module verify |
| `TOP-04` | homeomorphism inverse evidence and imported property-preservation evidence | `TOP-03`; compactness, connectedness, separation, homotopy, homology, dimension owners | source-free module verify or interface audit |
| `TOP-05` | exact separation axiom level, normality, Urysohn/Tietze codomain, compactification assumptions | `TOP-03`, `TOP-06`, `TOP-14` for Stone-Cech | source-free module verify or interface audit |
| `TOP-06` | open-cover compactness, finite-intersection property, finite subcover evidence, explicit choice where used | `TOP-03`, `TOP-05`, `TOP-11`, `TOP-14` | source-free module verify or interface audit |
| `TOP-07` | metric compactness, completeness, total boundedness, sequence compactness, equicontinuity evidence | `ANA-T05`, `ANA-T12`, `ANA-T22`, `ANA-T23`, `TOP-06`, `TOP-10` | source-free module verify or interface audit |
| `TOP-08` | separation by clopen sets, components, paths, local path-connectedness, continuum assumptions | `TOP-03`, `TOP-11`, `TOP-15`, analysis interval foundations | source-free module verify or interface audit |
| `TOP-09` | countability bases, dense sets, Lindelof covers, metrizability hypotheses, example-space constructions | `TOP-05`, `TOP-07`; example-space interfaces | source-free module verify or interface audit |
| `TOP-10` | Cauchy sequence, completion, complete-metric evidence, Baire category countable-intersection evidence | `TOP-07`, `ANA-T23`; functional analysis imports through `ANA-T27` | source-free module verify or interface audit |
| `TOP-11` | product topology, projections, universal property, product preservation, product compactness choice evidence | `TOP-02`, `TOP-03`, `TOP-05`, `TOP-06`, `TOP-08`, `TOP-09` | source-free module verify or interface audit |
| `TOP-12` | quotient topology, final topology, quotient-map criteria, gluing, model-space construction evidence | `TOP-02`, `TOP-03`, `TOP-05`, `TOP-06`, `TOP-08`, `TOP-20` | source-free module verify or interface audit |
| `TOP-13` | local property witnesses, locally finite refinements, paracompactness, partition-of-unity assumptions | `TOP-05`, `TOP-06`, `TOP-07`, `TOP-08`, `TOP-09`, `TOP-11` | source-free module verify or interface audit |
| `TOP-14` | directed-set, net, filter, ultrafilter, subnet, convergence, and ultrafilter lemma evidence | `TOP-03`, `TOP-05`, `TOP-06`, `TOP-09`, `TOP-11` | source-free module verify or interface audit |
| `TOP-15` | homotopy, contractibility, retract, homotopy-equivalence, extension, and lifting-property evidence | `TOP-03`, `TOP-08`, `TOP-11`, `TOP-12` | source-free module verify or interface audit |
| `TOP-16` | based loops, path homotopy, group structure, functoriality, van Kampen cover hypotheses | `TOP-08`, `TOP-12`, `TOP-15`, algebra group modules | source-free module verify or interface audit |
| `TOP-17` | covering maps, local homeomorphism, lifting, basepoint, semilocal simple-connectedness evidence | `TOP-08`, `TOP-12`, `TOP-15`, `TOP-16` | source-free module verify or interface audit |
| `TOP-18` | simplex, chain group, boundary-square-zero, homology, exactness, excision, computation evidence | `TOP-12`, `TOP-15`, `TOP-16`, `TOP-17`, `TOP-20`, algebra and chain-complex infrastructure | source-free module verify or interface audit |
| `TOP-19` | cochain, coboundary, cup product, grading, coefficient, exactness, duality, operations evidence | `TOP-18`, `TOP-21`, algebra modules; `TOP-27`, `TOP-28` blockers for late interfaces | source-free module verify or interface audit |
| `TOP-20` | simplicial complex, CW weak topology, attaching map, skeleton, cellular homology evidence | `TOP-12`, `TOP-15`, `TOP-18`; category/simplicial-set interfaces | source-free module verify or interface audit |
| `TOP-21` | charts, local Euclidean property, dimension tag, Hausdorffness, second countability, invariance assumptions | `TOP-05`, `TOP-08`, `TOP-09`, `TOP-12`, `TOP-13`, `TOP-18`, `TOP-25` | source-free module verify or interface audit |
| `TOP-22` | smooth charts, tangent, smooth map, Sard, regular value, transversality, compactness evidence | `TOP-21`, `TOP-27`, analysis derivative/inverse/implicit routes, linear algebra | interface audit before derived source |
| `TOP-23` | surface classification, 3-manifold, knot, PL/smooth, group, homology, and invariant evidence | `TOP-16`, `TOP-18`, `TOP-20`, `TOP-21`, `TOP-22` | interface audit |
| `TOP-24` | degree, homology/cohomology, manifold orientation, fixed-point and order/lattice prerequisites | `TOP-10`, `TOP-16`, `TOP-18`, `TOP-19`, `TOP-21`, `TOP-22` | source-free module verify or interface audit |
| `TOP-25` | covering and inductive dimensions, dimension invariance, cohomological dimension evidence | `TOP-06`, `TOP-09`, `TOP-18`, `TOP-19`, `TOP-21` | source-free module verify or interface audit |
| `TOP-26` | orbit closures, minimal sets, symbolic dynamics, recurrence aliases, stability and ODE prerequisites | `TOP-03`, `TOP-06`, `TOP-08`, `TOP-10`, `TOP-11`, `TOP-15`, `TOP-22`, `MEA-T51`, statistics dynamics routes | source-free module verify or interface audit |
| `TOP-27` | differential-form integration, orientation, boundary, de Rham, Stokes, characteristic-class and index assumptions | `TOP-19`, `TOP-21`, `TOP-22`, `TOP-24`, `TOP-28`, `ANA-T16` through `ANA-T18`, `MEA-T19` through `MEA-T25` | interface audit |
| `TOP-28` | vector bundles, K-theory, exact couples, filtrations, convergence, spectra, stable category assumptions | `TOP-15`, `TOP-18`, `TOP-19`, `TOP-20`, `TOP-27`, category theory | interface audit |
| `TOP-29` | selected certificate-backed modules, deterministic hashes, axiom report, downstream smoke, closure audit evidence | completed stable topology closure | package and closure audit gates |

## Duplicate-Home Map

| Theorem family or alias | Primary home | Topology status | Reason |
| --- | --- | --- | --- |
| general open-cover compactness, finite-intersection compactness, compact closed subsets, compact continuous images, compact-to-Hausdorff homeomorphism | `TOP-06` through `TOP-T10` and `TOP-T11` | primary here | these are general topological statements; analysis imports or specializes them |
| Heine-Borel, Bolzano-Weierstrass, metric compactness, compact metric complete/totally bounded equivalence | `TOP-07` through `TOP-T12`, coordinated with `ANA-T05`, `ANA-T12`, and `ANA-T22` | specialization primary here once the topology bridge exists | the metric bridge must reuse `Proofs.Ai.Analysis.AbstractMetricTopology` and Euclidean/sequence evidence explicitly |
| Baire category theorem for complete metric, locally compact Hausdorff, and Polish spaces | `TOP-10` through `TOP-T19`, with initial analysis authoring alias `ANA-T23` | primary here | functional-analysis theorems import Baire rather than duplicate it |
| open mapping, closed graph, uniform boundedness, Hahn-Banach, Banach-space duality | analysis and functional-analysis route `ANA-T27` | external owner | these are Banach-space theorem families that use Baire as input |
| Urysohn lemma and Tietze extension theorem | `TOP-05` through `TOP-T09` | primary here | normal-space and separation hypotheses are topology-owned |
| Stone-Cech compactification | `TOP-05`, `TOP-06`, and `TOP-14`, with concrete blocker `TOP-T27` | split owner inside topology | normality/function prerequisites and ultrafilter prerequisites must both be explicit |
| Banach fixed point theorem | `Proofs.Ai.Analysis.AbstractFixedPoint` and analysis fixed-point route | external owner, topology alias only | contraction and completeness evidence already lives in analysis; `TOP-T18` and `TOP-T47` may import aliases |
| Brouwer, Borsuk-Ulam, Schauder, Lefschetz, and Poincare-Hopf fixed-point theorem families | `TOP-24` through `TOP-T47` and `TOP-T48` | primary here except Banach | degree, homology/cohomology, manifold, or order-theoretic prerequisites are topology-facing |
| fundamental group, van Kampen, circle fundamental group, covering subgroup correspondence | `TOP-16` and `TOP-17` | primary here | algebra group facts are imported; basepoint and path hypotheses are topology-owned |
| homology, exact sequences, excision, Mayer-Vietoris, Hurewicz, homology computations | `TOP-18` | primary here | chain, coefficient, exactness, and functorial evidence are topology-owned with algebra imports |
| singular cohomology, cup product, universal coefficient, Poincare duality, Cech/de Rham interfaces | `TOP-19`, with de Rham blocker `TOP-T53` | primary here for cohomology; late interfaces point to `TOP-27` | cohomology definitions and duality interfaces are topology-owned, but differential-form evidence is not assumed early |
| Stokes, de Rham theorem, Gauss-Bonnet, Chern-Weil, Hodge, index-theorem interfaces | `TOP-27` through `TOP-T53` and `TOP-T54` | primary topology interface | these require analysis integration, measure integration, smooth manifold, orientation, and bundle prerequisites |
| Poincare duality | `TOP-19` through `TOP-T38`, with homology prerequisites from `TOP-T36` | primary cohomology interface | duality must not be used before homology, cohomology, manifold, and coefficient assumptions exist |
| Poincare conjecture and low-dimensional Poincare-family topology statements | `TOP-23` through `TOP-T46` | long-term interface | not a foundational axiom; waits for manifold, PL/smooth, and invariant machinery |
| smooth Poincare, surgery, h-cobordism, s-cobordism | `TOP-22` through `TOP-T44` and `TOP-23` where low-dimensional | long-term interface | smooth and PL prerequisites must be separated before source work |
| Poincare-Hopf theorem | `TOP-24` through `TOP-T48`, with characteristic-class alias from `TOP-T54` | primary fixed-point/degree route | vector-field, index, characteristic-class, and manifold orientation evidence must be explicit |
| Poincare recurrence | measure roadmap `MEA-T51` or probability/statistics dynamics routes | external owner, topology dynamics alias only | recurrence is measure/probability evidence, not purely topological |
| Poincare inequality and Sobolev/PDE compactness names | analysis/Sobolev/PDE route | external owner | analytic norms, derivatives, weak compactness, and PDE assumptions are not topology foundations |

## Analysis Alias Contract

| Analysis alias | Topology-owned card | Contract |
| --- | --- | --- |
| `ANA-T22` | `TOP-01`, `TOP-06`, and `TOP-07` | analysis may start the `Proofs.Ai.Topology.Basic` and metric-compactness bridge, but primary theorem cards live here once topology work begins |
| `ANA-T23` | `TOP-07` and `TOP-10` | Baire and function-space topology cards are topology-owned; analysis imports them for Arzela-Ascoli and function-space work |
| `ANA-T27` | `TOP-10` input only | Baire is imported from topology; open mapping, closed graph, uniform boundedness, Hahn-Banach, and Banach-space theorem families remain analysis-owned |

These aliases do not turn source files, replay files, generated theorem indexes,
or this todo into proof evidence. They only decide where future
certificate-backed theorem names should live.

## Milestone-To-Card Checklist

| Milestone | Card present | Primary home unique | Set-theoretic evidence explicit | Sidecar trust boundary clear |
| --- | --- | --- | --- | --- |
| `TOP-00` | yes | yes | yes | yes |
| `TOP-01` | yes | yes | yes | yes |
| `TOP-02` | yes | yes | yes | yes |
| `TOP-03` | yes | yes | yes | yes |
| `TOP-04` | yes | yes | yes | yes |
| `TOP-05` | yes | yes | yes | yes |
| `TOP-06` | yes | yes | yes | yes |
| `TOP-07` | yes | yes | yes | yes |
| `TOP-08` | yes | yes | yes | yes |
| `TOP-09` | yes | yes | yes | yes |
| `TOP-10` | yes | yes | yes | yes |
| `TOP-11` | yes | yes | yes | yes |
| `TOP-12` | yes | yes | yes | yes |
| `TOP-13` | yes | yes | yes | yes |
| `TOP-14` | yes | yes | yes | yes |
| `TOP-15` | yes | yes | yes | yes |
| `TOP-16` | yes | yes | yes | yes |
| `TOP-17` | yes | yes | yes | yes |
| `TOP-18` | yes | yes | yes | yes |
| `TOP-19` | yes | yes | yes | yes |
| `TOP-20` | yes | yes | yes | yes |
| `TOP-21` | yes | yes | yes | yes |
| `TOP-22` | yes | yes | yes | yes |
| `TOP-23` | yes | yes | yes | yes |
| `TOP-24` | yes | yes | yes | yes |
| `TOP-25` | yes | yes | yes | yes |
| `TOP-26` | yes | yes | yes | yes |
| `TOP-27` | yes | yes | yes | yes |
| `TOP-28` | yes | yes | yes | yes |
| `TOP-29` | yes | yes | yes | yes |

## Acceptance Status

`TOP-T00` is complete when this file is present, cited from `proofs/README.md`,
and the roadmap/todo verification searches find the required terms. This file
does not prove any mathematical theorem and does not create a certificate. Later
milestones must replace `L0` and `L1` planning surfaces with certificate-backed
source modules before claiming `L2` or `L3` status.
