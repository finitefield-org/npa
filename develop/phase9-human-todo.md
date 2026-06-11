# Phase 9 Human Task Breakdown

This task breakdown treats `develop/phase9-human.md` as authoritative and splits
the gap from the current `crates/npa-kernel` / `crates/npa-cert` /
`crates/npa-frontend` / `crates/npa-api` / `crates/npa-checker-ref`
implementation into kernel-facing / checker-facing / user-facing advanced
feature milestones for the Phase 9 Human Profile.

Phase 9 Human implements advanced inductives, strengthened universe
polymorphism, typeclasses, quotients, SMT certificates, theorem graphs, and
natural language formalization as high-level features usable by humans. However,
it does not change the trust boundary.

```text
Not trusted:
  parser / elaborator / typeclass search / SMT solver / theorem graph / AI formalizer / automation

Trusted:
  small Rust kernel
  canonical core AST
  canonical certificate
  independent checker
```

The Phase 9 AI deterministic validation / replay substrate in
`crates/npa-api/src/advanced_ai.rs` and the M9 fixture matrix are treated as
implemented. This is untrusted automation that returns advanced feature
candidates to the checking boundary; it does not replace kernel / checker-facing
trusted rules in the Human Profile.

Important constraints:

```text
- Do not put AI calls, SMT solver processes, theorem graph stores, networking, plugin loading, or filesystem discovery into the kernel.
- Typeclass search, theorem graph ranking, and natural language formalizers are not proof acceptance boundaries for the kernel / checker.
- Do not treat SMT solver unsat / valid results alone as success. The final NPA proof term is checked by the kernel / checker.
- If a quotient primitive is added, add the same rules not only to the fast kernel but also to the reference checker / external checker profile.
- Do not put unresolved universe metas, AI traces, typeclass search traces, SMT solver logs, or natural language confidence into certificates.
- Hashes for generated recursors / iota rules / theorem graphs / SMT reconstruction / intent certificates are deterministic.
- Do not synchronously insert the full independent checker, external checker, SMT proof reconstruction, release audit, or certificate-wide graph extraction into the AI candidate hot path.
- Phase 9 Human changes must not break the Phase 9 AI fixture matrix, Phase 5-7 replay / verify identity hashes, or Phase 8 checker result semantics.
```

---

## 0. Current Implementation Boundary

### 0.1 Things Treated As Implemented

The current repository has the following implementations usable as the foundation
for Phase 9 Human.

```text
crates/npa-kernel
- core Expr / Level / Env / Decl
- simple inductive / constructor / recursor check
- positivity failure / Prop large elimination restriction
- β / δ / ι / ζ conversion

crates/npa-cert
- canonical .npcert encode / decode
- declaration / export / certificate / axiom report hash
- simple inductive generated recursor artifact hash
- Phase 2 canonical inductive artifact generator

crates/npa-frontend
- Human Surface parser / resolver / elaborator
- simple inductive declaration
- explicit universe argument handling
- Human source interface metadata outside certificate hash

crates/npa-api
- Phase 5 Machine / Human API substrate
- Phase 7 search controller and replay / verify handoff
- Phase 8 independent checker audit automation
- Phase 9 AI advanced automation endpoint substrate and M9 fixture matrix

crates/npa-checker-ref
- source-free reference checker binary
- minimal type / conversion / simple inductive / axiom report checker
```

### 0.2 Target Scope Implemented In Phase 9 Human

```text
- universe meta / constraint solving / canonicalization and optional cumulativity policy
- indexed / mutual / approved nested inductive declarations
- deterministic recursor / induction principle / iota rule generation and checker comparison
- certificate-derived theorem graph extraction, deterministic export, query API, and retrieval integration
- class / instance surface syntax, dictionary elaboration, bounded typeclass search, and notation integration
- quotient_v1 primitive extension, feature flag, reference checker support, and Std.Quotient examples
- SMT certificate schema, small QF fragment encoding, proof reconstruction, and smt tactic
- natural language formalization confirmation flow, reverse translation, and intent certificate
- final documentation / release gate alignment
```

### 0.3 Items Remaining As Target Integrations In Phase 9 Human

```text
- production LLM / RAG / external SMT solver service operation
- online graph store or embedding index operation
- general nested inductive positivity beyond approved strictly-positive functors
- full nonlinear arithmetic, arrays, bitvectors, datatypes, and quantifier-heavy SMT success
- verified checker implementation beyond current reference checker profile
- AI confidence or theorem graph score as proof acceptance condition
```

---

## 1. Design Rules For Protecting The AI Fast Path

Each Phase 9 Human milestone treats the following as acceptance criteria.

```text
- Phase 9 Human heavy checks do not enter the inner loop of AI candidate enumeration.
- The theorem graph is not extracted from the whole certificate for every candidate; search a deterministic snapshot produced during build / release / index update.
- Bounded typeclass search has timeout / max_depth / max_candidates / cycle detection.
- SMT solver processes and proof reconstruction run at tactic / adoption / audit boundaries, not as ranking features.
- Natural language formalization fixes the formal statement hash before proof search.
- Phase 9 AI deterministic validation / replay fixtures remain reproducible without an AI model, network, or random seed.
- Phase 9 Human metadata does not change deterministic identity hashes for Phase 5-7 replay / verify or Phase 8 checker results.
```

The AI path keeps the following shape.

```text
Machine Surface request
  -> Phase 5 machine session / tactic batch / replay / verify
  -> Phase 7 candidate ranking / repair / minimization
  -> Phase 9 AI bounded candidate validation where applicable
  -> closed certificate candidate
  -> optional post-acceptance / checker / release audit
```

---

## 2. Implementation Order

Phase 9 Human implements section #10 of `develop/phase9-human.md` as
authoritative. First fix universes and advanced inductives, then layer theorem
graphs, typeclasses, quotients, SMT, and natural language formalization on top.

```text
0. Fix the Phase 9 Human / AI boundary and performance guard
1. Fix the universe constraint data model / canonical hash
2. Implement universe meta solver / elaborator integration
3. Fix universe-polymorphic library regression and checker consistency
4. Implement indexed inductive family core / certificates
5. Implement mutual inductive blocks / simultaneous recursors
6. Implement approved nested inductive / large elimination policy
7. Implement the certificate-derived theorem graph extractor
8. Implement theorem graph API / retrieval integration / performance guard
9. Implement class / instance declarations and dictionary elaboration
10. Implement bounded typeclass search / notation integration
11. Fix quotient_v1 primitive / certificate feature flag
12. Implement Std.Quotient / checker support / quotient examples
13. Implement SMT certificate schema / QF encoding / deterministic checker surface
14. Implement SMT proof reconstruction / smt tactic
15. Implement natural language formalization / intent certificates
16. Fix final docs / release completion gate
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-kernel
cargo test -p npa-cert
cargo test -p npa-frontend
cargo test -p npa-api advanced_ai
cargo test -p npa-checker-ref
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. Task List

### P9H-00: Fix The Phase 9 Human / AI Boundary And Performance Guard

Implementation tasks:

- [x] Connect the Phase 9 implementation boundary in `develop/phase9-human.md`, `develop/phase9-ai.md`, and README to test names or public docs.
- [x] Add regressions that Phase 9 Human heavy checks are not synchronously inserted into the AI candidate hot path.
- [x] State in public API docs that the Phase 9 AI substrate in `crates/npa-api` is not a trusted checker.
- [x] Add fixtures showing Phase 9 Human metadata does not change Phase 5-7 replay / verify identity hashes or Phase 8 checker results.
- [x] Confirm by README / docs / local script names that the Phase 9 Regression gate remains the fixed gate after Phase 9 Human.

Acceptance criteria:

- [x] Tests fix that AI sidecars, theorem graph scores, SMT solver output, and formalization confidence cannot create checker verdicts.
- [x] The full independent checker / external checker / release audit / SMT reconstruction do not enter the inner loop of AI candidate enumeration.
- [x] `/machine/*` request / response schemas, candidate hashes, and state fingerprints do not change when the Phase 9 Human boundary is added.

Verification:

```sh
cargo test -p npa-api advanced_ai
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

Dependencies:

```text
None
```

Notes:

```text
P9H-00 only handles boundary fixing and regression guards. Advanced features are implemented in later milestones.
```

### P9H-01: Fix Universe Constraint Data Model / Canonical Hash

Implementation tasks:

- [x] Add structured types for universe constraints to `crates/npa-kernel` and align them with `zero / succ / max / imax / param`.
- [x] Make per-declaration universe params / constraints representable in kernel declarations and certificate declarations.
- [x] Add constraint canonicalization and deterministic hashing to `crates/npa-cert`.
- [x] Make certificate encode / decode / verifier / reference checker reject unresolved universe metas.
- [x] Document Option A equality-only universe policy as the MVP default, and do not include cumulativity without an explicit feature flag.

Acceptance criteria:

- [x] Empty constraints equivalent to `List.map` and non-empty constraints equivalent to `max u v <= w` have canonical hashes.
- [x] Universe param order, duplicates, unknown params, and non-canonical level expressions become deterministic errors.
- [x] Certificate hashes / import hashes change stably according to universe constraints.
- [x] The reference checker and fast verifier check constraint canonical bytes with the same meaning.

Verification:

```sh
cargo test -p npa-kernel universe
cargo test -p npa-cert universe
cargo test -p npa-checker-ref universe
cargo test --workspace certificate_hash
```

Dependencies:

```text
P9H-00
```

Notes:

```text
Do not mix defeq and cumulativity. If cumulativity is added, treat it as a subtyping rule in a separate milestone.
```

### P9H-02: Implement Universe Meta Solver / Elaborator Integration

Implementation tasks:

- [x] Introduce elaboration-only universe metas in `crates/npa-frontend`.
- [x] Implement universe meta constraint collection, solving, minimization, and failure diagnostics.
- [x] Reflect solved universe args into explicit core terms and ensure no metas remain in certificates.
- [x] Keep Human Surface implicit universe inference separate from the Machine Surface explicit universe fast path.
- [x] Confirm that the Phase 9 AI `UniverseRepair` fixture and Human elaborator solver output return to the same canonical constraints.

Acceptance criteria:

- [x] Theorems equivalent to polymorphic identity / const / map can be elaborated from Human Surface without explicit universe args.
- [x] Unsolved metas, ambiguous universes, and unsatisfied constraints become structured diagnostics.
- [x] Machine Surface still requires explicit universe args, and Human inference does not change candidate hashes.
- [x] Solver results are deterministic, and the same source / imports produce the same certificate hash.

Verification:

```sh
cargo test -p npa-frontend universe
cargo test -p npa-api universe_repair
cargo test -p npa-api human
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-01
```

Notes:

```text
Universe metas are elaboration-only. Do not put them into the canonical core AST of the kernel / certificate / checker.
```

### P9H-03: Fix Universe-polymorphic Library Regression And Checker Consistency

Implementation tasks:

- [x] Regenerate polymorphic declarations in `Std.Logic` / `Std.List` / `Std.Algebra.Basic` with universe constraints.
- [x] Add fixtures where reference checker and fast kernel universe checks / conversion checks agree.
- [x] Add Human examples and Machine API handoff for universe-polymorphic theorem reuse.
- [x] Confirm that constraint canonical hashes are reflected in imports / release manifests / theorem indexes.
- [x] Update docs for equality-only MVP policy and future cumulativity policy.

Acceptance criteria:

- [x] Declarations equivalent to polymorphic `List` / `Eq` / `Prod` / `Sigma` can be checked by the source-free reference checker.
- [x] Positive and negative universe constraint cases exist in kernel / cert / checker tests.
- [x] Certificate fixtures containing unresolved metas are rejected by both fast verifier and reference checker.
- [x] Candidate hashes for Phase 7 retrieval / Phase 9 AI fixtures do not change after adding Human universe inference.

Verification:

```sh
cargo test -p npa-api std_library
cargo test -p npa-checker-ref universe
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-02
```

Notes:

```text
P9H-03 is mainly standard-library universe hardening. New algebraic hierarchy work is handled in P9H-09 and later.
```

### P9H-04: Implement Indexed Inductive Family Core / Certificates

Implementation tasks:

- [x] Fix an indexed family representation in `InductiveDecl` / certificate schema that clearly separates params and indices.
- [x] Have the kernel check that constructor results match the target family and declared params, and that indices are well typed.
- [x] Add certificate fixtures for `Vec` / `Fin`.
- [x] Support indexed families in generated recursor signature hashes / iota rules hashes.
- [x] Have the reference checker recheck indexed family declarations independently from the fast kernel.
- [x] Add a Human API wrapper corresponding to `POST /inductive/check` that returns constructors / recursor / positivity / iota hash.

Acceptance criteria:

- [x] Constructor result checks for `Vec A 0` / `Vec A (succ n)` pass.
- [x] Constructor result family mismatch, param mismatch, bad index type, and negative occurrence become deterministic errors.
- [x] Recursors / induction principles for indexed families have the same hash in the checker and fast kernel.
- [x] `.npcert` can be checked without source.
- [x] `/inductive/check` responses are diagnostic metadata and are not proof acceptance boundaries.

Verification:

```sh
cargo test -p npa-kernel inductive
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref inductive
cargo test -p npa-api inductive
```

Dependencies:

```text
P9H-03
```

Notes:

```text
Do not adopt AI-supplied recursors. Recursors are generated deterministically from declarations and checked.
```

### P9H-05: Implement Mutual Inductive Blocks / Simultaneous Recursors

Implementation tasks:

- [x] Add `MutualInductiveBlock` to the kernel / certificate / checker boundary.
- [x] Check whole-block name uniqueness, well-typedness, strict positivity, and constructor reference scope.
- [x] Add mutual inductive fixtures for `Even` / `Odd`.
- [x] Implement deterministic generation of simultaneous recursors / induction principles and iota rules.
- [x] Make import / export / theorem index handling use stable order for generated declarations from mutual blocks.

Acceptance criteria:

- [x] Mutually recursive `Even` / `Odd` can be checked as source-free certificates.
- [x] Block-local reference scope mismatch, duplicate generated names, and non-positive mutual occurrences are rejected.
- [x] Generated recursor artifact hashes are stable with respect to declaration order and canonical name order.
- [x] Iota reduction agrees between the reference checker and fast kernel.

Verification:

```sh
cargo test -p npa-kernel mutual
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref inductive
cargo test -p npa-api std_library
```

Dependencies:

```text
P9H-04
```

Notes:

```text
Adopting mutual blocks expands the kernel trusted base, so leave the reason / alternative / checker boundary in docs.
```

### P9H-06: Implement Approved Nested Inductive / Large Elimination Policy

Implementation tasks:

- [x] Implement the approved strictly-positive functor table as explicit kernel / checker policy.
- [x] Handle nested recursive occurrences through `List` / `Option` / `Prod` in positivity traversal.
- [x] Add a positive fixture for `Rose` tree and rejection fixtures for unknown functors / higher-order negative occurrences.
- [x] Make the large elimination restriction from `Prop` to `Type` and candidate exceptions structured policy.
- [x] Support the approved nested profile in recursor generation and iota rules hashes.

Acceptance criteria:

- [x] Approved nested occurrences equivalent to `List (Rose A)` pass.
- [x] Unknown type constructors, `I -> A`, `I -> I`, and higher-order negative occurrences are rejected.
- [x] Recursors from `I : Prop` to unrestricted `Type` motives are rejected.
- [x] The approved functor table agrees across certificates / checkers / docs.

Verification:

```sh
cargo test -p npa-kernel positivity
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref positivity
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-05
```

Notes:

```text
The generic positivity checker is a future target. P9H-06 is limited to the approved functor profile.
```

### P9H-07: Implement Certificate-derived Theorem Graph Extractor

Implementation tasks:

- [x] Add a certificate-derived theorem graph extractor to `crates/npa-api` or a dedicated module.
- [x] Fix node schema, edge schema, node identity, edge identity, and deterministic graph hash.
- [x] Extract `Const`s appearing in types / proofs / transparent def bodies / constructor types / axiom deps.
- [x] Exclude source notation, tactic scripts, and AI sidecars from graph extraction input.
- [x] Add fixtures for axiom dependency paths and direct / transitive dependency queries.

Acceptance criteria:

- [x] The same `.npcert` input produces the same graph export hash.
- [x] Graph hash does not change when source text or Human debug metadata changes.
- [x] Axiom deps, constructor deps, and recursor deps appear in the graph.
- [x] Binding between import `export_hash` / high-trust `certificate_hash` and graph snapshots can be checked.

Verification:

```sh
cargo test -p npa-api theorem_graph
cargo test -p npa-api std_library
cargo test -p npa-checker-ref
```

Dependencies:

```text
P9H-06
```

Notes:

```text
P9H-07 is graph extraction. It does not implement an online graph store, embeddings, or RAG.
```

### P9H-08: Implement Theorem Graph API / Retrieval Integration / Performance Guard

Implementation tasks:

- [x] Add Human API wrappers corresponding to `/graph/dependencies` / `/graph/related` / `/graph/query` to `crates/npa-api`.
- [x] Let Phase 7 premise retrieval use a precomputed theorem graph snapshot as a ranking feature.
- [x] Fix by tests that graph proximity scores do not affect proof acceptance / checker verdicts.
- [x] Let proof minimization use the graph to suggest unnecessary import candidates.
- [x] Add performance regression that does not put graph extraction into the per-candidate hot path.

Acceptance criteria:

- [x] Direct / transitive dependencies and related theorem queries are returned deterministically per declaration.
- [x] Graph result nodes are limited to certificate-bound public exports.
- [x] Final certificate hashes and checker results do not change based on the presence or absence of graph scores.
- [x] Phase 7 retrieval has the existing deterministic fallback when graph snapshots are missing.

Verification:

```sh
cargo test -p npa-api theorem_graph
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-07
```

Notes:

```text
The theorem graph is a sidecar for AI search efficiency. Do not put it in the trusted base.
```

### P9H-09: Implement Class / Instance Declarations And Dictionary Elaboration

Implementation tasks:

- [x] Add `class` / `instance` declaration syntax to Human Surface.
- [x] Elaborate classes into ordinary core declarations equivalent to structures / records plus searchable metadata.
- [x] Put instance declarations into certificates as ordinary definitions, and separate metadata outside certificate hashes.
- [x] Add an elaboration path that passes dictionary arguments as explicit core terms.
- [x] Add minimal examples for `Add` / `Mul` / `Zero` / `One`.

Acceptance criteria:

- [x] Class declarations can be checked as ordinary declarations even when the kernel knows nothing about typeclasses.
- [x] Broken instance metadata can be rejected by type checking the final core term.
- [x] Certificates retain only explicit dictionary terms, not search traces.
- [x] The Machine Surface fast path does not require typeclass metadata.

Verification:

```sh
cargo test -p npa-frontend typeclass
cargo test -p npa-api human
cargo test -p npa-cert certificate_hash
```

Dependencies:

```text
P9H-08
```

Notes:

```text
P9H-09 covers declarations and dictionary elaboration. Search algorithms are handled in P9H-10.
```

### P9H-10: Implement Bounded Typeclass Search / Notation Integration

Implementation tasks:

- [x] Implement bounded instance search with local / opened namespace / imported global / fallback priority.
- [x] Make max_depth / max_candidates / timeout / cycle detection / repeated goal cache into policy.
- [x] Make ambiguity, no solution, and budget exceeded into structured diagnostics.
- [x] Add a Human API wrapper corresponding to `POST /typeclass/search` that returns instance / core_term / bounded search trace.
- [x] Elaborate `+` / `*` / `0` / `1` notation through typeclasses into dictionary terms.
- [x] Connect the boundary between the Phase 9 AI TypeclassResolution fixture and Human search behavior to docs / tests.

Acceptance criteria:

- [x] Direct and recursive instances for `Add Nat` can be resolved within budget.
- [x] When multiple different proof terms exist, do not choose by score; return an ambiguity error.
- [x] Search traces are diagnostic metadata and do not enter certificate hashes.
- [x] The `core_term` in `/typeclass/search` responses is a kernel-checkable dictionary term, and the search trace is not a proof acceptance boundary.
- [x] Timeout / budget bounds latency on the AI hot path.

Verification:

```sh
cargo test -p npa-frontend typeclass
cargo test -p npa-api typeclass
cargo test -p npa-api advanced_ai
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-09
```

Notes:

```text
Typeclass search is an untrusted elaborator mechanism. Do not put searchers into the kernel / checker.
```

### P9H-11: Fix quotient_v1 Primitive / Certificate Feature Flag

Implementation tasks:

- [x] Add the `quotient_v1` core feature flag and unsupported feature rejection to certificates / checkers.
- [x] Fix the primitive interface for `Quotient`, `Quotient.mk`, `Quotient.sound`, and `Quotient.lift` as kernel policy.
- [x] Implement the scope where the `Quotient.lift` computation rule enters definitional equality.
- [x] Treat `Quotient.sound` as a proof term and add regressions that do not over-normalize quotient equality.
- [x] Make quotient use appear in axiom reports / feature reports.

Acceptance criteria:

- [x] Checkers without `quotient_v1` support reject quotient certificates deterministically.
- [x] In profiles supporting `quotient_v1`, primitive interfaces agree between the fast kernel and reference checker.
- [x] Quotient primitives are not silently allowed as custom axioms.
- [x] Feature flags, certificate hashes, and axiom report hashes change deterministically.

Verification:

```sh
cargo test -p npa-kernel quotient
cargo test -p npa-cert quotient
cargo test -p npa-checker-ref quotient
cargo test -p npa-api quotient
```

Dependencies:

```text
P9H-10
```

Notes:

```text
This milestone expands the kernel trusted base. Leave reasons, alternatives, and checker boundaries in docs.
```

### P9H-12: Implement Std.Quotient / Checker Support / Quotient Examples

Implementation tasks:

- [x] Add `Setoid`, relation notation, and quotient helper definitions to `Std.Quotient`.
- [x] Add a quotient-capable independent checker profile to Phase 8 / Phase 9 API policy.
- [x] Update the deterministic rejection surface of Phase 9 AI `QuotientConstruction` so it does not conflict with the quotient success profile.
- [x] Add an example certificate that builds a simple `Int` from `Nat x Nat`.
- [x] Add fixtures for the well-defined proof obligation of `Quotient.lift` and compatibility proof mismatch.

Acceptance criteria:

- [x] Proofs using `Setoid`, `Quotient.mk`, `Quotient.sound`, and `Quotient.lift` pass the source-free checker.
- [x] The kernel / checker rejects incorrect relation equivalence proofs or compatibility proofs.
- [x] The `Phase8MvpReference` unsupported fixture for Phase 9 AI MVP does not become stale after adding the new profile.
- [x] Quotient examples do not depend on custom axioms / sorry.

Verification:

```sh
cargo test -p npa-api std_library
cargo test -p npa-checker-ref quotient
cargo test -p npa-api advanced_ai
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-11
```

Notes:

```text
P9H-12 introduces the quotient-capable profile. Production full external checker integration is treated as Phase 8 target integration.
```

### P9H-13: Implement SMT Certificate Schema / QF Encoding / Deterministic Checker Surface

Implementation tasks:

- [x] Add format, solver, logic, encoded_goal_hash, smt_problem_hash, proof_hash, and reconstruction metadata to the SMT certificate schema.
- [x] Implement encoding tables for QF propositional / EUF / simple LIA and Nat-to-Int side condition representation.
- [x] Generate SMT-LIB problem bytes and encoding hashes deterministically.
- [x] Allow proof payloads such as Alethe / LFSC to be hash / size / schema validated as opaque artifacts.
- [x] Make unsupported fragments, solver-result-only, hash mismatch, and malformed proof payloads into structured errors.

Acceptance criteria:

- [x] Supported-fragment decisions for QF propositional / EUF / simple LIA are deterministic.
- [x] SMT solver unsat results alone do not become success.
- [x] Encoded problem hashes, SMT problem hashes, and proof payload hashes are stable.
- [x] Phase 9 AI SMT deterministic rejection fixtures do not contradict the Human SMT schema.

Verification:

```sh
cargo test -p npa-api smt
cargo test -p npa-api advanced_ai
cargo test -p npa-cert certificate_hash
```

Dependencies:

```text
P9H-12
```

Notes:

```text
P9H-13 covers schema / encoding / deterministic rejection surface. Solver-native success is handled in P9H-14.
```

### P9H-14: Implement SMT Proof Reconstruction / smt Tactic

Implementation tasks:

- [x] Convert proof-producing reconstruction for the small QF fragment into NPA proof terms.
- [x] Fix the reconstruction rule registry as a non-empty profile and add rule descriptor fingerprints.
- [x] Make the `smt` / `smt [lemmas]` tactics callable from Human Surface.
- [x] Add a Human API wrapper corresponding to `POST /smt/prove` that returns problem hash / proof hash / NPA proof hash / kernel_checked.
- [x] Return success only after checking the final NPA proof term with the kernel / reference checker.
- [x] Return failure / unsupported fragment / checker mismatch as structured diagnostics.

Acceptance criteria:

- [x] SMT final proof success is returned only when the reconstructed NPA proof term passes the kernel / checker.
- [x] Unknown solver-native proof rules, ambiguous premise order, or final conclusions not defeq to the target are rejected.
- [x] The `smt` tactic does not treat solver results as trusted input.
- [x] `/smt/prove` returns `kernel_checked: true` and the checked proof hash on success with `require_certificate: true`.
- [x] SMT reconstruction does not enter the inner loop of AI ranking / candidate enumeration.

Verification:

```sh
cargo test -p npa-api smt
cargo test -p npa-tactic smt
cargo test -p npa-checker-ref smt
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-13
```

Notes:

```text
Keep the first success fragment small. Arrays, bitvectors, nonlinear arithmetic, and quantifiers are later scope.
```

### P9H-15: Implement Natural Language Formalization / Intent Certificates

Implementation tasks:

- [x] Add a Human API wrapper corresponding to `/formalize` that returns multiple formal candidates, reverse translation, and ambiguity reports.
- [x] Store formal statement hashes, candidate statement hashes, accepted statement hashes, and intent certificates separately.
- [x] Make reviewer identity and status for user confirmation / formalization verifier into structured metadata.
- [x] Make proof search start only after a confirmed formal statement hash.
- [x] Add UI / API / docs regressions that do not call unconfirmed formalizations verified.

Acceptance criteria:

- [x] Natural language source text or confidence scores alone cannot define theorem statements.
- [x] Candidate statements pass through Machine Surface / Human elaboration and become canonical core statement hashes.
- [x] Intent certificates and proof certificates are separate artifacts, and natural language text is not mixed into kernel proof certificate hashes.
- [x] Fixtures for rejected / unreviewed / reviewed formalizations are deterministic.

Verification:

```sh
cargo test -p npa-api formalization
cargo test -p npa-api advanced_ai
cargo test -p npa-api human
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-14
```

Notes:

```text
Quality evaluation of LLM candidate generation is out of scope. P9H-15 fixes confirmation and certificate separation.
```

### P9H-16: Fix Final Docs / Release Completion Gate

Implementation tasks:

- [x] Synchronize Phase 9 completion status across `develop/phase9-human.md`, `develop/phase9-ai.md`, README, and `develop/overall-design.md`.
- [x] Fix by search that the difference between Phase 9 Human target scope and Phase 9 AI MVP implemented substrate has not gone stale.
- [x] Confirm that `./scripts/phase9-regression.sh` includes the tests needed as the required gate after Phase 9 Human completion.
- [x] Leave the trusted boundary, AI hot path performance boundary, and quotient / SMT feature flag boundary in docs.
- [x] Reconfirm in release / high-trust docs that only checker results and deterministic artifacts decide pass/fail.

Acceptance criteria:

- [x] Phase 9 Human completion criteria agree across docs / tests / release gate.
- [x] Phase 9 AI sidecars, graph scores, formalization confidence, and SMT solver output are not in the trusted boundary.
- [x] Production AI / graph store / full solver support that remain target integrations are not described as implemented.
- [x] No required gate slowing the AI candidate hot path has been added to PR / candidate enumeration.

Verification:

```sh
rg -n "Phase 9|advanced inductive|universe|typeclass|quotient|SMT|theorem graph|formalization|AI hot path" README.md develop/phase9-human.md develop/phase9-ai.md develop/overall-design.md
git diff --check
./scripts/phase9-regression.sh
```

Dependencies:

```text
P9H-15
```

Notes:

```text
P9H-16 is final documentation / gate alignment. Do not describe unimplemented production integrations as implemented.
```

---

## 4. Completion Criteria

Phase 9 Human can be called complete when:

```text
- universe metas are resolved elaboration-only, and only canonical constraints remain in certificates.
- polymorphic List / Eq / Prod / Sigma / Functor / Category-like definitions can be handled.
- Vec / Fin / mutual Even/Odd / approved nested Rose pass the kernel / reference checker.
- recursors / induction principles / iota rules are generated deterministically from declarations, and hashes agree between the checker and fast kernel.
- theorem graphs are extracted deterministically from certificates and can be used as dependencies / related query / retrieval sidecars.
- typeclass classes / instances are elaborated into ordinary core dictionary terms, and search traces do not enter certificates.
- quotient_v1 has a feature flag and checker support, and Setoid quotient / Quotient.lift examples pass the source-free checker.
- SMT certificates do not succeed on solver results alone; the kernel / checker checks reconstructed NPA proof terms.
- natural language formalization separates formal statement hashes and intent certificates from proof certificates.
- Phase 9 Human heavy checks are positioned so they do not slow the AI candidate hot path.
- `./scripts/phase9-regression.sh` passes as the Phase 9 completion gate.
```

---

## 5. Things Not Included In The MVP

```text
- using AI confidence for proof acceptance
- using theorem graph scores for checker verdicts
- launching SMT solver processes inside the kernel / checker
- putting typeclass search into the kernel / checker
- putting natural language text into proof certificate hashes
- production LLM / RAG / online graph store operation
- SMT success for full nonlinear arithmetic / arrays / bitvectors / quantifiers
- unrestricted generic nested inductive positivity
- silently allowing quotients as custom axioms
```
