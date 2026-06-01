# Fermat's Last Theorem Proof Phase Breakdown

This plan records a certificate-first route for starting a Fermat's Last Theorem
project in NPA.

The target is not to mark a source script, AI transcript, or imported paper
summary as trusted. The final claim is accepted only when the NPA certificate
for the final theorem and its full import closure are checked by the kernel and
the independent checker under a high-trust axiom policy.

## Scope

The final theorem target is the standard natural-number form:

```text
fermat_last_theorem:
  forall n a b c : Nat,
    2 < n ->
    a != 0 ->
    b != 0 ->
    c != 0 ->
    a ^ n + b ^ n = c ^ n ->
    False
```

This block is mathematical display notation, not the canonical certificate
payload. FLT0 must freeze the exact NPA surface statement and verify that
numeric literals, exponentiation, order, inequality, nonzero hypotheses, and
`False` elaborate to explicit certified constants such as `Nat.zero`,
`Nat.succ`, relation predicates, and `P -> False`-style negation before any
certificate is accepted.

Equivalent integer, positive-natural, and pairwise-coprime variants may be
exported, but this theorem is the public final target unless a later statement
freeze explicitly replaces it.

The mathematical route follows the modern Wiles/Taylor-Wiles proof:

```text
hypothetical FLT counterexample
  -> reduce to a primitive prime-exponent counterexample
  -> construct the Frey elliptic curve
  -> prove the Frey curve is semistable and has the required mod-p representation
  -> apply Ribet level lowering to show this curve cannot be modular
  -> apply semistable modularity to show this curve is modular
  -> contradiction
```

The project should initially build conditional, certificate-backed glue
theorems around named interfaces for Frey, Ribet, and semistable modularity.
Those interfaces are useful scaffolding, but the final theorem is not complete
until every such interface has been replaced by checked NPA certificates or by
an explicitly allowed foundation primitive.

## Trust Policy

Important constraints:

```text
- Do not add `FLT`, `Ribet`, `Wiles`, `modularity`, or `level lowering` as a
  trusted kernel primitive.
- Do not accept a theorem just because a paper, source script, tactic replay,
  external solver, or AI trace says it is true.
- All final proof evidence must be canonical `.npcert` bytes plus checker
  verdicts.
- Development-only bridge axioms must use names under `Flt.BridgeAxiom.*` and
  must be rejected by the final high-trust policy.
- The final package policy should have no custom axioms. If the project decides
  to rely on a standard foundation principle, such as a classical-choice
  package or quotient primitive, that dependency must be named, allowlisted,
  documented, and visible in the axiom or core-feature report.
- Source, replay, metadata, theorem graph scores, and AI sidecars remain
  non-trusted sidecars.
```

Development may use a bridge-axiom package to test statement shapes:

```text
Proofs.Ai.NumberTheory.Flt.Bridge
  Flt.BridgeAxiom.ribet_level_lowering
  Flt.BridgeAxiom.semistable_modularity
```

These bridge axioms are allowed only in early smoke certificates. They must not
appear in the final `fermat_last_theorem` axiom report.

## Repository Layout

Use narrow modules rather than one giant `FLT` certificate.

```text
Proofs.Ai.NumberTheory.Elementary
Proofs.Ai.NumberTheory.PrimeExponent
Proofs.Ai.NumberTheory.Flt.Statement
Proofs.Ai.NumberTheory.Flt.Reduction
Proofs.Ai.EllipticCurve.Basic
Proofs.Ai.EllipticCurve.Frey
Proofs.Ai.ModularForms.Basic
Proofs.Ai.GaloisRepresentation.Basic
Proofs.Ai.Modularity.Ribet
Proofs.Ai.Modularity.Semistable
Proofs.Ai.NumberTheory.Flt.Final
```

If a module is still abstract or bridge-backed, encode that in the module name
or metadata. A reader should be able to distinguish:

```text
checked theorem layer
bridge-interface layer
development smoke layer
```

without opening the proof term.

## Milestones

### FLT0: Project Contract And Statement Freeze

Status: planned.

Deliverables:

- Create the public theorem names for natural-number, positive-natural, and
  integer formulations.
- Define the final axiom policy and the development bridge-axiom policy.
- Add package-level gates that fail if `Flt.BridgeAxiom.*` appears in a release
  certificate.
- Record the exact high-trust checker command expected for final adoption.

Acceptance criteria:

- The final theorem statement is stable enough for downstream modules to target.
- The development and release trust policies are distinct.
- `fermat_last_theorem` cannot be released with bridge axioms in its transitive
  axiom report.

Verification:

```sh
cargo test -p npa-package
cargo test -p npa-checker-ref high_trust
```

### FLT1: Elementary Arithmetic Foundation

Status: planned.

Goal:

- Build the elementary number-theory layer needed to reduce FLT to primitive
  prime exponents.

Deliverables:

- Natural-number exponentiation API and basic exponent laws.
- Divisibility, gcd, coprimality, primality, and prime factor extraction.
- Integer-positive and natural-positive translation lemmas.
- Reduction:

```text
counterexample at exponent n > 2
  -> counterexample at exponent 4 or at an odd prime p
```

- Separate handling of the classical exponent-4 case.

Acceptance criteria:

- No custom axiom is used.
- The prime-exponent reduction is exported as a checked theorem independent of
  elliptic curves or modularity.
- Statement variants are connected by checked equivalence lemmas.

Needed NPA/library work:

- More complete `Nat`, `Int`, and finite descent support.
- Decidable equality/order APIs that remain ordinary certified library facts,
  not kernel plugins.

### FLT2: Algebraic Infrastructure

Status: planned.

Goal:

- Build the algebraic vocabulary needed by elliptic curves, modular forms, and
  Galois representations.

Deliverables:

- Explicit law packages for commutative rings, fields, ideals, modules,
  algebras, finite-dimensional vector spaces, and finite groups.
- Polynomial rings and rational functions over explicit fields.
- Quotients for groups, rings, and modules using the existing quotient profile
  only when the checker supports the required core features.
- Finite sums/products and finite-indexed matrices.

Acceptance criteria:

- The algebra layer does not rely on a direct theorem-shaped law whose
  conclusion is one of the later FLT bridge theorems.
- Quotient dependencies are visible through the certificate core-feature
  report.
- All reusable algebra facts are exported under theorem-search-friendly names.

### FLT3: Elliptic Curves And The Frey Curve

Status: planned.

Goal:

- Formalize enough elliptic-curve theory over `Q` to state and use the Frey
  curve construction.

Deliverables:

- Weierstrass model API over `Q`.
- Discriminant, conductor, reduction type, and semistability predicates.
- The Frey curve attached to a primitive prime-exponent counterexample:

```text
E_{a,b,p}: y^2 = x (x - a^p) (x + b^p)
```

- Checked theorem:

```text
primitive_prime_counterexample a b c p
  -> frey_curve_semistable (Frey a b p)
```

- Interface for the mod-`p` Galois representation attached to an elliptic
  curve.

Acceptance criteria:

- The Frey curve facts are proved from elementary arithmetic and elliptic-curve
  definitions, not asserted as a final bridge axiom.
- Any temporary local arithmetic lemmas are named as local bridge assumptions
  and tracked in the axiom report.

### FLT4: Modular Forms, Modular Curves, And Modularity Interfaces

Status: planned.

Goal:

- Establish the formal vocabulary needed to connect elliptic curves to modular
  forms.

Deliverables:

- Modular forms/newforms interface at the level needed for elliptic curves over
  `Q`.
- Modular elliptic curve predicate.
- Hecke operator and Hecke algebra interfaces.
- Modular Galois representation predicate.
- Bridge theorem shape:

```text
modular_elliptic_curve E
  -> modular_galois_representation (rho_E p)
```

Acceptance criteria:

- The interface separates definitions, bridge assumptions, and checked
  transport lemmas.
- The final contradiction theorem can refer to `modular_elliptic_curve E`
  without depending on source-level notation or typeclass search.

### FLT5: Ribet Level-Lowering Route

Status: planned.

Goal:

- Formalize the theorem path that a Frey curve from a primitive FLT
  counterexample is incompatible with modularity.

Deliverables:

- Statement of Ribet's level-lowering theorem in the exact form needed by the
  Frey curve.
- Checked glue theorem:

```text
primitive_prime_counterexample a b c p
  -> modular_elliptic_curve (Frey a b p)
  -> False
```

provided the Ribet theorem interface is available.

- Dependency map from the Ribet statement to required objects:
  conductors, mod-`p` representations, irreducibility, ramification, level
  lowering, and the excluded low-level modular form case.

Acceptance criteria:

- Early versions may use `Flt.BridgeAxiom.ribet_level_lowering`, but the module
  must expose the exact axiom dependency.
- The bridge theorem may be adopted as a milestone result only with a clear
  label such as "conditional on Ribet".
- Final FLT cannot depend on the bridge axiom.

### FLT6: Semistable Modularity Route

Status: planned.

Goal:

- Formalize the Wiles/Taylor-Wiles theorem that every semistable elliptic curve
  over `Q` is modular, at least in the form needed for the Frey curve.

Deliverables:

- Galois representation and deformation functor interfaces.
- Universal deformation rings and local deformation condition interfaces.
- Hecke algebra interface and `R = T` theorem shape.
- Minimal modularity lifting theorem.
- Non-minimal lifting steps needed for semistable curves.
- Final semistable modularity theorem:

```text
semistable_elliptic_curve_over_Q E
  -> modular_elliptic_curve E
```

Acceptance criteria:

- The theorem is decomposed into independently checkable certificates with
  explicit import hashes.
- Ring-theoretic complete-intersection/Gorenstein assumptions are not hidden in
  automation or external solver output.
- The theorem can be checked by the reference checker under a policy that
  disallows `Flt.BridgeAxiom.*`.

Pragmatic staging:

1. Start with the theorem as a bridge axiom only to validate the final glue.
2. Replace it with a sequence of certificate-backed modularity-lifting modules.
3. Keep every remaining unproved deep lemma visible as a named bridge axiom until
   it is replaced.

### FLT7: Final Contradiction Layer

Status: planned.

Goal:

- Combine the elementary reduction, Frey construction, Ribet route, and
  semistable modularity into the final theorem.

Deliverables:

- Checked theorem:

```text
primitive_prime_counterexample a b c p -> False
```

- Checked theorem:

```text
fermat_last_theorem
```

- Compatibility aliases for the integer and positive-natural statements.

Acceptance criteria:

- The final theorem has no dependency on `Flt.BridgeAxiom.*`.
- The axiom report is empty except for explicitly approved foundation
  dependencies.
- `certificate_hash`, `export_hash`, and `axiom_report_hash` are pinned in the
  package manifest.

### FLT8: Independent Checker And Release Audit

Status: planned.

Goal:

- Make the final theorem externally auditable without source, replay, or AI
  sidecars.

Deliverables:

- A package manifest that pins every import certificate hash.
- A high-trust reference-checker run in dependency-topological order.
- A release audit bundle containing only canonical certificates, checker
  requests/results, hashes, and policy.
- A negative audit fixture proving that injecting a bridge axiom causes release
  rejection.

Acceptance criteria:

- The final result is reproducible from checked-in canonical artifacts.
- The release gate does not read `.npa` source, tactic scripts, theorem graph
  scores, model output, or paper summaries as proof evidence.

Verification:

```sh
./scripts/phase8-release-audit.sh
cargo test --workspace
```

## First Practical Slice

The first useful slice should be deliberately smaller than the full theorem:

```text
FLT0
  -> FLT1 prime-exponent reduction
  -> FLT3 Frey curve statement interface
  -> FLT5/FLT6 bridge-backed contradiction
  -> FLT7 conditional final theorem with visible bridge axioms
```

This produces a certificate-backed statement of the form:

```text
fermat_last_theorem_conditional:
  RibetInterface ->
  SemistableModularityInterface ->
  fermat_last_theorem
```

or, during early smoke testing:

```text
fermat_last_theorem
  with axiom report:
    Flt.BridgeAxiom.ribet_level_lowering
    Flt.BridgeAxiom.semistable_modularity
```

The second form must be displayed as bridge-backed and not as a completed proof.
Its value is engineering: it freezes the final theorem shape, import graph,
certificate payload, and release gate before the deep mathematics is formalized.

## Feature Gaps To Track

The FLT project is larger than the current proof corpus. Track these gaps as
first-class work items:

| Area | Why it matters |
| --- | --- |
| `Int`, `Rat`, divisibility, gcd, primes | elementary FLT statement and exponent reduction |
| finite sets, finite sums, matrices | modular forms, Hecke operators, representation matrices |
| robust quotient support | quotient groups/rings/modules and modular curve constructions |
| subobjects and ideals | rings, modules, Hecke algebras, deformation rings |
| polynomial and fraction fields | elliptic-curve equations and rational functions |
| algebraic extensions and Galois groups | Galois representations and deformation theory |
| topology/completion for local rings | deformation rings and complete Noetherian local algebras |
| scalable theorem graph | searching a library of this size without trusting search output |
| external checker profile | high-trust release beyond the in-process Rust verifier |

## Done Definition For The Full Project

The FLT project is complete only when all of the following hold:

- `Proofs.Ai.NumberTheory.Flt.Final.fermat_last_theorem` is exported.
- Its `.npcert` verifies with `npa-checker-ref` in high-trust mode.
- Every transitive import is checked by the same checker. After standalone
  external checker integration exists, an approved external checker profile with
  pinned hashes may be added as an additional release requirement.
- The final axiom report contains no `Flt.BridgeAxiom.*`, no `sorry`, and no
  unapproved custom axiom.
- The final package manifest pins source hashes, certificate file hashes,
  export hashes, axiom-report hashes, and certificate hashes.
- The repository has a negative test showing that replacing a deep theorem with
  a bridge axiom prevents final release.

## References

These references guide the mathematical decomposition. They are not proof
evidence for NPA unless their contents are formalized into canonical
certificates.

- Andrew Wiles, "Modular elliptic curves and Fermat's Last Theorem",
  Annals of Mathematics 141 (1995), 443-551.
  <https://annals.math.princeton.edu/1995/141-3/p01>
- Richard Taylor and Andrew Wiles, "Ring-theoretic properties of certain Hecke
  algebras", Annals of Mathematics 141 (1995), 553-572.
  <https://annals.math.princeton.edu/1995/141-3/p02>
- Kenneth A. Ribet, "On modular representations of Gal(.../Q) arising from
  modular forms", Inventiones mathematicae 100 (1990), 431-476.
  <https://eudml.org/doc/143793>
- Gary Cornell, Joseph H. Silverman, and Glenn Stevens, editors, "Modular Forms
  and Fermat's Last Theorem", Springer, 1997.
  <https://link.springer.com/book/10.1007/978-1-4612-1974-3>
- Kenneth A. Ribet, "Galois representations and modular forms", Bulletin of the
  AMS 32 (1995), 375-402.
  <https://arxiv.org/abs/math/9503219>
