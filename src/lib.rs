use lazy_static::lazy_static;
use regex::Regex;
use unicode_categories::UnicodeCategories;

pub enum Element<'a> {
    Action(&'a str),
    Character(&'a str),
    SceneHeading(&'a str),
    Lyric(&'a str),
    Parenthetical(&'a str),
    Dialogue(&'a str),
    DialogueBlock(Box<[Element<'a>; 2]>),
    DualDialogueBlock(Box<[Element<'a>; 2]>),
    Transition(&'a str),
    Section(&'a str),
    Synopsis(&'a str),
    ColdOpening(&'a str),
    NewAct(&'a str),
    EndOfAct(&'a str),
}

fn remove_problematic_unicode(text: &str) -> String {
    text.replace(|c: char| c.is_other_format(), "")
}

fn remove_boneyard(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/").unwrap();
    }
    let output = RE.replace_all(&text, "");
    output.to_string()
}

pub fn parse(text: &str) -> Vec<Element> {
    vec![Element::SceneHeading("INT. HOUSE - DAY")]
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_remove_problematic_unicode() {
        let unicode_string = "Hello\u{200B}, \u{200D}\u{FEFF}World!";
        assert_eq!(remove_problematic_unicode(unicode_string), "Hello, World!");
    }

    #[test]
    fn test_remove_boneyard() {
        let boneyard = "/* boneyard */Hello, World!\n\n/* More bones \n Lower bones*/Goodbye!";
        assert_eq!(remove_boneyard(boneyard), "Hello, World!\n\nGoodbye!");
    }
}
