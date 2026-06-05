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
| `L1 Evidence package` | conclusion follows from an explicit construction, interface, or named external boundary | only if explicitly marked as an interface milestone |
| `L2 Derived certificate` | conclusion is derived from previously certified definitions and lemmas without assuming the conclusion itself | yes |
| `L3 Public closure` | stable theorem promoted or materialized into `npa-mathlib` with package checks | yes |

Very large classical results such as the prime number theorem, Dirichlet's
theorem, class field theory, Faltings' theorem, modularity, and the Langlands
correspondence may first land as `L1` interfaces. Conjectures from the
inventory, including the Riemann hypothesis, generalized Riemann hypothesis,
Birch and Swinnerton-Dyer, Artin conjecture, Fontaine-Mazur, and broad
Langlands functoriality, must remain `L0` statement surfaces unless the project
explicitly chooses a conditional theorem form.

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
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --changed-only
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
- Conjectures are statement objects or assumptions for conditional theorems,
  not proved theorems.

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
  7. Euler product statement interface for multiplicative Dirichlet series.
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
  approximation, simultaneous approximation, Liouville/Roth/Schmidt L1
  interfaces, Khintchine and Duffin-Schaeffer metric-measure prerequisites,
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
  L1 interfaces for Cauchy-Davenport, Kneser, Vosper, Freiman, Plunnecke-Ruzsa,
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
    defining L1 interfaces for Dirichlet series, abscissa of convergence, algebraic
    Euler product, analytic continuation, and Tauberian inputs.
  - `NT-T32` added certificate-backed `Proofs.Ai.NumberTheory.Zeta`
    defining L1 interfaces for Riemann Zeta function, half-plane Euler product,
    analytic continuation, functional equation, zero-free region, Riemann-von Mangoldt
    zero count, explicit formula, and Riemann hypothesis conditional consequence.
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
  5. analytic continuation and functional equation statement interfaces;
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
  - Use `L1` interfaces for analytic continuation and zero-free regions until
    complex analysis and measure/integration prerequisites are certified.
- Acceptance criteria:
  - The prime number theorem is not an input to elementary prime facts.
  - Riemann hypothesis and generalized Riemann hypothesis remain conjectural
    statements or assumptions for conditional theorems.

## NT-12 Sieve Methods And Circle Method

- Status: planned.
- Depends on: `NT-08`, `NT-11`, finite-sum and asymptotic-analysis APIs.
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

- Status: planned.
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

## NT-14 Local Fields And p-adic Analysis

- Status: planned.
- Depends on: `NT-13`, topology, metric completion, and valuation interfaces.
- Target modules:
  - `Proofs.Ai.NumberTheory.Valuation`
  - `Proofs.Ai.NumberTheory.Padic`
  - `Proofs.Ai.NumberTheory.LocalField`
  - `Proofs.Ai.NumberTheory.Hensel`
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

- Status: planned.
- Depends on: `NT-13`, `NT-14`, group cohomology, ideles, and Galois
  extensions.
- Target modules:
  - `Proofs.Ai.NumberTheory.ClassField.Local`
  - `Proofs.Ai.NumberTheory.ClassField.Global`
  - `Proofs.Ai.NumberTheory.ArtinReciprocity`
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

- Status: planned.
- Depends on: existing `Proofs.Ai.Algebra.*` modules, `NT-13`, local fields,
  and finite fields.
- Target modules:
  - `Proofs.Ai.EllipticCurve.Basic`
  - `Proofs.Ai.EllipticCurve.GroupLaw`
  - `Proofs.Ai.EllipticCurve.Reduction`
  - `Proofs.Ai.EllipticCurve.Semistable`
  - `Proofs.Ai.EllipticCurve.Height`
  - `Proofs.Ai.EllipticCurve.FiniteField`
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
  9. modularity, Gross-Zagier, Kolyvagin, Sato-Tate, and BSD statement or
     conditional theorem surfaces.
- Proof strategy:
  - Keep general elliptic-curve APIs independent of specialized Frey-curve
    routes.
  - Keep final-theorem-specific glue out of the reusable elliptic-curve APIs.
- Acceptance criteria:
  - Group law theorem does not rely on modularity or arithmetic geometry
    bridge axioms.
  - BSD remains a conjecture statement or conditional assumption.

## NT-17 Modular Forms And Modularity

- Status: planned.
- Depends on: complex analysis, linear algebra, representation theory, algebraic
  geometry, and `NT-16`.
- Target modules:
  - `Proofs.Ai.ModularForms.Basic`
  - `Proofs.Ai.ModularForms.Hecke`
  - `Proofs.Ai.ModularForms.QExpansion`
  - `Proofs.Ai.ModularForms.ModularCurve`
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
- Depends on: `NT-11`, `NT-15`, `NT-17`, Galois representations, automorphic
  representation interfaces, and trace formula prerequisites.
- Target modules:
  - `Proofs.Ai.NumberTheory.LFunction`
  - `Proofs.Ai.NumberTheory.ArtinL`
  - `Proofs.Ai.NumberTheory.HeckeL`
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
- Depends on: existing `Proofs.Ai.AlgebraicGeometry.*` modules, `NT-13`,
  `NT-16`, `NT-18`, scheme and cohomology foundations.
- Target modules:
  - `Proofs.Ai.ArithmeticGeometry.RationalPoints`
  - `Proofs.Ai.ArithmeticGeometry.Schemes`
  - `Proofs.Ai.ArithmeticGeometry.EtaleCohomology`
  - `Proofs.Ai.ArithmeticGeometry.WeilConjectures`
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
- Depends on: `NT-04`, `NT-05`, `NT-16`, finite-field and algorithmic
  correctness APIs.
- Target modules:
  - `Proofs.Ai.NumberTheory.Algorithm`
  - `Proofs.Ai.NumberTheory.PrimalityTest`
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
| treating conjectures as theorem targets | false proof-roadmap status | label conjectures `L0` or conditional theorem assumptions |
| hiding quotient assumptions inside residue rings or class groups | checker-policy surprises | expose quotient core features and package axiom reports |
| using analytic number theory to justify elementary prime facts | circular dependency graph | keep elementary primes and FTA before zeta, `L`-functions, and Chebotarev |
| large theorem interfaces becoming permanent bridge axioms | untrusted final theorem claims | use namespaced bridge assumptions and reject them in promotion gates |
| duplicate theorem ownership across elliptic curves, modularity, and Langlands | inconsistent APIs and import cycles | keep general APIs outside specialized final-theorem routes |

## Acceptance Gates

For a single theorem module:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.NumberTheory.X
cargo run -p npa-proof-corpus -- --changed-only
```

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
