# Number Theory Theorem Proof Roadmap

Date: 2026-06-04

This document plans how to prove the user-provided number-theory theorem
inventory one theorem family at a time in the NPA proof corpus. It is a
planning sidecar, not proof evidence.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this roadmap, tactics, and AI output are
untrusted.

## Scope

The theorem inventory covers these areas:

- divisibility, Euclidean division, gcd, lcm, Bezout identities, and Euclid's
  algorithm;
- primes, unique factorization, prime infinitude, prime distribution, and
  elementary primality tests;
- congruences, residue rings, Chinese remainder theorem, Fermat, Euler,
  Wilson, Carmichael, and RSA correctness statements;
- multiplicative groups modulo `n`, primitive roots, characters, orthogonality,
  Gauss sums, quadratic residues, and reciprocity laws;
- arithmetic functions, Dirichlet convolution, Mobius inversion, average
  orders, Dirichlet series, Euler products, zeta functions, and `L`-functions;
- continued fractions, Pell equations, Diophantine approximation, geometry of
  numbers, transcendence interfaces, and Diophantine equations;
- additive and combinatorial number theory, sieve methods, circle method, and
  arithmetic progressions;
- algebraic number theory, number fields, rings of integers, Dedekind domains,
  ideals, class groups, local fields, class field theory, and p-adic analysis;
- elliptic curves, modular forms, Galois representations, modularity,
  arithmetic geometry, Iwasawa theory, and Langlands interfaces;
- finite fields, computational number theory, cryptographic correctness
  theorems, and algorithmic theorem surfaces.

The first priority is not to encode every named theorem immediately. The first
priority is to build small reusable arithmetic foundations whose statements
will not need to be replaced after algebraic number theory, analytic number
theory, elliptic curves, cryptography, or arithmetic geometry milestones depend
on them.

## Existing Baseline

The current proof corpus has reusable algebra, group, ring, field, ordered
field, UFD-style prime factorization, metric-analysis, linear-algebra, and
algebraic-geometry interfaces. It does not yet expose a general
`Proofs.Ai.NumberTheory.*` foundation for divisibility, congruences,
arithmetic functions, local fields, or analytic number theory.

Relevant existing anchors:

| Needed foundation | Expected source |
| --- | --- |
| natural-number and proposition primitives | `Proofs.Ai.Nat`, `Proofs.Ai.Prop`, `Proofs.Ai.Logic.*` |
| abstract rings, fields, ordered fields | `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractField`, `Proofs.Ai.Algebra.AbstractOrderedField` |
| abstract Chinese remainder theorem | `Proofs.Ai.Algebra.AbstractRingChineseRemainder` |
| UFD-style factorization package | `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization` |
| field-extension and finite-field planning | `develop/proof-corpus-field-theory-roadmap.md` and `develop/proof-corpus-field-theory-roadmap-todo.md` |
| finite groups and quotient interfaces | `Proofs.Ai.Algebra.AbstractGroup*` |
| finite-dimensional linear algebra | `proofs/linear-algebra-theorem-proof-roadmap.md` |
| metric and analytic prerequisites | `proofs/analysis-theorem-proof-roadmap.md` and `proofs/measure-theory-theorem-proof-roadmap.md` |
| topology and compactness prerequisites | `proofs/topology-theorem-proof-roadmap.md` |

Number-theory work should start with explicit law packages and statement-level
interfaces, then gradually replace them with derived certificate-backed
modules.

## Proof Levels

Each theorem should be labeled with one of these proof levels while it moves
through the corpus:

| Level | Meaning | Accepted as final for this roadmap |
| --- | --- | --- |
| `L0 Statement` | statement constant, theorem card, or theorem shape only | no |
| `L1 Evidence package` | conclusion follows from an explicit construction, interface, or named external boundary | no for pending theorem-proof tasks; use only as a blocker/dependency note |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

Very large classical results such as the prime number theorem, Dirichlet's
theorem, class field theory, Faltings' theorem, modularity, and the Langlands
correspondence should first be recorded as dependency-map entries. Unresolved
conjectures must not be added as proof-corpus module, source, certificate,
metadata, replay, or theorem-index declarations; they may appear only as
roadmap exclusions or as named assumptions inside explicitly conditional
theorem forms.

## One-Theorem Work Unit

For each theorem, use this work unit:

1. Freeze the statement in the smallest suitable module. Use
   `Proofs.Ai.NumberTheory.*` for arithmetic-owned theorem families, and use an
   existing domain namespace such as `Proofs.Ai.EllipticCurve.*`,
   `Proofs.Ai.ModularForms.*`, `Proofs.Ai.GaloisRepresentation.*`, or
   `Proofs.Ai.AlgebraicGeometry.*` when that roadmap owns the construction.
2. Classify the target as `L0`, `L1`, `L2`, or `L3`.
3. Audit the target for circular assumptions.
4. Keep imports minimal and prefer existing corpus modules.
5. Add or update checked source, replay, metadata, and certificate.
6. Verify the target module source-free.
7. Verify changed proof-corpus artifacts.
8. At the end of a coherent batch, run the authoring gate.

Default proof-corpus commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Replace `Proofs.Ai.NumberTheory.X` with the actual target namespace when the
owning module is outside `Proofs.Ai.NumberTheory`.

Run `./scripts/check-corpus-package.sh` or `./scripts/check-corpus-full.sh`
only for package-wide compatibility, promotion, release readiness, or changes
to certificate encoding, checker behavior, package verification, or kernel
semantics.

## Statement Policy

- Natural numbers, integers, rationals, residue rings, number fields, local
  fields, elliptic curves, modular forms, and Galois representations are
  ordinary library structures or explicit law packages; they are not kernel
  primitives.
- Divisibility, primality, coprimality, congruence, ideal membership,
  valuation, completion, and height predicates must be ordinary definitions or
  explicit evidence packages.
- Tactics, elaborators, theorem search, notation, implicit arguments, and
  automation may produce proof terms or certificates, but are not trusted.
- Quotient rings, residue classes, quotient groups, class groups, and Galois
  quotients must expose their quotient and core-feature requirements in the
  certificate report.
- Analytic number theory may depend on real, complex, topological, measure, and
  integration roadmaps, but may not silently identify analytic facts with
  arithmetic facts.
- Large theorem interfaces may be used for development, but bridge assumptions
  must be named, localized, and rejected by final high-trust policy when the
  theorem is claimed as derived.
- Conjectures are not proof-corpus theorem declarations. They may be recorded
  only as roadmap exclusions or as named assumptions for conditional theorems.

## Milestone Map

| Milestone | Theme | First useful output |
| --- | --- | --- |
| `NT-00` | inventory and statement policy | theorem cards and duplicate-home map |
| `NT-01` | integers, divisibility, and Euclidean division | `Divides`, quotient-remainder, and sign-normalized divisibility APIs |
| `NT-02` | gcd, lcm, and Bezout | Euclid algorithm theorem package |
| `NT-03` | primes and unique factorization | prime extraction and arithmetic fundamental theorem route |
| `NT-04` | congruences and residue rings | congruence algebra and Chinese remainder specialization |
| `NT-05` | Fermat, Euler, Wilson, and finite unit groups | unit-group order and exponent theorem route |
| `NT-06` | primitive roots, characters, and Gauss sums | cyclic unit-group and character orthogonality interfaces |
| `NT-07` | quadratic residues and reciprocity | Legendre, Jacobi, and quadratic reciprocity route |
| `NT-08` | arithmetic functions and convolution | multiplicative functions and Mobius inversion |
| `NT-09` | continued fractions, Pell, and Diophantine approximation | continued-fraction convergent API |
| `NT-10` | Diophantine equations and additive number theory | squares, Pell, Waring, and additive-combinatorics interfaces |
| `NT-11` | analytic number theory foundations | zeta, Dirichlet series, Euler products, and prime number theorem interfaces |
| `NT-12` | sieve and circle method | sieve theorem surfaces and additive prime interfaces |
| `NT-13` | algebraic number theory | algebraic integers, number fields, ideals, and class groups |
| `NT-14` | local fields and p-adic analysis | p-adic valuation, completion, and Hensel route |
| `NT-15` | class field theory | local and global reciprocity interfaces |
| `NT-16` | elliptic curves | group law, height, Mordell-Weil, and finite-field bounds |
| `NT-17` | modular forms and modularity | modular-form, Hecke, and modularity-lifting interfaces |
| `NT-18` | L-functions and Langlands | Artin, Hecke, automorphic `L`-function and correspondence surfaces |
| `NT-19` | arithmetic geometry | rational points, schemes, etale cohomology, and Weil conjectures interfaces |
| `NT-20` | Iwasawa theory | Iwasawa algebra, Selmer, Euler-system, and p-adic `L`-function surfaces |
| `NT-21` | Galois representations and density | Frobenius, Chebotarev, and representation interfaces |
| `NT-22` | computational number theory and cryptography | algorithm correctness theorem targets |
| `NT-23` | finite fields and combinatorial number theory | finite-field application aliases and polynomial-method route |
| `NT-24` | packaging and promotion | stable number-theory closure batches |

## NT-00 Inventory And Statement Policy

- Status: planned.
- Depends on: none.
- Target modules:
  - `Proofs.Ai.NumberTheory.Inventory`
- Theorem order:
  1. classify each theorem from the inventory into exactly one primary
     milestone;
  2. mark duplicates shared with algebra, analysis, topology, measure theory,
     algebraic geometry, cryptography, and arithmetic geometry projects;
  3. assign each theorem a stable English identifier, theorem level, target
     module, dependencies, and acceptance gate;
  4. mark conjectural statements and conditional theorem forms explicitly.
- Deliverables:
  - Number-theory theorem cards in
    `proofs/number-theory-theorem-cards.md`.
  - Duplicate-home map for modularity, Chebotarev, class field theory, finite
    fields, additive combinatorics, and analytic number theory in
    `proofs/number-theory-theorem-cards.md`.
- Acceptance criteria:
  - Every theorem family has one primary home.
  - Cross-roadmap aliases point to the primary theorem instead of duplicating
    proof work.
  - Conjectures are not accidentally classified as derived theorem targets.

## NT-01 Integers, Divisibility, And Euclidean Division

- Status: planned.
- Depends on: `NT-00`, natural-number and integer statement surfaces.
- Target modules:
  - `Proofs.Ai.NumberTheory.Elementary`
  - `Proofs.Ai.NumberTheory.Divisibility`
  - `Proofs.Ai.NumberTheory.EuclideanDivision`
- Theorem order:
  1. divisibility definition and reflexivity, transitivity, sign rules, and
     antisymmetry under nonnegative normalization;
  2. divisor and multiple closure facts;
  3. integer division theorem and quotient-remainder uniqueness;
  4. Euclidean division as the normalized division theorem;
  5. finite descent and well-founded minimization interfaces needed by later
     gcd and Diophantine proofs.
- Proof strategy:
  - Keep `Nat`, `Int`, and positivity translations explicit.
  - Use existing algebra law packages for ring-like reasoning rather than
    adding arithmetic simplification to the kernel.
- Acceptance criteria:
  - Divisibility facts do not depend on prime factorization.
  - Quotient-remainder uniqueness states its exact sign and bound hypotheses.
  - Later gcd and Diophantine modules can import these facts without importing
    elliptic-curve or modularity modules.

## NT-02 Gcd, Lcm, Euclid Algorithm, And Bezout

- Status: planned.
- Depends on: `NT-01`.
- Target modules:
  - `Proofs.Ai.NumberTheory.Gcd`
  - `Proofs.Ai.NumberTheory.Bezout`
  - `Proofs.Ai.NumberTheory.EuclideanAlgorithm`
- Theorem order:
  1. gcd existence and uniqueness;
  2. lcm existence and uniqueness;
  3. gcd-lcm product formula under normalized hypotheses;
  4. Euclid algorithm correctness;
  5. extended Euclid algorithm correctness;
  6. Bezout identity and linear-combination characterization of gcd;
  7. coprime iff a linear combination equals `1`;
  8. linear Diophantine equation solvability and general solution formulas.
- Proof strategy:
  - Derive gcd from Euclidean division first, then derive Bezout and coprime
    facts.
  - Keep algorithmic correctness theorems separate from noncomputable existence
    statements when the extracted algorithm API is not yet available.
- Acceptance criteria:
  - Euclid's lemma and Gauss's lemma are not assumed before Bezout or prime
    divisibility facts are available.
  - Gcd normal form is stable enough for congruence, CRT, and Diophantine
    reduction.

## NT-03 Primes And Unique Factorization

- Status: planned.
- Depends on: `NT-02`, `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`.
- Target modules:
  - `Proofs.Ai.NumberTheory.Prime`
  - `Proofs.Ai.NumberTheory.Factorization`
  - `Proofs.Ai.NumberTheory.UfdBridge`
- Theorem order:
  1. prime and composite definitions over natural numbers and integers;
  2. `1` is not prime and primes have only trivial divisors;
  3. Euclid's lemma and prime divisibility of products;
  4. existence of prime factors for composite numbers;
  5. every integer greater than `1` factors into primes;
  6. uniqueness of prime factorization;
  7. divisor-count, divisor-sum, gcd, and lcm formulas from factorization;
  8. Euclid's infinitude of primes and elementary variants;
  9. square-root bound for composite-number prime factors.
- Proof strategy:
  - Bridge natural-number factorization to the existing abstract UFD theorem
    only through explicit ordered and positivity assumptions.
  - Export general prime-factor projections for finite fields, Diophantine
    equations, and arithmetic functions.
- Acceptance criteria:
  - Fundamental theorem of arithmetic is a derived theorem, not an input axiom.
  - Prime-factorization uniqueness exposes units and sign normalization.

## NT-04 Congruences, Residue Rings, And Chinese Remainder

- Status: planned.
- Depends on: `NT-02`, `NT-03`, `Proofs.Ai.Algebra.AbstractRingChineseRemainder`.
- Target modules:
  - `Proofs.Ai.NumberTheory.Congruence`
  - `Proofs.Ai.NumberTheory.ResidueRing`
  - `Proofs.Ai.NumberTheory.ChineseRemainder`
- Theorem order:
  1. congruence definition and equivalence relation laws;
  2. preservation under addition, multiplication, negation, and powers;
  3. cancellation and division conditions;
  4. residue classes modulo `n` and residue-ring law package;
  5. units modulo `n` and reduced residue classes;
  6. linear congruence solvability and number of solutions;
  7. Chinese remainder theorem for pairwise coprime moduli;
  8. generalized Chinese remainder theorem for compatible systems;
  9. constructive CRT and Garner-style interface.
- Proof strategy:
  - Use quotient or equivalence-relation APIs only with explicit core-feature
    reporting.
  - Specialize the abstract ring CRT to integers modulo ideals only after the
    residue-ring law package is stable.
- Acceptance criteria:
  - CRT construction and uniqueness are separated.
  - Linear congruence results cite gcd facts, not hidden solver behavior.

## NT-05 Fermat, Euler, Wilson, Carmichael, And RSA

- Status: planned.
- Depends on: `NT-04`, finite group order and unit-group interfaces.
- Implementation progress: NT-T14 added certificate-backed
  `Proofs.Ai.NumberTheory.FermatEulerWilson` and `Proofs.Ai.NumberTheory.Carmichael`
  surfaces for theorem steps 3 through 7. NT-T15 added certificate-backed
  `Proofs.Ai.NumberTheory.PrimalityTest` and `Proofs.Ai.NumberTheory.Rsa` surfaces for
  RSA correctness and early primality-test interfaces.
- Target modules:
  - `Proofs.Ai.NumberTheory.ModularGroup`
  - `Proofs.Ai.NumberTheory.FermatEulerWilson`
  - `Proofs.Ai.NumberTheory.Carmichael`
  - `Proofs.Ai.NumberTheory.PrimalityTest`
  - `Proofs.Ai.NumberTheory.Rsa`
- Theorem order:
  1. unit group of `Z/nZ` and Euler `phi` as its order;
  2. Lagrange theorem specialization for finite unit groups;
  3. Fermat's little theorem;
  4. Euler's theorem and Fermat-Euler combined form;
  5. Euler `phi` formula from prime factorization;
  6. Wilson theorem and converse;
  7. Carmichael function definition and exponent theorem;
  8. RSA correctness under coprime and key-congruence hypotheses;
  9. pseudoprime, Carmichael-number, Korselt, and Miller-Rabin theorem
     interfaces.
- Proof strategy:
  - Reuse abstract finite-group order facts for Fermat and Euler.
  - Keep algorithmic primality-test soundness separate from probabilistic or
    complexity claims.
- Acceptance criteria:
  - Fermat's little theorem is not proved by adding a theorem-shaped modular
    arithmetic axiom.
  - RSA correctness states all coprimality and modulus hypotheses explicitly.

## NT-06 Primitive Roots, Characters, And Gauss Sums

- Status: planned.
- Depends on: `NT-05`, finite cyclic group and character interfaces.
- Implementation progress: `NT-T16` added the certificate-backed
  `Proofs.Ai.NumberTheory.PrimitiveRoot` base surfaces for element order,
  primitive-root definitions, cyclic residue-unit generators, and the abstract
  cyclic-group generator-count formula without assuming primitive-root
  existence. `NT-T17` extended that package with odd-prime and prime_power
  existence surfaces plus a ChineseRemainder-dependent classification route for
  moduli admitting primitive roots. `NT-T18` added certificate-backed
  `Proofs.Ai.NumberTheory.Character` and `Proofs.Ai.NumberTheory.GaussSum`
  surfaces for discrete logarithm statements, Dirichlet character groups,
  orthogonality relations, and basic Gauss sum identities.
- Target modules:
  - `Proofs.Ai.NumberTheory.PrimitiveRoot`
  - `Proofs.Ai.NumberTheory.Character`
  - `Proofs.Ai.NumberTheory.GaussSum`
- Theorem order:
  1. element order and relation to powers modulo `n`;
  2. primitive root definition;
  3. primitive roots modulo odd primes;
  4. primitive roots modulo prime powers;
  5. classification of moduli admitting primitive roots;
  6. number of generators of a cyclic group;
  7. discrete logarithm statement surface;
  8. Dirichlet characters and character group structure;
  9. orthogonality relations;
  10. basic Gauss sum identities.
- Proof strategy:
  - Prove cyclic-group facts abstractly, then specialize to residue unit groups.
  - Treat discrete logarithm algorithms as computational interfaces, not
    mathematical primitives.
- Acceptance criteria:
  - Primitive-root classification exposes dependencies on prime powers and CRT.
  - Character orthogonality does not depend on analytic `L`-functions.

## NT-07 Quadratic Residues And Reciprocity

- Status: planned.
- Depends on: `NT-06`.
- Implementation progress: `NT-T19` added certificate-backed
  `Proofs.Ai.NumberTheory.QuadraticResidue` and
  `Proofs.Ai.NumberTheory.Legendre` surfaces for quadratic residue and
  nonresidue definitions, Legendre symbol multiplicativity, Euler criterion
  interfaces from finite cyclic group facts, and odd-prime residue-count
  statements. `NT-T20` added certificate-backed
  `Proofs.Ai.NumberTheory.QuadraticReciprocity` surfaces for Gauss lemma,
  supplementary laws for minus one and two, the first recorded proof route, and
  quadratic reciprocity over distinct odd primes. `NT-T21` added
  certificate-backed `Proofs.Ai.NumberTheory.Jacobi` surfaces for Jacobi symbol
  multiplicativity, separation from actual quadratic residuosity, and
  Solovay-Strassen probabilistic-test interfaces.
- Target modules:
  - `Proofs.Ai.NumberTheory.QuadraticResidue`
  - `Proofs.Ai.NumberTheory.Legendre`
  - `Proofs.Ai.NumberTheory.Jacobi`
  - `Proofs.Ai.NumberTheory.QuadraticReciprocity`
- Theorem order:
  1. quadratic residue and nonresidue definitions;
  2. Legendre symbol definition and multiplicativity;
  3. Euler criterion;
  4. Gauss lemma;
  5. count of quadratic residues modulo odd primes;
  6. supplementary laws for `-1` and `2`;
  7. quadratic reciprocity;
  8. Jacobi symbol definition and multiplicativity;
  9. Jacobi symbol versus actual quadratic residuosity;
  10. Solovay-Strassen theorem interface.
- Proof strategy:
  - Prove Euler criterion from finite cyclic group facts before Gauss lemma
    variants.
  - Choose one first derived route to quadratic reciprocity, then record other
    proofs as aliases or later theorem cards.
- Acceptance criteria:
  - Legendre and Jacobi symbols have distinct APIs.
  - Quadratic reciprocity states odd-prime and distinctness hypotheses
    explicitly.

## NT-08 Arithmetic Functions And Dirichlet Convolution

- Status: planned.
- Depends on: `NT-03`, `NT-04`.
- Implementation progress: `NT-T22` added certificate-backed
  `Proofs.Ai.NumberTheory.ArithmeticFunction` surfaces for divisor-count,
  divisor-sigma, Euler `phi`, Mobius, Liouville, von Mangoldt, and Carmichael
  function interfaces, with finite divisor-support evidence and coprimality
  hypotheses explicit in multiplicativity statements. `NT-T23` added
  certificate-backed `Proofs.Ai.NumberTheory.DirichletConvolution` surfaces for
  Dirichlet convolution definition, associativity, commutativity, identity,
  inverse interfaces, finite divisor-sum rearrangement, and boundaries excluding
  Mobius inversion and infinite-series assumptions. `NT-T24` added
  certificate-backed `Proofs.Ai.NumberTheory.Mobius` and
  `Proofs.Ai.NumberTheory.EulerProduct` surfaces for algebraic Mobius
  inversion, generalized Mobius inversion, finite Euler-product interfaces,
  multiplicative Dirichlet series interfaces, and deferred analytic convergence
  prerequisites.
- Target modules:
  - `Proofs.Ai.NumberTheory.ArithmeticFunction`
  - `Proofs.Ai.NumberTheory.DirichletConvolution`
  - `Proofs.Ai.NumberTheory.Mobius`
  - `Proofs.Ai.NumberTheory.EulerProduct`
- Theorem order:
  1. divisor-count, divisor-sum, Euler `phi`, Mobius, Liouville, von Mangoldt,
     and Carmichael functions;
  2. multiplicative and completely multiplicative function definitions;
  3. Dirichlet convolution definition;
  4. associativity, commutativity, identity, and inverses for convolution;
  5. Mobius inversion and generalized Mobius inversion;
  6. divisor and sigma formulas from prime factorization;
  7. Euler product dependency-map entry for multiplicative Dirichlet series.
- Proof strategy:
  - Keep finite divisor sums separate from analytic infinite series.
  - Prove Mobius inversion algebraically before using it in analytic number
    theory.
- Acceptance criteria:
  - Every sum over divisors has an explicit finiteness or finite-support
    witness.
  - Dirichlet convolution does not require complex analysis.

## NT-09 Continued Fractions, Pell, And Diophantine Approximation

- Status: planned.
- Depends on: `NT-02`, ordered-field and real-analysis prerequisites.
- Implementation progress: `NT-T25` added certificate-backed
  `Proofs.Ai.NumberTheory.ContinuedFraction` surfaces for finite continued
  fractions over rationals, convergent recurrence interfaces, EuclideanDivision
  based finite-expansion routes, normalized finite-expansion uniqueness, explicit
  final-partial-quotient conventions, and boundaries excluding real-analysis or
  infinite-continued-fraction dependencies. `NT-T26` extended
  `Proofs.Ai.NumberTheory.ContinuedFraction` with infinite continued-fraction
  interfaces for irrational inputs, best approximation surfaces, and explicit
  real-analysis prerequisites, and added certificate-backed
  `Proofs.Ai.NumberTheory.Pell` surfaces for quadratic irrational periodicity,
  Pell equation existence and structure, positivity and nonsquare hypotheses,
  normalized-solution conventions, and interface-vs-derived-certificate
  boundaries. `NT-T27` added certificate-backed
  `Proofs.Ai.NumberTheory.DiophantineApproximation` surfaces for Dirichlet
  approximation, simultaneous approximation, Liouville/Roth/Schmidt
  dependency-map entries, Khintchine and Duffin-Schaeffer metric-measure prerequisites,
  Baker and Lindemann-Weierstrass transcendence interfaces, geometry-of-numbers
  assumptions, and boundaries separating metric-measure, algebraic, real-field,
  and elementary-number-theory dependencies.
- Target modules:
  - `Proofs.Ai.NumberTheory.ContinuedFraction`
  - `Proofs.Ai.NumberTheory.Pell`
  - `Proofs.Ai.NumberTheory.DiophantineApproximation`
- Theorem order:
  1. finite continued fractions for rationals;
  2. convergent recurrence relations;
  3. uniqueness of finite and infinite continued-fraction expansions under
     normalized hypotheses;
  4. best-approximation theorems for convergents;
  5. quadratic irrationals and periodic continued fractions;
  6. Pell equation existence and structure;
  7. Dirichlet approximation and simultaneous approximation interfaces;
  8. Liouville, Roth, Schmidt, Khintchine, and geometry-of-numbers theorem
     surfaces.
- Proof strategy:
  - Start with rational finite continued fractions, because they depend only on
    Euclidean division.
  - Defer measure-theoretic metric approximation theorems until the measure
    roadmap has the needed Borel and integration foundations.
- Acceptance criteria:
  - Pell results state positivity and nonsquare hypotheses explicitly.
  - Advanced approximation theorems are interface-level until analytic and
    measure prerequisites are present.

## NT-10 Diophantine Equations And Additive Number Theory

- Status: completed.
- Depends on: `NT-03`, `NT-07`, `NT-09`.
- Implementation progress: `NT-T28` added certificate-backed
  `Proofs.Ai.NumberTheory.Diophantine` and
  `Proofs.Ai.NumberTheory.SumsOfSquares` surfaces for Pythagorean triple
  classification, primitive triple formula, Fermat two-square theorem
  statement route, quadratic residue dependencies, and algebraic identity reuse.
  `NT-T29` appended Lagrange four-square theorem route, Legendre three-square
  theorem interface to `Proofs.Ai.NumberTheory.SumsOfSquares`, and added
  `Proofs.Ai.NumberTheory.Waring` containing Waring's problem existence,
  Hilbert-Waring theorem, and Frobenius coin problem interfaces.
  `NT-T30` created the `Proofs.Ai.NumberTheory.Additive` module containing
  dependency-map entries for Cauchy-Davenport, Kneser, Vosper, Freiman, Plunnecke-Ruzsa,
  Szemeredi, Green-Tao, van der Waerden, Hindman, and Erdos-Ginzburg-Ziv theorems.
- Target modules:
  - `Proofs.Ai.NumberTheory.Diophantine`
  - `Proofs.Ai.NumberTheory.SumsOfSquares`
  - `Proofs.Ai.NumberTheory.Waring`
  - `Proofs.Ai.NumberTheory.Additive`
- Theorem order:
  1. linear Diophantine equation theorem reuse from `NT-02`;
  2. Pythagorean triple classification;
  3. Fermat two-square theorem;
  4. Lagrange four-square theorem;
  5. Legendre three-square theorem interface;
  6. Waring and Hilbert-Waring interfaces;
  7. Frobenius coin problem theorem;
  8. additive-combinatorics theorem surfaces: Cauchy-Davenport, Kneser,
     Vosper, Freiman, Plunnecke-Ruzsa, Szemeredi, Green-Tao.
- Proof strategy:
  - Derive small Diophantine classification results before theorem-heavy
    additive-combinatorics interfaces.
  - Reuse geometry and algebraic identities rather than adding Diophantine
    solver primitives.
- Acceptance criteria:
  - Wiles, Ribet, Frey, and final-theorem-specific work is outside this
    reusable additive-number-theory roadmap.
  - Advanced additive theorems expose finite-set, density, and ambient-group
    assumptions.

## NT-11 Analytic Number Theory Foundations

- Status: in progress.
- Depends on: `NT-08`, analysis, topology, measure, real, and complex
  foundations.
- Implementation progress:
  - `NT-T31` added certificate-backed `Proofs.Ai.NumberTheory.DirichletSeries`
    defining dependency-map entries for Dirichlet series, abscissa of convergence, algebraic
    Euler product, analytic continuation, and Tauberian inputs.
  - `NT-T32` added certificate-backed `Proofs.Ai.NumberTheory.Zeta`
    defining dependency-map entries for Riemann Zeta function, half-plane Euler product,
    analytic continuation, functional equation, zero-free region, Riemann-von Mangoldt
    zero count, explicit formula, and Riemann hypothesis conditional consequence.
  - `NT-T33` added certificate-backed `Proofs.Ai.NumberTheory.PrimeNumberTheorem`
    defining dependency-map entries for Chebyshev estimates, Prime Number Theorem asymptotic
    equivalence, de la Vallee Poussin zero-free region and error bound, Bertrand's
    postulate (independent elementary fact), and Ikehara Tauberian theorem dependency.
  - `NT-T34` added certificate-backed `Proofs.Ai.NumberTheory.DirichletL`
    defining dependency-map entries for Dirichlet L-functions, Euler products, analytic continuation,
    functional equations, $L(1, \chi) \neq 0$, Dirichlet's theorem on primes in arithmetic
    progressions, AP PNT, and GRH conditional consequence.
- Target modules:
  - `Proofs.Ai.NumberTheory.DirichletSeries`
  - `Proofs.Ai.NumberTheory.Zeta`
  - `Proofs.Ai.NumberTheory.DirichletL`
  - `Proofs.Ai.NumberTheory.PrimeNumberTheorem`
- Theorem order:
  1. Dirichlet series and abscissa interfaces;
  2. Euler product for multiplicative arithmetic functions;
  3. Riemann zeta definition;
  4. zeta Euler product in its half-plane of convergence;
  5. analytic continuation and functional equation dependency-map entries;
  6. Chebyshev functions and elementary estimates;
  7. prime number theorem interface;
  8. Dirichlet characters and `L`-function definitions;
  9. `L(1, chi) != 0` interface and Dirichlet theorem for arithmetic
     progressions;
  10. explicit formulas, zero-free regions, and Riemann-von Mangoldt
      interfaces.
- Proof strategy:
  - Separate algebraic Euler-product identities from complex analytic
    continuation and Tauberian arguments.
  - Use dependency-map entries for analytic continuation and zero-free regions until
    complex analysis and measure/integration prerequisites are certified.
- Acceptance criteria:
  - The prime number theorem is not an input to elementary prime facts.
  - Riemann hypothesis and generalized Riemann hypothesis remain conjectural
    statements or assumptions for conditional theorems.

## NT-12 Sieve Methods And Circle Method

- Status: completed.
- Implementation progress:
  - `NT-T35` added certificate-backed `Proofs.Ai.NumberTheory.Sieve`
    defining dependency-map entries for Brun sieve, Selberg sieve, large sieve, fundamental lemma,
    Brun's theorem, twin-prime reciprocal convergence, Chen's theorem, GPY, Zhang,
    Maynard-Tao, parity-problem limitations, explicit error-term/asymptotic inputs,
    visible analytic dependencies, and a boundary preventing sieve surfaces from deriving
    unresolved conjectures.
  - `NT-T36` added certificate-backed `Proofs.Ai.NumberTheory.CircleMethod`
    and `Proofs.Ai.NumberTheory.AdditivePrime` defining dependency-map entries for
    the Hardy-Littlewood circle method, major/minor arc contributions,
    named asymptotic assumptions, harmonic-analysis and exponential-sum dependencies,
    Vinogradov's three-prime theorem, the weak Goldbach conjecture, conditional
    analytic-prerequisite surfaces, and a boundary preventing elementary additive
    theorem surfaces from depending on weak Goldbach.
- Target modules:
  - `Proofs.Ai.NumberTheory.Sieve`
  - `Proofs.Ai.NumberTheory.CircleMethod`
  - `Proofs.Ai.NumberTheory.AdditivePrime`
- Theorem order:
  1. Brun sieve interface;
  2. Selberg sieve interface;
  3. large sieve inequality interface;
  4. fundamental lemma of sieve theory;
  5. Brun theorem and twin-prime reciprocal convergence;
  6. Chen theorem interface;
  7. GPY, Zhang, and Maynard-Tao bounded-gap interfaces;
  8. Hardy-Littlewood circle-method surface;
  9. Vinogradov three-primes theorem and weak Goldbach theorem interfaces.
- Proof strategy:
  - Begin with finite combinatorial sieve inequalities before analytic
    asymptotic theorem surfaces.
  - Keep density, error-term, and asymptotic notation APIs explicit.
- Acceptance criteria:
  - Sieve theorem interfaces expose parity-problem limitations when relevant.
  - Additive prime results state all analytic input dependencies.

## NT-13 Algebraic Number Theory

- Status: completed.
- Implementation progress:
  - `NT-T37` added certificate-backed `Proofs.Ai.NumberTheory.AlgebraicInteger`
    and `Proofs.Ai.NumberTheory.NumberField` defining dependency-map entries for
    algebraic numbers, algebraic integers, their ring structure, rational
    algebraic integer implies integer, explicit rational-to-extension
    embedding/coercion assumptions, number fields, the ring of integers,
    field-extension roadmap dependencies via
    `develop/proof-corpus-field-theory-roadmap.md`, and a boundary preventing
    algebraic-integer ring structure from becoming a kernel primitive.
  - `NT-T38` added certificate-backed `Proofs.Ai.NumberTheory.DedekindDomain`
    defining dependency-map entries for norm, trace, discriminant, integral basis, and
    Dedekind domain, plus field-extension, basis, and finite-dimensional
    vector-space theorem surfaces, a ring-of-integers Dedekind-domain surface,
    and a boundary preventing ideal factorization from being assumed as a
    definition.
  - `NT-T39` added certificate-backed `Proofs.Ai.NumberTheory.ClassGroup`
    defining dependency-map entries for ideal factorization, uniqueness, fractional ideals,
    class group, class number finiteness, Minkowski bound, Dirichlet unit theorem,
    and class number formula, with explicit quotient construction, geometry-of-numbers
    dependency, and analytic class-number formula blocker surfaces.
- Depends on: existing `Proofs.Ai.Algebra.*` modules,
  `develop/proof-corpus-field-theory-roadmap.md`, `NT-03`, ideals, modules,
  field extensions, and finite-dimensional vector spaces from
  `proofs/linear-algebra-theorem-proof-roadmap.md`.
- Target modules:
  - `Proofs.Ai.NumberTheory.AlgebraicInteger`
  - `Proofs.Ai.NumberTheory.NumberField`
  - `Proofs.Ai.NumberTheory.DedekindDomain`
  - `Proofs.Ai.NumberTheory.ClassGroup`
- Theorem order:
  1. algebraic numbers and algebraic integers;
  2. algebraic integers form a ring;
  3. rational algebraic integers are integers;
  4. number fields and rings of integers;
  5. norm, trace, discriminant, and integral basis interfaces;
  6. Dedekind-domain theorem surface for rings of integers;
  7. ideal factorization and uniqueness;
  8. fractional ideal group;
  9. ideal class group definition and class-number finiteness interface;
  10. Dirichlet unit theorem and class-number formula interfaces.
- Proof strategy:
  - Reuse abstract ring, module, ideal, and field-extension law packages.
  - Treat Minkowski theorem and geometry-of-numbers dependencies as explicit
    analysis/convex-geometry imports.
- Acceptance criteria:
  - Dedekind-domain ideal factorization is not assumed as a ring-of-integers
    definition.
  - Class group quotient dependencies are visible in core-feature reports.
  - Analytic class-number formula remains blocker work until analytic prerequisites are certified.

## NT-14 Local Fields And p-adic Analysis

- Status: in-progress.
- Implementation progress:
  - `NT-T40` added certificate-backed `Proofs.Ai.NumberTheory.Valuation` and
    `Proofs.Ai.NumberTheory.Padic` defining dependency-map entries for p-adic valuation,
    p-adic absolute value, non-Archimedean metric, completion, and p-adic field
    construction, with explicit algebra-before-completion, topology/analysis
    completion dependency, and no-local-field-dependency boundary surfaces.
  - `NT-T41` added certificate-backed `Proofs.Ai.NumberTheory.Hensel` and
    `Proofs.Ai.NumberTheory.LocalField` defining dependency-map entries for Hensel lemma,
    Ostrowski theorem, DVR, complete DVR, local-field structure, unramified extension,
    and totally ramified extension, with explicit named Hensel hypotheses,
    no-generic-root-finder boundary, valuation/completion dependency surfaces,
    interface-level construction boundaries, and shared Galois-representation
    ramification vocabulary.
  - `NT-T42` added certificate-backed `Proofs.Ai.NumberTheory.PadicAnalysis` and
    `Proofs.Ai.NumberTheory.PadicMeasure` defining dependency-map entries for p-adic exponential,
    logarithm, Newton polygon, Strassmann theorem, Weierstrass preparation, Mahler
    expansion, p-adic measure, p-adic integration, Amice transform, and
    Kubota-Leopoldt p-adic L-function interpolation, with explicit norm/series
    convergence dependencies, measure-theory-roadmap dependencies, and
    no-trusted-analytic-primitive boundaries.
- Depends on: `NT-13`, topology, metric completion, and valuation interfaces.
- Target modules:
  - `Proofs.Ai.NumberTheory.Valuation`
  - `Proofs.Ai.NumberTheory.Padic`
  - `Proofs.Ai.NumberTheory.LocalField`
  - `Proofs.Ai.NumberTheory.Hensel`
  - `Proofs.Ai.NumberTheory.PadicAnalysis`
  - `Proofs.Ai.NumberTheory.PadicMeasure`
- Theorem order:
  1. p-adic valuation and absolute value;
  2. non-Archimedean metric law package;
  3. completion and p-adic field construction interface;
  4. Hensel lemma;
  5. Ostrowski theorem interface;
  6. discrete valuation rings and complete DVR interfaces;
  7. unramified and totally ramified extension surfaces;
  8. Hilbert symbol and local reciprocity interfaces;
  9. p-adic exponential, logarithm, Newton polygon, Strassmann, Weierstrass
     preparation, Mahler expansion, and p-adic measure surfaces.
- Proof strategy:
  - First prove valuation algebra laws before metric completion statements.
  - Keep p-adic analytic convergence dependent on explicit norm and series
    theorems.
- Acceptance criteria:
  - Hensel lemma states completeness, valuation, derivative, or lifting
    hypotheses exactly for the chosen formulation.
  - Local class field theory remains interface-level until reciprocity modules
    are derived.

## NT-15 Class Field Theory

- Status: in-progress.
- Implementation progress:
  - `NT-T43` added certificate-backed `Proofs.Ai.NumberTheory.ClassField.Local` and
    `Proofs.Ai.NumberTheory.ClassField.Global` defining dependency-map entries for Artin map,
    local reciprocity, Kronecker-Weber theorem, idele class group, global reciprocity,
    Takagi existence, and Hilbert class field, with explicit reciprocity
    domain/codomain/normalization/functoriality data, no-generic-algebra-import
    boundaries, named bridge assumptions, final-promotion bridge rejection, and
    separated local/global reciprocity routes.
  - `NT-T44` added certificate-backed `Proofs.Ai.GaloisCohomology.Basic` defining
    dependency-map entries for Hilbert theorem 90, norm-residue symbol, Hasse norm theorem,
    Grunwald-Wang theorem, Brauer group, and Tate cohomology, plus explicit
    Hilbert-90 degree-one cocycle/coboundary, Norm residue local/global context,
    Brauer degree-two cohomology, Tate degree/functoriality, and
    interface-level-until-foundations boundaries.  It also added
    `Proofs.Ai.NumberTheory.ClassField.Cohomology` as the explicit bridge from
    class-field reciprocity routes to Galois cohomology dependencies.
- Depends on: `NT-13`, `NT-14`, group cohomology, ideles, and Galois
  extensions.
- Target modules:
  - `Proofs.Ai.NumberTheory.ClassField.Local`
  - `Proofs.Ai.NumberTheory.ClassField.Global`
  - `Proofs.Ai.NumberTheory.ArtinReciprocity`
  - `Proofs.Ai.GaloisCohomology.Basic`
  - `Proofs.Ai.NumberTheory.ClassField.Cohomology`
- Theorem order:
  1. Artin map statement surface;
  2. local reciprocity interface;
  3. ideles and idele class group theorem surface;
  4. global reciprocity interface;
  5. Hilbert class field and Takagi existence interfaces;
  6. Kronecker-Weber theorem interface;
  7. Hilbert theorem 90 and norm-residue symbol surfaces;
  8. Hasse norm theorem, Grunwald-Wang, Brauer group, and Tate cohomology
     interfaces.
- Proof strategy:
  - Use explicit `ClassFieldBridgeAxiom.*`-style names for early development
    interfaces if needed.
  - Keep local and global reciprocity separated so later modules can replace
    them independently with certificates.
- Acceptance criteria:
  - No class field theorem is imported under a generic algebra name.
  - Reciprocity maps state domain, codomain, normalization, and functoriality
    assumptions.

## NT-16 Elliptic Curves

- Status: in-progress.
- Implementation progress:
  - `NT-T45` added certificate-backed `Proofs.Ai.EllipticCurve.Basic` and
    `Proofs.Ai.EllipticCurve.GroupLaw` defining dependency-map entries for Weierstrass
    model, nonsingularity, and elliptic curve point group law, with explicit
    field and polynomial assumptions, discriminant/nonzero boundaries, Basic to
    GroupLaw dependency, general API reuse surfaces, and independence from
    modularity, Ribet, Frey-route, or bridge-axiom packages.
  - `NT-T46` added certificate-backed `Proofs.Ai.EllipticCurve.Reduction`,
    `Proofs.Ai.EllipticCurve.Semistable`, and `Proofs.Ai.EllipticCurve.Height`
    defining dependency-map entries for conductor, reduction type, minimal model,
    semistability, and height/Neron-Tate height, with explicit LocalField and
    valuation dependencies, conductor/reduction/minimal-model compatibility,
    semistability as a general non-Frey-specific elliptic-curve predicate, and
    named field/positivity hypotheses for height and Neron-Tate height.
  - `NT-T47` added certificate-backed
    `Proofs.Ai.EllipticCurve.GaloisRepresentation` and
    `Proofs.Ai.EllipticCurve.MordellWeil` defining dependency-map entries for Tate
    module actions, Weil pairing surfaces, Selmer sharing across Iwasawa and
    Galois representation tasks, torsion, Nagell-Lutz, weak Mordell-Weil,
    Mordell-Weil, Selmer group, and Tate-Shafarevich group statement surfaces,
    with explicit Weil pairing nondegeneracy boundaries that do not require
    cryptographic assumptions and an interface-level Mordell-Weil boundary
    until height/descent prerequisites are derived.
  - `NT-T48` added certificate-backed
    `Proofs.Ai.EllipticCurve.FiniteField` and
    `Proofs.Ai.EllipticCurve.LFunction` defining dependency-map entries for finite-field
    point-count, Hasse theorem, Weil bound, Frobenius trace, elliptic and
    Hasse-Weil L-functions, modularity links routed to `NT-T52`, Gross-Zagier,
    Kolyvagin, and Sato-Tate theorem surfaces, with finite-field core laws
    imported from `Proofs.Ai.Algebra.AbstractFiniteField` and unresolved
    conjectural claims excluded from proof-corpus declarations.
- L2 upgrade backlog:
  - `NT-T71` completed the `Proofs.Ai.EllipticCurve.Basic` L2 upgrade with
    structured short-Weierstrass definitions, discriminant/nonzero evidence,
    model-data introduction and projection certificates, and reusable
    route-independence certificates.
  - `NT-T72` completed the `Proofs.Ai.EllipticCurve.GroupLaw` L2 upgrade with
    structured point-at-infinity, doubling, exceptional-pair,
    short-Weierstrass point, and point-group data definitions plus derived
    closure, identity, inverse, associativity, nonsingularity, and reusable
    route certificates.
  - `NT-T73` completed the `Proofs.Ai.EllipticCurve.Reduction` and
    `Proofs.Ai.EllipticCurve.Semistable` L2 upgrade with structured
    local-field valuation input, reduction data, conductor, reduction-type,
    minimal-model, compatibility, general semistability, and
    not-Frey-specific certificates.
  - `NT-T74` completed the `Proofs.Ai.EllipticCurve.Height` L2 upgrade with
    explicit height-field, elliptic-height, and Neron-Tate height data plus
    derived positivity, finiteness, functoriality, and pairing certificates.
  - `NT-T75` completed the `Proofs.Ai.EllipticCurve.GaloisRepresentation` L2
    upgrade with explicit torsion inverse-system, Tate-module Galois-action,
    Weil-pairing, and local-condition bridge data plus derived projection
    certificates.
  - `NT-T76` completed the `Proofs.Ai.EllipticCurve.MordellWeil` L2 upgrade
    with explicit torsion/Nagell-Lutz, Mordell-Weil descent, and
    Selmer/Tate-Shafarevich status data plus derived projection certificates
    that keep height, descent, and cohomology prerequisites explicit.
  - `NT-T77` completed the `Proofs.Ai.EllipticCurve.FiniteField` L2 upgrade
    with imported `AbstractFiniteField` core data, structured point-count and
    Frobenius trace data, and Hasse/Weil-bound projections whose Lang-Weil and
    algebraic-geometry dependencies remain explicit.
  - `NT-T78` completed the `Proofs.Ai.EllipticCurve.LFunction` L2 upgrade with
    explicit `EllipticLFunctionData`, Hasse-Weil L-function projections,
    NT-T77 finite-field local-factor dependencies, and certified-prerequisite
    gates for modularity, Gross-Zagier, Kolyvagin, and Sato-Tate results.
  - `NT-T79` audits every `Proofs.Ai.EllipticCurve.*` declaration and promotes
    only the source-free verified L2-derived subset.

- Depends on: existing `Proofs.Ai.Algebra.*` modules, `NT-13`, local fields,
  and finite fields.
- Target modules:
  - `Proofs.Ai.EllipticCurve.Basic`
  - `Proofs.Ai.EllipticCurve.GroupLaw`
  - `Proofs.Ai.EllipticCurve.Reduction`
  - `Proofs.Ai.EllipticCurve.Semistable`
  - `Proofs.Ai.EllipticCurve.Height`
  - `Proofs.Ai.EllipticCurve.FiniteField`
  - `Proofs.Ai.EllipticCurve.LFunction`
  - `Proofs.Ai.EllipticCurve.GaloisRepresentation`
  - `Proofs.Ai.EllipticCurve.MordellWeil`
- Theorem order:
  1. Weierstrass model and nonsingularity;
  2. elliptic-curve point group law;
  3. torsion and Nagell-Lutz interface;
  4. height and Neron-Tate height surfaces;
  5. weak Mordell-Weil and Mordell-Weil interfaces;
  6. finite-field point-count and Hasse-Weil bounds;
  7. Tate module and Weil pairing interfaces;
  8. Selmer group and Tate-Shafarevich group statement surfaces;
  9. modularity, Gross-Zagier, Kolyvagin, Sato-Tate, and conditional theorem
     surfaces whose assumptions are named explicitly.
- L2 upgrade order:
  1. Basic Weierstrass/nonsingularity definitions and discriminant lemmas
     (`NT-T71`);
  2. point addition and group-law proof closure (`NT-T72`);
  3. reduction, minimal models, semistability, and local-field dependencies
     (`NT-T73`);
  4. height and Neron-Tate height prerequisites (`NT-T74`);
  5. Tate module, Weil pairing, and Galois-representation APIs (`NT-T75`);
  6. Mordell-Weil, Selmer, and Tate-Shafarevich theorem status split
     (`NT-T76`);
  7. finite-field point-count, Frobenius trace, Hasse theorem, and Weil bound
     (`NT-T77`);
  8. elliptic/Hasse-Weil L-functions and deep conditional theorem surfaces
     (`NT-T78`);
  9. closure audit and promotion decision (`NT-T79`).
- Proof strategy:
  - Keep general elliptic-curve APIs independent of specialized Frey-curve
    routes.
  - Keep final-theorem-specific glue out of the reusable elliptic-curve APIs.
- Acceptance criteria:
  - Group law theorem does not rely on modularity or arithmetic geometry
    bridge axioms.
  - No unresolved conjecture is present as an elliptic-curve proof-corpus
    theorem, source, certificate, metadata, replay, or theorem-index
    declaration.

## NT-17 Modular Forms And Modularity

- Status: planned.
- Progress:
  - `NT-T49` added certificate-backed `Proofs.Ai.ModularForms.Basic` and
    `Proofs.Ai.ModularForms.QExpansion` defining reusable modular-form,
    cusp-form, explicit complex-analytic domain, `weight`/`level`,
    `q_expansion`, Eisenstein series, coefficient, and q-expansion principle
    surfaces outside final-theorem glue.
  - `NT-T50` added certificate-backed `Proofs.Ai.ModularForms.Hecke` and
    `Proofs.Ai.ModularForms.ModularCurve` defining Hecke operator, eigenform,
    coefficient-field, Fourier-coefficient multiplicativity, Petersson inner
    product, trace formula, modular curve, Jacobian, and Eichler-Shimura
    surfaces with analytic/geometric prerequisites and construction evidence
    explicit.
  - `NT-T51` added certificate-backed
    `Proofs.Ai.Modularity.LevelLowering` and
    `Proofs.Ai.Modularity.Ribet` defining reusable conductor,
    irreducibility, ramification, newform, excluded-case, dependency-map, Ribet
    level-lowering, bridge-namespace, not-completed-proof, and high-trust
    import-boundary surfaces.
  - `NT-T52` added certificate-backed `Proofs.Ai.Modularity.Lifting` and
    `Proofs.Ai.Modularity.Semistable` defining deformation functor,
    deformation ring, Hecke/deformation comparison, `R_eq_T`, minimal and
    non-minimal modularity lifting, named deep-assumption, semistable
    modularity, reusable-assumption, and no-bridge-dependency surfaces.
- Depends on: complex analysis, linear algebra, representation theory, algebraic
  geometry, and `NT-16`.
- Target modules:
  - `Proofs.Ai.ModularForms.Basic`
  - `Proofs.Ai.ModularForms.Hecke`
  - `Proofs.Ai.ModularForms.QExpansion`
  - `Proofs.Ai.ModularForms.ModularCurve`
  - `Proofs.Ai.Modularity.LevelLowering`
  - `Proofs.Ai.Modularity.Lifting`
  - `Proofs.Ai.Modularity.Ribet`
  - `Proofs.Ai.Modularity.Semistable`
- Theorem order:
  1. modular forms, cusp forms, weights, levels, and q-expansions;
  2. Eisenstein series and q-expansion principle interfaces;
  3. finite-dimensionality of modular-form spaces;
  4. Hecke operators and eigenforms;
  5. Fourier-coefficient multiplicativity for eigenforms;
  6. Petersson inner product and trace formula interfaces;
  7. modular curves and Eichler-Shimura interface;
  8. modularity lifting theorem surfaces;
  9. Ribet level lowering and semistable modularity interfaces.
- Proof strategy:
  - Keep modularity-lifting APIs reusable and separate from any final-theorem
    route.
  - Do not hide modularity-lifting assumptions in reusable modules.
- Acceptance criteria:
  - Modular forms modules are usable outside modularity-lifting wrappers.
  - Ribet and Wiles/Taylor-Wiles surfaces are not hidden in downstream theorem
    imports.

## NT-18 L-functions And Langlands Interfaces

- Status: planned.
- Progress:
  - `NT-T53` added certificate-backed `Proofs.Ai.NumberTheory.LFunction`,
    `Proofs.Ai.NumberTheory.ArtinL`, and `Proofs.Ai.NumberTheory.HeckeL`
    defining coefficient-field, local-factor, Euler-product, analytic-domain,
    normalization, analytic-continuation, functional-equation, Artin
    reciprocity, Hasse-Weil, automorphic, and no-conjectural-`L2` surfaces.
  - `NT-T54` added certificate-backed `Proofs.Ai.Langlands.TraceFormula` and
    `Proofs.Ai.NumberTheory.AutomorphicL`, preserving explicit trace formula
    assumptions, Arthur-Selberg, endoscopic transfer, Fundamental lemma,
    Rankin-Selberg, Langlands-Shahidi, converse theorem, and analytic
    continuation blocker surfaces.
  - `NT-T55` added certificate-backed `Proofs.Ai.Langlands.Interface`,
    preserving local/global correspondence, Jacquet-Langlands, base change,
    conditional `L0` functoriality, Sato-Tate, potential automorphy,
    promotable-subtheorem dependency graph, and no-broad-derived-certificate
    boundary surfaces.
- Depends on: `NT-11`, `NT-15`, `NT-17`, Galois representations, automorphic
  representation interfaces, and trace formula prerequisites.
- Target modules:
  - `Proofs.Ai.NumberTheory.LFunction`
  - `Proofs.Ai.NumberTheory.ArtinL`
  - `Proofs.Ai.NumberTheory.HeckeL`
  - `Proofs.Ai.Langlands.TraceFormula`
  - `Proofs.Ai.NumberTheory.AutomorphicL`
  - `Proofs.Ai.Langlands.Interface`
- Theorem order:
  1. Artin, Hecke, Hasse-Weil, and automorphic `L`-function definitions;
  2. Euler product theorem surfaces;
  3. analytic continuation and functional equation interfaces;
  4. Artin reciprocity connection to class field theory;
  5. local and global Langlands correspondence statement surfaces;
  6. Jacquet-Langlands and base-change interfaces;
  7. trace formula, Arthur-Selberg, endoscopic transfer, fundamental lemma,
     Rankin-Selberg, Langlands-Shahidi, converse theorem, Sato-Tate, and
     potential automorphy interfaces.
- Proof strategy:
  - Treat the broad Langlands program as an interface graph first.
  - Promote individual proved theorem surfaces only when their import closures
    are narrow and auditable.
- Acceptance criteria:
  - Conjectural functoriality and correspondence statements are not marked
    `L2`.
  - Each `L`-function names its coefficient field, local factors, analytic
    domain, and normalization.

## NT-19 Arithmetic Geometry

- Status: planned.
- Progress:
  - `NT-T56` added certificate-backed
    `Proofs.Ai.ArithmeticGeometry.RationalPoints`, preserving curve genus,
    divisor, Riemann-Roch, Hasse-Weil bound and zeta-function surfaces,
    Mordell/Faltings and Siegel rational/integral-point statement surfaces,
    explicit rational/integral-point hypotheses, finite-field core reuse, and
    separation from etale cohomology construction interfaces.
  - `NT-T57` added certificate-backed
    `Proofs.Ai.ArithmeticGeometry.Schemes`,
    `Proofs.Ai.ArithmeticGeometry.EtaleCohomology`, and
    `Proofs.Ai.ArithmeticGeometry.WeilConjectures`, preserving scheme,
    fiber-product, Zariski-topology, flatness, base-change, Kummer exact
    sequence, etale-cohomology finiteness, Grothendieck/Lefschetz trace
    formula, Weil conjectures, and Deligne theorem surfaces while keeping
    cohomology assumptions and scheme/etale dependencies explicit.
  - `NT-T58` added certificate-backed
    `Proofs.Ai.ArithmeticGeometry.PadicHodge` and
    `Proofs.Ai.ArithmeticGeometry.SpecialPoints`, preserving Neron model,
    Neron-Ogg-Shafarevich, Chabauty-Coleman, l-adic representation, p-adic
    Hodge comparison, period-ring assumption, Galois-representation API reuse,
    Manin-Mumford, Mordell-Lang, Bogomolov, and Andre-Oort status-labeled
    statement surfaces.
- Depends on: existing `Proofs.Ai.AlgebraicGeometry.*` modules, `NT-13`,
  `NT-16`, `NT-18`, scheme and cohomology foundations.
- Target modules:
  - `Proofs.Ai.ArithmeticGeometry.RationalPoints`
  - `Proofs.Ai.ArithmeticGeometry.Schemes`
  - `Proofs.Ai.ArithmeticGeometry.EtaleCohomology`
  - `Proofs.Ai.ArithmeticGeometry.WeilConjectures`
  - `Proofs.Ai.ArithmeticGeometry.PadicHodge`
  - `Proofs.Ai.ArithmeticGeometry.SpecialPoints`
- Theorem order:
  1. genus, divisors, and Riemann-Roch interface for curves;
  2. Hasse-Weil bounds and zeta functions of varieties over finite fields;
  3. Weil conjectures and Deligne theorem interfaces;
  4. Mordell conjecture/Faltings theorem interface;
  5. Siegel integral-points theorem interface;
  6. Neron models and Neron-Ogg-Shafarevich interface;
  7. Chabauty-Coleman method surface;
  8. Manin-Mumford, Mordell-Lang, Bogomolov, and Andre-Oort statement
     surfaces;
  9. proper base change, smooth base change, Kummer exact sequence, trace
     formula, Poincare duality, l-adic representation, and p-adic Hodge
     comparison interfaces.
- Proof strategy:
  - Reuse the algebraic-geometry proof corpus modules where their theorem
    surfaces already exist.
  - Keep rational-point theorems separated from etale cohomology construction
    interfaces.
- Acceptance criteria:
  - Weil and Faltings-level results are clearly interface-level until their
    massive dependencies are available.
  - p-adic Hodge comparison states period-ring and representation hypotheses.

## NT-20 Iwasawa Theory

- Status: planned.
- Progress:
  - `NT-T59` added certificate-backed
    `Proofs.Ai.NumberTheory.Iwasawa.Basic`, preserving cyclotomic `Z_p`
    extension, Iwasawa algebra, explicit module-theoretic assumptions over the
    Iwasawa algebra, finitely generated torsion module structure, lambda, mu,
    and nu invariant, Iwasawa class-number formula, p-adic dependency,
    Galois-cohomology dependency, and non-confusion with the `NT-T39` analytic
    class-number formula surfaces.
  - `NT-T60` added certificate-backed
    `Proofs.Ai.NumberTheory.Iwasawa.MainConjecture` and
    `Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`, preserving
    Kubota-Leopoldt p-adic `L`-function reuse of `NT-T42`, interpolation
    formula, exact-assumption Iwasawa main conjecture conditional forms,
    Mazur-Wiles, Ferrero-Washington, `mu = 0`, Euler-system norm relations,
    Kato, Rubin, Coates-Wiles, Skinner-Urban, plus/minus Selmer groups shared
    with elliptic-curve modules, Gross-Koblitz, and Euler-system links back to
    Iwasawa main conjecture surfaces.
- Depends on: `NT-14`, `NT-15`, `NT-16`, `NT-18`, module theory, and Galois
  cohomology.
- Target modules:
  - `Proofs.Ai.NumberTheory.Iwasawa.Basic`
  - `Proofs.Ai.NumberTheory.Iwasawa.MainConjecture`
  - `Proofs.Ai.NumberTheory.Iwasawa.EulerSystem`
- Theorem order:
  1. cyclotomic `Z_p` extension interface;
  2. Iwasawa algebra and finitely generated torsion module structure theorem;
  3. lambda, mu, and nu invariants;
  4. Iwasawa class-number formula interface;
  5. Kubota-Leopoldt p-adic `L`-function and interpolation formula surfaces;
  6. Iwasawa main conjecture, Mazur-Wiles, Ferrero-Washington, and `mu = 0`
     theorem surfaces;
  7. Euler systems, Kato, Rubin, Coates-Wiles, Skinner-Urban, plus/minus
     Selmer groups, and Gross-Koblitz interfaces.
- Proof strategy:
  - Start with module-theoretic structure statements over the Iwasawa algebra.
  - Keep main conjectures as theorem interfaces or conditional forms with
    exact assumptions.
- Acceptance criteria:
  - p-adic `L`-function interfaces reuse `NT-14` p-adic analysis, not a private
    convergence vocabulary.
  - Selmer group definitions are shared with elliptic-curve and Galois
    representation modules.

## NT-21 Galois Representations And Density Theorems

- Status: planned.
- Progress:
  - `NT-T61` added certificate-backed
    `Proofs.Ai.NumberTheory.Frobenius`, preserving prime ideal decomposition,
    decomposition group, inertia group, Frobenius element with explicit
    unramified and prime-ideal hypotheses, Frobenius conjugacy-class theorem,
    Dedekind-domain and class-group dependencies, local-field ramification and
    Galois-representation local-condition dependencies, reusable decomposition
    and inertia terms for local conditions, shared ramification vocabulary, and
    a boundary excluding Chebotarev imports from the definition layer.
  - `NT-T62` added certificate-backed
    `Proofs.Ai.NumberTheory.Chebotarev`, preserving Frobenius conjugacy-class
    dependency on `NT-T61`, explicit density measure and analytic assumptions,
    Chebotarev density theorem, Frobenius density theorem, Dirichlet theorem
    from Chebotarev as an alias/later-card route, a no-duplicate-Dirichlet-`L`
    boundary, and independence boundaries for elementary prime infinitude and
    the fundamental theorem of arithmetic.
  - `NT-T63` added certificate-backed
    `Proofs.Ai.GaloisRepresentation.Basic`,
    `Proofs.Ai.GaloisRepresentation.Ramification`, and
    `Proofs.Ai.GaloisRepresentation.LocalCondition`, preserving reusable
    Galois representation, l-adic representation, cyclotomic character, Tate
    module representation, ramification, Frobenius compatibility,
    Fontaine-Laffaille to crystalline/semistable/de Rham/Hodge-Tate comparison,
    elliptic-curve and modular-form shared local-condition APIs, and
    Taylor-Wiles and potential-modularity interface-only boundaries.
- Depends on: field extensions, Galois groups, local fields, `NT-13`, `NT-16`,
  and representation theory.
- Target modules:
  - `Proofs.Ai.GaloisRepresentation.Basic`
  - `Proofs.Ai.GaloisRepresentation.Ramification`
  - `Proofs.Ai.GaloisRepresentation.LocalCondition`
  - `Proofs.Ai.NumberTheory.Chebotarev`
  - `Proofs.Ai.NumberTheory.Frobenius`
  - `Proofs.Ai.GaloisCohomology.Basic`
- Theorem order:
  1. Galois group and fundamental theorem interfaces as needed by arithmetic
     modules;
  2. prime ideal decomposition, decomposition group, inertia group, and
     Frobenius element;
  3. Frobenius conjugacy-class theorem;
  4. Chebotarev density theorem interface;
  5. Dirichlet theorem from Chebotarev interface;
  6. Galois representations, l-adic representations, cyclotomic character,
     Tate module representations;
  7. Hodge-Tate, de Rham, crystalline, semistable, Fontaine-Laffaille, and
     comparison theorem surfaces;
  8. Brauer-Nesbitt, Faltings-Serre, Taylor-Wiles, potential modularity, and
     modularity-lifting interfaces.
- Proof strategy:
  - Define Frobenius and local decomposition data before density statements.
  - Share representation APIs with elliptic-curve and modular-form modules.
- Acceptance criteria:
  - Chebotarev is not used to prove elementary prime infinitude or FTA.
  - Representation-theoretic local conditions are reusable outside any
    specialized final-theorem route.

## NT-22 Computational Number Theory And Cryptography

- Status: planned.
- Progress:
  - `NT-T64` added certificate-backed `Proofs.Ai.NumberTheory.Algorithm`,
    naming Euclid, extended Euclid, constructive CRT, and repeated-squaring
    algorithm tokens/functions, composing descent/remainder, Bezout, CRT
    residue checks, and repeated-squaring invariants into correctness, and
    recording cost-model, mathematical-existence, and external-solver/oracle
    boundaries.
  - `NT-T65` extended certificate-backed `Proofs.Ai.NumberTheory.PrimalityTest`
    with Fermat, Miller-Rabin, and AKS algorithm correctness chains, explicit
    randomness assumptions for probabilistic Miller-Rabin failure claims,
    cost-model separation, and hardness non-derivation boundaries, and added
    `Proofs.Ai.NumberTheory.FactoringAlgorithm` for Pollard rho, quadratic
    sieve, and number field sieve factor-extraction statement surfaces.
  - `NT-T66` added certificate-backed `Proofs.Ai.Cryptography.NumberTheory`
    and `Proofs.Ai.Cryptography.EllipticCurve`, reusing RSA, discrete-log,
    algorithmic factoring, and elliptic-curve group/pairing APIs for
    Diffie-Hellman, ECDSA, Weil/Tate pairing, LLL, and Coppersmith theorem
    surfaces, with explicit group, randomness, key-generation, and hardness
    non-`L2` boundaries.
- Depends on: `NT-04`, `NT-05`, `NT-16`, finite-field and algorithmic
  correctness APIs.
- Target modules:
  - `Proofs.Ai.NumberTheory.Algorithm`
  - `Proofs.Ai.NumberTheory.PrimalityTest`
  - `Proofs.Ai.NumberTheory.FactoringAlgorithm`
  - `Proofs.Ai.Cryptography.NumberTheory`
  - `Proofs.Ai.Cryptography.EllipticCurve`
- Theorem order:
  1. Euclid and extended Euclid algorithm correctness and complexity
     interfaces;
  2. constructive CRT correctness;
  3. repeated squaring correctness;
  4. Fermat, Miller-Rabin, and AKS primality-test theorem surfaces;
  5. Pollard rho, quadratic sieve, and number field sieve theory interfaces;
  6. RSA correctness;
  7. Diffie-Hellman, discrete log, and elliptic-curve cryptography statement
     surfaces;
  8. ECDSA correctness under explicit group and randomness assumptions;
  9. Weil and Tate pairing bilinearity and nondegeneracy interfaces;
  10. LLL and Coppersmith theorem interfaces.
- Proof strategy:
  - Separate mathematical correctness from runtime or probabilistic security
    claims.
  - Treat cryptographic assumptions as assumptions, not proved mathematical
    theorems.
- Acceptance criteria:
  - Algorithm theorem statements name the implemented function or abstract
    algorithm relation being verified.
  - Security hardness assumptions are never marked as derived certificates.

## NT-23 Finite Fields And Combinatorial Number Theory

- Status: planned.
- Depends on: existing `Proofs.Ai.Algebra.*` modules,
  `develop/proof-corpus-field-theory-roadmap.md`, `NT-03`, `NT-06`, `NT-07`,
  and polynomial APIs.
- Target modules:
  - `Proofs.Ai.Algebra.AbstractFiniteField`
  - `Proofs.Ai.NumberTheory.FiniteFieldApplications`
  - `Proofs.Ai.NumberTheory.ExponentialSum`
  - `Proofs.Ai.NumberTheory.Combinatorial`
- Progress:
  - `NT-T67` adds `Proofs.Ai.NumberTheory.FiniteFieldApplications` as the number-theoretic
    application namespace for field-theory-owned finite-field law, Frobenius, cardinality, root,
    ownership, primitive-root, and Gauss-sum dependency cards.
  - `NT-T68` adds `Proofs.Ai.NumberTheory.ExponentialSum` downstream of
    `FiniteFieldApplications`, `Character`, and `GaussSum`, with explicit field-size, degree,
    character, and nonvanishing hypotheses for Gauss/Jacobi sums, Hasse-Davenport,
    Stickelberger, Chevalley-Warning, Ax-Katz, Weil, and Lang-Weil
    dependency-map entries.
  - `NT-T69` adds `Proofs.Ai.NumberTheory.Combinatorial` downstream of
    `FiniteFieldApplications` and `Additive`, with explicit ambient structures, field-size,
    degree, and nonvanishing hypotheses for Ramsey-style, additive-combinatorics, polynomial
    method, and combinatorial Nullstellensatz interfaces.
- Theorem order:
  1. import or alias the field-theory finite-field law package, Frobenius,
     cardinality, and root-characterization results;
  2. record finite-field existence, uniqueness, and multiplicative-cyclicity
     theorem cards with ownership checked against the field-theory roadmap;
  3. subfield classification and Frobenius automorphism applications;
  4. root-count and irreducible-polynomial count formulas;
  5. Gauss sums, Jacobi sums, Hasse-Davenport, and Stickelberger interfaces;
  6. Chevalley-Warning and Ax-Katz interfaces;
  7. Weil exponential-sum estimates and Lang-Weil interfaces;
  8. pigeonhole, Ramsey, van der Waerden, Schur, Rado, Erdos-Ginzburg-Ziv,
     Cauchy-Davenport, Kneser, Vosper, Olson, Davenport constant, polynomial
     method, and combinatorial Nullstellensatz surfaces.
- Proof strategy:
  - Reuse the field-theory finite-field law package and do not redefine the
    finite-field core under `Proofs.Ai.NumberTheory`.
  - Reuse finite-group and polynomial algebra facts for finite-field
    applications.
  - Keep additive-combinatorics statements parameterized by ambient group or
    field.
- Acceptance criteria:
  - Finite-field construction, Frobenius, and root-characterization facts are
    imported or aliased from the field-theory route rather than privately
    duplicated under number theory.
  - Finite field multiplicative cyclicity is not assumed for primitive-root
    theorems without an explicit dependency.
  - Polynomial-method results name degree, field-size, and nonvanishing
    hypotheses.

## NT-24 Packaging And Promotion

- Status: planned.
- Depends on: stable local closure for one or more `NT-*` theorem batches.
- Target modules:
  - `Proofs.Ai.NumberTheory.*`
  - related domain namespaces selected by the closure batch, such as
    `Proofs.Ai.EllipticCurve.*`, `Proofs.Ai.ModularForms.*`,
    `Proofs.Ai.GaloisRepresentation.*`, `Proofs.Ai.Modularity.*`, or
    `Proofs.Ai.AlgebraicGeometry.*`
  - `npa-mathlib` closure sidecars after audit
- Theorem order:
  1. choose a coherent closure batch, such as elementary arithmetic through
     CRT, or arithmetic functions through Mobius inversion;
  2. run local proof-corpus authoring checks;
  3. run package checks and axiom reports;
  4. verify that bridge axioms and conjectural statement assumptions are absent
     from derived theorem exports;
  5. materialize the closure into `npa-mathlib` only after audit.
- Deliverables:
  - Closure manifest.
  - Axiom and core-feature report.
  - Public theorem list for promoted number-theory modules.
- Acceptance criteria:
  - Every public theorem has a source-free certificate verdict.
  - Package metadata, import hashes, and certificate hashes are deterministic.
  - Bridge interfaces remain in development namespaces or are removed from the
    promoted closure.

## NT-25 Fermat's Last Theorem Route

- Status: in progress.
- Depends on: `NT-16`, `NT-17`, `NT-21`, and certified elementary
  number-theory prerequisites.
- Target modules:
  - `Proofs.Ai.NumberTheory.FermatLastTheorem`
  - Frey-specific theorem batches under number theory rather than reusable
    elliptic-curve API modules
  - bridge-free `Proofs.Ai.Modularity.*` prerequisites once they have L2
    certificates
- Progress:
  - `NT-T80` adds the first certificate-backed
    `Proofs.Ai.NumberTheory.FermatLastTheorem` module with structured
    primitive-counterexample data and a Wiles/Ribet/Frey route data package.
    The module proves introduction, projection, and route-composition
    certificates only; it does not export a completed Fermat's Last Theorem.
  - `NT-T81` adds initial certificate-backed raw-counterexample,
    exponent-reduction, and primitive-normalization data plus composition
    certificates. Prime-exponent extraction and gcd/descent normalization remain
    open prerequisites.
  - `NT-T82` is adding Frey-model data, projections, model-data semistability
    composition, and a route-composition certificate from Frey model data. It
    does not yet construct the Frey curve from concrete elliptic-curve data.
  - `NT-T83` is adding route-level composition certificates from Frey model data
    through semistable modularity, no-bridge evidence, Ribet lowering, and the
    level-two contradiction. It also packages a primitive counterexample
    witness, Frey model data, and route data into a derived no-counterexample
    certificate, then connects raw counterexample normalization to the primitive
    Frey route and derives a raw tuple-level no-counterexample certificate from
    an explicit raw-realization translation law. It also derives the raw
    tuple-level contradiction from positive solution evidence once a
    solution-elimination provider is supplied, and lifts that contradiction to
    a surface positive-solution predicate through explicit projection laws.
    The current layer further adds `FermatPositiveSolutionData`, certified
    projections for its positivity/nonzero/exponent/equation fields, and a
    bridge to `FermatRawCounterexampleData`. It also adds
    `FermatPositiveIntegerSolutionData`, whose equation field is the concrete
    formula `EqualInt (Add (Pow x n) (Pow y n)) (Pow z n)`, and proves the
    bridge from that final-statement syntax record to the formula-instantiated
    positive-solution record. It lifts the pointwise surface contradiction to a
    universally quantified conditional no-positive-solution theorem under
    explicit global selector and elimination-provider families, specializes it
    to `False`-valued and `Not`-wrapped negated positive-solution shapes using
    the existing `Proofs.Ai.Logic.Iff` falsehood/negation API, specializes that
    global negation theorem to the concrete positive-solution record without
    accepting separate surface projection laws, and then transports the result
    to the concrete positive-integer solution syntax. It also specializes the
    no-raw-counterexample predicate to
    `Not (FermatRawCounterexampleData ...)`, deriving the positive-solution
    contradiction by applying `not_elim` to the raw-data bridge rather than
    accepting a separate contradiction law at that layer. The local
    raw-elimination layer is now specialized through the concrete
    positive-integer solution syntax as `Not` of the raw counterexample,
    `False` from a positive-integer solution plus elimination data, and `Not`
    of the concrete positive-integer solution from a solution-dependent local
    elimination provider; the solution-dependent provider is also lifted to a
    global concrete positive-integer no-solution theorem. The selector,
    positive-integer-to-raw, and solution-indexed elimination data are now
    packaged as `FermatPositiveIntegerGlobalEliminationData`; the
    formula-specialized `FermatGlobalEliminationData` closure constructs that
    concrete positive-integer closure through the certified solution-to-raw
    bridge, and that concrete closure now exposes certificate-backed
    projections from a positive-integer solution to formula-specialized raw
    counterexample data, including closures built from solution-indexed
    raw-elimination providers. The formula-specialized global closure is now
    constructed from the
    raw-elimination provider using certified projection and contradiction
    theorems; the raw-elimination provider is now built from
    raw-primitive-Frey-route, raw-realization, and no-raw law inputs. The
    raw-primitive-Frey-route closure now also exposes the primitive realization,
    Frey-model data, and route-data projections directly through
    `fermat_raw_primitive_frey_route_realizes_primitive`,
    `fermat_raw_primitive_frey_route_frey_data`, and
    `fermat_raw_primitive_frey_route_route_data`. The primitive and raw
    primitive closures also now expose direct semistability, no-bridge, and
    Ribet-lowering projections through the matching primitive-route and
    raw-primitive-route theorem families. The raw primitive route layer also
    derives direct no-raw, raw-contradiction, and concrete positive-integer
    contradiction closures before packaging them into raw-elimination data. The next
    provider-decomposition target splits the raw-primitive-Frey-route provider
    into primitive-normalization and primitive-Frey-route providers, then splits
    the primitive-Frey-route provider into primitive-realization, Frey-model,
    and Wiles/Ribet route-data inputs. The next layer splits the Frey-model
    provider into explicit builds-curve, discriminant/conductor/minimal-model,
    and Galois-representation providers, builds Wiles/Ribet route data from
    route laws, and splits the primitive-normalization provider into primitive
    positivity, nonzero, pairwise-coprime, exponent, and Fermat-equation
    providers. The following layer replaces selected Frey-model
    discriminant/conductor/minimal/Galois providers and direct route
    semistability by generic Frey-model laws plus semistability-from-model
    before routing `fermat_last_theorem` through that construction. The next
    layer replaces direct semistable-modularity and no-bridge route laws by
    imported `SemistableModularityData` specialized to Frey curves with
    selected local-field/Galois-representation providers and a
    modularity-lifting conclusion provider. The following Ribet layer preserves
    an explicit bridge-backed compatibility wrapper over `RibetBridgeData`,
    then derives the public Frey route `ribet_level_lowering_law` directly
    from `LevelLoweringData` and selected
    conductor/residual/ramification/newform/excluded/lowered-level providers.
    The next level-two layer splits the direct
    `level_two_contradiction_law` into a lowering-to-obstruction law and an
    obstruction-to-contradiction law packaged as
    `FermatLevelTwoObstructionData`. The following no-counterexample layer
    splits the direct `no_counterexample_law` into a
    level-two-contradiction-to-counterexample-contradiction law and a
    counterexample-contradiction-to-`NoFermatCounterexample` law packaged as
    `FermatNoCounterexampleData`. The next raw-refutation layer packages the
    selected raw-realization provider and no-raw-counterexample law as
    `FermatGlobalRawRefutationData`, and derives that raw-refutation data from
    the stronger solution-indexed raw-elimination provider by certified
    projection and local `Not` reconstruction; the stored raw-realization
    provider and no-raw-counterexample law also have certified projection
    theorems, with a concrete specialization of the raw-realization evidence
    and a component-wise recomposition of the selected raw-counterexample
    negation, then a concrete contradiction from that negation and the selected
    raw datum. That contradiction is now transported back to positive-integer
    solutions under a selected no-counterexample provider, under explicit
    Frey-model/Wiles-Ribet route laws, and under the corresponding imported
    semistable-modularity, bridge-free level-lowering, level-two obstruction,
    no-counterexample, and global raw-refutation data closures.
    The same direct Frey-model-law
    route now also constructs the pointwise raw primitive Frey route provider
    from explicit primitive-normalization providers, Frey-model laws,
    semistability-from-model, and direct semistable-modularity/no-bridge/Ribet/
    obstruction/no-counterexample route laws, then constructs the same provider
    from imported semistable-modularity data, bridge-free `LevelLoweringData`,
    level-two obstruction, and no-counterexample closures and combines the route-data provider
    with `FermatGlobalRawRefutationData` to construct the stronger
    raw-elimination provider. That route-data provider is now packaged into the
    formula-specialized `FermatGlobalEliminationData` closure and transported
    to the concrete `FermatPositiveIntegerGlobalEliminationData` closure. The
    latest route-data wrapper now wraps the explicit route-data raw-refutation
    contradiction theorem directly with `not_intro`. The public
    `fermat_positive_integer_solution_false` theorem exposes the same
    final-statement contradiction by calling the route-data `False` theorem
    directly, and public `fermat_last_theorem` wraps the explicit public
    contradiction with `not_intro`. The stronger solution-indexed raw-elimination-provider wrapper
    now also uses the bridge-free `LevelLoweringData` route-data surface,
    derives the concrete route-data closure, and wraps the explicit
    `FermatPositiveIntegerGlobalEliminationData` contradiction theorem without
    carrying the bridge-backed `RibetBridgeData` argument bundle, and now also
    exposes the corresponding explicit positive-integer `False` contradiction
    before its final `Not` wrapper. The public
    `fermat_positive_integer_solution_false` and `fermat_last_theorem`
    wrappers now use a bridge-free no-counterexample-data boundary:
    they derive route laws from semistable-modularity data, bridge-free
    `LevelLoweringData`, level-two-obstruction data, and no-counterexample data
    before deriving the explicit contradiction, instead of exposing
    `FermatGlobalRawRefutationData`, the solution-indexed global
    raw-elimination provider, or direct route-law assumptions as public
    prerequisites. The
    concrete positive-integer final wrapper now separates the extracted
    solution-specific raw counterexample contradiction from the `Not` wrapper,
    making the final step consume an explicit `False` theorem derived from
    `FermatPositiveIntegerGlobalEliminationData`. The direct Frey-model-law
    final-statement wrappers now wrap the explicit route-law raw-refutation
    contradiction theorem with `not_intro`, deriving
    `FermatGlobalRawRefutationData` first when starting from the stronger
    solution-indexed raw-elimination provider.
    It packages the same selector/provider families into
    `FermatGlobalEliminationData` and derives the global `Not` theorem from
    that explicit closure through a separate closure-data `False` contradiction
    theorem, now also transporting the closure through
    `FermatPositiveIntegerGlobalEliminationData` to an explicit concrete
    positive-integer solution contradiction and then to
    `Not (FermatPositiveIntegerSolutionData ...)`. The same explicit
    contradiction shape is now available directly from the
    formula-specialized raw-elimination provider before it is wrapped as the
    corresponding final-statement `Not` theorem, and also from the
    raw-primitive-Frey-route provider boundary after constructing that
    raw-elimination provider. The solution-indexed raw-primitive-Frey-route
    boundary now also exposes an explicit concrete positive-integer `False`
    theorem and matching `Not` wrapper without requiring route data for every
    raw counterexample, and it constructs the corresponding solution-indexed
    raw-elimination provider plus `FermatPositiveIntegerGlobalEliminationData`
    closure. The solution-indexed primitive-normalization /
    primitive-Frey-route boundary now constructs that same global elimination
    closure without requiring primitive data for every raw counterexample, and
    the solution-indexed Frey-model / Wiles-Ribet route-data boundary now
    constructs the primitive route data used by that closure. The
    solution-indexed Frey-model provider can now also be built from a selected
    builds-curve provider plus the generic Frey-model laws before packaging the
  same closure. The selected-level-two/raw-contradiction route is now also
  specialized to `Std.Nat`, kernel equality, and `FermatStdNatAtLeastThree`,
  exposing positive-integer, positive-arithmetic, and ordered-field-derived
  positive-arithmetic final-statement wrappers without adding a direct final
  conclusion assumption. The selected no-raw route is likewise specialized to
  `Std.Nat`, kernel equality, and `FermatStdNatAtLeastThree`, including
  ordered-field-derived positive-arithmetic wrappers that obtain
  `Positive -> Nonzero` from the ordered-field bridge before consuming the
  selected no-raw law. The solution-indexed primitive-normalization provider can now
  likewise be built from selected primitive component laws before packaging
  that closure, and the Wiles/Ribet route data can now be constructed at that
  boundary from the individual route laws before deriving both `False` and
  `Not` forms. The raw-indexed public provider surface can now be adapted into
  that solution-indexed route-law boundary by specializing each raw provider to
  the canonical raw datum generated from the concrete positive-integer solution,
  and the bridge-free public surface now exposes the resulting
  `FermatPositiveIntegerGlobalEliminationData` closure directly. The remaining
  no-counterexample data package on that bridge-free boundary can now also be
  reconstructed from its counterexample-contradiction and no-counterexample
  constructor laws before deriving the same data, `False`, and `Not` forms; the
  remaining level-two-obstruction data package on the same boundary can likewise
  be reconstructed from its obstruction and contradiction constructor laws, and
  the bridge-free level-lowering data package can now be reconstructed from its
  dependency-map, level-lowering conclusion, and non-Frey reuse laws. The
  remaining semistable-modularity data package can likewise be reconstructed
  from its reusable-assumptions, modularity-conclusion, semistable-route, and
  no-bridge constructor laws, and the remaining modularity-lifting data package
  can now be reconstructed from its deformation-ring, Hecke-comparison,
  `R_eq_T`, minimal lifting, nonminimal lifting, and non-Frey reuse constructor
  laws. The selected `modularity_conclusion_of_curve` provider can now also be
  derived from curve-indexed residual-irreducible, local-condition,
  minimal-condition, and modularity-assumption providers by applying the
  minimal modularity-lifting law. The reusable semistability assumptions law is
  now discharged by the identity proof
  `FreyCurveSemistable curve -> FreyCurveSemistable curve` instead of remaining
  a public prerequisite, and the semistable modularity route law is similarly
  discharged by returning the already constructed `SemistableModular curve`
  conclusion. The level-lowering non-Frey reuse law and its generic
  terminology/non-Frey marker parameters are no longer public prerequisites for
  the FLT route: the route-level `RibetLevelLowering` law is derived directly
  from the dependency-map and level-lowering conclusion laws plus the selected
  route providers. The deformation-functor, Hecke-comparison, `R_eq_T`,
  nonminimal-lifting, and modularity-lifting non-Frey reuse inputs are likewise
  no longer public prerequisites for this FLT boundary; only the minimal
  modularity-lifting law and the selected residual/local/minimal/modularity
  condition providers remain on that layer. The
  primitive-normalization plus
    primitive-Frey-route boundary now similarly derives the explicit
    contradiction by first constructing the raw-primitive route provider. The
    Frey-model plus Wiles/Ribet route-data boundary now constructs the
    primitive route provider before deriving the same explicit contradiction.
    The primitive-normalization provider boundary now constructs the
    Frey-model provider and route data before deriving the explicit
    contradiction. The selected component-provider boundary likewise
    constructs primitive-normalization first and exposes the same explicit
    contradiction before its final `Not` wrapper. The direct Frey-model-law
    boundary now constructs the primitive-normalization provider, selected
    Frey-model provider, and Wiles/Ribet route data before deriving the
    explicit contradiction. The imported semistable-modularity, bridge-free
    level-lowering, level-two obstruction, and no-counterexample data
    boundaries now also expose explicit concrete positive-integer `False`
    contradictions before their corresponding final `Not` wrappers. The lower
    formula-specialized positive-solution-data negation, raw-elimination-data,
    and positive-solution elimination-provider boundaries likewise expose
    explicit `False` contradictions before their global or `Not` wrappers. The
    public
    `fermat_positive_integer_solution_false` and `fermat_last_theorem`
    prerequisite surfaces no longer expose `RibetBridgeData`,
    `FermatGlobalRawRefutationData`, the global raw-elimination provider, or
    direct semistable-modularity/no-bridge/Ribet/level-two/no-counterexample
    route laws. The positive-solution-data provider boundary and the
    solution-indexed raw-elimination-provider boundary now also expose explicit
    concrete positive-integer `False` theorems before their `Not` wrappers, so
    downstream final-statement wrappers can depend on the contradiction step
    directly. The selected raw counterexample route now derives
    `NoFermatCounterexample (counterexample_of x y z n)` from the selected raw
    counterexample route itself, using `rho_of x y z n` and pointwise selected
    modularity-condition providers rather than global all-curve
    `rho_of_curve`/`frey_galois_of_curve` providers, and the public wrappers
    route through that boundary. The selected-facts layer replaces the
    remaining generic `frey_galois_law` and `route_*_provider` inputs by
    pointwise selected facts for the chosen raw counterexample while still
    deriving `DependencyMap`, `RibetLevelLowering`, `LevelTwoContradiction`,
    `NoFermatCounterexample`, and `Not raw`. The slim selected-facts boundary
    removes the unused primitive-normalization and Frey-model predicate surface
    from the public wrappers. The raw-contradiction boundary derives
    `DependencyMap`, `RibetLevelLowering`, and `LevelTwoContradiction` from
    selected level-lowering facts, then applies a selected raw-contradiction
    law to remove the remaining `Counterexample`/`FreyCurve`,
    no-counterexample, selected builds/Galois, and raw-realization surfaces from
    the public wrappers. The direct level-two tightening removes the public
    `LevelTwoObstruction` bridge by consuming a direct selected level-two
    contradiction law after the derived Ribet lowering step. The next
    dependency-map contradiction boundary removes public `RibetLevelLowering`
    and `level_lowering_conclusion_law` by applying a selected
    dependency-map-to-level-two contradiction law after constructing
    `DependencyMap` from selected level-lowering facts. The route-facts
    boundary removes the public `DependencyMap` predicate and dependency-map
    law by applying a selected route-facts-to-level-two contradiction law after
    producing the selected conductor, residual irreducibility, ramification,
    newform, excluded-case, and lowered-level facts. The selected-level-two
    boundary removes the residual/route-fact predicates and selected route-fact
    providers by consuming a selected
    `LevelTwoContradiction (rho_of x y z n)` provider before applying the raw
    contradiction law. The selected no-raw boundary removes the selected
    Galois/rho/level-two and raw-contradiction surfaces by applying a selected
    `Not (FermatRawCounterexampleData ...)` law to the raw counterexample
    constructed from a positive-integer solution. The remaining blocker for an
    unconditional final theorem is an L2 proof of that selected no-raw law
    itself, without assuming `Not raw` or the final FLT conclusion. The selected
    raw-false boundary replaces that public selected no-raw law with a selected
    raw-counterexample contradiction law and derives `Not raw` by `not_intro`.
    That selected raw-false route is now also specialized to `Std.Nat`, kernel
    equality, and `FermatStdNatAtLeastThree`, including ordered-field-derived
    positive-arithmetic wrappers that obtain `Positive -> Nonzero` from the
    ordered-field bridge before consuming the selected raw-counterexample
    contradiction law.
    The selected arithmetic boundary replaces that public selected raw
    contradiction law with a component law over the raw projections
    (`Positive`, `Nonzero`, `ExponentAtLeastThree`, and the concrete
    `EqualInt (Add (Pow x n) (Pow y n)) (Pow z n)` equation), then uses the raw
    projection theorems to derive `raw -> False`. That selected raw-arithmetic
    route now also exposes positive-arithmetic and ordered-field-derived
    positive-arithmetic wrappers, plus the corresponding `Std.Nat`, kernel
    equality, and `FermatStdNatAtLeastThree` standard surfaces. The positive-arithmetic
    boundary is now also specialized back to the standard raw surface by
    `fermat_raw_counterexample_false_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`
    and
    `fermat_not_raw_counterexample_std_nat_kernel_eq_at_least_three_from_selected_positive_arithmetic_facts`,
    which instantiate the generic selected-positive-arithmetic raw refutation
    at `Std.Nat`, kernel equality, and `FermatStdNatAtLeastThree` without adding
    a duplicate final theorem.
    boundary removes the redundant `Nonzero x`, `Nonzero y`, and `Nonzero z`
    premises from that public contradiction law, using only the positive,
    exponent, and concrete equation projections. The
    positive-arithmetic-solution boundary also introduces
    `FermatPositiveArithmeticSolutionData` and makes the public
    `fermat_last_theorem` conclude
    `Not (FermatPositiveArithmeticSolutionData ...)`, so the public final
    statement no longer quantifies a separate `Nonzero` predicate.
    `fermat_positive_integer_solution_data_from_positive_arithmetic_solution`
    and `fermat_last_theorem_from_positive_integer_refutation` add the reverse
    bridge under an explicit `Positive -> Nonzero` law, allowing this public
    positive-arithmetic shape to connect back to the positive-integer
    route/refutation layer without assuming the final contradiction directly.
    `fermat_positive_nonzero_law_from_ordered_field_bridge`,
    `fermat_positive_integer_solution_data_from_ordered_field_positive_arithmetic_solution`,
    `fermat_last_theorem_from_ordered_field_positive_integer_refutation`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_positive_integer_refutation`
    derive that reverse bridge from the existing ordered-field
    `ordered_field_nonzero_of_positive` theorem plus explicit interpretation
    maps between the FLT `Positive`/`Nonzero` predicates and the ordered-field
    predicates, and also derive the selected positive-arithmetic contradiction
    law from a positive-integer refutation under those inputs.
    `fermat_last_theorem_positive_arithmetic_from_global_raw_elimination_provider`
    composes that bridge with the existing global raw elimination provider
    theorem, yielding the public positive-arithmetic final shape from the
    structured Frey/Wiles/Ribet route-data boundary.
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_global_raw_elimination_provider`
    further synthesize the primitive `Nonzero` provider families from the
    positive primitive providers using the ordered-field-derived
    `Positive -> Nonzero` bridge, so those primitive nonzero providers are no
    longer separate prerequisites at that boundary.
    `fermat_positive_integer_solution_false_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_from_ordered_field_global_raw_elimination_provider`
    transport that ordered-field/global-raw route back to the older
    positive-integer solution surface by eliminating the positive-arithmetic
    projection of a positive-integer solution. Thus the positive-integer
    contradiction boundary also avoids separate primitive `Nonzero` provider
    families when the ordered-field interpretation data is available.
    `fermat_selected_positive_arithmetic_contradiction_law_from_global_raw_elimination_provider`
    also derives the short selected positive-arithmetic contradiction law from
    that same route-data boundary, so the selected law is no longer the only
    standalone public boundary.
    Completed L2 selected-route positive-arithmetic bridge targets derive
    `fermat_positive_arithmetic_solution_false_from_selected_*_facts`,
    `fermat_last_theorem_positive_arithmetic_from_selected_*_facts`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_selected_*_facts`
    for the selected direct level-two, dependency-map contradiction,
    route-facts contradiction, selected level-two, selected no-raw, and
    selected raw-false boundaries. These targets convert a positive-arithmetic
    solution to a positive-integer solution using an explicit `Positive ->
    Nonzero` law before consuming the selected positive-integer refutations;
    they are not aliases for a supplied selected positive-arithmetic law.
    Completed L2 ordered-field selected-route bridge targets derive the
    corresponding `*_from_ordered_field_selected_*_facts` theorems for those
    same six boundaries. These targets remove the explicit `Positive ->
    Nonzero` premise by deriving it from
    `fermat_positive_nonzero_law_from_ordered_field_bridge`, the ordered-field
    law package, the field bridge package, and the interpretation maps from the
    FLT `Positive`/`Nonzero` predicates to the ordered-field predicates.
    `fermat_last_theorem_std_nat_exponent` further specializes this final
    theorem to the repository's `Std.Nat.Basic` exponent carrier while keeping
    the integer carrier and arithmetic operations explicit.
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three` further specializes
    the final theorem to kernel equality `@Eq Int` and the certified
    `FermatStdNatAtLeastThree` exponent predicate.
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three`
    exposes the same selected positive-arithmetic contradiction law as a
    pointwise `False` eliminator for the standard positive-arithmetic solution
    surface. `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three`
    and `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three`
    transport that selected arithmetic contradiction to the standard
    positive-integer solution surface by projecting positive-integer solution
    data to positive-arithmetic data, without assuming the final negation.
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_positive_integer_refutation`
    specializes the same bridge to this `Std.Nat`/kernel-`Eq`/`n >= 3`
    boundary. `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    removes the explicit `Positive -> Nonzero` law from that standard boundary
    by deriving it from the ordered-field bridge, leaving the positive-integer
    refutation and concrete ordered-field interpretation data as the live
    prerequisites there.
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_refutation`
    expose the same standard boundary as a pointwise `False` eliminator and a
    short selected positive-arithmetic contradiction law, respectively, by
    eliminating the certified negation rather than assuming the contradiction.
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    specialize the explicit global-raw route boundary, while still taking the
    nonzero primitive providers and `Positive -> Nonzero` law explicitly, to
    `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree`.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    expose the same standard explicit global-raw route at the positive-integer
    surface; because that record already includes nonzero evidence, this pair
    does not require the extra `Positive -> Nonzero` bridge law.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    packages the same standard explicit global-raw boundary as a reusable
    `FermatPositiveIntegerGlobalEliminationData` closure, so downstream
    positive-integer contradictions can consume the standard closure rather than
    only the pointwise `False`/`Not` wrappers.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    constructs the same standard closure after deriving the primitive nonzero
    provider family from the ordered-field bridge.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    consume the standard closure directly at the positive-integer surface.
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_positive_integer_global_elimination_data`
    transport the same closure to the positive-arithmetic surface using an
    explicit `Positive -> Nonzero` law.
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_positive_integer_global_elimination_data`
    derive that bridge law from the ordered-field data before eliminating the
    positive-arithmetic solution.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    opens the more general `FermatGlobalEliminationData` closure at the standard
    `Nat`/kernel-`Eq` boundary and exposes it as reusable
    `FermatPositiveIntegerGlobalEliminationData`.
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    specialize that same global closure to the formula-level
    `FermatPositiveSolutionData` surface before moving on to the
    positive-integer wrapper. The matching
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_global_elimination_data`
    consume that global closure directly at the positive-integer surface.
    The positive-arithmetic and ordered-field variants ending in
    `_from_global_elimination_data` transport the same closure through either an
    explicit `Positive -> Nonzero` law or the ordered-field bridge, without
    constructing the still-missing concrete global provider families.
    `fermat_global_elimination_data_from_global_normalization_laws_builds_curve_and_route_laws`
    constructs the formula-specialized `FermatGlobalEliminationData` closure at
    the generic raw-indexed global-normalization boundary by building
    `FermatGlobalRawRefutationData` from raw-realization evidence and the
    no-raw-counterexample law.
    `fermat_positive_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`
    and
    `fermat_global_not_positive_solution_from_global_normalization_laws_builds_curve_and_route_laws`
    consume that closure directly at the generic `FermatPositiveSolutionData`
    surface.
    `fermat_positive_arithmetic_solution_false_from_global_normalization_laws_builds_curve_and_route_laws`,
    `fermat_last_theorem_from_global_normalization_laws_builds_curve_and_route_laws`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_global_normalization_laws_builds_curve_and_route_laws`
    transport the same generic closure through an explicit `Positive ->
    Nonzero` law to the positive-arithmetic final-statement surface and the
    selected arithmetic contradiction law.
    `fermat_positive_integer_global_elimination_data_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
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
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`
    constructs the same standard positive-integer closure from primitive
    normalization providers, Frey-model component laws, direct Wiles/Ribet
    route laws, raw-realization evidence, and the no-raw-counterexample law,
    without assuming a monolithic global raw-elimination provider.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`
    exposes the corresponding formula-specialized `FermatGlobalEliminationData`
    closure from the same decomposed global-normalization boundary.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_global_normalization_laws_builds_curve_and_route_laws`,
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
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    builds the same decomposed standard positive-integer closure after deriving
    the primitive `Nonzero` provider families from the ordered-field bridge and
    the positive primitive providers.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`
    exposes the ordered-field version of the formula-specialized
    `FermatGlobalEliminationData` closure.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_normalization_laws_builds_curve_and_route_laws`,
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
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    constructs the standard positive-integer global-elimination closure from
    structured Wiles/Ribet route-data inputs and
    `FermatGlobalRawRefutationData`, replacing a monolithic global
    raw-elimination provider at that boundary.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    exposes the more general formula-specialized `FermatGlobalEliminationData`
    closure at the same standard route-data/raw-refutation boundary, before
    transporting it to the positive-integer closure.
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    consume that formula-level closure directly at the
    `FermatPositiveSolutionData` surface.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_route_data_and_global_raw_refutation_data`
    consume that route-data/raw-refutation closure at the positive-integer and
    positive-arithmetic surfaces.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    opens the `FermatGlobalRawRefutationData` compatibility package at the
    standard route-data boundary by constructing it from explicit
    `realizes_raw_provider` and `no_raw_counterexample_law` components before
    reusing the certified route-data/global-raw-refutation closure.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    exposes the same formula-specialized `FermatGlobalEliminationData`
    closure directly from those explicit raw-refutation components.
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    consume that explicit-component closure at the formula-level positive
    solution surface.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_route_data_raw_realizes_provider_and_no_raw_law`
    consume that explicit raw-realization/no-raw-law route boundary at the same
    standard positive-integer and positive-arithmetic surfaces.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    derives the primitive `Nonzero` provider families from the ordered-field
    bridge and uses the same route-data/raw-refutation closure.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    exposes the corresponding ordered-field-derived
    `FermatGlobalEliminationData` closure before the positive-integer
    transport.
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    consume that ordered-field route-data closure directly at
    `FermatPositiveSolutionData`.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_and_global_raw_refutation_data`
    expose the ordered-field route-data/raw-refutation boundary at the same
    positive-integer and positive-arithmetic surfaces.
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    additionally derives the primitive `Nonzero` provider families from the
    ordered-field bridge while constructing the raw-refutation compatibility
    package from explicit raw-realization/no-raw-law components.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    exposes that ordered-field-derived formula-specialized global closure
    before the concrete positive-integer closure is consumed.
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    consume that ordered-field explicit-component closure at the same
    formula-level surface.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_route_data_raw_realizes_provider_and_no_raw_law`
    expose the ordered-field component-level route boundary at the same
    positive-integer and positive-arithmetic surfaces.
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    specialize the ordered-field/global-raw route-data boundary to
    `Std.Nat.Basic`, kernel equality, and `FermatStdNatAtLeastThree`. This
    removes the abstract `NatCarrier`, `EqualInt`, and `ExponentAtLeastThree`
    parameters from that standard boundary while still requiring concrete
    ordered-field, Frey, modularity, level-lowering, no-counterexample, and
    raw-elimination provider data.
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    and
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    carry the same standard ordered-field/global-raw route back to the older
    positive-integer solution surface by projecting a positive-integer solution
    into `FermatPositiveArithmeticSolutionData` and eliminating it with the
    certified standard route negation.
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`
    and
    `fermat_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    expose the formula-specialized `FermatGlobalEliminationData` closure at
    the same standard global-raw-provider boundaries. The matching
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_global_raw_elimination_provider`,
    `fermat_positive_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`,
    and
    `fermat_not_positive_solution_std_nat_kernel_eq_at_least_three_from_ordered_field_global_raw_elimination_provider`
    consume those closures at the formula-level `FermatPositiveSolutionData`
    surface.
    Completed L2 generic formula-solution closure consumer targets:
    `fermat_positive_solution_false_from_global_elimination_data` and
    `fermat_not_positive_solution_from_global_elimination_data` consume
    the already certified formula-specialized `FermatGlobalEliminationData`
    closure at the `FermatPositiveSolutionData` surface. These wrappers
    eliminate a concrete `FermatPositiveSolutionData` argument through the
    closure and `not_intro`; they are not aliases for a supplied contradiction
    law. Once proved, the route-data / Frey-model-law boundaries can reuse this
    smaller closure consumer instead of duplicating the closure-elimination
    proof.
    Completed L2 generic global-raw-elimination-provider formula-solution
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
    Completed L2 generic route-data formula-solution consumer targets:
    `fermat_positive_solution_false_from_route_data_and_global_raw_refutation_data`
    and
    `fermat_not_positive_solution_from_route_data_and_global_raw_refutation_data`
    construct the route-data/global-raw-refutation
    `FermatGlobalEliminationData` closure and consume it through the generic
    formula-solution closure consumer. These wrappers do not assume a
    positive-solution contradiction law directly.
    Completed L2 generic route-data/global-raw-elimination-provider
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
    Completed L2 direct Frey-model-law/global-raw-elimination-provider
    formula-solution consumer targets:
    `fermat_positive_solution_false_from_frey_model_laws_and_global_raw_elimination_provider`
    and
    `fermat_not_positive_solution_from_frey_model_laws_and_global_raw_elimination_provider`
    expose the provider-level formula-solution contradiction at the explicit
    Frey-model/route-law boundary by constructing the formula-specialized
    `FermatGlobalEliminationData` closure and eliminating the concrete
    `FermatPositiveSolutionData` argument, without adding a positive-solution
    contradiction law.
    Completed L2 direct Frey-model-law/global-raw-elimination-provider
    positive-arithmetic wrapper targets:
    `fermat_positive_arithmetic_solution_false_from_frey_model_laws_and_global_raw_elimination_provider`,
    `fermat_last_theorem_positive_arithmetic_from_frey_model_laws_and_global_raw_elimination_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_frey_model_laws_and_global_raw_elimination_provider`
    move the same boundary to the public positive-arithmetic solution shape by
    converting a positive-arithmetic solution to the positive-integer solution
    record under an explicit `Positive -> Nonzero` law and then eliminating it
    through the certified Frey/provider theorem.
    Completed L2 direct Frey-model-law/raw-refutation component targets:
    `fermat_positive_arithmetic_solution_false_from_frey_model_laws`,
    `fermat_last_theorem_positive_arithmetic_from_frey_model_laws`, and
    `fermat_selected_positive_arithmetic_contradiction_law_from_frey_model_laws`
    move the public positive-arithmetic surface to the decomposed Frey/model
    route that uses explicit raw-realization and no-raw-counterexample
    components instead of a monolithic global raw-elimination provider.
    Completed L2 standard direct Frey-model-law/raw-refutation component
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
    The ordered-field variants
    `fermat_positive_arithmetic_solution_false_from_ordered_field_frey_model_laws`,
    `fermat_last_theorem_positive_arithmetic_from_ordered_field_frey_model_laws`,
    `fermat_selected_positive_arithmetic_contradiction_law_from_ordered_field_frey_model_laws`,
    `fermat_positive_integer_solution_false_from_ordered_field_frey_model_laws`,
    and
    `fermat_last_theorem_positive_integer_from_ordered_field_frey_model_laws`
    derive both the primitive `Nonzero` providers and the final `Positive ->
    Nonzero` law from the ordered-field bridge before consuming that decomposed
    Frey/model route.
    Completed L2 standard ordered-field Frey-model-law/raw-refutation
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
    Completed L2 ordered-field direct Frey-model-law/global-raw-elimination-provider
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
    Completed L2 standard ordered-field direct Frey-model-law/global-raw-elimination-provider
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
    Completed L2 ordered-field Frey/provider formula-closure targets:
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
    Completed L2 Frey-model-law/raw-refutation formula-closure targets:
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
    Completed L2 route-data/raw-realization formula-closure targets:
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
    Completed L2 route-data/raw-realization public-surface wrapper targets:
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
    Completed L2 bridge-free no-counterexample-data positive-arithmetic and
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
    Completed L2 ordered-field bridge-free no-counterexample-data
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
    Completed L2 standard ordered-field bridge-free no-counterexample-data
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_data_bridge_free`.
    Completed L2 bridge-free no-counterexample-law positive-arithmetic
    public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_no_counterexample_laws_bridge_free`.
    Completed L2 ordered-field bridge-free no-counterexample-law public-surface
    targets:
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
    Completed L2 standard bridge-free no-counterexample-law public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_no_counterexample_laws_bridge_free`.
    Completed L2 standard ordered-field bridge-free no-counterexample-law
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_no_counterexample_laws_bridge_free`.
    Completed L2 bridge-free level-two-obstruction-law positive-arithmetic
    public-surface targets:
    `fermat_positive_arithmetic_solution_false_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_arithmetic_from_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_from_level_two_obstruction_laws_bridge_free`.
    Completed L2 standard bridge-free level-two-obstruction-law
    public-surface targets:
    `fermat_positive_integer_global_elimination_data_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_integer_solution_false_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_positive_integer_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_level_two_obstruction_laws_bridge_free`.
    Completed L2 bridge-free level-lowering-law positive-arithmetic and
    standard public-surface targets:
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
    Completed L2 bridge-free level-lowering-core-law positive-arithmetic
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
    Completed L2 minimal-modularity-lifting-core positive-arithmetic and
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
    Completed L2 semistable-modularity-law positive-arithmetic and
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
    Completed L2 modularity-lifting-law positive-arithmetic and standard
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
    Completed L2 modularity-conclusion-law positive-arithmetic and standard
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
    Completed L2 semistable-assumptions-identity positive-arithmetic and
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
    Completed L2 semistable-route-identity positive-arithmetic and standard
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
    Completed L2 decomposed-provider positive-arithmetic public-surface targets:
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
    Completed L2 ordered-field decomposed-provider public-surface targets:
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
    Completed L2 standard ordered-field raw-primitive-provider public-surface
    targets:
    `fermat_positive_arithmetic_solution_false_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`,
    `fermat_last_theorem_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`,
    and
    `fermat_selected_positive_arithmetic_contradiction_law_std_nat_kernel_eq_at_least_three_from_ordered_field_raw_primitive_frey_route_provider`.
    Completed L2 standard ordered-field decomposed-provider public-surface
    targets:
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
    The remaining blockers for an unconditional final theorem are
    concrete L2 constructions of the ordered-field bridge/interpretation data
    yielding `Positive -> Nonzero` for the concrete integer positivity
    predicate, the Frey, modularity-lifting, semistable-modularity,
    Ribet/level-lowering, no-raw-counterexample, primitive-normalization,
    global raw elimination, and arithmetic provider families, without assuming
    the final contradiction, so the final FLT theorem is not complete.
- Theorem order:
  1. primitive counterexample data and projections;
  2. exponent reduction to prime exponent or exponent `4`;
  3. primitive counterexample normalization by gcd/descent;
  4. Frey curve construction from a primitive counterexample;
  5. Frey discriminant, conductor, minimal model, and semistability;
  6. mod-`p` Galois representation attached to the Frey curve;
  7. bridge-free semistable modularity and Ribet level lowering;
  8. level-two contradiction and final no-counterexample theorem.
- Proof strategy:
  - Keep all Frey-specific facts in the FLT route module or a dedicated
    submodule, not in reusable elliptic-curve APIs.
  - Add source/certificate artifacts only for L2 derivations whose prerequisites
    are already explicit and verified.
  - Record missing Wiles/Ribet/Frey prerequisites in roadmap tasks instead of
    emitting L1 theorem-shaped source.
- Acceptance criteria:
  - No final FLT theorem is present until every prerequisite in the final import
    closure has a source-free L2 certificate.
  - No `BridgeAxiom`, bridge-backed Ribet theorem, or statement-only Wiles
    assumption appears in the final theorem import closure.
  - Promotion is not part of this route.

## Recommended Initial Execution Queue

1. `NT-00`: theorem cards, duplicate-home map, and conjecture labels.
2. `NT-01`: integer and divisibility statement freeze.
3. `NT-02`: Euclidean division, gcd, lcm, Euclid algorithm, and Bezout.
4. `NT-03`: primes, Euclid lemma, prime factorization, and prime infinitude.
5. `NT-04`: congruence algebra, residue rings, linear congruences, and CRT.
6. `NT-05`: unit groups, Fermat, Euler, Wilson, Carmichael, and RSA
   correctness.
7. `NT-06`: primitive roots, characters, and Gauss sums.
8. `NT-07`: Legendre symbol and quadratic reciprocity route.
9. `NT-08`: arithmetic functions, Dirichlet convolution, and Mobius inversion.
10. `NT-09`: continued fractions and Pell, if ordered-field and real
   prerequisites are ready.
11. `NT-13`: algebraic integer statement surfaces once ring and module
    foundations are stable.

This queue intentionally delays analytic number theory, class field theory,
elliptic curves, modular forms, and Langlands interfaces until elementary
arithmetic, quotient, finite-group, and algebraic foundations are reusable.

## Cross-Roadmap Ownership

| Theorem family | Primary roadmap | Number-theory role |
| --- | --- | --- |
| abstract ring/group/field theorems | existing `Proofs.Ai.Algebra.*` modules, `develop/proof-corpus-field-theory-roadmap.md`, and `proofs/linear-algebra-theorem-proof-roadmap.md` where finite-dimensional structure is needed | import and specialize |
| Chinese remainder theorem over abstract rings | `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | specialize to integer residue rings |
| UFD prime factorization | `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization` and `develop/proof-corpus-field-theory-roadmap.md` | specialize to natural/integer prime factorization |
| finite-field core laws, Frobenius, and root characterization | `Proofs.Ai.Algebra.AbstractFiniteField` through `develop/proof-corpus-field-theory-roadmap.md` | import or alias; add number-theoretic applications |
| real, complex, series, integration, Tauberian theorems | analysis and measure roadmaps | import for analytic number theory |
| topological compactness and local compactness | topology roadmap | import for local fields and harmonic analysis |
| scheme and derived algebraic geometry foundations | algebraic-geometry modules and future roadmap | import for arithmetic geometry |
| cryptographic security assumptions | future cryptography roadmap or theorem cards | state assumptions; prove algebraic correctness only |

## Risk Register

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| proving high-level theorems before elementary arithmetic is stable | duplicated private definitions and incompatible statements | freeze `Divides`, `Gcd`, `PrimeNat`, `Congruent`, and `ResidueRing` first |
| treating conjectures as theorem targets | false proof-roadmap status | exclude conjectures from proof-corpus declarations; record only roadmap exclusions or named conditional assumptions |
| hiding quotient assumptions inside residue rings or class groups | checker-policy surprises | expose quotient core features and package axiom reports |
| using analytic number theory to justify elementary prime facts | circular dependency graph | keep elementary primes and FTA before zeta, `L`-functions, and Chebotarev |
| large theorem interfaces becoming permanent bridge axioms | untrusted final theorem claims | use namespaced bridge assumptions and reject them in promotion gates |
| duplicate theorem ownership across elliptic curves, modularity, and Langlands | inconsistent APIs and import cycles | keep general APIs outside specialized final-theorem routes |

## Acceptance Gates

For a single theorem module:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring```

For a coherent authoring batch:

```sh
./scripts/check-corpus-authoring.sh
```

For package-wide or promotion work:

```sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

Document-only roadmap edits should at least pass:

```sh
git diff --check
```

Also search for unresolved local markers if an editor or generator inserted
any during drafting.
