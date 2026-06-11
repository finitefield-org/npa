> Implementation starts from [`crates/npa-kernel`](../crates/npa-kernel).
> Phase 1 is complete. Tests that directly assemble core terms check `id`, `const`, `Nat.add`, and `add_zero`.
> `Nat` and `Eq` are added to the environment through `InductiveDecl`, and recursor iota is handled by the generic path based on `RecursorRules` derived from inductives.

This list is the **minimum set that the small Phase 1 kernel must handle**.
Classified by category:

```text
Core term:
  Sort, Pi, Lambda, App, Let, Const

Initial library / inductives:
  Nat, Eq, simple inductive

Computation rules:
  β, δ, ι, ζ reduction
```

In other words, the Phase 1 goal is not tactics, parsers, or AI. It is a kernel
that can directly receive fully explicit core terms / declarations and check
whether they are correct.

---

# 1. Minimal Phase 1 Goal

An example of what Phase 1 should accept is:

```text
id : Π A : Sort u, A → A
id := λ A, λ x, x
```

Next:

```text
Nat : Type
Nat.zero : Nat
Nat.succ : Nat → Nat
```

And:

```text
Eq : Π A : Sort u, A → A → Prop
Eq.refl : Π A : Sort u, Π x : A, Eq A x x
```

Eventually, the goal is to check a theorem like the following as a certificate.

```text
theorem add_zero :
  Π n : Nat, Eq Nat (Nat.add n Nat.zero) n
```

Here, if `Nat.add` is defined to recurse on the second argument, then
`Nat.add n Nat.zero` reduces to `n` by computation alone, so the proof is
essentially just `Eq.refl`.

---

# 2. Core Term Syntax

The following core terms are sufficient for Phase 1.

```text
Term ::=
  Sort Level
| BVar Index
| Const Name [Level]
| App Term Term
| Lam Name Type Body
| Pi  Name Type Body
| Let Name Type Value Body
```

Do not include surface syntax conveniences.

Not included:

```text
- notation
- implicit arguments
- typeclass
- tactic
- pattern matching syntax
- unresolved metavariable
- source-level match
- natural language annotation
```

The kernel checks only fully explicit core terms.

In Rust style, an arena + ID approach is preferable to a direct tree structure.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ExprId(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct LevelId(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct NameId(u32);

enum ExprKind {
    Sort(LevelId),
    BVar(u32),
    Const(NameId, Vec<LevelId>),
    App(ExprId, ExprId),
    Lam {
        binder: NameId,
        ty: ExprId,
        body: ExprId,
    },
    Pi {
        binder: NameId,
        ty: ExprId,
        body: ExprId,
    },
    Let {
        binder: NameId,
        ty: ExprId,
        value: ExprId,
        body: ExprId,
    },
}
```

`NameId` is for debugging. Actual binding structure is handled with de Bruijn
indexes.

---

# 3. Sort

`Sort` is the hierarchy of types.

```text
Sort 0        = Prop
Sort 1        = Type 0
Sort 2        = Type 1
...
```

The basic rule is:

```text
Sort u : Sort (succ u)
```

That is:

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
```

In Phase 1, universe levels are fixed as follows to match core-spec v0.1.

```text
Level ::= 0 | succ Level | max Level Level | imax Level Level | param Name
```

Sort computation for `Π x : A, B` uses `imax`. This keeps `∀ x : A, P : Prop`
in `Prop`.

---

# 4. Pi

`Pi` is a dependent function type.

```text
Pi x : A, B
```

At the surface level:

```text
∀ x : A, B
```

Or:

```text
(x : A) → B
```

corresponds to it.

Typing rule:

```text
Γ ⊢ A : Sort u
Γ, x : A ⊢ B : Sort v
────────────────────────────
Γ ⊢ Pi x : A, B : Sort (imax u v)
```

In Phase 1, non-dependent function types are also represented by `Pi`.

```text
A → B
```

is internally:

```text
Pi _ : A, B
```

.

---

# 5. Lambda

`Lambda` is function abstraction.

```text
Lam x : A, body
```

Typing rule:

```text
Γ ⊢ A : Sort u
Γ, x : A ⊢ body : B
────────────────────────────────
Γ ⊢ Lam x : A, body : Pi x : A, B
```

Example:

```text
λ A : Sort u, λ x : A, x
```

has type:

```text
Π A : Sort u, A → A
```

.

At this point, `id` can be checked.

---

# 6. App

`App` is function application.

```text
App f a
```

Typing rule:

```text
Γ ⊢ f : Pi x : A, B
Γ ⊢ a : A
────────────────────
Γ ⊢ App f a : B[x := a]
```

In implementation, directly inspecting the type of `f` is not enough. First
reduce it to weak-head normal form.

```text
infer(f) = T
whnf(T) = Pi x : A, B
```

That is, reduce `T` to a `Pi` before checking argument `a`.

Pseudocode:

```rust
fn infer_app(env: &Env, ctx: &Ctx, f: ExprId, a: ExprId) -> Result<ExprId> {
    let f_ty = infer(env, ctx, f)?;
    let f_ty_whnf = whnf(env, ctx, f_ty)?;

    match env.expr(f_ty_whnf) {
        ExprKind::Pi { ty: dom, body, .. } => {
            check(env, ctx, a, dom)?;
            Ok(instantiate(body, a))
        }
        _ => Err(Error::ExpectedFunctionType),
    }
}
```

---

# 7. Let

`Let` is a local definition.

```text
Let x : A := v in body
```

Typing rule:

```text
Γ ⊢ A : Sort u
Γ ⊢ v : A
Γ, x : A := v ⊢ body : B
────────────────────────────
Γ ⊢ Let x : A := v in body : B[x := v]
```

`Let` is unfolded by zeta reduction.

```text
let x := v in body
  ↦ body[x := v]
```

In implementation, it may be inserted into the context as a local definition.

```rust
enum LocalDecl {
    Assumption {
        name: NameId,
        ty: ExprId,
    },
    Definition {
        name: NameId,
        ty: ExprId,
        value: ExprId,
    },
}
```

---

# 8. Const

`Const` is a constant registered in the global environment.

```text
Const Nat []
Const Nat.zero []
Const Eq [u]
Const Nat.rec [u]
```

`Const` gets its type from environment `Env`.

```text
Σ(c) = declaration of c
────────────────────────
Γ ⊢ Const c levels : instantiate_universes(type(c), levels)
```

The following declarations are sufficient for Phase 1.

```rust
enum Decl {
    Axiom {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: ExprId,
    },
    Def {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: ExprId,
        value: ExprId,
        reducibility: Reducibility,
    },
    Theorem {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: ExprId,
        proof: ExprId,
    },
    Inductive {
        data: InductiveDecl,
    },
}
```

`Def` can be unfolded by delta reduction. `Theorem` should basically be opaque
and not unfolded during conversion.

---

# 9. Nat

`Nat` is the first inductive type.

```text
inductive Nat : Type where
| zero : Nat
| succ : Nat → Nat
```

The core environment contains:

```text
Nat      : Sort 1
Nat.zero : Nat
Nat.succ : Nat → Nat
Nat.rec  : ...
```

Conceptually, the type of `Nat.rec` is:

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

This `Nat.rec` computation is iota reduction.

---

# 10. Eq

`Eq` is equality.

```text
inductive Eq.{u} (A : Sort u) (a : A) : A → Prop where
| refl : Eq A a a
```

The core environment contains:

```text
Eq      : Π A : Sort u, A → A → Prop
Eq.refl : Π A : Sort u, Π x : A, Eq A x x
```

`Eq` represents a proved equality.

Note that `Eq` and definitional equality are different things.

```text
definitional equality:
  equality that the kernel regards as the same by computation

Eq:
  equality as a logical proposition
```

For example:

```text
Nat.add n Nat.zero
```

if this reduces to `n` by reduction,

```text
Eq Nat (Nat.add n Nat.zero) n
```

then this has the same type as `Eq Nat n n` by definitional equality and can be
proved with `Eq.refl Nat n`.

---

# 11. Simple Inductive

Phase 1 `simple inductive` means that not all inductive features are implemented
from the beginning.

Supported:

```text
- a single inductive type
- parameters
- minimal index support for Eq
- finitely many constructors
- strict positivity
- recursor generation
```

Not supported yet:

```text
- mutual inductive
- nested inductive
- coinductive
- quotient
- higher inductive
- complex universe polymorphism
- detailed exceptions for large elimination
```

An internal representation for inductive declarations could be:

```rust
struct InductiveDecl {
    name: NameId,
    universe_params: Vec<NameId>,
    params: Vec<Binder>,
    indices: Vec<Binder>,
    sort: ExprId,
    constructors: Vec<ConstructorDecl>,
}

struct ConstructorDecl {
    name: NameId,
    ty: ExprId,
}
```

`Nat` has no indexes.

```text
Nat : Sort 1
zero : Nat
succ : Nat → Nat
```

`Eq` has indexes.

```text
Eq.{u} (A : Sort u) (a : A) : A → Prop
refl : Eq A a a
```

For Phase 1 implementation, the following order is realistic.

```text
1. Special-case Nat and Eq inside the kernel and make them work
2. Then implement the generic simple inductive checker
3. Make Nat and Eq pass as generic inductive declarations too
4. Remove the special cases
```

Eventually, `Nat` and `Eq` should be treated not as special syntax but as groups
of `Const`s generated from `InductiveDecl`.

---

# 12. Beta / Delta / Iota / Zeta Reduction

The Phase 1 conversion checker handles the following four reductions.

```text
β: beta
δ: delta
ι: iota
ζ: zeta
```

## 12.1 β-reduction

Function application.

```text
(λ x : A, body) a
  ↦ body[x := a]
```

Example:

```text
(λ x : Nat, x) Nat.zero
  ↦ Nat.zero
```

## 12.2 δ-reduction

Definition unfolding.

```text
Const c ↦ value(c)
```

However, unfold only when `c` is a `reducible` definition.

```text
def Nat.add := ...
```

may be unfolded.

```text
theorem Nat.add_zero := ...
```

is basically not unfolded.

Phase 1 reducibility can be this simple.

```rust
enum Reducibility {
    Reducible,
    Opaque,
}
```

Later:

```text
reducible
semireducible
irreducible
opaque
```

can be split out.

## 12.3 ι-reduction

This is computation when a recursor is applied to a constructor.

Example for `Nat.rec`:

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

Without this rule, recursive functions over natural numbers cannot compute.

For example:

```text
Nat.add n Nat.zero
```

if this is defined with `Nat.rec`, iota reduction lets it reduce to `n`.

## 12.4 ζ-reduction

Let unfolding.

```text
let x := v in body
  ↦ body[x := v]
```

---

# 13. WHNF And Conversion Checker

In the kernel, use weak-head normal form, WHNF, before trying to fully normalize
all terms.

```text
whnf(t)
```

reduces only the head of the term as much as needed.

Examples:

```text
whnf((λ x, body) a) = whnf(body[x := a])
```

```text
whnf(let x := v in body) = whnf(body[x := v])
```

```text
whnf(Const c) = whnf(value(c))  // when c is reducible
```

```text
whnf(Nat.rec motive z s Nat.zero) = whnf(z)
```

The conversion checker is roughly:

```rust
fn is_defeq(env: &Env, ctx: &Ctx, a: ExprId, b: ExprId) -> bool {
    let a = whnf(env, ctx, a);
    let b = whnf(env, ctx, b);

    if a == b {
        return true;
    }

    match (env.expr(a), env.expr(b)) {
        (ExprKind::Sort(u), ExprKind::Sort(v)) => level_eq(u, v),

        (ExprKind::BVar(i), ExprKind::BVar(j)) => i == j,

        (ExprKind::Const(c1, us1), ExprKind::Const(c2, us2)) => {
            c1 == c2 && levels_eq(us1, us2)
        }

        (ExprKind::App(f1, x1), ExprKind::App(f2, x2)) => {
            is_defeq(env, ctx, f1, f2)
                && is_defeq(env, ctx, x1, x2)
        }

        (ExprKind::Pi { ty: a1, body: b1, .. },
         ExprKind::Pi { ty: a2, body: b2, .. }) => {
            is_defeq(env, ctx, a1, a2)
                && is_defeq_under_binder(env, ctx, b1, b2)
        }

        (ExprKind::Lam { ty: a1, body: b1, .. },
         ExprKind::Lam { ty: a2, body: b2, .. }) => {
            is_defeq(env, ctx, a1, a2)
                && is_defeq_under_binder(env, ctx, b1, b2)
        }

        _ => false,
    }
}
```

Phase 1 does not need to include:

```text
- η-conversion
- proof irrelevance conversion
- quotient computation
- aggressive theorem unfolding
```

Adding these too early makes the conversion checker too complex.

---

# 14. Phase 1 Type Checking Algorithm

The minimal type checking API is:

```rust
fn infer(env: &Env, ctx: &Ctx, t: ExprId) -> Result<ExprId>;

fn check(env: &Env, ctx: &Ctx, t: ExprId, expected: ExprId) -> Result<()> {
    let actual = infer(env, ctx, t)?;
    if is_defeq(env, ctx, actual, expected) {
        Ok(())
    } else {
        Err(Error::TypeMismatch { actual, expected })
    }
}
```

Handling for each term:

```text
Sort:
  infer(Sort u) = Sort (succ u)

BVar:
  get the type from the context

Const:
  get the type from the environment and substitute universe levels

Pi:
  check that the domain and codomain are Sorts, then return a Sort

Lam:
  infer the body type, then return a Pi type

App:
  reduce the function type to WHNF, check that it is a Pi, then check the argument

Let:
  check the value, then infer the body in a context with the local definition
```

---

# 15. Tests To Build In Phase 1

## 15.1 `id`

```text
id.{u} :
  Π A : Sort u, A → A

id :=
  λ A : Sort u,
  λ x : A,
    x
```

This verifies:

```text
Sort
Pi
Lambda
BVar
universe param
```

## 15.2 `const`

```text
const.{u v} :
  Π A : Sort u, Π B : Sort v, A → B → A

const :=
  λ A, λ B, λ x, λ y, x
```

This verifies multiple binders and de Bruijn indexes.

## 15.3 `Nat`

```text
Nat : Sort 1
Nat.zero : Nat
Nat.succ : Nat → Nat
```

This verifies inductive declarations.

## 15.4 `Nat.rec`

```text
Nat.rec motive z s Nat.zero
  ≡ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ≡ s n (Nat.rec motive z s n)
```

This verifies iota reduction.

## 15.5 `Eq.refl`

```text
Eq.refl Nat Nat.zero :
  Eq Nat Nat.zero Nat.zero
```

This verifies the basics of Eq.

## 15.6 `Nat.add`

Define it to recurse on the second argument.

```text
Nat.add : Nat → Nat → Nat

Nat.add :=
  λ n : Nat,
  λ m : Nat,
    Nat.rec
      (λ _ : Nat, Nat)
      n
      (λ _ : Nat, λ ih : Nat, Nat.succ ih)
      m
```

Then:

```text
Nat.add n Nat.zero
  ↦ n
```

.

## 15.7 `add_zero`

```text
add_zero :
  Π n : Nat, Eq Nat (Nat.add n Nat.zero) n

add_zero :=
  λ n : Nat,
    Eq.refl Nat n
```

This proof passes because:

```text
Nat.add n Nat.zero
```

reduces by beta / delta / iota reduction to:

```text
n
```

.

That is, the kernel checks:

```text
Eq.refl Nat n :
  Eq Nat n n
```

And the expected type:

```text
Eq Nat (Nat.add n Nat.zero) n
```

is judged by conversion to be the same as:

```text
Eq Nat n n
```

.

This is a very good integration test for Phase 1.

---

# 16. Implementation Order

Recommended implementation order:

```text
1. Level implementation
   0, succ, max, imax, param

2. Expr arena implementation
   Sort, BVar, Const, App, Lam, Pi, Let

3. Context / Environment implementation
   local assumptions, local definitions, global declarations

4. Substitution / lifting implementation
   de Bruijn index shift, subst

5. infer / check implementation
   Sort, BVar, Const, Pi, Lam, App, Let

6. beta / zeta reduction implementation
   Lambda application, Let unfolding

7. delta reduction implementation
   unfolding reducible Def

8. Add Nat
   Nat, zero, succ, rec

9. iota reduction implementation
   Nat.rec on zero/succ

10. Add Eq
    Eq, refl, Eq.rec

In the current implementation, the generic recursor generator for indexed
inductives still cannot generate and check the type of `Eq.rec`, so `Eq.rec` is
treated as a kernel builtin axiom interface registered by `Env::with_builtins()`.
The reason is to let the small kernel directly check proofs generated by Phase 4
M4 `rw` / `simp-lite`. The alternative is to generalize the indexed recursor
generator first, but that would significantly expand the responsibility of the
Phase 1 inductive checker, so M4 does not adopt it. The trust boundary is only
the type interface of `Eq.rec`; tactics / elaborators / AI remain untrusted.
When the indexed recursor generator is added in the future, this builtin axiom
will be replaced by a generated recursor.

11. simple inductive checker implementation
    make Nat/Eq pass as generic declarations

12. Check through add_zero
```

---

# 17. Phase 1 Completion Criteria

Phase 1 can be called complete when:

```text
- Sorts can be type-checked
- Pi formation rules work
- Lambda type inference works
- App type inference works
- Let can be type-checked and zeta-reduced
- Const can be retrieved from the environment
- reducible definitions can be delta-reduced
- Nat can be declared
- Nat.rec iota reduction works
- Eq and Eq.refl can be used
- simple inductive declarations can be checked
- type equality can be judged by beta / delta / iota / zeta conversion
- `id`, `const`, `Nat.add`, and `add_zero` can be checked as core declarations without source/tactics
```

At this point, tactics, AI, and parsers are still unnecessary. The shortest path
is to first reach the point where **core declarations can be fed directly to the
kernel and checked**.
