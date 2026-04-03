use crate::Screenplay;
use serde_json;

impl Screenplay {
    #[cfg(feature = "fdx")]
    pub fn to_final_draft(&mut self) -> String {
        crate::rendering::fdx::prepare_screenplay(self);
        crate::rendering::fdx::render_document(self)
    }

    #[cfg(feature = "html")]
    pub fn to_html(&mut self, head: bool) -> String {
        crate::rendering::html::render_document(
            self,
            crate::html_output::HtmlRenderOptions {
                head,
                ..Default::default()
            },
        )
    }

    #[cfg(feature = "html")]
    pub fn to_html_with_options(&mut self, options: crate::html_output::HtmlRenderOptions) -> String {
        crate::rendering::html::render_document(self, options)
    }

    pub fn to_text(&self, options: &crate::text_output::TextRenderOptions) -> String {
        crate::text_output::render(self, options)
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
        screenplay
            .metadata
            .retain(|key, _| key == "fmt");
        let actual = normalize_markup_without_style(&screenplay.to_html(true));
        let expected = normalize_markup_without_style(&render_html_with_handlebars(&screenplay, true));
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
        screenplay.metadata.insert("fmt".into(), vec!["multicam dr-5.75".into()]);

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

    fn sample_screenplay() -> Screenplay {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["TITLE".into()]);
        metadata.insert("credit".into(), vec!["BY".into()]);
        metadata.insert("author".into(), vec!["AUTHOR".into()]);
        metadata.insert("source".into(), vec!["SOURCE".into()]);
        metadata.insert("draft".into(), vec!["DRAFT".into()]);
        metadata.insert("draft date".into(), vec!["DATE".into()]);
        metadata.insert("fmt".into(), vec!["bsh".into()]);

        let styled =
            ElementText::Styled(vec![tr("BOLD", vec!["Bold"]), tr("ITALIC", vec!["Italic"])]);
        let mut scene_attrs = blank_attributes();
        scene_attrs.scene_number = Some("1".into());
        let mut centered_attrs = blank_attributes();
        centered_attrs.centered = true;

        Screenplay {
            metadata,
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
            })
        {
            Some(true) => "screenplay multicam",
            _ => "screenplay",
        };
        let template = if head {
            include_str!("templates/html.hbs")
        } else {
            include_str!("templates/body.hbs")
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
                        Some(value) if value == "Cold Opening" => "coldOpening",
                        Some(value) if value == "New Act" => "newAct",
                        Some(value) if value == "End of Act" => "endOfAct",
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
