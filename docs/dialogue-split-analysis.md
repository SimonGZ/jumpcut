# How Dialogue Split Decisions Work

This document explains how JumpCut decides dialogue splits during pagination to match Final Draft behavior.

## 1. The paginator decides whether dialogue is even allowed to split

The first gate is in `choose_split_lines` in [paginator.rs](/src/pagination/paginator.rs).

```rust
let max_top_lines = ((available_lines - effective_spacing) / geometry.line_height)
    .floor() as usize;

// ...
plan_dialogue_split(
    dialogue,
    geometry,
    max_top_lines,
    geometry.orphan_limit,
    geometry.widow_limit,
)?
```

This is where the available page budget becomes a hard `max_top_lines` constraint. 

**Crucially**: The `max_top_lines` check is performed against the raw line count of the dialogue. The `(MORE)` line is **not** counted against this budget, allowing it to overflow into the bottom margin.

## 2. The paginator charges the raw top height

Immediately after a split plan is chosen, the paginator calculates the height to be consumed on the current page:

```rust
let top_lines = plan.top_line_count as f32 * geometry.line_height;
```

Unlike previous versions of the engine, we no longer charge for the `(MORE)` line in the layout. It lives "outside" the formal page budget.

## 3. Candidate boundaries come from text, not wrapped lines

JumpCut no longer allows splits at arbitrary wrapped-line breaks. Candidates are generated from only two sources:

1.  **Part Boundaries**: Between any two parts (e.g., Character to Dialogue, or Parenthetical to Dialogue).
2.  **Sentence Boundaries**: Only inside Dialogue or Lyric parts that are **at least 3 wrapped lines long**.

Short dialogue parts (1-2 lines) are considered too compact to split and are kept whole.

## 4. Each candidate is built by splitting text and rewrapping

Once a boundary is identified, the planner rewraps both halves independently. 

```rust
if matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric | DialoguePartKind::Parenthetical) {
    top_dialogue_lines += top_lines.len();
    bottom_dialogue_lines += bottom_lines.len();
}
```

**Content Counting**: When checking orphan and widow limits (usually 2 lines), JumpCut counts **both Dialogue and Parenthetical lines**. Character names are considered structural overhead and do not count toward these limits.

## 5. Scoring Priority

When multiple legal split candidates exist, the "best" split is chosen using a strict priority:

1.  **Sentence Boundaries**: Candidates that split at the end of a sentence are heavily preferred.
2.  **Page Fullness**: Among equal boundary types, the one that leaves more lines on the top page (using `top_page_line_count()` which *does* include the MORE line cost for scoring purposes) is preferred.
3.  **Balance**: Prefer splits that result in roughly equal-sized fragments.
4.  **Content Tiebreaker**: As a final tie-break, the split that keeps more raw text characters on the top page wins.

## 6. Practical debugging summary

Dialogue split debugging usually involves these checks:

1.  **Is the part long enough?** (Check the 3-line minimum rule).
2.  **Is there a sentence boundary?** (Check `sentence_boundary.rs` logic).
3.  **Does it meet the orphan/widow limit?** (Check if Dial+Paren lines ≥ 2).
4.  **Did (MORE) cause a rejection?** (It shouldn't; it's now budget-neutral).
