use jumpcut::{blank_attributes, p, parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_empty_action() {
    let text = "";
    let expected = vec![Element::Action(p(""), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle an empty string"
    );
}

#[test]
fn it_handles_basic_action() {
    let text = "John drives the car.";
    let expected = vec![Element::Action(
        p("John drives the car."),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a simple action element"
    );
}

#[test]
fn it_handles_multiline_action() {
    let text = "\nDavid looks around the room cautiously.\nShe's gone. He heads for the drawer, tip-toeing.\nThis is it. The moment he's been waiting for.";
    let expected = vec![Element::Action(p("David looks around the room cautiously.\nShe's gone. He heads for the drawer, tip-toeing.\nThis is it. The moment he's been waiting for."), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle multi-line action"
    );
}

#[test]
fn it_handles_shane_black() {
    let text = "Murtaugh, springing hell bent for leather -- and folks, grab your hats ... because just then, a BELL COBRA HELICOPTER crests the edge of the bluff.\n\nAn explosion of sound...\nAs it rises like an avenging angel ...\nHovers, shattering the air with turbo-throb, sandblasting the hillside with a roto-wash of loose dirt, tables, chairs, everything that's not nailed down ...\n\nScreaming, chaos, frenzy.\nThree words that apply to this scene.";
    let expected = vec![Element::Action(p("Murtaugh, springing hell bent for leather -- and folks, grab your hats ... because just then, a BELL COBRA HELICOPTER crests the edge of the bluff."), blank_attributes()), Element::Action(p("An explosion of sound...\nAs it rises like an avenging angel ...\nHovers, shattering the air with turbo-throb, sandblasting the hillside with a roto-wash of loose dirt, tables, chairs, everything that's not nailed down ..."), blank_attributes()), Element::Action(p("Screaming, chaos, frenzy.\nThree words that apply to this scene."), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle Shane Black"
    );
}

#[test]
fn it_handles_forced_action() {
    let text = "THE DEALER eyes the new player warily.\n\n!SCANNING THE AISLES...\nWhere is that pit boss?\n\nNo luck. He has no choice to deal the cards.";
    let expected = vec![
        Element::Action(
            p("THE DEALER eyes the new player warily."),
            blank_attributes(),
        ),
        Element::Action(
            p("SCANNING THE AISLES...\nWhere is that pit boss?"),
            blank_attributes(),
        ),
        Element::Action(
            p("No luck. He has no choice to deal the cards."),
            blank_attributes(),
        ),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced action"
    );
}

#[test]
fn it_retains_vertical_space() {
    let text = "John examines the gun.\n\n\n\n\n\n\n\n\n\nBANG!";
    let expected = vec![
        Element::Action(p("John examines the gun."), blank_attributes()),
        Element::Action(p(""), blank_attributes()),
        Element::Action(p(""), blank_attributes()),
        Element::Action(p(""), blank_attributes()),
        Element::Action(p(""), blank_attributes()),
        Element::Action(p("BANG!"), blank_attributes()),
    ];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should retain verticle space"
    );
}

#[test]
fn it_retains_horizontal_space() {
    let text =
        "          Jacob Billups\n          Palace Hotel, RM 412\n          1:00 pm tomorrow";
    let expected = vec![Element::Action(p(text), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should retain horizontal space"
    );
}

#[test]
fn it_retains_horizontal_space_on_single_line() {
    let text = "          Jacob Billups";
    let expected = vec![Element::Action(p(text), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should retain horizontal space on a single line"
    );
}

// Not sure this is necessary so I'm commenting it out for now:
// #[test]
// fn it_should_convert_tab_to_spaces() {
//     let text = "	Marty McFly eats shoes.";
//     let expected = vec![Element::Action("    Marty McFly eats shoes.".to_string())];

//     assert_eq!(
//         parse(text).elements,
//         expected,
//         "it should convert a tab to four spaces"
//     );
// }

#[test]
fn it_should_handle_centered_text() {
    let text = "> DUMBO <";
    let expected = vec![Element::Action(
        p("DUMBO"),
        Attributes {
            centered: true,
            ..Attributes::default()
        },
    )];
    // NEED to handle attributes
    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle centered text"
    );
}

#[test]
fn it_should_handle_centered_text_with_no_spaces() {
    let text = ">THE END<";
    let expected = vec![Element::Action(
        p("THE END"),
        Attributes {
            centered: true,
            ..Attributes::default()
        },
    )];
    // NEED to handle attributes
    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle centered text with no spaces"
    );
}

#[test]
fn it_should_handle_multi_line_centered_text() {
    let text = "> MISSION CRITICAL <\n>SUMMON JAMES<";
    let expected = vec![Element::Action(
        p("MISSION CRITICAL\nSUMMON JAMES"),
        Attributes {
            centered: true,
            ..Attributes::default()
        },
    )];
    // NEED to handle attributes
    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle multiline centered text"
    );
}
