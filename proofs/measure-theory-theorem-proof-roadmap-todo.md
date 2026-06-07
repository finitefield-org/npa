# Measure Theory Theorem Proof Roadmap Todo

Source: `proofs/measure-theory-theorem-proof-roadmap.md`

This task breakdown converts the measure theory theorem proof roadmap into
implementation-ready authoring milestones. It is a planning sidecar only: it
does not add trusted proof evidence, axioms, or certificate validity
assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files,
replay files, metadata, theorem indexes, this todo document, tactics, and AI
output are untrusted.

---

## Scope

This task list covers theorem-card inventory, namespace setup, sigma algebras,
measures, outer measure and extension, Lebesgue and Lebesgue-Stieltjes
construction, measurable functions, simple functions, Lebesgue integration,
convergence theorems, product measures, pushforwards, signed and complex
measures, decomposition theorems, regularity, differentiation, `L^p` spaces,
topological measure theory, probability bridges, geometric measure theory, and
promotion planning.

The list intentionally does not prove the measure-theory roadmap in one pass.
Later agents should implement exactly one milestone or a clearly bounded
contiguous batch. When a milestone introduces only a statement interface
because prerequisites are absent, its acceptance criteria must prevent the
interface from smuggling the target theorem as an axiom.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding measure spaces, integrals, real numbers, topological spaces,
  probability spaces, or `L^p` spaces as trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- treating theorem-search sidecars, AI indexes, replay files, metadata, or
  generated docs as trusted evidence;
- silently identifying Riemann integration with Lebesgue integration;
- promoting unstable measure modules into `npa-mathlib` before local closure,
  axiom-report, and package verification checks are clean.

## Authoring Loop

For ordinary measure-theory theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Reserve `check-corpus-package.sh` or
`check-corpus-full.sh` for package-wide verifier behavior, publish-plan or
package metadata updates, certificate/checker compatibility, release work, or
promotion into a high-trust closure.

## Current Implementation Facts

- `Proofs.Ai.Measure.Inventory`, `Proofs.Ai.Measure.SigmaAlgebra`,
  `Proofs.Ai.Measure.MonotoneClass`,
  `Proofs.Ai.Measure.MeasurableSpace`, and
  `Proofs.Ai.Measure.Product.SigmaAlgebra` provide the sigma-algebra and
  measurable-space foundation. `Proofs.Ai.Measure.Basic`,
  `Proofs.Ai.Measure.Completion`, and `Proofs.Ai.Measure.Restriction` now add
  basic measure structures, derived additivity/order interfaces, L1 completion
  hooks, and restriction hooks. `Proofs.Ai.Measure.Outer` and
  `Proofs.Ai.Measure.Caratheodory` add outer-measure laws, split-criterion
  measurability, Caratheodory sigma-algebra evidence, and restriction of an
  outer measure to the Caratheodory measurable sets. `Proofs.Ai.Measure.Extension`
  adds premeasure domain interfaces, premeasure-induced outer-measure extension
  interfaces, Caratheodory and Hahn-Kolmogorov extension packages, and
  sigma-finite uniqueness through pi-lambda routes; product measure and integral
  measure modules are not yet present.
- `Proofs.Ai.Measure.SigmaAlgebra` defines sigma-algebra core evidence,
  countable-intersection and set-difference vocabulary, explicit L1 routes for
  finite intersection, set difference, and symmetric difference, generated
  sigma-algebra minimality, Borel topology hooks, and real-line Borel generator
  hooks without importing measure, integral, or product-measure modules.
- `Proofs.Ai.Measure.MonotoneClass` defines pi-system, lambda-system, and
  monotone-class evidence packages, plus Dynkin pi-lambda and monotone-class
  generated-subset routes that reuse generated sigma-algebra minimality.
- `Proofs.Ai.Measure.MeasurableSpace` defines measurable spaces as
  sigma-algebra-equipped carriers, measurable-map preimage laws, and a
  certificate-backed measurable-map composition theorem.
- `Proofs.Ai.Measure.Product.SigmaAlgebra` defines product rectangles,
  rectangle-generated product sigma algebras, and coordinate-map
  measurability hooks without importing product-measure, Fubini, or Tonelli
  APIs.
- `Proofs.Ai.Measure.Basic` defines measure spaces over measurable spaces with
  explicit value-support assumptions, null empty set, finite additivity,
  countable additivity over disjoint families, monotonicity, finite and
  countable subadditivity routes, difference and finite inclusion-exclusion
  formulas with explicit finiteness premises, continuity hooks, finite /
  probability / sigma-finite measure interfaces, and measure-operation route
  statements. `Proofs.Ai.Measure.Completion` keeps null-subset measurability
  as an explicit `L1` hook, and `Proofs.Ai.Measure.Restriction` states the
  source-free restriction agreement interface.
- Existing concrete sequence and integral module trees include
  `Proofs.Ai.Analysis.Sequence.Basic`,
  `Proofs.Ai.Analysis.Sequence.Compactness`, and
  `Proofs.Ai.Analysis.Integral.Riemann.Basic`; there is not yet a concrete
  `Proofs.Ai.Analysis.Fourier.*` module tree.
- Existing reusable algebra and scalar foundations include
  `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractField`,
  `Proofs.Ai.Algebra.AbstractOrderedField`, and
  `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`.
- Existing reusable metric and analysis foundations include
  `Proofs.Ai.Analysis.AbstractMetricTopology`,
  `Proofs.Ai.Analysis.AbstractNormedSpace`,
  `Proofs.Ai.Analysis.AbstractLinearMap`,
  `Proofs.Ai.Analysis.AbstractDerivative`,
  `Proofs.Ai.Analysis.AbstractFixedPoint`,
  `Proofs.Ai.Analysis.AbstractInverseFunction`,
  `Proofs.Ai.Analysis.AbstractImplicitPhi`, and
  `Proofs.Ai.Analysis.AbstractImplicitFunction`.
- Existing reusable Hilbert and spectral interfaces include
  `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem` and
  `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem`.
- The analysis roadmap already has coarse measure milestones `ANA-T24` through
  `ANA-T26`; this document decomposes those coarse milestones into
  measure-specific `MEA-Txx` work units.
- Older analysis-roadmap wording used `Proofs.Ai.Measure.Construction` as a
  coarse construction bucket. The detailed route splits that bucket into
  `Proofs.Ai.Measure.Outer`, `Proofs.Ai.Measure.Caratheodory`, and
  `Proofs.Ai.Measure.Extension`.
- Statistics and probability work should wait for the appropriate measure
  milestones or remain at statement and evidence-package level.
- Public `npa-mathlib` has already materialized several analysis closures
  through `npa-mathlib v0.1.27`; measure-theory promotion must go through a
  separate closure audit.

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `MEA-00` inventory and statement policy | `MEA-T00` through `MEA-T01` |
| `MEA-01` sigma algebras and measurable spaces | `MEA-T02` through `MEA-T05` |
| `MEA-02` basic measures | `MEA-T06` through `MEA-T08` |
| `MEA-03` outer measure and extension | `MEA-T09` through `MEA-T12` |
| `MEA-04` Lebesgue and Lebesgue-Stieltjes measures | `MEA-T13` through `MEA-T15` |
| `MEA-05` measurable functions | `MEA-T16` through `MEA-T18` |
| `MEA-06` simple functions and integral construction | `MEA-T19` through `MEA-T21` |
| `MEA-07` convergence theorem chain | `MEA-T22` through `MEA-T25` |
| `MEA-08` product measures and Fubini-Tonelli | `MEA-T26` through `MEA-T29` |
| `MEA-09` pushforwards and change of variables | `MEA-T30` through `MEA-T32` |
| `MEA-10` signed, complex, and decomposed measures | `MEA-T33` through `MEA-T36` |
| `MEA-11` Lebesgue regularity and differentiation | `MEA-T37` through `MEA-T39` |
| `MEA-12` `L^p` spaces and inequalities | `MEA-T40` through `MEA-T43` |
| `MEA-13` topological measures and weak convergence | `MEA-T44` through `MEA-T47` |
| `MEA-14` probability, martingale, and ergodic bridges | `MEA-T48` through `MEA-T51` |
| `MEA-15` geometric and abstract measure theory | `MEA-T52` through `MEA-T55` |
| `MEA-16` packaging and promotion | `MEA-T56` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `MEA-T00` | `L0` planning, theorem-card inventory, duplicate map, and dependency tags |
| `MEA-T01` through `MEA-T05` | `L1` interfaces first where set and countability foundations are absent; `L2` closure lemmas where prerequisites exist |
| `MEA-T06` through `MEA-T08` | `L2` derived certificates for basic measure laws from explicit measure structures |
| `MEA-T09` through `MEA-T12` | `L1` construction interfaces are allowed for extension theorems; non-extension outer-measure laws target `L2` |
| `MEA-T13` through `MEA-T15` | `L1` construction interfaces until real-line and topology prerequisites are certified |
| `MEA-T16` through `MEA-T25` | `L2` derived certificates where measurable-function and integral foundations exist; split before source edits if numeric prerequisites are missing |
| `MEA-T26` through `MEA-T36` | `L2` derived certificates after product, signed-measure, and absolute-continuity APIs are stable; construction-heavy existence statements may start as `L1` |
| `MEA-T37` through `MEA-T55` | `L1` interfaces first for topology-heavy, probability, martingale, ergodic, geometric, and measure-algebra results; promote to `L2` only after prerequisites are present |
| `MEA-T56` | `L3` public closure planning and package verification |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `MEQ-001` | `MEA-T00`, `MEA-T01` |
| `MEQ-002` | `MEA-T02`, `MEA-T03` |
| `MEQ-003` | `MEA-T04`, `MEA-T05` |
| `MEQ-004` | `MEA-T06`, `MEA-T07`, `MEA-T08` |
| `MEQ-005` | `MEA-T09`, `MEA-T10` |
| `MEQ-006` | `MEA-T11`, `MEA-T12` |
| `MEQ-007` | `MEA-T16`, `MEA-T17`, `MEA-T18` |
| `MEQ-008` | `MEA-T19`, `MEA-T20`, `MEA-T21` |
| `MEQ-009` | `MEA-T22`, `MEA-T23`, `MEA-T24`, `MEA-T25` |
| `MEQ-010` | `MEA-T13`, `MEA-T14`, `MEA-T15` |
| `MEQ-011` | `MEA-T26`, `MEA-T27`, `MEA-T28`, `MEA-T29` |
| `MEQ-012` | `MEA-T33`, `MEA-T34`, `MEA-T35`, `MEA-T36` |
| `MEQ-013` | `MEA-T30`, `MEA-T31`, `MEA-T32` |
| `MEQ-014` | `MEA-T40`, `MEA-T41`, `MEA-T42`, `MEA-T43` |
| `MEQ-015` | `MEA-T37`, `MEA-T38`, `MEA-T39` |
| `MEQ-016` | `MEA-T44`, `MEA-T45`, `MEA-T46`, `MEA-T47` |
| `MEQ-017` | `MEA-T48`, `MEA-T49`, `MEA-T50`, `MEA-T51` |
| `MEQ-018` | `MEA-T52`, `MEA-T53`, `MEA-T54`, `MEA-T55` |
| `MEQ-019` | `MEA-T56` |

---

## Milestones

### MEA-T00 Build Measure-Theory Theorem Card Inventory

- Status: Completed (2026-06-08)
- Depends on: None
- Areas: `proofs/README.md`, `proofs/measure-theory-theorem-cards.md`,
  theorem-card documentation, AI theorem index sidecars
- Tasks:
  - Create theorem cards for `MEA-00` through `MEA-16`.
  - Record duplicate-home decisions for Fubini, Tonelli, dominated
    convergence, Radon-Nikodym, `L^p`, weak convergence, martingale, and
    ergodic theorem families.
  - Tag each card with target level, prerequisite modules, axiom expectations,
    and intended proof-corpus namespace.
- Deliverables:
  - Measure-theory theorem-card inventory and duplicate map.
- Acceptance criteria:
  - Every roadmap theorem family has a card or an intentionally grouped card.
  - Duplicates point to this roadmap or to the owning roadmap instead of being
    reproved under multiple names.
  - The inventory states that sidecars and theorem search are untrusted.
- Verification:
  - `rg -n "MEA-00|MEA-16|Fubini|Radon-Nikodym|sidecar" proofs`
  - `git diff --check`
- Completion notes:
  - Completed with `proofs/measure-theory-theorem-cards.md`, covering
    `MEA-00` through `MEA-16`, duplicate-home decisions, dependency tags,
    target levels, and the sidecar trust boundary.
  - No mathematical theorem certificate is claimed for the card document; it
    remains an untrusted planning sidecar.

### MEA-T01 Create Measure Namespace Skeleton And Statement Policy

- Status: Completed (2026-06-08)
- Depends on: `MEA-T00`
- Areas: `Proofs/Ai/Measure/Inventory/`, `tools/proof-corpus/src/main.rs`,
  `proofs/README.md`, `proofs/manifest.toml`, `proofs/npa-package.toml`
- Tasks:
  - Create the first `Proofs.Ai.Measure.Inventory` or equivalent statement
    module if the authoring route needs a concrete module.
  - Add namespace conventions for `Measure.Basic`, `Measure.Outer`,
    `Measure.Caratheodory`, `Measure.Extension`, `Measure.Integral`,
    `Measure.Product`, and `Measure.Decomposition`.
  - Record the trusted-boundary statement policy inside theorem cards or
    module comments.
- Deliverables:
  - A measure namespace entry point or documented statement-only plan.
- Acceptance criteria:
  - Measure objects are ordinary proof-corpus structures, not kernel
    primitives.
  - Parser, tactic, AI, theorem-search, replay, and metadata sidecars are not
    cited as proof evidence.
  - Namespace names match the detailed roadmap.
- Verification:
  - `rg -n "Proofs.Ai.Measure|trusted|kernel primitive" proofs/measure-theory-theorem-proof-roadmap-todo.md proofs`
  - `git diff --check`
- Completion notes:
  - Completed with the first concrete `Proofs.Ai.Measure.Inventory` module.
    Its checked policy theorems record ordinary proof-corpus measure objects,
    the namespace split, duplicate-home routing, untrusted sidecars,
    probability specialization, and source-free certificate requirements.
  - Public package metadata updates remain a promotion/package boundary; the
    authoring path verifies the generated source, certificate, metadata, replay,
    and untrusted AI theorem index entry locally.

### MEA-T02 Define Sigma-Algebra Core Interface

- Status: Completed (2026-06-08)
- Depends on: `MEA-T01`
- Areas: `Proofs/Ai/Measure/SigmaAlgebra/`
- Tasks:
  - Define the sigma-algebra carrier, membership predicate, and primitive
    closure laws.
  - Add statement names for empty set, universal set, complement, countable
    union, countable intersection, set difference, and symmetric difference.
  - Keep countability and set-operation assumptions explicit if the local set
    API is still interface-level.
- Deliverables:
  - Sigma-algebra core source, replay, metadata, and certificate artifacts or
    a statement-only interface explicitly marked `L1`.
- Acceptance criteria:
  - Derived closure statements do not assume the closure result as a field
    unless the milestone is explicitly `L1`.
  - The module does not depend on measure, integral, or product-measure
    structures.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.SigmaAlgebra`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.SigmaAlgebra --verified-cache authoring`
- Completion notes:
  - Completed with `Proofs.Ai.Measure.SigmaAlgebra`, generated source,
    certificate, metadata, replay, and AI theorem-index entries.
  - The core interface packages empty set, universal set, complement, and
    countable union as primitive sigma-algebra evidence; countable
    intersection is derived from complement plus countable union.
  - Finite intersection, set difference, and symmetric difference are exposed
    through the explicitly named `SigmaAlgebraDerivedClosureRoutes` L1 route
    package because finite set/cardinality foundations are not yet available as
    reusable lower-level closure proofs.
  - The module imports topology basics and the measure inventory contract, but
    does not import measure, integral, or product-measure structures.

### MEA-T03 Add Generated Sigma Algebra And Borel Generator Statements

- Status: Completed (2026-06-08)
- Depends on: `MEA-T02`
- Areas: `Proofs/Ai/Measure/SigmaAlgebra/`, `Proofs/Ai/Measure/MeasurableSpace/`
- Tasks:
  - Add generated sigma algebra as an intersection or explicit minimality
    evidence package.
  - Prove or package introduction and minimality lemmas.
  - Add Borel sigma algebra and real-line generator statement shapes without
    assuming real-line measure.
- Deliverables:
  - Generated sigma-algebra API with Borel statement hooks.
- Acceptance criteria:
  - Borel generator statements depend on topology or real-line assumptions,
    not on Lebesgue measure.
  - Minimality and seed-family inclusion are both available to later modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.SigmaAlgebra`
  - `rg -n "generated|Borel|minimal" proofs/Proofs/Ai/Measure proofs/measure-theory-theorem-proof-roadmap-todo.md`
- Completion notes:
  - Completed in `Proofs.Ai.Measure.SigmaAlgebra` with
    `GeneratedSigmaAlgebra`, `BorelSigmaAlgebra`, and
    `RealLineBorelGeneratorHook`.
  - Later modules can project generated sigma-algebra core evidence,
    seed-family inclusion, and minimality; Borel hooks project topology
    dependence, Borel sigma-core evidence, open-set inclusion, and minimality.
  - Real-line Borel generator statements depend on topology and interval-seed
    evidence only; no Lebesgue measure, measure space, or integral API is
    imported or assumed. `Proofs.Ai.Measure.MeasurableSpace` remains future
    work for `MEA-T05`.

### MEA-T04 Add Pi-Lambda And Monotone-Class Tools

- Status: Completed (2026-06-08)
- Depends on: `MEA-T02`, `MEA-T03`
- Areas: `Proofs/Ai/Measure/MonotoneClass/`
- Tasks:
  - Add pi-system and lambda-system statement interfaces.
  - Add Dynkin pi-lambda theorem and monotone-class theorem interfaces.
  - Record intended use sites for extension uniqueness and product-measure
    proofs.
- Deliverables:
  - Reusable uniqueness and closure proof tools for later measure modules.
- Acceptance criteria:
  - Pi-lambda and monotone-class statements do not import integration or
    product-measure APIs.
  - The theorem-card inventory marks these tools as prerequisites for
    extension and product-measure tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.MonotoneClass`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.MonotoneClass --verified-cache authoring`
  - `rg -n "pi-lambda|monotone class|Dynkin" proofs`
- Completion notes:
  - Completed with `Proofs.Ai.Measure.MonotoneClass`, generated source,
    certificate, metadata, replay, and AI theorem-index entries.
  - The module provides `PiSystem`, `LambdaSystem`, `MonotoneClass`,
    `DynkinPiLambdaRoute`, and `MonotoneClassRoute`.
  - The generated-subset theorems for Dynkin pi-lambda and monotone class
    routes apply `generated_sigma_algebra_minimal`, so they do not merely
    return a supplied law.
  - The module imports sigma-algebra/topology foundations only; integration,
    product-measure, Fubini, and Tonelli APIs are not imported.

### MEA-T05 Add Product Sigma Algebra And Measurable-Space Interface

- Status: Completed (2026-06-08)
- Depends on: `MEA-T03`
- Areas: `Proofs/Ai/Measure/MeasurableSpace/`, `Proofs/Ai/Measure/Product/`
- Tasks:
  - Define measurable spaces as carriers equipped with sigma algebras.
  - Add product sigma algebra generated by measurable rectangles.
  - Add coordinate-map measurability statement hooks for later product
    measure and vector-valued measurability tasks.
- Deliverables:
  - Measurable-space and product-sigma statement modules.
- Acceptance criteria:
  - Product sigma algebra is independent of product measure.
  - Coordinate-map statements do not assume Fubini or Tonelli.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.MeasurableSpace`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Product.SigmaAlgebra`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.MeasurableSpace --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Product.SigmaAlgebra --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only`
- Completion notes:
  - Completed with `Proofs.Ai.Measure.MeasurableSpace` and
    `Proofs.Ai.Measure.Product.SigmaAlgebra`, including generated source,
    certificates, metadata, replay sidecars, and AI theorem-index entries.
  - `MeasurableSpace` packages sigma-algebra core evidence and proves empty
    set, complement, measurable preimage, and measurable-map composition
    statements from that evidence.
  - `Product.SigmaAlgebra` defines product rectangle seed evidence and proves
    product sigma-algebra core, rectangle inclusion, and minimality by reusing
    `GeneratedSigmaAlgebra`.
  - Coordinate-map measurability remains an explicit hook and does not assume
    product measure, Fubini, or Tonelli.

### MEA-T06 Create Measure-Space Core And Additivity Laws

- Status: Completed (2026-06-08)
- Depends on: `MEA-T02`, `MEA-T05`
- Areas: `Proofs/Ai/Measure/Basic/`
- Tasks:
  - Define measure spaces over measurable spaces.
  - Add null empty set, finite additivity, and countable additivity for
    disjoint families.
  - Keep extended-nonnegative value assumptions explicit if numeric support is
    still an interface.
- Deliverables:
  - Basic measure-space source and certificate artifacts.
- Acceptance criteria:
  - Countable additivity is ordinary structure or derived evidence, not a
    trusted checker primitive.
  - Additivity statements include disjointness hypotheses where required.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Basic`
- Completed with `Proofs.Ai.Measure.Basic`, including generated source,
  certificate, metadata, replay, and AI theorem-index sidecar artifacts. The
  module keeps numeric/extended-nonnegative support explicit and represents
  countable additivity as ordinary proof-corpus structure, not a checker
  primitive.

### MEA-T07 Prove Monotonicity, Subadditivity, And Difference Laws

- Status: Completed (2026-06-08)
- Depends on: `MEA-T06`
- Areas: `Proofs/Ai/Measure/Basic/`
- Tasks:
  - Prove monotonicity from measure additivity.
  - Prove finite and countable subadditivity.
  - Add difference formula and finite inclusion-exclusion statements with
    explicit finite-measure hypotheses.
- Deliverables:
  - Derived basic measure law certificates.
- Acceptance criteria:
  - Upper bounds and difference formulas state every finiteness premise.
  - No theorem assumes subadditivity as a primitive law if it is targeted as
    derived.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Basic`
- Completed in `Proofs.Ai.Measure.Basic` with certificate-backed
  monotonicity, binary and countable subadditivity, difference, and finite
  inclusion-exclusion statements. Subadditivity is derived from additivity plus
  explicit disjointization/order support, not assumed as a primitive measure
  field.

### MEA-T08 Add Measure Continuity, Completion, Restriction, And Measure Operations

- Status: Completed (2026-06-08)
- Depends on: `MEA-T07`
- Areas: `Proofs/Ai/Measure/Basic/`, `Proofs/Ai/Measure/Completion/`, `Proofs/Ai/Measure/Restriction/`
- Tasks:
  - Prove continuity from below.
  - Prove continuity from above with an explicit finite first-set premise.
  - Add finite, probability, and sigma-finite measure-space interfaces.
  - Add completion, null-subset measurability, restricted measures, subspace
    measures, measure sums, scalar multiples, and monotone measure limits as
    separate statement groups.
- Deliverables:
  - Continuity and measure-operation modules ready for extension and
    probability use.
- Acceptance criteria:
  - Upper continuity cannot be applied without its finite-measure premise.
  - Completion is marked `L1` until null-subset measurability is actually
    derived.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Completion`
  - `cargo run -p npa-proof-corpus -- --changed-only`
- Completed with `Proofs.Ai.Measure.Basic`, `Proofs.Ai.Measure.Completion`,
  and `Proofs.Ai.Measure.Restriction`. Basic adds continuity from below,
  upper continuity with an explicit finite first-set premise, finite /
  probability / sigma-finite measure interfaces, and measure-operation route
  statements; Completion keeps null-subset measurability behind an explicit
  `L1` hook; Restriction records restricted-measure agreement as source-free
  evidence.

### MEA-T09 Define Outer Measure And Caratheodory Measurability

- Status: Completed (2026-06-08)
- Depends on: `MEA-T02`, `MEA-T07`
- Areas: `Proofs/Ai/Measure/Outer/`, `Proofs/Ai/Measure/Caratheodory/`
- Tasks:
  - Define outer measure, monotonicity, and countable subadditivity statement
    shapes.
  - Define Caratheodory measurable sets by the split equality over arbitrary
    test sets.
  - Add theorem cards separating outer-measure laws from extension theorems.
- Deliverables:
  - Outer-measure and Caratheodory-measurability base modules.
- Acceptance criteria:
  - The measurable-set criterion is represented as structured theorem
    arguments, not as parser-level set notation.
  - The module does not assume premeasure extension.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Outer`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Caratheodory`
- Completed with `Proofs.Ai.Measure.Outer` and
  `Proofs.Ai.Measure.Caratheodory`. Outer records empty-set, monotonicity, and
  countable-subadditivity laws. Caratheodory records the split equality over an
  arbitrary test set using structured `SetIntersection` and `SetDifference`
  arguments, with no premeasure-extension premise. The theorem-card sidecar now
  separates outer-measure laws, Caratheodory measurability/sigma-algebra
  evidence, and extension interfaces.

### MEA-T10 Prove Caratheodory Measurable Sets Form A Sigma Algebra

- Status: Completed (2026-06-08)
- Depends on: `MEA-T09`
- Areas: `Proofs/Ai/Measure/Caratheodory/`
- Tasks:
  - Prove complement closure for Caratheodory measurable sets.
  - Prove countable-union closure and sigma-algebra formation.
  - Prove restriction of an outer measure to Caratheodory measurable sets is a
    measure.
- Deliverables:
  - Caratheodory sigma-algebra and restriction-measure certificates.
- Acceptance criteria:
  - The proof uses outer-measure subadditivity and the Caratheodory criterion,
    not an assumed sigma-algebra field.
  - Restriction-measure statements reuse the basic measure API from `MEA-T06`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Caratheodory`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Caratheodory`
- Completed with `caratheodory_complement_closed_from_split`,
  `caratheodory_countable_union_closed_from_subadditivity`,
  `caratheodory_sigma_algebra_core`, `caratheodory_measurable_space`, and
  `caratheodory_restricted_outer_measure_space`. The sigma-algebra certificate
  is built from explicit split/subadditivity closure evidence, not from an
  assumed `SigmaAlgebraCore`; the restricted measure certificate reuses
  `MeasureSpace`, `MeasureValueSupport`, `MeasureFiniteAdditivityLaw`, and
  `MeasureCountableAdditivityLaw` from `Proofs.Ai.Measure.Basic`.

### MEA-T11 Add Premeasure-Induced Outer Measure And Extension Interfaces

- Status: Completed (2026-06-08)
- Depends on: `MEA-T10`
- Areas: `Proofs/Ai/Measure/Extension/`
- Tasks:
  - Add premeasure, semiring, ring, and algebra statement interfaces.
  - Add outer measure induced by a premeasure.
  - Add Caratheodory extension and Hahn-Kolmogorov extension theorem
    interfaces, with construction evidence fields explicit.
- Deliverables:
  - Extension theorem statement or evidence-package modules.
- Acceptance criteria:
  - Any `L1` extension package states exactly what construction evidence is
    assumed.
  - Extension theorems do not assume the target extended measure under another
    name.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Extension`
  - `rg -n "Caratheodory|Hahn-Kolmogorov|premeasure" proofs/Proofs/Ai/Measure proofs/measure-theory-theorem-proof-roadmap-todo.md`
- Completed with `Proofs.Ai.Measure.Extension`. The module defines
  `SetSemiringInterface`, `SetRingInterface`, `SetAlgebraInterface`,
  `PremeasureStructure`, `PremeasureInducedOuterMeasure`,
  `CaratheodoryExtensionInterface`, and
  `HahnKolmogorovExtensionInterface`. Construction assumptions remain explicit
  fields such as premeasure cover construction evidence, Caratheodory extension
  construction evidence, and Hahn-Kolmogorov construction evidence; the
  extension measure is the induced `outerMeasure`, not a separate hidden target
  measure.

### MEA-T12 Prove Extension Uniqueness Under Sigma-Finiteness

- Status: Completed (2026-06-08)
- Depends on: `MEA-T04`, `MEA-T11`
- Areas: `Proofs/Ai/Measure/Extension/`
- Tasks:
  - State and prove extension uniqueness under sigma-finite generation
    hypotheses where prerequisites allow.
  - Use pi-lambda or monotone-class tools instead of duplicating uniqueness
    machinery.
  - Add theorem cards for uniqueness variants on semirings, rings, and
    algebras.
- Deliverables:
  - Extension uniqueness theorem interfaces or certificates.
- Acceptance criteria:
  - Sigma-finiteness hypotheses are explicit in every uniqueness theorem.
  - The uniqueness proof does not rely on product measure or integration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Extension`
  - `cargo run -p npa-proof-corpus -- --changed-only`
- Completed with `SigmaFiniteOnSeed`, `SigmaFiniteExtensionUniqueness`, and
  `extension_uniqueness_on_generated_from_pi_lambda`. The uniqueness certificate
  carries explicit left/right sigma-finite seed-cover hypotheses and uses
  `DynkinPiLambdaRoute` plus `dynkin_pi_lambda_generated_subset`; it does not
  import or depend on product measure or integration. The theorem-card sidecar
  now records semiring, ring, and algebra uniqueness variants.

### MEA-T13 Build Real-Line Lebesgue Outer Measure Interface

- Status: Pending
- Depends on: `MEA-T11`, analysis real-line foundations, topology Borel foundations
- Areas: `Proofs/Ai/Measure/Lebesgue/`
- Tasks:
  - Add interval-cover definition or construction evidence for Lebesgue outer
    measure.
  - Add statements that intervals have length as measure.
  - Add Borel-measurable and Lebesgue-measurable predicate distinction.
- Deliverables:
  - Real-line Lebesgue outer-measure statement module.
- Acceptance criteria:
  - The module does not assume Lebesgue measure before constructing or
    packaging it.
  - Borel and Lebesgue measurability remain distinct predicates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lebesgue`
  - `rg -n "Lebesgue|Borel|interval" proofs/Proofs/Ai/Measure proofs/measure-theory-theorem-proof-roadmap-todo.md`

### MEA-T14 Add Lebesgue Measure Examples, Invariance, And Completion

- Status: Pending
- Depends on: `MEA-T13`
- Areas: `Proofs/Ai/Measure/Lebesgue/`
- Tasks:
  - Add translation invariance and scaling statement shapes.
  - Add countable-set, rational-set, Cantor-set, and selected null-set
    examples as theorem cards or certificates.
  - Add Lebesgue measure as completion of Borel measure when completion
    prerequisites are ready.
- Deliverables:
  - Lebesgue measure example and invariance route.
- Acceptance criteria:
  - Translation and scaling are deterministic theorem statements.
  - Null-set examples do not require Vitali or choice-heavy nonmeasurable-set
    interfaces.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lebesgue`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### MEA-T15 Add Lebesgue-Stieltjes And Distribution-Function Measure Interfaces

- Status: Pending
- Depends on: `MEA-T11`, `MEA-T13`
- Areas: `Proofs/Ai/Measure/LebesgueStieltjes/`
- Tasks:
  - Add distribution-function and right-continuous monotone-function
    interfaces.
  - Add Lebesgue-Stieltjes construction and uniqueness statement packages.
  - Add atom-jump, Cantor distribution, and singular-continuous measure
    interfaces.
- Deliverables:
  - Lebesgue-Stieltjes route ready for probability distribution modules.
- Acceptance criteria:
  - Stieltjes construction is separate from the Lebesgue measure module.
  - Probability consumers can import distribution-measure statements without
    introducing a second measure API.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.LebesgueStieltjes`
  - `rg -n "Stieltjes|distribution function|atom|jump" proofs`

### MEA-T16 Define Measurable Functions And Basic Criteria

- Status: Pending
- Depends on: `MEA-T05`, ordered-real foundations
- Areas: `Proofs/Ai/Measure/MeasurableFunction/`
- Tasks:
  - Define measurable functions by measurable preimages.
  - Add real-valued measurability criteria by sublevel and superlevel sets.
  - Add continuous-to-Borel-measurable and indicator-function theorem
    statements.
- Deliverables:
  - Measurable-function base module.
- Acceptance criteria:
  - Measurability criteria name the codomain sigma algebra or Borel structure.
  - Indicator-function statements identify the measurable set being indicated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.MeasurableFunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.MeasurableFunction`

### MEA-T17 Prove Closure Of Measurable Functions Under Operations And Limits

- Status: Pending
- Depends on: `MEA-T16`
- Areas: `Proofs/Ai/Measure/MeasurableFunction/`
- Tasks:
  - Add closure under sum, product, quotient where defined, absolute value,
    positive part, negative part, max, and min.
  - Add countable sup, inf, limsup, liminf, and pointwise-limit measurability.
  - Add a.e. limit measurability with null-set assumptions explicit.
- Deliverables:
  - Closure and limit theorem certificates or interfaces.
- Acceptance criteria:
  - Quotient theorems include a nonzero-domain premise.
  - Limit theorems express countable operations through measurable-set
    closure facts.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.MeasurableFunction`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### MEA-T18 Add Composition, Product-Space, And Vector-Valued Measurability

- Status: Pending
- Depends on: `MEA-T05`, `MEA-T17`
- Areas: `Proofs/Ai/Measure/MeasurableFunction/`
- Tasks:
  - Add composition with Borel and measurable maps.
  - Add coordinate-map and product-space measurability statement shapes.
  - Add vector-valued componentwise measurability interfaces, marked as
    topology-dependent where needed.
- Deliverables:
  - Measurable-map composition and product-space bridge modules.
- Acceptance criteria:
  - Product-space measurability does not assume product measure.
  - Vector-valued statements identify their topology and component maps.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.MeasurableFunction`
  - `rg -n "composition|coordinate|componentwise" proofs`

### MEA-T19 Create Simple-Function API And Approximation Theorems

- Status: Pending
- Depends on: `MEA-T16`
- Areas: `Proofs/Ai/Measure/SimpleFunction/`
- Tasks:
  - Define simple functions by finite measurable partitions or finite sums of
    indicator functions.
  - Add nonnegative simple-function approximation from below.
  - Add bounded and cut-off simple-function approximation statements.
- Deliverables:
  - Simple-function source and certificate artifacts.
- Acceptance criteria:
  - Approximation from below is monotone when used by the integral route.
  - Simple-function representation changes do not alter theorem statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.SimpleFunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.SimpleFunction`

### MEA-T20 Define Simple And Nonnegative Lebesgue Integrals

- Status: Pending
- Depends on: `MEA-T08`, `MEA-T19`
- Areas: `Proofs/Ai/Measure/Integral/Simple/`, `Proofs/Ai/Measure/Integral/Nonnegative/`
- Tasks:
  - Define the integral of nonnegative simple functions.
  - Prove simple integral independence from representation or land an
    explicit `L1` representation-invariance package.
  - Define the nonnegative measurable integral as a supremum of simple
    integrals.
- Deliverables:
  - Simple and nonnegative integral construction modules.
- Acceptance criteria:
  - The nonnegative integral API distinguishes extended values from finite
    integrable functions.
  - Supremum and extended-real assumptions are explicit if numeric foundations
    are not yet derived.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral.Simple`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral.Nonnegative`

### MEA-T21 Define General Integral And Basic Integral Laws

- Status: Pending
- Depends on: `MEA-T20`
- Areas: `Proofs/Ai/Measure/Integral/Basic/`
- Tasks:
  - Define positive and negative parts, general integral, and integrability.
  - Prove positivity, monotonicity, linearity for integrable functions, and
    triangle inequality.
  - Add a.e.-equal invariance, null-set modification invariance, set-integral
    formulas, restriction-measure formulas, and truncation approximation.
- Deliverables:
  - General integral module ready for convergence theorems.
- Acceptance criteria:
  - General integrability requires finite positive and negative part
    integrals.
  - A.e. invariance uses null-set and monotonicity facts rather than an
    assumed equality principle.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Integral.Basic`

### MEA-T22 Prove Monotone Convergence And Beppo Levi

- Status: Pending
- Depends on: `MEA-T20`, `MEA-T21`
- Areas: `Proofs/Ai/Measure/Integral/Convergence/`
- Tasks:
  - Prove monotone convergence for nonnegative measurable functions.
  - Add Beppo Levi theorem as a named alias or derived variant.
  - Add theorem-card disambiguation from sequence monotone convergence.
- Deliverables:
  - Certificate-backed monotone convergence theorem route.
- Acceptance criteria:
  - The theorem is about Lebesgue integration, not sequence convergence.
  - Nonnegativity, measurability, and monotonicity hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral.Convergence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Integral.Convergence`

### MEA-T23 Prove Fatou And Dominated Convergence

- Status: Pending
- Depends on: `MEA-T22`
- Areas: `Proofs/Ai/Measure/Integral/Convergence/`
- Tasks:
  - Prove Fatou lemma using monotone convergence over increasing infima.
  - Prove dominated convergence theorem with an explicit integrable
    dominating function.
  - Add bounded convergence theorem as a finite-measure specialization or
    separate theorem.
- Deliverables:
  - Fatou, dominated convergence, and bounded convergence theorem
    certificates.
- Acceptance criteria:
  - Dominated convergence names the dominating function and integrability
    premise.
  - Bounded convergence states its finite-measure or boundedness assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Integral.Convergence`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### MEA-T24 Add Measure Convergence Modes And Subsequence Principles

- Status: Pending
- Depends on: `MEA-T21`, `MEA-T23`
- Areas: `Proofs/Ai/Measure/Convergence/`
- Tasks:
  - Define a.e. convergence, convergence in measure, `L^1` convergence, and
    `L^p` convergence statement shapes.
  - Prove or package implications from `L^p` convergence to convergence in
    measure and from a.e. convergence to convergence in measure on finite
    measure spaces.
  - Add subsequence principle, Riesz subsequence theorem, and Cauchy in
    measure interfaces.
- Deliverables:
  - Measure-convergence module usable by probability and `L^p` milestones.
- Acceptance criteria:
  - Every convergence statement names its measure space and convergence mode.
  - Finite-measure hypotheses are explicit in a.e.-to-measure statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Convergence`
  - `rg -n "convergence in measure|almost everywhere|subsequence" proofs`

### MEA-T25 Add Uniform Integrability, Vitali, Egorov, And Lusin Interfaces

- Status: Pending
- Depends on: `MEA-T23`, `MEA-T24`
- Areas: `Proofs/Ai/Measure/UniformIntegrability/`
- Tasks:
  - Define uniform integrability and de la Vallee-Poussin criterion statement
    shapes.
  - Add Vitali convergence theorem and Scheffe lemma interfaces.
  - Add Egorov and Lusin theorem interfaces with finite-measure and topology
    assumptions explicit.
- Deliverables:
  - Late convergence theorem interface module.
- Acceptance criteria:
  - Vitali does not replace dominated convergence in dependencies.
  - Egorov and Lusin statements expose finite-measure and topological
    prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.UniformIntegrability`
  - `rg -n "Vitali|Egorov|Lusin|uniform integrability" proofs`

### MEA-T26 Construct Product Measures

- Status: Pending
- Depends on: `MEA-T05`, `MEA-T11`, `MEA-T21`
- Areas: `Proofs/Ai/Measure/Product/`
- Tasks:
  - Add product-measure existence and uniqueness for measurable rectangles.
  - Add finite, sigma-finite, probability, finite-product, and countable
    product measure interfaces.
  - Add Kolmogorov extension theorem as a late interface, not as a dependency
    for binary product measure.
- Deliverables:
  - Product-measure module with explicit sigma-finiteness and uniqueness
    assumptions.
- Acceptance criteria:
  - Product measure depends on product sigma algebra, not the reverse.
  - Sigma-finiteness hypotheses are explicit where uniqueness requires them.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Product`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Product`

### MEA-T27 Add Section Measurability And Cavalieri Principles

- Status: Pending
- Depends on: `MEA-T18`, `MEA-T26`
- Areas: `Proofs/Ai/Measure/Section/`
- Tasks:
  - Define sections of sets and functions in product spaces.
  - Prove or package measurable-section, section-measure measurability, and
    section-integral measurability statements.
  - Add Cavalieri principle statement.
- Deliverables:
  - Section module ready for Tonelli and Fubini.
- Acceptance criteria:
  - Section measurability does not assume Fubini.
  - Section-integral statements name the product measure and section measure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Section`
  - `rg -n "section|Cavalieri" proofs`

### MEA-T28 Prove Tonelli Theorem

- Status: Pending
- Depends on: `MEA-T22`, `MEA-T26`, `MEA-T27`
- Areas: `Proofs/Ai/Measure/Fubini/`
- Tasks:
  - Prove Tonelli for indicator functions and simple functions.
  - Extend Tonelli to nonnegative measurable functions by monotone
    convergence.
  - Add nonnegative repeated-integral theorem names.
- Deliverables:
  - Tonelli theorem certificates.
- Acceptance criteria:
  - Tonelli is proved before Fubini.
  - Nonnegative measurability and product-measure hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Fubini`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Fubini`

### MEA-T29 Prove Fubini And Product Null-Set Theorems

- Status: Pending
- Depends on: `MEA-T21`, `MEA-T28`
- Areas: `Proofs/Ai/Measure/Fubini/`
- Tasks:
  - Prove Fubini for integrable functions by applying Tonelli to positive and
    negative parts.
  - Add order-exchange theorem for iterated integrals.
  - Add product-space null-set and a.e. section theorems.
  - Add convolution and kernel-composition formula interfaces as late aliases.
- Deliverables:
  - Fubini theorem and a.e. section theorem modules.
- Acceptance criteria:
  - Integrability and sigma-finiteness assumptions are explicit.
  - Convolution and kernel-composition formulas remain interfaces until the
    needed algebra and function-space prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Fubini`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`

### MEA-T30 Add Pushforward Measure And Distribution Measure Formulas

- Status: Pending
- Depends on: `MEA-T16`, `MEA-T21`
- Areas: `Proofs/Ai/Measure/Pushforward/`, `Proofs/Ai/Measure/Distribution/`
- Tasks:
  - Define image measure and pushforward measure for measurable maps.
  - Prove the pushforward integration formula.
  - Add distribution measure and probability-variable specialization
    statement hooks.
- Deliverables:
  - Pushforward and distribution measure modules.
- Acceptance criteria:
  - Pushforward formulas require measurability of the map.
  - Probability aliases do not introduce a separate probability-measure API.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Pushforward`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Distribution`

### MEA-T31 Add Measure-Preserving And Elementary Change-Of-Variables Statements

- Status: Pending
- Depends on: `MEA-T14`, `MEA-T30`
- Areas: `Proofs/Ai/Measure/ChangeOfVariables/`
- Tasks:
  - Add measurable isomorphism and measure-preserving transformation theorem
    statements.
  - Add translation and scaling as change-of-variables examples.
  - Add linear transformation formula for Lebesgue measure when determinant
    prerequisites are present.
- Deliverables:
  - Elementary change-of-variables interface module.
- Acceptance criteria:
  - Every change-of-variables theorem names its regularity and
    nonsingularity assumptions.
  - Translation and scaling statements reuse the Lebesgue invariance route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.ChangeOfVariables`
  - `rg -n "change of variables|measure-preserving|translation|scaling" proofs`

### MEA-T32 Add Differentiable Change-Of-Variables And Disintegration Interfaces

- Status: Pending
- Depends on: `MEA-T30`, `MEA-T31`, analysis derivative and inverse-function foundations
- Areas: `Proofs/Ai/Measure/ChangeOfVariables/`
- Tasks:
  - Add differentiable change-of-variables, polar coordinate, and spherical
    coordinate interfaces.
  - Add coarea, area, and Sard-related interfaces.
  - Add density transformation and disintegration theorem interfaces with
    Radon-Nikodym dependencies explicit.
- Deliverables:
  - Late change-of-variables and disintegration statement package.
- Acceptance criteria:
  - Coarea, area, Sard, and disintegration statements are marked as late
    interfaces until geometric and topological prerequisites exist.
  - Density transformation statements point to the Radon-Nikodym route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.ChangeOfVariables`
  - `rg -n "coarea|disintegration|Radon-Nikodym|density transformation" proofs`

### MEA-T33 Define Signed Measures And Hahn Decomposition

- Status: Pending
- Depends on: `MEA-T08`, `MEA-T21`
- Areas: `Proofs/Ai/Measure/Signed/`
- Tasks:
  - Define signed measures, positive sets, and negative sets.
  - Add Hahn decomposition theorem and uniqueness modulo null sets.
  - Keep all existence evidence explicit if the proof starts as `L1`.
- Deliverables:
  - Signed-measure base module and Hahn decomposition route.
- Acceptance criteria:
  - Hahn decomposition is not assumed by Jordan decomposition under another
    name.
  - Positive and negative set definitions are reusable by later tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Signed`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Signed`

### MEA-T34 Add Jordan Decomposition And Total Variation

- Status: Pending
- Depends on: `MEA-T33`
- Areas: `Proofs/Ai/Measure/Signed/`
- Tasks:
  - Add Jordan decomposition theorem.
  - Define positive variation, negative variation, and total variation.
  - Prove or package minimality of total variation.
  - Add integration with respect to signed measures.
- Deliverables:
  - Jordan and total-variation theorem modules.
- Acceptance criteria:
  - Total variation statements are reusable by functional-analysis duality.
  - Integration with signed measures depends on the established integral API.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Signed`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### MEA-T35 Prove Radon-Nikodym And Lebesgue Decomposition

- Status: Pending
- Depends on: `MEA-T21`, `MEA-T34`
- Areas: `Proofs/Ai/Measure/RadonNikodym/`, `Proofs/Ai/Measure/Decomposition/`
- Tasks:
  - Define absolute continuity and singularity.
  - Prove or package Radon-Nikodym theorem and uniqueness of derivative.
  - Add density representation, Radon-Nikodym chain rule, and change-of-measure
    formulas.
  - Prove or package Lebesgue decomposition theorem and uniqueness.
- Deliverables:
  - Radon-Nikodym and Lebesgue decomposition modules.
- Acceptance criteria:
  - No decomposition theorem assumes the decomposition as a law package unless
    the target level is explicitly `L1`.
  - Absolute-continuity and singularity hypotheses are exposed in theorem
    statements.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.RadonNikodym`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Decomposition`
  - `./scripts/check-corpus-authoring.sh`

### MEA-T36 Add Complex Measure Decomposition Interfaces

- Status: Pending
- Depends on: `MEA-T34`, `MEA-T35`
- Areas: `Proofs/Ai/Measure/Complex/`
- Tasks:
  - Define complex measures and complex total variation.
  - Add polar decomposition, complex Radon-Nikodym theorem, and complex
    Lebesgue decomposition interfaces.
  - Add complex-measure integral statement hooks for Fourier and
    functional-analysis consumers.
- Deliverables:
  - Complex measure interface module.
- Acceptance criteria:
  - Complex-measure facts do not block the signed-measure core route.
  - Complex scalar assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Complex`
  - `rg -n "complex measure|polar decomposition|Fourier" proofs`

### MEA-T37 Add Lebesgue Regularity Theorems

- Status: Pending
- Depends on: `MEA-T14`, topology compactness foundations
- Areas: `Proofs/Ai/Measure/Lebesgue/Regularity/`
- Tasks:
  - Add outer regularity, inner regularity, and Borel/Lebesgue regularity.
  - Add approximation by open, closed, `G_delta`, and `F_sigma` sets.
  - Add measurable sets as Borel sets modulo null sets.
- Deliverables:
  - Lebesgue regularity module.
- Acceptance criteria:
  - Topological assumptions are explicit.
  - Regularity does not become a prerequisite for basic integration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lebesgue.Regularity`
  - `rg -n "regularity|G_delta|F_sigma|modulo null" proofs`

### MEA-T38 Add Covering And Maximal-Function Interfaces

- Status: Pending
- Depends on: `MEA-T37`, metric and Euclidean prerequisites
- Areas: `Proofs/Ai/Measure/Covering/`
- Tasks:
  - Add Vitali covering theorem interface.
  - Add Hardy-Littlewood maximal inequality and maximal theorem interfaces.
  - Record Euclidean or metric hypotheses required by each covering theorem.
- Deliverables:
  - Covering and maximal-function interface module.
- Acceptance criteria:
  - Covering lemmas state their metric or Euclidean assumptions.
  - Maximal-function interfaces do not assume Lebesgue differentiation.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Covering`
  - `rg -n "Vitali covering|Hardy-Littlewood|maximal" proofs`

### MEA-T39 Add Density, Differentiation, And Riemann-Lebesgue Bridges

- Status: Pending
- Depends on: `MEA-T38`, analysis Riemann integration foundations
- Areas: `Proofs/Ai/Measure/Lebesgue/Density/`, `Proofs/Ai/Measure/Lebesgue/Differentiation/`
- Tasks:
  - Add Lebesgue density theorem and Lebesgue differentiation theorem.
  - Add differentiation theorems for absolutely continuous, bounded-variation,
    and monotone functions as interfaces.
  - Add fundamental theorem of calculus for the Lebesgue integral and
    Lebesgue criterion for Riemann integrability.
- Deliverables:
  - Density and differentiation modules.
- Acceptance criteria:
  - Riemann/Lebesgue bridge statements name both APIs and assumptions.
  - Differentiation theorems do not back-propagate into basic integral
    dependencies.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lebesgue.Density`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lebesgue.Differentiation`

### MEA-T40 Define Lp Spaces And Norm Laws

- Status: Pending
- Depends on: `MEA-T21`, normed-space foundations
- Areas: `Proofs/Ai/Measure/Lp/Basic/`
- Tasks:
  - Define `L^p` spaces, a.e. equivalence classes, and `L^p` norms.
  - Prove or package norm laws and finite-measure inclusion statements.
  - Define essential supremum and `L^infinity` statement shapes.
- Deliverables:
  - `L^p` basic module.
- Acceptance criteria:
  - `L^p` values are quotient or equivalence-class aware.
  - Theorems distinguish `p` finite, `p` equal to one, and `p` equal to
    infinity.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lp.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Measure.Lp.Basic`

### MEA-T41 Prove Core Integral Inequalities

- Status: Pending
- Depends on: `MEA-T21`, `MEA-T40`
- Areas: `Proofs/Ai/Measure/Lp/Inequality/`
- Tasks:
  - Prove Markov and Chebyshev inequalities.
  - Prove Jensen and Young inequalities or add explicit interfaces if convex
    prerequisites are missing.
  - Prove Holder, Cauchy-Schwarz, and Minkowski in dependency order.
- Deliverables:
  - Core `L^p` and integral inequality certificates.
- Acceptance criteria:
  - Holder precedes Minkowski where Minkowski depends on Holder.
  - Jensen states convexity and integrability assumptions explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lp.Inequality`
  - `cargo run -p npa-proof-corpus -- --changed-only`

### MEA-T42 Prove Riesz-Fischer And L2 Hilbert Interfaces

- Status: Pending
- Depends on: `MEA-T23`, `MEA-T40`, `MEA-T41`
- Areas: `Proofs/Ai/Measure/Lp/Basic/`
- Tasks:
  - Prove Riesz-Fischer completeness for `L^p`.
  - Add `L^2` Hilbert-space structure.
  - Add projection theorem, Bessel, Parseval, separability, and essential
    supremum interfaces where prerequisites are not ready.
- Deliverables:
  - `L^p` completeness and `L^2` Hilbert bridge.
- Acceptance criteria:
  - Completeness proof uses Cauchy subsequences, a.e. convergence, and Fatou
    or explicitly marks missing evidence.
  - `L^2` interfaces coordinate with existing Hilbert-space spectral modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lp.Basic`
  - `./scripts/check-corpus-authoring.sh`

### MEA-T43 Add Lp Duality, Reflexivity, And Interpolation Interfaces

- Status: Pending
- Depends on: `MEA-T35`, `MEA-T42`, functional-analysis foundations
- Areas: `Proofs/Ai/Measure/Lp/Duality/`, `Proofs/Ai/Measure/Lp/Interpolation/`
- Tasks:
  - Add `L^p` duality for one less than `p` and finite `p`.
  - Add `L^1` and `L^infinity` duality interfaces.
  - Add reflexivity, Dunford-Pettis, Riesz-Thorin, Marcinkiewicz,
    Hausdorff-Young, Plancherel, Sobolev, and Rellich-Kondrachov interfaces.
- Deliverables:
  - Late `L^p` duality and interpolation interface modules.
- Acceptance criteria:
  - Duality statements name Radon-Nikodym and functional-analysis
    dependencies.
  - Fourier and Sobolev interfaces do not claim derived status before their
    analysis prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lp.Duality`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Lp.Interpolation`

### MEA-T44 Add Borel And Radon Measure Interfaces

- Status: Pending
- Depends on: `MEA-T13`, `MEA-T37`, topology roadmap foundations
- Areas: `Proofs/Ai/Measure/Borel/`, `Proofs/Ai/Measure/Radon/`
- Tasks:
  - Define Borel measures, regular Borel measures, and Radon measures.
  - Add Radon measures on locally compact Hausdorff spaces.
  - Add Riesz-Markov-Kakutani representation interface.
- Deliverables:
  - Borel and Radon measure modules.
- Acceptance criteria:
  - Theorems name Borel, regular, Radon, locally compact, and Hausdorff
    assumptions separately.
  - Representation interfaces do not assume functional-analysis duality under
    another name.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Borel`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Radon`

### MEA-T45 Add Tightness And Weak-Convergence Interfaces

- Status: Pending
- Depends on: `MEA-T44`
- Areas: `Proofs/Ai/Measure/WeakConvergence/`
- Tasks:
  - Define tightness and weak convergence of measures.
  - Add Ulam, Prokhorov, Portmanteau, weak-convergence characterization, and
    Skorokhod representation interfaces.
  - Add Wasserstein and vague-convergence interfaces as late hooks.
- Deliverables:
  - Weak-convergence measure module.
- Acceptance criteria:
  - Polish, Radon, tightness, and topology assumptions are explicit.
  - Probability and statistics consumers import these statements instead of
    defining parallel weak-convergence APIs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.WeakConvergence`
  - `rg -n "tightness|Portmanteau|Prokhorov|Skorokhod|Wasserstein" proofs`

### MEA-T46 Add Analytic-Set And Measurable-Selection Interfaces

- Status: Pending
- Depends on: `MEA-T44`, topology roadmap foundations
- Areas: `Proofs/Ai/Measure/Selection/`
- Tasks:
  - Add analytic sets, Suslin theorem, Lusin separation, and measurable
    selection interfaces.
  - Add standard Borel and Lusin-space interfaces.
  - Record which topology prerequisites remain external.
- Deliverables:
  - Selection and descriptive-measurability interface module.
- Acceptance criteria:
  - Selection theorems state standard Borel, Polish, or Lusin assumptions.
  - Interfaces do not become prerequisites for basic measure construction.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Selection`
  - `rg -n "Suslin|selection|standard Borel|Lusin" proofs`

### MEA-T47 Add Topological Measure Packaging Split

- Status: Pending
- Depends on: `MEA-T44`, `MEA-T45`, `MEA-T46`
- Areas: `Proofs/Ai/Measure/Borel/`, `Proofs/Ai/Measure/Radon/`, `Proofs/Ai/Measure/WeakConvergence/`
- Tasks:
  - Audit topological measure modules for dependency cycles.
  - Split Borel/Radon regularity, weak convergence, and selection interfaces
    if one module has become too broad.
  - Add theorem-card aliases for statistics asymptotics and probability
    weak-convergence consumers.
- Deliverables:
  - Stable topological-measure module split.
- Acceptance criteria:
  - No weak-convergence theorem imports statistics-specific APIs.
  - The module graph keeps regularity, weak convergence, and selection
    dependencies acyclic.
- Verification:
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "STAT-|weak convergence|Radon|Selection" proofs/measure-theory-theorem-proof-roadmap-todo.md proofs`

### MEA-T48 Add Probability-Space And Random-Variable Bridges

- Status: Pending
- Depends on: `MEA-T08`, `MEA-T30`, statistics roadmap finite probability foundations
- Areas: `Proofs/Ai/Measure/ProbabilityBridge/`
- Tasks:
  - Add probability spaces as finite measure spaces of total mass one.
  - Add random variables as measurable functions.
  - Add expectation as Lebesgue integral and LOTUS aliases from pushforward
    integration.
- Deliverables:
  - Measure-to-probability bridge module.
- Acceptance criteria:
  - Probability does not introduce a second measure API.
  - Random-variable statements reuse measurable-function definitions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.ProbabilityBridge`
  - `rg -n "ProbabilityBridge|LOTUS|random variable" proofs`

### MEA-T49 Add Conditional Expectation And Regular Conditional Probability Interfaces

- Status: Pending
- Depends on: `MEA-T35`, `MEA-T44`
- Areas: `Proofs/Ai/Measure/ConditionalExpectation/`
- Tasks:
  - Add conditional expectation existence and uniqueness via Radon-Nikodym.
  - Add tower, pull-out, monotonicity, and conditional Jensen properties.
  - Add regular conditional probability and disintegration interfaces.
- Deliverables:
  - Conditional expectation measure bridge.
- Acceptance criteria:
  - Conditional expectation statements include the conditioning sigma algebra.
  - Regular conditional probability remains an interface until standard Borel
    prerequisites are present.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.ConditionalExpectation`
  - `rg -n "conditional expectation|tower|disintegration" proofs`

### MEA-T50 Add Extension And Borel-Cantelli Probability Bridges

- Status: Pending
- Depends on: `MEA-T12`, `MEA-T26`, `MEA-T48`
- Areas: `Proofs/Ai/Measure/ProbabilityBridge/`
- Tasks:
  - Add Kolmogorov and Ionescu-Tulcea extension interfaces as probability
    aliases over measure extension machinery.
  - Add Borel-Cantelli lemmas.
  - Add convergence in probability and distribution aliases from measure
    convergence and weak-convergence modules.
- Deliverables:
  - Probability extension and convergence bridge interfaces.
- Acceptance criteria:
  - Extension interfaces name the measure extension theorem they depend on.
  - Convergence aliases do not duplicate measure-convergence definitions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.ProbabilityBridge`
  - `rg -n "Kolmogorov|Ionescu|Borel-Cantelli|convergence in probability" proofs`

### MEA-T51 Add Martingale And Ergodic Interfaces

- Status: Pending
- Depends on: `MEA-T40`, `MEA-T45`, `MEA-T49`
- Areas: `Proofs/Ai/Measure/Martingale/`, `Proofs/Ai/Measure/Ergodic/`
- Tasks:
  - Add martingale definition, Doob inequalities, optional stopping,
    upcrossing, and martingale convergence interfaces.
  - Add measure-preserving transformations, ergodicity, Poincare recurrence,
    Birkhoff theorem, and von Neumann ergodic theorem interfaces.
  - Coordinate theorem-card aliases with topology and statistics roadmaps.
- Deliverables:
  - Martingale and ergodic interface modules.
- Acceptance criteria:
  - Martingale statements state filtration, integrability, and conditioning
    assumptions.
  - Ergodic statements state measure-preserving and invariant-set assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Martingale`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Ergodic`

### MEA-T52 Define Hausdorff Measure And Dimension Interfaces

- Status: Pending
- Depends on: `MEA-T11`, `MEA-T37`, topology and metric foundations
- Areas: `Proofs/Ai/Measure/Geometric/`
- Tasks:
  - Define Hausdorff measure and Hausdorff dimension statement shapes.
  - Add basic Hausdorff-measure properties.
  - Add packing measure interface.
- Deliverables:
  - Geometric measure base interface.
- Acceptance criteria:
  - Hausdorff statements state metric-space assumptions.
  - Geometric measure theory remains late and does not block basic
    integration.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Geometric`
  - `rg -n "Hausdorff|packing" proofs`

### MEA-T53 Add Projection, Area, Coarea, And Rectifiability Interfaces

- Status: Pending
- Depends on: `MEA-T32`, `MEA-T38`, `MEA-T52`
- Areas: `Proofs/Ai/Measure/Geometric/`
- Tasks:
  - Add Frostman, Marstrand, and Besicovitch-Federer projection interfaces.
  - Add area and coarea formula aliases coordinated with change of variables.
  - Add Rademacher, Kirszbraun, rectifiability, finite-perimeter, BV
    compactness, isoperimetric, and Preiss theorem interfaces.
- Deliverables:
  - Late geometric measure theory interface module.
- Acceptance criteria:
  - Projection and rectifiability statements expose Euclidean, metric, and
    differentiability assumptions.
  - Area and coarea aliases point to the change-of-variables route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Geometric`
  - `rg -n "Frostman|Marstrand|coarea|rectifiability|Preiss" proofs`

### MEA-T54 Add Measure Algebra Core

- Status: Pending
- Depends on: `MEA-T08`
- Areas: `Proofs/Ai/Measure/Algebra/`
- Tasks:
  - Define measure algebra by quotienting measurable sets by null symmetric
    difference.
  - Add Boolean algebra relation and complete Boolean algebra interfaces.
  - Add atomic and nonatomic decomposition statement shapes.
- Deliverables:
  - Measure algebra base module.
- Acceptance criteria:
  - Quotienting by null sets is explicit.
  - Boolean algebra interfaces do not alter the kernel's trusted logic.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Algebra`
  - `rg -n "measure algebra|Boolean|atomic|nonatomic" proofs`

### MEA-T55 Add Abstract Measure-Space Classification Interfaces

- Status: Pending
- Depends on: `MEA-T44`, `MEA-T54`
- Areas: `Proofs/Ai/Measure/Algebra/`
- Tasks:
  - Add Maharam theorem, Lebesgue-space isomorphism, standard probability-space
    classification, Stone, and Loomis-Sikorski interfaces.
  - Mark all representation and classification evidence as external until
    prerequisites are certified.
  - Add theorem cards preventing these interfaces from being used as basic
    measure-construction assumptions.
- Deliverables:
  - Abstract measure-space classification interface module.
- Acceptance criteria:
  - Classification theorems are late interfaces and cannot be imported by
    `MEA-T01` through `MEA-T29`.
  - Representation assumptions are named in statement records.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Measure.Algebra`
  - `rg -n "Maharam|Loomis|classification|Stone" proofs`

### MEA-T56 Prepare Measure-Theory Packaging And Promotion

- Status: Pending
- Depends on: stable contiguous `MEA-Txx` batches
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/*`, `develop/npa-mathlib-next-closure-roadmap.md`
- Tasks:
  - Identify the smallest stable measure-theory closure batch.
  - Verify source-free modules, changed proof-corpus artifacts, package
    metadata, theorem indexes, axiom report, package lock, and publish plan.
  - Update closure-roadmap notes only after a separate closure audit selects a
    public promotion batch.
- Deliverables:
  - Promotion-ready measure-theory closure candidate or explicit deferral
    notes.
- Acceptance criteria:
  - The axiom report does not grow unexpectedly.
  - Package verification runs before downstream roadmaps rely on public measure
    modules.
  - Public closure ordering remains controlled by
    `develop/npa-mathlib-next-closure-roadmap.md`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

---

## Review Findings And Resolutions

Review passes against
`proofs/measure-theory-theorem-proof-roadmap.md`,
`proofs/analysis-theorem-proof-roadmap-todo.md`,
`proofs/statistics-theorem-proof-roadmap.md`, and the current proof corpus
produced these findings and resolutions:

| Finding | Resolution |
| --- | --- |
| A single task per roadmap milestone would make `MEA-01`, `MEA-07`, `MEA-08`, `MEA-10`, `MEA-12`, and later topology/probability groups too broad for one implementation agent. | Split every broad roadmap milestone into focused `MEA-Txx` tasks with independent verification commands. |
| Earlier docs said no concrete `Proofs.Ai.Measure.*` tree existed, so tasks could not assume those modules were already present. | `MEA-T01` created `Proofs.Ai.Measure.Inventory`; later tasks still must not assume sigma algebra, basic measure, outer measure, extension, or integral measure modules beyond that namespace entry point. |
| The abstract integration route and real-line Lebesgue construction branch have different dependency pressure. | Put Lebesgue-measure construction after the abstract extension route, but kept it as a branch that can be scheduled after the first convergence batch when corpus needs dictate. |
| Late probability, martingale, weak-convergence, geometric, and measure-algebra theorems could be mistaken for basic-measure prerequisites. | Marked those tasks as `L1` interfaces first and added acceptance criteria preventing imports into the basic measure route. |
| Verification commands must not imply full package gates are required for every local authoring task. | Added the local authoring loop and reserved package/full gates for promotion, compatibility, or package-wide changes. |
| The source roadmap's initial execution queue used `MEA-T01` for theorem cards while this task document uses `MEA-T00`. | Updated the source roadmap queue to match `MEA-T00` through `MEA-T10` and clarified the branch point after Caratheodory. |
| Analysis roadmap tasks still referred to a coarse `Proofs.Ai.Measure.Construction` module that the detailed measure todo did not create. | Split the analysis references into `Proofs.Ai.Measure.Outer`, `Proofs.Ai.Measure.Caratheodory`, and `Proofs.Ai.Measure.Extension`, and documented the compatibility split here. |
| `MEA-T52` depended on task `MEA-T11` only, while the source roadmap's `MEA-15` dependency points to the later regularity/differentiation milestone `MEA-11`. | Added `MEA-T37` as the explicit regularity-route dependency while keeping `MEA-T11` for outer-measure construction support. |

No open findings remain after this pass.

## Validation Checklist

Use this checklist after editing the task document:

```sh
git diff --check
rg -n "TO""DO|TB""D|未""定|PLACE""HOLDER" proofs/measure-theory-theorem-proof-roadmap-todo.md
rg -n "MEA-T00|MEA-T56|Proofs.Ai.Measure|ANA-T24|ANA-T26|Radon-Nikodym|Tonelli|Fubini" \
  proofs/measure-theory-theorem-proof-roadmap.md \
  proofs/measure-theory-theorem-proof-roadmap-todo.md \
  proofs/analysis-theorem-proof-roadmap-todo.md \
  proofs/statistics-theorem-proof-roadmap.md \
  proofs/statistics-theorem-proof-roadmap-todo.md
```
