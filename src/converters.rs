use crate::{Element::*, Metadata, Screenplay};
#[cfg(feature = "handlebars")]
use handlebars::Handlebars;
#[cfg(feature = "html")]
use handlebars::{
    handlebars_helper, Context, Helper, Output, RenderContext, RenderError, RenderErrorReason,
};
use serde_json;
#[cfg(feature = "fdx")]
use std::collections::{HashMap, HashSet};

impl Screenplay {
    #[cfg(feature = "fdx")]
    pub fn to_final_draft(&mut self) -> String {
        let metadata = &mut self.metadata;

        add_fdx_formatting(metadata);

        // Removing elements incompatible with Final Draft format
        self.elements.retain(|e| match e {
            PageBreak | Section(_, _, _) | Synopsis(_) => false,
            _ => true,
        });

        let template = include_str!("templates/fdx.hbs");
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("fdx", template)
            .expect("Expect template to load.");
        let result = handlebars.render("fdx", self);
        match result {
            Ok(string) => string,
            Err(error) => {
                eprint!("Failed conversion: {}", error);
                "Failed conversion. See error message.".to_string()
            }
        }
    }

    #[cfg(feature = "html")]
    pub fn to_html(&mut self, head: bool) -> String {
        let template: &str = if head {
            include_str!("templates/html.hbs")
        } else {
            include_str!("templates/body.hbs")
        };
        let mut handlebars: Handlebars = Handlebars::new();
        handlebars.register_helper("type_to_class", Box::new(type_to_class_helper));
        handlebars.register_helper("style", Box::new(style_helper));
        handlebars
            .register_template_string("html", template)
            .expect("Expect template to load.");
        let result = handlebars.render("html", self);
        match result {
            Ok(string) => string,
            Err(error) => {
                eprint!("Failed conversion: {}", error);
                "Failed conversion. See error message.".to_string()
            }
        }
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

#[cfg(feature = "fdx")]
fn insert_helper(metadata: &mut HashMap<String, Vec<String>>, key: &str, value: &str) -> () {
    metadata.insert(key.to_string(), vec![value.to_owned()]);
}

#[cfg(feature = "fdx")]
fn add_fdx_formatting(metadata: &mut Metadata) -> () {
    // Set Defaults
    let mut scene_heading_styles = vec!["AllCaps"];
    let mut space_before_heading = "24".to_string();
    let mut dialogue_spacing = "1".to_string();
    let mut action_text_style = "".to_string();
    let mut font_choice = "Courier Prime".to_string();
    let mut dialogue_left_indent = "2.50".to_string();
    let mut dialogue_right_indent = "6.00".to_string();

    let fmt = metadata.get_mut("fmt");
    match fmt {
        None => (),
        Some(opts_vec) => {
            if let Some(opts_string) = opts_vec.first() {
                let lowercase = opts_string.to_lowercase();
                let options: HashSet<&str> = lowercase.split_whitespace().collect();
                for option in options {
                    match option {
                        "bsh" => scene_heading_styles.push("Bold"),
                        "ush" => scene_heading_styles.push("Underline"),
                        "acat" => action_text_style.push_str("AllCaps"),
                        "ssbsh" => space_before_heading = "12".to_string(),
                        "dsd" => dialogue_spacing = "2".to_string(),
                        "cfd" => font_choice = "Courier Final Draft".to_string(),
                        opt if opt.starts_with("dl-") => {
                            if let Some(value) = opt.strip_prefix("dl-") {
                                // validate that value is a valid number
                                if let Ok(_) = value.parse::<f64>() {
                                    dialogue_left_indent = value.to_string();
                                }
                            }
                        }
                        opt if opt.starts_with("dr-") => {
                            if let Some(value) = opt.strip_prefix("dr-") {
                                // validate that value is a valid number
                                if let Ok(_) = value.parse::<f64>() {
                                    dialogue_right_indent = value.to_string();
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    scene_heading_styles.sort_unstable();
    let scene_heading_style: String = scene_heading_styles.join("+");
    insert_helper(metadata, "scene-heading-style", &scene_heading_style);
    insert_helper(metadata, "space-before-heading", &space_before_heading);
    insert_helper(metadata, "dialogue-spacing", &dialogue_spacing);
    insert_helper(metadata, "action-text-style", &action_text_style);
    insert_helper(metadata, "font-choice", &font_choice);
    insert_helper(metadata, "dialogue-left-indent", &dialogue_left_indent);
    insert_helper(metadata, "dialogue-right-indent", &dialogue_right_indent);
}

#[cfg(feature = "html")]
fn type_to_class_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let param = h
        .param(0)
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("format", 0))?;
    let input = param.render();
    let output = match input.as_str() {
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
        _ => input.as_str(),
    };
    out.write(output)?;
    Ok(())
}

#[cfg(feature = "html")]
handlebars_helper!(style_helper: |s: str| s.to_lowercase());

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;
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
            insert_helper(&mut expected, pair.0, pair.1);
        }

        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should produce the correct defaults");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "bsh ush");
        insert_helper(
            &mut expected,
            "scene-heading-style",
            "AllCaps+Bold+Underline",
        );
        insert_helper(&mut expected, "fmt", "bsh ush");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle scene-heading-style");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "acat");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "action-text-style", "AllCaps");
        insert_helper(&mut expected, "fmt", "acat");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle action-text-style");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dsd");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "dialogue-spacing", "2");
        insert_helper(&mut expected, "fmt", "dsd");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-spacing");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "cfd");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "font-choice", "Courier Final Draft");
        insert_helper(&mut expected, "fmt", "cfd");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle font-choice");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dl-1.25");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "dialogue-left-indent", "1.25");
        insert_helper(&mut expected, "fmt", "dl-1.25");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-left-indent");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dr-4.75");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "dialogue-right-indent", "4.75");
        insert_helper(&mut expected, "fmt", "dr-4.75");
        add_fdx_formatting(&mut metadata);
        assert_eq!(metadata, expected, "it should handle dialogue-right-indent");

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dl-3.0 dr-5.5");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "dialogue-left-indent", "3.0");
        insert_helper(&mut expected, "dialogue-right-indent", "5.5");
        insert_helper(&mut expected, "fmt", "dl-3.0 dr-5.5");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should handle both dialogue indents together"
        );

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dl-2 dr-8 bsh acat");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "dialogue-left-indent", "2");
        insert_helper(&mut expected, "dialogue-right-indent", "8");
        insert_helper(&mut expected, "scene-heading-style", "AllCaps+Bold");
        insert_helper(&mut expected, "action-text-style", "AllCaps");
        insert_helper(&mut expected, "fmt", "dl-2 dr-8 bsh acat");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should handle dialogue indents combined with other options"
        );

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dl-invalid");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "fmt", "dl-invalid");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should ignore invalid dialogue-left-indent values"
        );

        metadata = HashMap::new();
        insert_helper(&mut metadata, "fmt", "dr-notanumber");
        for pair in defaults.iter() {
            insert_helper(&mut expected, pair.0, pair.1);
        }
        insert_helper(&mut expected, "fmt", "dr-notanumber");
        add_fdx_formatting(&mut metadata);
        assert_eq!(
            metadata, expected,
            "it should ignore invalid dialogue-right-indent values"
        );
    }
}
