# Homological Algebra Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T02`, `BMQ-003`)
- `proofs/category-theory-theorem-proof-roadmap-todo.md`
- `proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
- `proofs/topology-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for homological algebra. It is
a planning sidecar only: it does not add trusted proof evidence, axioms,
source-free certificate verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers additive and abelian category prerequisites, complexes,
chain maps, homotopies, homology, exact sequences, diagram lemmas, projective
and injective objects, resolutions, derived functors, Ext and Tor,
spectral-sequence route packages, derived categories, triangulated categories,
and promotion planning.

Out of scope for this task document:

- adding exactness, quotient categories, derived categories, or homology as
  trusted kernel primitives;
- assuming abelian-category existence theorems before category and
  commutative-algebra prerequisites are explicit;
- using a derived-category interface as proof evidence for Ext, Tor, or sheaf
  cohomology without source-free verification;
- promoting unstable homological modules before closure audit and package
  checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.HomologicalAlgebra.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing related modules include
  `Proofs.Ai.AlgebraicGeometry.TorExt`,
  `Proofs.Ai.AlgebraicGeometry.DerivedCategory`,
  `Proofs.Ai.Topology.Homology.Computation`, and category model/stable
  infinity category interfaces.
- Category theory should own generic functor, naturality, adjunction, limit,
  colimit, and equivalence vocabulary.
- Commutative algebra should own module, tensor, exactness, projective,
  injective, and localization foundations when the theorem is module-specific.
- Algebraic topology should own topological consequences; this todo owns the
  chain-complex and homological algebra machinery those modules may import.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `HLA-00` inventory and namespace contract | `HLA-T00` |
| `HLA-01` additive and abelian category prerequisites | `HLA-T01` |
| `HLA-02` chain complexes and chain maps | `HLA-T02` |
| `HLA-03` homotopies and homology | `HLA-T03` |
| `HLA-04` exact sequences and diagram lemmas | `HLA-T04` |
| `HLA-05` projective, injective, and resolutions | `HLA-T05` |
| `HLA-06` derived functors | `HLA-T06` |
| `HLA-07` Ext and Tor | `HLA-T07` |
| `HLA-08` long exact sequences | `HLA-T08` |
| `HLA-09` spectral sequences | `HLA-T09` |
| `HLA-10` derived and triangulated categories | `HLA-T10` |
| `HLA-11` packaging and promotion | `HLA-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `HLA-T00` | `L0` planning and theorem-card inventory |
| `HLA-T01` through `HLA-T04` | `L2` for explicit algebraic exactness and chain-complex lemmas |
| `HLA-T05` through `HLA-T08` | `L2` where module and category prerequisites exist; otherwise split blockers before source edits |
| `HLA-T09`, `HLA-T10` | route packages first; no theorem-shaped assumption as final evidence |
| `HLA-T11` | `L3` public closure and package verification |

## Milestones

### HLA-T00 Build Homological Algebra Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/homological-algebra-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory exactness, complex, homology, resolution, Ext, Tor, derived
    functor, spectral-sequence, and derived-category theorem families.
  - Assign primary homes across category theory, commutative algebra,
    topology, and algebraic geometry.
  - Record construction-heavy blockers and target levels.
- Deliverables:
  - Homological algebra theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Topological homology and algebraic Ext/Tor routes have clear owners.
- Verification:
  - `rg -n "HLA-T00|Ext|Tor|exact|derived|spectral" proofs/homological-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### HLA-T01 Add Additive And Abelian Category Route

- Status: Pending
- Depends on: `CAT-T03`, `CMA-T03`
- Areas: `Proofs.Ai.HomologicalAlgebra.AbelianCategory`
- Tasks:
  - Define additive category, zero object, biproduct, kernel, cokernel, image,
    coimage, and abelian-category route packages.
  - Prove elementary exactness projections only from explicit assumptions.
  - Split abelian-category existence assumptions from module-category facts.
- Deliverables:
  - Additive and abelian category route module.
- Acceptance criteria:
  - Abelian-category laws are explicit; they are not inferred from a bare
    category interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.AbelianCategory`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### HLA-T02 Add Chain Complex Core

- Status: Pending
- Depends on: `HLA-T01`
- Areas: `Proofs.Ai.HomologicalAlgebra.ChainComplex`
- Tasks:
  - Define chain complexes, cochain complexes, differentials, chain maps, and
    composition.
  - Prove identity and composition laws for chain maps.
  - Add degree-shift routes with explicit index assumptions.
- Deliverables:
  - Chain complex core module.
- Acceptance criteria:
  - The condition `d^2 = 0` is an explicit field of the law package and is
    used in homology proofs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.ChainComplex`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.HomologicalAlgebra.ChainComplex --verified-cache authoring`

### HLA-T03 Add Homology And Chain Homotopy Core

- Status: Pending
- Depends on: `HLA-T02`
- Areas: `Proofs.Ai.HomologicalAlgebra.Homology`
- Tasks:
  - Define cycles, boundaries, homology object route packages, and chain
    homotopies.
  - Prove boundary subset cycle and homotopic maps induce the same homology
    map where quotient prerequisites are available.
  - Split quotient-object construction blockers if needed.
- Deliverables:
  - Homology core module.
- Acceptance criteria:
  - Quotient assumptions for homology objects are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.Homology`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### HLA-T04 Add Exact Sequence And Diagram Lemma Route

- Status: Pending
- Depends on: `HLA-T03`
- Areas: `Proofs.Ai.HomologicalAlgebra.Exact`
- Tasks:
  - Define exactness at an object, short exact sequences, split exact
    sequences, and diagram morphisms.
  - Prove elementary exactness transport and splitting lemmas.
  - Add snake, five, nine, and 3x3 lemma route packages.
- Deliverables:
  - Exact sequence core module and diagram lemma route map.
- Acceptance criteria:
  - Diagram lemmas list kernel, cokernel, image, and quotient prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.Exact`
  - `rg -n "snake|five lemma|short exact|diagram" proofs/homological-algebra-theorem-proof-roadmap-todo.md`

### HLA-T05 Add Projective, Injective, And Resolution Routes

- Status: Pending
- Depends on: `HLA-T04`, `CMA-T08`
- Areas: `Proofs.Ai.HomologicalAlgebra.Resolution`
- Tasks:
  - Define projective and injective objects, resolutions, quasi-isomorphisms,
    and lifting properties.
  - Split existence of enough projectives or injectives behind explicit
    assumptions.
  - Add comparison theorem route packages.
- Deliverables:
  - Resolution route module.
- Acceptance criteria:
  - Existence of enough projectives or injectives is never hidden.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.Resolution`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### HLA-T06 Add Derived Functor Route

- Status: Pending
- Depends on: `HLA-T05`, `CAT-T06`
- Areas: `Proofs.Ai.HomologicalAlgebra.DerivedFunctor`
- Tasks:
  - Define left and right derived functor route packages via resolutions.
  - State independence-of-resolution prerequisites.
  - Coordinate adjunction-derived routes with category theory.
- Deliverables:
  - Derived functor dependency module.
- Acceptance criteria:
  - A derived functor theorem cannot cite a resolution interface as the final
    proof of independence unless verified.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.DerivedFunctor`
  - `rg -n "derived functor|resolution|independence" proofs/homological-algebra-theorem-proof-roadmap-todo.md`

### HLA-T07 Add Ext And Tor Route

- Status: Pending
- Depends on: `HLA-T06`, `CMA-T08`
- Areas: `Proofs.Ai.HomologicalAlgebra.ExtTor`
- Tasks:
  - Define Ext and Tor through derived Hom and tensor routes.
  - Connect low-degree Ext/Tor facts to exactness and tensor prerequisites.
  - Audit existing `Proofs.Ai.AlgebraicGeometry.TorExt`.
- Deliverables:
  - Ext/Tor route module and migration notes for downstream aliases.
- Acceptance criteria:
  - Ext/Tor primary theorem names live here; algebraic geometry may alias
    application-specific consequences.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.ExtTor`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.TorExt --verified-cache authoring`

### HLA-T08 Add Long Exact Sequence Route

- Status: Pending
- Depends on: `HLA-T04`, `HLA-T07`
- Areas: `Proofs.Ai.HomologicalAlgebra.LongExact`
- Tasks:
  - Define connecting morphism route packages for short exact sequences of
    complexes.
  - Prove exactness at low-degree positions where prerequisites are complete.
  - Split cohomology long exact sequences from homology routes.
- Deliverables:
  - Long exact sequence route module.
- Acceptance criteria:
  - Connecting morphism construction assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.HomologicalAlgebra.LongExact`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### HLA-T09 Add Spectral Sequence Route

- Status: Pending
- Depends on: `HLA-T08`
- Areas: `Proofs.Ai.HomologicalAlgebra.SpectralSequence`
- Tasks:
  - Define filtered complex, double complex, pages, differentials, and
    convergence route packages.
  - Split Grothendieck and Leray spectral sequence routes by prerequisites.
  - Avoid using convergence as a hidden assumption.
- Deliverables:
  - Spectral sequence dependency map and route module.
- Acceptance criteria:
  - Every spectral-sequence theorem identifies filtration and convergence
    assumptions.
- Verification:
  - `rg -n "spectral sequence|filtered|double complex|convergence" proofs/homological-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### HLA-T10 Audit Derived And Triangulated Category Interfaces

- Status: Pending
- Depends on: `HLA-T05`, `CAT-T10`
- Areas: `Proofs.Ai.AlgebraicGeometry.DerivedCategory`,
  `Proofs.Ai.Category.Infinity.StableInfinityCategory`
- Tasks:
  - Audit existing derived-category and stable-category modules for theorem
    level, quotient boundaries, and localization assumptions.
  - Split triangulated category axioms, Verdier localization, and derived
    category construction.
  - Mark non-promotable interfaces clearly.
- Deliverables:
  - Audit notes and corrected theorem-card levels.
- Acceptance criteria:
  - Derived-category interfaces are not cited as `L2` construction evidence
    unless their closure is verified.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.DerivedCategory --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Category.Infinity.StableInfinityCategory --verified-cache authoring`

### HLA-T11 Promote Stable Homological Algebra Closures

- Status: Pending
- Depends on: selected stable `HLA-T01` through `HLA-T10` batches
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`,
  `proofs/generated/*`
- Tasks:
  - Run closure audits for stable homological-algebra modules.
  - Update public package metadata only at promotion.
  - Record excluded spectral-sequence and derived-category routes.
- Deliverables:
  - Verified homological-algebra closure ready for `npa-mathlib` promotion.
- Acceptance criteria:
  - Axiom reports and package checks are clean for the promoted closure.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `HLAQ-001` | theorem-card inventory | `L0` | `HLA-T00` |
| `HLAQ-002` | additive and abelian category route | `L2` for explicit law projections | `HLA-T01` |
| `HLAQ-003` | chain complex core | `L2` | `HLA-T02` |
| `HLAQ-004` | homology route with quotient boundary | `L2` or blocker split | `HLA-T03` |
| `HLAQ-005` | exact sequence core | `L2` | `HLA-T04` |
| `HLAQ-006` | resolution route | split before source edits if existence assumptions are absent | `HLA-T05` |
| `HLAQ-007` | Ext/Tor ownership audit | `L2` only for verified low-degree facts | `HLA-T07` |
| `HLAQ-008` | derived-category interface audit | audit before promotion | `HLA-T10` |

## Review Checklist

- Category, commutative algebra, topology, and algebraic geometry owners are
  distinct.
- Quotient, exactness, projective/injective existence, and convergence
  assumptions are visible.
- Spectral sequences and derived categories are route packages until their
  construction evidence is verified.
- Verification commands stay local until promotion or package metadata changes.
