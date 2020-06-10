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

}
