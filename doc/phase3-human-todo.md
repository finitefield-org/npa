# Phase 3 Human TODO

この TODO は `doc/phase3-human.md` を正とし、現在の `crates/npa-frontend` /
`crates/npa-api` 実装との差分を実装タスクに分解したものです。

重要な制約として、Phase 3 Human は人間が読み書きしやすい表層言語を追加しますが、
AI 向け Machine Surface の実行速度と決定性を落としてはいけません。
人間向け機能は Machine Surface の hot path に混ぜず、別の parser / AST /
resolver / elaborator entrypoint として追加します。

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在の `crates/npa-frontend` は、AI 向け Machine Surface を中心に実装されています。
以下は Phase 3 Human 実装の前提として使ってよいです。

```text
crates/npa-frontend/src/lib.rs
- Machine Surface public API の export
- module compile / term check / certificate handoff API

crates/npa-frontend/src/machine.rs
- MachineModule / MachineItem / MachineDecl / MachineTerm
- MachineSurfaceMode
- MachineTermElabContext
- Machine Surface callable interface table

crates/npa-frontend/src/lexer.rs
- Machine Surface tokenization
- unsupported character rejection

crates/npa-frontend/src/parser.rs
- import / def / theorem
- explicit universe parameters
- explicit binders
- Prop / Type / Sort
- application / lambda / forall / let / annotation / parenthesized term
- Machine Surface で使わない open / namespace / notation / axiom / inductive / hole の拒否

crates/npa-frontend/src/resolver.rs
- verified import interface lookup
- local context lookup
- deterministic global lookup
- no filesystem / network import lookup

crates/npa-frontend/src/elaborator.rs
- Machine Surface module elaboration
- term-level infer / check API
- Phase 1 kernel handoff
- Phase 2 certificate build / verify handoff

crates/npa-frontend/src/term_source.rs
- canonical Machine Surface term source encoding
- canonical hash
- canonical decode
```

`crates/npa-api` 側では、Phase 4 / 5 / 7 / 8 / 9 の automation が
Machine Surface term を前提に `parse_machine_term`、
`canonicalize_machine_term_source`、`elaborate_machine_term_check`、
`elaborate_machine_term_infer_from_ast` を呼んでいます。
これらの経路は AI 探索の高頻度候補検査経路なので、Phase 3 Human 実装で遅くしてはいけません。

### 0.2 未実装の Human Surface 範囲

`doc/phase3-human.md` が要求する以下の機能は、現在のコードには Human Surface として存在しません。

```text
Human Surface AST
Human parser entrypoint
FrontendState
namespace / open / end
human source import interface reconciliation
axiom declaration
simple inductive declaration
notation declaration
infix / infixl / infixr
operator precedence / associativity table
qualified / unqualified name resolution with open scopes
ambiguous name tracking
notation overload candidate tracking
implicit binder metadata
grouped binder expansion
arrow desugaring
implicit argument insertion
term metavariable store
universe metavariable store
hole goal reporting
named hole context check
simple unification
bidirectional elaboration for omitted arguments
source interface metadata export/import
human diagnostics separate from MachineDiagnostic
Human API endpoints
```

### 0.3 Machine Surface 側に入れてはいけないもの

以下は Human Surface で実装してもよいが、Machine Surface の grammar / resolver /
canonical term source / tactic candidate path に入れてはいけません。

```text
notation table
open scope
namespace stack
implicit argument insertion
unresolved hole
named hole reuse
overload transaction
typeclass search
coercion search
source-level axiom syntax
source-level inductive syntax
human display name metadata
filesystem import lookup
network import lookup
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 3 Human 実装中は、以下を必須の acceptance criteria として扱います。

```text
- parse_machine_* の accepted / rejected syntax を変えない
- MachineTerm / MachineModule に Human 専用 variant を追加しない
- Machine Surface canonical bytes / canonical hash を変えない
- MachineTermElabContext の lookup を namespace / open / notation に依存させない
- npa-api の Phase 4 / 5 / 7 / 8 / 9 automation が Human Surface を経由しない
- Human metadata を certificate hash payload に入れない
- unresolved hole / meta を certificate に入れない
- Human resolver でも filesystem / network lookup を行わない
- Human overload resolution は有限候補だけを決定的順序で試す
- typeclass / coercion / backtracking-heavy search は Phase 3 Human MVP に入れない
- Machine Surface regression を Human 実装と同時に固定する
```

推奨する構成は次です。

```text
AI path:
  parse_machine_* -> resolve_machine_* -> elaborate_machine_* -> kernel / cert

Human path:
  parse_human_* -> resolve_human_* -> elaborate_human_* -> explicit core -> kernel / cert
```

Human path は最終的に fully explicit core AST を作ります。
kernel / certificate / independent checker の trusted base は広げません。

---

## 2. 実装順

Phase 3 Human は、Machine Surface の速度影響を検出しやすい順に実装します。

```text
1. Human 専用 module / API skeleton を作る
2. Machine Surface 不変テストを追加する
3. Human Surface AST と parser を追加する
4. FrontendState / namespace / open の resolver を追加する
5. explicit-only Human elaboration を Machine elaborator と分離して動かす
6. axiom / simple inductive の declaration coverage を広げる
7. implicit binder / implicit arg insertion / simple unification を追加する
8. notation / overload resolution を有限候補で追加する
9. holes / metavariable goal reporting を追加する
10. Phase 2 certificate handoff と API / regression を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-frontend
cargo test -p npa-api
./scripts/phase9-regression.sh
```

大きな内部変更の後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 3. タスク一覧

### P3H-00: Human / Machine frontend 境界を固定する

実装タスク:

- [x] `crates/npa-frontend` 内に Human 専用 module を追加する。
- [x] 例: `human.rs`, `human_parser.rs`, `human_resolver.rs`, `human_elaborator.rs`, `human_diagnostic.rs`。
- [x] `parse_machine_*` / `resolve_machine_*` / `elaborate_machine_*` は既存 signature と意味を維持する。
- [x] `MachineTerm` / `MachineModule` を Human AST として再利用しない。
- [x] Human path から Machine path への変換が必要な場合は、明示的な lowering API に限定する。

AI 速度ガード:

- [x] `parse_machine_module` が `open` / `namespace` / `notation` / `axiom` / `inductive` /
  hole を引き続き拒否する regression test を固定する。
- [x] `canonicalize_machine_term_source("@Eq.refl.{1} Nat n")` の canonical bytes /
  hash が Human 実装前後で変わらないことを固定する。
- [x] `npa-api` の Phase 7 / Phase 9 が Human parser を呼ばないことを grep 可能な形で保つ。

影響ファイル:

```text
crates/npa-frontend/src/lib.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
```

完了条件:

```text
- Human public API と Machine public API が名前で区別できる
- Machine Surface tests が変更なしで通る
- Human skeleton だけを追加した状態で phase9 regression が通る
```

### P3H-01: Human Surface AST を追加する

実装タスク:

- [x] `HumanModule` / `HumanItem` / `HumanDecl` / `HumanExpr` を定義する。
- [x] `HumanItem` に `Import` / `Open` / `NamespaceStart` / `NamespaceEnd` /
  `Def` / `Theorem` / `Axiom` / `Inductive` / `Notation` を持たせる。
- [x] `HumanExpr` に `Ident` / `App` / `Lam` / `Pi` / `Let` / `Annot` /
  `Sort` / `Arrow` / `Hole` / `ExplicitMode` / `NotationApp` 相当を持たせる。
- [x] `HumanBinder` に `BinderInfo::Explicit` / `BinderInfo::Implicit` を持たせる。
- [x] grouped binder を AST 上で保持するか、parser 直後に binder list へ展開する方針を固定する。
- [x] 全 node に `Span` を保持する。

AI 速度ガード:

- [x] Human AST を `MachineTerm` に variant 追加して表現しない。
- [x] Machine canonical encoder が Human AST を知る必要がない形にする。

影響ファイル:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/span.rs
```

完了条件:

```text
- `def id (A : Type) (x : A) : A := x` を Human AST にできる
- `{A : Type}` と `(A : Type)` の binder info が区別される
- `_` と `?m` が Human hole として保持される
- Machine parser の AST snapshot tests は変わらない
```

### P3H-02: Human parser entrypoint を実装する

実装タスク:

- [x] `parse_human_module(file_id, source)` を追加する。
- [x] `parse_human_term(file_id, source)` を追加する。
- [x] `import` は module 先頭に限定し、途中 import は `ImportAfterItem` にする。
- [x] `open` / `namespace` / `end` を parse する。
- [x] `def` / `theorem` / `axiom` / simple `inductive` を parse する。
- [x] `fun` / `forall` / `let` / annotation / application / parenthesized term を parse する。
- [x] `->` / `→` を右結合の anonymous Pi に desugar する。
- [x] grouped binder `(x y : A)` / `{x y : A}` を扱う。
- [x] `_` / `?m` を Human hole として扱う。
- [x] `@f` を explicit implicit argument mode として扱う。
- [x] `notation` / `infix` / `infixl` / `infixr` declaration を parse する。

AI 速度ガード:

- [x] Human parser は `parse_machine_*` から呼ばれない。
- [x] Machine lexer/parser に notation parser state を追加しない。
- [x] Machine parser の unsupported syntax rejection tests を残す。

影響ファイル:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/lib.rs
```

完了条件:

```text
- `doc/phase3-human.md` 12.1 から 12.15 の parser 部分を parse できる
- import-after-item を Human parser が構造化 diagnostic で拒否する
- Machine parser は同じ入力を従来通り拒否または受理する
```

### P3H-03: Human FrontendState / source interface metadata を実装する

実装タスク:

- [x] `HumanFrontendState` を追加する。
- [x] current module name / namespace stack / lexical open scopes を持たせる。
- [x] verified imports の source interface と source 内 import を照合する。
- [x] import interface に Human metadata を追加する場合、trusted certificate payload とは分離する。
- [x] notation table / binder metadata / generated declaration display info を source interface に入れる。
- [x] Human metadata は `npa-frontend` 側の source interface として扱い、`npa-cert`
  の canonical certificate 型には追加しない。
- [x] duplicate import の扱いを決定的にする。

AI 速度ガード:

- [x] `VerifiedImport` / `MachineGlobalScopeEntry` の AI path lookup に Human metadata lookup を足さない。
- [x] Human source interface metadata は certificate hash に入れない。
- [x] Human import resolution でも filesystem / network lookup を行わない。

影響ファイル:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/resolver.rs
```

完了条件:

```text
- import は compile request の verified import list と照合される
- Human metadata を捨てても certificate verify が成立する
- Machine verified import lookup の挙動が変わらない
```

### P3H-04: namespace / open 付き name resolution を実装する

実装タスク:

- [x] local context と global declaration table を分離する。
- [x] current module declaration と imported declaration の優先順位を固定する。
- [x] namespace stack から fully qualified declaration name を生成する。
- [x] `open` の lexical scope を実装する。
- [x] qualified name / unqualified name resolution を実装する。
- [x] ambiguous name を `AmbiguousName` として決定的 payload で返す。
- [x] forward reference を `ForwardReference` として拒否する。

AI 速度ガード:

- [x] Machine resolver は suffix lookup / open scope lookup を持たないままにする。
- [x] Human resolver の candidate collection は bounded vector で、決定的 sort key を持つ。

影響ファイル:

```text
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/lib.rs
```

完了条件:

```text
- namespace 内の `def Nat.zero` 相当が fully qualified name に解決される
- `open Std.Nat` 後に `zero` が `Std.Nat.zero` に解決される
- ambiguous unqualified name は silent に片方を選ばない
- Machine resolver の direct exact lookup tests が通る
```

### P3H-05: notation / infix table を実装する

実装タスク:

- [x] notation declaration を namespace / open scope と連動させる。
- [x] notation target を declaration 処理時に global ref へ解決する。
- [x] prefix / postfix / infix / infixl / infixr のうち Phase 3 MVP 対象を固定する。
- [x] infix precedence / associativity を parser binding power に反映する。
- [x] notation conflict を `NotationConflict` として拒否する。
- [x] non-associative infix chain を parse error にする。
- [x] overloaded notation candidates を決定的順序で保持する。
- [x] elaboration 中に candidate transaction / rollback を bounded に実装する。

AI 速度ガード:

- [x] Machine parser に active notation table を持たせない。
- [x] Human notation resolution は typeclass search に接続しない。
- [x] 候補数上限を option として持ち、超過時は `TooManyNotationCandidates` で拒否する。

影響ファイル:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/lib.rs
```

完了条件:

```text
- `infixl:65 " + " => Nat.add` を parse / register できる
- `n + Nat.zero` を `Nat.add n Nat.zero` 相当に elaboration できる
- notation conflict が deterministic diagnostic になる
- Machine Surface の `n + Nat.zero` は引き続き unsupported / parse error になる
```

### P3H-06: implicit binder / implicit argument insertion を実装する

実装タスク:

- [x] `{A : Type}` を implicit binder として source interface に残す。
- [x] callable profile に binder info を反映する。
- [x] implicit binder に対して `SyntheticImplicit` metavariable を挿入する。
- [x] `@f` mode では implicit term arg を自動挿入しない。
- [x] `@` 付き explicit implicit argument syntax を Human path で扱う。
- [x] unresolved implicit が declaration close 時に残る場合は certificate 化を拒否する。

AI 速度ガード:

- [x] Machine Surface の implicit argument required behavior を維持する。
- [x] `Eq.refl n` は Machine path では implicit 補完されない。
- [x] Human implicit insertion は bounded simple unification だけに接続する。

影響ファイル:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/callable.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/lib.rs
```

完了条件:

```text
- Human path で `Eq.refl n` が `@Eq.refl.{u} A n` 相当に解決できる
- Human path で `@Eq.refl` は implicit insertion を抑制する
- Machine path の callable profile tests は変わらない
```

### P3H-07: metavariable / hole / constraint store を実装する

実装タスク:

- [x] term metavariable store と universe metavariable store を分ける。
- [x] `_` と `?m` を `UserHole` meta に変換する。
- [x] named hole の context snapshot を保存する。
- [x] named hole reuse の context mismatch を `NamedHoleContextMismatch` にする。
- [x] `TypeEq` / `TermEq` / `LevelEq` / `LevelLe` constraints を表現する。
- [x] simple unification と occurs check を実装する。
- [x] unsolved meta / hole が残る declaration は incomplete として扱い、certificate 化を拒否する。
- [x] hole goal を人間向け diagnostic payload として返す。

AI 速度ガード:

- [x] Machine Surface Complete mode では hole を AST に入れない。
- [x] Repair mode の suggestion は trusted payload に入れない。
- [x] Human meta store を MachineTermElabContext に持ち込まない。

影響ファイル:

```text
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/lib.rs
```

完了条件:

```text
- Human path で `_` が goal diagnostic になる
- unresolved hole を含む declaration から certificate を作れない
- named hole context mismatch を検出できる
- Machine path の `_` は引き続き HoleNotAllowed
```

### P3H-08: bidirectional Human elaboration を実装する

実装タスク:

- [x] `infer_human_expr` / `check_human_expr` の skeleton を作る。
- [x] local / global / app / lambda / Pi / let / annotation を core term に落とす。
- [x] expected type を使って lambda / hole / notation candidate を処理する。
- [x] declaration elaboration 中は自分自身を global env に入れない。
- [x] elaborated core declaration を Phase 1 kernel に渡す。
- [x] kernel が拒否した場合は Human diagnostic に包む。

AI 速度ガード:

- [x] Machine elaborator に Human expected-type candidate search を追加しない。
- [x] Human elaboration の backtracking は notation / overload candidate 単位で bounded にする。
- [x] Phase 7 candidate check は引き続き Machine term check を使う。

影響ファイル:

```text
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/elaborator.rs
```

完了条件:

```text
- explicit-only Human def/theorem が core declaration になる
- ill-typed Human term が structured diagnostic になる
- Machine explicit elaboration tests が変わらない
```

### P3H-09: axiom declaration を実装する

実装タスク:

- [x] Human parser が `axiom name : type` を受理する。
- [x] Human resolver が axiom name を global scope に登録する。
- [x] Human elaborator が axiom type を core declaration に変換する。
- [x] Phase 2 certificate handoff で axiom report に反映されることを確認する。
- [x] axiom policy によって verify が拒否されるケースをテストする。

AI 速度ガード:

- [x] Machine parser は source-level `axiom` を引き続き拒否する。
- [x] AI path が axiom を source syntax から生成する経路を増やさない。

影響ファイル:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_elaborator.rs
```

完了条件:

```text
- Human source の axiom が certificate の axiom report に出る
- 未許可 axiom を含む cert verify が拒否される
- Machine Surface の source-level axiom rejection test が通る
```

### P3H-10: simple inductive declaration を実装する

実装タスク:

- [x] simple `inductive` syntax を Human AST にする。
- [x] self reference / forward reference の禁止範囲を固定する。
- [x] temporary global を使って constructor type を elaboration する。
- [x] constructor type から core-spec v0.1 の `InductiveDecl` を作る。
- [x] generated constructor / recursor を source interface に `LocalGenerated` として登録する。
- [x] inductive 全体を kernel に渡し、成功後だけ global env に登録する。

AI 速度ガード:

- [x] Machine parser は source-level `inductive` を引き続き拒否する。
- [x] generated display metadata は certificate hash に入れない。
- [x] Phase 9 advanced inductive automation とは別経路として保つ。

影響ファイル:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-kernel/src/lib.rs  # 既存 InductiveDecl support に不足がある場合だけ
```

完了条件:

```text
- `inductive Nat : Type where | zero : Nat | succ : forall (n : Nat), Nat` を core inductive にできる
- bad constructor type を kernel が拒否する
- generated declarations を Human resolver が参照できる
- Machine and Phase 9 regression が通る
```

### P3H-11: Human diagnostics / repair payload を実装する

実装タスク:

- [x] `HumanDiagnosticKind` を MachineDiagnostic と分ける。
- [x] parser / resolver / elaborator / kernel handoff の phase を payload に含める。
- [x] AmbiguousName / AmbiguousNotation / NotationConflict / ForwardReference /
  NamedHoleContextMismatch / UnsolvedMeta を表現する。
- [x] hole goal display 用の local context / expected type を payload に入れる。
- [x] human-facing message は deterministic payload から生成する。

AI 速度ガード:

- [x] MachineDiagnostic の canonicalization と Phase 7 failed candidate error payload を変えない。
- [x] Human diagnostic message の変更を Machine M9 same-output 対象にしない。

影響ファイル:

```text
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-api/src/adapter.rs
```

完了条件:

```text
- Human path 固有のエラーが MachineDiagnosticKind に混ざらない
- Machine API の error kind regression が通る
- hole goal diagnostic がテスト可能な構造化 payload を持つ
```

### P3H-12: Human API を追加し Machine API と分離する

実装タスク:

- [x] `parse_human_module` / `resolve_human_module` / `elaborate_human_module` /
  `compile_human_source_to_core` / `compile_human_source_to_certificate` を公開する。
- [x] API 名に `human` を含め、Machine API と呼び分けしやすくする。
- [x] `npa-api` に Human compile endpoint / library API を追加する場合、Machine endpoints とは別型にする。
- [x] Human API は verified import / current module input を明示的に受け取る。
- [x] hidden global state を作らない。

AI 速度ガード:

- [x] `/machine/*` 相当 API は Human compile path を呼ばない。
- [x] AI automation の default profile は Machine Surface のままにする。
- [x] Human compile options に expensive search を有効化する flag を追加しない。

影響ファイル:

```text
crates/npa-frontend/src/lib.rs
crates/npa-api/src/lib.rs
crates/npa-api/src/types.rs
```

完了条件:

```text
- Human API と Machine API の型が混ざらない
- Human source から cert を作れる
- Machine session / tactic / search / replay / verify APIs が既存 tests を維持する
```

### P3H-13: regression / performance guard を固定する

実装タスク:

- [x] Machine Surface accepted syntax snapshot tests を追加する。
- [x] Machine Surface rejected human-feature tests を追加する。
- [x] Machine term canonical hash fixture を追加または既存 fixture を固定する。
- [x] Human 実装後も `./scripts/phase9-regression.sh` が通ることを CI / docs に明記する。
- [x] Human path の notation / overload candidate count limit tests を追加する。
- [x] Human path の unsolved meta certificate rejection tests を追加する。

AI 速度ガード:

- [x] wall-clock timing の固定閾値は flake しやすいため置かない。
- [x] 代わりに same-input same-output、resource guard、candidate count bound を固定する。
- [x] Phase 7 / Phase 9 の existing fixtures を Human syntax に置き換えない。

影響ファイル:

```text
crates/npa-frontend/src/parser.rs
crates/npa-frontend/src/elaborator.rs
crates/npa-frontend/src/term_source.rs
crates/npa-api/src/phase7.rs
crates/npa-api/src/phase9.rs
scripts/phase9-regression.sh
```

完了条件:

```text
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- ./scripts/phase9-regression.sh
- Human implementation does not change Machine canonical term source hash
```

### P3H-14: docs / README を更新する

実装タスク:

- [x] `README.md` に Phase 3 Human と Phase 3 AI の実装状態を分けて記載する。
- [x] `doc/phase3-human.md` の設計と実装済み範囲がずれた場合は更新する。
- [x] `doc/phase3-ai.md` に Machine Surface 高速経路の非回帰条件を維持する。
- [x] Phase 4 以降の docs が Human Surface を AI path の前提にしていないか確認する。

AI 速度ガード:

- [x] docs で AI 向け候補生成に Human Surface を使うような記述を入れない。
- [x] trusted boundary は README / docs で kernel / canonical certificate / independent checker に限定する。

影響ファイル:

```text
README.md
doc/phase3-human.md
doc/phase3-ai.md
doc/phase4-ai.md
doc/phase5-ai.md
doc/phase7-ai.md
```

完了条件:

```text
- Phase 3 Human は未信頼 convenience layer として説明されている
- Phase 3 AI は Machine Surface の explicit fast path として説明されている
- trusted base が広がったように読める記述がない
```

---

## 4. Phase 3 Human MVP の対象外

Phase 3 Human MVP では、以下を実装しません。

```text
full typeclass resolution
coercion search
instance search
macro system
user-defined syntax extension
multi-token / mixfix notation
tactic blocks
pattern matching elaboration
do notation
structure projection notation
term-level numeric literal overload
aliases
absolute global name syntax
source-level recursive definition syntax
reducibility annotations
termination checking
mutual declarations
automatic universe generalization
sophisticated universe minimization
SMT fallback
AI fallback from Human elaboration
unbounded proof search
```

これらは便利ですが、Human elaborator の探索空間を広げ、AI 向け Machine Surface の
速度・決定性・説明可能性を損ないやすい機能です。
必要になった場合も、Machine Surface とは別 profile として設計します。

---

## 5. 完了判定

Phase 3 Human 実装が完了したと言える条件は次です。

```text
- Human source で import/open/namespace/end を扱える
- Human source で def/theorem/axiom/simple inductive を扱える
- grouped binder / arrow / implicit binder / holes を扱える
- notation declaration と simple infix を扱える
- name / notation ambiguity を deterministic diagnostic にできる
- implicit args と universe metas を simple unification で解決できる
- unresolved hole / meta を certificate 化前に拒否できる
- solved Human declaration は fully explicit core AST になる
- Phase 1 kernel check に通った declaration だけを採用する
- Phase 2 certificate に Human metadata を入れずに渡せる
- Machine Surface accepted / rejected syntax と canonical hash が変わらない
- Phase 4 / 5 / 7 / 8 / 9 の AI automation が Machine Surface fast path を維持する
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- ./scripts/phase9-regression.sh
```

一文でまとめると、Phase 3 Human は人間向け syntax convenience layer です。
証明の受理根拠は引き続き canonical core AST、Phase 1 Rust kernel、
Phase 2 certificate verifier、Phase 8 independent checker に限定します。
