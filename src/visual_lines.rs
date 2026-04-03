use crate::pagination::composer::{self, LayoutBlock};
use crate::pagination::margin::line_height_for_element_type;
use crate::pagination::paginator;
use crate::pagination::wrapping::{self, ElementType, WrappedStyledFragment};
use crate::pagination::{
    build_semantic_screenplay, normalize_screenplay, DialoguePartKind, LayoutGeometry, Page,
    PageKind, PaginatedScreenplay, PaginationConfig, PaginationScope, ScreenplayLayoutProfile,
    SemanticUnit, StyleProfile,
};
use crate::styled_text::StyledText;
use crate::Screenplay;

const DEFAULT_LINES_PER_PAGE: f32 = 54.0;
const TITLE_PAGE_METADATA_KEYS: &[&str] = &[
    "title",
    "credit",
    "author",
    "authors",
    "source",
    "draft",
    "draft date",
    "contact",
    "copyright",
];

#[derive(Clone, Debug)]
pub(crate) struct VisualLine {
    pub text: String,
    pub counted: bool,
    pub centered: bool,
    pub fragments: Vec<VisualFragment>,
}

#[derive(Clone, Debug)]
pub(crate) struct VisualPage {
    pub page: Page,
    pub lines: Vec<VisualLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VisualFragment {
    pub text: String,
    pub styles: Vec<String>,
}

pub(crate) fn render_paginated_visual_pages(screenplay: &Screenplay) -> Vec<VisualPage> {
    let screenplay_id = "screenplay";
    let scope = default_pagination_scope(screenplay);
    let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
    let style_profile = style_profile_name(&layout_profile);
    let normalized = normalize_screenplay(screenplay_id, screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig {
        lines_per_page: DEFAULT_LINES_PER_PAGE,
        geometry: layout_profile.to_pagination_geometry(),
    };
    let blocks = composer::compose(&semantic.units, &config.geometry);
    let actual =
        PaginatedScreenplay::paginate(semantic.clone(), config.clone(), style_profile, scope);
    let layout_pages = nonempty_layout_pages(&blocks, &config.geometry, config.lines_per_page);

    actual
        .pages
        .into_iter()
        .zip(layout_pages)
        .map(|(page, layout_page)| VisualPage {
            page,
            lines: render_layout_page_lines(&layout_page, &config.geometry),
        })
        .collect()
}

pub(crate) fn render_unpaginated_visual_lines(screenplay: &Screenplay) -> Vec<VisualLine> {
    let screenplay_id = "screenplay";
    let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
    let config = PaginationConfig {
        lines_per_page: DEFAULT_LINES_PER_PAGE,
        geometry: layout_profile.to_pagination_geometry(),
    };
    let normalized = normalize_screenplay(screenplay_id, screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let blocks = composer::compose(&semantic.units, &config.geometry);

    blocks
        .iter()
        .filter(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        .flat_map(|block| render_continuous_block_lines(block, &config.geometry))
        .collect()
}

fn style_profile_name(layout_profile: &ScreenplayLayoutProfile) -> &'static str {
    match layout_profile.style_profile {
        StyleProfile::Screenplay => "standard",
        StyleProfile::Multicam => "multicam",
    }
}

fn default_pagination_scope(screenplay: &Screenplay) -> PaginationScope {
    if has_title_page_metadata(screenplay) {
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        }
    } else {
        PaginationScope {
            title_page_count: None,
            body_start_page: None,
        }
    }
}

fn has_title_page_metadata(screenplay: &Screenplay) -> bool {
    TITLE_PAGE_METADATA_KEYS
        .iter()
        .any(|key| screenplay.metadata.contains_key(*key))
}

fn nonempty_layout_pages<'a>(
    blocks: &'a [LayoutBlock<'a>],
    geometry: &LayoutGeometry,
    lines_per_page: f32,
) -> Vec<paginator::Page<'a>> {
    paginator::paginate(blocks, lines_per_page, geometry)
        .into_iter()
        .filter(|page| {
            page.blocks
                .iter()
                .any(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        })
        .collect()
}

fn render_continuous_block_lines(
    block: &LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> Vec<VisualLine> {
    let mut lines = Vec::new();
    for _ in 0..(block.spacing_above.round() as usize) {
        lines.push(VisualLine {
            text: String::new(),
            counted: true,
            centered: false,
            fragments: Vec::new(),
        });
    }
    lines.extend(render_layout_block_lines(block, geometry));
    lines
}

fn render_layout_page_lines(
    layout_page: &paginator::Page<'_>,
    geometry: &LayoutGeometry,
) -> Vec<VisualLine> {
    let mut lines = Vec::new();

    for block in &layout_page.blocks {
        for _ in 0..(block.spacing_above.round() as usize) {
            lines.push(VisualLine {
                text: String::new(),
                counted: true,
                centered: false,
                fragments: Vec::new(),
            });
        }
        lines.extend(render_layout_block_lines(block, geometry));
    }

    lines
}

fn render_layout_block_lines(block: &LayoutBlock<'_>, geometry: &LayoutGeometry) -> Vec<VisualLine> {
    if let SemanticUnit::Dialogue(dialogue) = block.unit {
        return render_dialogue_fragment_lines(
            dialogue,
            &block.fragment,
            block.dialogue_split.as_ref(),
            block.content_lines,
            geometry,
        );
    }

    if let SemanticUnit::Flow(flow) = block.unit {
        if let Some(plan) = block.flow_split.as_ref() {
            let element_type = ElementType::from_flow_kind(&flow.kind);
            let lines = if let Some(inline_text) = &flow.inline_text {
                let fragment_text = match block.fragment {
                    crate::pagination::Fragment::ContinuedToNext => {
                        inline_text.slice(0, plan.top_end_offset)
                    }
                    crate::pagination::Fragment::ContinuedFromPrev => {
                        inline_text.slice(plan.bottom_start_offset, inline_text.plain_text.len())
                    }
                    crate::pagination::Fragment::ContinuedFromPrevAndToNext => {
                        inline_text.slice(0, plan.top_end_offset)
                    }
                    crate::pagination::Fragment::Whole => inline_text.clone(),
                };

                render_indented_styled_lines(&fragment_text, element_type, geometry, flow.centered)
                    .into_iter()
                    .map(|line| rendered_element_line_from_styled(line, element_type, flow.centered))
                    .collect()
            } else {
                let text = match block.fragment {
                    crate::pagination::Fragment::ContinuedToNext => plan.top_text.clone(),
                    crate::pagination::Fragment::ContinuedFromPrev => plan.bottom_text.clone(),
                    crate::pagination::Fragment::ContinuedFromPrevAndToNext => plan.top_text.clone(),
                    crate::pagination::Fragment::Whole => flow.text.clone(),
                };

                render_indented_lines(&text, element_type, geometry, flow.centered)
                    .into_iter()
                    .map(|text| RenderedElementLine {
                        fragments: vec![plain_fragment_for_text(&text)],
                        text,
                        element_type,
                        centered: flow.centered,
                    })
                    .collect()
            };

            return counted_visual_lines(lines, geometry);
        }
    }

    let all_lines = render_semantic_unit_lines(block.unit, geometry);
    let lines = match block.fragment {
        crate::pagination::Fragment::Whole => all_lines,
        crate::pagination::Fragment::ContinuedToNext => {
            take_rendered_lines_from_top_by_height(&all_lines, block.content_lines, geometry)
        }
        crate::pagination::Fragment::ContinuedFromPrev => {
            take_rendered_lines_from_bottom_by_height(&all_lines, block.content_lines, geometry)
        }
        crate::pagination::Fragment::ContinuedFromPrevAndToNext => {
            take_rendered_lines_from_top_by_height(&all_lines, block.content_lines, geometry)
        }
    };

    counted_visual_lines(lines, geometry)
}

fn render_dialogue_fragment_lines(
    dialogue: &crate::pagination::DialogueUnit,
    fragment: &crate::pagination::Fragment,
    split_plan: Option<&crate::pagination::dialogue_split::DialogueSplitPlan>,
    content_lines: f32,
    geometry: &LayoutGeometry,
) -> Vec<VisualLine> {
    if let Some(plan) = split_plan {
        match fragment {
            crate::pagination::Fragment::ContinuedToNext => {
                let mut lines = plan
                    .parts
                    .iter()
                    .zip(dialogue.parts.iter())
                    .flat_map(|(part, dialogue_part)| {
                        let element_type =
                            ElementType::from_dialogue_part_kind(&dialogue_part.kind);
                        counted_visual_lines(
                            render_split_dialogue_part_lines(
                                dialogue_part,
                                part.top_text.as_str(),
                                0,
                                part.top_end_offset,
                                element_type,
                                geometry,
                            ),
                            geometry,
                        )
                    })
                    .collect::<Vec<_>>();
                lines.push(render_more_marker_line(geometry));
                return lines;
            }
            crate::pagination::Fragment::ContinuedFromPrev => {
                let continuation_prefix = render_dialogue_continuation_prefix(dialogue, geometry);
                let mut lines = continuation_prefix
                    .into_iter()
                    .map(|text| VisualLine {
                        fragments: vec![plain_fragment_for_text(&text)],
                        text,
                        counted: false,
                        centered: false,
                    })
                    .collect::<Vec<_>>();
                lines.extend(counted_visual_lines(
                    plan.parts
                        .iter()
                        .zip(dialogue.parts.iter())
                        .flat_map(|(part, dialogue_part)| {
                            let element_type =
                                ElementType::from_dialogue_part_kind(&dialogue_part.kind);
                            render_split_dialogue_part_lines(
                                dialogue_part,
                                part.bottom_text.as_str(),
                                part.bottom_start_offset,
                                dialogue_part.text.len(),
                                element_type,
                                geometry,
                            )
                                .into_iter()
                        })
                        .collect::<Vec<_>>(),
                    geometry,
                ));
                return lines;
            }
            crate::pagination::Fragment::Whole
            | crate::pagination::Fragment::ContinuedFromPrevAndToNext => {}
        }
    }

    let all_lines = render_semantic_unit_lines(&SemanticUnit::Dialogue(dialogue.clone()), geometry);
    let continuation_prefix = render_dialogue_continuation_prefix(dialogue, geometry);

    match fragment {
        crate::pagination::Fragment::Whole => counted_visual_lines(all_lines, geometry),
        crate::pagination::Fragment::ContinuedToNext => {
            let lines = take_rendered_lines_from_top_by_height(&all_lines, content_lines, geometry);
            let mut lines = counted_visual_lines(lines, geometry);
            lines.push(render_more_marker_line(geometry));
            lines
        }
        crate::pagination::Fragment::ContinuedFromPrev => continuation_prefix
            .into_iter()
            .map(|text| VisualLine {
                fragments: vec![plain_fragment_for_text(&text)],
                text,
                counted: false,
                centered: false,
            })
            .chain(counted_visual_lines(
                take_rendered_lines_from_bottom_by_height(&all_lines, content_lines, geometry),
                geometry,
            ))
            .collect(),
        crate::pagination::Fragment::ContinuedFromPrevAndToNext => {
            let mut lines = continuation_prefix
                .into_iter()
                .map(|text| VisualLine {
                    fragments: vec![plain_fragment_for_text(&text)],
                    text,
                    counted: false,
                    centered: false,
                })
                .chain(counted_visual_lines(
                    take_rendered_lines_from_top_by_height(&all_lines, content_lines, geometry),
                    geometry,
                ))
                .collect::<Vec<_>>();
            lines.push(render_more_marker_line(geometry));
            lines
        }
    }
}

fn render_split_dialogue_part_lines(
    dialogue_part: &crate::pagination::DialoguePart,
    plain_text: &str,
    start_offset: usize,
    end_offset: usize,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> Vec<RenderedElementLine> {
    if let Some(inline_text) = &dialogue_part.inline_text {
        return render_indented_styled_lines(
            &inline_text.slice(start_offset, end_offset),
            element_type,
            geometry,
            dialogue_part.centered,
        )
        .into_iter()
        .map(|line| rendered_element_line_from_styled(line, element_type, dialogue_part.centered))
        .collect();
    }

    render_indented_lines(plain_text, element_type, geometry, dialogue_part.centered)
        .into_iter()
        .map(|text| RenderedElementLine {
            fragments: vec![plain_fragment_for_text(&text)],
            text,
            element_type,
            centered: dialogue_part.centered,
        })
        .collect()
}

fn render_dialogue_continuation_prefix(
    dialogue: &crate::pagination::DialogueUnit,
    geometry: &LayoutGeometry,
) -> Vec<String> {
    dialogue
        .parts
        .iter()
        .take_while(|part| matches!(part.kind, DialoguePartKind::Character))
        .flat_map(|part| {
            render_indented_lines(
                &continued_character_cue_text(&part.text),
                ElementType::Character,
                geometry,
                false,
            )
        })
        .collect()
}

fn continued_character_cue_text(text: &str) -> String {
    let trimmed = text.trim_end();
    let upper = trimmed.to_ascii_uppercase();

    if upper.ends_with("(CONT'D)") || upper.ends_with("(CONT’D)") {
        trimmed.to_string()
    } else {
        format!("{trimmed} (CONT'D)")
    }
}

fn render_more_marker_line(geometry: &LayoutGeometry) -> VisualLine {
    VisualLine {
        text: render_indented_lines("(MORE)", ElementType::Character, geometry, false)
            .into_iter()
            .next()
            .unwrap_or_else(|| "(MORE)".to_string()),
        counted: false,
        centered: false,
        fragments: vec![plain_fragment_for_text(
            &render_indented_lines("(MORE)", ElementType::Character, geometry, false)
                .into_iter()
                .next()
                .unwrap_or_else(|| "(MORE)".to_string()),
        )],
    }
}

fn render_semantic_unit_lines(
    unit: &SemanticUnit,
    geometry: &LayoutGeometry,
) -> Vec<RenderedElementLine> {
    match unit {
        SemanticUnit::PageStart(_) => Vec::new(),
        SemanticUnit::Flow(flow) => {
            let element_type = ElementType::from_flow_kind(&flow.kind);
            if let Some(inline_text) = &flow.inline_text {
                return render_indented_styled_lines(
                    inline_text,
                    element_type,
                    geometry,
                    flow.centered,
                )
                    .into_iter()
                    .map(|line| rendered_element_line_from_styled(line, element_type, flow.centered))
                    .collect();
            }
            render_indented_lines(&flow.text, element_type, geometry, flow.centered)
                .into_iter()
                .map(|text| RenderedElementLine {
                    fragments: vec![plain_fragment_for_text(&text)],
                    text,
                    element_type,
                    centered: flow.centered,
                })
                .collect()
        }
        SemanticUnit::Lyric(lyric) => {
            if let Some(inline_text) = &lyric.inline_text {
                return render_indented_styled_lines(
                    inline_text,
                    ElementType::Lyric,
                    geometry,
                    lyric.centered,
                )
                    .into_iter()
                    .map(|line| rendered_element_line_from_styled(line, ElementType::Lyric, lyric.centered))
                    .collect();
            }
            render_indented_lines(&lyric.text, ElementType::Lyric, geometry, lyric.centered)
                .into_iter()
                .map(|text| RenderedElementLine {
                    fragments: vec![plain_fragment_for_text(&text)],
                    text,
                    element_type: ElementType::Lyric,
                    centered: lyric.centered,
                })
                .collect()
        }
        SemanticUnit::Dialogue(dialogue) => dialogue
            .parts
            .iter()
            .flat_map(|part| {
                let element_type = ElementType::from_dialogue_part_kind(&part.kind);
                if let Some(inline_text) = &part.inline_text {
                    return render_indented_styled_lines(
                        inline_text,
                        element_type,
                        geometry,
                        part.centered,
                    )
                        .into_iter()
                        .map(move |line| rendered_element_line_from_styled(line, element_type, part.centered))
                        .collect::<Vec<_>>();
                }
                render_indented_lines(&part.text, element_type, geometry, part.centered)
                    .into_iter()
                    .map(move |text| RenderedElementLine {
                        fragments: vec![plain_fragment_for_text(&text)],
                        text,
                        element_type,
                        centered: part.centered,
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
        SemanticUnit::DualDialogue(dual) => {
            let left_lines = dual
                .sides
                .iter()
                .find(|side| side.side == 1)
                .map(|side| {
                    render_dual_dialogue_side_rendered_lines(
                        &side.dialogue,
                        ElementType::DualDialogueLeft,
                        geometry,
                    )
                })
                .unwrap_or_default();
            let right_lines = dual
                .sides
                .iter()
                .find(|side| side.side == 2)
                .map(|side| {
                    render_dual_dialogue_side_rendered_lines(
                        &side.dialogue,
                        ElementType::DualDialogueRight,
                        geometry,
                    )
                })
                .unwrap_or_default();

            let right_indent = indent_spaces_for_element_type(ElementType::DualDialogueRight, geometry);
            let mut lines = Vec::new();
            for index in 0..left_lines.len().max(right_lines.len()) {
                let left = left_lines.get(index).cloned().unwrap_or_else(empty_rendered_styled_line);
                let right = right_lines
                    .get(index)
                    .cloned()
                    .unwrap_or_else(empty_rendered_styled_line);

                if right.text.is_empty() {
                    lines.push(RenderedElementLine {
                        fragments: left.fragments,
                        text: left.text,
                        element_type: ElementType::Dialogue,
                        centered: false,
                    });
                } else if left.text.is_empty() {
                    let gutter = " ".repeat(right_indent);
                    let text = format!("{gutter}{}", right.text);
                    let fragments = std::iter::once(plain_fragment_for_text(&gutter))
                        .chain(right.fragments.into_iter())
                        .collect();
                    lines.push(RenderedElementLine {
                        fragments,
                        text,
                        element_type: ElementType::Dialogue,
                        centered: false,
                    });
                } else {
                    let gutter_width = right_indent.saturating_sub(left.text.chars().count());
                    let gutter = " ".repeat(gutter_width);
                    let text = format!("{}{}{}", left.text, gutter, right.text);
                    let fragments = left
                        .fragments
                        .into_iter()
                        .chain(std::iter::once(plain_fragment_for_text(&gutter)))
                        .chain(right.fragments.into_iter())
                        .collect();
                    lines.push(RenderedElementLine {
                        fragments,
                        text,
                        element_type: ElementType::Dialogue,
                        centered: false,
                    });
                }
            }
            lines
        }
    }
}

fn render_dual_dialogue_side_rendered_lines(
    dialogue: &crate::pagination::DialogueUnit,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> Vec<RenderedStyledLine> {
    let config = wrapping::WrapConfig::from_geometry(geometry, element_type);
    dialogue
        .parts
        .iter()
        .flat_map(|part| {
            if let Some(inline_text) = &part.inline_text {
                wrapping::wrap_styled_text_for_element(inline_text, &config)
                    .into_iter()
                    .map(|line| RenderedStyledLine {
                        text: line.text,
                        fragments: line
                            .fragments
                            .into_iter()
                            .map(styled_fragment_to_visual_fragment)
                            .collect(),
                    })
                    .collect::<Vec<_>>()
            } else {
                wrapping::wrap_text_for_element(&part.text, &config)
                    .into_iter()
                    .map(|text| RenderedStyledLine {
                        fragments: vec![plain_fragment_for_text(&text)],
                        text,
                    })
                    .collect::<Vec<_>>()
            }
        })
        .collect()
}

fn render_indented_lines(
    text: &str,
    element_type: ElementType,
    geometry: &LayoutGeometry,
    centered: bool,
) -> Vec<String> {
    let config = wrapping::WrapConfig::from_geometry(geometry, element_type);
    let indent = if centered {
        String::new()
    } else {
        " ".repeat(indent_spaces_for_element_type(element_type, geometry))
    };
    wrapped_visual_lines(element_type, text, &config)
        .into_iter()
        .map(|line| format!("{indent}{line}"))
        .collect()
}

fn render_indented_styled_lines(
    text: &StyledText,
    element_type: ElementType,
    geometry: &LayoutGeometry,
    centered: bool,
) -> Vec<RenderedStyledLine> {
    let config = wrapping::WrapConfig::from_geometry(geometry, element_type);
    let indent = if centered {
        String::new()
    } else {
        " ".repeat(indent_spaces_for_element_type(element_type, geometry))
    };

    wrapping::wrap_styled_text_for_element(text, &config)
        .into_iter()
        .map(|line| {
            let mut fragments = Vec::new();
            if !indent.is_empty() {
                fragments.push(VisualFragment {
                    text: indent.clone(),
                    styles: Vec::new(),
                });
            }
            fragments.extend(
                line.fragments
                    .into_iter()
                    .map(styled_fragment_to_visual_fragment),
            );

            RenderedStyledLine {
                text: format!("{indent}{}", line.text),
                fragments,
            }
        })
        .collect()
}

fn wrapped_visual_lines(
    element_type: ElementType,
    text: &str,
    config: &wrapping::WrapConfig,
) -> Vec<String> {
    if matches!(element_type, ElementType::Action) && text.is_empty() {
        return vec![String::new()];
    }

    wrapping::wrap_text_for_element(text, config)
}

fn counted_visual_lines(lines: Vec<RenderedElementLine>, geometry: &LayoutGeometry) -> Vec<VisualLine> {
    let mut rendered = Vec::new();

    for line in lines {
        if uses_double_spaced_rows(line.element_type, geometry) {
            rendered.push(VisualLine {
                text: String::new(),
                counted: true,
                centered: false,
                fragments: Vec::new(),
            });
        }
        rendered.push(VisualLine {
            text: line.text,
            counted: true,
            centered: line.centered,
            fragments: line.fragments,
        });
    }

    rendered
}

fn uses_double_spaced_rows(element_type: ElementType, geometry: &LayoutGeometry) -> bool {
    (line_height_for_element_type(geometry, element_type) - 2.0).abs() < f32::EPSILON
}

fn take_rendered_lines_from_top_by_height(
    lines: &[RenderedElementLine],
    visible_height: f32,
    geometry: &LayoutGeometry,
) -> Vec<RenderedElementLine> {
    let mut used_height = 0.0;
    let mut visible = Vec::new();

    for line in lines {
        let line_height = line_height_for_element_type(geometry, line.element_type);
        if used_height + line_height > visible_height + f32::EPSILON {
            break;
        }
        visible.push(line.clone());
        used_height += line_height;
    }

    visible
}

fn take_rendered_lines_from_bottom_by_height(
    lines: &[RenderedElementLine],
    visible_height: f32,
    geometry: &LayoutGeometry,
) -> Vec<RenderedElementLine> {
    let mut used_height = 0.0;
    let mut start_index = lines.len();

    for (index, line) in lines.iter().enumerate().rev() {
        let line_height = line_height_for_element_type(geometry, line.element_type);
        if used_height + line_height > visible_height + f32::EPSILON {
            break;
        }
        used_height += line_height;
        start_index = index;
    }

    lines[start_index..].to_vec()
}

fn indent_spaces_for_element_type(element_type: ElementType, geometry: &LayoutGeometry) -> usize {
    let left_indent_in = match element_type {
        ElementType::Action | ElementType::SceneHeading => geometry.action_left,
        ElementType::ColdOpening => geometry.cold_opening_left,
        ElementType::NewAct => geometry.new_act_left,
        ElementType::EndOfAct => geometry.end_of_act_left,
        ElementType::Character => geometry.character_left,
        ElementType::Dialogue => geometry.dialogue_left,
        ElementType::Parenthetical => geometry.parenthetical_left,
        ElementType::Transition => geometry.transition_left,
        ElementType::Lyric => geometry.lyric_left,
        ElementType::DualDialogueLeft => geometry.dual_dialogue_left_left,
        ElementType::DualDialogueRight => geometry.dual_dialogue_right_left,
    };

    ((left_indent_in - geometry.action_left) * geometry.cpi).floor() as usize
}

#[derive(Clone)]
struct RenderedElementLine {
    text: String,
    element_type: ElementType,
    fragments: Vec<VisualFragment>,
    centered: bool,
}

#[derive(Clone)]
struct RenderedStyledLine {
    text: String,
    fragments: Vec<VisualFragment>,
}

fn empty_rendered_styled_line() -> RenderedStyledLine {
    RenderedStyledLine {
        text: String::new(),
        fragments: Vec::new(),
    }
}

pub(crate) fn display_page_number(page: &Page) -> Option<u32> {
    if matches!(page.metadata.kind, PageKind::Title) {
        return None;
    }

    let display_number = page
        .metadata
        .body_page_number
        .unwrap_or(page.metadata.number);
    if display_number == 1 {
        return None;
    }

    Some(display_number)
}

fn styled_fragment_to_visual_fragment(fragment: WrappedStyledFragment) -> VisualFragment {
    VisualFragment {
        text: fragment.text,
        styles: fragment.styles,
    }
}

fn plain_fragment_for_text(text: &str) -> VisualFragment {
    VisualFragment {
        text: text.to_string(),
        styles: Vec::new(),
    }
}

fn rendered_element_line_from_styled(
    line: RenderedStyledLine,
    element_type: ElementType,
    centered: bool,
) -> RenderedElementLine {
    RenderedElementLine {
        text: line.text,
        element_type,
        fragments: line.fragments,
        centered,
    }
}
