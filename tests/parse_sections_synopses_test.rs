use jumpcut::{blank_attributes, p, parse, Element};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_creates_sections() {
    let text = "# Act 1\n## John finds the horse\n\nJOHN\nWhoa!";
    let expected = vec![
        Element::Section(p("Act 1"), blank_attributes(), 1),
        Element::Section(p("John finds the horse"), blank_attributes(), 2),
        Element::DialogueBlock(vec![
            Element::Character(p("JOHN"), blank_attributes()),
            Element::Dialogue(p("Whoa!"), blank_attributes()),
        ]),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle basic sections"
    );
}

#[test]
fn it_handles_isolated_synopsis() {
    let text = "= John and Henry find the locomotive.";
    let expected = vec![Element::Synopsis(p("John and Henry find the locomotive."))];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle an isolated synopsis"
    );
}
