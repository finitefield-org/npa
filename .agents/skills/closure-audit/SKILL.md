---
name: closure-audit
description: Audit a proof-corpus theorem/module closure and materialize it into the standalone npa-mathlib repository. Use when the user asks to run a closure audit, choose the next mathlib theorem layer, materialize proof-corpus modules into npa-mathlib, update package artifacts/downstream smoke, document failures, or complete the workflow with commit/push/tag.
---

# Closure Audit

## Scope

Use this skill to turn one proof-corpus theorem route into a public
`npa-mathlib` release layer. The workflow has two linked outputs:

- an audit document in the `npa` repository under `develop/`;
- materialized public modules and package artifacts in the `npa-mathlib`
  repository.

Keep the NPA trust boundary explicit: source, replay, metadata, theorem index,
publish plan, CI, and this audit are sidecars. Proof acceptance depends on
canonical `.npcert` bytes, deterministic hashes, and source-free checker
verdicts.

## Locate Repositories

Prefer the current workspace when it is an `npa` or `npa-mathlib` checkout.
Otherwise look for sibling repositories under the same parent directory.

Expected layout:

```text
<parent>/npa
<parent>/npa-mathlib
```

Read repo-local instructions before changing files:

```text
npa/AGENTS.md
npa-mathlib/AGENTS.md
```

If either repository is missing, document the blocker and ask for the path.

## Preflight

Run `git status --short` in both repositories. Do not overwrite unrelated user
changes. If unrelated changes exist, leave them alone and stage only files that
belong to this closure work.

Identify the requested route from the user prompt. If the prompt says "next",
use the current roadmap/audit history to choose the next closure:

```text
npa/develop/npa-mathlib-next-closure-roadmap.md
npa/develop/npa-mathlib-*-closure-audit.md
npa-mathlib/docs/namespace-policy.md
npa-mathlib/npa-package.toml
npa/proofs/npa-package.toml
```

Before selecting or rewriting public module names, declaration names, paths, or
compatibility aliases, read `npa-mathlib/docs/namespace-policy.md` and treat it
as the source of truth for the public naming policy. Use it to resolve
`Proofs.Ai.*` corpus names into `Mathlib.*` / `Std.*` public names, stability
expectations, path layout, and already released modules. If a generated promote
plan, corpus seed name, or prior draft audit conflicts with the namespace
policy, follow the namespace policy and record the naming decision in the audit.

If the route is ambiguous and cannot be inferred from local docs, ask one short
question before editing.

## Audit Workflow

Create or update one audit document in `npa/develop/` before materializing:

```text
npa-mathlib-<layer-or-topic>-closure-audit.md
```

The audit must include:

- selected proof-corpus modules;
- explicitly deferred nearby modules;
- public module names and filesystem paths;
- import rewrite table from `Proofs.Ai.*` to public `Mathlib.*` / `Std.*`;
- public declaration inventory;
- corpus source hash, certificate file hash, export hash, axiom report hash,
  and certificate hash;
- axiom policy and direct/transitive `Eq.rec` status;
- downstream smoke theorem names;
- positive gates to run;
- negative package-copy checks to run;
- materialization result or failure result after the attempt.

Use `rg` against `proofs/npa-package.toml`, source files, and prior audits to
build the closure. Prefer structured manifest data over ad hoc guessing.

Local proof-corpus checks for candidate modules:

```sh
cargo run -q -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -q -p npa-proof-corpus -- --build-module Proofs.Ai.X
```

Use the smallest module-specific checks that prove the selected corpus closure
is valid.

When a single corpus module maps cleanly to a public module, prefer the
repo-local promotion plan command to seed the audit:

```sh
cargo run -q -p npa-proof-corpus -- \
  --promote-plan Proofs.Ai.X \
  --mathlib-root ../npa-mathlib \
  --to-module Mathlib.X \
  --out develop/npa-mathlib-x-closure-audit.md
```

The generated plan is an untrusted audit helper. It does not modify
`npa-mathlib` and does not replace source-free verification or package gates.

## Materialize Workflow

Materialize only after the audit has a clear selected set.

Prefer the PCT-07 materialize command for the source, certificate, meta,
replay, manifest, and namespace rewrite mechanics when the audit has resolved
import mapping, axiom policy, and compatibility alias decisions:

```sh
cargo run -q -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-x-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --dry-run \
  --compat-alias none

cargo run -q -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-x-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --apply \
  --compat-alias none
```

The command stages no git changes. Dry-run is for review only; apply writes the
target package files and reports exact paths. It is still necessary to run the
positive gates, downstream smoke gates, negative checks, and release artifact
generation below.
When apply mode covers one of the checklist items below, record the command
output and continue with the remaining items; do not manually repeat a copy or
rewrite step unless the command reports that the route needs manual handling.

In `npa-mathlib`:

1. Bump `npa-package.toml` version to the next release version.
2. Copy selected `source.npa`, `replay.json`, and `meta.json` sidecars from
   `npa/proofs/...` into the chosen public `Mathlib/...` paths.
3. Rewrite imports from corpus names to public names.
4. Ensure no public source, manifest, package lock, publish plan, or downstream
   fixture contains stale `Proofs.Ai.*` names for the materialized route.
5. Add module entries to `npa-package.toml` with source, certificate, meta,
   replay, public declarations, imports, axioms, and expected hashes.
6. Run `package build-certs` in write mode to generate certificates and
   `generated/package-lock.json`.
7. Regenerate `generated/axiom-report.json`,
   `generated/theorem-index.json`, and `generated/publish-plan.json`.
8. Update README and namespace policy.
9. Update downstream smoke to consume the public closure source-free through
   vendored certificate bytes, not source files.
10. Generate a local release artifact tarball and SHA-256 sidecar under
    `target/release-artifacts/`.

When expected hashes differ after public renaming, use the CLI diagnostics as
the source of truth and update the manifest/meta sidecars to the actual public
hashes. Never treat old corpus hashes as public hashes after module renaming.

## Positive Gates

Run these gates from the `npa` repository against the sibling `npa-mathlib`
checkout after materialization:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib --json
cargo run -q -p npa-cli -- package axiom-report --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package index --root ../npa-mathlib --check --json
cargo run -q -p npa-cli -- package publish-plan --root ../npa-mathlib --check --json
```

Run downstream smoke gates:

```sh
cargo run -q -p npa-cli -- package check --root ../npa-mathlib/fixtures/downstream-smoke --json
cargo run -q -p npa-cli -- package build-certs --root ../npa-mathlib/fixtures/downstream-smoke --check --json
cargo run -q -p npa-cli -- package verify-certs --root ../npa-mathlib/fixtures/downstream-smoke --checker reference --json
cargo run -q -p npa-cli -- package check-hashes --root ../npa-mathlib/fixtures/downstream-smoke --json
```

Run `git diff --check` in both repositories before finalization.

## Negative Checks

Use temporary copies outside the repositories. Do not corrupt live files.

Required negative checks:

- bad public export hash is rejected as `export_hash_mismatch`;
- bad public certificate hash is rejected as `certificate_hash_mismatch`;
- corrupted certificate bytes are rejected by source-free reference
  verification, usually as `certificate_decode_failed` or another verifier
  failure;
- stale downstream lock or package-version pin is rejected as
  `package_lock_stale`.

Record the observed reason codes in the audit document.

## Failure Handling

If materialization cannot be completed, update the audit document with a
`Materialization Failure` section containing:

- exact command that failed;
- diagnostic `reason_code` or compiler/checker error;
- files or modules involved;
- whether the failure is a source issue, namespace issue, dependency issue,
  hash/artifact issue, checker issue, or git/release issue;
- next concrete action needed.

Do not create a release tag on failure. Commit or push the failure document only
when the user explicitly asks for it.

## Success Finalization

On success, update the audit document with a `Materialization Result` section:

- public modules and paths;
- public source/certificate/export/axiom/certificate hashes;
- downstream smoke theorem names and hashes;
- positive gate commands and status;
- negative check reason codes;
- release artifact path and SHA-256;
- publish-plan hash.

Then commit, push, and tag:

1. In `npa`, commit the audit/roadmap documentation changes and push the current
   branch.
2. In `npa-mathlib`, commit only the materialization changes, generated
   artifacts, docs, and downstream fixture updates.
3. Create an annotated `npa-mathlib` release tag matching the package version,
   for example `v0.1.14`.
4. Push the `npa-mathlib` branch and tag.

Do not overwrite an existing tag. If the tag already exists, stop and report the
conflict.

Use concise commit messages, for example:

```text
Add npa-mathlib Layer 3E closure audit
Release npa-mathlib v0.1.14 Logic.Iff
```

After successful staging, commit, branch, push, or PR actions, emit any git UI
directives required by the current environment.
