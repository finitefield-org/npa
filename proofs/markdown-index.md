# Proof Corpus Markdown Index

Visibility: internal proof-corpus documentation.

This index organizes the Markdown files directly under `proofs/`. These files
are planning, roadmap, theorem-card, and proof-phase notes for proof-corpus
authoring. They are not trusted proof evidence; only canonical certificates that
pass source-free verification establish proof acceptance.

When adding, deleting, or renaming a top-level Markdown file under `proofs/`,
update this index in the same change.

## Entry Points

- `README.md`: proof-corpus artifact overview, trust boundary, and module bundle
  inventory.
- `markdown-index.md`: this file; top-level organization for proof-corpus
  Markdown documents.

## Coverage Planning

- `broad-mathematics-coverage-todo.md`: broad subject coverage tracker across
  the proof corpus.

## Theorem-Card Inventories

Use these files as the first stop for domains that have an extracted theorem
card list. A theorem card is an authoring inventory item, not proof evidence by
itself.

- `combinatorics-graph-theorem-cards.md`
- `measure-theory-theorem-cards.md`
- `number-theory-theorem-cards.md`
- `representation-lie-theory-theorem-cards.md`
- `statistics-theorem-cards.md`
- `topology-theorem-cards.md`

## Active Roadmap TODOs

These files track domain roadmap work that still needs authoring decisions,
proof construction, verification, or status cleanup. For proof-corpus authoring,
aim directly at `L2 Derived certificate` status; do not add new `L1` scaffolds
as a temporary substitute for missing proof prerequisites.

- `algebraic-geometry-theorem-proof-roadmap-todo.md`
- `algebraic-topology-theorem-proof-roadmap-todo.md`
- `analysis-theorem-proof-roadmap-todo.md`
- `arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
- `category-theory-theorem-proof-roadmap-todo.md`
- `coding-cryptography-theorem-proof-roadmap-todo.md`
- `combinatorics-graph-theorem-proof-roadmap-todo.md`
- `commutative-algebra-theorem-proof-roadmap-todo.md`
- `differential-geometry-theorem-proof-roadmap-todo.md`
- `geometry-theorem-proof-roadmap-todo.md`
- `homological-algebra-theorem-proof-roadmap-todo.md`
- `linear-algebra-theorem-proof-roadmap-todo.md`
- `logic-model-theory-theorem-proof-roadmap-todo.md`
- `mathematical-physics-theorem-proof-roadmap-todo.md`
- `measure-theory-theorem-proof-roadmap-todo.md`
- `number-theory-theorem-proof-roadmap-todo.md`
- `numerical-analysis-theorem-proof-roadmap-todo.md`
- `operator-functional-analysis-theorem-proof-roadmap-todo.md`
- `optimization-theorem-proof-roadmap-todo.md`
- `representation-lie-theory-theorem-proof-roadmap-todo.md`
- `set-theory-theorem-proof-roadmap-todo.md`
- `statistics-theorem-proof-roadmap-todo.md`
- `stochastic-calculus-theorem-proof-roadmap-todo.md`
- `theoretical-computer-science-theorem-proof-roadmap-todo.md`
- `topology-theorem-proof-roadmap-todo.md`

## Baseline And Historical Roadmaps

These files are retained as domain roadmap background, completed route history,
or predecessor plans. Prefer the matching `*-todo.md` file for current execution
status when both exist.

- `analysis-theorem-proof-roadmap.md`
- `combinatorics-graph-theorem-proof-roadmap.md`
- `linear-algebra-theorem-proof-roadmap.md`
- `measure-theory-theorem-proof-roadmap.md`
- `number-theory-theorem-proof-roadmap.md`
- `set-theory-theorem-proof-roadmap.md`
- `statistics-theorem-proof-roadmap.md`
- `topology-theorem-proof-roadmap.md`

## Focused Proof-Phase Notes

These files document focused theorem or route proof decomposition. Use them as
local implementation notes for the named route, not as global roadmap status.

- `correspondence-theorem-proof-phases.md`
- `first-isomorphism-proof-phases.md`
- `inner-product-to-metric-proof-phases.md`
- `inverse-implicit-function-proof-phases.md`
- `law-of-cosines-proof-phases.md`
- `pythagorean-proof-phases.md`
- `second-isomorphism-proof-phases.md`
- `third-isomorphism-proof-phases.md`

## Naming Conventions

| Pattern | Purpose | Update policy |
| --- | --- | --- |
| `*-theorem-cards.md` | Extracted theorem-card inventory for a domain. | Update when the domain theorem inventory changes. |
| `*-theorem-proof-roadmap-todo.md` | Current execution roadmap and proof-status tracker. | Update as theorem status changes. |
| `*-theorem-proof-roadmap.md` | Baseline or historical roadmap context. | Keep stable unless correcting stale guidance. |
| `*-proof-phases.md` | Focused proof decomposition for a theorem family or route. | Update when route decomposition changes. |

## Navigation Order

For a domain with multiple files, use this order:

1. Start with the `*-theorem-cards.md` file if it exists.
2. Use the matching `*-theorem-proof-roadmap-todo.md` file for current status.
3. Check the matching `*-theorem-proof-roadmap.md` file only for background or
   predecessor route context.
4. Consult `*-proof-phases.md` files only for focused route decomposition.
