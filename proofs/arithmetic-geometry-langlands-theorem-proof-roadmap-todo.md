# Arithmetic Geometry Langlands Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T14`, `BMQ-007`)
- `proofs/number-theory-theorem-proof-roadmap-todo.md`
- `proofs/commutative-algebra-theorem-proof-roadmap-todo.md`
- `proofs/homological-algebra-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for arithmetic geometry,
elliptic curves, modularity, and Langlands routes. It is a planning sidecar
only: it does not add trusted proof evidence, axioms, source-free certificate
verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers elliptic curve structural routes, heights,
Mordell-Weil, reduction, Galois representations, schemes over arithmetic
bases, etale cohomology, p-adic Hodge route packages, modular forms, modular
curves, modularity lifting, L-functions, trace formula interfaces,
automorphic representations, local/global Langlands interfaces, and promotion
planning.

Out of scope for this task document:

- adding schemes, cohomology, Galois representations, automorphic forms,
  trace formulas, or Langlands correspondences as trusted kernel primitives;
- treating major theorem interfaces such as modularity, Mordell-Weil, Weil
  conjectures, or Langlands reciprocity as proof evidence without source-free
  certificates;
- hiding conjectural status, analytic continuation, choice, cohomology, or
  field-of-definition assumptions;
- promoting unstable arithmetic-geometry modules before closure audit and
  package checks are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.ArithmeticGeometry.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for promotion, package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Existing related module trees include `Proofs.Ai.EllipticCurve.*`,
  `Proofs.Ai.ArithmeticGeometry.*`, `Proofs.Ai.ModularForms.*`,
  `Proofs.Ai.Modularity.*`, and `Proofs.Ai.Langlands.*`.
- Number theory owns elementary arithmetic, modular arithmetic, arithmetic
  functions, modular forms at the number-theory roadmap level, and p-adic
  measure prerequisites when they are not geometry-specific.
- Commutative algebra should own localization, ideals, spectra, Noetherian,
  and module foundations; homological algebra should own Ext, Tor, derived
  functors, and spectral sequences.
- Algebraic geometry needs its own broad roadmap later; this todo owns the
  arithmetic-geometry and Langlands execution plan over those foundations.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `AGL-00` inventory and namespace contract | `AGL-T00` |
| `AGL-01` elliptic curve basics and group law | `AGL-T01` |
| `AGL-02` heights, Mordell-Weil, and descent routes | `AGL-T02` |
| `AGL-03` reduction, finite fields, and local invariants | `AGL-T03` |
| `AGL-04` Galois representations and etale cohomology | `AGL-T04` |
| `AGL-05` schemes over arithmetic bases | `AGL-T05` |
| `AGL-06` p-adic Hodge and comparison route | `AGL-T06` |
| `AGL-07` modular forms and modular curves | `AGL-T07` |
| `AGL-08` modularity lifting and level lowering | `AGL-T08` |
| `AGL-09` L-functions and special values | `AGL-T09` |
| `AGL-10` trace formula and automorphic representations | `AGL-T10` |
| `AGL-11` local and global Langlands interfaces | `AGL-T11` |
| `AGL-12` packaging and promotion | `AGL-T12` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `AGL-T00` | `L0` planning and theorem-card inventory |
| `AGL-T01`, `AGL-T03` | `L2` for algebraic and finite-field structural lemmas where prerequisites exist |
| `AGL-T02`, `AGL-T04` through `AGL-T08` | route packages first unless arithmetic, cohomology, and descent prerequisites are explicit |
| `AGL-T09` through `AGL-T11` | interface audit and conjectural/theorem-status map before source edits |
| `AGL-T12` | `L3` public closure and package verification |

## Milestones

### AGL-T00 Build Arithmetic Geometry And Langlands Theorem Card Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory elliptic curve, arithmetic scheme, cohomology, modularity,
    L-function, trace formula, automorphic, and Langlands theorem families.
  - Assign primary homes across number theory, commutative algebra,
    homological algebra, algebraic geometry, and this roadmap.
  - Mark conjectural, theorem-heavy, and interface-only routes explicitly.
- Deliverables:
  - Arithmetic geometry and Langlands theorem-card inventory.
- Acceptance criteria:
  - Existing high-level modules are classified by theorem level and
    promotability.
- Verification:
  - `rg -n "AGL-T00|Mordell-Weil|Langlands|modularity|Galois" proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### AGL-T01 Audit Elliptic Curve Basic And Group Law Routes

- Status: Pending
- Depends on: `CMA-T01`
- Areas: `Proofs.Ai.EllipticCurve.Basic`,
  `Proofs.Ai.EllipticCurve.GroupLaw`
- Tasks:
  - Audit existing elliptic curve basic and group law modules for law-package
    boundaries, field assumptions, and theorem level.
  - Prove or route finite algebraic group-law identities from explicit curve
    assumptions.
  - Split nonsingularity, projective closure, and coordinate-change
    prerequisites.
- Deliverables:
  - Elliptic curve basic/group-law audit and follow-up theorem queue.
- Acceptance criteria:
  - Group law facts do not hide field or nonsingularity assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GroupLaw --verified-cache authoring`

### AGL-T02 Add Heights, Descent, And Mordell-Weil Route

- Status: Pending
- Depends on: `AGL-T01`, `CMA-T06`
- Areas: `Proofs.Ai.EllipticCurve.Height`,
  `Proofs.Ai.EllipticCurve.MordellWeil`
- Tasks:
  - Audit existing height and Mordell-Weil modules.
  - Define height pairing, descent data, finitely generated group route, and
    rank statement packages.
  - Split Mordell-Weil theorem behind descent and finiteness prerequisites.
- Deliverables:
  - Heights and Mordell-Weil dependency map.
- Acceptance criteria:
  - Mordell-Weil is not accepted as a statement-only theorem.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Height --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.MordellWeil --verified-cache authoring`

### AGL-T03 Add Reduction And Finite-Field Route

- Status: Pending
- Depends on: `AGL-T01`, `NT-T67`
- Areas: `Proofs.Ai.EllipticCurve.Reduction`,
  `Proofs.Ai.EllipticCurve.FiniteField`
- Tasks:
  - Audit existing reduction and finite-field modules.
  - Define good, multiplicative, and additive reduction; local invariants; and
    point-counting route packages.
  - Connect finite-field structural facts to number theory and algebra.
- Deliverables:
  - Reduction and finite-field route module or audit notes.
- Acceptance criteria:
  - Local field and finite-field assumptions are visible.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.Reduction --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.FiniteField --verified-cache authoring`

### AGL-T04 Add Galois Representation And Etale Cohomology Route

- Status: Pending
- Depends on: `AGL-T03`, `HLA-T07`
- Areas: `Proofs.Ai.EllipticCurve.GaloisRepresentation`,
  `Proofs.Ai.ArithmeticGeometry.EtaleCohomology`
- Tasks:
  - Audit existing Galois representation and etale cohomology modules.
  - Define representation, inertia, Frobenius, l-adic cohomology, and
    comparison route packages.
  - Split cohomological construction prerequisites from arithmetic
    applications.
- Deliverables:
  - Galois representation and etale cohomology dependency map.
- Acceptance criteria:
  - Representations state coefficient field, continuity, and action
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.GaloisRepresentation --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.EtaleCohomology --verified-cache authoring`

### AGL-T05 Add Arithmetic Scheme Route

- Status: Pending
- Depends on: `CMA-T11`
- Areas: `Proofs.Ai.ArithmeticGeometry.Schemes`
- Tasks:
  - Audit arithmetic scheme modules against commutative algebra and future
    algebraic-geometry owners.
  - Define schemes over bases, morphisms, fibers, integral models, and
    rational points route packages.
  - Split scheme construction and sheaf assumptions.
- Deliverables:
  - Arithmetic scheme route module or audit notes.
- Acceptance criteria:
  - Scheme structural facts import commutative-algebra spectrum prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.Schemes --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.RationalPoints --verified-cache authoring`

### AGL-T06 Add p-adic Hodge Route

- Status: Pending
- Depends on: `AGL-T04`, `HLA-T10`
- Areas: `Proofs.Ai.ArithmeticGeometry.PadicHodge`
- Tasks:
  - Audit the existing p-adic Hodge module.
  - Define period rings, crystalline, de Rham, semistable representation
    route packages, and comparison theorem prerequisites.
  - Keep comparison theorems as route packages until cohomology foundations
    exist.
- Deliverables:
  - p-adic Hodge dependency map.
- Acceptance criteria:
  - No comparison theorem is accepted without explicit period-ring and
    cohomology prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ArithmeticGeometry.PadicHodge --verified-cache authoring`
  - `rg -n "p-adic|crystalline|semistable|comparison" proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

### AGL-T07 Add Modular Forms And Modular Curve Route

- Status: Pending
- Depends on: `NT-T49`, `NT-T50`, `AGL-T05`
- Areas: `Proofs.Ai.ModularForms.*`
- Tasks:
  - Audit modular forms basic, q-expansion, Hecke, and modular curve modules.
  - Define q-expansion, Hecke operator, modular curve, cusp, and eigenform
    route packages.
  - Coordinate number-theory modular-form ownership and arithmetic-geometry
    modular-curve ownership.
- Deliverables:
  - Modular forms and modular curve route map.
- Acceptance criteria:
  - Modular curve facts state scheme and compactification prerequisites.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.Basic --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.ModularForms.ModularCurve --verified-cache authoring`

### AGL-T08 Add Modularity Lifting And Level Route

- Status: Pending
- Depends on: `AGL-T04`, `AGL-T07`, `NT-T52`
- Areas: `Proofs.Ai.Modularity.*`
- Tasks:
  - Audit modularity lifting, Ribet, level lowering, and semistable modules.
  - Define deformation rings, Hecke algebras, level lowering, semistability,
    and lifting theorem route packages.
  - Split theorem-heavy routes into explicit prerequisites.
- Deliverables:
  - Modularity dependency map.
- Acceptance criteria:
  - Modularity lifting is not used as an axiom for elliptic curve results.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.Lifting --verified-cache authoring`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Modularity.LevelLowering --verified-cache authoring`

### AGL-T09 Add L-Function And Special Value Route

- Status: Pending
- Depends on: `AGL-T07`, `ANA-T29`
- Areas: `Proofs.Ai.EllipticCurve.LFunction`
- Tasks:
  - Audit elliptic curve L-function modules.
  - Define Euler products, analytic continuation route, functional equation
    route, special values, and BSD statement boundaries.
  - Separate analytic prerequisites from algebraic consequences.
- Deliverables:
  - L-function route map.
- Acceptance criteria:
  - Analytic continuation and functional equation assumptions are explicit.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.EllipticCurve.LFunction --verified-cache authoring`
  - `rg -n "L-function|Euler product|functional equation|BSD" proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

### AGL-T10 Add Trace Formula And Automorphic Route

- Status: Pending
- Depends on: `AGL-T07`, `AGL-T09`
- Areas: `Proofs.Ai.Langlands.TraceFormula`
- Tasks:
  - Audit trace formula modules.
  - Define automorphic representations, test functions, orbital integrals,
    trace formula route packages, and stabilization prerequisites.
  - Split analytic, harmonic-analysis, and measure-theory prerequisites.
- Deliverables:
  - Trace formula and automorphic representation route map.
- Acceptance criteria:
  - Trace formula use states measure, harmonic analysis, and convergence
    assumptions.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Langlands.TraceFormula --verified-cache authoring`
  - `rg -n "trace formula|automorphic|orbital|stabilization" proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

### AGL-T11 Add Langlands Interface Audit

- Status: Pending
- Depends on: `AGL-T04`, `AGL-T10`
- Areas: `Proofs.Ai.Langlands.Interface`
- Tasks:
  - Audit existing Langlands interface modules.
  - Split local Langlands, global Langlands, functoriality, reciprocity, and
    compatibility with L-functions.
  - Mark conjectural or theorem-heavy statements explicitly.
- Deliverables:
  - Langlands interface theorem-status map.
- Acceptance criteria:
  - Conjectural status and theorem status are visible in each card.
- Verification:
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Langlands.Interface --verified-cache authoring`
  - `rg -n "Langlands|functoriality|reciprocity|conjectural" proofs/arithmetic-geometry-langlands-theorem-proof-roadmap-todo.md`

### AGL-T12 Promote Stable Arithmetic Geometry Closures

- Status: Pending
- Depends on: selected stable `AGL-T01` through `AGL-T11` batches
- Areas: `proofs/manifest.toml`, `proofs/npa-package.toml`,
  `proofs/generated/*`
- Tasks:
  - Run closure audits for stable arithmetic-geometry modules.
  - Update public package metadata only at promotion.
  - Record excluded theorem-heavy, analytic, and conjectural routes.
- Deliverables:
  - Verified arithmetic-geometry closure ready for `npa-mathlib` promotion.
- Acceptance criteria:
  - Axiom reports and package checks are clean for the promoted closure.
- Verification:
  - `./scripts/check-corpus-authoring.sh`
  - `./scripts/check-corpus-package.sh`
  - `./scripts/check-corpus-full.sh`

## First Execution Queue

| Queue item | First deliverable | Target level | Primary task |
| --- | --- | --- | --- |
| `AGLQ-001` | theorem-card inventory and theorem-status map | `L0` | `AGL-T00` |
| `AGLQ-002` | elliptic curve basic/group-law audit | `L2` for finite algebraic facts | `AGL-T01` |
| `AGLQ-003` | reduction and finite-field route | `L2` where prerequisites exist | `AGL-T03` |
| `AGLQ-004` | Galois representation and etale route split | audit or route package | `AGL-T04` |
| `AGLQ-005` | arithmetic scheme route split | route package first | `AGL-T05` |
| `AGLQ-006` | modular forms/modular curve ownership map | audit or aliases | `AGL-T07` |
| `AGLQ-007` | modularity dependency map | theorem-heavy route map | `AGL-T08` |
| `AGLQ-008` | Langlands interface audit | theorem-status map | `AGL-T11` |

## Review Checklist

- Theorem-heavy and conjectural routes are visibly separated from `L2`
  certificates.
- Commutative algebra, homological algebra, algebraic geometry, and number
  theory prerequisites are imported from their owners.
- Existing high-level modules are audited before promotion or downstream
  reliance.
- Verification commands stay local until promotion or package metadata changes.
