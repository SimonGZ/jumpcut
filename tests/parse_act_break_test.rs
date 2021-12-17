use jumpcut::{p, parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_new_act() {
    let text = "> ACT ONE <";
    let expected = vec![Element::NewAct(
        p("ACT ONE"),
        Attributes {
            centered: true,
            ..Attributes::default()
        },
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a plain new act"
    );
}
