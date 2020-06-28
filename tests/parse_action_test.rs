use fountain_converter::{parse, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_shane_black() {
    let hunks = "Murtaugh, springing hell bent for leather -- and folks, grab your hats ... because just then, a BELL COBRA HELICOPTER crests the edge of the bluff.\n\nAn explosion of sound...\nAs it rises like an avenging angel ...\nHovers, shattering the air with turbo-throb, sandblasting the hillside with a roto-wash of loose dirt, tables, chairs, everything that's not nailed down ...\n\nScreaming, chaos, frenzy.\nThree words that apply to this scene.";
    let expected = vec![Element::Action("Murtaugh, springing hell bent for leather -- and folks, grab your hats ... because just then, a BELL COBRA HELICOPTER crests the edge of the bluff.".to_string()), Element::Action("An explosion of sound...\nAs it rises like an avenging angel ...\nHovers, shattering the air with turbo-throb, sandblasting the hillside with a roto-wash of loose dirt, tables, chairs, everything that's not nailed down ...".to_string()), Element::Action("Screaming, chaos, frenzy.\nThree words that apply to this scene.".to_string())];

    assert_eq!(parse(hunks), expected, "it should handle Shane Black");
}
