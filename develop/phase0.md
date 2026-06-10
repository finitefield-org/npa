> The implementation baseline specification for Phase 0 is [`core-spec-v0.1.md`](core-spec-v0.1.md).
> This file remains as a design memo for creating that specification.

These five items are the contents that should be included in **NPA Core Spec
v0.1** before implementation. "Writing it down" here means fixing it as a
**mathematical specification** shared by the kernel, certificate checker, and
elaborator, not merely as implementation notes.

The overall dependency chain is:

```text
core calculus
  ↓
typing rules
  ↓
conversion rules
  ↓
universe rules
  ↓
inductive rules
  ↓
certificate format
  ↓
kernel implementation
```

---

# 1. Core calculus

First, decide the minimal language understood by the kernel.

Surface syntax has notation, implicit arguments, typeclasses, tactics, match,
natural language, and so on, but these **do not enter the core calculus**.

The core calculus handles only fully explicit terms.

## 1.1 Judgment Forms

The specification defines at least the following judgments.

```text
Σ ; Γ ⊢ t : T
```

Meaning:

```text
Under global environment Sigma and local context Gamma,
term t has type T.
```

Other judgments are also needed.

```text
Σ ; Γ ⊢ t ≡ u : T
```

Meaning:

```text
t and u are definitionally equal at type T.
```

```text
Σ ⊢ decl ok
```

Meaning:

```text
Global declaration decl is correct.
```

```text
Σ ⊢ module ok
```

Meaning:

```text
The whole module is correct.
```

Here:

```text
Σ = global environment
Γ = local context
t, u = terms
T = type
```

## 1.2 Universe level

In Lean style, treat `Prop` as `Sort 0` and `Type u` as `Sort (succ u)`.

```text
Level ℓ ::=
  0
| succ ℓ
| max ℓ₁ ℓ₂
| imax ℓ₁ ℓ₂
| param α
```

`imax` is used for sort computation of dependent function types.

Intuitively:

```text
Sort 0          = Prop
Sort (succ 0)   = Type 0
Sort (succ 1)   = Type 1
...
```

## 1.3 Core term

The following core terms are sufficient for v0.1.

```text
Term t ::=
  Sort ℓ
| BVar i
| Const c [ℓ₁, ..., ℓₙ]
| App f a
| Lam x : A, body
| Pi  x : A, B
| Let x : A := v in body
```

Intentionally excluded:

```text
- notation
- implicit arguments
- typeclass placeholder
- unresolved metavariable
- tactic block
- source-level match
- pattern matching syntax
- natural language annotation
- AI-generated hint
```

`Nat`, `Eq`, `List`, constructors, recursors, and similar entities are not
special syntax in core terms; they are basically registered in environment
`Sigma` as `Const`.

For example:

```text
Nat
Nat.zero
Nat.succ
Nat.rec
Eq
Eq.refl
```

are all global constants.

---

# 2. Typing rules

Next, define typing rules for core terms.

## 2.1 Context

The local context has the following form.

```text
Γ ::= · | Γ, x : A | Γ, x : A := v
```

However, inside certificates, name `x` is not essential; de Bruijn indexes are
used.

## 2.2 Sort rule

```text
Σ ; Γ ⊢ Sort ℓ : Sort (succ ℓ)
```

That is:

```text
Prop : Type 0
Type 0 : Type 1
Type 1 : Type 2
...
```

## 2.3 Variable rule

Variables in the context have their types.

```text
x : A ∈ Γ
────────────────
Σ ; Γ ⊢ x : A
```

In certificates, this is `BVar i`, not `x`.

## 2.4 Constant rule

Constants in the global environment have their types.

```text
c : A ∈ Σ
────────────────
Σ ; Γ ⊢ c : A
```

For universe-polymorphic constants, substitute levels.

Example:

```text
Eq.{u} : Π A : Sort u, A → A → Prop
```

For this:

```text
Eq.{0}
Eq.{1}
Eq.{max u v}
```

it is used like this.

## 2.5 Pi formation

Formation rule for dependent function types.

```text
Σ ; Γ ⊢ A : Sort u
Σ ; Γ, x : A ⊢ B : Sort v
────────────────────────────────
Σ ; Γ ⊢ Pi x : A, B : Sort (imax u v)
```

The reason for using `imax` is to make `Prop` impredicative.

In particular:

```text
A : Sort u
B : Prop
```

then:

```text
Π x : A, B : Prop
```

is desired.

In other words, a proposition remains a proposition even when quantified over
any type.

Example:

```text
∀ n : Nat, n = n : Prop
```

## 2.6 Lambda rule

```text
Σ ; Γ ⊢ A : Sort u
Σ ; Γ, x : A ⊢ body : B
────────────────────────────────────
Σ ; Γ ⊢ Lam x : A, body : Pi x : A, B
```

## 2.7 Application rule

```text
Σ ; Γ ⊢ f : Pi x : A, B
Σ ; Γ ⊢ a : A
────────────────────────────
Σ ; Γ ⊢ App f a : B[x := a]
```

## 2.8 Let rule

```text
Σ ; Γ ⊢ A : Sort u
Σ ; Γ ⊢ v : A
Σ ; Γ, x : A := v ⊢ body : B
────────────────────────────────────
Σ ; Γ ⊢ Let x : A := v in body : B[x := v]
```

## 2.9 Conversion rule

If types are equal by definitional equality, the type may be changed.

```text
Σ ; Γ ⊢ t : A
Σ ; Γ ⊢ A ≡ B : Sort u
──────────────────────
Σ ; Γ ⊢ t : B
```

Because of this rule, the conversion checker is central to the kernel.

---

# 3. Conversion rules

Conversion rules define the equality that the kernel treats as "the same type."

Importantly, this is not a **proved equality**, but an equality identified by
kernel computation.

## 3.1 β-reduction

```text
(App (Lam x : A, body) a)
  ↦ body[x := a]
```

Example:

```text
(fun x => x + 1) 3
```

reduces to:

```text
3 + 1
```

.

## 3.2 δ-reduction

Unfold transparent definitions.

```text
Const c ↦ value(c)
```

However, always unfolding every definition is slow, so definitions have
transparency. core-spec v0.1 initially restricts this to `reducible` and
`opaque`.

```text
reducible
opaque
```

Recommended:

```text
def      : reducible or opaque
theorem  : opaque
abbrev   : reducible
```

It is better not to unfold theorem proof bodies during conversion, because huge
proofs unfolding unexpectedly would be very slow. More detailed transparency
such as `semireducible` / `irreducible` is a candidate extension for Phase 9 or
later.

## 3.3 ι-reduction

Computation rules for inductive recursors.

Example:

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

This is the computation rule for induction / recursion on `Nat`.

## 3.4 ζ-reduction

Let unfolding.

```text
Let x : A := v in body
  ↦ body[x := v]
```

## 3.5 Eta-reduction Is Not Included In v0.1

Putting behavior close to function extensionality into definitional equality
makes the conversion checker complex.

Therefore, in v0.1:

```text
(fun x => f x) ≡ f
```

is not included in kernel conversion.

If needed, treat it later as a theorem or axiom.

## 3.6 Proof Irrelevance Is Also Not Included In Conversion

Identifying all proofs of `Prop` by definitional equality is convenient, but
makes the kernel complex.

In v0.1:

```text
h₁ : P
h₂ : P
```

does not imply:

```text
h₁ ≡ h₂
```

.

If proof irrelevance is desired, introduce it as a standard-library theorem or
optional axiom.

---

# 4. Universe rules

Universe rules prevent the hierarchy of types of types from becoming
inconsistent.

## 4.1 Sort hierarchy

```text
Sort 0 : Sort 1
Sort 1 : Sort 2
Sort 2 : Sort 3
...
```

Read as:

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
...
```

## 4.2 Universe Level Constraints

Universe levels have constraints.

```text
Constraint ::=
  ℓ₁ ≤ ℓ₂
| ℓ₁ = ℓ₂
```

For example, in a polymorphic definition:

```text
id.{u} : Π A : Sort u, A → A
```

`u` is a universe parameter.

## 4.3 Pi Universe

Writing the previous rule again:

```text
A : Sort u
B : Sort v
────────────────────────
Π x : A, B : Sort (imax u v)
```

Definition of `imax`:

```text
imax u 0        = 0
imax u (succ v) = max u (succ v)
```

Thus:

```text
Π x : Nat, Prop
```

becomes `Prop`.

On the other hand:

```text
Π x : Nat, Type 0
```

becomes at least `Type 0`.

## 4.4 Universe polymorphic constants

Constant declarations have universe parameters.

Example:

```text
Eq.{u} :
  Π A : Sort u, A → A → Prop
```

In certificates:

```json
{
  "name": "Eq",
  "universe_params": ["u"],
  "type": "Π A : Sort u, A → A → Prop"
}
```

At use sites, concrete levels are passed.

```text
Const Eq [u]
```

## 4.5 Universe consistency check

The kernel checks the following for each declaration.

```text
- universe parameters are declared
- level expressions are well formed
- universe constraints are satisfiable
- there is no cycle in the Sort hierarchy
```

Forbidden:

```text
Type u : Type u
```

Allowing this leads to Girard's paradox-like inconsistency, so always maintain a
hierarchy like:

```text
Type u : Type (u+1)
```

.

---

# 5. Inductive rules

Inductive types are central to a proof assistant.

`Nat`, `List`, `Eq`, `False`, `And`, `Or`, `Exists`, and similar types are
defined as inductive types.

## 5.1 Inductive Declaration Shape

Basic shape:

```text
inductive I.{u} (params : P) : indices → Sort s where
| c₁ : C₁
| c₂ : C₂
...
| cₙ : Cₙ
```

In v0.1, start with simple inductive types.

```text
inductive Nat : Type 0 where
| zero : Nat
| succ : Nat → Nat
```

```text
inductive List.{u} (A : Type u) : Type u where
| nil  : List A
| cons : A → List A → List A
```

```text
inductive Eq.{u} (A : Sort u) (a : A) : A → Prop where
| refl : Eq A a a
```

## 5.2 Constructor rule

The constructor type must ultimately return that inductive type.

Example:

```text
Nat.zero : Nat
Nat.succ : Nat → Nat
```

This is OK.

On the other hand, the following is invalid.

```text
bad : Nat → Bool
```

because it is a constructor of `Nat` but does not return `Nat`.

## 5.3 Strict positivity

When an inductive type uses itself, it may appear only in **strictly positive**
positions.

OK：

```text
inductive List A where
| nil  : List A
| cons : A → List A → List A
```

`List A` appears in a positive position as a constructor argument.

NG：

```text
inductive Bad where
| bad : (Bad → Nat) → Bad
```

`Bad` appears on the argument side of a function, that is, in a negative
position.

Allowing this may break the logic.

In v0.1, make the positivity checker conservative.

```text
Allow:
  I
  A → I
  I as a constructor argument, not arbitrary I -> I
  nested inductives such as List I are unsupported at first

Forbid:
  (I → A) → I
  negative occurrence
  nested inductive
  mutual inductive
```

Nested / mutual inductives are added later.

## 5.4 Recursor generation

Generate a recursor for each inductive type.

Conceptually, the recursor for `Nat` is:

```text
Nat.rec :
  Π motive : Nat → Sort u,
    motive Nat.zero →
    (Π n : Nat, motive n → motive (Nat.succ n)) →
    Π n : Nat, motive n
```

Computation rules:

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

Conceptually, the recursor for `List` is:

```text
List.rec :
  Π motive : List A → Sort u,
    motive List.nil →
    (Π x : A, Π xs : List A,
       motive xs → motive (List.cons x xs)) →
    Π xs : List A, motive xs
```

## 5.5 Prop elimination restriction

Allowing elimination from an inductive in `Prop` to arbitrary `Type` is
dangerous or complex.

v0.1 takes the conservative side.

```text
When I : Prop,
the motive of I.rec is limited to Prop.
```

That is:

```text
False.elim : False → P
```

is OK when `P : Prop`.

However:

```text
False → Nat
```

is restricted in v0.1.

Singleton elimination and similar Lean-style exceptions can be added later, but
keep the first version simple.

## 5.6 Inductive declaration check

The kernel checks the following for inductive declarations.

```text
- parameters are well typed
- indices are well typed
- result sort is well formed
- constructor types are well typed
- constructor return values are the target inductive type
- recursive occurrences are strictly positive
- universe constraints are consistent
- the recursor type can be generated correctly
- recursor computation rules are valid
```

---

# 6. Certificate format

Finally, decide the format of proof certificates received by the kernel /
checker.

This is very important.

Do not trust source code.
Do not trust tactics.
Do not trust AI.
Do not fully trust the elaborator.

The checker reads canonical certificate artifacts, not source. The kernel reads
the canonical core AST / declarations decoded by the checker / loader. File I/O
and reads from the import store are not kernel responsibilities.

## 6.1 Certificate Purpose

Certificates are for guaranteeing:

```text
- which proposition was proved
- which proof term proved it
- which definitions / theorems it depended on
- which axioms were used
- which hashes identify imports
- whether it can be rechecked by the kernel/checker
```

## 6.2 Certificate Outline

It can be represented as JSON for humans, but the real storage format is
canonical binary. JSON is for explanation / debugging and is not a hash-stable
artifact.

Conceptually:

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Nat.Basic",
  "imports": [],
  "universe_params": [],
  "declarations": [],
  "export_block": [],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": []
  },
  "hashes": {
    "export_hash": "sha256:...",
    "certificate_hash": "sha256:...",
    "axiom_report_hash": "sha256:..."
  }
}
```

## 6.3 Declaration Kinds

There are mainly four kinds of declarations in certificates.

```text
AxiomDecl
DefDecl
TheoremDecl
InductiveDecl
```

### AxiomDecl

```json
{
  "kind": "axiom",
  "name": "Classical.choice",
  "universe_params": ["u"],
  "type": "..."
}
```

Axioms are forbidden by default or controlled by an allowlist.

### DefDecl

```json
{
  "kind": "def",
  "name": "Nat.add",
  "universe_params": [],
  "type": "...",
  "value": "...",
  "reducibility": "reducible"
}
```

The kernel checks:

```text
value : type
```

.

### TheoremDecl

Theorems are basically opaque definitions.

```json
{
  "kind": "theorem",
  "name": "Nat.add_zero",
  "universe_params": [],
  "type": "Π n : Nat, Eq Nat (Nat.add n Nat.zero) n",
  "proof": "...",
  "opaque": true
}
```

The kernel checks:

```text
proof : type
```

.

However, conversion does not unfold the proof.

### InductiveDecl

```json
{
  "kind": "inductive",
  "name": "Nat",
  "universe_params": [],
  "params": [],
  "indices": [],
  "sort": "Type 0",
  "constructors": [
    {
      "name": "Nat.zero",
      "type": "Nat"
    },
    {
      "name": "Nat.succ",
      "type": "Nat → Nat"
    }
  ]
}
```

The kernel checks according to inductive rules and adds constructors and the
recursor to the environment.

## 6.4 Term encoding

Terms inside certificates are stored as canonical ASTs, not source text.

Bad format:

```text
"∀ n : Nat, n + 0 = n"
```

Good format:

```text
Pi n : Nat,
  Eq Nat
    (Nat.add n Nat.zero)
    n
```

In practice, names also use an intern table rather than strings.

```json
{
  "names": [
    "Nat",
    "Nat.zero",
    "Nat.add",
    "Eq"
  ],
  "terms": [
    ["Const", 0, []],
    ["Const", 1, []],
    ["Const", 2, []],
    ["Const", 3, []]
  ]
}
```

The implementation uses binary encoding rather than JSON.

## 6.5 Canonicalization

The same canonical payload must produce the same hash. Untrusted metadata such
as source maps is not included in hash targets.

For that:

```text
- use de Bruijn indexes
- fix name order
- fix import order
- include no whitespace
- include no notation
- include no implicit arguments
- allow no unresolved metavariables
- normalize universe constraints
- order declarations by dependency
```

## 6.6 Import hash

Names alone are insufficient for imports.

```json
{
  "module": "Std.Nat.Basic",
  "export_hash": "sha256:abc...",
  "certificate_hash": "sha256:def..."
}
```

This pins dependency hashes. `export_hash` is required, and `certificate_hash`
is required in high-trust mode.

This prevents the problem of:

```text
a module with the same name but different contents
```

.

## 6.7 Axiom report

Certificates always record used axioms.

```json
{
  "axioms_used": [
    "Classical.choice",
    "Propext"
  ]
}
```

In high-trust mode:

```text
- no custom axiom
- no sorry
- allowed axioms only
```

are checked.

## 6.8 Source Maps Are Untrusted Information

Source code location information is not included in the canonical certificate
trusted payload. If needed, it may be put in a debug sidecar / audit envelope
separate from the certificate.

```json
{
  "source_map": {
    "Nat.add_zero": {
      "file": "Std/Nat/Basic.npa",
      "line": 42,
      "column": 1
    }
  }
}
```

However, source maps are not used for kernel checking, certificate hashes, or
export hashes.

They are for IDEs and error display.

---

# 7. Proposed Table Of Contents For The v0.1 Specification

If written as a specification, use a document structure like this.

```text
NPA Core Specification v0.1

1. Overview
   1.1 Trusted boundary
   1.2 Global environment
   1.3 Local context
   1.4 Judgment forms

2. Core syntax
   2.1 Universe levels
   2.2 Sorts
   2.3 Terms
   2.4 Declarations
   2.5 Modules

3. Typing rules
   3.1 Sort
   3.2 Variable
   3.3 Constant
   3.4 Pi
   3.5 Lambda
   3.6 Application
   3.7 Let
   3.8 Conversion

4. Definitional equality
   4.1 Alpha equivalence
   4.2 Beta reduction
   4.3 Delta reduction
   4.4 Iota reduction
   4.5 Zeta reduction
   4.6 Transparency
   4.7 Conversion algorithm requirements

5. Universe system
   5.1 Level grammar
   5.2 Sort hierarchy
   5.3 imax
   5.4 Constraints
   5.5 Universe polymorphism
   5.6 Consistency checking

6. Inductive types
   6.1 Declaration form
   6.2 Constructor rules
   6.3 Strict positivity
   6.4 Recursor generation
   6.5 Computation rules
   6.6 Prop elimination restriction

7. Declarations
   7.1 Axiom
   7.2 Definition
   7.3 Theorem
   7.4 Inductive
   7.5 Reducibility and opacity

8. Certificate format
   8.1 Header
   8.2 Imports
   8.3 Name table
   8.4 Level encoding
   8.5 Term encoding
   8.6 Declaration encoding
   8.7 Dependency graph
   8.8 Axiom report
   8.9 Hashing
   8.10 Canonical binary format

9. Kernel checking algorithm
   9.1 Module checking
   9.2 Declaration checking
   9.3 Type inference
   9.4 Conversion checking
   9.5 Inductive checking

10. Examples
   10.1 id
   10.2 Nat
   10.3 Eq
   10.4 add
   10.5 add_zero
```

---

# 8. Minimal Examples

As a minimal example for the specification, first make this theorem pass.

```text
id.{u} : Π A : Sort u, A → A
id := λ A : Sort u, λ x : A, x
```

In core:

```text
Def id.{u}
  type:
    Pi A : Sort u,
      Pi x : A,
        A

  value:
    Lam A : Sort u,
      Lam x : A,
        BVar 0
```

What the kernel checks:

```text
λ A, λ x, x
:
Π A : Sort u, A → A
```

If this passes, the minimum:

```text
Sort
Pi
Lambda
Application
Variable
Universe polymorphism
```

is working.

Next:

```text
Nat
Nat.zero
Nat.succ
Nat.rec
```

are introduced by inductive rules, and:

```text
theorem Nat.zero_eq_zero : Eq Nat Nat.zero Nat.zero
```

can be proved by:

```text
Eq.refl Nat Nat.zero
```

.

---

# 9. Acceptance Criteria Before Implementation

Once the specification for these five items is written, check the following
before implementation.

```text
- the syntax read by the kernel is clear
- typing judgments are defined
- the extent of equality recognized by conversion is clear
- universe inconsistency can be prevented
- positivity conditions for inductive types are clear
- recursor types and computation rules are clear
- certificates can be hashed canonically
- certificates alone can be checked without source syntax
- the differences between theorem, def, axiom, and inductive are clear
- the design does not unfold opaque theorems
```

The most important thing at this stage is **not to add too many convenient
features**.

Keep v0.1 small.

```text
Include:
  Sort, Pi, Lambda, App, Let, Const, Nat, Eq, simple inductive

Exclude:
  quotient, mutual inductive, nested inductive, coinductive,
  typeclass, η-conversion, proof irrelevance conversion,
  general recursion, macros, tactic language
```

First build a small kernel and aim for a state where `id`, `Nat`, `Eq`,
`Nat.rec`, and `add_zero` can be checked as core declarations without
source/tactics.
