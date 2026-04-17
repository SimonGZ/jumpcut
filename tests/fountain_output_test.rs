use jumpcut::{
    blank_attributes, p, parse, parse_fdx, tr, Attributes, Element, ElementText::Styled, Metadata,
    Screenplay,
};
use pretty_assertions::assert_eq;

#[test]
fn fountain_output_round_trips_metadata_and_core_body_elements() {
    let mut metadata = Metadata::new();
    metadata.insert(
        "title".into(),
        vec![
            Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
            "FULL RETIRED".into(),
        ],
    );
    metadata.insert("credit".into(), vec!["Written by".into()]);
    metadata.insert("author".into(), vec!["Stu Maschwitz".into()]);
    metadata.insert("fmt".into(), vec!["balanced allow-lowercase-title".into()]);

    let screenplay = Screenplay {
        metadata,
        imported_layout: None,
        imported_title_page: None,
        elements: vec![
            Element::SceneHeading(
                p("INT. HOUSE - DAY"),
                Attributes {
                    scene_number: Some("12".into()),
                    ..Default::default()
                },
            ),
            Element::Action(p("John enters."), blank_attributes()),
            Element::DialogueBlock(vec![
                Element::Character(p("BRICK"), blank_attributes()),
                Element::Parenthetical(p("(quietly)"), blank_attributes()),
                Element::Dialogue(
                    Styled(vec![tr("Hello", vec!["Italic"]), tr(".", vec![])]),
                    blank_attributes(),
                ),
            ]),
            Element::Transition(p("CUT TO:"), blank_attributes()),
        ],
    };

    let rendered = screenplay.to_fountain();
    let reparsed = parse(&rendered);

    assert_eq!(reparsed.elements, screenplay.elements);
    assert_eq!(
        reparsed.metadata.get("title"),
        Some(&vec!["BRICK & STEEL".into(), "FULL RETIRED".into()])
    );
    for key in ["credit", "author", "fmt"] {
        assert_eq!(reparsed.metadata.get(key), screenplay.metadata.get(key));
    }
}

#[test]
fn fountain_output_forces_ambiguous_elements_to_preserve_type() {
    let screenplay = Screenplay {
        metadata: Metadata::new(),
        imported_layout: None,
        imported_title_page: None,
        elements: vec![
            Element::SceneHeading(p("inside the school bus"), blank_attributes()),
            Element::Action(p("INT. HOUSE - DAY"), blank_attributes()),
            Element::DialogueBlock(vec![
                Element::Character(p("McGregor"), blank_attributes()),
                Element::Dialogue(p("What the fuck!?"), blank_attributes()),
            ]),
            Element::Transition(p("Fade to black."), blank_attributes()),
        ],
    };

    let rendered = screenplay.to_fountain();

    assert!(rendered.contains(".inside the school bus"));
    assert!(rendered.contains("!INT. HOUSE - DAY"));
    assert!(rendered.contains("@McGregor"));
    assert!(rendered.contains("> Fade to black."));
    assert_eq!(parse(&rendered).elements, screenplay.elements);
}

#[test]
fn fountain_output_round_trips_dual_dialogue_page_breaks_and_centered_markers() {
    let screenplay = Screenplay {
        metadata: Metadata::new(),
        imported_layout: None,
        imported_title_page: None,
        elements: vec![
            Element::Action(
                p("THE END"),
                Attributes {
                    centered: true,
                    ..Default::default()
                },
            ),
            Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(p("BRICK"), blank_attributes()),
                    Element::Dialogue(p("Left side."), blank_attributes()),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(p("STEEL"), blank_attributes()),
                    Element::Dialogue(p("Right side."), blank_attributes()),
                ]),
            ]),
            Element::Action(
                p("New page action."),
                Attributes {
                    starts_new_page: true,
                    ..Default::default()
                },
            ),
        ],
    };

    let rendered = screenplay.to_fountain();

    assert!(rendered.contains("> THE END <"));
    assert!(rendered.contains("STEEL ^"));
    assert!(rendered.contains("===\n\nNew page action."));
    assert_eq!(parse(&rendered).elements, screenplay.elements);
}

#[test]
fn imported_fdx_can_be_emitted_as_fountain() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/brick-n-steel-basic.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");
    let fountain = screenplay.to_fountain();
    let reparsed = parse(&fountain);

    assert_eq!(
        reparsed.metadata.get("title"),
        screenplay.metadata.get("title")
    );
    assert_eq!(reparsed.elements, screenplay.elements);
}

#[test]
fn fountain_to_fdx_to_fountain_probe_surfaces_current_lossy_title_page_edge() {
    let source = r#"Title:
    _**BRICK & STEEL**_
    FULL RETIRED
Credit: Written by
Author: Stu Maschwitz
Fmt: balanced allow-lowercase-title

.inside the school bus #12#

!INT. HOUSE - DAY

@McGregor
(quietly)
*Hello*.

BRICK
Left side.

STEEL ^
Right side.

===

> ACT ONE <

~Sing me a song
~of sixpence

> Fade to black.
"#;

    let original = parse(source);
    let mut fdx_source = parse(source);
    let fdx = fdx_source.to_final_draft();
    let from_fdx = parse_fdx(&fdx).expect("generated fdx should parse");
    let fountain = from_fdx.to_fountain();
    let reparsed = parse(&fountain);

    assert_eq!(reparsed.elements, original.elements);
    assert_ne!(
        reparsed.metadata.get("title"),
        original.metadata.get("title")
    );
    assert_ne!(reparsed.metadata, original.metadata);
}

#[test]
fn fountain_output_renders_frontmatter_metadata_with_blank_line_paragraph_separation() {
    let mut metadata = Metadata::new();
    metadata.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata.insert(
        "frontmatter".into(),
        vec![
            "WRITERS' NOTE".into(),
            "".into(),
            "First paragraph of the note.".into(),
            "".into(),
            "Second paragraph.".into(),
        ],
    );

    let screenplay = Screenplay {
        metadata,
        imported_layout: None,
        imported_title_page: None,
        elements: vec![Element::Action(p("Body."), blank_attributes())],
    };

    let rendered = screenplay.to_fountain();

    // Should contain Frontmatter key with indented content
    assert!(
        rendered.contains("Frontmatter:"),
        "should contain Frontmatter key"
    );
    assert!(
        rendered.contains("    WRITERS' NOTE"),
        "should contain indented heading"
    );
    assert!(
        rendered.contains("    First paragraph of the note."),
        "should contain indented paragraph"
    );
    // Blank lines between paragraphs should be preserved as two-space lines
    assert!(
        rendered.contains("  \n    First paragraph"),
        "should have two-space blank line separator"
    );
}

#[test]
fn fountain_frontmatter_round_trips_through_parse() {
    let mut metadata = Metadata::new();
    metadata.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata.insert(
        "frontmatter".into(),
        vec!["WRITERS' NOTE".into(), "".into(), "First paragraph.".into()],
    );

    let screenplay = Screenplay {
        metadata,
        imported_layout: None,
        imported_title_page: None,
        elements: vec![Element::Action(p("Body."), blank_attributes())],
    };

    let rendered = screenplay.to_fountain();
    let reparsed = parse(&rendered);

    let original_fm = screenplay
        .metadata
        .get("frontmatter")
        .expect("original frontmatter");
    let reparsed_fm = reparsed
        .metadata
        .get("frontmatter")
        .expect("reparsed frontmatter");
    assert_eq!(reparsed_fm.len(), original_fm.len());
    assert_eq!(reparsed_fm[0].plain_text(), "WRITERS' NOTE");
    assert_eq!(reparsed_fm[1].plain_text(), "");
    assert_eq!(reparsed_fm[2].plain_text(), "First paragraph.");
}
