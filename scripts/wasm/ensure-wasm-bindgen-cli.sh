#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOCKED_VERSION="$(awk '
    $0 == "[[package]]" { package_name = "" }
    /^name = / { package_name = $0 }
    package_name == "name = \"wasm-bindgen\"" && /^version = / {
        gsub(/^version = \"|\"$/, "")
        print
        exit
    }
' "$ROOT_DIR/Cargo.lock")"

if [[ -z "$LOCKED_VERSION" ]]; then
    >&2 echo "Could not determine the locked wasm-bindgen version from Cargo.lock."
    exit 1
fi

VERSION="${WASM_BINDGEN_CLI_VERSION:-$LOCKED_VERSION}"

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

wasm_bindgen_bin="$(find_wasm_bindgen || true)"
if [[ -n "$wasm_bindgen_bin" ]] && [[ "$("$wasm_bindgen_bin" --version)" == "wasm-bindgen $VERSION" ]]; then
    printf '%s\n' "$wasm_bindgen_bin"
    exit 0
fi

if [[ -n "$wasm_bindgen_bin" ]]; then
    >&2 echo "Replacing $($wasm_bindgen_bin --version) with wasm-bindgen-cli v${VERSION} to match Cargo.lock."
else
    >&2 echo "Installing wasm-bindgen-cli v${VERSION} because it is required for the Node/JS wasm benchmark harness."
fi

cargo install --locked "wasm-bindgen-cli" --version "$VERSION" --force >/dev/null
wasm_bindgen_bin="$(find_wasm_bindgen)"

if [[ "$("$wasm_bindgen_bin" --version)" != "wasm-bindgen $VERSION" ]]; then
    >&2 echo "Expected wasm-bindgen-cli v${VERSION}, but found $("$wasm_bindgen_bin" --version)."
    exit 1
fi

printf '%s\n' "$wasm_bindgen_bin"
