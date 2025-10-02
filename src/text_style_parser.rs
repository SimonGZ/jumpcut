use crate::{Element, Element::*, ElementText, ElementText::*, TextRun};
use std::cmp::min;
use std::collections::HashSet;

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
struct Delimiter {
    start: usize,
    len: usize,
    marker: u8,
    kind: StyleKind,
    can_open: bool,
    can_close: bool,
}

#[derive(Clone, Copy)]
struct MarkerEvent {
    position: usize,
    len: usize,
    sentinel: char,
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
    // Inspired by Markdown delimiter stacks (see pulldown-cmark); we maintain a
    // stack of potential openers and match them as we scan forward so the pass
    // stays linear even for long Fountain blocks.
    let mut delimiter_stack: Vec<Delimiter> = Vec::new();
    let mut events: Vec<MarkerEvent> = Vec::new();

    let mut iter = text.char_indices().peekable();
    let mut prev_char: Option<char> = None;

    while let Some((start_idx, ch)) = iter.next() {
        match ch {
            '\\' => {
                // Skip escaped characters so they never appear as potential markers.
                if let Some((_, escaped)) = iter.next() {
                    prev_char = Some(escaped);
                } else {
                    prev_char = Some('\\');
                }
            }
            '*' | '_' => {
                let marker = ch as u8;
                let mut run_len = 1;
                while let Some(&(_, next_ch)) = iter.peek() {
                    if next_ch == ch {
                        iter.next();
                        run_len += 1;
                    } else {
                        break;
                    }
                }

                let primary_len = match marker {
                    b'*' => min(3, run_len),
                    b'_' => 1,
                    _ => 1,
                };

                if let Some(kind) = style_kind(marker, primary_len) {
                    let next_char = iter.peek().map(|(_, c)| *c);
                    let delimiter = Delimiter {
                        start: start_idx,
                        len: primary_len,
                        marker,
                        kind,
                        can_open: can_open(next_char),
                        can_close: can_close(prev_char),
                    };

                    if delimiter.can_close {
                        let mut matched_index: Option<usize> = None;
                        for idx in (0..delimiter_stack.len()).rev() {
                            let candidate = &delimiter_stack[idx];
                            if candidate.marker == delimiter.marker
                                && candidate.len == delimiter.len
                                && candidate.can_open
                            {
                                let open_end = candidate.start + candidate.len;
                                if is_valid_content_slice(&text[open_end..start_idx]) {
                                    matched_index = Some(idx);
                                    break;
                                }
                            }
                        }

                        if let Some(idx) = matched_index {
                            let candidate = delimiter_stack.swap_remove(idx);
                            events.push(MarkerEvent {
                                position: candidate.start,
                                len: candidate.len,
                                sentinel: sentinel_for(candidate.kind),
                            });
                            events.push(MarkerEvent {
                                position: start_idx,
                                len: delimiter.len,
                                sentinel: sentinel_for(delimiter.kind),
                            });
                        } else if delimiter.can_open {
                            delimiter_stack.push(delimiter);
                        }
                    } else if delimiter.can_open {
                        delimiter_stack.push(delimiter);
                    }
                }

                prev_char = Some(ch);
            }
            _ => {
                prev_char = Some(ch);
            }
        }
    }

    if events.is_empty() {
        return unescape_markup(text);
    }

    events.sort_unstable_by(|a, b| a.position.cmp(&b.position));

    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut event_iter = events.into_iter().peekable();

    while cursor < text.len() {
        if let Some(event) = event_iter.peek() {
            if event.position == cursor {
                result.push(event.sentinel);
                cursor += event.len;
                event_iter.next();
                continue;
            }
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
