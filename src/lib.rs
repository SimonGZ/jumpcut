use lazy_static::lazy_static;
use regex::Regex;
use std::str::Lines;

const SCENE_LOCATORS: [&str; 16] = [
    "INT ",
    "INT.",
    "EXT ",
    "EXT.",
    "EST.",
    "EST ",
    "INT./EXT.",
    "INT./EXT ",
    "INT/EXT.",
    "INT/EXT ",
    "I/E.",
    "I/E ",
    "EXT./INT.",
    "EXT./INT ",
    "EXT/INT.",
    "EXT/INT ",
];

#[derive(Debug, PartialEq)]
pub enum Element {
    Action(String),
    Character(String),
    SceneHeading(String),
    Lyric(String),
    Parenthetical(String),
    Dialogue(String),
    DialogueBlock(Box<[Element; 2]>),
    DualDialogueBlock(Box<[Element; 2]>),
    Transition(String),
    Section(String),
    Synopsis(String),
    ColdOpening(String),
    NewAct(String),
    EndOfAct(String),
}

pub fn parse(text: &str) -> Vec<Element> {
    let fountain_string = prepare_text(text);
    let lines = fountain_string.lines();
    let hunks: Vec<Vec<&str>> = lines_to_hunks(lines);
    println!("{:#?}", hunks);
    let elements: Vec<Element> = hunks_to_elements(hunks);
    // println!("{:#?}", elements);
    elements
}

/// Strips out problematic unicode and the boneyard element
fn prepare_text(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/|\p{gc:Cf}").unwrap();
    }
    RE.replace_all(text, "").to_string()
}

fn lines_to_hunks(lines: Lines) -> Vec<Vec<&str>> {
    let mut hunks = lines.fold(vec![vec![]], |mut acc, l: &str| match l.trim() {
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
    });
    // Handle special case of an empty string
    if hunks.len() == 1
        && hunks
            .first()
            .expect("There will always be at least one vec.")
            .is_empty()
    {
        hunks.first_mut().unwrap().push("");
    };
    hunks
}

fn hunks_to_elements<'a>(hunks: Vec<Vec<&'a str>>) -> Vec<Element> {
    hunks
        .into_iter()
        .map(|h| {
            let element = hunk_to_elements(h);
            element
        })
        .collect()
}

fn hunk_to_elements<'a>(hunk: Vec<&'a str>) -> Element {
    if hunk.len() == 1 {
        let line: String = hunk[0].to_string();
        match make_forced(&line) {
            Some(make_element) => {
                let stripped = line.trim_start_matches(&['!', '@', '~', '.', '>', '#', '='][..]);
                make_element(stripped.to_string())
            }
            _ => {
                if is_scene(&line) {
                    Element::SceneHeading(line)
                } else {
                    Element::Action(line)
                }
            }
        }
    } else {
        let top_line: String = hunk[0].to_string();
        let element_text = hunk.join("\n");
        match make_forced(&top_line) {
            Some(make_element) => {
                let stripped =
                    element_text.trim_start_matches(&['!', '@', '~', '.', '>', '#', '='][..]);
                make_element(stripped.to_string())
            }
            _ => Element::Action(element_text),
        }
    }
}

fn is_scene(line: &str) -> bool {
    let line = line.to_uppercase();
    SCENE_LOCATORS.iter().any(|&s| line.starts_with(s))
}

fn is_forced(line: &str) -> bool {
    match line.get(..1) {
        Some("!") | Some("@") | Some("~") | Some(".") | Some(">") | Some("#") | Some("=") => true,
        _ => false,
    }
}

fn make_forced(line: &str) -> Option<fn(String) -> Element> {
    match line.get(..1) {
        Some("!") => Some(Element::Action),
        Some("@") => Some(Element::Character),
        Some("~") => Some(Element::Lyric),
        Some(".") => Some(Element::SceneHeading),
        Some(">") => Some(Element::Transition),
        Some("#") => Some(Element::Section),
        Some("=") => Some(Element::Section),
        _ => None,
    }
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

        lines = "".lines();
        expected = vec![vec![""]];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should handle an empty string"
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
        assert_eq!(prepare_text(unicode_string), "Hello, World!");
    }

    #[test]
    fn test_remove_boneyard() {
        let boneyard = "/* boneyard */Hello, World!\n\n/* More bones \n Lower bones*/Goodbye!";
        assert_eq!(prepare_text(boneyard), "Hello, World!\n\nGoodbye!");
    }
}
