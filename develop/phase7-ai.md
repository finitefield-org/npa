以下は **Phase 7: AI探索** の詳細設計です。
Phase 7 の目的は、Phase 1〜6 で作った kernel・certificate・tactic・IDE/API・標準ライブラリを使って、**AIが証明候補を探索し、kernel / canonical certificate / independent checker が検査できるものだけを採用する仕組み**を作ることです。

対象はこの5つです。

```text
- premise retrieval
- tactic generation
- best-first search
- error repair
- proof minimization
```

実装メモ（2026-05-21）:

```text
- Phase 7 MVP M0-M9 は crates/npa-api の library substrate として実装済み
- no-model MVP profile、deterministic search controller、replay / verify closure、
  training trace identity、M9 integration fixtures は同 crate の unit tests で固定している
- crates/npa-api の Phase 7 controller は非信頼 producer / orchestrator であり、
  replay / verify と canonical certificate check を通るまで証明の受理根拠ではない
- Phase 7 MVP の候補生成は Phase 5 Machine API と Phase 3 AI Machine Surface を使い、
  Human Surface source、notation、open scope、pretty text を candidate identity / ranking / replay / verify の根拠にしない
- M10-M13 は Phase 7.5 / later profile であり、現行 MVP の成功条件ではない
```

設計原則は一貫してこれです。

```text
AIは信用しない。
AIは候補を出す。
tactic engine が試す。
kernel、canonical certificate verifier、independent checker が検証する。
検証済みでないものは証明ではない。
```

LeanDojo 系の既存研究でも、proof state・tactic・premise を抽出して、Lean とプログラム的に対話する仕組みがAI定理証明の基盤になっています。また、LeanDojo の論文は、premise selection が大規模ライブラリ上の証明探索で重要なボトルネックであると説明しています。([LeanDojo][1])

---

# 1. Phase 7 の全体像

Phase 7 では、証明探索を次のようなループにします。

```text
current proof state
  ↓
premise retrieval
  ↓
tactic generation
  ↓
tactic execution
  ↓
kernel-checked proof state transition
  ↓
best-first search
  ↓
proof found?
  ↓ yes
proof minimization
  ↓
certificate generation
  ↓
verified proof
```

図にするとこうです。

```text
┌──────────────────────────────┐
│ Structured Proof State        │
│ context, target, goals        │
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Premise Retriever             │
│ exact/apply/rw/simp candidates│
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Tactic Generator              │
│ templates + model + heuristics│
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Best-first Search             │
│ state graph exploration       │
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Tactic Execution API          │
│ intro/exact/apply/rw/simp/... │
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Error Repair                  │
│ type mismatch, rw fail, etc.  │
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Kernel / Cert / Indep Checker │
│ final verification            │
└───────────────┬──────────────┘
                ↓
┌──────────────────────────────┐
│ Proof Minimization            │
│ shorter, cleaner proof        │
└──────────────────────────────┘
```

---

# 2. Phase 7 の入出力

## 2.1 入力

Phase 7 の入力は、Phase 5 の structured proof state です。
Phase 7 MVP は独自の `/ai/*` proof-state / tactic execution protocol を定義せず、
`develop/phase5-ai.md` の Machine API client として実装します。
探索ノードの identity は `state_id` や text tactic ではなく、次の Phase 5 AI payload で固定します。

```text
required Phase 5 AI handles:
  - session_id
  - session_root_hash
  - snapshot_id
  - state_fingerprint
  - goal_id
  - MachineTacticCandidate raw payload
  - deterministic_budget
  - candidate_hash / proof_delta_hash for successful replay steps
```

MVP の proof search controller は `/machine/snapshots/get`、`/machine/search/for_goal`、
`/machine/tactics/batch`、`/machine/replay`、`/machine/verify` を使います。
`/machine/tactics/run` は Phase 5 の単一候補 API として存在しますが、Phase 7 MVP の探索アルゴリズムには使いません。
run-based controller を追加する場合は、stats、repair、deferred candidate、replay step mapping を batch 版と別に固定する
non-MVP profile とします。
`/machine/prompt_payload` は ModelGenerator を有効化した後だけ使う non-MVP 入力整形 API であり、
deterministic MVP の必須経路に入れてはいけません。

実装上の前提として、Phase 7 MVP は Phase 5 Machine API contract が実装済みであることを要求します。
Phase 5 実装より先に Phase 7 controller を作る場合でも、Phase 7 独自の簡易 protocol を定義してはいけません。
その場合は `MachineApiClient` のような境界を置き、production 実装は Phase 5 endpoint と同じ request / response /
error taxonomy を使う adapter、テスト実装は同じ contract を満たす deterministic fake に限定します。
fake / mock は replay plan、certificate、state fingerprint、candidate hash、training identity には入れてはいけません。

Phase 7 MVP が `/machine/snapshots/get` を呼ぶ場合、request の `include_pretty` は常に `false` に固定します。
`get_snapshot(...)` と疑似コードで書く箇所は、すべて
`/machine/snapshots/get { include_pretty = false }` を送る helper の省略表記です。
debug / UI 用に pretty 表示が必要な場合は、探索・ranking・trace identity とは別の表示用呼び出しに分けます。
pretty 付き response を候補生成、goal selection、priority、dedupe、training identity に使ってはいけません。

MVP の正本は次だけです。

```text
current state:
  Phase 5 MachineProofSnapshot
  session_id / snapshot_id / state_fingerprint / goal_id

tactic candidate:
  Phase 5 raw MachineTacticCandidate wire payload
  score や source などの Phase 7 metadata は candidate の外側にだけ持つ

transition:
  /machine/tactics/batch の per-candidate success result
  candidate_hash / deterministic_budget_hash / proof_delta_hash / next_state_fingerprint

accepted proof:
  complete MachineReplayPlan
  /machine/replay に通ること
  /machine/verify の status = verified response
```

この文書に残る `state_id`、`proof_script`、text tactic、`/ai/*` 形式の例は、
明示的に non-MVP wrapper と書いた箇所を除き、説明用の表示です。
MVP の wire contract として実装してはいけません。
特に `rw [Nat.add_zero]` のような text tactic は表示用であり、proof server へ送る payload は
必ず raw `MachineTacticCandidate` です。

```json
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "open_goals": ["g1"],
  "goals": [
    {
      "goal_id": "g1",
      "goal_fingerprint": "sha256:...",
      "context": [
        {
          "name": "n",
          "type": "Nat"
        }
      ],
      "target": {
        "pretty": "n + 0 = n",
        "head": "Eq",
        "constants": ["Eq", "Nat.add", "Nat.zero"],
        "core_hash": "sha256:..."
      },
      "allowed_tactics": ["intro", "exact", "apply", "rw", "simp-lite"]
    }
  ]
}
```

`pretty` や `statement` は人間・AI向け表示なので、Phase 6 の表示用省略として `0` を使うことがあります。
探索や certificate 検査で使う構造情報は `constants` / `core_hash` 側であり、`0` は `Nat.zero` への参照として扱います。

加えて、Phase 5 `/machine/search/for_goal` が返す verified premise metadata を使います。
Phase 6 の theorem index は、その response を構築するための upstream artifact または将来の ranking sidecar です。

```json
{
  "name": "Nat.add_zero",
  "statement": "∀ n : Nat, n + 0 = n",
  "attributes": ["simp", "rw"],
  "suggested_candidates": [
    {
      "status": "validated",
      "candidate_hash": "sha256:...",
      "candidate": {
        "kind": "simp-lite",
        "rules": [
          {
            "name": "Nat.add_zero",
            "decl_interface_hash": "sha256:...",
            "direction": "forward"
          }
        ]
      }
    }
  ],
  "axioms_used": []
}
```

## 2.2 出力

Phase 7 MVP が内部的に返す成功結果は、text proof ではなく replay / verify artifact です。
外部に `/ai/prove` のような wrapper を出す場合も、この情報から派生した表示だけを返し、
表示文字列を証明の正本にしてはいけません。

```rust
struct Phase7VerifiedProof {
    replay_plan: MachineReplayPlan,
    final_snapshot_id: SnapshotId,
    final_state_fingerprint: Hash,
    verify_response: MachineVerifySuccess,
    search_stats: SearchStats,
    minimization_stats: MinimizationStats,
    trace_events: Vec<Phase7TraceEvent>,
    training_trace_records: Vec<Phase7TrainingTraceRecord>,
}
```

`MachineReplaySuccess` と `MachineVerifySuccess` は Phase 5 Machine API の success response body を指す別名です。
Phase 7 独自の success schema を新しく定義してはいけません。
`Phase7TrainingTraceRecord` は 9.1 の `trace_schema = "npa.phase7.training-trace.v1"` を持つ
training trace record JSON object を指す別名です。

失敗時：

```rust
struct Phase7SearchFailure {
    reason: SearchFailureReason,
    best_partial_replay_prefix: Option<Vec<MachineReplayStep>>,
    best_snapshot_id: Option<SnapshotId>,
    best_state_fingerprint: Option<Hash>,
    remaining_goals: Option<Vec<GoalSummary>>,
    search_stats: SearchStats,
    trace_events: Vec<Phase7TraceEvent>,
    training_trace_records: Vec<Phase7TrainingTraceRecord>,
}
```

`SearchStats` の MVP field は次に固定します。

```rust
struct SearchStats {
    nodes_expanded: u64,
    candidates_evaluated: u64,
    scheduler_stops: u64,
    zero_progress_scheduler_stops: u64,
    closed_node_replay_rejections: u64,
    closed_node_verify_rejections: u64,
    controller_errors: u64,
    no_candidate_stops: u64,
    max_depth_stops: u64,
    best_partial_updates: u64,
}

struct MinimizationStats {
    pass_kinds_attempted: u64,
    rebuilt_plans: u64,
    replay_attempts: u64,
    verify_attempts: u64,
    accepted_proposals: u64,
}
```

`SearchFailureReason` は次に固定します。

```rust
enum SearchFailureReason {
    QueueExhausted,
    SearchBudgetExceeded { limit: SearchBudgetLimit },
    MachineControllerError {
        endpoint: String,
        error_kind: String,
        error_phase: Option<String>,
        diagnostic_hash: Option<Hash>,
    },
    NoCandidateForSelectedGoal { goal_id: GoalId },
}

enum SearchBudgetLimit {
    WallClock,
    MaxNodes,
    MaxDepth,
}
```

`SearchFailureReason` は controller 全体の最終終了理由です。
node ごとの no-candidate、zero-progress scheduler stop、repair-chain stop、closed-node replay/verify rejection は
`search_stats` / trace の node-local stop reason に記録し、探索 queue に他の node が残っている限り
final `SearchFailureReason` を上書きしてはいけません。
`NoCandidateForSelectedGoal` を final reason にしてよいのは、初期 node の selected goal で候補が 1 件も作れず、
かつ探索 queue に他の node が存在しない場合だけです。
`MachineControllerError.error_phase` と `diagnostic_hash` は、Phase 5 top-level error response に
`error.phase` / `error.diagnostic_hash` が存在する場合は `Some(...)` にし、存在しない transport / resource layer error では
`None` にします。
controller error を final reason に変換する箇所では `machine_controller_error_reason(endpoint, result)` を使い、
4 field すべてを埋めます。
`result` が Phase 5 `MachineApiDiagnostic` を持つ top-level error の場合、`error_kind` は Phase 5 の `error.kind`
wire string をそのまま使います。
Machine API response body を得られない transport layer failure では `error_kind = "transport_error"`、
endpoint contract 外の response-size / process / memory guard など resource layer failure では
`error_kind = "resource_error"` に固定します。
これら 2 つは Phase 7 controller-local final reason 用の wire string であり、Phase 5 error kind として送受信してはいけません。

疑似コード中の `SearchFailure(reason, best_partial)` は、次のように `Phase7SearchFailure` へ展開します。

```text
Phase7SearchFailure.reason = reason
if best_partial = Some(node):
  Phase7SearchFailure.best_partial_replay_prefix = Some(node.replay_steps)
  Phase7SearchFailure.best_snapshot_id = Some(node.snapshot_id)
  Phase7SearchFailure.best_state_fingerprint = Some(node.state_fingerprint)
  Phase7SearchFailure.remaining_goals = Some(node.goals)
else:
  Phase7SearchFailure.best_partial_replay_prefix = None
  Phase7SearchFailure.best_snapshot_id = None
  Phase7SearchFailure.best_state_fingerprint = None
  Phase7SearchFailure.remaining_goals = None
Phase7SearchFailure.search_stats = accumulated deterministic counters
Phase7SearchFailure.trace_events = accumulated Phase7TraceEvent list
Phase7SearchFailure.training_trace_records = accumulated Phase7TrainingTraceRecord list
```

`trace_events` は Phase 7 proof search 全体の top-level trace artifact です。
`record_*` で始まる擬似コード関数は、`record_training_trace_batch` を除き、
controller-local `trace_events` 配列に `Phase7TraceEvent` を append します。
`record_training_trace_batch` は controller-local `training_trace_records` 配列に
`Phase7TrainingTraceRecord` を append する関数であり、`trace_events` には書き込みません。
`trace_events` は `Phase7VerifiedProof` と `Phase7SearchFailure` の両方で返し、
training trace JSON とは別 artifact として扱います。
`trace_events` は replay plan、certificate、state fingerprint、candidate hash には入れてはいけません。
MVP controller は training trace を暗黙の file / external sink に直接書きません。
caller が `training_trace_records` を JSON array artifact として永続化します。

`SearchStats` counter は controller が観測した deterministic event の発生順に更新し、HashMap iteration order や
wall-clock race に依存してはいけません。
`nodes_expanded` は `nodes_expanded += 1` と同じ箇所で増やします。
`candidates_evaluated` は `/machine/tactics/batch` の `results.length` だけを加算します。
top-level controller error、prefix 外 candidate、scheduler stop 中 candidate は加算しません。
`scheduler_stops` は `partial_timeout` / `partial_resource_limit` response を受け取るたびに増やします。
`zero_progress_scheduler_stops` は、そのうち `results.length = 0` だった response だけを数えます。
`search_stats` は proof search 中の counter だけです。
minimization 中の replay / verify 試行は `search_stats` に加算せず、`minimization_stats` に記録します。
`MinimizationStats.pass_kinds_attempted` は MVP pass kind に入った回数で、通常は 3 です。
`rebuilt_plans`、`replay_attempts`、`verify_attempts` は minimization proposal ごとの実試行回数です。
`accepted_proposals` は replay / verify の両方に通って `current_plan` に採用された proposal 数です。

失敗は「偽」を意味しません。
探索予算内で証明が見つからなかっただけです。

---

# 3. Premise retrieval

## 3.1 目的

premise retrieval は、現在の goal に使えそうな定理・補題・定義・rewrite rule を探す機構です。

例：

```text
goal:
  n : Nat
  ⊢ n + 0 = n
```

retrieval 結果：

```text
Nat.add_zero
Eq.refl
Nat.add_succ
Nat.zero_add
```

AI証明探索では、LLMにライブラリ全体を渡すのではなく、**現在の goal に関係する少数の premise だけを渡す**ことが重要です。

LeanDojo の ReProver は retrieval-augmented な証明器として、巨大ライブラリから使う premise を選ぶ方針を採っています。LeanDojo は proof state・tactic・premise などを抽出し、Lean環境とプログラム的に対話できることを特徴としています。([arXiv][2])

---

## 3.2 Premise index

Phase 6 の theorem index を拡張して、検索用に次の情報を持たせます。
ただし Phase 7 MVP の探索器は独自の canonical premise index を定義しません。
MVP での premise の正本は Phase 5 `/machine/search/for_goal` の response です。

Phase 7 側に持ってよい index は、Phase 5 response を cache / ranking する非信頼 sidecar だけです。
その sidecar は certificate、replay plan、state fingerprint、candidate hash に入れてはいけません。

MVP で cache sidecar を持つ場合の保存単位は次に限定します。

```rust
struct Phase7PremiseCacheEntry {
    premise_ref: Phase7PremiseRef,
    universe_params: Vec<MachineUniverseParamName>,
    statement_core_hash: Hash,
    statement_head: Option<MachineGlobalRefView>,
    axioms_used: Vec<MachineAxiomRefWire>,
    modes: Vec<MachineTheoremMode>,
    response_index: u32,
}
```

pretty statement、attribute、rewrite_info、embedding id、graph neighbor、usage count、proof term size などは
non-MVP ranking sidecar です。
追加する場合は canonical source artifact、ordering、fingerprint input を別途固定します。

MVP で Phase 7 が保存・比較してよい verified premise locator は、
Phase 5 search response の `global_ref.module`、`global_ref.name`、`global_ref.export_hash`、
`global_ref.decl_interface_hash`、`universe_params`、`statement.core_hash`、`axioms_used` です。
これは enclosing `query_fingerprint` / `session_root_hash` と組み合わせた場合だけ verified premise identity になります。
locator 単体には `certificate_hash` がないため、release artifact や別 session にまたがる identity として使ってはいけません。
`usage_count`、`embedding_id`、`graph_neighbors`、自然言語 metadata、ranking score は non-MVP sidecar です。

重要な metadata：

```text
head_symbol:
  Eq, And, Or, Nat.add, List.append など

constants:
  定理文に出現する定数

attributes:
  simp, rw, intro, elim, trans など

rewrite_info:
  lhs, rhs, orientation, safe

axiom_deps:
  Classical.choice などに依存するか

usage_count:
  過去の証明でよく使われるか
```

---

## 3.3 accessible premises

検索対象は、現在の定理からアクセス可能な premise に限定します。

使ってよいもの：

```text
- import 済みモジュールの公開定理
- 現在のモジュールで既に定義済みの定理
- local context の仮定
```

MVP の `/machine/search/for_goal` が返す theorem index は Phase 5 の契約に従います。
Phase 7 MVP では、検索 API の premise は direct verified imports 由来の public theorem / axiom entry に限定します。
local context の仮定は premise search result ではなく、BuiltinGenerator / TemplateGenerator が
`TacticHead::Local` や `RawMachineTerm.source` として候補化します。
MVP で local context から自動生成するのは、まず `Exact` のように raw source を一意に作れる候補です。
local equality hypothesis から `Rw` を作る処理は、rewrite direction、site、args の決定規則を
別途固定するまで non-MVP とします。
current module の checked prior declarations を premise search に含める場合は non-MVP とし、
`Imported` / `CurrentModule` の premise identity と canonical ordering を別途固定します。

使ってはいけないもの：

```text
- まだ後で定義される定理
- import していないモジュールの定理
- custom axiom 禁止モードでの axiom 依存定理
- high-trust mode で許可されていない公理依存定理
```

これは非常に重要です。
未来の定理を使うと、循環依存や不正な証明になります。

---

## 3.4 検索モード

Premise retrieval は複数モードを合成します。
MVP の wire request に指定する mode は Phase 5 `MachineTheoremMode` の
`exact`, `apply`, `rw`, `simp` だけです。
lexical / type-aware / graph-aware / embedding は Phase 7 内部 ranking または将来 profile の候補であり、
Phase 5 MVP endpoint の mode として送ってはいけません。

```text
MVP wire modes:
  1. exact retrieval
  2. apply retrieval
  3. rw retrieval
  4. simp retrieval

non-MVP ranking/search sidecars:
  5. lexical retrieval
  6. type-aware retrieval
  7. graph-aware retrieval
  8. embedding retrieval
```

### exact retrieval

target と定理の結論が一致するかを調べます。

```text
goal:
  ⊢ n + 0 = n

candidate:
  Nat.add_zero : ∀ n, n + 0 = n

MVP result:
  verified premise metadata only

non-MVP candidate generation:
  MachineTacticCandidate::Exact
```

### apply retrieval

定理の結論を target に unify できるかを調べます。

```text
goal:
  ⊢ x = z

candidate:
  Eq.trans : x = y -> y = z -> x = z

MVP result:
  verified premise metadata only

non-MVP candidate generation:
  MachineTacticCandidate::Apply
```

### rw retrieval

target 内の subterm に rewrite rule が使えるか調べます。

```text
target:
  f (n + 0) = f n

rule:
  Nat.add_zero : n + 0 = n

MVP result:
  verified premise metadata
  MachineTacticCandidate::Rw only if Phase 5 suggested_candidates already includes it
```

### simp retrieval

`simp-lite` が使う safe rewrite rule を返します。

```text
Nat.add_zero
Nat.zero_add
List.append_nil
List.map_nil
```

### lexical retrieval

名前や文字列で検索します。

```text
query:
  add zero nat

results:
  Nat.add_zero
  Nat.zero_add
```

### type-aware retrieval

定理文の型構造を使って検索します。

```text
head target:
  Eq

target constants:
  Nat.add, Nat.zero

preferred:
  Eq を結論に持ち、Nat.add/Nat.zero を含む定理
```

### graph-aware retrieval

theorem dependency graph を使います。
MVP では graph-aware retrieval を Phase 5 query mode としては実装しません。
使う場合も Phase 7 内部の ranking sidecar です。
Phase 6 の certificate 由来 dependency graph / theorem index から導出できますが、
graph score は replay / certificate の正本ではありません。
Phase 9 の theorem graph は、これを大規模ライブラリ向けの schema / API / ranking 情報へ拡張します。

```text
現在使った定理:
  Nat.add_succ

近い定理:
  Nat.add_zero
  Nat.zero_add
  Nat.add_assoc
```

### embedding retrieval

自然言語名・定理文・proof state を embedding 化して検索します。

MVP には入れません。
Phase 7 後半または Phase 7.5 の non-MVP ranking sidecar として導入します。

---

## 3.5 Retriever score

各 premise にスコアを付けます。
Phase 5 MVP の `/machine/search/for_goal` response の `score` は JSON integer `0` に固定されています。
Phase 7 が探索優先度のために下のような score を計算する場合、それは Phase 7 内部の非信頼 metadata です。
`MachineTacticCandidate`、`MachineReplayPlan`、certificate payload には入れません。

```text
score(premise, goal) =
  + exact_match_score
  + apply_unification_score
  + rewrite_match_score
  + simp_attribute_bonus
  + head_symbol_match
  + constants_overlap
  + name_similarity
  + graph_proximity
  + usage_frequency
  + embedding_similarity
  - axiom_penalty
  - proof_complexity_penalty
  - import_distance_penalty
```

例：

```json
{
  "name": "Nat.add_zero",
  "phase7_internal_score": "0.982",
  "modes": ["exact", "rw", "simp"],
  "suggested_candidates": [
    {
      "candidate": {
        "kind": "simp-lite",
        "rules": []
      },
      "display_text": "simp-lite"
    }
  ]
}
```

---

## 3.6 Premise retrieval API

Phase 7 MVP の premise retrieval は Phase 5 AI の `/machine/search/for_goal` を呼びます。
独立した `/ai/retrieve_premises` endpoint は MVP には入れません。

```json
POST /machine/search/for_goal
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "modes": ["exact", "apply", "rw", "simp"],
  "limit": 32,
  "filters": {
    "exclude_axioms": true
  }
}
```

返却された premise は `module`、`name`、`export_hash`、`decl_interface_hash`、
`universe_params`、`statement.core_hash`、`axioms_used` に固定された verified metadata として扱います。
`suggested_candidates` がある場合も、探索器はそれを証明として信用せず、必ず
`/machine/tactics/batch` に再投入します。

Phase 7 MVP が `/machine/search/for_goal` に送る query は次に固定します。

```text
phase7_mvp_premise_query:
  modes = ["exact", "apply", "rw", "simp"]
  limit = 32
  filters = {
    "exclude_axioms": true
  }
```

`modes` は Phase 5 `MachineTheoremMode` canonical order のまま送ります。
`filters.allowed_modules` は omitted に固定し、Phase 5 の canonical all-direct form を使います。
明示的な module subset、`allowed_modules = []`、limit 変更、axiom 依存 premise を許す query は non-MVP profile です。
Phase 5 の response wire field は `results` です。
Phase 7 の内部変数で premise と呼ぶ場合も、MVP 実装は `response.results` をそのまま読みます。

`/ai/retrieve_premises` は MVP にはありません。
後で追加する場合は、Phase 5 `/machine/search/for_goal` を包む wrapper として、
request / response schema、query fingerprint、snapshot identity check、error taxonomy を別途固定してから実装します。

---

# 4. Tactic generation

## 4.1 目的

tactic generation は、現在の proof state と retrieved premises から、次に試す tactic 候補を生成します。

入力：

```text
goal
context
retrieved premises
failed tactics
available tactics
search budget
```

出力：

```text
CandidateEnvelope list
```

例：

```json
{
  "candidates": [
    {
      "candidate": {
        "kind": "simp-lite",
        "rules": []
      },
      "phase7_candidate_payload_hash": "sha256:...",
      "candidate_hash": null,
      "metadata": {
        "source": "builtin",
        "rank": {
          "source_rank": 1,
          "source_index": 0,
          "builtin_kind_rank": 3
        },
        "score": 0,
        "display_text": "simp-lite",
        "premises_used": [],
        "expected_effect": "simplify",
        "cost_estimate": {
          "estimated_timeout_ms": 100,
          "risk": "low"
        },
        "trust_flags": {
          "uses_axioms": [],
          "contains_forbidden_tokens": false,
          "forbidden_token_class": null
        }
      }
    },
    {
      "candidate": {
        "kind": "rw",
        "rule": {
          "head": {
            "imported": {
              "name": "Nat.add_zero",
              "decl_interface_hash": "sha256:..."
            }
          },
          "universe_args": [],
          "args": [
            {"mode": "infer_from_target"}
          ]
        },
        "direction": "forward",
        "site": "eq_target_left"
      },
      "phase7_candidate_payload_hash": "sha256:...",
      "candidate_hash": "sha256:...",
      "metadata": {
        "source": "phase5_suggested",
        "rank": {
          "source_rank": 0,
          "source_index": 0,
          "builtin_kind_rank": 255
        },
        "score": 0,
        "display_text": "rw [Nat.add_zero]",
        "premises_used": [
          {
            "premise_ref": {
              "module": "Std.Nat.Basic",
              "name": "Nat.add_zero",
              "export_hash": "sha256:...",
              "decl_interface_hash": "sha256:..."
            },
            "universe_params": [],
            "statement_core_hash": "sha256:...",
            "axioms_used": []
          }
        ],
        "expected_effect": "rewrite",
        "cost_estimate": {
          "estimated_timeout_ms": 200,
          "risk": "medium"
        },
        "trust_flags": {
          "uses_axioms": [],
          "contains_forbidden_tokens": false,
          "forbidden_token_class": null
        }
      }
    }
  ]
}
```

---

## 4.2 tactic generator の構成

1つのAIモデルに全部任せない方がよいです。
複数の generator を合成します。

```text
TacticGenerator
  ├── BuiltinGenerator
  ├── TemplateGenerator
  ├── PremiseBasedGenerator
  ├── ModelGenerator
  ├── RepairGenerator
  └── ExplorationGenerator
```

### BuiltinGenerator

Phase 5 snapshot から決定的に確認できる範囲だけで定番 tactic を出します。
MVP で自動生成してよい builtin candidate は、raw `MachineTacticCandidate` を構文的に一意に作れるものだけです。
Phase 7 MVP は `goal.target.pretty` や human display text を読んで候補を作ってはいけません。
`goal.target.machine` を読む場合は Phase 3 Machine Surface parser を使い、ad hoc な文字列分割や部分一致を
Pi 判定、binder 名抽出、Eq 判定に使ってはいけません。
この parse 結果は untrusted prefilter であり、成功の根拠は常に Phase 5 tactic execution です。

```text
goal.target.machine を Phase 3 Machine Surface parser で parse し、
outer syntactic form が Pi / forall:
  Intro { name = phase7_fresh_intro_name(snapshot, goal, outer_binder_name) }

target が Eq t t:
  non-MVP unless Eq.refl head and fully explicit RawMachineTerm.source can be built deterministically

context に h があり、h.ty.core_hash == goal.target.core_hash:
  Exact { term.source = h.machine_name }
  only when h.machine_name is unique in goal.context and h.value = None

context に induction 対象候補 n がある:
  InductionNat { local_name = n } only when phase7_induction_nat_prefilter(snapshot, goal, n) returns true

goal.allowed_tactics に "simp-lite" が含まれる:
  SimpLite { rules = [] }
```

`phase7_fresh_intro_name` は次の deterministic rule で 1 つの `MachineLocalName` を選びます。

```text
phase7_fresh_intro_name:
  candidate bases, in order:
    1. outer_binder_name if it is a valid MachineLocalName
    2. "x"
    3. "h"
    4. "n"

  forbidden names:
    - goal.context[*].machine_name

  for each base:
    try base, base1, base2, ... in decimal suffix order
    return the first valid MachineLocalName not in forbidden names
```

Phase 7 MVP は `Phase5MachineSurfaceGlobalRootSet` を再実装しません。
`intro.name` が global root と衝突する場合は Phase 5 の state-dependent candidate validation が拒否します。
その失敗は証明失敗ではなく候補生成の失敗として trace に残します。
Phase 7 は同じ binder に対して別名の `Intro` をその場で再生成して即時再投入してはいけません。
別名 retry を有効にする場合は、Phase 5 が返した post-canonical `InvalidCandidate` を accepted repair failure にしない限り、
separate deterministic intro-name profile として候補生成段階で複数名を作る必要があります。
MVP では global root collision した `Intro` は通常の failed candidate として記録し、同じ node の他 candidate を優先します。

`Exact` candidate を builtin 生成する場合、`RawMachineTerm.source` は Phase 5 Machine Surface の
candidate execution scope で受理される fully explicit source でなければなりません。
`Eq.refl` の universe / type arguments や custom Eq family が必要になる候補を text から推測してはいけません。
その場合は候補を出さず、`rw` / `simp-lite` / retrieved `suggested_candidates` に任せます。
local `Exact` の MVP prefilter は `local.ty.core_hash == goal.target.core_hash` の syntactic core hash 一致だけです。
conversion、WHNF、local let 展開が必要な場合、Phase 7 は local `Exact` を生成しません。

`phase7_induction_nat_prefilter` は Phase 4 の `induction-nat` validator を再実装しません。
Phase 7 MVP で候補を出してよいのは、snapshot から reduction なしに次を確認できる場合だけです。
Phase 5 の `MachineProofSnapshot` wire payload は resolved `nat_family` 本体を返しません。
Phase 7 MVP は Nat family の一致判定をせず、Phase 5 が `allowed_tactics` から `induction-nat` を省くことで
session-level availability を伝える前提にします。

```text
phase7_induction_nat_prefilter:
  - goal.allowed_tactics に "induction-nat" が含まれる
  - selected local name が goal context 内で一意
  - local declaration は value = None の assumption
  - `goal.context` 配列順で induction target より後ろに local declaration がない
  - goal.target.free_locals に selected local.local_id が含まれる
```

ここでの「後ろ」は `MachineProofSnapshot` の `goal.context` 配列順です。
`local_id` は identity としてだけ使い、declaration order の判定に使ってはいけません。

local type が resolved Nat family と一致すること、conversion、motive extraction、Nat family coherence は
Phase 5 / Phase 4 の実行時検査に任せます。
上の prefilter を snapshot から確認できない場合、Phase 7 MVP は `InductionNat` を生成しません。

### TemplateGenerator

retrieved premise から tactic template を作ります。
下の表記は表示用です。
実装では対応する raw `MachineTacticCandidate` を生成します。

```text
premise P:
  exact P ...       non-MVP
  apply P          non-MVP
  rw [P]           only if Phase 5 suggested_candidates already gave a validated Rw,
                   or Phase 7 has a separately specified deterministic rewrite-analysis profile
  rw [<- P]        non-MVP unless the same profile defines backward rule selection
  simp-lite        only as SimpLite { rules = [] } or with exact SimpRuleRef from Phase 5 suggestion
```

Phase 5 MVP の `/machine/search/for_goal` は `exact` / `apply` modes を返すことがありますが、
goal-specific `Exact` / `Apply` `suggested_candidates` は返しません。
Phase 7 MVP も premise から `exact` / `apply` を一般生成しません。
`Exact` は local hypothesis や fully explicit known term のように raw source を一意に作れる場合だけ、
`Apply` は non-MVP とします。

### PremiseBasedGenerator

retriever の `suggested_candidates[*].candidate` をそのまま使います。
Phase 5 response の `candidate_hash` がある場合は、Phase 7 の `CandidateEnvelope.candidate_hash` に保存します。

```text
Nat.add_zero
  -> MachineTacticCandidate::Rw
  -> MachineTacticCandidate::SimpLite
```

### ModelGenerator

LLMまたは小型モデルに tactic 候補を出させます。
MVP では ModelGenerator は無効です。
有効化する場合も、モデル出力は `npa.machine_tactic_candidate.v1` の raw `MachineTacticCandidate`
として parse / validate し、metadata や text tactic を candidate の中に混ぜてはいけません。

入力は構造化します。

```json
{
  "goal": {
    "context": [
      {"name": "n", "type": "Nat"}
    ],
    "target": "n + 0 = n"
  },
  "premises": [
    {"name": "Nat.add_zero", "statement": "∀ n, n + 0 = n"},
    {"name": "Eq.refl", "statement": "∀ x, x = x"}
  ],
  "allowed_tactics": [
    "intro",
    "exact",
    "apply",
    "rw",
    "simp-lite",
    "induction-nat"
  ],
  "failed_candidates": []
}
```

出力：

```json
{
  "candidates": [
    {"kind": "simp-lite", "rules": []},
    {"kind": "induction-nat", "local_name": "n"}
  ]
}
```

### RepairGenerator

失敗した raw candidate と Phase 5 error から修正版を作ります。
MVP では、修正版も決定的に作れる raw `MachineTacticCandidate` に限ります。

```text
failed:
  exact h

error:
  type_mismatch

repair:
  simp-lite

non-MVP repair:
  exact @Eq.refl Nat n
```

### ExplorationGenerator

探索用に少し低確率の候補を混ぜます。

```text
apply Eq.trans       non-MVP
induction-nat n
rw [<- Nat.add_zero] non-MVP unless a backward rewrite candidate was validated
```

これは局所最適を避けるためです。

---

## 4.3 tactic candidate schema

Phase 7 MVP で proof server に送る候補は、Phase 5 AI の raw `MachineTacticCandidate` wire payload です。
Phase 7 はその外側に探索用 metadata を持つ `CandidateEnvelope` を使ってよいですが、
`candidate` object の内側には Phase 5 schema で許可された field だけを入れます。

`candidate_hash` は、Phase 5 `/machine/search/for_goal` の `suggested_candidates` から来た候補、
または Phase 5 tactic execution API が validation 後に返した候補にだけ付きます。
Phase 7 MVP proof search では、この execution API は `/machine/tactics/batch` です。
Phase 7 が新規生成した raw candidate には、Phase 5 に渡すまで `candidate_hash` を作ってはいけません。
Phase 7 trace JSON で未検証候補を保存する場合は `candidate_hash = null` にします。
一方、`phase7_candidate_payload_hash` は `CandidateEnvelope` 作成時に raw `candidate` object から計算し、
その envelope に保存します。これは Phase 7 内の dedupe / local log 用 identity であり、
Phase 5 request、replay plan、certificate には入れてはいけません。

```rust
struct CandidateEnvelope {
    candidate: MachineTacticCandidate,
    phase7_candidate_payload_hash: Hash,
    candidate_hash: Option<Hash>,
    metadata: CandidateMetadata,
}

struct CandidateMetadata {
    source: CandidateSource,
    rank: CandidateRankMetadata,
    score: Phase7Score,
    display_text: Option<String>,
    premises_used: Vec<Phase7PremiseUsage>,
    expected_effect: ExpectedEffect,
    cost_estimate: CostEstimate,
    trust_flags: TrustFlags,
    repair: Option<RepairCandidateMetadata>,
}

enum CandidateSource {
    Phase5Suggested,
    Builtin,
    Model,
    Exploration,
    Repair,
}

struct CandidateRankMetadata {
    source_rank: u8,
    source_index: u32,
    builtin_kind_rank: u8,
}

struct RepairCandidateMetadata {
    parent_candidate_hash: Hash,
    error_kind: FailedCandidateErrorKind,
    repair_depth: u32,
    chain_tried_payload_hashes: Vec<Hash>,
}

struct Phase7PremiseRef {
    module: ModuleName,
    name: FullyQualifiedName,
    export_hash: Hash,
    decl_interface_hash: Hash,
}

struct Phase7PremiseUsage {
    premise_ref: Phase7PremiseRef,
    universe_params: Vec<MachineUniverseParamName>,
    statement_core_hash: Hash,
    axioms_used: Vec<MachineAxiomRefWire>,
}

type Phase7Score = i64;

enum ExpectedEffect {
    IntroBinder,
    CloseGoal,
    Rewrite,
    Simplify,
    InductionSplit,
    Unknown,
}

struct CostEstimate {
    estimated_timeout_ms: u64,
    risk: CostRisk,
}

enum CostRisk {
    Low,
    Medium,
    High,
}

struct TrustFlags {
    uses_axioms: Vec<MachineAxiomRefWire>,
    contains_forbidden_tokens: bool,
    forbidden_token_class: Option<ForbiddenTokenClass>,
}
```

`CandidateSource` の JSON trace wire name は次に固定します。

```text
CandidateSource JSON:
  Phase5Suggested -> "phase5_suggested"
  Builtin         -> "builtin"
  Model           -> "model"
  Exploration     -> "exploration"
  Repair          -> "repair"
```

MVP では `CandidateMetadata.score = Phase7Score::zero()` に固定し、candidate ordering には使いません。
候補順序に使う値は `CandidateMetadata.rank` と repair metadata だけです。
`CandidateMetadata` の field はすべて Phase 7 trace / ranking 用の controller-local metadata です。
MVP 実装では `display_text` と `repair` を除く全 field を必ず埋め、Phase 5 の `candidate` object、
`/machine/tactics/*` request、`MachineReplayPlan`、certificate には入れてはいけません。
`RepairCandidateMetadata.chain_tried_payload_hashes` は repair chain 内で既に実行済みまたは失敗元になった
`phase7_candidate_payload_hash` の列です。
この列は repair の重複防止だけに使い、Phase 5 request、replay plan、certificate、candidate hash、
positive / negative identity には入れてはいけません。
JSON trace に repair metadata を保存する場合は `chain_tried_payload_hashes` を同名 field の hash array として保存します。

MVP の `ExpectedEffect` は `MachineTacticCandidate.kind` だけから決定します。
JSON trace の wire name は `intro_binder`、`close_goal`、`rewrite`、`simplify`、`induction_split`、`unknown` です。
`intro -> IntroBinder`、`exact -> CloseGoal`、`rw -> Rewrite`、`simp-lite -> Simplify`、
`induction-nat -> InductionSplit`、それ以外の将来 tactic は `Unknown` とします。
MVP の `CostEstimate` は raw `MachineTacticCandidate.kind` と `SimpLite.rules.length` だけで決め、
計測値や wall-clock race から更新してはいけません。
`CostRisk` の JSON trace wire name は `low`、`medium`、`high` です。

```text
CostEstimate MVP table:
  intro:
    { estimated_timeout_ms = 100, risk = Low }
  exact:
    { estimated_timeout_ms = 100, risk = Low }
  rw:
    { estimated_timeout_ms = 200, risk = Medium }
  simp-lite with rules.length = 0:
    { estimated_timeout_ms = 100, risk = Low }
  simp-lite with rules.length > 0:
    { estimated_timeout_ms = 200, risk = Medium }
  induction-nat:
    { estimated_timeout_ms = 500, risk = Medium }
  apply:
    { estimated_timeout_ms = 500, risk = High }
  future unknown tactic kind:
    { estimated_timeout_ms = 500, risk = High }
```

Phase 5 suggested candidate、builtin candidate、repair candidate はすべて同じ table を使います。
MVP の `TrustFlags.uses_axioms` は、`filters.exclude_axioms = true` の premise response と builtin 候補では空配列です。
`contains_forbidden_tokens` は raw `MachineTacticCandidate` が Phase 7 禁止 token filter に触れた場合だけ true ですが、
true の候補は batch に送らず破棄します。
`forbidden_token_class` は `contains_forbidden_tokens = true` の場合だけ `Some(...)` にし、
`contains_forbidden_tokens = false` の場合は `None`、JSON では `null` に固定します。
`Phase7PremiseRef` の JSON trace shape は Phase 5 `/machine/search/for_goal` result の `global_ref` object と同じ
`module` / `name` / `export_hash` / `decl_interface_hash` です。
`Phase7PremiseRef` は locator だけであり、premise identity 全体ではありません。
full identity が必要な場合は、これを enclosing `query_fingerprint` / `session_root_hash` と組み合わせます。
`Phase7PremiseUsage` の JSON trace shape は `premise_ref`、`universe_params`、`statement_core_hash`、
`axioms_used` を必須 field とします。
`premises_used` に display string だけ、または `Phase7PremiseRef` だけを保存してはいけません。
`TrustFlags.uses_axioms` は Phase 5 search / verify response と同じ `MachineAxiomRefWire` JSON shape を使います。

JSON：

```json
{
  "candidate": {
    "kind": "simp-lite",
    "rules": []
  },
  "phase7_candidate_payload_hash": "sha256:...",
  "candidate_hash": null,
  "metadata": {
    "source": "builtin",
    "rank": {
      "source_rank": 1,
      "source_index": 0,
      "builtin_kind_rank": 3
    },
    "score": 0,
    "display_text": "simp-lite",
    "premises_used": [],
    "expected_effect": "simplify",
    "cost_estimate": {
      "estimated_timeout_ms": 100,
      "risk": "low"
    },
    "trust_flags": {
      "uses_axioms": [],
      "contains_forbidden_tokens": false,
      "forbidden_token_class": null
    }
  }
}
```

`display_text` は log / UI / debugging 用です。
`/machine/tactics/run`、`/machine/tactics/batch`、`/machine/replay` へは渡しません。

---

## 4.4 禁止 tactic / token

AIが生成しても即座に破棄するものを決めます。

禁止：

```text
sorry
admit
axiom
unsafe
import
set_option unsafe
declare
eval
shell
external command
```

Phase 7 の tactic grammar では、そもそも許可 tactic だけを parse するのが安全です。
Phase 7 MVP の禁止 token filter は、raw `MachineTacticCandidate` と `phase7_candidate_payload_hash` を持つ
`CandidateEnvelope` を作った後の候補だけに適用します。
現行 MVP grammar だけを実装する場合、許可外 tactic kind は raw `MachineTacticCandidate` にならないため、
`ForbiddenTokenClass::DisallowedTacticKind` は到達不能です。
この値は将来の拡張 grammar や互換 ingestion で、raw candidate としては表現できるが
Phase 7 MVP の実行許可集合には入っていない tactic kind を捨てるための予約値です。
モデル出力や text tactic が raw `MachineTacticCandidate` として parse / validation できない段階で拒否される場合、
その rejection は `ForbiddenCandidateDiscarded` ではなく、対応する generator-local parse / schema rejection として扱います。
禁止 token filter が複数の token class に触れた場合、`ForbiddenTokenClass` JSON wire name の定義順で
最初の 1 つを `forbidden_token_class` として記録します。
`contains_forbidden_tokens = true` の candidate は `/machine/tactics/batch` に送らず、
`Phase7TraceEventKind::ForbiddenCandidateDiscarded` を記録してから破棄します。

MVP の禁止 token filter は、candidate の表示文字列や `display_text` ではなく、raw
`MachineTacticCandidate` payload だけを入力にします。
判定は次の順に固定します。

```text
phase7_forbidden_token_filter(candidate):
  1. candidate.kind が MVP allowed tactic kind に含まれない場合:
       DisallowedTacticKind
  2. candidate 内のすべての RawMachineTerm.source を wire payload order で走査する:
       Exact.term.source
       Apply.args[*].Term.source
       Rw.rule.args[*].Term.source
  3. 各 source を Phase 3 Machine Surface lexer の token-only API
     `lex_machine_surface_tokens` で token 化する。
     文字列分割、substring search、pretty text search は使わない。
  4. token の kind / spelling が下の forbidden token class に完全一致した最初の class を返す。
     whitespace / comment token は無視する。
  5. どの token にも一致しなければ contains_forbidden_tokens = false。
```

`RawMachineTerm.source` が Phase 3 Machine Surface lexer で token 化できない候補は、
`CandidateEnvelope` を作る前の generator-local parse / schema rejection として扱います。
その候補に対して `ForbiddenCandidateDiscarded` を記録してはいけません。
禁止 token は token 単位の完全一致だけで判定し、識別子・完全修飾名・文字列 literal の部分文字列一致は使いません。
Phase 3 の token-only API が返す `MachineSurfaceToken.spelling` をそのまま使い、case folding、
Unicode normalization、name resolution、pretty 化後の spelling は使いません。
たとえば `UnsafeName` や `Std.UnsafeName.foo` のような identifier / qualified name は、
token spelling が完全一致で `unsafe` にならないため `Unsafe` にはしません。
`Std.unsafe.foo` のように dotted name component の spelling が完全一致で `unsafe` の場合は `Unsafe` です。
`SetOptionUnsafe` は、whitespace / comment を除いた token stream に `set_option` token の直後に
`unsafe` token が現れる場合だけです。
ここで `set_option` / `unsafe` token は `IdentLike` または `Reserved` token の `spelling` が完全一致するものです。
`Sorry`、`Admit`、`Axiom`、`Unsafe`、`Import`、`Declare`、`Eval`、`Shell` も同様に
`IdentLike` または `Reserved` token の `spelling` が、それぞれ `sorry`、`admit`、`axiom`、`unsafe`、
`import`、`declare`、`eval`、`shell` に完全一致する場合だけです。
`ExternalCommand` は Phase 3 token-only API が `MachineSurfaceTokenKind::ExternalCommand` を返す場合だけです。
現行 MVP の Machine Surface grammar にその token がないため到達不能です。

```text
allowed:
  intro
  exact
  apply
  rw
  simp-lite
  induction-nat
```

将来 `have`, `constructor`, `cases`, `refine` を追加しても、許可リスト制を続けます。

---

# 5. Best-first search

## 5.1 目的

LLMに1本の証明を出させるのではなく、proof state graph を探索します。

```text
node = proof state
edge = tactic
goal = open goals が空
```

探索中の各 tactic は実際に実行され、成功した state だけが探索木に残ります。

DeepMind の AlphaProof も、形式システムと強化学習を組み合わせ、Lean環境内で証明を探索する方向のシステムとして報告されています。([Nature][3])

---

## 5.2 Search node

```rust
struct SearchNode {
    node_id: NodeId,

    session_id: SessionId,
    session_root_hash: Hash,
    initial_state_fingerprint: Hash,
    snapshot_id: SnapshotId,
    state_fingerprint: Hash,
    goals: Vec<GoalSummary>,

    replay_steps: Vec<MachineReplayStep>,

    depth: u32,
    cumulative_score: Phase7Score,

    last_candidate: Option<MachineTacticCandidate>,
    last_candidate_hash: Option<Hash>,
    used_premises: Vec<Phase7PremiseUsage>,

    parent: Option<NodeId>,
    status: NodeStatus,
}

struct PendingCandidate {
    goal_id: GoalId,
    candidate: CandidateEnvelope,
    repair_depth: u32,
    parent_candidate_hash: Hash,
    error_kind: FailedCandidateErrorKind,
    chain_tried_payload_hashes: Vec<Hash>,
}

struct RepairCandidateOutput {
    pending: Vec<PendingCandidate>,
    repeated_candidate_payload_hashes: Vec<Hash>,
}
```

MVP の `NodeId` は Phase 7 controller 内だけの deterministic trace id です。
root は `NodeId(0)`、child は snapshot materialization に成功して `SearchNode` を作る時点で
`NodeId(1)`, `NodeId(2)`, ... を割り当てます。
割り当て順は batch response `results` order と、その中の success item 処理順に従います。
`NodeId` は `MachineReplayPlan`、candidate hash、certificate payload には入れません。

MVP の `NodeStatus` は trace 用に次だけを使います。

```rust
enum NodeStatus {
    Queued,
    Expanded,
}
```

root / child 作成時は `Queued`、`visited` に追加して展開を開始した時点で `Expanded` にします。
duplicate `state_fingerprint` で skip した node は展開されないため、status を変更せず trace に skip reason を記録します。
MVP では `NodeStatus` に closed、failed、depth-stopped、replay-rejected、duplicate-skipped などの終端状態を追加しません。
それらは `Phase7TraceEvent` として記録し、priority queue や replay plan construction の入力には使いません。

MVP の trace event schema は次に固定します。
`event_index` は controller 内で 0 から始まる連番で、event 発生順に 1 ずつ増やします。

```rust
struct Phase7TraceEvent {
    event_index: u64,
    node_id: Option<NodeId>,
    state_fingerprint: Option<Hash>,
    goal_id: Option<GoalId>,
    kind: Phase7TraceEventKind,
    detail: Phase7TraceEventDetail,
}

enum Phase7TraceEventKind {
    DuplicateStateSkipped,
    NodeDepthStopped,
    NoCandidateForSelectedGoal,
    SchedulerStopped,
    ZeroProgressSchedulerStopped,
    RepairChainStopped,
    ClosedNodeReplayRejected,
    ClosedNodeVerifyRejected,
    ControllerError,
    NonAcceptedCandidateError,
    DeferredCandidateDropped,
    ForbiddenCandidateDiscarded,
}

enum Phase7TraceEventDetail {
    DuplicateStateSkipped {
        duplicate_state_fingerprint: Hash,
    },
    NodeDepthStopped {
        max_depth: u32,
    },
    NoCandidateForSelectedGoal {
        selected_goal_id: GoalId,
    },
    SchedulerStopped {
        scheduler_status: String,
        completed_prefix_len: u32,
    },
    ZeroProgressSchedulerStopped {
        scheduler_status: String,
        completed_prefix_len: u32,
    },
    RepairChainStopped {
        parent_candidate_hash: Hash,
        error_kind: FailedCandidateErrorKind,
        repair_depth: u32,
        reason: RepairChainStopReason,
        repeated_candidate_payload_hash: Option<Hash>,
    },
    ClosedNodeReplayRejected {
        endpoint: String,
        status: String,
    },
    ClosedNodeVerifyRejected {
        endpoint: String,
        status: String,
    },
    ControllerError {
        endpoint: String,
        error_kind: String,
        error_phase: Option<String>,
        diagnostic_hash: Option<Hash>,
    },
    NonAcceptedCandidateError {
        candidate_id: CandidateId,
        phase7_candidate_payload_hash: Hash,
        error_kind: Option<String>,
        phase: Option<String>,
        has_candidate_hash: bool,
        has_diagnostic_hash: bool,
    },
    DeferredCandidateDropped {
        candidate_id: CandidateId,
        phase7_candidate_payload_hash: Hash,
        reason: DeferredCandidateDropReason,
    },
    ForbiddenCandidateDiscarded {
        phase7_candidate_payload_hash: Hash,
        forbidden_token_class: ForbiddenTokenClass,
    },
}

enum DeferredCandidateDropReason {
    SchedulerStoppedCandidate,
    MaxTacticsPerNode,
    WallClockBudgetExceeded,
}

enum RepairChainStopReason {
    RepeatedError,
    RepeatedCandidate,
    MaxRepairDepth,
}

enum ForbiddenTokenClass {
    Sorry,
    Admit,
    Axiom,
    Unsafe,
    Import,
    SetOptionUnsafe,
    Declare,
    Eval,
    Shell,
    ExternalCommand,
    DisallowedTacticKind,
}
```

`Phase7TraceEventKind` の JSON wire name は次に固定します。

```text
DuplicateStateSkipped       -> "duplicate_state_skipped"
NodeDepthStopped            -> "node_depth_stopped"
NoCandidateForSelectedGoal  -> "no_candidate_for_selected_goal"
SchedulerStopped            -> "scheduler_stopped"
ZeroProgressSchedulerStopped -> "zero_progress_scheduler_stopped"
RepairChainStopped          -> "repair_chain_stopped"
ClosedNodeReplayRejected    -> "closed_node_replay_rejected"
ClosedNodeVerifyRejected    -> "closed_node_verify_rejected"
ControllerError             -> "controller_error"
NonAcceptedCandidateError   -> "non_accepted_candidate_error"
DeferredCandidateDropped    -> "deferred_candidate_dropped"
ForbiddenCandidateDiscarded -> "forbidden_candidate_discarded"
```

JSON trace では `Phase7TraceEvent.detail` を `detail` object として保存し、`kind` と同じ variant の field だけを入れます。
`Phase7TraceEvent` の top-level option field は kind ごとに次で固定します。

```text
DuplicateStateSkipped:
  node_id = Some(skipped node id)
  state_fingerprint = Some(skipped node state_fingerprint)
  goal_id = None

NodeDepthStopped:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = None

NoCandidateForSelectedGoal:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = Some(selected goal_id)

SchedulerStopped / ZeroProgressSchedulerStopped:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = Some(selected goal_id)

RepairChainStopped:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = Some(selected goal_id)

ClosedNodeReplayRejected / ClosedNodeVerifyRejected:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = None

ControllerError:
  node_id = Some(node.node_id) if a node is available, otherwise None
  state_fingerprint = Some(node.state_fingerprint) if a node is available, otherwise None
  goal_id = Some(goal_id argument) only for goal-scoped endpoint errors, otherwise None

NonAcceptedCandidateError / DeferredCandidateDropped / ForbiddenCandidateDiscarded:
  node_id = Some(node.node_id)
  state_fingerprint = Some(node.state_fingerprint)
  goal_id = Some(selected goal_id)
```

MVP の `detail` 必須 field は次です。

```text
DuplicateStateSkipped:
  duplicate_state_fingerprint

NodeDepthStopped:
  max_depth

NoCandidateForSelectedGoal:
  selected_goal_id

SchedulerStopped / ZeroProgressSchedulerStopped:
  scheduler_status
  completed_prefix_len

RepairChainStopped:
  parent_candidate_hash
  error_kind
  repair_depth
  reason
  repeated_candidate_payload_hash

ClosedNodeReplayRejected / ClosedNodeVerifyRejected:
  endpoint
  status

record_closed_node_replay_rejection(node, replayed):
  detail.endpoint = "/machine/replay"
  detail.status = replayed.status

record_closed_node_verify_rejection(node, verified):
  detail.endpoint = "/machine/verify"
  detail.status = verified.status

ControllerError:
  endpoint
  error_kind
  error_phase
  diagnostic_hash

NonAcceptedCandidateError:
  candidate_id
  phase7_candidate_payload_hash
  error_kind
  phase
  has_candidate_hash
  has_diagnostic_hash

DeferredCandidateDropped:
  candidate_id
  phase7_candidate_payload_hash
  reason

ForbiddenCandidateDiscarded:
  phase7_candidate_payload_hash
  forbidden_token_class
```

`NonAcceptedCandidateError.detail.error_kind` と `phase` は JSON string または `null` です。
MVP `/machine/tactics/batch` では、error item の `error_kind` / `phase` / `diagnostic_hash` / `retryable` が
missing または wire type 不一致の場合、`NonAcceptedCandidateError` ではなく
`batch_response_contract_violation` として探索全体を終了します。
したがって MVP batch 由来の `NonAcceptedCandidateError` は `phase = Some(...)`、
`has_diagnostic_hash = true` に固定されます。
error item の `error_kind` が Phase 5 `FailedCandidateErrorKind` 外の場合は、
raw に読める wire string を `error_kind = Some(raw_string)` として保存し、
`AcceptedCandidateFailure` にはしません。
`error_kind = None`、`phase = None`、`has_diagnostic_hash = false` は
run-based / non-MVP profile か、別途固定された lossy diagnostic ingestion profile でだけ使えます。
`ControllerError.detail.error_phase` と `diagnostic_hash` は JSON では必ず field を出し、
値が `None` の場合は `null` として保存します。
`record_machine_controller_error(node, endpoint, result, goal_id=None)` は
`Phase7TraceEventKind::ControllerError` を記録します。
`/machine/search/for_goal` と `/machine/tactics/batch` の selected-goal scoped error では
`goal_id=goal.goal_id` を渡し、それ以外の controller/setup error では省略します。
`record_duplicate_state_skipped(node, duplicate_state_fingerprint)` は
`Phase7TraceEventKind::DuplicateStateSkipped` を記録し、
`detail.duplicate_state_fingerprint = duplicate_state_fingerprint` に固定します。
`SchedulerStopped.detail.scheduler_status` と `ZeroProgressSchedulerStopped.detail.scheduler_status` は、
Phase 5 `/machine/tactics/batch` response top-level `status` の wire string に固定します。
MVP で使う値は `"partial_timeout"` または `"partial_resource_limit"` だけです。
`completed_prefix_len` は Phase 5 response の `completed_prefix_len` をそのまま保存し、
存在しない場合は `result.results.length` として扱います。
擬似コード中の `record_scheduler_stop(node, goal_id, result)` は `result.results.length > 0` の partial response だけに使い、
`Phase7TraceEventKind::SchedulerStopped` を 1 件記録します。
`record_zero_progress_scheduler_stop(node, goal_id, result)` は `result.results.length = 0` の partial response だけに使い、
`Phase7TraceEventKind::ZeroProgressSchedulerStopped` を 1 件記録します。
zero-progress の場合は `SchedulerStopped` を重ねて記録してはいけません。
`record_depth_stop(node, max_depth)` は `Phase7TraceEventKind::NodeDepthStopped` を記録し、
`detail.max_depth = max_depth` に固定します。
`DeferredCandidateDropReason` の JSON wire name は `scheduler_stopped_candidate`、
`max_tactics_per_node`、`wall_clock_budget_exceeded` です。
`RepairChainStopReason` の JSON wire name は `repeated_error`、`repeated_candidate`、`max_repair_depth` です。
`record_repair_chain_stop(node, goal_id, envelope, failure, reason, repeated_candidate_payload_hash=None)` は
`Phase7TraceEventKind::RepairChainStopped` を記録し、
`detail.parent_candidate_hash = failure.candidate_hash`、
`detail.error_kind = failure.error_kind`、
`detail.repair_depth = repair_depth_of(envelope)`、
`detail.reason = reason`、
`detail.repeated_candidate_payload_hash = repeated_candidate_payload_hash` に固定します。
JSON trace では `repeated_candidate_payload_hash` field を常に出し、`None` は `null` として保存します。
ここで `parent_candidate_hash` は「次に作るはずだった repair candidate の parent になる hash」、
つまり今回失敗した candidate の `failure.candidate_hash` です。
`envelope.metadata.repair.parent_candidate_hash` が存在する場合でも、その値を trace の
`parent_candidate_hash` にコピーしてはいけません。
`repair_depth` は今回失敗した candidate の current repair depth です。
`MaxRepairDepth` は `repair_depth_of(envelope) >= 2` のため、次の repair candidate を作ると
MVP 上限を超える場合にだけ使います。
`RepeatedCandidate` は、生成しようとした repair candidate の `phase7_candidate_payload_hash` が
`chain_tried_payload_hashes` に既に含まれている場合にだけ使います。
この場合だけ `repeated_candidate_payload_hash = Some(生成しようとした repair candidate の hash)` にし、
それ以外の reason では必ず `None` にします。
`scheduler_stopped_candidate` は、partial response の stopped candidate と、
zero-progress scheduler stop でその node 展開を終了するため再投入しない assigned candidate 全体に使います。
`DeferredCandidateDropped` は、一度 `assign_candidate_ids` で `candidate_id` を割り当てたが
`/machine/tactics/batch.results` に現れず、同じ node 展開中に再投入もしない candidate だけに記録します。
rank / filter / dedupe / forbidden token check で batch request 前に捨てた candidate には、この event を使いません。
`ForbiddenTokenClass` の JSON wire name は `sorry`、`admit`、`axiom`、`unsafe`、`import`、
`set_option_unsafe`、`declare`、`eval`、`shell`、`external_command`、`disallowed_tactic_kind` です。

`detail` は trace / debugging 用であり、priority queue、replay plan、certificate、state fingerprint、
candidate hash、training positive / negative identity には使いません。

`PendingCandidate` は同じ `SearchNode` を再度 queue に入れるための state ではなく、
その node を展開している間だけ使う local retry item です。
`visited` は `state_fingerprint` だけを key にするため、同じ state を pending repair 付きで
priority queue に戻してはいけません。

`PendingCandidate` は accepted candidate failure からだけ作ります。
したがって `parent_candidate_hash` と `error_kind` は、5.5 の `AcceptedCandidateFailure` から必ず設定できます。
`chain_tried_payload_hashes` は repair chain 内で「既に batch に送られた候補」と「次 repair の失敗元候補」の
`phase7_candidate_payload_hash` を、実行 / 失敗の発生順に保存します。
fresh / suggested / builtin candidate の失敗から最初の repair を作る場合は
`[failed_envelope.phase7_candidate_payload_hash]` にします。
repair candidate の失敗から次の repair を作る場合は、失敗した repair envelope の
`metadata.repair.chain_tried_payload_hashes` の末尾に
`failed_envelope.phase7_candidate_payload_hash` を追加した列にします。
生成した repair candidate の `phase7_candidate_payload_hash` がこの列に既に含まれる場合、その repair は
`PendingCandidate` にせず、`RepairCandidateOutput.repeated_candidate_payload_hashes` に入れます。
呼び出し側はその hash ごとに `RepairChainStopReason::RepeatedCandidate` で `RepairChainStopped` を記録します。
`limit_repairs(max_per_parent=3)` は `(goal_id, parent_candidate_hash, error_kind)` ごとに request order を保って
先頭 3 件だけを残します。
`PendingCandidate` を batch に送るときは `candidate` の `CandidateEnvelope` だけを wire payload に変換します。
`repair_depth`、`parent_candidate_hash`、`error_kind`、`chain_tried_payload_hashes` は
`CandidateEnvelope.metadata.repair` にもコピーし、
trace / ranking / repair limit には使いますが、Phase 5 `candidate` object には入れてはいけません。

`GoalSummary`：

```rust
struct GoalSummary {
    goal_id: GoalId,
    open_goal_index: u32,
    goal_fingerprint: Hash,
    target_hash: Hash,
    target_head: Option<MachineGlobalRefView>,
    target_free_local_count: u32,
    context_size: u32,
    expr_size: u32,
}
```

`GoalSummary` は `MachineProofSnapshot.goals` を response order、つまり `open_goals` order のまま走査して作ります。
`open_goal_index` はその 0-based index です。
`target_hash = MachineGoalView.target_hash`、`target_head = MachineGoalView.target.head` をそのままコピーし、
`target_free_local_count = MachineGoalView.target.free_locals.length`、
`context_size = MachineGoalView.context.length`、`expr_size = MachineGoalView.target.size` です。
`target_head` を比較や trace key に使う場合は Phase 5 `MachineGlobalRefView canonical bytes` を使い、
certificate-local `NameId` や display string に変換してはいけません。
`open_goal_count(node) = node.goals.length`、`total_open_goal_target_size(node) = sum(node.goals[*].expr_size)` とします。
priority / best partial / goal selection はこの derived `GoalSummary` だけを使い、pretty text や HashMap iteration order を使いません。

---

## 5.3 State identity

MVP の visited set は Phase 5 の `state_fingerprint` を key にします。
Phase 7 が独自に state hash を作り、`state_fingerprint` の代わりに使ってはいけません。

```text
visited_key = state_fingerprint
```

`snapshot_id` は `state_fingerprint` から導出される API handle ですが、
request には両方を渡し、Phase 5 に snapshot self-check と fingerprint check を行わせます。

alpha-equivalent な state をさらに重複排除する normalized goal multiset hash は、
将来の非信頼 pruning hint としてなら追加できます。
その場合も、候補実行、replay、verify の正本は常に `state_fingerprint` です。

---

## 5.4 優先度関数

best-first search では、優先度の高い node から展開します。
MVP の priority queue は、次の `node_priority_key` を lexicographic ascending で比較します。
値が小さい node を先に pop します。

```text
node_priority_key(node):
  - open_goal_count
  - node.depth
  - replay_steps.length
  - total_open_goal_target_size
  - state_fingerprint_lex_tiebreaker
  - node_id
```

MVP の `priority(node)` はこの key です。
`state_fingerprint_lex_tiebreaker` は、Phase 5 `HashString / Hash` canonical bytes と同じく、
`sha256:` prefix を除いた 32-byte digest bytes を lexicographic ascending で比較します。
JSON 表示文字列、hex string の locale-aware sort、`snapshot_id`、または raw `state_fingerprint` object の
implementation-specific ordering を使ってはいけません。
`state_fingerprint` まで同じ duplicate path 同士は、deterministic に小さい `node_id` を先に pop します。
`model_value_score`、`tactic_logprob_sum`、embedding score、usage frequency は deterministic MVP では使いません。
将来 score を入れる場合は、score の wire 型、丸め、tie-break、profile version を固定してから
`node_priority_key` の前または後ろに追加します。
MVP の `score_initial(initial_snapshot)` は `Phase7Score::zero()` です。
MVP の `update_score(parent_score, step, child_snapshot)` は `parent_score` をそのまま返します。

以下の加点式は non-MVP scoring profile の説明です。

```text
priority(node) =
  + model_value_score
  + tactic_logprob_sum
  + retrieval_score
  + goal_reduction_score
  + solved_goal_bonus
  + simplicity_bonus
  - depth_penalty
  - open_goal_penalty
  - term_size_penalty
  - repeated_state_penalty
  - expensive_tactic_penalty
  - axiom_penalty
```

具体例：

```text
priority =
  1.5 * value_model_score
+ 1.0 * tactic_score
+ 0.8 * retrieval_score
+ 0.7 * goal_reduction
+ 1.2 * closed_goal_count
- 0.2 * depth
- 0.4 * open_goal_count
- 0.1 * proof_term_size
- 2.0 * forbidden_axiom_penalty
```

---

## 5.5 Best-first search アルゴリズム

```python
def prove(session, initial_snapshot, search_budget, tactic_budget, scheduler_limits, batch_policy):
    next_node_seq = 0
    root = SearchNode(
        node_id=NodeId(next_node_seq),
        session_id=session.session_id,
        session_root_hash=session.session_root_hash,
        initial_state_fingerprint=initial_snapshot.state_fingerprint,
        snapshot_id=initial_snapshot.snapshot_id,
        state_fingerprint=initial_snapshot.state_fingerprint,
        goals=summarize_goals(initial_snapshot),
        replay_steps=[],
        depth=0,
        cumulative_score=score_initial(initial_snapshot),
        last_candidate=None,
        last_candidate_hash=None,
        used_premises=[],
        parent=None,
        status=NodeStatus.Queued,
    )
    next_node_seq += 1

    queue = PriorityQueue()
    queue.push(root, priority=priority(root))

    visited = set()
    best_partial = None
    nodes_expanded = 0
    stats = SearchStats.zero()
    trace_events = []
    training_trace_records = []
    failure_reason = SearchFailureReason.QueueExhausted
    depth_budget_hit = False
    initial_no_candidate_goal = None

    while queue:
        if search_budget.wall_clock_exceeded():
            failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.WallClock)
            break

        node = queue.pop_best()

        if node.state_fingerprint in visited:
            record_duplicate_state_skipped(
                node,
                duplicate_state_fingerprint=node.state_fingerprint,
            )
            continue
        visited.add(node.state_fingerprint)
        node.status = NodeStatus.Expanded

        snapshot_result = get_snapshot(
            node.session_id,
            node.snapshot_id,
            node.state_fingerprint,
            include_pretty=False,
        )
        if snapshot_result.status != "ok":
            record_machine_controller_error(node, endpoint="/machine/snapshots/get", result=snapshot_result)
            stats.controller_errors += 1
            return SearchFailure(
                reason=machine_controller_error_reason(
                    endpoint="/machine/snapshots/get",
                    result=snapshot_result,
                ),
                best_partial=best_partial,
            )
        snapshot = snapshot_result.snapshot

        if snapshot.open_goals_empty():
            replay_plan = build_replay_plan(
                protocol_version=session.protocol_version,
                session_root_hash=node.session_root_hash,
                initial_state_fingerprint=node.initial_state_fingerprint,
                steps=node.replay_steps,
                final_state_fingerprint=node.state_fingerprint,
            )
            replayed = machine_replay(node.session_id, replay_plan)
            if is_replay_controller_error(replayed):
                record_machine_controller_error(node, endpoint="/machine/replay", result=replayed)
                stats.controller_errors += 1
                return SearchFailure(
                    reason=machine_controller_error_reason(
                        endpoint="/machine/replay",
                        result=replayed,
                    ),
                    best_partial=best_partial,
                )
            if replayed.status != "ok":
                record_closed_node_replay_rejection(node, replayed)
                stats.closed_node_replay_rejections += 1
                continue
            verified = machine_verify(
                node.session_id,
                replayed.final_snapshot_id,
                replayed.final_state_fingerprint,
                mode="certificate",
            )
            if is_verify_controller_error(verified):
                record_machine_controller_error(node, endpoint="/machine/verify", result=verified)
                stats.controller_errors += 1
                return SearchFailure(
                    reason=machine_controller_error_reason(
                        endpoint="/machine/verify",
                        result=verified,
                    ),
                    best_partial=best_partial,
                )
            if verified.status == "verified":
                minimized_plan, minimized_replay, minimized_verify, minimization_stats = minimize_replay_plan(
                    replay_plan,
                    replayed,
                    verified,
                    session,
                    initial_snapshot,
                )
                return Phase7VerifiedProof(
                    replay_plan=minimized_plan,
                    final_snapshot_id=minimized_replay.final_snapshot_id,
                    final_state_fingerprint=minimized_replay.final_state_fingerprint,
                    verify_response=minimized_verify,
                    search_stats=stats,
                    minimization_stats=minimization_stats,
                    trace_events=trace_events,
                    training_trace_records=training_trace_records,
                )
            record_closed_node_verify_rejection(node, verified)
            stats.closed_node_verify_rejections += 1
            continue

        if best_partial is None or is_better_partial(node, best_partial):
            best_partial = node
            stats.best_partial_updates += 1

        if node.depth >= search_budget.max_depth:
            depth_budget_hit = True
            record_depth_stop(node, max_depth=search_budget.max_depth)
            stats.max_depth_stops += 1
            continue

        if nodes_expanded >= search_budget.max_nodes:
            failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.MaxNodes)
            break
        nodes_expanded += 1
        stats.nodes_expanded += 1

        goal = select_goal(snapshot)

        premises_result = machine_search_for_goal(snapshot, goal, query=phase7_mvp_premise_query)
        if premises_result.status != "ok":
            record_machine_controller_error(
                node,
                endpoint="/machine/search/for_goal",
                result=premises_result,
                goal_id=goal.goal_id,
            )
            stats.controller_errors += 1
            return SearchFailure(
                reason=machine_controller_error_reason(
                    endpoint="/machine/search/for_goal",
                    result=premises_result,
                ),
                best_partial=best_partial,
            )
        premises = premises_result.results
        fresh_candidates = generate_candidates(snapshot, goal, premises)

        pending_repairs = []
        deferred_candidates = []
        evaluated_for_node = 0
        batch_index = 0

        while not search_budget.wall_clock_exceeded():
            candidates = merge_node_candidates(
                deferred_candidates=deferred_candidates,
                pending_repairs=pending_repairs,
                fresh_candidates=fresh_candidates,
                goal_id=goal.goal_id,
            )
            fresh_candidates = []
            deferred_candidates = []
            candidates = rank_and_filter_with_bucket_priority(candidates)
            candidates = discard_forbidden_candidates(node, goal.goal_id, candidates)
            candidates = dedupe_by_canonical_candidate_payload(candidates)
            candidates = take_remaining_node_tactic_budget(
                candidates,
                search_budget.max_tactics_per_node - evaluated_for_node,
            )
            if len(candidates) == 0:
                if evaluated_for_node == 0:
                    record_no_candidate_for_selected_goal(node, goal.goal_id)
                    stats.no_candidate_stops += 1
                    if node.parent is None:
                        initial_no_candidate_goal = goal.goal_id
                break

            candidate_items, candidate_by_id = assign_candidate_ids(candidates)

            result = machine_tactics_batch(
                session_id=node.session_id,
                snapshot_id=node.snapshot_id,
                state_fingerprint=node.state_fingerprint,
                goal_id=goal.goal_id,
                candidates=candidate_items,
                deterministic_budget=tactic_budget,
                scheduler_limits=scheduler_limits,
                batch_policy=cap_batch_policy(batch_policy, len(candidate_items)),
            )

            if is_top_level_batch_error(result):
                record_machine_controller_error(
                    node,
                    endpoint="/machine/tactics/batch",
                    result=result,
                    goal_id=goal.goal_id,
                )
                stats.controller_errors += 1
                return SearchFailure(
                    reason=machine_controller_error_reason(
                        endpoint="/machine/tactics/batch",
                        result=result,
                    ),
                    best_partial=best_partial,
                )

            if is_batch_response_contract_violation(candidate_items, result):
                record_machine_controller_error(
                    node,
                    endpoint="/machine/tactics/batch",
                    result=batch_response_contract_violation_error(),
                    goal_id=goal.goal_id,
                )
                stats.controller_errors += 1
                return SearchFailure(
                    reason=batch_response_contract_violation_reason(),
                    best_partial=best_partial,
                )

            if batch_has_candidate_hash_mismatch(candidate_by_id, result):
                record_machine_controller_error(
                    node,
                    endpoint="/machine/tactics/batch",
                    result=suggested_candidate_hash_mismatch_error(),
                    goal_id=goal.goal_id,
                )
                stats.controller_errors += 1
                return SearchFailure(
                    reason=suggested_candidate_hash_mismatch_reason(),
                    best_partial=best_partial,
                )

            stopped_candidate_id = None
            if is_scheduler_partial(result):
                stats.scheduler_stops += 1
                if len(result.results) == 0:
                    record_zero_progress_scheduler_stop(node, goal.goal_id, result)
                    stats.zero_progress_scheduler_stops += 1
                    for candidate_item in candidate_items:
                        record_deferred_candidate_dropped(
                            node,
                            goal.goal_id,
                            DeferredCandidate(
                                candidate_id=candidate_item.candidate_id,
                                candidate=candidate_by_id[candidate_item.candidate_id],
                            ),
                            reason=DeferredCandidateDropReason.SchedulerStoppedCandidate,
                        )
                    break
                record_scheduler_stop(node, goal.goal_id, result)
                if len(result.results) < len(candidate_items):
                    stopped_candidate_id = candidate_items[len(result.results)].candidate_id
                    record_deferred_candidate_dropped(
                        node,
                        goal.goal_id,
                        DeferredCandidate(
                            candidate_id=stopped_candidate_id,
                            candidate=candidate_by_id[stopped_candidate_id],
                        ),
                        reason=DeferredCandidateDropReason.SchedulerStoppedCandidate,
                    )

            if len(result.results) > 0:
                record_training_trace_batch(
                    node=node,
                    goal=goal,
                    batch_index=batch_index,
                    candidate_items=candidate_items,
                    candidate_by_id=candidate_by_id,
                    result=result,
                    premises=premises,
                    training_trace_records=training_trace_records,
                )
                batch_index += 1

            evaluated_for_node += len(result.results)
            stats.candidates_evaluated += len(result.results)
            next_repairs = []
            completed_candidate_ids = set(item.candidate_id for item in result.results)
            deferred_candidates = [
                DeferredCandidate(
                    candidate_id=item.candidate_id,
                    candidate=candidate_by_id[item.candidate_id],
                )
                for item in candidate_items
                if item.candidate_id not in completed_candidate_ids
                and item.candidate_id != stopped_candidate_id
            ]

            for item in result.results:
                if item.status == "success":
                    envelope = candidate_by_id[item.candidate_id]
                    step = make_replay_step(
                        previous_state_fingerprint=node.state_fingerprint,
                        goal_id=goal.goal_id,
                        candidate=envelope.candidate,
                        deterministic_budget=tactic_budget,
                        candidate_hash=item.candidate_hash,
                        deterministic_budget_hash=result.deterministic_budget_hash,
                        proof_delta_hash=item.proof_delta_hash,
                        next_state_fingerprint=item.next_state_fingerprint,
                    )
                    child_result = make_child(
                        node,
                        item.next_snapshot_id,
                        item.next_state_fingerprint,
                        step,
                        node_id=NodeId(next_node_seq),
                        envelope=envelope,
                    )
                    if child_result.status != "ok":
                        record_machine_controller_error(
                            node,
                            endpoint="/machine/snapshots/get",
                            result=child_result,
                        )
                        stats.controller_errors += 1
                        return SearchFailure(
                            reason=machine_controller_error_reason(
                                endpoint="/machine/snapshots/get",
                                result=child_result,
                            ),
                            best_partial=best_partial,
                        )
                    child = child_result.node
                    next_node_seq += 1
                    queue.push(child, priority=priority(child))
                else:
                    envelope = candidate_by_id[item.candidate_id]
                    failure = normalize_accepted_candidate_failure(
                        item,
                        deterministic_budget_hash=result.deterministic_budget_hash,
                    )
                    if failure is None:
                        record_non_accepted_candidate_error(node, goal.goal_id, envelope, item)
                        continue
                    if repeated_repair_error(envelope, failure):
                        record_repair_chain_stop(
                            node,
                            goal.goal_id,
                            envelope,
                            failure,
                            reason=RepairChainStopReason.RepeatedError,
                        )
                        continue
                    parent_repair_depth = repair_depth_of(envelope)
                    if parent_repair_depth >= 2:
                        record_repair_chain_stop(
                            node,
                            goal.goal_id,
                            envelope,
                            failure,
                            reason=RepairChainStopReason.MaxRepairDepth,
                        )
                        continue
                    repair_output = repair_candidate(
                        snapshot,
                        goal,
                        envelope,
                        failure,
                        repair_depth=parent_repair_depth + 1,
                    )
                    for repeated_hash in repair_output.repeated_candidate_payload_hashes:
                        record_repair_chain_stop(
                            node,
                            goal.goal_id,
                            envelope,
                            failure,
                            reason=RepairChainStopReason.RepeatedCandidate,
                            repeated_candidate_payload_hash=repeated_hash,
                        )
                    next_repairs.extend(repair_output.pending)

            if not next_repairs and not deferred_candidates:
                break
            if evaluated_for_node >= search_budget.max_tactics_per_node:
                for deferred in deferred_candidates:
                    record_deferred_candidate_dropped(
                        node,
                        goal.goal_id,
                        deferred,
                        reason=DeferredCandidateDropReason.MaxTacticsPerNode,
                    )
                break

            pending_repairs = limit_repairs(
                [r for r in next_repairs if r.repair_depth <= 2],
                max_per_parent=3,
            )
            if not pending_repairs and not deferred_candidates:
                break

        if search_budget.wall_clock_exceeded():
            for deferred in deferred_candidates:
                record_deferred_candidate_dropped(
                    node,
                    goal.goal_id,
                    deferred,
                    reason=DeferredCandidateDropReason.WallClockBudgetExceeded,
                )
            break

    if search_budget.wall_clock_exceeded():
        failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.WallClock)
    elif initial_no_candidate_goal is not None and best_partial is not None and best_partial.parent is None:
        failure_reason = SearchFailureReason.NoCandidateForSelectedGoal(goal_id=initial_no_candidate_goal)
    elif failure_reason == SearchFailureReason.QueueExhausted and depth_budget_hit:
        failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.MaxDepth)

    return SearchFailure(reason=failure_reason, best_partial=best_partial)
```

`get_snapshot`、`machine_search_for_goal`、`make_child` 内の child snapshot materialization が Phase 5 top-level
error を返した場合は、Phase 7 controller/setup error として扱い、final `MachineControllerError` を返します。
これらは candidate failure ではないため、repair、negative training、`AcceptedCandidateFailure` 正規化の対象にしません。

`best_partial` は snapshot materialization に成功し、かつ `snapshot.open_goals_empty() == false` の node だけで更新します。
初期 snapshot が closed で replay / verify rejection になった場合、`best_partial = None` のまま
`Phase7SearchFailure.best_partial_* = None` を返します。
closed node は `/machine/replay` と `/machine/verify` の両方に成功した時点で `VerifiedProof` として返すため、
`best_partial` には入れません。
snapshot lookup に失敗した node、closed-node replay/verify rejection になった node を
`Phase7SearchFailure.best_partial_*` に入れてはいけません。

`machine_replay` または `machine_verify` が closed node を reject した場合、その node は証明として採用しません。
ただし request/session binding、replay plan hash-chain validation、step `candidate_hash` /
`deterministic_budget_hash` mismatch、snapshot lookup、malformed verify request のような top-level controller/setup error は
fatal `MachineControllerError` として探索全体を終了します。
`is_replay_controller_error` と `is_verify_controller_error` は Phase 5 の `status` / `error.kind` /
`error.phase` だけで判定し、Phase 7 独自の verify status 名を作ってはいけません。
valid request に対する scheduler stop、certificate rejection、axiom rejection、closed でない final snapshot rejection は、
closed-node rejection として trace に記録して探索を続けます。

```text
is_replay_controller_error:
  true for:
    - status = "error" and error.kind in {
        "invalid_replay_plan",
        "unknown_session",
        "session_root_hash_mismatch",
        "state_fingerprint_mismatch",
        "replay_hash_mismatch",
        "invalid_machine_proof_state"
      }
  false for:
    - status = "scheduler_stopped"

is_verify_controller_error:
  true for:
    - status = "error", error.kind = "invalid_verify_request", and error.phase = "request_validation"
    - status = "error" and error.kind in {
        "unknown_session",
        "unknown_snapshot",
        "state_fingerprint_mismatch",
        "invalid_machine_proof_state"
      }
  false for:
    - status = "error", error.kind = "invalid_verify_request", and error.phase = "snapshot_lookup"
    - status = "error" and error.kind = "verify_failed"
    - status = "error" and error.kind = "disallowed_axiom"
```

MVP 実装では、success result から作った replay plan が `replay_hash_mismatch` や
`invalid_replay_plan` で reject される場合は Phase 5 integration bug としてテストで検出します。
実行時 controller は panic せず、final `MachineControllerError` として探索を終了します。
`scheduler_stopped` だけは replay controller error ではないため、closed-node rejection として別 node を試します。

`make_child` は `item.next_snapshot_id` / `item.next_state_fingerprint` を使って
`/machine/snapshots/get { include_pretty = false }` を呼び、
child snapshot を materialize してから、呼び出し元が渡した `node_id` で `SearchNode` を作ります。
`make_child` は `child.depth = node.depth + 1` に固定します。
`child.replay_steps` は `node.replay_steps` に `step` を末尾追加した配列です。
`child.session_root_hash` と `child.initial_state_fingerprint` は parent と同じ値を引き継ぎます。
`child.parent = Some(node.node_id)`、`child.last_candidate = Some(envelope.candidate)`、
`child.last_candidate_hash = Some(step.candidate_hash)`、
`child.used_premises = append_unique_premises(node.used_premises, envelope.metadata.premises_used)`、
`child.status = NodeStatus.Queued` とします。
`child.goals = summarize_goals(child_snapshot)`、
`child.cumulative_score = update_score(node.cumulative_score, step, child_snapshot)` とします。
MVP の `update_score` は deterministic に `node.cumulative_score` を引き継ぐだけでよいです。
`priority(child)` は materialized child snapshot の `child.goals` を使って計算します。

`SearchNode.used_premises` は root からその node までの path 全体で使われた premise の trace metadata です。
`append_unique_premises` は既存 `node.used_premises` の順序を保持し、`envelope.metadata.premises_used` を候補 metadata 内の順序で後ろに追加します。
重複判定は `Phase7PremiseUsage` の `premise_ref`、`universe_params`、`statement_core_hash`、
`axioms_used` の canonical bytes 完全一致で行い、
既に存在する premise は追加しません。
この field は replay plan、certificate、state fingerprint、candidate hash には入れてはいけません。

`search_budget.max_nodes` は、`visited` に入った後、候補生成を開始する前の unique `SearchNode` 展開数を数えます。
同じ `state_fingerprint` で skip された node は数えません。
`search_budget.max_depth` は `node.depth >= max_depth` の node を展開しない上限です。
ただし open goals が空の node は depth check の前に `/machine/replay` と `/machine/verify` へ進めます。
depth stop は node-local stop なので、その場で final `SearchFailureReason` を上書きしません。
MVP では queue が最終的に尽き、かつ少なくとも 1 回 `NodeDepthStopped` を記録していて、
他の final reason が設定されていない場合だけ、final reason を
`SearchBudgetExceeded { limit = MaxDepth }` にします。
`wall_clock_ms` は Phase 7 controller の外側停止であり、replay plan、candidate hash、training negative には入りません。

`is_better_partial(a, b)` は次の key を辞書順で比較し、小さい方を採用します。

```text
partial_key(node):
  - open_goal_count
  - total_open_goal_target_size
  - replay_steps.length
  - node.depth
  - state_fingerprint_lex_tiebreaker
  - node_id
```

ここで `open_goal_count` は小さいほど、`total_open_goal_target_size` は小さいほど、
`replay_steps.length` と `node.depth` は短いほどよいとします。
`state_fingerprint` が同じ duplicate path 同士では、`node_id` が小さいものを選びます。
`is_better_partial` は wall-clock、thread scheduling、HashMap iteration order、model sampling score に依存してはいけません。

`is_top_level_batch_error(result)` は、`/machine/tactics/batch` が per-candidate `results` container ではなく
request / controller / lookup level の error を返した場合です。
例は invalid batch policy、invalid deterministic budget、unknown session、snapshot/state mismatch、goal not open、
malformed request です。
これは candidate failure ではないため、`evaluated_for_node` に数えず、repair、negative training、
`AcceptedCandidateFailure` 正規化の対象にしません。
Phase 7 は controller/setup error として trace に記録し、final `MachineControllerError` を返して探索全体を終了します。
MVP 実装では、Phase 7 が生成する request がこの error を返す場合は実装バグとしてテストで検出します。

`is_batch_response_contract_violation(candidate_items, result)` は、top-level batch error ではない response が
Phase 5 `/machine/tactics/batch` response contract から外れた場合に true です。
MVP では次をすべて contract violation として扱います。

```text
batch response contract violation:
  - result.status が "ok" / "partial_timeout" / "partial_resource_limit" 以外
  - result.results が存在しない、または array ではない
  - result.results.length > candidate_items.length
  - result.results[*].candidate_id が request order prefix と一致しない
      i 番目の result は candidate_items[i].candidate_id でなければならない
  - result.results 内の candidate_id が重複する
  - result.results[*].candidate_id が candidate_items に存在しない
  - response.previous_state_fingerprint が存在しない、または request の state_fingerprint と一致しない
  - partial response で completed_prefix_len が存在しない、または result.results.length と一致しない
  - partial response で scheduler_artifact が存在しない
  - partial response なのに result.results.length == candidate_items.length
  - status = "ok" なのに completed_prefix_len または scheduler_artifact が存在する
  - response に必要な top-level deterministic_budget_hash がない
  - response.deterministic_budget_hash が request deterministic_budget から計算した hash と一致しない
  - result item の status が "success" / "error" 以外
  - success item に candidate_hash / proof_delta_hash / next_snapshot_id / next_state_fingerprint のいずれかがない
  - error item に error_kind / phase / diagnostic_hash / retryable のいずれかがない
```

contract violation は candidate failure ではなく Phase 5 integration / controller consistency error です。
Phase 7 は `record_training_trace_batch`、stats の `candidates_evaluated` 加算、replay step 作成、
repair、negative training より前にこの batch-level check を行います。
同じ response 内に処理可能に見える prefix があっても、その batch response 全体を破棄します。
このとき `endpoint = "/machine/tactics/batch"`、
`error_kind = "batch_response_contract_violation"`、`error_phase = None`、
`diagnostic_hash = None` に固定します。
`batch_response_contract_violation` は Phase 7 controller-local final reason string であり、
Phase 5 error kind として送受信してはいけません。
`batch_response_contract_violation_reason()` はこの段落の 4 field で
`SearchFailureReason::MachineControllerError` を作ります。
`batch_response_contract_violation_error()` は trace の `ControllerError.detail` を埋めるための
controller-local pseudo result であり、Phase 5 response として扱ってはいけません。

`/machine/tactics/batch` が `partial_timeout` / `partial_resource_limit` を返した場合、
`results` に含まれる prefix だけを処理します。
停止中だった candidate は success / deterministic failure のどちらにも数えず、repair や training negative にも使いません。
`is_scheduler_partial(result)` は `result.status in {"partial_timeout", "partial_resource_limit"}` です。
`scheduler_stops` は `is_scheduler_partial(result)` ごとに 1 増やします。
`partial_timeout` / `partial_resource_limit` で `results.length = 0` の場合は zero-progress scheduler stop として
`zero_progress_scheduler_stops` も 1 増やし、その `SearchNode` 展開を終了します。
`results.length > 0` の partial response では、`candidate_items[results.length]` を停止中だった candidate とみなし、
その candidate は同じ node 展開では捨てます。
`candidate_items[results.length + 1..]` の suffix だけを `deferred_candidates` として保持できます。
これにより、同じ scheduler limits で同じ停止 candidate を即時再投入してはいけません。
`status = "ok"` でも `batch_policy` の deterministic policy stop により `results.length < candidates.length` になることがあります。
partial では停止 candidate の次から、policy stop では `results` prefix の次からの suffix candidate を
`deferred_candidates` として同じ `SearchNode` 展開中に再投入します。
deferred candidate は実行されていないため、failure、negative training、repair parent には使いません。
`search_budget.max_tactics_per_node` に到達した場合だけ、残り suffix を捨てます。
捨てる deferred candidate には `DeferredCandidateDropped` を記録し、理由は tactic 数上限なら
`max_tactics_per_node`、scheduler partial の停止 candidate なら `scheduler_stopped_candidate`、
wall-clock 終了で node-local retry を続けない場合は `wall_clock_budget_exceeded` とします。

`merge_node_candidates` の bucket priority は次に固定します。

```text
1. deferred_candidates:
   直前 batch の retryable 未評価 suffix。scheduler partial の停止 candidate は含めない。
   `DeferredCandidate` wrapper の配列として直前 request order を保持し、この bucket 内では rerank しない。
   batch request を作るときは wrapper 内の `candidate` だけを使い、新しい `candidate_id` を割り当てる。
2. pending_repairs:
   `limit_repairs` 後の repair 候補。下の `repair_rank_key` だけで bucket 内 rank する。
3. fresh_candidates:
   この node の初回生成候補。通常の rank を使う。
```

`rank_and_filter_with_bucket_priority` は bucket 間の順序を入れ替えてはいけません。
`discard_forbidden_candidates(node, goal_id, candidates)` は ranked candidates を順に走査し、
`metadata.trust_flags.contains_forbidden_tokens = true` の candidate ごとに
`Phase7TraceEventKind::ForbiddenCandidateDiscarded` を記録して、その candidate を配列から除きます。
この discard は `rank_and_filter_with_bucket_priority` の後、`dedupe_by_canonical_candidate_payload` の前にだけ行います。
同じ `phase7_candidate_payload_hash` の forbidden candidate が複数ある場合も、dedupe 前なので各 candidate を 1 件ずつ trace に記録します。
`ForbiddenCandidateDiscarded` は batch request 前の discard なので、`candidate_id` は持ちません。
`dedupe_by_canonical_candidate_payload` で bucket をまたいで同じ `phase7_candidate_payload_hash` が出た場合は、
上の bucket priority で先に現れた候補を残します。
MVP では bucket 内 rank を済ませてから dedupe します。
同じ bucket 内で同じ `phase7_candidate_payload_hash` が複数出た場合も、bucket 内 rank で先に来た候補を残します。
MVP の bucket 内 rank は次に固定します。

```text
deferred_candidates:
  previous request order を保持する。bucket 内 rerank なし。

pending_repairs:
  repair_rank_key:
    - repair_depth ascending
    - parent_candidate_hash bytes lexicographic ascending
    - error_kind wire string lexicographic ascending
    - phase7_candidate_payload_hash bytes lexicographic ascending

fresh_candidates:
  fresh_rank_key:
    - source_rank
    - builtin_kind_rank
    - source_index
    - phase7_candidate_payload_hash bytes lexicographic ascending
```

`source_rank` は `CandidateSource::Phase5Suggested = 0`, `Builtin = 1`, `Model = 2`, `Exploration = 3`,
`Repair = 4` です。
MVP では `model` は無効なので出現しません。
`source_rank`、`source_index`、`builtin_kind_rank` は `CandidateMetadata.rank` に保存します。
`source_index` は Phase 5 `suggested_candidates` の flatten order、builtin kind 内の deterministic emission index、
または repair candidate の deterministic emission index です。
`builtin_kind_rank` は `Intro = 0`, `LocalExact = 1`, `InductionNat = 2`, `SimpLiteEmpty = 3` です。
該当しない source では `builtin_kind_rank = 255` を使います。
MVP の rank は Phase 7 metadata score、model score、HashMap iteration order に依存してはいけません。

Phase 5 suggested candidate の flatten order は次に固定します。

```text
for result in premises_result.results response order:
  for suggested in result.suggested_candidates response order:
    emit CandidateSource::Phase5Suggested with source_index = next_flat_index
```

`next_flat_index` は 0 から始まる連番です。
`premise_id` 文字列、premise name、HashMap iteration order で並べ替えてはいけません。
repair candidate は `pending_repairs` bucket で `repair_rank_key` を使うため、
`CandidateMetadata.rank.source_rank = 4`, `builtin_kind_rank = 255`,
`source_index = repair_candidate(...).pending` 内の 0-based emission index として trace に保存します。
この index は `RepairCandidateOutput.pending` を作った時点の deterministic emission order で決め、
`limit_repairs` や dedupe 後に詰め直してはいけません。

builtin generator の deterministic emission order は次に固定します。

```text
1. Intro:
   - at most one candidate for the selected goal
   - source_index = 0

2. LocalExact:
   - eligible locals are goal.context entries with value = None,
     unique machine_name in the goal context,
     and local.ty.core_hash == goal.target.core_hash
   - sort by goal.context array order
   - source_index starts at 0 within LocalExact

3. InductionNat:
   - eligible locals passing phase7_induction_nat_prefilter
   - sort by goal.context array order
   - source_index starts at 0 within InductionNat

4. SimpLiteEmpty:
   - at most one candidate
   - source_index = 0
```

If two candidates still have the same `source_rank`, `builtin_kind_rank`, and `source_index`,
`phase7_candidate_payload_hash` is the final deterministic tie-breaker.

`assign_candidate_ids` は `/machine/tactics/batch` 用の wire array と lookup map を分けて返します。
ID は rank / filter / dedup 済みの配列順に `c0`, `c1`, ... と決定的に割り当てます。
`candidate_id` は batch request 内だけの correlation id であり、trace や training の安定 identity には使いません。
`rank_index` は、この rank / filter / dedup 済み配列内の 0-based index です。
MVP では `candidate_id = "c" + decimal(rank_index)` なので、evaluated candidate の training trace では
batch request に入れた時点の `rank_index` を保存します。
deferred candidate が次の local retry batch に回った場合は、その retry batch で再度 rank / filter / dedup した後の
新しい `rank_index` を保存します。
永続ログでは、Phase 5 validation 後の `candidate_hash`、`rank_index`、Phase 7 candidate payload hash を保存します。

```rust
type CandidateBatchItem = {
    candidate_id: CandidateId,
    candidate: MachineTacticCandidate,
};

type DeferredCandidate = {
    candidate_id: CandidateId,
    candidate: CandidateEnvelope,
};

candidate_items: Vec<CandidateBatchItem>
candidate_by_id: Map<CandidateId, CandidateEnvelope>
```

`DeferredCandidate.candidate_id` は、その candidate が未評価 suffix になった直前 batch request の correlation id です。
retry batch に再投入する場合は `DeferredCandidate.candidate` だけを `merge_node_candidates` へ渡し、
`assign_candidate_ids` で新しい `candidate_id` を割り当てます。
`DeferredCandidateDropped.detail.candidate_id` には、drop 直前に保持していた `DeferredCandidate.candidate_id` を保存します。

Phase 7 candidate payload hash は、Phase 5 validation 前の dedupe / local log 用 identity です。
trusted hash、replay hash、certificate hash ではありません。

```text
phase7_candidate_payload_hash =
  sha256(
    phase7_candidate_payload_domain_separator ||
    canonical_json(MachineTacticCandidate raw wire payload)
  )

phase7_candidate_payload_domain_separator =
  UTF-8 bytes of "npa.phase7.candidate-payload.v1" followed by one 0x00 byte
```

`canonical_json` は Phase 5 の lossless request decoding 後、duplicate key がない raw candidate object だけに使います。
入力は JSON decode 後の syntax tree であり、object、array、string、integer、boolean、null だけを許します。
float、`NaN` / `Infinity` 相当、negative integer、duplicate key、または Phase 5 candidate schema が許さない JSON value は
`phase7_candidate_payload_hash` の入力にしてはいけません。
object key は JSON decode 後の UTF-8 bytes の bytewise lexicographic order で並べます。
array order は wire payload order を保持します。
出力 bytes は whitespace を含まない canonical JSON text です。
`null`、`true`、`false` はその ASCII spelling に固定します。
integer は base-10 shortest decimal に固定し、`0` 以外の leading zero、`+` sign、exponent 表記を使ってはいけません。
string は JSON string として quote し、`"`、`\`、U+0000..U+001F の control 文字だけを escape します。
control 文字は lowercase hex の `\u00xx`、その他の Unicode scalar value は JSON decode 後の UTF-8 bytes をそのまま出力します。
`/`、non-ASCII 文字、identifier 文字を任意に escape してはいけません。
Phase 7 が typed `MachineTacticCandidate` から候補を生成した場合は、まず Phase 5 `/machine/tactics/batch`
inner `candidate` と同じ raw wire object に変換します。
variant name、required field、empty array、level / local name / tactic head の JSON shape は Phase 5 7.0 schema と同一にします。
omitted optional field や display-only metadata を追加してはいけません。
Phase 5 `suggested_candidates[*].candidate` 由来の候補は、response の raw candidate object をそのまま使います。
repair 候補も同じ wire emitter を使うため、同じ raw candidate なら builtin / suggested / repair の由来に関係なく
同じ `phase7_candidate_payload_hash` になります。
Phase 7 はこの hash で `dedupe_by_canonical_candidate_payload` と pending repair の重複排除を行います。
Phase 5 が `candidate_hash` を返した後の replay / verify / training identity では、常に Phase 5 `candidate_hash` を優先します。

`CandidateEnvelope.candidate_hash = Some(expected_hash)` の候補に対して `/machine/tactics/batch` が
`item.candidate_hash = Some(actual_hash)` を返した場合、`expected_hash == actual_hash` でなければなりません。
また、`CandidateEnvelope.candidate_hash = Some(expected_hash)` なのに
`item.candidate_hash = None` が返った場合も同じ consistency error として扱います。
不一致または欠落は candidate failure ではなく Phase 5 integration / controller consistency error です。
その result は replay step、repair、positive / negative training、`tactic_candidates` には使わず、
探索全体を final `MachineControllerError` で終了します。
同じ batch response 内に他の success / accepted error item が含まれていても、その batch response 全体を破棄し、
training trace、child node、repair、negative example は 1 件も作りません。
このとき `endpoint = "/machine/tactics/batch"`、
`error_kind = "suggested_candidate_hash_mismatch"`、`error_phase = None`、
`diagnostic_hash = None` に固定します。
`suggested_candidate_hash_mismatch` は Phase 7 controller-local final reason string であり、Phase 5 error kind として送受信してはいけません。
擬似コード中の `candidate_hash_mismatch(envelope, item)` は、
`envelope.candidate_hash = Some(expected_hash)` かつ
（`item.candidate_hash = None`、または
`item.candidate_hash = Some(actual_hash)` かつ `expected_hash != actual_hash`）の場合だけ true です。
`batch_has_candidate_hash_mismatch(candidate_by_id, result)` は `result.results` の各 item に
`candidate_hash_mismatch(candidate_by_id[item.candidate_id], item)` を適用し、1 件でも true なら true です。
Phase 7 は `record_training_trace_batch`、replay step 作成、repair 作成より前にこの batch-level check を行います。
`suggested_candidate_hash_mismatch_reason()` はこの段落の 4 field で
`SearchFailureReason::MachineControllerError` を作ります。
`suggested_candidate_hash_mismatch_error()` は trace の `ControllerError.detail` を埋めるための
controller-local pseudo result であり、Phase 5 response として扱ってはいけません。

batch success から `MachineReplayStep` を作る mapping は次で固定します。

```text
MachineReplayStep.previous_state_fingerprint = node.state_fingerprint
MachineReplayStep.goal_id = selected goal.goal_id
MachineReplayStep.candidate = candidate_by_id[item.candidate_id].candidate
MachineReplayStep.deterministic_budget = request deterministic_budget payload
MachineReplayStep.candidate_hash = item.candidate_hash
MachineReplayStep.deterministic_budget_hash = batch response top-level deterministic_budget_hash
MachineReplayStep.proof_delta_hash = item.proof_delta_hash
MachineReplayStep.next_state_fingerprint = item.next_state_fingerprint
```

MVP の Phase 7 proof search は `/machine/tactics/batch` の per-candidate success result から replay step を作ります。
run-based controller を別 profile として追加する場合も、同じ replay step field だけを埋め、
Phase 7 metadata、score、display text、`candidate_id` を replay step に入れてはいけません。

batch の per-candidate error response は repair の前に次の共通形へ正規化します。
run-based controller profile を追加する場合は、run の nested `error` object も同じ形へ正規化します。

```rust
struct AcceptedCandidateFailure {
    error_kind: FailedCandidateErrorKind,
    phase: String, // Phase 5 diagnostic phase wire name
    goal_id: Option<GoalId>,
    tactic_kind: Option<String>, // Phase 5 MachineTacticKind wire name
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    diagnostic_hash: Hash,
    retryable: bool,
}
```

MVP の `/machine/tactics/batch` では flattened `item.error_kind` を使い、
`deterministic_budget_hash` は batch response の top-level field から補います。
run-based controller profile では nested `error.kind` を `error_kind` に写し、
run success / error response の `deterministic_budget_hash` を使います。
`CandidateEnvelope.candidate_hash = Some(expected_hash)` の error item でも、`item.candidate_hash` が存在する場合は
`expected_hash == item.candidate_hash` を先に検査します。
不一致なら上の `suggested_candidate_hash_mismatch` として扱い、`AcceptedCandidateFailure` にしてはいけません。
`diagnostic_hash` がない error item は Phase 5 response contract 違反なので、ここには到達しません。
`candidate_hash` がない result、または Phase 5 `FailedCandidateErrorKind` に含まれない kind は
`AcceptedCandidateFailure` にせず、repair と negative training には使いません。

pending repair は priority queue に戻さず、同じ `SearchNode` 展開中の local retry batch として実行します。
repair candidate が探索木の child node になるのは、Phase 5 tactic execution が success を返した後だけです。
pending repair は `repair_depth <= 2`、同一 goal 内の `phase7_candidate_payload_hash` で重複排除、
同じ parent candidate から最大 3 個までに制限します。
`visited` は `state_fingerprint` だけなので、同じ `node.snapshot_id` / `node.state_fingerprint` を
pending repair 付きで queue に戻す実装は禁止します。
`repair_depth` は node-wide counter ではなく、repair chain ごとの値です。
fresh / suggested / builtin 候補の `repair_depth_of(envelope)` は 0、
repair 候補は `CandidateEnvelope.metadata.repair.repair_depth` を使います。
失敗した候補から次の repair を作る場合は `repair_depth_of(failed_envelope) + 1` に固定し、
同じ node 内の別 repair chain や deferred candidate の存在で depth を増減してはいけません。
`repair_chain_tried_payload_hashes(failed_envelope)` は、fresh / suggested / builtin 候補では
`[failed_envelope.phase7_candidate_payload_hash]`、
repair 候補では `failed_envelope.metadata.repair.chain_tried_payload_hashes` の末尾に
`failed_envelope.phase7_candidate_payload_hash` を追加した列です。
次に生成する repair candidate の `phase7_candidate_payload_hash` がこの列に含まれる場合、その candidate は
`pending` に入れてはいけません。
この場合は `RepairCandidateOutput.repeated_candidate_payload_hashes` にその hash を入れ、呼び出し側が
`RepairChainStopReason::RepeatedCandidate` で `RepairChainStopped` を記録します。
この重複判定は repair chain 内だけに適用し、同じ node 内の別 fresh candidate や別 parent candidate には伝播しません。
`repeated_repair_error(envelope, failure)` は、
`envelope.metadata.repair = Some(repair)` かつ `repair.error_kind == failure.error_kind` の場合だけ true です。
この打ち切りは repair chain 単位であり、同じ node 内の別 candidate や fresh candidate には伝播しません。
`repair_candidate(snapshot, goal, envelope, failure, repair_depth)` は `RepairCandidateOutput` を返します。
`pending` の返却順は repair rule table の記載順、同一 rule 内では generated candidate の deterministic emission order です。
各 pending item の `parent_candidate_hash = failure.candidate_hash`、
`error_kind = failure.error_kind`、`repair_depth = repair_depth`、
`chain_tried_payload_hashes = repair_chain_tried_payload_hashes(envelope)` に固定します。
`repeated_candidate_payload_hashes` は同じ repair rule table / emission order で、chain 内重複として捨てた
repair candidate hash を first occurrence order で重複なしに保存します。
`repeated_candidate_payload_hashes` に hash が入っても、同じ failed envelope から生成された別の non-repeated
pending repair を捨ててはいけません。
`limit_repairs` は `(goal_id, parent_candidate_hash, error_kind)` ごとに最大 3 件までにし、
同じ `phase7_candidate_payload_hash` を持つ repair は最初の 1 件だけ残します。

---

## 5.6 Goal selection

複数 goal がある場合、どれを先に解くかが重要です。
MVP の `select_goal(snapshot)` は、Phase 5 snapshot 内の open goal order と derived view だけを使って
deterministic に 1 つ選びます。

```text
goal_selection_key(goal):
  - goal.expr_size
  - goal.context_size
  - goal.target_free_local_count
  - goal.open_goal_index
  - goal.goal_id canonical bytes
```

MVP は `goal_selection_key` を lexicographic ascending で比較し、最小の goal を選びます。
`exact_candidate_count`、`simp_likelihood`、retrieval score、model score は goal selection 前には使いません。
それらを使う場合は non-MVP goal-selection profile として、query、budget、tie-break を別途固定します。

以下の方針とスコア式は non-MVP goal-selection profile の設計メモです。
MVP 実装は使いません。

方針：

```text
- 小さい goal を優先
- exact/rw/simp で閉じられそうな goal を優先
- dependency が少ない goal を優先
- metavariable を多く制約する goal を優先
```

スコア：

```text
goal_priority =
  + exact_candidate_count
  + simp_likelihood
  + small_expr_bonus
  - context_size_penalty
  - dependency_complexity
```

---

## 5.7 Budget

探索には必ず予算を設けます。
Phase 7 の探索予算、Phase 5 の deterministic tactic budget、外側 scheduler limit は別物です。
混ぜると replay / cache / timeout の意味が崩れます。

MVP の controller config 型は次に固定します。

```rust
struct Phase7MvpControllerConfig {
    search_budget: SearchBudget,
    per_tactic_deterministic_budget: MachineDeterministicBudget,
    scheduler_limits: Option<MachineSchedulerLimits>,
    batch_policy: MachineBatchPolicy,
}

struct SearchBudget {
    wall_clock_ms: u64,
    max_nodes: u64,
    max_tactics_per_node: u32,
    max_depth: u32,
}
```

`search_budget`、`per_tactic_deterministic_budget`、`batch_policy` は必須です。
`scheduler_limits` は omitted / `None` を許し、その場合は Phase 5 request でも omitted にします。
`scheduler_limits = null`、unknown field、float、負数は invalid config です。
`SearchBudget.wall_clock_ms` と `max_nodes` は `>= 1`、`max_depth` は `>= 0` です。
MVP の `max_tactics_per_node` は `16` に固定し、他の値を受ける profile は non-MVP です。
`per_tactic_deterministic_budget`、`scheduler_limits`、`batch_policy` の field set と integer validation は
Phase 5 Machine API の同名 schema に従います。
Phase 7 は invalid config で proof search を開始してはいけません。

```json
{
  "search_budget": {
    "wall_clock_ms": 30000,
    "max_nodes": 10000,
    "max_tactics_per_node": 16,
    "max_depth": 64
  },
  "per_tactic_deterministic_budget": {
    "max_tactic_steps": 64,
    "max_whnf_steps": 10000,
    "max_conversion_steps": 10000,
    "max_rewrite_steps": 100,
    "max_meta_allocations": 8,
    "max_expr_nodes": 20000
  },
  "scheduler_limits": {
    "per_candidate_timeout_ms": 100,
    "batch_timeout_ms": 1000,
    "max_memory_mb": 1024
  },
  "batch_policy": {
    "max_evaluated_candidates": 16,
    "stop_after_successes": 8,
    "stop_after_failures": 16
  }
}
```

`per_tactic_deterministic_budget` は replay step に保存し、`deterministic_budget_hash` と照合します。
`scheduler_limits` による timeout / memory stop は deterministic tactic error ではないため、
training negative や replay plan には入れません。
探索予算超過時は、best partial が存在する場合だけ replay prefix と対応 snapshot を返します。
存在しない場合は `Phase7SearchFailure.best_partial_* = None` を返します。

候補数の cap は Phase 7 と Phase 5 で役割を分けます。
MVP では `search_budget.max_tactics_per_node = 16` を node ごとの総実行候補数にし、
初回 batch と repair retry batch の合計に適用します。
Phase 7 は rank / filter と dedupe の後、`take_remaining_node_tactic_budget` で
batch request を作る前に残り候補数まで切り詰めます。
`cap_batch_policy` は `max_evaluated_candidates`、`stop_after_successes`、`stop_after_failures` を
送信する `candidate_items.length` 以下に丸めます。
Phase 5 protocol cap の 256 は wire safety 上限であり、Phase 7 MVP の探索幅として使ってはいけません。

---

## 5.8 並列化

Phase 7 MVP controller 自体は、priority queue の pop、node expansion、trace append、stats update、
training trace append、child `NodeId` allocation をすべて単一の deterministic event order で行います。
MVP 実装は複数 node を並列に展開してはいけません。
並列探索、work stealing、非同期に返った candidate result で探索順を変える scheduler は non-MVP search profile です。
追加する場合は event order、trace order、stats update、`NodeId` allocation、best partial tie-break を別途固定します。

将来の non-MVP profile で並列化できる箇所：

```text
- premise retrieval
- tactic candidate generation
- tactic execution
- child node scoring
- independent goal solving
```

ただし、state mutation は transaction にします。

```text
parent state
  ├── tactic 1 trial
  ├── tactic 2 trial
  ├── tactic 3 trial
  └── tactic 4 trial
```

各 tactic は state copy または persistent state 上で試します。
MVP で許される並列性は Phase 5 `/machine/tactics/batch` server 内部の実装最適化だけです。
その場合でも Phase 5 の batch prefix contract、response order、scheduler stop contract が唯一の観測結果であり、
Phase 7 は response `results` order 以外の完了順を観測してはいけません。

---

## 5.9 探索モード

Phase 7 MVP では best-first search を主軸にします。

```text
MVP:
  best-first search

後で追加:
  beam search
  MCTS
  proof replay search
  lemma-splitting search
  RL-guided search
```

MVP の固定設定：

```text
- max tactics per node: 16
- deterministic tactics first
- model tactics disabled
- beam width parameter なし
```

MVP は beam search を実装しません。
priority queue の探索幅は `search_budget.max_nodes` と `search_budget.max_tactics_per_node` だけで制限します。
beam width、model tactics、risky tactic tier を導入する場合は non-MVP search profile として別途固定します。

---

# 6. Error repair

## 6.1 目的

AIやテンプレートが出した tactic は頻繁に失敗します。
error repair は、失敗情報を使って修正候補を生成します。

MVP 例：

```text
tactic:
  exact h

error:
  type_mismatch

repair:
  simp-lite
```

---

## 6.2 Error schema

Phase 5 の tactic execution API から構造化エラーを受け取ります。
Phase 7 MVP は Phase 5 の error kind をそのまま使い、独自の別名を作りません。
repair に使うのは、`candidate_hash` と `diagnostic_hash` を伴う accepted candidate failure だけです。
request validation failure、goal lookup failure、scheduler stop は repair 用の tactic failure として扱いません。
Phase 7 MVP は `/machine/tactics/batch` の per-candidate item にある flattened diagnostic fields を
5.5 の `AcceptedCandidateFailure` に正規化してから repair に渡します。
run-based controller profile を追加する場合は、`/machine/tactics/run` の nested `error` object も
同じ `AcceptedCandidateFailure` に正規化します。

```json
{
  "status": "error",
  "error": {
    "kind": "type_mismatch",
    "phase": "machine_term_check",
    "goal_id": "g1",
    "tactic_kind": "exact",
    "candidate_hash": "sha256:...",
    "deterministic_budget_hash": "sha256:...",
    "diagnostic_hash": "sha256:...",
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:...",
    "retryable": false
  },
  "unchanged_state_fingerprint": "sha256:..."
}
```

---

## 6.3 repair rule table

Phase 7 MVP の repair は、raw `MachineTacticCandidate` を決定的に作れるものだけに限定します。
MVP の repair table は Phase 5 `FailedCandidateErrorKind` を全件分類します。
下に個別規則がない error kind では repair candidate を生成してはいけません。

```text
MVP repair:
  - failed candidate を trace に記録し、同じ repair chain では同じ候補を再試行しない
  - Intro の `expected_pi_type` failure では同じ repair chain 内で Intro を再投入しない
  - `intro.name` collision 由来の post-canonical `InvalidCandidate` は accepted repair failure にせず trace に記録する
  - goal.allowed_tactics に "simp-lite" が含まれる場合だけ SimpLite { rules = [] } を追加する
  - local Exact は RawMachineTerm.source が一意の MachineLocalName だけの場合に限る

non-MVP repair:
  - failed candidate の score を下げ、別の既存 candidate を優先する
  - Phase 5 suggested_candidates 由来の candidate を再順位付けする
  - exact を apply に変える
  - Eq.symm などの補題適用を text source から合成する
  - local equality rw / backward rw を合成する
  - theorem search の結果から一般 Exact / Apply を合成する
```

MVP repair は `Phase7Score`、`CandidateMetadata.rank`、`fresh_rank_key` を変更しません。
この節で追加できるのは、上の rule から raw `MachineTacticCandidate` を決定的に作れる pending repair だけです。
既存 candidate の score 変更や rerank を行う場合は non-MVP repair profile として rank key を別途固定します。
この節の「trace に記録する」は、accepted candidate failure については 9.1 の
`training_trace_records[*].tactic_candidates[*]` に error item として保存することを意味します。
`candidate_hash` がない、または `FailedCandidateErrorKind` 外の error は accepted candidate failure ではないため、
`Phase7TraceEventKind::NonAcceptedCandidateError` に記録します。
repair depth / repeated error / 重複候補のため repair chain を止める場合は
`Phase7TraceEventKind::RepairChainStopped` に記録します。
accepted candidate failure ごとに追加の汎用 `Phase7TraceEvent` を作ってはいけません。

MVP で repair rule が `simp-lite` を追加すると書く場合、返す pending repair candidate は次の 1 種だけです。

```text
candidate = MachineTacticCandidate::SimpLite { rules = [] }
metadata.source = Repair
metadata.premises_used = []
metadata.expected_effect = Simplify
metadata.repair.parent_candidate_hash = failure.candidate_hash
metadata.repair.error_kind = failure.error_kind
metadata.repair.repair_depth = repair_depth
metadata.repair.chain_tried_payload_hashes = repair_chain_tried_payload_hashes(failed_envelope)
```

`goal.allowed_tactics` に `"simp-lite"` がない場合、その rule は `pending = []` を返します。
同じ `phase7_candidate_payload_hash` が同じ repair chain で既に試されている場合、その rule は
`pending = []` かつ `repeated_candidate_payload_hashes = [その hash]` を返します。
「failed tactic を削除する」「intro を削除する」「別 candidate を優先する」は、MVP では pending repair candidate を作らず、
trace event だけを記録します。

### no-op repair kinds

次の Phase 5 `FailedCandidateErrorKind` は、MVP では trace に記録するだけで追加 repair candidate を作りません。

```text
unsupported_tactic:
  repair:
    - trace に記録し、同じ repair chain では再試行しない
    - 追加の MVP repair candidate は作らない

machine_term_elaboration_error:
  repair:
    - trace に記録し、同じ repair chain では再試行しない
    - fully explicit RawMachineTerm.source を一般合成しない
    - 追加の MVP repair candidate は作らない

induction_target_not_nat:
  repair:
    - trace に記録し、同じ repair chain では InductionNat を再投入しない
    - 追加の MVP repair candidate は作らない

too_large_term:
  repair:
    - trace に記録し、同じ repair chain では再試行しない
    - deterministic budget や scheduler_limits を増やさない
    - 追加の MVP repair candidate は作らない
```

これらを修復する場合は、対応する candidate generation profile、budget profile、または ModelRepair profile を
non-MVP として別途固定します。

### `unknown_name`

```text
原因:
  定理名や仮定名が間違っている

repair:
  - trace に記録し、同じ repair chain では再試行しない
  - 追加の MVP repair candidate は作らない

non-MVP repair:
  - この candidate の score を下げる
  - 既存の Phase 5 suggested_candidates を再順位付けする
  - theorem search で近い名前を探す
  - namespace を補う
  - typo correction
```

non-MVP 例：

```text
Nat.add_zro
  -> Nat.add_zero
```

### `type_mismatch`

```text
原因:
  exact/apply の型が target と合わない

repair:
  - goal.allowed_tactics に "simp-lite" が含まれる場合だけ simp-lite を先に試す
  - local Exact の候補なら trace に記録し、同じ repair chain では再試行しない

non-MVP repair:
  - local Exact の候補なら score を下げる
  - Phase 5 suggested_candidates に同じ goal 用の候補があれば再順位付けする
  - exact を apply に変える
  - Eq.symm を使う
  - rw を先に試す
```

non-MVP 例：

```text
target:
  b = a

h:
  a = b

failed:
  exact h

repair:
  exact Eq.symm h
```

### `expected_pi_type`

```text
原因:
  intro を関数型でない goal に使った

repair:
  - failed Intro を trace に記録し、同じ repair chain では Intro を再投入しない
  - 「intro を削除する」pending candidate は作らない
  - goal.allowed_tactics に "simp-lite" が含まれる場合だけ simp-lite を試す

non-MVP repair:
  - 既存の Phase 5 suggested_candidates を再順位付けする
  - exact / apply を新規合成する
```

### `rewrite_rule_invalid` / `simp_no_progress`

```text
原因:
  rw rule が invalid、または simp-lite が進まない

repair:
  - goal.allowed_tactics に "simp-lite" が含まれる場合だけ simp-lite を試す

non-MVP repair:
  - Phase 5 suggested_candidates に別の rw candidate があれば再順位付けする
  - 逆向き rewrite を試す
  - theorem search で別の rewrite rule を探す
```

non-MVP 例：

```text
failed:
  rw [Nat.zero_add]

repair:
  rw [Nat.add_zero]
```

### `implicit_argument_required`

```text
原因:
  implicit argument が推論できない

repair:
  - trace に記録し、同じ repair chain では再試行しない
  - MVP では fully explicit RawMachineTerm.source を補う repair candidate は作らない

non-MVP repair:
  - score を下げ、別 candidate を優先する
  - `@` で明示引数を text source から合成する
  - expected type から一般に型を補う
```

non-MVP 例：

```text
Eq.refl n
  -> @Eq.refl Nat n
```

### `too_many_goals`

```text
原因:
  apply が大量の subgoal を作った

repair:
  - trace に記録し、同じ repair chain では再試行しない
  - simp-lite を決定的に生成できる場合だけ pending repair に追加する

non-MVP repair:
  - この tactic のスコアを下げる
  - 既存の Phase 5 suggested_candidates を再順位付けする
  - より具体的な定理を使う
  - exact 候補を新規合成する
```

### `budget_exceeded` / scheduler stop

```text
原因:
  deterministic budget を使い切った、または scheduler_limits で停止した

repair:
  - deterministic budget exceeded なら trace に記録し、同じ repair chain では再試行しない
  - 追加の MVP repair candidate は作らない

non-MVP repair:
  - deterministic budget exceeded なら candidate のスコアを下げる
  - simp rule を制限する
  - 別 tactic を優先する
  - deterministic budget を増やす profile を追加する
```

`scheduler_stopped`、`partial_timeout`、`partial_resource_limit` は retryable な外側停止であり、
deterministic failure ではありません。
同じ candidate の negative training example にはしません。

---

## 6.4 RepairGenerator

```rust
struct RepairGenerator {
    rule_based: RuleBasedRepair,
    model_based: Option<ModelRepair>,
    theorem_search: Option<TheoremSearchClient>, // non-MVP repair only
}
```

`repair_candidate(...)` の出力例（MVP）：

```text
RepairCandidateOutput {
  pending = [
    PendingCandidate {
      candidate = CandidateEnvelope {
        candidate = MachineTacticCandidate::SimpLite { rules = [] }
        metadata.source = Repair
        metadata.display_text = Some("simp-lite")
        metadata.repair.parent_candidate_hash = failure.candidate_hash
        metadata.repair.error_kind = "type_mismatch"
        metadata.repair.repair_depth = 1
        metadata.repair.chain_tried_payload_hashes =
          repair_chain_tried_payload_hashes(failed_envelope)
      }
      repair_depth = 1
      parent_candidate_hash = failure.candidate_hash
      error_kind = "type_mismatch"
      chain_tried_payload_hashes = repair_chain_tried_payload_hashes(failed_envelope)
    }
  ]
  repeated_candidate_payload_hashes = []
}
```

上の block は `RepairCandidateOutput` の説明用です。
実装の正本は 5.2 の `RepairCandidateOutput` / `PendingCandidate` と `CandidateEnvelope.metadata.repair` であり、
`reason` のような追加 field を返してはいけません。

`Exact { term.source = "Eq.symm h" }` や local equality `Rw` の repair は、
Eq head、universe arguments、term arguments、rewrite site を Phase 5 `MachineTacticCandidate`
として完全に決定できる profile を追加するまで non-MVP です。

---

## 6.5 Repair の制限

repair は無限に繰り返すと危険です。

制限：

```text
- 1 tactic あたり repair 最大 3 個
- 同じ `phase7_candidate_payload_hash` は再試行しない
- 同じ Phase 5 error kind が続いたら打ち切る
- repair depth 最大 2
- budget_exceeded / scheduler stop は同じ candidate で即時再試行しない
```

---

# 7. Proof minimization

## 7.1 目的

探索で見つかった証明は長く、冗長になりがちです。

例：

```npa
by
  rw [Nat.add_zero]
  exact Eq.refl n
```

これは：

```npa
by
  simp-lite
```

で済むかもしれません。

proof minimization は、証明を短く、読みやすく、再検査しやすくします。
MVP の最小化対象は text proof script ではなく `MachineReplayPlan.steps` です。
text proof は UI 表示として後から生成できますが、検証対象にはしません。

---

## 7.2 最小化の原則

重要：

```text
最小化後も必ず kernel / certificate checker で再検査する。
```

AIが「短くした」と言っても信用しません。

---

## 7.3 Minimization pass

MVP の minimization pass kind は次の 3 種だけです。
この順で処理し、各 proposal は `/machine/replay` と `/machine/verify` の両方を通った場合だけ採用します。

1. `delete_redundant_steps`
2. `replace_blocks_with_simp_lite_empty`
3. `minimize_existing_simp_lite_rules`

`replace_with_exact_theorem`、任意の theorem search を使う exact / apply 置換、namespace shortening、import minimization は
non-MVP です。
MVP 実装はこれらを pass list に入れてはいけません。

各 pass は deterministic proposal sequence を作ります。
1 つの proposal が replay / verify に通ったら採用し、同じ pass の proposal sequence を更新後の plan から最初から作り直します。
その pass で最後まで 1 つも採用されなかったら次の pass に進みます。
現在の step sequence と同一になる proposal は生成しません。

MVP の proposal order は次に固定します。

```text
delete_redundant_steps:
  - current step index を 0 から昇順に見る
  - proposal はその 1 step だけを削除する

replace_blocks_with_simp_lite_empty:
  - block_len を current_plan.steps.length から 1 まで降順に見る
  - 同じ block_len 内では start_index を 0 から昇順に見る
  - proposal はその contiguous block を 1 つの SimpLite { rules = [] } に置き換える

minimize_existing_simp_lite_rules:
  - current step index を 0 から昇順に見る
  - SimpLite step の rules index を 0 から昇順に見る
  - proposal はその 1 rule だけを削除する
```

### Pass 1: tactic deletion

各 replay step に対応する raw candidate を消しても証明が通るか試します。

```text
original:
  intro n
  rw [Nat.add_zero]
  exact Eq.refl n

try remove rw:
  intro n
  exact Eq.refl n
```

通るなら削除します。

これは delta debugging です。
ただし既存 step の `candidate_hash` / `proof_delta_hash` / `next_state_fingerprint` は再利用できません。
step を削除した `ReplayStepEdit` sequence を initial snapshot から再実行し、
fresh な `MachineReplayStep` を持つ新しい `MachineReplayPlan` を作ります。

### Pass 2: block replacement

連続した tactic block を短い tactic に置き換えます。
MVP では、連続 block を `MachineTacticCandidate::SimpLite { rules: [] }` に置き換える
`replace_blocks_with_simp_lite_empty` だけを許可します。
定理参照付き `simp-lite`、`exact`、`apply`、または tactic search による replacement は non-MVP です。
置換で作る `ReplayStepEdit` は削除対象 block の先頭 step edit から次を引き継ぎます。

```text
replacement ReplayStepEdit:
  original_goal_id = block[0].original_goal_id
  original_open_goal_index = block[0].original_open_goal_index
  candidate = MachineTacticCandidate::SimpLite { rules = [] }
  deterministic_budget = block[0].deterministic_budget
```

block は non-empty contiguous range だけです。
block 内の後続 step の `goal_id` や budget は置換 candidate には引き継ぎません。
別 budget で置換を試す profile は non-MVP とします。

```text
rw [...]
exact Eq.refl ...
```

を：

```text
simp-lite
```

に置換できるか試します。

### Non-MVP pass: exact replacement

これは non-MVP pass です。
MVP では実行しません。

長い証明項や tactic 列を、既存定理に置き換えます。

```text
by
  induction-nat n
  ...
```

を：

```text
exact Nat.zero_add n
```

に置き換えられるなら置き換えます。

### Pass 3: existing simp-lite rule deletion

不要な theorem reference を削ります。
MVP では既存 replay plan 内にある `MachineTacticCandidate::SimpLite` の `rules` から要素を削る
`minimize_existing_simp_lite_rules` だけを実行します。
rule の順序は維持し、削除候補は左から右へ決定的に試します。
`rules = []` の `SimpLite` は no-op です。
MVP の premise simplification は新しい theorem reference を追加してはいけません。

```text
simp-lite [Nat.add_zero, Nat.zero_add, Nat.add_assoc]
```

を：

```text
simp-lite [Nat.add_zero]
```

にする。

### Pass 5: namespace shortening

完全修飾名を短くできます。

```text
exact Std.Nat.Basic.Nat.add_zero n
```

を、namespace内なら：

```text
exact Nat.add_zero n
```

にする。

これは display proof 用の non-MVP pass です。
raw `MachineTacticCandidate` の `TacticHead.imported.name` は Phase 5 の renderable fully-qualified name のままにします。

### Pass 6: import minimization

不要な import を削ります。

```text
import Std.Nat
import Std.List
```

`Std.List` が不要なら削除。

これは non-MVP pass です。
import を変えるには新しい `/machine/sessions` を作り、同じ `ReplayStepEdit` sequence を再実行してから
`/machine/replay` と `/machine/verify` を通す必要があります。

---

## 7.4 Proof minimization algorithm

```python
def minimize_replay_plan(verified_replay_plan, verified_replay, verified_response, session, initial_snapshot):
    current_plan = verified_replay_plan
    current_replay = verified_replay
    current_verify = verified_response
    minimization_stats = MinimizationStats.zero()

    for minimization_pass in [
        delete_redundant_steps,
        replace_blocks_with_simp_lite_empty,
        minimize_existing_simp_lite_rules,
    ]:
        minimization_stats.pass_kinds_attempted += 1
        changed = True

        while changed:
            changed = False
            step_edits = make_step_edits_with_goal_indices(current_plan, session, initial_snapshot)
            if step_edits is None:
                break

            for proposed_steps in minimization_pass.proposals(step_edits, session):
                rebuilt = rebuild_replay_plan_from_step_edits(current_plan, proposed_steps, session, initial_snapshot)
                if rebuilt is None:
                    continue
                minimization_stats.rebuilt_plans += 1

                minimization_stats.replay_attempts += 1
                replayed = machine_replay(session.session_id, rebuilt)
                if replayed.status != "ok":
                    continue

                minimization_stats.verify_attempts += 1
                verified = machine_verify(
                    session.session_id,
                    replayed.final_snapshot_id,
                    replayed.final_state_fingerprint,
                    mode="certificate",
                )
                if verified.status != "verified":
                    continue

                current_plan = rebuilt
                current_replay = replayed
                current_verify = verified
                minimization_stats.accepted_proposals += 1
                changed = True
                break

    return current_plan, current_replay, current_verify, minimization_stats
```

minimizer は candidate 列だけを扱ってはいけません。
各要素は次の情報を保持する `ReplayStepEdit` として扱います。

```rust
struct ReplayStepEdit {
    original_goal_id: GoalId,
    original_open_goal_index: u32,
    candidate: MachineTacticCandidate,
    deterministic_budget: MachineDeterministicBudget,
}
```

`original_open_goal_index` は `MachineReplayStep` には保存されていません。
`make_step_edits_with_goal_indices(current_plan, session, initial_snapshot)` は
`current_plan` の initial state から step を順に再実行し、
各 step を実行する直前の `MachineProofSnapshot.open_goals` 内で `step.goal_id` が出現する 0-based index を記録して
`original_open_goal_index` に入れます。
initial snapshot lookup では、Phase 7 controller が proof search 開始時に受け取った
`initial_snapshot.snapshot_id` を使い、`/machine/snapshots/get` に `session.session_id`、
`initial_snapshot.snapshot_id`、`current_plan.initial_state_fingerprint`、`include_pretty = false` を渡します。
`MachineReplayPlan` には `initial_snapshot_id` がないため、`state_fingerprint` から snapshot id を逆引きする
`snapshot_id_from_state_fingerprint` のような helper を作ってはいけません。
`initial_snapshot.state_fingerprint != current_plan.initial_state_fingerprint` の場合は、
その minimization pass kind を中断して直近の verified plan を維持します。
この再実行は MVP では `/machine/tactics/batch` に 1 candidate だけを入れて行います。
minimization 再実行用の batch request は次に固定します。

```text
minimization_reexecute_batch:
  candidates = [{ candidate_id = "c0", candidate = step.candidate }]
  deterministic_budget = step.deterministic_budget
  scheduler_limits = omitted
  batch_policy = {
    max_evaluated_candidates = 1,
    stop_after_successes = 1,
    stop_after_failures = 1,
  }
```

`batch_policy` と `scheduler_limits` は `ReplayStepEdit` や `MachineReplayStep` に保存しません。
minimization 再実行では常に上の固定 request wrapper を使います。
`/machine/tactics/batch` が top-level error、partial response、`results.length != 1`、または `candidate_id != "c0"` を返した場合は
その pass kind を中断して次へ進みます。
success item から得られた `candidate_hash`、
`deterministic_budget_hash`、`proof_delta_hash`、`next_state_fingerprint` が元の step と一致することを確認します。
どれかが一致しない場合、または `step.goal_id` が直前 snapshot の `open_goals` に存在しない場合は
`make_step_edits_with_goal_indices` が `None` を返し、その minimization pass kind を中断して次へ進みます。
`original_open_goal_index` を `MachineReplayStep` に追加してはいけません。

`rebuild_replay_plan_from_step_edits(current_plan, proposed_steps, session, initial_snapshot)` は
initial snapshot から順に実行し直し、
現在 snapshot に `original_goal_id` が残っていればそれを使います。
残っていない場合は `original_open_goal_index` で同じ位置の open goal を選びます。
どちらでも一意に選べない場合、その minimization pass は失敗として元の plan を維持します。
再実行に使う goal を `execution_goal_id` と呼びます。
`execution_goal_id` は上の規則で選んだ現在 snapshot 内の goal id です。
`original_goal_id` が残っていない場合、再実行 request の `goal_id` と fresh に作り直す
`MachineReplayStep.goal_id` には、古い `original_goal_id` ではなく `execution_goal_id` を入れます。
`ReplayStepEdit.original_goal_id` は proposal の provenance であり、現在 snapshot に存在しない goal id を
fresh `MachineReplayStep` にコピーしてはいけません。
各 `ReplayStepEdit` の再実行も同じ `minimization_reexecute_batch` wrapper を使います。
成功した各 step から fresh な `MachineReplayStep` を作り直し、最後に `/machine/replay` と
`/machine/verify` を通したものだけを採用します。
minimization は既に `/machine/verify` 済みの plan から始めます。
どの pass が失敗しても panic / assert せず、直近の verified plan、対応する replay response、
verify response、`minimization_stats` を返します。
`current_replay` は直近の verified plan に対応する `/machine/replay` success response です。
Phase 7 success result の `final_snapshot_id` と `final_state_fingerprint` はこの `current_replay` から返します。

各 pass はこうします。

```text
ReplayStepEdit sequence を作る
  ↓
initial snapshot から /machine/tactics/batch で再実行する
  ↓
新しい MachineReplayPlan を作る
  ↓
/machine/replay
  ↓
/machine/verify
  ↓
成功なら採用
```

---

## 7.5 Minimization score

この節は non-MVP の minimization / display proof profile の設計メモです。
MVP minimizer は `minimization_score` を計算せず、`style` 設定も受け取りません。
MVP の採用規則は 7.3 の deterministic proposal order に従い、最初に `/machine/replay` と
`/machine/verify` の両方に通った proposal を採用するだけです。
score や style で proposal を並べ替える場合は、別の non-MVP minimization profile として
score の field、丸め、tie-break、proposal ordering を固定してから実装します。

短さだけを最適化しすぎると、読みにくくなります。
したがってスコアを分けます。

```text
minimization_score =
  - tactic_count
  - character_count
  - proof_term_size
  - import_count
  + readability_bonus
  + stable_tactic_bonus
  - fragile_tactic_penalty
```

たとえば：

```text
exact Nat.add_zero n
```

は読みやすい。

```text
simp-lite
```

は短いが、simp database に依存します。

non-MVP ではユーザー設定で選べるようにします。

```json
{
  "style": "short"
}
```

または：

```json
{
  "style": "readable"
}
```

---

## 7.6 Minimization API

MVP では独立した `/ai/minimize_proof` endpoint は定義しません。
Phase 7 controller 内の関数として次の形を実装します。

```rust
struct Phase7MinimizationResult {
    replay_plan: MachineReplayPlan,
    replay_response: MachineReplaySuccess,
    verify_response: MachineVerifySuccess,
    minimization_stats: MinimizationStats,
}
```

`minimize_replay_plan` は `Phase7MinimizationResult` と同じ 4 要素を返します。
plan だけを返す関数にしてはいけません。成功結果の `final_snapshot_id` と `final_state_fingerprint` は
minimized plan に対応する `replay_response` から取る必要があります。

後で endpoint 化する場合は、input を `proof_script` ではなく complete `MachineReplayPlan`、
current session handle、initial snapshot handle にします。
initial snapshot handle は少なくとも `snapshot_id` と `state_fingerprint` を含み、実装が必要なら
complete `MachineProofSnapshot` を渡します。
endpoint は `plan.session_root_hash` と current session の root、`initial_snapshot.state_fingerprint` と
`plan.initial_state_fingerprint` の一致を検査してから minimization を開始します。
endpoint 版でも `state_fingerprint` から snapshot id を逆引きしてはいけません。
output は minimized `MachineReplayPlan`、`/machine/replay` response、`/machine/verify` response にします。
text script は optional display artifact です。

---

# 8. Phase 7 controller boundary

Phase 7 MVP は独自の `/ai/*` endpoint を定義しません。
MVP は Phase 5 Machine API client / controller として実装します。

```text
Phase7Controller:
  input:
    - existing MachineProofSession handles
    - initial MachineProofSnapshot
    - search_budget
    - per_tactic_deterministic_budget
    - scheduler_limits
    - batch_policy

  calls:
    - /machine/snapshots/get
    - /machine/search/for_goal
    - /machine/prompt_payload        # ModelGenerator を有効化した後だけ
    - /machine/tactics/batch
    - /machine/replay
    - /machine/verify

  output:
    - Phase7VerifiedProof
    - Phase7SearchFailure
```

将来 `/ai/prove`、`/ai/next_tactics`、`/ai/search_step` のような wrapper を追加する場合は non-MVP です。
追加前に次を固定します。

```text
- request / response schema
- session / snapshot / state_fingerprint binding
- raw MachineTacticCandidate と metadata の分離
- MachineReplayPlan を含む成功 artifact
- failure / scheduler stop / partial result の taxonomy
- cache key と deterministic fingerprint input
```

wrapper が text proof script を返す場合も、それは display artifact です。
証明として保存するのは `/machine/replay` と `/machine/verify` に通った replay / certificate artifact だけです。

---

# 9. Training data

## 9.1 保存するデータ

Phase 7 では、成功・失敗の両方を学習データとして保存します。

```json
{
  "trace_schema": "npa.phase7.training-trace.v1",
  "session_root_hash": "sha256:...",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "node_id": 0,
  "batch_index": 0,
  "goal": {
    "goal_id": "g1",
    "open_goal_index": 0,
    "goal_fingerprint": "sha256:...",
    "target_hash": "sha256:...",
    "target_head": {
      "kind": "imported",
      "module": "Std.Init",
      "name": "Eq",
      "export_hash": "sha256:...",
      "decl_interface_hash": "sha256:...",
      "public_export": true,
      "tactic_head_visible": true
    },
    "target_free_local_count": 1,
    "context_size": 1,
    "expr_size": 9
  },
  "retrieved_premises": [
    {
      "premise_ref": {
        "module": "Std.Nat.Basic",
        "name": "Nat.add_zero",
        "export_hash": "sha256:...",
        "decl_interface_hash": "sha256:..."
      },
      "universe_params": [],
      "statement_core_hash": "sha256:...",
      "statement_head": {
        "kind": "imported",
        "module": "Std.Init",
        "name": "Eq",
        "export_hash": "sha256:...",
        "decl_interface_hash": "sha256:...",
        "public_export": true,
        "tactic_head_visible": true
      },
      "axioms_used": [],
      "modes": ["exact", "apply", "rw", "simp"],
      "response_index": 0
    }
  ],
  "tactic_candidates": [
    {
      "rank_index": 0,
      "phase7_candidate_payload_hash": "sha256:...",
      "candidate": {
        "kind": "simp-lite",
        "rules": []
      },
      "candidate_hash": "sha256:...",
      "result": "success",
      "deterministic_budget_hash": "sha256:...",
      "proof_delta_hash": "sha256:...",
      "next_snapshot_id": "mst_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "next_state_fingerprint": "sha256:..."
    },
    {
      "rank_index": 1,
      "phase7_candidate_payload_hash": "sha256:...",
      "candidate": {
        "kind": "rw",
        "rule": {
          "head": {
            "imported": {
              "name": "Nat.zero_add",
              "decl_interface_hash": "sha256:..."
            }
          },
          "universe_args": [],
          "args": [
            {"mode": "infer_from_target"}
          ]
        },
        "direction": "forward",
        "site": "eq_target_left"
      },
      "candidate_hash": "sha256:...",
      "result": "error",
      "error_kind": "rewrite_rule_invalid",
      "phase": "tactic_execution",
      "deterministic_budget_hash": "sha256:...",
      "diagnostic_hash": "sha256:...",
      "retryable": false
    }
  ]
}
```

MVP training trace は、`/machine/tactics/batch` response の `results.length > 0` で
evaluated result が 1 件以上ある batch だけを 1 record として保存します。
同じ `SearchNode` の retry batch は、保存対象になった場合だけ、同じ `node_id` かつ異なる `batch_index` を持つ別 record とします。
`batch_index` は node 展開内の saved training trace record index で、0 から始まり、
`record_training_trace_batch` で record を保存した場合だけ 1 ずつ増やします。
top-level batch error、suggested candidate hash mismatch、`results.length = 0` の zero-progress partial response は
training trace record を保存せず、`batch_index` も消費しません。
`rank_index` はその training trace record が表す batch 内だけの 0-based index です。
別 batch の `rank_index` と比較してはいけません。
擬似コード中の `record_training_trace_batch` は、この saved batch response 1 件に対して
training trace record 1 件を保存する関数です。
MVP controller は proof search 開始時に controller-local `training_trace_records = []` を作り、
`record_training_trace_batch` はこの配列に append します。
成功時は `Phase7VerifiedProof.training_trace_records`、失敗時は
`Phase7SearchFailure.training_trace_records` として同じ配列を返します。
MVP training trace artifact 全体の形は JSON array に固定します。
各要素はこの節の `trace_schema = "npa.phase7.training-trace.v1"` を持つ 1 record object で、
array order は `record_training_trace_batch` の呼び出し順です。
top-level wrapper object は置かず、NDJSON、1 record 1 file、後処理 sort 済み array は non-MVP storage profile とします。
training trace artifact に複数 record を保存する場合、保存順は `record_training_trace_batch` の呼び出し順に固定します。
後処理で並べ替える実装は使わず、replay / regression test もこの append order を比較対象にします。
top-level batch error では呼びません。
partial response では Phase 5 response の `results` prefix だけを `tactic_candidates` に反映します。
`results.length = 0` の zero-progress partial response では evaluated candidate がないため、
training trace record を保存しません。
その停止は `ZeroProgressSchedulerStopped` と `DeferredCandidateDropped` の trace event だけに記録します。
MVP training trace の `goal` は `GoalSummary` JSON shape を使います。
`retrieved_premises[*]` は `Phase7PremiseCacheEntry` JSON shape と同じ field set を使い、
`premise_ref`、`universe_params`、`statement_core_hash`、`statement_head`、`axioms_used`、`modes`、
`response_index` を必須にします。
`retrieved_premises[*].premise_ref` は `Phase7PremiseRef` JSON shape です。
`state.context` や `target` の pretty string、premise display string、natural-language theorem name は
training identity field にしてはいけません。
表示用に保存する場合は `display` sidecar に分離し、rank、dedupe、positive / negative 判定、cache key、
candidate hash、replay plan には使いません。

`tactic_candidates[*]` は実際に `/machine/tactics/batch` の `results` に現れた evaluated candidate だけを保存します。
未評価 deferred candidate、scheduler partial の停止中 candidate、top-level controller error はこの配列に入れず、
`Phase7VerifiedProof.trace_events` または `Phase7SearchFailure.trace_events` の trace event として保存します。
success item は `candidate_hash`、`deterministic_budget_hash`、`proof_delta_hash`、
`next_snapshot_id`、`next_state_fingerprint` を必須にします。
accepted error item は `candidate_hash`、`deterministic_budget_hash`、`error_kind`、`phase`、
`diagnostic_hash`、`retryable` を必須にします。
`goal_id` と `tactic_kind` は Phase 5 per-candidate item に存在する場合だけ保存します。
per-candidate error item で `diagnostic_hash` がないものは batch response contract violation として
探索全体を final `MachineControllerError` で終了するため、training trace record は保存しません。
per-candidate error item でも、`candidate_hash` がないもの、または
`error_kind` が Phase 5 `FailedCandidateErrorKind` に含まれないものは `tactic_candidates` に入れません。
それらは `Phase7TraceEventKind::NonAcceptedCandidateError` として保存し、repair、negative training、
prompt failed candidate context には使いません。
`phase7_candidate_payload_hash` は Phase 7 dedupe / audit 用であり、positive / negative の安定 identity は
Phase 5 `candidate_hash` を優先します。
MVP training trace には top-level `chosen_candidate_hash` や `new_state_fingerprint` を置きません。
1 回の batch で複数 success が返る場合、それぞれが別の child node になるため、採用された proof path は
`MachineReplayPlan.steps[*].candidate_hash` と `next_state_fingerprint` の列から別 artifact として復元します。

## 9.2 学習タスク

```text
premise selection:
  goal -> useful premises

tactic prediction:
  goal + premises -> next MachineTacticCandidate

value prediction:
  proof state -> solvability score

error repair:
  failed MachineTacticCandidate + Phase 5 error -> repaired MachineTacticCandidate

proof minimization:
  long MachineReplayPlan -> shorter MachineReplayPlan
```

## 9.3 Positive / negative examples

正例：

```text
identity:
  (state_fingerprint, goal_id, candidate_hash)

features / audit sidecar:
  MachineTacticCandidate raw payload
  phase7_candidate_payload_hash
  deterministic_budget_hash
  proof_delta_hash
  next_state_fingerprint
```

負例：

```text
identity:
  (state_fingerprint, goal_id, candidate_hash, error_kind, diagnostic_hash)

features / audit sidecar:
  MachineTacticCandidate raw payload
  phase7_candidate_payload_hash
  deterministic_budget_hash

included:
  deterministic failure with accepted candidate_hash and diagnostic_hash
  budget_exceeded only when it is Phase 5 FailedCandidateErrorKind::budget_exceeded
```

`scheduler_limits` 由来の timeout / resource stop は retryable scheduler artifact なので、
deterministic negative example にはしません。
`MachineTacticCandidate` raw payload は学習特徴量として保存できますが、positive / negative identity には使いません。
Phase 5 `candidate_hash` が存在する場合は常にそれを安定 identity とし、
`phase7_candidate_payload_hash` は Phase 7 dedupe / audit 用 sidecar に限定します。

負例が重要です。
「何をすべきか」だけでなく「何をすべきでないか」を学習できます。

---

# 10. Phase 7 AI Profile のマイルストーン

Phase 7 は、AI モデルを最初から入れる工程ではありません。
まず deterministic proof search controller を certificate-first な境界で完成させ、
その上に model generation、embedding retrieval、value model、parallel search を非信頼 sidecar として追加します。
各マイルストーンは、前段の replay / verify 境界を壊さずに次段へ渡せることを完了条件にします。

```text
M0. Machine API boundary fixed
M1. Deterministic snapshot and premise retrieval
M2. CandidateEnvelope and deterministic tactic generation
M3. Batch execution and replay step construction
M4. Deterministic best-first search controller
M5. Rule-based error repair
M6. Replay / verify closure
M7. Proof minimization
M8. Trace, training data, and audit artifacts
M9. MVP integration fixtures

M10. Model-based tactic generation
M11. Embedding retrieval sidecar
M12. Value model ranking
M13. Parallel search profile
```

M0-M9 が Phase 7 MVP です。
M10 以降は Phase 7.5 または later profile であり、MVP の replay plan、candidate hash、
training identity、verify result の意味を変えてはいけません。

## M0. Machine API boundary fixed

目的:

```text
- Phase 7 controller が Phase 5 Machine API client としてだけ動く境界を固定する
- 独自の /ai/* proof-state / tactic execution protocol を MVP に入れない
- Phase 5 未実装時の fake を同一 contract の deterministic test double に限定する
```

成果物:

```text
- MachineApiClient boundary
- production adapter:
    /machine/snapshots/get
    /machine/search/for_goal
    /machine/tactics/batch
    /machine/replay
    /machine/verify
- deterministic fake adapter for tests
- Phase7MvpControllerConfig validator
```

完了条件:

```text
- snapshot request は include_pretty = false に固定されている
- Phase 7 が state_id、text tactic、独自 hash を proof identity として使わない
- fake / mock が replay plan、certificate、state fingerprint、candidate hash、training identity に入らない
- invalid config では proof search を開始しない
```

## M1. Deterministic snapshot and premise retrieval

目的:

```text
- Phase 5 snapshot と /machine/search/for_goal の結果だけから premise source set を作る
- Phase 6 theorem index を Phase 7 の trusted input にしない
- premise retrieval の cache / ranking を非信頼 sidecar に閉じる
```

成果物:

```text
- initial snapshot loader
- goal selector
- phase7_mvp_premise_query builder
- Phase7PremiseRef / Phase7PremiseUsage collector
- retrieval cache key using Phase 5 fingerprints
```

完了条件:

```text
- response.results と suggested_candidates を Phase 5 response shape のまま読める
- verified premise locator は direct verified imports 由来の public theorem / axiom entry に限定される
- pretty / statement text が query identity、candidate identity、ranking identity に入らない
- stale global_ref / decl_interface_hash を持つ candidate は後段の Phase 5 validation で拒否される
```

## M2. CandidateEnvelope and deterministic tactic generation

目的:

```text
- raw MachineTacticCandidate を Phase 7 metadata から分離する
- MVP builtin generator を deterministic に固定する
- text tactic や general theorem synthesis に依存しない候補生成にする
```

成果物:

```text
- CandidateEnvelope
- phase7_candidate_payload_hash
- forbidden token filter
- deterministic candidate ordering / dedupe
- MVP generators:
    Phase 5 suggested_candidates
    Intro
    SimpLite { rules = [] }
    limited local Exact
    allowed_tactics gated InductionNat
```

完了条件:

```text
- Phase 7 が新規生成した候補は Phase 5 に渡すまで candidate_hash を作らない
- score / source / display_text / premises_used は candidate payload の外側にだけ存在する
- general premise Exact / Apply、local equality rw、constructor / cases / refine は non-MVP のまま残る
- candidate ordering は model score、HashMap iteration order、pretty text に依存しない
```

## M3. Batch execution and replay step construction

目的:

```text
- 候補を /machine/tactics/batch だけで transactional に検査する
- success result から MachineReplayStep を構成する
- accepted candidate failure を repair / trace 用に正規化する
```

成果物:

```text
- batch request builder
- batch policy capper
- candidate_hash mismatch detector
- deterministic_budget_hash checker
- proof_delta_hash based replay step builder
- AcceptedCandidateFailure normalizer
```

完了条件:

```text
- /machine/tactics/run は MVP search controller の実行経路に入らない
- success item だけが replay step になる
- scheduler stop は deterministic tactic failure として扱わない
- batch-level contract violation は final MachineControllerError で探索全体を終了する
```

## M4. Deterministic best-first search controller

目的:

```text
- proof state graph を deterministic event order で探索する
- visited identity を Phase 5 state_fingerprint に限定する
- budget と best partial failure を再現可能にする
```

成果物:

```text
- Phase7SearchNode
- deterministic NodeId allocator
- priority queue
- visited state_fingerprint set
- SearchBudget enforcement
- SearchStats counters
- best partial selection
```

完了条件:

```text
- priority queue pop、node expansion、trace append、stats update、child NodeId allocation が単一順序で固定される
- max_nodes、max_depth、max_tactics_per_node が探索に適用される
- duplicate state は trace に記録して探索 queue に入れない
- search_budget exceeded は証明失敗であり、kernel / checker failure と混同しない
```

## M5. Rule-based error repair

目的:

```text
- Phase 5 error kind をそのまま使って、決定的な修復候補だけを生成する
- repair が trusted proof boundary を広げないことを固定する
```

成果物:

```text
- RuleBasedRepair
- FailedCandidateErrorKind full classification
- RepairCandidateOutput / PendingCandidate integration
- repair chain dedupe by phase7_candidate_payload_hash
- RepairChainStopped trace events
```

完了条件:

```text
- repair に使うのは candidate_hash と diagnostic_hash を伴う accepted candidate failure だけである
- MVP repair は SimpLite { rules = [] } と限定的 local Exact 以外の一般合成をしない
- repair_depth <= 2 と同一 chain 内重複排除が守られる
- existing candidate の score rerank は non-MVP repair profile のまま残る
```

## M6. Replay / verify closure

目的:

```text
- open_goals empty を直接成功扱いしない
- 完全な MachineReplayPlan を replay し、final snapshot を verify する
- text proof ではなく replay / verify artifact を Phase 7 の成功結果にする
```

成果物:

```text
- MachineReplayPlan assembler
- /machine/replay integration
- /machine/verify integration
- Phase7VerifiedProof
- Phase7SearchFailure
```

完了条件:

```text
- /machine/replay success だけでは verified proof としない
- /machine/verify status = verified の response を成功結果に含める
- replay / verify 失敗は Phase 5 error phase を保ったまま扱う
- final_snapshot_id と final_state_fingerprint は verified replay result 由来である
```

## M7. Proof minimization

目的:

```text
- 探索で得た replay plan を短くする
- 最小化 proposal も必ず replay / verify で再検査する
```

成果物:

```text
- minimize_replay_plan
- delete_redundant_steps pass
- replace_blocks_with_simp_lite_empty pass
- minimize_existing_simp_lite_rules pass
- MinimizationStats
```

完了条件:

```text
- pass は文書で固定した順序で実行される
- 採用する proposal は /machine/replay と /machine/verify の両方を通る
- exact theorem replacement、namespace shortening、import minimization は MVP に入れない
- 最小化後の replay plan は deterministic budget hash を保存し直す
```

## M8. Trace, training data, and audit artifacts

目的:

```text
- 成功・失敗・探索停止を後から監査できる trace として残す
- 学習用 positive / negative identity を deterministic に固定する
- scheduler artifact と deterministic tactic failure を分離する
```

成果物:

```text
- Phase7TraceEvent
- Phase7TrainingTraceRecord
- trace_schema = "npa.phase7.training-trace.v1"
- positive / negative trace identity builder
- persistent log serializer
```

完了条件:

```text
- positive identity は state_fingerprint、goal_id、candidate_hash、proof_delta_hash、next_state_fingerprint に基づく
- negative identity は accepted candidate failure の candidate_hash と diagnostic_hash に基づく
- phase7_candidate_payload_hash は dedupe / audit sidecar に限定される
- scheduler timeout / resource stop は negative training example にしない
```

## M9. MVP integration fixtures

目的:

```text
- M0-M8 を一つの deterministic MVP prover として結合する
- Std.Nat / Std.List の基本 goal fixture で Phase 5 境界へ戻ることを確認する
```

成果物:

```text
- exact retrieval fixture
- simp-lite fixture
- local Exact fixture
- optional induction-nat fixture for induction-enabled sessions
- determinism regression tests
- no-model MVP profile test
```

完了条件:

```text
- 同一入力、同一 budget、同一 Phase 5 response から同一 trace / replay plan が得られる
- exact / simp-lite / local Exact の成功が /machine/tactics/batch、/machine/replay、/machine/verify を通る
- MVP で未指定の apply / rw / constructor / cases / refine は成功条件に入らない
- AI model、embedding、value model、parallel search を無効にした状態で Phase 7 MVP が成立する
```

## M10. Model-based tactic generation

目的:

```text
- /machine/prompt_payload を使い、LLM や小型モデルから tactic 候補を受け取る
- model output を deterministic MVP generator と同じ CandidateEnvelope 経路に入れる
```

完了条件:

```text
- model output は raw MachineTacticCandidate validation 前には proof として扱われない
- prompt text、model score、sampling seed は replay plan / certificate / verify identity に入らない
- model candidate は M2-M8 の filter、batch、repair、trace 境界をすべて通る
```

## M11. Embedding retrieval sidecar

目的:

```text
- theorem / goal embedding を premise ranking の非信頼 sidecar として追加する
- Phase 5 verified premise metadata を置き換えない
```

完了条件:

```text
- embedding score は retrieval ordering hint に限定される
- candidate は必ず Phase 5 suggested candidate または Phase 7 generator 経由で再検証される
- embedding index の stale / missing は探索品質低下であり、trusted proof failure ではない
```

## M12. Value model ranking

目的:

```text
- search trace から探索優先度を学習で改善する
- value model を non-trusted ranking metadata に限定する
```

完了条件:

```text
- value score は priority tie-break profile として別途固定される
- value score は candidate payload、replay step、certificate hash に入らない
- deterministic fallback ordering が常に存在する
```

## M13. Parallel search profile

目的:

```text
- node expansion や candidate execution を並列化する profile を追加する
- event order と trace order を deterministic に固定する
```

完了条件:

```text
- MVP の single-event-order controller とは別 profile として扱う
- response arrival order で NodeId、trace order、stats、best partial が変わらない
- Phase 5 batch prefix contract 以外の並列完了順を Phase 7 が観測しない
```

---

# 11. Phase 7 の最小構成

最初のMVPは、AIモデルなしでも成立します。

```text
premise retrieval:
  /machine/search/for_goal
  Phase 7 独自 index は非信頼 cache / ranking だけ

tactic generation:
  Phase 5 suggested_candidates
  Intro / SimpLite / limited local Exact raw MachineTacticCandidate
  InductionNat は Phase 5 snapshot の allowed_tactics に含まれる session でだけ生成
  general premise Exact / Apply は non-MVP

search:
  best-first over snapshot_id / state_fingerprint

repair:
  rule-based

minimization:
  replay step deletion + SimpLite { rules = [] } block replacement + existing simp-lite rule deletion

model:
  なし

/ai endpoints:
  なし
```

これだけでも、Phase 6 の標準ライブラリ上では多くの基本定理を自動で解けます。

AIモデルはその後に追加します。

```text
MVP:
  deterministic prover

Phase 7.5:
  LLM-assisted prover

Phase 7 later:
  learning prover
```

---

# 12. Phase 7 のテストケース

## 12.1 exact retrieval

```npa
theorem t (n : Nat) : n + 0 = n := by
  _
```

期待：

```text
first generate MachineTacticCandidate::Intro { name = "n" }
then search from the snapshot whose context contains n : Nat
retrieve Nat.add_zero
do not rely on Phase 5 exact suggested_candidates
do not generate general premise Exact from Nat.add_zero in MVP
generate MachineTacticCandidate::SimpLite { rules = [] } as the MVP closing candidate
/machine/tactics/batch per-candidate success
/machine/replay success
/machine/verify verified
```

## 12.2 simp-lite

```npa
theorem t (n : Nat) : n + 0 = n := by
  _
```

期待：

```text
first generate MachineTacticCandidate::Intro { name = "n" }
then search from the snapshot whose context contains n : Nat
generate MachineTacticCandidate::SimpLite
/machine/tactics/batch per-candidate success
minimized replay plan still uses simp-lite
```

## 12.3 rw

```npa
theorem t (a b : Nat) (h : a = b) : a = b := by
  _
```

期待：

```text
first generate Intro candidates for a, b, h
then search from the snapshot whose context contains a b : Nat and h : a = b
generate MachineTacticCandidate::Exact with RawMachineTerm.source = "h"
/machine/tactics/batch per-candidate success closes the goal
local equality rw is non-MVP until direction/site/args generation is specified
```

## 12.4 apply later

```npa
theorem t {A : Type} (x y z : A)
  (h1 : x = y) (h2 : y = z) : x = z := by
  _
```

期待：

```text
first generate Intro candidates for A, x, y, z, h1, h2
then search from the snapshot whose context contains h1 and h2
MachineTacticCandidate::Apply Eq.trans is non-MVP unless its universe_args and args are fully specified
subgoal 1 closed by MachineTacticCandidate::Exact h1
subgoal 2 closed by MachineTacticCandidate::Exact h2
```

このテストは MVP の完了条件には入れません。
MVP で有効にする場合は、`Eq.trans` の `TacticHead`、`universe_args`、
`CandidateApplyArg` list を fully specified JSON として固定してから追加します。

## 12.5 induction-nat

この fixture は induction-enabled session 専用です。
Phase 5 snapshot の `allowed_tactics` に `induction-nat` が含まれ、対応する session の
`MachineTacticEnv.nat_family = Some(_)` であることを前提にします。
Phase 6 の MVP emitted simp/rw recipes は `nat_family = null` なので、この fixture は baseline MVP 完了条件には入れません。

```npa
theorem t (n : Nat) : 0 + n = n := by
  _
```

期待：

```text
first generate MachineTacticCandidate::Intro { name = "n" }
then search from the snapshot whose context contains n : Nat
generate MachineTacticCandidate::InductionNat { local_name = "n" }
base solved by simp-lite
step solved by simp-lite
```

## 12.6 List induction later

```npa
theorem t {A : Type} (xs : List A) : xs ++ [] = xs := by
  _
```

期待：

```text
non-MVP until an induction-list or general induction tactic is specified
```

---

# 13. Phase 7 でまだ入れないもの

MVPでは後回しにするもの：

```text
- 自然言語からの形式化
- 補題生成 have
- constructor / cases / refine を使う高度探索
- full simp
- ring / omega / linarith
- MCTS
- RL training
- List induction / general induction
- theorem生成
- 複数ファイルをまたぐ大規模自動補題発見
- 証明戦略の自然言語説明
```

Phase 7 の最初の目標は、**既存の tactic と標準ライブラリを使って、基本定理を自動で見つけること**です。

---

# 14. Phase 7 の完了条件

Phase 7 MVP が完了したと言える条件は、10 章の M0-M9 がすべて満たされていることです。
要約するとこれです。

```text
- M0: Phase 5 Machine API client 境界だけで探索 controller が動く
- 現在goalから relevant premise を検索できる
- Phase 5 suggested_candidates と MVP builtin から raw MachineTacticCandidate を作れる
- general exact/apply 生成を non-MVP として扱える
- candidate候補を複数生成できる
- best-first search で proof state graph を探索できる
- tactic失敗時に構造化エラーを保存できる
- rule-based error repair が動く
- open_goals empty 後に /machine/replay と /machine/verify できる
- 証明が見つかったら replay plan minimization できる
- search trace / training trace を audit 可能な artifact として保存できる
- Std.Nat / Std.List の基本 fixture を AI model なしで自動証明できる
```

M10-M13 は Phase 7.5 / later profile の完了条件であり、Phase 7 MVP の完了を待たせません。
これらを追加しても、MVP の replay / verify 境界と deterministic fallback は維持します。

---

# 15. 一文でまとめると

Phase 7 は、**構造化 proof state と標準ライブラリを使って、AI・検索・tactic を組み合わせた検証駆動の証明探索エンジンを作る段階**です。

中核はこの流れです。

```text
retrieve useful premises
  ↓
generate raw MachineTacticCandidate candidates
  ↓
try them through /machine/tactics/batch
  ↓
repair failures
  ↓
search best-first
  ↓
build and replay MachineReplayPlan
  ↓
/machine/verify final snapshot
  ↓
minimize verified replay plan
```

最初はAIモデルなしの deterministic search で作り、あとから LLM・embedding retriever・value model を追加するのが安全です。
この順序なら、AIが間違えても証明器全体の信頼性は壊れません。

[1]: https://leandojo.readthedocs.io/?utm_source=chatgpt.com "LeanDojo: Machine Learning for Theorem Proving in Lean ..."
[2]: https://arxiv.org/abs/2306.15626?utm_source=chatgpt.com "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models"
[3]: https://www.nature.com/articles/s41586-025-09833-y?utm_source=chatgpt.com "Olympiad-level formal mathematical reasoning with ..."
