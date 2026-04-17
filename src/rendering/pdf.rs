#![allow(dead_code)]

use crate::pagination::margin::{dual_dialogue_character_left_indent, LayoutGeometry};
use crate::pagination::visual_lines::{
    display_page_number, render_paginated_visual_pages_with_options, VisualDualLine,
    VisualDualSide, VisualFragment, VisualRenderOptions,
};
use crate::pagination::wrapping::{
    wrap_styled_text_for_element, wrap_text_for_element, ElementType, WrapConfig,
};
use crate::pagination::ScreenplayLayoutProfile;
use crate::pagination::{
    BlockPlacement, ContinuationMarker, Fragment, PageItem, PaginatedScreenplay, PaginationScope,
};
use crate::title_page::{
    frontmatter_count, plain_title_uses_all_caps, TitlePage, TitlePageBlockKind, TitlePageRegion,
};
use crate::{
    styled_text::{StyledRun, StyledText},
    ElementText, ImportedTitlePageAlignment, ImportedTitlePageTabStop, Metadata, Screenplay,
};
use pdf_writer::types::{
    ArtifactAttachment, ArtifactSubtype, ArtifactType, CidFontType, FontFlags, NumberingStyle,
    StructRole, SystemInfo, UnicodeCmap,
};
use pdf_writer::{Content, Date, Finish, Name, Pdf, Rect, Ref, Str, TextStr};
use std::collections::{BTreeMap, BTreeSet};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use ttf_parser::Face;

const BODY_TEXT_FONT_SIZE: f32 = 12.0;
const BODY_PAGE_NUMBER_X: f32 = 508.5;
const PAGE_NUMBER_BASELINE_Y: f32 = 747.0;
const PAGE_NUMBER_LEFT: f32 = 508.5;
const TITLE_FONT_SIZE: f32 = 12.0;
const TITLE_META_FONT_SIZE: f32 = 12.0;
const DEFAULT_DOCUMENT_LANGUAGE: &str = "en-US";
const TOOL_IDENTITY: &str = concat!("JumpCut ", env!("CARGO_PKG_VERSION"));
const TITLE_BLOCK_LINE_STEP: f32 = 12.0;
const TITLE_TITLE_TOP_OFFSET_LINES_FROM_BODY_START: f32 = 18.0;
const TITLE_META_TOP_OFFSET_LINES_FROM_BODY_START: f32 = 22.0;
const TITLE_BOTTOM_TOP_OFFSET_LINES_FROM_BOTTOM_MARGIN: f32 = 5.25;
const PAGE_SIDE_MARGIN: f32 = 72.0;
const FONT_REGULAR_NAME: Name<'static> = Name(b"F1");
const FONT_BOLD_NAME: Name<'static> = Name(b"F2");
const FONT_ITALIC_NAME: Name<'static> = Name(b"F3");
const FONT_BOLD_ITALIC_NAME: Name<'static> = Name(b"F4");
const BODY_TEXT_CELL_WIDTH: f32 = 7.0;
const ORDINARY_CHARACTER_CONTD_SUFFIX_X_ADJUSTMENT: f32 = 0.5625;
const UNDERLINE_LINE_WIDTH: f32 = 0.75;
const UNDERLINE_Y_OFFSET: f32 = 1.5;
const SCENE_NUMBER_LEFT_X: f32 = 0.75 * 72.0;
const SCENE_NUMBER_Y_OFFSET: f32 = 1.129;
const COURIER_PRIME_REGULAR_BYTES: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-Regular.ttf");
const COURIER_PRIME_BOLD_BYTES: &[u8] = include_bytes!("../templates/fonts/CourierPrime-Bold.ttf");
const COURIER_PRIME_ITALIC_BYTES: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-Italic.ttf");
const COURIER_PRIME_BOLD_ITALIC_BYTES: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-BoldItalic.ttf");
const IDENTITY_H: Name<'static> = Name(b"Identity-H");
const ADOBE_IDENTITY: SystemInfo<'static> = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdfRenderOptions {
    pub render_continueds: bool,
    pub render_title_page: bool,
}

impl Default for PdfRenderOptions {
    fn default() -> Self {
        Self {
            render_continueds: true,
            render_title_page: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfRenderDocument {
    pub title_page: Option<PdfTitlePage>,
    pub title_overflow_pages: Vec<PdfTitleOverflowPage>,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTaggedDocument {
    pub title_page: Option<PdfTaggedTitlePage>,
    pub body_pages: Vec<PdfTaggedPage>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTaggedTitlePage {
    pub blocks: Vec<PdfTaggedTitleBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTaggedTitleBlock {
    pub id: String,
    pub kind: PdfTitleBlockKind,
    pub role: PdfTaggedRole,
    pub region: PdfTitleBlockRegion,
    pub lines: Vec<ElementText>,
    pub artifact: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfTaggedPage {
    pub page_number: u32,
    pub body_page_number: Option<u32>,
    pub blocks: Vec<PdfTaggedBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PdfBodyStructPage {
    tagged_lines: Vec<PdfBodyStructLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PdfBodyStructLine {
    mcid: i32,
    role: PdfTaggedRole,
    structure_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PdfEmittedStructLine {
    mcid: i32,
    role: PdfTaggedRole,
    dual_side: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PdfStructElementPlan {
    key: String,
    role: PdfTaggedRole,
    refs: Vec<PdfMarkedContentRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PdfMarkedContentRef {
    page_index: usize,
    mcid: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PdfPageLabelPlan {
    page_index: i32,
    style: PdfPageLabelStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PdfPageLabelStyle {
    Blank,
    LowerRoman { offset: i32 },
    Arabic { offset: i32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfTaggedBlock {
    pub id: String,
    pub source_block_id: Option<String>,
    pub placement: PdfTaggedBlockPlacement,
    pub fragment: Fragment,
    pub continuation_markers: Vec<ContinuationMarker>,
    pub items: Vec<PdfTaggedItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfTaggedItem {
    pub element_id: String,
    pub role: PdfTaggedRole,
    pub fragment: Fragment,
    pub line_range: Option<(u32, u32)>,
    pub continuation_markers: Vec<ContinuationMarker>,
    pub artifact: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PdfTaggedBlockPlacement {
    Flow,
    DualDialogue { group_id: String, side: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfTaggedRole {
    Title,
    SceneHeading,
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitleOverflowPage {
    pub page_number: u32,
    pub display_page_number: Option<u32>,
    pub lines: Vec<PdfTitleOverflowLine>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitleOverflowLine {
    pub text: String,
    pub segments: Vec<PdfTitleOverflowSegment>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfTitleOverflowSegment {
    pub x: f32,
    pub fragments: Vec<PdfRenderFragment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderLine {
    pub text: String,
    pub counted: bool,
    pub centered: bool,
    pub kind: Option<PdfLineKind>,
    pub fragments: Vec<PdfRenderFragment>,
    pub dual: Option<PdfRenderDualLine>,
    pub scene_number: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfRenderFragment {
    pub text: String,
    pub styles: Vec<String>,
    pub actual_text: Option<String>,
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
    actual_text: Option<String>,
    tagged_span: bool,
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
    geometry: &LayoutGeometry,
) -> PdfRenderDocument {
    let title_page = if options.render_title_page && screenplay.imported_title_page.is_none() {
        TitlePage::from_screenplay(screenplay)
    } else {
        None
    };

    let rendered_title_page = title_page.as_ref().map(|title_page| PdfTitlePage {
            plain_title_all_caps: plain_title_uses_all_caps(&screenplay.metadata),
            blocks: title_page
                .blocks
                .iter()
                .map(|block| PdfTitleBlock {
                    kind: block.kind.into(),
                    region: block.region.into(),
                    lines: block.lines.clone(),
                })
                .collect(),
        });
    let title_overflow_pages = if options.render_title_page {
        if let Some(imported_title_page) = &screenplay.imported_title_page {
            render_imported_title_pages(imported_title_page, geometry)
        } else {
            title_page
                .as_ref()
                .map(|title_page| render_title_overflow_pages(title_page, geometry))
                .unwrap_or_default()
        }
    } else {
        Vec::new()
    };

    let mut body_pages: Vec<PdfRenderPage> = render_paginated_visual_pages_with_options(
        screenplay,
        VisualRenderOptions {
            render_continueds: options.render_continueds,
            render_title_page: options.render_title_page,
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
                scene_number: line.scene_number,
            })
            .collect(),
    })
    .collect();

    while body_pages
        .last()
        .is_some_and(|page: &PdfRenderPage| page.lines.iter().all(|line| line.text.is_empty()))
    {
        body_pages.pop();
    }

    PdfRenderDocument {
        title_page: rendered_title_page,
        title_overflow_pages,
        body_pages,
    }
}

pub(crate) fn render(screenplay: &Screenplay) -> Vec<u8> {
    render_with_options(screenplay, PdfRenderOptions::default())
}

pub(crate) fn render_with_options(screenplay: &Screenplay, options: PdfRenderOptions) -> Vec<u8> {
    let profile = ScreenplayLayoutProfile::from_screenplay(screenplay);
    let geometry = profile.to_pagination_geometry();
    let document = build_render_document(screenplay, options, &geometry);
    let mut tagged_document = build_tagged_document(screenplay, &geometry);
    if !options.render_title_page {
        tagged_document.title_page = None;
    }
    let document_language = document_language(&screenplay.metadata);
    let render_timestamp = current_render_timestamp();
    let mut structure_pages = Vec::new();
    if let Some(title_page) = &tagged_document.title_page {
        structure_pages.push(build_title_structure_page(title_page));
    }
    structure_pages.extend(
        document
            .title_overflow_pages
            .iter()
            .map(|_| PdfBodyStructPage {
                tagged_lines: Vec::new(),
            }),
    );
    structure_pages.extend(build_body_structure_pages(
        &tagged_document,
        &document.body_pages,
        &geometry,
    ));
    let fonts = EmbeddedFonts::new(&document);
    let body_page_count = document.body_pages.len() as i32;
    let title_overflow_page_count = document.title_overflow_pages.len() as i32;
    let page_count = body_page_count
        + title_overflow_page_count
        + i32::from(document.title_page.is_some());
    let struct_parent_keys = (0..page_count).collect::<Vec<_>>();
    let struct_element_plans = build_struct_element_plans(&structure_pages);
    let page_label_plans = build_page_label_plans(&document);

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let regular_font_ids = font_object_ids(3);
    let bold_font_ids = font_object_ids(9);
    let italic_font_ids = font_object_ids(15);
    let bold_italic_font_ids = font_object_ids(21);
    let document_info_id = Ref::new(27);
    let metadata_id = Ref::new(28);
    let page_ids = (0..page_count)
        .map(|index| Ref::new(29 + index))
        .collect::<Vec<_>>();
    let _page_rect = Rect::new(
        0.0,
        0.0,
        geometry.page_width * 72.0,
        geometry.page_height * 72.0,
    );

    let content_ids = (0..page_count)
        .map(|index| Ref::new(29 + page_count + index))
        .collect::<Vec<_>>();
    let mut next_object_id = 29 + (2 * page_count);
    let struct_tree_root_id = Ref::new(next_object_id);
    next_object_id += 1;
    let parent_tree_array_ids = (0..page_count)
        .map(|_| {
            let id = Ref::new(next_object_id);
            next_object_id += 1;
            id
        })
        .collect::<Vec<_>>();
    let struct_element_ids = struct_element_plans
        .iter()
        .map(|_| {
            let id = Ref::new(next_object_id);
            next_object_id += 1;
            id
        })
        .collect::<Vec<_>>();
    let page_label_ids = page_label_plans
        .iter()
        .map(|_| {
            let id = Ref::new(next_object_id);
            next_object_id += 1;
            id
        })
        .collect::<Vec<_>>();

    let mut pdf = Pdf::new();
    {
        let mut catalog = pdf.catalog(catalog_id);
        catalog.pages(page_tree_id);
        catalog.lang(TextStr(&document_language));
        catalog.mark_info().marked(true);
        catalog.metadata(metadata_id);
        catalog.viewer_preferences().display_doc_title(true);
        catalog.pair(Name(b"StructTreeRoot"), struct_tree_root_id);
        if !page_label_plans.is_empty() {
            let mut page_labels = catalog.page_labels();
            let mut nums = page_labels.nums();
            for (plan, label_id) in page_label_plans.iter().zip(page_label_ids.iter().copied()) {
                nums.insert(plan.page_index, label_id);
            }
        }
        catalog.finish();
    }
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_count);
    {
        let mut info = pdf.document_info(document_info_id);
        info.producer(TextStr(TOOL_IDENTITY));
        let render_date = pdf_date(render_timestamp);
        info.creation_date(render_date).modified_date(render_date);
        if let Some(author) = document_author(&screenplay.metadata) {
            info.author(TextStr(&author));
        }
        if let Some(title) = document_title(&screenplay.metadata) {
            info.title(TextStr(&title));
        }
    }
    let xmp_metadata =
        build_xmp_metadata(&screenplay.metadata, &document_language, render_timestamp);
    pdf.metadata(metadata_id, xmp_metadata.as_bytes());
    write_embedded_font_objects(&mut pdf, &fonts.regular, regular_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.bold, bold_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.italic, italic_font_ids);
    write_embedded_font_objects(&mut pdf, &fonts.bold_italic, bold_italic_font_ids);

    for (index, page_id) in page_ids.iter().copied().enumerate() {
        let mut page = pdf.page(page_id);
        page.parent(page_tree_id)
            .media_box(Rect::new(
                0.0,
                0.0,
                geometry.page_width * 72.0,
                geometry.page_height * 72.0,
            ))
            .contents(content_ids[index])
            .struct_parents(struct_parent_keys[index]);
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
    if let (Some(title_page), Some(tagged_title_page)) =
        (&document.title_page, &tagged_document.title_page)
    {
        pdf.stream(
            content_ids[content_index],
            &render_title_page_content(title_page, tagged_title_page, &fonts, &geometry),
        );
        content_index += 1;
    }

    for title_overflow_page in &document.title_overflow_pages {
        pdf.stream(
            content_ids[content_index],
            &render_title_overflow_page_content(title_overflow_page, &geometry, &fonts),
        );
        content_index += 1;
    }

    for body_page in &document.body_pages {
        pdf.stream(
            content_ids[content_index],
            &render_body_page_content(body_page, &geometry, &fonts, &profile),
        );
        content_index += 1;
    }

    {
        let mut struct_tree_root: pdf_writer::writers::StructTreeRoot<'_> =
            pdf.indirect(struct_tree_root_id).start();
        {
            let mut children = struct_tree_root.children();
            children.items(struct_element_ids.iter().copied());
        }
        {
            let mut parent_tree = struct_tree_root.parent_tree();
            let mut parent_tree_nums = parent_tree.nums();
            for (key, array_id) in struct_parent_keys
                .iter()
                .copied()
                .zip(parent_tree_array_ids.iter().copied())
            {
                parent_tree_nums.insert(key, array_id);
            }
        }
        struct_tree_root.parent_tree_next_key(struct_parent_keys.len() as i32);
        write_tagged_pdf_role_map(&mut struct_tree_root);
    }

    for (page_index, parent_ids) in
        build_parent_tree_entries(&structure_pages, &struct_element_plans, &struct_element_ids)
            .into_iter()
            .enumerate()
    {
        let mut parent_array = pdf.indirect(parent_tree_array_ids[page_index]).array();
        parent_array.items(parent_ids);
    }

    for (plan, struct_element_id) in struct_element_plans
        .iter()
        .zip(struct_element_ids.iter().copied())
    {
        let mut struct_element = pdf.struct_element(struct_element_id);
        let first_ref = plan
            .refs
            .first()
            .expect("expected at least one marked-content ref per struct element plan");
        struct_element
            .custom_kind(tagged_role_name(plan.role))
            .parent(struct_tree_root_id)
            .page(page_ids[first_ref.page_index]);

        if plan.refs.len() == 1 {
            struct_element
                .marked_content_child()
                .page(page_ids[first_ref.page_index])
                .marked_content_id(first_ref.mcid);
        } else {
            let mut children = struct_element.children();
            for marked_ref in &plan.refs {
                children
                    .marked_content_ref()
                    .page(page_ids[marked_ref.page_index])
                    .marked_content_id(marked_ref.mcid);
            }
        }
    }

    for (plan, page_label_id) in page_label_plans.iter().zip(page_label_ids.iter().copied()) {
        let mut page_label: pdf_writer::writers::PageLabel<'_> =
            pdf.indirect(page_label_id).start();
        match plan.style {
            PdfPageLabelStyle::Blank => {}
            PdfPageLabelStyle::LowerRoman { offset } => {
                page_label.style(NumberingStyle::LowerRoman).offset(offset);
            }
            PdfPageLabelStyle::Arabic { offset } => {
                page_label.style(NumberingStyle::Arabic).offset(offset);
            }
        }
    }

    pdf.finish()
}

fn write_tagged_pdf_role_map(struct_tree_root: &mut pdf_writer::writers::StructTreeRoot<'_>) {
    let mut role_map = struct_tree_root.role_map();
    role_map.insert(Name(b"Title"), StructRole::P);
    role_map.insert(Name(b"SceneHeading"), StructRole::H1);
    role_map.insert(Name(b"Action"), StructRole::P);
    role_map.insert(Name(b"Character"), StructRole::P);
    role_map.insert(Name(b"Dialogue"), StructRole::P);
    role_map.insert(Name(b"Parenthetical"), StructRole::P);
    role_map.insert(Name(b"Transition"), StructRole::P);
}

fn document_title(metadata: &Metadata) -> Option<String> {
    let title = metadata
        .get("title")
        .into_iter()
        .flatten()
        .map(ElementText::plain_text)
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

fn document_creators(metadata: &Metadata) -> Vec<String> {
    metadata
        .get("author")
        .into_iter()
        .flatten()
        .chain(metadata.get("authors").into_iter().flatten())
        .map(ElementText::plain_text)
        .filter(|author| !author.trim().is_empty())
        .collect()
}

fn document_author(metadata: &Metadata) -> Option<String> {
    let authors = document_creators(metadata);
    (!authors.is_empty()).then(|| authors.join(", "))
}

fn document_language(metadata: &Metadata) -> String {
    metadata
        .get("lang")
        .or_else(|| metadata.get("language"))
        .and_then(|values| {
            let language = values
                .iter()
                .map(ElementText::plain_text)
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .replace('_', "-");
            sanitize_language_tag(&language)
        })
        .unwrap_or_else(|| DEFAULT_DOCUMENT_LANGUAGE.to_string())
}

fn sanitize_language_tag(language: &str) -> Option<String> {
    let trimmed = language.trim();
    if trimmed.is_empty() {
        return None;
    }

    trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-')
        .then(|| trimmed.to_string())
}

fn build_xmp_metadata(
    metadata: &Metadata,
    document_language: &str,
    render_timestamp: OffsetDateTime,
) -> String {
    let title_entries = document_title(metadata).map(|title| {
        let escaped_title = escape_xml_text(&title);
        let escaped_language = escape_xml_text(document_language);
        format!(
            "<dc:title><rdf:Alt>\
             <rdf:li xml:lang=\"x-default\">{escaped_title}</rdf:li>\
             <rdf:li xml:lang=\"{escaped_language}\">{escaped_title}</rdf:li>\
             </rdf:Alt></dc:title>",
            escaped_title = escaped_title,
            escaped_language = escaped_language,
        )
    });
    let creator_entries = {
        let creators = document_creators(metadata);
        (!creators.is_empty()).then(|| {
            let creator_items = creators
                .into_iter()
                .map(|creator| format!("<rdf:li>{}</rdf:li>", escape_xml_text(&creator)))
                .collect::<String>();
            format!("<dc:creator><rdf:Seq>{creator_items}</rdf:Seq></dc:creator>")
        })
    };
    let escaped_language = escape_xml_text(document_language);
    let escaped_tool_identity = escape_xml_text(TOOL_IDENTITY);
    let render_timestamp = render_timestamp
        .format(&Rfc3339)
        .expect("expected RFC3339 render timestamp");

    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
           <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
             <rdf:Description rdf:about=\"\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\" \
         xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\" \
         xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\">\n\
               {title_entries}\
               {creator_entries}\
               <dc:language><rdf:Bag><rdf:li>{escaped_language}</rdf:li></rdf:Bag></dc:language>\n\
               <xmp:CreatorTool>{escaped_tool_identity}</xmp:CreatorTool>\n\
               <pdf:Producer>{escaped_tool_identity}</pdf:Producer>\n\
               <xmp:CreateDate>{render_timestamp}</xmp:CreateDate>\n\
               <xmp:ModifyDate>{render_timestamp}</xmp:ModifyDate>\n\
               <xmp:MetadataDate>{render_timestamp}</xmp:MetadataDate>\n\
             </rdf:Description>\n\
           </rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>",
        escaped_language = escaped_language,
        escaped_tool_identity = escaped_tool_identity,
        render_timestamp = render_timestamp,
        creator_entries = creator_entries.unwrap_or_default(),
        title_entries = title_entries.unwrap_or_default(),
    )
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn pdf_date(timestamp: OffsetDateTime) -> Date {
    Date::new(timestamp.year() as u16)
        .month(timestamp.month() as u8)
        .day(timestamp.day())
        .hour(timestamp.hour())
        .minute(timestamp.minute())
        .second(timestamp.second())
        .utc_offset_hour(0)
}

fn current_render_timestamp() -> OffsetDateTime {
    #[cfg(target_arch = "wasm32")]
    {
        // `wasm32-unknown-unknown` does not provide a wall-clock source through `std`.
        // Use a stable fallback so PDF generation still works in size-sensitive wasm builds.
        OffsetDateTime::UNIX_EPOCH
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        OffsetDateTime::now_utc()
    }
}

fn render_body_page_content(
    page: &PdfRenderPage,
    geometry: &LayoutGeometry,
    fonts: &EmbeddedFonts,
    profile: &ScreenplayLayoutProfile,
) -> Vec<u8> {
    let mut content = Content::new();
    let mut underlines = Vec::new();
    let mut next_mcid = 0i32;
    content.begin_text();
    content.set_font(FONT_REGULAR_NAME, BODY_TEXT_FONT_SIZE);
    let line_step = body_line_step_points(geometry);

    if let Some(display_page_number) = page.display_page_number {
        let page_number = format!("{display_page_number}.");
        render_artifact_runs(
            &mut content,
            fonts,
            &[ResolvedRun {
                actual_text: None,
                tagged_span: false,
                text: page_number,
                styles: StyleFlags::default(),
            }],
            page_number_x(display_page_number, geometry),
            page_number_y(geometry),
            BODY_TEXT_FONT_SIZE,
            &mut underlines,
            geometry,
        );
    }

    let body_top = first_body_line_y_for_page(page, geometry);
    for (index, line) in page.lines.iter().enumerate() {
        if line.text.is_empty() {
            continue;
        }

        let line_y = body_top - (index as f32 * line_step);
        if let Some(dual) = &line.dual {
            render_dual_body_line(
                &mut content,
                dual,
                geometry,
                fonts,
                line_y,
                &mut underlines,
                profile,
                &mut next_mcid,
            );
            continue;
        }

        // Scene number rendering: left BEFORE heading text, right AFTER heading text,
        // matching the word stream order of Final Draft reference PDFs.
        if let Some(scene_number) = &line.scene_number {
            let plain_run = [ResolvedRun {
                actual_text: None,
                tagged_span: false,
                text: scene_number.clone(),
                styles: StyleFlags::default(),
            }];
            // Left scene number first (at 0.75" from page left)
            let scene_y = line_y + SCENE_NUMBER_Y_OFFSET;
            render_artifact_runs(
                &mut content,
                fonts,
                &plain_run,
                geometry.scene_number_left * 72.0,
                scene_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
                geometry,
            );
            // Heading text in the middle
            let fragments = displayed_body_fragments(line, geometry);
            let defaults = default_body_line_styles(line.kind, profile);
            render_body_line_runs(
                &mut content,
                fonts,
                line.kind,
                line.kind
                    .and_then(tagged_role_for_line_kind)
                    .filter(|_| line.counted),
                &mut next_mcid,
                &resolve_runs(&fragments, defaults),
                body_line_left(line, geometry),
                line_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
                geometry,
            );
            // Right scene number last (right-aligned to scene_number_right)
            let scene_number_right_x = geometry.scene_number_right * 72.0;
            let right_x =
                scene_number_right_x - (scene_number.chars().count() as f32 * BODY_TEXT_CELL_WIDTH);
            render_artifact_runs(
                &mut content,
                fonts,
                &plain_run,
                right_x,
                scene_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
                geometry,
            );
        } else if !line.counted {
            render_artifact_runs(
                &mut content,
                fonts,
                &resolve_runs(
                    &displayed_body_fragments(line, geometry),
                    default_body_line_styles(line.kind, profile),
                ),
                body_line_left(line, geometry),
                line_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
                geometry,
            );
        } else {
            let fragments = displayed_body_fragments(line, geometry);
            let defaults = default_body_line_styles(line.kind, profile);
            render_body_line_runs(
                &mut content,
                fonts,
                line.kind,
                line.kind
                    .and_then(tagged_role_for_line_kind)
                    .filter(|_| line.counted),
                &mut next_mcid,
                &resolve_runs(&fragments, defaults),
                body_line_left(line, geometry),
                line_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
                geometry,
            );
        }
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
    profile: &ScreenplayLayoutProfile,
    next_mcid: &mut i32,
) {
    if let Some(left) = &dual.left {
        render_body_line_runs(
            content,
            fonts,
            Some(left.kind),
            tagged_role_for_line_kind(left.kind),
            next_mcid,
            &resolve_runs(
                &left.fragments,
                default_body_line_styles(Some(left.kind), profile),
            ),
            dual_side_left(left, geometry),
            line_y,
            BODY_TEXT_FONT_SIZE,
            underlines,
            geometry,
        );
    }

    if let Some(right) = &dual.right {
        render_body_line_runs(
            content,
            fonts,
            Some(right.kind),
            tagged_role_for_line_kind(right.kind),
            next_mcid,
            &resolve_runs(
                &right.fragments,
                default_body_line_styles(Some(right.kind), profile),
            ),
            dual_side_left(right, geometry),
            line_y,
            BODY_TEXT_FONT_SIZE,
            underlines,
            geometry,
        );
    }
}

fn render_artifact_runs(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    runs: &[ResolvedRun],
    line_left: f32,
    line_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
    geometry: &LayoutGeometry,
) {
    let is_page_number =
        line_y == page_number_y(geometry) && line_left >= page_number_x(10, geometry);
    if is_page_number {
        let mut marked_content = content.begin_marked_content_with_properties(Name(b"Artifact"));
        marked_content
            .properties()
            .artifact()
            .kind(ArtifactType::Pagination)
            .subtype(ArtifactSubtype::PageNumber)
            .attached([ArtifactAttachment::Top, ArtifactAttachment::Right]);
    } else {
        content.begin_marked_content(Name(b"Artifact"));
    }
    render_fixed_cell_runs(
        content, fonts, runs, None, line_left, line_y, font_size, underlines,
    );
    content.end_marked_content();
}

fn build_page_label_plans(document: &PdfRenderDocument) -> Vec<PdfPageLabelPlan> {
    let page_label_styles = document
        .title_page
        .iter()
        .map(|_| PdfPageLabelStyle::Blank)
        .chain(
            document
                .title_overflow_pages
                .iter()
                .map(|page| {
                    page.display_page_number
                        .map(|number| PdfPageLabelStyle::LowerRoman {
                            offset: number as i32,
                        })
                        .unwrap_or(PdfPageLabelStyle::Blank)
                }),
        )
        .chain(document.body_pages.iter().map(|page| {
            page.display_page_number
                .map(|number| PdfPageLabelStyle::Arabic {
                    offset: number as i32,
                })
                .unwrap_or(PdfPageLabelStyle::Blank)
        }))
        .collect::<Vec<_>>();

    if !page_label_styles
        .iter()
        .any(|style| !matches!(style, PdfPageLabelStyle::Blank))
    {
        return Vec::new();
    }

    let mut plans = Vec::new();
    let mut current_numbered_start = None;

    for (page_index, style) in page_label_styles.iter().copied().enumerate() {
        let page_index = page_index as i32;
        let should_start_new_plan = match (current_numbered_start, style) {
            (
                Some((start_page_index, PdfPageLabelStyle::LowerRoman { offset: start_offset })),
                PdfPageLabelStyle::LowerRoman { offset },
            )
            | (
                Some((start_page_index, PdfPageLabelStyle::Arabic { offset: start_offset })),
                PdfPageLabelStyle::Arabic { offset },
            ) => offset != start_offset + (page_index - start_page_index),
            (Some(_), PdfPageLabelStyle::Blank) => true,
            (Some(_), _) => true,
            (None, PdfPageLabelStyle::Blank) => plans.is_empty(),
            (None, _) => true,
        };

        if should_start_new_plan {
            plans.push(PdfPageLabelPlan { page_index, style });
        }

        current_numbered_start = match style {
            PdfPageLabelStyle::LowerRoman { .. } | PdfPageLabelStyle::Arabic { .. } => {
                Some((page_index, style))
            }
            PdfPageLabelStyle::Blank => None,
        };
    }

    plans
}

fn render_body_line_runs(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    kind: Option<PdfLineKind>,
    role: Option<PdfTaggedRole>,
    next_mcid: &mut i32,
    runs: &[ResolvedRun],
    line_left: f32,
    line_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
    _geometry: &LayoutGeometry,
) {
    let contd_suffix_start_cell = ordinary_character_contd_suffix_start_in_runs(kind, runs);
    if let Some(role) = role {
        content
            .begin_marked_content_with_properties(tagged_role_name(role))
            .properties()
            .identify(*next_mcid);
        render_fixed_cell_runs(
            content,
            fonts,
            runs,
            contd_suffix_start_cell,
            line_left,
            line_y,
            font_size,
            underlines,
        );
        content.end_marked_content();
        *next_mcid += 1;
    } else {
        render_fixed_cell_runs(
            content,
            fonts,
            runs,
            contd_suffix_start_cell,
            line_left,
            line_y,
            font_size,
            underlines,
        );
    }
}

fn render_fixed_cell_runs(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    runs: &[ResolvedRun],
    contd_suffix_start_cell: Option<usize>,
    line_left: f32,
    line_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
) {
    let mut cell_index = 0usize;

    for run in runs {
        if run.tagged_span {
            let mut marked_content = content.begin_marked_content_with_properties(Name(b"Span"));
            let mut properties = marked_content.properties();
            if let Some(actual_text) = &run.actual_text {
                properties.actual_text(TextStr(actual_text.as_str()));
            }
        }

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
                    line_left
                        + (cell_index as f32 * BODY_TEXT_CELL_WIDTH)
                        - ordinary_character_contd_suffix_x_offset(
                            cell_index,
                            contd_suffix_start_cell,
                        ),
                    line_y,
                ]);
                let encoded_character = font.encode_char(character);
                content.show(Str(&encoded_character));
            }
            cell_index += 1;
        }

        if run.styles.underline {
            let underline_cell_count = run.text.trim_end_matches(' ').chars().count();
            let underline_end_cell = run_start_cell + underline_cell_count;
            if underline_end_cell > run_start_cell {
                underlines.push(UnderlineSegment {
                    start_x: line_left + (run_start_cell as f32 * BODY_TEXT_CELL_WIDTH),
                    end_x: line_left + (underline_end_cell as f32 * BODY_TEXT_CELL_WIDTH),
                    y: line_y,
                });
            }
        }

        if run.tagged_span {
            content.end_marked_content();
        }
    }
}

fn body_line_step_points(geometry: &LayoutGeometry) -> f32 {
    geometry.calculate_line_step()
}

fn ordinary_character_contd_suffix_start_in_runs(
    kind: Option<PdfLineKind>,
    runs: &[ResolvedRun],
) -> Option<usize> {
    if kind != Some(PdfLineKind::Character) {
        return None;
    }

    let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
    continuation_suffix_start_cell(&text)
}

fn continuation_suffix_start_cell(text: &str) -> Option<usize> {
    text.find(" (CONT'D)")
        .or_else(|| text.find(" (CONT’D)"))
        .map(|byte_index| text[..byte_index].chars().count() + 1)
}

fn ordinary_character_contd_suffix_x_offset(
    cell_index: usize,
    contd_suffix_start_cell: Option<usize>,
) -> f32 {
    contd_suffix_start_cell
        .filter(|start_cell| cell_index >= *start_cell)
        .map(|_| ORDINARY_CHARACTER_CONTD_SUFFIX_X_ADJUSTMENT)
        .unwrap_or(0.0)
}

fn first_body_line_y(geometry: &LayoutGeometry) -> f32 {
    (geometry.page_height * 72.0) - (geometry.top_margin * 72.0) - 9.0
}

fn first_body_line_y_for_page(page: &PdfRenderPage, geometry: &LayoutGeometry) -> f32 {
    let mut y = first_body_line_y(geometry);
    if page_starts_with_split_contd_character(page) {
        y += body_line_step_points(geometry);
    }
    y
}

fn page_starts_with_split_contd_character(page: &PdfRenderPage) -> bool {
    page.lines
        .iter()
        .find(|line| !line.text.is_empty())
        .is_some_and(|line| !line.counted && matches!(line.kind, Some(PdfLineKind::Character)))
}

fn page_number_y(geometry: &LayoutGeometry) -> f32 {
    (geometry.page_height * 72.0) - (geometry.header_margin * 72.0) - 9.0
}

fn title_page_center_title_top_y(geometry: &LayoutGeometry) -> f32 {
    first_body_line_y(geometry)
        - (TITLE_TITLE_TOP_OFFSET_LINES_FROM_BODY_START * body_line_step_points(geometry))
}

fn title_page_center_meta_top_y(geometry: &LayoutGeometry) -> f32 {
    first_body_line_y(geometry)
        - (TITLE_META_TOP_OFFSET_LINES_FROM_BODY_START * body_line_step_points(geometry))
}

fn title_page_bottom_top_y(geometry: &LayoutGeometry) -> f32 {
    (geometry.bottom_margin * 72.0)
        + (TITLE_BOTTOM_TOP_OFFSET_LINES_FROM_BOTTOM_MARGIN * body_line_step_points(geometry))
}

fn page_number_x(display_page_number: u32, geometry: &LayoutGeometry) -> f32 {
    let right_inches = geometry.page_width - 1.4375;
    let right_pts = right_inches * 72.0;
    let extra_digits = display_page_number.to_string().len().saturating_sub(1) as f32;
    right_pts - (extra_digits * BODY_TEXT_CELL_WIDTH)
}

fn title_page_number_text_x(page_number: &str) -> f32 {
    const TITLE_PAGE_HEADER_PAGE_NUMBER_RIGHT_TAB_INCHES: f32 = 7.25;

    let right_pts = TITLE_PAGE_HEADER_PAGE_NUMBER_RIGHT_TAB_INCHES * 72.0;
    right_pts - (page_number.chars().count() as f32 * BODY_TEXT_CELL_WIDTH)
}

fn lower_roman(number: u32) -> String {
    let values = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];

    let mut remaining = number;
    let mut result = String::new();
    for (value, numeral) in values {
        while remaining >= value {
            result.push_str(numeral);
            remaining -= value;
        }
    }
    result
}

fn render_title_page_content(
    title_page: &PdfTitlePage,
    tagged_title_page: &PdfTaggedTitlePage,
    fonts: &EmbeddedFonts,
    geometry: &LayoutGeometry,
) -> Vec<u8> {
    let mut content = Content::new();
    let mut underlines = Vec::new();
    let mut next_mcid = 0i32;
    content.begin_text();
    content.set_font(FONT_REGULAR_NAME, TITLE_META_FONT_SIZE);

    render_title_page_region(
        &mut content,
        title_page,
        tagged_title_page,
        fonts,
        PdfTitleBlockRegion::CenterTitle,
        title_page_center_title_top_y(geometry),
        TITLE_FONT_SIZE,
        &mut underlines,
        &mut next_mcid,
        geometry,
    );
    render_title_page_region(
        &mut content,
        title_page,
        tagged_title_page,
        fonts,
        PdfTitleBlockRegion::CenterMeta,
        title_page_center_meta_top_y(geometry),
        TITLE_META_FONT_SIZE,
        &mut underlines,
        &mut next_mcid,
        geometry,
    );
    render_title_page_bottom_regions(
        &mut content,
        title_page,
        tagged_title_page,
        fonts,
        &mut underlines,
        &mut next_mcid,
        geometry,
    );

    content.end_text();
    render_underlines(&mut content, &underlines);
    content.finish().to_vec()
}

fn render_title_page_region(
    content: &mut Content,
    title_page: &PdfTitlePage,
    tagged_title_page: &PdfTaggedTitlePage,
    fonts: &EmbeddedFonts,
    region: PdfTitleBlockRegion,
    top_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
    next_mcid: &mut i32,
    geometry: &LayoutGeometry,
) {
    let mut line_index = 0usize;

    for block in title_page
        .blocks
        .iter()
        .filter(|block| block.region == region)
    {
        let tagged_block = tagged_title_page
            .blocks
            .iter()
            .find(|candidate| candidate.kind == block.kind && candidate.region == block.region)
            .expect("missing tagged title-page block for rendered region block");
        for line in &block.lines {
            let text = line.plain_text();
            let y = top_y - (line_index as f32 * TITLE_BLOCK_LINE_STEP);
            render_title_page_line_runs(
                content,
                tagged_block,
                fonts,
                &resolve_runs(
                    &title_page_fragments(title_page, block.kind, line),
                    default_title_page_styles(block.kind, line),
                ),
                title_page_line_left(&text, region, geometry),
                y,
                font_size,
                underlines,
                next_mcid,
                geometry,
            );
            line_index += 1;
        }
        line_index += title_page_block_gap_after(block.kind);
    }
}

fn render_title_page_bottom_regions(
    content: &mut Content,
    title_page: &PdfTitlePage,
    tagged_title_page: &PdfTaggedTitlePage,
    fonts: &EmbeddedFonts,
    underlines: &mut Vec<UnderlineSegment>,
    next_mcid: &mut i32,
    geometry: &LayoutGeometry,
) {
    let left_lines = title_page
        .blocks
        .iter()
        .filter(|block| block.region == PdfTitleBlockRegion::BottomLeft)
        .flat_map(|block| {
            let tagged_block = tagged_title_page
                .blocks
                .iter()
                .find(|candidate| candidate.kind == block.kind && candidate.region == block.region)
                .expect("missing tagged title-page block for bottom-left region block");
            block
                .lines
                .iter()
                .map(move |line| (block.kind, tagged_block, line))
        })
        .collect::<Vec<_>>();
    let right_lines = title_page
        .blocks
        .iter()
        .filter(|block| block.region == PdfTitleBlockRegion::BottomRight)
        .flat_map(|block| {
            let tagged_block = tagged_title_page
                .blocks
                .iter()
                .find(|candidate| candidate.kind == block.kind && candidate.region == block.region)
                .expect("missing tagged title-page block for bottom-right region block");
            block
                .lines
                .iter()
                .map(move |line| (block.kind, tagged_block, line))
        })
        .collect::<Vec<_>>();

    let max_lines = left_lines.len().max(right_lines.len());
    render_title_page_bottom_region_lines(
        content,
        fonts,
        PdfTitleBlockRegion::BottomLeft,
        &left_lines,
        max_lines,
        underlines,
        next_mcid,
        geometry,
    );
    render_title_page_bottom_region_lines(
        content,
        fonts,
        PdfTitleBlockRegion::BottomRight,
        &right_lines,
        max_lines,
        underlines,
        next_mcid,
        geometry,
    );
}

fn render_title_page_bottom_region_lines(
    content: &mut Content,
    fonts: &EmbeddedFonts,
    region: PdfTitleBlockRegion,
    lines: &[(PdfTitleBlockKind, &PdfTaggedTitleBlock, &ElementText)],
    max_lines: usize,
    underlines: &mut Vec<UnderlineSegment>,
    next_mcid: &mut i32,
    geometry: &LayoutGeometry,
) {
    let line_offset = max_lines.saturating_sub(lines.len()) as f32 * TITLE_BLOCK_LINE_STEP;
    let top_y = title_page_bottom_top_y(geometry);

    for (line_index, (kind, tagged_block, line)) in lines.iter().enumerate() {
        let text = line.plain_text();
        let y = top_y - line_offset - (line_index as f32 * TITLE_BLOCK_LINE_STEP);
        render_title_page_line_runs(
            content,
            tagged_block,
            fonts,
            &resolve_runs(
                &title_page_fragments_for_kind(*kind, line),
                default_title_page_styles(*kind, line),
            ),
            title_page_bottom_line_left(&text, region, geometry),
            y,
            TITLE_META_FONT_SIZE,
            underlines,
            next_mcid,
            geometry,
        );
    }
}

fn render_title_page_line_runs(
    content: &mut Content,
    tagged_block: &PdfTaggedTitleBlock,
    fonts: &EmbeddedFonts,
    runs: &[ResolvedRun],
    line_left: f32,
    line_y: f32,
    font_size: f32,
    underlines: &mut Vec<UnderlineSegment>,
    next_mcid: &mut i32,
    _geometry: &LayoutGeometry,
) {
    if tagged_block.artifact {
        render_fixed_cell_runs(
            content, fonts, runs, None, line_left, line_y, font_size, underlines,
        );
        return;
    }

    content
        .begin_marked_content_with_properties(tagged_role_name(tagged_block.role))
        .properties()
        .identify(*next_mcid);
    render_fixed_cell_runs(
        content, fonts, runs, None, line_left, line_y, font_size, underlines,
    );
    content.end_marked_content();
    *next_mcid += 1;
}

fn title_page_bottom_line_left(
    text: &str,
    region: PdfTitleBlockRegion,
    geometry: &LayoutGeometry,
) -> f32 {
    match region {
        PdfTitleBlockRegion::BottomRight => title_page_bottom_right_left(text, geometry),
        _ => title_page_line_left(text, region, geometry),
    }
}

fn title_page_bottom_right_left(text: &str, geometry: &LayoutGeometry) -> f32 {
    let right_edge_points = (geometry.page_width - 1.0) * 72.0;
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

fn title_page_line_left(text: &str, region: PdfTitleBlockRegion, geometry: &LayoutGeometry) -> f32 {
    let width = text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;

    match region {
        PdfTitleBlockRegion::CenterTitle | PdfTitleBlockRegion::CenterMeta => {
            ((geometry.page_width * 72.0 - width) / 2.0).max(72.0)
        }
        PdfTitleBlockRegion::BottomLeft => 72.0,
        PdfTitleBlockRegion::BottomRight => ((geometry.page_width * 72.0) - 72.0 - width).max(72.0),
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
            - parenthetical_hang_offset_points(
                line.kind,
                rendered_body_line_text(line, geometry),
                geometry,
            );
    }

    let left_in = line
        .kind
        .map(|k| element_left_inches(k, geometry))
        .unwrap_or(geometry.action_left);
    let right_in = line
        .kind
        .map(|k| element_right_inches(k, geometry))
        .unwrap_or(geometry.action_right);

    let left = left_in * 72.0;
    let right = right_in * 72.0;
    let rendered_width = line.text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;
    let available_width = right - left;

    left + ((available_width - rendered_width) / 2.0).max(0.0)
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

fn snap_character_left_inches(left: f32) -> f32 {
    (left * 8.0).round() / 8.0
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
    left - parenthetical_hang_offset_points(Some(side.kind), &side.text, geometry)
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
        PdfLineKind::Character => snap_character_left_inches(geometry.character_left),
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

fn element_right_inches(kind: PdfLineKind, geometry: &LayoutGeometry) -> f32 {
    match kind {
        PdfLineKind::Action | PdfLineKind::SceneHeading => geometry.action_right,
        PdfLineKind::ColdOpening => geometry.cold_opening_right,
        PdfLineKind::NewAct => geometry.new_act_right,
        PdfLineKind::EndOfAct => geometry.end_of_act_right,
        PdfLineKind::Character => geometry.character_right,
        PdfLineKind::Dialogue => geometry.dialogue_right,
        PdfLineKind::Parenthetical => geometry.parenthetical_right,
        PdfLineKind::Transition => geometry.transition_right,
        PdfLineKind::Lyric => geometry.lyric_right,
        PdfLineKind::DualDialogueLeft => geometry.dual_dialogue_left_right,
        PdfLineKind::DualDialogueRight => geometry.dual_dialogue_right_right,
        PdfLineKind::DualDialogueCharacterLeft => geometry.dual_dialogue_left_character_right,
        PdfLineKind::DualDialogueCharacterRight => geometry.dual_dialogue_right_character_right,
        PdfLineKind::DualDialogueParentheticalLeft => {
            geometry.dual_dialogue_left_parenthetical_right
        }
        PdfLineKind::DualDialogueParentheticalRight => {
            geometry.dual_dialogue_right_parenthetical_right
        }
    }
}

fn element_first_indent_inches(kind: PdfLineKind, geometry: &LayoutGeometry) -> f32 {
    match kind {
        PdfLineKind::Action | PdfLineKind::SceneHeading => geometry.action_first_indent,
        PdfLineKind::ColdOpening => geometry.cold_opening_first_indent,
        PdfLineKind::NewAct => geometry.new_act_first_indent,
        PdfLineKind::EndOfAct => geometry.end_of_act_first_indent,
        PdfLineKind::Character => geometry.character_first_indent,
        PdfLineKind::Dialogue => geometry.dialogue_first_indent,
        PdfLineKind::Parenthetical => geometry.parenthetical_first_indent,
        PdfLineKind::Transition => geometry.transition_first_indent,
        PdfLineKind::Lyric => geometry.lyric_first_indent,
        PdfLineKind::DualDialogueLeft => geometry.dual_dialogue_left_first_indent,
        PdfLineKind::DualDialogueRight => geometry.dual_dialogue_right_first_indent,
        PdfLineKind::DualDialogueCharacterLeft => {
            geometry.dual_dialogue_left_character_first_indent
        }
        PdfLineKind::DualDialogueCharacterRight => {
            geometry.dual_dialogue_right_character_first_indent
        }
        PdfLineKind::DualDialogueParentheticalLeft => {
            geometry.dual_dialogue_left_parenthetical_first_indent
        }
        PdfLineKind::DualDialogueParentheticalRight => {
            geometry.dual_dialogue_right_parenthetical_first_indent
        }
    }
}

fn parenthetical_hang_offset_points(
    kind: Option<PdfLineKind>,
    text: &str,
    geometry: &LayoutGeometry,
) -> f32 {
    if hangs_opening_parenthesis(kind, text) {
        return kind
            .map(|kind| element_first_indent_inches(kind, geometry).abs() * 72.0)
            .unwrap_or(BODY_TEXT_CELL_WIDTH);
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
                Name(b"AAAAAA+CourierPrime-Regular"),
                b"Courier Prime",
                COURIER_PRIME_REGULAR_BYTES,
                Name(b"CourierPrime-Regular-UTF16"),
            ),
            bold: EmbeddedFont::new(
                document,
                FONT_BOLD_NAME,
                Name(b"AAAAAB+CourierPrime-Bold"),
                b"Courier Prime",
                COURIER_PRIME_BOLD_BYTES,
                Name(b"AAAAAB+CourierPrime-Bold-UTF16"),
            ),
            italic: EmbeddedFont::new(
                document,
                FONT_ITALIC_NAME,
                Name(b"AAAAAC+CourierPrime-Italic"),
                b"Courier Prime",
                COURIER_PRIME_ITALIC_BYTES,
                Name(b"AAAAAC+CourierPrime-Italic-UTF16"),
            ),
            bold_italic: EmbeddedFont::new(
                document,
                FONT_BOLD_ITALIC_NAME,
                Name(b"AAAAAD+CourierPrime-BoldItalic"),
                b"Courier Prime",
                COURIER_PRIME_BOLD_ITALIC_BYTES,
                Name(b"AAAAAD+CourierPrime-BoldItalic-UTF16"),
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
                let text = line.plain_text();
                chars.extend(text.chars());
                if block.kind == PdfTitleBlockKind::Title && title_page.plain_title_all_caps {
                    chars.extend(text.to_ascii_uppercase().chars());
                }
            }
        }
    }

    for page in &document.title_overflow_pages {
        if let Some(display_page_number) = page.display_page_number {
            chars.extend(lower_roman(display_page_number).chars());
        }
        for line in &page.lines {
            chars.extend(line.text.chars());
        }
    }

    for page in &document.body_pages {
        if let Some(display_page_number) = page.display_page_number {
            chars.extend(format!("{display_page_number}.").chars());
        }

        for line in &page.lines {
            chars.extend(rendered_line_chars(line));
            if let Some(scene_number) = &line.scene_number {
                chars.extend(scene_number.chars());
            }
        }
    }

    chars
}

fn render_imported_title_pages(
    imported_title_page: &crate::ImportedTitlePage,
    geometry: &LayoutGeometry,
) -> Vec<PdfTitleOverflowPage> {
    let renders_page_numbers = imported_title_page.header_footer.header_visible
        && imported_title_page.header_footer.header_has_page_number;
    let first_title_page_number = imported_title_page.header_footer.starting_page.unwrap_or(1);

    imported_title_page
        .pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let header_allows_this_page = index > 0 || imported_title_page.header_footer.header_first_page;
            PdfTitleOverflowPage {
                page_number: index as u32 + 1,
                display_page_number: (renders_page_numbers && header_allows_this_page)
                    .then_some(first_title_page_number + index as u32),
                lines: render_imported_title_overflow_page_lines(&page.paragraphs, geometry),
            }
        })
        .collect()
}

fn render_title_overflow_pages(
    title_page: &TitlePage,
    geometry: &LayoutGeometry,
) -> Vec<PdfTitleOverflowPage> {
    title_page
        .frontmatter
        .iter()
        .enumerate()
        .map(|(index, page)| PdfTitleOverflowPage {
            page_number: index as u32 + 2,
            display_page_number: None,
            lines: page
                .paragraphs
                .iter()
                .flat_map(|paragraph| {
                    let alignment = match paragraph.alignment {
                        crate::title_page::FrontmatterAlignment::Center => {
                            ImportedTitlePageAlignment::Center
                        }
                        crate::title_page::FrontmatterAlignment::Left => {
                            ImportedTitlePageAlignment::Left
                        }
                    };
                    render_imported_title_overflow_paragraph(
                        &crate::ImportedTitlePageParagraph {
                            text: paragraph.text.clone(),
                            alignment,
                            left_indent: Some(1.5),
                            space_before: None,
                            tab_stops: Vec::new(),
                        },
                        1.5,
                        geometry,
                    )
                })
                .collect(),
        })
        .collect()
}

fn render_imported_title_overflow_page_lines(
    paragraphs: &[crate::ImportedTitlePageParagraph],
    geometry: &LayoutGeometry,
) -> Vec<PdfTitleOverflowLine> {
    let page_left = paragraphs
        .iter()
        .filter(|paragraph| !paragraph.text.plain_text().trim().is_empty())
        .filter_map(|paragraph| paragraph.left_indent)
        .fold(None, |current: Option<f32>, indent| {
            Some(current.map_or(indent, |value| value.min(indent)))
        })
        .unwrap_or(1.5);

    let mut lines = Vec::new();
    for paragraph in paragraphs {
        lines.extend(
            std::iter::repeat_with(|| PdfTitleOverflowLine {
                text: String::new(),
                segments: Vec::new(),
            })
            .take(title_page_space_before_lines(paragraph.space_before)),
        );
        lines.extend(render_imported_title_overflow_paragraph(
            paragraph, page_left, geometry,
        ));
    }
    lines
}

fn render_imported_title_overflow_paragraph(
    paragraph: &crate::ImportedTitlePageParagraph,
    page_left: f32,
    _geometry: &LayoutGeometry,
) -> Vec<PdfTitleOverflowLine> {
    let display_text = imported_title_display_text(&paragraph.text);

    if display_text.plain_text().trim().is_empty() {
        return vec![PdfTitleOverflowLine {
            text: String::new(),
            segments: Vec::new(),
        }];
    }

    if !paragraph.tab_stops.is_empty() && display_text.plain_text().contains('\t') {
        return render_tabbed_title_overflow_paragraph(&display_text, paragraph, page_left);
    }

    let left = paragraph.left_indent.unwrap_or(page_left);
    let right = 7.5;
    let width_chars = width_chars_between(left, right);
    let wrapped_lines = wrap_element_text_to_lines(&display_text, width_chars);

    wrapped_lines
        .into_iter()
        .map(|fragments| {
            let text = fragments_text(&fragments);
            let x = match paragraph.alignment {
                ImportedTitlePageAlignment::Center => {
                    let width = text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;
                    let left_points = left * 72.0;
                    let right_points = right * 72.0;
                    left_points + ((right_points - left_points - width) / 2.0).max(0.0)
                }
                ImportedTitlePageAlignment::Right => {
                    let width = text.chars().count() as f32 * BODY_TEXT_CELL_WIDTH;
                    let right_points = right * 72.0;
                    (right_points - width).max(left * 72.0)
                }
                _ => left * 72.0,
            };

            PdfTitleOverflowLine {
                text,
                segments: vec![PdfTitleOverflowSegment { x, fragments }],
            }
        })
        .collect()
}

fn title_page_space_before_lines(space_before: Option<f32>) -> usize {
    (space_before.unwrap_or(0.0) / 12.0)
        .round()
        .max(0.0) as usize
}

fn render_tabbed_title_overflow_paragraph(
    display_text: &ElementText,
    paragraph: &crate::ImportedTitlePageParagraph,
    page_left: f32,
) -> Vec<PdfTitleOverflowLine> {
    let segments = split_fragments_on_tabs(&element_text_to_fragments(display_text));
    if segments.is_empty() {
        return vec![PdfTitleOverflowLine {
            text: String::new(),
            segments: Vec::new(),
        }];
    }

    let prefix_segments = &segments[..segments.len() - 1];
    let description = &segments[segments.len() - 1];
    let prefix_fragments = prefix_segments
        .iter()
        .flat_map(|segment| segment.clone())
        .collect::<Vec<_>>();
    let prefix_text = fragments_text(&prefix_fragments);
    let description_left =
        advance_through_tab_stops(page_left, prefix_segments, &paragraph.tab_stops);
    let description_lines = wrap_fragments_to_lines(description, width_chars_between(description_left, 7.5));

    let mut lines = Vec::new();
    if let Some(first_description_line) = description_lines.first() {
        let mut segments = Vec::new();
        if !prefix_fragments.is_empty() {
            segments.push(PdfTitleOverflowSegment {
                x: page_left * 72.0,
                fragments: prefix_fragments,
            });
        }
        segments.push(PdfTitleOverflowSegment {
            x: description_left * 72.0,
            fragments: first_description_line.clone(),
        });
        lines.push(PdfTitleOverflowLine {
            text: format!("{prefix_text}{}", fragments_text(first_description_line)),
            segments,
        });
    }

    for description_line in description_lines.into_iter().skip(1) {
        lines.push(PdfTitleOverflowLine {
            text: fragments_text(&description_line),
            segments: vec![PdfTitleOverflowSegment {
                x: description_left * 72.0,
                fragments: description_line,
            }],
        });
    }

    lines
}

fn imported_title_display_text(text: &ElementText) -> ElementText {
    match text {
        ElementText::Plain(text) => ElementText::Plain(text.clone()),
        ElementText::Styled(runs) => ElementText::Styled(
            runs.iter()
                .map(|run| {
                    let display_text = if run
                        .text_style
                        .iter()
                        .any(|style| style.eq_ignore_ascii_case("AllCaps"))
                    {
                        run.content.to_ascii_uppercase()
                    } else {
                        run.content.clone()
                    };
                    crate::TextRun {
                        content: display_text,
                        text_style: run.text_style.clone(),
                    }
                })
                .collect(),
        ),
    }
}

fn advance_through_tab_stops(
    page_left: f32,
    segments_before_description: &[Vec<PdfRenderFragment>],
    tab_stops: &[ImportedTitlePageTabStop],
) -> f32 {
    let mut current = page_left;
    let mut sorted_tab_positions = tab_stops.iter().map(|tab_stop| tab_stop.position).collect::<Vec<_>>();
    sorted_tab_positions.sort_by(|left, right| left.partial_cmp(right).unwrap());

    for segment in segments_before_description {
        current += (fragments_text(segment).chars().count() as f32 * BODY_TEXT_CELL_WIDTH) / 72.0;
        if let Some(next_tab) = sorted_tab_positions.iter().copied().find(|position| *position > current) {
            current = next_tab;
        }
    }

    current
}

fn split_fragments_on_tabs(fragments: &[PdfRenderFragment]) -> Vec<Vec<PdfRenderFragment>> {
    let mut segments = vec![Vec::new()];

    for fragment in fragments {
        let mut parts = fragment.text.split('\t').peekable();
        while let Some(part) = parts.next() {
            if !part.is_empty() {
                segments
                    .last_mut()
                    .expect("expected tab segment")
                    .push(PdfRenderFragment {
                        actual_text: fragment.actual_text.clone(),
                        text: part.to_string(),
                        styles: fragment.styles.clone(),
                    });
            }
            if parts.peek().is_some() {
                segments.push(Vec::new());
            }
        }
    }

    segments
}

fn width_chars_between(left: f32, right: f32) -> usize {
    (((right - left) * 72.0) / BODY_TEXT_CELL_WIDTH).floor().max(1.0) as usize
}

fn wrap_element_text_to_lines(text: &ElementText, width_chars: usize) -> Vec<Vec<PdfRenderFragment>> {
    wrap_fragments_to_lines(&element_text_to_fragments(text), width_chars)
}

fn wrap_fragments_to_lines(
    fragments: &[PdfRenderFragment],
    width_chars: usize,
) -> Vec<Vec<PdfRenderFragment>> {
    let plain_text = fragments_text(fragments);
    if plain_text.is_empty() {
        return vec![Vec::new()];
    }

    if fragments.iter().any(|fragment| !fragment.styles.is_empty()) {
        let styled_text = StyledText {
            plain_text,
            runs: fragments
                .iter()
                .map(|fragment| StyledRun {
                    text: fragment.text.clone(),
                    styles: fragment.styles.clone(),
                })
                .collect(),
        };
        return wrap_styled_text_for_element(
            &styled_text,
            &WrapConfig::with_exact_width_chars(width_chars),
        )
        .into_iter()
        .map(|line| {
            line.fragments
                .into_iter()
                .map(|fragment| PdfRenderFragment {
                    actual_text: None,
                    text: fragment.text,
                    styles: fragment.styles,
                })
                .collect()
        })
        .collect();
    }

    wrap_text_for_element(&plain_text, &WrapConfig::with_exact_width_chars(width_chars))
        .into_iter()
        .map(|text| {
            vec![PdfRenderFragment {
                actual_text: None,
                text,
                styles: Vec::new(),
            }]
        })
        .collect()
}

fn element_text_to_fragments(text: &ElementText) -> Vec<PdfRenderFragment> {
    match text {
        ElementText::Plain(text) => vec![PdfRenderFragment {
            actual_text: None,
            text: text.clone(),
            styles: Vec::new(),
        }],
        ElementText::Styled(runs) => runs
            .iter()
            .map(|run| PdfRenderFragment {
                actual_text: None,
                text: run.content.clone(),
                styles: {
                    let mut styles = run.text_style.iter().cloned().collect::<Vec<_>>();
                    styles.sort();
                    styles
                },
            })
            .collect(),
    }
}

fn fragments_text(fragments: &[PdfRenderFragment]) -> String {
    fragments.iter().map(|fragment| fragment.text.as_str()).collect()
}

fn render_title_overflow_page_content(
    page: &PdfTitleOverflowPage,
    geometry: &LayoutGeometry,
    fonts: &EmbeddedFonts,
) -> Vec<u8> {
    let mut content = Content::new();
    let mut underlines = Vec::new();
    content.begin_text();
    content.set_font(FONT_REGULAR_NAME, BODY_TEXT_FONT_SIZE);
    let line_step = body_line_step_points(geometry);
    let body_top = first_body_line_y(geometry);

    if let Some(display_page_number) = page.display_page_number {
        let page_number = lower_roman(display_page_number);
        render_artifact_runs(
            &mut content,
            fonts,
            &[ResolvedRun {
                actual_text: None,
                tagged_span: false,
                text: page_number.clone(),
                styles: StyleFlags::default(),
            }],
            title_page_number_text_x(&page_number),
            page_number_y(geometry),
            BODY_TEXT_FONT_SIZE,
            &mut underlines,
            geometry,
        );
    }

    for (index, line) in page.lines.iter().enumerate() {
        if line.segments.is_empty() {
            continue;
        }

        let line_y = body_top - (index as f32 * line_step);
        for segment in &line.segments {
            render_fixed_cell_runs(
                &mut content,
                fonts,
                &resolve_runs(&segment.fragments, StyleFlags::default()),
                None,
                segment.x,
                line_y,
                BODY_TEXT_FONT_SIZE,
                &mut underlines,
            );
        }
    }

    content.end_text();
    render_underlines(&mut content, &underlines);
    content.finish().to_vec()
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
        ElementText::Plain(text) => {
            let displayed_text = if kind == PdfTitleBlockKind::Title && plain_title_all_caps {
                text.to_ascii_uppercase()
            } else {
                text.clone()
            };
            vec![PdfRenderFragment {
                actual_text: None,
                text: displayed_text,
                styles: Vec::new(),
            }]
        }
        ElementText::Styled(runs) => runs
            .iter()
            .map(|run| PdfRenderFragment {
                actual_text: None,
                text: if kind == PdfTitleBlockKind::Title
                    && run.text_style.iter().any(|style| style == "AllCaps")
                {
                    run.content.to_ascii_uppercase()
                } else {
                    run.content.clone()
                },
                styles: sorted_run_styles(run.text_style.iter().cloned()),
            })
            .collect(),
    }
}

pub(crate) fn build_tagged_document(
    screenplay: &Screenplay,
    geometry: &LayoutGeometry,
) -> PdfTaggedDocument {
    let title_page = screenplay.imported_title_page.is_none().then(|| {
        TitlePage::from_screenplay(screenplay).map(|title_page| PdfTaggedTitlePage {
            blocks: title_page
                .blocks
                .into_iter()
                .enumerate()
                .map(|(index, block)| PdfTaggedTitleBlock {
                    id: format!("title-page-{:02}-{:02}", index + 1, block.kind.sort_order()),
                    kind: block.kind.into(),
                    role: PdfTaggedRole::Title,
                    region: block.region.into(),
                    lines: block.lines,
                    artifact: false,
                })
                .collect(),
        })
    }).flatten();

    let paginated = PaginatedScreenplay::from_screenplay(
        "screenplay",
        screenplay,
        geometry.lines_per_page,
        pdf_pagination_scope(screenplay),
    );
    let body_pages = paginated
        .pages
        .into_iter()
        .map(|page| PdfTaggedPage {
            page_number: page.metadata.number,
            body_page_number: page.metadata.body_page_number,
            blocks: page
                .blocks
                .into_iter()
                .map(|block| PdfTaggedBlock {
                    id: block.id,
                    source_block_id: block.source_block_id,
                    placement: block.placement.into(),
                    fragment: block.fragment,
                    continuation_markers: block.continuation_markers,
                    items: block
                        .item_ids
                        .iter()
                        .map(|item_id| tagged_item_for_page_item(item_id, &page.items))
                        .collect(),
                })
                .collect(),
        })
        .collect();

    PdfTaggedDocument {
        title_page,
        body_pages,
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
                actual_text: fragment.actual_text.clone(),
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
            actual_text: fragment.actual_text.clone(),
            tagged_span: !fragment.styles.is_empty() || fragment.actual_text.is_some(),
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

fn default_body_line_styles(
    kind: Option<PdfLineKind>,
    profile: &ScreenplayLayoutProfile,
) -> StyleFlags {
    let mut flags = StyleFlags::default();
    if let Some(kind) = kind {
        use PdfLineKind as PK;
        let style = match kind {
            PK::Action => Some(&profile.styles.action),
            PK::ColdOpening => Some(&profile.styles.cold_opening),
            PK::NewAct => Some(&profile.styles.new_act),
            PK::EndOfAct => Some(&profile.styles.end_of_act),
            PK::SceneHeading => Some(&profile.styles.scene_heading),
            PK::Character => Some(&profile.styles.character),
            PK::Dialogue => Some(&profile.styles.dialogue),
            PK::Parenthetical => Some(&profile.styles.parenthetical),
            PK::Transition => Some(&profile.styles.transition),
            PK::Lyric => Some(&profile.styles.lyric),
            PK::DualDialogueLeft | PK::DualDialogueRight => Some(&profile.styles.dialogue),
            PK::DualDialogueCharacterLeft | PK::DualDialogueCharacterRight => {
                Some(&profile.styles.character)
            }
            PK::DualDialogueParentheticalLeft | PK::DualDialogueParentheticalRight => {
                Some(&profile.styles.parenthetical)
            }
        };
        if let Some(s) = style {
            flags.bold = s.bold;
            flags.italic = s.italic;
            flags.underline = s.underline;
        }
    }
    flags
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

impl From<BlockPlacement> for PdfTaggedBlockPlacement {
    fn from(value: BlockPlacement) -> Self {
        match value {
            BlockPlacement::Flow => Self::Flow,
            BlockPlacement::DualDialogue { group_id, side } => {
                Self::DualDialogue { group_id, side }
            }
        }
    }
}

impl PdfTaggedRole {
    fn from_page_item_kind(kind: &str) -> Self {
        match kind {
            "Scene Heading" => Self::SceneHeading,
            "Character" => Self::Character,
            "Dialogue" => Self::Dialogue,
            "Parenthetical" => Self::Parenthetical,
            "Transition" => Self::Transition,
            "Action" | "Lyric" | "Cold Opening" | "New Act" | "End of Act" => Self::Action,
            other => panic!("unsupported tagged PDF role kind: {other}"),
        }
    }
}

impl TitlePageBlockKind {
    fn sort_order(self) -> u8 {
        match self {
            Self::Title => 1,
            Self::Credit => 2,
            Self::Author => 3,
            Self::Source => 4,
            Self::Contact => 5,
            Self::Draft => 6,
            Self::DraftDate => 7,
        }
    }
}

fn pdf_pagination_scope(screenplay: &Screenplay) -> PaginationScope {
    if let Some(title_page) = TitlePage::from_screenplay(screenplay) {
        let count = frontmatter_count(screenplay).unwrap_or_else(|| title_page.total_page_count());
        let first_page_number = if screenplay.metadata.contains_key("frontmatter-page-count") {
            Some(2)
        } else {
            Some(count + 1)
        };
        PaginationScope {
            first_page_number,
            title_page_count: Some(count),
            body_start_page: Some(count + 1),
        }
    } else {
        PaginationScope {
            first_page_number: None,
            title_page_count: None,
            body_start_page: None,
        }
    }
}

fn tagged_item_for_page_item(item_id: &str, page_items: &[PageItem]) -> PdfTaggedItem {
    let item = page_items
        .iter()
        .find(|item| item.element_id == item_id)
        .unwrap_or_else(|| panic!("missing page item for tagged block item id {item_id}"));

    PdfTaggedItem {
        element_id: item.element_id.clone(),
        role: PdfTaggedRole::from_page_item_kind(&item.kind),
        fragment: item.fragment.clone(),
        line_range: item.line_range,
        continuation_markers: item.continuation_markers.clone(),
        artifact: false,
    }
}

fn build_title_structure_page(title_page: &PdfTaggedTitlePage) -> PdfBodyStructPage {
    let mut tagged_lines = Vec::new();
    let mut next_mcid = 0i32;

    for region in [
        PdfTitleBlockRegion::CenterTitle,
        PdfTitleBlockRegion::CenterMeta,
        PdfTitleBlockRegion::BottomLeft,
        PdfTitleBlockRegion::BottomRight,
    ] {
        for block in title_page
            .blocks
            .iter()
            .filter(|block| block.region == region)
        {
            if block.artifact {
                continue;
            }

            for _ in &block.lines {
                tagged_lines.push(PdfBodyStructLine {
                    mcid: next_mcid,
                    role: block.role,
                    structure_key: block.id.clone(),
                });
                next_mcid += 1;
            }
        }
    }

    PdfBodyStructPage { tagged_lines }
}

fn build_body_structure_pages(
    tagged_document: &PdfTaggedDocument,
    body_pages: &[PdfRenderPage],
    _geometry: &LayoutGeometry,
) -> Vec<PdfBodyStructPage> {
    tagged_document
        .body_pages
        .iter()
        .zip(body_pages.iter())
        .map(|(tagged_page, rendered_page)| {
            let emitted_lines = emitted_structure_lines_for_page(rendered_page);
            let tagged_lines = reorder_emitted_lines_with_tagged_page(tagged_page, &emitted_lines);
            PdfBodyStructPage { tagged_lines }
        })
        .collect()
}

fn build_struct_element_plans(
    body_structure_pages: &[PdfBodyStructPage],
) -> Vec<PdfStructElementPlan> {
    let mut plans: Vec<PdfStructElementPlan> = Vec::new();
    let mut plan_index_by_key: BTreeMap<String, usize> = BTreeMap::new();

    for (page_index, page) in body_structure_pages.iter().enumerate() {
        for line in &page.tagged_lines {
            let marked_ref = PdfMarkedContentRef {
                page_index,
                mcid: line.mcid,
            };
            if let Some(&plan_index) = plan_index_by_key.get(&line.structure_key) {
                plans[plan_index].refs.push(marked_ref);
            } else {
                plan_index_by_key.insert(line.structure_key.clone(), plans.len());
                plans.push(PdfStructElementPlan {
                    key: line.structure_key.clone(),
                    role: line.role,
                    refs: vec![marked_ref],
                });
            }
        }
    }

    plans
}

fn build_parent_tree_entries(
    body_structure_pages: &[PdfBodyStructPage],
    struct_element_plans: &[PdfStructElementPlan],
    struct_element_ids: &[Ref],
) -> Vec<Vec<Ref>> {
    let struct_id_by_key = struct_element_plans
        .iter()
        .zip(struct_element_ids.iter().copied())
        .map(|(plan, id)| (plan.key.clone(), id))
        .collect::<BTreeMap<_, _>>();

    body_structure_pages
        .iter()
        .map(|page| {
            let mut entries = page
                .tagged_lines
                .iter()
                .map(|line| {
                    (
                        line.mcid,
                        *struct_id_by_key
                            .get(&line.structure_key)
                            .expect("missing struct element id for structure key"),
                    )
                })
                .collect::<Vec<_>>();
            entries.sort_by_key(|(mcid, _)| *mcid);
            entries.into_iter().map(|(_, id)| id).collect()
        })
        .collect()
}

fn emitted_structure_lines_for_page(page: &PdfRenderPage) -> Vec<PdfEmittedStructLine> {
    let mut next_mcid = 0i32;
    let mut emitted = Vec::new();
    let mut line_index = 0usize;

    while line_index < page.lines.len() {
        let line = &page.lines[line_index];
        if line.text.is_empty() {
            line_index += 1;
            continue;
        }

        if line.dual.is_some() {
            while line_index < page.lines.len() {
                let line = &page.lines[line_index];
                let Some(dual) = &line.dual else {
                    break;
                };

                if let Some(left) = &dual.left {
                    if let Some(role) = tagged_role_for_line_kind(left.kind) {
                        emitted.push(PdfEmittedStructLine {
                            mcid: next_mcid,
                            role,
                            dual_side: Some(1),
                        });
                        next_mcid += 1;
                    }
                }

                if let Some(right) = &dual.right {
                    if let Some(role) = tagged_role_for_line_kind(right.kind) {
                        emitted.push(PdfEmittedStructLine {
                            mcid: next_mcid,
                            role,
                            dual_side: Some(2),
                        });
                        next_mcid += 1;
                    }
                }

                line_index += 1;
            }
            continue;
        }

        if !line.counted {
            line_index += 1;
            continue;
        }

        if let Some(role) = line.kind.and_then(tagged_role_for_line_kind) {
            emitted.push(PdfEmittedStructLine {
                mcid: next_mcid,
                role,
                dual_side: None,
            });
            next_mcid += 1;
        }

        line_index += 1;
    }

    emitted
}

fn reorder_emitted_lines_with_tagged_page(
    tagged_page: &PdfTaggedPage,
    emitted_lines: &[PdfEmittedStructLine],
) -> Vec<PdfBodyStructLine> {
    let mut remaining = emitted_lines.to_vec();
    let mut ordered = Vec::new();

    for block in &tagged_page.blocks {
        match block.placement {
            PdfTaggedBlockPlacement::DualDialogue { side, .. } => {
                ordered.extend(take_dual_side_lines_for_block(&mut remaining, block, side));
            }
            PdfTaggedBlockPlacement::Flow => {
                if should_use_tagged_block_mapping(block) {
                    ordered.extend(take_non_dual_lines_for_block(
                        tagged_page,
                        &mut remaining,
                        block,
                    ));
                } else {
                    let leading_non_dual = take_leading_non_dual_lines(&mut remaining);
                    ordered.extend(leading_non_dual.into_iter().map(|line| PdfBodyStructLine {
                        mcid: line.mcid,
                        role: line.role,
                        structure_key: synthetic_structure_key(tagged_page.page_number, line.mcid),
                    }));
                }
            }
        }
    }

    ordered.extend(remaining.into_iter().map(|line| PdfBodyStructLine {
        mcid: line.mcid,
        role: line.role,
        structure_key: synthetic_structure_key(tagged_page.page_number, line.mcid),
    }));
    ordered
}

fn take_dual_side_lines_for_block(
    remaining: &mut Vec<PdfEmittedStructLine>,
    block: &PdfTaggedBlock,
    side: u8,
) -> Vec<PdfBodyStructLine> {
    let mut taken = Vec::new();

    for item in &block.items {
        let mut taken_for_item = Vec::new();
        let mut matching_positions = remaining
            .iter()
            .enumerate()
            .filter(|(_, line)| line.dual_side == Some(side) && line.role == item.role)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        let lines_to_take = item
            .line_range
            .map(|(start, end)| (end - start + 1) as usize)
            .unwrap_or_else(|| matching_positions.len());

        matching_positions.truncate(lines_to_take);
        for index in matching_positions.into_iter().rev() {
            let line = remaining.remove(index);
            taken_for_item.push(PdfBodyStructLine {
                mcid: line.mcid,
                role: line.role,
                structure_key: item.element_id.clone(),
            });
        }
        taken_for_item.reverse();
        taken.extend(taken_for_item);
    }
    taken
}

fn take_leading_non_dual_lines(
    remaining: &mut Vec<PdfEmittedStructLine>,
) -> Vec<PdfEmittedStructLine> {
    let mut take_count = 0usize;
    let mut first_role = None;

    for line in remaining.iter() {
        if line.dual_side.is_some() {
            break;
        }
        if let Some(role) = first_role {
            if line.role != role {
                break;
            }
        } else {
            first_role = Some(line.role);
        }
        take_count += 1;
    }

    remaining.drain(..take_count).collect()
}

fn take_non_dual_lines_for_block(
    tagged_page: &PdfTaggedPage,
    remaining: &mut Vec<PdfEmittedStructLine>,
    block: &PdfTaggedBlock,
) -> Vec<PdfBodyStructLine> {
    let mut taken = Vec::new();
    let leading_non_dual = take_leading_non_dual_lines(remaining);
    let mut cursor = 0usize;

    for item in &block.items {
        let line_count = item
            .line_range
            .map(|(start, end)| (end - start + 1) as usize)
            .unwrap_or_else(|| {
                leading_non_dual[cursor..]
                    .iter()
                    .take_while(|line| line.role == item.role)
                    .count()
            });

        for line in leading_non_dual
            .iter()
            .skip(cursor)
            .take(line_count)
            .copied()
        {
            taken.push(PdfBodyStructLine {
                mcid: line.mcid,
                role: line.role,
                structure_key: item.element_id.clone(),
            });
        }
        cursor += line_count;
    }

    if taken.is_empty() {
        return leading_non_dual
            .into_iter()
            .map(|line| PdfBodyStructLine {
                mcid: line.mcid,
                role: line.role,
                structure_key: synthetic_structure_key(tagged_page.page_number, line.mcid),
            })
            .collect();
    }

    remaining.splice(0..0, leading_non_dual.into_iter().skip(cursor));
    taken
}

fn should_use_tagged_block_mapping(block: &PdfTaggedBlock) -> bool {
    block.items.len() > 1
        || !block.continuation_markers.is_empty()
        || block.items.iter().any(|item| item.line_range.is_some())
}

fn synthetic_structure_key(page_number: u32, mcid: i32) -> String {
    format!("page-{page_number}-mcid-{mcid}")
}

fn tagged_role_for_line_kind(kind: PdfLineKind) -> Option<PdfTaggedRole> {
    match kind {
        PdfLineKind::Action
        | PdfLineKind::ColdOpening
        | PdfLineKind::NewAct
        | PdfLineKind::EndOfAct
        | PdfLineKind::Lyric => Some(PdfTaggedRole::Action),
        PdfLineKind::SceneHeading => Some(PdfTaggedRole::SceneHeading),
        PdfLineKind::Character
        | PdfLineKind::DualDialogueCharacterLeft
        | PdfLineKind::DualDialogueCharacterRight => Some(PdfTaggedRole::Character),
        PdfLineKind::Dialogue | PdfLineKind::DualDialogueLeft | PdfLineKind::DualDialogueRight => {
            Some(PdfTaggedRole::Dialogue)
        }
        PdfLineKind::Parenthetical
        | PdfLineKind::DualDialogueParentheticalLeft
        | PdfLineKind::DualDialogueParentheticalRight => Some(PdfTaggedRole::Parenthetical),
        PdfLineKind::Transition => Some(PdfTaggedRole::Transition),
    }
}

fn tagged_role_name(role: PdfTaggedRole) -> Name<'static> {
    match role {
        PdfTaggedRole::Title => Name(b"Title"),
        PdfTaggedRole::SceneHeading => Name(b"SceneHeading"),
        PdfTaggedRole::Action => Name(b"Action"),
        PdfTaggedRole::Character => Name(b"Character"),
        PdfTaggedRole::Dialogue => Name(b"Dialogue"),
        PdfTaggedRole::Parenthetical => Name(b"Parenthetical"),
        PdfTaggedRole::Transition => Name(b"Transition"),
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
            actual_text: None,
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
    use crate::{parse, parse_fdx};
    use crate::{blank_attributes, p, tr, Attributes, Element, Metadata};
    use regex::Regex;
    use std::fs;

    #[test]
    fn collect_document_chars_includes_uppercase_title() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["A4 Test".into()]);
        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: Vec::new(),
        };
        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let chars = collect_document_chars(&document);

        assert!(chars.contains(&'A'));
        assert!(chars.contains(&'4'));
        assert!(chars.contains(&'T'));
        assert!(chars.contains(&'E'));
        assert!(chars.contains(&'S'));
        assert!(chars.contains(&'T'));
    }

    #[derive(Debug)]
    struct TaggedPdfInspection {
        has_mark_info: bool,
        has_struct_tree_root: bool,
        has_parent_tree: bool,
        has_role_map: bool,
        catalog_language: Option<String>,
        parent_tree_next_key: Option<i32>,
        role_map_entries: BTreeMap<String, String>,
        struct_parents: Vec<i32>,
        mcids: Vec<i32>,
        artifact_marked_content_count: usize,
        pagination_artifact_count: usize,
        page_number_artifact_count: usize,
        struct_elem_count: usize,
        mcr_count: usize,
        shared_struct_element_count: usize,
        page_labels: Vec<InspectedPageLabel>,
        property_artifacts: Vec<InspectedArtifact>,
        property_marked_content: Vec<InspectedMarkedContent>,
        xmp_metadata: Option<String>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct InspectedPageLabel {
        page_index: i32,
        style: Option<String>,
        offset: Option<i32>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct InspectedArtifact {
        kind: Option<String>,
        subtype: Option<String>,
        attached: Vec<String>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct InspectedMarkedContent {
        tag: String,
        mcid: Option<i32>,
        actual_text: Option<String>,
    }

    fn inspect_tagged_pdf(pdf: &[u8]) -> TaggedPdfInspection {
        let pdf_text = String::from_utf8_lossy(pdf);
        let role_map_entries = extract_name_pairs_from_dict(&pdf_text, "/RoleMap");
        let struct_elem_objects = pdf_text
            .split("endobj")
            .filter(|object| object.contains("/Type /StructElem"))
            .collect::<Vec<_>>();
        let shared_struct_element_count = struct_elem_objects
            .iter()
            .filter(|object| object.matches("/Type /MCR").count() > 1)
            .count();

        TaggedPdfInspection {
            has_mark_info: pdf_text.contains("/MarkInfo"),
            has_struct_tree_root: pdf_text.contains("/StructTreeRoot")
                && pdf_text.contains("/Type /StructTreeRoot"),
            has_parent_tree: pdf_text.contains("/ParentTree"),
            has_role_map: pdf_text.contains("/RoleMap"),
            catalog_language: extract_literal_string_value(&pdf_text, "/Lang "),
            parent_tree_next_key: extract_first_marker_int(&pdf_text, "/ParentTreeNextKey "),
            role_map_entries,
            struct_parents: extract_marker_ints(&pdf_text, "/StructParents "),
            mcids: extract_marker_ints(&pdf_text, "/MCID "),
            artifact_marked_content_count: pdf_text.matches("/Artifact BMC").count()
                + pdf_text.matches("/Artifact <<").count(),
            pagination_artifact_count: pdf_text.matches("/Type /Pagination").count(),
            page_number_artifact_count: pdf_text.matches("/Subtype /PageNum").count(),
            struct_elem_count: struct_elem_objects.len(),
            mcr_count: pdf_text.matches("/Type /MCR").count(),
            shared_struct_element_count,
            page_labels: extract_page_labels(&pdf_text),
            property_artifacts: extract_property_artifacts(&pdf_text),
            property_marked_content: extract_property_marked_content(&pdf_text),
            xmp_metadata: extract_metadata_stream(&pdf_text),
        }
    }

    fn inspect_artifact_properties(stream: &[u8]) -> Vec<InspectedArtifact> {
        extract_property_artifacts(&String::from_utf8_lossy(stream))
    }

    fn inspect_property_marked_content(stream: &[u8]) -> Vec<InspectedMarkedContent> {
        extract_property_marked_content(&String::from_utf8_lossy(stream))
    }

    fn extract_name_pairs_from_dict(pdf_text: &str, marker: &str) -> BTreeMap<String, String> {
        let Some(marker_index) = pdf_text.find(marker) else {
            return BTreeMap::new();
        };
        let dict_text = &pdf_text[marker_index..];
        let Some(dict_start) = dict_text.find("<<") else {
            return BTreeMap::new();
        };
        let dict_text = &dict_text[dict_start + 2..];
        let Some(dict_end) = dict_text.find(">>") else {
            return BTreeMap::new();
        };
        let tokens = dict_text[..dict_end].split_whitespace().collect::<Vec<_>>();
        let mut pairs = BTreeMap::new();
        let mut index = 0usize;

        while index + 1 < tokens.len() {
            let key = tokens[index];
            let value = tokens[index + 1];
            if key.starts_with('/') && value.starts_with('/') {
                pairs.insert(
                    key.trim_start_matches('/').to_string(),
                    value.trim_start_matches('/').to_string(),
                );
                index += 2;
            } else {
                index += 1;
            }
        }

        pairs
    }

    fn extract_marker_ints(pdf_text: &str, marker: &str) -> Vec<i32> {
        let mut ints = Vec::new();
        let mut remaining = pdf_text;

        while let Some(index) = remaining.find(marker) {
            let after_marker = &remaining[index + marker.len()..];
            let digits = after_marker
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect::<String>();
            if let Ok(value) = digits.parse::<i32>() {
                ints.push(value);
            }
            remaining = after_marker;
        }

        ints
    }

    fn extract_first_marker_int(pdf_text: &str, marker: &str) -> Option<i32> {
        extract_marker_ints(pdf_text, marker).into_iter().next()
    }

    fn extract_page_labels(pdf_text: &str) -> Vec<InspectedPageLabel> {
        let object_bodies = pdf_object_bodies_by_id(pdf_text);
        let Some(page_labels_index) = pdf_text.find("/PageLabels") else {
            return Vec::new();
        };
        let page_labels_text = &pdf_text[page_labels_index..];
        let Some(nums_index) = page_labels_text.find("/Nums [") else {
            return Vec::new();
        };
        let nums_text = &page_labels_text[nums_index + "/Nums [".len()..];
        let Some(nums_end) = nums_text.find(']') else {
            return Vec::new();
        };
        let tokens = nums_text[..nums_end].split_whitespace().collect::<Vec<_>>();
        let mut labels = Vec::new();
        let mut index = 0usize;

        while index + 3 < tokens.len() {
            let Ok(page_index) = tokens[index].parse::<i32>() else {
                index += 1;
                continue;
            };
            let Ok(object_id) = tokens[index + 1].parse::<i32>() else {
                index += 1;
                continue;
            };
            let body = object_bodies
                .get(&object_id)
                .expect("expected page label object body");
            labels.push(InspectedPageLabel {
                page_index,
                style: extract_name_value(body, "/S "),
                offset: extract_first_marker_int(body, "/St "),
            });
            index += 4;
        }

        labels
    }

    fn pdf_object_bodies_by_id(pdf_text: &str) -> BTreeMap<i32, String> {
        let mut bodies = BTreeMap::new();

        for object in pdf_text.split("endobj") {
            let trimmed = object.trim_start();
            let mut tokens = trimmed.split_whitespace();
            let Some(id_token) = tokens.next() else {
                continue;
            };
            let Some(gen_token) = tokens.next() else {
                continue;
            };
            let Some(obj_token) = tokens.next() else {
                continue;
            };
            if gen_token != "0" || obj_token != "obj" {
                continue;
            }
            let Ok(object_id) = id_token.parse::<i32>() else {
                continue;
            };
            bodies.insert(object_id, trimmed.to_string());
        }

        bodies
    }

    fn extract_name_value(pdf_text: &str, marker: &str) -> Option<String> {
        let marker_index = pdf_text.find(marker)?;
        let after_marker = &pdf_text[marker_index + marker.len()..];
        let name = after_marker
            .split_whitespace()
            .next()?
            .trim_start_matches('/')
            .to_string();
        Some(name)
    }

    fn extract_literal_string_value(pdf_text: &str, marker: &str) -> Option<String> {
        let marker_index = pdf_text.find(marker)?;
        let after_marker = &pdf_text[marker_index + marker.len()..];
        let string_end = after_marker.find(')')?;
        after_marker
            .strip_prefix('(')
            .map(|text| text[..string_end - 1].to_string())
    }

    fn extract_metadata_stream(pdf_text: &str) -> Option<String> {
        let object_bodies = pdf_object_bodies_by_id(pdf_text);
        let metadata_index = pdf_text.find("/Metadata ")?;
        let metadata_text = &pdf_text[metadata_index + "/Metadata ".len()..];
        let object_id = metadata_text
            .split_whitespace()
            .next()?
            .parse::<i32>()
            .ok()?;
        let body = object_bodies.get(&object_id)?;
        let stream_start = body.find("stream\n")?;
        let stream_body = &body[stream_start + "stream\n".len()..];
        let stream_end = stream_body.find("\nendstream")?;
        Some(stream_body[..stream_end].to_string())
    }

    fn extract_property_artifacts(pdf_text: &str) -> Vec<InspectedArtifact> {
        let mut artifacts = Vec::new();
        let mut remaining = pdf_text;

        while let Some(index) = remaining.find("/Artifact <<") {
            let after_start = &remaining[index + "/Artifact <<".len()..];
            let Some(dict_end) = after_start.find(">> BDC") else {
                break;
            };
            let dict = &after_start[..dict_end];
            artifacts.push(InspectedArtifact {
                kind: extract_name_value(dict, "/Type "),
                subtype: extract_name_value(dict, "/Subtype "),
                attached: extract_name_array(dict, "/Attached ["),
            });
            remaining = &after_start[dict_end + ">> BDC".len()..];
        }

        artifacts
    }

    fn extract_property_marked_content(pdf_text: &str) -> Vec<InspectedMarkedContent> {
        let pattern = Regex::new(r"(?s)/([^[:space:]<]+)[[:space:]]*<<(.*?)>>[[:space:]]*BDC")
            .expect("expected marked-content regex");

        pattern
            .captures_iter(pdf_text)
            .map(|captures| {
                let tag = captures
                    .get(1)
                    .expect("expected marked-content tag")
                    .as_str()
                    .to_string();
                let dict = captures
                    .get(2)
                    .expect("expected marked-content property dictionary")
                    .as_str();

                InspectedMarkedContent {
                    tag,
                    mcid: extract_first_marker_int(dict, "/MCID "),
                    actual_text: extract_literal_string_value(dict, "/ActualText "),
                }
            })
            .collect()
    }

    fn extract_name_array(pdf_text: &str, marker: &str) -> Vec<String> {
        let Some(marker_index) = pdf_text.find(marker) else {
            return Vec::new();
        };
        let after_marker = &pdf_text[marker_index + marker.len()..];
        let Some(array_end) = after_marker.find(']') else {
            return Vec::new();
        };
        after_marker[..array_end]
            .split_whitespace()
            .map(|value| value.trim_start_matches('/').to_string())
            .collect()
    }

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
            imported_layout: None,
            imported_title_page: None,
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

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);

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
    fn pdf_render_options_can_suppress_title_page() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("FIRST BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(
            &screenplay,
            PdfRenderOptions {
                render_title_page: false,
                ..PdfRenderOptions::default()
            },
            &geometry,
        );

        assert!(document.title_page.is_none());
        assert_eq!(document.body_pages.len(), 1);
        assert_eq!(document.body_pages[0].page_number, 1);
        assert_eq!(document.body_pages[0].display_page_number, None);
    }

    #[test]
    fn pdf_render_document_includes_imported_title_overflow_pages() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
            .expect("expected cast-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);

        assert!(document.title_page.is_none());
        assert_eq!(document.title_overflow_pages.len(), 2);
        assert_eq!(document.title_overflow_pages[0].page_number, 1);
        assert_eq!(document.title_overflow_pages[0].display_page_number, None);
        assert_eq!(document.title_overflow_pages[1].page_number, 2);
        assert_eq!(document.title_overflow_pages[1].display_page_number, None);
        assert_eq!(document.body_pages[0].page_number, 3);
        assert!(document.title_overflow_pages[0]
            .lines
            .iter()
            .any(|line| line.text.contains("GUY TEXT")));
        assert!(document.title_overflow_pages[1]
            .lines
            .iter()
            .any(|line| line.text.contains("THE GUYS")));
    }

    #[test]
    fn pdf_render_document_honors_all_caps_in_imported_title_pages() {
        let xml = fs::read_to_string(
            "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fdx",
        )
        .expect("expected big fish fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);

        assert!(document.title_page.is_none());
        assert!(document.title_overflow_pages[0]
            .lines
            .iter()
            .any(|line| line.text.contains("BIG FISH")));
        assert!(!document.title_overflow_pages[0]
            .lines
            .iter()
            .any(|line| line.text.contains("Big Fish")));
    }

    #[test]
    fn pdf_render_document_right_aligns_imported_title_page_paragraphs() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
            .expect("expected cast-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let february_line = document.title_overflow_pages[0]
            .lines
            .iter()
            .find(|line| line.text.contains("February 6th, 2022"))
            .expect("expected February title-page line");
        let february_segment = february_line
            .segments
            .first()
            .expect("expected February title-page segment");

        assert!(february_segment.x > 400.0);
    }

    #[test]
    fn pdf_render_document_uses_imported_title_header_for_overflow_page_numbers() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-pages-multi.fdx")
            .expect("expected multi title-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);

        assert_eq!(document.title_overflow_pages.len(), 2);
        assert_eq!(document.title_overflow_pages[0].display_page_number, None);
        assert_eq!(document.title_overflow_pages[1].display_page_number, Some(2));
    }

    #[test]
    fn pdf_render_output_counts_imported_title_overflow_pages() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
            .expect("expected cast-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");
        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);

        let inspection = inspect_tagged_pdf(&render(&screenplay));
        let expected_page_count = document.body_pages.len()
            + document.title_overflow_pages.len()
            + usize::from(document.title_page.is_some());

        assert_eq!(inspection.struct_parents.len(), expected_page_count);
        assert_eq!(inspection.struct_parents, (0..expected_page_count as i32).collect::<Vec<_>>());
    }

    #[test]
    fn pdf_render_output_emits_lower_roman_labels_for_numbered_title_overflow_pages() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-pages-multi.fdx")
            .expect("expected multi title-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let inspection = inspect_tagged_pdf(&render(&screenplay));

        assert_eq!(
            inspection.page_labels,
            vec![
                InspectedPageLabel {
                    page_index: 0,
                    style: None,
                    offset: None,
                },
                InspectedPageLabel {
                    page_index: 1,
                    style: Some("r".into()),
                    offset: Some(2),
                },
            ]
        );
    }

    #[test]
    fn embedded_fonts_include_curly_apostrophe_from_imported_fdx_content() {
        let xml = fs::read_to_string("tests/fixtures/fdx-import/title-pages-multi.fdx")
            .expect("expected multi title-page fdx fixture");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);

        assert!(collect_document_chars(&document).contains(&'’'));
        assert!(fonts.regular.cid_by_char.contains_key(&'’'));
    }

    #[test]
    fn tagged_pdf_plan_is_derived_from_real_pipeline_for_split_dialogue() {
        let fountain =
            fs::read_to_string("tests/fixtures/corpus/public/big-fish/source/source.fountain")
                .expect("expected big-fish corpus fixture");
        let screenplay = parse(&fountain);
        let geometry = LayoutGeometry::default();
        let tagged = build_tagged_document(&screenplay, &geometry);

        let split_blocks = tagged
            .body_pages
            .iter()
            .flat_map(|page| {
                page.blocks
                    .iter()
                    .map(move |block| (page.page_number, page.body_page_number, block))
            })
            .filter(|(_, _, block)| !block.continuation_markers.is_empty())
            .collect::<Vec<_>>();

        let [(first_page_number, first_body_page_number, outgoing), (second_page_number, second_body_page_number, incoming)] =
            split_blocks
                .windows(2)
                .find(|pair| {
                    pair[0].2.id == pair[1].2.id
                        && pair[0].2.continuation_markers == vec![ContinuationMarker::More]
                        && pair[1].2.continuation_markers == vec![ContinuationMarker::Continued]
                        && pair[0]
                            .2
                            .items
                            .iter()
                            .any(|item| item.role == PdfTaggedRole::Dialogue)
                        && pair[1]
                            .2
                            .items
                            .iter()
                            .any(|item| item.role == PdfTaggedRole::Dialogue)
                })
                .map(|pair| [pair[0], pair[1]])
                .expect("expected a split dialogue block in the real corpus screenplay");

        assert!(tagged.title_page.is_some());
        assert_eq!(outgoing.id, incoming.id);
        assert_eq!(outgoing.fragment, Fragment::ContinuedToNext);
        assert_eq!(incoming.fragment, Fragment::ContinuedFromPrev);
        assert_eq!(
            outgoing.continuation_markers,
            vec![ContinuationMarker::More]
        );
        assert_eq!(
            incoming.continuation_markers,
            vec![ContinuationMarker::Continued]
        );
        assert!(first_page_number < second_page_number);
        assert!(first_body_page_number < second_body_page_number);

        let outgoing_dialogue = outgoing
            .items
            .iter()
            .find(|item| item.role == PdfTaggedRole::Dialogue)
            .expect("expected outgoing dialogue item");
        let incoming_dialogue = incoming
            .items
            .iter()
            .find(|item| item.role == PdfTaggedRole::Dialogue)
            .expect("expected incoming dialogue item");
        assert_eq!(outgoing_dialogue.element_id, incoming_dialogue.element_id);
        assert!(outgoing_dialogue.line_range.is_some());
        assert!(incoming_dialogue.line_range.is_some());
        assert_eq!(
            outgoing_dialogue.continuation_markers,
            vec![ContinuationMarker::More]
        );
        assert_eq!(
            incoming_dialogue.continuation_markers,
            vec![ContinuationMarker::Continued]
        );
        assert_eq!(outgoing_dialogue.artifact, false);
        assert_eq!(incoming_dialogue.artifact, false);
    }

    #[test]
    fn tagged_pdf_plan_preserves_source_order_for_dual_dialogue_blocks() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(p("BRICK"), blank_attributes()),
                    Element::Dialogue(p("Left side."), blank_attributes()),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(p("STEEL"), blank_attributes()),
                    Element::Dialogue(p("Right side."), blank_attributes()),
                ]),
            ])],
        };

        let geometry = LayoutGeometry::default();
        let tagged = build_tagged_document(&screenplay, &geometry);
        let page = tagged
            .body_pages
            .first()
            .expect("expected a body page for dual dialogue");

        assert_eq!(page.blocks.len(), 2);

        let left = &page.blocks[0];
        let right = &page.blocks[1];

        match (&left.placement, &right.placement) {
            (
                PdfTaggedBlockPlacement::DualDialogue {
                    group_id: left_group_id,
                    side: left_side,
                },
                PdfTaggedBlockPlacement::DualDialogue {
                    group_id: right_group_id,
                    side: right_side,
                },
            ) => {
                assert_eq!(left_side, &1);
                assert_eq!(right_side, &2);
                assert_eq!(left_group_id, right_group_id);
            }
            other => panic!("expected dual dialogue placements, got {other:?}"),
        }

        assert_eq!(
            left.items.iter().map(|item| &item.role).collect::<Vec<_>>(),
            vec![&PdfTaggedRole::Character, &PdfTaggedRole::Dialogue]
        );
        assert_eq!(
            right
                .items
                .iter()
                .map(|item| &item.role)
                .collect::<Vec<_>>(),
            vec![&PdfTaggedRole::Character, &PdfTaggedRole::Dialogue]
        );
        assert_eq!(
            left.items
                .iter()
                .map(|item| item.element_id.as_str())
                .collect::<Vec<_>>(),
            vec!["el-00001", "el-00002"]
        );
        assert_eq!(
            right
                .items
                .iter()
                .map(|item| item.element_id.as_str())
                .collect::<Vec<_>>(),
            vec!["el-00003", "el-00004"]
        );
    }

    #[test]
    fn pdf_render_output_emits_valid_pdf_bytes_with_expected_page_count() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
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
        assert!(pdf_text.contains("/BaseFont /AAAAAA+CourierPrime-Regular"));
    }

    #[test]
    fn pdf_render_output_emits_catalog_level_tagged_pdf_scaffolding() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let pdf = render(&screenplay);
        let inspection = inspect_tagged_pdf(&pdf);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(inspection.has_mark_info);
        assert!(pdf_text.contains("/Marked true"));
        assert!(inspection.has_struct_tree_root);
        assert!(inspection.has_parent_tree);
        assert_eq!(inspection.parent_tree_next_key, Some(2));
        assert!(inspection.has_role_map);
        assert_eq!(inspection.role_map_entries.get("Title"), Some(&"P".into()));
        assert_eq!(
            inspection.role_map_entries.get("SceneHeading"),
            Some(&"H1".into())
        );
        assert_eq!(
            inspection.role_map_entries.get("Dialogue"),
            Some(&"P".into())
        );
        assert_eq!(inspection.catalog_language, Some("en-US".into()));
        assert!(pdf_text.contains("/ViewerPreferences"));
        assert!(pdf_text.contains("/DisplayDocTitle true"));
        assert!(pdf_text.contains("/Title (MY SCREENPLAY)"));
    }

    #[test]
    fn pdf_render_output_emits_xmp_metadata_stream() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("My Screenplay")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let pdf = render(&screenplay);
        let pdf_text = String::from_utf8_lossy(&pdf);
        let inspection = inspect_tagged_pdf(&pdf);
        let xmp = inspection
            .xmp_metadata
            .as_deref()
            .expect("expected XMP metadata stream");

        assert!(pdf_text.contains("/Metadata "));
        assert!(pdf_text.contains("/Type /Metadata"));
        assert!(pdf_text.contains("/Subtype /XML"));
        assert!(xmp.contains("<dc:title><rdf:Alt>"));
        assert!(xmp.contains("<rdf:li xml:lang=\"x-default\">My Screenplay</rdf:li>"));
        assert!(xmp.contains("<xmp:CreatorTool>JumpCut 1.0.0-beta.1</xmp:CreatorTool>"));
        assert!(xmp.contains("<pdf:Producer>JumpCut 1.0.0-beta.1</pdf:Producer>"));
        assert!(xmp.contains("<xmp:CreateDate>"));
        assert!(xmp.contains("<xmp:ModifyDate>"));
        assert!(xmp.contains("<xmp:MetadataDate>"));
    }

    #[test]
    fn pdf_render_output_emits_info_author_from_author_and_authors_metadata() {
        let mut metadata = Metadata::new();
        metadata.insert("author".into(), vec![p("Alan Smithee")]);
        metadata.insert("authors".into(), vec![p("Jane Doe"), p("John Roe")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let pdf = render(&screenplay);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("/Author (Alan Smithee, Jane Doe, John Roe)"));
    }

    #[test]
    fn pdf_render_output_emits_matching_xmp_creator_entries() {
        let mut metadata = Metadata::new();
        metadata.insert("author".into(), vec![p("Alan Smithee")]);
        metadata.insert("authors".into(), vec![p("Jane Doe"), p("John Roe")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let inspection = inspect_tagged_pdf(&render(&screenplay));
        let xmp = inspection
            .xmp_metadata
            .as_deref()
            .expect("expected XMP metadata stream");

        assert!(xmp.contains("<dc:creator><rdf:Seq>"));
        assert!(xmp.contains("<rdf:li>Alan Smithee</rdf:li>"));
        assert!(xmp.contains("<rdf:li>Jane Doe</rdf:li>"));
        assert!(xmp.contains("<rdf:li>John Roe</rdf:li>"));
    }

    #[test]
    fn pdf_render_output_defaults_document_language_to_en_us() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let inspection = inspect_tagged_pdf(&render(&screenplay));
        let xmp = inspection
            .xmp_metadata
            .as_deref()
            .expect("expected XMP metadata stream");

        assert_eq!(inspection.catalog_language, Some("en-US".into()));
        assert!(
            xmp.contains("<dc:language><rdf:Bag><rdf:li>en-US</rdf:li></rdf:Bag></dc:language>")
        );
    }

    #[test]
    fn pdf_render_output_uses_metadata_language_override_in_catalog_and_xmp() {
        let mut metadata = Metadata::new();
        metadata.insert("lang".into(), vec![p("fr-CA")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let inspection = inspect_tagged_pdf(&render(&screenplay));
        let xmp = inspection
            .xmp_metadata
            .as_deref()
            .expect("expected XMP metadata stream");

        assert_eq!(inspection.catalog_language, Some("fr-CA".into()));
        assert!(
            xmp.contains("<dc:language><rdf:Bag><rdf:li>fr-CA</rdf:li></rdf:Bag></dc:language>")
        );
    }

    #[test]
    fn pdf_render_output_emits_page_level_struct_parent_and_mcid_markers() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("ALEX"), blank_attributes()),
                    Element::Dialogue(p("HELLO FROM PAGE ONE"), blank_attributes()),
                ]),
            ],
        };

        let pdf = render(&screenplay);
        let inspection = inspect_tagged_pdf(&pdf);
        let unique_mcids = inspection
            .mcids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(inspection.struct_parents, vec![0]);
        assert_eq!(unique_mcids, vec![0, 1, 2]);
        assert!(inspection.mcids.len() >= 6);
        assert_eq!(
            inspection
                .property_marked_content
                .iter()
                .filter_map(|content| content.mcid)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn pdf_render_output_emits_body_structure_roles_for_main_screenplay_content() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![
                Element::SceneHeading(p("INT. LAB - DAY"), blank_attributes()),
                Element::Action(p("Machines hum."), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("MARA"), blank_attributes()),
                    Element::Parenthetical(p("(quietly)"), blank_attributes()),
                    Element::Dialogue(p("Start the sequence."), blank_attributes()),
                ]),
                Element::Transition(p("CUT TO:"), blank_attributes()),
            ],
        };

        let pdf = render(&screenplay);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("/Type /StructElem"));
        assert!(pdf_text.contains("/S /SceneHeading"));
        assert!(pdf_text.contains("/S /Action"));
        assert!(pdf_text.contains("/S /Character"));
        assert!(pdf_text.contains("/S /Parenthetical"));
        assert!(pdf_text.contains("/S /Dialogue"));
        assert!(pdf_text.contains("/S /Transition"));
        assert!(pdf_text.contains("/Type /MCR"));
        assert!(pdf_text.contains("/K ["));

        let role_positions = [
            pdf_text.find("/S /SceneHeading"),
            pdf_text.find("/S /Action"),
            pdf_text.find("/S /Character"),
            pdf_text.find("/S /Parenthetical"),
            pdf_text.find("/S /Dialogue"),
            pdf_text.find("/S /Transition"),
        ];
        assert!(role_positions.iter().all(Option::is_some));

        let role_positions = role_positions.map(Option::unwrap);
        assert!(role_positions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn title_page_structure_page_is_built_from_tagged_title_blocks() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("SAMPLE SCRIPT")]);
        metadata.insert("credit".into(), vec![p("written by")]);
        metadata.insert("author".into(), vec![p("Alan Smithee")]);
        metadata.insert("contact".into(), vec![p("WME"), p("Los Angeles")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let tagged = build_tagged_document(&screenplay, &geometry);
        let title_page = tagged
            .title_page
            .as_ref()
            .expect("expected tagged title page");
        let structure_page = build_title_structure_page(title_page);

        assert_eq!(
            structure_page
                .tagged_lines
                .iter()
                .map(|line| line.structure_key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "title-page-01-01",
                "title-page-02-02",
                "title-page-03-03",
                "title-page-04-05",
                "title-page-04-05",
            ]
        );
        assert!(structure_page
            .tagged_lines
            .iter()
            .all(|line| line.role == PdfTaggedRole::Title));
        assert_eq!(
            structure_page
                .tagged_lines
                .iter()
                .map(|line| line.mcid)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn pdf_render_output_emits_title_page_structure_roles() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("SAMPLE SCRIPT")]);
        metadata.insert("credit".into(), vec![p("written by")]);
        metadata.insert("author".into(), vec![p("Alan Smithee")]);
        metadata.insert("contact".into(), vec![p("WME"), p("Los Angeles")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let pdf = render(&screenplay);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("/StructParents 0"));
        assert!(pdf_text.contains("/StructParents 1"));
        assert!(pdf_text.contains("/S /Title"));
        assert!(pdf_text.contains("/K ["));
        assert!(pdf_text.contains("/Type /MCR"));
        assert!(pdf_text.contains("/Pg "));
        assert!(pdf_text.matches("/S /Title").count() >= 4);
    }

    #[test]
    fn body_structure_pages_keep_dual_dialogue_in_authored_reading_order() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(p("BRICK"), blank_attributes()),
                    Element::Parenthetical(p("(whispering)"), blank_attributes()),
                    Element::Dialogue(
                        p("Left side gets the longer speech so it wraps onto another line in the dual dialogue lane."),
                        blank_attributes(),
                    ),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(p("STEEL"), blank_attributes()),
                    Element::Dialogue(p("Right side stays short."), blank_attributes()),
                ]),
            ])],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged = build_tagged_document(&screenplay, &geometry);
        assert!(
            document.body_pages[0]
                .lines
                .iter()
                .filter(|line| line.dual.is_some())
                .count()
                >= 2
        );
        let body_structure_pages =
            build_body_structure_pages(&tagged, &document.body_pages, &geometry);
        let tagged_lines = &body_structure_pages[0].tagged_lines;

        assert_eq!(
            tagged_lines,
            &vec![
                PdfBodyStructLine {
                    mcid: 0,
                    role: PdfTaggedRole::Character,
                    structure_key: "el-00001".into(),
                },
                PdfBodyStructLine {
                    mcid: 2,
                    role: PdfTaggedRole::Parenthetical,
                    structure_key: "el-00002".into(),
                },
                PdfBodyStructLine {
                    mcid: 4,
                    role: PdfTaggedRole::Dialogue,
                    structure_key: "el-00003".into(),
                },
                PdfBodyStructLine {
                    mcid: 5,
                    role: PdfTaggedRole::Dialogue,
                    structure_key: "el-00003".into(),
                },
                PdfBodyStructLine {
                    mcid: 6,
                    role: PdfTaggedRole::Dialogue,
                    structure_key: "el-00003".into(),
                },
                PdfBodyStructLine {
                    mcid: 7,
                    role: PdfTaggedRole::Dialogue,
                    structure_key: "el-00003".into(),
                },
                PdfBodyStructLine {
                    mcid: 1,
                    role: PdfTaggedRole::Character,
                    structure_key: "el-00004".into(),
                },
                PdfBodyStructLine {
                    mcid: 3,
                    role: PdfTaggedRole::Dialogue,
                    structure_key: "el-00005".into(),
                },
            ]
        );
    }

    #[test]
    fn body_structure_pages_keep_split_dialogue_on_one_structure_key_across_pages() {
        let fountain =
            fs::read_to_string("tests/fixtures/corpus/public/big-fish/source/source.fountain")
                .expect("expected big-fish corpus fixture");
        let screenplay = parse(&fountain);

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged = build_tagged_document(&screenplay, &geometry);
        let body_structure_pages =
            build_body_structure_pages(&tagged, &document.body_pages, &geometry);

        let matching_keys = body_structure_pages
            .windows(2)
            .find_map(|pair| {
                let left_dialogue_keys = pair[0]
                    .tagged_lines
                    .iter()
                    .filter(|line| line.role == PdfTaggedRole::Dialogue)
                    .map(|line| line.structure_key.as_str())
                    .collect::<Vec<_>>();
                let right_dialogue_keys = pair[1]
                    .tagged_lines
                    .iter()
                    .filter(|line| line.role == PdfTaggedRole::Dialogue)
                    .map(|line| line.structure_key.as_str())
                    .collect::<Vec<_>>();

                left_dialogue_keys
                    .iter()
                    .copied()
                    .find(|key| right_dialogue_keys.contains(key))
                    .map(|key| (left_dialogue_keys, right_dialogue_keys, key.to_string()))
            })
            .expect("expected adjacent pages with a continued dialogue structure key");

        assert!(matching_keys.0.contains(&matching_keys.2.as_str()));
        assert!(matching_keys.1.contains(&matching_keys.2.as_str()));
    }

    #[test]
    fn struct_element_plans_collapse_repeated_structure_keys_across_pages() {
        let fountain =
            fs::read_to_string("tests/fixtures/corpus/public/big-fish/source/source.fountain")
                .expect("expected big-fish corpus fixture");
        let screenplay = parse(&fountain);

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged = build_tagged_document(&screenplay, &geometry);
        let body_structure_pages =
            build_body_structure_pages(&tagged, &document.body_pages, &geometry);
        let struct_element_plans = build_struct_element_plans(&body_structure_pages);

        let repeated_dialogue_plan = struct_element_plans
            .iter()
            .find(|plan| {
                plan.role == PdfTaggedRole::Dialogue
                    && plan.refs.len() > 1
                    && plan
                        .refs
                        .windows(2)
                        .any(|pair| pair[0].page_index != pair[1].page_index)
            })
            .expect("expected a dialogue struct element plan spanning multiple pages");

        assert!(repeated_dialogue_plan.refs.len() >= 2);
    }

    #[test]
    fn pdf_render_output_collapses_split_dialogue_into_shared_struct_elements() {
        let fountain =
            fs::read_to_string("tests/fixtures/corpus/public/big-fish/source/source.fountain")
                .expect("expected big-fish corpus fixture");
        let screenplay = parse(&fountain);

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged = build_tagged_document(&screenplay, &geometry);
        let mut structure_pages = Vec::new();
        if let Some(title_page) = &tagged.title_page {
            structure_pages.push(build_title_structure_page(title_page));
        }
        structure_pages.extend(build_body_structure_pages(
            &tagged,
            &document.body_pages,
            &geometry,
        ));
        let struct_element_plans = build_struct_element_plans(&structure_pages);
        let total_tagged_lines: usize = structure_pages
            .iter()
            .map(|page| page.tagged_lines.len())
            .sum();

        let repeated_dialogue_plan = struct_element_plans
            .iter()
            .find(|plan| {
                plan.role == PdfTaggedRole::Dialogue
                    && plan.refs.len() > 1
                    && plan
                        .refs
                        .windows(2)
                        .any(|pair| pair[0].page_index != pair[1].page_index)
            })
            .expect("expected a multi-page dialogue struct element plan");

        let pdf = render(&screenplay);
        let inspection = inspect_tagged_pdf(&pdf);

        assert!(repeated_dialogue_plan.refs.len() >= 2);
        assert_eq!(inspection.struct_elem_count, struct_element_plans.len());
        assert!(inspection.struct_elem_count < total_tagged_lines);
        assert!(inspection.mcr_count > inspection.struct_elem_count);
        assert!(inspection.shared_struct_element_count >= 1);
    }

    #[test]
    fn tagged_pdf_inspection_reports_core_tagged_pdf_signals() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("MY SCREENPLAY")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::SceneHeading(
                    p("INT. LAB - DAY"),
                    Attributes {
                        scene_number: Some("12A".into()),
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
                Element::Action(p("Machines hum."), blank_attributes()),
            ],
        };

        let inspection = inspect_tagged_pdf(&render(&screenplay));
        let unique_mcids = inspection
            .mcids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        assert!(inspection.has_mark_info);
        assert!(inspection.has_struct_tree_root);
        assert!(inspection.has_parent_tree);
        assert!(inspection.has_role_map);
        assert_eq!(inspection.parent_tree_next_key, Some(3));
        assert_eq!(
            inspection.role_map_entries.get("SceneHeading"),
            Some(&"H1".into())
        );
        assert_eq!(inspection.role_map_entries.get("Title"), Some(&"P".into()));
        assert_eq!(inspection.struct_parents, vec![0, 1, 2]);
        assert_eq!(unique_mcids, vec![0, 1]);
        assert!(inspection.mcids.len() >= 8);
        assert_eq!(inspection.artifact_marked_content_count, 3);
        assert_eq!(inspection.pagination_artifact_count, 1);
        assert_eq!(inspection.page_number_artifact_count, 1);
        assert_eq!(
            inspection.property_artifacts,
            vec![InspectedArtifact {
                kind: Some("Pagination".into()),
                subtype: Some("PageNum".into()),
                attached: vec!["Top".into(), "Right".into()],
            }]
        );
        assert!(inspection.struct_elem_count >= 4);
        assert!(inspection.mcr_count >= unique_mcids.len());
    }

    #[test]
    fn pdf_render_output_emits_page_labels_matching_visible_screenplay_numbers() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("TITLE PAGE")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
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

        let inspection = inspect_tagged_pdf(&render(&screenplay));

        assert_eq!(
            inspection.page_labels,
            vec![
                InspectedPageLabel {
                    page_index: 0,
                    style: None,
                    offset: None,
                },
                InspectedPageLabel {
                    page_index: 2,
                    style: Some("D".into()),
                    offset: Some(2),
                },
            ]
        );
    }

    #[test]
    fn pdf_render_output_includes_body_page_text_in_content_streams() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![
                Element::Action(p("FIRST BODY PAGE"), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("ALEX"), blank_attributes()),
                    Element::Dialogue(p("HELLO FROM PAGE ONE"), blank_attributes()),
                ]),
            ],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );

        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "FIRST BODY PAGE",
            108.0,
            711.0,
        );
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
            imported_layout: None,
            imported_title_page: None,
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

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let first_page = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
        let second_page = render_body_page_content(
            &document.body_pages[1],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );

        assert_stream_lacks_text(&first_page, &fonts.regular, "1.");
        assert_stream_contains_fixed_cell_text_at(
            &second_page,
            &fonts.regular,
            "2.",
            page_number_x(2, &geometry),
            PAGE_NUMBER_BASELINE_Y,
        );
    }

    #[test]
    fn pdf_render_output_positions_centered_lines_away_from_the_body_left_margin() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(
                p("CENTERED LINE"),
                Attributes {
                    centered: true,
                    ..blank_attributes()
                },
            )],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(!pdf_text.contains("72 711 Tm"));
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "CENTERED LINE",
            278.5,
            711.0,
        );
    }

    #[test]
    fn pdf_render_output_uses_dual_dialogue_margins_for_both_sides() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
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

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
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
    fn body_page_artifacts_wrap_page_numbers_scene_numbers_and_split_contd_cues() {
        let document = PdfRenderDocument {
            title_page: None,
            title_overflow_pages: Vec::new(),
            body_pages: vec![PdfRenderPage {
                page_number: 35,
                display_page_number: Some(34),
                lines: vec![
                    PdfRenderLine {
                        text: "MAYOR (CONT'D)".into(),
                        counted: false,
                        centered: false,
                        kind: Some(PdfLineKind::Character),
                        fragments: vec![PdfRenderFragment {
                            actual_text: None,
                            text: "MAYOR (CONT'D)".into(),
                            styles: Vec::new(),
                        }],
                        dual: None,
                        scene_number: None,
                    },
                    PdfRenderLine {
                        text: "INT. HALLWAY - NIGHT".into(),
                        counted: true,
                        centered: false,
                        kind: Some(PdfLineKind::SceneHeading),
                        fragments: vec![PdfRenderFragment {
                            actual_text: None,
                            text: "INT. HALLWAY - NIGHT".into(),
                            styles: Vec::new(),
                        }],
                        dual: None,
                        scene_number: Some("12A".into()),
                    },
                ],
            }],
        };
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&Metadata::new()),
        );
        let pdf_text = String::from_utf8_lossy(&content);
        let artifacts = inspect_artifact_properties(&content);
        let marked_content = inspect_property_marked_content(&content)
            .into_iter()
            .filter(|content| content.tag != "Artifact")
            .collect::<Vec<_>>();

        assert_eq!(pdf_text.matches("/Artifact BMC").count(), 3);
        assert_eq!(
            artifacts,
            vec![InspectedArtifact {
                kind: Some("Pagination".into()),
                subtype: Some("PageNum".into()),
                attached: vec!["Top".into(), "Right".into()],
            }]
        );
        assert_eq!(
            marked_content,
            vec![InspectedMarkedContent {
                tag: "SceneHeading".into(),
                mcid: Some(0),
                actual_text: None,
            }]
        );
    }

    #[test]
    fn body_page_artifacts_wrap_more_markers() {
        let document = PdfRenderDocument {
            title_page: None,
            title_overflow_pages: Vec::new(),
            body_pages: vec![PdfRenderPage {
                page_number: 12,
                display_page_number: Some(11),
                lines: vec![
                    PdfRenderLine {
                        text: "        (MORE)".into(),
                        counted: false,
                        centered: false,
                        kind: Some(PdfLineKind::Character),
                        fragments: vec![PdfRenderFragment {
                            actual_text: None,
                            text: "        (MORE)".into(),
                            styles: Vec::new(),
                        }],
                        dual: None,
                        scene_number: None,
                    },
                    PdfRenderLine {
                        text: "HELLO".into(),
                        counted: true,
                        centered: false,
                        kind: Some(PdfLineKind::Dialogue),
                        fragments: vec![PdfRenderFragment {
                            actual_text: None,
                            text: "HELLO".into(),
                            styles: Vec::new(),
                        }],
                        dual: None,
                        scene_number: None,
                    },
                ],
            }],
        };
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&Metadata::new()),
        );
        let pdf_text = String::from_utf8_lossy(&content);
        let artifacts = inspect_artifact_properties(&content);
        let marked_content = inspect_property_marked_content(&content)
            .into_iter()
            .filter(|content| content.tag != "Artifact")
            .collect::<Vec<_>>();

        assert_eq!(pdf_text.matches("/Artifact BMC").count(), 1);
        assert_eq!(
            artifacts,
            vec![InspectedArtifact {
                kind: Some("Pagination".into()),
                subtype: Some("PageNum".into()),
                attached: vec!["Top".into(), "Right".into()],
            }]
        );
        assert_eq!(
            marked_content,
            vec![InspectedMarkedContent {
                tag: "Dialogue".into(),
                mcid: Some(0),
                actual_text: None,
            }]
        );
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
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_title_page_content(
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            &geometry,
        );

        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.bold,
            "SAMPLE SCRIPT",
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "written by",
            title_page_line_left("written by", PdfTitleBlockRegion::CenterMeta, &geometry),
            title_page_center_meta_top_y(&geometry),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "Alan Smithee",
            title_page_line_left("Alan Smithee", PdfTitleBlockRegion::CenterMeta, &geometry),
            title_page_center_meta_top_y(&geometry) - (TITLE_BLOCK_LINE_STEP * 2.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "based on the novel",
            title_page_line_left(
                "based on the novel",
                PdfTitleBlockRegion::CenterMeta,
                &geometry,
            ),
            title_page_center_meta_top_y(&geometry) - (TITLE_BLOCK_LINE_STEP * 7.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "by J.R.R. Smithee",
            title_page_line_left(
                "by J.R.R. Smithee",
                PdfTitleBlockRegion::CenterMeta,
                &geometry,
            ),
            title_page_center_meta_top_y(&geometry) - (TITLE_BLOCK_LINE_STEP * 8.0),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "WME",
            title_page_line_left("WME", PdfTitleBlockRegion::BottomLeft, &geometry),
            title_page_bottom_top_y(&geometry),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "First Draft",
            title_page_bottom_right_left("First Draft", &geometry),
            title_page_bottom_top_y(&geometry),
        );
        assert_stream_contains_fixed_cell_text_at(
            &content,
            &fonts.regular,
            "April 6, 1952",
            title_page_bottom_right_left("April 6, 1952", &geometry),
            title_page_bottom_top_y(&geometry) - TITLE_BLOCK_LINE_STEP,
        );
    }

    #[test]
    fn pdf_render_output_uppercases_and_underlines_plain_title_page_titles() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("Sample Script")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        let mut next_mcid = 0i32;
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            title_page_center_title_top_y(&geometry),
            TITLE_FONT_SIZE,
            &mut underlines,
            &mut next_mcid,
            &geometry,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = content.finish().to_vec();
        let stream_text = String::from_utf8_lossy(&stream);

        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "SAMPLE SCRIPT",
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
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
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        let mut next_mcid = 0i32;
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            title_page_center_title_top_y(&geometry),
            TITLE_FONT_SIZE,
            &mut underlines,
            &mut next_mcid,
            &geometry,
        );
        content.end_text();
        let stream = content.finish().to_vec();

        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "Sample Script",
            title_page_line_left("Sample Script", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
        );
        assert_stream_lacks_text(&stream, &fonts.bold, "SAMPLE SCRIPT");
    }

    #[test]
    fn pdf_render_output_wraps_styled_inline_fragments_in_span_marked_content() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
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

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
        let marked_content = inspect_property_marked_content(&content);

        assert_eq!(
            marked_content,
            vec![
                InspectedMarkedContent {
                    tag: "Action".into(),
                    mcid: Some(0),
                    actual_text: None,
                },
                InspectedMarkedContent {
                    tag: "Span".into(),
                    mcid: None,
                    actual_text: None,
                },
                InspectedMarkedContent {
                    tag: "Span".into(),
                    mcid: None,
                    actual_text: None,
                },
                InspectedMarkedContent {
                    tag: "Span".into(),
                    mcid: None,
                    actual_text: None,
                },
                InspectedMarkedContent {
                    tag: "Span".into(),
                    mcid: None,
                    actual_text: None,
                },
            ]
        );
    }

    #[test]
    fn pdf_render_output_forced_uppercase_plain_title_does_not_override_extraction_casing() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("Sample Script")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        let mut next_mcid = 0i32;
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            title_page_center_title_top_y(&geometry),
            TITLE_FONT_SIZE,
            &mut underlines,
            &mut next_mcid,
            &geometry,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = content.finish().to_vec();
        let stream_text = String::from_utf8_lossy(&stream);

        assert!(!stream_text.contains("/ActualText"));
        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "SAMPLE SCRIPT",
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
        );
    }

    #[test]
    fn pdf_render_output_allow_lowercase_title_keeps_authored_title_case() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec![p("Sample Script")]);
        metadata.insert("fmt".into(), vec![p("allow-lowercase-title")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        let mut next_mcid = 0i32;
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            title_page_center_title_top_y(&geometry),
            TITLE_FONT_SIZE,
            &mut underlines,
            &mut next_mcid,
            &geometry,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = content.finish().to_vec();
        let stream_text = String::from_utf8_lossy(&stream);

        assert!(!stream_text.contains("/ActualText"));
        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "Sample Script",
            title_page_line_left("Sample Script", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
        );
    }

    #[test]
    fn pdf_render_output_honors_all_caps_style_on_styled_imported_title_lines() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "title".into(),
            vec![ElementText::Styled(vec![tr(
                "Big Fish",
                vec!["Bold", "Underline", "AllCaps"],
            )])],
        );
        metadata.insert("fmt".into(), vec![p("allow-lowercase-title")]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("BODY PAGE"), blank_attributes())],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let tagged_document = build_tagged_document(&screenplay, &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let mut content = Content::new();
        let mut underlines = Vec::new();
        let mut next_mcid = 0i32;
        content.begin_text();
        render_title_page_region(
            &mut content,
            document.title_page.as_ref().expect("expected title page"),
            tagged_document
                .title_page
                .as_ref()
                .expect("expected tagged title page"),
            &fonts,
            PdfTitleBlockRegion::CenterTitle,
            title_page_center_title_top_y(&geometry),
            TITLE_FONT_SIZE,
            &mut underlines,
            &mut next_mcid,
            &geometry,
        );
        content.end_text();
        render_underlines(&mut content, &underlines);
        let stream = content.finish().to_vec();

        assert_stream_contains_fixed_cell_text_at(
            &stream,
            &fonts.bold,
            "BIG FISH",
            title_page_line_left("BIG FISH", PdfTitleBlockRegion::CenterTitle, &geometry),
            title_page_center_title_top_y(&geometry),
        );
    }

    #[test]
    fn pdf_render_output_uses_font_variants_and_underlines_for_styled_fragments() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
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

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
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
            imported_layout: None,
            imported_title_page: None,
            elements: vec![
                Element::SceneHeading(p("INT. OFFICE - DAY"), blank_attributes()),
                Element::Lyric(p("I love to sing"), blank_attributes()),
            ],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("/F3 12 Tf"));
    }

    #[test]
    fn pdf_render_output_applies_bsh_and_ush_to_scene_headings() {
        let mut metadata = Metadata::new();
        metadata.insert("fmt".into(), vec!["bsh ush".into()]);
        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::SceneHeading(
                p("INT. OFFICE - DAY"),
                blank_attributes(),
            )],
        };

        let geometry = LayoutGeometry::default();
        let document = build_render_document(&screenplay, PdfRenderOptions::default(), &geometry);
        let fonts = EmbeddedFonts::new(&document);
        let content = render_body_page_content(
            &document.body_pages[0],
            &LayoutGeometry::default(),
            &fonts,
            &ScreenplayLayoutProfile::from_metadata(&screenplay.metadata),
        );
        let pdf_text = String::from_utf8_lossy(&content);

        assert!(pdf_text.contains("/F2 12 Tf"));
        assert!(pdf_text.contains("0.75 w"));
    }

    #[test]
    fn underline_segments_do_not_extend_into_trailing_wrap_spaces() {
        let document = PdfRenderDocument {
            title_page: None,
            title_overflow_pages: Vec::new(),
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
                actual_text: None,
                tagged_span: false,
                text: "FALL ".into(),
                styles: StyleFlags {
                    underline: true,
                    ..StyleFlags::default()
                },
            }],
            None,
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
        let geometry = LayoutGeometry::default();
        assert_eq!(
            title_page_line_left("SAMPLE SCRIPT", PdfTitleBlockRegion::CenterTitle, &geometry),
            260.5
        );
        assert_eq!(
            title_page_bottom_right_left("April 6, 1952", &geometry),
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
    fn ordinary_character_cues_snap_left_indent_to_eighth_inches() {
        let mut geometry = LayoutGeometry::default();
        geometry.character_left = 3.38;

        assert_eq!(
            line_kind_left(Some(PdfLineKind::Character), &geometry),
            243.0
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
            scene_number: None,
        };

        assert!((body_line_left(&line, &geometry) - 462.2).abs() < 0.001);
    }

    #[test]
    fn body_line_left_hangs_opening_parenthetical_by_the_configured_first_indent() {
        let geometry = LayoutGeometry::default();
        let line = PdfRenderLine {
            text: format!("{}(quietly)", " ".repeat(14)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Parenthetical),
            fragments: Vec::new(),
            dual: None,
            scene_number: None,
        };
        let continuation = PdfRenderLine {
            text: format!("{}quietly", " ".repeat(15)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Parenthetical),
            fragments: Vec::new(),
            dual: None,
            scene_number: None,
        };

        assert!((body_line_left(&line, &geometry) - 208.8).abs() < 0.001);
        assert_eq!(rendered_body_line_text(&line, &geometry), "(quietly)");
        assert!((body_line_left(&continuation, &geometry) - 216.0).abs() < 0.001);
        assert_eq!(rendered_body_line_text(&continuation, &geometry), "quietly");
    }

    #[test]
    fn body_line_left_uses_parenthetical_first_indent_from_geometry() {
        let mut geometry = LayoutGeometry::default();
        geometry.parenthetical_first_indent = -0.14;

        let line = PdfRenderLine {
            text: format!("{}(quietly)", " ".repeat(14)),
            counted: true,
            centered: false,
            kind: Some(PdfLineKind::Parenthetical),
            fragments: Vec::new(),
            dual: None,
            scene_number: None,
        };

        assert!((body_line_left(&line, &geometry) - 205.92).abs() < 0.001);
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
        let geometry = LayoutGeometry::default();
        assert_eq!(body_line_step_points(&geometry), 12.0);
        assert_eq!(first_body_line_y(&geometry), 711.0);
        assert_eq!(page_number_y(&geometry), PAGE_NUMBER_BASELINE_Y);
    }

    #[test]
    fn title_page_vertical_metrics_follow_shared_geometry() {
        let geometry = LayoutGeometry {
            page_height: 11.69,
            top_margin: 1.2,
            bottom_margin: 1.3,
            lines_per_page: 58.0,
            ..LayoutGeometry::default()
        };
        let expected_step = ((11.69 - 1.2 - 1.3) * 72.0) / 58.0;
        let expected_first_body_line_y = (11.69 * 72.0) - (1.2 * 72.0) - 9.0;

        assert!((body_line_step_points(&geometry) - expected_step).abs() < 0.001);
        assert!(
            (title_page_center_title_top_y(&geometry)
                - (expected_first_body_line_y - (18.0 * expected_step)))
                .abs()
                < 0.001
        );
        assert!(
            (title_page_center_meta_top_y(&geometry)
                - (expected_first_body_line_y - (22.0 * expected_step)))
                .abs()
                < 0.001
        );
        assert!(
            (title_page_bottom_top_y(&geometry) - ((1.3 * 72.0) + (5.25 * expected_step))).abs()
                < 0.001
        );
    }

    #[test]
    fn page_number_x_keeps_the_period_column_fixed() {
        let geometry = LayoutGeometry::default();
        assert_eq!(page_number_x(2, &geometry), PAGE_NUMBER_LEFT);
        assert_eq!(
            page_number_x(34, &geometry),
            PAGE_NUMBER_LEFT - BODY_TEXT_CELL_WIDTH
        );
        assert_eq!(
            page_number_x(100, &geometry),
            PAGE_NUMBER_LEFT - (2.0 * BODY_TEXT_CELL_WIDTH)
        );
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
                    scene_number: None,
                },
                PdfRenderLine {
                    text: format!("{}But take with you this Key", " ".repeat(10)),
                    counted: true,
                    centered: false,
                    kind: Some(PdfLineKind::Dialogue),
                    fragments: Vec::new(),
                    dual: None,
                    scene_number: None,
                },
            ],
        };

        assert!(page_starts_with_split_contd_character(&page));
        let geometry = LayoutGeometry::default();
        assert_eq!(
            first_body_line_y_for_page(&page, &geometry),
            first_body_line_y(&geometry) + body_line_step_points(&geometry)
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
                scene_number: None,
            }],
        };

        assert!(!page_starts_with_split_contd_character(&page));
        let geometry = LayoutGeometry::default();
        assert_eq!(
            first_body_line_y_for_page(&page, &geometry),
            first_body_line_y(&geometry)
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
