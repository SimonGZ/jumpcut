set shell := ["bash", "-cu"]

pagination-diagnostics:
    cargo run --bin pagination-diagnostics -- all

mostly-genius-diagnostics:
    cargo run --bin pagination-diagnostics -- mostly-genius-linebreak
    cargo run --bin pagination-diagnostics -- mostly-genius-full-script
