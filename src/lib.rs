use lazy_static::lazy_static;
use regex::Regex;
use std::str::Lines;
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
    let mut fountain_string = remove_problematic_unicode(text);
    fountain_string = remove_boneyard(&fountain_string);
    let lines = fountain_string.lines();
    let _hunks = lines_to_hunks(lines);
    // println!("{:#?}", hunks);
    vec![Element::SceneHeading("INT. HOUSE - DAY")]
}

fn lines_to_hunks(lines: Lines) -> Vec<Vec<&str>> {
    let initial: Vec<Vec<&str>> = vec![vec![]];
    lines.fold(initial, |mut acc, l: &str| match l.trim() {
        // HANDLE BLANK LINES
        "" => {
            // If there are exactly two spaces in the line, it's intentional
            if l.len() == 2 {
                acc.last_mut().unwrap().push(l);
            // If the previous element was also blank, create an empty string
            } else if acc.last().unwrap().is_empty() {
                acc.last_mut().unwrap().push("");
            // Otherwise, start a new element by pushing a new empty vec
            } else {
                acc.push(vec![]);
            }
            acc
        }
        /* HANDLE SECTIONS
         * They don't follow the simple rules of blank line before or after.
         * So we need this special case to handle them.
         */
        l if l.starts_with('#') => {
            // If the previous hunk was empty, use it.
            if acc.last().unwrap().is_empty() {
                acc.last_mut().unwrap().push(l);
            // If previous hunk wasn't empty, create a new one.
            } else {
                acc.push(vec![l]);
            }
            // Sections are isolated, so start a new empty hunk for next element.
            acc.push(vec![]);
            acc
        }
        // HANDLE NORMAL, NON-EMPTY LINES
        l => {
            acc.last_mut().unwrap().push(l);
            acc
        }
    })
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_lines_to_hunks() {
        let mut lines = "hello hello hello\n\nwelcome back\ngoodbye".lines();
        let mut expected = vec![vec!["hello hello hello"], vec!["welcome back", "goodbye"]];
        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should handle simple line spacing"
        );

        lines = "# Act 1\nINT. HOUSE\n\nAn ugly place.".lines();
        expected = vec![vec!["# Act 1"], vec!["INT. HOUSE"], vec!["An ugly place."]];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should put sections in their own vec"
        );

        lines = "SALLY\nYou're screwed!\n\n# Act 1\nINT. HOUSE\n\nAn ugly place.".lines();
        expected = vec![
            vec!["SALLY", "You're screwed!"],
            vec!["# Act 1"],
            vec!["INT. HOUSE"],
            vec!["An ugly place."],
        ];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should handle sections in middle of content"
        );

        lines = "John examines the gun.\n\n\n\n\n\n\n\n\n\nBANG!".lines();
        expected = vec![
            vec!["John examines the gun."],
            vec![""],
            vec![""],
            vec![""],
            vec![""],
            vec!["BANG!"],
        ];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should create blank lines from multiple newlines in a row"
        );
    }

    #[test]
    fn test_lines_to_hunks_intentional_blanks() {
        let lines = "hello hello hello\n\nwelcome back\n  \ngoodbye".lines();
        let expected = vec![
            vec!["hello hello hello"],
            vec!["welcome back", "  ", "goodbye"],
        ];
        assert_eq!(lines_to_hunks(lines), expected);
    }

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
