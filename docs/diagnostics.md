# Diagnostics

This document covers the repo's internal pagination and PDF parity tooling.

## Pagination Diagnostics

The pagination/parity harness includes ignored tests and dedicated tooling that generate review packets and debug artifacts under:

```text
target/pagination-debug/
```

If you want a single command that rebuilds the main diagnostics set, install [`just`](https://github.com/casey/just):

```sh
cargo install just
```

Then run:

```sh
just pagination-diagnostics
```

That recipe calls the dedicated Rust diagnostics tool directly:

```sh
cargo run --bin pagination-diagnostics -- all
```

The main run currently regenerates:

- Big Fish review packet
- Big Fish full-script page-break review packet
- Big Fish line-break parity packet
- Little Women windowed review packet
- Little Women full-script page-break review packet
- Little Women line-break parity packet
- Mostly Genius full-script page-break review packet
- Mostly Genius line-break parity packet
- the extra paginated-output JSON dumps and visual-comparison export used for manual debugging

There are also narrower convenience tasks:

```sh
just big-fish-diagnostics
just mostly-genius-diagnostics
```

## PDF Parity Checks

The repo also includes a PDF word-position parity checker:

```sh
python3 tools/check_corpus_pdf_parity.py
```

That runs the wired corpus PDF checks and writes reports under:

```text
target/pdf-placement-diagnostics/
```

If you want to compare an ad hoc script without adding it to the permanent corpus, use one or more `--case` entries:

```sh
python3 tools/check_corpus_pdf_parity.py \
  --no-default-cases \
  --case my-script /abs/path/to/script.fountain /abs/path/to/reference.pdf
```

You can provide `--case` more than once in the same run. The script generates a PDF with JumpCut, compares it against the supplied reference PDF using `pdftotext -bbox-layout`, and writes a report packet for each case.

If a reference differs only in letter case, you can opt into case-insensitive text matching before geometry comparison:

```sh
python3 tools/check_corpus_pdf_parity.py \
  --no-default-cases \
  --ignore-case \
  --case mostly-genius /abs/path/to/mostly-genius.fountain /abs/path/to/mostly-genius.pdf
```

Local dependency:

```sh
sudo apt install poppler-utils
```

That provides `pdftotext`, which the parity checker requires.
