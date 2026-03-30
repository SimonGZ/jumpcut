# Line-Height Repair Plan

This document tracks the work needed to fix JumpCut's current line-height
model for multicam / Mostly Genius style scripts.

## Problem

JumpCut currently collapses line spacing into a single global pagination value:

- [layout_profile.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/layout_profile.rs)
  copies `dialogue.line_spacing` into `geometry.line_height`
- [composer.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/composer.rs)
  multiplies **all** measured content by that one scalar
- [paginator.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/paginator.rs)
  and diagnostics code then assume that one scalar applies everywhere

That means "double-spaced dialogue" currently behaves like "double-spaced
everything."

## Goal

Support element-specific line spacing so that:

- dialogue can be double-spaced without doubling action / scene heading height
- the pseudo-PDF matches the real pagination model instead of exposing a fake
  global line-height world
- Mostly Genius can be evaluated against a believable multicam layout model

## Guiding Rule

Change one layer at a time and run the full suite after every step:

- `~/.cargo/bin/cargo test`

Do not proceed to the next phase until the current one is green and the
intermediate behavior is understood.

## Phase 1: Lock In the Intended Behavior

Purpose:

- Write tests that describe the correct behavior before changing the model.

Tasks:

- Add a composer-level test proving that dialogue line spacing can be `2.0`
  while action remains `1.0`.
- Add a pagination-profile test that multicam / `dsd` affect dialogue spacing
  only, not global layout height.
- Add a pseudo-PDF test that double-height rendering is applied to dialogue
  lines only.
- Add an FDX-settings-derived geometry test if needed, so extracted settings
  can eventually drive the same distinction.

Exit condition:

- The new tests fail for the right reason under the current global model.

## Phase 2: Replace Global `geometry.line_height`

Purpose:

- Move from one global line-height scalar to element-specific line-spacing
  values.

Tasks:

- Extend [margin.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/margin.rs)
  so `LayoutGeometry` can carry per-element line spacing, not just a single
  `line_height`.
- Start with the elements that matter to pagination drift first:
  - dialogue
  - parenthetical
  - character
  - action
  - scene heading
- Keep defaults equivalent to today's screenplay behavior (`1.0` everywhere)
  so non-multicam scripts stay stable.
- Update [layout_profile.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/layout_profile.rs)
  to lower `fmt` metadata into those per-element spacing values.

Exit condition:

- Geometry can represent "dialogue is 2.0, action is 1.0" without hacks.

## Phase 3: Teach the Composer to Measure Physical Height Per Element

Purpose:

- Make composition produce real visual heights instead of multiplying by one
  global scalar.

Tasks:

- Refactor [composer.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/composer.rs)
  so each wrapped line contributes the spacing for its own element type.
- Dialogue blocks must sum their internal parts using the correct per-part
  spacing:
  - character line height
  - parenthetical line height
  - dialogue line height
  - lyric line height
- Dual dialogue must compute each side using the same per-part logic and still
  take the taller side.

Exit condition:

- `LayoutBlock.content_lines` becomes a true physical-height measure even when
  different element types on the page use different line spacing.

## Phase 4: Remove Global-Line-Height Assumptions from Pagination

Purpose:

- Stop converting between "wrapped line counts" and physical height using a
  single scalar.

Tasks:

- Audit [paginator.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/paginator.rs)
  for every division or multiplication by `geometry.line_height`.
- Split the concepts that are currently conflated:
  - wrapped content-line counts used for orphan/widow rules
  - physical visual height used for page budget
- Keep dialogue orphan/widow rules in content-line units, but compute top/bottom
  fragment physical height from actual per-part spacing.
- Do the same for flow splits if they currently assume a single scalar.

Exit condition:

- Page-budget checks use real visual height; orphan/widow checks still use the
  intended semantic/content units.

## Phase 5: Update Diagnostics and Pseudo-PDF Rendering

Purpose:

- Make debug artifacts reflect the repaired model.

Tasks:

- Remove the current "if global line height is 2.0, double every counted line"
  shortcut in
  [page_break_diagnostics.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/page_break_diagnostics.rs).
- Render double-height only for the element types that actually use it.
- Keep continuation markers readable without pretending they control physical
  layout height.
- Re-check Mostly Genius pseudo-PDF output by eye after this phase.

Exit condition:

- Pseudo-PDF visual rhythm matches the repaired paginator model instead of the
  old global-height approximation.

## Phase 6: Rebaseline Mostly Genius Diagnostics

Purpose:

- Measure what the repaired model actually changes.

Tasks:

- Run:
  - `~/.cargo/bin/cargo test`
  - `~/.cargo/bin/cargo run --bin pagination-diagnostics -- mostly-genius-full-script`
  - `~/.cargo/bin/cargo run --bin pagination-diagnostics -- mostly-genius-linebreak`
- Compare the new Mostly Genius packet against the current one.
- Verify Big Fish / Little Women did not regress before treating Mostly Genius
  deltas as signal.

Exit condition:

- We know whether the repaired line-height model improves Mostly Genius
  specifically, and we know what else moved.

## Known Risk Areas

- [pagination_ir.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/ir.rs)
  currently derives visible line counts using `geometry.line_height`.
- [page_break_diagnostics.rs](/home/ubuntu/.ductor/workspace/jumpcut/src/pagination/page_break_diagnostics.rs)
  uses `geometry.line_height` in several measurement/reporting helpers.
- Existing tests in
  [pagination_composer_test.rs](/home/ubuntu/.ductor/workspace/jumpcut/tests/pagination_composer_test.rs)
  and
  [pagination_paginator_test.rs](/home/ubuntu/.ductor/workspace/jumpcut/tests/pagination_paginator_test.rs)
  assume global line-height math and will need careful revision rather than
  blind updates.

## Recommended Order of Work

1. Phase 1: write failing tests that prove the current model is wrong.
2. Phase 2: extend geometry to represent per-element spacing.
3. Phase 3: refactor composer measurement.
4. Phase 4: refactor paginator / split math.
5. Phase 5: repair diagnostics / pseudo-PDF output.
6. Phase 6: rerun Mostly Genius diagnostics and inspect deltas.
