# How Dialogue Split Decisions Work

This document explains how JumpCut currently decides dialogue splits during
pagination. It describes the code path that runs today.

## 1. The paginator decides whether dialogue is even allowed to split

The first gate is in `choose_split_lines` in
[paginator.rs](/ductor/workspace/jumpcut/src/pagination/paginator.rs).

```rust
let max_top_lines = ((available_lines - effective_spacing) / geometry.line_height)
    .floor() as usize;

let plan = match (&block.fragment, block.dialogue_split.as_ref()) {
    (Fragment::ContinuedFromPrev, Some(previous_plan)) => {
        let current_parts = dialogue
            .parts
            .iter()
            .zip(previous_plan.parts.iter())
            .map(|(part, split)| DialogueTextPart {
                kind: part.kind.clone(),
                text: split.bottom_text.clone(),
            })
            .collect::<Vec<_>>();

        plan_dialogue_split_parts(
            dialogue,
            &current_parts,
            geometry,
            max_top_lines,
            geometry.orphan_limit,
            geometry.widow_limit,
        )?
    }
    _ => plan_dialogue_split(
        dialogue,
        geometry,
        max_top_lines,
        geometry.orphan_limit,
        geometry.widow_limit,
    )?,
};
```

This is where the available page budget becomes a hard `max_top_lines`
constraint for the dialogue planner.

Two behaviors matter:

- Fresh dialogue blocks call `plan_dialogue_split(...)`.
- Dialogue already continued from a previous page calls
  `plan_dialogue_split_parts(...)` on the old plan’s `bottom_text`, so we only
  split the remainder.

If that planner returns `None`, the paginator does not split the dialogue block
here at all.

## 2. The paginator charges the planner’s counted top height

Immediately after the plan is chosen, the paginator turns it into page height:

```rust
let top_lines = plan.top_page_line_count() as f32 * geometry.line_height;
let bottom_dialogue_lines = plan.bottom_line_count as f32 * geometry.line_height;
let bottom_lines = bottom_dialogue_lines;
```

This is important because `top_page_line_count()` is not the same as
`top_line_count`.

The current asymmetry is:

- top fragment height includes the counted bottom `(MORE)` cost
- bottom fragment height does not include the repeated cue at the top of the
  next page

That asymmetry is central to the current dialogue behavior.

## 3. The planner carries both text fragments and sentence-boundary metadata

The main split structure is in
[dialogue_split.rs](/ductor/workspace/jumpcut/src/pagination/dialogue_split.rs).

```rust
pub struct DialogueSplitPlan {
    pub top_line_count: usize,
    pub bottom_line_count: usize,
    pub ends_sentence: bool,
    pub parts: Vec<DialoguePartSplitLines>,
}

impl DialogueSplitPlan {
    pub fn top_page_line_count(&self) -> usize {
        self.top_line_count + more_line_cost()
    }
}
```

The planner keeps three different kinds of information together:

- visible wrapped line counts
- whether the chosen boundary ends a sentence
- the actual top and bottom text fragments for each dialogue part

That last piece is what lets JumpCut render and inspect the split accurately
after the choice has been made.

## 4. Candidate boundaries come from the original text, not only wrapped lines

Candidate generation happens here:

```rust
for (part_index, part) in parts.iter().enumerate() {
    boundaries
        .entry((part_index, part.text.len()))
        .and_modify(|ends_sentence| *ends_sentence |= text_ends_sentence(&part.text))
        .or_insert_with(|| text_ends_sentence(&part.text));

    if !matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric) {
        continue;
    }

    let config =
        WrapConfig::from_geometry(geometry, element_type_for_part_kind(part.kind.clone()));
    for line in wrap_text_for_element_with_offsets(&part.text, &config)
        .into_iter()
        .take_while(|line| line.end_offset < part.text.len())
    {
        boundaries
            .entry((part_index, line.end_offset))
            .or_insert(false);
    }

    for offset in sentence_boundary_offsets(&part.text) {
        boundaries
            .entry((part_index, offset))
            .and_modify(|ends_sentence| *ends_sentence = true)
            .or_insert(true);
    }
}
```

The planner therefore considers three kinds of boundaries:

- part ends
- wrapped-line ends
- sentence-boundary offsets inside the original text

That is why the dialogue splitter can now break inside what used to be a
single wrapped line: the choice is made at the text level, then rewrapped.

## 5. Each candidate is built by splitting text and rewrapping both halves

Once a boundary is chosen, the planner builds a candidate like this:

```rust
for (part_index, part) in parts.iter().enumerate() {
    let (top_text, bottom_text) = split_part_text(&part.text, part_index, boundary);
    let config =
        WrapConfig::from_geometry(geometry, element_type_for_part_kind(part.kind.clone()));
    let top_lines = wrap_fragment_lines(top_text, &config);
    let bottom_lines = wrap_fragment_lines(bottom_text, &config);

    top_line_count += top_lines.len();
    bottom_line_count += bottom_lines.len();

    if matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric) {
        top_dialogue_lines += top_lines.len();
        bottom_dialogue_lines += bottom_lines.len();
    }

    split_parts.push(DialoguePartSplitLines {
        top_text: top_text.to_string(),
        bottom_text: bottom_text.to_string(),
        top_lines,
        bottom_lines,
    });
}
```

That means the planner does not slice the old wrapped layout.

Instead it:

- splits the original text at the candidate offset
- wraps the top fragment independently
- wraps the bottom fragment independently
- counts dialogue lines from those rewrapped fragments

So legality and scoring are based on the post-split layout, not on the
unsplit one.

## 6. Scoring prefers sentence endings first, then a fuller top fragment

The main scoring pass is here:

```rust
let winner = choose_best_scored_split(0..candidates.len(), |candidate_index| {
    let candidate = &candidates[candidate_index];
    if candidate.plan.top_page_line_count() > max_top_lines {
        return None;
    }

    if candidate.top_dialogue_lines < min_top_dialogue_lines
        || candidate.bottom_dialogue_lines < min_bottom_dialogue_lines
    {
        return None;
    }

    Some(SplitScore {
        ends_sentence: policy.prefer_sentence_boundaries && candidate.ends_sentence,
        fuller_top_fragment: policy
            .prefer_fuller_top_fragment
            .then_some(candidate.plan.top_page_line_count())
            .unwrap_or(0),
        balance_score: balance_score(
            candidate.top_dialogue_lines,
            candidate.bottom_dialogue_lines,
        ),
    })
});
```

So the effective order is:

1. reject anything that does not fit the counted top budget
2. reject anything that violates orphan/widow minima
3. prefer `ends_sentence = true`
4. prefer a fuller top fragment
5. use balance as the final tie-break

One subtle but important point:

`fuller_top_fragment` is measured with `candidate.plan.top_page_line_count()`,
not raw visible dialogue lines. So the counted `(MORE)` cost affects both
legality and the “fill the page” preference.

## 7. Sentence boundaries are detected with a lightweight punctuation rule

The sentence helper lives in
[sentence_boundary.rs](/ductor/workspace/jumpcut/src/pagination/sentence_boundary.rs).

```rust
while index < chars.len() {
    if !matches!(chars[index].1, '.' | '!' | '?') {
        index += 1;
        continue;
    }

    let mut next = index + 1;
    while next < chars.len() && is_sentence_closer(chars[next].1) {
        next += 1;
    }

    if next == chars.len() {
        offsets.push(text.len());
        index += 1;
        continue;
    }

    if chars[next].1.is_whitespace() {
        while next < chars.len() && chars[next].1.is_whitespace() {
            next += 1;
        }

        offsets.push(if next < chars.len() {
            chars[next].0
        } else {
            text.len()
        });
    }
```

And sentence-ending classification is:

```rust
pub(crate) fn text_ends_sentence(text: &str) -> bool {
    text.trim_end_matches(char::is_whitespace)
        .trim_end_matches(is_sentence_closer)
        .chars()
        .last()
        .is_some_and(|ch| matches!(ch, '.' | '!' | '?'))
}
```

This is deliberately simple:

- punctuation candidates are `.`, `!`, and `?`
- trailing closing quotes/brackets are ignored
- a boundary is recognized only when punctuation is followed by whitespace or
  end-of-text

So this is a pragmatic detector, not full grammar analysis.

## 8. The `(MORE)` rule is one small helper with large downstream consequences

The current policy hook is tiny:

```rust
fn more_line_cost() -> usize {
    1
}
```

But because `top_page_line_count()` uses it, that one line affects:

- candidate fit rejection
- fuller-top scoring
- final top-fragment height charged by the paginator

So a change here does not merely tweak diagnostics. It changes the planner’s
actual choice set.

## 9. Practical debugging summary

Today, dialogue split debugging usually comes down to these questions:

- What text-level candidate boundaries existed?
- Which of them were sentence boundaries?
- Which of them still fit after the counted `(MORE)` line was charged?
- Among the remaining candidates, which one won on the score tuple?

That is the current operating chain:

1. paginator computes remaining page budget
2. planner generates text-level boundaries
3. planner rewraps both halves for each candidate
4. planner filters by counted height and orphan/widow minima
5. planner scores by sentence ending, then top fullness, then balance
6. paginator charges the winning top fragment using `top_page_line_count()`
