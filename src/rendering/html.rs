use super::shared::{escape_html, join_metadata, sorted_style_names};

use crate::pagination::margin::dual_dialogue_character_left_indent;
use crate::pagination::visual_lines::{
    display_page_number, render_paginated_visual_pages_with_options,
    render_unpaginated_visual_lines_with_options, visual_line_class_name, VisualLine,
    VisualRenderOptions,
};
use crate::pagination::wrapping::ElementType;
use crate::pagination::{ScreenplayLayoutProfile, StyleProfile};
use crate::title_page::{TitlePage, TitlePageBlockKind};
use crate::{Attributes, Element, ElementText, Screenplay};
use std::fmt::Write;

const HTML_STYLE: &str = include_str!("../templates/html_style.css");
#[cfg(not(target_arch = "wasm32"))]
const COURIER_PRIME_REGULAR_TTF: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-Regular.ttf");
#[cfg(not(target_arch = "wasm32"))]
const COURIER_PRIME_ITALIC_TTF: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-Italic.ttf");
#[cfg(not(target_arch = "wasm32"))]
const COURIER_PRIME_BOLD_TTF: &[u8] = include_bytes!("../templates/fonts/CourierPrime-Bold.ttf");
#[cfg(not(target_arch = "wasm32"))]
const COURIER_PRIME_BOLD_ITALIC_TTF: &[u8] =
    include_bytes!("../templates/fonts/CourierPrime-BoldItalic.ttf");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlRenderOptions {
    pub head: bool,
    pub exact_wraps: bool,
    pub paginated: bool,
    pub render_continueds: bool,
    pub render_title_page: bool,
    pub embed_courier_prime: bool,
    pub embedded_courier_prime_css: Option<String>,
}

impl Default for HtmlRenderOptions {
    fn default() -> Self {
        Self {
            head: true,
            exact_wraps: false,
            paginated: false,
            render_continueds: true,
            render_title_page: true,
            embed_courier_prime: false,
            embedded_courier_prime_css: None,
        }
    }
}

pub fn embedded_courier_prime_css_from_base64(
    regular_ttf_base64: &str,
    italic_ttf_base64: &str,
    bold_ttf_base64: &str,
    bold_italic_ttf_base64: &str,
) -> String {
    [
        embedded_font_face_from_base64("Courier Prime", 400, "normal", regular_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 400, "italic", italic_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 700, "normal", bold_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 700, "italic", bold_italic_ttf_base64),
    ]
    .join("\n")
}

fn embedded_font_face_from_base64(
    font_family: &str,
    font_weight: u16,
    font_style: &str,
    encoded: &str,
) -> String {
    format!(
        "@font-face {{\n  font-family: \"{font_family}\";\n  src: url(data:font/ttf;base64,{encoded}) format(\"truetype\");\n  font-weight: {font_weight};\n  font-style: {font_style};\n}}\n"
    )
}

pub(crate) fn render_document(screenplay: &Screenplay, options: HtmlRenderOptions) -> String {
    let layout_profile = ScreenplayLayoutProfile::from_screenplay(screenplay);
    let mut out = String::with_capacity(32 * 1024);
    if options.head {
        out.push_str("<!doctype html>\n\n<html>\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\n  <title>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "title",
            " ",
        )));
        out.push_str("</title>\n\n");
        write_style_block(&mut out, &options, &layout_profile);
        out.push_str("</head>\n\n<body");
        if options.paginated {
            out.push_str(" class=\"paginatedHtmlView\"");
        }
        out.push_str(">\n");
    } else {
        write_style_block(&mut out, &options, &layout_profile);
    }

    write!(
        out,
        "<section class=\"{}\">\n",
        root_class_name(&layout_profile, &options)
    )
    .unwrap();
    render_body(&mut out, screenplay, &options, &layout_profile);
    out.push_str("</section>\n");

    if options.head {
        out.push_str("</body>\n</html>\n");
    }

    out
}

fn write_style_block(
    out: &mut String,
    options: &HtmlRenderOptions,
    layout_profile: &ScreenplayLayoutProfile,
) {
    out.push_str("  <style type=\"text/css\">\n");
    out.push_str(&stylesheet_css(options, layout_profile));
    out.push_str("  </style>\n");
}

fn stylesheet_css(options: &HtmlRenderOptions, layout_profile: &ScreenplayLayoutProfile) -> String {
    let mut css = String::new();
    if let Some(font_css) = embedded_courier_prime_font_css(options) {
        css.push_str(&font_css);
        css.push('\n');
    }
    css.push_str(HTML_STYLE);
    css.push('\n');
    css.push_str(&layout_profile_css(layout_profile));
    css
}

fn layout_profile_css(layout_profile: &ScreenplayLayoutProfile) -> String {
    let page_width = css_inches(layout_profile.page_width);
    let page_height = css_inches(layout_profile.page_height);
    let page_left_margin = css_inches(layout_profile.styles.action.left_indent);
    let page_right_margin = css_inches(
        (layout_profile.page_width - layout_profile.styles.action.right_indent).max(0.0),
    );

    format!(
        "@page screenplay, screenplay-title {{\n  size: {page_width} {page_height};\n  margin-top: 1in;\n  margin-right: {page_right_margin};\n  margin-bottom: 0.75in;\n  margin-left: {page_left_margin};\n}}\n\n.screenplay.paginatedHtml .page {{\n  width: {page_width};\n  min-height: {page_height};\n  padding-left: {page_left_margin};\n  padding-right: {page_right_margin};\n}}\n\n.screenplay .title-page {{\n  width: {page_width};\n  min-height: {page_height};\n  padding-left: {page_left_margin};\n  padding-right: {page_right_margin};\n}}\n"
    )
}

fn css_inches(value: f32) -> String {
    let trimmed = format!("{value:.2}");
    let trimmed = trimmed.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}in")
}

fn root_class_name(
    layout_profile: &ScreenplayLayoutProfile,
    options: &HtmlRenderOptions,
) -> String {
    let mut classes = match layout_profile.style_profile {
        StyleProfile::Screenplay => vec!["screenplay"],
        StyleProfile::Multicam => vec!["screenplay", "multicam"],
    };

    if options.exact_wraps || options.paginated {
        classes.push("exactWraps");
    }
    if options.paginated {
        classes.push("paginatedHtml");
    }

    classes.join(" ")
}

fn render_body(
    out: &mut String,
    screenplay: &Screenplay,
    options: &HtmlRenderOptions,
    layout_profile: &ScreenplayLayoutProfile,
) {
    if options.render_title_page {
        if let Some(imported_title_page) = &screenplay.imported_title_page {
            render_imported_title_pages(out, imported_title_page, options.paginated);
        } else if let Some(title_page) = TitlePage::from_screenplay(screenplay) {
            render_title_page(out, &title_page, options.paginated);
        }
    }

    if options.paginated {
        render_paginated_body(out, screenplay, layout_profile, options);
        return;
    }

    if options.exact_wraps {
        render_exact_wrap_body(out, screenplay, layout_profile, options);
        return;
    }

    out.push_str("        <section class=\"body\">\n");
    for element in &screenplay.elements {
        match element {
            Element::DialogueBlock(block) => {
                out.push_str("    <div class=\"dialogueBlock\">\n");
                for child in block {
                    render_paragraph(out, child, layout_profile);
                }
                out.push_str("                </div>\n");
            }
            Element::DualDialogueBlock(blocks) => {
                out.push_str("                <div class=\"dualDialogueBlock\">\n");
                for block in blocks {
                    out.push_str("                    <div class=\"dialogueBlock\">\n");
                    if let Element::DialogueBlock(dialogue_block) = block {
                        for child in dialogue_block {
                            render_paragraph(out, child, layout_profile);
                        }
                    }
                    out.push_str("                    </div>\n");
                }
                out.push_str("                </div>\n");
            }
            _ => render_paragraph(out, element, layout_profile),
        }
    }
    out.push_str("        </section>\n");
}

fn render_exact_wrap_body(
    out: &mut String,
    screenplay: &Screenplay,
    layout_profile: &ScreenplayLayoutProfile,
    options: &HtmlRenderOptions,
) {
    out.push_str("        <section class=\"body exactWrapBody\">\n");
    for line in render_unpaginated_visual_lines_with_options(
        screenplay,
        VisualRenderOptions {
            render_continueds: options.render_continueds,
            render_title_page: options.render_title_page,
        },
    ) {
        render_visual_line(out, &line, layout_profile);
    }
    out.push_str("        </section>\n");
}

fn render_paginated_body(
    out: &mut String,
    screenplay: &Screenplay,
    layout_profile: &ScreenplayLayoutProfile,
    options: &HtmlRenderOptions,
) {
    out.push_str("        <section class=\"body paginatedBody\">\n");

    for page in render_paginated_visual_pages_with_options(
        screenplay,
        VisualRenderOptions {
            render_continueds: options.render_continueds,
            render_title_page: options.render_title_page,
        },
    ) {
        write!(
            out,
            "            <section class=\"page{}\" data-page-number=\"{}\">\n",
            if page.page.metadata.index == 0 {
                " firstPage"
            } else {
                ""
            },
            page.page.metadata.number
        )
        .unwrap();

        out.push_str("                <div class=\"pageHeader\">");
        if let Some(display_number) = display_page_number(&page.page) {
            write!(out, "<span class=\"pageNumber\">{}.</span>", display_number).unwrap();
        }
        out.push_str("</div>\n");
        out.push_str("                <div class=\"pageBody\">\n");
        for line in page.lines {
            render_visual_line(out, &line, layout_profile);
        }
        out.push_str("                </div>\n");
        out.push_str("            </section>\n");
    }

    out.push_str("        </section>\n");
}

fn render_visual_line(
    out: &mut String,
    line: &VisualLine,
    layout_profile: &ScreenplayLayoutProfile,
) {
    let mut classes = vec!["visualLine"];
    if line.text.is_empty() {
        classes.push("blankLine");
    }
    if let Some(element_type) = line.element_type {
        classes.push(visual_line_class_name(element_type));
        if let Some(style) = default_styles_for_element_type(element_type, layout_profile) {
            if style.bold {
                classes.push("bold");
            }
            if style.italic {
                classes.push("italic");
            }
            if style.underline {
                classes.push("underline");
            }
        }
    }
    if !line.counted {
        classes.push("uncountedLine");
    }
    if line.centered {
        classes.push("centeredLine");
    }
    if line.dual.is_some() {
        classes.push("dualDialogueLine");
    }

    write!(
        out,
        "                    <div class=\"{}\">",
        classes.join(" ")
    )
    .unwrap();
    if line.text.is_empty() {
        out.push_str("&nbsp;");
    } else if let Some(dual) = &line.dual {
        render_visual_dual_line(out, dual, layout_profile);
    } else {
        if let Some(scene_number) = &line.scene_number {
            write!(
                out,
                "<span class=\"sceneNumberLeft\">{}</span>",
                escape_html(scene_number)
            )
            .unwrap();
            render_visual_fragments(out, &line.fragments);
            write!(
                out,
                "<span class=\"sceneNumberRight\">{}</span>",
                escape_html(scene_number)
            )
            .unwrap();
        } else {
            render_visual_fragments(out, &line.fragments);
        }
    }
    out.push_str("</div>\n");
}

fn render_visual_dual_line(
    out: &mut String,
    dual: &crate::pagination::visual_lines::VisualDualLine,
    layout_profile: &ScreenplayLayoutProfile,
) {
    if let Some(left) = &dual.left {
        render_visual_dual_side(out, left, layout_profile);
    }
    if let Some(right) = &dual.right {
        render_visual_dual_side(out, right, layout_profile);
    }
}

fn render_visual_dual_side(
    out: &mut String,
    side: &crate::pagination::visual_lines::VisualDualSide,
    layout_profile: &ScreenplayLayoutProfile,
) {
    let class_name = visual_line_class_name(side.element_type);
    write!(
        out,
        "<span class=\"dualSegment {}\" style=\"left: {};\">",
        class_name,
        dual_side_left_offset_css(side, layout_profile),
    )
    .unwrap();
    render_visual_fragments(out, &side.fragments);
    out.push_str("</span>");
}

fn dual_side_left_offset_css(
    side: &crate::pagination::visual_lines::VisualDualSide,
    layout_profile: &ScreenplayLayoutProfile,
) -> String {
    let geometry = layout_profile.to_pagination_geometry();
    let mut left = match side.element_type {
        ElementType::DualDialogueLeft => geometry.dual_dialogue_left_left,
        ElementType::DualDialogueRight => geometry.dual_dialogue_right_left,
        ElementType::DualDialogueCharacterLeft => {
            dual_dialogue_character_left_indent(&side.text, 1)
        }
        ElementType::DualDialogueCharacterRight => {
            dual_dialogue_character_left_indent(&side.text, 2)
        }
        ElementType::DualDialogueParentheticalLeft => {
            geometry.dual_dialogue_left_parenthetical_left
        }
        ElementType::DualDialogueParentheticalRight => {
            geometry.dual_dialogue_right_parenthetical_left
        }
        _ => geometry.action_left,
    };
    if hangs_opening_parenthesis(side.element_type, &side.text) {
        left -= 1.0 / geometry.cpi;
    }
    format!("{}in", left - geometry.action_left)
}

fn hangs_opening_parenthesis(element_type: ElementType, text: &str) -> bool {
    matches!(element_type, ElementType::Parenthetical) && text.starts_with('(')
}

fn render_visual_fragments(
    out: &mut String,
    fragments: &[crate::pagination::visual_lines::VisualFragment],
) {
    for fragment in fragments {
        if fragment.styles.is_empty() {
            out.push_str(&escape_html(&fragment.text));
        } else {
            let classes = fragment
                .styles
                .iter()
                .map(|style| style.to_lowercase())
                .collect::<Vec<_>>();
            write!(out, "<span class=\"{}\">", classes.join(" ")).unwrap();
            out.push_str(&escape_html(&fragment.text));
            out.push_str("</span>");
        }
    }
}

fn render_paragraph(out: &mut String, element: &Element, layout_profile: &ScreenplayLayoutProfile) {
    let (type_name, text, attributes) = match element {
        Element::Action(text, attributes)
        | Element::Character(text, attributes)
        | Element::SceneHeading(text, attributes)
        | Element::Lyric(text, attributes)
        | Element::Parenthetical(text, attributes)
        | Element::Dialogue(text, attributes)
        | Element::Transition(text, attributes)
        | Element::ColdOpening(text, attributes)
        | Element::NewAct(text, attributes)
        | Element::EndOfAct(text, attributes) => (element.name(), text, attributes),
        Element::Section(text, attributes, _) => ("Section", text, attributes),
        Element::Synopsis(text) => ("Synopsis", text, &Attributes::default()),
        Element::DialogueBlock(_) | Element::DualDialogueBlock(_) | Element::PageBreak => return,
    };

    write!(
        out,
        "                <p class=\"{}{}",
        class_name(type_name),
        if attributes.starts_new_page {
            " startsNewPage"
        } else {
            ""
        }
    )
    .unwrap();
    if let Some(style) = default_styles_for_type_name(type_name, layout_profile) {
        if style.bold {
            out.push_str(" bold");
        }
        if style.italic {
            out.push_str(" italic");
        }
        if style.underline {
            out.push_str(" underline");
        }
    }
    if attributes.centered {
        out.push_str(" centered");
    }
    out.push_str("\">");
    if type_name == "Scene Heading" {
        if let Some(scene_number) = &attributes.scene_number {
            write!(
                out,
                "<span class=\"sceneNumberLeft\">{}</span>",
                escape_html(scene_number)
            )
            .unwrap();
        }
    }
    render_text(out, text);
    if type_name == "Scene Heading" {
        if let Some(scene_number) = &attributes.scene_number {
            write!(
                out,
                "<span class=\"sceneNumberRight\">{}</span>",
                escape_html(scene_number)
            )
            .unwrap();
        }
    }
    out.push_str("</p>\n");
}

fn render_text(out: &mut String, text: &ElementText) {
    match text {
        ElementText::Plain(text) => out.push_str(&escape_html(text)),
        ElementText::Styled(runs) => {
            for run in runs {
                let classes = sorted_style_names(run, false);
                if classes.is_empty() {
                    out.push_str(&escape_html(&run.content));
                } else {
                    write!(out, "<span class=\"{}\">", classes.join(" ")).unwrap();
                    out.push_str(&escape_html(&run.content));
                    out.push_str("</span>");
                }
            }
        }
    }
}

fn default_styles_for_type_name<'a>(
    type_name: &str,
    layout_profile: &'a ScreenplayLayoutProfile,
) -> Option<&'a crate::pagination::layout_profile::ScreenplayElementStyle> {
    match type_name {
        "Action" => Some(&layout_profile.styles.action),
        "Scene Heading" => Some(&layout_profile.styles.scene_heading),
        "Character" => Some(&layout_profile.styles.character),
        "Dialogue" => Some(&layout_profile.styles.dialogue),
        "Parenthetical" => Some(&layout_profile.styles.parenthetical),
        "Transition" => Some(&layout_profile.styles.transition),
        "Lyric" => Some(&layout_profile.styles.lyric),
        "Cold Opening" => Some(&layout_profile.styles.cold_opening),
        "New Act" => Some(&layout_profile.styles.new_act),
        "End of Act" => Some(&layout_profile.styles.end_of_act),
        _ => None,
    }
}

fn default_styles_for_element_type<'a>(
    element_type: crate::pagination::wrapping::ElementType,
    layout_profile: &'a ScreenplayLayoutProfile,
) -> Option<&'a crate::pagination::layout_profile::ScreenplayElementStyle> {
    use crate::pagination::wrapping::ElementType as ET;
    match element_type {
        ET::Action => Some(&layout_profile.styles.action),
        ET::SceneHeading => Some(&layout_profile.styles.scene_heading),
        ET::Character => Some(&layout_profile.styles.character),
        ET::Dialogue => Some(&layout_profile.styles.dialogue),
        ET::Parenthetical => Some(&layout_profile.styles.parenthetical),
        ET::Transition => Some(&layout_profile.styles.transition),
        ET::Lyric => Some(&layout_profile.styles.lyric),
        ET::DualDialogueLeft => Some(&layout_profile.styles.dual_dialogue_left_dialogue),
        ET::DualDialogueRight => Some(&layout_profile.styles.dual_dialogue_right_dialogue),
        ET::DualDialogueCharacterLeft => Some(&layout_profile.styles.dual_dialogue_left_character),
        ET::DualDialogueCharacterRight => {
            Some(&layout_profile.styles.dual_dialogue_right_character)
        }
        ET::DualDialogueParentheticalLeft => {
            Some(&layout_profile.styles.dual_dialogue_left_parenthetical)
        }
        ET::DualDialogueParentheticalRight => {
            Some(&layout_profile.styles.dual_dialogue_right_parenthetical)
        }
        ET::ColdOpening => Some(&layout_profile.styles.cold_opening),
        ET::NewAct => Some(&layout_profile.styles.new_act),
        ET::EndOfAct => Some(&layout_profile.styles.end_of_act),
    }
}

fn embedded_courier_prime_font_css(options: &HtmlRenderOptions) -> Option<String> {
    if let Some(css) = &options.embedded_courier_prime_css {
        return Some(css.clone());
    }

    if options.embed_courier_prime {
        return bundled_embedded_courier_prime_font_faces();
    }

    None
}

#[cfg(not(target_arch = "wasm32"))]
fn bundled_embedded_courier_prime_font_faces() -> Option<String> {
    Some(
        [
            embedded_font_face("Courier Prime", 400, "normal", COURIER_PRIME_REGULAR_TTF),
            embedded_font_face("Courier Prime", 400, "italic", COURIER_PRIME_ITALIC_TTF),
            embedded_font_face("Courier Prime", 700, "normal", COURIER_PRIME_BOLD_TTF),
            embedded_font_face(
                "Courier Prime",
                700,
                "italic",
                COURIER_PRIME_BOLD_ITALIC_TTF,
            ),
        ]
        .join("\n"),
    )
}

#[cfg(target_arch = "wasm32")]
fn bundled_embedded_courier_prime_font_faces() -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn embedded_font_face(
    font_family: &str,
    font_weight: u16,
    font_style: &str,
    bytes: &[u8],
) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    format!(
        "@font-face {{\n  font-family: \"{font_family}\";\n  src: url(data:font/ttf;base64,{}) format(\"truetype\");\n  font-weight: {font_weight};\n  font-style: {font_style};\n}}\n",
        STANDARD.encode(bytes)
    )
}

fn render_title_page(out: &mut String, title_page: &TitlePage, paginated: bool) {
    write!(
        out,
        "    <section class=\"title-page {}\">\n",
        if paginated {
            "paginatedTitlePage"
        } else {
            "unpaginatedTitlePage"
        }
    )
    .unwrap();

    if let Some(block) = title_page.block(TitlePageBlockKind::Title) {
        let default_title_styling = !title_lines_have_explicit_styles(&block.lines);
        out.push_str("        <div class=\"titlePageBlock titlePageTitle\">\n            <h1");
        if default_title_styling {
            out.push_str(" class=\"defaultTitleText\"");
        }
        out.push('>');
        render_title_page_lines(out, &block.lines);
        out.push_str("</h1>\n        </div>\n");
    }

    out.push_str("        <div class=\"titlePageBlock titlePageCenterMeta\">\n");
    for kind in [
        TitlePageBlockKind::Credit,
        TitlePageBlockKind::Author,
        TitlePageBlockKind::Source,
    ] {
        if let Some(block) = title_page.block(kind) {
            out.push_str("            <p>");
            render_title_page_lines(out, &block.lines);
            out.push_str("</p>\n");
        }
    }
    out.push_str("        </div>\n");

    out.push_str("        <div class=\"titlePageBlock titlePageBottom titlePageBottomLeft\">\n");
    if let Some(block) = title_page.block(TitlePageBlockKind::Contact) {
        out.push_str("            <p>");
        render_title_page_lines(out, &block.lines);
        out.push_str("</p>\n");
    }
    out.push_str("        </div>\n");

    out.push_str("        <div class=\"titlePageBlock titlePageBottom titlePageBottomRight\">\n");
    for kind in [TitlePageBlockKind::Draft, TitlePageBlockKind::DraftDate] {
        if let Some(block) = title_page.block(kind) {
            out.push_str("            <p>");
            render_title_page_lines(out, &block.lines);
            out.push_str("</p>\n");
        }
    }
    out.push_str("        </div>\n");

    out.push_str("    </section>\n");
}

fn render_imported_title_pages(
    out: &mut String,
    imported_title_page: &crate::ImportedTitlePage,
    paginated: bool,
) {
    let renders_page_numbers = imported_title_page.header_footer.header_visible
        && imported_title_page.header_footer.header_has_page_number;
    let first_title_page_number = imported_title_page.header_footer.starting_page.unwrap_or(1);

    for (index, page) in imported_title_page.pages.iter().enumerate() {
        write!(
            out,
            "    <section class=\"title-page importedTitlePage {}\" style=\"position: relative; min-height: 11in;{}\">\n",
            if paginated {
                "paginatedTitlePage"
            } else {
                "unpaginatedTitlePage"
            },
            if index + 1 < imported_title_page.pages.len() {
                " break-after: page; page-break-after: always;"
            } else {
                ""
            }
        )
        .unwrap();

        if renders_page_numbers && (index > 0 || imported_title_page.header_footer.header_first_page)
        {
            write!(
                out,
                "      <div class=\"importedTitlePageNumber\" style=\"position: absolute; top: 0.49in; right: 0.81in;\">{}</div>\n",
                lower_roman(first_title_page_number + index as u32)
            )
            .unwrap();
        }

        for paragraph in &page.paragraphs {
            render_imported_title_paragraph(out, paragraph);
        }

        out.push_str("    </section>\n");
    }
}

fn render_imported_title_paragraph(
    out: &mut String,
    paragraph: &crate::ImportedTitlePageParagraph,
) {
    let alignment = match paragraph.alignment {
        crate::ImportedTitlePageAlignment::Center => "center",
        crate::ImportedTitlePageAlignment::Right => "right",
        _ => "left",
    };
    let left_indent = paragraph.left_indent.unwrap_or(match paragraph.alignment {
        crate::ImportedTitlePageAlignment::Center
        | crate::ImportedTitlePageAlignment::Right
        | crate::ImportedTitlePageAlignment::Full => 1.0,
        crate::ImportedTitlePageAlignment::Left => 1.5,
    });
    let width = (7.5 - left_indent).max(0.5);
    let margin_top = paragraph.space_before.unwrap_or(0.0) / 72.0;

    write!(
        out,
        "      <p class=\"importedTitlePageParagraph\" style=\"margin: {}in 0 0 {}in; width: {}in; text-align: {}; white-space: pre-wrap; tab-size: 8;\">",
        margin_top, left_indent, width, alignment
    )
    .unwrap();
    if paragraph.text.plain_text().trim().is_empty() {
        out.push_str("&nbsp;");
    } else {
        render_text(out, &imported_title_display_text(&paragraph.text));
    }
    out.push_str("</p>\n");
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

fn render_title_page_lines(out: &mut String, lines: &[ElementText]) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push_str("<br>");
        }
        render_text(out, line);
    }
}

fn title_lines_have_explicit_styles(lines: &[ElementText]) -> bool {
    lines.iter().any(|line| match line {
        ElementText::Plain(_) => false,
        ElementText::Styled(runs) => runs.iter().any(|run| !run.text_style.is_empty()),
    })
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

fn class_name(type_name: &str) -> &'static str {
    match type_name {
        "Scene Heading" => "sceneHeading",
        "Action" => "action",
        "Character" => "character",
        "Dialogue" => "dialogue",
        "Parenthetical" => "parenthetical",
        "Transition" => "transition",
        "Lyric" => "lyric",
        "Section" => "section",
        "Synopsis" => "synopsis",
        "Cold Opening" => "coldOpening",
        "New Act" => "newAct",
        "End of Act" => "endOfAct",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blank_attributes, p, parse_fdx, tr, Attributes, Element, ElementText, Metadata};

    fn html_options(head: bool, exact_wraps: bool, paginated: bool) -> HtmlRenderOptions {
        HtmlRenderOptions {
            head,
            exact_wraps,
            paginated,
            render_continueds: true,
            render_title_page: true,
            embed_courier_prime: false,
            embedded_courier_prime_css: None,
        }
    }

    #[test]
    fn exact_wrap_html_renders_visual_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(
                p("THIS IS A LONG ACTION LINE THAT SHOULD WRAP WHEN EXACT HTML WRAPS ARE ENABLED"),
                blank_attributes(),
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("exactWraps"));
        assert!(output.contains("visualLine"));
        assert!(output.contains("exactWrapBody"));
        assert!(!output.contains("<p class=\"action"));
    }

    #[test]
    fn html_head_includes_local_courier_prime_font_face_by_default() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(&screenplay, html_options(true, false, false));

        assert!(output.contains("@font-face"));
        assert!(output.contains("CourierPrime-Regular"));
        assert!(!output.contains("data:font/ttf;base64,"));
        assert!(!output.contains("fonts.googleapis.com"));
    }

    #[test]
    fn headless_html_fragment_includes_a_style_block() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(&screenplay, html_options(false, false, false));

        assert!(output.starts_with("  <style type=\"text/css\">"));
        assert!(output.contains(".screenplay {"));
        assert!(output.contains("<section class=\"screenplay\">"));
    }

    #[test]
    fn html_stylesheet_is_not_screen_only() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(&screenplay, html_options(true, false, false));

        assert!(output.contains("<style type=\"text/css\">"));
        assert!(!output.contains("media=\"screen\""));
    }

    #[test]
    fn html_can_embed_courier_prime_font_data() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                embed_courier_prime: true,
                ..html_options(true, false, false)
            },
        );

        assert!(output.contains("data:font/ttf;base64,"));
        assert!(output.contains("font-family: \"Courier Prime\";"));
    }

    #[test]
    fn html_can_use_runtime_supplied_embedded_courier_prime_css() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                embedded_courier_prime_css: Some(embedded_courier_prime_css_from_base64(
                    "regular",
                    "italic",
                    "bold",
                    "bolditalic",
                )),
                ..html_options(true, false, false)
            },
        );

        assert!(output.contains("data:font/ttf;base64,regular"));
        assert!(output.contains("data:font/ttf;base64,italic"));
        assert!(output.contains("data:font/ttf;base64,bold"));
        assert!(output.contains("data:font/ttf;base64,bolditalic"));
    }

    #[test]
    fn exact_wrap_html_preserves_styled_spans_for_unsplit_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(
                ElementText::Styled(vec![
                    tr("BOLD", vec!["Bold"]),
                    tr(" plain", vec![]),
                    tr(" ITALIC", vec!["Italic"]),
                ]),
                blank_attributes(),
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains(
            "<span class=\"bold\">BOLD</span> plain<span class=\"italic\"> ITALIC</span>"
        ));
    }

    #[test]
    fn exact_wrap_html_marks_centered_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(
                p("THE END"),
                Attributes {
                    centered: true,
                    ..blank_attributes()
                },
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("visualLine"));
        assert!(output.contains("centeredLine"));
        assert!(output.contains(">THE END</div>"));
    }

    #[test]
    fn exact_wrap_html_marks_visual_lines_with_element_classes() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::NewAct(
                p("ACT TWO"),
                Attributes {
                    centered: true,
                    starts_new_page: true,
                    ..Attributes::default()
                },
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("visualLine newAct underline centeredLine"));
    }

    #[test]
    fn exact_wrap_html_hangs_the_opening_parenthesis_one_cell_left() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::DialogueBlock(vec![
                Element::Character(p("ALEX"), blank_attributes()),
                Element::Parenthetical(p("(quietly)"), blank_attributes()),
            ])],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output
            .contains("<div class=\"visualLine parenthetical\">              (quietly)</div>"));
    }

    #[test]
    fn exact_wrap_html_underlines_new_acts_by_default() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::NewAct(
                p("ACT TWO"),
                Attributes {
                    centered: true,
                    starts_new_page: true,
                    ..Attributes::default()
                },
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("visualLine newAct underline centeredLine"));
    }

    #[test]
    fn exact_wrap_html_underlines_cold_openings_by_default() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::ColdOpening(
                p("COLD OPENING"),
                Attributes {
                    centered: true,
                    ..Attributes::default()
                },
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("visualLine coldOpening underline centeredLine"));
    }

    #[test]
    fn fmt_can_disable_default_act_underlines_in_exact_wrap_html() {
        let mut metadata = Metadata::new();
        metadata.insert("fmt".into(), vec!["no-act-underlines".into()]);
        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::NewAct(
                p("ACT TWO"),
                Attributes {
                    centered: true,
                    starts_new_page: true,
                    ..Attributes::default()
                },
            )],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains("visualLine newAct centeredLine"));
        assert!(!output.contains("visualLine newAct underline centeredLine"));
    }

    #[test]
    fn title_page_html_preserves_styled_metadata_and_line_breaks() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "title".into(),
            vec![
                ElementText::Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
                ElementText::Styled(vec![tr("FULL RETIRED", vec!["Bold", "Underline"])]),
            ],
        );
        metadata.insert("credit".into(), vec!["Written by".into()]);
        metadata.insert(
            "author".into(),
            vec![ElementText::Styled(vec![tr(
                "Stu Maschwitz",
                vec!["Italic"],
            )])],
        );
        metadata.insert("source".into(), vec!["Based on a true story".into()]);
        metadata.insert("contact".into(), vec!["CAA".into(), "Los Angeles".into()]);
        metadata.insert("draft".into(), vec!["Blue Draft".into()]);
        metadata.insert("draft date".into(), vec!["1/27/2012".into()]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        let output = render_document(&screenplay, html_options(false, false, false));

        assert!(output.contains("<section class=\"title-page unpaginatedTitlePage\">"));
        assert!(output.contains("<span class=\"bold underline\">BRICK &amp; STEEL</span><br><span class=\"bold underline\">FULL RETIRED</span>"));
        assert!(output.contains("<p>Written by</p>"));
        assert!(output.contains("<p><span class=\"italic\">Stu Maschwitz</span></p>"));
        assert!(output.contains("titlePageCenterMeta"));
        assert!(output.contains("<p>Based on a true story</p>"));
        assert!(output.contains("titlePageBottomLeft"));
        assert!(output.contains("<p>CAA<br>Los Angeles</p>"));
        assert!(output.contains("titlePageBottomRight"));
        assert!(output.contains("<p>Blue Draft</p>"));
        assert!(output.contains("<p>1/27/2012</p>"));
    }

    #[test]
    fn imported_fdx_title_pages_render_from_raw_paragraph_pages_in_html() {
        let xml = std::fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
            .expect("fixture should load");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let output = render_document(&screenplay, html_options(false, false, false));

        assert!(output.contains("importedTitlePage"));
        assert!(output.contains("GUY TEXT"));
        assert!(output.contains("THE GUYS"));
        assert!(!output.contains("<div class=\"titlePageBlock titlePageCenterMeta\">"));
    }

    #[test]
    fn imported_fdx_title_pages_honor_all_caps_styles_in_html() {
        let xml = std::fs::read_to_string(
            "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fdx",
        )
        .expect("fixture should load");
        let screenplay = parse_fdx(&xml).expect("fdx should parse");

        let output = render_document(&screenplay, html_options(false, false, true));

        assert!(output.contains("BIG FISH"));
        assert!(!output.contains(">Big Fish<"));
    }

    #[test]
    fn title_page_plain_title_uses_default_emphasis_but_styled_title_does_not() {
        let mut plain_metadata = Metadata::new();
        plain_metadata.insert("title".into(), vec!["Big Fish".into()]);

        let plain_output = render_document(
            &Screenplay {
                metadata: plain_metadata,
                imported_layout: None,
                imported_title_page: None,
                elements: vec![],
            },
            html_options(false, false, false),
        );

        assert!(plain_output.contains("<h1 class=\"defaultTitleText\">Big Fish</h1>"));

        let mut styled_metadata = Metadata::new();
        styled_metadata.insert(
            "title".into(),
            vec![ElementText::Styled(vec![tr("Big Fish", vec!["Italic"])])],
        );

        let styled_output = render_document(
            &Screenplay {
                metadata: styled_metadata,
                imported_layout: None,
                imported_title_page: None,
                elements: vec![],
            },
            html_options(false, false, false),
        );

        assert!(styled_output.contains("<h1><span class=\"italic\">Big Fish</span></h1>"));
        assert!(!styled_output.contains("<h1 class=\"defaultTitleText\">"));
    }

    #[test]
    fn html_render_options_can_suppress_title_page() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["Big Fish".into()]);

        let output = render_document(
            &Screenplay {
                metadata,
                imported_layout: None,
                imported_title_page: None,
                elements: vec![Element::Action(p("BODY"), blank_attributes())],
            },
            HtmlRenderOptions {
                render_title_page: false,
                ..html_options(false, false, false)
            },
        );

        assert!(!output.contains("<section class=\"title-page"));
        assert!(output.contains("BODY"));
    }

    #[test]
    fn paginated_title_page_uses_paginated_title_page_class() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["Big Fish".into()]);

        let output = render_document(
            &Screenplay {
                metadata,
                imported_layout: None,
                imported_title_page: None,
                elements: vec![],
            },
            html_options(false, false, true),
        );

        assert!(output.contains("<section class=\"title-page paginatedTitlePage\">"));
        assert!(!output.contains("<section class=\"title-page unpaginatedTitlePage\">"));
    }

    #[test]
    fn paginated_html_renders_page_containers_and_hides_first_page_number() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
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

        let output = render_document(&screenplay, html_options(false, false, true));

        assert!(output.contains("paginatedHtml"));
        assert!(output.contains("class=\"page firstPage\""));
        assert!(output.contains("data-page-number=\"2\""));
        assert!(output.contains("<span class=\"pageNumber\">2.</span>"));
        assert!(!output.contains("<span class=\"pageNumber\">1.</span>"));
    }

    #[test]
    fn paginated_html_preserves_styled_spans_for_split_flow_fragments() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(
                ElementText::Styled(vec![tr(&"BOLD SENTENCE. ".repeat(500), vec!["Bold"])]),
                blank_attributes(),
            )],
        };

        let output = render_document(&screenplay, html_options(false, false, true));

        let second_page = output
            .split("data-page-number=\"2\"")
            .nth(1)
            .expect("expected a second paginated page");

        assert!(second_page.contains("<span class=\"bold\">"));
    }

    #[test]
    fn paginated_html_uses_a4_page_dimensions_from_metadata() {
        let mut metadata = Metadata::new();
        metadata.insert("fmt".into(), vec!["a4".into()]);
        metadata.insert("title".into(), vec!["A4 Sample".into()]);
        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("FIRST PAGE"), blank_attributes())],
        };

        let output = render_document(&screenplay, html_options(true, false, true));

        assert!(output.contains("@page screenplay, screenplay-title {"));
        assert!(output.contains("size: 8.26in 11.69in;"));
        assert!(output.contains(".screenplay.paginatedHtml .page {"));
        assert!(output.contains("width: 8.26in;"));
        assert!(output.contains("min-height: 11.69in;"));
        assert!(output.contains("padding-right: 0.76in;"));
        assert!(output.contains(".screenplay .title-page {"));
    }

    #[test]
    fn headless_paginated_html_includes_a4_layout_css() {
        let mut metadata = Metadata::new();
        metadata.insert("fmt".into(), vec!["a4".into()]);
        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::Action(p("FIRST PAGE"), blank_attributes())],
        };

        let output = render_document(&screenplay, html_options(false, false, true));

        assert!(output.starts_with("  <style type=\"text/css\">"));
        assert!(output.contains("size: 8.26in 11.69in;"));
        assert!(output.contains("width: 8.26in;"));
        assert!(output.contains("<section class=\"screenplay exactWraps paginatedHtml\">"));
    }

    #[test]
    fn paginated_html_preserves_styled_spans_for_dual_dialogue() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(
                        ElementText::Styled(vec![tr("BRICK", vec!["Bold"])]),
                        blank_attributes(),
                    ),
                    Element::Dialogue(
                        ElementText::Styled(vec![tr("Left side.", vec!["Italic"])]),
                        blank_attributes(),
                    ),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(
                        ElementText::Styled(vec![tr("STEEL", vec!["Underline"])]),
                        blank_attributes(),
                    ),
                    Element::Dialogue(
                        ElementText::Styled(vec![tr("Right side.", vec!["Bold"])]),
                        blank_attributes(),
                    ),
                ]),
            ])],
        };

        let output = render_document(&screenplay, html_options(false, false, true));

        assert!(output.contains("<span class=\"bold\">BRICK</span>"));
        assert!(output.contains("<span class=\"italic\">Left side.</span>"));
        assert!(output.contains("<span class=\"underline\">STEEL</span>"));
        assert!(output.contains("<span class=\"bold\">Right side.</span>"));
    }

    #[test]
    fn paginated_html_uses_distinct_dual_offsets_for_character_dialogue_and_parenthetical() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            imported_title_page: None,
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(p("BRICK"), blank_attributes()),
                    Element::Parenthetical(p("(quietly)"), blank_attributes()),
                    Element::Dialogue(p("Left side."), blank_attributes()),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(p("STEEL"), blank_attributes()),
                    Element::Parenthetical(p("(loudly)"), blank_attributes()),
                    Element::Dialogue(p("Right side."), blank_attributes()),
                ]),
            ])],
        };

        let output = render_document(&screenplay, html_options(false, false, true));

        assert!(output.contains("visualLine dialogue dualDialogueLine"));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueCharacterLeft\" style=\"left: 1.1944444in;\""
        ));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueParentheticalLeft\" style=\"left: 0.25in;\""
        ));
        assert!(output.contains("class=\"dualSegment dualDialogueLeft\" style=\"left: 0in;\""));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueCharacterRight\" style=\"left: 4.3194447in;\""
        ));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueParentheticalRight\" style=\"left: 3.375in;\""
        ));
        assert!(output.contains("class=\"dualSegment dualDialogueRight\" style=\"left: 3.125in;\""));
    }
}
