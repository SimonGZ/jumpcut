use jumpcut::pagination::{Alignment, ScreenplayLayoutProfile, StyleProfile};
use jumpcut::{
    blank_attributes, p, parse_fdx, Attributes, Element, ElementText::Styled, ImportedElementKind,
    TextRun,
};
use jumpcut::title_page::{TitlePage, TitlePageBlockKind};
use pretty_assertions::assert_eq;
use std::collections::HashSet;

fn tr(content: &str, styles: Vec<&str>) -> TextRun {
    let mut style_strings: HashSet<String> = HashSet::new();
    for style in styles {
        style_strings.insert(style.to_string());
    }
    TextRun {
        content: content.to_string(),
        text_style: style_strings,
    }
}

#[test]
fn it_imports_basic_fdx_body_content_into_screenplay_elements() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/brick-n-steel-basic.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    let expected = vec![
        Element::SceneHeading(p("EXT. BRICK'S PATIO - DAY"), blank_attributes()),
        Element::Action(p("A gorgeous day."), blank_attributes()),
        Element::DialogueBlock(vec![
            Element::Character(p("STEEL"), blank_attributes()),
            Element::Dialogue(p("Beer's ready!"), blank_attributes()),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("BRICK"), blank_attributes()),
            Element::Dialogue(p("Are they cold?"), blank_attributes()),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("STEEL"), blank_attributes()),
            Element::Parenthetical(p("(beer raised)"), blank_attributes()),
            Element::Dialogue(p("To retirement."), blank_attributes()),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("JACK"), blank_attributes()),
            Element::Parenthetical(p("(in Vietnamese, subtitled)"), blank_attributes()),
            Element::Dialogue(
                Styled(vec![tr(
                    "Did you know Brick and Steel are retired?",
                    vec!["Italic"],
                )]),
                blank_attributes(),
            ),
        ]),
        Element::DialogueBlock(vec![
            Element::Character(p("DAN"), blank_attributes()),
            Element::Dialogue(
                Styled(vec![
                    tr("Then let's retire them.\n", vec![]),
                    tr("Permanently", vec!["Underline"]),
                    tr(".", vec![]),
                ]),
                blank_attributes(),
            ),
        ]),
        Element::Transition(p("CUT TO:"), blank_attributes()),
    ];

    assert_eq!(screenplay.elements, expected);
}

#[test]
fn it_imports_scene_numbers_and_starts_new_page_from_big_fish_fdx() {
    let xml = std::fs::read_to_string(
        "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fdx",
    )
    .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    assert_eq!(
        screenplay.elements[1],
        Element::Action(
            Styled(vec![tr("FADE IN:", vec!["Bold"])]),
            Attributes {
                starts_new_page: true,
                ..Attributes::default()
            },
        )
    );

    let first_numbered_scene = screenplay
        .elements
        .iter()
        .find(|element| {
            matches!(
                element,
                Element::SceneHeading(_, Attributes { scene_number: Some(number), .. })
                    if number == "1"
            )
        })
        .expect("expected first numbered scene heading");
    assert_eq!(
        first_numbered_scene,
        &Element::SceneHeading(
            p("INT.  WILL'S BEDROOM - NIGHT (1973)"),
            Attributes {
                scene_number: Some("1".to_string()),
                ..Attributes::default()
            },
        )
    );

    let second_numbered_scene = screenplay
        .elements
        .iter()
        .find(|element| {
            matches!(
                element,
                Element::SceneHeading(_, Attributes { scene_number: Some(number), .. })
                    if number == "1A"
            )
        })
        .expect("expected second numbered scene heading");
    assert_eq!(
        second_numbered_scene,
        &Element::SceneHeading(
            p("EXT.  CAMPFIRE - NIGHT (1977)"),
            Attributes {
                scene_number: Some("1A".to_string()),
                ..Attributes::default()
            },
        )
    );
}

#[test]
fn it_imports_fdx_dual_dialogue_blocks() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/dual-dialogue-basic.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    let expected = vec![
        Element::Action(p("The men look at each other."), blank_attributes()),
        Element::DualDialogueBlock(vec![
            Element::DialogueBlock(vec![
                Element::Character(p("STEEL"), blank_attributes()),
                Element::Dialogue(p("Screw retirement."), blank_attributes()),
            ]),
            Element::DialogueBlock(vec![
                Element::Character(p("BRICK"), blank_attributes()),
                Element::Dialogue(p("Screw retirement."), blank_attributes()),
            ]),
        ]),
        Element::Transition(p("SMASH CUT TO:"), blank_attributes()),
    ];

    assert_eq!(screenplay.elements, expected);
}

#[test]
fn it_normalizes_imported_fdx_settings_into_shared_layout_metadata() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/settings-normalized.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");
    let profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);

    assert_eq!(profile.style_profile, StyleProfile::Multicam);
    assert_eq!(profile.page_width, 8.26);
    assert_eq!(profile.page_height, 11.69);
    assert_eq!(profile.lines_per_page, 58.0);
    assert_eq!(profile.bottom_margin, 1.2);
    assert_eq!(profile.footer_margin, 0.6);
    assert_eq!(profile.styles.scene_heading.spacing_before, 1.0);
    assert_eq!(profile.styles.dialogue.left_indent, 2.25);
    assert_eq!(profile.styles.dialogue.right_indent, 5.75);
    assert_eq!(profile.styles.dialogue.line_spacing, 2.0);
    assert_eq!(profile.styles.transition.right_indent, 7.25);
    assert_eq!(profile.styles.transition.alignment, Alignment::Right);
}

#[test]
fn it_preserves_richer_imported_layout_overrides_beyond_fmt_metadata() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <Content>
    <Paragraph Type="Action">
      <Text>Centered sample.</Text>
    </Paragraph>
  </Content>
  <ElementSettings Type="Action">
    <FontSpec Style="Bold+Italic"/>
    <ParagraphSpec Alignment="Center" LeftIndent="1.25" RightIndent="7.25" SpaceBefore="24" Spacing="1.5" StartsNewPage="Yes"/>
  </ElementSettings>
  <MoresAndContinueds>
    <DialogueBreaks AutomaticCharacterContinueds="No" BottomOfPage="Yes" DialogueBottom="(MORE)" DialogueTop="(CONT'D)" TopOfNext="Yes"/>
    <SceneBreaks ContinuedNumber="No" SceneBottom="(CONTINUED)" SceneBottomOfPage="No" SceneTop="CONTINUED:" SceneTopOfNext="No"/>
  </MoresAndContinueds>
</FinalDraft>"#;

    let screenplay = parse_fdx(xml).expect("fdx should parse");
    let imported_layout = screenplay
        .imported_layout
        .as_ref()
        .expect("expected imported layout overrides");
    let action_style = imported_layout
        .element_styles
        .get(&ImportedElementKind::Action)
        .expect("expected imported action style");

    assert_eq!(action_style.left_indent, Some(1.25));
    assert_eq!(action_style.first_indent, None);
    assert_eq!(action_style.right_indent, Some(7.25));
    assert_eq!(action_style.spacing_before, Some(2.0));
    assert_eq!(action_style.line_spacing, Some(1.5));
    assert_eq!(action_style.bold, Some(true));
    assert_eq!(action_style.italic, Some(true));
    assert_eq!(
        imported_layout
            .mores_and_continueds
            .dialogue
            .automatic_character_continueds,
        Some(false)
    );

    let resolved = ScreenplayLayoutProfile::from_screenplay(&screenplay);
    assert_eq!(resolved.styles.action.left_indent, 1.25);
    assert_eq!(resolved.styles.action.first_indent, 0.0);
    assert_eq!(resolved.styles.action.right_indent, 7.25);
    assert_eq!(resolved.styles.action.spacing_before, 2.0);
    assert_eq!(resolved.styles.action.line_spacing, 1.5);
    assert_eq!(resolved.styles.action.alignment, Alignment::Center);
    assert!(resolved.styles.action.bold);
    assert!(resolved.styles.action.italic);
    assert!(resolved.styles.action.starts_new_page);
    assert!(!resolved.automatic_character_continueds);
}

#[test]
fn it_imports_parenthetical_first_indent_from_fdx_settings() {
    let xml = std::fs::read_to_string(
        "tests/fixtures/corpus/public/extranormal/source/source.fdx",
    )
    .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");
    let imported_layout = screenplay
        .imported_layout
        .as_ref()
        .expect("expected imported layout overrides");
    let parenthetical_style = imported_layout
        .element_styles
        .get(&ImportedElementKind::Parenthetical)
        .expect("expected imported parenthetical style");

    assert_eq!(parenthetical_style.first_indent, Some(-0.14));

    let resolved = ScreenplayLayoutProfile::from_screenplay(&screenplay);
    assert_eq!(resolved.styles.parenthetical.first_indent, -0.14);
}

#[test]
fn it_imports_title_page_content_into_existing_metadata_keys() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <TitlePage>
    <Content>
      <Paragraph Alignment="Center">
        <Text Style="Bold+Underline">Brick &amp; Steel</Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text Style="Bold+Underline">Full Retired</Text>
      </Paragraph>
      <Paragraph Alignment="Center"><Text></Text></Paragraph>
      <Paragraph Alignment="Center">
        <Text>Written by</Text>
      </Paragraph>
      <Paragraph Alignment="Center"><Text></Text></Paragraph>
      <Paragraph Alignment="Center">
        <Text Style="Italic">Brick</Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text>and Steel</Text>
      </Paragraph>
      <Paragraph Alignment="Center"><Text></Text></Paragraph>
      <Paragraph Alignment="Center">
        <Text AdornmentStyle="-1">Based on a true tab story</Text>
      </Paragraph>
      <Paragraph Alignment="Left">
        <Text>brick@example.com</Text>
      </Paragraph>
      <Paragraph Alignment="Left">
        <Text>555-1234</Text>
        <Text>&#9;</Text>
        <Text>Blue Draft</Text>
      </Paragraph>
      <Paragraph Alignment="Right">
        <Text>April 13, 2026</Text>
      </Paragraph>
    </Content>
  </TitlePage>
  <Content>
    <Paragraph Type="Action"><Text>Body.</Text></Paragraph>
  </Content>
</FinalDraft>"#;

    let screenplay = parse_fdx(xml).expect("fdx should parse");

    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![
            Styled(vec![tr("Brick & Steel", vec!["Bold", "Underline"])]),
            Styled(vec![tr("Full Retired", vec!["Bold", "Underline"])])
        ])
    );
    assert_eq!(screenplay.metadata.get("credit"), Some(&vec!["Written by".into()]));
    assert_eq!(
        screenplay.metadata.get("authors"),
        Some(&vec![
            Styled(vec![tr("Brick", vec!["Italic"])]),
            "and Steel".into()
        ])
    );
    assert_eq!(
        screenplay.metadata.get("source"),
        Some(&vec!["Based on a true tab story".into()])
    );
    assert_eq!(
        screenplay.metadata.get("contact"),
        Some(&vec!["brick@example.com".into(), "555-1234".into()])
    );
    assert_eq!(screenplay.metadata.get("draft"), Some(&vec!["Blue Draft".into()]));
    assert_eq!(
        screenplay.metadata.get("draft date"),
        Some(&vec!["April 13, 2026".into()])
    );

    let title_page = TitlePage::from_metadata(&screenplay.metadata).expect("expected title page");
    assert_eq!(
        title_page.block(TitlePageBlockKind::Title).unwrap().lines,
        screenplay.metadata["title"]
    );
    assert_eq!(
        title_page.block(TitlePageBlockKind::Contact).unwrap().lines,
        screenplay.metadata["contact"]
    );
    assert_eq!(
        title_page.block(TitlePageBlockKind::DraftDate).unwrap().lines,
        screenplay.metadata["draft date"]
    );
}

#[test]
fn it_imports_a_single_credited_author_back_into_author_not_authors() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <TitlePage>
    <Content>
      <Paragraph Alignment="Center">
        <Text Style="Bold+Underline">Brick &amp; Steel</Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text></Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text>Written by</Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text>Stu Maschwitz</Text>
      </Paragraph>
    </Content>
  </TitlePage>
  <Content>
    <Paragraph Type="Action"><Text>Body.</Text></Paragraph>
  </Content>
</FinalDraft>"#;

    let screenplay = parse_fdx(xml).expect("fdx should parse");

    assert_eq!(
        screenplay.metadata.get("author"),
        Some(&vec!["Stu Maschwitz".into()])
    );
    assert!(!screenplay.metadata.contains_key("authors"));
}

#[test]
fn it_imports_public_big_fish_title_page_metadata() {
    let xml = std::fs::read_to_string(
        "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fdx",
    )
    .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![Styled(vec![tr(
            "Big Fish",
            vec!["AllCaps", "Bold", "Underline"]
        )])])
    );
    assert_eq!(
        screenplay.metadata.get("credit"),
        Some(&vec!["written by".into()])
    );
    assert_eq!(
        screenplay.metadata.get("author"),
        Some(&vec!["John August".into()])
    );
    assert_eq!(
        screenplay.metadata.get("source"),
        Some(&vec!["based on the novel by Daniel Wallace".into()])
    );
    assert!(TitlePage::from_metadata(&screenplay.metadata).is_some());
}

#[test]
fn it_imports_public_vikings_title_page_draft_date() {
    let xml =
        std::fs::read_to_string("tests/fixtures/corpus/public/vikings/source/source.fdx")
            .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    assert_eq!(
        screenplay.metadata.get("draft date"),
        Some(&vec!["Axaky 3, 3132".into()])
    );
    assert!(screenplay.metadata.get("source").is_some());
    assert!(TitlePage::from_metadata(&screenplay.metadata).is_some());
}

#[test]
fn it_imports_centered_action_as_structural_act_breaks() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <Content>
    <Paragraph Type="Action" Alignment="Center">
      <Text>COLD OPENING</Text>
    </Paragraph>
    <Paragraph Type="Action" Alignment="Center">
      <Text>END OF PILOT</Text>
    </Paragraph>
    <Paragraph Type="New Act">
      <Text>TAG</Text>
    </Paragraph>
  </Content>
</FinalDraft>"#;

    let screenplay = parse_fdx(xml).expect("fdx should parse");

    assert_eq!(
        screenplay.elements[0],
        Element::ColdOpening(
            p("COLD OPENING"),
            Attributes {
                centered: true,
                ..Attributes::default()
            }
        )
    );
    assert_eq!(
        screenplay.elements[1],
        Element::EndOfAct(
            p("END OF PILOT"),
            Attributes {
                centered: true,
                ..Attributes::default()
            }
        )
    );
    assert_eq!(
        screenplay.elements[2],
        Element::NewAct(
            p("TAG"),
            Attributes {
                centered: true,
                starts_new_page: true, // auto-pagination applies here because we saw ColdOpening
                ..Attributes::default()
            }
        )
    );
}

#[test]
fn it_imports_multi_page_title_page_frontmatter_from_fixture() {
    use jumpcut::title_page::FrontmatterAlignment;

    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/title-pages-multi.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    // Title page 1: standard title page metadata
    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![Styled(vec![tr(
            "WHEN WE WERE VIKINGS",
            vec!["AllCaps", "Bold", "Underline"]
        )])])
    );
    assert_eq!(
        screenplay.metadata.get("credit"),
        Some(&vec!["by".into()])
    );

    // Frontmatter should be captured
    let frontmatter_lines = screenplay
        .metadata
        .get("frontmatter")
        .expect("expected frontmatter metadata key");
    assert!(frontmatter_lines.len() >= 3, "expected at least 3 frontmatter lines");
    assert_eq!(frontmatter_lines[0].plain_text(), "WRITERS' NOTE");

    // Should produce a valid TitlePage with frontmatter
    let title_page = TitlePage::from_metadata(&screenplay.metadata).expect("expected title page");
    assert_eq!(title_page.frontmatter.len(), 1);
    assert_eq!(title_page.frontmatter[0].paragraphs.len(), 3);
    assert_eq!(
        title_page.frontmatter[0].paragraphs[0].text.plain_text(),
        "WRITERS' NOTE"
    );
    assert_eq!(
        title_page.frontmatter[0].paragraphs[0].alignment,
        FrontmatterAlignment::Left
    );
}

#[test]
fn it_imports_frontmatter_from_inline_fdx_with_action_indent_paragraphs() {
    use jumpcut::title_page::FrontmatterAlignment;

    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <TitlePage>
    <Content>
      <Paragraph Alignment="Center">
        <Text Style="Bold+Underline">MY SCREENPLAY</Text>
      </Paragraph>
      <Paragraph Alignment="Center"><Text></Text></Paragraph>
      <Paragraph Alignment="Center">
        <Text>Written by</Text>
      </Paragraph>
      <Paragraph Alignment="Center">
        <Text>Some Author</Text>
      </Paragraph>
      <Paragraph Alignment="Left" LeftIndent="1.50" SpaceBefore="12">
        <Text>A NOTE FROM THE WRITER</Text>
      </Paragraph>
      <Paragraph Alignment="Left" LeftIndent="1.50" SpaceBefore="12">
        <Text>This is a personal note about the screenplay.</Text>
      </Paragraph>
    </Content>
  </TitlePage>
  <Content>
    <Paragraph Type="Action"><Text>Body content.</Text></Paragraph>
  </Content>
</FinalDraft>"#;

    let screenplay = parse_fdx(xml).expect("fdx should parse");

    // Standard title page metadata should still be present
    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![Styled(vec![tr("MY SCREENPLAY", vec!["Bold", "Underline"])])])
    );

    // Frontmatter should be captured
    let frontmatter_lines = screenplay
        .metadata
        .get("frontmatter")
        .expect("expected frontmatter metadata key");
    assert_eq!(frontmatter_lines.len(), 2);
    assert_eq!(frontmatter_lines[0].plain_text(), "A NOTE FROM THE WRITER");
    assert_eq!(
        frontmatter_lines[1].plain_text(),
        "This is a personal note about the screenplay."
    );

    // Should produce a valid TitlePage with frontmatter
    let title_page = TitlePage::from_metadata(&screenplay.metadata).expect("expected title page");
    assert_eq!(title_page.frontmatter.len(), 1);
    assert_eq!(title_page.frontmatter[0].paragraphs.len(), 2);
    assert_eq!(
        title_page.frontmatter[0].paragraphs[0].alignment,
        FrontmatterAlignment::Left
    );
}

#[test]
fn fdx_export_emits_frontmatter_with_starts_new_page() {
    use jumpcut::{Metadata, Screenplay};

    let mut metadata = Metadata::new();
    metadata.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata.insert(
        "frontmatter".into(),
        vec![
            "A NOTE FROM THE WRITER".into(),
            "".into(),
            "This is a personal note.".into(),
        ],
    );

    let mut screenplay = Screenplay {
        metadata,
        imported_layout: None,
        elements: vec![Element::Action(p("Body."), blank_attributes())],
    };

    let fdx = screenplay.to_final_draft();

    // First frontmatter paragraph must have StartsNewPage="Yes"
    // so Final Draft puts it on a new page
    assert!(
        fdx.contains("StartsNewPage=\"Yes\""),
        "FDX output should contain StartsNewPage=Yes for frontmatter"
    );

    // Should contain the frontmatter text content
    assert!(fdx.contains("A NOTE FROM THE WRITER"), "should contain first frontmatter paragraph");
    assert!(fdx.contains("This is a personal note."), "should contain second frontmatter paragraph");

    // The frontmatter should use action-width indents (1.50/7.50), not title-page width (1.00)
    assert!(
        fdx.contains("LeftIndent=\"1.50\"") && fdx.contains("RightIndent=\"7.50\""),
        "frontmatter paragraphs should use action-width indents"
    );
}

#[test]
fn fdx_export_emits_multi_page_frontmatter_with_starts_new_page_on_each_page() {
    use jumpcut::{Metadata, Screenplay};

    let mut metadata = Metadata::new();
    metadata.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata.insert(
        "frontmatter".into(),
        vec![
            "Page A content.".into(),
            "===".into(),
            "Page B content.".into(),
        ],
    );

    let mut screenplay = Screenplay {
        metadata,
        imported_layout: None,
        elements: vec![Element::Action(p("Body."), blank_attributes())],
    };

    let fdx = screenplay.to_final_draft();

    // Both frontmatter pages should exist in the output
    assert!(fdx.contains("Page A content."), "should contain page A");
    assert!(fdx.contains("Page B content."), "should contain page B");

    // Should be importable and roundtrip the frontmatter
    let reimported = parse_fdx(&fdx).expect("should parse back");
    let fm = reimported.metadata.get("frontmatter").expect("should have frontmatter");
    assert!(fm.iter().any(|e| e.plain_text() == "Page A content."), "page A should roundtrip");
    assert!(fm.iter().any(|e| e.plain_text() == "Page B content."), "page B should roundtrip");
}

#[test]
fn title_page_frontmatter_count_is_available_for_pagination_scope() {
    use jumpcut::title_page::TitlePage;
    use jumpcut::Metadata;

    // Script with no frontmatter: title page count contribution from frontmatter = 0
    let mut metadata_no_fm = Metadata::new();
    metadata_no_fm.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    let tp_no_fm = TitlePage::from_metadata(&metadata_no_fm).expect("should have title page");
    assert_eq!(tp_no_fm.frontmatter.len(), 0);

    // Script with 1 frontmatter page: title_page_count should be 2
    let mut metadata_one_fm = Metadata::new();
    metadata_one_fm.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata_one_fm.insert("frontmatter".into(), vec!["A writer's note.".into()]);
    let tp_one_fm = TitlePage::from_metadata(&metadata_one_fm).expect("should have title page");
    assert_eq!(tp_one_fm.frontmatter.len(), 1);
    assert_eq!(tp_one_fm.total_page_count(), 2);

    // Script with 2 frontmatter pages: title_page_count should be 3
    let mut metadata_two_fm = Metadata::new();
    metadata_two_fm.insert("title".into(), vec!["MY SCREENPLAY".into()]);
    metadata_two_fm.insert(
        "frontmatter".into(),
        vec!["Page one.".into(), "===".into(), "Page two.".into()],
    );
    let tp_two_fm = TitlePage::from_metadata(&metadata_two_fm).expect("should have title page");
    assert_eq!(tp_two_fm.frontmatter.len(), 2);
    assert_eq!(tp_two_fm.total_page_count(), 3);
}
