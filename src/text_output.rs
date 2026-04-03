use crate::pagination::composer::{self, LayoutBlock};
use crate::pagination::margin::{calculate_element_width, line_height_for_element_type};
use crate::pagination::paginator;
use crate::pagination::wrapping::{self, ElementType};
use crate::pagination::{
    build_semantic_screenplay, normalize_screenplay, DialoguePartKind, LayoutGeometry,
    Page, PageKind, PaginatedScreenplay, PaginationConfig, PaginationScope, ScreenplayLayoutProfile,
    SemanticUnit, StyleProfile,
};
use crate::Screenplay;

const DEFAULT_LINES_PER_PAGE: f32 = 54.0;
const PAGE_NUMBER_OFFSET_FROM_ACTION_EDGE: usize = 5;
const PAGE_NUMBER_GAP_WIDTH: usize = 4;
const PAGE_DELIMITER_LEADING_SPACE_WIDTH: usize = 4;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRenderOptions {
    pub paginated: bool,
    pub line_numbers: bool,
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            paginated: false,
            line_numbers: false,
        }
    }
}

pub fn render(screenplay: &Screenplay, options: &TextRenderOptions) -> String {
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

    if options.paginated {
        let actual =
            PaginatedScreenplay::paginate(semantic.clone(), config.clone(), style_profile, scope);
        let layout_pages = nonempty_layout_pages(&blocks, &config.geometry, config.lines_per_page);
        render_paginated_text(&actual.pages, &layout_pages, &config.geometry, options)
    } else {
        render_unpaginated_text(&blocks, &config.geometry, options)
    }
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

fn render_paginated_text(
    pages: &[Page],
    layout_pages: &[paginator::Page<'_>],
    geometry: &LayoutGeometry,
    options: &TextRenderOptions,
) -> String {
    let mut rendered_pages = Vec::new();

    for (page, layout_page) in pages.iter().zip(layout_pages.iter()) {
        let mut lines = Vec::new();

        if let Some(header) = render_page_header(page, geometry) {
            lines.push(RenderedTextLine {
                text: String::new(),
                counted: false,
            });
            lines.push(RenderedTextLine {
                text: header,
                counted: false,
            });
            lines.push(RenderedTextLine {
                text: String::new(),
                counted: false,
            });
            lines.push(RenderedTextLine {
                text: String::new(),
                counted: false,
            });
        }

        lines.extend(render_layout_page_lines(layout_page, geometry));
        rendered_pages.push(render_text_lines(&lines, options.line_numbers));
    }

    rendered_pages.join("\n\n")
}

fn render_page_header(page: &Page, geometry: &LayoutGeometry) -> Option<String> {
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
    let indent = page_number_indent_spaces(geometry);
    Some(format!(
        "{}{}.",
        alternating_dash_prefix(indent),
        display_number
    ))
}

fn page_number_indent_spaces(geometry: &LayoutGeometry) -> usize {
    calculate_element_width(geometry, ElementType::Action)
        .saturating_sub(PAGE_NUMBER_OFFSET_FROM_ACTION_EDGE)
}

fn alternating_dash_prefix(width: usize) -> String {
    let dash_width = width.saturating_sub(PAGE_NUMBER_GAP_WIDTH);
    let repeated = "- ".repeat((dash_width + 1) / 2);
    let mut prefix = repeated.chars().take(dash_width).collect::<String>();
    let leading_blank = PAGE_DELIMITER_LEADING_SPACE_WIDTH.min(prefix.len());
    prefix.replace_range(0..leading_blank, &" ".repeat(leading_blank));
    prefix.push_str(&" ".repeat(width - dash_width));
    prefix
}

fn render_unpaginated_text(
    blocks: &[LayoutBlock<'_>],
    geometry: &LayoutGeometry,
    options: &TextRenderOptions,
) -> String {
    let lines = blocks
        .iter()
        .filter(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        .flat_map(|block| render_continuous_block_lines(block, geometry))
        .collect::<Vec<_>>();

    render_text_lines(&lines, options.line_numbers)
}

fn render_continuous_block_lines(
    block: &LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> Vec<RenderedTextLine> {
    let mut lines = Vec::new();
    for _ in 0..(block.spacing_above.round() as usize) {
        lines.push(RenderedTextLine {
            text: String::new(),
            counted: true,
        });
    }
    lines.extend(render_layout_block_lines(block, geometry));
    lines
}

fn render_layout_page_lines(
    layout_page: &paginator::Page<'_>,
    geometry: &LayoutGeometry,
) -> Vec<RenderedTextLine> {
    let mut lines = Vec::new();

    for block in &layout_page.blocks {
        for _ in 0..(block.spacing_above.round() as usize) {
            lines.push(RenderedTextLine {
                text: String::new(),
                counted: true,
            });
        }
        lines.extend(render_layout_block_lines(block, geometry));
    }

    lines
}

fn render_text_lines(lines: &[RenderedTextLine], line_numbers: bool) -> String {
    if !line_numbers {
        return lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
    }

    let total_counted = lines.iter().filter(|line| line.counted).count();
    let width = total_counted.max(1).to_string().len().max(2);
    let blank_prefix = " ".repeat(width + 2);
    let mut next_line_number = 1usize;
    let mut out = Vec::with_capacity(lines.len());

    for line in lines {
        if line.counted {
            out.push(format!(
                "{:0width$}: {}",
                next_line_number,
                line.text,
                width = width
            ));
            next_line_number += 1;
        } else if line.text.is_empty() {
            out.push(String::new());
        } else {
            out.push(format!("{blank_prefix}{}", line.text));
        }
    }

    out.join("\n")
}

fn render_layout_block_lines(
    block: &LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> Vec<RenderedTextLine> {
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
            let text = match block.fragment {
                crate::pagination::Fragment::ContinuedToNext => plan.top_text.clone(),
                crate::pagination::Fragment::ContinuedFromPrev => plan.bottom_text.clone(),
                crate::pagination::Fragment::ContinuedFromPrevAndToNext => plan.top_text.clone(),
                crate::pagination::Fragment::Whole => flow.text.clone(),
            };
            let element_type = ElementType::from_flow_kind(&flow.kind);

            return counted_rendered_lines(
                render_indented_lines(&text, element_type, geometry)
                    .into_iter()
                    .map(|text| RenderedElementLine { text, element_type })
                    .collect(),
                geometry,
            );
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

    counted_rendered_lines(lines, geometry)
}

fn render_dialogue_fragment_lines(
    dialogue: &crate::pagination::DialogueUnit,
    fragment: &crate::pagination::Fragment,
    split_plan: Option<&crate::pagination::dialogue_split::DialogueSplitPlan>,
    content_lines: f32,
    geometry: &LayoutGeometry,
) -> Vec<RenderedTextLine> {
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
                        counted_rendered_lines(
                            render_indented_lines(&part.top_text, element_type, geometry)
                                .into_iter()
                                .map(|text| RenderedElementLine { text, element_type })
                                .collect(),
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
                    .map(|text| RenderedTextLine {
                        text,
                        counted: false,
                    })
                    .collect::<Vec<_>>();
                lines.extend(counted_rendered_lines(
                    plan.parts
                        .iter()
                        .zip(dialogue.parts.iter())
                        .flat_map(|(part, dialogue_part)| {
                            let element_type =
                                ElementType::from_dialogue_part_kind(&dialogue_part.kind);
                            render_indented_lines(&part.bottom_text, element_type, geometry)
                                .into_iter()
                                .map(move |text| RenderedElementLine { text, element_type })
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
        crate::pagination::Fragment::Whole => counted_rendered_lines(all_lines, geometry),
        crate::pagination::Fragment::ContinuedToNext => {
            let lines = take_rendered_lines_from_top_by_height(&all_lines, content_lines, geometry);
            let mut lines = counted_rendered_lines(lines, geometry);
            lines.push(render_more_marker_line(geometry));
            lines
        }
        crate::pagination::Fragment::ContinuedFromPrev => continuation_prefix
            .into_iter()
            .map(|text| RenderedTextLine {
                text,
                counted: false,
            })
            .chain(counted_rendered_lines(
                take_rendered_lines_from_bottom_by_height(&all_lines, content_lines, geometry),
                geometry,
            ))
            .collect(),
        crate::pagination::Fragment::ContinuedFromPrevAndToNext => {
            let mut lines = continuation_prefix
                .into_iter()
                .map(|text| RenderedTextLine {
                    text,
                    counted: false,
                })
                .chain(counted_rendered_lines(
                    take_rendered_lines_from_top_by_height(&all_lines, content_lines, geometry),
                    geometry,
                ))
                .collect::<Vec<_>>();
            lines.push(render_more_marker_line(geometry));
            lines
        }
    }
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

fn render_more_marker_line(geometry: &LayoutGeometry) -> RenderedTextLine {
    RenderedTextLine {
        text: render_indented_lines("(MORE)", ElementType::Character, geometry)
            .into_iter()
            .next()
            .unwrap_or_else(|| "(MORE)".to_string()),
        counted: false,
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
            render_indented_lines(&flow.text, element_type, geometry)
                .into_iter()
                .map(|text| RenderedElementLine { text, element_type })
                .collect()
        }
        SemanticUnit::Lyric(lyric) => render_indented_lines(&lyric.text, ElementType::Lyric, geometry)
            .into_iter()
            .map(|text| RenderedElementLine {
                text,
                element_type: ElementType::Lyric,
            })
            .collect(),
        SemanticUnit::Dialogue(dialogue) => dialogue
            .parts
            .iter()
            .flat_map(|part| {
                let element_type = ElementType::from_dialogue_part_kind(&part.kind);
                render_indented_lines(&part.text, element_type, geometry)
                    .into_iter()
                    .map(move |text| RenderedElementLine { text, element_type })
            })
            .collect(),
        SemanticUnit::DualDialogue(dual) => {
            let left_lines = dual
                .sides
                .iter()
                .find(|side| side.side == 1)
                .map(|side| {
                    render_dual_dialogue_side_lines(
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
                    render_dual_dialogue_side_lines(
                        &side.dialogue,
                        ElementType::DualDialogueRight,
                        geometry,
                    )
                })
                .unwrap_or_default();

            let right_indent = indent_spaces_for_element_type(ElementType::DualDialogueRight, geometry);
            let mut lines = Vec::new();
            for index in 0..left_lines.len().max(right_lines.len()) {
                let left = left_lines.get(index).cloned().unwrap_or_default();
                let right = right_lines.get(index).cloned().unwrap_or_default();

                if right.is_empty() {
                    lines.push(RenderedElementLine {
                        text: left,
                        element_type: ElementType::Dialogue,
                    });
                } else if left.is_empty() {
                    lines.push(RenderedElementLine {
                        text: format!("{:width$}{}", "", right, width = right_indent),
                        element_type: ElementType::Dialogue,
                    });
                } else {
                    lines.push(RenderedElementLine {
                        text: format!("{left:width$}{right}", width = right_indent),
                        element_type: ElementType::Dialogue,
                    });
                }
            }
            lines
        }
    }
}

fn render_dual_dialogue_side_lines(
    dialogue: &crate::pagination::DialogueUnit,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> Vec<String> {
    let config = wrapping::WrapConfig::from_geometry(geometry, element_type);
    dialogue
        .parts
        .iter()
        .flat_map(|part| wrapping::wrap_text_for_element(&part.text, &config))
        .collect()
}

fn render_indented_lines(
    text: &str,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> Vec<String> {
    let config = wrapping::WrapConfig::from_geometry(geometry, element_type);
    let indent = " ".repeat(indent_spaces_for_element_type(element_type, geometry));
    wrapped_visual_lines(element_type, text, &config)
        .into_iter()
        .map(|line| format!("{indent}{line}"))
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

fn counted_rendered_lines(
    lines: Vec<RenderedElementLine>,
    geometry: &LayoutGeometry,
) -> Vec<RenderedTextLine> {
    let mut rendered = Vec::new();

    for line in lines {
        if uses_double_spaced_rows(line.element_type, geometry) {
            rendered.push(RenderedTextLine {
                text: String::new(),
                counted: true,
            });
        }
        rendered.push(RenderedTextLine {
            text: line.text,
            counted: true,
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
}

#[derive(Clone)]
struct RenderedTextLine {
    text: String,
    counted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blank_attributes, p, Attributes, Element, Metadata};

    #[test]
    fn paginated_text_uses_clean_page_headers() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![
                Element::Action(p("FIRST PAGE"), blank_attributes()),
                Element::Action(
                    p("SECOND PAGE"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let output = render(
            &screenplay,
            &TextRenderOptions {
                paginated: true,
                line_numbers: false,
            },
        );

        assert!(output.lines().next().is_some_and(|line| line.trim() == "FIRST PAGE"));
        assert!(output.contains("SECOND PAGE"));
        let page_two_header = output
            .lines()
            .find(|line| line.trim().ends_with("2."))
            .unwrap_or_default();
        assert_eq!(page_two_header.find("2.").unwrap_or_default(), 56);
        assert!(page_two_header.starts_with("    - -"));
        assert!(!output.contains("=== PAGE"));
    }

    #[test]
    fn text_output_line_numbers_default_off() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(p("HELLO"), blank_attributes())],
        };

        let output = render(&screenplay, &TextRenderOptions::default());

        assert!(output.contains("HELLO"));
        assert!(!output.contains("01:"));
    }

    #[test]
    fn text_output_can_render_line_numbers() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(p("HELLO"), blank_attributes())],
        };

        let output = render(
            &screenplay,
            &TextRenderOptions {
                paginated: false,
                line_numbers: true,
            },
        );

        assert!(output.contains("01:"));
        assert!(output.contains("HELLO"));
    }

    #[test]
    fn paginated_text_uses_body_page_numbers_when_title_metadata_exists() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        let screenplay = Screenplay {
            metadata,
            elements: vec![
                Element::Action(p("BODY PAGE ONE"), blank_attributes()),
                Element::Action(
                    p("BODY PAGE TWO"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let output = render(
            &screenplay,
            &TextRenderOptions {
                paginated: true,
                line_numbers: false,
            },
        );

        assert!(output.lines().next().is_some_and(|line| line.trim() == "BODY PAGE ONE"));
        let page_two_header = output
            .lines()
            .find(|line| line.trim().ends_with("2."))
            .unwrap_or_default();
        assert_eq!(page_two_header.find("2.").unwrap_or_default(), 56);
        assert!(page_two_header.starts_with("    - -"));
    }

    #[test]
    fn page_header_starts_a_few_columns_before_action_wrap_limit() {
        let page = Page {
            metadata: crate::pagination::PageMetadata {
                index: 1,
                number: 2,
                kind: PageKind::Body,
                body_page_number: Some(2),
                title_page_number: None,
            },
            items: Vec::new(),
            blocks: Vec::new(),
        };

        let header = render_page_header(&page, &LayoutGeometry::default()).unwrap_or_default();
        let number_start = header.find("2.").unwrap_or_default();

        assert_eq!(number_start, 56);
        assert!(header.starts_with("    - -"));
        assert!(header.ends_with("2."));
    }
}
