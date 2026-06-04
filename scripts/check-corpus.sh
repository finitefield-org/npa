#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Compatibility wrapper for the old corpus gate name. The proof corpus is a
# staging workspace, so the default corpus check now follows the lightweight
# authoring gate. Use check-corpus-full.sh explicitly for promotion, release, or
# package-wide compatibility checks.
exec bash scripts/check-corpus-authoring.sh
