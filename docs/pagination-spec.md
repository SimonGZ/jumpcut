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
- **Vertical Pitch/Line Height**: Approximately 6 lines per inch (LPI). _Note: Final Draft line height often hovers between 13-15pt. Exact calibration needed._

## 3. Element Widths and Indents

Each element type conforms to specific structural constraints defined relative to the left margin of the page. Different scripts may have different margin settings for elements but these are **the defaults**:

- **Action**
  - Left Indent: 1.5 inches
  - Right Indent: 7.5 inches
  - Available Width: 6.0 inches (60 characters)
- **Character**
  - Left Indent: 3.5 inches
  - Right Indent: 7.25 inches
  - Available Width: 4.25 inches
- **Dialogue**
  - Left Indent: 2.5 inches
  - Right Indent: 6.0 inches
  - Available Width: 3.5 inches (35 characters)
- **Lyric**
  - Left Indent: 2.5 inches
  - Right Indent: 7.375 inches
  - _Usually presented in italics._
- **Parenthetical**
  - Left Indent: 3.0 inches
  - Right Indent: 5.5 inches
- **Dual Dialogue**
  - _Configured as two side-by-side columns._
  - **Left Column**
    - Character Left Indent: 2.5 inches
    - Character Right Indent: 4.875 inches
    - Dialogue Left Indent: 1.0 inch
    - Dialogue Right Indent: 4.875 inches
  - **Right Column**
    - Character Left Indent: 5.875 inches
    - Character Right Indent: 7.5 inches
    - Dialogue Left Indent: 5.125 inches
    - Dialogue Right Indent: 7.5 inches
- **Transition**
  - Left Indent: 5.5 inches
  - Right Indent: 7.1 inches

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
- **Transitions**: Allowed to be placed as a single isolated line at the end of a page.
- **Orphan and Widow Thresholds**: Action or dialogue splits require a minimum of **2 visual lines** on the terminating page, and **2 visual lines** on the sequential next page. _(Note: This is a working hypothesis and has not been fully confirmed.)_
- **Splitting Logic**: Programs avoid ungainly "stranding." When splitting action or dialogue near the bottom of a page, splits should favor a sentence boundary where possible. The algorithm would rather push an entire element block onto the next page or slice at an earlier junction for a more elegant partition rather than stranding small bits of content at the top of the next page.

### 6.2 Configurable Elements

- **Dialogue Continuation**: Programs optionally allow for continuing split dialogue pages with automatic markers.
  - _(MORE)_ inserted at the bottom flap of the first page.
  - Character name repeated at the top of the next page with a _(CONT'D)_ tag.

### 6.3 Forced Page Breaks

- **Forced Page Breaks**: Manual hard returns (e.g., `===`) immediately truncate the current page. The page stops accepting elements, and the paginator moves onto the next page starting with the next element.
