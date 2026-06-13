# Logic Model Theory Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T11`, `BMQ-006`)
- `proofs/set-theory-theorem-proof-roadmap-todo.md`
- `develop/core-spec-v0.1.md`
- `develop/overall-design.md`

This task breakdown is the first dedicated todo for mathematical logic, model
theory, computability, and proof theory as proof-corpus mathematics. It is a
planning sidecar only: it does not add trusted proof evidence, axioms,
source-free certificate verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers first-order syntax, substitution, free variables,
semantics, satisfaction, theories, structures, soundness and completeness
routes, compactness, Lowenheim-Skolem routes, elementary embeddings, types,
ultraproducts, definability, computability, recursion theory, proof theory,
type-theoretic semantics, and closure-boundary planning.

Out of scope for this task document:

- changing NPA's trusted kernel or treating proof-corpus metatheorems as
  kernel soundness evidence;
- adding satisfaction, models, choice, replacement, recursion, or
  completeness as trusted kernel primitives;
- hiding metatheoretic assumptions such as coding, substitution correctness,
  quotienting, countability, choice, or classical reasoning;
- duplicating set theory's forcing, large-cardinal, descriptive-set-theory, or
  model-relative independence ownership.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.LogicModelTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing lightweight logic modules include `Proofs.Ai.Logic.Iff` plus core
  proof-corpus modules such as `Proofs.Ai.Prop`, `Proofs.Ai.Eq`, and
  `Proofs.Ai.EqReasoning`.
- Set theory already owns quotient, choice, cardinal, forcing, constructible
  universe, large cardinal, and model-relative set-theoretic routes.
- This todo is for mathematics in the proof corpus, not for changing the
  trusted Rust kernel or canonical certificate checker.
- Any metatheorem about another proof system must identify the encoded syntax,
  semantics, derivability relation, and metatheory assumptions explicitly.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `LMT-00` inventory and namespace contract | `LMT-T00` |
| `LMT-01` first-order syntax and substitution | `LMT-T01` |
| `LMT-02` structures, assignments, and satisfaction | `LMT-T02` |
| `LMT-03` theories, models, and elementary equivalence | `LMT-T03` |
| `LMT-04` proof systems and soundness | `LMT-T04` |
| `LMT-05` completeness and compactness routes | `LMT-T05` |
| `LMT-06` Lowenheim-Skolem and elementary embeddings | `LMT-T06` |
| `LMT-07` ultraproducts and types | `LMT-T07` |
| `LMT-08` definability and quantifier elimination | `LMT-T08` |
| `LMT-09` computability and recursion theory | `LMT-T09` |
| `LMT-10` proof theory and incompleteness routes | `LMT-T10` |
| `LMT-11` type-theoretic semantics and categorical semantics | `LMT-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `LMT-T00` | `L0` planning, theorem-card inventory, and metatheory taxonomy |
| `LMT-T01` through `LMT-T04` | `L2` for structural syntax, substitution, semantics, and soundness lemmas |
| `LMT-T05` through `LMT-T08` | route packages or `L2` only when coding, choice, and model-existence prerequisites are explicit |
| `LMT-T09` through `LMT-T11` | split before source edits unless machine, coding, or semantic model is fixed |

## Milestones

### LMT-T00 Build Logic And Model Theory Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory syntax, semantics, soundness, completeness, compactness,
    Lowenheim-Skolem, ultraproduct, type, definability, computability, and
    proof-theory theorem families.
  - Assign primary homes between logic/model theory and set theory.
  - Record metatheory assumptions for every theorem card.
- Deliverables:
  - Logic/model theory theorem-card inventory and duplicate-home map.
- Acceptance criteria:
  - Kernel soundness and proof-corpus metatheorems are clearly separated.
- Verification:
  - `rg -n "LMT-T00|satisfaction|completeness|compactness|computability" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T01 Add First-Order Syntax Core

- Status: Pending
- Depends on: `SET-T06`
- Areas: `Proofs.Ai.LogicModelTheory.Syntax`
- Tasks:
  - Define signatures, variables, terms, formulas, free variables,
    substitution, renaming, and alpha-equivalence route packages.
  - Prove structural substitution lemmas where binding representation is
    explicit.
  - Split quotient or alpha-equivalence features behind set-theory support.
- Deliverables:
  - First-order syntax core module.
- Acceptance criteria:
  - Binding and substitution assumptions are structured data, not string
    processing.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.Syntax`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LogicModelTheory.Syntax --verified-cache authoring`

### LMT-T02 Add Semantics And Satisfaction Core

- Status: Pending
- Depends on: `LMT-T01`
- Areas: `Proofs.Ai.LogicModelTheory.Semantics`
- Tasks:
  - Define structures, interpretations, assignments, term evaluation, formula
    satisfaction, and satisfaction invariance under assignment agreement.
  - Keep semantic domains and equality assumptions explicit.
  - Prove elementary substitution-satisfaction lemmas.
- Deliverables:
  - Semantics and satisfaction module.
- Acceptance criteria:
  - Satisfaction is not treated as a kernel primitive.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.Semantics`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### LMT-T03 Add Theories, Models, And Elementary Equivalence

- Status: Pending
- Depends on: `LMT-T02`
- Areas: `Proofs.Ai.LogicModelTheory.Theory`
- Tasks:
  - Define theories, model satisfaction, entailment, elementary equivalence,
    elementary substructure, and embeddings.
  - Split elementary chains and Tarski-Vaught route prerequisites.
  - Coordinate set-sized model assumptions with set theory.
- Deliverables:
  - Theory and model core module.
- Acceptance criteria:
  - Model existence is never inferred from a theory without an explicit
    route package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.Theory`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.LogicModelTheory.Theory --verified-cache authoring`

### LMT-T04 Add Proof System And Soundness Route

- Status: Pending
- Depends on: `LMT-T02`
- Areas: `Proofs.Ai.LogicModelTheory.Soundness`
- Tasks:
  - Define a proof system, inference rules, derivability, valid formulas, and
    rule soundness packages.
  - Prove soundness by induction over derivations when the induction
    principle is available.
  - Keep this proof-corpus soundness separate from NPA kernel soundness.
- Deliverables:
  - Proof-system soundness route module.
- Acceptance criteria:
  - The theorem states which encoded proof system it proves sound.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.Soundness`
  - `rg -n "soundness|derivability|kernel" proofs/logic-model-theory-theorem-proof-roadmap-todo.md develop/core-spec-v0.1.md`

### LMT-T05 Add Completeness And Compactness Route

- Status: Pending
- Depends on: `LMT-T04`, `SET-T15`
- Areas: `Proofs.Ai.LogicModelTheory.Completeness`
- Tasks:
  - Define Henkin extensions, consistency, maximally consistent sets,
    canonical models, completeness, and compactness route packages.
  - Split countability, choice, coding, and Lindenbaum assumptions.
  - Prove finite satisfiability to compactness only after completeness route
    evidence exists.
- Deliverables:
  - Completeness and compactness dependency map.
- Acceptance criteria:
  - Compactness is not accepted as an axiom to prove completeness or vice
    versa without an explicit direction.
- Verification:
  - `rg -n "Henkin|completeness|compactness|Lindenbaum" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T06 Add Lowenheim-Skolem And Elementary Embedding Route

- Status: Pending
- Depends on: `LMT-T05`, `SET-T10`
- Areas: `Proofs.Ai.LogicModelTheory.LowenheimSkolem`
- Tasks:
  - Define Skolem functions, Skolem hulls, downward and upward
    Lowenheim-Skolem route packages.
  - State cardinality and choice assumptions explicitly.
  - Coordinate elementary substructure with `LMT-T03`.
- Deliverables:
  - Lowenheim-Skolem route module.
- Acceptance criteria:
  - Cardinal assumptions are visible and imported from set theory.
- Verification:
  - `rg -n "Skolem|Lowenheim|elementary substructure|cardinal" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T07 Add Ultraproduct And Type Route

- Status: Pending
- Depends on: `LMT-T03`, `SET-T30`
- Areas: `Proofs.Ai.LogicModelTheory.Ultraproduct`
- Tasks:
  - Define ultraproducts, Los theorem route packages, types, saturation, and
    omitting types route packages.
  - Import ultrafilter prerequisites from set theory.
  - Split saturation existence behind cardinal assumptions.
- Deliverables:
  - Ultraproduct and type route module.
- Acceptance criteria:
  - Ultrafilter and choice dependencies are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.LogicModelTheory.Ultraproduct`
  - `rg -n "ultraproduct|Los|type|saturation" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`

### LMT-T08 Add Definability And Quantifier Elimination Route

- Status: Pending
- Depends on: `LMT-T03`
- Areas: `Proofs.Ai.LogicModelTheory.Definability`
- Tasks:
  - Define definable sets, interpretations, elimination of quantifiers,
    model completeness, and stability route packages.
  - Add first examples only after their algebraic or order-theoretic
    structures exist.
  - Split application-specific results to algebra or number theory.
- Deliverables:
  - Definability route module.
- Acceptance criteria:
  - Quantifier elimination names include the structure and language.
- Verification:
  - `rg -n "definable|quantifier elimination|model complete|stability" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T09 Add Computability And Recursion Route

- Status: Pending
- Depends on: `LMT-T01`, `SET-T09`
- Areas: `Proofs.Ai.LogicModelTheory.Computability`
- Tasks:
  - Define computable functions, partial recursive functions, Turing machines,
    lambda-calculus encodings, reductions, and decidability route packages.
  - Choose a primary machine model before source edits.
  - Split equivalence of models and undecidability theorems.
- Deliverables:
  - Computability route module or model-selection note.
- Acceptance criteria:
  - The computation model and encoding are explicit in each theorem.
- Verification:
  - `rg -n "computable|recursive|Turing|decidable|reduction" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T10 Add Proof Theory And Incompleteness Route

- Status: Pending
- Depends on: `LMT-T04`, `LMT-T09`
- Areas: `Proofs.Ai.LogicModelTheory.ProofTheory`
- Tasks:
  - Define arithmetization, provability predicates, consistency statements,
    cut elimination route packages, and incompleteness theorem routes.
  - Split syntactic metatheory from semantic model theory.
  - Record arithmetic and coding prerequisites.
- Deliverables:
  - Proof-theory dependency map.
- Acceptance criteria:
  - Incompleteness statements name the represented theory and coding
    assumptions.
- Verification:
  - `rg -n "incompleteness|provability|cut elimination|arithmetization" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### LMT-T11 Add Type-Theoretic And Categorical Semantics Route

- Status: Pending
- Depends on: `LMT-T02`, `CAT-T06`
- Areas: `Proofs.Ai.LogicModelTheory.Semantics.Categorical`
- Tasks:
  - Define categorical semantics, syntactic categories, classifying toposes
    route packages, and type-theoretic model interfaces.
  - Coordinate generic categorical statements with category theory.
  - Keep semantics of NPA itself separate from proof-corpus examples.
- Deliverables:
  - Categorical semantics route module.
- Acceptance criteria:
  - Categorical semantics theorems state the logic and categorical structure
    they interpret.
- Verification:
  - `rg -n "categorical semantics|syntactic category|topos|type-theoretic" proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `LMTQ-001` | theorem-card inventory and metatheory taxonomy | `L0` | `LMT-T00` |
| `LMTQ-002` | first-order syntax core | `L2` | `LMT-T01` |
| `LMTQ-003` | satisfaction semantics core | `L2` | `LMT-T02` |
| `LMTQ-004` | theory and model definitions | `L2` | `LMT-T03` |
| `LMTQ-005` | encoded soundness route | `L2` where derivation induction exists | `LMT-T04` |
| `LMTQ-006` | completeness dependency split | route package first | `LMT-T05` |
| `LMTQ-007` | ultraproduct dependency split | route package first | `LMT-T07` |
| `LMTQ-008` | computability model selection | documentation or route module | `LMT-T09` |

## Review Checklist

- Proof-corpus metatheorems are not described as trusted kernel evidence.
- Syntax, substitution, semantics, and derivability are structured data.
- Choice, quotient, countability, and cardinal assumptions are visible.
- Set-theory-owned forcing and independence results are not duplicated here.
