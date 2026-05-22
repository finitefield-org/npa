# Phase 5 Human Task Breakdown

このタスク分解は `doc/phase5-human.md` を正とし、現在の
`crates/npa-frontend` / `crates/npa-tactic` / `crates/npa-api` 実装との差分を
実装マイルストーンに分けたものです。

Phase 5 Human は、人間向け IDE / Web UI / CLI / Human API client が proof state、
tactic 実行、theorem search、goal display を扱うための非信頼層です。
AI 証明探索器向けの決定的な Machine API 契約は `doc/phase5-ai.md` の責務であり、
Human の text tactic、pretty display、LSP 都合、document cache を Machine hot path に混ぜてはいけません。

重要な制約:

```text
- Human API が `success` を返しても証明済みとは扱わない。
- 証明の受理根拠は canonical certificate と kernel / verifier / independent checker の結果だけにする。
- Human `/tactic/run` は text tactic を受けてよいが、AI 探索器の大量候補経路には使わない。
- `/machine/tactics/run`、`/machine/tactics/batch`、`/machine/replay`、`/machine/verify` の request grammar、
  fingerprint、cache key、diagnostic hash を変えない。
- Human goal display、source span、LSP diagnostic、assistant payload は certificate payload に入れない。
- kernel crate に HTTP server、network、I/O、plugin loading、AI 呼び出しを入れない。
```

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在の `crates/npa-frontend` には、Phase 3 Human Surface と Phase 4 Human tactic bridge の前提があります。

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanDeclValue
- HumanProofBlock / HumanTacticScript / HumanTacticSyntax
- HumanSourceInterface / HumanImportedSourceInterface
- HumanDiagnostic / HumanDiagnosticPayload / HumanHoleGoal

crates/npa-frontend/src/human_parser.rs
- parse_human_module / parse_human_term
- by proof block と intro / exact / apply / rw / simp-lite / induction の Human tactic parser

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core / compile_human_source_to_certificate
- Human tactic term elaboration context
- collect_human_by_proof_targets_with_source_interfaces
- prepare_human_proof_start_core_with_source_interfaces
```

現在の `crates/npa-tactic` には、Machine proof state primitive と tactic execution core があります。

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- start_machine_proof
- run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
- assign_goal / extract_closed_machine_proof / extract_closed_machine_theorem_decl
- intro / exact / apply / rw / simp-lite / induction-nat
- deterministic budget hash / tactic cache key / proof delta
```

現在の `crates/npa-api` には、Human by proof 実行と Machine API substrate があります。

```text
crates/npa-api/src/human.rs
- compile_human_source_to_core / compile_human_source_to_certificate
- start_human_proof
- check_human_tactic_term
- run_human_exact_tactic / run_human_intro_tactic / run_human_apply_tactic
- run_human_rewrite_tactic / run_human_simp_lite_tactic / run_human_induction_tactic
- run_human_tactic_script

crates/npa-api/src/session.rs
- create_machine_session

crates/npa-api/src/snapshot.rs
- MachineProofSnapshot / MachineGoalView materialization
- get_machine_snapshot

crates/npa-api/src/tactic.rs
- run_machine_tactic_request
- run_machine_tactic_batch_request

crates/npa-api/src/search.rs
- search_machine_theorems_for_goal

crates/npa-api/src/replay.rs / crates/npa-api/src/verify.rs
- Machine replay / verify handoff
```

### 0.2 未実装の Phase 5 Human 範囲

`doc/phase5-human.md` が要求する以下の範囲は、Human IDE/API profile としてまだ独立していません。

```text
HumanProofSession / HumanDocumentSnapshot / HumanProofStateStore
Human-facing StateId / DocumentId / DocumentVersion lifecycle
StructuredProofState / StructuredGoal / StructuredHypothesis / StructuredExpr response model
source position to proof state lookup
Human `/state/at` / `/state/current` / `/state/goals` / `/state/by_id` library API
Human `/tactic/run` request/response wrapper with text tactic parsing
Human `/tactic/check` and `/tactic/suggest`
Human theorem index and `/search/name` / `/search/by_type` / `/search/for_goal` / `/search/rewrite`
Human display modes: pretty / explicit / core / json
goal diff, context folding, relevant context ordering
document update and declaration-level cache
LSP-facing diagnostics / hover / code actions / custom goal view payloads
optional Human assistant payload
```

### 0.3 Machine fast path に入れてはいけないもの

以下は Phase 5 Human で扱ってよいが、Machine API の高頻度候補検査経路に入れてはいけません。

```text
Human text tactic parser
Human source document manager
Human source positions and spans
Human display names and pretty text
notation-based suggested tactic strings
LSP diagnostics / hover / code action objects
context folding and relevant-context UI ranking
assistant confidence / reason text
HTTP / JSON-RPC / LSP transport details
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 5 Human の各マイルストーンでは、次を acceptance criteria として扱います。

```text
- `/machine/*` endpoint の request / response schema を変更しない。
- `MachineTacticCandidate` に Human text tactic、pretty display、source span、assistant metadata を追加しない。
- Machine `state_fingerprint`、`snapshot_id`、`candidate_hash`、`proof_delta_hash`、`deterministic_budget_hash` を変更しない。
- Machine snapshot 取得の AI 経路は `include_pretty = false` を維持できる。
- Human `/tactic/run` は `run_machine_tactic_candidates_batch` の代替にしない。
- Human search result の suggested tactic string は Human UI 用に限定し、AI 探索器向けには
  `MachineTacticCandidate` を返す `doc/phase5-ai.md` の search contract を使う。
- Human document cache key は trusted certificate hash と混同しない。
- Human API は kernel crate に依存方向を増やさず、kernel に I/O / network / server state を入れない。
```

推奨する構成:

```text
Human IDE path:
  Human source document
  -> HumanProofSession / HumanDocumentSnapshot
  -> npa_frontend Human parser / resolver / elaborator
  -> npa_api Human tactic bridge
  -> StructuredProofState / display / search / LSP payload
  -> verify request
  -> kernel / certificate verifier

AI path:
  MachineProofSession
  -> MachineProofSnapshot with include_pretty = false
  -> raw MachineTacticCandidate
  -> /machine/tactics/batch
  -> /machine/replay
  -> /machine/verify
```

---

## 2. 実装順

Phase 5 Human は、まず既存 Human tactic bridge を壊さずに IDE 用 state model を作り、
その後に tactic / search / display / LSP を薄く積みます。

```text
1. Human / Machine API 境界と regression guard を固定する
2. Human session / document store を作る
3. Human proof state store と StructuredGoal を materialize する
4. state 取得 API を公開する
5. goal display renderer を追加する
6. Human tactic run / check を session state に接続する
7. tactic suggestion を追加する
8. Human theorem index と search API を追加する
9. session verify / certificate handoff を統合する
10. document update / incremental checking を追加する
11. LSP payload adapter を追加する
12. optional assistant payload を追加する
13. integration regression と doc consistency を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-api human
cargo test -p npa-api phase7
cargo test -p npa-api phase9
cargo test -p npa-tactic
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib machine_surface
```

大きな内部変更後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. タスク一覧

### P5H-00: Human / Machine IDE API 境界を固定する

実装タスク:

- [x] `doc/phase5-human.md`、`doc/phase5-ai.md`、`doc/phase7-ai.md` の境界を実装コメントと test 名に反映する。
- [x] Human IDE API 用 module の置き場所を固定する。候補は `crates/npa-api/src/human_ide.rs` または `human.rs` 内の Human-only submodule。
- [x] Human IDE API が Machine session を暗黙作成しないことを public API コメントに明記する。
- [x] `/machine/*` request grammar、`MachineProofSnapshot`、`MachineTacticCandidate` に Human 専用 field を足さない regression を追加する。

依存:

```text
なし
```

Deliverables:

```text
- Human IDE API boundary skeleton
- Machine fast-path regression guard
```

影響ファイル:

```text
crates/npa-api/src/lib.rs
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
doc/phase5-human.md
doc/phase5-ai.md
doc/phase7-ai.md
```

Acceptance criteria:

- [x] Human IDE API の入口が Machine API と別名で export されている。
- [x] `run_machine_tactic_request` / `run_machine_tactic_batch_request` は Human parser を呼ばない。
- [x] Phase 7 / Phase 9 の tests が Human IDE module を import しなくても通る。

Verification:

```sh
rg -n "parse_human|Human" crates/npa-api/src/tactic.rs crates/npa-api/src/search.rs crates/npa-api/src/replay.rs crates/npa-api/src/verify.rs
cargo test -p npa-api phase7
cargo test -p npa-api phase9
```

AI 速度ガード:

- [x] Machine `state_fingerprint` / `candidate_hash` / `deterministic_budget_hash` fixtures を変更しない。

---

### P5H-01: Human session / document store を作る

実装タスク:

- [x] `HumanProofSession`、`HumanDocumentId`、`HumanDocumentVersion`、`HumanSessionId` を追加する。
- [x] `HumanDocumentSnapshot` に source text、module name、verified imports、Human imported source interfaces、options を保持する。
- [x] `POST /sessions` 相当の library API を追加し、Human source を parse / collect して初期 messages を返す。
- [x] `POST /documents/update` 相当の library API を追加し、document version を単調増加させる。
- [x] session store は in-memory library data structure として実装し、kernel crate には入れない。

依存:

```text
P5H-00
```

Deliverables:

```text
- HumanProofSession and HumanDocumentSnapshot data model
- session create / document update library API
```

影響ファイル:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
crates/npa-api/src/lib.rs
```

Acceptance criteria:

- [x] Human source から `session_id`、`document_id`、`document_version = 1` を持つ open session を作れる。
- [x] update 後に古い `document_version` を指定した state request を stale request として構造化 error にできる。
- [x] verified imports と imported Human source interfaces は request で明示され、filesystem / network lookup は行わない。

Verification:

```sh
cargo test -p npa-api human_session
cargo test -p npa-api human
```

AI 速度ガード:

- [x] Human session store を `MachineProofSession` に統合しない。

---

### P5H-02: Human proof state store と stable state id を作る

実装タスク:

- [x] `HumanProofStateStore`、`HumanStateId`、`HumanGoalId` mapping を追加する。
- [x] `start_human_proof` の `MachineProofState` を Human session 内 state として保存する。
- [x] tactic 実行後の new state を immutable entry として追加し、old state を破壊しない。
- [x] state entry に document version、source position、selected goal、messages を紐付ける。

依存:

```text
P5H-01
```

Deliverables:

```text
- HumanProofStateStore with stable Human state handles
- immutable state transition storage
```

影響ファイル:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
```

Acceptance criteria:

- [x] 同じ session 内で `state_id` から元の proof state を再取得できる。
- [x] tactic 失敗後に old state の open goals と proof skeleton が変わらない。
- [x] state id は Human API handle であり、Machine `state_fingerprint` の代替として使われない。

Verification:

```sh
cargo test -p npa-api human_state_store
cargo test -p npa-tactic
```

AI 速度ガード:

- [x] Human `state_id` を Phase 7 search node identity に使う API を追加しない。

---

### P5H-03: StructuredProofState / StructuredGoal を materialize する

実装タスク:

- [x] `StructuredProofState`、`StructuredGoal`、`StructuredHypothesis`、`StructuredExpr` を追加する。
- [x] Machine goal context から local id、name、type、optional value、implicit flag、dependency list を作る。
- [x] target / hypothesis type から `core_hash`、head symbol、constants、free locals、size を計算する。
- [x] `pretty` は表示 field とし、identity / cache / verification の根拠にしない。

依存:

```text
P5H-02
```

Deliverables:

```text
- StructuredProofState / StructuredGoal / StructuredExpr model
- deterministic core metadata materialization
```

影響ファイル:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
crates/npa-api/src/renderer.rs
```

Acceptance criteria:

- [x] `theorem t (n : Nat) : n = n := by _` の open goal が context `n : Nat` と target `n = n` を持つ。
- [x] `core_hash` は pretty text 変更では変わらず、core expr 変更で変わる。
- [x] local dependency order は deterministic で、HashMap iteration order に依存しない。

Verification:

```sh
cargo test -p npa-api human_structured_goal
cargo test -p npa-api renderer
```

AI 速度ガード:

- [x] `MachineGoalView` canonical bytes を変更しない。

---

### P5H-04: state 取得 API を公開する

実装タスク:

- [x] `/state/by_id` 相当の library API を追加する。
- [x] `/state/goals` 相当の軽量 API を追加し、goal id と pretty display を返す。
- [x] `/state/current` 相当の API を session cursor state に接続する。
- [x] `/state/at` 相当の API を source position と proof block / hole position に対応付ける。
- [x] not found / stale document / no proof state の error kind を構造化する。

依存:

```text
P5H-03
```

Deliverables:

```text
- state lookup library APIs
- structured stale / not-found diagnostics
```

影響ファイル:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
```

Acceptance criteria:

- [x] source position から current proof state を取得できる。
- [x] `_` hole の位置で該当 goal が返る。
- [x] source position が proof 外の場合は empty goals または structured not-found response を返す。

Verification:

```sh
cargo test -p npa-api human_state_api
```

AI 速度ガード:

- [x] `/machine/snapshots/get` の include-pretty-free path に依存変更を入れない。

---

### P5H-05: Human goal display renderer を追加する

実装タスク:

- [x] `/display/goal` 相当の API に `pretty` / `explicit` / `core` / `json` mode を追加する。
- [x] `/display/expr` 相当の API を追加し、StructuredExpr 単位で表示できるようにする。
- [x] `/display/diff` 相当の API で tactic 前後の goal replacement / closed / added goals を表示する。
- [x] `/display/context` 相当の API で context folding と relevant context ordering を行う。
- [x] display options は Human display profile として固定し、trusted payload に入れない。

依存:

```text
P5H-03
P5H-04
```

Deliverables:

```text
- Human display renderer
- goal diff and context display APIs
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/renderer.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] pretty mode は notation / implicit hiding を使った人間向け表示を返せる。
- [x] explicit mode は implicit arguments を表示する。
- [x] core mode は kernel が見る core expression に近い表示を返す。
- [x] folding しても json / StructuredGoal には完全な context が残る。

Verification:

```sh
cargo test -p npa-api human_display
```

AI 速度ガード:

- [x] pretty renderer を Machine candidate validation / replay / verify の入力にしない。

---

### P5H-06: Human `/tactic/run` を session state に接続する

実装タスク:

- [x] Human text tactic request を parse し、既存 `run_human_*_tactic` に dispatch する。
- [x] `state_id` / `goal_id` / `tactic` / `budget` の request validation を追加する。
- [x] success / closed / partial / error / timeout / unsafe の response shape を固定する。
- [x] tactic 成功時は new state を `HumanProofStateStore` に追加し、old state を保持する。
- [x] tactic 失敗時は old state id、structured error、expected / actual hash、span、suggestions を返す。

依存:

```text
P5H-02
P5H-04
P5H-05
```

Deliverables:

```text
- Human tactic run library API
- transactional state update response model
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] `intro n` が Pi target を subgoal に変換し、new state id を返す。
- [x] `exact n` が matching local を使って goal を閉じる。
- [x] `apply Eq.trans` が expected subgoals を返す。
- [x] `rw [Nat.add_zero]` と `simp-lite` は proof-producing path だけを使う。
- [x] `intro h` を equality target に投げると `expected_pi_type` 相当の structured error になる。

Verification:

```sh
cargo test -p npa-api human_tactic_run
cargo test -p npa-api human
```

AI 速度ガード:

- [x] `/machine/tactics/batch` は Human `/tactic/run` を呼ばない。

---

### P5H-07: Human `/tactic/check` と `/tactic/suggest` を追加する

実装タスク:

- [x] `/tactic/check` 相当の API を追加し、parse / validation / expected effect を返すが state を保存しない。
- [x] `/tactic/suggest` 相当の builtin suggestion を追加する。
- [x] `target is Pi -> intro`、`Eq t t -> exact Eq.refl t`、context exact、rw 候補、Nat induction 候補を実装する。
- [x] suggestion response に source、confidence、reason、suggested tactic text を含める。
- [x] suggestion は非信頼であり、採用前に `/tactic/run` へ再投入する。

依存:

```text
P5H-06
```

Deliverables:

```text
- Human tactic check API
- builtin Human tactic suggestion API
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Pi target で `intro` suggestion が返る。
- [x] reflexive equality target で `exact Eq.refl ...` suggestion が返る。
- [x] context に target と同型の local があると `exact h` suggestion が返る。
- [x] suggestion が失敗しても proof state は変わらない。

Verification:

```sh
cargo test -p npa-api human_tactic_suggest
```

AI 速度ガード:

- [x] Human suggestion confidence / reason は Machine cache key、replay plan、certificate に入れない。

---

### P5H-08: Human theorem index を追加する

実装タスク:

- [x] verified imports と checked current declarations から Human theorem index を作る。
- [x] Index entry に name、module、statement core expr、statement pretty、head symbol、constants、attributes、kind、dependencies、axiom deps を持たせる。
- [x] import の `export_hash` / high-trust `certificate_hash` / `decl_interface_hash` を保持する。
- [x] current declaration は kernel check 済み prefix だけを index に入れる。
- [x] axiom dependency を ranking / filter で使える形にする。

依存:

```text
P5H-03
P5H-04
```

Deliverables:

```text
- Human theorem index data model
- verified import / current declaration index builder
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/search.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] direct verified import の theorem / def / axiom / constructor / recursor を index 化できる。
- [x] unchecked external theorem database は index に入らない。
- [x] `decl_interface_hash` のない theorem を verified result として返さない。
- [x] axiom 依存 theorem を識別できる。

Verification:

```sh
cargo test -p npa-api human_theorem_index
cargo test -p npa-api search
```

AI 速度ガード:

- [x] Machine `/machine/search/for_goal` の theorem index fingerprint を変更しない。

---

### P5H-09: Human theorem search API を追加する

実装タスク:

- [x] `/search/name` 相当の API を追加する。
- [x] `/search/by_type` 相当の API を追加する。
- [x] `/search/for_goal` 相当の API を exact / apply / rw / simp mode で追加する。
- [x] `/search/rewrite` 相当の API を追加する。
- [x] result に suggested Human tactic string、match、why、score、axiom info を含める。
- [x] high-trust mode では axiom 使用 theorem を penalty または filter できるようにする。

依存:

```text
P5H-08
P5H-07
```

Deliverables:

```text
- Human theorem search APIs
- suggested Human tactic rendering for search results
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/search.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] `Nat.add_zero` を名前検索できる。
- [x] `?x + 0 = ?x` 型パターンで matching theorem を返せる。
- [x] current goal `n + 0 = n` に対して `exact Nat.add_zero n` と `rw [Nat.add_zero]` の候補を返せる。
- [x] suggested tactic は `/tactic/run` に再投入して検査できる。

Verification:

```sh
cargo test -p npa-api human_search
```

AI 速度ガード:

- [x] Human suggested tactic string を raw `MachineTacticCandidate` の代わりに Phase 7 へ渡す API を作らない。

---

### P5H-10: Human session verify / certificate handoff を統合する

実装タスク:

- [x] `/session/verify` 相当の API を追加する。
- [x] open goals が残る state は verification 前に拒否する。
- [x] closed proof state から root proof term を抽出し、kernel check と certificate generation に渡す。
- [x] certificate verifier output 由来の certificate hash、axioms used、contains_sorry 相当を返す。
- [x] import の `export_hash` / `certificate_hash` と axiom report を無視しない。

依存:

```text
P5H-06
P5H-08
```

Deliverables:

```text
- Human session verify API
- certificate / axiom report response model
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/verify.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] goals empty だけでは verified にせず、kernel check と certificate verifier 成功後だけ verified を返す。
- [x] unresolved goal が1つでもあれば certificate 化を拒否する。
- [x] axiom report が response と certificate verifier output で一致する。

Verification:

```sh
cargo test -p npa-api human_verify
cargo test -p npa-cert
```

AI 速度ガード:

- [x] Human verify は `/machine/verify` の response schema / replay contract を変更しない。

---

### P5H-11: document update / incremental checking を追加する

実装タスク:

- [x] `source_decl_hash`、`resolved_decl_hash`、`core_decl_hash` を Human document cache key として導入する。
- [x] unchanged declaration の parse / resolve / elaborate result を再利用する。
- [x] changed declaration 以降の proof states と diagnostics を再計算する。
- [x] cache hit / cache miss は証明の受理根拠にしない。
- [x] cache invalidation を import interface hash と document version に紐付ける。

依存:

```text
P5H-01
P5H-04
P5H-10
```

Deliverables:

```text
- Human document incremental cache
- declaration-level invalidation rules
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
crates/npa-frontend/src/human_elaborator.rs
```

Acceptance criteria:

- [x] source edit 後に document version が増え、古い state request は stale として拒否できる。
- [x] unchanged prefix の declarations は再利用される。
- [x] reused result でも final verify は kernel / certificate verifier を通る。

Verification:

```sh
cargo test -p npa-api human_incremental
cargo test -p npa-frontend --lib human
```

AI 速度ガード:

- [x] Human incremental cache key を Machine `state_fingerprint` として再利用しない。

---

### P5H-12: LSP-facing payload adapter を追加する

実装タスク:

- [x] diagnostics payload を LSP diagnostic shape に変換する adapter を追加する。
- [x] hover payload に theorem statement、attributes、axioms を含める。
- [x] completion / code action 用に tactic suggestion と search command を返す。
- [x] semantic tokens / document symbols / inlay hints の最小 payload を追加する。
- [x] custom goal view は `/state/goals` と `/display/goal` を使う。
- [x] transport server は optional layer とし、kernel crate には入れない。

依存:

```text
P5H-04
P5H-05
P5H-07
P5H-09
```

Deliverables:

```text
- LSP-facing payload adapters
- goal view / hover / code action response models
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Human diagnostic span が LSP range に変換される。
- [x] hover で `Nat.add_zero` の statement と axiom info を返せる。
- [x] code action で `exact Eq.refl n` / `simp-lite` / search command を返せる。

Verification:

```sh
cargo test -p npa-api human_lsp
```

AI 速度ガード:

- [x] LSP payload type を Machine API response envelope に混ぜない。

---

### P5H-13: optional assistant payload を追加する

実装タスク:

- [x] Human UI 用に structured goal、nearby theorem、failed tactics、available tactics をまとめる payload を追加する。
- [x] assistant output は suggested Human tactic string と confidence / reason だけにする。
- [x] assistant output は必ず `/tactic/run` で検査してから候補採用する。
- [x] AI 証明探索器向け deterministic payload は `doc/phase5-ai.md` の `/machine/prompt_payload` を使うことを API docs に明記する。

依存:

```text
P5H-07
P5H-09
```

Deliverables:

```text
- optional Human assistant payload API
- assistant candidate validation rule
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
doc/phase5-human.md
```

Acceptance criteria:

- [x] assistant payload に state id、goal summary、available tactics、nearby theorem、failed tactic diagnostics が入る。
- [x] confidence / reason は certificate、replay plan、Machine cache key に入らない。
- [x] Machine `/machine/prompt_payload` の schema と fingerprint が変わらない。

Verification:

```sh
cargo test -p npa-api human_assistant_payload
cargo test -p npa-api prompt
```

AI 速度ガード:

- [x] assistant payload を Phase 7 MVP の required path にしない。

---

### P5H-14: Integration regression と documentation を固定する

実装タスク:

- [x] Phase 5 Human の end-to-end fixture を追加する。
- [x] fixture は session create、state lookup、tactic run、search、display、verify を通す。
- [x] Human path と Machine path の separation regression を追加する。
- [x] README の実装状況を更新する。
- [x] `doc/phase5-human.md` と本タスク分解の完了条件を同期する。

依存:

```text
P5H-00
P5H-01
P5H-02
P5H-03
P5H-04
P5H-05
P5H-06
P5H-07
P5H-08
P5H-09
P5H-10
P5H-11
P5H-12
P5H-13
```

Deliverables:

```text
- Phase 5 Human end-to-end fixtures
- README / phase documentation status update
```

影響ファイル:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
README.md
doc/phase5-human.md
doc/phase5-human-todo.md
```

Acceptance criteria:

- [x] `theorem t (n : Nat) : n = n := by exact Eq.refl n` を Human session から verify できる。
- [x] `theorem id (A : Type) (x : A) : A := by exact x` の type mismatch diagnostic が structured に返る。
- [x] Machine API Phase 7 fixtures が Human API 追加前後で同じ candidate / state identity を維持する。
- [x] `cargo fmt --all`、targeted tests、workspace tests の実行結果を PR / commit message で報告できる。

Verification:

```sh
cargo fmt --all
cargo test -p npa-api human
cargo test -p npa-api phase7
cargo test -p npa-api phase9
cargo test -p npa-tactic
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib machine_surface
./scripts/phase9-regression.sh
```

AI 速度ガード:

- [x] Human IDE integration 後も Phase 7 MVP は `MachineApiClient`、`MachineProofSnapshot`、
  raw `MachineTacticCandidate`、`/machine/tactics/batch`、`/machine/replay`、`/machine/verify` を使い続ける。

---

## 4. Review ledger

最終レビュー結果:

```text
No confirmed findings remain.
```

確認した観点:

```text
- `doc/phase5-human.md` の 4 つの中核機能を P5H-03 から P5H-10 に割り当てた。
- document / incremental checking、LSP、assistant payload、non-goals を P5H-11 から P5H-13 に分離した。
- verify / certificate handoff と trusted boundary を P5H-10 に明示した。
- AI 向け Machine fast path を守る acceptance criteria を全 milestone に入れた。
- 現在実装済みの Human tactic bridge を再実装タスクにせず、Phase 5 Human session/API 層の前提として扱った。
```
