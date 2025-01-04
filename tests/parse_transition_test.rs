use jumpcut::{blank_attributes, p, parse, Element};
use pretty_assertions::assert_eq;

#[test]
fn it_handles_typical_transitions() {
    let transitions: [&str; 4] = ["\nCUT TO:\n", "\nSMASH TO:", "\nfade to:", "abrupt Cut to:"];
    let expecteds = vec![
        vec![Element::Transition(p("CUT TO:"), blank_attributes())],
        vec![Element::Transition(p("SMASH TO:"), blank_attributes())],
        vec![Element::Transition(p("fade to:"), blank_attributes())],
        vec![Element::Transition(p("abrupt Cut to:"), blank_attributes())],
    ];
    for (i, text) in transitions.iter().enumerate() {
        assert_eq!(
            parse(text).elements,
            expecteds[i],
            "it should handle typical transitions"
        );
    }
}

#[test]
fn it_doesnt_turn_words_ending_in_to_into_transitions() {
    // Note: Have to put scene headings here because if first line of script
    // ends in colon, it'll be interpreted as key:value metadata. If a script
    // needs to begin with a line ending in a transition then the line should be
    // forced.
    let fake_transitions: [&str; 2] = [
        "\nINT. HOUSE - DAY\n\nalto:\n",
        "\nINT. HOUSE - DAY\n\nonto:",
    ];
    let expecteds = vec![
        vec![
            Element::SceneHeading(p("INT. HOUSE - DAY"), blank_attributes()),
            Element::Action(p("alto:"), blank_attributes()),
        ],
        vec![
            Element::SceneHeading(p("INT. HOUSE - DAY"), blank_attributes()),
            Element::Action(p("onto:"), blank_attributes()),
        ],
    ];
    dbg!(parse("\n\nalto:\n"));
    for (i, text) in fake_transitions.iter().enumerate() {
        assert_eq!(
            parse(text).elements,
            expecteds[i],
            "it should not turn every word ending in to: into a transition"
        );
    }
}

#[test]
fn it_handles_forced_transitions() {
    let text = "> Fade to black.";
    let expected = vec![Element::Transition(p("Fade to black."), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced transitions"
    );
}
