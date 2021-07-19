use crate::{Element::*, Metadata, Screenplay};
use handlebars::Handlebars;
use std::collections::{HashMap, HashSet};

impl Screenplay {
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
}

fn insert_helper(metadata: &mut HashMap<String, Vec<String>>, key: &str, value: &str) -> () {
    metadata.insert(key.to_string(), vec![value.to_owned()]);
}

fn add_fdx_formatting(metadata: &mut Metadata) -> () {
    // Set Defaults
    let mut scene_heading_styles = vec!["AllCaps"];
    let mut space_before_heading = "24".to_string();
    let mut dialogue_spacing = "1".to_string();
    let mut action_text_style = "".to_string();
    let mut font_choice = "Courier Prime".to_string();

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
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;

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
    }
}
