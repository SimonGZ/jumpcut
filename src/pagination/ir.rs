use crate::pagination::dialogue_split::DialogueSplitPlan;
use crate::pagination::fixtures::{
    Fragment, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixtureSourceRefs, PaginationScope,
};
use crate::pagination::margin::line_height_for_element_type;
use crate::pagination::normalize_screenplay;
use crate::pagination::ScreenplayLayoutProfile;
use crate::pagination::semantic::{
    build_semantic_screenplay, DialoguePartKind, DialogueUnit, FlowKind, FlowUnit, SemanticScreenplay,
    SemanticUnit,
};
use crate::pagination::wrapping::{wrap_text_for_element, ElementType, WrapConfig};
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
                let items = page_items_from_layout_block(&block, geometry);
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
            if is_non_visual_element_kind(&element.kind) {
                continue;
            }
            if element.render_attributes.starts_new_page && !current_items.is_empty() {
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

    pub fn from_screenplay(
        screenplay_id: &str,
        screenplay: &Screenplay,
        lines_per_page: f32,
        scope: PaginationScope,
    ) -> Self {
        let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
        let style_profile = match layout_profile.style_profile {
            crate::pagination::StyleProfile::Screenplay => "standard",
            crate::pagination::StyleProfile::Multicam => "multicam",
        };
        let normalized = normalize_screenplay(screenplay_id, screenplay);
        let semantic = build_semantic_screenplay(normalized);
        let config = PaginationConfig::from_screenplay(screenplay, lines_per_page);

        Self::paginate(semantic, config, style_profile, scope)
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



fn page_items_from_layout_block(
    block: &crate::pagination::composer::LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> Vec<PageItem> {
    match block.unit {
        SemanticUnit::PageStart(_) => Vec::new(),
        SemanticUnit::Flow(unit) => vec![flow_page_item(
            unit,
            matches!(block.fragment, Fragment::ContinuedFromPrev | Fragment::ContinuedFromPrevAndToNext),
            matches!(block.fragment, Fragment::ContinuedToNext | Fragment::ContinuedFromPrevAndToNext),
        )],
        SemanticUnit::Lyric(unit) => vec![PageItem {
            element_id: unit.element_id.clone(),
            kind: "Lyric".into(),
            fragment: block.fragment.clone(),
            line_range: None,
            block_id: None,
            dual_dialogue_group: None,
            dual_dialogue_side: None,
            continuation_markers: continuation_markers_for_fragment(&block.fragment),
        }],
        SemanticUnit::Dialogue(unit) => {
            dialogue_items_for_fragment(
                unit,
                None,
                None,
                &block.fragment,
                block.dialogue_split.as_ref(),
                block.content_lines,
                geometry,
            )
        }
        SemanticUnit::DualDialogue(unit) => unit
            .sides
            .iter()
            .flat_map(|side| {
                dialogue_items_for_fragment(
                    &side.dialogue,
                    Some(unit.group_id.as_str()),
                    Some(side.side),
                    &block.fragment,
                    None,
                    block.content_lines,
                    geometry,
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

fn dialogue_items_for_fragment(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    fragment: &Fragment,
    split_plan: Option<&DialogueSplitPlan>,
    visible_content_lines: f32,
    geometry: &LayoutGeometry,
) -> Vec<PageItem> {
    if matches!(fragment, Fragment::Whole) {
        return dialogue_items_with_fragment_markers(unit, dual_group, dual_side, false, false);
    }

    if let Some(plan) = split_plan {
        match fragment {
            Fragment::ContinuedToNext => {
                return dialogue_fragment_items_from_plan(
                    unit,
                    dual_group,
                    dual_side,
                    fragment,
                    plan,
                    true,
                );
            }
            Fragment::ContinuedFromPrev => {
                return dialogue_fragment_items_from_plan(
                    unit,
                    dual_group,
                    dual_side,
                    fragment,
                    plan,
                    false,
                );
            }
            Fragment::Whole | Fragment::ContinuedFromPrevAndToNext => {}
        }
    }

    let line_counts = dialogue_part_line_counts(unit, dual_side, geometry);
    let line_heights = dialogue_part_line_heights(unit, dual_side, geometry);
    let total_lines: usize = line_counts.iter().sum();

    match fragment {
        Fragment::ContinuedToNext => dialogue_top_fragment_items(
            unit,
            dual_group,
            dual_side,
            &line_counts,
            visible_prefix_dialogue_lines(&line_counts, &line_heights, visible_content_lines),
        ),
        Fragment::ContinuedFromPrev => dialogue_bottom_fragment_items(
            unit,
            dual_group,
            dual_side,
            &line_counts,
            skipped_prefix_dialogue_lines_for_bottom_fragment(
                &line_counts,
                &line_heights,
                visible_content_lines,
            ),
        ),
        Fragment::ContinuedFromPrevAndToNext => dialogue_middle_fragment_items(
            unit,
            dual_group,
            dual_side,
            &line_counts,
            total_lines.saturating_sub(visible_prefix_dialogue_lines(
                &line_counts,
                &line_heights,
                visible_content_lines,
            )),
            visible_prefix_dialogue_lines(&line_counts, &line_heights, visible_content_lines),
        ),
        Fragment::Whole => unreachable!(),
    }
}

fn dialogue_fragment_items_from_plan(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    fragment: &Fragment,
    plan: &DialogueSplitPlan,
    use_top_lines: bool,
) -> Vec<PageItem> {
    let mut items = Vec::new();

    for (part, part_plan) in unit.parts.iter().zip(plan.parts.iter()) {
        let visible_lines = if use_top_lines {
            &part_plan.top_lines
        } else {
            &part_plan.bottom_lines
        };

        if visible_lines.is_empty() {
            continue;
        }

        let hidden_line_count = if use_top_lines {
            part_plan.bottom_lines.len()
        } else {
            part_plan.top_lines.len()
        };
        let line_range = if hidden_line_count == 0 {
            None
        } else if use_top_lines {
            Some((1, visible_lines.len() as u32))
        } else {
            Some((
                part_plan.top_lines.len() as u32 + 1,
                (part_plan.top_lines.len() + visible_lines.len()) as u32,
            ))
        };

        items.push(dialogue_fragment_item(
            unit,
            part,
            dual_group,
            dual_side,
            Fragment::Whole,
            line_range,
        ));
    }

    match fragment {
        Fragment::ContinuedToNext => {
            if let Some(last) = items.last_mut() {
                if last.line_range.is_some() {
                    last.fragment = merge_fragment(&last.fragment, &Fragment::ContinuedToNext);
                    last.continuation_markers =
                        continuation_markers_for_fragment(&last.fragment);
                }
            }
        }
        Fragment::ContinuedFromPrev => {
            if let Some(first) = items.first_mut() {
                if first.line_range.is_some() {
                    first.fragment = merge_fragment(&first.fragment, &Fragment::ContinuedFromPrev);
                    first.continuation_markers =
                        continuation_markers_for_fragment(&first.fragment);
                }
            }
        }
        Fragment::Whole | Fragment::ContinuedFromPrevAndToNext => {}
    }

    items
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

fn dialogue_part_line_counts(
    unit: &DialogueUnit,
    dual_side: Option<u8>,
    geometry: &LayoutGeometry,
) -> Vec<usize> {
    unit.parts
        .iter()
        .map(|part| {
            let element_type = match dual_side {
                Some(1) => ElementType::DualDialogueLeft,
                Some(_) => ElementType::DualDialogueRight,
                None => match part.kind {
                    DialoguePartKind::Character => ElementType::Character,
                    DialoguePartKind::Parenthetical => ElementType::Parenthetical,
                    DialoguePartKind::Dialogue => ElementType::Dialogue,
                    DialoguePartKind::Lyric => ElementType::Lyric,
                },
            };
            let config = WrapConfig::from_geometry(geometry, element_type);
            wrap_text_for_element(&part.text, &config).len()
        })
        .collect()
}

fn dialogue_part_line_heights(
    unit: &DialogueUnit,
    dual_side: Option<u8>,
    geometry: &LayoutGeometry,
) -> Vec<f32> {
    unit.parts
        .iter()
        .map(|part| {
            let element_type = match dual_side {
                Some(1) => ElementType::DualDialogueLeft,
                Some(_) => ElementType::DualDialogueRight,
                None => match part.kind {
                    DialoguePartKind::Character => ElementType::Character,
                    DialoguePartKind::Parenthetical => ElementType::Parenthetical,
                    DialoguePartKind::Dialogue => ElementType::Dialogue,
                    DialoguePartKind::Lyric => ElementType::Lyric,
                },
            };
            line_height_for_element_type(geometry, element_type)
        })
        .collect()
}

fn visible_prefix_dialogue_lines(
    line_counts: &[usize],
    line_heights: &[f32],
    visible_content_height: f32,
) -> usize {
    let mut visible_lines = 0;
    let mut used_height = 0.0;

    for (&line_count, &line_height) in line_counts.iter().zip(line_heights.iter()) {
        for _ in 0..line_count {
            if used_height + line_height > visible_content_height + f32::EPSILON {
                return visible_lines;
            }
            used_height += line_height;
            visible_lines += 1;
        }
    }

    visible_lines
}

fn skipped_prefix_dialogue_lines_for_bottom_fragment(
    line_counts: &[usize],
    line_heights: &[f32],
    visible_content_height: f32,
) -> usize {
    let mut visible_suffix_lines = 0;
    let mut used_height = 0.0;

    for (&line_count, &line_height) in line_counts.iter().zip(line_heights.iter()).rev() {
        for _ in 0..line_count {
            if used_height + line_height > visible_content_height + f32::EPSILON {
                let total_lines: usize = line_counts.iter().sum();
                return total_lines.saturating_sub(visible_suffix_lines);
            }
            used_height += line_height;
            visible_suffix_lines += 1;
        }
    }

    line_counts.iter().sum::<usize>().saturating_sub(visible_suffix_lines)
}

fn dialogue_top_fragment_items(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    line_counts: &[usize],
    visible_lines: usize,
) -> Vec<PageItem> {
    let mut items = Vec::new();
    let mut remaining = visible_lines;

    for (part, part_lines) in unit.parts.iter().zip(line_counts.iter().copied()) {
        if remaining == 0 {
            break;
        }

        let taken = remaining.min(part_lines);
        if taken == 0 {
            continue;
        }

        items.push(dialogue_fragment_item(
            unit,
            part,
            dual_group,
            dual_side,
            Fragment::Whole,
            (taken < part_lines).then_some((1, taken as u32)),
        ));
        remaining -= taken;
    }

    if let Some(last) = items.last_mut() {
        last.fragment = merge_fragment(&last.fragment, &Fragment::ContinuedToNext);
        last.continuation_markers = continuation_markers_for_fragment(&last.fragment);
    }

    items
}

fn dialogue_bottom_fragment_items(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    line_counts: &[usize],
    mut skipped_lines: usize,
) -> Vec<PageItem> {
    let mut items = Vec::new();

    for (part, part_lines) in unit.parts.iter().zip(line_counts.iter().copied()) {
        if skipped_lines >= part_lines {
            skipped_lines -= part_lines;
            continue;
        }

        let start_line = skipped_lines + 1;
        let line_range = (start_line > 1).then_some((start_line as u32, part_lines as u32));

        items.push(dialogue_fragment_item(
            unit,
            part,
            dual_group,
            dual_side,
            Fragment::Whole,
            line_range,
        ));
        skipped_lines = 0;
    }

    if let Some(first) = items.first_mut() {
        first.fragment = merge_fragment(&first.fragment, &Fragment::ContinuedFromPrev);
        first.continuation_markers = continuation_markers_for_fragment(&first.fragment);
    }

    items
}

fn dialogue_middle_fragment_items(
    unit: &DialogueUnit,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    line_counts: &[usize],
    skipped_lines: usize,
    visible_lines: usize,
) -> Vec<PageItem> {
    let mut items = dialogue_bottom_fragment_items(unit, dual_group, dual_side, line_counts, skipped_lines);
    let mut remaining = visible_lines;

    for item in &mut items {
        if remaining == 0 {
            item.line_range = None;
            continue;
        }
        if let Some((start, end)) = item.line_range {
            let count = (end - start + 1) as usize;
            if count > remaining {
                item.line_range = Some((start, start + remaining as u32 - 1));
                remaining = 0;
            } else {
                remaining -= count;
            }
        } else {
            remaining = remaining.saturating_sub(1);
        }
    }

    items.retain(|item| item.line_range.map(|(start, end)| start <= end).unwrap_or(true));

    if let Some(last) = items.last_mut() {
        last.fragment = merge_fragment(&last.fragment, &Fragment::ContinuedToNext);
        last.continuation_markers = continuation_markers_for_fragment(&last.fragment);
    }

    items
}

fn dialogue_fragment_item(
    unit: &DialogueUnit,
    part: &crate::pagination::semantic::DialoguePart,
    dual_group: Option<&str>,
    dual_side: Option<u8>,
    fragment: Fragment,
    line_range: Option<(u32, u32)>,
) -> PageItem {
    PageItem {
        element_id: part.element_id.clone(),
        kind: dialogue_part_kind_name(&part.kind).to_string(),
        fragment: fragment.clone(),
        line_range,
        block_id: Some(unit.block_id.clone()),
        dual_dialogue_group: dual_group.map(str::to_string),
        dual_dialogue_side: dual_side,
        continuation_markers: continuation_markers_for_fragment(&fragment),
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

fn is_non_visual_element_kind(kind: &str) -> bool {
    matches!(kind, "Section" | "Synopsis")
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
