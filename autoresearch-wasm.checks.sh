#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

cargo test
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features html
cargo check -p jumpcut-wasm --target wasm32-unknown-unknown --release --no-default-features --features fdx
./autoresearch-wasm.node.sh --smoke >/dev/null
