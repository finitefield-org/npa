# Phase 6 Human Profile: Library

This document is the authoritative design for NPA's **human-facing Phase 6
standard library**. The goal of Phase 6 is not to immediately build a huge
Mathlib-like library, but to build a **small, solid standard library** that
later proof search, tactics, theorem search, and AI assistance can depend on.

This Human Profile covers:

```text
- structure of standard library source read and written by humans
- module dependency
- definitions, theorems, and naming rules
- human-facing notation
- attribute policy for simp-lite / rw / apply / theorem search
- library policy for no sorry / no custom axiom
```

This Human Profile is not authoritative for:

```text
- canonical certificate bytes
- release manifest
- import bundle wire payload
- theorem / simp / rewrite machine artifact hash
- import_closure / tactic_options recipe for Phase 5 session creation
- Phase 7 retrieval cache key
- Phase 8 audit hook
```

For those machine artifacts / wire contracts / validation order,
`develop/phase6-ai.md` is authoritative. Human Profile source text, pretty
statements, notation, and attribute annotations are inputs and design
information for building the standard library. They are not the final trust
root.

```text
Untrusted:
  library source text
  notation / pretty statement
  human-facing attribute annotation
  theorem search ranking
  prompt text
  AI-generated proof hints

Trusted:
  Phase 2 canonical certificate bytes
  export_hash / certificate_hash / decl_interface_hash
  Phase 1 kernel check
  Phase 2 verifier output
  Phase 8 independent checker
```

The target modules are these four.

```text
Std.Logic
Std.Nat
Std.List
Std.Algebra.Basic
```

This MVP module membership is exact. In the AI Profile
`npa.stdlib.mvp.v1` release, only the same four modules are allowed as release
modules. Certificate artifact paths, canonical module order, and package locator
rules follow the fixed tables in `develop/phase6-ai.md`. Do not infer machine
release identity from Human Profile source file layout or build tool output
order.

The basic policy is as follows.

```text
1. First include only definitions that are easy for the kernel to check
2. Adopt only proofs that can be certificate-generated
3. Make no sorry / no custom axiom the standard
4. Add attributes for simp-lite / rw / apply / theorem search from the beginning
5. Include theorem names, rewrite direction, dependencies, and axiom reports in the library design
```

---

# 1. Phase 6 Overview

What Phase 6 creates is not merely source files.

```text
library source
  ↓
elaboration
  ↓
core declarations
  ↓
kernel check
  ↓
certificate generation
  ↓
theorem index
  ↓
use from IDE / tactic / AI search
```

In Human source builds / local debug views, each module may emit per-module
derived files such as:

```text
Std/Logic.npa
Std/Logic.npcert
Std/Logic.index.json
Std/Logic.axioms.json   -- derived axiom report view
Std/Logic.graph.json    -- minimal derived dependency graph
```

Here, `graph.json` is a minimal dependency artifact derived from the
certificate. The Phase 9 theorem graph is the later stage that extends this
information with schema / API / ranking functionality. However, these
per-module JSON files are Human Profile views for explanation, debugging, and
local cache. The wire artifact set for the Phase 6 AI MVP is authoritative in
the `Std.machine-*.json` files described by `develop/phase6-ai.md`; AI-facing
import bundles / theorem indexes / simp profiles / rewrite profiles are read
from release-wide artifacts. The AI search path must not require reading Human
source file layout or per-module debug JSON.

The release-wide generated artifacts for the AI Profile are:

```text
Std.machine-release.json
Std.machine-import-bundles.json
Std.machine-theorem-index.json
Std.machine-simp-profiles.json
Std.machine-rewrite-profiles.json
Std.machine-axiom-report.json
Std.machine-prompt-metadata.json  optional
```

## 1.1 Artifact Handling

What is committed as authoritative in this repository is the Human source layout
/ skeleton, Rust implementation, tests, and docs. `.npcert` files generated from
the source package, per-module debug JSON, and release-wide `Std.machine-*.json`
are deterministic release/build artifacts.

They may be published as release packages or CI artifacts, but generated
artifacts are not hand-written or manually updated and committed in ordinary
development diffs. Tests place source skeletons such as `Std/Logic.npa` in a
temporary package, then regenerate and verify raw `.npcert` and machine
sidecars from there. In the MVP implementation, the manifest fixes module
membership / certificate paths, and source skeletons fix import intent.
Certificate contents are generated from deterministic Rust core-module
builders. Making complete Human source proof scripts authoritative for the
standard library is a future source elaboration extension. If stale artifacts
remain in the working tree, treat them as build output rather than source /
implementation changes, and delete them or confirm their regeneration source
before committing.

The AI-facing path reads release-wide machine artifacts. Human debug views,
pretty statements, and source layout are for explanation and diagnostics, and
are not required inputs for Phase 7 retrieval or Phase 5 session creation.

Phase 6 completion criteria are:

```text
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic are checked by the kernel
- each module is certificate-generated
- import entries have export_hash, and certificate_hash is also generated for high-trust mode
- module export_hash / certificate_hash / axiom_report_hash are generated
- axiom reports are empty or within the allowlist
- theorem search index is generated
- simp-lite can use basic theorems
- standard theorems can be proved with Phase 4 tactics
```

.

---

# 2. Module Dependencies

Keep dependencies small.

```text
Core kernel
  ├── Sort
  ├── Pi
  ├── Lambda
  ├── App
  ├── Let
  ├── Const
  ├── simple inductive
  ├── Nat primitive/generated
  └── Eq primitive/generated

Std.Logic
  └── uses kernel/core profile

Std.Nat
  └── depends on Std.Logic

Std.List
  ├── depends on Std.Logic
  └── depends on Std.Nat

Std.Algebra.Basic
  └── depends on Std.Logic
```

Here, `Core` / kernel/core profile is not a Phase 2 module import. The
`Std.Logic` certificate does not have an `ImportEntry` named `Core`; it is
checked internally by the verifier against `core_spec_id` /
`kernel_semantics_profile_id`. `Eq` is treated as a public inductive export of
the `Std.Logic` certificate, and `Nat` is treated as a public inductive export
of the `Std.Nat` certificate.

Diagram:

```text
Std.Logic
   ↓
Std.Nat
   ↓
Std.List

Std.Logic
   ↓
Std.Algebra.Basic
```

`Std.Algebra.Basic` should not depend on `Std.Nat`, because algebraic
structures are not limited to natural numbers.

---

# 3. Library-Wide Naming Rules

Make naming rules strict for future theorem search and AI search.

## 3.1 Basic Pattern

```text
Namespace.object_property
Namespace.operation_property
Namespace.theorem_name
```

Examples:

```text
Eq.refl
Eq.symm
Eq.trans
Nat.add_zero
Nat.zero_add
Nat.add_assoc
List.append_nil
List.nil_append
List.append_assoc
Monoid.left_id
Monoid.right_id
```

## 3.2 Naming Rewrite Theorems

```text
xxx_zero
zero_xxx
xxx_assoc
xxx_comm
xxx_left_id
xxx_right_id
```

Examples:

```text
Nat.add_zero     : n + 0 = n
Nat.zero_add     : 0 + n = n
Nat.mul_zero     : n * 0 = 0
Nat.zero_mul     : 0 * n = 0
List.append_nil  : xs ++ [] = xs
List.nil_append  : [] ++ xs = xs
```

In this Phase 6 document, `0`, `1`, and `2` are display abbreviations for
`Nat.zero` / `Nat.succ ...`. In Phase 3 MVP input, it is enough to write
`Nat.zero` or `zero` inside an opened namespace.

## 3.3 Naming Auxiliary Theorems

Internal auxiliary theorems may use `_aux`, but the policy is not to expose them
publicly.

```text
Nat.add_comm_aux
```

If they are exposed in the public API, give them meaningful names.

---

# 4. Attribute Design

In Phase 6, theorems receive attributes.

```text
@[simp]
@[rw]
@[intro]
@[elim]
@[apply]
@[refl]
@[trans]
@[congr]
```

As Human source / IDE metadata, accept at least these.

```text
@[simp]   used by simp-lite
@[rw]     used by rw search
@[intro]  used by intro/constructor-style search
@[elim]   used by apply/cases-style search
```

Attributes are not part of the trusted certificate. However, they are used by
the theorem index and tactic search. Here, attributes are human-facing source
annotations / IDE hints. In the Phase 6 AI MVP,
`MachineStdTheoremEntry.attributes` does not carry source annotations directly;
it emits only `simp` / `rw` derived from validated rewrite/simp profiles, and
`apply` corresponding to apply mode. `intro` / `elim` / `refl` / `trans` /
`congr` may be used as Human-side search/display hints, but are not emitted in
`npa.stdlib.theorem-index.mvp.v1`. This lets the AI path reconstruct candidate
sets only from certificate-bound machine profiles, without interpreting Human
attribute tables.

Attribute metadata example:

```json
{
  "name": "Nat.add_zero",
  "attributes": ["simp", "rw"],
  "rewrite": {
    "lhs": "n + 0",
    "rhs": "n",
    "orientation": "left_to_right",
    "safe": true
  },
  "axioms_used": []
}
```

---

# 5. Std.Logic

`Std.Logic` is the minimal logic library.

Purpose:

```text
- equality reasoning centered on Eq
- True / False / And / Or / Iff / Not / Exists
- foundation for rw / apply / exact / theorem search
- constructive logic without classical axioms
```

## 5.1 Definitions To Include

### Eq

`Eq` may exist as a simple inductive on the Phase 1 core side. `Std.Logic`
exposes it and adds basic theorems.

```npa
inductive Eq.{u} {A : Sort u} (x : A) : A -> Prop where
| refl : Eq x x
```

At the surface:

```npa
x = y
```

expands to:

```text
Eq x y
```

.

generated export：

```text
Eq.refl is the public constructor export generated from constructor refl of the Eq inductive.
Std.Logic does not separately public-export a theorem with the same exported name.
```

Basic theorems:

```npa
theorem Eq.symm {A : Sort u} {x y : A} : x = y -> y = x := ...
theorem Eq.trans {A : Sort u} {x y z : A} : x = y -> y = z -> x = z := ...
theorem Eq.subst {A : Sort u} {x y : A} (P : A -> Prop) :
  x = y -> P x -> P y := ...
theorem Eq.congrArg {A : Sort u} {B : Sort v} (f : A -> B) :
  x = y -> f x = f y := ...
```

Attributes:

```text
Eq.refl      @[refl] generated constructor
Eq.symm      usable by rw reverse
Eq.trans     @[trans]
Eq.congrArg  for theorem search
Eq.subst     for generating rw proof terms
```

## 5.2 True / False

```npa
inductive True : Prop where
| intro : True

inductive False : Prop where
```

Basic theorems:

```npa
theorem False.elim {P : Prop} : False -> P := ...
```

Attributes:

```text
True.intro   @[intro]
False.elim   @[elim]
```

Following the Phase 1 Prop elimination restriction, initially limit
`False.elim` to `P : Prop`. Large elimination such as `False -> Nat` is
postponed.

## 5.3 Not

`Not` can be a definition.

```npa
def Not (P : Prop) : Prop := P -> False
```

notation：

```text
¬ P
```

Basic theorems:

```npa
theorem absurd {P Q : Prop} : P -> ¬ P -> Q := ...
theorem not_intro {P : Prop} : (P -> False) -> ¬ P := ...
theorem not_elim {P : Prop} : ¬ P -> P -> False := ...
```

## 5.4 And

```npa
inductive And (P Q : Prop) : Prop where
| intro : P -> Q -> And P Q
```

notation：

```text
P ∧ Q
```

Basic theorems:

```npa
theorem And.left  {P Q : Prop} : P ∧ Q -> P := ...
theorem And.right {P Q : Prop} : P ∧ Q -> Q := ...
theorem And.intro {P Q : Prop} : P -> Q -> P ∧ Q := ...
```

Attributes:

```text
And.intro  @[intro]
And.left   @[elim]
And.right  @[elim]
```

## 5.5 Or

```npa
inductive Or (P Q : Prop) : Prop where
| inl : P -> Or P Q
| inr : Q -> Or P Q
```

notation：

```text
P ∨ Q
```

Basic theorems:

```npa
theorem Or.elim {P Q R : Prop} : P ∨ Q -> (P -> R) -> (Q -> R) -> R := ...
theorem Or.inl {P Q : Prop} : P -> P ∨ Q := ...
theorem Or.inr {P Q : Prop} : Q -> P ∨ Q := ...
```

## 5.6 Iff

`Iff` can be defined either as a definition or as an inductive.

In the MVP, make it a structure-like inductive.

```npa
inductive Iff (P Q : Prop) : Prop where
| intro : (P -> Q) -> (Q -> P) -> Iff P Q
```

notation：

```text
P ↔ Q
```

Basic theorems:

```npa
theorem Iff.mp  {P Q : Prop} : (P ↔ Q) -> P -> Q := ...
theorem Iff.mpr {P Q : Prop} : (P ↔ Q) -> Q -> P := ...
theorem Iff.refl {P : Prop} : P ↔ P := ...
theorem Iff.symm {P Q : Prop} : (P ↔ Q) -> (Q ↔ P) := ...
theorem Iff.trans {P Q R : Prop} : (P ↔ Q) -> (Q ↔ R) -> (P ↔ R) := ...
```

## 5.7 Exists

```npa
inductive Exists.{u} {A : Sort u} (P : A -> Prop) : Prop where
| intro : (x : A) -> P x -> Exists P
```

notation：

```text
∃ x : A, P x
```

Basic theorems:

```npa
theorem Exists.intro {A : Sort u} {P : A -> Prop} :
  (x : A) -> P x -> Exists P := ...

theorem Exists.elim {A : Sort u} {P : A -> Prop} {Q : Prop} :
  Exists P -> ((x : A) -> P x -> Q) -> Q := ...
```

## 5.8 Std.Logic Theorem Search Metadata

Example:

```json
{
  "name": "Eq.trans",
  "statement": "x = y -> y = z -> x = z",
  "mode": ["apply"],
  "attributes": ["trans"],
  "head": "Eq",
  "axioms_used": []
}
```

To make `apply Eq.trans` usable, give `Eq.trans` a high score in apply search.

---

# 6. Std.Nat

`Std.Nat` is the natural number library.

Purpose:

```text
- basic Nat definitions
- addition and multiplication
- basic equalities
- target of the induction tactic
- main rewrite source for simp-lite
```

## 6.1 Nat

```npa
inductive Nat : Type where
| zero : Nat
| succ : Nat -> Nat
```

notation：

```text
0     => Nat.zero
1     => Nat.succ Nat.zero
2     => Nat.succ (Nat.succ Nat.zero)
```

The Phase 6 MVP does not need overloaded numerals yet. Even if Nat-specific
numeral syntax is added, treat it as a Phase 6 surface convenience, not as
overloaded numerals.

## 6.2 add

For consistency with the earlier design, `Nat.add n m` should recurse on **the
second argument m**.

```npa
def Nat.add (n m : Nat) : Nat :=
  Nat.rec
    (fun _ => Nat)
    n
    (fun _ ih => Nat.succ ih)
    m
```

This makes the following definitional equalities.

```text
Nat.add n Nat.zero
  ≡ n

Nat.add n (Nat.succ m)
  ≡ Nat.succ (Nat.add n m)
```

notation：

```text
n + m
```

## 6.3 Basic Theorems For add

First, the ones provable by refl:

```npa
@[simp]
theorem Nat.add_zero (n : Nat) : n + 0 = n := Eq.refl n

@[simp]
theorem Nat.add_succ (n m : Nat) :
  n + Nat.succ m = Nat.succ (n + m) := Eq.refl _
```

The ones that require induction:

```npa
@[simp]
theorem Nat.zero_add (n : Nat) : 0 + n = n := by
  induction n
  exact Eq.refl 0
  simp-lite

theorem Nat.succ_add (n m : Nat) :
  Nat.succ n + m = Nat.succ (n + m) := by
  induction m
  exact Eq.refl _
  simp-lite

theorem Nat.add_assoc (a b c : Nat) :
  (a + b) + c = a + (b + c) := by
  induction c
  simp-lite
  simp-lite

theorem Nat.add_comm (a b : Nat) :
  a + b = b + a := by
  induction b
  simp-lite
  simp-lite
```

Note:

```text
Do not make Nat.add_comm a simp rule.
```

The reason is that putting commutativity into simp easily causes rewrite loops.

Attribute policy:

```text
Nat.add_zero   @[simp]
Nat.add_succ   @[simp]
Nat.zero_add   @[simp]
Nat.succ_add   not simp initially, or low priority
Nat.add_assoc  not simp
Nat.add_comm   not simp
```

## 6.4 mul

Multiplication also recurses on the second argument.

```npa
def Nat.mul (n m : Nat) : Nat :=
  Nat.rec
    (fun _ => Nat)
    0
    (fun _ ih => ih + n)
    m
```

That is:

```text
n * 0 ≡ 0
n * succ m ≡ n * m + n
```

notation：

```text
n * m
```

Basic theorems:

```npa
@[simp]
theorem Nat.mul_zero (n : Nat) : n * 0 = 0 := Eq.refl 0

@[simp]
theorem Nat.mul_succ (n m : Nat) :
  n * Nat.succ m = n * m + n := Eq.refl _

@[simp]
theorem Nat.zero_mul (n : Nat) : 0 * n = 0 := by
  induction n
  exact Eq.refl 0
  simp-lite

theorem Nat.succ_mul (n m : Nat) :
  Nat.succ n * m = m + n * m := by
  induction m
  simp-lite
  simp-lite

theorem Nat.mul_assoc (a b c : Nat) :
  (a * b) * c = a * (b * c) := ...

theorem Nat.mul_comm (a b : Nat) :
  a * b = b * a := ...

theorem Nat.left_distrib (a b c : Nat) :
  a * (b + c) = a * b + a * c := ...

theorem Nat.right_distrib (a b c : Nat) :
  (a + b) * c = a * c + b * c := ...
```

In the Phase 6 MVP, `mul_assoc`, `mul_comm`, and `distrib` can be deferred to
the second half. Stabilizing the `add` family first is the priority.

## 6.5 pred / one

For convenience, `one` and `pred` may also be defined.

```npa
def Nat.one : Nat := Nat.succ Nat.zero

def Nat.pred (n : Nat) : Nat :=
  Nat.rec
    (fun _ => Nat)
    0
    (fun k _ => k)
    n
```

notation：

```text
1 => Nat.one
```

Basic theorems:

```npa
@[simp]
theorem Nat.pred_zero : Nat.pred 0 = 0 := Eq.refl 0

@[simp]
theorem Nat.pred_succ (n : Nat) : Nat.pred (Nat.succ n) = n := Eq.refl n
```

## 6.6 Nat simp-lite Rules

Safe to include:

```text
Nat.add_zero    : n + 0 -> n
Nat.add_succ    : n + succ m -> succ (n + m)
Nat.zero_add    : 0 + n -> n
Nat.mul_zero    : n * 0 -> 0
Nat.mul_succ    : n * succ m -> n * m + n
Nat.zero_mul    : 0 * n -> 0
Nat.pred_zero   : pred 0 -> 0
Nat.pred_succ   : pred (succ n) -> n
```

Do not include, or include carefully:

```text
Nat.add_comm
Nat.mul_comm
Nat.add_assoc
Nat.mul_assoc
Nat.left_distrib
Nat.right_distrib
```

These are safer to leave to future `ring` or `normalize_nat`-style tactics
rather than `simp-lite`.

## 6.7 Std.Nat Search Metadata Example

```json
{
  "name": "Nat.add_zero",
  "statement": "∀ n : Nat, n + 0 = n",
  "attributes": ["simp", "rw"],
  "rewrite": {
    "lhs": "n + 0",
    "rhs": "n",
    "orientation": "left_to_right",
    "safe": true
  },
  "suggested_tactics": [
    "simp-lite",
    "rw [Nat.add_zero]",
    "exact Nat.add_zero n"
  ],
  "axioms_used": []
}
```

---

# 7. Std.List

`Std.List` is the list library.

Purpose:

```text
- testing polymorphic inductives
- preparation for List induction
- basic theorems for append / length / map
- improving practical usefulness of simp-lite
```

## 7.1 List

```npa
inductive List.{u} (A : Type u) : Type u where
| nil  : List A
| cons : A -> List A -> List A
```

notation：

```text
[]          => List.nil
x :: xs     => List.cons x xs
xs ++ ys    => List.append xs ys
```

In the Phase 6 MVP, list literals `[a, b, c]` can be postponed.

## 7.2 append

It is natural for `append xs ys` to recurse on the first argument.

```npa
def List.append {A : Type u} (xs ys : List A) : List A :=
  List.rec
    (fun _ => List A)
    ys
    (fun x xs ih => List.cons x ih)
    xs
```

definitional equality：

```text
[] ++ ys ≡ ys

(x :: xs) ++ ys ≡ x :: (xs ++ ys)
```

Basic theorems:

```npa
@[simp]
theorem List.nil_append {A : Type u} (xs : List A) :
  [] ++ xs = xs := Eq.refl xs

@[simp]
theorem List.cons_append {A : Type u} (x : A) (xs ys : List A) :
  (x :: xs) ++ ys = x :: (xs ++ ys) := Eq.refl _

@[simp]
theorem List.append_nil {A : Type u} (xs : List A) :
  xs ++ [] = xs := by
  induction xs
  exact Eq.refl []
  simp-lite

theorem List.append_assoc {A : Type u} (xs ys zs : List A) :
  (xs ++ ys) ++ zs = xs ++ (ys ++ zs) := by
  induction xs
  simp-lite
  simp-lite
```

Attribute policy:

```text
List.nil_append   @[simp]
List.cons_append  @[simp]
List.append_nil   @[simp]
List.append_assoc not simp
```

If `append_assoc` is put into simp, terms may keep transforming depending on
rewrite direction, so it is not included in the MVP.

## 7.3 length

```npa
def List.length {A : Type u} (xs : List A) : Nat :=
  List.rec
    (fun _ => Nat)
    0
    (fun _ _ ih => Nat.succ ih)
    xs
```

Basic theorems:

```npa
@[simp]
theorem List.length_nil {A : Type u} :
  List.length ([] : List A) = 0 := Eq.refl 0

@[simp]
theorem List.length_cons {A : Type u} (x : A) (xs : List A) :
  List.length (x :: xs) = Nat.succ (List.length xs) := Eq.refl _

theorem List.length_append {A : Type u} (xs ys : List A) :
  List.length (xs ++ ys) = List.length xs + List.length ys := by
  induction xs
  simp-lite
  simp-lite
```

## 7.4 map

```npa
def List.map {A : Type u} {B : Type v}
  (f : A -> B) (xs : List A) : List B :=
  List.rec
    (fun _ => List B)
    []
    (fun x xs ih => f x :: ih)
    xs
```

Basic theorems:

```npa
@[simp]
theorem List.map_nil {A : Type u} {B : Type v} (f : A -> B) :
  List.map f [] = [] := Eq.refl []

@[simp]
theorem List.map_cons {A : Type u} {B : Type v}
  (f : A -> B) (x : A) (xs : List A) :
  List.map f (x :: xs) = f x :: List.map f xs := Eq.refl _

@[simp]
theorem List.map_id {A : Type u} (xs : List A) :
  List.map (fun x => x) xs = xs := by
  induction xs
  simp-lite
  simp-lite

theorem List.map_comp {A : Type u} {B : Type v} {C : Type w}
  (f : B -> C) (g : A -> B) (xs : List A) :
  List.map f (List.map g xs) = List.map (fun x => f (g x)) xs := by
  induction xs
  simp-lite
  simp-lite
```

## 7.5 foldr / foldl

In the Phase 6 MVP, `foldr` alone is enough.

```npa
def List.foldr {A : Type u} {B : Type v}
  (f : A -> B -> B) (init : B) (xs : List A) : B :=
  List.rec
    (fun _ => B)
    init
    (fun x xs ih => f x ih)
    xs
```

Basic theorems:

```npa
@[simp]
theorem List.foldr_nil :
  List.foldr f init [] = init := Eq.refl init

@[simp]
theorem List.foldr_cons :
  List.foldr f init (x :: xs) = f x (List.foldr f init xs) := Eq.refl _
```

`foldl` can be postponed.

## 7.6 Std.List simp-lite Rules

Safe to include:

```text
List.nil_append
List.cons_append
List.append_nil
List.length_nil
List.length_cons
List.map_nil
List.map_cons
List.map_id
List.foldr_nil
List.foldr_cons
```

Do not include:

```text
List.append_assoc
List.map_comp
List.length_append
```

These are useful, but including them in simp-lite requires careful direction
control.

---

# 8. Std.Algebra.Basic

`Std.Algebra.Basic` is the minimal library for algebraic structures.

Purpose:

```text
- general definitions of associativity / commutativity / identity
- foundation for treating Nat.add and List.append as general structures
- preparation for future typeclass / ring / group / monoid tactics
```

At Phase 6, it is better not to add full typeclass resolution yet. Therefore,
start by designing these as **explicit structures**.

---

## 8.1 Basic Predicates

First, before making structures, define properties of operations.

```npa
def Associative {A : Type u} (op : A -> A -> A) : Prop :=
  forall a b c : A, op (op a b) c = op a (op b c)

def Commutative {A : Type u} (op : A -> A -> A) : Prop :=
  forall a b : A, op a b = op b a

def LeftIdentity {A : Type u} (e : A) (op : A -> A -> A) : Prop :=
  forall a : A, op e a = a

def RightIdentity {A : Type u} (e : A) (op : A -> A -> A) : Prop :=
  forall a : A, op a e = a
```

These are very easy to search.

Examples:

```text
Associative Nat.add
Commutative Nat.add
LeftIdentity 0 Nat.add
RightIdentity 0 Nat.add
```

## 8.2 IsSemigroup / IsMonoid

In Phase 6, it is simpler to start with unbundled properties rather than bundled
carriers.

```npa
inductive IsSemigroup {A : Type u} (op : A -> A -> A) : Prop where
| intro : Associative op -> IsSemigroup op
```

```npa
inductive IsMonoid {A : Type u} (op : A -> A -> A) (e : A) : Prop where
| intro :
    Associative op ->
    LeftIdentity e op ->
    RightIdentity e op ->
    IsMonoid op e
```

```npa
inductive IsCommMonoid {A : Type u} (op : A -> A -> A) (e : A) : Prop where
| intro :
    Associative op ->
    Commutative op ->
    LeftIdentity e op ->
    RightIdentity e op ->
    IsCommMonoid op e
```

Advantages of this design:

```text
- usable without typeclasses
- simple for the kernel
- treatable as inductives inside Prop
- easy for theorem search
```

Disadvantages:

```text
- op and e must be explicit every time
- there is not yet Lean-style typeclass automatic inference
```

Phase 6 accepts these disadvantages.

---

## 8.3 Projection Theorems

Provide theorems that extract each property from `IsMonoid`.

```npa
theorem IsMonoid.assoc {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> Associative op := ...

theorem IsMonoid.left_id {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> LeftIdentity e op := ...

theorem IsMonoid.right_id {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> RightIdentity e op := ...
```

`IsCommMonoid` is similar.

```npa
theorem IsCommMonoid.assoc : IsCommMonoid op e -> Associative op := ...
theorem IsCommMonoid.comm : IsCommMonoid op e -> Commutative op := ...
theorem IsCommMonoid.left_id : IsCommMonoid op e -> LeftIdentity e op := ...
theorem IsCommMonoid.right_id : IsCommMonoid op e -> RightIdentity e op := ...
```

These are easy to use in apply search.

## 8.4 identity uniqueness

Also include basic general theorems.

```npa
theorem identity_unique
  {A : Type u}
  {op : A -> A -> A}
  {e₁ e₂ : A}
  (h₁ : LeftIdentity e₁ op)
  (h₂ : RightIdentity e₂ op)
  : e₁ = e₂ := by
  -- h₁ e₂ : op e₁ e₂ = e₂
  -- h₂ e₁ : op e₁ e₂ = e₁
  ...
```

General theorems like this have a large effect on future library quality.

## 8.5 Register Nat.add As A Commutative Monoid

`Std.Algebra.Basic` itself should not depend on `Std.Nat`, but a separate module
such as:

```text
Std.Nat.Algebra
```

can be created later and used for registration.

```npa
theorem Nat.add_is_comm_monoid :
  IsCommMonoid Nat.add 0 := ...
```

Within Phase 6, `Std.Algebra.Basic` can contain only structure definitions, with
concrete examples postponed.

Do not add the following `Std.Algebra.Basic`-derived theorems to the `Std.Nat`
MVP release module itself. Adding them would make `Std.Nat` depend on
`Std.Algebra.Basic`, conflicting with the direct import / closure of
`std.nat.mvp`. If needed for Phase 6 tests, a non-release-module test fixture or
future `Std.Nat.Algebra` can import both `Std.Nat` and `Std.Algebra.Basic` and
prove the following.

```npa
theorem Nat.add_associative : Associative Nat.add := Nat.add_assoc
theorem Nat.add_commutative : Commutative Nat.add := Nat.add_comm
theorem Nat.add_left_identity : LeftIdentity 0 Nat.add := Nat.zero_add
theorem Nat.add_right_identity : RightIdentity 0 Nat.add := Nat.add_zero
```

---

# 9. Recommended File Structure For Each Module

Four files are enough at first, but for the future, split them like this.

```text
Std/
  Logic/
    Basic.npa
    Eq.npa
    Connectives.npa

  Nat/
    Basic.npa
    Add.npa
    Mul.npa

  List/
    Basic.npa
    Append.npa
    Length.npa
    Map.npa
    Fold.npa

  Algebra/
    Basic.npa
```

In the Phase 6 MVP, external public names are grouped as follows.

```text
Std.Logic
Std.Nat
Std.List
Std.Algebra.Basic
```

Only these four are MVP release modules. `Std.Nat.Basic` and `Std.Logic.Eq`
used by historical Human/frontend compatibility fixtures are neither release
modules nor source package roots. Similarly, even if legacy fixtures exist,
`Std/Nat/Basic.npa` and `Std/Logic/Eq.npa` are not included in the
`npa.stdlib.mvp.v1` package locator table. Release package source / certificate
paths are limited to the fixed four-module layout such as `Std/Nat.npa` /
`Std/Nat.npcert` and `Std/Logic.npa` / `Std/Logic.npcert`.

Internal imports:

```text
Std.Logic
  imports no Phase 2 module
  checked against kernel/core profile

Std.Nat
  imports Std.Logic

Std.List
  imports Std.Logic, Std.Nat

Std.Algebra.Basic
  imports Std.Logic
```

---

# 10. Connection With Certificate / Hash Design

Each module has `.npcert`.

Example:

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Nat",
  "imports": [
    {
      "module": "Std.Logic",
      "export_hash": "sha256:...",
      "certificate_hash": "sha256:..."
    }
  ],
  "declarations": [
    "Nat",
    "Nat.zero",
    "Nat.succ",
    "Nat.add",
    "Nat.add_zero",
    "Nat.zero_add"
  ],
  "export_block": [
    {
      "name": "Nat.add_zero",
      "decl_interface_hash": "sha256:..."
    },
    {
      "name": "Nat.zero_add",
      "decl_interface_hash": "sha256:..."
    }
  ],
  "axiom_report": {
    "module_axioms": ["Std.Logic.Eq.rec"],
    "per_declaration": []
  },
  "hashes": {
    "export_hash": "sha256:...",
    "certificate_hash": "sha256:...",
    "axiom_report_hash": "sha256:..."
  }
}
```

`ExportEntry.name` is the declaration name exported by the certificate itself.
Do not concatenate it with the `module` field to create synthetic names such as
`Std.Nat.add_zero`.

Important rules:

```text
- def bodies are included in export_hash
- proof bodies of opaque theorems do not need to be included in export_hash
- theorem axiom dependencies are included in export_hash
- build fails if an import export_hash does not match; in high-trust mode, certificate_hash must also match
```

---

# 11. theorem index

In Phase 6, generate theorem-search indexes from each module.

## 11.1 IndexEntry

```json
{
  "name": "List.append_nil",
  "module": "Std.List",
  "statement": "∀ xs : List A, xs ++ [] = xs",
  "head": "Eq",
  "constants": [
    "List.append",
    "List.nil",
    "Eq"
  ],
  "attributes": ["simp", "rw"],
  "rewrite": {
    "lhs": "xs ++ []",
    "rhs": "xs",
    "orientation": "left_to_right",
    "safe": true
  },
  "axioms_used": [],
  "proof_term_size": null,
  "suggested_tactics": [
    "simp-lite",
    "rw [List.append_nil]"
  ]
}
```

This JSON is a sketch of a human-facing theorem search view. The authoritative
schema for the AI Profile is `MachineStdTheoremIndex` / `MachineStdTheoremEntry`,
and reconstructs
`global_ref`, `statement_core_hash`, `statement_head`, `modes`, `rewrite_descriptors`,
`axiom_dependencies`, and similar fields from certificate verifier output and
rewrite/simp profiles. In the MVP machine theorem index, `proof_term_size =
null`; non-null is rejected as stale metadata. The `rewrite` object above is
convenient for human-facing display, but corresponds to validated
`rewrite_descriptors` in AI artifacts.

## 11.2 Search Categories

Classify each theorem into the following categories.

```text
exact:
  likely to directly match the target

apply:
  conclusion matches the target and turns premises into subgoals

rw:
  can rewrite as Eq lhs rhs

simp:
  usable by simp-lite

induction:
  recursor / induction theorem used by induction
```

Examples:

```text
Eq.refl              exact, generated constructor callable; not a theorem-index entry
Eq.trans             apply
Nat.add_zero         exact/rw/simp
Nat.add_comm         rw, but not simp
List.append_assoc    rw, but not simp
And.intro            apply/intro
False.elim           apply/elim
```

Here, `intro` / `elim` are Human IDE display/search categories. The AI MVP
theorem index does not emit `Intro` / `Elim` attributes; those theorems are
treated as `apply` mode / `apply` attributes.

---

# 12. simp-lite Database

One of the most important Phase 6 results is that `simp-lite` becomes usable.

## 12.1 Initial simp Set

```text
Std.Logic:
  none in Phase 6 AI MVP
  Eq.refl is a generated constructor used through the Eq family, not a SimpRuleRef

Std.Nat:
  Nat.add_zero
  Nat.add_succ
  Nat.zero_add
  Nat.mul_zero
  Nat.mul_succ
  Nat.zero_mul
  Nat.pred_zero
  Nat.pred_succ

Std.List:
  List.nil_append
  List.cons_append
  List.append_nil
  List.length_nil
  List.length_cons
  List.map_nil
  List.map_cons
  List.map_id
  List.foldr_nil
  List.foldr_cons
```

## 12.2 Theorems Not Included In simp

```text
Nat.add_comm
Nat.mul_comm
Nat.add_assoc
Nat.mul_assoc
List.append_assoc
List.map_comp
List.length_append
```

Reasons:

```text
- loop risk
- normal-form policy is needed
- too heavy for simp-lite
```

This list is the list of **theorems not included in simp-lite**. In the Phase 6
AI MVP rewrite profile, only the following are emitted as exact `RwOnly`.

```text
Nat.add_comm
Nat.add_assoc
List.append_assoc
List.length_append
```

By contrast, theorems such as `Nat.mul_comm` / `Nat.mul_assoc` /
`List.map_comp` that are not in the fixed `RwOnly` set of the AI Profile are
not emitted in the `npa.stdlib.mvp.v1` rewrite profile either. Stronger
normalization and commutative/associative transformations are left to future
dedicated tactics.

```text
assoc-normalizer
comm-normalizer
ring
monoid_normalize
```

---

# 13. Library Tests

In Phase 6, test not only proofs of each theorem, but also APIs, tactics, and
search.

## 13.1 Kernel check test

For every declaration:

```text
proof : theorem_type
value : def_type
inductive declaration ok
```

check the following.

## 13.2 Certificate check test

```text
- can be rechecked with only .npcert
- can be checked without source
- import export_hash matches, and in high-trust mode certificate_hash also matches
- declaration hash matches
- axiom report matches recomputed results
```

## 13.3 No axiom test

The Phase 6 standard library is axiom-free in principle. However, only if the
current kernel represents the standard `Eq.rec` family head as an `AxiomDecl` in
`Std.Logic`, the exact `Std.Logic.Eq.rec` may be explicitly listed as a
kernel-standard exception in the axiom report / allowlist. Arbitrary custom
axioms, `Classical.choice`, `funext`, and `propext` are not included in the MVP
standard library.

```json
{
  "module": "Std.Nat",
  "custom_axioms_used": [],
  "standard_axiom_exceptions": ["imported Std.Logic Eq.rec"],
  "contains_sorry": false
}
```

## 13.4 simp-lite test

Examples:

```npa
theorem test_nat_add_zero (n : Nat) : n + 0 = n := by
  simp-lite
```

```npa
theorem test_list_append_nil {A : Type} (xs : List A) :
  xs ++ [] = xs := by
  simp-lite
```

## 13.5 theorem search test

goal：

```text
n + 0 = n
```

Expected search result:

```text
Nat.add_zero
```

goal：

```text
xs ++ [] = xs
```

Expected search result:

```text
List.append_nil
```

goal：

```text
x = z
```

context：

```text
h1 : x = y
h2 : y = z
```

Expected candidate:

```text
apply Eq.trans
```

## 13.6 tactic regression test

Confirm that each tactic works on the standard library.

```text
intro:
  theorem id_nat : Nat -> Nat

exact:
  theorem refl_nat (n : Nat) : n = n

apply:
  Eq.trans examples

rw:
  rewrite by Nat.add_zero / List.append_nil

simp-lite:
  Nat/List simplification

induction:
  Nat.zero_add / List.append_nil
```

---

# 14. Phase 6 Implementation Order

The recommended order is:

```text
1. Std.Logic.Eq
   Eq.refl, Eq.symm, Eq.trans, Eq.subst, Eq.congrArg

2. Std.Logic.Connectives
   True, False, Not, And, Or, Iff, Exists

3. Std.Nat.Basic
   Nat, zero, succ, one, pred

4. Std.Nat.Add
   add, add_zero, add_succ, zero_add, succ_add, add_assoc, add_comm

5. Std.Nat.Mul
   mul, mul_zero, mul_succ, zero_mul, basic multiplication theorems

6. Std.List.Basic
   List, nil, cons

7. Std.List.Append
   append, nil_append, cons_append, append_nil, append_assoc

8. Std.List.Length
   length, length_nil, length_cons, length_append

9. Std.List.Map
   map, map_nil, map_cons, map_id, map_comp

10. Std.Algebra.Basic
    Associative, Commutative, LeftIdentity, RightIdentity,
    IsSemigroup, IsMonoid, IsCommMonoid

11. theorem index generation

12. simp-lite database generation

13. regression tests
```

---

# 15. Things Not Yet Included In Phase 6

To keep the MVP small, postpone the following.

```text
- full typeclass system
- coercions
- overloaded algebraic notation
- integers, rationals, reals
- finite sets
- options, arrays, trees
- quotient types
- classical choice
- function extensionality
- proof irrelevance axiom
- groups, rings, fields
- order theory
- category theory
- full simp
- ring / omega / linarith
```

In particular, even if `Classical.choice`, `funext`, and `propext` are added,
put them in a separate module such as:

```text
Std.Classical
```

and list them explicitly in the axiom report.

It is desirable to keep `Std.Logic` itself constructive.

---

# 16. Phase 6 Completion Criteria

Phase 6 can be considered complete when the following conditions hold.

```text
- Std.Logic provides Eq / connectives / Exists
- Std.Nat provides add / mul / basic theorems
- Std.List provides append / length / map / foldr
- Std.Algebra.Basic provides Associative / Commutative / Monoid-style definitions
- all modules are no sorry / no custom axiom
- for kernels that represent `Eq.rec` as a standard axiom, only that exact exception appears in the axiom report / allowlist
- if Nat/List/Algebra proofs use `Eq.rec`, the entry may appear as imported `Std.Logic Eq.rec` in verifier-derived
  `module_axioms` / `transitive_axioms`, but no custom axioms appear
- all modules are recheckable as .npcert
- import entries have export_hash, and certificate_hash is also generated for high-trust mode
- module export_hash / certificate_hash / axiom_report_hash are generated
- theorem index is generated
- simp-lite can close basic Nat/List goals
- theorem search can return exact/apply/rw/simp candidates
- reproving tests for basic theorems pass using only Phase 4 tactics
```

In implementation, this is traced mainly with the following regressions.

```text
source package / certificate:
  builds_mvp_certificate_artifacts_from_source_package
  source_built_std_artifacts_feed_machine_release_sessions_retrieval_and_audit
  manifest fixes module membership/certificate paths; source skeleton fixes import intent; Rust builders generate certificate contents

release identity / generated artifact boundary:
  machine_release_identity_ignores_human_source_layout_and_debug_views
  source_built_std_release_rejects_stale_machine_artifact_refs

fixed module and legacy fixture boundary:
  fixes_mvp_source_layout_without_expanding_release_modules
  docs_pin_human_ai_stdlib_release_contracts

simp / rw / axiom policy:
  classifies_std_nat_add_rules_as_simp_safe_or_rw_only
  classifies_std_list_rules_and_keeps_late_map_theorems_out_of_mvp_profiles
  mvp_certificate_loader_rejects_custom_axioms
  mvp_certificate_loader_rejects_nonstandard_eq_rec_exception

tactic / search handoff:
  phase6_human_real_stdlib_phase4_tactic_regressions_compile
  phase7_human_real_stdlib_search_missing_result_regressions
  std_library_release_artifacts_drive_m8_search_candidates_through_machine_api_batch
```

---

# 17. In One Sentence

Phase 6 is **the stage that places the first trustworthy standard mathematics
library on top of the kernel, certificates, tactics, and IDE/API**.

The first standard library to build is:

```text
Std.Logic:
  foundation for equality and propositional logic

Std.Nat:
  natural numbers, addition, multiplication, induction, basic rewrites

Std.List:
  polymorphic data、append、length、map

Std.Algebra.Basic:
  abstract structures for associativity, commutativity, identities, and monoids
```

Once this Phase 6 is complete, later Phase 7 work can seriously layer on **AI
proof search, RAG, premise selection, and proof search**.
