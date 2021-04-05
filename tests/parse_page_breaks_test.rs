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
        "it should handle action with a page break"
    );
}

/*
        it('should work with more than three equal signs', function() {
            let fountain = "Marcus listens intently.\n\n====\n\nIt was a lie. It was all a lie.";
            let expected = [
                {elementType: 'action', content: "Marcus listens intently."},
                {elementType: 'action', attributes: {startsNewPage: true}, content: "It was a lie. It was all a lie."}
            ];
            let actual = parser.parse(fountain).elements;
            assert.deepEqual(actual, expected);
        });
        it('should work with a dialogue block', function() {
            let fountain = "MARCUS\nWhat the eff?\n\n===\n\nINT. CAFE - DAY";
            let expected = [
                {elementType: 'dialogueBlock', content: [
                    {elementType: 'character', content: "MARCUS"},
                    {elementType: 'dialogue', content: "What the eff?"}
                ]},
                {elementType: 'sceneHeading', attributes: {startsNewPage: true}, content: "INT. CAFE - DAY"}
            ];
            let actual = parser.parse(fountain).elements;
            assert.deepEqual(actual, expected);
        });
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
