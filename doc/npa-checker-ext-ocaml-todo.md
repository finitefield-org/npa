# OCaml clean-room npa-checker-ext Todo

Source: `doc/npa-checker-ext-ocaml.md`

This document decomposes the OCaml clean-room `npa-checker-ext` specification
into implementation-ready milestones. The goal is a source-free external
checker that accepts canonical `.npcert` bytes, explicit import certificates,
and policy inputs only, then emits deterministic
`npa.independent-checker.checker_raw_result.v1` JSON.

---

## Scope

対象:

```text
- in-repository OCaml project for npa-checker-ext
- vendored OCaml SHA-256 implementation
- source-free canonical certificate decoder
- hash verifier
- import store and high-trust import policy
- type checker, conversion checker, and simple inductive/recursor checker
- axiom report recomputation and policy gates
- runner integration through existing Phase 8 request/result contracts
- release/high-trust external checker gate and benchmarks
```

非対象:

```text
- .npa source parsing or elaboration
- tactic replay
- AI trace or theorem index consumption
- package registry or network import resolution
- plugin loading
- checker identity manifest signatures in first release
- quotient feature implementation in first release
- formally verified checker implementation
```

Trusted-boundary constraints:

```text
- npa-checker-ext must not link to npa-kernel, npa-cert, npa-api,
  npa-frontend, or npa-tactic.
- npa-checker-ext may use public documents, canonical certificate fixtures,
  public JSON schema contracts, and differential results.
- ext_cli is the only OCaml module that may read the filesystem.
- checker raw result JSON must not include timestamps, absolute paths,
  locale-dependent messages, or human-readable proof evidence.
- runner-enforced timeout and resource exhaustion belong to MachineCheckResult,
  not checker_raw.error.kind.
```

Current implementation facts:

```text
- crates/npa-checker-ref already provides a source-free Rust reference checker.
- crates/npa-api already fixes external profile, runner policy, raw result
  parsing, MachineCheckResult adoption, normalization, and release bundle
  substrate.
- standalone npa-checker-ext binary is still target integration.
- M0-01 fixes the in-repository OCaml directory as
  `checkers/npa-checker-ext/`.
- M0-03 fixes vendored SHA-256 implementation and fixture layout.
```

Recommended validation baseline:

```sh
git diff --check
cargo test -p npa-checker-ref
cargo test -p npa-api independent_checker
```

Milestone verification command notation:

```text
OCAML_EXT_DIR:
  checkers/npa-checker-ext/

OCAML_EXT_TEST:
  checkers/npa-checker-ext/scripts/test.sh
```

---

## Milestone Map

```text
M0 repository and build identity
  -> M1 source-free decoder
  -> M2 hash verifier
  -> M3 import store
  -> M4 minimal type checker
  -> M5 conversion checker
  -> M6 inductive / recursor checker
  -> M7 axiom report / policy
  -> M8 runner integration
  -> M9 release gate
```

M0 through M7 are checker-binary work. M8 and M9 connect the binary to the
existing Phase 8 orchestration and package/release workflows.

---

## M0 Repository And Build Identity

- Status: Pending
- Depends on: None
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` sections 1, 3, 4, 7, 11, 12, 14, 16
  - `doc/phase8-ai.md` RunnerPolicy / CheckerBinaryRegistry / raw result sections
  - `crates/npa-api/src/independent_checker.rs`
- Likely touched areas:
  - `checkers/npa-checker-ext/`
  - repository build scripts or local checker scripts
  - documentation for selected OCaml directory
- Deliverables:
  - buildable OCaml project skeleton
  - `npa-checker-ext` executable name
  - vendored SHA-256 source files and tests
  - deterministic version / build identity command
  - CLI argument parser that rejects unsupported input shapes

### M0-01 Resolve In-Repository Directory

Implementation spec:

```text
1. Choose the exact repository-local directory for the OCaml project.
   The selected directory is `checkers/npa-checker-ext/`.
2. Record the decision in doc/npa-checker-ext-ocaml.md section 16.
3. Ensure the chosen path is outside Rust workspace crate membership unless a
   future decision explicitly adds OCaml build integration.
4. Reserve subdirectories for src, test fixtures, golden files, and scripts.
5. Do not create references to another repository.
```

Acceptance criteria:

```text
- The design doc no longer lists exact OCaml project path as an open decision.
- The task document and design doc use the same path.
- `rg -n "別 repository|exact in-repository directory" doc/npa-checker-ext-ocaml.md`
  has no stale path-decision hit.
- Section 16 reserves `src/`, `test/fixtures/`, `test/golden/`, and
  `scripts/` under `checkers/npa-checker-ext/`.
```

Verification:

```sh
git diff --check
rg -n "npa-checker-ext|OCaml project|in-repository" doc/npa-checker-ext-ocaml.md doc/npa-checker-ext-ocaml-todo.md
```

### M0-02 Create OCaml Project Skeleton

Implementation spec:

```text
1. Add the OCaml project files needed to build one executable named
   npa-checker-ext.
2. Add modules matching the design layout:
   ext_cli, ext_bytes, ext_name, ext_level, ext_term, ext_cert, ext_hash,
   ext_import, ext_axiom, ext_env, ext_reduce, ext_typecheck,
   ext_inductive, ext_result.
3. Add test project wiring that can run unit tests without invoking Rust crates.
4. Add a local build command script only if the OCaml build tool needs a stable
   wrapper.
5. Keep generated build artifacts out of git.
```

Acceptance criteria:

```text
- A fresh checkout can build npa-checker-ext with the documented OCaml command.
  The M0-02 command is `checkers/npa-checker-ext/scripts/build.sh`.
- The executable runs with --version and exits successfully.
- The executable with no arguments emits deterministic checker_raw_result
  failure JSON or a deterministic CLI error as specified by the chosen CLI
  policy.
- No OCaml project file links against crates/npa-kernel, crates/npa-cert,
  crates/npa-api, crates/npa-frontend, or crates/npa-tactic.
```

Verification:

```sh
git diff --check
rg -n "npa-kernel|npa-cert|npa-api|npa-frontend|npa-tactic" "$OCAML_EXT_DIR"
```

### M0-03 Add Vendored SHA-256 Implementation

Implementation spec:

```text
1. Implement SHA-256 in a small OCaml module owned by the external checker
   project.
2. Support exact bytes input and streaming/chunked input.
3. Return 32 raw bytes plus lowercase hex formatting through separate helpers.
4. Avoid external runtime dependencies for hashing.
5. Add standard vectors:
   empty input
   "abc"
   long standard message vector
   one million "a" bytes
   chunk boundary cases around 55, 56, 63, 64, 65, 119, 120, 127, 128 bytes
6. Add differential vectors produced by Rust sha2 for repository-specific
   domain-separated inputs.
```

Acceptance criteria:

```text
- All standard SHA-256 vectors pass.
- Rust sha2 differential fixtures pass.
- Hashing behavior is independent of locale, platform path, and newline
  conversion.
- Vendored SHA-256 source identity contributes to checker_build_hash.
```

Verification:

```sh
git diff --check
checkers/npa-checker-ext/scripts/test.sh sha256
```

### M0-04 Implement CLI Boundary And Identity

Implementation spec:

```text
1. Accept only the first-release CLI:
   --cert path
   --import-dir path
   --policy path
   --output json
   --version
2. Reject .npa paths for --cert, --policy, and import inputs.
3. Reject unknown flags, duplicate flags, missing values, and --output values
   other than json.
4. Make stdout contain only checker_raw_result JSON for check runs.
5. Make stderr diagnostic-only and never required for proof evidence.
6. Compute checker_build_hash from checker_id, checker_version,
   certificate format, core spec, vendored SHA-256 source identity, and
   selected build identity inputs.
7. Do not require checker identity manifest signatures in first release.
```

Acceptance criteria:

```text
- checker_id is exactly npa-checker-ext.
- checker_version is stable for the build.
- checker_build_hash changes when vendored SHA-256 source identity changes.
- CLI rejects source-looking paths and unsupported flags.
- Raw output schema is npa.independent-checker.checker_raw_result.v1.
```

Verification:

```sh
checkers/npa-checker-ext/scripts/test.sh cli
rg -n "checker_id.*npa-checker-ext|checker_raw_result" "$OCAML_EXT_DIR"
```

### M0-05 Pin First-Release Feature Policy

Implementation spec:

```text
1. Encode first-release supported core features as a set that excludes every
   quotient feature profile.
2. Reject quotient_v1, quotient_v2, and quotient_v3 certificate feature reports
   as unsupported_core_feature.
3. Add tests using quotient feature fixtures or minimal certificates generated
   by existing Rust fixture builders.
4. Document that adding quotient support requires fast/reference/external
   golden corpus expansion.
```

Acceptance criteria:

```text
- A quotient feature certificate fails with checker_raw.error.kind =
  unsupported_core_feature.
- Non-quotient MVP certificates are not rejected by this gate.
- The first-release feature policy is not controlled by AI sidecars or package
  metadata.
```

Verification:

```sh
checkers/npa-checker-ext/scripts/test.sh feature-policy
cargo test -p npa-checker-ref quotient_feature
```

---

## M1 Source-Free Decoder

- Status: Pending
- Depends on: M0
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` sections 5 and 6
  - `doc/phase2.md`
  - `doc/core-spec-v0.1.md`
  - canonical `.npcert` fixtures under `proofs/`
  - `crates/npa-checker-ref/src/main.rs` raw result shape as a compatibility oracle
- Likely touched areas:
  - `ext_bytes.ml`
  - `ext_name.ml`
  - `ext_level.ml`
  - `ext_term.ml`
  - `ext_cert.ml`
  - `ext_result.ml`
- Deliverables:
  - source-free binary decoder
  - canonical table validation
  - offset-preserving structured errors
  - deterministic decode-only test suite

### M1-01 Implement Byte Reader And Canonical Varint

Implementation spec:

```text
1. Build an immutable byte reader with explicit offset tracking.
2. Implement canonical unsigned varint decoding.
3. Reject unexpected EOF, overlong encodings, integer overflow, and
   non-canonical encodings.
4. Return structured decode errors with certificate section and byte offset.
5. Keep byte reader free of filesystem and JSON output concerns.
```

Acceptance criteria:

```text
- Empty input rejects as certificate_decode_error.
- Non-canonical varint rejects as noncanonical_encoding or
  certificate_decode_error according to raw result mapping.
- Error offset points to the byte that made the decoder fail.
```

Verification:

```sh
sh checkers/npa-checker-ext/scripts/test.sh decoder-bytes
```

### M1-02 Decode Header And Name Grammar

Implementation spec:

```text
1. Decode certificate format and core spec.
2. Require format NPA-CERT-0.1.
3. Require core spec NPA-Core-0.1.
4. Decode module and declaration names into structured components.
5. Reject empty names, empty components, dotted components, invalid UTF-8,
   and duplicate canonical name table entries.
```

Acceptance criteria:

```text
- Valid golden certificate header decodes without source input.
- Format mismatch rejects deterministically.
- Core spec mismatch rejects deterministically.
- Name errors do not rely on human string matching.
```

Verification:

```sh
$OCAML_EXT_TEST decoder-header
```

### M1-03 Decode Level And Term Tables

Implementation spec:

```text
1. Define OCaml algebraic data types for canonical levels:
   zero, succ, max, imax, param.
2. Define OCaml algebraic data types for core terms:
   sort, bvar, const, app, lam, pi, let.
3. Decode level and term tables from canonical binary sections.
4. Reject dangling table references.
5. Reject non-normalized level and term entries.
6. Reject unresolved universe metavariables before semantic trust.
```

Acceptance criteria:

```text
- Valid level and term tables decode into structured data.
- Unknown tags reject with deterministic section and offset.
- Dangling references reject before type checking.
- Decoded AST is not represented as raw source text.
```

Verification:

```sh
$OCAML_EXT_TEST decoder-tables
```

### M1-04 Decode Declarations, Export Block, And Axiom Report

Implementation spec:

```text
1. Decode axiom, definition, theorem, inductive, constrained declaration
   variants, and mutual inductive block variants supported by the certificate
   format.
2. Decode declaration dependency and axiom dependency vectors.
3. Decode export block entries with name, kind, interface hash, body flag,
   universe params, type, and axiom dependency data.
4. Decode axiom report per-declaration entries, module axioms, and core feature
   reports.
5. Decode stored export_hash, axiom_report_hash, and certificate_hash trailer.
```

Acceptance criteria:

```text
- Valid MVP certificates decode all top-level sections.
- Duplicate declaration names reject deterministically.
- Export entries with dangling declaration or term references reject.
- Axiom report length mismatch is preserved for later axiom-report validation
  and not silently ignored.
```

Verification:

```sh
$OCAML_EXT_TEST decoder-declarations
```

### M1-05 Validate Reachability And Canonical Ordering

Implementation spec:

```text
1. Build root sets from header, imports, declarations, export block, and
   axiom report.
2. Traverse reachable terms and levels.
3. Reject unused name, level, and term table entries.
4. Validate canonical table order and duplicate rules.
5. Reject trailing bytes after the module hash trailer.
```

Acceptance criteria:

```text
- Valid golden certificates pass reachability validation.
- Injected unused table entries reject.
- Reordered canonical tables reject.
- Trailing bytes reject.
```

Verification:

```sh
$OCAML_EXT_TEST decoder-reachability
```

---

## M2 Hash Verifier

- Status: Pending
- Depends on: M1
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` section 7
  - `crates/npa-cert/src/hash.rs`
  - `crates/npa-cert/tests/fixtures/golden_hashes.tsv`
  - valid `.npcert` fixtures
- Likely touched areas:
  - `ext_hash.ml`
  - `ext_level.ml`
  - `ext_term.ml`
  - `ext_cert.ml`
  - SHA-256 fixture files
- Deliverables:
  - domain-separated canonical hash recomputation
  - declaration, export, axiom, and certificate hash checks
  - mutation tests for every stored hash role

### M2-01 Implement Canonical Hash Encoder

Implementation spec:

```text
1. Implement canonical byte encoders for level, term, declaration dependency,
   axiom dependency, declaration payload, export block, and axiom report
   inputs.
2. Keep hash encoders independent from pretty printers and JSON output.
3. Match Rust domain separation labels exactly.
4. Add fixture tests for empty module, axiom-only module, theorem module,
   import module, and inductive module.
```

Acceptance criteria:

```text
- Encoder output drives hash equality with npa-cert golden fixtures.
- Encoder never reads source spans, debug sidecars, or filesystem paths.
- Hash tests fail when a domain label is changed.
```

Verification:

```sh
$OCAML_EXT_TEST hash-encoder
cargo test -p npa-cert
```

### M2-02 Recompute Level And Term Hashes

Implementation spec:

```text
1. Recompute hash for every normalized level table entry.
2. Recompute hash for every normalized term table entry.
3. Preserve table-index based dependency order.
4. Reject table entries whose hash-dependent references cannot be resolved.
```

Acceptance criteria:

```text
- Term hash parity with Rust fixtures is bit-for-bit.
- Level hash parity with Rust fixtures is bit-for-bit.
- Mutation of one referenced level or term changes dependent hashes.
```

Verification:

```sh
$OCAML_EXT_TEST hash-level-term
```

### M2-03 Recompute Declaration Hashes

Implementation spec:

```text
1. Recompute declaration interface hash.
2. Recompute declaration certificate hash.
3. Include dependencies and axiom dependencies according to the canonical
   certificate format.
4. Map mismatch to declaration_hash_mismatch or dependency_hash_mismatch
   consistently with Phase 8 raw result classification.
```

Acceptance criteria:

```text
- Valid certificates match all stored declaration hashes.
- Mutating declaration type, body, dependency, or axiom dependency rejects.
- Error includes section declarations and stable offset when available.
```

Verification:

```sh
$OCAML_EXT_TEST hash-declarations
```

### M2-04 Recompute Export, Axiom, And Module Certificate Hashes

Implementation spec:

```text
1. Rebuild expected export block from checked declaration interfaces.
2. Recompute export_hash with the canonical export block domain.
3. Recompute axiom_report_hash from decoded axiom report bytes in canonical
   representation.
4. Recompute module certificate hash from the exact certificate prefix before
   stored certificate_hash.
5. Map mismatch to export_hash_mismatch, axiom_report_mismatch, or
   certificate_hash_mismatch.
```

Acceptance criteria:

```text
- Valid certificates match export_hash, axiom_report_hash, and certificate_hash.
- Hash mutation corpus covers each final hash role.
- Module certificate hash recomputation uses exact input bytes, not re-encoded
  full certificate bytes.
```

Verification:

```sh
$OCAML_EXT_TEST hash-module
cargo test -p npa-checker-ref hash_verifier
```

---

## M3 Import Store

- Status: Pending
- Depends on: M2
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` section 8
  - `doc/phase8-ai.md` import lock and runner policy sections
  - `proofs/vendor/npa-std` certificate fixtures
- Likely touched areas:
  - `ext_import.ml`
  - `ext_env.ml`
  - `ext_cli.ml`
  - import fixture directories
- Deliverables:
  - explicit source-free import store
  - normal and high-trust import resolution
  - topological checked-module import harness

### M3-01 Load Explicit Import Store

Implementation spec:

```text
1. Load import certificates only from runner-provided --import-dir or future
   --imports manifest input.
2. Do not discover imports from package roots or registry metadata.
3. Decode and hash-verify import certificates before exposing public
   environments.
4. Build import entries keyed by module name, export_hash, and certificate_hash.
5. Reject duplicate import bindings.
```

Acceptance criteria:

```text
- Import directory containing expected certificates resolves normal imports.
- Missing import directory entry rejects as import_not_found.
- Duplicate module/export_hash import entries reject.
- No .npa source or replay files are read.
```

Verification:

```sh
$OCAML_EXT_TEST import-store
```

### M3-02 Implement Normal Import Resolution

Implementation spec:

```text
1. Resolve each requested import by exact module name and export_hash.
2. If the current certificate declares an import certificate_hash, require
   exact match.
3. Copy public export interfaces and module axiom dependencies into the current
   import environment.
4. Map missing import to import_not_found.
5. Map export or certificate hash mismatch to import_hash_mismatch.
```

Acceptance criteria:

```text
- Normal imports resolve by module and export_hash, not by module name alone.
- Present certificate_hash mismatch rejects in normal mode.
- Imported public environments are source-free.
```

Verification:

```sh
$OCAML_EXT_TEST import-normal
```

### M3-03 Implement High-Trust Import Policy

Implementation spec:

```text
1. Add checker policy mode for high-trust import resolution.
2. Require every import request to include certificate_hash.
3. Require high-trust import entries to come from modules checked earlier by
   npa-checker-ext in the same topological harness.
4. Reject unchecked source-free imports as high-trust imports.
5. Preserve deterministic import check order.
```

Acceptance criteria:

```text
- High-trust mode rejects missing import certificate_hash.
- High-trust mode rejects import entries not marked as checked by
  npa-checker-ext.
- A topologically ordered import closure can be checked without source.
```

Verification:

```sh
$OCAML_EXT_TEST import-high-trust
cargo test -p npa-checker-ref high_trust
```

---

## M4 Minimal Type Checker

- Status: Pending
- Depends on: M3
- Inputs:
  - `doc/core-spec-v0.1.md`
  - `doc/phase0.md`
  - `doc/npa-checker-ext-ocaml.md` section 9
  - `crates/npa-kernel/src/expr.rs` and `crates/npa-kernel/src/env.rs` as behavior oracle only
- Likely touched areas:
  - `ext_env.ml`
  - `ext_typecheck.ml`
  - `ext_term.ml`
  - type-checking fixture modules
- Deliverables:
  - source-free infer/check implementation for MVP core terms
  - declaration checks for axiom, definition, and theorem declarations
  - structured type and universe errors

### M4-01 Build Checked Environment Model

Implementation spec:

```text
1. Represent imported public exports, local checked declarations, builtin
   declarations, and generated local references in one environment structure.
2. Resolve constants by reference kind and declaration interface hash.
3. Reject unknown references.
4. Track universe parameters and reject duplicate universe parameter names.
5. Keep theorem bodies opaque for unfolding decisions.
```

Acceptance criteria:

```text
- Imported constants resolve only when name and decl_interface_hash match.
- Local references cannot point forward to unchecked declarations.
- Unknown reference rejects as type_mismatch or equivalent stable raw error
  classification.
```

Verification:

```sh
$OCAML_EXT_TEST type-env
```

### M4-02 Implement Core Infer And Check

Implementation spec:

```text
1. Implement inference for Sort, BVar, Const, App, Lam, Pi, and Let.
2. Implement checking by inference plus definitional equality.
3. Validate local de Bruijn indices against context depth.
4. Validate Pi domains and codomains are well-sorted.
5. Validate Lam against expected Pi type.
6. Validate App function type and argument type.
7. Validate Let type, value, and body.
```

Acceptance criteria:

```text
- Well-typed theorem and reducible definition fixtures pass.
- Ill-typed application rejects.
- Out-of-scope de Bruijn index rejects.
- Sort/type mismatch rejects with structured error.
```

Verification:

```sh
$OCAML_EXT_TEST type-core
cargo test -p npa-checker-ref type_check
```

### M4-03 Check Declaration Payloads

Implementation spec:

```text
1. Check axiom declaration types.
2. Check definition type and value.
3. Check theorem type and proof term.
4. Validate declaration dependencies are available and ordered.
5. Reject unresolved universe metavariables.
6. Reject bad universe arity on constants.
```

Acceptance criteria:

```text
- Axiom, definition, and theorem declaration fixtures pass.
- Wrong theorem proof type rejects.
- Bad universe arity rejects as universe_inconsistency.
- Unresolved universe metavariable rejects before checker acceptance.
```

Verification:

```sh
$OCAML_EXT_TEST type-declarations
```

---

## M5 Conversion Checker

- Status: Pending
- Depends on: M4
- Inputs:
  - `doc/core-spec-v0.1.md` conversion rules
  - `doc/npa-checker-ext-ocaml.md` section 9
  - `crates/npa-checker-ref` conversion fixtures
- Likely touched areas:
  - `ext_reduce.ml`
  - `ext_typecheck.ml`
  - substitution helpers in `ext_term.ml`
- Deliverables:
  - deterministic WHNF
  - beta/delta/iota/zeta conversion
  - fuel-bound conversion failure classification

### M5-01 Implement Substitution And Lifting

Implementation spec:

```text
1. Implement de Bruijn lifting.
2. Implement capture-avoiding substitution for bvar replacement.
3. Add round-trip tests for nested binders.
4. Add tests for Lam, Pi, Let, and App substitution boundaries.
```

Acceptance criteria:

```text
- Substitution preserves well-scoped terms.
- Nested binder cases match Rust fixture expectations.
- Invalid bvar cases are rejected before reduction can panic.
```

Verification:

```sh
$OCAML_EXT_TEST subst
```

### M5-02 Implement WHNF And Reduction Rules

Implementation spec:

```text
1. Implement beta reduction for application of lambda.
2. Implement zeta reduction for let.
3. Implement delta reduction for reducible definitions only.
4. Forbid theorem unfolding and opaque definition unfolding.
5. Implement iota reduction for supported MVP recursor rules.
6. Thread deterministic fuel through every recursive reduction.
```

Acceptance criteria:

```text
- beta, zeta, and reducible delta positive fixtures pass.
- theorem unfolding negative fixture rejects.
- opaque delta negative fixture rejects.
- Fuel exhaustion produces conversion_failure or checker_internal_error
  according to the checker policy, not timeout or resource_exhausted raw kind.
```

Verification:

```sh
$OCAML_EXT_TEST reduce
cargo test -p npa-checker-ref conversion
```

### M5-03 Implement Definitional Equality

Implementation spec:

```text
1. Compare terms structurally after WHNF.
2. Recurse through App, Lam, Pi, Let, Const, Sort, and BVar nodes.
3. Compare universe levels after normalization.
4. Keep conversion cache optional and semantics-neutral.
5. Return deterministic negative results.
```

Acceptance criteria:

```text
- Positive beta/delta/zeta equality fixtures pass.
- Negative type mismatch fixtures reject.
- Cache presence or absence does not change result.
```

Verification:

```sh
$OCAML_EXT_TEST defeq
```

---

## M6 Inductive And Recursor Checker

- Status: Pending
- Depends on: M5
- Inputs:
  - `doc/core-spec-v0.1.md` inductive sections
  - `doc/phase0.md` inductive declaration rules
  - `doc/npa-checker-ext-ocaml.md` section 9
  - Nat/List certificate fixtures
- Likely touched areas:
  - `ext_inductive.ml`
  - `ext_typecheck.ml`
  - `ext_reduce.ml`
  - inductive fixture corpus
- Deliverables:
  - conservative positivity checker
  - simple inductive and mutual block checks for MVP certificates
  - generated constructor and recursor interface checks
  - supported iota rules connected to conversion

### M6-01 Check Constructor Domains And Results

Implementation spec:

```text
1. Parse inductive parameters, indices, universe params, constructors, and
   generated declaration references.
2. Validate every constructor domain type.
3. Validate every constructor result targets the declared family.
4. Reject bad constructor result as inductive_invalid.
5. Preserve declaration order for generated constructor interfaces.
```

Acceptance criteria:

```text
- Valid Nat/List constructor fixtures pass.
- Constructor returning the wrong family rejects.
- Constructor with malformed generated interface rejects.
```

Verification:

```sh
$OCAML_EXT_TEST inductive-constructors
```

### M6-02 Implement Conservative Positivity

Implementation spec:

```text
1. Implement MVP strict positivity check for recursive occurrences.
2. Reject negative occurrences in constructor domains.
3. Keep accepted nested shapes limited to the explicitly approved fixture set.
4. Return positivity_failure for positivity violations.
```

Acceptance criteria:

```text
- Approved positive fixtures pass.
- Negative occurrence fixtures reject.
- Unsupported advanced inductive shapes reject instead of being accepted
  optimistically.
```

Verification:

```sh
$OCAML_EXT_TEST positivity
cargo test -p npa-checker-ref positivity
```

### M6-03 Check Recursor Shape And Iota Rules

Implementation spec:

```text
1. Validate generated recursor parameter binders.
2. Validate motive binder targets the inductive family.
3. Validate major premise targets the inductive family.
4. Validate minor premises match constructors.
5. Validate recursor result applies motive to major premise.
6. Register supported iota rules for conversion.
7. Reject bad recursor shapes as inductive_invalid.
```

Acceptance criteria:

```text
- Nat recursor zero and successor iota fixtures pass.
- List recursor fixtures pass if present in MVP corpus.
- Bad recursor motive, minor, or result fixtures reject.
```

Verification:

```sh
$OCAML_EXT_TEST recursor
cargo test -p npa-checker-ref iota inductive
```

---

## M7 Axiom Report And Policy

- Status: Pending
- Depends on: M6
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` section 10
  - `doc/phase8-human.md` axiom policy sections
  - `doc/phase8-ai.md` axiom report artifact sections
  - `crates/npa-checker-ref` axiom tests
- Likely touched areas:
  - `ext_axiom.ml`
  - `ext_import.ml`
  - `ext_result.ml`
  - policy parser and fixtures
- Deliverables:
  - source-free axiom report recomputation
  - policy parser for first-release axiom policy
  - deny_sorry and custom axiom gates
  - exact Std.Logic.Eq.rec exception

### M7-01 Recompute Direct And Transitive Axioms

Implementation spec:

```text
1. Compute direct axiom dependencies for each declaration.
2. Compute transitive dependencies through declaration dependencies.
3. Include imported public environment axiom dependencies.
4. Recompute module-level axiom union.
5. Compare recomputed report with certificate axiom report.
6. Map mismatch to axiom_report_mismatch.
```

Acceptance criteria:

```text
- Valid axiom report fixtures pass.
- Missing actual dependency rejects.
- Import axiom dependencies are not dropped.
- Axiom report hash remains consistent with recomputed report.
```

Verification:

```sh
$OCAML_EXT_TEST axiom-report
cargo test -p npa-checker-ref axiom_report
```

### M7-02 Implement Axiom Policy Parser

Implementation spec:

```text
1. Parse first-release policy input with deny_sorry, deny_custom_axioms, and
   allowed_axioms.
2. Accept deterministic TOML or JSON only if explicitly documented by the CLI
   contract.
3. Reject malformed policy before checker acceptance.
4. Keep policy parse errors in runner/input error handling when invoked through
   runner; do not report them as proof evidence.
```

Acceptance criteria:

```text
- Default policy denies synthetic sorry.
- Policy allowlist is exact by canonical axiom name.
- Malformed policy has deterministic diagnostic behavior.
```

Verification:

```sh
$OCAML_EXT_TEST axiom-policy-parse
```

### M7-03 Enforce Sorry, Custom Axiom, And Eq.rec Gates

Implementation spec:

```text
1. Reject synthetic sorry when deny_sorry is true.
2. Reject custom axioms not in allowed_axioms when deny_custom_axioms is true.
3. Allow only exact Std.Logic.Eq.rec standard exception by canonical name and
   declaration interface hash.
4. Do not use axiom descriptions, source spans, or debug metadata.
```

Acceptance criteria:

```text
- Synthetic sorry fixture rejects as forbidden_axiom.
- Non-Eq.rec custom axiom fixture rejects.
- Exact Std.Logic.Eq.rec fixture passes when allowed by policy.
- Axiom policy result matches npa-checker-ref for MVP fixtures.
```

Verification:

```sh
$OCAML_EXT_TEST axiom-policy
cargo test -p npa-checker-ref axiom_policy
```

---

## M8 Runner Integration

- Status: Pending
- Depends on: M7
- Inputs:
  - `doc/npa-checker-ext-ocaml.md` sections 4, 5, 11, 15
  - `doc/community-library-roadmap-clr-08-todo.md`
  - `doc/phase8-ai.md` raw result, MachineCheckResult, registry, and
    normalization sections
  - `crates/npa-api/src/independent_checker.rs`
- Likely touched areas:
  - `crates/npa-api/src/independent_checker.rs`
  - runner fixtures
  - checker binary registry fixtures
  - release bundle fixtures
- Deliverables:
  - runner-owned execution of npa-checker-ext
  - CheckerBinaryRegistry resolution for external profile
  - raw result adoption into MachineCheckResult
  - normalized comparison with fast-kernel/reference/external
  - missing checker failure behavior

### M8-01 Add External Binary Registry Fixture

Implementation spec:

```text
1. Add or update checker registry fixtures for profile external,
   checker_id npa-checker-ext, and platform-specific binary_id values.
2. Require binary hash pinning.
3. Require checker identity manifest hash pinning.
4. Do not require manifest signature in first release.
5. Reject AI-requested checker binary overrides.
```

Acceptance criteria:

```text
- Registry fixture resolves npa-checker-ext only through runner-owned policy.
- Missing binary hash or manifest hash rejects.
- Signature absence does not fail first-release policy.
- AI sidecar cannot select executable.
```

Verification:

```sh
cargo test -p npa-api independent_checker::tests::p8h00_pr_mode_requires_reference_and_keeps_external_on_demand_only
rg -n "npa-checker-ext|checker_identity|binary_hash" crates/npa-api/src/independent_checker.rs doc/phase8-ai.md
```

### M8-02 Execute Checker In Closed Environment

Implementation spec:

```text
1. Materialize command:
   npa-checker-ext --cert cert --import-dir imports --policy policy
   --output json
2. Set deterministic environment values:
   LC_ALL=C.UTF-8
   LANG=C.UTF-8
   TZ=UTC
3. Enforce timeout and memory limits in the runner.
4. Treat timeout and resource exhaustion as MachineCheckResult errors, not
   checker raw result semantic errors.
5. Mount certificate and import inputs read-only in integration environments.
6. Do not mount source directories unless the runner explicitly proves they are
   not passed to the checker process.
```

Acceptance criteria:

```text
- Runner launches npa-checker-ext with only allowed dynamic flags.
- Unknown runner policy flags reject before process launch.
- Timeout and resource exhaustion produce runner-owned MachineCheckResult
  failures.
```

Verification:

```sh
cargo test -p npa-api independent_checker
```

### M8-03 Adopt Raw Result Into MachineCheckResult

Implementation spec:

```text
1. Parse checker_raw_result.v1 stdout.
2. Copy checker_id, checker_version, checker_build_hash only when present.
3. Copy status, module, certificate_hash, export_hash, axiom_report_hash, and
   error fields according to existing raw result parser rules.
4. Reject malformed raw JSON as checker_internal_error in MachineCheckResult.
5. Record stderr presence only as diagnostic metadata.
6. Exclude process and resource metadata from semantic result_hash according to
   existing npa-api rules.
```

Acceptance criteria:

```text
- Checked raw result becomes checked MachineCheckResult.
- Failed raw result becomes failed MachineCheckResult.
- raw policy_failure, timeout, and resource_exhausted error kinds are rejected
  as raw schema errors.
- MachineCheckResult self-hashes validate.
```

Verification:

```sh
cargo test -p npa-api independent_checker::tests::m3_adopts_checked_raw_result_and_computes_distinct_hashes
cargo test -p npa-api independent_checker::tests::m3_malformed_raw_output_is_saved_as_checker_internal_error
```

### M8-04 Normalize And Compare External Results

Implementation spec:

```text
1. Include external profile results in NormalizedCheckResult comparison.
2. Require release mode profiles fast-kernel, reference, external.
3. Require high-trust mode profiles fast-kernel, reference, external,
   high-trust-reference.
4. Treat checked/failed disagreement as release blocker.
5. Treat export_hash, certificate_hash, or axiom_report_hash mismatch as
   release blocker.
6. Treat timeout, resource_exhausted, or checker_internal_error as
   inconclusive/failing for release gates.
```

Acceptance criteria:

```text
- Reference checked and external failed produces disagreement.
- Matching checked fast/reference/external results pass comparison.
- External hash mismatch fails comparison.
- PR mode keeps external optional/on-demand only.
```

Verification:

```sh
cargo test -p npa-api independent_checker::tests::independent_checker_challenge_p8h13_differential_disagreements_fail_ci
cargo test -p npa-api independent_checker::tests::p8h14_performance_gates_keep_reference_external_off_pr_hot_path
```

---

## M9 Release Gate

- Status: Pending
- Depends on: M8
- Inputs:
  - `doc/community-library-roadmap-clr-08-todo.md`
  - `doc/community-library-roadmap.md`
  - `README.md` package CLI section
  - package CLI and package manifest modules when available
- Likely touched areas:
  - package CLI integration
  - package verifier integration
  - release/high-trust CI templates
  - benchmark/audit output fixtures
  - docs and README package command examples
- Deliverables:
  - `npa package verify-certs --checker external`
  - release/high-trust comparison gate
  - external checker benchmark summary
  - verified_high_trust generation guard
  - CI/release template integration when package CLI exists

### M9-01 Add Package CLI External Checker Mode

Implementation spec:

```text
1. Enable --checker external only in the CLR-08 package command scope.
2. Require runner policy path and hash.
3. Require checker binary registry path.
4. Validate package manifest and package lock before checker execution.
5. Materialize source-free MachineCheckRequest values from package lock entries.
6. Run npa-checker-ext on local certificates and import certificates only.
7. Save deterministic MachineCheckResult JSON diagnostics.
8. Keep --changed, --all, --registry, --network, and --latest outside the base
   external checker command unless separately specified.
```

Acceptance criteria:

```text
- `npa package verify-certs --checker external` works only with explicit
  runner policy and checker registry.
- The command rejects missing external checker binary with structured
  diagnostic.
- The command does not read `.npa` source in verify-certs external mode.
```

Verification:

```sh
cargo test -p npa-cli package_verify_external
rg -n -- "--checker external|runner-policy|checker-registry|--changed|--all|--network|--latest" crates doc README.md
```

### M9-02 Enforce Release And High-Trust Pass Conditions

Implementation spec:

```text
1. For release mode, require fast-kernel, reference, and external results.
2. For high-trust mode, require fast-kernel, reference, external, and
   high-trust-reference results.
3. Require all required profiles to resolve to the same module,
   certificate_hash, export_hash, and axiom_report_hash when checked.
4. Fail release/high-trust when required external result is missing, failed,
   inconclusive, timeout, resource_exhausted, or checker_internal_error.
5. Prevent reference-only evidence from producing verified_high_trust.
```

Acceptance criteria:

```text
- Missing external checker blocks release/high-trust external gate.
- Hash disagreement blocks release/high-trust.
- Reference-only release evidence is clearly marked as not verified_high_trust.
```

Verification:

```sh
cargo test -p npa-api independent_checker::tests::p8h14_release_and_high_trust_pass_requirements_are_closed
cargo test -p npa-api independent_checker::tests::m12_release_bundle_generates_manifest_and_validation_auxiliary_passes
```

### M9-03 Add Benchmark And Audit Collection

Implementation spec:

```text
1. Record checker identity, checker profile, module, certificate_hash,
   result_hash, elapsed time, timeout budget, memory budget, and status.
2. Keep benchmark output out of proof evidence.
3. Exclude benchmark metadata from checker raw semantic identity.
4. Include external checker benchmark in release/high-trust audit collection.
5. Keep PR hot path free from required external benchmark completion.
```

Acceptance criteria:

```text
- Benchmark summaries are deterministic JSON.
- Benchmark summaries do not affect proof validity.
- PR mode does not require external checker benchmark completion.
```

Verification:

```sh
rg -n "external checker benchmark|PR hot path|proof validity" doc README.md crates
```

### M9-04 Update Release Documentation And Completion Gate

Implementation spec:

```text
1. Update README package command examples when external checker command is
   implemented.
2. Update Phase 8 docs to say npa-checker-ext exists only after the binary is
   present and runner integration passes.
3. Update CLR-08 completion checklist.
4. Add or update release gate script only when the external checker binary can
   run in fresh checkout or documented CI environment.
5. Keep source-free trust boundary explicit in every command example.
```

Acceptance criteria:

```text
- Docs do not imply npa-checker-ext exists before implementation.
- Docs do not imply verified_high_trust can be generated from reference-only
  evidence.
- Docs do not imply PR mode requires external checker.
- Release command examples match actual implemented commands.
```

Verification:

```sh
git diff --check
rg -n "npa-checker-ext|verified_high_trust|reference-checker-only|--checker external" README.md doc
```

---

## Review Ledger

Review pass 1 findings:

```text
F1: The original source design left the exact OCaml project directory and
vendored SHA-256 file layout open. The task breakdown must not pretend those
are already decided before their milestone tasks.
Resolution: M0-01 now fixes `checkers/npa-checker-ext/`; M0-03 now fixes
`src/ext_sha256.ml`, `src/ext_hash.ml`, and `test/golden/sha256_vectors.tsv`.

F2: The source design makes checker identity manifest signatures non-required
for first release. Runner integration tasks must not require signatures.
Resolution: M8-01 requires manifest and binary hash pinning, and explicitly
keeps manifest signatures non-required for first release.

F3: The source design rejects quotient features in first release. Type checker
and release tasks must not require quotient acceptance.
Resolution: M0-05 and M4/M5/M6 scope keep quotient support outside first
release, and differential tests require unsupported_core_feature rejection.

F4: Runner-owned timeout/resource failures could be confused with checker raw
result errors.
Resolution: M8-02 and M8-03 explicitly keep timeout/resource_exhausted in
MachineCheckResult, not checker_raw.error.kind.
```

Review pass 2 findings:

```text
No confirmed findings remain.
```

---

## Validation

Run after editing this task document:

```sh
git diff --check
rg -n "TO[D]O|TB[D]|未[定]|PLACEHOLDE[R]" doc/npa-checker-ext-ocaml-todo.md
perl -ne '$lt = chr 60; $gt = chr 62; print "$.:$_" if /$lt[A-Za-z0-9_ -]+$gt/' doc/npa-checker-ext-ocaml-todo.md
rg -n "reference checke[r] を独立バイナリ|npa-checker-ext --audit-bundl[e]|pinned OCaml SHA-256 librar[y]" doc/npa-checker-ext-ocaml.md doc/npa-checker-ext-ocaml-todo.md doc/phase8-human.md
rg -n "M[0]-|M[1]-|M[2]-|M[3]-|M[4]-|M[5]-|M[6]-|M[7]-|M[8]-|M[9]-" doc/npa-checker-ext-ocaml-todo.md
```
