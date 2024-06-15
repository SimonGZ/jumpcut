use jumpcut::{blank_attributes, p, parse, Attributes, Element};
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

#[test]
fn it_handles_page_breaks_with_extra_equal_signs() {
    let text = "Marcus listens intently.\n\n=====\n\nIt was a lie. It was all a lie.";
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
        "it should handle a page break with extra equal signs"
    );
}

#[test]
fn it_handles_page_breaks_following_dialogue() {
    let text = "MARCUS\nWhat the eff?\n\n===\n\nINT. CAFE - DAY";
    let expected = vec![
        Element::DialogueBlock(vec![
            Element::Character(p("MARCUS"), blank_attributes()),
            Element::Dialogue(p("What the eff?"), blank_attributes()),
        ]),
        Element::SceneHeading(
            p("INT. CAFE - DAY"),
            Attributes {
                starts_new_page: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a page break following a dialogue"
    );
}

#[test]
fn it_handles_page_breaks_following_centered_text() {
    let text = "> END ACT TWO <\n\n===\n\n> ACT THREE <";
    let expected = vec![
        Element::EndOfAct(
            p("END ACT TWO"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::NewAct(
            p("ACT THREE"),
            Attributes {
                starts_new_page: true,
                centered: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a page break following centered text"
    );
}

/*
        it('should work after centered text', function() {
            let fountain = "> END ACT TWO <\n\n===\n\n> ACT THREE <";
            let expected = [
                {elementType: 'endOfAct', content: "END ACT TWO"},
                {elementType: 'newAct', attributes: {startsNewPage: true}, content: "ACT THREE"}
            ];
            let actual = parser.parse(fountain).elements;
            assert.deepEqual(actual, expected);
        });
*/
