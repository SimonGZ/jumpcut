use crate::{Element, Element::*, ElementText, ElementText::*, TextRun};
use std::cmp::min;
use std::collections::{BTreeMap, HashSet};

const SENTINEL_BOLD_ITALIC: char = '⏋';
const SENTINEL_BOLD: char = '⎿';
const SENTINEL_ITALIC: char = '⏉';
const SENTINEL_UNDERLINE: char = '⏊';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StyleKind {
    Italic,
    Bold,
    BoldItalic,
    Underline,
}

#[derive(Clone, Copy)]
struct MarkerPair {
    open_start: usize,
    open_len: usize,
    close_start: usize,
    close_len: usize,
    kind: StyleKind,
}

#[derive(Clone, Copy)]
struct Delimiter {
    start: usize,
    len: usize,
    marker: u8,
    kind: StyleKind,
    can_open: bool,
    can_close: bool,
}

fn sentinel_for(kind: StyleKind) -> char {
    match kind {
        StyleKind::Italic => SENTINEL_ITALIC,
        StyleKind::Bold => SENTINEL_BOLD,
        StyleKind::BoldItalic => SENTINEL_BOLD_ITALIC,
        StyleKind::Underline => SENTINEL_UNDERLINE,
    }
}

fn style_kind(marker: u8, len: usize) -> Option<StyleKind> {
    match (marker, len) {
        (b'*', 1) => Some(StyleKind::Italic),
        (b'*', 2) => Some(StyleKind::Bold),
        (b'*', 3) => Some(StyleKind::BoldItalic),
        (b'_', 1) => Some(StyleKind::Underline),
        _ => None,
    }
}

fn is_whitespace(ch: Option<char>) -> bool {
    matches!(ch, Some(c) if c.is_whitespace())
}

fn is_newline(ch: Option<char>) -> bool {
    matches!(ch, Some('\n' | '\r'))
}

fn can_open(next: Option<char>) -> bool {
    !is_whitespace(next) && !is_newline(next)
}

fn can_close(prev: Option<char>) -> bool {
    match prev {
        Some(ch) => !ch.is_whitespace() && ch != '\\' && ch != '\n' && ch != '\r',
        None => false,
    }
}

fn is_valid_content_slice(slice: &str) -> bool {
    if slice.is_empty() {
        return false;
    }
    if slice.chars().any(|c| c == '\n' || c == '\r') {
        return false;
    }
    if let Some(last) = slice.chars().rev().next() {
        if last.is_whitespace() || last == '\\' {
            return false;
        }
    }
    true
}

fn insert_markers(text: &str) -> String {
    let char_indices: Vec<(usize, char)> = text.char_indices().collect();
    let mut char_pos = 0;
    // Inspired by Markdown delimiter stacks (see pulldown-cmark); we maintain a
    // stack of potential openers and match them as we scan forward so the pass
    // stays linear even for long Fountain blocks.
    let mut delimiter_stack: Vec<Delimiter> = Vec::new();
    let mut pairs: Vec<MarkerPair> = Vec::new();

    while char_pos < char_indices.len() {
        let (byte_index, ch) = char_indices[char_pos];
        match ch {
            '\\' => {
                if char_pos + 1 < char_indices.len() {
                    char_pos += 2;
                } else {
                    char_pos += 1;
                }
            }
            '*' | '_' => {
                let marker = ch as u8;
                let run_start = byte_index;
                let mut run_len_chars = 1;
                while char_pos + run_len_chars < char_indices.len()
                    && char_indices[char_pos + run_len_chars].1 == ch
                {
                    run_len_chars += 1;
                }

                let primary_len = match marker {
                    b'*' => min(3, run_len_chars),
                    b'_' => 1,
                    _ => 1,
                };

                if let Some(kind) = style_kind(marker, primary_len) {
                    let prev = if char_pos == 0 {
                        None
                    } else {
                        Some(char_indices[char_pos - 1].1)
                    };
                    let next = if char_pos + run_len_chars < char_indices.len() {
                        Some(char_indices[char_pos + run_len_chars].1)
                    } else {
                        None
                    };

                    let delim = Delimiter {
                        start: run_start,
                        len: primary_len,
                        marker,
                        kind,
                        can_open: can_open(next),
                        can_close: can_close(prev),
                    };

                    if delim.can_close {
                        let mut matched = false;
                        let mut search_idx = delimiter_stack.len();
                        while search_idx > 0 {
                            search_idx -= 1;
                            let candidate = delimiter_stack[search_idx];
                            if candidate.marker == delim.marker
                                && candidate.len == delim.len
                                && candidate.can_open
                            {
                                let open_end = candidate.start + candidate.len;
                                let close_start = run_start;
                                if is_valid_content_slice(&text[open_end..close_start]) {
                                    pairs.push(MarkerPair {
                                        open_start: candidate.start,
                                        open_len: candidate.len,
                                        close_start: run_start,
                                        close_len: delim.len,
                                        kind: delim.kind,
                                    });
                                    delimiter_stack.remove(search_idx);
                                    matched = true;
                                    break;
                                }
                            }
                        }
                        if !matched && delim.can_open {
                            delimiter_stack.push(delim);
                        }
                    } else if delim.can_open {
                        delimiter_stack.push(delim);
                    }
                }

                char_pos += run_len_chars;
            }
            _ => {
                char_pos += 1;
            }
        }
    }

    if pairs.is_empty() {
        return unescape_markup(text);
    }

    let mut open_events: BTreeMap<usize, (usize, StyleKind)> = BTreeMap::new();
    let mut close_events: BTreeMap<usize, (usize, StyleKind)> = BTreeMap::new();
    for pair in pairs {
        open_events.insert(pair.open_start, (pair.open_len, pair.kind));
        close_events.insert(pair.close_start, (pair.close_len, pair.kind));
    }

    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;
    while cursor < text.len() {
        if let Some(&(len, kind)) = open_events.get(&cursor) {
            result.push(sentinel_for(kind));
            cursor += len;
            continue;
        }
        if let Some(&(len, kind)) = close_events.get(&cursor) {
            result.push(sentinel_for(kind));
            cursor += len;
            continue;
        }
        if text.as_bytes()[cursor] == b'\\'
            && cursor + 1 < text.len()
            && matches!(text.as_bytes()[cursor + 1], b'*' | b'_')
        {
            result.push(text.as_bytes()[cursor + 1] as char);
            cursor += 2;
            continue;
        }
        let ch = text[cursor..].chars().next().unwrap();
        result.push(ch);
        cursor += ch.len_utf8();
    }

    result
}

fn unescape_markup(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(next @ ('*' | '_')) => result.push(next),
                Some(next) => {
                    result.push('\\');
                    result.push(next);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }
    result
}

impl Element {
    pub fn parse_and_convert_markup(&mut self) {
        match self {
            Action(plain, _) => *plain = convert_plain_to_styled(plain),
            Character(plain, _) => *plain = convert_plain_to_styled(plain),
            SceneHeading(plain, _) => *plain = convert_plain_to_styled(plain),
            Lyric(plain, _) => *plain = convert_plain_to_styled(plain),
            Parenthetical(plain, _) => *plain = convert_plain_to_styled(plain),
            Dialogue(plain, _) => *plain = convert_plain_to_styled(plain),
            Transition(plain, _) => *plain = convert_plain_to_styled(plain),
            ColdOpening(plain, _) => *plain = convert_plain_to_styled(plain),
            NewAct(plain, _) => *plain = convert_plain_to_styled(plain),
            EndOfAct(plain, _) => *plain = convert_plain_to_styled(plain),
            DialogueBlock(elements) => {
                for e in elements {
                    e.parse_and_convert_markup()
                }
            }
            DualDialogueBlock(elements) => {
                for e in elements {
                    e.parse_and_convert_markup()
                }
            }
            Section(_, _, _) => (),
            Synopsis(_) => (),
            PageBreak => (),
        };
    }
}

fn convert_plain_to_styled(plain: &mut ElementText) -> ElementText {
    match plain {
        Plain(txt) => create_styled_from_string(txt),
        _ => unreachable!(),
    }
}

fn create_styled_from_string(txt: &mut String) -> ElementText {
    let prepared_text = insert_markers(txt);
    if !prepared_text.chars().any(|ch| {
        matches!(
            ch,
            SENTINEL_BOLD_ITALIC | SENTINEL_BOLD | SENTINEL_ITALIC | SENTINEL_UNDERLINE
        )
    }) {
        *txt = prepared_text;
        return Plain(txt.to_string());
    }

    let mut styled_textruns: Vec<TextRun> = Vec::new();
    let mut current_text = String::new();
    let mut current_styles: HashSet<String> = HashSet::new();

    for ch in prepared_text.chars() {
        match ch {
            SENTINEL_BOLD_ITALIC => {
                if current_styles.contains("Bold") && current_styles.contains("Italic") {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text.clear();
                    }
                    current_styles.remove("Bold");
                    current_styles.remove("Italic");
                } else {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text.clear();
                    current_styles.insert("Bold".to_string());
                    current_styles.insert("Italic".to_string());
                }
            }
            SENTINEL_ITALIC => {
                if current_styles.contains("Italic") {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text.clear();
                    }
                    current_styles.remove("Italic");
                } else {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text.clear();
                    current_styles.insert("Italic".to_string());
                }
            }
            SENTINEL_BOLD => {
                if current_styles.contains("Bold") {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text.clear();
                    }
                    current_styles.remove("Bold");
                } else {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text.clear();
                    current_styles.insert("Bold".to_string());
                }
            }
            SENTINEL_UNDERLINE => {
                if current_styles.contains("Underline") {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text.clear();
                    }
                    current_styles.remove("Underline");
                } else {
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text.clear();
                    current_styles.insert("Underline".to_string());
                }
            }
            _ => current_text.push(ch),
        }
    }

    if !current_text.is_empty() {
        styled_textruns.push(TextRun {
            content: current_text,
            text_style: current_styles,
        });
    }

    Styled(styled_textruns)
}

// * Tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::{blank_attributes, p};
    use pretty_assertions::assert_eq;

    // Convenience function to make writing tests easier.
    fn tr(content: &str, styles: Vec<&str>) -> TextRun {
        let mut style_strings: HashSet<String> = HashSet::new();
        for style in styles {
            style_strings.insert(style.to_string());
        }
        TextRun {
            content: content.to_string(),
            text_style: style_strings,
        }
    }

    #[test]
    fn test_parse_with_element() {
        let mut element = Action(p("something"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(element, Action(p("something"), blank_attributes()));
    }

    #[test]
    fn test_parse_bold_emphasis() {
        let mut element = Action(p("Fuck ***this*** whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold", "Italic"]),
                    tr(" whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("***Fuck*** this whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck", vec!["Bold", "Italic"]),
                    tr(" this whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck this whole ***place!***"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck this whole ", vec![]),
                    tr("place!", vec!["Bold", "Italic"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck ***this*** whole ***place!***"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold", "Italic"]),
                    tr(" whole ", vec![]),
                    tr("place!", vec!["Bold", "Italic"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("***Fuck this whole place!***"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![tr("Fuck this whole place!", vec!["Bold", "Italic"]),]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_emphasis() {
        let mut element = Action(p("Fuck *this* whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Italic"]),
                    tr(" whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("*Fuck* this whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck", vec!["Italic"]),
                    tr(" this whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck this whole *place!*"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck this whole ", vec![]),
                    tr("place!", vec!["Italic"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck *this* whole *place!*"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Italic"]),
                    tr(" whole ", vec![]),
                    tr("place!", vec!["Italic"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("*Fuck this whole place!*"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![tr("Fuck this whole place!", vec!["Italic"]),]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_bold() {
        let mut element = Action(p("Fuck **this** whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold"]),
                    tr(" whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("**Fuck** this whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck", vec!["Bold"]),
                    tr(" this whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck this whole **place!**"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck this whole ", vec![]),
                    tr("place!", vec!["Bold"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck **this** whole **place!**"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold"]),
                    tr(" whole ", vec![]),
                    tr("place!", vec!["Bold"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("**Fuck this whole place!**"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![tr("Fuck this whole place!", vec!["Bold"]),]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_underline() {
        let mut element = Action(p("Fuck _this_ whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Underline"]),
                    tr(" whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(p("Fuck _this_ whole _place!_"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Underline"]),
                    tr(" whole ", vec![]),
                    tr("place!", vec!["Underline"]),
                ]),
                blank_attributes()
            )
        );
        element = Action(p("_Fuck this whole place!_"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![tr("Fuck this whole place!", vec!["Underline"]),]),
                blank_attributes()
            )
        );
        element = NewAct(p("_ACT ONE_"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            NewAct(
                Styled(vec![tr("ACT ONE", vec!["Underline"])]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_underline_bold_italic() {
        let mut element = Action(p("Fuck ***_this_*** whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold", "Italic", "Underline"]),
                    tr(" whole place!", vec![])
                ]),
                blank_attributes()
            )
        );
        element = Action(
            p("Fuck ***_this_*** whole _***place!***_"),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec![]),
                    tr("this", vec!["Bold", "Italic", "Underline"]),
                    tr(" whole ", vec![]),
                    tr("place!", vec!["Bold", "Italic", "Underline"]),
                ]),
                blank_attributes()
            )
        )
    }

    #[test]
    fn test_parse_overlapping_styles() {
        let mut element = Action(p("_Fuck *this* whole **place!**_"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Fuck ", vec!["Underline"]),
                    tr("this", vec!["Italic", "Underline"]),
                    tr(" whole ", vec!["Underline"]),
                    tr("place!", vec!["Bold", "Underline"]),
                ]),
                blank_attributes()
            )
        )
    }

    #[test]
    fn test_parse_special_cases() {
        let mut element = Action(p("*Fuck this whole place!"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(p("*Fuck this whole place!"), blank_attributes())
        );
        element = Action(
            p("He dialed *69 and then *23, and then hung up."),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                p("He dialed *69 and then *23, and then hung up."),
                blank_attributes()
            )
        );
        element = Action(
            p("He dialed *69 and then 23\\*, and then hung up."),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                p("He dialed *69 and then 23*, and then hung up."),
                blank_attributes()
            )
        );
        element = Action(
            p("He dialed _69 and then 23\\_, and then hung up."),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                p("He dialed _69 and then 23_, and then hung up."),
                blank_attributes()
            )
        );
        element = Action(
            p("As he rattles off the long list, Brick and Steel *share a look.\nThis is going to be BAD.*"),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                p("As he rattles off the long list, Brick and Steel *share a look.\nThis is going to be BAD.*"),
                blank_attributes()
            )
        );
    }
}
