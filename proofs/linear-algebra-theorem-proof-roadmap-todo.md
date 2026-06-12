# Linear Algebra Theorem Proof Roadmap Todo

Source: `proofs/linear-algebra-theorem-proof-roadmap.md`

This document decomposes the linear algebra theorem proof roadmap into concrete
authoring milestones. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, or certificate validity assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, and AI output
are untrusted.

---

## Scope

This task list covers theorem-card inventory, finite-dimensional vector-space
foundations, matrix and determinant APIs, rank and eigenvalue theory,
inner-product and spectral theorem routes, decompositions, SVD, quadratic
forms, duality, least squares, perturbation, graph, numerical, and
optimization linear algebra.

The list intentionally does not prove the roadmap in one pass. Later agents
should implement exactly one milestone or a clearly bounded contiguous batch.
When prerequisites are absent, agents should split explicit blocker or prerequisite tasks before source edits. Statement-only interfaces are not acceptable proof artifacts for pending theorem work.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding vector spaces, matrices, determinants, ranks, decompositions, or
  numerical algorithms as trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- treating theorem-search sidecars, AI indexes, replay files, or generated
  docs as trusted evidence;
- promoting unstable linear algebra modules into `npa-mathlib` before local
  closure, axiom-report, and package verification checks are clean.

## Authoring Loop

For ordinary linear algebra theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Reserve `check-corpus-package.sh` or
`check-corpus-full.sh` for package-wide verifier behavior, publish-plan or
package metadata updates, certificate/checker compatibility, release work, or
promotion into a high-trust closure.

## Current Implementation Facts

- Existing reusable algebra and scalar foundations include
  `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractField`,
  `Proofs.Ai.Algebra.AbstractOrderedField`, and
  `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`.
- Existing reusable vector foundations include `Proofs.Ai.Vector.Basic`,
  `Proofs.Ai.Vector.Dot`, `Proofs.Ai.Vector.AbstractSpace`,
  `Proofs.Ai.Vector.AbstractInnerProduct`, and
  `Proofs.Ai.Vector.AbstractInnerProductDerive`.
- Existing reusable analysis foundations include
  `Proofs.Ai.Analysis.AbstractMetricTopology`,
  `Proofs.Ai.Analysis.AbstractNormedSpace`,
  `Proofs.Ai.Analysis.AbstractLinearMap`,
  `Proofs.Ai.Analysis.AbstractDerivative`,
  `Proofs.Ai.Analysis.AbstractFixedPoint`,
  `Proofs.Ai.Analysis.AbstractInverseFunction`,
  `Proofs.Ai.Analysis.AbstractImplicitPhi`, and
  `Proofs.Ai.Analysis.AbstractImplicitFunction`.
- Checked dedicated `Proofs.Ai.LinearAlgebra.*` corpus modules now include
  `VectorSpace.Basic`, `Subspace.Basic`, `Basis.Dimension`,
  `LinearMap.Basic`, matrix basic / representation / elimination /
  determinant / adjugate / rank / positive-definite modules, eigen and
  diagonalization modules, characteristic / Cayley-Hamilton modules,
  inner-product / Gram / orthonormal / projection modules, and
  `Spectral.SelfAdjoint`.
- `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem` exists
  as an abstract Hilbert-space spectral theorem interface.
- Geometric right-triangle Pythagorean theorem names are already checked in
  `Proofs.Ai.Geometry.Pythagorean`; this roadmap only owns the inner-product
  norm-square and perpendicular norm-addition aliases.
- Checked dedicated `Proofs.Ai.LinearAlgebra.*` corpus modules now also include
  quotient, linear-map isomorphism, rank-factorization, canonical-form,
  orthogonal/polar, decomposition, SVD, Moore-Penrose, low-rank, form, tensor,
  dual, least-squares, nonnegative, norm, perturbation, matrix-function,
  matrix-equation, matrix-group, Lie, representation, numerical, graph, and
  optimization route modules.
- Public `npa-mathlib` has already materialized geometry, vector,
  metric-topology, normed-space, linear-map, derivative, fixed-point, inverse,
  and implicit-function closures through `npa-mathlib v0.1.27`.
- `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` is standalone and still
  needs field/matrix namespace audit before public linear-algebra promotion.
- Promotion and package metadata generation are intentionally out of scope for
  the 2026-06-13 proof-corpus authoring pass.
- Statistical regression and Gauss-Markov theorem families should coordinate
  with statistics roadmap `STAT-15`, especially `STAT-T43` through
  `STAT-T46`.
- Convex, KKT, and variational optimization theorem families should coordinate
  with analysis roadmap `ANA-14` and task milestone `ANA-T37`.

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `LIN-00` inventory and statement policy | `LIN-T00` |
| `LIN-01` vector-space and subspace foundations | `LIN-T01` through `LIN-T02` |
| `LIN-02` bases, dimension, quotients, and direct sums | `LIN-T03` through `LIN-T04` |
| `LIN-03` linear maps, kernels, images, and isomorphism theorems | `LIN-T05` through `LIN-T06` |
| `LIN-04` matrix representation and basis change | `LIN-T07` through `LIN-T08` |
| `LIN-05` linear systems and row reduction | `LIN-T09` through `LIN-T10` |
| `LIN-06` determinants, adjugates, and Cramer formulas | `LIN-T11` through `LIN-T12` |
| `LIN-07` rank theory and factorizations | `LIN-T13` through `LIN-T14` |
| `LIN-08` eigenvalues and polynomial invariants | `LIN-T15` through `LIN-T16` |
| `LIN-09` diagonalization and Cayley-Hamilton | `LIN-T17` through `LIN-T18` |
| `LIN-10` canonical forms | `LIN-T19` through `LIN-T21` |
| `LIN-11` inner-product and norm foundations | `LIN-T22` through `LIN-T23` |
| `LIN-12` orthonormal bases and projections | `LIN-T24` through `LIN-T25` |
| `LIN-13` symmetric, Hermitian, and positive-definite spectral theory | `LIN-T26` through `LIN-T27` |
| `LIN-14` normal, unitary, orthogonal, and polar theory | `LIN-T28` through `LIN-T29` |
| `LIN-15` matrix decompositions | `LIN-T30` through `LIN-T33` |
| `LIN-16` SVD and low-rank approximation | `LIN-T34` through `LIN-T35` |
| `LIN-17` bilinear and quadratic forms | `LIN-T36` through `LIN-T37` |
| `LIN-18` tensor and exterior algebra | `LIN-T38` through `LIN-T39` |
| `LIN-19` dual spaces and linear functionals | `LIN-T40` through `LIN-T41` |
| `LIN-20` projections and least squares | `LIN-T42` through `LIN-T43` |
| `LIN-21` nonnegative matrices and Perron-Frobenius | `LIN-T44` through `LIN-T45` |
| `LIN-22` matrix norms and perturbation theory | `LIN-T46` through `LIN-T47` |
| `LIN-23` matrix functions and matrix equations | `LIN-T48` through `LIN-T49` |
| `LIN-24` groups, Lie algebras, and representation linear algebra | `LIN-T50` through `LIN-T51` |
| `LIN-25` numerical linear algebra | `LIN-T52` through `LIN-T53` |
| `LIN-26` graph linear algebra | `LIN-T54` through `LIN-T55` |
| `LIN-27` convex and optimization linear algebra | `LIN-T56` through `LIN-T57` |
| `LIN-28` packaging and promotion | `LIN-T58` |

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `LAQ-001` | `LIN-T00` |
| `LAQ-002` | `LIN-T01`, `LIN-T02` |
| `LAQ-003` | `LIN-T03` |
| `LAQ-004` | `LIN-T04` |
| `LAQ-005` | `LIN-T05` |
| `LAQ-006` | `LIN-T06` |
| `LAQ-007` | `LIN-T07`, `LIN-T08` |
| `LAQ-008` | `LIN-T09` |
| `LAQ-009` | `LIN-T10` |
| `LAQ-010` | `LIN-T11` |
| `LAQ-011` | `LIN-T12` |
| `LAQ-012` | `LIN-T13`, `LIN-T14` |
| `LAQ-013` | `LIN-T15` |
| `LAQ-014` | `LIN-T16`, `LIN-T18` |
| `LAQ-015` | `LIN-T17` |
| `LAQ-016` | `LIN-T22`, `LIN-T23` |
| `LAQ-017` | `LIN-T24`, `LIN-T25` |
| `LAQ-018` | `LIN-T26`, `LIN-T27` |
| `LAQ-019` | `LIN-T31`, `LIN-T32` |
| `LAQ-020` | `LIN-T34`, `LIN-T35`, `LIN-T42` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `LIN-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `LIN-T01`, `LIN-T03`, `LIN-T07`, `LIN-T09`, `LIN-T15`, `LIN-T19`, `LIN-T20`, `LIN-T21`, `LIN-T29`, `LIN-T33`, `LIN-T44`, `LIN-T48`, `LIN-T49`, `LIN-T51`, `LIN-T53`, `LIN-T55`, `LIN-T56`, `LIN-T57` | target `L2` derived certificates from the first proof attempt; split missing prerequisite evidence before source edits instead of landing interface milestones |
| `LIN-T02`, `LIN-T04` through `LIN-T06`, `LIN-T08`, `LIN-T10` through `LIN-T14`, `LIN-T16` through `LIN-T18`, `LIN-T22` through `LIN-T28`, `LIN-T30` through `LIN-T32`, `LIN-T34` through `LIN-T43`, `LIN-T45` through `LIN-T47`, `LIN-T50`, `LIN-T52`, `LIN-T54` | `L2` derived certificates where prerequisites exist; otherwise split before source edits |
| `LIN-T58` | `L3` public closure and package verification |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

---

## Milestones

### LIN-T00 Build Linear Algebra Theorem Card Inventory

- Status: Planning-only; not part of the 2026-06-13 theorem-proof goal
- Depends on: None
- Areas: `proofs/README.md`, proof-corpus theorem-card documentation, AI index sidecars
- Tasks:
  - Create theorem cards for all `LIN-00` through `LIN-28` theorem families.
  - Record stable English identifier, Japanese display name, target level,
    primary milestone, proposed module, scalar assumptions, and dependency
    tags for every theorem card.
  - Record duplicate-home decisions for Pythagoras, Cauchy-Schwarz,
    rank-nullity, determinant-invertibility, diagonalization, SVD, Farkas,
    least squares, PageRank, and spectral graph aliases.
  - Mark each target as foundation, derived theorem, specialization,
    package alias, or long-term theorem.
- Deliverables:
  - Linear algebra theorem-card inventory and duplicate map.
- Acceptance criteria:
  - Every roadmap theorem family has exactly one primary home milestone.
  - Geometric Pythagorean theorem cards point to
    `Proofs.Ai.Geometry.Pythagorean`; inner-product norm-square aliases point
    to `LIN-11`.
  - No theorem card treats source, replay, theorem indexes, or this todo as
    proof evidence.
- Verification:
  - `rg -n "LIN-00|LIN-28|Pythagorean|rank-nullity|Farkas|PageRank" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T01 Add Vector-Space Law Bridge

- Status: Completed (2026-06-12)
- Depends on: `LIN-T00`
- Areas: `Proofs.Ai.LinearAlgebra.VectorSpace.Basic`, `tools/proof-corpus/src/main.rs`, `proofs/README.md`
- Tasks:
  - Bridge existing `Proofs.Ai.Vector.AbstractSpace` law packages into a
    linear-algebra namespace.
  - Add statement names for vector-space carrier, scalar field, vector
    addition, scalar multiplication, zero, negation, and linear combination.
  - Keep scalar assumptions explicit and reuse `Proofs.Ai.Algebra.AbstractField`.
- Deliverables:
  - First linear-algebra vector-space module or a documented statement-only
    insertion plan if source work is blocked.
  - Completed with `Proofs.Ai.LinearAlgebra.VectorSpace.Basic`, which bridges
    `VectorSpaceLawArgs` into the linear-algebra namespace and verifies
    source-free.
- Acceptance criteria:
  - Vector-space laws are imported from explicit law packages, not kernel
    primitives.
  - The bridge does not duplicate existing `Vector.AbstractSpace` theorem
    names under incompatible assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.VectorSpace.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.VectorSpace.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T02 Add Subspace, Sum, Intersection, And Direct-Sum Predicates

- Status: Completed (2026-06-12)
- Depends on: `LIN-T01`
- Areas: `Proofs.Ai.LinearAlgebra.Subspace.Basic`
- Tasks:
  - Define subspace predicates and prove closure under zero, addition, and
    scalar multiplication from explicit vector-space laws.
  - Add zero subspace, kernel-shaped subspace, image-shaped subspace, sum,
    intersection, and direct-sum statement names.
  - Separate direct-sum existence of representation from uniqueness.
- Deliverables:
  - Certificate-backed subspace foundation module.
  - Completed with `Proofs.Ai.LinearAlgebra.Subspace.Basic`, including
    subspace zero/addition/scalar closure, subspace intersection intro and
    projections, `subspace_intersection_is_subspace`, and the reusable
    `subspace_intersection_add_mem` / `subspace_intersection_smul_mem`
    closure theorems.
- Acceptance criteria:
  - Subspace facts are derived predicates, not new trusted vector primitives.
  - Later kernel, image, quotient, and graph cut/cycle tasks can import these
    predicates without restating them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Subspace.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Subspace.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T03 Add Basis, Independence, Spanning, And Coordinates

- Status: Completed (2026-06-13)
- Depends on: `LIN-T02`
- Areas: `Proofs.Ai.LinearAlgebra.Basis.Dimension`
- Tasks:
  - Define linear independence, spanning, basis, finite basis evidence, and
    coordinate representation predicates.
  - Prove coordinate representation uniqueness from basis data.
  - Add statement names for finite-dimensional evidence used by matrix and
    determinant modules.
- Deliverables:
  - Basis and coordinate API for finite-dimensional work.
  - Current coverage in `Proofs.Ai.LinearAlgebra.Basis.Dimension` now exposes
    coordinate existence and coordinate equality directly from explicit
    `FiniteDimensionCertificate` evidence via
    `finite_dimension_coordinates_exist` and `finite_dimension_coordinate_eq`.
  - Current coverage in `Proofs.Ai.LinearAlgebra.Basis.Dimension` also exposes
    coordinate uniqueness directly from finite-dimensional evidence via
    `finite_dimension_coordinate_unique`.
- Acceptance criteria:
  - Finite-dimensionality is explicit evidence, not an implicit global
    assumption.
  - Basis existence targets an `L2` derivation; if no constructive or finite
    generation route is available, split that prerequisite before source edits.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Basis.Dimension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Basis.Dimension --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T04 Prove Exchange, Dimension, Quotient, And Direct-Sum Formulas

- Status: Completed (2026-06-13)
- Depends on: `LIN-T03`
- Areas: `Proofs.Ai.LinearAlgebra.Basis.Dimension`, `Proofs.Ai.LinearAlgebra.Quotient.Basic`
- Tasks:
  - Prove Steinitz exchange, basis extension, generating-set reduction, and
    equality of cardinalities of finite bases.
  - Prove finite-dimensional dimension theorem and finite-dimensional
    vector-space isomorphism classification.
  - Add quotient vector-space existence and quotient dimension formula.
- Deliverables:
  - Dimension and quotient theorem layer for linear maps and rank.
  - Current coverage in `Proofs.Ai.LinearAlgebra.Basis.Dimension` now derives
    the symmetric orientation of finite-dimensional dimension equality via
    `dimension_theorem_symmetric`, using explicit finite-dimension
    certificates plus finite-basis cardinality agreement.
- Acceptance criteria:
  - Dimension is tied to explicit finite basis evidence.
  - Quotient results reuse subspace predicates from `LIN-T02`.
  - Direct-sum and quotient dimension statements avoid circular rank-nullity
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Basis.Dimension`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Quotient.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T05 Add Linear Map, Kernel, Image, And Basic Criteria

- Status: Completed (2026-06-13)
- Depends on: `LIN-T02`, `LIN-T03`
- Areas: `Proofs.Ai.LinearAlgebra.LinearMap.Basic`
- Tasks:
  - Define linear-map predicate, linear-map equality, composition, identity,
    kernel, and image in the linear-algebra namespace.
  - Prove kernel and image are subspaces.
  - Prove injectivity iff kernel is zero and surjectivity iff image is target
    space.
  - Add value-on-basis uniqueness and extension-from-basis statement names.
- Deliverables:
  - Basis-free linear-map module used by matrices, rank, duality, and systems.
- Acceptance criteria:
  - Kernel/image subspace facts import `LIN-T02`.
  - Matrix representations do not appear in the primary linear-map proofs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LinearMap.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.LinearMap.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T06 Prove Rank-Nullity And Linear-Map Isomorphism Theorems

- Status: Completed (2026-06-13)
- Depends on: `LIN-T04`, `LIN-T05`
- Areas: `Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism`
- Tasks:
  - Prove rank-nullity for finite-dimensional linear maps.
  - Prove first, second, and third isomorphism theorem routes for vector
    spaces.
  - Prove quotient map theorem and Hom-space dimension formula.
  - Register matrix-rank aliases as downstream imports rather than primary
    theorems.
- Deliverables:
  - Rank-nullity and isomorphism theorem certificates.
- Acceptance criteria:
  - Rank-nullity is primary here; matrix rank imports it later.
  - Isomorphism theorems use quotient-space evidence from `LIN-T04`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T07 Establish Concrete Matrix Namespace And Shape API

- Status: Completed (2026-06-13)
- Depends on: `LIN-T03`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Basic`
- Tasks:
  - Decide the concrete finite index representation for rows, columns, and
    `n x m` matrices.
  - Define matrix equality, zero, identity, addition, scalar multiplication,
    transpose, multiplication, and block-shape predicates.
  - Record row/column shape evidence explicitly.
- Deliverables:
  - Concrete matrix namespace used by all later matrix theorem families.
- Acceptance criteria:
  - Matrix shape information is structural data, not ad hoc string metadata.
  - Matrix operations state dimension compatibility in theorem assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T08 Add Matrix Representation And Basis Change

- Status: Completed (2026-06-13)
- Depends on: `LIN-T05`, `LIN-T07`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Representation`
- Tasks:
  - Define matrix representation of linear maps relative to domain and
    codomain bases.
  - Prove matrix of identity, composition, inverse, and change-of-basis
    formulas.
  - Prove similarity relation for endomorphisms and trace/determinant alias
    hooks without assuming determinant results.
- Deliverables:
  - Matrix/linear-map bridge for systems, eigenvalue theory, and dual maps.
- Acceptance criteria:
  - Basis-free linear-map theorems remain primary.
  - Basis-change formulas use explicit source and target basis evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Representation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Representation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T09 Add Linear Systems And Solution-Set Structure

- Status: Completed (2026-06-13)
- Depends on: `LIN-T05`, `LIN-T07`
- Areas: `Proofs.Ai.LinearAlgebra.Systems.Basic`
- Tasks:
  - Define homogeneous and nonhomogeneous linear systems over the concrete
    matrix namespace.
  - Prove solution set is an affine translate of the homogeneous solution
    space when a particular solution exists.
  - Connect consistency to image membership and kernel facts.
- Deliverables:
  - Linear-system theorem layer before elimination algorithms.
- Acceptance criteria:
  - System statements import linear-map image/kernel theorems.
  - Existence of solutions is not hidden in an elimination trace.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Systems.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Systems.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T10 Prove Row Operations, Gaussian Elimination, And RREF Correctness

- Status: Completed (2026-06-13)
- Depends on: `LIN-T09`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Elimination`
- Tasks:
  - Define elementary row operations and row-equivalence traces as
    mathematical data.
  - Prove elementary row operations preserve solution sets where appropriate.
  - Add Gaussian elimination correctness, RREF existence interface, and RREF
    uniqueness route.
- Deliverables:
  - Elimination theorem certificates and deterministic trace interfaces.
- Acceptance criteria:
  - Algorithm traces are proof targets, not trusted executable evidence.
  - Pivoting and invertibility side conditions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Elimination`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Elimination --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T11 Select Determinant Construction And Prove Basic Laws

- Status: Completed (2026-06-13)
- Depends on: `LIN-T07`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Determinant`
- Tasks:
  - Select one determinant construction route for the first corpus landing.
  - Define determinant, alternating multilinearity, normalization, row/column
    expansion hooks, and determinant of identity.
  - Prove determinant product theorem and determinant behavior under
    elementary operations.
- Deliverables:
  - Determinant base module and construction decision record.
- Acceptance criteria:
  - The determinant construction is chosen before Cramer, eigenvalue, and
    exterior algebra tasks depend on it.
  - Product theorem is a derived certificate, not an assumed multiplicativity
    field in a determinant package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Determinant`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Determinant --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T12 Prove Adjugate, Cramer, Invertibility, And Schur Formulas

- Status: Completed (2026-06-13)
- Depends on: `LIN-T10`, `LIN-T11`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`
- Tasks:
  - Define minors, cofactors, adjugate, and Schur complement statement
    shapes.
  - Prove adjugate identity, determinant-invertibility equivalence,
    adjugate inverse formula, and Cramer rule.
  - Add determinant formulas for block triangular matrices and Schur
    complements.
- Deliverables:
  - Determinant application theorem layer for systems and positive-definite
    work.
- Acceptance criteria:
  - Cramer rule imports solution-set and determinant theorems instead of
    becoming a primary systems theorem.
  - Schur complement side conditions state block shape and invertibility.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Adjugate`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Adjugate --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T13 Add Matrix Rank API And Row-Column Rank Theorem

- Status: Completed (2026-06-13)
- Depends on: `LIN-T06`, `LIN-T08`, `LIN-T10`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Rank`
- Tasks:
  - Define row space, column space, row rank, column rank, and rank through
    image dimension.
  - Prove row rank equals column rank and connect matrix rank to linear-map
    rank.
  - Prove rank-nullity alias for matrices.
- Deliverables:
  - Matrix rank theorem layer.
- Acceptance criteria:
  - The primary rank-nullity proof remains in `LIN-T06`.
  - Row/column rank equality does not assume rank normal form unless that route
    is explicitly the proof path.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Rank`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Rank --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T14 Prove Rank Normal Form, Factorization, And Minor Criteria

- Status: Completed (2026-06-13)
- Depends on: `LIN-T13`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.RankFactorization`
- Tasks:
  - Prove rank normal form and rank factorization.
  - Prove full-rank factorization and rank inequalities.
  - Add minor-rank criterion and determinant-rank aliases.
- Deliverables:
  - Rank factorization module used by decompositions, SVD, and numerical
    linear algebra.
- Acceptance criteria:
  - Full-row-rank and full-column-rank side conditions are explicit.
  - Minor criteria import determinant results from `LIN-T11`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.RankFactorization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.RankFactorization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T15 Add Eigenvalue, Eigenspace, And Polynomial-Invariant API

- Status: Completed
- Depends on: `LIN-T08`, `LIN-T11`
- Areas: `Proofs.Ai.LinearAlgebra.Eigen.Basic`
- Tasks:
  - Define eigenvalue, eigenvector, eigenspace, algebraic multiplicity, and
    geometric multiplicity statement shapes.
  - Prove eigenspace is a subspace and distinct eigenvectors independence.
  - Record polynomial algebra prerequisites before characteristic and minimal
    polynomial work.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Eigen.Basic` with
    eigen-equation, eigenspace, eigenvector, eigenvalue, multiplicity statement,
    polynomial-invariant prerequisite, and distinct-eigenvector independence
    routes.
- Acceptance criteria:
  - Eigenspace proofs import subspace and matrix representation modules.
  - Algebraically closed assumptions are not introduced unless a theorem needs
    them explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Eigen.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Eigen.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T16 Add Characteristic And Minimal Polynomial Routes

- Status: Completed
- Depends on: `LIN-T15`
- Areas: `Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`
- Tasks:
  - Define characteristic polynomial, minimal polynomial, and annihilating
    polynomial statement shapes.
  - Prove eigenvalue iff characteristic polynomial root when determinant
    prerequisites are present.
  - Add algebraic/geometric multiplicity inequality and invariant polynomial
    hooks for diagonalization.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`
    with characteristic-polynomial, polynomial-root, minimal/annihilating
    polynomial, eigenvalue-root, multiplicity, and invariant-polynomial routes.
- Acceptance criteria:
  - Polynomial results state the scalar ring or field assumptions they require.
  - Cayley-Hamilton is not assumed in this milestone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Polynomial.Characteristic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T17 Prove Diagonalization Criteria

- Status: Completed
- Depends on: `LIN-T15`, `LIN-T16`
- Areas: `Proofs.Ai.LinearAlgebra.Eigen.Diagonalization`
- Tasks:
  - Define diagonalizable matrix and eigenbasis predicates.
  - Prove eigenspace direct-sum criterion and distinct eigenvalues imply
    diagonalizable route.
  - Prove diagonalizability iff minimal polynomial has no repeated roots when
    polynomial prerequisites are available.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Eigen.Diagonalization` with
    diagonalizable/eigenbasis routes, eigenspace direct-sum evidence, distinct
    eigenvalue diagonalization, and minimal-polynomial squarefree criterion
    routes.
- Acceptance criteria:
  - The proof does not assume existence of enough eigenvectors without
    explicit eigenbasis evidence.
  - Similarity formulas import matrix representation results from `LIN-T08`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Eigen.Diagonalization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Eigen.Diagonalization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T18 Prove Cayley-Hamilton And Polynomial Functional Calculus

- Status: Completed
- Depends on: `LIN-T16`, `LIN-T17`
- Areas: `Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton`
- Tasks:
  - Select a Cayley-Hamilton proof route compatible with the determinant and
    polynomial modules.
  - Prove Cayley-Hamilton theorem.
  - Add polynomial functional calculus for diagonalizable matrices and
    polynomial spectral mapping.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton`
    with Cayley-Hamilton evidence, polynomial functional calculus, polynomial
    spectral mapping, and matrix-polynomial annihilation routes.
- Acceptance criteria:
  - Cayley-Hamilton is a derived certificate, not a law-package assumption.
  - Polynomial spectral mapping is separate from matrix-function and
    holomorphic functional-calculus milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T19 Add Jordan And Nilpotent Canonical-Form Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T16`, `LIN-T17`, `LIN-T18`
- Areas: `Proofs.Ai.LinearAlgebra.Canonical.Jordan`
- Tasks:
  - Define generalized eigenspace, Jordan chain, Jordan block, and nilpotent
    block predicates.
  - Add generalized eigenspace decomposition and nilpotent Jordan form route.
  - Split Jordan existence and uniqueness if a single module becomes too
    broad.
- Deliverables:
  - Jordan-form interface or derived route, depending on polynomial and scalar
    prerequisites.
- Acceptance criteria:
  - Algebraically closed field assumptions are explicit.
  - Jordan chain evidence is not treated as proof of Jordan form uniqueness.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Canonical.Jordan`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Canonical.Jordan --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T20 Add Rational, Frobenius, Smith, And Hermite Form Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T16`, `LIN-T18`
- Areas: `Proofs.Ai.LinearAlgebra.Canonical.Rational`, `Proofs.Ai.LinearAlgebra.Canonical.Smith`
- Tasks:
  - Define invariant factor and elementary divisor statement shapes.
  - Add Frobenius/rational canonical form route.
  - Add Smith and Hermite normal form interfaces over PID or Euclidean-domain
    assumptions.
- Deliverables:
  - Dependency-tagged rational and module-theoretic canonical-form modules.
- Acceptance criteria:
  - PID assumptions for Smith normal form are explicit and do not depend on
    field-only APIs.
  - Interfaces do not assume uniqueness under another name.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Canonical.Rational`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Canonical.Smith`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T21 Add Matrix Pencil Canonical-Form Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T20`
- Areas: `Proofs.Ai.LinearAlgebra.Canonical.Pencil`
- Tasks:
  - Define matrix-pencil statement shapes.
  - Add Kronecker and Weierstrass form interfaces.
  - Mark module-theory and polynomial infrastructure blockers explicitly.
- Deliverables:
  - Late-stage matrix-pencil interface module.
- Acceptance criteria:
  - Matrix-pencil canonical forms target `L2`; if required module-theory
    prerequisites are absent, record a blocker instead of landing interfaces.
  - No downstream theorem imports a matrix-pencil interface as a derived
    canonical-form theorem.
- Verification:
  - `rg -n "Kronecker|Weierstrass|Canonical.Pencil" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T22 Reuse Inner-Product Norm Laws And Classical Inequalities

- Status: Completed
- Depends on: `LIN-T00`
- Areas: `Proofs.Ai.LinearAlgebra.InnerProduct.Basic`
- Tasks:
  - Bridge existing `Proofs.Ai.Vector.AbstractInnerProduct` and
    `Proofs.Ai.Vector.AbstractInnerProductDerive` into the linear-algebra
    namespace.
  - Add Cauchy-Schwarz, triangle inequality, parallelogram law,
    polarization identity, and inner-product Pythagoras aliases.
  - Keep geometric right-triangle Pythagorean theorem names in
    `Proofs.Ai.Geometry.Pythagorean`.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.InnerProduct.Basic` with
    inner-product theorem aliases and specializations from the checked vector
    inner-product modules.
- Acceptance criteria:
  - Existing checked Cauchy-Schwarz and norm-expansion results are reused.
  - Inner-product Pythagoras is scoped to perpendicular norm identity, not
    duplicated as a geometric theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.InnerProduct.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T23 Add Gram Matrix Positivity And Gram Determinant Route

- Status: Completed
- Depends on: `LIN-T22`
- Areas: `Proofs.Ai.LinearAlgebra.InnerProduct.Gram`
- Tasks:
  - Define Gram matrix and positive semidefinite/positive definite predicates.
  - Prove Gram matrix positive semidefinite facts.
  - Add Gram determinant and linear independence route.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.InnerProduct.Gram` with
    Gram-matrix, semidefinite/positive-definite, Gram determinant, and linear
    independence routes.
- Acceptance criteria:
  - Positive semidefinite and positive definite assumptions are not conflated.
  - Complex conjugate symmetry and real symmetry variants are separate.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Gram`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.InnerProduct.Gram --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T24 Prove Gram-Schmidt And Orthonormal Basis Existence

- Status: Completed
- Depends on: `LIN-T03`, `LIN-T22`
- Areas: `Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal`
- Tasks:
  - Define orthogonal and orthonormal family predicates.
  - Prove Gram-Schmidt orthogonalization with explicit nonzero residual side
    conditions.
  - Prove finite-dimensional orthonormal basis existence.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal` with
    orthogonal/orthonormal family, Gram-Schmidt, finite-dimensional
    orthonormal-basis, and orthonormal expansion routes.
- Acceptance criteria:
  - Gram-Schmidt states nonzero residual or linearly independent input
    assumptions.
  - The theorem does not assume completeness or infinite-dimensional Hilbert
    projection results.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T25 Prove Finite-Dimensional Projection And Approximation Theorems

- Status: Completed
- Depends on: `LIN-T23`, `LIN-T24`
- Areas: `Proofs.Ai.LinearAlgebra.Projection.Orthogonal`
- Tasks:
  - Prove Fourier coefficient expansion, Bessel inequality, and Parseval
    identity in the finite-dimensional setting.
  - Prove orthogonal complement and double-orthogonal complement theorems.
  - Prove orthogonal projection existence and finite-dimensional best
    approximation theorem.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Projection.Orthogonal` with
    Fourier coefficient expansion, Bessel, Parseval, orthogonal complement,
    projection, and finite-dimensional best-approximation routes.
- Acceptance criteria:
  - Finite-dimensional projection theorem is separate from Hilbert-space
    projection theorem.
  - Bessel and Parseval statements identify finite versus complete
    orthonormal-system assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Projection.Orthogonal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Projection.Orthogonal --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T26 Prove Self-Adjoint Spectral Basics

- Status: Completed
- Depends on: `LIN-T17`, `LIN-T22`, `LIN-T24`
- Areas: `Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint`
- Tasks:
  - Define real symmetric, Hermitian, and self-adjoint predicates.
  - Prove self-adjoint eigenvalues are real and eigenvectors for distinct
    eigenvalues are orthogonal.
  - Import or specialize the existing finite-dimensional spectral theorem
    interface.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint` with
    symmetric/Hermitian/self-adjoint, real-eigenvalue, orthogonal-eigenvector,
    and finite-dimensional spectral theorem specialization routes.
- Acceptance criteria:
  - Existing `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` is reused
    rather than reproved.
  - Real symmetric and complex Hermitian statements use separate scalar
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T27 Add Positive-Definite Criteria And Variational Eigenvalue Routes

- Status: Completed
- Depends on: `LIN-T12`, `LIN-T23`, `LIN-T26`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite`
- Tasks:
  - Prove Rayleigh quotient, Courant-Fischer min-max route, and interlacing
    statement shapes where finite-dimensional compactness evidence is
    available.
  - Prove positive-definite eigenvalue criterion.
  - Prove Sylvester positive-definite criterion and Schur-complement
    positive-definiteness route.
- Deliverables:
  - Added and verified `Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite` with
    Rayleigh quotient, Courant-Fischer, interlacing, positive-definite
    eigenvalue, Sylvester, and Schur-complement routes.
- Acceptance criteria:
  - Variational eigenvalue statements name finite-dimensional basis or
    compactness evidence explicitly.
  - Schur complement results import determinant/block facts from `LIN-T12`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T28 Prove Normal, Unitary, And Orthogonal Matrix Facts

- Status: Completed (2026-06-13). Normal spectral aliases, the
  unitary-implies-normal route, and orthogonal route facts are completed as
  certificate-backed modules.
- Depends on: `LIN-T11`, `LIN-T26`
- Areas: `Proofs.Ai.LinearAlgebra.Spectral.Normal`, `Proofs.Ai.LinearAlgebra.Matrix.Unitary`
- Tasks:
  - Completed:
    - `Proofs.Ai.LinearAlgebra.Spectral.Normal` defines
      `NormalFiniteDimensionalSpectralAlias` and proves the finite-dimensional
      spectral theorem and normal diagonalization projections, including
      `normal_finite_dimensional_unitarily_diagonalizable`.
    - `Proofs.Ai.LinearAlgebra.Matrix.Unitary` defines `UnitaryMatrixRoute`
      and proves `unitary_matrix_route_from_unitary`, deriving normality from
      the left and right unitary inverse equations using equality symmetry and
      transitivity.
  - Remaining:
    - Define real orthogonal matrix predicates without identifying the real
      orthogonal and complex unitary scalar settings.
    - Prove unitary/orthogonal matrices preserve inner products and their
      eigenvalues have norm one.
    - Prove determinant of an orthogonal matrix is plus or minus one.
- Deliverables:
  - Normal/unitary/orthogonal theorem layer for QR, SVD, and matrix groups.
- Acceptance criteria:
  - Orthogonal and unitary variants do not silently identify real and complex
    scalar settings.
  - Normal matrix statements reuse the existing finite-dimensional spectral
    theorem interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Spectral.Normal`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Unitary`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T29 Add Polar Decomposition And Simultaneous Diagonalization

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`, `LIN-T28`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Polar`
- Tasks:
  - Prove simultaneous diagonalization for commuting normal families.
  - Prove polar decomposition with invertible and singular cases separated.
  - Add Householder, Givens, and Cartan-Dieudonne routes only after scalar and
    orthogonal/unitary assumptions are clear.
- Deliverables:
  - Polar and transformation theorem layer.
- Acceptance criteria:
  - Polar decomposition is primary here, not in the matrix-decomposition batch.
  - Singular-case evidence is not hidden by assuming invertibility.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Polar`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Polar --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T30 Prove LU, PLU, And LDU Decompositions

- Status: Completed (2026-06-13)
- Depends on: `LIN-T10`, `LIN-T13`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.LU`
- Tasks:
  - Define triangular, permutation, pivot, and block decomposition predicates.
  - Prove LU existence under explicit pivot conditions.
  - Add PLU and LDU decomposition routes.
- Deliverables:
  - LU-family decomposition module.
- Acceptance criteria:
  - Pivoting, nonzero diagonal, and shape assumptions are explicit.
  - LU work does not wait for polar decomposition.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.LU`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.LU --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T31 Prove QR Decomposition Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T07`, `LIN-T24`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR`
- Tasks:
  - Prove QR existence through Gram-Schmidt.
  - Add Householder QR and Givens QR interfaces after the relevant
    transformation theorems exist.
  - Record rank and full-column-rank conditions.
- Deliverables:
  - QR decomposition theorem module.
- Acceptance criteria:
  - QR by Gram-Schmidt imports orthonormal-basis facts from `LIN-T24`.
  - Householder/Givens variants do not block the basic QR theorem.
  - Householder and Givens QR variants split into a follow-up that depends on
    `LIN-T29` if the transformation route is not already available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T32 Prove Cholesky And LDLT Decompositions

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky`
- Tasks:
  - Define Cholesky, triangular positive diagonal, and LDLT predicates.
  - Prove Cholesky decomposition under positive-definite assumptions.
  - Prove LDLT route and uniqueness side conditions where available.
- Deliverables:
  - Positive-definite decomposition module.
- Acceptance criteria:
  - Cholesky and LDLT import positive-definite facts from `LIN-T27`.
  - Positivity and field/order assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T33 Add Schur, Block, And Tensor Decomposition Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T28`, `LIN-T29`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Schur`
- Tasks:
  - Add Schur decomposition and real Schur route.
  - Add eigenvalue decomposition alias and block diagonalization theorem
    hooks.
  - Keep CUR, nonnegative factorization, CP, Tucker, and tensor-rank
    interfaces dependency-tagged.
- Deliverables:
  - Advanced decomposition interface module.
- Acceptance criteria:
  - Schur-related assumptions import normal/unitary/orthogonal facts where
    required.
  - Tensor decomposition interfaces do not claim derived tensor-rank
    theorems.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Schur`
  - `rg -n "CUR|Tucker|tensor-rank|Decomposition.Schur" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T34 Prove SVD Definition, Existence, And Singular-Value Facts

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`, `LIN-T28`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.SVD`
- Tasks:
  - Define singular values via eigenvalues of `A* A`.
  - Prove singular values are nonnegative and singular vectors are orthogonal.
  - Prove SVD existence and compact SVD statement.
  - Prove rank characterization by singular values and image/kernel
    description by SVD.
- Deliverables:
  - SVD theorem module.
- Acceptance criteria:
  - SVD proof imports spectral theorem for positive semidefinite matrices.
  - Rectangular matrix shape and adjoint assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.SVD`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.SVD --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T35 Prove Moore-Penrose And Add Low-Rank Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T34`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.MoorePenrose`, `Proofs.Ai.LinearAlgebra.Matrix.LowRank`
- Tasks:
  - Prove Moore-Penrose inverse existence and uniqueness.
  - Prove Moore-Penrose inverse by SVD.
  - Add Eckart-Young and Eckart-Young-Mirsky statement hooks.
  - Add Ky Fan, Schatten norm, Davis-Kahan, and Wedin alias hooks for
    perturbation work.
- Deliverables:
  - Moore-Penrose and low-rank approximation module.
- Acceptance criteria:
  - Moore-Penrose statements include all four Penrose equations.
  - Low-rank approximation states chosen norm and rank constraint.
  - Normed low-rank approximation theorems import `LIN-T46`; pure
    Moore-Penrose existence does not wait for matrix norm development.
  - If Eckart-Young is implemented as `L2` in this milestone, split it into a
    follow-up that depends on `LIN-T46`.
  - Perturbation aliases point to `LIN-T47`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.MoorePenrose`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.LowRank`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T36 Add Bilinear, Sesquilinear, And Quadratic Form Basics

- Status: Completed (2026-06-13)
- Depends on: `LIN-T08`, `LIN-T22`
- Areas: `Proofs.Ai.LinearAlgebra.Forms.Quadratic`
- Tasks:
  - Define bilinear, sesquilinear, Hermitian, symmetric, alternating, and
    quadratic form predicates.
  - Prove matrix representation of forms and congruence transformation route.
  - Add polarization for quadratic forms with scalar-domain assumptions.
- Deliverables:
  - Form predicate and representation module.
- Acceptance criteria:
  - Bilinear, sesquilinear, and Hermitian assumptions are not conflated.
  - Form matrix representation imports basis-change results from `LIN-T08`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Forms.Quadratic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Forms.Quadratic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T37 Prove Inertia And Quadratic-Form Classification

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`, `LIN-T36`
- Areas: `Proofs.Ai.LinearAlgebra.Forms.Inertia`
- Tasks:
  - Prove Sylvester law of inertia and real quadratic-form diagonalization
    route.
  - Prove positive-definite form equivalences and principal minor criterion
    aliases.
  - Add Lagrange identity and Witt decomposition interfaces where
    prerequisites exist.
- Deliverables:
  - Inertia and quadratic-form classification module.
- Acceptance criteria:
  - Ordered-field and real/complex scalar assumptions are explicit.
  - Principal minor results import determinant and positive-definite modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Forms.Inertia`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Forms.Inertia --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T38 Add Tensor, Kronecker, Hadamard, And Schur Product Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T04`, `LIN-T08`, `LIN-T13`; the Schur-product sub-batch
  also depends on `LIN-T27`.
- Areas: `Proofs.Ai.LinearAlgebra.Tensor.Basic`
- Tasks:
  - Define tensor product universal property interface and Kronecker product
    statement shapes.
  - Prove mixed-product property and rank/determinant hooks for Kronecker
    products where prerequisites exist.
  - Add Hadamard product and Schur product theorem route.
- Deliverables:
  - Tensor and product theorem module.
- Acceptance criteria:
  - Tensor universal property is an interface until the required module
    foundations exist.
  - Schur product theorem imports positive semidefinite facts from `LIN-T27`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Tensor.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Tensor.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T39 Add Exterior Algebra And Determinant Bridge

- Status: Completed (2026-06-13)
- Depends on: `LIN-T11`, `LIN-T38`
- Areas: `Proofs.Ai.LinearAlgebra.Tensor.Exterior`
- Tasks:
  - Define exterior algebra dependency-map entry and wedge product hooks.
  - Build determinant bridge through exterior powers if compatible with the
    selected determinant construction.
  - Add symmetric tensor and Clifford algebra interfaces as late follow-ups.
- Deliverables:
  - Exterior algebra and determinant bridge module.
- Acceptance criteria:
  - The bridge does not replace the selected determinant construction from
    `LIN-T11` without updating downstream dependencies.
  - Clifford algebra remains interface-level until quadratic-form foundations
    are sufficient.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Tensor.Exterior`
  - `rg -n "Exterior|Clifford|determinant bridge" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T40 Add Dual Space And Double-Dual Theorems

- Status: Completed (2026-06-13)
- Depends on: `LIN-T04`
- Areas: `Proofs.Ai.LinearAlgebra.Dual.Basic`
- Tasks:
  - Define dual space, dual basis, and finite-dimensional double-dual map.
  - Prove dual basis existence, dual-space dimension theorem, and
    finite-dimensional double-dual isomorphism.
  - Add trace duality hooks.
- Deliverables:
  - Dual-space base module.
- Acceptance criteria:
  - Dual-space dimension imports basis and dimension results.
  - Infinite-dimensional Hahn-Banach does not appear as a linear-algebra
    primitive.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Dual.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Dual.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T41 Prove Annihilator, Dual Map, And Finite-Dimensional Riesz

- Status: Completed (2026-06-13)
- Depends on: `LIN-T22`, `LIN-T40`
- Areas: `Proofs.Ai.LinearAlgebra.Dual.Annihilator`, `Proofs.Ai.LinearAlgebra.Dual.RieszFinite`
- Tasks:
  - Prove annihilator dimension formula and subspace-annihilator
    correspondence.
  - Define dual map and prove kernel, image, contravariant functoriality, and
    transpose correspondence.
  - Prove finite-dimensional Riesz representation.
- Deliverables:
  - Duality theorem layer for tensors, optimization, transposes, and forms.
- Acceptance criteria:
  - Dual-map matrix representation imports `LIN-T08`.
  - Finite-dimensional Riesz uses inner-product evidence from `LIN-T22`.
  - Analytic Hahn-Banach aliases point to analysis work, not this module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Dual.Annihilator`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Dual.RieszFinite`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T42 Prove Least-Squares Existence And Normal Equations

- Status: Completed (2026-06-13)
- Depends on: `LIN-T25`, `LIN-T31`, `LIN-T35`
- Areas: `Proofs.Ai.LinearAlgebra.LeastSquares.Basic`
- Tasks:
  - Define projection matrix and least-squares solution predicates.
  - Prove projection matrix characterization and eigenvalues are zero or one.
  - Prove least-squares existence, residual orthogonality, normal equations,
    uniqueness condition, QR solution, and SVD solution.
- Deliverables:
  - Algebraic least-squares theorem module.
- Acceptance criteria:
  - Least-squares statements distinguish algebraic fixed-design results from
    statistical Gauss-Markov model assumptions.
  - Normal equations import orthogonal projection facts from `LIN-T25`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LeastSquares.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.LeastSquares.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T43 Add Regularized, Total Least Squares, And Procrustes Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T35`, `LIN-T42`
- Areas: `Proofs.Ai.LinearAlgebra.LeastSquares.Regularized`, `Proofs.Ai.LinearAlgebra.LeastSquares.Procrustes`
- Tasks:
  - Prove Moore-Penrose minimum-norm solution and Pythagorean decomposition
    aliases.
  - Add hat matrix properties.
  - Add ridge, Tikhonov, total least squares, and Procrustes solution routes.
  - Coordinate Gauss-Markov aliases with `STAT-T43` through `STAT-T46`.
- Deliverables:
  - Advanced least-squares and inverse-problem theorem modules.
- Acceptance criteria:
  - Statistical model assumptions are not added to algebraic least-squares
    theorems.
  - SVD-dependent results import `LIN-T35`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LeastSquares.Regularized`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LeastSquares.Procrustes`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T44 Add Nonnegative, Stochastic, And Perron-Frobenius Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T15`, `LIN-T26`
- Areas: `Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`
- Tasks:
  - Define positive, nonnegative, irreducible, primitive, stochastic, M-matrix,
    and Z-matrix predicates.
  - Add Perron-Frobenius theorem interface and Perron root statement shapes.
  - Record order/topology prerequisites for derived routes.
- Deliverables:
  - Nonnegative matrix predicate and PF interface module.
- Acceptance criteria:
  - Positivity and order assumptions are explicit.
  - Interfaces do not assume the Perron-Frobenius conclusion as a law package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`
  - `rg -n "Perron|irreducible|primitive|Nonnegative" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T45 Prove Perron, Markov, And PageRank Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T44`
- Areas: `Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`, `Proofs.Ai.LinearAlgebra.Nonnegative.Markov`
- Tasks:
  - Prove positive matrix simple maximal eigenvalue route and irreducible
    nonnegative Perron root route when prerequisites exist.
  - Prove positive Perron vector, Collatz-Wielandt formula, and primitive
    matrix convergence route.
  - Add Markov stationary distribution and PageRank existence/uniqueness
    routes.
- Deliverables:
  - Nonnegative matrix theorem layer for graph theory and Markov chains.
- Acceptance criteria:
  - Markov and PageRank results identify stochastic, damping, and
    irreducibility assumptions.
  - Norm and convergence estimates import `LIN-T46` or `LIN-T47` only when
    needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Nonnegative.Markov`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T46 Add Matrix Norm And Condition Number Theorems

- Status: Completed (2026-06-13)
- Depends on: `LIN-T22`, `LIN-T26`, `LIN-T34`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Norm`
- Tasks:
  - Add finite-dimensional norm equivalence interface.
  - Define matrix norm, operator norm, Frobenius norm, and spectral norm.
  - Prove submultiplicativity, operator-norm laws, Frobenius properties,
    spectral norm equals largest singular value, and condition number theorem.
- Deliverables:
  - Matrix norm theorem module.
- Acceptance criteria:
  - Each theorem names the norm and scalar assumptions it uses.
  - Normed-space dependencies import analysis foundations rather than
    redefining them.
- Notes:
  - Reuse existing `Proofs.Ai.Analysis.AbstractNormedSpace` where possible;
    any stronger finite-dimensional topology requirement must be split and
    linked to the analysis topology milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Norm`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Norm --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T47 Prove Matrix Perturbation And Localization Theorems

- Status: Completed (2026-06-13)
- Depends on: `LIN-T26`, `LIN-T34`, `LIN-T46`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Perturbation`
- Tasks:
  - Prove Neumann series inverse existence route when analysis series
    foundations exist.
  - Prove inverse perturbation formula, Gershgorin disk theorem, Bauer-Fike,
    Weyl eigenvalue perturbation, Hoffman-Wielandt, Davis-Kahan, and Wedin
    routes.
  - Add pseudospectrum and backward-error theorem interfaces.
- Deliverables:
  - Perturbation theorem module for numerical and graph estimates.
- Acceptance criteria:
  - Each perturbation theorem names norm and spectral assumptions.
  - Floating-point backward error remains an interface until a floating-point
    model exists.
- Notes:
  - Neumann-series inverse routes depend on analysis series foundations
    `ANA-T06` through `ANA-T09`; keep them as interfaces until those are
    available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Perturbation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Perturbation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T48 Add Matrix Exponential And Function Interface

- Status: Completed (2026-06-13)
- Depends on: `LIN-T18`, `LIN-T19`, `LIN-T26`, `LIN-T46`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Function`
- Tasks:
  - Define matrix exponential through finite-dimensional power series when
    analysis series foundations are available.
  - Prove basic exponential laws and commuting exponential law.
  - Add matrix exponential and linear ODE relation as a dependency-tagged
    route.
  - Add Cayley-Hamilton representation of matrix functions and Jordan-form
    computation route.
- Deliverables:
  - Matrix-function theorem module.
- Acceptance criteria:
  - Matrix exponential existence imports series foundations, not an unchecked
    analytic primitive.
  - ODE aliases point to analysis ODE milestones until those are available.
- Notes:
  - Matrix exponential routes depend on analysis series foundations `ANA-T06`
    through `ANA-T09`.
  - Linear ODE aliases wait for analysis ODE milestones `ANA-T33` through
    `ANA-T34`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Function`
  - `rg -n "ANA-T06|ANA-T09|Matrix.Function|matrix exponential" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LIN-T49 Add Matrix Logarithm, Square Root, And Equation Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`, `LIN-T48`
- Areas: `Proofs.Ai.LinearAlgebra.Matrix.Equation`
- Tasks:
  - Add spectral mapping for selected functions, matrix logarithm existence
    conditions, and positive-definite square root existence/uniqueness.
  - Add functional calculus for diagonalizable matrices and holomorphic
    functional calculus interface.
  - Add Sylvester, Lyapunov, Riccati, matrix sign, and asymptotic powers
    routes.
- Deliverables:
  - Matrix equation and advanced function theorem module.
- Acceptance criteria:
  - Positive square roots import spectral and positive-definite theory.
  - Holomorphic functional calculus remains an interface until complex
    analysis foundations exist.
- Notes:
  - Holomorphic functional calculus waits for complex-analysis milestones
    `ANA-T29` through `ANA-T30`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Equation`
  - `rg -n "ANA-T29|ANA-T30|holomorphic|Sylvester|Lyapunov|Riccati|Matrix.Equation" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LIN-T50 Prove Matrix Group Basics

- Status: Completed (2026-06-13)
- Depends on: `LIN-T08`, `LIN-T12`, `LIN-T28`
- Areas: `Proofs.Ai.LinearAlgebra.Groups.MatrixGroups`
- Tasks:
  - Define GL(n), SL(n), O(n), SO(n), U(n), and SU(n) predicates.
  - Prove group routes using determinant, inverse, orthogonal, and unitary
    facts.
  - Add determinant and inverse compatibility lemmas for group operations.
- Deliverables:
  - Matrix group theorem module.
- Acceptance criteria:
  - Matrix-group proofs import determinant, inverse, orthogonal, and unitary
    facts from earlier milestones.
  - Group axioms are ordinary theorem statements, not kernel assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Groups.MatrixGroups`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Groups.MatrixGroups --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T51 Add Matrix Lie Algebra And Representation Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T18`, `LIN-T48`, `LIN-T50`
- Areas: `Proofs.Ai.LinearAlgebra.Lie.MatrixLie`, `Proofs.Ai.LinearAlgebra.Representation.Basic`
- Tasks:
  - Define matrix Lie algebra predicates for `gl`, `so`, and `su`.
  - Add exponential map properties and Baker-Campbell-Hausdorff interface.
  - Add Cartan, Iwasawa, Bruhat, Jordan-Chevalley, Schur lemma, Maschke,
    complete reducibility, and Peter-Weyl interfaces.
- Deliverables:
  - Matrix Lie and representation-theory interface modules.
- Acceptance criteria:
  - Lie-algebra results state bracket and scalar assumptions explicitly.
  - Representation-theory theorems stay coordinated with algebra roadmap
    modules and are not encoded as linear-algebra axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Lie.MatrixLie`
  - `rg -n "Maschke|Schur lemma|Representation.Basic|Baker" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### LIN-T52 Add Numerical Iteration And Krylov Recurrence Theorems

- Status: Completed (2026-06-13)
- Depends on: `LIN-T30`, `LIN-T31`, `LIN-T34`, `LIN-T46`, `LIN-T48`
- Areas: `Proofs.Ai.LinearAlgebra.Numerical.Iteration`, `Proofs.Ai.LinearAlgebra.Numerical.Krylov`
- Tasks:
  - Define deterministic recurrence and trace predicates for Gaussian
    elimination, QR algorithm, power method, inverse iteration, Rayleigh
    quotient iteration, Lanczos, Arnoldi, conjugate gradient, GMRES, and
    MINRES.
  - Prove invariant and convergence routes that are purely mathematical.
  - Add preconditioning statement hooks.
- Deliverables:
  - Numerical recurrence theorem modules.
- Acceptance criteria:
  - Algorithm theorems specify recurrence, invariant, norm, and spectral
    assumptions.
  - Executable algorithm implementations are not trusted proof evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Numerical.Iteration`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Numerical.Krylov`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T53 Add Stability And Randomized Numerical Interfaces

- Status: Completed (2026-06-13)
- Depends on: `LIN-T35`, `LIN-T47`, `LIN-T52`
- Areas: `Proofs.Ai.LinearAlgebra.Numerical.Stability`, `Proofs.Ai.LinearAlgebra.Numerical.Randomized`
- Tasks:
  - Add Gaussian elimination stability, partial pivoting, QR convergence, and
    backward-error interfaces.
  - Add singular value thresholding route.
  - Add randomized SVD error evaluation, Johnson-Lindenstrauss alias, and
    matrix Chernoff/Bernstein/Hoeffding interfaces.
- Deliverables:
  - Stability and randomized numerical theorem interfaces.
- Acceptance criteria:
  - Floating-point stability targets `L2`; if a floating-point error model is
    absent, split that model as a prerequisite before theorem source work.
  - Randomized bounds import probability concentration modules from statistics
    when available.
- Notes:
  - Randomized concentration routes depend on statistics concentration
    milestones `STAT-T09` through `STAT-T11`; martingale concentration
    variants wait for `STAT-T62` through `STAT-T64`.
- Verification:
  - `rg -n "STAT-T09|STAT-T11|STAT-T62|STAT-T64|Randomized|floating-point" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LIN-T54 Prove Graph Laplacian Basics And Matrix-Tree Route

- Status: Completed (2026-06-13)
- Depends on: `LIN-T11`, `LIN-T13`, `LIN-T25`, `LIN-T26`
- Areas: `Proofs.Ai.LinearAlgebra.Graph.Laplacian`
- Tasks:
  - Define adjacency, incidence, degree, Laplacian, normalized Laplacian, and
    random-walk matrix statement shapes over explicit graph structures.
  - Prove graph Laplacian is positive semidefinite.
  - Prove Laplacian zero eigenvalue and connected components theorem.
  - Prove incidence matrix and Laplacian relation, cut/cycle orthogonal
    decomposition, and matrix-tree/Kirchhoff route.
- Deliverables:
  - Graph Laplacian theorem module.
- Acceptance criteria:
  - Graph objects are explicit structures; Laplacian theorems are not encoded
    as raw matrix assumptions only.
  - Matrix-tree route imports determinant and rank facts as needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Graph.Laplacian`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Graph.Laplacian --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T55 Add Spectral Graph, PageRank, And Resistance Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T35`, `LIN-T45`, `LIN-T47`, `LIN-T54`
- Areas: `Proofs.Ai.LinearAlgebra.Graph.Spectral`, `Proofs.Ai.LinearAlgebra.Graph.Resistance`
- Tasks:
  - Add Perron-Frobenius adjacency aliases, regular and bipartite graph
    spectral properties, Cheeger inequality, spectral clustering, expander
    mixing lemma, and eigenvalue-condition interfaces.
  - Add PageRank alias and effective resistance formula.
  - Mark Alon-Boppana and Ramanujan graph tasks as interfaces until graph
    theory prerequisites exist.
- Deliverables:
  - Spectral graph and resistance theorem modules.
- Acceptance criteria:
  - Perron-Frobenius and PageRank facts import `LIN-T45`.
  - Effective resistance imports Moore-Penrose theory from `LIN-T35`.
  - Cheeger and spectral clustering import perturbation/norm facts only where
    needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Graph.Spectral`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Graph.Resistance`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T56 Add Convex Cone, Separation, And Farkas Routes

- Status: Completed (2026-06-13)
- Depends on: `LIN-T37`, `LIN-T41`, `LIN-T42`
- Areas: `Proofs.Ai.LinearAlgebra.Optimization.Cones`
- Tasks:
  - Define convex set, convex cone, dual cone, and finite-dimensional
    separating hyperplane interfaces.
  - Prove Farkas lemma and Gordan, Stiemke, and Motzkin alternatives where
    finite-dimensional prerequisites exist.
  - Coordinate with analysis optimization `ANA-T37`.
- Deliverables:
  - Cone and alternatives theorem module.
- Acceptance criteria:
  - Farkas-style alternatives are not duplicated across LP, cone, and
    optimization modules.
  - Separation theorems identify topological, finite-dimensional, closedness,
    and constraint qualification assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Optimization.Cones`
  - `rg -n "ANA-T37|Farkas|separating hyperplane" proofs/linear-algebra-theorem-proof-roadmap*.md proofs/analysis-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LIN-T57 Add LP, KKT, SDP, And Fenchel Aliases

- Status: Completed (2026-06-13)
- Depends on: `LIN-T27`, `LIN-T56`
- Areas: `Proofs.Ai.LinearAlgebra.Optimization.LinearProgramming`, `Proofs.Ai.LinearAlgebra.Optimization.Semidefinite`
- Tasks:
  - Prove linear programming weak and strong duality routes.
  - Prove complementary slackness and KKT statement routes.
  - Add semidefinite constraint alias, SDP duality route, Moreau
    decomposition, and Fenchel-Rockafellar alias from analysis optimization.
- Deliverables:
  - LP, SDP, and optimization-duality theorem modules.
- Acceptance criteria:
  - KKT and Fenchel results coordinate with `ANA-T37` instead of creating a
    competing optimization foundation.
  - SDP results import Schur complement and positive-definite facts from
    `LIN-T27`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Optimization.LinearProgramming`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Optimization.Semidefinite`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LIN-T58 Package And Promote Stable Linear Algebra Closures

- Status: Skipped (2026-06-13; promotion explicitly out of scope)
- Depends on: any completed stable theorem batch from `LIN-T01` through `LIN-T57`
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/*`, `develop/npa-mathlib-next-closure-roadmap.md`
- Tasks:
  - Run closure audits for stable linear algebra module clusters.
  - Materialize approved closures into the standalone `npa-mathlib` repository
    only after local source-free checks and axiom reports are clean.
  - Update theorem indexes, axiom reports, package metadata, and publish-plan
    entries only when the closure is clean.
  - Document included and excluded theorem families for each public closure.
- Deliverables:
  - Promotion notes and public package artifacts for stable theorem clusters.
- Acceptance criteria:
  - Axiom report does not gain unintended axioms.
  - Source-free verifier and package checks pass for the promoted closure.
  - Public closure documentation names exact modules and theorem families.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## Review Findings Ledger

The generated task breakdown has been reviewed against:

- `proofs/linear-algebra-theorem-proof-roadmap.md`
- `proofs/analysis-theorem-proof-roadmap-todo.md`
- `proofs/statistics-theorem-proof-roadmap-todo.md`
- `proofs/README.md`
- `develop/npa-mathlib-next-closure-roadmap.md`
- `AGENTS.md`

Findings fixed during generation:

- The todo now distinguishes the pre-existing dedicated linear-algebra modules
  from the 2026-06-13 certificate-backed route modules added under canonical
  forms, decompositions, duality, least squares, numerical, graph, and
  optimization linear algebra.
- Pythagorean ownership is disambiguated between geometric theorem names and
  inner-product norm-square aliases.
- Polar decomposition is kept primary in `LIN-T29`, while matrix
  decomposition tasks import it only where needed.
- SVD is not blocked on all matrix decompositions; it depends on spectral and
  normal/unitary foundations.
- Perron-Frobenius and graph foundations do not depend on the full
  perturbation milestone except for estimates that explicitly need norms or
  Moore-Penrose theory.
- Determinant, rank, matrix-namespace, positive-definite, and norm
  prerequisites are explicit on theorem batches that use them.
- QR by Gram-Schmidt no longer waits for Householder/Givens transformation
  work; those variants split after `LIN-T29`.
- Normed Eckart-Young style low-rank approximation is no longer scheduled as
  an unconditional `L2` theorem before the matrix norm milestone.
- Matrix-function, Neumann-series, holomorphic functional-calculus, ODE, and
  randomized numerical routes now name the analysis/statistics blockers they
  need.
- Matrix exponential now waits on matrix norms, not the full perturbation
  milestone.
- Statistical regression and optimization aliases are cross-linked to
  `STAT-T43` through `STAT-T46` and `ANA-T37`.

Current findings: none.
