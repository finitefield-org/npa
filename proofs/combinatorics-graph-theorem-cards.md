# Combinatorics And Graph Theory Theorem Cards

Source roadmap: `proofs/combinatorics-graph-theorem-proof-roadmap.md`

This file is the `CG-T00` theorem-card inventory for the combinatorics and
graph-theory proof roadmap. It is a planning sidecar only. It does not add
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
| Level | Initial target level from the roadmap: `L0 Statement`, `L1 Evidence package`, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary home | Namespace or roadmap that owns the first formalization. |
| Finite status | Whether the card is finite, infinite, mixed, or external-owner. |
| Evidence | Main evidence or assumptions that must be explicit in statements. |
| Dependencies | Roadmap or module families that this card imports or waits for. |
| Gate | First acceptance gate for the card. |

Large structural theorem families are never marked `L2` until their prerequisite
definitions and intermediate lemmas are certificate-backed. Infinite Ramsey,
infinite cardinal, choice, ultrafilter, stationary-set, and large-cardinal
partition statements are not grouped with finite combinatorics cards; they are
set-theory-owned aliases or external-owner cards here.

## Namespace Contract

Primary concrete entry points:

```text
Proofs.Ai.Combinatorics.Inventory
Proofs.Ai.Graph.Inventory
```

These inventory modules are future certificate-backed policy entry points, not
mathematical foundations by themselves. Their role is to preserve:

| Contract item | Evidence it should preserve |
| --- | --- |
| `combinatorial_object_structure_policy` | finite sets, graphs, hypergraphs, matroids, designs, and random models are ordinary proof-corpus structures |
| `finite_infinite_boundary_policy` | finite combinatorics is separate from set-theoretic infinite combinatorics and partition calculus |
| `external_owner_alias_policy` | linear algebra, topology, set theory, number theory, probability, and optimization owners are imported rather than duplicated |
| `algorithm_trace_policy` | algorithm correctness uses explicit traces or certificates, not trusted execution |
| `derived_target_certificate_policy` | derived theorem targets require source-free certificate-verdict evidence |

Namespace ownership rules:

- Combinatorics-owned modules live under `Proofs.Ai.Combinatorics.*` when the
  roadmap owns finite counting, set-system, hypergraph, matroid, design, or
  enumerative theorem statements.
- Graph-owned modules live under `Proofs.Ai.Graph.*` when the roadmap owns graph
  data, graph-specific constructions, and combinatorial graph conclusions.
- `Proofs.Ai.LinearAlgebra.*` owns matrix, rank, determinant, eigenvalue, and
  spectral proof facts. Graph spectral modules here specialize those facts to
  explicit adjacency, incidence, and Laplacian constructions.
- `Proofs.Ai.SetTheory.*` owns infinite cardinal arithmetic, choice,
  ultrafilters, infinite Ramsey, Erdos-Rado, partition calculus, stationary-set,
  and large-cardinal assumptions.
- `Proofs.Ai.NumberTheory.*` owns arithmetic progressions in integers, additive
  number theory, sieve, circle method, finite-field arithmetic applications, and
  arithmetic theorem families.
- Probability and random-process facts are imported from
  `Proofs.Ai.Probability.*`, `Proofs.Ai.Statistics.*`, or measure-roadmap
  modules once those foundations exist.
- Algorithm modules may state and verify mathematical trace correctness, but
  executable algorithm runs are never trusted proof evidence.

## Primary Roadmap Cards

| Card | Stable id | Level | Primary home | Finite status | Evidence | Dependencies | Gate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `CG-00` | `combinatorics_graph_inventory_policy` | `L0 Statement` | `Proofs.Ai.Combinatorics.Inventory`, `Proofs.Ai.Graph.Inventory` | mixed policy | theorem-card inventory, duplicate-home map, target levels | roadmap only | `git diff --check` |
| `CG-01` | `finite_cardinality_foundation` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Finite`, `Proofs.Ai.Combinatorics.Cardinality` | finite | finite enumeration, injection, surjection, bijection, cardinality evidence | logic, equality, Nat, future set-theory finite bridge | source-free module verify |
| `CG-02` | `elementary_counting_rules` | `L2 Derived certificate` | `Proofs.Ai.Combinatorics.Counting.Basic` | finite | disjointness, finite products, finite comparison, pigeonhole evidence | `CG-01` | source-free module verify |
| `CG-03` | `binomial_permutation_multinomial_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Permutation`, `Proofs.Ai.Combinatorics.Binomial` | finite | factorial, permutation, `k`-subset, binomial, multinomial arithmetic evidence | `CG-01`, `CG-02`, algebra arithmetic carriers | source-free module verify |
| `CG-04` | `finite_inclusion_exclusion_set_systems` | `L2 Derived certificate` | `Proofs.Ai.Combinatorics.InclusionExclusion`, `Proofs.Ai.Combinatorics.SetSystem` | finite | finite family, intersection indexing, alternating-sum, covering evidence | `CG-02`, `CG-03` | source-free module verify |
| `CG-05` | `recurrence_generating_function_interfaces` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Recurrence`, `Proofs.Ai.Combinatorics.GeneratingFunction` | mixed finite/formal | initial conditions, recurrence solution evidence, formal coefficient extraction | `CG-03`, algebra/ring/polynomial route | source-free module verify or interface audit |
| `CG-06` | `graph_foundation_basic` | `L1 Evidence package` | `Proofs.Ai.Graph.Basic`, `Proofs.Ai.Graph.Directed`, `Proofs.Ai.Graph.Incidence` | finite first | vertex carrier, edge predicate, symmetry, loop policy, incidence and degree evidence | `CG-01`, `CG-02` | source-free module verify |
| `CG-07` | `graph_walk_connected_tree_route` | `L2 Derived certificate` | `Proofs.Ai.Graph.Walk`, `Proofs.Ai.Graph.Connected`, `Proofs.Ai.Graph.Tree` | finite first | walk/path evidence, connectedness, acyclicity, unique-path and edge-count hypotheses | `CG-06` | source-free module verify |
| `CG-08` | `bipartite_matching_hall_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Bipartite`, `Proofs.Ai.Graph.Matching` | finite | bipartition, matching witness, alternating path, Hall-condition evidence | `CG-06`, `CG-07` | source-free module verify or interface audit |
| `CG-09` | `graph_flow_cut_konig_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Flow`, `Proofs.Ai.Graph.Cut`, `Proofs.Ai.Graph.Konig` | finite | capacity/order laws, feasible-flow evidence, cut evidence, augmenting-path trace | `CG-08`, ordered algebra, linear-algebra/order prerequisites | source-free module verify or interface audit |
| `CG-10` | `graph_coloring_clique_independence` | `L2 Derived certificate` | `Proofs.Ai.Graph.Coloring`, `Proofs.Ai.Graph.Clique` | finite | finite color set, proper coloring, clique, independent set, complement graph evidence | `CG-06`, `CG-07` | source-free module verify |
| `CG-11` | `planar_graph_embedding_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Planar`, `Proofs.Ai.Graph.Embedding` | finite graph with topological evidence | embedding, face incidence, Euler-characteristic evidence | `CG-06`, `CG-07`, topology route | interface audit until topology dependencies exist |
| `CG-12` | `extremal_graph_theory_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Extremal` | finite/asymptotic split | forbidden-subgraph predicate, witness graph construction, extremal-number evidence | `CG-06`, `CG-10` | interface audit before derived bounds |
| `CG-13` | `finite_ramsey_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Ramsey`, `Proofs.Ai.Graph.Ramsey` | finite | finite coloring, finite color-count, finite subset or graph edge-color evidence | `CG-10`, finite set/coloring foundations; set-theory route for infinite aliases | interface audit before derived Ramsey proof |
| `CG-14` | `probabilistic_random_graph_limit_pseudorandom_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.ProbabilisticMethod`, `Proofs.Ai.Graph.Random`, `Proofs.Ai.Graph.Limit`, `Proofs.Ai.Graph.Pseudorandom` | mixed finite/asymptotic | probability-space evidence, random model, concentration, graph-process, graph-limit, pseudorandomness assumptions | `CG-06`, statistics/probability, measure, topology where needed | interface audit until probability dependencies verify |
| `CG-15` | `enumerative_combinatorics_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Enumerative`, `Proofs.Ai.Combinatorics.Partition`, `Proofs.Ai.Combinatorics.Catalan` | finite/formal | bijection evidence, recurrence evidence, partition and Catalan family surfaces | `CG-03`, `CG-05`; number theory for partition identities | source-free module verify or interface audit |
| `CG-16` | `algebraic_combinatorics_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Algebraic`, `Proofs.Ai.Combinatorics.SymmetricFunction`, `Proofs.Ai.Combinatorics.AssociationScheme` | finite/algebraic | group action, orbit/stabilizer, Burnside/Polya evidence, representation assumptions | `CG-05`, algebra, linear algebra, future representation route | interface audit |
| `CG-17` | `matroid_foundation_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Matroid.Basic`, `Proofs.Ai.Combinatorics.Matroid.Dual`, `Proofs.Ai.Combinatorics.Matroid.Graphic` | finite first | independent-set, basis, rank, circuit, closure, duality, graphic-matroid evidence | `CG-01`, `CG-07`, linear algebra for representable matroids | source-free module verify or interface audit |
| `CG-18` | `design_finite_geometry_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Design`, `Proofs.Ai.Combinatorics.FiniteGeometry` | finite | incidence structure, block design parameters, projective/affine plane construction evidence | `CG-01`, `CG-04`, `CG-16`, finite-field route | interface audit |
| `CG-19` | `hypergraph_set_system_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Hypergraph`, `Proofs.Ai.Combinatorics.Container` | finite/asymptotic split | hyperedge, uniformity, matching, covering, transversal, container evidence | `CG-04`; advanced tasks import `CG-12`, `CG-13`, `CG-14` | source-free module verify or interface audit |
| `CG-20` | `spectral_graph_specialization_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Spectral`, `Proofs.Ai.Graph.Laplacian` | finite first | adjacency/incidence/Laplacian construction evidence, matrix bridge evidence | `CG-06`, `CG-07`, linear algebra `LIN-26` | interface audit until linear algebra dependencies verify |
| `CG-21` | `graph_algorithm_trace_correctness_route` | `L1 Evidence package` | `Proofs.Ai.Graph.Algorithm.Search`, `Proofs.Ai.Graph.Algorithm.ShortestPath`, `Proofs.Ai.Graph.Algorithm.SpanningTree`, `Proofs.Ai.Graph.Algorithm.Flow` | finite | trace or certificate object, invariant, termination, optimality evidence | `CG-06`, `CG-07`, `CG-08`, `CG-09` | source-free module verify or trace-interface audit |
| `CG-22` | `combinatorial_optimization_route` | `L1 Evidence package` | `Proofs.Ai.Combinatorics.Optimization`, `Proofs.Ai.Graph.Polytope` | finite/optimization | submodular function, polytope, LP duality import, matroid/flow/matching evidence | `CG-08`, `CG-09`, `CG-17`, optimization routes | interface audit |
| `CG-23` | `combinatorics_graph_public_closure` | `L3 Public closure` | future `Mathlib.Combinatorics.*` and `Mathlib.Graph.*` closure batch | closure policy | selected certificate-backed modules, hashes, axiom report, downstream smoke | verified foundation slice | package and closure audit gates |

## Duplicate-Home Map

| Theorem family or alias | Primary home | Combinatorics/graph status | Reason |
| --- | --- | --- | --- |
| additive combinatorics over integers, arithmetic progressions, Schur/Rado as arithmetic statements | `proofs/number-theory-theorem-proof-roadmap.md` `NT-10` and `NT-12` | duplicate alias here | arithmetic structure and number-theoretic hypotheses are primary there |
| finite pigeonhole, finite Ramsey, finite Erdos-Szekeres style forms | this roadmap `CG-02` and `CG-13` | primary here | finite combinatorial forms should not wait for set-theoretic infinite combinatorics |
| infinite Ramsey, Erdos-Rado, partition calculus, ultrafilter combinatorics | `proofs/set-theory-theorem-proof-roadmap.md` `SET-20` | external owner | infinite and metatheoretic assumptions belong to set theory |
| graph Laplacian matrix facts, eigenvalue theorems, matrix-tree, Cheeger spectral proof facts, Perron-Frobenius | `proofs/linear-algebra-theorem-proof-roadmap.md` `LIN-21` and `LIN-26` | specialization consumer here | graph modules provide graph data and construction evidence, not duplicate matrix theory |
| random graph probability facts, concentration, martingale and process tools | `proofs/statistics-theorem-proof-roadmap.md` and `proofs/measure-theory-theorem-proof-roadmap.md` | imported prerequisites here | probability primitives and concentration inequalities are external foundations |
| graph limits and graphons | this roadmap with topology/measure dependencies | primary graph statement here | graph objects and limit statements are graph-owned, but analytic dependencies remain explicit |
| graph minors and surface embeddings | this roadmap plus `proofs/topology-theorem-proof-roadmap.md` | split owner | topology owns embedding/surface machinery; graph roadmap owns graph data and combinatorial conclusions |
| graph algorithms, BFS/DFS, shortest paths, matching, flow, spanning tree correctness | this roadmap unless a future algorithms roadmap supersedes it | primary here for trace correctness | no separate algorithms roadmap currently exists; executable runs are untrusted |
| general LP duality, convex duality, KKT, semidefinite optimization | analysis/linear-algebra optimization route | imported prerequisite here | graph polytopes and matching/flow are specializations |
| finite-field construction of projective planes and finite geometries | field-theory/number-theory route plus `CG-18` | split owner | finite fields are external algebra/arithmetic structures; incidence/design conclusions are combinatorial |
| Polya counting and Burnside lemma | this roadmap with algebra imports | primary here after group-action bridge | group laws are algebra-owned; enumeration statements are combinatorics-owned |

## Milestone-To-Card Checklist

| Milestone | Card present | Primary home unique | Infinite boundary clear | Sidecar trust boundary clear |
| --- | --- | --- | --- | --- |
| `CG-00` | yes | yes | yes | yes |
| `CG-01` | yes | yes | yes | yes |
| `CG-02` | yes | yes | yes | yes |
| `CG-03` | yes | yes | yes | yes |
| `CG-04` | yes | yes | yes | yes |
| `CG-05` | yes | yes | yes | yes |
| `CG-06` | yes | yes | yes | yes |
| `CG-07` | yes | yes | yes | yes |
| `CG-08` | yes | yes | yes | yes |
| `CG-09` | yes | yes | yes | yes |
| `CG-10` | yes | yes | yes | yes |
| `CG-11` | yes | yes | yes | yes |
| `CG-12` | yes | yes | yes | yes |
| `CG-13` | yes | yes | yes | yes |
| `CG-14` | yes | yes | yes | yes |
| `CG-15` | yes | yes | yes | yes |
| `CG-16` | yes | yes | yes | yes |
| `CG-17` | yes | yes | yes | yes |
| `CG-18` | yes | yes | yes | yes |
| `CG-19` | yes | yes | yes | yes |
| `CG-20` | yes | yes | yes | yes |
| `CG-21` | yes | yes | yes | yes |
| `CG-22` | yes | yes | yes | yes |
| `CG-23` | yes | yes | yes | yes |

## Acceptance Status

`CG-T00` is complete when this file is present, cited from `proofs/README.md`,
and the roadmap/todo verification searches find the required terms. This file
does not prove any mathematical theorem and does not create a certificate. Later
milestones must replace `L0` and `L1` planning surfaces with certificate-backed
source modules before claiming `L2` or `L3` status.
