use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::default::Default;
use std::str::Lines;
use Element::PageBreak;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

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

fn parse_metadata_line(line: &str) -> Option<(&str, &str)> {
    let mut chars = line.chars();
    let first = chars.next()?;
    if first.is_whitespace() || matches!(first, '!' | '.' | '@' | '~' | '>') {
        return None;
    }
    let colon_index = line.find(':')?;
    if colon_index == 0 {
        return None;
    }
    let key = &line[..colon_index];
    if key.contains(['\n', '\r', ':']) {
        return None;
    }
    let value = &line[colon_index + 1..];
    Some((key, value))
}

fn has_key_value(txt: &str) -> bool {
    txt.lines()
        .find(|line| !line.trim().is_empty())
        .and_then(parse_metadata_line)
        .is_some()
}

fn process_metadata(metadata: &mut Metadata, text: &str) {
    let mut current_key = String::new();
    for line in text.lines() {
        if let Some((key, value)) = parse_metadata_line(line) {
            current_key = key.to_lowercase();
            let trimmed_value = value.trim();
            if trimmed_value.is_empty() {
                metadata.insert(current_key.clone(), vec![]);
            } else {
                metadata.insert(current_key.clone(), vec![trimmed_value.to_string()]);
            }
        } else if !current_key.is_empty() {
            if let Some(values) = metadata.get_mut(&current_key) {
                values.push(line.trim().to_string());
            }
        }
    }
}

/// Strips out problematic unicode and the boneyard element
fn is_format_control(ch: char) -> bool {
    matches!(
        ch,
        '\u{00AD}'
            | '\u{061C}'
            | '\u{200B}'
            | '\u{200C}'
            | '\u{200D}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2060}'
            | '\u{2061}'
            | '\u{2062}'
            | '\u{2063}'
            | '\u{2064}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
            | '\u{206A}'
            | '\u{206B}'
            | '\u{206C}'
            | '\u{206D}'
            | '\u{206E}'
            | '\u{206F}'
            | '\u{FEFF}'
    )
}

fn prepare_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.trim_end().chars().peekable();
    let mut in_boneyard = false;

    while let Some(ch) = chars.next() {
        if in_boneyard {
            if ch == '*' && matches!(chars.peek(), Some('/')) {
                chars.next();
                in_boneyard = false;
            }
            continue;
        }

        if ch == '/' && matches!(chars.peek(), Some('*')) {
            chars.next();
            in_boneyard = true;
            continue;
        }

        if is_format_control(ch) {
            continue;
        }

        result.push(ch);
    }

    result
}

fn locate_scene_number_span(text: &str) -> Option<(usize, usize, String)> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for idx in 0..chars.len() {
        if chars[idx].1 != '#' {
            continue;
        }
        if idx == 0 || !chars[idx - 1].1.is_whitespace() {
            continue;
        }
        let mut whitespace_start = idx - 1;
        while whitespace_start > 0 && chars[whitespace_start - 1].1.is_whitespace() {
            whitespace_start -= 1;
        }
        let mut closing_index = None;
        for candidate in idx + 1..chars.len() {
            if chars[candidate].1 == '#' {
                closing_index = Some(candidate);
            }
        }
        if let Some(last_hash) = closing_index {
            let start = chars[whitespace_start].0;
            let end = if last_hash + 1 < chars.len() {
                chars[last_hash + 1].0
            } else {
                text.len()
            };
            let number = text[chars[idx].0 + 1..chars[last_hash].0]
                .trim()
                .to_string();
            return Some((start, end, number));
        }
    }
    None
}

fn strip_scene_number<'a>(text: &'a str) -> (Cow<'a, str>, Option<String>) {
    if let Some((start, end, number)) = locate_scene_number_span(text) {
        let mut cleaned = String::with_capacity(text.len() - (end - start));
        cleaned.push_str(&text[..start]);
        cleaned.push_str(&text[end..]);
        (Cow::Owned(cleaned), Some(number))
    } else {
        (Cow::Borrowed(text), None)
    }
}

fn lines_to_hunks(lines: Lines<'_>) -> Vec<Vec<&str>> {
    let mut hunks = lines.fold(vec![vec![]], |mut acc, line: &str| match line.trim() {
        // HANDLE BLANK LINES
        "" => {
            // If there are exactly two spaces in the line, it's intentional
            if line.len() == 2 {
                acc.last_mut().unwrap().push(line);
            // If the previous element was blank but it was the first element, do nothing
            } else if acc.last().unwrap().is_empty() && acc.len() == 1 {
                // do nothing
            } else if acc.last().unwrap().is_empty() {
                // If the previous element was also blank, create an empty string
                acc.last_mut().unwrap().push("");
            } else {
                // Otherwise, start a new element by pushing a new empty vec
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
            // If previous hunk was a section or blank, create a new hunk
            match last_line {
                Some(l) if l.starts_with('#') || l.is_empty() => acc.push(vec![]),
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

            if line.get(..1) == Some(".") {
                let (without_scene_number, scene_number) = strip_scene_number(stripped);
                if let Some(number) = scene_number {
                    attributes = Attributes {
                        scene_number: Some(number),
                        ..attributes
                    };
                }
                let final_text = remove_notes(without_scene_number.as_ref());
                make_element(Plain(final_text), attributes)
            } else {
                let final_text = remove_notes(stripped);
                make_element(Plain(final_text), attributes)
            }
        }
        _ if is_scene(&line) => {
            // Handle special case of scene numbers on scene headings
            let (without_scene_number, scene_number) = strip_scene_number(line);
            if let Some(number) = scene_number {
                attributes = Attributes {
                    scene_number: Some(number),
                    ..attributes
                };
            }
            let final_text = remove_notes(without_scene_number.as_ref());
            Element::SceneHeading(Plain(final_text), attributes)
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
    line.ends_with(" TO:")
}

fn tokenize_words(line: &str) -> Vec<String> {
    line.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn is_act_designator(token: &str) -> bool {
    matches!(
        token,
        "one" | "two" | "three" | "four" | "five" | "six" | "seven" | "eight" | "nine" | "ten"
    ) || token.chars().all(|c| c.is_ascii_digit())
}

fn is_end_act(line: &str) -> bool {
    let tokens = tokenize_words(line);
    if tokens.is_empty() || tokens[0] != "end" {
        return false;
    }

    let mut idx = 1;
    while idx < tokens.len() && tokens[idx] == "of" {
        idx += 1;
    }
    if idx >= tokens.len() {
        return false;
    }

    match tokens[idx].as_str() {
        "act" => tokens
            .get(idx + 1)
            .map(|next| is_act_designator(next))
            .unwrap_or(false),
        "cold" => tokens
            .get(idx + 1)
            .map(|next| next == "open")
            .unwrap_or(false),
        "teaser" => true,
        _ => false,
    }
}

fn is_new_act(line: &str) -> bool {
    let tokens = tokenize_words(line);
    if tokens.is_empty() {
        return false;
    }

    let mut idx = 0;
    while idx < tokens.len() {
        match tokens[idx].as_str() {
            "act" => {
                if tokens
                    .get(idx + 1)
                    .map(|next| is_act_designator(next))
                    .unwrap_or(false)
                {
                    return true;
                }
            }
            "cold" => {
                if tokens
                    .get(idx + 1)
                    .map(|next| next == "open")
                    .unwrap_or(false)
                {
                    return true;
                }
            }
            "teaser" => return true,
            _ => (),
        }
        idx += 1;
    }

    false
}

fn remove_notes(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut index = 0;

    while let Some(open_offset) = line[index..].find("[[") {
        let open_index = index + open_offset;
        result.push_str(&line[index..open_index]);

        let after_open = open_index + 2;
        if let Some(close_offset) = line[after_open..].find("]]") {
            index = after_open + close_offset + 2;
        } else {
            result.push_str(&line[open_index..]);
            return result;
        }
    }

    result.push_str(&line[index..]);
    result
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
    let mut result = Vec::new();
    let mut search_start = 0;

    while let Some(open_offset) = line[search_start..].find("[[") {
        let open_index = search_start + open_offset + 2;
        let tail = &line[open_index..];
        if let Some(close_offset) = tail.find("]]") {
            let close_index = open_index + close_offset;
            result.push(line[open_index..close_index].to_string());
            search_start = close_index + 2;
        } else {
            break;
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
    fn test_lines_to_hunks_odd_number_of_blanks() {
        let lines = "CHARACTER\nTalking talking talking--\n\n\nINT. PLACE - LATER\n\nA row of interview windows."
            .lines();
        let expected = vec![
            vec!["CHARACTER", "Talking talking talking--"],
            vec![""],
            vec!["INT. PLACE - LATER"],
            vec!["A row of interview windows."],
        ];

        assert_eq!(
            lines_to_hunks(lines),
            expected,
            "it should handle three returns in a row without creating weird groupings"
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
