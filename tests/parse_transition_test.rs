use fountain_converter::{parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}

#[test]
fn it_handles_typical_transitions() {
    let transitions: [&str; 4] = ["\nCUT TO:\n", "\nSMASH TO:", "\nfade to:", "abrupt Cut to:"];
    let expecteds = vec![
        vec![Element::Transition(
            "CUT TO:".to_string(),
            blank_attributes(),
        )],
        vec![Element::Transition(
            "SMASH TO:".to_string(),
            blank_attributes(),
        )],
        vec![Element::Transition(
            "fade to:".to_string(),
            blank_attributes(),
        )],
        vec![Element::Transition(
            "abrupt Cut to:".to_string(),
            blank_attributes(),
        )],
    ];
    for (i, text) in transitions.iter().enumerate() {
        assert_eq!(
            parse(text),
            expecteds[i],
            "it should handle typical transitions"
        );
    }
}

#[test]
fn it_handles_forced_transitions() {
    let text = "> Fade to black.";
    let expected = vec![Element::Transition(
        "Fade to black.".to_string(),
        blank_attributes(),
    )];

    assert_eq!(parse(text), expected, "it should handle forced transitions");
}
