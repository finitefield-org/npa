# NPA Core Specification v0.1

この文書は Phase 0 の成果物です。Rust kernel、certificate checker、elaborator が共有する
最小 core 仕様を固定します。

`doc/phase0.md` は設計メモ、この文書は実装が従う仕様です。

## 1. Overview

NPA v0.1 の trusted boundary は次です。

```text
trusted:
  canonical certificate
  Rust kernel
  independent checker

untrusted:
  source syntax
  parser
  name resolution
  elaborator
  tactic
  automation
  AI output
  source map
```

kernel は canonical core AST だけを検査します。表層構文、notation、implicit arguments、
holes、tactic block、AI hint は core calculus に入りません。

v0.1 に入れるもの:

```text
Sort, BVar, Const, App, Lam, Pi, Let
universe levels: 0, succ, max, imax, param
declarations: axiom, def, theorem, simple inductive
reduction: beta, delta, iota, zeta
initial inductives: Nat, Eq
```

v0.1 に入れないもの:

```text
eta conversion
proof irrelevance as conversion
quotient
typeclass search
metavariables in certificates
general recursion
mutual inductive
nested inductive
coinductive
macros
```

## 2. Judgment Forms

仕様で使う判断は次です。

```text
WFLevel(Delta, ell)
  universe parameter context Delta の下で ell が well-formed level である。

Sigma ; Delta ; Gamma |- t : T
  global environment Sigma、universe context Delta、local context Gamma の下で t は型 T を持つ。

Sigma ; Delta ; Gamma |- t == u : T
  t と u は型 T において definitional equality で等しい。

Sigma |- decl ok
  declaration decl は Sigma に追加可能である。

Sigma |- module ok
  module 内の declarations を順に検査できる。
```

`==` は kernel の計算で判定される definitional equality です。命題としての等号 `Eq` とは別です。

以降では、文脈上明らかな場合に `Delta` を省略して `Sigma ; Gamma |- t : T` と書きます。

## 3. Core Syntax

### 3.1 Names

名前は global constant とデバッグ用 binder name に使います。束縛構造は名前ではなく de Bruijn index で表します。

```text
Name ::= UTF-8 module-qualified identifier
```

certificate hashing では名前を canonical name table に入れ、term は name id を参照します。

### 3.2 Universe Levels

```text
Level ell ::=
  zero
| succ ell
| max ell ell
| imax ell ell
| param alpha
```

略記:

```text
Prop   = Sort zero
Type u = Sort (succ u)
```

`WFLevel(Delta, param alpha)` は `alpha in Delta` の場合だけ成立します。

### 3.3 Terms

```text
Term t ::=
  Sort ell
| BVar i
| Const c [ell_1, ..., ell_n]
| App f a
| Lam x : A, body
| Pi  x : A, B
| Let x : A := v in body
```

`BVar 0` は最も内側の binder を指します。`Lam`、`Pi`、`Let` の `body` 内では binder が 1 つ増えます。

### 3.4 Local Context

```text
Gamma ::= empty | Gamma, LocalDecl

LocalDecl ::=
  x : A
| x : A := v
```

local definition は typing と zeta reduction の両方で使います。

### 3.5 Global Declarations

```text
Decl ::=
  AxiomDecl(name, universe_context, type)
| DefDecl(name, universe_context, type, value, reducibility)
| TheoremDecl(name, universe_context, type, proof)
| InductiveDecl(data)

UniverseContext ::= universe_params + universe_constraints
UniverseConstraint ::= Level <= Level | Level = Level
Reducibility ::= reducible | opaque
```

`theorem` は型検査済みの opaque definition として環境に入ります。conversion では theorem proof を展開しません。

### 3.6 Modules

```text
Module ::= Header Imports Declarations

Import ::= module_name + export_hash + optional certificate_hash
```

declarations は依存関係順に並べます。同じ依存深度の宣言順は module 内の canonical order に従います。
`export_hash` は常に必須です。`certificate_hash` は通常検査では省略可能ですが、高信頼モードでは必須にします。

## 4. Universe System

### 4.1 Sort Hierarchy

```text
Sort ell : Sort (succ ell)
```

したがって:

```text
Prop   : Type 0
Type 0 : Type 1
Type 1 : Type 2
```

`Sort ell : Sort ell` は禁止です。

### 4.2 Constraints

```text
Constraint ::= ell <= ell | ell = ell
```

kernel は declaration ごとに次を確認します。

```text
- すべての level parameter が宣言済みである
- すべての level expression が well-formed である
- universe constraints が自然数解釈で充足可能である
- Sort hierarchy に循環を作らない
```

v0.1 の solver は保守的でよいです。判定不能または未対応の制約は reject します。

### 4.3 imax

`Pi` の sort 計算には `imax` を使います。

```text
imax u zero     = zero
imax u (succ v) = max u (succ v)
```

その他の式は deterministic に正規化します。実装が完全に簡約できない場合でも、同じ入力から同じ正規形を生成しなければいけません。

## 5. Typing Rules

### 5.1 Sort

```text
WFLevel(Delta, ell)
────────────────────────────────
Sigma ; Gamma |- Sort ell : Sort (succ ell)
```

### 5.2 Variable

```text
lookup(Gamma, i) = x : A
────────────────────────
Sigma ; Gamma |- BVar i : lift_for_lookup(A, i)
```

`lift_for_lookup` は de Bruijn index の標準的な lift です。実装は substitution / lifting を一箇所に集約し、
typing と reduction で同じ操作を使います。

local definition も variable として参照できます。

```text
lookup(Gamma, i) = x : A := v
──────────────────────────────
Sigma ; Gamma |- BVar i : lift_for_lookup(A, i)
```

### 5.3 Constant

```text
Sigma(c) = decl
universe_params(decl) = [alpha_1, ..., alpha_n]
length(levels) = n
WFLevel(Delta, levels_k) for all k
universe_constraints(decl)[alpha_k := levels_k] are satisfied
────────────────────────────────────────────
Sigma ; Gamma |- Const c levels : instantiate_levels(type(decl), levels)
```

### 5.4 Pi Formation

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma, x : A |- B : Sort v
────────────────────────────────────────────
Sigma ; Gamma |- Pi x : A, B : Sort (imax u v)
```

これにより `Pi x : A, P` は `P : Prop` なら `Prop` になります。

### 5.5 Lambda

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma, x : A |- body : B
────────────────────────────────────────────
Sigma ; Gamma |- Lam x : A, body : Pi x : A, B
```

### 5.6 Application

```text
Sigma ; Gamma |- f : F
whnf(F) = Pi x : A, B
Sigma ; Gamma |- a : A
────────────────────────────────────────────
Sigma ; Gamma |- App f a : instantiate(B, a)
```

`F` は weak-head normal form まで簡約してから `Pi` かどうかを判定します。

### 5.7 Let

```text
Sigma ; Gamma |- A : Sort u
Sigma ; Gamma |- v : A
Sigma ; Gamma, x : A := v |- body : B
────────────────────────────────────────────
Sigma ; Gamma |- Let x : A := v in body : instantiate(B, v)
```

### 5.8 Conversion

```text
Sigma ; Gamma |- t : A
Sigma ; Gamma |- A == B : Sort u
────────────────────────────
Sigma ; Gamma |- t : B
```

type checking は inference と checking の組み合わせで実装してよいです。ただし、最終的な accept / reject はこの規則に一致します。

## 6. Definitional Equality

### 6.1 Equality Basis

definitional equality は次で生成される最小の congruence です。

```text
alpha equivalence via de Bruijn representation
beta reduction
delta reduction for reducible definitions
iota reduction for recursors
zeta reduction for let and local definitions
```

v0.1 では次を含めません。

```text
eta reduction
proof irrelevance conversion
proof unfolding for theorems
axiom unfolding
```

### 6.2 Beta

```text
App (Lam x : A, body) a
  --> instantiate(body, a)
```

### 6.3 Delta

```text
Const c levels --> instantiate_levels(value(c), levels)
```

ただし `c` が `DefDecl` かつ `reducibility = reducible` の場合だけ展開します。

### 6.4 Iota

recursor の major premise が constructor-headed term の場合に計算します。

`Nat` の例:

```text
Nat.rec motive z s Nat.zero
  --> z

Nat.rec motive z s (Nat.succ n)
  --> s n (Nat.rec motive z s n)
```

generic inductive では、constructor ごとに生成された computation rule に従います。

### 6.5 Zeta

```text
Let x : A := v in body
  --> instantiate(body, v)
```

local definition の参照も weak-head reduction で展開可能です。

### 6.6 Conversion Algorithm Requirements

kernel の conversion checker は次を満たします。

```text
- deterministic
- source location や source syntax に依存しない
- theorem proof を展開しない
- opaque definition を展開しない
- 正規化できない、または resource limit に達した場合は reject する
```

v0.1 には general recursion がないため、well-typed な reducible definition の展開は停止する前提です。

## 7. Declarations

各 declaration は自分の `universe_context` を持ちます。以下の規則では、
`Delta = universe_context(decl)` とし、`Delta` 内の constraints が充足可能であることを前提にします。

### 7.1 Axiom

```text
Sigma ; empty |- type : Sort u
──────────────────────────────
Sigma |- AxiomDecl(name, universe_context, type) ok
```

axiom は certificate の axiom report に必ず現れます。高信頼モードでは allowlist 外 axiom を reject します。

### 7.2 Definition

```text
Sigma ; empty |- type : Sort u
Sigma ; empty |- value : type
────────────────────────────────────
Sigma |- DefDecl(name, universe_context, type, value, reducibility) ok
```

`DefDecl` は environment に定数として追加されます。`reducible` の場合だけ delta reduction の対象です。

### 7.3 Theorem

```text
Sigma ; empty |- type : Sort u
Sigma ; empty |- proof : type
────────────────────────────────
Sigma |- TheoremDecl(name, universe_context, type, proof) ok
```

theorem は environment に opaque constant として追加されます。証明本体は certificate 検査で使いますが、
conversion では展開しません。

### 7.4 Declaration Order

module check は imports を先に読み、declarations を上から順に検査します。未検査の後続 declaration への参照は禁止です。

inductive declaration だけは、inductive type、constructors、recursor、computation rules を同時に生成してから
environment に追加します。v0.1 では mutual block はありません。

## 8. Simple Inductive Types

### 8.1 Declaration Shape

```text
InductiveDecl:
  name
  universe_params
  params:  telescope
  indices: telescope
  sort:    Sort s
  constructors: [ConstructorDecl]

ConstructorDecl:
  name
  type
```

概念的な形:

```text
inductive I.{u} (params : P) : indices -> Sort s where
| c1 : C1
| ...
| cn : Cn
```

### 8.2 Constructor Rule

constructor type は telescope の末尾で対象帰納型を返さなければいけません。

```text
constructor_type =
  Pi y1 : A1, ... Pi yn : An, I params index_args
```

対象外の型を返す constructor は reject します。

### 8.3 Strict Positivity

v0.1 の positivity checker は保守的です。

許可:

```text
- recursive occurrence がない引数
- constructor 引数としての直接再帰 occurrence
  例: Nat.succ : Nat -> Nat
  例: List.cons : A -> List A -> List A
```

禁止:

```text
- 関数の domain 側に現れる recursive occurrence
  例: (I -> Nat) -> I
- nested inductive
  例: List I -> I
- mutual inductive
- recursive occurrence を含む不透明な型 alias
```

この制限は強すぎても v0.1 では構いません。疑わしい constructor は reject します。

### 8.4 Recursor Generation

各 inductive declaration は recursor を生成します。

`Nat` の概念形:

```text
Nat.rec :
  Pi motive : Nat -> Sort u,
    motive Nat.zero ->
    (Pi n : Nat, motive n -> motive (Nat.succ n)) ->
    Pi n : Nat, motive n
```

constructor ごとに 1 つの minor premise を持ち、major premise が constructor-headed term の場合に iota reduction します。

### 8.5 Prop Elimination

`I : Prop` の inductive では、v0.1 の recursor motive は `Prop` に限ります。

```text
allowed:
  motive : I ... -> Prop

rejected:
  motive : I ... -> Type u
```

singleton elimination などの例外は v0.1 には入れません。

### 8.6 Initial Inductives

Phase 1 kernel は少なくとも次を検査できる必要があります。

```text
Nat : Type 0
Nat.zero : Nat
Nat.succ : Nat -> Nat

Eq.{u} : Pi A : Sort u, A -> A -> Prop
Eq.refl.{u} : Pi A : Sort u, Pi x : A, Eq A x x
```

`Nat` と `Eq` は最終的には generic `InductiveDecl` から導入します。実装初期に特別扱いする場合でも、
observable な環境はこの仕様と一致させます。

## 9. Certificate Schema

### 9.1 Logical Schema

certificate は source ではなく canonical module です。

```text
Certificate:
  format: "NPA-CERT-0.1"
  module: module name
  imports: [Import]
  names: canonical name table
  levels: canonical level table
  terms: canonical term DAG
  declarations: [Decl]
  export_block: ExportBlock
  axiom_report: AxiomReport
  hashes:
    export_hash: sha256(canonical_export_block)
    axiom_report_hash: sha256(canonical_axiom_report)
    certificate_hash: sha256(trusted_payload_without_certificate_hash)
```

### 9.2 Canonicalization

hash に影響する payload は次を満たします。

```text
- field order is fixed
- arrays have explicit length
- names are sorted by UTF-8 byte lexicographic order
- level and term DAGs are topologically ordered; ties use structural tag order
- declarations are dependency ordered
- import order is lexicographic by module name, then export_hash, then certificate_hash option/value
- term binders use de Bruijn indices
- no whitespace or comments
- no notation
- no implicit arguments
- no unresolved metavariables
- universe constraints are normalized
```

source map、comments、diagnostics、AI traces は trusted payload に含めません。

### 9.3 Binary Encoding Requirement

on-disk `.npcert` は canonical binary にします。v0.1 の実装が JSON を使う場合でも、JSON はデバッグ形式であり、
hash-stable artifact ではありません。

canonical binary の最低条件:

```text
- integers are unsigned LEB128
- strings are byte length + UTF-8 bytes
- enum variants use fixed numeric tags
- maps are forbidden in the hashed payload
- optional fields use explicit tag 0/1
- sha256 is computed over the exact canonical byte sequence
```

### 9.4 Axiom Report

axiom report は、各 theorem / def / inductive が transitively 依存する axioms の集合から作ります。

```text
axioms_used(decl) =
  direct axioms referenced in type/value/proof
  union axioms_used(transitive dependencies)
```

module の `module_axioms` は declarations の axiom set の和集合を canonical order で保存します。

```text
AxiomReport:
  per_declaration: [(decl_index, [Name])]
  module_axioms: [Name]
```

`per_declaration` は declaration order、各 axiom list と `module_axioms` は canonical name order で保存します。
`safe_for_high_trust`、`contains_sorry`、allowlist 判定などは audit/policy view であり、trusted payload 内の真偽値としては信用しません。

### 9.5 Import Hash

import は module name だけではなく export hash を含めます。

```text
Import:
  module = "Std.Nat.Basic"
  export_hash = "sha256:..."
  certificate_hash = optional "sha256:..."
```

kernel は import name と hash が実際に読み込まれた module と一致することを確認します。
高信頼モードでは `certificate_hash` も一致し、同じ checker が import certificate を検査済みであることを要求します。

### 9.6 Hash Roles

`export_hash` と `certificate_hash` は同じ対象を hash しません。

```text
export_hash:
  downstream module が型検査・conversionに必要とする公開インターフェースのhash

certificate_hash:
  opaque theorem proof body などを含む trusted certificate payload 全体のhash
```

opaque theorem の proof だけが変わり、type・opacity・axiom dependency が変わらない場合、
`certificate_hash` は変わりますが `export_hash` は維持されます。
proof 変更によって axiom dependency が変わる場合は公開される信頼情報が変わるため、`export_hash` も変わります。

## 10. Kernel Checking Algorithm

### 10.1 Module Checking

```text
check_module(cert):
  verify format
  load imports by name and hash
  initialize Sigma from imports
  for decl in declarations:
    check_decl(Sigma, decl)
    recompute and compare declaration hashes
    extend Sigma with decl
  compute axiom_report
  compare with certificate axiom_report
  compute axiom_report_hash
  compare with certificate axiom_report_hash
  compute export_hash over canonical export block
  compare with certificate export_hash
  compute certificate_hash over trusted payload excluding certificate_hash itself
  compare with certificate certificate_hash
```

### 10.2 Type Inference

```text
infer(Sigma, Gamma, term) -> type
check(Sigma, Gamma, term, expected_type)
is_defeq(Sigma, Gamma, lhs, rhs, type) -> bool
whnf(Sigma, Gamma, term) -> term
```

`check` は `infer` の結果と expected type を conversion で比較してよいです。`Lam` のように expected type が有用な場合は
bidirectional typing を使ってよいですが、結果は typing rules と一致させます。

### 10.3 Error Model

kernel error は構造化 enum として返します。人間向け message は非信頼情報です。

最低限必要な error class:

```text
UnknownConstant
UnknownUniverseParam
BadUniverseArity
InvalidBVar
ExpectedSort
ExpectedPi
TypeMismatch
NotDefEq
InvalidInductive
NonPositiveOccurrence
BadConstructorResult
InvalidRecursor
HashMismatch
AxiomNotAllowed
ResourceLimit
```

## 11. Minimal Examples

### 11.1 id

```text
id.{u} :
  Pi A : Sort u,
    Pi x : A,
      A

id.{u} :=
  Lam A : Sort u,
    Lam x : BVar 0,
      BVar 0
```

Phase 1 kernel はこの definition を検査できなければいけません。

### 11.2 Eq.refl Proof

```text
theorem Nat.zero_eq_zero :
  Eq Nat Nat.zero Nat.zero

proof:
  Eq.refl Nat Nat.zero
```

`Eq.refl Nat Nat.zero` の型が theorem type と definitional equality で一致すれば accept します。

### 11.3 add_zero Target

`Nat.add` を第 2 引数で再帰する reducible definition として定義した場合:

```text
theorem add_zero :
  Pi n : Nat, Eq Nat (Nat.add n Nat.zero) n
```

`Nat.add n Nat.zero` が delta + iota により `n` へ簡約されるため、proof は本質的に `Eq.refl Nat n` です。

## 12. Phase 0 Exit Criteria

Phase 0 は次を満たしたら完了とします。

```text
- kernel が読む core syntax が固定されている
- typing judgment が定義されている
- conversion の範囲が明確である
- universe inconsistency を reject できる
- simple inductive の範囲と positivity 条件が明確である
- recursor と iota reduction の責務が明確である
- certificate の canonicalization と hash 方針がある
- source syntax なしで certificate だけを検査できる
- def / theorem / axiom / inductive の差が明確である
- Phase 1 の最小検査対象が明確である
```
