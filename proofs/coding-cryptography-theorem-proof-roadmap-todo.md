# Coding Cryptography Theorem Proof Roadmap Todo

Source context:

- `proofs/broad-mathematics-coverage-todo.md` (`BMC-T13`)
- `proofs/statistics-theorem-proof-roadmap-todo.md`
- `proofs/number-theory-theorem-proof-roadmap-todo.md`
- `proofs/combinatorics-graph-theorem-proof-roadmap-todo.md`
- `proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`

This task breakdown is the first dedicated todo for coding theory,
information-theory interfaces, and cryptography. It is a planning sidecar
only: it does not add trusted proof evidence, axioms, source-free certificate
verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, this todo document, tactics, plugins, and AI
output are untrusted.

## Scope

This task list covers block codes, Hamming distance, finite-field linear
codes, code bounds, cyclic and Reed-Solomon-style routes, source and channel
coding interfaces, entropy aliases, cryptographic primitive correctness,
number-theoretic and elliptic-curve crypto aliases, security-game boundaries,
zero-knowledge and commitment route packages, and closure-boundary planning.

Out of scope for this task document:

- adding codes, entropy, finite fields, adversaries, random oracles, or
  cryptographic security games as trusted kernel primitives;
- treating security assumptions, indistinguishability claims, or unproved
  hardness assumptions as mathematical proof evidence;
- duplicating statistics information theory, number-theory crypto, finite
  geometry, design theory, or TCS complexity ownership;
- hiding finite-field, distribution, adversary, oracle, computational-model, or
  hardness assumptions in theorem-shaped interfaces;
- publicly materializing coding or crypto modules before closure audit and package checks
  are clean.

## Authoring Loop

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.CodingTheory.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.CodingTheory.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

Use package gates only for package metadata, checker
compatibility, certificate compatibility, or release work.

## Current Implementation Facts

- Cryptography modules already exist under `Proofs.Ai.Cryptography.*`,
  including elliptic-curve and number-theory-facing routes.
- Combinatorics includes finite geometry and design coding modules.
- Statistics owns divergence, information inequality, entropy, coding, Fano,
  large deviations, and learning-theory information interfaces.
- Number theory owns modular arithmetic, finite-field, elliptic-curve, and
  computational number-theory prerequisites used by cryptography.
- TCS owns complexity models, reductions, randomized algorithms, and hardness
  boundaries used by security statements.

## Roadmap Coverage Map

| Roadmap area | Covered by task milestones |
| --- | --- |
| `CC-00` inventory and namespace contract | `CC-T00` |
| `CC-01` block codes and Hamming metrics | `CC-T01` |
| `CC-02` linear codes over finite fields | `CC-T02` |
| `CC-03` coding bounds | `CC-T03` |
| `CC-04` cyclic, BCH, and Reed-Solomon routes | `CC-T04` |
| `CC-05` Shannon source and channel coding routes | `CC-T05` |
| `CC-06` entropy and divergence alias map | `CC-T06` |
| `CC-07` cryptographic primitive correctness | `CC-T07` |
| `CC-08` number-theoretic and elliptic-curve crypto aliases | `CC-T08` |
| `CC-09` security assumptions and game boundaries | `CC-T09` |
| `CC-10` zero-knowledge and commitment routes | `CC-T10` |
| `CC-11` TCS hardness and reduction bridge | `CC-T11` |

## Target Level Defaults

| Milestones | Default target level |
| --- | --- |
| `CC-T00` | `L0` planning, theorem-card inventory, and duplicate-map maintenance |
| `CC-T01` through `CC-T04` | `L2` for finite combinatorial and algebraic coding results where prerequisites exist |
| `CC-T05` through `CC-T06` | route packages or aliases to statistics information theory |
| `CC-T07` through `CC-T11` | `L2` for algebraic/protocol correctness; security claims remain assumption-explicit |

## Milestones

### CC-T00 Build Coding And Cryptography Inventory

- Status: Pending
- Depends on: None
- Areas: `proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
- Tasks:
  - Inventory block code, linear code, bound, cyclic code, source coding,
    channel coding, entropy, primitive correctness, protocol, security-game,
    and hardness theorem families.
  - Assign primary homes across coding, cryptography, statistics, number
    theory, combinatorics, and TCS.
  - Mark finite-field, probability, adversary, oracle, cost-model, and
    hardness assumptions.
- Verification:
  - `rg -n "CC-T00|Hamming|Reed-Solomon|security|entropy" proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T01 Add Block Codes And Hamming Metric Core

- Status: Pending
- Depends on: `CC-T00`
- Areas: `Proofs.Ai.CodingTheory.BlockCode`
- Tasks:
  - Define words, alphabets, block codes, Hamming distance, minimum distance,
    decoding radius, and error-detection law packages.
  - Prove finite metric and correction-radius lemmas where finite-set
    prerequisites exist.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.CodingTheory.BlockCode`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.CodingTheory.BlockCode --verified-cache authoring`

### CC-T02 Add Linear Codes Over Finite Fields

- Status: Pending
- Depends on: `CC-T01`, `LIN-T05`
- Areas: `Proofs.Ai.CodingTheory.LinearCode`
- Tasks:
  - Define generator matrices, parity-check matrices, syndrome decoding, dual
    codes, and dimension law packages.
  - Import finite-field and linear-algebra prerequisites explicitly.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.CodingTheory.LinearCode`
  - `cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring`

### CC-T03 Add Coding Bounds Route

- Status: Pending
- Depends on: `CC-T01`, `CC-T02`
- Areas: `Proofs.Ai.CodingTheory.Bounds`
- Tasks:
  - State and split Singleton, Hamming, Plotkin, Gilbert-Varshamov, and
    sphere-packing bound routes.
  - Prove finite counting components where combinatorics prerequisites exist.
- Verification:
  - `rg -n "CC-T03|Singleton|Hamming bound|Gilbert|sphere-packing" proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T04 Add Cyclic And Reed-Solomon Routes

- Status: Pending
- Depends on: `CC-T02`, `CMA-T06`
- Areas: `Proofs.Ai.CodingTheory.Cyclic`
- Tasks:
  - Define cyclic codes, BCH-style route packages, Reed-Solomon evaluation
    codes, and algebraic decoding prerequisites.
  - Split polynomial-factorization and finite-field assumptions before source
    edits.
- Verification:
  - `rg -n "CC-T04|cyclic|BCH|Reed-Solomon|finite field" proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T05 Add Shannon Source And Channel Coding Routes

- Status: Pending
- Depends on: `STAT-T59`
- Areas: `Proofs.Ai.CodingTheory.Information`
- Tasks:
  - Record source coding, channel capacity, typical-set, and channel-coding
    theorem route packages as statistics information-theory aliases.
  - Keep asymptotic probability and large-deviation prerequisites explicit.
- Verification:
  - `rg -n "CC-T05|STAT-T59|source coding|channel coding|capacity" proofs/coding-cryptography-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T06 Coordinate Entropy And Divergence Aliases

- Status: Pending
- Depends on: `STAT-T58`, `STAT-T59`
- Areas: `Proofs.Ai.Statistics.InformationTheory`
- Tasks:
  - Map entropy, conditional entropy, mutual information, KL divergence, Fano,
    and data-processing aliases to statistics ownership.
  - Add coding-specific aliases only when they preserve primary ownership.
- Verification:
  - `rg -n "CC-T06|STAT-T58|mutual information|Fano|data-processing" proofs/coding-cryptography-theorem-proof-roadmap-todo.md proofs/statistics-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T07 Add Cryptographic Primitive Correctness Core

- Status: Pending
- Depends on: `CC-T00`
- Areas: `Proofs.Ai.Cryptography.Primitive`
- Tasks:
  - Define encryption, signature, commitment, hash, and key-exchange
    correctness law packages.
  - Prove algebraic correctness statements separately from security claims.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Cryptography.Primitive`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Cryptography.Primitive --verified-cache authoring`

### CC-T08 Coordinate Number-Theoretic And Elliptic Crypto Aliases

- Status: Pending
- Depends on: `CC-T07`, `NT-T67`
- Areas: `Proofs.Ai.Cryptography.NumberTheory`,
  `Proofs.Ai.Cryptography.EllipticCurve`
- Tasks:
  - Map RSA, Diffie-Hellman, discrete log, elliptic-curve group law, and
    pairing-facing theorem families to number theory and crypto ownership.
  - Keep hardness assumptions out of algebraic correctness proofs.
- Verification:
  - `rg -n "CC-T08|NT-T67|RSA|Diffie|elliptic" proofs/coding-cryptography-theorem-proof-roadmap-todo.md proofs/number-theory-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T09 Add Security Assumption And Game Boundary

- Status: Pending
- Depends on: `CC-T07`, `TCS-T10`
- Areas: `Proofs.Ai.Cryptography.Security`
- Tasks:
  - Define adversaries, experiments, games, advantage, reductions, random
    oracles, and computational assumptions as explicit route packages.
  - Do not mark security claims as unconditional `L2` theorems unless the
    assumptions are theorem inputs.
- Verification:
  - `rg -n "CC-T09|TCS-T10|adversary|advantage|random oracle" proofs/coding-cryptography-theorem-proof-roadmap-todo.md proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T10 Add Zero-Knowledge And Commitment Routes

- Status: Pending
- Depends on: `CC-T09`
- Areas: `Proofs.Ai.Cryptography.ZeroKnowledge`
- Tasks:
  - Define completeness, soundness, zero-knowledge, simulators, commitments,
    and transcript route packages.
  - Split probabilistic and computational assumptions before source edits.
- Verification:
  - `rg -n "CC-T10|zero-knowledge|simulator|commitment|soundness" proofs/coding-cryptography-theorem-proof-roadmap-todo.md`
  - `git diff --check`

### CC-T11 Coordinate TCS Hardness And Reduction Bridge

- Status: Pending
- Depends on: `CC-T09`, `TCS-T04`
- Areas: `proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
- Tasks:
  - Map reduction correctness and complexity assumptions to TCS primary
    ownership.
  - Keep protocol correctness separate from hardness-based security.
- Verification:
  - `rg -n "CC-T11|TCS-T04|reduction|hardness|security" proofs/coding-cryptography-theorem-proof-roadmap-todo.md proofs/theoretical-computer-science-theorem-proof-roadmap-todo.md`
  - `git diff --check`

## First Execution Queue

| Queue item | First deliverable | Target level | Milestone |
| --- | --- | --- | --- |
| `CCQ-001` | theorem-card inventory and duplicate-owner map | `L0` | `CC-T00` |
| `CCQ-002` | block code and Hamming metric core | `L2` | `CC-T01` |
| `CCQ-003` | linear code finite-field route | `L2` where prerequisites exist | `CC-T02` |
| `CCQ-004` | entropy and coding alias map | alias and dependency split | `CC-T06` |
| `CCQ-005` | crypto primitive correctness boundary | `L2` for algebraic correctness | `CC-T07` |

## Review Checklist

- Finite coding results are separated from asymptotic information-theory
  route packages.
- Algebraic cryptographic correctness is separated from security claims.
- Statistics, number theory, combinatorics, and TCS ownership is respected
  through imports or aliases.
- Hardness, adversary, oracle, probability, and cost-model assumptions are
  visible.
- Public package work is outside this TODO until closure audit confirms stable `L2` derived
  certificates.
