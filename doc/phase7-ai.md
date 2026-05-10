以下は **Phase 7: AI探索** の詳細設計です。
Phase 7 の目的は、Phase 1〜6 で作った kernel・certificate・tactic・IDE/API・標準ライブラリを使って、**AIが証明候補を探索し、kernel が正しいものだけ採用する仕組み**を作ることです。

対象はこの5つです。

```text
- premise retrieval
- tactic generation
- best-first search
- error repair
- proof minimization
```

設計原則は一貫してこれです。

```text
AIは信用しない。
AIは候補を出す。
tactic engine が試す。
kernel と certificate checker が検証する。
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
│ Kernel / Certificate Checker  │
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
`doc/phase5-ai.md` の Machine API client として実装します。
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
      "allowed_tactics": ["intro", "exact", "apply", "rw", "simp-lite", "induction-nat"]
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
}
```

`MachineReplaySuccess` と `MachineVerifySuccess` は Phase 5 Machine API の success response body を指す別名です。
Phase 7 独自の success schema を新しく定義してはいけません。

失敗時：

```rust
struct Phase7SearchFailure {
    reason: SearchFailureReason,
    best_partial_replay_prefix: Option<Vec<MachineReplayStep>>,
    best_snapshot_id: Option<SnapshotId>,
    best_state_fingerprint: Option<Hash>,
    remaining_goals: Option<Vec<GoalSummary>>,
    search_stats: SearchStats,
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
```

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

MVP で Phase 7 が保存・比較してよい verified premise identity は、
Phase 5 search response の `global_ref.module`、`global_ref.name`、`global_ref.export_hash`、
`global_ref.decl_interface_hash`、`universe_params`、`statement.core_hash`、`axioms_used` です。
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
          "contains_forbidden_tokens": false
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
            "module": "Std.Nat.Basic",
            "name": "Nat.add_zero",
            "export_hash": "sha256:...",
            "decl_interface_hash": "sha256:..."
          }
        ],
        "expected_effect": "rewrite",
        "cost_estimate": {
          "estimated_timeout_ms": 200,
          "risk": "medium"
        },
        "trust_flags": {
          "uses_axioms": [],
          "contains_forbidden_tokens": false
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
Phase 5 の `MachineProofSnapshot` wire payload は resolved `nat_family` 本体を返さないため、
Phase 7 MVP は Nat family の一致判定をしません。

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
    premises_used: Vec<Phase7PremiseRef>,
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
}

struct Phase7PremiseRef {
    module: ModuleName,
    name: FullyQualifiedName,
    export_hash: Hash,
    decl_interface_hash: Hash,
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
`Phase7PremiseRef` の JSON trace shape は Phase 5 `/machine/search/for_goal` result の `global_ref` object と同じ
`module` / `name` / `export_hash` / `decl_interface_hash` です。
`premises_used` に display string だけを保存してはいけません。
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
      "contains_forbidden_tokens": false
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
    used_premises: Vec<Phase7PremiseRef>,

    parent: Option<NodeId>,
    status: NodeStatus,
}

struct PendingCandidate {
    goal_id: GoalId,
    candidate: CandidateEnvelope,
    repair_depth: u32,
    parent_candidate_hash: Hash,
    error_kind: FailedCandidateErrorKind,
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
それらは `NodeStopReason` / trace event として記録し、priority queue や replay plan construction の入力には使いません。

`PendingCandidate` は同じ `SearchNode` を再度 queue に入れるための state ではなく、
その node を展開している間だけ使う local retry item です。
`visited` は `state_fingerprint` だけを key にするため、同じ state を pending repair 付きで
priority queue に戻してはいけません。

`PendingCandidate` は accepted candidate failure からだけ作ります。
したがって `parent_candidate_hash` と `error_kind` は、5.5 の `AcceptedCandidateFailure` から必ず設定できます。
`limit_repairs(max_per_parent=3)` は `(goal_id, parent_candidate_hash, error_kind)` ごとに request order を保って
先頭 3 件だけを残します。
`PendingCandidate` を batch に送るときは `candidate` の `CandidateEnvelope` だけを wire payload に変換します。
`repair_depth`、`parent_candidate_hash`、`error_kind` は `CandidateEnvelope.metadata.repair` にもコピーし、
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
    failure_reason = SearchFailureReason.QueueExhausted
    initial_no_candidate_goal = None

    while queue:
        if search_budget.wall_clock_exceeded():
            failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.WallClock)
            break

        node = queue.pop_best()

        if node.state_fingerprint in visited:
            continue
        visited.add(node.state_fingerprint)
        node.status = NodeStatus.Expanded

        snapshot_result = get_snapshot(node.session_id, node.snapshot_id, node.state_fingerprint)
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
                )
                return Phase7VerifiedProof(
                    replay_plan=minimized_plan,
                    final_snapshot_id=minimized_replay.final_snapshot_id,
                    final_state_fingerprint=minimized_replay.final_state_fingerprint,
                    verify_response=minimized_verify,
                    search_stats=stats,
                    minimization_stats=minimization_stats,
                )
            record_closed_node_verify_rejection(node, verified)
            stats.closed_node_verify_rejections += 1
            continue

        if best_partial is None or is_better_partial(node, best_partial):
            best_partial = node
            stats.best_partial_updates += 1

        if node.depth >= search_budget.max_depth:
            failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.MaxDepth)
            record_depth_stop(node)
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
            record_machine_controller_error(node, endpoint="/machine/search/for_goal", result=premises_result)
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
                record_batch_controller_error(node, result)
                stats.controller_errors += 1
                return SearchFailure(
                    reason=machine_controller_error_reason(
                        endpoint="/machine/tactics/batch",
                        result=result,
                    ),
                    best_partial=best_partial,
                )

            stopped_candidate_id = None
            if is_scheduler_partial(result):
                record_scheduler_stop(node, result.scheduler_artifact)
                stats.scheduler_stops += 1
                if len(result.results) == 0:
                    stats.zero_progress_scheduler_stops += 1
                    break
                if len(result.results) < len(candidate_items):
                    stopped_candidate_id = candidate_items[len(result.results)].candidate_id

            evaluated_for_node += len(result.results)
            stats.candidates_evaluated += len(result.results)
            next_repairs = []
            completed_candidate_ids = set(item.candidate_id for item in result.results)
            deferred_candidates = [
                candidate_by_id[item.candidate_id]
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
                        continue
                    if repeated_repair_error(envelope, failure):
                        record_repair_chain_stop(envelope, failure)
                        continue
                    parent_repair_depth = repair_depth_of(envelope)
                    if parent_repair_depth >= 2:
                        continue
                    repairs = repair_candidate(
                        snapshot,
                        goal,
                        envelope,
                        failure,
                        repair_depth=parent_repair_depth + 1,
                    )
                    next_repairs.extend(repairs)

            if not next_repairs and not deferred_candidates:
                break
            if evaluated_for_node >= search_budget.max_tactics_per_node:
                break

            pending_repairs = limit_repairs(
                [r for r in next_repairs if r.repair_depth <= 2],
                max_per_parent=3,
            )
            if not pending_repairs and not deferred_candidates:
                break

    if search_budget.wall_clock_exceeded():
        failure_reason = SearchFailureReason.SearchBudgetExceeded(limit=SearchBudgetLimit.WallClock)
    elif initial_no_candidate_goal is not None and best_partial is not None and best_partial.parent is None:
        failure_reason = SearchFailureReason.NoCandidateForSelectedGoal(goal_id=initial_no_candidate_goal)

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

`make_child` は `item.next_snapshot_id` / `item.next_state_fingerprint` を使って `/machine/snapshots/get` を呼び、
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
重複判定は `Phase7PremiseRef` の `module` / `name` / `export_hash` / `decl_interface_hash` の tuple 完全一致で行い、
既に存在する premise は追加しません。
この field は replay plan、certificate、state fingerprint、candidate hash には入れてはいけません。

`search_budget.max_nodes` は、`visited` に入った後、候補生成を開始する前の unique `SearchNode` 展開数を数えます。
同じ `state_fingerprint` で skip された node は数えません。
`search_budget.max_depth` は `node.depth >= max_depth` の node を展開しない上限です。
ただし open goals が空の node は depth check の前に `/machine/replay` と `/machine/verify` へ進めます。
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

`merge_node_candidates` の bucket priority は次に固定します。

```text
1. deferred_candidates:
   直前 batch の retryable 未評価 suffix。scheduler partial の停止 candidate は含めない。
   直前 request order を保持し、この bucket 内では rerank しない。
2. pending_repairs:
   `limit_repairs` 後の repair 候補。下の `repair_rank_key` だけで bucket 内 rank する。
3. fresh_candidates:
   この node の初回生成候補。通常の rank を使う。
```

`rank_and_filter_with_bucket_priority` は bucket 間の順序を入れ替えてはいけません。
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
`source_index = repair_candidate` が返す配列内の 0-based emission index として trace に保存します。

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
永続ログでは、Phase 5 validation 後の `candidate_hash`、rank index、Phase 7 candidate payload hash を保存します。

```rust
type CandidateBatchItem = {
    candidate_id: CandidateId,
    candidate: MachineTacticCandidate,
};

candidate_items: Vec<CandidateBatchItem>
candidate_by_id: Map<CandidateId, CandidateEnvelope>
```

Phase 7 candidate payload hash は、Phase 5 validation 前の dedupe / local log 用 identity です。
trusted hash、replay hash、certificate hash ではありません。

```text
phase7_candidate_payload_hash =
  sha256(
    "npa.phase7.candidate-payload.v1" ||
    canonical_json(MachineTacticCandidate raw wire payload)
  )
```

`canonical_json` は Phase 5 の lossless request decoding 後、duplicate key がない raw candidate object だけに使います。
object key は bytewise lexicographic order、array order は保持、string は UTF-8 bytes、integer は shortest decimal です。
Phase 7 が typed `MachineTacticCandidate` から候補を生成した場合は、まず Phase 5 `/machine/tactics/batch`
inner `candidate` と同じ raw wire object に変換します。
variant name、required field、empty array、level / local name / tactic head の JSON shape は Phase 5 7.0 schema と同一にします。
omitted optional field や display-only metadata を追加してはいけません。
Phase 5 `suggested_candidates[*].candidate` 由来の候補は、response の raw candidate object をそのまま使います。
repair 候補も同じ wire emitter を使うため、同じ raw candidate なら builtin / suggested / repair の由来に関係なく
同じ `phase7_candidate_payload_hash` になります。
Phase 7 はこの hash で `dedupe_by_canonical_candidate_payload` と pending repair の重複排除を行います。
Phase 5 が `candidate_hash` を返した後の replay / verify / training identity では、常に Phase 5 `candidate_hash` を優先します。

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
`candidate_hash` と `diagnostic_hash` のどちらかがない result、または Phase 5 `FailedCandidateErrorKind`
に含まれない kind は `AcceptedCandidateFailure` にせず、repair と negative training には使いません。

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
`repeated_repair_error(envelope, failure)` は、
`envelope.metadata.repair = Some(repair)` かつ `repair.error_kind == failure.error_kind` の場合だけ true です。
この打ち切りは repair chain 単位であり、同じ node 内の別 candidate や fresh candidate には伝播しません。
`repair_candidate(snapshot, goal, envelope, failure, repair_depth)` は `Vec<PendingCandidate>` を返します。
返却順は repair rule table の記載順、同一 rule 内では generated candidate の deterministic emission order です。
各 item の `parent_candidate_hash = failure.candidate_hash`、
`error_kind = failure.error_kind`、`repair_depth = repair_depth` に固定します。
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

並列化できる箇所：

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
下に個別規則がない error kind は実装者判断で repair してはいけません。

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

MVP で repair rule が `simp-lite` を追加すると書く場合、返す pending repair candidate は次の 1 種だけです。

```text
candidate = MachineTacticCandidate::SimpLite { rules = [] }
metadata.source = Repair
metadata.premises_used = []
metadata.expected_effect = Simplify
metadata.repair.parent_candidate_hash = failure.candidate_hash
metadata.repair.error_kind = failure.error_kind
metadata.repair.repair_depth = repair_depth
```

`goal.allowed_tactics` に `"simp-lite"` がない場合、または同じ `phase7_candidate_payload_hash` が同じ repair chain で既に試されている場合、
その rule は `[]` を返します。
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

repair の出力例（MVP）：

```json
{
  "repairs": [
    {
      "candidate": {
        "kind": "simp-lite",
        "rules": []
      },
      "reason": "type_mismatch on exact; try deterministic simplification before broader search",
      "repair_depth": 1,
      "display_text": "simp-lite"
    }
  ]
}
```

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

### Pass 3: exact replacement

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

### Pass 4: premise simplification

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
def minimize_replay_plan(verified_replay_plan, verified_replay, verified_response, session):
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
            step_edits = make_step_edits_with_goal_indices(current_plan, session)
            if step_edits is None:
                break

            for proposed_steps in minimization_pass.proposals(step_edits, session):
                rebuilt = rebuild_replay_plan_from_step_edits(current_plan, proposed_steps, session)
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
`make_step_edits_with_goal_indices(current_plan, session)` は `current_plan` の initial state から step を順に再実行し、
各 step を実行する直前の `MachineProofSnapshot.open_goals` 内で `step.goal_id` が出現する 0-based index を記録して
`original_open_goal_index` に入れます。
initial snapshot lookup では Phase 5 の規則どおり
`snapshot_id_from_state_fingerprint(current_plan.initial_state_fingerprint)` を使い、
`/machine/snapshots/get` に `session.session_id`、その snapshot id、`current_plan.initial_state_fingerprint` を渡します。
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

`rebuild_replay_plan_from_step_edits` は initial snapshot から順に実行し直し、
現在 snapshot に `original_goal_id` が残っていればそれを使います。
残っていない場合は `original_open_goal_index` で同じ位置の open goal を選びます。
どちらでも一意に選べない場合、その minimization pass は失敗として元の plan を維持します。
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

ユーザー設定で選べるようにします。

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

後で endpoint 化する場合は、input を `proof_script` ではなく complete `MachineReplayPlan` とし、
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
      "deterministic_budget_hash": "sha256:...",
      "diagnostic_hash": "sha256:..."
    }
  ],
  "chosen_candidate_hash": "sha256:...",
  "new_state_fingerprint": "sha256:..."
}
```

MVP training trace の `goal` は `GoalSummary` JSON shape、`retrieved_premises[*].premise_ref` は
`Phase7PremiseRef` JSON shape を使います。
`state.context` や `target` の pretty string、premise display string、natural-language theorem name は
training identity field にしてはいけません。
表示用に保存する場合は `display` sidecar に分離し、rank、dedupe、positive / negative 判定、cache key、
candidate hash、replay plan には使いません。

`tactic_candidates[*]` は実際に `/machine/tactics/batch` の `results` に現れた evaluated candidate だけを保存します。
未評価 deferred candidate、scheduler partial の停止中 candidate、top-level controller error はこの配列に入れず、
別の trace event として保存します。
success item は `candidate_hash`、`deterministic_budget_hash`、`proof_delta_hash`、
`next_snapshot_id`、`next_state_fingerprint` を必須にします。
accepted error item は `candidate_hash`、`deterministic_budget_hash`、`error_kind`、`diagnostic_hash` を必須にします。
`phase7_candidate_payload_hash` は Phase 7 dedupe / audit 用であり、positive / negative の安定 identity は
Phase 5 `candidate_hash` を優先します。

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
state_fingerprint, goal_id, MachineTacticCandidate で成功したもの
```

負例：

```text
state_fingerprint, goal_id, MachineTacticCandidate で deterministic failure になったもの
state, irrelevant premise
budget_exceeded candidate
loop candidate
```

`scheduler_limits` 由来の timeout / resource stop は retryable scheduler artifact なので、
deterministic negative example にはしません。

負例が重要です。
「何をすべきか」だけでなく「何をすべきでないか」を学習できます。

---

# 10. Phase 7 MVP の実装順序

おすすめ順はこれです。

```text
1. Deterministic premise retrieval
   /machine/search/for_goal を呼び、verified premise metadata と suggested_candidates を受け取る

2. Template tactic generation
   raw MachineTacticCandidate を生成する
   text tactic ではなく CandidateEnvelope にする
   MVP では Phase 5 suggested_candidates、Intro、SimpLite、限定的な local Exact を優先する

3. Best-first search engine
   priority queue, visited set = state_fingerprint, search_budget

4. Tactic execution integration
   /machine/tactics/batch を transactional に呼ぶ
   success から MachineReplayStep を組み立てる

5. Rule-based error repair
   type_mismatch, rewrite_rule_invalid, implicit_argument_required など
   Phase 5 error kind をそのまま使う

6. Proof found -> replay / certificate verification
   open_goals empty だけで成功にしない
   MachineReplayPlan を /machine/replay に通し、final snapshot を /machine/verify する

7. Proof minimization
   replay step deletion, SimpLite { rules = [] } block replacement, existing simp-lite rule deletion
   exact theorem replacement は MVP では実装しない
   最小化後は replay plan を作り直す

8. Search trace logging
   成功・失敗を保存

9. Model-based tactic generation
   LLMや小型モデルを追加

10. Embedding retrieval
    theorem / goal embedding を追加

11. Value model
    探索優先度を学習で改善

12. Parallel search
    tactic候補を並列試行
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
  Intro / SimpLite / InductionNat / limited local Exact raw MachineTacticCandidate
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

Phase 7 が完了したと言える条件はこれです。

```text
- 現在goalから relevant premise を検索できる
- Phase 5 suggested_candidates と MVP builtin から raw MachineTacticCandidate を作れる
- general exact/apply 生成を non-MVP として扱える
- candidate候補を複数生成できる
- best-first search で proof state graph を探索できる
- tactic失敗時に構造化エラーを保存できる
- rule-based error repair が動く
- open_goals empty 後に /machine/replay と /machine/verify できる
- 証明が見つかったら replay plan minimization できる
- search trace を学習データとして保存できる
- Std.Nat の基本定理を自動証明できる
```

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
