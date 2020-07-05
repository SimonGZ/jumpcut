use fountain_converter::{parse, Attributes, Element};
#[cfg(test)]
use pretty_assertions::assert_eq;

fn blank_attributes() -> Attributes {
    Attributes {
        centered: false,
        starts_new_page: false,
    }
}

#[test]
fn it_handles_typical_scene_headings() {
    let headings: [&str; 5] = [
        "INT. OBSERVATORY - NIGHT",
        "ext. observatory - day",
        "int/ext car - morning",
        "i/e carmel - dusk",
        "EXT/INT.  SWAMP SHACK - DAY",
    ];
    let expecteds: Vec<Vec<Element>> = headings
        .iter()
        .map(|text| vec![Element::SceneHeading(text.to_string(), blank_attributes())])
        .collect();
    for (i, text) in headings.iter().enumerate() {
        assert_eq!(parse(text), expecteds[i], "it should handle scene headings");
    }
}

#[test]
fn it_should_not_convert_other_int_words() {
    let text = "INTERCUT HOUSE / BARN";
    let expected = vec![Element::Action(text.to_string(), blank_attributes())];

    assert_eq!(
        parse(text),
        expected,
        "it should not convert words beginning with int into scene headings"
    );
}

#[test]
fn it_should_forced_scene_headings() {
    let text = ".inside the school bus";
    let expected = vec![Element::SceneHeading(
        "inside the school bus".to_string(),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text),
        expected,
        "it should handle forced scene headings"
    );
}
