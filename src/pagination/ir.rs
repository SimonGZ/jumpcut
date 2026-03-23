use crate::pagination::fixtures::{
    Fragment, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixtureSourceRefs, PaginationScope,
};
use crate::pagination::measurement::{
    measure_dialogue_part_lines, measure_dialogue_unit_lines, measure_dual_dialogue_unit_lines,
    measure_flow_unit_lines, measure_lyric_unit_lines, MeasurementConfig,
};
use crate::pagination::semantic::{
    DialoguePartKind, DialogueUnit, FlowKind, FlowUnit, SemanticScreenplay,
    SemanticUnit,
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

#[derive(Clone, Debug, PartialEq)]
pub struct PaginationConfig {
    pub lines_per_page: u32,
    pub measurement: MeasurementConfig,
}

impl PaginationConfig {
    pub fn screenplay(lines_per_page: u32) -> Self {
        Self {
            lines_per_page,
            measurement: MeasurementConfig::screenplay_default(),
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
        let mut pages: Vec<Page> = Vec::new();
        let mut next_page_number = semantic
            .starting_page_number
            .unwrap_or_else(|| first_page_number(&scope));
        let mut current_items: Vec<PageItem> = Vec::new();
        let mut current_lines = 0;
        let units = semantic.units;
        let style_profile = style_profile.into();
        let mut index = 0;

        while index < units.len() {
            match &units[index] {
                SemanticUnit::PageStart(_) => {
                    if !current_items.is_empty() {
                        pages.push(build_page(
                            pages.len(),
                            next_page_number,
                            &scope,
                            std::mem::take(&mut current_items),
                        ));
                        next_page_number += 1;
                        current_lines = 0;
                    }
                    index += 1;
                }
                unit => {
                    if let SemanticUnit::Flow(flow) = unit {
                        let remaining_lines =
                            config.lines_per_page.saturating_sub(current_lines);
                        let available_lines = if current_items.is_empty() {
                            config.lines_per_page
                        } else {
                            remaining_lines
                        };

                        if flow.cohesion.can_split
                            && measure_flow_unit_lines(flow, &config.measurement)
                                > available_lines
                        {
                            if let Some((current_segment, remainder)) =
                                split_flow_unit(flow, available_lines, &config.measurement)
                            {
                                current_items.push(flow_page_item(
                                    &current_segment,
                                    false,
                                    true,
                                ));

                                pages.push(build_page(
                                    pages.len(),
                                    next_page_number,
                                    &scope,
                                    std::mem::take(&mut current_items),
                                ));
                                next_page_number += 1;
                                current_lines = 0;

                                let mut carry = remainder;
                                loop {
                                    let carry_lines =
                                        measure_flow_unit_lines(&carry, &config.measurement);
                                    if carry_lines <= config.lines_per_page {
                                        current_items.push(flow_page_item(
                                            &carry,
                                            true,
                                            false,
                                        ));
                                        current_lines = current_lines.saturating_add(carry_lines);
                                        break;
                                    }

                                    if let Some((head, tail)) =
                                        split_flow_unit(
                                            &carry,
                                            config.lines_per_page,
                                            &config.measurement,
                                        )
                                    {
                                        current_items.push(flow_page_item(&head, true, true));
                                        pages.push(build_page(
                                            pages.len(),
                                            next_page_number,
                                            &scope,
                                            std::mem::take(&mut current_items),
                                        ));
                                        next_page_number += 1;
                                        current_lines = 0;
                                        carry = tail;
                                    } else {
                                        current_items.push(flow_page_item(&carry, true, false));
                                        current_lines =
                                            current_lines.saturating_add(carry_lines);
                                        break;
                                    }
                                }

                                index += 1;
                                continue;
                            } else if !current_items.is_empty() {
                                pages.push(build_page(
                                    pages.len(),
                                    next_page_number,
                                    &scope,
                                    std::mem::take(&mut current_items),
                                ));
                                next_page_number += 1;
                                current_lines = 0;
                                continue;
                            }
                        }
                    }

                    if let SemanticUnit::Dialogue(dialogue) = unit {
                        let remaining_lines =
                            config.lines_per_page.saturating_sub(current_lines);
                        let available_lines = if current_items.is_empty() {
                            config.lines_per_page
                        } else {
                            remaining_lines
                        };

                        if measure_dialogue_unit_lines(dialogue, &config.measurement)
                            > available_lines
                        {
                            if let Some((current_segment, remainder)) =
                                split_dialogue_unit(
                                    dialogue,
                                    available_lines,
                                    &config.measurement,
                                )
                            {
                                let placed_items = dialogue_items_with_fragment_markers(
                                    &current_segment,
                                    None,
                                    None,
                                    false,
                                    true,
                                );
                                current_items.extend(placed_items);

                                pages.push(build_page(
                                    pages.len(),
                                    next_page_number,
                                    &scope,
                                    std::mem::take(&mut current_items),
                                ));
                                next_page_number += 1;
                                current_lines = 0;

                                let mut carry = remainder;
                                loop {
                                    let carry_lines =
                                        measure_dialogue_unit_lines(&carry, &config.measurement);
                                    if carry_lines <= config.lines_per_page {
                                        let placed_items = dialogue_items_with_fragment_markers(
                                            &carry,
                                            None,
                                            None,
                                            true,
                                            false,
                                        );
                                        current_lines = current_lines.saturating_add(carry_lines);
                                        current_items.extend(placed_items);
                                        break;
                                    }

                                    if let Some((head, tail)) =
                                        split_dialogue_unit(
                                            &carry,
                                            config.lines_per_page,
                                            &config.measurement,
                                        )
                                    {
                                        let placed_items = dialogue_items_with_fragment_markers(
                                            &head,
                                            None,
                                            None,
                                            true,
                                            true,
                                        );
                                        current_items.extend(placed_items);
                                        pages.push(build_page(
                                            pages.len(),
                                            next_page_number,
                                            &scope,
                                            std::mem::take(&mut current_items),
                                        ));
                                        next_page_number += 1;
                                        current_lines = 0;
                                        carry = tail;
                                    } else {
                                        let placed_items = dialogue_items_with_fragment_markers(
                                            &carry,
                                            None,
                                            None,
                                            true,
                                            false,
                                        );
                                        current_lines =
                                            current_lines.saturating_add(carry_lines);
                                        current_items.extend(placed_items);
                                        break;
                                    }
                                }

                                index += 1;
                                continue;
                            } else if !current_items.is_empty() {
                                pages.push(build_page(
                                    pages.len(),
                                    next_page_number,
                                    &scope,
                                    std::mem::take(&mut current_items),
                                ));
                                next_page_number += 1;
                                current_lines = 0;
                                continue;
                            }
                        }
                    }

                    let unit_lines = measure_unit_lines(unit, &config.measurement);
                    let mut required_lines = unit_lines;
                    if should_keep_with_next(unit) {
                        if let Some(next_index) = next_placeable_unit_index(&units, index + 1) {
                            required_lines +=
                                measure_unit_lines(&units[next_index], &config.measurement);
                        }
                    }

                    let remaining_lines =
                        config.lines_per_page.saturating_sub(current_lines);
                    if !current_items.is_empty() && required_lines > remaining_lines {
                        pages.push(build_page(
                            pages.len(),
                            next_page_number,
                            &scope,
                            std::mem::take(&mut current_items),
                        ));
                        next_page_number += 1;
                        current_lines = 0;
                    }

                    let placed_items = page_items_from_semantic_unit(unit);
                    current_lines = current_lines.saturating_add(unit_lines);
                    current_items.extend(placed_items);
                    index += 1;
                }
            }
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

fn next_placeable_unit_index(units: &[SemanticUnit], start: usize) -> Option<usize> {
    units.iter()
        .enumerate()
        .skip(start)
        .find(|(_, unit)| !matches!(unit, SemanticUnit::PageStart(_)))
        .map(|(index, _)| index)
}

fn should_keep_with_next(unit: &SemanticUnit) -> bool {
    match unit {
        SemanticUnit::Flow(unit) => unit.cohesion.keep_with_next,
        SemanticUnit::Dialogue(unit) => unit.cohesion.keep_with_next,
        SemanticUnit::Lyric(unit) => unit.cohesion.keep_with_next,
        SemanticUnit::DualDialogue(unit) => unit.cohesion.keep_with_next,
        SemanticUnit::PageStart(_) => false,
    }
}

fn measure_unit_lines(unit: &SemanticUnit, measurement: &MeasurementConfig) -> u32 {
    match unit {
        SemanticUnit::PageStart(_) => 0,
        SemanticUnit::Flow(unit) => measure_flow_unit_lines(unit, measurement),
        SemanticUnit::Lyric(unit) => measure_lyric_unit_lines(unit, measurement),
        SemanticUnit::Dialogue(unit) => measure_dialogue_unit_lines(unit, measurement),
        SemanticUnit::DualDialogue(unit) => measure_dual_dialogue_unit_lines(unit, measurement),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SplitCandidateScore {
    awkward_continuation_penalty: u32,
    tiny_tail_penalty: u32,
    tiny_head_penalty: u32,
    sentence_boundary_penalty: u32,
    unused_space_penalty: u32,
    imbalance_penalty: u32,
}

#[derive(Clone, Debug)]
struct SplitCandidate<T> {
    prefix: T,
    suffix: T,
    score: SplitCandidateScore,
}

fn choose_best_split_candidate<T>(candidates: Vec<SplitCandidate<T>>) -> Option<SplitCandidate<T>> {
    candidates.into_iter().min_by_key(|candidate| candidate.score)
}

fn split_flow_unit(
    unit: &FlowUnit,
    available_lines: u32,
    measurement: &MeasurementConfig,
) -> Option<(FlowUnit, FlowUnit)> {
    let lines: Vec<&str> = unit.text.lines().collect();
    let explicit_boundary_count = lines.len().saturating_sub(1);
    let sentence_boundaries = sentence_split_offsets(&unit.text);
    if explicit_boundary_count == 0 && sentence_boundaries.is_empty() {
        return None;
    }

    let mut candidates = Vec::new();
    for split_index in 1..lines.len() {
        let prefix = FlowUnit {
            element_id: unit.element_id.clone(),
            kind: unit.kind.clone(),
            text: lines[..split_index].join("\n"),
            line_range: Some((1, split_index as u32)),
            scene_number: unit.scene_number.clone(),
            cohesion: unit.cohesion.clone(),
        };
        let prefix_lines = measure_flow_unit_lines(&prefix, measurement);
        if prefix_lines > available_lines {
            continue;
        }

        let suffix = FlowUnit {
            element_id: unit.element_id.clone(),
            kind: unit.kind.clone(),
            text: lines[split_index..].join("\n"),
            line_range: Some((split_index as u32 + 1, lines.len() as u32)),
            scene_number: unit.scene_number.clone(),
            cohesion: unit.cohesion.clone(),
        };
        let suffix_lines = measure_flow_unit_lines(&suffix, measurement);
        let ends_at_sentence = ends_at_sentence_boundary(&prefix.text);

        candidates.push(SplitCandidate {
            prefix,
            suffix,
            score: score_flow_split_candidate(
                prefix_lines,
                suffix_lines,
                available_lines,
                ends_at_sentence,
            ),
        });
    }

    for split_offset in sentence_boundaries {
        let prefix_text = unit.text[..split_offset].trim_end().to_string();
        let suffix_text = unit.text[split_offset..].trim_start().to_string();
        if prefix_text.is_empty() || suffix_text.is_empty() {
            continue;
        }

        let prefix = FlowUnit {
            element_id: unit.element_id.clone(),
            kind: unit.kind.clone(),
            text: prefix_text,
            line_range: None,
            scene_number: unit.scene_number.clone(),
            cohesion: unit.cohesion.clone(),
        };
        let prefix_lines = measure_flow_unit_lines(&prefix, measurement);
        if prefix_lines > available_lines {
            continue;
        }

        let suffix = FlowUnit {
            element_id: unit.element_id.clone(),
            kind: unit.kind.clone(),
            text: suffix_text,
            line_range: None,
            scene_number: unit.scene_number.clone(),
            cohesion: unit.cohesion.clone(),
        };
        let suffix_lines = measure_flow_unit_lines(&suffix, measurement);

        candidates.push(SplitCandidate {
            prefix,
            suffix,
            score: score_flow_split_candidate(
                prefix_lines,
                suffix_lines,
                available_lines,
                true,
            ),
        });
    }

    choose_best_split_candidate(candidates).map(|candidate| (candidate.prefix, candidate.suffix))
}

fn split_dialogue_unit(
    unit: &DialogueUnit,
    available_lines: u32,
    measurement: &MeasurementConfig,
) -> Option<(DialogueUnit, DialogueUnit)> {
    if unit.parts.len() < 2 {
        return None;
    }

    let mut lines_before_part = 0;
    let mut candidates = Vec::new();
    for (index, part) in unit.parts.iter().enumerate() {
        if can_split_within_dialogue_part(&part.kind) {
            for split_offset in sentence_split_offsets(&part.text) {
                let prefix_text = part.text[..split_offset].trim_end().to_string();
                let suffix_text = part.text[split_offset..].trim_start().to_string();
                if prefix_text.is_empty() || suffix_text.is_empty() {
                    continue;
                }

                let prefix_part_lines =
                    measure_dialogue_part_lines(&part.kind, &prefix_text, measurement);
                let prefix_lines = lines_before_part + prefix_part_lines;
                if prefix_lines > available_lines {
                    continue;
                }

                let mut prefix_parts = unit.parts[..index].to_vec();
                prefix_parts.push(crate::pagination::semantic::DialoguePart {
                    element_id: part.element_id.clone(),
                    kind: part.kind.clone(),
                    text: prefix_text,
                });

                let mut suffix_parts = Vec::with_capacity(unit.parts.len() - index);
                suffix_parts.push(crate::pagination::semantic::DialoguePart {
                    element_id: part.element_id.clone(),
                    kind: part.kind.clone(),
                    text: suffix_text,
                });
                suffix_parts.extend_from_slice(&unit.parts[index + 1..]);

                if !is_valid_dialogue_fragment(&prefix_parts)
                    || !is_valid_dialogue_fragment(&suffix_parts)
                {
                    continue;
                }

                let suffix_lines = measure_dialogue_parts_lines(&suffix_parts, measurement);
                candidates.push(SplitCandidate {
                    prefix: DialogueUnit {
                        block_id: unit.block_id.clone(),
                        parts: prefix_parts,
                        cohesion: unit.cohesion.clone(),
                    },
                    suffix: DialogueUnit {
                        block_id: unit.block_id.clone(),
                        parts: suffix_parts,
                        cohesion: unit.cohesion.clone(),
                    },
                    score: score_dialogue_split_candidate(
                        prefix_lines,
                        suffix_lines,
                        available_lines,
                        None,
                        true,
                    ),
                });
            }
        }

        let part_lines = measure_dialogue_part_lines(&part.kind, &part.text, measurement);
        let prefix_lines = lines_before_part + part_lines;
        if index == unit.parts.len() - 1 {
            break;
        }
        if prefix_lines > available_lines {
            break;
        }

        let prefix = &unit.parts[..=index];
        let suffix = &unit.parts[index + 1..];
        if is_valid_dialogue_fragment(prefix) && is_valid_dialogue_fragment(suffix) {
            let suffix_lines = measure_dialogue_parts_lines(suffix, measurement);
            candidates.push(SplitCandidate {
                prefix: DialogueUnit {
                    block_id: unit.block_id.clone(),
                    parts: prefix.to_vec(),
                    cohesion: unit.cohesion.clone(),
                },
                suffix: DialogueUnit {
                    block_id: unit.block_id.clone(),
                    parts: suffix.to_vec(),
                    cohesion: unit.cohesion.clone(),
                },
                score: score_dialogue_split_candidate(
                    prefix_lines,
                    suffix_lines,
                    available_lines,
                    suffix.first().map(|part| &part.kind),
                    ends_at_sentence_boundary(
                        &prefix.last().map(|part| part.text.as_str()).unwrap_or(""),
                    ),
                ),
            });
        }

        lines_before_part = prefix_lines;
    }

    choose_best_split_candidate(candidates).map(|candidate| (candidate.prefix, candidate.suffix))
}

fn can_split_within_dialogue_part(kind: &DialoguePartKind) -> bool {
    matches!(kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric)
}

fn is_valid_dialogue_fragment(parts: &[crate::pagination::semantic::DialoguePart]) -> bool {
    let has_spoken_content = parts.iter().any(|part| {
        matches!(
            part.kind,
            DialoguePartKind::Dialogue | DialoguePartKind::Lyric
        )
    });

    if !has_spoken_content {
        return false;
    }

    !matches!(
        parts.last().map(|part| &part.kind),
        Some(DialoguePartKind::Character | DialoguePartKind::Parenthetical)
    )
}

fn measure_dialogue_parts_lines(
    parts: &[crate::pagination::semantic::DialoguePart],
    measurement: &MeasurementConfig,
) -> u32 {
    parts.iter()
        .map(|part| measure_dialogue_part_lines(&part.kind, &part.text, measurement))
        .sum::<u32>()
        .max(1)
}

fn score_dialogue_split_candidate(
    prefix_lines: u32,
    suffix_lines: u32,
    available_lines: u32,
    suffix_first_kind: Option<&DialoguePartKind>,
    ends_at_sentence_boundary: bool,
) -> SplitCandidateScore {
    SplitCandidateScore {
        awkward_continuation_penalty: u32::from(matches!(
            suffix_first_kind,
            Some(DialoguePartKind::Parenthetical)
        )),
        tiny_tail_penalty: tiny_fragment_penalty(suffix_lines),
        tiny_head_penalty: tiny_fragment_penalty(prefix_lines),
        sentence_boundary_penalty: u32::from(!ends_at_sentence_boundary),
        unused_space_penalty: available_lines.saturating_sub(prefix_lines),
        imbalance_penalty: prefix_lines.abs_diff(suffix_lines),
    }
}

fn score_flow_split_candidate(
    prefix_lines: u32,
    suffix_lines: u32,
    available_lines: u32,
    ends_at_sentence_boundary: bool,
) -> SplitCandidateScore {
    SplitCandidateScore {
        awkward_continuation_penalty: 0,
        tiny_tail_penalty: tiny_fragment_penalty(suffix_lines),
        tiny_head_penalty: tiny_fragment_penalty(prefix_lines),
        sentence_boundary_penalty: u32::from(!ends_at_sentence_boundary),
        imbalance_penalty: prefix_lines.abs_diff(suffix_lines),
        unused_space_penalty: available_lines.saturating_sub(prefix_lines),
    }
}

fn tiny_fragment_penalty(lines: u32) -> u32 {
    2_u32.saturating_sub(lines)
}

fn sentence_split_offsets(text: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    for (index, (byte_index, ch)) in chars.iter().enumerate() {
        if !matches!(ch, '.' | '!' | '?') {
            continue;
        }

        let mut split_at = *byte_index + ch.len_utf8();
        let mut cursor = index + 1;

        while let Some((next_byte_index, next_char)) = chars.get(cursor) {
            if matches!(next_char, '"' | '\'' | ')' | ']') {
                split_at = *next_byte_index + next_char.len_utf8();
                cursor += 1;
                continue;
            }
            break;
        }

        let mut saw_whitespace = false;
        while let Some((_, next_char)) = chars.get(cursor) {
            if next_char.is_whitespace() {
                saw_whitespace = true;
                cursor += 1;
                continue;
            }
            break;
        }

        if saw_whitespace || cursor == chars.len() {
            offsets.push(split_at);
        }
    }

    offsets
}

fn ends_at_sentence_boundary(text: &str) -> bool {
    let trimmed = text.trim_end();
    trimmed.ends_with('.')
        || trimmed.ends_with('!')
        || trimmed.ends_with('?')
        || trimmed.ends_with(".)")
        || trimmed.ends_with("!)")
        || trimmed.ends_with("?)")
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

#[cfg(test)]
mod tests {
    use super::{split_dialogue_unit, split_flow_unit};
    use crate::pagination::{
        Cohesion, DialoguePart, DialoguePartKind, DialogueUnit, FlowKind, FlowUnit,
        MeasurementConfig,
    };

    #[test]
    fn flow_splitting_prefers_sentence_boundaries_inside_a_paragraph() {
        let unit = flow_unit(
            "Edward watches the room. He sees his future. He keeps walking anyway.",
        );
        let measurement = MeasurementConfig::screenplay_default();

        let (prefix, suffix) = split_flow_unit(&unit, 2, &measurement).unwrap();

        assert_eq!(prefix.text, "Edward watches the room.");
        assert_eq!(suffix.text, "He sees his future. He keeps walking anyway.");
    }

    #[test]
    fn flow_splitting_prefers_big_fish_sentence_break_after_witnesses_his_death() {
        let unit = flow_unit(
            "This time we don't cut. Instead, we HOLD ON Edward as he witnesses his death. He stares transfixed, perplexed and amused. Whatever he sees, it's not as dire as the other boys. His future has something strange in store.",
        );
        let measurement = MeasurementConfig::screenplay_default();

        let (prefix, suffix) = split_flow_unit(&unit, 2, &measurement).unwrap();

        assert_eq!(
            prefix.text,
            "This time we don't cut. Instead, we HOLD ON Edward as he witnesses his death."
        );
        assert_eq!(
            suffix.text,
            "He stares transfixed, perplexed and amused. Whatever he sees, it's not as dire as the other boys. His future has something strange in store."
        );
    }

    #[test]
    fn dialogue_splitting_can_split_inside_a_single_spoken_part_at_sentence_boundaries() {
        let unit = dialogue_unit(
            "EDWARD (CONT'D)",
            "Probably just as well. He would have told it wrong anyway. All the facts and none of the flavor.",
        );
        let measurement = MeasurementConfig::screenplay_default();

        let (prefix, suffix) = split_dialogue_unit(&unit, 2, &measurement).unwrap();

        assert_eq!(prefix.parts.len(), 2);
        assert_eq!(prefix.parts[0].text, "EDWARD (CONT'D)");
        assert_eq!(prefix.parts[1].text, "Probably just as well.");
        assert_eq!(
            suffix.parts[0].text,
            "He would have told it wrong anyway. All the facts and none of the flavor."
        );
    }

    #[test]
    fn dialogue_splitting_prefers_big_fish_song_break_after_unusual_man() {
        let unit = dialogue_unit(
            "PING (CONT'D)",
            "*But she won't. No, she can't. She needs a special special different unusual man. Because that girl, Who looks like me, She has wants, but she has needs.*",
        );
        let measurement = MeasurementConfig::screenplay_default();

        let (prefix, suffix) = split_dialogue_unit(&unit, 6, &measurement).unwrap();

        assert_eq!(
            prefix.parts[1].text,
            "*But she won't. No, she can't. She needs a special special different unusual man."
        );
        assert_eq!(
            suffix.parts[0].text,
            "Because that girl, Who looks like me, She has wants, but she has needs.*"
        );
    }

    fn flow_unit(text: &str) -> FlowUnit {
        FlowUnit {
            element_id: "el-00001".into(),
            kind: FlowKind::Action,
            text: text.into(),
            line_range: None,
            scene_number: None,
            cohesion: Cohesion {
                keep_together: false,
                keep_with_next: false,
                can_split: true,
            },
        }
    }

    fn dialogue_unit(character: &str, dialogue: &str) -> DialogueUnit {
        DialogueUnit {
            block_id: "block-00001".into(),
            parts: vec![
                DialoguePart {
                    element_id: "el-00001".into(),
                    kind: DialoguePartKind::Character,
                    text: character.into(),
                },
                DialoguePart {
                    element_id: "el-00002".into(),
                    kind: DialoguePartKind::Dialogue,
                    text: dialogue.into(),
                },
            ],
            cohesion: Cohesion {
                keep_together: false,
                keep_with_next: false,
                can_split: true,
            },
        }
    }
}
