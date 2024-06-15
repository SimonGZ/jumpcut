use jumpcut::{blank_attributes, p, parse, Element};
use pretty_assertions::assert_eq;

#[test]
fn it_handles_single_line_lyric() {
    let text = "~Willy Wonka! Willy Wonka!";
    let expected = vec![Element::Lyric(
        p("Willy Wonka! Willy Wonka!"),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle a single line lyric"
    );
}

#[test]
fn it_handles_multiple_line_lyric() {
    let text = "~Willy Wonka! Willy Wonka!\n~Loves Chocolate!";
    let expected = vec![Element::Lyric(
        p("Willy Wonka! Willy Wonka!\nLoves Chocolate!"),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle multiple line lyrics"
    );
}

#[test]
fn it_handles_longer_multiple_line_lyric() {
    let text = "SINGER\n~Some holidays we ring in wholeheartedly:\n~Mothers’ Day, Fathers’ Day, or Martin Luther King.\n~But some we celebrate a bit more guardedly.\n~Their history\n~Turns out to be\n~A complex issue, racially.\n~So thoughtfully we sinnnnnng...";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("SINGER"), blank_attributes()),
        Element::Lyric(
            p("Some holidays we ring in wholeheartedly:\nMothers’ Day, Fathers’ Day, or Martin Luther King.\nBut some we celebrate a bit more guardedly.\nTheir history\nTurns out to be\nA complex issue, racially.\nSo thoughtfully we sinnnnnng..."),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle multiple line lyrics"
    );
}

#[test]
fn it_handles_dialogue_block_with_lyrics() {
    let text = "SINGER\n~Willy Wonka! Willy Wonka!\n~Loves Chocolate!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("SINGER"), blank_attributes()),
        Element::Lyric(
            p("Willy Wonka! Willy Wonka!\nLoves Chocolate!"),
            blank_attributes(),
        ),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should dialogue with multiple line lyrics in a row"
    );
}

#[test]
fn it_handles_dialogue_block_with_mixed_dialogue_and_lyrics() {
    let text = "SINGER\nHow does this sound?\n~Loves Chocolate!";
    let expected = vec![Element::DialogueBlock(vec![
        Element::Character(p("SINGER"), blank_attributes()),
        Element::Dialogue(p("How does this sound?"), blank_attributes()),
        Element::Lyric(p("Loves Chocolate!"), blank_attributes()),
    ])];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle dialogue with mixed dialogue and lyrics"
    );
}
