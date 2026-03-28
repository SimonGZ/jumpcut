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
