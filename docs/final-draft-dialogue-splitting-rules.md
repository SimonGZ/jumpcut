# Final Draft Dialogue Splitting Rules

This document outlines the specific rules JumpCut uses to match Final Draft's dialogue pagination behavior, as established through probe testing.

## Core Rules

### 1. The (MORE) Overflow
The `(MORE)` marker is budget-neutral. It does not count against the maximum number of lines allowed on a page. This allows a dialogue fragment to "fit" on a page even if adding the `(MORE)` line would technically exceed the bottom margin.

### 2. Content Counting (Orphans & Widows)
When checking if a split is legal (i.e., satisfies the 2-line orphan/widow limit), both **Dialogue** and **Parenthetical** lines count as valid content. 
- **Character Name**: Does not count toward the orphan limit.
- **Example**: A split that leaves a Character name, one line of dialogue, and a parenthetical on the top page is valid (2 content lines).

### 3. The 3-Line Split Minimum
A single part (a continuous block of dialogue text or a parenthetical) will only be split internally if it wraps to **at least 3 lines**. 
- Parts that wrap to only 1 or 2 lines are considered too compact to split and will be kept whole on either the top or bottom page.

### 4. Sentence-Only Internal Splits
Dialogue text is never split at arbitrary wrapped-line breaks. Splits inside a text part occur **only at sentence boundaries**. If a sentence is too long to fit the remaining page space, Final Draft will push the entire block (or the entire next sentence) to the next page.

### 5. Part-Boundary Splits
Splits are always allowed between parts (e.g., between a Character name and Dialogue, or between a Parenthetical and Dialogue), provided the orphan/widow limits are met. However, these boundaries do not receive the "sentence boundary" scoring bonus unless the preceding text actually ends a sentence.

---

## Scoring Priority

When multiple legal split candidates exist, the "best" split is chosen using the following priority:

1.  **Sentence Boundaries**: Prefer splitting at the end of a sentence.
2.  **Page Fullness**: Prefer the split that leaves more lines on the top page (filling the page).
3.  **Balance**: Prefer splits that result in roughly equal-sized fragments.
4.  **Content Tiebreaker**: If all else is equal, prefer the split that keeps more characters/bytes on the top page.
