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
theory, and closure-boundary planning.

The list intentionally does not prove the number-theory roadmap in one pass.
Later agents should implement exactly one milestone or a clearly bounded
contiguous batch. When prerequisites are absent, agents should split explicit blocker or prerequisite tasks before source edits. Statement-only interfaces are not acceptable proof artifacts for pending theorem work.

Out of scope for this task document:

- changing the Rust kernel, certificate format, or independent checker;
- adding natural numbers, integers, rationals, residue rings, number fields,
  local fields, elliptic curves, modular forms, Galois representations, or
  cryptographic assumptions as trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, theorem-search runtime,
  or AI calls to trusted code;
- treating source files, replay files, metadata, generated indexes, theorem
  search, roadmap documents, or this task document as proof evidence;
- proving unresolved conjectures or adding them as proof-corpus theorem, source,
  certificate, metadata, replay, or theorem-index declarations;
- publicly materializing bridge-backed number-theory modules into `npa-mathlib` before
  local closure, axiom-report, and package verification checks are clean.

## Authoring Loop

For ordinary number-theory theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Replace `Proofs.Ai.NumberTheory.X` with the
actual owning namespace for elliptic-curve, modular-form, Galois
representation, algebraic-geometry, or field-theory milestones. Reserve
`check-corpus-package.sh` or `check-corpus-full.sh` for package-wide verifier
behavior, publish-plan or package metadata updates, certificate/checker
compatibility, release work, or high-trust closure work.

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
| `NT-16` elliptic curves | `NT-T45` through `NT-T48`; L2 upgrade backlog `NT-T71` through `NT-T78` |
| `NT-17` modular forms and modularity | `NT-T49` through `NT-T52` |
| `NT-18` L-functions and Langlands interfaces | `NT-T53` through `NT-T55` |
| `NT-19` arithmetic geometry | `NT-T56` through `NT-T58` |
| `NT-20` Iwasawa theory | `NT-T59` through `NT-T60` |
| `NT-21` Galois representations and density | `NT-T61` through `NT-T63` |
| `NT-22` computational number theory and cryptography | `NT-T64` through `NT-T66` |
| `NT-23` finite fields and combinatorial number theory | `NT-T67` through `NT-T69` |
| `NT-25` Fermat's Last Theorem route | `NT-T80` through `NT-T83` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `NT-T00` | `L0` theorem-card inventory, duplicate-home map, and conjecture labels |
| `NT-T01` through `NT-T12` | `L2` derived certificates where `Nat`/`Int` and quotient prerequisites exist; split missing integer APIs before source edits |
| `NT-T13` through `NT-T24` | `L2` derived certificates after finite group, residue ring, and divisor-sum APIs are stable; keep algorithmic traces as explicit evidence or blockers |
| `NT-T25` through `NT-T30` | `L2` for rational continued fractions and small Diophantine classifications; split advanced approximation and additive-combinatorics prerequisites |
| `NT-T31` through `NT-T36` | `L2` for algebraic Euler-product identities where prerequisites exist; analytic inputs remain blocker/dependency tasks until certified |
| `NT-T37` through `NT-T44` | `L2` for derived algebraic sublemmas where explicit law packages exist; split construction and reciprocity prerequisites before source edits |
| `NT-T45` through `NT-T63` | `L2` for bounded reusable lemmas in elliptic curves, modularity, Langlands, arithmetic geometry, Iwasawa, and density; theorem-heavy routes stay dependency maps until prerequisites exist |
| `NT-T64` through `NT-T69` | `L2` for algebraic correctness lemmas where functions exist; security and complexity assumptions remain explicit assumptions or non-`L2` blockers |
| `NT-T71` through `NT-T78` | `L2` upgrade tasks for every current `Proofs.Ai.EllipticCurve.*` theorem surface; conjectural or theorem-shaped boundary statements must either become derived certificates or remain explicitly non-`L2` |
| `NT-T80` through `NT-T83` | `L2` only for derived counterexample, Frey-route, modularity, level-lowering, and contradiction composition certificates; no FLT theorem/source/certificate is emitted until every named prerequisite has an L2 certificate |

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
| `NTQ-027` | `NT-T71`, `NT-T72` |
| `NTQ-028` | `NT-T73`, `NT-T74` |
| `NTQ-029` | `NT-T75`, `NT-T76` |
| `NTQ-030` | `NT-T77` |
| `NTQ-031` | `NT-T78` |

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
  - Label excluded conjecture families, conditional theorem forms, bridge
    interfaces, and derived theorem targets separately.
- Deliverables:
  - `proofs/number-theory-theorem-cards.md` number-theory theorem-card
    inventory.
  - Duplicate-home and excluded-conjecture / conditional-assumption map in
    `proofs/number-theory-theorem-cards.md`.
- Acceptance criteria:
  - Every roadmap theorem family has a card or an intentionally grouped card.
  - Conjectures are not added as proof modules, source files, certificates,
    theorem declarations, metadata, replay files, generated indexes, or package
    publication artifacts.
  - Sidecars, theorem search, AI output, metadata, and this task document are
    recorded as untrusted.
- Verification:
  - `rg -n "NT-00|NT-24|conjecture|Langlands|sidecar" proofs`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Inventory --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Divisibility --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - Descent/minimization dependency-map entry.
- Acceptance criteria:
  - Quotient-remainder uniqueness does not assume gcd or prime factorization.
  - Descent interfaces are general enough for Diophantine milestones.
  - Algorithm extraction is separated from mathematical existence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.EuclideanDivision`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Descent`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.EuclideanDivision --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Descent --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Gcd --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Lcm --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.EuclideanAlgorithm --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Bezout --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.LinearDiophantine --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Prime --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Composite --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.UfdBridge --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Factorization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Factorization --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimeInfinitude --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Congruence --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ResidueRing --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ModularGroup`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ModularGroup --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ChineseRemainder --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Phi --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatEulerWilson --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Carmichael`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Carmichael --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimalityTest --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Rsa`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Rsa --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimitiveRoot --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimitiveRoot --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Character --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Legendre --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.QuadraticReciprocity --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Jacobi --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ArithmeticFunction --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletConvolution --verified-cache authoring`
  - `rg -n "DirichletConvolution|divisor sum|identity|inverse" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T24 Prove Mobius Inversion And Euler Product Interface

- Status: Completed
- Depends on: `NT-T23`
- Areas: `Proofs/Ai/NumberTheory/Mobius/`, `Proofs/Ai/NumberTheory/EulerProduct/`
- Tasks:
  - Prove Mobius inversion and generalized Mobius inversion.
  - Add Euler product dependency-map entry for multiplicative Dirichlet series.
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Mobius --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ContinuedFraction --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Pell --verified-cache authoring`
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
  - Advanced theorems are dependency-map entries until analytic and measure
    prerequisites are certified.
  - Metric, measure, and real-field assumptions are named.
  - Transcendence results are not used by elementary number theory.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.DiophantineApproximation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DiophantineApproximation --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Diophantine --verified-cache authoring`
  - `rg -n "Pythagorean|SumsOfSquares|two_square|Coprime" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T29 Add Three-Square, Four-Square, Waring, And Coin Interfaces

- Status: Completed
- Depends on: `NT-T28`
- Areas: `Proofs/Ai/NumberTheory/SumsOfSquares/`, `Proofs/Ai/NumberTheory/Waring/`
- Tasks:
  - Add Lagrange four-square theorem route.
  - Add Legendre three-square, Waring, Hilbert-Waring, and Frobenius coin
    problem interfaces.
  - Record construction-heavy classical theorems as blockers until
    prerequisites are certified.
- Deliverables:
  - Sums-of-squares theorem package.
  - `Proofs.Ai.NumberTheory.Waring` dependency-map entry.
- Acceptance criteria:
  - Each theorem states positivity and representability hypotheses explicitly.
  - Theorems are not imported into downstream routes as hidden assumptions.
  - Coin problem theorem states gcd and nonnegative-combination assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.SumsOfSquares`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.SumsOfSquares --verified-cache authoring`
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
  - Advanced additive theorems stay as dependency-map entries until prerequisites are present.
  - Ambient structure and density assumptions are never implicit.
  - The module does not duplicate finite-field core laws.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Additive`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Additive --verified-cache authoring`
  - `rg -n "Cauchy|Davenport|Kneser|Szemeredi|Green|Tao" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T31 Add Dirichlet Series And Algebraic Euler Product Layer

- Status: Completed
- Depends on: `NT-T24`, analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/DirichletSeries/`, `Proofs/Ai/NumberTheory/EulerProduct/`
- Tasks:
  - Define Dirichlet series and abscissa dependency-map entries.
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletSeries --verified-cache authoring`
  - `rg -n "DirichletSeries|EulerProduct|abscissa|convergence" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T32 Add Riemann Zeta Function Interfaces

- Status: Completed
- Depends on: `NT-T31`, complex-analysis prerequisites
- Areas: `Proofs/Ai/NumberTheory/Zeta/`
- Tasks:
  - Add Riemann zeta definition and half-plane Euler product interface.
  - Add analytic continuation and functional equation dependency-map entries.
  - Add zero, explicit formula, and Riemann-von Mangoldt theorem surfaces.
- Deliverables:
  - `Proofs.Ai.NumberTheory.Zeta`.
- Acceptance criteria:
  - Analytic continuation remains blocker work until complex analysis is
    certified.
  - Riemann hypothesis remains a conjectural statement or conditional
    assumption, not a theorem target.
  - The zeta Euler product imports `NT-T31`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Zeta`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Zeta --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimeNumberTheorem --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DirichletL --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Sieve --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.CircleMethod --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.AlgebraicInteger --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.DedekindDomain --verified-cache authoring`
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
  - Analytic class-number formula stays as a dependency-map entry until analytic prerequisites
    are certified.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ClassGroup`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ClassGroup --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Valuation --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Hensel --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PadicAnalysis --verified-cache authoring`
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
  - Bridge assumptions are rejected by final public-package gates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ClassField.Local`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ClassField.Local --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisCohomology.Basic --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Semistable --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GaloisRepresentation --verified-cache authoring`
  - `rg -n "Mordell|Selmer|Tate module|Weil pairing|GaloisRepresentation" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

### NT-T48 Add Finite-Field Elliptic Curves And L-Function Statement Surfaces

- Status: Completed
- Depends on: `NT-T45`, `NT-T67`
- Areas: `Proofs/Ai/EllipticCurve/FiniteField/`, `Proofs/Ai/EllipticCurve/LFunction/`
- Tasks:
  - Add finite-field point-count, Hasse theorem, and Weil bound interfaces.
  - Add elliptic-curve `L`-function, Hasse-Weil `L`-function, modularity,
    Gross-Zagier, Kolyvagin, and Sato-Tate theorem surfaces.
  - Exclude unresolved conjectural claims from proof-corpus declarations.
- Deliverables:
  - `Proofs.Ai.EllipticCurve.FiniteField`.
  - Elliptic-curve `L`-function dependency-map entries.
- Acceptance criteria:
  - Finite-field core laws are imported from `Proofs.Ai.Algebra.AbstractFiniteField`.
  - No unresolved conjecture is emitted as a source, certificate, theorem,
    metadata, replay, or generated-index declaration.
  - Modularity links point to `NT-T52`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.FiniteField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.FiniteField --verified-cache authoring`
  - `rg -n "Hasse|Weil bound|Gross|Zagier|Sato|conjectur" proofs/Proofs/Ai/EllipticCurve proofs/README.md`

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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Basic --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Hecke --verified-cache authoring`
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Ribet --verified-cache authoring`
  - `rg -n "Ribet|level_lowering|BridgeAxiom|conductor|newform" proofs/Proofs/Ai/Modularity proofs/README.md`

### NT-T52 Add Modularity Lifting And Semistable Modularity Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Semistable --verified-cache authoring`
  - `rg -n "modularity_lifting|R_eq_T|semistable_modularity|BridgeAxiom" proofs/Proofs/Ai/Modularity proofs/README.md`

### NT-T53 Add General L-Function Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.LFunction --verified-cache authoring`
  - `rg -n "LFunction|ArtinL|HeckeL|local factor|Euler product" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T54 Add Automorphic L-Functions And Trace Formula Surfaces

- Status: Completed
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
  - Analytic continuation remains a blocker unless certified.
  - Ngo-style fundamental lemma references are statement surfaces until
    dependencies exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.AutomorphicL`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.AutomorphicL --verified-cache authoring`
  - `rg -n "AutomorphicL|TraceFormula|Rankin|Shahidi|Fundamental lemma" proofs/Proofs/Ai proofs/README.md`

### NT-T55 Add Langlands Correspondence Statement Graph

- Status: Completed
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
  - Proven subtheorems can be materialized individually without importing the whole
    graph.
  - Conjectural statements are not exported as derived certificates.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Langlands.Interface`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Langlands.Interface --verified-cache authoring`
  - `rg -n "Langlands|functoriality|Jacquet|base change|potential automorphy" proofs/Proofs/Ai proofs/README.md`

### NT-T56 Add Arithmetic Geometry Curve And Rational-Point Interfaces

- Status: Completed
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
  - Faltings-level results remain blocker work until massive dependencies
    exist.
  - Finite-field zeta functions reuse finite-field core ownership.
  - Rational and integral point hypotheses are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ArithmeticGeometry.RationalPoints`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.RationalPoints --verified-cache authoring`
  - `rg -n "RationalPoints|RiemannRoch|Faltings|Siegel|HasseWeil" proofs/Proofs/Ai proofs/README.md`

### NT-T57 Add Schemes, Etale Cohomology, And Weil Conjectures Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.EtaleCohomology --verified-cache authoring`
  - `rg -n "Etale|WeilConjectures|Deligne|trace formula|base change" proofs/Proofs/Ai proofs/README.md`

### NT-T58 Add p-adic Hodge And Special-Point Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.PadicHodge --verified-cache authoring`
  - `rg -n "PadicHodge|Neron|Chabauty|MordellLang|Andre" proofs/Proofs/Ai proofs/README.md`

### NT-T59 Add Iwasawa Algebra And Module Structure Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Iwasawa.Basic --verified-cache authoring`
  - `rg -n "Iwasawa|lambda|mu|nu|torsion module" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T60 Add Iwasawa Main Conjecture And Euler-System Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Iwasawa.EulerSystem --verified-cache authoring`
  - `rg -n "EulerSystem|MainConjecture|Kubota|Selmer|GrossKoblitz" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T61 Add Frobenius And Prime-Ideal Decomposition Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Frobenius --verified-cache authoring`
  - `rg -n "Frobenius|decomposition group|inertia|prime ideal" proofs/Proofs/Ai proofs/README.md`

### NT-T62 Add Chebotarev Density Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Chebotarev --verified-cache authoring`
  - `rg -n "Chebotarev|Frobenius density|Dirichlet theorem|prime infinitude" proofs/Proofs/Ai proofs/README.md`

### NT-T63 Add Galois Representation Local Conditions

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.GaloisRepresentation.LocalCondition --verified-cache authoring`
  - `rg -n "GaloisRepresentation|LocalCondition|ramification|crystalline|Taylor" proofs/Proofs/Ai proofs/README.md`

### NT-T64 Add Algorithmic Number Theory Correctness Foundations

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Algorithm --verified-cache authoring`
  - `rg -n "Algorithm|extended Euclid|constructive CRT|repeated squaring|complexity" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T65 Add Primality And Factoring Algorithm Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.PrimalityTest --verified-cache authoring`
  - `rg -n "Miller|AKS|Pollard|sieve|probability|hardness" proofs/Proofs/Ai/NumberTheory proofs/README.md`

### NT-T66 Add Cryptographic Correctness Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Cryptography.NumberTheory --verified-cache authoring`
  - `rg -n "Diffie|ECDSA|pairing|LLL|Coppersmith|hardness" proofs/Proofs/Ai proofs/README.md`

### NT-T67 Import Or Alias Finite-Field Core

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFiniteField --verified-cache authoring`
  - `rg -n "AbstractFiniteField|FiniteFieldApplications|Frobenius|ownership" proofs develop`
- Completion notes:
  - Added `Proofs.Ai.NumberTheory.FiniteFieldApplications` as a number-theoretic application
    namespace that imports `Proofs.Ai.Algebra.AbstractFiniteField` rather than redefining the
    finite-field core.
  - Recorded ownership theorem cards for field-theory-owned existence, uniqueness,
    multiplicative-cyclicity, subfield-classification, Frobenius, primitive-root, and Gauss-sum
    application routes.
  - Updated `PrimitiveRoot` and `GaussSum` to import the finite-field application facts explicitly.

### NT-T68 Add Finite-Field Exponential Sum Interfaces

- Status: Completed
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
  - Weil estimates and Lang-Weil stay as dependency-map entries until algebraic-geometry
    prerequisites exist.
  - No finite-field core laws are duplicated.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.ExponentialSum`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.ExponentialSum --verified-cache authoring`
  - `rg -n "ExponentialSum|Hasse|Davenport|Chevalley|AxKatz|LangWeil" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Completion notes:
  - Added `Proofs.Ai.NumberTheory.ExponentialSum` downstream of
    `FiniteFieldApplications`, `Character`, and `GaussSum`.
  - Recorded finite-field Gauss and Jacobi sum, Hasse-Davenport, Stickelberger,
    Chevalley-Warning, Ax-Katz, Weil blocker, and Lang-Weil blocker theorem
    cards.
  - Kept field-size, degree, character, and nonvanishing hypotheses explicit and did not
    duplicate finite-field core laws.

### NT-T69 Add Polynomial Method And Combinatorial Interfaces

- Status: Completed
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
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.Combinatorial --verified-cache authoring`
  - `rg -n "Combinatorial|Nullstellensatz|Ramsey|Davenport constant|polynomial method" proofs/Proofs/Ai/NumberTheory proofs/README.md`
- Completion notes:
  - Added `Proofs.Ai.NumberTheory.Combinatorial` downstream of
    `FiniteFieldApplications` and `Additive`.
  - Recorded pigeonhole/Ramsey/Schur/Rado, van der Waerden polynomial-method,
    Erdos-Ginzburg-Ziv/Olson/Davenport constant, finite-field combinatorial
    Nullstellensatz, and finite-field ownership boundary theorem cards.
  - Kept ambient structures, field-size, degree, and nonvanishing hypotheses
    explicit while reusing the NT-T67 finite-field application route.

### NT-T71 L2 Elliptic Curve Basic Definitions And Nonsingularity

- Status: Completed (2026-06-07)
- Depends on: `NT-T45`, stable field, polynomial, equality, and nonzero APIs
- Areas: `Proofs/Ai/EllipticCurve/Basic/`, `Proofs/Ai/Algebra/`
- Tasks:
  - Replaced statement-only Weierstrass and nonsingularity interfaces with
    structured model data, discriminant expressions, and certificate-derived
    nonsingularity evidence.
  - Removed theorem-shaped `curve_law`, `nonsingular_law`, and
    `assumption_law` premises from Basic theorem targets.
  - Kept route-independence evidence machine-visible without treating roadmap
    text or import absence as trusted proof evidence.
- Theorem coverage:
  - `ec_square`
  - `ec_cube`
  - `short_weierstrass_rhs`
  - `short_weierstrass_discriminant`
  - `ShortWeierstrassEquation`
  - `NonsingularShortWeierstrass`
  - `WeierstrassRouteIndependenceData`
  - `WeierstrassModelData`
  - `short_weierstrass_equation_rhs_refl`
  - `short_weierstrass_discriminant_defeq`
  - `weierstrass_route_independence_data_intro`
  - `weierstrass_model_data_intro`
  - `weierstrass_model_field_laws`
  - `weierstrass_model_nonsingular_from_discriminant`
  - `weierstrass_model_route_independence`
  - `weierstrass_model_reusable_outside_specialized_routes`
- Deliverables:
  - `Proofs.Ai.EllipticCurve.Basic` with L2-derived certificates for
    Weierstrass model assumptions and nonsingularity facts.
  - Direct Basic importers now import the explicit ring and field law packages
    required by Basic's exported model data surface.
- Acceptance criteria:
  - Each Basic theorem either has a source-free L2 certificate derived from
    explicit field/polynomial assumptions or is renamed/demoted so it is not
    counted as an L2 theorem.
  - No target theorem accepts a premise whose type is exactly the target
    conclusion under a different name.
  - The trusted base is not expanded; no field, polynomial, or elliptic-curve
    fact is added as a kernel primitive.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "curve_law|nonsingular_law|assumption_law|_interface|_surface|_boundary" proofs/Proofs/Ai/EllipticCurve/Basic/source.npa`
  - `./scripts/check-corpus-authoring.sh`

### NT-T72 L2 Elliptic Curve Point Group Law

- Status: Completed (2026-06-08)
- Depends on: `NT-T71`, stable rational-expression and equality-reasoning APIs
- Areas: `Proofs/Ai/EllipticCurve/GroupLaw/`, `Proofs/Ai/EllipticCurve/Basic/`
- Tasks:
  - Replaced statement-only point-group interfaces with structured
    short-Weierstrass point, point-at-infinity, doubling, exceptional-pair, and
    point-group data definitions.
  - Derived closure, identity, inverse, associativity, nonsingularity, and
    reusable-route projections from Basic model data and `GroupLawArgs`.
  - Removed modularity/Ribet/bridge boundary surfaces from theorem targets.
- Theorem coverage:
  - `PointAtInfinity`
  - `point_double`
  - `PointAdditionExceptionalCase`
  - `PointOnShortWeierstrass`
  - `EllipticPointGroupData`
  - `point_at_infinity_zero_refl`
  - `point_double_defeq`
  - `point_addition_exceptional_case_inverse_refl`
  - `point_on_short_weierstrass_from_coordinates`
  - `elliptic_point_group_data_intro`
  - `elliptic_point_group_model_data`
  - `elliptic_point_group_structure`
  - `elliptic_point_group_zero_on_curve`
  - `elliptic_point_group_add_closed`
  - `elliptic_point_group_neg_closed`
  - `elliptic_point_group_double_on_curve`
  - `elliptic_point_group_add_assoc`
  - `elliptic_point_group_left_identity`
  - `elliptic_point_group_right_identity`
  - `elliptic_point_group_left_inverse`
  - `elliptic_point_group_right_inverse`
  - `elliptic_point_group_nonsingular_model`
  - `elliptic_point_group_reusable_outside_specialized_routes`
- Deliverables:
  - `Proofs.Ai.EllipticCurve.GroupLaw` with an L2-derived point group-law
    certificate.
  - Reusable point-operation lemmas that downstream finite-field, height, and
    Galois-representation modules can import.
- Acceptance criteria:
  - Group-law evidence is derived from explicit point-operation definitions and
    Basic hypotheses, not supplied as a theorem-shaped `group_law` premise.
  - Exceptional cases are explicit and source-free checked.
  - No modularity, Ribet, Frey, or bridge-axiom module is imported by the L2
    group-law closure.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.EllipticCurve.GroupLaw Proofs.Ai.EllipticCurve.FiniteField Proofs.Ai.EllipticCurve.LFunction Proofs.Ai.EllipticCurve.GaloisRepresentation Proofs.Ai.Cryptography.EllipticCurve Proofs.Ai.NumberTheory.Frobenius Proofs.Ai.NumberTheory.Chebotarev Proofs.Ai.GaloisRepresentation.Basic Proofs.Ai.GaloisRepresentation.Ramification Proofs.Ai.GaloisRepresentation.LocalCondition Proofs.Ai.EllipticCurve.MordellWeil Proofs.Ai.NumberTheory.Iwasawa.EulerSystem Proofs.Ai.ArithmeticGeometry.RationalPoints Proofs.Ai.ArithmeticGeometry.Schemes Proofs.Ai.ArithmeticGeometry.EtaleCohomology Proofs.Ai.ArithmeticGeometry.WeilConjectures Proofs.Ai.ArithmeticGeometry.PadicHodge Proofs.Ai.ArithmeticGeometry.SpecialPoints`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GroupLaw --verified-cache authoring`
  - `rg -n "group_law|Closure|Associativity|Identity|Inverse|BridgeAxiom|Ribet|Modularity" proofs/Proofs/Ai/EllipticCurve/GroupLaw proofs/Proofs/Ai/EllipticCurve/Basic`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### NT-T73 L2 Elliptic Curve Reduction And Semistability

- Status: Completed (2026-06-08)
- Depends on: `NT-T71`, `NT-T72`, `NT-T40` through `NT-T42`
- Areas: `Proofs/Ai/EllipticCurve/Reduction/`, `Proofs/Ai/EllipticCurve/Semistable/`, local-field modules
- Tasks:
  - Completed structured local-field valuation input and elliptic reduction
    data definitions carrying Weierstrass model evidence, conductor,
    reduction type, minimal model, valuation, and compatibility predicates.
  - Completed semistability data as a general reduction/local-field predicate
    with a separate not-Frey-specific witness, independent of
    modularity-lifting assumptions.
  - Removed theorem-shaped reduction and semistability interface/boundary laws
    from the L2 targets.
- Theorem coverage:
  - `LocalFieldValuationInput`
  - `EllipticReductionData`
  - `local_field_valuation_input_intro`
  - `local_field_valuation_input_local_field`
  - `local_field_valuation_input_valuation`
  - `elliptic_reduction_data_intro`
  - `elliptic_reduction_local_valuation_input`
  - `elliptic_reduction_local_field_dependency`
  - `elliptic_reduction_valuation_dependency`
  - `elliptic_reduction_conductor`
  - `elliptic_reduction_type`
  - `elliptic_minimal_model`
  - `elliptic_reduction_compatibility`
  - `EllipticSemistabilityData`
  - `elliptic_semistability_data_intro`
  - `elliptic_semistability_reduction_data`
  - `elliptic_semistable_reduction_type`
  - `semistability_general_elliptic_curve_predicate`
  - `semistability_not_frey_specific`
  - `semistability_reduction_compatibility`
- Deliverables:
  - L2 certificates for reduction/minimal-model compatibility and
    semistability predicates where local-field prerequisites are available.
  - Downstream import/certificate refresh for `Proofs.Ai.Modularity.Semistable`,
    `Proofs.Ai.NumberTheory.AutomorphicL`, and `Proofs.Ai.Langlands.Interface`
    after Reduction/Semistable export hashes changed.
- Acceptance criteria:
  - Satisfied: local-field, valuation, conductor, and reduction dependencies
    are represented as explicit data/predicate fields and imported from owning
    modules.
  - Satisfied: semistability is represented as a general reduction predicate
    with a separate not-Frey-specific projection and no modularity-lifting
    dependency.
  - Satisfied: the previous interface/boundary theorem names are absent from
    generated proof artifacts and the AI theorem index.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.EllipticCurve.Reduction Proofs.Ai.EllipticCurve.Semistable Proofs.Ai.Modularity.Semistable Proofs.Ai.NumberTheory.AutomorphicL Proofs.Ai.Langlands.Interface`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Reduction --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Semistable --verified-cache authoring`
  - `rg -n "elliptic_conductor_interface|elliptic_reduction_type_interface|elliptic_minimal_model_interface|reduction_local_field_valuation_dependency_surface|conductor_reduction_minimal_model_compatibility_surface|elliptic_semistable_interface|semistability_general_elliptic_curve_predicate_boundary|semistability_not_frey_specific_boundary" proofs/Proofs/Ai/EllipticCurve/Reduction proofs/Proofs/Ai/EllipticCurve/Semistable tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### NT-T74 L2 Elliptic Curve Height And Neron-Tate Height

- Status: Completed (2026-06-08)
- Depends on: `NT-T71`, `NT-T72`, `NT-T73`, analysis/ordered-field prerequisites
- Areas: `Proofs/Ai/EllipticCurve/Height/`
- Tasks:
  - Satisfied: defined height field context, elliptic height data, and
    Neron-Tate height data as explicit structured prerequisites.
  - Satisfied: derived field-law, ordered-law, positivity, finiteness,
    functoriality, and pairing projections from those data records.
  - Satisfied: removed the previous theorem-shaped height and positivity
    interface/surface targets from source and generated artifacts.
- Theorem coverage:
  - `height_field_context_intro`
  - `height_field_context_field_laws`
  - `height_field_context_ordered_laws`
  - `height_analysis_prerequisites_explicit`
  - `elliptic_height_data_intro`
  - `elliptic_height_point_group_data`
  - `elliptic_height_field_context`
  - `elliptic_height_nonnegative`
  - `elliptic_height_finiteness`
  - `elliptic_height_functorial`
  - `neron_tate_height_data_intro`
  - `neron_tate_height_elliptic_height_data`
  - `neron_tate_height_nonnegative`
  - `neron_tate_height_functorial`
  - `neron_tate_height_pairing_bilinear`
  - `neron_tate_height_pairing_compatible`
  - `neron_tate_height_pairing_diagonal`
- Deliverables:
  - Satisfied: `Proofs.Ai.EllipticCurve.Height` L2-derived height certificates.
  - Satisfied: ordered-field and group-law prerequisites are imported from
    their owning modules and exposed through explicit data fields.
- Acceptance criteria:
  - Satisfied: height statements do not hide field, positivity, finiteness, or
    pairing assumptions.
  - Satisfied: construction-heavy prerequisites are data fields rather than
    theorem-shaped L2 targets.
  - Satisfied: downstream Mordell-Weil and Galois-representation tasks can
    import height facts without importing theorem-shaped height assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.Height`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Height --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.EllipticCurve.Height Proofs.Ai.EllipticCurve.GaloisRepresentation Proofs.Ai.Cryptography.EllipticCurve Proofs.Ai.NumberTheory.Frobenius Proofs.Ai.GaloisRepresentation.Basic Proofs.Ai.GaloisRepresentation.Ramification Proofs.Ai.GaloisRepresentation.LocalCondition Proofs.Ai.EllipticCurve.MordellWeil Proofs.Ai.NumberTheory.Iwasawa.EulerSystem Proofs.Ai.ArithmeticGeometry.PadicHodge Proofs.Ai.ArithmeticGeometry.SpecialPoints`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "elliptic_height_interface|elliptic_neron_tate_height_interface|height_field_and_positivity_hypotheses_surface|neron_tate_height_field_positivity_pairing_surface" proofs/Proofs/Ai/EllipticCurve/Height tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`
  - `./scripts/check-corpus-authoring.sh`

### NT-T75 L2 Tate Module, Weil Pairing, And Elliptic-Curve Galois Representation APIs

- Status: Completed (2026-06-08)
- Depends on: `NT-T72`, `NT-T74`, `NT-T61` through `NT-T63`
- Areas: `Proofs/Ai/EllipticCurve/GaloisRepresentation/`, `Proofs/Ai/GaloisRepresentation/`
- Tasks:
  - Satisfied: defined torsion inverse systems, Tate-module Galois actions,
    Weil pairing data, and local-condition bridges with explicit coefficient
    and local-condition dependencies.
  - Satisfied: derived Weil-pairing bilinearity and nondegeneracy projections
    from algebraic data rather than cryptographic assumptions.
  - Satisfied: connected Selmer-sharing and local-condition vocabulary to the
    general Galois-representation modules through imported prerequisites.
- Theorem coverage:
  - `torsion_inverse_system_data_intro`
  - `torsion_inverse_system_height_prerequisites`
  - `torsion_inverse_system_transition_compatible`
  - `tate_module_galois_action_data_intro`
  - `tate_module_torsion_inverse_system`
  - `tate_module_projection_compatible`
  - `tate_module_galois_action_compatible`
  - `tate_module_ladic_coefficient_dependency`
  - `weil_pairing_data_intro`
  - `weil_pairing_tate_module_data`
  - `weil_pairing_bilinear_from_data`
  - `weil_pairing_nondegenerate_from_data`
  - `weil_pairing_algebraic_dependency`
  - `galois_local_condition_bridge_data_intro`
  - `galois_local_condition_weil_pairing_data`
  - `galois_representation_local_condition`
  - `selmer_definition_shared_with_iwasawa_and_galois`
- Deliverables:
  - Satisfied: L2-derived Tate module and Weil pairing certificates with
    algebraic, height, and Galois prerequisites imported from owning modules.
  - Satisfied: previous sharing-boundary policy statements were replaced by
    explicit bridge data and derived projections.
- Acceptance criteria:
  - Satisfied: no cryptographic hardness or protocol-correctness assumption
    appears in the Weil-pairing proof closure.
  - Satisfied: local-condition and representation APIs are imported from their
    owning namespaces and do not duplicate definitions.
  - Satisfied: the module can be source-free verified with its dependency
    cache.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GaloisRepresentation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.EllipticCurve.Height Proofs.Ai.EllipticCurve.GaloisRepresentation Proofs.Ai.Cryptography.EllipticCurve Proofs.Ai.NumberTheory.Frobenius Proofs.Ai.GaloisRepresentation.Basic Proofs.Ai.GaloisRepresentation.Ramification Proofs.Ai.GaloisRepresentation.LocalCondition Proofs.Ai.EllipticCurve.MordellWeil Proofs.Ai.NumberTheory.Iwasawa.EulerSystem Proofs.Ai.ArithmeticGeometry.PadicHodge Proofs.Ai.ArithmeticGeometry.SpecialPoints`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "tate_module_interface|weil_pairing_interface|weil_pairing_nondegeneracy_without_crypto_boundary|selmer_definition_shared_iwasawa_galois_representation_surface|galois_representation_local_condition_surface|crypto|hardness|_interface|_surface|_boundary" proofs/Proofs/Ai/EllipticCurve/GaloisRepresentation/source.npa`
  - `./scripts/check-corpus-authoring.sh`

### NT-T76 L2 Torsion, Nagell-Lutz, Mordell-Weil, Selmer, And Tate-Shafarevich Surfaces

- Status: Completed (2026-06-08)
- Depends on: `NT-T72`, `NT-T74`, `NT-T75`, descent and cohomology prerequisites
- Areas: `Proofs/Ai/EllipticCurve/MordellWeil/`
- Tasks:
  - Satisfied: derived torsion and Nagell-Lutz conclusions from explicit
    height, integral-point, torsion, and group-law data.
  - Satisfied: weak Mordell-Weil and Mordell-Weil projections require explicit
    height/torsion data plus `DescentPrerequisites`.
  - Satisfied: Selmer and Tate-Shafarevich statements are represented through a
    cohomology/status data package rather than theorem-shaped axioms.
- Theorem coverage:
  - `torsion_nagell_lutz_data_intro`
  - `torsion_nagell_lutz_height_data`
  - `bounded_torsion_from_height_group_data`
  - `nagell_lutz_conclusion_from_integral_torsion`
  - `mordell_weil_descent_data_intro`
  - `mordell_weil_descent_height_torsion_data`
  - `mordell_weil_descent_prerequisites`
  - `weak_mordell_weil_finite_quotient_from_descent`
  - `mordell_weil_finitely_generated_from_height_descent`
  - `selmer_sha_status_data_intro`
  - `selmer_status_mordell_weil_descent_data`
  - `selmer_cohomology_foundations_explicit`
  - `selmer_local_to_global_from_status_data`
  - `tate_shafarevich_local_to_global_from_status_data`
  - `selmer_status_explicit`
  - `tate_shafarevich_status_explicit`
- Deliverables:
  - Satisfied: L2 certificates for bounded torsion, Nagell-Lutz, weak
    Mordell-Weil, and Mordell-Weil projections whose prerequisites are present.
  - Satisfied: module-level Selmer/Tate-Shafarevich status split records
    cohomology foundations and explicit status evidence as data fields.
- Acceptance criteria:
  - Satisfied: Mordell-Weil projections require certificate-derived height data
    and explicit descent prerequisites.
  - Satisfied: Selmer and Tate-Shafarevich statements do not become
    theorem-shaped axioms.
  - Satisfied: every theorem in the replacement coverage list is L2-derived
    from explicit data, with status-only claims carried as status evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.MordellWeil`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.MordellWeil --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.MordellWeil --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.NumberTheory.Iwasawa.EulerSystem Proofs.Ai.ArithmeticGeometry.RationalPoints Proofs.Ai.ArithmeticGeometry.Schemes Proofs.Ai.ArithmeticGeometry.EtaleCohomology Proofs.Ai.ArithmeticGeometry.WeilConjectures Proofs.Ai.ArithmeticGeometry.PadicHodge Proofs.Ai.ArithmeticGeometry.SpecialPoints`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "elliptic_torsion_subgroup_interface|nagell_lutz_theorem_interface|weak_mordell_weil_interface|mordell_weil_theorem_interface|selmer_group_interface|tate_shafarevich_group_statement_surface|mordell_weil_interface_level_until_height_descent_boundary" proofs/Proofs/Ai/EllipticCurve/MordellWeil tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`
  - `rg -n "mordell_weil_law|weak_law|selmer_law|sha_law" proofs/Proofs/Ai/EllipticCurve/MordellWeil/source.npa`
  - `./scripts/check-corpus-authoring.sh`

### NT-T77 L2 Finite-Field Elliptic Curves, Point Counts, Hasse Theorem, And Weil Bound

- Status: Completed (2026-06-08)
- Depends on: `NT-T67`, `NT-T71`, `NT-T72`, finite-field closure from field-theory roadmap
- Areas: `Proofs/Ai/EllipticCurve/FiniteField/`, `Proofs/Ai/Algebra/AbstractFiniteField`
- Tasks:
  - Satisfied: `Proofs.Ai.EllipticCurve.FiniteField` imports the owning
    `Proofs.Ai.Algebra.AbstractFiniteField` closure and projects finite-field
    cardinality, characteristic, Frobenius, `q`-power, and root-polynomial
    evidence from that imported core.
  - Satisfied: point-count and Frobenius trace targets are structured through
    `EllipticFiniteFieldPointCountData`, tied to `EllipticPointGroupData` and
    `EllipticFiniteFieldCoreData`, rather than theorem-shaped law packages.
  - Satisfied: Hasse and Weil-bound projections require
    `EllipticFiniteFieldHasseWeilData`, including explicit Lang-Weil,
    algebraic-geometry, and dependency-certified fields.
- Theorem coverage:
  - `elliptic_finite_field_core_data_intro`
  - `elliptic_finite_field_point_group_data`
  - `elliptic_finite_field_cardinality_from_abstract_core`
  - `elliptic_finite_field_characteristic_prime_from_abstract_core`
  - `elliptic_finite_field_frobenius_hom_from_abstract_core`
  - `elliptic_finite_field_pow_card_eq_self_from_abstract_core`
  - `elliptic_finite_field_roots_card_polynomial_from_abstract_core`
  - `elliptic_finite_field_point_count_data_intro`
  - `elliptic_finite_field_point_count_core_data`
  - `elliptic_finite_field_point_count_certified`
  - `elliptic_finite_field_frobenius_trace_certified`
  - `elliptic_finite_field_trace_matches_point_count`
  - `elliptic_finite_field_hasse_weil_data_intro`
  - `elliptic_finite_field_hasse_point_count_data`
  - `elliptic_finite_field_lang_weil_prerequisites_explicit`
  - `elliptic_finite_field_algebraic_geometry_prerequisites_explicit`
  - `elliptic_finite_field_hasse_dependencies_certified`
  - `elliptic_finite_field_weil_dependencies_certified`
  - `hasse_theorem_from_finite_field_point_count_data`
  - `weil_bound_from_frobenius_trace_data`
- Deliverables:
  - Satisfied: `Proofs.Ai.EllipticCurve.FiniteField` contains L2-derived
    finite-field elliptic-curve certificates where finite-field and point-group
    prerequisites are available.
  - Satisfied: Hasse/Weil-bound certificates expose Lang-Weil and
    algebraic-geometry dependencies explicitly and do not assert unconditional
    deep results without certificate-derived dependencies.
- Acceptance criteria:
  - Satisfied: no finite-field core theorem is duplicated in the elliptic-curve
    namespace; the finite-field core is imported from `AbstractFiniteField`.
  - Satisfied: point-count and Frobenius trace are structured data fields with
    specific certified values, not abstract `*_law` interface premises.
  - Satisfied: Hasse theorem and Weil bound are projected only from
    dependency-certified Hasse-Weil data.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.FiniteField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.FiniteField --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.FiniteField --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.LFunction`
  - `cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.NumberTheory.Iwasawa.EulerSystem Proofs.Ai.ArithmeticGeometry.RationalPoints Proofs.Ai.ArithmeticGeometry.Schemes Proofs.Ai.ArithmeticGeometry.EtaleCohomology Proofs.Ai.ArithmeticGeometry.WeilConjectures Proofs.Ai.ArithmeticGeometry.PadicHodge Proofs.Ai.ArithmeticGeometry.SpecialPoints`
  - `rg -n "finite_field_core_laws_imported_from_abstract_finite_field_boundary|finite_field_point_count_interface|hasse_theorem_interface|weil_bound_interface|finite_field_frobenius_trace_surface" proofs/Proofs/Ai/EllipticCurve/FiniteField tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`
  - `rg -n "_interface|_surface|_boundary" proofs/Proofs/Ai/EllipticCurve/FiniteField/source.npa`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `./scripts/check-corpus-authoring.sh`

### NT-T78 L2 Elliptic-Curve L-Functions And Certified Deep Theorem Prerequisites

- Status: Completed (2026-06-08)
- Depends on: `NT-T52`, `NT-T53` through `NT-T55`, `NT-T77`, analytic and modularity prerequisites
- Areas: `Proofs/Ai/EllipticCurve/LFunction/`, `Proofs/Ai/NumberTheory/LFunction/`, `Proofs/Ai/Modularity/`
- Completed tasks:
  - Defined `EllipticLFunctionData` by importing the general
    `Proofs.Ai.NumberTheory.LFunction` framework and carrying NT-T77
    finite-field local-factor data as explicit prerequisites.
  - Replaced the old modularity, Gross-Zagier, Kolyvagin, and Sato-Tate
    pass-through theorem names with `EllipticDeepTheoremStatusData` plus
    derived projection certificates whose analytic, modularity, Galois,
    Heegner, Euler-system, and equidistribution prerequisites are explicit.
  - Kept unresolved open-problem claims out of proof-corpus theorem/source/
    certificate/meta/replay/index declarations and represented any
    conditional status only through named prerequisite evidence.
- Theorem coverage:
  - `EllipticLFunctionData`
  - `EllipticDeepTheoremStatusData`
  - `elliptic_l_function_data_intro`
  - `elliptic_l_function_general_l_function_data`
  - `elliptic_l_function_nt_t77_finite_field_data`
  - `elliptic_l_function_local_factor_data`
  - `elliptic_l_function_finite_field_local_factors`
  - `elliptic_l_function_matches_general`
  - `elliptic_l_function_certified`
  - `hasse_weil_l_function_from_elliptic_data`
  - `elliptic_l_function_l2_unresolved_claims_excluded`
  - `elliptic_deep_status_data_intro`
  - `elliptic_deep_status_l_function_package_certified`
  - `elliptic_deep_status_nt_t52_modularity_certified`
  - `elliptic_deep_status_analytic_prerequisites_certified`
  - `elliptic_deep_status_galois_prerequisites_certified`
  - `elliptic_deep_status_heegner_prerequisites_certified`
  - `elliptic_deep_status_euler_system_prerequisites_certified`
  - `elliptic_deep_status_equidistribution_prerequisites_certified`
  - `modularity_link_from_nt_t52_certificate`
  - `gross_zagier_from_certified_prerequisites`
  - `kolyvagin_from_certified_prerequisites`
  - `sato_tate_from_certified_prerequisites`
  - `elliptic_deep_status_named_prerequisites`
- Deliverables:
  - L2-derived certificates for elliptic and Hasse-Weil L-function data, backed
    by generated source, certificate, replay, metadata, and theorem-index
    entries.
  - A machine-visible status split that keeps deep theorem public-package work behind
    named certified prerequisites instead of exporting open-problem statements.
- Acceptance criteria:
  - No unresolved open-problem statement is exported or declared as a
    proof-corpus theorem, source, certificate, metadata, replay, or
    generated-index entry.
  - Modularity links point to certified `NT-T52` evidence through
    `modularity_link_from_nt_t52_certificate` rather than generic bridge
    assumptions.
  - The old coverage names are removed, and every replacement theorem is either
    L2-derived or gated by named certified prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.EllipticCurve.LFunction`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.LFunction --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "elliptic_curve_l_function_interface|hasse_weil_l_function_interface|modularity_link_points_to_nt_t52_boundary|gross_zagier_statement_surface|kolyvagin_statement_surface|sato_tate_statement_surface" proofs/Proofs/Ai/EllipticCurve/LFunction tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`
  - `rg -n "conjectur|conditional|_interface|_surface|_boundary" proofs/Proofs/Ai/EllipticCurve/LFunction/source.npa`
  - `./scripts/check-corpus-authoring.sh`

### NT-T80 Add L2 Fermat Last Theorem Counterexample And Route Data

- Status: In progress (2026-06-13)
- Depends on: `NT-T45` through `NT-T52`, `NT-T71` through `NT-T78`
- Areas: `Proofs/Ai/NumberTheory/FermatLastTheorem/`
- Tasks:
  - Add a primitive-counterexample data definition for the equation
    `x^n + y^n = z^n`, positivity/nonzero evidence, pairwise coprimality, and
    exponent-at-least-three evidence.
  - Add a Wiles/Ribet/Frey route data definition that keeps Frey-curve
    construction, Frey semistability, semistable modularity,
    no-bridge-axiom evidence, Ribet level lowering, level-two contradiction,
    and no-counterexample extraction as named prerequisites.
  - Prove only introduction/projection/composition certificates whose proof
    terms actually use the structured data. The NT-T80 data layer itself does
    not own the final no-solution theorem; the final-statement wrapper is
    tracked under `NT-T83`.
- Theorem coverage:
  - `FermatPrimitiveCounterexampleData`
  - `FermatWilesRibetRouteData`
  - `fermat_primitive_counterexample_data_intro`
  - `fermat_counterexample_positive_x`
  - `fermat_counterexample_positive_y`
  - `fermat_counterexample_positive_z`
  - `fermat_counterexample_nonzero_x`
  - `fermat_counterexample_nonzero_y`
  - `fermat_counterexample_nonzero_z`
  - `fermat_counterexample_pairwise_coprime`
  - `fermat_counterexample_exponent_at_least_three`
  - `fermat_counterexample_equation`
  - `fermat_wiles_ribet_route_data_intro`
  - `fermat_route_frey_semistability`
  - `fermat_route_semistable_modularity`
  - `fermat_route_no_bridge_axiom_dependency`
  - `fermat_route_ribet_level_lowering`
  - `fermat_route_level_two_contradiction`
  - `fermat_route_no_counterexample_law`
  - `fermat_no_counterexample_from_wiles_ribet_route`
- Acceptance criteria:
  - The NT-T80 layer is not treated as a completed proof of Fermat's Last
    Theorem by itself.
  - No theorem assumes the FLT conclusion itself.
  - The route composition names every deep prerequisite instead of importing a
    bridge-backed Ribet or Wiles assumption silently.
  - Source-free verification succeeds for
    `Proofs.Ai.NumberTheory.FermatLastTheorem`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.FermatLastTheorem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatLastTheorem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "fermat_last_theorem|FermatPrimitiveCounterexampleData|FermatWilesRibetRouteData|BridgeAxiom" proofs/Proofs/Ai/NumberTheory/FermatLastTheorem tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`

### NT-T81 Prove L2 Exponent Reduction And Primitive Counterexample Normalization

- Status: In progress (2026-06-13)
- Depends on: `NT-T80`, divisibility, gcd, prime-factorization, and descent
  L2 certificates
- Areas: `Proofs/Ai/NumberTheory/FermatLastTheorem/`, elementary
  number-theory modules
- Current L2 theorem coverage:
  - `FermatRawCounterexampleData`
  - `FermatExponentReductionData`
  - `FermatPrimitiveNormalizationData`
  - `fermat_raw_counterexample_data_intro`
  - `fermat_raw_counterexample_positive_x`
  - `fermat_raw_counterexample_positive_y`
  - `fermat_raw_counterexample_positive_z`
  - `fermat_raw_counterexample_nonzero_x`
  - `fermat_raw_counterexample_nonzero_y`
  - `fermat_raw_counterexample_nonzero_z`
  - `fermat_raw_counterexample_exponent_at_least_three`
  - `fermat_raw_counterexample_equation`
  - `fermat_exponent_reduction_data_intro`
  - `fermat_reduced_counterexample_from_exponent_reduction`
  - `fermat_primitive_normalization_data_intro`
  - `fermat_primitive_counterexample_from_normalization_data`
- Remaining theorem targets before final FLT source emission:
  - reduction from exponent `n >= 3` to a prime exponent or exponent `4`;
  - normalization from an arbitrary positive counterexample to a primitive
    counterexample;
  - preservation of the Fermat equation under exponent factoring;
  - descent/minimality certificate for primitive counterexample selection.
- Acceptance criteria:
  - The L2 composition theorems must build `FermatPrimitiveCounterexampleData`
    from named raw/reduction/normalization data, not assume the final FLT
    conclusion.
  - Arithmetic facts that are not yet available, such as prime-exponent
    extraction and gcd descent, remain named prerequisites rather than L1
    source artifacts.
  - Prime-factor and gcd prerequisites are imported from their owning modules.
  - Failure to prove a target is recorded here, not as an L1 source artifact.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.FermatLastTheorem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatLastTheorem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "FermatRawCounterexampleData|FermatExponentReductionData|FermatPrimitiveNormalizationData|fermat_reduced_counterexample_from_exponent_reduction|fermat_primitive_counterexample_from_normalization_data" proofs/Proofs/Ai/NumberTheory/FermatLastTheorem tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`

### NT-T82 Prove L2 Frey Curve Construction And Semistability From Counterexamples

- Status: In progress (2026-06-13)
- Depends on: `NT-T80`, `NT-T81`, `NT-T71` through `NT-T77`, local-field and
  Galois-representation L2 prerequisites
- Areas: `Proofs/Ai/NumberTheory/FermatLastTheorem/`,
  `Proofs/Ai/EllipticCurve/*`
- Current L2 theorem coverage:
  - `FermatFreyModelData`
  - `fermat_frey_model_data_intro`
  - `fermat_frey_model_builds_curve`
  - `fermat_frey_model_discriminant_control`
  - `fermat_frey_model_conductor_control`
  - `fermat_frey_model_minimal_model`
  - `fermat_frey_model_galois_representation`
  - `fermat_frey_semistability_from_model_data`
  - `fermat_no_counterexample_from_frey_model_route`
- Remaining theorem targets before final FLT source emission:
  - construction of the Frey curve from a primitive Fermat counterexample using
    certified elliptic-curve data, not an abstract Frey-model witness;
  - discriminant, conductor, and minimal-model facts for the Frey curve;
  - semistability of the Frey curve as a derived theorem, not a supplied law;
  - attachment of the relevant mod-`p` Galois representation.
- Acceptance criteria:
  - Frey-specific facts remain outside reusable elliptic-curve API modules.
  - Semistability is derived from reduction/minimal-model data, not assumed by
    the final route.
  - The route-composition theorem may consume `FermatFreyModelData`, but no
    theorem named as the final Fermat's Last Theorem is emitted here.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.FermatLastTheorem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatLastTheorem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "FermatFreyModelData|fermat_frey_model_|fermat_no_counterexample_from_frey_model_route" proofs/Proofs/Ai/NumberTheory/FermatLastTheorem tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`

### NT-T83 Prove L2 Ribet-Modularity Contradiction And Final FLT Theorem

- Status: In progress (2026-06-13)
- Depends on: `NT-T80` through `NT-T82`, completed L2 Ribet level lowering,
  semistable modularity/Taylor-Wiles route, and the level-two contradiction
- Areas: `Proofs/Ai/NumberTheory/FermatLastTheorem/`,
  `Proofs/Ai/Modularity/*`
- Current L2 theorem coverage:
  - `fermat_semistable_modular_from_frey_model_route`
  - `fermat_no_bridge_axiom_from_frey_model_route`
  - `fermat_ribet_lowering_from_frey_model_route`
  - `fermat_level_two_contradiction_from_frey_model_route`
  - `FermatPrimitiveFreyRouteData`
  - `fermat_primitive_frey_route_data_intro`
  - `fermat_primitive_frey_route_primitive_data`
  - `fermat_primitive_frey_route_realizes`
  - `fermat_primitive_frey_route_frey_data`
  - `fermat_primitive_frey_route_route_data`
  - `fermat_semistable_modular_from_primitive_frey_route_data`
  - `fermat_no_bridge_axiom_from_primitive_frey_route_data`
  - `fermat_ribet_lowering_from_primitive_frey_route_data`
  - `fermat_level_two_contradiction_from_primitive_frey_route_data`
  - `fermat_no_counterexample_from_primitive_frey_route_data`
  - `FermatRawPrimitiveFreyRouteData`
  - `fermat_raw_primitive_frey_route_data_intro`
  - `fermat_raw_primitive_frey_route_normalization_data`
  - `fermat_raw_primitive_frey_route_primitive_route_data`
  - `fermat_raw_primitive_frey_route_realizes_primitive`
  - `fermat_raw_primitive_frey_route_frey_data`
  - `fermat_raw_primitive_frey_route_route_data`
  - `fermat_semistable_modular_from_raw_primitive_frey_route_data`
  - `fermat_no_bridge_axiom_from_raw_primitive_frey_route_data`
  - `fermat_ribet_lowering_from_raw_primitive_frey_route_data`
  - `fermat_primitive_counterexample_from_raw_primitive_frey_route_data`
  - `fermat_level_two_contradiction_from_raw_primitive_frey_route_data`
  - `fermat_no_counterexample_from_raw_primitive_frey_route_data`
  - `fermat_no_raw_counterexample_from_raw_primitive_frey_route_data`
  - `fermat_not_raw_counterexample_from_raw_primitive_frey_route_data`
  - `fermat_raw_counterexample_false_from_raw_primitive_frey_route_data`
  - `fermat_positive_integer_solution_false_from_raw_primitive_frey_route_data`
  - `fermat_not_positive_integer_solution_from_raw_primitive_frey_route_data`
  - `FermatRawCounterexampleEliminationData`
  - `fermat_raw_counterexample_elimination_data_intro`
  - `fermat_raw_counterexample_elimination_route_data`
  - `fermat_raw_counterexample_elimination_realizes_raw`
  - `fermat_raw_counterexample_elimination_no_raw_law`
  - `fermat_no_counterexample_from_raw_elimination_data`
  - `fermat_no_raw_counterexample_from_raw_elimination_data`
  - `FermatPositiveSolutionData` is the concrete positive-solution evidence
    record for the surface FLT tuple fields.
  - `fermat_positive_solution_data_intro`
  - `fermat_positive_solution_positive_x`
  - `fermat_positive_solution_positive_y`
  - `fermat_positive_solution_positive_z`
  - `fermat_positive_solution_nonzero_x`
  - `fermat_positive_solution_nonzero_y`
  - `fermat_positive_solution_nonzero_z`
  - `fermat_positive_solution_exponent_at_least_three`
  - `fermat_positive_solution_equation`
  - `fermat_raw_counterexample_from_positive_solution_data` bridges the
    concrete positive-solution record to the raw-counterexample record.
  - `FermatPositiveIntegerSolutionData` is the concrete final-statement syntax
    record whose equation field is
    `EqualInt (Add (Pow x n) (Pow y n)) (Pow z n)`.
  - `fermat_positive_integer_solution_data_intro`
  - `fermat_positive_integer_solution_positive_x`
  - `fermat_positive_integer_solution_positive_y`
  - `fermat_positive_integer_solution_positive_z`
  - `fermat_positive_integer_solution_nonzero_x`
  - `fermat_positive_integer_solution_nonzero_y`
  - `fermat_positive_integer_solution_nonzero_z`
  - `fermat_positive_integer_solution_exponent_at_least_three`
  - `fermat_positive_integer_solution_equation`
  - `fermat_positive_solution_data_from_positive_integer_solution` bridges the
    final-statement syntax record to `FermatPositiveSolutionData` instantiated
    with the concrete equation formula.
  - `fermat_positive_integer_solution_false_from_positive_solution_data_negation`
    separates the concrete final-statement `False` contradiction obtained from
    `Not` of formula-instantiated `FermatPositiveSolutionData`.
  - `fermat_not_positive_integer_solution_from_positive_solution_data_negation`
    transports a `Not` proof for formula-instantiated
    `FermatPositiveSolutionData` to the final positive-integer solution
    record.
  - `fermat_positive_solution_contradiction_from_no_raw_not` derives the
    positive-solution contradiction from `Not` of the corresponding raw
    counterexample record.
  - `fermat_positive_integer_solution_contradiction_from_no_raw_not` applies
    that contradiction to the concrete positive-integer solution syntax.
  - `fermat_not_raw_counterexample_from_formula_raw_elimination_data`
    specializes raw elimination data with
    `NoRawFermatCounterexample := Not FermatRawCounterexampleData` and derives
    `Not` of the formula-instantiated raw counterexample.
  - `fermat_raw_counterexample_false_from_formula_raw_elimination_data`
    separates the formula-instantiated raw-counterexample `False`
    contradiction from that raw-elimination data before downstream
    positive-solution contradictions consume it.
  - `fermat_positive_integer_solution_false_from_raw_elimination_data`
    combines a concrete positive-integer solution with that local
    raw-elimination data to derive `False`.
  - `fermat_not_positive_integer_solution_from_raw_elimination_data` wraps a
    local solution-dependent raw-elimination provider into `Not` of the
    concrete positive-integer solution.
  - `fermat_positive_solution_false_from_positive_solution_elimination_provider`
    separates the local `False` contradiction from a positive-solution
    elimination provider by first deriving the no-raw-counterexample evidence
    and then applying the supplied positive-solution contradiction law.
  - `fermat_global_not_positive_integer_solution_from_solution_raw_elimination_provider`
    lifts a solution-dependent raw-elimination provider to the global concrete
    positive-integer no-solution theorem.
  - `FermatPositiveIntegerGlobalEliminationData` packages selector data,
    positive-integer-to-raw construction, and solution-indexed
    raw-elimination data into one concrete positive-integer closure.
  - `fermat_positive_integer_global_elimination_data_intro` builds that closure
    from its certified components.
  - `fermat_positive_integer_global_elimination_data_from_solution_raw_elimination_provider`
    constructs the closure using the certified positive-integer
    solution-to-raw bridge and a solution-indexed raw-elimination provider.
  - `fermat_positive_integer_global_elimination_data_from_global_elimination_data`
    extracts the raw-elimination provider from the formula-specialized
    `FermatGlobalEliminationData` closure and converts it to the concrete
    positive-integer closure using the certified solution-to-raw bridge.
  - `fermat_last_theorem_from_positive_integer_global_elimination_data` is the
    direct final-statement wrapper over the explicit
    `FermatPositiveIntegerGlobalEliminationData` closure.
  - `fermat_positive_integer_solution_false_from_global_elimination_data`
    derives an explicit `False` contradiction for the concrete
    positive-integer solution syntax from the formula-specialized
    `FermatGlobalEliminationData` closure by transporting through
    `FermatPositiveIntegerGlobalEliminationData`.
  - `fermat_last_theorem_from_global_elimination_data` is the final-statement
    wrapper over the formula-specialized `FermatGlobalEliminationData`
    closure.
  - `fermat_global_elimination_data_from_not_raw_provider` constructs that
    formula-specialized global closure from the raw-elimination provider plus
    the certified projection and contradiction theorems for
    `FermatPositiveSolutionData`.
  - `fermat_positive_integer_solution_false_from_raw_elimination_provider`
    derives the explicit concrete positive-integer solution contradiction
    directly from the formula-specialized raw-elimination provider by first
    building the `FermatGlobalEliminationData` closure.
  - `fermat_last_theorem_from_raw_elimination_provider` is the final-statement
    wrapper over the formula-specialized raw-elimination provider.
  - `fermat_raw_elimination_provider_from_raw_primitive_frey_route_provider`
    constructs the formula-specialized raw-elimination provider from a
    raw-primitive-Frey-route provider, a raw-realization provider, and the
    no-raw-counterexample translation law.
  - `fermat_positive_integer_solution_false_from_raw_primitive_frey_route_provider`
    derives the explicit concrete positive-integer solution contradiction
    from those raw-primitive-Frey-route, raw-realization, and no-raw inputs by
    first constructing the formula-specialized raw-elimination provider.
  - `fermat_last_theorem_from_raw_primitive_frey_route_provider` is the
    final-statement L2 wrapper over raw-primitive-Frey-route,
    raw-realization, and no-raw inputs.
  - `fermat_positive_integer_solution_false_from_solution_raw_primitive_frey_route_provider`
    derives the concrete positive-integer `False` contradiction when
    raw-primitive-Frey-route data is supplied only for the raw datum generated by
    the given positive-integer solution, together with the raw-realization
    provider and no-raw-counterexample law.
  - `fermat_global_not_positive_integer_solution_from_solution_raw_primitive_frey_route_provider`
    wraps that solution-indexed raw-primitive-route-data contradiction with
    `not_intro`, avoiding an assumption that route data exists for every raw
    counterexample.
  - `fermat_solution_raw_elimination_provider_from_solution_raw_primitive_frey_route_provider`
    constructs the solution-indexed raw-elimination provider from
    solution-indexed raw-primitive-Frey-route data, the raw-realization provider,
    and the no-raw-counterexample law.
  - `fermat_positive_integer_global_elimination_data_from_solution_raw_primitive_frey_route_provider`
    packages that constructed solution-indexed raw-elimination provider into
    `FermatPositiveIntegerGlobalEliminationData`.
  - `fermat_solution_raw_primitive_frey_route_provider_from_solution_normalization_and_primitive_route_provider`
    constructs solution-indexed raw-primitive-Frey-route data from
    solution-indexed primitive-normalization and primitive-Frey-route providers.
  - `fermat_positive_integer_global_elimination_data_from_solution_primitive_frey_route_provider`
    packages the resulting solution-indexed route provider into
    `FermatPositiveIntegerGlobalEliminationData` through the existing
    raw-elimination construction.
  - `fermat_solution_primitive_frey_route_provider_from_solution_normalization_frey_model_and_route_data`
    constructs solution-indexed primitive-Frey-route data from
    solution-indexed primitive-normalization, primitive-realization,
    Frey-model, and Wiles/Ribet route-data inputs.
  - `fermat_positive_integer_global_elimination_data_from_solution_frey_model_and_route_data`
    packages that solution-indexed primitive-Frey-route construction into
    `FermatPositiveIntegerGlobalEliminationData` without requiring a
    primitive-Frey-route provider for every raw counterexample.
  - `fermat_solution_frey_model_provider_from_solution_builds_curve_and_frey_model_laws`
    constructs solution-indexed Frey-model data from a solution-indexed
    builds-Frey-curve provider and the generic discriminant, conductor,
    minimal-model, and Galois-representation laws.
  - `fermat_positive_integer_global_elimination_data_from_solution_builds_curve_and_frey_model_laws_and_route_data`
    packages that constructed solution-indexed Frey-model provider with the
    solution-indexed primitive-normalization / primitive-realization providers
    and Wiles/Ribet route data into `FermatPositiveIntegerGlobalEliminationData`.
  - `fermat_solution_primitive_normalization_provider_from_solution_normalization_laws`
    constructs solution-indexed primitive-normalization data from
    solution-indexed positive, nonzero, coprime, exponent, and Fermat-equation
    primitive component providers.
  - `fermat_positive_integer_global_elimination_data_from_solution_normalization_laws_builds_curve_and_route_data`
    packages those solution-indexed primitive-normalization components together
    with solution-indexed primitive-realization, builds-curve, Frey-model laws,
    Wiles/Ribet route data, raw-realization, and no-raw-counterexample inputs.
  - `fermat_positive_integer_global_elimination_data_from_solution_normalization_laws_builds_curve_and_route_laws`
    constructs Wiles/Ribet route data from the Frey semistability, modularity,
    no-bridge, Ribet lowering, level-two contradiction, and no-counterexample
    laws before packaging the same solution-indexed global elimination closure.
  - `fermat_positive_integer_solution_false_from_solution_normalization_laws_builds_curve_and_route_laws`
    derives the explicit concrete positive-integer solution contradiction from
    that route-law-based closure.
  - `fermat_global_not_positive_integer_solution_from_solution_normalization_laws_builds_curve_and_route_laws`
    wraps the route-law-based contradiction as a globally quantified
    positive-integer `Not` theorem.
  - `fermat_positive_integer_global_elimination_data_from_global_normalization_laws_builds_curve_and_route_laws`
    adapts the raw-indexed public primitive/builds-curve provider surface into
    the solution-indexed route-law closure by specializing every raw provider to
    the canonical raw datum generated from a concrete positive-integer solution.
  - `fermat_positive_integer_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`
    derives the concrete contradiction from that raw-to-solution provider
    adapter and the resulting positive-integer global elimination data.
  - `fermat_global_not_positive_integer_solution_from_global_normalization_laws_builds_curve_and_route_laws`
    wraps the raw-indexed public provider adapter as a globally quantified
    positive-integer `Not` theorem.
  - `fermat_global_elimination_data_from_global_normalization_laws_builds_curve_and_route_laws`
    constructs the formula-specialized `FermatGlobalEliminationData` closure at
    the generic raw-indexed global-normalization boundary by building
    `FermatGlobalRawRefutationData` from raw-realization evidence and the
    no-raw-counterexample law.
  - `fermat_positive_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`
    derives a generic `FermatPositiveSolutionData` contradiction from that
    formula-specialized closure.
  - `fermat_global_not_positive_solution_from_global_normalization_laws_builds_curve_and_route_laws`
    wraps the generic formula-level contradiction as a globally quantified
    `Not` theorem.
  - `fermat_positive_arithmetic_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`
    transports the same generic closure through an explicit `Positive ->
    Nonzero` law and derives a positive-arithmetic contradiction.
  - `fermat_last_theorem_from_global_normalization_laws_builds_curve_and_route_laws`
    wraps that positive-arithmetic contradiction as the generic final-statement
    `Not` surface under decomposed global-normalization and route laws.
  - `fermat_selected_positive_arithmetic_contradiction_law_from_global_normalization_laws_builds_curve_and_route_laws`
    exposes the same closure as a selected arithmetic contradiction law for a
    concrete positive triple and exponent.
  - `fermat_positive_integer_global_elimination_data_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    derives the primitive `Nonzero` providers from positive primitive providers
    and the generic ordered-field bridge before constructing the same
    positive-integer global-elimination closure.
  - `fermat_positive_integer_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    and
    `fermat_global_not_positive_integer_solution_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    expose the resulting ordered-field-derived positive-integer contradiction
    and `Not` wrapper.
  - `fermat_global_elimination_data_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_global_not_positive_solution_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    expose the same ordered-field-derived closure at the formula-level
    `FermatPositiveSolutionData` surface.
  - `fermat_positive_arithmetic_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    derives both the primitive `Nonzero` providers and the required `Positive
    -> Nonzero` law from the generic ordered-field bridge before applying the
    same generic global-normalization positive-arithmetic contradiction.
  - `fermat_last_theorem_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    wraps that ordered-field-derived contradiction as the generic final
    positive-arithmetic `Not` surface.
  - `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    exposes the same ordered-field global-normalization closure as a selected
    arithmetic contradiction law.
  - `fermat_last_theorem_from_primitive_frey_route_provider` is the
    final-statement L2 wrapper over primitive-normalization,
    primitive-Frey-route, raw-realization, and no-raw inputs.
  - `fermat_positive_integer_solution_false_from_primitive_frey_route_provider`
    derives the explicit concrete positive-integer solution contradiction from
    the primitive-normalization and primitive-Frey-route provider pair by
    constructing the raw-primitive-Frey-route provider first.
  - `fermat_raw_primitive_frey_route_provider_from_normalization_and_primitive_route_provider`
    constructs the formula-specialized raw-primitive-Frey-route provider from
    explicit primitive-normalization and primitive-Frey-route providers.
  - `fermat_primitive_frey_route_provider_from_normalization_frey_model_and_route_data`
    constructs the formula-specialized primitive-Frey-route provider from
    primitive normalization, primitive-realization evidence, Frey-model data,
    and Wiles/Ribet route data.
  - `fermat_positive_integer_solution_false_from_frey_model_and_route_data`
    derives the explicit concrete positive-integer solution contradiction from
    primitive normalization, primitive-realization evidence, Frey-model data,
    and Wiles/Ribet route data by first constructing the primitive-Frey-route
    provider.
  - `fermat_frey_model_provider_from_frey_model_laws` constructs the
    Frey-model provider from explicit builds-curve, discriminant-control,
    conductor-control, minimal-model, and Galois-representation provider
    families.
  - `fermat_last_theorem_from_frey_model_and_route_data` is the
    final-statement L2 wrapper over primitive-normalization,
    primitive-realization, Frey-model provider, Wiles/Ribet route data,
    raw-realization, and no-raw inputs.
  - `fermat_positive_integer_solution_false_from_primitive_normalization_provider`
    derives the explicit concrete positive-integer solution contradiction from
    primitive normalization, primitive realization, Frey-model component
    providers, route laws, raw realization, and no-raw data by constructing
    the Frey-model provider and Wiles/Ribet route data first.
  - `fermat_last_theorem_from_primitive_normalization_provider` is the
    final-statement L2 wrapper over a primitive-normalization provider plus
    primitive-realization, Frey-model component, Wiles/Ribet route,
    raw-realization, and no-raw inputs.
  - `fermat_primitive_normalization_provider_from_normalization_laws`
    constructs the formula-specialized primitive-normalization provider from
    explicit primitive positivity, nonzero, pairwise-coprime, exponent, and
    Fermat equation provider families.
  - `fermat_route_semistability_law_from_frey_model_laws` derives the generic
    Frey semistability route law from discriminant, conductor, minimal-model
    laws and a semistability-from-model theorem.
  - `fermat_frey_model_provider_from_builds_curve_and_frey_model_laws`
    constructs the selected Frey-model provider from the selected builds-curve
    provider plus generic discriminant, conductor, minimal-model, and
    Galois-representation laws.
  - `fermat_positive_integer_solution_false_from_selected_frey_model_component_providers`
    derives the explicit concrete positive-integer solution contradiction from
    primitive-normalization component providers, primitive realization,
    selected Frey-model component providers, route laws, raw realization, and
    no-raw data by first constructing the primitive-normalization provider.
  - `fermat_last_theorem_from_selected_frey_model_component_providers` keeps
    the previous final-statement wrapper with selected Frey-model component
    providers as an intermediate theorem.
  - `fermat_positive_integer_solution_false_from_frey_model_laws`
    derives the explicit concrete positive-integer solution contradiction from
    primitive-normalization component providers, selected builds-curve data,
    generic Frey-model laws, direct route laws, raw realization, and no-raw
    data by constructing the primitive-normalization provider, selected
    Frey-model provider, and Wiles/Ribet route data first.
  - `fermat_last_theorem_from_frey_model_laws` is the final-statement L2
    wrapper over primitive-normalization component providers,
    primitive-realization, selected builds-curve, generic Frey-model laws,
    Wiles/Ribet route laws, raw-realization, and no-raw inputs.
  - Public `fermat_positive_integer_solution_false` and `fermat_last_theorem`
    are being moved from the direct Frey-model-law boundary to a bridge-free
    no-counterexample-data boundary. The next L2 wrappers should derive
    semistable-modularity/no-bridge laws from `SemistableModularityData`,
    Ribet lowering from bridge-free `LevelLoweringData`, level-two
    contradiction from `FermatLevelTwoObstructionData`, and
    no-counterexample extraction from `FermatNoCounterexampleData`, while
    still keeping the remaining raw-realization and no-raw-counterexample
    inputs explicit.
  - `fermat_route_modularity_law_from_semistable_modularity_data` is the next
    route-law decomposition target: it should derive the Frey route
    semistable-modularity law from imported `SemistableModularityData`, a
    selected local field and Galois representation for each semistable Frey
    curve, and a modularity-lifting conclusion provider for that selected
    representation.
  - `fermat_route_no_bridge_law_from_semistable_modularity_data` is the next
    bridge-cleanliness decomposition target: it should derive the Frey route
    no-bridge-axiom law from the same imported semistable-modularity data,
    specialized so the route conclusion is the Frey `SemistableModular`
    predicate.
  - `fermat_last_theorem_from_semistable_modularity_data` is the next
    final-statement L2 wrapper: it should replace direct
    `semistable_modularity_law` and `no_bridge_law` inputs with the imported
    semistable-modularity data plus the selected curve-local providers.
  - `fermat_positive_integer_solution_false_from_semistable_modularity_data`
    derives the explicit concrete positive-integer solution contradiction from
    imported semistable-modularity data by first constructing the route
    semistable-modularity and no-bridge laws.
  - Completed L2 semistable-modularity-data positive-arithmetic transport
    targets:
    `fermat_positive_arithmetic_solution_false_from_semistable_modularity_data`,
    `fermat_last_theorem_positive_arithmetic_from_semistable_modularity_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_semistable_modularity_data`.
    These convert a positive-arithmetic solution to the positive-integer
    solution record under an explicit `Positive -> Nonzero` law, then consume
    the semistable-data positive-integer contradiction; they do not assume the
    final positive-arithmetic negation or alias a supplied selected law.
  - `fermat_route_ribet_law_from_ribet_bridge_data` remains the explicit
    bridge-backed compatibility decomposition: it derives the Frey route
    `ribet_level_lowering_law` from imported `RibetBridgeData`, its underlying
    `LevelLoweringData`, and selected conductor, residual-irreducibility,
    ramification, newform, excluded-case, and lowered-level providers for the
    Frey route context.
  - `fermat_route_ribet_law_from_level_lowering_data` is the bridge-free Ribet
    route decomposition: it derives the same Frey route
    `ribet_level_lowering_law` directly from `LevelLoweringData` plus the
    selected conductor, residual-irreducibility, ramification, newform,
    excluded-case, and lowered-level providers, without accepting
    `RibetBridgeData` or bridge-backed theorem-surface parameters.
  - `fermat_last_theorem_from_ribet_bridge_data` is the next final-statement
    L2 wrapper: it should replace the direct `ribet_level_lowering_law` input
    with that imported Ribet bridge data plus selected route-local providers.
  - `fermat_positive_integer_solution_false_from_ribet_bridge_data` derives
    the explicit concrete positive-integer solution contradiction from imported
    Ribet bridge data by first constructing the Frey route
    level-lowering law.
  - `FermatLevelTwoObstructionData` is the next local closure target: it
    should split the direct `level_two_contradiction_law` into a law deriving
    a level-two obstruction from Ribet level lowering and a law deriving the
    final contradiction from that obstruction.
  - `fermat_route_level_two_contradiction_law_from_level_two_obstruction_data`
    is the next route-law composition target: it should derive the Frey route
    `level_two_contradiction_law` by composing the two laws stored in
    `FermatLevelTwoObstructionData`.
  - `fermat_last_theorem_from_level_two_obstruction_data` is the next
    final-statement L2 wrapper: it should replace the direct
    `level_two_contradiction_law` input with the level-two obstruction data.
  - `fermat_positive_integer_solution_false_from_level_two_obstruction_data`
    derives the explicit concrete positive-integer solution contradiction from
    level-two obstruction data by first constructing the route
    level-two-contradiction law.
  - `FermatNoCounterexampleData` is the next local closure target: it should
    split the direct `no_counterexample_law` into a law deriving a named
    contradiction attached to the Fermat counterexample from a
    level-two contradiction, and a law translating that named contradiction
    into `NoFermatCounterexample`.
  - `fermat_route_no_counterexample_law_from_no_counterexample_data` is the
    next route-law composition target: it should derive the Frey route
    `no_counterexample_law` by composing the two laws stored in
    `FermatNoCounterexampleData`.
  - `fermat_last_theorem_from_no_counterexample_data` is the next
    final-statement L2 wrapper: it should replace the direct
    `no_counterexample_law` input with the no-counterexample data.
  - `fermat_positive_integer_solution_false_from_no_counterexample_data`
    derives the explicit concrete positive-integer solution contradiction from
    no-counterexample data by first constructing the Frey route
    no-counterexample law.
  - `fermat_positive_integer_solution_false_from_no_counterexample_data_bridge_free`
    is the bridge-free no-counterexample-data wrapper: it derives all direct
    route laws from semistable-modularity data, bridge-free
    `LevelLoweringData`, level-two-obstruction data, and no-counterexample
    data, then calls the Frey-model-law contradiction.
  - `fermat_last_theorem_from_no_counterexample_data_bridge_free` is the
    corresponding final-statement wrapper over the same bridge-free data
    boundary.
  - `FermatGlobalRawRefutationData` is a compatibility global closure that
    packages the selected raw-realization provider and the
    no-raw-counterexample law used to refute the raw counterexample obtained
    from a positive-integer solution.
  - `fermat_last_theorem_from_global_raw_refutation_data` is the
    final-statement L2 wrapper that replaces the direct
    `realizes_raw_provider` and `no_raw_counterexample_law` inputs with that
    global raw-refutation data, then wraps the route-data positive-integer
    solution `False` theorem with `not_intro`.
  - `fermat_global_raw_refutation_data_from_raw_elimination_provider` derives
    the global raw-refutation data from the stronger solution-indexed
    raw-elimination provider by projecting raw realization evidence and
    rebuilding the `Not` law from local raw-elimination data.
  - `fermat_global_raw_refutation_data_realizes_raw_provider` projects the raw
    realization provider from an explicit `FermatGlobalRawRefutationData`
    closure.
  - `fermat_raw_realizes_from_global_raw_refutation_data` specializes that
    projected provider to a concrete formula-specialized raw counterexample
    datum.
  - `fermat_global_raw_refutation_data_no_raw_counterexample_law` projects the
    no-raw-counterexample law from the same closure.
  - `fermat_not_raw_counterexample_from_global_raw_refutation_data_via_components`
    recomposes the projected no-raw law with the concrete raw-realization
    specialization to derive `Not` of the selected formula-specialized raw
    counterexample.
  - `fermat_raw_counterexample_false_from_global_raw_refutation_data_via_components`
    applies that component-wise `Not` proof to the selected raw datum to derive
    `False`.
  - `fermat_positive_integer_solution_false_from_global_raw_refutation_data`
    turns a concrete positive-integer solution into the selected raw datum and
    applies the raw-refutation contradiction under the corresponding
    `NoFermatCounterexample`.
  - `fermat_not_positive_integer_solution_from_global_raw_refutation_data`
    wraps that contradiction with `not_intro`, giving a local
    positive-integer no-solution theorem from global raw-refutation data and a
    selected no-counterexample provider.
  - `fermat_positive_integer_solution_false_from_global_raw_refutation_data_and_route_laws`
    derives that selected no-counterexample evidence from Frey-model laws and
    Wiles/Ribet route laws, then applies the raw-refutation contradiction.
  - `fermat_not_positive_integer_solution_from_global_raw_refutation_data_and_route_laws`
    wraps the route-law contradiction as a local positive-integer no-solution
    theorem.
  - `fermat_positive_integer_solution_false_from_global_raw_refutation_data_and_route_data`
    replaces the direct semistable-modularity, no-bridge, Ribet,
    level-two-contradiction, and no-counterexample route-law inputs in that
    local contradiction by the corresponding imported/packaged route data, and
    now uses the bridge-free `LevelLoweringData` Ribet route rather than
    `RibetBridgeData`.
  - `fermat_not_positive_integer_solution_from_global_raw_refutation_data_and_route_data`
    wraps the route-data contradiction as a local positive-integer no-solution
    theorem.
  - `fermat_raw_primitive_frey_route_provider_from_frey_model_laws` constructs
    the pointwise raw primitive Frey route provider from explicit
    primitive-normalization providers, Frey-model laws, semistability-from-model,
    and the direct route laws for semistable modularity, no-bridge, Ribet,
    level-two contradiction, and no-counterexample.
  - `fermat_raw_primitive_frey_route_provider_from_route_data` constructs the
    same provider after replacing those direct route-law inputs by imported
    semistable-modularity data, bridge-free `LevelLoweringData`, level-two
    obstruction data, and no-counterexample data.
  - `fermat_global_raw_elimination_provider_from_frey_model_laws` constructs
    that stronger raw-elimination provider from those direct Frey-model/route
    law prerequisites plus `FermatGlobalRawRefutationData`, materializing the
    pointwise `FermatRawCounterexampleEliminationData` provider instead of only
    consuming it at the final theorem boundary.
  - `fermat_global_raw_elimination_provider_from_route_data_and_global_raw_refutation_data`
    constructs the stronger raw-elimination provider from the imported route
    data closures and `FermatGlobalRawRefutationData`, routing through the
    route-data raw primitive Frey provider rather than direct route-law inputs.
  - `fermat_global_elimination_data_from_route_data_and_global_raw_refutation_data`
    packages that route-data raw-elimination provider into the
    formula-specialized `FermatGlobalEliminationData` closure.
  - `fermat_positive_integer_global_elimination_data_from_route_data_and_global_raw_refutation_data`
    transports that formula-specialized closure to
    `FermatPositiveIntegerGlobalEliminationData` for concrete final-statement
    consumers.
  - `fermat_global_elimination_data_from_route_data_and_global_raw_elimination_provider`
    packages the same route-data formula-specialized
    `FermatGlobalEliminationData` closure from the stronger solution-indexed
    raw-elimination provider by first deriving `FermatGlobalRawRefutationData`.
  - `fermat_positive_integer_global_elimination_data_from_route_data_and_global_raw_elimination_provider`
    derives the same concrete elimination closure from the stronger
    solution-indexed raw-elimination provider through that formula-specialized
    route-data closure.
  - `fermat_last_theorem_from_route_data_and_global_raw_refutation_data` is the
    latest-route final-statement wrapper that now wraps
    `fermat_positive_integer_solution_false_from_global_raw_refutation_data_and_route_data`
    directly with `not_intro`, rather than routing the final `Not` step through
    another final-statement wrapper.
  - `fermat_positive_integer_solution_false` is the public final-statement
    explicit contradiction theorem: under primitive-normalization component
    providers, primitive-realization, selected builds-curve data, generic
    Frey-model laws, direct Wiles/Ribet route laws, raw-realization, and
    no-raw-counterexample inputs, it calls
    `fermat_positive_integer_solution_false_from_frey_model_laws` directly to
    derive `False`.
  - `fermat_global_elimination_data_from_frey_model_laws_and_global_raw_refutation_data`
    packages the formula-specialized `FermatGlobalEliminationData` closure from
    the direct Frey-model/route-law raw-elimination provider and
    `FermatGlobalRawRefutationData`, so downstream final wrappers can consume a
    data closure rather than the pointwise provider directly.
  - `fermat_global_elimination_data_from_frey_model_laws_and_global_raw_elimination_provider`
    packages the same formula-specialized `FermatGlobalEliminationData` closure
    from direct Frey-model/route laws and the stronger solution-indexed raw
    elimination provider by first deriving `FermatGlobalRawRefutationData`.
  - `fermat_positive_integer_global_elimination_data_from_frey_model_laws_and_global_raw_elimination_provider`
    transports that direct Frey-model-law global elimination closure to
    `FermatPositiveIntegerGlobalEliminationData`, so the concrete
    positive-integer final wrappers can consume the derived closure directly.
  - `fermat_positive_integer_global_elimination_data_raw_of_solution` projects
    the certified positive-integer-solution-to-raw-counterexample map from a
    `FermatPositiveIntegerGlobalEliminationData` closure.
  - `fermat_positive_integer_global_elimination_data_solution_to_raw_counterexample`
    applies that projected map to a concrete positive-integer solution,
    producing the formula-specialized raw counterexample data used by the
    raw-elimination route.
  - `fermat_positive_integer_solution_false_from_positive_integer_global_elimination_data`
    separates the concrete contradiction step from
    `FermatPositiveIntegerGlobalEliminationData`: it extracts the
    solution-specific raw counterexample, obtains the certified raw-elimination
    data for that raw counterexample, and eliminates the raw counterexample to
    derive `False`.
  - `fermat_last_theorem_from_positive_integer_global_elimination_data` now
    wraps that explicit contradiction theorem in `Not`, instead of performing
    the raw-counterexample elimination directly inside the final wrapper.
  - `fermat_last_theorem_from_frey_model_laws_and_global_raw_refutation_data`
    is the direct Frey-model-law final-statement wrapper: it now wraps
    `fermat_positive_integer_solution_false_from_global_raw_refutation_data_and_route_laws`
    directly with `not_intro`, keeping the explicit contradiction separate
    from the final `Not` wrapper.
  - `fermat_last_theorem_from_frey_model_laws_and_global_raw_elimination_provider`
    is the direct Frey-model-law final-statement wrapper that replaces
    `FermatGlobalRawRefutationData` with the stronger solution-indexed raw
    elimination provider, derives the raw-refutation data by certified
    projection, and then wraps the same direct route-law contradiction theorem
    with `not_intro`.
  - `fermat_last_theorem_from_global_raw_elimination_provider` is the
    corresponding latest-route final-statement wrapper: it uses imported
    semistable-modularity data, bridge-free `LevelLoweringData`, selected Ribet
    route providers, level-two-obstruction data, no-counterexample data, and
    the solution-indexed raw-elimination provider to derive the concrete
    route-data `FermatPositiveIntegerGlobalEliminationData` closure before
    wrapping
    `fermat_positive_integer_solution_false_from_positive_integer_global_elimination_data`
    with `not_intro`, with no `RibetBridgeData` prerequisite on this wrapper.
  - `fermat_positive_integer_solution_false_from_global_raw_elimination_provider`
    separates the explicit concrete positive-integer `False` contradiction
    from the solution-indexed raw-elimination-provider final wrapper under the
    same bridge-free route-data boundary.
  - `fermat_positive_integer_solution_false_from_solution_raw_elimination_provider`
    separates the explicit concrete positive-integer `False` contradiction
    from the earlier solution-indexed raw-elimination-provider boundary before
    `fermat_global_not_positive_integer_solution_from_solution_raw_elimination_provider`
    wraps it as `Not`.
  - `fermat_positive_integer_solution_false_from_solution_raw_primitive_frey_route_provider`
    and
    `fermat_global_not_positive_integer_solution_from_solution_raw_primitive_frey_route_provider`
    add the analogous solution-indexed boundary directly over
    `FermatRawPrimitiveFreyRouteData`, so this intermediate route no longer
    needs a provider for all raw counterexamples.
  - `fermat_solution_raw_elimination_provider_from_solution_raw_primitive_frey_route_provider`
    constructs the solution-indexed raw-elimination provider from that same
    solution-indexed raw-primitive-route-data boundary.
  - `fermat_positive_integer_global_elimination_data_from_solution_raw_primitive_frey_route_provider`
    packages the constructed provider into
    `FermatPositiveIntegerGlobalEliminationData`, making the closure available
    to downstream positive-integer final-statement consumers.
  - `fermat_solution_raw_primitive_frey_route_provider_from_solution_normalization_and_primitive_route_provider`
    and
    `fermat_positive_integer_global_elimination_data_from_solution_primitive_frey_route_provider`
    add the analogous solution-indexed primitive-normalization /
    primitive-Frey-route boundary, so this intermediate route no longer needs
    primitive normalization or primitive route data for every raw counterexample.
  - `fermat_solution_primitive_frey_route_provider_from_solution_normalization_frey_model_and_route_data`
    and
    `fermat_positive_integer_global_elimination_data_from_solution_frey_model_and_route_data`
    add the solution-indexed Frey-model / Wiles-Ribet route-data boundary, so
    the intermediate route no longer needs a primitive-Frey-route provider for
    every raw counterexample.
  - `fermat_solution_frey_model_provider_from_solution_builds_curve_and_frey_model_laws`
    and
    `fermat_positive_integer_global_elimination_data_from_solution_builds_curve_and_frey_model_laws_and_route_data`
    split the solution-indexed Frey-model provider into selected builds-curve
    data plus generic Frey-model laws.
  - `fermat_solution_primitive_normalization_provider_from_solution_normalization_laws`
    and
    `fermat_positive_integer_global_elimination_data_from_solution_normalization_laws_builds_curve_and_route_data`
    split the solution-indexed primitive-normalization provider into its
    primitive component laws before packaging the same global elimination
    closure.
  - `fermat_positive_integer_global_elimination_data_from_solution_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_integer_solution_false_from_solution_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_global_not_positive_integer_solution_from_solution_normalization_laws_builds_curve_and_route_laws`
    remove the explicit `FermatWilesRibetRouteData` input from that
    solution-indexed boundary by constructing it from route laws, then expose
    both the concrete contradiction and the global `Not` wrapper.
  - `fermat_positive_integer_global_elimination_data_from_solution_normalization_laws_builds_curve_and_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_solution_false_from_solution_normalization_laws_builds_curve_and_no_counterexample_data_bridge_free`,
    and
    `fermat_global_not_positive_integer_solution_from_solution_normalization_laws_builds_curve_and_no_counterexample_data_bridge_free`
    remove the direct route-law inputs from the same solution-indexed boundary
    by deriving them from semistable-modularity data, bridge-free
    level-lowering data, level-two-obstruction data, and no-counterexample
    data.
  - `fermat_positive_integer_global_elimination_data_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_integer_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_global_not_positive_integer_solution_from_global_normalization_laws_builds_curve_and_route_laws`
    bridge the raw-indexed public primitive/builds-curve provider surface to
    that solution-indexed route-law boundary by applying each raw provider to
    the canonical raw datum obtained from the assumed positive-integer solution.
  - `fermat_positive_integer_global_elimination_data_from_no_counterexample_data_bridge_free`
    exposes the bridge-free public surface as
    `FermatPositiveIntegerGlobalEliminationData` by first adapting each
    raw-indexed primitive/builds-curve provider to the canonical raw datum
    generated from a concrete positive-integer solution, then using the
    solution-indexed bridge-free no-counterexample-data closure.
  - Completed L2 bridge-free no-counterexample-data positive-arithmetic and
    standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_no_counterexample_data_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`.
  - Completed L2 bridge-free no-counterexample-data formula-closure targets:
    `fermat_global_elimination_data_from_no_counterexample_data_bridge_free`,
    `fermat_positive_solution_false_from_no_counterexample_data_bridge_free`,
    `fermat_not_positive_solution_from_no_counterexample_data_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_no_counterexample_data_bridge_free`.
  - Completed L2 ordered-field bridge-free no-counterexample-data
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_no_counterexample_data_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_no_counterexample_data_bridge_free`,
    deriving the primitive `Nonzero` providers and public
    `Positive -> Nonzero` bridge from the ordered-field interpretation before
    applying the bridge-free no-counterexample-data route.
  - Completed L2 ordered-field bridge-free no-counterexample-data formula-closure targets:
    `fermat_global_elimination_data_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_no_counterexample_data_bridge_free`,
    and
    `fermat_not_positive_solution_from_ordered_field_no_counterexample_data_bridge_free`.
  - Completed L2 standard ordered-field bridge-free no-counterexample-data
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`.
  - Completed L2 standard ordered-field bridge-free no-counterexample-data formula-closure targets:
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`.
  - `fermat_positive_integer_global_elimination_data_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_no_counterexample_laws_bridge_free`,
    and `fermat_last_theorem_from_no_counterexample_laws_bridge_free`
    replace the remaining no-counterexample data package on that bridge-free
    public boundary by its two constructor laws, then expose the same global
    elimination data, concrete contradiction, and global `Not` forms.
  - Completed L2 bridge-free no-counterexample-law positive-arithmetic public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_no_counterexample_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free no-counterexample-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_no_counterexample_laws_bridge_free`,
    deriving the primitive `Nonzero` providers and public
    `Positive -> Nonzero` bridge from the ordered-field interpretation instead
    of leaving them as separate public assumptions.
  - Completed L2 ordered-field bridge-free no-counterexample-law formula-closure targets:
    `fermat_global_elimination_data_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_no_counterexample_laws_bridge_free`,
    and
    `fermat_not_positive_solution_from_ordered_field_no_counterexample_laws_bridge_free`.
  - Completed L2 standard bridge-free no-counterexample-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`.
  - Completed L2 bridge-free no-counterexample-law formula-closure targets:
    `fermat_global_elimination_data_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_solution_false_from_no_counterexample_laws_bridge_free`,
    `fermat_not_positive_solution_from_no_counterexample_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`.
  - Completed L2 standard ordered-field bridge-free no-counterexample-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`.
  - Completed L2 standard ordered-field bridge-free no-counterexample-law formula-closure targets:
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`.
  - `fermat_positive_integer_global_elimination_data_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_level_two_obstruction_laws_bridge_free`,
    and `fermat_last_theorem_from_level_two_obstruction_laws_bridge_free`
    also replace the remaining level-two-obstruction data package on that
    bridge-free public boundary by its obstruction and contradiction constructor
    laws before deriving the same global elimination data, concrete
    contradiction, and global `Not` forms.
  - Completed L2 bridge-free level-two-obstruction-law positive-arithmetic public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_level_two_obstruction_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free level-two-obstruction-law
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from the ordered-field interpretation before
    applying the bridge-free level-two-obstruction-law route.
  - Completed L2 standard bridge-free level-two-obstruction-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`.
  - Completed L2 standard ordered-field bridge-free level-two-obstruction-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from the ordered-field interpretation before
    applying the standard bridge-free level-two-obstruction-law route.
  - Completed L2 bridge-free level-two-obstruction-law formula-closure targets for
    this batch:
    `fermat_global_elimination_data_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_solution_false_from_level_two_obstruction_laws_bridge_free`,
    `fermat_not_positive_solution_from_level_two_obstruction_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_level_two_obstruction_laws_bridge_free`
    will expose the level-two-obstruction-law boundary as
    formula-specialized global-elimination data and positive-solution
    consumers, deriving the no-counterexample law package from the explicit
    level-two obstruction and contradiction laws.
  - `fermat_positive_integer_global_elimination_data_from_level_lowering_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_level_lowering_laws_bridge_free`,
    and `fermat_last_theorem_from_level_lowering_laws_bridge_free`
    similarly replace the remaining bridge-free `LevelLoweringData` package by
    its dependency-map, level-lowering conclusion, and non-Frey reuse laws before
    deriving the same global elimination data, concrete contradiction, and
    global `Not` forms.
  - Completed L2 bridge-free level-lowering-law positive-arithmetic and standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_level_lowering_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_level_lowering_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free level-lowering-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_level_lowering_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_level_lowering_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free level-lowering-law route.
  - Completed L2 standard ordered-field bridge-free level-lowering-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    specializing the same ordered-field-derived level-lowering-law boundary to
    `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree`.
  - Completed L2 bridge-free level-lowering-law formula-closure targets for this
    batch:
    `fermat_global_elimination_data_from_level_lowering_laws_bridge_free`,
    `fermat_positive_solution_false_from_level_lowering_laws_bridge_free`,
    `fermat_not_positive_solution_from_level_lowering_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_level_lowering_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_laws_bridge_free`
    will expose the level-lowering-law boundary as formula-specialized
    global-elimination data and positive-solution consumers, deriving
    `LevelLoweringData` from the dependency-map, conclusion, and non-Frey laws.
  - `fermat_route_ribet_law_from_level_lowering_core_laws`,
    `fermat_positive_integer_global_elimination_data_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_level_lowering_core_laws_bridge_free`,
    and `fermat_last_theorem_from_level_lowering_core_laws_bridge_free`
    remove the non-Frey reuse law and its generic terminology/non-Frey marker
    parameters from the public FLT boundary by deriving the route-level
    `RibetLevelLowering` law directly from the dependency-map and
    level-lowering conclusion laws plus the selected route providers.
  - Completed L2 bridge-free level-lowering-core-law positive-arithmetic
    and standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_level_lowering_core_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free level-lowering-core-law
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free level-lowering-core-law route, without
    reintroducing generic terminology, non-Frey marker, or non-Frey reuse
    level-lowering inputs.
  - Completed L2 formula-level level-lowering-core-law closure targets for this batch:
    `fermat_global_elimination_data_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_solution_false_from_level_lowering_core_laws_bridge_free`,
    `fermat_not_positive_solution_from_level_lowering_core_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_level_lowering_core_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_level_lowering_core_laws_bridge_free`
    expose the level-lowering-core-law boundary as formula-specialized
    global-elimination data and positive-solution consumers.
  - Completed L2 route cleanup for
    `fermat_positive_integer_global_elimination_data_from_level_lowering_core_laws_bridge_free`:
    it now adapts the raw-indexed primitive-normalization and builds-curve
    providers to the canonical raw datum generated by a concrete positive
    integer solution, then applies the solution-indexed route-law boundary
    before exposing the level-lowering-core-law global-elimination data.
  - `fermat_positive_integer_global_elimination_data_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_solution_false_from_minimal_modularity_lifting_core_bridge_free`,
    and `fermat_last_theorem_from_minimal_modularity_lifting_core_bridge_free`
    remove the unused deformation-functor, Hecke-comparison, `R_eq_T`,
    nonminimal-lifting, and modularity-lifting non-Frey reuse inputs from the
    public FLT boundary; the remaining modularity input is the minimal
    modularity-lifting law applied to the selected residual-irreducible,
    local-condition, minimal-condition, and modularity-assumption providers.
  - Completed L2 minimal-modularity-lifting-core positive-arithmetic and
    standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`.
  - Completed L2 ordered-field bridge-free minimal-modularity-lifting-core
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free minimal-modularity-lifting-core route, without
    reintroducing deformation-functor, Hecke-comparison, `R_eq_T`,
    nonminimal-lifting, or modularity-lifting non-Frey reuse inputs.
  - Completed L2 formula-level minimal-modularity-lifting-core closure targets for this batch:
    `fermat_global_elimination_data_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_solution_false_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_not_positive_solution_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_minimal_modularity_lifting_core_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_minimal_modularity_lifting_core_bridge_free`
    expose the minimal-modularity-lifting-core boundary as formula-specialized
    global-elimination data and positive-solution consumers.
  - Completed L2 route cleanup for
    `fermat_positive_integer_global_elimination_data_from_minimal_modularity_lifting_core_bridge_free`:
    it now uses the same solution-indexed primitive-normalization /
    builds-curve route-law boundary as the level-lowering-core cleanup,
    specializing raw-indexed providers to the canonical raw datum obtained from
    each concrete positive integer solution before applying the
    minimal-modularity-lifting-derived route laws.
  - Completed L2 standard formula-positive-solution public-surface targets:
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three` and
    `fermat_last_theorem_positive_solution_std_nat_kernel_eq_at_least_three`
    expose bare public `Std.Nat`/kernel-equality formula-specialized
    positive-solution surfaces through the ordered-field
    minimal-modularity/lifting-core bridge-free closure.
  - `fermat_positive_integer_global_elimination_data_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_semistable_modularity_laws_bridge_free`,
    and `fermat_last_theorem_from_semistable_modularity_laws_bridge_free`
    replace the remaining `SemistableModularityData` package on the bridge-free
    public boundary by its reusable-assumptions, modularity-conclusion,
    semistable-route, and no-bridge constructor laws before deriving the same
    global elimination data, concrete contradiction, and global `Not` forms.
  - Completed L2 semistable-modularity-law positive-arithmetic and
    standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_semistable_modularity_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free semistable-modularity-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_semistable_modularity_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_semistable_modularity_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free semistable-modularity-law route.
  - Completed L2 standard ordered-field bridge-free semistable-modularity-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    specializing the same ordered-field-derived semistable-modularity-law
    boundary to `Std.Nat.Basic`, kernel equality, and
    `FermatStdNatAtLeastThree`.
  - Completed L2 formula-level semistable-modularity-law closure targets:
    `fermat_global_elimination_data_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_solution_false_from_semistable_modularity_laws_bridge_free`,
    `fermat_not_positive_solution_from_semistable_modularity_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_semistable_modularity_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_modularity_laws_bridge_free`,
    deriving the global formula-level elimination package, direct contradiction,
    and `Not` form from the bridge-free semistable-modularity-law boundary.
  - `fermat_positive_integer_global_elimination_data_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_modularity_lifting_laws_bridge_free`,
    and `fermat_last_theorem_from_modularity_lifting_laws_bridge_free`
    replace the remaining `ModularityLiftingData` package on the public
    bridge-free boundary by its deformation-ring, Hecke-comparison, `R_eq_T`,
    minimal lifting, nonminimal lifting, and non-Frey reuse constructor laws
    before deriving the same global elimination data, concrete contradiction,
    and global `Not` forms.
  - Completed L2 modularity-lifting-law positive-arithmetic and standard
    public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_modularity_lifting_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free modularity-lifting-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_modularity_lifting_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_modularity_lifting_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free modularity-lifting-law route.
  - Completed L2 standard ordered-field bridge-free modularity-lifting-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    specializing the same ordered-field-derived modularity-lifting-law
    boundary to `Std.Nat.Basic`, kernel equality, and
    `FermatStdNatAtLeastThree`.
  - Completed L2 formula-level modularity-lifting-law closure targets:
    `fermat_global_elimination_data_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_solution_false_from_modularity_lifting_laws_bridge_free`,
    `fermat_not_positive_solution_from_modularity_lifting_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_modularity_lifting_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_lifting_laws_bridge_free`,
    deriving the global formula-level elimination package, direct contradiction,
    and `Not` form from the bridge-free modularity-lifting-law boundary.
  - `fermat_positive_integer_global_elimination_data_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_modularity_conclusion_laws_bridge_free`,
    and `fermat_last_theorem_from_modularity_conclusion_laws_bridge_free`
    replace the selected `modularity_conclusion_of_curve` provider by
    curve-indexed residual-irreducible, local-condition, minimal-condition, and
    modularity-assumption providers, deriving the required
    `ModularityConclusion` value with the minimal modularity-lifting law before
    deriving the same global elimination data, concrete contradiction, and
    global `Not` forms.
  - Completed L2 modularity-conclusion-law positive-arithmetic and standard
    public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_modularity_conclusion_laws_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`.
  - Completed L2 ordered-field bridge-free modularity-conclusion-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data before
    applying the bridge-free modularity-conclusion-law route.
  - Completed L2 standard ordered-field bridge-free modularity-conclusion-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    specializing the same ordered-field-derived modularity-conclusion-law
    boundary to `Std.Nat.Basic`, kernel equality, and
    `FermatStdNatAtLeastThree`.
  - Completed L2 formula-level modularity-conclusion-law closure targets:
    `fermat_global_elimination_data_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_solution_false_from_modularity_conclusion_laws_bridge_free`,
    `fermat_not_positive_solution_from_modularity_conclusion_laws_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_modularity_conclusion_laws_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_modularity_conclusion_laws_bridge_free`,
    deriving the global formula-level elimination package, direct contradiction,
    and `Not` form from the bridge-free modularity-conclusion-law boundary.
  - `fermat_positive_integer_global_elimination_data_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_integer_solution_false_from_semistable_assumptions_identity_bridge_free`,
    and `fermat_last_theorem_from_semistable_assumptions_identity_bridge_free`
    remove the reusable semistability assumptions law from the public boundary
    by deriving it as the identity proof
    `FreyCurveSemistable curve -> FreyCurveSemistable curve`, then deriving the
    same global elimination data, concrete contradiction, and global `Not`
    forms.
  - Completed L2 semistable-assumptions-identity positive-arithmetic and
    standard public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_semistable_assumptions_identity_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`.
  - Completed L2 ordered-field bridge-free semistable-assumptions-identity public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data while
    keeping the semistability assumptions law as the identity proof.
  - Completed L2 standard ordered-field bridge-free semistable-assumptions-identity public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    specializing the same ordered-field-derived identity-assumptions boundary
    to `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree`.
  - Completed L2 formula-level semistable-assumptions-identity closure targets:
    `fermat_global_elimination_data_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_solution_false_from_semistable_assumptions_identity_bridge_free`,
    `fermat_not_positive_solution_from_semistable_assumptions_identity_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_semistable_assumptions_identity_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_assumptions_identity_bridge_free`,
    deriving the global formula-level elimination package, direct contradiction,
    and `Not` form from the bridge-free semistable-assumptions identity
    boundary.
  - `fermat_positive_integer_global_elimination_data_from_semistable_route_identity_bridge_free`,
    `fermat_positive_integer_solution_false_from_semistable_route_identity_bridge_free`,
    and `fermat_last_theorem_from_semistable_route_identity_bridge_free`
    remove the semistable modularity route law from the public boundary by
    deriving it as the identity proof on the already constructed
    `SemistableModular curve` conclusion, then deriving the same global
    elimination data, concrete contradiction, and global `Not` forms.
  - Completed L2 semistable-route-identity positive-arithmetic and standard
    public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_semistable_route_identity_bridge_free`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_semistable_route_identity_bridge_free`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`.
  - Completed L2 ordered-field bridge-free semistable-route-identity public-surface targets:
    `fermat_positive_integer_global_elimination_data_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_integer_solution_false_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_semistable_route_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_semistable_route_identity_bridge_free`,
    deriving primitive `Nonzero` providers and the public
    `Positive -> Nonzero` bridge from ordered-field interpretation data while
    keeping the semistable route law as the identity proof.
  - Completed L2 standard ordered-field bridge-free semistable-route-identity public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    specializing the same ordered-field-derived route-identity boundary to
    `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree`.
  - Completed L2 formula-level semistable-route-identity closure targets:
    `fermat_global_elimination_data_from_semistable_route_identity_bridge_free`,
    `fermat_positive_solution_false_from_semistable_route_identity_bridge_free`,
    `fermat_not_positive_solution_from_semistable_route_identity_bridge_free`,
    `fermat_global_elimination_data_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_solution_false_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_not_positive_solution_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_semistable_route_identity_bridge_free`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_semistable_route_identity_bridge_free`,
    deriving the global formula-level elimination package, direct contradiction,
    and `Not` form from the bridge-free semistable-route identity boundary.
  - `fermat_positive_integer_solution_false_from_positive_solution_data_provider`
    separates the explicit concrete positive-integer `False` contradiction
    from the positive-solution-data provider boundary before
    `fermat_global_not_positive_integer_solution_from_provider` wraps it as
    `Not`.
  - The public `fermat_positive_integer_solution_false` and
    `fermat_last_theorem` wrappers now use the bridge-free
    minimal-modularity-lifting core boundary instead of the compatibility global
    raw-refutation, solution-indexed raw-elimination-provider, direct route-law,
    no-counterexample-data, level-two-obstruction, level-lowering-data, or
    semistable-route-identity/level-lowering-core boundaries. The remaining
    public blockers are the primitive-normalization component providers,
    primitive realization,
    selected builds-curve provider, generic Frey-model laws,
    minimal modularity-lifting law, selected lifting-condition providers,
    semistable-modularity conclusion/no-bridge constructor laws and selected
    Galois/modularity-condition providers,
    bridge-free level-lowering dependency/conclusion laws and selected Ribet
    route providers, level-two-obstruction constructor laws,
    no-counterexample constructor laws, raw-realization provider, and
    no-raw-counterexample law.
  - `fermat_no_counterexample_from_selected_raw_counterexample_route`,
    `fermat_not_raw_counterexample_from_selected_raw_counterexample_route`,
    `fermat_positive_integer_solution_false_from_selected_raw_counterexample_route`,
    and `fermat_last_theorem_from_selected_raw_counterexample_route` replace
    that public boundary by a selected raw counterexample route that derives
    `NoFermatCounterexample
    (counterexample_of x y z n)` directly from the selected Frey curve,
    `rho_of x y z n`, minimal modularity-lifting law, pointwise selected
    residual/local/minimal/modularity-condition providers, bridge-free
    level-lowering laws, level-two-obstruction constructor laws, and
    no-counterexample constructor laws. This avoids the global all-curve
    `rho_of_curve`/`frey_galois_of_curve` and curve-indexed modularity-condition
    provider boundary and does not introduce an alias-only theorem or assume
    the FLT conclusion. The public `fermat_positive_integer_solution_false` and
    `fermat_last_theorem` wrappers now use this selected raw route boundary.
  - `fermat_no_counterexample_from_selected_raw_counterexample_facts`,
    `fermat_not_raw_counterexample_from_selected_raw_counterexample_facts`,
    `fermat_positive_integer_solution_false_from_selected_raw_counterexample_facts`,
    and `fermat_last_theorem_from_selected_raw_counterexample_facts` replace
    the remaining generic `frey_galois_law` and `route_*_provider` inputs on
    the public selected raw route boundary by pointwise selected providers for
    `FreyGaloisRepresentation (curve_of x y z n) (rho_of x y z n)`,
    `Conductor`, `RamificationControlled`, `NewformAtLevel`, `ExcludedCase`,
    and `LoweredLevel` at the selected `rho_of x y z n` and selected levels.
    The proof still derives `DependencyMap`, `RibetLevelLowering`,
    `LevelTwoContradiction`, `NoFermatCounterexample`, and `Not raw` rather
    than returning any input unchanged.
  - `fermat_no_counterexample_from_selected_raw_counterexample_slim_facts`,
    `fermat_not_raw_counterexample_from_selected_raw_counterexample_slim_facts`,
    `fermat_positive_integer_solution_false_from_selected_raw_counterexample_slim_facts`,
    and `fermat_last_theorem_from_selected_raw_counterexample_slim_facts`
    replace the public selected-facts wrappers with a base-positive-integer
    boundary that no longer exposes the
    unused `PairwiseCoprime`, primitive-selector, primitive-realization,
    `FreyCurveSemistable`, Frey discriminant/conductor/minimal-model,
    `SemistableModular`, or `NoBridgeAxiomDependency` surface while preserving
    the same selected builds-curve, selected Galois, selected level-lowering,
    level-two, no-counterexample, raw-realization, and no-raw facts.
  - `fermat_dependency_map_from_selected_raw_contradiction_facts`,
    `fermat_ribet_lowering_from_selected_raw_contradiction_facts`,
    `fermat_level_two_contradiction_from_selected_raw_contradiction_facts`,
    `fermat_not_raw_counterexample_from_selected_raw_contradiction_facts`,
    `fermat_positive_integer_solution_false_from_selected_raw_contradiction_facts`,
    and `fermat_last_theorem_from_selected_raw_contradiction_facts` replace the
    public slim selected-facts wrappers with a route that derives
    `DependencyMap`, `RibetLevelLowering`, and `LevelTwoContradiction` from
    selected level-lowering facts, then applies a selected raw-contradiction law
    `LevelTwoContradiction (rho_of x y z n) -> Not raw`. This should remove
    `Counterexample`, `FreyCurve`, `BuildsFreyCurve`,
    `FreyGaloisRepresentation`, `NoFermatCounterexample`,
    `RawCounterexampleRealizes`, selected builds/Galois/raw-realization
    providers, and the counterexample/no-counterexample laws from the public
    final-statement boundary while still deriving the lowering and
    level-two-contradiction steps rather than returning a supplied final `Not`
    proof unchanged.
  - `fermat_dependency_map_from_selected_direct_level_two_facts`,
    `fermat_ribet_lowering_from_selected_direct_level_two_facts`,
    `fermat_level_two_contradiction_from_selected_direct_level_two_facts`,
    `fermat_not_raw_counterexample_from_selected_direct_level_two_facts`,
    `fermat_positive_integer_solution_false_from_selected_direct_level_two_facts`,
    and `fermat_last_theorem_from_selected_direct_level_two_facts` replace the
    public `LevelTwoObstruction`, `level_two_obstruction_law`, and
    `level_two_obstruction_contradiction_law` inputs by a direct selected
    level-two contradiction law from the derived Ribet lowering step, while
    preserving the proof that constructs `DependencyMap` and
    `RibetLevelLowering` from selected level-lowering facts.
  - `fermat_dependency_map_from_selected_dependency_map_contradiction_facts`,
    `fermat_level_two_contradiction_from_selected_dependency_map_contradiction_facts`,
    `fermat_not_raw_counterexample_from_selected_dependency_map_contradiction_facts`,
    `fermat_positive_integer_solution_false_from_selected_dependency_map_contradiction_facts`,
    and `fermat_last_theorem_from_selected_dependency_map_contradiction_facts`
    replace the public `RibetLevelLowering`, `level_lowering_conclusion_law`,
    and direct level-two-from-Ribet law with a selected
    dependency-map-to-level-two contradiction law. The proof still constructs
    `DependencyMap` from selected conductor, residual irreducibility,
    ramification, newform, excluded-case, and lowered-level facts before
    applying the raw contradiction law.
  - `fermat_level_two_contradiction_from_selected_route_facts_contradiction_facts`,
    `fermat_not_raw_counterexample_from_selected_route_facts_contradiction_facts`,
    `fermat_positive_integer_solution_false_from_selected_route_facts_contradiction_facts`,
    and `fermat_last_theorem_from_selected_route_facts_contradiction_facts`
    replace the public `DependencyMap` predicate,
    `level_lowering_dependency_map_law`, and dependency-map-to-level-two
    contradiction law by a direct selected route-facts-to-level-two
    contradiction law. The proof still applies the selected conductor, residual
    irreducibility, ramification, newform, excluded-case, and lowered-level
    providers to the raw counterexample before deriving the raw contradiction.
  - `fermat_level_two_contradiction_from_selected_level_two_facts`,
    `fermat_not_raw_counterexample_from_selected_level_two_facts`,
    `fermat_positive_integer_solution_false_from_selected_level_two_facts`,
    and `fermat_last_theorem_from_selected_level_two_facts` replace the public
    residual/route-fact predicates and selected route-fact providers by a
    selected `LevelTwoContradiction (rho_of x y z n)` provider for the raw
    counterexample. The proof still combines that selected level-two
    contradiction with the selected raw-contradiction law, rather than assuming
    `Not raw` or the final FLT conclusion directly.
  - Completed L2 standard selected-level-two wrapper targets:
    `fermat_level_two_contradiction_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_selected_level_two_facts`
    specialize the selected-level-two/raw-contradiction route to `Std.Nat`,
    kernel equality, and `FermatStdNatAtLeastThree`, exposing both
    positive-integer and positive-arithmetic final-statement surfaces without
    adding a final-conclusion assumption.
  - Completed L2 standard ordered-field selected-level-two wrapper targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_level_two_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_level_two_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_level_two_facts`
    derive the needed `Positive -> Nonzero` bridge from the ordered-field
    law package before consuming the same selected-level-two/raw-contradiction
    route at the standard positive-arithmetic surface.
  - `fermat_not_raw_counterexample_from_selected_no_raw_facts`,
    `fermat_positive_integer_solution_false_from_selected_no_raw_facts`, and
    `fermat_last_theorem_from_selected_no_raw_facts` replace the public
    `GaloisRepresentation`, `LevelTwoContradiction`, selected rho, selected
    level-two, and selected raw-contradiction surfaces by a selected
    `Not (FermatRawCounterexampleData ...)` law. The proof still derives the
    positive-integer contradiction by constructing the raw counterexample from
    `FermatPositiveIntegerSolutionData` and applying the selected no-raw law,
    rather than assuming the final `Not (FermatPositiveIntegerSolutionData ...)`
    conclusion.
  - Completed L2 standard selected-no-raw wrapper targets:
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_selected_no_raw_facts`
    specialize the selected no-raw boundary to `Std.Nat`, kernel equality, and
    `FermatStdNatAtLeastThree`, exposing both positive-integer and
    positive-arithmetic final-statement surfaces without assuming the final
    conclusion.
  - Completed L2 standard ordered-field selected-no-raw wrapper targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_no_raw_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_no_raw_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_no_raw_facts`
    derive the needed `Positive -> Nonzero` bridge from the ordered-field law
    package before consuming the selected no-raw law at the standard
    positive-arithmetic surface.
  - Remaining blocker for an unconditional final theorem: prove the selected
    no-raw-counterexample law itself as L2, without assuming `Not raw` or the
    final FLT conclusion. This is the point where the currently abstract Frey,
    modularity, level-lowering, and arithmetic route facts must be closed into a
    source-free certificate.
  - `fermat_not_raw_counterexample_from_selected_raw_false_facts`,
    `fermat_positive_integer_solution_false_from_selected_raw_false_facts`, and
    `fermat_last_theorem_from_selected_raw_false_facts` replace the public
    selected no-raw law with a selected raw-counterexample contradiction law
    `FermatRawCounterexampleData ... -> False`. The proof constructs
    `Not (FermatRawCounterexampleData ...)` using `not_intro` from that
    contradiction law, then constructs the raw counterexample from
    `FermatPositiveIntegerSolutionData` to refute positive-integer solutions.
  - Completed L2 standard selected-raw-false wrapper targets:
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_selected_raw_false_facts`
    specialize the selected raw-counterexample contradiction boundary to
    `Std.Nat`, kernel equality, and `FermatStdNatAtLeastThree`, exposing both
    positive-integer and positive-arithmetic final-statement surfaces.
  - Completed L2 standard ordered-field selected-raw-false wrapper targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_false_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_false_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_false_facts`
    derive the needed `Positive -> Nonzero` bridge from the ordered-field law
    package before consuming the selected raw-counterexample contradiction
    boundary at the standard positive-arithmetic surface.
  - Remaining blocker for an unconditional final theorem: prove the selected
    raw-counterexample contradiction law itself as L2, without assuming
    `raw -> False`, `Not raw`, or the final FLT conclusion. The following
    arithmetic tightening replaces this as the public boundary.
  - `fermat_raw_counterexample_false_from_selected_raw_arithmetic_facts`,
    `fermat_not_raw_counterexample_from_selected_raw_arithmetic_facts`,
    `fermat_positive_integer_solution_false_from_selected_raw_arithmetic_facts`,
    and `fermat_last_theorem_from_selected_raw_arithmetic_facts` derive the
    selected raw-counterexample contradiction from a narrower arithmetic
    contradiction law over the projected raw fields: `Positive x`,
    `Positive y`, `Positive z`, `Nonzero x`, `Nonzero y`, `Nonzero z`,
    `ExponentAtLeastThree n`, and
    `EqualInt (Add (Pow x n) (Pow y n)) (Pow z n)`. The proof uses the existing
    raw data projection theorems and does not assume `raw -> False`, `Not raw`,
    or the final FLT conclusion.
  - `fermat_positive_arithmetic_solution_false_from_selected_raw_arithmetic_facts`,
    `fermat_last_theorem_positive_arithmetic_from_selected_raw_arithmetic_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_selected_raw_arithmetic_facts`
    extend that selected raw-arithmetic boundary to the positive-arithmetic
    solution surface by using an explicit `Positive -> Nonzero` law and the
    certified positive-arithmetic-to-positive-integer conversion.
  - `fermat_positive_arithmetic_solution_false_from_ordered_field_selected_raw_arithmetic_facts`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_selected_raw_arithmetic_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_selected_raw_arithmetic_facts`
    derive the needed `Positive -> Nonzero` bridge from the ordered-field law
    package before consuming the selected raw-arithmetic contradiction
    boundary.
  - Completed L2 standard selected-raw-arithmetic wrapper targets:
    `fermat_raw_counterexample_false_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_selected_raw_arithmetic_facts`
    specialize the selected raw-arithmetic boundary to `Std.Nat`, kernel
    equality, and `FermatStdNatAtLeastThree`, exposing raw, positive-integer,
    and positive-arithmetic surfaces.
  - Completed L2 standard ordered-field selected-raw-arithmetic wrapper targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_arithmetic_facts`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_arithmetic_facts`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_selected_raw_arithmetic_facts`
    derive the needed `Positive -> Nonzero` bridge from the ordered-field law
    package before consuming the selected raw-arithmetic boundary at the
    standard positive-arithmetic surface.
  - Remaining blocker for an unconditional final theorem: prove the selected
    arithmetic contradiction law itself as L2, without assuming the arithmetic
    contradiction, `raw -> False`, `Not raw`, or the final FLT conclusion. The
    following positive-arithmetic tightening replaces this as the public
    boundary.
  - `fermat_raw_counterexample_false_from_selected_positive_arithmetic_facts`,
    `fermat_not_raw_counterexample_from_selected_positive_arithmetic_facts`,
    `fermat_positive_integer_solution_false_from_selected_positive_arithmetic_facts`,
    and `fermat_last_theorem_from_selected_positive_arithmetic_facts` remove
    the redundant `Nonzero x`, `Nonzero y`, and `Nonzero z` premises from the
    public arithmetic contradiction law. The proof derives the raw
    counterexample contradiction using only the raw projections for
    `Positive x`, `Positive y`, `Positive z`, `ExponentAtLeastThree n`, and the
    concrete Fermat equation, ignoring the separate nonzero projections and
    still not assuming the arithmetic contradiction conclusion, `raw -> False`,
    `Not raw`, or the final FLT conclusion.
  - Completed L2 standard selected-positive-arithmetic raw wrapper targets:
    `fermat_raw_counterexample_false_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`
    and
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`
    specialize the selected-positive-arithmetic raw refutation to `Std.Nat`,
    kernel equality, and `FermatStdNatAtLeastThree`. These wrappers expose the
    raw-counterexample surface at the public positive-arithmetic boundary
    without adding a duplicate final-statement theorem or returning the
    selected contradiction law unchanged.
  - Completed L2 selected-positive-arithmetic formula-positive-solution targets:
    `fermat_positive_solution_false_from_selected_positive_arithmetic_facts`,
    `fermat_not_positive_solution_from_selected_positive_arithmetic_facts`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`
    project the positive, exponent, and equation fields out of
    `FermatPositiveSolutionData` before applying the selected
    positive-arithmetic contradiction law, in generic and standard
    `Std.Nat`/kernel-equality forms.
  - Completed L2 positive-solution-to-positive-arithmetic conversion targets:
    `fermat_positive_arithmetic_solution_data_from_positive_solution_data`
    and
    `fermat_positive_arithmetic_solution_data_std_nat_kernel_eq_at_least_three_from_positive_solution_data`
    rebuild `FermatPositiveArithmeticSolutionData` from a
    `FermatPositiveSolutionData` record by projecting the positive, exponent,
    and equation fields, in generic and standard `Std.Nat`/kernel-equality
    forms.
  - Completed L2 positive-arithmetic-negation to positive-solution-negation
    bridge targets:
    `fermat_positive_solution_false_from_positive_arithmetic_solution_data_negation`,
    `fermat_not_positive_solution_from_positive_arithmetic_solution_data_negation`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_positive_arithmetic_solution_data_negation`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_positive_arithmetic_solution_data_negation`
    transport a positive-arithmetic no-solution theorem back to the
    formula-positive-solution surface by converting the supplied
    `FermatPositiveSolutionData` into `FermatPositiveArithmeticSolutionData`.
  - Completed L2 positive-arithmetic-negation to positive-integer-negation
    bridge targets:
    `fermat_positive_integer_solution_false_from_positive_arithmetic_solution_data_negation`,
    `fermat_not_positive_integer_solution_from_positive_arithmetic_solution_data_negation`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_positive_arithmetic_solution_data_negation`,
    and
    `fermat_not_positive_integer_solution_std_nat_kernel_eq_at_least_three_from_positive_arithmetic_solution_data_negation`
    transport a positive-arithmetic no-solution theorem back to the
    positive-integer surface by converting the supplied
    `FermatPositiveIntegerSolutionData` into
    `FermatPositiveArithmeticSolutionData`.
  - `FermatPositiveArithmeticSolutionData`,
    `fermat_positive_arithmetic_solution_data_intro`, the certified
    `fermat_positive_arithmetic_solution_positive_*`,
    `fermat_positive_arithmetic_solution_exponent_at_least_three`, and
    `fermat_positive_arithmetic_solution_equation` projections, plus
    `fermat_positive_arithmetic_solution_data_from_positive_integer_solution`,
    `fermat_positive_arithmetic_solution_false_from_selected_positive_arithmetic_facts`,
    and `fermat_last_theorem_from_selected_positive_arithmetic_solution_facts`
    move the public final-statement data shape to the positive-arithmetic
    fields only: `Positive x`, `Positive y`, `Positive z`,
    `ExponentAtLeastThree n`, and the concrete Fermat equation. The public
    `fermat_last_theorem` now concludes
    `Not (FermatPositiveArithmeticSolutionData ...)` and no longer quantifies a
    separate `Nonzero` predicate; the older positive-integer contradiction
    theorem remains as a compatibility theorem via the certified conversion.
  - `fermat_positive_integer_solution_data_from_positive_arithmetic_solution`
    and `fermat_last_theorem_from_positive_integer_refutation` add the reverse
    bridge needed to connect the positive-arithmetic public shape back to the
    existing positive-integer route layer. They require an explicit
    `positive_nonzero_law : forall value, Positive value -> Nonzero value`,
    then construct the missing `Nonzero x`, `Nonzero y`, and `Nonzero z`
    fields from the positive projections instead of assuming a completed FLT
    contradiction. This is prerequisite reduction for the Frey/Wiles/Ribet
    route, not an unconditional FLT proof.
  - Completed L2 positive-arithmetic/positive-integer equivalence targets:
    `fermat_positive_arithmetic_solution_data_iff_positive_integer_solution_data`,
    `fermat_positive_arithmetic_solution_data_iff_positive_integer_solution_data_from_ordered_field_bridge`,
    `fermat_positive_arithmetic_solution_data_iff_positive_integer_solution_data_std_nat_kernel_eq_at_least_three`,
    and
    `fermat_positive_arithmetic_solution_data_iff_positive_integer_solution_data_std_nat_kernel_eq_at_least_three_from_ordered_field_bridge`
    construct the `Iff` between the public positive-arithmetic surface and the
    older positive-integer route surface from the certified conversions in both
    directions. The explicit variant requires `Positive -> Nonzero`; the
    ordered-field variants derive that law from the ordered-field bridge.
  - Completed L2 positive-integer-negation to positive-arithmetic-negation
    bridge targets:
    `fermat_positive_arithmetic_solution_false_from_positive_integer_solution_data_negation`,
    `fermat_not_positive_arithmetic_solution_from_positive_integer_solution_data_negation`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_positive_integer_solution_data_negation`,
    `fermat_not_positive_arithmetic_solution_std_nat_kernel_eq_at_least_three_from_positive_integer_solution_data_negation`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_positive_integer_solution_data_negation`,
    `fermat_not_positive_arithmetic_solution_from_ordered_field_positive_integer_solution_data_negation`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_solution_data_negation`,
    and
    `fermat_not_positive_arithmetic_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_solution_data_negation`
    transport a positive-integer no-solution theorem to the public
    positive-arithmetic surface by applying the certified `Iff` forward
    direction. The ordered-field variants derive the required
    `Positive -> Nonzero` bridge from the ordered-field law package.
  - `fermat_positive_nonzero_law_from_ordered_field_bridge`,
    `fermat_positive_integer_solution_data_from_ordered_field_positive_arithmetic_solution`,
    `fermat_last_theorem_from_ordered_field_positive_integer_refutation`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_positive_integer_refutation`
    split that `Positive -> Nonzero` prerequisite through the existing
    ordered-field bridge theorem `ordered_field_nonzero_of_positive`. The new
    L2 route requires an ordered-field law package, an
    `OrderedFieldFieldBridgeArgs` package, and explicit interpretation maps
    from the FLT `Positive` predicate to ordered-field positivity and from
    ordered-field nonzero back to the FLT `Nonzero` predicate. It also derives
    the selected positive-arithmetic contradiction law from a positive-integer
    refutation under those ordered-field bridge inputs.
  - `fermat_last_theorem_positive_arithmetic_from_global_raw_elimination_provider`
    composes the new reverse bridge with the existing global raw elimination
    provider theorem. Under the explicit Frey-model, modularity-lifting,
    semistable-modularity, level-lowering/Ribet, no-counterexample, primitive
    normalization, and `Positive -> Nonzero` provider families, it concludes the
    public positive-arithmetic `Not (FermatPositiveArithmeticSolutionData ...)`
    shape directly. This replaces the short selected-positive-arithmetic-law
    boundary with the structured route-data boundary, while the concrete L2
    construction of those provider families remains open.
  - `fermat_last_theorem_positive_arithmetic_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_global_raw_elimination_provider`
    remove the explicit primitive `Nonzero` provider families from that
    boundary. They synthesize `nonzero_primitive_x/y/z_provider` by applying the
    ordered-field-derived `Positive -> Nonzero` bridge to the existing
    `positive_primitive_x/y/z_provider` witnesses, then reuse the global raw
    elimination route. The remaining nonzero work is therefore the concrete
    ordered-field interpretation/bridge data, not three separate primitive
    nonzero provider families.
  - `fermat_positive_integer_solution_false_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_from_ordered_field_global_raw_elimination_provider`
    carry the same ordered-field/global-raw boundary back to the older
    positive-integer solution surface. They first derive the public
    positive-arithmetic negation and then eliminate the positive-arithmetic
    projection of a positive-integer solution, so the positive-integer
    contradiction no longer needs separate primitive `Nonzero` provider
    families at this boundary.
  - `fermat_selected_positive_arithmetic_contradiction_law_from_global_raw_elimination_provider`
    derives the short selected positive-arithmetic contradiction law from the
    same structured route-data boundary. It constructs
    `FermatPositiveArithmeticSolutionData` from the supplied positive,
    exponent, and equation witnesses, then eliminates it with the global
    provider-derived negation. This means the selected law can now be supplied
    by L2 route data rather than being the only public boundary.
  - Completed L2 selected-route positive-arithmetic bridge targets:
    `fermat_positive_arithmetic_solution_false_from_selected_*_facts`,
    `fermat_last_theorem_positive_arithmetic_from_selected_*_facts`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_selected_*_facts`
    are now derived for the selected direct level-two, dependency-map
    contradiction, route-facts contradiction, selected level-two, selected
    no-raw, and selected raw-false boundaries. These targets convert a
    positive-arithmetic solution to a positive-integer solution using an
    explicit `Positive -> Nonzero` law, then consume the certified
    selected positive-integer refutations. They must not merely return a
    supplied selected positive-arithmetic law.
  - Completed L2 ordered-field selected-route bridge targets:
    the corresponding `*_from_ordered_field_selected_*_facts` theorems are now
    derived for those same six selected boundaries. These remove the explicit
    `Positive -> Nonzero` premise by deriving it from
    `fermat_positive_nonzero_law_from_ordered_field_bridge`, the ordered-field
    law package, the field bridge package, and the interpretation maps from the
    FLT `Positive`/`Nonzero` predicates to the ordered-field predicates.
  - Remaining blockers for an unconditional final theorem: construct the
    ordered-field bridge/interpretation data that yields `Positive -> Nonzero`
    for the concrete integer positivity predicate, plus the Frey-model,
    modularity-lifting, semistable-modularity, level-lowering/Ribet,
    no-counterexample, primitive normalization, and global raw elimination
    provider families concretely as L2 theorems, without assuming the
    positive-arithmetic contradiction, `raw -> False`, `Not raw`, or the final
    FLT conclusion.
  - `fermat_last_theorem_std_nat_exponent` specializes the positive-arithmetic
    final theorem to the repository's vendor `Std.Nat.Basic` exponent carrier,
    while leaving `Int`, `Pow`, `Add`, `EqualInt`, `Positive`, and
    `ExponentAtLeastThree` explicit. This is an L2 specialization step, not an
    unconditional FLT proof. The remaining standard-arithmetic prerequisites are
    a concrete integer carrier, concrete addition and exponentiation, equality
    and positivity predicates tied to that carrier, an `n >= 3` predicate over
    standard `Nat`, and L2 arithmetic/Frey-Wiles-Ribet proofs deriving the
    selected positive-arithmetic contradiction law from those concrete
    definitions.
  - `FermatStdNatThreePlus`, `FermatStdNatAtLeastThree`,
    `fermat_std_nat_at_least_three_intro`,
    `fermat_std_nat_at_least_three_elim`, and
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three` define a
    certificate-backed `n >= 3` proposition over the vendor `Std.Nat.Basic`
    constructors and specialize the final theorem to kernel equality `@Eq Int`
    plus that concrete exponent predicate. This reduces two more abstract
    public parameters (`EqualInt` and `ExponentAtLeastThree`) without claiming
    an unconditional FLT proof.
  - `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three`
    exposes the same selected positive-arithmetic contradiction law as a
    pointwise `False` eliminator for the standard positive-arithmetic solution
    surface.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three`
    and `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three`
    transport that selected arithmetic contradiction to the standard
    positive-integer solution surface by projecting positive-integer solution
    data to positive-arithmetic data, without assuming the final negation.
  - `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_positive_integer_refutation`
    specializes the positive-integer-refutation bridge to the same `Std.Nat`
    exponent carrier, kernel equality, and `FermatStdNatAtLeastThree` predicate.
    The remaining standard-arithmetic prerequisite at this boundary is a
    concrete L2 `positive_nonzero_law` and a positive-integer refutation coming
    from the route-data layer.
  - `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    removes that explicit `positive_nonzero_law` from the same standard
    `Nat`/kernel-`Eq` boundary by synthesizing it with the ordered-field bridge
    theorem. The remaining prerequisite at this boundary is therefore the
    positive-integer refutation itself, plus concrete ordered-field
    interpretation data for the chosen integer carrier.
  - `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    turn that standard ordered-field/positive-integer-refutation boundary into
    the pointwise `False` eliminator and short selected positive-arithmetic
    contradiction law. These are L2 wrappers over the certified negation, not
    new assumptions of the final contradiction.
  - `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    specialize the explicit global-raw route boundary to standard `Nat`, kernel
    equality, and `FermatStdNatAtLeastThree`, while still requiring explicit
    nonzero primitive providers plus the `Positive -> Nonzero` law needed to
    convert a positive-arithmetic solution into a positive-integer solution.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    expose the same standard explicit global-raw route at the positive-integer
    solution surface. This pair does not need the extra `Positive -> Nonzero`
    law because the positive-integer record already contains nonzero evidence.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    packages the standard explicit global-raw boundary as a reusable
    `FermatPositiveIntegerGlobalEliminationData` closure.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    constructs the same standard closure with primitive nonzero provider
    evidence synthesized from the ordered-field bridge.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    consume the standard `FermatPositiveIntegerGlobalEliminationData` closure
    at the positive-integer solution surface.
  - `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    transport the same closure to the positive-arithmetic surface using an
    explicit `Positive -> Nonzero` law.
  - `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`
    derive that bridge law from ordered-field data before eliminating the
    positive-arithmetic solution.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    opens the more general `FermatGlobalEliminationData` closure at the
    standard `Nat`/kernel-`Eq` boundary and exposes it as reusable
    `FermatPositiveIntegerGlobalEliminationData`.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    specialize that same global closure to the formula-level
    `FermatPositiveSolutionData` surface before moving on to the
    positive-integer wrapper.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    consume that global closure directly at the positive-integer surface.
  - The positive-arithmetic and ordered-field variants ending in
    `_from_global_elimination_data` transport the same closure through either
    an explicit `Positive -> Nonzero` law or the ordered-field bridge, without
    constructing the still-missing concrete global provider families.
  - `fermat_global_elimination_data_from_global_normalization_laws_builds_curve_and_route_laws`
    constructs the formula-specialized `FermatGlobalEliminationData` closure at
    the generic raw-indexed global-normalization boundary by building
    `FermatGlobalRawRefutationData` from raw-realization evidence and the
    no-raw-counterexample law.
  - `fermat_positive_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`
    and
    `fermat_global_not_positive_solution_from_global_normalization_laws_builds_curve_and_route_laws`
    consume that closure directly at the generic `FermatPositiveSolutionData`
    surface.
  - `fermat_positive_arithmetic_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_from_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_global_normalization_laws_builds_curve_and_route_laws`
    transport the same generic closure through an explicit `Positive ->
    Nonzero` law to the positive-arithmetic final-statement surface and the
    selected arithmetic contradiction law.
  - `fermat_positive_integer_global_elimination_data_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_integer_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_global_not_positive_integer_solution_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_global_elimination_data_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_global_not_positive_solution_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    derive both the primitive `Nonzero` providers and the final `Positive ->
    Nonzero` law from the generic ordered-field bridge before consuming the same
    decomposed global-normalization closure.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`
    constructs the standard positive-integer closure from primitive
    normalization providers, Frey-model component laws, direct Wiles/Ribet
    route laws, raw-realization evidence, and the no-raw-counterexample law.
    This removes the monolithic global raw-elimination provider from that
    standard closure boundary.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`
    exposes the formula-specialized `FermatGlobalEliminationData` closure from
    the same decomposed global-normalization boundary.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`
    consume those decomposed standard closures at the formula-level
    `FermatPositiveSolutionData`, positive-integer, and positive-arithmetic
    surfaces.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    constructs the same decomposed standard positive-integer closure after
    deriving the primitive `Nonzero` provider families from the ordered-field
    bridge and the positive primitive provider families.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    exposes the ordered-field version of the formula-specialized
    `FermatGlobalEliminationData` closure.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    consume that ordered-field-derived decomposed closure at the
    formula-level `FermatPositiveSolutionData`, positive-integer, and
    positive-arithmetic surfaces.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    constructs the standard positive-integer global-elimination closure from
    structured Wiles/Ribet route-data inputs and
    `FermatGlobalRawRefutationData`, replacing a monolithic global
    raw-elimination provider at that boundary.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    exposes the more general formula-specialized `FermatGlobalEliminationData`
    closure at the same standard route-data/raw-refutation boundary, before
    transporting it to the positive-integer closure.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    consume that formula-level closure directly at the
    `FermatPositiveSolutionData` surface.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    consume that route-data/raw-refutation closure at the positive-integer and
    positive-arithmetic surfaces.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    opens the `FermatGlobalRawRefutationData` compatibility package at the
    standard route-data boundary by constructing it from explicit
    `realizes_raw_provider` and `no_raw_counterexample_law` components before
    reusing the certified route-data/global-raw-refutation closure.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    exposes the same formula-specialized `FermatGlobalEliminationData`
    closure directly from those explicit raw-refutation components.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    consume that explicit-component closure at the formula-level positive
    solution surface.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    consume that explicit raw-realization/no-raw-law route boundary at the same
    standard positive-integer and positive-arithmetic surfaces.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    derives the primitive `Nonzero` provider families from the ordered-field
    bridge and uses the same route-data/raw-refutation closure.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    exposes the corresponding ordered-field-derived
    `FermatGlobalEliminationData` closure before the positive-integer
    transport.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    consume that ordered-field route-data closure directly at
    `FermatPositiveSolutionData`.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    expose the ordered-field route-data/raw-refutation boundary at the same
    positive-integer and positive-arithmetic surfaces.
  - `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    derives the primitive `Nonzero` provider families from the ordered-field
    bridge while constructing the raw-refutation compatibility package from
    explicit raw-realization/no-raw-law components.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    exposes that ordered-field-derived formula-specialized global closure
    before the concrete positive-integer closure is consumed.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    consume that ordered-field explicit-component closure at the same
    formula-level surface.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    expose the ordered-field component-level route boundary at the same
    positive-integer and positive-arithmetic surfaces.
  - `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    specialize the ordered-field/global-raw route-data boundary to the standard
    `Nat` exponent carrier, kernel equality, and `FermatStdNatAtLeastThree`.
    They remove the abstract `NatCarrier`, `EqualInt`, and
    `ExponentAtLeastThree` parameters from the route boundary, but still require
    concrete ordered-field interpretation, Frey, modularity-lifting,
    semistable-modularity, level-lowering/Ribet, no-counterexample, primitive
    normalization, and global raw-elimination provider data. This is L2
    specialization of the conditional route, not an unconditional FLT proof.
  - `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    transport that standard route to the positive-integer solution surface. They
    do not add a new route assumption; they convert a positive-integer solution
    to `FermatPositiveArithmeticSolutionData` and reuse the certified standard
    ordered-field/global-raw negation.
  - `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    and
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    expose the formula-specialized `FermatGlobalEliminationData` closure at the
    corresponding standard global-raw-provider boundaries.
  - `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    consume those closures at the formula-level `FermatPositiveSolutionData`
    surface before the positive-integer and positive-arithmetic transports.
  - Completed L2 generic formula-solution closure consumer targets:
    `fermat_positive_solution_false_from_global_elimination_data` and
    `fermat_not_positive_solution_from_global_elimination_data` consume
    the already certified formula-specialized `FermatGlobalEliminationData`
    closure at the `FermatPositiveSolutionData` surface. These wrappers
    eliminate a concrete `FermatPositiveSolutionData` argument through the
    closure and `not_intro`; they do not introduce a new positive-solution
    contradiction assumption or merely return a supplied law. After these are
    proved, the route-data / Frey-model-law boundaries can call this smaller
    closure consumer instead of duplicating the closure-elimination proof.
  - Completed L2 generic global-raw-elimination-provider formula-solution
    consumer targets:
    `fermat_positive_solution_false_from_global_raw_elimination_provider`
    and
    `fermat_not_positive_solution_from_global_raw_elimination_provider`
    build the formula-specialized `FermatGlobalEliminationData` closure from
    the explicit solution-indexed raw-elimination provider and eliminate a
    concrete `FermatPositiveSolutionData` argument, without requiring
    route-data or Frey-model-law context. These are the provider-level
    formula-solution consumers used before adding route-data or Frey-model-law
    boundary wrappers.
  - Completed L2 generic route-data formula-solution consumer targets:
    `fermat_positive_solution_false_from_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_from_route_data_and_global_raw_refutation_data`
    construct the route-data/global-raw-refutation
    `FermatGlobalEliminationData` closure and consume it through the generic
    formula-solution closure consumer. These wrappers do not assume a
    positive-solution contradiction law directly.
  - Completed L2 generic route-data/global-raw-elimination-provider
    formula-solution consumer targets:
    `fermat_positive_solution_false_from_route_data_and_global_raw_elimination_provider`
    and
    `fermat_not_positive_solution_from_route_data_and_global_raw_elimination_provider`
    consume the explicit global raw elimination provider at the route-data
    boundary by building the formula-specialized
    `FermatGlobalEliminationData` closure with
    `fermat_global_elimination_data_from_not_raw_provider` and eliminating the
    concrete `FermatPositiveSolutionData` argument. These wrappers keep the
    route-data context explicit and do not assume a positive-solution
    contradiction law directly.
  - Completed L2 direct Frey-model-law/global-raw-elimination-provider
    formula-solution consumer targets:
    `fermat_positive_solution_false_from_frey_model_laws_and_global_raw_elimination_provider`
    and
    `fermat_not_positive_solution_from_frey_model_laws_and_global_raw_elimination_provider`
    expose the provider-level formula-solution contradiction at the explicit
    Frey-model/route-law boundary by constructing the formula-specialized
    `FermatGlobalEliminationData` closure and eliminating the concrete
    `FermatPositiveSolutionData` argument, without adding a positive-solution
    contradiction law.
  - Completed L2 direct Frey-model-law/global-raw-elimination-provider
    positive-arithmetic wrapper targets:
    `fermat_positive_arithmetic_solution_false_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_last_theorem_positive_arithmetic_from_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_frey_model_laws_and_global_raw_elimination_provider`
    move the same boundary to the public positive-arithmetic solution shape by
    converting a positive-arithmetic solution to the positive-integer solution
    record under an explicit `Positive -> Nonzero` law and then eliminating it
    through the certified Frey/provider theorem.
  - Completed L2 standard direct Frey-model-law/global-raw-elimination-provider
    targets:
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_frey_model_laws_and_global_raw_elimination_provider`
    specialize the direct Frey/model-law plus global raw-elimination-provider
    boundary to `Std.Nat.Basic`, kernel equality, and
    `FermatStdNatAtLeastThree`, while keeping the explicit primitive
    `Nonzero` providers and the public `Positive -> Nonzero` law visible
    instead of deriving them from an ordered-field bridge.
  - Completed L2 direct Frey-model-law/raw-refutation component targets:
    `fermat_positive_arithmetic_solution_false_from_frey_model_laws`,
    `fermat_last_theorem_positive_arithmetic_from_frey_model_laws`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_frey_model_laws`
    move the positive-arithmetic public surface to the decomposed Frey/model
    route that consumes explicit `realizes_raw_provider` and
    `no_raw_counterexample_law` inputs instead of a monolithic global
    raw-elimination provider.
  - Completed L2 standard direct Frey-model-law/raw-refutation component
    targets:
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_frey_model_laws`
    specialize that decomposed Frey/model raw-refutation route to
    `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree` while
    keeping the explicit primitive `Nonzero`, raw-realization, and no-raw law
    inputs visible.
  - Completed L2 ordered-field Frey-model-law/raw-refutation component
    targets:
    `fermat_positive_arithmetic_solution_false_from_ordered_field_frey_model_laws`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_frey_model_laws`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_frey_model_laws`,
    `fermat_positive_integer_solution_false_from_ordered_field_frey_model_laws`,
    and
    `fermat_last_theorem_positive_integer_from_ordered_field_frey_model_laws`
    derive the primitive `Nonzero` provider families and the public
    `Positive -> Nonzero` law from the ordered-field bridge before consuming
    that decomposed Frey/model route.
  - Completed L2 standard ordered-field Frey-model-law/raw-refutation
    component targets:
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`
    specialize the ordered-field decomposed Frey/model route to standard
    `Nat`/kernel equality while deriving primitive `Nonzero` provider families
    and the public positive-to-nonzero law from the ordered-field bridge.
  - Completed L2 ordered-field direct Frey-model-law/global-raw-elimination-provider
    wrapper targets:
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_solution_false_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_last_theorem_positive_integer_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`
    remove the explicit `Positive -> Nonzero` and primitive-nonzero provider
    inputs at the Frey/provider boundary by deriving them from the ordered-field
    bridge before reusing the certified Frey/provider eliminators.
  - Completed L2 standard ordered-field direct Frey-model-law/global-raw-elimination-provider
    wrapper targets:
    `fermat_last_theorem_positive_arithmetic_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`
    specialize that ordered-field Frey/provider boundary to `Std.Nat.Basic`,
    kernel equality, and `FermatStdNatAtLeastThree`, while still deriving
    primitive `Nonzero` provider families from the ordered-field bridge.
  - Completed L2 ordered-field Frey/provider formula-closure targets:
    `fermat_global_elimination_data_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_global_elimination_data_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_solution_false_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_not_positive_solution_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws_and_global_raw_elimination_provider`
    expose the ordered-field Frey/provider boundary as formula-specialized
    global-elimination and positive-solution consumers in both generic and
    standard `Nat`/kernel-equality forms.
  - Completed L2 Frey-model-law/raw-refutation formula-closure targets:
    `fermat_global_elimination_data_from_frey_model_laws`,
    `fermat_positive_integer_global_elimination_data_from_frey_model_laws`,
    `fermat_positive_solution_false_from_frey_model_laws`,
    `fermat_not_positive_solution_from_frey_model_laws`,
    `fermat_global_elimination_data_from_ordered_field_frey_model_laws`,
    `fermat_positive_integer_global_elimination_data_from_ordered_field_frey_model_laws`,
    `fermat_positive_solution_false_from_ordered_field_frey_model_laws`,
    `fermat_not_positive_solution_from_ordered_field_frey_model_laws`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_frey_model_laws`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_laws`
    expose the decomposed Frey/model raw-refutation route as
    formula-specialized global-elimination data and positive-solution
    consumers, in generic, ordered-field, standard `Nat`, and standard
    ordered-field forms.
  - Completed L2 route-data/raw-realization formula-closure targets:
    `fermat_global_elimination_data_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_integer_global_elimination_data_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_solution_false_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_not_positive_solution_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_global_elimination_data_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_integer_global_elimination_data_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_solution_false_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_not_positive_solution_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    expose the generic route-data boundary with explicit
    `realizes_raw_provider` and `no_raw_counterexample_law` components as
    formula-specialized global-elimination data and positive-solution
    consumers, matching the already certified standard `Nat` route without
    introducing a monolithic global raw-elimination provider.
  - Completed L2 route-data/raw-realization public-surface wrapper targets:
    `fermat_positive_integer_solution_false_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_arithmetic_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_integer_solution_false_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_integer_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`.
  - Completed L2 decomposed-provider positive-arithmetic public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_raw_primitive_frey_route_provider`,
    `fermat_last_theorem_positive_arithmetic_from_raw_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_raw_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_from_primitive_frey_route_provider`,
    `fermat_last_theorem_positive_arithmetic_from_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_from_frey_model_and_route_data`,
    `fermat_last_theorem_positive_arithmetic_from_frey_model_and_route_data`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_frey_model_and_route_data`,
    `fermat_positive_arithmetic_solution_false_from_primitive_normalization_provider`,
    `fermat_last_theorem_positive_arithmetic_from_primitive_normalization_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_primitive_normalization_provider`.
  - Completed L2 ordered-field decomposed-provider public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_ordered_field_raw_primitive_frey_route_provider`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_raw_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_raw_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_primitive_frey_route_provider`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_frey_model_and_route_data`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_frey_model_and_route_data`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_frey_model_and_route_data`,
    `fermat_positive_arithmetic_solution_false_from_ordered_field_primitive_normalization_provider`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_primitive_normalization_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_primitive_normalization_provider`.
  - Completed L2 standard decomposed-provider public-surface targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`
    specialize the four decomposed provider boundaries to `Std.Nat`, kernel
    equality, and `FermatStdNatAtLeastThree` while keeping the explicit
    `Positive -> Nonzero` bridge law visible instead of deriving it from
    ordered-field interpretation data.
  - Completed L2 standard ordered-field raw-primitive-provider public-surface targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`.
  - Completed L2 formula-level raw-primitive-provider closure targets for this batch:
    `fermat_global_elimination_data_from_raw_primitive_frey_route_provider`,
    `fermat_positive_solution_false_from_raw_primitive_frey_route_provider`,
    `fermat_not_positive_solution_from_raw_primitive_frey_route_provider`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_raw_primitive_frey_route_provider`
    construct formula-specialized global-elimination data from the raw
    primitive Frey-route provider by deriving the raw-elimination provider and
    then consuming that package. Ordered-field-only formula variants are not
    added here because the formula-level positive-solution data already
    carries explicit `Nonzero` witnesses; the ordered-field bridge remains
    meaningful on the positive-arithmetic public surface above.
  - Completed L2 formula-level primitive-provider closure targets for this batch:
    `fermat_global_elimination_data_from_primitive_frey_route_provider`,
    `fermat_positive_solution_false_from_primitive_frey_route_provider`,
    `fermat_not_positive_solution_from_primitive_frey_route_provider`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_primitive_frey_route_provider`
    expose the decomposed primitive-normalization plus primitive-Frey-route
    provider boundary as formula-specialized global-elimination data and
    positive-solution consumers in generic and standard `Nat` forms.
  - Completed L2 formula-level Frey-model/route-data closure targets for this batch:
    `fermat_global_elimination_data_from_frey_model_and_route_data`,
    `fermat_positive_solution_false_from_frey_model_and_route_data`,
    `fermat_not_positive_solution_from_frey_model_and_route_data`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_frey_model_and_route_data`
    construct the primitive-Frey-route provider from primitive normalization,
    primitive realization, Frey-model data, and Wiles/Ribet route data before
    exposing the same formula-specialized global-elimination and
    positive-solution contradiction surfaces.
  - Completed L2 formula-level primitive-normalization closure targets for this batch:
    `fermat_global_elimination_data_from_primitive_normalization_provider`,
    `fermat_positive_solution_false_from_primitive_normalization_provider`,
    `fermat_not_positive_solution_from_primitive_normalization_provider`,
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_primitive_normalization_provider`
    build the Frey-model provider from its component laws and the route-data
    package from Wiles/Ribet route laws, then reuse the Frey-model/route-data
    formula-specialized closure.
  - Completed L2 standard ordered-field decomposed-provider public-surface targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_frey_route_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_frey_route_provider`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_frey_route_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_and_route_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_and_route_data`,
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_frey_model_and_route_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_normalization_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_normalization_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_primitive_normalization_provider`.
  - `fermat_no_raw_counterexample_from_positive_solution_elimination_provider`
  - `fermat_no_positive_solution_from_positive_solution_elimination_provider`
  - `fermat_global_no_positive_solution_from_global_elimination_provider`
    lifts the pointwise positive-solution eliminator to a universally
    quantified no-positive-solution theorem under explicit selector,
    projection, translation, and raw-elimination-provider families.
  - `fermat_global_false_from_global_elimination_provider` specializes the
    global no-positive-solution theorem to `False`, producing the standard
    negated positive-solution shape under the same explicit global provider
    families and a certified positive-solution-to-raw-contradiction law.
  - `fermat_global_not_positive_solution_from_global_elimination_provider`
    wraps the `False`-valued theorem in the existing `Not` API, yielding a
    final-theorem-shaped global negation under the same provider families.
  - `fermat_global_not_positive_solution_from_positive_solution_data_provider`
    specializes the global negation theorem to `FermatPositiveSolutionData`,
    using the certified concrete projections instead of accepting separate
    surface projection laws.
  - `fermat_global_not_positive_integer_solution_from_provider` specializes
    the global provider route to the concrete final-statement syntax record.
  - `FermatGlobalEliminationData` packages the global selectors, projection
    laws, positive-solution contradiction law, and raw-elimination provider as
    a single explicit closure.
  - `fermat_global_elimination_data_intro` builds that closure from its
    certified components.
  - `fermat_global_false_from_global_elimination_data` extracts the explicit
    `False` contradiction from a `FermatGlobalEliminationData` closure for a
    concrete positive solution, mirroring the positive-integer global
    elimination contradiction theorem at the surface-solution layer.
  - `fermat_global_not_positive_solution_from_global_elimination_data` derives
    the `Not (FermatPositiveSolution x y z n)` theorem from that explicit
    closure-data contradiction.
- Completed L2 target in the final-statement wrapper pass:
  - `fermat_last_theorem` was emitted without keeping the old long
    final-statement provider theorem name; the proof constructs the
    raw-elimination provider from the raw primitive Frey route provider and
    raw realization/no-raw laws, then constructs the formula-specialized global
    closure, the concrete positive-integer closure, and applies the direct
    positive-integer wrapper.
- Completed L2 public selected-law removal target:
  - Route the public `fermat_last_theorem` name through
    `fermat_last_theorem_positive_arithmetic_from_minimal_modularity_lifting_core_bridge_free`
    so it no longer takes `selected_positive_arithmetic_contradiction_law` as a
    premise. The older selected-law standard wrappers remain explicit
    compatibility surfaces and call
    `fermat_last_theorem_from_selected_positive_arithmetic_solution_facts`
    directly instead of depending on the public final name.
- Completed L2 public positive-nonzero removal target:
  - Route the public `fermat_last_theorem` name through the ordered-field
    minimal-modularity/lifting-core bridge-free closure so it no longer takes
    `positive_nonzero_law` as a direct premise. The proof should derive the
    nonzero bridge from ordered-field positivity data and use the concrete
    positive-arithmetic contradiction theorem directly, not just alias the
    longer ordered-field wrapper name.
- Completed L2 generic public pointwise contradiction target:
  - Added `fermat_positive_arithmetic_solution_false` as the pointwise `False`
    eliminator behind public `fermat_last_theorem`, routed through the
    ordered-field minimal-modularity/lifting-core bridge-free closure so the
    public positive-arithmetic surface has an explicit L2 contradiction theorem
    without `selected_positive_arithmetic_contradiction_law` or
    `positive_nonzero_law` direct premises.
- Completed L2 standard public surface target:
  - Route `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three`
    and `fermat_last_theorem_std_nat_kernel_eq_at_least_three` through the
    ordered-field minimal-modularity/lifting-core bridge-free closure so the
    standard `Std.Nat`/kernel-equality public surface no longer requires
    `selected_positive_arithmetic_contradiction_law` or a direct
    `positive_nonzero_law` premise.
- Completed L2 positive-integer public surface target:
  - Route `fermat_positive_integer_solution_false`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three`,
    and `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three`
    through the ordered-field minimal-modularity/lifting-core bridge-free
    closure so the positive-integer public surface no longer requires
    `selected_positive_arithmetic_contradiction_law` or a direct
    `positive_nonzero_law` premise.
- Completed L2 positive-integer public negation target:
  - Added `fermat_last_theorem_positive_integer` as the generic public negation
    behind `fermat_positive_integer_solution_false`, routed through the same
    ordered-field minimal-modularity/lifting-core bridge-free closure.
- Completed L2 formula-positive-solution public surface target:
  - Added `fermat_positive_solution_false` and
    `fermat_last_theorem_positive_solution` as bare public formula-specialized
    positive-solution surfaces, routed through the ordered-field
    minimal-modularity/lifting-core bridge-free closure.
- Completed L2 `Std.Nat` exponent public surface target:
  - Routed `fermat_last_theorem_std_nat_exponent` through the ordered-field
    minimal-modularity/lifting-core bridge-free closure while keeping
    `EqualInt` and `ExponentAtLeastThree` explicit, so this `Std.Nat` exponent
    specialization no longer requires
    `selected_positive_arithmetic_contradiction_law`.
- Completed L2 `Std.Nat` exponent pointwise contradiction target:
  - Added `fermat_positive_arithmetic_solution_false_std_nat_exponent` as the
    pointwise `False` eliminator behind
    `fermat_last_theorem_std_nat_exponent`, using the same ordered-field
    minimal-modularity/lifting-core bridge-free closure while keeping
    `EqualInt` and `ExponentAtLeastThree` explicit.
- Completed L2 generic public provider-decomposition route target:
  - Routed `fermat_positive_arithmetic_solution_false`,
    `fermat_last_theorem`, and the ordered-field minimal-modularity
    positive-arithmetic wrappers through the earlier Frey-model-law surface
    `fermat_positive_arithmetic_solution_false_from_ordered_field_frey_model_laws`
    / `fermat_last_theorem_positive_arithmetic_from_ordered_field_frey_model_laws`.
  - The wrapper boundary now derives the semistable-modularity, no-bridge,
    Ribet level-lowering, level-two-contradiction, and no-counterexample route
    laws from the level-lowering/minimal-modularity core laws before applying
    the primitive-normalization/Frey-model/raw-refutation route closure.
- Completed L2 raw-route projection provider target:
  - Re-proved `fermat_global_elimination_data_from_raw_primitive_frey_route_provider`
    so that it projects a raw-primitive-Frey-route provider into explicit
    primitive-normalization and primitive-Frey-route provider families before
    applying the primitive-provider raw-elimination closure.
  - Re-routed the formula-positive-solution raw-route wrappers through this
    directly re-proved wrapper and removed the now-alias-only
    `_via_primitive_providers` global-elimination wrappers.
- Completed L2 raw-elimination primitive-provider target:
  - Added `fermat_raw_elimination_provider_from_primitive_frey_route_provider`,
    which builds `FermatRawCounterexampleEliminationData` directly from
    explicit primitive-normalization and primitive-Frey-route provider
    families plus raw-realization/no-raw laws.
  - Re-routed the concrete positive-integer raw-route contradiction wrapper
    through projected primitive providers, and re-routed the primitive-provider
    global-elimination closure through the new raw-elimination provider instead
    of reconstructing a raw-primitive-Frey-route provider at that boundary.
  - Re-routed
    `fermat_positive_integer_solution_false_from_primitive_frey_route_provider`
    through the formula-specialized primitive-provider raw-elimination closure
    instead of first constructing a raw-primitive-Frey-route provider.
  - Re-proved
    `fermat_raw_elimination_provider_from_raw_primitive_frey_route_provider`
    itself through the primitive-provider raw-elimination closure by projecting
    the raw route provider to primitive-normalization and primitive-Frey-route
    provider families.
- Completed L2 raw-refutation primitive-provider reroute target:
  - Added `fermat_primitive_frey_route_provider_from_frey_model_laws` and
    `fermat_primitive_frey_route_provider_from_route_data` as explicit
    primitive-Frey-route provider constructors.
  - Re-routed `fermat_global_raw_elimination_provider_from_frey_model_laws`
    and
    `fermat_global_raw_elimination_provider_from_route_data_and_global_raw_refutation_data`
    through explicit primitive-normalization and primitive-Frey-route provider
    construction, so those raw-refutation wrappers no longer construct a
    raw-primitive-Frey-route provider before applying raw elimination.
  - Re-proved `fermat_raw_primitive_frey_route_provider_from_frey_model_laws`
    and `fermat_raw_primitive_frey_route_provider_from_route_data` by
    constructing `FermatRawPrimitiveFreyRouteData` directly from the explicit
    primitive-normalization provider and the corresponding primitive-Frey-route
    provider, removing the remaining raw-provider wrapper calls from these
    compatibility surfaces.
  - Re-routed `fermat_global_raw_elimination_provider_from_frey_model_laws`
    so its raw-elimination closure receives the primitive-Frey-route provider
    assembled directly from primitive normalization, primitive realization,
    Frey-model provider, and route-data inputs, instead of calling the
    higher-level primitive-route compatibility wrapper.
  - Re-proved `fermat_primitive_frey_route_provider_from_route_data` by
    assembling primitive normalization, primitive realization, Frey-model
    provider, and Wiles/Ribet route-data inputs directly, deriving the imported
    route-data laws at that boundary instead of routing through
    `fermat_primitive_frey_route_provider_from_frey_model_laws`; applied the
    same lower-provider assembly inside
    `fermat_raw_primitive_frey_route_provider_from_frey_model_laws`.
  - Re-routed `fermat_raw_primitive_frey_route_provider_from_route_data` and
    `fermat_global_raw_elimination_provider_from_route_data_and_global_raw_refutation_data`
    so route-data based raw wrappers also assemble the primitive-Frey-route
    provider directly from primitive normalization, primitive realization,
    Frey-model provider, and imported Wiles/Ribet route-data laws; this removes
    the remaining direct calls to
    `fermat_primitive_frey_route_provider_from_route_data` from the FLT source.
  - Re-routed
    `fermat_positive_integer_solution_false_from_selected_frey_model_component_providers`
    so its primitive-normalization provider is constructed directly with
    `fermat_primitive_normalization_data_intro` from the explicit positivity,
    nonzero, coprime, exponent, and Fermat-equation providers, instead of
    calling the normalization-provider compatibility wrapper.
  - Re-routed `fermat_primitive_frey_route_provider_from_frey_model_laws` so
    its normalization input to the primitive-route constructor is the same
    direct `fermat_primitive_normalization_data_intro` provider lambda, moving
    the primitive-route core off the normalization-provider wrapper for that
    boundary.
  - Re-routed the remaining FLT primitive-route, raw-primitive-route,
    raw-elimination, and selected positive-integer solution wrappers so their
    normalization inputs are built directly by
    `fermat_primitive_normalization_data_intro`; the FLT module source no longer
    calls `fermat_primitive_normalization_provider_from_normalization_laws` at
    those boundaries.
  - Re-routed the remaining FLT raw/solution Frey-model provider inputs so they
    are built directly with `fermat_frey_model_data_intro` from the explicit
    builds-curve, discriminant, conductor, minimal-model, and Galois providers
    or laws; the FLT module source no longer calls
    `fermat_frey_model_provider_from_builds_curve_and_frey_model_laws`,
    `fermat_solution_frey_model_provider_from_solution_builds_curve_and_frey_model_laws`,
    or `fermat_frey_model_provider_from_frey_model_laws` at those boundaries.
- Completed L2 solution raw-elimination primitive-provider target:
  - Added
    `fermat_solution_raw_elimination_provider_from_solution_primitive_frey_route_provider`,
    which builds the solution-indexed raw-elimination provider from explicit
    solution primitive-normalization and primitive-Frey-route provider
    families plus raw-realization/no-raw laws.
  - Re-routed the solution raw-primitive-route false/elimination wrappers
    through projected solution primitive providers, and made the
    solution-primitive global-elimination wrapper consume the new
    solution raw-elimination provider directly instead of reconstructing a
    solution raw-primitive-Frey-route provider at that boundary.
- Remaining L2 provider-decomposition target:
  - continue splitting the current raw primitive Frey route provider into
    explicit primitive-normalization and primitive-Frey-route provider
    families, replacing every remaining direct raw-route use by that
    construction.
  - split the primitive-Frey-route provider into primitive-realization,
    Frey-model, and Wiles/Ribet route-data inputs, deriving the primitive
    counterexample record from the existing normalization provider.
  - split the Frey-model provider into explicit builds-curve,
    discriminant-control, conductor-control, minimal-model, and
    Galois-representation providers, and build Wiles/Ribet route data from its
    six route laws at the final wrapper boundary.
  - split the primitive-normalization provider into explicit primitive
    positivity, nonzero, pairwise-coprime, exponent, and Fermat-equation
    providers, constructing the normalization record with the existing L2
    constructor.
  - replace selected Frey-model discriminant/conductor/minimal/Galois providers
    and direct route semistability by generic Frey-model laws plus a
    semistability-from-model theorem.
  - replace direct semistable-modularity and no-bridge route laws by imported
    `SemistableModularityData` specialized to Frey curves, using selected
    local-field/Galois-representation providers and a modularity-lifting
    conclusion provider.
- Remaining theorem targets after the final-statement wrapper:
  - bridge-free Ribet level lowering for the Frey representation;
  - remaining modularity-lifting prerequisites feeding the selected Frey
    representation conclusion provider;
  - incompatibility of the lowered level with the required modular form;
  - replacement of the current abstract raw-to-primitive and primitive Frey
    route witnesses by certified arithmetic, gcd/descent, and Frey-curve
    construction from concrete counterexamples;
  - replacement of the current raw counterexample realization and
    no-raw-counterexample translation law by a certified encoding of the final
    positive-integer no-solution statement;
  - replacement of the current positive-solution elimination provider by a
    certified construction from raw solution evidence for all provider-family
    use sites;
  - replacement of the current global selector and provider families by
    certified normalization, Frey-curve construction, Ribet, and modularity
    closures for each positive-integer solution;
  - unconditional construction of the
    `FermatPositiveIntegerGlobalEliminationData` closure from certified
    arithmetic, Frey-curve, Ribet, and modularity prerequisites.
- Acceptance criteria:
  - `BridgeAxiom`, `BridgeBackedNotCompletedProof`, or statement-only Wiles
    assumptions are not in the final import closure.
  - The final theorem is emitted only after source-free certificate verification
    for every prerequisite in the import closure.
  - Promotion remains out of scope for this task.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.FermatLastTheorem`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.FermatLastTheorem --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "FermatPrimitiveFreyRouteData|FermatRawPrimitiveFreyRouteData|FermatPositiveSolutionData|FermatPositiveIntegerSolutionData|FermatRawCounterexampleEliminationData|FermatGlobalEliminationData|FermatPositiveIntegerGlobalEliminationData|fermat_primitive_frey_route_|fermat_raw_primitive_frey_route_|fermat_semistable_modular_from_primitive_frey_route_data|fermat_no_bridge_axiom_from_primitive_frey_route_data|fermat_ribet_lowering_from_primitive_frey_route_data|fermat_semistable_modular_from_raw_primitive_frey_route_data|fermat_no_bridge_axiom_from_raw_primitive_frey_route_data|fermat_ribet_lowering_from_raw_primitive_frey_route_data|fermat_no_raw_counterexample_from_raw_primitive_frey_route_data|fermat_not_raw_counterexample_from_raw_primitive_frey_route_data|fermat_raw_counterexample_false_from_raw_primitive_frey_route_data|fermat_positive_integer_solution_false_from_raw_primitive_frey_route_data|fermat_not_positive_integer_solution_from_raw_primitive_frey_route_data|fermat_positive_solution_|fermat_positive_integer_solution_|fermat_raw_counterexample_from_positive_solution_data|fermat_positive_solution_data_from_positive_integer_solution|fermat_positive_integer_solution_false_from_positive_solution_data_negation|fermat_not_positive_integer_solution_from_positive_solution_data_negation|fermat_positive_solution_contradiction_from_no_raw_not|fermat_positive_integer_solution_contradiction_from_no_raw_not|fermat_not_raw_counterexample_from_formula_raw_elimination_data|fermat_raw_counterexample_false_from_formula_raw_elimination_data|fermat_positive_integer_solution_false_from_raw_elimination_data|fermat_not_positive_integer_solution_from_raw_elimination_data|fermat_positive_solution_false_from_positive_solution_elimination_provider|fermat_positive_integer_solution_false_from_positive_solution_data_provider|fermat_positive_integer_solution_false_from_solution_raw_elimination_provider|fermat_global_not_positive_integer_solution_from_solution_raw_elimination_provider|fermat_positive_integer_global_elimination_data_intro|fermat_positive_integer_global_elimination_data_from_solution_raw_elimination_provider|fermat_positive_integer_global_elimination_data_from_global_elimination_data|fermat_last_theorem_from_positive_integer_global_elimination_data|fermat_last_theorem_from_global_elimination_data|fermat_global_elimination_data_from_not_raw_provider|fermat_last_theorem_from_raw_elimination_provider|fermat_raw_elimination_provider_from_raw_primitive_frey_route_provider|fermat_raw_counterexample_elimination_|fermat_no_counterexample_from_primitive_frey_route_data|fermat_no_raw_counterexample_from_raw_elimination_data|fermat_no_raw_counterexample_from_positive_solution_elimination_provider|fermat_no_positive_solution_from_positive_solution_elimination_provider|fermat_global_no_positive_solution_from_global_elimination_provider|fermat_global_false_from_global_elimination_provider|fermat_global_not_positive_solution_from_global_elimination_provider|fermat_global_not_positive_solution_from_positive_solution_data_provider|fermat_global_not_positive_integer_solution_from_provider|fermat_global_elimination_data_intro|fermat_global_not_positive_solution_from_global_elimination_data|fermat_positive_integer_solution_false_from_global_raw_elimination_provider|fermat_last_theorem" proofs/Proofs/Ai/NumberTheory/FermatLastTheorem tools/proof-corpus/src/main.rs proofs/generated/ai-theorem-index.json`

## Review Findings

This task document was reviewed against:

- `proofs/number-theory-theorem-proof-roadmap.md`
- `develop/proof-corpus-field-theory-roadmap-todo.md`
- `AGENTS.md`

| Finding | Status | Resolution |
| --- | --- | --- |
| The finite-field core could be duplicated under number theory even though field theory owns `Proofs.Ai.Algebra.AbstractFiniteField`. | Fixed | `NT-T67` imports or aliases the field-theory finite-field core, and `NT-T68`/`NT-T69` only add applications. |
| The initial recommended queue placed finite-field elliptic curves before finite-field ownership and Ribet/modularity before Galois local conditions. | Fixed | Queue groups now place `NT-T67` before `NT-T48`, and `NT-T61` through `NT-T63` before `NT-T51` and `NT-T52`. |
| Completed elliptic-curve interface surfaces could be mistaken for fully L2-derived proofs. | Fixed | Added `NT-T71` through `NT-T78` to convert every current `Proofs.Ai.EllipticCurve.*` theorem surface to L2 or explicitly classify it as conditional or pending while excluding conjectural claims. |
| Analytic and conjectural theorem families could be mistaken for derived certificate targets. | Fixed | Target-level defaults and milestone acceptance criteria exclude conjectures from proof-corpus declarations and reserve conditional forms for named assumptions. |

## Validation

Document-only validation for this task breakdown:

```sh
git diff --check -- proofs/number-theory-theorem-proof-roadmap-todo.md
rg -n "NT-T[0-9][0-9]" proofs/number-theory-theorem-proof-roadmap-todo.md
```

Also search the task document for unresolved local markers, stale owner names,
and accidental references to a duplicated finite-field core before committing.
