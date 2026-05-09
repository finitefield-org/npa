# Phase 3 AI Profile: Machine Surface

この文書は、NPA の **AI 向け Phase 3** の設計と実装計画です。

従来の `doc/phase3-human.md` は、人間が読み書きしやすい表層言語を canonical core AST に落とすための
設計です。一方、AI 証明探索では、候補を大量に生成し、失敗を前提に高速に検査します。
そのため、AI 向け Phase 3 では人間向けの便利機能を削り、canonical core に近い明示的な
入力形式を使います。

この AI 向け入力形式を **Machine Surface** と呼びます。

Machine Surface は trusted language ではありません。parser / resolver / elaborator / AI output は
すべて非信頼層です。最終的な正しさは、Phase 1 kernel と Phase 2 certificate verifier が
fully explicit core declaration / proof term を検査することで確認します。

---

# 1. 目的

Machine Surface の目的は、AI が生成する証明候補を速く、決定的に、修復しやすく検査することです。

通常の人間向け Phase 3:

```text
human source
  ↓ parser with active notation table
surface AST with notation nodes
  ↓ name resolution + notation candidate collection
resolved AST with overloaded names / notation candidates
  ↓ elaboration + metavariable solving
fully explicit core AST
  ↓ certificate generation
canonical certificate
```

AI 向け Phase 3:

```text
machine source / structured AI request
  ↓ fixed parser
machine AST
  ↓ direct resolver
resolved machine AST
  ↓ explicit elaboration
fully explicit core AST
  ↓ kernel check
  ↓ certificate generation
canonical certificate
```

AI 向けでは次を優先します。

```text
- active notation table に依存しない
- open / namespace に依存しない
- overload candidate transaction を作らない
- implicit term argument を自動挿入しない
- unresolved hole を表現しない
- failure を structured error として返す
- 同じ入力から同じ core hash / error を返す
```

---

# 2. 信頼境界

Machine Surface の入力は信用しません。

```text
信頼しない:
  AI output
  Machine Surface parser
  direct resolver
  elaborator
  repair suggestion
  structured hint
  source span

信頼する:
  canonical core AST
  Phase 1 Rust kernel
  Phase 2 certificate verifier
  Phase 8 independent checker
```

Machine Surface 固有の情報は certificate に残しません。

```text
trusted payload に入れない:
  machine source text
  AI prompt / completion / trace
  repair suggestion
  source span
  display name
  notation metadata
  implicit argument metadata
  unresolved metavariable / hole / placeholder
```

AI が fully explicit な term を出しても、それは証明ではありません。
kernel check と certificate check に通ったものだけを証明として採用します。

---

# 3. 設計方針

Machine Surface は、人間向け Phase 3 の subset ではありますが、実装上は最初から AI-first として
作ります。人間向け機能を入れてから flag で無効化するのではなく、曖昧性を持たない parser /
resolver / elaborator を先に作ります。

## 3.1 使わない機能

Machine Surface MVP では次を使いません。

```text
- notation declaration
- prefix / postfix / infix notation
- overloaded notation
- overloaded short name
- open
- namespace
- implicit term argument insertion
- unannotated lambda binder
- unannotated declaration binder
- unannotated let
- unresolved hole
- named hole reuse
- source-level axiom declaration
- source-level inductive declaration
- source-level recursive definition syntax
- typeclass search
- coercion search
- numeric literal overload
```

これらは将来、人間向け surface language か別 profile として追加してよいですが、AI-first Phase 3 の
MVP には入れません。

## 3.2 使う機能

Machine Surface MVP で使うものは次です。

```text
module item:
  import
  def
  theorem

term:
  fully qualified global name
  local name
  explicit universe application
  Sort / Type / Prop
  application
  typed lambda
  typed forall / Pi
  typed let
  annotation
  parenthesized term
```

AI は `n + 0 = n` ではなく、次のような形を出します。

```npa
Eq.{1} Nat (Nat.add n Nat.zero) n
```

implicit argument がある定理や定義は、`@` 付きで term argument を明示します。

```npa
@Eq.refl.{1} Nat n
```

---

# 4. Machine Surface Syntax

## 4.1 Module grammar

```text
machine_module ::=
    import_item* machine_item*

import_item ::=
    "import" qual_name

machine_item ::=
    machine_def
  | machine_theorem

machine_def ::=
    "def" qual_name universe_params? machine_binder* ":" machine_term ":=" machine_term

machine_theorem ::=
    "theorem" qual_name universe_params? machine_binder* ":" machine_term ":=" machine_term

machine_binder ::=
    "(" ident ":" machine_term ")"
```

`import` は module の先頭にだけ置きます。
`def` / `theorem` の name は current namespace からの相対名ではなく、そのまま declaration name として扱います。
Machine Surface MVP には `namespace` がないため、declaration name の変換規則を持ちません。

## 4.2 Term grammar

```text
machine_term ::=
    machine_app
  | "fun" machine_binder+ "=>" machine_term
  | "forall" machine_binder+ "," machine_term
  | "let" ident ":" machine_term ":=" machine_term "in" machine_term
  | machine_app ":" machine_term

machine_app ::=
    machine_atom machine_atom*

machine_atom ::=
    term_name universe_args?
  | "@" term_name universe_args?
  | "Prop"
  | "Type" level?
  | "Sort" level
  | "(" machine_term ")"

term_name ::= name_component ("." name_component)*

universe_params ::= ".{" ident ("," ident)* "}"
universe_args   ::= ".{" level ("," level)* "}"

level ::=
    natural
  | ident
  | "succ" level
  | "max" level level
  | "imax" level level
```

`name_component` は machine source 上の name component spelling であり、identifier と同じ ASCII subset を使います。
Keyword と同じ spelling の component は、grammar 上 keyword として読む位置では keyword ですが、dotted name の
component として読む位置では name component です。
たとえば `Nat.succ` の `succ` は name component であり、level grammar の `succ` keyword ではありません。

Lexer / parser は reserved token を通常の binder `ident`、universe parameter `ident`、または
unqualified / head `term_name` として受理してはいけません。
次の token は、term の該当構文または dedicated keyword としてだけ扱います。

```text
reserved tokens:
  import
  def
  theorem
  forall
  fun
  let
  in
  Prop
  Type
  Sort
  succ
  max
  imax
  open
  namespace
  match
  with
```

ただし `term_name` の `.` より後ろの component では、上の spelling も name component として使えます。
たとえば `Nat.succ`、`M.Type`、`M.match` は dotted `term_name` として parse できます。
たとえば単独の `Prop` は常に Prop atom、`Type` は Type atom、`Sort` は Sort atom であり、
`NameAtom("Prop")` / `NameAtom("Type")` / `NameAtom("Sort")` として canonicalize してはいけません。
`succ` / `max` / `imax` は level grammar 内では keyword であり、level parameter name には使えません。

MVP では `->` / `→` を使いません。関数型は `forall (x : A), B` と書きます。
anonymous binder も使いません。

## 4.3 Syntax rejection

次は parse error または dedicated diagnostic として拒否します。

```npa
open Nat
namespace Nat
infixl:65 " + " => Nat.add
axiom choice : ...
inductive Nat : Type where ...
fun x => x
let x := Nat.zero in x
_
?m
n + Nat.zero
```

---

# 5. Name Resolution

## 5.1 Global names

global name は fully qualified exact match のみです。

許可:

```text
Nat
Nat.zero
Nat.add
Eq
Eq.refl
Std.Logic.False.elim
```

禁止:

```text
zero
add
refl
```

term name resolution は `canonicalize_machine_term_source` ではなく、`elaborate_machine_term_check` / module
elaboration でだけ行います。resolution は次の順に行います。

```text
1. term_name が @ term_name、universe_args 付き、または component_count > 1 の場合:
   global exact match だけを試す。local context は見ない。
2. それ以外の 1 component term_name の場合:
   local context に未修飾 local name として存在するなら local。
   存在しなければ global exact match を試す。
3. それ以外は UnknownName
```

ただし local name が global declaration と同じ fully qualified root name を shadow する場合は拒否します。

```npa
-- reject
theorem bad (Nat : Type) (x : Nat) : Nat := x
```

理由:

```text
- suffix lookup を消す
- open scope を消す
- overload candidate を消す
- AI repair を fully qualified name の補正だけにする
```

## 5.2 Imported names

import は Phase 2 verified module から解決します。

```text
source:
  import Std.Nat.Basic

compile request:
  verified_imports:
    Std.Nat.Basic + export_hash + optional certificate_hash
```

Machine Surface resolver は filesystem や network から import を探しません。
compile request が渡す verified import set だけを見ます。

imported global は、export block にある `name + decl_interface_hash` によって固定します。

将来的な AI API では、source text の name ではなく structured ref を直接渡せるようにします。

```json
{
  "kind": "imported",
  "module": "Std.Nat.Basic",
  "name": "Nat.add",
  "decl_interface_hash": "sha256:..."
}
```

structured ref も信用しません。Phase 2 certificate verifier が、import entry と export block によって
再検査します。

---

# 6. Elaboration

Machine Surface elaboration は explicit であることを前提にします。

```text
有効:
  explicit application
  typed lambda
  typed Pi
  typed let
  annotation
  explicit universe args

無効:
  implicit term arg insertion
  notation candidate selection
  overload resolution
  hole goal creation
  typeclass search
  coercion search
```

## 6.1 Application

`f a` は次の手順で elaboration します。

```text
1. f を infer
2. type(f) を WHNF
3. 先頭が Pi であることを確認
4. a を domain に対して check
5. codomain を instantiate
```

関数型の caller-supplied metadata に implicit binder があっても、Machine Surface は implicit term argument を
自動挿入しません。
implicit binder 位置へ明示 argument を渡したい場合は `@` 付き head を使います。

```npa
@Eq.refl.{1} Nat n
```

`Eq.refl n` を `ImplicitArgumentRequired` として拒否するのは、caller が渡した callable profile で
`Eq.refl` の先頭 term binder が implicit と固定されている場合です。
Phase 5 AI MVP v1 のように caller が all-explicit callable profile を渡す protocol では、その profile が
この判定の正本です。
その場合 `@` は exact-match / canonical source marker として許されますが、implicit binder 消費は発生せず、
implicit term binder 不足による `ImplicitArgumentRequired` も発生しません。
Machine Surface parser / elaborator は server-local registry、元 source、pretty metadata から implicit profile を
補完してはいけません。
現行の `MachineTermElabContext` に callable profile field がない実装では、Phase 5 adapter が wrapper context または
明示引数で同等の metadata を渡します。
Machine Surface 単体テストで implicit binder の挙動を検査する場合も、どの callable profile を使うかを fixture に
明示します。

## 6.2 Lambda

lambda binder は必ず型注釈を持ちます。

```npa
fun (x : Nat) => x
```

次は拒否します。

```npa
fun x => x
```

期待型から binder type を補う処理は MVP では使いません。

## 6.3 Pi / forall

`forall` binder も必ず型注釈を持ちます。

```npa
forall (x : Nat), Nat
```

`A -> B` は MVP では syntax として入れません。

## 6.4 Let

`let` も必ず型注釈を持ちます。

```npa
let x : Nat := Nat.zero in x
```

次は拒否します。

```npa
let x := Nat.zero in x
```

## 6.5 Universe

declaration-level universe parameter は明示します。

```npa
def id.{u} (A : Sort u) (x : A) : A := x
```

polymorphic constant を使う場合も、AI が生成する source では universe args を明示することを推奨します。

```npa
@Eq.refl.{1} Nat n
```

MVP 実装では、imported polymorphic constant の universe argument 省略を内部 universe metavariable にしてもよいですが、
complete mode の終了時に未解決なら拒否します。AI 生成の標準形は明示 universe です。

---

# 7. Modes

Machine Surface には 2 つの mode を置きます。

```rust
pub enum MachineSurfaceMode {
    Complete,
    Repair,
}
```

## 7.1 Complete mode

Complete mode は、certificate 候補として使う source を検査する mode です。

拒否するもの:

```text
- unresolved UserHole
- unresolved SyntheticImplicit
- unresolved UniverseMeta
- OverloadedRef / OverloadedApp
- notation
- open / namespace
- source-level axiom
- source-level inductive
- unknown global name
- short global name
```

成功時は fully explicit core module / term を返します。

```json
{
  "status": "complete",
  "core_hash": "sha256:...",
  "constants": ["Eq.refl", "Nat"],
  "ready_for_certificate": true
}
```

## 7.2 Repair mode

Repair mode は、AI 修復ループのために構造化エラーを返す mode です。

例:

```json
{
  "status": "error",
  "error": {
    "kind": "implicit_argument_required",
    "function": "Eq.refl",
    "binder_index": 0
  },
  "suggestions": [
    {
      "replacement": "@Eq.refl.{1} Nat n"
    }
  ]
}
```

Repair mode の suggestion は信用しません。suggestion を再投入し、Complete mode と kernel /
certificate check に通った場合だけ採用します。

---

# 8. Structured Errors

AI 向け Phase 3 のエラーは、自然文中心ではなく enum 中心にします。

```rust
pub enum MachineErrorKind {
    ParseError,
    UnsupportedItem,
    UnsupportedSyntax,
    ImportAfterItem,
    ImportResolutionError,
    MissingVerifiedImport,
    UnknownGlobalName,
    ShortGlobalName,
    AmbiguousGlobalName,
    GlobalShadowedByLocal,
    UnknownLocalName,
    DuplicateDeclaration,
    DuplicateUniverseParam,
    UnknownUniverseParam,
    ImplicitArgumentRequired,
    MissingExplicitUniverse,
    UnannotatedBinder,
    UnannotatedLet,
    HoleNotAllowed,
    ExpectedFunctionType,
    ExpectedSort,
    TypeMismatch,
    TooManyArguments,
    TooFewArguments,
    UnsolvedUniverseMeta,
    KernelRejected,
    CertificateRejected,
}
```

diagnostic payload には可能な範囲で次を入れます。

```text
- source span
- head symbol
- expected type core hash
- actual type core hash
- target core hash
- constants in term
- candidate fully qualified names
- suggested machine replacement
```

ただし source span や suggestion は trusted payload に入りません。

---

# 9. Public API

MVP の crate は `crates/npa-frontend` とします。
ただし public API は人間向け surface frontend ではなく、Machine Surface であることが分かる名前にします。

```rust
pub struct MachineModule {
    pub name: ModuleName,
    pub items: Vec<MachineItem>,
}

pub struct MachineCompileOptions {
    pub mode: MachineSurfaceMode,
    pub allow_universe_meta: bool,
}

pub struct MachineTermElabContext {
    pub global_scope: MachineGlobalScope,
    pub local_context: Vec<LocalDecl>,
    pub universe_params: Vec<String>,
    pub kernel_env: MachineKernelEnvView,
}

pub struct MachineKernelEnvView {
    // Opaque checked environment built by the caller from verified imports and checked current decls.
}

pub struct MachineGlobalScope {
    pub entries: Vec<MachineGlobalScopeEntry>,
}

pub enum MachineGlobalScopeEntry {
    Imported {
        name: Name,
        import_index: u32,
        decl_interface_hash: Hash,
    },
    CurrentModule {
        name: Name,
        source_index: u64,
        decl_interface_hash: Hash,
    },
    CurrentGenerated {
        name: Name,
        parent_source_index: u64,
        decl_interface_hash: Hash,
    },
}

pub struct MachineTermSourceCanonical {
    pub source: String,
    pub canonical_bytes: Vec<u8>,
    pub canonical_hash: Hash,
}

pub fn parse_machine_module(
    file_id: FileId,
    source: &str,
) -> Result<MachineModule, MachineDiagnostic>;

pub fn resolve_machine_module(
    module: MachineModule,
    verified_imports: &[VerifiedImport],
) -> Result<ResolvedMachineModule, MachineDiagnostic>;

pub fn elaborate_machine_module(
    module: ResolvedMachineModule,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<CoreModule, MachineDiagnostic>;

pub fn compile_machine_source_to_core(
    file_id: FileId,
    module_name: ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<CoreModule, MachineDiagnostic>;

pub fn compile_machine_source_to_certificate(
    file_id: FileId,
    module_name: ModuleName,
    source: &str,
    verified_imports: &[VerifiedModule],
    options: &MachineCompileOptions,
) -> Result<ModuleCert, MachineDiagnostic>;
```

Phase 5 / Phase 7 用には term 単体 API も用意します。

```rust
pub fn canonicalize_machine_term_source(
    source: &str,
) -> Result<MachineTermSourceCanonical, MachineDiagnostic>;

pub fn elaborate_machine_term_check(
    source: &str,
    context: &MachineTermElabContext,
    expected: &npa_kernel::Expr,
    options: &MachineCompileOptions,
) -> Result<npa_kernel::Expr, MachineDiagnostic>;
```

`MachineTermElabContext.global_scope` は exact name lookup 用の閉じた map です。
resolver は filesystem、network、global package cache、`open` / namespace state を読んではいけません。
Phase 5 は direct import の public export、checked current declaration、current generated constructor / recursor から
この scope を構築します。
`kernel_env` は同じ verified imports / checked current declaration から作った checked environment であり、
global name lookup の候補を増やすために使ってはいけません。型取得、reduction、conversion、kernel check の入力にだけ使います。

`Imported.import_index` は Phase 5 / Phase 4 が決めた canonical direct import order の index です。
`CurrentModule.source_index` と `CurrentGenerated.parent_source_index` は Phase 5 session 内の source_index 座標です。
最終 certificate 生成時に Phase 2 declaration index が source_index と異なる場合は、Phase 5 verify が
`GlobalRef::Local` / `GlobalRef::LocalGenerated` を certificate-local index へ rewrite します。

`canonicalize_machine_term_source` は raw source text ではなく、parse 後の Machine Surface term AST を canonical bytes にします。
`MachineTermSourceCanonical.canonical_hash` は `hash(canonical_bytes)` であり、Phase 3 term-source hash です。
これは Phase 4 wrapper hash ではありません。
Phase 4 `MachineTermSource.canonical_hash` は、Phase 4 tag と `canonical_bytes` から Phase 4 側で計算します。
`canonicalize_machine_term_source` は `MachineTermElabContext` を受け取らず、global name resolution、local context lookup、
type checking、reduction、kernel environment lookup を行いません。
context-dependent な check は `elaborate_machine_term_check` だけが行います。
下の `NameAtom` は parser-level の構文 variant であり、core `GlobalRef` / local de Bruijn への解決結果ではありません。
`Nat`、`Eq`、`n` のような 1 component name は、この段階ではすべて `NameAtom` として同じ規則で encode します。
local context による local/global 分類、shadowing check、UnknownName は `elaborate_machine_term_check` でだけ判定します。
この block の string / list / option / unsigned integer / hash primitive は、Phase 2 canonical encoding と同じ
minimal unsigned LEB128 length + UTF-8 bytes / raw digest bytes を使います。

```text
Machine Surface term-source canonical bytes:
  - tag "npa.phase3.machine-term-source.v1"
  - parsed term AST encoded with the canonical grammar below

Term canonical bytes:
  - NameAtom:
      variant tag
      name component list:
        component count as minimal unsigned LEB128 u32
        each component as UTF-8 string primitive bytes
      explicit-at marker: 0x00 normal | 0x01 at-form
      universe_args list in source order as Level canonical bytes
  - Sort:
      level canonical bytes
  - Type:
      level canonical bytes for the displayed Type parameter
  - Prop:
      fixed variant tag
  - App:
      function term canonical bytes
      argument term canonical bytes
  - Lam:
      binders in source order:
        binder local identifier
        binder type term canonical bytes
      body term canonical bytes
  - Pi:
      binders in source order:
        binder local identifier
        binder type term canonical bytes
      body term canonical bytes
  - Let:
      local identifier
      type term canonical bytes
      value term canonical bytes
      body term canonical bytes
  - Annot:
      term canonical bytes
      type term canonical bytes

Level canonical bytes:
  - natural:
      minimal unsigned LEB128 value
  - param:
      universe parameter identifier as UTF-8 string primitive bytes
  - succ:
      payload level canonical bytes
  - max / imax:
      lhs level canonical bytes
      rhs level canonical bytes
```

whitespace、式をグループ化するだけの parentheses、source span、diagnostic text、pretty text、AI trace は
term-source canonical bytes に入りません。
Binder / local / universe parameter name は JSON / source decode 後の UTF-8 byte sequence をそのまま encode します。
したがって binder 名だけが違う alpha-equivalent source は、異なる Phase 3 term-source hash を持ち得ます。
それを payload にする Phase 4 `MachineTermSource.canonical_hash` も異なり得ます。
これは cache key を保守的に分けるためであり、証明の正しさは elaborated core term と kernel check で判断します。

---

# 10. Phase 5 / Phase 7 との接続

AI に渡す proof state は、人間向け pretty string と Machine Surface string を分けます。

```json
{
  "target": {
    "pretty": "n + 0 = n",
    "machine": "Eq.{1} Nat (Nat.add n Nat.zero) n",
    "core_hash": "sha256:...",
    "constants": ["Eq", "Nat.add", "Nat.zero"]
  },
  "locals": [
    {
      "name": "n",
      "type_machine": "Nat",
      "type_hash": "sha256:..."
    }
  ]
}
```

AI の出力例:

```json
{
  "tactic": "exact @Eq.refl.{1} Nat n",
  "term_machine": "@Eq.refl.{1} Nat n",
  "expected_target_hash": "sha256:..."
}
```

Phase 5 の tactic execution API は、term 部分を Machine Surface Complete mode で check します。
通った term だけ tactic execution に渡します。

---

# 11. 実装計画

現在の方針は、人間向け Phase 3 実装をいったん戻し、AI-first Phase 3 を最初から実装することです。
以下の milestone は、`4d9438192c9b7f520e29f3fd682350710897b56c` 相当の状態を再出発点にする前提です。

## M0: Restart baseline

目的:

```text
人間向け frontend 実装を持たない clean baseline を作る。
```

作業:

```text
- main を revert 済み状態にする
- doc/phase3-ai.md を追加する
- README から AI-first Phase 3 を参照できるようにする
- cargo test --workspace を通す
```

完了条件:

```text
- crates/npa-frontend が存在しない、または空の AI-first skeleton のみ
- Phase 1 / Phase 2 の tests が通る
- Phase 3 の実装仕様が Machine Surface に固定されている
```

## M1: Frontend crate skeleton

目的:

```text
Machine Surface 専用の frontend crate を作る。
```

作るファイル:

```text
crates/npa-frontend/Cargo.toml
crates/npa-frontend/src/lib.rs
crates/npa-frontend/src/span.rs
crates/npa-frontend/src/diagnostic.rs
crates/npa-frontend/src/machine.rs
crates/npa-frontend/src/lexer.rs
crates/npa-frontend/src/parser.rs
crates/npa-frontend/src/resolver.rs
crates/npa-frontend/src/elaborator.rs
```

完了条件:

```text
- workspace に npa-frontend が入る
- public API が machine-oriented name になっている
- empty module / simple diagnostic tests が通る
```

## M2: Machine parser

目的:

```text
Machine Surface syntax だけを parse する。
```

accepted tests:

```text
- import
- def id
- theorem self_eq
- explicit universe args
- typed fun
- typed forall
- typed let
- annotation
```

rejected tests:

```text
- open
- namespace
- notation declaration
- axiom
- inductive
- hole
- unannotated lambda binder
- unannotated let
- operator notation
```

完了条件:

```text
- parser state に notation table / open scope がない
- same input から same AST が得られる
```

## M3: Direct resolver

目的:

```text
fully qualified exact match だけで global を解決する。
```

作業:

```text
- verified import interface を読む
- local context lookup を実装する
- exact global lookup を実装する
- short global name を拒否する
- local/global shadowing を拒否する
```

完了条件:

```text
- Nat.add は解決できる
- add は拒否される
- suffix lookup がない
- overload candidate が作られない
```

## M4: Explicit elaborator

目的:

```text
Machine Surface term を fully explicit core Expr に落とす。
```

作業:

```text
- Sort / Type / Prop
- Const / local BVar
- App
- Lam
- Pi
- Let
- annotation
- explicit universe args
- declaration binder closing
```

完了条件:

```text
- explicit id が core def になる
- explicit Eq.refl proof が core theorem になる
- implicit binder を含む callable profile では Eq.refl n が ImplicitArgumentRequired
- all-explicit callable profile では implicit term binder 不足による ImplicitArgumentRequired が発生しない
- fun x => x は UnannotatedBinder
- _ は HoleNotAllowed
```

## M5: Kernel handoff

目的:

```text
Machine Surface から CoreModule を作り、Phase 1 kernel に渡す。
```

作業:

```text
- imported environment を kernel env に入れる
- def value : type を check する
- theorem proof : type を check する
- kernel error を MachineDiagnostic に包む
```

完了条件:

```text
- well-typed def/theorem が通る
- ill-typed application が reject される
- generated CoreModule に Machine Surface metadata が残らない
```

## M6: Certificate integration

目的:

```text
Machine Surface source から Phase 2 certificate を作って verify する。
```

作業:

```text
- CoreModule -> build_module_cert
- encode_module_cert
- verify_module_cert
- verified imports と export_hash の接続
```

完了条件:

```text
- .npcert が source なしで verify できる
- same source から same certificate hash
- certificate に AI trace / source span / Machine metadata が入らない
```

## M7: Term-level API for tactics

目的:

```text
Phase 5 / Phase 7 が tactic candidate 内の term を Machine Surface として検査できる。
```

作業:

```text
- elaborate_machine_term_check
- local context import
- expected type check
- constants / core_hash extraction
```

完了条件:

```text
- exact @Eq.refl.{1} Nat n が goal を閉じる term として check できる
- failed term が structured error を返す
- tactic execution は unchecked AI text を直接信用しない
```

## M8: Repair mode

目的:

```text
AI repair loop が使いやすい structured error と suggestion を返す。
```

作業:

```text
- missing explicit implicit arg suggestion
- short name -> fully qualified candidate suggestion
- missing universe arg suggestion
- type mismatch payload
```

完了条件:

```text
- suggestion は trusted payload に入らない
- suggestion は再投入して通った場合だけ採用
- same failure から same error enum が返る
```

## M9: Performance / determinism gate

目的:

```text
AI 探索で大量の候補を投げても安定して動くことを確認する。
```

測定:

```text
- parse latency
- resolve latency
- elaborate latency
- failed candidate latency
- allocation
- same input same output
```

完了条件:

```text
- active notation table が存在しない
- open scope が存在しない
- overload transaction が存在しない
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
```

---

# 12. MVP Scope

最初の MVP は M0 から M6 までです。

```text
MVP includes:
  import
  def
  theorem
  fully qualified names
  explicit universe args
  typed lambda / Pi / let
  kernel handoff
  certificate generation / verification

MVP excludes:
  axiom source syntax
  inductive source syntax
  notation
  implicit insertion
  holes
  tactic blocks
  repair suggestions
```

Nat / Eq / standard theorem は、AI が source で定義するのではなく、Phase 2 済み verified import として
読む方針にします。

---

# 13. 完了条件

AI 向け Phase 3 が完了したと言える条件は次です。

```text
- Machine Surface syntax が実装されている
- parser / resolver / elaborator が notation / open / overload を持たない
- implicit term arg insertion がない
- unresolved hole が表現不能または complete mode で拒否される
- fully explicit def/theorem が CoreModule に落ちる
- kernel check に通るものだけ certificate 化される
- .npcert は source なしで verify できる
- Phase 5 / Phase 7 が term-level API を使える
- structured error が AI repair に使える
- cargo fmt / clippy / test が通る
```

---

# 14. 一文でまとめると

AI 向け Phase 3 は、**人間に便利な表層言語ではなく、AI が大量に生成する候補を高速・決定的に
canonical core AST へ落とす Machine Surface frontend** です。

便利な構文は後から別 profile として足せます。MVP では、完全明示に近い構文だけを通し、
kernel と certificate verifier に最短経路で渡します。
