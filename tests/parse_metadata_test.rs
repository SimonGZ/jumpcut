use jumpcut::{blank_attributes, p, parse, tr, Element, ElementText::Styled, Screenplay};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn it_handles_complex_metadata_without_elements() {
    let mut text = "Title:\n    _**BRICK & STEEL**_\n    _**FULL RETIRED**_\nCredit: Written by\nAuthor: Stu Maschwitz\nSource: Story by KTM\nDraft date: 1/20/2012\nContact:\n    Next Level Productions\n    1588 Mission Dr.\n    Solvang, CA 93463";
    let mut expected_metadata = HashMap::new();
    expected_metadata.insert(
        "title".to_string(),
        vec![
            Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
            Styled(vec![tr("FULL RETIRED", vec!["Bold", "Underline"])]),
        ],
    );
    expected_metadata.insert("credit".to_string(), vec!["Written by".into()]);
    expected_metadata.insert("author".to_string(), vec!["Stu Maschwitz".into()]);
    expected_metadata.insert("source".to_string(), vec!["Story by KTM".into()]);
    expected_metadata.insert("draft date".to_string(), vec!["1/20/2012".into()]);
    expected_metadata.insert(
        "contact".to_string(),
        vec![
            "Next Level Productions".into(),
            "1588 Mission Dr.".into(),
            "Solvang, CA 93463".into(),
        ],
    );
    let mut expected = Screenplay {
        elements: vec![],
        metadata: expected_metadata,
    };
    assert_eq! {
        parse(text),
        expected,
        "it should handle complex metadata with no elements (1)"
    }

    text = "Title:  **THE LAST BIRTHDAY CARD**\nCredit: Written by\nAuthor: Stu Maschwitz\nDraft date: 7/8/1998\nContact:\n PO Box 10031\n San Rafael CA 94912\n Registered WGAw No. 701428";
    expected_metadata = HashMap::new();
    expected_metadata.insert(
        "title".to_string(),
        vec![Styled(vec![tr("THE LAST BIRTHDAY CARD", vec!["Bold"])])],
    );
    expected_metadata.insert("credit".to_string(), vec!["Written by".into()]);
    expected_metadata.insert("author".to_string(), vec!["Stu Maschwitz".into()]);
    expected_metadata.insert("draft date".to_string(), vec!["7/8/1998".into()]);
    expected_metadata.insert(
        "contact".to_string(),
        vec![
            "PO Box 10031".into(),
            "San Rafael CA 94912".into(),
            "Registered WGAw No. 701428".into(),
        ],
    );
    expected = Screenplay {
        elements: vec![],
        metadata: expected_metadata,
    };

    assert_eq! {
        parse(text),
        expected,
        "it should handle complex metadata with no elements (2)"
    }
}

#[test]
fn it_handles_complex_metadata_with_elements() {
    let text = "Title:\n    _**BRICK & STEEL**_\n    _**FULL RETIRED**_\nCredit: Written by\nAuthor: Stu Maschwitz\nSource: Story by KTM\nDraft date: 1/20/2012\nContact:\n    Next Level Productions\n    1588 Mission Dr.\n    Solvang, CA 93463\n\nINT. THE ZOO";
    let mut expected_metadata = HashMap::new();
    expected_metadata.insert(
        "title".to_string(),
        vec![
            Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
            Styled(vec![tr("FULL RETIRED", vec!["Bold", "Underline"])]),
        ],
    );
    expected_metadata.insert("credit".to_string(), vec!["Written by".into()]);
    expected_metadata.insert("author".to_string(), vec!["Stu Maschwitz".into()]);
    expected_metadata.insert("source".to_string(), vec!["Story by KTM".into()]);
    expected_metadata.insert("draft date".to_string(), vec!["1/20/2012".into()]);
    expected_metadata.insert(
        "contact".to_string(),
        vec![
            "Next Level Productions".into(),
            "1588 Mission Dr.".into(),
            "Solvang, CA 93463".into(),
        ],
    );
    let expected = Screenplay {
        elements: vec![Element::SceneHeading(p("INT. THE ZOO"), blank_attributes())],
        metadata: expected_metadata,
    };

    assert_eq! {
        parse(text),
        expected,
        "it should handle complex metadata with elements"
    }
}

#[test]
fn it_handles_unusual_metadata() {
    let text = "format: scd\nrevision color: blue";
    let mut expected_metadata = HashMap::new();
    expected_metadata.insert("format".to_string(), vec!["scd".into()]);
    expected_metadata.insert("revision color".to_string(), vec!["blue".into()]);

    let expected = Screenplay {
        elements: vec![],
        metadata: expected_metadata,
    };

    assert_eq! {
        parse(text),
        expected,
        "it should handle unusual metadata"
    }
}
