use jumpcut::{blank_attributes, p, parse, Element};
#[cfg(test)]
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
