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
    let expected = vec![Element::DialogueBlock(Box::new(vec![
        Element::Character("DAVID".to_string(), blank_attributes()),
        Element::Dialogue(
            "And just what does that mean?".to_string(),
            blank_attributes(),
        ),
    ]))];

    assert_eq!(
        parse(text),
        expected,
        "it should not convert words beginning with int into scene headings"
    );
}
