# Set Theory Theorem Proof Roadmap

Date: 2026-06-04

This document is a planning sidecar for proving the pasted set theory theorem list in the
NPA proof corpus. It is not part of the trusted base. A theorem is accepted only when its
canonical certificate is checked, source-free verification succeeds, deterministic hashes
are stable, and any axiom footprint is visible in the axiom report.

## Scope

The source list covers elementary set algebra, relations, functions, cardinality, ordered
sets, choice principles, ordinals, cardinals, ZF/ZFC and class theories, paradox avoidance,
constructibility, CH and independence, forcing, inner models, large cardinals, descriptive
set theory, determinacy, set-theoretic topology, infinite combinatorics, Boolean algebras,
and model-theoretic interfaces.

This roadmap treats those topics as one staged proof program:

- Build elementary set, relation, function, order, ordinal, and cardinal infrastructure
  before importing high-level theorems.
- Keep choice, classical reasoning, quotient principles, replacement-like principles, and
  model-existence principles explicit as theorem assumptions or axiom packages.
- Record independence, consistency-strength, forcing, inner-model, and large-cardinal
  statements as interface-level theorem cards until the required metatheory is present.
- Share reusable foundations with the topology, measure theory, statistics, linear
  algebra, and analysis proof roadmaps without duplicating theorem ownership.

## Existing Baseline

- There is no checked `Proofs.Ai.SetTheory.*` proof tree yet.
- Existing reusable proof-corpus modules include `Proofs.Ai.Basic`, `Proofs.Ai.Eq`,
  `Proofs.Ai.EqReasoning`, `Proofs.Ai.Prop`, `Proofs.Ai.Nat`, `Proofs.Ai.Logic.Iff`,
  and algebraic modules such as groups, rings, fields, and quotient-oriented structures.
- The repository has audited quotient support through the explicit `quotient_v1` feature
  profile and `Std.Quotient` examples. Any quotient use in this roadmap must still be
  visible through the feature/profile boundary or an explicit quotient package; extensionality,
  choice, `funext`, and `propext` remain explicit theorem or law-package dependencies.
- Topology roadmap owns topological consequences such as Tychonoff-style product
  compactness, Urysohn, Tietze, Stone-Cech, and Baire category. This roadmap owns the
  set-theoretic choice and ultrafilter principles those results may import.
- Measure and descriptive set theory overlap around Borel, analytic, and Polish-space
  objects. This roadmap owns set-theoretic definability and regularity principles; measure
  and topology roadmaps own their analytic/topological specializations.

## Proof Levels

| Level | Meaning | Acceptance target |
| --- | --- | --- |
| `L0` | Constructive first-order/core proof with no additional classical or set-theoretic axiom package. | Checked `.npcert`, no unexpected axiom-report growth. |
| `L1` | Proof uses an explicit law package, such as extensional sets, quotients, excluded middle, choice evidence, replacement, or class comprehension. | Axiom package is named in the theorem statement or module header and appears in reports. |
| `L2` | Metatheorem or model-relative theorem, such as consistency, independence, forcing preservation, constructibility, or large-cardinal implication. | Formal interface and assumptions are checked; deeper model proof may be deferred behind explicit assumptions. |
| `L3` | Catalog or expository theorem card whose exact formalization depends on future infrastructure. | Statement, dependencies, owner module, and reduction path are recorded; not counted as proved. |

## One-Theorem Work Unit

Each theorem should be proved as a small independently reviewable unit:

1. Create or update the theorem card in the nearest `Proofs.Ai.SetTheory.*` module.
2. State every non-kernel principle as an explicit premise or imported law package.
3. Build the local certificate for the module and its import closure.
4. Verify the checked-in certificate source-free.
5. If useful for future AI search, write a replay file and refresh the AI theorem index.

Preferred local commands:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.SetTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.SetTheory.X
cargo run -p npa-proof-corpus -- --changed-only
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.SetTheory.X::theorem_name proofs/generated/replay-set-theory-theorem.json
cargo run -p npa-proof-corpus -- --write-ai-index
```

End-of-batch verification for ordinary theorem authoring:

```sh
./scripts/check-corpus-authoring.sh
```

Use `./scripts/check-corpus-package.sh` when package metadata, verifier examples, package
CLI behavior, index generation, axiom reports, or publish planning changes. Use
`./scripts/check-corpus-full.sh` before high-trust promotion.

## Statement Policy

- Sets, classes, relations, functions, ordinals, cardinals, models, filters, ideals,
  Boolean algebras, forcing notions, and elementary embeddings are ordinary structures or
  explicit evidence records. They are not kernel primitives.
- Extensionality for a set representation is an explicit law of that representation.
- Quotient sets and equivalence classes require either concrete representatives with
  invariance lemmas or an explicit quotient package.
- Choice principles, Zorn's lemma, the well-ordering theorem, Hausdorff maximal principle,
  the ultrafilter lemma, and Boolean prime ideal theorem are never silently available.
- ZF, ZFC, NBG, MK, global choice, replacement, foundation, inaccessible universes, and
  reflection principles are represented as named axiom packages or model assumptions.
- Russell-style paradoxes are formalized as impossibility or inconsistency theorems for
  unrestricted comprehension. Do not introduce a universal set as an ordinary set in a
  ZF-style layer.
- `Classical.choice`, `funext`, and `propext` are not used as hidden conveniences. If a
  theorem needs them, the theorem records that dependency.
- Independence and consistency claims are model-relative unless a future metatheory module
  proves the required soundness theorem inside NPA.

## Duplicate Theorem Policy

- Choice-equivalence theorems are owned by `SET-07`. Topology, algebra, analysis, and
  measure roadmaps import the needed consequences.
- Tychonoff, Alexander subbase theorem, Stone-Cech compactification, and Baire category
  are topological theorems. Ultrafilter, filter extension, and Boolean prime ideal
  principles are set-theoretic.
- Vector-space basis existence, Hahn-Banach, maximal ideals, and algebraic closure
  existence are owned by their domain roadmaps; this roadmap owns only the choice interface
  or maximal-principle schema.
- Borel, analytic, coanalytic, projective, and Polish-space theorems are shared. This
  roadmap owns definability and set-theoretic regularity; topology and measure roadmaps
  own concrete topological and measure statements.
- Ramsey, Erdos-Rado, partition calculus, Delta-system, club, stationary, and Fodor-style
  results are owned here unless a future combinatorics roadmap supersedes that ownership.
- Forcing, CH, GCH, constructibility, inner-model, determinacy, and large-cardinal
  statements are owned here as explicit metatheoretic interfaces.

## Milestone Map

Milestone numbers follow the source-list topic order, not a strict topological execution
order. Use the `Depends` column and the recommended execution queue to choose implementation
order, especially around forcing, Boolean algebras, model theory, inner models, and large
cardinals.

| Milestone | Theme | Level | Depends |
| --- | --- | --- | --- |
| `SET-00` | Inventory, theorem naming, and axiom taxonomy | `L0` planning | none |
| `SET-01` | Elementary set algebra | `L0/L1` | `SET-00` |
| `SET-02` | Relations and quotients | `L0/L1` | `SET-01` |
| `SET-03` | Functions, images, and inverse images | `L0/L1` | `SET-01`, `SET-02` |
| `SET-04` | Finite, countable, and enumerable sets | `L0/L1` | `SET-03`, Nat modules |
| `SET-05` | Cardinal comparison and Cantor theorems | `L0/L1` | `SET-03`, `SET-04` |
| `SET-06` | Ordered sets, lattices, and well-founded orders | `L0/L1` | `SET-02`, `SET-05` |
| `SET-07` | Choice principles and equivalents | `L1/L2` | `SET-06` |
| `SET-08` | Ordinals and transfinite methods | `L1/L2` | `SET-06`, `SET-07` |
| `SET-09` | Cumulative hierarchy and foundation | `L1/L2` | `SET-08` |
| `SET-10` | Cardinals, cofinality, regularity, stationary sets | `L1/L2` | `SET-05`, `SET-08` |
| `SET-11` | ZF/ZFC, class theories, and paradox boundaries | `L1/L2` | `SET-01` through `SET-10` |
| `SET-12` | Constructible universe and absoluteness | `L2/L3` | `SET-08`, `SET-11` |
| `SET-13` | CH, GCH, continuum function, and independence | `L2/L3` | `SET-10`, `SET-12`; forcing-backed cards wait for the forcing milestone |
| `SET-14` | Forcing and Boolean-valued models | `L2/L3` | `SET-11`, `SET-21`, `SET-22` |
| `SET-15` | Inner models and core-model interfaces | `L2/L3` | `SET-12`, `SET-14`; large-cardinal consequences wait for the large-cardinal milestone |
| `SET-16` | Large cardinals and ultrapowers | `L2/L3` | `SET-10`, `SET-21`, `SET-22` |
| `SET-17` | Descriptive set theory and Polish interfaces | `L1/L2` | `SET-11`, topology roadmap, measure roadmap |
| `SET-18` | Determinacy and regularity consequences | `L2/L3` | `SET-16`, `SET-17` |
| `SET-19` | Set-theoretic topology interfaces | `L1/L2` | `SET-07`, `SET-17`, topology roadmap |
| `SET-20` | Infinite combinatorics and partition calculus | `L1/L2` | `SET-06`, `SET-10`, `SET-14`, `SET-16` |
| `SET-21` | Boolean algebras, ultrafilters, and Stone duality | `L1/L2` | `SET-01`, `SET-07`, topology roadmap |
| `SET-22` | Model theory interfaces | `L1/L2` | `SET-08`, `SET-11`, `SET-12` |
| `SET-23` | Packaging, promotion, and cross-roadmap reuse | `L0/L1` | all applicable milestones |

## SET-00 Inventory, Naming, And Axiom Taxonomy

Status: planned.

Target modules:

- `Proofs.Ai.SetTheory.Index`
- `Proofs.Ai.SetTheory.Axioms`
- `Proofs.Ai.SetTheory.TheoremCards`

Theorem order:

1. Normalize all pasted theorem names into stable ASCII theorem identifiers.
2. Tag every theorem as elementary, classical, choice-dependent, replacement-dependent,
   class-theoretic, model-relative, independence, or large-cardinal.
3. Create theorem-card records for named results such as Cantor, Cantor-Bernstein,
   Schroeder-Bernstein, Hartogs, Burali-Forti, Mostowski collapse, Zorn, Tarski, Godel
   constructibility, Cohen forcing, Shoenfield absoluteness, Los, Kunen inconsistency,
   Silver, Solovay, Martin's axiom, Suslin theorem, Ramsey, Erdos-Rado, Fodor, and Stone.
4. Mark whether each theorem is intended for a proved certificate in this repository or
   only as an explicit interface pending future metatheory.

Deliverables:

- Theorem index grouped by milestone and owner module.
- Axiom package taxonomy with short names such as `set_extensionality`, `classical_em`,
  `quotient_lift`, `choice_family`, `zfc_model`, `forcing_extension`, and
  `large_cardinal_axiom`.
- Duplicate-ownership notes for topology, measure, analysis, algebra, and model theory.

Acceptance criteria:

- Every item from the pasted list maps to exactly one primary owner milestone or a named
  cross-roadmap import.
- No theorem card claims a hidden kernel extension.
- Axiom packages are fine-grained enough for axiom-report review.

## SET-01 Elementary Set Algebra

Status: planned.

Depends: `SET-00`.

Target modules:

- `Proofs.Ai.SetTheory.Basic`
- `Proofs.Ai.SetTheory.Subset`
- `Proofs.Ai.SetTheory.BooleanOps`

Theorem order:

1. Membership, subset, equality by extensionality, empty set, universal set relative to a
   fixed ambient carrier, singleton, pair, union, intersection, difference, complement,
   powerset, indexed union, and indexed intersection.
2. Subset preorder laws: reflexivity, transitivity, antisymmetry under extensionality.
3. Empty and total laws: `A union empty = A`, `A intersect empty = empty`,
   `A union U = U`, `A intersect U = A`.
4. Idempotent, commutative, associative, absorption, distributive, double-complement, and
   De Morgan laws.
5. Monotonicity of union/intersection, subset criteria for union/intersection, and
   powerset monotonicity.
6. Product-set definitions and elementary product laws.

Deliverables:

- A minimal set representation layer parameterized by membership and extensionality laws.
- Boolean algebra of subsets of a fixed carrier as the first stable API.
- Rewrite lemmas suitable for later topology, measure, and function modules.

Acceptance criteria:

- Elementary laws are either `L0` structural proofs or `L1` proofs whose only additional
  dependency is the explicit set-extensionality law.
- No theorem assumes unrestricted comprehension.
- Rewrite theorem names distinguish set equality from logical equivalence of membership.

## SET-02 Relations And Quotients

Status: planned.

Depends: `SET-01`.

Target modules:

- `Proofs.Ai.SetTheory.Relation`
- `Proofs.Ai.SetTheory.Equivalence`
- `Proofs.Ai.SetTheory.Quotient`

Theorem order:

1. Binary relation as subset or predicate over a product.
2. Domain, range, inverse relation, composition, identity relation, and restriction.
3. Reflexive, symmetric, antisymmetric, transitive, total, functional, injective-relation,
   and well-founded predicates.
4. Equivalence relation basics: equivalence class, class membership, class equality,
   disjointness of distinct classes, and partition induced by an equivalence relation.
5. Partition-to-equivalence construction.
6. Quotient map universal property as an `L1` theorem over an explicit quotient package.
7. Order relations as specialized relations for `SET-06`.

Deliverables:

- Relation algebra lemmas for later order, function, graph, and model theory modules.
- Equivalence relation and partition equivalence.
- A quotient interface that records the precise quotient principle used.

Acceptance criteria:

- Quotient theorems do not rely on an implicit kernel quotient.
- Partition/equivalence conversion is proved in both directions.
- Later modules can import relation composition and inverse-relation laws without copying
  membership arguments.

## SET-03 Functions, Images, And Inverse Images

Status: planned.

Depends: `SET-01`, `SET-02`.

Target modules:

- `Proofs.Ai.SetTheory.Function`
- `Proofs.Ai.SetTheory.Image`
- `Proofs.Ai.SetTheory.Family`

Theorem order:

1. Function as a functional relation or record with domain, codomain, graph, and
   application law.
2. Identity, constant, restriction, extension by agreement, composition, and inverse image.
3. Injective, surjective, bijective, left-inverse, right-inverse, and inverse-function
   characterizations.
4. Image and preimage laws for union, intersection, difference, complement, and indexed
   families.
5. Direct image monotonicity and inverse image Boolean-algebra homomorphism laws.
6. Function equality from pointwise equality as an explicit function-extensionality
   package if required by the selected representation.
7. Family of sets, indexed operations, choice function statement form, and dependent
   product interface.

Deliverables:

- Function API for cardinality, topology, measure, and algebra roadmaps.
- Stable image/preimage theorem names.
- Explicit split between constructive inverses and choice-based right inverses.

Acceptance criteria:

- Every inverse-existence theorem states whether it constructs an inverse or invokes
  choice.
- Preimage laws are available before topology opens closed-set and continuity proofs.
- Direct image laws document the usual inclusion weakening for intersections.

## SET-04 Finite, Countable, And Enumerable Sets

Status: planned.

Depends: `SET-03`, Nat modules.

Target modules:

- `Proofs.Ai.SetTheory.Finite`
- `Proofs.Ai.SetTheory.Countable`
- `Proofs.Ai.SetTheory.Enumeration`

Theorem order:

1. Finite set definitions via bijection with initial natural segments and via inductive
   finite closure.
2. Basic finite laws: empty finite, singleton finite, insertion, deletion, finite union,
   finite product, finite image, and finite subset.
3. Pigeonhole principle for finite sets.
4. Countable, countably infinite, enumerable, and at-most-countable definitions.
5. `Nat` is countable, finite sets are countable, subsets of countable sets are countable
   under the selected choice/effective enumeration assumptions.
6. Countable union of finite sets and countable union of countable sets, with choice
   assumptions made explicit.
7. `Int`, `Rat`, finite sequences over countable sets, and countable products in limited
   forms needed by analysis and topology.

Deliverables:

- Finite and countable cardinality library.
- Constructive enumeration lemmas separate from choice-based enumeration lemmas.
- Reusable countability proofs for rational numbers, polynomial codes, syntax codes, and
  basic bases of second-countable spaces.

Acceptance criteria:

- Pigeonhole and finite-cardinality theorems do not use choice.
- Countable-union theorems record whether they require countable choice.
- Every countability theorem specifies the representation of enumerability.

## SET-05 Cardinal Comparison And Cantor Theorems

Status: planned.

Depends: `SET-03`, `SET-04`.

Target modules:

- `Proofs.Ai.SetTheory.Cardinal.Basic`
- `Proofs.Ai.SetTheory.Cardinal.Cantor`
- `Proofs.Ai.SetTheory.Cardinal.Compare`

Theorem order:

1. Equipotence, cardinal inequality via injection, and cardinal inequality via surjection.
2. Equipotence equivalence relation.
3. Injection/surjection comparison lemmas and transport of finite/countable/cardinal
   properties across bijections.
4. Cantor theorem: no surjection from a set onto its powerset.
5. Diagonal arguments for `Nat -> Bool`, uncountability of powerset of `Nat`, and
   uncountability of reals once the real-number representation is available.
6. Cantor-Bernstein-Schroeder theorem.
7. Cardinal arithmetic basics for sums, products, function spaces, and powersets.
8. Hartogs theorem as an interface if ordinal infrastructure is not yet sufficient.

Deliverables:

- Cardinal comparison vocabulary independent of a global cardinal quotient where possible.
- Diagonal proof patterns reusable by analysis and computability-style modules.
- Explicit statement of Cantor-Bernstein dependencies.

Acceptance criteria:

- Cantor theorem is proved without choice.
- Cantor-Bernstein is isolated so it can be audited for the exact principles required.
- Cardinal arithmetic theorem names do not conflate representatives with quotient
  cardinals.

## SET-06 Ordered Sets, Lattices, And Well-Founded Orders

Status: planned.

Depends: `SET-02`, `SET-05`.

Target modules:

- `Proofs.Ai.SetTheory.Order.Poset`
- `Proofs.Ai.SetTheory.Order.Lattice`
- `Proofs.Ai.SetTheory.Order.WellFounded`

Theorem order:

1. Preorder, partial order, linear order, strict order, well-founded relation, well-order,
   chain, antichain, upper bound, lower bound, supremum, infimum, maximum, and minimum.
2. Dual order and suborder lemmas.
3. Product, lexicographic, and pointwise orders.
4. Complete lattice definitions and monotone map basics.
5. Knaster-Tarski fixed point theorem, if complete-lattice infrastructure is sufficient.
6. Well-founded induction and recursion interfaces.
7. Initial segment and order-isomorphism basics.
8. Dense order and endpoint definitions for later rational/real-order examples.

Deliverables:

- Poset and well-order APIs for ordinals, Zorn, and topology.
- Complete-lattice layer for fixed point and closure-operator proofs.
- Well-founded induction schema over explicit well-founded evidence.

Acceptance criteria:

- Order proofs reuse relation infrastructure.
- Maximal-principle theorems that require choice are deferred to `SET-07`.
- Well-founded recursion remains an explicit principle if the current corpus lacks the
  necessary recursion infrastructure.

## SET-07 Choice Principles And Equivalents

Status: planned.

Depends: `SET-06`.

Target modules:

- `Proofs.Ai.SetTheory.Choice`
- `Proofs.Ai.SetTheory.Maximal`
- `Proofs.Ai.SetTheory.Ultrafilter`

Theorem order:

1. Choice function for a family of nonempty sets.
2. Dependent choice and countable choice as weaker explicit principles.
3. Well-ordering theorem statement and direction from choice.
4. Zorn's lemma statement and choice-to-Zorn route.
5. Hausdorff maximal principle and maximal-chain theorem.
6. Tukey lemma and finite-character maximal principle.
7. Ultrafilter lemma and Boolean prime ideal theorem.
8. Equivalence web among choice, well-ordering, Zorn, Hausdorff maximal principle, and
   selected maximal principles.
9. Domain-specific corollary interfaces: every vector space has a basis, every proper
   ideal extends to a maximal ideal, and product of compact spaces is compact.

Deliverables:

- A small set of explicit choice packages with clear strength labels.
- Reusable maximal principle schemas.
- Import contracts for topology, algebra, analysis, and model theory.

Acceptance criteria:

- No theorem in this milestone is treated as kernel-trusted.
- Equivalence proofs identify direction-specific assumptions.
- Downstream roadmaps import the principle they need instead of restating global choice.

## SET-08 Ordinals And Transfinite Methods

Status: planned.

Depends: `SET-06`, `SET-07`.

Target modules:

- `Proofs.Ai.SetTheory.Ordinal.Basic`
- `Proofs.Ai.SetTheory.Ordinal.Induction`
- `Proofs.Ai.SetTheory.Ordinal.Recursion`

Theorem order:

1. Transitive set, ordinal predicate, zero ordinal, successor ordinal, limit ordinal, and
   ordinal membership order.
2. Ordinals are well-ordered by membership.
3. Trichotomy and transitivity of ordinal comparison.
4. Every element of an ordinal is an ordinal.
5. Successor and limit ordinal laws.
6. Transfinite induction over ordinals.
7. Transfinite recursion as explicit interface or proved theorem over a recursion package.
8. Supremum of a set of ordinals and union of ordinals.
9. Order-isomorphism between well-orders and ordinals, with choice assumptions explicit
   where arbitrary well-ordering is needed.
10. Burali-Forti paradox boundary: there is no set of all ordinals in a ZF-like universe.

Deliverables:

- Ordinal API for cardinal definitions, hierarchy construction, and transfinite proof
  schemas.
- Separation between ordinal facts and class-level facts about all ordinals.
- Transfinite induction pattern suitable for later use.

Acceptance criteria:

- Ordinal comparison lemmas do not assume global choice unless necessary.
- Burali-Forti is stated as a class/set boundary theorem, not as an unrestricted
  comprehension paradox inside the trusted core.
- Recursion theorem states the precise replacement/recursion principle it uses.

## SET-09 Cumulative Hierarchy And Foundation

Status: planned.

Depends: `SET-08`.

Target modules:

- `Proofs.Ai.SetTheory.Hierarchy`
- `Proofs.Ai.SetTheory.Foundation`
- `Proofs.Ai.SetTheory.Rank`

Theorem order:

1. Cumulative hierarchy levels `V_alpha` as an explicit recursive construction.
2. Monotonicity of hierarchy levels.
3. Transitivity of appropriate levels.
4. Rank of a set as explicit theorem under foundation/replacement assumptions.
5. Foundation and regularity consequences: no membership cycles, no set contains itself,
   and induction on membership.
6. Mostowski collapse theorem statement, first as an interface over extensional
   well-founded relations.
7. Reflection principle statement as a metatheoretic interface.

Deliverables:

- Rank and hierarchy vocabulary for ZFC, constructibility, large cardinals, and forcing.
- Explicit foundation package and consequences.
- Interface for collapse and reflection theorems.

Acceptance criteria:

- Foundation-dependent results are not mixed into elementary set algebra.
- Rank and hierarchy construction reports replacement-like dependencies.
- Collapse theorem remains `L2/L3` until relation, quotient, and recursion support are
  sufficient.

## SET-10 Cardinals, Cofinality, Regularity, And Stationary Sets

Status: planned.

Depends: `SET-05`, `SET-08`.

Target modules:

- `Proofs.Ai.SetTheory.Cardinal.Ordinal`
- `Proofs.Ai.SetTheory.Cardinal.Cofinality`
- `Proofs.Ai.SetTheory.Stationary`

Theorem order:

1. Initial ordinals and cardinal representatives.
2. Cardinal successor, aleph sequence, beth sequence, and continuum cardinal statement.
3. Cardinal arithmetic for infinite sums, products, and powers, with choice assumptions
   explicit.
4. Konig theorem as a cardinal-arithmetic theorem with required hypotheses.
5. Cofinality of an ordinal/cardinal.
6. Regular and singular cardinal definitions and elementary consequences.
7. Club and stationary subsets of regular uncountable cardinals.
8. Fodor pressing-down lemma.
9. Closed unbounded filter basics.
10. Delta-system lemma and first partition-calculus prerequisites.

Deliverables:

- Cardinal quotient/representative policy usable by later milestones.
- Cofinality, regularity, club, and stationary APIs.
- First infinite-combinatorics tools.

Acceptance criteria:

- Cardinal representative choices are explicit.
- Theorems depending on global choice or well-ordering are marked `L1`.
- Fodor and club/stationary results state regularity hypotheses clearly.

## SET-11 ZF/ZFC, Class Theories, And Paradox Boundaries

Status: planned.

Depends: `SET-01` through `SET-10`.

Target modules:

- `Proofs.Ai.SetTheory.ZF`
- `Proofs.Ai.SetTheory.ZFC`
- `Proofs.Ai.SetTheory.Class`
- `Proofs.Ai.SetTheory.Paradox`

Theorem order:

1. ZF axiom package: extensionality, empty set, pair, union, powerset, infinity,
   separation, replacement, foundation.
2. ZFC as ZF plus choice package.
3. NBG and MK class-theory interface packages.
4. Global choice as a separate class-theoretic principle.
5. Russell paradox for unrestricted comprehension.
6. Cantor paradox boundary for a set of all sets.
7. Burali-Forti boundary reused from ordinal milestone.
8. Separation and replacement consequences used by hierarchy, ordinals, cardinals, and
   function-set construction.
9. Low-level examples showing which earlier elementary set laws require only
   extensionality and which require richer ZF packages.

Deliverables:

- Named axiom packages for later theorem statements.
- Paradox boundary theorems that clarify why unrestricted comprehension is excluded.
- Axiom-report templates for ZF/ZFC/class-theory modules.

Acceptance criteria:

- ZF/ZFC packages are explicit assumptions, not trusted Rust kernel changes.
- Class/set boundary is represented structurally.
- Every paradox theorem is phrased as a contradiction from an unrestricted principle or
  as a nonexistence theorem under ZF-like assumptions.

## SET-12 Constructible Universe And Absoluteness

Status: planned.

Depends: `SET-08`, `SET-11`.

Target modules:

- `Proofs.Ai.SetTheory.Constructible`
- `Proofs.Ai.SetTheory.Absoluteness`
- `Proofs.Ai.SetTheory.InnerModel`

Theorem order:

1. Definability interface for first-order formulas over set structures.
2. Constructible hierarchy `L_alpha` as a class-level recursive construction.
3. Transitivity and monotonicity of `L_alpha`.
4. `L` is an inner model interface.
5. Godel constructibility theorem card: `L` satisfies ZFC under suitable ambient
   assumptions.
6. `V = L` implies global well-ordering of constructible sets.
7. `V = L` implies CH and GCH as model-relative theorem cards.
8. Shoenfield absoluteness and Levy absoluteness interfaces.
9. Condensation lemma interface.

Deliverables:

- Definability and hierarchy interfaces needed before serious constructibility proofs.
- Theorem cards separating formalized hierarchy lemmas from metatheoretic model claims.
- Import path for CH/GCH relative consistency statements.

Acceptance criteria:

- Definability is not represented by ad hoc strings where structured syntax is available.
- All satisfaction and absoluteness statements are clearly model-relative.
- No constructibility theorem claims an absolute proof of CH in ZFC.

## SET-13 CH, GCH, Continuum Function, And Independence

Status: planned.

Depends: `SET-10`, `SET-12`. Forcing-backed independence cards import `SET-14` after
the forcing interface exists.

Target modules:

- `Proofs.Ai.SetTheory.Continuum`
- `Proofs.Ai.SetTheory.Independence`
- `Proofs.Ai.SetTheory.RelativeConsistency`

Theorem order:

1. Continuum hypothesis and generalized continuum hypothesis statement forms.
2. Equivalent forms of CH using `2^aleph_0`, subsets of `Nat`, and real-number
   cardinality when real infrastructure exists.
3. Godel relative consistency theorem card: if ZF is consistent, then ZFC plus GCH is
   consistent, via constructibility.
4. Cohen independence theorem card: if ZFC is consistent, then ZFC plus not-CH is
   consistent, via forcing.
5. Easton theorem interface for continuum-function constraints.
6. Martin's axiom plus not-CH theorem cards.
7. Suslin hypothesis and independence interfaces.

Deliverables:

- Canonical statement modules for CH/GCH and continuum function results.
- Relative-consistency vocabulary reused by forcing and large-cardinal milestones.
- Clear distinction between statement formalization and proof of independence.

Acceptance criteria:

- No theorem states that CH or not-CH is proved from bare ZFC.
- Relative consistency assumptions are explicit hypotheses.
- Cohen-style independence cards are not promoted beyond theorem-card/interface status until
  the relevant `SET-14` forcing interface has landed.
- Equivalent CH formulations identify the representation of real numbers or powerset of
  natural numbers used.

## SET-14 Forcing And Boolean-Valued Models

Status: planned.

Depends: `SET-11`, `SET-21`, `SET-22`.

Target modules:

- `Proofs.Ai.SetTheory.Forcing.Poset`
- `Proofs.Ai.SetTheory.Forcing.Names`
- `Proofs.Ai.SetTheory.Forcing.Extension`
- `Proofs.Ai.SetTheory.Forcing.BooleanValued`

Theorem order:

1. Forcing poset, compatibility, dense set, filter, and generic filter definitions.
2. Names, valuation, forcing relation, and truth lemma as interfaces.
3. Generic extension construction interface.
4. Preservation of ZFC in generic extensions as theorem card.
5. Cohen forcing theorem cards for adding subsets of `Nat`.
6. Collapse forcing and Levy collapse interfaces.
7. Proper, ccc, closed, and chain-condition preservation statements.
8. Boolean-valued model interface and relationship with complete Boolean algebras.

Deliverables:

- Forcing vocabulary that later independence theorems can reference.
- Explicit model and genericity assumptions.
- Boolean-valued model bridge to Boolean algebra milestone.

Acceptance criteria:

- Forcing relation is represented structurally, not as prose.
- Truth lemma and preservation theorems remain explicit `L2/L3` interfaces until the
  satisfaction/metatheory layer is formalized.
- CH independence theorem cards import forcing theorem cards instead of duplicating them.

## SET-15 Inner Models And Core-Model Interfaces

Status: planned.

Depends: `SET-12`, `SET-14`. Large-cardinal lower-bound consequences import `SET-16`
after the large-cardinal vocabulary exists.

Target modules:

- `Proofs.Ai.SetTheory.InnerModel.Basic`
- `Proofs.Ai.SetTheory.InnerModel.Core`
- `Proofs.Ai.SetTheory.InnerModel.Mice`

Theorem order:

1. Transitive model, inner model, and class model definitions.
2. Absoluteness of elementary set operations for transitive models.
3. Constructible universe as an inner model, imported from `SET-12`.
4. Covering lemma theorem card.
5. Core model theorem card.
6. Fine structure interfaces.
7. Mouse and iterable premouse theorem cards.
8. Inner model consequences for large-cardinal lower bounds as interfaces.

Deliverables:

- Minimal inner-model vocabulary for constructibility, forcing, and large cardinals.
- Stable theorem cards for high-complexity named theorems.
- Clear line between formalized basic lemmas and advanced metatheory.

Acceptance criteria:

- Transitive-model lemmas state the exact absoluteness fragment.
- Core-model and mouse claims remain `L3` until fine-structure infrastructure exists.
- No large-cardinal consequence is recorded without its consistency-strength assumptions.

## SET-16 Large Cardinals And Ultrapowers

Status: planned.

Depends: `SET-10`, `SET-21`, `SET-22`.

Target modules:

- `Proofs.Ai.SetTheory.LargeCardinal.Basic`
- `Proofs.Ai.SetTheory.LargeCardinal.Ultrafilter`
- `Proofs.Ai.SetTheory.LargeCardinal.Embedding`

Theorem order:

1. Inaccessible, Mahlo, weakly compact, measurable, supercompact, huge, and Woodin
   cardinal statement forms.
2. Implication hierarchy theorem cards: supercompact implies measurable-style strength,
   measurable implies inaccessible under standard hypotheses, and similar relationships.
3. Normal measure, ultrafilter, and ultrapower definitions.
4. Los theorem interface for ultrapowers.
5. Elementary embedding statement forms and critical point.
6. Measurable cardinal from nontrivial elementary embedding interface.
7. Kunen inconsistency theorem card.
8. Reflection and compactness consequences of weak compactness.
9. Large-cardinal consistency-strength comparison cards used by determinacy and inner
   model milestones.

Deliverables:

- Large-cardinal statement taxonomy.
- Ultrafilter/ultrapower bridge to Boolean algebra and model theory.
- Explicit consistency-strength relationships as theorem cards.

Acceptance criteria:

- Large-cardinal assumptions are never hidden inside ordinary cardinal theorems.
- Elementary embedding theorems use model-theoretic interfaces from `SET-22`.
- Kunen-style results are stated only after the surrounding embedding vocabulary is fixed.

## SET-17 Descriptive Set Theory And Polish Interfaces

Status: planned.

Depends: `SET-11`, topology roadmap, measure roadmap.

Target modules:

- `Proofs.Ai.SetTheory.Descriptive.Borel`
- `Proofs.Ai.SetTheory.Descriptive.Analytic`
- `Proofs.Ai.SetTheory.Descriptive.Polish`

Theorem order:

1. Standard Borel space and Borel isomorphism statement forms.
2. Borel hierarchy definitions for open-set-generated sigma algebras.
3. Analytic and coanalytic set definitions via continuous images or projections.
4. Suslin theorem: Borel iff analytic and coanalytic, as theorem card until topology and
   coding infrastructure are sufficient.
5. Lusin separation and separation theorem interfaces.
6. Perfect set theorem for uncountable closed sets and analytic sets as staged theorem
   cards.
7. Baire space, Cantor space, Polish space, and coding of trees.
8. Regularity properties: measurability, property of Baire, and perfect set property.
9. Projective hierarchy statement forms and absoluteness connections.

Deliverables:

- Set-theoretic definitions that topology and measure roadmaps can import.
- Coding and tree interfaces for analytic sets.
- Explicit assumptions for regularity theorems.

Acceptance criteria:

- Polish-space results import topology definitions rather than redefining them.
- Measurability results import measure definitions rather than creating duplicate sigma
  algebra APIs.
- Projective and regularity statements identify whether they use determinacy or large
  cardinal assumptions.

## SET-18 Determinacy And Regularity Consequences

Status: planned.

Depends: `SET-16`, `SET-17`.

Target modules:

- `Proofs.Ai.SetTheory.Determinacy.Game`
- `Proofs.Ai.SetTheory.Determinacy.Axiom`
- `Proofs.Ai.SetTheory.Determinacy.Regularity`

Theorem order:

1. Infinite game, strategy, winning strategy, determined game.
2. Axiom of determinacy, projective determinacy, and related statement forms.
3. AD conflicts with full choice as theorem card.
4. AD implies regularity properties for sets of reals: Lebesgue measurability, property of
   Baire, and perfect set property, as interface theorem cards.
5. Projective determinacy from large-cardinal assumptions as theorem card.
6. Martin-Steel theorem card.
7. Determinacy consequences for descriptive set theory.

Deliverables:

- Game and strategy vocabulary.
- Explicit incompatibility/compatibility notes among AD, AC, DC, and large-cardinal
  assumptions.
- Import route for DST regularity statements.

Acceptance criteria:

- AD and AC are not combined without a theorem explaining the fragment boundaries.
- Strategy existence is represented as evidence, not as an untracked global principle.
- Large-cardinal-to-determinacy implications remain model-relative theorem cards until
  the metatheory is available.

## SET-19 Set-Theoretic Topology Interfaces

Status: planned.

Depends: `SET-07`, `SET-17`, topology roadmap.

Target modules:

- `Proofs.Ai.SetTheory.Topology.CardinalInvariants`
- `Proofs.Ai.SetTheory.Topology.Product`
- `Proofs.Ai.SetTheory.Topology.Compactness`

Theorem order:

1. Cardinal invariants of topological spaces: weight, density, character, cellularity,
   Lindelof number, and spread as interfaces over topology structures.
2. Product-set and product-topology cardinal lemmas needed by topology roadmap.
3. Tychonoff theorem dependency statement: full product compactness uses a choice-like
   principle or ultrafilter principle.
4. Alexander subbase theorem dependency statement.
5. Stone-Cech compactification dependency statement.
6. Set-theoretic forms of compactness via filters and ultrafilters.
7. Suslin line, Souslin hypothesis, and independence theorem cards.
8. Martin's axiom topological consequences as theorem cards.

Deliverables:

- Set-theoretic dependency layer for topology theorem proofs.
- Cardinal-invariant definitions shared with topology.
- Choice/ultrafilter assumptions attached to compactness routes.

Acceptance criteria:

- Topological statements import topology structures instead of reimplementing them.
- Choice dependencies are visible at the set-theoretic interface.
- Independence statements remain model-relative.

## SET-20 Infinite Combinatorics And Partition Calculus

Status: planned.

Depends: `SET-06`, `SET-10`, `SET-14`, `SET-16`.

Target modules:

- `Proofs.Ai.SetTheory.Combinatorics.Ramsey`
- `Proofs.Ai.SetTheory.Combinatorics.Partition`
- `Proofs.Ai.SetTheory.Combinatorics.Stationary`

Theorem order:

1. Infinite Ramsey theorem for pairs and finite tuples.
2. Finite Ramsey theorem if not already owned by a combinatorics module.
3. Erdos-Szekeres theorem card, with future combinatorics ownership noted.
4. Partition relation notation and basic monotonicity.
5. Erdos-Rado theorem card.
6. Delta-system lemma, imported from cardinal/cofinality milestone if proved there.
7. Fodor pressing-down lemma and stationary reflection interfaces.
8. Tree property, Aronszajn trees, Suslin trees, and weak compactness connections.
9. Square, diamond, and club guessing theorem cards.

Deliverables:

- Infinite-combinatorics theorem cards and initial formal proofs.
- Shared stationary/club lemmas with `SET-10`.
- Clear dependency route to forcing and large cardinals for independence-sensitive
  combinatorics.

Acceptance criteria:

- Finite combinatorics is not duplicated if another roadmap owns it later.
- Partition theorem statements spell out arity, color count, and target homogeneous set.
- Independence-sensitive claims are marked `L2/L3`.

## SET-21 Boolean Algebras, Ultrafilters, And Stone Duality

Status: planned.

Depends: `SET-01`, `SET-07`, topology roadmap for Stone-space statements.

Target modules:

- `Proofs.Ai.SetTheory.BooleanAlgebra`
- `Proofs.Ai.SetTheory.BooleanAlgebra.Filter`
- `Proofs.Ai.SetTheory.BooleanAlgebra.Stone`

Theorem order:

1. Boolean algebra as an abstract structure and as subsets of a carrier.
2. Ideals, filters, prime ideals, maximal ideals, and ultrafilters.
3. Boolean prime ideal theorem and ultrafilter lemma equivalence.
4. Filter extension theorem.
5. Complete Boolean algebra and regular open algebra interfaces.
6. Stone representation theorem as theorem card, then formal proof when topology support
   is sufficient.
7. Stone duality between Boolean algebras and Stone spaces as interface.
8. Boolean-valued model connection imported by forcing milestone.

Deliverables:

- Boolean algebra API reusable by forcing, topology, and algebra.
- Ultrafilter and prime-ideal principle labels with explicit choice strength.
- Stone representation bridge to topology.

Acceptance criteria:

- Abstract Boolean algebra lemmas are separated from subset-of-carrier Boolean algebra
  lemmas.
- Prime ideal and ultrafilter extension theorems record choice strength.
- Stone theorem does not proceed until topology dependencies are fixed or represented as
  explicit interfaces.

## SET-22 Model Theory Interfaces

Status: planned.

Depends: `SET-08`, `SET-11`, `SET-12`.

Target modules:

- `Proofs.Ai.SetTheory.Model.Structure`
- `Proofs.Ai.SetTheory.Model.Satisfaction`
- `Proofs.Ai.SetTheory.Model.Elementarity`

Theorem order:

1. First-order language, term, formula, structure, assignment, and satisfaction as
   structured syntax.
2. Elementary substructure and elementary embedding.
3. Isomorphism invariance of satisfaction for bounded fragments.
4. Tarski truth undefinability theorem card.
5. Lowenheim-Skolem theorem interface.
6. Skolem hull and Mostowski collapse connections.
7. Compactness theorem interface, if needed by ultraproduct and large-cardinal statements.
8. Ultraproduct and Los theorem interface.
9. Reflection theorem interface and absoluteness bridge to `SET-12`.

Deliverables:

- Structured syntax/satisfaction layer for metatheoretic set theory.
- Elementary embedding vocabulary for large cardinals.
- Model-relative theorem statement patterns for consistency and independence.

Acceptance criteria:

- Syntax is not encoded as fragile strings.
- Satisfaction theorems specify the fragment supported.
- Model-theoretic interfaces expose assumptions rather than implying full metatheory.

## SET-23 Packaging, Promotion, And Cross-Roadmap Reuse

Status: planned.

Depends: all applicable milestones.

Target modules:

- `Proofs.Ai.SetTheory`
- `Proofs.Ai.SetTheory.Prelude`
- `Proofs.Ai.SetTheory.Package`

Theorem order:

1. Stabilize import graph from elementary set theory upward.
2. Create a set theory prelude containing only low-risk `L0/L1` infrastructure.
3. Keep advanced theorem cards out of the prelude unless they are pure interfaces.
4. Audit axiom reports by module family.
5. Export theorem names needed by topology, measure, statistics, analysis, algebra, and
   model theory.
6. Run closure audit and promotion readiness checks before moving any theorem into a
   standalone standard library package.

Deliverables:

- Set theory module index.
- Stable prelude policy.
- Cross-roadmap import map and promotion checklist.

Acceptance criteria:

- `Proofs.Ai.SetTheory.Prelude` does not import global choice, ZFC, forcing, or large
  cardinal packages by accident.
- Axiom reports are reviewed before promotion.
- Package verification succeeds for any promoted closure.

## Recommended First Execution Queue

The first formalization pass should avoid advanced metatheory and build a foundation that
other roadmaps can import quickly.

| Queue | Target | Milestone | Why first |
| --- | --- | --- | --- |
| `SEQ-001` | Theorem-card inventory and axiom taxonomy | `SET-00` | Prevents duplicated ownership and hidden assumptions. |
| `SEQ-002` | Subset preorder and extensional equality | `SET-01` | Base of all later set proofs. |
| `SEQ-003` | Union, intersection, difference, complement, De Morgan | `SET-01` | Immediate reuse by topology and measure. |
| `SEQ-004` | Product sets and indexed unions/intersections | `SET-01` | Needed for relations, functions, and product spaces. |
| `SEQ-005` | Relation composition and equivalence relations | `SET-02` | Needed for orders, quotients, and model theory. |
| `SEQ-006` | Partitions and quotient interface | `SET-02` | Clarifies quotient assumptions early. |
| `SEQ-007` | Functions, composition, injective/surjective/bijective | `SET-03` | Required for cardinality. |
| `SEQ-008` | Image and inverse-image laws | `SET-03` | Required by topology, measure, and algebra. |
| `SEQ-009` | Finite sets and pigeonhole | `SET-04` | Low-risk constructive theorem batch. |
| `SEQ-010` | Countable/enumerable sets and countable union variants | `SET-04` | Needed by analysis, topology, statistics. |
| `SEQ-011` | Equipotence and cardinal comparison | `SET-05` | Cardinal API before big theorems. |
| `SEQ-012` | Cantor theorem and diagonal uncountability | `SET-05` | Central named theorem with manageable dependencies. |
| `SEQ-013` | Cantor-Bernstein-Schroeder | `SET-05` | Foundational cardinal comparison theorem. |
| `SEQ-014` | Posets, chains, upper bounds, well-founded induction | `SET-06` | Prepares Zorn and ordinal work. |
| `SEQ-015` | Choice, well-ordering, Zorn equivalence web | `SET-07` | Makes later choice dependencies explicit. |
| `SEQ-016` | Ordinal basics and transfinite induction | `SET-08` | Opens hierarchy and cardinal representatives. |
| `SEQ-017` | Cumulative hierarchy and foundation consequences | `SET-09` | Prepares ZF/ZFC and rank statements. |
| `SEQ-018` | Cardinal representatives, cofinality, regularity | `SET-10` | Prepares stationary and large-cardinal statements. |
| `SEQ-019` | ZF/ZFC axiom packages and paradox boundaries | `SET-11` | Prevents confusion between set and class principles. |
| `SEQ-020` | Boolean algebra filters and ultrafilter lemma interface | `SET-21` | Supports topology and forcing routes. |

After `SEQ-020`, choose the next branch by need:

- For topology reuse: continue with `SET-19`, then selected `SET-17`.
- For CH/forcing: continue with `SET-22`, `SET-14`, then `SET-13`.
- For descriptive set theory: continue with topology/measure prerequisites, then `SET-17`.
- For large cardinals and determinacy: continue with `SET-22`, `SET-16`, then `SET-18`.

## Named Theorem Priority Index

High-priority named theorems for early formal proof:

- Cantor theorem.
- Cantor diagonal uncountability of powerset of `Nat`.
- Cantor-Bernstein-Schroeder theorem.
- Pigeonhole principle for finite sets.
- Countable union theorem variants with explicit choice strength.
- Zorn's lemma and well-ordering theorem equivalence with choice.
- Transfinite induction.
- Fodor pressing-down lemma, after stationary infrastructure exists.
- Boolean prime ideal theorem and ultrafilter lemma equivalence.

High-priority theorem cards before full proof:

- Hartogs theorem.
- Mostowski collapse.
- Godel constructibility and `V = L` consequences.
- Cohen independence of CH.
- Shoenfield absoluteness.
- Easton theorem.
- Stone representation and Stone duality.
- Suslin theorem for analytic/coanalytic sets.
- Martin-Steel projective determinacy theorem.
- Kunen inconsistency.
- Los theorem for ultraproducts.

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Hidden choice enters elementary proofs. | Axiom reports become misleading and downstream roadmaps over-assume AC. | Keep choice in `SET-07`; require theorem-level imports for choice-dependent results. |
| Quotient or function extensionality is treated as kernel behavior. | Trusted base expands accidentally. | Route quotient and extensionality through explicit law packages. |
| Class/set boundary is blurred. | Paradox statements become inconsistent or unusable. | Maintain separate set, class, and model packages from the start. |
| Independence statements are phrased as ordinary theorems of ZFC. | The library records false mathematical content. | Use model-relative theorem-card templates and relative-consistency assumptions. |
| Advanced topics block elementary reusable work. | Other roadmaps cannot import basic set lemmas. | Execute `SEQ-001` through `SEQ-020` before forcing, inner models, and large cardinals. |
| Definability or satisfaction is encoded as strings. | Metatheory cannot be checked robustly. | Build structured syntax interfaces in `SET-22`. |
| Prelude imports too much. | Ordinary users inherit global choice or large-cardinal assumptions unexpectedly. | Keep `Proofs.Ai.SetTheory.Prelude` limited to audited low-level infrastructure. |

## Decision Points

- Decide the concrete set representation for early modules: predicate-over-carrier,
  explicit finite/container representation, or ZF-style object layer. The first pass should
  favor predicate-over-carrier for elementary laws and reserve ZF objects for later
  hierarchy/model work.
- Decide whether quotient cardinals are introduced early or whether cardinal comparison is
  handled by relations first. The conservative path is relation-first.
- Decide how much classical propositional reasoning is admitted in `SET-01` through
  `SET-05`. The conservative path is constructive where possible, with classical excluded
  middle imported only for theorem families that need it.
- Decide which topology and measure modules will be stable enough for `SET-17` and
  `SET-19` imports before starting descriptive set theory or set-theoretic topology.
- Decide whether forcing and model theory will use a shared first-order syntax layer from
  `SET-22` before any deep independence theorem cards are promoted.

## Completion Definition

The roadmap is complete when:

1. Every pasted theorem is either proved, assigned a checked interface theorem, or recorded
   as an `L3` theorem card with explicit dependencies.
2. Elementary set, relation, function, cardinal, order, and ordinal modules are reusable by
   other proof roadmaps.
3. Choice, ZFC, class theory, forcing, determinacy, and large-cardinal assumptions are
   visible in theorem statements and axiom reports.
4. Source-free verification passes for all checked certificates in the set theory proof
   tree.
5. Promotion candidates pass closure audit and package verification before moving into a
   standalone standard-library package.
