# NPA Core Specification v0.1

This document is a Phase 0 deliverable. It fixes the minimal core specification
shared by the Rust kernel, certificate checker, and elaborator.

`develop/phase0.md` is the design note; this document is the specification that
the implementation follows.

## 1. Overview

The trusted boundary for NPA v0.1 is:

```text
trusted:
  canonical certificate
  Rust kernel
  independent checker

untrusted:
  source syntax
  parser
  name resolution
  elaborator
  tactic
  automation
  AI output
  source map
```

The kernel checks only the canonical core AST. Surface syntax, notation,
implicit arguments, holes, tactic blocks, and AI hints do not enter the core
calculus.

Included in v0.1:

```text
Sort, BVar, Const, App, Lam, Pi, Let
universe levels: 0, succ, max, imax, param
declarations: axiom, def, theorem, simple inductive
reduction: beta, delta, iota, zeta
initial inductives: Nat, Eq
```

Not included in v0.1:

```text
eta conversion
proof irrelevance as conversion
quotient
typeclass search
metavariables in certificates
general recursion
mutual inductive
nested inductive
coinductive
macros
```

## 2. Judgment Forms

The judgments used in the specification are:

```text
WFLevel(Delta, ell)
  ell is a well-formed level under universe parameter context Delta.

Sigma ; Delta ; Gamma |- t : T
  t has type T under global environment Sigma, universe context Delta, and local context Gamma.

Sigma ; Delta ; Gamma |- t == u : T
  t and u are definitionally equal at type T.

Sigma |- decl ok
  declaration decl can be added to Sigma.

Sigma |- module ok
  the declarations in the module can be checked in order.
```

`==` is definitional equality decided by kernel computation. It is distinct from
propositional equality `Eq`.

Below, when clear from context, `Delta` is omitted and written as
`Sigma ; Gamma |- t : T`.

## 3. Core Syntax

### 3.1 Names

Names are used for global constants and debugging binder names. Binding
structure is represented by de Bruijn indexes, not names.

```text
DeclarationName = Component ("." Component)*
Component       = [A-Za-z_][A-Za-z0-9_']*
```

Only ASCII apostrophe (`U+0027`) is allowed for `'`. Unicode prime-like
characters and operator symbols are not included in declaration names. For
certificate hashing, names are put in the canonical name table and terms
reference name IDs.

### 3.2 Universe Levels

```text
Level ell ::=
  zero
| succ ell
| max ell ell
| imax ell ell
| param alpha
```

Abbreviations:

```text
Prop   = Sort zero
Type u = Sort (succ u)
```

`WFLevel(Delta, param alpha)` holds only when `alpha in Delta`.

### 3.3 Terms

```text
Term t ::=
  Sort ell
| BVar i
| Const c [ell_1, ..., ell_n]
| App f a
| Lam x : A, body
| Pi  x : A, B
| Let x : A := v in body
```

`BVar 0` refers to the innermost binder. Inside the `body` of `Lam`, `Pi`, and
`Let`, one binder is added.

### 3.4 Local Context

```text
Gamma ::= empty | Gamma, LocalDecl

LocalDecl ::=
  x : A
| x : A := v
```

Local definitions are used for both typing and zeta reduction.

### 3.5 Global Declarations

```text
Decl ::=
  AxiomDecl(name, universe_context, type)
| DefDecl(name, universe_context, type, value, reducibility)
| TheoremDecl(name, universe_context, type, proof)
| InductiveDecl(data)

UniverseContext ::= universe_params + universe_constraints
UniverseConstraint ::= Level <= Level | Level = Level
Reducibility ::= reducible | opaque
```

`theorem` enters the environment as a type-checked opaque definition. Conversion
does not unfold theorem proofs.

### 3.6 Modules

```text
Module ::= Header Imports Declarations

Import ::= module_name + export_hash + optional certificate_hash
```

Declarations are ordered by dependency. Declarations at the same dependency
depth follow canonical order inside the module. `export_hash` is always
required. `certificate_hash` is optional in normal checking, but required in
high-trust mode.

## 4. Universe System

### 4.1 Sort Hierarchy

```text
Sort ell : Sort (succ ell)
```

Therefore:

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
```

`Sort ell : Sort ell` is forbidden.

### 4.2 Constraints

```text
Constraint ::= ell <= ell | ell = ell
```

The kernel checks the following for each declaration.

```text
- all level parameters are declared
- all level expressions are well formed
- universe constraints are satisfiable under natural-number interpretation
- no cycle is created in the Sort hierarchy
```

The v0.1 solver may be conservative. Reject constraints that are undecidable or
unsupported.

### 4.3 imax

`Pi` sort computation uses `imax`.

```text
imax u zero     = zero
imax u (succ v) = max u (succ v)
```

Other expressions are normalized deterministically. Even if the implementation
cannot fully reduce them, it must produce the same normal form from the same
input.

## 5. Typing Rules

### 5.1 Sort

```text
WFLevel(Delta, ell)
────────────────────────────────
Sigma ; Gamma |- Sort ell : Sort (succ ell)
```

### 5.2 Variable

```text
lookup(Gamma, i) = x : A
────────────────────────
Sigma ; Gamma |- BVar i : lift_for_lookup(A, i)
```

`lift_for_lookup` is the standard lift for de Bruijn indexes. The implementation
centralizes substitution / lifting in one place and uses the same operation for
typing and reduction.

Local definitions can also be referenced as variables.

```text
lookup(Gamma, i) = x : A := v
──────────────────────────────
Sigma ; Gamma |- BVar i : lift_for_lookup(A, i)
```

### 5.3 Constant

```text
Sigma(c) = decl
universe_params(decl) = [alpha_1, ..., alpha_n]
length(levels) = n
WFLevel(Delta, levels_k) for all k
universe_constraints(decl)[alpha_k := levels_k] are satisfied
────────────────────────────────────────────
Sigma ; Gamma |- Const c levels : instantiate_levels(type(decl), levels)
```

### 5.4 Pi Formation

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma, x : A |- B : Sort v
────────────────────────────────────────────
Sigma ; Gamma |- Pi x : A, B : Sort (imax u v)
```

Thus `Pi x : A, P` is `Prop` when `P : Prop`.

### 5.5 Lambda

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma, x : A |- body : B
────────────────────────────────────────────
Sigma ; Gamma |- Lam x : A, body : Pi x : A, B
```

### 5.6 Application

```text
Sigma ; Gamma |- f : F
whnf(F) = Pi x : A, B
Sigma ; Gamma |- a : A
────────────────────────────────────────────
Sigma ; Gamma |- App f a : instantiate(B, a)
```

`F` is reduced to weak-head normal form before checking whether it is a `Pi`.

### 5.7 Let

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma |- v : A
Sigma ; Gamma, x : A := v |- body : B
────────────────────────────────────────────
Sigma ; Gamma |- Let x : A := v in body : instantiate(B, v)
```

### 5.8 Conversion

```text
Sigma ; Gamma |- t : A
Sigma ; Gamma |- A == B : Sort u
────────────────────────────
Sigma ; Gamma |- t : B
```

Type checking may be implemented as a combination of inference and checking.
However, final accept / reject behavior must match this rule.

## 6. Definitional Equality

### 6.1 Equality Basis

Definitional equality is the smallest congruence generated by:

```text
alpha equivalence via de Bruijn representation
beta reduction
delta reduction for reducible definitions
iota reduction for recursors
zeta reduction for let and local definitions
```

v0.1 does not include:

```text
eta reduction
proof irrelevance conversion
proof unfolding for theorems
axiom unfolding
```

### 6.2 Beta

```text
App (Lam x : A, body) a
  --> instantiate(body, a)
```

### 6.3 Delta

```text
Const c levels --> instantiate_levels(value(c), levels)
```

This unfolds only when `c` is a `DefDecl` and `reducibility = reducible`.

### 6.4 Iota

Compute when the recursor major premise is a constructor-headed term.

Example for `Nat`:

```text
Nat.rec motive z s Nat.zero
  --> z

Nat.rec motive z s (Nat.succ n)
  --> s n (Nat.rec motive z s n)
```

For generic inductives, follow the computation rule generated for each
constructor.

### 6.5 Zeta

```text
Let x : A := v in body
  --> instantiate(body, v)
```

References to local definitions may also be unfolded by weak-head reduction.

### 6.6 Conversion Algorithm Requirements

The kernel conversion checker satisfies:

```text
- deterministic
- does not depend on source locations or source syntax
- does not unfold theorem proofs
- does not unfold opaque definitions
- rejects when normalization cannot finish or a resource limit is reached
```

Since v0.1 has no general recursion, unfolding well-typed reducible definitions
is assumed to terminate.

## 7. Declarations

Each declaration has its own `universe_context`. In the following rules,
`Delta = universe_context(decl)`, and constraints in `Delta` are assumed
satisfiable.

### 7.1 Axiom

```text
Sigma ; empty |- type : Sort u
──────────────────────────────
Sigma |- AxiomDecl(name, universe_context, type) ok
```

Axioms always appear in the certificate axiom report. In high-trust mode, axioms
outside the allowlist are rejected.

### 7.2 Definition

```text
Sigma ; empty |- type : Sort u
Sigma ; empty |- value : type
────────────────────────────────────
Sigma |- DefDecl(name, universe_context, type, value, reducibility) ok
```

`DefDecl` is added to the environment as a constant. It is subject to delta
reduction only when `reducible`.

### 7.3 Theorem

```text
Sigma ; empty |- type : Sort u
Sigma ; empty |- proof : type
────────────────────────────────
Sigma |- TheoremDecl(name, universe_context, type, proof) ok
```

Theorems are added to the environment as opaque constants. The proof body is
used in certificate checking, but not unfolded during conversion.

### 7.4 Declaration Order

Module checking reads imports first, then checks declarations from top to
bottom. References to later unchecked declarations are forbidden.

Only inductive declarations generate the inductive type, constructors,
recursor, and computation rules together before adding them to the environment.
v0.1 has no mutual blocks.

## 8. Simple Inductive Types

### 8.1 Declaration Shape

```text
InductiveDecl:
  name
  universe_params
  params:  telescope
  indices: telescope
  sort:    Sort s
  constructors: [ConstructorDecl]

ConstructorDecl:
  name
  type
```

Conceptual shape:

```text
inductive I.{u} (params : P) : indices -> Sort s where
| c1 : C1
| ...
| cn : Cn
```

### 8.2 Constructor Rule

The constructor type must return the target inductive type at the end of the
telescope.

```text
constructor_type =
  Pi y1 : A1, ... Pi yn : An, I params index_args
```

Reject constructors that return a non-target type.

### 8.3 Strict Positivity

The v0.1 positivity checker is conservative.

Allowed:

```text
- arguments with no recursive occurrences
- direct recursive occurrences as constructor arguments
  example: Nat.succ : Nat -> Nat
  example: List.cons : A -> List A -> List A
```

Forbidden:

```text
- recursive occurrences on the domain side of a function
  example: (I -> Nat) -> I
- nested inductive
  example: List I -> I
- mutual inductive
- opaque type aliases containing recursive occurrences
```

It is acceptable for this restriction to be too strong in v0.1. Reject
suspicious constructors.

### 8.4 Recursor Generation

Each inductive declaration generates a recursor.

Conceptual shape for `Nat`:

```text
Nat.rec :
  Pi motive : Nat -> Sort u,
    motive Nat.zero ->
    (Pi n : Nat, motive n -> motive (Nat.succ n)) ->
    Pi n : Nat, motive n
```

It has one minor premise per constructor, and performs iota reduction when the
major premise is a constructor-headed term.

### 8.5 Prop Elimination

For an inductive `I : Prop`, the v0.1 recursor motive is limited to `Prop`.

```text
allowed:
  motive : I ... -> Prop

rejected:
  motive : I ... -> Type u
```

Exceptions such as singleton elimination are not included in v0.1.

### 8.6 Initial Inductives

The Phase 1 kernel must be able to check at least the following.

```text
Nat : Type 0
Nat.zero : Nat
Nat.succ : Nat -> Nat

Eq.{u} : Pi A : Sort u, A -> A -> Prop
Eq.refl.{u} : Pi A : Sort u, Pi x : A, Eq A x x
```

`Nat` and `Eq` are eventually introduced from generic `InductiveDecl`. Even if
they are special-cased in early implementation, the observable environment must
match this specification.

## 9. Certificate Schema

### 9.1 Logical Schema

A certificate is a canonical module, not source.

```text
Certificate:
  format: "NPA-CERT-0.1"
  core_spec: "NPA-Core-0.1"
  module: module name
  imports: [Import]
  names: canonical name table
  levels: canonical level table
  terms: canonical term DAG
  declarations: [Decl]
  export_block: ExportBlock
  axiom_report: AxiomReport
  hashes:
    export_hash: sha256("NPA-MODULE-EXPORT-0.1" || canonical_export_block)
    axiom_report_hash: sha256("NPA-AXIOM-REPORT-0.1" || canonical_axiom_report)
    certificate_hash: sha256("NPA-MODULE-CERT-0.1" || trusted_payload_without_certificate_hash)
```

### 9.2 Canonicalization

Payload that affects hashes satisfies:

```text
- field order is fixed
- arrays have explicit length
- names are sorted by UTF-8 byte lexicographic order
- names are non-empty component lists, and components are non-empty strings without `.`
- level and term DAGs are topologically ordered; ties use structural tag order
- declarations are dependency ordered
- import order is lexicographic by module name, then export_hash, then certificate_hash option/value
- term binders use de Bruijn indices
- no whitespace or comments
- no notation
- no implicit arguments
- no unresolved metavariables
- universe constraints are normalized
```

Source maps, comments, diagnostics, and AI traces are not included in the
trusted payload.

### 9.3 Binary Encoding Requirement

On-disk `.npcert` files are canonical binary. Even if the v0.1 implementation
uses JSON, JSON is a debug format and not a hash-stable artifact.

Minimum requirements for canonical binary:

```text
- integers are unsigned LEB128
- strings are byte length + UTF-8 bytes
- enum variants use fixed numeric tags
- maps are forbidden in the hashed payload
- optional fields use explicit tag 0/1
- sha256 is computed over the exact canonical byte sequence
```

### 9.4 Axiom Report

The axiom report is built from the set of axioms each theorem / def / inductive
transitively depends on.

```text
axioms_used(decl) =
  direct axioms referenced in type/value/proof
  union axioms_used(transitive dependencies)
```

The module's `module_axioms` stores the union of declaration axiom sets in
canonical order.

```text
AxiomReport:
  per_declaration: [(decl_index, [Name])]
  module_axioms: [Name]
```

`per_declaration` is stored in declaration order; each axiom list and
`module_axioms` are stored in canonical name order. `safe_for_high_trust`,
`contains_sorry`, allowlist decisions, and similar values are audit/policy
views, and are not trusted as booleans inside the trusted payload.

### 9.5 Import Hash

Imports include not only the module name but also the export hash.

```text
Import:
  module = "Std.Nat.Basic"
  export_hash = "sha256:..."
  certificate_hash = optional "sha256:..."
```

The kernel checks that the import name and hash match the modules in
`verified_imports` passed by the caller. In high-trust mode, `certificate_hash`
must also match, and the same checker must already have checked the import
certificate.

### 9.6 Hash Roles

`export_hash` and `certificate_hash` do not hash the same object.

```text
export_hash:
  hash of the public interface needed by downstream modules for type checking and conversion

certificate_hash:
  hash of the trusted certificate payload, including opaque theorem proof bodies, excluding certificate_hash itself
```

If only an opaque theorem proof changes and the type, opacity, and axiom
dependencies do not change, `certificate_hash` changes but `export_hash` is
preserved. If a proof change changes axiom dependencies, the published trust
information changes, so `export_hash` also changes.

For `Const` references in types / reducible bodies included in the public
interface, the referenced `decl_interface_hash` is also included in the
declaration interface hash. If only the `Local(decl_index)` index were hashed,
transparent dependency changes inside the same module would not propagate to
the downstream `export_hash`. Non-axiom dependencies of opaque theorem proofs
and opaque def bodies are included only on the certificate hash side.

### 9.7 SHA-256 collision threat model

v0.1 `export_hash`, `axiom_report_hash`, `certificate_hash`, and package
artifact hashes use SHA-256. These hashes are computed from domain-separated
canonical bytes to avoid ambiguity in hash inputs and mixups caused by encoding
differences or surface syntax differences.

However, NPA does not mathematically prove SHA-256 collision impossibility.
Hashes are compact identity / pinning mechanisms, and artifact identity plus
import pinning rely on SHA-256 collision resistance as a cryptographic
assumption.

Under this assumption, the guarantee scope is:

```text
guaranteed without trusting parser/elaborator/tactic/AI:
  the canonical certificate bytes actually passed to the checker are decoded
  declaration / export / axiom report / certificate hashes are recomputed
  declarations are type-checked by the kernel
  axiom reports and axiom policy are rechecked from certificate bytes and the import environment

not guaranteed if SHA-256 collision is available to the adversary:
  that two different canonical payloads are byte-for-byte identical based only on a hash
  that a fetched artifact is identical to the expected artifact based only on a package lock file hash
  that the expected public environment and actual public environment are identical based only on an import export_hash
  that expected certificate bytes and actual certificate bytes are identical based only on a high-trust certificate_hash
```

Therefore, even if a SHA-256 collision exists, the checker never accepts a
certificate it actually received as a theorem without type-checking it. An
ill-typed certificate is rejected by kernel type checking even if another
payload with the same `certificate_hash` can be constructed.

On the other hand, import / package "identity with the expected artifact"
depends on hash pins. If an attacker can construct a different public
environment with the same `export_hash`, or a different certificate with the
same `certificate_hash`, import resolution / package locks that look only at
hashes cannot distinguish them. In that case, the downstream module is checked
against the actually resolved import environment, but cannot be said to have
been checked against the original artifact intended by the user.

The difference between normal mode and high-trust mode is:

```text
normal mode:
  import identity depends on module name + export_hash
  an export_hash collision can cause an import public environment mixup

high-trust mode:
  requires certificate_hash in addition to module name + export_hash
  also requires the import certificate to have been checked by the same checker
  however, if certificate_hash collisions are possible, byte-for-byte identity is not guaranteed
```

High-trust mode is not a design that "trusts unchecked certificates only by
hash." Imports must also have been checked by the same checker. However, the
ability to distinguish a checked different certificate with the same hash from
the expected certificate depends on SHA-256 collision resistance.

Operations that need collision-independent artifact identity should use
additional measures, such as obtaining expected canonical certificate bytes or a
canonical export block for byte-for-byte comparison, requiring multiple hashes /
hash agility as policy, or combining signatures, transparency logs, and
reproducible build evidence. These are operational / package policies for
strengthening artifact provenance; the basis for the kernel accepting a theorem
remains rechecking the canonical certificate.

## 10. Kernel Checking Algorithm

### 10.1 Module Checking

```text
check_module(cert, verified_imports):
  verify format and core_spec
  resolve imports by name and hash from verified_imports
  initialize Sigma from imports
  for decl in declarations:
    check_decl(Sigma, decl)
    recompute and compare declaration hashes
    extend Sigma with decl
  compute axiom_report
  compare with certificate axiom_report
  compute axiom_report_hash
  compare with certificate axiom_report_hash
  compute export_hash over canonical export block
  compare with certificate export_hash
  compute certificate_hash over trusted payload excluding certificate_hash itself
  compare with certificate certificate_hash
```

Here, import resolution means referencing canonical modules from an import store
/ verified import set already prepared by the caller. The kernel itself does not
perform file I/O, network fetches, or package resolution.

### 10.2 Type Inference

```text
infer(Sigma, Gamma, term) -> type
check(Sigma, Gamma, term, expected_type)
is_defeq(Sigma, Gamma, lhs, rhs, type) -> bool
whnf(Sigma, Gamma, term) -> term
```

`check` may compare the result of `infer` with the expected type by conversion.
Bidirectional typing may be used when the expected type is useful, as for
`Lam`, but the result must match the typing rules.

### 10.3 Error Model

Kernel errors are returned as structured enums. Human-facing messages are
untrusted information.

Minimum required error classes:

```text
UnknownConstant
UnknownUniverseParam
BadUniverseArity
InvalidBVar
ExpectedSort
ExpectedPi
TypeMismatch
NotDefEq
InvalidInductive
NonPositiveOccurrence
BadConstructorResult
InvalidRecursor
HashMismatch
AxiomNotAllowed
ResourceLimit
```

## 11. Minimal Examples

### 11.1 id

```text
id.{u} :
  Pi A : Sort u,
    Pi x : A,
      A

id.{u} :=
  Lam A : Sort u,
    Lam x : BVar 0,
      BVar 0
```

The Phase 1 kernel must be able to check this definition.

### 11.2 Eq.refl Proof

```text
theorem Nat.zero_eq_zero :
  Eq Nat Nat.zero Nat.zero

proof:
  Eq.refl Nat Nat.zero
```

Accept when the type of `Eq.refl Nat Nat.zero` matches the theorem type by
definitional equality.

### 11.3 add_zero Target

If `Nat.add` is defined as a reducible definition that recurses on the second
argument:

```text
theorem add_zero :
  Pi n : Nat, Eq Nat (Nat.add n Nat.zero) n
```

`Nat.add n Nat.zero` reduces to `n` by delta + iota, so the proof is essentially
`Eq.refl Nat n`.

## 12. Phase 0 Exit Criteria

Phase 0 is complete when the following are satisfied.

```text
- the core syntax read by the kernel is fixed
- the typing judgment is defined
- the scope of conversion is clear
- universe inconsistencies can be rejected
- the scope and positivity conditions for simple inductives are clear
- the responsibilities of recursors and iota reduction are clear
- certificate canonicalization and hash policy exist
- certificates alone can be checked without source syntax
- the differences between def / theorem / axiom / inductive are clear
- the minimum Phase 1 checking target is clear
```
