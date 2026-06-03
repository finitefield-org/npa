# npa-mathlib Analysis Metric Topology Closure Audit

Date: 2026-06-03

This audit fixes the next `npa-mathlib` theorem layer after the `v0.1.20`
geometry Pythagorean closure. It selects the predicate-level metric ball,
neighborhood, local predicate, local equality, and local uniqueness route as
one public topology closure.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, meta files, theorem indexes, publish plans, release notes, and this
audit are untrusted sidecars.

## Baseline

Current package state:

- The geometry Pythagorean closure has been materialized, committed, tagged,
  and pushed in `/Users/kazuyoshitoshiya/ff/npa-mathlib` as
  `npa-mathlib v0.1.20`.
- The `npa-mathlib` release commit is
  `e329f59`.
- The local ignored `v0.1.20` release-bundle tar hash is
  `sha256:253ef90e75cc7c8f0b54c6e488b61291a327097d9bb016e6b41efe16253419dc`.

This closure should materialize as `npa-mathlib v0.1.21`. It must not change
package boundaries, registry assumptions, import identity rules, or proof
trust boundaries.

The public axiom policy remains:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

## Selected Candidate Set

The selected candidate set is:

| Corpus module | Public module | Public path | Declarations | Direct imports | Axiom surface |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` | `Mathlib/Topology/Metric/Basic/` | 6 definitions, 15 theorems | `Std.Logic.Eq`, equality reasoning | transitive `Eq.rec` |

The public namespace mapping is:

| Corpus module | Public module |
| --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` |

After public namespace materialization, the internal imports must become:

```text
Mathlib.Topology.Metric.Basic
  imports Std.Logic.Eq
  imports Mathlib.Logic.EqReasoning
```

`Std.Logic.Eq` remains a hash-pinned `npa-std v0.1.0` package import.
`Mathlib.Logic.EqReasoning` is already public in `npa-mathlib`.

The selected public surface is:

| Public module | Definitions | Theorems |
| --- | --- | --- |
| `Mathlib.Topology.Metric.Basic` | `MetricBall`, `Neighborhood`, `LocalMem`, `LocalPred`, `LocalEq`, `LocalUnique` | `metric_ball_intro`, `metric_ball_elim`, `neighborhood_intro`, `neighborhood_center`, `neighborhood_shrink`, `local_mem_intro`, `local_mem_elim`, `local_pred_intro`, `local_pred_apply`, `local_pred_shrink`, `metric_ball_mono`, `local_eq_refl`, `local_eq_symm`, `local_eq_trans`, `local_unique_apply` |

## Closure Unit Rationale

This audit keeps `Proofs.Ai.Analysis.AbstractMetricTopology` as a single-module
release because it is already a closed import slice over `Std.Logic.Eq` and
equality reasoning. It provides the metric/topology-local predicate vocabulary
that later analysis routes use, without bundling normed-space, linear-map,
derivative, fixed-point, inverse-function, or implicit-function theorem
surfaces.

The following nearby routes are intentionally split out:

| Route | Corpus modules | Reason deferred |
| --- | --- | --- |
| Analysis normed spaces | `Proofs.Ai.Analysis.AbstractNormedSpace` | Depends on vector-space foundations and introduces norm/product-norm APIs, not only local metric topology. |
| Analysis linear maps | `Proofs.Ai.Analysis.AbstractLinearMap` | Depends on normed-space and introduces bounded-linear-map and linear-isomorphism APIs. |
| Analysis derivatives | `Proofs.Ai.Analysis.AbstractDerivative` | Depends on metric topology, vector-space, normed-space, and linear-map closures. |
| Analysis fixed point | `Proofs.Ai.Analysis.AbstractFixedPoint` | Depends on metric topology, vector-space, and normed-space closures. |
| Analysis inverse/implicit function routes | `Proofs.Ai.Analysis.AbstractInverseFunction`, `Proofs.Ai.Analysis.AbstractImplicitPhi`, `Proofs.Ai.Analysis.AbstractImplicitFunction` | Later theorem routes that depend on derivative/fixed-point or derivative/linear-map APIs. |

## Namespace Decision

`Mathlib.Topology.Metric.Basic` is used because the selected surface is about
metric balls, neighborhoods, and local predicates/equality/uniqueness. It is a
topology foundation route, not an analysis theorem route. A broader
`Mathlib.Analysis.MetricTopology` name would make later analysis theorem
closures less clear.

The module uses `Basic` because it is the first public metric-topology module
and provides vocabulary rather than one final theorem family.

No declaration-name collisions were found against already released public
modules.

## Import Rewrite Table

| Source import/name | Public import/name | Status |
| --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `Mathlib.Topology.Metric.Basic` | materialize as a local public module |
| `Proofs.Ai.EqReasoning` | `Mathlib.Logic.EqReasoning` | already public |
| `Std.Logic.Eq` | `Std.Logic.Eq` | unchanged external package import |

The materialized public source, manifest, package lock, publish plan, and
downstream fixture must contain no stale route-specific
`Proofs.Ai.Analysis.AbstractMetricTopology` names.

## Axiom Policy

This closure does not widen the public axiom policy beyond the `v0.1.20`
baseline.

Materialization must keep:

```toml
[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]
```

The selected module has:

| Module | Direct axioms | Transitive axioms | Policy status |
| --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | none | `Eq.rec` | ok, already allowed |

The public module manifest entry should declare `axioms = ["Eq.rec"]` because
the checked corpus metadata records the equality-reasoning dependency. The
direct axiom report remains empty for this module; `Eq.rec` is transitive
through `Mathlib.Logic.EqReasoning`.

The theorem index is an untrusted sidecar; proof acceptance remains based on
canonical certificate bytes and verifier results.

## Hash Inputs

Corpus manifest hashes:

| Corpus module | Source hash | Certificate file hash | Export hash | Axiom report hash | Certificate hash |
| --- | --- | --- | --- | --- | --- |
| `Proofs.Ai.Analysis.AbstractMetricTopology` | `sha256:905fa70ef967459e417adea05e26a087b54c0fe8786b84f919a793410770fc4f` | `sha256:6d213ecc22a282ae10dbcd837e5437ad9d57b82c2a52d970ff9b13d1472877d6` | `sha256:afcdfd97b33939bea200e4d1e83374ea0a632b3d2e31a32d2f2d198f678352a8` | `sha256:6821508440c560fb4a20c6b8dde9b28116f41a1733e43c2dd5163cd66c37b969` | `sha256:f4ca8c631d94109fa7b6e0258a9813ed9695462c3ab193d9292544931d8f35fd` |

The public materialization must use the CLI diagnostics as the source of truth
for public hashes. Public hashes can differ from corpus hashes because module
names and imports change from `Proofs.Ai.*` to public `Mathlib.*` names.

## Corpus Verification

The selected corpus closure passed source-free module verification:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.Analysis.AbstractMetricTopology
```

Observed result:

```text
verified Proofs.Ai.Analysis.AbstractMetricTopology
verified 1 selected module(s), 3 module(s) including dependency cache
```

The selected corpus closure also passed module-local source rebuild:

```sh
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.Analysis.AbstractMetricTopology
```

Observed result:

```text
built Proofs.Ai.EqReasoning
built Proofs.Ai.Analysis.AbstractMetricTopology
wrote /Users/kazuyoshitoshiya/ff/npa/proofs/generated/ai-theorem-index.json
built Proofs.Ai.Analysis.AbstractMetricTopology (2 module(s) including import closure)
```

## Downstream Smoke Plan

The downstream smoke fixture should import the public topology closure through
vendored certificate bytes and exercise these theorem names:

- `metric_ball_mono`
- `local_eq_trans`
- `local_unique_apply`

The smoke module should remain source-free for upstream `npa-mathlib`
dependencies and should build only its own downstream certificate from source.

## Positive Gate Plan

Main package gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
```

Downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

## Negative Check Plan

Use temporary package copies outside both repositories. Do not corrupt the live
working tree.

Required negative checks:

- bad public export hash for `Mathlib.Topology.Metric.Basic` is rejected as
  `export_hash_mismatch`;
- bad public certificate hash for `Mathlib.Topology.Metric.Basic` is rejected
  as `certificate_hash_mismatch`;
- corrupted public certificate bytes for `Mathlib.Topology.Metric.Basic` are
  rejected by source-free reference verification;
- stale downstream `Mathlib.Topology.Metric.Basic` import identity is rejected
  by the downstream build gate.

## Materialization Result

Result: materialized in `npa-mathlib` v0.1.21.

Public module:

- `Mathlib.Topology.Metric.Basic`

Namespace mapping:

- `Proofs.Ai.Analysis.AbstractMetricTopology` ->
  `Mathlib.Topology.Metric.Basic`

Public module hashes:

| Field | Hash |
| --- | --- |
| source file | `sha256:e3188913d905dfb7d529806a568df95c201bef8174143e06ee37c4f1c2147860` |
| certificate file | `sha256:aacd0c14b7957f03edeb381d9241404c9da1cc6393803d2cbcfacb032f684971` |
| export | `sha256:e2c7132ac63593d59bd905ce657c8f8a2d8f66bc3a940f5f5ad9451996afbf36` |
| axiom report | `sha256:6821508440c560fb4a20c6b8dde9b28116f41a1733e43c2dd5163cd66c37b969` |
| canonical certificate | `sha256:72632a2941959b9e7ce7378ccf8627eff342630aa86fda4f43bdf6eefeb46c37` |

Downstream smoke module:

- `Downstream.MetricTopology`
- imported public theorems:
  `metric_ball_mono`, `local_eq_trans`, `local_unique_apply`

Downstream smoke hashes:

| Field | Hash |
| --- | --- |
| downstream source file | `sha256:6a3a29aa475b511b9ff48cdda5ae1011902e11b9ecb1a4146077510b333e96cb` |
| downstream manifest file | `sha256:0d183208830f0db8b4a57e014c03ca7a5d80669afd34ea2b29d8e1568b3fea9b` |
| downstream package lock file | `sha256:8639b79149e35320421cde0ee8d9171adce714f43ffcb1003df380891a9180e6` |
| downstream certificate file | `sha256:ff2f1dd61c8bdfa7150a30032b00f5fdfa82e83bf60ec35fe4ecd2e628b20d26` |
| downstream export | `sha256:e33b72aa82c9500dfb8ae3f264dd90b08f81a2a3413eaf8263b36c412de2116d` |
| downstream axiom report | `sha256:54502ead9b60479bd096ceef3ab93acb45f9474c3fc26b19c83e4bb43cff5c14` |
| downstream canonical certificate | `sha256:bba655285cc6ee14748e390d24fcf61f4e72beae8b4af0b0e1f2df608699fb26` |

Generated package artifacts:

| Artifact | Hash |
| --- | --- |
| `generated/package-lock.json` | `sha256:01cbeaee55ec056ca3f248109064b7dd16269f005ac44e6442bdc83904919435` |
| `generated/axiom-report.json` | `sha256:286e290355d12075d6712194f14bff5969ae8e73459ab50f0ea2211e816906b5` |
| `generated/theorem-index.json` | `sha256:567aa78f443600b7398fc0cb7cb3c3f788a6e0457e59f464caf74006ad40e130` |
| `generated/publish-plan.json` | `sha256:6d4712fdb8129112f544294ae41d6079d36f8f10634f485dffff2f97b53cb5f9` |
| publish plan self hash | `sha256:4bf46f427225b0728196c0f8c333e6e0402c67e25ea5c6a54fd48f7bd5d2c724` |

Release artifact:

- `target/release-artifacts/npa-mathlib-v0.1.21-release-artifacts.tar.gz`
- sha256:
  `5ab1aac8236934bf8b31737eb37a9a902fc2f52ca971a90baa7e0f4b11ffdad2`

Positive gates passed:

```sh
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root /Users/kazuyoshitoshiya/ff/npa-mathlib --check --json
cargo run -q -p npa-cli -- package check --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root /Users/kazuyoshitoshiya/ff/npa-mathlib/fixtures/downstream-smoke --json
```

Negative gates on temporary package copies:

- Public `expected_export_hash` tamper rejected by
  `package check-hashes` as `export_hash_mismatch`.
- Public `expected_certificate_hash` tamper rejected by
  `package check-hashes` as `certificate_hash_mismatch`.
- Public certificate byte corruption rejected first as
  `certificate_file_hash_mismatch`; after aligning the temporary file hash,
  `package verify-certs --checker reference` rejected it as
  `certificate_decode_failed`.
- Downstream `Mathlib.Topology.Metric.Basic` import `export_hash` tamper
  rejected by `package build-certs --check` as `export_hash_mismatch`.

Version-only downstream stale probe:

- Changing only the downstream import version string from `0.1.21` to
  `0.1.20` did not fail `package check` or `package check-hashes`.
- This is not a certificate soundness failure because downstream import
  identity is fixed by `export_hash` and `certificate_hash`, but package
  tooling should get a separate follow-up audit for version-lock consistency.
