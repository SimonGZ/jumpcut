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
        screenplay.metadata.get("authors"),
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
