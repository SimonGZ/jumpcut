use jumpcut::{p, parse, tr, Attributes, Element, ElementText};
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

#[test]
fn it_handles_cold_opening() {
    let text = "> COLD OPENING <";
    let expected = vec![Element::ColdOpening(
        p("COLD OPENING"),
        Attributes {
            centered: true,
            ..Attributes::default()
        },
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a cold opening marker"
    );
}

#[test]
fn it_handles_new_act_underlined() {
    let text = "> _ACT ONE_ <";
    let expected = vec![Element::NewAct(
        ElementText::Styled(vec![tr("ACT ONE", vec!["Underline"])]),
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

#[test]
fn it_handles_end_act() {
    let text = "> END ACT ONE <";
    let expected = vec![Element::EndOfAct(
        p("END ACT ONE"),
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

#[test]
fn later_new_acts_start_new_pages_automatically() {
    let text = "> ACT ONE <\n\n> END ACT ONE <\n\n> ACT TWO <";
    let expected = vec![
        Element::NewAct(
            p("ACT ONE"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::EndOfAct(
            p("END ACT ONE"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::NewAct(
            p("ACT TWO"),
            Attributes {
                centered: true,
                starts_new_page: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "later new acts should start a new page automatically"
    );
}

#[test]
fn first_new_act_after_cold_opening_starts_new_page() {
    let text = "> COLD OPENING <\n\n> END COLD OPEN <\n\n> ACT ONE <";
    let expected = vec![
        Element::ColdOpening(
            p("COLD OPENING"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::EndOfAct(
            p("END COLD OPEN"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::NewAct(
            p("ACT ONE"),
            Attributes {
                centered: true,
                starts_new_page: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "act one should start a new page after a cold open"
    );
}

#[test]
fn fmt_can_disable_automatic_new_act_page_starts() {
    let text = "Fmt: no-auto-act-breaks\n\n> ACT ONE <\n\n> END ACT ONE <\n\n> ACT TWO <";
    let expected = vec![
        Element::NewAct(
            p("ACT ONE"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::EndOfAct(
            p("END ACT ONE"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
        Element::NewAct(
            p("ACT TWO"),
            Attributes {
                centered: true,
                ..Attributes::default()
            },
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "fmt should be able to disable automatic new-act page starts"
    );
}
