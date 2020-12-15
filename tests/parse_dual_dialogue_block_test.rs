use jumpcut::{blank_attributes, p, parse, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_basic_dual_dialogue() {
    let text = "BRICK\nScrew retirement.\n\nSTEEL ^\nScrew retirement.";
    let expected = vec![Element::DualDialogueBlock(vec![
        Element::DialogueBlock(vec![
            Element::Character(p("BRICK"), blank_attributes()),
            Element::Dialogue(p("Screw retirement."), blank_attributes()),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("STEEL"), blank_attributes()),
            Element::Dialogue(p("Screw retirement."), blank_attributes()),
        ]),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle basic dual dialogue"
    );
}
