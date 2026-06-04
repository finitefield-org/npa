#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Compatibility wrapper for the pre-PCT-03 corpus gate name. Keep existing
# callers on the full gate while newer authoring loops can use the split gates.
exec bash scripts/check-corpus-full.sh
