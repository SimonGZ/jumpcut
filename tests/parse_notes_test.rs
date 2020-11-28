use fountain_converter::{blank_attributes, parse, Attributes, Element};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_single_line_interspersed_notes() {
    let text = "Jack smells the liquor. [[Or should he taste it?]] Not good. [[Or bad?]]";
    let expected = vec![Element::Action(
        "Jack smells the liquor.  Not good. ".to_string(),
        Attributes {
            notes: Some(vec![
                "Or should he taste it?".to_string(),
                "Or bad?".to_string(),
            ]),
            ..Attributes::default()
        },
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle single-line interspersed notes"
    );
}

#[test]
fn it_handles_multi_line_interspersed_notes() {
    let text = "His hand is an inch from the receiver when the phone RINGS.  Scott pauses for a moment, suspicious for some reason.[[This section needs work.\nEither that, or I need coffee.\n  \nDefinitely coffee.]] He looks around.  Phone ringing.";
    let expected = vec![Element::Action(
        "His hand is an inch from the receiver when the phone RINGS.  Scott pauses for a moment, suspicious for some reason. He looks around.  Phone ringing.".to_string(),
        Attributes {
            notes: Some(vec![
                "This section needs work.\nEither that, or I need coffee.\n  \nDefinitely coffee.".to_string(),
            ]),
            ..Attributes::default()
        },
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle multi-line interspersed notes"
    );
}

#[test]
fn it_handles_notes_on_dialogue() {
    // NOTE: I've decided to NOT include the ability to place notes on character names.
    // It adds hundreds of checks on a fountain document and it's a dumb place to put a note.
    // I am also NOT including the ability to place notes dialogue with forced line breaks.
    let text = "JAMIE\nWhat's the meaning of this [[that]]shit?";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character("JAMIE".to_string(), blank_attributes()),
        Element::Dialogue(
            "What's the meaning of this shit?".to_string(),
            Attributes {
                notes: Some(vec!["that".to_string()]),
                ..Attributes::default()
            },
        ),
    ])];

    assert_eq!(
        parse(text),
        expected,
        "it should handle basic dialogue blocks with notes"
    );
}

#[test]
fn it_handles_an_empty_line_note() {
    let text = "[[Dogs?]]";
    let expected = vec![Element::Action(
        "".to_string(),
        Attributes {
            notes: Some(vec!["Dogs?".to_string()]),
            ..Attributes::default()
        },
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle a note on a single blank line"
    );
}
