このリストは、**Phase 1 の小さい kernel が最低限扱えるべきもの**です。
分類するとこうです。

```text
Core term:
  Sort, Pi, Lambda, App, Let, Const

初期ライブラリ / 帰納型:
  Nat, Eq, simple inductive

計算規則:
  β, δ, ι, ζ reduction
```

つまり Phase 1 の目標は、tactic や parser や AI ではなく、**完全に明示化された証明項 certificate を受け取り、それが正しいか検査できる kernel** を作ることです。

---

# 1. Phase 1 の最小ゴール

Phase 1 で通せるようにしたいものは、たとえばこれです。

```text
id : Π A : Sort u, A → A
id := λ A, λ x, x
```

次に：

```text
Nat : Type
Nat.zero : Nat
Nat.succ : Nat → Nat
```

そして：

```text
Eq : Π A : Sort u, A → A → Prop
Eq.refl : Π A : Sort u, Π x : A, Eq A x x
```

最終的には、たとえば次のような定理を certificate として検査できる状態を目指します。

```text
theorem add_zero :
  Π n : Nat, Eq Nat (Nat.add n Nat.zero) n
```

ただし、ここでは `Nat.add` を第2引数で再帰するように定義しておくと、`Nat.add n Nat.zero` が計算だけで `n` に簡約されるので、証明は本質的に `Eq.refl` で済みます。

---

# 2. Core term の構文

Phase 1 の core term はこれで十分です。

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

表層構文の便利機能は入れません。

入れないもの：

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

kernel は、完全に明示化された core term だけを検査します。

Rust風に書くなら、直接木構造にするより、arena + ID 方式がよいです。

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

`NameId` はデバッグ用です。実際の束縛構造は de Bruijn index で扱います。

---

# 3. Sort

`Sort` は型の階層です。

```text
Sort 0        = Prop
Sort 1        = Type 0
Sort 2        = Type 1
...
```

基本規則は：

```text
Sort u : Sort (succ u)
```

つまり：

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
```

Phase 1 では universe level を単純化して、最初はこうしてもよいです。

```text
Level ::= 0 | succ Level | max Level Level | param Name
```

Lean風に `imax` まで入れるなら、`Π x : A, B` の sort 計算で便利です。最初は `max` だけでも動きますが、`∀ x : A, P : Prop` を自然に `Prop` にしたいなら `imax` がある方がよいです。

---

# 4. Pi

`Pi` は依存関数型です。

```text
Pi x : A, B
```

表層的には：

```text
∀ x : A, B
```

または：

```text
(x : A) → B
```

に対応します。

型付け規則は：

```text
Γ ⊢ A : Sort u
Γ, x : A ⊢ B : Sort v
────────────────────────────
Γ ⊢ Pi x : A, B : Sort (imax u v)
```

Phase 1 では、依存しない関数型も `Pi` で表します。

```text
A → B
```

は内部的には：

```text
Pi _ : A, B
```

です。

---

# 5. Lambda

`Lambda` は関数抽象です。

```text
Lam x : A, body
```

型付け規則：

```text
Γ ⊢ A : Sort u
Γ, x : A ⊢ body : B
────────────────────────────────
Γ ⊢ Lam x : A, body : Pi x : A, B
```

例：

```text
λ A : Sort u, λ x : A, x
```

の型は：

```text
Π A : Sort u, A → A
```

です。

この時点で `id` が検査できるようになります。

---

# 6. App

`App` は関数適用です。

```text
App f a
```

型付け規則：

```text
Γ ⊢ f : Pi x : A, B
Γ ⊢ a : A
────────────────────
Γ ⊢ App f a : B[x := a]
```

実装上は、`f` の型をそのまま見るだけでは不十分です。
まず weak-head normal form にします。

```text
infer(f) = T
whnf(T) = Pi x : A, B
```

のように、`T` を `Pi` まで簡約してから引数 `a` を検査します。

疑似コード：

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

`Let` は局所定義です。

```text
Let x : A := v in body
```

型付け規則：

```text
Γ ⊢ A : Sort u
Γ ⊢ v : A
Γ, x : A := v ⊢ body : B
────────────────────────────
Γ ⊢ Let x : A := v in body : B[x := v]
```

`Let` は ζ-reduction で展開されます。

```text
let x := v in body
  ↦ body[x := v]
```

実装上は、context に local definition として入れてもよいです。

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

`Const` は global environment に登録された定数です。

```text
Const Nat []
Const Nat.zero []
Const Eq [u]
Const Nat.rec [u]
```

`Const` は環境 `Env` から型を引きます。

```text
Σ(c) = declaration of c
────────────────────────
Γ ⊢ Const c levels : instantiate_universes(type(c), levels)
```

Phase 1 の declaration は、最低限これでよいです。

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

`Def` は δ-reduction で展開可能です。
`Theorem` は基本的に opaque にして、conversion では展開しない方がよいです。

---

# 9. Nat

`Nat` は最初の帰納型です。

```text
inductive Nat : Type where
| zero : Nat
| succ : Nat → Nat
```

core environment には次が入ります。

```text
Nat      : Sort 1
Nat.zero : Nat
Nat.succ : Nat → Nat
Nat.rec  : ...
```

`Nat.rec` の型は概念的に：

```text
Nat.rec :
  Π motive : Nat → Sort u,
    motive Nat.zero →
    (Π n : Nat, motive n → motive (Nat.succ n)) →
    Π n : Nat, motive n
```

計算規則は：

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

この `Nat.rec` の計算が ι-reduction です。

---

# 10. Eq

`Eq` は等号です。

```text
inductive Eq.{u} (A : Sort u) (a : A) : A → Prop where
| refl : Eq A a a
```

core environment には次が入ります。

```text
Eq      : Π A : Sort u, A → A → Prop
Eq.refl : Π A : Sort u, Π x : A, Eq A x x
```

`Eq` は「証明された等式」を表します。

注意点として、`Eq` と definitional equality は別物です。

```text
definitional equality:
  kernel が計算で同じと見なす等しさ

Eq:
  論理上の命題としての等しさ
```

たとえば：

```text
Nat.add n Nat.zero
```

が reduction により `n` に簡約されるなら、

```text
Eq Nat (Nat.add n Nat.zero) n
```

は `Eq Nat n n` と definitional equality で同じ型になり、`Eq.refl Nat n` で証明できます。

---

# 11. simple inductive

Phase 1 の `simple inductive` は、最初から全部の帰納型機能を実装しない、という意味です。

対応するもの：

```text
- 単一の帰納型
- parameter あり
- index は Eq のために最小限対応
- constructor は有限個
- strict positivity を満たす
- recursor を生成できる
```

まだ対応しないもの：

```text
- mutual inductive
- nested inductive
- coinductive
- quotient
- higher inductive
- complex universe polymorphism
- large elimination の細かい例外
```

帰納型宣言の内部表現は、たとえば：

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

`Nat` は index なし。

```text
Nat : Sort 1
zero : Nat
succ : Nat → Nat
```

`Eq` は index あり。

```text
Eq.{u} (A : Sort u) (a : A) : A → Prop
refl : Eq A a a
```

Phase 1 での実装方針としては、次の順序が現実的です。

```text
1. Nat と Eq を kernel 内で特別扱いして動かす
2. その後、generic simple inductive checker を実装する
3. Nat と Eq も generic inductive declaration として通す
4. 特別扱いを削る
```

最終的には、`Nat` と `Eq` は特殊構文ではなく、`InductiveDecl` から生成される `Const` 群として扱うのが望ましいです。

---

# 12. βδιζ reduction

Phase 1 の conversion checker は、以下の4種類の reduction を扱います。

```text
β: beta
δ: delta
ι: iota
ζ: zeta
```

## 12.1 β-reduction

関数適用です。

```text
(λ x : A, body) a
  ↦ body[x := a]
```

例：

```text
(λ x : Nat, x) Nat.zero
  ↦ Nat.zero
```

## 12.2 δ-reduction

定義展開です。

```text
Const c ↦ value(c)
```

ただし、`c` が transparent な定義のときだけ展開します。

```text
def Nat.add := ...
```

なら展開してよい。

```text
theorem Nat.add_zero := ...
```

は基本的に展開しない。

Phase 1 の reducibility は単純にこれでよいです。

```rust
enum Reducibility {
    Reducible,
    Opaque,
}
```

後で：

```text
reducible
semireducible
irreducible
opaque
```

に分ければよいです。

## 12.3 ι-reduction

recursor が constructor に適用されたときの計算です。

`Nat.rec` の例：

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

このルールがないと、自然数上の再帰関数が計算できません。

たとえば：

```text
Nat.add n Nat.zero
```

を `Nat.rec` で定義している場合、ι-reduction によって `n` に簡約できるようになります。

## 12.4 ζ-reduction

let 展開です。

```text
let x := v in body
  ↦ body[x := v]
```

---

# 13. WHNF と conversion checker

kernel では、全項を完全正規化するより、まず weak-head normal form, WHNF を使います。

```text
whnf(t)
```

は、項の先頭だけを必要なだけ簡約します。

例：

```text
whnf((λ x, body) a) = whnf(body[x := a])
```

```text
whnf(let x := v in body) = whnf(body[x := v])
```

```text
whnf(Const c) = whnf(value(c))  // c が reducible の場合
```

```text
whnf(Nat.rec motive z s Nat.zero) = whnf(z)
```

conversion checker は大まかにこうです。

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

Phase 1 では、次は入れなくてよいです。

```text
- η-conversion
- proof irrelevance conversion
- quotient computation
- aggressive theorem unfolding
```

これらを早く入れると、conversion checker が複雑になりすぎます。

---

# 14. Phase 1 の型検査アルゴリズム

最小の型検査 API はこれです。

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

各 term の処理：

```text
Sort:
  infer(Sort u) = Sort (succ u)

BVar:
  context から型を引く

Const:
  environment から型を引き、universe level を代入

Pi:
  domain と codomain が Sort か確認し、Sort を返す

Lam:
  body の型を推論し、Pi 型を返す

App:
  関数の型を WHNF して Pi か確認し、引数を check

Let:
  value を check し、body を local definition 付き context で推論
```

---

# 15. Phase 1 で作るべきテスト

## 15.1 `id`

```text
id.{u} :
  Π A : Sort u, A → A

id :=
  λ A : Sort u,
  λ x : A,
    x
```

これで以下を検証できます。

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

これで複数 binder と de Bruijn index を検証できます。

## 15.3 `Nat`

```text
Nat : Sort 1
Nat.zero : Nat
Nat.succ : Nat → Nat
```

これで帰納型 declaration を検証します。

## 15.4 `Nat.rec`

```text
Nat.rec motive z s Nat.zero
  ≡ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ≡ s n (Nat.rec motive z s n)
```

これで ι-reduction を検証します。

## 15.5 `Eq.refl`

```text
Eq.refl Nat Nat.zero :
  Eq Nat Nat.zero Nat.zero
```

これで Eq の基本を検証します。

## 15.6 `Nat.add`

第2引数で再帰する定義にします。

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

すると：

```text
Nat.add n Nat.zero
  ↦ n
```

になります。

## 15.7 `add_zero`

```text
add_zero :
  Π n : Nat, Eq Nat (Nat.add n Nat.zero) n

add_zero :=
  λ n : Nat,
    Eq.refl Nat n
```

この証明が通る理由は：

```text
Nat.add n Nat.zero
```

が βδι-reduction により：

```text
n
```

へ簡約されるからです。

つまり、kernel は次を確認します。

```text
Eq.refl Nat n :
  Eq Nat n n
```

そして期待型：

```text
Eq Nat (Nat.add n Nat.zero) n
```

が conversion により：

```text
Eq Nat n n
```

と同じだと判定します。

これは Phase 1 の非常に良い総合テストです。

---

# 16. 実装順序

おすすめの実装順はこれです。

```text
1. Level 実装
   0, succ, max, param

2. Expr arena 実装
   Sort, BVar, Const, App, Lam, Pi, Let

3. Context / Environment 実装
   local assumptions, local definitions, global declarations

4. Substitution / lifting 実装
   de Bruijn index の shift, subst

5. infer / check 実装
   Sort, BVar, Const, Pi, Lam, App, Let

6. βζ reduction 実装
   Lambda application, Let unfolding

7. δ reduction 実装
   transparent Def の展開

8. Nat を追加
   Nat, zero, succ, rec

9. ι reduction 実装
   Nat.rec on zero/succ

10. Eq を追加
    Eq, refl, possibly Eq.rec

11. simple inductive checker 実装
    Nat/Eq を generic declaration として通す

12. add_zero まで検査
```

---

# 17. Phase 1 の完了条件

Phase 1 が完了したと言える条件はこれです。

```text
- Sort の型検査ができる
- Pi の形成規則が動く
- Lambda の型推論ができる
- App の型推論ができる
- Let が型検査でき、ζ-reduction できる
- Const を environment から引ける
- transparent definition を δ-reduction できる
- Nat が宣言できる
- Nat.rec の ι-reduction ができる
- Eq と Eq.refl が使える
- simple inductive declaration を検査できる
- βδιζ conversion により型の一致判定ができる
- `id`, `const`, `Nat.add`, `add_zero` が certificate だけで検査できる
```

この時点では、まだ tactic も AI も parser も不要です。
まずは **certificate を直接食わせて kernel が検査できる** ところまで作るのが最短です。

