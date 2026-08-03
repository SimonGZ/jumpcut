#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT_DIR"

cargo test
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features html
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features fdx
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features pdf
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features pdf-font-subsetting
"$SCRIPT_DIR/generate-package.sh" --target nodejs --smoke >/dev/null
node "$SCRIPT_DIR/validate-node.mjs" \
    --pkg-dir "$ROOT_DIR/target/wasm-package/node-full"
