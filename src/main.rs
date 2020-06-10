use lazy_static::lazy_static;
use regex::Regex;
use unicode_categories::UnicodeCategories;

enum Element {
    Action,
    Character,
    SceneHeading,
    Lyric,
    Parenthetical,
    Dialogue,
    DialogueBlock,
    DualDialogueBlock,
    Transition,
    Section,
    Synopsis,
    ColdOpening,
    NewAct,
    EndOfAct,
}

fn remove_problematic_unicode(text: String) -> String {
    let output = text.chars().filter(|x| !x.is_other_format()).collect();
    output
}

fn remove_boneyard(text: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/").unwrap();
    }
    let output = RE.replace_all(&text, "");
    output.to_string()
}

fn main() {
    println!(
        "{}",
        remove_problematic_unicode("Hello\u{200B}, world!".to_string())
    );
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_remove_problematic_unicode() {
        let unicode_string = "Hello\u{200B}, \u{200D}\u{FEFF}World!".to_string();
        assert_eq!(remove_problematic_unicode(unicode_string), "Hello, World!");
    }

    #[test]
    fn test_remove_boneyard() {
        let boneyard =
            "/* boneyard */Hello, World!\n\n/* More bones \n Lower bones*/Goodbye!".to_string();
        assert_eq!(remove_boneyard(boneyard), "Hello, World!\n\nGoodbye!");
    }
}
