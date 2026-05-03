以下は **Phase 5: IDE/API** の詳細設計です。
Phase 4 までで、`intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction` のような tactic を使って proof term を構築できるようになりました。Phase 5 では、それを **IDE・Web UI・外部AIエージェント・証明探索器が扱える API** として公開します。

対象はこの4つです。

```text
- structured proof state
- tactic execution API
- theorem search API
- goal display
```

大事な方針はこれです。

```text
IDE/API層は便利な非信頼層。
APIが「成功」と言っても、それだけでは証明済みではない。
最終的に proof term / certificate を kernel が検査して初めて verified。
```

---

# 1. Phase 5 の目的

Phase 5 の目的は、証明器を「コマンドラインで一括検査するもの」から、**対話的に使える証明開発環境**にすることです。

Phase 4 まで：

```text
source
  ↓
elaboration
  ↓
tactic script
  ↓
proof term
  ↓
kernel check
```

Phase 5 では、途中状態を外部から見られるようにします。

```text
source document
  ↓
incremental parser / elaborator
  ↓
proof states
  ↓
IDE/API
  ↓
user / AI / theorem search / tactic execution
```

つまり、ユーザーやAIが次をできるようにします。

```text
- 現在のgoalを見る
- contextを見る
- targetを見る
- tactic候補を試す
- tactic実行結果を見る
- theoremを検索する
- エラーを構造化データとして受け取る
- 証明が完成したら certificate を生成する
```

---

# 2. 全体アーキテクチャ

Phase 5 では、証明器本体の上に **Proof Server** を置きます。

```text
┌──────────────────────────────────────┐
│ IDE / Web UI / CLI / AI Agent         │
└──────────────────────────────────────┘
                ↓ JSON-RPC / HTTP / LSP
┌──────────────────────────────────────┐
│ Proof Server                          │
│  - document manager                   │
│  - proof session manager              │
│  - goal state store                   │
│  - tactic executor                    │
│  - theorem search service             │
│  - pretty printer                     │
└──────────────────────────────────────┘
                ↓
┌──────────────────────────────────────┐
│ Elaborator / Tactic Engine / Kernel   │
└──────────────────────────────────────┘
                ↓
┌──────────────────────────────────────┐
│ Certificate Generator / Checker       │
└──────────────────────────────────────┘
```

Proof Server は trusted component ではありません。
間違った表示や間違った tactic 実行結果を返しても、最後の certificate check で弾ける設計にします。

---

# 3. Proof Session

IDE/API では、1つの証明開発単位を `ProofSession` として扱います。

```rust
struct ProofSession {
    session_id: SessionId,
    document_id: DocumentId,
    document_version: u64,
    environment: Env,
    elaboration_state: ElaborationState,
    proof_states: ProofStateStore,
    messages: Vec<Message>,
}
```

セッションは、たとえば次のように始まります。

```json
POST /sessions
{
  "module": "Scratch",
  "imports": ["Std.Nat.Basic"],
  "source": "theorem t (n : Nat) : n = n := by\n  _"
}
```

レスポンス：

```json
{
  "session_id": "sess_7f21",
  "document_id": "doc_91a0",
  "document_version": 1,
  "status": "open"
}
```

---

# 4. structured proof state

## 4.1 目的

文字列だけの goal 表示では、AIやIDEが扱いにくいです。

悪い例：

```text
n : Nat
⊢ n + 0 = n
```

これは人間には読みやすいですが、機械には不十分です。

望ましいのは、**人間向け表示と機械向け構造を両方持つ proof state** です。

```json
{
  "goal_id": "g1",
  "context": [...],
  "target": {...},
  "pretty": "n : Nat\n⊢ n + 0 = n"
}
```

## 4.2 ProofState の構造

```rust
struct StructuredProofState {
    state_id: StateId,
    session_id: SessionId,
    document_version: u64,
    goals: Vec<StructuredGoal>,
    selected_goal: Option<GoalId>,
    messages: Vec<Message>,
    proof_position: SourcePosition,
}
```

`state_id` は重要です。
tactic 実行 API は、この `state_id` に対して tactic を実行します。

## 4.3 Goal の構造

```rust
struct StructuredGoal {
    goal_id: GoalId,
    name: Option<NameId>,
    context: Vec<StructuredHypothesis>,
    target: StructuredExpr,
    target_core_hash: Hash,
    source_span: Option<Span>,
    mvar_id: MetaVarId,
    status: GoalStatus,
}
```

JSON 例：

```json
{
  "goal_id": "g1",
  "name": "?proof",
  "context": [
    {
      "id": "h1",
      "name": "n",
      "type": {
        "pretty": "Nat",
        "core_hash": "sha256:...",
        "head": "Nat"
      },
      "is_local_def": false,
      "is_implicit": false,
      "depends_on": []
    }
  ],
  "target": {
    "pretty": "n + 0 = n",
    "core_hash": "sha256:...",
    "head": "Eq",
    "free_locals": ["n"]
  },
  "status": "open"
}
```

## 4.4 Hypothesis の構造

```rust
struct StructuredHypothesis {
    local_id: LocalId,
    name: NameId,
    ty: StructuredExpr,
    value: Option<StructuredExpr>,
    is_local_def: bool,
    is_implicit: bool,
    depends_on: Vec<LocalId>,
}
```

例：

```json
{
  "name": "ih",
  "type": {
    "pretty": "0 + n = n",
    "core_hash": "sha256:..."
  },
  "is_local_def": false,
  "depends_on": ["n"]
}
```

`depends_on` は `induction` や `revert` で重要になります。

## 4.5 StructuredExpr

式は pretty string だけでなく、構造情報を持たせます。

```rust
struct StructuredExpr {
    pretty: String,
    core_hash: Hash,
    head: Option<NameId>,
    sort: Option<StructuredSort>,
    free_locals: Vec<LocalId>,
    constants: Vec<GlobalRef>,
    size: u32,
}
```

JSON 例：

```json
{
  "pretty": "n + 0 = n",
  "core_hash": "sha256:...",
  "head": "Eq",
  "constants": [
    "Eq",
    "Nat.add",
    "Nat.zero"
  ],
  "free_locals": ["n"],
  "size": 7
}
```

AI証明探索では、`head`, `constants`, `free_locals`, `core_hash` が重要です。

---

# 5. Proof state 取得 API

## 5.1 `/state/at`

ソースコード上の位置に対応する proof state を取得します。

```json
POST /state/at
{
  "session_id": "sess_7f21",
  "document_version": 3,
  "position": {
    "line": 3,
    "character": 4
  }
}
```

レスポンス：

```json
{
  "status": "ok",
  "state": {
    "state_id": "st_100",
    "goals": [
      {
        "goal_id": "g1",
        "context": [
          {
            "name": "n",
            "type": {
              "pretty": "Nat",
              "core_hash": "sha256:..."
            }
          }
        ],
        "target": {
          "pretty": "n = n",
          "core_hash": "sha256:..."
        }
      }
    ]
  }
}
```

## 5.2 `/state/current`

現在カーソル位置の state を返します。

```json
POST /state/current
{
  "session_id": "sess_7f21"
}
```

## 5.3 `/state/goals`

goal のみを軽量に返します。

```json
POST /state/goals
{
  "state_id": "st_100"
}
```

レスポンス：

```json
{
  "goals": [
    {
      "goal_id": "g1",
      "pretty": "n : Nat\n⊢ n = n"
    }
  ]
}
```

これは IDE の右ペイン更新用です。

---

# 6. tactic execution API

## 6.1 目的

外部から tactic を1つ実行できるようにします。

対象：

```text
- IDEでユーザーが tactic を試す
- AIが tactic 候補を試す
- 証明探索エンジンが並列に tactic を試す
- Web UIでワンクリック補完する
```

基本 API：

```json
POST /tactic/run
{
  "state_id": "st_100",
  "goal_id": "g1",
  "tactic": "exact Eq.refl n"
}
```

レスポンス：

```json
{
  "status": "success",
  "new_state_id": "st_101",
  "closed_goals": ["g1"],
  "new_goals": [],
  "proof_delta": {
    "kind": "assign",
    "goal_id": "g1",
    "term_hash": "sha256:..."
  },
  "messages": []
}
```

## 6.2 tactic 実行は transaction にする

tactic は失敗することが多いです。
失敗した場合、元の state を壊してはいけません。

```text
run tactic
  ↓
成功なら new_state を作る
失敗なら old_state はそのまま
```

つまり tactic execution は transactional です。

```rust
fn run_tactic_transactional(
    state: ProofState,
    goal: GoalId,
    tactic: TacticSyntax,
) -> Result<ProofState> {
    let mut cloned = state.clone();
    run_tactic(&mut cloned, goal, tactic)?;
    Ok(cloned)
}
```

## 6.3 timeout / budget

AIや探索器が tactic を大量に投げるため、budget が必要です。

```json
POST /tactic/run
{
  "state_id": "st_100",
  "goal_id": "g1",
  "tactic": "simp-lite",
  "budget": {
    "timeout_ms": 200,
    "max_rewrites": 100,
    "max_new_goals": 20
  }
}
```

失敗例：

```json
{
  "status": "error",
  "error_kind": "timeout",
  "message": "tactic exceeded 200ms budget"
}
```

## 6.4 tactic result の種類

```text
success:
  tacticが成功し、新しいstateができた

closed:
  tacticがgoalを閉じた

partial:
  tacticがgoalをsubgoalに分解した

error:
  tacticが失敗した

timeout:
  tacticが時間超過した

unsafe:
  tacticが許可されていない操作を要求した
```

JSON 例：

```json
{
  "status": "partial",
  "new_state_id": "st_120",
  "closed_goals": ["g1"],
  "new_goals": [
    {
      "goal_id": "g2",
      "pretty": "⊢ A"
    },
    {
      "goal_id": "g3",
      "pretty": "⊢ B"
    }
  ]
}
```

## 6.5 `intro` 実行例

リクエスト：

```json
{
  "state_id": "st_1",
  "goal_id": "g1",
  "tactic": "intro n"
}
```

元 goal：

```text
⊢ Nat → Nat
```

レスポンス：

```json
{
  "status": "partial",
  "new_state_id": "st_2",
  "closed_goals": ["g1"],
  "new_goals": [
    {
      "goal_id": "g2",
      "context": [
        {
          "name": "n",
          "type": {
            "pretty": "Nat"
          }
        }
      ],
      "target": {
        "pretty": "Nat"
      },
      "pretty": "n : Nat\n⊢ Nat"
    }
  ],
  "proof_delta": {
    "kind": "intro",
    "old_goal": "g1",
    "new_goal": "g2",
    "term_shape": "λ n : Nat, ?g2"
  }
}
```

## 6.6 `exact` 実行例

```json
{
  "state_id": "st_2",
  "goal_id": "g2",
  "tactic": "exact n"
}
```

レスポンス：

```json
{
  "status": "success",
  "new_state_id": "st_3",
  "closed_goals": ["g2"],
  "new_goals": [],
  "proof_delta": {
    "kind": "exact",
    "assigned": "n"
  }
}
```

## 6.7 `apply` 実行例

```json
{
  "state_id": "st_10",
  "goal_id": "g1",
  "tactic": "apply Eq.trans"
}
```

レスポンス：

```json
{
  "status": "partial",
  "new_state_id": "st_11",
  "closed_goals": ["g1"],
  "new_goals": [
    {
      "goal_id": "g2",
      "target": {
        "pretty": "x = ?m"
      }
    },
    {
      "goal_id": "g3",
      "target": {
        "pretty": "?m = z"
      }
    }
  ],
  "metavariables": [
    {
      "name": "?m",
      "type": "A",
      "status": "unsolved"
    }
  ]
}
```

## 6.8 エラー構造

tactic エラーは自然文だけでなく、機械可読にします。

```json
{
  "status": "error",
  "error_kind": "type_mismatch",
  "message": "term has type `Nat` but expected `n = n`",
  "expected": {
    "pretty": "n = n",
    "core_hash": "sha256:..."
  },
  "actual": {
    "pretty": "Nat",
    "core_hash": "sha256:..."
  },
  "span": {
    "line": 4,
    "character": 8
  },
  "suggestions": [
    {
      "kind": "try_tactic",
      "tactic": "exact Eq.refl n"
    }
  ]
}
```

AI修復には `error_kind`, `expected`, `actual`, `suggestions` が重要です。

---

# 7. tactic suggestion API

Phase 5 では、tactic の実行だけでなく、候補提示 API もあるとよいです。

```json
POST /tactic/suggest
{
  "state_id": "st_100",
  "goal_id": "g1",
  "max_results": 10,
  "include_search": true
}
```

レスポンス：

```json
{
  "suggestions": [
    {
      "tactic": "exact Eq.refl n",
      "source": "builtin",
      "confidence": 0.95,
      "reason": "target is reflexive equality"
    },
    {
      "tactic": "simp-lite",
      "source": "builtin",
      "confidence": 0.80,
      "reason": "target can likely be simplified to reflexive equality"
    }
  ]
}
```

この時点ではAIなしでも、簡単な builtin suggestion を作れます。

```text
- target が Pi なら intro
- target が Eq t t なら exact Eq.refl t
- context に target と同じ型の仮定があれば exact h
- target が Eq を含むなら rw 候補
- target が Nat を含むなら induction 候補
```

---

# 8. theorem search API

## 8.1 目的

証明中に「使えそうな定理」を検索する API です。

人間向けにもAI向けにも重要です。

検索方式は最初から複数用意します。

```text
- 名前検索
- 型検索
- target 類似検索
- rewrite rule 検索
- apply 候補検索
- exact 候補検索
```

## 8.2 `/search/name`

名前で検索します。

```json
POST /search/name
{
  "query": "add_zero",
  "limit": 10
}
```

レスポンス：

```json
{
  "results": [
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "module": "Std.Nat.Basic",
      "attributes": ["simp"],
      "decl_interface_hash": "sha256:..."
    }
  ]
}
```

## 8.3 `/search/by_type`

型パターンで検索します。

```json
POST /search/by_type
{
  "pattern": "?x + 0 = ?x",
  "context": [
    {
      "name": "n",
      "type": "Nat"
    }
  ],
  "limit": 10
}
```

レスポンス：

```json
{
  "results": [
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "match": {
        "?x": "n"
      },
      "suggested_tactic": "simpa using Nat.add_zero n"
    }
  ]
}
```

## 8.4 `/search/for_goal`

現在の goal に対して使えそうな定理を検索します。

```json
POST /search/for_goal
{
  "state_id": "st_100",
  "goal_id": "g1",
  "limit": 20,
  "modes": ["exact", "apply", "rw", "simp"]
}
```

レスポンス：

```json
{
  "goal": {
    "pretty": "n + 0 = n"
  },
  "results": [
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "score": 0.99,
      "mode": "exact",
      "suggested_tactic": "exact Nat.add_zero n",
      "reason": "the theorem conclusion matches the target"
    },
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "score": 0.95,
      "mode": "rw",
      "suggested_tactic": "rw [Nat.add_zero]",
      "reason": "the theorem is a rewrite rule matching a subterm"
    },
    {
      "name": "Eq.refl",
      "statement": "∀ {A} (x : A), x = x",
      "score": 0.82,
      "mode": "exact",
      "suggested_tactic": "exact Eq.refl n",
      "reason": "target may simplify to reflexive equality"
    }
  ]
}
```

## 8.5 検索インデックス

Theorem search のために、各宣言に metadata を持たせます。

```rust
struct TheoremIndexEntry {
    global_ref: GlobalRef,
    name: NameId,
    module: ModuleName,
    statement: ExprId,
    statement_pretty: String,
    head_symbol: Option<NameId>,
    constants: Vec<GlobalRef>,
    attributes: Vec<Attribute>,
    theorem_kind: TheoremKind,
    rewrite_info: Option<RewriteInfo>,
    dependencies: Vec<GlobalRef>,
    axiom_deps: Vec<GlobalRef>,
}
```

`TheoremKind`：

```text
- theorem
- def
- axiom
- constructor
- recursor
- rewrite_rule
- simp_rule
```

## 8.6 検索モード

### exact search

target と theorem の結論が一致するかを調べます。

```text
goal:
  ⊢ n + 0 = n

candidate:
  Nat.add_zero : ∀ n, n + 0 = n

suggest:
  exact Nat.add_zero n
```

### apply search

theorem の結論を target に unify できるか調べます。

```text
candidate:
  Eq.trans : x = y → y = z → x = z

goal:
  ⊢ a = c

suggest:
  apply Eq.trans
```

### rw search

target 内の subterm に rewrite rule の lhs/rhs が一致するか調べます。

```text
rule:
  Nat.add_zero : x + 0 = x

target:
  f (n + 0) = f n

suggest:
  rw [Nat.add_zero]
```

### simp search

`simp-lite` に登録されている rule を返します。

```json
{
  "name": "Nat.add_zero",
  "mode": "simp",
  "attribute": "simp",
  "orientation": "left_to_right"
}
```

## 8.7 ランキング

検索結果は ranking が重要です。

スコア例：

```text
score =
  + exact_match_score
  + type_unification_score
  + head_symbol_match
  + name_similarity
  + attribute_bonus
  + local_context_bonus
  + usage_frequency_bonus
  - axiom_penalty
  - theorem_complexity_penalty
```

高信頼モードでは、axiomを使う定理に penalty をかけます。

```text
constructive theorem:
  score unchanged

uses Classical.choice:
  score - 0.2

custom axiom:
  score - 1.0 or excluded
```

## 8.8 theorem search のレスポンスは tactic 付きにする

単に定理名を返すだけでは不十分です。

悪いレスポンス：

```json
{
  "name": "Nat.add_zero"
}
```

良いレスポンス：

```json
{
  "name": "Nat.add_zero",
  "statement": "∀ n : Nat, n + 0 = n",
  "suggested_tactic": "exact Nat.add_zero n",
  "why": "after instantiating n, the theorem matches the target",
  "match": {
    "n": "n"
  }
}
```

これにより、IDEもAIもそのまま tactic execution API に渡せます。

---

# 9. goal display

## 9.1 目的

goal display は、人間が見る UI の中心です。
しかし、ただ pretty print するだけでは不十分です。

必要な機能：

```text
- context を見やすく表示する
- target を表示する
- implicit arguments を必要に応じて隠す/表示する
- notation を使って表示する
- core expression も確認できる
- goal の変化を差分表示する
- multiple goals を整理して表示する
```

## 9.2 基本表示

```text
1 goal

n : Nat
⊢ n + 0 = n
```

複数 goal：

```text
2 goals

case zero
⊢ 0 + 0 = 0

case succ
n : Nat
ih : 0 + n = n
⊢ 0 + succ n = succ n
```

## 9.3 表示モード

IDE/API は複数の表示モードを持たせます。

```text
pretty:
  人間向け。notationあり。implicit省略。

explicit:
  implicit arguments を表示。

core:
  kernelが見るcore termを表示。

json:
  機械向け構造化表現。
```

API 例：

```json
POST /display/goal
{
  "state_id": "st_100",
  "goal_id": "g1",
  "mode": "pretty"
}
```

レスポンス：

```json
{
  "text": "n : Nat\n⊢ n + 0 = n"
}
```

explicit mode：

```json
{
  "text": "n : Nat\n⊢ Eq Nat (Nat.add n Nat.zero) n"
}
```

core mode：

```json
{
  "text": "Eq.{0} Nat (Nat.add n Nat.zero) n"
}
```

## 9.4 goal diff

tactic 実行後に、goal がどう変わったかを表示します。

例：

```text
before:
  ⊢ n + 0 = n

after simp-lite:
  closed
```

または：

```text
before:
  ⊢ A → B

after intro h:
  h : A
  ⊢ B
```

API：

```json
POST /display/diff
{
  "before_state_id": "st_100",
  "after_state_id": "st_101"
}
```

レスポンス：

```json
{
  "diff": [
    {
      "kind": "goal_replaced",
      "old_goal": "g1",
      "new_goals": ["g2"],
      "summary": "`intro n` introduced `n : Nat`"
    }
  ]
}
```

## 9.5 hiding / folding

大きな context では、全部表示すると読みにくいです。

表示オプション：

```json
{
  "show_implicit": false,
  "show_local_defs": "folded",
  "show_instances": false,
  "max_context_items": 30,
  "fold_large_terms": true
}
```

表示例：

```text
Γ contains 48 hypotheses. Showing 12 relevant hypotheses.

n : Nat
ih : 0 + n = n
...
⊢ 0 + succ n = succ n
```

AI向けには folding しない構造データを返します。

## 9.6 relevant context

goal に関係する仮定だけを上に出します。

```text
relevant:
  n : Nat
  ih : 0 + n = n

less relevant:
  A : Type
  h_unused : ...
```

関連度は簡単には：

```text
- target に出現する local
- target に出現する local に依存する仮定
- 型の head symbol が target と近い仮定
- exact/apply/rw で使えそうな仮定
```

で計算できます。

---

# 10. document / incremental checking

IDEではユーザーが1文字ずつ編集します。
毎回全ファイルを再検査すると遅いです。

## 10.1 document snapshot

```rust
struct DocumentSnapshot {
    document_id: DocumentId,
    version: u64,
    text: Arc<String>,
    parsed: Option<ParsedModule>,
    elaborated_prefix: Option<ElaboratedPrefix>,
}
```

## 10.2 incremental pipeline

```text
edit
  ↓
new document version
  ↓
parse affected region
  ↓
reuse unchanged declarations
  ↓
re-elaborate changed declarations
  ↓
update proof states
  ↓
push diagnostics/goals to IDE
```

## 10.3 declaration-level cache

各 declaration に hash を持たせます。

```text
source_decl_hash
  ↓
resolved_decl_hash
  ↓
core_decl_hash
  ↓
certificate_decl_hash
```

同じ declaration は再利用できます。

---

# 11. LSP integration

IDEは VS Code などと連携するなら LSP 互換がよいです。

対応したい機能：

```text
- diagnostics
- hover
- go to definition
- completion
- code actions
- semantic tokens
- document symbols
- inlay hints
- custom goal view
```

## 11.1 diagnostics

エラー例：

```text
type mismatch
expected: Nat
actual:   Prop
```

LSP diagnostic：

```json
{
  "range": {
    "start": {"line": 3, "character": 8},
    "end": {"line": 3, "character": 15}
  },
  "severity": 1,
  "source": "npa",
  "message": "type mismatch: expected `Nat`, got `Prop`"
}
```

## 11.2 hover

カーソルを `Nat.add_zero` に置くと：

```text
Nat.add_zero : ∀ n : Nat, n + 0 = n

attributes:
  simp

axioms:
  none
```

## 11.3 code actions

goal に対して候補 tactic を提案します。

```text
Apply tactic: exact Eq.refl n
Apply tactic: simp-lite
Search theorem for goal
```

---

# 12. AI向け API

Phase 5 の structured API は、AI証明探索の土台になります。

AIに渡す payload 例：

```json
{
  "state_id": "st_100",
  "goal": {
    "context": [
      {
        "name": "n",
        "type": "Nat"
      }
    ],
    "target": "n + 0 = n",
    "target_core": {
      "head": "Eq",
      "constants": ["Eq", "Nat.add", "Nat.zero"]
    }
  },
  "available_tactics": [
    "intro",
    "exact",
    "apply",
    "rw",
    "simp-lite",
    "induction"
  ],
  "nearby_theorems": [
    {
      "name": "Nat.add_zero",
      "statement": "∀ n : Nat, n + 0 = n",
      "suggested_tactic": "exact Nat.add_zero n"
    }
  ],
  "failed_tactics": [
    {
      "tactic": "rw [Nat.zero_add]",
      "error_kind": "rewrite_not_found"
    }
  ]
}
```

AIの出力：

```json
{
  "candidates": [
    {
      "tactic": "simp-lite",
      "confidence": 0.92
    },
    {
      "tactic": "exact Nat.add_zero n",
      "confidence": 0.88
    }
  ]
}
```

この出力を `/tactic/run` に渡して、通ったものだけ採用します。

---

# 13. セキュリティと信頼境界

IDE/API は外部入力を受けるので、制限が必要です。

## 13.1 tactic execution の制限

```json
{
  "budget": {
    "timeout_ms": 500,
    "max_memory_mb": 64,
    "max_new_goals": 100,
    "max_term_size": 100000
  }
}
```

## 13.2 API がしてはいけないこと

```text
- unchecked theorem を verified と返す
- unresolved goal を certificate 化する
- import hash を無視する
- axiom report を省略する
- tactic 失敗後に state を破壊する
- AI出力を自動で trusted にする
```

## 13.3 完成判定

proof state の goals が空でも、それだけでは不十分です。

最終的に：

```text
goals empty
  ↓
root proof term extracted
  ↓
kernel check
  ↓
certificate generated
  ↓
certificate checker passes
```

まで通って初めて成功です。

API：

```json
POST /session/verify
{
  "session_id": "sess_7f21",
  "declaration": "t"
}
```

レスポンス：

```json
{
  "status": "verified",
  "certificate_hash": "sha256:...",
  "axioms_used": [],
  "contains_sorry": false
}
```

---

# 14. Phase 5 の最小 API 一覧

## Session

```text
POST /sessions
POST /documents/update
POST /session/verify
DELETE /sessions/{id}
```

## State

```text
POST /state/at
POST /state/current
POST /state/goals
POST /state/by_id
```

## Tactic

```text
POST /tactic/run
POST /tactic/suggest
POST /tactic/check
```

## Search

```text
POST /search/name
POST /search/by_type
POST /search/for_goal
POST /search/rewrite
```

## Display

```text
POST /display/goal
POST /display/expr
POST /display/diff
POST /display/context
```

---

# 15. 実装順序

おすすめの順番はこれです。

```text
1. ProofStateStore
   state_id, goal_id, metavariable, context を安定管理する

2. StructuredGoal
   context / target / core_hash / pretty を返せるようにする

3. Pretty printer
   core term → human-readable display

4. /state/goals
   現在goalをAPIで取得できるようにする

5. /tactic/run
   Phase 4 tacticを1つずつAPIから実行できるようにする

6. Transactional state update
   tactic失敗時に元stateを壊さない

7. /display/goal
   pretty / explicit / core モードを作る

8. theorem index
   宣言名・型・属性をindex化する

9. /search/name
   名前検索

10. /search/for_goal
   exact/apply/rw/simp候補を返す

11. LSP連携
   diagnostics, hover, goal view

12. tactic suggestions
   builtin候補を返す

13. AI用payload
   structured goal + theorem search + failed tactics をまとめる
```

---

# 16. Phase 5 のテスト例

## 16.1 goal取得

入力：

```npa
theorem t (n : Nat) : n = n := by
  _
```

期待：

```json
{
  "goals": [
    {
      "context": [{"name": "n", "type": "Nat"}],
      "target": "n = n"
    }
  ]
}
```

## 16.2 tactic実行

```json
{
  "tactic": "exact Eq.refl n"
}
```

期待：

```json
{
  "status": "success",
  "new_goals": []
}
```

## 16.3 theorem search

goal：

```text
n + 0 = n
```

検索結果：

```json
{
  "name": "Nat.add_zero",
  "suggested_tactic": "exact Nat.add_zero n"
}
```

## 16.4 display mode

pretty：

```text
n : Nat
⊢ n + 0 = n
```

explicit：

```text
n : Nat
⊢ Eq Nat (Nat.add n Nat.zero) n
```

core：

```text
Eq.{0} Nat (Nat.add n Nat.zero) n
```

## 16.5 tactic failure

```json
{
  "tactic": "intro h"
}
```

goal：

```text
⊢ n = n
```

期待：

```json
{
  "status": "error",
  "error_kind": "expected_pi_type",
  "message": "`intro` can only be used on a function or forall target."
}
```

---

# 17. Phase 5 でまだ入れないもの

Phase 5 MVPでは、以下は後回しでよいです。

```text
- full semantic search embedding
- LLM統合
- collaborative editing
- proof replay visualization
- full tactic trace UI
- large-scale dependency graph UI
- proof minimization
- theorem recommendation model
- natural language formalization UI
```

まずは、**構造化された goal を取得し、tactic を実行し、定理検索できる API** を完成させるのが優先です。

---

# 18. Phase 5 の完了条件

Phase 5 が完了したと言える条件はこれです。

```text
- source位置から current proof state を取得できる
- proof state が人間向け表示と機械向け構造を両方持つ
- goal_id / state_id によって tactic を実行できる
- tactic 実行が transactional である
- tactic 成功時に新しい state が返る
- tactic 失敗時に構造化エラーが返る
- theorem search が名前・型・goalに対して動く
- theorem search が suggested_tactic を返す
- goal display に pretty / explicit / core モードがある
- unresolved goals がある場合、verify/certificate 化を拒否できる
- goals が空になった後、kernel check と certificate generation に接続できる
```

---

# 19. 一文でまとめると

Phase 5 は、**証明器の内部状態を、人間・IDE・AI・探索エンジンが安全に扱える構造化APIとして公開する段階**です。

中核はこの4つです。

```text
structured proof state:
  goal/context/targetを機械可読にする

tactic execution API:
  tacticをtransactionalに実行し、新しいstateを返す

theorem search API:
  goalに使える定理とsuggested tacticを返す

goal display:
  pretty / explicit / core 表示を切り替えられるようにする
```

この Phase 5 が完成すると、次の Phase 6 以降で **AI証明探索、RAG、proof search、IDE補完、教育UI** を本格的に載せられるようになります。

