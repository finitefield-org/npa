# Set Theory Theorem Proof Roadmap Todo

Source: `proofs/set-theory-theorem-proof-roadmap.md`

This document decomposes the set theory theorem proof roadmap into concrete
authoring milestones. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, kernel primitives, or certificate validity assumptions.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, source-free checker verdicts, and visible axiom or core
feature reports. Source files, replay files, metadata, theorem indexes, this
todo document, tactics, and AI output are untrusted.

---

## Scope

This task list covers theorem-card inventory, elementary set algebra,
relations, quotient interfaces, functions, finite and countable sets, cardinal
comparison, ordered sets, choice principles, ordinals, hierarchy and
foundation, cardinals and cofinality, ZF/ZFC/class-theory packages, paradox
boundaries, constructibility, CH/GCH and relative consistency cards, Boolean
algebras, model theory, forcing, inner models, large cardinals, descriptive set
theory, determinacy, set-theoretic topology, infinite combinatorics, and
cross-roadmap reuse.

The list intentionally does not prove the roadmap in one pass. Later agents
should implement exactly one milestone or a clearly bounded contiguous batch.
When prerequisites are absent, agents should split explicit blocker or prerequisite tasks before source edits. Statement-only interfaces are not acceptable proof artifacts for pending theorem work.

Out of scope for this task document:

- changing the Rust kernel, certificate format, checker profiles, or
  independent checker behavior;
- adding sets, classes, ordinals, cardinals, choice, ZF/ZFC, forcing,
  determinacy, large cardinals, model satisfaction, or theorem search as
  trusted kernel primitives;
- adding `unsafe` Rust, plugin loading, network calls, or AI calls to trusted
  code;
- using `Classical.choice`, `funext`, `propext`, unrestricted comprehension,
  or global choice as hidden conveniences;
- treating theorem-search sidecars, AI indexes, replay files, generated docs,
  or this todo document as trusted evidence;
- promoting unstable set theory modules into `npa-mathlib` before local
  closure, axiom-report, source-free, package, and public materialization
  checks are clean.

## Authoring Loop

For ordinary set theory theorem authoring, prefer local proof-corpus checks
before broad package gates:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --write-ai-index
./scripts/check-corpus-authoring.sh
```

Use `--build-module` before source-free `--module` checks when source changes
must be reflected in certificates. Reserve `check-corpus-package.sh` or
`check-corpus-full.sh` for package-wide verifier behavior, publish-plan or
package metadata updates, certificate/checker compatibility, release work, or
other high-trust closure work outside this TODO file.

If a milestone uses `quotient_v1`, verify with a quotient-capable checker
profile and confirm that the feature report exposes the quotient dependency.
If a theorem requires choice, ZFC, replacement, class comprehension, forcing,
determinacy, or a large-cardinal assumption, the assumption must be visible in
the theorem statement or imported law package.

## Current Implementation Facts

- Checked `Proofs.Ai.SetTheory.*` modules now exist for the elementary
  `SET-T01` through `SET-T16` foundation: `Basic`, `BooleanOps`, `Family`,
  `Relation`, `Equivalence`, `Quotient`, `Function`, `Image`, `Finite`,
  `Countable`, `Cardinal.Basic`, `Cardinal.Compare`, `Cardinal.Cantor`,
  `Cardinal.Arithmetic`, `Order.Poset`, `Order.Lattice`,
  `Order.WellFounded`, `Choice`, `Maximal`, and `Ultrafilter`.
- As of 2026-06-13, every set theory `SET-T*` item through `SET-T47`
  has a checked `L2` route certificate in the `Proofs.Ai.SetTheory.*`
  namespace.
- Existing reusable modules include `Proofs.Ai.Basic`, `Proofs.Ai.Eq`,
  `Proofs.Ai.EqReasoning`, `Proofs.Ai.Prop`, `Proofs.Ai.Nat`,
  `Proofs.Ai.Logic.Iff`, and checked algebra, geometry, vector, analysis,
  category, quotient-oriented, probability, topology, and set-theory modules
  under `Proofs.Ai.*`.
- `quotient_v1` and `Std.Quotient` exist behind an explicit feature/profile
  boundary. Quotient use is allowed only when that boundary is visible through
  feature/profile reports or explicit quotient law packages.
- Phase 6 MVP standard library policy excludes hidden custom axioms,
  `Classical.choice`, `funext`, and `propext`; set theory modules must keep
  those dependencies explicit if they are introduced later as law packages.
- Topology owns concrete topological consequences such as Urysohn, Tietze,
  Tychonoff, Stone-Cech, and Baire theorem routes. This set theory todo owns
  the set-theoretic choice, ultrafilter, Boolean prime ideal, and cardinal
  invariant route packages those routes may import.
- Measure theory owns concrete Borel/Radon/measure statements. This set
  theory todo owns descriptive-set-theoretic definability, analytic/coanalytic
  statement forms, and regularity-principle interfaces.
- Advanced model-relative results such as forcing preservation, CH
  independence, constructibility, core models, determinacy, and large-cardinal
  consistency strength remain `L2/L3` theorem cards until structured syntax,
  satisfaction, and metatheory support exist.

## Roadmap Coverage Map

| Roadmap milestone | Covered by task milestones |
| --- | --- |
| `SET-00` inventory, naming, and axiom taxonomy | `SET-T00` |
| `SET-01` elementary set algebra | `SET-T01` through `SET-T03` |
| `SET-02` relations and quotients | `SET-T04` through `SET-T05` |
| `SET-03` functions, images, and inverse images | `SET-T06` through `SET-T07` |
| `SET-04` finite, countable, and enumerable sets | `SET-T08` through `SET-T09` |
| `SET-05` cardinal comparison and Cantor theorems | `SET-T10` through `SET-T12` |
| `SET-06` ordered sets, lattices, and well-founded orders | `SET-T13` through `SET-T14` |
| `SET-07` choice principles and equivalents | `SET-T15` through `SET-T16` |
| `SET-08` ordinals and transfinite methods | `SET-T17` through `SET-T18` |
| `SET-09` cumulative hierarchy and foundation | `SET-T19` through `SET-T20` |
| `SET-10` cardinals, cofinality, regularity, stationary sets | `SET-T21` through `SET-T22` |
| `SET-11` ZF/ZFC, class theories, and paradox boundaries | `SET-T23` through `SET-T24` |
| `SET-12` constructible universe and absoluteness | `SET-T25` through `SET-T26` |
| `SET-13` CH, GCH, continuum function, and independence | `SET-T27` through `SET-T28` |
| `SET-14` forcing and Boolean-valued models | `SET-T33` through `SET-T34` |
| `SET-15` inner models and core-model interfaces | `SET-T35` through `SET-T36` |
| `SET-16` large cardinals and ultrapowers | `SET-T37` through `SET-T38` |
| `SET-17` descriptive set theory and Polish interfaces | `SET-T39` through `SET-T40` |
| `SET-18` determinacy and regularity consequences | `SET-T41` through `SET-T42` |
| `SET-19` set-theoretic topology interfaces | `SET-T43` through `SET-T44` |
| `SET-20` infinite combinatorics and partition calculus | `SET-T45` through `SET-T46` |
| `SET-21` Boolean algebras, ultrafilters, and Stone duality | `SET-T29` through `SET-T30` |
| `SET-22` model theory interfaces | `SET-T31` through `SET-T32` |
| `SET-23` cross-roadmap reuse | `SET-T47` |

## Recommended Queue Coverage

| Queue ID | Task milestones |
| --- | --- |
| `SEQ-001` | `SET-T00` |
| `SEQ-002` | `SET-T01` |
| `SEQ-003` | `SET-T02` |
| `SEQ-004` | `SET-T03` |
| `SEQ-005` | `SET-T04` |
| `SEQ-006` | `SET-T05` |
| `SEQ-007` | `SET-T06` |
| `SEQ-008` | `SET-T07` |
| `SEQ-009` | `SET-T08` |
| `SEQ-010` | `SET-T09` |
| `SEQ-011` | `SET-T10` |
| `SEQ-012` | `SET-T11` |
| `SEQ-013` | `SET-T12` |
| `SEQ-014` | `SET-T13`, `SET-T14` |
| `SEQ-015` | `SET-T15`, `SET-T16` |
| `SEQ-016` | `SET-T17`, `SET-T18` |
| `SEQ-017` | `SET-T19`, `SET-T20` |
| `SEQ-018` | `SET-T21`, `SET-T22` |
| `SEQ-019` | `SET-T23`, `SET-T24` |
| `SEQ-020` | `SET-T29` |

After `SEQ-020`, choose the next branch by project need:

- topology reuse: finish selected `SET-T39` and `SET-T40` interfaces plus
  topology prerequisites, then `SET-T43`; run `SET-T44` only after `SET-T30`
  and `SET-T34` are available;
- CH/forcing: `SET-T31`, `SET-T32`, `SET-T25`, `SET-T26`, `SET-T30`,
  `SET-T33`, `SET-T34`, then `SET-T27` and `SET-T28`;
- descriptive set theory: topology and measure prerequisites, then
  `SET-T39` and `SET-T40`;
- large cardinals and determinacy: `SET-T31`, `SET-T32`, `SET-T37`,
  `SET-T38`, `SET-T39`, `SET-T40`, then `SET-T41` and `SET-T42`;
## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `SET-T00` | `L0` planning, theorem-card inventory, duplicate map, and axiom taxonomy |
| `SET-T01` through `SET-T14` | target `L2` derived foundation lemmas where possible; keep extensionality, quotient, function extensionality, countable choice, and classical reasoning as explicit prerequisites rather than interface theorem landings |
| `SET-T15` through `SET-T18` | target `L2` direction-specific equivalences from explicit assumptions; split missing law-package prerequisites before source edits |
| `SET-T19` through `SET-T24` | target `L2` hierarchy, foundation, cardinal, ZF/ZFC, class, and paradox certificates where metatheory support exists; otherwise split visible replacement, foundation, class-comprehension, and choice blockers |
| `SET-T25` through `SET-T28` | target `L2/L3` model-relative theorem cards until structured syntax, satisfaction, constructibility, and forcing support exist |
| `SET-T29` through `SET-T34` | target `L2` Boolean, model-theory, and forcing certificates where metatheory support exists; keep truth lemma, generic extension, and preservation as roadmap blockers until metatheory lands |
| `SET-T35` through `SET-T42` | target `L2/L3` theorem cards for inner models, large cardinals, descriptive set theory, and determinacy, with basic vocabulary modules split out where feasible |
| `SET-T43` through `SET-T46` | target `L2` cross-roadmap certificates for topology and infinite combinatorics; independence-sensitive claims remain model-relative blockers until their assumptions are explicit |
| `SET-T47` | target `L2` cross-roadmap index and prelude route certificates |

For any milestone that contains more than one theorem family, the first task is
to split the module or theorem batch further if one implementation turn cannot
reasonably build, source-free verify, and review the whole milestone without
guessing. The split must preserve the dependency order in this document.

---

## Milestones

Task IDs are stable topic IDs, not a promise that the sections below can always
be implemented in numeric order. Use each task's `Depends on` line and the
recommended queue above as the implementation order. This matters for
constructibility, CH independence, Boolean algebras, model theory, forcing,
inner models, and large cardinals, where lower-numbered topic tasks may wait
for higher-numbered support tasks.

### SET-T00 Build Theorem Card Inventory And Axiom Taxonomy

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: None
- Areas: `Proofs.Ai.SetTheory.Index`, `Proofs.Ai.SetTheory.Axioms`, `Proofs.Ai.SetTheory.TheoremCards`, `proofs/README.md`
- Tasks:
  - Normalize every pasted theorem name into a stable ASCII theorem-card identifier.
  - Tag each theorem as elementary, classical, choice-dependent, quotient-dependent, replacement-dependent, class-theoretic, model-relative, independence-sensitive, determinacy-dependent, or large-cardinal.
  - Record duplicate-home decisions for topology, measure, analysis, algebra, statistics, combinatorics, and model theory overlaps.
  - Define initial axiom package names for set extensionality, classical excluded middle, quotient lift, choice, ZF/ZFC, class comprehension, forcing extension, determinacy, and large cardinals.
- Deliverables:
  - A set theory theorem-card inventory and axiom taxonomy that later tasks can cite.
- Acceptance criteria:
  - Every roadmap milestone `SET-00` through `SET-23` has at least one theorem card or intentionally grouped card.
  - No theorem card claims a hidden kernel extension or treats this todo document as proof evidence.
  - Cross-roadmap theorem ownership is explicit for choice, topology, measure, algebra, and DST overlaps.
- Verification:
  - `rg -n "SET-00|SET-23|choice|forcing|large-cardinal|sidecar" proofs/set-theory-theorem-proof-roadmap*.md proofs/README.md`
  - `git diff --check`

### SET-T01 Create Set Representation And Subset Equality Layer

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T00`
- Areas: `Proofs.Ai.SetTheory.Basic`, `Proofs.Ai.SetTheory.Subset`
- Tasks:
  - Choose the first-pass set representation for elementary modules, favoring predicate-over-carrier unless the implementation milestone records a stronger reason.
  - Define membership, subset, extensional equality, empty set, relative universal set, singleton, pair, and powerset statement forms.
  - Prove subset reflexivity, transitivity, and antisymmetry under explicit extensionality evidence.
- Deliverables:
  - First checked elementary set module with subset and equality APIs.
- Acceptance criteria:
  - Extensionality is an explicit law package or theorem premise.
  - No unrestricted comprehension or ZF object layer is required for elementary subset proofs.
  - Names distinguish set equality from logical equivalence of membership.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Basic --verified-cache authoring`

### SET-T02 Add Boolean Operations And Elementary Set Laws

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T01`
- Areas: `Proofs.Ai.SetTheory.BooleanOps`
- Tasks:
  - Define union, intersection, difference, complement relative to a carrier, and Boolean-algebra-style subset operations.
  - Prove empty, total, idempotent, commutative, associative, absorption, distributive, and double-complement laws.
  - Prove De Morgan laws and monotonicity of union/intersection.
- Deliverables:
  - Boolean operation theorem batch reusable by topology and measure roadmaps.
  - Current coverage in `Proofs.Ai.SetTheory.BooleanOps` now includes the
    right-sided absorption rewrites
    `set_intersection_absorption_union_right_extensional` and
    `set_union_absorption_intersection_right_extensional`.
  - Current coverage in `Proofs.Ai.SetTheory.BooleanOps` now also includes
    right-sided distributivity rewrites
    `set_intersection_distributes_union_right_extensional` and
    `set_union_distributes_intersection_right_extensional`.
- Acceptance criteria:
  - Complement laws state the ambient carrier explicitly.
  - Theorems use only structural or explicit extensionality assumptions.
  - Rewrite names are stable and do not rely on global ZF packages.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.BooleanOps`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.BooleanOps --verified-cache authoring`

### SET-T03 Add Indexed Operations, Products, And Powerset Monotonicity

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T02`
- Areas: `Proofs.Ai.SetTheory.Basic`, `Proofs.Ai.SetTheory.Family`
- Tasks:
  - Add indexed union, indexed intersection, product-set, and powerset monotonicity statements.
  - Prove subset criteria for indexed union/intersection and elementary product laws.
  - Record which indexed operations require only a family predicate and which require a richer set object layer.
- Deliverables:
  - Indexed set operation API for relations, functions, topology, and measure.
  - Current coverage in `Proofs.Ai.SetTheory.Family` now includes selected-cover
    extensionality for indexed unions and indexed intersections via
    `indexed_union_selected_cover_extensional` and
    `indexed_intersection_selected_cover_extensional`.
  - Current coverage in `Proofs.Ai.SetTheory.Family` now also includes
    product-set coordinate slice extensionality via
    `product_set_right_slice_extensional` and
    `product_set_left_slice_extensional`.
- Acceptance criteria:
  - No theorem silently assumes replacement, choice, or unrestricted indexed comprehension.
  - Product and indexed operation theorem names can be imported by relation and topology modules without redefinition.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Family`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Family --verified-cache authoring`

### SET-T04 Add Relation Algebra Foundations

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T03`
- Areas: `Proofs.Ai.SetTheory.Relation`
- Tasks:
  - Define binary relations as predicates or subsets over products.
  - Add domain, range, inverse relation, composition, identity relation, and restriction.
  - Prove basic composition, inverse, restriction, and identity laws.
  - Define reflexive, symmetric, antisymmetric, transitive, total, functional, injective-relation, and well-founded predicates.
- Deliverables:
  - Relation algebra module for orders, functions, equivalence relations, and model theory.
  - Current coverage in `Proofs.Ai.SetTheory.Relation` now includes relation
    extensional equality `refl`/`symm`/`trans` via
    `relation_extensional_equality_refl`,
    `relation_extensional_equality_symm`, and
    `relation_extensional_equality_trans`.
  - Current coverage in `Proofs.Ai.SetTheory.Relation` also includes inverse
    involution, composition monotonicity, associativity, identity laws, and
    relation-composition congruence via
    `relation_composition_congruent_extensional`.
- Acceptance criteria:
  - Relation laws reuse set/product infrastructure instead of restating membership arguments.
  - Well-foundedness is evidence, not an implicit recursion primitive.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Relation`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Relation --verified-cache authoring`

### SET-T05 Add Equivalence, Partition, And Quotient Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T04`
- Areas: `Proofs.Ai.SetTheory.Equivalence`, `Proofs.Ai.SetTheory.Quotient`
- Tasks:
  - Define equivalence relations, equivalence classes, induced partitions, and partition-to-equivalence construction.
  - Prove class membership, class equality, disjointness of distinct classes, equivalence-to-partition, and partition-to-equivalence directions.
  - Add quotient map and quotient universal-property statement forms over `quotient_v1` or an explicit quotient law package.
- Deliverables:
  - Equivalence/partition theorem batch and quotient interface.
  - Current coverage in `Proofs.Ai.SetTheory.Equivalence` now includes
    `SameEquivalenceClass` `refl`/`symm`/`trans` via
    `same_equivalence_class_refl`, `same_equivalence_class_symm`, and
    `same_equivalence_class_trans`.
  - Current coverage in `Proofs.Ai.SetTheory.Quotient` now exposes the
    `quotient_v1` profile and quotient-map related equality directly from the
    universal property via
    `set_quotient_v1_universal_property_feature_profile_visible` and
    `set_quotient_v1_universal_property_related_equal`.
- Acceptance criteria:
  - Quotient use is visible through a feature/profile report or explicit quotient package.
  - The quotient universal property is not stated as an unchecked axiom equal to the target theorem.
  - Equivalence and partition conversions are proved in both directions where prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Equivalence`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Quotient`
  - `rg -n "quotient_v1|Std.Quotient|SET-T05" proofs develop`

### SET-T06 Add Function Basics And Inverse Characterizations

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T04`
- Areas: `Proofs.Ai.SetTheory.Function`
- Tasks:
  - Define function records or functional relations with domain, codomain, graph, and application law.
  - Add identity, constant, restriction, extension by agreement, and composition.
  - Define injective, surjective, bijective, left inverse, right inverse, and inverse function.
  - Prove direction-specific inverse characterizations, separating constructive inverses from choice-based right inverses.
- Deliverables:
  - Function API needed by cardinality, topology, measure, and algebra roadmaps.
- Acceptance criteria:
  - Function equality records whether pointwise equality uses an explicit function-extensionality package.
  - Surjection-to-right-inverse theorems are not proved without choice or construction evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Function`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Function --verified-cache authoring`

### SET-T07 Add Image, Preimage, And Indexed Family Laws

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T06`
- Areas: `Proofs.Ai.SetTheory.Image`, `Proofs.Ai.SetTheory.Family`
- Tasks:
  - Define direct image, inverse image, image of a family, and dependent product statement forms.
  - Prove image/preimage laws for union, intersection, difference, complement, and indexed families.
  - Prove inverse-image Boolean-homomorphism laws and direct-image monotonicity with correct inclusion weakening.
  - Add choice-function statement forms without proving choice.
- Deliverables:
  - Image and preimage theorem batch for continuity, measurability, and cardinality routes.
- Acceptance criteria:
  - Preimage laws are available before topology closed-set and continuity milestones use them.
  - Direct image intersection laws state inclusion or injectivity hypotheses precisely.
  - Choice-function statements do not import global choice.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Image`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Image --verified-cache authoring`

### SET-T08 Add Finite Sets And Pigeonhole

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T06`
- Areas: `Proofs.Ai.SetTheory.Finite`
- Tasks:
  - Define finite sets via bijection with initial natural segments and, where useful, inductive finite closure.
  - Prove empty, singleton, insertion, deletion, finite union, finite product, finite image, and finite subset laws.
  - Prove finite pigeonhole principle.
- Deliverables:
  - Constructive finite-set theorem batch.
- Acceptance criteria:
  - Finite-cardinality and pigeonhole theorems do not use choice.
  - Finite definitions identify how they connect to `Proofs.Ai.Nat`.
  - Finite theorem names are reusable by statistics and combinatorics roadmaps.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Finite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Finite --verified-cache authoring`

### SET-T09 Add Countable And Enumerable Set Infrastructure

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T08`
- Areas: `Proofs.Ai.SetTheory.Countable`, `Proofs.Ai.SetTheory.Enumeration`
- Tasks:
  - Define countable, countably infinite, enumerable, and at-most-countable.
  - Prove `Nat` countability, finite-to-countable, finite sequences over countable sets, and selected `Int`/`Rat` countability interfaces.
  - Add countable union of finite sets and countable union of countable sets, splitting constructive and countable-choice variants.
- Deliverables:
  - Countability and enumeration API for analysis, topology, statistics, syntax coding, and DST.
- Acceptance criteria:
  - Every theorem states the representation of enumerability.
  - Countable-union theorems record whether countable choice is required.
  - Subset-of-countable results state whether the subset is decidable, enumerable, or choice-backed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Countable`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Countable --verified-cache authoring`

### SET-T10 Add Equipotence And Cardinal Comparison

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T07`, `SET-T09`
- Areas: `Proofs.Ai.SetTheory.Cardinal.Basic`, `Proofs.Ai.SetTheory.Cardinal.Compare`
- Tasks:
  - Define equipotence and cardinal inequality via injections and surjections.
  - Prove equipotence is an equivalence relation.
  - Prove injection/surjection comparison lemmas and transport of finite/countable/cardinal properties across bijections.
  - Keep quotient cardinal representatives optional until the representative policy is explicit.
- Deliverables:
  - Cardinal comparison foundation independent of global cardinal quotients.
- Acceptance criteria:
  - Theorems do not conflate representatives with quotient cardinals.
  - Comparison lemmas state whether choice or well-ordering is used.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Basic`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Compare`

### SET-T11 Prove Cantor And Diagonal Theorems

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T10`
- Areas: `Proofs.Ai.SetTheory.Cardinal.Cantor`
- Tasks:
  - Prove Cantor theorem: no surjection from a set onto its powerset.
  - Add diagonal uncountability for `Nat -> Bool` and powerset of `Nat`.
  - Add real-number uncountability theorem cards that wait for the chosen real representation.
- Deliverables:
  - Cantor and diagonal theorem batch.
- Acceptance criteria:
  - Cantor theorem is proved without choice.
  - The diagonal proof pattern is reusable by computability, analysis, and DST modules.
  - Real uncountability cards do not assume an unavailable real-number implementation.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Cantor`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Cardinal.Cantor --verified-cache authoring`

### SET-T12 Add Cantor-Bernstein And Cardinal Arithmetic Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T11`
- Areas: `Proofs.Ai.SetTheory.Cardinal.Compare`, `Proofs.Ai.SetTheory.Cardinal.Arithmetic`
- Tasks:
  - Prove Cantor-Bernstein-Schroeder or land a split theorem-card route if prerequisites are missing.
  - Add cardinal arithmetic statement forms for sums, products, function spaces, and powersets.
  - Add Hartogs theorem as an interface until ordinal infrastructure is sufficient.
- Deliverables:
  - Cardinal comparison and arithmetic theorem batch.
- Acceptance criteria:
  - Cantor-Bernstein dependencies are isolated for audit.
  - Cardinal arithmetic theorems state representative, quotient, choice, and well-ordering assumptions.
  - Hartogs remains an interface until `SET-T17` and `SET-T18` can support the proof route.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Compare`
  - `rg -n "Cantor-Bernstein|Hartogs|cardinal arithmetic" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T13 Add Posets, Linear Orders, And Order Isomorphisms

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T04`, `SET-T10`
- Areas: `Proofs.Ai.SetTheory.Order.Poset`
- Tasks:
  - Define preorder, partial order, linear order, strict order, chain, antichain, upper/lower bound, maximum, and minimum.
  - Add dual order, suborder, product order, lexicographic order, and pointwise order statements.
  - Prove basic order-isomorphism, initial-segment, and dense-order interface lemmas.
- Deliverables:
  - Ordered-set API for ordinals, Zorn-style arguments, topology, and combinatorics.
- Acceptance criteria:
  - Order proofs reuse relation infrastructure.
  - Maximal-principle theorems requiring choice are deferred to `SET-T15` and `SET-T16`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Order.Poset`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Order.Poset --verified-cache authoring`

### SET-T14 Add Lattices, Fixed Points, And Well-Founded Induction

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T13`
- Areas: `Proofs.Ai.SetTheory.Order.Lattice`, `Proofs.Ai.SetTheory.Order.WellFounded`
- Tasks:
  - Define supremum, infimum, complete lattices, monotone maps, and well-founded relations.
  - Prove complete-lattice basics and Knaster-Tarski fixed point theorem when prerequisites are sufficient.
  - Add well-founded induction and recursion interfaces over explicit well-founded evidence.
- Deliverables:
  - Lattice and well-founded APIs for closure operators, ordinals, fixed points, and recursive constructions.
- Acceptance criteria:
  - Well-founded recursion remains explicit if the corpus lacks recursion infrastructure.
  - Knaster-Tarski does not assume a complete lattice without evidence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Order.Lattice`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Order.WellFounded`

### SET-T15 Add Choice, Well-Ordering, And Zorn Equivalence Web

- Status: Completed
- Depends on: `SET-T14`
- Areas: `Proofs.Ai.SetTheory.Choice`, `Proofs.Ai.SetTheory.Maximal`
- Tasks:
  - Define choice function, countable choice, dependent choice, well-ordering theorem, Zorn's lemma, and Hausdorff maximal principle statement forms.
  - Prove direction-specific equivalence routes where assumptions are explicit.
  - Add theorem cards for domain-specific consequences such as vector-space bases, maximal ideals, and compact product routes without owning the domain proofs.
- Deliverables:
  - Explicit choice package and equivalence web.
  - Completed: Verified `Proofs.Ai.SetTheory.Choice` provides explicit
    choice-function, dependent-choice, well-ordering, Zorn, and Hausdorff
    maximal-principle law packages, plus named direction routes including
    `choice_equivalence_choice_to_zorn`,
    `choice_equivalence_well_ordering_to_hausdorff`, and
    `choice_equivalence_choice_to_hausdorff`.
- Acceptance criteria:
  - No choice theorem is treated as kernel-trusted.
  - Each equivalence direction identifies its assumptions and result strength.
  - Downstream roadmaps can import named choice principles instead of restating global choice.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Choice`
  - `rg -n "Classical.choice|Zorn|well-ordering|choice" proofs/set-theory-theorem-proof-roadmap*.md proofs`
  - Completed: `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Choice --verified-cache authoring`

### SET-T16 Add Maximal Principles, Ultrafilter Lemma, And Boolean Prime Ideal Interface

- Status: Completed
- Depends on: `SET-T15`
- Areas: `Proofs.Ai.SetTheory.Maximal`, `Proofs.Ai.SetTheory.Ultrafilter`
- Tasks:
  - Add Tukey lemma, finite-character maximal principle, ultrafilter lemma, and Boolean prime ideal theorem statement forms.
  - Prove or card equivalences among ultrafilter lemma, Boolean prime ideal theorem, and selected weaker choice principles.
  - Provide import contracts for topology, algebra, forcing, and Boolean algebra milestones.
- Deliverables:
  - Maximal-principle and ultrafilter choice-strength interface.
  - Completed: Verified `Proofs.Ai.SetTheory.Maximal` and
    `Proofs.Ai.SetTheory.Ultrafilter` provide finite-character, Tukey, Zorn,
    Hausdorff maximal-principle, filter, ultrafilter, ultrafilter-lemma,
    Boolean-prime-ideal, and Tukey/ultrafilter/Boolean-prime route packages.
    Choice strength and law dependencies are exposed through named projections
    such as `zorn_lemma_route_choice_law`, `tukey_lemma_route_law`,
    `ultrafilter_lemma_interface_choice_law`,
    `boolean_prime_ideal_interface_law`,
    `ultrafilter_prime_ideal_route_tukey_to_ultrafilter`, and
    `ultrafilter_prime_ideal_route_ultrafilter_to_tukey`.
- Acceptance criteria:
  - Choice strength is visible in theorem names, statements, or module headers.
  - Interfaces do not assert topology or algebra target conclusions directly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Ultrafilter`
  - `rg -n "ultrafilter lemma|Boolean prime ideal|Tukey|finite-character" proofs`
  - Completed: `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Maximal --verified-cache authoring`
  - Completed: `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Ultrafilter --verified-cache authoring`

### SET-T17 Add Ordinal Basics

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T13`, `SET-T15`
- Areas: `Proofs.Ai.SetTheory.Ordinal.Basic`
- Tasks:
  - Define transitive set, ordinal predicate, zero ordinal, successor ordinal, limit ordinal, and ordinal membership order.
  - Prove that ordinals are well-ordered by membership, every element of an ordinal is an ordinal, and ordinal comparison is transitive.
  - Add trichotomy as derived proof or explicitly label needed assumptions.
- Deliverables:
  - Basic ordinal theorem batch.
- Acceptance criteria:
  - Ordinal comparison does not assume global choice unless the statement says so.
  - Ordinal facts are separated from class-level claims about all ordinals.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Ordinal.Basic`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.Ordinal.Basic --verified-cache authoring`

### SET-T18 Add Transfinite Induction, Recursion, Supremum, And Burali-Forti Boundary

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T17`
- Areas: `Proofs.Ai.SetTheory.Ordinal.Induction`, `Proofs.Ai.SetTheory.Ordinal.Recursion`
- Tasks:
  - Prove transfinite induction over ordinals.
  - Add transfinite recursion as a theorem or explicit recursion package interface.
  - Define ordinal supremum and union of ordinals.
  - Add Burali-Forti boundary as a class/set nonexistence or contradiction theorem card.
- Deliverables:
  - Transfinite method API for hierarchy and cardinal modules.
- Acceptance criteria:
  - Recursion theorem states the precise replacement or recursion principle it uses.
  - Burali-Forti is not encoded via unrestricted comprehension inside the trusted core.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Ordinal.Induction`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Ordinal.Recursion`

### SET-T19 Add Cumulative Hierarchy, Rank, And Foundation Consequences

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T18`
- Areas: `Proofs.Ai.SetTheory.Hierarchy`, `Proofs.Ai.SetTheory.Foundation`, `Proofs.Ai.SetTheory.Rank`
- Tasks:
  - Define cumulative hierarchy levels, hierarchy monotonicity, and transitivity of appropriate levels.
  - Add rank theorem statements under explicit foundation/replacement assumptions.
  - Prove foundation consequences such as no membership cycles and membership induction where prerequisites exist.
- Deliverables:
  - Hierarchy, rank, and foundation vocabulary for ZFC, constructibility, forcing, and large cardinals.
- Acceptance criteria:
  - Foundation-dependent results are not imported into elementary set algebra.
  - Rank and hierarchy construction reports replacement-like dependencies.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Hierarchy`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Foundation`

### SET-T20 Add Collapse And Reflection Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T19`, `SET-T05`
- Areas: `Proofs.Ai.SetTheory.Hierarchy`, `Proofs.Ai.SetTheory.Foundation`
- Tasks:
  - Add Mostowski collapse theorem statement over extensional well-founded relations.
  - Add reflection principle statement forms.
  - Record relation, quotient, recursion, replacement, and satisfaction prerequisites before any proof work starts.
- Deliverables:
  - Collapse and reflection theorem cards with explicit blockers.
- Acceptance criteria:
  - Collapse remains `L2/L3` until relation, quotient, and recursion support is sufficient.
  - Reflection claims are model-relative and do not imply unrestricted truth predicates.
- Verification:
  - `rg -n "Mostowski|collapse|reflection|SET-T20" proofs/set-theory-theorem-proof-roadmap*.md proofs`
  - `git diff --check`

### SET-T21 Add Cardinal Representatives, Aleph, Beth, And Cardinal Arithmetic

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T12`, `SET-T18`
- Areas: `Proofs.Ai.SetTheory.Cardinal.Ordinal`
- Tasks:
  - Define initial ordinal/cardinal representatives, cardinal successor, aleph sequence, beth sequence, and continuum cardinal statement forms.
  - Add infinite sum/product/power theorem cards or proofs with explicit choice and well-ordering assumptions.
  - Add Konig theorem as a cardinal-arithmetic theorem card or proof with hypotheses.
- Deliverables:
  - Ordinal-cardinal representative policy and initial cardinal arithmetic API.
- Acceptance criteria:
  - Representative choices are explicit.
  - No theorem treats all cardinal comparisons as well-ordered without a stated well-ordering or choice dependency.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Ordinal`
  - `rg -n "aleph|beth|Konig|continuum" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T22 Add Cofinality, Regularity, Club, Stationary, And Fodor

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T21`
- Areas: `Proofs.Ai.SetTheory.Cardinal.Cofinality`, `Proofs.Ai.SetTheory.Stationary`
- Tasks:
  - Define cofinality, regular cardinal, singular cardinal, club subsets, stationary subsets, and closed unbounded filter basics.
  - Prove elementary cofinality and regularity facts.
  - Add Fodor pressing-down lemma and Delta-system lemma proof routes or theorem cards with prerequisites.
- Deliverables:
  - Cofinality, regularity, club, stationary, and first infinite-combinatorics APIs.
- Acceptance criteria:
  - Fodor and stationary results state regularity and uncountability hypotheses clearly.
  - Delta-system ownership is coordinated with `SET-T45` and `SET-T46`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Cardinal.Cofinality`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Stationary`

### SET-T23 Add ZF, ZFC, And Class-Theory Axiom Packages

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T01` through `SET-T22`
- Areas: `Proofs.Ai.SetTheory.ZF`, `Proofs.Ai.SetTheory.ZFC`, `Proofs.Ai.SetTheory.Class`
- Tasks:
  - Define ZF axiom package, ZFC as ZF plus choice, and NBG/MK class-theory interface packages.
  - Add global choice as a separate class-theoretic principle.
  - Add separation and replacement consequence statement forms used by hierarchy, ordinals, cardinals, and function-set constructions.
- Deliverables:
  - Named set/class axiom packages for later modules.
- Acceptance criteria:
  - ZF/ZFC packages are explicit assumptions, not trusted Rust kernel changes.
  - Global choice is separate from ordinary ZFC unless a theorem states otherwise.
  - Axiom-report expectations are documented for each package.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.ZF`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.ZFC`

### SET-T24 Add Paradox Boundaries And Low-Level Axiom Examples

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T23`
- Areas: `Proofs.Ai.SetTheory.Paradox`, `Proofs.Ai.SetTheory.Class`
- Tasks:
  - Add Russell paradox for unrestricted comprehension.
  - Add Cantor paradox boundary for a set of all sets.
  - Reuse Burali-Forti boundary from ordinal work and connect it to class/set separation.
  - Add low-level examples showing which elementary laws require only extensionality and which require richer ZF packages.
- Deliverables:
  - Paradox boundary theorem cards or checked contradiction theorems.
- Acceptance criteria:
  - No paradox theorem introduces a universal set as an ordinary ZF-style set.
  - Each paradox theorem is phrased as contradiction from an unrestricted principle or nonexistence under ZF-like assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Paradox`
  - `rg -n "Russell|Cantor paradox|Burali-Forti|unrestricted comprehension" proofs`

### SET-T25 Add Constructible Hierarchy Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T18`, `SET-T23`, `SET-T31`
- Areas: `Proofs.Ai.SetTheory.Constructible`
- Tasks:
  - Define first-order definability interface for set structures, reusing structured syntax from model theory once available.
  - Define constructible hierarchy levels and prove or card monotonicity and transitivity of `L_alpha`.
  - Add `L` as an inner-model interface.
- Deliverables:
  - Constructible hierarchy vocabulary and basic theorem cards.
- Acceptance criteria:
  - Definability is not encoded as fragile ad hoc strings.
  - All satisfaction and absoluteness assumptions are model-relative.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Constructible`
  - `rg -n "Constructible|L_alpha|definability|satisfaction" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T26 Add Absoluteness, V Equals L, And Godel Constructibility Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T25`, `SET-T32`
- Areas: `Proofs.Ai.SetTheory.Absoluteness`, `Proofs.Ai.SetTheory.InnerModel`
- Tasks:
  - Add Godel constructibility theorem card: `L` satisfies ZFC under suitable ambient assumptions.
  - Add `V = L` global well-ordering, CH, and GCH consequences as model-relative theorem cards.
  - Add Shoenfield absoluteness, Levy absoluteness, and condensation lemma interfaces.
- Deliverables:
  - Constructibility and absoluteness theorem-card batch.
- Acceptance criteria:
  - No constructibility theorem claims an absolute proof of CH in bare ZFC.
  - Model and satisfaction assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Absoluteness`
  - `rg -n "V = L|Godel|Shoenfield|condensation|CH" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T27 Add CH, GCH, And Continuum Statement Modules

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T21`, `SET-T26`
- Areas: `Proofs.Ai.SetTheory.Continuum`
- Tasks:
  - Define CH, GCH, continuum cardinal, continuum function, and equivalent CH forms over powerset of `Nat`.
  - Add real-cardinality variants as theorem cards pending real-number infrastructure.
  - Record `V = L` consequences by importing constructibility cards instead of proving CH from bare ZFC.
- Deliverables:
  - Canonical CH/GCH/continuum statement module.
- Acceptance criteria:
  - No theorem states that CH or not-CH is proved from bare ZFC.
  - Equivalent forms identify the representation of reals, subsets of `Nat`, or cardinal exponentiation used.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Continuum`
  - `rg -n "continuum hypothesis|GCH|2\\^aleph_0|not-CH" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T28 Add Relative Consistency And Independence Theorem Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T27`, `SET-T34`
- Areas: `Proofs.Ai.SetTheory.Independence`, `Proofs.Ai.SetTheory.RelativeConsistency`
- Tasks:
  - Add Godel relative consistency card for ZFC plus GCH via constructibility.
  - Add Cohen independence card for ZFC plus not-CH via forcing.
  - Add Easton theorem, Martin's axiom plus not-CH, and Suslin hypothesis independence cards.
- Deliverables:
  - Relative-consistency and independence theorem-card batch.
- Acceptance criteria:
  - Every consistency claim is model-relative or explicitly conditional.
  - Forcing-backed cards import `SET-T33` and `SET-T34` rather than duplicating forcing assumptions.
  - Independence-sensitive statements remain `L2/L3` until metatheory support exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Independence`
  - `rg -n "relative consistency|Cohen|Easton|Martin|Suslin" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T29 Add Boolean Algebra, Ideals, Filters, And Complete Boolean Algebra Basics

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T02`, `SET-T16`
- Areas: `Proofs.Ai.SetTheory.BooleanAlgebra`, `Proofs.Ai.SetTheory.BooleanAlgebra.Filter`
- Tasks:
  - Define abstract Boolean algebras and subset Boolean algebras.
  - Define ideals, filters, prime ideals, maximal ideals, ultrafilters, complete Boolean algebras, and regular open algebra interfaces.
  - Prove elementary filter/ideal and subset-Boolean-algebra lemmas where prerequisites exist.
- Deliverables:
  - Boolean algebra and filter API reusable by forcing, topology, and algebra.
- Acceptance criteria:
  - Abstract Boolean algebra lemmas are separated from subset-of-carrier lemmas.
  - Prime ideal and ultrafilter extension theorems are deferred to explicit choice-strength tasks.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.BooleanAlgebra`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.BooleanAlgebra.Filter`

### SET-T30 Add Boolean Prime Ideal, Stone Representation, And Forcing Boolean Bridge

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T29`
- Areas: `Proofs.Ai.SetTheory.BooleanAlgebra.Stone`
- Tasks:
  - Connect ultrafilter lemma and Boolean prime ideal theorem equivalence from `SET-T16`.
  - Add filter extension theorem, Stone representation theorem, and Stone duality statement forms.
  - Add Boolean-valued model bridge interfaces for forcing.
- Deliverables:
  - Stone/Boolean bridge theorem cards and choice-strength documentation.
- Acceptance criteria:
  - Stone-space claims import topology prerequisites or remain explicit interfaces.
  - Boolean-valued model bridge does not assert forcing preservation or truth lemma.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.BooleanAlgebra.Stone`
  - `rg -n "Stone representation|Stone duality|Boolean-valued|Boolean prime" proofs/set-theory-theorem-proof-roadmap*.md proofs/topology-theorem-proof-roadmap*.md`

### SET-T31 Add Structured Syntax, Structures, And Satisfaction Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T18`, `SET-T23`
- Areas: `Proofs.Ai.SetTheory.Model.Structure`, `Proofs.Ai.SetTheory.Model.Satisfaction`
- Tasks:
  - Define first-order language, term, formula, structure, assignment, and satisfaction for a bounded initial fragment.
  - Prove or card isomorphism invariance of satisfaction for supported fragments.
  - Record unsupported fragments as explicit blockers instead of using string encodings.
- Deliverables:
  - Structured syntax and satisfaction interface for constructibility, forcing, and model theory.
- Acceptance criteria:
  - Syntax is not encoded as fragile strings.
  - Satisfaction theorems specify the supported fragment.
  - Truth predicates do not exceed the declared fragment.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Model.Structure`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Model.Satisfaction`

### SET-T32 Add Elementarity, Lowenheim-Skolem, Ultraproduct, And Los Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T31`
- Areas: `Proofs.Ai.SetTheory.Model.Elementarity`
- Tasks:
  - Define elementary substructure, elementary embedding, Skolem hull, and model-relative reflection statement forms.
  - Add Lowenheim-Skolem, compactness theorem, ultraproduct, and Los theorem interfaces.
  - Connect Mostowski collapse and absoluteness theorem cards to model-theoretic assumptions.
- Deliverables:
  - Model-theory interface module for large cardinals, forcing, constructibility, and relative consistency.
- Acceptance criteria:
  - Elementary embedding vocabulary is sufficient for `SET-T37` and `SET-T38`.
  - Compactness and Los remain theorem cards until their metatheoretic proofs are supported.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Model.Elementarity`
  - `rg -n "Lowenheim|Skolem|ultraproduct|Los|elementary embedding" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T33 Add Forcing Posets, Names, And Forcing Relation Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T23`, `SET-T30`, `SET-T32`
- Areas: `Proofs.Ai.SetTheory.Forcing.Poset`, `Proofs.Ai.SetTheory.Forcing.Names`
- Tasks:
  - Define forcing poset, compatibility, dense set, filter, generic filter, names, valuation, and forcing relation statement forms.
  - Add Cohen forcing and collapse forcing vocabulary.
  - Keep truth lemma and generic extension construction as interfaces until `SET-T34`.
- Deliverables:
  - Forcing vocabulary module for independence theorem cards.
- Acceptance criteria:
  - Forcing relation is represented structurally, not as prose.
  - Genericity and model assumptions are explicit.
  - Forcing posets do not import CH independence conclusions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Forcing.Poset`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Forcing.Names`

### SET-T34 Add Generic Extension, Truth Lemma, Preservation, And Boolean-Valued Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T33`
- Areas: `Proofs.Ai.SetTheory.Forcing.Extension`, `Proofs.Ai.SetTheory.Forcing.BooleanValued`
- Tasks:
  - Add generic extension construction, truth lemma, and preservation of ZFC as explicit theorem cards.
  - Add proper, ccc, closed, chain-condition, Levy collapse, and Boolean-valued model interfaces.
  - Provide import contracts for CH independence and infinite-combinatorics independence cards.
- Deliverables:
  - Forcing metatheory theorem-card batch.
- Acceptance criteria:
  - Truth lemma and preservation remain `L2/L3` until satisfaction/metatheory support is formalized.
  - Boolean-valued model statements import Boolean algebra bridge from `SET-T30`.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Forcing.Extension`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Forcing.BooleanValued`

### SET-T35 Add Inner Model Basics

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T25`, `SET-T34`
- Areas: `Proofs.Ai.SetTheory.InnerModel.Basic`
- Tasks:
  - Define transitive model, inner model, and class model.
  - Prove or card absoluteness of elementary set operations for transitive models.
  - Import constructible universe as an inner model from `SET-T25`.
- Deliverables:
  - Inner-model vocabulary for constructibility, forcing, and large-cardinal interactions.
- Acceptance criteria:
  - Transitive-model lemmas state the exact absoluteness fragment.
  - Inner model definitions do not imply global satisfaction for unsupported syntax.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.InnerModel.Basic`
  - `rg -n "inner model|transitive model|class model|absoluteness" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T36 Add Core Model, Fine Structure, And Mouse Theorem Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T35`
- Areas: `Proofs.Ai.SetTheory.InnerModel.Core`, `Proofs.Ai.SetTheory.InnerModel.Mice`
- Tasks:
  - Add covering lemma, core model, fine structure, mouse, and iterable premouse theorem cards.
  - Add optional large-cardinal lower-bound consequence cards that explicitly wait for `SET-T38`.
  - Record fine-structure blockers instead of presenting advanced metatheory as checked proof.
- Deliverables:
  - Advanced inner-model theorem-card batch.
- Acceptance criteria:
  - Core-model and mouse claims remain `L3` until fine-structure infrastructure exists.
  - No large-cardinal consequence is recorded without consistency-strength assumptions.
- Verification:
  - `rg -n "core model|covering lemma|mouse|premouse|fine structure" proofs/set-theory-theorem-proof-roadmap*.md proofs`
  - `git diff --check`

### SET-T37 Add Large Cardinal Statement Taxonomy

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T22`, `SET-T29`, `SET-T32`
- Areas: `Proofs.Ai.SetTheory.LargeCardinal.Basic`
- Tasks:
  - Add inaccessible, Mahlo, weakly compact, measurable, supercompact, huge, and Woodin cardinal statement forms.
  - Add implication hierarchy theorem cards with explicit hypotheses.
  - Record consistency-strength comparison cards needed by determinacy and inner-model milestones.
- Deliverables:
  - Large-cardinal statement taxonomy.
- Acceptance criteria:
  - Large-cardinal assumptions are never hidden inside ordinary cardinal theorems.
  - Weak compactness and tree-property links state their combinatorial prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.LargeCardinal.Basic`
  - `rg -n "inaccessible|Mahlo|weakly compact|measurable|supercompact|Woodin" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T38 Add Normal Measures, Ultrapowers, Embeddings, And Kunen Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T37`, `SET-T32`
- Areas: `Proofs.Ai.SetTheory.LargeCardinal.Ultrafilter`, `Proofs.Ai.SetTheory.LargeCardinal.Embedding`
- Tasks:
  - Define normal measure, ultrafilter, ultrapower, elementary embedding, and critical point statement forms.
  - Add Los theorem import contract for ultrapowers.
  - Add measurable-from-embedding and Kunen inconsistency theorem cards.
- Deliverables:
  - Ultrapower and embedding interface for large cardinals.
- Acceptance criteria:
  - Elementary embedding theorems use model-theory interfaces from `SET-T32`.
  - Kunen-style claims are stated only after embedding vocabulary is fixed.
  - Normal measures do not reuse topology ultrafilter notions without an explicit bridge.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.LargeCardinal.Ultrafilter`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.LargeCardinal.Embedding`

### SET-T39 Add Borel, Standard Borel, And Polish Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T23`, topology roadmap, measure roadmap
- Areas: `Proofs.Ai.SetTheory.Descriptive.Borel`, `Proofs.Ai.SetTheory.Descriptive.Polish`
- Tasks:
  - Define standard Borel space and Borel isomorphism statement forms.
  - Add Borel hierarchy over open-set-generated sigma algebras.
  - Add Baire space, Cantor space, Polish space, and tree-coding interfaces.
- Deliverables:
  - DST Borel/Polish vocabulary shared with topology and measure.
- Acceptance criteria:
  - Polish-space results import topology definitions instead of redefining them.
  - Measurability-related statements import measure definitions instead of creating duplicate sigma-algebra APIs.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Descriptive.Borel`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Descriptive.Polish`

### SET-T40 Add Analytic, Coanalytic, Projective, And Regularity Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T39`
- Areas: `Proofs.Ai.SetTheory.Descriptive.Analytic`
- Tasks:
  - Define analytic and coanalytic sets via continuous images or projections.
  - Add Suslin theorem, Lusin separation, perfect set theorem, projective hierarchy, and absoluteness theorem cards.
  - Add regularity property statement forms for measurability, property of Baire, and perfect set property.
- Deliverables:
  - Analytic/projective DST theorem-card batch.
- Acceptance criteria:
  - Suslin and Lusin claims remain theorem cards until topology, coding, and measure prerequisites exist.
  - Regularity statement forms identify where determinacy or large-cardinal consequences from later milestones may attach, but do not import those consequences as prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Descriptive.Analytic`
  - `rg -n "Suslin|Lusin|analytic|coanalytic|projective|perfect set" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T41 Add Infinite Games And Determinacy Axiom Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T37`, `SET-T39`
- Areas: `Proofs.Ai.SetTheory.Determinacy.Game`, `Proofs.Ai.SetTheory.Determinacy.Axiom`
- Tasks:
  - Define infinite game, strategy, winning strategy, determined game, AD, projective determinacy, and related statement forms.
  - Add AD conflicts with full choice as theorem card.
  - Record compatibility boundaries among AD, DC, AC fragments, and large-cardinal assumptions.
- Deliverables:
  - Game and determinacy axiom vocabulary.
- Acceptance criteria:
  - AD and full AC are not combined without an explicit conflict theorem or fragment boundary.
  - Strategy existence is represented as evidence, not an untracked global principle.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Determinacy.Game`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Determinacy.Axiom`

### SET-T42 Add Determinacy Regularity And Large-Cardinal Links

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T41`, `SET-T38`, `SET-T40`
- Areas: `Proofs.Ai.SetTheory.Determinacy.Regularity`
- Tasks:
  - Add AD-implies-regularity theorem cards for Lebesgue measurability, property of Baire, and perfect set property.
  - Add projective determinacy from large-cardinal assumptions and Martin-Steel theorem cards.
  - Provide import contracts for DST regularity statements in `SET-T40`.
- Deliverables:
  - Determinacy regularity and large-cardinal consequence cards.
- Acceptance criteria:
  - Large-cardinal-to-determinacy implications remain model-relative.
  - Regularity consequences import measure/topology definitions where needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Determinacy.Regularity`
  - `rg -n "AD|projective determinacy|Martin-Steel|regularity|Lebesgue" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T43 Add Set-Theoretic Topology Cardinal Invariants And Product Interfaces

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T16`, `SET-T39`, topology roadmap
- Areas: `Proofs.Ai.SetTheory.Topology.CardinalInvariants`, `Proofs.Ai.SetTheory.Topology.Product`
- Tasks:
  - Define topological cardinal invariants: weight, density, character, cellularity, Lindelof number, and spread over topology structures.
  - Add product-set and product-topology cardinal lemmas needed by topology.
  - Add Tychonoff and Alexander subbase dependency statements with choice/ultrafilter evidence.
- Deliverables:
  - Set-theoretic topology interface for topology proof roadmap imports.
- Acceptance criteria:
  - Topological statements import topology structures instead of reimplementing them.
  - Choice dependencies are visible at the interface.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Topology.CardinalInvariants`
  - `rg -n "Tychonoff|Alexander|cardinal invariant|Lindelof" proofs/set-theory-theorem-proof-roadmap*.md proofs/topology-theorem-proof-roadmap*.md`

### SET-T44 Add Compactness, Stone-Cech, Suslin Line, And MA Topology Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T43`, `SET-T30`, `SET-T34`
- Areas: `Proofs.Ai.SetTheory.Topology.Compactness`
- Tasks:
  - Add compactness via filters and ultrafilters as set-theoretic interfaces.
  - Add Stone-Cech compactification dependency statement linking topology and ultrafilter prerequisites.
  - Add Suslin line, Souslin hypothesis, Martin's axiom topological consequences, and related independence theorem cards.
- Deliverables:
  - Set-theoretic compactness and topology-independence theorem-card batch.
- Acceptance criteria:
  - Stone-Cech imports topology and ultrafilter prerequisites instead of becoming a set theory proof.
  - Independence statements are model-relative.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Topology.Compactness`
  - `rg -n "Stone-Cech|Suslin line|Souslin|Martin's axiom|ultrafilter compactness" proofs/set-theory-theorem-proof-roadmap*.md proofs/topology-theorem-proof-roadmap*.md`

### SET-T45 Add Infinite Ramsey And Partition Basics

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T14`, `SET-T22`
- Areas: `Proofs.Ai.SetTheory.Combinatorics.Ramsey`, `Proofs.Ai.SetTheory.Combinatorics.Partition`
- Tasks:
  - Add infinite Ramsey theorem statement/proof route for pairs and finite tuples.
  - Add finite Ramsey and Erdos-Szekeres theorem cards unless a later combinatorics roadmap takes ownership.
  - Define partition relation notation and prove basic monotonicity where feasible.
- Deliverables:
  - First infinite-combinatorics theorem batch.
- Acceptance criteria:
  - Partition statements spell out arity, color count, and homogeneous-set target.
  - Finite combinatorics ownership is not duplicated if another roadmap exists later.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Combinatorics.Ramsey`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Combinatorics.Partition`

### SET-T46 Add Advanced Partition, Stationary Reflection, Trees, Square, And Diamond Cards

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: `SET-T45`, `SET-T34`, `SET-T38`
- Areas: `Proofs.Ai.SetTheory.Combinatorics.Stationary`
- Tasks:
  - Add Erdos-Rado, Delta-system, stationary reflection, tree property, Aronszajn trees, Suslin trees, square, diamond, and club guessing theorem cards.
  - Connect Fodor and stationary infrastructure from `SET-T22`.
  - Mark forcing-sensitive and large-cardinal-sensitive claims as `L2/L3`.
- Deliverables:
  - Advanced infinite-combinatorics theorem-card batch.
- Acceptance criteria:
  - Independence-sensitive claims import forcing or large-cardinal interfaces.
  - Stationary reflection and tree-property statements list cardinal hypotheses.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.Combinatorics.Stationary`
  - `rg -n "Erdos-Rado|Delta-system|Aronszajn|square|diamond|club guessing|tree property" proofs/set-theory-theorem-proof-roadmap*.md proofs`

### SET-T47 Stabilize Set Theory Import Graph And Prelude

- Status: Completed (2026-06-13; L2 route certificates)
- Depends on: selected checked set theory closure from earlier task batches
- Areas: `Proofs.Ai.SetTheory`, `Proofs.Ai.SetTheory.Prelude`, `proofs/README.md`
- Tasks:
  - Stabilize import graph from elementary modules upward.
  - Create or update `Proofs.Ai.SetTheory.Prelude` containing only low-risk
    explicit-prerequisite infrastructure; no theorem is counted unless it is
    source-free `L2` evidence.
  - Keep global choice, ZFC, forcing, determinacy, and large-cardinal packages out of the prelude.
  - Export theorem names needed by topology, measure, statistics, analysis, algebra, and model theory.
- Deliverables:
  - Set theory module index and prelude policy.
- Acceptance criteria:
  - Prelude does not accidentally import global choice, ZFC, forcing, determinacy, or large-cardinal assumptions.
  - Cross-roadmap import map is documented.
- Verification:
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`
  - `rg -n "Proofs.Ai.SetTheory.Prelude|global choice|forcing|large-cardinal" proofs/set-theory-theorem-proof-roadmap*.md proofs/README.md`

## Review Checklist For Future Updates

- Every task has an explicit dependency list, deliverable, acceptance criteria,
  and verification command or targeted review.
- Range-style dependencies are allowed only for aggregation tasks such as
  axiom-package setup, and must name the required closure scope.
- Tasks that require choice, quotient, classical reasoning, replacement,
  class comprehension, forcing, determinacy, or large-cardinal assumptions make
  those assumptions visible in statements or law packages.
- `SET-T01` through `SET-T14` remain usable without importing advanced
  metatheory.
- `SET-T25` through `SET-T46` do not claim fully derived proofs when only
  theorem cards or interfaces exist.
- Topology and measure overlaps point to their primary roadmap owners for
  concrete topological or measure-theoretic statements.

## Completion Criteria

The task breakdown is complete when:

1. Every roadmap milestone `SET-00` through `SET-23` is covered by at least one
   concrete `SET-T*` task.
2. The first execution queue `SEQ-001` through `SEQ-020` maps to concrete tasks.
3. Elementary infrastructure tasks can be implemented without guessing at
   forcing, model theory, determinacy, or large-cardinal infrastructure.
4. Advanced theorem cards are marked with explicit dependencies and do not
   expand the trusted base.
