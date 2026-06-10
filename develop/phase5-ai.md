# Phase 5 AI Profile: Machine IDE/API

This document is the design for **Phase 5 for AI** in NPA.

`develop/phase5-human.md` defines APIs for humans using an IDE / Web UI / CLI to
read goals, try tactics, and search theorems. AI proof search instead generates
many candidates, expects failures, and needs reproducible API calls. Therefore
Phase 5 AI exposes a structured **Machine Proof Session API**, not a
pretty-text-centered interface.

---

# 1. Purpose

Phase 5 AI connects Phase 3 AI, Phase 4 AI, and Phase 7 AI search with a
deterministic replayable proof-server API.

```text
Machine Surface request
  ↓ Phase 3 AI
fully explicit term / structured diagnostic
  ↓
Machine Tactic candidate
  ↓ Phase 4 AI
transactional proof state transition
  ↓
Machine IDE/API
batch execution / theorem retrieval / replay / verify handoff
  ↓
kernel check + certificate generation
```

Priorities:

```text
- pretty strings are not trusted payload
- tactic text is not primary input
- AI prompt / completion / score / trace is not in certificates
- state_fingerprint is derived deterministically from canonical state bytes
- snapshot_id is derived deterministically from the full state_fingerprint
- same state + same deterministic request + same deterministic budget returns the same result / error, excluding scheduler_limits
- scheduler_limits timeout / memory stops are retryable artifacts, not deterministic semantic results
- batch execution treats every candidate transactionally
- search results are tied to verified imports / decl_interface_hash
- nothing is called proved until verify succeeds
```

---

# 2. Trust Boundary

AI-facing APIs are not trusted.

```text
Not trusted:
  AI output
  API client
  request ordering
  tactic ranking
  theorem search ranking
  proof search trace
  repair suggestion
  pretty printer
  cache hit / cache miss

Trusted:
  deterministic result of the Phase 1 Rust kernel checking fully explicit core / proof terms
  canonical certificate bytes / hash and Phase 2 certificate verifier result
  Phase 8 independent checker result
```

`success` from the Machine IDE/API means only that an untrusted state transition
succeeded. A proof is accepted only after the closed proof term passes kernel
check, canonical certificate verification, and required independent checker
profiles. Candidate terms are Phase 3 AI Machine Surface terms; pretty text and
Human notation are never evidence for identity, ranking, replay, or verify.

---

# 3. Difference from Human API

Human Phase 5 favors readability.

```json
{
  "state_id": "st_100",
  "goal_id": "g1",
  "tactic": "exact Eq.refl n"
}
```

AI Phase 5 favors structured deterministic input.

```json
{
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "candidate": {
    "kind": "exact",
    "term": {
      "source": "@Eq.refl.{1} Nat n"
    }
  },
  "deterministic_budget": {
    "max_tactic_steps": 64,
    "max_whnf_steps": 10000,
    "max_conversion_steps": 10000,
    "max_rewrite_steps": 100,
    "max_meta_allocations": 16,
    "max_expr_nodes": 20000
  },
  "scheduler_limits": {
    "timeout_ms": 100
  }
}
```

The AI Profile MVP does not center:

```text
- arbitrary tactic script text
- notation-dependent tactic payloads
- state selection by source cursor position
- short names depending on open / namespace state
- LSP display options
- natural language instructions
- server-side acceptance based on model score
```

It centers:

```text
- MachineProofSession
- MachineProofSnapshot
- MachineGoalView
- MachineTacticCandidate
- MachineTheoremQuery
- MachineTacticBatch
- MachineReplayPlan
- verified import / export_hash / decl_interface_hash
```

---

# 4. Prerequisites

Phase 5 AI depends on:

```text
Phase 2:
  verified module import
  export_hash
  certificate_hash
  decl_interface_hash
  axiom report

Phase 3 AI:
  Machine Surface parser / resolver / elaborator
  elaborate_machine_term_check
  canonicalize_machine_term_source
  structured MachineDiagnostic

Phase 4 AI:
  MachineProofState
  MachineTactic
  run_machine_tactic
  MachineProofDelta
  extract_closed_machine_theorem_decl

Phase 6:
  verified standard-library certificates
  optional theorem-search metadata for non-MVP extensions
```

Phase 5 AI does not implement the search strategy. Best-first search, advanced
premise retrieval, and proof minimization belong to Phase 7. Phase 5 gives
Phase 7 a safe API contract.

---

# 5. Machine Proof Session

## 5.1 Role

`MachineProofSession` is an untrusted session for AI search to submit many
candidates. The session does not resolve imports from filesystem or network; the
caller provides the verified import set.

```rust
struct MachineProofSession {
    session_id: SessionId,
    protocol_version: MachineApiVersion,
    root: CheckedMachineProofRoot,
    imports: Vec<VerifiedImportRef>,
    import_certificate_context: MachineImportCertificateContext,
    machine_surface_callable_interface_table: MachineSurfaceCallableInterfaceTable,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    options: MachineApiOptions,
    initial_snapshot: MachineProofSnapshot,
    snapshots: MachineSnapshotStore,
}
```

Requests must not ask the server to complete imports by module/hash alone. The
MVP session create request separates:

```text
import_closure:
  all certificate payloads for direct imports and transitive dependencies

imports:
  root key list for direct imports visible to the current proof state
```

Certificates in `import_closure` are verified by the Phase 2 verifier. Modules
in the closure but not direct `imports` are used only for transitive dependency
reconstruction, not for direct Machine Surface global scope, theorem search
premises, or tactic heads.

Duplicate closure keys `(module, expected_export_hash, expected_certificate_hash)`
are invalid. Same module with different hashes in direct imports is invalid.

## 5.2 session create

Session create validates:

```text
- protocol version
- deterministic options
- import_closure completeness
- direct imports are present in the closure
- Phase 2 verification of the closure
- root theorem statement through Phase 3 AI Machine Surface
- checked_current_decls form a valid checked prefix
- all derived fingerprints match canonical bytes
```

Output includes:

```text
- session_id
- session_root_hash
- initial snapshot_id
- initial state_fingerprint
- direct import summary
- root theorem statement hash
```

`session_root_hash` is a deterministic hash over protocol version, root theorem,
verified direct imports, checked current declarations, tactic options, callable
interface table, and import closure identity. It excludes scheduler limits,
prompt text, model metadata, cache state, and wall-clock data.

## 5.3 session_root_hash

The `session_root_hash` is the stable identity for replay. It changes when:

```text
- protocol version changes
- root theorem changes
- direct imports or their export/certificate hashes change
- checked current declaration prefix changes
- Machine API options change
- callable interface table changes
- import closure identity changes
```

It does not change when:

```text
- snapshot store storage path changes
- pretty rendering options change
- cache hit/miss changes
- prompt/model metadata changes
```

## 5.4 session delete

Session delete releases untrusted server memory for snapshots, cached rendered
views, and candidate results. It does not delete verified certificates or
content-addressed artifacts owned by the caller.

---

# 6. Machine Proof Snapshot

## 6.0 Hash Wire Format

All hashes on the wire are lowercase `sha256:` hex strings. Canonical payloads
use raw 32-byte digests. Hash fields are never display-only when they are part
of API identity.

## 6.1 Snapshot Structure

```rust
struct MachineProofSnapshot {
    snapshot_id: SnapshotId,
    session_root_hash: Hash,
    state_fingerprint: Hash,
    open_goals: Vec<MachineGoalView>,
    proof_state_core_hash: Hash,
    tactic_options_fingerprint: Hash,
    imports_fingerprint: Hash,
}
```

`snapshot_id` is derived from the full `state_fingerprint`, not from allocation
order. Snapshot canonical bytes include proof root, open goals, metavariable
store, deterministic options, checked environment fingerprint, and reserved
local names.

Snapshot bytes exclude:

```text
- pretty strings
- prompt payloads
- search scores
- failed candidate history unless explicitly included in the request
- cache state
- wall-clock data
```

## 6.2 MachineGoalView

Goal views separate machine identity from display strings.

```rust
struct MachineGoalView {
    goal_id: GoalId,
    meta_id: MetaVarId,
    context: Vec<MachineLocalView>,
    context_hash: Hash,
    target_core_hash: Hash,
    target_machine_term: MachineTermPayload,
    pretty: Option<PrettyGoalView>,
}
```

Machine payloads contain explicit global refs, hashes, and universe args as
required by Phase 3 AI. `pretty` is optional and is not part of identity.

## 6.3 Snapshot Fetch

Snapshot fetch checks:

```text
- session exists
- requested snapshot_id belongs to session_root_hash
- stored canonical bytes hash to state_fingerprint
- rendered goal view matches canonical state
```

The response may include pretty text for humans or prompts, but pretty text must
not affect `state_fingerprint`.

## 6.4 Snapshot Store Lifetime / Quota

Snapshot stores are untrusted caches. Evicting a snapshot may make a session
need replay, but must not affect certificate identity. Quota errors are
structured API errors, not proof failures.

---

# 7. Machine Tactic Execution API

## 7.0 MachineTacticCandidate Wire Schema

A candidate contains:

```text
- kind
- goal_id
- raw tactic payload
- deterministic_budget
- optional caller candidate_hash
```

The server recomputes `candidate_hash` from canonical candidate bytes. Unknown
fields in strict mode are invalid. `scheduler_limits` are outside candidate
hashes because they control process scheduling, not semantic execution.

## 7.1 Single Candidate Execution

Request:

```json
{
  "session_id": "sess_1",
  "snapshot_id": "snap_1",
  "state_fingerprint": "sha256:...",
  "candidate": {
    "kind": "exact",
    "goal_id": "g0",
    "term": {
      "format": "machine-term-v1",
      "payload": "..."
    }
  },
  "deterministic_budget": {
    "max_tactic_steps": 64
  }
}
```

Execution:

```text
1. validate session_root_hash / snapshot_id / state_fingerprint
2. recompute candidate_hash and budget hash
3. run Phase 4 Machine Tactic transactionally
4. store next snapshot only on success
5. return proof_delta_hash and next state identity
```

Failure rolls back all state. Errors are structured.

## 7.2 Batch Execution

Batch execution runs candidates independently from the same input snapshot.

Rules:

```text
- candidate results are returned in input order
- every candidate has its own transaction
- candidate failure does not affect other candidates
- cache state must not affect result ordering or semantics
- deterministic budget is per candidate unless explicitly provided as a shared cap
```

Batch result entries include candidate hash, budget hash, status, error or proof
delta hash, and next snapshot identity on success.

## 7.3 Text Compatibility

The MVP may offer compatibility endpoints for human-like text, but these are not
Machine API identity. Text is parsed into a structured `MachineTacticCandidate`
first, and only the structured payload is replayable.

---

# 8. Deterministic Budget

Deterministic budget controls semantic execution.

```rust
struct DeterministicBudget {
    max_tactic_steps: u64,
    max_whnf_steps: u64,
    max_conversion_steps: u64,
    max_rewrite_steps: u64,
    max_meta_allocations: u64,
    max_expr_nodes: u64,
    max_kernel_checks: u64,
}
```

Budget bytes are part of candidate execution identity. Exceeding deterministic
budget returns a deterministic structured error.

`scheduler_limits` such as wall-clock timeout and memory cap are operational.
They may produce retryable artifacts and are not proof-search semantic results.

---

# 9. Theorem Retrieval API

## 9.1 Purpose

Theorem retrieval returns premise candidates fixed to verified metadata, not
name-only suggestions.

## 9.2 theorem index fingerprint

The theorem index fingerprint is computed from canonical theorem index bytes
derived from verified imports. It includes:

```text
- module name
- export_hash
- declaration name
- decl_interface_hash
- statement core hash
- axiom dependency summary
- attributes used by deterministic retrieval
```

It excludes embedding vectors, LLM output, cache status, and display-only text.

## 9.3 Goal Search

Search request includes session root, snapshot, goal id, theorem index
fingerprint, filters, modes, and limit. The response includes:

```text
- query_fingerprint
- theorem_index_fingerprint
- ordered results
- truncation flag
- suggested_candidates as a required JSON array field
```

Each result includes verified metadata:

```text
- global_ref
- module
- export_hash
- decl_interface_hash
- statement_core_hash
- axiom dependencies
- score
```

Scores are computed from a `search_score_key_fingerprint` that excludes `limit`.
Changing `modes`, `filters`, rendered query inputs, or theorem index changes the
`query_fingerprint`.

`suggested_candidates` contains only candidates that pass MVP suggested
candidate validation. Validation does not run Machine Surface Complete mode,
tactic execution, kernel check, or deterministic budget. MVP deterministic
templates do not generate candidates containing Machine Surface term source.

If future suggested candidates include term source, the term-source
fully-explicitness rule must be included in `suggestion_profile_version` and
`query_fingerprint`.

## 9.4 query fingerprint

`query_fingerprint` is a canonical hash of query structure:

```text
session_root_hash
state_fingerprint
goal_id
theorem_index_fingerprint
modes
filters
limit
suggestion_profile_version
```

It excludes wall-clock time, cache hit/miss, and model metadata.

---

# 10. Prompt Payload API

Prompt payloads are untrusted sidecars for AI. They are deterministic views over
machine state and verified retrieval results.

Prompt payload identity includes:

```text
- session_root_hash
- state_fingerprint
- goal_id
- prompt options
- selected premise metadata
- theorem_index_fingerprint
- premise_query_fingerprint
- allowed tactic set
- explicitly provided failed_candidates
```

`payload_fingerprint` is not included inside its own hash input. The response
must return premise metadata with `global_ref`, `decl_interface_hash`, and
statement core hash. It must not return name-only premises.

Prompt response excludes search `score` and `suggested_candidates` unless they
are explicitly part of the rendered prompt options. Failed candidates come only
from request payload, not session history or cache.

---

# 11. Error Taxonomy

Machine API errors are structured.

```text
InvalidProtocolVersion
InvalidSessionCreateRequest
ImportClosureIncomplete
ImportVerificationFailed
DuplicateImport
SessionNotFound
SessionRootHashMismatch
SnapshotNotFound
StateFingerprintMismatch
InvalidSnapshot
InvalidCandidate
CandidateHashMismatch
UnsupportedMachineTactic
DeterministicBudgetExceeded
SchedulerLimitExceeded
MachineTacticFailed
BatchItemFailed
TheoremIndexMismatch
InvalidTheoremQuery
PromptPayloadMismatch
InvalidReplayPlan
ReplayHashMismatch
InvalidMachineProofState
VerifyOpenGoals
KernelRejected
CertificateVerificationFailed
StoreQuotaExceeded
```

Every error includes a stable kind, phase, optional goal/candidate identifiers,
and machine-readable detail. Human messages are diagnostic only.

---

# 12. Replay Contract

## 12.1 Replay Plan

A replay plan records enough information to rerun a successful tactic chain.

```rust
struct MachineReplayPlan {
    session_root_hash: Hash,
    initial_snapshot_id: SnapshotId,
    steps: Vec<MachineReplayStep>,
}

struct MachineReplayStep {
    input_state_fingerprint: Hash,
    candidate_hash: Hash,
    candidate: MachineTacticCandidate,
    deterministic_budget_hash: Hash,
    deterministic_budget: DeterministicBudget,
    expected_proof_delta_hash: Hash,
    expected_next_state_fingerprint: Hash,
}
```

Each step includes raw `candidate` and `deterministic_budget` payloads, not only
hashes. Replay re-executes the candidate from payload, compares hashes and proof
deltas, and advances only on match.

## 12.2 replay API

MVP replay starts from the current session's initial snapshot. If the plan
structure is invalid, return `InvalidReplayPlan` before checking root hash. If a
well-formed plan's `session_root_hash` differs from the current session, return
`SessionRootHashMismatch`. If re-execution differs from the plan, return
`ReplayHashMismatch`.

A Phase 5 RawMachineTerm prepass failure during replay is reported as
`ReplayHashMismatch`. If prepass succeeds but Phase 4 returns
`InvalidMachineTermSource`, that is an adapter invariant failure:
`InvalidMachineProofState` with phase `replay_execution`.

---

# 13. Verify Handoff

`/machine/verify` is the only endpoint that can return `status = verified`.

Verify requires:

```text
- snapshot belongs to session_root_hash
- open goals are empty
- proof state is internally valid
- closed proof term can be extracted
- Phase 1 kernel accepts proof : theorem_type
- Phase 2 certificate builder emits canonical certificate
- Phase 2 certificate verifier accepts it
```

Verify success response returns:

```text
- canonical certificate bytes
- dependency_import_closure
- import_payload usable by later sessions
- root_decl_interface_hash
- root_decl_certificate_hash
- root_axioms_used
- module_export_hash
- module_certificate_hash
- module_axioms_used
```

Root declaration hashes are distinct from module certificate hashes.
`import_payload.expected_export_hash` equals `module_export_hash`;
`import_payload.expected_certificate_hash` equals `module_certificate_hash`.

Generated certificates use canonical import order from the session direct
imports, required transitive imports for public interface reconstruction, and
transitive axiom-origin imports needed for Phase 2 axiom provenance. Current
module refs by source-index coordinates are rewritten to certificate-local
declaration indices.

---

# 14. Cache / Store

Caches and stores are performance aids, not evidence.

```text
- snapshot store
- rendered goal cache
- search result cache
- prompt payload cache
- replay result cache
```

Cache keys include the relevant fingerprints. Cache hit/miss is never part of
canonical state, replay, verify, or certificate identity.

---

# 15. Security

The Machine API must not:

```text
- call LLMs server-side in the MVP
- resolve imports from network or filesystem implicitly
- execute plugins or arbitrary user tactics
- trust theorem search results without verified hashes
- accept name-only global references
- treat closed proof state as verified before kernel/certificate checks
- include AI prompts, completions, scores, or traces in trusted payloads
```

All untrusted payloads are size-limited, schema-validated, and deterministic
budget checked.

---

# 16. Minimum API List

```text
POST /machine/session/create
DELETE /machine/session/{session_id}
GET /machine/snapshot
POST /machine/tactic/run
POST /machine/tactic/batch
POST /machine/theorem/search
POST /machine/prompt/render
POST /machine/replay
POST /machine/verify
```

All APIs accept and return structured hashes. Every endpoint that consumes a
snapshot also checks `session_root_hash` and `state_fingerprint`.

---

# 17. Implementation Order

Recommended order:

```text
1. session create with verified import closure
2. deterministic session_root_hash
3. MachineProofSnapshot and snapshot_id derivation
4. single tactic run endpoint
5. batch tactic execution
6. deterministic budget and structured errors
7. theorem index fingerprint and search API
8. prompt payload rendering
9. replay plan and replay endpoint
10. verify handoff to kernel and Phase 2 certificate verifier
11. cache/store quotas and eviction behavior
```

---

# 18. Test Examples

## 18.1 snapshot is deterministic

The same session create request yields the same `session_root_hash`,
`state_fingerprint`, and `snapshot_id`. Pretty rendering changes do not change
them.

## 18.2 tactic failure does not corrupt state

Run a failing candidate and then fetch the same snapshot. The fingerprint and
open goals remain unchanged.

## 18.3 batch ordering

Batch results have the same length and order as input candidates. Candidate
failures do not affect other entries.

## 18.4 search result is fixed to import hash

The same theorem index canonical bytes produce the same
`theorem_index_fingerprint`; the same canonical query produces the same
`query_fingerprint` and truncation. Results contain verified `global_ref` and
`decl_interface_hash`, not names alone. `suggested_candidates` is always a JSON
array, even when empty.

## 18.5 prompt payload fingerprint is fixed by input

The same `session_root_hash`, `state_fingerprint`, `goal_id`, theorem index, and
prompt options produce the same `payload_fingerprint`. Changing pretty options,
failed candidates, selected premises, allowed tactics, or rendered payload
fields changes the fingerprint.

## 18.6 replay step payload is self-contained

Every replay step includes raw `candidate` and `deterministic_budget` payloads.
Replay reruns from payload and checks `candidate_hash`, budget hash,
`proof_delta_hash`, and next state fingerprint.

## 18.7 not a proof until verify

Only `/machine/verify` on a snapshot with no open goals can return
`status = verified`, and only after kernel check plus certificate generation /
verification succeeds.

---

# 19. Excluded from Phase 5 AI

The MVP excludes:

```text
- server-side LLM calls
- embedding semantic search
- natural language formalization
- automatic theorem invention
- global best-first search scheduler
- proof minimization
- distributed worker orchestration
- plugin tactic execution
- arbitrary user-defined tactic
- unverified external theorem database
```

These belong to Phase 7 or to separate untrusted services.

---

# 20. Completion Criteria

Phase 5 AI is complete when:

```text
- MachineProofSession can be created from verified imports
- MachineProofSnapshot has deterministic state_fingerprint
- AI goal views separate pretty and machine representations
- one MachineTacticCandidate can be executed
- batch tactic execution is transactional and order-deterministic
- tactic errors return MachineApiErrorKind and hash-linked diagnostics
- theorem search returns verified metadata and suggested MachineTacticCandidate values
- replay plans can be re-executed and delta hashes checked
- closed snapshots can be handed to kernel check / certificate generation
- certificates / theorems are not treated as verified before verify succeeds
```

---

# 21. One-Sentence Summary

Phase 5 AI is the untrusted Machine API layer that lets AI proof search handle
proof states, tactic execution, theorem retrieval, replay, and verify handoff at
high volume with deterministic hashes and transactional behavior.
