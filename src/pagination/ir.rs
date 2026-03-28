use crate::pagination::fixtures::{
    Fragment, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixtureSourceRefs, PaginationScope,
};
use crate::pagination::ScreenplayLayoutProfile;
use crate::pagination::semantic::{
    DialoguePartKind, DialogueUnit, FlowKind, FlowUnit, SemanticScreenplay,
    SemanticUnit,
};
use crate::pagination::LayoutGeometry;
use crate::Screenplay;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContinuationMarker {
    Continued,
    More,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockPlacement {
    Flow,
    DualDialogue { group_id: String, side: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PageKind {
    Title,
    Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageMetadata {
    pub index: usize,
    pub number: u32,
    pub kind: PageKind,
    pub body_page_number: Option<u32>,
    pub title_page_number: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageItem {
    pub element_id: String,
    pub kind: String,
    pub fragment: Fragment,
    pub line_range: Option<(u32, u32)>,
    pub block_id: Option<String>,
    pub dual_dialogue_group: Option<String>,
    pub dual_dialogue_side: Option<u8>,
    pub continuation_markers: Vec<ContinuationMarker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageBlock {
    pub id: String,
    pub source_block_id: Option<String>,
    pub item_ids: Vec<String>,
    pub placement: BlockPlacement,
    pub fragment: Fragment,
    pub continuation_markers: Vec<ContinuationMarker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    pub metadata: PageMetadata,
    pub items: Vec<PageItem>,
    pub blocks: Vec<PageBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginatedScreenplay {
    pub screenplay: String,
    pub style_profile: String,
    pub source: PageBreakFixtureSourceRefs,
    pub scope: PaginationScope,
    pub pages: Vec<Page>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaginationConfig {
    pub lines_per_page: f32,
    pub geometry: LayoutGeometry,
}

impl PaginationConfig {
    pub fn screenplay(lines_per_page: f32) -> Self {
        Self {
            lines_per_page,
            geometry: LayoutGeometry::default(),
        }
    }

    pub fn from_screenplay(screenplay: &Screenplay, lines_per_page: f32) -> Self {
        let profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
        Self {
            lines_per_page,
            geometry: profile.to_pagination_geometry(),
        }
    }
}

impl PaginatedScreenplay {
    pub fn paginate(
        semantic: SemanticScreenplay,
        config: PaginationConfig,
        style_profile: impl Into<String>,
        scope: PaginationScope,
    ) -> Self {
        let mut next_page_number = semantic
            .starting_page_number
            .unwrap_or_else(|| first_page_number(&scope));
        let style_profile = style_profile.into();
        let geometry = &config.geometry;

        let blocks = crate::pagination::composer::compose(&semantic.units, geometry);
        let paged_blocks = crate::pagination::paginator::paginate(&blocks, config.lines_per_page, geometry);

        let mut pages: Vec<Page> = Vec::new();

        for paged_page in paged_blocks.into_iter() {
            let mut current_items = Vec::new();
            for block in paged_page.blocks {
                let mut items = page_items_from_semantic_unit(block.unit);
                
                if !items.is_empty() {
                    match block.fragment {
                        crate::pagination::fixtures::Fragment::ContinuedToNext => {
                            if let Some(last) = items.last_mut() {
                                last.fragment = merge_fragment(&last.fragment, &crate::pagination::fixtures::Fragment::ContinuedToNext);
                                last.continuation_markers = continuation_markers_for_fragment(&last.fragment);
                            }
                        },
                        crate::pagination::fixtures::Fragment::ContinuedFromPrev => {
                            if let Some(first) = items.first_mut() {
                                first.fragment = merge_fragment(&first.fragment, &crate::pagination::fixtures::Fragment::ContinuedFromPrev);
                                first.continuation_markers = continuation_markers_for_fragment(&first.fragment);
                            }
                        },
                        _ => {}
                    }
                }

                current_items.extend(items);
            }

            // `build_page` expects us to skip empty pages. `SemanticUnit::PageStart` yields 0 items natively.
            if !current_items.is_empty() {
                pages.push(build_page(
                    pages.len(),
                    next_page_number,
                    &scope,
                    current_items,
                ));
                next_page_number += 1;
            }
        }

        Self {
            screenplay: semantic.screenplay,
            style_profile,
            source: PageBreakFixtureSourceRefs::default(),
            scope,
            pages,
        }
    }

    pub fn from_normalized(
        normalized: NormalizedScreenplay,
        style_profile: impl Into<String>,
        scope: PaginationScope,
    ) -> Self {
        let mut pages: Vec<Page> = Vec::new();
        let mut next_page_number = normalized
            .starting_page_number
            .unwrap_or_else(|| first_page_number(&scope));
        let mut current_items: Vec<PageItem> = Vec::new();

        for element in normalized.elements {
            if element.starts_new_page && !current_items.is_empty() {
                let rolled_block_items =
                    take_trailing_block_items_for_page_start(&mut current_items, &element);
                if !current_items.is_empty() {
                    pages.push(build_page(
                        pages.len(),
                        next_page_number,
                        &scope,
                        std::mem::take(&mut current_items),
                    ));
                    next_page_number += 1;
                }
                current_items = rolled_block_items;
            }

            current_items.push(page_item_from_normalized(element));
        }

        if !current_items.is_empty() {
            pages.push(build_page(
                pages.len(),
                next_page_number,
                &scope,
                current_items,
            ));
        }

        Self {
            screenplay: normalized.screenplay,
            style_profile: style_profile.into(),
            source: PageBreakFixtureSourceRefs::default(),
            scope,
            pages,
        }
    }

    pub fn from_fixture(fixture: PageBreakFixture) -> Self {
        let pages = fixture
            .pages
            .into_iter()
            .enumerate()
            .map(|(index, page)| {
                let items = page
                    .items
                    .into_iter()
                    .map(|item| PageItem {
                        line_range: item.line_range.map(|range| (range.0, range.1)),
                        continuation_markers: continuation_markers_for_fragment(&item.fragment),
                        element_id: item.element_id,
                        kind: item.kind,
                        fragment: item.fragment,
                        block_id: item.block_id,
                        dual_dialogue_group: item.dual_dialogue_group,
                        dual_dialogue_side: item.dual_dialogue_side,
                    })
                    .collect();

                build_page(index, page.number, &fixture.scope, items)
            })
            .collect();

        Self {
            screenplay: fixture.screenplay,
            style_profile: fixture.style_profile,
            source: fixture.source,
            scope: fixture.scope,
            pages,
        }
    }
}

fn build_page(
    index: usize,
    page_number: u32,
    scope: &PaginationScope,
    items: Vec<PageItem>,
) -> Page {
    let blocks = build_blocks(&items);

    Page {
        metadata: PageMetadata {
            index,
            number: page_number,
            kind: page_kind(page_number, scope),
            body_page_number: body_page_number(page_number, scope),
            title_page_number: title_page_number(page_number, scope),
        },
        items,
        blocks,
    }
}



fn page_items_from_semantic_unit(unit: &SemanticUnit) -> Vec<PageItem> {
    match unit {
        SemanticUnit::PageStart(_) => Vec::new(),
        SemanticUnit::Flow(unit) => vec![flow_page_item(unit, false, false)],
        SemanticUnit::Lyric(unit) => vec![PageItem {
            element_id: unit.element_id.clone(),
            kind: "Lyric".into(),
            fragment: Fragment::Whole,
            line_range: None,
            block_id: None,
            dual_dialogue_group: None,
            dual_dialogue_side: None,
            continuation_markers: Vec::new(),
        }],
        SemanticUnit::Dialogue(unit) => {
            dialogue_items_with_fragment_markers(unit, None, None, false, false)
        }
        SemanticUnit::DualDialogue(unit) => unit
            .sides
            .iter()
            .flat_map(|side| {
                dialogue_items_with_fragment_markers(
                    &side.dialogue,
                    Some(unit.group_id.as_str()),
                    Some(side.side),
                    false,
                    false,
                )
            })
            .collect(),
    }
}

fn flow_page_item(
    unit: &FlowUnit,
    continued_from_prev: bool,
    continued_to_next: bool,
) -> PageItem {
    let mut fragment = Fragment::Whole;
    if continued_from_prev {
        fragment = merge_fragment(&fragment, &Fragment::ContinuedFromPrev);
    }
    if continued_to_next {
        fragment = merge_fragment(&fragment, &Fragment::ContinuedToNext);
    }

    PageItem {
        element_id: unit.element_id.clone(),
        kind: flow_kind_name(&unit.kind).to_string(),
        fragment: fragment.clone(),
        line_range: unit.line_range,
        block_id: None,
        dual_dialogue_group: None,
        dual_dialogue_side: None,
        continuation_markers: continuation_markers_for_fragment(&fragment),
    }
}

fn dialogue_items_with_fragment_markers(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    continued_from_prev: bool,
    continued_to_next: bool,
) -> Vec<PageItem> {
    let mut items: Vec<PageItem> = unit
        .parts
        .iter()
        .map(|part| PageItem {
            element_id: part.element_id.clone(),
            kind: dialogue_part_kind_name(&part.kind).to_string(),
            fragment: Fragment::Whole,
            line_range: None,
            block_id: Some(unit.block_id.clone()),
            dual_dialogue_group: dual_group.map(str::to_string),
            dual_dialogue_side: dual_side,
            continuation_markers: Vec::new(),
        })
        .collect();

    if let Some(first) = items.first_mut() {
        if continued_from_prev {
            first.fragment = merge_fragment(&first.fragment, &Fragment::ContinuedFromPrev);
            first.continuation_markers = continuation_markers_for_fragment(&first.fragment);
        }
    }

    if let Some(last) = items.last_mut() {
        if continued_to_next {
            last.fragment = merge_fragment(&last.fragment, &Fragment::ContinuedToNext);
            last.continuation_markers = continuation_markers_for_fragment(&last.fragment);
        }
    }

    items
}

fn page_item_from_normalized(element: NormalizedElement) -> PageItem {
    let fragment = element.fragment.unwrap_or(Fragment::Whole);

    PageItem {
        element_id: element.element_id,
        kind: element.kind,
        continuation_markers: continuation_markers_for_fragment(&fragment),
        fragment,
        line_range: None,
        block_id: element.block_id,
        dual_dialogue_group: element.dual_dialogue_group,
        dual_dialogue_side: element.dual_dialogue_side,
    }
}

fn take_trailing_block_items_for_page_start(
    current_items: &mut Vec<PageItem>,
    element: &NormalizedElement,
) -> Vec<PageItem> {
    let Some(block_id) = element.block_id.as_ref() else {
        return Vec::new();
    };

    let split_index = current_items
        .iter()
        .rposition(|item| item.block_id.as_ref() != Some(block_id))
        .map(|index| index + 1)
        .unwrap_or(0);

    let trailing_items = &current_items[split_index..];
    if trailing_items.is_empty()
        || trailing_items
            .iter()
            .any(|item| item.block_id.as_ref() != Some(block_id))
    {
        return Vec::new();
    }

    current_items.split_off(split_index)
}

fn build_blocks(items: &[PageItem]) -> Vec<PageBlock> {
    let mut blocks: Vec<PageBlock> = Vec::new();

    for item in items {
        let block_id = item
            .block_id
            .clone()
            .unwrap_or_else(|| item.element_id.clone());
        let placement = placement_for_item(item);

        match blocks.last_mut() {
            Some(block)
                if block.id == block_id
                    && block.source_block_id == item.block_id
                    && block.placement == placement =>
            {
                block.item_ids.push(item.element_id.clone());
                block.fragment = merge_fragment(&block.fragment, &item.fragment);
                block.continuation_markers = continuation_markers_for_fragment(&block.fragment);
            }
            _ => blocks.push(PageBlock {
                id: block_id,
                source_block_id: item.block_id.clone(),
                item_ids: vec![item.element_id.clone()],
                placement,
                fragment: item.fragment.clone(),
                continuation_markers: item.continuation_markers.clone(),
            }),
        }
    }

    blocks
}

fn placement_for_item(item: &PageItem) -> BlockPlacement {
    match (&item.dual_dialogue_group, item.dual_dialogue_side) {
        (Some(group_id), Some(side)) => BlockPlacement::DualDialogue {
            group_id: group_id.clone(),
            side,
        },
        _ => BlockPlacement::Flow,
    }
}

fn flow_kind_name(kind: &FlowKind) -> &'static str {
    match kind {
        FlowKind::SceneHeading => "Scene Heading",
        FlowKind::Transition => "Transition",
        FlowKind::Section => "Section",
        FlowKind::Synopsis => "Synopsis",
        FlowKind::ColdOpening => "Cold Opening",
        FlowKind::NewAct => "New Act",
        FlowKind::EndOfAct => "End of Act",
        FlowKind::Action => "Action",
    }
}

fn dialogue_part_kind_name(kind: &DialoguePartKind) -> &'static str {
    match kind {
        DialoguePartKind::Character => "Character",
        DialoguePartKind::Parenthetical => "Parenthetical",
        DialoguePartKind::Dialogue => "Dialogue",
        DialoguePartKind::Lyric => "Lyric",
    }
}





fn merge_fragment(current: &Fragment, next: &Fragment) -> Fragment {
    use Fragment::*;

    match (current, next) {
        (ContinuedFromPrevAndToNext, _) | (_, ContinuedFromPrevAndToNext) => {
            ContinuedFromPrevAndToNext
        }
        (ContinuedFromPrev, ContinuedToNext) | (ContinuedToNext, ContinuedFromPrev) => {
            ContinuedFromPrevAndToNext
        }
        (ContinuedFromPrev, _) | (_, ContinuedFromPrev) => ContinuedFromPrev,
        (ContinuedToNext, _) | (_, ContinuedToNext) => ContinuedToNext,
        _ => Whole,
    }
}

fn continuation_markers_for_fragment(fragment: &Fragment) -> Vec<ContinuationMarker> {
    use ContinuationMarker::*;
    use Fragment::*;

    match fragment {
        Whole => Vec::new(),
        ContinuedFromPrev => vec![Continued],
        ContinuedToNext => vec![More],
        ContinuedFromPrevAndToNext => vec![Continued, More],
    }
}

fn first_page_number(scope: &PaginationScope) -> u32 {
    scope
        .body_start_page
        .unwrap_or_else(|| scope.title_page_count.map(|count| count + 1).unwrap_or(1))
}

fn page_kind(page_number: u32, scope: &PaginationScope) -> PageKind {
    match scope.body_start_page {
        Some(body_start) if page_number < body_start => PageKind::Title,
        _ => PageKind::Body,
    }
}

fn body_page_number(page_number: u32, scope: &PaginationScope) -> Option<u32> {
    let body_start = scope.body_start_page?;
    if page_number < body_start {
        None
    } else {
        Some(page_number - body_start + 1)
    }
}

fn title_page_number(page_number: u32, scope: &PaginationScope) -> Option<u32> {
    match scope.body_start_page {
        Some(body_start) if page_number < body_start => Some(page_number),
        None if scope.title_page_count.unwrap_or(0) > 0 => Some(page_number),
        _ => None,
    }
}

