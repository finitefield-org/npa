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
