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

```json
{
  "state_id": "st_100",
  "goals": [
    {
      "goal_id": "g1",
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
      }
    }
  ]
}
```

`pretty` や `statement` は人間・AI向け表示なので、Phase 6 の表示用省略として `0` を使うことがあります。
探索や certificate 検査で使う構造情報は `constants` / `core_hash` 側であり、`0` は `Nat.zero` への参照として扱います。

加えて、Phase 6 の theorem index を使います。

```json
{
  "name": "Nat.add_zero",
  "statement": "∀ n : Nat, n + 0 = n",
  "attributes": ["simp", "rw"],
  "suggested_tactics": [
    "exact Nat.add_zero n",
    "rw [Nat.add_zero]",
    "simp-lite"
  ],
  "axioms_used": []
}
```

## 2.2 出力

成功時：

```json
{
  "status": "verified",
  "proof_script": "by\n  simp-lite",
  "certificate_hash": "sha256:...",
  "checked_by": ["kernel", "certificate_checker"],
  "axioms_used": [],
  "search_stats": {
    "expanded_nodes": 12,
    "tried_tactics": 41,
    "failed_tactics": 29,
    "time_ms": 184
  }
}
```

失敗時：

```json
{
  "status": "not_found_within_budget",
  "best_partial_proof": [
    "intro n",
    "apply Eq.trans"
  ],
  "remaining_goals": [
    {
      "pretty": "⊢ ..."
    }
  ],
  "search_stats": {
    "expanded_nodes": 5000,
    "tried_tactics": 18720,
    "time_ms": 30000
  }
}
```

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

```rust
struct PremiseIndexEntry {
    global_ref: GlobalRef,
    name: NameId,
    module: ModuleName,

    statement: ExprId,
    statement_pretty: String,

    head_symbol: Option<NameId>,
    constants: Vec<GlobalRef>,
    local_patterns: Vec<ExprPattern>,

    attributes: Vec<Attribute>,
    rewrite_info: Option<RewriteInfo>,

    theorem_kind: TheoremKind,
    axiom_deps: Vec<GlobalRef>,

    proof_term_size: u32,
    usage_count: u64,

    embedding_id: Option<EmbeddingId>,
    graph_neighbors: Vec<GlobalRef>,
}
```

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

```text
1. exact retrieval
2. apply retrieval
3. rw retrieval
4. simp retrieval
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

suggest:
  exact Nat.add_zero n
```

### apply retrieval

定理の結論を target に unify できるかを調べます。

```text
goal:
  ⊢ x = z

candidate:
  Eq.trans : x = y -> y = z -> x = z

suggest:
  apply Eq.trans
```

### rw retrieval

target 内の subterm に rewrite rule が使えるか調べます。

```text
target:
  f (n + 0) = f n

rule:
  Nat.add_zero : n + 0 = n

suggest:
  rw [Nat.add_zero]
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

MVPでは後回しでもよいですが、Phase 7 後半で導入します。

---

## 3.5 Retriever score

各 premise にスコアを付けます。

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
  "score": 0.982,
  "modes": ["exact", "rw", "simp"],
  "suggested_tactics": [
    "exact Nat.add_zero n",
    "rw [Nat.add_zero]",
    "simp-lite"
  ]
}
```

---

## 3.6 Premise retrieval API

```json
POST /ai/retrieve_premises
{
  "state_id": "st_100",
  "goal_id": "g1",
  "limit": 32,
  "modes": ["exact", "apply", "rw", "simp", "semantic"],
  "trust": {
    "allow_axioms": [],
    "deny_custom_axioms": true
  }
}
```

レスポンス：

```json
{
  "premises": [
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "score": 0.982,
      "modes": ["exact", "rw", "simp"],
      "suggested_tactics": [
        "exact Nat.add_zero n",
        "rw [Nat.add_zero]",
        "simp-lite"
      ],
      "axioms_used": []
    },
    {
      "name": "Eq.refl",
      "statement": "∀ x, x = x",
      "score": 0.841,
      "modes": ["exact"],
      "suggested_tactics": [
        "exact Eq.refl n"
      ],
      "axioms_used": []
    }
  ]
}
```

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
tactic candidates
```

例：

```json
{
  "candidates": [
    {
      "tactic": "simp-lite",
      "score": 0.94,
      "source": "builtin"
    },
    {
      "tactic": "exact Nat.add_zero n",
      "score": 0.91,
      "source": "retriever"
    },
    {
      "tactic": "rw [Nat.add_zero]",
      "score": 0.75,
      "source": "template"
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

goal の形だけで定番 tactic を出します。

```text
target が Pi:
  intro x

target が Eq t t:
  exact Eq.refl t

context に h : target:
  exact h

target に Nat が出る:
  induction n

target に単純なrewrite可能部分:
  simp-lite
```

### TemplateGenerator

retrieved premise から tactic template を作ります。

```text
premise P:
  exact P ...
  apply P
  rw [P]
  rw [<- P]
  simp-lite
```

### PremiseBasedGenerator

retriever の suggested tactic をそのまま使います。

```text
Nat.add_zero
  -> exact Nat.add_zero n
  -> rw [Nat.add_zero]
```

### ModelGenerator

LLMまたは小型モデルに tactic 候補を出させます。

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
    "induction"
  ],
  "failed_tactics": []
}
```

出力：

```json
{
  "candidates": [
    "simp-lite",
    "exact Nat.add_zero n",
    "rw [Nat.add_zero]"
  ]
}
```

### RepairGenerator

失敗した tactic とエラーから修正版を作ります。

```text
exact Eq.refl
error: could not infer implicit argument A

repair:
  exact Eq.refl n
  exact @Eq.refl Nat n
```

### ExplorationGenerator

探索用に少し低確率の候補を混ぜます。

```text
apply Eq.trans
induction n
rw [<- Nat.add_zero]
```

これは局所最適を避けるためです。

---

## 4.3 tactic candidate schema

```rust
struct TacticCandidate {
    tactic_text: String,
    tactic_ast: Option<TacticSyntax>,

    source: CandidateSource,
    score: f32,

    premises_used: Vec<GlobalRef>,
    expected_effect: ExpectedEffect,

    cost_estimate: CostEstimate,
    trust_flags: TrustFlags,
}
```

JSON：

```json
{
  "tactic": "rw [Nat.add_zero]",
  "source": "template",
  "score": 0.76,
  "premises_used": ["Nat.add_zero"],
  "expected_effect": "rewrite",
  "cost_estimate": {
    "timeout_ms": 100,
    "risk": "low"
  },
  "trust_flags": {
    "uses_axioms": [],
    "contains_forbidden_tokens": false
  }
}
```

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
  induction
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

    state_id: StateId,
    goals: Vec<GoalSummary>,

    proof_script: Vec<String>,
    proof_deltas: Vec<ProofDelta>,

    depth: u32,
    cumulative_score: f32,

    last_tactic: Option<String>,
    used_premises: Vec<GlobalRef>,

    parent: Option<NodeId>,
    status: NodeStatus,
}
```

`GoalSummary`：

```rust
struct GoalSummary {
    goal_id: GoalId,
    target_hash: Hash,
    target_head: Option<NameId>,
    context_size: u32,
    expr_size: u32,
}
```

---

## 5.3 State hash

同じ proof state を何度も探索しないため、state hash を使います。

```text
state_hash =
  hash(
    multiset of goals:
      context core hashes
      target core hash
      unresolved metavars summary
  )
```

注意：

```text
goal_id は hash に入れない。
binder名も意味的には入れない。
core構造を正規化してhash化する。
```

これにより：

```text
intro x
```

と

```text
intro y
```

で本質的に同じ state なら重複排除できます。

---

## 5.4 優先度関数

best-first search では、優先度の高い node から展開します。

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
def prove(initial_state, budget):
    root = SearchNode(
        state=initial_state,
        proof_script=[],
        score=score_initial(initial_state),
    )

    queue = PriorityQueue()
    queue.push(root, priority=root.score)

    visited = set()
    best_partial = root

    while queue and not budget.exceeded():
        node = queue.pop_best()

        state_hash = hash_state(node.state)
        if state_hash in visited:
            continue
        visited.add(state_hash)

        if is_better_partial(node, best_partial):
            best_partial = node

        if node.state.goals_empty():
            cert = build_certificate(node)
            if kernel_check(cert) and certificate_check(cert):
                minimized = minimize_proof(node)
                return VerifiedProof(minimized)

        goal = select_goal(node.state)

        premises = retrieve_premises(node.state, goal)
        tactics = generate_tactics(node.state, goal, premises)
        tactics = rank_and_filter(tactics)

        for tac in tactics:
            result = run_tactic_transactional(node.state, goal, tac)

            if result.success:
                child = make_child(node, result, tac)
                queue.push(child, priority=score(child))
            else:
                repairs = repair_tactic(node.state, goal, tac, result.error)
                for repaired in repairs:
                    queue.push_repair(node, repaired)

    return SearchFailure(best_partial=best_partial)
```

---

## 5.6 Goal selection

複数 goal がある場合、どれを先に解くかが重要です。

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

```json
{
  "timeout_ms": 30000,
  "max_nodes": 10000,
  "max_tactics_per_node": 32,
  "max_depth": 64,
  "max_open_goals": 50,
  "max_term_size": 100000,
  "max_memory_mb": 1024
}
```

予算超過時は、best partial を返します。

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

MVP の推奨：

```text
- beam width: 32
- max tactics per node: 16
- deterministic tactics first
- model tactics second
- risky tactics later
```

---

# 6. Error repair

## 6.1 目的

AIやテンプレートが出した tactic は頻繁に失敗します。
error repair は、失敗情報を使って修正候補を生成します。

例：

```text
tactic:
  exact Eq.refl

error:
  could not infer implicit argument

repair:
  exact Eq.refl n
  exact @Eq.refl Nat n
```

---

## 6.2 Error schema

Phase 5 の tactic execution API から構造化エラーを受け取ります。

```json
{
  "status": "error",
  "error_kind": "type_mismatch",
  "tactic": "exact h",
  "expected": "n = n",
  "actual": "Nat",
  "context": [
    {"name": "n", "type": "Nat"},
    {"name": "h", "type": "Nat"}
  ],
  "target": "n = n"
}
```

---

## 6.3 repair rule table

### `unknown_identifier`

```text
原因:
  定理名や仮定名が間違っている

repair:
  - theorem search で近い名前を探す
  - namespace を補う
  - typo correction
```

例：

```text
Nat.add_zro
  -> Nat.add_zero
```

### `type_mismatch`

```text
原因:
  exact/apply の型が target と合わない

repair:
  - exact を apply に変える
  - Eq.symm を使う
  - rw を先に試す
  - simp-lite を先に試す
```

例：

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
  - intro を削除
  - exact / apply / simp-lite を試す
```

### `rewrite_not_found`

```text
原因:
  rw rule の lhs が target に出現しない

repair:
  - 逆向き rewrite を試す
  - theorem search で別の rewrite rule を探す
  - simp-lite を試す
```

例：

```text
failed:
  rw [Nat.zero_add]

repair:
  rw [Nat.add_zero]
```

### `unsolved_implicit`

```text
原因:
  implicit argument が推論できない

repair:
  - `@` で明示引数を付ける
  - expected type から型を補う
```

例：

```text
Eq.refl n
  -> @Eq.refl Nat n
```

### `too_many_goals`

```text
原因:
  apply が大量の subgoal を作った

repair:
  - より具体的な定理を使う
  - exact 候補を優先する
  - この tactic のスコアを下げる
```

### `timeout`

```text
原因:
  simp-lite や探索が長すぎる

repair:
  - budget を下げた軽量版を使う
  - simp rule を制限する
  - 別 tactic を優先する
```

---

## 6.4 RepairGenerator

```rust
struct RepairGenerator {
    rule_based: RuleBasedRepair,
    model_based: Option<ModelRepair>,
    theorem_search: TheoremSearchClient,
}
```

repair の出力：

```json
{
  "repairs": [
    {
      "tactic": "exact Eq.symm h",
      "reason": "target is the symmetric form of h",
      "score": 0.83
    },
    {
      "tactic": "rw [h]",
      "reason": "h can rewrite the target",
      "score": 0.71
    }
  ]
}
```

---

## 6.5 Repair の制限

repair は無限に繰り返すと危険です。

制限：

```text
- 1 tactic あたり repair 最大 3 個
- 同じ tactic text は再試行しない
- 同じ error_kind が続いたら打ち切る
- repair depth 最大 2
- timeout error は同じ tactic で再試行しない
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

---

## 7.2 最小化の原則

重要：

```text
最小化後も必ず kernel / certificate checker で再検査する。
```

AIが「短くした」と言っても信用しません。

---

## 7.3 Minimization pass

### Pass 1: tactic deletion

各 tactic を消しても証明が通るか試します。

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

### Pass 2: block replacement

連続した tactic block を短い tactic に置き換えます。

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

長い証明項や tactic 列を、既存定理に置き換えます。

```text
by
  induction n
  ...
```

を：

```text
exact Nat.zero_add n
```

に置き換えられるなら置き換えます。

### Pass 4: premise simplification

不要な theorem reference を削ります。

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

### Pass 6: import minimization

不要な import を削ります。

```text
import Std.Nat
import Std.List
```

`Std.List` が不要なら削除。

---

## 7.4 Proof minimization algorithm

```python
def minimize_proof(proof_script, theorem):
    current = proof_script

    current = delete_redundant_steps(current, theorem)
    current = replace_blocks_with_simp(current, theorem)
    current = replace_with_exact_theorem(current, theorem)
    current = minimize_simp_arguments(current, theorem)
    current = shorten_names(current, theorem)
    current = minimize_imports(current, theorem)

    assert verify(current, theorem)
    return current
```

各 pass はこうします。

```text
candidate_script を作る
  ↓
elaborate
  ↓
kernel check
  ↓
certificate check
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

```json
POST /ai/minimize_proof
{
  "theorem": "Nat.add_zero",
  "proof_script": [
    "intro n",
    "rw [Nat.add_zero]",
    "exact Eq.refl n"
  ],
  "style": "readable",
  "require_same_axioms": true
}
```

レスポンス：

```json
{
  "status": "success",
  "minimized_script": [
    "exact Nat.add_zero n"
  ],
  "old_metrics": {
    "tactics": 3,
    "chars": 42
  },
  "new_metrics": {
    "tactics": 1,
    "chars": 20
  },
  "verified": true,
  "axioms_used": []
}
```

---

# 8. AI探索 API

## 8.1 `/ai/prove`

```json
POST /ai/prove
{
  "session_id": "sess_1",
  "declaration": "theorem t (n : Nat) : n + 0 = n := by _",
  "budget": {
    "timeout_ms": 30000,
    "max_nodes": 5000,
    "max_tactics_per_node": 24,
    "max_depth": 32
  },
  "trust": {
    "deny_sorry": true,
    "deny_custom_axioms": true,
    "allow_axioms": []
  },
  "search": {
    "strategy": "best_first",
    "use_model": true,
    "use_retrieval": true,
    "use_repair": true,
    "minimize": true
  }
}
```

レスポンス：

```json
{
  "status": "verified",
  "proof_script": "by\n  simp-lite",
  "certificate_hash": "sha256:...",
  "axioms_used": [],
  "search_stats": {
    "expanded_nodes": 9,
    "tried_tactics": 27,
    "successful_tactics": 4,
    "failed_tactics": 23,
    "repairs_tried": 3,
    "time_ms": 112
  }
}
```

---

## 8.2 `/ai/next_tactics`

現在の goal に対して tactic 候補だけ返します。

```json
POST /ai/next_tactics
{
  "state_id": "st_100",
  "goal_id": "g1",
  "limit": 10
}
```

レスポンス：

```json
{
  "candidates": [
    {
      "tactic": "simp-lite",
      "score": 0.94,
      "source": "builtin"
    },
    {
      "tactic": "exact Nat.add_zero n",
      "score": 0.91,
      "source": "retriever"
    },
    {
      "tactic": "rw [Nat.add_zero]",
      "score": 0.74,
      "source": "template"
    }
  ]
}
```

---

## 8.3 `/ai/search_step`

探索を1ステップだけ進めます。

```json
POST /ai/search_step
{
  "search_id": "search_1"
}
```

レスポンス：

```json
{
  "expanded_node": "node_14",
  "new_nodes": ["node_15", "node_16"],
  "best_node": "node_16",
  "status": "running"
}
```

これは IDE で探索過程を可視化する場合に便利です。

---

# 9. Training data

## 9.1 保存するデータ

Phase 7 では、成功・失敗の両方を学習データとして保存します。

```json
{
  "theorem": "Nat.add_zero",
  "state": {
    "context": [
      {"name": "n", "type": "Nat"}
    ],
    "target": "n + 0 = n"
  },
  "retrieved_premises": [
    "Nat.add_zero",
    "Eq.refl"
  ],
  "tactic_candidates": [
    {
      "tactic": "simp-lite",
      "result": "success"
    },
    {
      "tactic": "rw [Nat.zero_add]",
      "result": "error",
      "error_kind": "rewrite_not_found"
    }
  ],
  "chosen_tactic": "simp-lite",
  "new_state": "closed"
}
```

## 9.2 学習タスク

```text
premise selection:
  goal -> useful premises

tactic prediction:
  goal + premises -> next tactic

value prediction:
  proof state -> solvability score

error repair:
  failed tactic + error -> repaired tactic

proof minimization:
  long proof -> shorter proof
```

## 9.3 Positive / negative examples

正例：

```text
state, tactic で成功したもの
```

負例：

```text
state, tactic で失敗したもの
state, irrelevant premise
timeout tactic
loop tactic
```

負例が重要です。
「何をすべきか」だけでなく「何をすべきでないか」を学習できます。

---

# 10. Phase 7 MVP の実装順序

おすすめ順はこれです。

```text
1. Deterministic premise retrieval
   exact / apply / rw / simp の構造検索

2. Template tactic generation
   retrieved premise から exact/apply/rw/simp-lite を生成

3. Best-first search engine
   priority queue, visited set, budget, state hash

4. Tactic execution integration
   /tactic/run を transactional に呼ぶ

5. Rule-based error repair
   type_mismatch, rw failure, unsolved implicit など

6. Proof found -> kernel/certificate verification
   goals empty だけで成功にしない

7. Proof minimization
   deletion, replacement, simp-lite, exact theorem replacement

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
  exact/apply/rw/simp の構造検索

tactic generation:
  builtin + template

search:
  best-first

repair:
  rule-based

minimization:
  deletion + simp replacement

model:
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
retrieve Nat.add_zero
generate exact Nat.add_zero n
success
```

## 12.2 simp-lite

```npa
theorem t (n : Nat) : n + 0 = n := by
  _
```

期待：

```text
generate simp-lite
success
minimize to simp-lite
```

## 12.3 rw

```npa
theorem t (a b : Nat) (h : a = b) : a = a := by
  _
```

期待：

```text
generate rw [h]
then exact Eq.refl b
```

## 12.4 apply

```npa
theorem t {A : Type} (x y z : A)
  (h1 : x = y) (h2 : y = z) : x = z := by
  _
```

期待：

```text
apply Eq.trans
exact h1
exact h2
```

## 12.5 induction

```npa
theorem t (n : Nat) : 0 + n = n := by
  _
```

期待：

```text
induction n
base solved by simp-lite
step solved by simp-lite
```

## 12.6 List

```npa
theorem t {A : Type} (xs : List A) : xs ++ [] = xs := by
  _
```

期待：

```text
induction xs
simp-lite
simp-lite
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
- exact/apply/rw/simp 用の suggested tactic を作れる
- tactic候補を複数生成できる
- best-first search で proof state graph を探索できる
- tactic失敗時に構造化エラーを保存できる
- rule-based error repair が動く
- goals empty 後に kernel/certificate check できる
- 証明が見つかったら proof minimization できる
- search trace を学習データとして保存できる
- Std.Nat / Std.List の基本定理を自動証明できる
```

---

# 15. 一文でまとめると

Phase 7 は、**構造化 proof state と標準ライブラリを使って、AI・検索・tactic を組み合わせた検証駆動の証明探索エンジンを作る段階**です。

中核はこの流れです。

```text
retrieve useful premises
  ↓
generate tactic candidates
  ↓
try them transactionally
  ↓
repair failures
  ↓
search best-first
  ↓
kernel-check final proof
  ↓
minimize verified proof
```

最初はAIモデルなしの deterministic search で作り、あとから LLM・embedding retriever・value model を追加するのが安全です。
この順序なら、AIが間違えても証明器全体の信頼性は壊れません。

[1]: https://leandojo.readthedocs.io/?utm_source=chatgpt.com "LeanDojo: Machine Learning for Theorem Proving in Lean ..."
[2]: https://arxiv.org/abs/2306.15626?utm_source=chatgpt.com "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models"
[3]: https://www.nature.com/articles/s41586-025-09833-y?utm_source=chatgpt.com "Olympiad-level formal mathematical reasoning with ..."
