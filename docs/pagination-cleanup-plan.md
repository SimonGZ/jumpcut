# Pagination Cleanup Plan

This document tracks the medium-priority pagination cleanup work identified in
code review after the recent dialogue-splitting parity changes.

Rule for every step in this plan:

- Make one isolated change.
- Run the full suite with `cargo test`.
- Do not start the next step until the suite is green again.

## Verification Log

- [x] Baseline recorded before cleanup work begins.
  - Command: `cargo test`
  - Result: Passed before Step 2/3 work; used as the green baseline for this cleanup pass.

- [x] Step 1 completed and verified.
  - Command: `cargo test`
  - Result: Passed after making split-score priority ordering explicit.

- [x] Step 2 completed and verified.
  - Command: `cargo test`
  - Result: Passed after removing the unused duplicate split path and rewriting dialogue split tests around the production planner.

- [x] Step 3 completed and verified.
  - Command: `cargo test`
  - Result: Passed after clarifying the physical-space preflight vs planner content-minimum checks and removing redundant post-split widow checks.

## Step 1: Make Dialogue Split Scoring Explicit

Problem:

- `SplitScore` ordering is currently encoded implicitly through struct field
  order plus `derive(Ord)`, which makes the scoring policy fragile and hard to
  read.

Goals:

- Replace the implicit `derive(Ord)` priority encoding with an explicit scoring
  comparator or clearly named ordered tuple builder.
- Preserve the current priority order exactly:
  1. sentence boundary preference
  2. substantial bottom continuation preference
  3. fuller top fragment preference
  4. balance preference
  5. top-content tiebreaker
- Leave a short code comment describing why each priority exists.

Checklist:

- [x] Refactor `SplitScore` so scoring order is explicit in code.
- [x] Keep current behavior unchanged under the active fd-probes and corpus
      suite.
- [x] Add or adjust unit coverage if the refactor changes how scoring is
      expressed.
- [x] Run `cargo test` and record the result in the verification log.

## Step 2: Remove or Fold the Duplicate Dialogue Split Path

Problem:

- `choose_dialogue_split()` and `plan_dialogue_split_parts()` express dialogue
  split policy through different entry points, but only the planner path is
  used by production pagination.

Goals:

- Decide whether `choose_dialogue_split()` should:
  - be deleted and its tests rewritten around the production planner, or
  - be reimplemented as a thin wrapper over the production planner so there is
    only one policy path.
- Ensure tests exercise the same split ranking logic the paginator uses in real
  pagination.

Checklist:

- [x] Trace every remaining use of `choose_dialogue_split()`.
- [x] Collapse the duplicate policy path or make it a thin wrapper over the
      production planner.
- [x] Update dialogue split tests so they validate the production path.
- [x] Run `cargo test` and record the result in the verification log.

## Step 3: Clarify the Two-Layer Widow/Orphan Checks

Problem:

- The planner filters split candidates using dialogue-content counts, while the
  paginator also applies a separate physical-line check afterward. The current
  code makes it hard to tell which rule is semantic and which is layout safety.

Goals:

- Make the two checks explicit and named for their roles.
- Either:
  - consolidate them into one well-defined rule path, or
  - keep both but document the distinction directly in code.
- Make it obvious which units each check uses:
  - dialogue/parenthetical content lines
  - total physical wrapped lines

Checklist:

- [x] Decide whether the second check is truly redundant or intentionally
      protecting a different invariant.
- [x] Refactor naming/comments so the distinction is obvious in code.
- [x] Add focused tests if needed to lock in the intended two-layer behavior.
- [x] Run `cargo test` and record the result in the verification log.

## Out of Scope For This Cleanup Pass

- FD probe harness rendering semantics for `push-whole` vs split fragments.
- General cleanup of overlapping truth sources between canonical fixtures,
  window fixtures, and fd-probes.

Those are worth revisiting, but they are lower priority than the three items
above.
