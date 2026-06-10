# Phase 5 Human Task Breakdown

This task breakdown treats `develop/phase5-human.md` as authoritative and splits
the gap from the current `crates/npa-frontend` / `crates/npa-tactic` /
`crates/npa-api` implementation into implementation milestones.

Phase 5 Human is an untrusted layer for human-facing IDE / Web UI / CLI / Human
API clients to handle proof state, tactic execution, theorem search, and goal
display. The deterministic Machine API contract for AI proof searchers belongs
to `develop/phase5-ai.md`; Human text tactics, pretty display, LSP concerns,
and document caches must not be mixed into the Machine hot path.

Important constraints:

```text
- Do not treat a Human API `success` response as proved.
- The basis for proof acceptance is only the canonical certificate and kernel / verifier / independent checker results.
- Human `/tactic/run` may accept text tactics, but it is not used for the high-volume candidate path of AI searchers.
- Do not change the request grammar, fingerprints, cache keys, or diagnostic hashes of `/machine/tactics/run`, `/machine/tactics/batch`, `/machine/replay`, or `/machine/verify`.
- Do not put Human goal display, source spans, LSP diagnostics, or assistant payloads into certificate payloads.
- Do not put an HTTP server, network, I/O, plugin loading, or AI calls into the kernel crate.
```

---

## 0. Current Implementation Boundary

### 0.1 Things Treated As Implemented

The current `crates/npa-frontend` has the prerequisites for the Phase 3 Human
Surface and Phase 4 Human tactic bridge.

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanDeclValue
- HumanProofBlock / HumanTacticScript / HumanTacticSyntax
- HumanSourceInterface / HumanImportedSourceInterface
- HumanDiagnostic / HumanDiagnosticPayload / HumanHoleGoal

crates/npa-frontend/src/human_parser.rs
- parse_human_module / parse_human_term
- Human tactic parser for by proof blocks and intro / exact / apply / rw / simp-lite / induction

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core / compile_human_source_to_certificate
- Human tactic term elaboration context
- collect_human_by_proof_targets_with_source_interfaces
- prepare_human_proof_start_core_with_source_interfaces
```

The current `crates/npa-tactic` has Machine proof state primitives and the tactic
execution core.

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- start_machine_proof
- run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
- assign_goal / extract_closed_machine_proof / extract_closed_machine_theorem_decl
- intro / exact / apply / rw / simp-lite / induction-nat
- deterministic budget hash / tactic cache key / proof delta
```

The current `crates/npa-api` has Human by-proof execution and Machine API
substrate.

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

### 0.2 Unimplemented Phase 5 Human Scope

The following areas required by `develop/phase5-human.md` are not yet separated
as a Human IDE/API profile.

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

### 0.3 Things That Must Not Enter The Machine Fast Path

The following may be handled in Phase 5 Human, but must not enter the
high-frequency candidate checking path of the Machine API.

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

## 1. Design Rules For Protecting The AI Fast Path

Each Phase 5 Human milestone treats the following as acceptance criteria.

```text
- Do not change request / response schemas for `/machine/*` endpoints.
- Do not add Human text tactics, pretty display, source spans, or assistant metadata to `MachineTacticCandidate`.
- Do not change Machine `state_fingerprint`, `snapshot_id`, `candidate_hash`, `proof_delta_hash`, or `deterministic_budget_hash`.
- The AI path for Machine snapshot retrieval can keep `include_pretty = false`.
- Human `/tactic/run` is not a replacement for `run_machine_tactic_candidates_batch`.
- Suggested tactic strings in Human search results are limited to Human UI use; AI searchers use the `develop/phase5-ai.md` search contract that returns `MachineTacticCandidate`.
- Do not confuse Human document cache keys with trusted certificate hashes.
- Human APIs do not add dependency direction into the kernel crate, and do not put I/O / network / server state into the kernel.
```

Recommended structure:

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

## 2. Implementation Order

Phase 5 Human first builds an IDE state model without breaking the existing
Human tactic bridge, then adds tactic / search / display / LSP layers thinly.

```text
1. Fix the Human / Machine API boundary and regression guard
2. Build the Human session / document store
3. Materialize the Human proof state store and StructuredGoal
4. Expose state retrieval APIs
5. Add the goal display renderer
6. Connect Human tactic run / check to session state
7. Add tactic suggestions
8. Add the Human theorem index and search APIs
9. Integrate session verify / certificate handoff
10. Add document update / incremental checking
11. Add the LSP payload adapter
12. Add the optional assistant payload
13. Fix integration regression and doc consistency
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-api human
cargo test -p npa-api phase7
cargo test -p npa-api phase9
cargo test -p npa-tactic
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib machine_surface
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. Task List

### P5H-00: Fix The Human / Machine IDE API Boundary

Implementation tasks:

- [x] Reflect the boundaries in `develop/phase5-human.md`, `develop/phase5-ai.md`, and `develop/phase7-ai.md` in implementation comments and test names.
- [x] Fix the module location for the Human IDE API. Candidates are `crates/npa-api/src/human_ide.rs` or a Human-only submodule inside `human.rs`.
- [x] State in public API comments that the Human IDE API does not implicitly create Machine sessions.
- [x] Add regressions that do not add Human-only fields to `/machine/*` request grammar, `MachineProofSnapshot`, or `MachineTacticCandidate`.

Dependencies:

```text
None
```

Deliverables:

```text
- Human IDE API boundary skeleton
- Machine fast-path regression guard
```

Affected files:

```text
crates/npa-api/src/lib.rs
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
develop/phase5-human.md
develop/phase5-ai.md
develop/phase7-ai.md
```

Acceptance criteria:

- [x] Human IDE API entry points are exported under names distinct from the Machine API.
- [x] `run_machine_tactic_request` / `run_machine_tactic_batch_request` do not call the Human parser.
- [x] Phase 7 / Phase 9 tests pass without importing the Human IDE module.

Verification:

```sh
rg -n "parse_human|Human" crates/npa-api/src/tactic.rs crates/npa-api/src/search.rs crates/npa-api/src/replay.rs crates/npa-api/src/verify.rs
cargo test -p npa-api phase7
cargo test -p npa-api phase9
```

AI speed guard:

- [x] Do not change Machine `state_fingerprint` / `candidate_hash` / `deterministic_budget_hash` fixtures.

---

### P5H-01: Build The Human Session / Document Store

Implementation tasks:

- [x] Add `HumanProofSession`, `HumanDocumentId`, `HumanDocumentVersion`, and `HumanSessionId`.
- [x] Store source text, module name, verified imports, Human imported source interfaces, and options in `HumanDocumentSnapshot`.
- [x] Add a library API corresponding to `POST /sessions`; parse / collect Human source and return initial messages.
- [x] Add a library API corresponding to `POST /documents/update`; increment document versions monotonically.
- [x] Implement the session store as an in-memory library data structure, not inside the kernel crate.

Dependencies:

```text
P5H-00
```

Deliverables:

```text
- HumanProofSession and HumanDocumentSnapshot data model
- session create / document update library API
```

Affected files:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
crates/npa-api/src/lib.rs
```

Acceptance criteria:

- [x] An open session with `session_id`, `document_id`, and `document_version = 1` can be created from Human source.
- [x] After an update, state requests specifying an old `document_version` can become structured stale-request errors.
- [x] Verified imports and imported Human source interfaces are explicit in the request; no filesystem / network lookup is performed.

Verification:

```sh
cargo test -p npa-api human_session
cargo test -p npa-api human
```

AI speed guard:

- [x] Do not integrate the Human session store into `MachineProofSession`.

---

### P5H-02: Build The Human Proof State Store And Stable State IDs

Implementation tasks:

- [x] Add `HumanProofStateStore`, `HumanStateId`, and `HumanGoalId` mappings.
- [x] Store the `MachineProofState` from `start_human_proof` as state inside the Human session.
- [x] After tactic execution, add the new state as an immutable entry and do not mutate the old state.
- [x] Link state entries to document version, source position, selected goal, and messages.

Dependencies:

```text
P5H-01
```

Deliverables:

```text
- HumanProofStateStore with stable Human state handles
- immutable state transition storage
```

Affected files:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
```

Acceptance criteria:

- [x] The original proof state can be retrieved from `state_id` inside the same session.
- [x] After tactic failure, the old state's open goals and proof skeleton do not change.
- [x] State IDs are Human API handles and are not used as replacements for Machine `state_fingerprint`.

Verification:

```sh
cargo test -p npa-api human_state_store
cargo test -p npa-tactic
```

AI speed guard:

- [x] Do not add an API that uses Human `state_id` as Phase 7 search node identity.

---

### P5H-03: Materialize StructuredProofState / StructuredGoal

Implementation tasks:

- [x] Add `StructuredProofState`, `StructuredGoal`, `StructuredHypothesis`, and `StructuredExpr`.
- [x] Build local IDs, names, types, optional values, implicit flags, and dependency lists from Machine goal contexts.
- [x] Compute `core_hash`, head symbol, constants, free locals, and size from target / hypothesis types.
- [x] Treat `pretty` as a display field, not as a basis for identity / cache / verification.

Dependencies:

```text
P5H-02
```

Deliverables:

```text
- StructuredProofState / StructuredGoal / StructuredExpr model
- deterministic core metadata materialization
```

Affected files:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
crates/npa-api/src/renderer.rs
```

Acceptance criteria:

- [x] The open goal for `theorem t (n : Nat) : n = n := by _` has context `n : Nat` and target `n = n`.
- [x] `core_hash` does not change when pretty text changes, and does change when the core expr changes.
- [x] Local dependency order is deterministic and does not depend on HashMap iteration order.

Verification:

```sh
cargo test -p npa-api human_structured_goal
cargo test -p npa-api renderer
```

AI speed guard:

- [x] Do not change `MachineGoalView` canonical bytes.

---

### P5H-04: Expose State Retrieval APIs

Implementation tasks:

- [x] Add a library API corresponding to `/state/by_id`.
- [x] Add a lightweight API corresponding to `/state/goals` that returns goal IDs and pretty display.
- [x] Connect the API corresponding to `/state/current` to session cursor state.
- [x] Map the API corresponding to `/state/at` to source position and proof block / hole position.
- [x] Structure error kinds for not found / stale document / no proof state.

Dependencies:

```text
P5H-03
```

Deliverables:

```text
- state lookup library APIs
- structured stale / not-found diagnostics
```

Affected files:

```text
crates/npa-api/src/types.rs
crates/npa-api/src/human.rs
```

Acceptance criteria:

- [x] Current proof state can be obtained from a source position.
- [x] The relevant goal is returned at the position of a `_` hole.
- [x] If a source position is outside a proof, empty goals or a structured not-found response is returned.

Verification:

```sh
cargo test -p npa-api human_state_api
```

AI speed guard:

- [x] Do not add dependency changes to the include-pretty-free path of `/machine/snapshots/get`.

---

### P5H-05: Add The Human Goal Display Renderer

Implementation tasks:

- [x] Add `pretty` / `explicit` / `core` / `json` modes to the API corresponding to `/display/goal`.
- [x] Add an API corresponding to `/display/expr` so display can operate per `StructuredExpr`.
- [x] Show goal replacement / closed / added goals before and after tactics through an API corresponding to `/display/diff`.
- [x] Perform context folding and relevant context ordering through an API corresponding to `/display/context`.
- [x] Fix display options as a Human display profile and do not put them into trusted payloads.

Dependencies:

```text
P5H-03
P5H-04
```

Deliverables:

```text
- Human display renderer
- goal diff and context display APIs
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/renderer.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Pretty mode can return human-facing display using notation / implicit hiding.
- [x] Explicit mode displays implicit arguments.
- [x] Core mode returns display close to the core expression seen by the kernel.
- [x] Even after folding, json / StructuredGoal retain the full context.

Verification:

```sh
cargo test -p npa-api human_display
```

AI speed guard:

- [x] Do not use the pretty renderer as input to Machine candidate validation / replay / verify.

---

### P5H-06: Connect Human `/tactic/run` To Session State

Implementation tasks:

- [x] Parse Human text tactic requests and dispatch them to existing `run_human_*_tactic`.
- [x] Add request validation for `state_id` / `goal_id` / `tactic` / `budget`.
- [x] Fix response shapes for success / closed / partial / error / timeout / unsafe.
- [x] On tactic success, add the new state to `HumanProofStateStore` and retain the old state.
- [x] On tactic failure, return the old state ID, structured error, expected / actual hash, span, and suggestions.

Dependencies:

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

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] `intro n` converts a Pi target into a subgoal and returns a new state ID.
- [x] `exact n` closes the goal using a matching local.
- [x] `apply Eq.trans` returns the expected subgoals.
- [x] `rw [Nat.add_zero]` and `simp-lite` use only proof-producing paths.
- [x] Sending `intro h` to an equality target yields a structured error corresponding to `expected_pi_type`.

Verification:

```sh
cargo test -p npa-api human_tactic_run
cargo test -p npa-api human
```

AI speed guard:

- [x] `/machine/tactics/batch` does not call Human `/tactic/run`.

---

### P5H-07: Add Human `/tactic/check` And `/tactic/suggest`

Implementation tasks:

- [x] Add an API corresponding to `/tactic/check` that returns parse / validation / expected effect but does not save state.
- [x] Add builtin suggestions corresponding to `/tactic/suggest`.
- [x] Implement `target is Pi -> intro`, `Eq t t -> exact Eq.refl t`, context exact, rw candidates, and Nat induction candidates.
- [x] Include source, confidence, reason, and suggested tactic text in suggestion responses.
- [x] Suggestions are untrusted and are resubmitted to `/tactic/run` before adoption.

Dependencies:

```text
P5H-06
```

Deliverables:

```text
- Human tactic check API
- builtin Human tactic suggestion API
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] An `intro` suggestion is returned for Pi targets.
- [x] An `exact Eq.refl ...` suggestion is returned for reflexive equality targets.
- [x] An `exact h` suggestion is returned when the context has a local with the same type as the target.
- [x] The proof state does not change when a suggestion fails.

Verification:

```sh
cargo test -p npa-api human_tactic_suggest
```

AI speed guard:

- [x] Human suggestion confidence / reason is not put into Machine cache keys, replay plans, or certificates.

---

### P5H-08: Add The Human Theorem Index

Implementation tasks:

- [x] Build the Human theorem index from verified imports and checked current declarations.
- [x] Give index entries name, module, statement core expr, statement pretty, head symbol, constants, attributes, kind, dependencies, and axiom deps.
- [x] Preserve import `export_hash` / high-trust `certificate_hash` / `decl_interface_hash`.
- [x] Put only the kernel-checked prefix of current declarations into the index.
- [x] Make axiom dependencies usable for ranking / filtering.

Dependencies:

```text
P5H-03
P5H-04
```

Deliverables:

```text
- Human theorem index data model
- verified import / current declaration index builder
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/search.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Theorems / defs / axioms / constructors / recursors from direct verified imports can be indexed.
- [x] Unchecked external theorem databases are not included in the index.
- [x] Theorems without `decl_interface_hash` are not returned as verified results.
- [x] Theorems with axiom dependencies can be identified.

Verification:

```sh
cargo test -p npa-api human_theorem_index
cargo test -p npa-api search
```

AI speed guard:

- [x] Do not change the theorem index fingerprint of Machine `/machine/search/for_goal`.

---

### P5H-09: Add The Human Theorem Search API

Implementation tasks:

- [x] Add an API corresponding to `/search/name`.
- [x] Add an API corresponding to `/search/by_type`.
- [x] Add an API corresponding to `/search/for_goal` with exact / apply / rw / simp modes.
- [x] Add an API corresponding to `/search/rewrite`.
- [x] Include suggested Human tactic string, match, why, score, and axiom info in results.
- [x] In high-trust mode, allow theorems using axioms to be penalized or filtered.

Dependencies:

```text
P5H-08
P5H-07
```

Deliverables:

```text
- Human theorem search APIs
- suggested Human tactic rendering for search results
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/search.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] `Nat.add_zero` can be found by name search.
- [x] Matching theorems can be returned for the type pattern `?x + 0 = ?x`.
- [x] Candidates `exact Nat.add_zero n` and `rw [Nat.add_zero]` can be returned for current goal `n + 0 = n`.
- [x] Suggested tactics can be resubmitted to `/tactic/run` for checking.

Verification:

```sh
cargo test -p npa-api human_search
```

AI speed guard:

- [x] Do not create an API that passes Human suggested tactic strings to Phase 7 instead of raw `MachineTacticCandidate`.

---

### P5H-10: Integrate Human Session Verify / Certificate Handoff

Implementation tasks:

- [x] Add an API corresponding to `/session/verify`.
- [x] Reject states with remaining open goals before verification.
- [x] Extract the root proof term from closed proof state and pass it to kernel check and certificate generation.
- [x] Return certificate hash, axioms used, and contains_sorry-equivalent data from certificate verifier output.
- [x] Do not ignore import `export_hash` / `certificate_hash` or axiom reports.

Dependencies:

```text
P5H-06
P5H-08
```

Deliverables:

```text
- Human session verify API
- certificate / axiom report response model
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/verify.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Do not mark as verified merely because goals are empty; return verified only after kernel check and certificate verifier success.
- [x] Reject certificate generation if even one unresolved goal remains.
- [x] Axiom reports match between the response and certificate verifier output.

Verification:

```sh
cargo test -p npa-api human_verify
cargo test -p npa-cert
```

AI speed guard:

- [x] Human verify does not change the response schema / replay contract of `/machine/verify`.

---

### P5H-11: Add Document Update / Incremental Checking

Implementation tasks:

- [x] Introduce `source_decl_hash`, `resolved_decl_hash`, and `core_decl_hash` as Human document cache keys.
- [x] Reuse parse / resolve / elaborate results for unchanged declarations.
- [x] Recompute proof states and diagnostics after the changed declaration.
- [x] Do not make cache hit / cache miss a basis for proof acceptance.
- [x] Tie cache invalidation to import interface hash and document version.

Dependencies:

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

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
crates/npa-frontend/src/human_elaborator.rs
```

Acceptance criteria:

- [x] After a source edit, document version increases and old state requests can be rejected as stale.
- [x] Declarations in the unchanged prefix are reused.
- [x] Even with reused results, final verify goes through the kernel / certificate verifier.

Verification:

```sh
cargo test -p npa-api human_incremental
cargo test -p npa-frontend --lib human
```

AI speed guard:

- [x] Do not reuse Human incremental cache keys as Machine `state_fingerprint`.

---

### P5H-12: Add The LSP-facing Payload Adapter

Implementation tasks:

- [x] Add an adapter that converts diagnostics payloads to LSP diagnostic shape.
- [x] Include theorem statements, attributes, and axioms in hover payloads.
- [x] Return tactic suggestions and search commands for completion / code actions.
- [x] Add minimal payloads for semantic tokens / document symbols / inlay hints.
- [x] Use `/state/goals` and `/display/goal` for custom goal view.
- [x] Treat the transport server as an optional layer and do not put it in the kernel crate.

Dependencies:

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

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
```

Acceptance criteria:

- [x] Human diagnostic spans are converted to LSP ranges.
- [x] Hover can return the statement and axiom info for `Nat.add_zero`.
- [x] Code actions can return `exact Eq.refl n` / `simp-lite` / search commands.

Verification:

```sh
cargo test -p npa-api human_lsp
```

AI speed guard:

- [x] Do not mix LSP payload types into Machine API response envelopes.

---

### P5H-13: Add The Optional Assistant Payload

Implementation tasks:

- [x] Add a payload for Human UI that bundles structured goals, nearby theorems, failed tactics, and available tactics.
- [x] Limit assistant output to suggested Human tactic strings and confidence / reason.
- [x] Always check assistant output through `/tactic/run` before adopting a candidate.
- [x] State in API docs that deterministic payloads for AI proof searchers use `/machine/prompt_payload` from `develop/phase5-ai.md`.

Dependencies:

```text
P5H-07
P5H-09
```

Deliverables:

```text
- optional Human assistant payload API
- assistant candidate validation rule
```

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
develop/phase5-human.md
```

Acceptance criteria:

- [x] Assistant payload includes state ID, goal summary, available tactics, nearby theorems, and failed tactic diagnostics.
- [x] Confidence / reason do not enter certificates, replay plans, or Machine cache keys.
- [x] The schema and fingerprint of Machine `/machine/prompt_payload` do not change.

Verification:

```sh
cargo test -p npa-api human_assistant_payload
cargo test -p npa-api prompt
```

AI speed guard:

- [x] Do not make assistant payload part of the required path for Phase 7 MVP.

---

### P5H-14: Fix Integration Regression And Documentation

Implementation tasks:

- [x] Add the Phase 5 Human end-to-end fixture.
- [x] The fixture runs session create, state lookup, tactic run, search, display, and verify.
- [x] Add separation regression for the Human path and Machine path.
- [x] Update README implementation status.
- [x] Synchronize completion criteria between `develop/phase5-human.md` and this task breakdown.

Dependencies:

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

Affected files:

```text
crates/npa-api/src/human.rs
crates/npa-api/src/types.rs
README.md
develop/phase5-human.md
develop/phase5-human-todo.md
```

Acceptance criteria:

- [x] `theorem t (n : Nat) : n = n := by exact Eq.refl n` can be verified from a Human session.
- [x] The type mismatch diagnostic for `theorem id (A : Type) (x : A) : A := by exact x` is returned structurally.
- [x] Machine API Phase 7 fixtures preserve the same candidate / state identity before and after Human API additions.
- [x] Results from `cargo fmt --all`, targeted tests, and workspace tests can be reported in the PR / commit message.

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

AI speed guard:

- [x] Even after Human IDE integration, Phase 7 MVP continues to use `MachineApiClient`, `MachineProofSnapshot`,
  raw `MachineTacticCandidate`, `/machine/tactics/batch`, `/machine/replay`, and `/machine/verify`.

---

## 4. Review ledger

Final review result:

```text
No confirmed findings remain.
```

Checked viewpoints:

```text
- Assigned the four core features from `develop/phase5-human.md` to P5H-03 through P5H-10.
- Separated document / incremental checking, LSP, assistant payload, and non-goals into P5H-11 through P5H-13.
- Made verify / certificate handoff and the trusted boundary explicit in P5H-10.
- Added acceptance criteria protecting the AI-oriented Machine fast path to every milestone.
- Treated the currently implemented Human tactic bridge as a prerequisite for the Phase 5 Human session/API layer, not as a reimplementation task.
```
