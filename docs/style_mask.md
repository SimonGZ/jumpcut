# StyleMask Bitset Guide

The inline styling pipeline now stores active styles in a compact `StyleMask` rather than `HashSet<String>`. This note captures the design so future tweaks stay consistent.

## Goals
- Eliminate per-run allocations and hashing when toggling bold/italic/underline styles.
- Keep serialization output identical (`["Bold", "Italic", ...]`) for HTML/FDX templates.
- Provide an easy path to add new inline styles while keeping the parser logic simple.

## Representation
`StyleMask` is a newtype over `u8` defined in `src/lib.rs`. Each bit represents a style:

```
Bit 0 -> Bold
Bit 1 -> Italic
Bit 2 -> Underline
```

Three helper constants (`StyleMask::BOLD`, `StyleMask::ITALIC`, `StyleMask::UNDERLINE`) make it clear which bit is being toggled. By using a byte, we leave five spare bits for future inline options (strike-through, highlight, etc.).

## Operations
- `insert(mask)`: bitwise ORs the requested flag (`self.0 |= mask.0`), enabling a style.
- `remove(mask)`: clears bits with `self.0 &= !mask.0` when a sentinel closes a style span.
- `contains(mask)`: checks whether all bits from `mask` are active, enabling fast lookups inside `create_styled_from_string`.
- `union(other)`: returns a new `StyleMask` composed of the current bits and another flag set. Used when bold+italic toggles fire from the combined sentinel.
- `iter()`: yields style names in a stable order (`Bold`, `Italic`, `Underline`) so serialization remains deterministic.

These methods keep toggling logic in `src/text_style_parser.rs` straightforward: when a sentinel is encountered we flush the current text run, call `insert` or `remove`, and resume collecting characters. No cloning or HashSet allocation is needed.

## Serialization
`text_style_serialize` converts the mask back into a `Vec<&'static str>` by calling `iter()` and collecting the names. Criterion benchmarks confirm that removing the HashSet/sort lowered the hot-path cost for dense markup by ~54%.

## Extending Styles
To add a new inline marker:
1. Add a new constant bit flag to `StyleMask` (e.g., `const STRIKE: StyleMask = StyleMask(1 << 3);`).
2. Update the `ORDERED_STYLES` array inside `StyleMask::iter()` to include the new mask/name pair.
3. Map the new sentinel to the mask in `create_styled_from_string` and update delimiter classification in `src/text_style_parser.rs`.
4. Add tests confirming the new style toggles correctly.

Because `StyleMask` is `Copy`, these changes keep runs cheap to duplicate while staying ABI-friendly for existing WASM bindings.
