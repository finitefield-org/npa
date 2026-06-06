# Combinatorics And Graph Theory Theorem Proof Roadmap

Date: 2026-06-05

This document plans how to prove combinatorics and graph-theory theorem
families one small batch at a time in the NPA proof corpus. It is a planning
sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covered by this roadmap includes:

- finite sets, finite families, cardinality, injections, surjections,
  bijections, and finite-choice interfaces;
- product and sum rules, pigeonhole principles, inclusion-exclusion,
  factorials, binomial coefficients, permutations, combinations, and
  multinomial coefficients;
- recurrences, generating functions, formal power series, partitions,
  Catalan-style families, Polya-style counting interfaces, and combinatorial
  species interfaces;
- simple graphs, multigraphs, directed graphs, hypergraphs, incidence
  structures, degree formulas, walks, paths, cycles, connectedness, trees,
  forests, and cuts;
- bipartite graphs, matchings, Hall theorem, Konig-style theorem families,
  network flows, max-flow/min-cut, and matching/flow algorithm correctness;
- coloring, chromatic number, clique and independent-set bounds, perfect graph
  interfaces, planar graphs, Euler formula, graph minors, and topological graph
  interfaces;
- extremal graph theory, Turan-style theorem families, Ramsey theory, finite
  and hypergraph Ramsey statements, and finite combinatorial set systems;
- probabilistic method, random graphs, concentration-dependent graph theorems,
  graph limits, expansion, and pseudorandom graph interfaces;
- algebraic combinatorics, symmetric-function and representation-theoretic
  interfaces, matroids, designs, finite geometries, spectral graph theory,
  graph algorithms, and combinatorial optimization.

The first priority is not to encode every named theorem immediately. The first
priority is to build finite-set, counting, and graph foundations whose
statements will not need to be replaced after matchings, Ramsey theory,
probabilistic combinatorics, spectral graph theory, or graph algorithms depend
on them.

## Existing Baseline

The current proof corpus exposes a checked `Proofs.Ai.Combinatorics.*`
foundation slice through finite families, cardinality, counting, permutation,
binomial, inclusion-exclusion, and set-system modules. It does not yet expose a
checked `Proofs.Ai.Graph.*` tree. The route also provides reusable foundation
modules and neighboring roadmaps that later graph and advanced combinatorics
tasks must import rather than duplicate.

| Corpus module or roadmap | Existing role |
| --- | --- |
| `Proofs.Ai.Basic`, `Proofs.Ai.Prop`, `Proofs.Ai.Logic.Iff` | proposition-level reasoning, implication, connectives, and small search targets |
| `Proofs.Ai.Eq`, `Proofs.Ai.EqReasoning` | equality and equality-transport basics |
| `Proofs.Ai.Nat` | natural-number smoke layer and `Std.Nat.Basic` bridge |
| `Proofs.Ai.Algebra.AbstractGroup*` | group, quotient, correspondence, and isomorphism theorem targets useful for symmetry and group-action interfaces |
| `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractField`, `Proofs.Ai.Algebra.AbstractOrderedField` | scalar, polynomial, and ordered law packages needed by enumeration, generating functions, and spectral graph theory |
| `Proofs.Ai.Vector.*`, `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` | vector, matrix, inner-product, and spectral prerequisites for graph adjacency and Laplacian specializations |
| `Proofs.Ai.Geometry.*`, `Proofs.Ai.Topology.*` future route | planar embedding, topological graph, and geometric graph consumers |
| `proofs/set-theory-theorem-proof-roadmap.md` | set, relation, function, finite/infinite cardinal, choice, ultrafilter, and partition-calculus interfaces |
| `proofs/number-theory-theorem-proof-roadmap.md` | additive and arithmetic combinatorics consumers; finite-field and polynomial-method dependencies |
| `proofs/statistics-theorem-proof-roadmap.md` | probability, concentration, martingale, and random-process prerequisites for probabilistic combinatorics |
| `proofs/linear-algebra-theorem-proof-roadmap.md` | matrix, rank, determinant, eigenvalue, Perron-Frobenius, and graph-linear-algebra prerequisites |

Combinatorics and graph work should begin with explicit finite law packages and
small derived certificates. Large theorem families such as the graph minor
theorem, Szemeredi regularity, Erdos-Stone, probabilistic thresholds, perfect
graph theorem, and graph limit representation theorems may first land as
interfaces, but those interfaces must keep construction evidence explicit and
must not be counted as fully derived results.

## Proof Levels

Each theorem should be labeled with one of these levels while it moves through
the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant, theorem card, or dependency-tagged theorem shape only | no |
| `L1 Evidence package` | conclusion follows from explicit finite data, construction evidence, algorithm trace, graph embedding, random model, or law package | only if explicitly marked as an interface milestone |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

For combinatorics and graph theory, `L1` interfaces are often useful for finite
enumeration witnesses, selected representatives, graph embeddings, matching
witnesses, flows, algorithm traces, random-model evidence, minor models, and
regularity partitions. Such interfaces must not be confused with derived
theorems. A theorem is mathematically complete only at `L2` or `L3`, unless the
scope explicitly says that the immediate target is an interface wrapper.

## One-Theorem Work Unit

For each theorem family, use this work unit:

1. Freeze the statement in the smallest suitable `Proofs.Ai.*` module.
2. Classify the target as `L0`, `L1`, `L2`, or `L3`.
3. Audit the target for circular assumptions. The theorem conclusion itself
   must not appear as an input under another name.
4. Keep imports minimal and prefer existing corpus modules.
5. Add or update checked source, replay, metadata, and certificate.
6. Verify the target module source-free.
7. Verify changed proof-corpus artifacts.
8. At the end of a coherent batch, run the authoring gate.

Default proof-corpus commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `Proofs.Ai.Graph.X` for graph-owned theorem families. Run
`./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh` only for
package-wide compatibility, promotion, release readiness, or changes to
certificate encoding, checker behavior, package verification, or kernel
semantics.

## Statement Policy

- Finite sets, families, graphs, edges, walks, trees, colorings, matchings,
  flows, hypergraphs, matroids, designs, and random graph models are ordinary
  library structures or explicit law packages. They are not kernel primitives.
- Cardinality and finite-enumeration claims must name the finite witness,
  equivalence, injection, surjection, bijection, or counting law they use.
- Algorithms such as BFS, DFS, shortest paths, union-find, augmenting paths,
  max-flow, and matching algorithms are represented by mathematical traces and
  correctness theorems. Runtime behavior is not trusted evidence.
- Randomized constructions and probabilistic-method proofs must import
  probability and concentration facts from the statistics/probability or
  measure routes. Randomness is never a checker primitive.
- Quotients, graph isomorphism classes, unlabeled counting, orbit counting, and
  Polya-style results must expose quotient or group-action evidence explicitly.
- Infinite Ramsey, ultrafilter, large-cardinal, and partition-calculus
  statements are owned by the set-theory route unless this roadmap is only
  specializing a finite combinatorial form.
- Large theorem interfaces may be useful, but bridge assumptions must be named,
  localized, and rejected by final high-trust policy when a theorem is claimed
  as derived.

## Duplicate Theorem Policy

- Finite counting and graph-theoretic statements are primary here.
- Infinite cardinal arithmetic, choice principles, ultrafilters, stationary
  sets, and large-cardinal partition calculus are primary in the set-theory
  roadmap.
- Additive number theory, arithmetic progressions in integers, sieve method,
  and circle-method theorem families are primary in the number-theory roadmap.
  This roadmap owns only the reusable finite combinatorial lemmas they import.
- Matrix spectral theorems, Perron-Frobenius, rank, determinant, and numerical
  linear algebra are primary in the linear-algebra roadmap. Graph spectral
  statements here should import those results and specialize them to explicit
  graph adjacency, incidence, and Laplacian constructions.
- Probability spaces, concentration inequalities, martingales, empirical
  processes, and random variables are primary in the statistics/probability and
  measure roadmaps. Random-graph theorem statements here should import those
  foundations.
- Topological graph theory, graph embeddings on surfaces, and graph-minor
  topology interfaces may import the topology roadmap. This roadmap owns the
  graph data and combinatorial conclusion.
- General optimization duality belongs to the analysis/linear-algebra
  optimization route. Matching, flow, cut, and graph polytope specializations
  are graph/combinatorics consumers unless a future optimization roadmap
  supersedes them.

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `CG-00` | inventory and statement policy | theorem cards, duplicate-home map, and target-level tags |
| `CG-01` | finite sets and cardinality | finite-family, injection, surjection, bijection, and cardinality APIs |
| `CG-02` | elementary counting rules | sum rule, product rule, pigeonhole, and finite disjoint-union counting |
| `CG-03` | factorials, binomial coefficients, and multinomials | permutation/combination counting route |
| `CG-04` | inclusion-exclusion and finite set systems | finite union counting and Bonferroni-style combinatorial bounds |
| `CG-05` | recurrences and generating functions | recurrence solution and formal generating-function interfaces |
| `CG-06` | graph foundations | simple graph, directed graph, adjacency, incidence, degree, and handshaking APIs |
| `CG-07` | walks, paths, cycles, connectedness, and trees | path composition, connected components, tree characterizations |
| `CG-08` | bipartite graphs, matchings, and Hall route | matching law package and Hall theorem interface |
| `CG-09` | flows, cuts, and Konig-style theorem families | max-flow/min-cut and bipartite matching bridge interfaces |
| `CG-10` | graph coloring and clique/independence bounds | chromatic number and greedy/coloring theorem route |
| `CG-11` | planarity and graph embeddings | Euler formula and planar graph interface |
| `CG-12` | extremal graph theory | Turan theorem route and forbidden-subgraph interfaces |
| `CG-13` | finite Ramsey theory | graph and hypergraph Ramsey theorem interfaces |
| `CG-14` | probabilistic method, random graphs, graph limits, and pseudorandomness | random graph, graph-process, graph-limit, and first-moment style targets |
| `CG-15` | enumerative combinatorics | partitions, Catalan objects, species, and Polya-style counting surfaces |
| `CG-16` | algebraic combinatorics | symmetric-function, association-scheme, and representation interfaces |
| `CG-17` | matroids | independent sets, rank, circuits, closure, duality, and graphic matroid route |
| `CG-18` | designs and finite geometries | block design, incidence, projective-plane, and finite-geometry interfaces |
| `CG-19` | hypergraphs and set systems | hypergraph matching, covering, container, and extremal interfaces |
| `CG-20` | spectral graph theory | adjacency/Laplacian specialization of linear-algebra spectral routes |
| `CG-21` | graph algorithms and correctness | BFS/DFS, shortest-path, spanning-tree, matching, and flow trace correctness |
| `CG-22` | combinatorial optimization | matching polytopes, cut polytopes, submodularity, and matroid optimization surfaces |
| `CG-23` | packaging and promotion | stable combinatorics/graph closure audits |

## CG-00 Inventory And Statement Policy

- Status: planned.
- Depends on: none.
- Target modules:
  - `Proofs.Ai.Combinatorics.Inventory`
  - `Proofs.Ai.Graph.Inventory`
- Theorem order:
  1. classify each theorem family into exactly one primary milestone;
  2. record aliases shared with set theory, number theory, statistics,
     topology, linear algebra, algorithms, and optimization;
  3. assign each target a stable English identifier, target level, proposed
     module, dependencies, and acceptance gate;
  4. mark large theorem interfaces and conjectural or open statements
     explicitly.
- Deliverables:
  - `proofs/combinatorics-graph-theorem-cards.md` theorem-card inventory and
    duplicate-home map.
- Acceptance criteria:
  - every theorem family has one primary home;
  - no card treats source, replay, AI index, or this roadmap as proof evidence;
  - finite and infinite theorem families are separated before source work.

## CG-01 Finite Sets And Cardinality

- Status: planned.
- Depends on: `CG-00`, logic/equality/natural-number basics.
- Target modules:
  - `Proofs.Ai.Combinatorics.Finite`
  - `Proofs.Ai.Combinatorics.Cardinality`
- Theorem order:
  1. finite-family and finite-enumeration law packages;
  2. injections, surjections, bijections, and finite equivalence transport;
  3. cardinality invariance under bijection;
  4. finite subset and finite image cardinality surfaces.
- Proof strategy:
  - keep finite evidence explicit until a general set-theory finite-cardinality
    module exists;
  - avoid silently importing choice or quotient principles.
- Acceptance criteria:
  - finite cardinality is not a kernel primitive;
  - bijective transport is reusable by graph isomorphism and enumeration tasks.

## CG-02 Elementary Counting Rules

- Status: planned.
- Depends on: `CG-01`.
- Target modules:
  - `Proofs.Ai.Combinatorics.Counting.Basic`
- Theorem order:
  1. finite disjoint-union cardinality;
  2. finite product cardinality;
  3. sum rule and product rule;
  4. pigeonhole principle and injective/surjective finite comparison variants.
- Acceptance criteria:
  - disjointness and finiteness hypotheses are explicit;
  - pigeonhole theorems do not assume decidable equality unless the theorem
    statement names it.

## CG-03 Factorials, Binomial Coefficients, And Multinomials

- Status: planned.
- Depends on: `CG-01`, `CG-02`.
- Target modules:
  - `Proofs.Ai.Combinatorics.Binomial`
  - `Proofs.Ai.Combinatorics.Permutation`
- Theorem order:
  1. factorial and falling-factorial statement surface;
  2. permutation counting for finite enumerated families;
  3. combination counting and binomial coefficient theorem targets;
  4. Pascal recurrence, binomial symmetry, and Vandermonde identity interfaces;
  5. multinomial coefficient surfaces.
- Acceptance criteria:
  - permutation objects and counting theorem statements do not depend on hidden
    list normalization;
  - binomial identities specify the numeric domain used for arithmetic.

## CG-04 Inclusion-Exclusion And Finite Set Systems

- Status: planned.
- Depends on: `CG-02`, `CG-03`.
- Target modules:
  - `Proofs.Ai.Combinatorics.InclusionExclusion`
  - `Proofs.Ai.Combinatorics.SetSystem`
- Theorem order:
  1. finite union upper bounds;
  2. pairwise and finite inclusion-exclusion;
  3. Bonferroni-style combinatorial inequalities;
  4. set-system intersection and covering number interfaces.
- Acceptance criteria:
  - signs, parity, and alternating sums are represented by ordinary arithmetic
    structures;
  - probability Bonferroni aliases import statistics/probability only as
    consumers.

## CG-05 Recurrences And Generating Functions

- Status: planned.
- Depends on: `CG-03`, algebra/ring/polynomial prerequisites.
- Target modules:
  - `Proofs.Ai.Combinatorics.Recurrence`
  - `Proofs.Ai.Combinatorics.GeneratingFunction`
- Theorem order:
  1. finite sequence and recurrence statement interfaces;
  2. linear recurrence solution evidence packages;
  3. ordinary generating-function law package;
  4. exponential generating-function and species interfaces;
  5. coefficient extraction and convolution identities.
- Acceptance criteria:
  - formal power series are ordinary algebraic structures or imported algebra
    modules, not syntax-level primitives;
  - analytic convergence is not assumed for formal generating functions.

## CG-06 Graph Foundations

- Status: planned.
- Depends on: `CG-01`, `CG-02`.
- Target modules:
  - `Proofs.Ai.Graph.Basic`
  - `Proofs.Ai.Graph.Directed`
  - `Proofs.Ai.Graph.Incidence`
- Theorem order:
  1. simple graph law package over a vertex carrier and edge predicate;
  2. directed graph and multigraph statement surfaces;
  3. adjacency, incidence, degree, neighborhood, subgraph, induced subgraph, and
     graph homomorphism APIs;
  4. handshaking lemma and degree-sum theorem targets.
- Acceptance criteria:
  - graph structures are ordinary records;
  - edge symmetry, irreflexivity, multiplicity, and direction assumptions are
    explicit in the module where they are used.

## CG-07 Walks, Paths, Cycles, Connectedness, And Trees

- Status: planned.
- Depends on: `CG-06`.
- Target modules:
  - `Proofs.Ai.Graph.Walk`
  - `Proofs.Ai.Graph.Connected`
  - `Proofs.Ai.Graph.Tree`
- Theorem order:
  1. walk concatenation and reversal;
  2. path and cycle predicates;
  3. connectedness and connected-component law packages;
  4. tree/forest definitions and equivalences between connected acyclic graphs,
     unique paths, and edge-count formulas.
- Acceptance criteria:
  - path and walk definitions do not depend on an untrusted parser notation;
  - tree characterizations are split so edge-count facts can be derived after
    finite cardinality is stable.

## CG-08 Bipartite Graphs, Matchings, And Hall Route

- Status: planned.
- Depends on: `CG-06`, `CG-07`.
- Target modules:
  - `Proofs.Ai.Graph.Bipartite`
  - `Proofs.Ai.Graph.Matching`
- Theorem order:
  1. bipartition law package and induced bipartite subgraph interface;
  2. matching, perfect matching, augmenting path, and alternating path surfaces;
  3. Hall condition and Hall theorem interface;
  4. stable marriage and transversal theorem aliases when dependencies exist.
- Acceptance criteria:
  - matching witnesses are explicit;
  - Hall theorem is not assumed as a field inside a package that claims to
    derive it.

## CG-09 Flows, Cuts, And Konig-Style Theorem Families

- Status: planned.
- Depends on: `CG-08`, linear algebra/order prerequisites as needed.
- Target modules:
  - `Proofs.Ai.Graph.Flow`
  - `Proofs.Ai.Graph.Cut`
  - `Proofs.Ai.Graph.Konig`
- Theorem order:
  1. network, capacity, feasible flow, and cut law packages;
  2. residual network and augmenting path statement surface;
  3. max-flow/min-cut theorem interface;
  4. bipartite matching from flow interface;
  5. Konig theorem and vertex-cover bridge interfaces.
- Acceptance criteria:
  - algorithmic augmenting-path traces are ordinary evidence objects;
  - capacity arithmetic and order assumptions are explicit.

## CG-10 Graph Coloring And Clique/Independence Bounds

- Status: planned.
- Depends on: `CG-06`, `CG-07`, `CG-12` for some bounds.
- Target modules:
  - `Proofs.Ai.Graph.Coloring`
  - `Proofs.Ai.Graph.Clique`
- Theorem order:
  1. proper coloring, chromatic number, clique, independent set, and complement
     graph APIs;
  2. greedy coloring and maximum-degree upper-bound route;
  3. clique lower bound and independence lower-bound interfaces;
  4. Brooks, perfect graph, and list-coloring interfaces after prerequisites.
- Acceptance criteria:
  - color palettes are finite structures with explicit cardinality evidence;
  - perfect graph theorem remains an interface until the required graph-minor or
    structural proof route is available.

## CG-11 Planarity And Graph Embeddings

- Status: planned.
- Depends on: `CG-06`, `CG-07`, topology route for embedding-heavy statements.
- Target modules:
  - `Proofs.Ai.Graph.Planar`
  - `Proofs.Ai.Graph.Embedding`
- Theorem order:
  1. planar embedding law package and face/incidence statement surface;
  2. Euler formula interface;
  3. planar edge bound and nonplanarity of `K5` / `K3,3` interfaces;
  4. Kuratowski and graph-minor theorem interfaces as long-term targets.
- Acceptance criteria:
  - topological embedding evidence is explicit;
  - planar graph results do not silently import geometric or topological
    theorems without dependency tags.

## CG-12 Extremal Graph Theory

- Status: planned.
- Depends on: `CG-06`, `CG-10`.
- Target modules:
  - `Proofs.Ai.Graph.Extremal`
- Theorem order:
  1. forbidden-subgraph predicate and extremal-number statement surface;
  2. Turan graph construction interface;
  3. Turan theorem route;
  4. Erdos-Stone, supersaturation, and stability interfaces;
  5. extremal set-system interfaces shared with hypergraph tasks.
- Acceptance criteria:
  - extremal constructions carry explicit witness graphs;
  - asymptotic claims are kept separate from finite exact bounds.

## CG-13 Finite Ramsey Theory

- Status: planned.
- Depends on: `CG-10`, finite set/coloring foundations, and the set-theory
  route for infinite variants.
- Target modules:
  - `Proofs.Ai.Combinatorics.Ramsey`
  - `Proofs.Ai.Graph.Ramsey`
- Theorem order:
  1. finite coloring-of-subsets interface;
  2. finite graph Ramsey number surface;
  3. finite Ramsey theorem interface;
  4. small Ramsey bound examples;
  5. hypergraph Ramsey interface.
- Acceptance criteria:
  - infinite Ramsey and partition calculus remain primary in set theory;
  - finite Ramsey statements specify all finiteness and color-count evidence.

## CG-14 Probabilistic Method, Random Graphs, Graph Limits, And Pseudorandomness

- Status: planned.
- Depends on: `CG-06`, statistics/probability foundations, measure route where
  needed.
- Target modules:
  - `Proofs.Ai.Combinatorics.ProbabilisticMethod`
  - `Proofs.Ai.Graph.Random`
  - `Proofs.Ai.Graph.Limit`
  - `Proofs.Ai.Graph.Pseudorandom`
- Theorem order:
  1. finite random choice and expectation method interfaces;
  2. first-moment and union-bound combinatorial theorem targets;
  3. random graph model `G(n,p)` statement surface;
  4. threshold and concentration-dependent interfaces;
  5. graph process, graph limit, and pseudorandom graph interfaces;
  6. Lovasz local lemma and alteration method interfaces.
- Acceptance criteria:
  - probability primitives are imported from the probability/statistics route;
  - random graph theorem interfaces do not count as derived until probability
    dependencies are certificate-backed.

## CG-15 Enumerative Combinatorics

- Status: planned.
- Depends on: `CG-03`, `CG-05`.
- Target modules:
  - `Proofs.Ai.Combinatorics.Enumerative`
  - `Proofs.Ai.Combinatorics.Partition`
  - `Proofs.Ai.Combinatorics.Catalan`
- Theorem order:
  1. partitions, compositions, integer partitions, and Young diagram surfaces;
  2. Catalan object family and common recurrence interfaces;
  3. Stirling number and Bell number theorem surfaces;
  4. Polya counting and cycle index interfaces;
  5. species operations and generating-function bridge.
- Acceptance criteria:
  - unlabeled counting exposes quotient/group-action evidence;
  - number-theoretic partition identities are cross-linked before duplication.

## CG-16 Algebraic Combinatorics

- Status: planned.
- Depends on: `CG-05`, algebra, linear algebra, and representation interfaces.
- Target modules:
  - `Proofs.Ai.Combinatorics.Algebraic`
  - `Proofs.Ai.Combinatorics.SymmetricFunction`
  - `Proofs.Ai.Combinatorics.AssociationScheme`
- Theorem order:
  1. group action and orbit-counting interface;
  2. Burnside and Polya bridge after group-action foundations;
  3. symmetric-function statement surface;
  4. association schemes and strongly regular graph interfaces;
  5. representation-theoretic combinatorics aliases.
- Acceptance criteria:
  - group-action theorem dependencies import algebra group modules;
  - representation-theoretic statements remain interfaces until representation
    foundations exist.

## CG-17 Matroids

- Status: planned.
- Depends on: `CG-01`, `CG-07`, and linear algebra for representable matroids.
- Target modules:
  - `Proofs.Ai.Combinatorics.Matroid.Basic`
  - `Proofs.Ai.Combinatorics.Matroid.Dual`
  - `Proofs.Ai.Combinatorics.Matroid.Graphic`
- Theorem order:
  1. independent-set, basis, rank, circuit, closure, and flat law packages;
  2. basis exchange and rank submodularity;
  3. dual matroid interface;
  4. graphic and cographic matroid bridge;
  5. greedy algorithm correctness interface for later algorithm and optimization
     consumers.
- Acceptance criteria:
  - basis existence and exchange assumptions are explicit until derived;
  - linear representability imports linear algebra rather than creating a
    private vector-space API;
  - algorithm trace correctness imports `CG-21` when a matroid greedy statement
    is treated as an algorithm theorem.

## CG-18 Designs And Finite Geometries

- Status: planned.
- Depends on: `CG-01`, `CG-04`, `CG-16`, number theory/field theory for finite
  field examples.
- Target modules:
  - `Proofs.Ai.Combinatorics.Design`
  - `Proofs.Ai.Combinatorics.FiniteGeometry`
- Theorem order:
  1. incidence structure and block design law packages;
  2. balanced incomplete block design parameter identities;
  3. projective plane and affine plane interfaces;
  4. finite-field construction interfaces;
  5. coding-theory and finite-geometry aliases.
- Acceptance criteria:
  - finite-field construction evidence imports field-theory modules;
  - design parameter equations are ordinary counting theorems.

## CG-19 Hypergraphs And Set Systems

- Status: planned.
- Depends on: `CG-04`. Advanced hypergraph Ramsey, extremal, and container
  tasks also depend on the relevant `CG-12`, `CG-13`, and `CG-14` foundations.
- Target modules:
  - `Proofs.Ai.Combinatorics.Hypergraph`
  - `Proofs.Ai.Combinatorics.Container`
- Theorem order:
  1. hypergraph, uniformity, degree, matching, covering, and transversal APIs;
  2. hypergraph Ramsey interface;
  3. Erdos-Ko-Rado and extremal set-system theorem surfaces;
  4. container method interfaces;
  5. property-testing and sparse extremal theorem interfaces.
- Acceptance criteria:
  - set-system and hypergraph APIs share finite-family foundations;
  - container method interfaces keep probabilistic and entropy assumptions
    explicit.

## CG-20 Spectral Graph Theory

- Status: planned.
- Depends on: `CG-06`, `CG-07`, linear algebra spectral prerequisites.
- Target modules:
  - `Proofs.Ai.Graph.Spectral`
  - `Proofs.Ai.Graph.Laplacian`
- Theorem order:
  1. adjacency, incidence, degree, and Laplacian matrix statement surface;
  2. basic symmetry and positive-semidefinite interfaces;
  3. connected components and Laplacian kernel interface;
  4. spectral bounds for degree, expansion, and coloring;
  5. expander and Cheeger-style interfaces.
- Acceptance criteria:
  - matrix and eigenvalue facts import linear algebra modules;
  - graph-specific theorem statements keep graph construction evidence separate
    from matrix spectral evidence.

## CG-21 Graph Algorithms And Correctness

- Status: planned.
- Depends on: `CG-06`, `CG-07`, `CG-08`, and `CG-09`.
- Target modules:
  - `Proofs.Ai.Graph.Algorithm.Search`
  - `Proofs.Ai.Graph.Algorithm.ShortestPath`
  - `Proofs.Ai.Graph.Algorithm.SpanningTree`
  - `Proofs.Ai.Graph.Algorithm.Flow`
- Theorem order:
  1. BFS and DFS trace correctness interfaces;
  2. shortest path and relaxation trace correctness;
  3. minimum spanning tree and cut/cycle property routes;
  4. augmenting path matching and max-flow algorithm correctness;
  5. trace and certificate schemas reused by later graph optimization and
     matroid-greedy interfaces.
- Acceptance criteria:
  - executable implementations are not trusted;
  - each algorithm theorem takes a trace or certificate object that the kernel
    can check structurally.

## CG-22 Combinatorial Optimization

- Status: planned.
- Depends on: `CG-08`, `CG-09`, `CG-17`, linear-algebra/analysis optimization
  routes.
- Target modules:
  - `Proofs.Ai.Combinatorics.Optimization`
  - `Proofs.Ai.Graph.Polytope`
- Theorem order:
  1. submodular function and greedy-choice surfaces;
  2. matching polytope and vertex-cover dual interfaces;
  3. cut polytope and flow polytope statement surfaces;
  4. total unimodularity and integral polyhedron interfaces;
  5. matroid intersection and submodular optimization interfaces.
- Acceptance criteria:
  - linear-programming duality imports the relevant optimization route;
  - graph-specific specialization statements do not duplicate general LP or
    convex-duality theorem ownership.

## CG-23 Packaging And Promotion

- Status: planned.
- Depends on: a coherent verified foundation slice.
- Target modules:
  - stable `Proofs.Ai.Combinatorics.*` or `Proofs.Ai.Graph.*` closure batch.
- Theorem order:
  1. select the smallest public closure that is useful without dragging in
     unstable advanced interfaces;
  2. run local source-free verification;
  3. create closure audit with import rewrite table and public declaration
     inventory;
  4. materialize into `npa-mathlib` only after package gates pass.
- Acceptance criteria:
  - closure includes deterministic source/certificate/export/axiom hashes;
  - downstream smoke consumes the closure source-free from vendored certificate
    bytes.

## Recommended Initial Execution Queue

1. `CG-00`: theorem cards, duplicate-home map, and theorem-level tags.
2. `CG-01`: finite-family, bijection, injection, and cardinality statement
   freeze.
3. `CG-02`: disjoint-union/product counting and pigeonhole principle.
4. `CG-03`: factorial, permutation, combination, and binomial coefficient route.
5. `CG-04`: finite inclusion-exclusion and set-system interfaces.
6. `CG-06`: graph basic law package, degree, incidence, and handshaking lemma.
7. `CG-07`: walks, paths, connected components, and tree characterizations.
8. `CG-08`: bipartite matching and Hall theorem interface.
9. `CG-10`: coloring and clique/independence basics.
10. `CG-13`: finite Ramsey theorem interface after finite-coloring foundations.
11. `CG-20`: spectral graph statement surface once linear-algebra dependencies
    are stable.

This queue intentionally delays random graphs, probabilistic method, graph
limits, graph minors, algebraic combinatorics, and combinatorial optimization
until finite counting, graph basics, and the relevant cross-roadmap foundations
are stable.

## Cross-Roadmap Ownership

| Theorem family | Primary roadmap | Combinatorics/graph role |
| --- | --- | --- |
| finite cardinality and elementary counting | this roadmap, with set-theory imports when available | primary theorem and graph-counting base |
| infinite cardinals, choice, ultrafilters, partition calculus | set-theory roadmap | import or specialize finite fragments only |
| additive number theory, arithmetic progressions, sieve, circle method | number-theory roadmap | provide finite combinatorial lemmas and graph/set-system aliases |
| probability, concentration, martingales, empirical process tools | statistics/probability and measure roadmaps | import for probabilistic method and random graphs |
| matrix spectral theorem, Perron-Frobenius, rank, determinant | linear-algebra roadmap | specialize to graph adjacency, incidence, and Laplacian matrices |
| planar embeddings, surface topology, graph minors with topological input | topology roadmap plus this roadmap | topology owns embedding machinery; this roadmap owns graph data and combinatorial conclusions |
| group actions and orbit counting prerequisites | algebra roadmap or existing algebra corpus modules | import group-action laws; expose Burnside/Polya combinatorial surfaces |
| graph algorithms and trace correctness | this roadmap unless a future algorithms roadmap supersedes it | prove mathematical correctness from explicit traces |

## Risk Register

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| finite cardinality is formalized incompatibly with set theory | later rewrites of counting and graph APIs | keep finite evidence explicit and record future set-theory bridge points |
| graph definitions mix simple, directed, and multigraph assumptions | theorem statements become ambiguous | split graph law packages and name edge assumptions in every theorem |
| algorithms are treated as trusted computation | trusted boundary expands incorrectly | represent algorithms by proof-checkable traces and certificates |
| probabilistic method lands before probability foundations | random graph statements become circular interfaces | keep `CG-14` at `L1` until probability dependencies are verified |
| spectral graph facts duplicate linear algebra | inconsistent eigenvalue and matrix APIs | import linear algebra spectral results and specialize them in `CG-20` |
| large structural theorems land too early | roadmap becomes a catalog of unchecked axioms | keep graph minor, perfect graph, regularity, and threshold theorems as named interfaces until prerequisites exist |
| additive combinatorics duplicates number theory | import cycles and naming conflicts | keep arithmetic theorem families primary in number theory and provide finite combinatorial support here |

## Acceptance Gates

For a single theorem module:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Combinatorics.X
cargo run -p npa-proof-corpus -- --changed-only
```

For graph-owned modules, replace the module name with `Proofs.Ai.Graph.X`.

For an ordinary authoring batch:

```sh
./scripts/check-corpus-authoring.sh
```

For public package or high-trust closure work:

```sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

## Decision Points

- Decide the first finite-cardinality representation before `CG-01` source work
  commits downstream APIs.
- Decide whether simple graphs, directed graphs, and multigraphs share one
  parameterized graph law package or split modules before `CG-06`.
- Decide the graph isomorphism and quotient policy before unlabeled counting,
  Polya counting, and graph-minor interfaces.
- Decide whether graph algorithms use trace records, inductive derivation
  objects, or certificate payloads before `CG-21`.
- Decide when probability dependencies are mature enough to move selected
  `CG-14` probabilistic, graph-limit, or pseudorandom graph targets from `L1`
  to `L2`.
- Decide the first public closure unit before any `L3` promotion. The likely
  first closure is finite counting plus graph basics, not the full roadmap.
