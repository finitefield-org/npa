以下は **Phase 6: library** の詳細設計です。
Phase 6 の目的は、巨大な Mathlib 的ライブラリをいきなり作ることではなく、以後の証明探索・tactic・定理検索・AI補助が依存できる **小さく堅い標準ライブラリ** を作ることです。

対象モジュールはこの4つです。

```text
Std.Logic
Std.Nat
Std.List
Std.Algebra.Basic
```

基本方針は次です。

```text
1. まず kernel で検査しやすい定義だけを入れる
2. 証明は certificate 化できるものだけ採用する
3. no sorry / no custom axiom を標準にする
4. simp-lite / rw / apply / theorem search 用の属性を最初から付ける
5. 定理名・rewrite方向・依存関係・axiom report をライブラリ設計に含める
```

---

# 1. Phase 6 の全体像

Phase 6 で作るものは、単なるソースファイルではありません。

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
IDE / tactic / AI search から利用
```

各モジュールは次を生成します。

```text
Std/Logic.npa
Std/Logic.npcert
Std/Logic.index.json
Std/Logic.axioms.json   -- derived axiom report view
Std/Logic.graph.json    -- derived dependency graph
```

Phase 6 の完了条件は：

```text
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic が kernel で検査済み
- 各モジュールが certificate 化されている
- import entries が export_hash を持ち、高信頼モード用の certificate_hash も生成されている
- module の export_hash / certificate_hash / axiom_report_hash が生成されている
- axiom report が空、または allowlist 内
- theorem search index が生成されている
- simp-lite が基本定理を使える
- Phase 4 の tactic で標準的な定理が証明できる
```

です。

---

# 2. モジュール依存関係

依存関係は小さく保ちます。

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
  └── depends on Core

Std.Nat
  └── depends on Std.Logic

Std.List
  ├── depends on Std.Logic
  └── depends on Std.Nat

Std.Algebra.Basic
  └── depends on Std.Logic
```

図にすると：

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

`Std.Algebra.Basic` は `Std.Nat` に依存させない方がよいです。
代数構造は自然数に限らないためです。

---

# 3. ライブラリ全体の命名規則

今後の theorem search と AI 検索のため、命名規則を厳密にします。

## 3.1 基本パターン

```text
Namespace.object_property
Namespace.operation_property
Namespace.theorem_name
```

例：

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

## 3.2 rewrite theorem の命名

```text
xxx_zero
zero_xxx
xxx_assoc
xxx_comm
xxx_left_id
xxx_right_id
```

例：

```text
Nat.add_zero     : n + 0 = n
Nat.zero_add     : 0 + n = n
Nat.mul_zero     : n * 0 = 0
Nat.zero_mul     : 0 * n = 0
List.append_nil  : xs ++ [] = xs
List.nil_append  : [] ++ xs = xs
```

## 3.3 補助定理の命名

内部補助定理は `_aux` を使ってもよいですが、公開しない方針にします。

```text
Nat.add_comm_aux
```

公開 API に出すなら、意味のある名前にします。

---

# 4. 属性設計

Phase 6 では、定理に属性を付けます。

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

Phase 6 MVP では、最低限これを実装します。

```text
@[simp]   simp-lite が使う
@[rw]     rw search が使う
@[intro]  intro/constructor 系検索が使う
@[elim]   apply/cases 系検索が使う
```

属性は certificate の trusted 部分ではありません。
ただし、theorem index と tactic search には使います。

属性 metadata 例：

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

`Std.Logic` は最小論理ライブラリです。

目的：

```text
- Eq を中心とする等式推論
- True / False / And / Or / Iff / Not / Exists
- rw / apply / exact / theorem search の土台
- 古典公理なしの構成的論理
```

## 5.1 含める定義

### Eq

`Eq` は Phase 1 の core 側で simple inductive として持っていてもよいです。
`Std.Logic` ではそれを公開し、基本定理を追加します。

```npa
inductive Eq.{u} {A : Sort u} (x : A) : A -> Prop where
| refl : Eq x x
```

表層では：

```npa
x = y
```

を：

```text
Eq x y
```

に展開します。

基本定理：

```npa
theorem Eq.refl {A : Sort u} (x : A) : x = x := ...
theorem Eq.symm {A : Sort u} {x y : A} : x = y -> y = x := ...
theorem Eq.trans {A : Sort u} {x y z : A} : x = y -> y = z -> x = z := ...
theorem Eq.subst {A : Sort u} {x y : A} (P : A -> Prop) :
  x = y -> P x -> P y := ...
theorem Eq.congrArg {A : Sort u} {B : Sort v} (f : A -> B) :
  x = y -> f x = f y := ...
```

属性：

```text
Eq.refl      @[refl]
Eq.symm      usable by rw reverse
Eq.trans     @[trans]
Eq.congrArg  theorem search 用
Eq.subst     rw の proof term 生成用
```

## 5.2 True / False

```npa
inductive True : Prop where
| intro : True

inductive False : Prop where
```

基本定理：

```npa
theorem False.elim {P : Prop} : False -> P := ...
```

属性：

```text
True.intro   @[intro]
False.elim   @[elim]
```

Phase 1 の Prop elimination restriction に合わせ、`False.elim` はまず `P : Prop` に限定します。
`False -> Nat` のような large elimination は後回しにします。

## 5.3 Not

`Not` は定義でよいです。

```npa
def Not (P : Prop) : Prop := P -> False
```

notation：

```text
¬ P
```

基本定理：

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

基本定理：

```npa
theorem And.left  {P Q : Prop} : P ∧ Q -> P := ...
theorem And.right {P Q : Prop} : P ∧ Q -> Q := ...
theorem And.intro {P Q : Prop} : P -> Q -> P ∧ Q := ...
```

属性：

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

基本定理：

```npa
theorem Or.elim {P Q R : Prop} : P ∨ Q -> (P -> R) -> (Q -> R) -> R := ...
theorem Or.inl {P Q : Prop} : P -> P ∨ Q := ...
theorem Or.inr {P Q : Prop} : Q -> P ∨ Q := ...
```

## 5.6 Iff

`Iff` は定義または inductive で作れます。

MVP では構造体的 inductive にします。

```npa
inductive Iff (P Q : Prop) : Prop where
| intro : (P -> Q) -> (Q -> P) -> Iff P Q
```

notation：

```text
P ↔ Q
```

基本定理：

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

基本定理：

```npa
theorem Exists.intro {A : Sort u} {P : A -> Prop} :
  (x : A) -> P x -> Exists P := ...

theorem Exists.elim {A : Sort u} {P : A -> Prop} {Q : Prop} :
  Exists P -> ((x : A) -> P x -> Q) -> Q := ...
```

## 5.8 Std.Logic の theorem search metadata

例：

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

`apply Eq.trans` が使えるよう、`Eq.trans` は apply search で高スコアにします。

---

# 6. Std.Nat

`Std.Nat` は自然数ライブラリです。

目的：

```text
- Nat の基本定義
- 加法・乗法
- 基本等式
- induction tactic の対象
- simp-lite の主要 rewrite source
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

Phase 6 MVP では overloaded numerals はまだ不要です。
`Nat` 専用 numeral として扱えば十分です。

## 6.2 add

以前の設計と整合させるため、`Nat.add n m` は **第2引数 m で再帰** するのがよいです。

```npa
def Nat.add (n m : Nat) : Nat :=
  Nat.rec
    (fun _ => Nat)
    n
    (fun _ ih => Nat.succ ih)
    m
```

これにより、次が definitional equality になります。

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

## 6.3 add の基本定理

まず refl で証明できるもの：

```npa
@[simp]
theorem Nat.add_zero (n : Nat) : n + 0 = n := Eq.refl n

@[simp]
theorem Nat.add_succ (n m : Nat) :
  n + Nat.succ m = Nat.succ (n + m) := Eq.refl _
```

induction が必要なもの：

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

注意点：

```text
Nat.add_comm は simp ルールにしない。
```

理由は、可換律を simp に入れると rewrite loop が起きやすいためです。

属性方針：

```text
Nat.add_zero   @[simp]
Nat.add_succ   @[simp]
Nat.zero_add   @[simp]
Nat.succ_add   not simp initially, or low priority
Nat.add_assoc  not simp
Nat.add_comm   not simp
```

## 6.4 mul

乗法も第2引数で再帰させます。

```npa
def Nat.mul (n m : Nat) : Nat :=
  Nat.rec
    (fun _ => Nat)
    0
    (fun _ ih => ih + n)
    m
```

つまり：

```text
n * 0 ≡ 0
n * succ m ≡ n * m + n
```

notation：

```text
n * m
```

基本定理：

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

Phase 6 MVP では、`mul_assoc`, `mul_comm`, `distrib` は後半に回してよいです。
最初は `add` 系を安定させるのが優先です。

## 6.5 pred / one

便利なので `one` と `pred` を定義してもよいです。

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

基本定理：

```npa
@[simp]
theorem Nat.pred_zero : Nat.pred 0 = 0 := Eq.refl 0

@[simp]
theorem Nat.pred_succ (n : Nat) : Nat.pred (Nat.succ n) = n := Eq.refl n
```

## 6.6 Nat の simp-lite ルール

安全に入れるもの：

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

入れない、または慎重にするもの：

```text
Nat.add_comm
Nat.mul_comm
Nat.add_assoc
Nat.mul_assoc
Nat.left_distrib
Nat.right_distrib
```

これらは `simp-lite` ではなく、将来の `ring` や `normalize_nat` 系 tactic に任せる方が安全です。

## 6.7 Std.Nat の検索メタデータ例

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

`Std.List` はリストライブラリです。

目的：

```text
- polymorphic inductive のテスト
- List induction の準備
- append / length / map の基本定理
- simp-lite の実用性向上
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

Phase 6 MVP では、リストリテラル `[a, b, c]` は後回しでもよいです。

## 7.2 append

`append xs ys` は第1引数で再帰するのが自然です。

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

基本定理：

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

属性方針：

```text
List.nil_append   @[simp]
List.cons_append  @[simp]
List.append_nil   @[simp]
List.append_assoc not simp
```

`append_assoc` を simp に入れると、rewrite の方向によって項が変形し続ける可能性があるため、MVPでは入れません。

## 7.3 length

```npa
def List.length {A : Type u} (xs : List A) : Nat :=
  List.rec
    (fun _ => Nat)
    0
    (fun _ _ ih => Nat.succ ih)
    xs
```

基本定理：

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

基本定理：

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

Phase 6 MVP では `foldr` だけで十分です。

```npa
def List.foldr {A : Type u} {B : Type v}
  (f : A -> B -> B) (init : B) (xs : List A) : B :=
  List.rec
    (fun _ => B)
    init
    (fun x xs ih => f x ih)
    xs
```

基本定理：

```npa
@[simp]
theorem List.foldr_nil :
  List.foldr f init [] = init := Eq.refl init

@[simp]
theorem List.foldr_cons :
  List.foldr f init (x :: xs) = f x (List.foldr f init xs) := Eq.refl _
```

`foldl` は後回しで構いません。

## 7.6 Std.List の simp-lite ルール

安全に入れるもの：

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

入れないもの：

```text
List.append_assoc
List.map_comp
List.length_append
```

これらは有用ですが、simp-lite に入れるには慎重な方向制御が必要です。

---

# 8. Std.Algebra.Basic

`Std.Algebra.Basic` は代数構造の最小ライブラリです。

目的：

```text
- associativity / commutativity / identity の一般定義
- Nat.add や List.append を一般的な構造として扱える土台
- 将来の typeclass / ring / group / monoid tactic の準備
```

Phase 6 の時点では、まだ本格的な typeclass resolution を入れない方がよいです。
したがって、まずは **明示的構造** として設計します。

---

## 8.1 基本述語

まず、構造体にする前に、操作の性質を定義します。

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

これらは非常に検索しやすいです。

例：

```text
Associative Nat.add
Commutative Nat.add
LeftIdentity 0 Nat.add
RightIdentity 0 Nat.add
```

## 8.2 IsSemigroup / IsMonoid

Phase 6 では、bundled carrier ではなく、unbundled property として始めるのが簡単です。

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

この設計の利点：

```text
- typeclass なしで使える
- kernel 的に単純
- Prop 内の inductive として扱える
- theorem search しやすい
```

欠点：

```text
- 毎回 op と e を明示する必要がある
- Lean の typeclass 的な自動推論はまだない
```

Phase 6 ではこの欠点を受け入れます。

---

## 8.3 projection theorem

`IsMonoid` から各性質を取り出す theorem を用意します。

```npa
theorem IsMonoid.assoc {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> Associative op := ...

theorem IsMonoid.left_id {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> LeftIdentity e op := ...

theorem IsMonoid.right_id {A : Type u} {op : A -> A -> A} {e : A} :
  IsMonoid op e -> RightIdentity e op := ...
```

`IsCommMonoid` も同様です。

```npa
theorem IsCommMonoid.assoc : IsCommMonoid op e -> Associative op := ...
theorem IsCommMonoid.comm : IsCommMonoid op e -> Commutative op := ...
theorem IsCommMonoid.left_id : IsCommMonoid op e -> LeftIdentity e op := ...
theorem IsCommMonoid.right_id : IsCommMonoid op e -> RightIdentity e op := ...
```

これらは apply search で使いやすいです。

## 8.4 identity uniqueness

基本的な一般定理も入れます。

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

このような一般定理は、今後のライブラリ品質に大きく効きます。

## 8.5 Nat.add を commutative monoid として登録

`Std.Algebra.Basic` 自体は `Std.Nat` に依存させないのがよいですが、別モジュールとして：

```text
Std.Nat.Algebra
```

を後で作り、そこで登録します。

```npa
theorem Nat.add_is_comm_monoid :
  IsCommMonoid Nat.add 0 := ...
```

Phase 6 の範囲では、`Std.Algebra.Basic` は構造定義だけにして、具体例は後回しでもよいです。

ただし、Phase 6 のテスト用に `Std.Nat` 側に次を追加してもよいです。

```npa
theorem Nat.add_associative : Associative Nat.add := Nat.add_assoc
theorem Nat.add_commutative : Commutative Nat.add := Nat.add_comm
theorem Nat.add_left_identity : LeftIdentity 0 Nat.add := Nat.zero_add
theorem Nat.add_right_identity : RightIdentity 0 Nat.add := Nat.add_zero
```

---

# 9. 各モジュールの推奨ファイル構成

最初は4ファイルでもよいですが、将来を見越すならこう分けます。

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

Phase 6 MVP では、外部公開名を次のようにまとめます。

```text
Std.Logic
Std.Nat
Std.List
Std.Algebra.Basic
```

内部 import：

```text
Std.Logic
  imports Core

Std.Nat
  imports Std.Logic

Std.List
  imports Std.Logic, Std.Nat

Std.Algebra.Basic
  imports Std.Logic
```

---

# 10. certificate / hash 設計との接続

各モジュールは `.npcert` を持ちます。

例：

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
      "name": "Std.Nat.add_zero",
      "decl_interface_hash": "sha256:..."
    },
    {
      "name": "Std.Nat.zero_add",
      "decl_interface_hash": "sha256:..."
    }
  ],
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

重要なルール：

```text
- def の body は export_hash に含める
- opaque theorem の proof body は export_hash に含めなくてよい
- theorem の axiom dependencies は export_hash に含める
- import の export_hash が一致しない場合は build fail、高信頼モードでは certificate_hash も一致させる
```

---

# 11. theorem index

Phase 6 では、各モジュールから theorem search 用 index を生成します。

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
  "proof_term_size": 128,
  "suggested_tactics": [
    "simp-lite",
    "rw [List.append_nil]"
  ]
}
```

## 11.2 検索カテゴリ

各 theorem を次のカテゴリに分類します。

```text
exact:
  target と直接一致しやすい

apply:
  結論が target に一致し、前提を subgoal にする

rw:
  Eq lhs rhs として rewrite できる

simp:
  simp-lite が使える

induction:
  induction で使う recursor / induction theorem
```

例：

```text
Eq.refl              exact
Eq.trans             apply
Nat.add_zero         exact/rw/simp
Nat.add_comm         rw, but not simp
List.append_assoc    rw, but not simp
And.intro            apply/intro
False.elim           apply/elim
```

---

# 12. simp-lite データベース

Phase 6 の最重要成果の一つは、`simp-lite` が使えるようになることです。

## 12.1 初期 simp set

```text
Std.Logic:
  Eq.refl-related closure
  Iff.refl maybe

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

## 12.2 simp に入れない定理

```text
Nat.add_comm
Nat.mul_comm
Nat.add_assoc
Nat.mul_assoc
List.append_assoc
List.map_comp
List.length_append
```

理由：

```text
- loop の危険がある
- normal form の方針が必要
- simp-lite には重すぎる
```

これらは将来の専用 tactic に回します。

```text
assoc-normalizer
comm-normalizer
ring
monoid_normalize
```

---

# 13. ライブラリテスト

Phase 6 では、各定理の証明だけでなく、API/tactic/検索もテストします。

## 13.1 Kernel check test

全宣言に対して：

```text
proof : theorem_type
value : def_type
inductive declaration ok
```

を確認します。

## 13.2 Certificate check test

```text
- .npcert だけで再検査できる
- sourceなしで検査できる
- import の export_hash が一致し、高信頼モードでは certificate_hash も一致する
- declaration hash が一致する
- axiom report が再計算結果と一致する
```

## 13.3 No axiom test

Phase 6 の標準ライブラリは、原則として axiom なしにします。

```json
{
  "module": "Std.Nat",
  "axioms_used": [],
  "contains_sorry": false
}
```

## 13.4 simp-lite test

例：

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

期待検索結果：

```text
Nat.add_zero
```

goal：

```text
xs ++ [] = xs
```

期待検索結果：

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

期待候補：

```text
apply Eq.trans
```

## 13.6 tactic regression test

各 tactic が標準ライブラリ上で動くことを確認します。

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

# 14. Phase 6 の実装順序

おすすめ順はこれです。

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

# 15. Phase 6 でまだ入れないもの

MVP を小さく保つため、次は後回しにします。

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

特に `Classical.choice`, `funext`, `propext` は、入れるとしても：

```text
Std.Classical
```

のような別モジュールに分離し、axiom report に明示します。

`Std.Logic` 本体は構成的に保つのが望ましいです。

---

# 16. Phase 6 の完了条件

Phase 6 が完了したと言える条件はこれです。

```text
- Std.Logic が Eq / connectives / Exists を提供する
- Std.Nat が add / mul / 基本定理を提供する
- Std.List が append / length / map / foldr を提供する
- Std.Algebra.Basic が Associative / Commutative / Monoid 系定義を提供する
- 全モジュールが no sorry / no custom axiom
- 全モジュールが .npcert として再検査可能
- import entries が export_hash を持ち、高信頼モード用の certificate_hash も生成される
- module の export_hash / certificate_hash / axiom_report_hash が生成される
- theorem index が生成される
- simp-lite が Nat/List の基本ゴールを閉じられる
- theorem search が exact/apply/rw/simp 候補を返せる
- Phase 4 tactic だけで基本定理の再証明テストが通る
```

---

# 17. 一文でまとめると

Phase 6 は、**kernel・certificate・tactic・IDE/API の上に、最初の信頼できる標準数学ライブラリを載せる段階**です。

最初に作るべき標準ライブラリは：

```text
Std.Logic:
  等式と命題論理の土台

Std.Nat:
  自然数、加法、乗法、帰納法、基本 rewrite

Std.List:
  polymorphic data、append、length、map

Std.Algebra.Basic:
  結合律・可換律・単位元・monoid の抽象構造
```

この Phase 6 が完成すると、以降の Phase 7 で **AI証明探索、RAG、premise selection、proof search** を本格的に載せられるようになります。
