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
    let newlines_removed: String = lines.filter(|l| !l.trim().is_empty()).collect();
    println!("{}", newlines_removed);
    vec![Element::SceneHeading("INT. HOUSE - DAY")]
}

fn lines_to_hunks(lines: Lines) -> Vec<Vec<&str>> {
    let initial: Vec<Vec<&str>> = vec![vec![]];
    let output: Vec<Vec<&str>> = lines.fold(initial, |mut acc, l| match l.trim() {
        "" => {
            if l.len() == 2 {
                acc.last_mut()
                    .expect("There should always be at least one vec")
                    .push(l);
            } else {
                acc.push(vec![]);
            }
            acc
        }
        l => {
            acc.last_mut()
                .expect("There should always be at least on vec")
                .push(l);
            acc
        }
    });
    output
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_lines_to_hunks() {
        let lines = "hello hello hello\n\nwelcome back\ngoodbye".lines();
        let expected = vec![vec!["hello hello hello"], vec!["welcome back", "goodbye"]];
        assert_eq!(lines_to_hunks(lines), expected);
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
