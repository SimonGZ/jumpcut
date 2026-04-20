use jumpcut::pagination::{Alignment, ScreenplayLayoutProfile, StyleProfile};
use jumpcut::title_page::{TitlePage, TitlePageBlockKind};
use jumpcut::{
    blank_attributes, p, parse_fdx, Attributes, Element, ElementText::Styled, ImportedElementKind,
    ImportedTitlePageAlignment, ImportedTitlePageTabStopKind, TextRun,
};
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
    let xml = std::fs::read_to_string("tests/fixtures/corpus/public/extranormal/source/source.fdx")
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
fn it_preserves_body_paragraph_layout_deviations_as_element_overrides() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/paragraph-layout-overrides.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");
    let imported_layout = screenplay
        .imported_layout
        .as_ref()
        .expect("expected imported layout baseline");
    let action_style = imported_layout
        .element_styles
        .get(&ImportedElementKind::Action)
        .expect("expected imported action style");

    assert_eq!(action_style.right_indent, Some(7.5));
    assert_eq!(action_style.spacing_before, Some(1.0));
    assert_eq!(
        imported_layout.element_styles.get(&ImportedElementKind::Dialogue),
        None
    );

    assert_eq!(
        screenplay.elements,
        vec![
            Element::Action(
                p("Raised action."),
                Attributes {
                    layout_overrides: jumpcut::ElementLayoutOverrides {
                        space_before_delta: Some(0.5),
                        ..Default::default()
                    },
                    ..Attributes::default()
                }
            ),
            Element::DialogueBlock(vec![
                Element::Character(p("JANE"), Attributes::default()),
                Element::Dialogue(
                    p("Hello there."),
                    Attributes {
                        layout_overrides: jumpcut::ElementLayoutOverrides {
                            right_indent_delta: Some(0.25),
                            ..Default::default()
                        },
                        ..Attributes::default()
                    }
                ),
            ]),
        ]
    );

    let resolved = ScreenplayLayoutProfile::from_screenplay(&screenplay);
    assert_eq!(resolved.styles.action.spacing_before, 1.0);
    assert_eq!(resolved.styles.dialogue.right_indent, 6.0);
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
    assert_eq!(
        screenplay.metadata.get("credit"),
        Some(&vec!["Written by".into()])
    );
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
    assert_eq!(
        screenplay.metadata.get("draft"),
        Some(&vec!["Blue Draft".into()])
    );
    assert_eq!(
        screenplay.metadata.get("draft date"),
        Some(&vec!["April 13, 2026".into()])
    );

    let title_page = TitlePage::from_screenplay(&screenplay).expect("expected title page");
    assert_eq!(
        title_page.block(TitlePageBlockKind::Title).unwrap().lines,
        screenplay.metadata["title"]
    );
    assert_eq!(
        title_page.block(TitlePageBlockKind::Contact).unwrap().lines,
        screenplay.metadata["contact"]
    );
    assert_eq!(
        title_page
            .block(TitlePageBlockKind::DraftDate)
            .unwrap()
            .lines,
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
    let xml = std::fs::read_to_string("tests/fixtures/corpus/public/vikings/source/source.fdx")
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

    let imported_title_page = screenplay
        .imported_title_page
        .as_ref()
        .expect("expected imported title-page pages");
    assert_eq!(imported_title_page.pages.len(), 2);
    assert!(
        imported_title_page.pages[1]
            .paragraphs
            .iter()
            .take_while(|paragraph| paragraph.text.plain_text().trim().is_empty())
            .count()
            >= 10,
        "expected page two to preserve the leading blank paragraphs above WRITERS' NOTE"
    );
    assert_eq!(
        imported_title_page.pages[1]
            .paragraphs
            .iter()
            .find(|paragraph| !paragraph.text.plain_text().trim().is_empty())
            .expect("expected first nonblank overflow paragraph")
            .text
            .plain_text(),
        "WRITERS' NOTE"
    );
    assert_eq!(
        imported_title_page.pages[1]
            .paragraphs
            .iter()
            .find(|paragraph| !paragraph.text.plain_text().trim().is_empty())
            .expect("expected first nonblank overflow paragraph")
            .alignment,
        ImportedTitlePageAlignment::Left
    );

    // Title page 1: standard title page metadata
    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![Styled(vec![tr(
            "WHEN WE WERE VIKINGS",
            vec!["AllCaps", "Bold", "Underline"]
        )])])
    );
    assert_eq!(screenplay.metadata.get("credit"), Some(&vec!["by".into()]));

    assert!(!screenplay.metadata.contains_key("frontmatter"));

    // Should produce a valid TitlePage with frontmatter
    let title_page = TitlePage::from_screenplay(&screenplay).expect("expected title page");
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
fn it_preserves_title_page_overflow_pages_without_promoting_centered_page_two_content() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");

    assert_eq!(
        screenplay.metadata.get("title"),
        Some(&vec![
            Styled(vec![tr("GUY TEXT", vec!["AllCaps", "Bold", "Underline"])]),
            "Pilot: \"The Power Betas\"".into(),
        ])
    );
    assert_eq!(screenplay.metadata.get("credit"), Some(&vec!["by".into()]));
    assert_eq!(
        screenplay.metadata.get("author"),
        Some(&vec!["Aaron Brownstein & Simon Ganz".into()])
    );
    assert!(!screenplay.metadata.contains_key("source"));

    assert!(
        screenplay
            .metadata
            .values()
            .flatten()
            .all(|value| value.plain_text() != "THE GUYS"),
        "page-two centered heading should remain preserved content, not semantic metadata"
    );

    let imported_title_page = screenplay
        .imported_title_page
        .as_ref()
        .expect("expected imported title-page pages");
    assert!(!imported_title_page.header_footer.header_has_page_number);
    assert!(imported_title_page.header_footer.header_visible);
    assert!(!imported_title_page.header_footer.header_first_page);
    assert_eq!(imported_title_page.header_footer.starting_page, Some(1));
    assert_eq!(imported_title_page.pages.len(), 2);
    assert_eq!(
        imported_title_page.pages[1].paragraphs[0].text.plain_text(),
        "THE GUYS"
    );
    assert_eq!(
        imported_title_page.pages[1].paragraphs[0].alignment,
        ImportedTitlePageAlignment::Center
    );
    assert_eq!(
        imported_title_page.pages[1].paragraphs[3].tab_stops
            .iter()
            .map(|tab_stop| (tab_stop.position, tab_stop.kind))
            .collect::<Vec<_>>(),
        vec![
            (6.0, ImportedTitlePageTabStopKind::Left),
            (2.0, ImportedTitlePageTabStopKind::Left),
            (1.82, ImportedTitlePageTabStopKind::Left),
            (2.32, ImportedTitlePageTabStopKind::Left),
        ]
    );
    assert_eq!(
        imported_title_page.pages[1].paragraphs[3].first_indent,
        Some(-1.06)
    );

    assert!(!screenplay.metadata.contains_key("frontmatter"));
}

#[test]
fn it_imports_title_page_header_page_number_signal_for_multi_page_fixture() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/title-pages-multi.fdx")
        .expect("fixture should load");

    let screenplay = parse_fdx(&xml).expect("fdx should parse");
    let imported_title_page = screenplay
        .imported_title_page
        .as_ref()
        .expect("expected imported title-page pages");

    assert!(imported_title_page.header_footer.header_visible);
    assert!(!imported_title_page.header_footer.header_first_page);
    assert!(imported_title_page.header_footer.header_has_page_number);
    assert_eq!(imported_title_page.header_footer.starting_page, Some(1));
}

#[test]
fn it_imports_frontmatter_from_inline_fdx_with_action_indent_paragraphs() {
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
        Some(&vec![Styled(vec![tr(
            "MY SCREENPLAY",
            vec!["Bold", "Underline"]
        )])])
    );

    assert!(!screenplay.metadata.contains_key("frontmatter"));
    let imported_title_page = screenplay
        .imported_title_page
        .as_ref()
        .expect("expected imported title page");
    assert_eq!(imported_title_page.pages.len(), 1);
    let page_one_extra_paragraphs = imported_title_page.pages[0]
        .paragraphs
        .iter()
        .filter(|paragraph| paragraph.alignment == ImportedTitlePageAlignment::Left)
        .map(|paragraph| paragraph.text.plain_text())
        .collect::<Vec<_>>();
    assert!(page_one_extra_paragraphs.contains(&"A NOTE FROM THE WRITER".to_string()));
    assert!(
        page_one_extra_paragraphs
            .contains(&"This is a personal note about the screenplay.".to_string())
    );
}

#[test]
fn fdx_export_preserves_imported_title_page_overflow_pages() {
    let xml = std::fs::read_to_string("tests/fixtures/fdx-import/title-page-cast-page.fdx")
        .expect("fixture should load");

    let mut screenplay = parse_fdx(&xml).expect("fdx should parse");
    let fdx = screenplay.to_final_draft();

    assert!(
        fdx.contains("THE GUYS"),
        "should preserve centered page-two heading"
    );
    assert!(fdx.contains("ALAN"), "should preserve cast-page content");
    assert!(
        fdx.contains("<Tabstop Position=\"2.00\" Type=\"Left\"/>"),
        "should preserve imported title-page tab stops"
    );
    assert!(
        fdx.contains("FirstIndent=\"-1.06\""),
        "should preserve imported title-page first indents"
    );

    let reimported = parse_fdx(&fdx).expect("round-tripped fdx should parse");
    let imported_title_page = reimported
        .imported_title_page
        .as_ref()
        .expect("expected imported title-page pages after round-trip");
    assert_eq!(imported_title_page.pages.len(), 2);
    assert_eq!(
        imported_title_page.pages[1]
            .paragraphs
            .iter()
            .find(|paragraph| !paragraph.text.plain_text().trim().is_empty())
            .expect("expected first nonblank overflow paragraph after round-trip")
            .text
            .plain_text(),
        "THE GUYS"
    );
    let alan_paragraph = imported_title_page.pages[1]
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.text.plain_text().starts_with("ALAN\t\tLate 30s"))
        .expect("expected ALAN cast paragraph after round-trip");
    assert_eq!(
        alan_paragraph
            .tab_stops
            .iter()
            .map(|tab_stop| (tab_stop.position, tab_stop.kind))
            .collect::<Vec<_>>(),
        vec![
            (6.0, ImportedTitlePageTabStopKind::Left),
            (2.0, ImportedTitlePageTabStopKind::Left),
            (1.82, ImportedTitlePageTabStopKind::Left),
            (2.32, ImportedTitlePageTabStopKind::Left),
        ]
    );
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

}
