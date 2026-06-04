# Linear Algebra Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided linear algebra theorem
inventory one theorem at a time in the NPA proof corpus. It is a planning
sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- vector spaces, subspaces, bases, dimension, quotient spaces, and dual spaces;
- linear maps, kernels, images, rank-nullity, matrix representations, and
  linear-map isomorphism theorems;
- matrix algebra, invertibility, linear systems, Gaussian elimination, row
  reduction, and least-squares normal equations;
- determinants, adjugates, Cramer formulas, ranks, minors, and Schur
  complements;
- eigenvalues, characteristic and minimal polynomials, diagonalization,
  Cayley-Hamilton, Jordan, rational, Smith, Hermite, Kronecker, and
  Weierstrass canonical forms;
- inner-product spaces, orthogonality, Gram-Schmidt, projection, Bessel,
  Parseval, and approximation theorems;
- symmetric, Hermitian, normal, unitary, and orthogonal matrix theorems,
  including spectral, Schur, polar, QR, LU, Cholesky, SVD, and low-rank
  approximation routes;
- bilinear and quadratic forms, tensors, exterior algebra, Kronecker products,
  matrix functions, groups, Lie-algebra-related linear algebra, numerical
  linear algebra, graph linear algebra, Perron-Frobenius theory, and
  convex-optimization linear algebra.

The plan is intentionally staged. The first priority is not to encode every
named theorem immediately, but to build reusable finite-dimensional vector,
matrix, determinant, rank, and inner-product foundations whose statements will
not need to be replaced after canonical forms, numerical theorems, graph
theorems, or optimization results depend on them.

## Existing Baseline

The current proof corpus already has reusable algebra, vector, inner-product,
linear-analysis, and spectral routes that should be reused instead of
recreated:

| Corpus module | Existing role |
| --- | --- |
| `Proofs.Ai.Algebra.AbstractRing` | explicit abstract scalar ring law package and ring theorem targets |
| `Proofs.Ai.Algebra.AbstractField` | abstract field law package over `AbstractRing` |
| `Proofs.Ai.Algebra.AbstractOrderedField` | ordered scalar laws and square-root theorem targets |
| `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge` | compatibility bridge between field and ordered-field packages |
| `Proofs.Ai.Vector.Basic` | first vector carrier and basic vector addition theorem targets |
| `Proofs.Ai.Vector.Dot` | dot product, squared norm, and squared distance theorem targets |
| `Proofs.Ai.Vector.AbstractSpace` | abstract vector-space theorem targets over explicit scalar/vector operations |
| `Proofs.Ai.Vector.AbstractInnerProduct` | abstract inner-product, norm-square, and vector norm theorem targets |
| `Proofs.Ai.Vector.AbstractInnerProductDerive` | checked norm expansion, parallelogram, polarization, and Cauchy-Schwarz routes |
| `Proofs.Ai.Analysis.AbstractLinearMap` | bounded linear maps, operator bounds, and linear isomorphism packages |
| `Proofs.Ai.Analysis.AbstractNormedSpace` | normed-space law packages and product norm estimates |
| `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` | finite-dimensional normal-matrix spectral theorem interface |
| `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem` | Hilbert-space spectral theorem interface with explicit construction evidence |

The existing `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` is deliberately
abstract and matrix-level. It records unitary diagonalization for normal
matrices via explicit finite-dimensional index/basis, complex-field, and
diagonalization evidence. A concrete `n x n` matrix namespace, determinant API,
rank API, characteristic-polynomial API, and canonical-form API still need to
be built before most classical finite-dimensional statements can be fully
derived.

Planned analysis-roadmap foundations also matter:

| Needed foundation | Expected source |
| --- | --- |
| real numbers, sequences, and completeness | `proofs/analysis-theorem-proof-roadmap.md` `ANA-01` |
| series and power series | `ANA-02` |
| one-variable and multivariable calculus | `ANA-04` through `ANA-06` |
| topology, compactness, normed spaces, and functional analysis | `ANA-07` through `ANA-09` |
| Fourier and spectral-analysis tools | `ANA-11` |
| variational methods and convex optimization | `proofs/analysis-theorem-proof-roadmap.md` roadmap `ANA-14` and task milestone `ANA-T37` |
| probability, concentration, and martingale inputs | `proofs/statistics-theorem-proof-roadmap.md` `STAT-01` through `STAT-04` and `STAT-20`; randomized bounds also track later statistics asymptotic and learning routes |

Until those prerequisites exist, numerical analysis, randomized linear algebra,
matrix concentration, infinite-dimensional Riesz/Hahn-Banach style results,
and optimization-duality statements may land as `L0` statement cards or `L1`
evidence-package interfaces, but not as fully derived `L2` theorems.

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant or shape theorem only | no |
| `L1 Evidence package` | theorem conclusion follows from explicit construction, basis, factorization, or law evidence | only if explicitly marked as an interface milestone |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

For linear algebra, `L1` interfaces are useful for basis choice,
finite-dimensionality, algebraic closure, Jordan chains, SVD factors,
decomposition outputs, and algorithm traces. Such interfaces must not be
confused with derived theorems. A task is mathematically complete only at `L2`
or `L3`, unless the scope explicitly says that the immediate target is an
interface wrapper.

## One-Theorem Work Unit

For each theorem, use this work unit:

1. Freeze the statement in the smallest suitable `Proofs.Ai.*` module.
2. Classify the target as `L0`, `L1`, `L2`, or `L3`.
3. Audit the target for circular assumptions. The theorem conclusion itself
   must not appear as an input under another name.
4. Keep imports minimal and prefer existing corpus modules.
5. Add or update the checked source, replay, metadata, and certificate.
6. Verify the target module source-free.
7. Verify changed proof-corpus artifacts.
8. At the end of a coherent batch, run the authoring gate.

Default proof-corpus commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
only for package-wide compatibility, promotion, release readiness, or changes
to certificate encoding, checker behavior, package verification, or kernel
semantics.

## Statement Policy

Linear algebra theorem statements must keep these boundaries explicit:

- Vector spaces, modules, matrices, determinants, ranks, and decompositions are
  ordinary structures and theorem-level predicates, not kernel primitives.
- Scalar assumptions are explicit. Field, ordered field, algebraically closed
  field, real closed field, complex star field, PID, Euclidean domain, and
  normed-field assumptions must not be conflated.
- Finite-dimensionality is explicit evidence. A basis, dimension, index set,
  or finite support predicate should be passed as data when the theorem needs
  it.
- Matrix results over concrete `n x n` arrays are separate from basis-free
  linear-map results. Aliases must identify which theorem is primary.
- Determinants may be introduced through alternating multilinear maps,
  exterior algebra, Leibniz sums, or universal properties, but one construction
  must be selected before determinant product and Cramer results depend on it.
- Algorithms such as Gaussian elimination, QR, LU, power iteration, Lanczos,
  Arnoldi, and conjugate gradient are proof targets about deterministic
  recurrences or traces. Their implementation is not trusted evidence.
- Numerical stability, convergence rates, and randomized matrix theorems must
  state norm, conditioning, floating-point model, probability, and asymptotic
  assumptions explicitly.
- Infinite-dimensional analytic results such as Hahn-Banach, projection
  theorem in Hilbert spaces, Riesz representation, and spectral calculus belong
  to analysis/functional-analysis routes, with finite-dimensional aliases here.

## Duplicate Theorem Policy

Several theorem names appear in multiple inventory sections. Each duplicate
must have one primary home, with other modules importing or aliasing it:

| Theorem family | Primary home |
| --- | --- |
| vector-space axioms, subspace criteria, direct sums, quotient spaces | `LIN-01` through `LIN-02` |
| rank-nullity and the fundamental theorem of linear maps | `LIN-03`, with matrix-rank aliases from `LIN-07` |
| matrix representation of linear maps and basis-change formulas | `LIN-04`, with dual-map aliases from `LIN-19` |
| Gaussian elimination, RREF, and solution-set structure | `LIN-05`, with rank and determinant aliases only after `LIN-06` and `LIN-07` |
| determinant product, adjugate, Cramer, Schur complement determinant formulas | `LIN-06` |
| row rank equals column rank, rank normal form, rank factorization | `LIN-07` |
| Cayley-Hamilton, diagonalization criteria, minimal polynomial basics | `LIN-08` through `LIN-09` |
| Jordan, rational, Frobenius, Smith, Hermite, Kronecker, and Weierstrass forms | `LIN-10`, split by scalar ring/field prerequisites |
| Cauchy-Schwarz, parallelogram, polarization, inner-product Pythagoras | `LIN-11`, reusing `Proofs.Ai.Vector.AbstractInnerProductDerive`; geometric right-triangle Pythagorean theorems remain in `Proofs.Ai.Geometry.Pythagorean` |
| Gram-Schmidt, QR by Gram-Schmidt, projection, best approximation | `LIN-12` and `LIN-15`, with least-squares aliases in `LIN-20` |
| symmetric, Hermitian, normal, and finite-dimensional spectral theorem | `LIN-13` through `LIN-14`, reusing `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` |
| LU, PLU, LDU, QR, Cholesky, LDLT, Schur, and block diagonalization | `LIN-15`; polar decomposition is primary in `LIN-14` |
| SVD, Moore-Penrose inverse, low-rank approximation, perturbation of singular subspaces | `LIN-16`, with least-squares aliases in `LIN-20` |
| Sylvester inertia and quadratic-form classification | `LIN-17` |
| tensor, exterior, symmetric, Clifford, Kronecker, Hadamard, Schur product | `LIN-18` |
| dual basis, annihilator, dual map, finite-dimensional Riesz | `LIN-19`, with analytic Hahn-Banach aliases from analysis |
| Gauss-Markov, ridge, Tikhonov, Procrustes, total least squares | `LIN-20`, with statistics aliases importing from `STAT-15` when statistical model assumptions matter |
| Perron-Frobenius and PageRank | `LIN-21`, with graph aliases in `LIN-26` |
| Gershgorin, Bauer-Fike, Weyl, Hoffman-Wielandt, Davis-Kahan, Wedin | `LIN-22`, with SVD and spectral aliases from `LIN-13` and `LIN-16` |
| matrix exponential, logarithm, functional calculus, Sylvester/Lyapunov/Riccati equations | `LIN-23` |
| GL, SL, O, SO, U, SU, matrix Lie algebras, Schur lemma, Maschke | `LIN-24`, coordinated with algebra and representation-theory routes |
| numerical algorithm convergence and stability | `LIN-25` |
| graph Laplacians, matrix-tree, Cheeger, spectral clustering, effective resistance | `LIN-26` |
| Farkas, LP duality, KKT, separation, convex cones, SDP duality | `LIN-27`, coordinated with analysis optimization roadmap `ANA-14` and task milestone `ANA-T37` |

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `LIN-00` | inventory and statement policy | theorem cards, duplicate map, target levels |
| `LIN-01` | vector-space and subspace foundations | finite vector-space and subspace law packages |
| `LIN-02` | bases, dimension, quotients, and direct sums | Steinitz exchange, basis extension, dimension theorem |
| `LIN-03` | linear maps, kernels, images, and isomorphism theorems | rank-nullity and first isomorphism theorem route |
| `LIN-04` | matrix representation and basis change | matrix representation of linear maps and similarity formulas |
| `LIN-05` | linear systems and row reduction | solution-set structure, Gaussian elimination, RREF route |
| `LIN-06` | determinants, adjugates, and Cramer formulas | determinant laws and invertibility criteria |
| `LIN-07` | rank theory and factorizations | row-rank equals column-rank and rank normal form |
| `LIN-08` | eigenvalues and polynomial invariants | eigenvalue basics, characteristic and minimal polynomial APIs |
| `LIN-09` | diagonalization and Cayley-Hamilton | diagonalization criteria and Cayley-Hamilton theorem route |
| `LIN-10` | canonical forms | Jordan, rational, Smith, Hermite, Kronecker, Weierstrass routes |
| `LIN-11` | inner-product and norm foundations | Cauchy-Schwarz, inner-product Pythagoras, Gram matrix predicates |
| `LIN-12` | orthonormal bases and projections | Gram-Schmidt, projection theorem, best approximation |
| `LIN-13` | symmetric/Hermitian/positive-definite spectral theory | real eigenvalue and orthogonal/unitary diagonalization routes |
| `LIN-14` | normal/unitary/orthogonal matrices and polar decomposition | normal spectral theorem aliases and polar route |
| `LIN-15` | matrix decompositions | LU, PLU, QR, Cholesky, LDLT, Schur route |
| `LIN-16` | SVD and low-rank approximation | SVD, Moore-Penrose inverse, Eckart-Young route |
| `LIN-17` | bilinear and quadratic forms | congruence, inertia, quadratic-form classification |
| `LIN-18` | tensor and exterior algebra | tensor universal property and exterior determinant bridge |
| `LIN-19` | dual spaces and linear functionals | dual basis, annihilator, dual map, finite-dimensional Riesz |
| `LIN-20` | projections and least squares | normal equations, minimum-norm solution, Procrustes route |
| `LIN-21` | nonnegative matrices and Perron-Frobenius | Perron root and positive eigenvector route |
| `LIN-22` | matrix norms and perturbation theory | condition number, Gershgorin, Bauer-Fike, Davis-Kahan route |
| `LIN-23` | matrix functions and matrix equations | exponential, logarithm, spectral mapping, Sylvester equations |
| `LIN-24` | groups, Lie algebras, and representation linear algebra | GL/O/U group facts and matrix Lie algebra interfaces |
| `LIN-25` | numerical linear algebra | convergence and stability routes for core algorithms |
| `LIN-26` | graph linear algebra | Laplacian spectrum, matrix-tree, Cheeger, spectral clustering |
| `LIN-27` | convex and optimization linear algebra | Farkas, duality, KKT, cone and SDP routes |
| `LIN-28` | packaging and promotion | stable `npa-mathlib` closure audits |

## LIN-00 Inventory And Statement Policy

- Status: complete for `LAQ-001`.
- Depends on: none.
- Deliverables:
  - Convert the theorem inventory into theorem cards.
  - Give every theorem a stable English identifier, Japanese display name,
    target level, dependencies, target module, and acceptance gate.
  - Mark duplicates across vector-space, matrix, determinant, rank, spectral,
    numerical, graph, statistics, and optimization areas.
- Acceptance criteria:
  - Every theorem has one primary home module.
  - Duplicates point to the primary theorem instead of being reproved.
  - Each card states whether the first target is a statement, evidence
    package, derived certificate, or public closure.
- Verification:
  - Documentation diff review.
  - `git diff --check`.

### LAQ-001 Theorem Card Register

The cards below are the canonical L0 inventory for this roadmap. The detailed
theorem-order bullets under each milestone inherit the milestone card's
primary home, dependency boundary, target module family, and acceptance gate
unless an individual theorem is later split into a more specific card. Alias
modules must import or restate by compatibility alias instead of reproving the
primary theorem.

| Card ID | Stable English identifier | Japanese display name | First target | Primary home and target modules | Acceptance gate |
| --- | --- | --- | --- | --- | --- |
| `LIN-00-CARD` | `linear_algebra_inventory_statement_policy` | 線形代数定理目録と文ポリシー | `L0 Statement` | `LIN-00`; this roadmap sidecar | documentation diff review; `git diff --check` |
| `LIN-01-CARD` | `vector_space_subspace_foundations` | ベクトル空間と部分空間の基礎 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-01`; `Proofs.Ai.LinearAlgebra.VectorSpace.Basic`, `Proofs.Ai.LinearAlgebra.Subspace.Basic` | `LIN-01` module build, module verify, changed-only |
| `LIN-02-CARD` | `basis_dimension_quotient_direct_sum` | 基底・次元・商空間・直和 | `L2 Derived certificate` | `LIN-02`; `Proofs.Ai.LinearAlgebra.Basis.Dimension`, `Proofs.Ai.LinearAlgebra.Quotient.Basic` | `LIN-02` module build, module verify, changed-only |
| `LIN-03-CARD` | `linear_maps_kernels_images_isomorphism` | 線形写像・核・像・同型定理 | `L2 Derived certificate` | `LIN-03`; `Proofs.Ai.LinearAlgebra.LinearMap.Basic`, `Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism` | `LIN-03` module build, module verify, changed-only |
| `LIN-04-CARD` | `matrix_representation_basis_change` | 行列表現と基底変換 | `L2 Derived certificate` | `LIN-04`; `Proofs.Ai.LinearAlgebra.Matrix.Basic`, `Proofs.Ai.LinearAlgebra.Matrix.Representation` | `LIN-04` module build, module verify, changed-only |
| `LIN-05-CARD` | `linear_systems_row_reduction` | 線形方程式系と行簡約 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-05`; `Proofs.Ai.LinearAlgebra.Matrix.Elimination`, `Proofs.Ai.LinearAlgebra.Systems.Basic` | `LIN-05` module build, module verify, changed-only |
| `LIN-06-CARD` | `determinants_adjugates_cramer` | 行列式・余因子行列・クラメル公式 | `L2 Derived certificate` | `LIN-06`; `Proofs.Ai.LinearAlgebra.Matrix.Determinant`, `Proofs.Ai.LinearAlgebra.Matrix.Adjugate` | `LIN-06` module build, module verify, changed-only |
| `LIN-07-CARD` | `rank_theory_rank_factorization` | 階数理論と階数分解 | `L2 Derived certificate` | `LIN-07`; `Proofs.Ai.LinearAlgebra.Matrix.Rank`, `Proofs.Ai.LinearAlgebra.Matrix.RankFactorization` | `LIN-07` module build, module verify, changed-only |
| `LIN-08-CARD` | `eigenvalues_polynomial_invariants` | 固有値と多項式不変量 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-08`; `Proofs.Ai.LinearAlgebra.Eigen.Basic`, `Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`, `Proofs.Ai.LinearAlgebra.Polynomial.Minimal` | `LIN-08` module build, module verify, changed-only |
| `LIN-09-CARD` | `diagonalization_cayley_hamilton` | 対角化とケイリー・ハミルトン | `L2 Derived certificate` | `LIN-09`; `Proofs.Ai.LinearAlgebra.Eigen.Diagonalization`, `Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton` | `LIN-09` module build, module verify, changed-only |
| `LIN-10-CARD` | `canonical_forms` | 標準形 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-10`; `Proofs.Ai.LinearAlgebra.Canonical.Jordan`, `Proofs.Ai.LinearAlgebra.Canonical.Rational`, `Proofs.Ai.LinearAlgebra.Canonical.Smith` | `LIN-10` module build, module verify, changed-only |
| `LIN-11-CARD` | `inner_product_norm_foundations` | 内積とノルムの基礎 | `L2 Derived certificate` | `LIN-11`; `Proofs.Ai.LinearAlgebra.InnerProduct.Basic`, `Proofs.Ai.LinearAlgebra.InnerProduct.Gram` | `LIN-11` module build, module verify, changed-only |
| `LIN-12-CARD` | `orthonormal_bases_projections` | 正規直交基底と射影 | `L2 Derived certificate` | `LIN-12`; `Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal`, `Proofs.Ai.LinearAlgebra.Projection.Orthogonal` | `LIN-12` module build, module verify, changed-only |
| `LIN-13-CARD` | `self_adjoint_positive_definite_spectral` | 自己随伴・正定値スペクトル理論 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-13`; `Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint`, `Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite` | `LIN-13` module build, module verify, changed-only |
| `LIN-14-CARD` | `normal_unitary_orthogonal_polar` | 正規・ユニタリ・直交・極分解 | `L2 Derived certificate` | `LIN-14`; `Proofs.Ai.LinearAlgebra.Spectral.Normal`, `Proofs.Ai.LinearAlgebra.Matrix.Unitary`, `Proofs.Ai.LinearAlgebra.Matrix.Polar` | `LIN-14` module build, module verify, changed-only |
| `LIN-15-CARD` | `matrix_decompositions` | 行列分解 | `L2 Derived certificate` | `LIN-15`; `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.LU`, `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR`, `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky`, `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Schur` | `LIN-15` module build, module verify, changed-only |
| `LIN-16-CARD` | `svd_low_rank_approximation` | 特異値分解と低階数近似 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-16`; `Proofs.Ai.LinearAlgebra.Matrix.SVD`, `Proofs.Ai.LinearAlgebra.Matrix.LowRank`, `Proofs.Ai.LinearAlgebra.Matrix.MoorePenrose` | `LIN-16` module build, module verify, changed-only |
| `LIN-17-CARD` | `bilinear_quadratic_forms` | 双線形形式と二次形式 | `L2 Derived certificate` | `LIN-17`; `Proofs.Ai.LinearAlgebra.Forms.Bilinear`, `Proofs.Ai.LinearAlgebra.Forms.Quadratic`, `Proofs.Ai.LinearAlgebra.Forms.Inertia` | `LIN-17` module build, module verify, changed-only |
| `LIN-18-CARD` | `tensor_exterior_algebra` | テンソル代数と外積代数 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-18`; `Proofs.Ai.LinearAlgebra.Tensor.Basic`, `Proofs.Ai.LinearAlgebra.Tensor.Exterior`, `Proofs.Ai.LinearAlgebra.Tensor.Kronecker` | `LIN-18` module build, module verify, changed-only |
| `LIN-19-CARD` | `dual_spaces_linear_functionals` | 双対空間と線形汎関数 | `L2 Derived certificate` | `LIN-19`; `Proofs.Ai.LinearAlgebra.Dual.Basic`, `Proofs.Ai.LinearAlgebra.Dual.Annihilator`, `Proofs.Ai.LinearAlgebra.Dual.RieszFinite` | `LIN-19` module build, module verify, changed-only |
| `LIN-20-CARD` | `projections_least_squares` | 射影と最小二乗法 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-20`; `Proofs.Ai.LinearAlgebra.LeastSquares.Basic`, `Proofs.Ai.LinearAlgebra.LeastSquares.Regularized`, `Proofs.Ai.LinearAlgebra.LeastSquares.Procrustes` | `LIN-20` module build, module verify, changed-only |
| `LIN-21-CARD` | `nonnegative_matrices_perron_frobenius` | 非負行列とペロン・フロベニウス | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-21`; `Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`, `Proofs.Ai.LinearAlgebra.Nonnegative.Markov` | `LIN-21` module build, module verify, changed-only |
| `LIN-22-CARD` | `matrix_norms_perturbation` | 行列ノルムと摂動理論 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-22`; `Proofs.Ai.LinearAlgebra.Matrix.Norm`, `Proofs.Ai.LinearAlgebra.Matrix.Perturbation` | `LIN-22` module build, module verify, changed-only |
| `LIN-23-CARD` | `matrix_functions_equations` | 行列関数と行列方程式 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-23`; `Proofs.Ai.LinearAlgebra.Matrix.Function`, `Proofs.Ai.LinearAlgebra.Matrix.Equation` | `LIN-23` module build, module verify, changed-only |
| `LIN-24-CARD` | `matrix_groups_lie_representation` | 行列群・リー代数・表現論 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-24`; `Proofs.Ai.LinearAlgebra.Groups.MatrixGroups`, `Proofs.Ai.LinearAlgebra.Lie.MatrixLie`, `Proofs.Ai.LinearAlgebra.Representation.Basic` | `LIN-24` module build, module verify, changed-only |
| `LIN-25-CARD` | `numerical_linear_algebra` | 数値線形代数 | `L1 Evidence package` | `LIN-25`; `Proofs.Ai.LinearAlgebra.Numerical.Iteration`, `Proofs.Ai.LinearAlgebra.Numerical.Krylov`, `Proofs.Ai.LinearAlgebra.Numerical.Stability`, `Proofs.Ai.LinearAlgebra.Numerical.Randomized` | `LIN-25` module build, module verify, changed-only |
| `LIN-26-CARD` | `graph_linear_algebra` | グラフ線形代数 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-26`; `Proofs.Ai.LinearAlgebra.Graph.Laplacian`, `Proofs.Ai.LinearAlgebra.Graph.Spectral`, `Proofs.Ai.LinearAlgebra.Graph.Resistance` | `LIN-26` module build, module verify, changed-only |
| `LIN-27-CARD` | `convex_optimization_linear_algebra` | 凸最適化の線形代数 | `L1 Evidence package`, then `L2 Derived certificate` | `LIN-27`; `Proofs.Ai.LinearAlgebra.Optimization.Cones`, `Proofs.Ai.LinearAlgebra.Optimization.LinearProgramming`, `Proofs.Ai.LinearAlgebra.Optimization.Semidefinite` | `LIN-27` module build, module verify, changed-only |
| `LIN-28-CARD` | `linear_algebra_packaging_promotion` | パッケージ化と公開クロージャ | `L3 Public closure` | `LIN-28`; `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/*`, `npa-mathlib` closure audit docs | authoring gate; package gate; full gate before promotion |

### LAQ-001 Duplicate Alias Register

This register makes the duplicate policy executable for future theorem
authoring: the primary card owns the theorem statement, and every listed
secondary area must import, specialize, or expose a compatibility alias.

| Duplicate family | Primary card | Secondary alias areas |
| --- | --- | --- |
| vector-space laws, subspace criteria, direct sums, quotient spaces | `LIN-01-CARD` and `LIN-02-CARD` | matrix, affine, functional-analysis, and optimization modules |
| rank-nullity and linear-map isomorphism theorems | `LIN-03-CARD` | matrix-rank aliases in `LIN-07`; systems aliases in `LIN-05` |
| matrix representation, basis change, transposes, and dual maps | `LIN-04-CARD` | dual-space aliases in `LIN-19`; representation aliases in `LIN-24` |
| Gaussian elimination, RREF, and solution-set structure | `LIN-05-CARD` | rank aliases in `LIN-07`; determinant aliases in `LIN-06` |
| determinant product, adjugate, Cramer, and Schur-complement determinant formulas | `LIN-06-CARD` | exterior-algebra bridge in `LIN-18`; matrix-group aliases in `LIN-24` |
| row rank, column rank, rank normal form, and rank factorization | `LIN-07-CARD` | SVD rank aliases in `LIN-16`; graph and statistics aliases |
| characteristic/minimal polynomial, diagonalization, and Cayley-Hamilton routes | `LIN-08-CARD` and `LIN-09-CARD` | canonical-form aliases in `LIN-10`; matrix-function aliases in `LIN-23` |
| Jordan, rational, Smith, Hermite, Kronecker, and Weierstrass forms | `LIN-10-CARD` | matrix-function and control aliases in `LIN-23` |
| Cauchy-Schwarz, parallelogram, polarization, Pythagoras, and norm identities | `LIN-11-CARD` | geometry aliases in `Proofs.Ai.Geometry.Pythagorean`; least-squares aliases in `LIN-20` |
| Gram-Schmidt, QR, projection, and best approximation | `LIN-12-CARD` | decomposition aliases in `LIN-15`; least-squares aliases in `LIN-20` |
| finite-dimensional spectral theorem, self-adjoint, Hermitian, and normal variants | `LIN-13-CARD` and `LIN-14-CARD` | perturbation aliases in `LIN-22`; graph aliases in `LIN-26` |
| LU, QR, Cholesky, Schur, polar, and block diagonalization | `LIN-15-CARD` | numerical aliases in `LIN-25`; polar primary remains `LIN-14-CARD` |
| SVD, Moore-Penrose, low-rank approximation, and singular-subspace perturbation | `LIN-16-CARD` | least-squares aliases in `LIN-20`; perturbation aliases in `LIN-22`; graph resistance aliases in `LIN-26` |
| Sylvester inertia, positive-definite criteria, quadratic forms, and Hessian tests | `LIN-17-CARD` | positive-definite matrix aliases in `LIN-13`; optimization aliases in `LIN-27` |
| tensor, exterior, Kronecker, Hadamard, and Schur product facts | `LIN-18-CARD` | determinant aliases in `LIN-06`; decomposition aliases in `LIN-15` |
| finite-dimensional duality, annihilators, Riesz, trace duality, and separation aliases | `LIN-19-CARD` | analysis Hahn-Banach aliases; optimization duality aliases in `LIN-27` |
| Gauss-Markov, ridge, Tikhonov, Procrustes, and normal equations | `LIN-20-CARD` | statistics aliases; SVD aliases from `LIN-16` |
| Perron-Frobenius, PageRank, Markov chains, and nonnegative spectra | `LIN-21-CARD` | graph aliases in `LIN-26`; numerical convergence aliases in `LIN-25` |
| Gershgorin, Bauer-Fike, Weyl, Davis-Kahan, Wedin, condition numbers, and backward error | `LIN-22-CARD` | SVD aliases in `LIN-16`; spectral aliases in `LIN-13`; numerical aliases in `LIN-25` |
| matrix exponential, logarithm, functional calculus, Sylvester, Lyapunov, and Riccati equations | `LIN-23-CARD` | ODE/control aliases from analysis; numerical aliases in `LIN-25` |
| matrix groups, Lie algebras, Schur lemma, Maschke, and representation interfaces | `LIN-24-CARD` | algebra and representation-theory roadmap aliases |
| numerical stability, Krylov methods, randomized SVD, and matrix concentration | `LIN-25-CARD` | probability/statistics concentration aliases; deterministic theorem imports from earlier cards |
| graph Laplacians, matrix-tree, Cheeger, spectral clustering, and effective resistance | `LIN-26-CARD` | Perron-Frobenius aliases from `LIN-21`; SVD/Moore-Penrose aliases from `LIN-16` |
| Farkas, LP duality, KKT, cones, SDP, and Fenchel-Rockafellar duality | `LIN-27-CARD` | analysis optimization aliases from `ANA-14` and `ANA-T37` |

## LIN-01 Vector-Space And Subspace Foundations

- Status: `LAQ-002` complete for the vector-space law bridge, subspace
  criterion, subspace projections, intersection subspace certificate, and sum
  evidence interface. Zero/kernel/image-shaped subspaces and direct-sum
  uniqueness remain planned for later `LIN-01` work.
- Depends on: existing `Proofs.Ai.Algebra.AbstractField` and
  `Proofs.Ai.Vector.AbstractSpace`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.VectorSpace.Basic`
  - `Proofs.Ai.LinearAlgebra.Subspace.Basic`
- Theorem order:
  1. vector-space law package bridge from existing abstract vector-space args;
  2. subspace criterion;
  3. zero subspace, kernel-shaped subspace, image-shaped subspace;
  4. sum and intersection of subspaces;
  5. direct-sum predicate and uniqueness of decomposition statement.
- Deliverables:
  - Reusable subspace, sum, intersection, and direct-sum predicates.
  - A compatibility layer over the existing `Vector.AbstractSpace` module.
- Acceptance criteria:
  - Vector-space laws are imported from existing explicit law packages.
  - Subspace facts are derived predicates, not new trusted vector primitives.
  - Direct sum separates existence of a representation from uniqueness.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Subspace.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Subspace.Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-02 Bases, Dimension, Quotients, And Direct Sums

- Status: `LAQ-004` complete for Steinitz exchange, basis extension,
  generating-set reduction, finite-basis cardinality agreement, and dimension
  theorem evidence certificates. Quotient vector-space existence, quotient
  dimension formulas, and finite-dimensional isomorphism classification remain
  planned for later `LIN-02` work.
- Depends on: `LIN-01`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Basis.Dimension`
  - `Proofs.Ai.LinearAlgebra.Quotient.Basic`
- Theorem order:
  1. linear independence, spanning, and basis predicates;
  2. coordinate representation uniqueness;
  3. Steinitz exchange lemma;
  4. basis extension theorem;
  5. generating-set reduction theorem;
  6. equality of cardinalities of finite bases;
  7. dimension theorem for finite-dimensional vector spaces;
  8. quotient vector-space existence and quotient dimension formula;
  9. finite-dimensional vector-space isomorphism classification.
- Deliverables:
  - Finite-dimensional evidence package used by linear maps, matrices,
    determinants, eigenvalue theory, and decomposition theorems.
- Acceptance criteria:
  - Dimension is tied to explicit finite basis evidence.
  - Quotient results reuse subspace predicates from `LIN-01`.
  - Basis existence is `L1` unless a constructive or finite generation route is
    provided.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Basis.Dimension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Basis.Dimension`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-03 Linear Maps, Kernels, Images, And Isomorphism Theorems

- Status: `LAQ-006` complete for `Proofs.Ai.LinearAlgebra.LinearMap.Basic`:
  linear-map law packaging, kernel/image predicates, kernel and image
  subspace certificates from explicit closure evidence, and the injective
  implies zero-kernel direction. The zero-kernel implies injective direction is
  exposed through an explicit criterion package until the subtraction/equality
  bridge is derived in this layer. Rank-nullity is now exposed through
  `LinearMapNullityCertificate`, `LinearMapRankCertificate`,
  `LinearMapRankNullityEvidence`, and `linear_map_rank_nullity`, which projects
  `domain_dim = nullity_dim + rank_dim` from explicit finite-domain,
  kernel-basis, image-basis, and equality evidence. Surjectivity/image-target,
  linear-map extension from basis data, Hom-space dimension, and isomorphism
  theorem routes remain planned.
- Depends on: `LIN-01` and `LIN-02`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.LinearMap.Basic`
  - `Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism`
- Theorem order:
  1. linear-map predicate and value-on-basis uniqueness;
  2. kernel and image are subspaces;
  3. injectivity iff kernel is zero;
  4. surjectivity iff image is target space;
  5. rank-nullity theorem;
  6. linear-map extension from basis data;
  7. Hom-space dimension formula;
  8. first, second, and third isomorphism theorem routes;
  9. quotient map theorem.
- Deliverables:
  - Basis-free linear-map theorem layer used by matrix representation,
    systems, rank, duality, and canonical forms.
- Acceptance criteria:
  - Rank-nullity is primary here; matrix rank imports it later.
  - Isomorphism theorems use quotient-space evidence from `LIN-02`.
  - Hom-space statements distinguish domain basis and codomain basis evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LinearMap.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LinearMap.Isomorphism`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-04 Matrix Representation And Basis Change

- Status: `LAQ-007` complete for `Proofs.Ai.LinearAlgebra.Matrix.Basic`
  and `Proofs.Ai.LinearAlgebra.Matrix.Representation`: matrices are modeled as
  indexed entry functions with pointwise equality, entrywise addition,
  transpose, explicit matrix-product evidence, and square matrix
  multiplication law packages. Linear-map matrix representation is now exposed
  through column coordinate certificates relative to chosen bases, and
  `linear_map_composition_corresponds_to_matrix_product` projects the product
  equality for a composed linear map from explicit representation and product
  evidence. Identity, inverse, basis-change/similarity, and dual-map matrix
  representation routes remain planned.
- Depends on: `LIN-02` and `LIN-03`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Basic`
  - `Proofs.Ai.LinearAlgebra.Matrix.Representation`
- Theorem order:
  1. concrete finite matrix carrier and matrix equality API;
  2. matrix addition, multiplication, transpose, identity, and associativity laws;
  3. matrix representation of a linear map relative to bases;
  4. composition corresponds to matrix product;
  5. identity corresponds to identity matrix;
  6. inverse map corresponds to inverse matrix;
  7. basis change and similarity formula;
  8. linear functional and dual-map matrix representations.
- Deliverables:
  - Matrix API that bridges basis-free linear maps and concrete finite arrays.
- Acceptance criteria:
  - Matrix multiplication laws are derived from finite sums or explicit matrix
    algebra law packages.
  - Similarity statements identify old basis, new basis, and change-of-basis
    matrix.
  - Dual-map formulas are aliases to `LIN-19` when dual-space machinery is
    needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Representation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-05 Linear Systems And Row Reduction

- Status: `LAQ-008` complete for
  `Proofs.Ai.LinearAlgebra.Systems.Basic`: homogeneous solutions are exposed
  as the kernel solution set with explicit kernel/subspace evidence, and
  nonhomogeneous solution sets are represented by a certified particular
  solution plus homogeneous offsets. `LAQ-009` complete for
  `Proofs.Ai.LinearAlgebra.Matrix.Elimination`: matrix system solution
  predicates, row-operation solution-set preservation, explicit row-reduction
  trace evidence, Gaussian-elimination correctness projections, and a separate
  RREF uniqueness route are now available. Gauss-Jordan correctness,
  pivot/free-variable, Rouche-Capelli, and fundamental solution-system results
  remain planned.
- Depends on: `LIN-03` and `LIN-04`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Systems.Basic`
  - `Proofs.Ai.LinearAlgebra.Matrix.Elimination`
- Theorem order:
  1. homogeneous solution space theorem;
  2. nonhomogeneous solution set is a translate of the homogeneous solution
     space;
  3. row operation preserves solution sets;
  4. Gaussian elimination correctness;
  5. Gauss-Jordan elimination correctness;
  6. row echelon and reduced row echelon forms;
  7. RREF uniqueness;
  8. pivot and free variable theorem;
  9. Rouche-Capelli theorem and augmented matrix criterion;
  10. fundamental solution system existence.
- Deliverables:
  - Verified row-reduction route for later rank, determinant, inverse, and
    numerical algorithm milestones.
- Acceptance criteria:
  - Algorithm traces are explicit evidence and not trusted executable code.
  - RREF uniqueness is stated separately from existence of an elimination
    trace.
  - Cramer and determinant criteria remain aliases to `LIN-06`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Systems.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Elimination`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-06 Determinants, Adjugates, And Cramer Formulas

- Status: `LAQ-010` complete for
  `Proofs.Ai.LinearAlgebra.Matrix.Determinant`: determinant identity
  normalization, transpose invariance, multilinear/alternating evidence
  packaging, square matrix multiplication law linkage, and determinant product
  theorem projections from explicit product-derivation evidence are available.
  `LAQ-011` complete for `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`: adjugate
  formula evidence, inverse-by-adjugate evidence, determinant-unit
  invertibility equivalence, and Cramer solution/coordinate formula projections
  are available with explicit square finite-index hypotheses. Row-operation
  determinant effects, triangular/block triangular formulas, Laplace/cofactor
  expansion, and advanced determinant identities remain planned.
- Depends on: `LIN-04`, with `LIN-18` optional if determinant is built through
  exterior algebra.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Determinant`
  - `Proofs.Ai.LinearAlgebra.Matrix.Adjugate`
- Theorem order:
  1. determinant construction choice and determinant normalization;
  2. multilinearity, alternation, and determinant of identity;
  3. transpose determinant;
  4. determinant of product;
  5. determinant effect of row operations;
  6. triangular and block triangular determinant formulas;
  7. Laplace expansion and cofactor theorem;
  8. adjugate formula and inverse by adjugate;
  9. determinant nonzero iff invertible;
  10. Cramer formula;
  11. Vandermonde, Cauchy, Gram, Sylvester, matrix determinant lemma, and
      Schur complement determinant routes.
- Deliverables:
  - Determinant theorem layer used by invertibility, eigenvalue theory,
    systems, rank minors, volume, and characteristic polynomial routes.
- Acceptance criteria:
  - Determinant product theorem is derived, not a determinant law assumption.
  - Invertibility equivalences state square matrix and finite-dimensional
    hypotheses explicitly.
  - Advanced determinant identities are split if the required block, inverse,
    or polynomial infrastructure is missing.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Determinant`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Adjugate`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-07 Rank Theory And Factorizations

- Status: `LAQ-012` complete for `Proofs.Ai.LinearAlgebra.Matrix.Rank`:
  row/column rank certificates, row-rank equals column-rank equality evidence,
  matrix-rank certificates, rank-nullity alias evidence, and rank-normal-form
  evidence/projections are available. Row/column operation preservation beyond
  the normal-form route, rank of transpose, product/Sylvester/Frobenius rank
  inequalities, rank-minor criteria, rank factorization, and low-rank
  approximation prerequisites remain planned.
- Depends on: `LIN-03`, `LIN-05`, and `LIN-06`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Rank`
  - `Proofs.Ai.LinearAlgebra.Matrix.RankFactorization`
- Theorem order:
  1. matrix rank agrees with rank of represented linear map;
  2. row rank equals column rank;
  3. row and column operations preserve rank;
  4. rank normal form;
  5. rank of transpose;
  6. product rank inequalities;
  7. Sylvester and Frobenius rank inequalities;
  8. rank and minors;
  9. rank factorization theorem;
  10. low-rank approximation prerequisites.
- Deliverables:
  - Matrix-rank theorem layer shared by systems, inverses, SVD, low-rank
    approximation, and numerical linear algebra.
- Acceptance criteria:
  - Matrix rank aliases rank-nullity from `LIN-03` when possible.
  - Minor-based rank criteria import determinant facts from `LIN-06`.
  - Rank factorization distinguishes existence of factors from algorithmic
    construction.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Rank`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix.Rank`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.RankFactorization`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-08 Eigenvalues And Polynomial Invariants

- Status: planned.
- Depends on: `LIN-04`, `LIN-06`, and polynomial algebra foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Eigen.Basic`
  - `Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`
  - `Proofs.Ai.LinearAlgebra.Polynomial.Minimal`
- Theorem order:
  1. eigenvalue, eigenvector, and eigenspace predicates;
  2. eigenspaces are subspaces;
  3. characteristic polynomial definition;
  4. eigenvalues are roots of the characteristic polynomial;
  5. algebraic and geometric multiplicity predicates;
  6. geometric multiplicity is at most algebraic multiplicity;
  7. eigenvectors for distinct eigenvalues are linearly independent;
  8. triangular matrix eigenvalue theorem;
  9. similar matrices have the same characteristic polynomial;
  10. trace and determinant as eigenvalue sum/product under splitting
      hypotheses;
  11. minimal polynomial existence and uniqueness;
  12. minimal polynomial divides characteristic polynomial route.
- Deliverables:
  - Polynomial invariant API for diagonalization, Cayley-Hamilton, canonical
    forms, spectral mapping, and matrix functions.
- Acceptance criteria:
  - Algebraically closed or polynomial splitting assumptions are explicit.
  - Characteristic and minimal polynomial results do not assume
    Cayley-Hamilton before `LIN-09`.
  - Trace and determinant eigenvalue formulas are split until multiset and
    splitting infrastructure exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Eigen.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Polynomial.Characteristic`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-09 Diagonalization And Cayley-Hamilton

- Status: planned.
- Depends on: `LIN-08`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Eigen.Diagonalization`
  - `Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton`
- Theorem order:
  1. diagonalizability predicates and diagonalizing basis evidence;
  2. eigenspace direct-sum theorem;
  3. eigenbasis iff diagonalizable;
  4. distinct eigenvalues imply diagonalizable route;
  5. diagonalizability iff minimal polynomial has no repeated roots;
  6. Cayley-Hamilton theorem;
  7. polynomial functional calculus for diagonalizable matrices;
  8. spectral mapping for polynomials.
- Deliverables:
  - Diagonalization and Cayley-Hamilton theorem layer used by matrix functions,
    canonical forms, and spectral theorems.
- Acceptance criteria:
  - Diagonalization by eigenbasis does not assume existence of enough
    eigenvectors without evidence.
  - Cayley-Hamilton proof selects a route compatible with determinant and
    polynomial modules.
  - Polynomial spectral mapping is separate from holomorphic functional
    calculus in `LIN-23`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Eigen.Diagonalization`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Polynomial.CayleyHamilton`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-10 Canonical Forms

- Status: planned.
- Depends on: `LIN-08`, `LIN-09`, and ring/PID/polynomial factorization
  foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Canonical.Jordan`
  - `Proofs.Ai.LinearAlgebra.Canonical.Rational`
  - `Proofs.Ai.LinearAlgebra.Canonical.Smith`
- Theorem order:
  1. generalized eigenspace decomposition;
  2. nilpotent Jordan form route;
  3. Jordan chains and Jordan form existence;
  4. Jordan form uniqueness;
  5. Jordan-Chevalley and Dunford decomposition routes;
  6. Fitting decomposition;
  7. Frobenius/rational canonical form;
  8. invariant factors and elementary divisors;
  9. Smith normal form over a PID;
  10. Hermite normal form;
  11. Kronecker and Weierstrass forms for matrix pencils.
- Deliverables:
  - Dependency-tagged canonical-form theorem routes.
- Acceptance criteria:
  - Algebraically closed field assumptions for Jordan form are explicit.
  - PID assumptions for Smith normal form are explicit and do not depend on
    field-only APIs.
  - Matrix-pencil canonical forms are late interfaces until the required
    module-theory and polynomial infrastructure exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Canonical.Jordan`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Canonical.Rational`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-11 Inner-Product And Norm Foundations

- Status: planned.
- Depends on: existing `Proofs.Ai.Vector.AbstractInnerProduct` and
  `Proofs.Ai.Vector.AbstractInnerProductDerive`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.InnerProduct.Basic`
  - `Proofs.Ai.LinearAlgebra.InnerProduct.Gram`
- Theorem order:
  1. inner-product law package bridge;
  2. norm from inner product;
  3. Cauchy-Schwarz inequality alias or specialization;
  4. triangle inequality route;
  5. parallelogram law and polarization identity aliases;
  6. inner-product Pythagoras / perpendicular norm identity;
  7. Gram matrix positive semidefinite predicate;
  8. Gram determinant and linear independence route.
- Deliverables:
  - Inner-product theorem layer reused by orthogonality, spectral theorem,
    QR, SVD, least squares, and perturbation theory.
- Acceptance criteria:
  - Existing checked Cauchy-Schwarz and norm-expansion results are reused.
  - Gram matrix statements distinguish positive semidefinite from positive
    definite assumptions.
  - Complex conjugate symmetry and real symmetry are not conflated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Gram`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-12 Orthonormal Bases And Projections

- Status: planned.
- Depends on: `LIN-02` and `LIN-11`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal`
  - `Proofs.Ai.LinearAlgebra.Projection.Orthogonal`
- Theorem order:
  1. orthogonal and orthonormal family predicates;
  2. Gram-Schmidt orthogonalization;
  3. orthonormal basis existence in finite-dimensional inner-product spaces;
  4. Fourier coefficient expansion in finite dimensions;
  5. Bessel inequality;
  6. Parseval identity;
  7. orthogonal complement theorem;
  8. double orthogonal complement in finite dimensions;
  9. orthogonal projection existence;
  10. best approximation theorem.
- Deliverables:
  - Orthogonality and projection theorem layer for QR, least squares, spectral
    theorem, and Hilbert-space aliases.
- Acceptance criteria:
  - Gram-Schmidt states nonzero residual side conditions or linearly
    independent input assumptions.
  - Finite-dimensional projection theorem is separate from Hilbert-space
    projection theorem.
  - Bessel and Parseval statements identify finite versus complete
    orthonormal-system assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.InnerProduct.Orthonormal`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Projection.Orthogonal`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-13 Symmetric, Hermitian, And Positive-Definite Spectral Theory

- Status: planned.
- Depends on: `LIN-09`, `LIN-11`, `LIN-12`, and existing
  `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint`
  - `Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite`
- Theorem order:
  1. real symmetric and Hermitian matrix predicates;
  2. self-adjoint eigenvalues are real;
  3. eigenvectors for distinct eigenvalues are orthogonal;
  4. real symmetric orthogonal diagonalization;
  5. Hermitian unitary diagonalization;
  6. finite-dimensional spectral theorem alias from
     `AbstractSpectralTheorem`;
  7. Rayleigh quotient;
  8. Courant-Fischer min-max theorem;
  9. Weyl and Cauchy interlacing routes;
  10. positive-definite eigenvalue criterion;
  11. Sylvester positive-definite criterion;
  12. Schur complement and positive definiteness.
- Deliverables:
  - Spectral and positive-definite theorem layer for SVD, Cholesky, numerical
    analysis, statistics, and optimization.
- Acceptance criteria:
  - Existing spectral theorem package is imported rather than reproved.
  - Real symmetric and complex Hermitian statements use separate scalar
    assumptions.
  - Variational eigenvalue theorems state compactness or finite-dimensional
    basis evidence explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Spectral.SelfAdjoint`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.PositiveDefinite`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-14 Normal, Unitary, Orthogonal, And Polar Theory

- Status: planned.
- Depends on: `LIN-13`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Spectral.Normal`
  - `Proofs.Ai.LinearAlgebra.Matrix.Unitary`
  - `Proofs.Ai.LinearAlgebra.Matrix.Polar`
- Theorem order:
  1. normal matrix spectral theorem alias;
  2. normal iff unitarily diagonalizable route;
  3. unitary and orthogonal matrices preserve inner products;
  4. eigenvalues of unitary/orthogonal matrices have norm one;
  5. determinant of an orthogonal matrix is plus or minus one;
  6. simultaneous diagonalization for commuting normal families;
  7. Householder and Givens transformation theorems;
  8. polar decomposition;
  9. Cartan-Dieudonne route.
- Deliverables:
  - Normal/unitary/orthogonal theorem layer used by QR, SVD, numerical
    algorithms, and matrix groups.
- Acceptance criteria:
  - Normal matrix statements reuse the existing finite-dimensional spectral
    theorem interface.
  - Orthogonal and unitary variants do not silently identify real and complex
    scalar settings.
  - Polar decomposition states invertible versus singular cases separately.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Spectral.Normal`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Polar`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-15 Matrix Decompositions

- Status: planned.
- Depends on: `LIN-05`, `LIN-12`, and `LIN-13`; Schur and polar-related
  aliases also depend on `LIN-14`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.LU`
  - `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR`
  - `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky`
  - `Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Schur`
- Theorem order:
  1. LU existence conditions;
  2. PLU and LDU decompositions;
  3. QR existence through Gram-Schmidt;
  4. Householder QR and Givens QR interfaces;
  5. Cholesky decomposition;
  6. LDLT decomposition;
  7. Schur decomposition and real Schur route;
  8. eigenvalue decomposition alias;
  9. block diagonalization and simultaneous diagonalization aliases;
  10. CUR, nonnegative factorization, CP, Tucker, and tensor-rank interfaces.
- Deliverables:
  - Matrix decomposition route organized by prerequisites and scalar
    assumptions.
- Acceptance criteria:
  - Each decomposition states shape, rank, pivoting, positivity, or normality
    assumptions explicitly.
  - QR by Gram-Schmidt imports `LIN-12`.
  - Cholesky and LDLT import positive-definite facts from `LIN-13`.
  - LU, QR, and Cholesky sub-batches should not wait for polar decomposition.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.QR`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Decomposition.Cholesky`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-16 SVD And Low-Rank Approximation

- Status: planned.
- Depends on: `LIN-13` and `LIN-14`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.SVD`
  - `Proofs.Ai.LinearAlgebra.Matrix.LowRank`
  - `Proofs.Ai.LinearAlgebra.Matrix.MoorePenrose`
- Theorem order:
  1. singular value definition via eigenvalues of `A* A`;
  2. singular values are nonnegative;
  3. SVD existence;
  4. compact SVD;
  5. left and right singular vector orthogonality;
  6. rank characterization by singular values;
  7. image and kernel description by SVD;
  8. Moore-Penrose inverse existence and uniqueness;
  9. Moore-Penrose inverse by SVD;
  10. Eckart-Young and Eckart-Young-Mirsky theorems;
  11. Ky Fan and Schatten norm routes;
  12. Davis-Kahan and Wedin singular-subspace perturbation aliases.
- Deliverables:
  - SVD theorem layer used by least squares, PCA, numerical linear algebra,
    statistics, and perturbation theory.
- Acceptance criteria:
  - SVD proof imports spectral theorem for positive semidefinite matrices.
  - Moore-Penrose inverse statements include all four Penrose equations.
  - Low-rank approximation states the chosen norm and rank constraint.
  - Moore-Penrose and least-squares applications coordinate with `LIN-15` and
    `LIN-20`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.SVD`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.MoorePenrose`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-17 Bilinear And Quadratic Forms

- Status: planned.
- Depends on: `LIN-06`, `LIN-09`, `LIN-13`, and `LIN-16`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Forms.Bilinear`
  - `Proofs.Ai.LinearAlgebra.Forms.Quadratic`
  - `Proofs.Ai.LinearAlgebra.Forms.Inertia`
- Theorem order:
  1. bilinear and quadratic form matrix representation;
  2. symmetric bilinear form and quadratic form correspondence;
  3. congruence transformation theorem;
  4. Lagrange square-completion route;
  5. real and complex quadratic-form standard forms;
  6. Sylvester law of inertia;
  7. positive and semidefinite form criteria;
  8. principal-minor criteria;
  9. Rayleigh quotient relation;
  10. Hessian extremum and quadratic minimization route.
- Deliverables:
  - Form classification layer for positive definiteness, optimization, and
    spectral theorem applications.
- Acceptance criteria:
  - Congruence is not confused with similarity.
  - Inertia law states real closed or ordered-field assumptions explicitly.
  - Hessian and optimization aliases import analysis/optimization routes when
    differentiability is required.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Forms.Quadratic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Forms.Inertia`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-18 Tensor And Exterior Algebra

- Status: planned.
- Depends on: `LIN-02`, `LIN-03`, and algebra module foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Tensor.Basic`
  - `Proofs.Ai.LinearAlgebra.Tensor.Exterior`
  - `Proofs.Ai.LinearAlgebra.Tensor.Kronecker`
- Theorem order:
  1. tensor product universal property;
  2. tensor product existence interface;
  3. tensor product basis theorem and dimension formula;
  4. Hom-tensor adjunction;
  5. exterior algebra universal property and alternating product;
  6. exterior basis theorem;
  7. determinant as action on top exterior power route;
  8. symmetric algebra and Clifford algebra interfaces;
  9. Kronecker product eigenvalue, determinant, and rank formulas;
  10. vec operation, Khatri-Rao, Hadamard, and Schur product theorem routes.
- Deliverables:
  - Tensor/exterior theorem layer that can support determinants, multilinear
    algebra, Kronecker products, and tensor decompositions.
- Acceptance criteria:
  - Universal properties are explicit evidence packages.
  - Exterior determinant bridge does not duplicate determinant primary theorems
    from `LIN-06` after they exist.
  - Tensor rank and tensor decomposition theorems remain late interfaces until
    multilinear foundations are stable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Tensor.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Tensor.Exterior`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-19 Dual Spaces And Linear Functionals

- Status: planned.
- Depends on: `LIN-02`, `LIN-03`, `LIN-04`, `LIN-11`, and analysis
  functional-analysis milestones for infinite-dimensional results.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Dual.Basic`
  - `Proofs.Ai.LinearAlgebra.Dual.Annihilator`
  - `Proofs.Ai.LinearAlgebra.Dual.RieszFinite`
- Theorem order:
  1. dual space definition;
  2. dual basis existence;
  3. dual-space dimension theorem;
  4. natural map to double dual;
  5. finite-dimensional double-dual isomorphism;
  6. annihilator dimension formula;
  7. subspace-annihilator correspondence;
  8. dual-map definition, kernel, and image;
  9. contravariant functoriality;
  10. transpose map and dual map correspondence;
  11. finite-dimensional Riesz representation;
  12. trace duality route;
  13. Hahn-Banach and separation theorem aliases from analysis.
- Deliverables:
  - Finite-dimensional duality theorem layer used by tensors, optimization,
    transposes, and bilinear forms.
- Acceptance criteria:
  - Dual-map matrix representation imports `LIN-04`.
  - Infinite-dimensional Hahn-Banach, dual norm, polar, and Fenchel duality
    remain analysis/optimization routes, not linear-algebra primitives.
  - Finite-dimensional Riesz uses inner-product evidence from `LIN-11`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Dual.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Dual.Annihilator`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-20 Projections And Least Squares

- Status: planned.
- Depends on: `LIN-12`, `LIN-15`, and `LIN-16`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.LeastSquares.Basic`
  - `Proofs.Ai.LinearAlgebra.LeastSquares.Regularized`
  - `Proofs.Ai.LinearAlgebra.LeastSquares.Procrustes`
- Theorem order:
  1. projection matrix characterization;
  2. projection matrix eigenvalues are zero or one;
  3. least-squares existence theorem;
  4. residual orthogonality;
  5. normal equations;
  6. uniqueness condition;
  7. QR least-squares solution;
  8. SVD least-squares solution;
  9. Moore-Penrose minimum-norm solution;
  10. Pythagorean decomposition;
  11. hat matrix properties;
  12. ridge and Tikhonov closed-form solution routes;
  13. total least squares by SVD;
  14. Procrustes solution.
- Deliverables:
  - Least-squares theorem layer shared by statistics, numerical linear
    algebra, inverse problems, and optimization.
- Acceptance criteria:
  - Least-squares statements distinguish algebraic fixed-design results from
    statistical Gauss-Markov model assumptions.
  - Normal equations import orthogonal projection facts.
  - Moore-Penrose and SVD solutions import `LIN-16`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LeastSquares.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.LeastSquares.Regularized`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-21 Nonnegative Matrices And Perron-Frobenius

- Status: planned.
- Depends on: `LIN-08`, `LIN-13`, and order/topology foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`
  - `Proofs.Ai.LinearAlgebra.Nonnegative.Markov`
- Theorem order:
  1. positive, nonnegative, irreducible, primitive, stochastic, M-matrix, and
     Z-matrix predicates;
  2. Perron-Frobenius theorem interface;
  3. positive matrix simple maximal eigenvalue route;
  4. irreducible nonnegative matrix Perron root route;
  5. positive Perron vector;
  6. Collatz-Wielandt formula;
  7. primitive matrix convergence theorem;
  8. Markov matrix stationary distribution theorem;
  9. PageRank existence and uniqueness route;
  10. Frobenius normal form.
- Deliverables:
  - Nonnegative-matrix theorem layer for graph theory, Markov chains, and
    PageRank.
- Acceptance criteria:
  - Positivity and order assumptions are explicit.
  - Markov and PageRank results identify stochastic, damping, and
    irreducibility assumptions.
  - Perron-Frobenius is primary here; graph modules import it.
  - Norm and convergence estimates import `LIN-22` only when needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Nonnegative.PerronFrobenius`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Nonnegative.Markov`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-22 Matrix Norms And Perturbation Theory

- Status: planned.
- Depends on: `LIN-11`, `LIN-13`, `LIN-16`, and analysis normed-space
  foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Norm`
  - `Proofs.Ai.LinearAlgebra.Matrix.Perturbation`
- Theorem order:
  1. finite-dimensional norm equivalence interface;
  2. matrix norm submultiplicativity;
  3. operator norm definition and basic laws;
  4. Frobenius norm properties;
  5. spectral norm equals largest singular value;
  6. condition number theorem;
  7. Neumann series inverse existence route;
  8. inverse perturbation formula;
  9. Gershgorin disk theorem;
  10. Bauer-Fike theorem;
  11. Weyl eigenvalue perturbation;
  12. Hoffman-Wielandt theorem;
  13. Davis-Kahan and Wedin routes;
  14. pseudospectrum and backward-error theorem interfaces.
- Deliverables:
  - Norm and perturbation theorem layer for numerical linear algebra and
    stability analysis.
- Acceptance criteria:
  - Each perturbation theorem names the norm and spectral assumptions it uses.
  - Neumann series route imports series foundations from analysis when needed.
  - Floating-point backward error remains an interface until a floating-point
    model exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Norm`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Perturbation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-23 Matrix Functions And Matrix Equations

- Status: planned.
- Depends on: `LIN-09`, `LIN-10`, `LIN-13`, `LIN-22`, and analysis series
  foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Matrix.Function`
  - `Proofs.Ai.LinearAlgebra.Matrix.Equation`
- Theorem order:
  1. matrix exponential existence by finite-dimensional power series;
  2. basic exponential laws;
  3. exponential law for commuting matrices;
  4. matrix exponential and linear ODE relation;
  5. Cayley-Hamilton representation of matrix functions;
  6. Jordan-form computation route;
  7. spectral mapping theorem for selected functions;
  8. matrix logarithm existence conditions;
  9. positive-definite square root existence and uniqueness;
  10. functional calculus for diagonalizable matrices;
  11. holomorphic functional calculus interface;
  12. Sylvester and Lyapunov equation existence/uniqueness;
  13. Riccati equation linear algebra route;
  14. matrix sign function and asymptotic powers.
- Deliverables:
  - Matrix-function theorem layer for ODEs, stability, control, and numerical
    algorithms.
- Acceptance criteria:
  - Matrix exponential existence imports series foundations, not an unchecked
    analytic primitive.
  - Positive square roots import spectral and positive-definite theory.
  - Holomorphic functional calculus remains an interface until complex
    analysis foundations exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Function`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix.Equation`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-24 Groups, Lie Algebras, And Representation Linear Algebra

- Status: planned.
- Depends on: `LIN-04`, `LIN-06`, `LIN-14`, `LIN-23`, and algebra group/ring
  foundations.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Groups.MatrixGroups`
  - `Proofs.Ai.LinearAlgebra.Lie.MatrixLie`
  - `Proofs.Ai.LinearAlgebra.Representation.Basic`
- Theorem order:
  1. GL(n) group route;
  2. SL(n), O(n), SO(n), U(n), and SU(n) group routes;
  3. matrix Lie algebra predicates for `gl`, `so`, and `su`;
  4. exponential map properties;
  5. Baker-Campbell-Hausdorff interface;
  6. Cartan, Iwasawa, and Bruhat decomposition interfaces;
  7. Jordan-Chevalley alias;
  8. Schur lemma;
  9. Maschke theorem;
  10. complete reducibility and Peter-Weyl interfaces.
- Deliverables:
  - Matrix group and representation-theory interface layer.
- Acceptance criteria:
  - Matrix-group proofs import determinant, inverse, orthogonal, and unitary
    facts from earlier milestones.
  - Lie-algebra results state bracket and scalar assumptions explicitly.
  - Representation-theory theorems remain coordinated with algebra roadmap
    modules and are not encoded as linear-algebra axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Groups.MatrixGroups`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Lie.MatrixLie`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-25 Numerical Linear Algebra

- Status: planned.
- Depends on: `LIN-05`, `LIN-15`, `LIN-16`, `LIN-22`, `LIN-23`, and
  probability/statistics concentration foundations for randomized results.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Numerical.Iteration`
  - `Proofs.Ai.LinearAlgebra.Numerical.Krylov`
  - `Proofs.Ai.LinearAlgebra.Numerical.Stability`
  - `Proofs.Ai.LinearAlgebra.Numerical.Randomized`
- Theorem order:
  1. Gaussian elimination stability interface;
  2. partial pivoting route;
  3. QR algorithm convergence interface;
  4. power method, inverse iteration, and Rayleigh quotient iteration routes;
  5. Lanczos tridiagonalization and Arnoldi Hessenberg decomposition;
  6. Krylov subspace properties;
  7. conjugate gradient convergence;
  8. GMRES minimum residual property;
  9. MINRES and preconditioning routes;
  10. singular value thresholding theorem;
  11. randomized SVD error evaluation;
  12. Johnson-Lindenstrauss lemma alias;
  13. matrix Chernoff, Bernstein, and Hoeffding interfaces.
- Deliverables:
  - Numerical linear algebra theorem route with algorithm traces and
    stability assumptions explicit.
- Acceptance criteria:
  - Iterative algorithm theorems specify recurrence, invariant, norm, and
    spectral assumptions.
  - Floating-point stability theorems are `L1` until a floating-point error
    model exists.
  - Randomized bounds import probability concentration modules when available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Numerical.Iteration`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Numerical.Krylov`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-26 Graph Linear Algebra

- Status: planned.
- Depends on: `LIN-13`, `LIN-21`, and graph-theory foundations; Cheeger,
  spectral clustering, and resistance estimates also use `LIN-16` and
  `LIN-22`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Graph.Laplacian`
  - `Proofs.Ai.LinearAlgebra.Graph.Spectral`
  - `Proofs.Ai.LinearAlgebra.Graph.Resistance`
- Theorem order:
  1. adjacency, incidence, degree, Laplacian, normalized Laplacian, and random
     walk matrix statement shapes;
  2. graph Laplacian is positive semidefinite;
  3. Laplacian zero eigenvalue and connected components theorem;
  4. incidence matrix and Laplacian relation;
  5. cut space and cycle space orthogonal decomposition;
  6. matrix-tree theorem and Kirchhoff theorem;
  7. Perron-Frobenius and adjacency matrix alias;
  8. regular and bipartite graph spectral properties;
  9. Cheeger inequality;
  10. spectral clustering theorem route;
  11. expander mixing lemma;
  12. Alon-Boppana and Ramanujan graph eigenvalue condition interfaces;
  13. PageRank alias and effective resistance formula.
- Deliverables:
  - Spectral graph theorem route using linear-algebra primary theorems.
- Acceptance criteria:
  - Graph-theory objects are explicit structures; graph Laplacian theorems are
    not encoded as raw matrix assumptions only.
  - Perron-Frobenius and PageRank facts import `LIN-21`.
  - Effective resistance imports Moore-Penrose inverse theory from `LIN-16`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Graph.Laplacian`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Graph.Spectral`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-27 Convex And Optimization Linear Algebra

- Status: planned.
- Depends on: `LIN-17`, `LIN-19`, `LIN-20`, `LIN-22`, and analysis
  optimization roadmap `ANA-14` / task milestone `ANA-T37`.
- Target modules:
  - `Proofs.Ai.LinearAlgebra.Optimization.Cones`
  - `Proofs.Ai.LinearAlgebra.Optimization.LinearProgramming`
  - `Proofs.Ai.LinearAlgebra.Optimization.Semidefinite`
- Theorem order:
  1. convex set, convex cone, dual cone, and separating hyperplane interfaces;
  2. Farkas lemma;
  3. Gordan, Stiemke, and Motzkin alternatives;
  4. linear programming weak and strong duality;
  5. complementary slackness;
  6. KKT conditions;
  7. Caratheodory, Helly, Radon, Minkowski-Weyl, and Krein-Milman routes;
  8. Schur complement and semidefinite constraint alias;
  9. SDP duality route;
  10. Moreau decomposition;
  11. Fenchel-Rockafellar duality alias from analysis optimization.
- Deliverables:
  - Linear-algebra optimization theorem layer for feasibility alternatives,
    cones, LP, SDP, and statistical learning aliases.
- Acceptance criteria:
  - Separation and duality theorems identify topological, finite-dimensional,
    closedness, and constraint qualification assumptions.
  - Farkas-style alternatives are not duplicated across LP, cone, and
    optimization modules.
  - KKT and Fenchel results coordinate with `ANA-14` / `ANA-T37` instead of
    creating a competing optimization foundation.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Optimization.Cones`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Optimization.LinearProgramming`
  - `cargo run -p npa-proof-corpus -- --changed-only`

## LIN-28 Packaging And Promotion

- Status: planned.
- Depends on: any completed stable theorem batch from `LIN-01` through
  `LIN-27`.
- Target areas:
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/*`
  - `develop/npa-mathlib-next-closure-roadmap.md`
- Deliverables:
  - Closure audits and `npa-mathlib` promotion notes for stable linear algebra
    module clusters.
  - Updated theorem indexes, axiom reports, package metadata, and publish-plan
    entries only when closure is clean.
- Acceptance criteria:
  - Axiom report does not gain unintended axioms.
  - Source-free verifier and package checks pass for the promoted closure.
  - Public closure documentation states which theorem families are included and
    excluded.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## Recommended First Execution Queue

The first batch should focus on foundations that unlock many later theorem
families:

| Queue ID | Theorem or task | Target level | Primary milestone |
| --- | --- | --- | --- |
| `LAQ-001` | theorem-card inventory and duplicate map | `L0` | `LIN-00` |
| `LAQ-002` | vector-space law bridge, subspace criterion, sums and intersections | `L1` then `L2` | `LIN-01` |
| `LAQ-003` | linear independence, spanning, basis predicates, coordinate uniqueness | `L2` | `LIN-02` |
| `LAQ-004` | Steinitz exchange, basis extension, dimension theorem | `L2` | `LIN-02` |
| `LAQ-005` | kernel/image subspace facts and injectivity/kernel criterion | `L2` | `LIN-03` |
| `LAQ-006` | rank-nullity theorem | `L2` | `LIN-03` |
| `LAQ-007` | matrix representation of linear maps and composition/matrix product | `L2` | `LIN-04` |
| `LAQ-008` | homogeneous and nonhomogeneous solution-set structure | `L2` | `LIN-05` |
| `LAQ-009` | Gaussian elimination correctness and RREF uniqueness route | `L1` then `L2` | `LIN-05` |
| `LAQ-010` | determinant basic properties and determinant product theorem | `L2` | `LIN-06` |
| `LAQ-011` | adjugate inverse, determinant-invertibility equivalence, Cramer formula | `L2` | `LIN-06` |
| `LAQ-012` | row rank equals column rank and rank normal form | `L2` | `LIN-07` |
| `LAQ-013` | eigenvalue/eigenspace basics and distinct eigenvectors independence | `L2` | `LIN-08` |
| `LAQ-014` | characteristic/minimal polynomial API and Cayley-Hamilton route | `L1` then `L2` | `LIN-08`, then `LIN-09` |
| `LAQ-015` | diagonalization criteria and eigenspace direct sum | `L2` | `LIN-09` |
| `LAQ-016` | Cauchy-Schwarz, inner-product Pythagoras, perpendicular norm identity, parallelogram, and polarization aliases | `L2` | `LIN-11` |
| `LAQ-017` | Gram-Schmidt and finite-dimensional orthogonal projection | `L2` | `LIN-12` |
| `LAQ-018` | finite-dimensional spectral theorem audit and self-adjoint aliases | `L1` then `L2` | `LIN-13` |
| `LAQ-019` | QR and Cholesky decompositions | `L2` | `LIN-15` |
| `LAQ-020` | SVD interface, Moore-Penrose inverse, and least-squares normal equations | `L1` then `L2` | `LIN-16`, then `LIN-20` |

After `LAQ-020`, choose based on project priority:

- continue to `LIN-10` for Jordan/rational/Smith canonical forms;
- continue to `LIN-22` and `LIN-25` for perturbation and numerical linear
  algebra;
- continue to `LIN-21` and `LIN-26` for Perron-Frobenius and graph spectral
  theorem work;
- continue to `LIN-27` for Farkas, linear programming duality, KKT, and
  semidefinite optimization;
- continue to `LIN-23` and `LIN-24` for matrix functions, matrix equations,
  matrix groups, and Lie-algebra-related theorem families.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Matrix and basis-free APIs diverge | duplicate theorem families and later alias churn | make `LIN-03` and `LIN-04` the bridge and record primary homes in theorem cards |
| Determinant construction is chosen too late | determinant, rank, eigenvalue, Cramer, and exterior algebra routes block each other | select a determinant construction in `LIN-06` before advanced determinant work |
| Existence theorems hide choice principles | `L1` packages are mistaken for fully derived `L2` theorems | label basis, decomposition, Jordan chain, and factorization evidence explicitly |
| Scalar assumptions are underspecified | real, complex, ordered, algebraically closed, PID, and star-field results become incompatible | include scalar-domain assumptions in every theorem card |
| Canonical forms are attempted before polynomial/PID foundations | broad rewrites and circular statements | keep canonical forms dependency-tagged in `LIN-10` |
| Numerical stability is treated as executable testing | trusted boundary is widened incorrectly | represent algorithms as mathematical recurrences and traces, not trusted code |
| Randomized linear algebra is attempted before probability foundations | theorem statements cannot be verified cleanly | keep matrix concentration and randomized SVD at `L1` until statistics/probability routes exist |
| Graph and optimization aliases duplicate primary linear algebra theorems | inconsistent APIs across applications | import Perron-Frobenius, SVD, duality, and spectral facts from their primary milestones |

## Decision Points

- Decide the concrete finite index and matrix representation before `LIN-04`
  commits downstream APIs.
- Decide the determinant construction route before `LIN-06`.
- Decide whether rank is represented primarily by image dimension, row/column
  span dimension, or rank normal form before `LIN-07` proofs land.
- Decide polynomial algebra prerequisites for characteristic/minimal
  polynomial work before `LIN-08`.
- Decide the scalar regimes for spectral work: real symmetric, complex
  Hermitian, normal over complex star field, and abstract evidence-package
  variants should not be merged prematurely.
- Before any `L3` promotion, run closure audit and choose package gates
  according to changed artifacts.
