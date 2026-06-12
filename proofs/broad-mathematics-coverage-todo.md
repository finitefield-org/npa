# Broad Mathematics Coverage Todo

Source context:

- `proofs/README.md`
- `develop/proof-corpus-field-theory-roadmap.md`
- `proofs/analysis-theorem-proof-roadmap.md`
- `proofs/measure-theory-theorem-proof-roadmap.md`
- `proofs/topology-theorem-proof-roadmap.md`
- `proofs/linear-algebra-theorem-proof-roadmap.md`
- `proofs/number-theory-theorem-proof-roadmap.md`
- `proofs/combinatorics-graph-theorem-proof-roadmap.md`
- `proofs/set-theory-theorem-proof-roadmap.md`
- `proofs/statistics-theorem-proof-roadmap.md`

This document is a cross-roadmap backlog for mathematical fields and theorem
families that are needed for broad theorem proving but are not yet covered by a
dedicated theorem-proof roadmap or are only present as scattered module
interfaces. It is a planning sidecar only: it does not add trusted proof
evidence, axioms, source-free certificate verdicts, or package verification
claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, theorem-search sidecars, this document,
roadmaps, tactics, plugins, and AI output are untrusted.

## Existing Roadmap Coverage

The current proof TODO set already has dedicated or detailed roadmap coverage
for these major areas:

- algebra and field-theory foundations through the field-theory roadmap;
- analysis, measure theory, topology, and functional-analysis-adjacent routes;
- linear algebra and finite-dimensional spectral routes;
- combinatorics and graph theory;
- number theory, including many arithmetic and modular-form routes;
- set theory foundations and selected metatheoretic interfaces;
- statistics, probability, and finite/asymptotic inference routes.

The additions below should be treated as missing broad-math coverage, not as
permission to land `L1` theorem-shaped assumptions. New work should target `L2`
derived certificates from the first proof attempt whenever prerequisites exist.
If prerequisites are missing, split a blocker or prerequisite task before source
edits.

## Coverage Gaps To Add

| ID | Missing field or theorem area | Why it matters for broad theorem proving | First roadmap artifact to add | Initial target level |
| --- | --- | --- | --- | --- |
| `BMC-T01` | Category theory and universal constructions | Needed to state and reuse functoriality, naturality, adjunctions, limits, sheaves, derived categories, monoidal structures, and categorical semantics without duplicating patterns in algebra, topology, and geometry. | `proofs/category-theory-theorem-proof-roadmap.md` and todo/cards | `L2` for small category/functor/natural-transformation lemmas; split larger existence results |
| `BMC-T02` | Homological algebra | Needed by algebraic topology, algebraic geometry, representation theory, number theory, and derived functor routes. | `proofs/homological-algebra-theorem-proof-roadmap.md` and todo/cards | `L2` for exact-sequence lemmas; `L1` only for construction-heavy derived functor interfaces |
| `BMC-T03` | Commutative algebra and module theory | Current algebra/field modules are broad but do not yet give a roadmap for Noetherian rings, localization, ideals/modules, dimension, primary decomposition, and algebraic-geometry prerequisites. | `proofs/commutative-algebra-theorem-proof-roadmap.md` and todo/cards | `L2` for ideal/module/localization laws where explicit law packages exist |
| `BMC-T04` | Algebraic geometry | Several `Proofs.Ai.AlgebraicGeometry.*` modules exist, but a theorem-card roadmap should own affine/projective varieties, schemes, sheaves, morphisms, fiber products, etale/smooth/flat maps, cohomology, and Riemann-Roch-style routes. | `proofs/algebraic-geometry-theorem-proof-roadmap.md` and todo/cards | `L2` for affine/scheme structural lemmas; `L1/L2` split for cohomology and derived routes |
| `BMC-T05` | Differential geometry and smooth manifolds | Topology and analysis cover inputs, but smooth manifolds, tangent/cotangent bundles, vector fields, differential forms, integration on manifolds, Stokes, de Rham, Riemannian geometry, curvature, and Gauss-Bonnet need a primary home. | `proofs/differential-geometry-theorem-proof-roadmap.md` and todo/cards | `L2` for smooth-map/tangent/form algebra where foundations exist; split integration/geometric analysis blockers |
| `BMC-T06` | Lie theory and representation theory | Needed for linear algebra, harmonic analysis, modular forms, algebraic geometry, Langlands, and mathematical physics routes. | `proofs/representation-lie-theory-theorem-proof-roadmap.md` and todo/cards | `L2` for finite-group representation facts; split Lie-group analytic prerequisites |
| `BMC-T07` | Operator algebras and advanced functional analysis | Existing functional-analysis modules are useful but do not yet cover Banach algebra, C*-algebra, von Neumann algebra, spectral calculus, compact operators, distributions, or locally convex spaces as a roadmap family. | `proofs/operator-functional-analysis-theorem-proof-roadmap.md` and todo/cards | `L2` for algebraic/operator-law projections; split analytic existence theorems |
| `BMC-T08` | Stochastic processes and stochastic calculus | Statistics has process milestones, and measure has martingale/ergodic bridges, but Brownian motion, Ito integral, Ito formula, SDEs, stopping times, Girsanov, Feynman-Kac, and Markov process semigroups need a primary route. | `proofs/stochastic-calculus-theorem-proof-roadmap.md` and todo/cards | `L2` only after measure/probability foundations; start with explicit finite/discrete process certificates |
| `BMC-T09` | Numerical analysis and approximation theory | Linear algebra and analysis mention numerical routes, but interpolation, quadrature, approximation theorems, finite elements, stability, conditioning, and convergence of algorithms need a dedicated proof plan. | `proofs/numerical-analysis-theorem-proof-roadmap.md` and todo/cards | `L2` for deterministic recurrence/error-bound theorems with explicit norm assumptions |
| `BMC-T10` | Optimization and operations research | Convex optimization appears in analysis, statistics, and linear algebra, but LP duality, Farkas, KKT, Fenchel duality, separation, simplex/interior-point correctness, dynamic programming, and game-theoretic minimax should have one ownership map. | `proofs/optimization-theorem-proof-roadmap.md` and todo/cards | `L2` for finite-dimensional convex/LP certificates; split algorithm traces from theorem conclusions |
| `BMC-T11` | Mathematical logic, model theory, and computability | Set theory covers axioms and some model-relative work, but first-order syntax/semantics, soundness, completeness, compactness, Lowenheim-Skolem, definability, computability, recursion theory, and proof theory need a separate roadmap. | `proofs/logic-model-theory-theorem-proof-roadmap.md` and todo/cards | `L2` for syntax/semantics structural lemmas; meta-theorems require explicit metatheory assumptions |
| `BMC-T12` | Theoretical computer science and algorithms | Graph algorithms exist, but automata, languages, complexity classes, reductions, data structures, randomized algorithms, type-theoretic semantics, and algorithm-correctness schemas need a broader plan. | `proofs/theoretical-computer-science-theorem-proof-roadmap.md` and todo/cards | `L2` for trace/correctness theorems; complexity lower bounds as explicit model-dependent targets |
| `BMC-T13` | Coding theory, information theory, and cryptography | Statistics covers information theory and number theory covers some crypto, but Hamming/Singleton/Gilbert-Varshamov bounds, Shannon coding, finite-field codes, protocol correctness, and security-assumption boundaries need a primary roadmap. | `proofs/coding-cryptography-theorem-proof-roadmap.md` and todo/cards | `L2` for algebraic correctness and finite-code bounds; security claims remain assumption-explicit |
| `BMC-T14` | Arithmetic geometry, elliptic curves, modularity, and Langlands | Modules exist for elliptic curves, arithmetic geometry, modularity, and Langlands interfaces, but broad theorem proving needs ownership for Mordell-Weil, heights, Galois representations, etale cohomology, modularity lifting, trace formula, and automorphic forms. | `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap.md` and todo/cards | `L2` for algebraic/finite-field structural lemmas; major conjectural or theorem-heavy routes as explicit dependency maps |
| `BMC-T15` | Algebraic topology beyond general topology | The topology roadmap covers algebraic topology milestones, but homotopy groups, spectra, generalized cohomology, spectral sequences, characteristic classes, K-theory, bordism, and stable homotopy need a separate long-range execution plan once foundations mature. | `proofs/algebraic-topology-theorem-proof-roadmap.md` and todo/cards | `L2` for chain/simplicial algebra; split stable homotopy and spectral sequence prerequisites |
| `BMC-T16` | Geometry outside smooth manifolds | Current geometry is mostly metric/right-triangle focused. Broad coverage needs Euclidean, affine, projective, convex, discrete, symplectic, algebraic, and incidence geometry theorem ownership. | `proofs/geometry-theorem-proof-roadmap.md` and todo/cards | `L2` for finite/affine/metric route certificates; split smooth/algebraic dependencies to their owners |
| `BMC-T17` | Mathematical physics interfaces | Broad theorem libraries often need Hamiltonian/Lagrangian mechanics, symplectic geometry, PDE models, quantum operator formalism, statistical mechanics, and gauge-theory interfaces. These should stay assumption-explicit and not block core mathematics. | `proofs/mathematical-physics-theorem-proof-roadmap.md` and todo/cards | `L1/L2` split; no physical postulate as hidden proof evidence |

## First Addition Queue

| Queue item | First deliverable | Why first |
| --- | --- | --- |
| `BMQ-001` | Category theory roadmap/cards (`BMC-T01`) | Provides reusable language for later algebraic geometry, homological algebra, sheaves, limits, adjunctions, and semantics. |
| `BMQ-002` | Commutative algebra roadmap/cards (`BMC-T03`) | Unlocks algebraic geometry, algebraic number theory, module theory, and localization-dependent routes. |
| `BMQ-003` | Homological algebra roadmap/cards (`BMC-T02`) | Unlocks algebraic topology, sheaf cohomology, derived functors, Ext/Tor, and spectral-sequence routes. |
| `BMQ-004` | Differential geometry roadmap/cards (`BMC-T05`) | Gives Stokes/de Rham/manifold theorems a primary home instead of scattering them across topology, analysis, and measure TODOs. |
| `BMQ-005` | Optimization roadmap/cards (`BMC-T10`) | Resolves overlap between analysis convexity, linear algebra, statistics computation, and combinatorial optimization. |
| `BMQ-006` | Logic/model theory roadmap/cards (`BMC-T11`) | Makes model-relative and metatheoretic assumptions visible before forcing, completeness, or independence-style theorem targets expand. |
| `BMQ-007` | Arithmetic geometry/Langlands roadmap/cards (`BMC-T14`) | Organizes existing modules into an auditable theorem plan before adding more high-level arithmetic interfaces. |
| `BMQ-008` | Stochastic calculus roadmap/cards (`BMC-T08`) | Splits continuous-time stochastic work away from finite statistics while reusing measure/probability foundations. |

## Initial Todo Artifacts

The first eight queue items now have dedicated TODO files:

| Queue item | Todo file |
| --- | --- |
| `BMQ-001` | `proofs/category-theory-theorem-proof-roadmap-todo.md` |
| `BMQ-002` | `proofs/commutative-algebra-theorem-proof-roadmap-todo.md` |
| `BMQ-003` | `proofs/homological-algebra-theorem-proof-roadmap-todo.md` |
| `BMQ-004` | `proofs/differential-geometry-theorem-proof-roadmap-todo.md` |
| `BMQ-005` | `proofs/optimization-theorem-proof-roadmap-todo.md` |
| `BMQ-006` | `proofs/logic-model-theory-theorem-proof-roadmap-todo.md` |
| `BMQ-007` | `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md` |
| `BMQ-008` | `proofs/stochastic-calculus-theorem-proof-roadmap-todo.md` |

## Ownership Rules

- If a theorem already has a primary owner in an existing roadmap, add only an
  alias or dependency note in the new roadmap.
- If a theorem crosses two fields, choose the owner by the structure that
  supplies the conclusion, not by the first field that uses it.
- If a theorem depends on choice, quotient features, replacement, classical
  reasoning, model theory, category-theoretic universes, physical postulates,
  computational complexity assumptions, or cryptographic hardness assumptions,
  expose that dependency in the theorem statement or law package.
- Do not add statement-only or theorem-shaped `L1` assumptions as a shortcut
  for the final theorem. Split missing prerequisites into blockers or smaller
  `L2` tasks.
- Promotion to `npa-mathlib` is out of scope for this coverage backlog. Each
  future field roadmap must get its own closure audit before public
  materialization.

## Verification

Document-only updates to this backlog should at least pass:

```sh
rg -n "BMC-T01|BMC-T17|BMQ-001|Broad Mathematics Coverage" proofs/broad-mathematics-coverage-todo.md proofs/README.md
git diff --check
```
