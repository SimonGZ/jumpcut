# Styled Text Fidelity Plan

This document lays out the next rendering refactor needed to restore inline
style fidelity in exact-wrap HTML and to prepare the same structures for future
PDF output.

## Problem

JumpCut's pagination pipeline currently drops inline style structure before it
reaches wrapping and page layout:

- [normalized.rs](/ductor/workspace/jumpcut/src/pagination/normalized.rs)
  collapses `ElementText::Styled` into plain strings in `flatten_text`
- [semantic.rs](/ductor/workspace/jumpcut/src/pagination/semantic.rs)
  then builds semantic units from those flattened strings
- [wrapping.rs](/ductor/workspace/jumpcut/src/pagination/wrapping.rs)
  wraps plain `&str` input into plain `WrappedLine { text, end_offset }`
- [visual_lines.rs](/ductor/workspace/jumpcut/src/visual_lines.rs)
  reconstructs exact visual rows from those plain wrapped strings

That is why exact-wrap and paginated HTML regained line-break fidelity but lost
bold / italic / underline fidelity.

## Guiding Rule

Styled text must be preserved as rendering metadata, not as width-affecting
content.

Important user rule:

- screenplay emphasis does **not** change character width
- bold, underline, and italic remain the same monospaced advance as plain text
- wrap decisions should therefore be based on plain text content / offsets, not
  on font-style-specific metrics

This is good news architecturally: we can keep the existing width model and add
styled fragment tracking on top of it.

## Goal

Create one shared rendering IR for wrapped styled lines so that:

- exact-wrap HTML can restore inline emphasis without giving up layout fidelity
- paginated HTML can use the same styled lines and page fragments
- future PDF output can consume the exact same wrapped styled-line structures
- text output and diagnostics can continue to derive plain-string views from the
  same canonical structures

## Core Design

Introduce a new renderer-facing layer between wrapping and output backends.

Suggested structures:

```rust
pub struct StyledText {
    pub plain_text: String,
    pub runs: Vec<StyledRun>,
}

pub struct StyledRun {
    pub start: usize,
    pub end: usize,
    pub styles: Vec<TextStyle>,
}

pub struct WrappedStyledLine {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub fragments: Vec<StyledLineFragment>,
}

pub struct StyledLineFragment {
    pub text: String,
    pub styles: Vec<TextStyle>,
}

pub struct RenderLine {
    pub fragments: Vec<StyledLineFragment>,
    pub counted: bool,
    pub indent_spaces: usize,
    pub element_type: ElementType,
}
```

The important distinction is:

- wrapping still happens against `plain_text`
- style runs are reapplied to the wrapped line ranges by offset
- output backends consume fragments, not flattened line strings

## Best Seam for the Refactor

The smallest stable seam is to keep the current pagination structure mostly
intact and replace the flattening/wrapping boundary.

### Phase A: Preserve structured text in normalization

Today, [normalized.rs](/ductor/workspace/jumpcut/src/pagination/normalized.rs)
stores only `text: String`.

Refactor target:

- `NormalizedElement` should carry both:
  - plain text
  - styled run metadata

This can be done by introducing a shared `StyledText` value and lowering
`ElementText` into it during normalization.

### Phase B: Preserve structured text in semantic units

Today, [semantic.rs](/ductor/workspace/jumpcut/src/pagination/semantic.rs)
stores `String` for:

- `FlowUnit.text`
- `DialoguePart.text`
- `LyricUnit.text`

Refactor target:

- replace those `String` fields with a structured text payload that preserves
  plain text plus run offsets

This keeps the paginator's semantic model intact while stopping style loss.

### Phase C: Upgrade wrapping to emit styled wrapped lines

Today, [wrapping.rs](/ductor/workspace/jumpcut/src/pagination/wrapping.rs)
returns plain `WrappedLine`.

Refactor target:

- keep plain wrapping helpers for compatibility
- add a canonical styled wrapper such as:
  - `wrap_styled_text_for_element(...) -> Vec<WrappedStyledLine>`

This function should:

- reuse the existing plain wrap algorithm and offsets
- slice the source styled runs into per-line fragments
- never let styles alter width calculations

### Phase D: Move exact visual reconstruction onto styled lines

Today, [visual_lines.rs](/ductor/workspace/jumpcut/src/visual_lines.rs) and
[text_output.rs](/ductor/workspace/jumpcut/src/text_output.rs) still work from
indented plain strings.

Refactor target:

- introduce a shared `RenderLine` / `RenderFragment` layer
- preserve:
  - counted vs non-counted line semantics
  - continuation-marker placement
  - indent / alignment metadata
  - styled fragments

Then:

- HTML renders spans from `RenderFragment`
- PDF later draws glyph runs from the same `RenderFragment`
- text output joins fragment text back into strings

## Migration Order

This should be done in tight slices.

### Phase 1: Add structured text types without changing output

Tasks:

- add `StyledText` / `StyledRun` types in a shared module
- lower `ElementText` into that form during normalization
- keep plain-text accessors so the rest of the pipeline still compiles

Exit condition:

- no visible behavior change
- pagination still runs on plain text extracted from the structured form

### Phase 2: Add styled wrapping helpers

Tasks:

- extend [wrapping.rs](/ductor/workspace/jumpcut/src/pagination/wrapping.rs)
  with a styled wrapper API
- keep existing `wrap_text_for_element` and
  `wrap_text_for_element_with_offsets` intact for compatibility
- add tests proving styled runs survive line splits correctly

Exit condition:

- the codebase can produce wrapped lines with fragment-level styles

### Phase 3: Refactor visual-line reconstruction onto styled render lines

Tasks:

- replace plain-string line payloads in
  [visual_lines.rs](/ductor/workspace/jumpcut/src/visual_lines.rs)
  with fragment-based render lines
- add plain-text adapters so
  [text_output.rs](/ductor/workspace/jumpcut/src/text_output.rs)
  still works without changing visible text output

Exit condition:

- one shared render-line layer exists below HTML and text outputs

### Phase 4: Restore styled exact-wrap HTML

Tasks:

- update [html.rs](/ductor/workspace/jumpcut/src/rendering/html.rs)
  exact-wrap and paginated modes to emit spans from render fragments
- preserve current default HTML mode unchanged

Exit condition:

- exact-wrap and paginated HTML show correct inline emphasis again

### Phase 5: Use the same render-line layer for PDF

Tasks:

- define a PDF renderer against the same render lines rather than against raw
  screenplay elements
- keep pagination and wrapping fully shared with HTML

Exit condition:

- PDF becomes a backend, not a second layout engine

## Key Risk Areas

### Continuation markers

[visual_lines.rs](/ductor/workspace/jumpcut/src/visual_lines.rs) currently
injects:

- `(MORE)`
- repeated `CHARACTER (CONT'D)` prefixes

Those lines need a clear rule:

- they should remain renderer-generated lines
- they do not need styled-source provenance
- they should still fit into the same `RenderLine` structure

### Dual dialogue

Dual dialogue is currently synchronized into visible row strings in
[visual_lines.rs](/ductor/workspace/jumpcut/src/visual_lines.rs).

When styled fragments are added, the two sides should not be flattened into one
style-agnostic string too early. Prefer:

- two independently wrapped styled line streams
- a later visual-row merge step

This will matter for both HTML and PDF.

### All-caps and other text transforms

If all-caps is treated as a visual style rather than literal source text, the
rendering layer must be clear about when transformed text is materialized.

Recommended rule:

- width math uses the exact plain text the screenplay intends to render
- any case transformation must happen before or during wrapping only if it is
  truly part of the rendered content, not a backend-only CSS effect

### Backward compatibility

Diagnostics and text output should not be forced to understand styled fragments
immediately.

Provide adapters:

- `RenderLine -> String`
- `WrappedStyledLine -> WrappedLine`

That keeps migration incremental.

## Recommended First Implementation Slice

The first code slice should be deliberately small:

1. add `StyledText` / `StyledRun`
2. preserve it through normalization and semantic units
3. keep every current output path still consuming plain text
4. add tests proving the style metadata survives unchanged through that path

That gives us the backbone we need before touching the wrap engine itself.
