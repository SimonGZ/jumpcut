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
fn it_handles_forced_transitions() {
    let text = "> Fade to black.";
    let expected = vec![Element::Transition(p("Fade to black."), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced transitions"
    );
}
