# Phase 7: AI Search

This document is the detailed design for **Phase 7: AI search**. Phase 7 uses
the kernel, certificates, tactics, IDE/API, and standard library built in
Phases 1-6 to let AI search for proof candidates, while accepting only artifacts
that the kernel, canonical certificate verifier, and independent checker can
validate.

Scope:

```text
- premise retrieval
- tactic generation
- best-first search
- error repair
- proof minimization
```

Implementation notes (2026-05-21):

```text
- Phase 7 MVP M0-M9 are implemented as library substrate in crates/npa-api.
- The no-model MVP profile, deterministic search controller, replay / verify closure,
  training trace identity, and M9 integration fixtures are fixed by unit tests in that crate.
- The Phase 7 controller in crates/npa-api is an untrusted producer / orchestrator.
  It is not proof-acceptance evidence until replay / verify and canonical certificate checks pass.
- Phase 7 MVP candidate generation uses the Phase 5 Machine API and Phase 3 AI Machine Surface.
  Human Surface source, notation, open scope, and pretty text are not candidate identity,
  ranking, replay, or verify evidence.
- M10-M13 are Phase 7.5 / later-profile work and are not current MVP success criteria.
```

Design rule:

```text
AI is not trusted.
AI proposes candidates.
The tactic engine tries them.
The kernel, canonical certificate verifier, and independent checker verify them.
Anything unverified is not a proof.
```

Existing LeanDojo-style work shows that extracting proof states, tactics, and
premises for programmatic interaction is central to AI theorem proving, and that
premise selection is a major bottleneck on large libraries. ([LeanDojo][1])

---

# 1. Overview

Phase 7 runs proof search as a loop.

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

The logical flow:

```text
Structured Proof State
  context, target, goals
    ↓
Premise Retriever
  exact/apply/rw/simp candidates
    ↓
Tactic Generator
  templates + model + heuristics
    ↓
Best-first Search
  state graph exploration
    ↓
Tactic Execution API
  intro/exact/apply/rw/simp/...
    ↓
Error Repair
  type mismatch, rewrite failure, etc.
    ↓
Kernel / Certificate / Independent Checker
  final verification
    ↓
Proof Minimization
  shorter replay plan
```

---

# 2. Inputs and Outputs

## 2.1 Input

Phase 7 input is the Phase 5 structured proof state. The MVP does not define an
independent `/ai/*` proof-state protocol; it is a Machine API client for
`develop/phase5-ai.md`.

Search node identity is fixed by Phase 5 AI payloads:

```text
required Phase 5 AI handles:
  - session_id
  - session_root_hash
  - snapshot_id
  - state_fingerprint
  - goal_id
  - raw MachineTacticCandidate payload
  - deterministic_budget
  - candidate_hash / proof_delta_hash for successful replay steps
```

The MVP controller uses:

```text
/machine/snapshots/get
/machine/search/for_goal
/machine/tactics/batch
/machine/replay
/machine/verify
```

`/machine/tactics/run` exists as a Phase 5 single-candidate API, but the Phase 7
MVP search algorithm uses batch execution. A run-based controller is a
non-MVP profile.

`/machine/prompt_payload` is used only after enabling `ModelGenerator`; it is not
on the deterministic MVP path.

If Phase 7 is built before a production Phase 5 implementation, it must use a
`MachineApiClient` boundary with the same request / response / error taxonomy as
Phase 5. Deterministic fakes may be used in tests, but fake/mock identity must
not enter replay plans, certificates, state fingerprints, candidate hashes, or
training identity.

`/machine/snapshots/get` uses `include_pretty = false` in the MVP. Pretty display
is never used for candidate generation, goal selection, priority, dedupe, or
training identity.

MVP sources of truth:

```text
current state:
  Phase 5 MachineProofSnapshot
  session_id / snapshot_id / state_fingerprint / goal_id

tactic candidate:
  Phase 5 raw MachineTacticCandidate wire payload
  Phase 7 score/source metadata lives outside the candidate

transition:
  /machine/tactics/batch per-candidate success result
  candidate_hash / deterministic_budget_hash / proof_delta_hash / next_state_fingerprint

accepted proof:
  complete MachineReplayPlan
  passes /machine/replay
  passes /machine/verify with status = verified
```

Any `state_id`, `proof_script`, text tactic, or `/ai/*` examples are display
only unless explicitly marked as non-MVP wrappers. MVP wire contracts must send
raw `MachineTacticCandidate` payloads.

## 2.2 Output

Phase 7 output is not a theorem by itself. Outputs are:

```text
- candidate batches
- search traces
- replay plans
- minimized replay plans
- training/audit traces
- verify responses from Phase 5
```

Only a `/machine/verify` success response can be treated as a verified proof.

---

# 3. Premise Retrieval

## 3.1 Purpose

Premise retrieval narrows a large verified theorem set to candidates useful for
the current goal.

## 3.2 Premise Index

The premise index comes from Phase 5 / Phase 6 verified theorem metadata.

Each premise identity includes:

```text
- module
- export_hash
- certificate_hash
- global_ref
- decl_interface_hash
- statement_core_hash
- axiom dependency summary
- attributes and rewrite metadata as untrusted sidecar
```

No premise is identified by pretty name alone.

## 3.3 Accessible Premises

Accessible premises are limited to direct imports and the verified import
closure available to the current Phase 5 session. Phase 7 does not auto-insert
imports and does not search unimported modules as accepted tactic heads.

## 3.4 Search Modes

Modes:

```text
exact retrieval:
  find premises whose statement can match the goal target

apply retrieval:
  find premises whose result can produce the target with new subgoals

rw retrieval:
  find equality theorems usable as rewrite rules

simp retrieval:
  find safe simp profile rules

lexical retrieval:
  name / token matching

type-aware retrieval:
  kernel-shaped matching against goal and context

graph-aware retrieval:
  prioritize nearby dependency graph nodes

embedding retrieval:
  later non-MVP sidecar retrieval mode
```

Embedding retrieval is not part of the deterministic MVP completion criteria.

## 3.5 Retriever Score

Scores are untrusted ranking metadata. They may combine:

```text
- exact/type match
- attribute match
- rewrite applicability
- graph proximity
- lexical overlap
- model/embedding score in later profiles
```

Scores do not affect proof acceptance. Deterministic MVP scores must be
reproducible from verified metadata and query canonical bytes.

## 3.6 Premise Retrieval API

Phase 7 calls Phase 5 theorem search. Responses include verified premise
metadata and optional `suggested_candidates`. Phase 7 may use suggestions, but
still sends them through `/machine/tactics/batch`.

---

# 4. Tactic Generation

## 4.1 Purpose

Tactic generation converts state + premises into raw `MachineTacticCandidate`
values.

Candidate sources:

```text
- built-in deterministic tactics
- templates
- premise-based generators
- model-based generators in later profiles
- repair generators
- exploration generators
```

MVP generation is no-model and deterministic.

## 4.2 Generator Components

### BuiltinGenerator

Produces candidates such as:

```text
intro
exact from reflexivity templates
apply from Phase 5 suggested candidates
simp-lite from safe profiles
```

### TemplateGenerator

Fills deterministic tactic templates with verified premise metadata. Templates
must emit fully structured `MachineTacticCandidate` payloads.

### PremiseBasedGenerator

Uses retrieval results to produce `apply`, `rw`, and `simp-lite` candidates
where supported by the current tactic profile.

### ModelGenerator

Non-MVP generator using prompt payloads and model output. It must emit only
structured candidates, never accepted proofs.

### RepairGenerator

Turns structured tactic errors into new candidates. Repairs are bounded and
deterministic in the MVP.

### ExplorationGenerator

Adds low-priority deterministic exploration candidates to avoid narrow search.

## 4.3 Tactic Candidate Schema

Every candidate envelope records:

```text
- raw MachineTacticCandidate payload
- candidate_hash recomputed by Phase 5
- source kind
- generator profile version
- score
- parent search node
- deterministic budget
```

Only the raw candidate and deterministic budget enter replay. Source kind and
score are trace metadata.

## 4.4 Forbidden Tactics / Tokens

MVP candidate generation must not emit:

```text
- human tactic scripts
- notation-dependent payloads
- short names requiring open/namespace state
- unknown tactic kinds
- plugin/user-defined tactics
- IO/network/file operations
- candidates with pretty-only references
- candidates requiring server-side model acceptance
```

---

# 5. Best-First Search

## 5.1 Purpose

Best-first search explores the proof-state graph by prioritizing promising
candidate transitions while preserving replayability.

## 5.2 Search Node

A search node contains:

```rust
struct SearchNode {
    node_id: SearchNodeId,
    session_root_hash: Hash,
    snapshot_id: SnapshotId,
    state_fingerprint: Hash,
    goal_id: GoalId,
    depth: u32,
    replay_prefix: Vec<MachineReplayStep>,
    score: SearchScore,
    status: NodeStatus,
}
```

Node identity is based on `session_root_hash`, `state_fingerprint`, open-goal
shape, and replay prefix identity. Pretty text and tactic display strings are
not identity.

## 5.3 State Identity

Use Phase 5 `state_fingerprint` for dedupe. If two paths reach the same state,
keep the cheaper / shorter replay prefix according to deterministic tie-breaks.

## 5.4 Priority Function

Priority may include:

```text
+ candidate score
+ premise retrieval score
+ goal simplification score
+ closed-goal bonus
+ short replay bonus
- depth penalty
- repeated-state penalty
- large-new-goal penalty
- deterministic budget cost
```

Tie-breakers are deterministic:

```text
state_fingerprint bytes
candidate_hash bytes
generator kind order
insertion sequence number
```

## 5.5 Best-First Search Algorithm

Sketch:

```python
def search(session, initial_snapshot, budget):
    queue = PriorityQueue()
    queue.push(initial_node(initial_snapshot))
    visited = {}

    while queue and not budget.exceeded():
        node = queue.pop()

        if dominated(node, visited):
            continue
        visited[node.state_fingerprint] = node

        snapshot = machine.get_snapshot(node, include_pretty=False)
        premises = machine.search_for_goal(snapshot, node.goal_id)
        candidates = generate_candidates(snapshot, premises, node)
        batch = machine.tactics_batch(snapshot, candidates)

        for result in batch.results:
            record_trace(node, result)

            if result.success:
                next_node = extend_node(node, result)

                if result.open_goals_empty:
                    plan = build_replay_plan(next_node)
                    if machine.replay(plan).ok and machine.verify(plan.final_snapshot).verified:
                        return minimize(plan)

                queue.push(next_node)
            else:
                repairs = repair(result.error, node, snapshot)
                queue.push_many(repairs)

    return Failure(trace=export_trace())
```

The controller never accepts a proof without replay and verify.

## 5.6 Goal Selection

MVP goal selection is deterministic:

```text
- pick the first open goal from the snapshot's canonical open_goals order
- later profiles may rank goals, but ranking must be trace metadata
```

## 5.7 Budget

Search budget is separate from Phase 5 deterministic tactic budget.

```text
Search budget:
  max_nodes
  max_edges
  max_depth
  max_successful_transitions
  max_repair_candidates

Tactic budget:
  Phase 5 deterministic_budget per candidate
```

Scheduler wall-clock stops are retryable artifacts, not semantic failures.

## 5.8 Parallelization

Parallel search may evaluate independent batch candidates or nodes. The result
must remain deterministic by using stable ordering when merging results.
Parallelism must not change replay plans for the same semantic inputs.

## 5.9 Search Modes

Modes:

```text
no-model deterministic MVP
model-assisted later profile
embedding retrieval later profile
parallel later profile
```

The MVP completion criteria apply to the no-model deterministic mode.

---

# 6. Error Repair

## 6.1 Purpose

Repair uses structured tactic errors to generate bounded follow-up candidates.

## 6.2 Error Schema

Repair consumes Phase 5 / Phase 4 structured errors, not display strings.

```text
error_kind
phase
goal_id
candidate_hash
expected / actual type hashes
related premise metadata
budget error category
```

## 6.3 Repair Rule Table

No-op repair kinds:

```text
UnsupportedMachineTactic
InvalidReplayPlan
KernelRejected after verify
```

`unknown_name`:

```text
do not guess names from pretty text in MVP
```

`type_mismatch`:

```text
try apply candidates whose result matches expected type
try exact candidates from matching premises
```

`expected_pi_type`:

```text
avoid intro on non-Pi goals
try apply/exact instead
```

`rewrite_rule_invalid` / `simp_no_progress`:

```text
remove the invalid rule
try reverse direction only when descriptor allows it
try smaller safe simp profile
```

`implicit_argument_required`:

```text
ask Machine Surface to emit explicit term payloads
do not rely on Human implicit insertion
```

`too_many_goals`:

```text
lower max_new_goals or deprioritize the candidate
```

`budget_exceeded` / scheduler stop:

```text
distinguish deterministic budget exhaustion from scheduler interruption
deterministic budget errors may be repaired by smaller candidates
scheduler stops are retryable artifacts
```

## 6.4 RepairGenerator

The repair generator emits structured candidate envelopes with source kind
`repair`. It is bounded, deterministic, and never mutates the current state
without Phase 5 batch execution.

## 6.5 Repair Limits

Repairs must not:

```text
- call an LLM in the MVP
- use pretty strings as identity
- invent imports
- invent axioms
- bypass replay / verify
```

---

# 7. Proof Minimization

## 7.1 Purpose

After finding a verified replay plan, minimization tries to shorten the plan
while preserving replay and verify success.

## 7.2 Principles

```text
- minimize only verified replay plans
- every proposed shorter plan must replay and verify
- minimization traces are sidecars
- proof acceptance remains certificate-based
```

## 7.3 Minimization Passes

Pass 1: tactic deletion

```text
remove one replay step and test replay / verify
```

Pass 2: block replacement

```text
replace a block with a shorter deterministic candidate sequence
```

Non-MVP pass: exact replacement

```text
try replacing a subproof with exact term generated from verified proof term
```

Pass 3: existing simp-lite rule deletion

```text
remove unnecessary simp rules and retest
```

Pass 5: namespace shortening

```text
display-only shortening; does not affect MachineReplayPlan identity
```

Pass 6: import minimization

```text
remove unused direct imports only if certificate verification still succeeds
```

## 7.4 Algorithm

Minimization loops over deterministic passes and accepts a change only if
`/machine/replay` and `/machine/verify` both succeed. The minimized plan keeps
the same trusted theorem result or produces a new verified certificate.

## 7.5 Score

Minimization score may include:

```text
- fewer replay steps
- fewer candidate bytes
- fewer imports
- fewer simp rules
- smaller proof term
```

Tie-breaks are deterministic.

## 7.6 API

The minimization API consumes a verified replay plan and returns a verified
replay plan plus sidecar trace. It cannot return unverified proof status.

---

# 8. Phase 7 Controller Boundary

The Phase 7 controller is an orchestrator. It is allowed to call retrieval,
generate candidates, schedule batches, repair failures, and minimize verified
plans. It is not allowed to accept proofs.

Proof acceptance path:

```text
MachineReplayPlan
  ↓ /machine/replay
final snapshot
  ↓ /machine/verify
kernel + certificate verifier
  ↓ optional Phase 8 checker
verified result
```

---

# 9. Training Data

## 9.1 Stored Data

Store:

```text
- session_root_hash
- state_fingerprint
- goal_id
- candidate_hash
- deterministic_budget_hash
- proof_delta_hash on success
- structured error kind on failure
- premise query fingerprint
- selected premise hashes
- generator kind
- replay plan hash
- verify result hash
```

Do not store as identity:

```text
- pretty goal text
- prompt text
- model name
- model score
- wall-clock timing
- cache hit/miss
```

These may appear in diagnostic sidecars only.

## 9.2 Learning Tasks

Possible later training tasks:

```text
- premise ranking
- tactic candidate ranking
- value model for states
- repair selection
- minimization choice
```

## 9.3 Positive / Negative Examples

Positive examples are successful transitions and verified replay plans.
Negative examples are structured failures. Scheduler stops are not semantic
negative examples unless categorized separately.

---

# 10. Milestones

## M0. Machine API boundary fixed

Controller depends only on Phase 5 Machine API contracts.

## M1. Deterministic snapshot and premise retrieval

Retrieve snapshots with `include_pretty = false` and query verified premise
metadata.

## M2. CandidateEnvelope and deterministic tactic generation

Generate raw `MachineTacticCandidate` payloads with metadata outside the
candidate.

## M3. Batch execution and replay step construction

Execute candidates through `/machine/tactics/batch` and build replay steps from
success results.

## M4. Deterministic best-first search controller

Search proof states with deterministic priority and dedupe.

## M5. Rule-based error repair

Generate bounded repair candidates from structured errors.

## M6. Replay / verify closure

When open goals are empty, call `/machine/replay` and `/machine/verify`.

## M7. Proof minimization

Minimize verified replay plans only.

## M8. Trace, training data, and audit artifacts

Emit trace identities suitable for audit and later training.

## M9. MVP integration fixtures

Prove basic Std.Nat / Std.List fixtures with no model.

## M10. Model-based tactic generation

Later profile; not MVP.

## M11. Embedding retrieval sidecar

Later profile; not MVP.

## M12. Value model ranking

Later profile; not MVP.

## M13. Parallel search profile

Later profile; not MVP.

---

# 11. Minimum Configuration

The Phase 7 MVP uses:

```text
- Phase 5 Machine API client
- deterministic premise retrieval
- no-model candidate generators
- exact / intro / apply baseline
- optional Phase 4-supported simp-lite / rw fixtures
- best-first search
- rule-based repair
- replay / verify closure
- minimization of verified replay plans
```

---

# 12. Test Cases

## 12.1 exact retrieval

Goal `n = n` retrieves reflexivity and generates an exact/apply candidate that
closes the goal through Phase 5 batch, replay, and verify.

## 12.2 simp-lite

When the session enables safe simp-lite, goal `n + Nat.zero = n` can be solved
with verified simp metadata and Phase 4 tactic validation.

## 12.3 rw

When rewrite is enabled, equality rewrite candidates use verified rule metadata
and replay successfully.

## 12.4 apply later

General apply fixtures beyond the MVP require fully specified `TacticHead`,
`universe_args`, and `CandidateApplyArg` JSON before inclusion in completion
criteria.

## 12.5 induction-nat

This fixture requires an induction-enabled session with `induction-nat` in
`allowed_tactics` and `MachineTacticEnv.nat_family = Some(_)`. It is not part of
the baseline MVP if the emitted Phase 6 recipes have `nat_family = null`.

Expected flow:

```text
generate Intro for n
search from snapshot with n : Nat
generate InductionNat for n
base solved by simp-lite
step solved by simp-lite
```

## 12.6 List induction later

List induction is non-MVP until an `induction-list` or general induction tactic
is specified.

---

# 13. Excluded from Phase 7 MVP

Deferred:

```text
- natural-language formalization
- generated intermediate lemmas with have
- advanced search using constructor / cases / refine
- full simp
- ring / omega / linarith
- MCTS
- RL training
- List induction / general induction
- theorem invention
- large-scale multi-file automated lemma discovery
- natural-language explanation of proof strategy
```

The initial target is to automatically find basic proofs using existing tactics
and the standard library.

---

# 14. Completion Criteria

Phase 7 MVP is complete when M0-M9 are satisfied:

```text
- the search controller works only through the Phase 5 Machine API client boundary
- relevant premises can be retrieved from the current goal
- raw MachineTacticCandidate values can be generated from Phase 5 suggested_candidates and MVP builtins
- general exact/apply generation can remain non-MVP when not specified
- multiple candidates can be generated
- best-first search explores the proof-state graph
- tactic failures are stored as structured errors
- rule-based repair works
- empty open_goals triggers /machine/replay and /machine/verify
- found proofs can be minimized as replay plans
- search / training traces are emitted as auditable artifacts
- basic Std.Nat / Std.List fixtures can be proved without an AI model
```

M10-M13 are later profile completion criteria and must not block the MVP. Adding
them must preserve deterministic fallback and the replay / verify boundary.

---

# 15. One-Sentence Summary

Phase 7 is the verified-search engine that combines structured proof states,
verified standard-library metadata, deterministic candidate generation, batch
tactic execution, error repair, best-first search, replay, verify, and
minimization.

The core flow is:

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

Start with deterministic search and no AI model; add LLMs, embedding retrieval,
and value models later without weakening proof trust.

[1]: https://leandojo.readthedocs.io/?utm_source=chatgpt.com "LeanDojo: Machine Learning for Theorem Proving in Lean ..."
[2]: https://arxiv.org/abs/2306.15626?utm_source=chatgpt.com "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models"
[3]: https://www.nature.com/articles/s41586-025-09833-y?utm_source=chatgpt.com "Olympiad-level formal mathematical reasoning with ..."
