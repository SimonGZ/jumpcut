// tests/pagination_composer_test.rs
use jumpcut::pagination::composer::compose;
use jumpcut::pagination::{
    Cohesion, DialoguePart, DialoguePartKind, DialogueUnit, DualDialogueSide, DualDialogueUnit,
    FlowKind, FlowUnit, LayoutGeometry, PaginationConfig, SemanticUnit,
};
use jumpcut::pagination::{build_semantic_screenplay, normalize_screenplay};
use jumpcut::parse;

fn mock_action(id: &str, text: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.to_string(),
        kind: FlowKind::Action,
        text: text.to_string(),
        inline_text: None,
        render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
        line_range: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

fn mock_scene_heading(id: &str, text: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.to_string(),
        kind: FlowKind::SceneHeading,
        text: text.to_string(),
        inline_text: None,
        render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
        line_range: None,
        cohesion: Cohesion {
            keep_together: true,
            keep_with_next: true,
            can_split: false,
        },
    })
}

fn mock_flow(id: &str, kind: FlowKind, text: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.to_string(),
        kind,
        text: text.to_string(),
        inline_text: None,
        render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
        line_range: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

fn mock_dual_dialogue(
    left_text: &str,
    right_text: &str,
) -> SemanticUnit {
    SemanticUnit::DualDialogue(DualDialogueUnit {
        group_id: "dual-00001".into(),
        sides: vec![
            DualDialogueSide {
                side: 1,
                dialogue: DialogueUnit {
                    block_id: "block-left".into(),
                    parts: vec![
                        DialoguePart {
                            element_id: "el-left-char".into(),
                            kind: DialoguePartKind::Character,
                            text: "LEFT".into(),
                            inline_text: None,
                            render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
                        },
                        DialoguePart {
                            element_id: "el-left-dialogue".into(),
                            kind: DialoguePartKind::Dialogue,
                            text: left_text.into(),
                            inline_text: None,
                            render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
                        },
                    ],
                    cohesion: Cohesion {
                        keep_together: false,
                        keep_with_next: false,
                        can_split: true,
                    },
                },
            },
            DualDialogueSide {
                side: 2,
                dialogue: DialogueUnit {
                    block_id: "block-right".into(),
                    parts: vec![
                        DialoguePart {
                            element_id: "el-right-char".into(),
                            kind: DialoguePartKind::Character,
                            text: "RIGHT".into(),
                            inline_text: None,
                            render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
                        },
                        DialoguePart {
                            element_id: "el-right-dialogue".into(),
                            kind: DialoguePartKind::Dialogue,
                            text: right_text.into(),
                            inline_text: None,
                            render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
                        },
                    ],
                    cohesion: Cohesion {
                        keep_together: false,
                        keep_with_next: false,
                        can_split: true,
                    },
                },
            },
        ],
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

#[test]
fn composer_determines_correct_non_additive_spacing_between_action_blocks() {
    let geometry = LayoutGeometry::default();
    let units = vec![
        mock_action(
            "el-1",
            "This action runs exactly one horizontal semantic line in width.",
        ),
        mock_action("el-2", "This connects."),
    ];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 2);

    // First element receives its intrinsic demanded spacing
    // because the Paginator (Phase 3) is responsible for stripping top-of-page padding.
    assert_eq!(
        blocks[0].spacing_above, 1.0,
        "Action requires 1 blank line above"
    );
    assert_eq!(blocks[0].content_lines, 2.0);

    assert_eq!(
        blocks[1].spacing_above, 1.0,
        "Action -> Action requires exactly 1 blank line (max of 1 and 1)"
    );
    assert_eq!(blocks[1].content_lines, 1.0);
}

#[test]
fn composer_counts_empty_action_as_a_single_blank_line() {
    let geometry = LayoutGeometry::default();
    let units = vec![mock_action("el-empty", "")];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].content_lines, 1.0,
        "an empty action element should still occupy one rendered blank line"
    );
}

#[test]
fn composer_handles_scene_heading_spacing_rules() {
    let geometry = LayoutGeometry::default();
    let units = vec![
        mock_action("el-1", "Start."),
        mock_scene_heading("el-2", "INT. HOUSE - DAY"),
        mock_action("el-3", "End."),
    ];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].content_lines, 1.0);

    // Scene Heading requires 2 lines above
    assert_eq!(
        blocks[1].spacing_above, 2.0,
        "Scene headings require 2 visual blank lines above"
    );
    assert_eq!(blocks[1].content_lines, 1.0);

    // Action requires 1 line above, Scene demands 1 below -> max(1, 1) = 1
    assert_eq!(blocks[2].spacing_above, 1.0);
    assert_eq!(blocks[2].content_lines, 1.0);
}

#[test]
fn composer_respects_custom_geometry() {
    let mut geometry = LayoutGeometry::default();
    // Default action is 1.5 to 7.5 (6.0 inches = 60+1 = 61 chars)
    // Make it much narrower: 1.5 to 3.5 (2.0 inches = 20+1 = 21 chars)
    geometry.action_right = 3.5;

    let text = "This is a long action line that should wrap more frequently.";
    // "This is a long action" -> 21 chars exactly.
    // " line that should wrap" -> 22 chars.
    // " more frequently."
    // Total 3 lines.

    let units = vec![mock_action("el-1", text)];
    let blocks = compose(&units, &geometry);

    assert_eq!(
        blocks[0].content_lines, 3.0,
        "Expected 3 lines with narrower custom geometry"
    );
}

#[test]
fn composer_respects_custom_vertical_spacing() {
    let mut geometry = LayoutGeometry::default();
    // Default action spacing is 1 above. Let's make it 3.
    geometry.action_spacing_before = 3.0;
    
    let units = vec![mock_action("el-1", "Starting action.")];
    let blocks = compose(&units, &geometry);
    
    assert_eq!(blocks[0].spacing_above, 3.0, "Expected 3 lines of spacing above due to custom geometry");
}

#[test]
fn composer_respects_custom_line_height() {
    let mut geometry = LayoutGeometry::default();
    // Default action leading is 1.0. Let's make action specifically 2.0.
    geometry.action_line_height = 2.0;
    
    // An action line that usually takes 2 lines of text.
    let text = "This is a sentence that is long enough to wrap onto a second line in the standard 6.0 inch action width.";
    // Check baseline (1.0 leading)
    let baseline_units = vec![mock_action("el-1", text)];
    let baseline_blocks = compose(&baseline_units, &LayoutGeometry::default());
    assert_eq!(baseline_blocks[0].content_lines, 2.0, "Baseline should be 2 lines");

    // Check custom action leading (2.0 leading)
    let units = vec![mock_action("el-1", text)];
    let blocks = compose(&units, &geometry);
    
    // 2 text lines * 2.0 leading = 4.0 visual lines.
    assert_eq!(
        blocks[0].content_lines, 4.0, 
        "Expected 4 visual lines for double-spaced 2-line text"
    );
}

#[test]
fn composer_respects_1_5_line_height() {
    let mut geometry = LayoutGeometry::default();
    geometry.action_line_height = 1.5;
    
    // 1 text line -> 1.5 visual lines (NO MORE CEIL)
    let units_1 = vec![mock_action("el-1", "Short.")];
    let blocks_1 = compose(&units_1, &geometry);
    assert_eq!(blocks_1[0].content_lines, 1.5, "1 text line @ 1.5 -> 1.5 visual lines");

    // 2 text lines -> 3.0 visual lines
    let text_2 = "This is a sentence that is long enough to wrap onto a second line in the standard 6.0 inch action width.";
    let units_2 = vec![mock_action("el-2", text_2)];
    let blocks_2 = compose(&units_2, &geometry);
    assert_eq!(blocks_2[0].content_lines, 3.0, "2 text lines @ 1.5 -> 3.0 visual lines");

    // Multi-line text -> baseline lines * 1.5 = expected
    let mut geometry_narrow = geometry.clone();
    geometry_narrow.action_right = 3.5; // Narrower
    
    // Baseline check for narrow geometry @ 1.0 leading
    let mut narrow_1_0 = geometry_narrow.clone();
    narrow_1_0.action_line_height = 1.0;
    let units_baseline = vec![mock_action("el-baseline", text_2)];
    let blocks_baseline = compose(&units_baseline, &narrow_1_0);
    let baseline_lines = blocks_baseline[0].content_lines;
    
    let units_3 = vec![mock_action("el-3", text_2)];
    let blocks_3 = compose(&units_3, &geometry_narrow);
    
    // baseline_lines * 1.5 = expected (f32)
    let expected = baseline_lines * 1.5;
    assert_eq!(
        blocks_3[0].content_lines, expected, 
        "Expected {} visual lines for {} text lines @ 1.5", expected, baseline_lines
    );
}

#[test]
fn composer_measures_dual_dialogue_with_special_column_width_and_uses_taller_side() {
    let geometry = LayoutGeometry::default();
    let unit = mock_dual_dialogue(
        "12345678901234567890123456789 12345678901234567890123456789",
        "Short right side.",
    );
    let units = [unit];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].content_lines, 3.0,
        "Left side should measure as 1 character line + 2 dialogue lines at the special dual-dialogue width, and the shared block should take that taller height"
    );
}

#[test]
fn composer_recognizes_dual_dialogue_from_real_parser_output() {
    let screenplay = parse(
        "BRICK\n12345678901234567890123456789 12345678901234567890123456789\n\nSTEEL ^\nShort right side.",
    );
    let normalized = normalize_screenplay("dual", &screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let geometry = LayoutGeometry::default();

    let blocks = compose(&semantic.units, &geometry);

    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0].unit, SemanticUnit::DualDialogue(_)));
    assert_eq!(blocks[0].content_lines, 3.0);
}

#[test]
fn composer_uses_explicit_multicam_flow_geometry_instead_of_action_fallbacks() {
    let mut geometry = LayoutGeometry::default();
    geometry.cold_opening_left = 1.0;
    geometry.cold_opening_right = 7.5;
    geometry.end_of_act_spacing_before = 2.0;

    let cold_opening_text =
        "12345678901234567890123456789012345678901234567890123456789012345";
    let units = vec![
        mock_flow("el-cold", FlowKind::ColdOpening, cold_opening_text),
        mock_flow("el-end", FlowKind::EndOfAct, "END ACT ONE"),
    ];

    let blocks = compose(&units, &geometry);

    assert_eq!(
        blocks[0].content_lines, 1.0,
        "Cold Opening should use its own wider geometry instead of Action fallback."
    );
    assert_eq!(
        blocks[1].spacing_above, 2.0,
        "End of Act should use its own spacing instead of Action fallback."
    );
}

#[test]
fn composer_multicam_keeps_action_single_spaced() {
    let screenplay = parse("Fmt: multicam\n\nA short action line.\n");
    let normalized = normalize_screenplay("multicam-action", &screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);

    let blocks = compose(&semantic.units, &config.geometry);

    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0].unit, SemanticUnit::Flow(_)));
    assert_eq!(
        blocks[0].content_lines, 1.0,
        "Multicam should not double-space action lines."
    );
}

#[test]
fn composer_multicam_only_doubles_dialogue_lines_inside_dialogue_blocks() {
    let screenplay = parse("Fmt: multicam\n\nALICE\nHello there.\n");
    let normalized = normalize_screenplay("multicam-dialogue", &screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);

    let blocks = compose(&semantic.units, &config.geometry);

    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0].unit, SemanticUnit::Dialogue(_)));
    assert_eq!(
        blocks[0].content_lines, 3.0,
        "A one-line character cue plus one double-spaced dialogue line should measure as 3.0 visual lines, not 4.0."
    );
}
