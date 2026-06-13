# Algebraic Geometry Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T04`)
- `proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
- `proofs/homological-algebra-theorem-proof-roadmap-todo.md`
- `proofs/category-theory-theorem-proof-roadmap-todo.md`
- `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for algebraic geometry.
It is a planning sidecar only: it does not add trusted proof evidence,
axioms, source-free certificate verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers affine and projective varieties, coordinate rings,
schemes, morphisms, fiber products, base change, sheaves, quasi-coherent
modules, etale, smooth, and flat morphisms, cohomology route packages,
intersection-theory routes, Riemann-Roch-style dependency maps, moduli and
stack interfaces, derived algebraic geometry audits, arithmetic-geometry
aliases, and closure-boundary planning.

Out of scope for this task document:

- adding schemes, sheaves, Grothendieck topologies, stacks, or derived
  categories as trusted kernel primitives;
- using statement-only scheme, cohomology, or moduli interfaces as proof
  evidence for downstream arithmetic geometry;
- duplicating commutative algebra, category theory, homological algebra, or
  arithmetic geometry theorem ownership;
- hiding choice, quotient, universe, replacement, or cohomological
  boundedness assumptions inside theorem-shaped law packages;
- publicly materializing algebraic-geometry modules before closure audit, axiom-report
  review, and package verification are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing algebraic-geometry module trees include
  `Proofs.Ai.AlgebraicGeometry.CotangentComplex`,
  `Proofs.Ai.AlgebraicGeometry.DerivedAffineSchemes`,
  `Proofs.Ai.AlgebraicGeometry.DerivedCategory`,
  `Proofs.Ai.AlgebraicGeometry.EtaleSmoothFlatTopology`,
  `Proofs.Ai.AlgebraicGeometry.QuasiCoherentSheaves`,
  `Proofs.Ai.AlgebraicGeometry.SimplicialCommutativeRingCdga`, and
  `Proofs.Ai.AlgebraicGeometry.TorExt`.
- Commutative algebra owns rings, ideals, localization, spectra,
  Noetherian routes, and module algebra used by affine schemes.
- Category theory owns reusable universal-property, sheaf-shaped,
  indexed-category, and higher-category vocabulary.
- Homological algebra owns complexes, Ext, Tor, derived functors, and
  spectral-sequence prerequisites.
- Arithmetic geometry owns number-theoretic consequences, Galois
  representations, modularity, and Langlands-facing aliases.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `AG-00` inventory and namespace contract | `AG-T00` |
| `AG-01` affine varieties and coordinate rings | `AG-T01` |
| `AG-02` projective varieties and homogeneous coordinate rings | `AG-T02` |
| `AG-03` schemes, morphisms, and local data | `AG-T03` |
| `AG-04` fiber products and base change | `AG-T04` |
| `AG-05` sheaves and quasi-coherent modules | `AG-T05` |
| `AG-06` etale, smooth, and flat topology routes | `AG-T06` |
| `AG-07` cohomology and derived-functor routes | `AG-T07` |
| `AG-08` derived algebraic geometry audit | `AG-T08` |
| `AG-09` intersection theory and Riemann-Roch routes | `AG-T09` |
| `AG-10` moduli and stacks | `AG-T10` |
| `AG-11` arithmetic-geometry alias map | `AG-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `AG-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `AG-T01` through `AG-T04` | `L2` derived certificates from commutative-algebra and category prerequisites where possible |
| `AG-T05` through `AG-T07` | `L2` for structural lemmas; split cohomology existence routes before source edits |
| `AG-T08` through `AG-T11` | audit and dependency maps first unless explicit lower-level certificates exist |

## Milestones

### AG-T00 Build Algebraic Geometry Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory affine, projective, scheme, sheaf, cohomology, derived,
    intersection, moduli, and arithmetic-facing theorem families.
  - Assign primary homes across algebraic geometry, commutative algebra,
    category theory, homological algebra, and arithmetic geometry.
  - Mark choice, quotient, universe, and cohomological boundedness
    assumptions on theorem cards.
- Verification:
  - `rg -n "AG-T00|scheme|sheaf|cohomology|moduli" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AG-T01 Add Affine Variety And Coordinate Ring Core

- Status: Pending
- Depends on: `AG-T00`, `CMA-T01`
- Areas: `Proofs.Ai.AlgebraicGeometry.AffineVariety`
- Tasks:
  - Define affine algebraic sets, coordinate rings, polynomial maps, and
    basic morphism law packages over explicit commutative rings.
  - Prove elementary coordinate-ring functoriality and closed-set algebra
    facts without importing scheme-level assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.AffineVariety`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.AffineVariety --verified-cache authoring`

### AG-T02 Add Projective Variety Route

- Status: Pending
- Depends on: `AG-T01`, `CMA-T11`
- Areas: `Proofs.Ai.AlgebraicGeometry.ProjectiveVariety`
- Tasks:
  - Define homogeneous coordinate data, projective morphism route packages,
    and projective closure interfaces.
  - Split any Nullstellensatz or elimination dependency to commutative
    algebra before source edits.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.ProjectiveVariety`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### AG-T03 Add Scheme And Morphism Core

- Status: Pending
- Depends on: `AG-T01`, `CMA-T11`, `CAT-T05`
- Areas: `Proofs.Ai.AlgebraicGeometry.Scheme.Basic`
- Tasks:
  - Define scheme law packages using explicit local affine data, structure
    sheaf hooks, and morphism compatibility evidence.
  - Prove identity, composition, open immersion, and affine morphism
    projections where prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.Scheme.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.Scheme.Basic --verified-cache authoring`

### AG-T04 Add Fiber Product And Base Change Routes

- Status: Pending
- Depends on: `AG-T03`, `CAT-T05`
- Areas: `Proofs.Ai.AlgebraicGeometry.Scheme.FiberProduct`
- Tasks:
  - Package fiber products by universal properties rather than by trusted
    primitives.
  - Prove projection morphism and base-change compatibility lemmas where the
    scheme construction evidence is explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.Scheme.FiberProduct`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### AG-T05 Add Sheaf And Quasi-Coherent Module Audit

- Status: Pending
- Depends on: `AG-T03`, `CAT-T09`, `CMA-T06`
- Areas: `Proofs.Ai.AlgebraicGeometry.QuasiCoherentSheaves`
- Tasks:
  - Audit existing sheaf and quasi-coherent module declarations for primary
    ownership, statement level, and downstream imports.
  - Replace theorem-shaped shortcuts with derived certificates or blocker
    tasks before new reuse.
- Verification:
  - `rg -n "QuasiCoherent|sheaf|AG-T05|CAT-T09" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md proofs/category-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AG-T06 Add Etale, Smooth, And Flat Topology Routes

- Status: Pending
- Depends on: `AG-T03`, `AG-T04`, `CMA-T07`
- Areas: `Proofs.Ai.AlgebraicGeometry.EtaleSmoothFlatTopology`
- Tasks:
  - Audit etale, smooth, and flat route packages for explicit algebraic and
    topological prerequisites.
  - Split descent or site-level assumptions before source edits.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.AlgebraicGeometry.EtaleSmoothFlatTopology`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.AlgebraicGeometry.EtaleSmoothFlatTopology --verified-cache authoring`

### AG-T07 Add Cohomology And Derived-Functor Route

- Status: Pending
- Depends on: `AG-T05`, `HLA-T07`, `HLA-T10`
- Areas: `Proofs.Ai.AlgebraicGeometry.Cohomology`
- Tasks:
  - Define sheaf cohomology route packages that import homological algebra
    rather than asserting cohomology as a primitive.
  - Split vanishing, base-change, and spectral-sequence dependencies into
    prerequisite tasks when needed.
- Verification:
  - `rg -n "AG-T07|sheaf cohomology|spectral sequence|HLA-T10" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md proofs/homological-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AG-T08 Audit Derived Algebraic Geometry Interfaces

- Status: Pending
- Depends on: `AG-T07`, `CAT-T10`, `HLA-T10`
- Areas: `Proofs.Ai.AlgebraicGeometry.DerivedAffineSchemes`,
  `Proofs.Ai.AlgebraicGeometry.CotangentComplex`
- Tasks:
  - Audit derived affine schemes, cdga, simplicial commutative ring, and
    cotangent complex modules for hidden higher-category assumptions.
  - Keep derived interfaces as route packages until lower-level certificates
    exist.
- Verification:
  - `rg -n "DerivedAffine|Cotangent|cdga|AG-T08" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md proofs/Proofs/Ai/AlgebraicGeometry`
  - `git diff --check`

### AG-T09 Add Intersection Theory And Riemann-Roch Route

- Status: Pending
- Depends on: `AG-T04`, `AG-T07`
- Areas: `Proofs.Ai.AlgebraicGeometry.IntersectionTheory`
- Tasks:
  - Define cycle, divisor, Chow-style, and Chern-class route packages with
    exact prerequisites.
  - Split Riemann-Roch conclusions behind cohomology and characteristic-class
    evidence.
- Verification:
  - `rg -n "AG-T09|intersection|Riemann-Roch|Chern" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AG-T10 Add Moduli And Stack Interfaces

- Status: Pending
- Depends on: `AG-T06`, `CAT-T10`
- Areas: `Proofs.Ai.AlgebraicGeometry.Moduli`
- Tasks:
  - Record moduli functor, representability, stack, and descent interfaces as
    assumption-explicit route packages.
  - Do not treat representability or stack existence as derived without
    lower-level proof evidence.
- Verification:
  - `rg -n "AG-T10|moduli|stack|representability|descent" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AG-T11 Coordinate Arithmetic-Geometry Aliases

- Status: Pending
- Depends on: `AG-T03`, `AGL-T04`
- Areas: `Proofs.Ai.ArithmeticGeometry.*`
- Tasks:
  - Map etale cohomology, Galois representation, modularity, and arithmetic
    scheme aliases to their primary theorem homes.
  - Keep arithmetic consequences primary in the arithmetic-geometry roadmap.
- Verification:
  - `rg -n "AG-T11|AGL-T04|arithmetic geometry|Galois" proofs/algebraic-geometry-theorem-proof-roadmap-todo.md proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `AGQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `AG-T00` |
| `AGQ-002` | affine variety and coordinate-ring core | `L2` | `AG-T01` |
| `AGQ-003` | scheme and morphism core | `L2` where prerequisites exist | `AG-T03` |
| `AGQ-004` | sheaf and quasi-coherent audit | audit then `L2` upgrades | `AG-T05` |
| `AGQ-005` | cohomology dependency split | route package first | `AG-T07` |

## Review Checklist

- The theorem-card inventory has one primary owner for each algebraic-geometry
  theorem family.
- Commutative algebra, category theory, homological algebra, and arithmetic
  geometry dependencies are imports or aliases, not duplicate theorem homes.
- All choice, quotient, universe, replacement, and cohomology assumptions are
  visible in theorem statements or law packages.
- No new statement-only theorem is used as evidence for downstream work.
- Public package work is outside this TODO until closure audit confirms `L2` derived
  certificates and package verification is clean.
