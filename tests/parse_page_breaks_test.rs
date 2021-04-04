use jumpcut::{blank_attributes, p, parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_action_text_page_break() {
    let text = "Marcus listens intently.\n\n===\n\nIt was a lie. It was all a lie.";
    let expected = vec![
        Element::Action(p("Marcus listens intently."), blank_attributes()),
        Element::Action(
            p("It was a lie. It was all a lie."),
            Attributes {
                starts_new_page: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle action with a page break"
    );
}
