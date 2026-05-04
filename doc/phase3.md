以下は **Phase 3: 表層言語** の詳細設計です。
Phase 1/2 で作った kernel / certificate は「完全に明示化された core AST」だけを扱います。Phase 3 では、人間が書きやすい構文を導入し、それを **canonical core AST** に落とします。

重要な前提はこれです。

```text
parser / names / notation / implicit args / holes / elaborator
は便利な非信頼層。

最終的な正しさは、
Phase 1 の kernel と Phase 2 の certificate checker が確認する。
```

---

# 1. Phase 3 の目的

Phase 3 の入力は、人間向けソースです。

```npa
namespace Nat

def add (n m : Nat) : Nat :=
  rec (fun _ => Nat) n (fun _ ih => succ ih) m

theorem add_zero (n : Nat) : add n zero = n :=
  Eq.refl n

end Nat
```

これを、Phase 1/2 の形式に変換します。

```text
surface source
  ↓ parser
surface AST
  ↓ name resolution
resolved AST
  ↓ notation expansion
desugared AST
  ↓ elaboration
elaboration AST + metavariables/goals
  ↓ metavariable solving
fully explicit core AST
  ↓ certificate generation
canonical certificate
```

Phase 3 の成功条件は、次です。

```text
- 人間が簡単な def/theorem/axiom/simple inductive を書ける
- import / namespace / open と名前解決が動く
- infix notation などを使える
- 省略引数を補える
- `_` や `?m` で holes を作れる
- simple elaboration で core term に変換できる
- unresolved hole がある場合は certificate に出さず、goal として表示する
```

ここでいう `elaboration AST` は、core term と同じ構造を持ちますが、elaboration 中だけ
metavariable を含められる一時表現です。Phase 1/2 に渡す `fully explicit core AST` には
metavariable、hole、notation、implicit argument metadata を一切残しません。

---

# 2. Parser

## 2.1 Parser の責任

parser は、文字列を surface AST に変換します。

parser がやること：

```text
- 字句解析
- 括弧構造の解析
- declaration の解析
- binder 構文の解析
- term 構文の解析
- notation の構文解析
- source location の保存
```

parser がやらないこと：

```text
- 名前解決
- 型推論
- implicit argument 推論
- typeclass 探索
- theorem search
- kernel check
```

たとえば parser は：

```npa
x + y = x
```

を見ても、`+` が Nat の加法なのか、Int の加法なのか、別の演算なのかは判断しません。
それは elaboration 側の仕事です。

## 2.2 最小構文

Phase 3 では、まずこの程度で十分です。

```text
module
  import
  open
  namespace
  end
  notation declaration
  def
  theorem
  axiom
  simple inductive

term
  identifier
  explicit universe application
  Sort
  Prop
  Type
  application
  lambda
  forall / Pi
  let
  annotation
  hole
  parenthesized term
  simple notation
```

表層文法の例：

```text
item ::=
    "import" qual_name
  | "open" qual_name
  | "namespace" name
  | "end" name?
  | notation_decl
  | def_decl
  | theorem_decl
  | axiom_decl
  | inductive_decl

def_decl ::=
    "def" name universe_params? decl_binder* ":" term ":=" term

theorem_decl ::=
    "theorem" name universe_params? decl_binder* ":" term ":=" term

axiom_decl ::=
    "axiom" name universe_params? decl_binder* ":" term

inductive_decl ::=
    "inductive" name universe_params? decl_binder* ":" term "where" ctor_decl+

ctor_decl ::=
    "|" name ":" term

notation_decl ::=
    ("prefix" | "postfix" | "infix" | "infixl" | "infixr")
    ":" precedence string "=>" qual_name

decl_binder ::=
    "(" ident ":" term ")"
  | "{" ident ":" term "}"

lambda_binder ::=
    decl_binder
  | ident

pi_binder ::=
    decl_binder

term ::=
    ident universe_args?
  | "@" ident universe_args?
  | "Prop"
  | "Type" level?
  | "Sort" level
  | term term
  | "fun" lambda_binder+ "=>" term
  | "forall" pi_binder+ "," term
  | "let" ident ":" term ":=" term "in" term
  | "let" ident ":=" term "in" term
  | term ":" term
  | "_"
  | "?" ident
  | "(" term ")"

universe_params ::= ".{" ident ("," ident)* "}"
universe_args   ::= ".{" level ("," level)* "}"

level ::=
    natural
  | ident
  | "succ" level
  | "max" level level
  | "imax" level level
```

MVP では、declaration binder と `forall` binder は型注釈必須です。`fun x => ...` のような
未注釈 lambda binder だけは、期待型がある check mode で補います。数値リテラルや
typeclass-driven overload は Phase 3 では扱わず、自然数のゼロは `Nat.zero` か開いた namespace
内の `zero` として書きます。

`inductive` は Phase 1/2 の `simple inductive` に落とせる形だけです。mutual / nested /
coinductive、pattern matching syntax、ユーザー定義 macro は Phase 3 MVP に含めません。

## 2.3 Surface AST

parser の出力は core AST ではなく surface AST です。

Rust風には：

```rust
enum ImplicitMode {
    Insert,
    Explicit,
}

enum SurfaceExpr {
    Ident {
        name: SurfaceName,
        universe_args: Option<Vec<SurfaceLevel>>,
        implicit_mode: ImplicitMode,
        span: Span,
    },
    Sort {
        level: SurfaceLevel,
        span: Span,
    },
    App {
        func: Box<SurfaceExpr>,
        arg: Box<SurfaceExpr>,
        span: Span,
    },
    Lam {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Pi {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Let {
        name: SurfaceName,
        ty: Option<Box<SurfaceExpr>>,
        value: Box<SurfaceExpr>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Annot {
        expr: Box<SurfaceExpr>,
        ty: Box<SurfaceExpr>,
        span: Span,
    },
    Hole {
        name: Option<SurfaceName>,
        span: Span,
    },
    Notation {
        head: NotationHead,
        args: Vec<SurfaceExpr>,
        span: Span,
    },
}
```

`implicit_mode` は通常 `Insert` で、`@Eq.refl` のように書いた場合だけ `Explicit` にします。
`Explicit` の head では implicit args を自動挿入せず、ユーザーがすべての引数を書く必要があります。

`Span` は IDE とエラー表示のために必須です。

```rust
struct Span {
    file_id: FileId,
    start: ByteOffset,
    end: ByteOffset,
}
```

source map は trusted な certificate payload には入れません。デバッグ用 section として
同梱する場合でも、kernel の検査、canonical hash、axiom report には使いません。

---

# 3. Names

## 3.1 名前の種類

Phase 3 で名前設計を雑にすると、後でライブラリが破綻します。
名前は最初から階層化します。

```text
Nat
Nat.zero
Nat.succ
Nat.add
Eq
Eq.refl
Algebra.Group.mul_assoc
```

内部表現：

```rust
struct Name(Vec<NamePart>);

enum NamePart {
    Str(String),
    Num(u32),
}
```

将来的には高速化のため、文字列ではなく intern します。

```rust
struct NameId(u32);
```

## 3.2 Local name と Global name

名前には大きく2種類あります。

```text
local name:
  λ や theorem binder で導入された変数

global name:
  def/theorem/axiom/inductive で定義された宣言
```

例：

```npa
theorem t (Nat : Type) (x : Nat) : x = x :=
  Eq.refl x
```

この場合、`Nat` は local name として global `Nat` を shadow します。
このような shadowing を許すかどうかは設計判断ですが、MVPでは許してもよいものの、warningを出すのが望ましいです。

```text
warning:
  local name `Nat` shadows global declaration `Nat`
```

## 3.3 名前解決の順序

未修飾名 `add` を解決するときの順序は、明示します。

```text
1. local context
2. current namespace
3. opened namespaces
4. imported global declarations
```

例：

```npa
namespace Nat

def double (n : Nat) : Nat :=
  add n n

end Nat
```

ここで `add` は `Nat.add` に解決されます。

`open Nat` は現在の namespace scope だけに効く非信頼の elaboration metadata です。
同じ短縮名が複数の opened/imported namespace から来る場合は、勝手に選ばず ambiguity とします。
aliases は Phase 3 MVP には入れません。

## 3.4 Ambiguity

同じ短縮名が複数候補に解決されることがあります。

```text
Nat.add
Int.add
Rat.add
```

`add` だけでは曖昧です。

Phase 3 の方針：

```text
- 型情報なしで一意なら即解決
- 複数候補があるなら OverloadedRef として保持
- elaboration 中に期待型から解決を試みる
- 解決不能ならエラー
```

Surface AST から resolved AST への変換では、次のような中間表現を許します。

```rust
enum ResolvedName {
    Local(LocalId),
    Global(GlobalRef),
    Overloaded(Vec<GlobalRef>),
    Unresolved(SurfaceName),
}
```

## 3.5 GlobalRef

Phase 2 の certificate と接続するため、global name は名前だけでなく hash 付き参照にします。

```rust
struct GlobalRef {
    module: ModuleName,
    name: NameId,
    decl_interface_hash: Hash,
}
```

これにより：

```text
同じ `Nat.add` という名前だが中身が違う
```

という問題を防げます。

これは source/elaboration 層の参照です。Phase 2 certificate へ渡す時点では、
`import_index` または current module の declaration index と `decl_interface_hash` を使う
canonical `Const` 参照に変換します。module 名や表示名だけを trusted payload の根拠にしません。

---

# 4. Notation

## 4.1 Notation の目的

人間はこう書きたいです。

```npa
n + Nat.zero = n
```

core はこうです。

```text
Eq Nat (Nat.add n Nat.zero) n
```

notation は、表層構文と core constant の橋渡しです。

## 4.2 Phase 3 の notation は小さく始める

最初はこの3種類で十分です。

```text
prefix
infix
postfix
```

例：

```npa
infixl:65 " + " => Nat.add
infix:50 " = " => Eq
```

ただし `=` は型 `A` を暗黙引数として取るため、実際には：

```text
x = y
```

を

```text
Eq ?A x y
```

に展開し、`?A` は elaboration が推論します。

## 4.3 Notation table

notation はテーブルで管理します。

```rust
struct NotationEntry {
    symbol: String,
    kind: NotationKind,
    precedence: u16,
    associativity: Associativity,
    target: NotationTarget,
    namespace: Option<Name>,
}

enum NotationKind {
    Prefix,
    Infix,
    Postfix,
}

enum Associativity {
    Left,
    Right,
    NonAssoc,
}

enum NotationTarget {
    Global(GlobalRef),
}
```

Phase 3 では `GlobalRef` への展開だけを許します。syntax macro やユーザー定義構文拡張は
Phase 3 MVP に含めません。

## 4.4 Parser と notation

infix notation を扱うには Pratt parser または precedence climbing が向いています。

たとえば：

```npa
a + b * c = d
```

が、優先順位によって：

```text
Eq (Nat.add a (Nat.mul b c)) d
```

のように解析されます。

Phase 3 での推奨：

```text
- Pratt parser を使う
- notation table は parser に渡す
- ただし型によるnotation解決は elaborator に任せる
```

notation declaration は固定構文なので、notation table がなくても parser が読めます。
module は上から順に処理し、ある notation declaration はそれ以降の term parsing にだけ効きます。
import された notation metadata は elaboration の利便性のために使いますが、certificate の
trusted payload には入りません。

## 4.5 Overloaded notation

`+` は複数の意味を持ちます。

```text
Nat.add
Int.add
Rat.add
Group.add
```

Phase 3 では、typeclass ではなく notation table / name resolution が作る有限の overload 候補だけを保持します。
各候補は明示的な `GlobalRef` であり、instance search や代数階層の探索は Phase 9 に回します。

```rust
SurfaceExpr::Notation {
    head: "+",
    args: [a, b],
}
```

を：

```rust
ResolvedExpr::OverloadedApp {
    candidates: [Nat.add, Int.add, Rat.add],
    args: [a, b],
}
```

のようにして、elaboration で期待型から選びます。

例：

```npa
theorem t (n : Nat) : n + Nat.zero = n := ...
```

ここでは `n : Nat` なので、`+` は `Nat.add` に解決できます。

## 4.6 Notation は certificate に残さない

重要です。

```text
source:
  n + Nat.zero = n

certificate:
  Eq Nat (Nat.add n Nat.zero) n
```

certificate には `+` や `=` という notation は残しません。
notation は完全に非信頼層です。

---

# 5. Implicit args

## 5.1 目的

core ではすべての引数が明示的です。

```text
Eq.refl Nat n
```

しかし人間はこう書きたいです。

```npa
Eq.refl n
```

あるいは：

```npa
refl
```

Phase 3 の elaborator は、省略された引数を補います。

## 5.2 BinderInfo

宣言の binder には、elaboration metadata として明示性を持たせます。
これは implicit argument insertion のための情報であり、canonical core term や certificate hash
には入れません。

```rust
enum BinderInfo {
    Explicit,
    Implicit,
}
```

`StrictImplicit` や `InstanceImplicit` は、将来の高度な elaborator / typeclass 層で扱います。
Phase 3 MVP では `Explicit` と `Implicit` だけです。

```text
Explicit
Implicit
```

例：

```npa
Eq.{u} {A : Sort u} (x : A) (y : A) : Prop
Eq.refl.{u} {A : Sort u} (x : A) : x = x
```

ここで `{A : Sort u}` は implicit です。
import された宣言の BinderInfo は、宣言 hash に紐づく非信頼の source interface metadata として
読む想定です。metadata が間違っていても、最終的に生成された explicit core term は kernel /
checker が型検査します。

## 5.3 Implicit insertion

ユーザーが：

```npa
Eq.refl n
```

と書いたとします。

`Eq.refl` の型は概念的に：

```text
Π {A : Sort u}, Π x : A, Eq A x x
```

なので、elaborator はこう補います。

```text
Eq.refl ?A n
```

そして `n : Nat` から：

```text
?A := Nat
```

を解きます。

最終core：

```text
Const Eq.refl [0] Nat n
```

## 5.4 アルゴリズム

関数適用を elaboration するとき、関数型を WHNF します。

```text
f_ty = Π binder, body
```

次に：

```text
binder が implicit:
  metavar を作って自動挿入

binder が explicit:
  ユーザー引数を消費
```

疑似コード：

```rust
fn elaborate_app(func: SurfaceExpr, args: Vec<SurfaceExpr>, expected: Option<ElabExpr>)
    -> Result<(ElabExpr, ElabExpr)>
{
    let (mut f_core, mut f_ty) = elaborate_infer(func)?;

    let mut remaining_args = args.into_iter();

    loop {
        let f_ty_whnf = whnf(f_ty);

        match f_ty_whnf {
            Pi { binder_info: Implicit, domain, body } => {
                let m = fresh_meta(domain);
                f_core = mk_app(f_core, m);
                f_ty = instantiate(body, m);
            }

            Pi { binder_info: Explicit, domain, body } => {
                let Some(arg) = remaining_args.next() else {
                    break;
                };

                let arg_core = elaborate_check(arg, domain)?;
                f_core = mk_app(f_core, arg_core);
                f_ty = instantiate(body, arg_core);
            }

            _ => break,
        }
    }

    if remaining_args.has_next() {
        return Err(Error::TooManyArguments);
    }

    Ok((f_core, f_ty))
}
```

## 5.5 明示的に implicit を指定する構文

Phase 3 では、明示的に implicit arg を指定できる構文もあると便利です。

```npa
@Eq.refl Nat n
```

`@` を付けると、implicit args を自動挿入しないモードにします。

```text
Eq.refl n
  elaborates to Eq.refl ?A n

@Eq.refl Nat n
  elaborates to Eq.refl Nat n
```

名前付き引数も将来入れるとよいです。

```npa
Eq.refl {A := Nat} n
```

Phase 3 MVPでは `@` だけでも十分です。

---

# 6. Holes

## 6.1 Hole の目的

holes は、未完成のtermを表します。

```npa
theorem add_zero (n : Nat) : n + Nat.zero = n :=
  _
```

または：

```npa
theorem add_zero (n : Nat) : n + Nat.zero = n :=
  ?proof
```

Phase 3では、hole を metavariable に変換します。

```text
_      -> fresh metavariable
?proof -> named metavariable
```

## 6.2 Hole は certificate に入らない

これは非常に重要です。

```text
未解決holeがある theorem は certificate 化できない。
```

開発中は holes を許します。

```text
interactive elaboration:
  holes allowed

certificate generation:
  holes forbidden
```

未解決holeがある場合：

```json
{
  "status": "incomplete",
  "goals": [
    {
      "name": "?proof",
      "context": [
        {"name": "n", "type": "Nat"}
      ],
      "target": "n + Nat.zero = n"
    }
  ]
}
```

## 6.3 Metavariable

hole は内部的に metavariable です。

```rust
struct MetaVar {
    id: MetaVarId,
    name: Option<NameId>,
    context: LocalContextSnapshot,
    ty: ElabExpr,
    assignment: Option<ElabExpr>,
    kind: MetaVarKind,
    span: Span,
}

enum MetaVarKind {
    UserHole,
    SyntheticImplicit,
    UniverseMeta,
}
```

`ElabExpr` は elaborator 内部だけの表現です。Phase 1 の `CoreExpr` と同じ構造を基本にしますが、
metavariable 参照を含められます。certificate 生成前には、すべての `ElabExpr` を
metavariable なしの `CoreExpr` に lower できなければいけません。

`context` をsnapshotとして持つのが重要です。

例：

```npa
theorem t (n : Nat) : n = n := _
```

このholeのgoalは：

```text
n : Nat
⊢ n = n
```

です。

## 6.4 Named holes

同じ名前のholeは同じ metavariable にするか、別々にするかを決めます。

推奨：

```text
?m は同じscope内では同じ metavariable
_ は毎回新しい metavariable
```

例：

```npa
(?m, ?m)
```

は両方同じ値を要求します。

```npa
(_, _)
```

は別々のholeです。

## 6.5 Hole 表示

IDE/API には structured goal として返します。

```json
{
  "hole": "?proof",
  "context": [
    {
      "name": "n",
      "type": "Nat"
    }
  ],
  "target": "Eq Nat (Nat.add n Nat.zero) n",
  "source_span": {
    "line": 3,
    "column": 3
  }
}
```

これが Phase 4 の tactic 実行APIにつながります。

---

# 7. Simple elaboration

## 7.1 Elaboration の責任

elaboration は、surface/resolved expression を `ElabExpr` に変換し、すべての metavariable が
解けた場合だけ canonical `CoreExpr` へ lower します。

やること：

```text
- 名前解決結果を core Const / BVar に変換
- notation を core application に展開
- implicit args を挿入
- holes を metavariable に変換
- 期待型を使って型を推論・検査
- simple unification で metavariable を解く
- universe metavariable を解く
```

やらないこと：

```text
- 複雑な typeclass 探索
- tactic 実行
- AI補完
- 高度な coercion
- overloaded algebraic hierarchy の完全解決
```

Phase 3 は「simple elaboration」です。
便利すぎる機能は Phase 4 以降に回します。

## 7.2 Bidirectional elaboration

elaboration は双方向にします。

```text
infer mode:
  式から型を推論する

check mode:
  期待型に対して式を検査する
```

API：

```rust
fn elab_infer(expr: SurfaceExpr) -> Result<(ElabExpr, ElabExpr)>;

fn elab_check(expr: SurfaceExpr, expected: ElabExpr) -> Result<ElabExpr>;
```

例：

```npa
fun x => x
```

これは期待型なしでは、`x` の型が分かりません。

```npa
fun x => x
```

を elaboration するには期待型が必要です。

```text
expected:
  forall (x : Nat), Nat
```

なら：

```text
fun x : Nat => x
```

にできます。

したがって：

```text
lambda は check mode が基本
application は infer mode が基本
```

## 7.3 Elaboration の基本規則

### Sort

```npa
Prop
Type
Type 0
Sort u
```

は core の `Sort` に変換します。

```text
Prop   -> Sort 0
Type   -> Sort 1
Type 0 -> Sort 1
Type 1 -> Sort 2
```

### Identifier

識別子は名前解決済みのものを使います。

```text
local x
  -> BVar i

global Nat.add
  -> Const Nat.add levels
```

global declaration が universe polymorphic なら、universe metavariable を作ります。

```text
id.{?u}
```

`?u` は source 構文ではなく、elaborator 内部の universe metavariable です。

### Application

```npa
f a
```

は：

```text
elab f
whnf type(f)
implicit args insertion
check a against domain
return instantiated codomain
```

### Lambda

```npa
fun x => body
```

期待型が：

```text
Π x : A, B
```

なら：

```text
Lam A body_core
```

を作ります。

期待型がない場合、binder annotation が必要です。

```npa
fun (x : Nat) => x
```

この場合は型を推論できます。

### Pi / forall

```npa
forall (x : A), B
```

は：

```text
Pi A B
```

に変換します。

`A : Sort u` と `B : Sort v` を確認し、`Sort (imax u v)` を返します。

### Let

```npa
let x : A := v in body
```

は：

```text
Let A v body
```

に変換します。

型注釈がない場合：

```npa
let x := v in body
```

では `v` の型を推論します。

### Annotation

```npa
(t : T)
```

は：

```text
elab T as type
check t against T
return t_core : T_core
```

## 7.4 Metavariable solving

implicit args や holes によって metavariable が発生します。

例：

```npa
Eq.refl n
```

生成：

```text
Eq.refl ?A n
```

制約：

```text
n : ?A
```

もし `n : Nat` なら：

```text
?A := Nat
```

simple unification でこれを解きます。

Phase 3 の unification は保守的でよいです。

対応するもの：

```text
- ?m := term
- definitional equality を使った型一致
- first-order application の簡単な一致
- universe metavariable の解決
```

対応しないもの：

```text
- higher-order pattern unification の完全版
- typeclass 探索
- coercion 探索
- backtracking-heavy overload resolution
```

## 7.5 Expected type propagation

期待型をうまく使うと、implicit args と notation が解けやすくなります。

例：

```npa
theorem t (n : Nat) : n = n :=
  Eq.refl n
```

期待型：

```text
Eq Nat n n
```

右辺：

```text
Eq.refl ?A n
```

期待型から：

```text
?A := Nat
```

を解けます。

## 7.6 Overload resolution

notation や overloaded name は、期待型と引数型から単純に解決します。
typeclass search や backtracking-heavy な探索は Phase 3 では使いません。

例：

```npa
n + Nat.zero
```

候補：

```text
Nat.add
Int.add
Rat.add
```

もし `n : Nat` なら：

```text
Nat.add
```

を選びます。

Phase 3 では単純な戦略にします。

```text
1. 各候補を試す
2. 型チェックに成功する候補を集める
3. 0個ならエラー
4. 1個なら採用
5. 複数なら ambiguity error
```

複数候補が残った場合：

```text
ambiguous notation `+`
candidates:
  Nat.add
  Int.add
  Rat.add
hint:
  add type annotation
```

---

# 8. Declaration elaboration

## 8.1 def

表層：

```npa
def id {A : Type} (x : A) : A := x
```

elaboration 後：

```text
name:
  id

universe params:
  inferred or explicit

type:
  Π {A : Sort 1}, Π x : A, A

value:
  λ A : Sort 1, λ x : A, x

reducibility:
  reducible
```

ここで `{A : Type}` は implicit binder です。
core term では binder info を消し、普通の `Pi` / `Lam` にします。BinderInfo は source interface
metadata として保持してよいですが、canonical certificate payload や hash には入れません。

## 8.2 theorem

表層：

```npa
theorem self_eq (n : Nat) : n = n :=
  Eq.refl n
```

elaboration：

```text
type:
  Π n : Nat, Eq Nat n n

proof:
  λ n : Nat, Eq.refl Nat n
```

kernel は：

```text
proof : type
```

を検査します。

## 8.3 axiom

表層：

```npa
axiom funext :
  ...
```

axiom は type だけ elaboration します。

```text
value/proof はない
axiom report に出る
```

高信頼モードでは allowlist なしの axiom は拒否します。

## 8.4 simple inductive

表層：

```npa
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
```

elaboration 後は、Phase 1/2 の `InductiveDecl` に落とします。

```text
name:
  Nat

universe params:
  inferred or explicit

params / indices:
  declaration binders を params にする
  `:` の後の型が `forall` telescope なら、その leading binders を indices にする

sort:
  Type

constructors:
  Nat.zero : Nat
  Nat.succ : forall (n : Nat), Nat
```

Phase 3 の責任は、constructor type を elaboration し、未解決 metavariable / hole がない
`InductiveDecl` を作るところまでです。positivity check、recursor / computation rule の生成、
constructor result の厳密な検査は Phase 1 kernel と Phase 2 checker が行います。

たとえば Eq のような indexed family は、result type 側の `forall` を index telescope として
扱います。

```npa
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a
```

MVP では core-spec v0.1 の simple inductive に限定します。Nat / Eq 型の単純な indices は
扱いますが、Phase 9 で扱うような高度な indexed family、mutual / nested / coinductive、
pattern matching elaboration は Phase 3 の対象外です。

## 8.5 namespace

```npa
namespace Nat
def double ...
end Nat
```

は名前を：

```text
Nat.double
```

として登録します。

namespace は core term には現れません。

---

# 9. Phase 3 のエラー設計

表層言語で重要なのは、良いエラーを出すことです。

## 9.1 Parser error

```text
expected `)` but found `:=`
```

source span を必ず出します。

## 9.2 Unresolved name

```text
unknown identifier `addz`
```

候補を出します。

```text
did you mean:
  Nat.add
  Int.add
```

## 9.3 Ambiguous name

```text
ambiguous name `add`
candidates:
  Nat.add
  Int.add
hint:
  use `Nat.add` or add a type annotation
```

## 9.4 Type mismatch

```text
type mismatch
  expected: Nat
  actual:   Prop
```

## 9.5 Unsolved implicit

```text
could not infer implicit argument `A`
in:
  Eq.refl ?A ?x
```

## 9.6 Unsolved hole

```text
unsolved hole `?proof`

context:
  n : Nat

target:
  n + Nat.zero = n
```

この structured goal が Phase 4 の tactic に渡されます。

---

# 10. Phase 3 の最小 API

## 10.1 parse

```json
{
  "source": "theorem t (n : Nat) : n = n := Eq.refl n"
}
```

response:

```json
{
  "status": "ok",
  "surface_ast_id": "ast_123"
}
```

## 10.2 resolve

```json
{
  "surface_ast_id": "ast_123"
}
```

response:

```json
{
  "status": "ok",
  "resolved_names": [
    {
      "surface": "Nat",
      "resolved": "Std.Nat.Nat"
    },
    {
      "surface": "Eq.refl",
      "resolved": "Std.Logic.Eq.refl"
    }
  ]
}
```

## 10.3 elaborate

```json
{
  "source": "theorem t (n : Nat) : n = n := _"
}
```

response:

```json
{
  "status": "incomplete",
  "core_type": "Pi Nat (Eq Nat (BVar 0) (BVar 0))",
  "goals": [
    {
      "name": "_",
      "context": [
        {
          "name": "n",
          "type": "Nat"
        }
      ],
      "target": "n = n"
    }
  ]
}
```

## 10.4 elaborate for certificate

```json
{
  "source": "theorem t (n : Nat) : n = n := Eq.refl n",
  "require_complete": true
}
```

response:

```json
{
  "status": "ok",
  "core_declaration": {
    "kind": "theorem",
    "name": "t",
    "type_hash": "sha256:...",
    "proof_hash": "sha256:..."
  }
}
```

---

# 11. Phase 3 の実装順序

おすすめ順はこれです。

```text
1. Lexer
   identifiers, keywords, symbols, string literals, spans

2. Basic parser
   import/open/namespace/end/notation
   def/theorem/axiom/simple inductive
   identifiers, application, lambda, Pi, let, annotation, holes

3. Name system
   hierarchical names
   namespace stack
   imports
   local context

4. Name resolution
   local/global/namespace/open
   ambiguity detection

5. Notation table
   infix/prefix/postfix
   top-to-bottom scope
   precedence climbing or Pratt parser

6. BinderInfo
   explicit / implicit binders

7. Simple elaborator
   infer/check
   const/local/app/lambda/pi/let/annotation

8. Metavariables
   holes
   implicit args
   universe metas

9. Simple unification
   assignment
   occurs check
   definitional equality

10. Declaration elaboration
   def/theorem/axiom/simple inductive

11. Kernel handoff
   core declaration を Phase 1 kernel に渡す

12. Certificate handoff
   fully solved core declaration を Phase 2 certificate に渡す
```

---

# 12. Phase 3 のテスト例

## 12.1 明示的 id

```npa
def id_explicit (A : Type) (x : A) : A :=
  x
```

確認すること：

```text
parser
binder
lambda生成
local name解決
core変換
```

## 12.2 implicit id

```npa
def id {A : Type} (x : A) : A :=
  x
```

確認すること：

```text
implicit binder
source interface metadata
```

## 12.3 implicit argument 補完

```npa
theorem refl_nat (n : Nat) : n = n :=
  Eq.refl n
```

確認すること：

```text
Eq.refl ?A n
?A := Nat
```

## 12.4 notation

```npa
theorem refl_add (n : Nat) : n + Nat.zero = n + Nat.zero :=
  Eq.refl (n + Nat.zero)
```

確認すること：

```text
+ が Nat.add に解決される
= が Eq に展開される
```

## 12.5 hole

```npa
theorem hole_test (n : Nat) : n = n :=
  _
```

期待結果：

```text
status: incomplete
goal:
  n : Nat
  ⊢ n = n
```

## 12.6 let

```npa
def let_test (n : Nat) : Nat :=
  let x : Nat := n in x
```

確認すること：

```text
Let core
ζ-reduction
```

## 12.7 ambiguity

```npa
theorem bad (x : _) : x + x = x + x :=
  Eq.refl (x + x)
```

期待結果：

```text
could not infer type of x
or ambiguous notation `+`
```

## 12.8 simple inductive

```npa
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
```

確認すること：

```text
constructor type elaboration
InductiveDecl 生成
kernel handoff
```

---

# 13. Phase 3 でまだ入れないもの

MVPを小さく保つため、次は後回しにします。

```text
- full typeclass resolution
- coercion search
- macro system
- syntax extensions by users
- tactic blocks
- pattern matching elaboration
- do notation
- structure projection notation
- term-level numeric literals / overloaded numerals
- aliases
- termination checking
- mutual declarations
- sophisticated universe minimization
```

特に typeclass と coercion は強力ですが、elaborator を一気に複雑にします。
Phase 3 では、**「明示的に書けば通る。簡単な省略なら補える」** 程度を目標にするのが安全です。

---

# 14. Phase 3 の完了条件

Phase 3 が完了したと言える条件はこれです。

```text
- import/open/namespace/end を parse できる
- def/theorem/axiom/simple inductive を parse できる
- namespace 付き名前を扱える
- local/global name resolution ができる
- notation declaration を上から順に反映できる
- simple infix notation を扱える
- explicit/implicit binder を扱える
- implicit args を metavariable として挿入できる
- `_` と `?m` を hole goal に変換できる
- bidirectional elaboration が動く
- simple unification で Eq.refl n の型を補える
- unresolved hole がある場合は certificate 化を拒否できる
- solved term は canonical core AST に変換できる
- simple inductive は core-spec v0.1 の `InductiveDecl` に変換できる
- Phase 1 kernel で検査できる
- Phase 2 certificate に渡せる
```

---

# 15. 一文でまとめると

Phase 3 は、**人間が書く便利な構文を、kernelが理解できる完全明示core termへ変換する非信頼層**です。

その中核は：

```text
parser:
  文字列 → surface AST

names:
  識別子 → local/global reference

notation:
  +, =, -> など → core constant application

implicit args:
  省略された引数 → metavariable → unificationで解決

holes:
  未完成部分 → proof goal

simple elaboration:
  surface AST + expected type → fully explicit core AST
```

Phase 3 では「便利さ」を入れ始めますが、まだ無理に賢くしません。
まずは **小さく、決定的で、kernelに渡せるcore termを確実に作る elaborator** を目指すのがよいです。
