#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

TARGET_DIR="$ROOT_DIR/target/autoresearch-wasm"
mkdir -p "$TARGET_DIR"

gzip_size() {
    python3 - "$1" <<'PY'
import gzip
import io
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
buf = io.BytesIO()
with gzip.GzipFile(fileobj=buf, mode="wb", compresslevel=9) as fh:
    fh.write(data)
print(len(buf.getvalue()))
PY
}

emit_variant_metrics() {
    local variant="$1"
    shift

    cargo build -p jumpcut-wasm --target wasm32-unknown-unknown --release "$@" >/dev/null

    local raw_wasm="$ROOT_DIR/target/wasm32-unknown-unknown/release/jumpcut_wasm.wasm"
    local variant_raw="$TARGET_DIR/jumpcut_wasm.${variant}.wasm"
    local variant_opt="$TARGET_DIR/jumpcut_wasm.${variant}.opt.wasm"

    cp "$raw_wasm" "$variant_raw"
    wasm-opt -Os "$variant_raw" -o "$variant_opt"

    local raw_bytes
    local opt_bytes
    local raw_gzip_bytes
    local opt_gzip_bytes
    raw_bytes="$(wc -c < "$variant_raw" | tr -d ' ')"
    opt_bytes="$(wc -c < "$variant_opt" | tr -d ' ')"
    raw_gzip_bytes="$(gzip_size "$variant_raw")"
    opt_gzip_bytes="$(gzip_size "$variant_opt")"

    if [[ "$variant" == "full" ]]; then
        echo "METRIC wasm_bytes=$raw_bytes"
        echo "METRIC wasm_opt_os_bytes=$opt_bytes"
        echo "METRIC wasm_gzip_bytes=$raw_gzip_bytes"
        echo "METRIC wasm_opt_os_gzip_bytes=$opt_gzip_bytes"
    else
        echo "METRIC wasm_${variant}_bytes=$raw_bytes"
        echo "METRIC wasm_${variant}_opt_os_bytes=$opt_bytes"
        echo "METRIC wasm_${variant}_gzip_bytes=$raw_gzip_bytes"
        echo "METRIC wasm_${variant}_opt_os_gzip_bytes=$opt_gzip_bytes"
    fi
}

emit_variant_metrics full
emit_variant_metrics json_only --no-default-features
emit_variant_metrics html_only --no-default-features --features html
emit_variant_metrics fdx_only --no-default-features --features fdx
emit_variant_metrics pdf_only --no-default-features --features pdf

cargo run --release --quiet --bin autoresearch_native_bench
./generate-wasm-package.sh --out-dir "$TARGET_DIR/node-full"
