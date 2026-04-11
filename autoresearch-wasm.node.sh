#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Compatibility wrapper for the older research-era script name.
"$SCRIPT_DIR/generate-wasm-package.sh" \
    --out-dir "$SCRIPT_DIR/target/autoresearch-wasm/node-full" \
    "$@"
