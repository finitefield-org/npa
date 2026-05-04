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
  ↓ parser with active notation table
surface AST with notation nodes
  ↓ name resolution + notation candidate collection
resolved AST with overloaded names / notation candidates
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

ここでいう `elaboration AST` は、core term と同じ論理構造を持ちますが、elaboration 中だけ
metavariable と非信頼 metadata を含められる一時表現です。たとえば implicit insertion のために
`Pi` spine へ `BinderInfo` を添付してよいです。Phase 1/2 に渡す `fully explicit core AST` には
metavariable、hole、notation、implicit argument metadata を一切残しません。

この文書の例では、読みやすさのために universe argument を省略して表示することがあります。
たとえば `Eq Nat n n` は説明用表示であり、fully explicit core / certificate では
`Eq.{1} Nat n n` のように universe level が必ず明示されます。

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
  non-dependent arrow
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
    "(" ident+ ":" term ")"
  | "{" ident+ ":" term "}"

lambda_binder ::=
    decl_binder
  | ident
  | "_"

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
  | term "->" term
  | term "→" term
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

`import` は module の先頭にだけ書けます。最初の非 import item が出た後の `import` は
`ImportAfterItem` として拒否します。理由は、import が notation table と global scope を変えるため、
source の途中で import を許すと parser state と certificate import table の対応が複雑になるからです。
`namespace`、`open`、`notation` は通常 item として上から順に処理し、それ以降の item にだけ効きます。

MVP では、declaration binder と `forall` binder は型注釈必須です。`fun x => ...` のような
未注釈 lambda binder だけは、期待型がある check mode で補います。数値リテラルや
typeclass-driven overload は Phase 3 では扱わず、自然数のゼロは `Nat.zero` か開いた namespace
内の `zero` として書きます。

lambda binder 位置の `_` は anonymous binder であり、term hole ではありません。
`fun _ => body` は、期待型から domain を得て local context に display name `_` の binder を
追加しますが、その binder は名前では参照できません。body 内で値を使う場合は `fun x => ...`
のように名前を付けます。

`A -> B` と `A → B` は組み込み構文として右結合に parse し、anonymous binder の
`Pi _ : A, B` に desugar します。これは notation table で解決するユーザー notation では
ありません。関数型は core では常に `Pi` です。`->` と `→` は予約済みで、ユーザー notation
として再定義できません。

term parser の優先順位は MVP では固定します。

```text
highest
  atom / parenthesized term / identifier universe args
  explicit application, left associative
  prefix notation
  postfix notation
  infix notation, notation table の precedence と associativity
  type annotation `:`, non-associative
  arrow `->` / `→`, right associative
  forall / lambda / let body
lowest
```

同じ active scope にある同一 symbol の notation は、kind / precedence / associativity が
一致している場合だけ overload 候補として共存できます。どれかが違う場合は parser の意味が
scope に依存して不安定になるため、`NotationConflict` として拒否します。

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

enum SurfaceBinderKind {
    Named(SurfaceName),
    Anonymous,
}

struct SurfaceBinder {
    kind: SurfaceBinderKind,
    ty: Option<Box<SurfaceExpr>>,
    binder_info: BinderInfo,
    span: Span,
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
`Explicit` の head では implicit term args を自動挿入せず、ユーザーが positional argument として
書いた term を左から順に消費します。これは application head の mode であり、binder の
`BinderInfo::Explicit` とは別物です。

`SurfaceBinder` は declaration binder、lambda binder、Pi binder に共通で使います。
`BinderInfo` は後述の implicit argument metadata と同じものです。

```text
(x : A)  -> kind = Named(x), ty = Some(A), binder_info = Explicit
{x : A}  -> kind = Named(x), ty = Some(A), binder_info = Implicit
x        -> kind = Named(x), ty = None,    binder_info = Explicit
_        -> kind = Anonymous, ty = None,   binder_info = Explicit
```

`(x y : A)` と `{x y : A}` は parser が左から順に複数の `SurfaceBinder` へ展開します。
このグループ内の型注釈 `A` は、グループで導入される `x` / `y` を scope に入れる前の
context で elaboration します。つまり `(x y : A)` は `(x : A) (y : A)` と同じ意味ですが、
`A` の中で `x` や `y` を参照することはできません。依存したい場合は
`(x : A) (y : B x)` のように binder を分けて書きます。

ただし declaration binder と `forall` / Pi binder では `ty = None` を禁止します。`fun x => ...`
と `fun _ => ...` だけが annotation なし binder を持てます。`A -> B` / `A → B` は parser が
`SurfaceExpr::Pi` に desugar し、`kind = Anonymous`, `ty = Some(A)`, `body = B` として表します。

lambda binder に `{x : A}` を書いた場合、infer mode では推論される Pi spine の BinderInfo に
`Implicit` を付けます。check mode では expected Pi の BinderInfo を優先し、source binder の
BinderInfo と違う場合は warning か `BinderInfoMismatch` にできます。MVP では warning に留め、
core lowering ではどちらも同じ `Lam` になります。

`Prop`、`Type`、`Type u` は parser で `SurfaceExpr::Sort` に正規化してよいです。

```text
Prop   -> Sort 0
Type   -> Sort 1
Type u -> Sort (succ u)
```

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

`SurfaceExpr::Notation` は parser が precedence / associativity を確定したことだけを表します。
その時点では target constant が1つに決まっていなくてよいです。resolver は active notation entries
から `ElabGlobalRef` 候補を attach し、elaborator が型情報で候補を1つに絞ります。

## 2.4 Front-end state と処理順

Phase 3 の parser / resolver / elaborator は、module を上から順に処理します。
これは notation と namespace が後続の source にだけ影響するようにするためです。

実装では、少なくとも次の非信頼 state を持ちます。

```rust
struct FrontendState {
    current_module: ModuleName,
    namespace_stack: Vec<Name>,
    open_scopes: Vec<OpenScope>,
    globals: GlobalScope,
    locals: LocalScopeStack,
    notation_table: NotationTable,
    source_interfaces: SourceInterfaceStore,
}
```

`current_module` は source file の中で推論せず、compile request または package manifest が
与えます。`import` は module name を source interface / verified export に解決する
非信頼な処理です。kernel は import 解決を行いません。

source の `import M` は module name だけを書きます。Phase 3 は compile request の
`verified_imports` から `M` の export interface と `export_hash` を見つけます。見つからない、
または同じ module name に複数の hash が与えられている場合は `ImportResolutionError` です。
`.npa` source 内に hash literal を書く構文は MVP には入れません。

同じ module を同じ hash で複数回 import した場合は warning を出し、1つに正規化してよいです。
同じ module name を違う hash で import しようとした場合は `ImportResolutionError` です。

1つの module は次の順に読むことを MVP の仕様にします。

```text
1. import を処理し、imported declarations / source interface metadata を FrontendState に追加する
2. import 由来の top-level notation metadata を active notation table に追加する
3. item を上から順に parse / resolve / elaborate する
4. namespace / open / notation item は、その item より後の item にだけ効かせる
5. def/theorem/axiom/inductive は kernel check に成功した後、後続 item から参照可能にする
6. module 終了時、complete declarations だけを Phase 2 certificate producer に渡す
```

`source interface metadata` には、表示名、BinderInfo、notation、source span、doc comment などを
入れてよいですが、trusted certificate payload ではありません。metadata が間違っていても、
生成された fully explicit core declaration は Phase 1/2 で再検査されます。

`open N` は、visible declarations / notation の中に prefix `N` が存在する場合だけ受理します。
存在しない namespace を開こうとした場合は `UnknownNamespace` です。空 namespace を先に宣言する
構文は Phase 3 MVP には入れません。

途中で unresolved hole を含む declaration は、interactive mode では goal として返してよいです。
ただしその declaration は complete になるまで後続 declaration の trusted dependency として
登録しません。MVP では、未完成 declaration への後続参照は `IncompleteDependency` として拒否します。

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

term 内の名前解決は、declaration name の qualification とは分けて考えます。
local name は未修飾 identifier だけで参照できます。`Nat.x` のような qualified name が
local binder を指すことはありません。

source に書く `qual_name` は module name ではなく、visible declaration name への参照です。
たとえば module `Std.Nat.Basic` が declaration name `Nat.add` を export している場合、source では
`Std.Nat.Basic.Nat.add` ではなく `Nat.add` と参照します。module name は import 解決と
certificate の import table で使い、term syntax の absolute prefix には使いません。

未修飾名 `add` を解決するときの順序は、次です。

```text
1. local context
2. current namespace + add
3. opened namespaces + add
4. root/imported short name add
```

qualified name `Nat.add` を解決するときは、次です。

```text
1. visible global declaration whose declaration_name is exactly Nat.add
2. current namespace + Nat.add
3. opened namespaces + Nat.add
4. imported declaration whose declaration_name has suffix Nat.add
```

例：

```npa
namespace Nat

def double (n : Nat) : Nat :=
  add n n

end Nat
```

ここで `add` は `Nat.add` に解決されます。

また、`namespace Nat` の中で `Nat.add` と書いた場合は、まず exact global name `Nat.add` を
探します。これにより、namespace 内から同じ namespace の declaration を修飾名で安定して
参照できます。

`open Nat` は現在の namespace scope だけに効く非信頼の elaboration metadata です。
同じ短縮名が複数の opened/imported namespace から来る場合は、勝手に選ばず ambiguity とします。
aliases は Phase 3 MVP には入れません。

各 priority level で候補が1つならそれを採用し、複数なら ambiguity とします。高い priority
で候補が見つかった場合、低い priority の候補は見ません。候補0なら次の priority に進みます。
これにより、local binder は global name を shadow できますが、同じ priority 内の複数候補は
型情報なしに勝手に選びません。

global candidate の中では、current module で既に kernel check 済みの declaration を imported
declaration より優先します。複数 import が同じ declaration name を提供し、current module に
同名 declaration がない場合は ambiguity です。current module declaration が imported declaration
を shadow する場合は warning を出すのが望ましいですが、trusted payload には影響しません。

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
    Global(ElabGlobalRef),
    Overloaded(Vec<ElabGlobalRef>),
    Unresolved(SurfaceName),
}
```

## 3.5 ElabGlobalRef

Phase 2 の certificate と接続するため、global name は名前だけでなく hash 付き参照にします。

```rust
enum ElabGlobalRef {
    Imported {
        module: ModuleName,
        name: NameId,
        decl_interface_hash: Hash,
    },
    Local {
        decl_index: usize,
        name: NameId,
    },
    LocalGenerated {
        decl_index: usize,
        name: NameId,
    },
}
```

これにより：

```text
同じ `Nat.add` という名前だが中身が違う
```

という問題を防げます。

これは source/elaboration 層の参照です。imported declaration は import の `export_hash` に
含まれる `decl_interface_hash` で固定します。current module の declaration は、kernel check
成功後に確定した local declaration index で参照します。inductive constructor / recursor のような
生成 declaration は `LocalGenerated` として source inductive の declaration index に紐づけます。

Phase 2 certificate へ渡す時点では、`Imported` は canonical `GlobalRef::Imported`、
`Local` は `GlobalRef::Local`、`LocalGenerated` は `GlobalRef::LocalGenerated` 相当に変換します。
module 名や表示名だけを trusted payload の根拠にしません。

kernel check 前の inductive head だけは temporary global として扱います。これは constructor type
elaboration のための仮参照であり、constructor type の elaboration が終わったら必ず kernel に
`InductiveDecl` 全体を渡して検査し、成功した場合だけ通常の `Local` / `LocalGenerated` に置き換えます。

## 3.6 Scope の細則

MVP では scope の動きを次のように固定します。

```text
namespace N:
  namespace_stack に N を push する

end N:
  namespace_stack の末尾が N なら pop する
  N が省略された場合は末尾を pop する
  末尾と一致しない場合は NamespaceMismatch

open N:
  現在の lexical scope に N を追加する
```

`open` は namespace block の外へ漏れません。実装上は `namespace` に入るたびに
open scope frame を push し、対応する `end` で pop します。top-level の `open` は
module 末尾まで有効です。

`open` は names だけでなく、source interface metadata として export/import された notation にも
効きます。`open Nat` 後は `Nat` namespace に属する notation entry も active notation table に
入ります。同じ symbol の conflict rule はここでも適用します。

declaration name は次で決まります。

```text
module_name = current_module
declaration_name = namespace_stack + declaration_name
```

source 上ですでに修飾名を書いた場合も、MVP では current namespace からの相対名として扱います。
たとえば module `Std.Nat.Basic` の `namespace Nat` 内で `def Extra.double` と書くと、module は
`Std.Nat.Basic`、declaration name は `Nat.Extra.double` です。absolute name 構文は
Phase 3 MVP には入れません。

同一 module 内で同じ global declaration name を2回定義することは禁止です。simple inductive が
生成する constructor / recursor 名も同じ global scope に登録されるため、source declaration と
衝突した場合は `DuplicateDeclaration` として拒否します。

local binder の shadowing は de Bruijn 表現では安全なので許可します。解決は nearest binder wins です。
ただし global declaration または外側の local binder を shadow した場合、IDE/API には warning を
返すのが望ましいです。warning は trusted payload に入りません。

notation declaration は、宣言時の current namespace を `namespace` metadata として持ちます。
current file では、その declaration より後、かつ現在の lexical namespace frame の内側で active
です。top-level notation は module 末尾まで active です。import された namespaced notation は、
対応する namespace に入っている時、または `open` された時だけ active にします。

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
    Global(ElabGlobalRef),
}
```

Phase 3 では `ElabGlobalRef` への展開だけを許します。syntax macro やユーザー定義構文拡張は
Phase 3 MVP に含めません。

notation declaration の target `qual_name` は、その declaration を処理する時点で解決します。
未定義 target や overloaded target は拒否します。つまり notation は forward declaration できません。
current module の既存 declaration または import 済み declaration だけを target にできます。
local binder や temporary global は notation target にできません。

`prefix` と `postfix` の associativity は常に `NonAssoc` とします。`infix` は `NonAssoc`、
`infixl` は `Left`、`infixr` は `Right` です。

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

prefix / postfix / infix は、すべて notation entry の `precedence` を binding power として使います。
同じ precedence の non-associative infix を連鎖させる場合は parse error にします。

```npa
a = b = c
```

は、`=` が `infix:50` なら `ParserError` です。必要なら括弧を書きます。

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
各候補は明示的な `ElabGlobalRef` であり、instance search や代数階層の探索は Phase 9 に回します。

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

## 4.7 Notation parsing の決定事項

notation symbol の字句解析は longest match にします。たとえば active symbols に `<` と `<=`
がある場合、入力 `<=` は必ず1つの token として読みます。symbol は UTF-8 文字列として扱い、
hash や core name には使いません。

notation declaration の string は、MVP では trim 後に1つの operator token になるものだけを
許します。つまり `" + "` と `"+"` は同じ symbol `+` です。空白を含む multi-token notation
や mixfix notation は Phase 3 MVP には入れません。

operator token は identifier / keyword ではなく、reserved structural token でもない1 token です。
MVP では `->`, `→`, `:`, `:=`, `=>`, `,`, `.`, `.{`, `(`, `)`, `{`, `}`, `|`, `@`, `_`, `?` を notation symbol
として使うことは禁止します。

notation の処理は独立した trusted expansion pass ではありません。実装上は次の2段階に分けます。

```text
parser:
  precedence / associativity だけを使って SurfaceExpr::Notation を作る

resolver / elaborator:
  notation table と名前解決結果から ElabGlobalRef 候補を作り、型に基づいて1つへ絞る
```

このため、`resolved AST with notation candidates` は「notation が全部 core constant に置換済み」という意味ではなく、
「parser precedence が確定し、elaborator が選べる候補集合になっている」という中間表現として
扱ってよいです。型情報なしで候補が1つに決まる場合だけ、早い段階で `ElabGlobalRef` application に
してもよいです。

候補の試行順は deterministic にします。

```text
1. 表示用 fully qualified declaration name の UTF-8 byte lexicographic order
2. ref kind order: Local, LocalGenerated, Imported
3. Local / LocalGenerated は decl_index と generated name
4. Imported は module name、decl_interface_hash の byte order
```

overload resolution では候補ごとに metavariable store / constraint store を transaction として
分けます。失敗した候補が作った metavariable や制約は、他の候補や最終エラーに漏らしません。
複数候補が成功した場合は、最初の候補を勝手に採用せず ambiguity error にします。

---

# 5. Implicit args

## 5.1 目的

core ではすべての引数が明示的です。

```text
Eq.refl.{1} Nat n
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

実装では、elaborator が扱う関数型 spine に `BinderInfo` を保持します。これは source interface
由来の非信頼 metadata であり、core `Pi` node そのものの一部ではありません。`ElabExpr` に直接
持たせても、型 spine side table として持ってもよいですが、core lowering では必ず消します。

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
Const Eq.refl [1] Nat n
```

## 5.4 アルゴリズム

関数適用を elaboration するとき、関数型を WHNF します。

```text
f_ty = Π binder, body
```

次に：

```text
binder が implicit かつ挿入条件を満たす:
  metavar を作って自動挿入

binder が implicit かつ `@` mode:
  ユーザー引数があれば消費

binder が explicit:
  ユーザー引数を消費
```

疑似コード：

```rust
fn elaborate_app(func: SurfaceExpr, args: Vec<SurfaceExpr>, expected: Option<ElabExpr>)
    -> Result<(ElabExpr, ElabExpr)>
{
    let auto_insert = implicit_insertion_enabled_for_head(&func);
    let (mut f_core, mut f_ty) = elaborate_infer(func)?;
    let mut remaining_args = args.into_iter().peekable();

    loop {
        let f_ty_whnf = whnf(f_ty);

        match f_ty_whnf {
            Pi { binder_info: Implicit, domain, body }
                if auto_insert
                    && (remaining_args.peek().is_some()
                        || expected_needs_implicit_instantiation(&f_ty_whnf, expected.as_ref())) =>
            {
                let m = fresh_meta(domain);
                f_core = mk_app(f_core, m);
                f_ty = instantiate(body, m);
            }

            Pi { binder_info: Implicit, domain, body } if !auto_insert => {
                let Some(arg) = remaining_args.next() else {
                    break;
                };

                let arg_core = elaborate_check(arg, domain)?;
                f_core = mk_app(f_core, arg_core);
                f_ty = instantiate(body, arg_core);
            }

            Pi { binder_info: Implicit, .. } => {
                break;
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

    if remaining_args.peek().is_some() {
        return Err(Error::TooManyArguments);
    }

    Ok((f_core, f_ty))
}
```

`expected_needs_implicit_instantiation` は保守的でよいです。たとえば expected type があり、
implicit binder を1つ以上入れないと expected type と合わない場合だけ true にします。
判定できない場合は false とし、後で `UnsolvedImplicit` または `TypeMismatch` として返します。

`implicit_insertion_enabled_for_head` は、head が `@ident` の場合だけ false を返します。
それ以外の application、notation、parenthesized expression では true です。

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
  elaborates to Eq.refl.{?u} Nat n
  and then ?u := 1
```

`@` が止めるのは implicit **term** argument の自動挿入です。universe argument を省略した場合の
universe metavariable 生成は止めません。完全に明示したい場合は：

```npa
@Eq.refl.{1} Nat n
```

のように universe argument も書きます。

`@` mode では implicit binder も user-visible binder として扱い、source の positional argument が
左から順に消費します。したがって `@Eq.refl Nat n` の `Nat` は implicit binder `{A}` に対応します。

名前付き引数も将来入れるとよいです。

```npa
Eq.refl {A := Nat} n
```

Phase 3 MVPでは `@` だけでも十分です。

## 5.6 Implicit insertion の境界

MVP では implicit insertion を決定的にするため、次の規則にします。

```text
- bare identifier を infer mode で読むだけでは implicit args を挿入しない
- application head として使われ、次のユーザー引数を消費する必要がある場合に implicit args を挿入する
- check mode で expected type がある場合は、expected type に合うところまで implicit args を挿入してよい
- `@` 付き head では、その head の implicit args は一切自動挿入しない
- source で明示された引数の後ろに残る implicit binder は、expected type が要求しない限り挿入しない
```

例：

```npa
Eq.refl
```

を infer mode で読むだけなら、型はまだ：

```text
Π {A : Sort ?u}, Π x : A, Eq A x x
```

として扱います。

```npa
Eq.refl n
```

のように明示引数 `n` が来た時点で、先行する implicit `A` を `?A` として挿入します。

import された declaration に BinderInfo metadata がない場合、MVP ではすべて explicit として扱います。
これは不便ですが保守的です。metadata がある場合でも、最終的な explicit core term が kernel check
されるため、metadata は信頼境界に入りません。

`SyntheticImplicit` metavariable は通常 goal としてユーザーに出しません。declaration を complete
にする時点で残っていれば `UnsolvedImplicit` です。`UserHole` metavariable は interactive mode で
goal として表示してよいですが、certificate 生成時には同じく拒否します。

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

ここで禁止するのは **未解決** の hole です。`_` や `?m` から作った `UserHole` metavariable でも、
elaboration / unification によって一意に代入できたものは goal として残らず、certificate 生成前に
通常の core term へ lower されます。未代入の `UserHole` だけが interactive goal として表示されます。

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
metavariable 参照と BinderInfo などの非信頼 metadata を含められます。certificate 生成前には、すべての `ElabExpr` を
metavariable なしの `CoreExpr` に lower できなければいけません。

実装では universe metavariable を term metavariable と別 store に分けてもよいです。その場合、
上の `UniverseMeta` は「metavariable store 全体に含まれる未解決項目」という概念的な分類です。

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

## 6.6 Named hole の reuse と context

`?m` を同じ metavariable として再利用する単位は、1 declaration の lexical scope 内です。
別 declaration の `?m` は別 metavariable です。

同じ declaration 内でも、同名 hole を違う local context snapshot で使う場合は MVP では拒否します。

```npa
def bad : Nat :=
  let x : Nat := ?m in ?m
```

理由は、metavariable を context abstraction として扱う higher-order な実装を Phase 3 MVP に
入れないためです。

```text
same named hole + same context snapshot:
  同じ MetaVarId を返す

same named hole + different context snapshot:
  NamedHoleContextMismatch
```

context snapshot の比較は、local declarations の長さ、型、local definition の値を core/ElabExpr
構造で比較します。display name や source span の違いは比較に使いません。

metavariable assignment は、その metavariable の context snapshot で型検査できる term だけを
受け付けます。代入時には occurs check を必ず行い、`?m := ... ?m ...` は拒否します。

---

# 7. Simple elaboration

## 7.1 Elaboration の責任

elaboration は、surface/resolved expression を `ElabExpr` に変換し、すべての metavariable が
解けた場合だけ canonical `CoreExpr` へ lower します。

やること：

```text
- 名前解決結果を core Const / BVar に変換
- notation candidate を core constant application に確定
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
- source declaration の再順序化
- forward reference の解決
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
Type u
Sort u
```

は core の `Sort` に変換します。

```text
Prop   -> Sort 0
Type   -> Sort 1
Type 0 -> Sort 1
Type 1 -> Sort 2
Type u -> Sort (succ u)
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

metavariable の自動代入方針は次です。

```text
SyntheticImplicit:
  unification が解けるなら自動代入する
  declaration close 時点で未解決なら UnsolvedImplicit

UniverseMeta:
  universe constraint solving が解けるなら自動代入する
  declaration close 時点で未解決なら UnsolvedUniverseMeta

UserHole:
  expected type や他の制約で一意に解けるなら自動代入してよい
  未解決なら interactive goal として表示し、certificate 生成では UnsolvedHole
```

たとえば `Eq.refl _` を `n = n` に対して check する場合、`_` は `n` に解けるので
goal として残りません。一方、`theorem t : Nat := _` の `_` は値が一意に決まらないため
goal として残ります。

## 7.5 Expected type propagation

期待型をうまく使うと、implicit args と notation が解けやすくなります。

例：

```npa
theorem t (n : Nat) : n = n :=
  Eq.refl n
```

期待型：

```text
Eq.{1} Nat n n
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

Phase 3 MVP では overload resolution のために深い backtracking はしません。候補ごとに
`elab_check` / `elab_infer` を1回ずつ transaction 内で走らせ、結果の unsolved metavariable を
constraint solver で解ける範囲だけ解きます。候補が未解決 `UserHole` を新たに残す場合、その候補は
interactive mode では「成功候補」ではなく保留として扱い、他に完全成功候補がなければ
`AmbiguousNotation` または `UnsolvedHole` を返します。certificate mode では未解決 metavariable を
残す候補は失敗候補です。

## 7.7 Local context と de Bruijn 変換

surface/elaboration 中は local name で文脈を持ってよいですが、core lowering では必ず
de Bruijn index に変換します。

```rust
struct LocalEntry {
    name: NameId,
    ty: ElabExpr,
    value: Option<ElabExpr>,
    binder_info: BinderInfo,
    span: Span,
}
```

lookup は末尾から検索します。最も内側の binder が `BVar 0` です。

```text
context:
  A : Type
  x : A

surface `x` -> BVar 0
surface `A` -> BVar 1
```

declaration binder を閉じる時は、source order と同じ順で `Pi` / `Lam` を作ります。

```npa
def id (A : Type) (x : A) : A := x
```

は：

```text
type  = Pi A : Sort 1, Pi x : BVar 0, BVar 1
value = Lam A : Sort 1, Lam x : BVar 0, BVar 0
```

になります。BinderInfo は source interface metadata にだけ残り、core `Pi` / `Lam` には入りません。

## 7.8 Constraint store

elaboration は即座にすべてを解こうとせず、制約を store に積んでよいです。

```rust
enum Constraint {
    IsType(ElabExpr),
    TypeEq { lhs: ElabExpr, rhs: ElabExpr },
    TermEq { ty: ElabExpr, lhs: ElabExpr, rhs: ElabExpr },
    LevelEq { lhs: ElabLevel, rhs: ElabLevel },
    LevelLe { lhs: ElabLevel, rhs: ElabLevel },
}
```

constraint solving は deterministic な worklist として実装します。順序は source span、生成順の
stable id、constraint kind の順に固定します。resource limit に達した場合は、complete declaration
としては reject します。

overload candidate の試行、notation candidate の試行、expected type による分岐は transaction を
使います。commit されなかった transaction の metavariable assignment / constraint / warning は
捨てます。

## 7.9 Universe parameter policy

Phase 3 MVP では、declaration-level universe polymorphism は明示的な `.{u, v}` を基本にします。

```npa
def id.{u} {A : Sort u} (x : A) : A := x
```

明示された universe parameter だけが、その declaration の `universe_params` に入ります。
source に現れる `Sort u` や `Type u` の `u` が declaration の universe params に入っていなければ
`UnknownUniverseParam` です。

universe parameter の namespace は term/local/global name の namespace と分けます。同じ declaration
内で universe parameter 名を重複させることは禁止です。term binder が同じ表示名を持つことは
de Bruijn 的には可能ですが、MVP では混乱を避けるため warning を出すのが望ましいです。

`Type` は `Sort 1`、`Type 0` も `Sort 1` です。したがって：

```npa
def id {A : Type} (x : A) : A := x
```

は universe polymorphic ではなく、`A : Sort 1` の monomorphic declaration です。

polymorphic constant を使うときには、elaborator 内部で universe metavariable を作ってよいです。

```npa
Eq.refl n
```

では `Eq.refl.{?u} ?A n` から始め、`n : Nat` と `Nat : Sort 1` により `?u := 1`,
`?A := Nat` を解きます。

declaration close 時点で未解決の universe metavariable が残る場合、MVP では自動 generalization
しません。`UnsolvedUniverseMeta` とし、ユーザーに `.{u}` と `Sort u` の明示を要求します。
将来、Phase 9 で universe minimization / generalization を強化します。

## 7.10 Core lowering の条件

`ElabExpr` を `CoreExpr` に lower できる条件は次です。

```text
- UserHole / SyntheticImplicit / UniverseMeta が残っていない
- OverloadedRef / OverloadedApp が残っていない
- local reference はすべて de Bruijn index に変換済み
- global reference は ElabGlobalRef と universe args が確定済み
- source-only BinderInfo / Span / notation head は除去済み
- generated core declaration を Phase 1 kernel が受理する
```

これに失敗した declaration は `.npcert` に出しません。interactive API は incomplete/error と
structured diagnostics を返します。

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
  明示 .{...} がなければ []; MVP では未解決 universe meta を自動 generalize しない

type:
  Π {A : Sort 1}, Π x : A, A

value:
  λ A : Sort 1, λ x : A, x

reducibility:
  reducible
```

ここで `{A : Type}` は implicit binder です。
上の `Π {A : Sort 1}` は source interface view です。core term では binder info を消し、
普通の `Pi A : Sort 1, ...` / `Lam A : Sort 1, ...` にします。BinderInfo は source interface
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
  Π n : Nat, Eq.{1} Nat n n

proof:
  λ n : Nat, Eq.refl.{1} Nat n
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
  明示 .{...} がなければ []; MVP では未解決 universe meta を自動 generalize しない

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

## 8.6 def/theorem/axiom の elaboration 手順

`def` と `theorem` は、declaration binder を local context に入れて body を check し、
最後に binder を閉じて closed core declaration を作ります。

```text
1. declaration name を current namespace で qualify する
2. universe params `.{...}` を declaration universe context に登録する
3. duplicate declaration name でないことを確認する
4. この declaration 自身を global scope に入れないまま、declaration binders を左から右に elaboration し、local context に追加する
5. `:` の後の result type を local context で elaboration し、Sort に住むことを確認する
6. declaration type を binders の source order で Pi に閉じる
7. body/proof を result type に対して check mode で elaboration する
8. body/proof を binders の source order で Lam に閉じる
9. metavariable / overload / universe meta が残っていないことを確認する
10. Phase 1 kernel に closed declaration を渡して検査する
11. 成功した declaration だけを後続 item の global scope に登録する
```

`axiom` は 1〜6 と 9〜11 だけを行い、value/proof は持ちません。

Phase 3 MVP では self reference と forward reference を禁止します。つまり、`def f := f` のように
自分自身を source elaboration 中に参照することはできません。再帰は source-level recursive
definition ではなく、Phase 1 の recursor constant を明示的に使う形へ elaboration します。

`def` の reducibility は default で `reducible`、`theorem` は opaque として扱います。
source syntax で reducibility annotation を付ける機能は MVP には入れません。

## 8.7 simple inductive の elaboration 手順

simple inductive は、constructor type から inductive 自身を参照できる必要があります。
そのため Phase 3 では、kernel check 前の temporary global を使います。

```text
1. inductive name を current namespace で qualify する
2. universe params を登録する
3. inductive / constructor / generated recursor の予定名が duplicate でないことを確認する
4. declaration binders を params telescope として elaboration する
5. `:` の後を elaboration し、leading forall telescope を indices、末尾 Sort を sort として読む
6. temporary global `I` を env に追加する
7. 各 constructor type を params context の下で elaboration する
8. constructor type を params で閉じた closed constructor type にする
9. InductiveDecl を作り、Phase 1 kernel に渡す
10. kernel check 成功後、I / constructors / recursor を後続 item の global scope に登録する
```

temporary global `I` の型は、params / indices を telescope として持つ inductive head type です。
概念的には：

```text
Nat : Sort 1
Eq  : Pi A : Sort u, Pi a : A, Pi b : A, Sort 0
```

です。ただし elaboration 中の temporary global には、inductive declaration binder 由来の
BinderInfo も source interface metadata として付けます。上の Eq では `A` は implicit、`a` と
index `b` は explicit として扱います。これにより constructor type 内の `Eq.{u} a a` のような
表層参照は、implicit param `A` を補った `Eq.{u} A a a` へ elaboration されます。

この BinderInfo は source/elaboration metadata であり、kernel に渡す `InductiveDecl.params` /
`indices` には入りません。

手順 5 の読み方は次です。

```text
inductive Nat : Type
  params  = []
  indices = []
  sort    = Sort 1

inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop
  params  = [{A : Sort u}, (a : A)]
  indices = [(b : A)]
  sort    = Sort 0
```

ここで leading telescope は、elaboration 後に WHNF した結果の先頭 `Pi` 列です。`->` / `→` は
anonymous binder の `Pi` に desugar されるため index telescope として扱えます。末尾が `Sort s`
でなければ `ExpectedSort` です。MVP では result type の途中に let や reducible definition を
挟んで telescope を隠す書き方は、WHNF で明らかに `Pi ... Sort` になる場合だけ受理します。

constructor source は params を local name として参照できますが、indices は constructor type の
return type 側で具体的に与えます。

```npa
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a
```

`refl` の source type は params context `{A}, a` の下で elaboration し、closed constructor type は：

```text
Pi A : Sort u, Pi a : A, Eq.{u} A a a
```

になります。

constructor name は、未修飾なら inductive name の namespace に入れます。

```text
inductive Nat ... where
| zero : Nat
| succ : Nat -> Nat

generated global names:
  Nat
  Nat.zero
  Nat.succ
  Nat.rec
```

constructor name が source 上で修飾されている場合も、MVP では inductive name からの相対名として
扱います。`inductive Nat ... | Extra.zero : Nat` は `Nat.Extra.zero` を生成します。constructor を
inductive namespace の外へ置く absolute name 構文はありません。

MVP では constructor type elaboration 中に同じ inductive block の他 constructor や recursor を
参照することは禁止します。参照できるのは、既存の imported/local globals、params、constructor
type 内で導入した binders、temporary global の inductive head だけです。

recursor は source syntax から elaboration しません。kernel が生成または検査した recursor を、
Phase 3 の global scope / source interface に generated declaration として追加します。Phase 2
certificate では `GlobalRef::LocalGenerated` 相当の参照になります。

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

## 9.7 実装で固定する DiagnosticKind

API とテストでは、少なくとも次の構造化 diagnostic kind を区別します。
hard error と warning は `severity` で分けます。certificate generation では hard error が1つでも
あれば失敗し、warning は trusted payload に影響しません。

```text
ParserError
ImportResolutionError
ImportAfterItem
NamespaceMismatch
UnknownNamespace
DuplicateDeclaration
DuplicateUniverseParam
InvalidNotation
NotationConflict
UnknownIdentifier
UnknownUniverseParam
AmbiguousName
AmbiguousNotation
TypeMismatch
ExpectedFunctionType
ExpectedSort
BinderInfoMismatch
TooManyArguments
UnsolvedImplicit
UnsolvedUniverseMeta
UnsolvedHole
NamedHoleContextMismatch
OccursCheckFailed
IncompleteDependency
ForwardReference
KernelRejected
ShadowingWarning
DuplicateImportWarning
```

message は人間向けに変えてよいですが、`DiagnosticKind`、severity、primary span、関連候補、expected/actual type は
テスト可能な structured data として返します。

---

# 10. Phase 3 の最小 API

## 10.1 parse

```json
{
  "module": "Scratch",
  "verified_imports": [],
  "source": "theorem t (n : Nat) : n = n := Eq.refl n"
}
```

response:

```json
{
  "status": "ok",
  "surface_ast_id": "ast_123",
  "frontend_state_id": "fe_1"
}
```

`parse` は imported notation metadata と preceding notation declarations を使うため、
`module` と `verified_imports` を受け取ります。notation を全く使わない小さな snippet parser を
別に作ってもよいですが、module parse API では state を明示します。

## 10.2 resolve

```json
{
  "surface_ast_id": "ast_123",
  "frontend_state_id": "fe_1"
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
  "module": "Scratch",
  "verified_imports": [],
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
  "module": "Scratch",
  "verified_imports": [],
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

`module` と `verified_imports` は trusted kernel へ直接渡すものではなく、Phase 3 が
`ElabGlobalRef` と import hash を確定するための入力です。`verified_imports` の各 entry は、
Phase 2 certificate verifier が検査済みの export interface を指します。

---

# 11. Phase 3 の実装マイルストーン

Phase 3 は一度に実装せず、次の小さな milestone に分けます。各 milestone は
「構造化 diagnostic を返す」「テストを追加する」「未解決 hole / meta を certificate に入れない」
ところまでを完了条件にします。

## 11.1 M1: Parser / Surface AST

- [x] Lexer を実装する
- [x] `Span` を全 token / AST node に保存する
- [x] `import` / `open` / `namespace` / `end` を parse する
- [x] `def` / `theorem` / `axiom` / simple `inductive` を parse する
- [x] `fun` / `forall` / `let` / annotation / application / parenthesized term を parse する
- [x] `->` / `→` を右結合の anonymous Pi に desugar する
- [x] grouped binder `(x y : A)` / `{x y : A}` を `SurfaceBinder` list に展開する
- [x] `_` / `?m` を surface hole として保持する
- [x] `@f` を `ImplicitMode::Explicit` として保持する
- [x] `Prop` / `Type` / `Type u` を `SurfaceExpr::Sort` に正規化する
- [x] `import` が module 先頭以外に出た場合は `ImportAfterItem` を返す

## 11.2 M2: FrontendState / Name Resolution

- [ ] `FrontendState` に current module / namespace stack / open scopes を持たせる
- [ ] `verified_imports` から import interface を読み、source 内 import と照合する
- [ ] duplicate import を決定的に扱う
- [ ] namespace / open の lexical scope を実装する
- [ ] local context と global declaration table を分離する
- [ ] current module declaration と imported declaration の優先順位を固定する
- [ ] qualified / unqualified name resolution を実装する
- [ ] ambiguous name を `AmbiguousName` として保持または拒否する
- [ ] forward reference を `ForwardReference` として拒否する

## 11.3 M3: Minimal Elaboration / Kernel Handoff

- [ ] `infer` / `check` の bidirectional elaboration skeleton を実装する
- [ ] local / global / app / lambda / Pi / let / annotation を core term に落とす
- [ ] explicit binder だけで書かれた `def` / `theorem` を elaboration する
- [ ] declaration elaboration 中は自分自身を global env に入れない
- [ ] elaborated core declaration を Phase 1 kernel に渡す
- [ ] kernel が拒否した場合は `KernelRejected` を返す
- [ ] well-typed / ill-typed の最小テストを追加する

## 11.4 M4: Metavariables / Implicit Args / Universe Meta

- [ ] term metavariable と universe metavariable の store を分けて管理する
- [ ] implicit binder に対して `SyntheticImplicit` meta を挿入する
- [ ] `@` mode では implicit term args を自動挿入しない
- [ ] `_` と `?m` を `UserHole` meta に変換する
- [ ] named hole の context snapshot を比較し、違う場合は `NamedHoleContextMismatch` を返す
- [ ] constraint store に `TypeEq` / `TermEq` / `LevelEq` / `LevelLe` を入れる
- [ ] simple unification と occurs check を実装する
- [ ] 未解決 implicit / universe meta / hole が残る declaration は certificate 化を拒否する

## 11.5 M5: Notation / Overload Resolution

- [ ] notation declaration を namespace / open scope と連動させる
- [ ] notation target を declaration 処理時に `ElabGlobalRef` へ解決する
- [ ] prefix / postfix / infix / infixl / infixr を parser binding power に反映する
- [ ] notation conflict を `NotationConflict` として拒否する
- [ ] non-associative infix chain を `ParserError` として拒否する
- [ ] overloaded notation candidates を決定的順序で保持する
- [ ] elaboration 中に transaction / rollback で候補を試す
- [ ] 解決不能な notation は `AmbiguousNotation` として返す

## 11.6 M6: Declaration Coverage / Simple Inductive

- [ ] `def` / `theorem` / `axiom` の certificate handoff を実装する
- [ ] axiom の使用が axiom report に反映されることを確認する
- [ ] simple inductive の temporary global を作る
- [ ] constructor type を temporary global 付き context で elaboration する
- [ ] core-spec v0.1 の `InductiveDecl` に変換する
- [ ] constructor / recursor などの generated declaration を `LocalGenerated` で参照する
- [ ] inductive 全体を kernel に渡し、成功後だけ通常の global env に登録する

## 11.7 M7: Phase 2 Certificate / API / Regression Tests

- [ ] fully solved core declaration を Phase 2 certificate builder に渡す
- [ ] imported declaration を `decl_interface_hash` 付き参照として certificate に入れる
- [ ] certificate hash / import hash が決定的であることをテストする
- [ ] axiom report が意図せず増えないことをテストする
- [ ] `parse` / `resolve` / `elaborate` API を安定させる
- [ ] diagnostic の severity と `DiagnosticKind` を API から返す
- [ ] `cargo fmt --all` を通す
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` を通す
- [ ] `cargo test --workspace` を通す

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

## 12.9 arrow

```npa
def const_nat : Nat -> Nat -> Nat :=
  fun x => fun y => x
```

確認すること：

```text
`->` が右結合の Pi に desugar される
anonymous binder が core で通常の binder として扱われる
```

## 12.10 universe parameter は明示

```npa
def poly_id.{u} {A : Sort u} (x : A) : A :=
  x
```

確認すること：

```text
universe_params = [u]
未宣言の `Sort v` は UnknownUniverseParam
未解決 universe meta は declaration close で UnsolvedUniverseMeta
```

## 12.11 named hole context mismatch

```npa
def bad_named_hole : Nat :=
  let x : Nat := ?m in ?m
```

期待結果：

```text
NamedHoleContextMismatch
```

## 12.12 notation conflict

```npa
infixl:65 " + " => Nat.add
infixr:70 " + " => Other.add
```

期待結果：

```text
NotationConflict
```

## 12.13 import の位置

```npa
def x : Nat := Nat.zero
import Std.Nat.Basic
```

期待結果：

```text
ImportAfterItem
```

## 12.14 non-associative notation chain

```npa
infix:50 " = " => Eq
theorem bad (a : Nat) (b : Nat) (c : Nat) : Prop :=
  a = b = c
```

期待結果：

```text
ParserError
```

## 12.15 grouped binder

```npa
def first (A : Type) (x y : A) : A :=
  x
```

確認すること：

```text
`(x y : A)` が `(x : A) (y : A)` 相当の SurfaceBinder list に展開される
型注釈 `A` は grouped binder 内で導入される `x` / `y` より外側の context で elaboration される
```

---

# 13. Phase 3 でまだ入れないもの

MVPを小さく保つため、次は後回しにします。

```text
- full typeclass resolution
- coercion search
- macro system
- syntax extensions by users
- multi-token / mixfix notation
- tactic blocks
- pattern matching elaboration
- do notation
- structure projection notation
- term-level numeric literals / overloaded numerals
- aliases
- absolute global name syntax
- source-level recursive definition syntax
- reducibility annotations
- termination checking
- mutual declarations
- automatic universe generalization
- sophisticated universe minimization
```

特に typeclass と coercion は強力ですが、elaborator を一気に複雑にします。
Phase 3 では、**「明示的に書けば通る。簡単な省略なら補える」** 程度を目標にするのが安全です。

---

# 14. Phase 3 の完了条件

Phase 3 が完了したと言える条件はこれです。

```text
- import/open/namespace/end を parse できる
- import は module 先頭に限定し、途中 import を拒否できる
- def/theorem/axiom/simple inductive を parse できる
- `->` / `→` を右結合 Pi に desugar できる
- grouped binder `(x y : A)` / `{x y : A}` を scope 規則付きで展開できる
- namespace 付き名前を扱える
- local/global name resolution ができる
- namespace/open の lexical scope を実装できる
- notation declaration を上から順に反映できる
- notation conflict を拒否できる
- non-associative infix chain を parse error にできる
- simple infix notation を扱える
- explicit/implicit binder を扱える
- implicit args を metavariable として挿入できる
- `_` と `?m` を hole goal に変換できる
- named hole の context mismatch を検出できる
- bidirectional elaboration が動く
- simple unification で Eq.refl n の型を補える
- universe metavariable を解決し、未解決なら certificate 化を拒否できる
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
  +, = など → core constant application

implicit args:
  省略された引数 → metavariable → unificationで解決

holes:
  未完成部分 → proof goal

simple elaboration:
  surface AST + expected type → fully explicit core AST
```

Phase 3 では「便利さ」を入れ始めますが、まだ無理に賢くしません。
まずは **小さく、決定的で、kernelに渡せるcore termを確実に作る elaborator** を目指すのがよいです。
