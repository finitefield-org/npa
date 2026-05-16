> Phase 0 の実装基準となる仕様書は [`core-spec-v0.1.md`](core-spec-v0.1.md) です。
> このファイルは、仕様を作るための設計メモとして残します。

この5項目は、実装前に作るべき **NPA Core Spec v0.1** の中身です。
ここでいう「紙に書く」とは、実装メモではなく、kernel・certificate checker・elaborator が共有する**数学的仕様**として固定する、という意味です。

全体の依存関係はこうです。

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

まず、kernel が理解する最小言語を決めます。

表層構文には notation、implicit arguments、typeclass、tactic、match、自然言語などがありますが、**core calculus には入れません**。

core calculus は完全に明示的な項だけを扱います。

## 1.1 判断形式

仕様書では、少なくとも次の判断を定義します。

```text
Σ ; Γ ⊢ t : T
```

意味：

```text
グローバル環境 Σ とローカル文脈 Γ の下で、
項 t は型 T を持つ。
```

他にも必要です。

```text
Σ ; Γ ⊢ t ≡ u : T
```

意味：

```text
t と u は型 T において definitional equality で等しい。
```

```text
Σ ⊢ decl ok
```

意味：

```text
グローバル宣言 decl は正しい。
```

```text
Σ ⊢ module ok
```

意味：

```text
モジュール全体が正しい。
```

ここで：

```text
Σ = global environment
Γ = local context
t, u = terms
T = type
```

## 1.2 Universe level

Lean風に、`Prop` を `Sort 0`、`Type u` を `Sort (succ u)` として扱います。

```text
Level ℓ ::=
  0
| succ ℓ
| max ℓ₁ ℓ₂
| imax ℓ₁ ℓ₂
| param α
```

`imax` は依存関数型のsort計算に使います。

直感的には：

```text
Sort 0          = Prop
Sort (succ 0)   = Type 0
Sort (succ 1)   = Type 1
...
```

## 1.3 Core term

v0.1 の core term はこれで十分です。

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

意図的に入れないもの：

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

`Nat`, `Eq`, `List`, constructor, recursor などは core term の特殊構文にせず、基本的には `Const` として環境 `Σ` に登録します。

たとえば：

```text
Nat
Nat.zero
Nat.succ
Nat.rec
Eq
Eq.refl
```

はすべて global constants です。

---

# 2. Typing rules

次に、core term の型付け規則を定義します。

## 2.1 Context

ローカル文脈は次の形です。

```text
Γ ::= · | Γ, x : A | Γ, x : A := v
```

ただし certificate 内部では名前 `x` は本質的ではなく、de Bruijn index を使います。

## 2.2 Sort rule

```text
Σ ; Γ ⊢ Sort ℓ : Sort (succ ℓ)
```

つまり：

```text
Prop : Type 0
Type 0 : Type 1
Type 1 : Type 2
...
```

## 2.3 Variable rule

文脈にある変数は、その型を持ちます。

```text
x : A ∈ Γ
────────────────
Σ ; Γ ⊢ x : A
```

certificate では `x` ではなく `BVar i` です。

## 2.4 Constant rule

グローバル環境にある定数は、その型を持ちます。

```text
c : A ∈ Σ
────────────────
Σ ; Γ ⊢ c : A
```

universe polymorphic な定数なら、level を代入します。

例：

```text
Eq.{u} : Π A : Sort u, A → A → Prop
```

に対して：

```text
Eq.{0}
Eq.{1}
Eq.{max u v}
```

のように使います。

## 2.5 Pi formation

依存関数型の形成規則です。

```text
Σ ; Γ ⊢ A : Sort u
Σ ; Γ, x : A ⊢ B : Sort v
────────────────────────────────
Σ ; Γ ⊢ Pi x : A, B : Sort (imax u v)
```

`imax` を使う理由は、`Prop` を impredicative にしたいからです。

特に：

```text
A : Sort u
B : Prop
```

なら：

```text
Π x : A, B : Prop
```

にしたい。

つまり、命題は任意の型を量化しても命題のままです。

例：

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

型が definitional equality で等しければ、型を変えてよいです。

```text
Σ ; Γ ⊢ t : A
Σ ; Γ ⊢ A ≡ B : Sort u
──────────────────────
Σ ; Γ ⊢ t : B
```

この規則があるため、conversion checker が kernel の中心になります。

---

# 3. Conversion rules

conversion rules は、kernel が「同じ型」と見なす等しさです。

重要なのは、これは**証明された等式**ではなく、kernel が計算によって同一視する等式です。

## 3.1 β-reduction

```text
(App (Lam x : A, body) a)
  ↦ body[x := a]
```

例：

```text
(fun x => x + 1) 3
```

は

```text
3 + 1
```

に簡約されます。

## 3.2 δ-reduction

透明な定義を展開します。

```text
Const c ↦ value(c)
```

ただし、すべての定義を常に展開すると遅くなるので、透明度を持たせます。
core-spec v0.1 ではまず `reducible` と `opaque` だけに絞ります。

```text
reducible
opaque
```

推奨：

```text
def      : reducible または opaque
theorem  : opaque
abbrev   : reducible
```

theorem の証明本体を conversion で展開しない方がよいです。巨大証明が勝手に展開されると非常に遅くなるからです。
`semireducible` / `irreducible` のような細かい透明度は Phase 9 以降の拡張候補です。

## 3.3 ι-reduction

帰納型の recursor に対する計算規則です。

例：

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

これは `Nat` の帰納法・再帰の計算規則です。

## 3.4 ζ-reduction

let の展開です。

```text
Let x : A := v in body
  ↦ body[x := v]
```

## 3.5 η-reduction は v0.1 では入れない

関数外延性に近い挙動を definitional equality に入れると、conversion checker が複雑になります。

したがって v0.1 では：

```text
(fun x => f x) ≡ f
```

を kernel conversion には入れません。

必要なら、後で theorem や axiom として扱います。

## 3.6 proof irrelevance も conversion には入れない

`Prop` の証明をすべて definitional equality で同一視すると便利ですが、kernel が複雑になります。

v0.1 では：

```text
h₁ : P
h₂ : P
```

だからといって、

```text
h₁ ≡ h₂
```

とはしません。

proof irrelevance が欲しい場合は、標準ライブラリの theorem または optional axiom として導入します。

---

# 4. Universe rules

Universe rules は、型の型の階層を破綻させないための規則です。

## 4.1 Sort hierarchy

```text
Sort 0 : Sort 1
Sort 1 : Sort 2
Sort 2 : Sort 3
...
```

読み替えると：

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
...
```

## 4.2 Universe level の制約

universe level には制約を持たせます。

```text
Constraint ::=
  ℓ₁ ≤ ℓ₂
| ℓ₁ = ℓ₂
```

たとえば polymorphic な定義では：

```text
id.{u} : Π A : Sort u, A → A
```

のように `u` が universe parameter です。

## 4.3 Pi の universe

先ほどの規則をもう一度書くと：

```text
A : Sort u
B : Sort v
────────────────────────
Π x : A, B : Sort (imax u v)
```

`imax` の定義は：

```text
imax u 0        = 0
imax u (succ v) = max u (succ v)
```

これにより：

```text
Π x : Nat, Prop
```

は `Prop` になります。

一方：

```text
Π x : Nat, Type 0
```

は `Type 0` 以上になります。

## 4.4 Universe polymorphic constants

定数宣言には universe parameter を持たせます。

例：

```text
Eq.{u} :
  Π A : Sort u, A → A → Prop
```

certificate では：

```json
{
  "name": "Eq",
  "universe_params": ["u"],
  "type": "Π A : Sort u, A → A → Prop"
}
```

使用時には具体的な level を渡します。

```text
Const Eq [u]
```

## 4.5 Universe consistency check

kernel は declaration ごとに以下を確認します。

```text
- universe parameter が宣言されている
- level expression が well-formed
- universe constraints が充足可能
- Sort の階層に循環がない
```

禁止したいもの：

```text
Type u : Type u
```

これを許すと Girard's paradox 系の不整合につながるため、必ず：

```text
Type u : Type (u+1)
```

のような階層を保ちます。

---

# 5. Inductive rules

帰納型は証明支援系の核です。

`Nat`, `List`, `Eq`, `False`, `And`, `Or`, `Exists` などは帰納型として定義します。

## 5.1 Inductive declaration の形

基本形は：

```text
inductive I.{u} (params : P) : indices → Sort s where
| c₁ : C₁
| c₂ : C₂
...
| cₙ : Cₙ
```

v0.1 では、まず単純な帰納型から始めます。

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

constructor の型は、最終的にその帰納型を返さなければいけません。

例：

```text
Nat.zero : Nat
Nat.succ : Nat → Nat
```

これはOKです。

一方、次のようなものはおかしいです。

```text
bad : Nat → Bool
```

`Nat` の constructor なのに `Nat` を返していないからです。

## 5.3 Strict positivity

帰納型が自分自身を使う場合、**strictly positive** な位置にしか現れてはいけません。

OK：

```text
inductive List A where
| nil  : List A
| cons : A → List A → List A
```

`List A` は constructor argument として正の位置にあります。

NG：

```text
inductive Bad where
| bad : (Bad → Nat) → Bad
```

`Bad` が関数の引数側、つまり負の位置に出ています。

これを許すと論理が壊れる可能性があります。

v0.1 では positivity checker を保守的にします。

```text
許す:
  I
  A → I
  I → I ではなく、constructor の引数としての I
  List I などの nested inductive は最初は非対応

禁止:
  (I → A) → I
  negative occurrence
  nested inductive
  mutual inductive
```

nested / mutual inductive は後で追加します。

## 5.4 Recursor generation

帰納型ごとに recursor を生成します。

`Nat` の recursor は概念的に：

```text
Nat.rec :
  Π motive : Nat → Sort u,
    motive Nat.zero →
    (Π n : Nat, motive n → motive (Nat.succ n)) →
    Π n : Nat, motive n
```

計算規則：

```text
Nat.rec motive z s Nat.zero
  ↦ z
```

```text
Nat.rec motive z s (Nat.succ n)
  ↦ s n (Nat.rec motive z s n)
```

`List` の recursor は概念的に：

```text
List.rec :
  Π motive : List A → Sort u,
    motive List.nil →
    (Π x : A, Π xs : List A,
       motive xs → motive (List.cons x xs)) →
    Π xs : List A, motive xs
```

## 5.5 Prop elimination restriction

`Prop` に属する帰納型から、任意の `Type` へ elimination できるようにすると危険または複雑です。

v0.1 では安全側に倒します。

```text
I : Prop の場合、
I.rec の motive は Prop に限る。
```

つまり：

```text
False.elim : False → P
```

で `P : Prop` はOK。

しかし：

```text
False → Nat
```

のような elimination は v0.1 では制限します。

後で、Leanのように singleton elimination などを追加できますが、最初は単純にします。

## 5.6 Inductive declaration check

kernel は inductive declaration に対して以下を検査します。

```text
- parameters が well-typed
- indices が well-typed
- result sort が well-formed
- constructor types が well-typed
- constructor の戻り値が対象帰納型
- recursive occurrence が strictly positive
- universe constraints が consistent
- recursor type が正しく生成できる
- recursor computation rules が妥当
```

---

# 6. Certificate format

最後に、kernel / checker が受け取る証明証明書の形式を決めます。

これはかなり重要です。

source code は信用しません。
tactic も信用しません。
AI も信用しません。
elaborator も完全には信用しません。

checker が読むのは source ではなく canonical certificate artifact です。
kernel が読むのは checker / loader が decode した canonical core AST / declaration です。
ファイル I/O や import store からの読み込みは kernel の責務ではありません。

## 6.1 Certificate の目的

certificate は次を保証するためのものです。

```text
- どの命題が証明されたか
- どのproof termで証明されたか
- どの定義・定理に依存したか
- どのaxiomを使ったか
- import元はどのhashか
- kernel/checkerで再検査できるか
```

## 6.2 Certificate の大枠

人間向けには JSON で表せますが、実際の保存形式は canonical binary にします。
JSON は説明用・デバッグ用であり、hash-stable artifact ではありません。

概念的には：

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

## 6.3 Declaration の種類

certificate に入る宣言は主に4種類です。

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

axiom は原則禁止または allowlist 制にします。

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

kernel は：

```text
value : type
```

を検査します。

### TheoremDecl

theorem は、基本的には opaque な definition です。

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

kernel は：

```text
proof : type
```

を検査します。

しかし、conversion では proof を展開しません。

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

kernel は inductive rules に従って検査し、constructor と recursor を環境に追加します。

## 6.4 Term encoding

certificate 内の term は、source text ではなく、canonical AST として保存します。

悪い形式：

```text
"∀ n : Nat, n + 0 = n"
```

良い形式：

```text
Pi n : Nat,
  Eq Nat
    (Nat.add n Nat.zero)
    n
```

さらに実際には、名前も文字列ではなく intern table を使います。

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

実装上は JSON ではなく binary encoding にします。

## 6.5 Canonicalization

同じ canonical payload は同じ hash にならなければいけません。
source map などの非信頼 metadata は hash 対象に含めません。

そのために：

```text
- de Bruijn index を使う
- 名前の順序を固定する
- import順を固定する
- whitespaceを含めない
- notationを含めない
- implicit argumentsを含めない
- unresolved metavariableを許さない
- universe constraintsを正規化する
- declaration順を依存関係順にする
```

## 6.6 Import hash

import は名前だけでは不十分です。

```json
{
  "module": "Std.Nat.Basic",
  "export_hash": "sha256:abc...",
  "certificate_hash": "sha256:def..."
}
```

のように、依存先のhashを固定します。
`export_hash` は必須、`certificate_hash` は高信頼モードで必須にします。

これにより：

```text
同じ名前のモジュールだが中身が違う
```

という問題を防げます。

## 6.7 Axiom report

certificate には使用axiomを必ず記録します。

```json
{
  "axioms_used": [
    "Classical.choice",
    "Propext"
  ]
}
```

高信頼モードでは：

```text
- no custom axiom
- no sorry
- allowed axioms only
```

を検査します。

## 6.8 Source map は非信頼情報

source code 位置情報は、canonical certificate trusted payload には入れません。
必要なら certificate とは別の debug sidecar / audit envelope に入れてよいです。

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

ただし、source map は kernel 検査、certificate hash、export hash には使いません。

これは IDE やエラー表示用です。

---

# 7. v0.1 仕様書の目次案

実際に書くなら、次のような文書構成にします。

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

# 8. 最小例

仕様の最小例として、まずこの定理を通せるようにします。

```text
id.{u} : Π A : Sort u, A → A
id := λ A : Sort u, λ x : A, x
```

core では：

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

kernel が確認すること：

```text
λ A, λ x, x
:
Π A : Sort u, A → A
```

これが通れば、最低限の：

```text
Sort
Pi
Lambda
Application
Variable
Universe polymorphism
```

が動いています。

次に：

```text
Nat
Nat.zero
Nat.succ
Nat.rec
```

を inductive rules で導入し、

```text
theorem Nat.zero_eq_zero : Eq Nat Nat.zero Nat.zero
```

を

```text
Eq.refl Nat Nat.zero
```

で証明できるようにします。

---

# 9. 実装に入る前の合格条件

この5項目の仕様が書けたら、実装前に以下を確認します。

```text
- kernel が読む構文が明確
- typing judgment が定義されている
- conversion がどこまで等しいと見なすか明確
- universe inconsistency を防げる
- inductive type の positivity 条件が明確
- recursor の型と計算規則が明確
- certificate が canonical にhash化できる
- source syntaxなしでcertificateだけ検査できる
- theorem, def, axiom, inductive の違いが明確
- opaque theorem を展開しない設計になっている
```

この段階で最も大事なのは、**便利な機能を入れすぎないこと**です。

v0.1 は小さくします。

```text
入れる:
  Sort, Pi, Lambda, App, Let, Const, Nat, Eq, simple inductive

入れない:
  quotient, mutual inductive, nested inductive, coinductive,
  typeclass, η-conversion, proof irrelevance conversion,
  general recursion, macros, tactic language
```

まず小さいkernelを作り、source/tactic なしの core declaration として `id`, `Nat`, `Eq`, `Nat.rec`, `add_zero` が検査できる状態を目指すのがよいです。
