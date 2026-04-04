use super::shared::{escape_html, join_metadata, sorted_style_names};
use crate::html_output::HtmlRenderOptions;
use crate::pagination::margin::dual_dialogue_character_left_indent;
use crate::pagination::wrapping::ElementType;
use crate::pagination::{ScreenplayLayoutProfile, StyleProfile};
use crate::title_page::{TitlePage, TitlePageBlockKind};
use crate::visual_lines::{
    display_page_number, render_paginated_visual_pages_with_options,
    render_unpaginated_visual_lines_with_options, visual_line_class_name, VisualLine,
    VisualRenderOptions,
};
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

pub(crate) fn render_document(screenplay: &Screenplay, options: HtmlRenderOptions) -> String {
    let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
    let mut out = String::with_capacity(32 * 1024);
    if options.head {
        out.push_str("<!doctype html>\n\n<html>\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\n  <title>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "title",
            " ",
        )));
        out.push_str("</title>\n\n  <style type=\"text/css\" media=\"screen\">\n   ");
        if let Some(font_css) = embedded_courier_prime_font_css(&options) {
            out.push_str(&font_css);
            out.push('\n');
        }
        out.push_str(HTML_STYLE);
        out.push_str("\n  </style>\n</head>\n\n<body");
        if options.paginated {
            out.push_str(" class=\"paginatedHtmlView\"");
        }
        out.push_str(">\n");
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
    if let Some(title_page) = TitlePage::from_metadata(&screenplay.metadata) {
        render_title_page(out, &title_page, options.paginated);
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
        if default_underlines_for_element_type(element_type, layout_profile) {
            classes.push("underline");
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
        render_visual_fragments(out, &line.fragments);
    }
    out.push_str("</div>\n");
}

fn render_visual_dual_line(
    out: &mut String,
    dual: &crate::visual_lines::VisualDualLine,
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
    side: &crate::visual_lines::VisualDualSide,
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
    side: &crate::visual_lines::VisualDualSide,
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
    matches!(
        element_type,
        ElementType::Parenthetical
            | ElementType::DualDialogueParentheticalLeft
            | ElementType::DualDialogueParentheticalRight
    ) && text.starts_with('(')
}

fn render_visual_fragments(out: &mut String, fragments: &[crate::visual_lines::VisualFragment]) {
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
    if default_underlines_for_type_name(type_name, layout_profile) {
        out.push_str(" underline");
    }
    if attributes.centered {
        out.push_str(" centered");
    }
    out.push_str("\">");
    render_text(out, text);
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

fn default_underlines_for_type_name(
    type_name: &str,
    layout_profile: &ScreenplayLayoutProfile,
) -> bool {
    match type_name {
        "Cold Opening" => layout_profile.styles.cold_opening.underline,
        "New Act" => layout_profile.styles.new_act.underline,
        "End of Act" => layout_profile.styles.end_of_act.underline,
        _ => false,
    }
}

fn default_underlines_for_element_type(
    element_type: crate::pagination::wrapping::ElementType,
    layout_profile: &ScreenplayLayoutProfile,
) -> bool {
    match element_type {
        crate::pagination::wrapping::ElementType::ColdOpening => {
            layout_profile.styles.cold_opening.underline
        }
        crate::pagination::wrapping::ElementType::NewAct => layout_profile.styles.new_act.underline,
        crate::pagination::wrapping::ElementType::EndOfAct => {
            layout_profile.styles.end_of_act.underline
        }
        _ => false,
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
    use crate::{blank_attributes, p, tr, Attributes, Element, ElementText, Metadata};

    fn html_options(head: bool, exact_wraps: bool, paginated: bool) -> HtmlRenderOptions {
        HtmlRenderOptions {
            head,
            exact_wraps,
            paginated,
            render_continueds: true,
            embed_courier_prime: false,
            embedded_courier_prime_css: None,
        }
    }

    #[test]
    fn exact_wrap_html_renders_visual_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
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
            elements: vec![],
        };

        let output = render_document(&screenplay, html_options(true, false, false));

        assert!(output.contains("@font-face"));
        assert!(output.contains("CourierPrime-Regular"));
        assert!(!output.contains("data:font/ttf;base64,"));
        assert!(!output.contains("fonts.googleapis.com"));
    }

    #[test]
    fn html_can_embed_courier_prime_font_data() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
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
            elements: vec![],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                embedded_courier_prime_css: Some(
                    crate::html_output::embedded_courier_prime_css_from_base64(
                        "regular",
                        "italic",
                        "bold",
                        "bolditalic",
                    ),
                ),
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
            elements: vec![Element::DialogueBlock(vec![
                Element::Character(p("ALEX"), blank_attributes()),
                Element::Parenthetical(p("(quietly)"), blank_attributes()),
            ])],
        };

        let output = render_document(&screenplay, html_options(false, true, false));

        assert!(output.contains(
            "<div class=\"visualLine parenthetical\">              (quietly)</div>"
        ));
    }

    #[test]
    fn exact_wrap_html_underlines_new_acts_by_default() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
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
    fn title_page_plain_title_uses_default_emphasis_but_styled_title_does_not() {
        let mut plain_metadata = Metadata::new();
        plain_metadata.insert("title".into(), vec!["Big Fish".into()]);

        let plain_output = render_document(
            &Screenplay {
                metadata: plain_metadata,
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
                elements: vec![],
            },
            html_options(false, false, false),
        );

        assert!(styled_output.contains("<h1><span class=\"italic\">Big Fish</span></h1>"));
        assert!(!styled_output.contains("defaultTitleText"));
    }

    #[test]
    fn paginated_title_page_uses_paginated_title_page_class() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["Big Fish".into()]);

        let output = render_document(
            &Screenplay {
                metadata,
                elements: vec![],
            },
            html_options(false, false, true),
        );

        assert!(output.contains("<section class=\"title-page paginatedTitlePage\">"));
        assert!(!output.contains("unpaginatedTitlePage"));
    }

    #[test]
    fn paginated_html_renders_page_containers_and_hides_first_page_number() {
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
    fn paginated_html_preserves_styled_spans_for_dual_dialogue() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
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
        assert!(output
            .contains("class=\"dualSegment dualDialogueCharacterLeft\" style=\"left: 1.1944444in;\""));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueParentheticalLeft\" style=\"left: 0.14999998in;\""
        ));
        assert!(output.contains("class=\"dualSegment dualDialogueLeft\" style=\"left: 0in;\""));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueCharacterRight\" style=\"left: 4.3194447in;\""
        ));
        assert!(output.contains(
            "class=\"dualSegment dualDialogueParentheticalRight\" style=\"left: 3.275in;\""
        ));
        assert!(output.contains("class=\"dualSegment dualDialogueRight\" style=\"left: 3.125in;\""));
    }
}
