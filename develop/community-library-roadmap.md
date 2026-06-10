# NPA Community Theorem Library Roadmap

This document summarizes the mechanisms that should be fixed first in the
current `npa` repository before publishing NPA and building collective theorem
knowledge, like Lean mathlib, where many people can add theorems.

The goal is to create a convenient theorem contribution workflow without
weakening NPA's trust boundary.

```text
Not trusted:
  source parser / elaborator / tactic / AI / theorem search / API orchestration / package registry

Trusted:
  canonical certificate
  Rust kernel verdict
  source-free independent checker verdict
  deterministic hash / axiom report
```

---

# 1. Target State

Eventually, the implementation itself and theorem libraries should be separated.

```text
finitefield-org/npa
  kernel / certificate format / checker / frontend / tactic / package CLI

finitefield-org/npa-std
  small, robust standard library
  Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic, etc.

finitefield-org/npa-mathlib
  community theorem library
  algebra / order / topology / analysis / geometry, etc.

NPA package registry
  published package metadata
  module export_hash / certificate_hash / axiom report
  source-free checker result
```

Not everything needs to be split from the beginning. For now, use `proofs/` and
`tools/proof-corpus` in this `npa` repository as the seed, and first build the
package contract that lets an external repository perform the same build /
verify flow.

---

# 2. Basic Policy

## 2.1 The theorem library publishes certificates, not source, as the trusted unit

The main objects humans review are `.npa` source, naming, statements,
dependencies, and documentation. However, the published artifact that is trusted
is the `.npcert`.

```text
source.npa
  -> frontend / elaborator / tactic / AI assistant
  -> canonical certificate
  -> kernel verify
  -> independent checker verify
  -> published theorem artifact
```

`source.npa`, `replay.json`, tactic traces, and AI traces are useful auxiliary
information. Their presence is not the basis for calling something proved.

## 2.2 Pin imports by hash, not just by name

In an external library, the same module name may point to different contents in
the future. Therefore, imports must be pinned at least by the following tuple.

```text
module
export_hash
certificate_hash
```

`export_hash` fixes the identity of the public interface. In high-trust mode,
`certificate_hash` also fixes the identity of the dependency certificate itself.

## 2.3 API endpoints are not proof acceptance boundaries

`crates/npa-api` has library APIs corresponding to `/machine/tactics/batch` and
`/machine/verify`. They are untrusted orchestration for proof state, tactic
execution, search, replay, and verify handoff.

The acceptance condition for a published package is not an API response, but the
following evidence.

```text
checked certificate
  + deterministic certificate_hash
  + deterministic export_hash
  + deterministic axiom_report_hash
  + source-free independent checker success
```

## 2.4 Separate high-traffic theorem repos from the trusted base

PRs to `npa-mathlib` are mainly about adding theorems, naming, abstraction, and
dependency improvements. PRs to the main `npa` repository are mainly about the
kernel, certificate format, checker, frontend, and package CLI.

This separation prevents community theorem contribution from being mixed with
changes to the trusted base.

---

# 3. Current Starting Point

The current repository already has components that can seed an external library.

```text
proofs/manifest.toml
  list of source / certificate / replay / meta / hash / axioms for each proof module

tools/proof-corpus
  proof corpus generator fixed to the current repository
  handles source generation, certificate encode, verify, and manifest updates

crates/npa-cert
  canonical certificate encode / decode / verify
  export_hash / certificate_hash / axiom_report_hash

crates/npa-checker-ref
  source-free reference checker binary

crates/npa-api
  Machine API substrate
  checker audit automation substrate
  theorem index / search / replay / verify handoff
```

What is missing is a package-level contract that external repositories can use
directly. In particular, `tools/proof-corpus` is currently tightly coupled to the
current proof corpus layout and to the module list inside Rust source. This must
be generalized.

---

# 4. Current Unfinished Items

This section makes the remaining implementation gaps explicit before moving on
to a registry. Items here should either be completed in this `npa` repository
before building a registry server, or at least fixed as contracts that external
theorem libraries can depend on.

## 4.1 Unfinished / target integrations on the NPA core side

The certificate, fast verifier, reference checker, and Machine API substrate
already exist. As of CLR-06, the package manifest, package CLI, hash-pinned
imports, source-free package verification, axiom report, theorem index, and
publish metadata are also implemented as the local package contract.

The remaining target integration for the public ecosystem is:

```text
- pinned built `npa-checker-ext` release/high-trust binary evidence
```

The source-free reference checker binary in `crates/npa-checker-ref` exists.
The source-free reference checker binary in `crates/npa-checker-ref` exists.
The OCaml clean-room `npa-checker-ext` source is in `checkers/npa-checker-ext/`,
and the package command `npa package verify-certs --checker external` is already
implemented with runner policy and checker registry as required inputs.
`package high-trust` is also implemented as a `verified_high_trust` artifact
generator. However, `npa-checker-ext` should be treated as existing release /
high-trust evidence only when a built executable is available in a fresh checkout
or documented CI environment, a runner-owned registry / policy verifies the
binary identity and hash, and package external-mode integration passes. The
copyable high-trust CI template is
`ci-templates/github-actions/npa-package-high-trust.yml`, but real high-trust
release evidence requires a pinned binary and release audit bundle on the
external repo side.
The deterministic summary contract for external checker benchmark / release
audit collection is implemented. A benchmark row links checker identity,
certificate hash, module, time, timeout, memory limit, and checker result hash to
the machine result inside the release audit bundle, but this is regression
evidence and not proof input that changes proof validity. Reference / external
checker benchmarks are not synchronous required jobs on the PR hot path; they
may fail as release audit diagnostics only when release / high-trust policy
requires them.

This repository's GitHub Actions workflows are currently removed, and
`scripts/phase8-release-audit.sh` and `scripts/phase9-regression.sh` are run
locally when needed. Copyable workflow templates for external theorem libraries
are available at `ci-templates/github-actions/npa-package-pr.yml` and
`ci-templates/github-actions/npa-package-release.yml`, but they are not active
gates for this repository. The opt-in high-trust release template is
`ci-templates/github-actions/npa-package-high-trust.yml` and is separated from
the base templates.

## 4.2 Pre-registry blockers

The blockers needed before a registry server are:

```text
1. package manifest
   The standard format declaring the module graph, source paths, certificate
   paths, imports, expected hashes, and axiom policy with `npa.package.v0.1` is
   fixed. The next validation target is using it in an external dogfood repo.

2. package CLI
   `npa package check`, `build-certs`, `verify-certs`, `check-hashes`,
   `axiom-report`, `index`, and `publish-plan` are implemented. External
   library CI templates were fixed under `ci-templates/github-actions/` in
   CLR-07.

3. CI contract
   What is required for theorem-only PRs and what is required for release /
   high-trust is fixed in `docs/external-theorem-library-ci.md` and
   `develop/community-library-roadmap-clr-07-todo.md`.

4. external package import resolution
   Imports between packages are locked by `module + export_hash +
   certificate_hash`. CLR-09 creates a record of importing from a seed release
   artifact into a downstream fixture.

5. source-free package verification
   The CLI contract for checking the whole package graph without source in
   dependency-topological order is implemented by `npa package verify-certs`.
   Integration into external repo CI templates was completed in CLR-07.

6. deterministic public artifacts
   Fix the theorem index, axiom report, package lock, and publish metadata as
   deterministic artifacts for the registry, docs, and downstream packages. In
   CLR-06, publish metadata was fixed as `generated/publish-plan.json`.

7. publish metadata
   CLR-06 provides the schema / generator that emits each module's
   `export_hash`, `certificate_hash`, `axiom_report_hash`, checker result, and
   import closure as registry entries. Do not build the registry server or
   dependency solver yet.

8. external dogfood repo
   In CLR-09, `fixtures/npa-mathlib-seed` was fixed as a reference-checker-only
   seed release fixture. Package commands equivalent to a fresh checkout,
   release artifacts, a downstream import fixture, and workflow templates have
   been validated. Standalone repo activation and CLR-08 high-trust evidence
   remain readiness / follow-up decisions for CLR-10 and later.
```

If the remaining CI / dogfood / registry integration is skipped and only the
registry server is built, the registry becomes a convenient layer for
distributing the latest source or latest package, blurring NPA's
certificate-first trust boundary.

## 4.3 Things that are not pre-registry blockers

The following are important, but they are not prerequisites for the registry seed.

```text
- production LLM / RAG integration
- online theorem graph store
- external SMT solver service
- browser IDE
- package dependency solver
- binary cache service
- proof search marketplace
```

Add these after the package contract and source-free verification are fixed.

---

# 5. What To Build First

## 5.1 `npa-package.toml`

An external theorem library places `npa-package.toml` at its root. This package
manifest describes the source layout, certificate artifacts, import lock, axiom
policy, and generated artifacts.

The draft structure is below. In a real manifest, hash fields contain the exact
SHA-256 strings generated by package commands. This document does not use
pseudo hash literals.

```toml
schema = "npa.package.v0.1"
package = "npa-mathlib-seed"
version = "0.1.0"
license = "Apache-2.0"

core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"

[policy]
allow_custom_axioms = false
allowed_axioms = []

[[modules]]
module = "Proofs.Ai.Basic"
source = "Proofs/Ai/Basic/source.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
meta = "Proofs/Ai/Basic/meta.json"
replay = "Proofs/Ai/Basic/replay.json"
producer_profile = "human-surface-explicit-term"

imports = []
# expected_source_hash, expected_certificate_file_hash,
# expected_export_hash, expected_certificate_hash, and
# expected_axiom_report_hash are required exact SHA-256 values
# in a real manifest.

definitions = []
theorems = ["id", "compose"]
axioms = []

[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"
# export_hash and certificate_hash are required exact SHA-256 values
# in a real manifest.
```

Manifest roles:

```text
- make the module graph explicit
- map source paths to certificate paths
- pin imports by hash
- make expected hashes comparable in CI
- fix axiom policy at the package level
- emit metadata needed for registry publishing
```

Import resolution rule:

```text
- module-level `imports = [...]` strings first resolve to same-package `[[modules]]`
  entries by exact module name.
- otherwise they must resolve to hash-pinned top-level `[[imports]]` entries.
- external import by module name alone is invalid.
- local/external module name collision is invalid.
- registry lookup, network fetch, package-cache fallback, and implicit latest-version
  resolution are forbidden.
```

Schema constants fixed by CLR-00:

| Constant | Schema string | Artifact |
| --- | --- | --- |
| `PACKAGE_MANIFEST_SCHEMA` | `npa.package.v0.1` | `npa-package.toml` |
| `PACKAGE_LOCK_SCHEMA` | `npa.package.lock.v0.1` | `generated/package-lock.json` |
| `PACKAGE_AXIOM_REPORT_SCHEMA` | `npa.package.axiom_report.v0.1` | `generated/axiom-report.json` |
| `PACKAGE_THEOREM_INDEX_SCHEMA` | `npa.package.theorem_index.v0.1` | `generated/theorem-index.json` |
| `PACKAGE_PUBLISH_PLAN_SCHEMA` | `npa.package.publish_plan.v0.1` | `generated/publish-plan.json` |
| `REGISTRY_MODULE_SCHEMA` | `npa.registry.module.v0.1` | module registry entry |

`npa.package.lock.v0.1` is a package-level artifact. The Phase 8
`npa.independent-checker.import_lock_manifest.v1` is source-free checker input
derived from package metadata for each checker run, and it is not the same
schema.
`generated/package-lock.json`、`generated/axiom-report.json`、`generated/theorem-index.json`、
`generated/publish-plan.json`, and registry module entries are metadata for
review, search, publishing, and CI freshness checks. They are not checker
evidence; the basis for proof acceptance is only the canonical certificate and
the kernel / source-free checker verdict.

Forbidden:

```text
- resolve imports by module name only
- have the package manager implicitly fill in the latest certificate from a registry
- treat a source file alone as verified
- downgrade an expected hash mismatch to a warning
```

## 5.2 package CLI

This `npa` repository provides a CLI that external repositories can use. In
CLR-00, the contributor-facing command is fixed as the installed binary `npa`,
and the Cargo package is fixed as `npa-cli`. In-repo validation uses
`cargo run -p npa-cli -- package ...`; docs for external contributors use
`npa package ...`.

Basic commands implemented in CLR-04:

```sh
npa package check
npa package build-certs
npa package verify-certs
npa package check-hashes
```

Generated metadata commands implemented by CLR-05:

```sh
npa package axiom-report
npa package index
```

CLR-06 release metadata command:

```sh
npa package publish-plan
```

Command responsibilities:

```text
npa package check
  Metadata gate that checks the manifest schema, module graph, paths, and policy.

npa package build-certs
  Regenerate certificate.npcert from source.npa.
  replay.json / helper data may be read, but they are not trusted inputs.
  --check only checks diffs and does not rewrite checked-in artifacts.

npa package verify-certs
  Check source-free from generated/package-lock.json and certificate artifacts.
  .npa source, replay, meta, theorem index, AI traces, and out-of-package state
  are not checker inputs.
  It can run both the fast verifier and the reference checker, but a fast result
  is not treated as a reference checker verdict.

npa package check-hashes
  Compare expected_export_hash / expected_certificate_hash /
  expected_axiom_report_hash with the actual artifacts. expected_source_hash and
  the certificate file hash are also checked against checked-in bytes.

npa package axiom-report
  Generate or --check `npa.package.axiom_report.v0.1` metadata for the whole
  package and for each module.
  The package axiom report schema is distinct from
  `npa.independent-checker.axiom_report.v1` and from the Std-only axiom report
  schema.

npa package index
  Generate or --check `npa.package.theorem_index.v0.1` metadata for theorem
  search, documentation, and future registry metadata.
  The package theorem index schema is distinct from the Std-only theorem index
  schema.

npa package publish-plan
  CLR-06. Emit publish metadata and an artifact list for
  `npa.package.publish_plan.v0.1`.
  Include `npa.registry.module.v0.1` theorem package module registry seed
  entries, a downstream import bundle, and a checksum-only SHA-256 signature
  policy.
  Do not perform registry server access, registry URL handling, network fetch,
  latest-version resolution, upload, or signing.
```

The CLI is an untrusted orchestration layer. CLI output, diagnostics, package
locks, generated axiom reports, generated theorem indexes, and generated publish
plans are deterministic metadata for review / CI / search / release, not proof
evidence. The basis for proof acceptance is limited to the canonical `.npcert`
and kernel / source-free checker verdict. `npa.registry.module.v0.1` is theorem
package registry metadata and is distinct from
`npa.independent-checker.checker_binary_registry.v1`. Registry metadata is not
checker input.
The kernel crate must not gain filesystem, network, or registry lookup behavior.
Package commands operate only on an explicit local package root and perform no
network access or binary cache lookup.

## 5.3 CI contract

In PR mode, the current base contract uses a full-package reference check.
Changed-module selection is useful, but it is not yet part of the required
package command contract. Copyable templates are in `ci-templates/github-actions/`,
and the detailed task breakdown is
`develop/community-library-roadmap-clr-07-todo.md`.

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

`axiom-report --check` and `index --check` are CLR-05 full-package freshness
gates. Both preserve the source-free artifact generation boundary and do not
require source, replay, meta, theorem graph scores, prompt metadata, or AI
traces as inputs. CLR-06 `publish-plan --check` is included in the release
artifact freshness gate.

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker fast --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

CLR-06 release gate extension:

```sh
npa package publish-plan --root . --check --json
```

PR mode does not need to require the external checker. Base release mode also
does not require the external checker. `--checker external` and
`package high-trust` are high-trust extensions that explicitly specify runner
policy and checker binary registry, and read only source-free package artifacts
and a built `npa-checker-ext` executable. `verified_high_trust` can be generated
only after the required evidence, including the external checker and
high-trust-reference, is present. It must not be generated from reference-only
evidence.
`--changed`, `--all`, `--registry`, `--network`, and `--latest` are not part of
the base contract. `npa-package-pr.yml` and `npa-package-release.yml` are
templates that external repositories copy or reference; they are not enabled as
this repository's `.github/workflows`. `npa-package-high-trust.yml` is an opt-in
template copied only when the same external repo provides a pinned external
checker binary and release audit evidence. In CLR-09, `npa-mathlib-seed` imports
these templates plus `summarize-npa-diagnostics.py` / `validate-workflows.py`.

## 5.4 artifact layout

The layout of an external theorem library should make the mapping between source
and checked artifacts mechanically clear.

```text
npa-mathlib/
  npa-package.toml
  Math/
    Basic/
      source.npa
      certificate.npcert
      replay.json
      meta.json
    Algebra/
      Group/
        Basic/
          source.npa
          certificate.npcert
          replay.json
          meta.json
  generated/
    package-lock.json
    theorem-index.json
    axiom-report.json
    publish-plan.json
```

`replay.json` is optional. It is useful for AI proof search and tactic replay
reproducibility, but the checker is assumed not to read it.
`generated/axiom-report.json` and `generated/theorem-index.json` are also not
checker inputs. Their generation and checking are source-free and require only
the manifest, package lock, certificate artifacts, source-free verifier output,
and checked generated JSON in check mode.

## 5.5 review policy

Review in the community theorem library is not where humans manually check proof
correctness. Correctness is checked by certificates and checkers.

Things human review should check:

```text
- whether the theorem statement represents the intended mathematics
- whether the namespace and theorem name are easy to search
- whether it duplicates existing theorems
- whether dependencies are too heavy
- whether axiom use has increased
- whether the abstraction is usable by later library code
- whether the source is maintainable
- whether documentation and tags are sufficient
```

Things CI should check:

```text
- whether the certificate can be regenerated
- whether the source-free checker passes
- whether expected hashes match
- whether the axiom report satisfies policy
- whether the import closure is hash-pinned
- whether the theorem index is deterministic
```

## 5.6 CLR-06 publish-plan handoff

CLR-06 `publish-plan` may consume the package metadata fixed by CLR-05. However,
neither publish metadata nor registry entries are part of the trusted base; the
basis for proof acceptance remains the canonical certificate and source-free
checker verdict.

CLR-06 may use CLR-05 artifact fields:

```text
generated/axiom-report.json
  schema = npa.package.axiom_report.v0.1
  package, version, manifest, package_lock
  policy.allow_custom_axioms, policy.allowed_axioms
  modules[].module, origin, export_hash, certificate_hash, axiom_report_hash,
  certificate_file_hash, direct_axioms, transitive_axioms, policy_status
  checker_summaries[]
  summary.*
  package_axiom_report_hash

generated/theorem-index.json
  schema = npa.package.theorem_index.v0.1
  package, version, manifest, package_lock, index_profile
  entries[].global_ref, kind, statement.core_hash, statement.head,
  statement.constants, modes, tags, axiom_dependencies,
  module_axiom_report_hash, artifact.certificate
  checker_summaries[]
  summary.*
  theorem_index_hash
```

CLR-06 may copy artifact paths, hashes, policy status, checker summaries, and theorem index
entries into publish metadata. It must not require source, replay, meta, theorem graph score,
prompt metadata, or AI traces to validate CLR-05 artifacts.

CLR-06 output is helper metadata, not proof evidence. `generated/publish-plan.json` uses
`npa.package.publish_plan.v0.1`, embeds `npa.registry.module.v0.1` theorem package module seed
entries, and records a checksum-only SHA-256 MVP signature policy. It must remain distinct from
Phase 8 independent checker binary registry metadata such as
`npa.independent-checker.checker_binary_registry.v1`. Downstream packages may use the
downstream import bundle to copy package, version, module, path, `export_hash`, and
`certificate_hash` pins, but they still rerun source-free local verification from certificate
bytes.

Handoff boundaries:

```text
CLR-07 may add an optional publish-plan check step to release-template variants.
Base CLR-07 templates still avoid registry/network/latest/upload/sign behavior.
CLR-09 consumes the publish plan, release artifact list, registry seed entries,
and downstream import bundle for the seed library release flow.
```

## 5.7 Registry

At first, it is acceptable to use only Git tags and release artifacts without
building a registry service. However, metadata that can migrate to a future
registry should be fixed from the beginning.

Minimum unit of a registry entry:

```text
schema = npa.registry.module.v0.1
package = npa-mathlib
package_version = 0.1.0
module = Math.Algebra.Group.Basic
core_spec = npa.core.v0.1
kernel_profile = npa.kernel.v0.1
certificate_format = npa.certificate.canonical.v0.1
export_hash = exact SHA-256 export hash
certificate_hash = exact SHA-256 certificate hash
axiom_report_hash = exact SHA-256 axiom report hash
imports = direct imports with module, export_hash, certificate_hash
checker_results = source-free checker summaries, for example npa-checker-ref accepted
artifact_hashes = release artifact file hashes
```

The registry is a convenient distribution and search layer, not the trusted
base. Registry metadata is treated as auxiliary input that helps the local
checker recheck certificates. `npa.registry.module.v0.1` is theorem package
module metadata, while `npa.independent-checker.checker_binary_registry.v1` is
external checker binary selection / runner policy metadata. These two are not
substitutes for each other. A registry seed alone does not approve imports;
downstream packages rerun the source-free checker locally.

---

# 6. Completion Criteria For Splitting Into Another Repo

The conditions for safely moving `npa-mathlib` out are:

```text
- the package graph can be read from only npa-package.toml at the external repo root
- certificates can be regenerated from source
- checked-in certificate hashes match regenerated certificate hashes
- the source-free reference checker can check all certificates
- the import closure is pinned by module / export_hash / certificate_hash
- the axiom report is deterministic and policy violations can become CI failures
- package lock / axiom report / theorem index / publish-plan can be generated deterministically
- fresh-checkout CI passes without depending on the registry or local machine state
- theorem-only PRs can be accepted without changing the kernel / certificate / checker in the main `npa` repository
```

Until these conditions are satisfied, it is safer to operate `proofs/` as the
seed corpus inside this repository.

---

# 7. Implementation Milestones

Detailed implementation units have already been split into
`develop/community-library-roadmap-todo.md` and
`develop/community-library-roadmap-clr-00-todo.md` through
`develop/community-library-roadmap-clr-10-todo.md`. This chapter maps the
original M0-M5 sequence to the current CLR sequence.

```text
M0: Package the current proof corpus
  -> CLR-00, CLR-01, CLR-02
     Fix the CLI / schema, add the `npa.package.v0.1` validator,
     and represent `proofs/` as a package fixture.

M1: package manifest validator
  -> CLR-01
     Manifest parse, closed schema, path/hash/axiom/import graph validation.

M2: generic package build / verify CLI
  -> CLR-03, CLR-04
     Import lock, source-free package graph verification,
     `npa package check/build-certs/verify-certs/check-hashes`。
     The detailed CLR-04 breakdown is `develop/community-library-roadmap-clr-04-todo.md`.

M3: deterministic public artifacts and CI template
  -> CLR-05, CLR-07
     Axiom report / theorem index, external theorem library CI templates.
     Base CI uses a full-package reference check; changed-module selection comes later.
     The detailed CLR-05 breakdown is `develop/community-library-roadmap-clr-05-todo.md`.
     The detailed CLR-07 breakdown is `develop/community-library-roadmap-clr-07-todo.md`.

M4: publish metadata / registry seed
  -> CLR-06
     `generated/publish-plan.json`、`npa.registry.module.v0.1`、
     downstream import bundle, checksum-only MVP policy.

M5: npa-mathlib-seed dogfood
  -> CLR-09
     Start from `Proofs.Ai.Basic`, `Proofs.Ai.Prop`, `Proofs.Ai.Eq`,
     `Proofs.Ai.Nat`, and `Proofs.Ai.Reduction`.
     Copy or reference CLR-07 `npa-package-pr.yml` / `npa-package-release.yml`.
     Larger algebra / geometry / analysis corpora come after package ergonomics
     are validated.

Registry readiness
  -> CLR-10
     Collect pass/fail evidence for the section 4.2 blockers,
     and record the decision to continue Git-release-based registry seed operation.
     Public `npa-mathlib` Layer 0 has been made a local baseline in
     `fixtures/npa-mathlib/` and `fixtures/npa-mathlib-downstream/`.
     The next work follows `develop/npa-mathlib-public-release-plan.md`, uses
     the procedure in `develop/npa-standalone-repo-activation.md` to proceed with
     standalone repo activation, and then adds larger theorem layers.
```

CLR-08 is an independent milestone for high-trust external checker integration.
The package external checker command, `verified_high_trust` generator, opt-in
high-trust CI template, and external checker benchmark / audit summary contract
are implemented, but `npa-mathlib-seed` and registry readiness can proceed as a
reference-checker-only release even if external / high-trust-reference evidence
does not exist yet.

---

# 8. Initial Contributor Workflow

Keep the contributor-facing flow in the first external library as short as
possible.

```text
1. Add the theorem to source.npa
2. Run npa package check --root . --json
3. Run npa package build-certs --root . --check --json
4. Run npa package check-hashes --root . --json
5. Run npa package verify-certs --root . --checker reference --json
6. Run npa package axiom-report --root . --check --json
7. Run npa package index --root . --check --json
8. After CLR-06, before release, run npa package publish-plan --root . --check --json
9. Commit the necessary diffs to source / certificate / replay / meta / generated artifacts
10. Open a PR
11. CI checks the same package commands in a fresh checkout
12. Reviewers inspect statements / naming / dependencies / axiom changes / documentation
```

AI assistants and tactics may help contributors, but PR pass/fail is decided by
certificates and checkers.

---

# 9. Non-Goals

Things this roadmap does not build immediately:

```text
- online theorem proving service
- registry server
- package dependency solver
- binary cache service
- proof search marketplace
- browser IDE
- production LLM / RAG integration
- external SMT solver service
```

These are useful, but first complete the package contract, source-free checker
CI, and hash-pinned imports.

---

# 10. Immediate Implementation Order

If starting now in this `npa` repo, the order is:

```text
1. Write the design diff that maps `proofs/manifest.toml` to the npa-package.toml draft.
2. Add the Rust data model and validator for `npa.package.v0.1`.
3. Separate the hard-coded module list in `tools/proof-corpus` from the package CLI responsibilities.
4. Make the current `proofs/` build / verify through the package CLI.
5. Make the source-free checker a required gate in the package CLI.
6. Make the theorem index / axiom report / publish-plan deterministic artifacts.
7. Create CI templates for external theorem libraries.
8. In `npa-mathlib-seed`, fix the reference-checker-only dogfood release artifact and downstream fixture.
9. In the registry readiness review, record the decision to continue Git release artifact operation instead of server implementation.
10. Stabilize `fixtures/npa-mathlib/` and `fixtures/npa-mathlib-downstream/` as the public Layer 0 baseline.
11. Follow `develop/npa-standalone-repo-activation.md` to activate `npa`, `npa-std`, and `npa-mathlib`, then proceed to larger theorem layers.
```

This order lets the repository validate the required trust boundary and
contributor experience before creating a separate repo.
