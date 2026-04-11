#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT_DIR=""
METRIC_PREFIX="wasm_node"
WARMUP=2
SAMPLES=9
TARGET_SAMPLE_MS=40

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root-dir)
            ROOT_DIR="$2"
            shift 2
            ;;
        --out-dir)
            OUT_DIR="$2"
            shift 2
            ;;
        --metric-prefix)
            METRIC_PREFIX="$2"
            shift 2
            ;;
        --warmup)
            WARMUP="$2"
            shift 2
            ;;
        --samples)
            SAMPLES="$2"
            shift 2
            ;;
        --target-sample-ms)
            TARGET_SAMPLE_MS="$2"
            shift 2
            ;;
        --smoke)
            WARMUP=0
            SAMPLES=1
            TARGET_SAMPLE_MS=1
            shift
            ;;
        *)
            >&2 echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

ROOT_DIR="$(cd "$ROOT_DIR" && pwd)"
cd "$ROOT_DIR"

if [[ -z "$OUT_DIR" ]]; then
    OUT_DIR="$ROOT_DIR/target/wasm-package/node-full"
fi

WASM_BINDGEN_BIN="$("$SCRIPT_DIR/ensure-wasm-bindgen-cli.sh")"

cargo build -p jumpcut-wasm --target wasm32-unknown-unknown --release >/dev/null

RAW_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/jumpcut_wasm.wasm"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

"$WASM_BINDGEN_BIN" \
    --target nodejs \
    --out-dir "$OUT_DIR" \
    "$RAW_WASM" >/dev/null

node "$SCRIPT_DIR/bench-node.mjs" \
    --pkg-dir "$OUT_DIR" \
    --fixtures-dir "$ROOT_DIR/benches" \
    --metric-prefix "$METRIC_PREFIX" \
    --warmup "$WARMUP" \
    --samples "$SAMPLES" \
    --target-sample-ms "$TARGET_SAMPLE_MS"
