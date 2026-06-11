# Phase 9 Human Profile: Advanced Features

This document is the detailed design for **Phase 9 Human Profile: Advanced
Features**. Phase 9 extends the small safe proof assistant built so far into a
platform suitable for practical mathematics, program verification, and AI proof
search.

Scope:

```text
- advanced inductive
- quotient
- typeclass
- stronger universe polymorphism
- SMT certificates
- theorem graph
- natural language formalization
```

Implementation / completion notes (2026-05-25):

```text
- This document describes the user-facing / kernel-facing completion state of the Phase 9 Human Profile.
- In the current repository, the Human target scope P9H-00 through P9H-15 is implemented.
- The Phase 9 AI deterministic validation / replay substrate and M9 fixture matrix are also implemented,
  but they are an untrusted Machine Profile that routes advanced-feature candidates back to checker boundaries.
  They do not replace the Human Profile kernel/checker-facing rules.
- P9H-00 boundary regressions are fixed by:
  p9h00_advanced_ai_sidecars_scores_and_smt_outputs_stay_untrusted
  p9h00_ai_fast_path_request_shapes_exclude_phase9_human_heavy_checks
- Production LLM / RAG / online theorem graph store / external SMT solver service operation remain target integration.
- Phase 9 regression is fixed by ./scripts/phase9-regression.sh.
  GitHub Actions workflows have been removed from the current repository, so this gate is run locally as needed.
```

Phase 9 policy:

```text
In the kernel:
  advanced inductive rules
  quotient primitive, if adopted
  universe polymorphism checking

Outside the kernel:
  typeclass search
  SMT solver itself
  theorem graph
  natural language formalizer
  AI model
```

Lean likewise treats inductive types, quotients, and universes as core
facilities, while typeclass search is elaboration machinery rather than kernel
proof checking. ([Lean Language][1])

---

# 1. Overview

Phase 9 expands expressivity, automation, trust, and searchability together.

Areas:

```text
Phase 9.1  advanced inductive
  indexed / mutual / nested inductives

Phase 9.2  stronger universe polymorphism
  real polymorphic library support

Phase 9.3  typeclass
  algebraic hierarchy and overloaded notation

Phase 9.4  quotient
  quotient sets, quotient groups/rings, equivalence classes

Phase 9.5  SMT certificates
  reconstruct and check solver results as proof certificates

Phase 9.6  theorem graph
  library-wide graph for search, recommendation, and learning

Phase 9.7  natural language formalization
  formal statement candidates from natural language / LaTeX
```

Implementation order is section 10, not the numbering above. Universe
polymorphism and advanced inductives come first because they are foundations for
typeclass, quotient, and large libraries.

## 1.1 AI Hot Path Performance Boundary

Heavy Phase 9 checks must not slow the normal AI candidate-generation path.

```text
AI candidate hot path:
  bounded typeclass search
  precomputed theorem graph snapshot query
  Phase 9 AI deterministic candidate validation
  lightweight verify returning to Phase 5-7 replay / fast kernel

release / audit / adoption path:
  full independent checker
  external checker profile
  theorem graph extraction from certificates
  SMT proof certificate checking / NPA proof reconstruction
  quotient-capable checker support
  high-trust release audit
```

Theorem graphs are queried from deterministic snapshots built during build /
release / index update, not extracted from all certificates for each candidate.
SMT solvers, quotient primitives, full independent checking, and release audit
run at proof adoption or high-trust/release boundaries, not inside ranking or
candidate enumeration loops.

P9H-00 tests ensure Phase 5-7 replay / verify request candidate hashes and state
fingerprints do not include Phase 9 Human metadata.

## 1.2 Release Gate vs Target Integration

Required completion gate:

```text
./scripts/phase9-regression.sh
```

Gate contents:

```text
Phase 9 AI M9 deterministic fixture matrix
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

This gate confirms release / high-trust pass/fail depends only on checker
results and deterministic artifacts. AI sidecars, theorem graph scores,
formalization confidence, and SMT solver output are diagnostic / search / audit
metadata outside the trusted boundary.

Production AI orchestrator, LLM/RAG integration, online graph store, full
external SMT solver support, and nonempty solver-native SMT success profiles
remain target integration. They must not be added synchronously to the
regression gate or PR AI candidate enumeration in a way that slows the AI hot
path.

---

# 2. Advanced Inductive

## 2.1 Purpose

Earlier phases handled mostly:

```text
Nat
Eq
List
```

Phase 9 supports:

```text
- indexed inductive families
- mutual inductive
- nested inductive
- dependent eliminator
- large elimination
- generated recursor / induction principle
- stronger positivity checker
```

Users do not add recursors as arbitrary axioms. The kernel/checker derives
recursors and computation rules mechanically from inductive declarations.

## 2.2 Indexed Inductive Family

Example:

```npa
inductive Vec.{u} (A : Type u) : Nat -> Type u where
| nil  : Vec A 0
| cons : {n : Nat} -> A -> Vec A n -> Vec A (Nat.succ n)
```

`Vec A n` depends on length index `n`. This supports specifications such as:

```text
append :
  Vec A m -> Vec A n -> Vec A (m + n)
```

Phase 9 formally handles `indices` in inductive declarations.

```rust
struct InductiveDecl {
    name: NameId,
    universe_params: Vec<UniverseParam>,
    params: Telescope,
    indices: Telescope,
    result_sort: SortExpr,
    constructors: Vec<ConstructorDecl>,
}
```

## 2.3 Constructor Result Check

Each constructor result must be an application of the declared inductive family
to valid params and indices. For indexed families, constructor result indices
must be checked structurally and with definitional equality as specified by the
kernel.

## 2.4 Mutual Inductive

Mutual inductives are checked as one strongly connected declaration block.

Example:

```text
Even / Odd
```

The positivity checker must analyze occurrences across the whole mutual block.
Generated recursors and induction principles are block-level artifacts.

## 2.5 Nested Inductive

Nested inductives allow recursive occurrences under approved strictly positive
type constructors such as `List`. The MVP should accept only a restricted,
auditable subset before generalizing.

## 2.6 Stronger Positivity Checker

The positivity checker must reject negative occurrences and unsupported nesting,
and must produce structured errors. Positivity is kernel/checker responsibility,
not an elaborator trust assumption.

## 2.7 Large Elimination

Large elimination is allowed only when the core rules permit it. The checker
must validate result universe/sort constraints and generated eliminators.

## 2.8 Certificate Changes

Certificates record:

```text
- params
- indices
- constructor specs
- generated recursor signature hash
- generated computation rule hash
- positivity / eliminator validation inputs needed by the checker
```

Generated artifacts are checked from the inductive declaration, not trusted from
source.

## 2.9 Completion

Advanced inductives are complete when:

```text
- Vec and Fin work
- mutual Even/Odd works
- a restricted nested inductive subset works
- generated recursors and computation rules are certificate-checked
- positivity failures are structured
```

---

# 3. Stronger Universe Polymorphism

## 3.1 Purpose

Practical libraries need real universe polymorphism for containers, algebra,
category theory, and generic programming.

## 3.2 Level Grammar

Levels include:

```text
0
succ u
max u v
imax u v
universe parameters
universe metavariables during elaboration
```

## 3.3 Universe Constraints

The checker stores and solves constraints such as:

```text
u = v
u <= v
```

Constraints must be deterministic and canonicalized in certificates.

## 3.4 Cumulativity

Cumulativity controls when values in lower universes can be used in higher
universes. The kernel rules must be explicit and tested; elaborator convenience
cannot define cumulativity.

## 3.5 Universe Minimization

Elaboration may minimize or generalize universes, but certificates record the
explicit resulting universe parameters and constraints. Unsolved universe metas
do not enter certificates.

## 3.6 Completion

Universe work is complete when polymorphic definitions such as `List`, `Eq`,
`Functor`, and category-style structures can be represented, checked, serialized,
and rechecked deterministically.

---

# 4. Typeclass

## 4.1 Purpose

Typeclasses support algebraic hierarchy and overloaded notation.

## 4.2 No Typeclass in the Kernel

Typeclass search is elaboration-side automation. The kernel sees explicit
dictionary arguments and explicit core terms.

## 4.3 Class Declaration

A class declaration elaborates to a structure-like dictionary type with fields.

```text
class Add A where
  add : A -> A -> A
```

## 4.4 Instance Declaration

Instances are named declarations registered in an instance table. Instance
metadata is untrusted; selected dictionaries are explicit terms checked by the
kernel.

## 4.5 Instance Search

Search is bounded and deterministic:

```text
- priority order
- depth / candidate limits
- cycle detection
- stable tie-breaks
- structured ambiguity errors
```

## 4.6 Ambiguity

Multiple valid instances at the same priority produce an ambiguity error rather
than arbitrary selection.

## 4.7 Typeclass and Notation

Overloaded notation uses typeclass search during elaboration to choose explicit
dictionary arguments. The certificate contains no notation or search trace.

## 4.8 Completion

Typeclass is complete when `Add`, `Mul`, `Zero`, `One`, `Semigroup`, and
`Monoid` style instance search works deterministically and lowers to explicit
core terms.

---

# 5. Quotient

## 5.1 Purpose

Quotients support equivalence classes, quotient sets, quotient groups/rings, and
real mathematical constructions.

## 5.2 Design Choice

If adopted, quotient is a small kernel/checker primitive with explicit
certificate rules. It is not implemented as an unchecked axiom bundle hidden in
the elaborator.

## 5.3 Setoid

```text
Setoid A:
  r : A -> A -> Prop
  equivalence proof
```

## 5.4 Quotient Primitive

Core operations include quotient construction and lift principles. Computation
rules are enabled only in profiles that explicitly support the corresponding
quotient version.

## 5.5 Certificate Changes

Certificates record quotient feature flags / profile requirements and the exact
terms involved in quotient operations. Checkers that do not support quotient
profiles reject them as unsupported core features.

## 5.6 Completion

Quotient is complete when `Setoid` quotient and `Quotient.lift` can be used and
rechecked by quotient-capable kernel/checker profiles.

---

# 6. SMT Certificates

## 6.1 Purpose

SMT integration is trusted only when solver results are reconstructed or checked
as proof certificates.

## 6.2 SMT Bridge

The bridge:

```text
NPA goal
  ↓ encode
SMT query
  ↓ solver
SMT proof / certificate
  ↓ checker / reconstruction
NPA proof term or certificate
```

Solver output alone is not accepted.

## 6.3 Supported Theories

Start with:

```text
- propositional logic
- EUF
- linear integer arithmetic
```

## 6.4 SMT Encoding

Encoding must record symbol mapping, assumptions, target, and theory profile so
that reconstruction is auditable.

## 6.5 SMT Certificate Checker

The checker validates an external proof format or reconstructs an NPA proof
term. Alethe is one possible target format. ([Alethe][5])

## 6.6 SMT Certificate Schema

SMT certificates contain:

```text
- theory profile
- encoded assumptions
- symbol map
- solver proof payload or reconstruction trace
- result hash
- NPA proof/certificate link
```

## 6.7 SMT Tactic

The SMT tactic is a proof-producing tactic. It may call a solver in an
untrusted layer, but must return a certificate or reconstructed proof accepted by
the checker.

## 6.8 Completion

SMT is complete when solver results are not trusted directly and proof
certificates can be checked or reconstructed for the supported theories.

---

# 7. Theorem Graph

## 7.1 Purpose

The theorem graph supports search, recommendation, minimization, library
refactoring, and AI training.

## 7.2 Graph Schema

Nodes:

```text
- theorem
- definition
- axiom
- inductive
- module
- domain tag
```

Edges:

```text
- depends_on
- uses_axiom
- rewrites_with
- imports
- similar_statement
- used_by
```

Graph identity is bound to certificate hashes and declaration interface hashes.

## 7.3 Graph Extraction

The graph is extracted from certificates and verified metadata during build /
release / index update. It is not extracted in the AI candidate hot path.

## 7.4 Graph API

API supports:

```text
- neighbors
- dependency paths
- axiom paths
- related theorem search
- premise retrieval features
```

## 7.5 AI Use

AI may use graph scores for ranking, but graph scores are not proof evidence and
do not enter certificate hashes.

## 7.6 Completion

Theorem graph is complete when dependency paths, axiom paths, related theorem
search, and premise retrieval can use certificate-bound graph snapshots.

---

# 8. Natural Language Formalization

## 8.1 Purpose

Natural language / LaTeX support proposes formal statement candidates. It does
not prove that the formalization matches the user's intent.

## 8.2 Formalization Pipeline

```text
natural language / LaTeX
  ↓
candidate generation
  ↓
reverse translation
  ↓
ambiguity display
  ↓
user confirmation
  ↓
formal proof search
```

## 8.3 Candidate Generation

Candidates include formal statement, confidence, ambiguity list, and required
imports. Confidence is not trusted.

## 8.4 Reverse Translation

Each formal candidate is translated back into human-readable language so users
can check meaning before proof search.

## 8.5 Intent Certificate

Intent records store:

```text
- informal statement
- formal statement
- reverse translation
- ambiguity choices
- user confirmation
- timestamp / signer metadata if needed
```

They are audit artifacts, not proof certificates.

## 8.6 Formalization Validation

The formal statement must still be elaborated, checked, and proved through the
normal certificate path.

## 8.7 Completion

Natural-language formalization is complete when it can produce candidates,
reverse translations, ambiguity reports, and intent certificates without
weakening proof trust.

---

# 9. Phase 9 API Additions

## 9.1 Advanced inductive

APIs expose parsed indexed/mutual/nested inductive declarations and structured
kernel errors.

## 9.2 Typeclass search

APIs expose bounded deterministic instance search results, ambiguity, and trace
metadata as untrusted elaboration diagnostics.

## 9.3 SMT

APIs expose SMT encoding, solver call sidecars, certificate validation, and NPA
proof reconstruction status.

## 9.4 Theorem graph

APIs expose graph snapshots and graph queries bound to certificate hashes.

## 9.5 Natural language formalization

Example response:

```json
{
  "candidates": [
    {
      "formal": "theorem user_goal : forall n : Nat, n + 0 = n := by _",
      "paraphrase": "For every natural number n, n + 0 = n.",
      "confidence": 0.97,
      "ambiguities": []
    }
  ]
}
```

---

# 10. Recommended Implementation Order

```text
1. stronger universe polymorphism
   universe constraints, metas, canonicalization

2. advanced inductive
   indexed families, recursor generation, positivity

3. theorem graph
   extract dependency graph from certificates

4. typeclass
   class/instance elaboration, dictionary passing

5. quotient
   Setoid, Quotient primitive, lift/sound

6. SMT certificates
   start from QF propositional / EUF / LIA

7. natural language formalization
   candidate generation, reverse translation, intent certificate
```

Reasoning:

```text
universe and inductive:
  foundation for other advanced features

theorem graph:
  improves typeclass, AI, and search

typeclass:
  needed for algebraic hierarchy

quotient:
  needed for real mathematics and quotient structures, but it extends the kernel

SMT:
  improves automation but requires proof certificate checking

natural language:
  useful, but meaning alignment issues make it last
```

---

# 11. Completion Criteria

Phase 9 is complete when:

```text
Advanced inductive:
  Vec, Fin, mutual Even/Odd, and restricted nested inductives work

Universe:
  polymorphic List/Eq/Functor/Category-style definitions work

Typeclass:
  instance search works for Add, Mul, Zero, One, Semigroup, Monoid

Quotient:
  Setoid quotient and Quotient.lift work

SMT certificates:
  solver results can be checked or reconstructed as proof certificates

Theorem graph:
  dependency paths, axiom paths, related theorem search, and premise retrieval use it

Natural language:
  formal candidates, reverse translations, ambiguities, and intent certificates can be generated
```

---

# 12. One-Sentence Summary

Phase 9 extends a small safe proof assistant into an advanced proof platform for
practical mathematics, automation, and AI formalization, without changing the
trust boundary.

```text
In kernel/checker:
  minimal inductive, universe, and quotient rules

In untrusted layers:
  typeclass search, SMT solver, theorem graph, AI formalizer

Ultimately checked:
  explicit core proof certificate
```

The goal is not merely to add convenience features; it is to add them without
breaking the meaning of verified.

[1]: https://lean-lang.org/doc/reference/latest/The-Type-System/Inductive-Types/?utm_source=chatgpt.com "4.4. Inductive Types"
[2]: https://lean-lang.org/doc/reference/latest/The-Type-System/Universes/?utm_source=chatgpt.com "4.3. Universes"
[3]: https://lean-lang.org/doc/reference/latest/Type-Classes/?utm_source=chatgpt.com "10. Type Classes"
[4]: https://lean-lang.org/doc/reference/latest/The-Type-System/Quotients/?utm_source=chatgpt.com "Quotients"
[5]: https://verit.gitlabpages.uliege.be/alethe/specification.pdf?utm_source=chatgpt.com "The Alethe Proof Format"
