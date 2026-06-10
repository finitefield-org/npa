# Phase 6 Human Task Breakdown

This task breakdown treats `develop/phase6-human.md` as authoritative and
divides the gap from the current `crates/npa-frontend` / `crates/npa-tactic` /
`crates/npa-api` / `crates/npa-cert` implementation into milestones for the
standard-library source implementation.

Phase 6 Human is an untrusted layer for building a small, robust standard
library that humans can read and write, then generating canonical certificates
and search metadata from that source. The AI-facing wire contract for the
release manifest, import bundles, Machine theorem index, and simp / rewrite
profiles belongs to `develop/phase6-ai.md`; Human source layout, notation,
pretty statements, and Human-facing attribute tables must not be mixed into the
AI hot path.

Important constraints:

```text
- Do not put standard-library source text, notation, or Human-facing
  attributes into the trusted base.
- Proof acceptance must rest only on canonical certificates and kernel /
  verifier / independent-checker results.
- The Phase 6 release modules are exactly Std.Logic / Std.Nat / Std.List /
  Std.Algebra.Basic.
- Core / prelude must not appear in emitted certificates as Phase 2
  ImportEntry values.
- Std.Algebra.Basic must not depend on Std.Nat. Nat algebra instances belong
  in a future separate module or test fixture, not a release module.
- Std.Nat.Algebra, Std.Classical, typeclasses, full simp, ring / omega /
  linarith, and overloaded numerals are not in the MVP.
- Emit the exact Std.Logic.Eq.rec exception in the axiom report / allowlist
  only when Eq.rec is represented as a kernel-standard AxiomDecl.
- Do not add I/O, network, plugin loading, AI calls, or a standard-library
  package resolver to the kernel crate.
```

---

## 0. Current Implementation Boundary

### 0.1 What Counts As Implemented

The current `crates/npa-frontend` has the Human Surface source parser /
resolver / elaborator. The Phase 6 Human implementation may use these for
source input.

```text
crates/npa-frontend/src/human.rs
- HumanModule / HumanItem / HumanDecl / HumanDeclValue
- HumanProofBlock / HumanTacticScript / HumanTacticSyntax
- HumanSourceInterface / HumanImportedSourceInterface
- HumanDiagnostic / HumanDiagnosticPayload / HumanHoleGoal

crates/npa-frontend/src/human_parser.rs
- parse_human_module / parse_human_term
- import / open / namespace / notation / def / theorem / axiom / inductive
- parser for `by` proof blocks and intro / exact / apply / rw / simp-lite /
  induction

crates/npa-frontend/src/human_resolver.rs
- namespace / open scope resolution
- imported Human metadata lookup
- ambiguity and forward-reference diagnostics

crates/npa-frontend/src/human_elaborator.rs
- compile_human_source_to_core / compile_human_source_to_certificate
- Human term elaboration with implicit insertion / simple metas / holes
- certificate handoff that rejects unresolved holes
```

The current `crates/npa-tactic` has proof-state primitives used by
standard-library proofs.

```text
crates/npa-tactic/src/lib.rs
- MachineProofState / MachineGoal / MetaVarStore
- intro / exact / apply / rw / simp-lite / induction-nat
- run_machine_tactic_with_budget / run_machine_tactic_candidates_batch
- extract_closed_machine_proof / extract_closed_machine_certificate
- deterministic budget hash / tactic cache key / proof delta
```

The current `crates/npa-api` has the machine artifact implementation for the
Phase 6 AI Profile. It consumes the output of the Human source implementation;
it is not the source of truth for Human source layout.

```text
crates/npa-api/src/std_library.rs
- load_machine_std_mvp_certificates
- load_machine_std_mvp_release
- generate_machine_std_mvp_import_bundle_set
- generate_machine_std_mvp_theorem_index
- generate_machine_std_mvp_rewrite_profile_set
- generate_machine_std_mvp_simp_profile_set
- generate_machine_std_mvp_final_theorem_index
- validate_machine_std_mvp_* validators
```

### 0.2 Phase 6 Human Scope Not Yet Implemented

The following scope required by `develop/phase6-human.md` does not currently
exist in the code as a standard-library source package.

```text
Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic source package
Std.Logic certificate with Eq / connectives / Exists
Std.Nat certificate with Nat / add / mul / pred and basic theorems
Std.List certificate with List / append / length / map / foldr and basic theorems
Std.Algebra.Basic certificate with explicit algebraic property definitions
source package build pipeline from Human source to raw .npcert bytes
per-module Human debug views: index / axiom report / dependency graph
Phase 4 tactic regression over real standard-library certificates
handoff from real standard-library certificates to Phase 6 AI machine release loader
```

Existing frontend / API tests still contain old fixture module names such as
`Std.Nat.Basic` and `Std.Logic.Eq`. Treat these as test-only compatibility
fixtures; do not publish them as Phase 6 MVP release module names.

### 0.3 Items That Must Not Enter The AI Fast Path

The following may be used in the Phase 6 Human source implementation, but must
not enter the AI path for high-volume candidate generation, search, or tactic
execution.

```text
Human source package file layout
Human parser / resolver / notation table
open / namespace / overloaded display conveniences
Human-facing attributes such as intro / elim / refl / trans / congr
pretty theorem statements
per-module debug JSON files
source spans and source diagnostics
Human theorem search ranking
prompt text / natural-language explanations
filesystem package discovery during /machine/* request execution
```

The AI path keeps the following shape.

```text
Std.machine-* release artifacts
  -> Phase 5 Machine session import bundle
  -> MachineProofSnapshot with Machine Surface state
  -> MachineTacticCandidate
  -> /machine/tactics/run or /machine/tactics/batch
  -> /machine/replay
  -> /machine/verify
```

---

## 1. Design Rules For Preserving The AI Fast Path

Each Phase 6 Human milestone treats the following as acceptance criteria.

```text
- Do not change request / response schemas for `/machine/*` endpoints.
- Do not add Human source paths, pretty theorem text, or Human attribute
  metadata to `MachineTacticCandidate`.
- Do not add inputs to Machine `state_fingerprint`, `candidate_hash`,
  `theorem_index_fingerprint`, or `std_library_release_hash`.
- Run Human source builds as release preprocessing; do not put source parsing
  or package discovery into AI request runtime.
- Read AI theorem index / rw / simp metadata from the validated sidecars of the
  Phase 6 AI Profile.
- Human per-module debug JSON is for explanation / review / local cache use;
  it is not a required AI runtime input.
- Do not emit Human-facing `intro` / `elim` / `refl` / `trans` / `congr`
  attributes into the AI MVP theorem index.
- Do not include theorem search ranking, prompt metadata, embeddings, or usage
  statistics in certificate hashes / release hashes.
- Do not make anything other than the kernel / certificate verifier /
  independent checker a proof acceptance boundary.
```

---

## 2. Implementation Order

Phase 6 Human fixes the source package and certificate boundary first, then
expands the contents of each module. On the Machine artifact side, reuse the
existing Phase 6 AI implementation and move toward validation with real
certificates.

```text
1. Fix the Human / AI standard-library boundary and source package skeleton.
2. Implement the Std.Logic Eq family.
3. Implement Std.Logic connectives.
4. Implement Std.Nat basic definitions.
5. Implement Std.Nat add theorems.
6. Implement Std.Nat mul / pred theorems.
7. Implement Std.List basic / append.
8. Implement Std.List length / map / foldr.
9. Implement Std.Algebra.Basic.
10. Generate .npcert / hash / axiom report from the source package.
11. Generate Human theorem index / debug views.
12. Compare simp-lite / rw / search metadata with the Phase 6 AI profile.
13. Fix Phase 4 tactic regressions on the real stdlib.
14. Pass real stdlib artifacts to the Phase 6 AI release loader.
15. Fix docs / regression gate.
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-frontend --lib human
cargo test -p npa-frontend --lib human_parser
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-tactic
cargo test -p npa-api std_library
cargo test -p npa-api search
cargo test -p npa-api phase7
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. Task List

### P6H-00: Fix The Human / AI Standard-Library Boundary

Implementation tasks:

- [x] Reflect the boundary from `develop/phase6-human.md`,
  `develop/phase6-ai.md`, and README in implementation comments or module
  docs.
- [x] Fix the standard-library source package root. Public release module names
  are only `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic`.
- [x] Materialize the Phase 6 design's `Std/...` layout as the source package
  root, and fix source path to `.npcert` artifact path mapping in docs / tests.
- [x] Add a regression proving machine release identity is not inferred from
  Human source layout.
- [x] Add a regression proving the AI fast path does not read the Human source
  parser / per-module debug JSON / Human attribute table.
- [x] State in a test comment that legacy fixture module names `Std.Nat.Basic`
  / `Std.Logic.Eq` must not be confused with Phase 6 release modules.

Acceptance criteria:

- [x] Phase 6 release module membership is fixed in tests as exactly four
  modules.
- [x] The boundary that rejects `Core` / prelude if emitted as a Phase 2
  ImportEntry is preserved.
- [x] A regression passes showing Machine API candidate identity / state
  fingerprint does not change before and after Human integration.

Verification:

```sh
cargo test -p npa-api phase7_machine_api_identity_is_stable_around_phase5_human_integration_fixture
cargo test -p npa-api std_library
cargo test -p npa-frontend --lib human
```

Dependencies:

```text
None
```

Notes:

```text
This milestone does not implement source package contents. It only fixes the
boundary and package skeleton.
```

### P6H-01: Implement The Std.Logic Eq Family

Implementation tasks:

- [x] Define the `Eq` inductive and generated public exports in `Std.Logic`
  source.
- [x] Treat `Eq.refl` as a generated constructor export and do not separately
  export a theorem with the same name.
- [x] Prove `Eq.symm`, `Eq.trans`, `Eq.subst`, and `Eq.congrArg`.
- [x] Reflect the exact exception in the axiom report when `Eq.rec` is emitted
  as a kernel-standard AxiomDecl.
- [x] Check that the `Std.Logic` certificate has no ordinary `Core`
  ImportEntry.

Acceptance criteria:

- [x] `Std.Logic.npcert` can be rechecked by the verifier / kernel without
  source.
- [x] `Eq.refl` is treated as a generated family head, not a theorem index
  entry.
- [x] Custom axioms are rejected, and the only allowed axiom is the exact
  `Std.Logic.Eq.rec` exception.

Verification:

```sh
cargo test -p npa-cert
cargo test -p npa-api std_library
cargo test -p npa-tactic rw
```

Dependencies:

```text
P6H-00
```

### P6H-02: Implement Std.Logic Connectives

Implementation tasks:

- [x] Add `True` / `False` / `Not` / `And` / `Or` / `Iff` / `Exists` to
  `Std.Logic` source.
- [x] Limit `False.elim` to `P : Prop` in the MVP and do not add large
  elimination.
- [x] Prove `And.left` / `And.right` / `And.intro`, `Or.elim` / `Or.inl` /
  `Or.inr`, basic `Iff` theorems, and `Exists.intro` / `Exists.elim`.
- [x] Keep Human-facing `intro` / `elim` / `trans` / `congr` attributes as
  source metadata and do not pass them into AI theorem index attributes.

Acceptance criteria:

- [x] `Std.Logic` remains constructive and does not include
  `Classical.choice` / `funext` / `propext`.
- [x] Apply search produces `Eq.trans`, `And.intro`, and `False.elim` as
  candidates.
- [x] The AI MVP theorem index does not emit `Intro` / `Elim` / `Refl` /
  `Trans` / `Congr` attributes.

Verification:

```sh
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-api search
cargo test -p npa-api std_library
```

Dependencies:

```text
P6H-01
```

### P6H-03: Implement Std.Nat Basic Definitions

Implementation tasks:

- [x] Create the `Std.Nat` source module and make its only direct import
  `Std.Logic`.
- [x] Publicly export the `Nat` inductive, `Nat.zero`, `Nat.succ`, and the
  generated recursor.
- [x] Define `Nat.one` and `Nat.pred`.
- [x] Prove `Nat.pred_zero` and `Nat.pred_succ` by refl.
- [x] Even if Nat-specific numeral display / parsing is added, do not make
  numerals overloaded.

Acceptance criteria:

- [x] The import closure of the `Std.Nat` certificate consists only of
  `Std.Logic` and `Std.Nat`.
- [x] Generated `Nat` exports can be resolved as Phase 6 AI `NatFamilyRef`.
- [x] `Nat.pred_zero` / `Nat.pred_succ` can be registered as simp-safe
  candidates.

Verification:

```sh
cargo test -p npa-frontend --lib human
cargo test -p npa-api std_library
cargo test -p npa-tactic induction
```

Dependencies:

```text
P6H-02
```

### P6H-04: Implement Std.Nat Add Theorems

Implementation tasks:

- [x] Define `Nat.add` by recursion on the second argument.
- [x] Prove `Nat.add_zero` and `Nat.add_succ` by definitional equality / refl.
- [x] Prove `Nat.zero_add`, `Nat.succ_add`, `Nat.add_assoc`, and
  `Nat.add_comm` by induction and Phase 4 tactics.
- [x] Do not put `Nat.add_comm` and `Nat.add_assoc` in simp; treat them as
  exact `RwOnly` entries in the AI rewrite profile.

Acceptance criteria:

- [x] Definitional equality tests for `Nat.add n Nat.zero` and
  `Nat.add n (Nat.succ m)` pass.
- [x] `Nat.add_zero` / `Nat.add_succ` / `Nat.zero_add` work as simp-safe
  rules.
- [x] `Nat.add_comm` / `Nat.add_assoc` are rw candidates but not simp
  candidates.

Verification:

```sh
cargo test -p npa-tactic rw
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

Dependencies:

```text
P6H-03
```

### P6H-05: Implement The Std.Nat Mul / Pred Theorem Set

Implementation tasks:

- [x] Define `Nat.mul` by recursion on the second argument.
- [x] Prove `Nat.mul_zero` and `Nat.mul_succ` by refl.
- [x] Prove `Nat.zero_mul` and `Nat.succ_mul` by induction / simp-lite.
- [x] Prove `Nat.mul_assoc`, `Nat.mul_comm`, `Nat.left_distrib`, and
  `Nat.right_distrib` as late-MVP theorems, but do not add them to the fixed
  simp-safe / AI `RwOnly` sets.
- [x] Confirm that `Nat.pred_zero` / `Nat.pred_succ` remain in the simp-safe
  set from P6H-03.

Acceptance criteria:

- [x] Definitional equality tests for `Nat.mul n Nat.zero` and
  `Nat.mul n (Nat.succ m)` pass.
- [x] `Nat.mul_zero` / `Nat.mul_succ` / `Nat.zero_mul` work as simp-safe
  rules.
- [x] `Nat.mul_assoc` / `Nat.mul_comm` / distribution theorems are found by
  theorem search but do not enter simp / AI MVP rewrite profiles.

Verification:

```sh
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

Dependencies:

```text
P6H-04
```

### P6H-06: Implement Std.List Basic / Append

Implementation tasks:

- [x] Create the `Std.List` source module and make its direct imports
  `Std.Logic` and `Std.Nat`.
- [x] Publicly export the `List` inductive, `List.nil`, `List.cons`, and the
  generated recursor.
- [x] Define `List.append` by recursion on the first argument.
- [x] Prove `List.nil_append` and `List.cons_append` by refl.
- [x] Prove `List.append_nil` and `List.append_assoc` by induction /
  simp-lite.
- [x] Do not put `List.append_assoc` in simp; treat it as exact `RwOnly` in
  the AI rewrite profile.

Acceptance criteria:

- [x] Definitional equality tests for `[] ++ ys` and `(x :: xs) ++ ys` pass.
- [x] `List.nil_append` / `List.cons_append` / `List.append_nil` work as
  simp-safe rules.
- [x] `std.list.simp` does not include `Std.Nat` rewrite rule sources.

Verification:

```sh
cargo test -p npa-tactic induction
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

Dependencies:

```text
P6H-05
```

### P6H-07: Implement Std.List Length / Map / Foldr

Implementation tasks:

- [x] Add `List.length`, `List.map`, and `List.foldr` to source.
- [x] Prove `List.length_nil` / `List.length_cons` / `List.map_nil` /
  `List.map_cons` / `List.foldr_nil` / `List.foldr_cons` by refl.
- [x] Prove `List.length_append`, `List.map_id`, and `List.map_comp` by
  induction / simp-lite.
- [x] Include `List.length_append` in AI `RwOnly`; do not include
  `List.map_comp` in the AI MVP rewrite profile.
- [x] Do not include `foldl` or list literal `[a, b, c]` in the MVP.

Acceptance criteria:

- [x] The exact List simp-safe set matches the Phase 6 Human / AI Profile.
- [x] `List.length_append` is an rw candidate but not a simp candidate.
- [x] Even though `List.map_comp` exists as a theorem, it does not appear in
  AI MVP rewrite / simp profiles.

Verification:

```sh
cargo test -p npa-tactic simp
cargo test -p npa-api std_library
cargo test -p npa-api search
```

Dependencies:

```text
P6H-06
```

### P6H-08: Implement Std.Algebra.Basic

Implementation tasks:

- [x] Create the `Std.Algebra.Basic` source module and make its only direct
  import `Std.Logic`.
- [x] Define `Associative` / `Commutative` / `LeftIdentity` /
  `RightIdentity` as unbundled properties.
- [x] Define `IsSemigroup` / `IsMonoid` / `IsCommMonoid` as explicit Prop
  inductives.
- [x] Prove `IsMonoid.assoc` / `IsMonoid.left_id` / `IsMonoid.right_id` and
  `IsCommMonoid` projection theorems.
- [x] Prove `identity_unique`.
- [x] Do not include `Nat.add_is_comm_monoid` in the `Std.Algebra.Basic` or
  `Std.Nat` MVP release modules.

Acceptance criteria:

- [x] The import closure of the `Std.Algebra.Basic` certificate consists only
  of `Std.Logic` and `Std.Algebra.Basic`.
- [x] Typeclass resolution, bundled carriers, and implicit instance search are
  not introduced.
- [x] Algebra projection theorems become candidates in apply search.

Verification:

```sh
cargo test -p npa-frontend --lib human_elaborator
cargo test -p npa-api std_library
cargo test -p npa-api search
```

Dependencies:

```text
P6H-02
```

### P6H-09: Generate Certificate Artifacts From The Source Package

Implementation tasks:

- [x] Add a build entrypoint that compiles the standard-library source package
  in deterministic order.
- [x] Generate raw `.npcert` bytes, export_hash, certificate_hash, and
  module-level axiom_report_hash for each module.
- [x] Make export_hash mismatches in import entries build failures; in
  high-trust mode, make certificate_hash mismatches failures as well.
- [x] Make `ExportEntry.name` the declaration name itself, not a synthetic name
  concatenated with the module name.
- [x] Reject source artifacts that emit `Core` / prelude as ordinary
  ImportEntry values.

Acceptance criteria:

- [x] Artifacts corresponding to `Std/Logic.npcert`, `Std/Nat.npcert`,
  `Std/List.npcert`, and `Std/Algebra/Basic.npcert` are generated as raw Phase
  2 certificate bytes.
- [x] All modules can be rechecked by the certificate verifier without source.
- [x] The axiom report is empty or contains only the exact `Eq.rec` exception.

Verification:

```sh
cargo test -p npa-cert
cargo test -p npa-api std_library
cargo test --workspace
```

Dependencies:

```text
P6H-01
P6H-02
P6H-03
P6H-04
P6H-05
P6H-06
P6H-07
P6H-08
```

### P6H-10: Generate Human Theorem Index / Debug Views

Implementation tasks:

- [x] Generate the Human-facing theorem search view from certificate verifier
  output.
- [x] Generate per-module `index` / `axioms` / minimal dependency graph debug
  views.
- [x] Keep `proof_term_size` null in AI MVP artifacts and do not confuse it
  with optional Human debug-view information.
- [x] Even when producing `simp` / `rw` / `apply` / `intro` / `elim` displays
  from Human source attributes, bind them to certificate-derived identity.
- [x] Limit suggested tactic strings in Human theorem search to the Human UI.

Acceptance criteria:

- [x] Human search shows `Nat.add_zero`, `List.append_nil`, and `Eq.trans` in
  the expected categories.
- [x] Debug views do not use source text or pretty statements as trusted hash
  inputs.
- [x] Tests / docs separate the responsibilities of the AI
  `MachineStdTheoremIndex` schema and the Human debug schema.

Verification:

```sh
cargo test -p npa-api human_theorem_index
cargo test -p npa-api search
cargo test -p npa-api std_library
```

Dependencies:

```text
P6H-09
```

### P6H-11: Compare Simp-Lite / Rw / Search Metadata With The AI Profile

Implementation tasks:

- [x] Add comparison tests from Human source `simp` / `rw` intent to the exact
  Phase 6 AI MVP SimpSafe / RwOnly fixed sets.
- [x] Fix the Nat SimpSafe set to `Nat.add_zero` / `Nat.add_succ` /
  `Nat.zero_add` / `Nat.mul_zero` / `Nat.mul_succ` / `Nat.zero_mul` /
  `Nat.pred_zero` / `Nat.pred_succ`.
- [x] Fix the List SimpSafe set to `List.nil_append` / `List.cons_append` /
  `List.append_nil` / `List.length_nil` / `List.length_cons` /
  `List.map_nil` / `List.map_cons` / `List.map_id` / `List.foldr_nil` /
  `List.foldr_cons`.
- [x] Fix the RwOnly set to `Nat.add_comm` / `Nat.add_assoc` /
  `List.append_assoc` / `List.length_append`.
- [x] Add regressions proving `Nat.mul_comm` / `Nat.mul_assoc` /
  `List.map_comp` do not appear in the AI MVP rewrite profile.

Acceptance criteria:

- [x] `std.logic.simp` is empty, and `Eq.refl` is not emitted as a
  SimpRuleRef.
- [x] `std.list.simp` does not include `Std.Nat` rule sources.
- [x] `std.all.simp` and `std.all.rw` are revalidated as semantic unions of
  source profiles.

Verification:

```sh
cargo test -p npa-api std_library
cargo test -p npa-tactic simp
cargo test -p npa-tactic rw
```

Dependencies:

```text
P6H-10
```

### P6H-12: Fix Phase 4 Tactic Regressions On The Real Stdlib

Implementation tasks:

- [x] Add regressions for `intro` / `exact` / `apply` / `rw` / `simp-lite` /
  `induction` on real standard-library certificates.
- [x] Add tests that reprove `Nat.zero_add`, `List.append_nil`, and `Eq.trans`
  using only Phase 4 tactics.
- [x] Add negative tests for failed rewrites, loop-prone simp rules, and
  missing theorem search results.
- [x] Confirm that Human `by` proof examples do not change Machine Surface
  fixture hashes.

Acceptance criteria:

- [x] `simp-lite` closes basic Nat/List goals.
- [x] `rw [Nat.add_zero]` and `rw [List.append_nil]` work using real stdlib
  theorems.
- [x] Theorems containing unresolved goals, sorry-equivalents, or disallowed
  axioms are not turned into certificates.

Verification:

```sh
cargo test -p npa-tactic
cargo test -p npa-api human
cargo test -p npa-api search
cargo test -p npa-api phase7
```

Dependencies:

```text
P6H-11
```

### P6H-13: Connect Real Stdlib Artifacts To The Phase 6 AI Release Loader

Implementation tasks:

- [x] Add an integration fixture that passes the raw `.npcert` artifacts
  generated in P6H-09 to `load_machine_std_mvp_certificates`.
- [x] Generate `MachineStdLibraryRelease` / import bundles / theorem index /
  rewrite profiles / simp profiles / axiom report from real stdlib artifacts.
- [x] Expand the `std.nat.mvp` / `std.list.mvp` / `std.all.mvp` import bundles
  into requests equivalent to Phase 5 `/machine/sessions`.
- [x] Add a regression where a Phase 7 retrieval fixture builds candidates
  from the real stdlib theorem index and returns to Phase 5 batch.
- [x] Confirm the Phase 8 audit hook can recheck agreement between sidecars and
  verifier output.

Acceptance criteria:

- [x] Release manifest hashes match certificate bytes / sidecar hashes.
- [x] Recommended tactic options recipes pass Phase 5 option validation.
- [x] Artifacts with stale export_hash / certificate_hash /
  decl_interface_hash are rejected.
- [x] AI candidates are not adopted without Phase 5 run/batch/replay/verify.

Verification:

```sh
cargo test -p npa-api std_library
cargo test -p npa-api phase7
cargo test -p npa-api independent_checker
```

Dependencies:

```text
P6H-12
```

### P6H-14: Fix Documentation / Release Regression Gate

Implementation tasks:

- [x] Reflect the Phase 6 Human standard-library source / artifact handoff
  status in the README implementation status.
- [x] Confirm that the module set, simp set, RwOnly set, and axiom exception
  match between `develop/phase6-human.md` and `develop/phase6-ai.md`.
- [x] Clarify the relationship between legacy fixture module names and release
  module names in docs / tests.
- [x] Run formatting, clippy, workspace tests, and the Phase 9 regression
  script as the final regression gate.
- [x] Document whether generated artifacts are commit targets or build
  artifacts.

Acceptance criteria:

- [x] Phase 6 Human completion criteria are traceable through docs and tests.
- [x] `./scripts/phase9-regression.sh` passes.
- [x] The working tree has no generated junk or stale fixture artifacts left
  behind.

Verification:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

Dependencies:

```text
P6H-13
```

---

## 4. Milestone dependency graph

```text
P6H-00
  ↓
P6H-01
  ↓
P6H-02
  ├── P6H-03
  │     ↓
  │   P6H-04
  │     ↓
  │   P6H-05
  │     ↓
  │   P6H-06
  │     ↓
  │   P6H-07
  └── P6H-08
        ↓
P6H-01 + P6H-02 + P6H-03 + P6H-04 + P6H-05 + P6H-06 + P6H-07 + P6H-08
  ↓
P6H-09
  ↓
P6H-10
  ↓
P6H-11
  ↓
P6H-12
  ↓
P6H-13
  ↓
P6H-14
```

P6H-08 depends only on `Std.Logic`, so it can proceed in parallel with
Nat/List work. However, the full source package build in P6H-09 requires all
four release modules to be present.

---

## 5. Excluded Items

The MVP does not include the following.

```text
- full typeclass system
- coercions
- overloaded algebraic notation
- integers, rationals, reals
- finite sets, options, arrays, trees
- quotient types
- classical choice
- function extensionality
- proof irrelevance axiom
- groups, rings, fields
- order theory
- category theory
- full simp
- ring / omega / linarith
- theorem ranking or embedding as trusted metadata
- automatic import insertion by AI server
- server-side package download during Machine API request handling
```

---

## 6. Completion Criteria

Phase 6 Human can be considered complete when the following conditions hold.

```text
- The Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic source package
  skeleton exists, the manifest fixes module membership / certificate paths,
  and the source skeleton fixes import intent.
- The four release modules are generated as `.npcert` files checked by the
  kernel / verifier.
- Import entries carry export_hash, and high-trust mode also checks
  certificate_hash.
- Module export_hash / certificate_hash / axiom_report_hash values are
  generated.
- The axiom report is empty or contains only the exact Std.Logic.Eq.rec
  kernel-standard exception.
- The responsibilities of the Human theorem search view and AI Machine theorem
  index are separated.
- simp-lite closes basic Nat/List goals.
- Theorem search returns exact / apply / rw / simp candidates.
- Tests reproving basic theorems with only Phase 4 tactics pass.
- The Phase 6 AI release loader receives real standard-library certificate
  artifacts, and Phase 5 / Phase 7 / Phase 8 regressions pass.
```
