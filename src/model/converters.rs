use crate::pagination::ScreenplayLayoutProfile;
use crate::{Element, Screenplay};
use serde_json;

impl Screenplay {
    pub fn apply_structural_act_break_policy(&mut self) {
        let mut saw_prior_opener = false;
        let profile = ScreenplayLayoutProfile::from_screenplay(self);
        let auto_new_act_page_breaks = profile.styles.new_act.starts_new_page;

        for element in &mut self.elements {
            match element {
                Element::ColdOpening(_, _) => {
                    saw_prior_opener = true;
                }
                Element::NewAct(_, attributes) => {
                    if auto_new_act_page_breaks && saw_prior_opener && !attributes.starts_new_page {
                        attributes.starts_new_page = true;
                    }
                    saw_prior_opener = true;
                }
                _ => {}
            }
        }
    }

    pub fn to_fountain(&self) -> String {
        crate::rendering::fountain::render(self)
    }

    #[cfg(feature = "fdx")]
    pub fn to_final_draft(&mut self) -> String {
        crate::rendering::fdx::prepare_screenplay(self);
        crate::rendering::fdx::render_document(self)
    }

    #[cfg(feature = "html")]
    pub fn to_html(&mut self, head: bool) -> String {
        crate::rendering::html::render_document(
            self,
            crate::rendering::html::HtmlRenderOptions {
                head,
                ..Default::default()
            },
        )
    }

    #[cfg(feature = "html")]
    pub fn to_html_with_options(
        &mut self,
        options: crate::rendering::html::HtmlRenderOptions,
    ) -> String {
        crate::rendering::html::render_document(self, options)
    }

    pub fn to_text(&self, options: &crate::rendering::text::TextRenderOptions) -> String {
        crate::rendering::text::render(self, options)
    }

    #[cfg(feature = "pdf")]
    pub fn to_pdf(&self) -> Vec<u8> {
        crate::rendering::pdf::render(self)
    }

    #[cfg(feature = "pdf")]
    pub fn to_pdf_with_options(&self, options: crate::rendering::pdf::PdfRenderOptions) -> Vec<u8> {
        crate::rendering::pdf::render_with_options(self, options)
    }

    pub fn to_json_string(self) -> String {
        serde_json::to_string(&self)
            .expect("Should be impossible for this JSON serialization to fail.")
    }

    pub fn to_json_value(self) -> serde_json::Value {
        serde_json::to_value(&self)
            .expect("Should be impossible for this JSON serialization to fail.")
    }
}

// * Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::fdx::{add_fdx_formatting, insert_metadata_value};
    use crate::{blank_attributes, p, tr, Element, ElementText, Metadata};
    use handlebars::{Handlebars, JsonRender};
    use pretty_assertions::assert_eq;
    use serde_json::{json, Map, Value};
    use std::collections::HashMap;

    #[test]
    fn test_add_fdx_formatting() {
        let mut metadata: Metadata = HashMap::new();
        let mut expected: Metadata = HashMap::new();
        let defaults: Vec<(&str, &str)> = vec![
            ("scene-heading-style", "AllCaps"),
            ("space-before-heading", "24"),
            ("dialogue-spacing", "1"),
            ("action-text-style", ""),
            ("font-choice", "Courier Prime"),
            ("dialogue-left-indent", "2.50"),
            ("dialogue-right-indent", "6.00"),
        ];

        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }

        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should produce the correct defaults");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "bsh ush");
        insert_metadata_value(
            &mut expected,
            "scene-heading-style",
            "AllCaps+Bold+Underline",
        );
        insert_metadata_value(&mut expected, "fmt", "bsh ush");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle scene-heading-style");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "acat");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "action-text-style", "AllCaps");
        insert_metadata_value(&mut expected, "fmt", "acat");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle action-text-style");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dsd");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "dialogue-spacing", "2");
        insert_metadata_value(&mut expected, "fmt", "dsd");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-spacing");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "cfd");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "font-choice", "Courier Final Draft");
        insert_metadata_value(&mut expected, "fmt", "cfd");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle font-choice");

        metadata = HashMap::new();
        insert_metadata_value(
            &mut metadata,
            "fmt",
            "bold-scene-headings underline-scene-headings all-caps-action single-space-before-scene-headings courier-final-draft",
        );
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(
            &mut expected,
            "scene-heading-style",
            "AllCaps+Bold+Underline",
        );
        insert_metadata_value(&mut expected, "space-before-heading", "12");
        insert_metadata_value(&mut expected, "action-text-style", "AllCaps");
        insert_metadata_value(&mut expected, "font-choice", "Courier Final Draft");
        insert_metadata_value(
            &mut expected,
            "fmt",
            "bold-scene-headings underline-scene-headings all-caps-action single-space-before-scene-headings courier-final-draft",
        );
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should accept long-form fmt aliases");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dl-1.25");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "dialogue-left-indent", "1.25");
        insert_metadata_value(&mut expected, "fmt", "dl-1.25");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-left-indent");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dr-4.75");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "dialogue-right-indent", "4.75");
        insert_metadata_value(&mut expected, "fmt", "dr-4.75");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-right-indent");

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dl-3.0 dr-5.5");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "dialogue-left-indent", "3.00");
        insert_metadata_value(&mut expected, "dialogue-right-indent", "5.50");
        insert_metadata_value(&mut expected, "fmt", "dl-3.0 dr-5.5");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should handle both dialogue indents together"
        );

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dl-2 dr-8 bsh acat");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "dialogue-left-indent", "2.00");
        insert_metadata_value(&mut expected, "dialogue-right-indent", "8.00");
        insert_metadata_value(&mut expected, "scene-heading-style", "AllCaps+Bold");
        insert_metadata_value(&mut expected, "action-text-style", "AllCaps");
        insert_metadata_value(&mut expected, "fmt", "dl-2 dr-8 bsh acat");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should handle dialogue indents combined with other options"
        );

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dl-invalid");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "fmt", "dl-invalid");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should ignore invalid dialogue-left-indent values"
        );

        metadata = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "dr-notanumber");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "fmt", "dr-notanumber");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should ignore invalid dialogue-right-indent values"
        );

        metadata = HashMap::new();
        expected = HashMap::new();
        insert_metadata_value(&mut metadata, "fmt", "multicam dr-5.75");
        for pair in defaults.iter() {
            insert_metadata_value(&mut expected, pair.0, pair.1);
        }
        insert_metadata_value(&mut expected, "style-profile", "multicam");
        insert_metadata_value(&mut expected, "dialogue-spacing", "2");
        insert_metadata_value(&mut expected, "dialogue-left-indent", "2.25");
        insert_metadata_value(&mut expected, "dialogue-right-indent", "5.75");
        insert_metadata_value(&mut expected, "character-right-indent", "6.25");
        insert_metadata_value(&mut expected, "parenthetical-left-indent", "2.75");
        insert_metadata_value(&mut expected, "transition-right-indent", "7.25");
        insert_metadata_value(&mut expected, "fmt", "multicam dr-5.75");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should apply multicam defaults first and then let explicit fmt knobs override them"
        );
    }

    #[test]
    fn test_html_renderer_matches_template_output() {
        let mut screenplay = sample_screenplay();
        screenplay.metadata.retain(|key, _| key == "fmt");
        let actual = normalize_markup_without_style(&screenplay.to_html(true));
        let expected =
            normalize_markup_without_style(&render_html_with_handlebars(&screenplay, true));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fdx_renderer_matches_template_output() {
        let mut screenplay = sample_screenplay();
        let actual = screenplay.to_final_draft();
        assert!(
            actual.contains("<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"4\">")
        );
        assert!(actual.contains("<Paragraph Type=\"Scene Heading\" Number=\"1\">"));
        assert!(actual.contains("<Text Style=\"Bold\">BOLD</Text>"));
        assert!(actual.contains("<DualDialogue>"));
        assert!(actual.contains("<ElementSettings Type=\"Dialogue\">"));
        assert!(actual
            .contains("LeftIndent=\"2.50\" RightIndent=\"6.00\" SpaceBefore=\"0\" Spacing=\"1\""));
        assert!(actual.contains("<Text AdornmentStyle=\"-1\""));
        assert!(actual.contains("Font=\"Courier Prime\""));
        assert!(actual.contains(">DRAFT</Text>"));
        assert!(actual.contains(">DATE</Text>"));
    }

    #[test]
    fn test_fdx_renderer_uses_shared_layout_profile_for_multicam_settings() {
        let mut screenplay = sample_screenplay();
        screenplay
            .metadata
            .insert("fmt".into(), vec!["multicam dr-5.75".into()]);

        let actual = screenplay.to_final_draft();

        assert!(actual.contains("<ElementSettings Type=\"Dialogue\">"));
        assert!(actual
            .contains("LeftIndent=\"2.25\" RightIndent=\"5.75\" SpaceBefore=\"0\" Spacing=\"2\""));
        assert!(actual.contains("<ElementSettings Type=\"Character\">"));
        assert!(actual.contains("RightIndent=\"6.25\" SpaceBefore=\"12\""));
        assert!(actual.contains("<ElementSettings Type=\"Parenthetical\">"));
        assert!(actual.contains("LeftIndent=\"2.75\" RightIndent=\"5.50\""));
        assert!(actual.contains("<ElementSettings Type=\"Transition\">"));
        assert!(actual.contains("Alignment=\"Right\""));
        assert!(actual.contains("LeftIndent=\"5.50\" RightIndent=\"7.25\""));
    }

    #[test]
    fn test_fdx_renderer_does_not_add_extra_space_before_lyric() {
        let mut screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            elements: vec![Element::Lyric(p("I love to sing"), blank_attributes())],
        };

        let actual = screenplay.to_final_draft();

        assert!(actual.contains("<ElementSettings Type=\"Lyric\">"));
        assert!(actual.contains(
            "<ParagraphSpec Alignment=\"Left\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"2.50\" RightIndent=\"7.38\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\"/>"
        ));
    }

    #[test]
    fn test_fdx_renderer_title_credit_omits_trailing_space() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        metadata.insert("credit".into(), vec!["written by".into()]);

        let mut screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![],
        };

        let actual = screenplay.to_final_draft();

        assert!(actual.contains(">written by</Text>"));
        assert!(!actual.contains(">written by </Text>"));
    }

    #[test]
    fn test_fdx_renderer_does_not_invent_by_without_author_or_credit() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);

        let mut screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![],
        };

        let actual = screenplay.to_final_draft();

        assert!(!actual.contains(">by</Text>"));
    }

    #[test]
    fn test_fdx_renderer_writes_each_title_source_line_as_its_own_paragraph() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        metadata.insert(
            "source".into(),
            vec!["based on the novel".into(), "by J.R.R. Smithee".into()],
        );

        let mut screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![],
        };

        let actual = screenplay.to_final_draft();

        assert!(actual.contains(
            ">based on the novel</Text>\n      </Paragraph>\n      <Paragraph Alignment=\"Center\""
        ));
        assert!(actual.contains(">by J.R.R. Smithee</Text>"));
        assert!(!actual.contains(">based on the novel</Text>\n        <Text AdornmentStyle=\"-1\""));
    }

    #[test]
    fn test_fdx_renderer_writes_each_title_line_as_its_own_paragraph_and_preserves_styles() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "title".into(),
            vec![
                ElementText::Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
                "FULL RETIRED".into(),
            ],
        );

        let mut screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![],
        };

        let actual = screenplay.to_final_draft();

        assert!(actual.contains(
            "<Paragraph Alignment=\"Center\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.00\" RightIndent=\"7.50\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n        <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"Courier Prime\" RevisionID=\"0\" Size=\"12\" Style=\"Bold+Underline\">BRICK &amp; STEEL</Text>\n      </Paragraph>"
        ));
        assert!(actual.contains(
            "<Paragraph Alignment=\"Center\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.00\" RightIndent=\"7.50\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">\n        <Text AdornmentStyle=\"0\" Background=\"#FFFFFFFFFFFF\" Color=\"#000000000000\" Font=\"Courier Prime\" RevisionID=\"0\" Size=\"12\" Style=\"Bold+Underline+AllCaps\">FULL RETIRED</Text>\n      </Paragraph>"
        ));
    }

    #[test]
    fn test_fdx_renderer_includes_contact_and_bottom_right_title_rows_with_tabstops() {
        let actual = title_page_fdx_for_bottom_rows(
            vec!["Anonymous Content"],
            vec!["Second Revised Network (Goldenrod)"],
            "April 6, 1952",
        );

        assert_title_page_bottom_rows(
            &actual,
            &[
                (&["\t", "Second Revised Network (Goldenrod)"], Some("4.12")),
                (&["Anonymous Content", "\t", "April 6, 1952"], Some("6.19")),
            ],
        );
    }

    #[test]
    fn test_fdx_title_page_bottom_rows_are_bottom_aligned() {
        let actual = title_page_fdx_for_bottom_rows(
            vec!["Cobalt Artists", "555-555-0593", "contact@cobalt.example"],
            vec!["STUDIO DRAFT"],
            "September 17, 1998",
        );

        assert_title_page_bottom_rows(
            &actual,
            &[
                (&["Cobalt Artists"], None),
                (&["555-555-0593", "\t", "STUDIO DRAFT"], Some("6.29")),
                (
                    &["contact@cobalt.example", "\t", "September 17, 1998"],
                    Some("5.70"),
                ),
            ],
        );
    }

    #[test]
    fn test_fdx_title_page_bottom_rows_cover_probe_suite() {
        let cases = [
            (
                vec!["Atlas Literary"],
                vec!["WRITERS DRAFT"],
                "04/04/2026",
                vec![
                    (vec!["\t", "WRITERS DRAFT"], Some("6.19")),
                    (vec!["Atlas Literary", "\t", "04/04/2026"], Some("6.49")),
                ],
            ),
            (
                vec!["North Fork Management", "scripts@northfork.example"],
                vec!["NETWORK REVISED"],
                "Mar. 3, 2022",
                vec![
                    (
                        vec!["North Fork Management", "\t", "NETWORK REVISED"],
                        Some("5.99"),
                    ),
                    (
                        vec!["scripts@northfork.example", "\t", "Mar. 3, 2022"],
                        Some("6.29"),
                    ),
                ],
            ),
            (
                vec!["Meridian Creative Partners", "555-555-0593"],
                vec!["SECOND REVISED NETWORK (GOLDENROD)"],
                "1/1/00",
                vec![
                    (
                        vec![
                            "Meridian Creative Partners",
                            "\t",
                            "SECOND REVISED NETWORK (GOLDENROD)",
                        ],
                        Some("4.12"),
                    ),
                    (vec!["555-555-0593", "\t", "1/1/00"], Some("6.88")),
                ],
            ),
        ];

        for (contact_lines, draft_lines, draft_date, expected_rows) in cases {
            let actual = title_page_fdx_for_bottom_rows(contact_lines, draft_lines, draft_date);
            let expected_refs = expected_rows
                .iter()
                .map(|(texts, tab)| (texts.as_slice(), *tab))
                .collect::<Vec<_>>();
            assert_title_page_bottom_rows(&actual, &expected_refs);
        }
    }

    fn title_page_fdx_for_bottom_rows(
        contact_lines: Vec<&str>,
        draft_lines: Vec<&str>,
        draft_date: &str,
    ) -> String {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        if !contact_lines.is_empty() {
            metadata.insert(
                "contact".into(),
                contact_lines.into_iter().map(Into::into).collect(),
            );
        }
        if !draft_lines.is_empty() {
            metadata.insert(
                "draft".into(),
                draft_lines.into_iter().map(Into::into).collect(),
            );
        }
        metadata.insert("draft date".into(), vec![draft_date.into()]);

        let mut screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![],
        };

        screenplay.to_final_draft()
    }

    fn assert_title_page_bottom_rows(actual: &str, expected_rows: &[(&[&str], Option<&str>)]) {
        let rows = extract_title_page_bottom_rows(actual);
        let actual_rows = rows
            .iter()
            .filter(|row| row.texts.iter().any(|text| !text.is_empty()))
            .rev()
            .take(expected_rows.len())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        assert_eq!(actual_rows.len(), expected_rows.len());
        for (row, (expected_texts, expected_tab)) in actual_rows.iter().zip(expected_rows.iter()) {
            let expected_texts = expected_texts
                .iter()
                .map(|text| text.to_string())
                .collect::<Vec<_>>();
            assert_eq!(row.texts, expected_texts);
            assert_eq!(row.tabstop.as_deref(), *expected_tab);
        }
    }

    fn extract_title_page_bottom_rows(actual: &str) -> Vec<TitlePageBottomRow> {
        const BOTTOM_PARAGRAPH_START: &str = "<Paragraph Alignment=\"Left\" FirstIndent=\"0.00\" Leading=\"Regular\" LeftIndent=\"1.00\" RightIndent=\"7.50\" SpaceBefore=\"0\" Spacing=\"1\" StartsNewPage=\"No\">";
        const TITLE_PAGE_END: &str = "  </TitlePage>";

        let title_page = actual
            .split("<TitlePage>")
            .nth(1)
            .and_then(|rest| rest.split(TITLE_PAGE_END).next())
            .expect("expected title page");
        let mut rows = Vec::new();

        for paragraph in title_page.split(BOTTOM_PARAGRAPH_START).skip(1) {
            let Some(block) = paragraph.split("</Paragraph>").next() else {
                continue;
            };
            let texts = block
                .split("<Text")
                .skip(1)
                .filter_map(|segment| segment.split_once('>').map(|(_, rest)| rest))
                .filter_map(|segment| segment.split_once("</Text>").map(|(text, _)| text))
                .map(|text| text.to_string())
                .collect::<Vec<_>>();
            let tabstop = block
                .split("<Tabstop Position=\"")
                .nth(1)
                .and_then(|segment| segment.split('"').next())
                .map(str::to_string);
            rows.push(TitlePageBottomRow { texts, tabstop });
        }

        rows
    }

    struct TitlePageBottomRow {
        texts: Vec<String>,
        tabstop: Option<String>,
    }

    fn sample_screenplay() -> Screenplay {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        metadata.insert("credit".into(), vec!["BY".into()]);
        metadata.insert("author".into(), vec!["AUTHOR".into()]);
        metadata.insert("source".into(), vec!["SOURCE".into()]);
        metadata.insert("draft".into(), vec!["DRAFT".into()]);
        metadata.insert("draft date".into(), vec!["DATE".into()]);

        let styled =
            ElementText::Styled(vec![tr("BOLD", vec!["Bold"]), tr("ITALIC", vec!["Italic"])]);
        let mut scene_attrs = blank_attributes();
        scene_attrs.scene_number = Some("1".into());
        let mut centered_attrs = blank_attributes();
        centered_attrs.centered = true;

        Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![
                Element::SceneHeading(p("INT.KITCHEN"), scene_attrs),
                Element::Action(styled, blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("ALICE"), blank_attributes()),
                    Element::Dialogue(p("HELLO"), blank_attributes()),
                ]),
                Element::DualDialogueBlock(vec![
                    Element::DialogueBlock(vec![
                        Element::Character(p("BOB"), blank_attributes()),
                        Element::Dialogue(p("ONE"), blank_attributes()),
                    ]),
                    Element::DialogueBlock(vec![
                        Element::Character(p("CAROL"), blank_attributes()),
                        Element::Dialogue(p("TWO"), blank_attributes()),
                    ]),
                ]),
                Element::ColdOpening(p("COLD"), centered_attrs),
            ],
        }
    }

    fn normalize_markup(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn normalize_markup_without_style(input: &str) -> String {
        let without_style = if let (Some(style_start), Some(style_end)) =
            (input.find("<style"), input.find("</style>"))
        {
            let mut stripped = String::new();
            stripped.push_str(&input[..style_start]);
            stripped.push_str(&input[style_end + "</style>".len()..]);
            stripped
        } else {
            input.to_string()
        };
        normalize_markup(&without_style)
    }

    fn render_html_with_handlebars(screenplay: &Screenplay, head: bool) -> String {
        let root_class = match screenplay
            .metadata
            .get("fmt")
            .and_then(|values| values.first())
            .map(|value| {
                value
                    .plain_text()
                    .split_whitespace()
                    .any(|option| option.eq_ignore_ascii_case("multicam"))
            }) {
            Some(true) => "screenplay multicam",
            _ => "screenplay",
        };
        let template = if head {
            include_str!("../templates/html.hbs")
        } else {
            include_str!("../templates/body.hbs")
        };
        let mut handlebars = Handlebars::new();
        handlebars.register_helper(
            "type_to_class",
            Box::new(
                |h: &handlebars::Helper<'_>,
                 _: &Handlebars<'_>,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext<'_, '_>,
                 out: &mut dyn handlebars::Output|
                 -> Result<(), handlebars::RenderError> {
                    let output = match h.param(0).map(|p| p.value().render()) {
                        Some(value) if value == "Scene Heading" => "sceneHeading",
                        Some(value) if value == "Action" => "action",
                        Some(value) if value == "Character" => "character",
                        Some(value) if value == "Dialogue" => "dialogue",
                        Some(value) if value == "Parenthetical" => "parenthetical",
                        Some(value) if value == "Transition" => "transition",
                        Some(value) if value == "Lyric" => "lyric",
                        Some(value) if value == "Section" => "section",
                        Some(value) if value == "Synopsis" => "synopsis",
                        Some(value) if value == "Cold Opening" => "coldOpening underline",
                        Some(value) if value == "New Act" => "newAct underline",
                        Some(value) if value == "End of Act" => "endOfAct underline",
                        _ => "unknown",
                    };
                    out.write(output)?;
                    Ok(())
                },
            ),
        );
        handlebars.register_helper(
            "style",
            Box::new(
                |h: &handlebars::Helper<'_>,
                 _: &Handlebars<'_>,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext<'_, '_>,
                 out: &mut dyn handlebars::Output|
                 -> Result<(), handlebars::RenderError> {
                    let value = h.param(0).map(|p| p.value().render()).unwrap_or_default();
                    out.write(&value.to_lowercase())?;
                    Ok(())
                },
            ),
        );
        handlebars.register_helper(
            "root_class",
            Box::new(
                move |_: &handlebars::Helper<'_>,
                      _: &Handlebars<'_>,
                      _: &handlebars::Context,
                      _: &mut handlebars::RenderContext<'_, '_>,
                      out: &mut dyn handlebars::Output|
                      -> Result<(), handlebars::RenderError> {
                    out.write(root_class)?;
                    Ok(())
                },
            ),
        );
        handlebars
            .register_template_string("html", template)
            .expect("Expect template to load.");
        handlebars
            .render("html", &legacy_html_context(screenplay))
            .unwrap()
    }

    fn legacy_html_context(screenplay: &Screenplay) -> Value {
        let metadata = screenplay
            .metadata
            .iter()
            .map(|(key, values)| {
                (
                    key.clone(),
                    Value::Array(
                        values
                            .iter()
                            .map(|value| Value::String(value.plain_text()))
                            .collect(),
                    ),
                )
            })
            .collect::<Map<String, Value>>();

        json!({
            "metadata": metadata,
            "elements": screenplay.elements,
        })
    }
}
