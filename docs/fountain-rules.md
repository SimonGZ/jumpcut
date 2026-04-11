# Fountain Syntax Rules

This document provides a summary of the Fountain screenplay syntax, designed for technical reference within the JumpCut project.

**Source:** [Fountain Syntax (https://fountain.io/syntax/)](https://fountain.io/syntax/)

---

## 1. Scene Headings
*   **Standard:** Must be preceded by a blank line. Starts with `INT`, `EXT`, `INT./EXT.`, `EST.`, etc.
*   **Forced:** Start a line with a single period (`.`) to force a scene heading regardless of the starting text.
*   **Scene Numbers:** Added at the end of the line, wrapped in `#` (e.g., `#1#` or `#A1#`).

## 2. Action
*   **Standard:** Any paragraph that doesn't match other element types is treated as Action.
*   **Forced:** Start a line with an exclamation mark (`!`) to force it as Action.
*   **Indentation:** Leading tabs or spaces are generally ignored unless forcing a specific layout.

## 3. Characters
*   **Standard:** A line in ALL CAPS, preceded by a blank line, and followed by Dialogue or a Parenthetical.
*   **Forced:** Start a line with `@` to force it as a Character name (useful for names with lowercase letters).
*   **Extensions:** Character names can include extensions in parentheses, like `(O.S.)` or `(V.O.)`.

## 4. Dialogue and Parentheticals
*   **Dialogue:** Follows a Character cue or a Parenthetical.
*   **Parentheticals:** Wrapped in `(parentheses)` and must occur between a Character and Dialogue, or between two Dialogue blocks.
*   **Dual Dialogue:** Add a caret (`^`) at the end of the second Character cue to indicate it should be rendered side-by-side with the preceding block.

## 5. Transitions
*   **Standard:** Must be in ALL CAPS, preceded by a blank line, and end in `TO:`.
*   **Forced:** Start a line with a greater-than symbol (`>`) to force a Transition (useful for "FADE OUT" or other non-standard cues).

## 6. Formatting and Lyrics
*   **Emphasis:** Supports standard Markdown:
    *   `*Italic*`
    *   `**Bold**`
    *   `***Bold Italic***`
    *   `_Underline_`
*   **Lyrics:** Start a line with a tilde (`~`) to indicate sung lyrics.
*   **Centered Text:** Wrap text in greater-than/less-than symbols: `> Center This <`.

## 7. Notes and Comments
*   **Notes:** Wrapped in double brackets: `[[This is a note]]`.
*   **Boneyard:** Block comments wrapped in `/*` and `*/`.
*   **Page Breaks:** A line with three or more equals signs `===` forces a page break.

## 8. Title Page
*   Metadata is placed at the very top of the document.
*   Format: `Key: Value`.
*   Keys like `Title`, `Credit`, `Author`, `Source`, `Draft date`, `Contact`.
*   Multiline values are supported by indenting subsequent lines.

## 9. Sections and Synopses
*   **Sections:** Start with one or more `#` (e.g., `# Part 1`, `## Sequence A`). Used for navigation and organization.
*   **Synopses:** Start with an equals sign (`=`) followed by text. Usually placed after a Scene Heading or Section.
