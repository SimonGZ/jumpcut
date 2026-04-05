#![allow(dead_code)]

use crate::pagination::margin::{dual_dialogue_character_left_indent, LayoutGeometry};
use crate::pagination::wrapping::ElementType;
use crate::pagination::ScreenplayLayoutProfile;
use crate::title_page::{
    plain_title_uses_all_caps, TitlePage, TitlePageBlockKind, TitlePageRegion,
};
use crate::visual_lines::{
    display_page_number, render_paginated_visual_pages_with_options, VisualDualLine,
    VisualDualSide, VisualFragment, VisualRenderOptions,
};
use crate::{ElementText, Screenplay};
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use std::collections::{BTreeMap, BTreeSet};
use ttf_parser::Face;

const LETTER_WIDTH: f32 = 612.0;
const LETTER_HEIGHT: f32 = 792.0;
const BODY_LINES_PER_PAGE: f32 = 54.0;
const PAGE_TOP_BOTTOM_MARGIN_INCHES: f32 = 1.0;
const BODY_TEXT_FONT_SIZE: f32 = 12.0;
const PAGE_NUMBER_LEFT: f32 = 7.0625 * 72.0;
const PAGE_NUMBER_BASELINE_Y: f32 = 747.0;
const TITLE_FONT_SIZE: f32 = 12.0;
const TITLE_META_FONT_SIZE: f32 = 12.0;
const TITLE_BLOCK_LINE_STEP: f32 = 12.0;
const TITLE_TITLE_TOP: f32 = 495.0;
const TITLE_META_TOP: f32 = 447.0;
const TITLE_BOTTOM_TOP: f32 = 135.0;
const PAGE_SIDE_MARGIN: f32 = 72.0;
const FONT_REGULAR_NAME: Name<'static> = Name(b"F1");
const FONT_BOLD_NAME: Name<'static> = Name(b"F2");
const FONT_ITALIC_NAME: Name<'static> = Name(b"F3");
const FONT_BOLD_ITALIC_NAME: Name<'static> = Name(b"F4");
const BODY_TEXT_CELL_WIDTH: f32 = 7.0;
const UNDERLINE_LINE_WIDTH: f32 = 0.75;
const UNDERLINE_Y_OFFSET: f32 = 1.5;
const COURIER_PRIME_REGULAR_BYTES: &[u8] =
    include_bytes!("templates/fonts/CourierPrime-Regular.ttf");
const COURIER_PRIME_BOLD_BYTES: &[u8] = include_bytes!("templates/fonts/CourierPrime-Bold.ttf");
const COURIER_PRIME_ITALIC_BYTES: &[u8] =
    include_bytes!("templates/fonts/CourierPrime-Italic.ttf");
const COURIER_PRIME_BOLD_ITALIC_BYTES: &[u8] =
    include_bytes!("templates/fonts/CourierPrime-BoldItalic.ttf");
const IDENTITY_H: Name<'static> = Name(b"Identity-H");
const ADOBE_IDENTITY: SystemInfo<'static> = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdfRenderOptions {
    pub render_continueds: bool,
}

impl Default for PdfRenderOptions {
    fn default() -> Self {
        Self {
            render_continueds: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfRenderDocument {
    pub title_page: Option<PdfTitlePage>,
    pub body_pages: Vec<PdfRenderPage>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitlePage {
    pub blocks: Vec<PdfTitleBlock>,
    pub plain_title_all_caps: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitleBlock {
    pub kind: PdfTitleBlockKind,
    pub region: PdfTitleBlockRegion,
    pub lines: Vec<ElementText>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfTitleBlockKind {
    Title,
    Credit,
    Author,
    Source,
    Contact,
    Draft,
    DraftDate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfTitleBlockRegion {
    CenterTitle,
    CenterMeta,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderPage {
    pub page_number: u32,
    pub display_page_number: Option<u32>,
    pub lines: Vec<PdfRenderLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderLine {
    pub text: String,
    pub counted: bool,
    pub centered: bool,
    pub kind: Option<PdfLineKind>,
    pub fragments: Vec<PdfRenderFragment>,
    pub dual: Option<PdfRenderDualLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderFragment {
    pub text: String,
    pub styles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderDualLine {
    pub left: Option<PdfRenderDualSide>,
    pub right: Option<PdfRenderDualSide>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderDualSide {
    pub text: String,
    pub kind: PdfLineKind,
    pub fragments: Vec<PdfRenderFragment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfLineKind {
    Action,
    ColdOpening,
    NewAct,
    EndOfAct,
    SceneHeading,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Lyric,
    DualDialogueLeft,
    DualDialogueRight,
    DualDialogueCharacterLeft,
    DualDialogueCharacterRight,
    DualDialogueParentheticalLeft,
    DualDialogueParentheticalRight,
}

struct EmbeddedFont {
    resource_name: Name<'static>,
    base_font: Name<'static>,
    family_name: &'static [u8],
    font_bytes: &'static [u8],
    cid_by_char: BTreeMap<char, u16>,
    font_descriptor: EmbeddedFontDescriptor,
    cid_widths: Vec<f32>,
    cid_to_gid_map: Vec<u8>,
    to_unicode_cmap: Vec<u8>,
}

struct EmbeddedFonts {
    regular: EmbeddedFont,
    bold: EmbeddedFont,
    italic: EmbeddedFont,
    bold_italic: EmbeddedFont,
}

#[derive(Clone, Copy)]
struct UnderlineSegment {
    start_x: f32,
    end_x: f32,
    y: f32,
}

#[derive(Clone, Copy, Default)]
struct StyleFlags {
    bold: bool,
    italic: bool,
    underline: bool,
}

#[derive(Clone)]
struct ResolvedRun {
    text: String,
    styles: StyleFlags,
}

#[derive(Clone, Copy)]
struct FontObjectIds {
    type0_font_id: Ref,
    cid_font_id: Ref,
    font_descriptor_id: Ref,
    font_file_id: Ref,
    to_unicode_id: Ref,
    cid_to_gid_map_id: Ref,
}

struct EmbeddedFontDescriptor {
    ascent: f32,
    descent: f32,
    cap_height: f32,
    x_height: f32,
    bbox: Rect,
    italic_angle: f32,
    avg_width: f32,
    max_width: f32,
    missing_width: f32,
    flags: FontFlags,
}

pub(crate) fn build_render_document(
    screenplay: &Screenplay,
    options: PdfRenderOptions,
) -> PdfRenderDocument {
    let title_page =
        TitlePage::from_metadata(&screenplay.metadata).map(|title_page| PdfTitlePage {
            plain_title_all_caps: plain_title_uses_all_caps(&screenplay.metadata),
            blocks: title_page
                .blocks
                .into_iter()
                .map(|block| PdfTitleBlock {
                    kind: block.kind.into(),
                    region: block.region.into(),
                    lines: block.lines,
                })
                .collect(),
        });

    let body_pages = render_paginated_visual_pages_with_options(
        screenplay,
        VisualRenderOptions {
            render_continueds: options.render_continueds,
        },
    )
        .into_iter()
        .map(|page| PdfRenderPage {
            page_number: page.page.metadata.number,
            display_page_number: display_page_number(&page.page),
            lines: page
                .lines
                .into_iter()
                .map(|line| PdfRenderLine {
                    text: line.text,
                    counted: line.counted,
                    centered: line.centered,
                    kind: line.element_type.map(Into::into),
                    fragments: line
                        .fragments
                        .into_iter()
                        .map(PdfRenderFragment::from)
                        .collect(),
                    dual: line.dual.map(Into::into),
                })
                .collect(),
        })
        .collect();

    PdfRenderDocument {
        title_page,
        body_pages,
    }
}

pub(crate) fn render(screenplay: &Screenplay) -> Vec<u8> {
    render_with_options(screenplay, PdfRenderOptions::default())
}

pub(crate) fn render_with_options(screenplay: &Screenplay, options: PdfRenderOptions) -> Vec<u8> {
    let document = build_render_document(screenplay, options);
    let geometry =
        ScreenplayLayoutProfile::from_metadata(&screenplay.metadata).to_pagination_geometry();
    let fonts = EmbeddedFonts::new(&document);
    let body_page_count = document.body_pages.len() as i32;
    let page_count = body_page_count + i32::from(document.title_page.is_some());

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let regular_font_ids = font_object_ids(3);
    let bold_font_ids = font_object_ids(9);
    let italic_font_ids = font_object_ids(15);
    let bold_italic_font_ids = font_object_ids(21);
    let page_ids = (0..page_count)
        .map(|index| Ref::new(27 + index))
        .collect::<Vec<_>>();
    let content_ids = (0..page_count)
        .map(|index| Ref::new(27 + page_count + index))
        .collect::<Vec<_>>();

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_count);
    write_embedded_font_objects(&mut pdf, &fonts.regular, regular_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.bold, bold_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.italic, italic_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.bold_italic, bold_italic_font_ids);

    for (index, page_id) in page_ids.iter().copied().enumerate() {
        let mut page = pdf.page(page_id);
        page.parent(page_tree_id)
            .media_box(Rect::new(0.0, 0.0, LETTER_WIDTH, LETTER_HEIGHT))
            .contents(content_ids[index]);
        page.resources()
            .fonts()
            .pair(fonts.regular.resource_name, regular_font_ids.type0_font_id)
            .pair(fonts.bold.resource_name, bold_font_ids.type0_font_id)
            .pair(fonts.italic.resource_name, italic_font_ids.type0_font_id)
            .pair(
                fonts.bold_italic.resource_name,
                bold_italic_font_ids.type0_font_id,
            );
        page.finish();
    }

    let mut content_index = 0usize;
    if let Some(title_page) = &document.title_page {
        pdf.stream(
            content_ids[content_index],
            &render_title_page_content(title_page, &fonts),
        );
        content_index += 1;
    }

    for body_page in &document.body_pages {
        pdf.stream(
            content_ids[content_index],
            &render_body_page_content(body_page, &geometry, &fonts),
        );
        content_index += 1;
    }

    pdf.finish()
}

fn render_body_page_content(
    page: &PdfRenderPage,
    geometry: &LayoutGeometry,
    fonts: &EmbeddedFonts,
) -> Vec<u8> {
    let mut content = Content::new();
    let mut underlines = Vec::new();
    content.begin_text();
    content.set_font(FONT_REGULAR_NAME, BODY_TEXT_FONT_SIZE);
    let line_step = body_line_step_points();

    if let Some(display_page_number) = page.display_page_number {
        let page_number = format!("{display_page_number}.");
        render_fixed_cell_runs(
            &mut content,
            fonts,
            &[ResolvedRun {
                text: page_number,
                styles: StyleFlags::default(),
            }],
            page_number_x(display_page_number),
            page_number_y(),
            BODY_TEXT_FONT_SIZE,
            &mut underlines,
        );
    }

    let body_top = first_body_line_y_for_page(page);
    for (index, line) in page.lines.iter().enumerate() {
        if line.text.is_empty() {
            continue;
        }

        let line_y = body_top - (index as f32 * line_step);
        if let Some(dual) = &line.dual {
            render_dual_body_line(&mut content, dual, geometry, fonts, line_y, &mut underlines);
            continue;
        }
        let fragments = displayed_body_fragments(line, geometry);
        let defaults = default_body_line_styles(line.kind);
        render_fixed_cell_runs(
            &mut content,
            fonts,
            &resolve_runs(&fragments, defaults),
            body_line_left(line, geometry),
            line_y,
            BODY_TEXT_FONT_SIZE,
            &mut underlines,
        );
    }

    content.end_text();
    render_underlines(&mut content, &underlines);
    content.finish().to_vec()
}

fn render_dual_body_line(
    content: &mut Content,
    dual: &PdfRenderDualLine,
    geometry: &LayoutGeometry,
    fonts: &EmbeddedFonts,
    line_y: f32,
    underlines: &mut Vec<UnderlineSegment>,
) {
    if let Some(left) = &dual.left {
        render_fixed_cell_runs(
            content,
            fonts,
            &resolve_runs(
                &left.fragments,
                default_body_line_styles(Some(left.kind)),
            ),
            dual_side_left(left, geometry),
            line_y,
            BODY_TEXT_FONT_SIZE,
            underlines,
        );
    }

    if let Some(right) = &dual.right {
        render_fixed_cell_runs(
            content,
            fonts,
            &resolve_runs(
                &right.fragments,
                default_body_line_styles(Some(right.kind)),
            ),
            dual_side_left(right, geometry),
            line_y,
            BODY_TEXT_FONT_SIZE,
            underlines,
        );
    }
}

fn render_fixed_cell_runs(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    runs: &[ResolvedRun],
    line_left: f32,
    line_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
) {
    let mut cell_index = 0usize;

    for run in runs {
        let font = fonts.for_styles(run.styles);
        content.set_font(font.resource_name, font_size);
        let run_start_cell = cell_index;

        for character in run.text.chars() {
            if character != ' ' {
                content.set_text_matrix([
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    line_left + (cell_index as f32 * BODY_TEXT_CELL_WIDTH),
                    line_y,
                ]);
                let encoded_character = font.encode_char(character);
                content.show(Str(&encoded_character));
            }
            cell_index += 1;
        }

        if run.styles.underline {
            let underline_cell_count = run
                .text
                .trim_end_matches(' ')
                .chars()
                .count();
            let underline_end_cell = run_start_cell + underline_cell_count;
            if underline_end_cell > run_start_cell {
                underlines.push(UnderlineSegment {
                    start_x: line_left + (run_start_cell as f32 * BODY_TEXT_CELL_WIDTH),
                    end_x: line_left + (underline_end_cell as f32 * BODY_TEXT_CELL_WIDTH),
                    y: line_y,
                });
            }
        }
    }
}

fn body_line_step_points() -> f32 {
    let usable_page_height = LETTER_HEIGHT - (2.0 * PAGE_TOP_BOTTOM_MARGIN_INCHES * 72.0);
    usable_page_height / BODY_LINES_PER_PAGE
}

fn first_body_line_y() -> f32 {
    711.0
}

fn first_body_line_y_for_page(page: &PdfRenderPage) -> f32 {
    let mut y = first_body_line_y();
    if page_starts_with_split_contd_character(page) {
        y += body_line_step_points();
    }
    y
}

fn page_starts_with_split_contd_character(page: &PdfRenderPage) -> bool {
    page.lines
        .iter()
        .find(|line| !line.text.is_empty())
        .is_some_and(|line| !line.counted && matches!(line.kind, Some(PdfLineKind::Character)))
}

fn page_number_y() -> f32 {
    PAGE_NUMBER_BASELINE_Y
}

fn page_number_x(display_page_number: u32) -> f32 {
    let extra_digits = display_page_number.to_string().len().saturating_sub(1) as f32;
    PAGE_NUMBER_LEFT - (extra_digits * BODY_TEXT_CELL_WIDTH)
}

fn render_title_page_content(title_page: &PdfTitlePage, fonts: &EmbeddedFonts) -> Vec<u8> {
    let mut content = Content::new();
    let mut underlines = Vec::new();
    content.begin_text();
    content.set_font(FONT_REGULAR_NAME, TITLE_META_FONT_SIZE);

    render_title_page_region(
        &mut content,
        title_page,
        fonts,
        PdfTitleBlockRegion::CenterTitle,
        TITLE_TITLE_TOP,
        TITLE_FONT_SIZE,
        &mut underlines,
    );
    render_title_page_region(
        &mut content,
        title_page,
        fonts,
        PdfTitleBlockRegion::CenterMeta,
        TITLE_META_TOP,
        TITLE_META_FONT_SIZE,
        &mut underlines,
    );
    render_title_page_bottom_regions(&mut content, title_page, fonts, &mut underlines);

    content.end_text();
    render_underlines(&mut content, &underlines);
    content.finish().to_vec()
}

fn render_title_page_region(
    content: &mut Content,
    title_page: &PdfTitlePage,
    fonts: &EmbeddedFonts,
    region: PdfTitleBlockRegion,
    top_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
) {
    let mut line_index = 0usize;

    for block in title_page
        .blocks
        .iter()
        .filter(|block| block.region == region)
    {
        for line in &block.lines {
            let text = line.plain_text();
            let y = top_y - (line_index as f32 * TITLE_BLOCK_LINE_STEP);
            render_fixed_cell_runs(
                content,
                fonts,
                &resolve_runs(
                    &title_page_fragments(title_page, block.kind, line),
                    default_title_page_styles(block.kind, line),
                ),
                title_page_line_left(&text, region),
                y,
                font_size,
                underlines,
            );
            line_index += 1;
        }
        line_index += title_page_block_gap_after(block.kind);
    }
}

fn render_title_page_bottom_regions(
    content: &mut Content,
    title_page: &PdfTitlePage,
    fonts: &EmbeddedFonts,
    underlines: &mut Vec<UnderlineSegment>,
) {
    let left_lines = title_page
        .blocks
        .iter()
        .filter(|block| block.region == PdfTitleBlockRegion::BottomLeft)
        .flat_map(|block| block.lines.iter().map(move |line| (block.kind, line)))
        .collect::<Vec<_>>();
    let right_lines = title_page
        .blocks
        .iter()
        .filter(|block| block.region == PdfTitleBlockRegion::BottomRight)
        .flat_map(|block| block.lines.iter().map(move |line| (block.kind, line)))
        .collect::<Vec<_>>();

    let max_lines = left_lines.len().max(right_lines.len());
    render_title_page_bottom_region_lines(
        content,
        fonts,
        PdfTitleBlockRegion::BottomLeft,
        &left_lines,
        max_lines,
        underlines,
    );
    render_title_page_bottom_region_lines(
        content,
        fonts,
        PdfTitleBlockRegion::BottomRight,
        &right_lines,
        max_lines,
        underlines,
    );
}

fn render_title_page_bottom_region_lines(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    region: PdfTitleBlockRegion,
    lines: &[(PdfTitleBlockKind, &ElementText)],
    max_lines: usize,
    underlines: &mut Vec<UnderlineSegment>,
) {
    let line_offset = max_lines.saturating_sub(lines.len()) as f32 * TITLE_BLOCK_LINE_STEP;

    for (line_index, (kind, line)) in lines.iter().enumerate() {
        let text = line.plain_text();
        let y = TITLE_BOTTOM_TOP - line_offset - (line_index as f32 * TITLE_BLOCK_LINE_STEP);
        render_fixed_cell_runs(
            content,
            fonts,
            &resolve_runs(
                &title_page_fragments_for_kind(*kind, line),
                default_title_page_styles(*kind, line),
            ),
            title_page_bottom_line_left(&text, region),
            y,
            TITLE_META_FONT_SIZE,
            underlines,
        );
    }
}

fn title_page_bottom_line_left(text: &str, region: PdfTitleBlockRegion) -> f32 {
    match region {
        PdfTitleBlockRegion::BottomRight => title_page_bottom_right_left(text),
        _ => title_page_line_left(text, region),
    }
}

fn title_page_bottom_right_left(text: &str) -> f32 {
    let right_edge_points = LETTER_WIDTH - PAGE_SIDE_MARGIN;
    let text_width_points = (text.chars().count() as f32 * 7.1) + 2.0;
    right_edge_points - text_width_points
}

fn title_page_block_gap_after(kind: PdfTitleBlockKind) -> usize {
    match kind {
        PdfTitleBlockKind::Author => 4,
        _ => 1,
    }
}

fn write_embedded_font_objects(pdf: &mut Pdf, font: &EmbeddedFont, ids: FontObjectIds) {
    pdf.type0_font(ids.type0_font_id)
        .base_font(font.base_font)
        .encoding_predefined(IDENTITY_H)
        .descendant_font(ids.cid_font_id)
        .to_unicode(ids.to_unicode_id);

    let mut cid_font = pdf.cid_font(ids.cid_font_id);
    cid_font
        .subtype(CidFontType::Type2)
        .base_font(font.base_font)
        .system_info(ADOBE_IDENTITY)
        .font_descriptor(ids.font_descriptor_id)
        .default_width(font.font_descriptor.missing_width);
    cid_font
        .widths()
        .consecutive(1, font.cid_widths.iter().copied())
        .finish();
    cid_font.cid_to_gid_map_stream(ids.cid_to_gid_map_id);
    cid_font.finish();

    pdf.font_descriptor(ids.font_descriptor_id)
        .name(font.base_font)
        .family(Str(font.family_name))
        .flags(FontFlags::from_bits_retain(
            font.font_descriptor.flags.bits(),
        ))
        .bbox(font.font_descriptor.bbox)
        .italic_angle(font.font_descriptor.italic_angle)
        .ascent(font.font_descriptor.ascent)
        .descent(font.font_descriptor.descent)
        .cap_height(font.font_descriptor.cap_height)
        .x_height(font.font_descriptor.x_height)
        .stem_v(80.0)
        .avg_width(font.font_descriptor.avg_width)
        .max_width(font.font_descriptor.max_width)
        .missing_width(font.font_descriptor.missing_width)
        .font_file2(ids.font_file_id);

    let mut font_file_stream = pdf.stream(ids.font_file_id, font.font_bytes);
    font_file_stream.pair(Name(b"Length1"), font.font_bytes.len() as i32);
    font_file_stream.finish();

    pdf.stream(ids.to_unicode_id, &font.to_unicode_cmap);
    pdf.stream(ids.cid_to_gid_map_id, &font.cid_to_gid_map);
}

fn title_page_line_left(text: &str, region: PdfTitleBlockRegion) -> f32 {
    let width = text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;

    match region {
        PdfTitleBlockRegion::CenterTitle | PdfTitleBlockRegion::CenterMeta => {
            ((LETTER_WIDTH - width) / 2.0).max(PAGE_SIDE_MARGIN)
        }
        PdfTitleBlockRegion::BottomLeft => PAGE_SIDE_MARGIN,
        PdfTitleBlockRegion::BottomRight => {
            (LETTER_WIDTH - PAGE_SIDE_MARGIN - width).max(PAGE_SIDE_MARGIN)
        }
    }
}

fn body_line_left(line: &PdfRenderLine, geometry: &LayoutGeometry) -> f32 {
    if !line.centered {
        if matches!(line.kind, Some(PdfLineKind::Transition))
            && geometry.transition_alignment == crate::pagination::Alignment::Right
        {
            let text = rendered_body_line_text(line, geometry);
            let char_width = BODY_TEXT_CELL_WIDTH;
            let rendered_width = text.chars().count() as f32 * char_width;
            return (geometry.transition_right * 72.0) - rendered_width;
        }
        return line_kind_left(line.kind, geometry)
            - parenthetical_hang_offset_points(line.kind, rendered_body_line_text(line, geometry));
    }

    let action_left = geometry.action_left * 72.0;
    let action_right = geometry.action_right * 72.0;
    let rendered_width = line.text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;
    let available_width = action_right - action_left;

    action_left + ((available_width - rendered_width) / 2.0).max(0.0)
}

fn rendered_body_line_text<'a>(line: &'a PdfRenderLine, geometry: &LayoutGeometry) -> &'a str {
    if line.centered {
        return &line.text;
    }

    let Some(kind) = line.kind else {
        return &line.text;
    };
    let indent = " ".repeat(body_line_indent_spaces(kind, &line.text, geometry));
    line.text.strip_prefix(&indent).unwrap_or(&line.text)
}

fn line_kind_left(kind: Option<PdfLineKind>, geometry: &LayoutGeometry) -> f32 {
    kind.map(|kind| element_left_inches(kind, geometry) * 72.0)
        .unwrap_or(geometry.action_left * 72.0)
}

fn dual_side_left(side: &PdfRenderDualSide, geometry: &LayoutGeometry) -> f32 {
    let left = match side.kind {
        PdfLineKind::DualDialogueCharacterLeft => {
            dual_dialogue_character_left_indent(&side.text, 1) * 72.0
        }
        PdfLineKind::DualDialogueCharacterRight => {
            dual_dialogue_character_left_indent(&side.text, 2) * 72.0
        }
        _ => line_kind_left(Some(side.kind), geometry),
    };
    left - parenthetical_hang_offset_points(Some(side.kind), &side.text)
}

fn synthetic_indent_spaces(kind: PdfLineKind, geometry: &LayoutGeometry) -> usize {
    let left = element_left_inches(kind, geometry);
    ((left - geometry.action_left) * geometry.cpi).floor() as usize
}

fn body_line_indent_spaces(kind: PdfLineKind, text: &str, geometry: &LayoutGeometry) -> usize {
    let base = synthetic_indent_spaces(kind, geometry);
    if hangs_opening_parenthesis(Some(kind), text) {
        return base.saturating_sub(1);
    }
    base
}

fn element_left_inches(kind: PdfLineKind, geometry: &LayoutGeometry) -> f32 {
    match kind {
        PdfLineKind::Action | PdfLineKind::SceneHeading => geometry.action_left,
        PdfLineKind::ColdOpening => geometry.cold_opening_left,
        PdfLineKind::NewAct => geometry.new_act_left,
        PdfLineKind::EndOfAct => geometry.end_of_act_left,
        PdfLineKind::Character => geometry.character_left,
        PdfLineKind::Dialogue => geometry.dialogue_left,
        PdfLineKind::Parenthetical => geometry.parenthetical_left,
        PdfLineKind::Transition => geometry.transition_left,
        PdfLineKind::Lyric => geometry.lyric_left,
        PdfLineKind::DualDialogueLeft => geometry.dual_dialogue_left_left,
        PdfLineKind::DualDialogueRight => geometry.dual_dialogue_right_left,
        PdfLineKind::DualDialogueCharacterLeft => geometry.dual_dialogue_left_character_left,
        PdfLineKind::DualDialogueCharacterRight => geometry.dual_dialogue_right_character_left,
        PdfLineKind::DualDialogueParentheticalLeft => {
            geometry.dual_dialogue_left_parenthetical_left
        }
        PdfLineKind::DualDialogueParentheticalRight => {
            geometry.dual_dialogue_right_parenthetical_left
        }
    }
}

fn parenthetical_hang_offset_points(kind: Option<PdfLineKind>, text: &str) -> f32 {
    if hangs_opening_parenthesis(kind, text) {
        return BODY_TEXT_CELL_WIDTH;
    }
    0.0
}

fn hangs_opening_parenthesis(kind: Option<PdfLineKind>, text: &str) -> bool {
    matches!(kind, Some(PdfLineKind::Parenthetical)) && text.trim_start().starts_with('(')
}

impl EmbeddedFonts {
    fn new(document: &PdfRenderDocument) -> Self {
        Self {
            regular: EmbeddedFont::new(
                document,
                FONT_REGULAR_NAME,
                Name(b"CourierPrime-Regular"),
                b"Courier Prime",
                COURIER_PRIME_REGULAR_BYTES,
                Name(b"CourierPrime-Regular-UTF16"),
            ),
            bold: EmbeddedFont::new(
                document,
                FONT_BOLD_NAME,
                Name(b"CourierPrime-Bold"),
                b"Courier Prime",
                COURIER_PRIME_BOLD_BYTES,
                Name(b"CourierPrime-Bold-UTF16"),
            ),
            italic: EmbeddedFont::new(
                document,
                FONT_ITALIC_NAME,
                Name(b"CourierPrime-Italic"),
                b"Courier Prime",
                COURIER_PRIME_ITALIC_BYTES,
                Name(b"CourierPrime-Italic-UTF16"),
            ),
            bold_italic: EmbeddedFont::new(
                document,
                FONT_BOLD_ITALIC_NAME,
                Name(b"CourierPrime-BoldItalic"),
                b"Courier Prime",
                COURIER_PRIME_BOLD_ITALIC_BYTES,
                Name(b"CourierPrime-BoldItalic-UTF16"),
            ),
        }
    }

    fn for_styles(&self, styles: StyleFlags) -> &EmbeddedFont {
        match (styles.bold, styles.italic) {
            (true, true) => &self.bold_italic,
            (true, false) => &self.bold,
            (false, true) => &self.italic,
            (false, false) => &self.regular,
        }
    }
}

impl EmbeddedFont {
    fn new(
        document: &PdfRenderDocument,
        resource_name: Name<'static>,
        base_font: Name<'static>,
        family_name: &'static [u8],
        font_bytes: &'static [u8],
        cmap_name: Name<'static>,
    ) -> Self {
        let chars = collect_document_chars(document);
        let face = Face::parse(font_bytes, 0).expect("Courier Prime TTF should parse");
        let units_per_em = face.units_per_em() as f32;
        let scale = 1000.0 / units_per_em;

        let mut cid_by_char = BTreeMap::new();
        let mut cid_widths = Vec::new();
        let mut cid_to_gid_map = vec![0_u8, 0_u8];
        let mut to_unicode = UnicodeCmap::new(cmap_name, ADOBE_IDENTITY);

        for (index, character) in chars.into_iter().enumerate() {
            let cid = u16::try_from(index + 1).expect("too many distinct characters for CID font");
            let glyph_id = face.glyph_index(character).unwrap_or_else(|| {
                face.glyph_index('?')
                    .expect("Courier Prime should contain a replacement question mark glyph")
            });
            let width = face.glyph_hor_advance(glyph_id).unwrap_or(0);

            cid_by_char.insert(character, cid);
            cid_widths.push(width as f32 * scale);
            cid_to_gid_map.extend_from_slice(&glyph_id.0.to_be_bytes());
            to_unicode.pair(cid, character);
        }

        let bbox = face.global_bounding_box();
        let widths = cid_widths.clone();

        Self {
            resource_name,
            base_font,
            family_name,
            font_bytes,
            cid_by_char,
            font_descriptor: EmbeddedFontDescriptor {
                ascent: face.ascender() as f32 * scale,
                descent: face.descender() as f32 * scale,
                cap_height: face
                    .capital_height()
                    .map(|value| value as f32 * scale)
                    .unwrap_or(face.ascender() as f32 * scale),
                x_height: face
                    .x_height()
                    .map(|value| value as f32 * scale)
                    .unwrap_or(0.0),
                bbox: Rect::new(
                    bbox.x_min as f32 * scale,
                    bbox.y_min as f32 * scale,
                    bbox.x_max as f32 * scale,
                    bbox.y_max as f32 * scale,
                ),
                italic_angle: face.italic_angle(),
                avg_width: average_width(&widths),
                max_width: widths.iter().copied().fold(0.0, f32::max),
                missing_width: widths.first().copied().unwrap_or(600.0),
                flags: font_flags(&face),
            },
            cid_widths,
            cid_to_gid_map,
            to_unicode_cmap: to_unicode.finish().to_vec(),
        }
    }

    fn encode_text(&self, text: &str) -> Vec<u8> {
        let fallback_cid = *self
            .cid_by_char
            .get(&'?')
            .expect("Courier Prime character map should include '?'");
        let mut encoded = Vec::with_capacity(text.chars().count() * 2);
        for character in text.chars() {
            let cid = self
                .cid_by_char
                .get(&character)
                .copied()
                .unwrap_or(fallback_cid);
            encoded.extend_from_slice(&cid.to_be_bytes());
        }
        encoded
    }

    fn encode_char(&self, character: char) -> [u8; 2] {
        let fallback_cid = *self
            .cid_by_char
            .get(&'?')
            .expect("Courier Prime character map should include '?'");
        self.cid_by_char
            .get(&character)
            .copied()
            .unwrap_or(fallback_cid)
            .to_be_bytes()
    }
}

fn collect_document_chars(document: &PdfRenderDocument) -> BTreeSet<char> {
    let mut chars = BTreeSet::from(['?', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);

    if let Some(title_page) = &document.title_page {
        for block in &title_page.blocks {
            for line in &block.lines {
                chars.extend(line.plain_text().chars());
            }
        }
    }

    for page in &document.body_pages {
        if let Some(display_page_number) = page.display_page_number {
            chars.extend(format!("{display_page_number}.").chars());
        }

        for line in &page.lines {
            chars.extend(rendered_line_chars(line));
        }
    }

    chars
}

fn font_object_ids(start: i32) -> FontObjectIds {
    FontObjectIds {
        type0_font_id: Ref::new(start),
        cid_font_id: Ref::new(start + 1),
        font_descriptor_id: Ref::new(start + 2),
        font_file_id: Ref::new(start + 3),
        to_unicode_id: Ref::new(start + 4),
        cid_to_gid_map_id: Ref::new(start + 5),
    }
}

fn title_page_fragments(
    title_page: &PdfTitlePage,
    kind: PdfTitleBlockKind,
    line: &ElementText,
) -> Vec<PdfRenderFragment> {
    title_page_fragments_for_kind_and_caps(kind, line, title_page.plain_title_all_caps)
}

fn title_page_fragments_for_kind(
    kind: PdfTitleBlockKind,
    line: &ElementText,
) -> Vec<PdfRenderFragment> {
    title_page_fragments_for_kind_and_caps(kind, line, false)
}

fn title_page_fragments_for_kind_and_caps(
    kind: PdfTitleBlockKind,
    line: &ElementText,
    plain_title_all_caps: bool,
) -> Vec<PdfRenderFragment> {
    match line {
        ElementText::Plain(text) => vec![PdfRenderFragment {
            text: if kind == PdfTitleBlockKind::Title && plain_title_all_caps {
                text.to_ascii_uppercase()
            } else {
                text.clone()
            },
            styles: Vec::new(),
        }],
        ElementText::Styled(runs) => runs
            .iter()
            .map(|run| PdfRenderFragment {
                text: run.content.clone(),
                styles: sorted_run_styles(run.text_style.iter().cloned()),
            })
            .collect(),
    }
}

fn sorted_run_styles(styles: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut styles = styles.into_iter().collect::<Vec<_>>();
    styles.sort();
    styles
}

fn displayed_body_fragments(
    line: &PdfRenderLine,
    geometry: &LayoutGeometry,
) -> Vec<PdfRenderFragment> {
    let Some(kind) = line.kind else {
        return line.fragments.clone();
    };
    trim_leading_fragment_spaces(
        &line.fragments,
        body_line_indent_spaces(kind, &line.text, geometry),
    )
}

fn trim_leading_fragment_spaces(
    fragments: &[PdfRenderFragment],
    mut spaces_to_trim: usize,
) -> Vec<PdfRenderFragment> {
    let mut trimmed = Vec::new();

    for fragment in fragments {
        if spaces_to_trim == 0 {
            trimmed.push(fragment.clone());
            continue;
        }

        let fragment_len = fragment.text.chars().count();
        if spaces_to_trim >= fragment_len && fragment.text.chars().all(|ch| ch == ' ') {
            spaces_to_trim -= fragment_len;
            continue;
        }

        let mut skipped = 0usize;
        let text = fragment
            .text
            .chars()
            .filter(|character| {
                if skipped < spaces_to_trim && *character == ' ' {
                    skipped += 1;
                    false
                } else {
                    true
                }
            })
            .collect::<String>();
        spaces_to_trim = spaces_to_trim.saturating_sub(skipped);
        if !text.is_empty() {
            trimmed.push(PdfRenderFragment {
                text,
                styles: fragment.styles.clone(),
            });
        }
    }

    trimmed
}

fn resolve_runs(fragments: &[PdfRenderFragment], default_styles: StyleFlags) -> Vec<ResolvedRun> {
    let mut runs = Vec::new();
    for fragment in fragments {
        if fragment.text.is_empty() {
            continue;
        }
        runs.push(ResolvedRun {
            text: fragment.text.clone(),
            styles: merge_style_flags(default_styles, style_flags_from_names(&fragment.styles)),
        });
    }
    runs
}

fn style_flags_from_names(styles: &[String]) -> StyleFlags {
    let mut flags = StyleFlags::default();
    for style in styles {
        if style.eq_ignore_ascii_case("bold") {
            flags.bold = true;
        } else if style.eq_ignore_ascii_case("italic") {
            flags.italic = true;
        } else if style.eq_ignore_ascii_case("underline") {
            flags.underline = true;
        }
    }
    flags
}

fn merge_style_flags(base: StyleFlags, extra: StyleFlags) -> StyleFlags {
    StyleFlags {
        bold: base.bold || extra.bold,
        italic: base.italic || extra.italic,
        underline: base.underline || extra.underline,
    }
}

fn default_body_line_styles(kind: Option<PdfLineKind>) -> StyleFlags {
    match kind {
        Some(PdfLineKind::Lyric) => StyleFlags {
            italic: true,
            ..StyleFlags::default()
        },
        _ => StyleFlags::default(),
    }
}

fn default_title_page_styles(kind: PdfTitleBlockKind, line: &ElementText) -> StyleFlags {
    if kind == PdfTitleBlockKind::Title && matches!(line, ElementText::Plain(_)) {
        return StyleFlags {
            bold: true,
            underline: true,
            ..StyleFlags::default()
        };
    }
    StyleFlags::default()
}

fn render_underlines(content: &mut Content, underlines: &[UnderlineSegment]) {
    if underlines.is_empty() {
        return;
    }

    content.set_line_width(UNDERLINE_LINE_WIDTH);
    for underline in underlines {
        let y = underline.y - UNDERLINE_Y_OFFSET;
        content.move_to(underline.start_x, y);
        content.line_to(underline.end_x, y);
        content.stroke();
    }
}

fn rendered_line_chars(line: &PdfRenderLine) -> Vec<char> {
    let mut chars = Vec::new();
    chars.extend(line.text.chars());
    if let Some(dual) = &line.dual {
        if let Some(left) = &dual.left {
            chars.extend(left.text.chars());
        }
        if let Some(right) = &dual.right {
            chars.extend(right.text.chars());
        }
    }
    chars
}

fn average_width(widths: &[f32]) -> f32 {
    if widths.is_empty() {
        return 0.0;
    }

    widths.iter().sum::<f32>() / widths.len() as f32
}

fn font_flags(face: &Face<'_>) -> FontFlags {
    let mut flags = FontFlags::NON_SYMBOLIC;
    if face.is_monospaced() {
        flags |= FontFlags::FIXED_PITCH;
    }
    if face.is_italic() {
        flags |= FontFlags::ITALIC;
    }
    flags
}

impl From<TitlePageBlockKind> for PdfTitleBlockKind {
    fn from(value: TitlePageBlockKind) -> Self {
        match value {
            TitlePageBlockKind::Title => Self::Title,
            TitlePageBlockKind::Credit => Self::Credit,
            TitlePageBlockKind::Author => Self::Author,
            TitlePageBlockKind::Source => Self::Source,
            TitlePageBlockKind::Contact => Self::Contact,
            TitlePageBlockKind::Draft => Self::Draft,
            TitlePageBlockKind::DraftDate => Self::DraftDate,
        }
    }
}

impl From<TitlePageRegion> for PdfTitleBlockRegion {
    fn from(value: TitlePageRegion) -> Self {
        match value {
            TitlePageRegion::CenterTitle => Self::CenterTitle,
            TitlePageRegion::CenterMeta => Self::CenterMeta,
            TitlePageRegion::BottomLeft => Self::BottomLeft,
            TitlePageRegion::BottomRight => Self::BottomRight,
        }
    }
}

impl From<ElementType> for PdfLineKind {
    fn from(value: ElementType) -> Self {
        match value {
            ElementType::Action => Self::Action,
            ElementType::ColdOpening => Self::ColdOpening,
            ElementType::NewAct => Self::NewAct,
            ElementType::EndOfAct => Self::EndOfAct,
            ElementType::SceneHeading => Self::SceneHeading,
            ElementType::Character => Self::Character,
            ElementType::Dialogue => Self::Dialogue,
            ElementType::Parenthetical => Self::Parenthetical,
            ElementType::Transition => Self::Transition,
            ElementType::Lyric => Self::Lyric,
            ElementType::DualDialogueLeft => Self::DualDialogueLeft,
            ElementType::DualDialogueRight => Self::DualDialogueRight,
            ElementType::DualDialogueCharacterLeft => Self::DualDialogueCharacterLeft,
            ElementType::DualDialogueCharacterRight => Self::DualDialogueCharacterRight,
            ElementType::DualDialogueParentheticalLeft => Self::DualDialogueParentheticalLeft,
            ElementType::DualDialogueParentheticalRight => Self::DualDialogueParentheticalRight,
        }
    }
}

impl From<VisualFragment> for PdfRenderFragment {
    fn from(value: VisualFragment) -> Self {
        Self {
            text: value.text,
            styles: value.styles,
        }
    }
}

impl From<VisualDualLine> for PdfRenderDualLine {
    fn from(value: VisualDualLine) -> Self {
        Self {
            left: value.left.map(Into::into),
            right: value.right.map(Into::into),
        }
    }
}

impl From<VisualDualSide> for PdfRenderDualSide {
    fn from(value: VisualDualSide) -> Self {
        Self {
            text: value.text,
            kind: value.element_type.into(),
            fragments: value.fragments.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blank_attributes, p, tr, Attributes, Element, Metadata};

    #[test]
    fn pdf_render_document_is_derived_from_real_title_page_and_paginated_visual_pages() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);
        metadata.insert("credit".into(), vec![p("Written by")]);
        metadata.insert("author".into(), vec![p("A WRITER")]);
        metadata.insert("contact".into(), vec![p("writer@example.com")]);
        metadata.insert("draft".into(), vec![p("Blue Draft")]);
        metadata.insert("draft date".into(), vec![p("April 4, 2026")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::Action(
                    p("SECOND BODY PAGE"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());

        let title_page = document.title_page.expect("expected title page");
        assert_eq!(title_page.blocks.len(), 6);
        assert_eq!(title_page.blocks[0].kind, PdfTitleBlockKind::Title);
        assert_eq!(
            title_page.blocks[0].region,
            PdfTitleBlockRegion::CenterTitle
        );
        assert_eq!(title_page.blocks[3].kind, PdfTitleBlockKind::Contact);
        assert_eq!(title_page.blocks[3].region, PdfTitleBlockRegion::BottomLeft);

        assert_eq!(document.body_pages.len(), 2);
        assert_eq!(document.body_pages[0].page_number, 2);
        assert_eq!(document.body_pages[0].display_page_number, None);
        assert_eq!(
            document.body_pages[0].lines[0].text.trim(),
            "FIRST BODY PAGE"
        );
        assert_eq!(
            document.body_pages[0].lines[0].kind,
            Some(PdfLineKind::Action)
        );

        assert_eq!(document.body_pages[1].page_number, 3);
        assert_eq!(document.body_pages[1].display_page_number, Some(2));
        assert_eq!(
            document.body_pages[1].lines[0].text.trim(),
            "SECOND BODY PAGE"
        );
        assert_eq!(
            document.body_pages[1].lines[0].kind,
            Some(PdfLineKind::Action)
        );
    }

    #[test]
    fn pdf_render_output_emits_valid_pdf_bytes_with_expected_page_count() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::Action(
                    p("SECOND BODY PAGE"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let pdf = render(&screenplay);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf_text.contains("/Type /Catalog"));
        assert!(pdf_text.contains("/Type /Pages"));
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("/Subtype /Type0"));
        assert!(pdf_text.contains("/FontFile2"));
        assert!(pdf_text.contains("/BaseFont /CourierPrime-Regular"));
    }

    #[test]
    fn pdf_render_output_includes_body_page_text_in_content_streams() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("ALEX"), blank_attributes()),
                    Element::Dialogue(p("HELLO FROM PAGE ONE"), blank_attributes()),
                ]),
            ],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);

        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "FIRST BODY PAGE", 108.0, 711.0);
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "ALEX", 252.0, 687.0);
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "HELLO FROM PAGE ONE",
            180.0,
            675.0,
        );
    }

    #[test]
    fn pdf_render_output_omits_first_body_page_number_and_renders_later_body_numbers() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("TITLE PAGE")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::Action(
                    p("SECOND BODY PAGE"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let first_page =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);
        let second_page =
            render_body_page_content(&document.body_pages[1], &LayoutGeometry::default(), &fonts);

        assert_stream_lacks_text(&first_page, &fonts.regular, "1.");
        assert_stream_contains_fixed_cell_text_at(
            &second_page,
            &fonts.regular,
            "2.",
            page_number_x(2),
            PAGE_NUMBER_BASELINE_Y,
        );
    }

    #[test]
    fn pdf_render_output_positions_centered_lines_away_from_the_body_left_margin() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                p("CENTERED LINE"),
                Attributes {
                    centered: true,
                    ..blank_attributes()
                },
            )],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(!pdf_text.contains("72 711 Tm"));
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "CENTERED LINE", 278.5, 711.0);
    }

    #[test]
    fn pdf_render_output_uses_dual_dialogue_margins_for_both_sides() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(p("BOB"), blank_attributes()),
                    Element::Dialogue(p("LEFT"), blank_attributes()),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(p("CAROL"), blank_attributes()),
                    Element::Dialogue(p("RIGHT"), blank_attributes()),
                ]),
            ])],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("201 711 Tm"));
        assert!(pdf_text.contains("419 711 Tm"));
        assert!(pdf_text.contains("108 699 Tm"));
        assert!(pdf_text.contains("333 699 Tm"));
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "BOB", 201.0, 711.0);
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "CAROL", 419.0, 711.0);
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "LEFT", 108.0, 699.0);
        assert_stream_contains_fixed_cell_text_at(&content, &fonts.regular, "RIGHT", 333.0, 699.0);
    }

    #[test]
    fn pdf_render_output_includes_title_page_text_regions() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("SAMPLE SCRIPT")]);
        metadata.insert("credit".into(), vec![p("written by")]);
        metadata.insert("author".into(), vec![p("Alan Smithee")]);
        metadata.insert(
            "source".into(),
            vec![p("based on the novel"), p("by J.R.R. Smithee")],
        );
        metadata.insert("contact".into(), vec![p("WME"), p("Los Angeles")]);
        metadata.insert("draft".into(), vec![p("First Draft")]);
        metadata.insert("draft date".into(), vec![p("April 6, 1952")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content = render_title_page_content(
            document.title_page.as_ref().expect("expected title page"),
            &fonts,
        );

        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.bold,
            "SAMPLE SCRIPT",
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle),
            TITLE_TITLE_TOP,
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "written by",
            title_page_line_left("written by", PdfTitleBlockRegion::CenterMeta),
            TITLE_META_TOP,
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "Alan Smithee",
            title_page_line_left("Alan Smithee", PdfTitleBlockRegion::CenterMeta),
            TITLE_META_TOP - (TITLE_BLOCK_LINE_STEP * 2.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "based on the novel",
            title_page_line_left("based on the novel", PdfTitleBlockRegion::CenterMeta),
            TITLE_META_TOP - (TITLE_BLOCK_LINE_STEP * 7.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "by J.R.R. Smithee",
            title_page_line_left("by J.R.R. Smithee", PdfTitleBlockRegion::CenterMeta),
            TITLE_META_TOP - (TITLE_BLOCK_LINE_STEP * 8.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "WME",
            title_page_line_left("WME", PdfTitleBlockRegion::BottomLeft),
            TITLE_BOTTOM_TOP,
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "First Draft",
            title_page_bottom_right_left("First Draft"),
            TITLE_BOTTOM_TOP,
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "April 6, 1952",
            title_page_bottom_right_left("April 6, 1952"),
            TITLE_BOTTOM_TOP - TITLE_BLOCK_LINE_STEP,
        );
    }

    #[test]
    fn pdf_render_output_uppercases_and_underlines_plain_title_page_titles() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("Sample Script")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            TITLE_TITLE_TOP,
            TITLE_FONT_SIZE,
            &mut underlines,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = content.finish().to_vec();
        let stream_text = String::from_utf8_lossy(&stream);

        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "SAMPLE SCRIPT",
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle),
            TITLE_TITLE_TOP,
        );
        assert!(stream_text.contains("260.5 493.5 m"));
        assert!(stream_text.contains("351.5 493.5 l"));
    }

    #[test]
    fn pdf_render_output_preserves_plain_title_case_when_fmt_allows_lowercase_title() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("Sample Script")]);
        metadata.insert("fmt".into(), vec![p("allow-lowercase-title")]);

        let screenplay = Screenplay {
            metadata,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            TITLE_TITLE_TOP,
            TITLE_FONT_SIZE,
            &mut underlines,
        );
        content.end_text();
        let stream = content.finish().to_vec();

        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "Sample Script",
            title_page_line_left("Sample Script", PdfTitleBlockRegion::CenterTitle),
            TITLE_TITLE_TOP,
        );
        assert_stream_lacks_text(&stream, &fonts.bold, "SAMPLE SCRIPT");
    }

    #[test]
    fn pdf_render_output_uses_font_variants_and_underlines_for_styled_fragments() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                ElementText::Styled(vec![
                    tr("PLAIN ", vec![]),
                    tr("BOLD ", vec!["Bold"]),
                    tr("ITALIC ", vec!["Italic"]),
                    tr("BOTH ", vec!["Bold", "Italic"]),
                    tr("UNDER", vec!["Underline"]),
                ]),
                blank_attributes(),
            )],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("/F2 12 Tf"));
        assert!(pdf_text.contains("/F3 12 Tf"));
        assert!(pdf_text.contains("/F4 12 Tf"));
        assert!(pdf_text.contains("0.75 w"));
    }

    #[test]
    fn pdf_render_output_applies_default_scene_heading_and_lyric_styles() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![
                Element::SceneHeading(p("INT. OFFICE - DAY"), blank_attributes()),
                Element::Lyric(p("I love to sing"), blank_attributes()),
            ],
        };

        let document = build_render_document(&screenplay, PdfRenderOptions::default());
        let fonts = EmbeddedFonts::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &fonts);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("/F3 12 Tf"));
    }

    #[test]
    fn underline_segments_do_not_extend_into_trailing_wrap_spaces() {
        let document = PdfRenderDocument {
            title_page: None,
            body_pages: vec![],
        };
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        content.begin_text();
        render_fixed_cell_runs(
            &mut content,
            &fonts,
            &[ResolvedRun {
                text: "FALL ".into(),
                styles: StyleFlags {
                    underline: true,
                    ..StyleFlags::default()
                },
            }],
            108.0,
            720.0,
            BODY_TEXT_FONT_SIZE,
            &mut underlines,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = String::from_utf8_lossy(&content.finish().to_vec()).into_owned();

        assert!(stream.contains("108 718.5 m"));
        assert!(stream.contains("136 718.5 l"));
        assert!(!stream.contains("143 718.5 l"));
    }

    #[test]
    fn title_page_line_left_uses_fixed_cell_widths() {
        assert_eq!(
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle),
            260.5
        );
        assert_eq!(
            title_page_bottom_right_left("April 6, 1952"),
            445.7
        );
    }

    #[test]
    fn body_line_left_uses_layout_geometry_for_element_kinds() {
        let geometry = LayoutGeometry::default();

        assert_eq!(line_kind_left(Some(PdfLineKind::Action), &geometry), 108.0);
        assert_eq!(
            line_kind_left(Some(PdfLineKind::Character), &geometry),
            252.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::Dialogue), &geometry),
            180.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::Parenthetical), &geometry),
            216.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::Transition), &geometry),
            396.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::DualDialogueLeft), &geometry),
            108.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::DualDialogueRight), &geometry),
            333.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::DualDialogueParentheticalLeft), &geometry),
            126.0
        );
        assert_eq!(
            line_kind_left(Some(PdfLineKind::DualDialogueParentheticalRight), &geometry),
            351.0
        );
    }

    #[test]
    fn body_line_left_right_aligns_transitions_to_the_transition_right_margin() {
        let geometry = LayoutGeometry::default();
        let line = PdfRenderLine {
            text: format!("{}CUT TO:", " ".repeat(40)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Transition),
            fragments: Vec::new(),
            dual: None,
        };

        assert!((body_line_left(&line, &geometry) - 462.2).abs() < 0.001);
    }

    #[test]
    fn body_line_left_hangs_opening_parenthetical_one_cell_left() {
        let geometry = LayoutGeometry::default();
        let line = PdfRenderLine {
            text: format!("{}(quietly)", " ".repeat(14)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Parenthetical),
            fragments: Vec::new(),
            dual: None,
        };
        let continuation = PdfRenderLine {
            text: format!("{}quietly", " ".repeat(15)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Parenthetical),
            fragments: Vec::new(),
            dual: None,
        };

        assert!((body_line_left(&line, &geometry) - 209.0).abs() < 0.001);
        assert_eq!(rendered_body_line_text(&line, &geometry), "(quietly)");
        assert!((body_line_left(&continuation, &geometry) - 216.0).abs() < 0.001);
        assert_eq!(rendered_body_line_text(&continuation, &geometry), "quietly");
    }

    #[test]
    fn dual_side_left_uses_the_dynamic_final_draft_formula_for_dual_character_cues() {
        let geometry = LayoutGeometry::default();

        let short_left = PdfRenderDualSide {
            text: "BOB".into(),
            kind: PdfLineKind::DualDialogueCharacterLeft,
            fragments: Vec::new(),
        };
        let longer_right = PdfRenderDualSide {
            text: "CAROL".into(),
            kind: PdfLineKind::DualDialogueCharacterRight,
            fragments: Vec::new(),
        };

        assert_eq!(dual_side_left(&short_left, &geometry), 201.0);
        assert_eq!(dual_side_left(&longer_right, &geometry), 419.0);
    }

    #[test]
    fn dual_side_left_centers_final_draft_dual_cues_on_the_speaker_name_not_the_contd_suffix() {
        let geometry = LayoutGeometry::default();
        let right_with_contd = PdfRenderDualSide {
            text: "AMY (CONT'D)".into(),
            kind: PdfLineKind::DualDialogueCharacterRight,
            fragments: Vec::new(),
        };

        assert!((dual_side_left(&right_with_contd, &geometry) - 426.0).abs() < 0.001);
    }

    #[test]
    fn dual_parenthetical_left_does_not_hang_opening_parenthesis() {
        let geometry = LayoutGeometry::default();
        let side = PdfRenderDualSide {
            text: "(quietly)".into(),
            kind: PdfLineKind::DualDialogueParentheticalLeft,
            fragments: Vec::new(),
        };

        assert_eq!(dual_side_left(&side, &geometry), 126.0);
    }

    #[test]
    fn body_vertical_metrics_are_derived_from_letter_and_54_lines() {
        assert_eq!(body_line_step_points(), 12.0);
        assert_eq!(first_body_line_y(), 711.0);
        assert_eq!(page_number_y(), PAGE_NUMBER_BASELINE_Y);
    }

    #[test]
    fn page_number_x_keeps_the_period_column_fixed() {
        assert_eq!(page_number_x(2), PAGE_NUMBER_LEFT);
        assert_eq!(page_number_x(34), PAGE_NUMBER_LEFT - BODY_TEXT_CELL_WIDTH);
        assert_eq!(page_number_x(100), PAGE_NUMBER_LEFT - (2.0 * BODY_TEXT_CELL_WIDTH));
    }

    #[test]
    fn split_contd_character_at_top_of_page_lifts_the_page_start_by_one_line() {
        let page = PdfRenderPage {
            page_number: 35,
            display_page_number: Some(34),
            lines: vec![
                PdfRenderLine {
                    text: "MAYOR (CONT'D)".into(),
                    counted: false,
                    centered: false,
                    kind: Some(PdfLineKind::Character),
                    fragments: Vec::new(),
                    dual: None,
                },
                PdfRenderLine {
                    text: format!("{}But take with you this Key", " ".repeat(10)),
                    counted: true,
                    centered: false,
                    kind: Some(PdfLineKind::Dialogue),
                    fragments: Vec::new(),
                    dual: None,
                },
            ],
        };

        assert!(page_starts_with_split_contd_character(&page));
        assert_eq!(
            first_body_line_y_for_page(&page),
            first_body_line_y() + body_line_step_points()
        );
    }

    #[test]
    fn ordinary_counted_contd_character_does_not_get_the_top_of_page_lift() {
        let page = PdfRenderPage {
            page_number: 35,
            display_page_number: Some(34),
            lines: vec![PdfRenderLine {
                text: "MAYOR (CONT'D)".into(),
                counted: true,
                centered: false,
                kind: Some(PdfLineKind::Character),
                fragments: Vec::new(),
                dual: None,
            }],
        };

        assert!(!page_starts_with_split_contd_character(&page));
        assert_eq!(first_body_line_y_for_page(&page), first_body_line_y());
    }

    fn assert_stream_lacks_text(stream: &[u8], font: &EmbeddedFont, text: &str) {
        let encoded = pdf_literal_text(font, text);
        assert!(
            !stream
                .windows(encoded.len())
                .any(|window| window == encoded),
            "expected stream to omit encoded text: {text}"
        );
    }

    fn assert_stream_contains_fixed_cell_text_at(
        stream: &[u8],
        font: &EmbeddedFont,
        text: &str,
        start_x: f32,
        y: f32,
    ) {
        let stream_text = String::from_utf8_lossy(stream);
        for (index, character) in text.chars().enumerate() {
            if character == ' ' {
                continue;
            }

            let x = start_x + (index as f32 * BODY_TEXT_CELL_WIDTH);
            let matrix = format!("1 0 0 1 {x} {y} Tm");
            assert!(
                stream_text.contains(&matrix),
                "expected stream to contain matrix: {matrix}"
            );

            let encoded = pdf_literal_text(font, &character.to_string());
            assert!(
                stream
                    .windows(encoded.len())
                    .any(|window| window == encoded),
                "expected stream to contain encoded character: {character}"
            );
        }
    }

    fn pdf_literal_text(font: &EmbeddedFont, text: &str) -> Vec<u8> {
        let mut escaped = Vec::new();
        for byte in font.encode_text(text) {
            match byte {
                b'(' | b')' | b'\\' => {
                    escaped.push(b'\\');
                    escaped.push(byte);
                }
                b'\n' => escaped.extend_from_slice(b"\\n"),
                b'\r' => escaped.extend_from_slice(b"\\r"),
                b'\t' => escaped.extend_from_slice(b"\\t"),
                0x08 => escaped.extend_from_slice(b"\\b"),
                0x0C => escaped.extend_from_slice(b"\\f"),
                0x20..=0x7E => escaped.push(byte),
                _ => escaped.extend_from_slice(format!("\\{byte:03o}").as_bytes()),
            }
        }
        escaped
    }
}
