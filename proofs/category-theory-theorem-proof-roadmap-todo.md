# Category Theory Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T01`, `BMQ-001`)
- `proofs/topology-theorem-proof-roadmap-todo.md`
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/set-theory-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for category theory and
universal constructions. A full
`proofs/category-theory-theorem-proof-roadmap.md` can be split from this file
later, but this document already fixes the execution order and trust boundary
for authoring work. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers small-category foundations, functors, natural
transformations, isomorphisms and equivalences, universal constructions,
limits and colimits, adjunctions, monads, Yoneda-style representation facts,
monoidal categories, sheaf-oriented categorical vocabulary, model categories,
infinity-category interfaces, and closure-boundary planning.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding categories, functors, universes, quotients, choice, or higher
  categories as trusted kernel primitives;
- treating theorem-shaped category interfaces as proof evidence for later
  algebraic geometry, homological algebra, or topology theorems;
- hiding universe-size, choice, quotient, replacement, or coherence
  assumptions in uninspected law packages;
- publicly materializing category modules into `npa-mathlib` before closure audit,
  axiom-report review, and package verification are clean.

## Authoring Loop

For ordinary category-theory authoring, prefer local proof-corpus checks:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Checked category-related module trees already include
  `Proofs.Ai.Category.Basic`, `Proofs.Ai.Category.Functor`,
  `Proofs.Ai.Category.NaturalTransformation`,
  `Proofs.Ai.Category.Equivalence`,
  `Proofs.Ai.Category.Limit.Basic`, `Proofs.Ai.Category.Adjunction`,
  `Proofs.Ai.Category.Yoneda`, `Proofs.Ai.Category.Monoidal.Basic`,
  `Proofs.Ai.Category.SheafRoute`, `Proofs.Ai.Category.ModelCategory`,
  `Proofs.Ai.Category.MonoidalModelCategory`,
  `Proofs.Ai.Category.Infinity.SimplicialSet`, and
  `Proofs.Ai.Category.Infinity.StableInfinityCategory`.
- On 2026-06-13, category split modules were materialized without public package work:
  `Proofs.Ai.Category.Basic`, `Proofs.Ai.Category.Functor`,
  `Proofs.Ai.Category.NaturalTransformation`,
  `Proofs.Ai.Category.Equivalence`,
  `Proofs.Ai.Category.Limit.Basic`, `Proofs.Ai.Category.Adjunction`,
  `Proofs.Ai.Category.Yoneda`,
  `Proofs.Ai.Category.Monoidal.Basic`, and
  `Proofs.Ai.Category.SheafRoute`.
- Later on 2026-06-13, the former monolithic category module was deleted. Its
  proof bodies now live directly in the split modules above, and no
  compatibility alias module remains.
- The previous thin alias wrappers such as `category_basic_*`,
  `functor_core_*`, `natural_transformation_core_*`,
  `adjunction_core_*`, `yoneda_route_lemma`, and
  `sheaf_route_sheafification_theorem` were removed from generated category
  sources. The canonical theorem names now live directly in their primary
  category modules.
- The 2026-06-13 source-free authoring verification covered the new split
  modules plus `Proofs.Ai.Category.ModelCategory`,
  `Proofs.Ai.Category.MonoidalModelCategory`,
  `Proofs.Ai.Category.Infinity.SimplicialSet`, and
  `Proofs.Ai.Category.Infinity.StableInfinityCategory`.
- Public package metadata was refreshed only to remove the deleted monolithic
  category module and its stale imports. No public package artifact
  was produced.
- Algebraic-geometry modules already use category-shaped names such as
  `Proofs.Ai.AlgebraicGeometry.DerivedCategory` and
  `Proofs.Ai.AlgebraicGeometry.QuasiCoherentSheaves`; this todo makes
  category ownership explicit before more downstream aliases are added.
- Set theory owns quotient, choice, replacement, and universe-like
  foundations when those assumptions are needed. Category modules must import
  those assumptions explicitly instead of treating them as built-in.
- Topology and algebraic topology own topological consequences. Category
  theory owns only the reusable universal-construction vocabulary and route
  packages those theorem families may import.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `CAT-00` inventory and namespace contract | `CAT-T00` |
| `CAT-01` small categories and categorical equality | `CAT-T01` |
| `CAT-02` functors and composition laws | `CAT-T02` |
| `CAT-03` natural transformations and naturality | `CAT-T03` |
| `CAT-04` isomorphisms, equivalences, and fully faithful functors | `CAT-T04` |
| `CAT-05` products, pullbacks, limits, and colimits | `CAT-T05` |
| `CAT-06` adjunctions, monads, and reflective subcategories | `CAT-T06` |
| `CAT-07` representables and Yoneda route | `CAT-T07` |
| `CAT-08` monoidal and enriched category basics | `CAT-T08` |
| `CAT-09` sheaf and indexed-category vocabulary | `CAT-T09` |
| `CAT-10` model, infinity, and stable categories | `CAT-T10` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `CAT-T00` | `L0` planning, theorem-card inventory, duplicate-map maintenance |
| `CAT-T01` through `CAT-T04` | `L2` derived certificates from explicit law packages whenever possible |
| `CAT-T05` through `CAT-T09` | `L2` for finite or algebraic universal-property lemmas; split existence-heavy results before source edits |
| `CAT-T10` | interface audit first; replace or split theorem-shaped assumptions before downstream reuse |

## Milestones

### CAT-T00 Build Category Theorem Card Inventory

- Status: Completed 2026-06-13
- Depends on: None
- Areas: `proofs/category-theory-theorem-proof-roadmap-todo.md`,
  future category theorem cards
- Tasks:
  - Inventory existing `Proofs.Ai.Category.*` modules and downstream
    category-shaped modules.
  - Assign primary homes for category, functor, natural transformation,
    equivalence, limit, adjunction, Yoneda, monoidal, sheaf, and higher
    category theorem families.
  - Record universe, quotient, choice, and coherence assumptions for every
    theorem card.
- Deliverables:
  - Category theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Every downstream category-shaped theorem has exactly one primary owner.
  - The inventory distinguishes derived certificates from interface packages.
- Verification:
  - `rg -n "CAT-T00|Category|Functor|Yoneda|Adjunction" proofs/category-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CAT-T01 Add Small Category Law Package

- Status: Completed 2026-06-13
- Depends on: `CAT-T00`
- Areas: `Proofs.Ai.Category.Basic`
- Tasks:
  - Define object, morphism, identity, composition, associativity, and unit
    law packages for small categories.
  - Add derived identity-composition and reassociation lemmas.
  - Keep universe-size assumptions explicit.
- Deliverables:
  - Source, certificate, replay, and metadata for a basic small-category
    module.
- Acceptance criteria:
  - Associativity and unit consequences are derived from the law package.
  - No set-theoretic universe assumption is hidden in the module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Basic --verified-cache authoring`

### CAT-T02 Add Functor Composition Core

- Status: Completed 2026-06-13
- Depends on: `CAT-T01`
- Areas: `Proofs.Ai.Category.Functor`
- Tasks:
  - Define functors between explicit category law packages.
  - Prove identity functor and functor composition laws.
  - Add opposite-category route only if the needed object/morphism reversal
    assumptions are explicit.
- Deliverables:
  - Functor core module with composition certificates.
- Acceptance criteria:
  - Functor laws are not assumed as theorems for composition results.
  - Opposite category is split if it needs additional equality machinery.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Functor`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Functor --verified-cache authoring`

### CAT-T03 Add Natural Transformation Core

- Status: Completed 2026-06-13
- Depends on: `CAT-T02`
- Areas: `Proofs.Ai.Category.NaturalTransformation`
- Tasks:
  - Define natural transformation law packages between parallel functors.
  - Prove vertical composition and identity natural transformation laws.
  - Add horizontal composition only after the interchange assumptions are
    explicit.
- Deliverables:
  - Natural transformation module and first naturality certificates.
- Acceptance criteria:
  - Naturality squares are visible hypotheses or derived facts.
  - Interchange law is not smuggled into a theorem-shaped interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.NaturalTransformation`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CAT-T04 Add Isomorphism And Equivalence Routes

- Status: Completed 2026-06-13
- Depends on: `CAT-T03`
- Areas: `Proofs.Ai.Category.Equivalence`
- Tasks:
  - Define isomorphisms, natural isomorphisms, full and faithful functors, and
    essential surjectivity.
  - Prove elementary equivalence transport lemmas.
  - Split any quotient-of-objects or skeleton theorem behind set-theory
    prerequisites.
- Deliverables:
  - Equivalence route module with explicit quotient and choice boundaries.
- Acceptance criteria:
  - Equivalence results state whether they are strict, up-to-isomorphism, or
    quotient-mediated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Equivalence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Equivalence --verified-cache authoring`

### CAT-T05 Add Finite Universal Construction Core

- Status: Completed 2026-06-13
- Depends on: `CAT-T04`
- Areas: `Proofs.Ai.Category.Limit.Basic`
- Tasks:
  - Define terminal, initial, product, coproduct, equalizer, coequalizer,
    pullback, and pushout universal-property packages.
  - Prove uniqueness up to unique isomorphism for finite universal objects.
  - Keep general small limits and colimits as later route packages.
- Deliverables:
  - Finite universal-construction certificates.
- Acceptance criteria:
  - Universal uniqueness proofs do not assume the conclusion as an interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Limit.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CAT-T06 Add Adjunction And Monad Routes

- Status: Completed 2026-06-13
- Depends on: `CAT-T05`
- Areas: `Proofs.Ai.Category.Adjunction`
- Tasks:
  - Define hom-set adjunction and unit-counit adjunction packages.
  - Prove triangle-identity equivalence where equality prerequisites exist.
  - Add monad and reflective-subcategory statement routes.
- Deliverables:
  - Adjunction core module and monad route map.
- Acceptance criteria:
  - Unit-counit laws are explicit, and equivalence directions are not merged
    without proof evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Adjunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Adjunction --verified-cache authoring`

### CAT-T07 Add Representable And Yoneda Route

- Status: Completed 2026-06-13
- Depends on: `CAT-T06`
- Areas: `Proofs.Ai.Category.Yoneda`
- Tasks:
  - Define representable functors and natural transformations into set-valued
    functors.
  - Split covariant and contravariant Yoneda statements.
  - Record set-valued functor and universe assumptions.
- Deliverables:
  - Yoneda route package with explicit prerequisites.
- Acceptance criteria:
  - The Yoneda statement does not hide extensionality, quotient, or universe
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Yoneda`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CAT-T08 Add Monoidal Category Basics

- Status: Completed 2026-06-13
- Depends on: `CAT-T05`
- Areas: `Proofs.Ai.Category.Monoidal.Basic`
- Tasks:
  - Define tensor, unit object, associator, left and right unitors, and
    coherence law packages.
  - Prove elementary tensor functoriality projections.
  - Audit existing `MonoidalModelCategory` assumptions against the new core.
- Deliverables:
  - Monoidal-category basic module and compatibility notes.
- Acceptance criteria:
  - Coherence assumptions are named law-package fields, not hidden axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Category.Monoidal.Basic`
  - `rg -n "MonoidalModelCategory|coherence|tensor" proofs/category-theory-theorem-proof-roadmap-todo.md proofs/Proofs/Ai/Category`

### CAT-T09 Split Sheaf And Indexed-Category Dependencies

- Status: Completed 2026-06-13
- Depends on: `CAT-T05`, `CAT-T07`
- Areas: `Proofs.Ai.Category.SheafRoute`
- Tasks:
  - Define presheaf, sheaf-condition, Grothendieck topology, and indexed
    category route packages.
  - Assign concrete sheaf cohomology to homological algebra or algebraic
    geometry when appropriate.
  - Keep site and coverage assumptions visible.
- Deliverables:
  - Sheaf dependency map and route module.
- Acceptance criteria:
  - Sheaf routes do not duplicate topology or algebraic-geometry ownership.
- Verification:
  - `rg -n "sheaf|Grothendieck|presheaf|site" proofs/category-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CAT-T10 Audit Model And Infinity Category Interfaces

- Status: Completed 2026-06-13
- Depends on: `CAT-T08`, `CAT-T09`
- Areas: `Proofs.Ai.Category.ModelCategory`,
  `Proofs.Ai.Category.Infinity.*`
- Tasks:
  - Review existing model-category and infinity-category modules for theorem
    level, law-package boundaries, and hidden existence assumptions.
  - Split stable infinity category and derived-category dependencies from
    ordinary category foundations.
  - Mark any interface that cannot be materialized as public package evidence as a blocker.
- Deliverables:
  - Audit notes and corrected theorem-card levels.
- Acceptance criteria:
  - No downstream module can cite an interface as `L2` unless source-free
    evidence supports it.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.ModelCategory --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Infinity.StableInfinityCategory --verified-cache authoring`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `CATQ-001` | category theorem-card inventory | `L0` | `CAT-T00` |
| `CATQ-002` | small category law package | `L2` | `CAT-T01` |
| `CATQ-003` | functor composition laws | `L2` | `CAT-T02` |
| `CATQ-004` | natural transformation composition laws | `L2` | `CAT-T03` |
| `CATQ-005` | equivalence and isomorphism routes | `L2` or blocker split | `CAT-T04` |
| `CATQ-006` | finite universal constructions | `L2` | `CAT-T05` |
| `CATQ-007` | adjunction core | `L2` where prerequisites exist | `CAT-T06` |
| `CATQ-008` | Yoneda route package | split before source edits if universe support is absent | `CAT-T07` |

## Review Checklist

- Every theorem family has one primary owner and no downstream duplicate.
- Universe, quotient, choice, and extensionality assumptions are visible.
- Existing `Proofs.Ai.Category.*` modules are not treated as public beyond their
  audited theorem level.
- Universal-property theorems prove uniqueness instead of assuming it.
- Verification commands check the module being changed and reserve package
  gates for verifier or package work.
