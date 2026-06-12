# Commutative Algebra Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T03`, `BMQ-002`)
- `develop/proof-corpus-field-theory-roadmap.md`
- `proofs/linear-algebra-theorem-proof-roadmap-todo.md`
- `proofs/number-theory-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for commutative algebra and
module theory. It is a planning sidecar only: it does not add trusted proof
evidence, axioms, source-free certificate verdicts, or package verification
claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers commutative rings, ideals, quotient rings, modules,
submodules, localization, polynomial rings, Noetherian and Artinian conditions,
PID/UFD routes, tensor products, exactness, integral extensions, primary
decomposition, dimension theory, spectra, Nakayama-style lemmas, and promotion
planning.

Out of scope for this task document:

- adding rings, ideals, modules, localization, quotients, spectra, or choice
  as trusted kernel primitives;
- treating existing field or abstract-ring law packages as if they already
  provide Noetherian, localization, or scheme-level theorem evidence;
- landing statement-only Noetherian, primary-decomposition, or Hilbert basis
  theorems as shortcuts for later algebraic geometry;
- promoting unstable commutative-algebra modules before closure audit and
  package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.Commutative.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing reusable algebra foundations include
  `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractField`,
  `Proofs.Ai.Algebra.AbstractOrderedField`, and
  `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`.
- Linear algebra already owns vector-space, basis, rank, determinant, and
  finite-dimensional linear-map routes. Commutative algebra should import
  those rather than redefining finite-dimensional module facts.
- Number theory and algebraic geometry already contain high-level modules that
  need localization, ideals, spectra, and Noetherian hypotheses. This todo
  makes those prerequisites explicit before adding more theorem-shaped
  interfaces.
- Category theory should own generic limits, adjunctions, and monoidal
  vocabulary when tensor or localization routes need them.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `CMA-00` inventory and namespace contract | `CMA-T00` |
| `CMA-01` commutative rings and ring homomorphisms | `CMA-T01` |
| `CMA-02` ideals and quotient rings | `CMA-T02` |
| `CMA-03` modules and submodules | `CMA-T03` |
| `CMA-04` localization of rings and modules | `CMA-T04` |
| `CMA-05` polynomial rings and finitely generated algebras | `CMA-T05` |
| `CMA-06` Noetherian and Artinian routes | `CMA-T06` |
| `CMA-07` PID, UFD, and factorization | `CMA-T07` |
| `CMA-08` tensor products and exactness | `CMA-T08` |
| `CMA-09` integral dependence and Nakayama routes | `CMA-T09` |
| `CMA-10` primary decomposition and dimension | `CMA-T10` |
| `CMA-11` prime spectrum and Zariski topology | `CMA-T11` |
| `CMA-12` packaging and promotion | `CMA-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `CMA-T00` | `L0` planning and theorem-card inventory |
| `CMA-T01` through `CMA-T04` | `L2` from explicit ring, ideal, module, and localization law packages |
| `CMA-T05` through `CMA-T09` | `L2` where algebraic construction prerequisites exist; split missing construction blockers before source edits |
| `CMA-T10`, `CMA-T11` | dependency route first; `L2` only for proved structural lemmas |
| `CMA-T12` | `L3` public closure and package verification |

## Milestones

### CMA-T00 Build Commutative Algebra Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory algebra, linear-algebra, number-theory, and algebraic-geometry
    theorem families that need commutative algebra.
  - Assign primary homes for ring, ideal, module, localization, Noetherian,
    integral-dependence, dimension, and spectrum theorem families.
  - Record which results require quotient, choice, finite generation, or
    category-theoretic universal properties.
- Deliverables:
  - Theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Algebraic geometry does not own foundational ideal or localization
    theorems that should live here.
  - Each theorem card states target level and prerequisite modules.
- Verification:
  - `rg -n "CMA-T00|ideal|localization|Noetherian|spectrum" proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CMA-T01 Add Commutative Ring Core

- Status: Pending
- Depends on: `CMA-T00`
- Areas: `Proofs.Ai.Algebra.Commutative.Ring`
- Tasks:
  - Define commutative-ring and ring-homomorphism law packages over existing
    abstract ring foundations.
  - Prove basic homomorphism and kernel/image projections.
  - Keep field specializations as imports from field-theory routes.
- Deliverables:
  - Commutative ring core module.
- Acceptance criteria:
  - Commutativity is explicit and not inferred from field modules unless a
    field bridge theorem is imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Ring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.Commutative.Ring --verified-cache authoring`

### CMA-T02 Add Ideals And Quotient Rings

- Status: Pending
- Depends on: `CMA-T01`
- Areas: `Proofs.Ai.Algebra.Commutative.Ideal`
- Tasks:
  - Define ideals, generated ideals, prime ideals, maximal ideals, and quotient
    ring route packages.
  - Prove ideal closure, kernel ideal, quotient projection, and first
    isomorphism-style lemmas where quotient support is explicit.
  - Record quotient feature/profile requirements.
- Deliverables:
  - Ideal and quotient ring module.
- Acceptance criteria:
  - Quotient-ring theorems expose quotient assumptions or law packages.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Ideal`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T03 Add Module And Submodule Core

- Status: Pending
- Depends on: `CMA-T01`
- Areas: `Proofs.Ai.Algebra.Commutative.Module`
- Tasks:
  - Define modules over a commutative ring, submodules, linear maps, kernels,
    images, quotients, and finite generation.
  - Coordinate vector-space specializations with linear algebra.
  - Prove basic submodule closure and homomorphism lemmas.
- Deliverables:
  - Module core module and vector-space alias map.
- Acceptance criteria:
  - Vector-space theorems are imported from linear algebra when available.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Module`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.Commutative.Module --verified-cache authoring`

### CMA-T04 Add Localization Routes

- Status: Pending
- Depends on: `CMA-T02`, `CMA-T03`
- Areas: `Proofs.Ai.Algebra.Commutative.Localization`
- Tasks:
  - Define multiplicative subsets and localization universal-property
    packages for rings and modules.
  - Prove map extension and denominator-clearing lemmas where construction
    support exists.
  - Split prime-local and field-of-fractions routes.
- Deliverables:
  - Localization route module with explicit universal properties.
- Acceptance criteria:
  - Localization does not assume existence without construction or route
    evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Localization`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T05 Add Polynomial And Finitely Generated Algebra Routes

- Status: Pending
- Depends on: `CMA-T01`
- Areas: `Proofs.Ai.Algebra.Commutative.Polynomial`
- Tasks:
  - Define polynomial-ring and finitely generated algebra law packages.
  - Add evaluation, universal property, degree, monic, and algebra-hom routes.
  - Split multivariate polynomial and symmetric-polynomial prerequisites.
- Deliverables:
  - Polynomial route module.
- Acceptance criteria:
  - Hilbert basis theorem is not assumed in the polynomial core.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Polynomial`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T06 Add Noetherian And Artinian Route

- Status: Pending
- Depends on: `CMA-T03`, `CMA-T05`
- Areas: `Proofs.Ai.Algebra.Commutative.Noetherian`
- Tasks:
  - Define ascending and descending chain conditions for ideals and modules.
  - Prove finite-generation consequences that can be derived from explicit
    chain-condition assumptions.
  - Add Hilbert basis theorem as a dependency route, not as an axiom.
- Deliverables:
  - Noetherian route package.
- Acceptance criteria:
  - Chain-condition assumptions are visible in theorem statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Noetherian`
  - `rg -n "Noetherian|Artinian|Hilbert basis" proofs/commutative-algebra-theorem-proof-roadmap-todo.md`

### CMA-T07 Add PID, UFD, And Factorization Route

- Status: Pending
- Depends on: `CMA-T02`, `CMA-T05`
- Areas: `Proofs.Ai.Algebra.Commutative.Factorization`
- Tasks:
  - Define domain, irreducible, prime element, PID, UFD, gcd domain, and
    factorization route packages.
  - Coordinate Euclidean-domain facts with number theory.
  - Prove elementary implication directions where prerequisites exist.
- Deliverables:
  - Factorization route module.
- Acceptance criteria:
  - Number theory owns arithmetic applications; this module owns algebraic
    factorization structure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Factorization`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T08 Add Tensor And Exactness Core

- Status: Pending
- Depends on: `CMA-T03`, `CAT-T05`
- Areas: `Proofs.Ai.Algebra.Commutative.Tensor`
- Tasks:
  - Define tensor-product universal-property packages for modules.
  - Add exact sequence, flatness, projective, and injective route interfaces.
  - Coordinate long exact sequences with homological algebra.
- Deliverables:
  - Tensor and exactness route module.
- Acceptance criteria:
  - Universal properties import category prerequisites explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Tensor`
  - `rg -n "tensor|exact|flat|projective|injective" proofs/commutative-algebra-theorem-proof-roadmap-todo.md`

### CMA-T09 Add Integral Dependence And Nakayama Route

- Status: Pending
- Depends on: `CMA-T04`, `CMA-T06`
- Areas: `Proofs.Ai.Algebra.Commutative.Integral`
- Tasks:
  - Define integral elements, integral extensions, finite modules, Jacobson
    radical, and local ring route packages.
  - Prove Nakayama-style consequences only from explicit finite-generation and
    radical assumptions.
  - Split going-up, going-down, and normalization routes.
- Deliverables:
  - Integral-dependence and Nakayama route module.
- Acceptance criteria:
  - No local-ring or radical theorem is used without visible assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Integral`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T10 Add Primary Decomposition And Dimension Route

- Status: Pending
- Depends on: `CMA-T06`, `CMA-T09`
- Areas: `Proofs.Ai.Algebra.Commutative.Dimension`
- Tasks:
  - Define primary ideals, radicals, Krull dimension, height, depth, and
    regular sequences.
  - Split primary decomposition, dimension theorem, and depth theorem routes.
  - Mark construction-heavy results as blockers until prerequisites exist.
- Deliverables:
  - Dimension and primary-decomposition dependency map.
- Acceptance criteria:
  - Primary decomposition is not landed as a theorem-shaped axiom.
- Verification:
  - `rg -n "primary|dimension|height|depth|regular sequence" proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CMA-T11 Add Prime Spectrum And Zariski Route

- Status: Pending
- Depends on: `CMA-T02`, `CMA-T04`, `CMA-T10`
- Areas: `Proofs.Ai.Algebra.Commutative.Spectrum`
- Tasks:
  - Define prime spectrum, basic opens, Zariski closure, localization at
    primes, and structure sheaf prerequisites.
  - Coordinate scheme ownership with algebraic geometry.
  - Prove basic open set identities where ideal laws are available.
- Deliverables:
  - Spectrum route module and algebraic-geometry alias map.
- Acceptance criteria:
  - Scheme-level theorems remain in algebraic geometry; ring-spectrum
    foundations live here.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.Commutative.Spectrum`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CMA-T12 Promote Stable Commutative Algebra Closures

- Status: Pending
- Depends on: selected stable `CMA-T01` through `CMA-T11` batches
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`,
  `proofs/generated/*`
- Tasks:
  - Run closure audits for stable commutative-algebra modules.
  - Update package metadata only at public-boundary promotion.
  - Record excluded Noetherian, dimension, and scheme-dependent routes.
- Deliverables:
  - Verified commutative-algebra closure ready for `npa-mathlib` promotion.
- Acceptance criteria:
  - Axiom reports and package checks are clean for the promoted closure.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `CMAQ-001` | theorem-card inventory | `L0` | `CMA-T00` |
| `CMAQ-002` | commutative ring law package | `L2` | `CMA-T01` |
| `CMAQ-003` | ideals and quotient rings | `L2` with quotient boundary visible | `CMA-T02` |
| `CMAQ-004` | module and submodule core | `L2` | `CMA-T03` |
| `CMAQ-005` | localization universal property route | split before source edits if construction prerequisites are absent | `CMA-T04` |
| `CMAQ-006` | polynomial route | `L2` for finite algebraic lemmas | `CMA-T05` |
| `CMAQ-007` | Noetherian route split | dependency map or `L2` for explicit chain-condition lemmas | `CMA-T06` |
| `CMAQ-008` | spectrum basics | `L2` for basic open identities | `CMA-T11` |

## Review Checklist

- Algebraic geometry imports ring and spectrum foundations instead of
  duplicating them.
- Noetherian, primary decomposition, and dimension theorems are not accepted
  as shortcuts.
- Quotient and choice assumptions are visible in law packages or theorem
  statements.
- Verification commands stay local until promotion or package metadata changes.
