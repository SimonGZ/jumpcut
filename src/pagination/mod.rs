mod comparison;
pub mod composer;
pub mod dialogue_split;
mod fixtures;
pub mod flow_split;
mod ir;
pub mod layout_profile;
pub mod margin;
mod normalized;
pub mod paginator;
mod semantic;
mod sentence_boundary;
mod split_scoring;
#[cfg(any(feature = "html", feature = "pdf"))]
pub mod visual_lines;
pub mod wrapping;

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
pub use layout_profile::{
    ScreenplayElementStyle, ScreenplayElementStyles, ScreenplayLayoutProfile, StyleProfile,
};
pub use margin::{Alignment, FdxExtractedSettings, FdxParagraphStyle, LayoutGeometry};
pub use normalized::normalize_screenplay;
pub use semantic::{
    build_semantic_screenplay, build_semantic_screenplay_with_options, Cohesion, DialoguePart,
    DialoguePartKind, DialogueUnit, DualDialogueSide, DualDialogueUnit, FlowKind, FlowUnit,
    LyricUnit, PageStartUnit, SemanticOptions, SemanticScreenplay, SemanticUnit,
};
pub use wrapping::InterruptionDashWrap;
