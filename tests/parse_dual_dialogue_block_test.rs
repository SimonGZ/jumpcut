use fountain_converter::{parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}

#[test]
fn it_handles_basic_dual_dialogue() {
    let text = "BRICK\nScrew retirement.\n\nSTEEL ^\nScrew retirement.";
    let expected = vec![Element::DualDialogueBlock(vec![
        Element::DialogueBlock(vec![
            Element::Character("BRICK".to_string(), blank_attributes()),
            Element::Dialogue("Screw retirement.".to_string(), blank_attributes()),
        ]),
        Element::DialogueBlock(vec![
            Element::Character("STEEL".to_string(), blank_attributes()),
            Element::Dialogue("Screw retirement.".to_string(), blank_attributes()),
        ]),
    ])];

    assert_eq!(
        parse(text),
        expected,
        "it should handle basic dual dialogue"
    );
}
