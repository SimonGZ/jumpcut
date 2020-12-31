use jumpcut::{blank_attributes, p, parse, Element, ElementText::Styled, TextRun};
#[cfg(test)]
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
fn it_handles_basic_dialogue() {
    let text = "\nDAVID\nAnd just what does that mean?\n";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DAVID"), blank_attributes()),
        Element::Dialogue(p("And just what does that mean?"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle basic dialogue"
    );
}

#[test]
fn it_handles_multiple_parentheticals() {
    let text = "\nDAVID\n(prissy)\nAnd just what does that mean?\n(sniffing)\nUgh. Pooh!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DAVID"), blank_attributes()),
        Element::Parenthetical(p("(prissy)"), blank_attributes()),
        Element::Dialogue(p("And just what does that mean?"), blank_attributes()),
        Element::Parenthetical(p("(sniffing)"), blank_attributes()),
        Element::Dialogue(p("Ugh. Pooh!"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle dialogue with multiple parentheticals"
    );
}

#[test]
fn it_handles_dialogue_with_line_breaks() {
    let text = "DAN\nThen let's retire them.\n_Permanently_.";
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
        "it should handle dialogue with line breaks"
    );
}

#[test]
fn it_handles_dialogue_with_multiple_line_breaks() {
    let text = "DEALER\nTen.\nFour.\nDealer gets a seven.  Hit or stand sir?\n\nMONKEY\nDude, I'm a monkey.";
    let expected = vec![
        Element::DialogueBlock(vec![
            Element::Character(p("DEALER"), blank_attributes()),
            Element::Dialogue(
                p("Ten.\nFour.\nDealer gets a seven.  Hit or stand sir?"),
                blank_attributes(),
            ),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("MONKEY"), blank_attributes()),
            Element::Dialogue(p("Dude, I'm a monkey."), blank_attributes()),
        ]),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced character names"
    );
}

#[test]
fn it_handles_dialogue_with_forced_blank_lines() {
    let text = "DEALER\nTen.\nFour.\nDealer gets a seven.\n  \nHit or stand sir?";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("DEALER"), blank_attributes()),
        Element::Dialogue(
            p("Ten.\nFour.\nDealer gets a seven.\n  \nHit or stand sir?"),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced character names"
    );
}

#[test]
fn it_ignores_a_single_space_when_forcing_blank_lines() {
    let text = "DEALER\nTen.\nFour.\nDealer gets a seven.\n \nHit or stand sir?";
    let expected = vec![
        Element::DialogueBlock(vec![
            Element::Character(p("DEALER"), blank_attributes()),
            Element::Dialogue(p("Ten.\nFour.\nDealer gets a seven."), blank_attributes()),
        ]),
        Element::Action(p("Hit or stand sir?"), blank_attributes()),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should not turn a blank line with a single extraneous space into extended dialogue"
    );
}

#[test]
fn it_handles_forced_character_names() {
    let text = "\n@McGregor\nWhat the fuck!?";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("McGregor"), blank_attributes()),
        Element::Dialogue(p("What the fuck!?"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced character names"
    );
}
