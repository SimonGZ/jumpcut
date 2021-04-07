use crate::{Element::*, Screenplay};
use handlebars::Handlebars;
use std::collections::{HashMap, HashSet};

impl Screenplay {
    pub fn to_final_draft(&mut self) -> String {
        let metadata = &mut self.metadata;
        // Set Defaults
        let mut scene_heading_style = "AllCaps".to_string();
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
                            "bsh" => scene_heading_style.push_str("+Bold"),
                            "ush" => scene_heading_style.push_str("+Underline"),
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

        insert_helper(metadata, "scene-heading-style", scene_heading_style);
        insert_helper(metadata, "space-before-heading", space_before_heading);
        insert_helper(metadata, "dialogue-spacing", dialogue_spacing);
        insert_helper(metadata, "action-text-style", action_text_style);
        insert_helper(metadata, "font-choice", font_choice);

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

fn insert_helper(metadata: &mut HashMap<String, Vec<String>>, key: &str, value: String) -> () {
    metadata.insert(key.to_string(), vec![value]);
}
