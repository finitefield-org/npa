# npa-checker-ext OCaml Skeleton

This directory is the clean-room OCaml project for `npa-checker-ext`.
It is intentionally outside the Cargo workspace and has no Rust crate
dependency.

## Commands

```sh
scripts/build.sh
_build/npa-checker-ext --version
scripts/test.sh
```

`scripts/build.sh` builds one executable at `_build/npa-checker-ext` using
`ocamlc`. Generated files stay under `_build/`.

## M0-02 Scope

The current executable is a skeleton. It provides deterministic CLI behavior
for `--version`, deterministic errors for incomplete CLI input, and a stable
failed raw result for complete check-shaped invocations. Certificate decoding,
hashing, import resolution, type checking, and axiom policy enforcement are
implemented by later milestones.
