# Phase 2 implementation TODO

This TODO treats `develop/phase2.md` as authoritative and breaks down the gap
from the current `crates/npa-cert` implementation into implementation tasks.

At the time of investigation, the implementation substantially satisfies the
following as the existing Phase 2 certificate verifier.

```text
- generates ModuleCert from CoreModule
- performs canonical binary encode/decode
- confirms canonical bytes by checking re-encode equality after decode
- checks canonical order and reachability for the name / level / term tables
- checks canonical order for import / declaration / export block / axiom report
- recomputes level / term / declaration / export / axiom report / module certificate hashes
- checks normal / high-trust import policy
- reconstructs the kernel environment from the verified import store
- checks axiom reports and axiom policy from recomputed results, not stored values
- checks inductive constructor / recursor exports and generated artifact mismatches
- passes decoded declarations to the Phase 1 Rust kernel for rechecking
```

The main unimplemented scope is the Human producer / AI producer separation,
AI candidate fast path, checked token, producer public environment fingerprint,
and producer separation tests added in `develop/phase2.md` 7.1 / 11.1.1 /
12.7. There are also small differences between the new hash payload contract
and the current implementation.

---

## 0. Boundary Between Implemented And Unimplemented

### 0.1 Items Treated As Implemented

The following may be used as assumptions for this producer separation
implementation.

```text
crates/npa-cert/src/lib.rs
- build_module_cert
- encode_module_cert
- decode_module_cert
- verify_module_cert
- term_hash

crates/npa-cert/src/types.rs
- CoreModule
- ModuleCert
- VerifiedModule
- VerifierSession
- AxiomPolicy / TrustMode
- ImportEntry / ImportKey
- DeclCert / DeclPayload
- GlobalRef
- DependencyEntry / AxiomRef
- ExportBlock / AxiomReport

crates/npa-cert/src/canonical.rs
- build_module_cert_impl
- canonical declaration ordering
- import sorting / exact import dedup
- name / level / term table construction
- dependency and axiom dependency construction

crates/npa-cert/src/verify.rs
- canonical round-trip check
- table order / reachability / bvar scope checks
- hash recomputation
- import resolution
- axiom policy enforcement
- kernel recheck
```

### 0.2 Items With No Implementation

The current code has no types, APIs, or tests for the following.

```text
ProducerProfile
ProducerLimits
CoreDeclCandidate
CandidateBatch
CheckedDeclCandidate
CandidateHashPreview
CandidateStatus
CandidateBatchResult
check_core_decl_candidates
build_module_cert_from_checked_candidates
ProducerImportEnvKey
ProducerCheckedDeclInterface
ProducerLookupEnv
ProducerPriorChainEntry
producer_limits_hash
stricter_or_equal
producer_env_fingerprint
prior_chain_fingerprint
DuplicateImportEnvKey
```

---

## 1. Fix Hash Payload Contract Differences

### P2-01: Align The `decl_interface_hash` Payload Order For Def With `phase2.md`

Current state:

```text
decl_interface_payload(Def) in crates/npa-cert/src/hash.rs:
  kind, name, universe_params, type_hash,
  value_hash if reducible,
  reducibility,
  public_dependency_entries,
  axiom_dependencies
```

Requirement from `develop/phase2.md`:

```text
Def:
  kind, name, universe_params, type_hash, reducibility,
  public_dependency_entries, axiom_dependencies,
  value_hash only when reducibility = reducible
```

Implementation tasks:

- Change the Def branch of `decl_interface_payload` to the field order in the specification.
- Preserve the policy that `value_hash` is included only for reducible definitions.
- After the change, update the golden hash fixture.
- Confirm that the existing hash role tests pass with the new payload order.

Affected files:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

Completion criteria:

```text
- canonical payload order for Def interface hash matches phase2.md 11.11
- existing tests for binder name stability / transparent body change / opaque body change pass
- golden fixtures are updated to the new hashes
```

### P2-02: Always Put `value_hash` Into Def `decl_certificate_hash`

Current state:

```text
decl_certificate_payload in crates/npa-cert/src/hash.rs:
- opaque defs include value_hash
- reducible defs do not include value_hash directly, and reflect it only through decl_interface_hash
```

Requirement from `develop/phase2.md`:

```text
DeclCertificatePayload.Def:
  decl_interface_hash, value_hash, dependency entries, axiom_dependencies
```

Implementation tasks:

- Encode `value_hash` explicitly for every Def in the Def branch of `decl_certificate_payload`, without branching on reducible / opaque.
- Even for reducible defs where the value enters through `decl_interface_hash`, include `value_hash` redundantly as part of the payload contract.
- The `rehash_cert_after_decl_change` helper and fixtures will need updates.

Affected files:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

Completion criteria:

```text
- both transparent defs and opaque defs include value_hash in the decl_certificate_hash payload
- transparent def body changes change decl_certificate_hash / certificate_hash
- opaque def body changes change decl_certificate_hash / certificate_hash while preserving export_hash
```

### P2-03: Make Generated Artifact Hashes Explicit In The Inductive Interface Payload

Current state:

```text
decl_interface_payload(Inductive) in crates/npa-cert/src/hash.rs:
  directly encodes constructor name/type hashes and recursor name/universe_params/type/rules
```

Requirement from `develop/phase2.md`:

```text
Inductive:
  kind, name, universe_params, params, indices, sort,
  constructors,
  generated recursor signature hash,
  generated computation rule hash,
  public_dependency_entries,
  axiom_dependencies
```

Implementation tasks:

- Define canonical payloads for `generated_recursor_signature_hash` and `generated_computation_rule_hash`.
- Split the currently directly encoded recursor type / rules into the hash payloads above.
- On the constructor side, clearly separate in code what the specification puts directly into `constructors` and what goes into generated artifact hashes.
- Use the same canonical bytes for hash calculation as the regeneration logic in `verify_inductive_generated_artifacts`.

Affected files:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/verify.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

Completion criteria:

```text
- Inductive decl_interface_hash payload matches the field names in phase2.md 11.11
- recursor type / rule tamper tests continue to fail
- stability tests exist for generated recursor signature / computation rule hash
```

---

## 2. Add Producer API Types

### P2-04: Add A Producer Module To `crates/npa-cert`

Current state:

```text
crates/npa-cert exposes only certificate build / verify APIs.
There are no types or functions for the AI candidate fast path.
```

Implementation tasks:

- Add `crates/npa-cert/src/producer.rs`.
- Re-export `mod producer;` and the necessary public types / functions from `lib.rs`.
- Do not encode producer types into trusted certificate payloads.
- Keep `ProducerProfile` only for sidecar / audit use; do not put it into arguments for `build_module_cert` / `verify_module_cert`.

Types to add:

```rust
pub enum ProducerProfile {
    HumanSurface,
    AiCoreMvp,
}

pub struct ProducerLimits {
    pub max_declarations: u32,
    pub max_expr_nodes: u32,
    pub max_level_nodes: u32,
    pub max_name_components: u32,
    pub max_reduction_steps: u64,
    pub max_conversion_steps: u64,
}

pub struct CoreDeclCandidate {
    pub declaration: npa_kernel::Decl,
}

pub struct CandidateBatch<'a> {
    pub imports: &'a [VerifiedModule],
    pub prior_current_decls: &'a [CheckedDeclCandidate],
    pub candidates: Vec<CoreDeclCandidate>,
    pub limits: ProducerLimits,
}

pub struct CandidateHashPreview {
    pub type_hash: Option<Hash>,
    pub body_hash: Option<Hash>,
    pub decl_interface_hash: Option<Hash>,
    pub decl_certificate_hash: Option<Hash>,
}

pub enum CandidateStatus {
    Accepted(CheckedDeclCandidate),
    Rejected(CertError),
}

pub struct CandidateBatchResult {
    pub statuses: Vec<CandidateStatus>,
}
```

Completion criteria:

```text
- producer API types are usable from the npa-cert public API
- ProducerProfile does not mix into the certificate build / verify path
- cargo doc / clippy missing docs pass
```

### P2-05: Implement `CheckedDeclCandidate` As An Opaque Token

Current state:

```text
There is no CheckedDeclCandidate type.
There is also no boundary that prevents arbitrary callers from treating raw
npa_kernel::Decl values as tokens.
```

Implementation tasks:

- Make the fields of `CheckedDeclCandidate` private.
- Keep constructors only inside `check_core_decl_candidates`.
- Do not provide public getters that extract raw declarations.
- If diagnostic getters are added, limit them to non-authoritative values such as preview hash / interface hash.

Internal fields:

```rust
declaration: npa_kernel::Decl,
preview_hashes: CandidateHashPreview,
pre_env_fingerprint: Hash,
post_env_fingerprint: Hash,
prior_chain_fingerprint: Hash,
limits: ProducerLimits,
limit_profile_hash: Hash,
decl_interface_hash: Hash,
decl_certificate_hash: Hash,
```

Completion criteria:

```text
- external crates cannot directly construct CheckedDeclCandidate
- callers cannot build raw CoreModule values from tokens
- only build_module_cert_from_checked_candidates can use declarations inside tokens
```

---

## 3. Producer Limits And Deterministic Resource Check

### P2-06: Implement `ProducerLimits` Canonical Bytes / Hash

Current state:

```text
There is no ProducerLimits type or producer_limits_hash.
The kernel side has fixed fuel and metered APIs, but they are not used as a
limit profile from the candidate fast path.
```

Implementation tasks:

- Implement canonical encode for `ProducerLimits` with fixed struct field order.
- Encode each field as minimal ULEB128.
- Implement `producer_limits_hash(limits)`.
- Fix the domain separator as `"NPA-PRODUCER-LIMITS-0.1"`.
- Implement `stricter_or_equal(a, b)`.

Completion criteria:

```text
- identical limits produce the same producer_limits_hash
- tests show that changing field order changes the hash
- stricter_or_equal is judged by <= for every field
```

### P2-07: Apply Deterministic Limits To Candidate Precheck

Current state:

```text
npa-kernel::Env has check_with_fuel_metered / infer_with_fuel_metered and
similar APIs. Meanwhile, add_decl_to_env / build_module_cert in npa-cert use
ordinary kernel checks and do not receive ProducerLimits.
```

Implementation tasks:

- Create a declaration precheck function dedicated to the producer fast path.
- Map `max_reduction_steps` / `max_conversion_steps` to kernel WHNF / conversion fuel.
- Check `max_declarations` / `max_expr_nodes` / `max_level_nodes` / `max_name_components` during candidate schema validation.
- Make limit overflows per-candidate `Rejected(CertError)`, or batch-level `Err(CertError)` when they concern the whole batch schema. Fix the classification in tests.

Completion criteria:

```text
- limit profiles are saved in tokens
- strictness can be judged between token-creation limits and batch.limits
- errors on limit overflow are deterministic
```

---

## 4. Producer Public Environment Fingerprint

### P2-08: Implement `ProducerImportEnvKey` And Import Order Validation

Current state:

```text
build_module_cert sorts / exact-deduplicates imports by (module, export_hash,
Some(certificate_hash)). There is no canonical order check for
CandidateBatch.imports or duplicate rejection for ProducerImportEnvKey(module,
export_hash). CertError::DuplicateImportEnvKey also does not exist.
```

Implementation tasks:

- Add `CertError::DuplicateImportEnvKey { module: ModuleName, export_hash: Hash }`.
- Check that `CandidateBatch.imports` are in the same canonical import order as `ModuleCert.Imports`.
- Reject duplicate `ProducerImportEnvKey(module, export_hash)` as a batch-level error.
- Prevent imports with the same module / export_hash but different certificate_hash from being submitted to the candidate fast path at the same time.

Completion criteria:

```text
- noncanonical CandidateBatch.imports is Err(CertError::NonCanonicalEncoding { object: "Imports" })
- duplicate ProducerImportEnvKey is Err(CertError::DuplicateImportEnvKey)
- the correspondence between GlobalRef::Imported(import_index, ...) and batch.imports[import_index] does not break
```

### P2-09: Implement `ProducerEnvFingerprintBytes`

Current state:

```text
There is no producer_env_fingerprint.
The public-environment reuse unit for imports is confined to VerifiedModule /
ExportBlock inside the certificate implementation.
```

Implementation tasks:

- Define `ProducerImportEnvKey { module, export_hash }`.
- Define `ProducerCheckedDeclInterface { decl_interface_hash, axiom_dependencies }`.
- Canonically encode `ProducerEnvFingerprintBytes` in fixed record order.
- Implement `producer_env_fingerprint(env)`.
- Fix the domain separator as `"NPA-PRODUCER-ENV-0.1"`.

Completion criteria:

```text
- if only the proof body of an import changes and module/export_hash are the same, the fingerprint is preserved
- changing checked_decls order changes the fingerprint
- axiom_dependencies order is normalized to canonical order
```

### P2-10: Implement `ProducerLookupEnv` And Axiom Dependency Recalculation

Current state:

```text
Certificate generation calculates axiom deps from imported ExportBlock and
prior declarations. There is no API that separates fingerprint bytes and lookup
environment for the producer fast path.
```

Implementation tasks:

- Add `ProducerLookupEnv`.
- Ensure `canonical_import_env_keys(imports)` and `canonical_import_export_views(imports)` preserve the same import order.
- Implement `producer_checked_decl_interface(decl, lookup_env)`.
- Recompute axiom dependencies by the same rules as existing certificate generation, not from AI producer reports or preview hashes.
- Do not look up axiom deps inside imports from only `ProducerImportEnvKey(module, export_hash)`.

Completion criteria:

```text
- indexes of canonical_import_env_keys and canonical_import_export_views match
- GlobalRef::Imported(import_index, ...) points to the same import in imports / direct_imports / import_exports
- imported axiom deps are recalculated from the lookup view derived from VerifiedModule.export_block
```

### P2-11: Implement `post_env_fingerprint` By Full Recompute

Current state:

```text
There is no incremental producer env fingerprint.
```

Implementation tasks:

- Implement `initial_env_fingerprint(imports)`.
- Implement `post_env_fingerprint(imports, checked_decls_before, decl)`.
- Recompute from the full imports and checked declaration interface sequence instead of appending to the previous fingerprint.
- Ensure the presence or absence of an incremental cache does not affect the fingerprint.

Completion criteria:

```text
- fingerprints match for the same accepted module with cache enabled / disabled
- the same checked_decls_before and decl produce the same post_env_fingerprint
```

---

## 5. Prior Chain Fingerprint And Token Validation

### P2-12: Implement `ProducerPriorChainEntry` / `prior_chain_fingerprint`

Current state:

```text
There is no hash that fixes the exact token sequence of checked declarations
inside the current module.
```

Implementation tasks:

- Define `ProducerPriorChainEntry`.
- Canonically encode `ProducerPriorChainBytes`.
- Implement `prior_chain_fingerprint(chain)`.
- Fix the domain separator as `"NPA-PRODUCER-CHAIN-0.1"`.

Entry fields:

```text
decl_interface_hash
decl_certificate_hash
pre_env_fingerprint
post_env_fingerprint
```

Completion criteria:

```text
- empty chain fingerprint is deterministic
- changing declaration order changes prior_chain_fingerprint
- a body-only change to an opaque theorem proof / opaque def body preserves producer public env,
  while the prior chain changes due to the decl_certificate_hash difference
```

### P2-13: Implement `prior_current_decls` Token Validation

Current state:

```text
There is no CandidateBatch.prior_current_decls.
There is also no check for safely reusing previously checked current-module
declaration tokens in a batch.
```

Implementation tasks:

- Check that the first token's `pre_env_fingerprint` matches the initial env fingerprint.
- Check that the `pre_env_fingerprint` of the second and later tokens matches the previous token's `post_env_fingerprint`.
- Check that the token's `prior_chain_fingerprint` matches the chain of accepted prior declarations before it.
- Recompute private `decl_interface_hash` / `decl_certificate_hash` from the declaration and compare them.
- Check `producer_limits_hash(token.limits) == token.limit_profile_hash`.
- Check that the token-creation limits are identical to current batch limits or stricter_or_equal.
- Recompute the token's `post_env_fingerprint` and compare it.

Completion criteria:

```text
- invalid prior tokens are batch-level Err(CertError), not per-candidate rejections
- forged tokens / mismatched chains / mismatched env fingerprints are rejected
- stricter prior tokens can be reused, and looser prior tokens are rejected
```

---

## 6. Candidate Fast Path Body

### P2-14: Implement `check_core_decl_candidates`

Current state:

```text
There is no API that performs schema validation / import ref validation /
kernel precheck without creating a certificate for each candidate.
```

Implementation tasks:

- Check the schema of the whole `CandidateBatch`.
- Check canonical order of `imports` / duplicate ProducerImportEnvKey.
- Validate `prior_current_decls` by the P2-13 rules and add them to the environment.
- Check `candidates` sequentially in input order.
- For each candidate:
  - Check that no unresolved metavariable / placeholder equivalent is represented.
  - Check size limits for name / level / expr nodes.
  - Check resolution of import GlobalRef / local ref / generated ref / builtin ref.
  - Run kernel precheck.
  - Recompute dependency / axiom dependency / decl hashes.
  - Create `CheckedDeclCandidate`.
- Return failed candidates as `Rejected(CertError)`.
- Fix the boundary between batch-level failures and per-candidate failures in tests.

Completion criteria:

```text
- Ok(result) has statuses.len() == candidates.len()
- statuses[i] is the result for candidates[i], and results are not reordered by score/hash/cache state
- Accepted cannot be treated as VerifiedModule
- .npcert bytes / certificate_hash are not created by this API
```

### P2-15: Fix The Boundary Between Name-Based `npa_kernel::Decl` And Hash-Bound `GlobalRef`

Current state:

```text
npa_kernel::Expr::Const has only a name string + levels.
Inside certificates, it is resolved to GlobalRef::{Imported, Local,
LocalGenerated, Builtin}. The AI producer MVP text requires that pretty-only /
fully-qualified names alone are not treated as certificate-facing core terms.
```

Implementation tasks:

- If `CoreDeclCandidate { declaration: npa_kernel::Decl }` is kept, always fix it to `GlobalRef` through the certificate resolver during candidate validation.
- Do not fill in `GlobalRef` from preview hashes / dependency reports / scores passed by the AI producer.
- Names that cannot be resolved to any import / prior declaration / builtin become `Rejected(CertError::UnknownDependency)`.
- Add an internal resolved candidate representation if needed, but do not put it into the public certificate schema.

Completion criteria:

```text
- pretty-only names do not become the basis for Accepted tokens
- private decl hashes inside Accepted tokens are recomputed from resolved GlobalRef payloads
- import decl_interface_hash mismatches are rejected
```

### P2-16: Implement `build_module_cert_from_checked_candidates`

Current state:

```text
There is no API that creates the final ModuleCert from Accepted tokens.
```

Implementation tasks:

- Revalidate the token chain for `imports` and `checked_decls`.
- Recompute and compare `pre_env_fingerprint` / `post_env_fingerprint` / `prior_chain_fingerprint`.
- Compare `producer_limits_hash(token.limits) == token.limit_profile_hash`.
- Recompute and compare private `decl_interface_hash` / `decl_certificate_hash`.
- Construct `CoreModule` internally only if the chain matches exactly.
- Pass the constructed `CoreModule` to existing `build_module_cert`.
- This API does not perform strictness comparison against new `ProducerLimits`.

Completion criteria:

```text
- token chain mismatches are rejected
- token order mismatches are rejected
- forged tokens are rejected
- generated ModuleCert values do not enter the trusted import store until they pass verify_module_cert
```

---

## 7. Producer Sidecar And Trusted Payload Separation

### P2-17: Keep Producer Sidecars Separate From Certificate Payloads

Current state:

```text
There is no Phase 2 producer sidecar type.
Certificate payloads do not contain source maps / diagnostics / AI traces.
```

Implementation tasks:

- If needed, define `ProducerSidecar` in an upper crate or separate artifact, not in `npa-cert`.
- Do not encode `ProducerProfile` / model name / prompt / score / diagnostics / cache hit into `.npcert`.
- Even if sidecars are created, do not make them inputs to `export_hash` / `axiom_report_hash` / `certificate_hash`.

Completion criteria:

```text
- adding, removing, or changing sidecar contents does not change .npcert bytes or any hashes
- verify_module_cert passes with sidecar None
```

---

## 8. Tests

### P2-18: Update Hash Payload Contract Tests

Implementation tasks:

- Update golden fixtures for P2-01 / P2-02 / P2-03.
- Add a regression test for `decl_interface_hash(def)` payload order.
- Fix with a mutation test that `decl_certificate_hash` for reducible defs directly includes `value_hash`.
- Add mutation tests for generated recursor signature / computation rule hash.

Completion criteria:

```text
- cargo test -p npa-cert golden_certificate_hashes_cover_core_shapes
- cargo test -p npa-cert hash role tests
- cargo test -p npa-cert inductive generated artifact tests
```

### P2-19: Add Producer Separation Tests

Implement the items from `develop/phase2.md` 12.7 directly as a test checklist.

Required tests:

```text
- when CoreModule from a Human producer and CoreModule from an AI producer represent the same core declaration,
  .npcert bytes and all hashes match
- changing producer_profile / producer_run_id / model name / score / diagnostics in the sidecar does not change
  .npcert bytes or any hashes
- Accepted from check_core_decl_candidates cannot be treated directly as VerifiedModule
- .npcert built from an Accepted candidate by build_module_cert does not enter the trusted import store until it passes verify_module_cert
- invalid prior tokens become batch-level Err(CertError), not per-candidate rejections
- if CandidateBatch.imports are not in canonical import order, the result is batch-level Err(CertError::NonCanonicalEncoding)
- duplicate ProducerImportEnvKey is Err(CertError::DuplicateImportEnvKey)
- CandidateBatchResult.statuses has the same length and same order as input candidates
- build_module_cert_from_checked_candidates rejects token chain / pre_env_fingerprint / post_env_fingerprint mismatches
- build_module_cert_from_checked_candidates rejects producer_limits_hash(token.limits) mismatches in tokens
- producer public env / prior chain fingerprints can be deterministically recomputed from canonical bytes and domain separators
- canonical_import_env_keys and canonical_import_export_views preserve the same order
- if only the proof body of an import changes and module/export_hash are the same, producer public env fingerprint is preserved
- axiom dependencies in producer public env fingerprints are recalculated by the same rules as compute_axiom_deps
- if only an opaque theorem proof / opaque def body changes while public interface and axiom dependencies are the same,
  producer public env fingerprint is preserved and prior chain fingerprint changes due to the decl_certificate_hash difference
- ProducerLimits canonical hash and stricter_or_equal judgment are deterministic
- even if preview hashes are wrong, token validation / build_module_cert / verify_module_cert use only recomputed results
- candidates from an AI producer are rejected if they contain unresolved metavariables / placeholders / pretty-only GlobalRefs
- if one candidate in a batch fails, other candidate results do not depend on failure order or cache state
- enabling/disabling cache still produces the same .npcert bytes from the same accepted module
```

Affected files:

```text
crates/npa-cert/src/tests.rs
```

### P2-20: Add Public API Compile Tests / Visibility Tests

Implementation tasks:

- Guard at compile time that external crates cannot access private fields of `CheckedDeclCandidate`.
- Confirm as API shape that `ProducerProfile` does not mix into `build_module_cert` / `verify_module_cert`.
- Test that `Accepted(CheckedDeclCandidate)` and `VerifiedModule` cannot be confused by type.

Completion criteria:

```text
- cargo check --workspace
- cargo clippy --workspace --all-targets -- -D warnings
```

---

## 9. Integration / Rollout

### P2-21: Organize Internal `npa-cert` Helpers At A Granularity Reusable From Producer

Current state:

```text
canonicalize_decl / resolver / dependency construction are confined to private
helpers in canonical.rs. The producer fast path needs to use the same rules.
```

Implementation tasks:

- Organize `canonicalize_decl`, dependency collection, axiom dependency calculation, and decl hash calculation into internal APIs reusable from the producer path.
- Do not double-implement hash / dependency / axiom dependency rules between the trusted certificate path and producer fast path.
- Keep them out of the public API as `pub(crate)`, and organize the module boundary.

Completion criteria:

```text
- build_module_cert and check_core_decl_candidates use the same hash/dependency implementation
- drift where only one side is updated is less likely
```

### P2-22: Split Connection To Frontend / API Crates Into A Separate Task

Current state:

```text
crates/npa-frontend creates CoreModule from human source and calls
build_module_cert / verify_module_cert. Phase 4/5/7 in crates/npa-api have
tactic/search candidates, but are not connected to the Phase 2 CoreDeclCandidate
fast path.
```

Implementation tasks:

- After the Phase 2 producer fast path is added to `npa-cert`, design upper-crate usage separately.
- Do not confuse existing Phase 4/5/7 MachineTacticCandidate with Phase 2 `CoreDeclCandidate`.
- When connecting APIs, preserve the boundary that ultimately builds/verifies `.npcert`.

Completion criteria:

```text
- the Phase 2 npa-cert API is complete on its own
- connection to upper crates can happen without expanding trusted payloads
```

---

## 10. Suggested Implementation Order

The following implementation order is safe.

```text
1. P2-01 / P2-02 / P2-03
   First align the hash payload contract with phase2.md.

2. P2-04 / P2-05 / P2-06
   Add producer API types, opaque tokens, and limits hash.

3. P2-08 / P2-09 / P2-10 / P2-11 / P2-12
   Implement producer public env fingerprint and prior chain fingerprint.

4. P2-13 / P2-14 / P2-15
   Implement token validation and the candidate fast path body.

5. P2-16
   Implement the helper API that creates ModuleCert from accepted tokens.

6. P2-17 / P2-18 / P2-19 / P2-20
   Solidify sidecar separation and tests.

7. P2-21 / P2-22
   Organize internal helpers and prepare upper-crate connection.
```

---

## 11. Done definition

Completion criteria for this whole TODO.

```text
- producer API from develop/phase2.md 11.1.1 is implemented in crates/npa-cert
- CheckedDeclCandidate is implemented as an opaque token
- ProducerLimits canonical hash and strictness judgment are implemented
- producer public env fingerprint and prior chain fingerprint can be recomputed from canonical bytes
- check_core_decl_candidates returns batch-level failures and per-candidate statuses as specified
- build_module_cert_from_checked_candidates revalidates the token chain before creating ModuleCert
- producer sidecar / metadata does not affect .npcert bytes or hashes
- hash payload contracts for Def / Inductive match develop/phase2.md
- producer separation tests from develop/phase2.md 12.7 pass as automated tests
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
```
