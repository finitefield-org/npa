# Geometry Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T16`)
- `proofs/differential-geometry-theorem-proof-roadmap-todo.md`
- `proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
- `proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for geometry outside smooth
manifolds and algebraic geometry. It is a planning sidecar only: it does not
add trusted proof evidence, axioms, source-free certificate verdicts, or
package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers Euclidean, affine, projective, incidence, convex,
discrete, combinatorial, metric, symplectic-interface, finite, computational,
integral, and geometric-measure route planning, with explicit aliases to
differential geometry and algebraic geometry where those fields are primary.

Out of scope for this task document:

- adding points, lines, planes, manifolds, schemes, metrics, convex bodies, or
  symplectic forms as trusted kernel primitives;
- duplicating smooth manifold theorem ownership from differential geometry or
  scheme/variety theorem ownership from algebraic geometry;
- treating diagrammatic, incidence, convexity, or geometric-measure
  interfaces as proof evidence without certificates;
- hiding order, orientation, completeness, dimension, topology, measure,
  smoothness, or field assumptions;
- promoting geometry modules before closure audit and package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing geometry modules include abstract metric, affine, right-triangle,
  Pythagorean, and metric theorem routes.
- Differential geometry owns smooth manifolds, tangent and cotangent bundles,
  differential forms, Riemannian geometry, curvature, and smooth
  characteristic-class routes.
- Algebraic geometry owns varieties, schemes, sheaves, morphisms, and
  algebraic-geometric cohomology routes.
- Combinatorics owns finite geometry, designs, incidence systems, graph
  embeddings, and discrete-combinatorial geometry prerequisites.
- Linear algebra owns vector spaces, affine-linear facts, inner products,
  tensors, quadratic forms, and finite-dimensional spectral inputs.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `GEO-00` inventory and namespace contract | `GEO-T00` |
| `GEO-01` Euclidean and affine geometry core | `GEO-T01` |
| `GEO-02` projective and incidence geometry | `GEO-T02` |
| `GEO-03` convex geometry | `GEO-T03` |
| `GEO-04` discrete and combinatorial geometry | `GEO-T04` |
| `GEO-05` metric geometry | `GEO-T05` |
| `GEO-06` symplectic geometry interface | `GEO-T06` |
| `GEO-07` finite geometry and coding aliases | `GEO-T07` |
| `GEO-08` computational geometry route | `GEO-T08` |
| `GEO-09` integral and geometric measure bridge | `GEO-T09` |
| `GEO-10` smooth and algebraic geometry aliases | `GEO-T10` |
| `GEO-11` topology and physics bridge aliases | `GEO-T11` |
| `GEO-12` packaging and promotion | `GEO-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `GEO-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `GEO-T01` through `GEO-T05` | `L2` for finite, affine, metric, and convex structural certificates where prerequisites exist |
| `GEO-T06` through `GEO-T11` | alias maps or route packages unless lower-level proofs exist |
| `GEO-T12` | `L3` public closure and package verification |

## Milestones

### GEO-T00 Build Geometry Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/geometry-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory Euclidean, affine, projective, incidence, convex, discrete,
    metric, symplectic, finite, computational, and geometric-measure theorem
    families.
  - Assign primary homes across geometry, differential geometry, algebraic
    geometry, combinatorics, topology, and mathematical physics.
  - Mark dimension, field, order, orientation, topology, measure, smoothness,
    and completeness assumptions.
- Verification:
  - `rg -n "GEO-T00|Euclidean|affine|projective|convex|metric" proofs/geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T01 Add Euclidean And Affine Geometry Core

- Status: Pending
- Depends on: `LIN-T22`
- Areas: `Proofs.Ai.Geometry.Affine`
- Tasks:
  - Audit existing affine, right-triangle, Pythagorean, and metric modules for
    theorem ownership and statement level.
  - Add derived affine incidence, parallelism, barycentric, and Euclidean
    distance lemmas where linear-algebra prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Affine`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Affine --verified-cache authoring`

### GEO-T02 Add Projective And Incidence Geometry Route

- Status: Pending
- Depends on: `GEO-T01`, `LIN-T22`
- Areas: `Proofs.Ai.Geometry.Projective`
- Tasks:
  - Define projective spaces, incidence, duality, cross ratios, and finite
    incidence route packages with explicit field assumptions.
  - Split Desargues, Pappus, and coordinatization prerequisites before source
    edits.
- Verification:
  - `rg -n "GEO-T02|projective|incidence|Desargues|Pappus" proofs/geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T03 Add Convex Geometry Route

- Status: Pending
- Depends on: `ANA-T37`, `OPT-T01`
- Areas: `Proofs.Ai.Geometry.Convex`
- Tasks:
  - Define convex sets, convex hulls, separation, supporting hyperplanes,
    polytopes, and volume route packages.
  - Coordinate analytic convexity and optimization duality with their primary
    roadmaps.
- Verification:
  - `rg -n "GEO-T03|convex|separation|supporting hyperplane|polytope" proofs/geometry-theorem-proof-roadmap-todo.md proofs/optimization-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T04 Add Discrete And Combinatorial Geometry Route

- Status: Pending
- Depends on: `CG-T36`
- Areas: `Proofs.Ai.Geometry.Discrete`
- Tasks:
  - Map incidence configurations, finite point-line systems, arrangements,
    geometric graphs, and extremal geometry to combinatorics ownership where
    appropriate.
  - Prove finite incidence consequences only when counting prerequisites are
    explicit.
- Verification:
  - `rg -n "GEO-T04|CG-T36|finite geometry|incidence|arrangement" proofs/geometry-theorem-proof-roadmap-todo.md proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T05 Add Metric Geometry Route

- Status: Pending
- Depends on: `TOP-T05`
- Areas: `Proofs.Ai.Geometry.Metric`
- Tasks:
  - Define geodesic spaces, length spaces, curvature-bound interfaces,
    Gromov-Hausdorff route packages, and metric embedding aliases.
  - Keep completeness, compactness, and curvature assumptions explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Metric`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### GEO-T06 Add Symplectic Geometry Interface

- Status: Pending
- Depends on: `DG-T05`, `MP-T03`
- Areas: `Proofs.Ai.Geometry.Symplectic`
- Tasks:
  - Define symplectic forms, Poisson brackets, Hamiltonian vector fields, and
    moment-map route packages as aliases to differential geometry and
    mathematical physics where needed.
  - Do not use physical Hamiltonian postulates as mathematical proof evidence.
- Verification:
  - `rg -n "GEO-T06|DG-T05|MP-T03|symplectic|Poisson" proofs/geometry-theorem-proof-roadmap-todo.md proofs/differential-geometry-theorem-proof-roadmap-todo.md proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T07 Coordinate Finite Geometry And Coding Aliases

- Status: Pending
- Depends on: `GEO-T02`, `CC-T02`
- Areas: `Proofs.Ai.Combinatorics.FiniteGeometry.*`
- Tasks:
  - Map finite projective planes, designs, incidence codes, and finite-field
    geometry to combinatorics and coding ownership.
  - Keep coding-theory bounds primary in the coding roadmap.
- Verification:
  - `rg -n "GEO-T07|CC-T02|finite geometry|coding" proofs/geometry-theorem-proof-roadmap-todo.md proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T08 Add Computational Geometry Route

- Status: Pending
- Depends on: `GEO-T03`, `TCS-T06`
- Areas: `Proofs.Ai.Geometry.Computational`
- Tasks:
  - Define orientation tests, convex hull algorithms, triangulation, Voronoi,
    Delaunay, and arrangement algorithm route packages.
  - Coordinate trace correctness and complexity with TCS.
- Verification:
  - `rg -n "GEO-T08|TCS-T06|convex hull|Voronoi|Delaunay" proofs/geometry-theorem-proof-roadmap-todo.md proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T09 Add Integral And Geometric Measure Bridge

- Status: Pending
- Depends on: `MEA-T30`, `GEO-T03`
- Areas: `Proofs.Ai.Geometry.Measure`
- Tasks:
  - Record Hausdorff measure, rectifiability, area formula, and geometric
    measure route packages with explicit measure prerequisites.
  - Split analytic regularity assumptions before source edits.
- Verification:
  - `rg -n "GEO-T09|Hausdorff measure|rectifiable|area formula|MEA-T30" proofs/geometry-theorem-proof-roadmap-todo.md proofs/measure-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T10 Coordinate Smooth And Algebraic Geometry Aliases

- Status: Pending
- Depends on: `DG-T01`, `AG-T03`
- Areas: `proofs/differential-geometry-theorem-proof-roadmap-todo.md`,
  `proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map smooth manifold, Riemannian, algebraic variety, and scheme theorem
    families to their primary roadmaps.
  - Add geometry aliases only when they prevent duplicate proof targets.
- Verification:
  - `rg -n "GEO-T10|DG-T01|AG-T03|smooth manifold|scheme" proofs/geometry-theorem-proof-roadmap-todo.md proofs/differential-geometry-theorem-proof-roadmap-todo.md proofs/algebraic-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T11 Coordinate Topology And Physics Bridge Aliases

- Status: Pending
- Depends on: `AT-T08`, `MP-T03`
- Areas: `proofs/algebraic-topology-theorem-proof-roadmap-todo.md`,
  `proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map characteristic classes, symplectic interfaces, Hamiltonian routes, and
    gauge-facing geometry to primary owners.
  - Keep physical postulates outside mathematical theorem evidence.
- Verification:
  - `rg -n "GEO-T11|AT-T08|MP-T03|Hamiltonian|gauge" proofs/geometry-theorem-proof-roadmap-todo.md proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### GEO-T12 Promote Stable Geometry Closures

- Status: Pending
- Depends on: selected stable `GEO-T01` through `GEO-T11` batches
- Areas: `npa-mathlib` promotion candidates
- Tasks:
  - Promote only closure-audited `L2` affine, metric, convex, finite, or
    incidence theorem closures.
  - Keep smooth, algebraic, geometric-measure, and physics-facing route
    packages out of public materialization until proven.
- Verification:
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `GEOQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `GEO-T00` |
| `GEOQ-002` | affine and Euclidean core audit | `L2` where prerequisites exist | `GEO-T01` |
| `GEOQ-003` | projective and incidence route | route package or `L2` finite facts | `GEO-T02` |
| `GEOQ-004` | convex geometry ownership split | alias and route split | `GEO-T03` |
| `GEOQ-005` | smooth and algebraic alias map | alias and dependency split | `GEO-T10` |

## Review Checklist

- Smooth manifold and scheme theorem families remain primary in their
  dedicated roadmaps.
- Linear algebra, combinatorics, topology, measure, and physics dependencies
  are imports or aliases rather than duplicate theorem homes.
- Dimension, field, order, orientation, topology, measure, smoothness, and
  completeness assumptions are visible.
- Physical or diagrammatic interfaces are not treated as proof evidence.
- Promotion is deferred until closure audit confirms stable `L2` derived
  certificates.
