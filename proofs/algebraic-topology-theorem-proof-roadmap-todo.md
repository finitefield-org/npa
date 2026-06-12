# Algebraic Topology Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T15`)
- `proofs/topology-theorem-proof-roadmap-todo.md`
- `proofs/homological-algebra-theorem-proof-roadmap-todo.md`
- `proofs/category-theory-theorem-proof-roadmap-todo.md`
- `proofs/differential-geometry-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated long-range todo for algebraic
topology beyond the general topology roadmap. It is a planning sidecar only:
it does not add trusted proof evidence, axioms, source-free certificate
verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers the audit and execution split for fundamental groups,
homotopy, singular homology and cohomology, simplicial and CW complexes,
spectra, stable homotopy, generalized cohomology, spectral sequences,
characteristic classes, K-theory, bordism, obstruction theory, fibrations, and
promotion planning.

Out of scope for this task document:

- adding homotopy types, quotient spaces, spectra, infinity categories, or
  generalized cohomology theories as trusted kernel primitives;
- duplicating the topology roadmap's current general topology and algebraic
  topology milestone ownership without an explicit alias map;
- treating spectral sequences, stable homotopy, or generalized cohomology as
  theorem-shaped assumptions for downstream geometry;
- hiding quotient, choice, universe, model-category, coherence, or homotopy
  coherence assumptions;
- promoting algebraic-topology modules before closure audit and package checks
  are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.Algebraic.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Topology.Algebraic.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- The topology roadmap already covers many algebraic topology modules,
  including fundamental group, homotopy, homology, cohomology, simplicial
  complexes, CW complexes, covering spaces, and manifold-facing routes.
- Homological algebra owns chain complexes, homology algebra, exact
  sequences, Ext, Tor, derived functors, and spectral-sequence prerequisites.
- Category theory owns higher-category, model-category, and stable-category
  interfaces.
- Differential geometry owns smooth manifolds, de Rham, characteristic-class,
  and geometric topology inputs when they use smooth structure.
- This todo is an ownership and long-range execution layer; it should not
  reassign already-owned topology work without an explicit alias entry.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `AT-00` inventory and topology ownership audit | `AT-T00` |
| `AT-01` fundamental group and homotopy aliases | `AT-T01` |
| `AT-02` singular homology and cohomology audit | `AT-T02` |
| `AT-03` simplicial and CW algebraic core | `AT-T03` |
| `AT-04` chain-level algebra and exact sequences | `AT-T04` |
| `AT-05` spectra and stable homotopy route | `AT-T05` |
| `AT-06` generalized cohomology theories | `AT-T06` |
| `AT-07` spectral sequences | `AT-T07` |
| `AT-08` characteristic classes and K-theory | `AT-T08` |
| `AT-09` bordism and cobordism routes | `AT-T09` |
| `AT-10` obstruction theory and fibrations | `AT-T10` |
| `AT-11` geometry and physics bridge aliases | `AT-T11` |
| `AT-12` packaging and promotion | `AT-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `AT-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `AT-T01` through `AT-T04` | audit existing topology modules; `L2` for chain and simplicial algebra where prerequisites exist |
| `AT-T05` through `AT-T10` | dependency maps first unless category and homological prerequisites are explicit |
| `AT-T11` | alias map only unless lower-level proofs exist |
| `AT-T12` | `L3` public closure and package verification |

## Milestones

### AT-T00 Build Algebraic Topology Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/algebraic-topology-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory existing topology modules and theorem cards for homotopy,
    fundamental group, homology, cohomology, simplicial, CW, covering,
    manifold, and stable-homotopy-shaped work.
  - Decide which items remain primary in the topology roadmap and which need
    long-range algebraic topology aliases.
  - Mark quotient, choice, universe, homotopy-coherence, and model-category
    assumptions.
- Verification:
  - `rg -n "AT-T00|homotopy|homology|cohomology|spectra" proofs/algebraic-topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T01 Coordinate Fundamental Group And Homotopy Aliases

- Status: Pending
- Depends on: `AT-T00`, `TOP-T34`
- Areas: `Proofs.Ai.Topology.FundamentalGroup`,
  `Proofs.Ai.Topology.Homotopy`
- Tasks:
  - Audit fundamental group, homotopy, homotopy equivalence, and covering
    aliases for primary topology ownership.
  - Add algebraic topology aliases only when they clarify downstream
    execution.
- Verification:
  - `rg -n "AT-T01|TOP-T34|fundamental group|homotopy" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T02 Audit Singular Homology And Cohomology Routes

- Status: Pending
- Depends on: `AT-T00`, `TOP-T37`
- Areas: `Proofs.Ai.Topology.Homology.*`,
  `Proofs.Ai.Topology.Cohomology.*`
- Tasks:
  - Audit singular homology, exactness, computation, cohomology, cup product,
    and duality modules for statement level and assumptions.
  - Split theorem-shaped cohomology assumptions into chain-level blockers
    before new reuse.
- Verification:
  - `rg -n "AT-T02|TOP-T37|singular homology|cup product|duality" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T03 Add Simplicial And CW Algebraic Core

- Status: Pending
- Depends on: `AT-T02`, `TOP-T39`
- Areas: `Proofs.Ai.Topology.SimplicialComplex`,
  `Proofs.Ai.Topology.CWComplex.*`
- Tasks:
  - Define simplicial chain, cellular chain, boundary, subdivision, and
    cellular approximation route packages.
  - Prove finite chain-boundary and cellular algebra lemmas where prerequisites
    exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Topology.SimplicialComplex`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### AT-T04 Add Chain-Level Algebra And Exact Sequence Bridges

- Status: Pending
- Depends on: `AT-T03`, `HLA-T03`
- Areas: `Proofs.Ai.Topology.Algebraic.Chain`
- Tasks:
  - Map long exact sequences, excision routes, Mayer-Vietoris, and relative
    homology to homological algebra prerequisites.
  - Prove chain-level naturality and exactness lemmas where possible.
- Verification:
  - `rg -n "AT-T04|HLA-T03|Mayer-Vietoris|excision|exact sequence" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/homological-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T05 Add Spectra And Stable Homotopy Route

- Status: Pending
- Depends on: `AT-T04`, `CAT-T10`
- Areas: `Proofs.Ai.Topology.StableHomotopy`
- Tasks:
  - Record spectrum, suspension, loop, stable equivalence, and stable homotopy
    group route packages with explicit higher-category assumptions.
  - Keep stable homotopy results as dependency maps until foundations exist.
- Verification:
  - `rg -n "AT-T05|spectra|stable homotopy|CAT-T10" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/category-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T06 Add Generalized Cohomology Route

- Status: Pending
- Depends on: `AT-T05`
- Areas: `Proofs.Ai.Topology.GeneralizedCohomology`
- Tasks:
  - Define Eilenberg-Steenrod-style axioms, representability interfaces, and
    ordinary cohomology comparison routes.
  - Avoid using generalized cohomology axioms as proof evidence for concrete
    theories without stated assumptions.
- Verification:
  - `rg -n "AT-T06|generalized cohomology|Eilenberg-Steenrod|representability" proofs/algebraic-topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T07 Add Spectral Sequence Route

- Status: Pending
- Depends on: `AT-T04`, `HLA-T10`
- Areas: `Proofs.Ai.Topology.SpectralSequence`
- Tasks:
  - Define filtered complexes, pages, differentials, convergence, and
    comparison route packages by importing homological algebra.
  - Split Serre, Leray, Atiyah-Hirzebruch, and Adams routes into explicit
    theorem cards.
- Verification:
  - `rg -n "AT-T07|HLA-T10|spectral sequence|Serre|Adams" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/homological-algebra-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T08 Add Characteristic Classes And K-Theory Route

- Status: Pending
- Depends on: `AT-T06`, `DG-T10`
- Areas: `Proofs.Ai.Topology.KTheory`,
  `Proofs.Ai.Topology.CharacteristicClass`
- Tasks:
  - Define Chern, Stiefel-Whitney, Euler, Pontryagin, vector-bundle, and
    K-theory route packages with explicit bundle prerequisites.
  - Coordinate smooth-bundle facts with differential geometry ownership.
- Verification:
  - `rg -n "AT-T08|DG-T10|K-theory|Chern|Pontryagin" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/differential-geometry-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T09 Add Bordism And Cobordism Route

- Status: Pending
- Depends on: `AT-T08`, `DG-T06`
- Areas: `Proofs.Ai.Topology.Bordism`
- Tasks:
  - Define bordism, cobordism, Thom-style, and manifold-boundary route
    packages with explicit smooth or topological manifold assumptions.
  - Split orientation, transversality, and compactness prerequisites.
- Verification:
  - `rg -n "AT-T09|bordism|cobordism|Thom|transversality" proofs/algebraic-topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T10 Add Obstruction Theory And Fibration Route

- Status: Pending
- Depends on: `AT-T04`, `AT-T07`
- Areas: `Proofs.Ai.Topology.Fibration`
- Tasks:
  - Define fibration, cofibration, lifting, obstruction cocycle, and
    Postnikov-style route packages.
  - Keep obstruction theory results behind cohomology and homotopy
    prerequisites.
- Verification:
  - `rg -n "AT-T10|fibration|cofibration|obstruction|Postnikov" proofs/algebraic-topology-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T11 Coordinate Geometry And Physics Bridge Aliases

- Status: Pending
- Depends on: `AT-T08`, `GEO-T11`, `MP-T08`
- Areas: `proofs/geometry-theorem-proof-roadmap-todo.md`,
  `proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map characteristic classes, gauge-theory interfaces, symplectic uses, and
    topological quantum field route packages to primary owners.
  - Keep physical postulates and field-theory axioms out of proof evidence.
- Verification:
  - `rg -n "AT-T11|GEO-T11|MP-T08|gauge|characteristic" proofs/algebraic-topology-theorem-proof-roadmap-todo.md proofs/geometry-theorem-proof-roadmap-todo.md proofs/mathematical-physics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AT-T12 Promote Stable Algebraic Topology Closures

- Status: Pending
- Depends on: selected stable `AT-T01` through `AT-T11` batches
- Areas: `npa-mathlib` promotion candidates
- Tasks:
  - Promote only closure-audited `L2` chain, homotopy, or cohomology theorem
    closures.
  - Keep stable homotopy, spectral sequence, and generalized cohomology route
    packages out of public materialization until proven.
- Verification:
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `ATQ-001` | topology ownership and duplicate-owner audit | `L0` | `AT-T00` |
| `ATQ-002` | fundamental group and homotopy alias map | alias and audit | `AT-T01` |
| `ATQ-003` | singular homology and cohomology audit | audit then `L2` upgrades | `AT-T02` |
| `ATQ-004` | simplicial and CW chain core | `L2` where prerequisites exist | `AT-T03` |
| `ATQ-005` | spectral sequence dependency split | route package first | `AT-T07` |

## Review Checklist

- Existing topology roadmap ownership is preserved unless this file records an
  explicit alias or dependency split.
- Chain and simplicial algebra are separated from stable homotopy and
  generalized cohomology interfaces.
- Quotient, choice, universe, coherence, and model-category assumptions are
  visible.
- Geometry and physics bridges do not use physical or smooth-manifold
  assumptions as hidden proof evidence.
- Promotion is deferred until closure audit confirms stable `L2` derived
  certificates.
