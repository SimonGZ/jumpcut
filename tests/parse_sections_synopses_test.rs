use fountain_converter::{blank_attributes, parse, Element};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_creates_sections() {
    let text = "# Act 1\n## John finds the horse\n\nJOHN\nWhoa!";
    let expected = vec![
        Element::Section("Act 1".to_string(), blank_attributes(), 1),
        Element::Section("John finds the horse".to_string(), blank_attributes(), 2),
        Element::DialogueBlock(vec![
            Element::Character("JOHN".to_string(), blank_attributes()),
            Element::Dialogue("Whoa!".to_string(), blank_attributes()),
        ]),
    ];

    assert_eq!(parse(text), expected, "it should handle basic sections");
}

#[test]
fn it_handles_isolated_synopsis() {
    let text = "= John and Henry find the locomotive.";
    let expected = vec![Element::Synopsis(
        "John and Henry find the locomotive.".to_string(),
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle an isolated synopsis"
    );
}
