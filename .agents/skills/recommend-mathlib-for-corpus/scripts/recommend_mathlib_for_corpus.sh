#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bin="${TMPDIR:-/tmp}/npa-recommend-mathlib-for-corpus-$$"
trap 'rm -f "$bin"' EXIT

rustc --edition=2021 "$script_dir/recommend_mathlib_for_corpus.rs" -o "$bin"
exec "$bin" "$script_dir" "$@"
