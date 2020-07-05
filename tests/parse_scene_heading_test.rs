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
