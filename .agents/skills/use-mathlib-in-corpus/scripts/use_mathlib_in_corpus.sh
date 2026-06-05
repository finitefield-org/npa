#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bin="${TMPDIR:-/tmp}/npa-use-mathlib-in-corpus-$$"
trap 'rm -f "$bin"' EXIT

rustc --edition=2021 "$script_dir/use_mathlib_in_corpus.rs" -o "$bin"
exec "$bin" "$script_dir" "$@"
