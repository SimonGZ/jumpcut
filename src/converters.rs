use crate::Screenplay;
use handlebars::Handlebars;

impl Screenplay {
    pub fn to_final_draft(&self) -> String {
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
