# Representation And Lie Theory Theorem Cards

Source roadmaps:

- `proofs/representation-lie-theory-theorem-proof-roadmap-todo.md`
- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T06`)
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/differential-geometry-theorem-proof-roadmap-todo.md`
- `proofs/operator-functional-analysis-theorem-proof-roadmap-todo.md`
- `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

This file is the `RLT-T00` theorem-card inventory and duplicate-owner map for
representation theory and Lie theory. It is a planning sidecar only. It does
not add trusted proof evidence, axioms, source-free certificate verdicts, or
package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, theorem-search sidecars, this document,
roadmaps, tactics, plugins, and AI output are untrusted.

## Card Legend

| Field | Meaning |
| --- | --- |
| Card | Primary roadmap theorem family. |
| Stable id | English identifier used for later source/module naming. |
| Display | Human-facing theorem family name. |
| Level | Initial target level: `L0 Statement`, dependency-map / blocker, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary milestones | `RLT-T*` task milestones that own the first formalization. |
| Proposed modules | Planned or checked `Proofs.Ai.RepresentationTheory.*` / `Proofs.Ai.LieTheory.*` entry points. |
| Kind | `foundation`, `derived theorem`, `bridge`, `alias`, `long-term interface`, or `package boundary`. |
| Evidence | Main law package, route, model, or hypothesis evidence that must be explicit in statements. |
| Dependencies | Roadmap or module families this card imports or waits for. |
| Gate | First acceptance gate for the card. |

Theorem cards decide ownership, target level, and prerequisite visibility. They
never turn statement text, generated indexes, replay files, or AI output into
proof evidence.

## Namespace Contract

Representation and Lie theory owns these checked route entry points:

| Module | Contract |
| --- | --- |
| `Proofs.Ai.RepresentationTheory.FiniteGroup` | finite-group representation route over explicit group and vector-space prerequisites |
| `Proofs.Ai.RepresentationTheory.GroupAlgebra` | group-algebra action and module-equivalence route |
| `Proofs.Ai.RepresentationTheory.Character` | character, trace, class-function, and orthogonality route |
| `Proofs.Ai.RepresentationTheory.Semisimple` | Maschke-style semisimplicity route with explicit field-characteristic and invertibility assumptions |
| `Proofs.Ai.LieTheory.LieAlgebra` | Lie algebra law package and structural projection route |
| `Proofs.Ai.LieTheory.EnvelopingAlgebra` | universal enveloping algebra and PBW dependency route |
| `Proofs.Ai.LieTheory.LieGroup` | Lie group to Lie algebra bridge route over differential-geometry prerequisites |
| `Proofs.Ai.RepresentationTheory.CompactLie` | compact Lie and unitary representation route with Haar and Hilbert prerequisites explicit |
| `Proofs.Ai.RepresentationTheory.AlgebraicGroup` | algebraic group representation route over algebraic-geometry prerequisites |

Namespace ownership rules:

- `Proofs.Ai.RepresentationTheory.*` owns finite-group representation theory,
  group algebras as representation interfaces, characters, Maschke-style
  semisimplicity, compact Lie representation interfaces, algebraic group
  representation interfaces, and representation-facing aliases.
- `Proofs.Ai.LieTheory.*` owns Lie algebra law packages, universal enveloping
  algebra routes, PBW route interfaces, and Lie group / Lie algebra bridge
  interfaces.
- `Proofs.Ai.LinearAlgebra.*` owns vector spaces, bases, linear maps, trace,
  determinant, eigenvalue, tensor/exterior algebra prerequisites, and
  finite-dimensional linear algebra facts. Representation modules import these
  facts rather than redefining them.
- `Proofs.Ai.Geometry.Differential.*` owns smooth manifolds, tangent spaces,
  vector fields, flows, smooth maps, bundles, differential forms, and analytic
  Lie-group prerequisites. Lie theory owns only the representation-facing
  bridge after those prerequisites are explicit.
- `Proofs.Ai.FunctionalAnalysis.*`, measure, and analysis modules own Hilbert
  space, spectral, Haar-measure, integration, and compactness prerequisites
  needed by unitary and compact-group route cards.
- `Proofs.Ai.AlgebraicGeometry.*`, arithmetic geometry, and Langlands modules
  own algebraic varieties/schemes, automorphic, modular, and arithmetic
  consequences. Representation modules provide aliases or dependencies only
  after primary arithmetic/geometric owners are explicit.

## Primary Roadmap Cards

| Card | Stable id | Display | Level | Primary milestones | Proposed modules | Kind |
| --- | --- | --- | --- | --- | --- | --- |
| `RLT-00` | `representation_lie_inventory_statement_policy` | Representation and Lie theory inventory | `L0 Statement` | `RLT-T00` | this file; future `Proofs.Ai.RepresentationTheory.Inventory` | foundation |
| `RLT-01` | `finite_group_representation_core` | Finite-group representation core | `L2 Derived certificate` | `RLT-T01` | `Proofs.Ai.RepresentationTheory.FiniteGroup` | foundation |
| `RLT-02` | `group_algebra_module_route` | Group algebra and module route | `L2 Derived certificate` | `RLT-T02` | `Proofs.Ai.RepresentationTheory.GroupAlgebra` | derived theorem |
| `RLT-03` | `character_trace_orthogonality_route` | Character, trace, and orthogonality route | `L2 Derived certificate` | `RLT-T03` | `Proofs.Ai.RepresentationTheory.Character` | derived theorem |
| `RLT-04` | `maschke_semisimplicity_route` | Maschke and semisimplicity route | `L2 Derived certificate` | `RLT-T04` | `Proofs.Ai.RepresentationTheory.Semisimple` | derived theorem |
| `RLT-05` | `lie_algebra_law_package` | Lie algebra core | `L2 Derived certificate` | `RLT-T05` | `Proofs.Ai.LieTheory.LieAlgebra` | foundation |
| `RLT-06` | `universal_enveloping_pbw_route` | Universal enveloping algebra and PBW route | `L2 Derived certificate` for route package; PBW construction prerequisites split | `RLT-T06` | `Proofs.Ai.LieTheory.EnvelopingAlgebra` | bridge, long-term interface |
| `RLT-07` | `lie_group_lie_algebra_bridge` | Lie group and Lie algebra bridge | `L2 Derived certificate` | `RLT-T07` | `Proofs.Ai.LieTheory.LieGroup` | bridge |
| `RLT-08` | `compact_lie_unitary_representation_route` | Compact Lie and unitary representation routes | `L2 Derived certificate` for route package; Haar/Peter-Weyl prerequisites explicit | `RLT-T08` | `Proofs.Ai.RepresentationTheory.CompactLie` | bridge, long-term interface |
| `RLT-09` | `algebraic_group_representation_route` | Algebraic group representation route | `L2 Derived certificate` for route package; highest-weight/category-O prerequisites split | `RLT-T09` | `Proofs.Ai.RepresentationTheory.AlgebraicGroup` | bridge, long-term interface |
| `RLT-10` | `harmonic_modular_langlands_aliases` | Harmonic, modular, and Langlands aliases | `L0` alias and duplicate-owner map | `RLT-T10` | `Proofs.Ai.ArithmeticGeometry.*`, `Proofs.Ai.Analysis.*` aliases only | alias |

## Checked Route Certificate Register

| Card | Module | Route package intro theorem | Derived route theorem | Certificate path |
| --- | --- | --- | --- | --- |
| `RLT-01` | `Proofs.Ai.RepresentationTheory.FiniteGroup` | `representation_theory_finite_group_route_package_intro` | `rlt_t_01_add_finite_group_representation_core_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/FiniteGroup/certificate.npcert` |
| `RLT-02` | `Proofs.Ai.RepresentationTheory.GroupAlgebra` | `representation_theory_group_algebra_route_package_intro` | `rlt_t_02_add_group_algebra_and_module_route_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/GroupAlgebra/certificate.npcert` |
| `RLT-03` | `Proofs.Ai.RepresentationTheory.Character` | `representation_theory_character_route_package_intro` | `rlt_t_03_add_character_and_orthogonality_route_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/Character/certificate.npcert` |
| `RLT-04` | `Proofs.Ai.RepresentationTheory.Semisimple` | `representation_theory_semisimple_route_package_intro` | `rlt_t_04_add_maschke_and_semisimplicity_route_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/Semisimple/certificate.npcert` |
| `RLT-05` | `Proofs.Ai.LieTheory.LieAlgebra` | `lie_theory_lie_algebra_route_package_intro` | `rlt_t_05_add_lie_algebra_core_route_derived` | `proofs/Proofs/Ai/LieTheory/LieAlgebra/certificate.npcert` |
| `RLT-06` | `Proofs.Ai.LieTheory.EnvelopingAlgebra` | `lie_theory_enveloping_algebra_route_package_intro` | `rlt_t_06_add_universal_enveloping_algebra_and_pbw_route_route_derived` | `proofs/Proofs/Ai/LieTheory/EnvelopingAlgebra/certificate.npcert` |
| `RLT-07` | `Proofs.Ai.LieTheory.LieGroup` | `lie_theory_lie_group_route_package_intro` | `rlt_t_07_add_lie_group_and_lie_algebra_bridge_route_derived` | `proofs/Proofs/Ai/LieTheory/LieGroup/certificate.npcert` |
| `RLT-08` | `Proofs.Ai.RepresentationTheory.CompactLie` | `representation_theory_compact_lie_route_package_intro` | `rlt_t_08_add_compact_lie_and_unitary_representation_routes_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/CompactLie/certificate.npcert` |
| `RLT-09` | `Proofs.Ai.RepresentationTheory.AlgebraicGroup` | `representation_theory_algebraic_group_route_package_intro` | `rlt_t_09_add_algebraic_group_representation_route_route_derived` | `proofs/Proofs/Ai/RepresentationTheory/AlgebraicGroup/certificate.npcert` |

The register records the current checked route certificates. The certificate
paths are the proof evidence; this table is only an index.

## Evidence And Dependency Map

| Card | Evidence | Dependencies | Gate |
| --- | --- | --- | --- |
| `RLT-00` | roadmap review, theorem-card inventory, duplicate-owner map, target levels, assumption taxonomy; no source, replay, theorem index, or todo evidence | roadmap only | `git diff --check` |
| `RLT-01` | explicit finite group action, vector-space carrier, linear automorphism/action-law route, invariant subspace and direct-sum projection evidence | `LIN-T05` and linear-map foundations | source-free module verify |
| `RLT-02` | group algebra action route, module equivalence route, imported ring/module assumptions | `RLT-01`, `CMA-T03`, tensor/module foundations | source-free module verify |
| `RLT-03` | trace, character, class-function, inner-product, irreducibility, finite group averaging, and orthogonality route evidence | `RLT-01`, finite-dimensional trace from `LIN-T26` | source-free module verify |
| `RLT-04` | field characteristic, group-order invertibility, averaging/projection evidence, complement route, semisimplicity route | `RLT-01`, `RLT-03`, field and finite algebra prerequisites | source-free module verify |
| `RLT-05` | Lie algebra law package, bracket bilinearity, antisymmetry, Jacobi, homomorphism, ideal, subalgebra, and representation-by-derivations evidence | linear algebra and tensor prerequisites | source-free module verify |
| `RLT-06` | universal enveloping algebra route, quotient/tensor prerequisites, filtration and basis blockers for PBW | `RLT-05`, `CMA-T06`, tensor, quotient, filtration, and basis routes | source-free module verify for route; construction blockers before deeper source |
| `RLT-07` | smooth group law package, tangent identity data, bracket/flow prerequisites, Lie algebra bridge route | `RLT-05`, `DG-T04`, tangent and vector-field routes | source-free module verify |
| `RLT-08` | compact group, unitary representation, Haar averaging, Hilbert space, spectral and Peter-Weyl dependency visibility | `RLT-07`, `ANA-T28`, measure/functional-analysis prerequisites | source-free module verify for route; interface audit before Peter-Weyl use |
| `RLT-09` | algebraic group object, representation action, coordinate/geometric prerequisites, reductive/highest-weight/category-O split | `RLT-05`, `AG-T03`, commutative algebra and algebraic geometry routes | source-free module verify for route; interface audit before deeper source |
| `RLT-10` | alias map from harmonic, modular, automorphic, and Langlands-facing users to primary representation cards | `RLT-03`, `RLT-08`, `AGL-T10`, analysis and arithmetic-geometry owners | roadmap audit only |

## Duplicate-Owner Map

| Theorem family or alias | Primary home | Representation/Lie status | Reason |
| --- | --- | --- | --- |
| vector spaces, bases, linear maps, matrices, trace, determinant, eigenvalue facts | `Proofs.Ai.LinearAlgebra.*` | external prerequisite | representation theory consumes linear structure but does not duplicate linear algebra proofs |
| finite-group representation law packages and invariant subspace projections | `RLT-01` | primary here | group action by linear automorphisms is representation-specific |
| group algebra action and representation-as-module equivalence | `RLT-02` | primary here | commutative algebra owns ring/module infrastructure; representation theory owns this representation interface |
| character, trace-as-character, class functions, character inner product, and orthogonality | `RLT-03` | primary here | linear algebra owns trace primitives; representation theory owns character-theoretic statements |
| Maschke theorem, finite-group semisimplicity, and complement routes | `RLT-04` | primary here | field characteristic and group-order assumptions are representation-theoretic theorem hypotheses |
| abstract module semisimplicity, Noetherian/Artinian module facts, tensor exactness | commutative algebra and homological algebra roadmaps | external prerequisite | these are general algebra results, not representation-specific ownership |
| Lie algebra law package, ideals, subalgebras, homomorphisms, and derivation action routes | `RLT-05` | primary here | Lie-theoretic bracket laws and representation by derivations belong to Lie theory |
| smooth vector-field brackets, flows, tangent bundles, differential forms | differential geometry roadmap | external prerequisite | smooth manifold evidence is not owned by Lie theory |
| universal enveloping algebra and PBW route | `RLT-06` | primary route here | tensor, quotient, filtration, and basis facts remain external prerequisites |
| Lie group to Lie algebra bridge | `RLT-07` | primary bridge here | differential geometry supplies smooth/tangent evidence; Lie theory owns the bridge theorem family |
| compact Lie unitary representation, Haar averaging, Peter-Weyl interface | `RLT-08` | primary route here | Haar and Hilbert prerequisites are external, but representation-facing statement ownership is here |
| Haar measure, integration, Hilbert spectral theorem, operator theory | measure, analysis, and functional-analysis roadmaps | external prerequisite | analytic evidence must not be hidden inside representation law packages |
| algebraic group representation, reductive group and highest-weight interfaces | `RLT-09` | primary route here | algebraic geometry owns schemes/varieties; representation theory owns representation statements |
| algebraic group object, scheme, smooth/flat/etale topology, coherent sheaves | algebraic geometry roadmap | external prerequisite | geometric infrastructure is not duplicated in representation modules |
| harmonic analysis aliases and unitary representation uses | analysis / functional analysis for analytic statements, `RLT-08` for representation interface | split owner | analytic theorems import compact/unitary representation routes only after explicit prerequisites |
| automorphic, modular, Langlands, trace formula, and arithmetic consequences | arithmetic geometry and Langlands roadmap | alias only here | representation theory supplies dependency names; arithmetic owners keep arithmetic theorem statements |

## Assumption Taxonomy

| Assumption family | Required visibility |
| --- | --- |
| finite-dimensionality | stated on representation, character, trace, semisimplicity, and compact Lie cards before finite-dimensional linear algebra is used |
| field characteristic and group-order invertibility | explicit in Maschke and finite-group semisimplicity routes |
| algebraic closedness / splitting field | explicit on character and highest-weight interfaces; never hidden in a generic field law package |
| compactness | explicit on compact Lie and unitary representation routes; topological or Lie group compactness is imported |
| Haar measure and averaging | named measure/analysis prerequisite; not encoded as a representation theorem axiom |
| smoothness and manifold structure | imported from differential geometry for Lie group bridge routes |
| quotient, tensor, filtration, and basis evidence | imported from algebra/linear algebra for enveloping algebra and PBW routes |
| choice or maximality principles | named wherever complements, invariant inner products, or decomposition arguments require them |
| arithmetic or automorphic hypotheses | left with arithmetic geometry / Langlands owners and referenced only through alias cards |

## First Execution Queue

| Queue ID | Theorem or task | Target level | Primary milestone | Primary card |
| --- | --- | --- | --- | --- |
| `RLTQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `RLT-T00` | `RLT-00` |
| `RLTQ-002` | finite-group representation core | `L2` | `RLT-T01` | `RLT-01` |
| `RLTQ-003` | character and trace route | `L2` where prerequisites exist | `RLT-T03` | `RLT-03` |
| `RLTQ-004` | Lie algebra law package | `L2` | `RLT-T05` | `RLT-05` |
| `RLTQ-005` | Lie group bridge dependency split | route package first | `RLT-T07` | `RLT-07` |

## Milestone-To-Card Checklist

| Roadmap item | Card present | Primary home unique | Prerequisites explicit | Sidecar trust boundary clear |
| --- | --- | --- | --- | --- |
| `RLT-T00` | yes | yes | yes | yes |
| `RLT-T01` | yes | yes | yes | yes |
| `RLT-T02` | yes | yes | yes | yes |
| `RLT-T03` | yes | yes | yes | yes |
| `RLT-T04` | yes | yes | yes | yes |
| `RLT-T05` | yes | yes | yes | yes |
| `RLT-T06` | yes | yes | yes | yes |
| `RLT-T07` | yes | yes | yes | yes |
| `RLT-T08` | yes | yes | yes | yes |
| `RLT-T09` | yes | yes | yes | yes |
| `RLT-T10` | yes | yes | yes | yes |

## Review Checklist

- Every `RLT-T*` milestone has a card or an intentionally grouped alias card.
- L2 route certificates are indexed by module, theorem declaration, and
  certificate path, but this sidecar is not proof evidence.
- Linear algebra, differential geometry, functional analysis, measure,
  algebraic geometry, arithmetic geometry, and Langlands ownership boundaries
  are visible.
- PBW, Peter-Weyl, highest-weight, category O, and Langlands-facing statements
  are not used as evidence before lower-level proof certificates exist.
- Public package work remains outside this sidecar until closure audit confirms
  stable `L2` derived certificates.
