use jumpcut::{blank_attributes, p, parse, Element, ElementText::Styled, TextRun};
use pretty_assertions::assert_eq;
use std::collections::HashSet;

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
fn it_trims_leading_whitespace_from_characters() {
    // From Brick-n-Steel.fountain: "\t\t\tJACK" should parse as Character "JACK"
    let text = "\n\t\t\tJACK\nHello!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("JACK"), blank_attributes()),
        Element::Dialogue(p("Hello!"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from character names"
    );
}

#[test]
fn it_trims_leading_whitespace_from_parentheticals() {
    // From Brick-n-Steel.fountain: "\t\t(in Vietnamese, subtitled)"
    let text = "JACK\n\t\t(whispering)\nHello!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("JACK"), blank_attributes()),
        Element::Parenthetical(p("(whispering)"), blank_attributes()),
        Element::Dialogue(p("Hello!"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from parentheticals"
    );
}

#[test]
fn it_trims_leading_whitespace_from_dialogue() {
    // From Brick-n-Steel.fountain: "\tThen let's retire them."
    let text = "DAN\n\tThen let's retire them.";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DAN"), blank_attributes()),
        Element::Dialogue(p("Then let's retire them."), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from dialogue"
    );
}

#[test]
fn it_trims_leading_whitespace_from_dialogue_with_markup() {
    // From Brick-n-Steel.fountain: "\t_Permanently_."
    let text = "DAN\n\t_Permanently_.";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DAN"), blank_attributes()),
        Element::Dialogue(
            Styled(vec![tr("Permanently", vec!["Underline"]), tr(".", vec![])]),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from dialogue with markup"
    );
}

#[test]
fn it_trims_leading_whitespace_from_transitions() {
    // From Brick-n-Steel.fountain: "\t\t\t\tCUT TO:"
    let text = "\n\t\t\t\tCUT TO:\n";
    let expected = vec![Element::Transition(p("CUT TO:"), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from transitions"
    );
}

#[test]
fn it_trims_leading_whitespace_from_multi_line_dialogue() {
    // Dialogue continuation lines with leading whitespace should be trimmed
    let text = "DAN\n\tThen let's retire them.\n\t_Permanently_.";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DAN"), blank_attributes()),
        Element::Dialogue(
            Styled(vec![
                tr("Then let's retire them.\n", vec![]),
                tr("Permanently", vec!["Underline"]),
                tr(".", vec![]),
            ]),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should trim leading tabs/spaces from multi-line dialogue"
    );
}

#[test]
fn it_handles_brick_n_steel_indented_dialogue_block() {
    // Full JACK dialogue block from Brick-n-Steel.fountain (lines 57-59)
    let text =
        "\t\t\tJACK\n\t\t(in Vietnamese, subtitled)\n\t*Did you know Brick and Steel are retired?*";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("JACK"), blank_attributes()),
        Element::Parenthetical(p("(in Vietnamese, subtitled)"), blank_attributes()),
        Element::Dialogue(
            Styled(vec![tr(
                "Did you know Brick and Steel are retired?",
                vec!["Italic"],
            )]),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle the full indented dialogue block from Brick-n-Steel"
    );
}
