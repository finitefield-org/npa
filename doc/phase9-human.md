以下は **Phase 9 Human Profile: 高度化** の詳細設計です。
Phase 9 は、ここまで作った小さく安全な証明支援系を、実用的な数学・プログラム検証・AI証明探索に耐える水準へ拡張する段階です。

対象はこの7項目です。

```text
- advanced inductive
- quotient
- typeclass
- universe polymorphism強化
- SMT certificates
- theorem graph
- natural language formalization
```

実装メモ（2026-05-21）:

```text
- この文書は Phase 9 Human Profile の user-facing / kernel-facing 最終ターゲット設計を記述する
- 現リポジトリで実装済みなのは crates/npa-api の Phase 9 AI deterministic validation /
  replay substrate と M9 fixture matrix である
- advanced inductive / quotient / typeclass / SMT / theorem graph / natural language formalization の
  full human workflow は、この文書の後続 implementation target として扱う
- Phase 9 完了後の回帰確認は ./scripts/phase9-regression.sh と
  GitHub Actions の Phase 9 Regression で固定している
```

Phase 9 の基本方針はこれです。

```text
kernel に入れるもの:
  advanced inductive
  quotient primitive, 採用する場合
  universe polymorphism の検査

kernel に入れないもの:
  typeclass 探索
  SMT solver 本体
  theorem graph
  natural language formalizer
  AIモデル
```

Lean でも、inductive types は新しい型を導入する主要手段であり、quotient types と universe などは組み込み primitive として扱われます。また Lean の typeclass は overloaded operations を扱う柔軟な仕組みですが、kernel の証明検査そのものではなく elaboration 側の機構です。([Lean Language][1])

---

# 1. Phase 9 の全体像

Phase 9 は、単に機能を追加する段階ではありません。
ここでは **表現力・自動化・信頼性・検索性** を同時に拡張します。

全体の拡張方針はこうです。

```text
Phase 9.1  advanced inductive
  indexed / mutual / nested inductive を扱えるようにする

Phase 9.2  universe polymorphism強化
  polymorphic library を本格化する

Phase 9.3  typeclass
  algebraic hierarchy と overloaded notation を実用化する

Phase 9.4  quotient
  商集合、商群、商環、同値類を扱えるようにする

Phase 9.5  SMT certificates
  SMT solver の結果を証明証明書として再構成・検査する

Phase 9.6  theorem graph
  ライブラリ全体を検索・推薦・学習可能なグラフにする

Phase 9.7  natural language formalization
  自然言語 / LaTeX から形式命題候補を作る
```

おすすめの実装順序は、上の順に近いです。
特に **advanced inductive と universe polymorphism** は、typeclass・quotient・大規模ライブラリの土台になるため先に固めます。

---

# 2. Advanced inductive

## 2.1 目的

Phase 1〜6 の `simple inductive` では、主に次を扱いました。

```text
Nat
Eq
List
```

Phase 9 では、より実用的な帰納型を扱います。

```text
- indexed inductive families
- mutual inductive
- nested inductive
- dependent eliminator
- large elimination
- generated recursor / induction principle
- stronger positivity checker
```

Lean の inductive type は、type constructor と constructor から指定され、その他の性質はそこから導かれる設計です。NPAでも、ユーザーが recursor を自由に公理として追加するのではなく、kernel/checker が inductive declaration から recursor と computation rule を機械的に導出する方針にします。([Lean Language][1])

---

## 2.2 Indexed inductive family

代表例は `Vec` です。

```npa
inductive Vec.{u} (A : Type u) : Nat -> Type u where
| nil  : Vec A 0
| cons : {n : Nat} -> A -> Vec A n -> Vec A (Nat.succ n)
```

ここでは `Vec A n` の `n` が index です。

`List A` は単なる型でした。

```text
List A : Type u
```

一方、`Vec A n` は長さ `n` に依存する型です。

```text
Vec A 0
Vec A 1
Vec A 2
...
```

この機能があると、次のような型安全な仕様を書けます。

```text
append :
  Vec A m -> Vec A n -> Vec A (m + n)
```

Phase 9 では、`indices` を含む inductive declaration を正式に扱います。

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

---

## 2.3 Constructor result check

indexed inductive では、constructor の戻り値が単に `I params indices` の形であることを確認するだけでは足りません。

例：

```npa
Vec.nil : Vec A 0
Vec.cons : A -> Vec A n -> Vec A (succ n)
```

kernel は constructor ごとに以下を確認します。

```text
- constructor type が well-typed
- constructor の最終戻り値が対象 inductive family
- parameters が宣言された params と一致する
- indices が正しい型を持つ
- recursive occurrence が strictly positive
```

constructor の戻り値が別の型になるものは拒否します。

```npa
bad : Nat -> Bool
```

これは `Nat` の constructor にはなれません。

---

## 2.4 Mutual inductive

相互再帰的な帰納型を許します。

例：

```npa
mutual
inductive Even : Nat -> Prop where
| zero : Even 0
| succ : Odd n -> Even (Nat.succ n)

inductive Odd : Nat -> Prop where
| succ : Even n -> Odd (Nat.succ n)
end
```

内部表現では、1つの `MutualInductiveBlock` として扱います。

```rust
struct MutualInductiveBlock {
    universe_params: Vec<UniverseParam>,
    inductives: Vec<InductiveDecl>,
}
```

検査項目：

```text
- block 全体で名前が一意
- 各 inductive が well-typed
- constructor が block 内の inductive を正しく参照
- 相互再帰出現が strictly positive
- recursor 群を同時に生成できる
```

---

## 2.5 Nested inductive

nested inductive は、再帰出現が他の型構成子の中に現れるものです。

例：

```npa
inductive Rose (A : Type u) : Type u where
| node : A -> List (Rose A) -> Rose A
```

ここでは `Rose A` が `List` の中に出ています。

Phase 9 の方針は段階的にします。

```text
Step 1:
  nested inductive は禁止

Step 2:
  approved strictly-positive type constructors の中だけ許可
  例: List, Option, Prod

Step 3:
  generic positivity checker で処理
```

最初から完全一般の nested inductive を許すと positivity checker が難しくなるため、Human Profile の目標としては
`List` のような既知の strictly-positive functor の中だけ許す段階を置きます。
ただし AI Machine Profile の最初の MVP は Phase 2 の既存 certificate schema と deterministic recursor 生成に合わせ、
nested inductive をいったん全面拒否します。
approved functor 越しの nested inductive は、functoriality 証明、positivity traversal、recursor 生成、hash rule を固定した
後続 profile で有効化します。

---

## 2.6 Positivity checker 強化

Phase 1 の positivity checker は保守的でした。
Phase 9 では、次を判定できるようにします。

```text
OK:
  I
  A -> I
  I -> I              constructor argument としての正の出現
  List I
  Option I
  Prod A I

NG:
  (I -> A) -> I
  ((I -> A) -> B) -> I
  negative occurrence
  type constructor whose positivity is unknown
```

内部的には、各型構成子に polarity 情報を持たせます。

```rust
enum Polarity {
    Positive,
    Negative,
    Neutral,
}

struct PositivityInfo {
    type_constructor: GlobalRef,
    argument_polarities: Vec<Polarity>,
}
```

例：

```text
List : Type u -> Type u
argument polarity:
  [Positive]

Function type A -> B:
  A は Negative
  B は Positive
```

---

## 2.7 Large elimination

`Nat : Type` のようなデータ型からは、`Type` への elimination を許してよいです。

```text
Nat.rec :
  Π motive : Nat -> Sort u,
    motive 0 ->
    (Π n, motive n -> motive (succ n)) ->
    Π n, motive n
```

一方、`Prop` に属する inductive から任意の `Type` へ elimination するのは慎重にします。

Phase 9 の方針：

```text
I : Type u:
  motive : I -> Sort v を許可

I : Prop:
  原則 motive : I -> Prop のみ許可

例外:
  proof-irrelevant / subsingleton / empty-like な命題は
  restricted large elimination を検討
```

最初は安全側に倒し、`Prop` から `Type` への large elimination は厳しく制限します。

---

## 2.8 Certificate 変更

inductive declaration の certificate に以下を追加します。

```json
{
  "kind": "inductive_block",
  "mutual": true,
  "inductives": [
    {
      "name": "Vec",
      "params": ["A : Type u"],
      "indices": ["n : Nat"],
      "sort": "Type u",
      "constructors": ["nil", "cons"],
      "positivity_certificate": "...",
      "recursor_signature_hash": "sha256:...",
      "iota_rules_hash": "sha256:..."
    }
  ]
}
```

重要なのは、recursor を certificate にそのまま信用して保存するのではなく、checker が inductive declaration から再生成して照合することです。

```text
certificate says:
  recursor_signature_hash = H(...)

checker:
  inductive declaration から recursor を再生成
  hash が一致するか確認
```

---

## 2.9 Advanced inductive の完了条件

```text
- Vec を定義できる
- Fin を定義できる
- mutually recursive Even/Odd を定義できる
- List nested Rose tree を限定的に扱える
- positivity checker が negative occurrence を拒否できる
- recursor / induction principle が自動生成される
- iota reduction が checker と fast kernel で一致する
- certificate に recursor hash / iota hash が含まれる
```

---

# 3. Universe polymorphism 強化

## 3.1 目的

Phase 1 では、core-spec v0.1 の `zero / succ / max / imax / param` と宣言ごとの
universe constraints までを扱います。
Phase 9 では、そこに elaboration 用 universe meta、constraint solving の強化、
canonicalization、必要なら cumulativity を加えて、実用的な polymorphic library を作れるようにします。

Lean では、environment 内の constant が universe parameter を取り、使用時に具体的な universe level で instantiate される universe polymorphism がサポートされています。典型例は `id.{u} {α : Sort u} (x : α) : α` のような定義です。([Lean Language][2])

NPAでも、`List`, `Eq`, `Functor`, `Category` などを universe-polymorphic に扱えるようにします。

---

## 3.2 Level grammar

Phase 9 の elaboration 中の level grammar はこうします。

```text
Level ℓ ::=
  zero
| succ ℓ
| max ℓ₁ ℓ₂
| imax ℓ₁ ℓ₂
| param u
| meta ?u
```

`meta ?u` は elaboration 中だけ使います。
certificate には unresolved universe meta を許しません。

---

## 3.3 Universe constraints

core-spec v0.1 でも universe constraints は certificate に入ります。
Phase 9 では、これを大きな polymorphic library で使えるように solver と canonicalization を強化します。

```text
Constraint ::=
  ℓ₁ ≤ ℓ₂
| ℓ₁ = ℓ₂
```

declaration ごとに universe parameter と constraint set を持たせます。

```json
{
  "name": "List.map",
  "universe_params": ["u", "v"],
  "constraints": [],
  "type": "..."
}
```

より複雑な宣言では：

```json
{
  "universe_params": ["u", "v", "w"],
  "constraints": [
    "max u v <= w"
  ]
}
```

---

## 3.4 Cumulativity

検討すべき設計は2つあります。

```text
Option A:
  universe equality のみ。
  Sort u と Sort v は u = v のときだけ同じ。

Option B:
  cumulativity を導入。
  Sort u : Sort v へ u < v なら持ち上げ可能。
```

MVPでは Option A の方が簡単です。
ただし大規模ライブラリでは cumulativity が便利なので、Phase 9 では Option B を検討します。

導入するなら、typing rule に conversion とは別に cumulativity/subtyping を入れます。

```text
Γ ⊢ t : Sort u
u ≤ v
────────────────
Γ ⊢ t : Sort v
```

ただし、これを definitional equality と混ぜない方がよいです。

```text
defeq:
  u = v

cumulativity:
  u ≤ v による型の持ち上げ
```

---

## 3.5 Universe minimization

elaborator は universe meta を作ります。

```text
?id_univ
```

そして constraint solving で決定します。

```text
?u ≤ max v w
?u = succ v
```

Phase 9 では、universe solver を独立モジュール化します。

```rust
struct UniverseSolver {
    metas: Vec<UniverseMeta>,
    constraints: Vec<UniverseConstraint>,
}
```

certificate 生成時には：

```text
- universe meta が残っていない
- constraints が canonical
- universe params が最小化されている
```

ことを要求します。

---

## 3.6 Universe polymorphism 完了条件

```text
- polymorphic List / Eq / Prod / Sigma が定義できる
- universe-polymorphic theorem が再利用できる
- declaration certificate に universe params / constraints が入る
- unresolved universe meta を certificate が拒否する
- reference checker と fast kernel の universe check が一致する
- universe constraint の canonical hash が安定する
```

---

# 4. Typeclass

## 4.1 目的

Typeclass は、代数構造・notation・自動引数補完の実用性を大きく上げます。

例：

```npa
class Add (A : Type u) where
  add : A -> A -> A

infixl:65 " + " => Add.add
```

これにより、`Nat`, `Int`, `List`, `Matrix` などで同じ `+` notation を使えるようになります。

Lean の typeclass は overloaded operations と関連する型をまとめる仕組みで、複数の型をまたぐ overloading も扱えます。NPAでは、typeclass resolution は elaborator 側の非信頼機構として扱い、kernel には明示的な dictionary argument だけを渡します。([Lean Language][3])

---

## 4.2 Kernel には typeclass を入れない

重要です。

```text
source:
  x + y

elaborator:
  Add.add ?inst x y

typeclass search:
  ?inst := Nat.add_inst

core:
  Add.add Nat Nat.add_inst x y
```

kernel が見るのは普通の関数適用だけです。

```text
Const Add.add Nat Nat.add_inst x y
```

つまり、typeclass search がバグっても、kernel は最終 core term を型検査します。

---

## 4.3 Class declaration

表層構文：

```npa
class Add (A : Type u) where
  add : A -> A -> A
```

内部的には structure / record として扱います。

```npa
structure Add (A : Type u) where
  add : A -> A -> A
```

class は、追加 metadata を持つ structure です。

```json
{
  "kind": "class",
  "name": "Add",
  "fields": [
    {
      "name": "add",
      "type": "A -> A -> A"
    }
  ],
  "searchable": true
}
```

---

## 4.4 Instance declaration

```npa
instance Nat.add_inst : Add Nat where
  add := Nat.add
```

instance は普通の定義です。

```text
Nat.add_inst : Add Nat
```

typeclass database に登録されます。

```json
{
  "class": "Add",
  "target": "Nat",
  "instance": "Nat.add_inst",
  "priority": 1000
}
```

---

## 4.5 Instance search

instance search の入力：

```text
?inst : Add Nat
```

出力：

```text
Nat.add_inst
```

探索規則：

```text
1. local instances
2. explicitly opened namespaces
3. imported global instances
4. lower priority fallback instances
```

必ず制限を入れます。

```text
- max_depth
- max_candidates
- timeout
- cycle detection
- repeated goal cache
```

例：

```json
{
  "max_depth": 16,
  "max_candidates": 128,
  "timeout_ms": 50
}
```

---

## 4.6 Ambiguity

複数 instance が見つかる場合があります。

```text
Add Matrix
  - pointwise addition
  - block addition
```

この場合は勝手に選ばず、エラーにします。

```text
ambiguous instance for Add Matrix
candidates:
  Matrix.pointwise_add
  Matrix.block_add
```

ただし priority が明確なら高 priority を選びます。

---

## 4.7 Typeclass と notation

`+` notation は typeclass を使って解決します。

```npa
x + y
```

elaboration:

```text
Add.add ?inst x y
```

型から：

```text
x : Nat
y : Nat
```

なら：

```text
?inst : Add Nat
?inst := Nat.add_inst
```

最終 core:

```text
Add.add Nat Nat.add_inst x y
```

---

## 4.8 Typeclass 完了条件

```text
- class を structure として elaboration できる
- instance を database に登録できる
- implicit instance argument を補完できる
- +, *, 0, 1 などを typeclass 経由で解決できる
- instance search に timeout / depth / cycle detection がある
- certificate には instance search trace ではなく、明示的 dictionary term が入る
- kernel/checker は typeclass を知らなくても検査できる
```

---

# 5. Quotient

## 5.1 目的

quotient は、同値関係で割った型を扱うために必要です。

例：

```text
整数:
  Nat × Nat を (a,b) ~ (c,d) iff a+d = c+b で割る

有理数:
  Int × Nat+ を同値関係で割る

商群:
  G / N

商環:
  R / I
```

Lean の quotient types は、ある型上の同値関係により粒度を粗くした新しい型を作るもので、関係している要素同士を quotient 上で等しいものとして扱います。([Lean Language][4])

---

## 5.2 設計選択

quotient の実装には2つの選択肢があります。

```text
Option A:
  quotient を axiom として追加する

Option B:
  quotient を kernel primitive として追加する
```

Option A は実装が楽ですが、axiom report に quotient axioms が出ます。
Option B は kernel が増えますが、quotient を基礎機能としてきれいに扱えます。

Phase 9 での推奨は：

```text
MVP:
  quotient は別モジュール Std.Quotient で primitive extension として実装

High-trust mode:
  quotient primitive の checker 実装を reference checker と external checker にも追加
```

つまり、quotient を入れるなら fast kernel だけでなく independent checker 側にも同じ規則を実装します。

---

## 5.3 Setoid

まず同値関係を表す構造を定義します。

```npa
structure Setoid (A : Type u) where
  r : A -> A -> Prop
  refl  : forall x, r x x
  symm  : forall x y, r x y -> r y x
  trans : forall x y z, r x y -> r y z -> r x z
```

notation:

```text
x ≈ y
```

---

## 5.4 Quotient primitive

core constants:

```text
Quotient :
  {A : Type u} -> Setoid A -> Type u

Quotient.mk :
  {A : Type u} -> (s : Setoid A) -> A -> Quotient s

Quotient.sound :
  {A : Type u} -> {s : Setoid A} ->
  s.r a b -> Quotient.mk s a = Quotient.mk s b

Quotient.lift :
  {A : Type u} -> {B : Type v} -> (s : Setoid A) ->
  (f : A -> B) ->
  (forall a b, s.r a b -> f a = f b) ->
  Quotient s -> B
```

computation rule:

```text
Quotient.lift s f h (Quotient.mk s a)
  ↦ f a
```

この computation rule を kernel conversion に入れるかどうかは慎重に決めます。

推奨：

```text
- Quotient.lift の computation rule は definitional equality に入れる
- Quotient.sound は proof term として扱う
- quotient equality を勝手に正規化するような強い計算規則は入れない
```

---

## 5.5 Certificate 変更

quotient primitive を使う証明では、certificate に primitive feature flag を入れます。

```json
{
  "core_features": [
    "quotient_v1"
  ],
  "quotients_used": true
}
```

checker は、`quotient_v1` をサポートしていなければ拒否します。

```text
UnsupportedCoreFeature: quotient_v1
```

---

## 5.6 Quotient 完了条件

```text
- Setoid を定義できる
- Quotient を作れる
- Quotient.sound を使える
- Quotient.lift で well-defined 関数を定義できる
- lift computation が checker と fast kernel で一致する
- quotient を使った Nat pair から Int の簡易版を定義できる
- axiom report / feature report に quotient 使用が出る
```

---

# 6. SMT certificates

## 6.1 目的

SMT solver は、算術・配列・ビットベクトル・論理式の自動証明に強力です。
しかし、solver の結果をそのまま信用してはいけません。

方針はこれです。

```text
SMT solver:
  unsat / valid と言う

NPA:
  SMT proof certificate を受け取る
  certificate を検査またはNPA proofに再構成する
  kernel/checkerで最終検査する
```

Alethe は SMT solver の proof format で、SMT-LIB に基づく柔軟な形式として説明されており、cvc5 のドキュメントも Alethe proof format を SMT-LIB ベースの形式として紹介しています。Carcara は Alethe proof の checker / elaborator として公開されています。([GitLab][5])

---

## 6.2 SMT bridge の構成

```text
NPA goal
  ↓
SMT encoder
  ↓
SMT-LIB problem
  ↓
SMT solver
  ↓
SMT proof certificate, e.g. Alethe
  ↓
SMT certificate checker
  ↓
NPA proof reconstruction
  ↓
kernel check
```

---

## 6.3 対応理論

最初に対応する理論は限定します。

```text
Phase 9 MVP:
  propositional logic
  equality with uninterpreted functions, EUF
  linear integer arithmetic, LIA
  simple Nat-to-Int embedding, optional

Later:
  arrays
  bitvectors
  datatypes
  nonlinear arithmetic
  quantifiers
```

SMT は量化子で非常に難しくなるため、最初は quantifier-free fragment を優先します。

---

## 6.4 SMT encoding

NPA の式を SMT-LIB に変換します。

例：

```npa
a + b = b + a
```

を SMT に送る場合：

```smt2
(declare-const a Int)
(declare-const b Int)
(assert (not (= (+ a b) (+ b a))))
(check-sat)
(get-proof)
```

しかし、NPA の `Nat` は SMT の `Int` と完全には同じではありません。
したがって、encoding には side condition が必要です。

```text
n : Nat
  -> n_smt : Int
  -> assert n_smt >= 0
```

この対応関係も certificate に記録します。

---

## 6.5 SMT certificate checker

2段階に分けるのが安全です。

```text
Stage A:
  external SMT proof checker で Alethe/LFSC 等を検査

Stage B:
  検査済みSMT proof を NPA theorem に再構成
```

あるいは：

```text
SMT proof を直接 NPA proof term に変換し、
kernel が検査する
```

最初の functional MVP では、扱う理論を小さくして、NPA内で proof-producing reconstruction を実装する方がよいです。
AI Machine Profile の最初の SMT milestone はこれより手前で、canonical schema、encoding payload、proof payload、
reconstruction plan の deterministic rejection surface を先に固定します。
SMT certificate success は、非空の encoder table と solver-native rule registry を持つ profile を定義してから有効化します。

---

## 6.6 SMT certificate schema

```json
{
  "kind": "smt_certificate",
  "format": "alethe",
  "solver": "cvc5",
  "logic": "QF_LIA",
  "encoded_goal_hash": "sha256:...",
  "smt_problem_hash": "sha256:...",
  "proof_hash": "sha256:...",
  "reconstruction": {
    "npa_proof_hash": "sha256:...",
    "trusted": false
  }
}
```

`trusted: false` でよいです。
なぜなら最終的には NPA proof term が kernel/checker で検査されるからです。

---

## 6.7 SMT tactic

表層 tactic:

```npa
smt
```

または：

```npa
smt [Nat.add_comm, Nat.add_assoc]
```

内部処理：

```text
1. target を SMT 対応 fragment か判定
2. local hypotheses を encoding
3. SMT solver に送る
4. proof certificate を取得
5. certificate を検査
6. NPA proof term に再構成
7. kernel check
```

`SMT solver says unsat` だけでは絶対に成功にしません。

---

## 6.8 SMT 完了条件

```text
- QF propositional / EUF / simple LIA を encoding できる
- SMT proof certificate を受け取れる
- certificate hash を保存できる
- certificate checker または reconstruction が動く
- final NPA proof term を kernel が検査できる
- solver の unsat 結果だけでは成功扱いしない
- SMT failure / unsupported fragment を構造化エラーとして返せる
```

---

# 7. Theorem graph

## 7.1 目的

theorem graph は、大規模ライブラリ・AI探索・証明最小化・依存関係監査の中核です。
Phase 6/7 の minimal dependency graph / theorem index を土台にし、Phase 9 では
schema、API、ranking、監査用途まで含む本格的な theorem graph に拡張します。

ノード：

```text
definitions
theorems
axioms
inductive types
constructors
recursors
classes
instances
simp rules
```

エッジ：

```text
uses
imports
depends_on_axiom
rewrites_by
applies
instantiates
is_instance_of
is_simp_rule_for
generalizes
specializes
```

---

## 7.2 Graph schema

```rust
struct TheoremNode {
    id: NodeId,
    global_ref: GlobalRef,
    kind: NodeKind,
    statement_hash: Hash,
    proof_hash: Option<Hash>,
    module: ModuleName,
    attributes: Vec<Attribute>,
    axiom_deps: Vec<GlobalRef>,
    constants: Vec<GlobalRef>,
    embeddings: Option<EmbeddingId>,
}
```

edge:

```rust
struct TheoremEdge {
    from: NodeId,
    to: NodeId,
    kind: EdgeKind,
    weight: f32,
}
```

edge kinds:

```text
Uses
Imports
DependsOnAxiom
RewriteRule
SimpRule
ApplyCandidate
InstanceOf
Generalizes
Specializes
SimilarStatement
UsedInProof
```

---

## 7.3 Graph extraction

Graph は certificate から抽出します。

```text
source からではなく certificate から抽出する
```

理由：

```text
- source notation に依存しない
- tactic script に依存しない
- proof minimization 後の正しい依存関係を取れる
- import の export_hash / high-trust 時の certificate_hash と一致する
```

抽出対象：

```text
- type に出現する Const
- proof に出現する Const
- transparent def body に出現する Const
- inductive constructor type に出現する Const
- axiom deps
```

---

## 7.4 Graph API

```json
POST /graph/dependencies
{
  "declaration": "Nat.add_comm",
  "mode": "transitive"
}
```

レスポンス：

```json
{
  "declaration": "Nat.add_comm",
  "dependencies": [
    "Nat.add_zero",
    "Nat.zero_add",
    "Nat.add_succ",
    "Eq.trans",
    "Eq.congrArg"
  ],
  "axioms_used": []
}
```

関連定理検索：

```json
POST /graph/related
{
  "declaration": "List.append_assoc",
  "limit": 10
}
```

レスポンス：

```json
{
  "related": [
    "List.append_nil",
    "List.nil_append",
    "List.length_append",
    "Monoid.assoc"
  ]
}
```

---

## 7.5 AI への利用

theorem graph は premise retrieval を大幅に強化します。

```text
goal constants:
  List.append, Eq

graph expansion:
  List.append_nil
  List.nil_append
  List.append_assoc
  List.length_append
```

検索スコアに graph proximity を入れます。

```text
score += 0.4 * graph_neighborhood_score
```

---

## 7.6 Graph 完了条件

```text
- certificate から dependency graph を抽出できる
- declaration ごとの direct/transitive dependencies が取れる
- axiom dependency path を表示できる
- theorem search が graph score を使える
- proof minimization が graph を使って不要 import を削れる
- graph export が deterministic hash を持つ
```

---

# 8. Natural language formalization

## 8.1 目的

自然言語や LaTeX の命題を、NPA の形式命題候補へ変換します。

例：

```text
任意の自然数 n について n + 0 = n を証明せよ
```

候補：

```npa
theorem user_goal : forall n : Nat, n + 0 = n := by
  _
```

自然言語形式化の候補は、後続の elaboration / confirmation に渡す表層案です。
ここでの `0` は表示上の省略であり、受理される候補は最終的に `Nat.zero` への canonical 参照まで elaboration されなければいけません。

しかし、ここは最も危険な層です。

```text
kernel は「形式化された命題」が証明されたことしか保証しない。
自然言語の意図と一致するかは保証しない。
```

---

## 8.2 Formalization pipeline

```text
natural language / LaTeX
  ↓
parse mathematical entities
  ↓
generate multiple formal candidates
  ↓
typecheck candidates
  ↓
reverse-translate candidates
  ↓
ambiguity report
  ↓
user confirmation or formalization verifier
  ↓
proof search
```

---

## 8.3 Candidate generation

AI formalizer は複数候補を出します。

入力：

```text
任意の正数 x について x^2 > 0
```

候補：

```npa
theorem c1 : forall x : Nat, 0 < x -> 0 < x * x := by _
theorem c2 : forall x : Int, 0 < x -> 0 < x * x := by _
theorem c3 : forall x : Rat, 0 < x -> 0 < x * x := by _
theorem c4 : forall x : Real, 0 < x -> 0 < x * x := by _
```

「正数」がどの型か曖昧なので、候補を分けます。

---

## 8.4 Reverse translation

各候補を自然言語へ戻します。

```json
{
  "candidate": "forall x : Real, 0 < x -> 0 < x * x",
  "paraphrase": "すべての実数 x について、x が 0 より大きければ x*x も 0 より大きい。",
  "ambiguities": [
    "正数を Real と解釈しました。",
    "^2 を multiplication x*x と解釈しました。"
  ]
}
```

ユーザーまたは検証者が確認します。

```json
{
  "confirmed": true,
  "formal_statement_hash": "sha256:..."
}
```

---

## 8.5 Intent certificate

自然言語の意図確認を、数学的 certificate とは別に保存します。

```json
{
  "kind": "intent_certificate",
  "informal_statement": "任意の正数 x について x^2 > 0",
  "formal_statement_hash": "sha256:...",
  "paraphrase": "すべての実数 x について、x が正なら x^2 も正。",
  "confirmed_by": "user",
  "confirmed_at": "2026-05-03T00:00:00Z"
}
```

これは kernel proof certificate ではありません。
ただし監査上は重要です。

---

## 8.6 Formalization validation

自然言語形式化は、証明探索とは分離します。

```text
Step 1:
  formal statement を確定

Step 2:
  proof search
```

証明探索が成功しても、形式化が間違っていれば意味がありません。

したがって UI では必ずこう表示します。

```text
この形式命題を証明します:

  forall x : Real, 0 < x -> 0 < x * x

意味:
  すべての実数 x について、x が正なら x*x は正。

曖昧性:
  「正数」を Real と解釈しました。
```

---

## 8.7 Natural language formalization 完了条件

```text
- 自然言語/LaTeX から複数形式候補を生成できる
- 候補を型検査できる
- 候補を自然言語へ逆翻訳できる
- 曖昧性を表示できる
- confirmed formal statement hash を保存できる
- proof certificate と intent certificate を分離できる
- 未確認の自然言語形式化を verified と呼ばない
```

---

# 9. Phase 9 API 追加

## 9.1 Advanced inductive

```json
POST /inductive/check
{
  "declaration": "inductive Vec ..."
}
```

レスポンス：

```json
{
  "status": "ok",
  "constructors": ["Vec.nil", "Vec.cons"],
  "recursor": "Vec.rec",
  "positivity": "passed",
  "iota_rules_hash": "sha256:..."
}
```

---

## 9.2 Typeclass search

```json
POST /typeclass/search
{
  "goal": "Add Nat",
  "context": [],
  "budget": {
    "timeout_ms": 50,
    "max_depth": 16
  }
}
```

レスポンス：

```json
{
  "status": "success",
  "instance": "Nat.add_inst",
  "core_term": "Nat.add_inst",
  "search_trace": [
    "found global instance Nat.add_inst"
  ]
}
```

---

## 9.3 SMT

```json
POST /smt/prove
{
  "state_id": "st_100",
  "goal_id": "g1",
  "logic": "QF_LIA",
  "require_certificate": true
}
```

レスポンス：

```json
{
  "status": "success",
  "smt_format": "alethe",
  "smt_problem_hash": "sha256:...",
  "smt_proof_hash": "sha256:...",
  "npa_proof_hash": "sha256:...",
  "kernel_checked": true
}
```

---

## 9.4 Theorem graph

```json
POST /graph/query
{
  "start": "Nat.add_comm",
  "edge_kinds": ["Uses"],
  "depth": 2
}
```

---

## 9.5 Natural language formalization

```json
POST /formalize
{
  "input": "任意の自然数 n について n + 0 = n を証明せよ",
  "language": "ja",
  "limit": 5
}
```

レスポンス：

```json
{
  "candidates": [
    {
      "formal": "theorem user_goal : forall n : Nat, n + 0 = n := by _",
      "paraphrase": "すべての自然数 n について、n + 0 = n。",
      "confidence": 0.97,
      "ambiguities": []
    }
  ]
}
```

---

# 10. Phase 9 の推奨実装順序

```text
1. universe polymorphism強化
   universe constraints, metas, canonicalization

2. advanced inductive
   indexed families, recursor generation, positivity

3. theorem graph
   certificate から依存グラフを抽出

4. typeclass
   class/instance elaboration, dictionary passing

5. quotient
   Setoid, Quotient primitive, lift/sound

6. SMT certificates
   QF propositional / EUF / LIA から始める

7. natural language formalization
   候補生成、逆翻訳、intent certificate
```

理由：

```text
universe と inductive:
  他の高度機能の基礎

theorem graph:
  typeclass / AI / search を改善する

typeclass:
  algebraic hierarchy に必要

quotient:
  実数学・商構造に必要だが kernel 拡張なので慎重に

SMT:
  自動化を強化するが、証明証明書検査が必須

natural language:
  便利だが意味一致問題があるため最後
```

---

# 11. Phase 9 の完了条件

Phase 9 が完了したと言える条件はこれです。

```text
Advanced inductive:
  Vec, Fin, mutual Even/Odd, 限定的 nested inductive が通る

Universe:
  polymorphic List/Eq/Functor/Category 的定義が扱える

Typeclass:
  Add, Mul, Zero, One, Semigroup, Monoid の instance search が動く

Quotient:
  Setoid quotient と Quotient.lift が使える

SMT certificates:
  solver結果だけでなく proof certificate を検査または再構成できる

Theorem graph:
  依存関係、axiom path、関連定理検索、premise retrieval に使える

Natural language:
  形式候補、逆翻訳、曖昧性、intent certificate を生成できる
```

---

# 12. Phase 9 の一文要約

Phase 9 は、**小さく安全な証明支援系を、実用数学・自動証明・AI形式化に耐える高度な証明基盤へ拡張する段階**です。

ただし、信頼境界は変えません。

```text
kernel/checker に入れる:
  inductive, universe, quotient の最小規則

非信頼層に置く:
  typeclass search, SMT solver, theorem graph, AI formalizer

最終的に検査する:
  explicit core proof certificate
```

つまり Phase 9 の目標は、便利機能を増やすことではなく、**便利機能を増やしても verified の意味を壊さないこと**です。

[1]: https://lean-lang.org/doc/reference/latest/The-Type-System/Inductive-Types/?utm_source=chatgpt.com "4.4. Inductive Types"
[2]: https://lean-lang.org/doc/reference/latest/The-Type-System/Universes/?utm_source=chatgpt.com "4.3. Universes"
[3]: https://lean-lang.org/doc/reference/latest/Type-Classes/?utm_source=chatgpt.com "10. Type Classes"
[4]: https://lean-lang.org/doc/reference/latest/The-Type-System/Quotients/?utm_source=chatgpt.com "Quotients"
[5]: https://verit.gitlabpages.uliege.be/alethe/specification.pdf?utm_source=chatgpt.com "The Alethe Proof Format"
