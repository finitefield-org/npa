# Phase 8 Human Task Breakdown

This task breakdown treats `develop/phase8-human.md` as authoritative and splits
the gap from the current `crates/npa-checker-ref` / `crates/npa-cert` /
`crates/npa-kernel` / `crates/npa-api` implementation into implementation
milestones for the independent checker / external checker contract / CI audit
fixture.

Phase 8 Human is the layer for rechecking canonical `.npcert` through a path
separate from the main kernel without trusting source / tactics / elaborators /
AI search / theorem search. Phase 8 checker audit automation in `crates/npa-api`
is untrusted orchestration substrate that constructs and normalizes checker
requests / results / audit artifacts. The basis for proof acceptance is only the
canonical certificate and deterministic result returned by the independent
checker.

Important constraints:

```text
- Reference checkers / external checkers do not read .npa source, tactic scripts, AI search traces, or theorem indexes.
- The reference checker does not share type checker, conversion checker, or inductive checker implementations with the fast kernel / npa-cert verifier.
- Do not treat `npa_cert::verify_module_cert` as a substitute for the reference checker verdict.
- The external checker runs as a separate process / binary and has no network / plugin / source directory access.
- Phase 8 automation, AI sidecars, challenge generators, and audit summaries do not enter the trusted base.
- Phase 8 audit is not synchronously inserted into the Phase 5 / Phase 7 AI candidate generation hot path.
- In PR mode, keep the required checker profile to `reference`; the external checker is optional / on-demand.
- In nightly / release / high-trust mode, external checker and full audit are required.
- Reject custom axioms / sorry. Allow an exact exception only when the current kernel emits `Std.Logic.Eq.rec` as the standard recursor axiom.
- Do not put I/O, networking, plugin loading, AI calls, or checker runner state into the kernel crate.
```

---

## 0. Current Implementation Boundary

### 0.1 Things Treated As Implemented

The current `crates/npa-cert` has the fast verifier and certificate codec. Phase
8 Human implementation may use these as comparison targets and fixture
generators, but must not reuse them as the reference checker verdict
implementation.

```text
crates/npa-cert/src/lib.rs
- build_module_cert / encode_module_cert / decode_module_cert / verify_module_cert
- AxiomPolicy / VerifierSession / VerifiedModule

crates/npa-cert/src/binary.rs
- canonical binary encode / decode

crates/npa-cert/src/hash.rs
- term / declaration / export / certificate / axiom report hash

crates/npa-cert/src/verify.rs
- current fast verifier implementation
```

The current `crates/npa-kernel` has the fast kernel type checker, conversion,
and inductive checker. It may be compared against as a test oracle for the
reference checker, but must not be called directly and claimed as an independent
checker.

```text
crates/npa-kernel/src/lib.rs
crates/npa-kernel/src/expr.rs
crates/npa-kernel/src/level.rs
crates/npa-kernel/src/env.rs
crates/npa-kernel/src/error.rs
crates/npa-kernel/src/decl.rs
```

The current `crates/npa-api` has checker audit automation substrate for the
Phase 8 AI Profile. This receives requests / results / bundles produced by the
Human Profile standalone checker / CI integration.

```text
crates/npa-api/src/independent_checker.rs
- MachineCheckRequest / MachineCheckResult
- RunnerPolicy / ImportLockManifest / AxiomPolicy TOML
- NormalizedCheckResult / comparison / disagreement
- challenge generation / materialize / replay / coverage summary
- AuxiliaryResult / ReleaseAuditBundleManifest
- AI audit sidecar validation / required sidecar diagnostic gate
- training export labels from checker result only
```

### 0.2 Current Phase 8 Human Completion Boundary

For P8H-00 through P8H-15, the current repository treats the following as
implemented.

```text
- standalone reference checker binary in crates/npa-checker-ref
- source-free decoder / hash verifier / import environment builder inside the reference checker boundary
- minimal type / conversion / simple inductive / axiom report checker in the reference checker
- external checker profile / runner policy / request-result normalization contract in crates/npa-api
- challenge statement hash enforcement and differential disagreement fixtures
- release audit bundle generation / validation substrate and standard-library release audit fixture
- Phase 8 Release Audit fixture workflow
- performance policy that does not require the external checker / full audit on the PR hot path
```

The following still remain target integrations. README and
`develop/phase8-human.md` must not describe them as implemented, and must treat
current repository status separately from target design.

```text
- `npa-checker-ext` release evidence from a built executable selected by
  runner-owned registry / policy and passing package external-mode integration
- verified_high_trust artifact
- full external-checker release audit CI
- production release / high-trust full independent check workflow
- external checker benchmark collection job
```

`checkers/npa-checker-ext/` contains the OCaml clean-room source project, and
the package CLI has an external runner path. That source tree alone is not
release evidence. Docs must treat `npa-checker-ext` as present for
release/high-trust evidence only after a built executable is available in the
fresh-checkout or documented CI environment and the runner policy / checker
registry validate its binary identity and hash.

### 0.3 Things That Must Not Enter The AI Hot Path

The following may be used in Phase 8 Human audit / CI / high-trust flows, but
must not be synchronously inserted into the AI high-volume candidate generation /
search / tactic execution path.

```text
reference checker process
external checker process
audit bundle generation
challenge mutation / challenge replay
AI sidecar triage / summary / suggested challenge
Human source file lookup
source map and pretty goal rendering
release audit bundle validation
full recursive import certificate recheck
nightly / release benchmark collection
```

The AI path keeps the following shape.

```text
Machine Surface request
  -> Phase 5 machine session / tactic batch / replay / verify
  -> Phase 7 candidate ranking / repair / minimization
  -> closed certificate candidate
  -> optional post-acceptance / CI / release audit
```

---

## 1. Design Rules For Protecting The AI Fast Path

Each Phase 8 Human milestone treats the following as acceptance criteria.

```text
- Do not change request / response schemas for `/machine/*` endpoints.
- Do not change Machine `candidate_hash`, `state_fingerprint`, or `replay` / `verify` identity hashes with Phase 8 audit metadata.
- Do not synchronously run the reference / external checker for each tactic candidate expansion.
- Do not require AI sidecars / challenge generation before Phase 5 verify responses.
- Do not block premise retrieval on missing Phase 8 audit results.
- If materialized certificate hashes / NormalizedCheckResult / audit summaries are used, limit them to cache/ranking features.
- Phase 8 AI sidecars cannot overwrite checker results or NormalizedCheckResult.comparison.
- PR mode requires changed-cert checks by the reference checker; the external checker is optional / on-demand.
- Nightly / release / high-trust modes require the external checker and full audit.
- Do not make anything other than the kernel / certificate verifier / independent checker a proof acceptance boundary.
```

---

## 2. Implementation Order

Phase 8 Human first fixes the certificate-only boundary of the reference
checker, then layers on the checker itself, external process runner, and CI /
release audit. Existing `crates/npa-api/src/independent_checker.rs` is reused as
request / result / audit artifact substrate.

```text
1. Fix the Phase 8 Human / AI audit boundary and regression guard
2. Create the reference checker crate / API skeleton
3. Implement the source-free canonical certificate decoder
4. Implement the hash verifier
5. Implement the import store / environment builder
6. Implement the minimal type checker
7. Implement the conversion checker
8. Implement the simple inductive / recursor checker
9. Implement axiom report recomputation / axiom policy
10. Recheck Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic with the reference checker
11. Fix the standalone reference checker binary and external runner contract
12. Connect checker result normalization / disagreement CI
13. Connect challenge mode / audit bundle
14. Fix fuzzing / mutation / differential testing
15. Fix CI modes / performance gates
16. Fix docs / release completion gate
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-cert
cargo test -p npa-kernel
cargo test -p npa-api independent_checker
cargo test -p npa-api std_library
cargo test -p npa-api ai_search
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. Task List

### P8H-00: Fix The Phase 8 Human / AI Audit Boundary

Implementation tasks:

- [x] Reflect the boundary in `develop/phase8-human.md`, `develop/phase8-ai.md`, and README in implementation comments or test names.
- [x] State in public API docs that checker audit automation in `crates/npa-api` is not a trusted checker.
- [x] Add regressions showing Phase 8 audit is not synchronously inserted into the Phase 5 / Phase 7 AI hot path.
- [x] Fix required checker profile differences for PR / nightly / release / high-trust mode in test fixtures.
- [x] Connect the exact standard exception for `Std.Logic.Eq.rec` and custom axiom prohibition to Phase 8 policy docs / tests.

Acceptance criteria:

- [x] Tests fix that AI sidecars, challenge generators, and audit summaries cannot create checker verdicts.
- [x] Fixtures fix that the required profile in PR mode is only `reference`, and the external checker is optional / on-demand.
- [x] `/machine/*` request / response schemas, candidate hashes, and state fingerprints do not change when the Phase 8 boundary is added.

Verification:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api ai_search
cargo test -p npa-api phase7
```

Dependencies:

```text
None
```

Notes:

```text
This milestone does not implement the reference checker itself. It only fixes the boundary and regression guard.
```

### P8H-01: Create The Reference Checker Crate / API Skeleton

Implementation tasks:

- [x] Fix the reference checker location. Candidates are a new `crates/npa-checker-ref` or an equivalent independent crate.
- [x] Fix the public API in the form `check_certificate(cert_bytes, import_store, policy) -> ReferenceCheckResult`.
- [x] Use types that prevent the reference checker from receiving `.npa` source, tactic scripts, AI traces, or theorem indexes.
- [x] Create a skeleton that returns results without calling the fast kernel / `npa_cert::verify_module_cert`.
- [x] Make the error enum structured / deterministic, and add tests that do not depend only on human strings.

Acceptance criteria:

- [x] The reference checker crate does not depend on `npa-api`.
- [x] The reference checker crate does not depend on `npa-tactic` / `npa-frontend`.
- [x] The skeleton returns source-free empty / malformed certificates as deterministic errors.
- [x] Do not use `unsafe`. If it becomes necessary, document the boundary and alternatives.

Verification:

```sh
cargo test -p npa-checker-ref
cargo test -p npa-api independent_checker
cargo clippy --workspace --all-targets -- -D warnings
```

Dependencies:

```text
P8H-00
```

Notes:

```text
The certificate format specification and golden fixtures may be shared. Do not share type checker / conversion checker / inductive checker implementations.
```

### P8H-02: Implement The Source-Free Canonical Certificate Decoder

Implementation tasks:

- [x] Decode the `.npcert` canonical binary inside the reference checker boundary.
- [x] Check magic / format version / core spec version / section order / unknown tags.
- [x] Check canonical order and dangling references in the name / level / term / declaration tables.
- [x] Make unused table entries, duplicate names, and non-normalized level entries deterministic errors.
- [x] Pass the decode test fixture with source path, source map, and debug JSON set to None.

Acceptance criteria:

- [x] Valid golden certificates can be decoded.
- [x] Noncanonical certificates that are semantically readable are rejected.
- [x] Decode error kind / section / offset are comparable in tests.
- [x] The decoder does not perform import resolution, type checking, or AI sidecar validation.

Verification:

```sh
cargo test -p npa-checker-ref decode
cargo test -p npa-cert
```

Dependencies:

```text
P8H-01
```

Notes:

```text
P8H-02 does not perform semantic checks. It is responsible only for decode / canonical shape / table reachability.
```

### P8H-03: Implement The Reference Checker Hash Verifier

Implementation tasks:

- [x] Recompute term hashes inside the reference checker.
- [x] Recompute declaration interface / declaration certificate hashes.
- [x] Recompute export hash / certificate hash / axiom report hash.
- [x] Cross-check domain separation tags against `develop/core-spec-v0.1.md` / `develop/phase2.md`.
- [x] Do not trust stored hashes; make mismatches against recomputed results structured errors.

Acceptance criteria:

- [x] The target object for a hash mismatch is classified deterministically.
- [x] Timestamps, paths, source text, and checker versions do not enter certificate hash inputs.
- [x] Obtain the same golden certificate hash as the fast verifier, but do not use the fast verifier's hash helper as a verdict.
- [x] The hash verifier does not use type correctness as an acceptance basis.

Verification:

```sh
cargo test -p npa-checker-ref hash
cargo test -p npa-cert golden_hashes
```

Dependencies:

```text
P8H-02
```

Notes:

```text
Mechanical codec fixture sharing for hash helpers is allowed, but the fast verifier's pass/fail must not become the reference checker's pass/fail.
```

### P8H-04: Implement The Import Store / Environment Builder

Implementation tasks:

- [x] Build the import certificate store from source-free bytes / checked module interfaces.
- [x] Check import `export_hash` in normal mode.
- [x] Check import `certificate_hash` and same-checker checked status in high-trust mode.
- [x] Add imported public environments to the current module environment in canonical order.
- [x] Make missing imports, export hash mismatches, certificate hash mismatches, and duplicate imports deterministic errors.

Acceptance criteria:

- [x] Do not resolve by import name alone. Always use `export_hash` as the binding.
- [x] High-trust mode cannot use unchecked imported certificates.
- [x] The import store does not perform network access, filesystem package discovery, or remote imports.
- [x] Imported axiom dependencies do not disappear through hidden/private export filtering.

Verification:

```sh
cargo test -p npa-checker-ref import
cargo test -p npa-cert import
cargo test -p npa-api independent_checker
```

Dependencies:

```text
P8H-03
```

Notes:

```text
The import store sees only the certificate set explicitly passed by the runner. It does not perform automatic import insertion.
```

### P8H-05: Implement The Minimal Type Checker And Declaration Check

Implementation tasks:

- [x] Implement inference / checking for Sort / Pi / Lam / App / Let / Const.
- [x] Align de Bruijn indexes / binder scope / context lookup with the specification.
- [x] Check `type : Sort` and `value/proof : type` for AxiomDecl / DefDecl / TheoremDecl.
- [x] Recheck declaration order and dependency order.
- [x] Structure errors as type mismatch, unknown reference, expected function, expected sort, and similar cases.

Acceptance criteria:

- [x] Well-typed theorems / definitions pass.
- [x] Ill-typed applications / wrong theorem proofs are rejected.
- [x] Theorem proofs are registered as opaque exports, and untrusted theorem unfolding is not performed.
- [x] The checker does not use source pretty text, Human name shortening, or notation.

Verification:

```sh
cargo test -p npa-checker-ref type_check
cargo test -p npa-kernel checks_
cargo test -p npa-cert
```

Dependencies:

```text
P8H-04
```

Notes:

```text
P8H-05 may keep conversion to minimal structural equality. The real βδζι implementation is fixed in a later milestone.
```

### P8H-06: Implement The Conversion Checker

Implementation tasks:

- [x] Implement WHNF inside the reference checker.
- [x] Implement β reduction.
- [x] Implement δ reduction according to reducibility metadata.
- [x] Implement ζ reduction.
- [x] Implement definitional equality for Pi / Lam / App / Sort / Const / BVar.
- [x] Treat fuel / recursion bounds as deterministic errors.

Acceptance criteria:

- [x] Positive and negative examples for β / δ / ζ are fixed in tests.
- [x] Do not add η conversion, proof irrelevance conversion, quotient computation, or untrusted theorem unfolding.
- [x] Accept the same certificates as the fast kernel, but do not share the conversion implementation.
- [x] Do not use a conversion cache, or if one is used, limit it to deterministic optimizations that do not affect semantics.

Verification:

```sh
cargo test -p npa-checker-ref conversion
cargo test -p npa-kernel reduces_
cargo test -p npa-kernel rejects_
```

Dependencies:

```text
P8H-05
```

Notes:

```text
Prioritize readability against the specification over performance. Decide on optimization after the P8H-14 benchmark.
```

### P8H-07: Implement The Simple Inductive / Recursor Checker

Implementation tasks:

- [x] Check inductive parameters / indexes / result sorts / constructor types.
- [x] Check that constructor results return the target inductive family.
- [x] Implement the MVP positivity checker with a conservative specification sufficient for Nat / Eq / List.
- [x] Check that generated recursor types and iota rules match declarations.
- [x] Connect WHNF ι reduction to recursor applications.

Acceptance criteria:

- [x] Valid inductive certificates for Nat / Eq / List pass.
- [x] Negative occurrences, nested inductives, and mutual inductives are deterministically rejected in the MVP.
- [x] Recursor result mismatches / constructor result mismatches become structured errors.
- [x] Positive and negative examples for ι reduction are compared against the fast kernel in differential tests.

Verification:

```sh
cargo test -p npa-checker-ref inductive
cargo test -p npa-checker-ref iota
cargo test -p npa-kernel inductive
```

Dependencies:

```text
P8H-06
```

Notes:

```text
Advanced inductives from Phase 9 are out of scope. The MVP is limited to Nat / Eq / List / simple generated artifacts.
```

### P8H-08: Implement Axiom Report Recalculation / Axiom Policy

Implementation tasks:

- [x] Recalculate direct / transitive axiom dependencies for each declaration.
- [x] Compare the module axiom report with the report inside the certificate.
- [x] Recalculate the axiom report hash and reject stale reports.
- [x] Connect policy-file `deny_sorry` / `deny_custom_axioms` / allowed axiom sets to the checker boundary.
- [x] Treat the exact `Std.Logic.Eq.rec` standard exception separately from custom axioms.

Acceptance criteria:

- [x] Certificates whose actual dependencies were removed from the axiom report are rejected.
- [x] Custom axioms / synthetic sorry are rejected by high-trust policy.
- [x] Classical axioms other than `Std.Logic.Eq.rec` are rejected by the MVP standard library policy.
- [x] Axiom policy failures can be classified deterministically as either checker results or auxiliary results.

Verification:

```sh
cargo test -p npa-checker-ref axiom
cargo test -p npa-cert axiom
cargo test -p npa-api independent_checker
```

Dependencies:

```text
P8H-07
```

Notes:

```text
Axiom policy does not change the core typing rules. Treat proof validity and release/high-trust pass conditions separately.
```

### P8H-09: Recheck Standard Library Certificates With The Reference Checker

Implementation tasks:

- [x] Pass the MVP certificate fixtures for `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic` through the reference checker.
- [x] Check the import closure and `export_hash` / high-trust `certificate_hash` with the reference checker.
- [x] Add regressions showing that the standard library theorem index / rewrite profile / simp profile are not used as a basis for checker acceptance.
- [x] Cross-check hashes / axiom reports between Phase 6 release artifacts and reference checker results.
- [x] Fix in tests that source package skeletons and Human debug views do not enter checker input.

Acceptance criteria:

- [x] The four MVP release modules are accepted by the reference checker with source None.
- [x] Standard library modules containing custom axioms are rejected.
- [x] Even if the theorem index is broken, certificate check results are determined only from certificate bytes.
- [x] Machine API / Phase 7 retrieval candidate hashes do not change when P8H-09 is added.

Verification:

```sh
cargo test -p npa-checker-ref std
cargo test -p npa-api std_library
cargo test -p npa-api ai_search
```

Dependencies:

```text
P8H-08
```

Notes:

```text
P8H-09 does not extend the standard library. It makes existing Phase 6 artifacts targets for the independent checker.
```

### P8H-10: Fix The Standalone Checker Binary / External Runner Contract

Implementation tasks:

- [x] Add the `npa-checker-ref` binary or an equivalent standalone checker binary.
- [x] Implement or fix by wrapper the `npa-checker-ext` runner contract as the target CLI.
- [x] The runner selects checker binaries only from the policy allowlist and runner-owned binary registry.
- [x] Limit dynamic args to certificate / import dir or import lock / policy / output JSON.
- [x] Fix the runner sandbox policy as no network, read-only cert dir, no source mount, and no plugin.

Acceptance criteria:

- [x] AI / requests cannot specify arbitrary binary paths, extra flags, env vars, or cwd overrides.
- [x] Raw checker output is saved to MachineCheckResult and materialized before AI sidecars.
- [x] Process launched / exit status / checker id / binary hash are recorded deterministically.
- [x] Malformed raw output becomes a structured failure, not checker success.

Verification:

```sh
cargo test -p npa-checker-ref --bin npa-checker-ref
cargo test -p npa-api independent_checker
cargo clippy --workspace --all-targets -- -D warnings
```

Dependencies:

```text
P8H-09
```

Notes:

```text
The external checker is an operational separation milestone. Do not delegate the correctness of the reference checker to AI sidecars or runner policy.
```

### P8H-11: Connect Checker Result Normalization / Disagreement CI

Implementation tasks:

- [x] Convert fast kernel / reference / external checker raw results to `MachineCheckResult`.
- [x] Generate `NormalizedCheckResult` deterministically in required checker profile order.
- [x] Fix comparison statuses for checked / failed / resource exhausted / missing checker results.
- [x] Add a gate that treats checker disagreement as a CI failure.
- [x] Use required profile `reference` for PR mode, `reference, external` for nightly, and `fast-kernel, reference, external` for release.

Acceptance criteria:

- [x] Normalization does not pass if the checker result hash / policy hash / request hash mismatch.
- [x] Missing optional checkers do not break the PR pass condition, and missing required checkers are failures.
- [x] AI sidecars cannot overwrite comparison status / result hash.
- [x] Disagreement reports include module / declaration / checker profile / certificate hash.

Verification:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api independent_checker_normalized
cargo test -p npa-api phase9
```

Dependencies:

```text
P8H-10
```

Notes:

```text
P8H-11 is the milestone that connects the existing Phase 8 AI substrate to real checker outputs.
```

### P8H-12: Connect Challenge Mode / Audit Bundle

Implementation tasks:

- [x] Fix challenge-file statement_core_hash / allowed axioms / import hashes as schema.
- [x] Check that the proof certificate theorem statement hash matches the challenge.
- [x] Materialize proof `.npcert`, imports, policy, checker outputs, hashes, and axiom report into the audit bundle.
- [x] Include real MachineCheckResult / NormalizedCheckResult / AuxiliaryResult in ReleaseAuditBundleManifest.
- [x] Make bundle validation rerunnable with source / tactic / AI trace None.

Acceptance criteria:

- [x] Challenge statement mismatches become deterministic failures.
- [x] Missing imports, wrong certificate hashes, and forbidden axioms become failures in audit bundle validation.
- [x] AI sidecars do not change release pass/fail decisions, even if included as bundle metadata.
- [x] High-trust audit bundles can be checked under the assumptions source ignored / no network / no plugin.

Verification:

```sh
cargo test -p npa-api independent_checker_release
cargo test -p npa-api independent_checker_challenge
cargo test -p npa-checker-ref audit
```

Dependencies:

```text
P8H-11
```

Notes:

```text
Use only the statement hash and allowed axiom policy fixed by the challenge owner. AI does not choose the expected verdict.
```

### P8H-13: Fix Fuzzing / Mutation / Differential Testing

Implementation tasks:

- [x] Pass malformed certificate fuzz cases to the reference checker and reject them without panicking.
- [x] Create proof mutation fixtures and compare that the fast kernel / reference / external checker reject them.
- [x] Add axiom report mutation, import hash mutation, and noncanonical table mutation to the challenge corpus.
- [x] Save fast kernel OK / reference NG and reference OK / external NG cases as failures in differential tests.
- [x] Connect challenge replay results to the checker result oracle.

Acceptance criteria:

- [x] If a mutation target is invalid, do not confuse generator failure with checker rejection.
- [x] Outcome hints are test helpers and do not replace checker results.
- [x] Accepted mutations become unexpected checker acceptance CI failures.
- [x] Fuzz / mutation tests record deterministic seeds and artifact hashes.

Verification:

```sh
cargo test -p npa-checker-ref fuzz
cargo test -p npa-api independent_checker_challenge
cargo test -p npa-cert mutation
```

Dependencies:

```text
P8H-12
```

Notes:

```text
Distributed fuzzing and external SMT certificate checkers are not included in the Phase 8 MVP.
```

### P8H-14: Fix CI Modes / Performance Gates

Implementation tasks:

- [x] Fix the PR / nightly / release / high-trust CI command sets in repository scripts or documented workflows.
- [x] Make PR mode changed certs + reverse dependencies + reference checker required.
- [x] Make the external checker / full recursive import check / full audit bundle required for nightly / release / high-trust.
- [x] Split performance benchmarks into fast kernel, Machine API, theorem index build, AI benchmark, and reference/external checker.
- [x] Do not put reference / external checker benchmarks in the synchronous required PR job; handle them as a separate job or cached audit result.

Acceptance criteria:

- [x] PR AI candidate hot-path latency does not increase when Phase 8 audit is added.
- [x] Nightly / release records detailed benchmarks for the reference / external checker.
- [x] Release mode does not pass unless full independent check and audit bundle generation pass.
- [x] High-trust mode satisfies all imports recursively checked and at least two independent checkers required.

Verification:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

Dependencies:

```text
P8H-13
```

Notes:

```text
Benchmark policy is not a proof acceptance boundary. Treat performance results as regression gates / release policy.
```

### P8H-15: Fix Docs / Release Completion Gate

Implementation tasks:

- [x] Update implemented boundaries in README, `develop/phase8-human.md`, and `develop/phase8-ai.md`.
- [x] Align command examples for the standalone checker binary, external checker runner, and CI audit with real commands.
- [x] Tie Phase 8 completion criteria to test / local script gates.
- [x] Document the role difference between Phase 9 regression and the Phase 8 release audit fixture gate.
- [x] Specify storage locations for release / high-trust audit artifacts and the generated artifact policy.

Acceptance criteria:

- [x] Docs and tests agree that the reference checker checks `.npcert` with source None, and the external profile / runner contract is fixed by source-free request / result fixtures.
- [x] Fast kernel / reference / external checker comparison failures are documented as release blockers.
- [x] The difference between current repository status and target design is not stale.
- [x] Phase 8 docs do not put AI sidecars in the trust boundary.

Verification:

```sh
git diff --check
rg -n "standalone CLI binary does not exist yet|external checker on changed certs|AI sidecar.*pass condition" README.md develop/phase8-human.md develop/phase8-ai.md
cargo test --workspace
./scripts/phase9-regression.sh
```

Dependencies:

```text
P8H-14
```

Notes:

```text
P8H-15 is final documentation / gate alignment. Do not describe unimplemented features as implemented in README.
```

---

## 4. Completion Criteria

The Phase 8 Human MVP can be considered complete in the current repository when the following conditions hold.

```text
- .npcert can be checked by the reference checker with source None.
- The reference checker has type / conversion / inductive checkers independent from the fast kernel / npa-cert verifier.
- The standalone reference checker binary works as a source-free checker.
- The external checker runner contract is fixed by crates/npa-api policy / request / result fixtures.
- Import export_hash / high-trust certificate_hash can be checked.
- Declaration hash / export_hash / certificate_hash / axiom_report_hash can be recalculated.
- Axiom reports can be recalculated, and custom axioms / sorry can be rejected.
- Only the exact Std.Logic.Eq.rec standard exception can be treated separately from custom axioms.
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic certificates can be rechecked with source None.
- The fast kernel / reference / external profiles are compared with deterministic fixtures.
- CI / release audit fails on checker disagreement.
- Audit bundles can be generated and checked.
- Release / high-trust full independent check requirements are tied to policy tests and the Phase 8 Release Audit fixture gate.
- Phase 8 audit does not increase normal AI candidate hot-path latency.
```

Release-ready binary evidence for `npa-checker-ext`, the verified_high_trust
artifact, and full external-checker release audit CI are target integrations
after this MVP. Even if the OCaml source project and package external runner
path exist, they are not release evidence until a built executable is verified
by the runner-owned registry / policy.

---

## 5. Things Not Included In The MVP

The MVP does not include the following.

```text
- formally verified checker
- full support for mutual / nested inductives
- quotient computation
- proof irrelevance conversion
- η conversion
- external SMT certificate checker
- distributed certificate verification
- cryptographic signature infrastructure
- AI majority vote over checker disagreement
- AI-selected trusted checker binary
- source re-elaboration as independent verification
```
