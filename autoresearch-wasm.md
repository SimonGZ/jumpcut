# Jumpcut WASM Autoresearch Session

Branch:

- `feature/wasm-baseline`

Objective:

- establish a reproducible wasm baseline for the in-repo `jumpcut-wasm` wrapper crate
- measure wasm size and optimized wasm size
- preserve native parser performance guardrails while optimizing for wasm

Current crate shape:

- core published crate: `jumpcut`
- wrapper crate: `jumpcut-wasm`

Primary metrics from `./autoresearch-wasm.sh`:

- `wasm_bytes`
- `wasm_opt_os_bytes`
- `wasm_gzip_bytes`
- `wasm_opt_os_gzip_bytes`

Native guardrail metrics from `./autoresearch-wasm.sh`:

- `parse_108_ns`
- `parse_big_fish_ns`
- `total_ns`

Validation command:

- `./autoresearch-wasm.checks.sh`

Current guardrail rule:

- discard wasm-size changes that meaningfully regress native parser performance unless the tradeoff is explicitly accepted

Notes:

- HTML and FDX remain part of the intended wasm use case
- this is not a parser-only wasm target
- old `origin/feature/wasm` ideas are inputs to test, not a baseline to port wholesale
