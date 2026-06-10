# Phase 4 AI Profile: Machine Tactics

This document is the design for **Phase 4 for AI** in NPA.

`develop/phase4-human.md` is about human-written `by ...` blocks and tactic
scripts that advance proof states with small readable commands. AI proof search
has different needs: it generates many tactic candidates, expects failure, and
must test candidates quickly, deterministically, and transactionally.

Therefore Phase 4 AI does not trust human tactic syntax directly. It accepts
structured **Machine Tactic** input and connects only successful candidates to
proof terms / certificates.

---

# 1. Purpose

Machine Tactics let AI try tactic candidates outside the trusted boundary and
convert only successful candidates into proof terms the kernel can check.

```text
AI candidate
  ↓ parse / validate Machine Tactic
structured tactic AST
  ↓ transactional tactic execution
new proof state + proof delta
  ↓ unresolved goal check
core proof term
  ↓ kernel check
  ↓ certificate generation / verification
canonical certificate
```

Priorities:

```text
- tactic text is not trusted payload
- tactic trace / score / prompt / model metadata is not in certificates
- proof-state updates are transactional
- failures are structured errors
- the same state + same tactic + same deterministic budget produces the same result / error
- term payloads are checked through the Phase 3 Machine Surface term-level API
```

---

# 2. Trust Boundary

Machine Tactics are not trusted.

```text
Not trusted:
  AI output
  tactic parser
  tactic selection / ranking
  repair suggestion
  proof search trace
  tactic execution log

Trusted:
  deterministic result of the Phase 1 Rust kernel checking a fully explicit proof term
  canonical certificate bytes / hash and Phase 2 certificate verifier result
  Phase 8 independent checker result
```

A successful AI tactic is not a proof. It is accepted only after the final proof
term passes kernel check, canonical certificate verification, and, for required
release / audit profiles, independent checker validation. Human Surface tactic
syntax, notation, and open scopes are not default Machine Tactic inputs. Term
payloads are checked by the explicit Phase 3 AI Machine Surface term API.

---

# 3. Difference from Human Tactics

Human Phase 4 prioritizes readability.

```npa
by
  intro n
  rw [Nat.add_zero]
  exact Eq.refl n
```

AI Phase 4 prioritizes structured input.

```json
{
  "kind": "exact",
  "term": "@Eq.refl.{1} Nat n"
}
```

The Machine Tactic layer does not use:

```text
- short names depending on open / namespace state
- notation / infix / numeric literal overload
- implicit term argument insertion
- tactic macro
- arbitrary tactic script
- backtracking tactic language
- user-defined tactic
- plugin execution
- IO / network / file access
```

Full target tactics / APIs:

```text
- exact
- intro
- apply
- rw
- simp-lite
- induction-nat
- Phase 3 Machine Surface term payload
- verified import / export_hash
```

The first AI MVP completion scope is limited to `exact`, `intro`, and `apply`
as described in section 13.

---

# 4. Prerequisites

Machine Tactics depend on the Phase 3 AI term-level API.

```text
required before Phase 4 AI M2:
  - elaborate_machine_term_check
  - canonicalize_machine_term_source
  - local context import
  - expected type check
  - constants / core_hash extraction
```

`exact`, `apply`, `rw`, and `simp-lite` all require term type checking or rule
interface checks. The current implementation assumes the Phase 3 AI M7
term-level API before running proof-producing tactics. If a future profile lacks
that dependency, implement only the `MachineProofState` skeleton and
parser/validator; proof-producing tactics must fail closed.

---

# 5. Proof State

The proof state exposed to AI is structured, but display state is not trusted
payload.

```rust
struct MachineProofState {
    state_id: StateId,
    root: ProofRoot,
    open_goals: Vec<GoalId>,
    metavars: MetaVarStore,
    env: MachineTacticEnv,
    reserved_local_names: Vec<String>,
    fingerprint: Hash,
}

struct ProofRoot {
    module: ModuleName,
    theorem_name: Name,
    source_index: u64,
    universe_params: Vec<String>,
    theorem_type: Expr,
    body: ProofExpr,
}

struct MachineTacticEnv {
    imports: Vec<VerifiedImportRef>,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    simp_registry: SimpRegistry,
    eq_family: Option<ResolvedEqFamily>,
    nat_family: Option<ResolvedNatFamily>,
    options: MachineTacticOptions,
    options_fingerprint: Hash,
}

struct VerifiedImportRef {
    module: ModuleName,
    export_hash: Hash,
    certificate_hash: Hash,
    exports: Vec<CheckedDeclSignature>,
    certified_env_decls: Vec<Decl>,
}

struct CheckedCurrentDecl {
    // Private in implementation; construct only through the dedicated constructor.
    source_index: u64,
    signature: CheckedDeclSignature,
    core_decl: Decl,
    prior_chain_fingerprint: Hash,
    checked_env_fingerprint: Hash,
}

struct CheckedDeclSignature {
    name: Name,
    universe_params: Vec<String>,
    ty: Expr,
    decl_interface_hash: Hash,
}

enum ProofExpr {
    Core(Expr),
    Meta(MetaVarId),
    App(Box<ProofExpr>, Box<ProofExpr>),
    Lam { name: String, ty: Expr, body: Box<ProofExpr> },
    Let { name: String, ty: Expr, value: Expr, body: Box<ProofExpr> },
}

struct MetaVarStore {
    metas: BTreeMap<MetaVarId, MachineMetaVar>,
    goal_to_meta: BTreeMap<GoalId, MetaVarId>,
    next_id: u64,
}

struct MachineGoal {
    goal_id: GoalId,
    meta_id: MetaVarId,
    context: Vec<MachineLocalDecl>,
    context_hash: Hash,
    target: Expr,
    target_hash: Hash,
}

struct MachineMetaVar {
    id: MetaVarId,
    goal_id: GoalId,
    context: Vec<MachineLocalDecl>,
    target: Expr,
    assignment: Option<ProofExpr>,
}

struct MachineProofDelta {
    previous_state_fingerprint: Hash,
    assigned_goal: GoalId,
    assigned_meta: MetaVarId,
    assigned_proof_expr_hash: Hash,
    new_goals: Vec<GoalId>,
    new_metas: Vec<MachineNewMetaDelta>,
    next_state_fingerprint: Hash,
    proof_delta_hash: Hash,
}

struct MachineNewMetaDelta {
    meta_id: MetaVarId,
    goal_id: GoalId,
    context_hash: Hash,
    target_hash: Hash,
}

struct MachineLocalDecl {
    name: String,
    ty: Expr,
    value: Option<Expr>,
}
```

`ProofExpr` canonical bytes are shared by proof skeleton hashes, metavariable
assignment hashes, and `MachineProofDelta.assigned_proof_expr_hash`.

```text
ProofExpr canonical bytes:
  - tag "npa.phase4.proof-expr.v1"
  - fixed variant tags
  - local references are de Bruijn after final lowering
  - meta refs are allowed only before certificate handoff
  - source spans, tactic text, prompt ids, and scores are excluded
```

`MachineProofState.fingerprint` is a deterministic hash of the theorem root,
open-goal list, metavariable store, checked environment fingerprint, tactic
options fingerprint, and reserved local names. It is not a certificate hash, but
it is the identity used for deterministic tactic execution and caching.

The state fingerprint must not include:

```text
- model name
- prompt
- ranking score
- wall-clock time
- cache hit / miss
- tactic execution log
- diagnostic display strings
```

---

# 6. Machine Tactic AST

Machine tactics are structured values.

```rust
enum MachineTactic {
    Exact {
        term: MachineTermPayload,
    },
    Intro {
        name_hint: Option<String>,
    },
    Apply {
        term: MachineTermPayload,
        max_new_goals: u32,
    },
    Rw {
        rules: Vec<RewriteRulePayload>,
        direction: RewriteDirection,
        target: RewriteTarget,
    },
    SimpLite {
        rules: Vec<RewriteRulePayload>,
        target: RewriteTarget,
        max_steps: u32,
    },
    InductionNat {
        major: LocalRef,
        names: InductionNatNames,
    },
}

enum RewriteDirection {
    Forward,
    Backward,
}

enum RewriteTarget {
    Goal,
    Hyp(LocalRef),
}

struct RewriteRulePayload {
    theorem: MachineTermPayload,
    orientation: Option<RewriteDirection>,
}
```

`MachineTermPayload` is not arbitrary human source. It is the Phase 3 AI Machine
Surface term payload, with explicit global references, explicit universe
arguments when required, no notation, and no implicit insertion unless the
Machine Surface API explicitly validated it.

The tactic AST is validated before execution:

```text
- supported kind
- size limit
- no unknown fields in strict mode
- local refs exist in the target goal context
- term payloads validate through Phase 3 AI term API
- max_new_goals / max_steps within deterministic budget
- rewrite targets and induction major are in scope
```

Unknown tactic kinds fail closed with `UnsupportedMachineTactic`.

---

# 7. Tactic Semantics

## 7.1 exact

`exact t` checks `t` against the goal target.

```text
input:
  goal Γ ⊢ T
  term t

steps:
  elaborate_machine_term_check Γ t T
  assign goal metavariable := t
  close the goal
```

Failure modes include `TermCheckFailed`, `GoalAlreadyAssigned`,
`UnsolvedMachineTermMeta`, and `BudgetExceeded`.

## 7.2 intro

`intro` applies when the goal target is a Pi type after WHNF.

```text
goal:
  Γ ⊢ Π x : A, B

result:
  Γ, x : A ⊢ B
  assigned proof expression is Lam x A ?new_goal
```

The introduced name is generated deterministically from `name_hint` and the
reserved-name set. Name choice affects display only; final lowering uses de
Bruijn indices.

## 7.3 apply

`apply f` tries to use a theorem/function whose result can match the target.

```text
goal:
  Γ ⊢ T

f type:
  Π a1 : A1, ... Π an : An, R

find:
  substitution / metavariables so R matches T

result:
  new goals for unresolved explicit premises
  assigned proof expression f ?g1 ... ?gk
```

`apply` is deterministic and bounded:

```text
- WHNF the candidate type
- instantiate implicit/explicit binders according to Machine Surface metadata
- use conservative unification against the target
- create at most max_new_goals
- preserve goal order from the candidate binder spine
- fail rather than search deeply
```

If multiple instantiations are possible, fail with `AmbiguousApply`; do not pick
based on AI score.

## 7.4 rw

`rw` rewrites by equality theorems.

Required rule shape, conceptually:

```text
theorem r : Eq A lhs rhs
```

Forward direction rewrites `lhs` to `rhs`; backward direction rewrites `rhs` to
`lhs`.

The tactic:

```text
- validates the rule theorem type through the kernel / Machine Surface API
- extracts lhs/rhs under the configured Eq family
- finds deterministic rewrite positions in the target goal or hypothesis
- applies the first configured deterministic position unless a target position is specified
- produces a proof delta using Eq.rec / recursor machinery
```

If no position matches, return `RewriteNoMatch`. If many matches exist and no
deterministic policy is configured, return `AmbiguousRewrite`.

`rw` is target functionality beyond the first MVP scope.

## 7.5 simp-lite

`simp-lite` repeatedly applies safe rewrite rules from the local rule list and
the deterministic simp registry.

Required properties:

```text
- only proof-producing rewrites
- deterministic rule order
- deterministic traversal order
- max_steps budget
- no typeclass search
- no theorem search during execution
- no AI calls
```

Rule order:

```text
1. explicit rules in tactic payload order
2. registry rules by priority descending
3. tie-break by theorem name and decl_interface_hash bytes
```

If the rewrite loop reaches `max_steps`, return `SimpBudgetExceeded`. The result
contains the rewrite sequence and proof delta, but the log is not trusted
payload. The proof term / certificate is what matters.

`simp-lite` is target functionality beyond the first MVP scope.

## 7.6 induction-nat

`induction-nat` is a bounded induction tactic for `Nat`.

It requires:

```text
- configured Nat family
- major premise is a local variable of type Nat
- target can be abstracted over the major premise
- generated base and step goals are deterministic
```

It produces:

```text
base goal:
  major := Nat.zero

step goal:
  n : Nat
  ih : P n
  ⊢ P (Nat.succ n)
```

and assigns the original goal to a Nat recursor proof expression. This tactic is
target functionality beyond the first MVP scope.

---

# 8. Transaction and Budget

Every tactic execution is transactional.

```text
begin transaction
  clone or checkpoint metavariable store
  run validation and tactic semantics
  run local kernel checks required by the tactic
  produce proof delta
commit on success
rollback on failure
```

A failed tactic must not mutate:

```text
- metavariable assignments
- open-goal list
- proof root
- reserved local names
- simp registry
- environment
```

Budget is deterministic and part of the input.

```rust
struct MachineTacticBudget {
    max_term_nodes: u32,
    max_new_goals: u32,
    max_rewrite_steps: u32,
    max_whnf_steps: u64,
    max_unification_steps: u64,
    max_kernel_checks: u32,
}
```

Budget exhaustion returns structured errors. It is not a timeout based on wall
clock. Wall-clock limits may kill the outer process, but they are not part of
semantic tactic results.

---

# 9. Structured Result

Tactic execution returns structured results.

```rust
enum MachineTacticStatus {
    Success(MachineTacticSuccess),
    Failure(MachineTacticError),
}

struct MachineTacticSuccess {
    previous_state_fingerprint: Hash,
    next_state_fingerprint: Hash,
    closed_goals: Vec<GoalId>,
    new_goals: Vec<MachineGoal>,
    proof_delta: MachineProofDelta,
    diagnostics: Vec<MachineTacticDiagnostic>,
}

enum MachineTacticError {
    UnsupportedMachineTactic,
    InvalidPayload,
    UnknownGoal,
    GoalAlreadyAssigned,
    UnknownLocal,
    TermCheckFailed,
    ExpectedFunctionType,
    ExpectedPiGoal,
    UnsolvedMachineTermMeta,
    AmbiguousApply,
    TooManyNewGoals,
    RewriteRuleInvalid,
    RewriteNoMatch,
    AmbiguousRewrite,
    SimpBudgetExceeded,
    InductionMajorNotNat,
    BudgetExceeded,
    KernelRejected,
}
```

Diagnostics are useful for repair, but only the proof delta and later kernel /
certificate checks matter for acceptance.

Result JSON example:

```json
{
  "status": "success",
  "previous_state_fingerprint": "sha256:...",
  "next_state_fingerprint": "sha256:...",
  "closed_goals": ["g0"],
  "new_goals": [],
  "proof_delta_hash": "sha256:..."
}
```

Failure example:

```json
{
  "status": "failure",
  "error_kind": "TermCheckFailed",
  "goal": "g0",
  "repair_hints": [
    "try an exact term whose type matches the target",
    "try apply with a theorem ending in the target"
  ]
}
```

---

# 10. Certificate Handoff

When all goals are closed:

```text
MachineProofState
  ↓ ensure every metavariable has an assignment
ProofExpr skeleton
  ↓ lower to core Expr
closed proof term
  ↓ Phase 1 kernel check against theorem_type
checked declaration
  ↓ Phase 2 certificate builder / verifier
canonical .npcert
```

Certificate handoff rejects:

```text
- unassigned metavariables
- MachineProofState with stale fingerprint
- proof expression containing Meta after lowering
- local refs not convertible to de Bruijn indices
- unchecked current declarations
- imports not verified by Phase 2
- any kernel rejection
```

Tactic traces, proof-search scores, prompts, and model metadata may be kept as
sidecars, but they are not part of the canonical certificate payload.

---

# 11. API Sketch

Initialize:

```json
{
  "module": "Scratch",
  "theorem_name": "add_zero",
  "theorem_type": "...explicit machine term...",
  "verified_imports": [],
  "checked_current_decls": [],
  "options": {
    "profile": "phase4-ai-mvp"
  }
}
```

Run tactic:

```json
{
  "state_id": "s0",
  "state_fingerprint": "sha256:...",
  "budget": {
    "max_new_goals": 8,
    "max_unification_steps": 1000,
    "max_kernel_checks": 8
  },
  "tactic": {
    "kind": "apply",
    "term": {
      "format": "machine-term-v1",
      "payload": "..."
    },
    "max_new_goals": 4
  }
}
```

Response:

```json
{
  "status": "success",
  "next_state_id": "s1",
  "next_state_fingerprint": "sha256:...",
  "closed_goals": ["g0"],
  "new_goals": [
    {
      "goal_id": "g1",
      "target_hash": "sha256:..."
    }
  ],
  "proof_delta_hash": "sha256:..."
}
```

Finalize:

```json
{
  "state_id": "s_final",
  "state_fingerprint": "sha256:...",
  "require_kernel_check": true,
  "require_certificate_verify": true
}
```

---

# 12. Milestones

## M0: Baseline split

- [x] Keep Human tactics and Machine tactics separate.
- [x] Define Machine Tactic trust boundary.
- [x] Fail closed when required Phase 3 AI term APIs are absent.

## M1: MachineProofState core

- [x] Define `MachineProofState`, goals, metavars, proof expressions, and state fingerprint.
- [x] Make fingerprints deterministic and independent of AI metadata.
- [x] Validate imports / checked current declarations as verified environment input.
- [x] Add structured result and error types.

## M2: exact / intro

- [x] Implement `exact` using Phase 3 AI term check.
- [x] Implement `intro` for Pi goals.
- [x] Make both transactional.
- [x] Add tests for success, failure, rollback, and deterministic fingerprint changes.

## M3: apply

- [x] Implement bounded deterministic `apply`.
- [x] Produce new goals in binder-spine order.
- [x] Reject ambiguous or too-large applications.
- [x] Add kernel handoff tests for closed proofs using exact/intro/apply.

## M4: rw / simp-lite

- [ ] Implement proof-producing equality rewrite.
- [ ] Implement deterministic simp-lite rule registry and bounded loop.
- [ ] Add no-match, ambiguity, and budget tests.

## M5: induction-nat

- [ ] Implement bounded Nat induction tactic.
- [ ] Generate base and step goals deterministically.
- [ ] Produce recursor proof expression.

## M6: Certificate handoff

- [x] Lower closed `ProofExpr` to core term.
- [x] Run Phase 1 kernel check.
- [x] Pass checked declaration to Phase 2 certificate builder / verifier.
- [x] Reject unassigned metavariables and stale fingerprints.

## M7: AI search integration gate

- [x] Expose deterministic Machine Tactic execution to AI search.
- [x] Ensure AI trace / score remains sidecar-only.
- [x] Add regression tests for state + tactic + budget determinism.

---

# 13. MVP Completion

The Phase 4 AI MVP is complete when:

```text
- Machine Tactic input is structured and validated
- Human tactic scripts are not accepted as Machine Tactic payload
- proof state, metavariables, and proof deltas have deterministic fingerprints
- exact, intro, and apply are implemented transactionally
- failures roll back all state changes
- results are structured and testable
- closed proofs lower to fully explicit proof terms
- Phase 1 kernel checks the final proof term
- Phase 2 certificate builder / verifier accepts the result
- AI metadata remains outside trusted payloads and hashes
```

`rw`, `simp-lite`, and `induction-nat` are part of the full target but not the
first MVP completion scope unless explicitly included by the implementation
milestone.

---

# 14. One-Sentence Summary

Phase 4 AI is the deterministic, transactional, structured tactic layer that
lets AI try proof steps without making AI, tactic text, traces, or ranking
metadata part of the trusted proof.
