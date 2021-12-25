use lazy_static::lazy_static;
use regex::Regex;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::default::Default;
use std::str::Lines;
use Element::PageBreak;

mod converters;
mod text_style_parser;

use ElementText::*;

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

pub type Metadata = HashMap<String, Vec<String>>;

#[derive(Debug, PartialEq, Serialize)]
pub struct Screenplay {
    pub metadata: Metadata,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Action(ElementText, Attributes),
    Character(ElementText, Attributes),
    SceneHeading(ElementText, Attributes),
    Lyric(ElementText, Attributes),
    Parenthetical(ElementText, Attributes),
    Dialogue(ElementText, Attributes),
    DialogueBlock(Vec<Element>),
    DualDialogueBlock(Vec<Element>),
    Transition(ElementText, Attributes),
    Section(ElementText, Attributes, u8),
    Synopsis(ElementText),
    ColdOpening(ElementText, Attributes),
    NewAct(ElementText, Attributes),
    EndOfAct(ElementText, Attributes),
    PageBreak,
}

impl Element {
    fn name(&self) -> &str {
        use Element::*;
        match *self {
            Action(_, _) => "Action",
            Character(_, _) => "Character",
            SceneHeading(_, _) => "Scene Heading",
            Lyric(_, _) => "Lyric",
            Parenthetical(_, _) => "Parenthetical",
            Dialogue(_, _) => "Dialogue",
            DialogueBlock(_) => "Dialogue Block",
            DualDialogueBlock(_) => "Dual Dialogue Block",
            Transition(_, _) => "Transition",
            Section(_, _, _) => "Section",
            Synopsis(_) => "Synopsis",
            ColdOpening(_, _) => "Cold Opening",
            NewAct(_, _) => "New Act",
            EndOfAct(_, _) => "End of Act",
            PageBreak => "Page Break",
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct SerializeElementHelper<'a> {
    #[serde(rename = "type")]
    element_type: &'a str,
    text: &'a ElementText,
    attributes: &'a Attributes,
}

impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Element::Action(ref text, ref attributes)
            | Element::Character(ref text, ref attributes)
            | Element::SceneHeading(ref text, ref attributes)
            | Element::Lyric(ref text, ref attributes)
            | Element::Parenthetical(ref text, ref attributes)
            | Element::Dialogue(ref text, ref attributes)
            | Element::Transition(ref text, ref attributes)
            | Element::ColdOpening(ref text, ref attributes)
            | Element::NewAct(ref text, ref attributes)
            | Element::EndOfAct(ref text, ref attributes) => {
                let el = SerializeElementHelper {
                    element_type: self.name(),
                    text,
                    attributes,
                };
                el.serialize(serializer)
            }
            Element::DialogueBlock(ref block) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "DialogueBlock")?;
                map.serialize_entry("block", block)?;
                map.end()
            }
            Element::DualDialogueBlock(ref blocks) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "DualDialogueBlock")?;
                map.serialize_entry("blocks", blocks)?;
                map.end()
            }
            Element::Section(ref text, ref attributes, ref level) => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("type", "Section")?;
                map.serialize_entry("text", text)?;
                map.serialize_entry("attributes", attributes)?;
                map.serialize_entry("level", level)?;
                map.end()
            }
            Element::Synopsis(ref text) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "Synopsis")?;
                map.serialize_entry("text", text)?;
                map.end()
            }
            PageBreak => serializer.serialize_none(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Attributes {
    pub centered: bool,
    pub starts_new_page: bool,
    pub scene_number: Option<String>,
    pub notes: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ElementText {
    Plain(String),
    Styled(Vec<TextRun>),
}

// Convenience function
pub fn p(p: &str) -> ElementText {
    ElementText::Plain(p.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TextRun {
    pub content: String,
    #[serde(serialize_with = "text_style_serialize")]
    pub text_style: HashSet<String>,
}

// Convenience function
pub fn tr(t: &str, s: Vec<&str>) -> TextRun {
    let mut styles: HashSet<String> = HashSet::new();
    for str in s {
        styles.insert(str.to_string());
    }
    TextRun {
        content: t.to_string(),
        text_style: styles,
    }
}

fn text_style_serialize<S>(x: &HashSet<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut styles: Vec<String> = x.clone().into_iter().collect();
    styles.sort();
    styles.serialize(s)
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            centered: false,
            starts_new_page: false,
            scene_number: None,
            notes: None,
        }
    }
}

pub fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}

pub fn parse(text: &str) -> Screenplay {
    let fountain_string = prepare_text(text);
    let lines = fountain_string.lines();
    let hunks: Vec<Vec<&str>> = lines_to_hunks(lines);
    // println!("{:#?}", hunks);
    let mut elements: Vec<Element> = hunks_to_elements(hunks);
    // println!("{:#?}", elements);
    let mut metadata: Metadata = HashMap::new();
    match elements.first() {
        Some(Element::Action(Plain(txt), _)) if has_key_value(txt) => {
            process_metadata(&mut metadata, txt);
            elements.remove(0);
        }
        _ => (),
    }
    for element in elements.iter_mut() {
        element.parse_and_convert_markup();
    }
    Screenplay { elements, metadata }
}

fn has_key_value(txt: &str) -> bool {
    lazy_static! {
        static ref KEY: Regex = Regex::new(r"(?P<key>^[^\s][^:\n\r]+):(?P<value>.*)").unwrap();
    }
    KEY.is_match(txt)
}

fn process_metadata(metadata: &mut Metadata, text: &str) {
    lazy_static! {
        static ref KEY: Regex = Regex::new(r"(?P<key>^[^\s][^:\n\r]+):(?P<value>.*)").unwrap();
    }
    let lines = text.lines();
    let mut current_key = "".to_string();
    for line in lines {
        if KEY.is_match(line) {
            // Means we have a new key
            // NOTE: I can safely unwrap these values because of the above is_match
            let caps = KEY.captures(line).unwrap();
            let key = caps.name("key").unwrap().as_str();
            current_key = key.to_lowercase().to_string();
            let current_value = caps.name("value").unwrap().as_str();
            if current_value.is_empty() {
                metadata.insert(current_key.to_string(), vec![]);
            } else {
                metadata.insert(
                    current_key.to_string(),
                    vec![current_value.trim().to_string()],
                );
            }
        } else {
            // Means we have a line without a key and thus an additional value to push
            if let Some(values) = metadata.get_mut(&current_key) {
                values.push(line.trim().to_string());
            }
        }
    }
}

/// Strips out problematic unicode and the boneyard element
fn prepare_text(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/|\p{gc:Cf}").unwrap();
    }
    RE.replace_all(text.trim_end(), "").to_string()
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
                // do nothing
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
            acc
        }
        // HANDLE NORMAL, NON-EMPTY LINES
        _ => {
            let last_line = acc.last().unwrap().first();
            // If previous hunk was a section, create a new hunk
            match last_line {
                Some(l) if l.starts_with('#') => acc.push(vec![]),
                _ => (),
            }
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

fn hunks_to_elements(hunks: Vec<Vec<&str>>) -> Vec<Element> {
    let initial: Vec<Element> = Vec::with_capacity(hunks.len());
    let mut elements = hunks
        .into_iter()
        .rev()
        .fold(initial, |mut acc, hunk: Vec<&str>| {
            if hunk.len() == 1 {
                let element = make_single_line_element(hunk[0]);
                if element == PageBreak {
                    // If the single line element was a PageBreak, we need to
                    // mark the next element as startsNewPage = true
                    let last_element = acc.last_mut();
                    match last_element {
                        Some(Element::Action(_, attributes))
                        | Some(Element::Character(_, attributes))
                        | Some(Element::SceneHeading(_, attributes))
                        | Some(Element::Lyric(_, attributes))
                        | Some(Element::Parenthetical(_, attributes))
                        | Some(Element::Dialogue(_, attributes))
                        | Some(Element::Transition(_, attributes))
                        | Some(Element::ColdOpening(_, attributes))
                        | Some(Element::NewAct(_, attributes))
                        | Some(Element::EndOfAct(_, attributes)) => {
                            attributes.starts_new_page = true
                        }
                        Some(_) | None => (),
                    }
                } else {
                    acc.push(element);
                }
            } else {
                let element = make_multi_line_element(hunk);
                match (acc.last_mut(), &element) {
                    // If the previous element was a dual dialogue block and it only contains one block
                    // then put this element into that block so long as it's a dialogue element
                    (Some(Element::DualDialogueBlock(dialogues)), Element::DialogueBlock(_))
                        if dialogues.len() == 1 =>
                    {
                        dialogues.insert(0, element);
                    }
                    (Some(Element::Section(_, attr, _)), Element::Synopsis(Plain(note))) => {
                        attr.notes = Some(vec![note.to_string()]);
                    }
                    _ => acc.push(element),
                }
            }
            acc
        });
    elements.reverse();
    elements
}

fn make_single_line_element(line: &str) -> Element {
    lazy_static! {
        static ref SCENE_NUMBER_REGEX: Regex = Regex::new(r"\s+#(.*)#").unwrap();
        static ref NOTE_REGEX: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    }
    let mut attributes = blank_attributes();
    if has_note(line) {
        let notes = retrieve_notes(line);
        attributes = Attributes {
            notes,
            ..attributes
        };
    }
    match make_forced(&line) {
        Some(make_element) => {
            let stripped: &str = line
                .trim_start_matches(&['!', '@', '~', '.', '>', '='][..])
                .trim_start();

            if line.get(..1) == Some(".") && SCENE_NUMBER_REGEX.is_match(stripped) {
                // Handle special case of scene numbers on scene headings
                match SCENE_NUMBER_REGEX.find(stripped) {
                    None => make_element(Plain(stripped.to_string()), attributes),
                    Some(mat) => {
                        attributes = Attributes {
                            scene_number: Some(
                                stripped
                                    .get(mat.start()..mat.end())
                                    .unwrap()
                                    .trim_matches(&[' ', '#'][..])
                                    .to_string(),
                            ),
                            ..attributes
                        };
                        let text_without_scene_number = SCENE_NUMBER_REGEX.replace(stripped, "");
                        let final_text = remove_notes(&text_without_scene_number);
                        make_element(Plain(final_text), attributes)
                    }
                }
            } else {
                let final_text = remove_notes(stripped);
                make_element(Plain(final_text), attributes)
            }
        }
        _ if is_scene(&line) => {
            // Handle special case of scene numbers on scene headings
            if SCENE_NUMBER_REGEX.is_match(line) {
                match SCENE_NUMBER_REGEX.find(line) {
                    None => Element::SceneHeading(Plain(line.to_string()), attributes),
                    Some(mat) => {
                        attributes = Attributes {
                            scene_number: Some(
                                line.get(mat.start()..mat.end())
                                    .unwrap()
                                    .trim_matches(&[' ', '#'][..])
                                    .to_string(),
                            ),
                            ..attributes
                        };
                        let text_without_scene_number = SCENE_NUMBER_REGEX.replace(line, "");
                        let final_text = remove_notes(&text_without_scene_number);
                        Element::SceneHeading(Plain(final_text), attributes)
                    }
                }
            } else {
                let final_text = remove_notes(line);
                Element::SceneHeading(Plain(final_text), attributes)
            }
        }
        _ if is_transition(&line) => {
            let final_text = remove_notes(line);
            Element::Transition(Plain(final_text), attributes)
        }
        _ if is_centered(&line) => {
            let final_text = remove_notes(trim_centered_marks(line));
            if is_end_act(&final_text) {
                // Check end_act first because new act regex also matches end act regex
                Element::EndOfAct(
                    Plain(final_text),
                    Attributes {
                        centered: true,
                        ..attributes
                    },
                )
            } else if is_new_act(&final_text) {
                Element::NewAct(
                    Plain(final_text),
                    Attributes {
                        centered: true,
                        ..attributes
                    },
                )
            } else {
                Element::Action(
                    Plain(final_text),
                    Attributes {
                        centered: true,
                        ..attributes
                    },
                )
            }
        }
        _ => {
            let final_text = remove_notes(line);
            Element::Action(Plain(final_text), attributes)
        }
    }
}

fn make_multi_line_element(hunk: Vec<&str>) -> Element {
    let mut attributes = blank_attributes();
    let temp_joined = hunk.join("\n");
    if has_note(&temp_joined) {
        let notes = retrieve_notes(&temp_joined);
        attributes = Attributes {
            notes,
            ..attributes
        };
    }
    let top_line: String = hunk[0].to_string();
    match make_forced(&top_line) {
        Some(make_element) => {
            // Check if it's a forced character because that means dialogueblock
            if top_line.trim().get(..1) == Some("@") {
                let stripped_hunk = hunk
                    .into_iter()
                    .map(|l| l.trim_start_matches('@'))
                    .collect::<Vec<&str>>();
                make_dialogue_block(stripped_hunk)
            } else {
                // It's not forced character, so we can create a string with newlines
                let stripped_string = hunk
                    .into_iter()
                    .map(|l| l.trim_start_matches(&['!', '@', '~', '.', '>', '='][..]))
                    .collect::<Vec<&str>>()
                    .join("\n");
                let final_text = remove_notes(&stripped_string);
                make_element(Plain(final_text), attributes)
            }
        }
        // Check if the text is centered
        _ if hunk.iter().any(|&line| is_centered(line)) => {
            let cleaned_text = hunk
                .into_iter()
                .map(trim_centered_marks)
                .collect::<Vec<&str>>()
                .join("\n");
            let final_text = remove_notes(&cleaned_text);
            Element::Action(
                Plain(final_text),
                Attributes {
                    centered: true,
                    ..attributes
                },
            )
        }
        _ if is_character(hunk[0]) => make_dialogue_block(hunk),
        _ => {
            let final_text = remove_notes(&hunk.join("\n"));
            Element::Action(Plain(final_text), attributes)
        }
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

fn is_end_act(line: &str) -> bool {
    lazy_static! {
        static ref END_ACT_REGEX: Regex = Regex::new(
            r"(end (of )*(act ((\d)|one|two|three|four|five|six|seven|eight|nine|ten)|cold open|teaser))"
        )
        .unwrap();
    }
    let line = line.to_lowercase();
    END_ACT_REGEX.is_match(&line)
}

fn is_new_act(line: &str) -> bool {
    lazy_static! {
        static ref NEW_ACT_REGEX: Regex = Regex::new(
            r"(act ((\d)|one|two|three|four|five|six|seven|eight|nine|ten)|cold open|teaser)"
        )
        .unwrap();
    }
    let line = line.to_lowercase();
    NEW_ACT_REGEX.is_match(&line)
}

fn remove_notes(line: &str) -> String {
    lazy_static! {
        static ref NOTE_REGEX: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    }
    NOTE_REGEX.replace_all(line, "").to_string()
}

fn is_centered(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('>') && trimmed.ends_with('<')
}

fn trim_centered_marks(line: &str) -> &str {
    line.trim_matches(&['>', '<'][..]).trim()
}

fn is_character(line: &str) -> bool {
    line == line.to_uppercase()
}

fn is_parenthetical(line: &str) -> bool {
    line.trim().starts_with('(') && line.trim().ends_with(')')
}

fn is_lyric(line: &str) -> bool {
    line.trim().starts_with('~')
}

fn is_dual_dialogue(line: &str) -> bool {
    line.trim().ends_with('^')
}

fn has_note(line: &str) -> bool {
    line.contains("[[")
}

fn make_forced(line: &str) -> Option<fn(ElementText, Attributes) -> Element> {
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
        Some("#") => Some(make_section),
        Some("=") => {
            if line.trim().starts_with("===") {
                Some(make_page_break)
            } else {
                Some(make_synopsis)
            }
        }
        // This could also be page-break ("==="),
        // so we have to run a check in hunks_to_elements
        _ => None,
    }
}

fn make_section(line: ElementText, _: Attributes) -> Element {
    match line {
        Plain(txt) => {
            let trimmed = txt.trim().trim_start_matches('#');
            let level: u8 = (txt.len() - trimmed.len()).try_into().unwrap();
            Element::Section(Plain(trimmed.trim().to_string()), blank_attributes(), level)
        }
        _ => panic!("Shouldn't be receiving Styled text here."),
    }
}

fn make_page_break(_line: ElementText, _: Attributes) -> Element {
    PageBreak
}

fn make_synopsis(line: ElementText, _: Attributes) -> Element {
    match line {
        Plain(line) => {
            let trimmed = line.trim().trim_start_matches('=').trim();
            Element::Synopsis(Plain(trimmed.to_string()))
        }
        _ => panic!("Shouldn't be receiving Styled text here."),
    }
}

fn make_dialogue_block(hunk: Vec<&str>) -> Element {
    let mut elements = Vec::with_capacity(hunk.len());
    let raw_name: &str = hunk[0];
    let clean_name: &str = raw_name.trim().trim_end_matches('^').trim();
    let character: Element = Element::Character(Plain(clean_name.to_string()), blank_attributes());
    elements.push(character);
    for line in hunk[1..].iter() {
        let mut processed_line = line.to_string();
        let mut attributes = blank_attributes();
        if has_note(line) {
            let notes = retrieve_notes(line);
            attributes = Attributes {
                notes,
                ..attributes
            };
            processed_line = remove_notes(line);
        }
        if is_parenthetical(&processed_line) {
            elements.push(Element::Parenthetical(Plain(processed_line), attributes));
        } else if is_lyric(&processed_line) {
            let stripped_line = processed_line.trim().trim_start_matches('~').trim();
            if let Element::Lyric(Plain(s), _) = elements.last_mut().unwrap() {
                // if previous element was lyric and so is this one, add this line to that previous lyric
                s.push_str("\n");
                s.push_str(&stripped_line);
            } else {
                // this line is lyric but previous line wasn't, create new lyric element
                elements.push(Element::Lyric(Plain(stripped_line.to_string()), attributes));
            }
        } else if let Element::Dialogue(Plain(s), _) = elements.last_mut().unwrap() {
            // if previous element was dialogue, add this line to that dialogue
            s.push_str("\n");
            s.push_str(&processed_line);
        } else {
            // otherwise this is a new dialogue
            elements.push(Element::Dialogue(Plain(processed_line), attributes));
        }
    }
    if is_dual_dialogue(raw_name) {
        let mut blocks = Vec::with_capacity(2);
        blocks.push(Element::DialogueBlock(elements));
        Element::DualDialogueBlock(blocks)
    } else {
        Element::DialogueBlock(elements)
    }
}

fn retrieve_notes(line: &str) -> Option<Vec<String>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    }
    let mut result = vec![];
    for mat in RE.find_iter(line) {
        match line.get(mat.start() + 2..mat.end() - 2) {
            Some(str) => result.push(str.to_string()),
            None => (),
        }
    }
    Some(result)
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

        lines = "# Act 1\n## John finds the horse\n\nJOHN\nWhoa!".lines();
        expected = vec![
            vec!["# Act 1"],
            vec!["## John finds the horse"],
            vec!["JOHN", "Whoa!"],
        ];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should handle two newlines after a section"
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
