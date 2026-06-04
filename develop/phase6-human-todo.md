# Phase 6 Human Task Breakdown

このタスク分解は `develop/phase6-human.md` を正とし、現在の
`crates/npa-frontend` / `crates/npa-tactic` / `crates/npa-api` / `crates/npa-cert`
実装との差分を、標準ライブラリ source 実装のマイルストーンに分けたものです。

Phase 6 Human は、人間が読み書きできる小さく堅い標準ライブラリを作り、その source から
canonical certificate と検索用 metadata を生成するための非信頼層です。
AI 向けの release manifest、import bundle、Machine theorem index、simp / rewrite profile の wire contract は
`develop/phase6-ai.md` の責務であり、Human source layout、notation、pretty statement、Human-facing 属性表を
AI hot path に混ぜてはいけません。

重要な制約:

```text
- 標準ライブラリ source text、notation、Human-facing 属性は trusted base に入れない。
- 証明の受理根拠は canonical certificate と kernel / verifier / independent checker の結果だけにする。
- Phase 6 release module は exactly Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic にする。
- Core / prelude は Phase 2 ImportEntry として emitted certificate に入れない。
- Std.Algebra.Basic は Std.Nat に依存させない。Nat 用 algebra instance は release module ではなく将来の別 module または test fixture に置く。
- Std.Nat.Algebra、Std.Classical、typeclass、full simp、ring / omega / linarith、overloaded numerals は MVP に入れない。
- Eq.rec を kernel-standard AxiomDecl として表現する場合だけ、exact Std.Logic.Eq.rec 例外を axiom report / allowlist に出す。
- kernel crate に I/O、network、plugin loading、AI 呼び出し、standard-library package resolver を入れない。
```

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在の `crates/npa-frontend` には、Human Surface の source parser / resolver / elaborator があります。
Phase 6 Human 実装では、これらを source 入力のために使ってよいです。

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanDeclValue
- HumanProofBlock / HumanTacticScript / HumanTacticSyntax
- HumanSourceInterface / HumanImportedSourceInterface
- HumanDiagnostic / HumanDiagnosticPayload / HumanHoleGoal

crates/npa-frontend/src/human_parser.rs
- parse_human_module / parse_human_term
- import / open / namespace / notation / def / theorem / axiom / inductive
- by proof block と intro / exact / apply / rw / simp-lite / induction の parser

crates/npa-frontend/src/human_resolver.rs
- namespace / open scope resolution
- imported Human metadata lookup
- ambiguity and forward-reference diagnostics

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core / compile_human_source_to_certificate
- Human term elaboration with implicit insertion / simple metas / holes
- certificate handoff that rejects unresolved holes
```

現在の `crates/npa-tactic` には、標準ライブラリ証明で使う proof-state primitive があります。

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- intro / exact / apply / rw / simp-lite / induction-nat
- run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
- extract_closed_machine_proof / extract_closed_machine_certificate
- deterministic budget hash / tactic cache key / proof delta
```

現在の `crates/npa-api` には、Phase 6 AI Profile の machine artifact 実装があります。
これは Human source 実装の出力を受け取る側であり、Human source layout の正本ではありません。

```text
crates/npa-api/src/std_library.rs
- load_machine_std_mvp_certificates
- load_machine_std_mvp_release
- generate_machine_std_mvp_import_bundle_set
- generate_machine_std_mvp_theorem_index
- generate_machine_std_mvp_rewrite_profile_set
- generate_machine_std_mvp_simp_profile_set
- generate_machine_std_mvp_final_theorem_index
- validate_machine_std_mvp_* validators
```

### 0.2 未実装の Phase 6 Human 範囲

`develop/phase6-human.md` が要求する以下の範囲は、現在のコードには standard-library source package として存在しません。

```text
Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic source package
Std.Logic certificate with Eq / connectives / Exists
Std.Nat certificate with Nat / add / mul / pred and basic theorems
Std.List certificate with List / append / length / map / foldr and basic theorems
Std.Algebra.Basic certificate with explicit algebraic property definitions
source package build pipeline from Human source to raw .npcert bytes
per-module Human debug views: index / axiom report / dependency graph
Phase 4 tactic regression over real standard-library certificates
handoff from real standard-library certificates to Phase 6 AI machine release loader
```

既存の frontend / API tests には `Std.Nat.Basic` や `Std.Logic.Eq` という古い fixture module name が残っています。
これらは test-only compatibility fixture として扱い、Phase 6 MVP release module name として公開してはいけません。

### 0.3 AI fast path に入れてはいけないもの

以下は Phase 6 Human の source 実装で使ってよいが、AI の大量候補生成・検索・tactic 実行経路に入れてはいけません。

```text
Human source package file layout
Human parser / resolver / notation table
open / namespace / overloaded display conveniences
Human-facing attributes such as intro / elim / refl / trans / congr
pretty theorem statements
per-module debug JSON files
source spans and source diagnostics
Human theorem search ranking
prompt text / natural-language explanations
filesystem package discovery during /machine/* request execution
```

AI path は次の形を維持します。

```text
Std.machine-* release artifacts
  -> Phase 5 Machine session import bundle
  -> MachineProofSnapshot with Machine Surface state
  -> MachineTacticCandidate
  -> /machine/tactics/run or /machine/tactics/batch
  -> /machine/replay
  -> /machine/verify
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 6 Human の各マイルストーンでは、次を acceptance criteria として扱います。

```text
- `/machine/*` endpoint の request / response schema を変更しない。
- `MachineTacticCandidate` に Human source path、pretty theorem text、Human attribute metadata を追加しない。
- Machine `state_fingerprint`、`candidate_hash`、`theorem_index_fingerprint`、`std_library_release_hash` の入力を増やさない。
- Human source build は release 前処理として実行し、AI request runtime に source parsing や package discovery を入れない。
- AI 用 theorem index / rw / simp metadata は Phase 6 AI Profile の validated sidecar から読む。
- Human per-module debug JSON は説明・review・local cache 用であり、AI runtime の必須入力にしない。
- Human-facing `intro` / `elim` / `refl` / `trans` / `congr` 属性は AI MVP theorem index に emit しない。
- theorem search ranking、prompt metadata、embedding、usage statistics は certificate hash / release hash に入れない。
- kernel / certificate verifier / independent checker 以外を proof acceptance boundary にしない。
```

---

## 2. 実装順

Phase 6 Human は、source package と certificate boundary を先に固定し、その後に各 module の内容を増やします。
Machine artifact 側は既存の Phase 6 AI 実装を再利用し、実 certificate で検証する方向に寄せます。

```text
1. Human / AI standard-library boundary と source package skeleton を固定する
2. Std.Logic Eq family を実装する
3. Std.Logic connectives を実装する
4. Std.Nat basic definitions を実装する
5. Std.Nat add theorems を実装する
6. Std.Nat mul / pred theorems を実装する
7. Std.List basic / append を実装する
8. Std.List length / map / foldr を実装する
9. Std.Algebra.Basic を実装する
10. source package から .npcert / hash / axiom report を生成する
11. Human theorem index / debug views を生成する
12. simp-lite / rw / search metadata を Phase 6 AI profile と照合する
13. Phase 4 tactic regression を real stdlib 上で固定する
14. Phase 6 AI release loader に real stdlib artifacts を渡す
15. docs / regression gate を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib human_parser
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-tactic
cargo test -p npa-api std_library
cargo test -p npa-api search
cargo test -p npa-api phase7
```

大きな内部変更後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. タスク一覧

### P6H-00: Human / AI standard-library boundary を固定する

実装タスク:

- [x] `develop/phase6-human.md`、`develop/phase6-ai.md`、README の境界を実装コメントまたは module docs に反映する。
- [x] standard-library source package root を固定する。public release module name は `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic` のみとする。
- [x] source package root は Phase 6 design の `Std/...` layout を materialize し、source path と `.npcert` artifact path の対応を docs / tests に固定する。
- [x] Human source layout から machine release identity を推測しない regression を追加する。
- [x] AI fast path が Human source parser / per-module debug JSON / Human attribute table を読まない regression を追加する。
- [x] legacy fixture module name `Std.Nat.Basic` / `Std.Logic.Eq` を Phase 6 release module と混同しない方針を test comment に明記する。

受け入れ条件:

- [x] Phase 6 release module membership が exactly four modules として test で固定されている。
- [x] `Core` / prelude が Phase 2 ImportEntry として出た場合に reject される境界が維持されている。
- [x] Human integration の前後で Machine API candidate identity / state fingerprint が変わらない regression が通る。

検証:

```sh
cargo test -p npa-api phase7_machine_api_identity_is_stable_around_phase5_human_integration_fixture
cargo test -p npa-api std_library
cargo test -p npa-frontend --lib human
```

依存:

```text
None
```

注意:

```text
この milestone は source package の中身を実装しない。境界と package skeleton だけを固定する。
```

### P6H-01: Std.Logic Eq family を実装する

実装タスク:

- [x] `Std.Logic` source に `Eq` inductive と generated public exports を定義する。
- [x] `Eq.refl` は generated constructor export として扱い、同名 theorem を別途 public export しない。
- [x] `Eq.symm`、`Eq.trans`、`Eq.subst`、`Eq.congrArg` を証明する。
- [x] `Eq.rec` が kernel-standard AxiomDecl として出る場合の exact exception を axiom report に反映する。
- [x] `Std.Logic` certificate が ordinary `Core` ImportEntry を持たないことを検査する。

受け入れ条件:

- [x] `Std.Logic.npcert` が source なしで verifier / kernel により再検査できる。
- [x] `Eq.refl` は theorem index entry ではなく generated family head として扱われる。
- [x] custom axiom は拒否され、許可される axiom は exact `Std.Logic.Eq.rec` 例外だけである。

検証:

```sh
cargo test -p npa-cert
cargo test -p npa-api std_library
cargo test -p npa-tactic rw
```

依存:

```text
P6H-00
```

### P6H-02: Std.Logic connectives を実装する

実装タスク:

- [x] `True` / `False` / `Not` / `And` / `Or` / `Iff` / `Exists` を `Std.Logic` source に追加する。
- [x] `False.elim` は MVP では `P : Prop` に限定し、large elimination を入れない。
- [x] `And.left` / `And.right` / `And.intro`、`Or.elim` / `Or.inl` / `Or.inr`、`Iff` 基本定理、`Exists.intro` / `Exists.elim` を証明する。
- [x] Human-facing `intro` / `elim` / `trans` / `congr` 属性は source metadata として保持し、AI theorem index attribute には流さない。

受け入れ条件:

- [x] `Std.Logic` が構成的に保たれ、`Classical.choice` / `funext` / `propext` を含まない。
- [x] apply search で `Eq.trans`、`And.intro`、`False.elim` が候補になる。
- [x] AI MVP theorem index に `Intro` / `Elim` / `Refl` / `Trans` / `Congr` attributes が出ない。

検証:

```sh
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-api search
cargo test -p npa-api std_library
```

依存:

```text
P6H-01
```

### P6H-03: Std.Nat basic definitions を実装する

実装タスク:

- [x] `Std.Nat` source module を作り、direct import を `Std.Logic` のみにする。
- [x] `Nat` inductive、`Nat.zero`、`Nat.succ`、generated recursor を public export する。
- [x] `Nat.one` と `Nat.pred` を定義する。
- [x] `Nat.pred_zero` と `Nat.pred_succ` を refl で証明する。
- [x] Nat 専用 numeral display / parsing を入れる場合も overloaded numerals にはしない。

受け入れ条件:

- [x] `Std.Nat` certificate の import closure が `Std.Logic` と `Std.Nat` だけで構成される。
- [x] `Nat` generated exports が Phase 6 AI `NatFamilyRef` として解決できる。
- [x] `Nat.pred_zero` / `Nat.pred_succ` が simp-safe candidate として登録可能である。

検証:

```sh
cargo test -p npa-frontend --lib human
cargo test -p npa-api std_library
cargo test -p npa-tactic induction
```

依存:

```text
P6H-02
```

### P6H-04: Std.Nat add theorems を実装する

実装タスク:

- [x] `Nat.add` を第2引数再帰として定義する。
- [x] `Nat.add_zero` と `Nat.add_succ` を definitional equality / refl で証明する。
- [x] `Nat.zero_add`、`Nat.succ_add`、`Nat.add_assoc`、`Nat.add_comm` を induction と Phase 4 tactic で証明する。
- [x] `Nat.add_comm` と `Nat.add_assoc` は simp に入れず、AI rewrite profile では exact `RwOnly` として扱う。

受け入れ条件:

- [x] `Nat.add n Nat.zero` と `Nat.add n (Nat.succ m)` の definitional equality tests が通る。
- [x] `Nat.add_zero` / `Nat.add_succ` / `Nat.zero_add` が simp-safe rule として動く。
- [x] `Nat.add_comm` / `Nat.add_assoc` は rw 候補だが simp 候補ではない。

検証:

```sh
cargo test -p npa-tactic rw
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

依存:

```text
P6H-03
```

### P6H-05: Std.Nat mul / pred theorem set を実装する

実装タスク:

- [x] `Nat.mul` を第2引数再帰として定義する。
- [x] `Nat.mul_zero` と `Nat.mul_succ` を refl で証明する。
- [x] `Nat.zero_mul` と `Nat.succ_mul` を induction / simp-lite で証明する。
- [x] `Nat.mul_assoc`、`Nat.mul_comm`、`Nat.left_distrib`、`Nat.right_distrib` を MVP 後半 theorem として証明し、simp-safe / AI `RwOnly` 固定集合には入れない。
- [x] `Nat.pred_zero` / `Nat.pred_succ` が P6H-03 から継続して simp-safe set に含まれることを確認する。

受け入れ条件:

- [x] `Nat.mul n Nat.zero` と `Nat.mul n (Nat.succ m)` の definitional equality tests が通る。
- [x] `Nat.mul_zero` / `Nat.mul_succ` / `Nat.zero_mul` が simp-safe rule として動く。
- [x] `Nat.mul_assoc` / `Nat.mul_comm` / distrib theorem が theorem search では見つかるが simp / AI MVP rewrite profile に混入しない。

検証:

```sh
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

依存:

```text
P6H-04
```

### P6H-06: Std.List basic / append を実装する

実装タスク:

- [x] `Std.List` source module を作り、direct imports を `Std.Logic` と `Std.Nat` にする。
- [x] `List` inductive、`List.nil`、`List.cons`、generated recursor を public export する。
- [x] `List.append` を第1引数再帰として定義する。
- [x] `List.nil_append` と `List.cons_append` を refl で証明する。
- [x] `List.append_nil` と `List.append_assoc` を induction / simp-lite で証明する。
- [x] `List.append_assoc` は simp に入れず、AI rewrite profile では exact `RwOnly` として扱う。

受け入れ条件:

- [x] `[] ++ ys` と `(x :: xs) ++ ys` の definitional equality tests が通る。
- [x] `List.nil_append` / `List.cons_append` / `List.append_nil` が simp-safe rule として動く。
- [x] `std.list.simp` が `Std.Nat` rewrite rule source を含まない。

検証:

```sh
cargo test -p npa-tactic induction
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

依存:

```text
P6H-05
```

### P6H-07: Std.List length / map / foldr を実装する

実装タスク:

- [x] `List.length`、`List.map`、`List.foldr` を source に追加する。
- [x] `List.length_nil` / `List.length_cons` / `List.map_nil` / `List.map_cons` / `List.foldr_nil` / `List.foldr_cons` を refl で証明する。
- [x] `List.length_append`、`List.map_id`、`List.map_comp` を induction / simp-lite で証明する。
- [x] `List.length_append` は AI `RwOnly` に含め、`List.map_comp` は AI MVP rewrite profile に含めない。
- [x] `foldl` と list literal `[a, b, c]` は MVP に入れない。

受け入れ条件:

- [x] List simp-safe exact set が Phase 6 Human / AI Profile と一致する。
- [x] `List.length_append` は rw 候補だが simp 候補ではない。
- [x] `List.map_comp` は theorem として存在しても AI MVP rewrite / simp profile に出ない。

検証:

```sh
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

依存:

```text
P6H-06
```

### P6H-08: Std.Algebra.Basic を実装する

実装タスク:

- [x] `Std.Algebra.Basic` source module を作り、direct import を `Std.Logic` のみにする。
- [x] `Associative` / `Commutative` / `LeftIdentity` / `RightIdentity` を unbundled property として定義する。
- [x] `IsSemigroup` / `IsMonoid` / `IsCommMonoid` を explicit Prop inductive として定義する。
- [x] `IsMonoid.assoc` / `IsMonoid.left_id` / `IsMonoid.right_id` と `IsCommMonoid` projection theorem を証明する。
- [x] `identity_unique` を証明する。
- [x] `Nat.add_is_comm_monoid` は `Std.Algebra.Basic` や `Std.Nat` の MVP release module に入れない。

受け入れ条件:

- [x] `Std.Algebra.Basic` certificate の import closure が `Std.Logic` と `Std.Algebra.Basic` だけで構成される。
- [x] typeclass resolution、bundled carrier、implicit instance search が導入されていない。
- [x] algebra projection theorem が apply search で候補になる。

検証:

```sh
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-api std_library
cargo test -p npa-api search
```

依存:

```text
P6H-02
```

### P6H-09: source package から certificate artifacts を生成する

実装タスク:

- [x] standard-library source package を deterministic order で compile する build entrypoint を追加する。
- [x] 各 module の raw `.npcert` bytes、export_hash、certificate_hash、module-level axiom_report_hash を生成する。
- [x] import entries の export_hash mismatch を build failure にし、高信頼モードでは certificate_hash mismatch も failure にする。
- [x] `ExportEntry.name` は declaration name そのものとし、module name と連結した synthetic name を作らない。
- [x] `Core` / prelude を ordinary ImportEntry として出す source artifact を reject する。

受け入れ条件:

- [x] `Std/Logic.npcert`、`Std/Nat.npcert`、`Std/List.npcert`、`Std/Algebra/Basic.npcert` 相当の artifact が raw Phase 2 certificate bytes として生成される。
- [x] 全 module が source なしで certificate verifier により再検査できる。
- [x] axiom report は empty または exact `Eq.rec` exception のみである。

検証:

```sh
cargo test -p npa-cert
cargo test -p npa-api std_library
cargo test --workspace
```

依存:

```text
P6H-01
P6H-02
P6H-03
P6H-04
P6H-05
P6H-06
P6H-07
P6H-08
```

### P6H-10: Human theorem index / debug views を生成する

実装タスク:

- [x] Human-facing theorem search view を certificate verifier output から生成する。
- [x] per-module `index` / `axioms` / minimal dependency graph debug view を生成する。
- [x] `proof_term_size` は AI MVP artifact では null にし、Human debug view の任意情報と混同しない。
- [x] Human source attributes から `simp` / `rw` / `apply` / `intro` / `elim` 表示を作る場合も、certificate-derived identity に bind する。
- [x] Human theorem search の suggested tactic string は Human UI 用に限定する。

受け入れ条件:

- [x] Human search で `Nat.add_zero`、`List.append_nil`、`Eq.trans` が期待カテゴリに出る。
- [x] debug view は source text や pretty statement を trusted hash input にしない。
- [x] AI `MachineStdTheoremIndex` schema と Human debug schema の責務が test / docs で分離されている。

検証:

```sh
cargo test -p npa-api human_theorem_index
cargo test -p npa-api search
cargo test -p npa-api std_library
```

依存:

```text
P6H-09
```

### P6H-11: simp-lite / rw / search metadata を AI Profile と照合する

実装タスク:

- [x] Human source の `simp` / `rw` intent から、Phase 6 AI MVP の exact SimpSafe / RwOnly fixed sets との照合 test を追加する。
- [x] Nat SimpSafe set を `Nat.add_zero` / `Nat.add_succ` / `Nat.zero_add` / `Nat.mul_zero` / `Nat.mul_succ` / `Nat.zero_mul` / `Nat.pred_zero` / `Nat.pred_succ` に固定する。
- [x] List SimpSafe set を `List.nil_append` / `List.cons_append` / `List.append_nil` / `List.length_nil` / `List.length_cons` / `List.map_nil` / `List.map_cons` / `List.map_id` / `List.foldr_nil` / `List.foldr_cons` に固定する。
- [x] RwOnly set を `Nat.add_comm` / `Nat.add_assoc` / `List.append_assoc` / `List.length_append` に固定する。
- [x] `Nat.mul_comm` / `Nat.mul_assoc` / `List.map_comp` が AI MVP rewrite profile に出ない regression を追加する。

受け入れ条件:

- [x] `std.logic.simp` は empty であり、`Eq.refl` は SimpRuleRef として emit されない。
- [x] `std.list.simp` は `Std.Nat` rule source を含まない。
- [x] `std.all.simp` と `std.all.rw` は source profile の semantic union として再検証される。

検証:

```sh
cargo test -p npa-api std_library
cargo test -p npa-tactic simp
cargo test -p npa-tactic rw
```

依存:

```text
P6H-10
```

### P6H-12: Phase 4 tactic regression を real stdlib 上で固定する

実装タスク:

- [x] `intro` / `exact` / `apply` / `rw` / `simp-lite` / `induction` の regression を real standard-library certificates 上で追加する。
- [x] `Nat.zero_add`、`List.append_nil`、`Eq.trans` を Phase 4 tactic だけで再証明する tests を追加する。
- [x] failed rewrite、loop-prone simp rule、missing theorem search result の negative tests を追加する。
- [x] Human by proof examples が Machine Surface fixture hash を変えないことを確認する。

受け入れ条件:

- [x] `simp-lite` が Nat/List の基本ゴールを閉じる。
- [x] `rw [Nat.add_zero]` と `rw [List.append_nil]` が real stdlib theorem を使って動く。
- [x] unresolved goal、sorry 相当、未許可 axiom を含む theorem は certificate 化されない。

検証:

```sh
cargo test -p npa-tactic
cargo test -p npa-api human
cargo test -p npa-api search
cargo test -p npa-api phase7
```

依存:

```text
P6H-11
```

### P6H-13: real stdlib artifacts を Phase 6 AI release loader に接続する

実装タスク:

- [x] P6H-09 で生成した raw `.npcert` artifacts を `load_machine_std_mvp_certificates` に渡す integration fixture を追加する。
- [x] `MachineStdLibraryRelease` / import bundles / theorem index / rewrite profiles / simp profiles / axiom report を real stdlib artifacts から生成する。
- [x] `std.nat.mvp` / `std.list.mvp` / `std.all.mvp` import bundle を Phase 5 `/machine/sessions` 相当の request に展開する。
- [x] Phase 7 retrieval fixture が real stdlib theorem index から候補を作り、Phase 5 batch に戻る regression を追加する。
- [x] Phase 8 audit hook が sidecar と verifier output の一致を再検査できることを確認する。

受け入れ条件:

- [x] release manifest hashes が certificate bytes / sidecar hashes と一致する。
- [x] recommended tactic options recipe が Phase 5 option validation に通る。
- [x] stale export_hash / certificate_hash / decl_interface_hash を持つ artifact が拒否される。
- [x] AI candidate は Phase 5 run/batch/replay/verify なしに採用されない。

検証:

```sh
cargo test -p npa-api std_library
cargo test -p npa-api phase7
cargo test -p npa-api independent_checker
```

依存:

```text
P6H-12
```

### P6H-14: documentation / release regression gate を固定する

実装タスク:

- [x] README の実装状況に Phase 6 Human standard-library source / artifact handoff の状態を反映する。
- [x] `develop/phase6-human.md` と `develop/phase6-ai.md` の module set、simp set、RwOnly set、axiom exception が一致していることを確認する。
- [x] legacy fixture module names と release module names の関係を docs / tests で明確にする。
- [x] final regression gate として formatting、clippy、workspace tests、Phase 9 regression script を実行する。
- [x] generated artifacts が commit 対象か build artifact かを docs に明記する。

受け入れ条件:

- [x] Phase 6 Human 完了条件が docs と tests で trace できる。
- [x] `./scripts/phase9-regression.sh` が通る。
- [x] working tree に generated junk や stale fixture artifact が残らない。

検証:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

依存:

```text
P6H-13
```

---

## 4. Milestone dependency graph

```text
P6H-00
  ↓
P6H-01
  ↓
P6H-02
  ├── P6H-03
  │     ↓
  │   P6H-04
  │     ↓
  │   P6H-05
  │     ↓
  │   P6H-06
  │     ↓
  │   P6H-07
  └── P6H-08
        ↓
P6H-01 + P6H-02 + P6H-03 + P6H-04 + P6H-05 + P6H-06 + P6H-07 + P6H-08
  ↓
P6H-09
  ↓
P6H-10
  ↓
P6H-11
  ↓
P6H-12
  ↓
P6H-13
  ↓
P6H-14
```

P6H-08 は `Std.Logic` だけに依存するため、Nat/List と並行して進められます。
ただし P6H-09 の full source package build では、4 release modules すべてが揃っている必要があります。

---

## 5. 入れないもの

MVP では次を入れません。

```text
- full typeclass system
- coercions
- overloaded algebraic notation
- integers, rationals, reals
- finite sets, options, arrays, trees
- quotient types
- classical choice
- function extensionality
- proof irrelevance axiom
- groups, rings, fields
- order theory
- category theory
- full simp
- ring / omega / linarith
- theorem ranking or embedding as trusted metadata
- automatic import insertion by AI server
- server-side package download during Machine API request handling
```

---

## 6. 完了条件

Phase 6 Human が完了したと言える条件はこれです。

```text
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic source package skeleton が存在し、manifest が module membership / certificate path を固定し、source skeleton が import intent を固定する。
- 4 release modules が kernel / verifier で検査済みの `.npcert` として生成される。
- import entries が export_hash を持ち、高信頼モードで certificate_hash も照合される。
- module の export_hash / certificate_hash / axiom_report_hash が生成される。
- axiom report は empty、または exact Std.Logic.Eq.rec kernel-standard exception のみである。
- Human theorem search view と AI Machine theorem index の責務が分離されている。
- simp-lite が Nat/List の基本ゴールを閉じる。
- theorem search が exact / apply / rw / simp 候補を返す。
- Phase 4 tactic だけで基本定理の再証明 tests が通る。
- Phase 6 AI release loader が real standard-library certificate artifacts を受け取り、Phase 5 / Phase 7 / Phase 8 の regression が通る。
```
