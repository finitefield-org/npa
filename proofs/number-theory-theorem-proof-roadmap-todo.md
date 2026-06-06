# Number Theory Theorem Proof Roadmap Todo

Source: `proofs/number-theory-theorem-proof-roadmap.md`

This task breakdown converts the number theory theorem proof roadmap into
implementation-ready authoring milestones. It is a planning sidecar only: it
does not add trusted proof evidence, axioms, or certificate validity
assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, and AI output
are untrusted.

---

## Scope

This task list covers theorem-card inventory, elementary arithmetic,
divisibility, Euclidean division, gcd/lcm, Bezout, primes, unique
factorization, congruences, residue rings, Chinese remainder theorem, finite
unit groups, Fermat/Euler/Wilson/Carmichael/RSA, primitive roots, characters,
quadratic residues, arithmetic functions, continued fractions, Pell,
Diophantine equations, analytic number theory, sieve and circle-method
interfaces, algebraic number theory, local fields, class field theory,
elliptic curves, modular forms, Langlands interfaces, arithmetic geometry,
Iwasawa theory, Galois representations, computational number theory,
cryptographic correctness, finite-field applications, combinatorial number
theory, and promotion planning.

The list intentionally does not prove the number-theory roadmap in one pass.
Later agents should implement exactly one milestone or a clearly bounded
contiguous batch. When a milestone introduces only a statement interface
because prerequisites are absent, its acceptance criteria must prevent the
interface from smuggling the target theorem as an axiom.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding natural numbers, integers, rationals, residue rings, number fields,
  local fields, elliptic curves, modular forms, Galois representations, or
  cryptographic assumptions as trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, theorem-search runtime,
  or AI calls to trusted code;
- treating source files, replay files, metadata, generated indexes, theorem
  search, roadmap documents, or this task document as proof evidence;
- proving conjectures such as the Riemann hypothesis, generalized Riemann
  hypothesis, Birch and Swinnerton-Dyer, Artin conjecture, Fontaine-Mazur, or
  broad Langlands functoriality;
- promoting bridge-backed number-theory modules into `npa-mathlib` before
  local closure, axiom-report, and package verification checks are clean.

## Authoring Loop

For ordinary number-theory theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --changed-only
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Replace `Proofs.Ai.NumberTheory.X` with the
actual owning namespace for elliptic-curve, modular-form, Galois
representation, algebraic-geometry, or field-theory milestones. Reserve
`check-corpus-package.sh` or `check-corpus-full.sh` for package-wide verifier
behavior, publish-plan or package metadata updates, certificate/checker
compatibility, release work, or promotion into a high-trust closure.

## Current Implementation Facts

- The proof corpus has no general checked `Proofs.Ai.NumberTheory.*`
  foundation for divisibility, gcd, congruences, arithmetic functions, local
  fields, or analytic number theory.
- Existing reusable algebra foundations include `Proofs.Ai.Algebra.AbstractRing`,
  `Proofs.Ai.Algebra.AbstractField`,
  `Proofs.Ai.Algebra.AbstractOrderedField`,
  `Proofs.Ai.Algebra.AbstractRingChineseRemainder`, and
  `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`.
- Existing field-theory planning owns the finite-field core route through
  `develop/proof-corpus-field-theory-roadmap.md`,
  `develop/proof-corpus-field-theory-roadmap-todo.md`, and
  `Proofs.Ai.Algebra.AbstractFiniteField`.
- Existing reusable analysis, measure, topology, and linear-algebra prerequisite
  documents are inputs for analytic number theory, p-adic analysis, geometry
  of numbers, and modular forms.
- If later authors run this task breakdown in a dirty worktree, they must stage
  only the number-theory task's files and leave unrelated changes alone.

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `NT-00` inventory and statement policy | `NT-T00` through `NT-T01` |
| `NT-01` integers, divisibility, and Euclidean division | `NT-T02` through `NT-T03` |
| `NT-02` gcd, lcm, Euclid algorithm, and Bezout | `NT-T04` through `NT-T06` |
| `NT-03` primes and unique factorization | `NT-T07` through `NT-T09` |
| `NT-04` congruences, residue rings, and Chinese remainder | `NT-T10` through `NT-T12` |
| `NT-05` Fermat, Euler, Wilson, Carmichael, and RSA | `NT-T13` through `NT-T15` |
| `NT-06` primitive roots, characters, and Gauss sums | `NT-T16` through `NT-T18` |
| `NT-07` quadratic residues and reciprocity | `NT-T19` through `NT-T21` |
| `NT-08` arithmetic functions and convolution | `NT-T22` through `NT-T24` |
| `NT-09` continued fractions, Pell, and Diophantine approximation | `NT-T25` through `NT-T27` |
| `NT-10` Diophantine equations and additive number theory | `NT-T28` through `NT-T30` |
| `NT-11` analytic number theory foundations | `NT-T31` through `NT-T34` |
| `NT-12` sieve methods and circle method | `NT-T35` through `NT-T36` |
| `NT-13` algebraic number theory | `NT-T37` through `NT-T39` |
| `NT-14` local fields and p-adic analysis | `NT-T40` through `NT-T42` |
| `NT-15` class field theory | `NT-T43` through `NT-T44` |
| `NT-16` elliptic curves | `NT-T45` through `NT-T48` |
| `NT-17` modular forms and modularity | `NT-T49` through `NT-T52` |
| `NT-18` L-functions and Langlands interfaces | `NT-T53` through `NT-T55` |
| `NT-19` arithmetic geometry | `NT-T56` through `NT-T58` |
| `NT-20` Iwasawa theory | `NT-T59` through `NT-T60` |
| `NT-21` Galois representations and density | `NT-T61` through `NT-T63` |
| `NT-22` computational number theory and cryptography | `NT-T64` through `NT-T66` |
| `NT-23` finite fields and combinatorial number theory | `NT-T67` through `NT-T69` |
| `NT-24` packaging and promotion | `NT-T70` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `NT-T00` | `L0` theorem-card inventory, duplicate-home map, and conjecture labels |
| `NT-T01` through `NT-T12` | `L2` derived certificates where `Nat`/`Int` and quotient prerequisites exist; `L1` statement packages only for missing integer APIs |
| `NT-T13` through `NT-T24` | `L2` derived certificates after finite group, residue ring, and divisor-sum APIs are stable; algorithmic tests may start as `L1` |
| `NT-T25` through `NT-T30` | `L2` for rational continued fractions and small Diophantine classifications; `L1` for advanced approximation and additive-combinatorics theorems |
| `NT-T31` through `NT-T36` | `L1` analytic interfaces first, promoting individual algebraic Euler-product identities to `L2` as prerequisites land |
| `NT-T37` through `NT-T44` | `L1` construction and reciprocity interfaces first, with derived algebraic sublemmas at `L2` when explicit law packages exist |
| `NT-T45` through `NT-T63` | `L1` interfaces first for elliptic curves, modularity, Langlands, arithmetic geometry, Iwasawa, and density; `L2` only for bounded reusable lemmas |
| `NT-T64` through `NT-T69` | `L2` for algebraic correctness lemmas where functions exist; security and complexity assumptions remain `L0` or `L1` |
| `NT-T70` | `L3` public closure planning and package verification |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the execution dependency order in the
`Recommended Queue Coverage` table below. The detailed milestone section keeps
roadmap/topic order, so a task card can name dependencies whose details appear
later in the file.

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `NTQ-001` | `NT-T00`, `NT-T01` |
| `NTQ-002` | `NT-T02`, `NT-T03` |
| `NTQ-003` | `NT-T04`, `NT-T05`, `NT-T06` |
| `NTQ-004` | `NT-T07`, `NT-T08`, `NT-T09` |
| `NTQ-005` | `NT-T10`, `NT-T11`, `NT-T12` |
| `NTQ-006` | `NT-T13`, `NT-T14`, `NT-T15` |
| `NTQ-007` | `NT-T16`, `NT-T17`, `NT-T18` |
| `NTQ-008` | `NT-T19`, `NT-T20`, `NT-T21` |
| `NTQ-009` | `NT-T22`, `NT-T23`, `NT-T24` |
| `NTQ-010` | `NT-T25`, `NT-T26`, `NT-T27` |
| `NTQ-011` | `NT-T28`, `NT-T29`, `NT-T30` |
| `NTQ-012` | `NT-T31`, `NT-T32`, `NT-T33`, `NT-T34` |
| `NTQ-013` | `NT-T35`, `NT-T36` |
| `NTQ-014` | `NT-T37`, `NT-T38`, `NT-T39` |
| `NTQ-015` | `NT-T40`, `NT-T41`, `NT-T42` |
| `NTQ-016` | `NT-T43`, `NT-T44` |
| `NTQ-017` | `NT-T45`, `NT-T46`, `NT-T47` |
| `NTQ-018` | `NT-T49`, `NT-T50` |
| `NTQ-019` | `NT-T61`, `NT-T62`, `NT-T63` |
| `NTQ-020` | `NT-T51`, `NT-T52` |
| `NTQ-021` | `NT-T53`, `NT-T54`, `NT-T55` |
| `NTQ-022` | `NT-T67`, `NT-T48`, `NT-T68`, `NT-T69` |
| `NTQ-023` | `NT-T56`, `NT-T57`, `NT-T58` |
| `NTQ-024` | `NT-T59`, `NT-T60` |
| `NTQ-025` | `NT-T64`, `NT-T65`, `NT-T66` |
| `NTQ-026` | `NT-T70` |

---

## Milestones

### NT-T00 Build Number-Theory Theorem Card Inventory

- Status: Completed
- Depends on: None
- Areas: `proofs/README.md`, theorem-card documentation, AI theorem index sidecars
- Tasks:
  - Create theorem cards for `NT-00` through `NT-24`.
  - Record duplicate-home decisions for finite fields, Chebotarev,
    modularity, Langlands, elliptic curves, algebraic geometry, cryptography,
    and analytic number theory.
  - Label conjectures, conditional theorem forms, bridge interfaces, and
    derived theorem targets separately.
- Deliverables:
  - `proofs/number-theory-theorem-cards.md` number-theory theorem-card
    inventory.
  - Duplicate-home and conjecture-status map in
    `proofs/number-theory-theorem-cards.md`.
- Acceptance criteria:
  - Every roadmap theorem family has a card or an intentionally grouped card.
  - Conjectures are never marked as `L2` derived theorem targets.
  - Sidecars, theorem search, AI output, metadata, and this task document are
    recorded as untrusted.
- Verification:
  - `rg -n "NT-00|NT-24|Riemann hypothesis|Birch|Langlands|sidecar" proofs`
  - `git diff --check`

### NT-T01 Create Number-Theory Namespace Contract And Statement Policy

- Status: Completed
- Depends on: `NT-T00`
- Areas: `Proofs/Ai/NumberTheory/Inventory/`, `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/README.md`
- Tasks:
  - Create `Proofs.Ai.NumberTheory.Inventory` as the concrete
    certificate-backed namespace-policy entry point.
  - Record namespace ownership rules for arithmetic-owned modules and external
    owner namespaces such as `EllipticCurve`, `ModularForms`,
    `GaloisRepresentation`, `AlgebraicGeometry`, and `Algebra.AbstractFiniteField`.
  - Add trusted-boundary wording to theorem cards or module comments.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Inventory` namespace-policy entry point.
  - Namespace ownership contract in
    `proofs/number-theory-theorem-cards.md`.
- Acceptance criteria:
  - Arithmetic objects are ordinary proof-corpus structures, not kernel
    primitives.
  - External owner namespaces are not duplicated under `Proofs.Ai.NumberTheory`.
  - Bridge assumptions and conjectures remain explicitly named.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Inventory`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Inventory`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Proofs.Ai.NumberTheory|kernel primitive|AbstractFiniteField|BridgeAxiom" proofs`
  - `git diff --check`

### NT-T02 Define Integer And Divisibility Core Interface

- Status: Completed
- Depends on: `NT-T01`
- Areas: `Proofs/Ai/NumberTheory/Elementary/`, `Proofs/Ai/NumberTheory/Divisibility/`, `proofs/README.md`
- Tasks:
  - Define `Divides`, sign-normalized divisibility, divisor, multiple,
    nonzero, positivity, and natural/integer translation surfaces.
  - Prove or package divisibility reflexivity, transitivity, sign rules, and
    basic divisor/multiple closure.
  - Keep all simplification as ordinary theorem targets, not kernel behavior.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Elementary`.
  - `Proofs.Ai.NumberTheory.Divisibility`.
- Acceptance criteria:
  - Divisibility facts do not import prime factorization, elliptic curves, or
    modularity.
  - Integer and natural-number variants state coercion and positivity
    hypotheses explicitly.
  - No arithmetic automation is added to the trusted core.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Divisibility`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Divisibility`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Divides|divisibility|kernel primitive" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T03 Add Euclidean Division And Descent Interfaces

- Status: Completed
- Depends on: `NT-T02`
- Areas: `Proofs/Ai/NumberTheory/EuclideanDivision/`, `Proofs/Ai/NumberTheory/Descent/`
- Tasks:
  - State and prove or package integer quotient-remainder existence and
    uniqueness with exact sign and bound hypotheses.
  - Add Euclidean division as the normalized division theorem.
  - Add finite descent and well-founded minimization interfaces needed by gcd,
    continued fractions, and Diophantine proofs.
- Deliverables:
  - `Proofs.Ai.NumberTheory.EuclideanDivision`.
  - Descent/minimization statement interface.
- Acceptance criteria:
  - Quotient-remainder uniqueness does not assume gcd or prime factorization.
  - Descent interfaces are general enough for Diophantine milestones.
  - Algorithm extraction is separated from mathematical existence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.EuclideanDivision`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Descent`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.EuclideanDivision`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Descent`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "EuclideanDivision|quotient|remainder|Descent" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T04 Add Gcd And Lcm Normal Forms

- Status: Completed
- Depends on: `NT-T03`
- Areas: `Proofs/Ai/NumberTheory/Gcd/`, `Proofs/Ai/NumberTheory/Lcm/`
- Tasks:
  - Add gcd existence, uniqueness, symmetry, divisor characterization, and
    normalized sign convention.
  - Add lcm existence, uniqueness, multiple characterization, and gcd-lcm
    product formula under explicit hypotheses.
  - Export the normal forms used by congruence and Diophantine reduction.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Gcd`.
  - `Proofs.Ai.NumberTheory.Lcm`.
- Acceptance criteria:
  - Gcd/lcm statements cite Euclidean division or explicit law evidence.
  - Normalization choices are documented and stable for downstream imports.
  - No theorem assumes Bezout before `NT-T05`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Gcd`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Lcm`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Gcd`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Lcm`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Gcd|Lcm|gcd_lcm|normalized" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T05 Prove Euclid Algorithm And Bezout Package

- Status: Completed
- Depends on: `NT-T04`
- Areas: `Proofs/Ai/NumberTheory/EuclideanAlgorithm/`, `Proofs/Ai/NumberTheory/Bezout/`
- Tasks:
  - Add Euclid algorithm correctness and extended Euclid correctness.
  - Add Bezout identity and gcd linear-combination characterization.
  - Add coprime iff a linear combination equals `1`.
- Deliverables:
  - `Proofs.Ai.NumberTheory.EuclideanAlgorithm`.
  - `Proofs.Ai.NumberTheory.Bezout`.
- Acceptance criteria:
  - Algorithmic correctness is separated from runtime complexity.
  - Bezout does not import prime factorization or CRT.
  - Coprimality statements expose integer/natural variants explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.EuclideanAlgorithm`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Bezout`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.EuclideanAlgorithm`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Bezout`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Bezout|EuclideanAlgorithm|Coprime|linear_combination" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T06 Add Linear Diophantine Equation Package

- Status: Completed
- Depends on: `NT-T05`
- Areas: `Proofs/Ai/NumberTheory/LinearDiophantine/`
- Tasks:
  - Prove solvability criterion for `a x + b y = c`.
  - Add general solution formula under gcd divisibility hypotheses.
  - Export reusable normal-form lemmas for later Diophantine milestones.
- Deliverables:
  - `Proofs.Ai.NumberTheory.LinearDiophantine`.
- Acceptance criteria:
  - The theorem depends on gcd/Bezout, not on a hidden Diophantine solver.
  - All zero and sign edge cases are stated or intentionally split.
  - The module is reusable outside this linear Diophantine milestone.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.LinearDiophantine`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.LinearDiophantine`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "LinearDiophantine|ax|Bezout|Coprime" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T07 Add Prime And Composite Definitions

- Status: Completed
- Depends on: `NT-T05`
- Areas: `Proofs/Ai/NumberTheory/Prime/`, `Proofs/Ai/NumberTheory/Composite/`
- Tasks:
  - Define natural-number and integer primality, composite, unit, associated,
    and sign-normalized prime predicates.
  - Prove `1` is not prime, primes have only trivial divisors, and composites
    have nontrivial divisors.
  - Align terminology with `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Prime`.
  - `Proofs.Ai.NumberTheory.Composite`.
  - Prime/composite README notes.
- Acceptance criteria:
  - Prime predicates do not conflict with UFD-local `PrimeElement`.
  - Unit and sign normalization are explicit.
  - No unique factorization theorem is assumed yet.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Prime`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Composite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Prime`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Composite`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "PrimeNat|Composite|PrimeElement|unit|associated" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T08 Bridge Prime Divisibility And Factor Extraction

- Status: Completed
- Depends on: `NT-T07`, `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`
- Areas: `Proofs/Ai/NumberTheory/UfdBridge/`, `Proofs/Ai/NumberTheory/Factorization/`
- Tasks:
  - Bridge natural/integer prime predicates to explicit UFD factorization law
    packages where useful.
  - Prove Euclid's lemma and prime divisibility of products.
  - Add existence of prime factors for composite numbers.
- Deliverables:
  - `Proofs.Ai.NumberTheory.UfdBridge`.
  - `Proofs.Ai.NumberTheory.Factorization`.
  - Prime-factor extraction theorem package.
- Acceptance criteria:
  - The bridge imports the abstract UFD package without making UFD depend on
    number theory.
  - Euclid's lemma is derived from Bezout/UFD evidence, not assumed.
  - Factor extraction can be consumed by Diophantine equations and arithmetic
    functions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Factorization`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.UfdBridge`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Factorization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.UfdBridge`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Factorization`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "UfdBridge|Euclid|prime_divides|Factorization" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T09 Prove Fundamental Theorem Of Arithmetic And Prime Infinitude

- Status: Completed
- Depends on: `NT-T08`
- Areas: `Proofs/Ai/NumberTheory/Factorization/`, `Proofs/Ai/NumberTheory/PrimeInfinitude/`
- Tasks:
  - Prove existence and uniqueness of prime factorization with unit/sign
    normalization.
  - Derive divisor-count, divisor-sum, gcd, and lcm formulas from
    factorization.
  - Prove Euclid's infinitude of primes and square-root bound for composite
    prime factors.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Factorization` FTA theorem package.
  - `Proofs.Ai.NumberTheory.PrimeInfinitude`.
  - Prime-infinitude theorem package.
- Acceptance criteria:
  - FTA is a derived theorem, not a theorem-shaped axiom.
  - Prime infinitude does not depend on analytic number theory or Chebotarev.
  - Factorization formulas expose finite-list or finite-multiset evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimeInfinitude`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Factorization`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimeInfinitude`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Factorization`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimeInfinitude`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "fundamental_theorem|prime_factorization|PrimeInfinitude|sqrt_bound" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T10 Define Congruence Algebra

- Status: Completed
- Depends on: `NT-T05`
- Areas: `Proofs/Ai/NumberTheory/Congruence/`
- Tasks:
  - Define congruence modulo `n` and prove equivalence relation laws.
  - Prove preservation under addition, multiplication, negation, and powers.
  - Add cancellation and division conditions using gcd hypotheses.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Congruence`.
- Acceptance criteria:
  - Congruence statements cite divisibility and gcd facts explicitly.
  - Division and cancellation do not hide coprimality requirements.
  - Powers reuse ordinary exponent laws, not simplifier primitives.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Congruence`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Congruence`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "Congruent|modulo|cancellation|Coprime" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T11 Add Residue Ring And Unit Group Interfaces

- Status: Completed
- Depends on: `NT-T10`, `Proofs.Ai.Algebra.AbstractRing`
- Areas: `Proofs/Ai/NumberTheory/ResidueRing/`, `Proofs/Ai/NumberTheory/ModularGroup/`
- Tasks:
  - Build residue classes modulo `n` and residue-ring law package.
  - Add unit modulo `n`, reduced residue class, and unit-group interfaces.
  - Expose quotient/core-feature requirements in package reports.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ResidueRing`.
  - `Proofs.Ai.NumberTheory.ModularGroup`.
- Acceptance criteria:
  - Quotient requirements are visible and deterministic.
  - Unit-group facts do not assume Euler's theorem.
  - Residue-ring construction is reusable by CRT and finite unit-group tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ResidueRing`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ResidueRing`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ModularGroup`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ModularGroup`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "ResidueRing|unit modulo|quotient|core-feature" proofs/Proofs/Ai/NumberTheory proofs/generated proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T12 Specialize Chinese Remainder Theorem

- Status: Completed
- Depends on: `NT-T11`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder`
- Areas: `Proofs/Ai/NumberTheory/ChineseRemainder/`
- Tasks:
  - Prove linear congruence solvability and number of solutions.
  - Specialize abstract ring CRT to integer residue rings for pairwise coprime
    moduli.
  - Add generalized CRT for compatible systems and constructive CRT interface.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ChineseRemainder`.
- Acceptance criteria:
  - Construction and uniqueness are separated.
  - The abstract ring CRT is imported rather than duplicated.
  - Garner-style interface is marked algorithmic if no executable function is
    verified yet.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ChineseRemainder`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ChineseRemainder`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "ChineseRemainder|linear_congruence|Garner|AbstractRingChineseRemainder" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T13 Add Euler Phi And Finite Unit Group Order

- Status: Completed
- Depends on: `NT-T11`, `NT-T09`, finite group order interfaces
- Areas: `Proofs/Ai/NumberTheory/Phi/`, `Proofs/Ai/NumberTheory/ModularGroup/`
- Tasks:
  - Define Euler `phi` as the order of the unit group modulo `n`.
  - Prove the `phi` formula from prime factorization.
  - Add Lagrange specialization for finite unit groups.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Phi`.
  - Unit-group order theorem package.
- Acceptance criteria:
  - `phi` statements expose finite-set/cardinality evidence.
  - The proof route imports finite group facts, not analytic number theory.
  - Prime-factor formulas cite FTA.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Phi`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Phi`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "EulerPhi|unit group|Lagrange|prime_factorization" proofs/Proofs/Ai/NumberTheory proofs/README.md`
  - `git diff --check`
  - `./scripts/check-fast.sh`
  - `./scripts/check-corpus-authoring.sh`

### NT-T14 Prove Fermat, Euler, Wilson, And Carmichael Theorems

- Status: Completed
- Depends on: `NT-T13`
- Areas: `Proofs/Ai/NumberTheory/FermatEulerWilson/`, `Proofs/Ai/NumberTheory/Carmichael/`
- Tasks:
  - Prove Fermat's little theorem from finite unit-group order.
  - Prove Euler's theorem and Fermat-Euler combined form.
  - Add Wilson theorem, Wilson converse, Carmichael function, and exponent
    theorem interfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.FermatEulerWilson`.
  - `Proofs.Ai.NumberTheory.Carmichael`.
- Acceptance criteria:
  - Fermat's little theorem is derived, not added as a modular arithmetic axiom.
  - Wilson statements state prime and modulus hypotheses exactly.
  - Carmichael function does not duplicate Euler `phi`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.FermatEulerWilson`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatEulerWilson`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Carmichael`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Carmichael`
  - `rg -n "Fermat|Euler theorem|Wilson|Carmichael" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T15 Add RSA And Primality-Test Interfaces

- Status: Completed
- Depends on: `NT-T14`
- Areas: `Proofs/Ai/NumberTheory/Rsa/`, `Proofs/Ai/NumberTheory/PrimalityTest/`
- Tasks:
  - Prove RSA correctness under coprime and key-congruence hypotheses.
  - Add pseudoprime, Carmichael-number, Korselt, Fermat test, and
    Miller-Rabin theorem surfaces.
  - Separate mathematical soundness from runtime and probabilistic-security
    claims.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Rsa`.
  - `Proofs.Ai.NumberTheory.PrimalityTest`.
- Acceptance criteria:
  - RSA correctness states modulus factorization and coprimality hypotheses.
  - Security assumptions are not marked as derived certificates.
  - Primality tests do not call external randomness or runtime solvers from
    trusted code.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimalityTest`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimalityTest`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Rsa`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Rsa`
  - `rg -n "RSA|Korselt|Miller|pseudoprime|security assumption" proofs/Proofs/Ai proofs/README.md`

### NT-T16 Add Element Order And Primitive Root Basics

- Status: Completed
- Depends on: `NT-T14`, finite cyclic group interfaces
- Areas: `Proofs/Ai/NumberTheory/PrimitiveRoot/`
- Tasks:
  - Add element order modulo `n` and relation to powers.
  - Define primitive roots and generators of cyclic residue unit groups.
  - Prove generator-count formula for abstract cyclic groups when available.
- Deliverables:
  - `Proofs.Ai.NumberTheory.PrimitiveRoot` base package.
- Acceptance criteria:
  - Order facts depend on group APIs and residue unit groups explicitly.
  - Primitive-root definitions do not assume existence.
  - Generator-count theorem is abstract where possible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimitiveRoot`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimitiveRoot`
  - `rg -n "PrimitiveRoot|element_order|generator|cyclic" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T17 Add Primitive Root Existence And Classification Route

- Status: Completed
- Depends on: `NT-T16`, `NT-T12`
- Areas: `Proofs/Ai/NumberTheory/PrimitiveRoot/`, `Proofs/Ai/NumberTheory/PrimePower/`
- Tasks:
  - Add primitive roots modulo odd primes.
  - Add primitive roots modulo prime powers.
  - Add classification interface for moduli admitting primitive roots.
- Deliverables:
  - Primitive-root existence and classification theorem package.
- Acceptance criteria:
  - Prime-power dependencies and CRT dependencies are explicit.
  - Classification theorem is not used by earlier finite unit-group theorems.
  - If full classification is too large, split prime and prime-power stages.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimitiveRoot`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimitiveRoot`
  - `rg -n "prime_power|primitive_root_exists|classification|ChineseRemainder" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T18 Add Dirichlet Characters And Gauss Sums

- Status: Completed
- Depends on: `NT-T16`
- Areas: `Proofs/Ai/NumberTheory/Character/`, `Proofs/Ai/NumberTheory/GaussSum/`
- Tasks:
  - Add Dirichlet character and character group interfaces.
  - Prove or package character orthogonality relations.
  - Add basic Gauss sum identities and discrete-log statement surfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Character`.
  - `Proofs.Ai.NumberTheory.GaussSum`.
- Acceptance criteria:
  - Character orthogonality does not depend on analytic `L`-functions.
  - Discrete logarithm algorithms are statement surfaces, not trusted solvers.
  - Gauss sum identities name coefficient ring and additive character data.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Character`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Character`
  - `rg -n "DirichletCharacter|orthogonality|GaussSum|discrete_log" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T19 Add Quadratic Residue And Legendre Symbol Package

- Status: Completed
- Depends on: `NT-T16`
- Areas: `Proofs/Ai/NumberTheory/QuadraticResidue/`, `Proofs/Ai/NumberTheory/Legendre/`
- Tasks:
  - Define quadratic residues and nonresidues.
  - Define Legendre symbol and prove multiplicativity.
  - Prove Euler criterion and count quadratic residues modulo odd primes.
- Deliverables:
  - `Proofs.Ai.NumberTheory.QuadraticResidue`.
  - `Proofs.Ai.NumberTheory.Legendre`.
- Acceptance criteria:
  - Odd-prime assumptions are stated explicitly.
  - Legendre symbol API is separate from Jacobi symbol API.
  - Euler criterion cites finite cyclic group facts.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Legendre`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Legendre`
  - `rg -n "QuadraticResidue|Legendre|Euler criterion|odd prime" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T20 Prove Gauss Lemma And Quadratic Reciprocity

- Status: Completed
- Depends on: `NT-T19`
- Areas: `Proofs/Ai/NumberTheory/QuadraticReciprocity/`
- Tasks:
  - Prove Gauss lemma.
  - Prove supplementary laws for `-1` and `2`.
  - Prove quadratic reciprocity for distinct odd primes.
- Deliverables:
  - `Proofs.Ai.NumberTheory.QuadraticReciprocity`.
- Acceptance criteria:
  - The first proof route is recorded so alternative proofs become aliases or
    later theorem cards.
  - Distinctness, oddness, and primality hypotheses are explicit.
  - Reciprocity is not assumed by primitive-root or character milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.QuadraticReciprocity`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.QuadraticReciprocity`
  - `rg -n "Gauss lemma|quadratic_reciprocity|supplementary" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T21 Add Jacobi Symbol And Probabilistic-Test Interfaces

- Status: Completed
- Depends on: `NT-T20`
- Areas: `Proofs/Ai/NumberTheory/Jacobi/`, `Proofs/Ai/NumberTheory/PrimalityTest/`
- Tasks:
  - Define Jacobi symbol and prove multiplicativity.
  - Add theorem separating Jacobi symbol from actual quadratic residuosity.
  - Add Solovay-Strassen interface with randomness and soundness assumptions
    explicit.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Jacobi`.
  - Solovay-Strassen theorem surface.
- Acceptance criteria:
  - Jacobi and Legendre are not interchangeable APIs.
  - Probabilistic claims are not accepted as deterministic security theorems.
  - All composite/modulus assumptions are named.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Jacobi`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Jacobi`
  - `rg -n "Jacobi|Solovay|Strassen|quadratic residuosity" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T22 Define Arithmetic Functions And Multiplicativity

- Status: Completed
- Depends on: `NT-T09`
- Areas: `Proofs/Ai/NumberTheory/ArithmeticFunction/`
- Tasks:
  - Define divisor-count, divisor-sum, Euler `phi`, Mobius, Liouville, von
    Mangoldt, and Carmichael functions.
  - Define multiplicative and completely multiplicative functions.
  - Add divisor and sigma formulas from prime factorization.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ArithmeticFunction`.
- Acceptance criteria:
  - Divisor sums expose finite-support evidence.
  - Multiplicativity statements state coprimality hypotheses.
  - No complex analysis or Dirichlet series imports are required.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ArithmeticFunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ArithmeticFunction`
  - `rg -n "ArithmeticFunction|Mobius|Liouville|von_Mangoldt|multiplicative" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T23 Add Dirichlet Convolution Algebra

- Status: Completed
- Depends on: `NT-T22`
- Areas: `Proofs/Ai/NumberTheory/DirichletConvolution/`
- Tasks:
  - Define Dirichlet convolution.
  - Prove associativity, commutativity, identity, and inverse interfaces.
  - Package finite divisor-sum rearrangement lemmas.
- Deliverables:
  - `Proofs.Ai.NumberTheory.DirichletConvolution`.
- Acceptance criteria:
  - Convolution is algebraic and does not import infinite series.
  - Finite-support or divisor-finiteness evidence is explicit.
  - Inverse statements do not assume Mobius inversion before `NT-T24`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DirichletConvolution`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletConvolution`
  - `rg -n "DirichletConvolution|divisor sum|identity|inverse" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T24 Prove Mobius Inversion And Euler Product Interface

- Status: Completed
- Depends on: `NT-T23`
- Areas: `Proofs/Ai/NumberTheory/Mobius/`, `Proofs/Ai/NumberTheory/EulerProduct/`
- Tasks:
  - Prove Mobius inversion and generalized Mobius inversion.
  - Add Euler product statement interface for multiplicative Dirichlet series.
  - Mark analytic convergence prerequisites as deferred to analytic number
    theory tasks.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Mobius`.
  - Algebraic Euler-product interface.
- Acceptance criteria:
  - Mobius inversion is algebraic and certificate-backed.
  - Euler product convergence is not claimed without analysis prerequisites.
  - The interface can feed zeta and Dirichlet `L` milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Mobius`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Mobius`
  - `rg -n "Mobius inversion|EulerProduct|Dirichlet series|convergence" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T25 Add Finite Continued Fractions For Rationals

- Status: Completed
- Depends on: `NT-T03`
- Areas: `Proofs/Ai/NumberTheory/ContinuedFraction/`
- Tasks:
  - Define finite continued fractions and convergent recurrence relations.
  - Prove finite continued fraction expansion for rationals.
  - Add normalized uniqueness theorem for finite expansions.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ContinuedFraction`.
- Acceptance criteria:
  - The first route depends only on Euclidean division and ordered/rational
    interfaces.
  - Normalization rules make the final partial quotient convention explicit.
  - No real-analysis theorem is needed for finite rational expansions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ContinuedFraction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ContinuedFraction`
  - `rg -n "ContinuedFraction|convergent|rational|EuclideanDivision" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T26 Add Infinite Continued Fractions And Pell Interfaces

- Status: Completed
- Depends on: `NT-T25`, ordered-field and real-analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/ContinuedFraction/`, `Proofs/Ai/NumberTheory/Pell/`
- Tasks:
  - Add infinite continued-fraction expansion interface for irrationals.
  - Add best-approximation theorem surface for convergents.
  - Add quadratic irrational periodicity and Pell equation existence/structure
    interfaces.
- Deliverables:
  - Infinite continued-fraction interface.
  - `Proofs.Ai.NumberTheory.Pell`.
- Acceptance criteria:
  - Pell statements name positivity, nonsquare, and normalization hypotheses.
  - Real-analysis dependencies are explicit.
  - Interface-level theorems are not confused with derived certificates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Pell`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Pell`
  - `rg -n "Pell|quadratic irrational|periodic|best approximation" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T27 Add Diophantine Approximation Statement Interfaces

- Status: Completed
- Depends on: `NT-T26`, measure and real-analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/DiophantineApproximation/`
- Tasks:
  - Add Dirichlet approximation and simultaneous approximation interfaces.
  - Add Liouville, Roth, Schmidt, Khintchine, Duffin-Schaeffer, Baker,
    Lindemann-Weierstrass, and geometry-of-numbers theorem surfaces.
  - Separate measure-theoretic metric approximation dependencies from
    algebraic approximation dependencies.
- Deliverables:
  - `Proofs.Ai.NumberTheory.DiophantineApproximation`.
- Acceptance criteria:
  - Advanced theorems are `L1` interfaces until analytic and measure
    prerequisites are certified.
  - Metric, measure, and real-field assumptions are named.
  - Transcendence results are not used by elementary number theory.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DiophantineApproximation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DiophantineApproximation`
  - `rg -n "DiophantineApproximation|Dirichlet approximation|Roth|Khintchine|Baker" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T28 Add Pythagorean Triples And Sums-Of-Squares Entry Points

- Status: Completed
- Depends on: `NT-T06`, `NT-T07`
- Areas: `Proofs/Ai/NumberTheory/Diophantine/`, `Proofs/Ai/NumberTheory/SumsOfSquares/`
- Tasks:
  - Add Pythagorean triple classification and primitive triple formula.
  - Add Fermat two-square theorem statement route.
  - Reuse existing geometry/algebra identities where available.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Diophantine`.
  - `Proofs.Ai.NumberTheory.SumsOfSquares` entry point.
- Acceptance criteria:
  - Pythagorean classification uses gcd/coprime facts explicitly.
  - Two-square theorem dependencies on quadratic residues are named.
  - No Diophantine solver primitive is introduced.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Diophantine`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Diophantine`
  - `rg -n "Pythagorean|SumsOfSquares|two_square|Coprime" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T29 Add Three-Square, Four-Square, Waring, And Coin Interfaces

- Status: Completed
- Depends on: `NT-T28`
- Areas: `Proofs/Ai/NumberTheory/SumsOfSquares/`, `Proofs/Ai/NumberTheory/Waring/`
- Tasks:
  - Add Lagrange four-square theorem route.
  - Add Legendre three-square, Waring, Hilbert-Waring, and Frobenius coin
    problem interfaces.
  - Label construction-heavy classical theorems as `L1` until prerequisites
    are certified.
- Deliverables:
  - Sums-of-squares theorem package.
  - `Proofs.Ai.NumberTheory.Waring` statement interface.
- Acceptance criteria:
  - Each theorem states positivity and representability hypotheses explicitly.
  - Theorems are not imported into downstream routes as hidden assumptions.
  - Coin problem theorem states gcd and nonnegative-combination assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.SumsOfSquares`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.SumsOfSquares`
  - `rg -n "four_square|three_square|Waring|Frobenius coin" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T30 Add Additive Number Theory And Combinatorics Interfaces

- Status: Completed
- Depends on: `NT-T29`
- Areas: `Proofs/Ai/NumberTheory/Additive/`, `Proofs/Ai/NumberTheory/Combinatorial/`
- Tasks:
  - Add Cauchy-Davenport, Kneser, Vosper, Freiman, Plunnecke-Ruzsa,
    Szemeredi, Green-Tao, van der Waerden, Hindman, and
    Erdos-Ginzburg-Ziv theorem surfaces.
  - Parameterize statements by ambient finite group, density, or field
    assumptions.
  - Mark analytic or ergodic dependencies explicitly.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Additive`.
  - Additive-combinatorics statement map.
- Acceptance criteria:
  - Advanced additive theorems remain `L1` until prerequisites are present.
  - Ambient structure and density assumptions are never implicit.
  - The module does not duplicate finite-field core laws.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Additive`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Additive`
  - `rg -n "Cauchy|Davenport|Kneser|Szemeredi|Green|Tao" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T31 Add Dirichlet Series And Algebraic Euler Product Layer

- Status: Completed
- Depends on: `NT-T24`, analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/DirichletSeries/`, `Proofs/Ai/NumberTheory/EulerProduct/`
- Tasks:
  - Define Dirichlet series and abscissa statement interfaces.
  - Connect multiplicative arithmetic functions to Euler product theorem
    surfaces.
  - Keep convergence, analytic continuation, and Tauberian inputs separate.
- Deliverables:
  - `Proofs.Ai.NumberTheory.DirichletSeries`.
- Acceptance criteria:
  - Algebraic Euler-product facts do not claim analytic convergence.
  - Series convergence depends on analysis roadmap theorem surfaces.
  - The layer can feed zeta and Dirichlet `L` modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DirichletSeries`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletSeries`
  - `rg -n "DirichletSeries|EulerProduct|abscissa|convergence" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T32 Add Riemann Zeta Function Interfaces

- Status: Completed
- Depends on: `NT-T31`, complex-analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/Zeta/`
- Tasks:
  - Add Riemann zeta definition and half-plane Euler product interface.
  - Add analytic continuation and functional equation statement interfaces.
  - Add zero, explicit formula, and Riemann-von Mangoldt theorem surfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Zeta`.
- Acceptance criteria:
  - Analytic continuation is `L1` until complex analysis is certified.
  - Riemann hypothesis remains a conjectural statement or conditional
    assumption, not a theorem target.
  - The zeta Euler product imports `NT-T31`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Zeta`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Zeta`
  - `rg -n "Riemann zeta|Euler product|functional equation|Riemann hypothesis" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T33 Add Chebyshev And Prime Number Theorem Interfaces

- Status: Completed
- Depends on: `NT-T32`, Tauberian-analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/PrimeNumberTheorem/`
- Tasks:
  - Add Chebyshev functions and elementary estimates.
  - Add prime number theorem, zero-free region, and de la Vallee Poussin
    theorem interfaces.
  - Record that elementary prime facts do not depend on PNT.
- Deliverables:
  - `Proofs.Ai.NumberTheory.PrimeNumberTheorem`.
- Acceptance criteria:
  - PNT is not an input to `NT-T07` through `NT-T09`.
  - Zero-free regions are named analytic assumptions until derived.
  - Tauberian theorem dependencies are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimeNumberTheorem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimeNumberTheorem`
  - `rg -n "PrimeNumberTheorem|Chebyshev|zero-free|Tauberian" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T34 Add Dirichlet L-Functions And Arithmetic Progression Interfaces

- Status: Completed
- Depends on: `NT-T18`, `NT-T31`, `NT-T33`
- Areas: `Proofs/Ai/NumberTheory/DirichletL/`, `Proofs/Ai/NumberTheory/ArithmeticProgressionPrime/`
- Tasks:
  - Define Dirichlet `L`-functions from characters.
  - Add Euler product, analytic continuation, functional equation, and
    `L(1, chi) != 0` interfaces.
  - Add Dirichlet theorem for primes in arithmetic progressions and arithmetic
    progression PNT statement surfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.DirichletL`.
  - Arithmetic progression prime theorem surface.
- Acceptance criteria:
  - Character definitions import `NT-T18` rather than redefining characters.
  - Generalized Riemann hypothesis remains conjectural or conditional.
  - Nonvanishing hypotheses and primitive-character assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DirichletL`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletL`
  - `rg -n "DirichletL|L(1|arithmetic progression|GRH" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T35 Add Sieve Theory Interfaces

- Status: Completed
- Depends on: `NT-T31`, `NT-T33`
- Areas: `Proofs/Ai/NumberTheory/Sieve/`
- Tasks:
  - Add Brun sieve, Selberg sieve, large sieve, and fundamental lemma
    interfaces.
  - Add Brun theorem, twin-prime reciprocal convergence, Chen theorem, GPY,
    Zhang, and Maynard-Tao theorem surfaces.
  - Expose parity-problem limitations where relevant.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Sieve`.
- Acceptance criteria:
  - Error terms and asymptotic notation are explicit theorem inputs.
  - Sieve results do not imply unresolved conjectures as derived theorems.
  - Analytic dependencies are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Sieve`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Sieve`
  - `rg -n "Sieve|Brun|Selberg|large sieve|Maynard|Tao" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T36 Add Circle Method And Additive Prime Interfaces

- Status: Completed
- Depends on: `NT-T35`, `NT-T30`
- Areas: `Proofs/Ai/NumberTheory/CircleMethod/`, `Proofs/Ai/NumberTheory/AdditivePrime/`
- Tasks:
  - Add Hardy-Littlewood circle-method theorem surface.
  - Add Vinogradov three-primes and weak Goldbach interfaces.
  - Record harmonic-analysis and exponential-sum dependencies explicitly.
- Deliverables:
  - `Proofs.Ai.NumberTheory.CircleMethod`.
  - `Proofs.Ai.NumberTheory.AdditivePrime`.
- Acceptance criteria:
  - Major/minor arc and asymptotic assumptions are named.
  - Weak Goldbach is not used by elementary additive theorems.
  - Interfaces remain conditional until analytic prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.CircleMethod`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.CircleMethod`
  - `rg -n "CircleMethod|Vinogradov|Goldbach|major arc|minor arc" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T37 Add Algebraic Integer And Number Field Interfaces

- Status: Completed
- Depends on: `NT-T09`, field-theory roadmap milestones through field extension
- Areas: `Proofs/Ai/NumberTheory/AlgebraicInteger/`, `Proofs/Ai/NumberTheory/NumberField/`
- Tasks:
  - Add algebraic number, algebraic integer, number field, and ring of
    integers interfaces.
  - Prove or package algebraic integers form a ring.
  - Add rational algebraic integer implies integer theorem surface.
- Deliverables:
  - `Proofs.Ai.NumberTheory.AlgebraicInteger`.
  - `Proofs.Ai.NumberTheory.NumberField`.
- Acceptance criteria:
  - Field-extension dependencies point to `develop/proof-corpus-field-theory-roadmap.md`.
  - Algebraic integer ring structure is not a kernel primitive.
  - Rational-integer theorem states embedding/coercion assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.AlgebraicInteger`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.AlgebraicInteger`
  - `rg -n "AlgebraicInteger|NumberField|ring_of_integers|field-theory" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T38 Add Norm Trace Discriminant And Dedekind Domain Route

- Status: Completed
- Depends on: `NT-T37`, linear algebra and ideal APIs
- Areas: `Proofs/Ai/NumberTheory/NumberField/`, `Proofs/Ai/NumberTheory/DedekindDomain/`
- Tasks:
  - Add norm, trace, discriminant, and integral-basis interfaces.
  - Add ring-of-integers Dedekind-domain theorem surface.
  - Record finite-dimensional vector-space dependencies explicitly.
- Deliverables:
  - Number-field invariant theorem package.
  - `Proofs.Ai.NumberTheory.DedekindDomain`.
- Acceptance criteria:
  - Dedekind-domain ideal factorization is not assumed as a definition.
  - Norm/trace/discriminant statements name basis and field-extension data.
  - Linear-algebra imports are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DedekindDomain`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DedekindDomain`
  - `rg -n "Norm|Trace|Discriminant|DedekindDomain|integral basis" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T39 Add Ideal Factorization, Class Group, Unit, And Class Number Interfaces

- Status: Completed
- Depends on: `NT-T38`
- Areas: `Proofs/Ai/NumberTheory/ClassGroup/`, `Proofs/Ai/NumberTheory/UnitTheorem/`
- Tasks:
  - Add ideal factorization, uniqueness, fractional ideal group, and ideal
    class group interfaces.
  - Add class-number finiteness and Minkowski-bound interfaces.
  - Add Dirichlet unit theorem and class-number formula statement surfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ClassGroup`.
  - Unit/class-number theorem surfaces.
- Acceptance criteria:
  - Class group quotient requirements are visible in core-feature reports.
  - Minkowski and geometry-of-numbers dependencies are explicit.
  - Analytic class-number formula remains `L1` until analytic prerequisites
    are certified.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ClassGroup`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ClassGroup`
  - `rg -n "ClassGroup|fractional ideal|class number|Dirichlet unit|Minkowski" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T40 Add Valuation And p-adic Metric Interfaces

- Status: Completed
- Depends on: `NT-T37`, topology and metric prerequisites
- Areas: `Proofs/Ai/NumberTheory/Valuation/`, `Proofs/Ai/NumberTheory/Padic/`
- Tasks:
  - Define p-adic valuation and p-adic absolute value.
  - Prove or package non-Archimedean metric laws.
  - Add completion and p-adic field construction interface.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Valuation`.
  - `Proofs.Ai.NumberTheory.Padic`.
- Acceptance criteria:
  - Valuation laws are algebraic before metric completion statements.
  - Completion dependencies are imported from topology/analysis roadmaps.
  - No local-field theorem is used to prove the basic valuation laws.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Valuation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Valuation`
  - `rg -n "Valuation|p_adic|non_Archimedean|completion" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T41 Add Hensel Lemma And Local Field Structure Interfaces

- Status: Completed
- Depends on: `NT-T40`
- Areas: `Proofs/Ai/NumberTheory/Hensel/`, `Proofs/Ai/NumberTheory/LocalField/`
- Tasks:
  - Add Hensel lemma in the chosen formulation.
  - Add Ostrowski theorem, DVR, complete DVR, local-field structure,
    unramified extension, and totally ramified extension interfaces.
  - Name derivative, valuation, and completeness hypotheses exactly.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Hensel`.
  - `Proofs.Ai.NumberTheory.LocalField`.
- Acceptance criteria:
  - Hensel lemma is not a generic root-finder primitive.
  - Local field structure statements are interface-level until construction
    dependencies are available.
  - Ramification vocabulary is shared with Galois representation tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Hensel`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Hensel`
  - `rg -n "Hensel|LocalField|DVR|ramified|Ostrowski" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T42 Add p-adic Analysis And p-adic Measure Interfaces

- Status: Completed
- Depends on: `NT-T41`, analysis and measure prerequisites
- Areas: `Proofs/Ai/NumberTheory/PadicAnalysis/`, `Proofs/Ai/NumberTheory/PadicMeasure/`
- Tasks:
  - Add p-adic exponential, logarithm, Newton polygon, Strassmann, and
    Weierstrass preparation theorem surfaces.
  - Add Mahler expansion, Amice transform, p-adic measure, and
    Kubota-Leopoldt p-adic `L`-function interfaces.
  - Keep p-adic convergence dependent on explicit norm and series theorems.
- Deliverables:
  - `Proofs.Ai.NumberTheory.PadicAnalysis`.
  - p-adic measure and p-adic `L`-function interfaces.
- Acceptance criteria:
  - p-adic analytic functions are not trusted primitives.
  - Measure dependencies point to the measure-theory roadmap.
  - p-adic `L`-function interpolation assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PadicAnalysis`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PadicAnalysis`
  - `rg -n "PadicAnalysis|Strassmann|Weierstrass|Mahler|Kubota" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T43 Add Class Field Theory Reciprocity Surfaces

- Status: Completed
- Depends on: `NT-T39`, `NT-T41`, Galois and cohomology prerequisites
- Areas: `Proofs/Ai/NumberTheory/ClassField/Local/`, `Proofs/Ai/NumberTheory/ClassField/Global/`
- Tasks:
  - Add Artin map, local reciprocity, ideles, idele class group, and global
    reciprocity interfaces.
  - Add Hilbert class field, Takagi existence, and Kronecker-Weber theorem
    surfaces.
  - Use explicit bridge names for early development interfaces if needed.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ClassField.Local`.
  - `Proofs.Ai.NumberTheory.ClassField.Global`.
- Acceptance criteria:
  - Reciprocity maps state domain, codomain, normalization, and functoriality.
  - No class field theorem is imported under a generic algebra name.
  - Bridge assumptions are rejected by final promotion gates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ClassField.Local`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ClassField.Local`
  - `rg -n "ClassField|Artin map|reciprocity|idele|Kronecker" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T44 Add Hilbert 90, Norm-Residue, Brauer, And Tate Cohomology Interfaces

- Status: Completed
- Depends on: `NT-T43`
- Areas: `Proofs/Ai/NumberTheory/ClassField/Cohomology/`, `Proofs/Ai/GaloisCohomology/Basic/`
- Tasks:
  - Add Hilbert theorem 90 and norm-residue symbol surfaces.
  - Add Hasse norm theorem, Grunwald-Wang, Brauer group, and Tate cohomology
    interfaces.
  - Link cohomology terms to the class field theory reciprocity route.
- Deliverables:
  - Class-field cohomology interface package.
- Acceptance criteria:
  - Cohomology dependencies are not hidden in class field modules.
  - Theorems are interface-level until Galois cohomology foundations exist.
  - Norm-residue notation states local/global context.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.GaloisCohomology.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisCohomology.Basic`
  - `rg -n "Hilbert 90|Norm residue|Brauer|Tate cohomology|Hasse norm" proofs/Proofs/Ai proofs/README.md`

### NT-T45 Add Elliptic Curve Basic And Group Law Interfaces

- Status: Completed
- Depends on: `NT-T37`, existing algebra modules
- Areas: `Proofs/Ai/EllipticCurve/Basic/`, `Proofs/Ai/EllipticCurve/GroupLaw/`
- Tasks:
  - Add Weierstrass model and nonsingularity interfaces.
  - Add elliptic-curve point group law theorem surface.
  - Keep general elliptic-curve APIs independent of specialized Frey modules.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Basic`.
  - `Proofs.Ai.EllipticCurve.GroupLaw`.
- Acceptance criteria:
  - Group law does not depend on modularity, Ribet, or bridge axioms.
  - Field and polynomial assumptions are explicit.
  - The API is usable outside specialized final-theorem routes.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic`
  - `rg -n "EllipticCurve|Weierstrass|nonsingular|GroupLaw" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

### NT-T46 Add Elliptic Curve Reduction, Semistability, And Height Interfaces

- Status: Completed
- Depends on: `NT-T45`, `NT-T41`
- Areas: `Proofs/Ai/EllipticCurve/Reduction/`, `Proofs/Ai/EllipticCurve/Semistable/`, `Proofs/Ai/EllipticCurve/Height/`
- Tasks:
  - Add conductor, reduction type, minimal model, and semistability interfaces.
  - Add height and Neron-Tate height theorem surfaces.
  - Keep module names general enough for arithmetic-geometry reuse.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Reduction`.
  - `Proofs.Ai.EllipticCurve.Semistable`.
  - `Proofs.Ai.EllipticCurve.Height`.
- Acceptance criteria:
  - Semistability is a general elliptic-curve predicate, not Frey-specific.
  - Local-field and valuation dependencies are explicit.
  - Height statements name field and positivity hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Semistable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Semistable`
  - `rg -n "Conductor|Reduction|Semistable|Height|Neron" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

### NT-T47 Add Mordell-Weil, Selmer, Tate Module, And Pairing Interfaces

- Status: Completed
- Depends on: `NT-T46`, `NT-T21`
- Areas: `Proofs/Ai/EllipticCurve/MordellWeil/`, `Proofs/Ai/EllipticCurve/GaloisRepresentation/`
- Tasks:
  - Add torsion, Nagell-Lutz, weak Mordell-Weil, and Mordell-Weil interfaces.
  - Add Selmer group and Tate-Shafarevich group statement surfaces.
  - Add Tate module and Weil pairing interfaces.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.MordellWeil`.
  - `Proofs.Ai.EllipticCurve.GaloisRepresentation`.
- Acceptance criteria:
  - Selmer definitions are shared with Iwasawa and Galois representation tasks.
  - Weil pairing nondegeneracy does not require cryptographic assumptions.
  - Mordell-Weil remains interface-level until height/descent prerequisites
    are derived.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `rg -n "Mordell|Selmer|Tate module|Weil pairing|GaloisRepresentation" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

### NT-T48 Add Finite-Field Elliptic Curves And L-Function Statement Surfaces

- Status: Completed
- Depends on: `NT-T45`, `NT-T67`
- Areas: `Proofs/Ai/EllipticCurve/FiniteField/`, `Proofs/Ai/EllipticCurve/LFunction/`
- Tasks:
  - Add finite-field point-count, Hasse theorem, and Weil bound interfaces.
  - Add elliptic-curve `L`-function, Hasse-Weil `L`-function, modularity,
    Gross-Zagier, Kolyvagin, Sato-Tate, and BSD statement surfaces.
  - Label BSD as conjectural or conditional.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.FiniteField`.
  - Elliptic-curve `L`-function statement interfaces.
- Acceptance criteria:
  - Finite-field core laws are imported from `Proofs.Ai.Algebra.AbstractFiniteField`.
  - BSD is not marked as a derived theorem.
  - Modularity links point to `NT-T52`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.FiniteField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.FiniteField`
  - `rg -n "Hasse|Weil bound|BSD|Gross|Zagier|Sato" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

### NT-T49 Add Modular Forms Basic And q-Expansion Interfaces

- Status: Completed
- Depends on: complex analysis, linear algebra, and algebra prerequisites
- Areas: `Proofs/Ai/ModularForms/Basic/`, `Proofs/Ai/ModularForms/QExpansion/`
- Tasks:
  - Add modular forms, cusp forms, weights, levels, and q-expansion interfaces.
  - Add Eisenstein series and q-expansion principle statement surfaces.
  - Keep modular forms independent of final-theorem glue.
- Deliverables:
  - `Proofs.Ai.ModularForms.Basic`.
  - `Proofs.Ai.ModularForms.QExpansion`.
- Acceptance criteria:
  - The module is reusable outside modularity-lifting wrappers.
  - Complex-analytic domain assumptions are explicit.
  - No Ribet or Wiles theorem is introduced here.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ModularForms.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Basic`
  - `rg -n "ModularForm|CuspForm|q_expansion|Eisenstein|level|weight" proofs/Proofs/Ai/ModularForms proofs/README.md`

### NT-T50 Add Hecke Operators, Eigenforms, And Modular Curves

- Status: Completed
- Depends on: `NT-T49`
- Areas: `Proofs/Ai/ModularForms/Hecke/`, `Proofs/Ai/ModularForms/ModularCurve/`
- Tasks:
  - Add Hecke operator, eigenform, coefficient-field, and Fourier-coefficient
    multiplicativity interfaces.
  - Add Petersson inner product and trace formula surfaces.
  - Add modular curve, Jacobian, and Eichler-Shimura interfaces.
- Deliverables:
  - `Proofs.Ai.ModularForms.Hecke`.
  - `Proofs.Ai.ModularForms.ModularCurve`.
- Acceptance criteria:
  - Hecke facts are general APIs, not hidden Wiles assumptions.
  - Modular-curve construction evidence is explicit.
  - Trace formula interfaces name analytic/geometric prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ModularForms.Hecke`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Hecke`
  - `rg -n "Hecke|Eigenform|Petersson|ModularCurve|Eichler" proofs/Proofs/Ai/ModularForms proofs/README.md`

### NT-T51 Add Ribet And Level-Lowering Interfaces

- Status: Completed
- Depends on: `NT-T50`, `NT-T63`, `NT-T47`
- Areas: `Proofs/Ai/Modularity/Ribet/`, `Proofs/Ai/Modularity/LevelLowering/`
- Tasks:
  - State Ribet level lowering in a reusable interface form.
  - Add conductor, irreducibility, ramification, newform, and excluded-case
    dependency map.
  - Keep any early bridge axiom under explicit bridge-marked namespace.
- Deliverables:
  - `Proofs.Ai.Modularity.Ribet`.
  - `Proofs.Ai.Modularity.LevelLowering` interface or replacement plan.
- Acceptance criteria:
  - The module cannot be confused with a completed Ribet proof while
    bridge-backed.
  - General level-lowering terminology is reusable for non-Frey representations.
  - High-trust downstream routes cannot import bridge-backed variants.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Ribet`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Ribet`
  - `rg -n "Ribet|level_lowering|BridgeAxiom|conductor|newform" proofs/Proofs/Ai/Modularity proofs/README.md`

### NT-T52 Add Modularity Lifting And Semistable Modularity Interfaces

- Status: Pending
- Depends on: `NT-T50`, `NT-T51`, `NT-T63`, `NT-T46`
- Areas: `Proofs/Ai/Modularity/Lifting/`, `Proofs/Ai/Modularity/Semistable/`
- Tasks:
  - Add deformation functor, deformation ring, Hecke/deformation comparison,
    and `R = T` theorem surfaces.
  - Add minimal and non-minimal modularity lifting interfaces.
  - Add semistable modularity theorem route with reusable assumptions.
- Deliverables:
  - `Proofs.Ai.Modularity.Lifting`.
  - `Proofs.Ai.Modularity.Semistable`.
- Acceptance criteria:
  - Remaining deep assumptions are named and machine-visible.
  - The final semistable modularity version has no unapproved bridge axiom
    dependency.
  - Lifting wrappers are useful outside the Frey curve case.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Modularity.Semistable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Semistable`
  - `rg -n "modularity_lifting|R_eq_T|semistable_modularity|BridgeAxiom" proofs/Proofs/Ai/Modularity proofs/README.md`

### NT-T53 Add General L-Function Interfaces

- Status: Pending
- Depends on: `NT-T31`, `NT-T43`, `NT-T50`
- Areas: `Proofs/Ai/NumberTheory/LFunction/`, `Proofs/Ai/NumberTheory/ArtinL/`, `Proofs/Ai/NumberTheory/HeckeL/`
- Tasks:
  - Add Artin, Hecke, Hasse-Weil, and automorphic `L`-function definitions.
  - Add Euler product, local factor, coefficient-field, and normalization
    interfaces.
  - Link Artin reciprocity connections to class field theory.
- Deliverables:
  - `Proofs.Ai.NumberTheory.LFunction`.
  - `Proofs.Ai.NumberTheory.ArtinL`.
  - `Proofs.Ai.NumberTheory.HeckeL`.
- Acceptance criteria:
  - Each `L`-function names coefficient field, local factors, analytic domain,
    and normalization.
  - Analytic continuation and functional equation remain separate theorem
    surfaces.
  - No conjectural statement is marked `L2`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.LFunction`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.LFunction`
  - `rg -n "LFunction|ArtinL|HeckeL|local factor|Euler product" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T54 Add Automorphic L-Functions And Trace Formula Surfaces

- Status: Pending
- Depends on: `NT-T53`, `NT-T52`
- Areas: `Proofs/Ai/NumberTheory/AutomorphicL/`, `Proofs/Ai/Langlands/TraceFormula/`
- Tasks:
  - Add automorphic `L`-function, Rankin-Selberg, Langlands-Shahidi, and
    converse theorem interfaces.
  - Add trace formula, Arthur-Selberg trace formula, endoscopic transfer, and
    fundamental lemma surfaces.
  - Keep broad analytic assumptions explicit.
- Deliverables:
  - `Proofs.Ai.NumberTheory.AutomorphicL`.
  - Trace formula interface map.
- Acceptance criteria:
  - Trace formula assumptions are not hidden behind a generic `Langlands`
    theorem.
  - Analytic continuation remains `L1` unless certified.
  - Ngo-style fundamental lemma references are statement surfaces until
    dependencies exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.AutomorphicL`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.AutomorphicL`
  - `rg -n "AutomorphicL|TraceFormula|Rankin|Shahidi|Fundamental lemma" proofs/Proofs/Ai proofs/README.md`

### NT-T55 Add Langlands Correspondence Statement Graph

- Status: Pending
- Depends on: `NT-T54`, `NT-T63`
- Areas: `Proofs/Ai/Langlands/Interface/`
- Tasks:
  - Add local and global Langlands correspondence statement surfaces.
  - Add Jacquet-Langlands, base change, functoriality, Sato-Tate, and
    potential automorphy interfaces.
  - Mark conjectural functoriality and broad correspondence statements as
    `L0` or conditional assumptions.
- Deliverables:
  - `Proofs.Ai.Langlands.Interface`.
  - Langlands dependency graph.
- Acceptance criteria:
  - The broad Langlands program is an interface graph, not a claimed proof.
  - Proven subtheorems can be promoted individually without importing the whole
    graph.
  - Conjectural statements are not exported as derived certificates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Langlands.Interface`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Langlands.Interface`
  - `rg -n "Langlands|functoriality|Jacquet|base change|potential automorphy" proofs/Proofs/Ai proofs/README.md`

### NT-T56 Add Arithmetic Geometry Curve And Rational-Point Interfaces

- Status: Pending
- Depends on: `NT-T39`, `NT-T48`, existing `Proofs.Ai.AlgebraicGeometry.*`
- Areas: `Proofs/Ai/ArithmeticGeometry/RationalPoints/`
- Tasks:
  - Add genus, divisor, Riemann-Roch, Hasse-Weil bound, and zeta function
    interfaces for curves.
  - Add Mordell/Faltings and Siegel integral-points theorem surfaces.
  - Keep rational-point statements separated from etale cohomology
    construction interfaces.
- Deliverables:
  - `Proofs.Ai.ArithmeticGeometry.RationalPoints`.
- Acceptance criteria:
  - Faltings-level results are `L1` until massive dependencies exist.
  - Finite-field zeta functions reuse finite-field core ownership.
  - Rational and integral point hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ArithmeticGeometry.RationalPoints`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.RationalPoints`
  - `rg -n "RationalPoints|RiemannRoch|Faltings|Siegel|HasseWeil" proofs/Proofs/Ai proofs/README.md`

### NT-T57 Add Schemes, Etale Cohomology, And Weil Conjectures Interfaces

- Status: Pending
- Depends on: `NT-T56`, existing algebraic-geometry modules
- Areas: `Proofs/Ai/ArithmeticGeometry/Schemes/`, `Proofs/Ai/ArithmeticGeometry/EtaleCohomology/`, `Proofs/Ai/ArithmeticGeometry/WeilConjectures/`
- Tasks:
  - Add scheme, fiber product, Zariski topology, flatness, base change, Kummer
    exact sequence, and etale cohomology finiteness interfaces.
  - Add Grothendieck trace formula, Lefschetz trace formula, Weil conjectures,
    and Deligne theorem surfaces.
  - Record scheme/cohomology dependencies explicitly.
- Deliverables:
  - `Proofs.Ai.ArithmeticGeometry.Schemes`.
  - `Proofs.Ai.ArithmeticGeometry.EtaleCohomology`.
  - `Proofs.Ai.ArithmeticGeometry.WeilConjectures`.
- Acceptance criteria:
  - Weil conjectures are interface-level until cohomology foundations exist.
  - Deligne theorem is not a generic finite-field axiom.
  - Etale cohomology assumptions are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ArithmeticGeometry.EtaleCohomology`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.EtaleCohomology`
  - `rg -n "Etale|WeilConjectures|Deligne|trace formula|base change" proofs/Proofs/Ai proofs/README.md`

### NT-T58 Add p-adic Hodge And Special-Point Interfaces

- Status: Pending
- Depends on: `NT-T57`, `NT-T42`, `NT-T63`
- Areas: `Proofs/Ai/ArithmeticGeometry/PadicHodge/`, `Proofs/Ai/ArithmeticGeometry/SpecialPoints/`
- Tasks:
  - Add Neron models, Neron-Ogg-Shafarevich, Chabauty-Coleman, l-adic
    representation, and p-adic Hodge comparison interfaces.
  - Add Manin-Mumford, Mordell-Lang, Bogomolov, and Andre-Oort statement
    surfaces.
  - State period-ring and representation hypotheses explicitly.
- Deliverables:
  - p-adic Hodge comparison interface package.
  - Special-points statement map.
- Acceptance criteria:
  - p-adic Hodge comparison is not used before period-ring assumptions exist.
  - Special-point conjectural or theorem status is labeled.
  - Arithmetic geometry modules reuse Galois representation APIs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ArithmeticGeometry.PadicHodge`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.PadicHodge`
  - `rg -n "PadicHodge|Neron|Chabauty|MordellLang|Andre" proofs/Proofs/Ai proofs/README.md`

### NT-T59 Add Iwasawa Algebra And Module Structure Interfaces

- Status: Pending
- Depends on: `NT-T41`, `NT-T44`, module theory prerequisites
- Areas: `Proofs/Ai/NumberTheory/Iwasawa/Basic/`
- Tasks:
  - Add cyclotomic `Z_p` extension and Iwasawa algebra interfaces.
  - Add finitely generated torsion module structure theorem surface.
  - Add lambda, mu, and nu invariants and Iwasawa class-number formula
    interface.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Iwasawa.Basic`.
- Acceptance criteria:
  - Module-theoretic assumptions over the Iwasawa algebra are explicit.
  - p-adic and Galois dependencies point to previous milestones.
  - The class-number formula is not confused with analytic class-number
    formula in `NT-T39`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Iwasawa.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Iwasawa.Basic`
  - `rg -n "Iwasawa|lambda|mu|nu|torsion module" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T60 Add Iwasawa Main Conjecture And Euler-System Interfaces

- Status: Pending
- Depends on: `NT-T59`, `NT-T47`, `NT-T53`
- Areas: `Proofs/Ai/NumberTheory/Iwasawa/MainConjecture/`, `Proofs/Ai/NumberTheory/Iwasawa/EulerSystem/`
- Tasks:
  - Add Kubota-Leopoldt p-adic `L`-function and interpolation formula
    surfaces.
  - Add Iwasawa main conjecture, Mazur-Wiles, Ferrero-Washington, and
    `mu = 0` theorem surfaces.
  - Add Euler systems, Kato, Rubin, Coates-Wiles, Skinner-Urban, plus/minus
    Selmer groups, and Gross-Koblitz interfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Iwasawa.MainConjecture`.
  - `Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`.
- Acceptance criteria:
  - Main conjectures are theorem interfaces or conditional forms with exact
    assumptions.
  - Selmer definitions are shared with elliptic-curve modules.
  - p-adic `L`-function interfaces reuse `NT-T42`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`
  - `rg -n "EulerSystem|MainConjecture|Kubota|Selmer|GrossKoblitz" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T61 Add Frobenius And Prime-Ideal Decomposition Interfaces

- Status: Pending
- Depends on: `NT-T38`, Galois group prerequisites
- Areas: `Proofs/Ai/NumberTheory/Frobenius/`
- Tasks:
  - Add prime ideal decomposition, decomposition group, inertia group, and
    Frobenius element interfaces.
  - Add Frobenius conjugacy-class theorem surface.
  - Share ramification vocabulary with local-field and Galois representation
    tasks.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Frobenius`.
- Acceptance criteria:
  - Frobenius definitions state unramified and prime-ideal hypotheses.
  - Chebotarev is not imported into this definition layer.
  - Decomposition and inertia terms are reusable by local conditions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Frobenius`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Frobenius`
  - `rg -n "Frobenius|decomposition group|inertia|prime ideal" proofs/Proofs/Ai proofs/README.md`

### NT-T62 Add Chebotarev Density Interfaces

- Status: Pending
- Depends on: `NT-T61`, analytic density prerequisites
- Areas: `Proofs/Ai/NumberTheory/Chebotarev/`
- Tasks:
  - Add Chebotarev density theorem interface.
  - Add Frobenius density theorem and Dirichlet theorem from Chebotarev
    interface.
  - Record that Chebotarev is not used by elementary prime infinitude or FTA.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Chebotarev`.
- Acceptance criteria:
  - Density measure and analytic assumptions are explicit.
  - Elementary prime facts remain independent.
  - The Chebotarev-to-Dirichlet route is an alias or later theorem card, not a
    duplicate proof of elementary Dirichlet `L` milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Chebotarev`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Chebotarev`
  - `rg -n "Chebotarev|Frobenius density|Dirichlet theorem|prime infinitude" proofs/Proofs/Ai proofs/README.md`

### NT-T63 Add Galois Representation Local Conditions

- Status: Pending
- Depends on: `NT-T61`, `NT-T47`
- Areas: `Proofs/Ai/GaloisRepresentation/Basic/`, `Proofs/Ai/GaloisRepresentation/Ramification/`, `Proofs/Ai/GaloisRepresentation/LocalCondition/`
- Tasks:
  - Add Galois representation, l-adic representation, cyclotomic character,
    and Tate module representation interfaces.
  - Add ramification and local-condition theorem surfaces aligned with
    modularity and arithmetic-geometry prerequisites.
  - Add Hodge-Tate, de Rham, crystalline, semistable, Fontaine-Laffaille, and
    comparison theorem surfaces.
- Deliverables:
  - `Proofs.Ai.GaloisRepresentation.Basic`.
  - `Proofs.Ai.GaloisRepresentation.Ramification`.
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`.
- Acceptance criteria:
  - Local conditions are reusable outside specialized final-theorem routes.
  - Elliptic-curve and modular-form representation interfaces can share this
    API.
  - Taylor-Wiles and potential modularity statements remain interfaces until
    modularity prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `rg -n "GaloisRepresentation|LocalCondition|ramification|crystalline|Taylor" proofs/Proofs/Ai proofs/README.md`

### NT-T64 Add Algorithmic Number Theory Correctness Foundations

- Status: Pending
- Depends on: `NT-T12`, `NT-T14`
- Areas: `Proofs/Ai/NumberTheory/Algorithm/`
- Tasks:
  - Add Euclid and extended Euclid algorithm correctness and complexity
    interfaces.
  - Add constructive CRT correctness and repeated-squaring correctness.
  - Name the implemented function or abstract algorithm relation being
    verified.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Algorithm`.
- Acceptance criteria:
  - Algorithm correctness is separated from mathematical existence.
  - Complexity statements are interfaces unless cost-model APIs exist.
  - No external solver or runtime oracle is trusted.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Algorithm`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Algorithm`
  - `rg -n "Algorithm|extended Euclid|constructive CRT|repeated squaring|complexity" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T65 Add Primality And Factoring Algorithm Interfaces

- Status: Pending
- Depends on: `NT-T15`, `NT-T64`
- Areas: `Proofs/Ai/NumberTheory/PrimalityTest/`, `Proofs/Ai/NumberTheory/FactoringAlgorithm/`
- Tasks:
  - Add Fermat, Miller-Rabin, and AKS primality-test theorem surfaces.
  - Add Pollard rho, quadratic sieve, and number field sieve theory
    interfaces.
  - Separate correctness, failure probability, and complexity assumptions.
- Deliverables:
  - `Proofs.Ai.NumberTheory.PrimalityTest`.
  - Factoring algorithm statement surfaces.
- Acceptance criteria:
  - Probabilistic claims state randomness assumptions.
  - Security/hardness assumptions are never derived theorem exports.
  - Algorithm modules do not mutate trusted kernel behavior.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.PrimalityTest`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimalityTest`
  - `rg -n "Miller|AKS|Pollard|sieve|probability|hardness" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T66 Add Cryptographic Correctness Interfaces

- Status: Pending
- Depends on: `NT-T15`, `NT-T47`, `NT-T64`
- Areas: `Proofs/Ai/Cryptography/NumberTheory/`, `Proofs/Ai/Cryptography/EllipticCurve/`
- Tasks:
  - Add Diffie-Hellman, discrete logarithm, elliptic-curve cryptography,
    ECDSA correctness, Weil/Tate pairing, LLL, and Coppersmith theorem
    surfaces.
  - Reuse RSA correctness from `NT-T15`.
  - Keep cryptographic hardness assumptions as assumptions, not theorems.
- Deliverables:
  - `Proofs.Ai.Cryptography.NumberTheory`.
  - `Proofs.Ai.Cryptography.EllipticCurve`.
- Acceptance criteria:
  - Correctness theorems name group, randomness, and key-generation
    assumptions.
  - Hardness assumptions are not marked as `L2`.
  - Pairing facts reuse elliptic-curve APIs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Cryptography.NumberTheory`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Cryptography.NumberTheory`
  - `rg -n "Diffie|ECDSA|pairing|LLL|Coppersmith|hardness" proofs/Proofs/Ai proofs/README.md`

### NT-T67 Import Or Alias Finite-Field Core

- Status: Pending
- Depends on: field-theory `FT-13`, `NT-T18`
- Areas: `Proofs/Ai/Algebra/AbstractFiniteField/`, `Proofs/Ai/NumberTheory/FiniteFieldApplications/`
- Tasks:
  - Import or alias `Proofs.Ai.Algebra.AbstractFiniteField` finite-field law,
    Frobenius, cardinality, and root-characterization facts.
  - Record finite-field existence, uniqueness, multiplicative-cyclicity, and
    subfield classification theorem cards with ownership checked against the
    field-theory route.
  - Add finite-field application namespace only for number-theoretic uses.
- Deliverables:
  - `Proofs.Ai.NumberTheory.FiniteFieldApplications`.
  - Finite-field ownership notes.
- Acceptance criteria:
  - The finite-field core is not redefined under `Proofs.Ai.NumberTheory`.
  - Primitive-root and Gauss-sum tasks import finite-field facts explicitly.
  - Ownership agrees with `develop/proof-corpus-field-theory-roadmap-todo.md`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFiniteField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFiniteField`
  - `rg -n "AbstractFiniteField|FiniteFieldApplications|Frobenius|ownership" proofs develop`

### NT-T68 Add Finite-Field Exponential Sum Interfaces

- Status: Pending
- Depends on: `NT-T67`, `NT-T20`
- Areas: `Proofs/Ai/NumberTheory/ExponentialSum/`
- Tasks:
  - Add finite-field Gauss sums, Jacobi sums, Hasse-Davenport, and
    Stickelberger interfaces.
  - Add Chevalley-Warning, Ax-Katz, Weil exponential-sum estimates, and
    Lang-Weil interfaces.
  - Reuse finite-field core and character APIs.
- Deliverables:
  - `Proofs.Ai.NumberTheory.ExponentialSum`.
- Acceptance criteria:
  - Field-size, degree, character, and nonvanishing hypotheses are explicit.
  - Weil estimates and Lang-Weil remain `L1` until algebraic-geometry
    prerequisites exist.
  - No finite-field core laws are duplicated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ExponentialSum`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ExponentialSum`
  - `rg -n "ExponentialSum|Hasse|Davenport|Chevalley|AxKatz|LangWeil" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T69 Add Polynomial Method And Combinatorial Interfaces

- Status: Pending
- Depends on: `NT-T67`, `NT-T30`
- Areas: `Proofs/Ai/NumberTheory/Combinatorial/`
- Tasks:
  - Add pigeonhole, Ramsey, van der Waerden, Schur, Rado,
    Erdos-Ginzburg-Ziv, Olson, Davenport constant, polynomial method, and
    combinatorial Nullstellensatz surfaces.
  - Connect finite-field polynomial-method results to finite-field aliases.
  - Keep additive-combinatorics ownership consistent with `NT-T30`.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Combinatorial`.
- Acceptance criteria:
  - Polynomial-method results name degree, field-size, and nonvanishing
    hypotheses.
  - Ambient structures are explicit.
  - Finite-field dependencies import `NT-T67`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Combinatorial`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Combinatorial`
  - `rg -n "Combinatorial|Nullstellensatz|Ramsey|Davenport constant|polynomial method" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T70 Package And Promote Stable Number-Theory Closure Batches

- Status: Pending
- Depends on: stable local closure for a selected contiguous batch
- Areas: `Proofs/Ai/NumberTheory/`, selected related domain namespaces, `proofs/generated/`, `npa-mathlib` closure sidecars
- Tasks:
  - Choose a coherent closure batch, such as elementary arithmetic through
    CRT, arithmetic functions through Mobius inversion, or finite-field
    applications after field-theory closure.
  - Run local authoring checks, package checks, axiom reports, and core-feature
    reports.
  - Verify bridge axioms and conjectural assumptions are absent from derived
    theorem exports before promotion.
- Deliverables:
  - Closure manifest.
  - Axiom and core-feature report.
  - Public theorem list for promoted number-theory modules.
- Acceptance criteria:
  - Every promoted public theorem has a source-free certificate verdict.
  - Package metadata, import hashes, and certificate hashes are deterministic.
  - Bridge interfaces remain in development namespaces or are removed from the
    promoted closure.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`
  - `rg -n "BridgeAxiom|conjecture|axiom report|NumberTheory" proofs/generated proofs/README.md develop`

## Review Findings

This task document was reviewed against:

- `proofs/number-theory-theorem-proof-roadmap.md`
- `develop/proof-corpus-field-theory-roadmap-todo.md`
- `AGENTS.md`

| Finding | Status | Resolution |
| --- | --- | --- |
| The finite-field core could be duplicated under number theory even though field theory owns `Proofs.Ai.Algebra.AbstractFiniteField`. | Fixed | `NT-T67` imports or aliases the field-theory finite-field core, and `NT-T68`/`NT-T69` only add applications. |
| The initial recommended queue placed finite-field elliptic curves before finite-field ownership and Ribet/modularity before Galois local conditions. | Fixed | Queue groups now place `NT-T67` before `NT-T48`, and `NT-T61` through `NT-T63` before `NT-T51` and `NT-T52`. |
| Analytic and conjectural theorem families could be mistaken for derived certificate targets. | Fixed | Target-level defaults and milestone acceptance criteria keep conjectures at `L0` or conditional `L1` until dependencies are certified. |

## Validation

Document-only validation for this task breakdown:

```sh
git diff --check -- proofs/number-theory-theorem-proof-roadmap-todo.md
rg -n "NT-T[0-9][0-9]" proofs/number-theory-theorem-proof-roadmap-todo.md
```

Also search the task document for unresolved local markers, stale owner names,
and accidental references to a duplicated finite-field core before committing.
