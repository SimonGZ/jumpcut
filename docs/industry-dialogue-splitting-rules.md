# Industry profile: dialogue splitting

This is a contributor reference for the `industry` pagination profile. It explains the compatibility rules used when JumpCut needs to match conventional screenplay pagination or imported FDX behavior.

> **Compatibility note:** The `industry` profile uses an ordered local
> correction structure rather than JumpCut's ordinary global split ranking.
> Sentence correction is a configurable document option and covers Action,
> Dialogue, Lyric, and Parenthetical pagination types.

## The core rules

### How `(MORE)` affects page fit
JumpCut currently treats `(MORE)` as budget-neutral for fit and charged page
height. A split is allowed to fit on the page based on the raw top fragment,
and the paginator charges only the raw top fragment height.

Continuation insertion happens after the semantic boundary correction. Current
reference PDFs show that charging `(MORE)` as an additional fit line would move
otherwise-correct breaks, so compatibility mode keeps it budget-neutral.

### What counts as content
When checking if a split is legal, both **Dialogue** and **Parenthetical**
lines count as valid content.

- **Character Name**: Does not count toward the orphan limit.
- **Example**: A split that leaves a Character name, one line of dialogue, and a parenthetical on the top page is valid (2 content lines).

### Keep short dialogue groups together
The first physical line below the proposed page break is the center of a small
local line window. The industry correction pass inspects nearby line types
instead of ranking every possible split:

- If cache position 5 ends a dialogue group and its Character cue occurs in
  positions 2 through 4, the entire short group moves to the next page.
- If position 3 is Character and positions 4 and 5 are dialogue content, that
  short group also moves together.
- If the group is longer, the proposal may move one physical line upward before
  sentence correction runs.

### Correct breaks to sentence boundaries
With `BreakDialogueAndActionAtSentences="Yes"`, an internal physical boundary
is corrected backward to the latest recognized sentence boundary at or before
that line. If none exists, the block cannot split there. With the option set to
`No`, the physical wrapped-line boundary is retained.

### Respect part boundaries
A physical proposal between two paragraph parts (for example Parenthetical and
Dialogue) already has a semantic boundary, so sentence backtracking is not
applied. The local group checks and orphan/widow minima can still move or reject
that boundary.

---

## This profile uses ordered corrections

Compatibility mode does not select a winner from a globally scored candidate
list. It derives one physical proposal from page geometry, applies the relevant
local element handler, and optionally corrects that result backward to a
sentence boundary. The ordinary `jumpcut` profile retains JumpCut's ranked
split selection.
