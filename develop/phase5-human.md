The following is the detailed design for **Phase 5 Human Profile: IDE/API**.

This document organizes Phase 5 as a design for human-facing IDE / Web UI / CLI
use. Structured goals / search results can also be used by helper features in a
Human UI, but the deterministic, transactional machine API contract for AI proof
searchers is separated into `develop/phase5-ai.md`.

Through Phase 4, proof terms can be constructed using tactics such as `intro`,
`exact`, `apply`, `rw`, `simp-lite`, and `induction`. Phase 5 Human exposes that
as an **API usable by IDEs, Web UIs, CLIs, and Human API clients**.

The targets are these four items.

```text
- structured proof state
- tactic execution API
- theorem search API
- goal display
```

The important policy is:

```text
The IDE/API layer is a convenient untrusted layer.
Even if an API says "success", that alone does not mean proved.
It is verified only after the kernel finally checks the proof term / certificate.
```

---

# 1. Purpose Of Phase 5

The purpose of Phase 5 is to turn the prover from "something checked in batch on
the command line" into an **interactive proof development environment**.

Through Phase 4:

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

In Phase 5, intermediate states become externally visible.

```text
source document
  ↓
incremental parser / elaborator
  ↓
proof states
  ↓
IDE/API
  ↓
user / IDE action / theorem search / tactic execution
```

That is, users and Human API clients can:

```text
- view the current goal
- view the context
- view the target
- try tactic candidates
- view tactic execution results
- search theorems
- receive errors as structured data
- generate a certificate when the proof is complete
```

---

# 2. Overall Architecture

In Phase 5, place a **Proof Server** on top of the prover itself.

```text
┌──────────────────────────────────────┐
│ IDE / Web UI / CLI / Human API Client │
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

The Proof Server is not a trusted component. The design rejects wrong displays
or wrong tactic execution results at the final certificate check.

---

# 3. Proof Session

In IDE/API usage, one proof development unit is treated as a `ProofSession`.

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

A session starts, for example, like this.

```json
POST /sessions
{
  "module": "Scratch",
  "imports": ["Std.Nat.Basic"],
  "source": "theorem t (n : Nat) : n = n := by\n  _"
}
```

Response:

```json
{
  "session_id": "sess_7f21",
  "document_id": "doc_91a0",
  "document_version": 1,
  "status": "open"
}
```

---

# 4. Structured Proof State

## 4.1 Purpose

Goal display as strings alone is hard for AI and IDEs to handle.

In `pretty` / `statement` / goal displays in this Phase 5 document, `0` may be
used for readability. This is display shorthand; machine-facing structures use
canonical `Const` references to `Nat.zero` and `core_hash`.

Bad example:

```text
n : Nat
⊢ n + 0 = n
```

This is readable for humans, but insufficient for machines.

What is desirable is a **proof state with both human-facing display and
machine-facing structure**.

```json
{
  "goal_id": "g1",
  "context": [...],
  "target": {...},
  "pretty": "n : Nat\n⊢ n + 0 = n"
}
```

## 4.2 ProofState Structure

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

`state_id` is important. The tactic execution API runs tactics against this
`state_id`.

## 4.3 Goal Structure

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

JSON example:

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

## 4.4 Hypothesis Structure

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

Example:

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

`depends_on` is important for `induction` and `revert`.

## 4.5 StructuredExpr

Expressions carry structural information, not only pretty strings.

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

JSON example:

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

For AI proof search, `head`, `constants`, `free_locals`, and `core_hash` are
important.

---

# 5. Proof State Retrieval API

## 5.1 `/state/at`

Retrieve the proof state corresponding to a source code position.

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

Response:

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

Return the state at the current cursor position.

```json
POST /state/current
{
  "session_id": "sess_7f21"
}
```

## 5.3 `/state/goals`

Return only goals in a lightweight form.

```json
POST /state/goals
{
  "state_id": "st_100"
}
```

Response:

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

This is for updating the right pane of an IDE.

---

# 6. Tactic Execution API

## 6.1 Purpose

Allow one tactic to be executed externally.

Use cases:

```text
- a user tries a tactic in an IDE
- helper features in a Human UI try tactic candidates
- a Human API client interactively tries small candidates
- a Web UI performs one-click completion
```

The path where an AI proof searcher deterministically tries many candidates is
not this `/tactic/run`, but `/machine/tactics/run` / `/machine/tactics/batch` in
`develop/phase5-ai.md`.

Basic API:

```json
POST /tactic/run
{
  "state_id": "st_100",
  "goal_id": "g1",
  "tactic": "exact Eq.refl n"
}
```

Response:

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

## 6.2 Make Tactic Execution Transactional

Tactics often fail. If they fail, the original state must not be mutated.

```text
run tactic
  ↓
on success, create new_state
on failure, keep old_state unchanged
```

That is, tactic execution is transactional.

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

External clients and UI helpers may try tactics repeatedly, so a budget is
needed.

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

Failure example:

```json
{
  "status": "error",
  "error_kind": "timeout",
  "message": "tactic exceeded 200ms budget"
}
```

## 6.4 Tactic Result Kinds

```text
success:
  the tactic succeeded and a new state was created

closed:
  the tactic closed the goal

partial:
  the tactic split the goal into subgoals

error:
  the tactic failed

timeout:
  the tactic timed out

unsafe:
  the tactic requested an operation that is not allowed
```

JSON example:

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

## 6.5 `intro` Execution Example

Request:

```json
{
  "state_id": "st_1",
  "goal_id": "g1",
  "tactic": "intro n"
}
```

Original goal:

```text
⊢ Nat → Nat
```

Response:

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

## 6.6 `exact` Execution Example

```json
{
  "state_id": "st_2",
  "goal_id": "g2",
  "tactic": "exact n"
}
```

Response:

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

## 6.7 `apply` Execution Example

```json
{
  "state_id": "st_10",
  "goal_id": "g1",
  "tactic": "apply Eq.trans"
}
```

Response:

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

## 6.8 Error Structure

Tactic errors are machine-readable, not only natural-language text.

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

For AI repair, `error_kind`, `expected`, `actual`, and `suggestions` are
important.

---

# 7. Tactic Suggestion API

In Phase 5, it is useful to have not only tactic execution but also a candidate
suggestion API.

```json
POST /tactic/suggest
{
  "state_id": "st_100",
  "goal_id": "g1",
  "max_results": 10,
  "include_search": true
}
```

Response:

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

At this point, simple builtin suggestions can be produced even without AI.

```text
- if the target is Pi, intro
- if the target is Eq t t, exact Eq.refl t
- if the context has a hypothesis with the same type as the target, exact h
- if the target contains Eq, rw candidates
- if the target contains Nat, induction candidates
```

---

# 8. Theorem Search API

## 8.1 Purpose

This API searches for "theorems that look useful" during proof development.

It is also important for aligning human-facing IDE/API design with the verified
retrieval used by the Machine API.

Provide multiple search methods from the start.

```text
- name search
- type search
- target similarity search
- rewrite rule search
- apply candidate search
- exact candidate search
```

## 8.2 `/search/name`

Search by name.

```json
POST /search/name
{
  "query": "add_zero",
  "limit": 10
}
```

Response:

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

Search by type pattern.

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

Response:

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

Search for theorems that look useful for the current goal.

```json
POST /search/for_goal
{
  "state_id": "st_100",
  "goal_id": "g1",
  "limit": 20,
  "modes": ["exact", "apply", "rw", "simp"]
}
```

Response:

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

## 8.5 Search Index

For theorem search, each declaration carries metadata.

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

## 8.6 Search Modes

### exact search

Check whether the target matches the theorem conclusion.

```text
goal:
  ⊢ n + 0 = n

candidate:
  Nat.add_zero : ∀ n, n + 0 = n

suggest:
  exact Nat.add_zero n
```

### apply search

Check whether the theorem conclusion can be unified with the target.

```text
candidate:
  Eq.trans : x = y → y = z → x = z

goal:
  ⊢ a = c

suggest:
  apply Eq.trans
```

### rw search

Check whether the lhs/rhs of a rewrite rule matches a subterm in the target.

```text
rule:
  Nat.add_zero : x + 0 = x

target:
  f (n + 0) = f n

suggest:
  rw [Nat.add_zero]
```

### simp search

Return rules registered with `simp-lite`.

```json
{
  "name": "Nat.add_zero",
  "mode": "simp",
  "attribute": "simp",
  "orientation": "left_to_right"
}
```

## 8.7 Ranking

Ranking search results is important.

Example score:

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

In high-trust mode, penalize theorems that use axioms.

```text
constructive theorem:
  score unchanged

uses Classical.choice:
  score - 0.2

custom axiom:
  score - 1.0 or excluded
```

## 8.8 Theorem Search Responses Include Tactics

Returning only theorem names is not enough.

Bad response:

```json
{
  "name": "Nat.add_zero"
}
```

Good response:

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

This lets the IDE pass the result directly to the Human tactic execution API.
For AI searchers, separately fix the contract that converts the same search
result into `MachineTacticCandidate` in `develop/phase5-ai.md`.

---

# 9. Goal Display

## 9.1 Purpose

Goal display is central to the UI humans see. However, pretty printing alone is
not enough.

Needed features:

```text
- display context readably
- display the target
- hide/show implicit arguments as needed
- display using notation
- allow checking the core expression too
- show goal changes as diffs
- organize multiple goals for display
```

## 9.2 Basic Display

```text
1 goal

n : Nat
⊢ n + 0 = n
```

Multiple goals:

```text
2 goals

case zero
⊢ 0 + 0 = 0

case succ
n : Nat
ih : 0 + n = n
⊢ 0 + succ n = succ n
```

## 9.3 Display Modes

IDE/API should have multiple display modes.

```text
pretty:
  for humans. Uses notation. Omits implicits.

explicit:
  displays implicit arguments.

core:
  displays the core term seen by the kernel.

json:
  machine-facing structured representation.
```

API example:

```json
POST /display/goal
{
  "state_id": "st_100",
  "goal_id": "g1",
  "mode": "pretty"
}
```

Response:

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

## 9.4 Goal Diff

Show how goals changed after tactic execution.

Example:

```text
before:
  ⊢ n + 0 = n

after simp-lite:
  closed
```

Or:

```text
before:
  ⊢ A → B

after intro h:
  h : A
  ⊢ B
```

API:

```json
POST /display/diff
{
  "before_state_id": "st_100",
  "after_state_id": "st_101"
}
```

Response:

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

## 9.5 Hiding / Folding

In large contexts, displaying everything is hard to read.

Display options:

```json
{
  "show_implicit": false,
  "show_local_defs": "folded",
  "show_instances": false,
  "max_context_items": 30,
  "fold_large_terms": true
}
```

Display example:

```text
Γ contains 48 hypotheses. Showing 12 relevant hypotheses.

n : Nat
ih : 0 + n = n
...
⊢ 0 + succ n = succ n
```

For the Machine API, return structured data without folding.

## 9.6 Relevant Context

Move only hypotheses relevant to the goal to the top.

```text
relevant:
  n : Nat
  ih : 0 + n = n

less relevant:
  A : Type
  h_unused : ...
```

Relevance can be computed simply by:

```text
- locals appearing in the target
- hypotheses depending on locals appearing in the target
- hypotheses whose type head symbol is close to the target
- hypotheses that look usable by exact/apply/rw
```

.

---

# 10. Document / Incremental Checking

In an IDE, users edit one character at a time. Rechecking the whole file every
time is slow.

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

## 10.3 Declaration-level Cache

Give each declaration hashes.

```text
source_decl_hash
  ↓
resolved_decl_hash
  ↓
core_decl_hash
  ↓
decl_interface_hash / decl_certificate_hash
```

`source_decl_hash` through `core_decl_hash` are cache keys for the proof server.
Declaration hashes on the trusted certificate side are Phase 2
`decl_interface_hash` / `decl_certificate_hash`.

The same declaration can be reused.

---

# 11. LSP integration

If the IDE integrates with VS Code or similar tools, LSP compatibility is useful.

Features to support:

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

Error example:

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

When the cursor is placed on `Nat.add_zero`:

```text
Nat.add_zero : ∀ n : Nat, n + 0 = n

attributes:
  simp

axioms:
  none
```

## 11.3 code actions

Suggest candidate tactics for goals.

```text
Apply tactic: exact Eq.refl n
Apply tactic: simp-lite
Search theorem for goal
```

---

# 12. AI Assistance / Connection To Machine API

The Human structured API can also be used for helper display and small candidate
suggestions in the IDE. However, it is not the canonical wire contract for AI
proof search. Search node identity, batch execution, replay, and verify are
fixed to `session_id` / `snapshot_id` / `state_fingerprint` /
`MachineTacticCandidate` in `develop/phase5-ai.md`.

Example display payload that may be passed from the Human UI to a helper
service:

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

Example helper service output:

```json
{
  "candidates": [
    {
      "tactic": "simp-lite",
      "confidence": 92,
      "reason": "target can likely be simplified to reflexive equality"
    },
    {
      "tactic": "exact Nat.add_zero n",
      "confidence": 88,
      "reason": "nearby theorem matches the current goal"
    }
  ]
}
```

One assistant output candidate consists only of a Human tactic string,
confidence, and reason. This confidence / reason is optional metadata for UI
ranking and explanation, and must not enter certificates, replay plans, Machine
tactic cache keys, or Machine prompt payload fingerprints.

The optional assistant payload of the Human API is a Human UI response bundling
`state_id`, `goal_summary`, `structured_goal`, `available_tactics`,
`tactic_suggestions`, `nearby_theorems`, and `failed_tactics`. `failed_tactics`
returns only diagnostics from running the same execution path as existing
`/tactic/run` on a scratch store for display; it does not update the original
Human session state.

In the Human UI, before adopting assistant output as a candidate, always pass it
through the same checking path as `/tactic/run`, and adopt only tactic strings
that pass as UI candidates. This check alone does not adopt the proof state.
When actually advancing the proof, run the same tactic string again against the
current session with `/tactic/run`.

When an AI proof searcher executes it as part of proof search, convert it not to
a text tactic but to `MachineTacticCandidate` for `/machine/tactics/run` or
`/machine/tactics/batch`, and treat only results that pass through
`/machine/replay` and `/machine/verify` as proofs. Deterministic payloads for AI
proof searchers use `/machine/prompt_payload` in `develop/phase5-ai.md`.
Assistant payload is not part of the Phase 7 MVP required path, and does not
change the schema of Machine `/machine/prompt_payload`,
`PromptRenderedContent` canonical bytes, or `payload_fingerprint`.

---

# 13. Security And Trust Boundary

IDE/API receives external input, so limits are needed.

## 13.1 Tactic Execution Limits

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

## 13.2 Things The API Must Not Do

```text
- return unchecked theorems as verified
- turn unresolved goals into certificates
- ignore import export_hash / high-trust certificate_hash
- omit axiom reports
- mutate state after tactic failure
- automatically trust AI output or helper service candidates
```

## 13.3 Completion Decision

Even if proof state goals are empty, that alone is insufficient.

Finally:

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

must pass before it is considered successful.

API：

```json
POST /session/verify
{
  "session_id": "sess_7f21",
  "declaration": "t"
}
```

Response:

```json
{
  "status": "verified",
  "certificate_hash": "sha256:...",
  "axioms_used": [],
  "contains_sorry": false
}
```

---

# 14. Minimal Phase 5 API List

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

# 15. Implementation Order

Recommended order:

```text
1. ProofStateStore
   manage state_id, goal_id, metavariable, and context stably

2. StructuredGoal
   return context / target / core_hash / pretty

3. Pretty printer
   core term -> human-readable display

4. /state/goals
   make the current goal retrievable by API

5. /tactic/run
   make Phase 4 tactics executable one by one from the API

6. Transactional state update
   do not mutate the original state when tactics fail

7. /display/goal
   create pretty / explicit / core modes

8. theorem index
   index declaration names, types, and attributes

9. /search/name
   name search

10. /search/for_goal
   return exact/apply/rw/simp candidates

11. LSP integration
   diagnostics, hover, goal view

12. tactic suggestions
   return builtin candidates

13. optional assistant payload
   bundle structured goal + theorem search + failed tactics for Human UI.
   Deterministic payloads for AI searchers are implemented on the `develop/phase5-ai.md` side.

14. integration regression
   pass session create, state lookup, tactic run, search, display, and verify
   Fix the Phase 5 Human end-to-end fixture.
   Even after Human IDE integration, Phase 7 MVP continues using `MachineApiClient`,
   `MachineProofSnapshot`, raw `MachineTacticCandidate`, `/machine/tactics/batch`,
   `/machine/replay`, and `/machine/verify` as the required path.
```

---

# 16. Phase 5 Test Examples

## 16.1 Goal Retrieval

Input:

```npa
theorem t (n : Nat) : n = n := by
  _
```

Expected:

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

## 16.2 Tactic Execution

```json
{
  "tactic": "exact Eq.refl n"
}
```

Expected:

```json
{
  "status": "success",
  "new_goals": []
}
```

## 16.3 theorem search

Goal:

```text
n + 0 = n
```

Search result:

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

## 16.5 Tactic Failure

```json
{
  "tactic": "intro h"
}
```

Goal:

```text
⊢ n = n
```

Expected:

```json
{
  "status": "error",
  "error_kind": "expected_pi_type",
  "message": "`intro` can only be used on a function or forall target."
}
```

---

# 17. Things Not Yet Included In Phase 5

For the Phase 5 MVP, the following can be deferred.

```text
- full semantic search embedding
- LLM integration
- collaborative editing
- proof replay visualization
- full tactic trace UI
- large-scale dependency graph UI
- proof minimization
- theorem recommendation model
- natural language formalization UI
```

The priority is first to complete an **API that can retrieve structured goals,
execute tactics, and search theorems**.

---

# 18. Phase 5 Completion Criteria

Phase 5 can be called complete when:

```text
- current proof state can be retrieved from source position
- proof states have both human-facing display and machine-facing structure
- tactics can be executed by goal_id / state_id
- tactic execution is transactional
- a new state is returned on tactic success
- structured errors are returned on tactic failure
- theorem search works by name, type, and goal
- theorem search returns suggested_tactic
- goal display has pretty / explicit / core modes
- verify/certificate generation can be rejected when unresolved goals remain
- after goals become empty, the flow can connect to kernel check and certificate generation
- the end-to-end fixture for Human session create -> state lookup -> tactic run -> theorem search -> goal display -> verify passes
- adding Human UI / LSP / assistant payload does not change Machine `/machine/*` endpoint schemas, Phase 7 candidate hashes, or Machine state identity
```

---

# 19. In One Sentence

Phase 5 Human is the stage that **exposes the prover's internal state as a
structured API that humans, IDEs, and Human API clients can handle safely**.

The core consists of these four items.

```text
structured proof state:
  make goal/context/target machine-readable

tactic execution API:
  execute tactics transactionally and return new states

theorem search API:
  return theorems usable for the goal and suggested tactics

goal display:
  allow switching between pretty / explicit / core display
```

Once this Phase 5 Human work and the Machine API in `develop/phase5-ai.md` are
both in place, later phases from Phase 6 onward can seriously build **AI proof
search, RAG, proof search, IDE completion, and educational UI** on top.
