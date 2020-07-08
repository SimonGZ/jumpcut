use fountain_converter::{parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}

#[test]
fn it_handles_basic_dialogue() {
    let text = "\nDAVID\nAnd just what does that mean?\n";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character("DAVID".to_string(), blank_attributes()),
        Element::Dialogue(
            "And just what does that mean?".to_string(),
            blank_attributes(),
        ),
    ])];

    assert_eq!(parse(text), expected, "it should handle basic dialogue");
}

#[test]
fn it_handles_multiple_parentheticals() {
    let text = "\nDAVID\n(prissy)\nAnd just what does that mean?\n(sniffing)\nUgh. Pooh!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character("DAVID".to_string(), blank_attributes()),
        Element::Parenthetical("(prissy)".to_string(), blank_attributes()),
        Element::Dialogue(
            "And just what does that mean?".to_string(),
            blank_attributes(),
        ),
        Element::Parenthetical("(sniffing)".to_string(), blank_attributes()),
        Element::Dialogue("Ugh. Pooh!".to_string(), blank_attributes()),
    ])];

    assert_eq!(
        parse(text),
        expected,
        "it should handle dialogue with multiple parentheticals"
    );
}

#[test]
fn it_handles_dialogue_with_line_breaks() {
    let text = "DAN\nThen let's retire them.\n_Permanently_.";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character("DAN".to_string(), blank_attributes()),
        Element::Dialogue(
            "Then let's retire them.\n_Permanently_.".to_string(),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text),
        expected,
        "it should handle dialogue with line breaks"
    );
}

#[test]
fn it_handles_forced_character_names() {
    let text = "\n@McGregor\nWhat the fuck!?";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character("McGregor".to_string(), blank_attributes()),
        Element::Dialogue("What the fuck!?".to_string(), blank_attributes()),
    ])];

    assert_eq!(
        parse(text),
        expected,
        "it should handle forced character names"
    );
}
