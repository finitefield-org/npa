# Phase 4 Human Task Breakdown

このタスク分解は `doc/phase4-human.md` を正とし、現在の
`crates/npa-frontend` / `crates/npa-tactic` / `crates/npa-api` 実装との差分を
実装マイルストーンに分けたものです。

Phase 4 Human は、人間が `by ...` block と tactic script を書くための層です。
AI 向けの高速・構造化された Machine Tactic は `doc/phase4-ai.md` の責務であり、
Human parser / resolver / notation / implicit insertion を AI fast path に混ぜてはいけません。

重要な制約:

```text
- Human tactic parser は `crates/npa-tactic` の Machine tactic hot path に入れない。
- `MachineTacticCandidate`, `MachineTactic`, `MachineTermSource` に Human 専用 metadata を追加しない。
- `parse_machine_*` と `canonicalize_machine_term_source` は Human syntax を受け付けるように拡張しない。
- Phase 5 / Phase 7 / Phase 9 の Machine API は Human source を経由しない。
- tactic は信用せず、最終的な proof term / certificate を kernel と verifier が検査する。
- Human bridge に I/O、network、plugin loading、AI 呼び出し、探索用 import lookup を入れない。
```

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在の `crates/npa-frontend` には、Phase 3 Human Surface の基盤があります。
Phase 4 Human 実装では、これらを前提として使ってよいです。

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanExpr
- HumanFrontendState
- HumanSourceInterface / HumanImportedSourceInterface
- HumanExpr::Hole
- HumanCompileOptions

crates/npa-frontend/src/human_parser.rs
- parse_human_module
- parse_human_module_with_source_interfaces
- parse_human_term
- import / open / namespace / end
- def / theorem / axiom / inductive / notation
- grouped binders, implicit binders, holes, notation application

crates/npa-frontend/src/human_resolver.rs
- namespace / open scope resolution
- source interface reconciliation
- imported Human metadata lookup
- ambiguity and forward-reference diagnostics

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core
- compile_human_source_to_certificate
- Human term elaboration with implicit insertion / simple metas / holes
- certificate handoff that rejects unresolved holes

crates/npa-api/src/human.rs
- Human compile-to-core API wrapper
- Human compile-to-certificate API wrapper
- Machine session API remains Machine Surface only
```

重要な依存関係:

```text
npa-tactic -> npa-frontend
npa-api -> npa-frontend + npa-tactic
```

したがって、`crates/npa-frontend` から `npa-tactic` を直接呼ぶ実装は循環依存になります。
Human tactic bridge は `crates/npa-api` 内の Human-only adapter、または
`npa-frontend` と `npa-tactic` の両方に依存する新規 adapter crate に置きます。
`crates/npa-frontend` は Human AST / parser / resolver / term elaboration helper を提供し、
proof-state 実行は adapter 層で行います。

現在の `crates/npa-tactic` には、AI 向け Machine tactic の proof-state core と
6つの tactic が実装済みです。

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- start_machine_proof
- validate_machine_tactic_candidate
- run_machine_tactic_with_budget
- run_machine_tactic_candidates_batch
- assign_goal / ProofExpr
- extract_closed_machine_proof
- extract_closed_machine_theorem_decl
- extract_closed_machine_core_module
- extract_closed_machine_certificate
- exact / intro / apply / rw / simp-lite / induction-nat
- deterministic budget / tactic hash / cache key
```

### 0.2 未実装の Human tactic 範囲

`doc/phase4-human.md` が要求する以下の範囲は、現在のコードには Human Profile として存在しません。

```text
HumanTacticScript
HumanTacticSyntax
HumanRewriteRuleSyntax
HumanProofBlock / by block AST
`by ...` を theorem value として parse する構文
intro / exact / apply / rw / simp-lite / induction の Human parser
Human tactic script executor
Human proof state bridge
Human source term を tactic goal context で elaborate する bridge
Human `exact` から goal を閉じる実装
Human `intro` から binder local を導入する実装
Human `apply` から subgoal を生成する実装
Human `rw` から target rewrite を行う実装
Human `simp-lite` から proof-producing simplifier を呼ぶ実装
Human `induction n` から Nat.rec base/step goal を作る実装
Human-facing tactic diagnostics / goal display
Human API から by proof を含む theorem を compile / certificate 化する regression
```

### 0.3 Machine fast path に入れてはいけないもの

以下は Human tactic 実装で使ってよいが、Machine tactic の高頻度候補検査経路に入れてはいけません。

```text
Human tactic text parser
Human `by` proof block AST
Human name shortening
open / namespace scope
notation table
implicit argument insertion
hole / named hole metadata
Human source spans
Human diagnostics
Human goal pretty text
case syntax
backtracking tactic language
filesystem / network import lookup
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 4 Human の各マイルストーンでは、次を acceptance criteria として扱います。

```text
- `crates/npa-tactic` の public Machine API は Human parser を呼ばない。
- `/machine/tactics/run` と `/machine/tactics/batch` は `MachineTacticCandidate` だけを主入力にする。
- Human `by` parser は `crates/npa-frontend` または Human-only adapter に閉じる。
- `crates/npa-frontend` は `npa-tactic` に依存しない。proof-state bridge は `npa-api` または新規 adapter crate に置く。
- Human term を Machine Surface source に pretty-print して再parseする実装を既定にしない。
- Human term は Human elaborator で core Expr に落とすか、明示的に Machine-compatible な形へ変換する。
- Machine Surface accepted / rejected syntax と canonical hash を変えない。
- tactic metadata、source span、Human display name は certificate に入れない。
- Human tactic bridge の追加で Phase 7 / Phase 9 regression fixture が変わらない。
```

推奨する構成:

```text
Human source:
  parse_human_module
  -> Human theorem with HumanProofBlock
  -> Human resolver / Human elaborator
  -> Human-only tactic adapter outside npa-frontend
  -> npa_tactic proof-state primitive or MachineTactic where safe
  -> extract closed core proof
  -> kernel / certificate

AI path:
  MachineTacticCandidate JSON
  -> validate_machine_tactic_candidate
  -> run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
  -> extract closed core proof
  -> kernel / certificate
```

---

## 2. 実装順

Phase 4 Human は、Machine fast path への影響を検出しやすい順に実装します。

```text
1. Human / Machine tactic 境界と regression guard を固定する
2. Human tactic AST と by block parser を追加する
3. Human proof-state bridge の skeleton を作る
4. exact / intro で最小 proof script を閉じる
5. apply で subgoal generation を通す
6. rw / simp-lite で Eq target rewrite と簡約を通す
7. induction で Nat.rec base/step goal を通す
8. script executor / diagnostics / API / certificate handoff を統合する
9. Machine fast path regression と doc consistency を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib human_parser
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-frontend --lib machine_surface
cargo test -p npa-tactic
cargo test -p npa-api phase7
cargo test -p npa-api phase9
```

大きな内部変更後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 3. タスク一覧

### P4H-00: Human / Machine tactic 境界を固定する

Status: Done

Depends on: None

Inputs:

```text
doc/phase4-human.md section 0
doc/phase4-ai.md section 2, 3, 6, 13
doc/phase5-ai.md MachineTacticCandidate wire schema
crates/npa-tactic/src/lib.rs
crates/npa-api/src/tactic.rs
crates/npa-api/src/phase7.rs
crates/npa-api/src/phase9.rs
```

実装タスク:

- Human tactic 実装の entrypoint を `crates/npa-api` 内 adapter または新規 adapter crate に置く方針を固定する。
- `crates/npa-frontend` に `npa-tactic` dependency を追加しない regression を固定する。
- `crates/npa-tactic` の Machine API に Human parser dependency を追加しない regression test を置く。
- `parse_machine_*` / `canonicalize_machine_term_source` の Human syntax rejected snapshot を維持する。
- Phase 7 / Phase 9 が `MachineTacticCandidate` と Machine Surface canonicalization を使い続けることをテスト名で固定する。

AI 速度ガード:

- `rg -n "parse_human|compile_human|Human" crates/npa-tactic/src/lib.rs crates/npa-api/src/tactic.rs crates/npa-api/src/adapter.rs` が production hit を持たない。
- `cargo test -p npa-frontend --lib machine_surface` が通る。
- `cargo test -p npa-api phase7` と `cargo test -p npa-api phase9` が通る。

完了条件:

- Human tactic bridge の実装場所と Machine API 非依存境界がコード上で明確である。
- Machine candidate validation / batch 実行に Human parser fallback が存在しない。
- Machine Surface canonical bytes / hash の golden behavior が変わっていない。

完了確認:

- `npa-api` の Human API test で、Human tactic bridge は `npa-api` または新規 adapter crate に置き、`npa-frontend -> npa-tactic` 依存を追加しない境界を固定した。
- `npa-api` の Machine tactic API test と `npa-tactic` test で、Machine hot path に Human parser / compiler fallback marker が混入していないことを固定した。
- 既存の Machine Surface rejected syntax snapshot と Phase 7 / Phase 9 regression を維持する。

### P4H-01: Human tactic AST を追加する

Status: Done

Depends on: P4H-00

Inputs:

```text
doc/phase4-human.md section 10, 12
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_diagnostic.rs
```

実装タスク:

- `HumanProofBlock` または同等の型を追加し、`by` block を theorem value と区別して保持する。
- `HumanTacticScript` / `HumanTacticSyntax` を追加する。
- tactic variant は `Intro`, `Exact`, `Apply`, `Rewrite`, `SimpLite`, `Induction` に限定する。
- `rw` 用に forward / backward direction と rule term list を表す Human AST を追加する。
- source span を保持するが、hash / certificate payload へ入れない。
- case syntax、constructor、refine、have、calc は AST に入れない。

AI 速度ガード:

- `MachineTacticCandidate` / `MachineTactic` に Human variant や Human span field を追加しない。
- `crates/npa-tactic` に `HumanTacticSyntax` を import しない。

完了条件:

- Human tactic script を typed AST として表現できる。
- Human AST と Machine tactic AST が型レベルで分離されている。
- unsupported tactic は parse/diagnostic 上で明確に拒否できる。

完了確認:

- `npa-frontend` に `HumanDeclValue::{Term, ProofBlock}`、`HumanProofBlock`、`HumanTacticScript`、`HumanTacticSyntax` を追加し、`by` block を通常 term value と型レベルで分離した。
- tactic AST は `intro` / `exact` / `apply` / `rw` / `simp-lite` / `induction` の MVP 6種類だけを表し、`rw` rule は forward / backward direction と rule term span を保持する。
- `HumanDiagnosticKind::UnsupportedTactic` を追加し、P4H-01 時点では proof block resolver が tactic bridge 未実装を構造化 diagnostic として拒否する。
- `MachineTacticCandidate` / `MachineTactic` と `npa-tactic` は変更していない。

### P4H-02: `by` proof block parser を実装する

Status: Done

Depends on: P4H-01

Inputs:

```text
doc/phase4-human.md section 10
crates/npa-frontend/src/human_parser.rs
```

実装タスク:

- Human lexer に `by`, `intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction` を追加する。
- theorem declaration の `:=` 右辺として term または `by` proof block を parse できるようにする。
- `intro ident`, `exact term`, `apply term`, `rw [rule]`, `rw [<- rule]`, `simp-lite`, `induction ident` を parse する。
- 複数 tactic は source order を保持する。
- indentation は MVP では semantic に扱わず、token sequence として deterministic に読む。
- `case zero =>` / `case succ =>` は Phase 4 Human MVP では拒否する。

AI 速度ガード:

- Machine parser に `by` / tactic keyword を追加しない。
- Machine Surface rejected syntax fixture に `by`, `intro`, `rw [h]`, `simp-lite`, `induction n` を含める。

完了条件:

- `theorem id_nat : Nat -> Nat := by intro n exact n` を Human AST に parse できる。
- unsupported tactic / malformed `rw` / trailing tactic token は Human parse diagnostic になる。
- Machine parser は同じ入力を拒否し続ける。

完了確認:

- Human parser に `by` proof block parser を追加し、theorem の `:=` 右辺で `HumanDeclValue::ProofBlock` を生成する。
- `intro ident` / `exact term` / `apply term` / `rw [rule]` / `rw [<- rule]` / `simp-lite` / `induction ident` を source order の `HumanTacticScript` として parse する。
- MVP では indentation を semantic に扱わず、`case` / unsupported tactic / malformed `rw` / trailing token を parser-phase diagnostic として拒否する。
- Machine Surface rejected syntax fixture に `by intro ... exact ...`、`rw [h]`、`simp-lite`、`induction n` を含め、Machine parser 側へ tactic keyword は追加していない。

### P4H-03: Human proof-state bridge skeleton を作る

Status: Done

Depends on: P4H-02

Inputs:

```text
doc/phase4-human.md section 1, 2, 3, 11
crates/npa-frontend/src/human_elaborator.rs
crates/npa-api/src/human.rs
crates/npa-tactic/src/lib.rs
```

実装タスク:

- bridge の配置を `crates/npa-api` 内 Human-only module か新規 adapter crate に決める。
- `crates/npa-frontend` には `npa-tactic` dependency を追加しない。
- 必要であれば、`npa-frontend` から Human theorem type / term を core Expr に落とすための小さな public helper だけを追加する。
- Human theorem type を先に elaboration し、`MachineProofSpec` へ渡せる fully explicit core type を作る。
- prior current declarations を `check_current_decl_for_machine_tactic` で checked chain にする。
- verified imports / source interfaces から `VerifiedImportRef` と Human lookup context を組み立てる。
- `start_machine_proof` を呼ぶ Human-only bridge を追加する。
- Human bridge 用 options を追加する場合は `HumanCompileOptions` に閉じ、`MachineTacticOptions` は既存 canonical rule を使う。
- Human bridge は filesystem / network / package registry lookup を行わない。

AI 速度ガード:

- `start_machine_proof` の signature と validation order を Human 都合で変更しない。
- Human bridge から Machine batch API を呼ぶ場合も、AI の batch hot pathには逆依存させない。
- workspace dependency graph に `npa-frontend -> npa-tactic` が生えない。

完了条件:

- Human theorem の proof state を決定的に開始できる。
- root goal の theorem type が kernel で Sort として検査される。
- Human bridge が作った state fingerprint は Machine state validation を通る。
- `cargo metadata` または `Cargo.toml` inspection で循環依存がない。

完了確認:

- bridge は `crates/npa-api/src/human.rs` の Human-only API `start_human_proof` として配置し、`crates/npa-frontend` には `npa-tactic` dependency を追加していない。
- `npa-frontend` には `prepare_human_proof_start_core_with_source_interfaces` だけを追加し、Human source の target theorem type と prior current declarations を core 化する。tactic 実行や `npa-tactic` 呼び出しは行わない。
- Human current declaration names は bridge 境界で Machine proof state 用に current module prefix 付きへ射影し、imported names と Machine Surface canonicalizer は変更していない。
- `start_human_proof` は active Human imports / source interfaces から `VerifiedImportRef` を組み立て、prior current declarations を `check_current_decl_for_machine_tactic_from_verified_imports` で checked chain にしてから `start_machine_proof` を呼ぶ。
- 生成された proof state は `validate_machine_proof_state` を通し、root goal theorem type は既存 `start_machine_proof` の kernel Sort check に任せる。
- AI 速度ガードとして、`start_machine_proof` の signature / validation order、Machine batch API、`npa-tactic` hot path、Machine Surface parser は変更していない。

### P4H-04: Human tactic term elaboration context を実装する

Status: Done

Depends on: P4H-03

Inputs:

```text
doc/phase4-human.md section 5, 6, 7
crates/npa-frontend/src/human_elaborator.rs
crates/npa-tactic/src/lib.rs
```

実装タスク:

- 現在 goal の local context を Human term elaborator が読める形へ変換する。
- Human source term を goal target に対して check / infer できる helper を追加する。
- local binder、checked current declaration、verified import、generated constructor / recursor を解決できるようにする。
- Human term elaboration の unsolved hole / synthetic implicit は tactic failure として certificate 前に拒否する。
- Human term を pretty string 化して Machine Surface に再投入する実装を既定にしない。

AI 速度ガード:

- Machine term elaboration context に Human open scope / notation lookup を追加しない。
- Human context conversion の結果を Machine tactic cache key に入れない。

完了条件:

- tactic 内の `exact x` が goal context の local `x` を解決できる。
- tactic 内の `exact Eq.refl n` が Human implicit insertion を使える。
- unresolved hole は Human diagnostic として返り、certificate 化されない。

### P4H-05: `exact` を実装する

Status: Done

Depends on: P4H-04

Inputs:

```text
doc/phase4-human.md section 5, 13.1, 13.2
crates/npa-tactic/src/lib.rs assign_goal / ProofExpr
```

実装タスク:

- `exact term` を現在 goal の target に対して Human elaborator で check する。
- check 済み core Expr を `ProofExpr::Core` として `assign_goal` または同等の proof-state primitive に渡す。
- `exact _` や unresolved meta を含む exact は失敗させる。
- closed goal を open_goals から取り除く。

AI 速度ガード:

- `MachineTactic::Exact` と `RawMachineTerm` の semantics を変更しない。
- Human exact のために `MachineTermSource` が Human syntax を受け付けるようにしない。

完了条件:

- `theorem id_nat : Nat -> Nat := by intro n exact n` の exact 部分が local を使って goal を閉じる。
- `theorem self_eq (n : Nat) : n = n := by exact Eq.refl n` が閉じる。
- `exact _` は conservative に拒否される。

### P4H-06: `intro` を実装する

Status: Done

Depends on: P4H-03

Inputs:

```text
doc/phase4-human.md section 4, 13.1
crates/npa-tactic/src/lib.rs run_machine_tactic_with_budget
```

実装タスク:

- `intro name` を現在 goal に適用する。
- Machine `Intro` を再利用できる場合は Human bridge から `MachineTactic::Intro` を呼ぶ。
- target が Pi / forall でない場合は Human-facing diagnostic に写像する。
- local name shadowing / invalid binder name は deterministic に拒否する。

AI 速度ガード:

- `MachineTactic::Intro` の candidate hash / proof delta hash を変更しない。
- Human display name は state fingerprint に入れない。

完了条件:

- `intro n` が `Nat -> Nat` の body goal を作る。
- `intro` が使えない target では structured error が返る。
- `intro` 後の exact と組み合わせて `id_nat` が閉じる。

### P4H-07: Human tactic script executor を実装する

Status: Done

Depends on: P4H-05, P4H-06

Inputs:

```text
doc/phase4-human.md section 10, 11
crates/npa-frontend/src/human_elaborator.rs
```

実装タスク:

- `HumanTacticScript` を source order で逐次実行する。
- tactic は常に先頭 open goal に適用する。
- goal がない状態で tactic が残る場合は `NoGoalsButTacticRemaining` 相当の Human diagnostic を返す。
- script 終了時に open goal が残る場合は unresolved goal diagnostic を返す。
- closed proof を `extract_closed_machine_proof` または同等の API で取り出す。

AI 速度ガード:

- Machine batch execution policy は変更しない。
- Human sequential executor は `/machine/tactics/batch` の実装に入れない。

完了条件:

- `intro` + `exact` の2行 script を閉じられる。
- 余分な tactic / 未解決 goal の両方が Human diagnostic として区別される。
- extracted proof term は kernel check を通る。

### P4H-08: `apply` を実装する

Status: Done

Depends on: P4H-04, P4H-07

Inputs:

```text
doc/phase4-human.md section 6, 13.3
crates/npa-tactic/src/lib.rs run_apply_tactic_with_budget behavior
```

実装タスク:

- `apply term` の MVP では、resolved local / global head を扱う。
- Human name resolution で head を解決し、Machine `TacticHead` または Human-only apply helper に落とす。
- implicit / inferable arguments は subgoal にせず、明示的・証明 relevant な前提を subgoal にする。
- target と conclusion が unify できない場合は structured diagnostic を返す。
- arbitrary term head や complex expression apply は MVP 外として明示的に拒否するか、別タスク化する。

AI 速度ガード:

- Machine `Apply` candidate schema を Human term expression 用に広げない。
- Human apply の name resolution を Machine tactic head lookup に混ぜない。

完了条件:

- local assumption または checked theorem を `apply` して subgoal を作れる。
- `apply` 後の subgoal を後続 `exact` で閉じられる。
- apply 失敗時に target / head type の要約を Human diagnostic として返せる。

### P4H-09: `rw` を実装する

Status: Done

Depends on: P4H-04, P4H-07

Inputs:

```text
doc/phase4-human.md section 7, 13.4
crates/npa-tactic/src/lib.rs rewrite implementation
```

実装タスク:

- `rw [h]` と `rw [<- h]` を parse 済み Human rule から実行する。
- MVP は target のみを書き換える。
- Eq のみ対応し、setoid rewrite / hypothesis rewrite / occurrence selection は拒否する。
- rule head を local / checked theorem から解決する。
- rewrite 後の target goal と Eq.rec / Eq.subst transport proof を生成する。

AI 速度ガード:

- Machine `Rewrite` の `RewriteSite` / `RewriteDirection` semantics を変更しない。
- Human notation-based rule lookup は Machine simp/rw registry に混ぜない。

完了条件:

- `rw [h]` が target 内の Eq side を書き換えられる。
- reverse rewrite が deterministic に動く。
- dependent rewrite や unsupported site は Human diagnostic で拒否される。

### P4H-10: `simp-lite` を実装する

Status: Done

Depends on: P4H-07

Inputs:

```text
doc/phase4-human.md section 8, 13.5
crates/npa-tactic/src/lib.rs simp-lite implementation
```

実装タスク:

- `simp-lite` を現在 goal に適用する。
- Phase 4 Human MVP では、既存の Machine `SimpLite` と state の `SimpRegistry` を再利用する。
- Human source-level simp attribute / custom simp set は追加しない。
- 閉じられない場合は失敗させる方針を MVP として固定する。
- rewrite limit / target hash revisit / max steps は existing `MachineTacticOptions` に従う。

AI 速度ガード:

- `MachineTacticOptions.simp_rules` canonical bytes を変更しない。
- Human notation table から implicit に simp rules を追加しない。

完了条件:

- `simp-lite` が reflexive Eq target を閉じられる。
- registered safe rule を使って target を簡約し、proof-producing chain を作れる。
- max rewrite steps による deterministic failure が保たれる。

### P4H-11: `induction` を実装する

Status: Pending

Depends on: P4H-07, P4H-10

Inputs:

```text
doc/phase4-human.md section 9, 13.6
crates/npa-tactic/src/lib.rs induction-nat implementation
```

実装タスク:

- Human syntax `induction n` を Nat 限定の Machine `InductionNat` または同等の helper に変換する。
- 対象は local context に直接ある Nat local に限定する。
- induction target より後ろに依存仮定がある場合は失敗させる。
- base goal / step goal を deterministic order で生成する。
- `case zero =>` / `case succ =>` は MVP 外として拒否する。

AI 速度ガード:

- Machine wire name は `induction-nat` のまま維持する。
- Human `induction` keyword を Machine API allowed_tactics に追加しない。

完了条件:

- `induction n` が base / step の2 goal を作る。
- base と step を後続 `exact` / `simp-lite` で閉じられる。
- unsupported dependent induction は Human diagnostic で拒否される。

### P4H-12: by proof の core / certificate handoff を統合する

Status: Pending

Depends on: P4H-07, P4H-08, P4H-09, P4H-10, P4H-11

Inputs:

```text
doc/phase4-human.md section 11, 15
crates/npa-frontend/src/human_elaborator.rs
crates/npa-cert/src/canonical.rs
```

実装タスク:

- theorem value が Human term の場合と Human `by` proof の場合を compile path で分岐する。
- `by` proof から extracted proof Expr を theorem declaration value として入れる。
- `npa-frontend` 単体の compile path が by proof を扱えない段階では、明示的な diagnostic で adapter path へ誘導する。
- proof state / tactic trace / Human spans / diagnostics を CoreModule と certificate payload へ入れない。
- unresolved goal が1つでもあれば certificate 化を拒否する。
- closed proof は既存 kernel / certificate verifier で再検査する。

AI 速度ガード:

- certificate canonical encoding に Human tactic metadata field を追加しない。
- Machine certificate handoff fixture を変更しない。

完了条件:

- Human tactic adapter の compile-to-core path が `by` theorem を core theorem declaration に変換できる。
- Human tactic adapter の compile-to-certificate path が `by` theorem を含む certificate を生成し、verify できる。
- 既存の `npa_frontend::compile_human_source_to_core` / `compile_human_source_to_certificate` を変更する場合は、循環依存なしで実現している。
- unresolved goal / unsupported tactic は certificate construction より前に止まる。

### P4H-13: Human tactic diagnostics と goal display を整える

Status: Pending

Depends on: P4H-07

Inputs:

```text
doc/phase4-human.md section 3, 4, 5, 6, 9
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-api/src/human.rs
```

実装タスク:

- tactic parse / validation / execution / unresolved goal を HumanDiagnosticPhase で区別する。
- current goal の context / target を Human-friendly payload として返す。
- MachineTacticDiagnostic を Human diagnostic に写像する helper を追加する。
- error message は人間向けにしてよいが、判定は enum / structured payload で行う。
- diagnostics payload に tactic trace や AI metadata を含めない。

AI 速度ガード:

- MachineApiDiagnostic canonicalization を Human diagnostic で変更しない。
- Human goal display は Machine state fingerprint / tactic cache key に入れない。

完了条件:

- `intro` non-Pi、`exact` type mismatch、`apply` mismatch、unsupported induction の診断が区別される。
- tests が diagnostic kind と payload を直接 assert できる。
- display text の変更で hash / replay が変わらない。

### P4H-14: Human API から by proof を使えるようにする

Status: Pending

Depends on: P4H-12, P4H-13

Inputs:

```text
doc/phase4-human.md section 3, 11, 15
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

実装タスク:

- Human API wrapper で by proof を通す。実装先は `npa-api` adapter または新規 adapter crate の public API とする。
- 既存の plain Human compile wrapper との責務分担を明記する。
- Human API request shape は current module/source/imports/options の明示入力を維持する。
- Human API から Machine session を暗黙作成しない。
- API tests に `by intro n exact n` と `by exact Eq.refl n` を追加する。

AI 速度ガード:

- `/machine/*` endpoint の request grammar を変更しない。
- `create_machine_session` は `by` source を引き続き Machine term parse error として拒否する。

完了条件:

- Human API が by proof を含む source から certificate を作れる。
- Machine API が Human tactic text を受け付けない regression が通る。
- imported Human source interface と by proof が同時に使える。

### P4H-15: 最小 examples と regression fixture を固定する

Status: Pending

Depends on: P4H-14

Inputs:

```text
doc/phase4-human.md section 13, 14, 15
README.md
doc/phase4-ai.md
```

実装タスク:

- `intro` + `exact` の id theorem fixture を追加する。
- `Eq.refl` exact fixture を追加する。
- `apply` fixture を追加する。
- `rw` fixture を追加する。
- `simp-lite` fixture を追加する。
- `induction` fixture を追加する。
- unsupported features list の regression を追加する。
- README または docs に Phase 4 Human の実装状態を反映する必要があれば更新する。

AI 速度ガード:

- Human examples の追加で Phase 7 / Phase 9 fixture hash が変わらないことを確認する。
- Machine Surface rejected Human feature snapshot を更新しない、または意図した拒否だけを追加する。

完了条件:

- `doc/phase4-human.md` section 13 の最小テストがコード上の regression として存在する。
- section 14 の「まだ入れないもの」は parser / diagnostic で拒否される。
- Phase 4 Human 完了条件を満たすテスト群がある。

---

## 4. 完了ゲート

Phase 4 Human 全体が完了したと言える条件:

```text
- Human `by` proof block を parse できる。
- Human tactic script を順に実行できる。
- intro / exact / apply / rw / simp-lite / induction が MVP 範囲で動く。
- tactic 後の proof term を kernel が検査できる。
- unresolved goal が残った場合は certificate 化を拒否できる。
- Human tactic parser / bridge が AI 向け Machine tactic API の hot path に入っていない。
- Phase 7 / Phase 9 が Human source を経由せず Machine API を使い続ける。
- Machine tactic canonical hash / cache key / budget が Human syntax 追加で変わらない。
```

推奨する最終確認:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p npa-frontend
cargo test -p npa-tactic
cargo test -p npa-api
./scripts/phase9-regression.sh
cargo test --workspace
```

`./scripts/phase9-regression.sh` が存在しない環境では、代わりに Phase 7 / Phase 9 の
targeted regression を必ず実行します。
