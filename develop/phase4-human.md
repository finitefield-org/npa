The following is the detailed design for **Phase 4 Human Profile: Human
Tactic**. By Phase 3, surface syntax can be lowered to core terms. In Phase 4
overall, unresolved goals produced by `_` or `?m` become solvable by small
commands. However, this document defines the Human Profile for humans to write
`by ...` blocks and tactic scripts. The fast, structured Machine Tactic for AI
is the responsibility of `develop/phase4-ai.md`, and this Human syntax is not
the default input there.

The target tactics in human-facing syntax are these six. On the AI/Machine wire,
the Nat-only induction tactic is called `induction-nat`, but in Human syntax it
is written `induction n`, and the Human bridge converts it to the Nat-only
Machine tactic or a core proof term.

```text
intro
exact
apply
rw
simp-lite
induction
```

The core principle does not change.

```text
tactics are not trusted.
tactics only assemble proof terms.
the kernel checks the proof term at the end.
```

# 0. Constraint: Do Not Slow The AI-Facing Machine Tactic Fast Path

Phase 4 Human is the layer that adds human-facing `by` blocks and tactic
scripts. It is implemented in a way that does not harm execution speed for the
AI-facing Machine tactic / Machine API. Human syntax is only a convenient input
format and is not placed on the hot path where AI submits many candidates.

The AI-facing path continues to go only through the following Machine
representations.

```text
Phase 7 / Phase 9 / Machine API
  → MachineTacticCandidate JSON
  → validate_machine_tactic_candidate
  → run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
  → MachineTermSource / parse_machine_* / canonicalize_machine_term_source
  → kernel / certificate
```

The human-facing path interprets Human syntax once, lowers it to an existing
Machine tactic or core proof term, and then proceeds to checking.

```text
Human source
  → parse_human_* / `by` tactic script parser
  → resolve_human_*
  → convert to MachineTactic or checked core proof term
  → run_machine_tactic_* / extract proof
  → kernel / certificate
```

Therefore, implementation observes the following constraints.

```text
- Do not put the Human tactic parser into the Machine tactic hot path in `crates/npa-tactic`.
- Do not mix Human notation / namespace / implicit auxiliary information into `MachineTacticCandidate`, `MachineTactic`, or `MachineTermSource`.
- Do not perform Human syntax fallback parsing in `run_machine_tactic_with_budget` or `run_machine_tactic_candidates_batch`.
- Do not extend `parse_machine_*` or `canonicalize_machine_term_source` to accept Human syntax.
- Phase 7 / Phase 9 automation continues to use Machine JSON and Machine term canonicalization.
- Human convenience is untrusted preprocessing, and is checked by the kernel as a core proof term before certificate generation.
- Do not put I/O, network, plugin loading, AI calls, or search import lookup into the Human tactic bridge.
- Machine tactic budgets, hashes, and cache keys depend only on Machine payload / proof state fingerprint / deterministic budget.
```

Thus, even if human-facing syntax grows, AI-facing high-volume candidate
verification does not go through the Human parser or name resolution. Phase 4
Human implementation reviews must always confirm that this separation is
preserved.

The check method is as follows.

```sh
rg -n "parse_human|compile_human|Human" \
  crates/npa-tactic/src/lib.rs crates/npa-api/src/tactic.rs crates/npa-api/src/adapter.rs

rg -n "parse_machine|canonicalize_machine_term_source|MachineTacticCandidate" \
  crates/npa-api/src/phase7.rs crates/npa-api/src/phase9.rs
```

The former is a static check that the Human parser has not entered the Machine
hot path; the latter confirms that Phase 7 / Phase 9 continue to use the
Machine API. On implementation changes, pass at least the regression tests for
Machine surface / Phase 7 / Phase 9 / tactic crate.

```sh
cargo test -p npa-frontend --lib machine_surface
cargo test -p npa-api phase7
cargo test -p npa-api phase9
cargo test -p npa-tactic
```

The tactic script examples in this Phase 4 document sometimes use `0` for
readability. In real Phase 3 MVP input, until numeric literals are added, it is
enough to write `Nat.zero` or `zero` inside an opened namespace. The core proof
term assembled by the tactic retains only the canonical `Const` reference to
`Nat.zero`.

---

# 1. Basic Model Of The Tactic Layer

A tactic is a function that transforms proof state.

```text
ProofState → TacticResult
```

More precisely:

```text
tactic:
  reads the current goal,
  creates part of a proof term satisfying that goal,
  and generates new subgoals if necessary.
```

For example:

```npa
theorem t : A → A := by
  intro x
  exact x
```

internally becomes this.

```text
initial goal:
  ⊢ A → A

intro x:
  x : A
  ⊢ A

exact x:
  closed
```

However, the final proof passed to the kernel is not the tactic script.

```text
λ x : A, x
```

It is this core proof term.

---

# 2. ProofState Structure

In Phase 4, holes / metavariables are treated as goals.

```rust
struct ProofState {
    goals: Vec<GoalId>,
    metavars: MetaVarStore,
    env: Env,
}
```

Each goal is:

```rust
struct Goal {
    id: GoalId,
    context: LocalContext,
    target: ExprId,
    assignment: Option<ExprId>,
    source_span: Option<Span>,
}
```

Display example:

```text
goal g1

n : Nat
⊢ n = n
```

Internally:

```text
?g1 : Eq Nat n n
```

This is a metavariable.

Solving a goal with a tactic basically means assigning:

```text
?g1 := proof_term
```

to it.

When new subgoals appear, new metavariables are embedded inside the proof term.

---

# 3. Common Tactic API

```rust
trait Tactic {
    fn run(&self, state: &mut ProofState, goal: GoalId) -> Result<TacticOutcome>;
}
```

Result:

```rust
struct TacticOutcome {
    solved_goal: GoalId,
    new_goals: Vec<GoalId>,
    proof_delta: ProofDelta,
    messages: Vec<Message>,
}
```

For an external API aimed at a Human IDE / CLI:

```json
{
  "state_id": "s1",
  "goal_id": "g1",
  "tactic": "intro n"
}
```

At the external boundary of the AI / Machine API, this `"tactic": "intro n"`
form is not the main input. The AI path receives `MachineTacticCandidate` JSON
from `develop/phase4-ai.md` and proceeds to `validate_machine_tactic_candidate`
without passing through the Human text tactic parser.

Result:

```json
{
  "status": "success",
  "new_state_id": "s2",
  "closed_goals": [],
  "new_goals": [
    {
      "goal_id": "g2",
      "context": [
        {"name": "n", "type": "Nat"}
      ],
      "target": "n = n"
    }
  ]
}
```

On failure:

```json
{
  "status": "error",
  "error_kind": "expected_pi_type",
  "message": "`intro` can only be used when the target is a function type or forall."
}
```

---

# 4. `intro`

## 4.1 Role

`intro` adds an assumption to the context when the goal target is a `Pi` type,
that is, a function type or universal proposition.

Example:

```npa
theorem id_nat : Nat → Nat := by
  intro n
  exact n
```

Initial state:

```text
⊢ Nat → Nat
```

After `intro n`:

```text
n : Nat
⊢ Nat
```

## 4.2 Core Proof Term

`intro` creates a lambda.

```text
goal:
  ⊢ Π x : A, B

intro x:
  new goal:
    x : A ⊢ B

old goal assignment:
  ?g := λ x : A, ?g_new
```

That is:

```text
?g : Π x : A, B
?g := Lam x : A, ?g_new
```

## 4.3 Algorithm

```rust
fn tactic_intro(state: &mut ProofState, goal_id: GoalId, name: NameId) -> Result<()> {
    let goal = state.get_goal(goal_id);
    let target = whnf(state.env, &goal.context, goal.target)?;

    match target {
        ExprKind::Pi { ty: domain, body, binder_info, .. } => {
            let local = state.context_add_local(goal.context, name, domain);
            let new_target = instantiate_with_local(body, local);
            let new_goal = state.new_goal(local.context, new_target);

            let proof = mk_lam(name, domain, mk_mvar(new_goal));
            state.assign_goal(goal_id, proof);
            state.replace_goal(goal_id, new_goal);

            Ok(())
        }
        _ => Err(Error::ExpectedPiType),
    }
}
```

## 4.4 Supported Targets

Targets where `intro` can be used:

```text
⊢ A → B
⊢ ∀ x : A, P x
⊢ Π x : A, B x
```

Targets where it cannot be used:

```text
⊢ Nat
⊢ n = n
⊢ A ∧ B
```

For `A ∧ B`, `constructor` will be used in the future. The Phase 4 MVP does not
need to include `constructor` yet.

---

# 5. `exact`

## 5.1 Role

`exact t` directly closes the current goal with term `t`.

Example:

```npa
theorem self_eq (n : Nat) : n = n := by
  exact Eq.refl n
```

Goal:

```text
n : Nat
⊢ n = n
```

`exact Eq.refl n` confirms that `Eq.refl n` has the same type as the target, and
closes the goal.

## 5.2 Core Proof Term

```text
?g := elaborated_term
```

That is all.

## 5.3 Algorithm

```rust
fn tactic_exact(state: &mut ProofState, goal_id: GoalId, term: SurfaceExpr) -> Result<()> {
    let goal = state.get_goal(goal_id);

    let core = elaborate_check(
        &state.env,
        &goal.context,
        term,
        goal.target,
    )?;

    if has_unsolved_metas(core) {
        return Err(Error::UnsolvedMetasInExact);
    }

    state.assign_goal(goal_id, core);
    state.remove_goal(goal_id);

    Ok(())
}
```

## 5.4 Design Decision For `exact`

In the Phase 4 MVP, `exact` is conservative.

```text
exact t
```

If unresolved metavariables remain inside `t`, it fails.

That is:

```npa
exact _
```

fails as `exact`.

In the future, however, `refine` can be introduced:

```npa
refine f ?_
```

so explicit subgoals can be left behind.

In Phase 4:

```text
exact:
  closes completely

apply:
  may generate necessary subgoals
```

This division is easy to understand.

---

# 6. `apply`

## 6.1 Role

`apply f` matches the current target against the conclusion of some theorem,
assumption, or function `f`, and turns the necessary premises into subgoals.

Example:

```npa
theorem trans_example
  (A : Type)
  (x y z : A)
  (h1 : x = y)
  (h2 : y = z)
  : x = z := by
  apply Eq.trans
  exact h1
  exact h2
```

Conceptually:

```text
Eq.trans : x = y → y = z → x = z
```

Therefore, applying `apply Eq.trans` to goal `x = z` produces:

```text
subgoal 1:
  ⊢ x = ?y

subgoal 2:
  ⊢ ?y = z
```

The result looks like this. `?y` may also be determined by unification.

## 6.2 Core Proof Term

`apply f` creates the following shape.

```text
?g := f ?a1 ?a2 ... ?an
```

Then unresolved `?ai` values become new goals.

## 6.3 Basic Algorithm

```text
goal target:
  T

term f:
  f : Π x₁ : A₁, Π x₂ : A₂, ..., R

apply f:
  unify R with T
  create metavariable ?mᵢ for each xᵢ when needed
  ?g := f ?m₁ ?m₂ ...
  make unresolved ?mᵢ values into subgoals
```

Pseudocode:

```rust
fn tactic_apply(state: &mut ProofState, goal_id: GoalId, f_expr: SurfaceExpr) -> Result<()> {
    let goal = state.get_goal(goal_id);

    let (mut f_core, mut f_ty) = elaborate_infer(
        &state.env,
        &goal.context,
        f_expr,
    )?;

    let mut args = Vec::new();
    let mut new_goals = Vec::new();

    loop {
        let ty_whnf = whnf(state.env, &goal.context, f_ty)?;

        match ty_whnf {
            ExprKind::Pi { ty: domain, body, binder_info, .. } => {
                let m = state.new_meta(goal.context.clone(), domain);

                args.push(m.as_expr());
                f_core = mk_app(f_core, m.as_expr());
                f_ty = instantiate(body, m.as_expr());

                if binder_info.is_explicit_or_proof_relevant() {
                    new_goals.push(m.goal_id());
                }

                // Try checking whether the conclusion can unify with the target.
                if can_unify(state, f_ty, goal.target) {
                    unify(state, f_ty, goal.target)?;
                    break;
                }
            }
            _ => {
                unify(state, ty_whnf, goal.target)?;
                break;
            }
        }
    }

    state.assign_goal(goal_id, f_core);
    state.replace_goal_with_many(goal_id, new_goals);

    Ok(())
}
```

## 6.4 Important Points For `apply`

`apply` fills arguments until the conclusion matches the target.

Example:

```text
f : A → B → C
goal:
  ⊢ C
```

then:

```text
?g := f ?a ?b
```

New goals:

```text
⊢ A
⊢ B
```

are created.

However, implicit arguments should generally not become goals. They are often
solvable by unification.

```text
implicit metavariable:
  solved by the elaborator/unifier

explicit/proof argument:
  shown to the user as a subgoal
```

## 6.5 Failure Conditions

```text
- f is not a function type
- f's conclusion cannot unify with the target
- implicit metavariables cannot be solved and are not acceptable
- occurs check fails
```

Error example:

```text
cannot apply `Nat.succ`

target:
  n = n

`Nat.succ` has type:
  Nat → Nat

which does not match the target.
```

---

# 7. `rw`

## 7.1 Role

`rw [h]` rewrites the target or assumptions using equality `h`.

Example:

```npa
theorem rw_example
  (a b : Nat)
  (h : a = b)
  : a = a := by
  rw [h]
```

Target:

```text
a = a
```

After `rw [h]`:

```text
b = b
```

With occurrence selection, it can also rewrite only one side. In the Phase 4
MVP, it is enough to first rewrite "all occurrences in the target".

## 7.2 Rewrite Rule Shape

The thing passed to `rw` is basically an equality proof.

```text
h : Eq A lhs rhs
```

Using this:

```text
lhs  ↦ rhs
```

it rewrites as:

The reverse direction is:

```npa
rw [← h]
```

or:

```npa
rw [<- h]
```

.

## 7.3 What Gets Rewritten

In the Phase 4 MVP, only the target is rewritten.

```npa
rw [h]
```

rewrites the target.

Syntax for rewriting assumptions:

```npa
rw [h] at h2
rw [h] at *
```

is added later.

## 7.4 Internal Processing Of `rw`

```text
1. elaborate h
2. reduce the type of h to WHNF
3. check that it has the shape Eq A lhs rhs
4. find subterms in the target that match lhs
5. create new_target by replacing those parts with rhs
6. generate a transformation that creates a proof of the old target from a proof of the new target
7. create a new goal
```

The important point is that changing the target is not enough; a **proof term
transformation** is also needed.

## 7.5 Proof Term Idea

Suppose the target is:

```text
P lhs
```

and rewriting gives:

```text
P rhs
```

.

The equality:

```text
h : lhs = rhs
```

can be used to create a proof of `P lhs` from a proof of `P rhs`.

This is because the original goal is `P lhs`.

Thus `rw [h]` usually shows the user a new goal `P rhs`, but internally creates
a proof term like:

```text
?old_goal := Eq.subst h ?new_goal
```

Direction needs care.

```text
old target:
  P lhs

new target:
  P rhs

new goal proof:
  ?new : P rhs

old proof:
  Eq.subst h ?new : P lhs
```

The actual type and direction of `Eq.subst` depend on the design, so the
implementation must match them exactly.

## 7.6 MVP Simplification

`rw` is difficult to build fully. In Phase 4, it is better to start with a
heavily restricted version.

Phase 4 MVP `rw`:

```text
- supports Eq only
- supports target only
- explicit left-to-right / right-to-left direction
- all first-found occurrences or only the first occurrence
- limited dependent rewrite
- rewrite proof generated by Eq.rec / Eq.subst
```

Things not added yet:

```text
- setoid rewrite
- rewriting under binders
- rewriting in hypotheses
- occurrence selection
- integration with simp attributes
- heterogeneous equality
- conditional rewrite
```

## 7.7 Rewrite Search

Search for subterms in the target that match lhs.

```text
target:
  Eq Nat (Nat.add n Nat.zero) n

rule:
  Nat.add ?x Nat.zero ↦ ?x
```

Match:

```text
?x := n
```

After replacement:

```text
Eq Nat n n
```

Now `Eq.refl n` can be used.

---

# 8. `simp-lite`

## 8.1 Role

`simp-lite` is a tactic that automatically simplifies the target using
registered simple rewrite rules.

Lean's full `simp` is very powerful, but building the same thing from the start
is difficult. In Phase 4, build a **small but proof-producing simplifier**.

Example:

```npa
theorem add_zero_example (n : Nat) : n + 0 = n := by
  simp-lite
```

`simp-lite` uses `Nat.add_zero` and definition unfolding to simplify the target
to:

```text
n = n
```

and closes it with `Eq.refl n`.

## 8.2 simp rule

Simp rules are basically equality theorems.

```text
Nat.add_zero : ∀ n : Nat, n + 0 = n
Nat.zero_add : ∀ n : Nat, 0 + n = n
```

Internally, they are registered as rewrite rules.

```rust
struct SimpRule {
    theorem: GlobalRef,
    lhs: ExprId,
    rhs: ExprId,
    orientation: RewriteDirection,
    priority: u32,
}
```

## 8.3 simp-lite Processing

```text
1. reduce the target to WHNF
2. try rewrite rules on each subterm in the target
3. replace when successful
4. repeat until it stops
5. if the final target is reflexive equality, close with Eq.refl
6. if it cannot close, leave a new simplified goal
```

In Phase 4, it is fine to fail when the goal cannot be closed.

```text
simp-lite:
  success if the target can be fully closed
  failure if it cannot close
```

Later:

```text
simp:
  simplify the target and leave a new goal
```

This behavior can also be added.

## 8.4 Proof-Producing Requirement

It is not enough for `simp-lite` to merely rewrite the target.

For each rewrite, record a proof term.

```text
target0
  -- rewrite by Nat.add_zero
target1
  -- rewrite by Nat.zero_add
target2
  -- refl
closed
```

Finally:

```text
?g := proof_composed_from_rewrites
```

is created.

For simplification, Phase 4 may define `simp-lite` output as follows.

```text
1. simplify the target to target'
2. if target' is Eq t t, use Eq.refl t
3. create a proof of equivalence between target and target' as an Eq.subst chain
```

## 8.5 Termination

`simp-lite` must avoid infinite loops.

Dangerous rule:

```text
x = x + 0
```

Using this left-to-right makes the expression larger.

In the Phase 4 MVP, restrict simp rules at registration time.

```text
- allow if lhs size > rhs size
- or allow only rules explicitly marked safe
- stop if the same target hash is revisited
- set a maximum rewrite count
```

## 8.6 Minimal Rules For simp-lite

Initially, these are sufficient.

```text
Nat.add_zero
Nat.zero_add
Nat.add_succ
Nat.succ_add
Nat.mul_zero
Nat.zero_mul
Eq.refl-related
βζδ reduction
```

However, if `Nat.add` is defined by recursion on the second argument:

```text
n + 0
```

then βδι reduction alone turns it into `n`, so even `Nat.add_zero` may be
unnecessary.

---

# 9. `induction`

## 9.1 Role

`induction n` performs induction on the inductive value `n`.

Example:

```npa
theorem zero_add (n : Nat) : 0 + n = n := by
  induction n
  case zero =>
    exact Eq.refl 0
  case succ n ih =>
    simp-lite
```

Initial goal:

```text
n : Nat
⊢ 0 + n = n
```

After `induction n`:

```text
case zero:
  ⊢ 0 + 0 = 0

case succ:
  n : Nat
  ih : 0 + n = n
  ⊢ 0 + succ n = succ n
```

## 9.2 core proof term

Internally, `induction n` uses `Nat.rec`.

If the target has the form `P n`:

```text
Nat.rec
  motive
  base_case
  step_case
  n
```

is used.

Here:

```text
motive := λ n : Nat, target_with_n
```

Example:

```text
target:
  Eq Nat (Nat.add Nat.zero n) n
```

then:

```text
motive :=
  λ n : Nat, Eq Nat (Nat.add Nat.zero n) n
```

base case：

```text
motive Nat.zero
```

step case：

```text
Π n : Nat, motive n → motive (Nat.succ n)
```

## 9.3 Algorithm

```text
goal:
  context Γ, n : Nat
  target T

induction n:
  1. create motive P := λ n, T from target T
  2. create base goal:
       Γ ⊢ P Nat.zero
  3. create step goal:
       Γ, n : Nat, ih : P n ⊢ P (Nat.succ n)
  4. assign Nat.rec P ?base ?step n to the old goal
```

Pseudocode:

```rust
fn tactic_induction_nat(
    state: &mut ProofState,
    goal_id: GoalId,
    var_name: NameId,
) -> Result<()> {
    let goal = state.get_goal(goal_id);
    let local = goal.context.lookup(var_name)?;

    ensure_type_is_nat(local.ty)?;

    let motive = abstract_over_local(goal.target, local);
    // motive : Nat -> Sort u

    let base_target = mk_app(motive, Nat.zero);
    let base_goal = state.new_goal(goal.context.without_or_with(local), base_target);

    let n_local = fresh_local("n", Nat);
    let ih_ty = mk_app(motive, n_local.expr);
    let ih_local = fresh_local("ih", ih_ty);

    let step_target = mk_app(motive, mk_app(Nat.succ, n_local.expr));
    let step_ctx = goal.context
        .replace_or_extend(local, n_local)
        .add(ih_local);

    let step_goal = state.new_goal(step_ctx, step_target);

    let proof = mk_nat_rec(
        motive,
        mk_mvar(base_goal),
        mk_lam(n_local, mk_lam(ih_local, mk_mvar(step_goal))),
        local.expr,
    );

    state.assign_goal(goal_id, proof);
    state.replace_goal_with_many(goal_id, vec![base_goal, step_goal]);

    Ok(())
}
```

## 9.4 Important Issue: Whether The Target Depends On The Variable

In `induction n`, the `n` in the target is abstracted to create the motive.

```text
target:
  0 + n = n

motive:
  λ k : Nat, 0 + k = k
```

If this abstraction is not done correctly, induction breaks.

Therefore, in the Phase 4 MVP, it is best to first limit support to `Nat` and
only handle cases where the target variable is directly present in the local
context.

Supported:

```text
n : Nat
⊢ P n
```

Not supported yet:

```text
f n : Nat
⊢ P (f n)
```

Complex dependent induction is postponed.

## 9.5 Handling Assumptions In The Context

For example:

```text
n : Nat
h : Q n
⊢ P n
```

if `induction n` is performed, `h` also depends on `n`.

Full support requires dependent context abstraction.

The Phase 4 MVP simplifies this.

```text
restriction:
  fail when a later assumption depends on the induction target n
```

Error:

```text
cannot perform simple induction on `n`
because hypothesis `h` depends on `n`.

Hint:
  generalize or revert dependent hypotheses first.
```

In the future, add `revert`, `generalize`, and dependent induction.

---

# 10. tactic parser

Phase 4 also needs a simple tactic script parser. This parser is only for Human
source. It is implemented separately from the AI-facing `MachineTacticCandidate`
JSON parser and Machine term canonicalizer, and is not called during Machine
tactic batch execution.

```npa
by
  intro n
  exact n
```

Minimal grammar:

```text
tactic_script ::=
  "by" tactic_seq

tactic_seq ::=
  tactic*

tactic ::=
    "intro" ident
  | "exact" term
  | "apply" term
  | "rw" "[" rw_rule_list "]"
  | "simp-lite"
  | "induction" ident
```

Examples of `rw`:

```text
rw [h]
rw [<- h]
rw [Nat.add_zero]
```

In Phase 4, case syntax may be simplified.

```npa
induction n
exact Eq.refl 0
simp-lite
```

However, when there are multiple goals, use the rule that tactics always apply
to the first goal.

```text
current goals = [g1, g2, g3]
tactic applies to g1
```

In the future:

```npa
case zero =>
  ...
case succ n ih =>
  ...
```

will be introduced.

---

# 11. Overall Tactic Execution Flow

This section describes the flow for elaborating `by ...` proof blocks in Human
source. The AI / Machine path uses the same proof-state primitives after
validation of structured `MachineTacticCandidate`, but does not go through the
Human tactic parser or Human name resolution.

The implementation responsibilities are split as follows.

```text
npa-frontend plain compile wrapper
  - Lower Human terms / notation / implicits / source interfaces to core.
  - Do not execute tactic blocks by itself for `by` proof blocks.
  - Core-ize theorems containing `by` proofs only when a checked core proof is passed from the npa-api adapter.

npa-api Human API wrapper
  - Receive current module / current source / verified imports / imported Human source interfaces / options as explicit input.
  - Collect `by` proof targets and run Machine proof-state primitives through the Human tactic bridge.
  - Return the extracted core proof term to npa-frontend to create the core module / certificate.
  - Do not implicitly create `/machine/*` endpoints or `create_machine_session`.
```

With this separation, the Human API can pass source containing `by` proofs all
the way to certificates, while the Machine API request grammar is not widened to
accept Human tactic text.

```text
1. if the theorem right-hand side is `by ...`, enter proof mode
2. create initial goal ?g from the theorem type
3. execute tactics sequentially
4. each tactic assigns a goal and creates new goals
5. if there are zero goals at the end, extract the proof term
6. check proof : theorem_type with the kernel
7. save it in the certificate
```

Pseudocode:

```rust
fn elaborate_by_proof(theorem_type: ExprId, tactics: Vec<TacticSyntax>) -> Result<ExprId> {
    let mut state = ProofState::new(theorem_type);

    for tac in tactics {
        let goal = state.current_goal()
            .ok_or(Error::NoGoalsButTacticRemaining)?;

        run_tactic(&mut state, goal, tac)?;
    }

    if !state.goals.is_empty() {
        return Err(Error::UnsolvedGoals(state.goals));
    }

    let proof = state.extract_root_proof()?;

    kernel_check(proof, theorem_type)?;

    Ok(proof)
}
```

---

# 12. Relationship Between The Six Tactics

| tactic      | What It Does           | proof term                     |
| ----------- | ---------------------- | ------------------------------ |
| `intro`     | decomposes a `Π` target and adds an assumption | `λ x, ?g`                      |
| `exact`     | directly closes a goal with a term | `t`                            |
| `apply`     | uses a theorem/assumption and turns premises into subgoals | `f ?a ?b ...`                  |
| `rw`        | rewrites the target by equality | `Eq.subst` / `Eq.rec`          |
| `simp-lite` | simplifies and closes with registered rewrites | rewrite proof chain            |
| `induction` | splits cases by induction | `Nat.rec motive ?base ?step n` |

---

# 13. Minimal Tests For Phase 4

The examples in this chapter are fixed as regression fixtures in
`crates/npa-api/src/human.rs`:
`phase4_human_section13_minimal_certificate_fixtures_compile` and
`phase4_human_section13_rw_and_induction_certificate_fixtures_compile`.
The fixtures generate certificates from Human `by` source, and `rw` /
`induction` also recheck extracted proof terms with canonical certificates and
the verifier. At the same time, Machine Surface fixtures in
`crates/npa-frontend/src/term_source.rs` confirm that Human-only tactic syntax
does not enter Machine canonical term source.

## 13.1 `intro` + `exact`

```npa
theorem id_nat : Nat → Nat := by
  intro n
  exact n
```

Expected core proof:

```text
λ n : Nat, n
```

## 13.2 `Eq.refl`

```npa
theorem self_eq (n : Nat) : n = n := by
  exact Eq.refl n
```

Checks:

```text
implicit arg Nat is filled
exact closes the goal
```

## 13.3 `apply`

```npa
theorem use_id (n : Nat) : Nat := by
  apply id
  exact Nat
  exact n
```

If implicit arguments of `id` can be used:

```npa
theorem use_id (n : Nat) : Nat := by
  apply id
  exact n
```

## 13.4 `rw`

```npa
theorem rw_test (a b : Nat) (h : a = b) : a = a := by
  rw [h]
  exact Eq.refl b
```

Depending on the implementation policy, the target after `rw [h]` is:

```text
b = b
```

.

## 13.5 `simp-lite`

```npa
theorem add_zero (n : Nat) : n + 0 = n := by
  simp-lite
```

Expected:

```text
n + 0 simplifies to n
the target becomes n = n
it closes with Eq.refl n
```

## 13.6 `induction`

```npa
theorem zero_add (n : Nat) : 0 + n = n := by
  induction n
  exact Eq.refl 0
  simp-lite
```

Expected:

```text
base:
  0 + 0 = 0

step:
  n : Nat
  ih : 0 + n = n
  ⊢ 0 + succ n = succ n
```

---

# 14. Things Not Yet Included In Phase 4

In the Phase 4 MVP, postpone the following.

```text
- constructor
- cases
- refine
- have
- specialize
- assumption
- contradiction
- calc
- case syntax
- rewrite in hypotheses
- occurrence selection
- full dependent induction
- typeclass-driven apply
- full simp
- ring / omega / linarith
```

In particular, `rw` and `induction` are harder than they look. It is best to
first build a **safe minimal version limited to Nat and Eq**.

---

# 15. Phase 4 Completion Criteria

Phase 4 can be considered complete when the following conditions hold.

```text
- `by` proof blocks can be parsed
- ProofState / Goal / MetaVar are implemented
- tactics can be applied to the current goal
- `intro` can convert Pi targets to lambdas
- `exact` can close goals with terms
- `apply` can use theorems/assumptions to create subgoals
- `rw` can rewrite targets by Eq
- `simp-lite` can close goals by simple rewrite and refl
- `induction` can create base/step goals by Nat.rec
- proof terms after tactics can be checked by the kernel
- certificate generation can be rejected when unresolved goals remain
- the Human tactic parser / bridge is not in the hot path of the AI-facing Machine tactic API
- regressions pass showing that Phase 7 / Phase 9 continue to use the Machine API without going through Human source
- Machine tactic canonical hash / cache key / budget inputs do not change when Human syntax is added
```

---

In one sentence, Phase 4 Human is **the layer that converts small,
human-friendly proof commands into core proof terms that the kernel can
check**. The equivalent AI-facing functionality is implemented as Machine
Tactic in `develop/phase4-ai.md`, on a fast path that does not go through the
Human parser.

The first implementation order should be:

```text
1. exact
2. intro
3. apply
4. rw
5. simp-lite
6. induction
```

With `exact` and `intro`, the basics of proof state are fixed; `apply` adds
subgoal generation; `rw` and `simp-lite` make equality reasoning usable; and
finally `induction` reaches induction.
