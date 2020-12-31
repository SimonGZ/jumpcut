use crate::{Element, Element::*, ElementText, ElementText::*, TextRun};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

impl Element {
    pub fn parse_and_convert_markup(&mut self) {
        let text_element = match self {
            Action(plain, _) => plain,
            _ => unreachable!(),
        };
        *text_element = convert_plain_to_styled(text_element);
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
    let mut prepared_text = RE_BOLD_ITALIC.replace_all(&txt, "⏋$1⏋").into_owned();
    println!("{}", prepared_text);
    prepared_text = RE_BOLD.replace_all(&prepared_text, "⎿$1⎿").into_owned();
    prepared_text = RE_ITALIC.replace_all(&prepared_text, "$1⏉$2⏉").into_owned();
    prepared_text = RE_UNDERLINE
        .replace_all(&prepared_text, "$1⏊$2⏊")
        .into_owned();
    prepared_text = prepared_text.replace("\\*", "*").replace("\\_", "_");

    let mut styled_textruns: Vec<TextRun> = vec![];
    let mut current_text: String = "".to_string();
    let mut current_styles: HashSet<String> = HashSet::new();
    for char in prepared_text.graphemes(true) {
        match char {
            "⏋" => {
                if current_styles.contains(&"Bold".to_string())
                    && current_styles.contains(&"Italic".to_string())
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
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text = "".to_string();
                    }
                    current_styles.remove(&"Bold".to_string());
                    current_styles.remove(&"Italic".to_string());
                } else {
                    // Time to start this hunk
                    // See if there's another hunk we need to close off
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text = "".to_string();
                    current_styles.insert("Bold".to_string());
                    current_styles.insert("Italic".to_string());
                }
            }
            "⏉" => {
                if current_styles.contains(&"Italic".to_string()) {
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
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text = "".to_string();
                    }
                    current_styles.remove(&"Italic".to_string());
                } else {
                    // Time to start this hunk
                    // See if there's another hunk we need to close off
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text = "".to_string();
                    current_styles.insert("Italic".to_string());
                }
            }
            "⎿" => {
                if current_styles.contains(&"Bold".to_string()) {
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
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text = "".to_string();
                    }
                    current_styles.remove(&"Bold".to_string());
                } else {
                    // Time to start this hunk
                    // See if there's another hunk we need to close off
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text = "".to_string();
                    current_styles.insert("Bold".to_string());
                }
            }
            "⏊" => {
                if current_styles.contains(&"Underline".to_string()) {
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
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                        current_text = "".to_string();
                    }
                    current_styles.remove(&"Underline".to_string());
                } else {
                    // Time to start this hunk
                    // See if there's another hunk we need to close off
                    if !current_text.is_empty() {
                        styled_textruns.push(TextRun {
                            content: current_text.clone(),
                            text_style: current_styles.clone(),
                        });
                    }
                    current_text = "".to_string();
                    current_styles.insert("Underline".to_string());
                }
            }
            _ => current_text.push_str(char),
        }
    }
    // Check if any text wasn't handled in the loop
    if !current_text.is_empty() {
        styled_textruns.push(TextRun {
            content: current_text.clone(),
            text_style: current_styles.clone(),
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

    // Quick helper function to make writing tests easier.
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
        assert_eq!(
            element,
            Action(
                Styled(vec![TextRun {
                    content: "something".to_string(),
                    text_style: HashSet::new()
                }]),
                blank_attributes()
            )
        );
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
            Action(
                Styled(vec![tr("*Fuck this whole place!", vec![]),]),
                blank_attributes()
            )
        );
        element = Action(
            p("He dialed *69 and then *23, and then hung up."),
            blank_attributes(),
        );
        element.parse_and_convert_markup();
        assert_eq!(
            element,
            Action(
                Styled(vec![tr(
                    "He dialed *69 and then *23, and then hung up.",
                    vec![]
                ),]),
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
                Styled(vec![tr(
                    "He dialed *69 and then 23*, and then hung up.",
                    vec![]
                ),]),
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
                Styled(vec![tr(
                    "He dialed _69 and then 23_, and then hung up.",
                    vec![]
                ),]),
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
                Styled(vec![tr(
                    "As he rattles off the long list, Brick and Steel *share a look.\nThis is going to be BAD.*",
                    vec![]
                ),]),
                blank_attributes()
            )
        );
    }
}
