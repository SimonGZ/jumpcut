use jumpcut::pagination::{
    build_semantic_screenplay, build_semantic_screenplay_with_options, normalize_screenplay,
    Cohesion, DialoguePartKind, FlowKind, NormalizedElement, NormalizedScreenplay, SemanticOptions,
    SemanticUnit,
};
use jumpcut::render_attributes::RenderAttributes;
use jumpcut::styled_text::{StyledRun, StyledText};
use jumpcut::{blank_attributes, p, parse, tr, Attributes, Element, ElementText, Screenplay};
use pretty_assertions::assert_eq;

#[test]
fn it_inserts_page_start_units_before_forced_new_pages() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Scene Heading", "INT. OFFICE - DAY")
                .with_starts_new_page(true),
            normalized_element("el-00002", "Action", "Papers everywhere."),
        ],
    });

    assert_eq!(semantic.units.len(), 3);

    match &semantic.units[0] {
        SemanticUnit::PageStart(unit) => assert_eq!(unit.source_element_id, "el-00001"),
        other => panic!("expected page start, got {other:?}"),
    }

    match &semantic.units[1] {
        SemanticUnit::Flow(unit) => {
            assert_eq!(unit.kind, FlowKind::SceneHeading);
            assert_eq!(
                unit.cohesion,
                Cohesion {
                    keep_together: true,
                    keep_with_next: true,
                    can_split: false,
                }
            );
        }
        other => panic!("expected flow unit, got {other:?}"),
    }
}

#[test]
fn it_groups_dialogue_and_lyric_blocks_into_dialogue_units() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Character", "SINGER")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
            normalized_element("el-00002", "Dialogue", "How does this sound?")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
            normalized_element("el-00003", "Lyric", "Loves Chocolate!")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
        ],
    });

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::Dialogue(unit) => {
            assert_eq!(unit.block_id, "block-00001");
            assert_eq!(unit.parts.len(), 3);
            assert_eq!(unit.parts[0].kind, DialoguePartKind::Character);
            assert_eq!(unit.parts[1].kind, DialoguePartKind::Dialogue);
            assert_eq!(unit.parts[2].kind, DialoguePartKind::Lyric);
            assert!(!unit.should_append_contd);
            assert_eq!(
                unit.cohesion,
                Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                }
            );
        }
        other => panic!("expected dialogue unit, got {other:?}"),
    }
}

#[test]
fn it_groups_dual_dialogue_into_a_single_dual_dialogue_unit() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Character", "LEFT")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 1),
            normalized_element("el-00002", "Dialogue", "First line.")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 1),
            normalized_element("el-00003", "Character", "RIGHT")
                .in_block("block-00002")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 2),
            normalized_element("el-00004", "Dialogue", "Second line.")
                .in_block("block-00002")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 2),
        ],
    });

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::DualDialogue(unit) => {
            assert_eq!(unit.group_id, "dual-00001");
            assert_eq!(unit.sides.len(), 2);
            assert_eq!(unit.sides[0].side, 1);
            assert_eq!(unit.sides[0].dialogue.parts.len(), 2);
            assert_eq!(unit.sides[1].side, 2);
            assert_eq!(unit.sides[1].dialogue.parts.len(), 2);
        }
        other => panic!("expected dual dialogue unit, got {other:?}"),
    }
}

#[test]
fn it_creates_standalone_lyric_units_from_normalized_parser_output() {
    let screenplay = parse("~Willy Wonka! Willy Wonka!");
    let normalized = normalize_screenplay("lyric", &screenplay);
    let semantic = build_semantic_screenplay(normalized);

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::Lyric(unit) => {
            assert_eq!(unit.element_id, "el-00001");
            assert_eq!(unit.text, "Willy Wonka! Willy Wonka!");
            assert_eq!(unit.cohesion.can_split, true);
        }
        other => panic!("expected lyric unit, got {other:?}"),
    }
}

#[test]
fn parsed_dual_dialogue_is_normalized_and_grouped_as_a_dual_dialogue_unit() {
    let screenplay = parse("BRICK\nLeft side.\n\nSTEEL ^\nRight side.");
    let normalized = normalize_screenplay("dual", &screenplay);

    assert_eq!(normalized.elements.len(), 4);
    assert_eq!(
        normalized.elements[0].dual_dialogue_group,
        normalized.elements[1].dual_dialogue_group
    );
    assert_eq!(
        normalized.elements[2].dual_dialogue_group,
        normalized.elements[3].dual_dialogue_group
    );
    assert_eq!(
        normalized.elements[0].dual_dialogue_group,
        normalized.elements[2].dual_dialogue_group
    );
    assert_eq!(normalized.elements[0].dual_dialogue_side, Some(1));
    assert_eq!(normalized.elements[1].dual_dialogue_side, Some(1));
    assert_eq!(normalized.elements[2].dual_dialogue_side, Some(2));
    assert_eq!(normalized.elements[3].dual_dialogue_side, Some(2));

    let semantic = build_semantic_screenplay(normalized);

    assert_eq!(semantic.units.len(), 1);
    match &semantic.units[0] {
        SemanticUnit::DualDialogue(unit) => {
            assert_eq!(unit.sides.len(), 2);
            assert_eq!(unit.sides[0].side, 1);
            assert_eq!(unit.sides[1].side, 2);
        }
        other => panic!("expected dual dialogue unit, got {other:?}"),
    }
}

#[test]
fn styled_runs_survive_normalization_and_semantic_building() {
    let screenplay = Screenplay {
        metadata: Default::default(),
        imported_layout: None,
        imported_title_page: None,
        elements: vec![
            Element::Action(
                ElementText::Styled(vec![tr("BOLD", vec!["Bold"]), tr(" plain", vec![])]),
                blank_attributes(),
            ),
            Element::DialogueBlock(vec![
                Element::Character(
                    ElementText::Styled(vec![tr("ALICE", vec!["Underline"])]),
                    blank_attributes(),
                ),
                Element::Dialogue(
                    ElementText::Styled(vec![tr("Hello", vec!["Italic"])]),
                    blank_attributes(),
                ),
            ]),
        ],
    };

    let normalized = normalize_screenplay("styled", &screenplay);

    assert_eq!(
        normalized.elements[0].inline_text,
        Some(StyledText {
            plain_text: "BOLD plain".into(),
            runs: vec![
                StyledRun {
                    text: "BOLD".into(),
                    styles: vec!["Bold".into()],
                },
                StyledRun {
                    text: " plain".into(),
                    styles: vec![],
                },
            ],
        })
    );
    assert_eq!(
        normalized.elements[2].inline_text,
        Some(StyledText {
            plain_text: "Hello".into(),
            runs: vec![StyledRun {
                text: "Hello".into(),
                styles: vec!["Italic".into()],
            }],
        })
    );

    let semantic = build_semantic_screenplay(normalized);

    match &semantic.units[0] {
        SemanticUnit::Flow(unit) => {
            assert_eq!(unit.inline_text, normalized_action_inline_text());
        }
        other => panic!("expected flow unit, got {other:?}"),
    }

    match &semantic.units[1] {
        SemanticUnit::Dialogue(unit) => {
            assert_eq!(
                unit.parts[0].inline_text,
                normalized_character_inline_text()
            );
            assert_eq!(unit.parts[1].inline_text, normalized_dialogue_inline_text());
        }
        other => panic!("expected dialogue unit, got {other:?}"),
    }
}

#[test]
fn centered_flag_survives_normalization_and_semantic_building() {
    let screenplay = Screenplay {
        metadata: Default::default(),
        imported_layout: None,
        imported_title_page: None,
        elements: vec![Element::Action(
            p("THE END"),
            Attributes {
                centered: true,
                ..blank_attributes()
            },
        )],
    };

    let normalized = normalize_screenplay("centered", &screenplay);
    assert!(normalized.elements[0].render_attributes.centered);

    let semantic = build_semantic_screenplay(normalized);
    match &semantic.units[0] {
        SemanticUnit::Flow(unit) => assert!(unit.render_attributes.centered),
        other => panic!("expected flow unit, got {other:?}"),
    }
}

#[test]
fn render_attributes_survive_normalization_and_semantic_building() {
    let screenplay = Screenplay {
        metadata: Default::default(),
        imported_layout: None,
        imported_title_page: None,
        elements: vec![Element::SceneHeading(
            p("INT. OFFICE - DAY"),
            Attributes {
                centered: true,
                starts_new_page: true,
                scene_number: Some("12".into()),
                ..blank_attributes()
            },
        )],
    };

    let normalized = normalize_screenplay("render-attrs", &screenplay);
    assert_eq!(
        normalized.elements[0].render_attributes,
        RenderAttributes {
            centered: true,
            starts_new_page: true,
            scene_number: Some("12".into()),
            ..RenderAttributes::default()
        }
    );

    let semantic = build_semantic_screenplay(normalized);
    match &semantic.units[1] {
        SemanticUnit::Flow(unit) => {
            assert_eq!(
                unit.render_attributes,
                RenderAttributes {
                    centered: true,
                    starts_new_page: true,
                    scene_number: Some("12".into()),
                    ..RenderAttributes::default()
                }
            );
        }
        other => panic!("expected flow unit, got {other:?}"),
    }
}

#[test]
fn resumed_same_scene_dialogue_adds_contd_after_action_but_not_after_another_speaker() {
    let screenplay = parse(
        "INT. OFFICE - DAY\n\nEDWARD\nFirst line.\n\nA beat.\n\nEDWARD\nSecond line.\n\nLITTLE BRAVE\nDifferent speaker.\n\nEDWARD\nThird line.",
    );

    let semantic = build_semantic_screenplay(normalize_screenplay("contd", &screenplay));
    let dialogue_units = semantic
        .units
        .iter()
        .filter_map(|unit| match unit {
            SemanticUnit::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(dialogue_units.len(), 4);
    assert!(!dialogue_units[0].should_append_contd);
    assert!(dialogue_units[1].should_append_contd);
    assert!(!dialogue_units[2].should_append_contd);
    assert!(!dialogue_units[3].should_append_contd);
}

#[test]
fn voice_over_dialogue_does_not_trigger_or_receive_contd() {
    let screenplay = parse(
        "INT. OFFICE - DAY\n\nEDWARD (V.O.)\nFirst line.\n\nA beat.\n\nEDWARD (V.O.)\nSecond line.",
    );

    let semantic = build_semantic_screenplay(normalize_screenplay("contd-vo", &screenplay));
    let dialogue_units = semantic
        .units
        .iter()
        .filter_map(|unit| match unit {
            SemanticUnit::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(dialogue_units.len(), 2);
    assert!(!dialogue_units[0].should_append_contd);
    assert!(!dialogue_units[1].should_append_contd);
}

#[test]
fn speaker_extensions_other_than_voice_over_still_match_for_contd() {
    let screenplay = parse(
        "INT. OFFICE - DAY\n\nJOSEPHINE\nFirst line.\n\nA beat.\n\nJOSEPHINE (O.S.)\nSecond line.",
    );

    let semantic = build_semantic_screenplay(normalize_screenplay("contd-os", &screenplay));
    let dialogue_units = semantic
        .units
        .iter()
        .filter_map(|unit| match unit {
            SemanticUnit::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(dialogue_units.len(), 2);
    assert!(!dialogue_units[0].should_append_contd);
    assert!(dialogue_units[1].should_append_contd);
}

#[test]
fn dual_dialogue_contd_flags_follow_final_draft_side_state() {
    let screenplay = parse(
        "INT. CARRIAGE - DAY\n\nAMY\nSTOP THE CARRIAGE!\n\nLAURIE\nAMY! You're so /grown up!\n\nAMY ^\n/You wrote you'd come to the hotel!\n\nLAURIE\nI looked for you and couldn't find /you anywhere!\n\nAMY ^\n/You didn't look hard enough!",
    );

    let semantic = build_semantic_screenplay(normalize_screenplay("dual-contd", &screenplay));
    let dual = semantic
        .units
        .iter()
        .find_map(|unit| match unit {
            SemanticUnit::DualDialogue(dual) => Some(dual),
            _ => None,
        })
        .expect("expected dual dialogue unit");

    let left_flags = dual
        .sides
        .iter()
        .find(|side| side.side == 1)
        .expect("expected left side")
        .dialogue
        .parts
        .iter()
        .filter(|part| part.kind == DialoguePartKind::Character)
        .map(|part| part.should_append_contd)
        .collect::<Vec<_>>();
    let right_flags = dual
        .sides
        .iter()
        .find(|side| side.side == 2)
        .expect("expected right side")
        .dialogue
        .parts
        .iter()
        .filter(|part| part.kind == DialoguePartKind::Character)
        .map(|part| part.should_append_contd)
        .collect::<Vec<_>>();

    assert_eq!(left_flags, vec![false]);
    assert_eq!(right_flags, vec![true]);
}

#[test]
fn dual_dialogue_contd_eligibility_can_be_disabled_for_balanced_mode() {
    let screenplay = parse(
        "INT. CARRIAGE - DAY\n\nLAURIE\nHello /there.\n\nAMY ^\n/Hi.\n\nLAURIE\nStill talking.\n\nAMY\nMe too.",
    );

    let semantic = build_semantic_screenplay_with_options(
        normalize_screenplay("dual-contd-balanced", &screenplay),
        SemanticOptions {
            dual_dialogue_counts_for_contd: false,
            automatic_character_continueds: true,
        },
    );
    let dialogue_units = semantic
        .units
        .iter()
        .filter_map(|unit| match unit {
            SemanticUnit::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(dialogue_units.len(), 2);
    assert!(!dialogue_units[0].should_append_contd);
    assert!(!dialogue_units[1].should_append_contd);
}

fn normalized_element(element_id: &str, kind: &str, text: &str) -> NormalizedElement {
    NormalizedElement {
        element_id: element_id.into(),
        kind: kind.into(),
        text: text.into(),
        inline_text: None,
        render_attributes: RenderAttributes::default(),
        fragment: None,
        block_kind: None,
        block_id: None,
        dual_dialogue_group: None,
        dual_dialogue_side: None,
    }
}

fn normalized_action_inline_text() -> Option<StyledText> {
    Some(StyledText {
        plain_text: "BOLD plain".into(),
        runs: vec![
            StyledRun {
                text: "BOLD".into(),
                styles: vec!["Bold".into()],
            },
            StyledRun {
                text: " plain".into(),
                styles: vec![],
            },
        ],
    })
}

fn normalized_character_inline_text() -> Option<StyledText> {
    Some(StyledText {
        plain_text: "ALICE".into(),
        runs: vec![StyledRun {
            text: "ALICE".into(),
            styles: vec!["Underline".into()],
        }],
    })
}

fn normalized_dialogue_inline_text() -> Option<StyledText> {
    Some(StyledText {
        plain_text: "Hello".into(),
        runs: vec![StyledRun {
            text: "Hello".into(),
            styles: vec!["Italic".into()],
        }],
    })
}

trait NormalizedElementExt {
    fn with_starts_new_page(self, starts_new_page: bool) -> Self;
    fn with_block_kind(self, block_kind: &str) -> Self;
    fn in_block(self, block_id: &str) -> Self;
    fn in_dual(self, group_id: &str, side: u8) -> Self;
}

impl NormalizedElementExt for NormalizedElement {
    fn with_starts_new_page(mut self, starts_new_page: bool) -> Self {
        self.render_attributes.starts_new_page = starts_new_page;
        self
    }

    fn with_block_kind(mut self, block_kind: &str) -> Self {
        self.block_kind = Some(block_kind.into());
        self
    }

    fn in_block(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    fn in_dual(mut self, group_id: &str, side: u8) -> Self {
        self.dual_dialogue_group = Some(group_id.into());
        self.dual_dialogue_side = Some(side);
        self
    }
}
