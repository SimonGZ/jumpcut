use crate::pagination::fixtures::{
    Fragment, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixtureSourceRefs, PaginationScope,
};

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

impl PaginatedScreenplay {
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
                pages.push(build_page(
                    pages.len(),
                    next_page_number,
                    &scope,
                    std::mem::take(&mut current_items),
                ));
                next_page_number += 1;
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
