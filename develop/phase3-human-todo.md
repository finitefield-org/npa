# Phase 3 Human TODO

This TODO treats `develop/phase3-human.md` as authoritative and breaks down
the gap from the current `crates/npa-frontend` / `crates/npa-api`
implementation into implementation tasks.

As an important constraint, Phase 3 Human adds a surface language that is easy
for humans to read and write, but must not reduce the execution speed or
determinism of the AI-facing Machine Surface. Human-facing features are not
mixed into the Machine Surface hot path; they are added as separate parser /
AST / resolver / elaborator entrypoints.

---

## 0. Current Implementation Boundary

### 0.1 Items Treated As Implemented

The current `crates/npa-frontend` is implemented primarily around the
AI-facing Machine Surface. The following may be used as assumptions for the
Phase 3 Human implementation.

```text
crates/npa-frontend/src/lib.rs
- Machine Surface public API exports
- module compile / term check / certificate handoff API

crates/npa-frontend/src/machine.rs
- MachineModule / MachineItem / MachineDecl / MachineTerm
- MachineSurfaceMode
- MachineTermElabContext
- Machine Surface callable interface table

crates/npa-frontend/src/lexer.rs
- Machine Surface tokenization
- unsupported character rejection

crates/npa-frontend/src/parser.rs
- import / def / theorem
- explicit universe parameters
- explicit binders
- Prop / Type / Sort
- application / lambda / forall / let / annotation / parenthesized term
- rejection of open / namespace / notation / axiom / inductive / hole that are not used in Machine Surface

crates/npa-frontend/src/resolver.rs
- verified import interface lookup
- local context lookup
- deterministic global lookup
- no filesystem / network import lookup

crates/npa-frontend/src/elaborator.rs
- Machine Surface module elaboration
- term-level infer / check API
- Phase 1 kernel handoff
- Phase 2 certificate build / verify handoff

crates/npa-frontend/src/term_source.rs
- canonical Machine Surface term source encoding
- canonical hash
- canonical decode
```

On the `crates/npa-api` side, Phase 4 / 5 / 7 / 8 / 9 automation assumes
Machine Surface terms and calls `parse_machine_term`,
`canonicalize_machine_term_source`、`elaborate_machine_term_check`、
and `elaborate_machine_term_infer_from_ast`.
These paths are high-frequency candidate checking paths for AI search, so the
Phase 3 Human implementation must not slow them down.

### 0.2 Unimplemented Human Surface Scope

The following features required by `develop/phase3-human.md` do not currently
exist in the code as Human Surface features.

```text
Human Surface AST
Human parser entrypoint
FrontendState
namespace / open / end
human source import interface reconciliation
axiom declaration
simple inductive declaration
notation declaration
infix / infixl / infixr
operator precedence / associativity table
qualified / unqualified name resolution with open scopes
ambiguous name tracking
notation overload candidate tracking
implicit binder metadata
grouped binder expansion
arrow desugaring
implicit argument insertion
term metavariable store
universe metavariable store
hole goal reporting
named hole context check
simple unification
bidirectional elaboration for omitted arguments
source interface metadata export/import
human diagnostics separate from MachineDiagnostic
Human API endpoints
```

### 0.3 Things That Must Not Enter The Machine Surface Side

The following may be implemented in Human Surface, but must not enter the
Machine Surface grammar / resolver / canonical term source / tactic candidate
path.

```text
notation table
open scope
namespace stack
implicit argument insertion
unresolved hole
named hole reuse
overload transaction
typeclass search
coercion search
source-level axiom syntax
source-level inductive syntax
human display name metadata
filesystem import lookup
network import lookup
```

---

## 1. Design Rules For Protecting The AI-Facing Fast Path

During Phase 3 Human implementation, treat the following as required
acceptance criteria.

```text
- Do not change accepted / rejected syntax for parse_machine_*.
- Do not add Human-only variants to MachineTerm / MachineModule.
- Do not change Machine Surface canonical bytes / canonical hash.
- Do not make MachineTermElabContext lookup depend on namespace / open / notation.
- Phase 4 / 5 / 7 / 8 / 9 automation in npa-api must not go through Human Surface.
- Do not put Human metadata into certificate hash payloads.
- Do not put unresolved holes / metas into certificates.
- Do not perform filesystem / network lookup in the Human resolver either.
- Human overload resolution tries only finite candidates in deterministic order.
- Do not include typeclass / coercion / backtracking-heavy search in the Phase 3 Human MVP.
- Fix Machine Surface regressions at the same time as the Human implementation.
```

The recommended structure is as follows.

```text
AI path:
  parse_machine_* -> resolve_machine_* -> elaborate_machine_* -> kernel / cert

Human path:
  parse_human_* -> resolve_human_* -> elaborate_human_* -> explicit core -> kernel / cert
```

The Human path ultimately produces a fully explicit core AST. It does not
expand the trusted base of the kernel / certificate / independent checker.

---

## 2. Implementation Order

Phase 3 Human is implemented in an order that makes Machine Surface speed
impact easy to detect.

```text
1. Create the Human-only module / API skeleton
2. Add Machine Surface invariance tests
3. Add the Human Surface AST and parser
4. Add the resolver for FrontendState / namespace / open
5. Run explicit-only Human elaboration separately from the Machine elaborator
6. Expand declaration coverage for axiom / simple inductive
7. Add implicit binder / implicit arg insertion / simple unification
8. Add notation / overload resolution with finite candidates
9. Add holes / metavariable goal reporting
10. Fix Phase 2 certificate handoff and API / regression
```

At each stage, check at least the following.

```sh
cargo fmt --all
cargo test -p npa-frontend
cargo test -p npa-api
./scripts/phase9-regression.sh
```

After large internal changes, also pass the following.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 3. Task List

### P3H-00: Fix The Human / Machine Frontend Boundary

Implementation tasks:

- [x] Add Human-only modules inside `crates/npa-frontend`.
- [x] Examples: `human.rs`, `human_parser.rs`, `human_resolver.rs`, `human_elaborator.rs`, `human_diagnostic.rs`.
- [x] Preserve the existing signatures and meaning of `parse_machine_*` / `resolve_machine_*` / `elaborate_machine_*`.
- [x] Do not reuse `MachineTerm` / `MachineModule` as the Human AST.
- [x] If conversion from the Human path to the Machine path is needed, limit it to an explicit lowering API.

AI Speed Guard:

- [x] Fix regression tests showing that `parse_machine_module` continues to reject `open` / `namespace` / `notation` / `axiom` / `inductive` / holes.
- [x] Fix that the canonical bytes / hash of `canonicalize_machine_term_source("@Eq.refl.{1} Nat n")` do not change before and after the Human implementation.
- [x] Keep it grep-able that Phase 7 / Phase 9 in `npa-api` do not call the Human parser.

Affected files:

```text
crates/npa-frontend/src/lib.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
```

Completion criteria:

```text
- Human public API and Machine public API are distinguishable by name
- Machine Surface tests pass with no changes
- phase9 regression passes with only the Human skeleton added
```

### P3H-01: Add The Human Surface AST

Implementation tasks:

- [x] Define `HumanModule` / `HumanItem` / `HumanDecl` / `HumanExpr`.
- [x] Give `HumanItem` `Import` / `Open` / `NamespaceStart` / `NamespaceEnd` / `Def` / `Theorem` / `Axiom` / `Inductive` / `Notation`.
- [x] Give `HumanExpr` equivalents of `Ident` / `App` / `Lam` / `Pi` / `Let` / `Annot` / `Sort` / `Arrow` / `Hole` / `ExplicitMode` / `NotationApp`.
- [x] Give `HumanBinder` `BinderInfo::Explicit` / `BinderInfo::Implicit`.
- [x] Fix the policy for either preserving grouped binders in the AST or expanding them to binder lists immediately after parsing.
- [x] Preserve `Span` on every node.

AI Speed Guard:

- [x] Do not represent the Human AST by adding variants to `MachineTerm`.
- [x] Ensure the Machine canonical encoder does not need to know about the Human AST.

Affected files:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/span.rs
```

Completion criteria:

```text
- `def id (A : Type) (x : A) : A := x` can become a Human AST
- Binder info for `{A : Type}` and `(A : Type)` is distinguished
- `_` and `?m` are preserved as Human holes
- Machine parser AST snapshot tests do not change
```

### P3H-02: Implement The Human Parser Entrypoint

Implementation tasks:

- [x] Add `parse_human_module(file_id, source)`.
- [x] Add `parse_human_term(file_id, source)`.
- [x] Limit `import` to the beginning of the module, and turn mid-module imports into `ImportAfterItem`.
- [x] Parse `open` / `namespace` / `end`.
- [x] Parse `def` / `theorem` / `axiom` / simple `inductive`.
- [x] Parse `fun` / `forall` / `let` / annotation / application / parenthesized terms.
- [x] Desugar `->` / `→` to right-associative anonymous Pi.
- [x] Handle grouped binders `(x y : A)` / `{x y : A}`.
- [x] Treat `_` / `?m` as Human holes.
- [x] Treat `@f` as explicit implicit argument mode.
- [x] Parse `notation` / `infix` / `infixl` / `infixr` declarations.

AI Speed Guard:

- [x] The Human parser is not called from `parse_machine_*`.
- [x] Do not add notation parser state to the Machine lexer/parser.
- [x] Keep Machine parser unsupported syntax rejection tests.

Affected files:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/lib.rs
```

Completion criteria:

```text
- The parser parts of `develop/phase3-human.md` 12.1 through 12.15 can be parsed
- The Human parser rejects import-after-item with a structured diagnostic
- The Machine parser rejects or accepts the same inputs as before
```

### P3H-03: Implement Human FrontendState / Source Interface Metadata

Implementation tasks:

- [x] Add `HumanFrontendState`.
- [x] Store current module name / namespace stack / lexical open scopes.
- [x] Reconcile verified import source interfaces with imports in the source.
- [x] If Human metadata is added to the import interface, separate it from trusted certificate payloads.
- [x] Put notation table / binder metadata / generated declaration display info into the source interface.
- [x] Treat Human metadata as an `npa-frontend` source interface, and do not add it to canonical certificate types in `npa-cert`.
- [x] Make duplicate import handling deterministic.

AI Speed Guard:

- [x] Do not add Human metadata lookup to AI path lookup for `VerifiedImport` / `MachineGlobalScopeEntry`.
- [x] Do not put Human source interface metadata into certificate hashes.
- [x] Do not perform filesystem / network lookup in Human import resolution either.

Affected files:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/resolver.rs
```

Completion criteria:

```text
- Imports are reconciled against the verified import list in the compile request
- Certificate verification still succeeds even if Human metadata is discarded
- Machine verified import lookup behavior does not change
```

### P3H-04: Implement Name Resolution With Namespace / Open

Implementation tasks:

- [x] Separate local context from the global declaration table.
- [x] Fix priority between current module declarations and imported declarations.
- [x] Generate fully qualified declaration names from the namespace stack.
- [x] Implement lexical scope for `open`.
- [x] Implement qualified name / unqualified name resolution.
- [x] Return ambiguous names as `AmbiguousName` with deterministic payloads.
- [x] Reject forward references as `ForwardReference`.

AI Speed Guard:

- [x] Keep the Machine resolver without suffix lookup / open scope lookup.
- [x] Human resolver candidate collection uses a bounded vector with a deterministic sort key.

Affected files:

```text
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/lib.rs
```

Completion criteria:

```text
- The equivalent of `def Nat.zero` inside a namespace resolves to a fully qualified name
- After `open Std.Nat`, `zero` resolves to `Std.Nat.zero`
- Ambiguous unqualified names do not silently choose one side
- Machine resolver direct exact lookup tests pass
```

### P3H-05: Implement The Notation / Infix Table

Implementation tasks:

- [x] Integrate notation declarations with namespace / open scope.
- [x] Resolve notation targets to global refs during declaration processing.
- [x] Fix which of prefix / postfix / infix / infixl / infixr are included in the Phase 3 MVP.
- [x] Reflect infix precedence / associativity in parser binding power.
- [x] Reject notation conflicts as `NotationConflict`.
- [x] Make non-associative infix chains parse errors.
- [x] Preserve overloaded notation candidates in deterministic order.
- [x] Implement candidate transaction / rollback during elaboration in a bounded way.

AI Speed Guard:

- [x] Do not give the Machine parser an active notation table.
- [x] Do not connect Human notation resolution to typeclass search.
- [x] Keep a candidate count limit as an option, and reject overflow as `TooManyNotationCandidates`.

Affected files:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/lib.rs
```

Completion criteria:

```text
- `infixl:65 " + " => Nat.add` can be parsed / registered
- `n + Nat.zero` can be elaborated to the equivalent of `Nat.add n Nat.zero`
- Notation conflicts become deterministic diagnostics
- `n + Nat.zero` in Machine Surface remains unsupported / a parse error
```

### P3H-06: Implement Implicit Binder / Implicit Argument Insertion

Implementation tasks:

- [x] Preserve `{A : Type}` in the source interface as an implicit binder.
- [x] Reflect binder info in the callable profile.
- [x] Insert `SyntheticImplicit` metavariables for implicit binders.
- [x] Do not automatically insert implicit term args in `@f` mode.
- [x] Handle explicit implicit argument syntax with `@` in the Human path.
- [x] Reject certificate generation if unresolved implicits remain when the declaration closes.

AI Speed Guard:

- [x] Preserve Machine Surface behavior that requires implicit arguments.
- [x] `Eq.refl n` is not implicitly completed on the Machine path.
- [x] Connect Human implicit insertion only to bounded simple unification.

Affected files:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/callable.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/lib.rs
```

Completion criteria:

```text
- On the Human path, `Eq.refl n` can resolve to the equivalent of `@Eq.refl.{u} A n`
- On the Human path, `@Eq.refl` suppresses implicit insertion
- Machine path callable profile tests do not change
```

### P3H-07: Implement The Metavariable / Hole / Constraint Store

Implementation tasks:

- [x] Separate the term metavariable store from the universe metavariable store.
- [x] Convert `_` and `?m` to `UserHole` metas.
- [x] Save context snapshots for named holes.
- [x] Turn context mismatches for named hole reuse into `NamedHoleContextMismatch`.
- [x] Represent `TypeEq` / `TermEq` / `LevelEq` / `LevelLe` constraints.
- [x] Implement simple unification and occurs check.
- [x] Treat declarations with remaining unsolved metas / holes as incomplete and reject certificate generation.
- [x] Return hole goals as human-facing diagnostic payloads.

AI Speed Guard:

- [x] Do not put holes in the AST in Machine Surface Complete mode.
- [x] Do not put Repair mode suggestions into trusted payloads.
- [x] Do not bring the Human meta store into MachineTermElabContext.

Affected files:

```text
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/human_resolver.rs
crates/npa-frontend/src/lib.rs
```

Completion criteria:

```text
- On the Human path, `_` becomes a goal diagnostic
- Certificates cannot be created from declarations containing unresolved holes
- Named hole context mismatches can be detected
- `_` on the Machine path remains HoleNotAllowed
```

### P3H-08: Implement Bidirectional Human Elaboration

Implementation tasks:

- [x] Create skeletons for `infer_human_expr` / `check_human_expr`.
- [x] Lower local / global / app / lambda / Pi / let / annotation to core terms.
- [x] Use expected types to process lambdas / holes / notation candidates.
- [x] Do not put a declaration itself into the global env during declaration elaboration.
- [x] Pass elaborated core declarations to the Phase 1 kernel.
- [x] If the kernel rejects, wrap the result in a Human diagnostic.

AI Speed Guard:

- [x] Do not add Human expected-type candidate search to the Machine elaborator.
- [x] Bound Human elaboration backtracking at the notation / overload candidate level.
- [x] Phase 7 candidate checks continue to use Machine term checks.

Affected files:

```text
crates/npa-frontend/src/human_elaborator.rs
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-frontend/src/elaborator.rs
```

Completion criteria:

```text
- Explicit-only Human definitions/theorems become core declarations
- Ill-typed Human terms become structured diagnostics
- Machine explicit elaboration tests do not change
```

### P3H-09: Implement Axiom Declarations

Implementation tasks:

- [x] The Human parser accepts `axiom name : type`.
- [x] The Human resolver registers axiom names in global scope.
- [x] The Human elaborator converts axiom types to core declarations.
- [x] Confirm that Phase 2 certificate handoff reflects them in the axiom report.
- [x] Test cases where verification is rejected by axiom policy.

AI Speed Guard:

- [x] The Machine parser continues to reject source-level `axiom`.
- [x] Do not add AI path routes that generate axioms from source syntax.

Affected files:

```text
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_elaborator.rs
```

Completion criteria:

```text
- Axioms from Human source appear in the certificate axiom report
- Certificate verification rejects certificates containing unauthorized axioms
- Machine Surface source-level axiom rejection tests pass
```

### P3H-10: Implement Simple Inductive Declarations

Implementation tasks:

- [x] Turn simple `inductive` syntax into the Human AST.
- [x] Fix the prohibited scope for self references / forward references.
- [x] Elaborate constructor types using temporary globals.
- [x] Create core-spec v0.1 `InductiveDecl` from constructor types.
- [x] Register generated constructors / recursors in the source interface as `LocalGenerated`.
- [x] Pass the whole inductive to the kernel and register it in the global env only after success.

AI Speed Guard:

- [x] The Machine parser continues to reject source-level `inductive`.
- [x] Do not put generated display metadata into certificate hashes.
- [x] Keep this as a separate path from Phase 9 advanced inductive automation.

Affected files:

```text
crates/npa-frontend/src/human.rs
crates/npa-frontend/src/human_parser.rs
crates/npa-frontend/src/human_elaborator.rs
crates/npa-kernel/src/lib.rs  # only if existing InductiveDecl support is insufficient
```

Completion criteria:

```text
- `inductive Nat : Type where | zero : Nat | succ : forall (n : Nat), Nat` can become a core inductive
- The kernel rejects bad constructor types
- The Human resolver can reference generated declarations
- Machine and Phase 9 regression passes
```

### P3H-11: Implement Human Diagnostics / Repair Payload

Implementation tasks:

- [x] Separate `HumanDiagnosticKind` from MachineDiagnostic.
- [x] Include the parser / resolver / elaborator / kernel handoff phase in the payload.
- [x] Represent AmbiguousName / AmbiguousNotation / NotationConflict / ForwardReference / NamedHoleContextMismatch / UnsolvedMeta.
- [x] Put local context / expected type for hole goal display into the payload.
- [x] Generate human-facing messages from deterministic payloads.

AI Speed Guard:

- [x] Do not change MachineDiagnostic canonicalization or Phase 7 failed candidate error payloads.
- [x] Do not make Human diagnostic message changes part of Machine M9 same-output.

Affected files:

```text
crates/npa-frontend/src/human_diagnostic.rs
crates/npa-api/src/adapter.rs
```

Completion criteria:

```text
- Human-path-specific errors are not mixed into MachineDiagnosticKind
- Machine API error kind regression passes
- Hole goal diagnostics have testable structured payloads
```

### P3H-12: Add Human APIs And Separate Them From Machine APIs

Implementation tasks:

- [x] Expose `parse_human_module` / `resolve_human_module` / `elaborate_human_module` / `compile_human_source_to_core` / `compile_human_source_to_certificate`.
- [x] Include `human` in API names so they are easy to distinguish from Machine APIs.
- [x] If Human compile endpoints / library APIs are added to `npa-api`, use separate types from Machine endpoints.
- [x] Human APIs explicitly receive verified imports / current module input.
- [x] Do not create hidden global state.

AI Speed Guard:

- [x] APIs equivalent to `/machine/*` do not call the Human compile path.
- [x] The default profile for AI automation remains Machine Surface.
- [x] Do not add flags to Human compile options that enable expensive search.

Affected files:

```text
crates/npa-frontend/src/lib.rs
crates/npa-api/src/lib.rs
crates/npa-api/src/types.rs
```

Completion criteria:

```text
- Human API and Machine API types are not mixed
- Certificates can be created from Human source
- Machine session / tactic / search / replay / verify APIs preserve existing tests
```

### P3H-13: Fix Regression / Performance Guard

Implementation tasks:

- [x] Add Machine Surface accepted syntax snapshot tests.
- [x] Add Machine Surface rejected human-feature tests.
- [x] Add a Machine term canonical hash fixture or fix an existing fixture.
- [x] State in CI / docs that `./scripts/phase9-regression.sh` passes after Human implementation.
- [x] Add Human path notation / overload candidate count limit tests.
- [x] Add Human path unsolved meta certificate rejection tests.

AI Speed Guard:

- [x] Do not set fixed wall-clock timing thresholds because they are prone to flakes.
- [x] Instead, fix same-input same-output, resource guards, and candidate count bounds.
- [x] Do not replace existing Phase 7 / Phase 9 fixtures with Human syntax.

Affected files:

```text
crates/npa-frontend/src/parser.rs
crates/npa-frontend/src/elaborator.rs
crates/npa-frontend/src/term_source.rs
crates/npa-api/src/phase7.rs
crates/npa-api/src/phase9.rs
scripts/phase9-regression.sh
```

Completion criteria:

```text
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- ./scripts/phase9-regression.sh
- Human implementation does not change Machine canonical term source hash
```

### P3H-14: Update Docs / README

Implementation tasks:

- [x] Separately describe the implementation status of Phase 3 Human and Phase 3 AI in `README.md`.
- [x] Update `develop/phase3-human.md` if the design and implemented scope diverge.
- [x] Preserve the Machine Surface fast-path non-regression conditions in `develop/phase3-ai.md`.
- [x] Check that docs for Phase 4 and later do not assume Human Surface in the AI path.

AI Speed Guard:

- [x] Do not add docs text implying that AI-facing candidate generation uses Human Surface.
- [x] In README / docs, limit the trusted boundary to the kernel / canonical certificate / independent checker.

Affected files:

```text
README.md
develop/phase3-human.md
develop/phase3-ai.md
develop/phase4-ai.md
develop/phase5-ai.md
develop/phase7-ai.md
```

Completion criteria:

```text
- Phase 3 Human is described as an untrusted convenience layer
- Phase 3 AI is described as the explicit fast path for Machine Surface
- No text reads as if the trusted base has expanded
```

---

## 4. Out Of Scope For The Phase 3 Human MVP

The Phase 3 Human MVP does not implement the following.

```text
full typeclass resolution
coercion search
instance search
macro system
user-defined syntax extension
multi-token / mixfix notation
tactic blocks
pattern matching elaboration
do notation
structure projection notation
term-level numeric literal overload
aliases
absolute global name syntax
source-level recursive definition syntax
reducibility annotations
termination checking
mutual declarations
automatic universe generalization
sophisticated universe minimization
SMT fallback
AI fallback from Human elaboration
unbounded proof search
```

These are convenient, but they expand the Human elaborator search space and
can easily harm the speed, determinism, and explainability of the AI-facing
Machine Surface. If they become necessary, design them as a separate profile
from Machine Surface.

---

## 5. Completion Decision

The Phase 3 Human implementation can be considered complete when the following
conditions hold.

```text
- Human source can handle import/open/namespace/end
- Human source can handle def/theorem/axiom/simple inductive
- grouped binder / arrow / implicit binder / holes can be handled
- notation declarations and simple infix can be handled
- name / notation ambiguity can become deterministic diagnostics
- implicit args and universe metas can be solved with simple unification
- unresolved holes / metas can be rejected before certificate generation
- solved Human declarations become fully explicit core AST
- only declarations that pass Phase 1 kernel check are accepted
- handoff to Phase 2 certificate can happen without adding Human metadata
- Machine Surface accepted / rejected syntax and canonical hash do not change
- Phase 4 / 5 / 7 / 8 / 9 AI automation preserves the Machine Surface fast path
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- ./scripts/phase9-regression.sh
```

In one sentence, Phase 3 Human is a human-facing syntax convenience layer. The
basis for accepting proofs remains limited to the canonical core AST, the Phase
1 Rust kernel, the Phase 2 certificate verifier, and the Phase 8 independent
checker.
