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

実装・完了メモ（2026-05-25）:

```text
- この文書は Phase 9 Human Profile の user-facing / kernel-facing 完了状態を記述する
- 現リポジトリでは P9H-00 から P9H-15 までの Human target scope を実装済み
- Phase 9 AI deterministic validation / replay substrate と M9 fixture matrix も実装済みだが、
  これは高度機能候補を検査境界へ戻す非信頼 Machine Profile であり、
  Human Profile の kernel / checker-facing rules を置き換えない
- P9H-00 の境界 regression は
  `p9h00_advanced_ai_sidecars_scores_and_smt_outputs_stay_untrusted` と
  `p9h00_ai_fast_path_request_shapes_exclude_phase9_human_heavy_checks` で固定する
- production LLM / RAG / online theorem graph store / external SMT solver service operation は
  target integration として残し、実装済みとは書かない
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

上の番号は設計領域の並びです。
実装順序は後述の #10 を正とし、まず **universe polymorphism と advanced inductive** を固めます。
この2つは typeclass・quotient・大規模ライブラリの土台になるため、他の高度機能より先に検査規則と
certificate 形式を安定させます。

## 1.1 AI hot path の性能境界

Phase 9 Human Profile で追加する高信頼検査や重い再構築処理は、AI 候補生成の通常経路を遅くしないように
実行位置を分けます。

```text
AI candidate hot path:
  bounded typeclass search
  precomputed theorem graph snapshot query
  Phase 9 AI deterministic candidate validation
  fast kernel / Phase 5-7 replay に戻す軽量 verify

release / audit / adoption path:
  full independent checker
  external checker profile
  theorem graph extraction from certificates
  SMT proof certificate checking / NPA proof reconstruction
  quotient-capable checker support
  high-trust release audit
```

したがって theorem graph は候補ごとに certificate 全体から抽出せず、build / release / index update で作った
deterministic snapshot を検索します。SMT solver、quotient primitive、full independent checker、release audit は
証明採用・高信頼化・release 判断の境界で実行し、AI ranking や candidate enumeration の inner loop には入れません。
AI 側の速度要件は Phase 9 AI Profile の bounded validation / deterministic fixture matrix と対応させます。
この対応は P9H-00 regression として、Phase 5-7 replay / verify request の candidate hash と
state fingerprint に Phase 9 Human metadata を混ぜないことまで test 名で追跡します。

## 1.2 Release completion gate と target integration の境界

Phase 9 Human の完了状態は、docs、tests、release gate で同じ意味に揃えます。

```text
required completion gate:
  ./scripts/phase9-regression.sh

gate contents:
  Phase 9 AI M9 deterministic fixture matrix
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace

also wired as:
  GitHub Actions: Phase 9 Regression / phase9-regression
```

この gate は release / high-trust の pass/fail を、checker result と deterministic artifact だけで
決める方針を確認します。AI sidecar、theorem graph score、formalization confidence、SMT solver output は
診断・探索・監査 metadata であり、trusted boundary には入りません。

Phase 9 Human target scope はこのリポジトリ内の Rust crates と fixtures で完了しています。
一方で、production AI orchestrator、LLM / RAG 接続、online graph store、full external SMT solver
support、非空 solver-native SMT success profile は target integration として残します。
これらを `./scripts/phase9-regression.sh` や PR の AI candidate enumeration に同期追加して、
AI hot path の速度を落としてはいけません。

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

P9H-05 実装では、`MutualInductiveBlock` を kernel / certificate / source-free reference checker の
共通境界に追加します。trusted base を広げる理由は、相互再帰 family の constructor minor premise と
iota rule が block 全体の constructor 順序に依存し、単独 inductive の後処理だけでは検査境界を
閉じられないためです。代替案として、mutual を表層 elaborator だけで単独 inductive 群へ desugar する方法は
検討できますが、recursor / induction principle / iota rule の同時生成を certificate checker から
見えなくするため採用しません。

checker boundary は次の形に固定します。certificate は block-local family / constructor / recursor を
canonical generated declarations として持ちますが、recursor は declaration から deterministic に再生成して照合します。
fast kernel と reference checker は同じ block-wide minor premise order で iota reduction を実行し、
`Even` / `Odd` の source-free certificate と iota fixture で一致を確認します。

AI candidate hot path には mutual inductive の candidate precheck や source-free reference checker を同期挿入しません。
AI 側は従来どおり bounded candidate validation と deterministic replay を走らせ、mutual block の重い検査は
proof acceptance / post-acceptance checker 境界でのみ実行します。

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

P9H-06 実装では、generic positivity checker ではなく、次の approved strictly-positive functor profile に閉じて
nested occurrence を許可します。

```text
approved nested functor table:
  List   arity 1  positive args [0]
  Option arity 1  positive args [0]
  Prod   arity 2  positive args [0, 1]
```

kernel / certificate generator / source-free reference checker はこの table と同じ traversal を使います。
recursive occurrence は direct family reference、または approved functor の positive argument 内だけ許可されます。
Pi / function type の domain 側に recursive occurrence が現れる場合は、二重否定風の higher-order shape も含めて拒否します。
unknown type constructor 越しの recursive occurrence も拒否します。

この profile の recursor は既存の deterministic generated recursor / iota hash pipeline に接続します。
approved nested field は constructor minor premise に field として現れ、recursor signature hash と iota rule hash は
canonical encode / decode 後も stable です。functor contents 全体への generic induction hypothesis 生成は
future target の generic positivity / functoriality profile に残します。

AI candidate hot path には approved nested profile の source-free checker や full certificate recheck を同期挿入しません。
AI 側の bounded candidate validation は従来通りで、nested inductive の重い検査は proof acceptance /
post-acceptance checker 境界でのみ実行します。

---

## 2.6 Positivity checker 強化

Phase 1 の positivity checker は保守的でした。
Phase 9 では、次を判定できるようにします。

```text
OK:
  I
  A -> I              A は I を含まない外部型
  Nat -> I            constructor argument としての正の出現
  List I
  Option I
  Prod A I

NG:
  I -> A
  I -> I
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

P9H-04 実装では、`params` と `indices` を certificate の inductive payload に分離したまま、
indexed family の recursor を次の canonical binder order で生成・照合します。

```text
params,
motive : Π indices, I params indices -> Sort v,
constructor minor premises,
indices,
major : I params indices
```

`Vec` / `Fin` の fixtures は fast certificate verifier と source-free reference checker の両方で再検査され、
recursor signature hash と iota rules hash は certificate 側の deterministic hash helper から取得します。
Human API の `/inductive/check` 相当 wrapper はこの metadata を診断として返すだけで、
proof acceptance boundary は canonical `.npcert` verification と independent checker のままです。

P9H-05 実装では、mutual block についても同じ規則を拡張します。block 内の family / constructor /
recursor export は canonical name order で安定化し、generated recursor artifact hash は canonical encode /
decode 後も変わりません。import / theorem index / current-declaration projection は mutual block の root name と
generated declarations を読み取れますが、AI trace、ranking、candidate sidecar は certificate hash や checker verdict に
影響しません。

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

実装上の既定は **Option A: equality-only universe policy** とします。
MVP certificate / kernel / reference checker は declaration-local universe constraints を
canonical data として保持・hash しますが、`Sort u` から `Sort v` への持ち上げを
definitional equality に混ぜません。cumulativity / subtyping rule は明示 feature flag と
別 milestone の rule 追加なしには有効化しません。

P9H-03 時点の標準ライブラリ regression では、`Std.Logic` の `Eq` / `And` / `Exists`、
`Std.List` の `List` / `List.map` / `List.map_comp`、`Std.Algebra.Basic` の
`Associative` / `IsMonoid` 系 declaration を、source-free reference checker で検査できる
polymorphic export として固定します。`And` は product-like、`Exists` は sigma-like な
Prop-level fixture として扱い、Phase 9 の新しいデータ型追加は advanced inductive / typeclass 側の
後続 milestone に残します。これらの universe params / constraint context は declaration interface hash、
certificate hash、release manifest、import bundle、theorem index global ref に反映されるため、
Human 側の universe inference を追加しても Machine / AI retrieval の candidate hash には混ぜません。
certificate に残った `?u` や Human elaborator 内部 prefix の universe meta は、kernel producer、
fast verifier、reference checker のすべてで拒否します。

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

P9H-09 の MVP 実装では、kernel に新しい typeclass primitive は追加しません。Human `class`
は ordinary inductive declaration として class head / `mk` constructor / generated recursor を
certificate に出し、field projection は ordinary declaration として明示します。検索用の
`HumanTypeclassClassMetadata` は source interface 側にだけ保持し、certificate hash の入力にはしません。

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

P9H-09 では search trace は生成しません。P9H-10 では Human 側だけに bounded search trace を
追加しますが、`instance` は class constructor に field 値を渡す ordinary definition に elaboration
され、最終 certificate には explicit dictionary term だけが残ります。metadata や search trace が
壊れても proof acceptance boundary は変わらず、checker は certificate 内の core term を型検査します。

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

P9H-10 の実装では、current module の instance を最優先し、imported source interface の
instance は opened namespace に属するものを次順位、それ以外を fallback として扱います。
同じ順位と priority の中で複数の異なる proof term が成立する場合は score で選ばず ambiguity にします。

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

policy は `HumanTypeclassSearchPolicy` として Human compile/API options に保持します。この policy は
Human search のみを制限し、Machine Surface schema や AI candidate hot path には同期的な追加処理を入れません。

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

ambiguity / no solution / budget exceeded は structured diagnostic/status として返します。
search trace は診断 metadata であり、certificate hash、candidate payload hash、proof acceptance boundary
のいずれにも含めません。

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

`*` は `Mul.mul`、`0` は `Zero.zero`、`1` は `One.one` への Human notation として扱い、
elaboration 後はそれぞれ明示的な dictionary term を引数に持つ ordinary core application になります。
kernel と checker は typeclass search を知らず、最終的な dictionary term だけを検査します。

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

`quotient_v1` の canonical core primitive interface は implicit を持たないため、
kernel / certificate / reference checker では次の explicit spine に固定します。

```text
Setoid.{u}        : Type u -> Type u
RelEquiv.{u}      : (A : Type u) -> (A -> A -> Prop) -> Prop
Setoid.mk.{u}     : (A : Type u) -> (r : A -> A -> Prop) -> RelEquiv A r -> Setoid A
Setoid.r.{u}      : (A : Type u) -> Setoid A -> A -> A -> Prop

Quotient.{u}      : (A : Type u) -> Setoid A -> Type u
Quotient.mk.{u}   : (A : Type u) -> (s : Setoid A) -> A -> Quotient A s
Quotient.sound.{u}: (A : Type u) -> (s : Setoid A) -> (a b : A) ->
                    Setoid.r A s a b ->
                    Eq (Quotient A s) (Quotient.mk A s a) (Quotient.mk A s b)
Quotient.lift.{u,v}:
  (A : Type u) -> (B : Type v) -> (s : Setoid A) ->
  (f : A -> B) ->
  (forall a b, Setoid.r A s a b -> Eq B (f a) (f b)) ->
  Quotient A s -> B
```

`quotient_v2` は `quotient_v1` の primitive を変更せず、binary quotient operation 用の
opt-in extension として次だけを追加します。

```text
Quotient.lift2.{u,v}:
  (A : Type u) -> (B : Type v) -> (s : Setoid A) ->
  (f : A -> A -> B) ->
  (forall a a2 b b2,
    Setoid.r A s a a2 ->
    Setoid.r A s b b2 ->
    Eq B (f a b) (f a2 b2)) ->
  Quotient A s -> Quotient A s -> B
```

`quotient_v3` は proposition-valued quotient induction のための opt-in extension として次を追加します。
これは arbitrary quotient element に対する equality proposition を certificate 内で代表元へ戻すために
使います。

```text
Quotient.indProp.{u}:
  (A : Type u) -> (s : Setoid A) ->
  (motive : Quotient A s -> Prop) ->
  (mk_case : forall a, motive (Quotient.mk A s a)) ->
  forall q, motive q
```

computation rule:

```text
Quotient.lift s f h (Quotient.mk s a)
  ↦ f a

Quotient.lift2 s f h (Quotient.mk s a) (Quotient.mk s b)
  ↦ f a b

Quotient.indProp s P h (Quotient.mk s a)
  ↦ h a
```

この computation rule を kernel conversion に入れるかどうかは慎重に決めます。

推奨：

```text
- Quotient.lift の computation rule は definitional equality に入れる
- Quotient.lift2 の computation rule は `quotient_v2` opt-in profile の definitional equality に入れる
- Quotient.indProp の computation rule は `quotient_v3` opt-in profile の definitional equality に入れる
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

`Quotient.lift2` を使う certificate は `quotient_v2` も併記します。

```json
{
  "core_features": [
    "quotient_v1",
    "quotient_v2"
  ],
  "quotients_used": true
}
```

`Quotient.indProp` を使う certificate は `quotient_v3` も併記します。

```json
{
  "core_features": [
    "quotient_v1",
    "quotient_v3"
  ],
  "quotients_used": true
}
```

checker は、必要な quotient feature をサポートしていなければ拒否します。

```text
UnsupportedCoreFeature: quotient_v1
UnsupportedCoreFeature: quotient_v2
UnsupportedCoreFeature: quotient_v3
```

実装上は、`core_features` は axiom report hash の入力に含めます。ただし空の feature report は
既存 certificate bytes に追記しないため、quotient を使わない Phase 5-8 / Phase 9 AI hot path の
certificate identity hash は変えません。`quotient_v1` を含む certificate だけ、axiom report hash と
certificate hash が deterministic に変わります。

`Quotient` / `Quotient.mk` / `Quotient.sound` / `Quotient.lift` / `Quotient.lift2` /
`Quotient.indProp` および `Setoid` 系 primitive の exact name は reserved core primitive です。
同名の local axiom として silently allowed にはしません。

trusted base を広げる理由は、quotient equality を custom axiom 群で表す Option A だと checker 間で
axiom policy と computation rule が drift しやすいためです。代替案として quotient を Std module の
通常 axiom として持つ案は残せますが、high-trust profile では fast kernel と reference checker が同じ
primitive interface / `Quotient.lift` 計算規則を実装する境界を採用します。AI / tactic / elaborator は
この primitive を証明受理条件にできず、canonical certificate の feature report と checker profile だけが
受理境界です。

P9H-12 では、この境界を `quotient_v1` opt-in として実装します。第一同型定理の商群積のように
binary quotient operation を certificate 化する場合は、同じ境界で `quotient_v2` を追加します。
arbitrary quotient element に対する equality proposition を証明する場合は、同じ境界で `quotient_v3`
を追加します。代替案は unary `Quotient.lift` と関数外延性で curried operation を作る方法、または
より一般の dependent quotient eliminator を入れる方法です。ただし前者は別の強い原理を trusted base に
入れる必要があり、後者は elimination 境界が大きいため、FI5c では Prop-valued induction のみに限定します。

```text
- fast kernel と source-free reference checker は `RelEquiv A r` を equivalence witness 型へ展開する
- `Setoid.r A (Setoid.mk A r h) a b` は `r a b` に WHNF reduction する
- `Quotient.lift A B s f h (Quotient.mk A s a)` は `f a` に WHNF reduction する
- `Quotient.lift2 A B s f h (Quotient.mk A s a) (Quotient.mk A s b)` は `f a b` に WHNF reduction する
- `Quotient.indProp A s P h (Quotient.mk A s a)` は `h a` に WHNF reduction する
- `Std.Quotient` example certificate は Nat × Nat の `IntPair`、`Setoid`、`Quotient.mk`、
  `Quotient.sound`、`Quotient.lift` を custom axiom / sorry なしで使う
```

この成功経路は `quotient_v1` を許可した checker profile だけで有効です。Phase 9 AI の既定
`Phase8MvpReference` profile は quotient を引き続き deterministic `UnsupportedFeature` として扱うため、
AI candidate hot path、MVP certificate identity、既存 stdlib release profile には同期的な追加検査を入れません。

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
  "format": "mvp_proof_node_table_v1 | alethe_opaque_v1 | lfsc_opaque_v1 | solver_result_only_v1",
  "solver": "cvc5",
  "logic": "QF_LIA",
  "encoded_goal_hash": "sha256:...",
  "smt_problem_hash": "sha256:...",
  "proof_hash": "sha256:...",
  "reconstruction": {
    "rule_registry_profile": "mvp_empty_registry_v1",
    "reconstruction_plan_hash": "sha256:...",
    "imported_theory_count": 0,
    "step_count": 1,
    "trusted": false
  }
}
```

`trusted: false` でよいです。
なぜなら最終的には NPA proof term が kernel/checker で検査されるからです。
P9H-13 時点では、SMT-LIB problem bytes / hash、encoded problem hash、proof payload hash を deterministic に固定し、
Alethe / LFSC は opaque proof artifact として hash / size / schema validation だけを行います。
`solver_result_only` は明示的な structured rejection であり、solver の `unsat` だけでは success になりません。
この schema 生成と validation は tactic adoption / audit 境界の処理であり、AI candidate enumeration の inner loop には入れません。

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

P9H-07 実装では、最初の certificate-derived extractor を `crates/npa-api` に置きます。
入力は canonical `.npcert` bytes と verified import bindings だけで、source notation、tactic script、
Human debug metadata、AI sidecar は graph extraction input に含めません。

MVP schema は次を固定します。

```text
snapshot:
  source module + source export_hash + source certificate_hash + extractor_version

node identity:
  scope + module + name + decl_interface_hash

node kinds:
  Axiom / Definition / Theorem / Inductive / Constructor / Recursor / Builtin / Unknown

edge identity:
  from node id + to node id + edge kind

edge kinds:
  ImportsDeclaration
  MentionsType
  UsesConstant
  GeneratedDeclaration
  DependsOnDirectAxiom
  DependsOnTransitiveAxiom
```

extractor は declaration type、theorem proof、transparent def body、inductive constructor type、
recursor type、axiom dependency report から `Const` を抽出します。graph hash は source module、
source `export_hash`、source `certificate_hash`、import binding、sorted node set、sorted edge set から
deterministic に計算します。import は normal path では `export_hash`、high-trust path では
`certificate_hash` も snapshot binding として検査します。

この extractor は offline/source-free な post-certificate artifact です。AI candidate hot path や
ranking inner loop に同期挿入せず、Phase 9 AI theorem graph query は引き続き bounded snapshot query
として扱います。

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

P9H-08 実装では、HTTP handler の手前に置く Human API wrapper として
`crates/npa-api` に次を追加します。

```text
certificate_theorem_graph_dependencies
certificate_theorem_graph_related
certificate_theorem_graph_query
certificate_theorem_graph_unused_imports
```

各 wrapper は P9H-07 の `CertificateTheoremGraphSnapshot` だけを入力にし、結果 node は
certificate-bound public export に限定します。`Builtin` / `Unknown` node や certificate export に
結びつかない metadata node は返しません。direct / transitive dependency と related query の並び順は
hash ではなく構造化 node id / deterministic integer score で固定します。

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

P9H-08 の `ai_search_premise_graph_ranking_features` は、Phase 7 premise retrieval の結果に対して
precomputed snapshot から graph score sidecar を付けるだけです。snapshot がない場合は既存の
Machine API result order を graph score 0 の deterministic fallback として保持します。

この sidecar は candidate payload hash、replay plan、verify request、checker result を変更しません。
graph extraction は build / release / index update 側で済ませ、AI candidate ごとの enumeration / replay /
verify hot path では実行しません。

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
  "diagnostic_only": true,
  "iota_rules_hash": "sha256:..."
}
```

この response は IDE / Human UI 向けの診断 metadata であり、AI candidate hot path には同期挿入しません。
証明採用は従来どおり kernel と canonical certificate verifier、必要に応じて reference checker の判定に戻します。

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

`/typeclass/search` は Human API wrapper であり、成功時の `core_term` は kernel-checkable な
dictionary term です。`search_trace` は Human UI / IDE 向けの bounded diagnostic metadata であり、
certificate に入らず、AI 側の `AdvancedMachineTypeclassResolutionPlan` や Machine API payload hash も
変更しません。AI 向け fixture は従来どおり `cargo test -p npa-api advanced_ai` で検証し、Human search の
timeout / budget は AI hot path latency を増やさないための境界として扱います。

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
