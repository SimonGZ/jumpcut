# How Dialogue Split Decisions Work

This document explains how JumpCut currently decides dialogue splits during
pagination to match Final Draft behavior.

## 1. The paginator does the physical-space preflight

The first gate is in `choose_split_lines` in
[paginator.rs](/src/pagination/paginator.rs). The paginator now has an explicit
physical-space preflight before it even asks the split planner for candidates:

```rust
if !has_room_for_minimum_top_fragment(available_lines, effective_spacing, geometry) {
    return None;
}

let max_top_lines =
    max_top_content_lines(available_lines, effective_spacing, geometry);

// ...
plan_dialogue_split(
    dialogue,
    geometry,
    max_top_lines,
    geometry.orphan_limit,
    geometry.widow_limit,
)?
```

This is where the available page budget becomes a hard `max_top_lines`
constraint.

The important distinction is:

- the paginator owns **physical page budget**
- the planner owns **semantic split minima**

`max_top_lines` is computed from the raw page budget. `(MORE)` is **not**
counted against that fit check.

## 2. The paginator charges only the raw top fragment height

Immediately after a split plan is chosen, the paginator calculates the height to be consumed on the current page:

```rust
let top_lines = plan.top_line_count as f32 * geometry.line_height;
```

Unlike earlier experiments, JumpCut does not charge `(MORE)` into layout
height. It is outside the formal page budget.

## 3. Candidate boundaries come from text, not arbitrary wrapped lines

JumpCut no longer allows splits at arbitrary wrapped-line breaks. Candidates are generated from only two sources:

1.  **Part Boundaries**: Between any two parts (e.g., Character to Dialogue, or Parenthetical to Dialogue).
2.  **Sentence Boundaries**: Only inside Dialogue or Lyric parts that are **at least 3 wrapped lines long**.

Short dialogue parts (1-2 lines) are considered too compact to split and are kept whole.

Part-boundary candidates are legal, but they do not currently receive the
sentence-boundary bonus. That bonus is reserved for explicit mid-part sentence
boundaries discovered from text.

## 4. The planner owns semantic minima

Once a boundary is identified, the planner rewraps both halves independently
and checks semantic minima in content-line units:

```rust
if matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric | DialoguePartKind::Parenthetical) {
    top_dialogue_lines += top_lines.len();
    bottom_dialogue_lines += bottom_lines.len();
}
```

When checking orphan and widow limits, JumpCut counts wrapped **Dialogue**,
**Lyric**, and **Parenthetical** lines. Character names are structural overhead
and do not count toward those minima.

This is now the only place dialogue continuation minima are enforced. The old
redundant post-selection widow check in the paginator was removed during the
cleanup pass.

## 5. Scoring Priority

When multiple legal split candidates exist, the "best" split is chosen using a strict priority:

1.  **Sentence Boundaries**: Candidates that split at the end of a sentence are heavily preferred.
2.  **Substantial Bottom Continuation**: Among legal candidates, prefer ones
    whose continuation block is substantial enough to avoid looking stranded.
    This is currently implemented as a threshold check, not a continuous
    "bigger bottom always wins" rule.
    As of the `gumshoe` / `two-short-sentences-plus-one-liner` comparison,
    this "substantial bottom" signal is based on spoken continuation lines
    only (`Dialogue` / `Lyric`), not parenthetical lines. Treat that as a
    tentative Final Draft parity rule, not a proven universal screenplay rule:
    it currently explains why FD can still strand a parenthetical when the
    next-page spoken continuation is substantial, while rejecting the
    `gumshoe` split where the continuation was only a one-line spoken fragment
    before another parenthetical.
3.  **Page Fullness**: Once candidates are equal on that substantial-bottom
    check, prefer the one that leaves more lines on the top page.
4.  **Balance**: Prefer splits that result in roughly equal-sized fragments.
5.  **Content Tiebreaker**: As a final tie-break, the split that keeps more raw
    text characters on the top page wins.

This "substantial bottom continuation" preference is the balancing change that
resolved the page-54 / Big Fish continued-dialogue disagreement without
globally charging `(MORE)` into page height or scoring.

## 6. Practical debugging summary

Dialogue split debugging usually involves these checks:

1.  **Did the paginator have enough physical room to even try a split?**
    (Spacing + minimum top fragment.)
2.  **Is the part long enough?** (Check the 3-line minimum rule for mid-part sentence splits.)
3.  **Is there a sentence boundary?** (Check `sentence_boundary.rs` logic.)
4.  **Does it meet the semantic orphan/widow limit?** (Check if Dial/Lyric/Paren lines meet the configured minima.)
5.  **Did scoring favor a fuller continuation?** (Check the substantial-bottom preference before page fullness.)
