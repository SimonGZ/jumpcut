use jumpcut::{blank_attributes, p, parse, Attributes, Element};
use pretty_assertions::assert_eq;

#[test]
fn it_handles_typical_scene_headings() {
    let headings: [&str; 5] = [
        "INT. OBSERVATORY - NIGHT",
        "\next. observatory - day",
        "int/ext car - morning",
        "i/e carmel - dusk",
        "EXT/INT.  SWAMP SHACK - DAY",
    ];
    let expecteds: Vec<Vec<Element>> = headings
        .iter()
        .map(|text| vec![Element::SceneHeading(p(text.trim()), blank_attributes())])
        .collect();
    for (i, text) in headings.iter().enumerate() {
        assert_eq!(
            parse(text).elements,
            expecteds[i],
            "it should handle scene headings"
        );
    }
}

#[test]
fn it_should_not_convert_other_int_words() {
    let text = "INTERCUT HOUSE / BARN";
    let expected = vec![Element::Action(p(text), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should not convert words beginning with int into scene headings"
    );
}

#[test]
fn it_handles_forced_scene_headings() {
    let text = ".inside the school bus";
    let expected = vec![Element::SceneHeading(
        p("inside the school bus"),
        blank_attributes(),
    )];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should handle forced scene headings"
    );
}

#[test]
fn it_should_not_turn_leading_ellipses_into_scene_headings() {
    let text = "...and lowers his guns.";
    let expected = vec![Element::Action(p(text), blank_attributes())];

    assert_eq!(
        parse(text).elements,
        expected,
        "it should not turn leading ellipses into scene headings"
    );
}

#[test]
fn it_handles_scene_headings_with_scene_numbers() {
    let headings: [&str; 8] = [
        "INT. HOUSE - DAY #1#",
        "INT. HOUSE - DAY #1A#",
        "INT. HOUSE - DAY #1a#",
        "INT. HOUSE - DAY #A1#",
        "INT. HOUSE - DAY #I-1-A#",
        "INT. HOUSE - DAY #1.#",
        "INT. HOUSE - DAY - FLASHBACK (1944) #110A#",
        ".INSIDE THE BUS #12#",
    ];
    let expecteds: Vec<Vec<Element>> = vec![
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("1".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("1A".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("1a".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("A1".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("I-1-A".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY"),
            Attributes {
                scene_number: Some("1.".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INT. HOUSE - DAY - FLASHBACK (1944)"),
            Attributes {
                scene_number: Some("110A".to_string()),
                ..Attributes::default()
            },
        )],
        vec![Element::SceneHeading(
            p("INSIDE THE BUS"),
            Attributes {
                scene_number: Some("12".to_string()),
                ..Attributes::default()
            },
        )],
    ];
    for (i, text) in headings.iter().enumerate() {
        assert_eq!(
            parse(text).elements,
            expecteds[i],
            "it should handle scene headings with scene numbers"
        );
    }
}
