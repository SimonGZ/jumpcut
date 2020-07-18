use fountain_converter::{blank_attributes, parse, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn it_handles_single_line_lyric() {
    let text = "~Willy Wonka! Willy Wonka!";
    let expected = vec![Element::Lyric(
        "Willy Wonka! Willy Wonka!".to_string(),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle a single line lyric"
    );
}

#[test]
fn it_handles_multiple_line_lyric() {
    let text = "~Willy Wonka! Willy Wonka!\n~Loves Chocolate!";
    let expected = vec![Element::Lyric(
        "Willy Wonka! Willy Wonka!\nLoves Chocolate!".to_string(),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle multiple line lyrics"
    );
}
