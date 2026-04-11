# Final Draft Dialogue Splitting Rules

This document outlines the current dialogue-splitting rules JumpCut uses to
match Final Draft behavior, based on the active fd-probe set and the corpus
checks that still serve as parity guards.

## Core Rules

### 1. The (MORE) Overflow
JumpCut currently treats `(MORE)` as budget-neutral for fit and charged page
height. A split is allowed to fit on the page based on the raw top fragment,
and the paginator charges only the raw top fragment height.

Important nuance:

- The current probes suggest the remaining Final Draft behavior is better
  explained by split ranking than by a global "always charge `(MORE)` as a
  line" rule.

### 2. Content Counting (Orphans & Widows)
When checking if a split is legal, both **Dialogue** and **Parenthetical**
lines count as valid content.

- **Character Name**: Does not count toward the orphan limit.
- **Example**: A split that leaves a Character name, one line of dialogue, and a parenthetical on the top page is valid (2 content lines).

### 3. The 3-Line Split Minimum
A single part (a continuous block of dialogue text or a parenthetical) will only be split internally if it wraps to **at least 3 lines**. 
- Parts that wrap to only 1 or 2 lines are considered too compact to split and will be kept whole on either the top or bottom page.

### 4. Sentence-Only Internal Splits
Dialogue text is never split at arbitrary wrapped-line breaks. Splits inside a text part occur **only at sentence boundaries**. If a sentence is too long to fit the remaining page space, Final Draft will push the entire block (or the entire next sentence) to the next page.

### 5. Part-Boundary Splits
Splits are always allowed between parts (e.g., between a Character name and
Dialogue, or between a Parenthetical and Dialogue), provided the orphan/widow
limits are met.

In the current implementation, part-boundary splits are legal candidates but do
not receive the sentence-boundary bonus. Sentence-boundary preference is only
applied to mid-part boundaries discovered inside long dialogue/lyric text.

---

## Scoring Priority

When multiple legal split candidates exist, the "best" split is chosen using the following priority:

1.  **Sentence Boundaries**: Prefer splitting at the end of a sentence.
2.  **Substantial Bottom Continuation**: Prefer candidates whose continuation
    block on the next page is substantial enough to avoid looking stranded. In
    the current implementation, this is a threshold rule, not a continuous
    "more bottom is always better" preference.
3.  **Page Fullness**: Once candidates are equal on the substantial-bottom
    check, prefer the split that leaves more material on the top page.
4.  **Balance**: Prefer splits that result in roughly equal-sized fragments.
5.  **Content Tiebreaker**: If all else is equal, prefer the split that keeps
    more characters/bytes on the top page.
