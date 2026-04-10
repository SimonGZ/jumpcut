use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::Lines;

use crate::pagination::ScreenplayLayoutProfile;
use crate::Element::PageBreak;
use crate::{
    blank_attributes, text_style_parser, Attributes, Element, ElementText, Metadata, Screenplay,
};
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
    apply_structural_act_break_policy(&mut elements, &metadata);
    Screenplay { elements, metadata }
}

fn apply_structural_act_break_policy(elements: &mut [Element], metadata: &Metadata) {
    let mut saw_prior_opener = false;
    let auto_new_act_page_breaks = ScreenplayLayoutProfile::from_metadata(metadata)
        .styles
        .new_act
        .starts_new_page;

    for element in elements {
        match element {
            Element::ColdOpening(_, _) => {
                saw_prior_opener = true;
            }
            Element::NewAct(_, attributes) => {
                if auto_new_act_page_breaks && saw_prior_opener && !attributes.starts_new_page {
                    attributes.starts_new_page = true;
                }
                saw_prior_opener = true;
            }
            _ => {}
        }
    }
}

fn has_key_value(txt: &str) -> bool {
    split_metadata_line(txt).is_some()
}

fn process_metadata(metadata: &mut Metadata, text: &str) {
    let lines = text.lines();
    let mut current_key = "".to_string();
    for line in lines {
        if let Some((key, current_value)) = split_metadata_line(line) {
            current_key = key.to_lowercase().to_string();
            if current_value.is_empty() {
                metadata.insert(current_key.to_string(), vec![]);
            } else {
                metadata.insert(
                    current_key.to_string(),
                    vec![parse_metadata_value(current_value.trim())],
                );
            }
        } else {
            // Means we have a line without a key and thus an additional value to push
            if let Some(values) = metadata.get_mut(&current_key) {
                values.push(parse_metadata_value(line.trim()));
            }
        }
    }
}

fn parse_metadata_value(value: &str) -> ElementText {
    let mut text = value.to_string();
    text_style_parser::parse_plain_text_markup(&mut text)
}

fn split_metadata_line(line: &str) -> Option<(&str, &str)> {
    let line = trim_classifier_start(line);
    let colon_index = line.find(':')?;
    let key = line.get(..colon_index)?;
    let first = key.chars().next()?;
    if key.chars().count() < 2
        || first.is_whitespace()
        || matches!(first, '!' | '.' | '@' | '~' | '>')
        || key.contains(['\n', '\r'])
    {
        return None;
    }

    let value = line.get(colon_index + 1..)?;
    Some((key, value))
}

/// Strips out problematic unicode and the boneyard element
fn prepare_text(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/").unwrap();
    }
    RE.replace_all(text.trim_end(), "").to_string()
}

fn is_classifier_invisible(ch: char) -> bool {
    matches!(
        ch,
        '\u{061C}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
    )
}

fn trim_classifier_start(line: &str) -> &str {
    let start = line
        .char_indices()
        .find(|(_, ch)| !is_classifier_invisible(*ch))
        .map(|(idx, _)| idx)
        .unwrap_or(line.len());
    &line[start..]
}

fn trim_classifier_end(line: &str) -> &str {
    let end = line
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_classifier_invisible(*ch))
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    &line[..end]
}

fn trim_classifier_edges(line: &str) -> &str {
    trim_classifier_end(trim_classifier_start(line))
}

fn classifier_trimmed(line: &str) -> &str {
    trim_classifier_edges(line).trim()
}

fn lines_to_hunks<'a>(lines: Lines<'a>) -> Vec<Vec<&'a str>> {
    let mut hunks = lines.fold(vec![vec![]], |mut acc, line: &str| {
        let classified = classifier_trimmed(line);
        match classified {
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
                let last_classified = acc
                    .last()
                    .unwrap()
                    .first()
                    .map(|line| classifier_trimmed(line));
                // If previous hunk was a section or blank, create a new hunk
                match last_classified {
                    Some(l) if l.starts_with('#') || l.is_empty() => acc.push(vec![]),
                    _ => (),
                }
                acc.last_mut().unwrap().push(line);
                acc
            }
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
        static ref NOTE_REGEX: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    }
    let mut attributes = blank_attributes();
    let line_has_note = has_note(line);
    if line_has_note {
        let notes = retrieve_notes(line);
        attributes = Attributes {
            notes,
            ..attributes
        };
    }
    match make_forced(&line) {
        Some(make_element) => {
            let stripped: &str = trim_classifier_start(line)
                .trim_start_matches(&['!', '@', '~', '.', '>', '='][..])
                .trim_start();

            if trim_classifier_edges(line).get(..1) == Some(".")
                && extract_scene_number(stripped).is_some()
            {
                // Handle special case of scene numbers on scene headings
                match extract_scene_number(stripped) {
                    None => make_element(Plain(stripped.to_string()), attributes),
                    Some((text_without_scene_number, scene_number)) => {
                        attributes = Attributes {
                            scene_number: Some(scene_number),
                            ..attributes
                        };
                        let final_text = if line_has_note {
                            remove_notes(&text_without_scene_number)
                        } else {
                            text_without_scene_number
                        };
                        make_element(Plain(final_text), attributes)
                    }
                }
            } else {
                let final_text = if line_has_note {
                    remove_notes(stripped)
                } else {
                    stripped.to_string()
                };
                make_element(Plain(final_text), attributes)
            }
        }
        _ if is_scene(&line) => {
            let line = trim_classifier_edges(line);
            // Handle special case of scene numbers on scene headings
            if extract_scene_number(line).is_some() {
                match extract_scene_number(line) {
                    None => Element::SceneHeading(Plain(line.to_string()), attributes),
                    Some((text_without_scene_number, scene_number)) => {
                        attributes = Attributes {
                            scene_number: Some(scene_number),
                            ..attributes
                        };
                        let final_text = if line_has_note {
                            remove_notes(&text_without_scene_number)
                        } else {
                            text_without_scene_number
                        };
                        Element::SceneHeading(Plain(final_text), attributes)
                    }
                }
            } else {
                let final_text = if line_has_note {
                    remove_notes(line)
                } else {
                    line.to_string()
                };
                Element::SceneHeading(Plain(final_text), attributes)
            }
        }
        _ if is_transition(&line) => {
            let line = classifier_trimmed(line);
            let final_text = if line_has_note {
                remove_notes(line)
            } else {
                line.to_string()
            };
            Element::Transition(Plain(final_text), attributes)
        }
        _ if is_centered(&line) => {
            let line = trim_classifier_edges(line);
            let final_text = if line_has_note {
                remove_notes(trim_centered_marks(line))
            } else {
                trim_centered_marks(line).to_string()
            };
            if is_end_act(&final_text) {
                // Check end_act first because new act regex also matches end act regex
                Element::EndOfAct(
                    Plain(final_text),
                    Attributes {
                        centered: true,
                        ..attributes
                    },
                )
            } else if is_cold_opening(&final_text) {
                Element::ColdOpening(
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
            let final_text = if line_has_note {
                remove_notes(line)
            } else {
                line.to_string()
            };
            Element::Action(Plain(final_text), attributes)
        }
    }
}

fn extract_scene_number(line: &str) -> Option<(String, String)> {
    let hash_start = line.rfind(" #")?;
    let scene_number = line.get(hash_start + 1..)?;
    if !scene_number.starts_with('#') || !scene_number.ends_with('#') || scene_number.len() < 2 {
        return None;
    }

    let scene_number = scene_number.trim_matches('#').trim().to_string();
    let text_without_scene_number = line.get(..hash_start)?.to_string();
    Some((text_without_scene_number, scene_number))
}

fn make_multi_line_element(hunk: Vec<&str>) -> Element {
    let top_line = hunk[0];
    let top_classified = classifier_trimmed(top_line);
    let forced_element = make_forced(top_line);
    if top_classified.starts_with('@')
        || (forced_element.is_none()
            && is_character(top_line)
            && !hunk.iter().any(|&line| is_centered(line)))
    {
        return make_dialogue_block(hunk);
    }

    let mut attributes = blank_attributes();
    let joined_hunk_with_notes = hunk
        .iter()
        .any(|line| has_note(line))
        .then(|| hunk.join("\n"));
    if let Some(joined_hunk) = joined_hunk_with_notes.as_deref() {
        let notes = retrieve_notes(joined_hunk);
        attributes = Attributes {
            notes,
            ..attributes
        };
    }
    match forced_element {
        Some(make_element) => {
            // Check if it's a forced character because that means dialogueblock
            if top_classified.get(..1) == Some("@") {
                let stripped_hunk = hunk
                    .into_iter()
                    .map(|l| trim_classifier_start(l).trim_start_matches('@'))
                    .collect::<Vec<&str>>();
                make_dialogue_block(stripped_hunk)
            } else {
                // It's not forced character, so we can create a string with newlines
                let stripped_string = hunk
                    .into_iter()
                    .map(|l| {
                        trim_classifier_start(l)
                            .trim_start_matches(&['!', '@', '~', '.', '>', '='][..])
                    })
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
            let final_text = match joined_hunk_with_notes.as_deref() {
                Some(joined_hunk) => remove_notes(joined_hunk),
                None => hunk.join("\n"),
            };
            Element::Action(Plain(final_text), attributes)
        }
    }
}

fn is_scene(line: &str) -> bool {
    let line = trim_classifier_start(line);
    SCENE_LOCATORS.iter().any(|&locator| {
        line.get(..locator.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(locator))
    })
}

fn is_transition(line: &str) -> bool {
    let line = classifier_trimmed(line);
    line.len() >= 4
        && line
            .get(line.len() - 4..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(" TO:"))
}

fn is_end_act(line: &str) -> bool {
    let owned = line.to_lowercase();
    let tokens = split_lowercase_tokens(&owned);
    if tokens.first().copied() != Some("end") {
        return false;
    }

    let mut cursor = 1;
    while tokens.get(cursor).copied() == Some("of") {
        cursor += 1;
    }
    is_act_marker(&tokens[cursor..])
}

fn is_new_act(line: &str) -> bool {
    let owned = line.to_lowercase();
    let tokens = split_lowercase_tokens(&owned);
    is_act_marker(&tokens)
}

fn is_cold_opening(line: &str) -> bool {
    let owned = line.to_lowercase();
    let tokens = split_lowercase_tokens(&owned);
    matches!(
        tokens.as_slice(),
        ["cold", "open", ..] | ["cold", "opening", ..]
    )
}

fn split_lowercase_tokens<'a>(line: &'a str) -> Vec<&'a str> {
    line.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect()
}

fn is_act_marker(tokens: &[&str]) -> bool {
    match tokens {
        ["teaser", ..] => true,
        ["cold", "open", ..] => true,
        ["cold", "opening", ..] => true,
        ["act", label, ..] => is_supported_act_label(label),
        _ => false,
    }
}

fn is_supported_act_label(label: &str) -> bool {
    matches!(
        label,
        "0" | "1"
            | "2"
            | "3"
            | "4"
            | "5"
            | "6"
            | "7"
            | "8"
            | "9"
            | "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
    )
}

fn remove_notes(line: &str) -> String {
    lazy_static! {
        static ref NOTE_REGEX: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    }
    NOTE_REGEX.replace_all(line, "").to_string()
}

fn is_centered(line: &str) -> bool {
    let trimmed = classifier_trimmed(line);
    trimmed.starts_with('>') && trimmed.ends_with('<')
}

fn trim_centered_marks(line: &str) -> &str {
    trim_classifier_edges(line)
        .trim_matches(&['>', '<'][..])
        .trim()
}

fn is_character(line: &str) -> bool {
    !line.chars().any(char::is_lowercase)
}

fn is_parenthetical(line: &str) -> bool {
    let trimmed = classifier_trimmed(line);
    trimmed.starts_with('(') && trimmed.ends_with(')')
}

fn is_lyric(line: &str) -> bool {
    classifier_trimmed(line).starts_with('~')
}

fn is_dual_dialogue(line: &str) -> bool {
    classifier_trimmed(line).ends_with('^')
}

fn has_note(line: &str) -> bool {
    line.contains("[[")
}

fn make_forced(line: &str) -> Option<fn(ElementText, Attributes) -> Element> {
    let line = trim_classifier_edges(line);
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
    let clean_name: &str = trim_classifier_edges(raw_name)
        .trim_start_matches('@')
        .trim_end_matches('^')
        .trim();
    let character: Element = Element::Character(Plain(clean_name.to_string()), blank_attributes());
    elements.push(character);
    for line in hunk[1..].iter() {
        let (processed_line, attributes) = if has_note(line) {
            let notes = retrieve_notes(line);
            (
                Cow::Owned(remove_notes(line)),
                Attributes {
                    notes,
                    ..blank_attributes()
                },
            )
        } else {
            (Cow::Borrowed(*line), blank_attributes())
        };
        if is_parenthetical(processed_line.as_ref()) {
            elements.push(Element::Parenthetical(
                Plain(classifier_trimmed(processed_line.as_ref()).to_string()),
                attributes,
            ));
        } else if is_lyric(processed_line.as_ref()) {
            let stripped_line = classifier_trimmed(processed_line.as_ref())
                .trim_start_matches('~')
                .trim();
            if let Element::Lyric(Plain(s), _) = elements.last_mut().unwrap() {
                // if previous element was lyric and so is this one, add this line to that previous lyric
                s.push_str("\n");
                s.push_str(stripped_line);
            } else {
                // this line is lyric but previous line wasn't, create new lyric element
                elements.push(Element::Lyric(Plain(stripped_line.to_string()), attributes));
            }
        } else if let Element::Dialogue(Plain(s), _) = elements.last_mut().unwrap() {
            // if previous element was dialogue, add this line to that dialogue
            s.push_str("\n");
            let trimmed = processed_line.as_ref();
            let trimmed = if trimmed.trim().is_empty() {
                trimmed
            } else {
                trimmed.trim_start()
            };
            s.push_str(trimmed);
        } else {
            // otherwise this is a new dialogue
            elements.push(Element::Dialogue(
                Plain(if processed_line.trim().is_empty() {
                    processed_line.to_string()
                } else {
                    processed_line.trim_start().to_string()
                }),
                attributes,
            ));
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
    use super::*;
    use crate::{p, tr, ElementText::Styled};

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
    fn test_prepare_text_preserves_internal_invisible_chars() {
        let unicode_string = "Hello\u{200B}, \u{200D}\u{FEFF}World!";
        assert_eq!(prepare_text(unicode_string), unicode_string);
    }

    #[test]
    fn test_parse_handles_leading_bom_before_metadata() {
        let fountain = "\u{FEFF}Title: Example Script\nAuthor: Test Writer\n";
        let screenplay = parse(fountain);
        assert_eq!(
            screenplay.metadata.get("title"),
            Some(&vec![p("Example Script")])
        );
        assert_eq!(
            screenplay.metadata.get("author"),
            Some(&vec![p("Test Writer")])
        );
    }

    #[test]
    fn test_parse_preserves_styled_title_page_metadata() {
        let fountain = "Title: _**BRICK & STEEL**_\nCredit: Written by\nAuthor: *Stu Maschwitz*\n";
        let screenplay = parse(fountain);

        assert_eq!(
            screenplay.metadata.get("title"),
            Some(&vec![Styled(vec![tr(
                "BRICK & STEEL",
                vec!["Bold", "Underline"]
            )])])
        );
        assert_eq!(
            screenplay.metadata.get("author"),
            Some(&vec![Styled(vec![tr("Stu Maschwitz", vec!["Italic"])])])
        );
    }

    #[test]
    fn test_parse_handles_leading_format_chars_before_scene_heading() {
        let fountain = "\u{200B}\u{2060}INT. HOUSE - DAY";
        assert_eq!(
            parse(fountain).elements,
            vec![Element::SceneHeading(
                p("INT. HOUSE - DAY"),
                blank_attributes()
            )]
        );
    }

    #[test]
    fn test_prepare_text_trims_only_trailing_whitespace() {
        let fountain = "Title: Example  \n\n\t";
        assert_eq!(prepare_text(fountain), "Title: Example");
    }

    #[test]
    fn test_remove_boneyard() {
        let boneyard = "/* boneyard */Hello, World!\n\n/* More bones \n Lower bones*/Goodbye!";
        assert_eq!(prepare_text(boneyard), "Hello, World!\n\nGoodbye!");
    }

    #[test]
    fn test_remove_boneyard_preserves_line_boundaries() {
        let boneyard = "Title: Example\n/* note\nstill note */\nINT. ROOM - DAY";
        assert_eq!(prepare_text(boneyard), "Title: Example\n\nINT. ROOM - DAY");
    }

    #[test]
    fn test_prepare_text_preserves_zwj_emoji_sequence() {
        let emoji = "Family: 👨‍👩‍👧‍👦";
        assert_eq!(prepare_text(emoji), emoji);
    }
}
