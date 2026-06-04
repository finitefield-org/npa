# Fermat's Last Theorem Proof Phases Todo

Source: `proofs/fermats-last-theorem-proof-phases.md`

This task breakdown turns the FLT proof phase document into implementation
milestones for the proof corpus. It deliberately treats FLT as both a final
theorem project and a theorem-library growth project.

## Scope

対象:

```text
- `Proofs.Ai.NumberTheory.*` elementary arithmetic, divisibility, primes, and FLT reduction routes
- `Proofs.Ai.Algebra.*` reusable algebra, field, ideal, quotient, polynomial, matrix, and finite-object routes
- `Proofs.Ai.EllipticCurve.*` elliptic-curve and Frey-curve APIs
- `Proofs.Ai.ModularForms.*`, `Proofs.Ai.GaloisRepresentation.*`, and `Proofs.Ai.Modularity.*`
- bridge-backed early smoke layers whose axiom dependencies remain explicit
- final source-free, bridge-free release audit for `Proofs.Ai.NumberTheory.Flt.Final.fermat_last_theorem`
```

非対象:

```text
- `FLT`, `Ribet`, `Wiles`, modularity, or level lowering as kernel primitives
- accepting papers, source scripts, tactic logs, AI traces, theorem search, or metadata as proof evidence
- importing a theorem-shaped axiom into the final FLT certificate
- adding typeclass search, implicit arguments, overloaded notation, or tactics to the trusted core
- hiding quotient, choice, completion, or bridge dependencies from axiom and core-feature reports
```

信頼境界:

```text
信頼しない:
  source.npa / replay.json / meta.json / theorem index / roadmap / todo / AI proof candidate

信頼する:
  canonical .npcert
  deterministic export_hash / certificate_hash / axiom_report_hash
  kernel / certificate verifier verdict
  source-free independent checker verdict
```

## Library Growth Rule

Each domain implementation milestone must produce a reusable theorem-library
surface, not only the narrow theorem needed by the final FLT route.

Rules:

- Every mathematical domain milestone must export at least one domain-oriented
  theorem, definition, or law package that can be used independently of FLT.
- Governance, bridge-policy, smoke, and release-audit milestones are exempt
  from exporting a new domain theorem package, but they must document how they
  protect the reusable library surface and must route mathematical dependencies
  through reusable modules.
- `Proofs.Ai.NumberTheory.Flt.*` glue modules may depend on reusable
  `NumberTheory`, `Algebra`, `EllipticCurve`, `ModularForms`,
  `GaloisRepresentation`, and `Modularity` modules, but reusable modules must
  not depend on final FLT glue.
- If a theorem is introduced only because the Wiles/Ribet route needs it, the
  milestone must also add nearby standard lemmas that make the API usable for
  other number-theory or geometry proofs.
- Bridge axiom declarations use the `Flt.BridgeAxiom.*` prefix and live only in
  clearly named development modules such as `Proofs.Ai.NumberTheory.Flt.Bridge`
  or another explicitly bridge-marked namespace. They are never accepted as
  library facts.
- The README, package manifest, generated theorem index, and axiom report must
  make the reusable surface discoverable as metadata. These files remain
  untrusted sidecars; proof acceptance still comes only from canonical
  certificates and checker verdicts.

## Current Implementation Facts

Existing proof-corpus assets useful for this plan:

```text
Proofs.Ai.Algebra.AbstractField
Proofs.Ai.Algebra.AbstractRing
Proofs.Ai.Algebra.AbstractRingFirstIsoBase
Proofs.Ai.Algebra.AbstractRingFirstIso
Proofs.Ai.Algebra.AbstractRingChineseRemainder
Proofs.Ai.Algebra.AbstractUfdPrimeFactorization
Proofs.Ai.Algebra.AbstractHilbertBasisTheorem
Proofs.Ai.Algebra.AbstractHilbertNullstellensatz
Proofs.Ai.Algebra.AbstractKrullTheorem
Proofs.Ai.Algebra.AbstractOrderedField
Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem
Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem
```

Missing or not yet materialized for FLT:

```text
Proofs.Ai.NumberTheory.*
Proofs.Ai.EllipticCurve.*
Proofs.Ai.ModularForms.*
Proofs.Ai.GaloisRepresentation.*
Proofs.Ai.Modularity.*
```

Implementation areas likely touched by most milestones:

```text
tools/proof-corpus/src/main.rs
proofs/Proofs/Ai/
proofs/README.md
proofs/manifest.toml
proofs/npa-package.toml
proofs/generated/package-lock.json
proofs/generated/axiom-report.json
proofs/generated/theorem-index.json
proofs/generated/ai-theorem-index.json
```

## Milestone Order

```text
FLT-00 -> FLT-01
FLT-01 -> NT-01 -> NT-02 -> NT-03 -> NT-04 -> NT-05
FLT-01 -> ALG-01 -> ALG-02 -> ALG-03 -> ALG-04
NT-05 + ALG-04 -> EC-01 -> EC-02 -> EC-03 -> EC-04
ALG-04 -> MF-01 -> MF-02
ALG-04 -> GAL-01 -> GAL-02
MF-02 + GAL-02 + EC-04 -> RIB-01 -> RIB-02 -> RIB-03
ALG-04 + MF-02 + GAL-02 -> MOD-01 -> MOD-02 -> MOD-03 -> MOD-04
RIB-03 + MOD-04 + NT-05 + EC-04 -> FINAL-01 -> FINAL-02 -> FINAL-03 -> REL-01
```

## Milestones

### FLT-00 Project Contract, Statement Freeze, And Library Charter

- Status: Completed
- Depends on: None
- Inputs:
  - `proofs/fermats-last-theorem-proof-phases.md`
  - `AGENTS.md`
  - `develop/proof-corpus-ai-workflow.md`
  - `proofs/README.md`
- Code or documentation areas:
  - `proofs/README.md`
  - `proofs/Proofs/Ai/NumberTheory/Flt/Statement/`
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
- Tasks:
  - Freeze the exact NPA surface statement for `fermat_last_theorem`.
  - Create theorem names for natural-number, positive-natural, and integer variants.
  - Document the library growth rule in proof-corpus documentation.
  - Record which definitions are reusable library APIs and which are FLT-only glue.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Flt.Statement` with final theorem statement constants and variant aliases.
  - Documentation that separates final theorem target, bridge policy, and reusable library goal.
- Acceptance criteria:
  - Numeric literals, exponentiation, order, inequality, nonzero hypotheses, and falsehood are explicit certified constants or definitions.
  - The statement module contains no custom bridge axiom.
  - The project contract says that milestones are incomplete if they only add hidden FLT glue without reusable library surface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.Statement`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.Statement`
  - `rg -n "fermat_last_theorem|Flt.BridgeAxiom|library growth" proofs/README.md proofs/Proofs/Ai/NumberTheory`
- Notes:
  - Implemented in `Proofs.Ai.NumberTheory.Flt.Statement`.
  - The current statement freezes the final target as a Prop-valued constant
    parameterized by explicit `add`, `pow`, and `lt` operations because the
    reusable Nat arithmetic/order APIs are scheduled for NT-01 through NT-03.
  - It has no `Flt.BridgeAxiom.*` dependency and records the library growth
    rule in `proofs/README.md`.

### FLT-01 Bridge Policy And Release-Gate Skeleton

- Status: Pending
- Depends on: FLT-00
- Inputs:
  - `proofs/fermats-last-theorem-proof-phases.md`
  - `crates/npa-package`
  - `crates/npa-cli`
  - `crates/npa-checker-ref`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Flt/Bridge/`
  - `proofs/npa-package.toml`
  - package verifier tests or release-audit fixture tests
- Tasks:
  - Add development-only bridge axiom names under `Flt.BridgeAxiom.*`.
  - Add a release policy fixture that rejects bridge axioms in final theorem imports.
  - Add negative tests proving that bridge-backed final release fails.
- Deliverables:
  - Bridge interface module for early smoke tests.
  - Release-gate skeleton that can be reused by later FLT final audits.
- Acceptance criteria:
  - Bridge axioms are visible in the axiom report when used.
  - Final release policy rejects any transitive `Flt.BridgeAxiom.*`.
  - Bridge policy tests do not modify kernel trusted base.
  - As a governance milestone, it documents and enforces the library growth
    rule rather than exporting a domain theorem package.
- Verification:
  - `cargo test -p npa-package`
  - `cargo test -p npa-checker-ref high_trust`
  - `rg -n "Flt.BridgeAxiom|bridge" proofs crates develop`
- Notes:
  - This milestone supports engineering smoke certificates only; it does not prove mathematical content.

### NT-01 Natural, Integer, Rational, And Positivity Foundations

- Status: Pending
- Depends on: FLT-01
- Inputs:
  - `Proofs.Ai.NumberTheory.Flt.Statement`
  - existing `Std.Logic.Eq` and `Std.Nat.Basic`
  - existing algebra and field foundation modules
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Elementary/`
  - `proofs/Proofs/Ai/NumberTheory/IntRat/`
  - `proofs/README.md`
- Tasks:
  - Add reusable `Nat`, `Int`, `Rat`, positivity, and nonzero predicate APIs needed by FLT statement variants.
  - Add translation lemmas between positive naturals, nonzero naturals, integers, and rationals.
  - Export arithmetic facts useful outside FLT, such as positivity preservation under multiplication and powers.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Elementary` and, if needed, `Proofs.Ai.NumberTheory.IntRat`.
  - Reusable positivity and coercion theorem surfaces.
- Acceptance criteria:
  - The statement variants from FLT-00 can import these modules without bridge axioms.
  - The module names and theorem names are not FLT-specific.
  - `Rat` facts depend on explicit field or fraction evidence, not a new kernel primitive.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Elementary`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Elementary`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Positive|Nonzero|Int|Rat|coerce" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Notes:
  - This is the first milestone where the theorem-library goal should be visible to non-FLT arithmetic proofs.

### NT-02 Divisibility, GCD, Coprimality, And Prime API

- Status: Pending
- Depends on: NT-01
- Inputs:
  - `Proofs.Ai.NumberTheory.Elementary`
  - `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`
  - `Proofs.Ai.Algebra.AbstractField`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Divisibility/`
  - `proofs/Proofs/Ai/NumberTheory/Prime/`
  - `proofs/README.md`
- Tasks:
  - Add reusable `Divides`, `Gcd`, `Coprime`, `PrimeNat`, and prime-factor extraction APIs.
  - Bridge number-theoretic primality to existing abstract UFD prime-factor packages where useful.
  - Add standard lemmas for divisibility transitivity, gcd symmetry, coprime multiplication, and prime divisibility.
- Deliverables:
  - Certificate-backed divisibility and prime modules.
  - README table describing which theorem names support FLT reduction and which are general library facts.
- Acceptance criteria:
  - Prime and gcd facts can be consumed without importing elliptic-curve or modularity modules.
  - FLT reduction can express primitive counterexamples using only this API plus NT-03.
  - No theorem-shaped axiom states the prime-exponent reduction directly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Prime`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Prime`
  - `rg -n "Divides|Gcd|Coprime|PrimeNat|prime" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Notes:
  - Keep local `Nonzero` definitions aligned with `AbstractField` and UFD terminology to avoid later name collisions.

### NT-03 Exponentiation And Power Arithmetic Library

- Status: Pending
- Depends on: NT-02
- Inputs:
  - `Proofs.Ai.NumberTheory.Elementary`
  - `Proofs.Ai.NumberTheory.Prime`
  - existing ring and field APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Powers/`
  - `proofs/README.md`
- Tasks:
  - Add natural and integer exponentiation APIs with explicit base and exponent evidence.
  - Add reusable power laws for multiplication, addition of exponents, nonzero powers, parity, and prime exponents.
  - Add theorem aliases specialized to FLT statement shapes only after general power laws exist.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Powers` with general theorem names.
  - FLT statement compatibility lemmas importing the general module.
- Acceptance criteria:
  - `a ^ n + b ^ n = c ^ n` can be elaborated without hidden notation or source-only sugar.
  - Power theorems are reusable for later Frey curve and conductor computations.
  - No bridge axiom is introduced.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Powers`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Powers`
  - `rg -n "pow|power|exponent|PrimeNat" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Notes:
  - Do not rely on overloaded numerals or implicit coercions as certificate evidence.

### NT-04 Infinite Descent And Exponent-Four Route

- Status: Pending
- Depends on: NT-03
- Inputs:
  - `Proofs.Ai.NumberTheory.Powers`
  - `Proofs.Ai.NumberTheory.Divisibility`
  - `Proofs.Ai.NumberTheory.Prime`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Descent/`
  - `proofs/Proofs/Ai/NumberTheory/Flt/ExponentFour/`
  - `proofs/README.md`
- Tasks:
  - Add a reusable finite-descent or minimal-counterexample theorem pattern.
  - Formalize the exponent-four FLT route as a checked theorem.
  - Export descent lemmas useful for Diophantine equations beyond FLT.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Descent`.
  - `Proofs.Ai.NumberTheory.Flt.ExponentFour`.
- Acceptance criteria:
  - The exponent-four theorem is bridge-free and imports only arithmetic foundations.
  - Descent theorem names are general enough for future number-theory routes.
  - The module records the exact well-founded or minimality evidence used.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.ExponentFour`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.ExponentFour`
  - `rg -n "Descent|exponent_four|minimal" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Notes:
  - If well-founded recursion support is insufficient, keep construction evidence explicit rather than adding a kernel primitive.

### NT-05 Primitive Prime-Exponent Reduction

- Status: Pending
- Depends on: NT-04
- Inputs:
  - `Proofs.Ai.NumberTheory.Flt.Statement`
  - `Proofs.Ai.NumberTheory.Flt.ExponentFour`
  - `Proofs.Ai.NumberTheory.Prime`
  - `Proofs.Ai.NumberTheory.Powers`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/PrimeExponent/`
  - `proofs/Proofs/Ai/NumberTheory/Flt/Reduction/`
  - `proofs/README.md`
- Tasks:
  - Define primitive counterexamples and primitive prime-exponent counterexamples.
  - Prove the standard reduction from an arbitrary FLT counterexample to exponent four or an odd prime exponent.
  - Export general primitive/coprime normalization lemmas for future Diophantine proofs.
- Deliverables:
  - `Proofs.Ai.NumberTheory.PrimeExponent`.
  - `Proofs.Ai.NumberTheory.Flt.Reduction`.
- Acceptance criteria:
  - The reduction is checked and bridge-free.
  - The prime-exponent theorem is independent of elliptic curves, modular forms, Ribet, and Wiles.
  - The primitive normalization API is reusable outside FLT.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.Reduction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.Reduction`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "primitive_prime_counterexample|coprime|PrimeExponent" proofs/Proofs/Ai/NumberTheory`
- Notes:
  - This milestone is the arithmetic boundary before elliptic-curve construction begins.

### ALG-01 Field, Fraction, And Rational Function Alignment

- Status: Pending
- Depends on: FLT-01
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `develop/proof-corpus-field-theory-roadmap-todo.md`
  - `Proofs.Ai.Algebra.AbstractRing`
- Code or documentation areas:
  - `proofs/Proofs/Ai/Algebra/AbstractField/`
  - `proofs/Proofs/Ai/Algebra/AbstractFractionField/`
  - `proofs/README.md`
- Tasks:
  - Audit existing `AbstractField` against FLT needs for `Q`, fraction fields, rational functions, and nonzero denominators.
  - Add reusable fraction-field theorem targets if the existing field module is too small.
  - Provide theorem names for cancellation, field hom compatibility, and rational-function evaluation.
- Deliverables:
  - Either updated `AbstractField` theorem surface or a split `AbstractFractionField` module.
  - Documentation explaining how FLT modules should import field facts.
- Acceptance criteria:
  - Elliptic-curve equations over `Q` can be stated without ad hoc field assumptions.
  - Fraction-field facts are not tied to Frey curves by name.
  - The module remains source-free verifiable and bridge-free.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField`
  - `rg -n "AbstractField|FractionField|FieldLawArgs|rational" proofs develop`
- Notes:
  - If a new split module is created, run the build and source-free checks for that split module as well.

### ALG-02 Ideals, Quotients, Modules, And Subobjects Library

- Status: Pending
- Depends on: ALG-01
- Inputs:
  - existing group quotient modules
  - `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`
  - `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`
  - `Proofs.Ai.Algebra.AbstractKrullTheorem`
- Code or documentation areas:
  - `proofs/Proofs/Ai/Algebra/AbstractIdeal/`
  - `proofs/Proofs/Ai/Algebra/AbstractModule/`
  - `proofs/Proofs/Ai/Algebra/AbstractRingQuotient/`
  - `proofs/README.md`
- Tasks:
  - Extract reusable ideal, submodule, quotient ring, quotient module, and finite-dimensional module APIs.
  - Add standard inclusion, kernel, image, exactness, and quotient universal-property theorem targets.
  - Keep quotient dependencies visible through package metadata and core-feature reports.
- Deliverables:
  - Reusable algebraic subobject modules needed by Hecke algebras, deformation rings, and Galois representations.
  - README dependency map from existing algebra corpus to new subobject APIs.
- Acceptance criteria:
  - Later modularity modules can state `R = T` style theorems without inventing local quotient APIs.
  - No quotient-related core feature is hidden from generated reports.
  - The APIs are domain-oriented and not named after FLT.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractRingQuotient`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractRingQuotient`
  - `rg -n "Ideal|Submodule|Quotient|Exact|kernel|image" proofs/Proofs/Ai/Algebra proofs/README.md`
- Notes:
  - This milestone may need to be split if quotient support requires checker or package-tooling work.

### ALG-03 Polynomial, Power Series, And Local Ring Library

- Status: Pending
- Depends on: ALG-02
- Inputs:
  - `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`
  - `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`
  - `Proofs.Ai.Algebra.AbstractKrullTheorem`
- Code or documentation areas:
  - `proofs/Proofs/Ai/Algebra/AbstractPolynomial/`
  - `proofs/Proofs/Ai/Algebra/AbstractPowerSeries/`
  - `proofs/Proofs/Ai/Algebra/AbstractLocalRing/`
  - `proofs/README.md`
- Tasks:
  - Add polynomial-ring operations, evaluation, degree, roots, and finite support APIs.
  - Add local ring, maximal ideal, completion, and formal power series theorem surfaces needed by deformation theory.
  - Export standard algebraic-geometry and commutative-algebra lemmas useful beyond FLT.
- Deliverables:
  - Reusable polynomial and local-ring modules.
  - Bridge documentation to existing Hilbert basis, Nullstellensatz, and Krull modules.
- Acceptance criteria:
  - Elliptic-curve and deformation modules can import polynomial/local-ring facts directly.
  - Complete local ring assumptions stay explicit as construction or law-package evidence.
  - No set-theoretic existence theorem is hidden in automation.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractPolynomial`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractPolynomial`
  - `rg -n "Polynomial|PowerSeries|LocalRing|Completion|MaximalIdeal" proofs/Proofs/Ai/Algebra proofs/README.md`
- Notes:
  - Power series and completion may become separate milestones if their certificate size grows quickly.

### ALG-04 Finite Sets, Finite Sums, Matrices, And Linear Algebra Interfaces

- Status: Pending
- Depends on: ALG-03
- Inputs:
  - `Proofs.Ai.Vector.AbstractSpace`
  - `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem`
  - standard-library finite data structures if available
- Code or documentation areas:
  - `proofs/Proofs/Ai/Combinatorics/Finite/`
  - `proofs/Proofs/Ai/LinearAlgebra/Matrix/`
  - `proofs/README.md`
- Tasks:
  - Add finite-indexed sums, products, cardinality, finite support, and matrix APIs.
  - Add determinant, trace, characteristic polynomial, eigenvalue, and matrix representation theorem targets.
  - Export reusable finite-dimensional linear algebra lemmas for modular forms and Galois representations.
- Deliverables:
  - `Proofs.Ai.Combinatorics.Finite`.
  - `Proofs.Ai.LinearAlgebra.Matrix`.
- Acceptance criteria:
  - Hecke operators and representation matrices can be stated over these APIs.
  - Finite collection facts are general and not modular-form-specific.
  - Certificate artifacts are deterministic and source-free verifiable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LinearAlgebra.Matrix`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LinearAlgebra.Matrix`
  - `rg -n "Finite|sum|product|Matrix|determinant|trace" proofs/Proofs/Ai proofs/README.md`
- Notes:
  - This is the common algebraic dependency for elliptic curves, modular forms, and Galois representations.

### EC-01 Weierstrass Models And Elliptic-Curve Basic API

- Status: Pending
- Depends on: NT-05, ALG-04
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - polynomial and rational-function APIs
  - finite/matrix APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/EllipticCurve/Basic/`
  - `proofs/README.md`
- Tasks:
  - Add Weierstrass equations, curve points, point-at-infinity, nonsingularity, and coordinate-change APIs.
  - Add reusable theorems for curve isomorphism, discriminant nonzero, and point membership transport.
  - Keep definitions over explicit field law packages.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Basic`.
  - General elliptic-curve theorem surface not specialized to Frey curves.
- Acceptance criteria:
  - Frey curve construction can import a general elliptic-curve API.
  - The module is useful for other elliptic-curve proofs over fields.
  - No modularity or Ribet bridge axiom is imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic`
  - `rg -n "Weierstrass|EllipticCurve|discriminant|nonsingular" proofs/Proofs/Ai/EllipticCurve proofs/README.md`
- Notes:
  - The group law on elliptic-curve points can be a follow-up submodule if too large for this milestone.

### EC-02 Discriminant, Conductor, Reduction Type, And Semistability

- Status: Pending
- Depends on: EC-01
- Inputs:
  - `Proofs.Ai.EllipticCurve.Basic`
  - number-theory divisibility and valuation APIs
  - polynomial/local-ring APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/EllipticCurve/Reduction/`
  - `proofs/Proofs/Ai/EllipticCurve/Semistable/`
  - `proofs/README.md`
- Tasks:
  - Add discriminant, minimal model, valuation, conductor, reduction type, good/multiplicative/additive reduction, and semistability predicates.
  - Export reusable conductor and reduction lemmas for elliptic curves over `Q`.
  - Document which arithmetic facts are still construction evidence rather than proved reductions.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Reduction`.
  - `Proofs.Ai.EllipticCurve.Semistable`.
- Acceptance criteria:
  - Semistability is a general elliptic-curve predicate, not a Frey-only theorem name.
  - The conductor API is sufficient for Ribet level-lowering statements.
  - Any temporary arithmetic gap is visible as a local bridge assumption, not an unmarked theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Semistable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Semistable`
  - `rg -n "Conductor|Reduction|Semistable|valuation" proofs/Proofs/Ai/EllipticCurve proofs/README.md`
- Notes:
  - This milestone builds library value for arithmetic geometry independent of FLT.

### EC-03 Frey Curve Construction

- Status: Pending
- Depends on: EC-02
- Inputs:
  - `Proofs.Ai.NumberTheory.Flt.Reduction`
  - `Proofs.Ai.EllipticCurve.Basic`
  - `Proofs.Ai.EllipticCurve.Semistable`
- Code or documentation areas:
  - `proofs/Proofs/Ai/EllipticCurve/Frey/`
  - `proofs/README.md`
- Tasks:
  - Define the Frey curve attached to a primitive prime-exponent counterexample.
  - Prove curve membership, nonsingularity, discriminant, conductor-shape, and semistability facts as far as the current arithmetic library permits.
  - Export reusable specialization lemmas for parametric Weierstrass models.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Frey`.
  - Checked Frey construction facts with explicit dependency metadata.
- Acceptance criteria:
  - Frey facts are proved from number theory and elliptic-curve definitions, not final bridge axioms.
  - Any remaining local construction assumption is named and tracked in the axiom report.
  - The module contributes reusable parameterized curve lemmas where possible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Frey`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Frey`
  - `rg -n "Frey|frey_curve|primitive_prime_counterexample|semistable" proofs/Proofs/Ai`
- Notes:
  - Frey-specific names are acceptable here, but supporting Weierstrass lemmas should stay general.

### EC-04 Mod-p Representation Interface For Elliptic Curves

- Status: Pending
- Depends on: EC-03, GAL-01
- Inputs:
  - `Proofs.Ai.EllipticCurve.Frey`
  - `Proofs.Ai.GaloisRepresentation.Basic`
  - finite group and matrix APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/EllipticCurve/GaloisRepresentation/`
  - `proofs/README.md`
- Tasks:
  - Define the mod-prime representation attached to elliptic-curve torsion data at interface level.
  - Add reusable facts relating elliptic-curve isomorphism, torsion, and representation equivalence.
  - State Frey representation properties needed by Ribet with explicit evidence parameters.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.GaloisRepresentation`.
  - Frey representation theorem targets used by level lowering.
- Acceptance criteria:
  - The representation interface is usable for elliptic curves other than Frey curves.
  - Ramification and irreducibility assumptions are explicit.
  - Bridge assumptions, if any, are named outside reusable library theorem names.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `rg -n "rho|GaloisRepresentation|torsion|irreducible|ramification" proofs/Proofs/Ai`
- Notes:
  - This milestone bridges elliptic curves to the general Galois representation library.

### MF-01 Modular Forms And Newforms Basic Library

- Status: Pending
- Depends on: ALG-04
- Inputs:
  - finite sums and matrices
  - polynomial, field, and vector-space APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/ModularForms/Basic/`
  - `proofs/README.md`
- Tasks:
  - Add modular form, cusp form, newform, level, weight, q-expansion, and coefficient-field interfaces.
  - Add reusable theorem targets for equality by q-expansion, level/weight transport, and newform membership.
  - Keep analytic or geometric construction evidence explicit.
- Deliverables:
  - `Proofs.Ai.ModularForms.Basic`.
  - General modular-form API for later modularity and Ribet statements.
- Acceptance criteria:
  - The module does not mention FLT or Frey curves.
  - Newform and q-expansion theorem names are reusable for other modular-form proofs.
  - Any construction not yet formalized is marked as construction evidence or bridge interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ModularForms.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Basic`
  - `rg -n "ModularForm|Newform|q_expansion|level|weight" proofs/Proofs/Ai/ModularForms proofs/README.md`
- Notes:
  - This is a vocabulary milestone, not Ribet or Wiles.

### MF-02 Modular Curves, Hecke Operators, And Hecke Algebras

- Status: Pending
- Depends on: MF-01, ALG-04
- Inputs:
  - `Proofs.Ai.ModularForms.Basic`
  - finite-dimensional matrix and algebra APIs
  - quotient and polynomial APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/ModularForms/Hecke/`
  - `proofs/Proofs/Ai/ModularForms/ModularCurve/`
  - `proofs/README.md`
- Tasks:
  - Add modular curve, Jacobian interface, Hecke operator, Hecke algebra, eigenform, and eigensystem APIs.
  - Add reusable operator algebra lemmas for commuting Hecke operators and eigenvalue transport.
  - Provide theorem targets needed by both Ribet and modularity lifting.
- Deliverables:
  - `Proofs.Ai.ModularForms.Hecke`.
  - `Proofs.Ai.ModularForms.ModularCurve`.
- Acceptance criteria:
  - Hecke facts are stated as general algebraic APIs, not as hidden Wiles assumptions.
  - Later `R = T` modules can import the Hecke algebra interface directly.
  - No source-level notation is needed to identify operator composition or eigensystems.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ModularForms.Hecke`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Hecke`
  - `rg -n "Hecke|ModularCurve|eigen|Jacobian|algebra" proofs/Proofs/Ai/ModularForms proofs/README.md`
- Notes:
  - Modular curves may need bridge construction evidence until the geometry library is strong enough.

### GAL-01 Galois Groups And Representation Basics

- Status: Pending
- Depends on: ALG-04
- Inputs:
  - finite group, field, matrix, and polynomial APIs
  - quotient and subobject APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/GaloisRepresentation/Basic/`
  - `proofs/README.md`
- Tasks:
  - Add Galois group, field extension, representation, representation equivalence, irreducibility, determinant, trace, and residual representation APIs.
  - Export reusable representation lemmas over finite-dimensional vector spaces.
  - Keep absolute Galois group construction evidence explicit.
- Deliverables:
  - `Proofs.Ai.GaloisRepresentation.Basic`.
  - General theorem surface for representations independent of elliptic curves and modular forms.
- Acceptance criteria:
  - Elliptic-curve and modular-form representation interfaces can share this API.
  - Irreducibility and determinant facts are reusable.
  - No FLT bridge axiom is imported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.GaloisRepresentation.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisRepresentation.Basic`
  - `rg -n "Galois|Representation|irreducible|determinant|trace" proofs/Proofs/Ai/GaloisRepresentation proofs/README.md`
- Notes:
  - This milestone should not assume the representation comes from an elliptic curve.

### GAL-02 Ramification, Local Conditions, And Compatible Systems

- Status: Pending
- Depends on: GAL-01, ALG-03
- Inputs:
  - local ring and completion APIs
  - `Proofs.Ai.GaloisRepresentation.Basic`
  - number-theory prime and valuation APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/GaloisRepresentation/Ramification/`
  - `proofs/Proofs/Ai/GaloisRepresentation/LocalCondition/`
  - `proofs/README.md`
- Tasks:
  - Add ramification, unramified, finite-flat, semistable local condition, conductor, and compatible-system theorem surfaces.
  - Add reusable transport lemmas for representation equivalence and local conditions.
  - Provide explicit evidence hooks for Ribet and modularity-lifting assumptions.
- Deliverables:
  - `Proofs.Ai.GaloisRepresentation.Ramification`.
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`.
- Acceptance criteria:
  - Ribet and modularity modules do not define local conditions privately.
  - Local assumptions are explicit in theorem statements and axiom reports.
  - The library facts can support future arithmetic geometry proofs beyond FLT.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `rg -n "Ramification|LocalCondition|unramified|finite_flat|conductor" proofs/Proofs/Ai/GaloisRepresentation proofs/README.md`
- Notes:
  - This is the main dependency for both Ribet and Taylor-Wiles routes.

### RIB-01 Ribet Level-Lowering Statement And Dependency Map

- Status: Pending
- Depends on: MF-02, GAL-02, EC-04
- Inputs:
  - `Proofs.Ai.ModularForms.Hecke`
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `Proofs.Ai.EllipticCurve.GaloisRepresentation`
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/Ribet/`
  - `proofs/README.md`
- Tasks:
  - State Ribet level lowering in the exact interface needed by the Frey curve.
  - Add a dependency map for conductor, irreducibility, ramification, newform, and excluded low-level cases.
  - Add reusable level-lowering interface lemmas that are not Frey-specific.
- Deliverables:
  - `Proofs.Ai.Modularity.Ribet` interface module.
  - README dependency map from Ribet statement to reusable library modules.
- Acceptance criteria:
  - Any early bridge axiom is clearly named `Flt.BridgeAxiom.ribet_level_lowering` or an equivalent bridge namespace.
  - General level-lowering terminology is reusable for non-Frey representations.
  - The module cannot be confused with a completed Ribet proof while bridge-backed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Ribet`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Ribet`
  - `rg -n "Ribet|level_lowering|BridgeAxiom|conductor|newform" proofs/Proofs/Ai proofs/README.md`
- Notes:
  - This milestone is allowed to be conditional, but only if the conditionality is machine-visible.

### RIB-02 Level-Lowering Supporting Library

- Status: Pending
- Depends on: RIB-01
- Inputs:
  - `Proofs.Ai.Modularity.Ribet`
  - modular forms, Hecke, and Galois representation modules
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/LevelLowering/`
  - `proofs/README.md`
- Tasks:
  - Replace pieces of the Ribet bridge with certificate-backed lemmas for congruence, conductor lowering, and newform transport.
  - Export reusable theorem surfaces for residual representations and level lowering.
  - Keep remaining unproved deep components as named construction evidence.
- Deliverables:
  - `Proofs.Ai.Modularity.LevelLowering`.
  - A bridge-elimination checklist for Ribet dependencies.
- Acceptance criteria:
  - At least one bridge dependency from RIB-01 is removed or narrowed.
  - General level-lowering lemmas are discoverable in theorem index.
  - Remaining bridge assumptions are still rejected by final release policy.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.LevelLowering`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.LevelLowering`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "level_lowering|bridge|residual|congruence" proofs/Proofs/Ai/Modularity proofs/README.md`
- Notes:
  - This milestone may repeat as several implementation batches, but each batch must remove or localize a named bridge.

### RIB-03 Frey Nonmodularity Glue

- Status: Pending
- Depends on: RIB-02, EC-04, NT-05
- Inputs:
  - `Proofs.Ai.EllipticCurve.Frey`
  - `Proofs.Ai.Modularity.LevelLowering`
  - `Proofs.Ai.NumberTheory.Flt.Reduction`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Flt/RibetContradiction/`
  - `proofs/README.md`
- Tasks:
  - Prove the glue theorem from a primitive prime-exponent counterexample and Frey modularity to contradiction, conditional on the current Ribet interface.
  - Export supporting lemmas linking Frey conductor and representation properties to general level-lowering hypotheses.
  - Label the theorem as conditional if any Ribet bridge remains.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Flt.RibetContradiction`.
  - Reusable Frey-to-level-lowering transport lemmas.
- Acceptance criteria:
  - The theorem's bridge status is visible in axiom reports.
  - General transport lemmas do not depend on final FLT theorem names.
  - Final FLT cannot import this module in high-trust mode until bridge dependencies are gone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.RibetContradiction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.RibetContradiction`
  - `rg -n "RibetContradiction|modular_elliptic_curve|Frey|BridgeAxiom" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Notes:
  - This is FLT glue, but it should still contribute Frey representation transport theorems.

### MOD-01 Deformation Functors And Local Deformation Conditions

- Status: Pending
- Depends on: GAL-02, ALG-03
- Inputs:
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`
  - local ring and completion APIs
  - quotient and module APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/Deformation/`
  - `proofs/README.md`
- Tasks:
  - Add deformation functor, lift, local deformation condition, tangent space, and obstruction interfaces.
  - Export reusable theorem targets for functoriality, local-global compatibility, and deformation equivalence.
  - Keep deformation-ring existence as explicit construction evidence until proved.
- Deliverables:
  - `Proofs.Ai.Modularity.Deformation`.
  - General deformation-theory API.
- Acceptance criteria:
  - The API is usable for representations beyond elliptic curves.
  - Universal deformation ring assumptions are explicit.
  - No Wiles theorem is introduced as a primitive or hidden bridge.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Deformation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Deformation`
  - `rg -n "Deformation|Functor|Lift|tangent|obstruction" proofs/Proofs/Ai/Modularity proofs/README.md`
- Notes:
  - This milestone builds a reusable representation-deformation library before semistable modularity.

### MOD-02 Universal Deformation Rings And Hecke Algebra Interface

- Status: Pending
- Depends on: MOD-01, MF-02
- Inputs:
  - `Proofs.Ai.Modularity.Deformation`
  - `Proofs.Ai.ModularForms.Hecke`
  - local ring and complete-intersection APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/HeckeDeformation/`
  - `proofs/README.md`
- Tasks:
  - Add universal deformation ring, Hecke algebra, Gorenstein, complete-intersection, and comparison-map theorem surfaces.
  - Add reusable commutative-algebra lemmas needed for `R = T`.
  - Document all complete-intersection and Gorenstein evidence explicitly.
- Deliverables:
  - `Proofs.Ai.Modularity.HeckeDeformation`.
  - General Hecke/deformation comparison API.
- Acceptance criteria:
  - Ring-theoretic assumptions are not hidden in automation.
  - `R = T` can be stated using reusable ring and Hecke APIs.
  - The module can be source-free verified under a policy that rejects FLT bridge axioms.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.HeckeDeformation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.HeckeDeformation`
  - `rg -n "universal_deformation|Hecke|Gorenstein|complete_intersection|R_eq_T" proofs/Proofs/Ai/Modularity proofs/README.md`
- Notes:
  - This milestone should benefit future modularity-lifting projects beyond FLT.

### MOD-03 Minimal And Non-Minimal Modularity Lifting

- Status: Pending
- Depends on: MOD-02
- Inputs:
  - `Proofs.Ai.Modularity.HeckeDeformation`
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`
  - modular forms and Hecke APIs
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/Lifting/`
  - `proofs/README.md`
- Tasks:
  - Add minimal modularity lifting theorem surface.
  - Add non-minimal lifting steps needed for semistable curves.
  - Export reusable lifting theorem wrappers parameterized by local conditions and residual modularity evidence.
- Deliverables:
  - `Proofs.Ai.Modularity.Lifting`.
  - Bridge-elimination ledger for Wiles/Taylor-Wiles dependencies.
- Acceptance criteria:
  - The theorem is decomposed into independently checkable certificates.
  - Remaining deep assumptions are named and machine-visible.
  - Lifting theorem wrappers are useful outside the Frey curve case.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Lifting`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Lifting`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "modularity_lifting|minimal|non_minimal|BridgeAxiom|R_eq_T" proofs/Proofs/Ai/Modularity proofs/README.md`
- Notes:
  - This is expected to be a sequence of sub-batches; each batch must be independently source-free verifiable.

### MOD-04 Semistable Modularity Theorem

- Status: Pending
- Depends on: MOD-03, EC-02
- Inputs:
  - `Proofs.Ai.Modularity.Lifting`
  - `Proofs.Ai.EllipticCurve.Semistable`
  - `Proofs.Ai.EllipticCurve.GaloisRepresentation`
- Code or documentation areas:
  - `proofs/Proofs/Ai/Modularity/Semistable/`
  - `proofs/README.md`
- Tasks:
  - Prove or bridge-stage the theorem that semistable elliptic curves over `Q` are modular.
  - Add reusable transport lemmas from semistable elliptic-curve evidence to modularity-lifting hypotheses.
  - Remove or narrow `Flt.BridgeAxiom.semistable_modularity` as certificates replace interfaces.
- Deliverables:
  - `Proofs.Ai.Modularity.Semistable`.
  - Final or conditional semistable modularity certificate with explicit axiom report.
- Acceptance criteria:
  - The final version has no `Flt.BridgeAxiom.*` dependency.
  - The theorem is decomposed over explicit import hashes.
  - The modularity theorem can be used by non-FLT semistable elliptic-curve modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Semistable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Semistable`
  - `rg -n "semistable_modularity|modular_elliptic_curve|BridgeAxiom" proofs/Proofs/Ai/Modularity proofs/README.md`
- Notes:
  - Early versions may be bridge-backed only for final-glue validation; they are not final proof evidence.

### FINAL-01 Conditional Final-Theorem Smoke

- Status: Pending
- Depends on: RIB-03, MOD-04, NT-05, EC-04
- Inputs:
  - `Proofs.Ai.NumberTheory.Flt.Reduction`
  - `Proofs.Ai.NumberTheory.Flt.RibetContradiction`
  - `Proofs.Ai.Modularity.Semistable`
  - `Proofs.Ai.NumberTheory.Flt.Bridge`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Flt/ConditionalFinal/`
  - `proofs/README.md`
  - release-gate negative fixtures
- Tasks:
  - Prove `fermat_last_theorem_conditional` over explicit Ribet and semistable modularity interfaces.
  - Build the early smoke certificate that may still show bridge axioms.
  - Add documentation that this is not the completed FLT proof.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Flt.ConditionalFinal`.
  - A source-free smoke certificate and visible bridge axiom report.
- Acceptance criteria:
  - The final theorem shape, import graph, and package metadata are frozen.
  - Bridge-backed smoke output is clearly labeled as conditional or development-only.
  - The negative release gate rejects this module as final if bridge axioms remain.
  - Smoke glue does not count as reusable library growth by itself; every
    mathematical dependency points to a reusable domain module.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.ConditionalFinal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.ConditionalFinal`
  - `rg -n "ConditionalFinal|fermat_last_theorem_conditional|Flt.BridgeAxiom" proofs/Proofs/Ai/NumberTheory/Flt proofs/README.md`
- Notes:
  - This milestone is valuable for engineering integration even before deep theorems are fully certificate-backed.

### FINAL-02 Bridge Elimination And Library Closure Audit

- Status: Pending
- Depends on: FINAL-01
- Inputs:
  - all `Proofs.Ai.NumberTheory.*`, `EllipticCurve.*`, `ModularForms.*`, `GaloisRepresentation.*`, and `Modularity.*` modules
  - generated axiom reports and theorem index
- Code or documentation areas:
  - `proofs/README.md`
  - `develop/` audit documents if needed
  - package verification fixtures
- Tasks:
  - Audit every remaining bridge axiom and assign it to a concrete replacement module.
  - Confirm reusable theorem-library surfaces exist for every major field used on the FLT route.
  - Add missing nearby lemmas where a module only exposes a narrow FLT theorem.
- Deliverables:
  - Bridge-elimination report.
  - Library-coverage report for arithmetic, algebra, elliptic curves, modular forms, Galois representations, and modularity.
- Acceptance criteria:
  - No bridge axiom remains undocumented.
  - Every FLT-only dependency either has a reusable backing library theorem or a documented reason why it cannot be generalized.
  - The next final theorem milestone has a zero-bridge dependency plan.
- Verification:
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `cargo run -p npa-proof-corpus -- --write-ai-index`
  - `rg -n "Flt.BridgeAxiom|Bridge|conditional|Final" proofs/Proofs/Ai proofs/generated proofs/README.md`
- Notes:
  - This milestone enforces the user's library-building objective before final proof assembly.

### FINAL-03 Bridge-Free Final Contradiction And Statement Aliases

- Status: Pending
- Depends on: FINAL-02
- Inputs:
  - bridge-free Ribet route
  - bridge-free semistable modularity route
  - `Proofs.Ai.NumberTheory.Flt.Reduction`
  - `Proofs.Ai.EllipticCurve.Frey`
- Code or documentation areas:
  - `proofs/Proofs/Ai/NumberTheory/Flt/Final/`
  - `proofs/README.md`
  - `proofs/npa-package.toml`
  - generated package artifacts
- Tasks:
  - Prove the primitive prime-exponent contradiction without bridge axioms.
  - Prove `fermat_last_theorem`.
  - Add integer and positive-natural compatibility aliases.
  - Pin source, certificate file, export, axiom-report, and certificate hashes.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Flt.Final`.
  - Final theorem certificate and aliases.
- Acceptance criteria:
  - The final axiom report contains no `Flt.BridgeAxiom.*`, no unapproved custom axiom, and no hidden source-side assumption.
  - The theorem imports reusable library modules rather than private copies of arithmetic, elliptic-curve, modular-form, or Galois-representation facts.
  - All certificate and package hashes are deterministic and checked in.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Flt.Final`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Flt.Final`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "fermat_last_theorem|Flt.BridgeAxiom" proofs/Proofs/Ai/NumberTheory/Flt proofs/generated`
- Notes:
  - This milestone is not complete if it merely hides bridge axioms behind renamed imports.

### REL-01 High-Trust Independent Checker Release Audit

- Status: Pending
- Depends on: FINAL-03
- Inputs:
  - `Proofs.Ai.NumberTheory.Flt.Final`
  - package lock
  - axiom report
  - theorem index
  - independent checker profile
- Code or documentation areas:
  - release audit scripts
  - `proofs/generated/`
  - high-trust package fixtures
  - `proofs/README.md`
- Tasks:
  - Run source-free verification over every transitive import in dependency-topological order.
  - Build a release audit bundle containing canonical certificates, checker requests/results, hashes, and policy.
  - Add a negative fixture proving bridge-axiom injection prevents release.
  - Record the theorem-library coverage produced along the route.
- Deliverables:
  - Reproducible high-trust release audit bundle.
  - Negative bridge-injection fixture.
  - Final library coverage summary.
- Acceptance criteria:
  - The final theorem is reproducible without `.npa` source, replay files, theorem search, AI sidecars, or paper summaries.
  - The release gate rejects bridge-backed variants.
  - The project leaves behind a documented reusable library across the fields used by the proof route.
- Verification:
  - `./scripts/check-corpus-full.sh`
  - `./scripts/phase8-release-audit.sh`
  - `cargo test --workspace`
  - `git diff --check`
- Notes:
  - `scripts/phase8-release-audit.sh` currently exists; if it is renamed or
    superseded, update this milestone to the repository's current high-trust
    release gate.

## Review Ledger

Review pass 1 findings:

- Fixed in this document: added an explicit library growth rule so milestones
  cannot collapse into only FLT-specific theorem targets.
- Fixed in this document: separated conditional bridge smoke milestones from
  bridge-free final proof and release audit milestones.
- Fixed in this document: added reusable domain-library deliverables to
  number theory, algebra, elliptic curve, modular forms, Galois representation,
  Ribet, and semistable modularity milestones.
- Fixed in this document: added verification commands that check both local
  module generation and source-free module verification.

Review pass 2 findings:

- None.

Review pass 3 findings:

- Fixed in this document: narrowed the library growth rule from every
  non-final milestone to mathematical domain milestones, and added explicit
  duties for governance, bridge-policy, smoke, and release-audit milestones.
- Fixed in this document: clarified that `Flt.BridgeAxiom.*` is the bridge
  declaration prefix while `Proofs.Ai.NumberTheory.Flt.Bridge` is the intended
  development module namespace.
- Fixed in this document: updated the release-audit note now that
  `scripts/phase8-release-audit.sh` exists in the repository.

Review pass 4 findings:

- None.
