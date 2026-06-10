# Phase 8 AI Profile: Checker Audit Automation

This document is the detailed design for **Phase 8 AI Profile: Checker Audit
Automation**. The Phase 8 Human Profile is defined in
`develop/phase8-human.md`; this document defines the AI / machine-client profile
for running checkers, normalizing results, organizing differences, and producing
audit sidecars.

The purpose of the Phase 8 AI Profile is not to let AI judge `.npcert`
correctness. AI may launch checkers, normalize results, triage failures, and
propose additional tests. Proof acceptance is always based only on deterministic
results emitted by independent checkers.

Scope:

```text
- machine check orchestration
- checker result normalization
- checker disagreement triage
- adversarial challenge generation
- CI / release audit summarization
- training and evaluation sidecars
```

Implementation notes (2026-05-21):

```text
- Phase 8 AI MVP M0-M13 are implemented as library substrate in crates/npa-api.
- `npa-checker-ref` is implemented as the standalone reference checker binary in crates/npa-checker-ref.
- The `npa-check ...` commands in this document are normative command contracts;
  in the current repository they are fixed by crates/npa-api library APIs and tests.
- The same validation / materialization / normalization / audit logic is fixed by library APIs and unit tests.
- crates/npa-api Phase 8 automation is an untrusted orchestration layer for checker requests,
  results, and audit artifacts. It cannot override checker verdicts or NormalizedCheckResult.comparison.
- External checker binary operation and full external-checker release audit CI remain later integration scope in the Human Profile.
- The current repository fixes the Phase 8 Release Audit fixture gate in scripts/phase8-release-audit.sh.
- GitHub Actions workflows have been removed from the current repository. External theorem-library CI is integration scope after package contracts are fixed.
```

Core rule:

```text
AI is not a checker.
AI does not produce verdicts.
AI only explains, classifies, and reruns checker results.
Accepted proofs are based only on canonical certificates and independent checker results.
```

---

# 1. Role of the Phase 8 AI Profile

The Phase 8 Human target flow is:

```text
certificate
  ↓
reference checker
  ↓
external checker
  ↓
CI / release audit
  ↓
verified_high_trust artifact
```

This is the target high-trust flow. The current repository's
`npa package high-trust` is the `verified_high_trust` artifact generator, but it
may generate the artifact only when required external and high-trust-reference
release audit evidence exists. It must not be generated from
reference-checker-only evidence. `npa-checker-ext` is release/high-trust
evidence only when a built executable is resolved through runner-owned registry
/ policy and passes binary identity and hash validation.

The AI Profile is a sidecar next to this flow.

```text
certificate manifest
  ↓
machine check runner
  ↓
normalized checker results
  ↓
AI triage / summary / challenge proposal
  ↓
human-readable audit report
```

Trusted path:

```text
.npcert
  → reference checker
  → external checker
  → deterministic check result
```

Untrusted sidecar:

```text
check result
  → AI summary
  → repair suggestion
  → challenge proposal
```

AI summaries never replace `checked` status.

---

# 2. Untrusted Boundary

Not trusted:

```text
- LLM output
- AI agent plan
- AI generated challenge
- AI generated certificate
- AI generated proof script
- AI explanation of checker errors
- AI selected imports
- AI selected checker binary
- AI modified checker config
- AI generated CI summary
- training labels inferred by AI
```

Trusted:

```text
- canonical .npcert bytes
- checker binary identity selected by policy, not AI
- checker result produced by that binary
- checker version / build hash recorded by CI
- export_hash / certificate_hash recomputed by checker
- axiom report recomputed by checker
```

AI may read but must not trust:

```text
- .npa source
- tactic script
- AI search trace
- pretty printed goal
- source map
- theorem search index
- previous failure summary
- human-written PR comment
```

These inputs can inform explanations and repair suggestions, but they are not
the authoritative checker request.

---

# 3. Inputs and Outputs

## 3.1 Input

Inputs are separated into checker inputs and AI sidecar inputs.

Checker dynamic inputs:

```text
- certificate bytes
- import certificates or trusted import store references
- explicit checker profile
- explicit trust mode
- deterministic budget
- axiom policy file
```

Ownership:

```text
certificate bytes:
  checker core input; target of proof validity and canonical certificate checks

import certificates / store refs:
  checker core input; target of import hash and referenced certificate checks

checker_profile:
  runner-owned selection deciding which checker binary / mode to run

trust_mode:
  runner-owned policy deciding normal / high-trust requirements

deterministic budget:
  runner/checker resource bound with deterministic semantics when specified

axiom policy:
  explicit policy input, not AI-inferred
```

AI sidecar inputs may include normalized results, source maps, logs, prompt
metadata, and human comments. They cannot affect checker execution or pass/fail.

## 3.2 Output

Outputs:

```text
- MachineCheckRequest
- MachineCheckResult
- NormalizedCheckResult
- AiAuditSidecar
- AuxiliaryResult
- ChallengeGenerationResult
- ChallengeCoverageSummary
- ReleaseAuditBundleManifest
- TrainingExample JSON Lines and manifest
```

Only checker results, normalized comparisons, deterministic auxiliary results,
and release bundle validation can drive CI or release status.

## 3.3 Canonical Serialization and Hashes

Every artifact with a hash uses canonical bytes, deterministic order, duplicate
key rejection, fixed schema fields, and closed-world validation. JSON may be a
wire form, but artifact hashes are computed from canonical bytes, not pretty JSON
text, map iteration order, or file paths unless explicitly specified.

Hash rules:

```text
- self-hash fields exclude themselves
- artifact_hash is derived from canonical artifact bytes
- file_hash is derived from file bytes
- run_artifact_hash is derived from runner artifact bytes
- normalized_result_hash is derived from NormalizedCheckResult canonical bytes
- comparison hashes are derived from normalized comparison payloads
```

Validators report first failures in fixed field order, array index order, and
bytewise tie-break order. Duplicate fields are deterministic validation errors.

---

# 4. MachineCheckRequest

`MachineCheckRequest` is the runner-owned request that materializes a checker
run.

It records:

```text
- request schema / protocol version
- certificate input reference and expected hashes
- import lock manifest
- checker_profile
- trust_mode
- policy_id / axiom policy
- deterministic budget
- runner policy id
- requested output mode
- request_hash
```

AI cannot choose checker binaries directly. It may request a profile, but runner
policy and registry decide the executable.

## 4.1 Runner Policy

Runner policy fixes:

```text
- allowed checker profiles
- binary registry entries
- checker identity manifest
- allowed dynamic arguments
- denied environment variables
- no network by default
- input/output path rules
- resource limits
- high-trust requirements
```

Checker identity includes checker id, version, binary id, binary hash, build
hash, and profile. Identity manifests are deterministic, sorted, and unique by
profile/binary id. Duplicate profile or binary identities are validation
failures.

## 4.2 Runner Command Construction

Commands are built from policy-approved components only.

Dynamic args are limited to:

```text
- certificate path
- import directory / store path
- policy path
- output json path
- deterministic budget flags
```

The runner must not pass AI prompt text, source files, tactic scripts, or search
traces to the checker. It must materialize inputs from validated request
payloads and import locks.

---

# 5. MachineCheckResult

`MachineCheckResult` is the raw runner envelope saved before any AI processing.

It records:

```text
- result schema / protocol version
- request_hash
- checker identity
- process status
- raw checker output reference
- parsed checker status
- module
- export_hash
- certificate_hash
- axiom_report_hash
- axiom list / error kind when available
- run_artifact_hash
- result_hash
```

The runner envelope includes process metadata and resource usage. Those fields
may help audit, but proof validity comes from checker verdict and hashes.

`MachineCheckResult` is stored before normalization and AI sidecars. AI cannot
alter it. Any schema/domain failure is a deterministic validation error.

---

# 6. NormalizedCheckResult

`NormalizedCheckResult` converts one or more `MachineCheckResult` artifacts into
a deterministic comparison object.

It records:

```text
- normalized schema / version
- selected MachineCheckResult references
- normalized per-checker status
- normalized hashes
- normalized axiom summaries
- comparison status
- normalized_result_hash
```

Comparison statuses include:

```text
all_agree_checked
all_agree_failed
checker_disagreement
policy_failure
incomplete
invalid_input
```

`CompareValidationResult valid` does not imply `all_agree_checked`; it only
means the comparison artifact is valid. Disagreement is always release-blocking
where comparison is required.

Normalizer invariants:

```text
- source-copy fields must match raw MachineCheckResult exactly
- source-copy invariant violations are implementation errors
- no partial NormalizedCheckResult is emitted on normalizer invariant failure
- comparison is deterministic and independent of input artifact order
- normalized_result_hash is validated before policy/comparison use
```

---

# 7. AiAuditSidecar

`AiAuditSidecar` is diagnostic metadata bound to checker result hashes.

It may contain:

```text
- summary
- categorized failure explanation
- likely component area
- suggested next commands
- challenge proposal references
- prompt payload hashes
- training labels copied from checker status
```

It must not contain:

```text
- verdict
- pass/fail override
- trusted checker selection
- secrets
- raw credentials
- source of truth for labels
```

The schema is closed-world. Forbidden verdict/source/secret fields are rejected
by deterministic field-name rules. Sidecars are valid only when they point to
existing normalized result hashes and policy hashes.

---

# 8. AI Triage

AI triage reads normalized results and emits sidecars. Triage may classify:

```text
- type mismatch
- conversion failure
- hash mismatch
- import mismatch
- axiom policy failure
- checker crash
- timeout / resource limit
- checker disagreement
```

Triage labels are diagnostic. Training export labels are derived from
`MachineCheckResult` status/error, never from AI sidecar prose.

---

# 9. Disagreement Triage

Checker disagreement is always failure for CI/release gates that require
comparison.

Triage may help identify:

```text
- fast kernel bug
- reference checker bug
- external checker bug
- certificate generator bug
- import/store mismatch
- policy mismatch
```

But it cannot decide that one checker should be ignored. Any waiver is explicit
human/release policy outside AI sidecars.

---

# 10. Challenge Generation

Challenge generation creates adversarial or regression inputs from existing
certificates and results.

Generated challenges include:

```text
- challenge statement hash
- expected import hashes
- policy id
- mutation kind
- outcome hint
- materialized request payload
- challenge artifact hash
```

Challenges are deterministic. Outcome hints are not oracle verdicts; checker
results are the oracle.

---

# 11. Challenge Minimization

Minimization reduces failing challenge inputs while preserving checker-observed
failure. It must rerun materialized checker requests and compare result hashes.
AI may propose minimization, but deterministic checker results decide whether a
minimized challenge is valid.

---

# 12. CI Integration

CI pass/fail uses:

```text
- checker result status
- NormalizedCheckResult comparison
- deterministic AuxiliaryResult values
- release bundle validation
```

CI does not require AI sidecars. If sidecar diagnostics are present, they are
audited as metadata.

The current local gate:

```text
scripts/phase8-release-audit.sh
```

fixes the `npa-checker-ref` binary path, independent checker audit substrate,
standard-library release audit fixture, and AI fast-path boundary.

---

# 13. Release Audit

Release audit is two-phase:

```text
1. pre-bundle staging
   validate explicit artifact path/hash inputs and produce a staging plan

2. bundle generation / validation
   materialize deterministic release bundle artifacts and validate the manifest
```

Release bundle artifacts include:

```text
- canonical certificates
- MachineCheckResult artifacts
- NormalizedCheckResult artifacts
- AuxiliaryResult artifacts
- axiom reports
- checker identity manifests
- optional AI sidecars
- ReleaseAuditBundleManifest
```

Bundle paths, bundle ids, summary ids, and artifact ids are deterministic.
Generation uses only pre-bundle staged artifacts and explicit path/hash inputs.

`ReleaseAuditBundleManifest` entries are sorted and keyed by kind/path. The
manifest validator checks file hashes, parsed artifact hashes, expected schemas,
and cross-artifact references. An audit bundle auxiliary result runs the full
manifest validator, not only file/hash presence checks.

---

# 14. Prompt and Data Policy

Prompt inputs and rendered text are sidecars.

Rules:

```text
- no secrets in prompt artifacts
- prompt hashes are recorded when prompts are saved
- prompts do not affect checker request hashes
- prompts do not affect normalized comparison
- prompts do not affect release pass/fail
- prompt metadata may be included in release bundles for audit only
```

AI sidecars in release bundles must record input policy and prompt hashes so
auditors can reproduce what the AI saw without making the sidecar trusted.

---

# 15. Training Data

Training export reads validated result stores and normalized stores.

Rules:

```text
- labels come only from MachineCheckResult status / error
- absent copied metadata is omitted without skipping records
- selected artifacts must match across stores
- ambiguous retries are rejected
- duplicate run_artifact_hash records are deduplicated only when candidate TrainingExample canonical bytes are identical
- JSON Lines output is one RFC 8785 canonical TrainingExample JSON object plus LF per record, including the final record
- zero records serialize as empty bytes
```

`TrainingExample.input` MVP fields are exactly:

```text
module
checker_profile
policy_id
```

`trust_mode` is not inferred from policy id. `example_id` derives from
`source.run_artifact_hash`; `export_id` derives from JSON Lines file hash.
`--out` and `--manifest-out` are required in the MVP. There is no inline
manifest-only or stdout JSON Lines mode.

Training export manifests are not CI or release audit artifacts.

---

# 16. Security Considerations

Security rules:

```text
- AI cannot select trusted checker binaries
- no remote import resolution in checker requests
- no source re-elaboration as independent verification
- no tactic replay as independent verification
- no AI confidence as pass condition
- no noncanonical certificate acceptance for compatibility
- no self-modifying checker config
- checker registry and policy are runner-owned
```

Future AI may find checker bug candidates, but trust resumes only after fixed
checker binaries produce deterministic results.

---

# 17. Machine Commands

Normative command families:

```text
npa-check request materialize
npa-check run
npa-check normalize
npa-check compare
npa-check ai-sidecar validate
npa-check challenge generate
npa-check challenge materialize
npa-check challenge replay
npa-check auxiliary axiom-policy
npa-check auxiliary reproducibility
npa-check release stage-bundle
npa-check release generate-bundle
npa-check release validate-bundle
npa-check training export
```

In the current repository these contracts are fixed by library APIs and tests in
`crates/npa-api`.

Command errors are structured `CommandError` / `ApiError` responses without
`result_hash`. Transient response types such as `CompareValidationResult`,
`AuditSidecarValidationResult`, `ChallengeGenerationResult`,
`ReleaseBundleStagingResult`, and `NormalizationWriteResult` do not carry
artifact result hashes unless explicitly defined as stored artifacts.

Auxiliary commands exit 0 when they successfully emit failed or inconclusive
`AuxiliaryResult` values. `release validate-bundle` exits 0 when it successfully
emits passed or failed audit-bundle `AuxiliaryResult`.

---

# 18. API Shape

APIs are closed-world schemas with fixed first-failure order. Validation layers:

```text
1. input reference readable / hash match
2. JSON parse
3. schema
4. artifact self-hash
5. manifest-field match
6. cross-store / cross-artifact consistency
7. policy / comparison validation
8. output write
```

Response schemas specify which fields are required or forbidden for each status,
error kind, reason code, and payload shape. Unknown fields are deterministic
errors. Unrepresentable unknown keys are mapped to response-local fallback paths
as specified by each validator family.

---

# 19. Milestones

## M0. Canonical artifact and hash foundation

Define canonical bytes, hash roles, closed-world schemas, and deterministic
validation order.

## M1. Runner policy and checker identity boundary

Fix runner-owned binary registry, checker identity manifest, and allowed dynamic
args.

## M2. Check request materialization and import locks

Materialize checker requests from explicit certificate/import/policy inputs.

## M3. Checker run and MachineCheckResult envelope

Run checkers and store raw runner envelopes before AI processing.

## M4. Result stores and normalization

Store and normalize checker results deterministically.

## M5. Deterministic checker comparison

Compare normalized checker results and make disagreement failure.

## M6. AI audit sidecar schema and prompt input policy

Define sidecars as diagnostic metadata without verdict fields.

## M7. Auxiliary CI pass conditions

Define deterministic `AuxiliaryResult` families for axiom policy,
reproducibility, import certificate hashes, and bundle validation.

## M8. Challenge generation and request materialization

Generate deterministic challenge artifacts and checker requests.

## M9. Challenge replay and coverage summary

Replay challenges and summarize deterministic coverage.

## M10. Required AI sidecar diagnostic gate

Validate required AI sidecar diagnostics as metadata when policy requests them.

## M11. Release policy and bundle staging

Define release policies and pre-bundle staging plans.

## M12. Release audit bundle generation and validation

Generate and validate deterministic release audit bundles.

## M13. Training export and end-to-end integration fixtures

Export training data from checker results and run end-to-end fixtures.

---

# 20. Tests

Tests cover:

```text
- MachineCheckRequest / MachineCheckResult schema and hash stability
- import lock and checker runner dynamic args
- runner policy allowlist and checker identity registry
- command construction excludes AI/source/tactic inputs
- MachineCheckResult is saved before normalization
- NormalizedCheckResult source-copy and comparison invariants
- disagreement is always failure where comparison is required
- AiAuditSidecar cannot contain verdict/source/secret fields
- sidecar hashes bind to normalized result hashes and policies
- challenge generation, materialization, replay, and coverage summaries
- AuxiliaryResult reason codes and deterministic oracle inputs
- axiom report hash, duplicate/order/name validation, and policy checks
- result store / axiom report store manifests with deterministic order and unique keys
- reproducibility checks baseline/repeated rows in fixed order
- import certificate hash auxiliary behavior
- release staging and bundle manifest path/hash semantics
- bundle validator performs full artifact and cross-reference validation
- training export labels derive only from checker results
- JSON Lines training export canonical serialization
- closed-world transient response schemas
- fixed first-failure order and deterministic tie-breaks for all validators
```

The critical invariant is that no AI sidecar can overwrite checker results or
`NormalizedCheckResult.comparison`.

---

# 21. Non-goals

Not included in the Phase 8 AI Profile:

```text
- LLM-based proof checker
- natural language proof acceptance
- source re-elaboration as independent verification
- tactic replay as independent verification
- AI majority vote over checker disagreement
- AI-selected trusted checker binary
- remote import resolution
- self-modifying checker config
- accepting noncanonical certificates for compatibility
- using AI confidence as CI pass condition
```

AI may help find checker bugs in the future. The trust boundary remains the
fixed checker binary and deterministic result after the bug is fixed.

---

# 22. Completion Criteria

Phase 8 AI is complete when:

```text
- MachineCheckRequest / MachineCheckResult schemas are fixed
- import lock schema and checker runner dynamic args are fixed
- checker runner uses only policy allowlist and runner-owned binary registry
- MachineCheckResult runner envelope is stored before AI processing
- NormalizedCheckResult is generated deterministically
- disagreement always becomes failure
- AiAuditSidecar schema cannot carry verdicts
- AI summaries are bound to checker result hash or normalized comparison hash
- challenge generator can create deterministic mutation/outcome-hint reject corpora
- challenge results use checker results as oracle
- challenge replay result store and coverage target are explicit inputs/outputs
- AuxiliaryResult / ChallengeCoverageSummary / ReleaseAuditBundleManifest are generated by deterministic commands
- ReleaseBundleStagingResult and two-phase pre-bundle staging are deterministic
- ReleaseBundleStagingPlan file-hash / parsed-hash semantics are fixed
- release bundle generation uses only pre-bundle staged artifacts and explicit path/hash inputs
- release bundle artifact path, bundle_id, and summary_id are deterministic
- training export labels are derived only from checker results
- CI can decide pass/fail without AI sidecars
- release audit bundles can retain AI sidecar input policy and prompt hashes as metadata
```

Repository gates:

```text
scripts/phase8-release-audit.sh:
  fixes npa-checker-ref binary, independent_checker audit substrate,
  standard-library release audit, and AI fast path boundary.

scripts/phase9-regression.sh:
  later regression gate for Phase 9 fixtures, fmt, clippy, and workspace tests.

GitHub Actions workflow:
  removed in the current repository. External theorem-library CI is reintroduced
  after package contracts are fixed.
```

AI sidecars may appear in release audit bundles as diagnostics / metadata. CI
pass/fail is determined by checker results, normalized comparisons,
deterministic auxiliary results, and release bundle validation.

---

# 23. One-Sentence Summary

Phase 8 AI uses AI around independent checkers as audit assistance, never as a
checker replacement or part of the trusted boundary.
