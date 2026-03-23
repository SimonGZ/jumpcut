mod comparison;
mod fixtures;
mod ir;
mod measurement;
mod normalized;
mod semantic;

pub use comparison::{
    compare_paginated_to_fixture, ComparisonIssue, ComparisonIssueKind, ComparisonReport,
};
pub use fixtures::{
    Fragment, LineRange, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixturePage, PageBreakFixtureSourceRefs, PaginationScope,
};
pub use ir::{
    BlockPlacement, ContinuationMarker, Page, PageBlock, PageItem, PageKind, PageMetadata,
    PaginatedScreenplay, PaginationConfig,
};
pub use measurement::{
    measure_dialogue_part_lines, measure_dialogue_unit_lines, measure_dual_dialogue_unit_lines,
    measure_flow_unit_lines, measure_lyric_unit_lines, measure_text_lines, MeasurementConfig,
};
pub use normalized::normalize_screenplay;
pub use semantic::{
    build_semantic_screenplay, Cohesion, DialoguePart, DialoguePartKind, DialogueUnit,
    DualDialogueSide, DualDialogueUnit, FlowKind, FlowUnit, LyricUnit, PageStartUnit,
    SemanticScreenplay, SemanticUnit,
};
