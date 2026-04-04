# Final Draft Pagination Specification

This is a living document formalizing the mechanics of screenplay pagination and element rendering modeled after industry-standard programs (e.g., Final Draft, Highland).

## 1. Page Geometry

- **Page Size**: Configurable. Typically US Letter (8.5" x 11").
- **Margins**: Configurable constraints for the physical layout.
  - _Edge Margins_: Set to 1.0 inch on Left and Right.
  - _Vertical Margins_:
    - Header and Footer are strictly 0.5 inches from the top and bottom edge.
    - Text margins for the body are **1.0 inch** on Top and Bottom.
  - _Print Area_: A standard 11-inch screenplay page affords exactly 9 inches of vertical space for body text elements.

## 2. Font & Measurement

- **Typesetting**: Standard format is Courier (or Courier-equivalent monospaced typeface, in our case Courier Prime).
- **Size**: Usually 12-point.
- **Horizontal Pitch**: Exactly 10 characters per inch (10 CPI).
- **Vertical Pitch/Line Height**: Approximately 6 lines per inch (LPI).
  - _Line Leading (Spacing)_: Configurable (default 1.0). Supported values are 1.0, 1.5, and 2.0. Extra space is added *above* each visual line of text within an element (but not below).

## 3. Element Widths and Indents

Each element type conforms to specific structural constraints defined relative to the left margin of the page. Different scripts may have different margin settings for elements but these are **the defaults**:

- **Action**
  - Left Indent: 1.5 inches
  - Right Indent: 7.5 inches
  - Available Width: 6.0 inches (Should be 60 characters, but Final Draft allows 61, so we copy that)
- **Character**
  - Left Indent: 3.5 inches
  - Right Indent: 7.25 inches (Multi-Cam: 6.25 inches)
  - Available Width: 4.25 inches
- **Dialogue**
  - Left Indent: 2.5 inches (Multi-Cam: 2.25 inches)
  - Right Indent: 6.0 inches
  - Available Width: 3.5 inches (35 characters)
- **Lyric**
  - Left Indent: 2.5 inches
  - Right Indent: 7.375 inches
  - _Usually presented in italics._
- **Parenthetical**
  - Left Indent: 3.0 inches (Multi-Cam: 2.75 inches)
  - Right Indent: 5.5 inches
  - Available Width: 2.5 inches (Should be 25 characters, but Final Draft allows 26, so we copy that)
- **Dual Dialogue**
  - _Configured as two side-by-side columns._
  - **Left Column**
    - Character cues do **not** use a fixed left indent. The best current human model is that Final Draft tries to keep them visually centered on a 3-inch anchor, then snaps the resulting left edge through its own quantization rules.
    - The current measured model that reproduces those probe points is:
      - base left indent of 2.875 inches for a 1-character cue
      - subtract 3/64 inch per additional character
      - round to the nearest 1/16 inch
      - clamp wrapping width to 29 characters
    - Example measured left indents:
      - 1 character: 2.875 inches
      - 2 characters: 2.8125 inches
      - 4 characters: 2.75 inches
      - 9 characters: 2.5 inches
      - 26 characters: 1.6875 inches
      - 29 characters: 1.5625 inches
    - Parenthetical Left Indent: 1.75 inches
    - Parenthetical Right Indent: 4.125 inches
    - Dialogue Left Indent: 1.5 inch
    - Dialogue Right Indent: 4.375 inches
    - Available Width: 2.875 inches (Should be 28 characters, but Final Draft allows 29, so we copy that)
  - **Right Column**
    - Character cues appear to follow the same centering behavior around a 6-inch anchor.
    - The current measured left-indent model is the left-column formula shifted 3.125 inches further right.
    - Parenthetical Left Indent: 4.875 inches
    - Parenthetical Right Indent: 7.25 inches
    - Dialogue Left Indent: 4.625 inches
    - Dialogue Right Indent: 7.5 inches
    - Available Width: 2.875 inches (Should be 28 characters, but Final Draft allows 29, so we copy that)
- **Transition**
  - Left Indent: 5.5 inches
  - Right Indent: 7.1 inches (Multi-Cam: 7.25 inches)

## 4. Line Wrapping

- **Algorithm**: Greedy matching, preserving spaces.
- **Space Rendering**:
  - Trailing spaces at the end of a visual line _do not_ prompt a line break.
  - Internal spaces (e.g., double spaces separating sentences) count towards visual line length, therefore affecting wrapping.

## 5. Intrinsic Spacing

Elements dictate vertical padding above and below themselves by standard multiples of blank lines. The number of blank lines above and below an element is configurable per element type, but there are default values:

- **Spacing Value**:
  - Scene headings require **2 visual blank lines** above and **1 visual blank line** below.
  - Action blocks, and character blocks (grouping character, parenthetical, dialogue, and lyric) require **1 visual blank line** above and below.
  - Dual Dialogue blocks follow the identical rule: **1 visual blank line** above and below.
- **Shared Padding**: Intrinsic padding is not strictly additive. Two consecutive action blocks maintain _one_ blank line between them, not two.
- **Page Boundaries**: Elements placed at the absolute top of a blank page disregard their required top visual spacing metric (no intrinsic blank line added at top). Elements placed at the absolute bottom of a page disregard their bottom visual spacing metric (no intrinsic blank line added at bottom).

## 6. Page Breaks and Flow Control

Pages are filled until visual height is exhausted, prompting a hard pagination move.

### 6.1 Stranding and Splitting

- **Scene Headings**: Must _never_ sit alone at the bottom of the page. It must be paired with at least a small portion of subsequent action or dialogue.
- **Transitions**: Allowed to be placed as a single isolated line at the end of a page, but they _should avoid_ being placed as the first element of a page with no content above them.
- **Orphan and Widow Thresholds**:
  - Flow/action splits require a minimum of **2 visual lines** on the
    terminating page and **2 visual lines** on the next page.
  - Dialogue splits use **2 content lines** on each side by default, counting
    dialogue, lyric, and parenthetical lines but not character cues.
- **Splitting Logic**: Programs avoid ungainly "stranding." When splitting action or dialogue near the bottom of a page, splits should favor a sentence boundary where possible. The algorithm would rather push an entire element block onto the next page or slice at an earlier junction for a more elegant partition rather than stranding small bits of content at the top of the next page.
  - In JumpCut's current parity model, ordinary action splits still reject a
    short runt final top line by default.
  - There is one narrower exception: when a **scene heading** would otherwise
    be forced onto the next page with no scene content, JumpCut allows the
    following action to split under the normal action rules, including an
    exact-fit sentence-ending top fragment even if that top fragment ends on a
    short final line. This is intentionally scoped to the scene-heading case
    rather than ordinary action blocks.
  - **Minimum block size for scene-heading splits**: This keep-with-next split
    is only attempted when the following action block has at least
    `orphan_limit + widow_limit + 1` content lines (5 lines with default
    geometry). Shorter blocks are too thin to split meaningfully — sentence-boundary
    re-wrapping can inflate each fragment's apparent line count just enough to
    satisfy orphan/widow checks, but the resulting split is not defensible. Final
    Draft pushes the entire scene heading + action together to the next page
    instead.
  - Explicitly empty **Action** paragraphs render as **one blank visual line**,
    not zero lines. They still follow the normal spacing-above rules, so they
    affect page budget like any other one-line action beat.

### 6.2 Configurable Elements

- **Dialogue Continuation**: Programs optionally allow for continuing split dialogue pages with automatic markers.
  - _(MORE)_ inserted at the bottom flap of the first page.
  - Character name repeated at the top of the next page with a _(CONT'D)_ tag.
  - In JumpCut's current model, these continuation markers do not consume the
    formal page-height budget.

### 6.3 Forced Page Breaks

- **Forced Page Breaks**: Manual hard returns (e.g., `===`) immediately truncate the current page. The page stops accepting elements, and the paginator moves onto the next page starting with the next element.
