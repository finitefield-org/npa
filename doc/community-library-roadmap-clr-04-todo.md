# Community Library Roadmap CLR-04 Todo

Source: `doc/community-library-roadmap-todo.md` CLR-04

CLR-04 exposes the package contract through the contributor-facing `npa package`
CLI. It wraps the manifest validator from CLR-01, the proof-corpus package
fixture from CLR-02, and the source-free package verifier from CLR-03 into
deterministic commands for checking, rebuilding certificates, verifying
certificates, and checking artifact hashes.

---

## Scope

対象:

```text
- `crates/npa-cli` workspace binary crate
- installed binary name `npa`
- `npa package check`
- `npa package build-certs`
- `npa package verify-certs`
- `npa package check-hashes`
- common package-root loading and package-relative path handling
- structured command diagnostics and deterministic exit codes
- package-driven source-to-certificate rebuild for local modules
- CLI integration tests against the current proof-corpus package fixture
```

非対象:

```text
- `npa package axiom-report`
- `npa package index`
- `npa package publish-plan`
- changed-module or reverse-dependency selection
- external checker binary as a required pass condition
- external package discovery or resolution, network access, binary cache, or implicit version selection
- source-free verifier implementation internals already owned by CLR-03
- theorem index or publish metadata generation
- automatically rewriting expected hashes in `npa-package.toml`
```

Trusted-boundary rule:

```text
`npa-cli` is an untrusted orchestration layer. It may read package files,
source files, replay helpers, certificates, and generated metadata according to
the selected command, but it never becomes proof evidence.

`verify-certs --checker reference` must remain source-free and must not read
source, replay, meta, theorem index, AI trace, or out-of-package state.
The kernel, `npa-cert`, and `npa-checker-ref` must not depend on
`npa-cli`.
```

---

## Implementation Specification

### Crate Placement

Add a workspace crate:

```text
crates/npa-cli
  Cargo package name: npa-cli
  binary name: npa
```

Suggested module layout:

```text
crates/npa-cli/Cargo.toml
crates/npa-cli/src/main.rs
crates/npa-cli/src/lib.rs
crates/npa-cli/src/args.rs
crates/npa-cli/src/diagnostic.rs
crates/npa-cli/src/fs.rs
crates/npa-cli/src/package.rs
crates/npa-cli/src/package_check.rs
crates/npa-cli/src/package_build.rs
crates/npa-cli/src/package_hashes.rs
crates/npa-cli/src/package_verify.rs
crates/npa-cli/tests/package_cli.rs
crates/npa-cli/tests/fixtures/package
```

Allowed dependency direction:

```text
npa-cli -> npa-package
npa-cli -> npa-api
npa-cli -> npa-frontend
npa-cli -> npa-cert
npa-cli -> sha2 or the workspace hash helper selected by implementation
```

Disallowed dependency direction:

```text
npa-package -> npa-cli
npa-api -> npa-cli
npa-checker-ref -> npa-cli
npa-cert -> npa-cli
npa-kernel -> npa-cli
```

`npa-cli` owns CLI parsing and filesystem access. It should not move those
responsibilities into the kernel, certificate, reference checker, or package
metadata crates.

### Command Contract

Cargo development commands:

```sh
cargo run -p npa-cli -- package check --root proofs
cargo run -p npa-cli -- package build-certs --root proofs --check
cargo run -p npa-cli -- package verify-certs --root proofs --checker reference
cargo run -p npa-cli -- package check-hashes --root proofs
```

Contributor-facing commands after installation:

```sh
npa package check --root proofs
npa package build-certs --root proofs --check
npa package verify-certs --root proofs --checker reference
npa package check-hashes --root proofs
```

Common flags:

```text
--root PATH
  Package root. Defaults to the current directory only. The CLI must not search
  parent directories or registry locations.

--json
  Emit deterministic JSON diagnostics. Human output may be the default, but
  tests should assert the structured JSON form.
```

Command-specific flags:

```text
package build-certs
  --check
    Rebuild certificates in memory and fail if checked-in certificate artifacts
    or generated lock artifacts would change. No files are written.

package verify-certs
  --checker reference
    Run the CLR-03 reference source-free verifier. This is the default if the
    flag is omitted.

  --checker fast
    Run the CLR-03 fast source-free verifier. The result must be labeled as
    fast-kernel and must not be reported as a reference checker verdict.
```

Unsupported in CLR-04:

```text
--changed
--all
--checker external
--registry
--network
--update-manifest-hashes
```

Unsupported flags fail deterministically with a usage diagnostic. Full-package
operation is the only CLR-04 scope.

### Exit Codes

Use stable process exit codes:

```text
0  command succeeded
1  package validation, hash, build, or checker failure
2  CLI usage error or unexpected internal command failure
```

Manifest/schema failures, stale hashes, source build failures, and checker
rejections are exit code `1`, because they are expected CI failures. Unknown
flags, invalid flag combinations, unreadable current directory, and command
implementation panics are exit code `2`.

### Structured Diagnostic Shape

All commands should build diagnostics before rendering them. JSON output should
use stable field names:

```text
schema = "npa.package.command_result.v0.1"
command
root
status
diagnostics
artifacts
```

Each diagnostic should include:

```text
kind
reason_code
severity
module when applicable
path when applicable
field when applicable
expected_hash when applicable
actual_hash when applicable
expected_value when applicable
actual_value when applicable
checker when applicable
```

Required diagnostic categories:

```text
Usage
PackageManifest
PackageGraph
PackageLock
ArtifactIo
HashMismatch
Build
SourceFreeBoundary
FastVerifier
ReferenceVerifier
Internal
```

Required reason codes:

```text
unknown_command
unknown_flag
missing_flag_value
duplicate_flag
unsupported_flag
manifest_missing
manifest_invalid
package_graph_invalid
package_lock_missing
package_lock_stale
source_missing
certificate_missing
source_hash_mismatch
certificate_file_hash_mismatch
export_hash_mismatch
axiom_report_hash_mismatch
certificate_hash_mismatch
build_certificate_changed
build_failed
source_artifact_read_attempted
fast_verifier_rejected
reference_verifier_rejected
unsupported_checker
```

Rendered paths should be package-relative where possible. Do not include host
absolute paths, timestamps, temp directory names, environment variables, or
machine-local command timing in deterministic JSON.

### Package Root Loading

The package root loader:

```text
1. Resolve `--root` or default to `.`.
2. Read exactly `npa-package.toml` at that root unless a later milestone adds a
   manifest override.
3. Pass manifest bytes to `npa-package` parse and validation APIs.
4. Join package-relative paths against the root only after lexical path
   validation has succeeded.
5. Reject path escape, absolute package paths, URI-like paths, and backslashes
   through existing CLR-01 path validation.
```

The loader must not:

```text
- search parent directories
- contact a registry
- infer package dependencies from Cargo metadata
- read source, replay, meta, theorem index, or AI trace files as part of
  manifest-only `package check`
```

### `package check`

`package check` is a metadata check:

```text
reads:
  npa-package.toml

does not read:
  .npa source
  certificate bytes
  replay/meta sidecars
  generated package lock
  theorem index
  AI traces
```

It validates:

```text
- package manifest schema
- package profile strings
- closed-object field set
- path grammar
- hash grammar
- local/external import resolution
- local graph cycle detection
- package axiom policy against declared module axiom summaries
```

It does not prove certificates and does not check artifact byte hashes.

### `package check-hashes`

`package check-hashes` is the checked-in artifact freshness gate.

It reads:

```text
- npa-package.toml
- local module source files
- local module certificate files
- external import certificate files
- generated/package-lock.json
```

It checks:

```text
- exact source file SHA-256 against `expected_source_hash`
- exact certificate file SHA-256 against `expected_certificate_file_hash`
- certificate export hash against `expected_export_hash`
- certificate axiom report hash against `expected_axiom_report_hash`
- certificate hash against `expected_certificate_hash`
- external import certificate export and certificate hashes against top-level imports
- regenerated package lock canonical JSON against `generated/package-lock.json`
```

It does not run the frontend, reference checker, fast kernel checker, tactic,
AI, registry, or theorem search. Stale artifacts produce exit code `1`.

### `package build-certs`

`package build-certs` rebuilds local module certificates from package source.

It reads:

```text
- npa-package.toml
- local `.npa` source files
- external import certificate files
- local import certificates or freshly rebuilt local imports in topological order
- replay/meta helper files only if the frontend build path explicitly needs them
```

It may use:

```text
npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy
```

Build order:

```text
1. Validate package manifest.
2. Build or load external import verified modules from certificate artifacts.
3. Build local modules in dependency-topological order.
4. For each local module, compile source with verified imports and imported
   source interfaces from earlier local builds or certificate-derived fallback
   interfaces.
5. Encode the canonical certificate bytes.
6. Compare generated certificate bytes and hashes against checked-in artifacts.
```

`--check` mode:

```text
- writes no files
- fails if a generated local certificate differs from the checked-in
  certificate file
- fails if generated hashes differ from package manifest expected hashes
- fails if regenerated `generated/package-lock.json` would differ
```

write mode:

```text
- writes local module certificate files only
- may rewrite `generated/package-lock.json` after certificates are written
- does not rewrite source files
- does not rewrite `npa-package.toml`
- does not rewrite expected hashes automatically
- writes files only after all modules have built successfully
```

External import certificates are never rebuilt by `build-certs`; they are
checked as pinned inputs.

### `package verify-certs`

`package verify-certs` is source-free.

It reads:

```text
- npa-package.toml
- generated/package-lock.json
- local module certificate files
- external import certificate files
```

It does not read:

```text
- `.npa` source
- replay files
- meta files
- theorem index
- axiom report artifact from CLR-05
- AI trace
- registry metadata
```

Verification flow:

```text
1. Validate package manifest.
2. Load and validate `generated/package-lock.json`.
3. Rebuild the package lock from certificate bytes and compare it to the
   checked-in lock.
4. Call the CLR-03 package verifier with checker mode `reference` or `fast`.
5. Render deterministic per-module status and aggregate status.
```

`--checker reference` must use CLR-03 reference source-free verification. It
must build import stores from previous same-checker successful results. It must
not delegate high-trust package verification to unchecked `npa-checker-ref`
directory scanning.

`--checker fast` is allowed for local development, but output must make clear
that it is not a reference checker verdict.

### Command Output Artifacts

CLR-04 commands may report artifacts, but only these files are written by
default command behavior:

```text
package build-certs
  local module certificate files, only in write mode
  generated/package-lock.json, only in write mode after successful build
```

No CLR-04 command writes:

```text
npa-package.toml
generated/axiom-report.json
generated/theorem-index.json
generated/publish-plan.json
registry metadata
source files
AI sidecars
```

CLR-05 and CLR-06 own axiom report, theorem index, and publish metadata
commands.

---

## Tasks

### CLR-04-01 Create `npa-cli` Crate And Package Command Parser

- Status: Completed
- Depends on: CLR-03
- Inputs:
  - `doc/community-library-roadmap-clr-00-todo.md`
  - root `Cargo.toml`
  - existing crate layout under `crates/`
- Code or documentation areas:
  - `Cargo.toml`
  - `crates/npa-cli/Cargo.toml`
  - `crates/npa-cli/src/main.rs`
  - `crates/npa-cli/src/lib.rs`
  - `crates/npa-cli/src/args.rs`
- Deliverables:
  - Workspace member `crates/npa-cli`.
  - Cargo package name `npa-cli`.
  - Binary name `npa`.
  - Parser for `package check`, `package build-certs`, `package verify-certs`, and `package check-hashes`.
  - Common `--root` and `--json` flag parsing.
  - Command-specific parsing for `build-certs --check` and `verify-certs --checker fast/reference`.
- Acceptance criteria:
  - Unknown commands and unsupported flags fail with deterministic usage diagnostics.
  - `--root` defaults to the current directory only.
  - `--checker reference` is the default for `verify-certs`.
  - `--checker external`, `--changed`, `--all`, `--registry`, and `--network` fail as unsupported in CLR-04.
  - `npa-cli` does not become a dependency of `npa-package`, `npa-api`, `npa-cert`, `npa-kernel`, or `npa-checker-ref`.
- Verification:
  - `cargo test -p npa-cli package_cli_args`
  - `cargo run -p npa-cli -- package check --help`
  - `cargo tree -p npa-cli`
- Notes:
  - A small manual parser is acceptable for this command set if diagnostics stay structured and deterministic.

### CLR-04-02 Implement Shared Package Root Loader And Diagnostics

- Status: Completed
- Depends on: CLR-04-01
- Inputs:
  - CLR-01 manifest parser and validator
  - CLR-03 package lock loader and verifier APIs
  - package-relative path rules from CLR-01
- Code or documentation areas:
  - `crates/npa-cli/src/diagnostic.rs`
  - `crates/npa-cli/src/fs.rs`
  - `crates/npa-cli/src/package.rs`
  - tests in `crates/npa-cli`
- Deliverables:
  - Shared package root loader for `npa-package.toml`.
  - Shared package-relative path join helper.
  - Shared command result and diagnostic model.
  - JSON renderer for deterministic `npa.package.command_result.v0.1` output.
  - Exit-code mapping.
- Acceptance criteria:
  - Manifest validation errors from `npa-package` are preserved as structured CLI diagnostics.
  - JSON output contains no absolute host paths, timestamps, environment values, or temp paths.
  - Filesystem read errors identify package-relative paths where possible.
  - Usage errors exit `2`; package/checker failures exit `1`.
- Verification:
  - `cargo test -p npa-cli package_cli_diagnostics`
  - `cargo test -p npa-cli package_root_loader`
  - `cargo run -p npa-cli -- package check --root proofs --json`
- Notes:
  - Keep human output derived from the same structured diagnostics.

### CLR-04-03 Implement `package check`

- Status: Completed
- Depends on: CLR-04-02
- Inputs:
  - `npa-package` manifest validation API
  - proof-corpus package fixture from CLR-02
- Code or documentation areas:
  - `crates/npa-cli/src/package_check.rs`
  - `crates/npa-cli/src/package.rs`
  - `crates/npa-cli/tests/package_cli.rs`
- Deliverables:
  - `cargo run -p npa-cli -- package check --root proofs`.
  - Manifest/schema/profile/path/hash/import graph/policy validation through `npa-package`.
  - JSON and human success/failure output.
  - Tests for valid fixture and representative invalid manifests.
- Acceptance criteria:
  - `package check` reads only `npa-package.toml`.
  - It does not read source files, certificate files, replay files, meta files, generated lock files, theorem indexes, or AI traces.
  - It fails deterministically for unknown imports, import cycles, path escapes, malformed hashes, duplicate modules, and disallowed declared axioms.
  - It succeeds on the proof-corpus package fixture after CLR-02.
- Verification:
  - `cargo run -p npa-cli -- package check --root proofs`
  - `cargo run -p npa-cli -- package check --root proofs --json`
  - `cargo test -p npa-cli package_check`
- Notes:
  - This command is a metadata gate. Certificate hash checking belongs to `check-hashes` and `verify-certs`.

### CLR-04-04 Implement `package check-hashes`

- Status: Completed
- Depends on: CLR-04-02, CLR-04-03
- Inputs:
  - CLR-03 package lock builder
  - local module source files
  - local and external certificate files
  - `generated/package-lock.json`
- Code or documentation areas:
  - `crates/npa-cli/src/package_hashes.rs`
  - `crates/npa-cli/tests/package_cli.rs`
  - proof-corpus generated lock fixture from CLR-03
- Deliverables:
  - `cargo run -p npa-cli -- package check-hashes --root proofs`.
  - Source hash checks for local modules.
  - Certificate file hash checks for local modules.
  - Certificate export, axiom report, and certificate hash checks.
  - External import certificate identity checks.
  - Regenerated package lock comparison against `generated/package-lock.json`.
- Acceptance criteria:
  - Stale source, stale local certificate, stale external certificate, stale canonical hash, or stale package lock fails with a structured hash diagnostic.
  - `check-hashes` does not run the frontend or either checker.
  - `check-hashes` does not rewrite artifacts.
  - The current proof-corpus package passes after CLR-03 generated lock artifacts are present.
- Verification:
  - `cargo run -p npa-cli -- package check-hashes --root proofs`
  - `cargo run -p npa-cli -- package check-hashes --root proofs --json`
  - `cargo test -p npa-cli package_check_hashes`
  - `cargo test --workspace package_lock`
- Notes:
  - This command is the CI freshness gate for checked-in package artifacts.

### CLR-04-05 Implement `package build-certs --check`

- Status: Completed
- Depends on: CLR-04-02, CLR-04-03, CLR-04-04
- Inputs:
  - `npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy`
  - package graph topological order
  - package policy
  - source files and certificate import artifacts
- Code or documentation areas:
  - `crates/npa-cli/src/package_build.rs`
  - `crates/npa-cli/tests/package_cli.rs`
  - shared build helpers if extracted from `tools/proof-corpus`
- Deliverables:
  - In-memory package source build for local modules.
  - External import loading from pinned certificate artifacts.
  - Local import handoff through freshly built verified modules and source interfaces.
  - Certificate byte comparison against checked-in local certificate files.
  - Generated hash comparison against manifest expectations and package lock expectations.
- Acceptance criteria:
  - `--check` writes no files.
  - Build order is dependency-topological and independent of filesystem traversal.
  - The command fails if generated certificate bytes differ from checked-in certificate bytes.
  - The command fails if generated hashes differ from manifest expected hashes.
  - External import certificates are not rebuilt.
  - The current proof-corpus package passes after CLR-02 and CLR-03 are complete.
- Verification:
  - `cargo run -p npa-cli -- package build-certs --root proofs --check`
  - `cargo run -p npa-cli -- package build-certs --root proofs --check --json`
  - `cargo test -p npa-cli package_build_certs_check`
- Notes:
  - Replay and meta files may be read only as untrusted frontend helper data if the package build API explicitly needs them.

### CLR-04-06 Implement `package build-certs` Write Mode

- Status: Completed
- Depends on: CLR-04-05
- Inputs:
  - in-memory build output from CLR-04-05
  - checked-in certificate paths from the package manifest
  - generated package lock path from CLR-03
- Code or documentation areas:
  - `crates/npa-cli/src/package_build.rs`
  - `crates/npa-cli/src/fs.rs`
  - `crates/npa-cli/tests/package_cli.rs`
- Deliverables:
  - Write mode for local module certificate files.
  - Regeneration of `generated/package-lock.json` after successful certificate writes.
  - Atomic or all-or-nothing write strategy for command-owned artifacts where practical.
  - No-op behavior when generated bytes are identical.
- Acceptance criteria:
  - Files are written only after every local module builds successfully.
  - Source files, `npa-package.toml`, expected hashes, external import certificates, axiom reports, theorem indexes, publish plans, and AI sidecars are never written by this command.
  - A partial build failure leaves checked-in certificate artifacts unchanged.
  - Running write mode twice without input changes is idempotent.
- Verification:
  - `cargo run -p npa-cli -- package build-certs --root proofs`
  - `git diff --exit-code -- proofs`
  - `cargo test -p npa-cli package_build_certs_write`
- Notes:
  - If expected hashes need updates after intentional source changes, that is an explicit contributor action or future command, not an implicit CLR-04 write.

### CLR-04-07 Implement `package verify-certs`

- Status: Completed
- Depends on: CLR-04-02, CLR-04-04
- Inputs:
  - CLR-03 package verifier API
  - `generated/package-lock.json`
  - local and external certificate files
  - package policy
- Code or documentation areas:
  - `crates/npa-cli/src/package_verify.rs`
  - `crates/npa-cli/tests/package_cli.rs`
- Deliverables:
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference`.
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast`.
  - Lock reload and regeneration check before verification.
  - Source-free reference checker path.
  - Source-free fast verifier path.
  - Per-module and aggregate structured results.
- Acceptance criteria:
  - Reference mode reads no `.npa` source, replay, meta, theorem index, or AI trace files.
  - Reference mode uses CLR-03 same-checker checked import stores, not unchecked directory scanning alone.
  - Fast mode is explicitly labeled as fast-kernel and not a reference checker verdict.
  - Stale lock or stale certificate hashes fail before successful checker status is reported.
  - Checker rejection is preserved as a structured CLI diagnostic.
- Verification:
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast`
  - `cargo test -p npa-cli package_verify_certs`
  - `cargo test -p npa-api package_source_free`
  - `./scripts/phase8-release-audit.sh`
- Notes:
  - `--checker external` remains target integration and must fail as unsupported in CLR-04.

### CLR-04-08 Add End-To-End CLI Regression Fixtures

- Status: Completed
- Depends on: CLR-04-03, CLR-04-04, CLR-04-05, CLR-04-06, CLR-04-07
- Inputs:
  - proof-corpus package fixture from CLR-02
  - generated package lock from CLR-03
  - CLI commands from CLR-04
- Code or documentation areas:
  - `crates/npa-cli/tests/package_cli.rs`
  - `crates/npa-cli/tests/fixtures/package`
  - `tools/proof-corpus/tests/ai_proof_artifacts.rs` only if cross-crate proof-corpus assertions are needed
- Deliverables:
  - End-to-end tests for all four commands on the proof-corpus package.
  - Temp fixture tests for invalid manifest, stale source hash, stale certificate file hash, stale package lock, source-free verification with source files removed, unsupported checker, and unsupported flags.
  - Snapshot-like assertions for JSON diagnostic shape without depending on host paths.
- Acceptance criteria:
  - A fresh checkout can run the four CLR-04 command examples successfully.
  - Source-free verification still succeeds when source/replay/meta files are absent from a temp copy but certificate artifacts and package metadata are valid.
  - Invalid cases fail with exit code `1` for package/checker failures and `2` for usage failures.
  - Test fixtures do not mutate checked-in `proofs/` unless the command under test is explicitly write mode in a temp copy.
- Verification:
  - `cargo test -p npa-cli package_cli`
  - `cargo test --workspace package_cli`
  - `cargo run -p npa-cli -- package check --root proofs`
  - `cargo run -p npa-cli -- package build-certs --root proofs --check`
  - `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference`
  - `cargo run -p npa-cli -- package check-hashes --root proofs`
- Notes:
  - Keep full-package behavior as the CLR-04 test target. Changed-module selection belongs to later CI work.

### CLR-04-09 Update Documentation And CLR-05 Handoff

- Status: Completed
- Depends on: CLR-04-08
- Inputs:
  - `doc/community-library-roadmap-todo.md`
  - `doc/community-library-roadmap.md`
  - `README.md`
  - `proofs/README.md`
- Code or documentation areas:
  - README command examples
  - `proofs/README.md`
  - `doc/community-library-roadmap-todo.md`
- Deliverables:
  - Contributor-facing examples using installed `npa package ...`.
  - Repository verification examples using `cargo run -p npa-cli -- package ...`.
  - Documentation of source-reading versus source-free command boundaries.
  - Note that CLR-05 owns `axiom-report` and `index`, and CLR-06 owns `publish-plan`.
- Acceptance criteria:
  - Documentation does not imply `verify-certs` reads source.
  - Documentation does not imply CLI output is proof evidence.
  - Documentation does not mention registry fetch or dependency solving for CLR-04 commands.
  - Parent roadmap points to this detailed CLR-04 task document.
- Verification:
  - `rg -n "npa package check|build-certs|verify-certs|check-hashes|source-free|npa-cli" README.md doc proofs/README.md`
  - `git diff --check`
- Notes:
  - Keep `axiom-report`, `index`, and `publish-plan` examples marked as later milestones until they are implemented.

---

## Review Findings

Review pass 1 findings and fixes:

```text
Finding: `verify-certs` and `build-certs` both touch certificates, but only
`build-certs` is allowed to read `.npa` source.
Fix: The command specification has an explicit source-reading matrix:
`build-certs` may read source, while `verify-certs` is source-free and has a
boundary regression test.

Finding: The parent roadmap names `build-certs --check`, but does not define
write behavior.
Fix: CLR-04-05 defines no-write check mode; CLR-04-06 defines write mode and
forbids implicit manifest hash rewrites.

Finding: CLI diagnostics suitable for CI require stable machine-readable output,
not only human text.
Fix: The specification adds `npa.package.command_result.v0.1`, diagnostic
categories, reason codes, and exit-code mapping.

Finding: Earlier source roadmap drafts mentioned `--changed`, `--all`, and
external checker mode, but the parent CLR-04 milestone only requires
full-package fast and reference verification.
Fix: CLR-04 explicitly rejects `--changed`, `--all`, and `--checker external`
as unsupported; later CI/external-checker milestones can add them.

Finding: Generic `build-certs` may need imported source interface data while
the package manifest primarily pins certificates.
Fix: The build spec uses local build outputs and certificate-derived fallback
interfaces for imports; richer source-interface sidecars are not introduced in
CLR-04.
```

Review pass 2 result:

```text
No remaining findings. The task sequence now fixes command parsing, root
loading, diagnostics, command-specific file access, source-free verification,
write-mode safety, end-to-end tests, and handoff boundaries for CLR-05.
```
