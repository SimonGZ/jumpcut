use super::shared::{
    escape_xml_attr, escape_xml_text, join_metadata, sorted_style_names,
};
use crate::{Element, ElementText, Metadata, Screenplay};
use std::collections::HashMap;
use std::fmt::Write;

pub(crate) fn prepare_screenplay(screenplay: &mut Screenplay) {
    add_fdx_formatting(&mut screenplay.metadata);

    screenplay.elements.retain(|e| match e {
        Element::PageBreak | Element::Section(_, _, _) | Element::Synopsis(_) => false,
        _ => true,
    });
}

pub(crate) fn render_document(screenplay: &Screenplay) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>\n<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"4\">\n    <Content>\n");
    render_content(&mut out, screenplay);
    out.push_str("    </Content>\n\n");
    render_header_and_footer(&mut out, &screenplay.metadata);
    out.push('\n');
    render_element_settings(&mut out, &screenplay.metadata);
    out.push('\n');
    render_title_page(&mut out, &screenplay.metadata);
    out.push_str("\n  <MoresAndContinueds>\n");
    write!(
        out,
        "    <FontSpec AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"\"/>\n",
        escape_xml_attr(font_choice(&screenplay.metadata))
    )
    .unwrap();
    out.push_str("    <DialogueBreaks AutomaticCharacterContinueds=\"Yes\" BottomOfPage=\"Yes\" DialogueBottom=\"(MORE)\" DialogueTop=\"(CONT'D)\" TopOfNext=\"Yes\"/>\n    <SceneBreaks ContinuedNumber=\"No\" SceneBottom=\"(CONTINUED)\" SceneBottomOfPage=\"No\" SceneTop=\"CONTINUED:\" SceneTopOfNext=\"No\"/>\n  </MoresAndContinueds>\n\n</FinalDraft>\n");
    out
}

pub(crate) fn add_fdx_formatting(metadata: &mut Metadata) {
    let mut scene_heading_styles = vec!["AllCaps"];
    let mut space_before_heading = "24".to_string();
    let mut dialogue_spacing = "1".to_string();
    let mut action_text_style = "".to_string();
    let mut font_choice = "Courier Prime".to_string();
    let mut dialogue_left_indent = "2.50".to_string();
    let mut dialogue_right_indent = "6.00".to_string();

    if let Some(opts_vec) = metadata.get_mut("fmt") {
        if let Some(opts_string) = opts_vec.first() {
            for option in opts_string.split_whitespace() {
                if option.eq_ignore_ascii_case("bsh") {
                    scene_heading_styles.push("Bold");
                } else if option.eq_ignore_ascii_case("ush") {
                    scene_heading_styles.push("Underline");
                } else if option.eq_ignore_ascii_case("acat") {
                    action_text_style.push_str("AllCaps");
                } else if option.eq_ignore_ascii_case("ssbsh") {
                    space_before_heading = "12".to_string();
                } else if option.eq_ignore_ascii_case("dsd") {
                    dialogue_spacing = "2".to_string();
                } else if option.eq_ignore_ascii_case("cfd") {
                    font_choice = "Courier Final Draft".to_string();
                } else if let Some(value) = option.strip_prefix("dl-") {
                    if value.parse::<f64>().is_ok() {
                        dialogue_left_indent = value.to_string();
                    }
                } else if let Some(value) = option.strip_prefix("dr-") {
                    if value.parse::<f64>().is_ok() {
                        dialogue_right_indent = value.to_string();
                    }
                }
            }
        }
    }

    scene_heading_styles.sort_unstable();
    let scene_heading_style: String = scene_heading_styles.join("+");
    insert_metadata_value(metadata, "scene-heading-style", &scene_heading_style);
    insert_metadata_value(metadata, "space-before-heading", &space_before_heading);
    insert_metadata_value(metadata, "dialogue-spacing", &dialogue_spacing);
    insert_metadata_value(metadata, "action-text-style", &action_text_style);
    insert_metadata_value(metadata, "font-choice", &font_choice);
    insert_metadata_value(metadata, "dialogue-left-indent", &dialogue_left_indent);
    insert_metadata_value(metadata, "dialogue-right-indent", &dialogue_right_indent);
}

pub(crate) fn insert_metadata_value(
    metadata: &mut HashMap<String, Vec<String>>,
    key: &str,
    value: &str,
) {
    metadata.insert(key.to_string(), vec![value.to_owned()]);
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

    write!(out, "      <Paragraph Type=\"{}\"", escape_xml_attr(type_name)).unwrap();
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
    let font = escape_xml_attr(font_choice(metadata));
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

fn render_element_settings(out: &mut String, metadata: &Metadata) {
    struct ElementSetting<'a> {
        type_name: &'a str,
        style: &'a str,
        alignment: &'a str,
        first_indent: &'a str,
        left_indent: &'a str,
        right_indent: &'a str,
        space_before: &'a str,
        spacing: &'a str,
        starts_new_page: &'a str,
        paginate_as: &'a str,
        return_key: &'a str,
        shortcut: &'a str,
    }

    let settings = [
        ElementSetting { type_name: "General", style: "", alignment: "Left", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "0", spacing: "1", starts_new_page: "No", paginate_as: "General", return_key: "General", shortcut: "0" },
        ElementSetting { type_name: "Scene Heading", style: metadata_value(metadata, "scene-heading-style"), alignment: "Left", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: metadata_value(metadata, "space-before-heading"), spacing: "1", starts_new_page: "No", paginate_as: "Scene Heading", return_key: "Action", shortcut: "1" },
        ElementSetting { type_name: "Action", style: metadata_value(metadata, "action-text-style"), alignment: "Left", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "12", spacing: "1", starts_new_page: "No", paginate_as: "Action", return_key: "Action", shortcut: "2" },
        ElementSetting { type_name: "Character", style: "AllCaps", alignment: "Left", first_indent: "0.00", left_indent: "3.50", right_indent: "7.25", space_before: "12", spacing: "1", starts_new_page: "No", paginate_as: "Character", return_key: "Dialogue", shortcut: "3" },
        ElementSetting { type_name: "Parenthetical", style: "", alignment: "Left", first_indent: "-0.10", left_indent: "3.00", right_indent: "5.50", space_before: "0", spacing: "1", starts_new_page: "No", paginate_as: "Parenthetical", return_key: "Dialogue", shortcut: "4" },
        ElementSetting { type_name: "Dialogue", style: "", alignment: "Left", first_indent: "0.00", left_indent: metadata_value(metadata, "dialogue-left-indent"), right_indent: metadata_value(metadata, "dialogue-right-indent"), space_before: "0", spacing: metadata_value(metadata, "dialogue-spacing"), starts_new_page: "No", paginate_as: "Dialogue", return_key: "Action", shortcut: "5" },
        ElementSetting { type_name: "Transition", style: "AllCaps", alignment: "Right", first_indent: "0.00", left_indent: "5.50", right_indent: "7.10", space_before: "12", spacing: "1", starts_new_page: "No", paginate_as: "Transition", return_key: "Scene Heading", shortcut: "6" },
        ElementSetting { type_name: "Shot", style: "AllCaps", alignment: "Left", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "12", spacing: "1", starts_new_page: "No", paginate_as: "Scene Heading", return_key: "Action", shortcut: "7" },
        ElementSetting { type_name: "Cast List", style: "AllCaps", alignment: "Left", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "0", spacing: "1", starts_new_page: "No", paginate_as: "Action", return_key: "Action", shortcut: "8" },
        ElementSetting { type_name: "New Act", style: "Underline+AllCaps", alignment: "Center", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "0", spacing: "1", starts_new_page: "Yes", paginate_as: "General", return_key: "Scene Heading", shortcut: "9" },
        ElementSetting { type_name: "End of Act", style: "Underline+AllCaps", alignment: "Center", first_indent: "0.00", left_indent: "1.50", right_indent: "7.50", space_before: "24", spacing: "1", starts_new_page: "No", paginate_as: "Character", return_key: "New Act", shortcut: ":" },
        ElementSetting { type_name: "Cold Opening", style: "Underline+AllCaps", alignment: "Center", first_indent: "0.00", left_indent: "1.00", right_indent: "7.50", space_before: "12", spacing: "1", starts_new_page: "No", paginate_as: "General", return_key: "Scene Heading", shortcut: "" },
        ElementSetting { type_name: "Lyric", style: "Italic", alignment: "Left", first_indent: "0.00", left_indent: "2.50", right_indent: "7.38", space_before: "0", spacing: "1", starts_new_page: "No", paginate_as: "Dialogue", return_key: "Action", shortcut: ";" },
    ];

    let font = escape_xml_attr(font_choice(metadata));
    for setting in settings {
        write!(
            out,
            "  <ElementSettings Type=\"{}\">\n    <FontSpec AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"{}\" RevisionID=\"0\" Size=\"12\" Style=\"{}\"/>\n    <ParagraphSpec Alignment=\"{}\" FirstIndent=\"{}\" Leading=\"Regular\" LeftIndent=\"{}\" RightIndent=\"{}\" SpaceBefore=\"{}\" Spacing=\"{}\" StartsNewPage=\"{}\"/>\n    <Behavior PaginateAs=\"{}\" ReturnKey=\"{}\" Shortcut=\"{}\"/>\n  </ElementSettings>\n\n",
            escape_xml_attr(setting.type_name),
            font,
            escape_xml_attr(setting.style),
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

fn render_title_page(out: &mut String, metadata: &Metadata) {
    let font = escape_xml_attr(font_choice(metadata));
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
    render_title_draft_paragraph(out, metadata, &font);
    render_title_draft_date_paragraph(out, metadata, &font);
    render_title_blank_paragraph(out, &font, "Left");

    out.push_str("    </Content>\n    <TextState Scaling=\"90\" Selection=\"233,233\" ShowInvisibles=\"No\"/>\n  </TitlePage>\n");
}

fn render_title_blank_paragraph(out: &mut String, font: &str, alignment: &str) {
    start_title_paragraph(out, alignment);
    push_title_text(out, font, "0", "", "");
    end_title_paragraph(out);
}

fn render_title_title_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Center");
    for value in metadata.get("title").into_iter().flatten() {
        push_title_text(out, font, "0", "Bold+Underline+AllCaps", value);
    }
    end_title_paragraph(out);
}

fn render_title_credit_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Center");
    let credit = if let Some(values) = metadata.get("credit") {
        let mut credit = String::new();
        for value in values {
            credit.push_str(value);
            credit.push(' ');
        }
        credit
    } else {
        "by".to_string()
    };
    push_title_text(out, font, "0", "", &credit);
    end_title_paragraph(out);
}

fn render_title_author_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Center");
    for key in ["author", "authors"] {
        for value in metadata.get(key).into_iter().flatten() {
            push_title_text(out, font, "0", "", value);
        }
    }
    end_title_paragraph(out);
}

fn render_title_source_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Center");
    for value in metadata.get("source").into_iter().flatten() {
        push_title_text(out, font, "-1", "", value);
    }
    end_title_paragraph(out);
}

fn render_title_draft_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Right");
    push_title_text(out, font, "0", "", &join_metadata(metadata, "draft", ""));
    out.push_str("        <Tabstops>\n          <Tabstop Position=\"6.32\" Type=\"Left\"/>\n          <Tabstop Position=\"6.00\" Type=\"Left\"/>\n        </Tabstops>\n      </Paragraph>\n");
}

fn render_title_draft_date_paragraph(out: &mut String, metadata: &Metadata, font: &str) {
    start_title_paragraph(out, "Right");
    push_title_text(out, font, "0", "", &metadata_first(metadata, "draft date"));
    out.push_str("        <Tabstops>\n          <Tabstop Position=\"6.00\" Type=\"Left\"/>\n        </Tabstops>\n      </Paragraph>\n");
}

fn start_title_paragraph(out: &mut String, alignment: &str) {
    write!(
        out,
        "      <Paragraph Alignment=\"{}\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.00\" RightIndent=\"7.50\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n",
        alignment
    )
    .unwrap();
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

fn end_title_paragraph(out: &mut String) {
    out.push_str("      </Paragraph>\n");
}

fn metadata_first(metadata: &Metadata, key: &str) -> String {
    metadata
        .get(key)
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or_default()
}

fn font_choice(metadata: &Metadata) -> &str {
    metadata_value(metadata, "font-choice")
}

fn metadata_value<'a>(metadata: &'a Metadata, key: &str) -> &'a str {
    metadata
        .get(key)
        .and_then(|values| values.first())
        .map(String::as_str)
        .unwrap_or("")
}
