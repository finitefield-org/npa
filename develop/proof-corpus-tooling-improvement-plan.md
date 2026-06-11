# Proof Corpus Tooling Improvement Plan

Date: 2026-06-03

This document is the tooling improvement plan and specification for speeding
up proof corpus authoring. It is an implementation plan, not grounds for proof
acceptance.

## 1. Purpose

The proof corpus is a staging environment where AI can try many theorems and
move settled proofs into `npa-mathlib`. During authoring, local verification
should be fast; release / promotion / package compatibility checks are pushed
to explicit gates.

This plan covers:

- Batch-building multiple modules and consolidating authoring index updates at
  the end. Public package metadata is generated explicitly during promotion /
  release handoff.
- Reusing the verified certificate cache for `npa-mathlib` / external packages
  read by corpus authoring across processes.
- Removing package-wide CLI examples from the corpus authoring gate and moving
  them to the daily / PR gate side.
- Creating commands / skills that standardize promotion from corpus modules to
  the `npa-mathlib` package.

## 2. Trust Boundary

This does not change NPA's trust boundary.

```text
Not trusted:
  AI / tactic / replay / metadata / theorem index / promotion plan / cache file

Trusted:
  canonical certificate
  deterministic hash
  small Rust kernel
  source-free checker / verifier verdict
```

In particular, the cross-process cache is a performance helper. Cache hits may
be used to shorten the authoring fast path, but must not be used as grounds
for release verdicts, public `npa-mathlib` promotion verdicts, or high-trust
audits. Final decisions run source-free verifier and package artifact checks
without cache, or in a mode where the cache is not treated as proof evidence.

## 3. Improvement 1: Batch Module Build

### 3.1 Background

`--build-module MODULE` rebuilds the specified module and its import closure.
During normal authoring, it does not update the public `manifest.toml`,
`npa-package.toml`, or `generated/package-lock.json`; it updates only module
artifacts and the AI theorem index. When adding multiple modules in sequence,
the rebuild order for downstream dependencies is easy to handle manually in
the wrong order, so batch builds are used.

### 3.2 CLI Specification

Authoring commands to add:

```sh
cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.X Proofs.Ai.Y
cargo run -p npa-proof-corpus -- --build-modules-file proofs/generated/build-batch.txt
```

Options:

```text
--build-modules <MODULE>...
  Build the specified modules and the required import closure once in
  topological order.

--build-modules-file <PATH>
  Read a batch spec with one module per line. Empty lines and # comments are
  ignored.

--package-metadata
  For promotion / release handoff. After all module artifacts have been
  generated, update manifest / package / package lock / AI index exactly once.

--metadata-once
  Compatibility alias for `--package-metadata`.

--failures-out <PATH>
  Emit failed modules / declarations / diagnostics as a JSON sidecar.
```

`--build-module MODULE` remains for compatibility and is treated internally as
a one-element batch.

### 3.3 Behavior

1. Validate input module names.
2. Compute the import closure.
3. Sort the closure in topological order.
4. Skip builds for modules whose hashes already match.
5. Build dirty / changed / explicitly requested modules.
6. Update the AI index only if everything succeeds.
7. Update manifest / package metadata / lock together only if
   `--package-metadata` is specified.
8. If some modules fail, certificates for successful modules may remain, but
   metadata is not updated.

### 3.4 Completion Criteria

- `--build-modules A B` reduces rebuild steps compared with running
  `--build-module A` and `--build-module B` sequentially.
- Shared import closures within a batch are built / verified only once.
- Stale package metadata is not written on failure.
- Existing `--build-module` behavior is not broken.

## 4. Improvement 2: Verified Certificate Cache

### 4.1 Background

`--module`, `--changed-only`, and package verifier tests decode / verify the
same checked-in certificates and the same import certificates again in each
process. This increases iteration time, especially when authoring corpus
modules that import verified certificates from `npa-mathlib` or external
packages.

### 4.2 Scope

The cache is for the authoring fast path. It is not a mechanism for shortening
the public release verdict for `npa-mathlib`. It is disabled by default for:

- `./scripts/check-corpus.sh`
- release / publish-plan / public package verification
- independent checker / high-trust audit
- The final gate for `npa-mathlib` release handoff.

### 4.3 Cache key

The cache key includes at least:

```text
core_spec
certificate_format
kernel_profile
verifier_profile
npa binary build identity
certificate_hash
certificate_file_hash
direct import module names
direct import export hashes
direct import certificate hashes
import closure certificate file hashes
axiom policy fingerprint
enabled core features
```

Cache entries are placed under content-addressed paths.

```text
target/npa-proof-cache/verified-v0.1/<cache-key>.json
```

PCT-05 implemented the `npa-proof-corpus.verified-cache.v0.1` schema,
content-addressed key material, entry JSON, and schema-version-mismatch-as-miss
behavior as the authoring cache data model. PCT-06 implemented lookup / write
/ hit reporting for `--module` / `--changed-only`. Because the cache key
includes certificate file hashes for the import closure in addition to direct
import identity, a changed dependency certificate file does not produce an
authoring hit and falls back to the live verifier. Gate scripts do not pass
`--verified-cache authoring`, so release-like paths default to cache off.

### 4.4 CLI Specification

```sh
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache off
```

mode:

```text
off
  Do not use the cache. This is the default for release / corpus gates.

authoring
  Use cache hits to shorten authoring verification. Output explicitly states
  that the verdict is cached.

read-through
  Look up the cache, but ultimately rerun the verifier and compare with the
  cache. This is for debugging. If a cache entry does not match the live
  verifier result, discard it as stale and rewrite the live result.
```

When cache mode is enabled, deterministic text output reports status in this
form.

```text
verified Proofs.Ai.X cache_status = "hit" cache_mode = "authoring"
verified Proofs.Ai.X cache_status = "stale" cache_mode = "read-through"
```

### 4.5 Completion Criteria

- Deleting the cache does not change acceptance results.
- Cache hits appear in machine-readable output as `cache_status = "hit"`.
- `check-corpus.sh` passes without cache.
- Cache entry schema version mismatches are safely treated as misses.

## 5. Improvement 3: Gate Split For Package-Wide CLI Examples

### 5.1 Background

The old `check-corpus.sh` included package-wide CLI examples and was too heavy
for the feedback loop immediately after authoring. These examples are
important as end-to-end regressions for the package CLI, but excessive for
every check during individual theorem authoring.

### 5.2 Gate Categories

Gates added in PCT-03:

```sh
./scripts/check-corpus-authoring.sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

Roles:

```text
check-corpus-authoring.sh
  Run only source-free checks of changed proof corpus modules with the
  authoring cache. Do not include package-wide CLI examples, axiom-report,
  index, or publish-plan.

check-corpus-package.sh
  Package-wide regressions for package verifier, package CLI examples,
  publish-plan, index, and axiom-report. Run this at npa-mathlib promotion,
  package tooling, and release/high-trust boundaries.

check-corpus-full.sh
  Full pre-promotion / pre-release / pre-high-trust gate combining authoring
  and package gates.
```

The existing `./scripts/check-corpus.sh` remains for compatibility and acts as
an alias that calls the lightweight `check-corpus-authoring.sh`. Heavy gates
are invoked explicitly through `check-corpus-package.sh` /
`check-corpus-full.sh`. Older instructions before the script split treated
`./scripts/check-corpus.sh` as the full corpus gate, but the current AGENTS.md
/ CONTRIBUTING.md / README.md direct normal staging-corpus authoring toward
the lightweight gate.

### 5.3 Completion Criteria

- Normal theorem authoring completion can point only to
  `check-corpus-authoring.sh`.
- Package-wide CLI examples remain in the PR / daily gate.
- The gate policies in `AGENTS.md` and `develop/proof-corpus-ai-workflow.md`
  follow the new script names.

## 6. Improvement 4: Promotion Command / Skill

### 6.1 Background

Settled corpus modules should move to `npa-mathlib`, but namespace
conversion, import mapping, package metadata updates, and downstream smoke
checks are still manual.

The `judge-promote-to-mathlib` skill already exists for judgment. The next
step is to make promotion plan generation and materialization command-driven.

### 6.2 CLI Specification

Implemented in PCT-04:

```sh
cargo run -p npa-proof-corpus -- \
  --promote-plan Proofs.Ai.Algebra.AbstractField \
  --mathlib-root ../npa-mathlib \
  --to-module Mathlib.Algebra.Field.Basic \
  --out develop/npa-mathlib-field-closure-audit.md
```

`--promote-plan` treats the tree under `--mathlib-root` as a read-only evidence
source. If `--out` points under `--mathlib-root`, it fails with a deterministic
diagnostic before generating the plan.

Materialize command implemented in PCT-07:

```sh
cargo run -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-field-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --dry-run \
  --compat-alias none
```

The default is dry-run. It writes target package source, certificate, meta,
replay, and `npa-package.toml` only when `--apply` is specified. It does not
stage git changes, and it lists written paths in deterministic text output. It
rejects PCT-04 plans with unresolved import mapping, axiom policy, or
compatibility alias decisions. `--compat-alias none` is the option an operator
uses to explicitly decide that no compatibility alias is needed.

### 6.3 Promotion Plan Contents

The promotion plan includes:

- Mapping between the corpus source module and target `Mathlib.*` module.
- Direct import mapping.
- Import closure and the module set entering the public package.
- Axiom policy diff.
- Theorem / definition / inductive export list.
- Whether a compatibility alias is required.
- `npa-mathlib` package gate and downstream smoke commands.
- Source-free verification evidence fields.

### 6.4 Completion Criteria

- Plan generation alone does not modify the `npa-mathlib` repository.
- Materialize separates dry-run from apply.
- Dry-run displays intended file / manifest / package metadata / namespace
  changes and does not modify the `npa-mathlib` repository.
- Apply writes only target package artifacts and manifests, and does not stage
  git changes.
- After materialize, package check, build-certs --check, verify-certs --checker
  reference, check-hashes, axiom-report --check, and index --check are
  recommended.
- Source-free downstream smoke is included in the promotion checklist.

## 7. Implementation Order

Recommended order:

1. Add batch module build.
2. Add the gate split and make the default authoring loop lighter.
3. Add the promotion plan command.
4. Add the verified certificate cache as authoring-only.
5. Add the promotion materialize command.

The cache is highly effective, but its trust boundary is subtle, so add it
after separating it from the release path.

## 8. Measurement Metrics

Record the following for each milestone.

- single module build time
- batch build time
- changed-only verification time
- authoring gate time
- package gate time
- cache hit ratio
- cache disabled full gate time

The goal is to make the normal theorem authoring loop closer to local build /
verify time than to full corpus gate time.

The final PCT-08 measurements are recorded in
`develop/proof-corpus-tooling-pct-08-measurement.md`. The PCT-00 baseline full
corpus gate was 1059.81s. In PCT-08, the clean small-module authoring loop,
meaning the total of `--build-module Proofs.Ai.Basic`, selected module
source-free verification, and `--changed-only`, was 2.69s. This is about 394
times shorter than the baseline full gate.

At PCT-08 time, `./scripts/check-corpus-authoring.sh` passed in 115.21s. The
current authoring gate is narrowed further to changed-only source-free checks
and is used as the normal batch-boundary gate for proof corpus staging.
`./scripts/check-corpus-package.sh` passed in 1122.39s. Because it includes
regressions for package verifier, package CLI examples, axiom-report, index,
and publish-plan, it is reserved for npa-mathlib promotion / release handoff /
compatibility-change boundaries.

Caches, promotion plans, promotion dry-runs, theorem indexes, replay,
metadata, CI status, and timing logs are all untrusted sidecars. They may be
used for work efficiency or as audit inputs, but they are not grounds for proof
acceptance.

## 9. Next Stage: Package / Closure Audit Acceleration

This plan targets the theorem authoring loop. Even after PCT-08, the package
gate includes regressions for package verifier, package CLI examples,
axiom-report, index, and publish-plan, so it remains a major bottleneck at
promotion / release / compatibility boundaries.

Next-stage acceleration for package verification, promotion readiness, and
closure audit is tracked separately in
`develop/proof-corpus-package-audit-speed-plan.md`. That plan covers techniques
analogous to Go package export data / build cache / package DAGs:

- Package checker result store.
- Verified export summary.
- Reverse dependency invalidation based on `export_hash` and
  `certificate_hash`.
- Cheap source-free preflight before checker execution.
- Deterministic topological layer parallelism.
- Integration of local cache / final cache-off gate into the closure audit
  workflow.

These are also untrusted acceleration layers, and the final judgment for
release / high-trust / public `npa-mathlib` handoff remains based on canonical
certificates, deterministic hashes, and source-free checker verdicts.
