use super::shared::{escape_xml_attr, escape_xml_text, join_metadata, sorted_style_names};
use crate::pagination::{Alignment, ScreenplayElementStyle, ScreenplayLayoutProfile};
use crate::title_page::plain_title_uses_all_caps;
use crate::{
    Element, ElementText, ImportedTitlePageAlignment, ImportedTitlePageTabStopKind, Metadata,
    Screenplay,
};
use std::fmt::Write;

pub(crate) fn prepare_screenplay(screenplay: &mut Screenplay) {
    add_fdx_formatting(&mut screenplay.metadata);

    screenplay.elements.retain(|e| match e {
        Element::PageBreak | Element::Section(_, _, _) | Element::Synopsis(_) => false,
        _ => true,
    });
}

pub(crate) fn render_document(screenplay: &Screenplay) -> String {
    let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>\n<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"4\">\n    <Content>\n");
    render_content(&mut out, screenplay);
    out.push_str("    </Content>\n\n");
    render_page_layout(&mut out, &layout_profile);
    out.push('\n');
    render_header_and_footer(&mut out, &screenplay.metadata);
    out.push('\n');
    render_element_settings(&mut out, &screenplay.metadata, &layout_profile);
    out.push('\n');
    render_title_page(&mut out, screenplay);
    out.push_str("\n  <MoresAndContinueds>\n");
    write!(
        out,
        "    <FontSpec AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\"/>\n",
        escape_xml_attr(&font_choice(&screenplay.metadata))
    )
    .unwrap();
    out.push_str("    <DialogueBreaks AutomaticCharacterContinueds=\"Yes\" BottomOfPage=\"Yes\" DialogueBottom=\"(MORE)\" DialogueTop=\"(CONT'D)\" TopOfNext=\"Yes\"/>\n    <SceneBreaks ContinuedNumber=\"No\" SceneBottom=\"(CONTINUED)\" SceneBottomOfPage=\"No\" SceneTop=\"CONTINUED:\" SceneTopOfNext=\"No\"/>\n  </MoresAndContinueds>\n\n</FinalDraft>\n");
    out
}

pub(crate) fn add_fdx_formatting(metadata: &mut Metadata) {
    let mut scene_heading_styles = vec!["AllCaps"];
    let mut space_before_heading = "24".to_string();
    let mut action_text_style = "".to_string();
    let mut font_choice = "Courier Prime".to_string();
    let mut style_profile = None;

    if let Some(opts_vec) = metadata.get_mut("fmt") {
        if let Some(opts_string) = opts_vec.first() {
            let opts_string = opts_string.plain_text();
            for option in opts_string.split_whitespace() {
                if option.eq_ignore_ascii_case("multicam") {
                    style_profile = Some("multicam".to_string());
                } else if matches_fmt_option(option, &["bsh", "bold-scene-headings"]) {
                    scene_heading_styles.push("Bold");
                } else if matches_fmt_option(option, &["ush", "underline-scene-headings"]) {
                    scene_heading_styles.push("Underline");
                } else if matches_fmt_option(option, &["acat", "all-caps-action"]) {
                    action_text_style.push_str("AllCaps");
                } else if matches_fmt_option(
                    option,
                    &["ssbsh", "single-space-before-scene-headings"],
                ) {
                    space_before_heading = "12".to_string();
                } else if matches_fmt_option(option, &["cfd", "courier-final-draft"]) {
                    font_choice = "Courier Final Draft".to_string();
                }
            }
        }
    }

    let layout_profile = ScreenplayLayoutProfile::from_metadata(metadata);

    scene_heading_styles.sort_unstable();
    let scene_heading_style: String = scene_heading_styles.join("+");
    insert_metadata_value(metadata, "scene-heading-style", &scene_heading_style);
    insert_metadata_value(metadata, "space-before-heading", &space_before_heading);
    insert_metadata_value(
        metadata,
        "dialogue-spacing",
        &format_spacing(layout_profile.styles.dialogue.line_spacing),
    );
    insert_metadata_value(metadata, "action-text-style", &action_text_style);
    insert_metadata_value(metadata, "font-choice", &font_choice);
    insert_metadata_value(
        metadata,
        "dialogue-left-indent",
        &format_indent(layout_profile.styles.dialogue.left_indent),
    );
    insert_metadata_value(
        metadata,
        "dialogue-right-indent",
        &format_indent(layout_profile.styles.dialogue.right_indent),
    );
    if let Some(profile) = style_profile {
        insert_metadata_value(metadata, "style-profile", &profile);
    }
    if layout_profile.styles.character.right_indent != 7.25 {
        insert_metadata_value(
            metadata,
            "character-right-indent",
            &format_indent(layout_profile.styles.character.right_indent),
        );
    }
    if layout_profile.styles.parenthetical.left_indent != 3.0 {
        insert_metadata_value(
            metadata,
            "parenthetical-left-indent",
            &format_indent(layout_profile.styles.parenthetical.left_indent),
        );
    }
    if layout_profile.styles.transition.right_indent != 7.1 {
        insert_metadata_value(
            metadata,
            "transition-right-indent",
            &format_indent(layout_profile.styles.transition.right_indent),
        );
    }
}

fn matches_fmt_option(option: &str, accepted: &[&str]) -> bool {
    accepted
        .iter()
        .any(|candidate| option.eq_ignore_ascii_case(candidate))
}

pub(crate) fn insert_metadata_value(metadata: &mut Metadata, key: &str, value: &str) {
    metadata.insert(key.to_string(), vec![value.into()]);
}

fn render_content(out: &mut String, screenplay: &Screenplay) {
    for element in &screenplay.elements {
        match element {
            Element::DialogueBlock(block) => {
                for child in block {
                    render_paragraph(out, child);
                }
            }
            Element::DualDialogueBlock(blocks) => {
                out.push_str("      <Paragraph Alignment=\"Left\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.50\" RightIndent=\"7.50\" SpaceBefore=\"12\" Spacing=\"1\" StartsNewPage=\"No\" Type=\"General\">\n              <DualDialogue>\n");
                for block in blocks {
                    if let Element::DialogueBlock(dialogue_block) = block {
                        for child in dialogue_block {
                            render_paragraph(out, child);
                        }
                    }
                }
                out.push_str("              </DualDialogue>\n              <Text></Text>\n          </Paragraph>\n");
            }
            _ => render_paragraph(out, element),
        }
    }
}

fn render_paragraph(out: &mut String, element: &Element) {
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
        Element::DialogueBlock(_) | Element::DualDialogueBlock(_) => return,
        Element::Section(_, _, _) | Element::Synopsis(_) | Element::PageBreak => return,
    };

    write!(
        out,
        "      <Paragraph Type=\"{}\"",
        escape_xml_attr(type_name)
    )
    .unwrap();
    if let Some(scene_number) = &attributes.scene_number {
        write!(out, " Number=\"{}\"", escape_xml_attr(scene_number)).unwrap();
    }
    if attributes.starts_new_page {
        out.push_str(" StartsNewPage=\"Yes\"");
    }
    if attributes.centered {
        out.push_str(" Alignment=\"Center\"");
    }
    out.push_str(">\n");
    render_text(out, text);
    out.push_str("      </Paragraph>\n");
}

fn render_text(out: &mut String, text: &ElementText) {
    match text {
        ElementText::Plain(text) => {
            write!(out, "        <Text>{}</Text>\n", escape_xml_text(text)).unwrap();
        }
        ElementText::Styled(runs) => {
            for run in runs {
                out.push_str("        <Text");
                let styles = sorted_style_names(run, true);
                if !styles.is_empty() {
                    write!(out, " Style=\"{}\"", escape_xml_attr(&styles.join("+"))).unwrap();
                }
                write!(out, ">{}</Text>\n", escape_xml_text(&run.content)).unwrap();
            }
        }
    }
}

fn render_header_and_footer(out: &mut String, metadata: &Metadata) {
    let font = escape_xml_attr(&font_choice(metadata));
    out.push_str("    <HeaderAndFooter FooterFirstPage=\"Yes\" FooterVisible=\"No\" HeaderFirstPage=\"No\" HeaderVisible=\"Yes\" StartingPage=\"1\">\n        <Header>\n            <Paragraph Alignment=\"Left\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.25\" RightIndent=\"-1.00\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n");
    write!(out, "                <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\">\t</Text>\n", font).unwrap();
    write!(out, "                <DynamicLabel AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\" Type=\"Collated Revisions\"/>\n", font).unwrap();
    write!(out, "                <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\">\t</Text>\n", font).unwrap();
    write!(out, "                <DynamicLabel AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\" Type=\"Page #\"/>\n", font).unwrap();
    write!(out, "                <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\">.</Text>\n", font).unwrap();
    out.push_str("                <Tabstops>\n                    <Tabstop Position=\"4.50\" Type=\"Center\"/>\n                    <Tabstop Position=\"7.25\" Type=\"Right\"/>\n                </Tabstops>\n            </Paragraph>\n        </Header>\n        <Footer>\n            <Paragraph Alignment=\"Right\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.25\" RightIndent=\"-1.25\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n");
    write!(out, "                <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\"> </Text>\n", font).unwrap();
    out.push_str("            </Paragraph>\n        </Footer>\n    </HeaderAndFooter>\n");
}

fn render_page_layout(out: &mut String, layout_profile: &ScreenplayLayoutProfile) {
    let top_margin = (layout_profile.top_margin * 72.0).round() as i32;
    let bottom_margin = (layout_profile.bottom_margin * 72.0).round() as i32;
    let header_margin = (layout_profile.header_margin * 72.0).round() as i32;
    let footer_margin = (layout_profile.footer_margin * 72.0).round() as i32;

    writeln!(
        out,
        "    <PageLayout BackgroundColor=\"#FFFFFFFFFFFF\" BottomMargin=\"{}\" BreakDialogueAndActionAtSentences=\"Yes\" DocumentLeading=\"Normal\" FooterMargin=\"{}\" ForegroundColor=\"#000000000000\" HeaderMargin=\"{}\" InvisiblesColor=\"#C0C0C0C0C0C0\" TopMargin=\"{}\" UsesSmartQuotes=\"Yes\">",
        bottom_margin, footer_margin, header_margin, top_margin
    )
    .unwrap();
    writeln!(
        out,
        "      <PageSize Height=\"{:.2}\" Width=\"{:.2}\"/>",
        layout_profile.page_height, layout_profile.page_width
    )
    .unwrap();
    out.push_str("      <AutoCastList AddParentheses=\"Yes\" AutomaticallyGenerate=\"Yes\" CastListElement=\"Cast List\"/>\n");
    out.push_str("    </PageLayout>\n");
}

fn render_element_settings(
    out: &mut String,
    metadata: &Metadata,
    layout_profile: &ScreenplayLayoutProfile,
) {
    struct ElementSetting<'a> {
        type_name: &'a str,
        style: String,
        alignment: String,
        first_indent: &'a str,
        left_indent: String,
        right_indent: String,
        space_before: String,
        spacing: String,
        starts_new_page: String,
        paginate_as: &'a str,
        return_key: &'a str,
        shortcut: &'a str,
    }

    let settings = [
        ElementSetting {
            type_name: "General",
            style: String::new(),
            alignment: "Left".to_string(),
            first_indent: "0.00",
            left_indent: "1.50".to_string(),
            right_indent: "7.50".to_string(),
            space_before: "0".to_string(),
            spacing: "1".to_string(),
            starts_new_page: "No".to_string(),
            paginate_as: "General",
            return_key: "General",
            shortcut: "0",
        },
        ElementSetting {
            type_name: "Scene Heading",
            style: metadata_value(metadata, "scene-heading-style"),
            alignment: alignment_name(layout_profile.styles.scene_heading.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.scene_heading.left_indent),
            right_indent: format_indent(layout_profile.styles.scene_heading.right_indent),
            space_before: format_space_before(&layout_profile.styles.scene_heading),
            spacing: format_spacing(layout_profile.styles.scene_heading.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.scene_heading.starts_new_page,
            ),
            paginate_as: "Scene Heading",
            return_key: "Action",
            shortcut: "1",
        },
        ElementSetting {
            type_name: "Action",
            style: metadata_value(metadata, "action-text-style"),
            alignment: alignment_name(layout_profile.styles.action.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.action.left_indent),
            right_indent: format_indent(layout_profile.styles.action.right_indent),
            space_before: format_space_before(&layout_profile.styles.action),
            spacing: format_spacing(layout_profile.styles.action.line_spacing),
            starts_new_page: format_starts_new_page(layout_profile.styles.action.starts_new_page),
            paginate_as: "Action",
            return_key: "Action",
            shortcut: "2",
        },
        ElementSetting {
            type_name: "Character",
            style: "AllCaps".to_string(),
            alignment: alignment_name(layout_profile.styles.character.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.character.left_indent),
            right_indent: format_indent(layout_profile.styles.character.right_indent),
            space_before: format_space_before(&layout_profile.styles.character),
            spacing: format_spacing(layout_profile.styles.character.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.character.starts_new_page,
            ),
            paginate_as: "Character",
            return_key: "Dialogue",
            shortcut: "3",
        },
        ElementSetting {
            type_name: "Parenthetical",
            style: String::new(),
            alignment: alignment_name(layout_profile.styles.parenthetical.alignment),
            first_indent: "-0.10",
            left_indent: format_indent(layout_profile.styles.parenthetical.left_indent),
            right_indent: format_indent(layout_profile.styles.parenthetical.right_indent),
            space_before: format_space_before(&layout_profile.styles.parenthetical),
            spacing: format_spacing(layout_profile.styles.parenthetical.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.parenthetical.starts_new_page,
            ),
            paginate_as: "Parenthetical",
            return_key: "Dialogue",
            shortcut: "4",
        },
        ElementSetting {
            type_name: "Dialogue",
            style: String::new(),
            alignment: alignment_name(layout_profile.styles.dialogue.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.dialogue.left_indent),
            right_indent: format_indent(layout_profile.styles.dialogue.right_indent),
            space_before: format_space_before(&layout_profile.styles.dialogue),
            spacing: format_spacing(layout_profile.styles.dialogue.line_spacing),
            starts_new_page: format_starts_new_page(layout_profile.styles.dialogue.starts_new_page),
            paginate_as: "Dialogue",
            return_key: "Action",
            shortcut: "5",
        },
        ElementSetting {
            type_name: "Transition",
            style: "AllCaps".to_string(),
            alignment: alignment_name(layout_profile.styles.transition.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.transition.left_indent),
            right_indent: format_indent(layout_profile.styles.transition.right_indent),
            space_before: format_space_before(&layout_profile.styles.transition),
            spacing: format_spacing(layout_profile.styles.transition.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.transition.starts_new_page,
            ),
            paginate_as: "Transition",
            return_key: "Scene Heading",
            shortcut: "6",
        },
        ElementSetting {
            type_name: "Shot",
            style: "AllCaps".to_string(),
            alignment: "Left".to_string(),
            first_indent: "0.00",
            left_indent: "1.50".to_string(),
            right_indent: "7.50".to_string(),
            space_before: "12".to_string(),
            spacing: "1".to_string(),
            starts_new_page: "No".to_string(),
            paginate_as: "Scene Heading",
            return_key: "Action",
            shortcut: "7",
        },
        ElementSetting {
            type_name: "Cast List",
            style: "AllCaps".to_string(),
            alignment: "Left".to_string(),
            first_indent: "0.00",
            left_indent: "1.50".to_string(),
            right_indent: "7.50".to_string(),
            space_before: "0".to_string(),
            spacing: "1".to_string(),
            starts_new_page: "No".to_string(),
            paginate_as: "Action",
            return_key: "Action",
            shortcut: "8",
        },
        ElementSetting {
            type_name: "New Act",
            style: "Underline+AllCaps".to_string(),
            alignment: alignment_name(layout_profile.styles.new_act.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.new_act.left_indent),
            right_indent: format_indent(layout_profile.styles.new_act.right_indent),
            space_before: format_space_before(&layout_profile.styles.new_act),
            spacing: format_spacing(layout_profile.styles.new_act.line_spacing),
            starts_new_page: format_starts_new_page(layout_profile.styles.new_act.starts_new_page),
            paginate_as: "General",
            return_key: "Scene Heading",
            shortcut: "9",
        },
        ElementSetting {
            type_name: "End of Act",
            style: "Underline+AllCaps".to_string(),
            alignment: alignment_name(layout_profile.styles.end_of_act.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.end_of_act.left_indent),
            right_indent: format_indent(layout_profile.styles.end_of_act.right_indent),
            space_before: format_space_before(&layout_profile.styles.end_of_act),
            spacing: format_spacing(layout_profile.styles.end_of_act.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.end_of_act.starts_new_page,
            ),
            paginate_as: "Character",
            return_key: "New Act",
            shortcut: ":",
        },
        ElementSetting {
            type_name: "Cold Opening",
            style: "Underline+AllCaps".to_string(),
            alignment: alignment_name(layout_profile.styles.cold_opening.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.cold_opening.left_indent),
            right_indent: format_indent(layout_profile.styles.cold_opening.right_indent),
            space_before: format_space_before(&layout_profile.styles.cold_opening),
            spacing: format_spacing(layout_profile.styles.cold_opening.line_spacing),
            starts_new_page: format_starts_new_page(
                layout_profile.styles.cold_opening.starts_new_page,
            ),
            paginate_as: "General",
            return_key: "Scene Heading",
            shortcut: "",
        },
        ElementSetting {
            type_name: "Lyric",
            style: "Italic".to_string(),
            alignment: alignment_name(layout_profile.styles.lyric.alignment),
            first_indent: "0.00",
            left_indent: format_indent(layout_profile.styles.lyric.left_indent),
            right_indent: format_indent(layout_profile.styles.lyric.right_indent),
            space_before: format_space_before(&layout_profile.styles.lyric),
            spacing: format_spacing(layout_profile.styles.lyric.line_spacing),
            starts_new_page: format_starts_new_page(layout_profile.styles.lyric.starts_new_page),
            paginate_as: "Dialogue",
            return_key: "Action",
            shortcut: ";",
        },
    ];

    let font = escape_xml_attr(&font_choice(metadata));
    for setting in settings {
        write!(
            out,
            "  <ElementSettings Type=\"{}\">\n    <FontSpec AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"{}\"/>\n    <ParagraphSpec Alignment=\"{}\" FirstIndent=\"{}\" Leading=\"Regular\" LeftIndent=\"{}\" RightIndent=\"{}\" SpaceBefore=\"{}\" Spacing=\"{}\" StartsNewPage=\"{}\"/>\n    <Behavior PaginateAs=\"{}\" ReturnKey=\"{}\" Shortcut=\"{}\"/>\n  </ElementSettings>\n\n",
            escape_xml_attr(setting.type_name),
            font,
            escape_xml_attr(&setting.style),
            setting.alignment,
            setting.first_indent,
            setting.left_indent,
            setting.right_indent,
            setting.space_before,
            setting.spacing,
            setting.starts_new_page,
            escape_xml_attr(setting.paginate_as),
            escape_xml_attr(setting.return_key),
            escape_xml_attr(setting.shortcut),
        )
        .unwrap();
    }
}

fn alignment_name(alignment: Alignment) -> String {
    match alignment {
        Alignment::Left => "Left".to_string(),
        Alignment::Center => "Center".to_string(),
        Alignment::Right => "Right".to_string(),
    }
}

fn format_indent(value: f32) -> String {
    format!("{value:.2}")
}

fn format_spacing(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i32)
    } else {
        format!("{value:.2}")
    }
}

fn format_space_before(style: &ScreenplayElementStyle) -> String {
    let points = style.spacing_before * 12.0;
    if points.fract() == 0.0 {
        format!("{}", points as i32)
    } else {
        format!("{points:.2}")
    }
}

fn format_starts_new_page(starts_new_page: bool) -> String {
    if starts_new_page {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

fn render_title_page(out: &mut String, screenplay: &Screenplay) {
    let metadata = &screenplay.metadata;
    let font = escape_xml_attr(&font_choice(metadata));
    out.push_str("  <TitlePage>\n    <HeaderAndFooter FooterFirstPage=\"Yes\" FooterVisible=\"No\" HeaderFirstPage=\"No\" HeaderVisible=\"Yes\" StartingPage=\"1\">\n      <Header>\n        <Paragraph Alignment=\"Right\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.25\" RightIndent=\"-1.25\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n");
    write!(out, "          <DynamicLabel AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\" Type=\"Page #\"/>\n", font).unwrap();
    out.push_str("        </Paragraph>\n      </Header>\n      <Footer>\n        <Paragraph Alignment=\"Right\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.25\" RightIndent=\"-1.25\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n");
    write!(out, "          <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\"></Text>\n", font).unwrap();
    out.push_str("        </Paragraph>\n      </Footer>\n    </HeaderAndFooter>\n    <Content>\n");

    // FDX is verbose XML, but the output shape is intentionally boring here:
    // emit the fixed blank layout first, then write the semantic groups that
    // users usually edit on a title page. This keeps future changes local.
    for _ in 0..18 {
        render_title_blank_paragraph(out, &font, "Left");
    }
    render_title_title_paragraph(out, metadata, &font);
    for _ in 0..3 {
        render_title_blank_paragraph(out, &font, "Center");
    }
    render_title_credit_paragraph(out, metadata, &font);
    render_title_blank_paragraph(out, &font, "Center");
    render_title_author_paragraph(out, metadata, &font);
    for _ in 0..4 {
        render_title_blank_paragraph(out, &font, "Center");
    }
    render_title_source_paragraph(out, metadata, &font);
    for _ in 0..7 {
        render_title_blank_paragraph(out, &font, "Center");
    }
    for _ in 0..10 {
        render_title_blank_paragraph(out, &font, "Full");
    }
    render_title_bottom_rows(out, metadata, &font);
    render_title_blank_paragraph(out, &font, "Left");
    render_title_frontmatter(out, screenplay, &font);

    out.push_str("    </Content>\n    <TextState Scaling=\"90\" Selection=\"233,233\" ShowInvisibles=\"No\"/>\n  </TitlePage>\n");
}

fn render_title_blank_paragraph(out: &mut String, font: &str, alignment: &str) {
    start_title_paragraph(out, alignment);
    push_title_text(out, font, "0", "", "");
    end_title_paragraph(out);
}

fn render_title_frontmatter(out: &mut String, screenplay: &Screenplay, font: &str) {
    if let Some(imported_title_page) = &screenplay.imported_title_page {
        for page in imported_title_page.pages.iter().skip(1) {
            for (para_index, para) in page.paragraphs.iter().enumerate() {
                start_imported_title_paragraph(out, para, para_index == 0);
                push_title_element_text(out, font, "0", "", &para.text);
                end_imported_title_paragraph(out, para);
            }
        }
    }
}

fn start_imported_title_paragraph(
    out: &mut String,
    paragraph: &crate::ImportedTitlePageParagraph,
    starts_new_page: bool,
) {
    let alignment = match paragraph.alignment {
        ImportedTitlePageAlignment::Left => "Left",
        ImportedTitlePageAlignment::Center => "Center",
        ImportedTitlePageAlignment::Right => "Right",
        ImportedTitlePageAlignment::Full => "Full",
    };
    let left_indent = paragraph.left_indent.unwrap_or(match paragraph.alignment {
        ImportedTitlePageAlignment::Center
        | ImportedTitlePageAlignment::Right
        | ImportedTitlePageAlignment::Full => 1.00,
        ImportedTitlePageAlignment::Left => 1.50,
    });
    let space_before = paragraph.space_before.unwrap_or(12.0);
    let starts_new_page = if starts_new_page { "Yes" } else { "No" };
    write!(
        out,
        "      <Paragraph Alignment=\"{}\" FirstIndent=\"{:.2}\" Leading=\"Regular\" LeftIndent=\"{:.2}\" RightIndent=\"7.50\" SpaceBefore=\"{:.0}\" Spacing=\"1\" StartsNewPage=\"{}\">\n",
        alignment,
        paragraph.first_indent.unwrap_or(0.0),
        left_indent,
        space_before,
        starts_new_page
    )
    .unwrap();
}

fn end_imported_title_paragraph(
    out: &mut String,
    paragraph: &crate::ImportedTitlePageParagraph,
) {
    if !paragraph.tab_stops.is_empty() {
        out.push_str("        <Tabstops>\n");
        for tab_stop in &paragraph.tab_stops {
            let kind = match tab_stop.kind {
                ImportedTitlePageTabStopKind::Left => "Left",
                ImportedTitlePageTabStopKind::Center => "Center",
                ImportedTitlePageTabStopKind::Right => "Right",
            };
            writeln!(
                out,
                "          <Tabstop Position=\"{:.2}\" Type=\"{}\"/>",
                tab_stop.position, kind
            )
            .unwrap();
        }
        out.push_str("        </Tabstops>\n");
    }
    out.push_str("      </Paragraph>\n");
}

fn render_title_title_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    let plain_style = if plain_title_uses_all_caps(metadata) {
        "Bold+Underline+AllCaps"
    } else {
        "Bold+Underline"
    };
    let mut saw_any = false;
    for value in metadata.get("title").into_iter().flatten() {
        saw_any = true;
        start_title_paragraph(out, "Center");
        push_title_element_text(out, font, "0", plain_style, value);
        end_title_paragraph(out);
    }
    if !saw_any {
        render_title_blank_paragraph(out, font, "Center");
    }
}

fn title_page_has_author(metadata: &Metadata) -> bool {
    ["author", "authors"]
        .into_iter()
        .any(|key| metadata.get(key).is_some_and(|values| !values.is_empty()))
}

fn render_title_credit_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Center");
    let credit = if metadata.contains_key("credit") {
        join_metadata(metadata, "credit", " ")
    } else if title_page_has_author(metadata) {
        "by".to_string()
    } else {
        String::new()
    };
    push_title_text(out, font, "0", "", &credit);
    end_title_paragraph(out);
}

fn render_title_author_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    let mut saw_any = false;
    for key in ["author", "authors"] {
        for value in metadata.get(key).into_iter().flatten() {
            saw_any = true;
            start_title_paragraph(out, "Center");
            push_title_element_text(out, font, "0", "", value);
            end_title_paragraph(out);
        }
    }
    if !saw_any {
        render_title_blank_paragraph(out, font, "Center");
    }
}

fn render_title_source_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    for value in metadata.get("source").into_iter().flatten() {
        start_title_paragraph(out, "Center");
        push_title_text(out, font, "-1", "", &value.plain_text());
        end_title_paragraph(out);
    }
}

fn render_title_bottom_rows(out: &mut String, metadata: &Metadata, font: &str) {
    let contact_lines = metadata
        .get("contact")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.plain_text())
        .collect::<Vec<_>>();
    let mut right_rows = Vec::new();
    let draft = join_metadata(metadata, "draft", "");
    if !draft.is_empty() {
        right_rows.push(draft);
    }
    let draft_date = metadata_first(metadata, "draft date");
    if !draft_date.is_empty() {
        right_rows.push(draft_date);
    }

    let row_count = contact_lines.len().max(right_rows.len());
    let left_start_index = row_count.saturating_sub(contact_lines.len());
    let right_start_index = row_count.saturating_sub(right_rows.len());
    for index in 0..row_count {
        start_title_paragraph(out, "Left");
        if index >= left_start_index {
            let contact_index = index - left_start_index;
            if let Some(contact) = contact_lines.get(contact_index) {
                push_title_text(out, font, "0", "", contact);
            }
        }
        let mut tabstop_positions = Vec::new();
        if index >= right_start_index {
            let right_index = index - right_start_index;
            if let Some(right_text) = right_rows.get(right_index) {
                push_title_text(out, font, "0", "", "\t");
                push_title_text(out, font, "0", "", right_text);
                tabstop_positions.push(title_page_bottom_right_tabstop(right_text));
            }
        }
        end_title_bottom_paragraph(out, &tabstop_positions);
    }
}

fn title_page_bottom_right_tabstop(text: &str) -> f32 {
    let right_edge_points = 7.5 * 72.0;
    // Final Draft title-page tab stops need a small safety margin beyond the
    // nominal cell grid or longer bottom-right rows wrap earlier than expected.
    let text_width_points = (text.chars().count() as f32 * 7.1) + 2.0;
    (right_edge_points - text_width_points) / 72.0
}

fn start_title_paragraph(out: &mut String, alignment: &str) {
    write!(
        out,
        "      <Paragraph Alignment=\"{}\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.00\" RightIndent=\"7.50\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n",
        alignment
    )
    .unwrap();
}

fn end_title_bottom_paragraph(out: &mut String, tabstop_positions: &[f32]) {
    if !tabstop_positions.is_empty() {
        out.push_str("        <Tabstops>\n");
        for position in tabstop_positions {
            writeln!(
                out,
                "          <Tabstop Position=\"{position:.2}\" Type=\"Left\"/>"
            )
            .unwrap();
        }
        out.push_str("        </Tabstops>\n");
    }
    out.push_str("      </Paragraph>\n");
}

fn push_title_text(out: &mut String, font: &str, adornment_style: &str, style: &str, text: &str) {
    write!(
        out,
        "        <Text AdornmentStyle=\"{}\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"{}\">{}</Text>\n",
        adornment_style,
        font,
        style,
        escape_xml_text(text)
    )
    .unwrap();
}

fn push_title_element_text(
    out: &mut String,
    font: &str,
    adornment_style: &str,
    plain_style: &str,
    text: &ElementText,
) {
    match text {
        ElementText::Plain(text) => push_title_text(out, font, adornment_style, plain_style, text),
        ElementText::Styled(runs) => {
            for run in runs {
                let styles = sorted_style_names(run, true).join("+");
                push_title_text(out, font, adornment_style, &styles, &run.content);
            }
        }
    }
}

fn end_title_paragraph(out: &mut String) {
    out.push_str("      </Paragraph>\n");
}

fn metadata_first(metadata: &Metadata, key: &str) -> String {
    metadata
        .get(key)
        .and_then(|values| values.first())
        .map(|value| value.plain_text())
        .unwrap_or_default()
}

fn font_choice(metadata: &Metadata) -> String {
    metadata_value(metadata, "font-choice")
}

fn metadata_value(metadata: &Metadata, key: &str) -> String {
    metadata
        .get(key)
        .and_then(|values| values.first())
        .map(|value| value.plain_text())
        .unwrap_or_default()
}
