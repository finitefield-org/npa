# npa-checker-ext OCaml Skeleton

This directory is the clean-room OCaml project for `npa-checker-ext`.
It is intentionally outside the Cargo workspace and has no Rust crate
dependency.

## Commands

```sh
scripts/build.sh
_build/npa-checker-ext --version
scripts/test.sh
scripts/test.sh sha256
scripts/test.sh feature-policy
scripts/test.sh axiom-report
scripts/test.sh axiom-policy-parse
scripts/test.sh decoder-bytes
scripts/test.sh decoder-header
scripts/test.sh decoder-tables
scripts/test.sh decoder-declarations
scripts/test.sh decoder-reachability
scripts/test.sh hash-encoder
scripts/test.sh hash-level-term
scripts/test.sh hash-declarations
scripts/test.sh hash-module
scripts/test.sh import-store
scripts/test.sh inductive-constructors
scripts/test.sh positivity
scripts/test.sh recursor
```

`scripts/build.sh` builds one executable at `_build/npa-checker-ext` using
`ocamlc`. Generated files stay under `_build/`.

Set `OCAMLC=/path/to/ocamlc` when `ocamlc` is not on `PATH`. On macOS the
scripts also check Homebrew's `ocaml` prefix.

## M0-02 Scope

The current executable is a skeleton. It provides deterministic CLI behavior
for `--version`, deterministic errors for incomplete CLI input, and a stable
failed raw result for complete check-shaped invocations. Certificate decoding,
import resolution, type checking, and axiom policy enforcement are implemented
by later milestones.

M0-03 adds a vendored SHA-256 implementation in `src/ext_sha256.ml`. It is used
by `src/ext_hash.ml` and by the checker build hash material.

M0-04 fixes the first-release CLI boundary:

```text
--cert path
--import-dir path
--policy path
--output json
--version
```

`--version` must be used alone and prints deterministic build identity fields.
Check-shaped invocations write only checker raw result JSON to stdout.

M0-05 pins the first-release core feature policy. The supported core feature set
is empty, so `quotient_v1`, `quotient_v2`, and `quotient_v3` certificate feature
reports fail deterministically with `unsupported_core_feature`. This policy is
driven only by the canonical certificate feature report; AI sidecars, package
metadata, and source-derived data cannot enable features. Adding quotient support
requires expanding fast-kernel, reference-checker, and external-checker golden
corpora before the feature is enabled. The feature policy contract is included
in `--version` build identity material.

M1-01 adds the source-free byte reader foundation. `src/ext_bytes.ml` tracks
certificate section and byte offsets, keeps input bytes immutable after reader
construction, and rejects malformed canonical unsigned LEB128 with structured
decode errors. The byte reader has no filesystem or JSON output dependency.

M1-02 adds source-free header and name grammar decoding. The decoder requires
`NPA-CERT-0.1` and `NPA-Core-0.1`, decodes names into structured components,
and rejects invalid UTF-8, empty names, empty components, dotted components,
and duplicate name table entries with structured reasons.

M1-03 adds source-free level and term table decoding. Level and term nodes are
kept as OCaml algebraic data types, not source text. The decoder rejects
unknown tags, dangling table references, non-normalized level entries,
duplicate term entries, and unresolved universe metavariable names before
semantic checking.

M1-04 adds source-free decoding for imports, declarations, export block, axiom
report, optional core feature report, and stored module hash trailer. Declaration
payloads, dependencies, axiom references, export entries, and hash fields are
kept as structured OCaml values. Duplicate declaration names and export-local
dangling term/declaration references reject deterministically; axiom report
length mismatches are decoded and preserved for later axiom-report validation.

M1-05 validates decoded module table reachability and canonical order before a
module is accepted. The validator marks roots from the header, imports,
declarations, exports, and axiom report, traverses reachable terms and levels,
rejects unused name/level/term table entries, enforces canonical table ordering,
and rejects bytes after the module hash trailer.

M2-01 adds canonical hash input encoders in `src/ext_canonical.ml`. These
encoders produce domain-separated inputs for level, term, declaration
dependency, axiom dependency, declaration payload, export block, and axiom
report hashing without reading source spans, debug sidecars, filesystem paths,
pretty printers, or JSON output.

M2-02 makes level and term table hash recomputation table-order based. Child
level and term references must resolve to already hashed table entries, and the
`hash-level-term` fixture pins the expected Rust-compatible hash bytes plus
dependent-hash mutation behavior.

M2-03 recomputes stored declaration interface and declaration certificate
hashes from decoded canonical certificate data. The `hash-declarations` fixture
checks golden certificates and rejects mutations to declaration type, body,
dependency, and axiom dependency material with deterministic declaration-section
offsets.

M2-04 recomputes the final export, axiom report, and module certificate hashes.
The export hash is checked against an export block rebuilt from declaration
interfaces, while the module certificate hash uses the exact original
certificate bytes before the stored certificate hash.

M3-01 adds an explicit source-free import store loader. `--import-dir` style
directories are traversed only for `.npcert` files, each import certificate is
decoded and hash-verified before its public environment is exposed, and duplicate
module/export-hash bindings are rejected.
