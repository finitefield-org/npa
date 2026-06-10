# Phase 9 AI Profile: Advanced Automation

This document specifies **Phase 9 for AI** in NPA.

The Phase 9 Human Profile organizes advanced inductives, stronger universe
polymorphism, typeclasses, quotients, SMT certificates, theorem graphs, and
natural-language formalization as high-level features for humans.

The Phase 9 AI Profile is the **untrusted Machine Profile** for using those
features from AI searchers, formalizers, and repair systems. AI emits candidates
and auxiliary data, but it is never correctness evidence. The only accepted
result is a canonical certificate checked by the kernel and independent checker.

```text
Not trusted:
  AI model
  prompt / completion
  theorem graph score
  typeclass search heuristic
  SMT solver process
  natural language formalizer
  repair suggestion

Trusted:
  small Rust kernel
  canonical core AST
  canonical certificate
  independent checker
```

Goals:

```text
- let AI emit advanced-feature candidates in structured form
- pass every candidate through deterministic validation / replay
- keep AI trace and score out of certificate hashes
- keep theorem graph and natural-language data as search sidecars only
- fix checker boundaries for SMT and quotient features that can expand the trusted base
```

Implementation / boundary notes (2026-05-25):

```text
- Phase 9 AI MVP M1-M9 are implemented as advanced automation endpoint substrate
  and deterministic fixture matrix in crates/npa-api.
- The Phase 9 Human target scope P9H-00 through P9H-15 is also implemented,
  but this AI MVP is deterministic Machine Profile validation / replay substrate,
  not production AI / solver / graph service operation.
- Production AI orchestrator, LLM / RAG, external SMT solver processes, and graph store operation
  remain caller / Post-MVP integration outside the trusted path.
- crates/npa-api Phase 9 endpoint substrate is untrusted candidate validation / replay automation.
  It does not use AI candidates, scores, prompts, or sidecars as certificate hash or checker pass/fail evidence.
- This remains fixed by:
  p9h00_advanced_ai_sidecars_scores_and_smt_outputs_stay_untrusted
  p9h00_ai_fast_path_request_shapes_exclude_phase9_human_heavy_checks
- The fixed post-Phase-9 regression gate is ./scripts/phase9-regression.sh.
  GitHub Actions workflows have been removed from the current repository, so this gate is run locally as needed.
```

---

# 1. Overall Architecture

Phase 9 AI is a higher-level profile over Phase 3 AI, Phase 4 AI, Phase 5 AI,
Phase 7, and Phase 8 AI.

```text
AI Orchestrator
  ↓ untrusted proposals
Phase 9 AI Machine Profile
  ↓ validation / replay
Phase 3 AI Machine Surface
Phase 4 AI Machine Tactics
Phase 5 AI Machine API
Phase 7 Search
  ↓ checked proof term / declaration / certificate
Rust kernel
  ↓ canonical certificate
Phase 8 independent checker
```

Phase 9 AI does not add AI calls to the kernel. AI calls, RAG, embeddings, graph
ranking, and SMT solver execution stay outside the trusted base.

---

# 2. Common Candidate Envelope

Every Phase 9 AI feature uses a structured candidate envelope, not free text.

```rust
enum Phase9AiProfileVersion {
    MvpV1,
}

struct Phase9AiCandidateEnvelope<T> {
    profile_version: Phase9AiProfileVersion,
    task_kind: Phase9AiTaskKind,
    target: Phase9AiTarget,
    imports: Vec<VerifiedImportRef>,
    options: Phase9AiOptionsRef,
    payload: T,
}

enum Phase9AiTaskKind {
    AdvancedInductive,
    UniverseRepair,
    TypeclassResolution,
    QuotientConstruction,
    SmtCertificate,
    TheoremGraphQuery,
    NaturalLanguageFormalization,
}

struct Phase9AiTarget {
    env_fingerprint: Hash256,
    target_decl_hash: Option<Hash256>,
    goal_fingerprint: Option<Hash256>,
}
```

Common machine shapes:

```rust
struct Phase9AiGoal {
    universe_params: Vec<UniverseParam>,
    local_context: Vec<MachineLocalDecl>,
    target: CoreExpr,
}

type Telescope = Vec<MachineTelescopeBinder>;

struct MachineTelescopeBinder {
    ty: CoreExpr,
}

struct Phase9AiGlobalRef {
    module: ModuleName,
    export_hash: Hash256,
    certificate_hash: Hash256,
    name: GlobalName,
    decl_interface_hash: Hash256,
}

struct MachineSurfaceTerm {
    universe_params: Vec<UniverseParam>,
    term_canonical_bytes: Vec<u8>,
}
```

Options may be inline canonical bytes or referenced artifacts.

```rust
enum Phase9AiOptionsRef {
    Inline {
        options_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        options_hash: Hash256,
        size_bytes: u64,
    },
}
```

Options include independent-checker profile, approved nested type constructors,
typeclass declarations, quotient references, SMT references, and formalization
tactic settings.

Every envelope has a canonical hash. AI score, prompt text, and explanation are
sidecars outside the envelope hash unless explicitly included as untrusted
metadata fields.

---

# 3. Advanced Inductive AI

Purpose:

```text
validate AI-proposed advanced inductive declarations without trusting AI-generated recursors
```

Scope:

```text
- indexed inductive payload validation
- approved nested constructor validation
- positivity precheck diagnostics
- generated recursor hash comparison
- Phase 1/Phase 9 Human kernel handoff
```

Rules:

```text
- AI cannot provide trusted recursors
- constructor result types are checked by kernel rules
- positivity is checked by the kernel/checker
- unsupported mutual/nested/large-elimination profiles fail closed
- validation result hashes are deterministic
```

Completion:

```text
- valid Vec-like indexed examples pass
- invalid constructor result examples fail
- AI-generated recursor mismatch is rejected
- no sidecar score affects acceptance
```

---

# 4. Universe Polymorphism AI

Purpose:

```text
allow AI to propose universe repairs and explicit universe arguments while accepting only deterministic solver/checker results
```

Scope:

```text
- universe metavariable repair
- explicit universe argument suggestions
- constraint normalization
- deterministic universe solver output
- replay through Machine Surface / kernel
```

Rules:

```text
- AI suggestions are candidates only
- final universe assignments come from deterministic solving/checking
- unresolved universe metas fail
- automatic generalization is profile-controlled
- universe repair hashes exclude AI prompt/score
```

Completion:

```text
- deterministic solver repairs simple universe metas
- invalid repairs are rejected
- replay produces the same validation result hash
```

---

# 5. Typeclass AI

Purpose:

```text
expose bounded typeclass resolution to AI without making heuristics trusted
```

Scope:

```text
- class/instance candidate validation
- bounded instance search
- dictionary term construction
- ambiguity diagnostics
- explicit core lowering
```

Rules:

```text
- typeclass search is not in the kernel
- selected instances become explicit dictionary terms
- ambiguity is not resolved by score alone
- search is bounded and deterministic
- memoization is performance-only
```

Completion:

```text
- basic algebraic instances resolve deterministically
- ambiguous instances produce structured errors
- selected dictionary terms kernel-check
```

---

# 6. Quotient AI

Purpose:

```text
validate quotient construction candidates while routing all trusted behavior through quotient-capable kernel/checker profiles
```

Scope:

```text
- Setoid reference validation
- quotient constructor/lift/sound references
- equivalence proof checking
- compatibility proof checking
- opt-in independent checker profile support
```

Rules:

```text
- quotient support is explicit profile behavior
- Phase8MvpReference rejects unsupported quotient certificates
- QuotientV1Reference supports the implemented unary quotient_v1 surface
- compatibility proofs are checked by the kernel
- AI cannot assert quotient soundness
```

Completion:

```text
- valid quotient construction passes under quotient-capable profile
- same certificate fails closed under unsupported profile
- missing equivalence / compatibility proof is rejected
```

---

# 7. SMT Certificates AI

Purpose:

```text
allow AI to propose SMT-backed automation without trusting solver output
```

Scope:

```text
- SMT task envelope
- encoded formula / symbol map validation
- deterministic rejection fixtures
- proof payload validation hooks
- future solver-native success profile
```

Rules:

```text
- solver process is untrusted
- solver result alone is not success
- success requires reconstructed proof term or checked SMT certificate
- current MVP is deterministic rejection / validation substrate
- nonempty solver-native success is Post-MVP
```

Completion:

```text
- malformed SMT payloads are rejected deterministically
- unsupported theory/profile fails closed
- no SMT output is treated as trusted proof without reconstruction
```

---

# 8. Theorem Graph AI

Purpose:

```text
use certificate-bound theorem graph snapshots for retrieval and ranking sidecars
```

Scope:

```text
- graph snapshot reference validation
- node/edge query validation
- certificate-bound node refs
- ranking sidecar isolation
- deterministic query result hashes
```

Rules:

```text
- graph nodes are bound to module/export/certificate/declaration hashes
- ranking scores are untrusted
- graph queries do not add imports automatically
- online graph store operation is Post-MVP integration
```

Completion:

```text
- graph queries return only snapshot-fixed certificate-bound node refs
- changing graph snapshot changes query identity
- ranking score does not affect certificate acceptance
```

---

# 9. Natural Language Formalization AI

Purpose:

```text
separate natural-language intent records from Machine Surface statement checking,
and connect to the Phase 4 proof bridge only when requested
```

Scope:

```text
- /machine/phase9/formalize/check
- source_document / rejection_reason artifact validation
- claim_span byte range / UTF-8 boundary validation
- ReviewerId regex validation
- FormalizationIntentRecord status validation
- MachineSurfaceTerm canonical decode
- candidate statement check through Phase 3 AI complete mode
- optional_proof_candidate bridge through Phase 4 single tactic
- CandidateStatementChecked / IntentRecordOnly / ProofBridgeChecked
```

Rules:

```text
- natural-language explanation is not theorem identity
- confidence score is not theorem identity
- reviewer state is separate from proof verification
- rejected intent cannot carry a proof candidate
- formal statement must pass Machine Surface checking
```

Completion:

```text
- rejected intent with proof candidate is RejectedIntentHasProofCandidate
- Unreviewed / Reviewed / Rejected intent fixtures exist
- statement elaboration failure differs from proof bridge failure
- reviewer-free formalization is not called verified mathematical intent
```

---

# 10. API Surface

MVP endpoint families:

```text
/machine/phase9/advanced-inductive/check
/machine/phase9/universe-repair/check
/machine/phase9/typeclass/resolve
/machine/phase9/quotient/check
/machine/phase9/smt/check
/machine/phase9/theorem-graph/query
/machine/phase9/formalize/check
```

Every endpoint:

```text
- accepts a Phase9AiCandidateEnvelope
- validates schema and profile version
- checks imports/options hashes
- produces deterministic validation_result_hash
- returns accepted / rejected / error with structured reason
- never writes trusted certificate state directly
```

Accepted candidate output is still only an intermediate value. It must flow back
to Phase 3/4/5/7/8 verification paths before becoming a proof.

---

# 11. Error Model

Common error families:

```text
InvalidProfileVersion
InvalidEnvelopeSchema
OptionsHashMismatch
ImportHashMismatch
UnsupportedProfile
UnsupportedFeature
InvalidCoreTerm
KernelRejected
IndependentCheckerRejected
AmbiguousTypeclassResolution
UniverseRepairFailed
QuotientProofMissing
SmtProofMissing
GraphSnapshotMismatch
FormalizationIntentInvalid
ProofBridgeFailed
```

Errors are structured, deterministic, and testable. Human-readable messages are
diagnostic only.

---

# 12. Security and Sandboxing

Security rules:

```text
- AI sidecars, scores, prompts, SMT outputs, and graph ranks do not enter certificate hashes
- external SMT solver execution is outside the trusted path
- graph/RAG/LLM services are callers, not trusted validators
- accepted candidates do not mutate trusted env until replay/check succeeds
- network and filesystem access are not implicit in endpoint validation
- all production proof acceptance returns to canonical certificate checking
```

---

# 13. MVP / Follow-up Milestones

## 13.1 M1 Common Validation Substrate

Implement candidate envelope parsing, options hashing, import validation,
validation result hashing, and closed-world schemas.

## 13.2 M2 Universe Repair MVP

Validate deterministic universe repair candidates and reject unsolved/invalid
repairs.

## 13.3 M3 Advanced Inductive MVP

Validate advanced inductive candidates and reject AI-generated recursor trust.

## 13.4 M4 Theorem Graph Query MVP

Validate graph snapshot queries and return certificate-bound node references.

## 13.5 M5 Typeclass Resolution MVP

Run bounded deterministic typeclass search and return explicit dictionary
candidates or ambiguity errors.

## 13.6 M6 Quotient Construction Surface / P9H-12 QuotientV1 Profile

Validate quotient construction candidates under explicit quotient-capable
profiles. `QuotientV1Reference` is opt-in and supports the implemented unary
`quotient_v1` surface.

## 13.7 M7 SMT Deterministic Rejection MVP

Validate SMT payload shape and fail closed for unsupported or proofless solver
outputs.

## 13.8 M8 Natural Language Formalization MVP

Validate intent records, Machine Surface statement candidates, and optional
Phase 4 proof bridge output.

## 13.9 M9 Phase 8 Audit Integration and Fixture Matrix

Continuously check that Phase 9 AI sidecars do not break certificate-first
boundaries.

Scope:

```text
- success / rejected / error fixture matrix for all endpoints
- validation_result_hash stability tests
- artifact replay tests
- independent checker support matrix tests
- regression that AI sidecars do not enter certificate hashes
- axiom report regression
- API / error enum / profile version compatibility tests
```

Completion:

```text
- deterministic fixtures pass without an AI model
- accepted certificate candidates can be judged by Phase 8 checker alone
- rejected candidates do not mutate trusted env
- validation results contain no time / random seed / network result
- docs and fixture names match milestone / endpoint / error enum
- after Phase 9 completion, ./scripts/phase9-regression.sh is the local fixed gate
```

## 13.10 Post-MVP Milestones

Post-MVP extensions require explicit schema / profile / checker support.

```text
P1  SMT success profile
    Add nonempty solver-native rule registry, rule descriptor fingerprint,
    PayloadNode proof reconstruction, and final_proof kernel / Phase 8 checker success.

P2  Quotient profile expansion
    P9H-12 QuotientV1Reference opt-in profile is implemented.
    `quotient_v2` adds Quotient.lift2 to kernel / certificate / reference checker.
    `quotient_v3` adds Quotient.indProp for Prop-valued theorems over arbitrary quotient elements.
    Phase 9 AI QuotientConstruction remains unary quotient_v1 until separate candidate profiles are added.

P3  Advanced inductive expansion
    Add mutual inductive, nested inductive through approved functors,
    large elimination, generic positivity traversal, and matching recursor hash rules.

P4  Theorem graph retrieval expansion
    Add embedding / graph ranking / premise recommendation sidecars.
    Ranking score remains outside certificate hash and validation pass/fail.

P5  Formalization orchestrator integration
    Human `/formalize` wrapper and intent certificate boundary are implemented in P9H-15.
    Remaining work is production LLM / RAG / theorem graph candidate generation and UI workflow outside the trusted path.

P6  Typeclass scalability
    Add memoization, priority profiles, and larger algebraic hierarchy support.
    Do not add a profile that resolves ambiguity by score alone.
```

The MVP must be fully testable without an AI model. LLMs and embedding
retrievers are later replaceable callers.

---

# 14. Completion Criteria

Phase 9 AI is complete when:

```text
- every Phase 9 AI candidate has a canonical hash
- AI trace / score / prompt never enters certificate hash
- advanced inductive does not accept AI-generated recursors
- universe repair accepts only deterministic solver results
- typeclass resolution does not resolve ambiguity by score
- quotient construction kernel-checks equivalence / compatibility proofs
- SMT results are not success without reconstructed proof terms or checked certificates
- theorem graph results return only snapshot-fixed certificate-bound node refs
- natural language formalization separates Machine Surface statements from intent review state
- Phase 8 independent checker can decide pass/fail without AI sidecars
```

Phase 9 AI is the Machine Profile that exposes advanced automation, search, and
formalization to AI while keeping AI outside the trusted base and routing every
accepted result back to canonical certificate checking.
