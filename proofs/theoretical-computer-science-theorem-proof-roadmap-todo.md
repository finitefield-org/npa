# Theoretical Computer Science Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T12`)
- `proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
- `proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
- `proofs/optimization-theorem-proof-roadmap-todo.md`
- `proofs/coding-cryptography-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for theoretical computer
science and algorithms. It is a planning sidecar only: it does not add trusted
proof evidence, axioms, source-free certificate verdicts, or package
verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers automata, formal languages, computability interfaces,
complexity classes, reductions, data structures, algorithm-correctness
schemas, graph algorithm aliases, randomized algorithms, type and programming
language semantics, cryptographic hardness boundaries, and closure-boundary planning.

Out of scope for this task document:

- adding automata, machines, algorithms, complexity classes, random oracles, or
  programming languages as trusted kernel primitives;
- treating complexity lower bounds, hardness assumptions, or semantic
  adequacy statements as proof evidence without model-explicit certificates;
- duplicating logic/model theory, graph/combinatorics, optimization, coding,
  cryptography, or numerical algorithm ownership;
- hiding computational model, cost metric, encoding, randomness, oracle, or
  hardness assumptions in theorem-shaped interfaces;
- publicly materializing TCS modules before closure audit and package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ComputerScience.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.ComputerScience.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Graph algorithm modules already exist under `Proofs.Ai.Graph.Algorithm.*`
  for search, shortest paths, spanning trees, and flow-style work.
- Number theory already has algorithm and factoring-algorithm modules.
- Logic and model theory own formal syntax, semantics, computability, proof
  theory, and metatheoretic assumptions.
- Combinatorics owns graph algorithm theorem families; optimization owns
  optimization algorithm consequences.
- This todo owns the broad algorithm and computation-model roadmap that
  coordinates those owners and exposes complexity assumptions.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `TCS-00` inventory and namespace contract | `TCS-T00` |
| `TCS-01` finite automata and regular languages | `TCS-T01` |
| `TCS-02` context-free languages and pushdown automata | `TCS-T02` |
| `TCS-03` Turing machines and computability aliases | `TCS-T03` |
| `TCS-04` complexity classes and reductions | `TCS-T04` |
| `TCS-05` data structures and finite invariants | `TCS-T05` |
| `TCS-06` algorithm-correctness schemas | `TCS-T06` |
| `TCS-07` graph algorithm alias map | `TCS-T07` |
| `TCS-08` randomized algorithms | `TCS-T08` |
| `TCS-09` programming language semantics and types | `TCS-T09` |
| `TCS-10` cryptographic hardness boundaries | `TCS-T10` |
| `TCS-11` numerical and optimization algorithm bridges | `TCS-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `TCS-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `TCS-T01` through `TCS-T03` | `L2` for syntax, transition, language, and simulation lemmas where prerequisites exist |
| `TCS-T04` | model-explicit targets; lower bounds require explicit assumptions |
| `TCS-T05` through `TCS-T08` | `L2` trace and invariant certificates for finite algorithms |
| `TCS-T09` through `TCS-T11` | route packages or `L2` only with explicit semantics and cost models |

## Milestones

### TCS-T00 Build TCS Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory automata, languages, computability, complexity, reductions,
    data structures, algorithm correctness, randomized algorithms, semantics,
    and cryptographic hardness theorem families.
  - Assign primary homes across TCS, logic/model theory, combinatorics,
    optimization, numerical analysis, coding, and cryptography.
  - Mark computational model, encoding, cost, randomness, oracle, and
    hardness assumptions.
- Verification:
  - `rg -n "TCS-T00|automata|complexity|algorithm|reduction" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T01 Add Finite Automata And Regular Language Core

- Status: Pending
- Depends on: `TCS-T00`
- Areas: `Proofs.Ai.ComputerScience.Automata.Finite`
- Tasks:
  - Define deterministic and nondeterministic finite automata, transitions,
    accepted languages, closure operations, and regular expression interfaces.
  - Prove finite transition and closure lemmas where set and finite sequence
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ComputerScience.Automata.Finite`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ComputerScience.Automata.Finite --verified-cache authoring`

### TCS-T02 Add Context-Free Language Route

- Status: Pending
- Depends on: `TCS-T01`
- Areas: `Proofs.Ai.ComputerScience.Automata.ContextFree`
- Tasks:
  - Define grammars, parse trees, derivations, pushdown automata, and
    pumping-style route packages.
  - Split equivalence and ambiguity results before source edits if induction
    or tree prerequisites are absent.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ComputerScience.Automata.ContextFree`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### TCS-T03 Coordinate Computability And Machine Models

- Status: Pending
- Depends on: `LMT-T09`
- Areas: `Proofs.Ai.ComputerScience.Computability`
- Tasks:
  - Map Turing machine, partial recursive function, decidability, and
    reduction interfaces to logic/model-theory ownership.
  - Keep Church-Turing-style theses and encoding assumptions explicit.
- Verification:
  - `rg -n "TCS-T03|LMT-T09|Turing|computability|decidable" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/logic-model-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T04 Add Complexity Classes And Reduction Route

- Status: Pending
- Depends on: `TCS-T03`
- Areas: `Proofs.Ai.ComputerScience.Complexity`
- Tasks:
  - Define explicit time, space, nondeterministic, oracle, and reduction law
    packages for machine models.
  - Record P, NP, coNP, PSPACE, EXP, completeness, and lower-bound route
    assumptions without treating open problems as theorems.
- Verification:
  - `rg -n "TCS-T04|complexity|NP|reduction|oracle" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T05 Add Data Structure Invariant Core

- Status: Pending
- Depends on: `TCS-T06`
- Areas: `Proofs.Ai.ComputerScience.DataStructure`
- Tasks:
  - Define stacks, queues, heaps, balanced trees, union-find, and invariant
    preservation route packages.
  - Prove finite operation postconditions where state-transition evidence is
    explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ComputerScience.DataStructure`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ComputerScience.DataStructure --verified-cache authoring`

### TCS-T06 Add Algorithm Correctness Schema

- Status: Pending
- Depends on: `TCS-T00`
- Areas: `Proofs.Ai.ComputerScience.Algorithm.Trace`
- Tasks:
  - Define finite traces, preconditions, postconditions, loop invariants,
    variants, partial correctness, and termination evidence.
  - Keep cost models separate from functional correctness.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ComputerScience.Algorithm.Trace`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### TCS-T07 Coordinate Graph Algorithm Aliases

- Status: Pending
- Depends on: `TCS-T06`, `CG-T42`
- Areas: `Proofs.Ai.Graph.Algorithm.*`
- Tasks:
  - Map graph search, shortest path, spanning tree, flow, matching, and minor
    algorithm theorem families to combinatorics primary ownership.
  - Add TCS aliases only for generic trace and complexity schemas.
- Verification:
  - `rg -n "TCS-T07|CG-T42|Graph.Algorithm|shortest path|flow" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T08 Add Randomized Algorithm Route

- Status: Pending
- Depends on: `TCS-T06`, `STAT-T11`
- Areas: `Proofs.Ai.ComputerScience.Algorithm.Randomized`
- Tasks:
  - Define randomized traces, success probability, amplification, Las Vegas,
    and Monte Carlo route packages.
  - Import probability inequalities and concentration prerequisites from
    statistics.
- Verification:
  - `rg -n "TCS-T08|randomized|amplification|Monte Carlo|STAT-T11" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T09 Add Programming Language Semantics Route

- Status: Pending
- Depends on: `LMT-T11`, `CAT-T10`
- Areas: `Proofs.Ai.ComputerScience.Semantics`
- Tasks:
  - Define operational, denotational, type-safety, normalization, and
    categorical semantics route packages with explicit metatheory.
  - Keep semantic adequacy and normalization theorem prerequisites visible.
- Verification:
  - `rg -n "TCS-T09|semantics|type safety|normalization|CAT-T10" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/category-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T10 Coordinate Cryptographic Hardness Boundaries

- Status: Pending
- Depends on: `TCS-T04`, `CC-T08`
- Areas: `proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map one-way functions, reductions, adversary models, and security games
    to coding and cryptography ownership.
  - Keep hardness assumptions and random oracle assumptions explicit.
- Verification:
  - `rg -n "TCS-T10|CC-T08|hardness|adversary|random oracle" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### TCS-T11 Coordinate Numerical And Optimization Algorithm Bridges

- Status: Pending
- Depends on: `TCS-T06`, `NUM-T11`, `OPT-T07`
- Areas: `Proofs.Ai.NumericalAnalysis.*`, `Proofs.Ai.Optimization.*`
- Tasks:
  - Map generic trace schemas to numerical and optimization algorithms.
  - Keep numeric convergence and mathematical optimality conclusions primary
    in their own roadmaps.
- Verification:
  - `rg -n "TCS-T11|NUM-T11|OPT-T07|trace" proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md proofs/numerical-analysis-theorem-proof-roadmap-todo.md proofs/optimization-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `TCSQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `TCS-T00` |
| `TCSQ-002` | finite automata core | `L2` where prerequisites exist | `TCS-T01` |
| `TCSQ-003` | algorithm trace schema | `L2` finite trace certificates | `TCS-T06` |
| `TCSQ-004` | graph algorithm alias map | alias and dependency split | `TCS-T07` |
| `TCSQ-005` | complexity model boundary | model-explicit route | `TCS-T04` |

## Review Checklist

- Each theorem card states its computational model, encoding, cost metric, and
  randomness assumptions.
- Graph, optimization, numerical, logic, coding, and cryptography ownership is
  respected through imports or aliases.
- Complexity lower bounds and cryptographic hardness are never treated as
  hidden proof evidence.
- Algorithm-correctness claims distinguish functional correctness,
  termination, and cost.
- Public package work is outside this TODO until closure audit confirms stable `L2` derived
  certificates.
