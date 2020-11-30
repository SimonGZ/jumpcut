use fountain_converter::{parse, Screenplay};
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn it_handles_complex_metadata_without_notes() {
    let text = "Title:\n    _**BRICK & STEEL**_\n    _**FULL RETIRED**_\nCredit: Written by\nAuthor: Stu Maschwitz\nSource: Story by KTM\nDraft date: 1/20/2012\nContact:\n    Next Level Productions\n    1588 Mission Dr.\n    Solvang, CA 93463";
    let mut expected_metadata = HashMap::new();
    expected_metadata.insert(
        "title".to_string(),
        vec![
            "_**BRICK & STEEL**_".to_string(),
            "_**FULL RETIRED**_".to_string(),
        ],
    );
    expected_metadata.insert("credit".to_string(), vec!["Written by".to_string()]);
    expected_metadata.insert("author".to_string(), vec!["Stu Maschwitz".to_string()]);
    expected_metadata.insert("source".to_string(), vec!["Story by KTM".to_string()]);
    expected_metadata.insert("draft date".to_string(), vec!["1/20/2012".to_string()]);
    expected_metadata.insert(
        "contact".to_string(),
        vec![
            "Next Level Productions".to_string(),
            "1588 Mission Dr.".to_string(),
            "Solvang, CA 93463".to_string(),
        ],
    );
    let expected = Screenplay {
        elements: vec![],
        metadata: expected_metadata,
    };

    assert_eq! {
        parse(text),
        expected,
        "it should handle complex metadata with no elements"
    }
}
