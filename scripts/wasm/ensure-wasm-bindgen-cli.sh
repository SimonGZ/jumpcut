#!/usr/bin/env bash
set -euo pipefail

VERSION="${WASM_BINDGEN_CLI_VERSION:-0.2.92}"

find_wasm_bindgen() {
    local candidate

    if candidate="$(command -v wasm-bindgen 2>/dev/null)"; then
        printf '%s\n' "$candidate"
        return 0
    fi

    for candidate in \
        "${CARGO_HOME:-$HOME/.cargo}/bin/wasm-bindgen" \
        "/ductor/runtime/cache/cargo/bin/wasm-bindgen"
    do
        if [[ -x "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    return 1
}

if ! wasm_bindgen_bin="$(find_wasm_bindgen)"; then
    >&2 echo "Installing wasm-bindgen-cli v${VERSION} because it is required for the Node/JS wasm benchmark harness."
    cargo install --locked "wasm-bindgen-cli" --version "$VERSION" >/dev/null
    wasm_bindgen_bin="$(find_wasm_bindgen)"
fi

printf '%s\n' "$wasm_bindgen_bin"
