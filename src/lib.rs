use lazy_static::lazy_static;
use regex::Regex;
use std::default::Default;
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
    Action(String, Attributes),
    Character(String, Attributes),
    SceneHeading(String, Attributes),
    Lyric(String, Attributes),
    Parenthetical(String, Attributes),
    Dialogue(String, Attributes),
    DialogueBlock(Box<[Element; 2]>),
    DualDialogueBlock(Box<[Element; 2]>),
    Transition(String, Attributes),
    Section(String, Attributes),
    Synopsis(String, Attributes),
    ColdOpening(String, Attributes),
    NewAct(String, Attributes),
    EndOfAct(String, Attributes),
}

#[derive(Debug, PartialEq)]
pub struct Attributes {
    pub centered: bool,
    pub starts_new_page: bool,
    pub scene_number: Option<String>,
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            centered: false,
            starts_new_page: false,
            scene_number: None,
        }
    }
}

fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}

pub fn parse(text: &str) -> Vec<Element> {
    let fountain_string = prepare_text(text);
    let lines = fountain_string.lines();
    let hunks: Vec<Vec<&str>> = lines_to_hunks(lines);
    // println!("{:#?}", hunks);
    let elements: Vec<Element> = hunks_to_elements(hunks);
    println!("{:#?}", elements);
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
    let mut hunks = lines.fold(vec![vec![]], |mut acc, line: &str| match line.trim() {
        // HANDLE BLANK LINES
        "" => {
            // If there are exactly two spaces in the line, it's intentional
            if line.len() == 2 {
                acc.last_mut().unwrap().push(line);
            // If the previous element was blank but it was the first element, do nothing
            } else if acc.last().unwrap().is_empty() && acc.len() == 1 {
                () // do nothing
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
                acc.last_mut().unwrap().push(line);
            // If previous hunk wasn't empty, create a new one.
            } else {
                acc.push(vec![line]);
            }
            // Sections are isolated, so start a new empty hunk for next element.
            acc.push(vec![]);
            acc
        }
        // HANDLE NORMAL, NON-EMPTY LINES
        _ => {
            // If previous part of hunk was blank, just replace it.
            // This usually only occurs if blank lines are placed at the start
            // of the document.
            acc.last_mut().unwrap().push(line);
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
        make_single_line_element(hunk[0])
    } else {
        make_multi_line_element(hunk)
    }
}

fn make_single_line_element(line: &str) -> Element {
    lazy_static! {
        static ref SCENE_NUMBER_REGEX: Regex = Regex::new(r"\s+#(.*)#").unwrap();
    }
    match make_forced(&line) {
        Some(make_element) => {
            let stripped: &str = line
                .trim_start_matches(&['!', '@', '~', '.', '>', '#', '='][..])
                .trim_start();
            if make_element == Element::SceneHeading && SCENE_NUMBER_REGEX.is_match(stripped) {
                // Handle special case of scene numbers on scene headings
                match SCENE_NUMBER_REGEX.find(stripped) {
                    None => make_element(stripped.to_string(), blank_attributes()),
                    Some(mat) => {
                        let attributes = Attributes {
                            scene_number: Some(
                                stripped
                                    .get(mat.start()..mat.end())
                                    .unwrap()
                                    .trim_matches(&[' ', '#'][..])
                                    .to_string(),
                            ),
                            ..Attributes::default()
                        };
                        let text_without_scene_number =
                            SCENE_NUMBER_REGEX.replace(stripped, "").into_owned();
                        make_element(text_without_scene_number, attributes)
                    }
                }
            } else {
                make_element(stripped.to_string(), blank_attributes())
            }
        }
        _ if is_scene(&line) => {
            // Handle special case of scene numbers on scene headings
            if SCENE_NUMBER_REGEX.is_match(line) {
                match SCENE_NUMBER_REGEX.find(line) {
                    None => Element::SceneHeading(line.to_string(), blank_attributes()),
                    Some(mat) => {
                        let attributes = Attributes {
                            scene_number: Some(
                                line.get(mat.start()..mat.end())
                                    .unwrap()
                                    .trim_matches(&[' ', '#'][..])
                                    .to_string(),
                            ),
                            ..Attributes::default()
                        };
                        let text_without_scene_number =
                            SCENE_NUMBER_REGEX.replace(line, "").into_owned();
                        Element::SceneHeading(text_without_scene_number, attributes)
                    }
                }
            } else {
                Element::SceneHeading(line.to_string(), blank_attributes())
            }
        }
        _ if is_transition(&line) => Element::Transition(line.to_string(), blank_attributes()),
        _ if is_centered(&line) => Element::Action(
            line.to_string(),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        _ => Element::Action(line.to_string(), blank_attributes()),
    }
}

fn make_multi_line_element(hunk: Vec<&str>) -> Element {
    let top_line: String = hunk[0].to_string();
    match make_forced(&top_line) {
        Some(make_element) => {
            let stripped = hunk
                .into_iter()
                .map(|l| l.trim_start_matches(&['!', '@', '~', '.', '>', '#', '='][..]))
                .collect::<Vec<&str>>()
                .join("\n");
            make_element(stripped, blank_attributes())
        }
        // Check if the text is centered
        _ if hunk.iter().any(|&line| is_centered(line)) => {
            let cleaned_text = hunk
                .into_iter()
                .map(|l| l.trim_matches(&['>', '<'][..]))
                .collect::<Vec<&str>>()
                .join("\n");
            Element::Action(
                cleaned_text,
                Attributes {
                    centered: true,
                    ..Attributes::default()
                },
            )
        }
        _ => Element::Action(hunk.join("\n"), blank_attributes()),
    }
}

fn is_scene(line: &str) -> bool {
    let line = line.to_uppercase();
    SCENE_LOCATORS.iter().any(|&s| line.starts_with(s))
}

fn is_transition(line: &str) -> bool {
    let line = line.trim().to_uppercase();
    line.ends_with("TO:")
}

fn is_centered(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('>') && trimmed.ends_with('<')
}

fn make_forced(line: &str) -> Option<fn(String, Attributes) -> Element> {
    match line.get(..1) {
        Some("!") => Some(Element::Action),
        Some("@") => Some(Element::Character),
        Some("~") => Some(Element::Lyric),
        Some(".") => {
            // check for starting ellipsis
            if line.starts_with("..") {
                None
            } else {
                Some(Element::SceneHeading)
            }
        }
        Some(">") => {
            // check for centered text
            if line.ends_with('<') {
                None
            } else {
                Some(Element::Transition)
            }
        }
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
