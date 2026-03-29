set shell := ["bash", "-cu"]

pagination-diagnostics:
    cargo run --bin pagination-diagnostics -- all

big-fish-diagnostics:
    cargo run --bin pagination-diagnostics -- big-fish-linebreak
    cargo run --bin pagination-diagnostics -- big-fish-review
    cargo run --bin pagination-diagnostics -- big-fish-full-script

mostly-genius-diagnostics:
    cargo run --bin pagination-diagnostics -- mostly-genius-linebreak
    cargo run --bin pagination-diagnostics -- mostly-genius-full-script

fd-probe:
    cargo test --test pagination_fd_probe_test -- --nocapture

fd-probe-new name source:
    python3 scripts/new_fd_probe.py "{{name}}" "{{source}}"

fd-probe-diagnostics:
    cargo run --bin pagination-diagnostics -- fd-probes
