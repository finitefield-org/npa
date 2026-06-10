# Phase 4 Human Task Breakdown

This task breakdown treats `develop/phase4-human.md` as authoritative and
splits the gap from the current `crates/npa-frontend` / `crates/npa-tactic` /
`crates/npa-api` implementation into implementation milestones.

Phase 4 Human is the layer for humans to write `by ...` blocks and tactic
scripts. The fast, structured Machine Tactic for AI is the responsibility of
`develop/phase4-ai.md`; Human parser / resolver / notation / implicit insertion
must not be mixed into the AI fast path.

Important constraints:

```text
- Do not put the Human tactic parser into the Machine tactic hot path in `crates/npa-tactic`.
- Do not add Human-only metadata to `MachineTacticCandidate`, `MachineTactic`, or `MachineTermSource`.
- Do not extend `parse_machine_*` and `canonicalize_machine_term_source` to accept Human syntax.
- The Machine APIs for Phase 5 / Phase 7 / Phase 9 do not go through Human source.
- Tactics are not trusted; the final proof term / certificate is checked by the kernel and verifier.
- Do not put I/O, network, plugin loading, AI calls, or search import lookup into the Human bridge.
```

---

## 0. Current Implementation Boundary

### 0.1 Items Treated As Implemented

The current `crates/npa-frontend` has the foundation for Phase 3 Human Surface.
The Phase 4 Human implementation may use these as assumptions.

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanExpr
- HumanFrontendState
- HumanSourceInterface / HumanImportedSourceInterface
- HumanExpr::Hole
- HumanCompileOptions

crates/npa-frontend/src/human_parser.rs
- parse_human_module
- parse_human_module_with_source_interfaces
- parse_human_term
- import / open / namespace / end
- def / theorem / axiom / inductive / notation
- grouped binders, implicit binders, holes, notation application

crates/npa-frontend/src/human_resolver.rs
- namespace / open scope resolution
- source interface reconciliation
- imported Human metadata lookup
- ambiguity and forward-reference diagnostics

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core
- compile_human_source_to_certificate
- Human term elaboration with implicit insertion / simple metas / holes
- certificate handoff that rejects unresolved holes

crates/npa-api/src/human.rs
- Human compile-to-core API wrapper
- Human compile-to-certificate API wrapper
- Machine session API remains Machine Surface only
```

Important dependencies:

```text
npa-tactic -> npa-frontend
npa-api -> npa-frontend + npa-tactic
```

Therefore, an implementation that directly calls `npa-tactic` from
`crates/npa-frontend` creates a cyclic dependency. Put the Human tactic bridge
in a Human-only adapter inside `crates/npa-api`, or in a new adapter crate that
depends on both `npa-frontend` and `npa-tactic`. `crates/npa-frontend` provides
Human AST / parser / resolver / term elaboration helpers, and proof-state
execution happens in the adapter layer.

The current `crates/npa-tactic` already implements the proof-state core for
AI-facing Machine tactics and six tactics.

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- start_machine_proof
- validate_machine_tactic_candidate
- run_machine_tactic_with_budget
- run_machine_tactic_candidates_batch
- assign_goal / ProofExpr
- extract_closed_machine_proof
- extract_closed_machine_theorem_decl
- extract_closed_machine_core_module
- extract_closed_machine_certificate
- exact / intro / apply / rw / simp-lite / induction-nat
- deterministic budget / tactic hash / cache key
```

### 0.2 Unimplemented Human Tactic Scope

The following scope required by `develop/phase4-human.md` does not currently
exist in the code as a Human Profile.

```text
HumanTacticScript
HumanTacticSyntax
HumanRewriteRuleSyntax
HumanProofBlock / by block AST
syntax for parsing `by ...` as a theorem value
Human parser for intro / exact / apply / rw / simp-lite / induction
Human tactic script executor
Human proof state bridge
bridge for elaborating Human source terms in the tactic goal context
implementation for closing goals from Human `exact`
implementation for introducing binder locals from Human `intro`
implementation for generating subgoals from Human `apply`
implementation for target rewrite from Human `rw`
implementation for calling the proof-producing simplifier from Human `simp-lite`
implementation for creating Nat.rec base/step goals from Human `induction n`
Human-facing tactic diagnostics / goal display
regression for compiling / certificate-generating theorems containing by proofs from the Human API
```

### 0.3 Things That Must Not Enter The Machine Fast Path

The following may be used in the Human tactic implementation, but must not
enter the high-frequency candidate checking path for Machine tactics.

```text
Human tactic text parser
Human `by` proof block AST
Human name shortening
open / namespace scope
notation table
implicit argument insertion
hole / named hole metadata
Human source spans
Human diagnostics
Human goal pretty text
case syntax
backtracking tactic language
filesystem / network import lookup
```

---

## 1. Design Rules For Protecting The AI-Facing Fast Path

For each Phase 4 Human milestone, treat the following as acceptance criteria.

```text
- The public Machine API in `crates/npa-tactic` does not call the Human parser.
- `/machine/tactics/run` and `/machine/tactics/batch` use only `MachineTacticCandidate` as their main input.
- The Human `by` parser is confined to `crates/npa-frontend` or a Human-only adapter.
- `crates/npa-frontend` does not depend on `npa-tactic`. Put the proof-state bridge in `npa-api` or a new adapter crate.
- Do not make pretty-printing Human terms to Machine Surface source and reparsing the default implementation.
- Human terms are lowered to core Expr by the Human elaborator, or explicitly converted to a Machine-compatible form.
- Do not change Machine Surface accepted / rejected syntax or canonical hashes.
- Do not put tactic metadata, source spans, or Human display names into certificates.
- Adding the Human tactic bridge does not change Phase 7 / Phase 9 regression fixtures.
```

Recommended structure:

```text
Human source:
  parse_human_module
  -> Human theorem with HumanProofBlock
  -> Human resolver / Human elaborator
  -> Human-only tactic adapter outside npa-frontend
  -> npa_tactic proof-state primitive or MachineTactic where safe
  -> extract closed core proof
  -> kernel / certificate

AI path:
  MachineTacticCandidate JSON
  -> validate_machine_tactic_candidate
  -> run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
  -> extract closed core proof
  -> kernel / certificate
```

---

## 2. Implementation Order

Phase 4 Human is implemented in an order that makes impact on the Machine fast
path easy to detect.

```text
1. Fix the Human / Machine tactic boundary and regression guard
2. Add the Human tactic AST and by block parser
3. Create the Human proof-state bridge skeleton
4. Close the minimal proof script with exact / intro
5. Pass subgoal generation with apply
6. Pass Eq target rewrite and reduction with rw / simp-lite
7. Pass Nat.rec base/step goals with induction
8. Integrate script executor / diagnostics / API / certificate handoff
9. Fix Machine fast path regression and doc consistency
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib human_parser
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-frontend --lib machine_surface
cargo test -p npa-tactic
cargo test -p npa-api phase7
cargo test -p npa-api phase9
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 3. Task List

### P4H-00: Fix The Human / Machine Tactic Boundary

Status: Done

Depends on: None

Inputs:

```text
develop/phase4-human.md section 0
develop/phase4-ai.md section 2, 3, 6, 13
develop/phase5-ai.md MachineTacticCandidate wire schema
crates/npa-tactic/src/lib.rs
crates/npa-api/src/tactic.rs
crates/npa-api/src/phase7.rs
crates/npa-api/src/phase9.rs
```

Implementation tasks:

- Fix the policy that the Human tactic implementation entrypoint lives in an adapter inside `crates/npa-api` or a new adapter crate.
- Fix a regression ensuring `crates/npa-frontend` does not add an `npa-tactic` dependency.
- Add a regression test ensuring the Machine API in `crates/npa-tactic` does not add a Human parser dependency.
- Preserve Human syntax rejected snapshots for `parse_machine_*` / `canonicalize_machine_term_source`.
- Fix in test names that Phase 7 / Phase 9 continue to use `MachineTacticCandidate` and Machine Surface canonicalization.

AI Speed Guard:

- `rg -n "parse_human|compile_human|Human" crates/npa-tactic/src/lib.rs crates/npa-api/src/tactic.rs crates/npa-api/src/adapter.rs` has no production hits.
- `cargo test -p npa-frontend --lib machine_surface` passes.
- `cargo test -p npa-api phase7` and `cargo test -p npa-api phase9` pass.

Completion criteria:

- The implementation location of the Human tactic bridge and the Machine API independence boundary are clear in code.
- Machine candidate validation / batch execution has no Human parser fallback.
- Machine Surface canonical bytes / hash golden behavior has not changed.

Completion confirmation:

- Human API tests in `npa-api` fixed that the Human tactic bridge lives in `npa-api` or a new adapter crate and does not add an `npa-frontend -> npa-tactic` dependency.
- Machine tactic API tests in `npa-api` and `npa-tactic` tests fixed that Human parser / compiler fallback markers do not enter the Machine hot path.
- Existing Machine Surface rejected syntax snapshots and Phase 7 / Phase 9 regression are preserved.

### P4H-01: Add The Human Tactic AST

Status: Done

Depends on: P4H-00

Inputs:

```text
develop/phase4-human.md section 10, 12
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_diagnostic.rs
```

Implementation tasks:

- Add `HumanProofBlock` or an equivalent type and preserve `by` blocks separately from theorem values.
- Add `HumanTacticScript` / `HumanTacticSyntax`.
- Limit tactic variants to `Intro`, `Exact`, `Apply`, `Rewrite`, `SimpLite`, `Induction`.
- Add a Human AST for `rw` that represents forward / backward direction and rule term lists.
- Preserve source spans, but do not put them into hash / certificate payloads.
- Do not put case syntax, constructor, refine, have, or calc into the AST.

AI Speed Guard:

- Do not add Human variants or Human span fields to `MachineTacticCandidate` / `MachineTactic`.
- Do not import `HumanTacticSyntax` into `crates/npa-tactic`.

Completion criteria:

- Human tactic scripts can be represented as a typed AST.
- Human AST and Machine tactic AST are separated at the type level.
- Unsupported tactics can be clearly rejected during parse/diagnostic handling.

Completion confirmation:

- Added `HumanDeclValue::{Term, ProofBlock}`, `HumanProofBlock`, `HumanTacticScript`, and `HumanTacticSyntax` to `npa-frontend`, separating `by` blocks from ordinary term values at the type level.
- The tactic AST represents only the six MVP tactics `intro` / `exact` / `apply` / `rw` / `simp-lite` / `induction`, and `rw` rules preserve forward / backward direction and rule term span.
- Added `HumanDiagnosticKind::UnsupportedTactic`; as of P4H-01, the proof block resolver rejects the unimplemented tactic bridge as a structured diagnostic.
- `MachineTacticCandidate` / `MachineTactic` and `npa-tactic` were not changed.

### P4H-02: Implement The `by` Proof Block Parser

Status: Done

Depends on: P4H-01

Inputs:

```text
develop/phase4-human.md section 10
crates/npa-frontend/src/human_parser.rs
```

Implementation tasks:

- Add `by`, `intro`, `exact`, `apply`, `rw`, `simp-lite`, and `induction` to the Human lexer.
- Allow parsing either a term or a `by` proof block as the right-hand side of `:=` in theorem declarations.
- Parse `intro ident`, `exact term`, `apply term`, `rw [rule]`, `rw [<- rule]`, `simp-lite`, and `induction ident`.
- Preserve multiple tactics in source order.
- In the MVP, do not treat indentation semantically; read it deterministically as a token sequence.
- Reject `case zero =>` / `case succ =>` in the Phase 4 Human MVP.

AI Speed Guard:

- Do not add `by` / tactic keywords to the Machine parser.
- Include `by`, `intro`, `rw [h]`, `simp-lite`, and `induction n` in Machine Surface rejected syntax fixtures.

Completion criteria:

- `theorem id_nat : Nat -> Nat := by intro n exact n` can be parsed into the Human AST.
- Unsupported tactics / malformed `rw` / trailing tactic tokens become Human parse diagnostics.
- The Machine parser continues to reject the same input.

Completion confirmation:

- Added a `by` proof block parser to the Human parser, generating `HumanDeclValue::ProofBlock` on the right-hand side of theorem `:=`.
- Parses `intro ident` / `exact term` / `apply term` / `rw [rule]` / `rw [<- rule]` / `simp-lite` / `induction ident` as `HumanTacticScript` in source order.
- In the MVP, indentation is not treated semantically, and `case` / unsupported tactics / malformed `rw` / trailing tokens are rejected as parser-phase diagnostics.
- Machine Surface rejected syntax fixtures include `by intro ... exact ...`, `rw [h]`, `simp-lite`, and `induction n`, and tactic keywords were not added to the Machine parser.

### P4H-03: Create The Human Proof-State Bridge Skeleton

Status: Done

Depends on: P4H-02

Inputs:

```text
develop/phase4-human.md section 1, 2, 3, 11
crates/npa-frontend/src/human_elaborator.rs
crates/npa-api/src/human.rs
crates/npa-tactic/src/lib.rs
```

Implementation tasks:

- Decide whether the bridge lives in a Human-only module inside `crates/npa-api` or in a new adapter crate.
- Do not add an `npa-tactic` dependency to `crates/npa-frontend`.
- If necessary, add only small public helpers for lowering Human theorem types / terms from `npa-frontend` to core Expr.
- Elaborate the Human theorem type first and create a fully explicit core type that can be passed to `MachineProofSpec`.
- Turn prior current declarations into a checked chain with `check_current_decl_for_machine_tactic`.
- Assemble `VerifiedImportRef` and the Human lookup context from verified imports / source interfaces.
- Add a Human-only bridge that calls `start_machine_proof`.
- If options for the Human bridge are added, keep them inside `HumanCompileOptions`; `MachineTacticOptions` uses the existing canonical rules.
- The Human bridge does not perform filesystem / network / package registry lookup.

AI Speed Guard:

- Do not change the signature or validation order of `start_machine_proof` for Human convenience.
- Even if the Human bridge calls the Machine batch API, do not make the AI batch hot path depend back on it.
- The workspace dependency graph does not grow an `npa-frontend -> npa-tactic` edge.

Completion criteria:

- Proof state for Human theorems can be started deterministically.
- The theorem type of the root goal is checked by the kernel as a Sort.
- The state fingerprint created by the Human bridge passes Machine state validation.
- `cargo metadata` or `Cargo.toml` inspection shows no cyclic dependency.

Completion confirmation:

- The bridge is placed as the Human-only API `start_human_proof` in `crates/npa-api/src/human.rs`, and no `npa-tactic` dependency was added to `crates/npa-frontend`.
- Added only `prepare_human_proof_start_core_with_source_interfaces` to `npa-frontend`; it core-izes the target theorem type and prior current declarations from Human source. It does not execute tactics or call `npa-tactic`.
- Human current declaration names are projected at the bridge boundary to current-module-prefixed names for Machine proof state, and imported names and the Machine Surface canonicalizer were not changed.
- `start_human_proof` assembles `VerifiedImportRef` from active Human imports / source interfaces, makes prior current declarations into a checked chain with `check_current_decl_for_machine_tactic_from_verified_imports`, then calls `start_machine_proof`.
- The generated proof state passes `validate_machine_proof_state`, and the root goal theorem type is left to the existing kernel Sort check in `start_machine_proof`.
- As AI speed guards, the signature / validation order of `start_machine_proof`, Machine batch API, `npa-tactic` hot path, and Machine Surface parser were not changed.

### P4H-04: Implement The Human Tactic Term Elaboration Context

Status: Done

Depends on: P4H-03

Inputs:

```text
develop/phase4-human.md section 5, 6, 7
crates/npa-frontend/src/human_elaborator.rs
crates/npa-tactic/src/lib.rs
```

Implementation tasks:

- Convert the current goal's local context into a form readable by the Human term elaborator.
- Add helpers that can check / infer Human source terms against the goal target.
- Allow resolution of local binders, checked current declarations, verified imports, and generated constructors / recursors.
- Reject unsolved holes / synthetic implicits from Human term elaboration as tactic failures before certificate generation.
- Do not make pretty-stringifying Human terms and feeding them back to Machine Surface the default implementation.

AI Speed Guard:

- Do not add Human open scope / notation lookup to the Machine term elaboration context.
- Do not put Human context conversion results into Machine tactic cache keys.

Completion criteria:

- `exact x` inside tactics can resolve the local `x` in the goal context.
- `exact Eq.refl n` inside tactics can use Human implicit insertion.
- Unresolved holes are returned as Human diagnostics and are not certificate-generated.

### P4H-05: Implement `exact`

Status: Done

Depends on: P4H-04

Inputs:

```text
develop/phase4-human.md section 5, 13.1, 13.2
crates/npa-tactic/src/lib.rs assign_goal / ProofExpr
```

Implementation tasks:

- Check `exact term` against the current goal target with the Human elaborator.
- Pass the checked core Expr to `assign_goal` or an equivalent proof-state primitive as `ProofExpr::Core`.
- Fail `exact _` and exact expressions containing unresolved metas.
- Remove closed goals from open_goals.

AI Speed Guard:

- Do not change the semantics of `MachineTactic::Exact` or `RawMachineTerm`.
- Do not make `MachineTermSource` accept Human syntax for Human exact.

Completion criteria:

- The exact part of `theorem id_nat : Nat -> Nat := by intro n exact n` closes the goal using the local.
- `theorem self_eq (n : Nat) : n = n := by exact Eq.refl n` closes.
- `exact _` is rejected conservatively.

### P4H-06: Implement `intro`

Status: Done

Depends on: P4H-03

Inputs:

```text
develop/phase4-human.md section 4, 13.1
crates/npa-tactic/src/lib.rs run_machine_tactic_with_budget
```

Implementation tasks:

- Apply `intro name` to the current goal.
- If Machine `Intro` can be reused, call `MachineTactic::Intro` from the Human bridge.
- If the target is not Pi / forall, map it to a Human-facing diagnostic.
- Reject local name shadowing / invalid binder names deterministically.

AI Speed Guard:

- Do not change the candidate hash / proof delta hash of `MachineTactic::Intro`.
- Do not put Human display names into state fingerprints.

Completion criteria:

- `intro n` creates the body goal for `Nat -> Nat`.
- Targets where `intro` cannot be used return a structured error.
- Combined with exact after `intro`, `id_nat` closes.

### P4H-07: Implement The Human Tactic Script Executor

Status: Done

Depends on: P4H-05, P4H-06

Inputs:

```text
develop/phase4-human.md section 10, 11
crates/npa-frontend/src/human_elaborator.rs
```

Implementation tasks:

- Execute `HumanTacticScript` sequentially in source order.
- Always apply tactics to the first open goal.
- If tactics remain when there are no goals, return a Human diagnostic equivalent to `NoGoalsButTacticRemaining`.
- If open goals remain at script end, return an unresolved goal diagnostic.
- Extract the closed proof with `extract_closed_machine_proof` or an equivalent API.

AI Speed Guard:

- Do not change Machine batch execution policy.
- Do not put the Human sequential executor into the implementation of `/machine/tactics/batch`.

Completion criteria:

- A two-line script with `intro` + `exact` can be closed.
- Extra tactics and unresolved goals are distinguished as Human diagnostics.
- The extracted proof term passes kernel check.

### P4H-08: Implement `apply`

Status: Done

Depends on: P4H-04, P4H-07

Inputs:

```text
develop/phase4-human.md section 6, 13.3
crates/npa-tactic/src/lib.rs run_apply_tactic_with_budget behavior
```

Implementation tasks:

- In the MVP for `apply term`, handle resolved local / global heads.
- Resolve the head with Human name resolution and lower it to Machine `TacticHead` or a Human-only apply helper.
- Do not make implicit / inferable arguments into subgoals; make explicit, proof-relevant premises into subgoals.
- Return a structured diagnostic if the target and conclusion cannot be unified.
- Explicitly reject arbitrary term heads and complex expression apply as out of scope for the MVP, or split them into another task.

AI Speed Guard:

- Do not widen the Machine `Apply` candidate schema for Human term expressions.
- Do not mix Human apply name resolution into Machine tactic head lookup.

Completion criteria:

- A local assumption or checked theorem can be applied to create subgoals.
- Subgoals after `apply` can be closed by following `exact`.
- On apply failure, a summary of target / head type can be returned as a Human diagnostic.

### P4H-09: Implement `rw`

Status: Done

Depends on: P4H-04, P4H-07

Inputs:

```text
develop/phase4-human.md section 7, 13.4
crates/npa-tactic/src/lib.rs rewrite implementation
```

Implementation tasks:

- Execute `rw [h]` and `rw [<- h]` from parsed Human rules.
- The MVP rewrites only the target.
- Support only Eq, and reject setoid rewrite / hypothesis rewrite / occurrence selection.
- Resolve rule heads from locals / checked theorems.
- Generate the rewritten target goal and Eq.rec / Eq.subst transport proof.

AI Speed Guard:

- Do not change `RewriteSite` / `RewriteDirection` semantics for Machine `Rewrite`.
- Do not mix Human notation-based rule lookup into the Machine simp/rw registry.

Completion criteria:

- `rw [h]` can rewrite the Eq side inside the target.
- Reverse rewrite works deterministically.
- Dependent rewrite and unsupported sites are rejected with Human diagnostics.

### P4H-10: Implement `simp-lite`

Status: Done

Depends on: P4H-07

Inputs:

```text
develop/phase4-human.md section 8, 13.5
crates/npa-tactic/src/lib.rs simp-lite implementation
```

Implementation tasks:

- Apply `simp-lite` to the current goal.
- In the Phase 4 Human MVP, reuse existing Machine `SimpLite` and the state's `SimpRegistry`.
- Do not add Human source-level simp attributes / custom simp sets.
- Fix as MVP policy that failure to close is a failure.
- Rewrite limit / target hash revisit / max steps follow existing `MachineTacticOptions`.

AI Speed Guard:

- Do not change canonical bytes for `MachineTacticOptions.simp_rules`.
- Do not implicitly add simp rules from the Human notation table.

Completion criteria:

- `simp-lite` can close reflexive Eq targets.
- It can reduce targets using registered safe rules and produce a proof-producing chain.
- Deterministic failure from max rewrite steps is preserved.

### P4H-11: Implement `induction`

Status: Done

Depends on: P4H-07, P4H-10

Inputs:

```text
develop/phase4-human.md section 9, 13.6
crates/npa-tactic/src/lib.rs induction-nat implementation
```

Implementation tasks:

- Convert Human syntax `induction n` to Nat-only Machine `InductionNat` or an equivalent helper.
- Limit targets to Nat locals directly present in the local context.
- Fail if there are dependent assumptions after the induction target.
- Generate base goal / step goal in deterministic order.
- Reject `case zero =>` / `case succ =>` as out of scope for the MVP.

AI Speed Guard:

- Keep the Machine wire name as `induction-nat`.
- Do not add the Human `induction` keyword to Machine API allowed_tactics.

Completion criteria:

- `induction n` creates the two goals base / step.
- Base and step can be closed by following `exact` / `simp-lite`.
- Unsupported dependent induction is rejected with a Human diagnostic.

### P4H-12: Integrate Core / Certificate Handoff For By Proofs

Status: Done

Depends on: P4H-07, P4H-08, P4H-09, P4H-10, P4H-11

Inputs:

```text
develop/phase4-human.md section 11, 15
crates/npa-frontend/src/human_elaborator.rs
crates/npa-cert/src/canonical.rs
```

Implementation tasks:

- Branch the compile path between theorem values that are Human terms and those that are Human `by` proofs.
- Insert the extracted proof Expr from the `by` proof as the theorem declaration value.
- While the standalone `npa-frontend` compile path cannot handle by proofs, use an explicit diagnostic to direct users to the adapter path.
- Do not put proof state / tactic trace / Human spans / diagnostics into CoreModule or certificate payloads.
- Reject certificate generation if even one unresolved goal remains.
- Recheck closed proofs with the existing kernel / certificate verifier.

AI Speed Guard:

- Do not add Human tactic metadata fields to certificate canonical encoding.
- Do not change Machine certificate handoff fixtures.

Completion criteria:

- The Human tactic adapter compile-to-core path can convert `by` theorems to core theorem declarations.
- The Human tactic adapter compile-to-certificate path can generate and verify certificates containing `by` theorems.
- If existing `npa_frontend::compile_human_source_to_core` / `compile_human_source_to_certificate` are changed, this is done with no cyclic dependency.
- Unresolved goals / unsupported tactics stop before certificate construction.

### P4H-13: Arrange Human Tactic Diagnostics And Goal Display

Status: Done

Depends on: P4H-07

Inputs:

```text
develop/phase4-human.md section 3, 4, 5, 6, 9
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-api/src/human.rs
```

Implementation tasks:

- Distinguish tactic parse / validation / execution / unresolved goal with HumanDiagnosticPhase.
- Return the current goal context / target as a Human-friendly payload.
- Add a helper that maps MachineTacticDiagnostic to Human diagnostics.
- Error messages may be human-facing, but decisions are made by enum / structured payload.
- Do not include tactic traces or AI metadata in diagnostic payloads.

AI Speed Guard:

- Do not change MachineApiDiagnostic canonicalization with Human diagnostics.
- Do not put Human goal display into Machine state fingerprints / tactic cache keys.

Completion criteria:

- Diagnostics for `intro` non-Pi, `exact` type mismatch, `apply` mismatch, and unsupported induction are distinguished.
- Tests can directly assert diagnostic kind and payload.
- Changes to display text do not change hash / replay.

### P4H-14: Make By Proofs Usable From The Human API

Status: Done

Depends on: P4H-12, P4H-13

Inputs:

```text
develop/phase4-human.md section 3, 11, 15
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Implementation tasks:

- Pass by proofs through the Human API wrapper. The implementation target is the public API of the `npa-api` adapter or a new adapter crate.
- Specify the responsibility split with the existing plain Human compile wrapper.
- Preserve explicit current module/source/imports/options inputs in the Human API request shape.
- Do not implicitly create Machine sessions from the Human API.
- Add `by intro n exact n` and `by exact Eq.refl n` to API tests.

AI Speed Guard:

- Do not change request grammar for `/machine/*` endpoints.
- `create_machine_session` continues to reject `by` source as a Machine term parse error.

Completion criteria:

- The Human API can create certificates from source containing by proofs.
- Regression passes showing that the Machine API does not accept Human tactic text.
- Imported Human source interfaces and by proofs can be used together.

### P4H-15: Fix Minimal Examples And Regression Fixtures

Status: Done

Depends on: P4H-14

Inputs:

```text
develop/phase4-human.md section 13, 14, 15
README.md
develop/phase4-ai.md
```

Implementation tasks:

- Add an id theorem fixture for `intro` + `exact`.
- Add an `Eq.refl` exact fixture.
- Add an `apply` fixture.
- Add an `rw` fixture.
- Add a `simp-lite` fixture.
- Add an `induction` fixture.
- Add regressions for the unsupported features list.
- Update README or docs if the Phase 4 Human implementation status needs to be reflected.

AI Speed Guard:

- Confirm that adding Human examples does not change Phase 7 / Phase 9 fixture hashes.
- Do not update Machine Surface rejected Human feature snapshots, or add only intentional rejections.

Completion criteria:

- The minimal tests from `develop/phase4-human.md` section 13 exist as code regressions.
- Items from section 14 "things not added yet" are rejected by parser / diagnostics.
- There is a test set that satisfies the Phase 4 Human completion conditions.

---

## 4. Completion Gate

The whole of Phase 4 Human can be considered complete when the following
conditions hold:

```text
- Human `by` proof blocks can be parsed.
- Human tactic scripts can be executed sequentially.
- intro / exact / apply / rw / simp-lite / induction work within the MVP scope.
- The kernel can check proof terms after tactics.
- Certificate generation can be rejected when unresolved goals remain.
- The Human tactic parser / bridge is not in the hot path of the AI-facing Machine tactic API.
- Phase 7 / Phase 9 continue to use the Machine API without going through Human source.
- Machine tactic canonical hash / cache key / budget do not change when Human syntax is added.
```

Recommended final checks:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p npa-frontend
cargo test -p npa-tactic
cargo test -p npa-api
./scripts/phase9-regression.sh
cargo test --workspace
```

In environments where `./scripts/phase9-regression.sh` does not exist, always
run targeted Phase 7 / Phase 9 regression instead.
