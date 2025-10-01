default:
    cargo build --release

wasm-browser:
    wasm-pack build --target web -- --no-default-features --features wasm

wasm-node:
    wasm-pack build --target nodejs -- --no-default-features --features wasm
