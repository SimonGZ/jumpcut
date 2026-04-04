#![allow(dead_code)]

use crate::pagination::margin::{
    calculate_element_width, dual_dialogue_character_left_indent, LayoutGeometry,
};
use crate::pagination::wrapping::ElementType;
use crate::pagination::ScreenplayLayoutProfile;
use crate::title_page::{TitlePage, TitlePageBlockKind, TitlePageRegion};
use crate::visual_lines::{
    display_page_number, render_paginated_visual_pages, VisualDualLine, VisualDualSide,
    VisualFragment,
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
const PAGE_NUMBER_LEFT: f32 = 475.2;
const PAGE_NUMBER_LINES_ABOVE_BODY: f32 = 3.0;
const PAGE_NUMBER_BODY_GAP_LINES: f32 = 4.0;
const TITLE_FONT_SIZE: f32 = 24.0;
const TITLE_META_FONT_SIZE: f32 = 12.0;
const TITLE_BLOCK_LINE_STEP: f32 = 14.0;
const TITLE_TITLE_TOP: f32 = 620.0;
const TITLE_META_TOP: f32 = 520.0;
const TITLE_BOTTOM_TOP: f32 = 120.0;
const TITLE_BOTTOM_MARGIN: f32 = 72.0;
const FONT_NAME: Name<'static> = Name(b"F1");
const BODY_TEXT_HORIZONTAL_SCALING: f32 = 97.68321;
const COURIER_PRIME_FONT_BYTES: &[u8] = include_bytes!("templates/fonts/CourierPrime-Regular.ttf");
const COURIER_PRIME_BASE_FONT: Name<'static> = Name(b"CourierPrime-Regular");
const COURIER_PRIME_CMAP_NAME: Name<'static> = Name(b"CourierPrime-Regular-UTF16");
const IDENTITY_H: Name<'static> = Name(b"Identity-H");
const ADOBE_IDENTITY: SystemInfo<'static> = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfRenderDocument {
    pub title_page: Option<PdfTitlePage>,
    pub body_pages: Vec<PdfRenderPage>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitlePage {
    pub blocks: Vec<PdfTitleBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitleBlock {
    pub kind: PdfTitleBlockKind,
    pub region: PdfTitleBlockRegion,
    pub lines: Vec<ElementText>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    cid_by_char: BTreeMap<char, u16>,
    font_descriptor: EmbeddedFontDescriptor,
    cid_widths: Vec<f32>,
    cid_to_gid_map: Vec<u8>,
    to_unicode_cmap: Vec<u8>,
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

pub(crate) fn build_render_document(screenplay: &Screenplay) -> PdfRenderDocument {
    let title_page =
        TitlePage::from_metadata(&screenplay.metadata).map(|title_page| PdfTitlePage {
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

    let body_pages = render_paginated_visual_pages(screenplay)
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
    let document = build_render_document(screenplay);
    let geometry =
        ScreenplayLayoutProfile::from_metadata(&screenplay.metadata).to_pagination_geometry();
    let font = EmbeddedFont::new(&document);
    let body_page_count = document.body_pages.len() as i32;
    let page_count = body_page_count + i32::from(document.title_page.is_some());

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let type0_font_id = Ref::new(3);
    let cid_font_id = Ref::new(4);
    let font_descriptor_id = Ref::new(5);
    let font_file_id = Ref::new(6);
    let to_unicode_id = Ref::new(7);
    let cid_to_gid_map_id = Ref::new(8);
    let page_ids = (0..page_count)
        .map(|index| Ref::new(9 + index))
        .collect::<Vec<_>>();
    let content_ids = (0..page_count)
        .map(|index| Ref::new(9 + page_count + index))
        .collect::<Vec<_>>();

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_count);
    write_embedded_font_objects(
        &mut pdf,
        &font,
        type0_font_id,
        cid_font_id,
        font_descriptor_id,
        font_file_id,
        to_unicode_id,
        cid_to_gid_map_id,
    );

    for (index, page_id) in page_ids.iter().copied().enumerate() {
        let mut page = pdf.page(page_id);
        page.parent(page_tree_id)
            .media_box(Rect::new(0.0, 0.0, LETTER_WIDTH, LETTER_HEIGHT))
            .contents(content_ids[index]);
        page.resources().fonts().pair(FONT_NAME, type0_font_id);
        page.finish();
    }

    let mut content_index = 0usize;
    if let Some(title_page) = &document.title_page {
        pdf.stream(
            content_ids[content_index],
            &render_title_page_content(title_page, &font),
        );
        content_index += 1;
    }

    for body_page in &document.body_pages {
        pdf.stream(
            content_ids[content_index],
            &render_body_page_content(body_page, &geometry, &font),
        );
        content_index += 1;
    }

    pdf.finish()
}

fn render_body_page_content(
    page: &PdfRenderPage,
    geometry: &LayoutGeometry,
    font: &EmbeddedFont,
) -> Vec<u8> {
    let mut content = Content::new();
    content.begin_text();
    content.set_font(FONT_NAME, BODY_TEXT_FONT_SIZE);
    content.set_horizontal_scaling(BODY_TEXT_HORIZONTAL_SCALING);
    let line_step = body_line_step_points();

    if let Some(display_page_number) = page.display_page_number {
        let page_number = format!("{display_page_number}.");
        content.set_text_matrix([1.0, 0.0, 0.0, 1.0, PAGE_NUMBER_LEFT, page_number_y()]);
        let encoded_page_number = font.encode_text(&page_number);
        content.show(Str(&encoded_page_number));
    }

    let body_top = if page.display_page_number.is_some() {
        first_body_line_y() - (PAGE_NUMBER_BODY_GAP_LINES * line_step)
    } else {
        first_body_line_y()
    };
    for (index, line) in page.lines.iter().enumerate() {
        if line.text.is_empty() {
            continue;
        }

        let line_y = body_top - (index as f32 * line_step);
        if let Some(dual) = &line.dual {
            render_dual_body_line(&mut content, dual, geometry, font, line_y);
            continue;
        }
        let text = rendered_body_line_text(line, geometry);
        content.set_text_matrix([1.0, 0.0, 0.0, 1.0, body_line_left(line, geometry), line_y]);
        let encoded_text = font.encode_text(text);
        content.show(Str(&encoded_text));
    }

    content.end_text();
    content.finish().to_vec()
}

fn render_dual_body_line(
    content: &mut Content,
    dual: &PdfRenderDualLine,
    geometry: &LayoutGeometry,
    font: &EmbeddedFont,
    line_y: f32,
) {
    if let Some(left) = &dual.left {
        let encoded_left = font.encode_text(&left.text);
        content.set_text_matrix([1.0, 0.0, 0.0, 1.0, dual_side_left(left, geometry), line_y]);
        content.show(Str(&encoded_left));
    }

    if let Some(right) = &dual.right {
        let encoded_right = font.encode_text(&right.text);
        content.set_text_matrix([1.0, 0.0, 0.0, 1.0, dual_side_left(right, geometry), line_y]);
        content.show(Str(&encoded_right));
    }
}

fn body_line_step_points() -> f32 {
    let usable_page_height = LETTER_HEIGHT - (2.0 * PAGE_TOP_BOTTOM_MARGIN_INCHES * 72.0);
    usable_page_height / BODY_LINES_PER_PAGE
}

fn first_body_line_y() -> f32 {
    LETTER_HEIGHT - (PAGE_TOP_BOTTOM_MARGIN_INCHES * 72.0)
}

fn page_number_y() -> f32 {
    first_body_line_y() + (PAGE_NUMBER_LINES_ABOVE_BODY * body_line_step_points())
}

fn render_title_page_content(title_page: &PdfTitlePage, font: &EmbeddedFont) -> Vec<u8> {
    let mut content = Content::new();
    content.begin_text();
    content.set_font(FONT_NAME, TITLE_META_FONT_SIZE);

    render_title_page_region(
        &mut content,
        title_page,
        font,
        PdfTitleBlockRegion::CenterTitle,
        TITLE_TITLE_TOP,
        TITLE_FONT_SIZE,
    );
    render_title_page_region(
        &mut content,
        title_page,
        font,
        PdfTitleBlockRegion::CenterMeta,
        TITLE_META_TOP,
        TITLE_META_FONT_SIZE,
    );
    render_title_page_region(
        &mut content,
        title_page,
        font,
        PdfTitleBlockRegion::BottomLeft,
        TITLE_BOTTOM_TOP,
        TITLE_META_FONT_SIZE,
    );
    render_title_page_region(
        &mut content,
        title_page,
        font,
        PdfTitleBlockRegion::BottomRight,
        TITLE_BOTTOM_TOP,
        TITLE_META_FONT_SIZE,
    );

    content.end_text();
    content.finish().to_vec()
}

fn render_title_page_region(
    content: &mut Content,
    title_page: &PdfTitlePage,
    font: &EmbeddedFont,
    region: PdfTitleBlockRegion,
    top_y: f32,
    font_size: f32,
) {
    let mut line_index = 0usize;
    content.set_font(FONT_NAME, font_size);

    for block in title_page
        .blocks
        .iter()
        .filter(|block| block.region == region)
    {
        for line in &block.lines {
            let text = line.plain_text();
            let y = top_y - (line_index as f32 * TITLE_BLOCK_LINE_STEP);
            let encoded_text = font.encode_text(&text);
            content.set_text_matrix([
                1.0,
                0.0,
                0.0,
                1.0,
                title_page_line_left(&text, region, font_size),
                y,
            ]);
            content.show(Str(&encoded_text));
            line_index += 1;
        }
        line_index += 1;
    }
}

fn write_embedded_font_objects(
    pdf: &mut Pdf,
    font: &EmbeddedFont,
    type0_font_id: Ref,
    cid_font_id: Ref,
    font_descriptor_id: Ref,
    font_file_id: Ref,
    to_unicode_id: Ref,
    cid_to_gid_map_id: Ref,
) {
    pdf.type0_font(type0_font_id)
        .base_font(COURIER_PRIME_BASE_FONT)
        .encoding_predefined(IDENTITY_H)
        .descendant_font(cid_font_id)
        .to_unicode(to_unicode_id);

    let mut cid_font = pdf.cid_font(cid_font_id);
    cid_font
        .subtype(CidFontType::Type2)
        .base_font(COURIER_PRIME_BASE_FONT)
        .system_info(ADOBE_IDENTITY)
        .font_descriptor(font_descriptor_id)
        .default_width(font.font_descriptor.missing_width);
    cid_font
        .widths()
        .consecutive(1, font.cid_widths.iter().copied())
        .finish();
    cid_font.cid_to_gid_map_stream(cid_to_gid_map_id);
    cid_font.finish();

    pdf.font_descriptor(font_descriptor_id)
        .name(COURIER_PRIME_BASE_FONT)
        .family(Str(b"Courier Prime"))
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
        .font_file2(font_file_id);

    let mut font_file_stream = pdf.stream(font_file_id, COURIER_PRIME_FONT_BYTES);
    font_file_stream.pair(Name(b"Length1"), COURIER_PRIME_FONT_BYTES.len() as i32);
    font_file_stream.finish();

    pdf.stream(to_unicode_id, &font.to_unicode_cmap);
    pdf.stream(cid_to_gid_map_id, &font.cid_to_gid_map);
}

fn title_page_line_left(text: &str, region: PdfTitleBlockRegion, font_size: f32) -> f32 {
    let width = text.chars().count() as f32 * (font_size * 0.6);

    match region {
        PdfTitleBlockRegion::CenterTitle | PdfTitleBlockRegion::CenterMeta => {
            ((LETTER_WIDTH - width) / 2.0).max(TITLE_BOTTOM_MARGIN)
        }
        PdfTitleBlockRegion::BottomLeft => TITLE_BOTTOM_MARGIN,
        PdfTitleBlockRegion::BottomRight => {
            (LETTER_WIDTH - TITLE_BOTTOM_MARGIN - width).max(TITLE_BOTTOM_MARGIN)
        }
    }
}

fn body_line_left(line: &PdfRenderLine, geometry: &LayoutGeometry) -> f32 {
    if !line.centered {
        if matches!(line.kind, Some(PdfLineKind::Transition))
            && geometry.transition_alignment == crate::pagination::Alignment::Right
        {
            let text = rendered_body_line_text(line, geometry);
            let char_width = body_text_char_width_points(geometry);
            let rendered_width = text.chars().count() as f32 * char_width;
            return (geometry.transition_right * 72.0) - rendered_width;
        }
        return line_kind_left(line.kind, geometry);
    }

    let action_left = geometry.action_left * 72.0;
    let chars_per_line = calculate_element_width(geometry, ElementType::Action);
    let char_width = body_text_char_width_points(geometry);
    let rendered_width = line.text.chars().count() as f32 * char_width;
    let available_width = chars_per_line as f32 * char_width;

    action_left + ((available_width - rendered_width) / 2.0).max(0.0)
}

fn body_text_char_width_points(geometry: &LayoutGeometry) -> f32 {
    (72.0 / geometry.cpi) * (BODY_TEXT_HORIZONTAL_SCALING / 100.0)
}

fn rendered_body_line_text<'a>(line: &'a PdfRenderLine, geometry: &LayoutGeometry) -> &'a str {
    if line.centered {
        return &line.text;
    }

    let Some(kind) = line.kind else {
        return &line.text;
    };
    let indent = " ".repeat(synthetic_indent_spaces(kind, geometry));
    line.text.strip_prefix(&indent).unwrap_or(&line.text)
}

fn line_kind_left(kind: Option<PdfLineKind>, geometry: &LayoutGeometry) -> f32 {
    kind.map(|kind| element_left_inches(kind, geometry) * 72.0)
        .unwrap_or(geometry.action_left * 72.0)
}

fn dual_side_left(side: &PdfRenderDualSide, geometry: &LayoutGeometry) -> f32 {
    match side.kind {
        PdfLineKind::DualDialogueCharacterLeft => {
            dual_dialogue_character_left_indent(&side.text, 1) * 72.0
        }
        PdfLineKind::DualDialogueCharacterRight => {
            dual_dialogue_character_left_indent(&side.text, 2) * 72.0
        }
        _ => line_kind_left(Some(side.kind), geometry),
    }
}

fn synthetic_indent_spaces(kind: PdfLineKind, geometry: &LayoutGeometry) -> usize {
    let left = element_left_inches(kind, geometry);
    ((left - geometry.action_left) * geometry.cpi).floor() as usize
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

impl EmbeddedFont {
    fn new(document: &PdfRenderDocument) -> Self {
        let chars = collect_document_chars(document);
        let face = Face::parse(COURIER_PRIME_FONT_BYTES, 0)
            .expect("Courier Prime regular TTF should parse");
        let units_per_em = face.units_per_em() as f32;
        let scale = 1000.0 / units_per_em;

        let mut cid_by_char = BTreeMap::new();
        let mut cid_widths = Vec::new();
        let mut cid_to_gid_map = vec![0_u8, 0_u8];
        let mut to_unicode = UnicodeCmap::new(COURIER_PRIME_CMAP_NAME, ADOBE_IDENTITY);

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
    use crate::{blank_attributes, p, Attributes, Element, Metadata};

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

        let document = build_render_document(&screenplay);

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

        let document = build_render_document(&screenplay);
        let font = EmbeddedFont::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &font);

        assert_stream_contains_text(&content, &font, "FIRST BODY PAGE");
        assert_stream_contains_text(&content, &font, "ALEX");
        assert_stream_contains_text(&content, &font, "HELLO FROM PAGE ONE");
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

        let document = build_render_document(&screenplay);
        let font = EmbeddedFont::new(&document);
        let first_page =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &font);
        let second_page =
            render_body_page_content(&document.body_pages[1], &LayoutGeometry::default(), &font);

        assert_stream_lacks_text(&first_page, &font, "1.");
        assert_stream_contains_text(&second_page, &font, "2.");
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

        let document = build_render_document(&screenplay);
        let font = EmbeddedFont::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &font);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(!pdf_text.contains("72 720 Tm"));
        assert!(pdf_text.contains("276.79657 720 Tm"));
        assert_stream_contains_text(&content, &font, "CENTERED LINE");
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

        let document = build_render_document(&screenplay);
        let font = EmbeddedFont::new(&document);
        let content =
            render_body_page_content(&document.body_pages[0], &LayoutGeometry::default(), &font);
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("202.5 720 Tm"));
        assert!(pdf_text.contains("418.5 720 Tm"));
        assert!(pdf_text.contains("108 708 Tm"));
        assert!(pdf_text.contains("333 708 Tm"));
        assert_stream_contains_text(&content, &font, "BOB");
        assert_stream_contains_text(&content, &font, "CAROL");
        assert_stream_contains_text(&content, &font, "LEFT");
        assert_stream_contains_text(&content, &font, "RIGHT");
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

        let document = build_render_document(&screenplay);
        let font = EmbeddedFont::new(&document);
        let content = render_title_page_content(
            document.title_page.as_ref().expect("expected title page"),
            &font,
        );

        assert_stream_contains_text(&content, &font, "SAMPLE SCRIPT");
        assert_stream_contains_text(&content, &font, "written by");
        assert_stream_contains_text(&content, &font, "Alan Smithee");
        assert_stream_contains_text(&content, &font, "based on the novel");
        assert_stream_contains_text(&content, &font, "by J.R.R. Smithee");
        assert_stream_contains_text(&content, &font, "WME");
        assert_stream_contains_text(&content, &font, "First Draft");
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

        assert!((body_line_left(&line, &geometry) - 461.96765).abs() < 0.001);
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

        assert_eq!(dual_side_left(&short_left, &geometry), 202.5);
        assert_eq!(dual_side_left(&longer_right, &geometry), 418.5);
    }

    #[test]
    fn body_vertical_metrics_are_derived_from_letter_and_54_lines() {
        assert_eq!(body_line_step_points(), 12.0);
        assert_eq!(first_body_line_y(), 720.0);
        assert_eq!(page_number_y(), 756.0);
    }

    fn assert_stream_contains_text(stream: &[u8], font: &EmbeddedFont, text: &str) {
        let encoded = pdf_literal_text(font, text);
        assert!(
            stream
                .windows(encoded.len())
                .any(|window| window == encoded),
            "expected stream to contain encoded text: {text}"
        );
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
