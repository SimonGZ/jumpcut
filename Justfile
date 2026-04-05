set shell := ["bash", "-cu"]

verify:
    uv run ./tools/verify.py

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

# Compare script parity; case_name defaults to the script's filename (no extension)
compare script_path pdf_path case_name=file_stem(script_path):
    uv run tools/check_corpus_pdf_parity.py \
      --ignore-case \
      --no-default-cases \
      --case {{case_name}} {{script_path}} {{pdf_path}}
