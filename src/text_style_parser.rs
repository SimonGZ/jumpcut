use crate::{Element, Element::*, ElementText, ElementText::*, TextRun};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::mem;

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
    lazy_static! {
        static ref RE_BOLD_ITALIC: Regex = Regex::new(r"\*{3}([^*\n]*[^ \\])\*{3}").unwrap();
        static ref RE_BOLD: Regex = Regex::new(r"\*{2}([^*\n]*[^ \\])\*{2}").unwrap();
        static ref RE_ITALIC: Regex = Regex::new(r"(^|[^\\])\*{1}([^*\n]*[^ \\])\*{1}").unwrap();
        static ref RE_UNDERLINE: Regex = Regex::new(r"(^|[^\\])_{1}([^_\n]*[^ \\])_{1}").unwrap();
    }

    let has_star = txt.contains('*');
    let has_underscore = txt.contains('_');

    if !has_star && !has_underscore {
        if txt.contains('\\') {
            *txt = txt.replace("\\*", "*").replace("\\_", "_");
        }
        return Plain(mem::take(txt));
    }

    let has_bold_italic = has_star && RE_BOLD_ITALIC.is_match(txt);
    let has_bold = has_star && RE_BOLD.is_match(txt);
    let has_italic = has_star && RE_ITALIC.is_match(txt);
    let has_underline = has_underscore && RE_UNDERLINE.is_match(txt);

    if !has_bold_italic && !has_bold && !has_italic && !has_underline
    {
        // If none of the regexes match, just return a Plain(txt)
        if txt.contains('\\') {
            *txt = txt.replace("\\*", "*").replace("\\_", "_");
        }
        Plain(mem::take(txt))
    } else {
        let mut prepared_text = if has_bold_italic {
            RE_BOLD_ITALIC.replace_all(&txt, "⏋$1⏋").into_owned()
        } else {
            txt.to_string()
        };
        if has_bold {
            prepared_text = RE_BOLD.replace_all(&prepared_text, "⎿$1⎿").into_owned();
        }
        if has_italic {
            prepared_text = RE_ITALIC.replace_all(&prepared_text, "$1⏉$2⏉").into_owned();
        }
        if has_underline {
            prepared_text = RE_UNDERLINE
                .replace_all(&prepared_text, "$1⏊$2⏊")
                .into_owned();
        }
        if prepared_text.contains('\\') {
            prepared_text = prepared_text.replace("\\*", "*").replace("\\_", "_");
        }

        let mut styled_textruns: Vec<TextRun> = Vec::with_capacity(4);
        let mut current_text = String::new();
        let mut current_styles: HashSet<String> = HashSet::new();
        for ch in prepared_text.chars() {
            match ch {
                '⏋' => {
                    if current_styles.contains("Bold")
                        && current_styles.contains("Italic")
                    {
                        // Time to end Bold/Italic style
                        if current_text.is_empty() {
                            // Current text is empty but this style hunk is ending.
                            // That means these styles were immediately preceded by
                            // another style that just ended. We just need to remove
                            // these styles and we can move on to building the next
                            // hunk.
                        } else {
                            // Current text is NOT empty, so it's time to end this
                            // textrun.
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.remove("Bold");
                        current_styles.remove("Italic");
                    } else {
                        // Time to start this hunk
                        // See if there's another hunk we need to close off
                        if !current_text.is_empty() {
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.insert("Bold".to_string());
                        current_styles.insert("Italic".to_string());
                    }
                }
                '⏉' => {
                    if current_styles.contains("Italic") {
                        // Time to end Italic style
                        if current_text.is_empty() {
                            // Current text is empty but this style hunk is ending.
                            // That means these styles were immediately preceded by
                            // another style that just ended. We just need to remove
                            // these styles and we can move on to building the next
                            // hunk.
                        } else {
                            // Current text is NOT empty, so it's time to end this
                            // textrun.
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.remove("Italic");
                    } else {
                        // Time to start this hunk
                        // See if there's another hunk we need to close off
                        if !current_text.is_empty() {
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.insert("Italic".to_string());
                    }
                }
                '⎿' => {
                    if current_styles.contains("Bold") {
                        // Time to end Bold style
                        if current_text.is_empty() {
                            // Current text is empty but this style hunk is ending.
                            // That means these styles were immediately preceded by
                            // another style that just ended. We just need to remove
                            // these styles and we can move on to building the next
                            // hunk.
                        } else {
                            // Current text is NOT empty, so it's time to end this
                            // textrun.
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.remove("Bold");
                    } else {
                        // Time to start this hunk
                        // See if there's another hunk we need to close off
                        if !current_text.is_empty() {
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.insert("Bold".to_string());
                    }
                }
                '⏊' => {
                    if current_styles.contains("Underline") {
                        // Time to end Underline style
                        if current_text.is_empty() {
                            // Current text is empty but this style hunk is ending.
                            // That means these styles were immediately preceded by
                            // another style that just ended. We just need to remove
                            // these styles and we can move on to building the next
                            // hunk.
                        } else {
                            // Current text is NOT empty, so it's time to end this
                            // textrun.
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.remove("Underline");
                    } else {
                        // Time to start this hunk
                        // See if there's another hunk we need to close off
                        if !current_text.is_empty() {
                            styled_textruns.push(TextRun {
                                content: mem::take(&mut current_text),
                                text_style: current_styles.clone(),
                            });
                        }
                        current_styles.insert("Underline".to_string());
                    }
                }
                _ => current_text.push(ch),
            }
        }
        // Check if any text wasn't handled in the loop
        if !current_text.is_empty() {
            styled_textruns.push(TextRun {
                content: current_text,
                text_style: current_styles.clone(),
            });
        }
        Styled(styled_textruns)
    }
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

    #[test]
    fn test_parse_emphasis_with_emoji_sequence() {
        let mut element =
            Action(p("Family *👨‍👩‍👧‍👦* dinner"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Family ", vec![]),
                    tr("👨‍👩‍👧‍👦", vec!["Italic"]),
                    tr(" dinner", vec![]),
                ]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_bold_with_combining_mark_text() {
        let mut element =
            Action(p("Mix **Cafe\u{301} noir** tonight"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Mix ", vec![]),
                    tr("Cafe\u{301} noir", vec!["Bold"]),
                    tr(" tonight", vec![]),
                ]),
                blank_attributes()
            )
        );
    }

    #[test]
    fn test_parse_underline_with_non_latin_text() {
        let mut element =
            Action(p("Cue _Привет мир_ แล้วต่อ"), blank_attributes());
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![
                    tr("Cue ", vec![]),
                    tr("Привет мир", vec!["Underline"]),
                    tr(" แล้วต่อ", vec![]),
                ]),
                blank_attributes()
            )
        );
    }
}
