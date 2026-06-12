# Differential Geometry Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T05`, `BMQ-004`)
- `proofs/topology-theorem-proof-roadmap-todo.md`
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/measure-theory-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for differential geometry and
smooth manifolds. It is a planning sidecar only: it does not add trusted proof
evidence, axioms, source-free certificate verdicts, or package verification
claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers smooth manifolds, charts, atlases, smooth maps,
tangent and cotangent bundles, vector fields, differential forms, pullbacks,
Lie derivatives, flows, integration on manifolds, Stokes route packages,
de Rham cohomology, Riemannian metrics, Levi-Civita connection, geodesics,
curvature, Gauss-Bonnet-style routes, fiber bundles, characteristic classes,
and promotion planning.

Out of scope for this task document:

- adding manifolds, tangent spaces, differential forms, integration, or
  curvature as trusted kernel primitives;
- assuming smooth partitions of unity, existence of flows, Stokes theorem, or
  de Rham theorem as statement-only shortcuts;
- duplicating general topology, measure/integration, linear algebra, or
  analysis derivative foundations;
- promoting unstable differential-geometry modules before closure audit and
  package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Differential.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Topology already has a topological manifold module:
  `Proofs.Ai.Topology.Manifold.Topological`.
- Existing reusable analysis modules include abstract derivative, inverse
  function, implicit function, fixed point, normed-space, metric-topology, and
  linear-map routes.
- Existing geometry modules are mostly metric, affine, and right-triangle
  oriented; they do not own smooth-manifold structure.
- Measure theory owns integration, Radon, change-of-variables, covering, and
  geometric measure routes. Differential geometry must import those
  prerequisites for integration on manifolds.
- Homological algebra and topology should own generic cohomology machinery;
  this todo owns de Rham-specific differential-form routes.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `DG-00` inventory and namespace contract | `DG-T00` |
| `DG-01` smooth manifolds, charts, and atlases | `DG-T01` |
| `DG-02` smooth maps and local diffeomorphisms | `DG-T02` |
| `DG-03` tangent and cotangent bundles | `DG-T03` |
| `DG-04` vector fields, flows, and Lie brackets | `DG-T04` |
| `DG-05` differential forms and exterior derivative | `DG-T05` |
| `DG-06` orientation, integration, and Stokes route | `DG-T06` |
| `DG-07` de Rham cohomology | `DG-T07` |
| `DG-08` Riemannian metrics and connections | `DG-T08` |
| `DG-09` curvature, geodesics, and comparison routes | `DG-T09` |
| `DG-10` bundles and characteristic classes | `DG-T10` |
| `DG-11` packaging and promotion | `DG-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `DG-T00` | `L0` planning and theorem-card inventory |
| `DG-T01` through `DG-T03` | `L2` for explicit chart, smooth-map, and tangent-bundle law projections |
| `DG-T04` through `DG-T07` | split before source edits unless analytic existence and integration prerequisites are visible |
| `DG-T08` through `DG-T10` | `L2` for algebraic/local structural lemmas; route packages for global existence theorems |
| `DG-T11` | `L3` public closure and package verification |

## Milestones

### DG-T00 Build Differential Geometry Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/differential-geometry-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory smooth manifold, tangent, cotangent, vector field, form,
    integration, de Rham, Riemannian, curvature, bundle, and characteristic
    class theorem families.
  - Assign primary homes across topology, analysis, measure theory,
    homological algebra, and geometry.
  - Record target levels and construction blockers.
- Deliverables:
  - Differential geometry theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Smooth manifold facts do not duplicate topological manifold ownership.
  - Integration and cohomology dependencies point to their owners.
- Verification:
  - `rg -n "DG-T00|smooth|tangent|Stokes|Riemannian" proofs/differential-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### DG-T01 Add Smooth Manifold And Atlas Core

- Status: Pending
- Depends on: `TOP-T31`, `ANA-T19`
- Areas: `Proofs.Ai.Geometry.Differential.Manifold`
- Tasks:
  - Define smooth charts, atlases, compatibility, smooth manifold law packages,
    and chart transition functions.
  - Import topological manifold and finite-dimensional analysis prerequisites.
  - Prove elementary chart compatibility and restriction lemmas.
- Deliverables:
  - Smooth manifold core module.
- Acceptance criteria:
  - A smooth structure is explicit and is not inferred from topological
    manifold data alone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.Manifold`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Differential.Manifold --verified-cache authoring`

### DG-T02 Add Smooth Maps And Local Diffeomorphism Route

- Status: Pending
- Depends on: `DG-T01`, `ANA-T13`
- Areas: `Proofs.Ai.Geometry.Differential.SmoothMap`
- Tasks:
  - Define smooth maps, smooth equivalence, immersions, submersions,
    embeddings, and local diffeomorphism route packages.
  - Import inverse and implicit function prerequisites for local theorems.
  - Split global inverse routes behind topology assumptions.
- Deliverables:
  - Smooth map module and local diffeomorphism route map.
- Acceptance criteria:
  - Local inverse statements cite analysis inverse-function evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.SmoothMap`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### DG-T03 Add Tangent And Cotangent Bundle Core

- Status: Pending
- Depends on: `DG-T02`, `LIN-T05`
- Areas: `Proofs.Ai.Geometry.Differential.Tangent`
- Tasks:
  - Define tangent vectors through chart or derivation route packages.
  - Add differential, tangent bundle, cotangent bundle, and pullback maps.
  - Prove functoriality of differentials where smooth-map composition exists.
- Deliverables:
  - Tangent and cotangent core module.
- Acceptance criteria:
  - Coordinate-change assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.Tangent`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Differential.Tangent --verified-cache authoring`

### DG-T04 Add Vector Fields, Lie Brackets, And Flow Route

- Status: Pending
- Depends on: `DG-T03`, `ANA-T33`
- Areas: `Proofs.Ai.Geometry.Differential.VectorField`
- Tasks:
  - Define vector fields, derivations, Lie brackets, integral curves, and
    flow route packages.
  - Prove algebraic Lie bracket identities where prerequisites exist.
  - Split flow existence and uniqueness behind ODE prerequisites.
- Deliverables:
  - Vector field route module.
- Acceptance criteria:
  - Flow theorems do not appear before ODE existence assumptions are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.VectorField`
  - `rg -n "Lie bracket|flow|ODE" proofs/differential-geometry-theorem-proof-roadmap-todo.md`

### DG-T05 Add Differential Forms And Exterior Derivative Core

- Status: Pending
- Depends on: `DG-T03`, `LIN-T38`
- Areas: `Proofs.Ai.Geometry.Differential.Form`
- Tasks:
  - Define alternating forms, wedge products, pullback of forms, exterior
    derivative, closed forms, and exact forms.
  - Prove algebraic wedge and pullback laws where exterior algebra exists.
  - Split Poincare lemma route behind local contractibility prerequisites.
- Deliverables:
  - Differential form core module.
- Acceptance criteria:
  - Exterior algebra facts are imported or explicitly proved, not assumed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.Form`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### DG-T06 Add Orientation, Integration, And Stokes Route

- Status: Pending
- Depends on: `DG-T05`, `MEA-T30`
- Areas: `Proofs.Ai.Geometry.Differential.Integration`
- Tasks:
  - Define orientation, manifolds with boundary, compact support, integration
    of forms, and Stokes theorem route packages.
  - Import measure/change-of-variables prerequisites.
  - Split Stokes, divergence theorem, and Green theorem aliases.
- Deliverables:
  - Integration route module and dependency map.
- Acceptance criteria:
  - Stokes theorem is not accepted without explicit integration and boundary
    prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.Integration`
  - `rg -n "Stokes|orientation|boundary|change of variables" proofs/differential-geometry-theorem-proof-roadmap-todo.md`

### DG-T07 Add De Rham Route

- Status: Pending
- Depends on: `DG-T05`, `DG-T06`, `HLA-T03`
- Areas: `Proofs.Ai.Geometry.Differential.DeRham`
- Tasks:
  - Define de Rham complex, de Rham cohomology, pullback on cohomology, and
    de Rham theorem route packages.
  - Coordinate generic cohomology with homological algebra.
  - Split Mayer-Vietoris and Poincare lemma prerequisites.
- Deliverables:
  - De Rham dependency module.
- Acceptance criteria:
  - De Rham theorem is route-packaged until homological and sheaf
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.DeRham`
  - `rg -n "de Rham|Poincare|Mayer" proofs/differential-geometry-theorem-proof-roadmap-todo.md`

### DG-T08 Add Riemannian Metric And Connection Core

- Status: Pending
- Depends on: `DG-T03`, `DG-T04`, `LIN-T26`
- Areas: `Proofs.Ai.Geometry.Differential.Riemannian`
- Tasks:
  - Define Riemannian metrics, musical isomorphism route packages,
    Levi-Civita connection, covariant derivative, and metric compatibility.
  - Prove local algebraic consequences from explicit metric and connection
    laws.
  - Split Levi-Civita existence and uniqueness if prerequisites are absent.
- Deliverables:
  - Riemannian metric and connection route module.
- Acceptance criteria:
  - Levi-Civita theorems state torsion-free and metric-compatible
    assumptions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Geometry.Differential.Riemannian`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### DG-T09 Add Curvature And Geodesic Route

- Status: Pending
- Depends on: `DG-T08`, `ANA-T33`
- Areas: `Proofs.Ai.Geometry.Differential.Curvature`
- Tasks:
  - Define curvature tensor, Ricci curvature, scalar curvature, sectional
    curvature, geodesics, exponential map, and comparison route packages.
  - Split geodesic existence behind ODE prerequisites.
  - Add Gauss-Bonnet theorem route with topological and integration
    dependencies.
- Deliverables:
  - Curvature and geodesic dependency map.
- Acceptance criteria:
  - Global comparison theorems identify completeness, compactness, and
    curvature-bound assumptions.
- Verification:
  - `rg -n "curvature|geodesic|Gauss-Bonnet|exponential map" proofs/differential-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### DG-T10 Add Bundle And Characteristic Class Route

- Status: Pending
- Depends on: `DG-T07`, `CAT-T05`, `HLA-T09`
- Areas: `Proofs.Ai.Geometry.Differential.Bundle`
- Tasks:
  - Define vector bundles, principal bundles, connections on bundles, Chern
    classes, Pontryagin classes, and Euler class route packages.
  - Coordinate characteristic classes with algebraic topology.
  - Split Chern-Weil theory behind differential-form and cohomology
    prerequisites.
- Deliverables:
  - Bundle and characteristic-class route module.
- Acceptance criteria:
  - Characteristic-class theorems state cohomology and bundle assumptions
    explicitly.
- Verification:
  - `rg -n "bundle|Chern|Pontryagin|Euler class|Chern-Weil" proofs/differential-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### DG-T11 Promote Stable Differential Geometry Closures

- Status: Pending
- Depends on: selected stable `DG-T01` through `DG-T10` batches
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`,
  `proofs/generated/*`
- Tasks:
  - Run closure audits for stable differential-geometry modules.
  - Update public package metadata only at promotion.
  - Record excluded global analysis, Stokes, and characteristic-class routes.
- Deliverables:
  - Verified differential-geometry closure ready for `npa-mathlib` promotion.
- Acceptance criteria:
  - Axiom reports and package checks are clean for the promoted closure.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `DGQ-001` | theorem-card inventory | `L0` | `DG-T00` |
| `DGQ-002` | smooth atlas core | `L2` where topology and analysis prerequisites exist | `DG-T01` |
| `DGQ-003` | smooth map and local diffeomorphism route | `L2` or blocker split | `DG-T02` |
| `DGQ-004` | tangent bundle core | `L2` | `DG-T03` |
| `DGQ-005` | vector field and Lie bracket route | `L2` for algebraic bracket facts | `DG-T04` |
| `DGQ-006` | differential forms core | `L2` where exterior algebra exists | `DG-T05` |
| `DGQ-007` | Stokes dependency split | route package first | `DG-T06` |
| `DGQ-008` | Riemannian metric route | `L2` for local algebraic consequences | `DG-T08` |

## Review Checklist

- Topological, analytical, measure-theoretic, and homological dependencies are
  imported from their owners.
- Smooth, integration, flow, and global-existence assumptions are visible.
- Stokes, de Rham, and Gauss-Bonnet are route packages until prerequisites are
  verified.
- Verification commands stay local until promotion or package metadata changes.
