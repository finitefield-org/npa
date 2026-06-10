The recommendation is to design this not as an ordinary wiki, but as a
**multilingual, proof-aware MathWiki centered on a mathematical knowledge
graph**.

If you center the design on "pages" from the beginning, multilingual support,
theorem dependencies, formal proofs, multiple proofs of the same theorem,
proofs in different foundations, and notation differences become painful later.
The center should not be pages, but **entities for definitions, theorems,
proofs, concepts, notation, references, and formal proof certificates**.

In one sentence, this is the target shape.

```text
MathWiki = multilingual articles + mathematical knowledge graph + proof database + formal proof registry
```

---

# 1. Existing Projects To Learn From

There is a lot to learn from existing examples.

ProofWiki describes itself as an "online compendium of mathematical proofs" and
aims to collect, collaboratively edit, and classify proofs. As of the top page
display in March 2026, it showed 29,443 proofs and 35,139 definitions. This is
a good precedent showing that a proof-centered wiki can work. ([ProofWiki][1])

The Stacks Project is an open source textbook and reference for algebraic
geometry. Its design emphasizes online browsing, search, and hyperlinks that
let readers follow dependent lemmas and theorems. It also explains that results
receive permanent tags. This shows the importance of **stable IDs** in a
mathematical site. ([The Stacks Project][2])

Kerodon also adopts a Stacks Project style tagging scheme. It explains a design
where definitions, lemmas, theorems, propositions, examples, sections, formulas,
and similar objects receive stable tags that keep pointing to the same
mathematical object even if the content moves. This is essential for this site
as well. ([Kerodon][3])

Mathlib is Lean's community-driven formalized mathematics library, and the Lean
official site describes it as having over two million lines of formalized
mathematics. In other words, long term, the design should assume connection to
formal proof libraries, not only an informal explanatory wiki. ([Lean
Language][4])

For multilingual support and structured data, Wikibase and MediaWiki Translate
are useful references. Wikibase is described as a mechanism for storing and
organizing collaboratively editable information in a machine-consumable form
that is easy to multilingualize and share as Linked Open Data. ([MediaWiki][5])
The MediaWiki Translate extension provides in-wiki translation, proofreading,
translation memory, machine-translation assistance, and related workflows.
([MediaWiki][6])

---

# 2. Most Important Design Principle

The first principle to decide is this.

```text
Be entity-centered, not page-centered.
```

Bad design:

```text
/ja/right-identity-of-addition-on-natural-numbers
/en/Additive_identity_of_natural_numbers
/fr/...
```

With this design, each language page becomes a separate object, making it
impossible to manage whether they refer to the same theorem or subtly different
claims.

Preferred design:

```text
Entity: T00001234
type: theorem
canonical_name: Nat.add_zero
formal_statement: ∀ n : Nat, n + 0 = n

labels:
  ja: Right identity of addition on natural numbers
  en: Right identity of addition on natural numbers
  fr: Élément neutre à droite de l’addition des entiers naturels

pages:
  ja: /ja/theorem/T00001234
  en: /en/theorem/T00001234
  fr: /fr/theorem/T00001234
```

In short: **one mathematical object, multiple displays and languages**.

---

# 3. Basic Site Structure

In the final form, it is best to separate the system into the following four
layers.

```text
[4] Multilingual web display layer
    articles, search, browsing, translation, comments, review

[3] Mathematical knowledge graph layer
    definitions, theorems, proofs, dependencies, references, classification

[2] Proof / formalization layer
    human-facing proofs, Lean/Rocq/NPA proofs, proof certificates

[1] Persistent ID / version / audit layer
    entity IDs, hashes, history, licenses, review status
```

This separation matters. Web pages can be rebuilt later, but **mathematical
entity IDs, theorem dependencies, proof history, and formal proof
correspondence** are very hard to repair after the fact.

---

# 4. Entity Design

## 4.1 Entity Types

At minimum, provide the following types.

```text
Concept
  mathematical concept. Examples: group, topological space, continuous map

Definition
  definition. Examples: definition of group, definition of compactness

Theorem
  theorem, proposition, lemma, corollary

Proof
  human-facing proof

FormalProof
  formal proof in Lean / Rocq / NPA, etc.

Example
  example

Counterexample
  counterexample

Notation
  notation

Construction
  construction. Examples: quotient group, tensor product, direct product

TheoryContext
  underlying theory, foundation, axiom system

Reference
  reference

Person
  mathematician, author

Topic
  field, classification

Problem
  open problem, exercise
```

Assign a stable ID to everything.

```text
C00000001  Concept
D00000001  Definition
T00000001  Theorem
P00000001  Proof
F00000001  FormalProof
N00000001  Notation
R00000001  Reference
```

What should be learned from tag systems such as the Stacks Project and Kerodon
is that **permanent references** to mathematical objects must be designed in
from the beginning. ([The Stacks Project][2])

---

# 5. Ideal Theorem Page Structure

Theorem pages should not be plain prose pages; they should hold structured
information.

Example:

```text
T00001234: Nat.add_zero
```

Page structure:

```text
1. Name
   Japanese name, English name, aliases, symbolic names

2. Statement
   human-facing text
   formal text
   definitions used
   prerequisites

3. Context
   foundation
   target field
   required assumptions
   axiom dependencies

4. Proofs
   Proof A: elementary proof
   Proof B: proof by induction
   Proof C: generalization from algebraic structures

5. Formal proofs
   Lean
   Rocq
   NPA
   proof certificate hash
   axioms used
   import version

6. Related items
   generalizations
   specializations
   corollaries
   converses
   similar theorems
   counterexamples

7. Dependencies
   definitions and lemmas used by this theorem
   theorems that use this theorem

8. References
   first appearance
   textbooks
   papers
   reference URLs

9. Multilingual
   translation status for each language
   terminology correspondence
   translation review status
```

---

# 6. Theorem Data Model

For example, a theorem is represented like this.

```json
{
  "id": "T00001234",
  "type": "Theorem",
  "canonical_name": "Nat.add_zero",
  "labels": {
    "ja": "Right identity of addition on natural numbers",
    "en": "Right identity of addition on natural numbers"
  },
  "aliases": {
    "ja": ["n + 0 = n"],
    "en": ["addition by zero"]
  },
  "statement": {
    "informal": {
      "ja": "For every natural number n, n + 0 = n.",
      "en": "For every natural number n, n + 0 = n."
    },
    "formal_latex": "\\forall n \\in \\mathbb{N},\\ n + 0 = n",
    "formal_ast": {
      "language": "NPA-Core",
      "hash": "sha256:..."
    }
  },
  "context": {
    "foundation": "ConstructiveTypeTheory",
    "requires": ["D00000001"],
    "axioms_allowed": []
  },
  "classification": {
    "msc": ["03E", "11A"],
    "topics": ["NaturalNumbers", "Arithmetic"]
  },
  "proofs": ["P00004567", "P00004568"],
  "formal_proofs": ["F00000091"],
  "dependencies": ["D00000001", "T00000002"],
  "used_by": ["T00002001", "T00002002"],
  "status": "formally_verified"
}
```

This is close to the Wikidata/Wikibase idea: concepts and objects are items,
statements are recorded as property-value data, and context is added where
needed with qualifiers, references, and rank. Wikidata statements are described
as property-value pairs about items, contextualized with qualifiers and
references. ([Wikidata][7])

---

# 7. Proof Model

Separate proofs from theorems.

The reason is that one theorem can have multiple proofs.

```text
T00001234
  ├── P00000001: proof by induction
  ├── P00000002: proof from a general monoid theorem
  └── P00000003: proof generated from a formal proof
```

Proof data:

```json
{
  "id": "P00000001",
  "type": "Proof",
  "proves": "T00001234",
  "method": ["induction"],
  "difficulty": "beginner",
  "language_neutral_steps": [
    {
      "id": "s1",
      "kind": "intro",
      "uses": []
    },
    {
      "id": "s2",
      "kind": "induction",
      "on": "n"
    },
    {
      "id": "s3",
      "kind": "rewrite",
      "uses": ["D00000001"]
    }
  ],
  "localized_text": {
    "ja": "...",
    "en": "..."
  },
  "dependencies": ["D00000001", "T00000002"],
  "status": "reviewed"
}
```

Proof text is also translated, but **proof step IDs are language-independent**.
This makes it possible to detect cases where proof steps are missing in only
one language.

---

# 8. Connection To Formal Proofs

If the eventual goal is "all mathematical theorems", connections to formal
proofs should be built in from the beginning.

Formal proof entity:

```json
{
  "id": "F00000091",
  "type": "FormalProof",
  "proves": "T00001234",
  "system": "Lean",
  "system_version": "4.x",
  "library": "Mathlib",
  "formal_statement": "theorem Nat.add_zero ...",
  "proof_code_ref": "git:...",
  "certificate_hash": "sha256:...",
  "kernel_checked": true,
  "axioms_used": [],
  "imports": [
    {
      "module": "Mathlib.Data.Nat.Basic",
      "hash": "sha256:..."
    }
  ],
  "status": "verified"
}
```

The important point is to treat formal proofs not as "reference links", but as
verified evidence attached to theorem entities. Mathlib has grown into a large
library of formalized mathematics, and maintaining correspondence tables with
such external formal libraries has high long-term value. ([Lean Language][4])

However, there are many theorems without formal proofs. Therefore page status
should be staged.

```text
draft
  draft

informal
  has informal theorem statement / proof

reviewed
  human-reviewed

formalized
  formalized in Lean/Rocq/NPA, etc.

verified
  formal proof verified by a kernel/checker

certified
  independent checker, certificate hash, and axiom report confirmed
```

---

# 9. Multilingual Design

Multilingual support should be designed in from the beginning, not added later.

## 9.1 Language-Independent IDs

Every mathematical object has a language-independent ID.

```text
T00001234
```

Each language page is a display of that ID.

```text
/ja/T00001234
/en/T00001234
/fr/T00001234
/de/T00001234
```

URLs may add slugs for SEO.

```text
/ja/theorem/T00001234/right-identity-of-addition-on-natural-numbers
/en/theorem/T00001234/right-identity-of-addition-on-natural-numbers
```

Slugs are mutable; IDs are immutable.

## 9.2 Translation Units

Do not make the whole page one translation target. Mathematical articles are
structured, so split translation units.

```text
title
short description
statement
intuition
proof step 1
proof step 2
proof step 3
examples
notes
references
```

MediaWiki Translate is described as providing in-wiki translation,
proofreading, translation memory, machine-translation assistance, warnings for
unused parameters, and more. Similarly, translation units, translation memory,
and proofreading workflows should be standard features. ([MediaWiki][6])

## 9.3 Terminology Dictionary

In mathematics, translation consistency is extremely important.

Example:

```text
field
  ja: field
  fr: corps
  de: Körper

ring
  ja: ring
  fr: anneau
  de: Ring
```

Make the terminology dictionary entity-based.

```json
{
  "id": "C00000123",
  "canonical_name": "Field",
  "labels": {
    "ja": "field",
    "en": "field",
    "fr": "corps"
  },
  "disambiguation": [
    {
      "term": "field",
      "meaning": "algebraic field"
    },
    {
      "term": "field",
      "meaning": "vector field"
    }
  ]
}
```

Many concepts, such as "field", are ambiguous in English but use distinct words
in Japanese. This should not be a mere translation table; it should be a
**terminology dictionary linked to concept IDs**.

## 9.4 Translation Status

Keep status per language.

```text
missing
machine_draft
human_translated
reviewed
mathematically_reviewed
outdated
```

The especially important state is `outdated`. If the English theorem statement
or proof is updated, the Japanese version and others should automatically show
"unconfirmed after source update".

```json
{
  "entity": "T00001234",
  "language": "ja",
  "translation_status": "outdated",
  "source_revision": "rev_100",
  "translated_from_revision": "rev_093"
}
```

---

# 10. Mathematical Context Design

Mathematical theorems do not have fixed meaning without context.

Example:

```text
continuity
```

has at least the following contexts.

```text
continuous maps between topological spaces
continuous maps between metric spaces
epsilon-delta continuity of real functions
continuity in order topology
```

Therefore, definitions and theorems must always have `Context`.

```json
{
  "id": "CTX000012",
  "type": "TheoryContext",
  "name": "TopologicalSpaces",
  "assumptions": [
    "X : TopologicalSpace",
    "Y : TopologicalSpace"
  ],
  "foundation": "ConstructiveTypeTheory",
  "classical": false,
  "choice": false
}
```

On the theorem side:

```json
{
  "theorem": "T00004567",
  "context": "CTX000012",
  "statement": "A map f : X -> Y is continuous iff ..."
}
```

This makes it safe to handle different concepts that have the same Japanese
name.

---

# 11. Dependency Graph

The greatest value of this site is being able to visualize dependencies between
theorems.

Example edges:

```text
Definition uses Definition
Theorem uses Theorem
Theorem uses Definition
Proof proves Theorem
FormalProof verifies Theorem
Theorem generalizes Theorem
Theorem specializes Theorem
Theorem equivalent_to Theorem
Theorem has_counterexample Counterexample
Theorem depends_on_axiom Axiom
```

Example:

```text
Nat.zero_add
  uses Nat.rec
  uses Eq.refl
  implies Nat.add_left_identity
  used_by Nat.add_comm
```

With a dependency graph, the following become possible.

```text
- items to read before understanding this theorem
- where this theorem is used
- proof axiom dependencies
- theorem generalizations and specializations
- premise retrieval for AI
- automatic learning-course generation
- detection of broken proof dependencies
```

---

# 12. Page Display Design

Rather than cramming everything into one page, tabs are better.

```text
Overview
  theorem statement, intuition, figures, minimal explanation

Proofs
  multiple proofs, proof steps, dependent lemmas

Formal proofs
  Lean/Rocq/NPA code, certificate hash, axiom report

Dependencies
  prerequisites, used by, generalizations

Examples / counterexamples
  examples, non-examples, counterexamples

References
  books, papers, historical notes

Translation
  language status, terminology, translation diffs

Edit history
  revisions, reviewers, discussions
```

Mathematics users vary greatly in level, so explanation layers should also be
separated.

```text
Intuition
  intuitive explanation

Standard proof
  standard proof

Detailed proof
  proof with fewer omissions

Formal proof
  machine-verified proof

Research notes
  advanced comments
```

---

# 13. Search Design

Ordinary full-text search is not enough.

The following searches are needed.

```text
1. Keyword search
   "compact", "addition identity"

2. Formula search
   "x + 0 = x"

3. Type / structure search
   "?x + 0 = ?x"

4. Theorem search
   find theorems usable for the current goal

5. Dependency search
   theorems that use this theorem

6. Field search
   algebra, analysis, topology, category theory

7. Reference search
   authors, books, papers

8. Multilingual search
   search in Japanese and also return English items

9. Formal proof search
   Lean name, Rocq name, NPA entity ID
```

Put the following into the search index.

```json
{
  "entity_id": "T00001234",
  "type": "Theorem",
  "labels": {
    "ja": ["Right identity of addition on natural numbers"],
    "en": ["Right identity of addition on natural numbers"]
  },
  "symbols": ["+", "0", "="],
  "formal_patterns": ["?n + 0 = ?n"],
  "dependencies": ["Nat", "Nat.add", "Nat.zero"],
  "topics": ["Arithmetic", "NaturalNumbers"],
  "proof_status": "verified"
}
```

---

# 14. Proposed Technical Stack

## 14.1 Recommended Architecture

For the long term, the following structure is recommended.

```text
Frontend:
  Next.js / SvelteKit / Nuxt
  multilingual routing, MathJax/KaTeX, figures, search UI

Backend API:
  Rust / TypeScript / Go
  entity API, proof API, search API, translation API

Primary DB:
  PostgreSQL
  entity, revision, translation, permission, review state

Graph:
  start with PostgreSQL recursive query
  later consider Neo4j / RDF store / custom graph index

Search:
  OpenSearch / Meilisearch / Typesense
  add a dedicated index for formula search

Object storage:
  proof certificates, formal proof artifacts, generated PDFs

Version control:
  Git-like revision model
  important data has content-addressed hashes

Formal proof workers:
  sandboxed Lean / Rocq / NPA checker execution

Translation:
  translation memory
  glossary
  machine translation draft
  human review workflow

Public API:
  REST / GraphQL
  future RDF / JSON-LD / SPARQL export
```

The Wikibase data model emphasizes a clear conceptual model for the information
being handled, extensibility, flexibility, data exchange, and representation in
JSON/RDF. This MathWiki should also be designed with a similar **conceptual
model first** approach. ([MediaWiki][8])

## 14.2 Should MediaWiki Be Used?

There are two options.

### Option A: Start With MediaWiki + Wikibase + Translate

Advantages:

```text
- wiki editing, history, permissions, and multilingual support are strong from the start
- Wikibase-style structured data can be used
- the Translate extension makes translation workflows easier to build
- existing community operation knowledge exists
```

Disadvantages:

```text
- deep integration of proof certificates, formal verification, formula search, and dependency graphs can become difficult
- fully mathematics-specialized UI/UX requires work
- integration with large-scale formal proof workers needs another system
```

### Option B: Custom App + Wikibase-Style Data Model

Advantages:

```text
- mathematical entities, proofs, formal proofs, and dependency graphs can be optimized from the start
- the UI can specialize in theorems, proofs, translation, and formalization
- integration with AI proof search and the NPA prover is easier
```

Disadvantages:

```text
- wiki features, translation, history, permissions, and anti-vandalism need to be built in-house
- initial development cost is high
```

The recommendation is to **design closer to Option B at first, while borrowing
concepts from MediaWiki/Wikibase**. In other words, implementation can be
custom, but the data model should be Wikibase-like.

```text
Item = mathematical entity
Statement = mathematical relation
Qualifier = context, foundation, condition
Reference = reference, formal proof, source
Rank = recommended definition, standard theorem, historical expression
```

---

# 15. Editing / Review / Permission Design

A MathWiki needs stronger quality control than Wikipedia.

## 15.1 Roles

```text
Reader
  reader

Contributor
  draft submission / correction proposal

Editor
  ordinary article editing

Reviewer
  mathematical review

Formalizer
  Lean/Rocq/NPA formalization

Translator
  translation

Translation Reviewer
  translation review

Maintainer
  field-specific maintainer

Admin
  system administration
```

## 15.2 Review Status

Give each item a status.

```text
stub
draft
needs_review
reviewed
needs_formalization
formalized
verified
certified
deprecated
merged
split
```

## 15.3 Editing Units

Allow review by structured unit, not only by whole page.

```text
theorem statement review
proof review
translation review
formal proof review
reference review
notation review
```

This makes collaboration easier for people who only want to fix Japanese
translations, only add Lean proofs, or only add references.

---

# 16. Version Control

Theorems and definitions should be changed very carefully.

## 16.1 Entity revision

Every entity has a revision.

```json
{
  "entity_id": "T00001234",
  "revision": "rev_00041",
  "statement_hash": "sha256:...",
  "modified_by": "user123",
  "modified_at": "...",
  "change_type": "proof_added"
}
```

## 16.2 Make A Different Theorem When The Claim Changes

Minor wording changes can remain the same theorem, but if the mathematical
claim changes, make it a separate entity.

Example:

```text
∀ n : Nat, n + 0 = n
```

to:

```text
∀ n : Int, n + 0 = n
```

then it is a different theorem.

Connect them on the same page as a "generalization".

```text
T00001234 specializes T00004567
```

## 16.3 Definition Versions

When a definition changes, the meaning of many theorems changes.

Therefore, definitions must always have hashes.

```text
Definition D000001 version hash
```

Theorems record which definition revision they depend on.

---

# 17. Handling Axioms And Foundations

If the site includes "all kinds of mathematics", the same theorem may have
different foundations.

```text
ZFC
ZFC + Choice
Constructive type theory
Classical type theory
HoTT / univalent foundations
Setoid-based constructive mathematics
```

Therefore, each theorem and proof has `foundation_context`.

```json
{
  "foundation": "ZFC",
  "uses_choice": true,
  "uses_excluded_middle": true,
  "uses_univalence": false
}
```

For formal proofs, attach an axiom report.

```json
{
  "axioms_used": [
    "Classical.choice",
    "Propext"
  ]
}
```

This makes it possible to distinguish constructive proofs from classical
proofs.

---

# 18. How To Use AI

AI is very useful, but the trust boundary must be explicit.

Things AI may be used for:

```text
- generating draft theorem pages
- generating proof candidates
- recommending related theorems
- drafting translations
- generating formalization candidates
- finding similar theorems
- recommending reference candidates
```

Things AI must not be trusted with:

```text
- displaying unverified proofs as verified
- changing theorem statements on its own
- marking translations as reviewed
- displaying certified status without a formal proof
- fabricating sources
```

AI outputs get status labels.

```text
ai_draft
needs_human_review
human_reviewed
formal_verified
```

---

# 19. API Design

It is best to provide a public API from the beginning.

## 19.1 Entity API

```http
GET /api/entities/T00001234
```

```json
{
  "id": "T00001234",
  "type": "Theorem",
  "labels": {
    "ja": "Right identity of addition on natural numbers",
    "en": "Right identity of addition on natural numbers"
  },
  "statement": "...",
  "proofs": ["P00000001"],
  "formal_proofs": ["F00000091"]
}
```

## 19.2 Dependency API

```http
GET /api/entities/T00001234/dependencies
```

```json
{
  "direct": ["D00000001", "T00000002"],
  "transitive": ["..."],
  "axioms": []
}
```

## 19.3 Search API

```http
GET /api/search?q=n+%2B+0+%3D+n&lang=ja
```

## 19.4 Formal proof API

```http
GET /api/entities/T00001234/formal-proofs
```

```json
{
  "formal_proofs": [
    {
      "system": "Lean",
      "status": "verified",
      "certificate_hash": "sha256:..."
    }
  ]
}
```

## 19.5 Translation API

```http
GET /api/entities/T00001234/translations
```

```json
{
  "ja": "reviewed",
  "en": "reviewed",
  "fr": "draft",
  "de": "missing"
}
```

---

# 20. License Design

Licenses should be decided at the beginning.

Recommended:

```text
Article text:
  CC BY-SA 4.0 or CC BY 4.0

Structured data:
  consider something close to CC0

Formal proof code:
  reusable licenses such as Apache-2.0 / MIT / CC0

Proof certificates:
  consider CC0 or public-domain equivalent
```

However, license compatibility is extremely important when importing content
from existing sites or books. If content from ProofWiki, nLab, Wikipedia, the
Stacks Project, Mathlib, or similar sources is used directly, check each license
and import only in compatible forms.

---

# 21. Initial MVP

All of mathematics is impossible at the start. Begin with a **small scope whose
design can survive future expansion**.

## MVP Scope

```text
Fields:
  Logic
  Set / Type basics
  Natural numbers
  Lists
  Elementary algebra
  Groups / monoids
  entry point to basic topology

Languages:
  Japanese
  English

Features:
  entity ID
  theorem page
  definition page
  proof page
  translation status
  theorem dependencies
  human review
  simple theorem search
  LaTeX/KaTeX rendering
  formal proof link
```

## Example Pages To Build In The MVP

```text
D00000001: natural numbers
D00000002: addition
T00000001: n + 0 = n
T00000002: 0 + n = n
T00000003: associativity of addition
T00000004: commutativity of addition
D00000010: monoid
T00000020: uniqueness of the monoid identity element
```

Complete the following within this small scope.

```text
- Japanese/English switching
- theorem IDs
- proof IDs
- dependencies
- translation status
- review status
- links to formal proofs
- search
```

---

# 22. Long-Term Roadmap

## Phase A: Foundation

```text
- entity model
- multilingual labels
- theorem / definition / proof pages
- revision management
- basic search
- Markdown/LaTeX editing
```

## Phase B: Mathematical Knowledge Graph

```text
- depends_on
- proves
- uses
- generalizes
- specializes
- equivalent_to
- counterexample_to
- theorem graph visualization
```

## Phase C: Formal Proof Integration

```text
- Lean/Rocq/NPA formal proof mapping
- certificate hash
- axiom report
- proof status badge
- formal proof search
```

## Phase D: Full Multilingual Support

```text
- translation memory
- terminology dictionary
- translation review
- outdated translation detection
- multilingual SEO
```

## Phase E: AI Assistance

```text
- proof draft generation
- translation drafts
- related theorem recommendation
- formalization candidate generation
- similar theorem search
```

## Phase F: Large-Scale Expansion

```text
- Mathlib mapping
- textbook import
- MSC classification
- advanced formula search
- public API
- dataset export
- RDF/JSON-LD export
```

---

# 23. Failures To Avoid Most

These should be avoided.

```text
- creating separate theorem pages per language
- not giving theorems stable IDs
- embedding theorem statements and proofs in the same unstructured body text
- not managing definition versions
- fixing proofs and theorems as one-to-one
- ignoring differences between classical axioms, choice, and constructive proofs
- leaving translations as independent articles
- treating formal proofs as mere external links
- treating AI-generated text as reviewed
- searching only page bodies and postponing formula / dependency search
```

The most dangerous mistake is starting with an ordinary blog/wiki-like page
structure. Adding a knowledge graph, multilingual synchronization, and formal
proof support later is extremely difficult.

---

# 24. Recommended Final Design

In the end, the system should aim for this shape.

```text
MathWiki Core:
  mathematical entity DB
  theorems, definitions, proofs, references, notation, classification

MathWiki Graph:
  dependencies, directed graph, generalization, specialization, usage relations

MathWiki Proof:
  human-facing proofs
  formal proofs
  proof certificates
  axiom report

MathWiki Translate:
  multilingual labels
  translation units
  terminology dictionary
  translation memory
  review status

MathWiki Search:
  full-text search
  formula search
  theorem search
  dependency search
  multilingual search

MathWiki AI:
  proof candidates
  formalization candidates
  translation drafts
  related theorem recommendations

MathWiki API:
  REST / GraphQL
  JSON-LD / RDF export
  proof certificate export
```

---

# 25. Conclusion

What should be built is not merely a "mathematics version of Wikipedia".

More precisely:

```text
proof-aware mathematical encyclopedia platform centered on a multilingual
mathematical knowledge graph, integrating definitions, theorems, proofs, formal
proofs, references, and dependencies
```

That is the target.

The five core elements that should be included from the beginning are:

```text
1. language-independent stable IDs
2. an entity model separating theorems, definitions, and proofs
3. multilingual labels, translation units, and terminology dictionary
4. dependency graph
5. connections to formal proofs, certificates, and axiom reports
```

In the short term, it is fine to start as a **small Japanese/English MathWiki**.
However, the internal structure should be able to support "all mathematics,
multilingual content, formal proofs, and AI search" from the beginning.

[1]: https://proofwiki.org/wiki/Main_Page "ProofWiki"
[2]: https://stacks.math.columbia.edu/about "About—The Stacks project"
[3]: https://kerodon.net/tags?utm_source=chatgpt.com "Tags explained"
[4]: https://lean-lang.org/use-cases/mathlib/ "Mathlib: A Foundation for Formal Mathematics Research and Verification — Lean Lang "
[5]: https://www.mediawiki.org/wiki/Wikibase/Reference/en "Wikibase/Reference - MediaWiki"
[6]: https://www.mediawiki.org/wiki/Extension%3ATranslate "Extension:Translate - MediaWiki"
[7]: https://www.wikidata.org/wiki/Help%3AStatements "Help:Statements - Wikidata"
[8]: https://www.mediawiki.org/wiki/Wikibase/DataModel "Wikibase/DataModel - MediaWiki"
